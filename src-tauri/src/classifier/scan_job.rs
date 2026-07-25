//! MIG-021v2 §1F' — Background scan that classifies every note in the
//! universe whose `sources:` and/or `content_type:` are not yet set
//! and which doesn't already have a pending suggestion in the queue.
//!
//! Resumability is implicit: the queue rows + frontmatter ARE the cursor.
//! If the user closes Constellation mid-scan, the next start() re-walks
//! `note_meta`, skips notes already classified or already in the queue,
//! and resumes on the residual.
//!
//! Cancellation is cooperative: a per-note check on the cancel flag
//! between embeddings. Cost: one atomic load per note (~2 ns).
//!
//! Per Performance Rule 1 + 3: the loop runs on a background thread,
//! never on the main UI thread.
//!
//! COST — corrected 2026-07-24. This header used to claim "~30ms per note …
//! a 7,000-note universe takes ~3.5 minutes". Measured on a real 7,339-note
//! Universe the scan took **over an hour**, and the estimate was never revisited.
//! The per-note database work alone measured ~155 ms warm (~19 min across the
//! Universe) and far worse cold. Two of the three causes are fixed here and in
//! `cece/wiring.rs` (the writer-lock grabs and the missing yield); the structural
//! one — reloading the whole classified-neighbour set once per note rather than
//! once per scan — is PJ-144 and needs its own migration. **Do not restore a
//! throughput claim to this comment without measuring it on a large Universe.**

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Hand the machine back between notes. Matches the sibling background jobs
/// (`links_backfill`, `note_body_backfill`, `sky_backfill`, `review_backfill`,
/// `props_reparse_backfill` at 50 ms; `nsc/backfill` at 30 ms). Without this the
/// scan is an hour of uninterrupted disk + lock pressure with no gap for anything
/// else on the system.
const INTER_NOTE_SLEEP_MS: u64 = 30;

#[derive(Default)]
pub struct ScanState {
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    pub completed: AtomicUsize,
    pub total: AtomicUsize,
    pub last_error: Mutex<Option<String>>,
}

impl ScanState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Returned by `classifier_scan_status` so the frontend can render
/// the progress strip without listening for events (e.g. on first
/// boot, before any event has fired).
#[derive(Serialize, Clone)]
pub struct ScanStatus {
    pub running: bool,
    pub cancelling: bool,
    pub completed: usize,
    pub total: usize,
    pub last_error: Option<String>,
}

/// Tauri event payload — identical shape to the one
/// `MigrationProgressStrip` consumes, so the frontend strip can be
/// modeled after the same pattern.
#[derive(Serialize, Clone)]
struct ScanProgressEvent {
    phase: String,
    total: usize,
    completed: usize,
    error: Option<String>,
}

/// Snapshot the scan state. Callable any time, including before/after
/// a scan run, to populate the UI on mount or recover from a missed
/// event.
#[tauri::command]
pub fn classifier_scan_status(app: AppHandle) -> ScanStatus {
    let state = app.state::<ScanState>();
    ScanStatus {
        running: state.running.load(Ordering::Relaxed),
        cancelling: state.cancel.load(Ordering::Relaxed),
        completed: state.completed.load(Ordering::Relaxed),
        total: state.total.load(Ordering::Relaxed),
        last_error: state
            .last_error
            .lock()
            .ok()
            .and_then(|g| g.clone()),
    }
}

/// Set the cancel flag. The scan loop checks this between each note
/// and exits cleanly with a `cancelled` event. Returns immediately;
/// the actual scan thread observes the flag on its next iteration.
#[tauri::command]
pub fn classifier_scan_cancel(app: AppHandle) -> Result<(), String> {
    let state = app.state::<ScanState>();
    state.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

/// Kick off the background scan. Idempotent: returns an error if a
/// scan is already running. Spawns a worker thread; the main thread
/// returns immediately.
// App-freeze audit Batch-D (2026-07-03): ensure_search_db_ready moved INSIDE
// the spawned worker — as the command's first statement it parked the dispatch
// thread for the whole 20-40s cold init (and this command AUTO-FIRES at boot+5s
// when cece.backgroundScan='on_startup' → a guaranteed mid-boot freeze). The
// worker's existing error path (last_error + error event + flag reset) now
// covers an ensure failure too.
#[tauri::command]
pub fn classifier_scan_start(app: AppHandle) -> Result<(), String> {
    let state = app.state::<ScanState>();

    // swap returns the previous value — if it was already true,
    // someone beat us to it.
    if state.running.swap(true, Ordering::Relaxed) {
        return Err("Scan already running".into());
    }
    state.cancel.store(false, Ordering::Relaxed);
    state.completed.store(0, Ordering::Relaxed);
    state.total.store(0, Ordering::Relaxed);
    if let Ok(mut g) = state.last_error.lock() {
        *g = None;
    }

    let app_clone = app.clone();
    thread::spawn(move || {
        let result = crate::search::ensure_search_db_ready(&app_clone)
            .and_then(|_| run_scan(app_clone.clone()));
        let state = app_clone.state::<ScanState>();
        if let Err(e) = result {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(e.clone());
            }
            let _ = app_clone.emit(
                "classifier:scan",
                ScanProgressEvent {
                    phase: "error".into(),
                    total: state.total.load(Ordering::Relaxed),
                    completed: state.completed.load(Ordering::Relaxed),
                    error: Some(e),
                },
            );
        }
        state.running.store(false, Ordering::Relaxed);
        state.cancel.store(false, Ordering::Relaxed);
    });

    Ok(())
}

fn run_scan(app: AppHandle) -> Result<(), String> {
    let pending = enumerate_pending(&app)?;
    let total = pending.len();
    let state = app.state::<ScanState>();
    state.total.store(total, Ordering::Relaxed);

    let _ = app.emit(
        "classifier:scan",
        ScanProgressEvent {
            phase: "start".into(),
            total,
            completed: 0,
            error: None,
        },
    );

    if total == 0 {
        let _ = app.emit(
            "classifier:scan",
            ScanProgressEvent {
                phase: "done".into(),
                total: 0,
                completed: 0,
                error: None,
            },
        );
        return Ok(());
    }

    let mut completed: usize = 0;
    for note_path in pending {
        if state.cancel.load(Ordering::Relaxed) {
            let _ = app.emit(
                "classifier:scan",
                ScanProgressEvent {
                    phase: "cancelled".into(),
                    total,
                    completed,
                    error: None,
                },
            );
            return Ok(());
        }

        // Per-note errors are recorded in last_error but do NOT abort
        // the scan — one corrupt note shouldn't stop classification of
        // the other 6,999.
        let res = super::classifier_suggest_for_note(app.clone(), note_path.clone());
        if let Err(e) = res {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(format!("{}: {}", note_path, e));
            }
        }
        completed += 1;
        state.completed.store(completed, Ordering::Relaxed);

        // Throttle event emission: every 5 notes + always at the end.
        // Per Performance Rule 3 — batch IPC traffic; never emit per
        // keystroke-equivalent operation.
        if completed % 5 == 0 || completed == total {
            let _ = app.emit(
                "classifier:scan",
                ScanProgressEvent {
                    phase: "progress".into(),
                    total,
                    completed,
                    error: None,
                },
            );
        }

        // 2026-07-24 scan-perf investigation. This was the ONLY background job in
        // the codebase with no yield between items — every sibling has one:
        // links_backfill.rs:51, note_body_backfill.rs:44, sky_backfill.rs:48,
        // review_backfill.rs:24, props_reparse_backfill.rs:42 (50 ms), and
        // nsc/backfill.rs:32 (30 ms). Over a 7,000-note Universe that is an hour of
        // uninterrupted disk and lock pressure with no gap for anything else. The
        // pause makes the scan marginally longer in wall-clock and gives the machine
        // back while it runs — the same trade every sibling already makes.
        thread::sleep(Duration::from_millis(INTER_NOTE_SLEEP_MS));
    }

    let _ = app.emit(
        "classifier:scan",
        ScanProgressEvent {
            phase: "done".into(),
            total,
            completed,
            error: None,
        },
    );
    Ok(())
}

/// Pull the work list from `note_meta`: every note that doesn't yet
/// have BOTH axes set in the SQLite mirror, AND doesn't already have
/// a pending suggestion in the queue.
///
/// The exact predicate matches the §1A' frontmatter-extraction logic:
/// a column is "empty" when NULL, empty string, or the literal `[]`
/// (which is what the YAML serializer emits for empty list).
fn enumerate_pending(app: &AppHandle) -> Result<Vec<String>, String> {
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT m.path FROM note_meta m
             WHERE NOT EXISTS (
                SELECT 1 FROM sources_suggestions s WHERE s.note_path = m.path
             )
             AND (
                (m.sources IS NULL OR m.sources = '' OR m.sources = '[]')
                OR (m.content_type IS NULL OR m.content_type = '' OR m.content_type = '[]')
             )
             ORDER BY m.path",
        )
        .map_err(|e| format!("prepare enumerate: {}", e))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("query enumerate: {}", e))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("row: {}", e))?);
    }
    Ok(out)
}

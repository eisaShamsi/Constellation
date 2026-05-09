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
//! never on the main UI thread. Each note takes ~30ms (Tier 1 e5-small
//! embedding + cosine to ~274 candidates + DB write). At that rate, a
//! 7,000-note universe takes ~3.5 minutes.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

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
#[tauri::command]
pub fn classifier_scan_start(app: AppHandle) -> Result<(), String> {
    crate::search::ensure_search_db_ready(&app)?;
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
        let result = run_scan(app_clone.clone());
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

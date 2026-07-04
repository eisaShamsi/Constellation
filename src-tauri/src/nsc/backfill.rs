//! NSC summary backfill — Rule 8 "first-time population".
//!
//! A background, resumable worker that pre-computes the summary for every
//! note that doesn't yet have a current-version cached summary, so cards show
//! their summary instantly instead of computing it lazily on scroll.
//!
//! Modeled on `classifier::scan_job` (same AtomicBool/AtomicUsize state, same
//! event shape, same per-note cancel check) plus the `sky_backfill` throttle:
//!
//!   - **After paint, never at boot.** Started by a frontend `invoke` once the
//!     window is ready (gated on a Settings toggle) — it never runs during the
//!     Tauri setup/boot path, so it can't regress boot time.
//!   - **Resumable by re-enumeration.** Each run asks the DB which notes still
//!     lack a `NSC_ALGO_VERSION` summary row; killing + relaunching resumes on
//!     the residual. No cursor table.
//!   - **Gentle.** `crate::nsc::get_or_compute_cached` acquires + releases the
//!     embedding-engine lock per note (never across notes), so interactive
//!     calls interleave; an inter-note sleep adds margin; and the loop PAUSES
//!     entirely while a classifier scan is running (they share the engine).
//!   - **Cancellable.** A per-note atomic-load check (~2 ns).

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Inter-note pause so the auto-background backfill yields the CPU + the
/// embedding engine to anything interactive. Small enough that a full pass
/// still completes in a reasonable time, large enough to stay invisible.
const INTER_NOTE_SLEEP_MS: u64 = 30;
/// While a classifier scan is running we don't process at all (they fight for
/// the same embedding engine). Poll the scan flag at this interval.
const SCAN_WAIT_POLL_MS: u64 = 250;
/// Throttle progress events: emit every N notes (+ always at the end).
const EVENT_EVERY: usize = 25;

#[derive(Default)]
pub struct NscBackfillState {
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    pub completed: AtomicUsize,
    pub total: AtomicUsize,
    pub last_error: Mutex<Option<String>>,
}

impl NscBackfillState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Snapshot for `nsc_backfill_status` — lets the frontend strip render on
/// mount without waiting for an event (e.g. recover after a missed event).
#[derive(Serialize, Clone)]
pub struct NscBackfillStatus {
    pub running: bool,
    pub cancelling: bool,
    pub completed: usize,
    pub total: usize,
    pub last_error: Option<String>,
}

/// Tauri event payload — same shape as `classifier:scan` so the frontend strip
/// can be modeled on `ClassifierScanProgressStrip`.
#[derive(Serialize, Clone)]
struct NscBackfillEvent {
    phase: String,
    total: usize,
    completed: usize,
    error: Option<String>,
}

#[tauri::command]
pub fn nsc_backfill_status(app: AppHandle) -> NscBackfillStatus {
    let state = app.state::<NscBackfillState>();
    NscBackfillStatus {
        running: state.running.load(Ordering::Relaxed),
        cancelling: state.cancel.load(Ordering::Relaxed),
        completed: state.completed.load(Ordering::Relaxed),
        total: state.total.load(Ordering::Relaxed),
        last_error: state.last_error.lock().ok().and_then(|g| g.clone()),
    }
}

#[tauri::command]
pub fn nsc_backfill_cancel(app: AppHandle) -> Result<(), String> {
    let state = app.state::<NscBackfillState>();
    state.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

/// Kick off the background backfill. Idempotent: a second call while one is
/// running returns an error. Spawns a worker thread and returns immediately.
// App-freeze audit Batch-D (2026-07-03): ensure_search_db_ready moved INSIDE
// the spawned worker (it parked the dispatch thread for the whole cold init);
// the worker's error path covers an ensure failure.
#[tauri::command]
pub fn nsc_backfill_start(app: AppHandle) -> Result<(), String> {
    let state = app.state::<NscBackfillState>();

    if state.running.swap(true, Ordering::Relaxed) {
        return Err("NSC backfill already running".into());
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
            .and_then(|_| run_backfill(app_clone.clone()));
        let state = app_clone.state::<NscBackfillState>();
        if let Err(e) = result {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(e.clone());
            }
            let _ = app_clone.emit(
                "nsc:backfill",
                NscBackfillEvent {
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

fn run_backfill(app: AppHandle) -> Result<(), String> {
    let pending = enumerate_pending(&app)?;
    let total = pending.len();
    let state = app.state::<NscBackfillState>();
    state.total.store(total, Ordering::Relaxed);

    // Nothing to do — emit nothing visible. The strip only ever appears when
    // there is real work (it goes visible on `start`), so a boot where every
    // note already has a current summary shows no UI at all.
    if total == 0 {
        return Ok(());
    }

    let _ = app.emit(
        "nsc:backfill",
        NscBackfillEvent { phase: "start".into(), total, completed: 0, error: None },
    );

    let mut completed: usize = 0;
    for note_path in pending {
        if state.cancel.load(Ordering::Relaxed) {
            let _ = app.emit(
                "nsc:backfill",
                NscBackfillEvent { phase: "cancelled".into(), total, completed, error: None },
            );
            return Ok(());
        }

        // Stand down entirely while a classifier scan runs — they share the
        // embedding engine, and the scan is the higher-priority foreground job.
        while classifier_scan_running(&app) {
            if state.cancel.load(Ordering::Relaxed) {
                let _ = app.emit(
                    "nsc:backfill",
                    NscBackfillEvent { phase: "cancelled".into(), total, completed, error: None },
                );
                return Ok(());
            }
            thread::sleep(Duration::from_millis(SCAN_WAIT_POLL_MS));
        }

        // Per-note errors are recorded but never abort the pass — one bad note
        // shouldn't stop the rest.
        if let Err(e) = crate::nsc::get_or_compute_cached(&app, &note_path) {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(format!("{}: {}", note_path, e));
            }
        }
        completed += 1;
        state.completed.store(completed, Ordering::Relaxed);

        if completed % EVENT_EVERY == 0 || completed == total {
            let _ = app.emit(
                "nsc:backfill",
                NscBackfillEvent { phase: "progress".into(), total, completed, error: None },
            );
        }

        // Yield so interactive work cuts in front.
        thread::sleep(Duration::from_millis(INTER_NOTE_SLEEP_MS));
    }

    let _ = app.emit(
        "nsc:backfill",
        NscBackfillEvent { phase: "done".into(), total, completed, error: None },
    );
    Ok(())
}

/// True while a classifier scan is in progress (so the backfill can stand down).
fn classifier_scan_running(app: &AppHandle) -> bool {
    app.state::<crate::classifier::scan_job::ScanState>()
        .running
        .load(Ordering::Relaxed)
}

/// Work list: every note in `note_meta` that does not yet have a
/// current-algorithm-version (`NSC_ALGO_VERSION`) summary row. As the worker
/// writes versioned rows the set shrinks, so a re-run after a kill resumes on
/// the residual (and pre-version rows from an older algorithm are recomputed).
fn enumerate_pending(app: &AppHandle) -> Result<Vec<String>, String> {
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;

    // `NSC_ALGO_VERSION` is a controlled internal constant (e.g. "v2"), not
    // user input — safe to interpolate into the LIKE prefix.
    let prefix_pattern = format!("{}:%", crate::nsc::NSC_ALGO_VERSION);
    let mut stmt = conn
        .prepare(
            "SELECT m.path FROM note_meta m
             WHERE NOT EXISTS (
                SELECT 1 FROM note_summaries s
                WHERE s.path = m.path AND s.content_hash LIKE ?1
             )
             ORDER BY m.path",
        )
        .map_err(|e| format!("prepare enumerate: {}", e))?;
    let rows = stmt
        .query_map([prefix_pattern], |row| row.get::<_, String>(0))
        .map_err(|e| format!("query enumerate: {}", e))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("row: {}", e))?);
    }
    Ok(out)
}

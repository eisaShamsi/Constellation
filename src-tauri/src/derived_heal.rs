//! PJ-228 — **the interrupted-repair heal, off the boot path.**
//!
//! When a repair walk dies mid-flight, the five derived families (outgoing/incoming link
//! aggregates, Sky stratum, `tag_counts`, `review_schedule`) can disagree with
//! `note_meta`. Two crash markers record that; the next launch heals.
//!
//! Until now that heal ran **inside `init_db`**, synchronously, before `state.db` was
//! published — so the whole app waited on it, with no progress, no error surface (its
//! `eprintln!`s go nowhere in a Windows release build), and no way to tell a slow launch
//! from a hung one. Measured on a real 2,721-note universe: **3,143 ms**
//! (`converge::tests::converge_boot_heal_cost`). §11's Cancel made the armed state
//! reachable by an ordinary gesture — cancel a repair and the marker stays set — so this
//! stopped being theoretical.
//!
//! CLAUDE.md Rule 8 already says what a pass like this owes: *"the one-off back-fill
//! should run in the background after paint, with progress in the status bar — and must
//! be resumable."* Every sibling back-fill (`sky_backfill`, `tag_counts`,
//! `review_backfill`, `reconcile`) is scheduled that way. This one simply never was.
//!
//! ## What makes backgrounding safe, and what would break it
//!
//! Running after the database is published means the app is usable for ~3 s while the
//! derived views are still stale. That is survivable **only because every family is an
//! absolute recompute** — `tag_counts` is one `DELETE` + one `INSERT … SELECT` inside a
//! single transaction, and the link families are single `UPDATE … SET col = (SELECT …)`
//! statements. So a note saved during the window either lands before the recompute's
//! SELECT (and is counted) or after its COMMIT (and applies to a correct base). There is
//! no interleaving in which a stale read is written back permanently. **Make a family
//! incremental and this module becomes unsafe.**
//!
//! ## The three things that would make the naive version wrong
//!
//! 1. **Clearing a marker it did not earn.** A repair can start while the heal runs, and
//!    a repair arms `derived_tail_pending` *before* its own tail (`search.rs`). A heal
//!    that finished afterwards and cleared on "no failures" would wipe the in-flight
//!    repair's crash net — and nothing would ever heal again, undetectably. So the heal
//!    **yields** to a repair and clears only when it converged fully, nothing overlapped
//!    it, and the universe did not change under it. A repair is a superset of a heal (its
//!    tail converges the same five families and clears the same markers), so yielding
//!    costs nothing.
//! 2. **A connection without the tokenizer.** A bare `Connection::open` fails at prepare
//!    time on any `UPDATE note_meta`, because the FTS trigger opens `notes_fts`
//!    (`tokenize='constellation'`). Proven, not assumed: the first run of the cost
//!    harness aborted three of five families with *"no such tokenizer: constellation"*.
//! 3. **Running against a busy database.** A defrag VACUUM holds the file for minutes and
//!    the recomputes' busy-retry budget is finite, so the heal refuses to start while a
//!    heavy job or a repair holds it, and waits for the next launch.
//!
//! It never touches `state.db` — its own connection, so a 3 s job can never become a 3 s
//! app-wide freeze.

use crate::converge::{ConvergeKey, ConvergeReport, Ctx, Families};
use crate::search::SearchState;
use rusqlite::Connection;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

/// The five families, which is what the progress strip counts. Coarse by nature: this is
/// a five-step job, not an N-item one.
const FAMILY_COUNT: usize = 5;

/// The progress event name — the shared `JobProgressStrip` contract (PJ-207 §10).
const EVENT: &str = "derived-heal:progress";

#[derive(Default)]
pub struct HealState {
    running: AtomicBool,
    cancel: AtomicBool,
    completed: AtomicUsize,
    total: AtomicUsize,
    last_error: Mutex<Option<String>>,
}

impl HealState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// The status snapshot the shared strip's recover-on-mount reads. Field names match
/// `jobProgressCore.ts`'s `JobStatus` exactly.
#[derive(serde::Serialize)]
pub struct HealStatus {
    pub running: bool,
    pub cancelling: bool,
    pub completed: usize,
    pub total: usize,
    pub last_error: Option<String>,
}

fn emit(app: &AppHandle, phase: &str) {
    let state = app.state::<HealState>();
    let _ = app.emit(
        EVENT,
        serde_json::json!({
            "phase": phase,
            "total": state.total.load(Ordering::Relaxed),
            "completed": state.completed.load(Ordering::Relaxed),
            "error": Option::<String>::None,
        }),
    );
}

/// Is either crash marker armed? Two indexed point reads on `schema_versions`.
///
/// Deliberately not a scan: this runs on every launch, and the boot path is where a
/// full-table read becomes a freeze (the PJ-066 shape).
fn markers_armed(conn: &Connection) -> bool {
    crate::search::outgoing_triggers_dropped_marker(conn)
        || crate::search::derived_tail_pending_marker(conn)
}

/// Schedule the heal on a background thread. Returns immediately; no-op when nothing is
/// armed, which is every ordinary launch.
pub fn maybe_schedule(app: AppHandle) {
    {
        let state = app.state::<SearchState>();
        let Ok(guard) = state.db.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        if !markers_armed(conn) {
            return;
        }
    }
    // A repair converges the same five families and clears the same markers, so it is a
    // superset of this job — never a competitor to race. A heavy job (defrag VACUUM)
    // holds the file for minutes, which would burn the recomputes' retry budget.
    if crate::index_repair::is_running(&app) || crate::search::heavy_db_job_running() {
        return;
    }
    let state = app.state::<HealState>();
    if state
        .running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    state.cancel.store(false, Ordering::SeqCst);
    state.completed.store(0, Ordering::Relaxed);
    state.total.store(FAMILY_COUNT, Ordering::Relaxed);

    let app_bg = app.clone();
    let spawned = std::thread::Builder::new()
        .name("derived-heal".into())
        .spawn(move || {
            let outcome = run(&app_bg);
            let st = app_bg.state::<HealState>();
            let phase = match &outcome {
                Ok(report) if report.stopped => "cancelled",
                Ok(_) => "done",
                Err(e) => {
                    if let Ok(mut g) = st.last_error.lock() {
                        *g = Some(e.clone());
                    }
                    "error"
                }
            };
            if matches!(outcome, Ok(ref r) if !r.stopped) {
                st.completed.store(FAMILY_COUNT, Ordering::Relaxed);
            }
            st.running.store(false, Ordering::SeqCst);
            emit(&app_bg, phase);
        });

    if spawned.is_err() {
        // Observable rather than a silently un-run heal: the flag must not stay claimed.
        state.running.store(false, Ordering::SeqCst);
        if let Ok(path) = crate::search::db_path(&app) {
            crate::search::diag_log(&path, "[derived-heal] could not spawn the heal thread");
        }
    }
}

fn run(app: &AppHandle) -> Result<ConvergeReport, String> {
    let path = crate::search::db_path(app)?;
    // Its own connection — never `state.db`. See the module header.
    let mut conn = Connection::open(&path)
        .map_err(|e| format!("open search.db for the derived heal: {}", e))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA recursive_triggers=ON;",
    )
    .map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(30))
        .map_err(|e| e.to_string())?;
    // Without this the FTS trigger's INSERT INTO notes_fts fails on this connection —
    // and the failure is per-family and best-effort, so it would look like a heal that
    // simply never works. See the module header, point 2.
    crate::search::register_fts5_tokenizer(&mut conn)?;

    let gen0 = crate::search::federation_generation_now(app);
    let cdir = path.parent().map(|p| p.to_path_buf());

    // The stop closure doubles as the progress tick: `converge_derived_views` asks it
    // once before each family, which is exactly the five-step cadence the strip renders.
    let ticks = AtomicUsize::new(0);
    let repair_overlapped = AtomicBool::new(false);
    let app_s = app.clone();
    let stop_fn = move || -> bool {
        let st = app_s.state::<HealState>();
        let k = ticks.fetch_add(1, Ordering::Relaxed);
        st.completed.store(k, Ordering::Relaxed);
        if k > 0 {
            emit(&app_s, "progress");
        }
        if crate::index_repair::is_running(&app_s) {
            repair_overlapped.store(true, Ordering::SeqCst);
            return true;
        }
        st.cancel.load(Ordering::SeqCst)
            || crate::search::federation_generation_now(&app_s) != gen0
    };

    emit(app, "start");
    let report = crate::converge::heal_interrupted_walk(&conn, cdir.as_deref(), &stop_fn);

    for (family, msg) in report.failures() {
        crate::search::diag_log(
            &path,
            &format!("[derived-heal] {} failed (markers kept; retried next launch): {}", family, msg),
        );
    }

    // The clear condition, stated once. Every term matters: `converged_fully` rules out
    // a run that gave way (see `ConvergeReport::converged_fully`); the overlap check
    // stops us wiping a concurrent repair's crash net; the generation check stops us
    // clearing the DEPARTING universe's marker after a switch.
    let gen_stable = crate::search::federation_generation_now(app) == gen0;
    let overlapped = crate::index_repair::is_running(app);
    if report.converged_fully() && gen_stable && !overlapped {
        if let Err(e) = crate::search::clear_outgoing_triggers_dropped_marker(&conn) {
            crate::search::diag_log(&path, &format!("[derived-heal] clear outgoing marker failed: {}", e));
        }
        if crate::search::derived_tail_pending_marker(&conn) {
            if let Err(e) = crate::search::clear_derived_tail_pending_marker(&conn) {
                crate::search::diag_log(&path, &format!("[derived-heal] clear derived-tail marker failed: {}", e));
            }
        }
        crate::search::diag_log(&path, "[derived-heal] converged; markers cleared");
    } else {
        crate::search::diag_log(
            &path,
            &format!(
                "[derived-heal] markers KEPT (converged_fully={} gen_stable={} repair_overlapped={}) — the next launch retries",
                report.converged_fully(), gen_stable, overlapped
            ),
        );
    }
    Ok(report)
}

#[tauri::command]
pub fn derived_heal_status(app: AppHandle) -> HealStatus {
    let s = app.state::<HealState>();
    HealStatus {
        running: s.running.load(Ordering::SeqCst),
        cancelling: s.cancel.load(Ordering::SeqCst),
        completed: s.completed.load(Ordering::Relaxed),
        total: s.total.load(Ordering::Relaxed),
        last_error: s.last_error.lock().ok().and_then(|g| g.clone()),
    }
}

#[tauri::command]
pub fn derived_heal_cancel(app: AppHandle) {
    app.state::<HealState>().cancel.store(true, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The strip renders `completed / total`; a five-family job must present five steps,
    /// not a note count. Pinned so a future family addition updates the strip's meaning
    /// deliberately rather than silently drifting.
    #[test]
    fn the_job_is_five_steps() {
        assert_eq!(FAMILY_COUNT, 5);
    }

    /// The event name is the strip's contract with the frontend; a rename here without
    /// the matching `eventName` prop is a strip that never shows anything.
    #[test]
    fn the_event_name_matches_the_strip_contract() {
        assert_eq!(EVENT, "derived-heal:progress");
    }
}

//! PJ-207 §7 — **only one thing may walk the library.**
//!
//! Before this module, Constellation had **two independent walkers** and no guard on
//! either:
//!
//! - `search::reconcile_filesystem` — the mtime-gated walk plus the five-family
//!   convergence tail. Reached from four frontend call sites.
//! - `libraries::reindex_library` — **not** a wrapper over it: its own
//!   `collect_md_paths` + per-file `reindex_single_note`. Reached from three more.
//!
//! Seven entry points, two algorithms, zero mutual exclusion. Two of them fire from a
//! *single* user gesture: "bring in a library" calls `bringInLibrary` (which invokes
//! `reindex_library`) and then `initSearchIndex()` (which invokes the other walker).
//!
//! Everything now submits here. The runner owns the single-flight guard, the trigger
//! window, the universe-switch check, the mutual exclusions, and the cancel handshake.
//!
//! ## Why this is one commit and not three
//!
//! Splitting it produces exactly the half-migrated state this project calls its dominant
//! defect: a tree in which one walker is guarded and the other is not, while a comment
//! claims "one index job process-wide". The plan says so explicitly, and it is right —
//! this is the largest commit in the migration and the smallest diff that is not a lie.
//!
//! ## What it deliberately does NOT do
//!
//! - **It does not take `MIGRATION_ACTIVE`.** That flag stands the WAL checkpoint daemon
//!   down (`search.rs`), and a full re-read with checkpointing off means unbounded WAL
//!   growth on a multi-GB database. The runner self-checkpoints instead
//!   (`wal_checkpoint(PASSIVE)` every [`CHECKPOINT_EVERY`] notes on its own connection) —
//!   the same bargain the bigram purge already makes.
//! - **It gates no trigger-creation site.** Making `create_outgoing_link_triggers` a
//!   no-op behind a flag would mean a leaked flag silently freezes `note_meta.outgoing_*`
//!   on the live save path for every note. Instead the window is bounded by RAII, and a
//!   mid-run re-arm by the vocabulary path is *absorbed*: that function drops-then-creates
//!   (idempotent), and the convergence tail recomputes everything afterwards. The cost of
//!   a collision is the O(N²) trigger fire the walk was avoiding — **performance, never
//!   correctness.**

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Hand the machine back between notes, matching every sibling background job
/// (`links_backfill` / `sky_backfill` / `review_backfill` at 50 ms, `nsc/backfill` and
/// the classifier scan at 30 ms). Without it a full walk is uninterrupted disk and lock
/// pressure with no gap for anything else.
const INTER_NOTE_SLEEP_MS: u64 = 30;

/// Self-checkpoint cadence — see the module note on `MIGRATION_ACTIVE`.
const CHECKPOINT_EVERY: usize = 500;

/// What a submitter wants brought current.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// The whole active universe: the mtime-gated walk over every own library, then the
    /// five-family convergence. What the four `constellation_search_init` callers mean.
    Full,
    /// One library that may never have been indexed. Preserves `reindex_library`'s
    /// semantics exactly, including its cheap `COUNT(*)` gate (which is what honours
    /// ZERO-BOOT-WALKS / LL-022) and its per-file library attribution.
    ColdStart {
        library_path: String,
        library_name: String,
        only_if_unindexed: bool,
    },
}

impl Scope {
    /// Does a running job of `self` already cover a newly submitted `other`?
    /// A Full run subsumes everything; a ColdStart covers only its own library.
    fn covers(&self, other: &Scope) -> bool {
        match (self, other) {
            (Scope::Full, _) => true,
            (
                Scope::ColdStart { library_path: a, .. },
                Scope::ColdStart { library_path: b, .. },
            ) => a == b,
            _ => false,
        }
    }
}

/// The answer to a submit. **Typed, never a bare `Err`** — every existing caller
/// swallows errors (`.catch(() => {})`, `.catch(() => 0)`), so a refusal expressed as an
/// error is a refusal nobody sees. §11's report is only truthful because this is.
/// **Wire note (safety review, finding 8).** `rename_all` on an enum renames the VARIANT
/// names, not the fields inside struct variants — so `run_id` was reaching the frontend as
/// `run_id` while the TypeScript type declared `runId`, which would simply have been
/// `undefined` at the first consumer. Each variant carries its own `rename_all`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SubmitOutcome {
    /// This submit started the run.
    #[serde(rename_all = "camelCase")]
    Started { run_id: u64 },
    /// A run was already going and did not cover this scope, so it was **queued** —
    /// never refused (Invariant 4). Refusing is what would silently re-open the
    /// LL-027 / BUG-022 cold-start gap for every library but the first.
    #[serde(rename_all = "camelCase")]
    Queued { run_id: u64 },
    /// A run was already going and already covers this scope. Nothing to do.
    #[serde(rename_all = "camelCase")]
    AlreadyRunning { run_id: u64 },
    /// PJ-207 §8 — the submitted library belongs to a **linked universe**, so this
    /// universe's index is not where its notes go. Neither work nor failure: the notes
    /// stay reachable through the federated search path, and writing them here is the
    /// Charter W2-9 defect this refusal exists to prevent. Distinct from `Blocked` on
    /// purpose — `Blocked` means "not now" and is re-offered by the drain, whereas this
    /// is "not ever, by this universe", and re-offering it would loop.
    #[serde(rename_all = "camelCase")]
    Foreign { library_name: String },
    /// Refused, with the reason. Something incompatible holds the database, or the
    /// library registry that decides what is in scope could not be read.
    Blocked { reason: String },
}

/// Why a run refused to start or stopped early.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    /// The MIG-108 unification engine holds the world open.
    Mig108Running,
    /// The defrag worker holds `state.db` for its VACUUM.
    DefragRunning,
    /// The active universe changed under the run.
    UniverseSwitched,
    /// The user (or app close) asked it to stop.
    Cancelled,
}

impl StopReason {
    fn as_str(self) -> &'static str {
        match self {
            StopReason::Mig108Running => "a universe unification is running",
            StopReason::DefragRunning => "the database is being compacted",
            StopReason::UniverseSwitched => "the active universe changed",
            StopReason::Cancelled => "cancelled",
        }
    }
}

/// Managed state, registered in `lib.rs` beside `ScanState`.
#[derive(Default)]
pub struct RepairState {
    running: AtomicBool,
    cancel: AtomicBool,
    /// Monotonic; also the identity a `TriggerWindow`'s marker is owned by, so a second
    /// run cannot clear a first run's crash protection.
    run_id: AtomicU64,
    completed: AtomicUsize,
    total: AtomicUsize,
    /// Scopes submitted while a run was in flight and not covered by it. Drained by the
    /// runner before it exits, so a queued submit is genuinely processed.
    pending: Mutex<Vec<Scope>>,
    /// What the in-flight run is doing. Without this a `Full` submitted during a `Full`
    /// queued a SECOND whole-universe walk — four `initSearchIndex()` sites can overlap.
    /// Perf only, but a duplicate walk of 7,800 notes is not a rounding error.
    current: Mutex<Option<Scope>>,
    last_error: Mutex<Option<String>>,
    /// PJ-207 §9 — the last drift report the boot reconcile produced, so a surface that
    /// mounted after the event fired can still ask. `None` means "no answer yet", which is
    /// emphatically not "nothing is wrong": the pass may still be walking, or it may have
    /// stopped before it had an answer. The frontend must render nothing for `None`.
    drift: Mutex<Option<crate::reconcile::DriftReport>>,
}

impl RepairState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Snapshot for the status command — so a strip can recover on mount rather than relying
/// on having caught an event (the `classifier_scan_status` discipline).
#[derive(Serialize, Clone)]
pub struct RepairStatus {
    pub running: bool,
    pub cancelling: bool,
    pub completed: usize,
    pub total: usize,
    pub run_id: u64,
    pub last_error: Option<String>,
}

/// True while a repair run is walking. Read by the window-close handler.
pub fn is_running(app: &AppHandle) -> bool {
    app.state::<RepairState>().running.load(Ordering::SeqCst)
}

/// Ask a running repair to stop at the next window boundary. Returns immediately.
pub fn request_cancel(app: &AppHandle) {
    app.state::<RepairState>().cancel.store(true, Ordering::SeqCst);
}

/// Has a stop been asked for? Checked by the walk between notes, so the app-close
/// handshake lands on a boundary rather than guillotining a half-written note.
pub(crate) fn cancel_requested(app: &AppHandle) -> bool {
    app.state::<RepairState>().cancel.load(Ordering::SeqCst)
}

/// PJ-207 §9 — publish the boot pass's drift report: store it for a late-mounting
/// surface, then push it.
///
/// **Both, not either.** An event alone is lost by anything that mounts after it fires
/// (the second screen, a panel opened later); a stored value alone means the notice waits
/// for whatever the user does next. This is the `classifier_scan_status` discipline the
/// progress strips already follow — recover on mount, update on event.
///
/// Nothing is emitted when the report is empty. A launch that finds nothing wrong says
/// nothing at all; a green "all clear" on every boot is noise, and noise is how a real
/// warning stops being read.
pub(crate) fn record_drift_report(app: &AppHandle, report: crate::reconcile::DriftReport) {
    if let Ok(mut g) = app.state::<RepairState>().drift.lock() {
        *g = Some(report);
    }
    if report.has_findings() {
        let _ = app.emit("index-drift:report", report);
    }
}

/// Record progress for the status command / a progress strip.
pub(crate) fn note_progress(app: &AppHandle, completed: usize) {
    app.state::<RepairState>().completed.store(completed, Ordering::Relaxed);
}

/// PJ-207 §7 — the per-note checks the bulk walk owes, in one place so the walk cannot
/// drift from the cold-start loop that already does them.
///
/// Returns `false` when the walk must stop:
/// - **cancelled** — the app is closing, or the user asked. Landing on a note boundary
///   is what lets the close handshake share the existing 5 s budget.
/// - **the universe changed** — `db_path` was captured when the run started, so
///   continuing would write the departing universe's notes into a database the user has
///   already left. Nothing in the old walk read the generation at all.
///
/// Also self-checkpoints every [`CHECKPOINT_EVERY`] notes. The runner deliberately does
/// NOT take `MIGRATION_ACTIVE` (that would stand the WAL daemon down for the whole run
/// and let the log grow unbounded on a multi-GB database), so it has to keep its own WAL
/// in check — `PASSIVE`, which never blocks a reader and yields if the writer is busy.
pub(crate) fn walk_should_continue(
    app: Option<&AppHandle>,
    conn: &rusqlite::Connection,
    generation: u64,
    seen: usize,
) -> bool {
    // `None` means there is no run context — the walk is being driven directly by a unit
    // test of the walker itself. There is nothing to cancel and no universe to switch, so
    // the gate is inert. The DECISION it makes is tested as a pure function below.
    if let Some(app) = app {
        if !walk_may_proceed(
            cancel_requested(app),
            crate::search::federation_generation_now(app),
            generation,
        ) {
            eprintln!(
                "[index_repair] walk abandoned: {}",
                if cancel_requested(app) { StopReason::Cancelled.as_str() }
                else { StopReason::UniverseSwitched.as_str() }
            );
            return false;
        }
        note_progress(app, seen);
    }
    if seen > 0 && seen % CHECKPOINT_EVERY == 0 {
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);");
    }
    true
}

/// The stop DECISION, as a pure function — so it can be pinned without an `AppHandle`.
///
/// A walk continues only while nothing has asked it to stop and the universe it started
/// in is still the active one. The second half is the one the old walk had no notion of:
/// it captured `db_path` once and then wrote for minutes.
pub(crate) fn walk_may_proceed(cancelled: bool, generation_now: u64, generation_at_start: u64) -> bool {
    !cancelled && generation_now == generation_at_start
}

// ─── The trigger window ──────────────────────────────────────────────────────

/// RAII around the trigger-free bulk window.
///
/// `new` persists the crash marker **before** dropping anything — preserving the
/// discipline the previous inline code stated in terms: *"If the marker cannot be
/// persisted, do NOT enter the unprotected window."* `Drop` recreates the outgoing
/// triggers from the **then-current** link-type registry and clears the marker, on every
/// exit path including `?` and a panic. Precedent: `mig108::RunningGuard`.
///
/// **The sky edge-mirror family stays armed, and that is the status quo rather than a new
/// choice.** `drop_sky_aggregate_triggers` drops only the stratum/maturity triggers, not
/// `note_links_sky_ai/_ad/_au`. Dropping the mirror would be worse: nothing in the
/// codebase rebuilds `sky_links` — `recompute_all_sky` only UPDATEs `sky_nodes` columns —
/// so the edges would simply be lost. Its cost is bounded by `index_note`'s diff-edges,
/// which leaves unchanged edges untouched, so the mirror fires only for edges that
/// actually changed.
pub(crate) struct TriggerWindow<'a> {
    conn: &'a rusqlite::Connection,
    run_id: u64,
    /// Set by `close()`. `Drop` is then a no-op — the caller has already recreated the
    /// triggers and owns the marker decision.
    closed: bool,
}

impl<'a> TriggerWindow<'a> {
    pub(crate) fn open(conn: &'a rusqlite::Connection, run_id: u64) -> Result<Self, String> {
        crate::search::set_outgoing_triggers_dropped_marker(conn).map_err(|e| {
            format!("refusing to enter the trigger-free window: its marker could not be persisted ({e})")
        })?;
        let _ = crate::search::drop_outgoing_link_triggers(conn);
        // Incoming maintenance is a save-path Rust diff, not triggers; drop any a prior
        // build left. Idempotent cleanup.
        let _ = crate::search::drop_incoming_link_triggers(conn);
        // Sky stratum/maturity maintenance is likewise a save-path diff now; the bulk
        // convergence restores every node's value in one pass.
        let _ = crate::search::drop_sky_aggregate_triggers(conn);
        Ok(TriggerWindow { conn, run_id, closed: false })
    }

    pub(crate) fn run_id(&self) -> u64 {
        self.run_id
    }
}

impl<'a> TriggerWindow<'a> {
    /// Close the window explicitly and report whether the triggers came back.
    ///
    /// **The marker is deliberately NOT cleared here.** Recreating the triggers is only
    /// half of what the marker means: it says "the outgoing aggregates may be stale", and
    /// they still are until the convergence tail has recomputed them. Only the caller
    /// knows whether that succeeded, so only the caller may clear it.
    ///
    /// The first version of this got that wrong in a way worth recording: `Drop` cleared
    /// the marker, and a stale `outgoing_restore_err` check downstream then cleared it
    /// *again* on the failure path — so a failed recreate disarmed the boot heal that
    /// exists for exactly that failure, and the run still reported `Ok`.
    pub(crate) fn close(mut self) -> Result<(), String> {
        self.closed = true;
        crate::search::create_outgoing_link_triggers(self.conn)
    }
}

impl<'a> Drop for TriggerWindow<'a> {
    fn drop(&mut self) {
        if self.closed {
            return; // `close()` already recreated them and handed the caller the outcome.
        }
        // The UNWIND path: a `?` early-return or a panic inside the window. Recreate
        // best-effort so live saves are covered again, and **keep the marker** — the
        // convergence tail provably did not run, so the next boot must heal.
        if let Err(e) = crate::search::create_outgoing_link_triggers(self.conn) {
            eprintln!("[index_repair] run {}: recreating outgoing triggers on the unwind path failed: {e}", self.run_id);
        }
    }
}

/// PJ-207 §7 — RAII for the single-flight flag, following `mig108::RunningGuard`, which
/// exists for this exact reason and says so: *"A flag left set would make the window
/// permanently unclosable, which would be a worse bug than the one this guard prevents."*
///
/// The stakes here are the same shape. `panic = "abort"` is not set, so a panic anywhere
/// in the worker unwinds and the thread dies. A plain `store(false)` at the end of the
/// closure is never reached, and the leak is **session-permanent**: every later submit
/// returns `AlreadyRunning`, so no repair and no cold start ever runs again — and because
/// the defrag worker now refuses while a repair is "running", the database can never
/// compact either. One missed line would disable two subsystems until relaunch.
struct RunGuard {
    app: AppHandle,
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        self.app.state::<RepairState>().running.store(false, Ordering::SeqCst);
    }
}

// ─── Submit ──────────────────────────────────────────────────────────────────

/// The one door. Everything that used to walk the library calls this.
pub fn submit(app: &AppHandle, scope: Scope) -> SubmitOutcome {
    let state = app.state::<RepairState>();

    // PJ-207 §8 — SCOPE before timing. The boot fan-out submits one `ColdStart` per entry
    // of the frontend's library list, and that list is the federation-recursive set, so a
    // linked universe's libraries arrive here every boot. Answer them at the door.
    //
    // Checked first, and before the single-flight flag is touched, for two reasons: a
    // refusal expressed as `Blocked` would be pushed onto the pending list and re-offered
    // by the drain forever, and a refusal discovered later — inside the worker, where the
    // old registration check lived — would land in `last_error` and emit `ok: false`,
    // making an ordinary federated boot look like a failed repair.
    if let Scope::ColdStart { ref library_path, ref library_name, .. } = scope {
        match crate::libraries::try_load_libraries(app) {
            Ok(own) => {
                if !own.iter().any(|l| l.path == *library_path) {
                    return SubmitOutcome::Foreign { library_name: library_name.clone() };
                }
            }
            // "I could not read the registry" is not "it is not yours". Say which.
            Err(e) => return SubmitOutcome::Blocked { reason: e },
        }
    }

    // Mutual exclusion, checked before the guard so a blocked submit does not consume it.
    if crate::mig108::engine_is_running() {
        return SubmitOutcome::Blocked { reason: StopReason::Mig108Running.as_str().into() };
    }
    if crate::search::heavy_db_job_running() {
        return SubmitOutcome::Blocked { reason: StopReason::DefragRunning.as_str().into() };
    }

    // Single-flight. `swap` returns the previous value; if it was already true, a run
    // owns the walk and this submit is queued rather than refused.
    if state.running.swap(true, Ordering::SeqCst) {
        let run_id = state.run_id.load(Ordering::SeqCst);
        // Already covered by what is RUNNING? Then there is genuinely nothing to do.
        if state.current.lock().ok().and_then(|c| c.clone()).map_or(false, |c| c.covers(&scope)) {
            return SubmitOutcome::AlreadyRunning { run_id };
        }
        if let Ok(mut pending) = state.pending.lock() {
            // …or by something already waiting.
            if !pending.iter().any(|p| p.covers(&scope)) {
                pending.push(scope);
                return SubmitOutcome::Queued { run_id };
            }
        }
        return SubmitOutcome::AlreadyRunning { run_id };
    }

    let run_id = state.run_id.fetch_add(1, Ordering::SeqCst) + 1;
    state.cancel.store(false, Ordering::SeqCst);
    state.completed.store(0, Ordering::SeqCst);
    state.total.store(0, Ordering::SeqCst);
    if let Ok(mut g) = state.last_error.lock() {
        *g = None;
    }
    if let Ok(mut c) = state.current.lock() {
        *c = Some(scope.clone());
    }

    let app_bg = app.clone();
    // `Builder::spawn` rather than `thread::spawn` so a spawn FAILURE is observable.
    // `thread::spawn` panics instead of returning, which would leave the flag taken and
    // the app permanently unable to repair — the leak `RunGuard` exists to prevent, walked
    // in through the front door.
    if std::thread::Builder::new()
        .name(format!("index-repair-{run_id}"))
        .spawn(move || {
        // The guard is the FIRST thing, so every exit path — `?`, early return, panic —
        // releases the single-flight flag. See `RunGuard`.
        let _guard = RunGuard { app: app_bg.clone() };

        // `ensure_search_db_ready` INSIDE the worker — as a command's first statement it
        // parks the dispatch thread for the whole cold init (the App-freeze audit's
        // Batch-D finding, and the reason the classifier chassis does the same).
        let outcome = crate::search::ensure_search_db_ready(&app_bg)
            .and_then(|_| run(&app_bg, scope.clone(), run_id));

        let state = app_bg.state::<RepairState>();
        // A run that STOPPED EARLY (app closing, universe switched) is not a failure —
        // but it is emphatically not a completed repair either, and the difference has to
        // reach the caller. Reporting `ok: true` for a walk that stopped one note in is
        // the exact "success for work that did not happen" class this migration is about.
        let stopped_early = outcome.as_ref().map(|c| c.stopped_early).unwrap_or(false);
        let ok = outcome.is_ok() && !stopped_early;
        if let Err(ref e) = outcome {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(e.clone());
            }
            eprintln!("[index_repair] run {run_id} failed: {e}");
        } else if stopped_early {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some("the repair stopped before it finished — run it again".into());
            }
        }

        if let Ok(mut c) = state.current.lock() {
            *c = None;
        }
        // Release the flag BEFORE the follow-ups: the defrag schedule and the drain both
        // consult `is_running`, and with the flag still set the post-run compaction was
        // silently a no-op every single time.
        drop(_guard);

        let _ = app_bg.emit(
            "index-repair:done",
            serde_json::json!({
                "runId": run_id,
                "ok": ok,
                "stoppedEarly": stopped_early,
            }),
        );

        // The standing defrag rule: after a mass rewrite the database compacts itself,
        // gated by its own state-based predicate. AFTER the run and after the flag is
        // released, or it can never fire.
        if ok && matches!(scope, Scope::Full) {
            crate::search::maybe_schedule_defrag(app_bg.clone());
        }

        // Drain anything queued while we ran. Looped, because a submit can land between
        // the drain and the flag release; and a scope the exclusions REFUSE is put back
        // rather than dropped — the submitter was promised `Queued`, and dropping it here
        // would silently re-open the cold-start gap for a boot that overlapped a VACUUM.
        loop {
            let queued: Vec<Scope> = state
                .pending
                .lock()
                .map(|mut p| p.drain(..).collect())
                .unwrap_or_default();
            if queued.is_empty() {
                break;
            }
            let mut deferred = Vec::new();
            // PJ-207 §9 (safety inspection 2026-08-07) — did this batch hand the queue to a
            // new run? Without knowing, the loop below spins.
            let mut handed_off = false;
            for s in queued {
                match submit(&app_bg, s.clone()) {
                    SubmitOutcome::Blocked { .. } => deferred.push(s),
                    // A run now owns the walk — either this submit started it, or it joined
                    // that run's queue. Either way ITS drain will finish the work.
                    SubmitOutcome::Started { .. }
                    | SubmitOutcome::Queued { .. }
                    | SubmitOutcome::AlreadyRunning { .. } => handed_off = true,
                    SubmitOutcome::Foreign { .. } => {}
                }
            }
            if !deferred.is_empty() {
                if let Ok(mut p) = state.pending.lock() {
                    for d in deferred {
                        if !p.iter().any(|q| q.covers(&d)) {
                            p.push(d);
                        }
                    }
                }
                break; // still blocked — leave them for the next run's drain.
            }
            // **The spin this closes.** With three libraries pending, the first re-submit
            // starts run 2 and the second returns `Queued` — and `Queued` means `submit`
            // has already pushed that scope BACK onto `pending`. `deferred` collects only
            // `Blocked`, so it stayed empty, nothing broke the loop, and the next iteration
            // drained the very scope just pushed back: an unbounded busy-spin re-reading
            // and re-parsing `libraries.json` for the entire duration of run 2 — minutes,
            // on a genuinely unindexed library. Silent, because it burns a background
            // worker rather than the UI thread, so it reads as ordinary slow boot indexing.
            if handed_off {
                break;
            }
        }
        })
        .is_err()
    {
        // `spawn` failed AFTER the flag was taken. Without this the flag leaks exactly as
        // a panic would.
        state.running.store(false, Ordering::SeqCst);
        return SubmitOutcome::Blocked { reason: "could not start a worker thread".into() };
    }

    SubmitOutcome::Started { run_id }
}

// ─── The run ─────────────────────────────────────────────────────────────────

/// What a completed run has to say about itself.
pub(crate) struct RunCompletion {
    /// The run stopped before finishing — the app is closing, or the universe changed.
    pub stopped_early: bool,
}

fn run(app: &AppHandle, scope: Scope, run_id: u64) -> Result<RunCompletion, String> {
    match scope {
        Scope::ColdStart { library_path, library_name, only_if_unindexed } => {
            run_cold_start(app, &library_path, &library_name, only_if_unindexed, run_id)
        }
        Scope::Full => run_full(app, run_id),
    }
}

/// The absorbed `reindex_library`. Its semantics are preserved deliberately and in full:
/// the cheap `COUNT(*)` gate that honours ZERO-BOOT-WALKS, the per-file library
/// attribution (the 2026-07-25 Whole-Ecosystem fix, which makes the outcome
/// order-independent), and — importantly — `reindex_single_note` as the per-note
/// primitive rather than the bulk walker's bare `index_note`. The comment at its old site
/// says why: `reindex_single_note` also runs the incoming-aggregate diff post-commit, so
/// a cold-started library's TARGET notes get correct backlink counts, not just outgoing.
fn run_cold_start(
    app: &AppHandle,
    library_path: &str,
    library_name: &str,
    only_if_unindexed: bool,
    run_id: u64,
) -> Result<RunCompletion, String> {
    // PJ-207 §8 — the OWN set, through the strict loader. `submit` already refused a
    // linked universe's library at the door; re-deriving the authorization here from the
    // same source means the door's decision and the walk's can never disagree (a registry
    // that changed in between is caught, not walked). It is also the list
    // `library_name_for_path` sees below — feeding it the recursive set is what stamped a
    // foreign note with an own library's name.
    let libraries = crate::libraries::try_load_libraries(app)?;
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a library of this universe.".to_string());
    }
    let state = app.state::<crate::search::SearchState>();

    if only_if_unindexed {
        let indexed: i64 = crate::search::with_read_conn(state.inner(), |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM note_meta WHERE library_name = ?1",
                rusqlite::params![library_name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
        })
        .unwrap_or(0);
        if indexed > 0 {
            // Already indexed — no walk. The ZERO-BOOT-WALKS gate, and a genuine completion.
            return Ok(RunCompletion { stopped_early: false });
        }
    }

    let mut md_paths: Vec<std::path::PathBuf> = Vec::new();
    crate::libraries::collect_md_paths(std::path::Path::new(library_path), &mut md_paths);
    // PJ-207 §8 — `collect_md_paths` descends through every non-dot directory and has no
    // notion of a library boundary at all. `universe_notes` has `path == the Universe
    // root`, so cold-starting it collects everything nested underneath — including a
    // LINKED universe's directory if one sits there. Drop those before the loop: left in,
    // `library_name_for_path` would find no OWN library for them and the
    // `unwrap_or_else` fallback below would stamp them with THIS library's name — a
    // foreign note filed under an own library, which is worse than an unscoped walk
    // because every name-scoped count and search then inherits it.
    let foreign_roots = crate::libraries::foreign_library_roots(app, &libraries);
    if !foreign_roots.is_empty() {
        md_paths.retain(|p| {
            !crate::libraries::path_is_under_any(&p.to_string_lossy(), &foreign_roots)
        });
    }

    let rs = app.state::<RepairState>();
    rs.total.store(md_paths.len(), Ordering::Relaxed);

    let generation = crate::search::federation_generation_now(app);
    for (i, p) in md_paths.iter().enumerate() {
        if !crate::index_repair::walk_may_proceed(
            rs.cancel.load(Ordering::SeqCst),
            crate::search::federation_generation_now(app),
            generation,
        ) {
            eprintln!("[index_repair] run {run_id} stopped early");
            return Ok(RunCompletion { stopped_early: true });
        }
        let ps = p.to_string_lossy().to_string();
        let lib_name = crate::libraries::library_name_for_path(&libraries, &ps)
            .unwrap_or_else(|| library_name.to_string());
        let _ = crate::search::reindex_single_note(state.inner(), &ps, &lib_name);
        rs.completed.store(i + 1, Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(INTER_NOTE_SLEEP_MS));
    }
    Ok(RunCompletion { stopped_early: false })
}

/// The full walk plus the five-family convergence.
fn run_full(app: &AppHandle, run_id: u64) -> Result<RunCompletion, String> {
    let stats = crate::search::reconcile_filesystem_guarded(app, run_id)?;
    // The walk's own account of itself. Discarding it — which the first version of this
    // did with `let _ = report` — is what let an abandoned walk be emitted as `ok: true`
    // one layer up, with a doc comment two layers down claiming the opposite.
    let stopped_early = stats.walk.as_ref().map(|w| w.stopped_early).unwrap_or(false);
    Ok(RunCompletion { stopped_early })
}

// ─── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn index_repair_status(app: AppHandle) -> RepairStatus {
    let state = app.state::<RepairState>();
    RepairStatus {
        running: state.running.load(Ordering::Relaxed),
        cancelling: state.cancel.load(Ordering::Relaxed),
        completed: state.completed.load(Ordering::Relaxed),
        total: state.total.load(Ordering::Relaxed),
        run_id: state.run_id.load(Ordering::Relaxed),
        last_error: state.last_error.lock().ok().and_then(|g| g.clone()),
    }
}

#[tauri::command]
pub fn index_repair_cancel(app: AppHandle) -> Result<(), String> {
    request_cancel(&app);
    Ok(())
}

/// PJ-207 §9 — **what changed on disk while Constellation was closed.**
///
/// A pure read of the report the boot reconcile already produced. It starts nothing,
/// walks nothing and locks nothing, so it is safe to call from the boot path — the answer
/// was computed by the pass that was going to walk the tree anyway.
///
/// `None` while that pass is still running (it is scheduled on a background thread at the
/// same moment as everything else post-paint), which is why the frontend also listens for
/// `index-drift:report` rather than polling.
#[tauri::command]
pub fn index_drift_report(app: AppHandle) -> Option<crate::reconcile::DriftReport> {
    app.state::<RepairState>().drift.lock().ok().and_then(|g| *g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_run_covers_every_later_submit_but_a_cold_start_covers_only_its_own_library() {
        let full = Scope::Full;
        let a = Scope::ColdStart {
            library_path: "/L/A".into(),
            library_name: "A".into(),
            only_if_unindexed: true,
        };
        let b = Scope::ColdStart {
            library_path: "/L/B".into(),
            library_name: "B".into(),
            only_if_unindexed: true,
        };

        assert!(full.covers(&a), "a full walk subsumes any single library");
        assert!(full.covers(&Scope::Full));
        assert!(a.covers(&a), "the same library is already covered");
        assert!(!a.covers(&b), "a DIFFERENT library must be queued, never dropped");
        assert!(
            !a.covers(&full),
            "one library's walk does not cover the whole universe — this is the boot fan-out bug",
        );
    }

    /// The boot fan-out submits one scope per library. Under a naive guard the first
    /// would start and the rest would be refused into a `.catch(() => 0)` — silently
    /// re-opening the LL-027 / BUG-022 cold-start gap for every library but the first.
    /// `covers` is what makes them queue instead.
    /// PJ-207 §7 — the `TriggerWindow` must restore the triggers on EVERY exit path,
    /// including a panic. Before this the recreate was straight-line code after the walk,
    /// so any early return left the outgoing triggers DOWN — and with them, every live
    /// save silently serving stale `note_meta.outgoing_*` until the next boot healed it.
    ///
    /// Asserted against `sqlite_master`, not against a flag: the question is whether the
    /// triggers are actually there.
    /// PJ-207 §7 — the two reasons a walk stops, and the one case it must NOT.
    #[test]
    fn a_walk_stops_on_cancel_or_a_universe_switch_and_otherwise_continues() {
        assert!(walk_may_proceed(false, 7, 7), "nothing wrong — keep walking");
        assert!(!walk_may_proceed(true, 7, 7), "cancelled: the app is closing");
        assert!(
            !walk_may_proceed(false, 8, 7),
            "the universe changed under the run — continuing would write the departing              universe's notes into the arriving one's database",
        );
        assert!(!walk_may_proceed(true, 8, 7), "both at once still stops");
    }

    #[test]
    fn the_trigger_window_restores_the_triggers_even_through_a_panic() {
        let dir = std::env::temp_dir().join(format!(
            "pj207_tw_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let conn = crate::search::init_db(&dir.join("search.db")).expect("init_db");

        let count_outgoing_triggers = |c: &rusqlite::Connection| -> i64 {
            c.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name LIKE 'note_links_outgoing%'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(-1)
        };

        let before = count_outgoing_triggers(&conn);
        assert!(before > 0, "init_db must leave the outgoing triggers in place");

        // Open a window and panic inside it. `catch_unwind` lets the test observe what
        // unwinding did — which is the whole point of RAII here.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _w = TriggerWindow::open(&conn, 1).expect("open window");
            assert_eq!(
                count_outgoing_triggers(&conn),
                0,
                "the window must actually drop them, or this test proves nothing",
            );
            panic!("simulated failure inside the trigger-free window");
        }));
        assert!(result.is_err(), "the panic must have propagated");

        assert_eq!(
            count_outgoing_triggers(&conn),
            before,
            "Drop must recreate the outgoing triggers even when the window is left by a panic",
        );
        // And the marker is KEPT — deliberately. Recreating the triggers is only half of
        // what it means: it says "the outgoing aggregates may be stale", and after a panic
        // inside the window the convergence tail provably never ran, so they are.
        // Clearing it here is precisely the HIGH finding the safety review caught: it
        // disarmed the boot heal on the exact failure the marker exists for.
        assert!(
            crate::search::outgoing_triggers_dropped_marker(&conn),
            "the unwind path must KEEP the crash marker — the tail did not run, so the next boot must heal",
        );

        // The EXPLICIT close is the other half of the contract: it recreates and reports,
        // and still does not touch the marker — the caller decides that once the tail has
        // either converged or not.
        let w = TriggerWindow::open(&conn, 2).expect("reopen");
        assert_eq!(count_outgoing_triggers(&conn), 0);
        w.close().expect("close recreates");
        assert_eq!(
            count_outgoing_triggers(&conn),
            before,
            "close() must restore the triggers",
        );
        assert!(
            crate::search::outgoing_triggers_dropped_marker(&conn),
            "close() must NOT clear the marker — that decision belongs to the tail",
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn distinct_libraries_are_never_absorbed_into_each_other() {
        let libs: Vec<Scope> = ["/L/A", "/L/B", "/L/C"]
            .iter()
            .map(|p| Scope::ColdStart {
                library_path: (*p).into(),
                library_name: p.rsplit('/').next().unwrap().into(),
                only_if_unindexed: true,
            })
            .collect();
        for (i, a) in libs.iter().enumerate() {
            for (j, b) in libs.iter().enumerate() {
                assert_eq!(a.covers(b), i == j, "only a library covers itself");
            }
        }
    }
}

#[cfg(test)]
mod tests_pj207_s8_write_scope_guard {
    //! PJ-207 §8 — the WIRING guard the behaviour tests cannot be.
    //!
    //! `search.rs`'s `tests_pj207_s8_index_write_scope` pins the **mechanism**: given the
    //! own library set and the foreign skip-set, the walk does not adopt a linked
    //! universe's note. It cannot pin the **wiring** — that production actually hands it
    //! the own set — because every one of these entry points takes an `AppHandle`, and
    //! this crate has no Tauri test harness (`Cargo.toml` carries no `tauri`/`test`
    //! feature, checked 2026-08-07). Reverting `try_load_libraries` back to
    //! `load_all_libraries` at any of the four call sites would leave the whole suite green.
    //!
    //! §7's own safety review is the reason this is here rather than left to a comment:
    //! *"a guard you have just written is exactly the guard you are least likely to
    //! check."* The precedent for making a rule structural instead of remembered is §6's
    //! `ConvergeKey`; a compiler token does not fit here (the parameter is a plain
    //! `&[LibraryInfo]` shared with a dozen legitimate READ paths), so the invariant is
    //! asserted against the source instead.
    //!
    //! The module-level checks are deliberately whole-file rather than per-function: after
    //! §8 the count is **zero**, and zero is a bound that cannot rot the way a line number
    //! does. A legitimate future read-only use inside one of these modules should not just
    //! silence this test — it should be weighed, because these modules exist to write.

    fn src_full(file: &str) -> String {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("could not read {}: {e}", p.display()))
    }

    /// A module's PRODUCTION source — everything before its first `#[cfg(test)]`. Test code
    /// may name the recursive loader freely (this very module does, in the message below);
    /// production may not. Only valid for modules whose tests sit at the END, which is why
    /// `search.rs` — whose inline test modules are scattered throughout — is checked by
    /// extracting the two function bodies instead.
    fn src_production(file: &str) -> String {
        let all = src_full(file);
        match all.find("\n#[cfg(test)]") {
            Some(i) => all[..i].to_string(),
            None => all,
        }
    }

    /// A top-level `fn`'s text: from its signature to the first lone `}` at column 0.
    /// (Nested braces are indented, so this lands on the function's own closing brace.)
    fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
        let start = source
            .find(signature)
            .unwrap_or_else(|| panic!("`{signature}` is no longer in the source — renamed? Then this guard needs updating, not deleting."));
        let rest = &source[start..];
        let end = rest.find("\n}\n").map(|i| i + 3).unwrap_or(rest.len());
        &rest[..end]
    }

    fn offending_lines(body: &str) -> Vec<&str> {
        body.lines()
            .filter(|l| l.contains("load_all_libraries") && !l.trim_start().starts_with("//"))
            .collect()
    }

    /// These three modules are index-WRITE paths end to end. None of them may resolve
    /// libraries through the federation-recursive loader.
    #[test]
    fn no_index_write_module_resolves_libraries_through_the_federation() {
        for file in ["reconcile.rs", "index_repair.rs", "library_attribution_backfill.rs"] {
            let body = src_production(file);
            let hits = offending_lines(&body);
            assert!(
                hits.is_empty(),
                "{file} resolves libraries through `load_all_libraries`, which INCLUDES the \
                 libraries of LINKED universes. Every path in this module writes or deletes \
                 rows in the ACTIVE universe's index, so its scope must come from \
                 `try_load_libraries` — the active universe's own libraries, and an ERROR \
                 rather than an empty list when the registry cannot be read. \
                 Offending line(s): {hits:?}"
            );
        }
    }

    /// The boot reconcile runs on a spawned thread with no cancel channel, computes its
    /// stale/orphan sets from the universe active at start, and writes through `state.db`
    /// — which a universe switch replaces underneath it. §7 built the generation check and
    /// wired only the bulk walk to it; the §8 safety inspection found the boot reconcile
    /// still unguarded (HIGH). Same invariant, so the same shared decision function.
    #[test]
    fn the_boot_reconcile_checks_the_universe_generation_before_it_writes() {
        let body = src_production("reconcile.rs");
        assert!(
            body.contains("federation_generation_now"),
            "reconcile.rs no longer consults the universe generation. Without it, a switch \
             mid-pass makes its re-adopt tail index the DEPARTED universe's notes into the \
             newly-active universe's index — Charter W2-9 through a second door."
        );
        assert!(
            body.contains("walk_may_proceed"),
            "reconcile.rs stopped using the SHARED stop decision. A second copy of it is how \
             the two passes drift apart; §7 and §8 deliberately share one."
        );
    }

    /// PJ-207 §9 — the drift check has no walk of its own **on purpose**, and that is the
    /// one thing about it a behaviour test cannot see.
    ///
    /// Every count it reports is produced by the boot reconcile, which was already walking
    /// exactly these roots on every launch. Give §9 its own walker and every test in this
    /// crate still passes — while the app pays the tree walk twice at the same instant, on
    /// a machine whose universes live on a USB mechanical disk. The measurements are on
    /// `reconcile::collect_md`, which owns them.
    ///
    /// So the invariant is asserted against the source: the reconcile must still hand its
    /// report over, and the drift command must remain a pure read.
    #[test]
    fn the_drift_check_reads_the_boot_reconcile_rather_than_walking_again() {
        let reconcile = src_production("reconcile.rs");
        assert!(
            reconcile.contains("record_drift_report"),
            "reconcile.rs no longer publishes its drift report. Its stale/orphan sets are \
             computed on EVERY launch and were surfaced nowhere but diagnostics.log for \
             months — on the Boss's live universe that silence was 825 notes absent from \
             search. If the report now travels some other way, prove that way reaches the \
             user before changing this assertion."
        );

        let repair = src_production("index_repair.rs");
        let cmd = fn_body(&repair, "\npub fn index_drift_report(");
        for walker in ["collect_md", "read_dir", "collect_md_paths", "reconcile_filesystem"] {
            assert!(
                !cmd.contains(walker),
                "`index_drift_report` now reaches `{walker}` — it must stay a pure read of \
                 the report the boot pass already produced. A walk behind this command runs \
                 a second full traversal of the library at boot, which is the ZERO-BOOT-WALKS \
                 rule's actual target and, on a USB disk, several seconds of it."
            );
        }
    }

    /// `search.rs` legitimately uses the recursive set in many READ paths, so the check is
    /// scoped to the two functions §8 narrowed: the bulk walk and the watcher's batch.
    #[test]
    fn the_bulk_walk_and_the_watcher_batch_take_the_own_library_set() {
        let s = src_full("search.rs");
        for sig in ["\nfn reconcile_filesystem(", "\npub fn reindex_changed_paths("] {
            let body = fn_body(&s, sig);
            let hits = offending_lines(body);
            assert!(
                hits.is_empty(),
                "`{}` resolves libraries through `load_all_libraries` — it writes index rows, \
                 so it must take the active universe's OWN set. Offending line(s): {hits:?}",
                sig.trim()
            );
            assert!(
                body.contains("try_load_libraries("),
                "`{}` no longer calls `try_load_libraries` — if the scope now comes from \
                 somewhere else, prove that somewhere else excludes linked universes before \
                 changing this assertion.",
                sig.trim()
            );
        }
    }
}

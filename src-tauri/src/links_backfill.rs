//! MIG-066 §A.2 — Resumable back-fill for the outgoing-link aggregates.
//!
//! §A.1 added three columns to `note_meta` — `outgoing_count`,
//! `outgoing_link_types`, `outgoing_top_rank` — and the
//! `note_links_outgoing_*` triggers that keep them in lock-step with live
//! edge writes. But on the first boot after the migration lands, existing
//! notes (7,600+ on the target universe) have links that predate the
//! triggers, so their columns sit at the schema defaults (0 / '' / 9).
//! This module recomputes them once from `note_links`.
//!
//! Design constraints — identical in spirit to `sky_backfill.rs` (the model):
//!
//! - **Must not block boot.** Runs on a background thread scheduled by
//!   `ensure_search_db_ready` after the connection is live and first paint
//!   has happened. The MIG-013 lesson: a single bulk `UPDATE note_meta`
//!   froze boot for tens of seconds on a large universe — never again.
//! - **Must be resumable.** `links_outgoing_backfill_cursor` holds the last
//!   processed path. Killing the app mid-run and relaunching resumes from
//!   the cursor, not from scratch.
//! - **Must coexist with live writes.** Each batch is one transaction; the
//!   DB mutex is released between batches (plus a short sleep) so user saves
//!   and other IPC calls interleave.
//! - **Idempotent.** The recompute reads the current `note_links` state, so
//!   re-running a row — or racing a trigger on the same row — converges to
//!   the same value (both read the same source of truth). New notes created
//!   during the back-fill are handled by the triggers, not here.
//!
//! Unlike `sky_backfill`, this back-fill is **pure SQL** — every value comes
//! from `note_links`, so there are no per-note file reads. That makes it far
//! lighter than the sky/stratum/maturity back-fill it mirrors.
//!
//! Completion stamps `schema_versions.links_outgoing = LINKS_OUTGOING_SCHEMA_VERSION`.
//! Next boot detects the stamp and skips the back-fill.

use rusqlite::{params, Connection};
use std::thread;
use std::time::Duration;
use tauri::Manager;

use crate::search::{outgoing_aggregate_assignments, SearchState, LINKS_OUTGOING_SCHEMA_VERSION};

/// Notes recomputed per transaction. Smaller than sky_backfill's 1000 because
/// each row here runs three correlated subqueries over `note_links` *under the
/// lock* (sky does its expensive file reads outside the lock), so we keep the
/// lock-hold per batch short. 500 indexed recomputes is a few tens of ms.
const BATCH_SIZE: i64 = 500;

/// Sleep between batches — hands the DB mutex to other callers so the
/// back-fill never starves the main thread on a large universe.
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// Schedule the back-fill on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after `init_db` completes and the
/// connection is in state. Silent no-op if `schema_versions.links_outgoing`
/// is already current.
/// **Safety inspection 2026-08-23 (B1 sweep, MED) — the PJ-332b single-flight slot,
/// extended to this module (the Whole-Ecosystem gap the sweep named).** `maybe_schedule`
/// is re-armed by every structural vocabulary save (`on_link_vocabulary_changed`), and a
/// run takes multi-seconds on a large universe — so two rapid saves used to spawn two
/// CONCURRENT recomputes pinned (B1) to two DIFFERENT registries, batch-interleaving over
/// the same rows and racing the fingerprint stamp: in the overtake-then-re-overtake
/// ordering the stamp claims the current vocabulary over a band computed under the
/// retired one — silent, and never re-healed because the stamp matches. The slot makes
/// the second save a no-op; the re-arm on clean exit re-checks the gate, sees the stale
/// fingerprint, and runs ONE follow-up pass pinned to the newest vocabulary. Copied from
/// `sky_backfill` / `review_backfill` — one consistent shape across the back-fills.
static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// B1 pass 3 — whether a run currently holds the single-flight slot. The derived
/// heal's vocabulary give-way waits on this before clearing the vocab stamp: a clear
/// issued while a run is in flight would be overwritten by that run's finalize, and
/// the re-arm gate would then read stored == disk over a band the heal poisoned.
pub(crate) fn is_running() -> bool {
    RUNNING.load(std::sync::atomic::Ordering::SeqCst)
}

/// **LL-014's three-strike law, spent — this is the structural backstop, not a fourth patch.**
///
/// The self-re-arming full-table recompute has now appeared THREE times in one cycle: removed
/// once, reintroduced by a fix for a different defect, removed again, and reintroduced a third
/// time by the guards added to make the trigger rebuild safe. Each instance had a different
/// unclearable condition, and each fix addressed that condition. That is the pattern LL-014
/// forbids continuing: after three, stop patching instances and remove the CAPABILITY.
///
/// The loop needs two ingredients: a gate term a run cannot clear, and a re-arm with no cost.
/// Every fix so far attacked the first. This attacks the second, and it holds no matter what
/// any future gate term does: a re-arm waits, and a session gets a bounded number of them.
/// A genuinely-needed pass still runs; an unclearable condition costs a few bounded passes and
/// a loud line instead of an unbounded background rewrite of the user's whole universe.
const REARM_FLOOR: Duration = Duration::from_secs(90);
const REARM_BUDGET: usize = 3;
static REARMS_THIS_SESSION: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Set by `run` when it returns early because the database was BUSY rather than because it
/// finished. A defer is not a loop — it did no work and changed nothing — so it must not spend
/// the budget above, or ordinary boot contention would exhaust the backstop and a later
/// vocabulary save dropped by the single-flight CAS would get no follow-up pass at all.
/// (Found by the frozen pass, in the backstop itself, one pass after it was written.)
static LAST_RUN_DEFERRED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn maybe_schedule(app: tauri::AppHandle) {
    // B1 pass 10 — resolve the log sink HERE, while the ambient universe is still the
    // one this run serves (see `diag_at`).
    let log_path = crate::search::db_path(&app).ok();
    // Cheap pre-check on the main thread — avoids spawning a thread for the
    // common case (already current).
    // Safety inspection 2026-08-23 (B1 pass 2, MED) — the gate compares the stored
    // stamp against the DISK registry's fingerprint: the SAME source the run pins and
    // stamps. B1 split them (gate read the in-memory global, run stamped the disk
    // read), and with the clean-exit re-arm a persistent global-vs-disk divergence
    // (the lenient boot fallback to seeds while link-types.json is briefly held) became
    // a session-long zero-delay loop of full-table re-materializes — correct values,
    // silent, WAL-churning. One source, no loop: each pass equalizes stored to disk.
    // STRICT read; a refusal skips scheduling (retried at the next boot/save/switch).
    let current_fp = {
        let root = match crate::search::db_path(&app) {
            Ok(db) => match db.parent().and_then(|c| c.parent()).map(|r| r.to_path_buf()) {
                Some(r) => r,
                None => return,
            },
            Err(_) => return,
        };
        match crate::link_types::registry_for_root(&root) {
            Ok(r) => {
                // B1 pass 10 — the strict read just SUCCEEDED. If this session started on
                // the seed fallback (an unreadable link-types.json at boot), that is the
                // moment to repair it: adopt the real vocabulary now, so the rest of the
                // session's saves stop writing seed-vocabulary aggregates and the stamp this
                // run is about to write is true rather than merely re-issued.
                if crate::link_types::recover_active_vocabulary(&app) {
                    diag_at(
                        log_path.as_deref(),
                        "[link_types] link-types.json became readable — this universe's own link types are active again; the derived link aggregates are being rebuilt under them",
                    );
                }
                r.fingerprint()
            }
            Err(e) => {
                diag_at(log_path.as_deref(), &format!("[links_backfill] NOT scheduled — vocabulary unreadable: {}", e));
                return;
            }
        }
    };
    let state = app.state::<SearchState>();
    let needs_run = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        is_needed(conn, current_fp)
    };
    if !needs_run {
        return;
    }

    // Claim the single run-slot; if a run is already in flight, do nothing — the
    // in-flight run's clean-exit re-arm below picks up whatever this call wanted.
    if RUNNING
        .compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    let app_bg = app.clone();
    thread::spawn(move || {
        let clean_exit = match run(&app_bg) {
            Ok(n) => {
                diag_at(log_path.as_deref(), &format!("[links_backfill] completed: {} notes recomputed", n));
                true
            }
            Err(e) => {
                diag_at(log_path.as_deref(), &format!("[links_backfill] FAILED: {}", e));
                false
            }
        };
        RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
        // Re-arm after release (the sky_backfill shape): a vocabulary save dropped by the
        // CAS above re-enters through `is_needed`, which compares the PINNED stamp this
        // run wrote against the now-current fingerprint — one follow-up pass, newest
        // vocabulary, no hot loop (gated on a clean exit).
        if clean_exit {
            // The backstop (see REARM_BUDGET). A re-arm is never free and never unbounded.
            let deferred = LAST_RUN_DEFERRED.load(std::sync::atomic::Ordering::SeqCst);
            let n = if deferred {
                // Contention, not a loop: retry after the floor without spending the budget.
                REARMS_THIS_SESSION.load(std::sync::atomic::Ordering::SeqCst)
            } else {
                REARMS_THIS_SESSION.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            };
            if n >= REARM_BUDGET {
                diag_at(
                    log_path.as_deref(),
                    "[links_backfill] re-arm budget spent for this session — the scheduling gate is still asking for a pass after several complete runs, which means a condition it reads is not being cleared by running. Stopping rather than rewriting the whole universe's link aggregates in a loop; the next start re-evaluates. This is worth reporting.",
                );
            } else {
                thread::sleep(REARM_FLOOR);
                maybe_schedule(app_bg);
            }
        }
    });
}

/// True when the back-fill still needs to run. Mirrors `sky_backfill::is_needed`:
/// the version is stamped only at `finalize` (completion), so an interrupted run
/// leaves it below target and re-runs, resuming from the cursor.

/// Safety inspection 2026-08-23 (B1 pass 7, MED) — the fingerprint the outgoing-aggregate
/// TRIGGERS were actually built under (`create_outgoing_link_triggers` stamps it). B1
/// pointed this module's gate and stamp at the STRICT on-disk registry while that DDL kept
/// reading the lenient in-memory global; when a transiently unreadable `link-types.json`
/// makes them disagree, the triggers write seed-vocabulary aggregates for a whole session
/// under a stored stamp that already equals the disk fingerprint — so the gate saw nothing
/// to do and those rows were never healed. Reading the trigger stamp too makes the
/// disagreement visible: the re-materialize fires and the rows come back under the
/// vocabulary on disk. Absent stamp (a database from before this pass) reads as agreeing,
/// so no universe re-runs merely for upgrading.
pub(crate) fn trigger_vocab_disagrees(conn: &Connection, current_fp: i64) -> bool {
    use rusqlite::OptionalExtension;
    match conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'trigger_vocab'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
    {
        Ok(Some(stamped)) => stamped != current_fp,
        // No stamp yet, or the read failed: do NOT manufacture work.
        _ => false,
    }
}

fn is_needed(conn: &Connection, current_fp: i64) -> bool {
    if !version_current(conn) {
        return true;
    }
    if trigger_vocab_disagrees(conn, current_fp) {
        return true;
    }
    // MIG-067 §B — vocabulary-change gate. The materialized columns (rank order,
    // per-type counts, the JSON) are derived from the active link-type vocabulary;
    // when it changes (a user adds / reorders / removes a type) the stored
    // aggregates go stale. We stamp the vocabulary fingerprint at each completed
    // back-fill; a mismatch re-runs the SAME resumable machinery to re-materialize
    // every row. This also covers the §A→§B upgrade: a universe last back-filled
    // under §A has no `links_vocab` stamp (fingerprint 0), so the seed registry's
    // non-zero fingerprint mismatches → a one-time pass fills the new JSON column.
    // The comparison fingerprint arrives from the caller's STRICT DISK read — the same
    // source the run pins and stamps (B1 pass 2: gate and run must share one source or
    // the clean-exit re-arm can loop on a stale in-memory global).
    stored_vocab_fingerprint(conn) != current_fp
}

/// True once the §A.2 back-fill version stamp has reached target — i.e. a completed
/// pass. Distinguishes a fresh first-time back-fill (version behind → keep the
/// cursor so an interrupted run resumes) from a vocabulary-change re-run (version
/// current → the cursor refers to the old vocabulary's pass and must reset).
fn version_current(conn: &Connection) -> bool {
    let v: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'links_outgoing'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    v >= LINKS_OUTGOING_SCHEMA_VERSION
}

/// The vocabulary fingerprint stamped at the last completed back-fill (0 if never
/// — e.g. a universe back-filled under §A, before the `links_vocab` stamp existed).
fn stored_vocab_fingerprint(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'links_vocab'",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

/// The back-fill loop. Re-locks the DB mutex per batch so frontend IPC stays
/// responsive. Returns the number of notes recomputed.
fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    // MIG-111 B1 — pinned ONCE, together: the database file, the universe root, and the
    // VOCABULARY, all resolved from the same universe at the same moment (the
    // name_fold_backfill shape; PJ-332's rule). This module previously re-locked the
    // SWAPPABLE SearchState.db per batch with no switch guard and read the vocabulary
    // fingerprint from the process-global — a universe switch mid-run would have
    // continued batches against the NEW universe's connection and then stamped IT
    // complete under the OLD universe's fingerprint (the PJ-332 wrong-stamp class,
    // found by the B1 investigation; the sky_backfill doc's claim that this module
    // already pinned was inaccurate). After this line the universe this run serves
    // cannot change.
    let db_file = crate::search::db_path(app)?;
    // ONE active read: the universe root is DERIVED from the pinned db path
    // (`<root>/.constellation/search.db` — `active_constellation_dir`'s own layout),
    // never read from the ambient active universe a second time. Two sequential
    // ambient reads leave a window where a switch pairs universe A's database with
    // universe B's vocabulary — the exact class B1 removes.
    let universe_root = db_file
        .parent()
        .and_then(|c| c.parent())
        .ok_or_else(|| String::from("search.db path has no universe root"))?
        .to_path_buf();
    let vocab = crate::link_types::registry_for_root(&universe_root)?;
    let mut conn = Connection::open(&db_file)
        .map_err(|e| format!("links_backfill open {}: {}", db_file.display(), e))?;
    // Safety inspection 2026-08-23 (B1 pass 13, MED) — **the busy timeout is set FIRST, before
    // anything that needs the write lock.** It used to be set further down, after the WAL
    // pragma and the cursor-table setup — harmless while this ran on the shared connection
    // under the app-wide writer mutex (which excluded other writers AND already carried a
    // timeout), but B1 moved the run onto a private connection with neither. Pass 5 then made
    // a failed `ALTER TABLE … ADD COLUMN vocab_fp` fatal rather than swallowed. Together those
    // three facts meant a single concurrent writer — the incoming back-fill's own CREATE INDEX
    // is measured at ~50 s on this universe, and both are scheduled from the same boot — hit an
    // immediate SQLITE_BUSY with a zero-millisecond timeout and aborted the whole outgoing
    // pass for the session, with no re-arm and one diagnostics line. Both siblings already set
    // the timeout before their first lock-taking statement; this module was the outlier.
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;
    // Defensive tokenizer registration (the incoming_links_backfill precedent): the
    // aggregate UPDATEs should not fire the guarded FTS trigger, but a legacy
    // note_meta_au that lost its WHEN guard would otherwise fail every batch with
    // "no such tokenizer" on this connection alone.
    crate::search::register_fts5_tokenizer(&mut conn)
        .map_err(|e| format!("register tokenizer: {}", e))?;

    // One-time setup: the resumable cursor table. Idempotent.
    //
    // Safety inspection 2026-08-23 (B1 pass 14, MED) — a BUSY here is not a reason to abandon
    // the pass for the whole session. Both this and the ANALYZE below take the write lock, and
    // the sibling incoming back-fill's `CREATE INDEX` — scheduled from the same boot — is
    // measured at ~50 s on this universe, longer than the timeout. Propagating that as Err
    // makes `clean_exit` false, which skips the re-arm, and only three sites ever schedule
    // this module: the outgoing aggregates then stay stale until the next launch, silently.
    // A contended setup returns Ok(0) instead: nothing has been written, nothing is stamped,
    // `is_needed` stays true, and the clean-exit re-arm runs it again shortly. An error that
    // is NOT contention still propagates.
    if let Err(e) = ensure_cursor_table(&conn) {
        if is_busy_message(&e) {
            diag_at(Some(&db_file), &format!("[links_backfill] setup deferred — the database was busy ({}); re-arming rather than abandoning the pass for this session", e));
            LAST_RUN_DEFERRED.store(true, std::sync::atomic::Ordering::SeqCst);
            return Ok(0);
        }
        return Err(e);
    }

    // Give the planner statistics before the correlated subqueries run. Without
    // `sqlite_stat1`, an equality on `status` (every link is 'active' — a single
    // distinct value) looks as good as the equality on `source_path`, so the
    // planner can pick the non-selective `idx_link_status` and fan each subquery
    // across the whole `note_links` table — the exact trap `sky_backfill` hit
    // (200× slower). ANALYZE is idempotent; on an existing universe sky already
    // wrote these stats, so this just refreshes them. Cheap, once, background.
    if let Err(e) = conn.execute_batch("ANALYZE") {
        let msg = format!("ANALYZE: {}", e);
        if is_busy_message(&msg) {
            diag_at(Some(&db_file), &format!("[links_backfill] planner statistics deferred — the database was busy ({}); re-arming rather than abandoning the pass for this session", msg));
            LAST_RUN_DEFERRED.store(true, std::sync::atomic::Ordering::SeqCst);
            return Ok(0);
        }
        return Err(msg);
    }

    // MIG-067 §B — capture the vocabulary fingerprint for THIS run up-front; it is
    // stamped at finalize. If the vocabulary changes again mid-run, the stamp will
    // differ from the then-current fingerprint and `is_needed` re-runs us next time
    // (eventual consistency, without tracking the vocabulary per batch).
    // B1 — the fingerprint of the PINNED vocabulary, not the process-global: the stamp
    // below then describes exactly the registry these aggregates were computed with.
    let run_fp = vocab.fingerprint();

    // MIG-067 §B — if the version is already current, this run was triggered purely
    // by the vocabulary-change gate; any cursor left by a prior run belongs to the
    // OLD vocabulary's pass, so reset it and re-materialize every row (not just the
    // tail). A first-time back-fill (version behind) keeps its cursor so an
    // interrupted run resumes from where it stopped — but ONLY if that cursor's band
    // was computed under THIS run's vocabulary.
    //
    // Safety inspection 2026-08-23 (B1 diff pass, MED) — the version gate alone was
    // not enough: a FIRST-TIME run interrupted at cursor C under vocabulary V1, then
    // resumed after an edit to V2, kept C (version still behind ⇒ no reset), recomputed
    // only (C, end] under V2, and stamped fingerprint(V2) — every row ≤ C permanently
    // carried V1-derived aggregates under a stamp claiming V2, and `is_needed` never
    // fired again. The cursor now records its band's fingerprint; a mismatch (including
    // the 0 of a pre-column cursor) resets to a full re-materialize.
    if version_current(&conn) {
        conn.execute("DELETE FROM links_outgoing_backfill_cursor", [])
            .map_err(|e| format!("vocab-change cursor reset: {}", e))?;
    }

    // The generation stop (the sky_backfill shape): our connection is pinned, so
    // continuing after a switch is never a correctness hazard — only wasted I/O on a
    // departed universe. Stop at the loop top; the cursor makes the next boot resume.
    let gen0 = crate::search::federation_generation_now(app);
    let still_ours = || crate::search::federation_generation_now(app) == gen0;

    // B1 pass 8 — REPAIR, not just detect. If the live triggers were built under a
    // vocabulary that disagrees with this run's pinned one, rebuild them HERE, under the
    // registry read strictly from this universe's own root — the vocabulary they should
    // have carried. This also re-stamps `trigger_vocab`, which is what makes the gate
    // condition clearable: without it the clean-exit re-arm would re-enter a gate that can
    // never go quiet. Best-effort: a failed rebuild leaves the stamp disagreeing, so the
    // gate simply tries again next time rather than looping inside this run.
    if trigger_vocab_disagrees(&conn, run_fp) {
        // Safety inspection 2026-08-23 (B1 pass 12, MED) — the rebuild is a DROP followed by
        // a CREATE, in two separate implicit transactions. Pass 8 made this the first caller
        // to run that sequence from a DETACHED thread on a private connection with the app
        // fully live: if the CREATE half fails after the DROP has committed, the outgoing
        // aggregate triggers are simply GONE for the rest of the session — and since they
        // are the only live maintainer of `outgoing_count` / `outgoing_link_types` /
        // `outgoing_link_types_json` / `outgoing_top_rank`, every save silently stops
        // maintaining them. The run would then stamp completion, and the next boot's
        // `init_db` would recreate the triggers AND re-stamp `trigger_vocab`, satisfying
        // every term of the gate — so the notes edited inside that window keep their stale
        // breakdown permanently, in the sidebar, Base columns and Reviewer rank.
        //
        // The codebase already owns the right shape for a trigger-free window: arm a crash
        // marker BEFORE dropping and clear it only after a successful recreate, so a boot
        // that finds the marker armed heals the families. `index_repair`'s `TriggerWindow`
        // is that abstraction; the marker functions are used directly here because the
        // rebuild is a single call rather than a long walk.
        // Safety inspection 2026-08-23 (B1 pass 14, MED) — **two guards the sibling already
        // had and this fix, written one pass earlier to "use the existing shape", did not.**
        //
        // The marker is a SINGLE shared row with no owner field, even though `RepairState`
        // documents run-id ownership as the thing that stops "a second run clearing a first
        // run's crash protection". `derived_heal` gates its identical clear on
        // `index_repair::is_running` for exactly this reason, in a comment that says so. This
        // module did neither: it armed over whatever was there, recreated the triggers, and
        // then DELETED the row — which, mid-repair, both reintroduces the per-edge trigger
        // cost that repair's trigger-free window exists to avoid AND disarms its crash net, so
        // an app close mid-walk leaves incoming / sky / tag_counts / review_schedule unhealed
        // and stamp-gated against ever re-running.
        //
        // So: never while a repair is running, and never clear a marker we did not arm.
        if crate::index_repair::is_running(app) {
            diag_at(Some(&db_file), "[links_backfill] outgoing trigger rebuild SKIPPED — an index repair is running and owns the trigger-free window; the aggregates below are still recomputed, and the repair rebuilds the triggers itself");
        } else if crate::search::outgoing_triggers_dropped_marker(&conn) {
            diag_at(Some(&db_file), "[links_backfill] outgoing trigger rebuild SKIPPED — a crash marker is already armed by another run; recreating the triggers now would clear a net this run does not own. The aggregates below are still recomputed; the armed marker heals the rest at the next start.");
        } else if let Err(e) = crate::search::set_outgoing_triggers_dropped_marker(&conn) {
            diag_at(Some(&db_file), &format!("[links_backfill] outgoing trigger rebuild SKIPPED — its crash marker could not be armed ({}); refusing to enter a trigger-free window that a crash could not heal. The aggregates below are still recomputed correctly.", e));
        } else {
            match crate::search::create_outgoing_link_triggers_with(&conn, &vocab) {
                Ok(()) => {
                    let _ = crate::search::clear_outgoing_triggers_dropped_marker(&conn);
                    diag_at(Some(&db_file), "[links_backfill] outgoing triggers rebuilt under this universe's own vocabulary (they had been built under a different one)");
                }
                Err(e) => {
                    // Marker deliberately LEFT ARMED: the triggers may be dropped right now,
                    // and the armed marker is what makes the next start heal these families.
                    diag_at(Some(&db_file), &format!("[links_backfill] outgoing trigger rebuild FAILED ({}) — the live outgoing-aggregate triggers may be absent for the rest of this session, so notes saved from now on can keep a stale link-type breakdown until the next start, which will heal them (its crash marker is armed). The aggregates recomputed below are correct.", e));
                }
            }
        }
    }

    LAST_RUN_DEFERRED.store(false, std::sync::atomic::Ordering::SeqCst);
    let (mut last_path, cursor_fp) = read_cursor(&conn)?;
    if !last_path.is_empty() && cursor_fp != run_fp {
        diag_at(Some(&db_file), &format!(
            "[links_backfill] cursor band was computed under a different vocabulary (fp {} ≠ {}) — restarting the pass from the top",
            cursor_fp, run_fp
        ));
        conn.execute("DELETE FROM links_outgoing_backfill_cursor", [])
            .map_err(|e| format!("stale-vocab cursor reset: {}", e))?;
        last_path = String::new();
    }
    let mut total: u64 = 0;

    loop {
        if !still_ours() {
            return Ok(total);
        }
        // Retry a busy batch instead of aborting the whole pass — the shape
        // `recompute_all_outgoing` / `_incoming` / `_sky` already use. Without it a single
        // concurrent save could end a full-universe re-materialize with one diag line.
        let mut attempt = 0;
        let (batch_count, new_last_path) = loop {
            match process_batch(&mut conn, &last_path, &vocab) {
                Ok(v) => break v,
                Err(e) if attempt < 8 && is_busy_message(&e) => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(200 * attempt as u64));
                }
                Err(e) => return Err(e),
            }
        };
        if batch_count == 0 {
            // Drained. Stamp the version + vocabulary fingerprint and clear the
            // cursor atomically.
            finalize(&mut conn, run_fp)?;
            return Ok(total);
        }
        total += batch_count as u64;
        last_path = new_last_path;
        write_cursor(&conn, &last_path, run_fp)?;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

/// One batch under a single lock+transaction: read the next window of paths
/// (to find this batch's upper boundary), then recompute the three aggregates
/// for every note in `(after_path, last_path]`. The SELECT and the UPDATE share
/// one transaction so the range can't shift underneath us. The lock is released
/// when this function returns, before the inter-batch sleep.
///
/// Returns `(notes_in_batch, new_cursor)`. A 0 count means the table is drained.
fn process_batch(
    conn: &mut Connection,
    after_path: &str,
    reg: &crate::link_types::LinkTypeRegistry,
) -> Result<(usize, String), String> {
    // Safety inspection 2026-08-23 (B1 pass 9, LOW) — IMMEDIATE, not DEFERRED. This batch
    // reads (the path window) and then writes (the aggregate UPDATE); on a DEFERRED
    // transaction that is a snapshot UPGRADE, and SQLite does NOT invoke the busy handler
    // for it — so the 30 s busy_timeout is no protection and a concurrent app write aborts
    // the pass with SQLITE_BUSY_SNAPSHOT. B1 made this reachable by moving off the app-wide
    // writer mutex onto a private connection (the right move for pinning, but it removed
    // the mutual exclusion this DEFERRED transaction had been relying on). Taking the write
    // lock up front makes the busy handler apply, and the caller retries on top of that.
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| format!("begin: {}", e))?;

    // Next window of paths — only the boundary + count are used; the recompute
    // itself is range-scoped (no big IN-list), so this stays cheap.
    let paths: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT path FROM note_meta WHERE path > ?1 ORDER BY path LIMIT ?2")
            .map_err(|e| format!("prepare batch: {}", e))?;
        let rows = stmt
            .query_map(params![after_path, BATCH_SIZE], |row| row.get::<_, String>(0))
            .map_err(|e| format!("query batch: {}", e))?;
        let mut v = Vec::with_capacity(BATCH_SIZE as usize);
        for r in rows {
            v.push(r.map_err(|e| format!("row batch: {}", e))?);
        }
        v
    };

    if paths.is_empty() {
        tx.commit().map_err(|e| format!("commit empty: {}", e))?;
        return Ok((0, after_path.to_string()));
    }

    let last_path = paths.last().cloned().unwrap_or_default();
    recompute_range(&tx, reg, after_path, &last_path)
        .map_err(|e| format!("recompute range: {}", e))?;
    tx.commit().map_err(|e| format!("commit: {}", e))?;

    Ok((paths.len(), last_path))
}

/// The core recompute: set the three outgoing-link aggregates for every
/// `note_meta` row in `(after_path, last_path]` from `note_links`, using the
/// SAME SQL the §A.1 triggers use (via `outgoing_aggregate_assignments`, here
/// correlated on `note_meta.path`). Shared by `process_batch` and the tests so
/// the back-fill and the triggers can never drift. Returns rows touched.
pub(crate) fn recompute_range(conn: &Connection, reg: &crate::link_types::LinkTypeRegistry, after_path: &str, last_path: &str) -> rusqlite::Result<usize> {
    let sql = format!(
        "UPDATE note_meta SET {assign} WHERE path > ?1 AND path <= ?2",
        assign = outgoing_aggregate_assignments(reg, "note_meta.path"),
    );
    conn.execute(&sql, params![after_path, last_path])
}

/// MIG-066 §A.2 — recompute the outgoing aggregates for EVERY note from
/// `note_links`. `reconcile_filesystem` calls this after a deliberately
/// trigger-free full re-index to restore the columns. Same SQL the triggers +
/// the batched back-fill use, so the three population paths can never drift.
/// Returns rows touched.
///
/// **BATCHED + lock-tolerant** (was a single whole-table UPDATE — which silently
/// failed under boot DB contention on a large universe, leaving the column stale:
/// the 2026-05-30 overnight blank). It now walks `note_meta` in 500-row windows,
/// each its own short UPDATE (so it never holds a long write lock), and retries a
/// batch on SQLITE_BUSY/locked instead of aborting the whole pass.
pub(crate) fn recompute_all_outgoing(conn: &Connection, reg: &crate::link_types::LinkTypeRegistry, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize> {
    let mut after = String::new();
    let mut total = 0usize;
    loop {
        let paths: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT path FROM note_meta WHERE path > ?1 ORDER BY path LIMIT 500")?;
            let rows = stmt.query_map(params![after], |r| r.get::<_, String>(0))?;
            let mut v = Vec::with_capacity(500);
            for r in rows {
                v.push(r?);
            }
            v
        };
        if paths.is_empty() {
            break;
        }
        let last = paths.last().cloned().unwrap_or_default();
        // One short UPDATE per window; retry on transient lock contention.
        let mut attempt = 0;
        loop {
            match recompute_range(conn, reg, &after, &last) {
                Ok(_) => break,
                Err(e) if is_busy_error(&e) && attempt < 8 => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(400));
                }
                Err(e) => return Err(e),
            }
        }
        total += paths.len();
        after = last;
    }
    Ok(total)
}

/// MIG-079 §C.2a — recompute the INCOMING-link aggregates for a `(after, last]`
/// path window from `note_links` (the same `incoming_aggregate_assignments` SQL
/// the triggers use — single source of truth, can't drift). Shared by
/// `recompute_all_incoming` and the §C.2a backfill.
pub(crate) fn recompute_incoming_range(conn: &Connection, reg: &crate::link_types::LinkTypeRegistry, after: &str, last: &str) -> rusqlite::Result<usize> {
    let sql = format!(
        "UPDATE note_meta SET {assign} WHERE path > ?1 AND path <= ?2",
        assign = crate::search::incoming_aggregate_assignments(reg, "note_meta"),
    );
    conn.execute(&sql, params![after, last])
}

/// MIG-079 §C.2a — recompute EVERY note's incoming aggregate from `note_links`.
/// `reconcile_filesystem` calls this after the trigger-free walk; the §C.2a
/// backfill calls it once on first upgrade. Batched (500-row windows, each its own
/// short UPDATE) + busy-retry — mirrors `recompute_all_outgoing` so it never holds
/// a long write lock on a large universe. Idempotent (reads current note_links).
pub(crate) fn recompute_all_incoming(conn: &Connection, reg: &crate::link_types::LinkTypeRegistry, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize> {
    let mut after = String::new();
    let mut total = 0usize;
    loop {
        let paths: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT path FROM note_meta WHERE path > ?1 ORDER BY path LIMIT 500")?;
            let rows = stmt.query_map(params![after], |r| r.get::<_, String>(0))?;
            let mut v = Vec::with_capacity(500);
            for r in rows {
                v.push(r?);
            }
            v
        };
        if paths.is_empty() {
            break;
        }
        let last = paths.last().cloned().unwrap_or_default();
        let mut attempt = 0;
        loop {
            match recompute_incoming_range(conn, reg, &after, &last) {
                Ok(_) => break,
                Err(e) if is_busy_error(&e) && attempt < 8 => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(400));
                }
                Err(e) => return Err(e),
            }
        }
        total += paths.len();
        after = last;
    }
    Ok(total)
}

/// PJ-066 §B1 — recompute `sky_nodes.stratum` + `maturity` for a `(after, last]` path
/// window from `note_links`, using the SAME shared `STRATUM_SQL_EXPR` / `MATURITY_SQL_EXPR`
/// the triggers + sky_backfill use (single source of truth — cannot drift). One combined
/// UPDATE per window. Replaces the per-edge sky triggers' work on the bulk/reconcile path.
pub(crate) fn recompute_sky_range(conn: &Connection, reg: &crate::link_types::LinkTypeRegistry, after: &str, last: &str) -> rusqlite::Result<usize> {
    // B1 — the registry arrives from the caller's pinned scope (the B4 annotation's
    // promise, kept).
    let sql = format!(
        "UPDATE sky_nodes SET stratum = ({stratum}), maturity = ({maturity}) WHERE path > ?1 AND path <= ?2",
        stratum = crate::search::stratum_sql_expr(reg),
        maturity = crate::search::maturity_sql_expr(reg),
    );
    conn.execute(&sql, params![after, last])
}

/// PJ-066 §B1 — recompute EVERY note's sky stratum + maturity from `note_links`.
/// `reconcile_filesystem` calls this after the trigger-free bulk walk (the per-edge sky
/// stratum/maturity triggers are dropped by §B4, so reconcile no longer maintains sky via
/// triggers — this is the replacement). Batched (500-row windows) + busy-retry, mirroring
/// `recompute_all_incoming` so it never holds a long write lock on a large universe.
/// Idempotent (reads current note_links); unconditional (self-heals stale values).
pub(crate) fn recompute_all_sky(conn: &Connection, reg: &crate::link_types::LinkTypeRegistry, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize> {
    let mut after = String::new();
    let mut total = 0usize;
    loop {
        let paths: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT path FROM sky_nodes WHERE path > ?1 ORDER BY path LIMIT 500")?;
            let rows = stmt.query_map(params![after], |r| r.get::<_, String>(0))?;
            let mut v = Vec::with_capacity(500);
            for r in rows {
                v.push(r?);
            }
            v
        };
        if paths.is_empty() {
            break;
        }
        let last = paths.last().cloned().unwrap_or_default();
        let mut attempt = 0;
        loop {
            match recompute_sky_range(conn, reg, &after, &last) {
                Ok(_) => break,
                Err(e) if is_busy_error(&e) && attempt < 8 => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(400));
                }
                Err(e) => return Err(e),
            }
        }
        total += paths.len();
        after = last;
    }
    Ok(total)
}

/// True for SQLITE_BUSY / SQLITE_LOCKED (the transient contention worth retrying).
/// The string form of the busy test — `process_batch` returns `String`, not
/// `rusqlite::Error`, so the typed check below cannot be reused directly.
fn is_busy_message(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("database is locked") || m.contains("busy")
}

fn is_busy_error(e: &rusqlite::Error) -> bool {
    let s = e.to_string().to_lowercase();
    s.contains("locked") || s.contains("busy")
}

fn ensure_cursor_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS links_outgoing_backfill_cursor (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_path TEXT,
            started_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )
    .map_err(|e| format!("cursor table create: {}", e))?;
    // Safety inspection 2026-08-23 (B1 diff pass, MED) — the cursor records the
    // FINGERPRINT of the vocabulary its band was computed under, so a resume can tell
    // whether the rows at or below it still belong to this run's registry. Guarded
    // ALTER for tables created before the column existed; 0 = unknown (pre-column
    // cursor), which deliberately mismatches every real fingerprint and forces the
    // safe full re-materialize.
    // Safety inspection 2026-08-23 (B1 pass 5, MED) — tolerate ONLY the expected
    // "duplicate column" (the column already exists); propagate every OTHER failure
    // (busy/locked/io) so the run aborts HERE, before any destructive step reads
    // through the missing column. The old `let _` swallowed a busy-timeout failure,
    // read_cursor's .ok() then collapsed "no such column" into an EMPTY cursor, and
    // sky's whole-table wipe ran scoped `path > ''`.
    if let Err(e) = conn.execute(
        "ALTER TABLE links_outgoing_backfill_cursor ADD COLUMN vocab_fp INTEGER NOT NULL DEFAULT 0",
        [],
    ) {
        let msg = e.to_string();
        if !msg.contains("duplicate column") {
            return Err(format!("cursor vocab_fp column: {}", msg));
        }
    }
    Ok(())
}

fn read_cursor(conn: &Connection) -> Result<(String, i64), String> {
    let row: Option<(String, i64)> = conn
        .query_row(
            "SELECT COALESCE(last_path, ''), COALESCE(vocab_fp, 0) FROM links_outgoing_backfill_cursor WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    Ok(row.unwrap_or_default())
}

fn write_cursor(conn: &Connection, last_path: &str, vocab_fp: i64) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO links_outgoing_backfill_cursor (id, last_path, vocab_fp) VALUES (1, ?1, ?2)",
        params![last_path, vocab_fp],
    )
    .map_err(|e| format!("cursor write: {}", e))?;
    Ok(())
}

fn finalize(conn: &mut Connection, vocab_fingerprint: i64) -> Result<(), String> {
    // Stamp + cursor clear in one transaction so a crash between them can't
    // leave a completed back-fill with a live cursor row (which the next boot
    // would read as an interrupted run).
    let tx = conn.transaction().map_err(|e| format!("finalize begin: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('links_outgoing', ?1, strftime('%s','now'))",
        params![LINKS_OUTGOING_SCHEMA_VERSION],
    )
    .map_err(|e| format!("finalize stamp: {}", e))?;
    // MIG-067 §B — stamp the vocabulary fingerprint these aggregates were
    // materialized under, so a later vocabulary change re-triggers the back-fill
    // (see `is_needed`). Stored in `schema_versions` (version column = fingerprint)
    // — no new table; the value is an opaque i64 token, not an ordered version.
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('links_vocab', ?1, strftime('%s','now'))",
        params![vocab_fingerprint],
    )
    .map_err(|e| format!("finalize vocab stamp: {}", e))?;
    tx.execute("DELETE FROM links_outgoing_backfill_cursor", [])
        .map_err(|e| format!("finalize cursor: {}", e))?;
    tx.commit().map_err(|e| format!("finalize commit: {}", e))?;
    Ok(())
}

/// Write a line to the universe's diagnostics log. Thin wrapper around
/// `search::diag_log` — kept here so this module doesn't reach into the
/// search module's private helpers.
/// Safety inspection 2026-08-23 (B1 pass 10, LOW) — **the log line goes to the universe the
/// work was done for.** `diag` resolves its sink from the AMBIENT active universe at call
/// time, so a thread pinned to universe A that finishes after a switch to B writes A's
/// completion / FAILED / reset lines into B's `diagnostics.log`: absent where an
/// investigator would look, and present-but-wrong where they would not. `finalize` was
/// already given a pinned path for exactly this reason; the surrounding lines were not.
/// The path is resolved ONCE at schedule time, when ambient is still correct, and carried.
fn diag_at(db_file: Option<&std::path::Path>, msg: &str) {
    if let Some(p) = db_file {
        crate::search::diag_log(p, msg);
    }
}


#[cfg(test)]
mod tests {
    //! MIG-066 §A.2 — pins the back-fill's core recompute (`recompute_range`,
    //! the same `outgoing_aggregate_assignments` SQL production runs) against the
    //! bundled SQLite: it populates pre-existing rows from `note_links`, honors the
    //! canonical order + the rank sentinel, excludes archived edges, is range-scoped,
    //! and is idempotent on re-run. The scheduler/cursor/threading is mirrored from
    //! the proven `sky_backfill`, so the novel part — the recompute — is what we test.
    use super::*;


    /// **B1 pass 8 — the loop this build created, pinned so it cannot return.**
    ///
    /// Pass 7 added a `trigger_vocab` gate term so the back-fill could SEE that the live
    /// outgoing triggers had been built under a vocabulary disagreeing with the one on
    /// disk. Nothing a run did could write that stamp, so the term could never be cleared —
    /// and with the clean-exit re-arm, one disagreement became a permanent, zero-delay loop
    /// of full-table re-materializes on the user's largest universe. Detection without
    /// repair is not a fix; it is a spin.
    ///
    /// The two properties that make the loop impossible, asserted directly:
    ///   1. rebuilding the triggers under a registry CLEARS the disagreement for that
    ///      registry (so the run that repairs is the run that quiets the gate), and
    ///   2. an ABSENT stamp reads as agreeing (so no universe re-runs merely by upgrading
    ///      to a build that has the stamp).
    #[test]
    fn a_trigger_vocabulary_disagreement_is_cleared_by_rebuilding_the_triggers() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                outgoing_link_types TEXT NOT NULL DEFAULT '', outgoing_link_types_json TEXT NOT NULL DEFAULT '{}',
                outgoing_top_rank INTEGER NOT NULL DEFAULT 9
             );
             CREATE TABLE note_links (
                source_path TEXT, target_name TEXT, link_type TEXT, status TEXT DEFAULT 'active'
             );
             CREATE TABLE schema_versions (
                module TEXT PRIMARY KEY, version INTEGER NOT NULL, updated_at INTEGER
             );",
        )
        .unwrap();

        let reg = crate::link_types::LinkTypeRegistry::seeds_only();
        let fp = reg.fingerprint();
        assert_ne!(fp, 0, "guard: the seed fingerprint is non-zero, so 0 is a distinguishable 'unknown'");

        // (2) No stamp at all — an existing universe upgrading into this build.
        assert!(
            !trigger_vocab_disagrees(&conn, fp),
            "an ABSENT trigger stamp must read as agreeing; otherwise every universe re-runs a full pass on upgrade"
        );

        // A disagreement, as a lenient-boot global would leave it.
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('trigger_vocab', ?1)",
            params![fp + 1],
        )
        .unwrap();
        assert!(
            trigger_vocab_disagrees(&conn, fp),
            "a stamp naming a different vocabulary must be seen"
        );

        // (1) The repair the run performs — and the gate goes quiet.
        crate::search::create_outgoing_link_triggers_with(&conn, &reg).unwrap();
        assert!(
            !trigger_vocab_disagrees(&conn, fp),
            "rebuilding the triggers under this registry must CLEAR the disagreement — if it \
             cannot, the clean-exit re-arm turns the gate into an endless full-table recompute"
        );
    }

    /// **The ratchet, pass 14.** Thirty-seven behavioural fixes in this cycle were guarded by
    /// two assertions, and the close panel named that as the MECHANISM behind the cycle's own
    /// headline: most serious findings lived in code the cycle itself had just written, which
    /// is exactly what an unguarded fix sequence produces. Fourteen reading passes re-derived
    /// by inspection what these hold for free. Each test below pins one fix whose failure mode
    /// is silent.

    /// A contended setup must DEFER, never abandon the pass for the session. `run` returns
    /// Ok(0) on a busy setup precisely so `clean_exit` stays true and the re-arm fires; if
    /// this classification is wrong, the busy error propagates, `clean_exit` goes false, and
    /// the outgoing aggregates stay stale until the next launch with one log line.
    #[test]
    fn busy_errors_are_classified_so_a_contended_setup_defers_instead_of_abandoning() {
        for msg in [
            "cursor table create: database is locked",
            "ANALYZE: database is locked",
            "database is BUSY",
            "cursor vocab_fp column: database is locked",
        ] {
            assert!(is_busy_message(msg), "must be treated as contention: {msg}");
        }
        for msg in [
            "cursor vocab_fp column: no such table: links_outgoing_backfill_cursor",
            "recompute range: no such column: outgoing_top_rank",
            "disk I/O error",
        ] {
            assert!(
                !is_busy_message(msg),
                "must NOT be silently deferred — a real fault has to propagate: {msg}"
            );
        }
    }

    /// The cursor carries the fingerprint of the vocabulary its band was computed under, and a
    /// resume under a DIFFERENT vocabulary must restart from the top. Without this a
    /// first-time pass interrupted under V1 and resumed under V2 recomputes only the tail,
    /// stamps V2, and leaves every row at or below the cursor permanently describing V1 under
    /// a stamp that claims otherwise.
    #[test]
    fn a_cursor_band_from_another_vocabulary_is_not_resumed() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_cursor_table(&conn).unwrap();

        // A pre-column cursor (an existing universe upgrading into this build) reads as
        // fingerprint 0 — deliberately unequal to every real fingerprint.
        conn.execute(
            "INSERT OR REPLACE INTO links_outgoing_backfill_cursor (id, last_path) VALUES (1, '/m.md')",
            [],
        )
        .unwrap();
        let (path, fp) = read_cursor(&conn).unwrap();
        assert_eq!(path, "/m.md");
        assert_eq!(fp, 0, "an upgraded cursor must read as UNKNOWN, forcing a full pass");

        write_cursor(&conn, "/m.md", 4242).unwrap();
        assert_eq!(read_cursor(&conn).unwrap(), ("/m.md".to_string(), 4242));
    }

    /// `ensure_cursor_table` must tolerate ONLY "duplicate column" and propagate everything
    /// else: it is called before any destructive step, and swallowing a real failure there let
    /// `read_cursor` collapse a missing column into an EMPTY cursor — which, in the sibling
    /// module, widened a stratum/maturity wipe to the whole table.
    #[test]
    fn the_cursor_setup_is_idempotent_and_does_not_swallow_real_failures() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_cursor_table(&conn).unwrap();
        // Second call: the ALTER now fails with "duplicate column" and MUST be tolerated.
        ensure_cursor_table(&conn).unwrap();
        assert!(read_cursor(&conn).is_ok());
    }

    /// Seed note_meta (columns at schema defaults) + note_links, WITHOUT the
    /// triggers — exactly the back-fill's scenario: links that predate them.
    fn seeded_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                outgoing_link_types TEXT NOT NULL DEFAULT '', outgoing_link_types_json TEXT NOT NULL DEFAULT '{}',
                outgoing_top_rank INTEGER NOT NULL DEFAULT 9
             );
             CREATE TABLE note_links (
                source_path TEXT, target_name TEXT, link_type TEXT, status TEXT DEFAULT 'active'
             );",
        )
        .unwrap();
        for p in ["/a.md", "/b.md", "/c.md"] {
            conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![p]).unwrap();
        }
        // /a.md: two typed (reverse canonical order) + one untyped + one ARCHIVED.
        // /b.md: one typed. /c.md: no links at all (stays at the default sentinel).
        let edges = [
            ("/a.md", "T1", "contradicts", "active"),
            ("/a.md", "T2", "supports", "active"),
            ("/a.md", "T3", "", "active"),
            ("/a.md", "T4", "causes", "archived"),
            ("/b.md", "T5", "exemplifies", "active"),
        ];
        for (s, t, lt, st) in edges {
            conn.execute(
                "INSERT INTO note_links (source_path, target_name, link_type, status) VALUES (?1, ?2, ?3, ?4)",
                params![s, t, lt, st],
            )
            .unwrap();
        }
        conn
    }

    fn read(conn: &Connection, path: &str) -> (i64, String, i64) {
        conn.query_row(
            "SELECT outgoing_count, outgoing_link_types, outgoing_top_rank FROM note_meta WHERE path = ?1",
            params![path],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    }

    #[test]
    fn backfill_populates_existing_rows() {
        let conn = seeded_db();
        // Pre-state: everything at the schema default.
        assert_eq!(read(&conn, "/a.md"), (0, String::new(), 9));

        // Full-range recompute (what process_batch runs per batch, here in one go).
        let touched = recompute_range(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), "", "/zzz").unwrap();
        assert_eq!(touched, 3, "all three note_meta rows in range are recomputed");

        // /a.md: archived 'causes' excluded → count 3 (supports/contradicts/untyped),
        // types in canonical order (supports=1 before contradicts=2), top rank = 1.
        assert_eq!(read(&conn, "/a.md"), (3, "supports (1), contradicts (1)".to_string(), 1));
        // /b.md: one typed link.
        assert_eq!(read(&conn, "/b.md"), (1, "exemplifies (1)".to_string(), 4));
        // /c.md: genuinely no links → recompute yields the same default sentinel.
        assert_eq!(read(&conn, "/c.md"), (0, String::new(), 9));
    }

    /// MIG-067 §B — the vocabulary-change gate. With the version already at target,
    /// `is_needed` is driven purely by the stored-vs-current vocabulary fingerprint:
    /// absent (a §A-era universe) → needed; matching → not needed; differing (a
    /// vocabulary edit) → needed again. (The global registry defaults to the 8 seeds
    /// in tests, so `snapshot().fingerprint()` is stable here.)
    #[test]
    fn vocab_fingerprint_gate_triggers_rematerialize() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        // §A.2 version satisfied — the only remaining driver is the fingerprint.
        conn.execute(
            "INSERT INTO schema_versions (module, version) VALUES ('links_outgoing', ?1)",
            params![LINKS_OUTGOING_SCHEMA_VERSION],
        )
        .unwrap();

        // No `links_vocab` stamp (a universe back-filled under §A) → stored 0 ≠ the
        // seed registry's non-zero fingerprint → re-trigger (fills the JSON column).
        let seeds_fp = crate::link_types::LinkTypeRegistry::seeds_only().fingerprint();
        assert!(is_needed(&conn, seeds_fp), "missing vocab stamp must re-trigger the back-fill");

        // Stamp the CURRENT fingerprint → in sync → not needed.
        let fp = seeds_fp;
        assert_ne!(fp, 0, "seed registry fingerprint is non-zero");
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('links_vocab', ?1)",
            params![fp],
        )
        .unwrap();
        assert!(!is_needed(&conn, seeds_fp), "matching vocab stamp must NOT re-trigger");

        // Simulate a vocabulary edit: a different stored fingerprint → needed again.
        conn.execute(
            "UPDATE schema_versions SET version = ?1 WHERE module = 'links_vocab'",
            params![fp ^ 0x5555],
        )
        .unwrap();
        assert!(is_needed(&conn, seeds_fp), "changed vocab fingerprint must re-trigger");
    }

    #[test]
    fn backfill_is_range_scoped() {
        let conn = seeded_db();
        // Recompute only (after "", up to and including "/a.md") — paths sort
        // "/a.md" < "/b.md" < "/c.md", so only /a.md is in range.
        recompute_range(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), "", "/a.md").unwrap();
        assert_eq!(read(&conn, "/a.md"), (3, "supports (1), contradicts (1)".to_string(), 1));
        assert_eq!(read(&conn, "/b.md"), (0, String::new(), 9), "/b.md is outside the range — untouched");
    }

    #[test]
    fn backfill_is_idempotent() {
        let conn = seeded_db();
        recompute_range(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), "", "/zzz").unwrap();
        let first = read(&conn, "/a.md");
        // Re-running over the same range converges to the identical value.
        recompute_range(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), "", "/zzz").unwrap();
        assert_eq!(read(&conn, "/a.md"), first, "recompute is deterministic from note_links");
    }

    /// MIG-066 §A.2 perf gate (Rule 8 / WA#4). The only thing §A.2 can regress is
    /// the §A.1 `note_links_outgoing_*` triggers firing per-edge during a full
    /// re-index (each note's links are rebuilt via per-source DELETE + re-INSERT —
    /// the `index_note` shape, search.rs:3850). This isolates that family's
    /// MARGINAL cost: it times the identical full rebuild over a 7,600-note /
    /// ~217k-link synthetic universe WITHOUT the triggers (baseline = "before")
    /// then WITH them ("after"). The other 3 note_links trigger families (sky /
    /// maturity / stratum) are unchanged by this MIG, so they cancel in the delta
    /// and are omitted — the delta IS the regression attributable to §A.1+§A.2.
    ///
    /// Run (release, so the rusqlite glue is optimized):
    ///   cargo test --release --lib --manifest-path src-tauri/Cargo.toml \
    ///     -- --ignored --nocapture bench_reindex_trigger_overhead
    #[test]
    #[ignore = "perf benchmark — run explicitly with --ignored --nocapture"]
    fn bench_reindex_trigger_overhead() {
        use std::time::Instant;

        const N: usize = 7_600;
        // 9 link-type slots: the 8 canonical types + untyped (the real on-disk mix).
        let types = [
            "supports", "contradicts", "causes", "exemplifies", "generalizes",
            "derives-from", "part-of", "supersedes", "",
        ];

        // Skewed link plan: links_i = 5 + (i % 48) → 5..52 per note, avg ~28.5 →
        // ~217k total, matching the target universe's note_links row count.
        let mut plan: Vec<(String, Vec<(String, String)>)> = Vec::with_capacity(N);
        let mut total_links = 0usize;
        for i in 0..N {
            let src = format!("/lib/note_{:05}.md", i);
            let k = 5 + (i % 48);
            let mut edges = Vec::with_capacity(k);
            for j in 0..k {
                let tgt = format!("Target {}", (i + j * 13) % N);
                let lt = types[(i + j) % types.len()].to_string();
                edges.push((tgt, lt));
            }
            total_links += k;
            plan.push((src, edges));
        }

        // Temp file DB (real WAL behavior, not in-memory) — the production shape.
        let db_file = std::env::temp_dir().join("mig066_bench_reindex.db");
        let _ = std::fs::remove_file(&db_file);
        let _ = std::fs::remove_file(db_file.with_extension("db-wal"));
        let conn = Connection::open(&db_file).unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;").unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY, name TEXT, library_name TEXT,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                outgoing_link_types TEXT NOT NULL DEFAULT '', outgoing_link_types_json TEXT NOT NULL DEFAULT '{}',
                outgoing_top_rank INTEGER NOT NULL DEFAULT 9);
             CREATE TABLE note_links (
                source_path TEXT, source_name TEXT, target_name TEXT,
                link_type TEXT, status TEXT DEFAULT 'active');
             CREATE INDEX idx_link_source ON note_links(source_path);
             CREATE INDEX idx_link_status ON note_links(status);",
        )
        .unwrap();

        // Seed note_meta + initial note_links (trigger-free: the family doesn't
        // exist yet). One transaction.
        conn.execute_batch("BEGIN").unwrap();
        for (src, edges) in &plan {
            conn.execute(
                "INSERT INTO note_meta (path, name, library_name) VALUES (?1, ?1, 'lib')",
                params![src],
            )
            .unwrap();
            for (tgt, lt) in edges {
                conn.execute(
                    "INSERT INTO note_links (source_path, source_name, target_name, link_type, status)
                     VALUES (?1, ?1, ?2, ?3, 'active')",
                    params![src, tgt, lt],
                )
                .unwrap();
            }
        }
        conn.execute_batch("COMMIT").unwrap();
        conn.execute_batch("ANALYZE").unwrap();
        eprintln!("[bench] seeded {} notes, {} links", N, total_links);

        // One full re-index-pattern rebuild: per-source DELETE + re-INSERT of every
        // edge (exactly index_note's note_links churn). Returns elapsed.
        let rebuild = |conn: &Connection| -> std::time::Duration {
            let t = Instant::now();
            conn.execute_batch("BEGIN").unwrap();
            for (src, edges) in &plan {
                conn.execute("DELETE FROM note_links WHERE source_path = ?1", params![src]).unwrap();
                for (tgt, lt) in edges {
                    conn.execute(
                        "INSERT INTO note_links (source_path, source_name, target_name, link_type, status)
                         VALUES (?1, ?1, ?2, ?3, 'active')",
                        params![src, tgt, lt],
                    )
                    .unwrap();
                }
            }
            conn.execute_batch("COMMIT").unwrap();
            t.elapsed()
        };

        let _warmup = rebuild(&conn); // warm the page cache so the delta is fair.
        let t_without = rebuild(&conn);

        // Add the production outgoing-link trigger family.
        conn.execute_batch(&format!(
            "CREATE TRIGGER note_links_outgoing_ai AFTER INSERT ON note_links \
               BEGIN UPDATE note_meta SET {ins} WHERE path = NEW.source_path; END; \
             CREATE TRIGGER note_links_outgoing_ad AFTER DELETE ON note_links \
               BEGIN UPDATE note_meta SET {del} WHERE path = OLD.source_path; END; \
             CREATE TRIGGER note_links_outgoing_au AFTER UPDATE ON note_links \
               BEGIN UPDATE note_meta SET {del} WHERE path = OLD.source_path; \
                     UPDATE note_meta SET {ins} WHERE path = NEW.source_path; END;",
            ins = outgoing_aggregate_assignments(&crate::link_types::LinkTypeRegistry::seeds_only(), "NEW.source_path"),
            del = outgoing_aggregate_assignments(&crate::link_types::LinkTypeRegistry::seeds_only(), "OLD.source_path"),
        ))
        .unwrap();

        let t_with = rebuild(&conn);

        // Sanity: the triggers actually populated the aggregates during the rebuild.
        let sample: (i64, String, i64) = read(&conn, "/lib/note_00100.md");
        eprintln!("[bench] sample /lib/note_00100.md after rebuild: {:?}", sample);
        assert!(sample.0 > 0, "triggers maintained outgoing_count during the rebuild");

        // The §A.2 fix `reconcile_filesystem` applies: drop the family for the
        // bulk walk (→ the `t_without` baseline) then ONE `recompute_all_outgoing`
        // pass. Measure that pass so we can report the fixed total vs the unfixed.
        let t_recompute = {
            let t = Instant::now();
            recompute_all_outgoing(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), &crate::converge::ConvergeKey::for_test()).unwrap();
            t.elapsed()
        };
        let fixed_total = t_without + t_recompute;

        let delta = t_with.saturating_sub(t_without);
        let pct = 100.0 * delta.as_secs_f64() / t_without.as_secs_f64().max(1e-9);
        eprintln!("[bench] full re-index rebuild — {} notes / {} links:", N, total_links);
        eprintln!("[bench]   UNFIXED triggers-on per-edge:        BEFORE {:?} → AFTER {:?}  (DELTA {:?}, +{:.1}%)", t_without, t_with, delta, pct);
        eprintln!("[bench]   FIXED   paused-for-walk + recompute: {:?}  (bulk {:?} + recompute {:?})", fixed_total, t_without, t_recompute);
        eprintln!("[bench]   single-note save (triggers stay on), amortized: {:.3} ms/note", delta.as_secs_f64() * 1000.0 / N as f64);

        drop(conn);
        let _ = std::fs::remove_file(&db_file);
        let _ = std::fs::remove_file(db_file.with_extension("db-wal"));
    }
}

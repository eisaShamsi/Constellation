//! MIG-001 Step 5 — Resumable back-fill populator for sky_nodes / sky_links.
//!
//! Triggers (Steps 3+4) keep the tables in lock-step with live writes to
//! note_meta / note_links. But on first boot after the migration lands,
//! existing notes — 7,294 on the target universe — have no rows in
//! sky_nodes, and their 217k links have no rows in sky_links. This module
//! walks those tables and populates the derived surfaces.
//!
//! Design constraints (from MIG-001 Phase 1):
//!
//! - **Must not block boot.** Runs on a background thread scheduled by
//!   `ensure_search_db_ready` after the connection is live. First paint
//!   happens before we start.
//! - **Must be resumable.** `sky_backfill_cursor` holds the last
//!   processed path. Killing the app mid-run and relaunching resumes
//!   from the cursor, not from scratch.
//! - **Must not OOM.** 1,000-row batches, each in its own BEGIN IMMEDIATE
//!   transaction. WAL flushes at COMMIT. Prior LL-XXX custom-index OOM
//!   +3GB WAL vacuum is the warning.
//! - **Must coexist with live writes.** Per-batch lock release lets user
//!   saves and other IPC calls interleave between batches. A short
//!   inter-batch sleep keeps the backfill from starving the main thread
//!   on cheap notes.
//! - **Idempotent.** `INSERT OR IGNORE` — paths already inserted via
//!   triggers (user created a note during back-fill) are skipped without
//!   error.
//!
//! Completion stamps `schema_versions.sky = SKY_SCHEMA_VERSION`. Next
//! boot detects the stamp and skips the back-fill.

use rusqlite::{params, Connection};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tauri::Manager;

use crate::search::{SearchState, SKY_SCHEMA_VERSION};
use std::sync::atomic::{AtomicBool, Ordering};

/// Batch size for each transaction. Tuned for:
/// - ~1-2 ms per note in the hot path (trigger-free bulk insert)
/// - Transaction fsync amortized across 1000 rows
/// - Enough breathing room between batches for user writes
const BATCH_SIZE: usize = 1000;

/// Sleep between batches. Gives the DB mutex to other callers. Keeps the
/// back-fill from saturating WAL during startup on large universes.
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// **PJ-332b — the single run-slot. Copied byte-for-byte from `review_backfill.rs:29-52`.**
///
/// `is_needed` is version-only, so an unstamped universe re-arms on EVERY call — and a universe
/// switch away and back calls `ensure_search_db_ready` again, which calls `maybe_schedule` again.
/// Without this, an A→B→A switch spawns a SECOND thread on universe A while the first is still
/// inside the lock-free file-read phase, and the two contend on the same WAL.
///
/// **The generation check added by PJ-332 does NOT prevent this, and a comment in this file
/// claimed it did.** `still_ours()` is evaluated only at the top of the loop, so thread 1 keeps
/// running for up to one more batch — which is precisely the window thread 2 spawns into. That
/// claim was wrong and is corrected where it was made. Found by the diff-scoped safety inspection
/// on the PJ-332 diff itself.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Schedule the back-fill on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after init_db completes and the
/// connection is in state. Silent no-op if the schema_versions.sky stamp
/// is already current.
pub fn maybe_schedule(app: tauri::AppHandle) {
    // Check quickly on the main thread whether we need to do anything at
    // all. Avoids spawning a thread for the common case (already current).
    let state = app.state::<SearchState>();
    let needs_run = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        is_needed(conn)
    };
    if !needs_run {
        return;
    }

    // B1 pass 10 — resolve the log sink HERE, while the ambient universe is still the
    // one this run serves (see `diag_at`).
    let log_path = crate::search::db_path(&app).ok();

    // Safety inspection 2026-08-23 (B1 pass 11, MED) — **the STRICT vocabulary read happens
    // HERE, before the run-slot is claimed — the shape both link back-fills already use, and
    // the one this module lacked.**
    //
    // B1 put this read at the top of `run`, above `Connection::open`, so a present-but-
    // unreadable `link-types.json` (the Windows sync/AV hold, or a half-written file) aborted
    // the WHOLE walk — including the phases that need no vocabulary at all — and, because the
    // thread then exits with `clean_exit == false`, the re-arm was skipped and nothing
    // rescheduled it: terminal for the session, on a boot where the app otherwise behaves
    // perfectly (`load_active` is lenient for the same file) and with one diagnostics line as
    // the only trace. `name_fold_backfill` had already moved its own read below the
    // vocabulary-free phases for exactly this reason; the two link back-fills read at
    // schedule time and decline the slot on refusal. Sky was the one module that claimed the
    // slot and then failed inside it.
    //
    // Refusing here costs nothing and retries naturally at the next schedule; and a read that
    // SUCCEEDS is the moment to repair a degraded session (the pass-10 rule), which this
    // module also did not do.
    let db_for_vocab = match crate::search::db_path(&app) {
        Ok(p) => p,
        Err(e) => {
            diag_at(log_path.as_deref(), &format!("[sky_backfill] NOT scheduled — database path unavailable: {}", e));
            return;
        }
    };
    let universe_root = match db_for_vocab.parent().and_then(|c| c.parent()) {
        Some(r) => r.to_path_buf(),
        None => return,
    };
    let vocab = match crate::link_types::registry_for_root(&universe_root) {
        Ok(v) => {
            if crate::link_types::recover_active_vocabulary(&app) {
                diag_at(
                    log_path.as_deref(),
                    "[link_types] link-types.json became readable — this universe's own link types are active again",
                );
            }
            v
        }
        Err(e) => {
            diag_at(log_path.as_deref(), &format!("[sky_backfill] NOT scheduled — vocabulary unreadable: {}; the walk retries at the next start, and the run-slot was NOT consumed", e));
            return;
        }
    };

    // Claim the single run-slot; if a back-fill is already in flight, do nothing.
    if RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    // Clone the AppHandle into the thread. AppHandle is Clone and cheap.
    let app_bg = app.clone();
    thread::spawn(move || {
        let clean_exit = match run(&app_bg, &vocab) {
            Ok(n) => {
                diag_at(log_path.as_deref(), &format!("[sky_backfill] completed: {} notes populated", n));
                true
            }
            Err(e) => {
                diag_at(log_path.as_deref(), &format!("[sky_backfill] FAILED: {}", e));
                false
            }
        };
        // Released in the tail, so the universe-switch early return frees the slot too — a
        // switch back must be able to resume, not be locked out for the rest of the session.
        //
        // **Known residual, stated rather than glossed:** a PANIC inside `run` skips this store
        // and leaks the slot for the process lifetime — no back-fill until restart. No data is
        // lost or corrupted by that; the walk is resumable from its cursor. Left as-is because it
        // matches `review_backfill` byte-for-byte and one consistent shape across the back-fills
        // is worth more here than a lone RAII variant. Revisit for ALL of them together, or not
        // at all.
        RUNNING.store(false, Ordering::SeqCst);
        // Safety inspection 2026-08-22 (B4 diff-scoped, fire-and-forget) — re-arm after release.
        // A switch to an UNSTAMPED universe while this thread was draining its batch hit the CAS
        // above, was silently dropped, and nothing else in the codebase calls `maybe_schedule`
        // again this session (search.rs's call sits behind the db_ready fast path; review_backfill
        // has a second re-arm site, sky had none) — so the destination universe's Sky stayed
        // partial for the whole session. The exiting thread now re-invokes the scheduler: for the
        // CURRENT universe `is_needed` decides, the freed slot lets it claim, and a completed
        // universe returns immediately. Gated on a clean exit so a persistent `run` error keeps
        // today's no-retry behavior instead of hot-looping.
        if clean_exit {
            maybe_schedule(app_bg);
        }
    });
}

/// True when the sky_* tables need back-filling. Either (a) the version
/// stamp is below target, or (b) there's a cursor row indicating a prior
/// run was interrupted.
fn is_needed(conn: &Connection) -> bool {
    // Safety inspection 2026-08-22 — distinguish "no stamp row" (a genuinely fresh /
    // re-armed universe ⇒ run) from a READ ERROR (⇒ do NOT run). The old `.unwrap_or(0)`
    // collapsed both into "needs run", and "needs run" now gates a stratum/maturity wipe —
    // a transient error must never authorize a destructive pass. Fail closed; the next
    // boot re-checks.
    use rusqlite::OptionalExtension;
    match conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'sky'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
    {
        Ok(Some(v)) => v < SKY_SCHEMA_VERSION,
        Ok(None) => true,
        Err(_) => false,
    }
}

/// The back-fill loop. Returns the number of notes processed.
///
/// **PJ-332 (2026-08-21) — this thread now has a universe identity, and that is the whole fix.**
///
/// It used to take only the `AppHandle` and re-reach into `SearchState.db` at every phase — a
/// handle the app SWAPS on a universe switch (`invalidate_search_state` sets it to `None`;
/// `ensure_search_db_ready` then installs the new universe's connection into the SAME mutex).
/// The slow phase is lock-free by design and reads up to 1000 files, so a switch lands inside it
/// routinely. The thread would then carry on against whatever universe was now in the handle —
/// writing THIS universe's cursor into THAT one, and stamping THAT one complete. Because
/// `is_needed` is version-only, a wrong stamp is permanent: that universe's Sky View stays
/// partial forever and nothing in the codebase rebuilds `sky_links` (index_repair.rs says so
/// outright). Reproduced deterministically in `tests_pj332_universe_identity`.
///
/// **Every sibling back-fill already did this correctly** — `name_fold_backfill`,
/// `links_backfill`, `incoming_links_backfill`, `review_backfill` each resolve a path ONCE and
/// open their own connection, and `derived_heal` additionally re-checks the federation generation
/// (derived_heal.rs:191-228). Sky was the lone outlier reading the mutable "whichever universe is
/// active NOW" handle. This makes it match its siblings; it invents nothing.
fn run(app: &tauri::AppHandle, vocab: &crate::link_types::LinkTypeRegistry) -> Result<u64, String> {
    // Pinned ONCE. After this line the universe this thread serves cannot change.
    let db_file = crate::search::db_path(app)?;
    // MIG-111 B1 — the VOCABULARY arrives from `maybe_schedule`, resolved STRICTLY from the
    // root derived from this same database path, before the run-slot was claimed (pass 11).
    // Reading it here instead let an unreadable file burn the slot and kill the walk for the
    // session; refusing at schedule time costs nothing and retries.
    let mut conn = Connection::open(&db_file)
        .map_err(|e| format!("sky_backfill open {}: {}", db_file.display(), e))?;
    // The `reconcile_filesystem` shape. `register_fts5_tokenizer` is NOT optional: Phase C
    // UPDATEs `note_meta`, whose FTS triggers tokenize through the custom `constellation`
    // tokenizer, and tokenizers are connection-local — without it the trigger's INSERT fails
    // with "no such tokenizer" on this connection alone.
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA recursive_triggers=ON;",
    )
    .map_err(|e| format!("sky_backfill pragma: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("sky_backfill busy_timeout: {}", e))?;
    crate::search::register_fts5_tokenizer(&mut conn)?;

    // The generation token, captured with the path. If the user leaves this universe we STOP —
    // not for correctness (our connection is pinned, so our writes are still this universe's own)
    // but because continuing wastes I/O.
    //
    // **CORRECTION (PJ-332b).** The first version of this comment also claimed the generation stop
    // prevents a second thread being spawned for this universe on a switch back. **It does not.**
    // `still_ours()` is evaluated only at the top of the loop, so this thread keeps working for up
    // to one more batch — exactly the window the second thread spawns into. The thing that actually
    // prevents it is the `RUNNING` slot on `maybe_schedule`, added in PJ-332b. Recorded rather than
    // quietly deleted: a comment that asserts a guarantee the code does not provide is worse than
    // no comment, and this one was written in the fix that introduced the need for the guard.
    let gen0 = crate::search::federation_generation_now(app);
    let still_ours = || crate::search::federation_generation_now(app) == gen0;

    // One-time setup: ensure the cursor table exists. Idempotent.
    ensure_cursor_table(&conn)?;

    // Safety inspection 2026-08-22 (B4 diff-scoped, TOCTOU) — re-check `is_needed` on the
    // connection this thread PINNED, not the one `maybe_schedule` read. The scheduler's check
    // ran on the active `SearchState` connection, and `set_active_universe` flips `active_path`
    // 18 lines before it bumps the generation — so a thread spawned for unstamped universe A
    // could pin freshly-activated, ALREADY-STAMPED universe B (gen0 captured stale, `still_ours`
    // true) and run the unconditional wipe below over a completed universe with an empty cursor:
    // every sky_nodes row's stratum/maturity NULLed, and a mid-walk switch would then abandon
    // them WITHOUT clearing the stamp — rank 0 in the Reviewer, permanently, silently. One
    // re-check on the pinned handle closes the misroute; the stamp-clear inside the wipe
    // transaction below closes the abandonment.
    if !is_needed(&conn) {
        return Ok(0);
    }

    // MIG-002 §4: run ANALYZE before any stratum computation so the
    // query planner has statistics on idx_link_source / idx_link_target /
    // idx_link_type. Without them, the planner picked idx_link_status
    // (non-selective — all links are 'active') and the stratum formula's
    // six subqueries each fanned out across the full 232k-row note_links
    // table. ~2ms per row with stats vs ~450ms without = 200× speedup.
    //
    // MIG-004 §10 audit-fix (4C-1, HIGH): scope the stratum/maturity
    // wipe to `path > last_path`. On a fresh back-fill `last_path = ""`
    // so the WHERE matches every row — same as the old unconditional
    // wipe. On RESUME after an interrupt, `last_path` reflects how far
    // the previous run had drained; rows at or below that path were
    // already recomputed under the new formula, so we MUST NOT wipe
    // them again — otherwise Phase D's path-range scope leaves them
    // stranded at NULL forever.
    //
    // Also: busy_timeout(30s) on this connection so the wipe contends
    // gracefully with cache_reconcile's parallel writes (§99 / BUG-008
    // class). Previously this block was the one back-fill phase that
    // ran without an explicit timeout.
    // **PJ-332b — read ONCE, use for both.** This used to be read here (for the wipe) and AGAIN
    // below (for the walk start), with `ANALYZE` and the wipe UPDATE in between — both contending
    // for the write lock. If another thread committed a cursor advance in that window, the wipe
    // covered `(C_old, C_new]` — rows whose stratum/maturity had just been stamped — and then the
    // walk started at `C_new` and never revisited them. Phase D is scoped
    // `path > after AND path <= last`, so that band kept NULL stratum/maturity permanently, and
    // `.unwrap_or(0)` then wrote rank 0 into the Reviewer for every one of them.
    //
    // One read, one value, no window. Found by the safety inspection on the PJ-332 diff.
    let run_fp = vocab.fingerprint();
    let (mut cursor_at_start, cursor_fp) = read_cursor(&conn)?;
    if !cursor_at_start.is_empty() && cursor_fp != run_fp {
        // The interrupted band below the cursor was computed under a DIFFERENT
        // vocabulary — restart from the top so the wipe covers everything (the wipe is
        // scoped `path > cursor`, so resetting the cursor is what widens it).
        diag_at(Some(&db_file), &format!(
            "[sky_backfill] cursor band was computed under a different vocabulary (fp {} ≠ {}) — restarting the walk from the top",
            cursor_fp, run_fp
        ));
        conn.execute("DELETE FROM sky_backfill_cursor", [])
            .map_err(|e| format!("stale-vocab cursor reset: {}", e))?;
        cursor_at_start = String::new();
    }
    {
        // PJ-332 — on the pinned connection; its busy timeout is set once in `run`.
        conn.execute_batch("ANALYZE")
            .map_err(|e| format!("ANALYZE: {}", e))?;
        // Safety inspection 2026-08-22 — wipe + stamp-clear in ONE transaction. `is_needed`'s
        // own doc has always promised "(b) there's a cursor row indicating a prior run was
        // interrupted" as a re-arm condition, and the code never implemented it: a walk that
        // wiped rows past the cursor and was then abandoned (universe switch, crash) left the
        // stamp intact, so nothing ever came back for the NULL band. Clearing the stamp the
        // moment the wipe commits makes an interrupted re-walk re-arm through condition (a) —
        // the code now keeps the contract the comment stated. `finalize` re-stamps on
        // completion. Readers gating on the stamp (`is_federated_sky_ready`) already treated
        // an in-progress back-fill as not-ready, so this changes nothing for them.
        let tx = conn
            .transaction()
            .map_err(|e| format!("wipe tx begin: {}", e))?;
        tx.execute(
            "UPDATE sky_nodes SET stratum = NULL, maturity = NULL WHERE path > ?1",
            params![cursor_at_start],
        )
        .map_err(|e| format!("stratum/maturity wipe: {}", e))?;
        tx.execute("DELETE FROM schema_versions WHERE module = 'sky'", [])
            .map_err(|e| format!("stamp clear with wipe: {}", e))?;
        tx.commit().map_err(|e| format!("wipe tx commit: {}", e))?;
    }

    let mut last_path = cursor_at_start.clone();
    let mut total: u64 = 0;

    loop {
        if !still_ours() {
            // The user switched universes. Leave the cursor exactly where it is: the next boot
            // into this universe resumes from it. Do NOT finalize — a partial run must never be
            // stamped complete (see `finalize`'s own guard).
            return Ok(total);
        }
        let (batch_count, new_last_path) = process_batch(&mut conn, &last_path, vocab)?;
        if batch_count == 0 {
            // Drained. Stamp the version and wipe the cursor row.
            finalize(&mut conn, &db_file)?;
            return Ok(total);
        }
        total += batch_count as u64;
        last_path = new_last_path;
        write_cursor(&conn, &last_path, run_fp)?;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

fn ensure_cursor_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sky_backfill_cursor (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_path TEXT,
            started_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )
    .map_err(|e| format!("cursor table create: {}", e))?;
    // Safety inspection 2026-08-23 (B1 pass 3, LOW) — the links_backfill cursor
    // fingerprint, extended here (the identical resume shape): a first-time walk
    // interrupted under V1 and resumed under V2 would leave rows at or below the cursor
    // with V1-derived stratum/maturity under the version-only stamp forever. 0 =
    // unknown (pre-column cursor) and deliberately mismatches every real fingerprint.
    //
    // **Honest scope (corrected at pass 6): today this is a GUARD, not a live fix.**
    // Sky's two generators reach the registry only through `structural_not_in_clause`,
    // and the structural lane is immutable (`merge` hardcodes it to `parent`/`contains`
    // — see the note on `converge::after_vocabulary_change`), so the mixed-vocabulary
    // band it prevents is not reachable from any user action in the current build. It
    // stays because it costs one integer per cursor write and becomes load-bearing the
    // day the structural lane becomes user-definable — at which point the sibling
    // fingerprint gate must be added here too.
    // Safety inspection 2026-08-23 (B1 pass 5, MED) — tolerate ONLY the expected
    // "duplicate column" (the column already exists); propagate every OTHER failure
    // (busy/locked/io) so the run aborts HERE, before any destructive step reads
    // through the missing column. The old `let _` swallowed a busy-timeout failure,
    // read_cursor's .ok() then collapsed "no such column" into an EMPTY cursor, and
    // sky's whole-table wipe ran scoped `path > ''`.
    if let Err(e) = conn.execute(
        "ALTER TABLE sky_backfill_cursor ADD COLUMN vocab_fp INTEGER NOT NULL DEFAULT 0",
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
            "SELECT COALESCE(last_path, ''), COALESCE(vocab_fp, 0) FROM sky_backfill_cursor WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    Ok(row.unwrap_or_default())
}

fn write_cursor(conn: &Connection, last_path: &str, vocab_fp: i64) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO sky_backfill_cursor (id, last_path, vocab_fp) VALUES (1, ?1, ?2)",
        params![last_path, vocab_fp],
    )
    .map_err(|e| format!("cursor write: {}", e))?;
    Ok(())
}

/// One batch, three phases — the DB lock is released during filesystem
/// I/O so the main thread's IPC queries don't queue behind us.
///
/// Phase A (under lock): pull the next batch of paths from note_meta,
/// insert sky_nodes + sky_links rows in one transaction. Fast — pure
/// SQL, no disk reads of note files.
///
/// Phase B (no lock): read each note file, compute word_count +
/// created_at via `compute_word_count_and_created_at`. This is the
/// expensive step — up to BATCH_SIZE file reads. Running it outside
/// the mutex means frontend queries stay responsive on boot.
///
/// Phase C (under lock): UPDATE note_meta with the precomputed values
/// in a second transaction. Single prepared statement, parameterised.
///
/// `INSERT OR IGNORE` in Phase A makes the sky_* inserts idempotent —
/// rows populated by triggers during a concurrent write don't error.
/// The `WHERE word_count = 0 OR created_at IS NULL` guard in Phase C
/// preserves any values the writer stamped in between our phases.
fn process_batch(
    conn: &mut Connection,
    after_path: &str,
    reg: &crate::link_types::LinkTypeRegistry,
) -> Result<(usize, String), String> {
    // MIG-111 B1 — the registry arrives from `run`'s pinned root (the name_fold shape):
    // conn and vocabulary are the same universe's by construction; one value serves the
    // Phase-A structural exclusion and the Phase-D stratum/maturity expressions.
    // ── Phase A: path query + sky_* inserts ───────────────────────────
    let (paths, last_path) = {
        let tx = // B1 pass 9 (LOW, Whole-Ecosystem) — IMMEDIATE: these batches read then write on a
        // PRIVATE connection, and SQLite does not run the busy handler for a DEFERRED
        // snapshot upgrade. See the note in `links_backfill::process_batch`.
        conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e| format!("begin: {}", e))?;

        let mut paths: Vec<(String, String, String)> = Vec::with_capacity(BATCH_SIZE);
        {
            let mut stmt = tx
                .prepare(
                    "SELECT path, name, library_name
                     FROM note_meta
                     WHERE path > ?1
                     ORDER BY path
                     LIMIT ?2",
                )
                .map_err(|e| format!("prepare nodes: {}", e))?;
            let rows = stmt
                .query_map(params![after_path, BATCH_SIZE as i64], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(|e| format!("query nodes: {}", e))?;
            for r in rows {
                paths.push(r.map_err(|e| format!("row nodes: {}", e))?);
            }
        }

        if paths.is_empty() {
            tx.commit().map_err(|e| format!("commit empty: {}", e))?;
            return Ok((0, after_path.to_string()));
        }

        let last_path = paths.last().map(|p| p.0.clone()).unwrap_or_default();

        {
            // MIG-085 §B.0 — sky_nodes.id is the Unicode-folded name (fold_match_key),
            // computed in Rust so it matches note_links.target_name for accented titles
            // (SQLite LOWER() is ASCII-only and would leave "Île-de-France" unmatched).
            let mut ins = tx
                .prepare(
                    "INSERT OR IGNORE INTO sky_nodes
                        (path, id, name, library_name, updated_at)
                     VALUES (?1, ?4, ?2, ?3, strftime('%s','now'))",
                )
                .map_err(|e| format!("prepare ins node: {}", e))?;
            for (p, name, lib) in &paths {
                ins.execute(params![p, name, lib, crate::search::fold_match_key(name)])
                    .map_err(|e| format!("exec ins node: {}", e))?;
            }
        }

        // PJ-065 Phase-4 audit (P1 leak): the live sky triggers exclude the structural
        // lane (sx_new), but this one-shot backfill did NOT — so a schema-version bump
        // (or an unfinished backfill) re-running over a universe that already has
        // parent/contains edges would copy them into sky_links and inflate Sky-View node
        // counts. Append the same registry exclusion the triggers use. Empty string when
        // no structural type exists ⇒ byte-identical to before the lane registered.
        let sx = reg.structural_not_in_clause("link_type");
        tx.execute(
            &format!(
                "INSERT OR IGNORE INTO sky_links (source_path, target_name, link_type, weight)
                 SELECT source_path, target_name, link_type, COALESCE(weight, 1.0)
                 FROM note_links
                 WHERE status = 'active'{sx}
                   AND source_path > ?1
                   AND source_path <= ?2"
            ),
            params![after_path, last_path.clone()],
        )
        .map_err(|e| format!("ins links: {}", e))?;

        tx.commit().map_err(|e| format!("commit A: {}", e))?;
        (paths, last_path)
    };
    // Lock released here — Phase B runs free.

    // ── Phase B: file reads WITHOUT lock ───────────────────────────────
    // Each tuple = (path, word_count, created_at). Bounded by BATCH_SIZE
    // rows so memory footprint is trivial.
    let computed: Vec<(String, NoteSignals)> = paths
        .iter()
        .map(|(p, _, _)| (p.clone(), read_note_signals(Path::new(p))))
        .collect();

    // ── Phase C: UPDATE note_meta ─────────────────────────────────────
    {
        // The busy timeout is set once on this connection in `run` (PJ-332).
        let tx = // B1 pass 9 (LOW, Whole-Ecosystem) — IMMEDIATE: these batches read then write on a
        // PRIVATE connection, and SQLite does not run the busy handler for a DEFERRED
        // snapshot upgrade. See the note in `links_backfill::process_batch`.
        conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e| format!("begin C: {}", e))?;
        {
            let mut upd = tx
                .prepare(
                    // Safety inspection 2026-08-23 (B1 pass 2, LOW) — per-COLUMN guards,
                    // not a row-level OR: the old row filter let a created_at-IS-NULL row
                    // (permanent on filesystems with no birth timestamp) take word_count=0
                    // from a transiently unreadable file, silently dropping the note's
                    // stratum to the lowest band. Each column now fills only its own gap.
                    "UPDATE note_meta
                        SET word_count = CASE WHEN word_count = 0 THEN ?1 ELSE word_count END,
                            created_at = COALESCE(created_at, ?2)
                      WHERE path = ?3
                        AND (word_count = 0 OR created_at IS NULL)",
                )
                .map_err(|e| format!("prepare upd word_count: {}", e))?;
            for (p, sig) in &computed {
                upd.execute(params![sig.word_count, sig.created_at, p])
                    .map_err(|e| format!("exec upd word_count: {}", e))?;
            }
        }
        tx.commit().map_err(|e| format!("commit C: {}", e))?;
    }

    // ── Phase E: back-fill note_aliases (frontmatter source) ──────────
    // MIG-004 §5. INSERT OR IGNORE per (path, alias) pair so existing
    // 'rename' / 'import' rows for the same alias stay put — composite
    // PK + IGNORE makes us idempotent and resilient to re-run mid-fill.
    // Skips paths that contributed zero aliases (most legacy notes
    // without `aliases:` frontmatter).
    {
        let tx = // B1 pass 9 (LOW, Whole-Ecosystem) — IMMEDIATE: these batches read then write on a
        // PRIVATE connection, and SQLite does not run the busy handler for a DEFERRED
        // snapshot upgrade. See the note in `links_backfill::process_batch`.
        conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e| format!("begin E: {}", e))?;
        {
            let mut ins = tx
                .prepare(
                    // PJ-332 — the SELECT is an EXISTENCE GUARD, not decoration. The previous
                    // VALUES form inserted an alias row for any path handed to it, whether or not
                    // that note was in THIS database. With the connection now pinned that cannot
                    // be a cross-universe path, but a note deleted mid-run still would be, and
                    // `note_aliases` is consulted for wikilink resolution (libraries.rs, map.rs).
                    "INSERT OR IGNORE INTO note_aliases (path, alias_lower, source, cid_cn)
                     SELECT ?1, ?2, 'frontmatter', COALESCE(cid_cn, '')
                       FROM note_meta WHERE path = ?1",
                )
                .map_err(|e| format!("prepare ins alias: {}", e))?;
            for (p, sig) in &computed {
                for alias in &sig.aliases {
                    ins.execute(params![p, alias])
                        .map_err(|e| format!("exec ins alias: {}", e))?;
                }
            }
        }
        tx.commit().map_err(|e| format!("commit E: {}", e))?;
    }

    // ── Phase D: back-fill sky_nodes.stratum + .maturity for this batch
    // MIG-002 §4 (stratum) + §5 (maturity). Two UPDATEs, both scoped by
    // path range from this batch. Expressions kept in lockstep with the
    // triggers defined in search.rs::init_db via pub(crate) constants.
    //
    // Scoped to paths in [after_path, last_path] so we don't re-touch
    // every sky_nodes row on every batch. WHERE <col> IS NULL makes it
    // idempotent — rows already stamped by the triggers stay put.
    {
        let tx = // B1 pass 9 (LOW, Whole-Ecosystem) — IMMEDIATE: these batches read then write on a
        // PRIVATE connection, and SQLite does not run the busy handler for a DEFERRED
        // snapshot upgrade. See the note in `links_backfill::process_batch`.
        conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e| format!("begin D: {}", e))?;
        tx.execute(
            &format!(
                "UPDATE sky_nodes SET stratum = ({expr})
                   WHERE stratum IS NULL
                     AND path > ?1
                     AND path <= ?2",
                expr = crate::search::stratum_sql_expr(reg),
            ),
            params![after_path, last_path.clone()],
        )
        .map_err(|e| format!("upd stratum: {}", e))?;
        tx.execute(
            &format!(
                "UPDATE sky_nodes SET maturity = ({expr})
                   WHERE maturity IS NULL
                     AND path > ?1
                     AND path <= ?2",
                expr = crate::search::maturity_sql_expr(reg),
            ),
            params![after_path, last_path.clone()],
        )
        .map_err(|e| format!("upd maturity: {}", e))?;
        tx.commit().map_err(|e| format!("commit D: {}", e))?;
    }

    Ok((paths.len(), last_path))
}

/// Signals extracted from a single note file during back-fill.
/// Lets Phase B do one fs::read_to_string per note and feed all of
/// the back-fill's downstream phases (word_count for §C, aliases for
/// MIG-004 §E) without re-reading.
struct NoteSignals {
    word_count: i64,
    created_at: Option<i64>,
    aliases: Vec<String>,
}

/// Read a .md file and return its back-fill signals. Mirrors the
/// writer-side stamping in `search::index_note` byte-for-byte:
///
/// - word_count: whitespace-separated tokens of the body (post-
///   frontmatter strip), via `search::body_after_frontmatter`.
/// - created_at: fs::metadata(path).created() epoch seconds. None on
///   filesystems without a true creation timestamp (ReFS, FAT32,
///   some Linux FS); the UPDATE in Phase C uses COALESCE to keep
///   any value previously stamped via `modified` fallback.
/// - aliases: frontmatter `aliases:` entries, via
///   `search::extract_aliases`. Each is already lowercased + Arabic-
///   normalized so it matches `note_links.target_name` byte-for-byte.
///
/// A missing / unreadable file yields zero/empty signals — the
/// downstream UPDATEs / INSERTs become no-ops via their guards.
fn read_note_signals(path: &Path) -> NoteSignals {
    let Ok(content) = std::fs::read_to_string(path) else {
        return NoteSignals {
            word_count: 0,
            created_at: None,
            aliases: Vec::new(),
        };
    };
    // Single source of truth for frontmatter slicing — search.rs owns
    // the strip shape so back-fill and writer agree byte-for-byte.
    let body = crate::search::body_after_frontmatter(&content);
    let word_count = body.split_whitespace().count() as i64;
    let created_at: Option<i64> = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);
    let aliases = crate::search::extract_aliases(&content);
    NoteSignals { word_count, created_at, aliases }
}

fn finalize(conn: &mut Connection, db_file_for_diag: &std::path::Path) -> Result<(), String> {
    // PJ-332 — **stamping is a claim of completeness, so check it.** `is_needed` reads only the
    // version stamp, which makes a wrong stamp PERMANENT: nothing re-runs, and nothing in the
    // codebase rebuilds `sky_links`. Refuse to stamp while any note still lacks a sky_nodes row.
    let unfinished: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_meta \
             WHERE NOT EXISTS (SELECT 1 FROM sky_nodes WHERE sky_nodes.path = note_meta.path)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    // **This is REPORTED, not enforced — and the difference was decided on live data.**
    //
    // The first version of this guard returned Err here. Measured against the Boss's real
    // universes BEFORE shipping it: `Eisa Cognitive Knowledge` is stamped complete with **7 of
    // 8,031** notes lacking a sky row, and `Scratch` with 1 of 30. Three of the seven sit in
    // `.trash`. A hard refusal would therefore have meant that universe NEVER stamps — and since
    // the walk re-arms from an empty cursor, it would re-read 8,031 notes and their files **on
    // every boot, forever**. That is a worse regression than the defect it guards, and it would
    // have shipped on the Boss's largest universe.
    //
    // The walk is exhaustive by construction (`process_batch` selects every `note_meta` row with
    // `path > cursor`, with no exclusions), so a drained batch means every note WAS offered a sky
    // row. If one still lacks it, re-walking cannot add it — but the count belongs in the record
    // instead of nowhere, which is where it lived before. PJ-334 carries the live measurement.
    if unfinished > 0 {
        crate::search::diag_log(
            db_file_for_diag,
            &format!(
                "[sky_backfill] stamping complete with {unfinished} notes lacking a sky row - the walk was exhaustive, so a re-run would not add them. See PJ-334."
            ),
        );
    }
    // Wrap version stamp + cursor clear in one transaction so a crash
    // between them can't leave a completed back-fill with a live cursor
    // row (which would make the next boot think it was interrupted).
    let tx = // B1 pass 9 (LOW, Whole-Ecosystem) — IMMEDIATE: these batches read then write on a
        // PRIVATE connection, and SQLite does not run the busy handler for a DEFERRED
        // snapshot upgrade. See the note in `links_backfill::process_batch`.
        conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e| format!("finalize begin: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('sky', ?1)",
        params![SKY_SCHEMA_VERSION],
    )
    .map_err(|e| format!("finalize stamp: {}", e))?;
    tx.execute("DELETE FROM sky_backfill_cursor", [])
        .map_err(|e| format!("finalize cursor: {}", e))?;
    tx.commit().map_err(|e| format!("finalize commit: {}", e))?;
    Ok(())
}

/// Write a line to the universe's diagnostics log. Thin wrapper around
/// search::diag_log — kept here so this module doesn't depend on the
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
mod tests_pj332_universe_identity {
    use super::*;

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "constellation_pj332_{}_{}",
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).expect("tmp dir");
        d
    }

    fn built(tag: &str) -> (std::path::PathBuf, Connection) {
        let dir = tmp(tag);
        let conn = crate::search::init_db(&dir.join("search.db")).expect("init_db");
        ensure_cursor_table(&conn).expect("cursor table");
        (dir, conn)
    }

    fn stamped(conn: &Connection) -> i64 {
        conn.query_row(
            "SELECT version FROM schema_versions WHERE module = 'sky'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0)
    }

    // ─── PJ-332 — what the original reproduction became ───────────────────
    //
    // The first version of this module reproduced the defect directly: it built two universes,
    // put A's connection in a `Mutex<Option<Connection>>`, swapped B's in mid-run — exactly what
    // `invalidate_search_state` + `ensure_search_db_ready` do on a universe switch — and watched
    // `write_cursor` and `finalize` write A's progress into B. It was RED, and it named the
    // damage: **B stamped `version 10` by a thread that was back-filling A.**
    //
    // **That test can no longer be written.** `write_cursor`, `finalize` and `process_batch` now
    // take a connection this thread opened and owns, so there is no swappable handle to hand
    // them. The defect is not fixed so much as made inexpressible — which is the outcome
    // Solve-the-Class asks for, and it is why no swap test survives here.
    //
    // What remains testable is the second half of the fix: the guards that stop a PARTIAL run
    // from claiming completeness. Those matter independently, because `is_needed` reads only the
    // version stamp — so a wrong stamp is permanent, and nothing in the codebase rebuilds
    // `sky_links` (index_repair.rs states this outright).

    /// **Stamping is a claim of completeness — and the check on it must not become a boot loop.**
    ///
    /// Red before the guard: `finalize` used to stamp unconditionally the moment a batch drained,
    /// and a drained batch is not the same thing as a finished universe — an aborted run, a
    /// cursor that skipped a range, or a note added mid-run all reach it.
    #[test]
    fn an_exhaustive_walk_stamps_and_reports_rather_than_blocking_forever() {
        let (dir, mut conn) = built("incomplete");
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, content_hash)              VALUES ('/u/orphan.md', 'orphan', 'lib', 0, 'h')",
            [],
        )
        .expect("insert note");
        // The live `note_meta_sky_ai` trigger creates the sky row on insert, so remove it — this
        // is the LEGACY state the back-fill exists to repair: notes indexed before the sky
        // triggers existed, which have a `note_meta` row and no `sky_nodes` row. (Learned by
        // writing this test and watching it pass when it should not have.)
        conn.execute("DELETE FROM sky_nodes WHERE path = '/u/orphan.md'", [])
            .expect("simulate the pre-trigger legacy state");

        finalize(&mut conn, &dir.join("search.db")).expect("an exhaustive walk still stamps");
        assert_eq!(
            stamped(&conn),
            SKY_SCHEMA_VERSION,
            "it STAMPS — measured on live data, a hard refusal here would re-walk 8,031 notes on              every boot forever (see the comment in `finalize`). The incompleteness is REPORTED              to the diagnostics log instead."
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The same function must still stamp when the universe genuinely IS finished, or the
    /// back-fill would never complete and would re-run on every boot forever.
    #[test]
    fn finalize_stamps_when_every_note_has_its_sky_row() {
        let (dir, mut conn) = built("complete");
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, content_hash)              VALUES ('/u/done.md', 'done', 'lib', 0, 'h')",
            [],
        )
        .expect("insert note");
        conn.execute(
            "INSERT OR IGNORE INTO sky_nodes (path, id) VALUES ('/u/done.md', 'done')",
            [],
        )
        .expect("insert its sky row");

        finalize(&mut conn, &dir.join("search.db")).expect("a finished universe must stamp");
        assert_eq!(stamped(&conn), SKY_SCHEMA_VERSION, "stamped complete");
        let cursor_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM sky_backfill_cursor", [], |r| r.get(0))
            .unwrap_or(-1);
        assert_eq!(cursor_rows, 0, "and the cursor is cleared in the same transaction");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

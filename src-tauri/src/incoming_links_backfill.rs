//! MIG-079 §C.2a — one-shot backfill for the INCOMING-link aggregate columns
//! (`note_meta.incoming_count` / `incoming_link_types` / `incoming_top_rank` /
//! `incoming_link_types_json`). On the first boot after the migration lands,
//! existing notes' incoming columns sit at the schema defaults; this recomputes
//! them once from `note_links`, then stamps `schema_versions.incoming_links`.
//!
//! Design — mirrors `links_backfill::recompute_all_outgoing` (the proven model),
//! NOT the `tag_counts` atomic pattern:
//! - **Convergent, not additive.** The recompute reads the CURRENT `note_links`
//!   state (the `incoming_aggregate_assignments` SQL), so it coexists with the
//!   live `note_links_incoming_*` / `note_aliases_incoming_*` triggers with no
//!   race — a note recomputed by the backfill and then edited is fixed by the
//!   trigger; a note edited then recomputed converges to the same value. Both
//!   derive from `note_links`. No atomic single-transaction needed.
//! - **Never blocks boot.** Background thread, own connection (the
//!   `reconcile_filesystem` walk_conn pattern); each 500-row batch is its own
//!   short autocommit UPDATE (busy-retry), so live saves interleave between batches.
//! - **Idempotent / restart-safe.** Not cursor-resumable, but cheap to restart
//!   (recompute is deterministic from `note_links`); an interrupted run leaves the
//!   stamp below target → reads fall back to the live `getBacklinks` path → next
//!   boot re-runs. The `idx_note_links_target_name_lower` expression index (created
//!   in `init_db`) keeps each note's recompute index-served (hub `isbn` = 5,358).

use rusqlite::{params, Connection};
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Bump to force a one-time recompute on existing DBs (e.g. if the match
/// semantics change). Parallel to `LINKS_OUTGOING_SCHEMA_VERSION`.
pub(crate) const SCHEMA_VERSION: i64 = 1;

/// True once the incoming aggregate has been built + stamped **under the current
/// link-type vocabulary**. The read-flip in `constellation_search_link_counts`,
/// the save-path maintenance gate, and the reconcile recompute gate all sit on this.
///
/// Safety-inspection 2026-08-01 — vocabulary-fingerprint gate, mirroring
/// `links_backfill::is_needed` (MIG-067 §B): `incoming_link_types` / `_json` /
/// `_top_rank` are derived from the active link-type vocabulary (the registry's
/// IN-list + rank CASE), so a vocabulary edit leaves the stored rows stale. A
/// fingerprint mismatch reads as NOT stamped, which (a) re-schedules this backfill
/// (`maybe_schedule`, also called from `on_link_vocabulary_changed`) and (b) flips
/// every gated reader back to the live `getBacklinks` path until the
/// re-materialize completes — stale aggregates are never served. A universe last
/// backfilled before this gate existed has no vocab stamp (0 ≠ fingerprint) and
/// re-materializes once — the same one-time upgrade path as outgoing §A→§B.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    // Whose vocabulary? The ACTIVE universe's IN-MEMORY one — correct because this is
    // the READER gate (the search.rs read-flip): it asks "may the stored columns be
    // served under the vocabulary the UI is rendering right now?" A mismatch (including
    // a stale lenient-boot global) sends readers to the live fallback — slow but always
    // correct. The SCHEDULER no longer routes through this fn (B1 pass 2): its gate in
    // `maybe_schedule` compares against the DISK fingerprint the run pins and stamps,
    // so a stale global cannot loop the re-arm.
    is_built(conn) && stored_vocab_fingerprint(conn) == crate::link_types::active_universe_vocabulary().fingerprint()
}

/// **Structure exists** — the aggregate has been built at least once, whatever vocabulary it
/// was built under. This is the correct gate for the WRITE side, and the distinction matters:
///
/// Safety inspection 2026-08-01 — the fingerprint gate above (added earlier the same day)
/// reads as NOT stamped for the whole duration of a vocabulary re-materialize. Gating the
/// SAVE-PATH maintenance on it therefore switched maintenance off across that window, while
/// the re-materialize's ascending cursor cannot revisit a target it has already passed. A
/// note saved mid-run whose target sat BEHIND the cursor was left permanently stale — and
/// then served, because the stamp lands at the end and flips every reader onto the stored
/// columns. Two different questions were being asked of one predicate:
///
///   · READERS ask "are these columns TRUSTWORTHY right now?" → `is_stamped`
///     (version AND fingerprint). Unchanged: stale aggregates are never served; readers fall
///     back to the live `getBacklinks` path until the re-materialize completes.
///   · WRITERS ask "do these columns EXIST to be maintained?" → `is_built` (version only).
///     Maintaining during the re-materialize is free of risk: the values are recomputed from
///     `note_links`, which is the source of truth, so a maintenance write and the backfill's
///     own pass agree — and nothing behind the cursor is stranded.
///
/// Before the FIRST backfill the columns are genuinely inert, so both predicates are false
/// and the save path correctly skips the work.
pub(crate) fn is_built(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'incoming_links'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        >= SCHEMA_VERSION
}

/// The vocabulary fingerprint stamped at the last completed incoming backfill
/// (0 if never). Parallel to `links_backfill::stored_vocab_fingerprint`
/// (`links_vocab`); this module stamps `incoming_links_vocab`.
fn stored_vocab_fingerprint(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'incoming_links_vocab'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
}

/// Schedule the one-shot backfill on a background thread. Silent no-op once
/// stamped. Mirrors `note_body_backfill::maybe_schedule`.
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
/// Safety inspection 2026-08-23 (B1 pass 12, MED) — **the write-side gate must also be open
/// WHILE the first build is running.**
///
/// `is_built` only becomes true when the version stamp lands, at the very END of the run. For
/// the whole first build the save path therefore skipped incoming maintenance entirely —
/// and the build walks `note_meta` in ASCENDING path order and never revisits. So a link
/// created during the walk, whose TARGET sorts below the cursor, is maintained by nobody: not
/// by the save (gate closed) and not by the walk (already past). The stamp then flips every
/// reader onto those columns, and the miss is permanent — a badge one short, a type missing
/// from a breakdown, with nothing to heal it.
///
/// The module's own doc reasoned this through for the vocabulary re-materialize case and
/// declared the FIRST build safe because "the columns are genuinely inert" — true for
/// READERS, and only until the stamp at the end of that same run. The write side has no such
/// excuse: maintaining a column nobody is reading yet is harmless, and the walk's later pass
/// over the same target recomputes the identical value (the maintenance and the bulk pass
/// share `incoming_aggregate_assignments`). Keeping it open costs an idempotent recompute;
/// keeping it closed costs a silent permanent hole.
pub(crate) fn maintenance_is_live(conn: &Connection) -> bool {
    is_built(conn) || is_running()
}

pub(crate) fn is_running() -> bool {
    RUNNING.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn maybe_schedule(app: tauri::AppHandle) {
    // B1 pass 10 — resolve the log sink HERE, while the ambient universe is still the
    // one this run serves (see `diag_at`).
    let log_path = crate::search::db_path(&app).ok();
    // B1 pass 2 (MED) — the scheduler's gate compares against the DISK fingerprint,
    // the same source the run pins and stamps (see links_backfill::maybe_schedule for
    // the loop this closes). `is_stamped` stays the READER gate on the in-memory
    // vocabulary. STRICT read; a refusal skips scheduling (retried next call).
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
                diag_at(log_path.as_deref(), &format!("[incoming_links_backfill] NOT scheduled — vocabulary unreadable: {}", e));
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
        // B1 pass 8 — the `trigger_vocab` term was REMOVED from this gate. It belongs to
        // the OUTGOING triggers, which this module neither owns nor rebuilds, so it was a
        // condition this module's run could never clear: with the clean-exit re-arm, one
        // disagreement became a permanent full-universe recompute loop. The outgoing
        // back-fill both detects and REPAIRS that disagreement (see its run); the incoming
        // aggregates do not depend on those triggers at all.
        !(is_built(conn) && stored_vocab_fingerprint(conn) == current_fp)
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
    std::thread::spawn(move || {
        let clean_exit = match run(&app_bg) {
            Ok(n) => {
                diag_at(log_path.as_deref(), &format!("[incoming_links_backfill] completed: {} notes recomputed", n));
                true
            }
            Err(e) => {
                diag_at(log_path.as_deref(), &format!("[incoming_links_backfill] FAILED (non-fatal): {}", e));
                false
            }
        };
        RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
        // Re-arm after release (the sky_backfill shape): a save dropped by the CAS above
        // re-enters through the `is_stamped` gate — one follow-up pass, no hot loop.
        if clean_exit {
            maybe_schedule(app_bg);
        }
    });
}

/// Recompute every note's incoming aggregate on a DEDICATED connection (batched,
/// busy-tolerant) then stamp. Convergent with the live triggers (both read
/// note_links), so no single-transaction atomicity is required.
fn run(app: &tauri::AppHandle) -> Result<usize, String> {
    // MIG-111 B1 — path, root and VOCABULARY pinned together, from the same universe at
    // the same moment (the name_fold shape): the fingerprint stamped below and the
    // aggregates computed below then describe the same registry as the connection's
    // universe, whatever becomes active later.
    let path = crate::search::db_path(app)?;
    // ONE active read: the universe root is DERIVED from the pinned db path
    // (`<root>/.constellation/search.db` — `active_constellation_dir`'s own layout),
    // never read from the ambient active universe a second time. Two sequential
    // ambient reads leave a window where a switch pairs universe A's database with
    // universe B's vocabulary — the exact class B1 removes.
    let universe_root = path
        .parent()
        .and_then(|c| c.parent())
        .ok_or_else(|| String::from("search.db path has no universe root"))?
        .to_path_buf();
    let vocab = crate::link_types::registry_for_root(&universe_root)?;
    let mut conn = Connection::open(&path).map_err(|e| format!("open incoming conn: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;
    // Register the 'constellation' FTS5 tokenizer on this dedicated connection
    // (like reconcile_filesystem's walk_conn). The incoming-only UPDATE shouldn't
    // fire the guarded note_meta_au FTS trigger, but registering is defensive:
    // a legacy universe whose note_meta_au lost its WHEN guard would otherwise
    // fail every recompute with "no such tokenizer".
    crate::search::register_fts5_tokenizer(&mut conn)
        .map_err(|e| format!("register tokenizer: {}", e))?;

    // Build the index over the VIRTUAL target_name_lower column so the recompute —
    // and the live note_links_incoming_* triggers — SEEK it (plain-column equality/
    // JOIN). CREATE INDEX is ~50 s on 234k edges and fires NO triggers, but it is
    // kept here (background) rather than init_db so it never blocks boot. IF NOT
    // EXISTS → no-op once built.
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_nl_tnl ON note_links(target_name_lower, status);")
        .map_err(|e| format!("create idx_nl_tnl: {}", e))?;

    // Safety-inspection 2026-08-01 — capture the vocabulary fingerprint for THIS
    // run up-front (the links_backfill MIG-067 §B pattern): if the vocabulary
    // changes mid-run, the stamp below differs from the then-current fingerprint,
    // is_stamped stays false, and the next schedule re-runs — eventual consistency.
    let run_fp = vocab.fingerprint();

    // PJ-207 §6 — through the one assembly, via the entry point named for this caller:
    // it is the BUILD of the incoming aggregates, not a heal, so it is deliberately
    // ungated on the stamp it is itself about to write.
    let rep = crate::converge::after_incoming_backfill(&conn, &vocab);
    let n = match &rep.incoming {
        crate::converge::ConvergeOutcome::Converged(n) => *n,
        crate::converge::ConvergeOutcome::Failed(e) => return Err(format!("recompute: {}", e)),
        crate::converge::ConvergeOutcome::Skipped(r) => {
            return Err(format!("recompute skipped unexpectedly ({}) — the back-fill is the builder and must never be gated", r))
        }
    };

    // Stamp version + vocabulary fingerprint atomically (mirrors
    // links_backfill::finalize) so a crash between the two can't leave a stamped
    // backfill carrying a missing/foreign vocab stamp.
    let tx = conn.transaction().map_err(|e| format!("stamp begin: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('incoming_links', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('incoming_links_vocab', ?1, strftime('%s','now'))",
        params![run_fp],
    )
    .map_err(|e| format!("vocab stamp: {}", e))?;
    tx.commit().map_err(|e| format!("stamp commit: {}", e))?;
    Ok(n as usize)
}

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
    //! MIG-079 §C.2a rehearsal (the P0 gate) — run the REAL `recompute_all_incoming`
    //! (the exact `incoming_aggregate_assignments` SQL the triggers use) against a
    //! COPY of the live universe DB and assert `note_meta.incoming_count` equals the
    //! `getBacklinks` count (alias-aware, distinct-source) for EVERY note. Proves the
    //! badge will equal the panel before the save path is touched.
    //!
    //! Run:
    //!   INCOMING_REHEARSAL_DB="E:\Backups\Constellation\rehearsal\incoming-rehearsal.db" \
    //!   INCOMING_TARGET_JSON="..\lab\tag-counts\incoming-target.json" \
    //!   cargo test --lib incoming_links_backfill::tests::rehearse -- --ignored --nocapture
    use super::*;
    use std::collections::HashMap;

    /// §C.2a — pins the incoming-count semantics that must match `getBacklinks`:
    /// dedupe-by-source, alias resolution, case-insensitive name match, archived
    /// exclusion. In-memory; uses the real `recompute_all_incoming` SQL.
    #[test]
    fn incoming_count_dedupes_resolves_aliases_and_excludes_archived() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, name_lower TEXT,
                incoming_count INTEGER NOT NULL DEFAULT 0,
                incoming_link_types TEXT NOT NULL DEFAULT '',
                incoming_link_types_json TEXT NOT NULL DEFAULT '{}',
                incoming_top_rank INTEGER NOT NULL DEFAULT 9);
             CREATE TABLE note_aliases (path TEXT, alias_lower TEXT);
             CREATE TABLE note_links (source_path TEXT, target_name TEXT, link_type TEXT, status TEXT,
                target_name_lower TEXT GENERATED ALWAYS AS (LOWER(target_name)) VIRTUAL);
             CREATE INDEX idx_nl_tnl ON note_links(target_name_lower, status);",
        )
        .unwrap();
        conn.execute_batch(
            "INSERT INTO note_meta(path,name) VALUES ('/A.md','Alpha'),('/B.md','Beta');
             INSERT INTO note_aliases(path,alias_lower) VALUES ('/A.md','al');
             INSERT INTO note_links(source_path,target_name,link_type,status) VALUES
               ('/S1.md','Alpha','supports','active'),
               ('/S1.md','al','','active'),
               ('/S2.md','alpha','supports','active'),
               ('/S3.md','Beta','causes','active'),
               ('/S4.md','Alpha','supports','archived');",
        )
        .unwrap();
        crate::links_backfill::recompute_all_incoming(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), &crate::converge::ConvergeKey::for_test()).unwrap();
        let a: i64 = conn
            .query_row("SELECT incoming_count FROM note_meta WHERE path='/A.md'", [], |r| r.get(0))
            .unwrap();
        let b: i64 = conn
            .query_row("SELECT incoming_count FROM note_meta WHERE path='/B.md'", [], |r| r.get(0))
            .unwrap();
        // A: S1 (via name AND alias — deduped to ONE source) + S2 (case-insensitive
        // name); archived S4 excluded. B: S3.
        assert_eq!(a, 2, "dedupe-by-source + alias + case-insensitive; archived excluded");
        assert_eq!(b, 1);
    }

    /// Safety inspection 2026-08-01 — the read/write gate split.
    ///
    /// A vocabulary change makes the stored aggregates untrustworthy (readers must fall back)
    /// but does NOT un-build them (writers must keep maintaining). Collapsing both questions
    /// into one predicate switched save-path maintenance off for the whole re-materialize,
    /// and the backfill's ascending cursor cannot revisit a target it has already passed — so
    /// a note saved mid-run left its target permanently stale, then served once the stamp
    /// landed. `is_built` must therefore stay TRUE across a fingerprint mismatch.
    #[test]
    fn is_built_survives_a_vocabulary_change_while_is_stamped_does_not() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL DEFAULT 0);",
        )
        .unwrap();

        // Never built: both false — the columns are genuinely inert, so the save path is
        // right to skip the work.
        assert!(!is_built(&conn), "never built");
        assert!(!is_stamped(&conn), "never built ⇒ never stamped");

        // Built and stamped under the CURRENT vocabulary: both true.
        let fp = crate::link_types::active_universe_vocabulary().fingerprint();
        conn.execute(
            "INSERT INTO schema_versions(module,version) VALUES ('incoming_links',?1)",
            [SCHEMA_VERSION],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO schema_versions(module,version) VALUES ('incoming_links_vocab',?1)",
            [fp],
        )
        .unwrap();
        assert!(is_built(&conn));
        assert!(is_stamped(&conn), "current vocabulary ⇒ trustworthy");

        // The vocabulary changes: readers must stop trusting the columns, writers must NOT
        // stop maintaining them. This is the whole point of the split.
        conn.execute(
            "UPDATE schema_versions SET version = ?1 WHERE module = 'incoming_links_vocab'",
            [fp.wrapping_add(1)],
        )
        .unwrap();
        assert!(
            !is_stamped(&conn),
            "vocabulary drifted ⇒ readers fall back to the live path"
        );
        assert!(
            is_built(&conn),
            "…but the aggregate still EXISTS, so save-path maintenance must keep running — \
             otherwise every target behind the re-materialize cursor is stranded stale"
        );
    }

    /// Safety-inspection 2026-08-01 — the vocabulary-fingerprint gate (the mirror
    /// of `links_backfill::vocab_fingerprint_gate_triggers_rematerialize`): with
    /// the version stamped, `is_stamped` is driven by the stored-vs-current
    /// vocabulary fingerprint — absent (a pre-gate universe) → not stamped;
    /// matching → stamped; differing (a vocabulary edit) → not stamped again,
    /// which both re-schedules the backfill and flips readers to the fallback.
    #[test]
    fn vocab_fingerprint_gate_unstamps_incoming() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO schema_versions (module, version) VALUES ('incoming_links', ?1)",
            params![SCHEMA_VERSION],
        )
        .unwrap();

        // Version stamped but no vocab stamp → 0 ≠ the seed fingerprint → not stamped.
        assert!(!is_stamped(&conn), "missing vocab stamp must read as NOT stamped");

        let fp = crate::link_types::active_universe_vocabulary().fingerprint();
        assert_ne!(fp, 0, "seed registry fingerprint is non-zero");
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('incoming_links_vocab', ?1)",
            params![fp],
        )
        .unwrap();
        assert!(is_stamped(&conn), "matching vocab stamp reads as stamped");

        // A vocabulary edit → differing stored fingerprint → unstamped again.
        conn.execute(
            "UPDATE schema_versions SET version = ?1 WHERE module = 'incoming_links_vocab'",
            params![fp ^ 0x5555],
        )
        .unwrap();
        assert!(!is_stamped(&conn), "changed vocab fingerprint must unstamp");
    }

    #[test]
    #[ignore = "rehearsal — needs a live-DB copy via INCOMING_REHEARSAL_DB"]
    fn rehearse_incoming_equals_getbacklinks() {
        let db = std::env::var("INCOMING_REHEARSAL_DB").expect("set INCOMING_REHEARSAL_DB");
        let target_path = std::env::var("INCOMING_TARGET_JSON").expect("set INCOMING_TARGET_JSON");
        let mut conn = Connection::open(&db).unwrap();
        // The copy predates the migration — add what init_db would.
        for sql in [
            "ALTER TABLE note_meta ADD COLUMN incoming_count INTEGER NOT NULL DEFAULT 0;",
            "ALTER TABLE note_meta ADD COLUMN incoming_link_types TEXT NOT NULL DEFAULT '';",
            "ALTER TABLE note_meta ADD COLUMN incoming_top_rank INTEGER NOT NULL DEFAULT 9;",
            "ALTER TABLE note_meta ADD COLUMN incoming_link_types_json TEXT NOT NULL DEFAULT '{}';",
            "ALTER TABLE note_links ADD COLUMN target_name_lower TEXT GENERATED ALWAYS AS (LOWER(target_name)) VIRTUAL;",
        ] {
            let _ = conn.execute_batch(sql);
        }
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_nl_tnl ON note_links(target_name_lower, status);",
        )
        .unwrap();
        // Mirror production: the copy carries the real note_meta FTS trigger.
        crate::search::register_fts5_tokenizer(&mut conn).unwrap();
        eprintln!("[incoming-rehearsal] assign SQL: {}", crate::search::incoming_aggregate_assignments(&crate::link_types::active_universe_vocabulary(), "note_meta"));

        let t = std::time::Instant::now();
        let n = crate::links_backfill::recompute_all_incoming(&conn, &crate::link_types::LinkTypeRegistry::seeds_only(), &crate::converge::ConvergeKey::for_test()).unwrap();
        eprintln!("[incoming-rehearsal] recompute_all_incoming: {} notes in {:?}", n, t.elapsed());

        let mut got: HashMap<String, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare("SELECT path, incoming_count FROM note_meta").unwrap();
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .unwrap();
            for r in rows {
                let (p, c) = r.unwrap();
                got.insert(p, c);
            }
        }
        let expected: HashMap<String, i64> =
            serde_json::from_str(&std::fs::read_to_string(&target_path).unwrap()).unwrap();

        let mut diffs = 0usize;
        let mut samples: Vec<(String, i64, i64)> = Vec::new();
        for (p, exp) in &expected {
            let g = got.get(p).copied().unwrap_or(-1);
            if g != *exp {
                diffs += 1;
                if samples.len() < 15 {
                    samples.push((p.clone(), *exp, g));
                }
            }
        }
        eprintln!(
            "[incoming-rehearsal] expected {} notes, recomputed {} notes, mismatches {}",
            expected.len(),
            got.len(),
            diffs
        );
        for (p, exp, g) in &samples {
            let base = p.rsplit(['\\', '/']).next().unwrap_or(p);
            eprintln!("   MISMATCH {base}: getBacklinks={exp} incoming_count={g}");
        }
        assert_eq!(diffs, 0, "incoming_count must equal getBacklinks for EVERY note");
        eprintln!("[incoming-rehearsal] PASS — incoming_count == getBacklinks for all {} notes", expected.len());
    }
}

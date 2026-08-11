//! PJ-249 §4 — one-shot backfill for the rename-cascade seek column `note_links.target_base`.
//!
//! Why: the cascade finds referrers by reading every markdown file in the universe (measured
//! live: 2,105 files / 140.8 MB / 8.3–8.5 s per rename, paid even for a title nothing links
//! to). `note_links` already knows the referrers — but `target_name` stores whatever the
//! wikilink spelled (`folder/a`, `a#h`, `foo::a`; 1,148 such rows on the live corpus), so a
//! seek on the bare title would silently miss them. §1 added `target_base` (the bare folded
//! title, `target_base_of`), §3 made every writer stamp it; this fills the rows that predate
//! §3, then stamps `schema_versions.target_base`. §6's cascade gate trusts the seek ONLY
//! where the stamp is present — a mixed universe keeps the walk.
//!
//! Design — convergent + restart-safe, the `name_fold_backfill` template:
//! - **Never blocks boot.** Background thread, dedicated connection, batched.
//! - **Convergent with the live write path.** Both compute `target_base_of(target_name)`;
//!   a row filled here then re-saved converges, and vice versa. A mid-run crash needs no
//!   journal: the next boot's run recomputes and the stamp only lands at the end.
//! - **THE DRIFT GUARD** (the rollback/LL-023 hole, and the addition over the template):
//!   `maybe_schedule` re-arms even when STAMPED if any row has `target_base IS NULL`. That
//!   is exactly what a session on an older build leaves behind — its column-listed INSERTs
//!   don't know the column — and what a §3-missing future writer would leave too. The stamp
//!   is deleted, the run repeats, the stamp returns. Rollback is not merely harmless; the
//!   return trip self-heals.
//! - **A `target_base`-only UPDATE fires no triggers** — both AU mirror triggers guard on
//!   column lists that exclude it, pinned by `tests_pj249_writer_stamps_target_base`.

use rusqlite::{params, Connection};
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Bump to force a one-time re-fold on existing DBs (e.g. if `target_base_of` changes).
///
/// **2 (§6f)** — not a fold change; a re-run to repair what version 1 LEFT BEHIND. Filling
/// 31,367 rows turned `target_base` from uniform-NULL into a column with ~3.8 rows per
/// value, and said nothing to the query planner: `sqlite_stat1` kept reporting the
/// pre-fill cardinality, so every plan on the column was chosen from a number the
/// back-fill itself had made false. Re-arming is how an already-stamped universe reaches
/// `widen_seek_index` + `analyze_note_links` — on the background thread, once, finding
/// zero dirty rows on the way through.
pub(crate) const SCHEMA_VERSION: i64 = 2;

const BATCH: usize = 500;

/// True once `target_base` has been back-filled + stamped. §6's cascade gate reads this:
/// stamp present → the index seek is trusted; absent → the filesystem walk, unchanged.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'target_base'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        >= SCHEMA_VERSION
}

/// What `maybe_schedule` decided, factored out so tests can drive it on a bare Connection.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Needs {
    /// Not stamped — first run (or a mid-run crash's resume).
    Fresh,
    /// Stamped, but a NULL row exists — an older build (or an unstamped writer) inserted
    /// behind the stamp's back. The stamp is a claim that is no longer true.
    Rearm,
    No,
}

pub(crate) fn needs_run(conn: &Connection) -> Needs {
    if !is_stamped(conn) {
        return Needs::Fresh;
    }
    // Phase-4 audit (4A + 4C independently) — an ERRORED probe reads as DIRTY, not clean.
    // `unwrap_or(false)` admitted the seek on the one state we could not verify; the
    // fail-safe direction is the walk plus a heal attempt. A genuinely broken DB also
    // fails the heal and the seek's own prepare, so the cost of leaning dirty is nil.
    let null_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM note_links WHERE target_base IS NULL)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(true);
    if null_exists {
        Needs::Rearm
    } else {
        Needs::No
    }
}

/// Schedule the backfill on a background thread. Silent no-op once stamped AND clean.
pub fn maybe_schedule(app: tauri::AppHandle) {
    let state = app.state::<SearchState>();
    let decision = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        needs_run(conn)
    };
    if decision == Needs::No {
        // PJ-249 §6e — nothing to back-fill, but the cascade's read path is still COLD.
        // Warm it here, on a background thread, instead of on the user's first rename.
        warm_seek_path(app.clone());
        return;
    }
    let app_bg = app.clone();
    std::thread::spawn(move || {
        if decision == Needs::Rearm {
            diag(
                &app_bg,
                "[target_base_backfill] RE-ARMED: NULL target_base rows exist behind the stamp \
                 (an older build ran, or a writer missed §3) — unstamping and re-running",
            );
        }
        match run(&app_bg) {
            Ok(n) => diag(&app_bg, &format!("[target_base_backfill] completed: {} rows updated", n)),
            Err(e) => diag(&app_bg, &format!("[target_base_backfill] FAILED (non-fatal): {}", e)),
        }
        // Same reason as the stamped path above — and here it matters more, because the
        // back-fill just rewrote every row of the index the cascade is about to seek.
        warm_seek_path(app_bg.clone());
    });
}

/// PJ-249 §6e — pay the cascade's cold-start cost at boot, on a background thread.
///
/// MEASURED on the Boss's universe, two renames 30 s apart on identical data:
/// **2,606 ms then 17 ms** for the same candidate lookup. Nothing about the second rename
/// is cheaper — the first one was paying for FIRST USE of the read-only connection this
/// session: SQLite parses the schema on that connection's first statement (this schema
/// carries a lot of trigger DDL), then faults in the `note_links` index pages, off a USB
/// mechanical disk. A user's rename is the wrong place to pay for that.
///
/// It runs THROUGH `with_read_conn` deliberately — the same connection
/// `cascade_candidates_via_index` uses — because warming any other one leaves that one's
/// schema parse unpaid.
///
/// §6f: it calls **the cascade's own seek function**, rather than a hand-written query
/// that resembles it. The first version did the latter and warmed the wrong pages
/// entirely: its `COVERING INDEX` plan and the cascade's full-`SCAN` plan touched
/// disjoint parts of the file, so the warm reported 674 ms of honest success and bought
/// the Boss nothing. A warm-up that is not literally the thing being warmed is a guess
/// about the query planner, and this one was wrong. The key cannot match any note (a
/// title cannot contain NUL), so this reads index pages and returns empty.
///
/// Best-effort and silent on failure: it can only ever make the first rename faster.
fn warm_seek_path(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let state = app.state::<SearchState>();
        let t = std::time::Instant::now();
        // §6g — `with_read_conn` holds `read_db` for the WHOLE closure (search.rs:1497).
        // That is fine for a covering-index seek (microseconds) and unacceptable for a full
        // table SCAN, which is what this degrades to if `widen_seek_index` failed — it is
        // non-fatal and the pass stamps regardless. A warm-up meant to save the first rename
        // would then block every other read at boot for seconds. So the shape is checked
        // first, cheaply, on the same connection; a universe whose rebuild did not take skips
        // the warm and pays the old cost on its first rename. Slow and correct beats fast and
        // blocking. (Surfaced by the safety sweep, against code written the same evening.)
        let warmed = crate::search::with_read_conn(state.inner(), |conn| {
            let covering: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM pragma_index_info('idx_link_target_base') \
                     WHERE seqno = 1)",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(false);
            if !covering {
                return Ok(false);
            }
            crate::libraries::cascade_candidates_via_index(conn, "\u{0}pj249-warm-no-such-title");
            Ok(true)
        });
        diag(
            &app,
            &match warmed {
                Ok(true) => format!(
                    "[target_base_backfill] seek path warmed in {} ms",
                    t.elapsed().as_millis()
                ),
                Ok(false) => "[target_base_backfill] seek warm SKIPPED: idx_link_target_base is \
                              not covering (the rebuild did not take) — the first rename will \
                              pay the full-scan cost"
                    .to_string(),
                Err(e) => format!("[target_base_backfill] seek warm unavailable: {}", e),
            },
        );
    });
}

/// PJ-249 §6f — give an EXISTING universe the covering index shape.
///
/// `CREATE INDEX IF NOT EXISTS` cannot do this: the name already exists, so it is a silent
/// no-op and every universe that booted the §1 build would keep the narrow index forever.
/// The shape has to be detected and the index rebuilt. `pragma_index_info` lists one row
/// per indexed column; a second row (`seqno = 1`) means `source_path` is already aboard.
///
/// Non-fatal by design: this makes the seek fast, never correct. A universe where the
/// rebuild fails keeps the narrow index and the old plan — slow, and right.
/// MEASURED: 118 ms to rebuild across 31,368 rows.
///
/// §6g — the drop-and-rebuild lives in `search::ensure_index_shape`, shared with
/// `link_boot_index`, because the same trap was found on two indexes within one hour and
/// a second hand-rolled copy is how it becomes a third.
fn widen_seek_index(conn: &Connection) {
    let expected = ["target_base".to_string(), "source_path".to_string()];
    if let Err(e) = crate::search::ensure_index_shape(
        conn,
        "idx_link_target_base",
        &expected,
        "CREATE INDEX IF NOT EXISTS idx_link_target_base ON note_links(target_base, source_path);",
    ) {
        eprintln!("[target_base_backfill] widen idx_link_target_base failed (non-fatal): {}", e);
    }
}

/// PJ-249 §6f — tell the query planner what this back-fill just did to the data.
///
/// A back-fill's whole job is to change a column from uniform to diverse, and that is
/// precisely the change `sqlite_stat1` cannot notice: it keeps reporting the cardinality
/// measured before the fill, and every plan for every query on that column is chosen from
/// the stale number. Here it bought a full table scan on each cold rename.
///
/// `widen_seek_index` already makes the CASCADE immune to that. This runs anyway, because
/// the column is now indexed and readable by anything, and leaving a knowingly-false
/// statistic in the database is a trap set for the next query written against it.
///
/// Scoped to `note_links` deliberately — a bare `ANALYZE` re-measures every table in a
/// 327 MB database to fix one. Non-fatal: statistics are an optimisation, and a universe
/// that cannot write them still answers every query correctly.
/// MEASURED: 89 ms.
fn analyze_note_links(conn: &Connection) {
    if let Err(e) = conn.execute_batch("ANALYZE note_links;") {
        eprintln!("[target_base_backfill] ANALYZE note_links failed (non-fatal): {}", e);
    }
}

/// The whole pass on a dedicated connection. Kept separate from `run_on` only by the
/// connection plumbing, so tests exercise the real logic.
fn run(app: &tauri::AppHandle) -> Result<usize, String> {
    let path = crate::search::db_path(app)?;
    let mut conn = Connection::open(&path).map_err(|e| format!("open target_base conn: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;
    // Defensive, per the template's precedent: nothing on this path should reach FTS
    // (the AU guards exclude target_base), but a legacy trigger that lost its guard
    // would otherwise fail with "no such tokenizer".
    crate::search::register_fts5_tokenizer(&mut conn)
        .map_err(|e| format!("register tokenizer: {}", e))?;
    run_on(&mut conn)
}

/// Recompute `target_base` for EVERY row and update the ones that differ (NULL included) —
/// self-healing for drifted values, not just missing ones. Unstamps first when re-arming
/// (so a crash mid-heal leaves the honest "not done" state), stamps only at the end.
pub(crate) fn run_on(conn: &mut Connection) -> Result<usize, String> {
    conn.execute("DELETE FROM schema_versions WHERE module = 'target_base'", [])
        .map_err(|e| format!("unstamp: {}", e))?;

    let all: Vec<(i64, String, Option<String>)> = {
        let mut stmt = conn
            .prepare("SELECT id, target_name, target_base FROM note_links")
            .map_err(|e| format!("select links: {}", e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| format!("query links: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // /simplify (efficiency) — compute the fold during the scan and retain only the
    // MISMATCHED pairs: on the recurring re-arm path (an old build left a handful of NULL
    // rows) the retained set is that handful, not 31k rows. And `prepare_cached` inside
    // the loop — `execute` re-prepares per call, ~0.15–0.3 s of avoidable one-shot work.
    let dirty: Vec<(i64, String)> = all
        .into_iter()
        .filter_map(|(id, target_name, stored)| {
            let base = crate::search::target_base_of(&target_name);
            if stored.as_deref() != Some(base.as_str()) {
                Some((id, base))
            } else {
                None
            }
        })
        .collect();
    let mut updated = 0usize;
    for chunk in dirty.chunks(BATCH) {
        let tx = conn.transaction().map_err(|e| format!("tx: {}", e))?;
        {
            let mut stmt = tx
                .prepare_cached("UPDATE note_links SET target_base = ?2 WHERE id = ?1")
                .map_err(|e| format!("prepare: {}", e))?;
            for (id, base) in chunk {
                stmt.execute(params![id, base])
                    .map_err(|e| format!("update target_base: {}", e))?;
                updated += 1;
            }
        }
        tx.commit().map_err(|e| format!("commit: {}", e))?;
    }

    // §6f — the column is now correct; make it USABLE. Both are non-fatal and both run
    // before the stamp, so a universe that fails them re-arms next boot and tries again.
    widen_seek_index(conn);
    analyze_note_links(conn);

    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('target_base', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    Ok(updated)
}

#[cfg(test)]
mod tests_pj249_6f_seek_plan {
    //! §6f — the cascade seek must be answered FROM THE INDEX, and must stay that way even
    //! when `sqlite_stat1` lies about the column.
    //!
    //! This is the test the migration was missing. Every §1–§4 test asserted the right
    //! `target_base` VALUES, and every one of them passed on a build whose seek full-scanned
    //! 31,368 rows per rename: the data was never wrong, the PLAN was. A correctness test
    //! cannot see a plan, so it has to be read directly.
    use super::*;

    /// The cascade's query, verbatim from `libraries::cascade_candidates_via_index`. If that
    /// query is ever reworded, this constant must follow it — a plan pinned to a query the
    /// app no longer runs is worse than no pin at all.
    const SEEK: &str = "SELECT DISTINCT source_path FROM note_links WHERE target_base = ?1";

    fn plan(conn: &Connection, sql: &str) -> String {
        let mut stmt = conn.prepare(&format!("EXPLAIN QUERY PLAN {}", sql)).unwrap();
        let rows = stmt
            .query_map(["zzz-no-such-note"], |r| r.get::<_, String>(3))
            .unwrap()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        rows.join(" / ")
    }

    /// `note_links` with enough of the real shape to plan against, plus the sibling index
    /// the planner actually chose in the field (`idx_link_source`) — without it there is no
    /// alternative to reject and the test proves nothing.
    fn corpus(widened: bool) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_path TEXT NOT NULL,
                target_name TEXT NOT NULL,
                link_type TEXT NOT NULL DEFAULT 'relates',
                target_base TEXT
             );
             CREATE INDEX idx_link_source ON note_links(source_path);",
        )
        .unwrap();
        conn.execute_batch(if widened {
            "CREATE INDEX idx_link_target_base ON note_links(target_base, source_path);"
        } else {
            "CREATE INDEX idx_link_target_base ON note_links(target_base);"
        })
        .unwrap();
        {
            let tx = conn.unchecked_transaction().unwrap();
            for i in 0..2000 {
                tx.execute(
                    "INSERT INTO note_links (source_path, target_name, target_base)
                     VALUES (?1, ?2, ?2)",
                    params![format!("/n/{}.md", i), format!("t{}", i % 500)],
                )
                .unwrap();
            }
            tx.commit().unwrap();
        }
        conn
    }

    /// THE REGRESSION. `sqlite_stat1` is written to say what it said on the Boss's live DB:
    /// one distinct `target_base` across every row — the truth as of before the back-fill,
    /// and a lie afterwards. The narrow index loses to it; the covering index must not.
    fn poison_stats(conn: &Connection) {
        conn.execute_batch(
            "ANALYZE;
             DELETE FROM sqlite_stat1 WHERE idx = 'idx_link_target_base';
             INSERT INTO sqlite_stat1 (tbl, idx, stat)
                VALUES ('note_links', 'idx_link_target_base', '2000 2000');
             ANALYZE sqlite_master;",
        )
        .unwrap();
    }

    #[test]
    fn narrow_index_loses_to_a_stale_statistic() {
        // Not an aspiration — a record of the shipped defect, so the fix is provably a fix
        // and not a coincidence. If this ever stops full-scanning, the premise changed.
        let conn = corpus(false);
        poison_stats(&conn);
        let p = plan(&conn, SEEK);
        assert!(
            !p.contains("idx_link_target_base"),
            "premise check: the narrow index was expected to be REJECTED under the stale              stat (that is the bug §6f fixes); planner said: {}",
            p
        );
    }

    #[test]
    fn covering_index_wins_despite_a_stale_statistic() {
        let conn = corpus(true);
        poison_stats(&conn);
        let p = plan(&conn, SEEK);
        assert!(
            p.contains("COVERING INDEX idx_link_target_base"),
            "the seek must be answered from the covering index even when sqlite_stat1              claims the column has one distinct value; planner said: {}",
            p
        );
    }

    #[test]
    fn covering_index_wins_with_no_statistics_at_all() {
        // A fresh universe has never run ANALYZE. The plan must not depend on that.
        let conn = corpus(true);
        let p = plan(&conn, SEEK);
        assert!(
            p.contains("COVERING INDEX idx_link_target_base"),
            "planner said: {}",
            p
        );
    }

    #[test]
    fn widen_seek_index_rebuilds_a_narrow_index_and_is_idempotent() {
        let conn = corpus(false);
        let cols = |c: &Connection| -> i64 {
            c.query_row(
                "SELECT COUNT(*) FROM pragma_index_info('idx_link_target_base')",
                [],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(cols(&conn), 1, "starts narrow");
        widen_seek_index(&conn);
        assert_eq!(cols(&conn), 2, "widened to (target_base, source_path)");
        widen_seek_index(&conn); // second boot: must not churn
        assert_eq!(cols(&conn), 2, "idempotent");
        // and the rows survived the rebuild
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM note_links", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2000
        );
    }

    #[test]
    fn a_run_leaves_the_planner_told_and_the_index_covering() {
        // The end-to-end claim: after the pass, a stale-stat universe seeks the index.
        let mut conn = corpus(false);
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        poison_stats(&conn);
        run_on(&mut conn).unwrap();
        assert!(is_stamped(&conn), "stamped at the new version");
        let p = plan(&conn, SEEK);
        assert!(
            p.contains("COVERING INDEX idx_link_target_base"),
            "planner said: {}",
            p
        );
        let stat: String = conn
            .query_row(
                "SELECT stat FROM sqlite_stat1 WHERE idx = 'idx_link_target_base'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_ne!(stat, "2000 2000", "ANALYZE must have replaced the poisoned stat");
    }
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests_pj249_backfill {
    //! Real `init_db` fixtures (the mirror trap forbids hand-built schemas). One test per
    //! plan clause: dirty rows healed; converged re-run is a no-op; unstamped-partial
    //! resumes to a stamp; and the drift guard re-arms on a NULL row behind the stamp.
    use super::{is_stamped, needs_run, run_on, Needs};
    use crate::search::init_db;
    use rusqlite::params;

    // /simplify (reuse) — tempfile::TempDir cleans on drop, panic included; the hand-rolled
    // helper this replaces leaked its directory on every failing assert.
    fn tmp_db(_tag: &str) -> (rusqlite::Connection, tempfile::TempDir) {
        let td = tempfile::tempdir().expect("tempdir");
        let conn = init_db(&td.path().join("search.db")).expect("init_db");
        (conn, td)
    }

    /// A pre-§3 row: column-listed INSERT that omits target_base — the old-build shape.
    fn insert_old_shape(conn: &rusqlite::Connection, src: &str, target: &str) {
        conn.execute(
            "INSERT INTO note_links (source_path, source_name, target_name, link_type, status)
             VALUES (?1, 'S', ?2, 'associative', 'active')",
            params![src, target],
        )
        .unwrap();
    }

    #[test]
    fn dirty_rows_heal_and_the_stamp_lands() {
        let (mut conn, d) = tmp_db("dirty");
        insert_old_shape(&conn, "/a.md", "folder/nested note");
        insert_old_shape(&conn, "/b.md", "anchored#heading");
        insert_old_shape(&conn, "/c.md", "clean title");
        assert_eq!(needs_run(&conn), Needs::Fresh);

        let n = run_on(&mut conn).unwrap();
        assert_eq!(n, 3);
        assert!(is_stamped(&conn));
        let nulls: i64 = conn
            .query_row("SELECT COUNT(*) FROM note_links WHERE target_base IS NULL", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nulls, 0);
        let base: String = conn
            .query_row(
                "SELECT target_base FROM note_links WHERE source_path='/a.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(base, "nested note");
    }

    #[test]
    fn a_converged_second_run_updates_nothing() {
        let (mut conn, d) = tmp_db("noop");
        insert_old_shape(&conn, "/a.md", "folder/x");
        run_on(&mut conn).unwrap();
        assert_eq!(needs_run(&conn), Needs::No);
        assert_eq!(run_on(&mut conn).unwrap(), 0); // recompute finds nothing to change
        assert!(is_stamped(&conn));
    }

    /// The crash shape: some rows filled, NO stamp (run_on stamps only at the end).
    /// The next boot sees Fresh and completes.
    #[test]
    fn an_unstamped_partial_state_resumes_to_a_stamp() {
        let (mut conn, d) = tmp_db("resume");
        insert_old_shape(&conn, "/a.md", "folder/x");
        insert_old_shape(&conn, "/b.md", "folder/y");
        // Simulate the crash's partial progress: one row filled by hand, no stamp.
        conn.execute("UPDATE note_links SET target_base='x' WHERE source_path='/a.md'", [])
            .unwrap();
        assert_eq!(needs_run(&conn), Needs::Fresh);
        let n = run_on(&mut conn).unwrap();
        assert_eq!(n, 1, "only the unfinished row needs an update");
        assert!(is_stamped(&conn));
    }

    /// THE DRIFT GUARD — the reason rollback is safe. An older build inserts a row behind
    /// the stamp (its INSERT doesn't know the column → NULL): the stamp must stop being
    /// trusted, the re-run must heal, the stamp must return.
    #[test]
    fn a_null_row_behind_the_stamp_rearms_and_heals() {
        let (mut conn, d) = tmp_db("rearm");
        insert_old_shape(&conn, "/a.md", "clean");
        run_on(&mut conn).unwrap();
        assert_eq!(needs_run(&conn), Needs::No);

        // The older build's session:
        insert_old_shape(&conn, "/b.md", "folder/added on old build");
        assert_eq!(needs_run(&conn), Needs::Rearm);

        let n = run_on(&mut conn).unwrap();
        assert_eq!(n, 1);
        assert_eq!(needs_run(&conn), Needs::No);
        let base: String = conn
            .query_row(
                "SELECT target_base FROM note_links WHERE source_path='/b.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(base, "added on old build");
    }
}

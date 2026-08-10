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
pub(crate) const SCHEMA_VERSION: i64 = 1;

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
    });
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

    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('target_base', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    Ok(updated)
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

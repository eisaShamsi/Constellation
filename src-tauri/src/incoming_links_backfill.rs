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

/// True once the incoming aggregate has been built + stamped. The read-flip in
/// `constellation_search_link_counts` and the reconcile recompute gate on this.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'incoming_links'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        >= SCHEMA_VERSION
}

/// Schedule the one-shot backfill on a background thread. Silent no-op once
/// stamped. Mirrors `note_body_backfill::maybe_schedule`.
pub fn maybe_schedule(app: tauri::AppHandle) {
    let state = app.state::<SearchState>();
    let needs_run = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        !is_stamped(conn)
    };
    if !needs_run {
        return;
    }
    let app_bg = app.clone();
    std::thread::spawn(move || match run(&app_bg) {
        Ok(n) => diag(&app_bg, &format!("[incoming_links_backfill] completed: {} notes recomputed", n)),
        Err(e) => diag(&app_bg, &format!("[incoming_links_backfill] FAILED (non-fatal): {}", e)),
    });
}

/// Recompute every note's incoming aggregate on a DEDICATED connection (batched,
/// busy-tolerant) then stamp. Convergent with the live triggers (both read
/// note_links), so no single-transaction atomicity is required.
fn run(app: &tauri::AppHandle) -> Result<usize, String> {
    let path = crate::search::db_path(app)?;
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

    let n = crate::links_backfill::recompute_all_incoming(&conn)
        .map_err(|e| format!("recompute: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('incoming_links', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    Ok(n as usize)
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
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
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT,
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
        crate::links_backfill::recompute_all_incoming(&conn).unwrap();
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
        eprintln!("[incoming-rehearsal] assign SQL: {}", crate::search::incoming_aggregate_assignments("note_meta"));

        let t = std::time::Instant::now();
        let n = crate::links_backfill::recompute_all_incoming(&conn).unwrap();
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

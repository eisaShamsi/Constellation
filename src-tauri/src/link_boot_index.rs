//! MIG-079 §C.3 — the boot-edge COVERING index `idx_link_boot`.
//!
//! `cache_full_links` (the deferred edge load — §C.2b) runs the projection named by
//! [`BOOT_LINK_COLUMNS`] — **which is now the single source of that list**, shared with
//! `cache::read_links_in_schema` and with this module's own test. It used to be written
//! out here in prose, in `cache.rs`, and a third time inside the test; PJ-249 §6g found
//! that the three had drifted (`created` joined the query in `6c810836` and reached
//! neither the index nor the test), so the guarantee below was false from June until
//! 2026-08-10 while the test reported green. Every row is `status='active'`
//! on the reference universe, so without a covering index SQLite does a full
//! scan of the WIDE `note_links` row-store (`id`, `target_path`, `created`,
//! the virtual `target_name_lower`, … — pages the scan never needs). This
//! index carries EXACTLY the scan's columns (leading `status` for the
//! equality seek) so the lazy scan reads narrow index leaf pages only →
//! `EXPLAIN QUERY PLAN` reports `USING COVERING INDEX idx_link_boot`.
//! `idx_link_boot_covers_every_projected_column` now pins that as a STRUCTURAL claim —
//! it compares the index against `BOOT_LINK_COLUMNS` itself, so the next widening of the
//! projection fails the suite instead of quietly un-covering the index.
//!
//! Column-set note (deviation from the Plan's literal list, measurement-
//! justified — see SESSION-LOG-2026-06-17): the list INCLUDES `source_name`
//! (required by `getBacklinks`/`buildSkyData`) and `annotation`. For a query
//! that returns every row, a NON-covering index is ignored in favour of a
//! table scan, so the index must contain every selected column or the
//! "USING COVERING INDEX" goal is unreachable. Measured: 233,995 rows, total
//! annotation text only ~1.33 MB (~6 B/row) — including it is negligible and
//! keeps every frontend consumer byte-identical (no per-row annotation IPC).
//! `context` stays out (always `''` at boot).
//!
//! Build discipline — mirrors `incoming_links_backfill` / `tag_counts`:
//! background thread, own connection, `CREATE INDEX IF NOT EXISTS` (no-op
//! once built), busy-tolerant, stamped in `schema_versions` so it runs at
//! most once. It is its OWN module (not folded into the §C.2a backfill,
//! whose run is gated on the already-set `incoming_links` stamp and so would
//! never fire on an existing universe). CREATE INDEX is pure DDL — it fires
//! NO row triggers — so no FTS tokenizer registration is needed. The one-time
//! build write-locks the DB for its duration (busy_timeout covers concurrent
//! saves), same accepted property as the §C.2a `idx_nl_tnl` build.

use rusqlite::{params, Connection};
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Every column the boot edge load projects, EXCEPT `status` (which leads the index for
/// the equality seek and is appended to the query separately). Order matches the `row.get`
/// indices in `cache::read_links_in_schema` and must not be reordered casually.
///
/// §6g — this exists so the query and the index that must cover it cannot be edited
/// apart. `context` stays out (always `''` at boot).
pub(crate) const BOOT_LINK_COLUMNS: &str = "source_path, source_name, target_name, \
     link_type, library_name, weight, traversal_count, annotation, last_traversed, \
     confidence, created";

/// The index's columns, derived from [`BOOT_LINK_COLUMNS`] so the two cannot disagree:
/// leading `status` for the equality seek, then every projected column.
fn boot_index_columns() -> Vec<String> {
    std::iter::once("status".to_string())
        .chain(BOOT_LINK_COLUMNS.split(',').map(|c| c.trim().to_string()))
        .collect()
}

/// Bump to force a rebuild (e.g. if the covering column set changes).
///
/// **2 (§6g)** — and the bump now actually rebuilds. Version 1 shipped this doc line above
/// a `CREATE INDEX IF NOT EXISTS` with no `DROP`: the statement keys on the index NAME, so
/// bumping re-ran a no-op and re-stamped, and no existing universe could ever receive a new
/// column set. `search::ensure_index_shape` is what makes the promise true.
pub(crate) const SCHEMA_VERSION: i64 = 2;

/// True once `idx_link_boot` has been built + stamped.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'link_boot_index'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        >= SCHEMA_VERSION
}

/// Schedule the one-shot index build on a background thread. Silent no-op once
/// stamped. Mirrors `incoming_links_backfill::maybe_schedule`.
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
        Ok(()) => diag(&app_bg, "[link_boot_index] idx_link_boot built + stamped"),
        Err(e) => diag(&app_bg, &format!("[link_boot_index] FAILED (non-fatal): {}", e)),
    });
}

/// Create `idx_link_boot` on a DEDICATED connection then stamp. `IF NOT
/// EXISTS` → no-op when the index already exists; the stamp prevents the
/// thread from even spawning on subsequent boots.
fn run(app: &tauri::AppHandle) -> Result<(), String> {
    let path = crate::search::db_path(app)?;
    let conn = Connection::open(&path).map_err(|e| format!("open link_boot_index conn: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;

    // The covering index for `cache_full_links`'s active-edge scan, built FROM the
    // projection itself. Leading `status` enables the equality seek; the remaining columns
    // make the scan index-only.
    //
    // MEASURED on a copy of the live DB (31,368 active rows) before shipping the widening,
    // because "covered" is not automatically "cheap" and a wider index can lose to the
    // scan it replaces: 71.8 ms scanning vs 70.2 ms covering, rebuild 232 ms once, database
    // file +0 KB. No gain visible warm — that benchmark is CPU-bound on materialising every
    // row — but no regression either, and it restores the invariant this module exists for.
    crate::search::ensure_index_shape(
        &conn,
        "idx_link_boot",
        &boot_index_columns(),
        &format!(
            "CREATE INDEX IF NOT EXISTS idx_link_boot ON note_links(status, {});",
            BOOT_LINK_COLUMNS
        ),
    )
    .map_err(|e| format!("create idx_link_boot: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('link_boot_index', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    Ok(())
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The covering index must satisfy `cache_full_links`'s exact active-edge
    /// projection with `USING COVERING INDEX` — i.e. the scan reads index leaf
    /// pages only, never the wide `note_links` row-store. Builds a tiny
    /// note_links, creates `idx_link_boot`, and asserts the plan.
    #[test]
    fn idx_link_boot_is_covering_for_the_boot_scan() {
        let conn = Connection::open_in_memory().unwrap();
        // The full note_links shape (only the columns the scan + index touch
        // need real values; the rest mirror production defaults/width).
        let conn = corpus();
        // §6g — the projection is BUILT from the same constant `cache.rs` builds it from.
        // The previous version of this test wrote the columns out by hand — ten of them,
        // missing `created` and `status` — and so asserted a covering plan for a query the
        // app does not run, staying green from June to 2026-08-10 while production scanned.
        let joined = plan_for(&conn, &boot_scan_sql());
        assert!(
            joined.contains("USING COVERING INDEX idx_link_boot"),
            "expected a covering-index scan for the REAL projection, got: {joined}"
        );
    }

    /// The structural half, and the one that actually holds the line: **every column the
    /// boot projection selects must be IN the index.** The plan test above can be defeated
    /// by a planner that happens to choose the index anyway; this cannot. A future widening
    /// of `BOOT_LINK_COLUMNS` fails here immediately, at the edit that causes it.
    #[test]
    fn idx_link_boot_covers_every_projected_column() {
        let conn = corpus();
        let indexed: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM pragma_index_info('idx_link_boot') ORDER BY seqno")
                .unwrap();
            let rows = stmt.query_map([], |r| r.get::<_, String>(0)).unwrap();
            rows.map(|r| r.unwrap()).collect()
        };
        for col in BOOT_LINK_COLUMNS.split(',').map(str::trim) {
            assert!(
                indexed.iter().any(|c| c == col),
                "`{}` is projected by the boot scan but missing from idx_link_boot — the \
                 index no longer covers the query it exists for. Add it to the index (and \
                 bump SCHEMA_VERSION so existing universes rebuild), or drop it from \
                 BOOT_LINK_COLUMNS. Index has: {:?}",
                col,
                indexed
            );
        }
        assert_eq!(indexed.first().map(String::as_str), Some("status"),
            "`status` must LEAD the index for the equality seek; got {:?}", indexed);
    }

    /// §6g — a shape change must actually REACH a universe that already has the index.
    /// `CREATE INDEX IF NOT EXISTS` alone cannot: it keys on the name. This pins the
    /// repair, because the module documented it for a year without having it.
    #[test]
    fn a_stale_index_shape_is_rebuilt_not_silently_kept() {
        let conn = corpus();
        conn.execute_batch(
            "DROP INDEX idx_link_boot;
             CREATE INDEX idx_link_boot ON note_links(status, source_path);",
        )
        .unwrap();
        crate::search::ensure_index_shape(
            &conn,
            "idx_link_boot",
            &boot_index_columns(),
            &format!(
                "CREATE INDEX IF NOT EXISTS idx_link_boot ON note_links(status, {});",
                BOOT_LINK_COLUMNS
            ),
        )
        .unwrap();
        let joined = plan_for(&conn, &boot_scan_sql());
        assert!(
            joined.contains("USING COVERING INDEX idx_link_boot"),
            "the outdated 2-column index should have been dropped and rebuilt; got: {joined}"
        );
    }

    /// The production projection, assembled the way `cache::read_links_in_schema`
    /// assembles it — from the constant, never by hand.
    fn boot_scan_sql() -> String {
        format!(
            "SELECT {}, status FROM note_links WHERE status = 'active'",
            BOOT_LINK_COLUMNS
        )
    }

    fn plan_for(conn: &Connection, sql: &str) -> String {
        let mut stmt = conn.prepare(&format!("EXPLAIN QUERY PLAN {}", sql)).unwrap();
        let rows = stmt.query_map([], |r| r.get::<_, String>(3)).unwrap();
        rows.map(|r| r.unwrap()).collect::<Vec<_>>().join(" | ")
    }

    /// The full `note_links` shape, with the index built the way `run` builds it.
    fn corpus() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_path TEXT NOT NULL,
                source_name TEXT NOT NULL,
                target_path TEXT,
                target_name TEXT NOT NULL,
                link_type TEXT NOT NULL DEFAULT 'relates',
                annotation TEXT DEFAULT '',
                confidence TEXT DEFAULT 'hypothesis',
                weight REAL DEFAULT 1.0,
                created TEXT DEFAULT '',
                last_traversed TEXT DEFAULT '',
                traversal_count INTEGER DEFAULT 0,
                library_name TEXT DEFAULT '',
                status TEXT DEFAULT 'active'
            );",
        )
        .unwrap();
        conn.execute_batch(&format!(
            "CREATE INDEX IF NOT EXISTS idx_link_boot ON note_links(status, {});",
            BOOT_LINK_COLUMNS
        ))
        .unwrap();
        conn
    }

    /// §6g — the ORDER of `BOOT_LINK_COLUMNS` is load-bearing, and this is the risk the
    /// constant itself introduces: `cache::read_links_in_schema` reads its results
    /// POSITIONALLY (`row.get(0)` … `row.get(11)`), so re-ordering the constant does not
    /// fail to compile and does not fail any other test — it silently feeds every link's
    /// annotation into its confidence, its weight into its traversal count, and so on, for
    /// every edge loaded at boot.
    ///
    /// Extracting one shared string removed the drift between the query and the index and
    /// put this in its place. Pinned deliberately: changing the order must be a decision,
    /// taken together with the `row.get` indices in `cache.rs`.
    #[test]
    fn boot_projection_order_is_pinned_to_the_positional_reads_in_cache_rs() {
        let cols: Vec<&str> = BOOT_LINK_COLUMNS.split(',').map(str::trim).collect();
        assert_eq!(
            cols,
            vec![
                "source_path",      // row.get(0)
                "source_name",      // row.get(1)
                "target_name",      // row.get(2)
                "link_type",        // row.get(3)
                "library_name",     // row.get(4)
                "weight",           // row.get(5)
                "traversal_count",  // row.get(6)
                "annotation",       // row.get(7)
                "last_traversed",   // row.get(8)
                "confidence",       // row.get(9)
                "created",          // row.get(10)
                                    // `status` is appended by the caller -> row.get(11)
            ],
            "BOOT_LINK_COLUMNS changed order. cache::read_links_in_schema reads these by              POSITION - update its row.get indices in the same commit, or put the order back."
        );
    }

    /// `is_stamped` gates the one-shot build: false before, true at/after the
    /// stamp — so the build thread never re-spawns once it has run.
    #[test]
    fn stamp_gate_flips_after_run() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        assert!(!is_stamped(&conn), "unstamped DB must report not-stamped");
        conn.execute(
            "INSERT INTO schema_versions (module, version, updated_at) VALUES ('link_boot_index', ?1, 0)",
            params![SCHEMA_VERSION],
        )
        .unwrap();
        assert!(is_stamped(&conn), "stamped DB must report stamped");
    }
}

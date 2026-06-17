//! MIG-079 §C.3 — the boot-edge COVERING index `idx_link_boot`.
//!
//! `cache_full_links` (the deferred edge load — §C.2b) runs
//! `SELECT source_path, source_name, target_name, link_type, library_name,
//!  weight, traversal_count, annotation, last_traversed, confidence
//!  FROM note_links WHERE status='active'`. Every row is `status='active'`
//! on the reference universe, so without a covering index SQLite does a full
//! scan of the WIDE `note_links` row-store (`id`, `target_path`, `created`,
//! the virtual `target_name_lower`, … — pages the scan never needs). This
//! index carries EXACTLY the scan's columns (leading `status` for the
//! equality seek) so the lazy scan reads narrow index leaf pages only →
//! `EXPLAIN QUERY PLAN` reports `USING COVERING INDEX idx_link_boot`.
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

/// Bump to force a rebuild (e.g. if the covering column set changes).
pub(crate) const SCHEMA_VERSION: i64 = 1;

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

    // The covering index for `cache_full_links`'s active-edge scan. Leading
    // `status` enables the equality seek; the remaining columns make the scan
    // index-only. `context` is intentionally absent (always '' at boot).
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_link_boot ON note_links(\
            status, source_path, source_name, target_name, link_type, \
            library_name, weight, traversal_count, last_traversed, confidence, annotation\
        );",
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
            );
            CREATE INDEX IF NOT EXISTS idx_link_boot ON note_links(
                status, source_path, source_name, target_name, link_type,
                library_name, weight, traversal_count, last_traversed, confidence, annotation
            );",
        )
        .unwrap();

        // The EXACT projection cache_full_links / read_links_in_schema runs.
        let plan: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "EXPLAIN QUERY PLAN \
                     SELECT source_path, source_name, target_name, link_type, library_name, \
                            weight, traversal_count, annotation, last_traversed, confidence \
                     FROM note_links WHERE status = 'active'",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(3))
                .unwrap();
            rows.map(|r| r.unwrap()).collect()
        };
        let joined = plan.join(" | ");
        assert!(
            joined.contains("USING COVERING INDEX idx_link_boot"),
            "expected a covering-index scan, got: {joined}"
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

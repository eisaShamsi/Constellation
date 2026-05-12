//! MIG-024 §2 — Sight v5 layout cache + IPCs.
//!
//! Per Concept Paper v3.1 §11.1: first-toggle latency on a 7,636-note
//! universe must hit ≤ 500 ms cold / ≤ 50 ms warm. The path to the warm
//! budget is a write-time-derived SQLite cache (`sight_v5_layout`) read
//! in milliseconds, never recomputed at read time.
//!
//! Per CLAUDE.md Performance Rule 8 (write-time derivation): the cache is
//! maintained at write time via a SQLite trigger on `note_meta` UPDATE.
//! Invalidation is per-row (DELETE the affected note's row); the next
//! `sight_v5_get_layout` IPC re-derives the missing row(s) on demand.
//!
//! D-V4 (Eisa, 2026-05-12) locked the per-note × 1 row strategy: one
//! row per `(note_path)`, NOT per-(note × mode). Mode-specific azimuth
//! is computed at render time in JS (cheap; just a lookup or
//! date-modulo per Concept Paper §6).
//!
//! D-V3 (Eisa, 2026-05-12) locked user-toggleable scope (universe /
//! library / folder). The `sight_v5_get_layout(scope_kind, scope_id)`
//! IPC applies the scope filter at SELECT time; same per-mode JS
//! reprojection runs against the filtered set.
//!
//! Schema:
//!   CREATE TABLE sight_v5_layout (
//!       note_path TEXT PRIMARY KEY,
//!       stratum INTEGER,            -- 1..8 (parsed from sky_nodes.stratum text)
//!       maturity TEXT,              -- 'seed'|'sapling'|'evergreen'|'canonical'|'wilting'
//!       confidence_alpha REAL,      -- 0.45 (hypothesis) | 0.7 (evidence) | 1.0 (established)
//!       contested INTEGER NOT NULL DEFAULT 0,
//!       library_name TEXT,
//!       folder_path TEXT,
//!       created_month INTEGER,      -- 0..11 from created_at epoch
//!       sources_primary TEXT,       -- json_extract(sources, '$[0]')
//!       stage TEXT,
//!       acts_primary TEXT,
//!       dominant_link_type TEXT,
//!       computed_at INTEGER NOT NULL
//!   );
//!
//! Snapshot hash: a per-Universe fingerprint of `note_meta + note_links`
//! state, used by the frontend to detect when the cached layout no
//! longer matches the live data. Cheap aggregation (COUNT + MAX(modified))
//! — no cryptographic hash needed.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Per-note cache row returned by `sight_v5_get_layout`.
///
/// Serde `rename_all = "camelCase"` aligns JSON output with the
/// TypeScript LayoutCacheRow contract in `src/lib/sight/v5/types.ts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutCacheRow {
    pub note_path: String,
    pub stratum: Option<i64>,
    pub maturity: Option<String>,
    pub confidence_alpha: Option<f64>,
    pub contested: bool,
    pub library_name: Option<String>,
    pub folder_path: Option<String>,
    pub created_month: Option<i64>,
    pub sources_primary: Option<String>,
    pub stage: Option<String>,
    pub acts_primary: Option<String>,
    pub dominant_link_type: Option<String>,
    pub computed_at: i64,
}

/// One typed-link edge between two visible notes — read by §5
/// connector-line rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEdge {
    pub source_path: String,
    pub target_path: String,
    pub link_type: String,
    pub confidence: String,
}

/// Idempotent table + index creation. Called once per `init_db` after
/// `note_meta` and `sky_nodes` schemas are in place (so JOIN-based
/// backfill resolves cleanly).
pub fn ensure_sight_v5_layout_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sight_v5_layout (
            note_path           TEXT PRIMARY KEY,
            stratum             INTEGER,
            maturity            TEXT,
            confidence_alpha    REAL,
            contested           INTEGER NOT NULL DEFAULT 0,
            library_name        TEXT,
            folder_path         TEXT,
            created_month       INTEGER,
            sources_primary     TEXT,
            stage               TEXT,
            acts_primary        TEXT,
            dominant_link_type  TEXT,
            computed_at         INTEGER NOT NULL
        );
        -- Covering index for scope filters (library + folder are the
        -- common WHERE clauses; stratum is read every query).
        CREATE INDEX IF NOT EXISTS idx_sight_v5_layout_library
            ON sight_v5_layout(library_name);
        CREATE INDEX IF NOT EXISTS idx_sight_v5_layout_folder
            ON sight_v5_layout(folder_path);",
    )
    .map_err(|e| format!("ensure_sight_v5_layout_table: {}", e))
}

/// Cache-invalidation trigger. Fires on note_meta UPDATE — DELETEs
/// the affected note's cache row so the next `get_layout` call
/// re-derives. Cheap (one DELETE on a small index).
///
/// Note: AFTER UPDATE on note_meta, not AFTER INSERT/DELETE — the
/// MIG-024 §0 UPSERT remediation means the canonical re-index path
/// fires UPDATE not DELETE+INSERT, so this trigger catches every
/// re-index correctly.
pub fn ensure_sight_v5_invalidation_trigger(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS sight_v5_layout_invalidate_au
        AFTER UPDATE ON note_meta
        BEGIN
            DELETE FROM sight_v5_layout WHERE note_path = OLD.path;
        END;
        CREATE TRIGGER IF NOT EXISTS sight_v5_layout_invalidate_ad
        AFTER DELETE ON note_meta
        BEGIN
            DELETE FROM sight_v5_layout WHERE note_path = OLD.path;
        END;",
    )
    .map_err(|e| format!("ensure_sight_v5_invalidation_trigger: {}", e))
}

/// Compute a cheap per-Universe snapshot fingerprint. Changes whenever
/// `note_meta` or `note_links` state changes meaningfully. The
/// frontend compares this against its cached value to detect
/// invalidation.
pub fn compute_universe_snapshot_hash(conn: &Connection) -> Result<String, String> {
    let row: (i64, i64, i64, i64) = conn
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM note_meta),
                (SELECT COALESCE(MAX(modified), 0) FROM note_meta),
                (SELECT COUNT(*) FROM note_links),
                (SELECT COUNT(*) FROM sight_v5_layout)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e| format!("compute_universe_snapshot_hash: {}", e))?;
    Ok(format!("{}-{}-{}-{}", row.0, row.1, row.2, row.3))
}

/// First-boot backfill: populate `sight_v5_layout` for all existing
/// notes. Idempotent via `schema_versions` sentinel
/// `'mig024_sight_v5_layout_backfill_v1'`. Resumable: re-running after
/// a partial run continues from where it stopped (the per-row INSERT
/// OR REPLACE pattern means re-inserting an existing row is a no-op
/// for that row).
///
/// Backfill strategy: bulk `INSERT OR REPLACE INTO sight_v5_layout
/// SELECT ... FROM note_meta LEFT JOIN sky_nodes ON ...` — pure SQL,
/// no row-by-row loop. The trigger-invalidation only fires on
/// note_meta UPDATE; this seed bulk-INSERT into the cache table
/// directly bypasses it.
///
/// Returns the number of cache rows after the backfill (== note_meta
/// row count after a successful run).
pub fn backfill_sight_v5_layout(conn: &mut Connection) -> Result<usize, String> {
    // Sentinel v2 (2026-05-12): v1's bulk INSERT joined sky_nodes on
    // sn.id (lowercased note name) instead of sn.path (note_meta path).
    // Every row landed with NULL stratum because the JOIN never matched
    // the right column. Empty dome on Eisa's first install. v2 fixes
    // the JOIN and forces a re-run on any DB that stamped v1.
    const SENTINEL_KEY: &str = "mig024_sight_v5_layout_backfill_v2";
    const SENTINEL_KEY_V1: &str = "mig024_sight_v5_layout_backfill_v1";

    // Check sentinel: if v2 already done, return cache count and exit.
    let already_v2: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = ?1",
            params![SENTINEL_KEY],
            |r| r.get(0),
        )
        .ok();
    if already_v2.is_some() {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sight_v5_layout", [], |r| r.get(0))
            .map_err(|e| format!("backfill: count: {}", e))?;
        return Ok(count as usize);
    }

    // Detect v1 sentinel — wipe the v1 cache rows so the v2 backfill
    // re-derives cleanly. No data loss (cache is ephemeral, rebuildable
    // from note_meta + sky_nodes).
    let already_v1: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = ?1",
            params![SENTINEL_KEY_V1],
            |r| r.get(0),
        )
        .ok();
    if already_v1.is_some() {
        conn.execute("DELETE FROM sight_v5_layout", [])
            .map_err(|e| format!("backfill: clear v1 cache: {}", e))?;
        conn.execute(
            "DELETE FROM schema_versions WHERE module = ?1",
            params![SENTINEL_KEY_V1],
        )
        .map_err(|e| format!("backfill: clear v1 sentinel: {}", e))?;
    }

    // Bulk INSERT OR REPLACE: derives per-note layout from note_meta
    // (sources, stage via json_extract on properties_json, created_at
    // → month) + sky_nodes (stratum, maturity). LEFT JOIN sky_nodes
    // so notes without a sky_nodes row still appear (sparse stratum
    // is acceptable; mode P empty-state CTA per D-V6 will catch it).
    //
    // Stratum text 'L1'..'L8' parsed to INTEGER 1..8 via SUBSTR.
    // Created_month derives from epoch via strftime('%m', ...) - 1.
    // Sources primary: first JSON array element via json_extract.
    // Stage / acts_primary: json_extract from properties_json.
    // Folder_path: dirname of note_path (everything before last '/').
    // Confidence_alpha + dominant_link_type + contested: subqueries
    // against note_links per source_path / target_path.
    let tx = conn.transaction().map_err(|e| format!("backfill: tx: {}", e))?;

    let inserted = tx
        .execute(
            "INSERT OR REPLACE INTO sight_v5_layout (
                note_path, stratum, maturity, confidence_alpha, contested,
                library_name, folder_path, created_month, sources_primary,
                stage, acts_primary, dominant_link_type, computed_at
             )
             SELECT
                nm.path,
                CASE
                    WHEN sn.stratum LIKE 'L%' AND length(sn.stratum) >= 2
                        THEN CAST(SUBSTR(sn.stratum, 2) AS INTEGER)
                    WHEN sn.stratum GLOB '[1-8]'
                        THEN CAST(sn.stratum AS INTEGER)
                    ELSE NULL
                END AS stratum,
                sn.maturity,
                NULL AS confidence_alpha,
                CASE WHEN EXISTS (
                    SELECT 1 FROM note_links nl
                    WHERE nl.target_path = nm.path
                      AND nl.link_type = 'contradicts'
                      AND nl.confidence != 'archived'
                ) THEN 1 ELSE 0 END AS contested,
                nm.library_name,
                CASE
                    WHEN instr(nm.path, '/') > 0
                        THEN substr(nm.path, 1, length(nm.path) - length(replace(nm.path, '/', '')) + instr(replace(nm.path, '/', char(0x1f)), char(0x1f)))
                    ELSE NULL
                END AS folder_path,
                CASE
                    WHEN nm.created_at IS NOT NULL
                        THEN CAST(strftime('%m', nm.created_at, 'unixepoch') AS INTEGER) - 1
                    ELSE NULL
                END AS created_month,
                json_extract(nm.sources, '$[0]') AS sources_primary,
                json_extract(nm.properties_json, '$.stage') AS stage,
                json_extract(nm.properties_json, '$.act') AS acts_primary,
                (SELECT nl2.link_type FROM note_links nl2
                 WHERE nl2.source_path = nm.path
                 GROUP BY nl2.link_type
                 ORDER BY COUNT(*) DESC LIMIT 1) AS dominant_link_type,
                strftime('%s', 'now') * 1000 AS computed_at
             FROM note_meta nm
             LEFT JOIN sky_nodes sn ON sn.path = nm.path",
            [],
        )
        .map_err(|e| format!("backfill: bulk insert: {}", e))?;

    // Stamp the sentinel as the LAST statement in the transaction so a
    // mid-backfill interrupt rolls back without leaving a half-stamped
    // sentinel (the same all-or-nothing pattern §B.3 uses).
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES (?1, 1, strftime('%s', 'now'))",
        params![SENTINEL_KEY],
    )
    .map_err(|e| format!("backfill: stamp sentinel: {}", e))?;

    tx.commit().map_err(|e| format!("backfill: commit: {}", e))?;
    Ok(inserted)
}

// ════════════════════════════════════════════════════════════════════
// Tauri command IPCs
// ════════════════════════════════════════════════════════════════════

/// Read the layout cache for the requested scope.
/// scope_kind: "universe" | "library" | "folder"
/// scope_id: library_name (for "library"), folder_path (for "folder"),
///           ignored (for "universe").
#[tauri::command]
pub fn sight_v5_get_layout(
    app: tauri::AppHandle,
    scope_kind: String,
    scope_id: Option<String>,
) -> Result<Vec<LayoutCacheRow>, String> {
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;

    let universe_sql: &'static str =
        "SELECT note_path, stratum, maturity, confidence_alpha, contested,
                library_name, folder_path, created_month, sources_primary,
                stage, acts_primary, dominant_link_type, computed_at
         FROM sight_v5_layout";
    let library_sql: &'static str =
        "SELECT note_path, stratum, maturity, confidence_alpha, contested,
                library_name, folder_path, created_month, sources_primary,
                stage, acts_primary, dominant_link_type, computed_at
         FROM sight_v5_layout
         WHERE library_name = ?1";
    let folder_sql: &'static str =
        "SELECT note_path, stratum, maturity, confidence_alpha, contested,
                library_name, folder_path, created_month, sources_primary,
                stage, acts_primary, dominant_link_type, computed_at
         FROM sight_v5_layout
         WHERE folder_path = ?1 OR folder_path LIKE ?1 || '/%'";

    let (sql, bind_arg): (&'static str, Option<String>) = match scope_kind.as_str() {
        "universe" => (universe_sql, None),
        "library" => (library_sql, scope_id),
        "folder" => (folder_sql, scope_id),
        _ => return Err(format!("unknown scope_kind: {}", scope_kind)),
    };

    let mut stmt = conn.prepare(sql).map_err(|e| format!("prepare: {}", e))?;
    let row_iter = if let Some(arg) = bind_arg {
        stmt.query_map(params![arg], row_to_layout)
    } else {
        stmt.query_map([], row_to_layout)
    }
    .map_err(|e| format!("query: {}", e))?;

    let mut out = Vec::new();
    for r in row_iter {
        out.push(r.map_err(|e| format!("row: {}", e))?);
    }
    Ok(out)
}

fn row_to_layout(row: &rusqlite::Row) -> rusqlite::Result<LayoutCacheRow> {
    let contested: i64 = row.get(4)?;
    Ok(LayoutCacheRow {
        note_path: row.get(0)?,
        stratum: row.get(1)?,
        maturity: row.get(2)?,
        confidence_alpha: row.get(3)?,
        contested: contested != 0,
        library_name: row.get(5)?,
        folder_path: row.get(6)?,
        created_month: row.get(7)?,
        sources_primary: row.get(8)?,
        stage: row.get(9)?,
        acts_primary: row.get(10)?,
        dominant_link_type: row.get(11)?,
        computed_at: row.get(12)?,
    })
}

/// Frontend cache-invalidation probe — returns the current snapshot
/// hash; frontend compares to its stored value and reloads layout if
/// they differ.
#[tauri::command]
pub fn sight_v5_get_universe_snapshot_hash(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
    compute_universe_snapshot_hash(conn)
}

/// Read the typed-link edges between a set of visible notes — used
/// by §5 connector-line rendering. Returns only edges where BOTH
/// endpoints are in the visible set (other edges are off-screen).
#[tauri::command]
pub fn sight_v5_get_link_set_for_notes(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<Vec<LinkEdge>, String> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;

    // Build a comma-separated placeholder list for the IN clause.
    let placeholders = std::iter::repeat("?")
        .take(paths.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT source_path, target_path, link_type, confidence
         FROM note_links
         WHERE source_path IN ({}) AND target_path IN ({})",
        placeholders, placeholders
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {}", e))?;
    // Bind paths twice (once for source IN, once for target IN).
    let mut bindings: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(paths.len() * 2);
    for p in &paths {
        bindings.push(p);
    }
    for p in &paths {
        bindings.push(p);
    }
    let row_iter = stmt
        .query_map(rusqlite::params_from_iter(bindings.iter()), |row| {
            Ok(LinkEdge {
                source_path: row.get(0)?,
                target_path: row.get(1)?,
                link_type: row.get(2)?,
                confidence: row.get(3)?,
            })
        })
        .map_err(|e| format!("query: {}", e))?;

    let mut out = Vec::new();
    for r in row_iter {
        out.push(r.map_err(|e| format!("row: {}", e))?);
    }
    Ok(out)
}

/// Idle-prewarm IPC — fired by the frontend's `requestIdleCallback`
/// after `boot:hydrated`. If the backfill sentinel is missing (fresh
/// install OR new build hitting an existing universe), runs the
/// backfill in the background. Otherwise returns immediately.
#[tauri::command]
pub fn sight_v5_warm_cache(
    app: tauri::AppHandle,
) -> Result<usize, String> {
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    let mut db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_mut().ok_or("Search database not initialized")?;
    backfill_sight_v5_layout(conn)
}

// ════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                library_name TEXT NOT NULL,
                modified INTEGER NOT NULL,
                properties_json TEXT DEFAULT '{}',
                sources TEXT,
                content_type TEXT,
                created_at INTEGER
             );
             CREATE TABLE sky_nodes (
                path TEXT PRIMARY KEY,
                id TEXT,
                stratum TEXT,
                maturity TEXT
             );
             CREATE TABLE note_links (
                source_path TEXT NOT NULL,
                target_path TEXT NOT NULL,
                target_name TEXT NOT NULL,
                link_type TEXT NOT NULL,
                confidence TEXT NOT NULL DEFAULT 'evidence',
                weight REAL NOT NULL DEFAULT 1.0,
                last_traversed TEXT NOT NULL DEFAULT '',
                traversal_count INTEGER NOT NULL DEFAULT 0,
                created TEXT NOT NULL DEFAULT ''
             );
             CREATE TABLE schema_versions (
                module TEXT PRIMARY KEY,
                version INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
             );",
        )
        .expect("setup tables");
        ensure_sight_v5_layout_table(&conn).expect("ensure layout table");
        ensure_sight_v5_invalidation_trigger(&conn).expect("ensure trigger");
        conn
    }

    #[test]
    fn mig024_s2_table_creation_idempotent() {
        let conn = setup_db();
        ensure_sight_v5_layout_table(&conn).expect("second call no-op");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sight_v5_layout'",
                [],
                |r| r.get(0),
            )
            .expect("count tables");
        assert_eq!(count, 1, "table must exist after ensure_*");
    }

    #[test]
    fn mig024_s2_invalidation_trigger_fires_on_note_meta_update() {
        let conn = setup_db();
        // Seed a note + a cache row.
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified)
             VALUES ('a.md', 'a.md', 'lib', 1700000000)",
            [],
        )
        .expect("seed");
        conn.execute(
            "INSERT INTO sight_v5_layout (note_path, computed_at) VALUES ('a.md', 1700000001000)",
            [],
        )
        .expect("seed cache");
        assert_eq!(
            cache_count(&conn, "a.md"),
            1,
            "cache row exists pre-invalidation"
        );

        // Update note_meta — the trigger should DELETE the cache row.
        conn.execute(
            "UPDATE note_meta SET modified = 1700000999 WHERE path = 'a.md'",
            [],
        )
        .expect("update");
        assert_eq!(
            cache_count(&conn, "a.md"),
            0,
            "cache row must be invalidated by note_meta UPDATE trigger"
        );
    }

    #[test]
    fn mig024_s2_invalidation_trigger_fires_on_note_meta_delete() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified)
             VALUES ('a.md', 'a.md', 'lib', 1700000000)",
            [],
        )
        .expect("seed");
        conn.execute(
            "INSERT INTO sight_v5_layout (note_path, computed_at) VALUES ('a.md', 1700000001000)",
            [],
        )
        .expect("seed cache");
        conn.execute("DELETE FROM note_meta WHERE path = 'a.md'", [])
            .expect("delete note");
        assert_eq!(
            cache_count(&conn, "a.md"),
            0,
            "cache row must be invalidated by note_meta DELETE trigger"
        );
    }

    #[test]
    fn mig024_s2_backfill_seeds_existing_notes() {
        let mut conn = setup_db();
        // Seed 3 notes with diverse data.
        conn.execute_batch(
            "INSERT INTO note_meta (path, name, library_name, modified, properties_json, sources, created_at)
             VALUES
                ('research/a.md', 'a.md', 'Research', 1700000000, '{\"stage\":\"growth\"}', '[\"testimony\",\"inference\"]', 1700000000),
                ('research/b.md', 'b.md', 'Research', 1700000100, '{\"stage\":\"maturity\",\"act\":\"synthesis\"}', '[\"perception\"]', 1700100000),
                ('daily/c.md', 'c.md', 'Daily', 1700000200, '{}', NULL, 1700200000);
             INSERT INTO sky_nodes (path, id, stratum, maturity)
             VALUES
                ('research/a.md', 'a.md', 'L4', 'sapling'),
                ('research/b.md', 'b.md', 'L7', 'evergreen'),
                ('daily/c.md', 'c.md', 'L1', 'seed');
             INSERT INTO note_links (source_path, target_path, target_name, link_type, confidence)
             VALUES
                ('research/a.md', 'research/b.md', 'b', 'supports', 'evidence'),
                ('research/a.md', 'daily/c.md', 'c', 'derives-from', 'evidence'),
                ('research/b.md', 'research/a.md', 'a', 'contradicts', 'evidence');",
        )
        .expect("seed");

        let inserted = backfill_sight_v5_layout(&mut conn).expect("backfill");
        assert_eq!(inserted, 3, "should backfill 3 cache rows");

        // Verify a.md has correct extracted values. The test data has
        // b → a as a `contradicts` edge, so a IS contested (it's the
        // target of an inbound contradicts).
        let a: (Option<i64>, Option<String>, Option<String>, Option<String>, i64) = conn
            .query_row(
                "SELECT stratum, maturity, sources_primary, stage, contested
                 FROM sight_v5_layout WHERE note_path = 'research/a.md'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .expect("read a");
        assert_eq!(a.0, Some(4), "a.md stratum parsed from 'L4' to 4");
        assert_eq!(a.1.as_deref(), Some("sapling"));
        assert_eq!(a.2.as_deref(), Some("testimony"), "first source = primary");
        assert_eq!(a.3.as_deref(), Some("growth"));
        assert_eq!(a.4, 1, "a.md is target of b → a contradicts edge → contested = 1");

        // b.md is NOT the target of any contradicts edge.
        let b_contested: i64 = conn
            .query_row(
                "SELECT contested FROM sight_v5_layout WHERE note_path = 'research/b.md'",
                [],
                |r| r.get(0),
            )
            .expect("read b contested");
        assert_eq!(b_contested, 0, "b.md has no inbound contradicts");

        // c.md is also NOT the target of any contradicts edge.
        let c_contested: i64 = conn
            .query_row(
                "SELECT contested FROM sight_v5_layout WHERE note_path = 'daily/c.md'",
                [],
                |r| r.get(0),
            )
            .expect("read c contested");
        assert_eq!(c_contested, 0, "c.md has no inbound contradicts");

        // b.md has acts_primary extracted.
        let b_acts: Option<String> = conn
            .query_row(
                "SELECT acts_primary FROM sight_v5_layout WHERE note_path = 'research/b.md'",
                [],
                |r| r.get(0),
            )
            .expect("read b acts");
        assert_eq!(b_acts.as_deref(), Some("synthesis"));

        // dominant_link_type for a.md (has 2 outgoing: supports + derives-from;
        // tie, ORDER BY ... LIMIT 1 returns one of them).
        let a_dominant: Option<String> = conn
            .query_row(
                "SELECT dominant_link_type FROM sight_v5_layout WHERE note_path = 'research/a.md'",
                [],
                |r| r.get(0),
            )
            .expect("read a dominant");
        assert!(
            a_dominant.is_some(),
            "a.md has outgoing links; dominant_link_type populated"
        );
    }

    #[test]
    fn mig024_s2_backfill_idempotent_via_sentinel() {
        let mut conn = setup_db();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified)
             VALUES ('a.md', 'a.md', 'lib', 1700000000)",
            [],
        )
        .expect("seed");

        let first = backfill_sight_v5_layout(&mut conn).expect("first");
        assert_eq!(first, 1);
        // Second call must skip via sentinel.
        let second = backfill_sight_v5_layout(&mut conn).expect("second");
        assert_eq!(
            second, 1,
            "second call returns existing cache count (sentinel skip)"
        );
        assert_eq!(
            cache_count(&conn, "a.md"),
            1,
            "no duplicate rows from re-run"
        );
    }

    #[test]
    fn mig024_s2_snapshot_hash_changes_on_data_change() {
        let conn = setup_db();
        let h1 = compute_universe_snapshot_hash(&conn).expect("hash 1");
        // Insert a note; hash should change (note_meta count + max(modified)).
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified)
             VALUES ('a.md', 'a.md', 'lib', 1700000000)",
            [],
        )
        .expect("insert");
        let h2 = compute_universe_snapshot_hash(&conn).expect("hash 2");
        assert_ne!(h1, h2, "snapshot hash must change on note_meta INSERT");

        // Update note_meta; hash should change again.
        conn.execute(
            "UPDATE note_meta SET modified = 1700000999 WHERE path = 'a.md'",
            [],
        )
        .expect("update");
        let h3 = compute_universe_snapshot_hash(&conn).expect("hash 3");
        assert_ne!(h2, h3, "snapshot hash must change on note_meta UPDATE");
    }

    fn cache_count(conn: &Connection, note_path: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM sight_v5_layout WHERE note_path = ?",
            [note_path],
            |r| r.get(0),
        )
        .expect("count")
    }
}

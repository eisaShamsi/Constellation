//! MIG-025 §A — Sight v6 layout cache + IPCs (skeleton).
//!
//! Per Concept Paper v4.0 §9.3: the Sight v6 architecture (Coordinated
//! Views — anchor dome + 4 mini-domes + facet sidebar + 7-register chip)
//! reads from a write-time-derived SQLite cache (`sight_v6_layout`) at
//! render time. The §A.4 progressive backfill (Architect Option C3)
//! populates the cache via Tauri events with status-bar progress; until
//! the backfill is wired, this module's surface is just schema setup +
//! the snapshot fingerprint helper.
//!
//! Per CLAUDE.md Performance Rule 8 (write-time derivation) + B2 dual
//! mount strategy: the v6 cache coexists with the v5 cache through
//! Phases 1–3 of the MIG-025 build. v5 cache + triggers stay live;
//! v6 cache + triggers are added alongside. Both invalidation triggers
//! fire on every `note_meta` UPDATE / DELETE — cheap (indexed DELETE
//! against a small key). §D.6 (Phase 4) drops the v5 surface in one
//! atomic migration.
//!
//! Schema additions vs v5 (4 new columns per Architect §1.2):
//!   link_in_count            INTEGER  -- inbound typed-link count
//!   link_out_count           INTEGER  -- outbound typed-link count
//!   frontmatter_key_count    INTEGER  -- key count in YAML frontmatter
//!   body_chars               INTEGER  -- length(content), for empty-body diagnostics
//!
//! All other v5 columns survive verbatim (additive migration). Frontend-
//! derived fields like `topDecileActs`, `provenanceSector`, and
//! `libraryShapeIndex` are computed in JS at render time from the raw
//! cache columns + Universe-wide context (acts distribution, library
//! ordering); they don't need their own cache columns.
//!
//! Sentinel chain isolated from v5: `mig025_sight_v6_layout_backfill_v1`
//! (allocated for §A.3 backfill; not consumed by this skeleton).
//!
//! References:
//!   docs/Constellation-Sight-Concept-Paper-v4.0.md
//!   lab/reports/MIG-025-SIGHT-V6-ARCHITECT.md  (§1.2, §1.3, §4.3)
//!   lab/reports/MIG-025-SIGHT-V6-PLAN.md       (§A.2)

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Per-note cache row returned by `sight_v6_get_layout` (§A.5).
///
/// Serde `rename_all = "camelCase"` aligns JSON output with the
/// TypeScript LayoutCacheRow contract in `src/lib/sight/v6/types.ts`
/// (created in §A.6).
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
    // v6 additions (Architect §1.2):
    pub link_in_count: i64,
    pub link_out_count: i64,
    pub frontmatter_key_count: i64,
    pub body_chars: i64,
}

/// One typed-link edge between two visible notes — read by the anchor
/// dome's connector-line rendering (§A.9). Unchanged from v5; v6's
/// 800-visible auto-fade lives on the frontend per Concept Paper §2.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEdge {
    pub source_path: String,
    pub target_path: String,
    pub link_type: String,
    pub confidence: String,
}

/// Idempotent table + index creation. Called once per `init_db`
/// alongside `ensure_sight_v5_layout_table` per B2 dual-mount.
/// Same covering-index strategy as v5 (Library + Folder are the
/// hot facet-sidebar filters per Concept Paper §2.4 + §3.2).
pub fn ensure_sight_v6_layout_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sight_v6_layout (
            note_path               TEXT PRIMARY KEY,
            stratum                 INTEGER,
            maturity                TEXT,
            confidence_alpha        REAL,
            contested               INTEGER NOT NULL DEFAULT 0,
            library_name            TEXT,
            folder_path             TEXT,
            created_month           INTEGER,
            sources_primary         TEXT,
            stage                   TEXT,
            acts_primary            TEXT,
            dominant_link_type      TEXT,
            computed_at             INTEGER NOT NULL,
            link_in_count           INTEGER NOT NULL DEFAULT 0,
            link_out_count          INTEGER NOT NULL DEFAULT 0,
            frontmatter_key_count   INTEGER NOT NULL DEFAULT 0,
            body_chars              INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_sight_v6_layout_library
            ON sight_v6_layout(library_name);
        CREATE INDEX IF NOT EXISTS idx_sight_v6_layout_folder
            ON sight_v6_layout(folder_path);",
    )
    .map_err(|e| format!("ensure_sight_v6_layout_table: {}", e))
}

/// Cache-invalidation triggers. Coexists with v5's triggers per B2;
/// both fire on every `note_meta` UPDATE / DELETE. Cheap (indexed
/// DELETE on a small key). §D.6 (Phase 4) drops the v5 triggers
/// atomically with the v5 cache table.
pub fn ensure_sight_v6_invalidation_trigger(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS sight_v6_layout_invalidate_au
        AFTER UPDATE ON note_meta
        BEGIN
            DELETE FROM sight_v6_layout WHERE note_path = OLD.path;
        END;
        CREATE TRIGGER IF NOT EXISTS sight_v6_layout_invalidate_ad
        AFTER DELETE ON note_meta
        BEGIN
            DELETE FROM sight_v6_layout WHERE note_path = OLD.path;
        END;",
    )
    .map_err(|e| format!("ensure_sight_v6_invalidation_trigger: {}", e))
}

/// Compute a cheap per-Universe snapshot fingerprint for the v6
/// cache. Same shape as v5's helper but counts `sight_v6_layout`
/// rows so the v6 freshness signal is independent during dual-mount.
/// The frontend compares this against its cached value to detect
/// invalidation.
pub fn compute_universe_snapshot_hash(conn: &Connection) -> Result<String, String> {
    let row: (i64, i64, i64, i64) = conn
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM note_meta),
                (SELECT COALESCE(MAX(modified), 0) FROM note_meta),
                (SELECT COUNT(*) FROM note_links),
                (SELECT COUNT(*) FROM sight_v6_layout)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e| format!("compute_universe_snapshot_hash: {}", e))?;
    Ok(format!("{}-{}-{}-{}", row.0, row.1, row.2, row.3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Helper: build an empty in-memory DB with the minimum tables
    /// sight_v6 cares about (`note_meta` + `note_links`). No data;
    /// just the schema sight_v6 functions need to interact with.
    fn empty_universe_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                modified INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE note_links (
                source_path TEXT NOT NULL,
                target_path TEXT NOT NULL,
                link_type TEXT NOT NULL,
                confidence TEXT NOT NULL DEFAULT 'hypothesis'
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn ensure_sight_v6_layout_table_is_idempotent() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        // Calling twice must not error (CREATE TABLE IF NOT EXISTS).
        ensure_sight_v6_layout_table(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sight_v6_layout'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn ensure_sight_v6_layout_table_creates_covering_indexes() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master \
                 WHERE type='index' AND name LIKE 'idx_sight_v6_layout_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn ensure_sight_v6_layout_table_has_v6_columns() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        // Verify the 4 new v6 columns are present.
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(sight_v6_layout)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for required in [
            "link_in_count",
            "link_out_count",
            "frontmatter_key_count",
            "body_chars",
        ] {
            assert!(
                cols.iter().any(|c| c == required),
                "missing v6 column: {}",
                required
            );
        }
    }

    #[test]
    fn ensure_sight_v6_invalidation_trigger_creates_both_triggers() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        ensure_sight_v6_invalidation_trigger(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master \
                 WHERE type='trigger' AND name LIKE 'sight_v6_layout_invalidate_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn invalidation_trigger_fires_on_note_meta_update() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        ensure_sight_v6_invalidation_trigger(&conn).unwrap();
        // Seed a note_meta row + a fake cache row.
        conn.execute("INSERT INTO note_meta (path, modified) VALUES ('a.md', 100)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO sight_v6_layout (note_path, computed_at) VALUES ('a.md', 0)",
            [],
        )
        .unwrap();
        // UPDATE note_meta → trigger DELETEs the cache row.
        conn.execute(
            "UPDATE note_meta SET modified = 200 WHERE path = 'a.md'",
            [],
        )
        .unwrap();
        let cache_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sight_v6_layout WHERE note_path = 'a.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cache_count, 0);
    }

    #[test]
    fn invalidation_trigger_fires_on_note_meta_delete() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        ensure_sight_v6_invalidation_trigger(&conn).unwrap();
        conn.execute("INSERT INTO note_meta (path, modified) VALUES ('b.md', 100)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO sight_v6_layout (note_path, computed_at) VALUES ('b.md', 0)",
            [],
        )
        .unwrap();
        conn.execute("DELETE FROM note_meta WHERE path = 'b.md'", [])
            .unwrap();
        let cache_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sight_v6_layout WHERE note_path = 'b.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cache_count, 0);
    }

    #[test]
    fn snapshot_hash_returns_four_hyphen_separated_counts() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        let hash = compute_universe_snapshot_hash(&conn).unwrap();
        // 4 hyphen-separated counts: note_meta, max(modified), note_links, sight_v6_layout
        let parts: Vec<&str> = hash.split('-').collect();
        assert_eq!(parts.len(), 4);
        // All zeros on an empty DB.
        assert_eq!(hash, "0-0-0-0");
    }

    #[test]
    fn snapshot_hash_changes_when_note_meta_grows() {
        let conn = empty_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        let h1 = compute_universe_snapshot_hash(&conn).unwrap();
        conn.execute(
            "INSERT INTO note_meta (path, modified) VALUES ('a.md', 100)",
            [],
        )
        .unwrap();
        let h2 = compute_universe_snapshot_hash(&conn).unwrap();
        assert_ne!(h1, h2);
        assert_eq!(h2, "1-100-0-0");
    }

    #[test]
    fn dual_mount_v5_and_v6_caches_coexist() {
        // Both v5 and v6 functions can be called against the same DB
        // without collision. v5's table/triggers are not affected
        // by v6's table/triggers, and vice versa. This is the B2
        // dual-mount invariant from the Architect doc §3 Option B.
        let conn = empty_universe_db();
        // Pretend v5 schema is already there by creating v5-shaped
        // table + AU trigger directly.
        conn.execute_batch(
            "CREATE TABLE sight_v5_layout (
                note_path TEXT PRIMARY KEY,
                computed_at INTEGER NOT NULL DEFAULT 0
            );
            CREATE TRIGGER sight_v5_layout_invalidate_au
            AFTER UPDATE ON note_meta
            BEGIN
                DELETE FROM sight_v5_layout WHERE note_path = OLD.path;
            END;",
        )
        .unwrap();

        // Now add v6 alongside.
        ensure_sight_v6_layout_table(&conn).unwrap();
        ensure_sight_v6_invalidation_trigger(&conn).unwrap();

        // Both tables exist.
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master \
                 WHERE type='table' AND name IN ('sight_v5_layout', 'sight_v6_layout')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 2);

        // INSERT a row into both caches; UPDATE note_meta;
        // both rows get DELETEd (each cache's trigger fires).
        conn.execute("INSERT INTO note_meta (path, modified) VALUES ('c.md', 100)", [])
            .unwrap();
        conn.execute("INSERT INTO sight_v5_layout (note_path) VALUES ('c.md')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO sight_v6_layout (note_path, computed_at) VALUES ('c.md', 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE note_meta SET modified = 200 WHERE path = 'c.md'",
            [],
        )
        .unwrap();

        let v5_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sight_v5_layout", [], |r| r.get(0))
            .unwrap();
        let v6_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sight_v6_layout", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v5_count, 0, "v5 trigger should fire and DELETE the v5 row");
        assert_eq!(v6_count, 0, "v6 trigger should fire and DELETE the v6 row");
    }
}

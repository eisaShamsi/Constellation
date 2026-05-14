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

/// First-boot backfill: populate `sight_v6_layout` for all existing
/// notes. Idempotent via `schema_versions` sentinel
/// `'mig025_sight_v6_layout_backfill_v1'`. Resumable via the
/// INSERT OR REPLACE pattern (re-running over an existing row is a
/// no-op for that row's data; the WHOLE bulk INSERT runs atomically
/// in a transaction so a mid-run interrupt rolls back cleanly).
///
/// This is the **synchronous bulk** backfill skeleton landed in §A.3.
/// §A.4 wraps it with stratum-tiered passes + Tauri progress events
/// per Architect Option C3 (the user-facing flow uses §A.4's
/// progressive variant). This synchronous variant is kept for unit
/// tests + as a fallback / repair operation.
///
/// Mirrors `sight_v5::backfill_sight_v5_layout` (with the same fix-7
/// confidence_alpha aggregation) and adds the 4 new v6 columns:
///   - link_in_count: subquery over note_links.target_path
///   - link_out_count: subquery over note_links.source_path
///   - frontmatter_key_count: COUNT over json_each(properties_json),
///                            null-guarded so empty/missing JSON → 0
///   - body_chars: COALESCE(length(body_text), 0)
///
/// Returns the number of cache rows after the backfill (== note_meta
/// row count after a successful run).
pub fn backfill_sight_v6_layout(conn: &mut Connection) -> Result<usize, String> {
    use rusqlite::params;

    const SENTINEL_KEY: &str = "mig025_sight_v6_layout_backfill_v1";

    // Check sentinel: if v1 already done, return cache count and exit.
    let already_done: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = ?1",
            params![SENTINEL_KEY],
            |r| r.get(0),
        )
        .ok();
    if already_done.is_some() {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sight_v6_layout", [], |r| r.get(0))
            .map_err(|e| format!("backfill: count: {}", e))?;
        return Ok(count as usize);
    }

    let tx = conn
        .transaction()
        .map_err(|e| format!("backfill: tx: {}", e))?;

    // Bulk INSERT OR REPLACE: derives per-note layout from note_meta
    // (sources, stage, acts via json_extract; created_month from
    // epoch via strftime; body_chars from length(body_text);
    // frontmatter_key_count from json_each on properties_json; link
    // in/out counts from note_links) + sky_nodes (stratum, maturity)
    // via LEFT JOIN.
    //
    // confidence_alpha: dominant outgoing-link confidence per note,
    // mapped to alpha (per v5 fix-7 pattern). Notes with no outgoing
    // typed links return NULL → frontend defaults to 0.45 (hypothesis)
    // AND renders hollow per Concept Paper §3.4 fallback.
    let inserted = tx
        .execute(
            "INSERT OR REPLACE INTO sight_v6_layout (
                note_path, stratum, maturity, confidence_alpha, contested,
                library_name, folder_path, created_month, sources_primary,
                stage, acts_primary, dominant_link_type, computed_at,
                link_in_count, link_out_count, frontmatter_key_count, body_chars
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
                (SELECT
                    CASE confidence
                        WHEN 'established' THEN 1.0
                        WHEN 'evidence'    THEN 0.7
                        WHEN 'contested'   THEN 0.85
                        ELSE 0.45
                    END
                 FROM note_links nl3
                 WHERE nl3.source_path = nm.path
                 GROUP BY confidence
                 ORDER BY COUNT(*) DESC
                 LIMIT 1) AS confidence_alpha,
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
                strftime('%s', 'now') * 1000 AS computed_at,
                -- v6 additions (Architect §1.2):
                (SELECT COUNT(*) FROM note_links WHERE target_path = nm.path) AS link_in_count,
                (SELECT COUNT(*) FROM note_links WHERE source_path = nm.path) AS link_out_count,
                CASE
                    WHEN nm.properties_json IS NULL OR nm.properties_json = ''
                        THEN 0
                    ELSE (SELECT COUNT(*) FROM json_each(nm.properties_json))
                END AS frontmatter_key_count,
                COALESCE(length(nm.body_text), 0) AS body_chars
             FROM note_meta nm
             LEFT JOIN sky_nodes sn ON sn.path = nm.path",
            [],
        )
        .map_err(|e| format!("backfill: bulk insert: {}", e))?;

    // Stamp the sentinel as the LAST statement in the transaction so
    // a mid-backfill interrupt rolls back without leaving a half-
    // stamped sentinel (same all-or-nothing pattern as v5).
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES (?1, 1, strftime('%s', 'now'))",
        params![SENTINEL_KEY],
    )
    .map_err(|e| format!("backfill: stamp sentinel: {}", e))?;

    tx.commit()
        .map_err(|e| format!("backfill: commit: {}", e))?;
    Ok(inserted)
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

    // ── §A.3 backfill tests ─────────────────────────────────────────

    /// Helper: build an in-memory DB with the full note_meta + sky_nodes
    /// + schema_versions schema sight_v6 backfill needs. Populates a small
    /// fixture (3 notes, 4 typed links, partial frontmatter) so subqueries
    /// can be verified against deterministic counts.
    fn seeded_universe_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL DEFAULT '',
                library_name TEXT NOT NULL DEFAULT '',
                modified INTEGER NOT NULL DEFAULT 0,
                content_hash TEXT,
                properties_json TEXT DEFAULT '{}',
                tags_json TEXT DEFAULT '[]',
                outgoing_links_json TEXT DEFAULT '[]',
                headings_json TEXT DEFAULT '[]',
                body_text TEXT DEFAULT '',
                word_count INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER,
                cid_cn TEXT NOT NULL DEFAULT '',
                sources TEXT DEFAULT NULL
            );
            CREATE TABLE note_links (
                source_path TEXT NOT NULL,
                target_path TEXT NOT NULL,
                link_type TEXT NOT NULL,
                confidence TEXT NOT NULL DEFAULT 'hypothesis'
            );
            CREATE TABLE sky_nodes (
                path TEXT PRIMARY KEY,
                stratum TEXT,
                maturity TEXT
            );
            CREATE TABLE schema_versions (
                module TEXT PRIMARY KEY,
                version INTEGER NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT 0
            );",
        )
        .unwrap();

        // 3 notes:
        //   a.md — Research library, full frontmatter, 2 outgoing + 1 inbound link
        //   b.md — Research library, minimal frontmatter, 0 outgoing + 1 inbound
        //   c.md — Personal library, no frontmatter, 1 outgoing + 0 inbound
        conn.execute_batch(
            "INSERT INTO note_meta
                (path, name, library_name, modified, properties_json, body_text,
                 created_at, sources)
             VALUES
                ('a.md', 'a', 'Research', 100,
                 '{\"stage\":\"established\",\"act\":\"Synthesis\",\"x\":1,\"y\":2}',
                 'hello world body of note a',
                 1717200000,
                 '[\"https://example.com/source-a\"]'),
                ('b.md', 'b', 'Research', 200,
                 '{\"stage\":\"fresh\"}',
                 '',
                 1717200000,
                 NULL),
                ('c.md', 'c', 'Personal', 300,
                 '{}',
                 'body of c',
                 1717200000,
                 NULL);

             INSERT INTO note_links
                (source_path, target_path, link_type, confidence)
             VALUES
                ('a.md', 'b.md', 'supports',    'evidence'),
                ('a.md', 'c.md', 'causes',      'established'),
                ('c.md', 'a.md', 'derives-from', 'hypothesis'),
                ('a.md', 'b.md', 'contradicts', 'evidence');

             INSERT INTO sky_nodes (path, stratum, maturity) VALUES
                ('a.md', 'L3', 'evergreen'),
                ('b.md', 'L1', 'seed'),
                ('c.md', NULL, NULL);",
        )
        .unwrap();

        conn
    }

    #[test]
    fn backfill_writes_all_rows() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        ensure_sight_v6_invalidation_trigger(&conn).unwrap();

        let inserted = backfill_sight_v6_layout(&mut conn).unwrap();
        assert_eq!(inserted, 3, "all 3 fixture notes backfilled");

        let cache_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sight_v6_layout", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cache_count, 3);
    }

    #[test]
    fn backfill_populates_link_in_and_out_counts() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        backfill_sight_v6_layout(&mut conn).unwrap();

        // a.md: 3 outgoing (a→b supports, a→c causes, a→b contradicts), 1 inbound (c→a)
        // b.md: 0 outgoing, 2 inbound (a→b twice — supports + contradicts)
        // c.md: 1 outgoing (c→a), 1 inbound (a→c)
        let (a_out, a_in): (i64, i64) = conn
            .query_row(
                "SELECT link_out_count, link_in_count FROM sight_v6_layout WHERE note_path = 'a.md'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(a_out, 3);
        assert_eq!(a_in, 1);

        let (b_out, b_in): (i64, i64) = conn
            .query_row(
                "SELECT link_out_count, link_in_count FROM sight_v6_layout WHERE note_path = 'b.md'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(b_out, 0);
        assert_eq!(b_in, 2);
    }

    #[test]
    fn backfill_populates_frontmatter_key_count() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        backfill_sight_v6_layout(&mut conn).unwrap();

        // a.md: {stage, act, x, y} = 4 keys
        // b.md: {stage} = 1 key
        // c.md: {} = 0 keys
        let a_keys: i64 = conn
            .query_row(
                "SELECT frontmatter_key_count FROM sight_v6_layout WHERE note_path = 'a.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a_keys, 4);

        let b_keys: i64 = conn
            .query_row(
                "SELECT frontmatter_key_count FROM sight_v6_layout WHERE note_path = 'b.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(b_keys, 1);

        let c_keys: i64 = conn
            .query_row(
                "SELECT frontmatter_key_count FROM sight_v6_layout WHERE note_path = 'c.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(c_keys, 0);
    }

    #[test]
    fn backfill_populates_body_chars() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        backfill_sight_v6_layout(&mut conn).unwrap();

        let a_chars: i64 = conn
            .query_row(
                "SELECT body_chars FROM sight_v6_layout WHERE note_path = 'a.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a_chars, "hello world body of note a".len() as i64);

        let b_chars: i64 = conn
            .query_row(
                "SELECT body_chars FROM sight_v6_layout WHERE note_path = 'b.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(b_chars, 0); // empty body
    }

    #[test]
    fn backfill_stamps_sentinel_v1() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        backfill_sight_v6_layout(&mut conn).unwrap();

        let version: i64 = conn
            .query_row(
                "SELECT version FROM schema_versions
                 WHERE module = 'mig025_sight_v6_layout_backfill_v1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn backfill_is_idempotent_via_sentinel() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        // First call: full backfill, returns 3.
        let first = backfill_sight_v6_layout(&mut conn).unwrap();
        assert_eq!(first, 3);

        // Second call: sentinel set, should short-circuit and return current
        // cache count (still 3) WITHOUT re-running the bulk INSERT.
        let second = backfill_sight_v6_layout(&mut conn).unwrap();
        assert_eq!(second, 3);

        // Cache still has 3 rows.
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sight_v6_layout", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn backfill_resolves_stratum_from_sky_nodes_l_prefix() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        backfill_sight_v6_layout(&mut conn).unwrap();

        // a.md → 'L3' → 3
        // b.md → 'L1' → 1
        // c.md → NULL (no sky_nodes entry that the LEFT JOIN finds)
        let a_stratum: Option<i64> = conn
            .query_row(
                "SELECT stratum FROM sight_v6_layout WHERE note_path = 'a.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a_stratum, Some(3));

        let b_stratum: Option<i64> = conn
            .query_row(
                "SELECT stratum FROM sight_v6_layout WHERE note_path = 'b.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(b_stratum, Some(1));

        let c_stratum: Option<i64> = conn
            .query_row(
                "SELECT stratum FROM sight_v6_layout WHERE note_path = 'c.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(c_stratum, None);
    }

    #[test]
    fn backfill_marks_contested_when_inbound_contradicts() {
        let mut conn = seeded_universe_db();
        ensure_sight_v6_layout_table(&conn).unwrap();
        backfill_sight_v6_layout(&mut conn).unwrap();

        // b.md has an inbound 'contradicts' from a.md → contested = 1
        // a.md has no inbound contradicts → contested = 0
        let b_contested: i64 = conn
            .query_row(
                "SELECT contested FROM sight_v6_layout WHERE note_path = 'b.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(b_contested, 1);

        let a_contested: i64 = conn
            .query_row(
                "SELECT contested FROM sight_v6_layout WHERE note_path = 'a.md'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a_contested, 0);
    }
}

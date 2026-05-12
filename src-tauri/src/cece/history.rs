//! MIG-022 §B — Note state history.
//!
//! Persists every change to a note's epistemic frontmatter fields so
//! the user can answer questions like "show me where my certainty
//! dropped in the last 6 months" and "show the evolution of my stance
//! on this question". The on-disk `.md` file remains the source of
//! truth (CLAUDE.md "File over app"); this table is the temporal
//! index — maintained at write-time via a SQLite trigger on
//! `note_meta` per CLAUDE.md Performance Rule 8 (write-time
//! derivation).
//!
//! Design (per the Plan + WA #5 cross-check refinement):
//!
//! - **Single JSON-diff column shape** (`changes_json TEXT`) rather
//!   than `(axis_changed, old_value, new_value)` triples. When future
//!   epistemic fields are added (e.g. warrant_chain, ḥadīth_grade),
//!   the trigger does NOT need rewriting — the JSON object grows.
//! - **Trigger guard** (`WHEN OLD.field IS NOT NEW.field`) prevents
//!   the trigger from firing on no-op writes. The canonical SQLite
//!   footgun the cross-check warned about.
//! - **Cascade on delete** (`ON DELETE CASCADE`) for MIG-022; matches
//!   today's hard-delete semantics on `note_meta`. A future MIG could
//!   move to soft-delete (PJ candidate) for delete-recovery; the
//!   trigger + table shape would not change.
//!
//! Watched fields:
//!   - `note_meta.sources`         (MIG-021 §1A column)
//!   - `note_meta.content_type`    (MIG-021 §1A column)
//!   - `note_meta.properties_json` (the YAML blob; catches §A
//!     epistemic fields like held_by/domain/function/warrant/ikhtilāf
//!     by inclusion in the JSON; finer per-field diffing is a query-
//!     time concern via json_extract)
//!
//! §B.1 ships the table + index. §B.2 ships the trigger.
//! §B.3 backfills `created` events for existing notes.
//! §B.4 ships the query-API IPC.
//! §B.5 ships the Sight v3 overlay UI per D-B4.β.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Idempotent table + index creation. Called once per `init_db` after
/// the sources subsystem schema is in place (so the foreign-key
/// reference to `note_meta(path)` resolves cleanly).
///
/// The `path` foreign key matches `note_meta.path`'s TEXT PRIMARY KEY
/// shape. CASCADE on delete keeps the history aligned with the
/// note_meta lifecycle.
///
/// `captured_at` is Unix epoch milliseconds (matching the
/// `cataloger_reliability.json` timestamp convention from V3-§9.C).
/// `changes_json` is a JSON object whose keys are watched-field names
/// and whose values are `{ "old": ..., "new": ... }` sub-objects;
/// fields that didn't change are absent from the object (the trigger
/// uses `CASE WHEN ... THEN ... ELSE NULL END` and `json_object`
/// natively skips NULL values).
///
/// The covering index `(note_path, captured_at DESC)` matches the
/// dominant access pattern: "show this note's history in chronological
/// order" — a single index range scan, no sort.
pub fn ensure_note_state_history_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS note_state_history (
            history_id   INTEGER PRIMARY KEY AUTOINCREMENT,
            note_path    TEXT NOT NULL,
            captured_at  INTEGER NOT NULL,
            changes_json TEXT NOT NULL,
            FOREIGN KEY (note_path) REFERENCES note_meta(path) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_note_state_history_note_time
            ON note_state_history(note_path, captured_at DESC);
        ",
    )?;
    Ok(())
}

/// MIG-022 §B.2 — write-time trigger on note_meta UPDATE.
///
/// Fires AFTER any UPDATE on note_meta, but the WHEN guard skips
/// no-op writes (where none of the three watched fields actually
/// changed). The canonical SQLite footgun the cross-check warned
/// about — without this guard, every typo-fix save on a note's
/// body would fire the trigger.
///
/// Watched columns:
///   - `sources` (MIG-021 §1A)
///   - `content_type` (MIG-021v2 §1A')
///   - `properties_json` (the YAML blob; catches §A epistemic
///     fields like held_by/domain/function/warrant/ikhtilāf by
///     inclusion in the JSON)
///
/// On fire, captures both old and new values for EACH changed
/// field into a single JSON object stored in `changes_json`.
/// Fields that didn't change are recorded as `null` in the JSON
/// (storage cost is trivial — ~30 bytes — and query-time
/// filtering via `json_extract IS NOT NULL` is fast).
///
/// `captured_at` is `strftime('%s','now') * 1000` for milliseconds-
/// since-epoch, matching the cataloger_reliability.json convention.
///
/// **Intentionally idempotent** via `CREATE TRIGGER IF NOT EXISTS`.
/// To replace the trigger (if the watched-field set evolves), drop
/// it first via the `drop_note_state_history_trigger` helper below.
pub fn ensure_note_state_history_trigger(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TRIGGER IF NOT EXISTS note_state_history_au
        AFTER UPDATE ON note_meta
        WHEN OLD.sources IS NOT NEW.sources
          OR OLD.content_type IS NOT NEW.content_type
          OR OLD.properties_json IS NOT NEW.properties_json
        BEGIN
            INSERT INTO note_state_history (note_path, captured_at, changes_json)
            VALUES (
                NEW.path,
                CAST(strftime('%s', 'now') AS INTEGER) * 1000,
                json_object(
                    'sources', CASE WHEN OLD.sources IS NOT NEW.sources
                        THEN json_object('old', OLD.sources, 'new', NEW.sources)
                        ELSE NULL END,
                    'content_type', CASE WHEN OLD.content_type IS NOT NEW.content_type
                        THEN json_object('old', OLD.content_type, 'new', NEW.content_type)
                        ELSE NULL END,
                    'properties_json', CASE WHEN OLD.properties_json IS NOT NEW.properties_json
                        THEN json_object('old', OLD.properties_json, 'new', NEW.properties_json)
                        ELSE NULL END
                )
            );
        END;
        ",
    )?;
    Ok(())
}

/// Convenience helper for the §B.3 backfill protocol: callers drop
/// the trigger before doing a bulk INSERT (so each row doesn't fire
/// the trigger 7,600 times on Eisa's primary universe), then
/// re-attach via `ensure_note_state_history_trigger` after the
/// backfill commits. The cross-check warned this is the canonical
/// SQLite bulk-update footgun.
pub fn drop_note_state_history_trigger(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("DROP TRIGGER IF EXISTS note_state_history_au;")?;
    Ok(())
}

/// MIG-022 §B.3 — First-boot backfill: seed an initial-state history
/// row for every existing note that has any epistemic data set.
///
/// Idempotent via the `schema_versions` sentinel — once stamped, the
/// backfill skips on every subsequent boot. Per CLAUDE.md SO #6 the
/// backfill is **resumable**: if interrupted mid-flight, the next
/// boot finds the sentinel un-stamped and re-runs cleanly (the bulk
/// INSERT is wrapped in a transaction; partial writes are rolled
/// back).
///
/// Protocol (per the WA #5 cross-check refinement):
///   1. Check sentinel — skip + return Ok(0) if already stamped
///   2. BEGIN IMMEDIATE — exclusive lock for the duration
///   3. DROP TRIGGER note_state_history_au — so the bulk INSERT
///      doesn't fire the trigger 7,600 times on Eisa's primary
///      universe (the canonical SQLite footgun)
///   4. Bulk INSERT one row per existing note_meta row whose
///      epistemic fields are non-trivially set (sources, content_type,
///      OR properties_json != '{}'). Empty notes don't get seed
///      events — their first UPDATE creates the first history row
///      naturally
///   5. Re-attach trigger via ensure_note_state_history_trigger
///   6. Stamp `schema_versions.note_state_history_backfill = 1`
///   7. COMMIT (or rollback on any error)
///
/// `captured_at` for seed events: the note's `modified` timestamp
/// (in milliseconds), representing "this is the state at the file's
/// last save." A pure-history view would require git log or similar
/// — out of scope; the seed marks "this is where the timeline starts
/// for this note."
///
/// `changes_json` for seed events uses the special `'_seed'` key
/// (distinct from the `'sources'`/`'content_type'`/`'properties_json'`
/// keys the UPDATE trigger emits) so query-time consumers can
/// distinguish "initial state snapshot" from "field change":
///   `{"_seed": {"sources": "testimony", "content_type": "fact", "properties_json": "{...}"}}`
///
/// Returns the count of rows seeded (or 0 if backfill was skipped).
pub fn backfill_initial_history(conn: &mut Connection) -> rusqlite::Result<usize> {
    // Check sentinel — skip if already done.
    let stored_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions
             WHERE module = 'note_state_history_backfill'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stored_version >= 1 {
        return Ok(0);
    }

    // BEGIN IMMEDIATE for exclusive write-lock during the backfill.
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    // DROP TRIGGER so each seed row doesn't re-fire it (the trigger
    // wouldn't actually fire here anyway since we're INSERTing into
    // note_state_history directly, not UPDATE'ing note_meta — but
    // dropping it is the canonical protocol per the cross-check + a
    // belt-and-braces guard against future trigger additions).
    tx.execute_batch("DROP TRIGGER IF EXISTS note_state_history_au;")?;

    // Single bulk INSERT seeds one row per existing note that has
    // any non-trivial epistemic data. The WHERE filter avoids
    // seeding empty notes.
    //
    // captured_at: note_meta.modified is seconds-since-epoch (per the
    // existing schema); convert to milliseconds via `* 1000`.
    //
    // changes_json: uses '_seed' key (one underscore prefix) to
    // distinguish from the UPDATE trigger's per-field keys. Stores
    // the snapshot of the three watched fields (each may be NULL).
    let rows_seeded = tx.execute(
        "INSERT INTO note_state_history (note_path, captured_at, changes_json)
         SELECT
             path,
             modified * 1000,
             json_object('_seed', json_object(
                 'sources', sources,
                 'content_type', content_type,
                 'properties_json', properties_json
             ))
         FROM note_meta
         WHERE sources IS NOT NULL
            OR content_type IS NOT NULL
            OR (properties_json IS NOT NULL AND properties_json != '{}')",
        [],
    )?;

    // Re-attach the trigger (same SQL as ensure_note_state_history_trigger;
    // inlined here so we stay inside the transaction).
    tx.execute_batch(
        "
        CREATE TRIGGER IF NOT EXISTS note_state_history_au
        AFTER UPDATE ON note_meta
        WHEN OLD.sources IS NOT NEW.sources
          OR OLD.content_type IS NOT NEW.content_type
          OR OLD.properties_json IS NOT NEW.properties_json
        BEGIN
            INSERT INTO note_state_history (note_path, captured_at, changes_json)
            VALUES (
                NEW.path,
                CAST(strftime('%s', 'now') AS INTEGER) * 1000,
                json_object(
                    'sources', CASE WHEN OLD.sources IS NOT NEW.sources
                        THEN json_object('old', OLD.sources, 'new', NEW.sources)
                        ELSE NULL END,
                    'content_type', CASE WHEN OLD.content_type IS NOT NEW.content_type
                        THEN json_object('old', OLD.content_type, 'new', NEW.content_type)
                        ELSE NULL END,
                    'properties_json', CASE WHEN OLD.properties_json IS NOT NEW.properties_json
                        THEN json_object('old', OLD.properties_json, 'new', NEW.properties_json)
                        ELSE NULL END
                )
            );
        END;
        ",
    )?;

    // Stamp the sentinel so subsequent boots skip.
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('note_state_history_backfill', 1, strftime('%s','now'))",
        [],
    )?;

    tx.commit()?;
    Ok(rows_seeded)
}

// ─── §B.4 query API ──────────────────────────────────────────────────────

/// One row of the note_state_history table, as returned to the frontend.
/// `changes_json` is the raw JSON string; the frontend parses + walks
/// the diff structure (different keys per event source — `'_seed'` for
/// backfill rows, per-field keys for trigger rows).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub history_id: i64,
    pub note_path: String,
    pub captured_at: i64,
    pub changes_json: String,
}

/// Filter for the cross-note query IPC. All fields optional; the
/// resulting query is the conjunction of the supplied filters.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryFilter {
    /// Substring match on JSON keys (e.g. "sources" matches every row
    /// where `changes_json` contains the literal `"sources"` key).
    /// Future polish: proper JSON path matching via json_extract.
    pub axis: Option<String>,
    /// captured_at >= since (milliseconds since epoch).
    pub since: Option<i64>,
    /// captured_at <= until (milliseconds since epoch).
    pub until: Option<i64>,
    /// Filter to a specific library_name via JOIN with note_meta.
    pub library_name: Option<String>,
    /// Caps result-set size; defaults to 1000 if not specified.
    pub limit: Option<usize>,
}

/// MIG-022 §B.4 — read all history events for a single note in
/// chronological order (most recent first; reverse-chronological
/// matches the dominant UX read pattern of "show me recent changes").
///
/// Uses the covering index `idx_note_state_history_note_time` for
/// the lookup — no full table scan, no sort. Bounded by the note's
/// individual row count (typically O(10) per note).
#[tauri::command]
pub fn cece_get_note_history(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<Vec<HistoryEvent>, String> {
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
    let mut stmt = conn
        .prepare(
            "SELECT history_id, note_path, captured_at, changes_json
             FROM note_state_history
             WHERE note_path = ?
             ORDER BY captured_at DESC",
        )
        .map_err(|e| format!("prepare failed: {}", e))?;
    let rows = stmt
        .query_map([note_path], |row| {
            Ok(HistoryEvent {
                history_id: row.get(0)?,
                note_path: row.get(1)?,
                captured_at: row.get(2)?,
                changes_json: row.get(3)?,
            })
        })
        .map_err(|e| format!("query failed: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("row collect failed: {}", e))
}

/// MIG-022 §B.4 — cross-note query with optional filters. Useful for
/// gap-analysis §6.3-style queries: "show me all notes where my
/// `sources` changed in the last 6 months", "show all changes in
/// library X since date Y", etc.
///
/// Implementation note: filter values are concatenated into the SQL
/// dynamically (`format!()`) for the WHERE clauses BUT the actual
/// values are bound via parameterized `?` placeholders to prevent
/// SQL injection. The `axis` filter uses `LIKE '%key%'` substring
/// match on the changes_json text — coarse but sufficient for the
/// gap-analysis query patterns. Future polish: json_extract for
/// precise key matching + json_each for iteration.
#[tauri::command]
pub fn cece_query_history(
    app: tauri::AppHandle,
    filter: HistoryFilter,
) -> Result<Vec<HistoryEvent>, String> {
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;

    let mut sql = String::from(
        "SELECT h.history_id, h.note_path, h.captured_at, h.changes_json
         FROM note_state_history h",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut clauses: Vec<String> = Vec::new();

    if filter.library_name.is_some() {
        sql.push_str(" JOIN note_meta n ON n.path = h.note_path");
    }
    if let Some(axis) = &filter.axis {
        // Substring match on the JSON text. e.g. axis="sources" matches
        // any row whose changes_json contains the literal "sources" key.
        clauses.push("h.changes_json LIKE ?".to_string());
        params.push(Box::new(format!("%\"{}\"%", axis.replace('%', "\\%"))));
    }
    if let Some(since) = filter.since {
        clauses.push("h.captured_at >= ?".to_string());
        params.push(Box::new(since));
    }
    if let Some(until) = filter.until {
        clauses.push("h.captured_at <= ?".to_string());
        params.push(Box::new(until));
    }
    if let Some(library) = &filter.library_name {
        clauses.push("n.library_name = ?".to_string());
        params.push(Box::new(library.clone()));
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY h.captured_at DESC LIMIT ?");
    let limit = filter.limit.unwrap_or(1000) as i64;
    params.push(Box::new(limit));

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare failed: {}", e))?;
    let param_refs: Vec<&dyn rusqlite::ToSql> =
        params.iter().map(|p| p.as_ref() as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(HistoryEvent {
                history_id: row.get(0)?,
                note_path: row.get(1)?,
                captured_at: row.get(2)?,
                changes_json: row.get(3)?,
            })
        })
        .map_err(|e| format!("query failed: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("row collect failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Helper: spin up an in-memory DB with the minimum note_meta
    /// schema needed for the foreign-key reference, plus the history
    /// table. Returns the connection ready for assertions.
    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        // Foreign keys are off by default in rusqlite; enable for the test.
        conn.execute_batch("PRAGMA foreign_keys = ON;").expect("enable fk");
        // Minimal note_meta — only the fields the foreign key needs.
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                library_name TEXT NOT NULL,
                modified INTEGER NOT NULL,
                sources TEXT,
                content_type TEXT,
                properties_json TEXT DEFAULT '{}'
            );",
        ).expect("create note_meta");
        ensure_note_state_history_table(&conn).expect("ensure history table");
        conn
    }

    /// Variant of `setup_db` that ALSO installs the trigger. Used by
    /// the §B.2 trigger tests below.
    fn setup_db_with_trigger() -> Connection {
        let conn = setup_db();
        ensure_note_state_history_trigger(&conn).expect("ensure history trigger");
        conn
    }

    fn count_history(conn: &Connection, note_path: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM note_state_history WHERE note_path = ?",
            [note_path],
            |row| row.get(0),
        ).expect("count history")
    }

    #[test]
    fn mig022_b1_table_creation_idempotent() {
        let conn = setup_db();
        // Calling twice should not error (CREATE TABLE IF NOT EXISTS).
        ensure_note_state_history_table(&conn).expect("second call should be no-op");

        // Verify the table exists with the expected columns.
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='note_state_history'")
            .expect("prepare table check");
        let count: i64 = stmt
            .query_row([], |row| row.get::<_, String>(0).map(|_| 1))
            .expect("table should exist");
        assert_eq!(count, 1, "note_state_history table missing after ensure_*");
    }

    #[test]
    fn mig022_b1_index_exists() {
        let conn = setup_db();
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type='index' AND name='idx_note_state_history_note_time'",
            )
            .expect("prepare index check");
        let count: i64 = stmt
            .query_row([], |row| row.get::<_, String>(0).map(|_| 1))
            .expect("covering index should exist");
        assert_eq!(count, 1, "covering index missing after ensure_*");
    }

    #[test]
    fn mig022_b1_insert_then_select_round_trip() {
        let conn = setup_db();
        // Seed a note_meta row first so the foreign key resolves.
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified)
             VALUES ('test/note.md', 'note.md', 'test-lib', 1700000000)",
            [],
        ).expect("seed note_meta");
        // Insert a history event manually (the trigger lands in §B.2;
        // this test just exercises the schema shape).
        conn.execute(
            "INSERT INTO note_state_history (note_path, captured_at, changes_json)
             VALUES ('test/note.md', 1700000001000, '{\"sources\":{\"old\":null,\"new\":\"testimony\"}}')",
            [],
        ).expect("insert history event");
        // Read back via the covering-index access pattern.
        let mut stmt = conn
            .prepare(
                "SELECT captured_at, changes_json FROM note_state_history
                 WHERE note_path = ? ORDER BY captured_at DESC",
            )
            .expect("prepare select");
        let rows: Vec<(i64, String)> = stmt
            .query_map(["test/note.md"], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("collect rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 1700000001000);
        assert!(rows[0].1.contains("\"sources\""));
    }

    #[test]
    fn mig022_b1_cascade_on_delete() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified)
             VALUES ('a.md', 'a.md', 'lib', 1700000000)",
            [],
        ).expect("seed");
        conn.execute(
            "INSERT INTO note_state_history (note_path, captured_at, changes_json)
             VALUES ('a.md', 1700000001000, '{}')",
            [],
        ).expect("history seed");
        // Delete the note — history rows should cascade away.
        conn.execute("DELETE FROM note_meta WHERE path = 'a.md'", [])
            .expect("delete note_meta");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM note_state_history WHERE note_path = 'a.md'",
                [],
                |row| row.get(0),
            )
            .expect("count history");
        assert_eq!(count, 0, "history rows must cascade on note_meta delete");
    }

    // ─── §B.2 trigger tests ──────────────────────────────────────────────

    #[test]
    fn mig022_b2_trigger_fires_on_sources_change() {
        let conn = setup_db_with_trigger();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources)
             VALUES ('a.md', 'a.md', 'lib', 1700000000, 'testimony')",
            [],
        ).expect("seed");
        // Update sources only.
        conn.execute(
            "UPDATE note_meta SET sources = 'inference' WHERE path = 'a.md'",
            [],
        ).expect("update");
        assert_eq!(count_history(&conn, "a.md"), 1, "trigger should fire on sources change");
        // Verify the changes_json captured both old and new.
        let changes: String = conn.query_row(
            "SELECT changes_json FROM note_state_history WHERE note_path = 'a.md'",
            [],
            |row| row.get(0),
        ).expect("read changes_json");
        assert!(changes.contains("\"sources\""), "changes_json must include sources key: {}", changes);
        assert!(changes.contains("\"old\":\"testimony\""), "old value missing: {}", changes);
        assert!(changes.contains("\"new\":\"inference\""), "new value missing: {}", changes);
    }

    #[test]
    fn mig022_b2_trigger_skips_noop_update() {
        let conn = setup_db_with_trigger();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources)
             VALUES ('a.md', 'a.md', 'lib', 1700000000, 'testimony')",
            [],
        ).expect("seed");
        // Update a non-watched field (modified). Trigger must NOT fire.
        conn.execute(
            "UPDATE note_meta SET modified = 1700000999 WHERE path = 'a.md'",
            [],
        ).expect("update modified only");
        assert_eq!(
            count_history(&conn, "a.md"), 0,
            "trigger must NOT fire when no watched field changed (canonical SQLite footgun)"
        );
        // Update sources to the SAME value. Trigger must NOT fire.
        conn.execute(
            "UPDATE note_meta SET sources = 'testimony' WHERE path = 'a.md'",
            [],
        ).expect("noop update sources");
        assert_eq!(
            count_history(&conn, "a.md"), 0,
            "trigger must NOT fire when watched field's value didn't actually change"
        );
    }

    #[test]
    fn mig022_b2_trigger_captures_multi_field_change() {
        let conn = setup_db_with_trigger();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources, content_type, properties_json)
             VALUES ('a.md', 'a.md', 'lib', 1700000000, 'testimony', 'fact', '{\"old\":1}')",
            [],
        ).expect("seed");
        // Change all three watched fields in a single UPDATE — should
        // produce exactly one history row with all three diffs.
        conn.execute(
            "UPDATE note_meta SET
                sources = 'inference',
                content_type = 'concept',
                properties_json = '{\"new\":2}'
             WHERE path = 'a.md'",
            [],
        ).expect("multi-field update");
        assert_eq!(count_history(&conn, "a.md"), 1, "single UPDATE → single history row");
        let changes: String = conn.query_row(
            "SELECT changes_json FROM note_state_history WHERE note_path = 'a.md'",
            [],
            |row| row.get(0),
        ).expect("read changes_json");
        assert!(changes.contains("\"sources\""), "sources diff captured");
        assert!(changes.contains("\"content_type\""), "content_type diff captured");
        assert!(changes.contains("\"properties_json\""), "properties_json diff captured");
    }

    // ─── §B.3 backfill tests ─────────────────────────────────────────────

    /// Setup variant that includes the `schema_versions` table the
    /// backfill uses for its idempotency sentinel.
    fn setup_db_with_trigger_and_sentinel() -> Connection {
        let conn = setup_db_with_trigger();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions (
                module TEXT PRIMARY KEY,
                version INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        ).expect("create schema_versions");
        conn
    }

    #[test]
    fn mig022_b3_backfill_seeds_existing_notes() {
        let mut conn = setup_db_with_trigger_and_sentinel();
        // Seed 3 notes: two with epistemic data, one empty.
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources, content_type, properties_json)
             VALUES
                ('a.md', 'a.md', 'lib', 1700000000, 'testimony', 'fact', '{\"held_by\":\"al-Shāfiʿī\"}'),
                ('b.md', 'b.md', 'lib', 1700000100, 'inference', NULL, '{}'),
                ('c.md', 'c.md', 'lib', 1700000200, NULL, NULL, '{}')",
            [],
        ).expect("seed notes");

        let count = backfill_initial_history(&mut conn).expect("backfill should succeed");
        assert_eq!(count, 2, "should seed only the 2 notes with epistemic data (skip c.md)");

        // Verify the seed shape uses the '_seed' key.
        let changes: String = conn.query_row(
            "SELECT changes_json FROM note_state_history WHERE note_path = 'a.md'",
            [],
            |row| row.get(0),
        ).expect("read a.md history");
        assert!(changes.contains("\"_seed\""), "seed events use _seed key: {}", changes);
        assert!(changes.contains("\"testimony\""), "sources captured: {}", changes);
        // properties_json is stringified-inside-JSON so its inner
        // quotes get escaped to `\"...\"`. Loose substring check on
        // the value's text content is sufficient.
        assert!(changes.contains("al-Shāfiʿī"), "properties_json captured: {}", changes);

        // Verify captured_at uses the note's modified timestamp × 1000.
        let captured_at: i64 = conn.query_row(
            "SELECT captured_at FROM note_state_history WHERE note_path = 'a.md'",
            [],
            |row| row.get(0),
        ).expect("read a.md captured_at");
        assert_eq!(captured_at, 1700000000 * 1000, "captured_at should be modified * 1000");

        // c.md (empty) should NOT have a history row.
        assert_eq!(count_history(&conn, "c.md"), 0, "empty notes don't get seed events");
    }

    #[test]
    fn mig022_b3_backfill_idempotent() {
        let mut conn = setup_db_with_trigger_and_sentinel();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources)
             VALUES ('a.md', 'a.md', 'lib', 1700000000, 'testimony')",
            [],
        ).expect("seed");
        let first = backfill_initial_history(&mut conn).expect("first run");
        assert_eq!(first, 1);
        // Second call should skip via sentinel.
        let second = backfill_initial_history(&mut conn).expect("second run should not error");
        assert_eq!(second, 0, "sentinel should skip subsequent backfill calls");
        // Verify only one history row total (no duplicates).
        assert_eq!(count_history(&conn, "a.md"), 1);
    }

    #[test]
    fn mig022_b3_backfill_preserves_trigger_after_run() {
        let mut conn = setup_db_with_trigger_and_sentinel();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources)
             VALUES ('a.md', 'a.md', 'lib', 1700000000, 'testimony')",
            [],
        ).expect("seed");
        backfill_initial_history(&mut conn).expect("backfill");
        // Now an UPDATE on note_meta should fire the trigger.
        conn.execute(
            "UPDATE note_meta SET sources = 'inference' WHERE path = 'a.md'",
            [],
        ).expect("update");
        // Should now have 2 history rows: one from backfill (_seed) +
        // one from trigger (sources diff).
        assert_eq!(count_history(&conn, "a.md"), 2, "trigger must work after backfill");
    }

    #[test]
    fn mig022_b2_drop_trigger_helper() {
        let conn = setup_db_with_trigger();
        // Drop and re-add — both should be idempotent.
        drop_note_state_history_trigger(&conn).expect("drop should not error");
        drop_note_state_history_trigger(&conn).expect("drop on missing trigger should not error");
        // Re-add via the public API.
        ensure_note_state_history_trigger(&conn).expect("re-attach should not error");
        // Verify trigger is back: an UPDATE should fire it.
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, sources)
             VALUES ('a.md', 'a.md', 'lib', 1700000000, 'old')",
            [],
        ).expect("seed");
        conn.execute(
            "UPDATE note_meta SET sources = 'new' WHERE path = 'a.md'",
            [],
        ).expect("update");
        assert_eq!(count_history(&conn, "a.md"), 1, "trigger should fire after re-attach");
    }
}

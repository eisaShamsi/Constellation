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

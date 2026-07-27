//! MIG-101 Phase A — note SHAPE, and the reversibility that has to exist first.
//!
//! **Concept (the horse):** *a note's shape is a claim the note is making about
//! itself.* Shape governs how a note is PRESENTED and TEMPLATED — never what it
//! CONTAINS. That single constraint is what makes every shape change reversible
//! by construction rather than by careful coding, and reversibility is what
//! decides whether automatic container graduation is permissible at all
//! (`docs/MIG-101-Shape-Graduation-Quiet-Signal-Plan.md` §3 Phase A).
//!
//! The evidence for building revert FIRST is the Excel gene-name case: an
//! inference that was automatic, silent, destructive in place, and irreversible
//! after save corrupted >30% of a literature by 2020, and the vendor's eventual
//! fix was not a smarter classifier — it was showing a notification before the
//! conversion. Constellation's shape change must fail all four of those
//! properties, and this module is where three of them are failed.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Bumping this makes [`is_stamped`] report false until the table is rebuilt —
/// the same rollback/self-heal shape as `REVIEW_SCHEMA_VERSION`.
///
/// **v2 (§A3-fix)** — adds `undone`, the undo cursor. Bumping is not optional
/// here and forgetting it was a real defect: v1 tables already carried a `shape`
/// stamp, so the version gate reported "up to date", the `ALTER TABLE` inside
/// [`ensure_shape_schema`] never ran, and every query naming `undone` failed
/// with "no such column" — undo went silently inert on exactly the machines that
/// had used the feature before (Boss-reported 2026-07-20).
///
/// **The rule this encodes: any change to the table's SHAPE must bump this in
/// the same edit.** The gate is only as good as the number it compares.
pub const SHAPE_SCHEMA_VERSION: i64 = 2;

/// The frontmatter key. One key, one meaning, everywhere.
pub const SHAPE_KEY: &str = "shape";

/// The CONTAINER vocabulary — deliberately tiny.
///
/// These describe how much room a note is given, not what KIND of thing it is.
/// Kind (journal, essay, book…) is a different axis and is **proposal-only**
/// (Plan §3 Phase F): structural inference may change how a note BEHAVES; it may
/// never silently change what a note IS. Absent shape is a valid, common state
/// and means "unshaped" — it is not an error and must never be auto-filled.
pub const SHAPES: [&str; 2] = ["scrap", "page"];

pub fn is_valid_shape(s: &str) -> bool {
    SHAPES.contains(&s)
}

/// One recorded shape transition. `from_shape: None` means the note was
/// unshaped before the change; reverting to it REMOVES the key rather than
/// writing an empty value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeChange {
    pub path: String,
    pub from_shape: Option<String>,
    pub to_shape: Option<String>,
    pub changed_at: i64,
    /// `"user"` | `"container_auto"`. Phase A only ever writes `"user"`;
    /// `container_auto` is reserved for Phase F, and keeping the column from the
    /// start means an auto-graduation is inspectable the day it ships.
    pub changed_by: String,
}

pub fn ensure_shape_schema(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS shape_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL,
            from_shape TEXT,
            to_shape TEXT,
            changed_at INTEGER NOT NULL,
            changed_by TEXT NOT NULL DEFAULT 'user',
            -- §A3-fix — undo CONSUMES a row rather than appending its inverse.
            -- Appending made each undo the next undo's target, so repeated undo
            -- oscillated page→scrap→page forever instead of walking back to
            -- unshaped (Boss-reported 2026-07-20). The trail stays append-only
            -- for audit; this flag is the cursor.
            undone INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS idx_shape_history_path ON shape_history(path, id DESC);",
    )
    .map_err(|e| format!("Failed to create shape_history: {}", e))?;

    // ── v1 → v2 upgrade ──
    let prior: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'shape'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if prior < 2 {
        // Tables created at v1 lack `undone`. Adding it is idempotent-by-ignore
        // (a fresh CREATE above already has the column).
        let _ = conn.execute(
            "ALTER TABLE shape_history ADD COLUMN undone INTEGER NOT NULL DEFAULT 0",
            [],
        );
        // v1 rows were produced by the design in which an undo APPENDED its own
        // inverse, so they are not a valid undo stack — replaying them would
        // re-enact the oscillation as history. They are an audit trail of a
        // defect, not of the user's intent. Discard them; the notes' actual
        // shapes live in the files and are untouched by this.
        let _ = conn.execute("DELETE FROM shape_history", []);
    }

    conn.execute(
        "INSERT INTO schema_versions (module, version, updated_at) VALUES ('shape', ?1, ?2)
         ON CONFLICT(module) DO UPDATE SET version = ?1, updated_at = ?2",
        rusqlite::params![SHAPE_SCHEMA_VERSION, now_secs()],
    )
    .map_err(|e| format!("Failed to stamp shape schema: {}", e))?;
    Ok(())
}

pub fn is_stamped(conn: &rusqlite::Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'shape'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .map(|v| v >= SHAPE_SCHEMA_VERSION)
    .unwrap_or(false)
}

/// Bring the schema up to the current version if it is behind, then report
/// whether it is usable.
///
/// **Every entry point calls this instead of bailing on `!is_stamped`.** The
/// original code returned early when the stamp was missing, which meant a table
/// one version behind could never upgrade itself — the feature just went quiet.
/// Ensuring here is cheap (a version read, then a no-op) and removes the whole
/// class rather than the one instance.
fn ensure_ready(conn: &rusqlite::Connection) -> bool {
    if is_stamped(conn) {
        return true;
    }
    ensure_shape_schema(conn).is_ok()
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Read a note's current shape straight off disk. Disk is the source of truth
/// (File Over App); the DB is an index, never the authority on what a note says.
pub fn read_shape_from_disk(content: &str) -> Option<String> {
    crate::bases::parse_frontmatter(content)
        .and_then(|p| p.get(SHAPE_KEY).cloned())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Resolve the library that owns `file_path` within the ACTIVE universe's OWN
/// libraries (non-recursive — a write must never land in a read-only cUniverse).
/// MIG-105 Stage-0 C7 (PJ-156): delegates to the shared longest-root resolver —
/// the old first-match fs::canonicalize `find` attributed a nested library's
/// note to the parent library whose root prefixes it (wrong library stamped
/// into note_meta on reindex). Intended behavior changes: a missing file now
/// resolves lexically (the gate_rmw read surfaces the honest error downstream);
/// `..` paths are denied (None → Access denied).
fn owning_library(app: &tauri::AppHandle, file_path: &str) -> Result<String, String> {
    crate::libraries::owning_own_library_name(app, file_path)
        .ok_or_else(|| "Access denied: file is not in a registered library.".to_string())
}

/// Apply a shape to disk and record the transition. `to` of `None` removes the
/// key entirely.
///
/// The read → rewrite → write cycle runs INSIDE `gate_rmw`, so the per-path lock
/// covers the whole cycle and a debounced editor save can land before or after
/// but never inside its window (the Note-open-freeze Batch-2 discipline).
fn apply_shape(
    app: &tauri::AppHandle,
    file_path: &str,
    to: Option<&str>,
    changed_by: &str,
) -> Result<Option<String>, String> {
    let lib_name = owning_library(app, file_path)?;

    // Capture the prior shape under the SAME lock that performs the write, so
    // the history row can never disagree with what was actually replaced.
    let mut from: Option<String> = None;
    crate::write_gate::gate_rmw(Path::new(file_path), "shape_set", |content| {
        from = read_shape_from_disk(content);
        if from.as_deref() == to {
            return Ok(None); // no-op: identical shape, nothing written
        }
        Ok(Some(match to {
            Some(s) => crate::bases::update_frontmatter_property(content, SHAPE_KEY, s),
            None => crate::bases::remove_frontmatter_property(content, SHAPE_KEY),
        }))
    })?;

    if from.as_deref() == to {
        return Ok(from);
    }

    record_change(app, file_path, from.as_deref(), to, changed_by);

    // Best-effort index refresh: the disk write is the source of truth, and a
    // reindex glitch must not fail the edit (the watcher would catch it anyway).
    {
        use tauri::Manager;
        if let Some(state) = app.try_state::<crate::search::SearchState>() {
            let _ = crate::search::reindex_single_note(&state, file_path, &lib_name);
        }
    }
    Ok(from)
}

fn record_change(
    app: &tauri::AppHandle,
    path: &str,
    from: Option<&str>,
    to: Option<&str>,
    changed_by: &str,
) {
    use tauri::Manager;
    let Some(state) = app.try_state::<crate::search::SearchState>() else {
        return;
    };
    let Ok(guard) = state.db.lock() else { return };
    let Some(conn) = guard.as_ref() else { return };
    if !ensure_ready(conn) {
        return;
    }
    // §A3-fix — a fresh user change truncates the redo branch, exactly like any
    // undo stack: once you act after undoing, the undone steps are gone and the
    // stack is linear again. Without this, undo would later walk back into
    // superseded history and appear to "jump".
    let _ = conn.execute(
        "DELETE FROM shape_history WHERE path = ?1 AND undone = 1",
        rusqlite::params![path],
    );
    let _ = conn.execute(
        "INSERT INTO shape_history (path, from_shape, to_shape, changed_at, changed_by, undone)
         VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        rusqlite::params![path, from, to, now_secs(), changed_by],
    );
}

// ─── Tauri commands ───

#[tauri::command(async)]
pub fn set_note_shape(
    app: tauri::AppHandle,
    file_path: String,
    shape: String,
) -> Result<(), String> {
    if !is_valid_shape(&shape) {
        return Err(format!(
            "Unknown shape '{}'. Known shapes: {}.",
            shape,
            SHAPES.join(", ")
        ));
    }
    apply_shape(&app, &file_path, Some(&shape), "user")?;
    Ok(())
}

/// Clear a note's shape, returning it to the unshaped state.
#[tauri::command(async)]
pub fn clear_note_shape(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    apply_shape(&app, &file_path, None, "user")?;
    Ok(())
}

/// **Phase A3 — the revert gesture.** Undo the most recent shape change for this
/// note, restoring exactly what was there before it. Reverting a change that
/// came FROM unshaped removes the key, so the file returns to its original
/// bytes. The revert is itself recorded, so the trail stays complete and a
/// revert can in turn be reverted.
#[tauri::command(async)]
pub fn revert_note_shape(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    use tauri::Manager;
    let previous: Option<String> = {
        let state = app
            .try_state::<crate::search::SearchState>()
            .ok_or_else(|| "Search index is not ready.".to_string())?;
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard
            .as_ref()
            .ok_or_else(|| "Search index is not ready.".to_string())?;
        if !ensure_ready(conn) {
            return Err("Shape history is unavailable.".to_string());
        }
        conn.query_row(
            "SELECT from_shape FROM shape_history WHERE path = ?1 ORDER BY id DESC LIMIT 1",
            rusqlite::params![&file_path],
            |r| r.get::<_, Option<String>>(0),
        )
        .map_err(|_| "No shape history for this note yet.".to_string())?
    };
    apply_shape(&app, &file_path, previous.as_deref(), "user")?;
    Ok(())
}

/// Record a shape transition WITHOUT touching the file.
///
/// **MIG-101 §A-fix.** When a note is open, its in-memory model owns its content
/// (MIG-076 single content ownership) and the shape is written through the model's
/// own durable save — never by this module writing around it. Discovered the hard
/// way: writing `shape:` to disk behind an open note left the model composing
/// frontmatter from its open-time snapshot, so the next keystroke-triggered save
/// silently dropped the key. The disk write and the history record therefore have
/// to be separable, and this is the history half.
#[tauri::command(async)]
pub fn record_shape_change(
    app: tauri::AppHandle,
    file_path: String,
    from_shape: Option<String>,
    to_shape: Option<String>,
) -> Result<(), String> {
    if let Some(s) = to_shape.as_deref() {
        if !is_valid_shape(s) {
            return Err(format!("Unknown shape '{}'.", s));
        }
    }
    record_change(
        &app,
        &file_path,
        from_shape.as_deref(),
        to_shape.as_deref(),
        "user",
    );
    Ok(())
}

/// **§A3-fix — take one step BACK through the history, consuming it.**
///
/// Returns the shape to restore (`Ok(None)` legitimately means "back to
/// unshaped") and marks the step consumed in the same locked section, so the
/// caller cannot be handed the same step twice.
///
/// The original implementation returned the last step's `from_shape` and then
/// let the caller APPEND the revert as a new change. That made every undo the
/// next undo's target, so repeated undo oscillated `page → scrap → page …`
/// forever and could never reach unshaped. Consuming the step instead makes undo
/// walk backwards and stop, which is what "undo" means.
///
/// `Err` means there is nothing left to undo — a normal end state, not a fault.
#[tauri::command(async)]
pub fn undo_shape(app: tauri::AppHandle, file_path: String) -> Result<Option<String>, String> {
    use tauri::Manager;
    let state = app
        .try_state::<crate::search::SearchState>()
        .ok_or_else(|| "Search index is not ready.".to_string())?;
    let guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| "Search index is not ready.".to_string())?;
    if !ensure_ready(conn) {
        return Err("Shape history is unavailable.".to_string());
    }
    let (id, target): (i64, Option<String>) = conn
        .query_row(
            "SELECT id, from_shape FROM shape_history
             WHERE path = ?1 AND undone = 0 ORDER BY id DESC LIMIT 1",
            rusqlite::params![&file_path],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "Nothing to undo for this note.".to_string())?;
    conn.execute(
        "UPDATE shape_history SET undone = 1 WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(target)
}

#[tauri::command(async)]
pub fn get_note_shape(file_path: String) -> Result<Option<String>, String> {
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    Ok(read_shape_from_disk(&content))
}

#[tauri::command(async)]
pub fn get_shape_history(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<Vec<ShapeChange>, String> {
    use tauri::Manager;
    let state = app
        .try_state::<crate::search::SearchState>()
        .ok_or_else(|| "Search index is not ready.".to_string())?;
    // Read side: prefer the read-only connection so we never queue behind the
    // writer (PJ-066 §C3), falling back to `db` pre-init / mid-switch.
    let read_guard = state.read_db.lock().map_err(|e| e.to_string())?;
    let write_guard;
    let conn = match read_guard.as_ref() {
        Some(c) => c,
        None => {
            drop(read_guard);
            write_guard = state.db.lock().map_err(|e| e.to_string())?;
            write_guard
                .as_ref()
                .ok_or_else(|| "Search index is not ready.".to_string())?
        }
    };
    if !ensure_ready(conn) {
        return Ok(Vec::new());
    }
    let mut stmt = conn
        .prepare(
            "SELECT path, from_shape, to_shape, changed_at, changed_by
             FROM shape_history WHERE path = ?1 AND undone = 0 ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![&file_path], |r| {
            Ok(ShapeChange {
                path: r.get(0)?,
                from_shape: r.get(1)?,
                to_shape: r.get(2)?,
                changed_at: r.get(3)?,
                changed_by: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bases::{remove_frontmatter_property, update_frontmatter_property};

    fn db() -> rusqlite::Connection {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        ensure_shape_schema(&c).unwrap();
        c
    }

    #[test]
    fn vocabulary_is_closed() {
        assert!(is_valid_shape("scrap"));
        assert!(is_valid_shape("page"));
        assert!(!is_valid_shape("journal"), "kind must not be settable as a shape");
        assert!(!is_valid_shape(""));
        assert!(!is_valid_shape("Scrap"), "vocabulary is case-sensitive");
    }

    #[test]
    fn schema_stamps_and_reports() {
        let c = db();
        assert!(is_stamped(&c));
    }

    /// **The migration regression.** A table created at v1 (no `undone` column,
    /// stamped `shape = 1`) must upgrade itself when the code that needs the
    /// column runs. The original gate asked only "is there a stamp?", so a v1
    /// table reported healthy, the ALTER never ran, and every query naming
    /// `undone` failed — undo went silently inert on machines that had used the
    /// feature before. This is the test that would have caught it.
    #[test]
    fn a_v1_table_upgrades_itself_to_v2() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        // Reconstruct a genuine v1 database, exactly as it existed on disk.
        c.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);
             CREATE TABLE shape_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                from_shape TEXT,
                to_shape TEXT,
                changed_at INTEGER NOT NULL,
                changed_by TEXT NOT NULL DEFAULT 'user'
             );
             INSERT INTO schema_versions (module, version) VALUES ('shape', 1);
             INSERT INTO shape_history (path, from_shape, to_shape, changed_at, changed_by)
                VALUES ('/n.md', 'scrap', 'page', 1, 'user');",
        )
        .unwrap();

        // The v1 stamp must NOT be mistaken for "up to date".
        assert!(!is_stamped(&c), "a v1 stamp must read as out-of-date at v2");

        assert!(ensure_ready(&c), "the schema must upgrade itself");
        assert!(is_stamped(&c), "and re-stamp at the current version");

        // The column now exists and is queryable — the exact query that failed.
        let live: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM shape_history WHERE undone = 0",
                [],
                |r| r.get(0),
            )
            .expect("querying `undone` must succeed after the upgrade");

        // v1 rows recorded undo-inverses, so they are not a valid undo stack and
        // are discarded rather than replayed.
        assert_eq!(live, 0, "defect-era history must be discarded on upgrade");
    }

    /// Ensuring twice must be a no-op, not a repeated destructive upgrade.
    #[test]
    fn ensure_is_idempotent_and_does_not_eat_live_history() {
        let c = db();
        push(&c, "/n.md", None, Some("scrap"));
        assert!(ensure_ready(&c));
        assert!(ensure_ready(&c));
        let live: i64 = c
            .query_row("SELECT COUNT(*) FROM shape_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(live, 1, "a second ensure must not clear real history");
    }

    #[test]
    fn reads_shape_off_disk_and_treats_blank_as_unshaped() {
        assert_eq!(
            read_shape_from_disk("---\nshape: scrap\n---\nBody\n").as_deref(),
            Some("scrap")
        );
        assert_eq!(read_shape_from_disk("---\nshape: \n---\nBody\n"), None);
        assert_eq!(read_shape_from_disk("No frontmatter\n"), None);
    }

    // ── The Phase A3 promise, proven end-to-end on content ──

    /// Unshaped → shaped → reverted returns the file to its ORIGINAL BYTES,
    /// including the case where the note had no frontmatter at all and the
    /// whole block therefore has to disappear again.
    #[test]
    fn revert_to_unshaped_restores_original_bytes() {
        for original in [
            "Just a body\n",
            "Body with no trailing newline",
            "---\ntitle: A\n---\nBody\n",
            "---\r\ntitle: A\r\n---\r\nBody\r\n",
            "---\ntags:\n  - one\n  - two\n---\n\nBody\n\n",
        ] {
            let shaped = update_frontmatter_property(original, SHAPE_KEY, "scrap");
            assert_eq!(read_shape_from_disk(&shaped).as_deref(), Some("scrap"));
            let reverted = remove_frontmatter_property(&shaped, SHAPE_KEY);
            assert_eq!(
                reverted, original,
                "revert-to-unshaped was not byte-exact for {original:?}"
            );
        }
    }

    /// shaped → re-shaped → reverted returns the earlier shape and leaves the
    /// rest of the file identical.
    #[test]
    fn revert_to_previous_shape_restores_bytes() {
        let original = "---\ntitle: A\nshape: scrap\n---\nBody\n";
        let changed = update_frontmatter_property(original, SHAPE_KEY, "page");
        assert_eq!(read_shape_from_disk(&changed).as_deref(), Some("page"));
        let reverted = update_frontmatter_property(&changed, SHAPE_KEY, "scrap");
        assert_eq!(reverted, original, "revert to previous shape was not byte-exact");
    }

    #[test]
    fn history_round_trips_and_orders_newest_first() {
        let c = db();
        for (from, to, at) in [
            (None, Some("scrap"), 100i64),
            (Some("scrap"), Some("page"), 200),
        ] {
            c.execute(
                "INSERT INTO shape_history (path, from_shape, to_shape, changed_at, changed_by)
                 VALUES (?1, ?2, ?3, ?4, 'user')",
                rusqlite::params!["/n.md", from, to, at],
            )
            .unwrap();
        }
        let latest_from: Option<String> = c
            .query_row(
                "SELECT from_shape FROM shape_history WHERE path = ?1 ORDER BY id DESC LIMIT 1",
                rusqlite::params!["/n.md"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(latest_from.as_deref(), Some("scrap"), "revert target is the newest row's from_shape");
    }

    // ── §A3-fix — the undo STACK. Boss-reported 2026-07-20: repeated undo
    //    oscillated page → scrap → page … forever and never reached unshaped,
    //    because each undo appended its own inverse as a new step. These pin
    //    the corrected semantics: undo CONSUMES a step and walks backwards.

    /// Simulates the exact sequence the Boss performed. The original design
    /// looped here; this asserts it terminates at unshaped and then stops.
    fn undo_target(c: &rusqlite::Connection, path: &str) -> Result<Option<String>, ()> {
        let (id, target): (i64, Option<String>) = c
            .query_row(
                "SELECT id, from_shape FROM shape_history
                 WHERE path = ?1 AND undone = 0 ORDER BY id DESC LIMIT 1",
                rusqlite::params![path],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| ())?;
        c.execute("UPDATE shape_history SET undone = 1 WHERE id = ?1", rusqlite::params![id])
            .unwrap();
        Ok(target)
    }

    fn push(c: &rusqlite::Connection, path: &str, from: Option<&str>, to: Option<&str>) {
        c.execute("DELETE FROM shape_history WHERE path = ?1 AND undone = 1", rusqlite::params![path]).unwrap();
        c.execute(
            "INSERT INTO shape_history (path, from_shape, to_shape, changed_at, changed_by, undone)
             VALUES (?1, ?2, ?3, 0, 'user', 0)",
            rusqlite::params![path, from, to],
        )
        .unwrap();
    }

    #[test]
    fn undo_walks_back_to_unshaped_and_then_stops() {
        let c = db();
        push(&c, "/n.md", None, Some("scrap")); // Shape: Scrap
        push(&c, "/n.md", Some("scrap"), Some("page")); // Shape: Page

        assert_eq!(undo_target(&c, "/n.md"), Ok(Some("scrap".into())), "first undo → scrap");
        assert_eq!(undo_target(&c, "/n.md"), Ok(None), "second undo → unshaped");
        assert_eq!(undo_target(&c, "/n.md"), Err(()), "third undo → nothing left");
    }

    /// The oscillation itself, stated as an invariant: undoing N times from a
    /// stack of N steps must visit N DISTINCT states and terminate — never
    /// return to a state it already left.
    #[test]
    fn undo_never_oscillates() {
        let c = db();
        push(&c, "/n.md", None, Some("scrap"));
        push(&c, "/n.md", Some("scrap"), Some("page"));
        push(&c, "/n.md", Some("page"), Some("scrap"));

        let mut seen = Vec::new();
        while let Ok(t) = undo_target(&c, "/n.md") {
            assert!(seen.len() < 10, "undo did not terminate — it is looping: {seen:?}");
            seen.push(t);
        }
        assert_eq!(
            seen,
            vec![Some("page".to_string()), Some("scrap".to_string()), None],
            "undo must walk the stack backwards and end at unshaped"
        );
    }

    /// A fresh change after undoing truncates the redo branch, so undo cannot
    /// later walk back into superseded history and appear to jump.
    #[test]
    fn a_new_change_truncates_the_undone_branch() {
        let c = db();
        push(&c, "/n.md", None, Some("scrap"));
        push(&c, "/n.md", Some("scrap"), Some("page"));
        let _ = undo_target(&c, "/n.md"); // back to scrap; the page step is undone

        push(&c, "/n.md", Some("scrap"), Some("page")); // act again

        let live: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM shape_history WHERE path = '/n.md' AND undone = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(live, 0, "the undone branch must be discarded on a new change");
        assert_eq!(undo_target(&c, "/n.md"), Ok(Some("scrap".into())));
        assert_eq!(undo_target(&c, "/n.md"), Ok(None));
        assert_eq!(undo_target(&c, "/n.md"), Err(()));
    }

    /// A note whose first shape came from nothing must revert to NULL, not to
    /// an empty string — the difference between "unshaped" and "shaped as ''".
    #[test]
    fn first_change_reverts_to_null_not_empty_string() {
        let c = db();
        c.execute(
            "INSERT INTO shape_history (path, from_shape, to_shape, changed_at, changed_by)
             VALUES ('/n.md', NULL, 'scrap', 1, 'user')",
            [],
        )
        .unwrap();
        let from: Option<String> = c
            .query_row(
                "SELECT from_shape FROM shape_history WHERE path = '/n.md' ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(from, None);
    }
}

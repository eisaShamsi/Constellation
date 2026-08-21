//! MIG-066 §A.2 — Resumable back-fill for the outgoing-link aggregates.
//!
//! §A.1 added three columns to `note_meta` — `outgoing_count`,
//! `outgoing_link_types`, `outgoing_top_rank` — and the
//! `note_links_outgoing_*` triggers that keep them in lock-step with live
//! edge writes. But on the first boot after the migration lands, existing
//! notes (7,600+ on the target universe) have links that predate the
//! triggers, so their columns sit at the schema defaults (0 / '' / 9).
//! This module recomputes them once from `note_links`.
//!
//! Design constraints — identical in spirit to `sky_backfill.rs` (the model):
//!
//! - **Must not block boot.** Runs on a background thread scheduled by
//!   `ensure_search_db_ready` after the connection is live and first paint
//!   has happened. The MIG-013 lesson: a single bulk `UPDATE note_meta`
//!   froze boot for tens of seconds on a large universe — never again.
//! - **Must be resumable.** `links_outgoing_backfill_cursor` holds the last
//!   processed path. Killing the app mid-run and relaunching resumes from
//!   the cursor, not from scratch.
//! - **Must coexist with live writes.** Each batch is one transaction; the
//!   DB mutex is released between batches (plus a short sleep) so user saves
//!   and other IPC calls interleave.
//! - **Idempotent.** The recompute reads the current `note_links` state, so
//!   re-running a row — or racing a trigger on the same row — converges to
//!   the same value (both read the same source of truth). New notes created
//!   during the back-fill are handled by the triggers, not here.
//!
//! Unlike `sky_backfill`, this back-fill is **pure SQL** — every value comes
//! from `note_links`, so there are no per-note file reads. That makes it far
//! lighter than the sky/stratum/maturity back-fill it mirrors.
//!
//! Completion stamps `schema_versions.links_outgoing = LINKS_OUTGOING_SCHEMA_VERSION`.
//! Next boot detects the stamp and skips the back-fill.

use rusqlite::{params, Connection};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

use crate::search::{outgoing_aggregate_assignments, SearchState, LINKS_OUTGOING_SCHEMA_VERSION};

/// Notes recomputed per transaction. Smaller than sky_backfill's 1000 because
/// each row here runs three correlated subqueries over `note_links` *under the
/// lock* (sky does its expensive file reads outside the lock), so we keep the
/// lock-hold per batch short. 500 indexed recomputes is a few tens of ms.
const BATCH_SIZE: i64 = 500;

/// Sleep between batches — hands the DB mutex to other callers so the
/// back-fill never starves the main thread on a large universe.
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// Schedule the back-fill on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after `init_db` completes and the
/// connection is in state. Silent no-op if `schema_versions.links_outgoing`
/// is already current.
pub fn maybe_schedule(app: tauri::AppHandle) {
    // Cheap pre-check on the main thread — avoids spawning a thread for the
    // common case (already current).
    let state = app.state::<SearchState>();
    let needs_run = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        is_needed(conn)
    };
    if !needs_run {
        return;
    }

    let app_bg = app.clone();
    thread::spawn(move || {
        match run(&app_bg) {
            Ok(n) => diag(&app_bg, &format!("[links_backfill] completed: {} notes recomputed", n)),
            Err(e) => diag(&app_bg, &format!("[links_backfill] FAILED: {}", e)),
        }
    });
}

/// True when the back-fill still needs to run. Mirrors `sky_backfill::is_needed`:
/// the version is stamped only at `finalize` (completion), so an interrupted run
/// leaves it below target and re-runs, resuming from the cursor.
fn is_needed(conn: &Connection) -> bool {
    if !version_current(conn) {
        return true;
    }
    // MIG-067 §B — vocabulary-change gate. The materialized columns (rank order,
    // per-type counts, the JSON) are derived from the active link-type vocabulary;
    // when it changes (a user adds / reorders / removes a type) the stored
    // aggregates go stale. We stamp the vocabulary fingerprint at each completed
    // back-fill; a mismatch re-runs the SAME resumable machinery to re-materialize
    // every row. This also covers the §A→§B upgrade: a universe last back-filled
    // under §A has no `links_vocab` stamp (fingerprint 0), so the seed registry's
    // non-zero fingerprint mismatches → a one-time pass fills the new JSON column.
    stored_vocab_fingerprint(conn) != crate::link_types::active_universe_vocabulary().fingerprint()
}

/// True once the §A.2 back-fill version stamp has reached target — i.e. a completed
/// pass. Distinguishes a fresh first-time back-fill (version behind → keep the
/// cursor so an interrupted run resumes) from a vocabulary-change re-run (version
/// current → the cursor refers to the old vocabulary's pass and must reset).
fn version_current(conn: &Connection) -> bool {
    let v: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'links_outgoing'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    v >= LINKS_OUTGOING_SCHEMA_VERSION
}

/// The vocabulary fingerprint stamped at the last completed back-fill (0 if never
/// — e.g. a universe back-filled under §A, before the `links_vocab` stamp existed).
fn stored_vocab_fingerprint(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'links_vocab'",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

/// The back-fill loop. Re-locks the DB mutex per batch so frontend IPC stays
/// responsive. Returns the number of notes recomputed.
fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    let state = app.state::<SearchState>();

    // One-time setup: the resumable cursor table. Idempotent.
    {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        ensure_cursor_table(conn)?;
    }

    // Give the planner statistics before the correlated subqueries run. Without
    // `sqlite_stat1`, an equality on `status` (every link is 'active' — a single
    // distinct value) looks as good as the equality on `source_path`, so the
    // planner can pick the non-selective `idx_link_status` and fan each subquery
    // across the whole `note_links` table — the exact trap `sky_backfill` hit
    // (200× slower). ANALYZE is idempotent; on an existing universe sky already
    // wrote these stats, so this just refreshes them. Cheap, once, background.
    {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        conn.busy_timeout(Duration::from_secs(30))
            .map_err(|e| format!("busy_timeout: {}", e))?;
        conn.execute_batch("ANALYZE")
            .map_err(|e| format!("ANALYZE: {}", e))?;
    }

    // MIG-067 §B — capture the vocabulary fingerprint for THIS run up-front; it is
    // stamped at finalize. If the vocabulary changes again mid-run, the stamp will
    // differ from the then-current fingerprint and `is_needed` re-runs us next time
    // (eventual consistency, without tracking the vocabulary per batch).
    let run_fp = crate::link_types::active_universe_vocabulary().fingerprint();

    // MIG-067 §B — if the version is already current, this run was triggered purely
    // by the vocabulary-change gate; any cursor left by a prior run belongs to the
    // OLD vocabulary's pass, so reset it and re-materialize every row (not just the
    // tail). A first-time back-fill (version behind) keeps its cursor so an
    // interrupted run resumes from where it stopped.
    {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        if version_current(conn) {
            conn.execute("DELETE FROM links_outgoing_backfill_cursor", [])
                .map_err(|e| format!("vocab-change cursor reset: {}", e))?;
        }
    }

    let mut last_path = read_cursor(&state.db)?;
    let mut total: u64 = 0;

    loop {
        let (batch_count, new_last_path) = process_batch(&state.db, &last_path)?;
        if batch_count == 0 {
            // Drained. Stamp the version + vocabulary fingerprint and clear the
            // cursor atomically.
            finalize(&state.db, run_fp)?;
            return Ok(total);
        }
        total += batch_count as u64;
        last_path = new_last_path;
        write_cursor(&state.db, &last_path)?;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

/// One batch under a single lock+transaction: read the next window of paths
/// (to find this batch's upper boundary), then recompute the three aggregates
/// for every note in `(after_path, last_path]`. The SELECT and the UPDATE share
/// one transaction so the range can't shift underneath us. The lock is released
/// when this function returns, before the inter-batch sleep.
///
/// Returns `(notes_in_batch, new_cursor)`. A 0 count means the table is drained.
fn process_batch(
    db: &Mutex<Option<Connection>>,
    after_path: &str,
) -> Result<(usize, String), String> {
    let mut guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not initialized")?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;
    let tx = conn.transaction().map_err(|e| format!("begin: {}", e))?;

    // Next window of paths — only the boundary + count are used; the recompute
    // itself is range-scoped (no big IN-list), so this stays cheap.
    let paths: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT path FROM note_meta WHERE path > ?1 ORDER BY path LIMIT ?2")
            .map_err(|e| format!("prepare batch: {}", e))?;
        let rows = stmt
            .query_map(params![after_path, BATCH_SIZE], |row| row.get::<_, String>(0))
            .map_err(|e| format!("query batch: {}", e))?;
        let mut v = Vec::with_capacity(BATCH_SIZE as usize);
        for r in rows {
            v.push(r.map_err(|e| format!("row batch: {}", e))?);
        }
        v
    };

    if paths.is_empty() {
        tx.commit().map_err(|e| format!("commit empty: {}", e))?;
        return Ok((0, after_path.to_string()));
    }

    let last_path = paths.last().cloned().unwrap_or_default();
    recompute_range(&tx, after_path, &last_path)
        .map_err(|e| format!("recompute range: {}", e))?;
    tx.commit().map_err(|e| format!("commit: {}", e))?;

    Ok((paths.len(), last_path))
}

/// The core recompute: set the three outgoing-link aggregates for every
/// `note_meta` row in `(after_path, last_path]` from `note_links`, using the
/// SAME SQL the §A.1 triggers use (via `outgoing_aggregate_assignments`, here
/// correlated on `note_meta.path`). Shared by `process_batch` and the tests so
/// the back-fill and the triggers can never drift. Returns rows touched.
pub(crate) fn recompute_range(conn: &Connection, after_path: &str, last_path: &str) -> rusqlite::Result<usize> {
    let sql = format!(
        "UPDATE note_meta SET {assign} WHERE path > ?1 AND path <= ?2",
        assign = outgoing_aggregate_assignments(&crate::link_types::active_universe_vocabulary(), "note_meta.path"),
    );
    conn.execute(&sql, params![after_path, last_path])
}

/// MIG-066 §A.2 — recompute the outgoing aggregates for EVERY note from
/// `note_links`. `reconcile_filesystem` calls this after a deliberately
/// trigger-free full re-index to restore the columns. Same SQL the triggers +
/// the batched back-fill use, so the three population paths can never drift.
/// Returns rows touched.
///
/// **BATCHED + lock-tolerant** (was a single whole-table UPDATE — which silently
/// failed under boot DB contention on a large universe, leaving the column stale:
/// the 2026-05-30 overnight blank). It now walks `note_meta` in 500-row windows,
/// each its own short UPDATE (so it never holds a long write lock), and retries a
/// batch on SQLITE_BUSY/locked instead of aborting the whole pass.
pub(crate) fn recompute_all_outgoing(conn: &Connection, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize> {
    let mut after = String::new();
    let mut total = 0usize;
    loop {
        let paths: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT path FROM note_meta WHERE path > ?1 ORDER BY path LIMIT 500")?;
            let rows = stmt.query_map(params![after], |r| r.get::<_, String>(0))?;
            let mut v = Vec::with_capacity(500);
            for r in rows {
                v.push(r?);
            }
            v
        };
        if paths.is_empty() {
            break;
        }
        let last = paths.last().cloned().unwrap_or_default();
        // One short UPDATE per window; retry on transient lock contention.
        let mut attempt = 0;
        loop {
            match recompute_range(conn, &after, &last) {
                Ok(_) => break,
                Err(e) if is_busy_error(&e) && attempt < 8 => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(400));
                }
                Err(e) => return Err(e),
            }
        }
        total += paths.len();
        after = last;
    }
    Ok(total)
}

/// MIG-079 §C.2a — recompute the INCOMING-link aggregates for a `(after, last]`
/// path window from `note_links` (the same `incoming_aggregate_assignments` SQL
/// the triggers use — single source of truth, can't drift). Shared by
/// `recompute_all_incoming` and the §C.2a backfill.
pub(crate) fn recompute_incoming_range(conn: &Connection, after: &str, last: &str) -> rusqlite::Result<usize> {
    let sql = format!(
        "UPDATE note_meta SET {assign} WHERE path > ?1 AND path <= ?2",
        assign = crate::search::incoming_aggregate_assignments(&crate::link_types::active_universe_vocabulary(), "note_meta"),
    );
    conn.execute(&sql, params![after, last])
}

/// MIG-079 §C.2a — recompute EVERY note's incoming aggregate from `note_links`.
/// `reconcile_filesystem` calls this after the trigger-free walk; the §C.2a
/// backfill calls it once on first upgrade. Batched (500-row windows, each its own
/// short UPDATE) + busy-retry — mirrors `recompute_all_outgoing` so it never holds
/// a long write lock on a large universe. Idempotent (reads current note_links).
pub(crate) fn recompute_all_incoming(conn: &Connection, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize> {
    let mut after = String::new();
    let mut total = 0usize;
    loop {
        let paths: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT path FROM note_meta WHERE path > ?1 ORDER BY path LIMIT 500")?;
            let rows = stmt.query_map(params![after], |r| r.get::<_, String>(0))?;
            let mut v = Vec::with_capacity(500);
            for r in rows {
                v.push(r?);
            }
            v
        };
        if paths.is_empty() {
            break;
        }
        let last = paths.last().cloned().unwrap_or_default();
        let mut attempt = 0;
        loop {
            match recompute_incoming_range(conn, &after, &last) {
                Ok(_) => break,
                Err(e) if is_busy_error(&e) && attempt < 8 => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(400));
                }
                Err(e) => return Err(e),
            }
        }
        total += paths.len();
        after = last;
    }
    Ok(total)
}

/// PJ-066 §B1 — recompute `sky_nodes.stratum` + `maturity` for a `(after, last]` path
/// window from `note_links`, using the SAME shared `STRATUM_SQL_EXPR` / `MATURITY_SQL_EXPR`
/// the triggers + sky_backfill use (single source of truth — cannot drift). One combined
/// UPDATE per window. Replaces the per-edge sky triggers' work on the bulk/reconcile path.
pub(crate) fn recompute_sky_range(conn: &Connection, after: &str, last: &str) -> rusqlite::Result<usize> {
    let sql = format!(
        "UPDATE sky_nodes SET stratum = ({stratum}), maturity = ({maturity}) WHERE path > ?1 AND path <= ?2",
        stratum = crate::search::stratum_sql_expr(),
        maturity = crate::search::maturity_sql_expr(),
    );
    conn.execute(&sql, params![after, last])
}

/// PJ-066 §B1 — recompute EVERY note's sky stratum + maturity from `note_links`.
/// `reconcile_filesystem` calls this after the trigger-free bulk walk (the per-edge sky
/// stratum/maturity triggers are dropped by §B4, so reconcile no longer maintains sky via
/// triggers — this is the replacement). Batched (500-row windows) + busy-retry, mirroring
/// `recompute_all_incoming` so it never holds a long write lock on a large universe.
/// Idempotent (reads current note_links); unconditional (self-heals stale values).
pub(crate) fn recompute_all_sky(conn: &Connection, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize> {
    let mut after = String::new();
    let mut total = 0usize;
    loop {
        let paths: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT path FROM sky_nodes WHERE path > ?1 ORDER BY path LIMIT 500")?;
            let rows = stmt.query_map(params![after], |r| r.get::<_, String>(0))?;
            let mut v = Vec::with_capacity(500);
            for r in rows {
                v.push(r?);
            }
            v
        };
        if paths.is_empty() {
            break;
        }
        let last = paths.last().cloned().unwrap_or_default();
        let mut attempt = 0;
        loop {
            match recompute_sky_range(conn, &after, &last) {
                Ok(_) => break,
                Err(e) if is_busy_error(&e) && attempt < 8 => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(400));
                }
                Err(e) => return Err(e),
            }
        }
        total += paths.len();
        after = last;
    }
    Ok(total)
}

/// True for SQLITE_BUSY / SQLITE_LOCKED (the transient contention worth retrying).
fn is_busy_error(e: &rusqlite::Error) -> bool {
    let s = e.to_string().to_lowercase();
    s.contains("locked") || s.contains("busy")
}

fn ensure_cursor_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS links_outgoing_backfill_cursor (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_path TEXT,
            started_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )
    .map_err(|e| format!("cursor table create: {}", e))
}

fn read_cursor(db: &Mutex<Option<Connection>>) -> Result<String, String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    let last: Option<String> = conn
        .query_row(
            "SELECT last_path FROM links_outgoing_backfill_cursor WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .ok();
    Ok(last.unwrap_or_default())
}

fn write_cursor(db: &Mutex<Option<Connection>>, last_path: &str) -> Result<(), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    conn.execute(
        "INSERT OR REPLACE INTO links_outgoing_backfill_cursor (id, last_path) VALUES (1, ?1)",
        params![last_path],
    )
    .map_err(|e| format!("cursor write: {}", e))?;
    Ok(())
}

fn finalize(db: &Mutex<Option<Connection>>, vocab_fingerprint: i64) -> Result<(), String> {
    let mut guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not initialized")?;
    // Stamp + cursor clear in one transaction so a crash between them can't
    // leave a completed back-fill with a live cursor row (which the next boot
    // would read as an interrupted run).
    let tx = conn.transaction().map_err(|e| format!("finalize begin: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('links_outgoing', ?1, strftime('%s','now'))",
        params![LINKS_OUTGOING_SCHEMA_VERSION],
    )
    .map_err(|e| format!("finalize stamp: {}", e))?;
    // MIG-067 §B — stamp the vocabulary fingerprint these aggregates were
    // materialized under, so a later vocabulary change re-triggers the back-fill
    // (see `is_needed`). Stored in `schema_versions` (version column = fingerprint)
    // — no new table; the value is an opaque i64 token, not an ordered version.
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('links_vocab', ?1, strftime('%s','now'))",
        params![vocab_fingerprint],
    )
    .map_err(|e| format!("finalize vocab stamp: {}", e))?;
    tx.execute("DELETE FROM links_outgoing_backfill_cursor", [])
        .map_err(|e| format!("finalize cursor: {}", e))?;
    tx.commit().map_err(|e| format!("finalize commit: {}", e))?;
    Ok(())
}

/// Write a line to the universe's diagnostics log. Thin wrapper around
/// `search::diag_log` — kept here so this module doesn't reach into the
/// search module's private helpers.
fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests {
    //! MIG-066 §A.2 — pins the back-fill's core recompute (`recompute_range`,
    //! the same `outgoing_aggregate_assignments` SQL production runs) against the
    //! bundled SQLite: it populates pre-existing rows from `note_links`, honors the
    //! canonical order + the rank sentinel, excludes archived edges, is range-scoped,
    //! and is idempotent on re-run. The scheduler/cursor/threading is mirrored from
    //! the proven `sky_backfill`, so the novel part — the recompute — is what we test.
    use super::*;

    /// Seed note_meta (columns at schema defaults) + note_links, WITHOUT the
    /// triggers — exactly the back-fill's scenario: links that predate them.
    fn seeded_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                outgoing_link_types TEXT NOT NULL DEFAULT '', outgoing_link_types_json TEXT NOT NULL DEFAULT '{}',
                outgoing_top_rank INTEGER NOT NULL DEFAULT 9
             );
             CREATE TABLE note_links (
                source_path TEXT, target_name TEXT, link_type TEXT, status TEXT DEFAULT 'active'
             );",
        )
        .unwrap();
        for p in ["/a.md", "/b.md", "/c.md"] {
            conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![p]).unwrap();
        }
        // /a.md: two typed (reverse canonical order) + one untyped + one ARCHIVED.
        // /b.md: one typed. /c.md: no links at all (stays at the default sentinel).
        let edges = [
            ("/a.md", "T1", "contradicts", "active"),
            ("/a.md", "T2", "supports", "active"),
            ("/a.md", "T3", "", "active"),
            ("/a.md", "T4", "causes", "archived"),
            ("/b.md", "T5", "exemplifies", "active"),
        ];
        for (s, t, lt, st) in edges {
            conn.execute(
                "INSERT INTO note_links (source_path, target_name, link_type, status) VALUES (?1, ?2, ?3, ?4)",
                params![s, t, lt, st],
            )
            .unwrap();
        }
        conn
    }

    fn read(conn: &Connection, path: &str) -> (i64, String, i64) {
        conn.query_row(
            "SELECT outgoing_count, outgoing_link_types, outgoing_top_rank FROM note_meta WHERE path = ?1",
            params![path],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    }

    #[test]
    fn backfill_populates_existing_rows() {
        let conn = seeded_db();
        // Pre-state: everything at the schema default.
        assert_eq!(read(&conn, "/a.md"), (0, String::new(), 9));

        // Full-range recompute (what process_batch runs per batch, here in one go).
        let touched = recompute_range(&conn, "", "/zzz").unwrap();
        assert_eq!(touched, 3, "all three note_meta rows in range are recomputed");

        // /a.md: archived 'causes' excluded → count 3 (supports/contradicts/untyped),
        // types in canonical order (supports=1 before contradicts=2), top rank = 1.
        assert_eq!(read(&conn, "/a.md"), (3, "supports (1), contradicts (1)".to_string(), 1));
        // /b.md: one typed link.
        assert_eq!(read(&conn, "/b.md"), (1, "exemplifies (1)".to_string(), 4));
        // /c.md: genuinely no links → recompute yields the same default sentinel.
        assert_eq!(read(&conn, "/c.md"), (0, String::new(), 9));
    }

    /// MIG-067 §B — the vocabulary-change gate. With the version already at target,
    /// `is_needed` is driven purely by the stored-vs-current vocabulary fingerprint:
    /// absent (a §A-era universe) → needed; matching → not needed; differing (a
    /// vocabulary edit) → needed again. (The global registry defaults to the 8 seeds
    /// in tests, so `snapshot().fingerprint()` is stable here.)
    #[test]
    fn vocab_fingerprint_gate_triggers_rematerialize() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        // §A.2 version satisfied — the only remaining driver is the fingerprint.
        conn.execute(
            "INSERT INTO schema_versions (module, version) VALUES ('links_outgoing', ?1)",
            params![LINKS_OUTGOING_SCHEMA_VERSION],
        )
        .unwrap();

        // No `links_vocab` stamp (a universe back-filled under §A) → stored 0 ≠ the
        // seed registry's non-zero fingerprint → re-trigger (fills the JSON column).
        assert!(is_needed(&conn), "missing vocab stamp must re-trigger the back-fill");

        // Stamp the CURRENT fingerprint → in sync → not needed.
        let fp = crate::link_types::active_universe_vocabulary().fingerprint();
        assert_ne!(fp, 0, "seed registry fingerprint is non-zero");
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('links_vocab', ?1)",
            params![fp],
        )
        .unwrap();
        assert!(!is_needed(&conn), "matching vocab stamp must NOT re-trigger");

        // Simulate a vocabulary edit: a different stored fingerprint → needed again.
        conn.execute(
            "UPDATE schema_versions SET version = ?1 WHERE module = 'links_vocab'",
            params![fp ^ 0x5555],
        )
        .unwrap();
        assert!(is_needed(&conn), "changed vocab fingerprint must re-trigger");
    }

    #[test]
    fn backfill_is_range_scoped() {
        let conn = seeded_db();
        // Recompute only (after "", up to and including "/a.md") — paths sort
        // "/a.md" < "/b.md" < "/c.md", so only /a.md is in range.
        recompute_range(&conn, "", "/a.md").unwrap();
        assert_eq!(read(&conn, "/a.md"), (3, "supports (1), contradicts (1)".to_string(), 1));
        assert_eq!(read(&conn, "/b.md"), (0, String::new(), 9), "/b.md is outside the range — untouched");
    }

    #[test]
    fn backfill_is_idempotent() {
        let conn = seeded_db();
        recompute_range(&conn, "", "/zzz").unwrap();
        let first = read(&conn, "/a.md");
        // Re-running over the same range converges to the identical value.
        recompute_range(&conn, "", "/zzz").unwrap();
        assert_eq!(read(&conn, "/a.md"), first, "recompute is deterministic from note_links");
    }

    /// MIG-066 §A.2 perf gate (Rule 8 / WA#4). The only thing §A.2 can regress is
    /// the §A.1 `note_links_outgoing_*` triggers firing per-edge during a full
    /// re-index (each note's links are rebuilt via per-source DELETE + re-INSERT —
    /// the `index_note` shape, search.rs:3850). This isolates that family's
    /// MARGINAL cost: it times the identical full rebuild over a 7,600-note /
    /// ~217k-link synthetic universe WITHOUT the triggers (baseline = "before")
    /// then WITH them ("after"). The other 3 note_links trigger families (sky /
    /// maturity / stratum) are unchanged by this MIG, so they cancel in the delta
    /// and are omitted — the delta IS the regression attributable to §A.1+§A.2.
    ///
    /// Run (release, so the rusqlite glue is optimized):
    ///   cargo test --release --lib --manifest-path src-tauri/Cargo.toml \
    ///     -- --ignored --nocapture bench_reindex_trigger_overhead
    #[test]
    #[ignore = "perf benchmark — run explicitly with --ignored --nocapture"]
    fn bench_reindex_trigger_overhead() {
        use std::time::Instant;

        const N: usize = 7_600;
        // 9 link-type slots: the 8 canonical types + untyped (the real on-disk mix).
        let types = [
            "supports", "contradicts", "causes", "exemplifies", "generalizes",
            "derives-from", "part-of", "supersedes", "",
        ];

        // Skewed link plan: links_i = 5 + (i % 48) → 5..52 per note, avg ~28.5 →
        // ~217k total, matching the target universe's note_links row count.
        let mut plan: Vec<(String, Vec<(String, String)>)> = Vec::with_capacity(N);
        let mut total_links = 0usize;
        for i in 0..N {
            let src = format!("/lib/note_{:05}.md", i);
            let k = 5 + (i % 48);
            let mut edges = Vec::with_capacity(k);
            for j in 0..k {
                let tgt = format!("Target {}", (i + j * 13) % N);
                let lt = types[(i + j) % types.len()].to_string();
                edges.push((tgt, lt));
            }
            total_links += k;
            plan.push((src, edges));
        }

        // Temp file DB (real WAL behavior, not in-memory) — the production shape.
        let db_file = std::env::temp_dir().join("mig066_bench_reindex.db");
        let _ = std::fs::remove_file(&db_file);
        let _ = std::fs::remove_file(db_file.with_extension("db-wal"));
        let conn = Connection::open(&db_file).unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;").unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY, name TEXT, library_name TEXT,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                outgoing_link_types TEXT NOT NULL DEFAULT '', outgoing_link_types_json TEXT NOT NULL DEFAULT '{}',
                outgoing_top_rank INTEGER NOT NULL DEFAULT 9);
             CREATE TABLE note_links (
                source_path TEXT, source_name TEXT, target_name TEXT,
                link_type TEXT, status TEXT DEFAULT 'active');
             CREATE INDEX idx_link_source ON note_links(source_path);
             CREATE INDEX idx_link_status ON note_links(status);",
        )
        .unwrap();

        // Seed note_meta + initial note_links (trigger-free: the family doesn't
        // exist yet). One transaction.
        conn.execute_batch("BEGIN").unwrap();
        for (src, edges) in &plan {
            conn.execute(
                "INSERT INTO note_meta (path, name, library_name) VALUES (?1, ?1, 'lib')",
                params![src],
            )
            .unwrap();
            for (tgt, lt) in edges {
                conn.execute(
                    "INSERT INTO note_links (source_path, source_name, target_name, link_type, status)
                     VALUES (?1, ?1, ?2, ?3, 'active')",
                    params![src, tgt, lt],
                )
                .unwrap();
            }
        }
        conn.execute_batch("COMMIT").unwrap();
        conn.execute_batch("ANALYZE").unwrap();
        eprintln!("[bench] seeded {} notes, {} links", N, total_links);

        // One full re-index-pattern rebuild: per-source DELETE + re-INSERT of every
        // edge (exactly index_note's note_links churn). Returns elapsed.
        let rebuild = |conn: &Connection| -> std::time::Duration {
            let t = Instant::now();
            conn.execute_batch("BEGIN").unwrap();
            for (src, edges) in &plan {
                conn.execute("DELETE FROM note_links WHERE source_path = ?1", params![src]).unwrap();
                for (tgt, lt) in edges {
                    conn.execute(
                        "INSERT INTO note_links (source_path, source_name, target_name, link_type, status)
                         VALUES (?1, ?1, ?2, ?3, 'active')",
                        params![src, tgt, lt],
                    )
                    .unwrap();
                }
            }
            conn.execute_batch("COMMIT").unwrap();
            t.elapsed()
        };

        let _warmup = rebuild(&conn); // warm the page cache so the delta is fair.
        let t_without = rebuild(&conn);

        // Add the production outgoing-link trigger family.
        conn.execute_batch(&format!(
            "CREATE TRIGGER note_links_outgoing_ai AFTER INSERT ON note_links \
               BEGIN UPDATE note_meta SET {ins} WHERE path = NEW.source_path; END; \
             CREATE TRIGGER note_links_outgoing_ad AFTER DELETE ON note_links \
               BEGIN UPDATE note_meta SET {del} WHERE path = OLD.source_path; END; \
             CREATE TRIGGER note_links_outgoing_au AFTER UPDATE ON note_links \
               BEGIN UPDATE note_meta SET {del} WHERE path = OLD.source_path; \
                     UPDATE note_meta SET {ins} WHERE path = NEW.source_path; END;",
            ins = outgoing_aggregate_assignments(&crate::link_types::active_universe_vocabulary(), "NEW.source_path"),
            del = outgoing_aggregate_assignments(&crate::link_types::active_universe_vocabulary(), "OLD.source_path"),
        ))
        .unwrap();

        let t_with = rebuild(&conn);

        // Sanity: the triggers actually populated the aggregates during the rebuild.
        let sample: (i64, String, i64) = read(&conn, "/lib/note_00100.md");
        eprintln!("[bench] sample /lib/note_00100.md after rebuild: {:?}", sample);
        assert!(sample.0 > 0, "triggers maintained outgoing_count during the rebuild");

        // The §A.2 fix `reconcile_filesystem` applies: drop the family for the
        // bulk walk (→ the `t_without` baseline) then ONE `recompute_all_outgoing`
        // pass. Measure that pass so we can report the fixed total vs the unfixed.
        let t_recompute = {
            let t = Instant::now();
            recompute_all_outgoing(&conn, &crate::converge::ConvergeKey::for_test()).unwrap();
            t.elapsed()
        };
        let fixed_total = t_without + t_recompute;

        let delta = t_with.saturating_sub(t_without);
        let pct = 100.0 * delta.as_secs_f64() / t_without.as_secs_f64().max(1e-9);
        eprintln!("[bench] full re-index rebuild — {} notes / {} links:", N, total_links);
        eprintln!("[bench]   UNFIXED triggers-on per-edge:        BEFORE {:?} → AFTER {:?}  (DELTA {:?}, +{:.1}%)", t_without, t_with, delta, pct);
        eprintln!("[bench]   FIXED   paused-for-walk + recompute: {:?}  (bulk {:?} + recompute {:?})", fixed_total, t_without, t_recompute);
        eprintln!("[bench]   single-note save (triggers stay on), amortized: {:.3} ms/note", delta.as_secs_f64() * 1000.0 / N as f64);

        drop(conn);
        let _ = std::fs::remove_file(&db_file);
        let _ = std::fs::remove_file(db_file.with_extension("db-wal"));
    }
}

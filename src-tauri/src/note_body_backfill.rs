//! MIG-078 Phase BL §BL.1 — Resumable back-fill that copies existing note
//! bodies from `note_meta.body_text` into the new `note_body` table.
//!
//! §BL.1 adds `note_body` and makes `index_note` dual-write (note_body first,
//! then the note_meta UPSERT). But on the first boot after the migration lands,
//! the 7,653 pre-existing notes have a body only in `note_meta` — this module
//! copies them across once, in the background, so §BL.2 (flip reads + FTS
//! triggers to `note_body`) can later assert completeness before activating.
//!
//! Design — mirrors `links_backfill.rs` (the proven model):
//! - **Never blocks boot.** Background thread scheduled by `ensure_search_db_ready`
//!   after the connection is live and first paint has happened.
//! - **Resumable.** `note_body_backfill_cursor` holds the last copied path;
//!   an interrupted run resumes from the cursor, not from scratch.
//! - **Coexists with live writes.** Each batch is one short transaction; the DB
//!   mutex is released between batches (+ a short sleep) so user saves interleave.
//!   `INSERT ... SELECT ... ON CONFLICT DO UPDATE` makes the back-fill idempotent
//!   AND lets it coexist with the live dual-write that may have already populated
//!   some rows ahead of the cursor.
//! - **Large-body-safe.** The copy is `INSERT INTO note_body SELECT body_text
//!   FROM note_meta` — the body never crosses into Rust memory; SQLite copies it
//!   internally (matters for the 123 MB outlier note).
//!
//! Completion stamps `schema_versions.note_body_backfill = NOTE_BODY_BACKFILL_VERSION`.

use rusqlite::{params, Connection};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Bump if the back-fill logic changes in a way that must re-run on existing DBs.
const NOTE_BODY_BACKFILL_VERSION: i64 = 1;

/// Notes copied per transaction. Kept modest because a batch may include a very
/// large body (the 123 MB outlier) whose copy dominates the WAL write for that
/// txn; smaller batches keep the lock-hold per batch short for the common case.
const BATCH_SIZE: i64 = 200;

/// Sleep between batches — hands the DB mutex to other callers so the back-fill
/// never starves the main thread on a large universe.
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// Schedule the back-fill on a background thread. Returns immediately. Silent
/// no-op if `schema_versions.note_body_backfill` is already current.
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
        is_needed(conn)
    };
    if !needs_run {
        return;
    }

    let app_bg = app.clone();
    thread::spawn(move || match run(&app_bg) {
        Ok(n) => diag(&app_bg, &format!("[note_body_backfill] completed: {} bodies copied", n)),
        Err(e) => diag(&app_bg, &format!("[note_body_backfill] FAILED (non-fatal): {}", e)),
    });
}

/// True while the back-fill still needs to run (version stamped only at completion).
fn is_needed(conn: &Connection) -> bool {
    let v: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'note_body_backfill'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    v < NOTE_BODY_BACKFILL_VERSION
}

fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    let state = app.state::<SearchState>();
    {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        ensure_cursor_table(conn)?;
    }

    let mut last_path = read_cursor(&state.db)?;
    let mut total: u64 = 0;

    loop {
        let (batch_count, new_last_path) = process_batch(&state.db, &last_path)?;
        if batch_count == 0 {
            // Drained. Verify completeness, then stamp + clear the cursor.
            finalize(&state.db)?;
            return Ok(total);
        }
        total += batch_count as u64;
        last_path = new_last_path;
        write_cursor(&state.db, &last_path)?;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

/// One batch under a single lock+transaction: find this batch's path window, then
/// copy `(after_path, last_path]` bodies into note_body via INSERT…SELECT (the
/// body stays inside SQLite). Returns `(rows_in_batch, new_cursor)`; 0 = drained.
fn process_batch(
    db: &Mutex<Option<Connection>>,
    after_path: &str,
) -> Result<(usize, String), String> {
    let mut guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not initialized")?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;
    let tx = conn.transaction().map_err(|e| format!("begin: {}", e))?;

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
    tx.execute(
        "INSERT INTO note_body (path, body_text)
           SELECT path, body_text FROM note_meta
           WHERE path > ?1 AND path <= ?2
         ON CONFLICT(path) DO UPDATE SET body_text = excluded.body_text",
        params![after_path, last_path],
    )
    .map_err(|e| format!("copy range: {}", e))?;
    tx.commit().map_err(|e| format!("commit: {}", e))?;

    Ok((paths.len(), last_path))
}

fn ensure_cursor_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS note_body_backfill_cursor (
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
            "SELECT last_path FROM note_body_backfill_cursor WHERE id = 1",
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
        "INSERT OR REPLACE INTO note_body_backfill_cursor (id, last_path) VALUES (1, ?1)",
        params![last_path],
    )
    .map_err(|e| format!("cursor write: {}", e))?;
    Ok(())
}

/// Verify every note_meta row now has a note_body row, then stamp the version +
/// clear the cursor in one transaction. If any row is still missing (shouldn't
/// happen — the loop drained note_meta), DON'T stamp: the next boot re-runs and
/// the idempotent ON CONFLICT copy converges.
fn finalize(db: &Mutex<Option<Connection>>) -> Result<(), String> {
    let mut guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not initialized")?;

    let missing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_meta m
               LEFT JOIN note_body b ON b.path = m.path
             WHERE b.path IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    if missing != 0 {
        return Err(format!("completeness check failed: {} rows still missing in note_body", missing));
    }

    let tx = conn.transaction().map_err(|e| format!("finalize begin: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('note_body_backfill', ?1, strftime('%s','now'))",
        params![NOTE_BODY_BACKFILL_VERSION],
    )
    .map_err(|e| format!("finalize stamp: {}", e))?;
    tx.execute("DELETE FROM note_body_backfill_cursor", [])
        .map_err(|e| format!("finalize cursor: {}", e))?;
    tx.commit().map_err(|e| format!("finalize commit: {}", e))?;
    Ok(())
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

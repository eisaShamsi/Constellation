//! MIG-001 Step 5 — Resumable back-fill populator for sky_nodes / sky_links.
//!
//! Triggers (Steps 3+4) keep the tables in lock-step with live writes to
//! note_meta / note_links. But on first boot after the migration lands,
//! existing notes — 7,294 on the target universe — have no rows in
//! sky_nodes, and their 217k links have no rows in sky_links. This module
//! walks those tables and populates the derived surfaces.
//!
//! Design constraints (from MIG-001 Phase 1):
//!
//! - **Must not block boot.** Runs on a background thread scheduled by
//!   `ensure_search_db_ready` after the connection is live. First paint
//!   happens before we start.
//! - **Must be resumable.** `sky_backfill_cursor` holds the last
//!   processed path. Killing the app mid-run and relaunching resumes
//!   from the cursor, not from scratch.
//! - **Must not OOM.** 1,000-row batches, each in its own BEGIN IMMEDIATE
//!   transaction. WAL flushes at COMMIT. Prior LL-XXX custom-index OOM
//!   +3GB WAL vacuum is the warning.
//! - **Must coexist with live writes.** Per-batch lock release lets user
//!   saves and other IPC calls interleave between batches. A short
//!   inter-batch sleep keeps the backfill from starving the main thread
//!   on cheap notes.
//! - **Idempotent.** `INSERT OR IGNORE` — paths already inserted via
//!   triggers (user created a note during back-fill) are skipped without
//!   error.
//!
//! Completion stamps `schema_versions.sky = SKY_SCHEMA_VERSION`. Next
//! boot detects the stamp and skips the back-fill.

use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

use crate::search::{SearchState, SKY_SCHEMA_VERSION};

/// Batch size for each transaction. Tuned for:
/// - ~1-2 ms per note in the hot path (trigger-free bulk insert)
/// - Transaction fsync amortized across 1000 rows
/// - Enough breathing room between batches for user writes
const BATCH_SIZE: usize = 1000;

/// Sleep between batches. Gives the DB mutex to other callers. Keeps the
/// back-fill from saturating WAL during startup on large universes.
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// Schedule the back-fill on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after init_db completes and the
/// connection is in state. Silent no-op if the schema_versions.sky stamp
/// is already current.
pub fn maybe_schedule(app: tauri::AppHandle) {
    // Check quickly on the main thread whether we need to do anything at
    // all. Avoids spawning a thread for the common case (already current).
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

    // Clone the AppHandle into the thread. AppHandle is Clone and cheap.
    let app_bg = app.clone();
    thread::spawn(move || {
        match run(&app_bg) {
            Ok(n) => {
                diag(&app_bg, &format!("[sky_backfill] completed: {} notes populated", n));
            }
            Err(e) => {
                diag(&app_bg, &format!("[sky_backfill] FAILED: {}", e));
            }
        }
    });
}

/// True when the sky_* tables need back-filling. Either (a) the version
/// stamp is below target, or (b) there's a cursor row indicating a prior
/// run was interrupted.
fn is_needed(conn: &Connection) -> bool {
    let stored_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'sky'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    stored_version < SKY_SCHEMA_VERSION
}

/// The back-fill loop. Takes the app handle so we can re-lock the DB
/// mutex per batch. Returns the number of notes processed.
fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    let state = app.state::<SearchState>();

    // One-time setup: ensure the cursor table exists. Idempotent.
    {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        ensure_cursor_table(conn)?;
    }

    let mut last_path = read_cursor(&state.db)?;
    let mut total: u64 = 0;

    loop {
        let (batch_count, new_last_path) = process_batch(&state.db, &last_path)?;
        if batch_count == 0 {
            // Drained. Stamp the version and wipe the cursor row.
            finalize(&state.db)?;
            return Ok(total);
        }
        total += batch_count as u64;
        last_path = new_last_path;
        write_cursor(&state.db, &last_path)?;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

fn ensure_cursor_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sky_backfill_cursor (
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
            "SELECT last_path FROM sky_backfill_cursor WHERE id = 1",
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
        "INSERT OR REPLACE INTO sky_backfill_cursor (id, last_path) VALUES (1, ?1)",
        params![last_path],
    )
    .map_err(|e| format!("cursor write: {}", e))?;
    Ok(())
}

/// One transactional batch. Reads up to BATCH_SIZE rows from note_meta
/// beyond `after_path`, inserts corresponding sky_nodes rows, then
/// inserts sky_links rows for any note_links whose source_path lies in
/// the same batch window. `INSERT OR IGNORE` makes it idempotent — rows
/// populated by triggers during a concurrent write don't cause errors.
fn process_batch(
    db: &Mutex<Option<Connection>>,
    after_path: &str,
) -> Result<(usize, String), String> {
    let mut guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not initialized")?;

    let tx = conn.transaction().map_err(|e| format!("begin: {}", e))?;

    // Pull the next window of notes, ordered by path so the cursor
    // advances deterministically.
    let mut paths: Vec<(String, String, String)> = Vec::with_capacity(BATCH_SIZE);
    {
        let mut stmt = tx
            .prepare(
                "SELECT path, name, library_name
                 FROM note_meta
                 WHERE path > ?1
                 ORDER BY path
                 LIMIT ?2",
            )
            .map_err(|e| format!("prepare nodes: {}", e))?;
        let rows = stmt
            .query_map(params![after_path, BATCH_SIZE as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| format!("query nodes: {}", e))?;
        for r in rows {
            paths.push(r.map_err(|e| format!("row nodes: {}", e))?);
        }
    }

    if paths.is_empty() {
        tx.commit().map_err(|e| format!("commit empty: {}", e))?;
        return Ok((0, after_path.to_string()));
    }

    let last_path = paths.last().map(|p| p.0.clone()).unwrap_or_default();

    // Insert sky_nodes. OR IGNORE in case a trigger already populated
    // the path during this batch window.
    {
        let mut ins = tx
            .prepare(
                "INSERT OR IGNORE INTO sky_nodes
                    (path, id, name, library_name, updated_at)
                 VALUES (?1, LOWER(?2), ?2, ?3, strftime('%s','now'))",
            )
            .map_err(|e| format!("prepare ins node: {}", e))?;
        for (p, name, lib) in &paths {
            ins.execute(params![p, name, lib])
                .map_err(|e| format!("exec ins node: {}", e))?;
        }
    }

    // Insert sky_links for any active note_links whose source_path lies
    // in this batch window. Bounded by [after_path, last_path] so we
    // don't re-scan the whole note_links table on each batch.
    {
        tx.execute(
            "INSERT OR IGNORE INTO sky_links (source_path, target_name, link_type, weight)
             SELECT source_path, target_name, link_type, COALESCE(weight, 1.0)
             FROM note_links
             WHERE status = 'active'
               AND source_path > ?1
               AND source_path <= ?2",
            params![after_path, last_path.clone()],
        )
        .map_err(|e| format!("ins links: {}", e))?;
    }

    // MIG-002: back-fill word_count + created_at on note_meta for each
    // path in this batch. Only touches rows where word_count = 0 (fresh
    // column default) AND/OR created_at IS NULL so that rows already
    // stamped by the writer stay put — idempotent, safe to re-run.
    //
    // Cost: one file read per row in the batch. Bounded by BATCH_SIZE
    // and amortized by the inter-batch sleep. On the 7.6k-note target
    // universe: ~8 batches × up to 1000 file reads each, interleaved.
    {
        let mut upd = tx
            .prepare(
                "UPDATE note_meta
                    SET word_count = ?1,
                        created_at = COALESCE(created_at, ?2)
                  WHERE path = ?3
                    AND (word_count = 0 OR created_at IS NULL)",
            )
            .map_err(|e| format!("prepare upd word_count: {}", e))?;
        for (p, _, _) in &paths {
            let (wc, created_at) = compute_word_count_and_created_at(Path::new(p));
            // `created_at` is Option<i64>. COALESCE keeps any existing
            // non-null DB value; the bound value only wins when the DB
            // side is NULL. None maps to NULL which COALESCE skips.
            upd.execute(params![wc, created_at, p])
                .map_err(|e| format!("exec upd word_count: {}", e))?;
        }
    }

    tx.commit().map_err(|e| format!("commit: {}", e))?;
    Ok((paths.len(), last_path))
}

/// Read a .md file and return (word_count, created_at_epoch_seconds).
/// Mirrors the writer-side stamping in `search::index_note` so back-
/// filled rows agree with newly-written rows to the byte.
///
/// - word_count = whitespace-separated token count of the body (post-
///   frontmatter strip). Matches body.split_whitespace().count() in
///   search::index_note.
/// - created_at = fs::metadata(path).created() epoch seconds. None when
///   the platform lacks a true creation timestamp (ReFS, FAT32, some
///   Linux filesystems); caller uses COALESCE to preserve the existing
///   DB value (which may already be stamped to `modified`).
///
/// A missing / unreadable file yields (0, None) — the UPDATE then
/// writes `word_count = 0` with COALESCE keeping any prior created_at.
fn compute_word_count_and_created_at(path: &Path) -> (i64, Option<i64>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return (0, None);
    };
    // Single source of truth for frontmatter slicing — search.rs owns
    // the strip shape so back-fill and writer agree byte-for-byte.
    let body = crate::search::body_after_frontmatter(&content);
    let wc = body.split_whitespace().count() as i64;
    let created_at: Option<i64> = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);
    (wc, created_at)
}

fn finalize(db: &Mutex<Option<Connection>>) -> Result<(), String> {
    let mut guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not initialized")?;
    // Wrap version stamp + cursor clear in one transaction so a crash
    // between them can't leave a completed back-fill with a live cursor
    // row (which would make the next boot think it was interrupted).
    let tx = conn.transaction().map_err(|e| format!("finalize begin: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('sky', ?1)",
        params![SKY_SCHEMA_VERSION],
    )
    .map_err(|e| format!("finalize stamp: {}", e))?;
    tx.execute("DELETE FROM sky_backfill_cursor", [])
        .map_err(|e| format!("finalize cursor: {}", e))?;
    tx.commit().map_err(|e| format!("finalize commit: {}", e))?;
    Ok(())
}

/// Write a line to the universe's diagnostics log. Thin wrapper around
/// search::diag_log — kept here so this module doesn't depend on the
/// search module's private helpers.
fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

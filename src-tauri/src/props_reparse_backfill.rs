//! One-off correction: re-parse `properties_json` for notes the old indexer mis-read.
//!
//! **What was wrong.** The indexer's key branch tested only "does this line contain a
//! colon?", so a YAML LIST ITEM whose text contains one was recorded as a property. A
//! note carrying
//!
//! ```yaml
//! notable_works:
//!   - "Mimesis: The Representation of Reality in Western Literature"
//! ```
//!
//! indexed a phantom property named `- "Mimesis`. Every book title with a subtitle did
//! it. The `.md` files were always correct — **only the index lied**, which is why it
//! went unnoticed until the Template Studio put those names on screen as "fields these
//! notes carry" (Boss, 2026-07-23).
//!
//! `search::parse_frontmatter` now skips list items, so no NEW pollution is written.
//! This clears what the old parser already stored.
//!
//! **Why re-parse rather than reindex.** A full reindex re-reads and re-tokenises every
//! note — minutes of work on a 7,802-note Universe, and it rewrites `body_text` and the
//! FTS rows, which were never wrong. The defect lives entirely in one column, and the
//! `.md` file is the source of truth for it, so re-parsing frontmatter from disk fixes
//! exactly what broke and touches nothing else. Same correction, a fraction of the cost
//! and the risk.
//!
//! Rule 8 shape: background after paint, batched, RESUMABLE via a cursor, version
//! stamped only on completion — so an interrupted run resumes instead of restarting.

use crate::search::SearchState;
use rusqlite::{params, Connection};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

/// Bump to re-run the correction on every Universe.
const PROPS_REPARSE_VERSION: i64 = 1;

const BATCH_SIZE: i64 = 200;
/// Hand the DB mutex back between batches so this never starves the UI.
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// Schedule on a background thread. Returns immediately; silent no-op once stamped.
pub fn maybe_schedule(app: tauri::AppHandle) {
    let state = app.state::<SearchState>();
    let needs_run = {
        let Ok(guard) = state.db.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        is_needed(conn)
    };
    if !needs_run {
        return;
    }

    let app_bg = app.clone();
    thread::spawn(move || match run(&app_bg) {
        Ok(n) => diag(&app_bg, &format!("[props_reparse] completed: {n} notes corrected")),
        Err(e) => diag(&app_bg, &format!("[props_reparse] FAILED (non-fatal): {e}")),
    });
}

fn is_needed(conn: &Connection) -> bool {
    let v: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'props_reparse'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    v < PROPS_REPARSE_VERSION
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(p) = crate::search::db_path(app) {
        crate::search::diag_log(&p, msg);
    }
}

fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    let state = app.state::<SearchState>();
    let mut cursor = String::new();
    let mut total: u64 = 0;

    loop {
        let (scanned, fixed, last) = process_batch(&state.db, &cursor)?;
        total += fixed;
        if scanned == 0 {
            stamp(&state.db)?;
            return Ok(total);
        }
        cursor = last;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

/// One batch: re-parse each note's frontmatter FROM DISK and rewrite the column only
/// when it actually differs. Returns `(scanned, fixed, last_path)`.
///
/// A note whose file is missing or unreadable is SKIPPED, not blanked — an unreadable
/// file is an unknown, and writing an empty property set for it would be exactly the
/// kind of silent loss this correction exists to undo.
fn process_batch(
    db: &Mutex<Option<Connection>>,
    after: &str,
) -> Result<(u64, u64, String), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT path, properties_json FROM note_meta \
             WHERE path > ?1 ORDER BY path LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = stmt
        .query_map(params![after, BATCH_SIZE], |r| {
            Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default()))
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    if rows.is_empty() {
        return Ok((0, 0, after.to_string()));
    }
    let last = rows.last().map(|(p, _)| p.clone()).unwrap_or_default();
    let scanned = rows.len() as u64;
    let mut fixed = 0u64;

    for (path, stored) in rows {
        // Only touch rows that carry the defect's signature — a key beginning `- `.
        // Cheap string test first so a clean Universe costs one scan and no file reads.
        if !stored.contains("\"- ") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        let (properties, _tags, _body) = crate::search::parse_frontmatter(&content);
        let Ok(fresh) = serde_json::to_string(&properties) else { continue };
        if fresh == stored {
            continue;
        }
        conn.execute(
            "UPDATE note_meta SET properties_json = ?1 WHERE path = ?2",
            params![fresh, path],
        )
        .map_err(|e| e.to_string())?;
        fixed += 1;
    }
    Ok((scanned, fixed, last))
}

/// Verify no row still carries the defect's signature, THEN stamp.
///
/// 2026-07-24 inspection. The cursor lives in memory while the connection under it
/// can be swapped: a universe switch mid-run (`invalidate_search_state` NULLs the
/// conn, `ensure_search_db_ready` installs the new one — easily inside the 50 ms
/// inter-batch sleep) leaves Universe A's cursor driving a scan of Universe B. B's
/// rows below that cursor are never examined, the drained scan then stamps B's
/// `schema_versions`, and because the stamp makes `is_needed` false B never gets its
/// own pass — its phantom properties stay wrong forever.
///
/// The mature sibling `note_body_backfill::finalize` guards exactly this with a
/// completeness check before stamping; the same shape applies here and is exact,
/// because the defect has a cheap SQL signature. If anything is still unconverted we
/// simply DON'T stamp — the next boot re-runs, and the pass is idempotent.
fn stamp(db: &Mutex<Option<Connection>>) -> Result<(), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;

    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_meta WHERE properties_json LIKE '%\"- %'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    if remaining != 0 {
        return Err(format!(
            "completeness check failed: {} row(s) still carry the phantom signature — not stamping, the next boot re-runs",
            remaining
        ));
    }

    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) \
         VALUES ('props_reparse', ?1, strftime('%s','now'))",
        params![PROPS_REPARSE_VERSION],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

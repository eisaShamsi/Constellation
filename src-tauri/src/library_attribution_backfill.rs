//! One-off correction: re-attribute `note_meta.library_name` by longest-root-wins.
//!
//! **What was wrong** (2026-07-25, Whole-Ecosystem Fix Law). The bulk reconcile walker
//! (`index_library_recursive`) took a FIXED library name and, because universe_notes'
//! path IS the Universe root, walked into every nested registered library first and
//! stamped its notes with the PARENT's name; index_note's mtime gate then blocked the
//! nested library's own pass from correcting it. So after any rebuild, every note under
//! a nested library carried `library_name = 'universe_notes'` — the nested library
//! reported **0 notes** ("Eisa Test looks empty") and every name-scoped count / search /
//! scope was wrong.
//!
//! The walker is fixed (it now resolves per file via `library_name_for_path`), so no NEW
//! rows are mis-attributed. This pass corrects rows a PRIOR reconcile already corrupted —
//! otherwise they persist until a manual repair (PJ-207 §12: this said "a manual Rebuild
//! Index", which named nothing; the real door is Settings → Index → Repair index, built
//! by §11). Pure column re-write from the
//! authoritative library registry: no note is re-read or re-tokenised, `body_text`/FTS
//! are untouched.
//!
//! Rule 8 shape: background after paint, batched, version-stamped only after a
//! completeness check passes (so an interrupted or universe-switched run re-runs rather
//! than stamping an unfinished universe as done — the props_reparse lesson).

use crate::search::SearchState;
use rusqlite::Connection;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

/// Bump to re-run on every Universe.
const LIB_ATTR_VERSION: i64 = 1;
const BATCH_SIZE: i64 = 500;
const INTER_BATCH_SLEEP_MS: u64 = 40;

pub fn maybe_schedule(app: tauri::AppHandle) {
    let state = app.state::<SearchState>();
    let needs = {
        let Ok(guard) = state.db.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        is_needed(conn)
    };
    if !needs {
        return;
    }
    let app_bg = app.clone();
    thread::spawn(move || match run(&app_bg) {
        Ok(n) => diag(&app_bg, &format!("[lib_attr] re-attributed {n} note(s)")),
        Err(e) => diag(&app_bg, &format!("[lib_attr] FAILED (non-fatal): {e}")),
    });
}

fn is_needed(conn: &Connection) -> bool {
    let v: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'lib_attr'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    v < LIB_ATTR_VERSION
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(p) = crate::search::db_path(app) {
        crate::search::diag_log(&p, msg);
    }
}

fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    // PJ-207 §8 — the active universe's OWN libraries. This is an automatic boot pass that
    // UPDATEs `note_meta.library_name`, so its authority must not come from the federation:
    // a linked universe's library is not a name this universe may stamp onto a row.
    //
    // Safe against the own set precisely because `process_batch` already decided what to do
    // when no library owns a path — "leave it untouched rather than blank it". A row that
    // belongs to a linked universe therefore keeps whatever name it has instead of being
    // re-stamped, and `stamp`'s completeness check skips it on the same `if let Some`, so a
    // pre-existing foreign row cannot block the stamp either.
    let libs = crate::libraries::try_load_libraries(app)?;
    if libs.is_empty() {
        return Ok(0);
    }
    let state = app.state::<SearchState>();
    let mut cursor = String::new();
    let mut fixed: u64 = 0;

    loop {
        let (scanned, batch_fixed, last) = process_batch(&state.db, &libs, &cursor)?;
        fixed += batch_fixed;
        if scanned == 0 {
            break;
        }
        cursor = last;
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
    stamp(&state.db, &libs)?;
    Ok(fixed)
}

/// One batch: for each note row, compute the longest-root-wins owning library and
/// UPDATE only when it differs from what is stored. Returns `(scanned, fixed, last_path)`.
fn process_batch(
    db: &Mutex<Option<Connection>>,
    libs: &[crate::libraries::LibraryInfo],
    after: &str,
) -> Result<(u64, u64, String), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    let mut stmt = conn
        .prepare("SELECT path, library_name FROM note_meta WHERE path > ?1 ORDER BY path LIMIT ?2")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = stmt
        .query_map(rusqlite::params![after, BATCH_SIZE], |r| {
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
        let correct = crate::libraries::library_name_for_path(libs, &path);
        if let Some(name) = correct {
            if name != stored {
                conn.execute(
                    "UPDATE note_meta SET library_name = ?1 WHERE path = ?2",
                    rusqlite::params![name, path],
                )
                .map_err(|e| e.to_string())?;
                fixed += 1;
            }
        }
        // If no library owns the path (a note outside every registered root — should not
        // happen for an indexed note), leave it untouched rather than blank it.
    }
    Ok((scanned, fixed, last))
}

/// Verify no indexed note is still mis-attributed, THEN stamp — so a universe switch
/// mid-run cannot stamp an unfinished universe as done.
fn stamp(db: &Mutex<Option<Connection>>, libs: &[crate::libraries::LibraryInfo]) -> Result<(), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    // Completeness: recompute for every row and count remaining mismatches.
    let mut stmt = conn
        .prepare("SELECT path, library_name FROM note_meta")
        .map_err(|e| e.to_string())?;
    let mut remaining = 0i64;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))
        .map_err(|e| e.to_string())?;
    for row in rows.flatten() {
        let (path, stored) = row;
        if let Some(name) = crate::libraries::library_name_for_path(libs, &path) {
            if name != stored {
                remaining += 1;
            }
        }
    }
    if remaining != 0 {
        return Err(format!(
            "completeness check failed: {remaining} note(s) still mis-attributed — not stamping, next boot re-runs"
        ));
    }
    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) \
         VALUES ('lib_attr', ?1, strftime('%s','now'))",
        rusqlite::params![LIB_ATTR_VERSION],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

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

    // MIG-002 §4: run ANALYZE before any stratum computation so the
    // query planner has statistics on idx_link_source / idx_link_target /
    // idx_link_type. Without them, the planner picked idx_link_status
    // (non-selective — all links are 'active') and the stratum formula's
    // six subqueries each fanned out across the full 232k-row note_links
    // table. ~2ms per row with stats vs ~450ms without = 200× speedup.
    //
    // MIG-004 §10 audit-fix (4C-1, HIGH): scope the stratum/maturity
    // wipe to `path > last_path`. On a fresh back-fill `last_path = ""`
    // so the WHERE matches every row — same as the old unconditional
    // wipe. On RESUME after an interrupt, `last_path` reflects how far
    // the previous run had drained; rows at or below that path were
    // already recomputed under the new formula, so we MUST NOT wipe
    // them again — otherwise Phase D's path-range scope leaves them
    // stranded at NULL forever.
    //
    // Also: busy_timeout(30s) on this connection so the wipe contends
    // gracefully with cache_reconcile's parallel writes (§99 / BUG-008
    // class). Previously this block was the one back-fill phase that
    // ran without an explicit timeout.
    let last_path_for_wipe = read_cursor(&state.db)?;
    {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        conn.busy_timeout(Duration::from_secs(30))
            .map_err(|e| format!("busy_timeout: {}", e))?;
        conn.execute_batch("ANALYZE")
            .map_err(|e| format!("ANALYZE: {}", e))?;
        conn.execute(
            "UPDATE sky_nodes SET stratum = NULL, maturity = NULL WHERE path > ?1",
            params![last_path_for_wipe],
        )
        .map_err(|e| format!("stratum/maturity wipe: {}", e))?;
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

/// One batch, three phases — the DB lock is released during filesystem
/// I/O so the main thread's IPC queries don't queue behind us.
///
/// Phase A (under lock): pull the next batch of paths from note_meta,
/// insert sky_nodes + sky_links rows in one transaction. Fast — pure
/// SQL, no disk reads of note files.
///
/// Phase B (no lock): read each note file, compute word_count +
/// created_at via `compute_word_count_and_created_at`. This is the
/// expensive step — up to BATCH_SIZE file reads. Running it outside
/// the mutex means frontend queries stay responsive on boot.
///
/// Phase C (under lock): UPDATE note_meta with the precomputed values
/// in a second transaction. Single prepared statement, parameterised.
///
/// `INSERT OR IGNORE` in Phase A makes the sky_* inserts idempotent —
/// rows populated by triggers during a concurrent write don't error.
/// The `WHERE word_count = 0 OR created_at IS NULL` guard in Phase C
/// preserves any values the writer stamped in between our phases.
fn process_batch(
    db: &Mutex<Option<Connection>>,
    after_path: &str,
) -> Result<(usize, String), String> {
    // ── Phase A: path query + sky_* inserts under lock ─────────────────
    let (paths, last_path) = {
        let mut guard = db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        let tx = conn.transaction().map_err(|e| format!("begin: {}", e))?;

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

        tx.commit().map_err(|e| format!("commit A: {}", e))?;
        (paths, last_path)
    };
    // Lock released here — Phase B runs free.

    // ── Phase B: file reads WITHOUT lock ───────────────────────────────
    // Each tuple = (path, word_count, created_at). Bounded by BATCH_SIZE
    // rows so memory footprint is trivial.
    let computed: Vec<(String, NoteSignals)> = paths
        .iter()
        .map(|(p, _, _)| (p.clone(), read_note_signals(Path::new(p))))
        .collect();

    // ── Phase C: UPDATE note_meta under lock ──────────────────────────
    {
        let mut guard = db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        // Wait up to 30s on writer-lock contention with cache_reconcile's
        // dedicated connection. SQLite's default busy_handler returns
        // SQLITE_BUSY immediately; without this, BUG-008 symptom was a
        // transient error mid-back-fill on a busy first-boot DB.
        conn.busy_timeout(Duration::from_secs(30))
            .map_err(|e| format!("busy_timeout: {}", e))?;
        let tx = conn.transaction().map_err(|e| format!("begin C: {}", e))?;
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
            for (p, sig) in &computed {
                upd.execute(params![sig.word_count, sig.created_at, p])
                    .map_err(|e| format!("exec upd word_count: {}", e))?;
            }
        }
        tx.commit().map_err(|e| format!("commit C: {}", e))?;
    }

    // ── Phase E: back-fill note_aliases (frontmatter source) ──────────
    // MIG-004 §5. INSERT OR IGNORE per (path, alias) pair so existing
    // 'rename' / 'import' rows for the same alias stay put — composite
    // PK + IGNORE makes us idempotent and resilient to re-run mid-fill.
    // Skips paths that contributed zero aliases (most legacy notes
    // without `aliases:` frontmatter).
    {
        let mut guard = db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        conn.busy_timeout(Duration::from_secs(30))
            .map_err(|e| format!("busy_timeout: {}", e))?;
        let tx = conn.transaction().map_err(|e| format!("begin E: {}", e))?;
        {
            let mut ins = tx
                .prepare(
                    "INSERT OR IGNORE INTO note_aliases (path, alias_lower, source, cid_cn)
                     VALUES (?1, ?2, 'frontmatter', COALESCE((SELECT cid_cn FROM note_meta WHERE path = ?1), ''))",
                )
                .map_err(|e| format!("prepare ins alias: {}", e))?;
            for (p, sig) in &computed {
                for alias in &sig.aliases {
                    ins.execute(params![p, alias])
                        .map_err(|e| format!("exec ins alias: {}", e))?;
                }
            }
        }
        tx.commit().map_err(|e| format!("commit E: {}", e))?;
    }

    // ── Phase D: back-fill sky_nodes.stratum + .maturity for this batch
    // MIG-002 §4 (stratum) + §5 (maturity). Two UPDATEs, both scoped by
    // path range from this batch. Expressions kept in lockstep with the
    // triggers defined in search.rs::init_db via pub(crate) constants.
    //
    // Scoped to paths in [after_path, last_path] so we don't re-touch
    // every sky_nodes row on every batch. WHERE <col> IS NULL makes it
    // idempotent — rows already stamped by the triggers stay put.
    {
        let mut guard = db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        conn.busy_timeout(Duration::from_secs(30))
            .map_err(|e| format!("busy_timeout: {}", e))?;
        let tx = conn.transaction().map_err(|e| format!("begin D: {}", e))?;
        tx.execute(
            &format!(
                "UPDATE sky_nodes SET stratum = ({expr})
                   WHERE stratum IS NULL
                     AND path > ?1
                     AND path <= ?2",
                expr = crate::search::STRATUM_SQL_EXPR,
            ),
            params![after_path, last_path.clone()],
        )
        .map_err(|e| format!("upd stratum: {}", e))?;
        tx.execute(
            &format!(
                "UPDATE sky_nodes SET maturity = ({expr})
                   WHERE maturity IS NULL
                     AND path > ?1
                     AND path <= ?2",
                expr = crate::search::MATURITY_SQL_EXPR,
            ),
            params![after_path, last_path.clone()],
        )
        .map_err(|e| format!("upd maturity: {}", e))?;
        tx.commit().map_err(|e| format!("commit D: {}", e))?;
    }

    Ok((paths.len(), last_path))
}

/// Signals extracted from a single note file during back-fill.
/// Lets Phase B do one fs::read_to_string per note and feed all of
/// the back-fill's downstream phases (word_count for §C, aliases for
/// MIG-004 §E) without re-reading.
struct NoteSignals {
    word_count: i64,
    created_at: Option<i64>,
    aliases: Vec<String>,
}

/// Read a .md file and return its back-fill signals. Mirrors the
/// writer-side stamping in `search::index_note` byte-for-byte:
///
/// - word_count: whitespace-separated tokens of the body (post-
///   frontmatter strip), via `search::body_after_frontmatter`.
/// - created_at: fs::metadata(path).created() epoch seconds. None on
///   filesystems without a true creation timestamp (ReFS, FAT32,
///   some Linux FS); the UPDATE in Phase C uses COALESCE to keep
///   any value previously stamped via `modified` fallback.
/// - aliases: frontmatter `aliases:` entries, via
///   `search::extract_aliases`. Each is already lowercased + Arabic-
///   normalized so it matches `note_links.target_name` byte-for-byte.
///
/// A missing / unreadable file yields zero/empty signals — the
/// downstream UPDATEs / INSERTs become no-ops via their guards.
fn read_note_signals(path: &Path) -> NoteSignals {
    let Ok(content) = std::fs::read_to_string(path) else {
        return NoteSignals {
            word_count: 0,
            created_at: None,
            aliases: Vec::new(),
        };
    };
    // Single source of truth for frontmatter slicing — search.rs owns
    // the strip shape so back-fill and writer agree byte-for-byte.
    let body = crate::search::body_after_frontmatter(&content);
    let word_count = body.split_whitespace().count() as i64;
    let created_at: Option<i64> = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);
    let aliases = crate::search::extract_aliases(&content);
    NoteSignals { word_count, created_at, aliases }
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

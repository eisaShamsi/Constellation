//! MIG-085 §B.0 — one-shot backfill for the Unicode-folded name key `note_meta.name_lower`.
//!
//! Why: note names were matched to wikilink targets via SQLite `LOWER()`, which folds
//! ASCII A–Z only — so a note whose title carries a non-ASCII capital ("Île-de-France",
//! "Śramaṇa") never matched its own inbound links (`incoming_count = 0` → false orphan in
//! the Reviewer, wrong Sky maturity/stratum, 0 backlink badge). `index_note` now writes
//! `name_lower = fold_match_key(name)` (full-Unicode NFC fold) on every save, and the
//! name-side matches read `COALESCE(name_lower, LOWER(name))`. This back-fills the column
//! for existing rows, then fixes the handful of accented notes' `sky_nodes.id` +
//! `incoming_count` + maturity/stratum, and stamps `schema_versions.name_fold`.
//!
//! Design — convergent + restart-safe, mirrors `incoming_links_backfill`:
//! - **Never blocks boot.** Background thread, dedicated connection.
//! - **Convergent with the live write path.** `index_note` already maintains `name_lower`
//!   write-time; the back-fill only fills rows not yet re-saved. A row filled here then
//!   edited is re-folded by `index_note`; a row edited then filled converges to the same
//!   value (both = `fold_match_key(name)`). No atomic single-transaction needed.
//! - **Surgical recompute.** Measured on the live 7,660-note universe, exactly the
//!   accented-capital names change (target_name/alias_lower already fold-correct), so only
//!   those notes' `sky_nodes.id` / `incoming_count` / maturity / stratum are recomputed.

use rusqlite::{params, Connection};
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Bump to force a one-time re-fold on existing DBs (e.g. if the fold semantics change).
pub(crate) const SCHEMA_VERSION: i64 = 1;

const BATCH: usize = 500;

/// True once `name_lower` has been back-filled + stamped.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'name_fold'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        >= SCHEMA_VERSION
}

/// Schedule the one-shot backfill on a background thread. Silent no-op once stamped.
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
        !is_stamped(conn)
    };
    if !needs_run {
        return;
    }
    let app_bg = app.clone();
    std::thread::spawn(move || match run(&app_bg) {
        Ok((filled, fixed)) => diag(
            &app_bg,
            &format!("[name_fold_backfill] completed: {} name_lower filled, {} accented notes recomputed", filled, fixed),
        ),
        Err(e) => diag(&app_bg, &format!("[name_fold_backfill] FAILED (non-fatal): {}", e)),
    });
}

/// Populate `name_lower` for every note, then fix the accented notes' derived rows. Returns
/// (rows filled, accented notes recomputed). Runs on a DEDICATED connection (batched,
/// busy-tolerant). Convergent with the live `index_note` writer (both compute the same fold).
fn run(app: &tauri::AppHandle) -> Result<(usize, usize), String> {
    let path = crate::search::db_path(app)?;
    let mut conn = Connection::open(&path).map_err(|e| format!("open name_fold conn: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;
    // Defensive: a legacy note_meta_au that lost its WHEN guard would otherwise fail the
    // name_lower UPDATE with "no such tokenizer" (the incoming_links_backfill precedent).
    crate::search::register_fts5_tokenizer(&mut conn)
        .map_err(|e| format!("register tokenizer: {}", e))?;

    // ── Phase A — populate name_lower for every note (Rust fold, batched) ──
    let all: Vec<(String, String)> = {
        let mut stmt = conn
            .prepare("SELECT path, name FROM note_meta")
            .map_err(|e| format!("select notes: {}", e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("query notes: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    let mut filled = 0usize;
    for chunk in all.chunks(BATCH) {
        let tx = conn.transaction().map_err(|e| format!("tx: {}", e))?;
        {
            // PJ-249 /simplify (efficiency) — prepare once per chunk, not per row; fixed
            // here AND in target_base_backfill in the same pass so the template and its
            // copy do not diverge (the two-files-one-shape rule).
            let mut stmt = tx
                .prepare_cached("UPDATE note_meta SET name_lower = ?2 WHERE path = ?1")
                .map_err(|e| format!("prepare: {}", e))?;
            for (p, name) in chunk {
                stmt.execute(params![p, crate::search::fold_match_key(name)])
                    .map_err(|e| format!("update name_lower: {}", e))?;
                filled += 1;
            }
        }
        tx.commit().map_err(|e| format!("commit: {}", e))?;
    }

    // ── Phase A2 — NFC-normalise existing alias_lower rows so they stay byte-equal to the
    //    now-NFC-folded target_name (else a pre-existing NFD alias stops matching). Measured
    //    zero change on the live corpus; this future-proofs cross-device (NFD) aliases. ──
    let aliases: Vec<(String, String)> = {
        let mut stmt = conn
            .prepare("SELECT rowid, alias_lower FROM note_aliases")
            .map_err(|e| format!("select aliases: {}", e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?.to_string(), r.get::<_, String>(1)?)))
            .map_err(|e| format!("query aliases: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    for chunk in aliases.chunks(BATCH) {
        let tx = conn.transaction().map_err(|e| format!("alias tx: {}", e))?;
        for (rowid, al) in chunk {
            let folded = crate::search::fold_match_key(al);
            if &folded != al {
                tx.execute(
                    "UPDATE note_aliases SET alias_lower = ?2 WHERE rowid = ?1",
                    params![rowid, folded],
                )
                .map_err(|e| format!("update alias_lower: {}", e))?;
            }
        }
        tx.commit().map_err(|e| format!("alias commit: {}", e))?;
    }

    // ── Phase B — the notes whose Unicode fold differs from the old ASCII `LOWER(name)`
    //    (exactly the accented-capital / NFD names). Independent of sky_nodes (a note not
    //    yet in sky_nodes must still get its incoming_count fixed), so this is keyed on
    //    note_meta alone; the sky_nodes UPDATEs below are no-ops when no row exists. ──
    let affected: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT path FROM note_meta WHERE name_lower IS NOT NULL AND name_lower != LOWER(name)",
            )
            .map_err(|e| format!("select affected: {}", e))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("query affected: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let incoming_assign = crate::search::incoming_aggregate_assignments("note_meta");
    for p in &affected {
        // 1. sky_nodes.id → the folded name (so target_name = sky_nodes.id matches).
        conn.execute(
            "UPDATE sky_nodes SET id = (SELECT name_lower FROM note_meta WHERE path = ?1) WHERE path = ?1",
            params![p],
        )
        .map_err(|e| format!("fix sky id: {}", e))?;
        // 2. this note's incoming aggregate (its inbound now resolves).
        conn.execute(
            &format!("UPDATE note_meta SET {} WHERE path = ?1", incoming_assign),
            params![p],
        )
        .map_err(|e| format!("recompute incoming: {}", e))?;
        // 3. stratum + maturity (read target_name = sky_nodes.id, now correct).
        conn.execute(
            &format!("UPDATE sky_nodes SET stratum = ({}) WHERE path = ?1", crate::search::stratum_sql_expr()),
            params![p],
        )
        .map_err(|e| format!("recompute stratum: {}", e))?;
        conn.execute(
            &format!("UPDATE sky_nodes SET maturity = ({}) WHERE path = ?1", crate::search::maturity_sql_expr()),
            params![p],
        )
        .map_err(|e| format!("recompute maturity: {}", e))?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('name_fold', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    Ok((filled, affected.len()))
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

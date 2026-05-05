//! CTSE first-fill (MIG-013 §1D).
//!
//! Walks every row of `note_meta` and re-fires
//! [`super::hooks::on_note_indexed`] with `old_body = None`, so an
//! existing library that pre-dates §1C populates its `term_vocab`
//! ledger (and the fast-path `bridge_concept_id` column) without
//! waiting for the user to edit notes one by one.
//!
//! ## Why a separate command
//!
//! - The per-save hook is the canonical maintenance path. First-fill
//!   piggybacks on it for correctness — same tokenizer, same delta
//!   semantics, same fast-path resolution.
//! - Bulk performance differs from steady-state: writing 7K+ notes
//!   into `term_vocab` benefits from batched transactions. The hook
//!   doesn't manage transactions itself; first-fill wraps each chunk
//!   of 50 notes in one explicit transaction.
//! - Cancellation must be granular (between notes, not just between
//!   batches) — hence the inner-loop cancel check.
//!
//! ## Resumability
//!
//! Each chunk commits in its own transaction. A mid-fill cancel keeps
//! the partial state intact; re-running this command iterates from
//! where it left off (the `term_vocab` row state is the implicit
//! cursor — already-applied tokens contribute idempotently because
//! `on_note_indexed` reads `note_meta.body_text` and sees no delta on
//! revisit *for nuotes already counted*… except first-fill always
//! passes `old_body = None`, which means re-running first-fill on the
//! same rows would double-count.
//!
//! Resolution: first-fill is gated by the frontend's
//! "term_vocab is empty" check (or an explicit user trigger). A
//! resume after cancellation instead routes through the regular
//! per-save hook on subsequent edits, which has the correct old/new
//! delta semantics. The unfilled tail is left for incremental
//! maintenance.
//!
//! ## Cost profile
//!
//! On Boss's 7,635-note library: ~5 ms tokenize per note + ~20 ms
//! commit per 50-note chunk = ~50 sec total. Acceptable as a one-shot
//! background job with a status-bar strip.

use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager};

use crate::embeddings::EmbeddingState;
use crate::search::SearchState;

/// Per-batch transaction size. Chosen to keep each commit short
/// (~250 ms tokenize + ~20 ms commit on a typical library) while
/// amortizing the BEGIN/COMMIT cost across enough rows to matter.
const CHUNK: usize = 50;

/// Progress payload emitted on `ctse-firstfill-progress`. Mirrors the
/// shape of [`super::backfill::CtseBackfillProgress`] for frontend
/// uniformity (same status-bar strip can render either stream by
/// switching the event name).
#[derive(Debug, Clone, Serialize)]
pub struct CtseFirstFillProgress {
    pub processed: u32,
    pub total: u32,
    pub done: bool,
    pub cancelled: bool,
}

fn emit(app: &tauri::AppHandle, payload: CtseFirstFillProgress) {
    let _ = app.emit("ctse-firstfill-progress", payload);
}

/// Pull all (path, body_text) rows. Skip empty bodies (no terms to
/// extract). One short-lived db lock; sub-second on Boss's library.
fn load_note_bodies(app: &tauri::AppHandle) -> Result<Vec<(String, String)>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;
    let mut stmt = conn
        .prepare("SELECT path, body_text FROM note_meta WHERE body_text != ''")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Report whether first-fill should run. Returns true iff `term_vocab`
/// is empty AND `note_meta` has at least one row with body content.
/// Frontend uses this on boot to decide whether to fire the command.
#[tauri::command]
pub fn ctse_first_fill_status(app: tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;
    let term_vocab_empty: bool = conn
        .query_row("SELECT COUNT(*) FROM term_vocab LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .map(|n| n == 0)
        .unwrap_or(true);
    if !term_vocab_empty {
        return Ok(false);
    }
    let has_notes: bool = conn
        .query_row(
            "SELECT 1 FROM note_meta WHERE body_text != '' LIMIT 1",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);
    Ok(has_notes)
}

/// First-fill the empty `term_vocab` from `note_meta.body_text`.
/// Re-fires `on_note_indexed(old=None, new=body)` per row, in chunks
/// of 50 wrapped in one transaction each. Resumable via the same
/// `term_embed_cancel` atomic that the backfill uses.
///
/// Idempotent only when called from a state where `term_vocab` is
/// empty — re-running on a partially-populated table double-counts.
/// Frontend gates this via [`ctse_first_fill_status`].
#[tauri::command]
pub fn ctse_first_fill(app: tauri::AppHandle) -> Result<(), String> {
    let embed_state = app.state::<EmbeddingState>();
    embed_state.term_embed_cancel.store(false, Ordering::SeqCst);

    let notes = load_note_bodies(&app)?;
    let total = notes.len() as u32;

    emit(
        &app,
        CtseFirstFillProgress {
            processed: 0,
            total,
            done: false,
            cancelled: false,
        },
    );

    if total == 0 {
        emit(
            &app,
            CtseFirstFillProgress {
                processed: 0,
                total: 0,
                done: true,
                cancelled: false,
            },
        );
        return Ok(());
    }

    let mut processed = 0u32;
    let mut cancelled = false;

    for chunk in notes.chunks(CHUNK) {
        if embed_state.term_embed_cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        // Open a fresh DB lock + transaction per chunk. Holding the
        // lock for an entire library would freeze every concurrent
        // reader; per-chunk release lets readers in between batches.
        {
            let state = app.state::<SearchState>();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let conn = db.as_ref().ok_or("Search database not initialized")?;
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            for (path, body) in chunk {
                if embed_state.term_embed_cancel.load(Ordering::SeqCst) {
                    cancelled = true;
                    break;
                }
                // `&tx` auto-derefs to `&Connection` via Transaction's
                // Deref impl, so the hook signature is unchanged.
                if let Err(e) = super::hooks::on_note_indexed(&tx, path, None, body) {
                    eprintln!("[ctse-firstfill] hook failed for {path:?}: {e}");
                }
                processed = processed.saturating_add(1);
            }
            tx.commit().map_err(|e| e.to_string())?;
        }
        emit(
            &app,
            CtseFirstFillProgress {
                processed,
                total,
                done: false,
                cancelled: false,
            },
        );
        if cancelled {
            break;
        }
    }

    emit(
        &app,
        CtseFirstFillProgress {
            processed,
            total,
            done: true,
            cancelled,
        },
    );
    Ok(())
}

/// Cancel an in-flight first-fill. Idempotent. Reuses the same atomic
/// flag as the backfill (only one of the two runs at a time in
/// practice — frontend serializes them).
#[tauri::command]
pub fn ctse_cancel_first_fill(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<EmbeddingState>();
    state.term_embed_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

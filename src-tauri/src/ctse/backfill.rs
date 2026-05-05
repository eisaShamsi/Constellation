//! CTSE slow-path backfill (MIG-013 §1C).
//!
//! Resolves `term_vocab.bridge_concept_id` for every row currently
//! NULL. Two failure modes the row can land in after we're done:
//!
//! - **resolved** — a real `c:…` M11 concept id (fast-path hit during
//!   the multi-lang FST sweep, or slow-path hit above the cosine
//!   threshold).
//! - **sentinel** — the literal `'-'` string. Means "we tried both
//!   paths and neither cleared the bar; do not try again." Distinct
//!   from NULL so the backfill is idempotent: re-running visits only
//!   genuinely-new NULL rows, never re-attempts known-misses.
//!
//! ## Cancellation
//!
//! Reuses [`crate::embeddings::EmbeddingState::term_embed_cancel`]
//! (orphaned by the §1C-5 retirement of the old term-embedding job).
//! The worker checks the flag at every batch boundary; on cancel it
//! emits a final `done: true, cancelled: true` event and returns.
//! Already-resolved rows stay resolved.
//!
//! ## Resumability
//!
//! Each batch commits in its own transaction. App close mid-fill →
//! next launch's `ctse_run_backfill` invocation walks the still-NULL
//! tail. No checkpoint table required; the NULL filter IS the cursor.
//!
//! ## Cost profile
//!
//! Slow-path resolution is ~50 ms per term (one e5 ONNX inference +
//! a 20K × 384 cosine sweep). Fast-path resolution adds ~7 µs (15 FST
//! queries). On Boss's 7,635-note library the term_vocab will grow
//! to a few tens of thousands of unique terms over time; backfill
//! lands in the ~tens of minutes range, fully usable in the background
//! while the user works.

use rusqlite::params;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager};

use crate::embeddings::EmbeddingState;
use crate::search::SearchState;

/// Sentinel placed in `bridge_concept_id` for terms that the slow path
/// has tried and failed to resolve. Prevents re-attempting the same
/// term across runs. Distinct from NULL ("never tried"); distinct
/// from any real concept id (they all start with `"c:"`).
pub const SENTINEL_TRIED_NO_HIT: &str = "-";

/// Per-batch transaction size. Small enough to keep transaction lock
/// time bounded (~25 sec at 50 ms/term), large enough to amortize
/// `db.lock()` overhead across many UPDATEs.
const BATCH_SIZE: usize = 500;

/// Progress payload emitted on the `ctse-backfill-progress` Tauri
/// event. Frontend (status-bar strip in §1D) subscribes and renders
/// `processed / total` plus the done/cancelled flags.
#[derive(Debug, Clone, Serialize)]
pub struct CtseBackfillProgress {
    pub processed: u32,
    pub total: u32,
    pub done: bool,
    pub cancelled: bool,
}

fn emit(app: &tauri::AppHandle, payload: CtseBackfillProgress) {
    let _ = app.emit("ctse-backfill-progress", payload);
}

/// Pull the count of NULL rows that the backfill will actually process.
/// Filters out bigrams (joined by `BIGRAM_SEP` = U+001F / CHAR(31)) —
/// those are not lexicon-resolvable and are bulk-sentinelled by the
/// schema-version 2 migration in `init_db`. Belt-and-suspenders: if a
/// new note save introduces a fresh bigram between migrations, we
/// still skip it here.
fn count_null_rows(app: &tauri::AppHandle) -> Result<u32, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;
    let n: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM term_vocab \
             WHERE bridge_concept_id IS NULL \
               AND term NOT LIKE '%' || CHAR(31) || '%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(n)
}

/// Pull the next batch of unresolved terms. TF-IDF descending
/// (rarest first → search becomes useful early in the backfill, since
/// rare terms carry the most discriminative signal). Bigram rows are
/// excluded — see [`count_null_rows`].
fn next_batch(app: &tauri::AppHandle, limit: usize) -> Result<Vec<String>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;
    let mut stmt = conn
        .prepare(
            "SELECT term FROM term_vocab \
             WHERE bridge_concept_id IS NULL \
               AND term NOT LIKE '%' || CHAR(31) || '%' \
             ORDER BY total_count ASC, term \
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit as i64], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Apply a batch of (term, concept_id_or_sentinel) updates inside one
/// transaction. The concept_id slot is never NULL on output — every
/// processed term lands as either a real c: id or the sentinel.
fn write_batch(app: &tauri::AppHandle, results: &[(String, String)]) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("UPDATE term_vocab SET bridge_concept_id = ?1 WHERE term = ?2")
            .map_err(|e| e.to_string())?;
        for (term, value) in results {
            stmt.execute(params![value, term])
                .map_err(|e| format!("term_vocab UPDATE failed for {term:?}: {}", e))?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Run the slow-path backfill. Idempotent: re-invoking after a
/// successful run is a no-op (zero NULL rows). Re-invoking after a
/// cancellation resumes from the still-NULL tail.
///
/// Synchronous: the caller (Tauri runtime) spawns this on the IPC
/// thread pool. The frontend renders progress via the
/// `ctse-backfill-progress` event stream — no need to poll.
#[tauri::command]
pub fn ctse_run_backfill(app: tauri::AppHandle) -> Result<(), String> {
    // Reset cancel flag at start; `ctse_cancel_backfill` flips it true
    // to terminate the loop gracefully.
    let embed_state = app.state::<EmbeddingState>();
    embed_state.term_embed_cancel.store(false, Ordering::SeqCst);

    let total = count_null_rows(&app)?;
    let mut processed = 0u32;
    emit(
        &app,
        CtseBackfillProgress {
            processed,
            total,
            done: false,
            cancelled: false,
        },
    );

    if total == 0 {
        emit(
            &app,
            CtseBackfillProgress {
                processed: 0,
                total: 0,
                done: true,
                cancelled: false,
            },
        );
        return Ok(());
    }

    loop {
        if embed_state.term_embed_cancel.load(Ordering::SeqCst) {
            emit(
                &app,
                CtseBackfillProgress {
                    processed,
                    total,
                    done: true,
                    cancelled: true,
                },
            );
            return Ok(());
        }

        let batch = next_batch(&app, BATCH_SIZE)?;
        if batch.is_empty() {
            break;
        }

        let mut results: Vec<(String, String)> = Vec::with_capacity(batch.len());
        for term in &batch {
            // Sample cancel flag inside the inner loop too — a 500-term
            // batch at 50ms/term is 25 sec; we don't want a Boss-cancel
            // to wait that long.
            if embed_state.term_embed_cancel.load(Ordering::SeqCst) {
                break;
            }
            let resolved = match super::resolve_term_multilang(&app, term) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[ctse-backfill] resolve failed for {term:?}: {}", e);
                    None
                }
            };
            let value = resolved.unwrap_or_else(|| SENTINEL_TRIED_NO_HIT.to_string());
            results.push((term.clone(), value));
        }

        write_batch(&app, &results)?;
        processed = processed.saturating_add(results.len() as u32);
        emit(
            &app,
            CtseBackfillProgress {
                processed,
                total,
                done: false,
                cancelled: false,
            },
        );

        // If the inner loop bailed early due to cancellation, the next
        // outer-loop iteration will see the flag and emit the final
        // cancelled event.
    }

    emit(
        &app,
        CtseBackfillProgress {
            processed,
            total: total.max(processed),
            done: true,
            cancelled: false,
        },
    );
    Ok(())
}

/// Request that the running backfill stop at the next safe point
/// (between batches, or between terms within a batch). Idempotent:
/// calling when no backfill is running is a no-op.
#[tauri::command]
pub fn ctse_cancel_backfill(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<EmbeddingState>();
    state.term_embed_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

/// Report unresolved-row count (`bridge_concept_id IS NULL`). Frontend
/// uses this to decide whether to auto-fire `ctse_run_backfill` on
/// boot (count > 0) or skip it (count == 0). Cheap.
#[tauri::command]
pub fn ctse_backfill_status(app: tauri::AppHandle) -> Result<u32, String> {
    count_null_rows(&app)
}

//! MIG-021v2 §1F'.b — Bulk approve/reject for the Source Review queue.
//!
//! After the §1F' background scan, a fresh Universe ends up with
//! thousands of pending suggestions. Reviewing each one by hand is
//! impractical for a large vault — the user needs a "clear the queue"
//! affordance.
//!
//! Two operations:
//!   * `sources_accept_all_pending` — for every pending record, write
//!     ALL of its top-3 suggestions per axis to the note's frontmatter
//!     and clear the suggestion row. Mirrors the per-card Accept's
//!     semantics exactly. Runs on a background thread; emits
//!     `sources:bulk_accept` events with phase = start / progress /
//!     done / cancelled / error.
//!   * `sources_reject_all_pending` — single SQL DELETE; instant.
//!     Returns the count of rows cleared so the UI can show a toast.
//!
//! Cooperative cancellation via an AtomicBool checked between records.
//! Per-record errors are logged but don't abort the loop — one corrupt
//! note shouldn't block clearing the other 6,000.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

use super::{
    clear_suggestions, extract_content_type, extract_sources, is_valid_content_type_id,
    is_valid_source_id, read_suggestions, rewrite_frontmatter_content_type,
    rewrite_frontmatter_sources, union_preserve_order, write_content_type_to_db,
    write_sources_to_db, Suggestion,
};

#[derive(Default)]
pub struct BulkAcceptState {
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    pub completed: AtomicUsize,
    pub total: AtomicUsize,
    pub last_error: Mutex<Option<String>>,
}

impl BulkAcceptState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Clone)]
pub struct BulkAcceptStatus {
    pub running: bool,
    pub cancelling: bool,
    pub completed: usize,
    pub total: usize,
    pub last_error: Option<String>,
}

#[derive(Serialize, Clone)]
struct BulkAcceptEvent {
    phase: String,
    total: usize,
    completed: usize,
    error: Option<String>,
}

/// Snapshot the bulk-accept state. Called by the frontend on mount or
/// after a missed event.
#[tauri::command]
pub fn sources_bulk_accept_status(app: AppHandle) -> BulkAcceptStatus {
    let state = app.state::<BulkAcceptState>();
    BulkAcceptStatus {
        running: state.running.load(Ordering::Relaxed),
        cancelling: state.cancel.load(Ordering::Relaxed),
        completed: state.completed.load(Ordering::Relaxed),
        total: state.total.load(Ordering::Relaxed),
        last_error: state
            .last_error
            .lock()
            .ok()
            .and_then(|g| g.clone()),
    }
}

/// Set the cancel flag on a running bulk-accept. Returns immediately;
/// the worker thread observes it on its next iteration.
#[tauri::command]
pub fn sources_bulk_accept_cancel(app: AppHandle) -> Result<(), String> {
    let state = app.state::<BulkAcceptState>();
    state.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

/// Kick off bulk-accept on every pending suggestion in the queue.
/// Idempotent: returns an error if already running.
///
/// Each record is committed exactly the way per-card Accept does:
/// `sources:` gets every horizontal suggestion ID and `content_type:`
/// gets every vertical suggestion ID. The user can subsequently trim
/// via the PropertyEditor pickers.
///
/// V3-§8.r5.5 (audit UX agent): added `skip_split` parameter (default
/// true from the frontend). When true, cards whose composite_json
/// reports a Split regime on either axis are EXCLUDED from the bulk
/// accept — the engine "refused to assign" on those cards and
/// auto-applying the top suggestion would defeat the Sibling
/// Disambiguation design. They stay in the queue for the user to
/// resolve via the radio-chip form.
// App-freeze audit Batch-D (2026-07-03): ensure_search_db_ready moved INSIDE
// the spawned worker (it parked the dispatch thread for the whole cold init);
// the worker's error path covers an ensure failure.
#[tauri::command]
pub fn sources_accept_all_pending(app: AppHandle, skip_split: Option<bool>) -> Result<(), String> {
    let state = app.state::<BulkAcceptState>();
    if state.running.swap(true, Ordering::Relaxed) {
        return Err("Bulk accept already running".into());
    }
    state.cancel.store(false, Ordering::Relaxed);
    state.completed.store(0, Ordering::Relaxed);
    state.total.store(0, Ordering::Relaxed);
    if let Ok(mut g) = state.last_error.lock() {
        *g = None;
    }

    let skip_split_flag = skip_split.unwrap_or(true);
    let app_clone = app.clone();
    thread::spawn(move || {
        let result = crate::search::ensure_search_db_ready(&app_clone)
            .and_then(|_| run_bulk_accept(app_clone.clone(), skip_split_flag));
        let state = app_clone.state::<BulkAcceptState>();
        if let Err(e) = result {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(e.clone());
            }
            let _ = app_clone.emit(
                "sources:bulk_accept",
                BulkAcceptEvent {
                    phase: "error".into(),
                    total: state.total.load(Ordering::Relaxed),
                    completed: state.completed.load(Ordering::Relaxed),
                    error: Some(e),
                },
            );
        }
        state.running.store(false, Ordering::Relaxed);
        state.cancel.store(false, Ordering::Relaxed);
    });

    Ok(())
}

fn run_bulk_accept(app: AppHandle, skip_split: bool) -> Result<(), String> {
    // Snapshot the queue once up front so we don't race against new
    // entries being added (e.g. a scan still running in the background).
    // V3-§8.r5.5: when skip_split is true, also pull composite_json so
    // we can filter out Split-regime cards in Rust.
    let pending_paths: Vec<String> = {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard
            .as_ref()
            .ok_or("Search database not initialized")?;
        let sql = if skip_split {
            "SELECT note_path, composite_json FROM sources_suggestions ORDER BY created_at ASC"
        } else {
            "SELECT note_path, NULL FROM sources_suggestions ORDER BY created_at ASC"
        };
        let mut stmt = conn.prepare(sql).map_err(|e| format!("prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1).ok().flatten(),
                ))
            })
            .map_err(|e| format!("query: {}", e))?;
        let mut out = Vec::new();
        for r in rows {
            let (path, composite_json) = r.map_err(|e| format!("row: {}", e))?;
            if skip_split {
                if let Some(json) = &composite_json {
                    if has_split_regime(json) {
                        continue; // skip Split-regime cards
                    }
                }
            }
            out.push(path);
        }
        out
    };

    let total = pending_paths.len();
    let state = app.state::<BulkAcceptState>();
    state.total.store(total, Ordering::Relaxed);

    let _ = app.emit(
        "sources:bulk_accept",
        BulkAcceptEvent {
            phase: "start".into(),
            total,
            completed: 0,
            error: None,
        },
    );

    if total == 0 {
        let _ = app.emit(
            "sources:bulk_accept",
            BulkAcceptEvent {
                phase: "done".into(),
                total: 0,
                completed: 0,
                error: None,
            },
        );
        return Ok(());
    }

    let mut completed: usize = 0;
    for note_path in pending_paths {
        if state.cancel.load(Ordering::Relaxed) {
            let _ = app.emit(
                "sources:bulk_accept",
                BulkAcceptEvent {
                    phase: "cancelled".into(),
                    total,
                    completed,
                    error: None,
                },
            );
            return Ok(());
        }

        if let Err(e) = accept_one(&app, &note_path) {
            // Record but don't abort.
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(format!("{}: {}", note_path, e));
            }
        }

        completed += 1;
        state.completed.store(completed, Ordering::Relaxed);

        // Throttle event emission per Performance Rule 3.
        if completed % 5 == 0 || completed == total {
            let _ = app.emit(
                "sources:bulk_accept",
                BulkAcceptEvent {
                    phase: "progress".into(),
                    total,
                    completed,
                    error: None,
                },
            );
        }
    }

    let _ = app.emit(
        "sources:bulk_accept",
        BulkAcceptEvent {
            phase: "done".into(),
            total,
            completed,
            error: None,
        },
    );
    Ok(())
}

fn accept_one(app: &AppHandle, note_path: &str) -> Result<(), String> {
    // 1. Read the suggestion record from the queue.
    let record_opt = {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard
            .as_ref()
            .ok_or("Search database not initialized")?;
        read_suggestions(conn, note_path)?
    };
    let record = match record_opt {
        Some(r) => r,
        None => return Ok(()), // already cleared by another path; nothing to do
    };

    // 2. Split by axis + validate (defense-in-depth: strip any IDs the
    //    classifier emitted that aren't in the current taxonomy).
    let horizontal_ids: Vec<String> = record
        .suggestions
        .iter()
        .filter(|s: &&Suggestion| s.axis == "horizontal" && is_valid_source_id(&s.source))
        .map(|s| s.source.clone())
        .collect();
    let vertical_ids: Vec<String> = record
        .suggestions
        .iter()
        .filter(|s: &&Suggestion| s.axis == "vertical" && is_valid_content_type_id(&s.source))
        .map(|s| s.source.clone())
        .collect();

    // 3. Rewrite BOTH axes' frontmatter as ONE locked read-modify-write (gate_rmw), so a concurrent
    //    editor save can land before or after but NEVER inside the window. PJ-071: the bulk path was
    //    the last source-accept still on the racy unlocked-read + gate_write — the per-card path
    //    already moved to gate_rmw (sources/mod.rs::rewrite_note_sources_on_disk). The closure is
    //    pure string work: NO gate_* and NO DB lock inside it (gate_rmw's two hard rules) — the
    //    SQLite-mirror update is step 4, after this returns. Idempotent rewrite → Ok(None), no write.
    let path = std::path::Path::new(note_path);
    if !path.exists() {
        return Err(format!("Note not found: {}", note_path));
    }
    // PJ-091: accept ENRICHES, it must not SUBTRACT. Union each axis's suggestion
    // with the note's CURRENT on-disk values (read inside the gate — race-free),
    // so Approve-All never drops a source/type the user set by hand after this
    // suggestion was queued. `merged_*` are stashed for the step-4 DB mirror so
    // note_meta reflects exactly what landed on disk.
    let mut merged_h = Vec::new();
    let mut merged_v = Vec::new();
    crate::write_gate::gate_rmw(path, "bulk_accept", |content| {
        merged_h = union_preserve_order(&extract_sources(content), &horizontal_ids);
        merged_v = union_preserve_order(&extract_content_type(content), &vertical_ids);
        let after_h = rewrite_frontmatter_sources(content, &merged_h);
        let after_both = rewrite_frontmatter_content_type(&after_h, &merged_v);
        Ok(if after_both == content { None } else { Some(after_both) })
    })?;

    // 4. Update the SQLite mirror + clear the suggestion row.
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    write_sources_to_db(conn, note_path, &merged_h)?;
    write_content_type_to_db(conn, note_path, &merged_v)?;
    clear_suggestions(conn, note_path)?;

    Ok(())
}

/// V3-§8.r5.5: Check if a composite_json blob reports a Split regime
/// on either axis. Defensive: malformed/missing JSON returns false
/// (don't skip cards we can't read — better to bulk-accept than to
/// silently leave them in the queue forever).
fn has_split_regime(composite_json: &str) -> bool {
    let value: serde_json::Value = match serde_json::from_str(composite_json) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let regime_is_split = |key: &str| -> bool {
        value
            .get(key)
            .and_then(|v| v.get("regime"))
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("Split"))
            .unwrap_or(false)
    };
    regime_is_split("horizontal") || regime_is_split("vertical")
}

/// Clear EVERY pending suggestion from the queue without writing to
/// any frontmatter. Synchronous + instant; returns the count of rows
/// cleared so the UI can show a toast.
///
/// Reversible in the loose sense that the user can re-run the
/// classifier scan to regenerate suggestions; not reversible in the
/// strict sense (the originally-suggested IDs are gone).
// Note-open-freeze Batch-2 §B2-2 (2026-07-03): `(async)` — off the IPC dispatch thread.
// Discovery-verified async-only-safe: DB-only / mutex-covered body, no note-file writes,
// all callers await. See SESSION-LOG-2026-07-03 (Architect findings).
#[tauri::command(async)]
pub fn sources_reject_all_pending(app: AppHandle) -> Result<usize, String> {
    crate::search::ensure_search_db_ready(&app)?;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    let n = conn
        .execute("DELETE FROM sources_suggestions", [])
        .map_err(|e| format!("delete: {}", e))?;
    Ok(n)
}

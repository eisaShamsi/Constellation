//! Maturity Lifecycle — Cognitive Engine Phase 3.
//!
//! Tracks note growth through 5 maturity states, computed from structural signals.
//! No manual tagging. States derived from inbound link count + file age.
//!
//! States:
//!   🌱 seed       — 0 inbound links, modified ≤1 day after creation
//!   🌿 sapling    — 1–3 inbound links OR modified 2+ days after creation
//!   🌳 evergreen  — 4+ inbound links AND modified 7+ days after creation
//!   ⭐ canonical  — 10+ inbound links AND last modified 30+ days ago
//!   🥀 wilting    — evergreen but untouched 90+ days
//!
//! MIG-085 §B.1 — single-sourced inbound. This panel reads the write-time
//! `note_meta.incoming_count` (DISTINCT source notes, alias-aware, Unicode-folded via
//! MIG-085 §B.0) and the same `compute_state` thresholds the Reviewer uses
//! (`review::maturity_label`) and the Sky trigger uses (`MATURITY_SQL_EXPR`), so maturity
//! reads identically on every surface. This also removes the prior full-filesystem walk
//! (a Rule-8 violation): the panel is now a single indexed `note_meta` read.

use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

/// Per-note maturity result returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct NoteMaturity {
    pub note_path: String,
    pub note_name: String,
    pub state: String,              // "seed" | "sapling" | "evergreen" | "canonical" | "wilting"
    pub inbound_count: usize,
    pub days_since_modified: u64,
}

/// Compute the maturity state for every note in a library — a pure `note_meta` read.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn compute_note_maturity(
    app: tauri::AppHandle,
    library_path: String,
    _library_name: String,
) -> Result<Vec<NoteMaturity>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let state = app.state::<crate::search::SearchState>();
    let guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    // Scope to the library by path prefix, matching the Reviewer's `scope_clause`
    // (handles both '/' and '\\' separators so a sibling library whose path is a
    // prefix of this one — "Lib" vs "Library2" — never bleeds in). Trim any trailing
    // separator first (as review.rs does) — otherwise the `substr(.. +1, 1)` boundary
    // check lands one char past the real separator and zeroes the whole library out.
    let lib = library_path.trim_end_matches(['/', '\\']);
    let sql = "SELECT path, name, incoming_count, created_at, modified \
               FROM note_meta \
               WHERE substr(path, 1, length(?1)) = ?1 \
                 AND substr(path, length(?1) + 1, 1) IN ('/', char(92))";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([lib], |r| {
            Ok((
                r.get::<_, String>(0)?,        // path
                r.get::<_, String>(1)?,        // name
                r.get::<_, i64>(2)?,           // incoming_count
                r.get::<_, Option<i64>>(3)?,   // created_at
                r.get::<_, i64>(4)?,           // modified
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut results: Vec<NoteMaturity> = Vec::new();
    for row in rows.flatten() {
        let (path, name, inc, created_at, modified) = row;
        let inbound = inc.max(0) as usize;
        // Days computed exactly like review::maturity_label so the surfaces agree.
        let created = created_at.unwrap_or(modified).max(0);
        let modified = modified.max(0);
        let dsc = ((now - created).max(0) / 86_400) as u64;
        let dsm = ((now - modified).max(0) / 86_400) as u64;
        results.push(NoteMaturity {
            note_path: path,
            note_name: name,
            state: compute_state(inbound, dsc, dsm),
            inbound_count: inbound,
            days_since_modified: dsm,
        });
    }

    Ok(results)
}

/// Assign maturity state based on inbound links + file age.
/// `pub(crate)` so the Reviewer (MIG-084 §B) + the Sky trigger derive the same vocabulary
/// from the write-time `note_meta` columns — one source of the thresholds.
pub(crate) fn compute_state(inbound: usize, days_since_created: u64, days_since_modified: u64) -> String {
    // Canonical: 10+ inbound, untouched 30+ days (stable, authoritative)
    if inbound >= 10 && days_since_modified >= 30 {
        return "canonical".to_string();
    }
    // Wilting: was evergreen-level but untouched 90+ days
    if inbound >= 4 && days_since_created >= 7 && days_since_modified >= 90 {
        return "wilting".to_string();
    }
    // Evergreen: 4+ inbound, created 7+ days ago
    if inbound >= 4 && days_since_created >= 7 {
        return "evergreen".to_string();
    }
    // Sapling: 1–3 inbound OR modified 2+ days after creation
    if inbound >= 1 || days_since_created >= 2 {
        return "sapling".to_string();
    }
    // Seed: everything else
    "seed".to_string()
}

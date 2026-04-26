//! Maturity Lifecycle — Cognitive Engine Phase 3.
//!
//! Tracks note growth through 4 maturity states, computed from structural signals.
//! No manual tagging. States derived from inbound link count + file age.
//!
//! States:
//!   🌱 seed       — 0 inbound links, modified ≤1 day after creation
//!   🌿 sapling    — 1–3 inbound links OR modified 2+ days after creation
//!   🌳 evergreen  — 4+ inbound links AND modified 7+ days after creation
//!   ⭐ canonical  — 10+ inbound links AND last modified 30+ days ago
//!   🥀 wilting    — evergreen but untouched 90+ days

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

/// MIG-005 §3: read `note_aliases` into an in-memory map. See
/// `map.rs::load_alias_map` for the canonical comment — same shape per
/// Option A's per-surface discipline.
fn load_alias_map(app: &tauri::AppHandle) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let state = app.state::<crate::search::SearchState>();
    let guard = match state.db.lock() {
        Ok(g) => g,
        Err(_) => return map,
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return map,
    };
    let mut stmt = match conn
        .prepare("SELECT alias_lower, path FROM note_aliases ORDER BY path")
    {
        Ok(s) => s,
        Err(_) => return map,
    };
    let rows = match stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    }) {
        Ok(rs) => rs,
        Err(_) => return map,
    };
    for r in rows.flatten() {
        map.entry(r.0).or_insert(r.1);
    }
    map
}

/// Per-note maturity result returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct NoteMaturity {
    pub note_path: String,
    pub note_name: String,
    pub state: String,              // "seed" | "sapling" | "evergreen" | "canonical" | "wilting"
    pub inbound_count: usize,
    pub days_since_modified: u64,
}

/// Compute the maturity state for every note in a library.
#[tauri::command]
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
        .as_secs();

    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]")
        .map_err(|e| e.to_string())?;

    // Phase 1: Scan all notes — collect metadata + outgoing links
    let mut notes: HashMap<String, NoteRecord> = HashMap::new();
    scan_notes_recursive(Path::new(&library_path), &re, &mut notes, now);

    // Phase 2: Count inbound links
    // MIG-005 §3: alias-aware. Renamed targets map back to their canonical
    // note via note_aliases, so a wikilink targeting an old title still
    // counts toward the renamed note's inbound — preventing the
    // canonical/evergreen/sapling tier from regressing on rename.
    // 3-tier resolution mirrors cache.rs::read_sky_links_raw (MIG-004 §8).
    let note_names: std::collections::HashSet<String> = notes.keys().cloned().collect();
    let alias_to_path = load_alias_map(&app);
    let path_to_name: HashMap<String, String> = notes
        .values()
        .map(|n| (n.path.clone(), n.name_lower.clone()))
        .collect();

    let mut inbound_counts: HashMap<String, usize> = HashMap::new();
    for record in notes.values() {
        for target in &record.outgoing_targets {
            let target_lower = target.to_lowercase();
            let canonical = if note_names.contains(&target_lower) {
                target_lower
            } else if let Some(canonical_path) = alias_to_path.get(&target_lower) {
                match path_to_name.get(canonical_path) {
                    Some(n) => n.clone(),
                    None => continue,
                }
            } else {
                continue;
            };
            *inbound_counts.entry(canonical).or_insert(0) += 1;
        }
    }

    // Phase 3: Assign maturity state
    let results: Vec<NoteMaturity> = notes.values().map(|n| {
        let inbound = inbound_counts.get(&n.name_lower).copied().unwrap_or(0);
        let state = compute_state(inbound, n.days_since_created, n.days_since_modified);
        NoteMaturity {
            note_path: n.path.clone(),
            note_name: n.name.clone(),
            state,
            inbound_count: inbound,
            days_since_modified: n.days_since_modified,
        }
    }).collect();

    Ok(results)
}

struct NoteRecord {
    path: String,
    name: String,
    name_lower: String,
    outgoing_targets: Vec<String>,
    days_since_created: u64,
    days_since_modified: u64,
}

fn scan_notes_recursive(
    dir: &Path,
    re: &regex::Regex,
    notes: &mut HashMap<String, NoteRecord>,
    now_secs: u64,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_notes_recursive(&path, re, notes, now_secs);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let note_name = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // File metadata
            let meta = fs::metadata(&path).ok();
            let created_secs = meta.as_ref()
                .and_then(|m| m.created().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(now_secs);
            let modified_secs = meta.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(now_secs);

            let days_since_created = (now_secs.saturating_sub(created_secs)) / 86400;
            let days_since_modified = (now_secs.saturating_sub(modified_secs)) / 86400;

            // Parse outgoing links
            let mut outgoing_targets: Vec<String> = Vec::new();
            if let Ok(content) = fs::read_to_string(&path) {
                for cap in re.captures_iter(&content) {
                    outgoing_targets.push(cap[1].trim().to_string());
                }
            }

            let name_lower = note_name.to_lowercase();
            notes.insert(name_lower.clone(), NoteRecord {
                path: path.to_string_lossy().to_string(),
                name: note_name,
                name_lower,
                outgoing_targets,
                days_since_created,
                days_since_modified,
            });
        }
    }
}

/// Assign maturity state based on inbound links + file age.
fn compute_state(inbound: usize, days_since_created: u64, days_since_modified: u64) -> String {
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

//! Provenance Chain — Cognitive Engine Phase 5 (سلسلة الإسناد).
//!
//! Tracks source lineage for every knowledge claim via `|derives-from` typed links.
//! Inspired by the Islamic isnad tradition — history's most rigorous provenance system.
//!
//! - Received (متلقّاة): chain traces to external source (url/author/doi/isbn in frontmatter)
//! - Discovered (مُكتشَفة): chain originates from user's own note (no external attribution)
//! - Trust depth: count of chain links (direct source = 1, fourth-hand = 4)

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceChain {
    pub note_path: String,
    pub note_name: String,
    pub origin_type: String,    // "received" | "discovered" | "mixed" | "none"
    pub trust_depth: usize,
    pub ancestors: Vec<AncestorNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AncestorNode {
    pub name: String,
    pub path: String,
    pub depth: usize,
    pub has_external_source: bool,
}

/// Result for batch origin classification (used by GraphMind).
#[derive(Debug, Clone, Serialize)]
pub struct NoteOrigin {
    pub note_path: String,
    pub origin_type: String, // "received" | "discovered" | "mixed" | "none"
    pub trust_depth: usize,
}

/// External source property keys (checked in frontmatter).
const EXTERNAL_KEYS: &[&str] = &["url", "author", "source", "doi", "isbn", "reference"];

/// Get the provenance chain for a single note.
#[tauri::command]
pub fn get_provenance_chain(
    app: tauri::AppHandle,
    library_path: String,
    note_path: String,
    max_depth: usize,
) -> Result<ProvenanceChain, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let link_re = regex::Regex::new(r"\[\[([^\]|]+?)\|derives-from\]\]").map_err(|e| e.to_string())?;

    // Scan all notes for derives-from links + frontmatter
    let mut notes: HashMap<String, NoteInfo> = HashMap::new();
    scan_notes_recursive(Path::new(&library_path), &link_re, &mut notes);

    let note_name = Path::new(&note_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let name_lower = note_name.to_lowercase();

    // Walk the chain
    let max = if max_depth == 0 { 10 } else { max_depth };
    let mut ancestors: Vec<AncestorNode> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(name_lower.clone());
    walk_chain(&name_lower, &notes, &mut ancestors, &mut visited, 1, max);

    // Classify origin
    let origin_type = if ancestors.is_empty() {
        if notes.get(&name_lower).map(|n| n.has_external_source).unwrap_or(false) {
            "received".to_string()
        } else {
            "none".to_string()
        }
    } else {
        classify_origin(&ancestors)
    };

    let trust_depth = ancestors.iter().map(|a| a.depth).max().unwrap_or(0);

    Ok(ProvenanceChain {
        note_path,
        note_name,
        origin_type,
        trust_depth,
        ancestors,
    })
}

/// Batch compute origin type for all notes in a library (for GraphMind glow).
#[tauri::command]
pub fn compute_note_origins(
    app: tauri::AppHandle,
    library_path: String,
    _library_name: String,
) -> Result<Vec<NoteOrigin>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let link_re = regex::Regex::new(r"\[\[([^\]|]+?)\|derives-from\]\]").map_err(|e| e.to_string())?;

    let mut notes: HashMap<String, NoteInfo> = HashMap::new();
    scan_notes_recursive(Path::new(&library_path), &link_re, &mut notes);

    let note_names: Vec<String> = notes.keys().cloned().collect();
    let mut results: Vec<NoteOrigin> = Vec::new();

    for name in &note_names {
        let info = &notes[name];
        let mut ancestors: Vec<AncestorNode> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(name.clone());
        walk_chain(name, &notes, &mut ancestors, &mut visited, 1, 10);

        let origin_type = if ancestors.is_empty() {
            if info.has_external_source { "received".to_string() } else { "none".to_string() }
        } else {
            classify_origin(&ancestors)
        };
        let trust_depth = ancestors.iter().map(|a| a.depth).max().unwrap_or(0);

        results.push(NoteOrigin {
            note_path: info.path.clone(),
            origin_type,
            trust_depth,
        });
    }

    Ok(results)
}

struct NoteInfo {
    path: String,
    name: String,
    derives_from: Vec<String>, // lowercase target names
    has_external_source: bool,        // has url/author/source/doi/isbn in frontmatter
}

fn scan_notes_recursive(
    dir: &Path,
    link_re: &regex::Regex,
    notes: &mut HashMap<String, NoteInfo>,
) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_notes_recursive(&path, link_re, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let note_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Parse derives-from links
                let mut derives_from: Vec<String> = Vec::new();
                for cap in link_re.captures_iter(&content) {
                    derives_from.push(cap[1].trim().to_lowercase());
                }

                // Check frontmatter for external source indicators
                let has_external_source = check_external_frontmatter(&content);

                notes.insert(note_name.to_lowercase(), NoteInfo {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    derives_from,
                    has_external_source,
                });
            }
        }
    }
}

/// Walk the derives-from chain recursively.
fn walk_chain(
    current: &str,
    notes: &HashMap<String, NoteInfo>,
    ancestors: &mut Vec<AncestorNode>,
    visited: &mut HashSet<String>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth { return; }
    if let Some(info) = notes.get(current) {
        for target in &info.derives_from {
            if visited.contains(target) { continue; } // circular ref guard
            visited.insert(target.clone());
            let has_external_source = notes.get(target).map(|n| n.has_external_source).unwrap_or(false);
            let target_path = notes.get(target).map(|n| n.path.clone()).unwrap_or_default();
            let target_name = notes.get(target).map(|n| n.name.clone()).unwrap_or(target.clone());
            ancestors.push(AncestorNode {
                name: target_name,
                path: target_path,
                depth,
                has_external_source,
            });
            walk_chain(target, notes, ancestors, visited, depth + 1, max_depth);
        }
    }
}

/// Classify origin based on ancestor chain roots.
fn classify_origin(ancestors: &[AncestorNode]) -> String {
    if ancestors.is_empty() { return "none".to_string(); }
    // Find chain roots (deepest ancestors)
    let max_depth = ancestors.iter().map(|a| a.depth).max().unwrap_or(0);
    let roots: Vec<&AncestorNode> = ancestors.iter().filter(|a| a.depth == max_depth).collect();
    let has_received = roots.iter().any(|r| r.has_external_source);
    let has_discovered = roots.iter().any(|r| !r.has_external_source);
    if has_received && has_discovered { "mixed".to_string() }
    else if has_received { "received".to_string() }
    else { "discovered".to_string() }
}

/// Check if frontmatter contains external source properties.
fn check_external_frontmatter(content: &str) -> bool {
    if !content.starts_with("---") { return false; }
    let end = match content[3..].find("\n---") {
        Some(i) => i + 3,
        None => return false,
    };
    let yaml = &content[3..end].to_lowercase();
    EXTERNAL_KEYS.iter().any(|key| {
        // Match "key:" at start of line with a non-empty value
        yaml.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with(key) && trimmed.len() > key.len() + 1
                && trimmed.as_bytes()[key.len()] == b':'
                && trimmed[key.len() + 1..].trim().len() > 0
        })
    })
}

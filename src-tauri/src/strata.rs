//! Knowledge Strata — Cognitive Engine Phase 2.
//!
//! Auto-classifies every note into an 8-level hierarchy using graph topology.
//! No AI, no manual tagging. Pure structural signals: word count, link count,
//! link types, and centrality proxy.
//!
//! 8-Level Hierarchy:
//!   1 Datum          — ≤50 words, 0 links, raw fact
//!   2 Information    — 50–200 words, 0–1 links, single topic
//!   3 Proposition    — 200+ words or 2+ links
//!   4 Concept        — links 3+ notes, has `generalizes` links
//!   5 Principle      — links 3+ concepts, has `causes` or `supports` links
//!   6 Theory         — MOC (8+ outgoing links), many `part-of` inbound
//!   7 Paradigm       — referenced by 3+ high-stratum notes, high centrality
//!   8 Worldview      — highest centrality, deepest `derives-from` chain root

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Per-note stratum result returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct NoteStratum {
    pub note_path: String,
    pub note_name: String,
    pub stratum: u8,
    pub word_count: usize,
    pub outgoing_links: usize,
    pub inbound_links: usize,
}

/// Known semantic link types (same as libraries.rs KNOWN_LINK_TYPES).
const KNOWN_LINK_TYPES: &[&str] = &[
    "supports", "contradicts", "causes", "exemplifies",
    "generalizes", "derives-from", "part-of", "associative",
];

/// Internal note record used during computation.
struct NoteRecord {
    path: String,
    name: String,
    word_count: usize,
    outgoing: Vec<String>,           // target note names
    outgoing_types: HashSet<String>, // link types used in outgoing links
    inbound_count: usize,
    inbound_sources: HashSet<String>, // unique source note names
}

/// Compute the knowledge stratum for every note in a library.
#[tauri::command]
pub fn compute_note_strata(
    app: tauri::AppHandle,
    library_path: String,
    _library_name: String,
) -> Result<Vec<NoteStratum>, String> {
    // Validate access
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]")
        .map_err(|e| e.to_string())?;

    // Phase 1: Scan all notes — collect word count + outgoing links + link types
    let mut notes: HashMap<String, NoteRecord> = HashMap::new();
    scan_notes_recursive(Path::new(&library_path), &re, &mut notes);

    // Phase 2: Build inbound map
    let note_names: HashSet<String> = notes.keys().cloned().collect();
    // Collect inbound data first (can't borrow notes mutably while iterating)
    let mut inbound_data: HashMap<String, (usize, HashSet<String>)> = HashMap::new();
    for record in notes.values() {
        for target_name in &record.outgoing {
            let target_lower = target_name.to_lowercase();
            if note_names.contains(&target_lower) {
                let entry = inbound_data.entry(target_lower).or_insert((0, HashSet::new()));
                entry.0 += 1;
                entry.1.insert(record.name.clone());
            }
        }
    }
    for (name, (count, sources)) in inbound_data {
        if let Some(record) = notes.get_mut(&name) {
            record.inbound_count += count;
            record.inbound_sources = sources;
        }
    }

    // Phase 3: Compute stratum for each note
    let results: Vec<NoteStratum> = notes.values().map(|n| {
        let stratum = compute_stratum(n);
        NoteStratum {
            note_path: n.path.clone(),
            note_name: n.name.clone(),
            stratum,
            word_count: n.word_count,
            outgoing_links: n.outgoing.len(),
            inbound_links: n.inbound_count,
        }
    }).collect();

    Ok(results)
}

/// Recursively scan a directory for .md files, building NoteRecords.
fn scan_notes_recursive(
    dir: &Path,
    re: &regex::Regex,
    notes: &mut HashMap<String, NoteRecord>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden directories/files
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan_notes_recursive(&path, re, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let note_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Strip YAML frontmatter for word count
                let body = strip_frontmatter(&content);
                let word_count = count_words(body);

                // Parse outgoing wikilinks + link types
                let mut outgoing: Vec<String> = Vec::new();
                let mut outgoing_types: HashSet<String> = HashSet::new();

                for cap in re.captures_iter(&content) {
                    let target = cap[1].trim().to_string();
                    outgoing.push(target.to_lowercase());

                    // Extract link type if present
                    if let Some(alias) = cap.get(2) {
                        let lower = alias.as_str().trim().to_lowercase();
                        let type_str = if lower.starts_with("type:") {
                            lower[5..].trim().to_string()
                        } else if KNOWN_LINK_TYPES.contains(&lower.as_str()) {
                            lower
                        } else {
                            continue;
                        };
                        outgoing_types.insert(type_str);
                    }
                }

                notes.insert(note_name.to_lowercase(), NoteRecord {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    word_count,
                    outgoing,
                    outgoing_types,
                    inbound_count: 0,
                    inbound_sources: HashSet::new(),
                });
            }
        }
    }
}

/// Compute the stratum (1–8) for a single note based on structural signals.
fn compute_stratum(n: &NoteRecord) -> u8 {
    // Base from word count
    let base: u8 = if n.word_count <= 50 {
        1
    } else if n.word_count <= 200 {
        2
    } else {
        3
    };

    let mut bonus: u8 = 0;

    // Link count bonus
    if n.outgoing.len() >= 3 {
        bonus += 1;
    }
    if n.inbound_count >= 5 {
        bonus += 1;
    }

    // Semantic link type bonus
    if n.outgoing_types.contains("generalizes") {
        bonus += 1;
    }
    if n.outgoing_types.contains("causes") || n.outgoing_types.contains("supports") {
        bonus += 1;
    }

    // Centrality proxy: inbound from 3+ unique sources
    if n.inbound_sources.len() >= 3 {
        bonus += 1;
    }

    // Clamp to 1–8
    (base + bonus).clamp(1, 8)
}

/// Strip YAML frontmatter (--- delimited) from content, return body.
/// Also exposed as `strip_frontmatter_pub` for use by other modules.
pub fn strip_frontmatter_pub(content: &str) -> &str { strip_frontmatter(content) }
fn strip_frontmatter(content: &str) -> &str {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            return &content[end + 7..]; // skip past closing ---\n
        }
    }
    content
}

/// Count words in text (split on whitespace, filter empty).
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

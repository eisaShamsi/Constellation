//! Constellation Map — CE Layer 2.
//!
//! Computes a hierarchical tree with knowledge weight, maturity, and stratum
//! per node for the radial sunburst visualization (Constellation Map).
//!
//! Pipeline: recursive filesystem walk → word count + link extraction →
//! inbound link aggregation → maturity/stratum inference → weight computation →
//! recursive tree with aggregated metrics.
//!
//! Based on the Constellation Map Concept Paper (April 2026).

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::strata::strip_frontmatter_pub;

/// A node in the Map tree — either a folder (with children) or a note (leaf).
#[derive(Debug, Clone, Serialize)]
pub struct MapNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub weight: f64,
    pub note_count: u32,
    pub word_count: u32,
    pub link_count: u32,
    pub maturity: Option<String>,
    pub stratum: Option<u8>,
    pub modified: Option<u64>,
    pub children: Option<Vec<MapNode>>,
}

/// Internal record for a note, built during the first pass.
struct NoteRecord {
    path: String,
    name: String,
    word_count: u32,
    outgoing_links: Vec<String>, // target note names (lowercase)
    modified: u64,               // unix timestamp
    created: u64,                // unix timestamp
}

/// Compute the Constellation Map tree for a library.
#[tauri::command]
pub fn constellation_map_data(
    app: tauri::AppHandle,
    library_path: String,
    max_depth: Option<u32>,
) -> Result<MapNode, String> {
    // Security: validate library access
    let libraries = crate::libraries::load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }

    let root = Path::new(&library_path);
    if !root.is_dir() {
        return Err("Library path is not a directory.".to_string());
    }

    let depth_limit = max_depth.unwrap_or(5);

    // Pass 1: Collect all notes with metadata
    let mut all_notes: Vec<NoteRecord> = Vec::new();
    collect_notes_recursive(root, &mut all_notes);

    // Build inbound link count map: target_name_lower → count
    let mut inbound_map: HashMap<String, usize> = HashMap::new();
    // Build note name → path map for link resolution
    let mut name_to_path: HashMap<String, String> = HashMap::new();
    for note in &all_notes {
        let key = note.name.to_lowercase();
        name_to_path.insert(key, note.path.clone());
    }
    for note in &all_notes {
        for target in &note.outgoing_links {
            *inbound_map.entry(target.clone()).or_insert(0) += 1;
        }
    }

    // Build note metadata map: path_lower → (word_count, link_count, inbound, maturity, stratum, modified)
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut note_meta: HashMap<String, (u32, u32, String, Option<u8>, u64)> = HashMap::new();
    for note in &all_notes {
        let key = note.path.replace('\\', "/").to_lowercase();
        let inbound = *inbound_map.get(&note.name.to_lowercase()).unwrap_or(&0);

        // Maturity (reuse logic from maturity.rs)
        let days_since_created = (now_secs.saturating_sub(note.created)) / 86400;
        let days_since_modified = (now_secs.saturating_sub(note.modified)) / 86400;
        let maturity = compute_maturity(inbound, days_since_created, days_since_modified);

        // Simplified stratum (1-8 based on word count + links)
        let stratum = compute_simple_stratum(note.word_count, note.outgoing_links.len(), inbound);

        note_meta.insert(key, (
            note.word_count,
            note.outgoing_links.len() as u32,
            maturity,
            Some(stratum),
            note.modified,
        ));
    }

    // Pass 2: Build the MapNode tree
    let tree = build_tree(root, &note_meta, 0, depth_limit);

    Ok(tree)
}

/// Recursively collect all .md notes with word count, links, and timestamps.
fn collect_notes_recursive(dir: &Path, notes: &mut Vec<NoteRecord>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }

        if path.is_dir() {
            collect_notes_recursive(&path, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let note_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Strip frontmatter for word counting
                let body = strip_frontmatter_pub(&content);

                // Count words (split on whitespace)
                let word_count = body.split_whitespace().count() as u32;

                // Extract outgoing wikilinks
                let wiki_re = regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
                let outgoing_links: Vec<String> = wiki_re
                    .captures_iter(body)
                    .map(|cap| cap[1].trim().to_lowercase())
                    .collect();

                // File timestamps
                let metadata = fs::metadata(&path).ok();
                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let created = metadata.as_ref()
                    .and_then(|m| m.created().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(modified);

                notes.push(NoteRecord {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    word_count,
                    outgoing_links,
                    modified,
                    created,
                });
            }
        }
    }
}

/// Build the recursive MapNode tree from the filesystem.
fn build_tree(
    dir: &Path,
    note_meta: &HashMap<String, (u32, u32, String, Option<u8>, u64)>,
    depth: u32,
    max_depth: u32,
) -> MapNode {
    let dir_name = dir.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string());

    let mut children: Vec<MapNode> = Vec::new();
    let mut total_weight: f64 = 0.0;
    let mut total_notes: u32 = 0;
    let mut total_words: u32 = 0;
    let mut total_links: u32 = 0;
    let mut latest_modified: u64 = 0;

    if depth < max_depth {
        if let Ok(read_dir) = fs::read_dir(dir) {
            let mut entries: Vec<_> = read_dir.flatten().collect();
            entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            for entry in entries {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; }

                if path.is_dir() {
                    let child = build_tree(&path, note_meta, depth + 1, max_depth);
                    total_weight += child.weight;
                    total_notes += child.note_count;
                    total_words += child.word_count;
                    total_links += child.link_count;
                    if child.modified.unwrap_or(0) > latest_modified {
                        latest_modified = child.modified.unwrap_or(0);
                    }
                    // Only include non-empty folders
                    if child.note_count > 0 {
                        children.push(child);
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    let key = path.to_string_lossy().replace('\\', "/").to_lowercase();
                    let note_name = path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if let Some((wc, lc, maturity, stratum, modified)) = note_meta.get(&key) {
                        let weight = compute_weight(*wc, *lc, *modified);
                        total_weight += weight;
                        total_notes += 1;
                        total_words += wc;
                        total_links += lc;
                        if *modified > latest_modified {
                            latest_modified = *modified;
                        }

                        children.push(MapNode {
                            name: note_name,
                            path: path.to_string_lossy().to_string(),
                            is_dir: false,
                            weight,
                            note_count: 1,
                            word_count: *wc,
                            link_count: *lc,
                            maturity: Some(maturity.clone()),
                            stratum: *stratum,
                            modified: Some(*modified),
                            children: None,
                        });
                    }
                }
            }
        }
    }

    MapNode {
        name: dir_name,
        path: dir.to_string_lossy().to_string(),
        is_dir: true,
        weight: total_weight.max(0.1), // minimum weight so empty folders still visible
        note_count: total_notes,
        word_count: total_words,
        link_count: total_links,
        maturity: None,
        stratum: None,
        modified: if latest_modified > 0 { Some(latest_modified) } else { None },
        children: if children.is_empty() { None } else { Some(children) },
    }
}

/// Compute knowledge weight for a single note.
fn compute_weight(word_count: u32, link_count: u32, modified: u64) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days_since = (now.saturating_sub(modified)) as f64 / 86400.0;

    let word_score = (word_count as f64 / 500.0).min(5.0);
    let link_score = (link_count as f64 / 3.0).min(5.0);
    let recency_score = (1.0 - days_since / 365.0).max(0.0);

    (word_score + link_score + recency_score).max(0.1)
}

/// Maturity state (reuses logic from maturity.rs).
fn compute_maturity(inbound: usize, days_since_created: u64, days_since_modified: u64) -> String {
    if inbound >= 10 && days_since_modified >= 30 {
        return "canonical".to_string();
    }
    if inbound >= 4 && days_since_created >= 7 && days_since_modified >= 90 {
        return "wilting".to_string();
    }
    if inbound >= 4 && days_since_created >= 7 {
        return "evergreen".to_string();
    }
    if inbound >= 1 || days_since_created >= 2 {
        return "sapling".to_string();
    }
    "seed".to_string()
}

/// Simplified stratum (1-8) based on word count + link count.
fn compute_simple_stratum(word_count: u32, outgoing: usize, inbound: usize) -> u8 {
    let total_links = outgoing + inbound;
    if word_count > 2000 && total_links > 15 { return 8; }
    if word_count > 1500 && total_links > 10 { return 7; }
    if word_count > 1000 && total_links > 8 { return 6; }
    if word_count > 500 && total_links > 5 { return 5; }
    if word_count > 300 && total_links > 3 { return 4; }
    if word_count > 200 && total_links > 1 { return 3; }
    if word_count > 50 { return 2; }
    1
}

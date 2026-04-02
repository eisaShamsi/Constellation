//! Tension Detector — Cognitive Engine Phase 4.
//!
//! Surfaces contradictions, orphan knowledge, structural gaps, and single
//! points of failure. Zero AI — pure graph topology analysis.
//!
//! 4 Detection Types:
//!   1. Contradictions  — notes linked with `|contradicts`
//!   2. Orphans         — notes with 0 inbound links
//!   3. Structural gaps — tag-clusters with no cross-wikilinks
//!   4. Single points   — notes with 5+ inbound but ≤1 derives-from source
//!
//! Earned complexity: activates only when library has 50+ linked notes.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct TensionReport {
    pub contradictions: Vec<TensionItem>,
    pub orphans: Vec<TensionItem>,
    pub structural_gaps: Vec<GapItem>,
    pub single_points: Vec<TensionItem>,
    pub total_linked_notes: usize,
    pub total_notes: usize,
    pub active: bool, // false if <50 linked notes
}

#[derive(Debug, Clone, Serialize)]
pub struct TensionItem {
    pub note_name: String,
    pub note_path: String,
    pub severity: String, // "low" | "medium" | "high"
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapItem {
    pub tag: String,
    pub notes: Vec<String>, // note names in the cluster with no cross-links
    pub severity: String,
}

const KNOWN_LINK_TYPES: &[&str] = &[
    "supports", "contradicts", "causes", "exemplifies",
    "generalizes", "derives-from", "part-of", "associative",
];

struct NoteInfo {
    path: String,
    name: String,
    word_count: usize,
    outgoing: Vec<(String, Option<String>)>, // (target_name_lower, link_type)
    tags: HashSet<String>,
}

/// Detect knowledge tensions in a library.
#[tauri::command]
pub fn detect_tensions(
    app: tauri::AppHandle,
    library_path: String,
    _library_name: String,
) -> Result<TensionReport, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let link_re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").map_err(|e| e.to_string())?;
    let tag_re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").map_err(|e| e.to_string())?;

    // Phase 1: Scan all notes
    let mut notes: HashMap<String, NoteInfo> = HashMap::new();
    scan_notes_recursive(Path::new(&library_path), &link_re, &tag_re, &mut notes);

    let note_names: HashSet<String> = notes.keys().cloned().collect();

    // Phase 2: Build inbound map
    let mut inbound_count: HashMap<String, usize> = HashMap::new();
    let mut inbound_sources: HashMap<String, HashSet<String>> = HashMap::new();
    let mut derives_from_sources: HashMap<String, HashSet<String>> = HashMap::new();

    for info in notes.values() {
        for (target, link_type) in &info.outgoing {
            if note_names.contains(target) {
                *inbound_count.entry(target.clone()).or_insert(0) += 1;
                inbound_sources.entry(target.clone()).or_default().insert(info.name.clone());
                if link_type.as_deref() == Some("derives-from") {
                    derives_from_sources.entry(target.clone()).or_default().insert(info.name.clone());
                }
            }
        }
    }

    let total_notes = notes.len();
    let total_linked = inbound_count.len();

    // Earned complexity check
    if total_linked < 50 {
        return Ok(TensionReport {
            contradictions: vec![],
            orphans: vec![],
            structural_gaps: vec![],
            single_points: vec![],
            total_linked_notes: total_linked,
            total_notes,
            active: false,
        });
    }

    // Detection 1: Contradictions
    let mut contradictions: Vec<TensionItem> = Vec::new();
    for info in notes.values() {
        for (target, link_type) in &info.outgoing {
            if link_type.as_deref() == Some("contradicts") {
                if let Some(target_info) = notes.get(target) {
                    contradictions.push(TensionItem {
                        note_name: info.name.clone(),
                        note_path: info.path.clone(),
                        severity: "high".to_string(),
                        detail: format!("contradicts \"{}\"", target_info.name),
                    });
                }
            }
        }
    }

    // Detection 2: Orphans (0 inbound links, has content)
    let mut orphans: Vec<TensionItem> = Vec::new();
    for info in notes.values() {
        let inbound = inbound_count.get(&info.name.to_lowercase()).copied().unwrap_or(0);
        if inbound == 0 && info.word_count > 20 {
            let severity = if info.word_count > 500 { "high" }
                else if info.word_count > 100 { "medium" }
                else { "low" };
            orphans.push(TensionItem {
                note_name: info.name.clone(),
                note_path: info.path.clone(),
                severity: severity.to_string(),
                detail: format!("{} words, no inbound links", info.word_count),
            });
        }
    }
    // Sort orphans: high severity first
    orphans.sort_by(|a, b| {
        let ord = |s: &str| match s { "high" => 0, "medium" => 1, _ => 2 };
        ord(&a.severity).cmp(&ord(&b.severity))
    });

    // Detection 3: Structural gaps (tag-clusters without cross-links)
    let mut structural_gaps: Vec<GapItem> = Vec::new();
    // Group notes by tag
    let mut tag_notes: HashMap<String, Vec<String>> = HashMap::new();
    for info in notes.values() {
        for tag in &info.tags {
            tag_notes.entry(tag.clone()).or_default().push(info.name.to_lowercase());
        }
    }
    // Find tags with 3+ notes where notes don't link to each other
    for (tag, members) in &tag_notes {
        if members.len() < 3 { continue; }
        // Check how many cross-links exist within this tag group
        let member_set: HashSet<&String> = members.iter().collect();
        let mut cross_links = 0;
        for member in members {
            if let Some(info) = notes.get(member) {
                for (target, _) in &info.outgoing {
                    if member_set.contains(target) && target != member {
                        cross_links += 1;
                    }
                }
            }
        }
        // If fewer than 20% of possible links exist, it's a gap
        let possible = members.len() * (members.len() - 1);
        if possible > 0 && cross_links * 5 < possible {
            let note_names: Vec<String> = members.iter()
                .filter_map(|m| notes.get(m).map(|i| i.name.clone()))
                .take(5)
                .collect();
            structural_gaps.push(GapItem {
                tag: tag.clone(),
                notes: note_names,
                severity: if members.len() >= 8 { "high".to_string() }
                    else if members.len() >= 5 { "medium".to_string() }
                    else { "low".to_string() },
            });
        }
    }
    structural_gaps.sort_by(|a, b| b.notes.len().cmp(&a.notes.len()));
    structural_gaps.truncate(20); // limit to top 20 gaps

    // Detection 4: Single points of failure
    let mut single_points: Vec<TensionItem> = Vec::new();
    for (name_lower, sources) in &inbound_sources {
        if sources.len() >= 5 {
            let derives_count = derives_from_sources.get(name_lower)
                .map(|s| s.len()).unwrap_or(0);
            if derives_count <= 1 {
                if let Some(info) = notes.get(name_lower) {
                    single_points.push(TensionItem {
                        note_name: info.name.clone(),
                        note_path: info.path.clone(),
                        severity: if sources.len() >= 10 { "high".to_string() }
                            else { "medium".to_string() },
                        detail: format!("referenced by {} notes, only {} source", sources.len(), derives_count),
                    });
                }
            }
        }
    }

    Ok(TensionReport {
        contradictions,
        orphans,
        structural_gaps,
        single_points,
        total_linked_notes: total_linked,
        total_notes,
        active: true,
    })
}

fn scan_notes_recursive(
    dir: &Path,
    link_re: &regex::Regex,
    tag_re: &regex::Regex,
    notes: &mut HashMap<String, NoteInfo>,
) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_notes_recursive(&path, link_re, tag_re, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let note_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                let body = crate::strata::strip_frontmatter_pub(&content);
                let word_count = body.split_whitespace().count();

                let mut outgoing: Vec<(String, Option<String>)> = Vec::new();
                for cap in link_re.captures_iter(&content) {
                    let target = cap[1].trim().to_lowercase();
                    let link_type = cap.get(2).and_then(|alias| {
                        let lower = alias.as_str().trim().to_lowercase();
                        if KNOWN_LINK_TYPES.contains(&lower.as_str()) { Some(lower) } else { None }
                    });
                    outgoing.push((target, link_type));
                }

                let mut tags: HashSet<String> = HashSet::new();
                for cap in tag_re.captures_iter(&content) {
                    tags.insert(cap[1].to_string().to_lowercase());
                }

                notes.insert(note_name.to_lowercase(), NoteInfo {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    word_count,
                    outgoing,
                    tags,
                });
            }
        }
    }
}

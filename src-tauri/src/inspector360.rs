//! 360.3D Inspector — Cognitive Engine Phase 12 (المنظار الكروي).
//!
//! Spherical knowledge inspector: the note is the CORE of a sphere.
//! The ATMOSPHERE surrounding it is every dimension of understanding.
//! Gaps in the atmosphere are blind spots. Dense areas are rich understanding.
//!
//! Aggregates data from ALL Layer 1 phases for a single note.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// A note connected to the inspected note.
#[derive(Debug, Clone, Serialize)]
pub struct LinkedNote {
    pub name: String,
    pub path: String,
    pub depth: usize, // 1 = direct, 2 = second-order, 3 = third-order
    pub stratum: u8,  // 1..=8, the connected note's CE stratum
}

/// Complete 360° view of a single note.
#[derive(Debug, Clone, Serialize)]
pub struct Note360View {
    // Identity
    pub note_path: String,
    pub note_name: String,
    pub word_count: usize,

    // Phase 1: Typed links grouped by type
    pub typed_links: HashMap<String, Vec<LinkedNote>>,
    pub untyped_links: Vec<LinkedNote>,
    pub total_outbound: usize,
    pub total_inbound: usize,

    // Phase 2: Stratum
    pub stratum: u8,

    // Phase 3: Maturity
    pub maturity: String,

    // Phase 4: Tensions
    pub contradictions: Vec<String>, // note names this note contradicts
    pub is_orphan: bool,
    pub single_point_of_failure: bool,

    // Phase 5: Provenance
    pub origin_type: String,
    pub trust_depth: usize,

    // Phase 6: Stage
    pub stage: String,

    // Phase 7: Review
    pub last_reviewed: Option<String>,
    pub is_due: bool,

    // Phase 8: Trail membership
    pub trails: Vec<String>,

    // Phase 9: Lens groups
    pub lens_groups: Vec<String>,

    // Gaps (blind spots)
    pub missing_link_types: Vec<String>,
    pub used_link_types: Vec<String>,
}

/// Get the complete 360° view for a single note.
#[tauri::command]
pub fn get_360_view(
    app: tauri::AppHandle,
    library_path: String,
    note_path: String,
) -> Result<Note360View, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let link_re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").map_err(|e| e.to_string())?;
    let tag_re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").map_err(|e| e.to_string())?;

    // MIG-008 Step 2: derive the active note's display name from
    // frontmatter `title:` (with file_stem fallback). For canonical
    // notes (`20260426T140737Z_NOTE_E561.md`) the title is the only
    // value that matches incoming wikilinks like `[[Apple Tree Fruit]]`,
    // so this is also a correctness fix — not just a label fix.
    let note_path_p = Path::new(&note_path);
    let note_name = crate::libraries::note_display_name(note_path_p, None);
    let note_lower = note_name.to_lowercase();

    // Scan all notes in the library
    let mut all_notes: HashMap<String, NoteInfo> = HashMap::new();
    scan_all_notes(Path::new(&library_path), &link_re, &tag_re, &mut all_notes);

    // §112: Pre-compute stratum for every note in the library so each
    // LinkedNote can carry its source/target's stratum (the matrix
    // visualisation needs this for vertical positioning). One pass over
    // the link graph builds inbound_counts + unique_sources_count per
    // note; another pass computes stratum from those + the existing
    // word_count / outgoing inputs. Total cost: O(N + total_links),
    // not O(N²).
    let strata: HashMap<String, u8> = precompute_all_strata(&all_notes);

    let target_info = all_notes.get(&note_lower);

    // ─── Phase 1: Typed links ───
    let mut typed_links: HashMap<String, Vec<LinkedNote>> = HashMap::new();
    let mut untyped_links: Vec<LinkedNote> = Vec::new();
    let mut used_types: HashSet<String> = HashSet::new();
    let mut total_outbound = 0;
    let mut total_inbound = 0;

    // Outbound links from this note
    if let Some(info) = target_info {
        total_outbound = info.outgoing.len();
        for (target, link_type) in &info.outgoing {
            let linked = LinkedNote {
                name: all_notes.get(target).map(|n| n.name.clone()).unwrap_or(target.clone()),
                path: all_notes.get(target).map(|n| n.path.clone()).unwrap_or_default(),
                depth: 1,
                stratum: strata.get(target).copied().unwrap_or(1),
            };
            if let Some(lt) = link_type {
                used_types.insert(lt.clone());
                typed_links.entry(lt.clone()).or_default().push(linked);
            } else {
                untyped_links.push(linked);
            }
        }
    }

    // Inbound links to this note
    for (source_lower, info) in &all_notes {
        for (target, link_type) in &info.outgoing {
            if target == &note_lower {
                total_inbound += 1;
                let linked = LinkedNote {
                    name: info.name.clone(),
                    path: info.path.clone(),
                    depth: 1,
                    stratum: strata.get(source_lower).copied().unwrap_or(1),
                };
                if let Some(lt) = link_type {
                    used_types.insert(lt.clone());
                    typed_links.entry(lt.clone()).or_default().push(linked);
                } else {
                    untyped_links.push(linked);
                }
            }
        }
    }

    // Second-order connections (depth 2)
    let direct_names: HashSet<String> = typed_links.values()
        .flatten()
        .chain(untyped_links.iter())
        .map(|n| n.name.to_lowercase())
        .collect();

    for direct_name in &direct_names {
        if let Some(info) = all_notes.get(direct_name) {
            for (target, _lt) in &info.outgoing {
                if target != &note_lower && !direct_names.contains(target) {
                    untyped_links.push(LinkedNote {
                        name: all_notes.get(target).map(|n| n.name.clone()).unwrap_or(target.clone()),
                        path: all_notes.get(target).map(|n| n.path.clone()).unwrap_or_default(),
                        depth: 2,
                        stratum: strata.get(target).copied().unwrap_or(1),
                    });
                }
            }
        }
    }

    // ─── Phase 2: Stratum ───
    let stratum = compute_stratum_for_note(target_info, total_inbound, &all_notes, &note_lower);

    // ─── Phase 3: Maturity ───
    let maturity = compute_maturity_for_note(target_info, total_inbound);

    // ─── Phase 4: Tensions ───
    let contradictions: Vec<String> = typed_links.get("contradicts")
        .map(|v| v.iter().map(|n| n.name.clone()).collect())
        .unwrap_or_default();
    let is_orphan = total_inbound == 0 && target_info.map(|i| i.word_count > 20).unwrap_or(false);
    let derives_count = typed_links.get("derives-from").map(|v| v.len()).unwrap_or(0);
    let single_point_of_failure = total_inbound >= 5 && derives_count <= 1;

    // ─── Phase 5: Provenance ───
    let (origin_type, trust_depth) = compute_provenance_for_note(&note_lower, &all_notes);

    // ─── Phase 6: Stage ───
    let stage = target_info
        .and_then(|i| i.stage.clone())
        .unwrap_or_default();

    // ─── Phase 7: Review ───
    let (last_reviewed, is_due) = get_review_for_note(&app, &note_path);

    // ─── Phase 8: Trails ───
    let trails = get_trails_for_note(Path::new(&library_path), &note_name);

    // ─── Phase 9: Lens groups ───
    // Simplified: return tags as lens groups
    let lens_groups: Vec<String> = target_info
        .map(|i| i.tags.iter().cloned().collect())
        .unwrap_or_default();

    // ─── Gaps ───
    // MIG-067 §D — the typed acts a note lacks come from the active registry
    // (the 8 seeds + custom, ordered), not a hardcoded 7. This is the drift fix:
    // `supersedes` (and any custom type) now appears as a possible gap.
    let all_link_types = crate::link_types::snapshot().ids();
    let missing_link_types: Vec<String> = all_link_types
        .iter()
        .filter(|t| !used_types.contains(t.as_str()))
        .cloned()
        .collect();

    let word_count = target_info.map(|i| i.word_count).unwrap_or(0);

    Ok(Note360View {
        note_path,
        note_name,
        word_count,
        typed_links,
        untyped_links,
        total_outbound,
        total_inbound,
        stratum,
        maturity,
        contradictions,
        is_orphan,
        single_point_of_failure,
        origin_type,
        trust_depth,
        stage,
        last_reviewed,
        is_due,
        trails,
        lens_groups,
        missing_link_types,
        used_link_types: used_types.into_iter().collect(),
    })
}

// ─── Internal data structures ───

struct NoteInfo {
    path: String,
    name: String,
    word_count: usize,
    outgoing: Vec<(String, Option<String>)>, // (target_lower, link_type)
    tags: HashSet<String>,
    has_external: bool,
    stage: Option<String>,
    days_since_modified: u64,
    days_since_created: u64,
}

fn scan_all_notes(
    dir: &Path,
    link_re: &regex::Regex,
    tag_re: &regex::Regex,
    notes: &mut HashMap<String, NoteInfo>,
) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    // MIG-067 §D — snapshot the link-type registry once per directory (not a
    // hardcoded set rebuilt per note); `is_link_type_value` accepts the 8 typed
    // acts + custom + the null `associative`, so custom types and `supersedes`
    // are recognized here too. Off-lock clone, reused across this dir's notes.
    let reg = crate::link_types::snapshot();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_all_notes(&path, link_re, tag_re, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // MIG-008 Step 2: prefer frontmatter title for the scanned
                // note's name so neighbor labels in the 360° view show
                // human titles, not canonical filenames.
                let note_name = crate::libraries::note_display_name(&path, Some(&content));
                let body = crate::strata::strip_frontmatter_pub(&content);
                let word_count = body.split_whitespace().count();

                let mut outgoing: Vec<(String, Option<String>)> = Vec::new();

                for cap in link_re.captures_iter(&content) {
                    // MIG-067 — predicate-first aware ([[type::target]] AND legacy
                    // [[target|type]]); `associative` counts as untyped for the matrix.
                    let (target, link_type) = crate::link_types::resolve_wikilink_type(
                        &reg, &cap[1], cap.get(2).map(|m| m.as_str()), false,
                    );
                    outgoing.push((target.to_lowercase(), link_type));
                }

                let mut tags: HashSet<String> = HashSet::new();
                for cap in tag_re.captures_iter(&content) {
                    tags.insert(cap[1].to_string().to_lowercase());
                }

                let has_external = crate::provenance::check_external_pub(&content);
                let stage = extract_stage(&content);

                let meta = fs::metadata(&path).ok();
                let created = meta.as_ref().and_then(|m| m.created().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()).unwrap_or(now);
                let modified = meta.as_ref().and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()).unwrap_or(now);

                notes.insert(note_name.to_lowercase(), NoteInfo {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    word_count,
                    outgoing,
                    tags,
                    has_external,
                    stage,
                    days_since_modified: (now.saturating_sub(modified)) / 86400,
                    days_since_created: (now.saturating_sub(created)) / 86400,
                });
            }
        }
    }
}

fn extract_stage(content: &str) -> Option<String> {
    let normalized = content.replace("\r\n", "\n");
    let mut pos = 0;
    while let Some(start) = normalized[pos..].find("---") {
        let block_start = pos + start + 3;
        if let Some(end) = normalized[block_start..].find("\n---") {
            let yaml = &normalized[block_start..block_start + end];
            for line in yaml.lines() {
                let trimmed = line.trim().to_lowercase();
                if let Some(val) = trimmed.strip_prefix("stage:") {
                    let v = val.trim().to_string();
                    if !v.is_empty() { return Some(v); }
                }
            }
            pos = block_start + end + 4;
        } else { break; }
    }
    None
}

/// §112: Pre-compute stratum for every note. Builds inbound counts and
/// unique-sources counts from a single pass over the link graph, then
/// computes each note's stratum using the existing rule set without
/// re-scanning all_notes per note.
fn precompute_all_strata(all_notes: &HashMap<String, NoteInfo>) -> HashMap<String, u8> {
    // Pass 1: inbound counts + unique source-note sets per target.
    let mut inbound_counts: HashMap<String, usize> = HashMap::new();
    let mut sources_of: HashMap<String, HashSet<String>> = HashMap::new();
    for (_, info) in all_notes {
        for (target_lower, _) in &info.outgoing {
            *inbound_counts.entry(target_lower.clone()).or_insert(0) += 1;
            sources_of
                .entry(target_lower.clone())
                .or_insert_with(HashSet::new)
                .insert(info.name.clone());
        }
    }

    // Pass 2: per-note stratum from the existing rule set.
    let mut strata: HashMap<String, u8> = HashMap::with_capacity(all_notes.len());
    for (name_lower, info) in all_notes {
        let inbound = inbound_counts.get(name_lower).copied().unwrap_or(0);
        let sources_count = sources_of.get(name_lower).map(|s| s.len()).unwrap_or(0);
        let base: u8 = if info.word_count <= 50 {
            1
        } else if info.word_count <= 200 {
            2
        } else {
            3
        };
        let mut bonus: u8 = 0;
        if info.outgoing.len() >= 3 {
            bonus += 1;
        }
        if inbound >= 5 {
            bonus += 1;
        }
        let types_used: HashSet<&str> =
            info.outgoing.iter().filter_map(|(_, t)| t.as_deref()).collect();
        if types_used.contains("generalizes") {
            bonus += 1;
        }
        if types_used.contains("causes") || types_used.contains("supports") {
            bonus += 1;
        }
        if sources_count >= 3 {
            bonus += 1;
        }
        strata.insert(name_lower.clone(), (base + bonus).clamp(1, 8));
    }
    strata
}

fn compute_stratum_for_note(info: Option<&NoteInfo>, inbound: usize, all_notes: &HashMap<String, NoteInfo>, note_lower: &str) -> u8 {
    let Some(info) = info else { return 1; };
    let base: u8 = if info.word_count <= 50 { 1 } else if info.word_count <= 200 { 2 } else { 3 };
    let mut bonus: u8 = 0;
    if info.outgoing.len() >= 3 { bonus += 1; }
    if inbound >= 5 { bonus += 1; }
    let types_used: HashSet<&str> = info.outgoing.iter().filter_map(|(_, t)| t.as_deref()).collect();
    if types_used.contains("generalizes") { bonus += 1; }
    if types_used.contains("causes") || types_used.contains("supports") { bonus += 1; }
    let unique_sources: HashSet<&str> = all_notes.values()
        .filter(|n| n.outgoing.iter().any(|(t, _)| t == note_lower))
        .map(|n| n.name.as_str())
        .collect();
    if unique_sources.len() >= 3 { bonus += 1; }
    (base + bonus).clamp(1, 8)
}

fn compute_maturity_for_note(info: Option<&NoteInfo>, inbound: usize) -> String {
    let Some(info) = info else { return "seed".to_string(); };
    if inbound >= 10 && info.days_since_modified >= 30 { return "canonical".to_string(); }
    if inbound >= 4 && info.days_since_created >= 7 && info.days_since_modified >= 90 { return "wilting".to_string(); }
    if inbound >= 4 && info.days_since_created >= 7 { return "evergreen".to_string(); }
    if inbound >= 1 || info.days_since_created >= 2 { return "sapling".to_string(); }
    "seed".to_string()
}

fn compute_provenance_for_note(note_lower: &str, all_notes: &HashMap<String, NoteInfo>) -> (String, usize) {
    let Some(info) = all_notes.get(note_lower) else { return ("none".to_string(), 0); };
    let derives: Vec<&str> = info.outgoing.iter()
        .filter(|(_, t)| t.as_deref() == Some("derives-from"))
        .map(|(target, _)| target.as_str())
        .collect();
    if derives.is_empty() {
        return if info.has_external { ("received".to_string(), 0) } else { ("none".to_string(), 0) };
    }
    // Walk chain
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(note_lower.to_string());
    let mut max_depth = 0;
    let mut has_external = false;
    fn walk(current: &str, notes: &HashMap<String, NoteInfo>, visited: &mut HashSet<String>, depth: usize, max_depth: &mut usize, has_ext: &mut bool) {
        if depth > 10 { return; }
        if let Some(info) = notes.get(current) {
            for (target, lt) in &info.outgoing {
                if lt.as_deref() == Some("derives-from") && !visited.contains(target) {
                    visited.insert(target.clone());
                    if depth + 1 > *max_depth { *max_depth = depth + 1; }
                    if let Some(t_info) = notes.get(target.as_str()) {
                        if t_info.has_external { *has_ext = true; }
                    }
                    walk(target, notes, visited, depth + 1, max_depth, has_ext);
                }
            }
        }
    }
    walk(note_lower, all_notes, &mut visited, 0, &mut max_depth, &mut has_external);
    let origin = if has_external { "received" } else { "discovered" };
    (origin.to_string(), max_depth)
}

fn get_review_for_note(app: &tauri::AppHandle, note_path: &str) -> (Option<String>, bool) {
    let cdir = match crate::universe::active_constellation_dir(app) {
        Ok(d) => d,
        Err(_) => return (None, false),
    };
    let pulse_path = cdir.join("review-pulse.json");
    if !pulse_path.exists() { return (None, true); }
    if let Ok(data) = fs::read_to_string(&pulse_path) {
        if let Ok(pulse) = serde_json::from_str::<crate::review::ReviewPulseData>(&data) {
            let lr = pulse.last_reviewed.get(note_path).cloned();
            let is_due = lr.is_none(); // simplified: due if never reviewed
            return (lr, is_due);
        }
    }
    (None, true)
}

fn get_trails_for_note(lib: &Path, note_name: &str) -> Vec<String> {
    let mut trails: Vec<String> = Vec::new();
    let link_re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap();
    scan_trails_for_note(lib, note_name, &link_re, &mut trails);
    trails
}

fn scan_trails_for_note(dir: &Path, note_name: &str, link_re: &regex::Regex, trails: &mut Vec<String>) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_trails_for_note(&path, note_name, link_re, trails);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if crate::trails::is_trail_file_pub(&content) {
                    for cap in link_re.captures_iter(&content) {
                        if cap[1].trim().eq_ignore_ascii_case(note_name) {
                            // MIG-008 Step 2: trail label uses frontmatter
                            // title so canonical-named trails read as
                            // their human title in the inspector.
                            let trail_name = crate::libraries::note_display_name(&path, Some(&content));
                            if !trails.contains(&trail_name) {
                                trails.push(trail_name);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

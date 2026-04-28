//! Trails — Cognitive Engine Phase 8 (المسارات).
//!
//! Named, ordered sequences of notes. A Trail tells a story, traces an
//! argument, or records a research journey. First-class objects in the
//! knowledge graph.
//!
//! Data format: any .md file with `trail: true` in frontmatter.
//! Wikilinks in the body define the ordered sequence.

use serde::Serialize;
use std::fs;
use std::path::Path;

/// Summary info for trail listing.
#[derive(Debug, Clone, Serialize)]
pub struct TrailInfo {
    pub trail_path: String,
    pub title: String,
    pub description: String,
    pub note_count: usize,
}

/// Full trail data with resolved note paths.
#[derive(Debug, Clone, Serialize)]
pub struct TrailData {
    pub trail_path: String,
    pub title: String,
    pub description: String,
    pub notes: Vec<TrailNote>,
}

/// A single note in a trail sequence.
#[derive(Debug, Clone, Serialize)]
pub struct TrailNote {
    pub name: String,
    pub path: String,
    pub exists: bool,
}

/// List all trails in a library.
#[tauri::command]
pub fn list_trails(
    app: tauri::AppHandle,
    library_path: String,
) -> Result<Vec<TrailInfo>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let mut trails: Vec<TrailInfo> = Vec::new();
    scan_trails_recursive(Path::new(&library_path), &mut trails);
    Ok(trails)
}

/// Read a specific trail and resolve its note paths.
#[tauri::command]
pub fn read_trail(
    app: tauri::AppHandle,
    trail_path: String,
    library_path: String,
) -> Result<TrailData, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let content = fs::read_to_string(&trail_path)
        .map_err(|e| format!("Failed to read trail: {}", e))?;

    let (title, description) = parse_trail_frontmatter(&content);
    let note_names = parse_trail_links(&content);

    // Resolve note names to file paths
    let lib_path = Path::new(&library_path);
    let notes: Vec<TrailNote> = note_names.iter().map(|name| {
        let resolved = resolve_note_path(lib_path, name);
        TrailNote {
            name: name.clone(),
            path: resolved.clone().unwrap_or_default(),
            exists: resolved.is_some(),
        }
    }).collect();

    Ok(TrailData {
        trail_path,
        title,
        description,
        notes,
    })
}

fn scan_trails_recursive(dir: &Path, trails: &mut Vec<TrailInfo>) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_trails_recursive(&path, trails);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if is_trail_file(&content) {
                    let (title, description) = parse_trail_frontmatter(&content);
                    let note_count = parse_trail_links(&content).len();
                    trails.push(TrailInfo {
                        trail_path: path.to_string_lossy().to_string(),
                        title,
                        description,
                        note_count,
                    });
                }
            }
        }
    }
}

/// Check if a file has `trail: true` in ANY frontmatter block.
/// Handles double frontmatter (auto-generated created: block + user block).
/// Public wrapper for trail detection (used by inspector360).
pub fn is_trail_file_pub(content: &str) -> bool { is_trail_file(content) }
fn is_trail_file(content: &str) -> bool {
    let normalized = content.replace("\r\n", "\n").to_lowercase();
    // Check all ---...--- blocks, not just the first
    let mut pos = 0;
    while let Some(start) = normalized[pos..].find("---") {
        let block_start = pos + start + 3;
        if let Some(end) = normalized[block_start..].find("\n---") {
            let yaml = &normalized[block_start..block_start + end];
            if yaml.lines().any(|line| {
                let t = line.trim();
                t == "trail: true" || t == "trail: yes"
            }) {
                return true;
            }
            pos = block_start + end + 4;
        } else {
            break;
        }
    }
    false
}

/// Parse title and description from ALL frontmatter blocks.
fn parse_trail_frontmatter(content: &str) -> (String, String) {
    let normalized = content.replace("\r\n", "\n");
    let mut title = String::new();
    let mut description = String::new();

    let mut pos = 0;
    while let Some(start) = normalized[pos..].find("---") {
        let block_start = pos + start + 3;
        if let Some(end) = normalized[block_start..].find("\n---") {
            let yaml = &normalized[block_start..block_start + end];
            for line in yaml.lines() {
                let trimmed = line.trim();
                if let Some(val) = trimmed.strip_prefix("title:") {
                    title = val.trim().trim_matches('"').trim_matches('\'').to_string();
                } else if let Some(val) = trimmed.strip_prefix("description:") {
                    description = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }
            pos = block_start + end + 4;
        } else {
            break;
        }
    }

    (title, description)
}

/// Parse ordered wikilinks from trail body (after frontmatter).
fn parse_trail_links(content: &str) -> Vec<String> {
    let normalized = content.replace("\r\n", "\n");
    let body = if normalized.starts_with("---") {
        if let Some(end) = normalized[3..].find("\n---") {
            &normalized[3 + end + 4..] // skip past closing ---\n
        } else {
            &normalized
        }
    } else {
        &normalized
    };

    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap();
    let mut links: Vec<String> = Vec::new();
    for cap in re.captures_iter(body) {
        let name = cap[1].trim().to_string();
        if !links.contains(&name) {
            links.push(name);
        }
    }
    links
}

/// Resolve a note name to its file path within a library.
fn resolve_note_path(lib_path: &Path, note_name: &str) -> Option<String> {
    // Try exact match first
    let direct = lib_path.join(format!("{}.md", note_name));
    if direct.exists() {
        return Some(direct.to_string_lossy().to_string());
    }
    // Search recursively
    find_note_recursive(lib_path, note_name)
}

fn find_note_recursive(dir: &Path, note_name: &str) -> Option<String> {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return None };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            if let Some(found) = find_note_recursive(&path, note_name) {
                return Some(found);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            // MIG-008 Step 6 — correctness fix, not just a label fix.
            // The caller passes `note_name` derived from a wikilink's
            // target text (a title like "Apple Tree Fruit"). Comparing
            // it to `path.file_stem()` would never match for canonical
            // notes whose stem is `20260426T...`. The helper short-
            // circuits to the title field for canonical files so the
            // wikilink-to-path resolution actually works on Universes
            // that have run through canonical naming.
            let display = crate::libraries::note_display_name(&path, None);
            if display.eq_ignore_ascii_case(note_name) {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

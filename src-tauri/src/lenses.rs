//! Multi-Lens Views — Cognitive Engine Phase 9 (العدسات المتعددة).
//!
//! View the same library through multiple classification schemes.
//! No note duplication, no file movement. Switch lenses from sidebar.
//!
//! Lens types:
//!   - tag-hierarchy: groups notes by root tags
//!   - property-query: groups notes by frontmatter property value

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensDefinition {
    pub id: String,
    pub name: String,
    pub lens_type: String,          // "tag-hierarchy" | "property-query"
    pub root_tags: Option<Vec<String>>,  // for tag-hierarchy
    pub property: Option<String>,        // for property-query (e.g., "stage")
    pub values: Option<Vec<String>>,     // for property-query (e.g., ["fleeting", "literature"])
    #[serde(default)]
    pub built_in: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LensGroup {
    pub name: String,
    pub notes: Vec<LensNote>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LensNote {
    pub name: String,
    pub path: String,
}

/// Default built-in lenses.
fn default_lenses() -> Vec<LensDefinition> {
    vec![
        LensDefinition {
            id: "by-stage".to_string(),
            name: "By Stage".to_string(),
            lens_type: "property-query".to_string(),
            root_tags: None,
            property: Some("stage".to_string()),
            values: Some(vec!["fleeting".to_string(), "literature".to_string(), "permanent".to_string(), "synthesis".to_string()]),
            built_in: true,
        },
        LensDefinition {
            id: "by-topic".to_string(),
            name: "By Topic".to_string(),
            lens_type: "tag-hierarchy".to_string(),
            root_tags: None, // None = auto-detect all root tags
            property: None,
            values: None,
            built_in: true,
        },
    ]
}

/// List all lenses (built-in + custom).
#[tauri::command]
pub fn list_lenses(app: tauri::AppHandle) -> Result<Vec<LensDefinition>, String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let path = cdir.join("lenses.json");
    if path.exists() {
        let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let lenses: Vec<LensDefinition> = serde_json::from_str(&data).unwrap_or_else(|_| default_lenses());
        Ok(lenses)
    } else {
        let lenses = default_lenses();
        // Write defaults on first access
        if let Ok(json) = serde_json::to_string_pretty(&lenses) {
            let _ = fs::write(&path, json);
        }
        Ok(lenses)
    }
}

/// Save lenses (replaces all).
#[tauri::command]
pub fn save_lenses(app: tauri::AppHandle, lenses: Vec<LensDefinition>) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let path = cdir.join("lenses.json");
    let json = serde_json::to_string_pretty(&lenses).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to write lenses.json: {}", e))
}

/// Apply a lens to a library — returns grouped notes.
#[tauri::command]
pub fn apply_lens(
    app: tauri::AppHandle,
    library_path: String,
    lens: LensDefinition,
) -> Result<Vec<LensGroup>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    match lens.lens_type.as_str() {
        "property-query" => apply_property_lens(Path::new(&library_path), &lens),
        "tag-hierarchy" => apply_tag_lens(Path::new(&library_path), &lens),
        _ => Err(format!("Unknown lens type: {}", lens.lens_type)),
    }
}

/// Group notes by a frontmatter property value.
fn apply_property_lens(lib: &Path, lens: &LensDefinition) -> Result<Vec<LensGroup>, String> {
    let property = lens.property.as_deref().ok_or("Missing property for property-query lens")?;
    let values = lens.values.as_ref();

    let mut groups: HashMap<String, Vec<LensNote>> = HashMap::new();
    let mut unclassified: Vec<LensNote> = Vec::new();

    // Initialize groups from defined values
    if let Some(vals) = values {
        for v in vals {
            groups.insert(v.to_lowercase(), Vec::new());
        }
    }

    scan_property_recursive(lib, property, &mut groups, &mut unclassified);

    let mut result: Vec<LensGroup> = Vec::new();
    if let Some(vals) = values {
        for v in vals {
            let key = v.to_lowercase();
            let notes = groups.remove(&key).unwrap_or_default();
            if !notes.is_empty() {
                // Capitalize first letter for display
                let display = format!("{}{}", &v[..1].to_uppercase(), &v[1..]);
                result.push(LensGroup { name: display, notes });
            }
        }
    }
    // Add any auto-discovered values not in the predefined list
    for (key, notes) in groups {
        if !notes.is_empty() {
            let display = format!("{}{}", &key[..1].to_uppercase(), &key[1..]);
            result.push(LensGroup { name: display, notes });
        }
    }
    if !unclassified.is_empty() {
        result.push(LensGroup { name: "Unclassified".to_string(), notes: unclassified });
    }

    Ok(result)
}

/// Group notes by tags.
fn apply_tag_lens(lib: &Path, lens: &LensDefinition) -> Result<Vec<LensGroup>, String> {
    let root_filter = lens.root_tags.as_ref();
    let tag_re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").map_err(|e| e.to_string())?;

    let mut groups: HashMap<String, Vec<LensNote>> = HashMap::new();
    let mut untagged: Vec<LensNote> = Vec::new();

    scan_tags_lens_recursive(lib, &tag_re, root_filter, &mut groups, &mut untagged);

    let mut result: Vec<LensGroup> = groups.into_iter()
        .map(|(tag, notes)| LensGroup { name: format!("#{}", tag), notes })
        .collect();
    result.sort_by(|a, b| b.notes.len().cmp(&a.notes.len())); // largest groups first
    if !untagged.is_empty() {
        result.push(LensGroup { name: "Untagged".to_string(), notes: untagged });
    }

    Ok(result)
}

fn scan_property_recursive(
    dir: &Path,
    property: &str,
    groups: &mut HashMap<String, Vec<LensNote>>,
    unclassified: &mut Vec<LensNote>,
) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_property_recursive(&path, property, groups, unclassified);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            // MIG-008 Step 4: lens groupings show frontmatter title.
            let note_name = crate::libraries::note_display_name(&path, None);
            let note = LensNote { name: note_name, path: path.to_string_lossy().to_string() };

            if let Ok(content) = fs::read_to_string(&path) {
                let value = extract_frontmatter_value(&content, property);
                if let Some(val) = value {
                    let key = val.to_lowercase();
                    groups.entry(key).or_default().push(note);
                } else {
                    unclassified.push(note);
                }
            } else {
                unclassified.push(note);
            }
        }
    }
}

fn scan_tags_lens_recursive(
    dir: &Path,
    tag_re: &regex::Regex,
    root_filter: Option<&Vec<String>>,
    groups: &mut HashMap<String, Vec<LensNote>>,
    untagged: &mut Vec<LensNote>,
) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_tags_lens_recursive(&path, tag_re, root_filter, groups, untagged);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // MIG-008 Step 4: tag-lens groupings show frontmatter title.
                // Pass already-read content so canonical detection short-
                // circuits to the title field directly.
                let note_name = crate::libraries::note_display_name(&path, Some(&content));
                let mut found_tags: Vec<String> = Vec::new();
                for cap in tag_re.captures_iter(&content) {
                    let tag = cap[1].to_string();
                    // Get root tag (first segment before /)
                    let root = tag.split('/').next().unwrap_or(&tag).to_lowercase();
                    if let Some(filter) = root_filter {
                        if filter.iter().any(|f| f.to_lowercase().trim_start_matches('#') == root) {
                            if !found_tags.contains(&root) { found_tags.push(root); }
                        }
                    } else {
                        if !found_tags.contains(&root) { found_tags.push(root); }
                    }
                }

                let note = LensNote { name: note_name, path: path.to_string_lossy().to_string() };
                if found_tags.is_empty() {
                    untagged.push(note);
                } else {
                    for tag in &found_tags {
                        groups.entry(tag.clone()).or_default().push(note.clone());
                    }
                }
            }
        }
    }
}

/// Extract a single frontmatter property value (checks all --- blocks).
fn extract_frontmatter_value(content: &str, property: &str) -> Option<String> {
    let normalized = content.replace("\r\n", "\n");
    let prop_lower = property.to_lowercase();
    let mut pos = 0;
    while let Some(start) = normalized[pos..].find("---") {
        let block_start = pos + start + 3;
        if let Some(end) = normalized[block_start..].find("\n---") {
            let yaml = &normalized[block_start..block_start + end];
            for line in yaml.lines() {
                let trimmed = line.trim().to_lowercase();
                if let Some(val) = trimmed.strip_prefix(&format!("{}:", prop_lower)) {
                    let v = val.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !v.is_empty() {
                        return Some(v);
                    }
                }
            }
            pos = block_start + end + 4;
        } else {
            break;
        }
    }
    None
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
// tauri::Manager unused — removed

// ─── Security ───

/// Validate that a file path is within a registered vault or the active universe's bases directory.
fn validate_base_path(app: &tauri::AppHandle, file_path: &str) -> Result<(), String> {
    let target = fs::canonicalize(file_path)
        .or_else(|_| {
            // File may not exist yet (save); canonicalize parent
            Path::new(file_path).parent()
                .ok_or_else(|| "Invalid path".to_string())
                .and_then(|p| fs::canonicalize(p).map_err(|e| e.to_string()))
        })
        .map_err(|_| "Cannot resolve file path.".to_string())?;

    // Check if path is within the active universe directory
    if let Ok(universe_dir) = crate::universe::active_universe_dir(app) {
        if let Ok(canon_universe) = fs::canonicalize(&universe_dir) {
            if target.starts_with(&canon_universe) {
                return Ok(());
            }
        }
    }

    // Check if path is within any registered vault
    let vaults = crate::libraries::load_libraries_pub(app);
    for vault in &vaults {
        if let Ok(canon_vault) = fs::canonicalize(&vault.path) {
            if target.starts_with(&canon_vault) {
                return Ok(());
            }
        }
    }

    Err("Path is outside of registered vaults and universe directory.".to_string())
}

// ─── Data Structures ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSource {
    #[serde(rename = "type")]
    pub source_type: String,   // "folder" | "tag" | "all"
    pub path: Option<String>,  // folder path (relative to vault root)
    pub tag: Option<String>,   // tag filter
    #[serde(rename = "includeSubfolders", default = "default_true")]
    pub include_subfolders: bool,
    #[serde(rename = "selectedVaults", default)]
    pub selected_vaults: Vec<String>, // empty = all vaults; populated = only these vault names
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub property: String,
    pub label: Option<String>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub direction: Option<String>, // per-column direction override
}

fn default_width() -> u32 { 150 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub property: String,
    pub operator: String, // "is" | "is_not" | "contains" | "not_contains" | "gt" | "lt" | "is_empty" | "is_not_empty"
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRule {
    pub property: String,
    #[serde(default = "default_asc")]
    pub direction: String, // "asc" | "desc"
}

fn default_asc() -> String { "asc".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDefinition {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub name: String,
    pub source: BaseSource,
    #[serde(default)]
    pub columns: Vec<ColumnDef>,
    #[serde(default)]
    pub filters: Vec<FilterRule>,
    #[serde(default)]
    pub sorts: Vec<SortRule>,
    #[serde(default = "default_view")]
    pub view: String, // "table" | "card" | "list"
    #[serde(default = "default_auto")]
    pub direction: String, // "auto" | "rtl" | "ltr"
}

fn default_version() -> u32 { 1 }
fn default_view() -> String { "table".to_string() }
fn default_auto() -> String { "auto".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRow {
    pub file_path: String,
    pub file_name: String,
    pub library_name: String,
    pub library_path: String,
    pub properties: HashMap<String, String>,
    pub modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseQueryResult {
    pub rows: Vec<BaseRow>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub columns_detected: Vec<String>, // auto-detected property keys from data
}

// ─── Frontmatter Parser ───

/// Parse YAML frontmatter from a markdown note into a HashMap.
/// Returns None if no valid frontmatter found.
pub fn parse_frontmatter(content: &str) -> Option<HashMap<String, String>> {
    if !content.starts_with("---") {
        return None;
    }
    let lines: Vec<&str> = content.lines().collect();
    let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---")?;
    let end_idx = end_idx + 1; // offset from skip(1)

    let mut props = HashMap::new();
    let mut i = 1;
    while i < end_idx {
        let line = lines[i];
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim();
            // Skip indented lines (part of nested YAML)
            if key.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
                i += 1;
                continue;
            }
            let mut value = line[colon + 1..].trim().to_string();

            // Handle multi-line list values (key:\n  - item1\n  - item2)
            if value.is_empty() && i + 1 < end_idx {
                let next = lines.get(i + 1).unwrap_or(&"");
                if next.trim_start().starts_with("- ") {
                    let mut items = Vec::new();
                    let mut j = i + 1;
                    while j < end_idx {
                        let item_line = lines[j].trim();
                        if item_line.starts_with("- ") {
                            let item = item_line[2..].trim();
                            let item = item.trim_matches('"').trim_matches('\'');
                            items.push(item.to_string());
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    value = items.join(", ");
                    i = j;
                    if !key.is_empty() {
                        props.insert(key.to_string(), value);
                    }
                    continue;
                }
            }

            // Strip surrounding quotes
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }

            // Handle inline list [a, b, c]
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                let items: Vec<&str> = inner.split(',').map(|s| {
                    s.trim().trim_matches('"').trim_matches('\'')
                }).collect();
                value = items.join(", ");
            }

            if !key.is_empty() {
                props.insert(key.to_string(), value);
            }
        }
        i += 1;
    }

    Some(props)
}

// ─── Scanning ───

/// Recursively scan a directory for .md files and extract their frontmatter.
pub fn scan_folder(
    dir: &Path,
    library_name: &str,
    library_path: &str,
    include_subfolders: bool,
    rows: &mut Vec<BaseRow>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/folders
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if include_subfolders {
                scan_folder(&path, library_name, library_path, true, rows);
            }
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let properties = parse_frontmatter(&content).unwrap_or_default();
            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            let file_name = name.trim_end_matches(".md").to_string();

            rows.push(BaseRow {
                file_path: path.to_string_lossy().to_string(),
                file_name,
                library_name: library_name.to_string(),
                library_path: library_path.to_string(),
                properties,
                modified,
            });
        }
    }
}

/// Scan notes filtered by tag across a vault.
pub fn scan_by_tag(
    dir: &Path,
    library_name: &str,
    library_path: &str,
    tag: &str,
    rows: &mut Vec<BaseRow>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let tag_clean = tag.trim_start_matches('#').to_lowercase();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') { continue; }

        if path.is_dir() {
            scan_by_tag(&path, library_name, library_path, tag, rows);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let properties = parse_frontmatter(&content).unwrap_or_default();

            // Check if note has the tag in frontmatter or body
            let has_tag = {
                // Check frontmatter tags property
                let fm_tags = properties.get("tags").map(|t| t.to_lowercase()).unwrap_or_default();
                let has_fm = fm_tags.split(',').any(|t| t.trim() == tag_clean);
                // Check body for #tag
                let has_body = content.contains(&format!("#{}", tag_clean));
                has_fm || has_body
            };

            if !has_tag { continue; }

            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            rows.push(BaseRow {
                file_path: path.to_string_lossy().to_string(),
                file_name: name.trim_end_matches(".md").to_string(),
                library_name: library_name.to_string(),
                library_path: library_path.to_string(),
                properties,
                modified,
            });
        }
    }
}

// ─── Filtering ───

pub fn apply_filters(rows: &mut Vec<BaseRow>, filters: &[FilterRule]) {
    for filter in filters {
        rows.retain(|row| {
            let value = if filter.property == "file_name" {
                Some(&row.file_name as &str)
            } else {
                row.properties.get(&filter.property).map(|s| s.as_str())
            };

            match filter.operator.as_str() {
                "is" => value.map(|v| v.to_lowercase() == filter.value.to_lowercase()).unwrap_or(false),
                "is_not" => value.map(|v| v.to_lowercase() != filter.value.to_lowercase()).unwrap_or(true),
                "contains" => value.map(|v| v.to_lowercase().contains(&filter.value.to_lowercase())).unwrap_or(false),
                "not_contains" => value.map(|v| !v.to_lowercase().contains(&filter.value.to_lowercase())).unwrap_or(true),
                "gt" => {
                    if let (Some(v), Ok(fv)) = (value, filter.value.parse::<f64>()) {
                        v.parse::<f64>().map(|nv| nv > fv).unwrap_or(false)
                    } else { false }
                },
                "lt" => {
                    if let (Some(v), Ok(fv)) = (value, filter.value.parse::<f64>()) {
                        v.parse::<f64>().map(|nv| nv < fv).unwrap_or(false)
                    } else { false }
                },
                "is_empty" => value.map(|v| v.is_empty()).unwrap_or(true),
                "is_not_empty" => value.map(|v| !v.is_empty()).unwrap_or(false),
                _ => true,
            }
        });
    }
}

// ─── Sorting ───

// ─── Tauri Commands ───

#[tauri::command]
pub fn parse_base_file(app: tauri::AppHandle, file_path: String) -> Result<BaseDefinition, String> {
    // Security: validate path is within a vault or the active universe bases dir
    validate_base_path(&app, &file_path)?;

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read base file: {}", e))?;

    // Parse YAML
    serde_json::from_str::<BaseDefinition>(&content)
        .or_else(|_| {
            // Try parsing as YAML (simple key: value format)
            parse_base_yaml(&content)
        })
        .map_err(|e| format!("Failed to parse base file: {}", e))
}

/// Simple YAML-like parser for .base files.
/// For MVP, we use serde_json after converting YAML to JSON.
/// In production, add serde_yaml dependency.
fn parse_base_yaml(content: &str) -> Result<BaseDefinition, String> {
    // For now, try to parse as JSON first (the frontend will save as JSON)
    serde_json::from_str(content)
        .map_err(|e| format!("Invalid base file format: {}", e))
}

#[tauri::command]
pub fn query_base(
    _app: tauri::AppHandle,
    definition: BaseDefinition,
    library_paths: Vec<(String, String)>, // (library_name, library_path) pairs
) -> Result<BaseQueryResult, String> {
    let start = Instant::now();
    let mut rows = Vec::new();

    // Filter vaults by selectedVaults (empty = all vaults)
    let active_vaults: Vec<&(String, String)> = if definition.source.selected_vaults.is_empty() {
        library_paths.iter().collect()
    } else {
        library_paths.iter()
            .filter(|(vname, _)| definition.source.selected_vaults.contains(vname))
            .collect()
    };

    match definition.source.source_type.as_str() {
        "folder" => {
            let folder = definition.source.path.as_deref().unwrap_or("");
            for (vname, vpath) in &active_vaults {
                let full_path = Path::new(vpath).join(folder);
                if full_path.exists() && full_path.is_dir() {
                    scan_folder(
                        &full_path,
                        vname,
                        vpath,
                        definition.source.include_subfolders,
                        &mut rows,
                    );
                }
            }
        }
        "tag" => {
            let tag = definition.source.tag.as_deref().unwrap_or("");
            for (vname, vpath) in &active_vaults {
                scan_by_tag(
                    Path::new(vpath),
                    vname,
                    vpath,
                    tag,
                    &mut rows,
                );
            }
        }
        "all" => {
            for (vname, vpath) in &active_vaults {
                scan_folder(
                    Path::new(vpath),
                    vname,
                    vpath,
                    true,
                    &mut rows,
                );
            }
        }
        // Legacy "vault" type: treat as "all" with selectedVaults=[vault]
        "vault" => {
            let target = definition.source.selected_vaults.first()
                .or(definition.source.path.as_ref())
                .cloned()
                .unwrap_or_default();
            for (vname, vpath) in &library_paths {
                if *vname == target {
                    scan_folder(Path::new(vpath), vname, vpath, true, &mut rows);
                    break;
                }
            }
        }
        _ => return Err(format!("Unknown source type: {}", definition.source.source_type)),
    }

    let total_count = rows.len();

    // Apply filters
    apply_filters(&mut rows, &definition.filters);

    // Detect all property keys across results (for auto-column discovery)
    let mut columns_detected: Vec<String> = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();
    for row in &rows {
        for key in row.properties.keys() {
            if seen_keys.insert(key.clone()) {
                columns_detected.push(key.clone());
            }
        }
    }
    columns_detected.sort();

    // Apply sorts
    apply_sorts_fixed(&mut rows, &definition.sorts);

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(BaseQueryResult {
        rows,
        total_count,
        query_time_ms,
        columns_detected,
    })
}

/// Fixed sorting that handles owned strings properly.
pub fn apply_sorts_fixed(rows: &mut Vec<BaseRow>, sorts: &[SortRule]) {
    if sorts.is_empty() { return; }

    rows.sort_by(|a, b| {
        for sort in sorts {
            let av = if sort.property == "file_name" {
                a.file_name.clone()
            } else if sort.property == "modified" {
                a.modified.to_string()
            } else {
                a.properties.get(&sort.property).cloned().unwrap_or_default()
            };
            let bv = if sort.property == "file_name" {
                b.file_name.clone()
            } else if sort.property == "modified" {
                b.modified.to_string()
            } else {
                b.properties.get(&sort.property).cloned().unwrap_or_default()
            };

            // Try numeric comparison first
            let ord = match (av.parse::<f64>(), bv.parse::<f64>()) {
                (Ok(an), Ok(bn)) => an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal),
                _ => av.to_lowercase().cmp(&bv.to_lowercase()),
            };

            let ord = if sort.direction == "desc" { ord.reverse() } else { ord };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
}

#[tauri::command]
pub fn create_base(
    app: tauri::AppHandle,
    folder_path: String,
    file_name: String,
) -> Result<String, String> {
    // Validate folder is in a registered vault
    let vaults = crate::libraries::load_libraries_pub(&app);
    let folder = Path::new(&folder_path);
    let canon_folder = fs::canonicalize(folder)
        .map_err(|_| "Folder does not exist.".to_string())?;
    let in_vault = vaults.iter().any(|v| {
        fs::canonicalize(&v.path)
            .map(|vp| canon_folder.starts_with(vp))
            .unwrap_or(false)
    });
    if !in_vault {
        return Err("Access denied: path is not within any registered vault.".to_string());
    }
    if !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    // Sanitize name
    let safe_name = file_name.trim().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "");
    if safe_name.is_empty() {
        return Err("Invalid file name.".to_string());
    }

    let name = if safe_name.ends_with(".base") {
        safe_name
    } else {
        format!("{}.base", safe_name)
    };

    let file_path = folder.join(&name);
    if file_path.exists() {
        return Err("A file with this name already exists.".to_string());
    }

    // Build default BaseDefinition
    let display_name = name.trim_end_matches(".base").to_string();
    let definition = BaseDefinition {
        version: 1,
        name: display_name,
        source: BaseSource {
            source_type: "all".to_string(),
            path: None,
            tag: None,
            include_subfolders: true,
            selected_vaults: vec![],
        },
        columns: vec![],
        filters: vec![],
        sorts: vec![],
        view: "table".to_string(),
        direction: "auto".to_string(),
    };

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create base file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_base_file(app: tauri::AppHandle, file_path: String, definition: BaseDefinition) -> Result<(), String> {
    // Security: validate path is within a vault or the active universe bases dir
    validate_base_path(&app, &file_path)?;

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write base file: {}", e))
}

#[tauri::command]
pub fn update_note_property(
    app: tauri::AppHandle,
    file_path: String,
    key: String,
    value: String,
) -> Result<(), String> {
    // Security: validate path is in a vault
    let vaults = crate::libraries::load_libraries_pub(&app);
    let in_vault = vaults.iter().any(|v| {
        fs::canonicalize(&file_path).ok()
            .and_then(|fp| fs::canonicalize(&v.path).ok().map(|vp| fp.starts_with(vp)))
            .unwrap_or(false)
    });
    if !in_vault {
        return Err("Access denied: file is not in a registered vault.".to_string());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;

    let new_content = update_frontmatter_property(&content, &key, &value);

    fs::write(&file_path, new_content)
        .map_err(|e| format!("Failed to write note: {}", e))
}

/// Update or insert a single property in a note's YAML frontmatter.
fn update_frontmatter_property(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if !content.starts_with("---") {
        // No frontmatter — create one
        let mut result = format!("---\n{}: {}\n---\n", key, format_yaml_value(value));
        result.push_str(content);
        return result;
    }

    let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---");
    let end_idx = match end_idx {
        Some(i) => i + 1,
        None => {
            // Malformed frontmatter — prepend new one
            let mut result = format!("---\n{}: {}\n---\n", key, format_yaml_value(value));
            result.push_str(content);
            return result;
        }
    };

    // Check if property already exists
    let mut found = false;
    let mut new_lines: Vec<String> = Vec::new();
    new_lines.push("---".to_string());

    let mut i = 1;
    while i < end_idx {
        let line = lines[i];
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim();
            if !k.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                if k == key {
                    // Replace existing value
                    new_lines.push(format!("{}: {}", key, format_yaml_value(value)));
                    found = true;
                    // Skip any continuation lines (multi-line list)
                    i += 1;
                    while i < end_idx && (lines[i].starts_with("  - ") || lines[i].starts_with("  ")) {
                        if lines[i].trim().starts_with("- ") {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
        i += 1;
    }

    if !found {
        new_lines.push(format!("{}: {}", key, format_yaml_value(value)));
    }

    new_lines.push("---".to_string());

    // Append body (everything after frontmatter)
    for line in &lines[end_idx + 1..] {
        new_lines.push(line.to_string());
    }

    new_lines.join("\n")
}

// ─── Workspace-level Base Storage ───

/// Get the workspace bases directory: {active_universe}/.constellation/bases/
fn workspace_bases_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    let bases_dir = cdir.join("bases");
    fs::create_dir_all(&bases_dir).map_err(|e| format!("Failed to create bases dir: {}", e))?;
    Ok(bases_dir)
}

/// Workspace base entry returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBaseEntry {
    pub id: String,        // file stem (e.g. "My Research")
    pub name: String,      // display name from definition
    pub path: String,      // full file path
    pub modified: u64,     // last modified timestamp
}

#[tauri::command]
pub fn list_workspace_bases(app: tauri::AppHandle) -> Result<Vec<WorkspaceBaseEntry>, String> {
    let dir = workspace_bases_dir(&app)?;
    let mut entries = Vec::new();

    let read = fs::read_dir(&dir).map_err(|e| format!("Failed to read workspace bases: {}", e))?;
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "base").unwrap_or(false) {
            let id = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Try to read the name from the definition
            let name = fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<BaseDefinition>(&c).ok())
                .map(|d| d.name)
                .unwrap_or_else(|| id.clone());

            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            entries.push(WorkspaceBaseEntry {
                id,
                name,
                path: path.to_string_lossy().to_string(),
                modified,
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

#[tauri::command]
pub fn create_workspace_base(
    app: tauri::AppHandle,
    file_name: String,
) -> Result<String, String> {
    let dir = workspace_bases_dir(&app)?;

    // Sanitize name
    let safe_name = file_name.trim().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "");
    if safe_name.is_empty() {
        return Err("Invalid file name.".to_string());
    }

    let name = if safe_name.ends_with(".base") {
        safe_name
    } else {
        format!("{}.base", safe_name)
    };

    let file_path = dir.join(&name);
    if file_path.exists() {
        return Err("A base with this name already exists.".to_string());
    }

    let display_name = name.trim_end_matches(".base").to_string();
    let definition = BaseDefinition {
        version: 1,
        name: display_name,
        source: BaseSource {
            source_type: "all".to_string(),
            path: None,
            tag: None,
            include_subfolders: true,
            selected_vaults: vec![],
        },
        columns: vec![],
        filters: vec![],
        sorts: vec![],
        view: "table".to_string(),
        direction: "auto".to_string(),
    };

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to create workspace base: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
    definition: BaseDefinition,
) -> Result<(), String> {
    // Validate the path is inside the workspace bases directory
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Invalid workspace bases directory.".to_string())?;
    // For new files that don't exist yet, canonicalize the parent directory and
    // append only the filename — avoids raw-path starts_with bypass via ".." components.
    let canon_target = if target.exists() {
        fs::canonicalize(target)
            .map_err(|_| "Invalid target path.".to_string())?
    } else {
        let parent = target.parent().ok_or("Invalid target path.".to_string())?;
        let canon_parent = fs::canonicalize(parent)
            .map_err(|_| "Parent directory does not exist.".to_string())?;
        let fname = target.file_name().ok_or("Invalid file name.".to_string())?;
        canon_parent.join(fname)
    };

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write workspace base: {}", e))
}

#[tauri::command]
pub fn delete_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<(), String> {
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);

    // Validate path is inside workspace bases directory
    let canon_target = fs::canonicalize(target)
        .map_err(|_| "File does not exist.".to_string())?;
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Workspace directory not found.".to_string())?;

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete workspace base: {}", e))
}

#[tauri::command]
pub fn parse_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<BaseDefinition, String> {
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);

    // Validate path is inside workspace bases directory
    let canon_target = fs::canonicalize(target)
        .map_err(|_| "File does not exist.".to_string())?;
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Workspace directory not found.".to_string())?;

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read workspace base: {}", e))?;

    serde_json::from_str::<BaseDefinition>(&content)
        .map_err(|e| format!("Failed to parse workspace base: {}", e))
}

/// Format a value for YAML output.
/// Quotes strings that contain special characters.
fn format_yaml_value(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    // Check if value needs quoting
    if value.contains(':') || value.contains('#') || value.contains('\'')
        || value.contains('"') || value.contains('\n') || value.starts_with(' ')
        || value.ends_with(' ') || value.starts_with('[') || value.starts_with('{')
    {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

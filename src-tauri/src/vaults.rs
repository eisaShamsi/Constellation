use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileEntry>>,
    pub extension: Option<String>,
}

/// Get the path to the vaults config file.
fn vaults_config_path(app: &tauri::AppHandle) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
    fs::create_dir_all(&app_dir).ok();
    app_dir.join("vaults.json")
}

/// Load registered vaults from config.
fn load_vaults(app: &tauri::AppHandle) -> Vec<VaultInfo> {
    let path = vaults_config_path(app);
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

/// Save registered vaults to config.
fn save_vaults(app: &tauri::AppHandle, vaults: &[VaultInfo]) -> Result<(), String> {
    let path = vaults_config_path(app);
    let data = serde_json::to_string_pretty(vaults).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Failed to save vaults config: {}", e))
}

/// List all registered vaults.
#[tauri::command]
pub fn list_vaults(app: tauri::AppHandle) -> Vec<VaultInfo> {
    load_vaults(&app)
}

/// Add a vault by its folder path.
#[tauri::command]
pub fn add_vault(app: tauri::AppHandle, path: String) -> Result<VaultInfo, String> {
    let vault_path = Path::new(&path);

    if !vault_path.exists() || !vault_path.is_dir() {
        return Err("Path does not exist or is not a folder.".to_string());
    }

    // Check if it looks like an Obsidian vault (has .obsidian folder)
    let obsidian_dir = vault_path.join(".obsidian");
    if !obsidian_dir.exists() {
        return Err("This folder does not appear to be an Obsidian vault (no .obsidian folder found).".to_string());
    }

    let mut vaults = load_vaults(&app);

    // Check for duplicates
    if vaults.iter().any(|v| v.path == path) {
        return Err("This vault is already registered.".to_string());
    }

    let name = vault_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unnamed Vault".to_string());

    let id = format!("vault_{}", uuid_simple());

    let vault = VaultInfo {
        id: id.clone(),
        name,
        path: path.clone(),
    };

    vaults.push(vault.clone());
    save_vaults(&app, &vaults)?;

    Ok(vault)
}

/// Remove a vault by ID (does NOT delete any files).
#[tauri::command]
pub fn remove_vault(app: tauri::AppHandle, vault_id: String) -> Result<(), String> {
    let mut vaults = load_vaults(&app);
    let before = vaults.len();
    vaults.retain(|v| v.id != vault_id);

    if vaults.len() == before {
        return Err("Vault not found.".to_string());
    }

    save_vaults(&app, &vaults)
}

/// Read the file tree of a vault (up to 2 levels deep for performance).
#[tauri::command]
pub fn read_vault_tree(path: String, max_depth: Option<u32>) -> Result<Vec<FileEntry>, String> {
    let vault_path = Path::new(&path);
    if !vault_path.exists() {
        return Err("Vault path does not exist.".to_string());
    }

    let depth = max_depth.unwrap_or(2);
    Ok(read_dir_recursive(vault_path, 0, depth))
}

/// Read the content of a file inside a vault.
#[tauri::command]
pub fn read_note(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist.".to_string());
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Write content to a markdown file inside a vault.
#[tauri::command]
pub fn write_note(file_path: String, content: String) -> Result<(), String> {
    let path = Path::new(&file_path);

    // Safety: only allow writing .md files
    match path.extension().and_then(|e| e.to_str()) {
        Some("md") => {}
        _ => return Err("Can only write to .md files.".to_string()),
    }

    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err("Parent directory does not exist.".to_string());
            }
        }
    }

    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStats {
    pub vault_id: String,
    pub name: String,
    pub path: String,
    pub star_count: u32,
    pub folder_count: u32,
    pub recent_stars: Vec<StarInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarInfo {
    pub name: String,
    pub path: String,
    pub vault_id: String,
    pub vault_name: String,
    pub modified: u64,
    pub preview: String,
}

/// Get stats for all vaults — star counts, folder counts, recent stars.
#[tauri::command]
pub fn get_all_vault_stats(app: tauri::AppHandle) -> Vec<VaultStats> {
    let vaults = load_vaults(&app);
    vaults.iter().map(|v| {
        let (star_count, folder_count) = count_contents(Path::new(&v.path));
        let recent_stars = get_recent_notes(Path::new(&v.path), &v.id, &v.name, 5);
        VaultStats {
            vault_id: v.id.clone(),
            name: v.name.clone(),
            path: v.path.clone(),
            star_count,
            folder_count,
            recent_stars,
        }
    }).collect()
}

/// Search across all vaults for notes matching a query.
#[tauri::command]
pub fn search_stars(app: tauri::AppHandle, query: String) -> Vec<StarInfo> {
    let vaults = load_vaults(&app);
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for vault in &vaults {
        search_notes_recursive(
            Path::new(&vault.path),
            &vault.id,
            &vault.name,
            &query_lower,
            &mut results,
            0,
        );
    }

    // Sort by relevance (name match first, then content match)
    results.sort_by(|a, b| {
        let a_name_match = a.name.to_lowercase().contains(&query_lower);
        let b_name_match = b.name.to_lowercase().contains(&query_lower);
        b_name_match.cmp(&a_name_match).then(b.modified.cmp(&a.modified))
    });

    results.truncate(50); // Limit results
    results
}

fn count_contents(dir: &Path) -> (u32, u32) {
    let mut stars = 0u32;
    let mut folders = 0u32;
    count_recursive(dir, &mut stars, &mut folders, 0);
    (stars, folders)
}

fn count_recursive(dir: &Path, stars: &mut u32, folders: &mut u32, depth: u32) {
    if depth > 20 { return; } // Prevent stack overflow from deep/circular structures
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        // Skip symlinks to prevent circular recursion
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            *folders += 1;
            count_recursive(&path, stars, folders, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            *stars += 1;
        }
    }
}

fn get_recent_notes(dir: &Path, vault_id: &str, vault_name: &str, limit: usize) -> Vec<StarInfo> {
    let mut notes = Vec::new();
    collect_notes_recursive(dir, vault_id, vault_name, &mut notes, 0);
    notes.sort_by(|a, b| b.modified.cmp(&a.modified));
    notes.truncate(limit);
    notes
}

/// Safely truncate a string to approximately `max_len` characters.
fn safe_truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

fn collect_notes_recursive(dir: &Path, vault_id: &str, vault_name: &str, notes: &mut Vec<StarInfo>, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            collect_notes_recursive(&path, vault_id, vault_name, notes, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            let preview = fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .filter(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            let preview = safe_truncate(&preview, 120);

            notes.push(StarInfo {
                name: name.trim_end_matches(".md").to_string(),
                path: path.to_string_lossy().to_string(),
                vault_id: vault_id.to_string(),
                vault_name: vault_name.to_string(),
                modified,
                preview,
            });
        }
    }
}

fn search_notes_recursive(dir: &Path, vault_id: &str, vault_name: &str, query: &str, results: &mut Vec<StarInfo>, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            search_notes_recursive(&path, vault_id, vault_name, query, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let name_clean = name.trim_end_matches(".md").to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            let name_match = name_clean.to_lowercase().contains(query);
            let content_match = content.to_lowercase().contains(query);

            if name_match || content_match {
                let modified = entry.metadata()
                    .and_then(|m| m.modified())
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                    .unwrap_or(0);

                let preview = if content_match {
                    content.lines()
                        .find(|l| l.to_lowercase().contains(query))
                        .unwrap_or("")
                        .to_string()
                } else {
                    content.lines()
                        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                        .take(1)
                        .collect::<String>()
                };
                let preview = safe_truncate(&preview, 120);

                results.push(StarInfo {
                    name: name_clean,
                    path: path.to_string_lossy().to_string(),
                    vault_id: vault_id.to_string(),
                    vault_name: vault_name.to_string(),
                    modified,
                    preview,
                });
            }
        }
    }
}

/// Create a new markdown note inside a vault folder.
#[tauri::command]
pub fn create_note(folder_path: String, file_name: String) -> Result<String, String> {
    let folder = Path::new(&folder_path);
    if !folder.exists() || !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    let name = if file_name.ends_with(".md") {
        file_name
    } else {
        format!("{}.md", file_name)
    };

    let file_path = folder.join(&name);
    if file_path.exists() {
        return Err("A file with this name already exists.".to_string());
    }

    let initial = format!("---\n---\n\n");
    fs::write(&file_path, initial)
        .map_err(|e| format!("Failed to create note: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Create a new folder inside a vault.
#[tauri::command]
pub fn create_folder(parent_path: String, folder_name: String) -> Result<String, String> {
    let parent = Path::new(&parent_path);
    if !parent.exists() || !parent.is_dir() {
        return Err("Parent directory does not exist.".to_string());
    }

    let folder_path = parent.join(&folder_name);
    if folder_path.exists() {
        return Err("A folder with this name already exists.".to_string());
    }

    fs::create_dir(&folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    Ok(folder_path.to_string_lossy().to_string())
}

/// Rename a file or folder.
#[tauri::command]
pub fn rename_item(old_path: String, new_path: String) -> Result<(), String> {
    let old = Path::new(&old_path);
    if !old.exists() {
        return Err("Item does not exist.".to_string());
    }

    let new_p = Path::new(&new_path);
    if new_p.exists() {
        return Err("An item with this name already exists.".to_string());
    }

    fs::rename(old, new_p)
        .map_err(|e| format!("Failed to rename: {}", e))
}

/// Delete a file or folder (permanent delete).
#[tauri::command]
pub fn delete_item(path: String, permanent: Option<bool>) -> Result<(), String> {
    let target = Path::new(&path);
    if !target.exists() {
        return Err("Item does not exist.".to_string());
    }

    let _ = permanent; // For now, always permanent delete
    if target.is_dir() {
        fs::remove_dir_all(target)
            .map_err(|e| format!("Failed to delete folder: {}", e))
    } else {
        fs::remove_file(target)
            .map_err(|e| format!("Failed to delete file: {}", e))
    }
}

/// Resolve a wikilink target to an actual file path within a vault.
#[tauri::command]
pub fn resolve_wikilink(vault_path: String, target: String) -> Result<Option<String>, String> {
    let vault_dir = Path::new(&vault_path);
    if !vault_dir.exists() {
        return Err("Vault path does not exist.".to_string());
    }

    let target_lower = target.to_lowercase();
    let mut matches: Vec<PathBuf> = Vec::new();
    find_note_by_name(vault_dir, &target_lower, &mut matches, 0);

    if matches.is_empty() {
        return Ok(None);
    }

    // Prefer shortest path (closest to vault root)
    matches.sort_by_key(|p| p.to_string_lossy().len());
    Ok(Some(matches[0].to_string_lossy().to_string()))
}

fn find_note_by_name(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            find_note_by_name(&path, target, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let stem = name.trim_end_matches(".md").to_lowercase();
            if stem == *target {
                results.push(path);
            }
        }
    }
}

/// Read Obsidian's appearance.json for a vault.
#[tauri::command]
pub fn read_obsidian_appearance(vault_path: String) -> Result<serde_json::Value, String> {
    let path = Path::new(&vault_path).join(".obsidian").join("appearance.json");
    if !path.exists() {
        // Return defaults
        return Ok(serde_json::json!({
            "accent_color": null,
            "base_font_size": null,
            "text_font_family": null,
            "monospace_font_family": null,
            "interface_font_family": null,
            "css_theme": null
        }));
    }

    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read appearance.json: {}", e))?;

    let raw: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse appearance.json: {}", e))?;

    // Map Obsidian's camelCase to our field names
    Ok(serde_json::json!({
        "accent_color": raw.get("accentColor").and_then(|v| v.as_str()),
        "base_font_size": raw.get("baseFontSize").and_then(|v| v.as_u64()),
        "text_font_family": raw.get("textFontFamily").and_then(|v| v.as_str()),
        "monospace_font_family": raw.get("monospaceFontFamily").and_then(|v| v.as_str()),
        "interface_font_family": raw.get("interfaceFontFamily").and_then(|v| v.as_str()),
        "css_theme": raw.get("cssTheme").and_then(|v| v.as_str())
    }))
}

/// Open a folder picker dialog and return the selected path.
#[tauri::command]
pub async fn pick_folder() -> Result<Option<String>, String> {
    // Use Tauri's dialog API via rfd (rust file dialog)
    let result = rfd::FileDialog::new()
        .set_title("Select Obsidian Vault Folder")
        .pick_folder();

    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

fn read_dir_recursive(dir: &Path, current_depth: u32, max_depth: u32) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/folders (starting with .)
        if name.starts_with('.') {
            continue;
        }
        // Skip symlinks to prevent circular recursion
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
            continue;
        }

        let is_dir = path.is_dir();
        let extension = if !is_dir {
            path.extension().map(|e| e.to_string_lossy().to_string())
        } else {
            None
        };

        // Only include markdown files and folders
        if !is_dir && extension.as_deref() != Some("md") {
            continue;
        }

        let children = if is_dir && current_depth < max_depth {
            Some(read_dir_recursive(&path, current_depth + 1, max_depth))
        } else if is_dir {
            Some(vec![]) // Indicate it's a folder but don't load children
        } else {
            None
        };

        entries.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            children,
            extension,
        });
    }

    // Sort: folders first, then files, alphabetically
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

/// Simple UUID-like generator without external crate.
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", timestamp)
}

// ─── Graph / Backlinks scanning ───

#[derive(Debug, Clone, Serialize)]
pub struct NoteLink {
    pub source_path: String,
    pub source_name: String,
    pub target: String,
    pub context: String,
}

/// Scan all notes in a vault and extract wikilinks from each.
#[tauri::command]
pub fn scan_vault_links(vault_path: String) -> Result<Vec<NoteLink>, String> {
    let mut links = Vec::new();
    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap();
    scan_links_recursive(Path::new(&vault_path), &re, &mut links);
    Ok(links)
}

fn scan_links_recursive(dir: &Path, re: &regex::Regex, links: &mut Vec<NoteLink>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_links_recursive(&path, re, links);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let source_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                for cap in re.captures_iter(&content) {
                    let target = cap[1].trim().to_string();
                    // Extract context: the line containing the link
                    let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
                    let line_start = content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let line_end = content[pos..].find('\n').map(|i| pos + i).unwrap_or(content.len());
                    let context = safe_truncate(&content[line_start..line_end], 120);

                    links.push(NoteLink {
                        source_path: path.to_string_lossy().to_string(),
                        source_name: source_name.clone(),
                        target,
                        context,
                    });
                }
            }
        }
    }
}

/// Scan all tags across a vault.
#[tauri::command]
pub fn scan_vault_tags(vault_path: String) -> Result<std::collections::HashMap<String, u32>, String> {
    let mut tags: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").unwrap();
    scan_tags_recursive(Path::new(&vault_path), &re, &mut tags);
    Ok(tags)
}

fn scan_tags_recursive(dir: &Path, re: &regex::Regex, tags: &mut std::collections::HashMap<String, u32>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_tags_recursive(&path, re, tags);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // Inline tags
                for cap in re.captures_iter(&content) {
                    let tag = cap[1].to_string();
                    *tags.entry(tag).or_insert(0) += 1;
                }
                // YAML tags
                if content.starts_with("---") {
                    if let Some(end) = content[3..].find("---") {
                        let yaml = &content[3..3+end];
                        for line in yaml.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("- ") {
                                // Check if inside tags: block
                                let tag = trimmed.trim_start_matches("- ").trim().trim_matches('"').trim_matches('\'');
                                if !tag.is_empty() && !tag.contains(':') && !tag.contains(' ') {
                                    // Only count if it looks like a tag
                                    if tag.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '-' || c == '_') {
                                        // We'll count it if there was a tags: line before
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Collect all note names in a vault (for autocomplete).
#[tauri::command]
pub fn collect_vault_notes(vault_path: String) -> Result<Vec<serde_json::Value>, String> {
    let mut notes = Vec::new();
    collect_notes_names_recursive(Path::new(&vault_path), &mut notes);
    Ok(notes)
}

fn collect_notes_names_recursive(dir: &Path, notes: &mut Vec<serde_json::Value>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            collect_notes_names_recursive(&path, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let note_name = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            notes.push(serde_json::json!({
                "name": note_name,
                "path": path.to_string_lossy().to_string()
            }));
        }
    }
}

/// Get daily note path for today.
#[tauri::command]
pub fn get_daily_note_path(vault_path: String, format: String, folder: String) -> Result<String, String> {
    let now = chrono::Local::now();
    let filename = now.format(&format).to_string();
    let daily_folder = if folder.is_empty() {
        Path::new(&vault_path).to_path_buf()
    } else {
        Path::new(&vault_path).join(&folder)
    };
    fs::create_dir_all(&daily_folder).map_err(|e| e.to_string())?;
    let file_path = daily_folder.join(format!("{}.md", filename));

    // Create the file if it doesn't exist
    if !file_path.exists() {
        let content = format!("---\ndate: {}\n---\n", now.format("%Y-%m-%d"));
        fs::write(&file_path, content).map_err(|e| e.to_string())?;
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// Update all links in a vault when a note is renamed.
#[tauri::command]
pub fn update_links_on_rename(vault_path: String, old_name: String, new_name: String) -> Result<u32, String> {
    let mut count = 0u32;
    update_links_recursive(Path::new(&vault_path), &old_name, &new_name, &mut count);
    Ok(count)
}

fn update_links_recursive(dir: &Path, old_name: &str, new_name: &str, count: &mut u32) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            update_links_recursive(&path, old_name, new_name, count);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let old_link = format!("[[{}]]", old_name);
                let new_link = format!("[[{}]]", new_name);
                if content.contains(&old_link) {
                    let updated = content.replace(&old_link, &new_link);
                    // Also handle [[old_name|display]]
                    let old_pipe = format!("[[{}|", old_name);
                    let new_pipe = format!("[[{}|", new_name);
                    let updated = updated.replace(&old_pipe, &new_pipe);
                    if updated != content {
                        let _ = fs::write(&path, updated);
                        *count += 1;
                    }
                }
            }
        }
    }
}

/// Read a note's content for preview (used by hover preview)
#[tauri::command]
pub fn read_note_preview(file_path: String, max_chars: usize) -> Result<String, String> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(safe_truncate(&content, max_chars))
}

/// Save a base64-encoded image from clipboard to the vault's attachments folder.
/// Returns the relative path suitable for embedding as `![[filename]]`.
#[tauri::command]
pub fn save_clipboard_image(vault_path: String, image_data: String) -> Result<String, String> {
    // Create attachments folder if it doesn't exist
    let attachments_dir = Path::new(&vault_path).join("attachments");
    if !attachments_dir.exists() {
        fs::create_dir_all(&attachments_dir)
            .map_err(|e| format!("Failed to create attachments folder: {}", e))?;
    }

    // Generate filename with timestamp
    let now = chrono::Local::now();
    let filename = format!("Pasted image {}.png", now.format("%Y%m%d%H%M%S"));
    let file_path = attachments_dir.join(&filename);

    // Decode base64 data (strip data URL prefix if present)
    let b64_data = if let Some(idx) = image_data.find(",") {
        &image_data[idx + 1..]
    } else {
        &image_data
    };

    use std::io::Write;
    let decoded = base64_decode(b64_data)
        .map_err(|e| format!("Failed to decode image data: {}", e))?;

    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create image file: {}", e))?;
    file.write_all(&decoded)
        .map_err(|e| format!("Failed to write image file: {}", e))?;

    Ok(filename)
}

/// Simple base64 decoder (no external crate needed)
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let table: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &b) in table.iter().enumerate() {
        lookup[b as usize] = i as u8;
    }

    let input = input.trim().replace('\n', "").replace('\r', "");
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() * 3 / 4);

    let mut i = 0;
    while i < bytes.len() {
        let mut buf = [0u8; 4];
        let mut count = 0;
        while count < 4 && i < bytes.len() {
            let b = bytes[i];
            i += 1;
            if b == b'=' || b == b' ' || b == b'\t' {
                if b == b'=' { count += 1; }
                continue;
            }
            let val = lookup[b as usize];
            if val == 255 { continue; }
            buf[count] = val;
            count += 1;
        }
        if count >= 2 {
            output.push((buf[0] << 2) | (buf[1] >> 4));
        }
        if count >= 3 {
            output.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if count >= 4 {
            output.push((buf[2] << 6) | buf[3]);
        }
    }

    Ok(output)
}

/// Export a note's rendered content as HTML
#[tauri::command]
pub fn export_note_html(file_path: String) -> Result<String, String> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(content)
}

/// Move item to system trash (or ".trash" folder inside vault)
#[tauri::command]
pub fn move_to_trash(path: String, vault_path: String) -> Result<(), String> {
    let trash_dir = Path::new(&vault_path).join(".trash");
    if !trash_dir.exists() {
        fs::create_dir_all(&trash_dir)
            .map_err(|e| format!("Failed to create .trash folder: {}", e))?;
    }

    let source = Path::new(&path);
    let file_name = source.file_name()
        .ok_or("Invalid path")?;
    let dest = trash_dir.join(file_name);

    fs::rename(&source, &dest)
        .map_err(|e| format!("Failed to move to trash: {}", e))?;

    Ok(())
}

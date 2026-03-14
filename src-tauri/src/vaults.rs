use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
// tauri::Manager unused — removed

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
    pub modified: Option<u64>,
}

/// Get the path to the vaults config file (in the active universe).
fn vaults_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let universe_dir = crate::universe::active_universe_dir(app)?;
    Ok(universe_dir.join("vaults.json"))
}

/// Load registered vaults from the active universe's vaults.json (own vaults only).
fn load_vaults(app: &tauri::AppHandle) -> Vec<VaultInfo> {
    let path = match vaults_config_path(app) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    if path.exists() {
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[vaults] Failed to read {}: {}", path.display(), e);
                return vec![];
            }
        };
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("[vaults] Corrupt JSON in {}: {}", path.display(), e);
            vec![]
        })
    } else {
        vec![]
    }
}

/// Load ALL vaults: own + child universe vaults (recursive, deduplicated).
/// This is what the frontend and query_base should use.
pub fn load_all_vaults(app: &tauri::AppHandle) -> Vec<VaultInfo> {
    match crate::universe::resolve_universe_vaults(app.clone()) {
        Ok(vaults) => vaults,
        Err(_) => load_vaults(app),
    }
}

/// Public accessor for other modules (e.g., bases.rs).
pub fn load_vaults_pub(app: &tauri::AppHandle) -> Vec<VaultInfo> {
    load_all_vaults(app)
}

/// Save registered vaults to the active universe's config.
fn save_vaults(app: &tauri::AppHandle, vaults: &[VaultInfo]) -> Result<(), String> {
    let path = vaults_config_path(app)?;
    let data = serde_json::to_string_pretty(vaults).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Failed to save vaults config: {}", e))
}

/// Validate that a file path is contained within a vault directory.
/// Prevents path traversal attacks by canonicalizing both paths.
fn validate_path_in_vault(file_path: &str, vault_path: &str) -> Result<PathBuf, String> {
    let vault_canon = fs::canonicalize(vault_path)
        .map_err(|_| "Invalid vault path.".to_string())?;
    let file = Path::new(file_path);
    // If the file doesn't exist yet, canonicalize the parent
    let file_canon = if file.exists() {
        fs::canonicalize(file).map_err(|_| "Invalid file path.".to_string())?
    } else {
        let parent = file.parent().ok_or("Invalid file path.".to_string())?;
        let parent_canon = fs::canonicalize(parent)
            .map_err(|_| "Parent directory does not exist.".to_string())?;
        parent_canon.join(file.file_name().ok_or("Invalid file name.".to_string())?)
    };
    if !file_canon.starts_with(&vault_canon) {
        return Err("Access denied: path is outside the vault.".to_string());
    }
    Ok(file_canon)
}

/// Validate that a path is within any registered vault (including child universe vaults)
/// or the active universe directory.
fn validate_path_in_any_vault(app: &tauri::AppHandle, file_path: &str) -> Result<PathBuf, String> {
    let vaults = load_all_vaults(app);
    for vault in &vaults {
        if let Ok(canon) = validate_path_in_vault(file_path, &vault.path) {
            return Ok(canon);
        }
    }
    // Also allow the active universe directory for workspace bases
    if let Ok(universe_dir) = crate::universe::active_universe_dir(app) {
        if let Ok(uni_canon) = fs::canonicalize(&universe_dir) {
            let file = Path::new(file_path);
            if let Ok(file_canon) = fs::canonicalize(file) {
                if file_canon.starts_with(&uni_canon) {
                    return Ok(file_canon);
                }
            }
        }
    }
    Err("Access denied: path is not within any registered vault.".to_string())
}

/// Sanitize a file or folder name to prevent path traversal.
fn sanitize_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Name cannot be empty.".to_string());
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Name contains invalid characters.".to_string());
    }
    Ok(name.to_string())
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
pub fn read_vault_tree(app: tauri::AppHandle, path: String, max_depth: Option<u32>) -> Result<Vec<FileEntry>, String> {
    // Validate the path is a registered vault (including child universe vaults)
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
    let vault_path = Path::new(&path);
    if !vault_path.exists() {
        return Err("Vault path does not exist.".to_string());
    }

    let depth = max_depth.unwrap_or(2);
    Ok(read_dir_recursive(vault_path, 0, depth))
}

/// Read the content of a file inside a vault.
#[tauri::command]
pub fn read_note(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    validate_path_in_any_vault(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist.".to_string());
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Extract headings from a note file.
#[tauri::command]
pub fn get_note_headings(app: tauri::AppHandle, file_path: String) -> Result<Vec<String>, String> {
    validate_path_in_any_vault(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut headings = Vec::new();
    let re = regex::Regex::new(r"(?m)^#{1,6}\s+(.+)$").unwrap();
    for cap in re.captures_iter(&content) {
        if let Some(m) = cap.get(1) {
            headings.push(m.as_str().trim().to_string());
        }
    }
    Ok(headings)
}

/// Write content to a markdown file inside a vault.
#[tauri::command]
pub fn write_note(app: tauri::AppHandle, file_path: String, content: String) -> Result<(), String> {
    validate_path_in_any_vault(&app, &file_path)?;
    let path = Path::new(&file_path);

    // Safety: only allow writing .md files, reject ADS on Windows
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.contains(':') {
            return Err("Invalid file name.".to_string());
        }
    }
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

/// Get stats for all vaults (own + child universe) — star counts, folder counts, recent stars.
#[tauri::command]
pub fn get_all_vault_stats(app: tauri::AppHandle) -> Vec<VaultStats> {
    let vaults = load_all_vaults(&app);
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
    let vaults = load_all_vaults(&app);
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
/// `initial_frontmatter` is optional YAML content (without delimiters) to insert between `---` markers.
#[tauri::command]
pub fn create_note(app: tauri::AppHandle, folder_path: String, file_name: String, initial_frontmatter: Option<String>) -> Result<String, String> {
    let safe_name = sanitize_name(&file_name)?;
    validate_path_in_any_vault(&app, &folder_path)?;
    let folder = Path::new(&folder_path);
    if !folder.exists() || !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    let name = if safe_name.ends_with(".md") {
        safe_name
    } else {
        format!("{}.md", safe_name)
    };

    let file_path = folder.join(&name);
    if file_path.exists() {
        return Err("A file with this name already exists.".to_string());
    }

    let fm = initial_frontmatter.unwrap_or_default();
    let initial = if fm.is_empty() {
        "---\n---\n\n".to_string()
    } else {
        format!("---\n{}\n---\n\n", fm.trim())
    };
    fs::write(&file_path, &initial)
        .map_err(|e| format!("Failed to create note: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Search notes by property key/value across all vaults.
#[tauri::command]
pub fn search_by_property(app: tauri::AppHandle, key: String, value: String) -> Vec<StarInfo> {
    let vaults = load_all_vaults(&app);
    let key_lower = key.to_lowercase();
    let value_lower = value.to_lowercase();
    let mut results = Vec::new();

    for vault in &vaults {
        search_property_recursive(
            Path::new(&vault.path),
            &vault.id,
            &vault.name,
            &key_lower,
            &value_lower,
            &mut results,
            0,
        );
    }

    results.sort_by(|a, b| b.modified.cmp(&a.modified));
    results.truncate(50);
    results
}

fn search_property_recursive(dir: &Path, vault_id: &str, vault_name: &str, key: &str, value: &str, results: &mut Vec<StarInfo>, depth: u32) {
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
            search_property_recursive(&path, vault_id, vault_name, key, value, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Quick check: must have frontmatter
            if !content.starts_with("---") { continue; }

            // Parse frontmatter lines
            let lines: Vec<&str> = content.lines().collect();
            let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---");
            let end_idx = match end_idx {
                Some(i) => i + 1,
                None => continue,
            };

            let mut matched = false;
            let mut match_preview = String::new();

            for line in &lines[1..end_idx] {
                if let Some(colon) = line.find(':') {
                    let k = line[..colon].trim().to_lowercase();
                    let v = line[colon+1..].trim().to_lowercase();
                    // Strip quotes
                    let v = v.trim_matches('"').trim_matches('\'');

                    if k == key {
                        if value.is_empty() || v.contains(value) {
                            matched = true;
                            match_preview = format!("{}: {}", line[..colon].trim(), line[colon+1..].trim());
                            break;
                        }
                    }
                }
            }

            if matched {
                let name_clean = name.trim_end_matches(".md").to_string();
                let modified = entry.metadata()
                    .and_then(|m| m.modified())
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                    .unwrap_or(0);

                results.push(StarInfo {
                    name: name_clean,
                    path: path.to_string_lossy().to_string(),
                    vault_id: vault_id.to_string(),
                    vault_name: vault_name.to_string(),
                    modified,
                    preview: safe_truncate(&match_preview, 120),
                });
            }
        }
    }
}

/// Create a new folder inside a vault.
#[tauri::command]
pub fn create_folder(app: tauri::AppHandle, parent_path: String, folder_name: String) -> Result<String, String> {
    let safe_name = sanitize_name(&folder_name)?;
    validate_path_in_any_vault(&app, &parent_path)?;
    let parent = Path::new(&parent_path);
    if !parent.exists() || !parent.is_dir() {
        return Err("Parent directory does not exist.".to_string());
    }

    let folder_path = parent.join(&safe_name);
    if folder_path.exists() {
        return Err("A folder with this name already exists.".to_string());
    }

    fs::create_dir(&folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    Ok(folder_path.to_string_lossy().to_string())
}

/// Rename a file or folder.
#[tauri::command]
pub fn rename_item(app: tauri::AppHandle, old_path: String, new_path: String) -> Result<(), String> {
    validate_path_in_any_vault(&app, &old_path)?;
    validate_path_in_any_vault(&app, &new_path)?;
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
pub fn delete_item(app: tauri::AppHandle, path: String, permanent: Option<bool>) -> Result<(), String> {
    validate_path_in_any_vault(&app, &path)?;
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
pub fn resolve_wikilink(app: tauri::AppHandle, vault_path: String, target: String) -> Result<Option<String>, String> {
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == vault_path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
    let vault_dir = Path::new(&vault_path);
    if !vault_dir.exists() {
        return Err("Vault path does not exist.".to_string());
    }

    let target_lower = target.to_lowercase();
    let mut matches: Vec<PathBuf> = Vec::new();
    find_note_by_name_or_alias(vault_dir, &target_lower, &mut matches, 0);

    if matches.is_empty() {
        return Ok(None);
    }

    // Prefer shortest path (closest to vault root)
    matches.sort_by_key(|p| p.to_string_lossy().len());
    Ok(Some(matches[0].to_string_lossy().to_string()))
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedLink {
    pub path: String,
    pub vault_name: String,
    pub vault_path: String,
    pub fragment: Option<String>,
}

/// Resolve a wikilink across all vaults. Searches current vault first, then others.
/// Supports `vault_name:note` syntax to target a specific vault.
/// Supports `note#heading` and `note#^block-id` — fragment is stripped before resolution and returned separately.
#[tauri::command]
pub fn resolve_wikilink_cross_vault(
    vaults: Vec<(String, String, String)>, // (vault_id, vault_name, vault_path)
    current_vault_path: String,
    target: String,
) -> Result<Option<ResolvedLink>, String> {
    // Strip fragment (#heading or #^block-id)
    let (base_target, fragment) = if let Some(hash_pos) = target.find('#') {
        (target[..hash_pos].to_string(), Some(target[hash_pos + 1..].to_string()))
    } else {
        (target.clone(), None)
    };

    // Check for vault:note syntax
    if let Some(colon_pos) = base_target.find(':') {
        let vault_prefix = base_target[..colon_pos].trim().to_lowercase();
        let note_target = base_target[colon_pos + 1..].trim().to_lowercase();
        if !note_target.is_empty() {
            for (_id, name, path) in &vaults {
                if name.to_lowercase() == vault_prefix {
                    let vault_dir = Path::new(path);
                    if !vault_dir.exists() { continue; }
                    let mut matches: Vec<PathBuf> = Vec::new();
                    find_note_by_name_or_alias(vault_dir, &note_target, &mut matches, 0);
                    if !matches.is_empty() {
                        matches.sort_by_key(|p| p.to_string_lossy().len());
                        return Ok(Some(ResolvedLink {
                            path: matches[0].to_string_lossy().to_string(),
                            vault_name: name.clone(),
                            vault_path: path.clone(),
                            fragment,
                        }));
                    }
                    return Ok(None);
                }
            }
        }
    }

    let target_lower = base_target.to_lowercase();

    // Search current vault first
    let current_dir = Path::new(&current_vault_path);
    if current_dir.exists() {
        let mut matches: Vec<PathBuf> = Vec::new();
        find_note_by_name_or_alias(current_dir, &target_lower, &mut matches, 0);
        if !matches.is_empty() {
            matches.sort_by_key(|p| p.to_string_lossy().len());
            let vault_name = vaults.iter()
                .find(|(_, _, p)| p == &current_vault_path)
                .map(|(_, n, _)| n.clone())
                .unwrap_or_default();
            return Ok(Some(ResolvedLink {
                path: matches[0].to_string_lossy().to_string(),
                vault_name,
                vault_path: current_vault_path,
                fragment,
            }));
        }
    }

    // Search other vaults
    for (_id, name, path) in &vaults {
        if *path == current_vault_path { continue; }
        let vault_dir = Path::new(path);
        if !vault_dir.exists() { continue; }
        let mut matches: Vec<PathBuf> = Vec::new();
        find_note_by_name_or_alias(vault_dir, &target_lower, &mut matches, 0);
        if !matches.is_empty() {
            matches.sort_by_key(|p| p.to_string_lossy().len());
            return Ok(Some(ResolvedLink {
                path: matches[0].to_string_lossy().to_string(),
                vault_name: name.clone(),
                vault_path: path.clone(),
                fragment,
            }));
        }
    }

    Ok(None)
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

/// Like find_note_by_name, but also checks frontmatter aliases.
fn find_note_by_name_or_alias(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
    // First try exact filename match (fast)
    find_note_by_name(dir, target, results, depth);
    if !results.is_empty() { return; }

    // If no filename match, scan for aliases
    find_note_by_alias(dir, target, results, depth);
}

fn find_note_by_alias(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
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
            find_note_by_alias(&path, target, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if has_alias(&content, target) {
                    results.push(path);
                }
            }
        }
    }
}

/// Check if a note's frontmatter contains a matching alias.
fn has_alias(content: &str, target: &str) -> bool {
    if !content.starts_with("---") { return false; }
    let end = match content[3..].find("\n---") {
        Some(pos) => pos + 3,
        None => return false,
    };
    let frontmatter = &content[3..end];
    // Look for aliases: [...] or aliases:\n- ...
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        // Inline YAML array: aliases: [a, b, c]
        if trimmed.starts_with("aliases:") {
            let value = trimmed["aliases:".len()..].trim();
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len()-1];
                for alias in inner.split(',') {
                    let a = alias.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                    if a == target { return true; }
                }
            } else if !value.is_empty() {
                // Single value: aliases: my alias
                let a = value.trim_matches('"').trim_matches('\'').to_lowercase();
                if a == target { return true; }
            }
        }
        // YAML list item: - alias
        if trimmed.starts_with("- ") {
            let a = trimmed[2..].trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if a == target { return true; }
        }
    }
    false
}

/// Read Obsidian's appearance.json for a vault.
#[tauri::command]
pub fn read_obsidian_appearance(app: tauri::AppHandle, vault_path: String) -> Result<serde_json::Value, String> {
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == vault_path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
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

        // Only include markdown files, .base files, and folders
        if !is_dir && !matches!(extension.as_deref(), Some("md") | Some("base")) {
            continue;
        }

        let modified = entry.metadata().ok().and_then(|m| {
            m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()))
        });

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
            modified,
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
    // Add random component to avoid collisions on low-resolution clocks (Windows ~100ns)
    let random: u32 = (timestamp as u32).wrapping_mul(2654435761) ^ std::process::id();
    format!("{:x}{:04x}", timestamp, random & 0xFFFF)
}

// ─── Graph / Backlinks scanning ───

#[derive(Debug, Clone, Serialize)]
pub struct NoteLink {
    pub source_path: String,
    pub source_name: String,
    pub target: String,
    pub context: String,
    pub vault_name: String,
    pub link_type: Option<String>,
}

/// Scan all notes in a vault and extract wikilinks from each.
#[tauri::command]
pub fn scan_vault_links(app: tauri::AppHandle, vault_path: String, vault_name: String) -> Result<Vec<NoteLink>, String> {
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == vault_path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
    let mut links = Vec::new();
    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").unwrap();
    scan_links_recursive(Path::new(&vault_path), &re, &mut links, &vault_name);
    Ok(links)
}

fn scan_links_recursive(dir: &Path, re: &regex::Regex, links: &mut Vec<NoteLink>, vault_name: &str) {
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
            scan_links_recursive(&path, re, links, vault_name);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let source_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                for cap in re.captures_iter(&content) {
                    let target = cap[1].trim().to_string();
                    // Extract link type from alias: [[note|type:related-to]]
                    let link_type = cap.get(2).and_then(|alias| {
                        let alias_str = alias.as_str().trim();
                        if alias_str.to_lowercase().starts_with("type:") {
                            Some(alias_str[5..].trim().to_string())
                        } else {
                            None
                        }
                    });
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
                        vault_name: vault_name.to_string(),
                        link_type,
                    });
                }
            }
        }
    }
}

/// Scan for unlinked mentions of a note name across all vaults.
/// Returns notes that mention the name as plain text but don't have a [[wikilink]] to it.
#[tauri::command]
pub fn scan_unlinked_mentions(
    app: tauri::AppHandle,
    note_name: String,
    note_path: String,
    vault_paths: Vec<(String, String)>, // (vault_name, vault_path)
) -> Result<Vec<NoteLink>, String> {
    let registered = load_all_vaults(&app);
    let wikilink_pattern = format!("[[{}]]", &note_name);
    let wikilink_pattern_lower = wikilink_pattern.to_lowercase();
    let word_re = match regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&note_name))) {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };

    let mut results = Vec::new();
    let cap = 50usize;

    for (vault_name, vault_path) in &vault_paths {
        if !registered.iter().any(|v| &v.path == vault_path) { continue; }
        scan_unlinked_recursive(
            Path::new(vault_path),
            &note_path,
            &word_re,
            &wikilink_pattern_lower,
            vault_name,
            &mut results,
            cap,
            0,
        );
        if results.len() >= cap { break; }
    }

    Ok(results)
}

fn scan_unlinked_recursive(
    dir: &Path,
    note_path: &str,
    word_re: &regex::Regex,
    wikilink_lower: &str,
    vault_name: &str,
    results: &mut Vec<NoteLink>,
    cap: usize,
    depth: u32,
) {
    if depth > 20 || results.len() >= cap { return; }
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        if results.len() >= cap { return; }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            scan_unlinked_recursive(&path, note_path, word_re, wikilink_lower, vault_name, results, cap, depth + 1);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            // Skip self
            if path.to_string_lossy() == note_path { continue; }

            if let Ok(content) = fs::read_to_string(&path) {
                let content_lower = content.to_lowercase();
                // Skip if already has a wikilink to this note
                if content_lower.contains(wikilink_lower) { continue; }

                // Check for plain text mention
                if let Some(m) = word_re.find(&content) {
                    let pos = m.start();
                    let line_start = content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let line_end = content[pos..].find('\n').map(|i| pos + i).unwrap_or(content.len());
                    let context = safe_truncate(&content[line_start..line_end], 120);
                    let source_name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("").to_string();
                    results.push(NoteLink {
                        source_path: path.to_string_lossy().to_string(),
                        source_name,
                        target: String::new(),
                        context,
                        vault_name: vault_name.to_string(),
                        link_type: None,
                    });
                }
            }
        }
    }
}

/// Scan all tags across a vault.
#[tauri::command]
pub fn scan_vault_tags(app: tauri::AppHandle, vault_path: String) -> Result<std::collections::HashMap<String, u32>, String> {
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == vault_path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
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

// ─── Index: Word Index ───
// Extracts every word from every note, counts total occurrences,
// tracks which notes each word appears in, detects bigrams,
// filters stopwords, and merges case variants.

#[derive(Debug, Clone, Serialize)]
pub struct IndexMention {
    pub note_path: String,
    pub note_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexEntry {
    pub term: String,
    pub count: u32,
    pub mentions: Vec<IndexMention>,
    pub is_compound: bool,
}

fn build_stopwords() -> std::collections::HashSet<&'static str> {
    let words: &[&str] = &[
        // English
        "the","be","to","of","and","a","in","that","have","i","it","for","not","on","with",
        "he","as","you","do","at","this","but","his","by","from","they","we","say","her","she",
        "or","an","will","my","one","all","would","there","their","what","so","up","out","if",
        "about","who","get","which","go","me","when","make","can","like","time","no","just",
        "him","know","take","people","into","year","your","good","some","could","them","see",
        "other","than","then","now","look","only","come","its","over","think","also","back",
        "after","use","two","how","our","work","first","well","way","even","new","want",
        "because","any","these","give","day","most","us","are","was","were","been","has","had",
        "did","does","may","might","must","shall","should","being","is","am","very","too",
        "each","every","both","few","more","much","own","same","such","where","here","let",
        "still","yet","while","per","via","etc","else","done","got","put","set","run",
        // Arabic
        "في","من","على","إلى","هذا","هذه","التي","الذي","عن","مع","هو","هي","كان","كانت",
        "ذلك","تلك","ما","لا","أن","إن","لم","لن","قد","ثم","أو","حتى","بين","عند","كل",
        "بعد","قبل","بعض","نحو","أي","أنه","أنها","لقد","فقط","هنا","هناك","منذ","حيث",
        "كما","إذا","عبر","ضد","خلال","حول","فيه","فيها","عليه","عليها","منه","منها",
        "به","بها","له","لها","لهم","هؤلاء","أولئك","وهو","وهي","ولا","ولم","إلا",
        "أما","إما","سوف","لكن","ليس","ليست","كذلك","أيضا","مثل","غير","دون","ضمن",
        "تلك","ذات","ذو","ذي","التي","اللذين","اللتين","اللواتي","الذين",
        // French
        "le","la","les","de","des","du","un","une","et","est","en","que","qui","dans","pour",
        "sur","avec","par","pas","il","elle","ce","se","au","aux","son","sa","ses","ont","sont",
        "mais","ou","où","ne","plus","tout","cette","mon","ton","nous","vous","ils","elles",
        "été","être","avoir","fait","comme","même","aussi","bien","très","peut","autre",
        // Spanish
        "el","la","los","las","de","del","un","una","en","que","es","por","con","para","se",
        "al","lo","su","como","más","no","ya","pero","sus","le","me","sin","sobre","este",
        "entre","cuando","muy","ser","hay","también","fue","todo","esta","son","dos","hasta",
        // German
        "der","die","das","und","in","den","von","zu","ist","mit","sich","des","ein","für",
        "auf","nicht","es","eine","auch","als","an","dem","so","ich","er","sie","hat","aus",
        "bei","nur","noch","wie","nach","über","aber","dann","war","mir","bis","doch","vor",
        "oder","sehr","durch","wenn","man","zum","zur","kann","sind","wird","vom","wir",
        // Russian
        "и","в","не","на","я","что","он","с","это","а","как","но","она","по","к","из","у",
        "за","так","то","все","мы","бы","от","до","же","вы","ее","его","для","их","уже",
        "при","без","ни","тот","эти","вот","чем","где","быть","был","была","были","нет",
        "или","если","них","нас","вас","ему","ней","ним","них","себя","есть","очень","еще",
        // Portuguese
        "o","a","os","as","de","da","do","em","no","na","um","uma","que","para","com","por",
        "se","mais","não","como","mas","foi","ao","dos","das","nos","nas","seu","sua","esse",
        // Turkish
        "bir","bu","ve","da","de","ile","için","olan","gibi","daha","çok","ama","ya","hem",
        "ne","var","ben","sen","biz","siz","her","hiç","kadar","sonra","önce","arasında",
        // Hindi
        "का","के","की","में","है","को","और","से","पर","ने","यह","वह","एक","हैं","था",
        "इस","उस","कि","जो","भी","नहीं","कर","हो","तो","ही","या","अपने","सब","कुछ",
    ];
    words.iter().copied().collect()
}

/// Scan all notes in a vault and build a word index.
#[tauri::command]
pub fn scan_vault_index(app: tauri::AppHandle, vault_path: String) -> Result<Vec<IndexEntry>, String> {
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == vault_path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
    let stopwords = build_stopwords();

    // word_key -> { casing_variants: HashMap<String,u32>, total_count, sources }
    let mut index: std::collections::HashMap<String, (
        std::collections::HashMap<String, u32>, // casing variants -> count
        u32,                                     // total count
        Vec<(String, String)>,                   // (path, note_name)
    )> = std::collections::HashMap::new();

    // bigram_key -> (display_form, total_count, sources)
    let mut bigrams: std::collections::HashMap<String, (String, u32, Vec<(String, String)>)> =
        std::collections::HashMap::new();

    let md_strip = regex::Regex::new(
        r"(?x)
          \!\[([^\]]*)\]\([^)]*\)   |  # images
          \[([^\]]*)\]\([^)]*\)      |  # markdown links
          \[\[([^\]|]+?)(?:\|[^\]]+?)?\]\] | # wikilinks -> keep inner text
          ```[\s\S]*?```             |  # fenced code blocks
          `[^`]+`                    |  # inline code
          \*\*([^*]+)\*\*           |  # bold -> keep inner
          \*([^*]+)\*               |  # italic -> keep inner
          __([^_]+)__               |  # bold alt
          _([^_]+)_                 |  # italic alt
          ~~([^~]+)~~               |  # strikethrough
          <[^>]+>                   |  # HTML tags
          ^---\s*$                  |  # horizontal rules
          ^\#{1,6}\s+                  # heading markers (keep text after)
        "
    ).unwrap();

    scan_index_words_recursive(
        Path::new(&vault_path), &md_strip, &stopwords, &mut index, &mut bigrams,
    );

    // Build single-word entries: pick most common casing variant
    let mut entries: Vec<IndexEntry> = index
        .into_values()
        .filter(|(_, count, _)| *count >= 2)
        .map(|(variants, count, sources)| {
            let term = variants.into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(s, _)| s)
                .unwrap_or_default();
            let mentions: Vec<IndexMention> = sources
                .into_iter()
                .map(|(note_path, note_name)| IndexMention { note_path, note_name })
                .collect();
            IndexEntry { term, count, mentions, is_compound: false }
        })
        .collect();

    // Build bigram entries (compound terms)
    let bigram_entries: Vec<IndexEntry> = bigrams
        .into_values()
        .filter(|(_, count, _)| *count >= 3)
        .map(|(term, count, sources)| {
            let mentions: Vec<IndexMention> = sources
                .into_iter()
                .map(|(note_path, note_name)| IndexMention { note_path, note_name })
                .collect();
            IndexEntry { term, count, mentions, is_compound: true }
        })
        .collect();

    entries.extend(bigram_entries);
    entries.sort_by(|a, b| a.term.to_lowercase().cmp(&b.term.to_lowercase()));
    Ok(entries)
}

fn is_same_script(a: &str, b: &str) -> bool {
    let ca = a.chars().next().unwrap_or(' ');
    let cb = b.chars().next().unwrap_or(' ');
    // Both ASCII Latin
    if ca.is_ascii_alphabetic() && cb.is_ascii_alphabetic() { return true; }
    // Both in same Unicode block (rough check: same high byte)
    let ba = (ca as u32) >> 8;
    let bb = (cb as u32) >> 8;
    ba == bb
}

fn scan_index_words_recursive(
    dir: &Path,
    md_strip: &regex::Regex,
    stopwords: &std::collections::HashSet<&str>,
    index: &mut std::collections::HashMap<String, (
        std::collections::HashMap<String, u32>, u32, Vec<(String, String)>,
    )>,
    bigrams: &mut std::collections::HashMap<String, (String, u32, Vec<(String, String)>)>,
) {
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
            scan_index_words_recursive(&path, md_strip, stopwords, index, bigrams);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let note_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let note_path = path.to_string_lossy().to_string();

                // Strip YAML frontmatter
                let body = if content.starts_with("---") {
                    if let Some(end) = content[3..].find("---") {
                        &content[3 + end + 3..]
                    } else {
                        content.as_str()
                    }
                } else {
                    content.as_str()
                };

                let cleaned = md_strip.replace_all(body, |caps: &regex::Captures| {
                    for i in 1..=8 {
                        if let Some(m) = caps.get(i) {
                            return m.as_str().to_string();
                        }
                    }
                    String::new()
                });

                let mut seen_in_note: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut seen_bigrams: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut prev_word: Option<String> = None;
                let mut prev_key: Option<String> = None;

                for word in cleaned.split(|c: char| !c.is_alphabetic() && c != '\'') {
                    let word = word.trim_matches('\'');
                    if word.is_empty() {
                        prev_word = None;
                        prev_key = None;
                        continue;
                    }
                    let char_count = word.chars().count();
                    let is_non_latin = word.chars().any(|c| !c.is_ascii_alphabetic());
                    if is_non_latin && char_count < 2 {
                        prev_word = None;
                        prev_key = None;
                        continue;
                    }
                    if !is_non_latin && char_count < 3 {
                        prev_word = None;
                        prev_key = None;
                        continue;
                    }

                    let key = word.to_lowercase();

                    // Skip stopwords for single-word index (but still use for bigram detection)
                    let is_stop = stopwords.contains(key.as_str());

                    if !is_stop {
                        let entry = index.entry(key.clone()).or_insert_with(|| {
                            (std::collections::HashMap::new(), 0, Vec::new())
                        });
                        // Track casing variant
                        *entry.0.entry(word.to_string()).or_insert(0) += 1;
                        entry.1 += 1;

                        if !seen_in_note.contains(&key) {
                            seen_in_note.insert(key.clone());
                            entry.2.push((note_path.clone(), note_name.clone()));
                        }
                    }

                    // Bigram detection: pair with previous non-stop word if same script
                    if let (Some(pw), Some(pk)) = (&prev_word, &prev_key) {
                        let prev_is_stop = stopwords.contains(pk.as_str());
                        if !is_stop && !prev_is_stop && is_same_script(pw, word) {
                            let bi_key = format!("{} {}", pk, key);
                            let bi_display = format!("{} {}", pw, word);
                            let bi_entry = bigrams.entry(bi_key.clone())
                                .or_insert_with(|| (bi_display, 0, Vec::new()));
                            bi_entry.1 += 1;
                            if !seen_bigrams.contains(&bi_key) {
                                seen_bigrams.insert(bi_key);
                                bi_entry.2.push((note_path.clone(), note_name.clone()));
                            }
                        }
                    }

                    prev_word = Some(word.to_string());
                    prev_key = Some(key);
                }
            }
        }
    }
}

/// Collect all note names in a vault (for autocomplete).
#[tauri::command]
pub fn collect_vault_notes(app: tauri::AppHandle, vault_path: String) -> Result<Vec<serde_json::Value>, String> {
    let vaults = load_all_vaults(&app);
    if !vaults.iter().any(|v| v.path == vault_path) {
        return Err("Access denied: not a registered vault.".to_string());
    }
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
pub fn get_daily_note_path(app: tauri::AppHandle, vault_path: String, format: String, folder: String) -> Result<String, String> {
    validate_path_in_any_vault(&app, &vault_path)?;
    if !folder.is_empty() {
        if folder.contains("..") || folder.contains('\\') || folder.starts_with('/') {
            return Err("Folder name contains invalid characters.".to_string());
        }
    }
    let now = chrono::Local::now();
    let filename = now.format(&format).to_string();
    let daily_folder = if folder.is_empty() {
        Path::new(&vault_path).to_path_buf()
    } else {
        Path::new(&vault_path).join(&folder)
    };
    // Validate the resolved path is still within the vault
    validate_path_in_vault(&daily_folder.to_string_lossy(), &vault_path)?;
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
pub fn update_links_on_rename(app: tauri::AppHandle, vault_path: String, old_name: String, new_name: String) -> Result<u32, String> {
    validate_path_in_any_vault(&app, &vault_path)?;
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
pub fn read_note_preview(app: tauri::AppHandle, file_path: String, max_chars: usize) -> Result<String, String> {
    validate_path_in_any_vault(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(safe_truncate(&content, max_chars))
}

/// Save a base64-encoded image from clipboard to the vault's attachments folder.
/// Returns the relative path suitable for embedding as `![[filename]]`.
#[tauri::command]
pub fn save_clipboard_image(app: tauri::AppHandle, vault_path: String, image_data: String) -> Result<String, String> {
    validate_path_in_any_vault(&app, &vault_path)?;
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
pub fn export_note_html(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    validate_path_in_any_vault(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(content)
}

/// Move item to system trash (or ".trash" folder inside vault)
#[tauri::command]
pub fn move_to_trash(_app: tauri::AppHandle, path: String, vault_path: String) -> Result<(), String> {
    validate_path_in_vault(&path, &vault_path)?;
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

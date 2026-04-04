use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
// tauri::Manager unused — removed

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub is_universe_notes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileEntry>>,
    pub extension: Option<String>,
    pub modified: Option<u64>,
    pub status: Option<String>,
}

/// Get the path to the libraries config file (in .constellation/).
fn libraries_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    Ok(cdir.join("libraries.json"))
}

/// Load registered libraries from the active universe's libraries.json (own libraries only).
fn load_libraries(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    let path = match libraries_config_path(app) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    if path.exists() {
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[libraries] Failed to read {}: {}", path.display(), e);
                return vec![];
            }
        };
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("[libraries] Corrupt JSON in {}: {}", path.display(), e);
            vec![]
        })
    } else {
        vec![]
    }
}

/// Load ALL libraries: own + child universe libraries (recursive, deduplicated).
/// This is what the frontend and query_base should use.
pub fn load_all_libraries(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    match crate::universe::resolve_universe_libraries(app.clone()) {
        Ok(libs) => libs,
        Err(_) => load_libraries(app),
    }
}

/// Public accessor for other modules (e.g., bases.rs).
pub fn load_libraries_pub(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    load_all_libraries(app)
}

/// Save registered libraries to the active universe's config.
fn save_libraries(app: &tauri::AppHandle, libraries: &[LibraryInfo]) -> Result<(), String> {
    let path = libraries_config_path(app)?;
    let data = serde_json::to_string_pretty(libraries).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Failed to save libraries config: {}", e))
}

/// Validate that a file path is contained within a library directory.
/// Prevents path traversal attacks by canonicalizing both paths.
fn validate_path_in_library(file_path: &str, library_path: &str) -> Result<PathBuf, String> {
    let library_canon = fs::canonicalize(library_path)
        .map_err(|_| "Invalid library path.".to_string())?;
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
    if !file_canon.starts_with(&library_canon) {
        return Err("Access denied: path is outside the library.".to_string());
    }
    Ok(file_canon)
}

/// Validate that a path is within any registered library (including child universe libraries)
/// or the active universe directory.
pub fn validate_path_in_any_library(app: &tauri::AppHandle, file_path: &str) -> Result<PathBuf, String> {
    let libraries = load_all_libraries(app);
    for lib in &libraries {
        if let Ok(canon) = validate_path_in_library(file_path, &lib.path) {
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
    Err("Access denied: path is not within any registered library.".to_string())
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

/// List all registered libraries.
#[tauri::command]
pub fn list_libraries(app: tauri::AppHandle) -> Vec<LibraryInfo> {
    load_libraries(&app)
}

/// Add a library by its folder path.
#[tauri::command]
pub fn add_library(app: tauri::AppHandle, path: String) -> Result<LibraryInfo, String> {
    let library_path = Path::new(&path);

    if !library_path.exists() || !library_path.is_dir() {
        return Err("Path does not exist or is not a folder.".to_string());
    }

    // Any directory is accepted as a library — no .obsidian or .md requirement

    let mut libraries = load_libraries(&app);

    // Check for duplicates
    if libraries.iter().any(|v| v.path == path) {
        return Err("This library is already registered.".to_string());
    }

    let name = library_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unnamed Library".to_string());

    let id = format!("library_{}", uuid_simple());

    let library = LibraryInfo {
        id: id.clone(),
        name,
        path: path.clone(),
        is_universe_notes: false,
    };

    libraries.push(library.clone());
    save_libraries(&app, &libraries)?;

    Ok(library)
}

/// Remove a library by ID (does NOT delete any files).
#[tauri::command]
pub fn remove_library(app: tauri::AppHandle, library_id: String) -> Result<(), String> {
    let mut libraries = load_libraries(&app);
    let before = libraries.len();
    libraries.retain(|v| v.id != library_id);

    if libraries.len() == before {
        return Err("Library not found.".to_string());
    }

    save_libraries(&app, &libraries)
}

/// Read the file tree of a library (up to 2 levels deep for performance).
#[tauri::command]
pub fn read_library_tree(app: tauri::AppHandle, path: String, max_depth: Option<u32>) -> Result<Vec<FileEntry>, String> {
    // Validate the path is a registered library (including child universe libraries)
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let library_path = Path::new(&path);
    if !library_path.exists() {
        return Err("Library path does not exist.".to_string());
    }

    let depth = max_depth.unwrap_or(2);
    Ok(read_dir_recursive(library_path, 0, depth))
}

/// Read the content of a file inside a library.
#[tauri::command]
pub fn read_note(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist.".to_string());
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Extract headings from a note file.
#[tauri::command]
pub fn get_note_headings(app: tauri::AppHandle, file_path: String) -> Result<Vec<String>, String> {
    validate_path_in_any_library(&app, &file_path)?;
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

/// Write content to a markdown file inside a library.
#[tauri::command]
pub fn write_note(app: tauri::AppHandle, file_path: String, content: String) -> Result<(), String> {
    validate_path_in_any_library(&app, &file_path)?;
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
pub struct LibraryStats {
    pub library_id: String,
    pub name: String,
    pub path: String,
    pub star_count: u32,
    pub folder_count: u32,
    pub recent_stars: Vec<StarInfo>,
    #[serde(default)]
    pub is_universe_notes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarInfo {
    pub name: String,
    pub path: String,
    pub library_id: String,
    pub library_name: String,
    pub modified: u64,
    pub preview: String,
}

/// Get stats for all libraries (own + child universe) — star counts, folder counts, recent stars.
#[tauri::command]
pub fn get_all_library_stats(app: tauri::AppHandle) -> Vec<LibraryStats> {
    let libraries = load_all_libraries(&app);
    libraries.iter().map(|v| {
        let (star_count, folder_count) = count_contents(Path::new(&v.path));
        let recent_stars = get_recent_notes(Path::new(&v.path), &v.id, &v.name, 10);
        LibraryStats {
            library_id: v.id.clone(),
            name: v.name.clone(),
            path: v.path.clone(),
            star_count,
            folder_count,
            recent_stars,
            is_universe_notes: v.is_universe_notes,
        }
    }).collect()
}

/// Search across all libraries for notes matching a query.
#[tauri::command]
pub fn search_stars(app: tauri::AppHandle, query: String) -> Vec<StarInfo> {
    let libraries = load_all_libraries(&app);
    let mut results = Vec::new();

    // Parse search operators: file:, tag:, path:
    let mut file_filter: Option<String> = None;
    let mut tag_filter: Option<String> = None;
    let mut path_filter: Option<String> = None;
    let mut text_query = String::new();

    for part in query.split_whitespace() {
        let lower = part.to_lowercase();
        if let Some(val) = lower.strip_prefix("file:") {
            file_filter = Some(val.to_string());
        } else if let Some(val) = lower.strip_prefix("tag:") {
            tag_filter = Some(val.trim_start_matches('#').to_string());
        } else if let Some(val) = lower.strip_prefix("path:") {
            path_filter = Some(val.to_string());
        } else {
            if !text_query.is_empty() { text_query.push(' '); }
            text_query.push_str(&lower);
        }
    }

    for lib in &libraries {
        search_notes_recursive(
            Path::new(&lib.path),
            &lib.id,
            &lib.name,
            &text_query,
            &mut results,
            0,
        );
    }

    // Apply operator filters
    if let Some(ref ff) = file_filter {
        results.retain(|r| r.name.to_lowercase().contains(ff));
    }
    if let Some(ref pf) = path_filter {
        results.retain(|r| r.path.to_lowercase().replace('\\', "/").contains(pf));
    }
    if let Some(ref tf) = tag_filter {
        results.retain(|r| {
            // Read file content and check for tag
            let content = fs::read_to_string(&r.path).unwrap_or_default().to_lowercase();
            content.contains(&format!("#{}", tf))
                || content.contains(&format!("- {}", tf)) // YAML tags list
        });
    }

    // Sort by relevance (name match first, then content match)
    let query_lower = text_query.clone();
    results.sort_by(|a, b| {
        let a_name_match = a.name.to_lowercase().contains(&query_lower);
        let b_name_match = b.name.to_lowercase().contains(&query_lower);
        b_name_match.cmp(&a_name_match).then(b.modified.cmp(&a.modified))
    });

    results.truncate(200); // Limit results
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

fn get_recent_notes(dir: &Path, library_id: &str, library_name: &str, limit: usize) -> Vec<StarInfo> {
    let mut notes = Vec::new();
    collect_notes_recursive(dir, library_id, library_name, &mut notes, 0);
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

fn collect_notes_recursive(dir: &Path, library_id: &str, library_name: &str, notes: &mut Vec<StarInfo>, depth: u32) {
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
            collect_notes_recursive(&path, library_id, library_name, notes, depth + 1);
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
                library_id: library_id.to_string(),
                library_name: library_name.to_string(),
                modified,
                preview,
            });
        }
    }
}

fn search_notes_recursive(dir: &Path, library_id: &str, library_name: &str, query: &str, results: &mut Vec<StarInfo>, depth: u32) {
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
            search_notes_recursive(&path, library_id, library_name, query, results, depth + 1);
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
                    library_id: library_id.to_string(),
                    library_name: library_name.to_string(),
                    modified,
                    preview,
                });
            }
        }
    }
}

/// Create a new markdown note inside a library folder.
/// `initial_frontmatter` is optional YAML content (without delimiters) to insert between `---` markers.
#[tauri::command]
pub fn create_note(app: tauri::AppHandle, folder_path: String, file_name: String, initial_frontmatter: Option<String>) -> Result<String, String> {
    let safe_name = sanitize_name(&file_name)?;
    validate_path_in_any_library(&app, &folder_path)?;
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

/// Search notes by property key/value across all libraries.
#[tauri::command]
pub fn search_by_property(app: tauri::AppHandle, key: String, value: String) -> Vec<StarInfo> {
    let libraries = load_all_libraries(&app);
    let key_lower = key.to_lowercase();
    let value_lower = value.to_lowercase();
    let mut results = Vec::new();

    for lib in &libraries {
        search_property_recursive(
            Path::new(&lib.path),
            &lib.id,
            &lib.name,
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

fn search_property_recursive(dir: &Path, library_id: &str, library_name: &str, key: &str, value: &str, results: &mut Vec<StarInfo>, depth: u32) {
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
            search_property_recursive(&path, library_id, library_name, key, value, results, depth + 1);
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
                    library_id: library_id.to_string(),
                    library_name: library_name.to_string(),
                    modified,
                    preview: safe_truncate(&match_preview, 120),
                });
            }
        }
    }
}

/// Create a new folder inside a library.
#[tauri::command]
pub fn create_folder(app: tauri::AppHandle, parent_path: String, folder_name: String) -> Result<String, String> {
    let safe_name = sanitize_name(&folder_name)?;
    validate_path_in_any_library(&app, &parent_path)?;
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
    validate_path_in_any_library(&app, &old_path)?;
    validate_path_in_any_library(&app, &new_path)?;
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

/// Move a file or folder to a different directory within any registered library.
#[tauri::command]
pub fn move_item(app: tauri::AppHandle, source_path: String, target_folder: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &source_path)?;
    validate_path_in_any_library(&app, &target_folder)?;
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("Source item does not exist.".to_string());
    }
    let target_dir = Path::new(&target_folder);
    if !target_dir.is_dir() {
        return Err("Target folder does not exist.".to_string());
    }
    let file_name = source.file_name()
        .ok_or("Cannot determine file name.")?;
    let dest = target_dir.join(file_name);
    if dest.exists() {
        return Err("An item with this name already exists in the target folder.".to_string());
    }
    fs::rename(source, &dest)
        .map_err(|e| format!("Failed to move: {}", e))?;
    Ok(dest.to_string_lossy().to_string())
}

/// Delete a file or folder (permanent delete).
#[tauri::command]
pub fn delete_item(app: tauri::AppHandle, path: String, permanent: Option<bool>) -> Result<(), String> {
    validate_path_in_any_library(&app, &path)?;
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

/// Resolve a wikilink target to an actual file path within a library.
#[tauri::command]
pub fn resolve_wikilink(app: tauri::AppHandle, library_path: String, target: String) -> Result<Option<String>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let library_dir = Path::new(&library_path);
    if !library_dir.exists() {
        return Err("Library path does not exist.".to_string());
    }

    let target_lower = target.to_lowercase();
    let mut matches: Vec<PathBuf> = Vec::new();
    find_note_by_name_or_alias(library_dir, &target_lower, &mut matches, 0);

    if matches.is_empty() {
        return Ok(None);
    }

    // Prefer shortest path (closest to library root)
    matches.sort_by_key(|p| p.to_string_lossy().len());
    Ok(Some(matches[0].to_string_lossy().to_string()))
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedLink {
    pub path: String,
    pub library_name: String,
    pub library_path: String,
    pub fragment: Option<String>,
}

/// Resolve a wikilink across all libraries. Searches current library first, then others.
/// Supports `library_name:note` syntax to target a specific library.
/// Supports `note#heading` and `note#^block-id` — fragment is stripped before resolution and returned separately.
#[tauri::command]
pub fn resolve_wikilink_cross_library(
    libraries: Vec<(String, String, String)>, // (library_id, library_name, library_path)
    current_library_path: String,
    target: String,
) -> Result<Option<ResolvedLink>, String> {
    // Strip fragment (#heading or #^block-id)
    let (base_target, fragment) = if let Some(hash_pos) = target.find('#') {
        (target[..hash_pos].to_string(), Some(target[hash_pos + 1..].to_string()))
    } else {
        (target.clone(), None)
    };

    // Check for library:note syntax
    if let Some(colon_pos) = base_target.find(':') {
        let library_prefix = base_target[..colon_pos].trim().to_lowercase();
        let note_target = base_target[colon_pos + 1..].trim().to_lowercase();
        if !note_target.is_empty() {
            for (_id, name, path) in &libraries {
                if name.to_lowercase() == library_prefix {
                    let library_dir = Path::new(path);
                    if !library_dir.exists() { continue; }
                    let mut matches: Vec<PathBuf> = Vec::new();
                    find_note_by_name_or_alias(library_dir, &note_target, &mut matches, 0);
                    if !matches.is_empty() {
                        matches.sort_by_key(|p| p.to_string_lossy().len());
                        return Ok(Some(ResolvedLink {
                            path: matches[0].to_string_lossy().to_string(),
                            library_name: name.clone(),
                            library_path: path.clone(),
                            fragment,
                        }));
                    }
                    return Ok(None);
                }
            }
        }
    }

    let target_lower = base_target.to_lowercase();

    // Search current library first
    let current_dir = Path::new(&current_library_path);
    if current_dir.exists() {
        let mut matches: Vec<PathBuf> = Vec::new();
        find_note_by_name_or_alias(current_dir, &target_lower, &mut matches, 0);
        if !matches.is_empty() {
            matches.sort_by_key(|p| p.to_string_lossy().len());
            let library_name = libraries.iter()
                .find(|(_, _, p)| p == &current_library_path)
                .map(|(_, n, _)| n.clone())
                .unwrap_or_default();
            return Ok(Some(ResolvedLink {
                path: matches[0].to_string_lossy().to_string(),
                library_name,
                library_path: current_library_path,
                fragment,
            }));
        }
    }

    // Search other libraries
    for (_id, name, path) in &libraries {
        if *path == current_library_path { continue; }
        let library_dir = Path::new(path);
        if !library_dir.exists() { continue; }
        let mut matches: Vec<PathBuf> = Vec::new();
        find_note_by_name_or_alias(library_dir, &target_lower, &mut matches, 0);
        if !matches.is_empty() {
            matches.sort_by_key(|p| p.to_string_lossy().len());
            return Ok(Some(ResolvedLink {
                path: matches[0].to_string_lossy().to_string(),
                library_name: name.clone(),
                library_path: path.clone(),
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

/// Read Obsidian's appearance.json for a library.
#[tauri::command]
pub fn read_library_appearance(app: tauri::AppHandle, library_path: String) -> Result<serde_json::Value, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let path = Path::new(&library_path).join(".obsidian").join("appearance.json");
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
        .set_title("Select Library Folder")
        .pick_folder();

    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

/// Pick a parent folder, create a named subfolder, and register it as a library.
#[tauri::command]
pub async fn create_new_library(app: tauri::AppHandle, name: String) -> Result<Option<LibraryInfo>, String> {
    // 1. Pick parent location
    let parent = rfd::FileDialog::new()
        .set_title("Choose location for new library")
        .pick_folder();
    let parent_path = match parent {
        Some(p) => p,
        None => return Ok(None), // user cancelled
    };

    // 2. Create the library folder
    let library_dir = parent_path.join(&name);
    if library_dir.exists() {
        return Err(format!("Folder '{}' already exists at that location", name));
    }
    fs::create_dir_all(&library_dir)
        .map_err(|e| format!("Failed to create library folder: {}", e))?;

    // 3. Register it as a library
    let path_str = library_dir.to_string_lossy().to_string();
    let library = add_library(app, path_str)?;
    Ok(Some(library))
}

/// Extract the `status` value from a markdown file's YAML frontmatter.
/// Reads only the first 512 bytes for performance.
fn extract_frontmatter_status(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut file = fs::File::open(path).ok()?;
    let mut buf = [0u8; 512];
    let n = file.read(&mut buf).ok()?;
    let text = std::str::from_utf8(&buf[..n]).ok()?;
    let mut lines = text.lines();
    // Must start with ---
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "..." {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("status:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if matches!(val.as_str(), "seedling" | "growing" | "evergreen") {
                return Some(val);
            }
        }
    }
    None
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

        let status = if !is_dir && extension.as_deref() == Some("md") {
            extract_frontmatter_status(&path)
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
            status,
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
    pub library_name: String,
    pub link_type: Option<String>,
}

/// Scan all notes in a library and extract wikilinks from each.
#[tauri::command]
pub fn scan_library_links(app: tauri::AppHandle, library_path: String, library_name: String) -> Result<Vec<NoteLink>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut links = Vec::new();
    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").unwrap();
    scan_links_recursive(Path::new(&library_path), &re, &mut links, &library_name);
    Ok(links)
}

fn scan_links_recursive(dir: &Path, re: &regex::Regex, links: &mut Vec<NoteLink>, library_name: &str) {
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
            scan_links_recursive(&path, re, links, library_name);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let source_name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                for cap in re.captures_iter(&content) {
                    let target = cap[1].trim().to_string();
                    // Extract link type from alias:
                    //   [[note|causes]]          → direct type name
                    //   [[note|type:causes]]      → legacy explicit prefix (backward compat)
                    //   [[note|Display Text]]     → display alias, not a type → None
                    const KNOWN_LINK_TYPES: &[&str] = &[
                        "supports", "contradicts", "causes", "exemplifies",
                        "generalizes", "derives-from", "part-of", "associative",
                    ];
                    let link_type = cap.get(2).and_then(|alias| {
                        let alias_str = alias.as_str().trim();
                        let lower = alias_str.to_lowercase();
                        if lower.starts_with("type:") {
                            Some(lower[5..].trim().to_string())
                        } else if KNOWN_LINK_TYPES.contains(&lower.as_str()) {
                            Some(lower)
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
                        library_name: library_name.to_string(),
                        link_type,
                    });
                }
            }
        }
    }
}

/// Scan for unlinked mentions of a note name across all libraries.
/// Returns notes that mention the name as plain text but don't have a [[wikilink]] to it.
#[tauri::command]
pub fn scan_unlinked_mentions(
    app: tauri::AppHandle,
    note_name: String,
    note_path: String,
    library_paths: Vec<(String, String)>, // (library_name, library_path)
) -> Result<Vec<NoteLink>, String> {
    let registered = load_all_libraries(&app);
    let wikilink_pattern = format!("[[{}]]", &note_name);
    let wikilink_pattern_lower = wikilink_pattern.to_lowercase();
    let word_re = match regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&note_name))) {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };

    let mut results = Vec::new();
    let cap = 50usize;

    for (library_name, library_path) in &library_paths {
        if !registered.iter().any(|v| &v.path == library_path) { continue; }
        scan_unlinked_recursive(
            Path::new(library_path),
            &note_path,
            &word_re,
            &wikilink_pattern_lower,
            library_name,
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
    library_name: &str,
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
            scan_unlinked_recursive(&path, note_path, word_re, wikilink_lower, library_name, results, cap, depth + 1);
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
                        library_name: library_name.to_string(),
                        link_type: None,
                    });
                }
            }
        }
    }
}

/// Scan all tags across a library.
#[tauri::command]
pub fn scan_library_tags(app: tauri::AppHandle, library_path: String) -> Result<std::collections::HashMap<String, u32>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut tags: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").unwrap();
    scan_tags_recursive(Path::new(&library_path), &re, &mut tags);
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

/// Return notes that contain a given tag (inline `#tag` or YAML frontmatter).
#[tauri::command]
pub fn notes_by_tag(app: tauri::AppHandle, library_path: String, tag: String) -> Result<Vec<StarInfo>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let lib = libraries.iter().find(|v| v.path == library_path).unwrap();
    let re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").unwrap();
    let mut results = Vec::new();
    collect_notes_with_tag(Path::new(&library_path), &lib.id, &lib.name, &re, &tag, &mut results);
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

fn collect_notes_with_tag(dir: &Path, lib_id: &str, lib_name: &str, re: &regex::Regex, tag: &str, results: &mut Vec<StarInfo>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            collect_notes_with_tag(&path, lib_id, lib_name, re, tag, results);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let has_tag = re.captures_iter(&content).any(|cap| cap[1].eq_ignore_ascii_case(tag));
                if has_tag {
                    let modified = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                        .unwrap_or(0);
                    let preview = safe_truncate(content.lines()
                        .find(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
                        .unwrap_or(""), 80);
                    results.push(StarInfo {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        library_id: lib_id.to_string(),
                        library_name: lib_name.to_string(),
                        modified,
                        preview,
                    });
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

/// Normalize Arabic text: unify hamza forms, remove tashkeel, normalize ta marbuta
fn normalize_arabic(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    for ch in word.chars() {
        match ch {
            // Remove tashkeel (diacritics)
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}' => continue,
            // Normalize hamza variants → ا
            'أ' | 'إ' | 'آ' | 'ٱ' => result.push('ا'),
            // Normalize ta marbuta → ه
            'ة' => result.push('ه'),
            // Normalize alef maqsura → ي
            'ى' => result.push('ي'),
            // Tatweel (kashida) — remove
            '\u{0640}' => continue,
            _ => result.push(ch),
        }
    }
    result
}

/// Remove common Arabic prefixes: و ف بـ كـ لـ الـ وال فال بال كال لل
fn strip_arabic_prefix(word: &str) -> &str {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 3 { return word; } // don't strip very short words

    // Three-char prefixes: وال فال بال كال
    if len > 4 {
        if (chars[0] == 'و' || chars[0] == 'ف' || chars[0] == 'ب' || chars[0] == 'ك')
            && chars[1] == 'ا' && chars[2] == 'ل' {
            let rest: String = chars[3..].iter().collect();
            let byte_offset = word.len() - rest.len();
            return &word[byte_offset..];
        }
    }

    // Two-char prefixes: ال لل
    if len > 3 {
        if (chars[0] == 'ا' && chars[1] == 'ل') || (chars[0] == 'ل' && chars[1] == 'ل') {
            let rest: String = chars[2..].iter().collect();
            let byte_offset = word.len() - rest.len();
            return &word[byte_offset..];
        }
    }

    // Single-char prefixes: و ف بـ كـ لـ
    if len > 3 {
        if chars[0] == 'و' || chars[0] == 'ف' || chars[0] == 'ب' || chars[0] == 'ك' || chars[0] == 'ل' {
            let rest: String = chars[1..].iter().collect();
            let byte_offset = word.len() - rest.len();
            return &word[byte_offset..];
        }
    }

    word
}

/// Remove common Hebrew prefixes: ב ל מ ה ו כ ש
fn strip_hebrew_prefix(word: &str) -> &str {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 3 { return word; }

    // Two-char prefix: וה (and the)
    if len > 3 && chars[0] == 'ו' && (chars[1] == 'ה' || chars[1] == 'ב' || chars[1] == 'ל' || chars[1] == 'מ' || chars[1] == 'כ') {
        let rest: String = chars[2..].iter().collect();
        let byte_offset = word.len() - rest.len();
        return &word[byte_offset..];
    }

    // Single-char prefixes
    if len > 3 {
        match chars[0] {
            'ב' | 'ל' | 'מ' | 'ה' | 'ו' | 'כ' | 'ש' => {
                let rest: String = chars[1..].iter().collect();
                let byte_offset = word.len() - rest.len();
                return &word[byte_offset..];
            }
            _ => {}
        }
    }

    word
}

/// Basic Arabic stemming: remove common suffixes
fn stem_arabic(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 4 { return word.to_string(); }

    // Remove plural/dual/feminine suffixes
    // ات ون ين ان ية ها هم كم نا تم
    if len > 4 {
        let last2: String = chars[len-2..].iter().collect();
        match last2.as_str() {
            "ات" | "ون" | "ين" | "ان" | "يه" | "ها" | "هم" | "كم" | "نا" | "تم" | "يا" | "وا" => {
                return chars[..len-2].iter().collect();
            }
            _ => {}
        }
    }

    // Remove single suffix: ة ه ي
    if len > 3 {
        match chars[len-1] {
            'ه' | 'ي' => {
                return chars[..len-1].iter().collect();
            }
            _ => {}
        }
    }

    word.to_string()
}

/// Detect if a word is Arabic script
fn is_arabic(word: &str) -> bool {
    word.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c) || ('\u{0750}'..='\u{077F}').contains(&c) || ('\u{FB50}'..='\u{FDFF}').contains(&c) || ('\u{FE70}'..='\u{FEFF}').contains(&c))
}

/// Detect if a word is Hebrew script
fn is_hebrew(word: &str) -> bool {
    word.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c) || ('\u{FB1D}'..='\u{FB4F}').contains(&c))
}

fn build_stopwords() -> std::collections::HashSet<String> {
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
        // Arabic (including normalized forms)
        "في","من","على","الى","هذا","هذه","التي","الذي","عن","مع","هو","هي","كان","كانت",
        "ذلك","تلك","ما","لا","ان","ان","لم","لن","قد","ثم","او","حتى","بين","عند","كل",
        "بعد","قبل","بعض","نحو","اي","انه","انها","لقد","فقط","هنا","هناك","منذ","حيث",
        "كما","اذا","عبر","ضد","خلال","حول","فيه","فيها","عليه","عليها","منه","منها",
        "به","بها","له","لها","لهم","هولاء","اولئك","وهو","وهي","ولا","ولم","الا",
        "اما","سوف","لكن","ليس","ليست","كذلك","ايضا","مثل","غير","دون","ضمن",
        "ذات","ذو","ذي","اللذين","اللتين","اللواتي","الذين","عليهم","لديه","لديها",
        "وقد","ولقد","والتي","والذي","ومن","وعلى","وفي","ومع","وعن","والى",
        // Hebrew
        "של","הוא","היא","את","זה","זו","אני","אנחנו","הם","הן","אתה","את","אתם","אתן",
        "יש","אין","לא","כי","גם","או","עם","על","אל","מן","אם","כל","עוד","רק","אבל",
        "היה","היתה","היו","יהיה","כמו","אחר","אחרי","לפני","בין","אצל","עד","מאד","כבר",
        "אז","שם","פה","למה","איך","מה","מי","איפה","מתי","כאשר","אשר","שלו","שלה","שלהם",
        // Persian/Farsi
        "از","به","در","با","که","این","آن","را","است","بر","تا","هم","و","یا","اما",
        "برای","اگر","هر","یک","شد","بود","خود","ما","شما","او","آنها","ایشان","هیچ",
        "چون","پس","زیرا","ولی","نه","بلکه","همه","بعد","قبل","بین","روی","زیر","کنار",
        // Urdu
        "کا","کی","کے","میں","ہے","کو","اور","سے","پر","نے","یہ","وہ","ایک","ہیں","تھا",
        "اس","جو","بھی","نہیں","کر","ہو","تو","ہی","یا","اپنے","سب","کچھ","لیے","ساتھ",
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
        "или","если","них","нас","вас","ему","ней","ним","себя","есть","очень","еще",
        // Portuguese
        "o","a","os","as","de","da","do","em","no","na","um","uma","que","para","com","por",
        "se","mais","não","como","mas","foi","ao","dos","das","nos","nas","seu","sua","esse",
        // Turkish
        "bir","bu","ve","da","de","ile","için","olan","gibi","daha","çok","ama","ya","hem",
        "ne","var","ben","sen","biz","siz","her","hiç","kadar","sonra","önce","arasında",
        // Hindi
        "का","के","की","में","है","को","और","से","पर","ने","यह","वह","एक","हैं","था",
        "इस","उस","कि","जो","भी","नहीं","कर","हो","तो","ही","या","अपने","सब","कुछ",
        // Japanese (particles and common function words)
        "の","に","は","を","た","が","で","て","と","し","れ","さ","ある","いる","も",
        "する","から","な","こと","として","い","や","れる","など","なっ","ない","この",
        "ため","その","あっ","よう","また","もの","という","あり","まで","られ","なる",
        // Korean (particles and common function words)
        "이","그","저","것","수","등","들","및","에","를","의","는","은","로","와","과",
        "도","가","한","할","하는","하고","하여","되","된","되는","있","없","않","위",
        // Chinese (common function words — particles, conjunctions, pronouns)
        "的","了","在","是","我","有","和","就","不","人","都","一","一个","上","也","很",
        "到","说","要","去","你","会","着","没有","看","好","自己","这","那","她","他",
        "它","我们","你们","他们","什么","怎么","哪","为什么","因为","所以","但是","而且",
    ];
    // Normalize Arabic words in stopwords list too
    words.iter().map(|w| {
        let s = w.to_string();
        if is_arabic(&s) { normalize_arabic(&s) } else { s }
    }).collect()
}

/// CE Phase 6: Scan all notes for `stage:` frontmatter property.
/// Returns a map of note_path → stage value (fleeting|literature|permanent|synthesis).
#[tauri::command]
pub fn scan_note_stages(app: tauri::AppHandle, library_path: String) -> Result<Vec<(String, String)>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut stages: Vec<(String, String)> = Vec::new();
    scan_stages_recursive(Path::new(&library_path), &mut stages);
    Ok(stages)
}

fn scan_stages_recursive(dir: &Path, stages: &mut Vec<(String, String)>) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            scan_stages_recursive(&path, stages);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.starts_with("---") {
                    if let Some(end) = content[3..].find("\n---") {
                        let yaml = &content[3..3 + end];
                        for line in yaml.lines() {
                            let trimmed = line.trim().to_lowercase();
                            if let Some(val) = trimmed.strip_prefix("stage:") {
                                let stage = val.trim().to_string();
                                if !stage.is_empty() {
                                    stages.push((path.to_string_lossy().to_string(), stage));
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Scan all notes in a library and build a word index.
#[tauri::command]
pub fn scan_library_index(app: tauri::AppHandle, library_path: String) -> Result<Vec<IndexEntry>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
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
        Path::new(&library_path), &md_strip, &stopwords, &mut index, &mut bigrams,
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
    stopwords: &std::collections::HashSet<String>,
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
                    let word_is_arabic = is_arabic(word);
                    let word_is_hebrew = is_hebrew(word);
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

                    // Phase 1: Normalize Arabic (hamza, tashkeel, ta marbuta)
                    let normalized = if word_is_arabic {
                        normalize_arabic(word)
                    } else {
                        word.to_string()
                    };

                    // Phase 3 & 4: Strip prefixes for Semitic languages
                    let stripped = if word_is_arabic {
                        strip_arabic_prefix(&normalized).to_string()
                    } else if word_is_hebrew {
                        strip_hebrew_prefix(&normalized).to_string()
                    } else {
                        normalized.clone()
                    };

                    // Phase 5: Basic Arabic stemming (suffix removal)
                    let stemmed = if word_is_arabic && stripped.chars().count() >= 3 {
                        stem_arabic(&stripped)
                    } else {
                        stripped.clone()
                    };

                    // Use stemmed form as index key, but keep original display form
                    let key = stemmed.to_lowercase();

                    // Skip stopwords (check both original normalized and stemmed forms)
                    let norm_lower = normalized.to_lowercase();
                    let is_stop = stopwords.contains(&key) || stopwords.contains(&norm_lower);

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

/// Collect all note names in a library (for autocomplete).
#[tauri::command]
pub fn collect_library_notes(app: tauri::AppHandle, library_path: String) -> Result<Vec<serde_json::Value>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut notes = Vec::new();
    collect_notes_names_recursive(Path::new(&library_path), &mut notes);
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

/// Collect all notes with rich metadata (name, path, modified, size, preview, tags, folder).
/// Used by the Notebook Navigator for fast file listing without N+1 calls.
#[tauri::command]
pub fn collect_library_notes_with_metadata(app: tauri::AppHandle, library_path: String) -> Result<Vec<serde_json::Value>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut notes = Vec::new();
    let tag_re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z\p{Arabic}][\w\p{Arabic}/\-]*)").unwrap();
    let lib_path = Path::new(&library_path);
    collect_notes_meta_recursive(lib_path, lib_path, &tag_re, &mut notes);
    Ok(notes)
}

fn collect_notes_meta_recursive(
    dir: &Path,
    lib_root: &Path,
    tag_re: &regex::Regex,
    notes: &mut Vec<serde_json::Value>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            collect_notes_meta_recursive(&path, lib_root, tag_re, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let note_name = path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // File metadata
            let meta = fs::metadata(&path).ok();
            let modified = meta.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

            // Relative folder path
            let folder = path.parent()
                .and_then(|p| p.strip_prefix(lib_root).ok())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            // Read content for preview and tags
            let content = fs::read_to_string(&path).unwrap_or_default();

            // Strip frontmatter for preview
            let body = if content.starts_with("---") {
                if let Some(end) = content[3..].find("---") {
                    content[3 + end + 3..].trim_start().to_string()
                } else {
                    content.clone()
                }
            } else {
                content.clone()
            };
            let preview: String = body.chars().take(200).collect();

            // Extract tags (inline)
            let mut tags: Vec<String> = Vec::new();
            for cap in tag_re.captures_iter(&content) {
                let tag = cap[1].to_string();
                if !tags.contains(&tag) {
                    tags.push(tag);
                }
            }

            // Extract YAML tags
            if content.starts_with("---") {
                if let Some(end) = content[3..].find("---") {
                    let yaml = &content[3..3 + end];
                    let mut in_tags = false;
                    for line in yaml.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("tags:") {
                            in_tags = true;
                            // Inline tags: tags: [a, b, c]
                            if let Some(bracket) = trimmed.strip_prefix("tags:").map(|s| s.trim()) {
                                if bracket.starts_with('[') && bracket.ends_with(']') {
                                    for t in bracket[1..bracket.len()-1].split(',') {
                                        let t = t.trim().trim_matches('"').trim_matches('\'').to_string();
                                        if !t.is_empty() && !tags.contains(&t) {
                                            tags.push(t);
                                        }
                                    }
                                    in_tags = false;
                                }
                            }
                        } else if in_tags && trimmed.starts_with("- ") {
                            let t = trimmed.trim_start_matches("- ").trim().trim_matches('"').trim_matches('\'').to_string();
                            if !t.is_empty() && !tags.contains(&t) {
                                tags.push(t);
                            }
                        } else if in_tags && !trimmed.is_empty() && !trimmed.starts_with('-') {
                            in_tags = false;
                        }
                    }
                }
            }

            notes.push(serde_json::json!({
                "name": note_name,
                "path": path.to_string_lossy().to_string(),
                "modified": modified,
                "size": size,
                "preview": preview,
                "tags": tags,
                "folder": folder,
            }));
        }
    }
}

/// Get daily note path for today.
#[tauri::command]
pub fn get_daily_note_path(app: tauri::AppHandle, library_path: String, format: String, folder: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &library_path)?;
    if !folder.is_empty() {
        if folder.contains("..") || folder.contains('\\') || folder.starts_with('/') {
            return Err("Folder name contains invalid characters.".to_string());
        }
    }
    let now = chrono::Local::now();
    let filename = now.format(&format).to_string();
    let daily_folder = if folder.is_empty() {
        Path::new(&library_path).to_path_buf()
    } else {
        Path::new(&library_path).join(&folder)
    };
    // Validate the resolved path is still within the library
    validate_path_in_library(&daily_folder.to_string_lossy(), &library_path)?;
    fs::create_dir_all(&daily_folder).map_err(|e| e.to_string())?;
    let file_path = daily_folder.join(format!("{}.md", filename));

    // Create the file if it doesn't exist
    if !file_path.exists() {
        let content = format!("---\ndate: {}\n---\n", now.format("%Y-%m-%d"));
        fs::write(&file_path, content).map_err(|e| e.to_string())?;
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// Quick capture: create a timestamped note in the inbox folder.
#[tauri::command]
pub fn quick_capture(app: tauri::AppHandle, library_path: String, inbox_folder: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &library_path)?;
    if inbox_folder.contains("..") || inbox_folder.contains('\\') || inbox_folder.starts_with('/') {
        return Err("Inbox folder name contains invalid characters.".to_string());
    }
    let inbox_dir = if inbox_folder.is_empty() {
        Path::new(&library_path).to_path_buf()
    } else {
        Path::new(&library_path).join(&inbox_folder)
    };
    validate_path_in_library(&inbox_dir.to_string_lossy(), &library_path)?;
    fs::create_dir_all(&inbox_dir).map_err(|e| e.to_string())?;

    let now = chrono::Local::now();
    let base_name = now.format("%Y-%m-%d %H-%M").to_string();

    // Deduplicate filename
    let mut file_path = inbox_dir.join(format!("{}.md", base_name));
    if file_path.exists() {
        for i in 1..=100 {
            file_path = inbox_dir.join(format!("{} {}.md", base_name, i));
            if !file_path.exists() {
                break;
            }
        }
    }

    let content = format!("---\ncreated: {}\n---\n\n", now.format("%Y-%m-%d"));
    fs::write(&file_path, &content).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Update all links in a library when a note is renamed.
#[tauri::command]
pub fn update_links_on_rename(app: tauri::AppHandle, library_path: String, old_name: String, new_name: String) -> Result<u32, String> {
    validate_path_in_any_library(&app, &library_path)?;
    let mut count = 0u32;
    update_links_recursive(Path::new(&library_path), &old_name, &new_name, &mut count);
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
    validate_path_in_any_library(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(safe_truncate(&content, max_chars))
}

/// Save a base64-encoded image from clipboard to the library's attachments folder.
/// Returns the relative path suitable for embedding as `![[filename]]`.
#[tauri::command]
pub fn save_clipboard_image(app: tauri::AppHandle, library_path: String, image_data: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &library_path)?;
    // Create attachments folder if it doesn't exist
    let attachments_dir = Path::new(&library_path).join("attachments");
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

/// Resolve an image embed filename to a base64 data URL.
/// Searches: note's folder → library/attachments/ → library root.
/// Returns `data:image/...;base64,...` or an empty string if not found.
#[tauri::command]
pub fn resolve_embed_image(
    library_path: String,
    note_path: String,
    filename: String,
) -> String {
    let note_dir = Path::new(&note_path).parent().map(|p| p.to_path_buf());

    // Candidate paths in priority order
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(ref nd) = note_dir {
        candidates.push(nd.join(&filename));
    }
    if !library_path.is_empty() {
        candidates.push(Path::new(&library_path).join("attachments").join(&filename));
        candidates.push(Path::new(&library_path).join("images").join(&filename));
        candidates.push(Path::new(&library_path).join("assets").join(&filename));
        candidates.push(Path::new(&library_path).join(&filename));
    }

    for cand in &candidates {
        if cand.is_file() {
            if let Ok(bytes) = fs::read(cand) {
                let ext = cand.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                let mime = match ext.as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "svg" => "image/svg+xml",
                    "webp" => "image/webp",
                    "bmp" => "image/bmp",
                    "ico" => "image/x-icon",
                    "avif" => "image/avif",
                    _ => "image/png",
                };
                return format!("data:{};base64,{}", mime, base64_encode(&bytes));
            }
        }
    }
    String::new()
}

/// Simple base64 encoder
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(TABLE[((n >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(TABLE[(n & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
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
    validate_path_in_any_library(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(content)
}

/// Move item to system trash (or ".trash" folder inside library)
#[tauri::command]
pub fn move_to_trash(app: tauri::AppHandle, path: String, library_path: String) -> Result<(), String> {
    // Verify the file is within a registered library (not just any caller-supplied library_path)
    validate_path_in_any_library(&app, &path)?;
    validate_path_in_library(&path, &library_path)?;
    let trash_dir = Path::new(&library_path).join(".trash");
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

// ─── File Metadata ───

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileMetadata {
    pub created: u64,
    pub modified: u64,
}

/// Get file creation and modification timestamps (Unix seconds).
#[tauri::command]
pub fn get_file_metadata(file_path: String) -> Result<FileMetadata, String> {
    let meta = fs::metadata(&file_path)
        .map_err(|e| format!("Failed to read metadata for {}: {}", file_path, e))?;

    let created = meta.created()
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    let modified = meta.modified()
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    Ok(FileMetadata { created, modified })
}

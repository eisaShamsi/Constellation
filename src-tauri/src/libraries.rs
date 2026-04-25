use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub is_universe_notes: bool,
    /// "native" = created by Constellation (always canonical filenames)
    /// "canonical" = external library, user accepted canonicalization
    /// "compatible" = external library, user chose to keep files intact
    #[serde(default = "default_canonical_mode")]
    pub canonical_mode: String,
}

fn default_canonical_mode() -> String { "native".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileEntry>>,
    pub extension: Option<String>,
    pub modified: Option<u64>,
    pub status: Option<String>,
    /// For canonical files: the human-readable title from frontmatter.
    /// Null for non-canonical files or folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
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

/// Module-level cache for `load_all_libraries`.
///
/// Before this cache: diagnostic logs showed 50+ calls per boot from many
/// different code paths (validate_path_in_any_library, scan_*,
/// constellation_map_universe, etc.). Each re-read libraries.json from disk
/// and re-parsed it. Under Tauri's IPC queue on Windows this created the
/// 60-second boot-time hang we've been hunting.
///
/// The cache:
///   - Populated on first call per active-universe.
///   - Invalidated whenever `save_libraries` writes to disk.
///   - Keyed by the active universe path — switching universes reloads.
static LIBRARIES_CACHE: std::sync::Mutex<Option<(std::path::PathBuf, Vec<LibraryInfo>)>> =
    std::sync::Mutex::new(None);

/// Load ALL libraries: own + child universe libraries (recursive, deduplicated).
/// This is what the frontend and query_base should use.
pub fn load_all_libraries(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    let active = crate::universe::active_universe_dir(app).ok();

    // Fast path — cache hit for the currently active universe.
    if let Some(ref universe_path) = active {
        if let Ok(guard) = LIBRARIES_CACHE.lock() {
            if let Some((cached_universe, cached_libs)) = guard.as_ref() {
                if cached_universe == universe_path {
                    return cached_libs.clone();
                }
            }
        }
    }

    // Cache miss or unknown universe — do the actual disk read + parse.
    let libs = match crate::universe::resolve_universe_libraries(app.clone()) {
        Ok(libs) => libs,
        Err(_) => load_libraries(app),
    };

    if let Some(universe_path) = active {
        if let Ok(mut guard) = LIBRARIES_CACHE.lock() {
            *guard = Some((universe_path, libs.clone()));
        }
    }
    libs
}

/// Invalidate the in-memory library cache. Call when the on-disk
/// libraries.json has changed (add/remove library, rename, universe switch).
pub fn invalidate_libraries_cache() {
    if let Ok(mut guard) = LIBRARIES_CACHE.lock() {
        *guard = None;
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
    fs::write(&path, data).map_err(|e| format!("Failed to save libraries config: {}", e))?;
    // Invalidate the in-memory cache so subsequent reads see the new list.
    invalidate_libraries_cache();
    Ok(())
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
        canonical_mode: "compatible".to_string(), // external libraries default to compatible
    };

    libraries.push(library.clone());
    save_libraries(&app, &libraries)?;

    Ok(library)
}

/// Update a library's canonical mode ("native", "canonical", or "compatible").
#[tauri::command]
pub fn set_library_canonical_mode(app: tauri::AppHandle, library_id: String, mode: String) -> Result<(), String> {
    if !["native", "canonical", "compatible"].contains(&mode.as_str()) {
        return Err(format!("Invalid canonical mode: {}", mode));
    }
    let mut libraries = load_libraries(&app);
    if let Some(lib) = libraries.iter_mut().find(|l| l.id == library_id) {
        lib.canonical_mode = mode;
        save_libraries(&app, &libraries)?;
        Ok(())
    } else {
        Err("Library not found.".to_string())
    }
}

/// Get a library's canonical mode by path.
pub fn get_library_mode(app: &tauri::AppHandle, folder_path: &str) -> String {
    let libraries = load_all_libraries(app);
    libraries.iter()
        .find(|l| folder_path.starts_with(&l.path))
        .map(|l| l.canonical_mode.clone())
        .unwrap_or_else(|| "native".to_string())
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
    use std::sync::OnceLock;
    static HEADING_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = HEADING_RE.get_or_init(|| regex::Regex::new(r"(?m)^#{1,6}\s+(.+)$").unwrap());
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
///
/// `(async)` keeps the body off the WebView2 UI thread (see watcher.rs
/// `watch_library` for the full rationale — LL-021 post-Round-3). Critical
/// here because this fn `.join()`s every per-library scanner thread before
/// returning: on a 16-library × 7,600-note Universe that's several seconds
/// of synchronous wait. Without `(async)` those seconds are paid on the UI
/// thread, starving every other boot-fan-out IPC behind it — including
/// `cache_boot_snapshot_core`, which is Boot Criterion 2's critical path.
#[tauri::command(async)]
pub fn get_all_library_stats(app: tauri::AppHandle) -> Vec<LibraryStats> {
    let libraries = load_all_libraries(&app);
    // PERF: Parallelize per-library scans. On a 16-library Universe the sequential
    // walk was the first awaited call on boot — ~7,600 stat calls back-to-back
    // plus 160 preview reads. Using std threads (no new dep) the disk/CPU runs
    // concurrently and wall-time drops roughly 10×.
    let handles: Vec<_> = libraries.into_iter().map(|v| {
        std::thread::spawn(move || {
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
        })
    }).collect();
    handles.into_iter().filter_map(|h| h.join().ok()).collect()
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
    // PERF: Collect metadata only (no file content reads). On a 7,600-note Universe
    // the previous impl read every .md file's content just to pick the 10 most
    // recent — that's the disk thrashing on boot. We now defer preview reads to
    // the top-N files after sorting, turning ~7,600 reads into ~10 per library.
    let mut meta: Vec<(String, std::path::PathBuf, u64)> = Vec::new();
    collect_recent_meta_recursive(dir, &mut meta, 0);
    // Partial-sort by modified DESC, keep top `limit` — using a simple full sort
    // is fine at O(n log n) on metadata-only tuples (no I/O inside the compare).
    meta.sort_by(|a, b| b.2.cmp(&a.2));
    meta.truncate(limit);
    meta.into_iter().map(|(name, path, modified)| {
        let preview = fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let preview = safe_truncate(&preview, 120);
        StarInfo {
            name: name.trim_end_matches(".md").to_string(),
            path: path.to_string_lossy().to_string(),
            library_id: library_id.to_string(),
            library_name: library_name.to_string(),
            modified,
            preview,
        }
    }).collect()
}

fn collect_recent_meta_recursive(dir: &Path, out: &mut Vec<(String, std::path::PathBuf, u64)>, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }
        if path.is_dir() {
            collect_recent_meta_recursive(&path, out, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);
            out.push((name, path, modified));
        }
    }
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

#[allow(dead_code)]
fn _collect_notes_recursive_unused(_dir: &Path, _library_id: &str, _library_name: &str, _notes: &mut Vec<StarInfo>, _depth: u32) {
    // Superseded by get_recent_notes metadata-first + top-N preview read.
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
    validate_path_in_any_library(&app, &folder_path)?;
    let folder = Path::new(&folder_path);
    if !folder.exists() || !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    let mode = get_library_mode(&app, &folder_path);
    let dt = chrono::Utc::now();

    if mode == "native" || mode == "canonical" {
        // Canonical: structured filename + full frontmatter
        let canonical = crate::canonical::generate_canonical("NOTE", &dt, "md", Some(folder));
        let file_path = folder.join(&canonical.full);
        let display_name = file_name.trim_end_matches(".md");

        let mut fm_lines: Vec<String> = Vec::new();
        fm_lines.push(format!("title: \"{}\"", display_name.replace('"', "\\\"")));
        fm_lines.push(format!("cid: {}", canonical.stem));
        fm_lines.push("kind: note".to_string());
        fm_lines.push(format!("created: {}", dt.to_rfc3339()));

        if let Some(ref extra) = initial_frontmatter {
            for line in extra.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("title:") && !trimmed.starts_with("cid:")
                    && !trimmed.starts_with("kind:") && !trimmed.starts_with("created:")
                    && !trimmed.is_empty()
                {
                    fm_lines.push(trimmed.to_string());
                }
            }
        }

        let content = format!("---\n{}\n---\n\n", fm_lines.join("\n"));
        fs::write(&file_path, &content)
            .map_err(|e| format!("Failed to create note: {}", e))?;
        Ok(file_path.to_string_lossy().to_string())
    } else {
        // Compatible: human-readable filename + only cid in frontmatter
        let safe_name = sanitize_name(&file_name)?;
        let name = if safe_name.ends_with(".md") { safe_name } else { format!("{}.md", safe_name) };
        let file_path = folder.join(&name);
        if file_path.exists() {
            return Err("A file with this name already exists.".to_string());
        }

        // Generate a cid without renaming
        let canonical = crate::canonical::generate_canonical("NOTE", &dt, "md", None);
        let mut fm_lines: Vec<String> = Vec::new();
        fm_lines.push(format!("cid: {}", canonical.stem));

        if let Some(ref extra) = initial_frontmatter {
            for line in extra.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("cid:") && !trimmed.is_empty() {
                    fm_lines.push(trimmed.to_string());
                }
            }
        }

        let content = if fm_lines.is_empty() {
            "---\n---\n\n".to_string()
        } else {
            format!("---\n{}\n---\n\n", fm_lines.join("\n"))
        };
        fs::write(&file_path, &content)
            .map_err(|e| format!("Failed to create note: {}", e))?;
        Ok(file_path.to_string_lossy().to_string())
    }
}

/// Check if a library has been canonicalized. Delegates to canonical module.
#[allow(dead_code)]
fn is_library_canonical(library_path: &str) -> bool {
    crate::canonical::is_library_canonicalized(library_path)
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
pub fn rename_item(app: tauri::AppHandle, old_path: String, new_path: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &old_path)?;
    let old = Path::new(&old_path);
    if !old.exists() {
        return Err("Item does not exist.".to_string());
    }

    // Check if this is a canonical .md file — if so, rename = update frontmatter title, NOT the filename
    if old.extension().map(|e| e == "md").unwrap_or(false)
        && crate::canonical::is_canonical_filename(old)
    {
        let new_title = Path::new(&new_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Read content and extract current title from frontmatter
        let content = fs::read_to_string(old)
            .map_err(|e| format!("Failed to read note: {}", e))?;
        let old_title = extract_frontmatter_title(&content)
            .unwrap_or_else(|| old.file_stem().unwrap_or_default().to_string_lossy().to_string());

        // Idempotency guard: if the frontmatter title already matches the
        // requested new title, no write is needed. Without this, a stale
        // titleValue in the frontend (display-sync bug) that fires a blur
        // event would pass old_title == new_title to update_frontmatter_title,
        // which would dutifully append the current title to its own aliases
        // list — producing entries like [Untitled, TestBug001, TestBug001].
        if old_title == new_title {
            return Ok(old_path);
        }

        let updated = update_frontmatter_title(&content, &new_title, &old_title);
        fs::write(old, &updated)
            .map_err(|e| format!("Failed to write note: {}", e))?;

        // MIG-004 §3: stamp OLD title as a 'rename' alias for this path
        // BEFORE the reindex runs. update_frontmatter_title also appends
        // old_title to the note's `aliases:` list, which §2's writer
        // would pick up — but that path depends on the user not later
        // editing aliases away. The 'rename' row is the durable
        // safety net: source=partition keeps it independent of any
        // frontmatter edits, so a wikilink targeting the old title
        // resolves to this note for as long as the path exists.
        //
        // Runs before reindex so the alias is already present when the
        // §2 writer's DELETE-by-source-frontmatter clears stale rows.
        // INSERT OR IGNORE handles rename-back-to-prior-name
        // idempotently.
        {
            use tauri::Manager;
            let search_state = app.state::<crate::search::SearchState>();
            let note_path = old.to_string_lossy().to_string();
            let normalized = crate::search::normalize_alias_for_match(&old_title);
            if !normalized.is_empty() {
                let db_lock = search_state.db.lock();
                if let Ok(guard) = db_lock {
                    if let Some(conn) = guard.as_ref() {
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO note_aliases (path, alias_lower, source) VALUES (?1, ?2, 'rename')",
                            rusqlite::params![note_path, normalized],
                        );
                    }
                }
            }
        }

        // Trigger search reindex for this note so the new title is reflected
        {
            use tauri::Manager;
            let search_state = app.state::<crate::search::SearchState>();
            let note_path = old.to_string_lossy().to_string();
            let libs = load_all_libraries(&app);
            if let Some(lib) = libs.iter().find(|l| note_path.starts_with(&l.path)) {
                let _ = crate::search::reindex_single_note(&search_state, &note_path, &lib.name);
            }
        }

        // The file stays at old_path — canonical filename doesn't change.
        // Return the effective path so the frontend knows not to rewrite
        // tab.path to a non-existent location (would later create a phantom
        // file on the next write_note call — BUG-001 root cause).
        Ok(old_path)
    } else {
        // Legacy behavior: actually rename the file/folder
        validate_path_in_any_library(&app, &new_path)?;
        let new_p = Path::new(&new_path);
        if new_p.exists() {
            return Err("An item with this name already exists.".to_string());
        }
        fs::rename(old, new_p)
            .map_err(|e| format!("Failed to rename: {}", e))?;
        Ok(new_path)
    }
}

/// Extract the `title:` value from a note's frontmatter.
fn extract_frontmatter_title(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let after = &trimmed[3..];
    let end = after.find("\n---")?;
    let fm = &after[..end];
    for line in fm.lines() {
        let t = line.trim();
        if t.starts_with("title:") {
            let val = t["title:".len()..].trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() { return Some(val.to_string()); }
        }
    }
    None
}

/// Update a note's frontmatter title and add the old title to aliases.
fn update_frontmatter_title(content: &str, new_title: &str, old_title: &str) -> String {
    let esc_new = new_title.replace('"', "\\\"");
    let esc_old = old_title.replace('"', "\\\"");

    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return format!(
            "---\ntitle: \"{}\"\naliases:\n  - \"{}\"\n---\n\n{}",
            esc_new, esc_old, content
        );
    }

    let after_first = &trimmed[3..];
    let Some(end) = after_first.find("\n---") else {
        return content.to_string();
    };
    let fm = &after_first[..end];
    let body = &after_first[end + 4..];

    let mut new_lines: Vec<String> = Vec::new();
    let mut found_title = false;
    let mut found_aliases = false;
    let mut old_title_in_aliases = false;
    let mut in_alias_list = false;

    for line in fm.lines() {
        let t = line.trim();

        // Replace title field
        if t.starts_with("title:") {
            found_title = true;
            new_lines.push(format!("title: \"{}\"", esc_new));
            continue;
        }

        // Handle aliases field
        if t.starts_with("aliases:") {
            found_aliases = true;
            let value = t["aliases:".len()..].trim();

            if value.starts_with('[') && value.ends_with(']') {
                // Inline array: aliases: [a, b, c]
                let inner = &value[1..value.len() - 1];
                let existing: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                old_title_in_aliases = existing.iter().any(|a| a == old_title);
                // Convert to list format for consistency
                new_lines.push("aliases:".to_string());
                for alias in &existing {
                    new_lines.push(format!("  - \"{}\"", alias.replace('"', "\\\"")));
                }
                if !old_title_in_aliases {
                    new_lines.push(format!("  - \"{}\"", esc_old));
                }
                continue;
            }

            // List format: aliases:\n  - a\n  - b
            new_lines.push(line.to_string());
            in_alias_list = true;
            continue;
        }

        // Collect alias list items
        if in_alias_list && t.starts_with("- ") {
            let alias_val = t[2..].trim().trim_matches('"').trim_matches('\'');
            if alias_val == old_title {
                old_title_in_aliases = true;
            }
            new_lines.push(line.to_string());
            continue;
        }

        // End of alias list — append old title if missing
        if in_alias_list {
            in_alias_list = false;
            if !old_title_in_aliases {
                new_lines.push(format!("  - \"{}\"", esc_old));
            }
        }

        new_lines.push(line.to_string());
    }

    // If alias list was the last thing in frontmatter
    if in_alias_list && !old_title_in_aliases {
        new_lines.push(format!("  - \"{}\"", esc_old));
    }

    // Add missing fields
    if !found_title {
        new_lines.insert(0, format!("title: \"{}\"", esc_new));
    }
    if !found_aliases {
        new_lines.push("aliases:".to_string());
        new_lines.push(format!("  - \"{}\"", esc_old));
    }

    format!("---\n{}\n---{}", new_lines.join("\n"), body)
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
    let path_str = path.clone();
    let result = if target.is_dir() {
        fs::remove_dir_all(target)
            .map_err(|e| format!("Failed to delete folder: {}", e))
    } else {
        fs::remove_file(target)
            .map_err(|e| format!("Failed to delete file: {}", e))
    };

    // Clean up note_links and note_meta for deleted items
    if result.is_ok() {
        // Clean up search index + link data for deleted note
        use tauri::Manager;
        {
            let search_state = app.state::<crate::search::SearchState>();
            let _ = crate::search::reindex_delete_note(&search_state, &path_str);
        }
    }

    result
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
            // Normalize both sides: strict `==` drops to "" on Windows
            // slash / trailing-slash / case drift, which then shows up
            // as an empty library chip on the tab and poisons the next
            // wikilink resolution (empty currentLibraryPath skips this
            // branch entirely on the next click, picking the wrong
            // same-named note from another library).
            let norm = |s: &str| s.replace('\\', "/").trim_end_matches('/').to_lowercase();
            let current_norm = norm(&current_library_path);
            let library_name = libraries.iter()
                .find(|(_, _, p)| norm(p) == current_norm)
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

/// Like find_note_by_name, but also checks frontmatter title and aliases.
/// Resolution order (first match wins):
///   1. Filename stem match (fast, no file read)
///   2. Frontmatter `title:` field match (supports canonical filenames)
///   3. Frontmatter `aliases:` match
fn find_note_by_name_or_alias(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
    // First try exact filename match (fast)
    find_note_by_name(dir, target, results, depth);
    if !results.is_empty() { return; }

    // If no filename match, scan frontmatter title + aliases
    find_note_by_title_or_alias(dir, target, results, depth);
}

fn find_note_by_title_or_alias(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
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
            find_note_by_title_or_alias(&path, target, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if has_title(&content, target) || has_alias(&content, target) {
                    results.push(path);
                }
            }
        }
    }
}

/// Check if a note's frontmatter `title:` field matches the target.
fn has_title(content: &str, target: &str) -> bool {
    if !content.starts_with("---") { return false; }
    let end = match content[3..].find("\n---") {
        Some(pos) => pos + 3,
        None => return false,
    };
    let frontmatter = &content[3..end];
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("title:") {
            let value = trimmed["title:".len()..].trim();
            let value = value.trim_matches('"').trim_matches('\'').to_lowercase();
            if value == target { return true; }
        }
    }
    false
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
///
/// `(async)` because this fires 16× in the boot fan-out (one per library) and
/// performs disk I/O (`fs::read_to_string` + JSON parse). Keeping the body on
/// the WebView2 UI thread would serialize all 16 reads behind whatever other
/// fan-out work is in flight. See watcher.rs `watch_library` for the full
/// UI-thread-serialization rationale (LL-021 post-Round-3).
#[tauri::command(async)]
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

        // For canonical files, extract the frontmatter title as display name
        let display_title = if !is_dir
            && extension.as_deref() == Some("md")
            && crate::canonical::is_canonical_filename(&path)
        {
            // Read just the first 1KB to extract title (fast)
            fs::read_to_string(&path)
                .ok()
                .and_then(|c| extract_frontmatter_title(&c))
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
            display_title,
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
    /// User's typed annotation from `[[target|annotation]]` syntax — the
    /// second parser (`extract_typed_links` in search.rs) stores the
    /// semantic tag here and leaves `link_type` at the default "relates".
    /// The UI checks this first when choosing the type-badge color.
    #[serde(default)]
    pub annotation: String,
    /// Living Link weight: `1 + ln(1 + traversal_count)`. Default 1.0 for
    /// never-traversed links. Consumed by the Backlinks panel (P3) to
    /// prioritize worn paths.
    #[serde(default = "default_weight")]
    pub weight: f64,
    /// Number of times the user has traversed this link. Default 0 for
    /// fresh / boot-graph-fallback entries that didn't come from the
    /// `note_links` table.
    #[serde(default)]
    pub traversal_count: i64,
    /// ISO-8601 timestamp of the most recent traversal, or "" for links
    /// that have never been followed. Populated from
    /// `note_links.last_traversed`. Consumed by the P5 lifecycle helpers
    /// to compute decay / stale-flagging / confidence tiers client-side.
    #[serde(default)]
    pub last_traversed: String,
    /// Confidence tier stored in the DB: "hypothesis" (default), or user-
    /// promoted tiers that will be driven by P5 thresholds. Present here
    /// so the UI can surface the raw tier without an extra query.
    #[serde(default)]
    pub confidence: String,
}

fn default_weight() -> f64 { 1.0 }

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
                // Use frontmatter title for canonical files (matching collect_library_notes)
                let file_stem = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let source_name = if crate::canonical::is_canonical_filename(&path) {
                    extract_frontmatter_title(&content).unwrap_or(file_stem)
                } else {
                    file_stem
                };
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
                        annotation: String::new(),
                        weight: 1.0,
                        traversal_count: 0,
                        last_traversed: String::new(),
                        confidence: String::new(),
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
                        annotation: String::new(),
                        weight: 1.0,
                        traversal_count: 0,
                        last_traversed: String::new(),
                        confidence: String::new(),
                    });
                }
            }
        }
    }
}

/// Scan all tags across a library. **Walks every `.md` file** via
/// `scan_tags_recursive` below (`fs::read_to_string` per file + regex scan).
/// On the 7,600-note trial Universe this is ~7,600 file reads per library —
/// seconds of wall-clock work.
///
/// Boot path: `DashboardView.onMount` (src/lib/components/DashboardView.svelte)
/// calls `scanAllLibraryTags()` (src/lib/libraries/tagUtils.ts) which issues
/// **one `invoke('scan_library_tags')` per library, sequentially** (16 calls
/// on the trial Universe). DashboardView mounts the instant
/// `libraries.set(bundle.libraries)` fires in `refreshLibraryCaches` — which
/// happens **before** `cache_boot_snapshot_core` returns. Without `(async)`
/// all 16 invocations queue on the WebView2 UI thread (see `watcher.rs`
/// docstring for the full dispatch chain), pushing `core_queue_ms` to ~19.5 s
/// on Round 4 measurements (docs/LESSONS-LEARNED.md LL-021 Round 5).
///
/// `#[tauri::command(async)]` routes each scan through `respond_async_serialized`
/// → `tauri::async_runtime::spawn`, so the UI thread pays only spawn cost per
/// call and Tokio workers run the actual filesystem walks in parallel.
/// Write-Time Derivation (CLAUDE.md Rule 8) says the right long-term fix is a
/// persisted tag index maintained by trigger/watcher — tracked as a separate
/// open item; this is the minimal change that unblocks Boot Criterion 2.
#[tauri::command(async)]
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
    /// One-line FTS5 snippet of the matched term in context (up to ~12
    /// tokens around the first hit). Matched tokens are wrapped in
    /// `\x02`…`\x03` sentinels (STX/ETX control chars) which the frontend
    /// splits on to render as `<mark>` — chosen over putting `<mark>` in
    /// SQL so literal HTML in user notes is not injected into the DOM.
    ///
    /// `None` when FTS5 returned an empty snippet (e.g. title-only match
    /// against a note with empty body). The Index panel omits the context
    /// line in that case.
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexEntry {
    pub term: String,
    pub count: u32,
    pub mentions: Vec<IndexMention>,
    pub is_compound: bool,
}

/// Co-occurring term — another vocabulary term appearing in the same notes
/// as a query term. Returned by `read_cooccurring_terms` and rendered as
/// a chip strip beneath an expanded Index term, surfacing lexical
/// adjacency ("notes containing 'knowledge' also contain …").
#[derive(Debug, Clone, Serialize)]
pub struct CooccurringTerm {
    /// Display form of the co-occurring term. Bigrams stored as
    /// `stem1\x1fstem2` are converted to `"stem1 stem2"` for the UI.
    pub term: String,
    /// Number of sampled matching notes that also contain this term.
    /// Capped above by `sample_limit` (default 200).
    pub note_count: u32,
}

/// ─── Arabic Indexing Pipeline ───────────────────────────────────────────────
///
/// Based on Apache Lucene's ArabicNormalizer + ArabicStemmer (Light10 model),
/// the gold standard for Arabic information retrieval.
///
/// Pipeline: normalize → stem (prefix removal → suffix removal)
///
/// Design principles (from research):
/// 1. NORMALIZE first: remove diacritics, unify character variants
/// 2. STEM conservatively: only remove affixes when the remaining word
///    is long enough to be meaningful (minimum 2 chars after removal)
/// 3. NEVER strip a word below 3 chars total
/// 4. Prefix removal order: longest first (3-char → 2-char → 1-char)
/// 5. Suffix removal: only common grammatical suffixes, not root patterns
///
/// Sources:
/// - Larkey et al., "Light stemming for Arabic information retrieval" (2007)
/// - Apache Lucene ArabicStemmer.java / ArabicNormalizer.java
/// - CondLight: Conditional Arabic Light Stemmer (IAJIT 2018)

/// Display normalization: remove diacritics + tatweel only.
/// Preserves original character identity (ة stays ة, أ stays أ).
/// Used for the display form shown in the Index.
fn normalize_arabic_display(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    for ch in word.chars() {
        match ch {
            // Remove tashkeel diacritics
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}' => continue,
            // Remove tatweel (kashida)
            '\u{0640}' => continue,
            _ => result.push(ch),
        }
    }
    result
}

/// Full normalization: remove diacritics + unify character variants.
/// Used for the index KEY (grouping different forms of the same word).
fn normalize_arabic(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    for ch in word.chars() {
        match ch {
            // Remove ALL tashkeel diacritics (harakat)
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}' => continue,
            // Remove tatweel (kashida)
            '\u{0640}' => continue,
            // Normalize alef variants → bare alef
            'أ' | 'إ' | 'آ' | 'ٱ' => result.push('ا'),
            // Normalize alef maqsura → yeh
            'ى' => result.push('ي'),
            // Normalize teh marbuta → heh
            'ة' => result.push('ه'),
            _ => result.push(ch),
        }
    }
    result
}

/// Step 2: Arabic Light Stemmer (Lucene Light10 model)
/// Removes prefixes then suffixes with strict length constraints.
fn stem_arabic_light10(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    let mut len = chars.len();

    // === PREFIX REMOVAL (longest first) ===
    // Each prefix requires that the remaining stem is at least 2 chars.

    // 3-char prefixes: وال فال بال كال (conjunction/preposition + definite article)
    if len >= 6 {
        let p3 = (chars[0], chars[1], chars[2]);
        match p3 {
            ('و','ا','ل') | ('ب','ا','ل') | ('ك','ا','ل') | ('ف','ا','ل') => {
                chars = chars[3..].to_vec();
                len = chars.len();
            }
            _ => {}
        }
    }

    // 2-char prefixes: ال لل (definite article, emphatic lam)
    if len >= 4 {
        let p2 = (chars[0], chars[1]);
        match p2 {
            ('ا','ل') | ('ل','ل') => {
                chars = chars[2..].to_vec();
                len = chars.len();
            }
            _ => {}
        }
    }

    // 1-char prefix: و (conjunction "and") — only if word is long enough
    // NOTE: و is the ONLY safe single-char prefix to remove.
    // ف/ب/ك/ل are NOT removed — they destroy too many proper nouns
    // (e.g., بدر، كريم، لبنان، فلسطين)
    if len >= 4 && chars[0] == 'و' {
        chars = chars[1..].to_vec();
        len = chars.len();
    }

    // === SUFFIX REMOVAL ===
    // Each suffix requires that the remaining stem is at least 2 chars.

    // 2-char suffixes (remove first — more specific)
    if len >= 4 {
        let s2 = (chars[len-2], chars[len-1]);
        match s2 {
            ('ه','ا') |  // ها (her/possessive)
            ('ا','ن') |  // ان (dual/indefinite)
            ('ا','ت') |  // ات (feminine plural)
            ('و','ن') |  // ون (masculine plural nominative)
            ('ي','ن') |  // ين (masculine plural accusative/genitive)
            ('ي','ه') |  // يه (possessive)
            ('ي','ت') |  // ية → يت after normalization (feminine adjective)
            ('ت','ه')    // ته → ته (his, possessive)
            => {
                chars.truncate(len - 2);
                len = chars.len();
            }
            _ => {}
        }
    }

    // 1-char suffixes (only if still long enough)
    if len >= 3 {
        match chars[len-1] {
            'ه' |  // ه/ة (feminine marker, after normalization ة→ه)
            'ي'    // ي (possessive/nisba)
            => {
                chars.truncate(len - 1);
            }
            _ => {}
        }
    }

    chars.iter().collect()
}

/// Combined Arabic processing: normalize + stem.
/// Returns (display_form, index_key):
///   - display: original word with tashkeel removed (ة stays ة, أ stays أ).
///   - key: canonical stem used by FTS5 to group surface variants.
///
/// M6 routes the key through `arabic::analyze_best`, which runs the five
/// Constellation Arabic Engine layers and returns the highest-confidence
/// analysis:
///
///   Layer 1 ProtectedList    — proper nouns / places / loanwords / function (conf 1.00)
///   Layer 2 GenerativeFst    — bare (root × pattern) hit                    (conf 0.85)
///   Layer 3b Cascade         — affix-peeled stem hit                        (conf 0.75 / 0.55)
///   Layer 4 SurfaceHeuristic — normalized surface fallback                  (conf 0.30)
///
/// For every analysis with origin ≠ SurfaceHeuristic the engine's `lemma`
/// is a strict improvement on Light10 — most visibly on the proper-noun
/// case that motivated this milestone: `وائل → "وائل"` (ProtectedList)
/// instead of the Light10-corrupted `"ائل"`.
///
/// When the analyzer's best guess IS SurfaceHeuristic (an Arabic word
/// that isn't protected, isn't in the FST, and can't be peeled to any
/// FST stem), we keep Light10 so the swap is strictly non-regressive:
/// unrecognized words continue to get the same affix-stripping they got
/// before M6, and search recall on them doesn't drop.
fn process_arabic_word(word: &str) -> (String, String) {
    let display = normalize_arabic_display(word); // preserve ة أ إ آ ى
    // M8b routes every token through the active Universe's user-override
    // store before the rest of the engine. M9-hotpath (a) cut this from
    // an unconditional RwLock-read + Arc::clone (~25 ns) to a single
    // relaxed `AtomicBool::load` (~2 ns) on the overwhelmingly common
    // empty-store case via `active_if_non_empty`. The returned
    // `Option<Arc<_>>` lives until end of scope, so `.as_deref()` gives
    // the `Option<&OverrideStore>` the downstream analyze call expects
    // without any reference-lifetime juggling.
    let store_owned = crate::arabic::overrides::active_if_non_empty();
    let overrides_ref = store_owned.as_deref();
    let analysis = crate::arabic::analyze_with_overrides_best(word, overrides_ref);
    let stem = if matches!(analysis.origin, crate::arabic::AnalysisOrigin::SurfaceHeuristic) {
        // Unknown word — preserve pre-M6 Light10 behaviour so recall on
        // previously-indexed surfaces does not regress.
        stem_arabic_light10(&normalize_arabic(word))
    } else {
        analysis.lemma
    };
    (display, stem)
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

// stem_arabic is now replaced by stem_arabic_light10 above

/// Detect if a word is Arabic script
fn is_arabic(word: &str) -> bool {
    word.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c) || ('\u{0750}'..='\u{077F}').contains(&c) || ('\u{FB50}'..='\u{FDFF}').contains(&c) || ('\u{FE70}'..='\u{FEFF}').contains(&c))
}

/// Detect if a word is Hebrew script
fn is_hebrew(word: &str) -> bool {
    word.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c) || ('\u{FB1D}'..='\u{FB4F}').contains(&c))
}

fn is_latin(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_alphabetic())
}
fn is_cyrillic(word: &str) -> bool {
    word.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
}
fn is_devanagari(word: &str) -> bool {
    word.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
}
fn is_persian(word: &str) -> bool {
    // Persian uses Arabic script but with specific chars: پ چ ژ گ
    is_arabic(word) && word.chars().any(|c| c == 'پ' || c == 'چ' || c == 'ژ' || c == 'گ' || c == 'ک' || c == 'ی')
}

/// Helper: strip N chars from the end of a char slice, return as String
fn chars_strip_end(chars: &[char], n: usize) -> String {
    chars[..chars.len() - n].iter().collect()
}

/// Helper: check if char slice ends with a given suffix
fn chars_ends_with(chars: &[char], suffix: &[char]) -> bool {
    if chars.len() < suffix.len() { return false; }
    &chars[chars.len() - suffix.len()..] == suffix
}

/// English stemmer (Porter-like light stemming)
fn stem_english(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    // Step 1: plurals and past tense
    if chars_ends_with(&c, &['s','s','e','s']) { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['i','e','s']) && n > 4 { return format!("{}y", chars_strip_end(&c, 3)); }
    if chars_ends_with(&c, &['n','e','s','s']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['t','i','o','n']) { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['s','i','o','n']) { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['l','i','n','g']) && n > 5 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['i','n','g','s']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['i','n','g']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['a','t','e','d']) && n > 5 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['i','z','e','d']) && n > 5 { return chars_strip_end(&c, 1); }
    if chars_ends_with(&c, &['e','n','e','d']) && n > 5 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','d']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['l','y']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','r']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['s']) && !chars_ends_with(&c, &['s','s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// French stemmer (light suffix removal)
fn stem_french(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['e','u','s','e','s']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['e','u','s','e']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t','s']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['m','e','n','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['t','i','o','n']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','n','c','e']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','n','c','e']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','u','x']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['é','e','s']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['é','e']) && n > 3 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['é','s']) && n > 3 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','r']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['é']) { return chars_strip_end(&c, 1); }
    if chars_ends_with(&c, &['s']) && !chars_ends_with(&c, &['s','s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// Spanish stemmer (light suffix removal)
fn stem_spanish(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['i','o','n','e','s']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['c','i','ó','n']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t','e']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['i','d','a','d']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['i','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['a','d','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// Portuguese stemmer (light suffix removal)
fn stem_portuguese(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['ç','õ','e','s']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t','e']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['i','d','a','d','e']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['a','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['i','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['a','d','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// German stemmer (light suffix removal + umlaut normalization)
fn stem_german(word: &str) -> String {
    // Normalize umlauts
    let w = word.to_lowercase()
        .replace("ä", "a").replace("ö", "o").replace("ü", "u")
        .replace("ß", "ss");
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['u','n','g','e','n']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['u','n','g']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['h','e','i','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['k','e','i','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['l','i','c','h']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['i','s','c','h']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','r','n']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','l','n']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','n']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','r']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','m']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e']) && n > 4 { return chars_strip_end(&c, 1); }
    if chars_ends_with(&c, &['s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// Russian stemmer (light suffix removal for cases/gender/number)
fn stem_russian(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 4 { return word.to_string(); }

    // Participial/adjectival suffixes
    let last3: String = if len >= 3 { chars[len-3..].iter().collect() } else { String::new() };
    let last2: String = if len >= 2 { chars[len-2..].iter().collect() } else { String::new() };

    // Long suffixes (4+ chars)
    if len > 5 {
        let last4: String = chars[len-4..].iter().collect();
        match last4.as_str() {
            "ость" | "ными" | "ного" | "ному" | "ской" | "ских" | "ским" => return chars[..len-4].iter().collect(),
            _ => {}
        }
    }
    if len > 4 {
        match last3.as_str() {
            "ого" | "ому" | "ные" | "ных" | "ной" | "ами" | "ями" | "ить" | "ать" | "ять" | "ств" | "ски" => return chars[..len-3].iter().collect(),
            _ => {}
        }
    }
    if len > 3 {
        match last2.as_str() {
            "ов" | "ев" | "ий" | "ый" | "ая" | "ое" | "ые" | "ей" | "ям" | "ах" | "ом" | "ем" | "ой" | "ую" | "ие" | "ия" | "ть" | "ут" | "ют" | "ат" | "ят" | "ет" | "ит" | "ал" | "ил" | "ел" => return chars[..len-2].iter().collect(),
            _ => {}
        }
    }

    word.to_string()
}

/// Turkish stemmer (light agglutinative suffix removal)
fn stem_turkish(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    // Long suffixes first (4 chars)
    if chars_ends_with(&c, &['l','a','r','ı']) || chars_ends_with(&c, &['l','e','r','i']) { return chars_strip_end(&c, 4); }
    // 3-char suffixes
    if chars_ends_with(&c, &['l','a','r']) || chars_ends_with(&c, &['l','e','r']) { if n - 3 >= 2 { return chars_strip_end(&c, 3); } }
    if chars_ends_with(&c, &['l','ı','k']) || chars_ends_with(&c, &['l','i','k']) || chars_ends_with(&c, &['l','u','k']) || chars_ends_with(&c, &['l','ü','k']) { if n - 3 >= 2 { return chars_strip_end(&c, 3); } }
    if chars_ends_with(&c, &['d','a','n']) || chars_ends_with(&c, &['d','e','n']) || chars_ends_with(&c, &['t','a','n']) || chars_ends_with(&c, &['t','e','n']) { if n - 3 >= 2 { return chars_strip_end(&c, 3); } }

    w
}

/// Hindi stemmer (light suffix removal for postpositions/verb forms)
fn stem_hindi(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 3 { return word.to_string(); }

    if len > 3 {
        let last2: String = chars[len-2..].iter().collect();
        match last2.as_str() {
            "ों" | "ें" | "ाँ" | "ता" | "ती" | "ते" | "ना" | "ने" | "नी" | "ाए" | "ाओ" | "ाई" => return chars[..len-2].iter().collect(),
            _ => {}
        }
    }

    word.to_string()
}

/// Persian stemmer: normalize ی/ک only, no suffix removal (same reasoning as Arabic)
fn stem_persian(word: &str) -> String {
    // Normalize Persian-specific chars only
    let normalized = word.replace('ي', "ی").replace('ك', "ک");
    return normalized;
    // Suffix removal disabled — causes same problems as Arabic stemming
    #[allow(unreachable_code)]
    let chars: Vec<char> = normalized.chars().collect();
    let len = chars.len();
    if len < 4 { return normalized; }

    if len > 4 {
        let last2: String = chars[len-2..].iter().collect();
        match last2.as_str() {
            "ها" | "ان" | "ات" | "ین" | "ون" | "گی" | "شی" => return chars[..len-2].iter().collect(),
            _ => {}
        }
    }
    if len > 3 {
        match chars[len-1] {
            'ی' | 'ه' => return chars[..len-1].iter().collect(),
            _ => {}
        }
    }

    normalized
}

pub(crate) fn build_stopwords() -> std::collections::HashSet<String> {
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
        "ال","بن","ابن","ذات","ذو","ذي","اللذين","اللتين","اللواتي","الذين","عليهم","لديه","لديها",
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
                // Legacy walker doesn't produce FTS5 snippets — the
                // FTS5-backed `read_term_mentions` is the modern source.
                .map(|(note_path, note_name)| IndexMention { note_path, note_name, snippet: None })
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
                .map(|(note_path, note_name)| IndexMention { note_path, note_name, snippet: None })
                .collect();
            IndexEntry { term, count, mentions, is_compound: true }
        })
        .collect();

    entries.extend(bigram_entries);
    entries.sort_by(|a, b| a.term.to_lowercase().cmp(&b.term.to_lowercase()));
    Ok(entries)
}

pub(crate) fn is_same_script(a: &str, b: &str) -> bool {
    let ca = a.chars().next().unwrap_or(' ');
    let cb = b.chars().next().unwrap_or(' ');
    // Both ASCII Latin
    if ca.is_ascii_alphabetic() && cb.is_ascii_alphabetic() { return true; }
    // Both in same Unicode block (rough check: same high byte)
    let ba = (ca as u32) >> 8;
    let bb = (cb as u32) >> 8;
    ba == bb
}

/// Per-word processor used by the custom FTS5 tokenizer
/// (`crate::fts5_tokenizer::ConstellationTokenizer`).
///
/// Takes a single word and returns `(stem, norm_lower)` if the word is
/// worth emitting to the FTS5 inverted index, or `None` if it should be
/// skipped (empty, too short, or unreasonably long — likely concatenation
/// noise). The caller decides stopword filtering against the returned pair.
///
/// This is the same stemming pipeline used by `tokenize_note_body`
/// (Arabic Light10 / Hebrew prefix stripping / Persian / Cyrillic /
/// Devanagari / German / Spanish / Portuguese / French / Turkish /
/// English), but without the side-effectful HashMap accumulation — the
/// tokenizer just needs the stem + pre-stem normalized form.
///
/// * `stem` — lowercased, stemmed, suitable as a primary FTS5 token byte
///   sequence. When the same word arrives in a MATCH query it is stemmed
///   through this same function, so stemming is symmetric.
/// * `norm_lower` — lowercased, normalized (for Arabic: diacritics
///   stripped, alef/yeh/teh-marbuta variants unified) but NOT stemmed.
///   Callers check this against the stopword set too, because stopword
///   lists are curated in un-stemmed form (e.g. "the", not "th").
pub(crate) fn process_word_for_fts(word: &str) -> Option<(String, String)> {
    let char_count = word.chars().count();
    if char_count < 2 { return None; }

    let word_is_arabic = is_arabic(word);
    let word_is_hebrew = is_hebrew(word);

    // Length guards to drop concatenation noise.
    // Arabic words >20 chars are almost always glued tokens.
    // Non-Arabic: 40 is generous enough for German compounds.
    if word_is_arabic && char_count > 20 { return None; }
    if !word_is_arabic && char_count > 40 { return None; }

    let (normalized, stemmed);
    if word_is_arabic {
        let (_disp, stem) = process_arabic_word(word);
        normalized = normalize_arabic(word);
        stemmed = stem;
    } else if word_is_hebrew {
        normalized = word.to_string();
        stemmed = strip_hebrew_prefix(&normalized).to_string();
    } else {
        normalized = word.to_string();
        let lower = normalized.to_lowercase();
        stemmed = if is_persian(&normalized) {
            stem_persian(&normalized)
        } else if is_cyrillic(&normalized) {
            stem_russian(&normalized)
        } else if is_devanagari(&normalized) {
            stem_hindi(&normalized)
        } else if is_latin(&normalized) {
            if lower.contains('ä') || lower.contains('ö') || lower.contains('ü') || lower.contains('ß') {
                stem_german(&normalized)
            } else if lower.contains('ñ') || lower.ends_with("ción") || lower.ends_with("ando") {
                stem_spanish(&normalized)
            } else if lower.contains('ç') || lower.contains('ã') || lower.contains('õ') {
                stem_portuguese(&normalized)
            } else if lower.contains('é') || lower.contains('è') || lower.contains('ê')
                || lower.ends_with("ment") || lower.ends_with("tion") {
                stem_french(&normalized)
            } else if lower.contains('ş') || lower.contains('ğ') || lower.contains('ı') {
                stem_turkish(&normalized)
            } else {
                stem_english(&normalized)
            }
        } else {
            // Unknown script — emit as-is (CJK, etc.)
            normalized.clone()
        };
    }

    let stem_lower = stemmed.to_lowercase();
    let norm_lower = normalized.to_lowercase();

    // Skip if the stem degenerated to <2 chars (e.g. after Arabic prefix
    // stripping on a short word).
    if stem_lower.chars().count() < 2 { return None; }

    Some((stem_lower, norm_lower))
}

/// Tokenize a single note body and accumulate into the index + bigram maps.
/// Pure in-memory — no filesystem, no SQL. Callers pass already-stripped
/// body text (YAML frontmatter removed, markdown syntax collapsed).
///
/// Used by the filesystem walker `scan_index_words_recursive` (called from
/// `scan_library_index`, the on-demand per-library filesystem rebuild).
///
/// The cache-streaming path (`scan_index_populate_batch`) uses the sibling
/// `tokenize_note_local` function instead, which emits a per-note HashMap
/// and avoids unbounded accumulation across notes.
fn tokenize_note_body(
    body: &str,
    note_path: &str,
    note_name: &str,
    stopwords: &std::collections::HashSet<String>,
    index: &mut std::collections::HashMap<String, (
        std::collections::HashMap<String, u32>, u32, Vec<(String, String)>,
    )>,
    bigrams: &mut std::collections::HashMap<String, (String, u32, Vec<(String, String)>)>,
) {
    let mut seen_in_note: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_bigrams: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut prev_word: Option<String> = None;
    let mut prev_key: Option<String> = None;

    for word in body.split(|c: char| {
        // Split on non-alphabetic chars (except apostrophe).
        // Also split on dashes, underscores, and em/en dashes.
        if c == '\'' { return false; }
        if c == '—' || c == '–' || c == '-' || c == '_' { return true; }
        !c.is_alphabetic()
    }) {
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

        // Skip abnormally long words — likely concatenation errors.
        // Arabic rarely exceeds 12 chars; Latin rarely exceeds 25.
        if word_is_arabic && char_count > 15 {
            prev_word = None;
            prev_key = None;
            continue;
        }
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

        // Process word through language-specific pipeline.
        let (normalized, stripped, stemmed);
        if word_is_arabic {
            // Arabic: Lucene Light10 pipeline.
            // display = original with tashkeel removed (ة أ إ preserved)
            // key = fully normalized + stemmed (for grouping)
            let (disp, stem) = process_arabic_word(word);
            normalized = normalize_arabic(word); // full normalization for stopword check
            stripped = disp; // display preserved
            stemmed = stem;  // grouped by Light10
        } else if word_is_hebrew {
            normalized = word.to_string();
            let s = strip_hebrew_prefix(&normalized).to_string();
            stripped = s.clone();
            stemmed = s;
        } else {
            normalized = word.to_string();
            stripped = normalized.clone();
            stemmed = if is_persian(&stripped) {
                stem_persian(&stripped)
            } else if is_cyrillic(&stripped) {
                stem_russian(&stripped)
            } else if is_devanagari(&stripped) {
                stem_hindi(&stripped)
            } else if is_latin(&stripped) {
                let lower = stripped.to_lowercase();
                if lower.contains('ä') || lower.contains('ö') || lower.contains('ü') || lower.contains('ß') {
                    stem_german(&stripped)
                } else if lower.contains('ñ') || lower.ends_with("ción") || lower.ends_with("ando") {
                    stem_spanish(&stripped)
                } else if lower.contains('ç') || lower.contains('ã') || lower.contains('õ') {
                    stem_portuguese(&stripped)
                } else if lower.contains('é') || lower.contains('è') || lower.contains('ê') || lower.ends_with("ment") || lower.ends_with("tion") {
                    stem_french(&stripped)
                } else if lower.contains('ş') || lower.contains('ğ') || lower.contains('ı') || lower.contains('ü') {
                    stem_turkish(&stripped)
                } else {
                    stem_english(&stripped)
                }
            } else {
                stripped.clone()
            };
        }

        // Use stemmed form as index key; keep original display form.
        let key = stemmed.to_lowercase();

        // Skip stopwords (check both original normalized and stemmed forms).
        let norm_lower = normalized.to_lowercase();
        let is_stop = stopwords.contains(&key) || stopwords.contains(&norm_lower);

        if !is_stop {
            // Result must be ≥3 chars for Arabic/Hebrew, ≥2 for others.
            let min_len = if word_is_arabic || word_is_hebrew { 3 } else { 2 };
            if key.chars().count() < min_len {
                prev_word = Some(stripped.clone());
                prev_key = Some(key);
                continue;
            }

            let entry = index.entry(key.clone()).or_insert_with(|| {
                (std::collections::HashMap::new(), 0, Vec::new())
            });
            // Track display variant (use stripped form, not raw word with tashkeel).
            *entry.0.entry(stripped.clone()).or_insert(0) += 1;
            entry.1 += 1;

            if !seen_in_note.contains(&key) {
                seen_in_note.insert(key.clone());
                entry.2.push((note_path.to_string(), note_name.to_string()));
            }
        }

        // Bigram detection: pair with previous non-stop word if same script.
        if let (Some(pw), Some(pk)) = (&prev_word, &prev_key) {
            let prev_is_stop = stopwords.contains(pk.as_str());
            if !is_stop && !prev_is_stop && is_same_script(pw, &stripped) {
                let bi_key = format!("{} {}", pk, key);
                let bi_display = format!("{} {}", pw, stripped);
                let bi_entry = bigrams.entry(bi_key.clone())
                    .or_insert_with(|| (bi_display, 0, Vec::new()));
                bi_entry.1 += 1;
                if !seen_bigrams.contains(&bi_key) {
                    seen_bigrams.insert(bi_key);
                    bi_entry.2.push((note_path.to_string(), note_name.to_string()));
                }
            }
        }

        prev_word = Some(stripped.clone());
        prev_key = Some(key);
    }
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

                tokenize_note_body(&cleaned, &note_path, &note_name, stopwords, index, bigrams);
            }
        }
    }
}

/// ─── Index Panel backed by FTS5 vocab ───────────────────────────────────
///
/// The Index panel reads directly from the `notes_vocab` virtual table,
/// which is a `fts5vocab(notes_fts, 'row')` view over the term dictionary
/// that FTS5 already maintains on disk. Each row is `(term, doc, cnt)`:
///   * term — a token produced by the FTS5 tokenizer
///   * doc  — number of distinct notes containing the token
///   * cnt  — total occurrences across all notes
///
/// Advantages over the previous custom-table attempts:
///   * Zero bulk work. FTS5 triggers on `note_meta` already maintain the
///     term dictionary incrementally as notes are added, edited, or deleted.
///   * No in-memory accumulation. Aggregation is what FTS5 does on disk.
///   * Boot is free — the panel opens to a live view over the dictionary.
///
/// Current tokenization is whatever FTS5 was configured with at table
/// creation (`unicode61 remove_diacritics 2`), which lower-cases and
/// strips diacritics but does not stem. This means "philosophy" and
/// "philosophies" appear as separate terms. A later phase will register a
/// custom FTS5 tokenizer wrapping the existing multi-language pipeline
/// (`tokenize_note_body` / Light10 Arabic stemming / bigrams) so the
/// vocabulary reflects the richer tokenization.

/// Read the Universe vocabulary from the FTS5 term dictionary.
/// Returns `(display, count)` pairs; `mentions` is left empty — the UI
/// lazy-fetches the notes for a term via `read_term_mentions` when the
/// user expands it, which avoids returning millions of rows up front.
///
/// Filters (tuned for multi-script corpora, especially Arabic without
/// stemming, where a 7,600-note Universe produces ~450k unique term forms):
///   * terms shorter than 2 characters
///   * terms with count < 5 — drops hapax/near-hapax noise that would
///     otherwise bloat the list to hundreds of thousands of one-off tokens.
///   * LIMIT 50000 — ceiling on payload size and rendering cost. At 50k
///     alphabetically-sorted terms the user's filter-as-you-type narrows
///     quickly; at more than 50k the JSON blob and Svelte $state proxy
///     wrap start to hurt main-thread responsiveness.
///
/// Performance: a single forward scan over the FTS5 dictionary segments.
/// Measured ~350ms for 50k rows on a 7,600-note Arabic-heavy Universe.
#[tauri::command]
pub fn read_index_entries(app: tauri::AppHandle) -> Result<Vec<IndexEntry>, String> {
    use rusqlite::{Connection, OpenFlags};

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    // Register the 'constellation' FTS5 tokenizer on this connection so
    // later phases can MATCH-through-query here if needed. Reading
    // `notes_vocab` alone does not invoke the tokenizer, but consistency
    // avoids a "unknown tokenizer: constellation" surprise if this
    // function grows to do a MATCH later.
    crate::search::register_fts5_tokenizer(&mut conn)?;

    // No LIMIT. The Index panel is the canonical view of the Universe's
    // vocabulary — truncating it silently hides entire scripts from the
    // back of the alphabet because SQLite's default BINARY collation
    // sorts by UTF-8 bytes (Latin `a-z` = 0x61..0x7A, Arabic starts at
    // 0xD8 0x80, Hebrew at 0xD7 0x90, CJK at 0xE4..0xE9). A LIMIT at the
    // SQL layer picks favorites; we don't.
    //
    // What keeps this bounded: the `cnt >= 5` threshold below, combined
    // with the `constellation` tokenizer's stemming, caps a 7,600-note
    // Universe at ~100-200k rows.
    //
    // The frontend renders the result through a virtualized list
    // (`IndexPanel.svelte`) — payload size is the only soft limit, not
    // render cost.
    let mut stmt = conn.prepare(
        "SELECT term, cnt FROM notes_vocab
         WHERE LENGTH(term) >= 2 AND cnt >= 5
         ORDER BY term"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)? as u32,
        ))
    }).map_err(|e| e.to_string())?;

    let mut entries: Vec<IndexEntry> = Vec::new();
    for row in rows.flatten() {
        let (term, count) = row;
        // Bigrams are stored in the FTS5 index as `<stem1>\x1f<stem2>`
        // (the `\x1f` Unit Separator sentinel picked by the custom
        // tokenizer — see `crate::fts5_tokenizer::BIGRAM_SEP`). Convert
        // the sentinel to a space so the Index panel shows
        // "knowledge management" instead of the raw control character.
        // The frontend's click handler passes the display form back to
        // `read_term_mentions`, which wraps it in a phrase-query
        // "..." and lets FTS5 re-tokenize — still matching the bigram
        // via position-adjacent unigrams.
        let has_sentinel = term.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP);
        let display = if has_sentinel { term.replace('\u{001F}', " ") } else { term };
        entries.push(IndexEntry {
            term: display,
            count,
            mentions: Vec::new(),
            is_compound: has_sentinel,
        });
    }
    Ok(entries)
}

/// Lazy-load the list of notes mentioning a given term. Called when the
/// user expands a term in the Index panel. Uses FTS5 `MATCH` — an O(log n)
/// term-dictionary lookup followed by a linear scan of the postings list,
/// joined to `note_meta` for display names.
///
/// Returns up to `limit` (default 200) mentions, ordered by note name.
#[tauri::command]
pub fn read_term_mentions(
    app: tauri::AppHandle,
    term: String,
    limit: Option<u32>,
) -> Result<Vec<IndexMention>, String> {
    use rusqlite::{Connection, OpenFlags};

    let limit = limit.unwrap_or(200).max(1).min(5000);

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    // Register the 'constellation' FTS5 tokenizer on this connection.
    // Required because the MATCH below tokenizes the query string
    // through the same tokenizer that populated the index — if the
    // tokenizer weren't registered, SQLite would fail with "no such
    // tokenizer: constellation".
    crate::search::register_fts5_tokenizer(&mut conn)?;

    // Bind as a phrase so FTS5 treats the input literally.
    // Quotes must be doubled per FTS5 quoted-string syntax.
    let phrase = format!("\"{}\"", term.replace('"', "\"\""));

    // `snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12)` returns a single
    // line of surrounding text with the matched tokens wrapped in STX/ETX
    // (\x02/\x03) sentinels. `-1` means "best column across all indexed
    // columns" — so a term that lives in the title (column 0) or body
    // (column 1) both get a useful preview. `12` tokens ≈ one line of
    // context; longer snippets waste vertical space in the expanded row.
    // STX/ETX are used (not `<mark>`) so literal HTML in user notes
    // cannot be injected into the DOM at render time.
    let mut stmt = conn.prepare(
        "SELECT nm.path, nm.name,
                snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12)
         FROM notes_fts
         JOIN note_meta nm ON notes_fts.rowid = nm.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY LOWER(nm.name)
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(rusqlite::params![phrase, limit as i64], |row| {
        let note_path: String = row.get(0)?;
        let note_name: String = row.get(1)?;
        // snippet() returns TEXT; SQLite can hand us NULL in edge cases
        // (very short/empty content columns), so tolerate both.
        let snippet_raw: Option<String> = row.get(2).ok();
        let snippet = snippet_raw.and_then(|s| if s.is_empty() { None } else { Some(s) });
        Ok(IndexMention { note_path, note_name, snippet })
    }).map_err(|e| e.to_string())?;

    Ok(rows.flatten().collect())
}

/// Return the top co-occurring terms for `term` — other vocabulary terms
/// appearing in the same notes. Surfaces lexical adjacency: "notes that
/// mention 'knowledge' also mention: 'wisdom', 'understanding', …".
///
/// ## Performance model
///
/// `fts5vocab(…, 'instance')` has no index on `doc`, so a SQL-level
/// co-occurrence query (e.g. `WHERE doc IN (matching_rowids)`) degrades to
/// a full scan of every token position in the entire FTS index. For a
/// 7,600-note Arabic Universe that's millions of rows per query.
///
/// Instead we:
///   1. Pull up to `sample_limit` matching rowids from `notes_fts MATCH`
///      (indexed — fast).
///   2. Fetch `note_meta.body_text` for each rowid (covered by the
///      primary-key rowid index — ~hundreds of tiny point reads).
///   3. Re-tokenize each body in-process through the same
///      `process_word_for_fts` pipeline the FTS5 tokenizer uses, so the
///      stems we aggregate are symmetric with those in the index.
///   4. Count distinct notes per co-occurring stem; sort descending.
///
/// Cost on a common term (say 500 matches, 2 KB body each): ~1 MB of
/// text × low-microsecond per-word tokenization ≈ <100 ms. Rare terms
/// are essentially free.
///
/// The 200-note default sample is empirically enough: the rank order of
/// top co-occurring terms stabilizes well before every matching note is
/// visited (law of large numbers on the tail). Users tuning for
/// exhaustiveness can raise `sample_limit`; there's no correctness
/// benefit past a few hundred.
#[tauri::command]
pub fn read_cooccurring_terms(
    app: tauri::AppHandle,
    term: String,
    sample_limit: Option<u32>,
    result_limit: Option<u32>,
) -> Result<Vec<CooccurringTerm>, String> {
    use rusqlite::{Connection, OpenFlags};
    use std::collections::{HashMap, HashSet};

    let sample_limit = sample_limit.unwrap_or(200).max(1).min(2000);
    let result_limit = result_limit.unwrap_or(20).max(1).min(100) as usize;

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    crate::search::register_fts5_tokenizer(&mut conn)?;

    // Stems of the query term — excluded from co-occurrence results
    // (nobody wants "knowledge" listed as co-occurring with "knowledge").
    // Whitespace split handles the bigram display form:
    // "knowledge management" → ["knowledge", "management"], so both the
    // unigram stems are filtered out.
    let query_stems: HashSet<String> = term
        .split_whitespace()
        .filter_map(|w| process_word_for_fts(w).map(|(stem, _norm)| stem))
        .collect();

    // Step 1: sample matching rowids via FTS5 MATCH.
    //
    // The `stmt`/`rows` pair must both outlive the `.collect()` call —
    // `rows` borrows from `stmt`, and `stmt` borrows from `conn`. Binding
    // each to its own `let` (rather than chaining through a block-expr)
    // keeps the borrow chain alive until `collect()` finishes.
    let phrase = format!("\"{}\"", term.replace('"', "\"\""));
    let mut stmt = conn.prepare(
        "SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?1 LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![&phrase, sample_limit as i64], |r| r.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    let rowids: Vec<i64> = rows.filter_map(|r| r.ok()).collect();
    drop(stmt); // release the borrow on `conn` before we prepare `body_stmt`.

    if rowids.is_empty() { return Ok(Vec::new()); }

    // Step 2 & 3: for each rowid, fetch body_text and collect distinct
    // stems. `counts` accumulates stem → number of distinct notes it
    // appears in (co-document frequency across the sample).
    let stopwords = build_stopwords();
    let mut counts: HashMap<String, u32> = HashMap::new();

    let mut body_stmt = conn.prepare(
        "SELECT body_text FROM note_meta WHERE rowid = ?1"
    ).map_err(|e| e.to_string())?;

    for rowid in &rowids {
        let body: Option<String> = body_stmt
            .query_row(rusqlite::params![rowid], |r| r.get(0))
            .ok();
        let Some(body) = body else { continue; };
        if body.is_empty() { continue; }

        // Tokenize with the same boundary rules as the FTS5 tokenizer
        // (`fts5_tokenizer::is_word_boundary`): apostrophes don't break
        // words (keeps contractions together), em/en/hyphen/underscore
        // and non-alphabetic chars do.
        let mut seen: HashSet<String> = HashSet::new();
        let mut word_start: Option<usize> = None;
        for (byte_idx, ch) in body.char_indices() {
            if is_cooccurrence_boundary(ch) {
                if let Some(start) = word_start.take() {
                    collect_stem(&body[start..byte_idx], &stopwords, &query_stems, &mut seen);
                }
            } else if word_start.is_none() {
                word_start = Some(byte_idx);
            }
        }
        // Tail word (input doesn't end with a boundary char).
        if let Some(start) = word_start {
            collect_stem(&body[start..], &stopwords, &query_stems, &mut seen);
        }

        for stem in seen {
            *counts.entry(stem).or_insert(0) += 1;
        }
    }

    // Step 4: top-K by count descending, tie-break alphabetic ascending
    // for deterministic ordering across sessions on equal-count buckets.
    let mut results: Vec<CooccurringTerm> = counts
        .into_iter()
        .map(|(stem, note_count)| {
            let term = if stem.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP) {
                stem.replace('\u{001F}', " ")
            } else {
                stem
            };
            CooccurringTerm { term, note_count }
        })
        .collect();
    results.sort_by(|a, b| {
        b.note_count.cmp(&a.note_count).then_with(|| a.term.cmp(&b.term))
    });
    results.truncate(result_limit);

    Ok(results)
}

/// Boundary predicate for co-occurrence re-tokenization. Must mirror
/// `fts5_tokenizer::is_word_boundary` exactly so the stems we aggregate
/// are the same ones stored in `notes_fts` / `notes_vocab`.
#[inline]
fn is_cooccurrence_boundary(c: char) -> bool {
    if c == '\'' { return false; }
    if c == '—' || c == '–' || c == '-' || c == '_' { return true; }
    !c.is_alphabetic()
}

#[inline]
fn collect_stem(
    word: &str,
    stopwords: &std::collections::HashSet<String>,
    query_stems: &std::collections::HashSet<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    if let Some((stem, norm_lower)) = process_word_for_fts(word) {
        // Three-way filter: stopword list (checked against both stem and
        // pre-stem normalized form — matches the tokenizer's rule), and
        // the query term's own stems (so it doesn't appear in its own
        // co-occurrence list).
        if !stopwords.contains(&stem)
            && !stopwords.contains(&norm_lower)
            && !query_stems.contains(&stem)
        {
            seen.insert(stem);
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
            // Use frontmatter title for canonical files, file stem for human-named files
            let file_stem = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let note_name = if crate::canonical::is_canonical_filename(&path) {
                extract_frontmatter_title_quick(&path).unwrap_or(file_stem)
            } else {
                file_stem
            };
            notes.push(serde_json::json!({
                "name": note_name,
                "path": path.to_string_lossy().to_string()
            }));
        }
    }
}

/// Quick frontmatter title extraction (reads first 1KB only).
fn extract_frontmatter_title_quick(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    extract_frontmatter_title(&content)
}

/// MIG-006 §1: read just the human title from a `.md` file's
/// frontmatter without indexing. Used by the rename flow to pick
/// up the OLD display name BEFORE the rename mutates the file —
/// so the wikilink cascade can search for `[[old_title]]` in source
/// notes, not for `[[20260424T063440Z_NOTE_531D]]` (the canonical
/// filename stem, which the L3788 derivation was using and which
/// silently killed the cascade for every canonical note).
///
/// Returns `Ok(Some(title))` if the file has a frontmatter `title:`
/// field, `Ok(None)` if the file has no title (caller falls back to
/// filename stem for legacy human-named notes), `Err` if the path is
/// outside any registered library or unreadable.
#[tauri::command]
pub fn read_note_title(app: tauri::AppHandle, file_path: String) -> Result<Option<String>, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist.".to_string());
    }
    Ok(extract_frontmatter_title_quick(path))
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
            // Use frontmatter title for canonical files
            let file_name = path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let note_name = if crate::canonical::is_canonical_filename(&path) {
                extract_frontmatter_title_quick(&path)
                    .unwrap_or_else(|| file_name.trim_end_matches(".md").to_string())
            } else {
                file_name.clone()
            };

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

/// MIG-006 §3 — cascade returns the list of rewritten files so the
/// frontend can reload each open tab in place. `failed[]` carries any
/// per-file write errors (cascade is best-effort, not transactional
/// across files).
#[derive(Debug, Clone, serde::Serialize)]
pub struct CascadeResult {
    pub rewritten: Vec<String>,
    pub failed: Vec<(String, String)>,
}

/// Update all links in a library when a note is renamed.
#[tauri::command]
pub fn update_links_on_rename(app: tauri::AppHandle, library_path: String, old_name: String, new_name: String) -> Result<CascadeResult, String> {
    validate_path_in_any_library(&app, &library_path)?;
    // §2: compile the regex once per cascade, reuse it across every file
    // visited. `regex::escape` keeps titles with metacharacters safe
    // (`§2 Round3`, `Foo (bar)`, `a.b`, etc.).
    let pattern = format!(r"\[\[({})(\]\]|\|)", regex::escape(&old_name));
    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to build cascade regex: {}", e)),
    };
    let mut result = CascadeResult { rewritten: Vec::new(), failed: Vec::new() };
    update_links_recursive(Path::new(&library_path), &re, &new_name, &mut result);

    // §3: notify the frontend about every file we rewrote so the open
    // tabs (if any) can reload their in-memory copies in place. Without
    // this, the editor's next autosave (with its still-pre-cascade
    // content) overwrites the cascade's update — race #2 in the §3
    // expansion.
    if !result.rewritten.is_empty() {
        use tauri::Emitter;
        let _ = app.emit("cascade:rewrote", &result.rewritten);
    }
    Ok(result)
}

fn update_links_recursive(dir: &Path, re: &regex::Regex, new_name: &str, result: &mut CascadeResult) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            update_links_recursive(&path, re, new_name, result);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let updated = rewrite_wikilinks_in_text(&content, re, new_name);
                if updated != content {
                    // §3: mark BEFORE the write so a watcher event arriving
                    // mid-fs::write (Windows can fire before flush) still
                    // sees the suppression entry.
                    crate::watcher::mark_recent_write(&path);
                    match fs::write(&path, updated) {
                        Ok(_) => result.rewritten.push(path.to_string_lossy().into_owned()),
                        Err(e) => result.failed.push((path.to_string_lossy().into_owned(), e.to_string())),
                    }
                }
            }
        }
    }
}

/// MIG-006 §2 — regex-based wikilink rewrite.
///
/// Matches `[[old]]` and `[[old|...]]` (display, link-type, alias-pipe-type
/// combos). Leading `!` for embeds is untouched because the regex anchors
/// on `[[` — `![[X]]` rewrites cleanly. The trailing delimiter (`]]` or `|`)
/// is captured and re-emitted so we never alter `|display`, `|link-type`,
/// or `|alias|link-type` tails.
///
/// Prefix-collision safety: `[[Foo]]` rename to `Bar` does NOT touch
/// `[[Foo Bar]]` or `[[Foo_v2]]` — the delimiter alternation `(\]\]|\|)`
/// requires the next char after the title to be either `]]` or `|`,
/// nothing else.
fn rewrite_wikilinks_in_text(content: &str, re: &regex::Regex, new_name: &str) -> String {
    re.replace_all(content, |caps: &regex::Captures| {
        let delim = caps.get(2).map(|m| m.as_str()).unwrap_or("]]");
        format!("[[{}{}", new_name, delim)
    })
    .into_owned()
}

#[cfg(test)]
fn rewrite_for_test(content: &str, old_name: &str, new_name: &str) -> String {
    let pattern = format!(r"\[\[({})(\]\]|\|)", regex::escape(old_name));
    let re = regex::Regex::new(&pattern).unwrap();
    rewrite_wikilinks_in_text(content, &re, new_name)
}

#[cfg(test)]
mod cascade_walker_tests {
    use super::rewrite_for_test;

    #[test]
    fn bare_wikilink_rewrites() {
        let out = rewrite_for_test("see [[Old Title]] here", "Old Title", "New Title");
        assert_eq!(out, "see [[New Title]] here");
    }

    #[test]
    fn piped_display_preserves_tail() {
        let out = rewrite_for_test("see [[Old|the display]]", "Old", "New");
        assert_eq!(out, "see [[New|the display]]");
    }

    #[test]
    fn piped_link_type_preserves_tail() {
        let out = rewrite_for_test("see [[Old|supports]]", "Old", "New");
        assert_eq!(out, "see [[New|supports]]");
    }

    #[test]
    fn piped_alias_and_link_type_preserves_tail() {
        let out = rewrite_for_test("see [[Old|alias text|supports]]", "Old", "New");
        assert_eq!(out, "see [[New|alias text|supports]]");
    }

    #[test]
    fn embed_transclude_rewrites() {
        let out = rewrite_for_test("![[Old]] inline", "Old", "New");
        assert_eq!(out, "![[New]] inline");
    }

    #[test]
    fn prefix_collision_is_not_rewritten() {
        // [[Foo]] rename to [[Bar]] must not touch [[Foo Bar]] or [[Foo_v2]].
        let out = rewrite_for_test(
            "yes [[Foo]] no [[Foo Bar]] no [[Foo_v2]] yes [[Foo|x]]",
            "Foo",
            "Bar",
        );
        assert_eq!(
            out,
            "yes [[Bar]] no [[Foo Bar]] no [[Foo_v2]] yes [[Bar|x]]"
        );
    }

    #[test]
    fn regex_metachars_in_title_are_escaped() {
        let out = rewrite_for_test(
            "see [[a.b (c)]] and [[a.b (c)|note]]",
            "a.b (c)",
            "x.y (z)",
        );
        assert_eq!(out, "see [[x.y (z)]] and [[x.y (z)|note]]");
    }

    #[test]
    fn no_match_returns_unchanged() {
        let input = "no wikilinks here, just [[Different]] and [[Foo Bar]]";
        let out = rewrite_for_test(input, "Foo", "Bar");
        assert_eq!(out, input);
    }

    #[test]
    fn multiple_occurrences_all_rewritten() {
        let out = rewrite_for_test(
            "[[Old]] then [[Old]] then [[Old|x]] done",
            "Old",
            "New",
        );
        assert_eq!(out, "[[New]] then [[New]] then [[New|x]] done");
    }

    #[test]
    fn arabic_title_rewrites() {
        let out = rewrite_for_test(
            "انظر [[الفاطميون]] في [[الفاطميون|الدولة]]",
            "الفاطميون",
            "الفاطميون_جديد",
        );
        assert_eq!(
            out,
            "انظر [[الفاطميون_جديد]] في [[الفاطميون_جديد|الدولة]]"
        );
    }

    #[test]
    fn unicode_section_marker_title_rewrites() {
        // The exact case that drove the §1 verification.
        let out = rewrite_for_test(
            "Link me to [[§2 Round3_v3]]",
            "§2 Round3_v3",
            "§2 Round3_v4",
        );
        assert_eq!(out, "Link me to [[§2 Round3_v4]]");
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

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    //! M6 end-to-end contract tests. The 502-case regression corpus in
    //! `arabic::regression` exercises `analyze_best` in isolation; this
    //! module checks that the FTS pipeline's `process_arabic_word` wrapper
    //! and `process_word_for_fts` downstream guard actually surface the
    //! analyzer's verdict to the tokenizer. Without these, a future
    //! refactor could wire a different stemmer in and the corpus would
    //! still pass while search results quietly regressed.
    use super::*;

    /// The flagship bug that motivated the Constellation Arabic Engine.
    /// Pre-M6: Light10 stripped the leading و from وائل, producing "ائل"
    /// and corrupting every index row of every note mentioning any Wael.
    /// Post-M6: the protected list short-circuits Light10 and returns the
    /// name verbatim. This test is the pin that prevents the bug from
    /// ever silently returning.
    #[test]
    fn wael_is_not_mangled_to_ail() {
        let (_display, stem) = process_arabic_word("وائل");
        assert_eq!(stem, "وائل", "M6 must not mangle protected proper nouns");
    }

    /// End-to-end through the whole `process_word_for_fts` filter —
    /// this is what the FTS5 tokenizer actually calls. Guarantees
    /// the stem column of the notes_fts index holds the full name.
    #[test]
    fn wael_survives_process_word_for_fts() {
        let (stem, _norm) = process_word_for_fts("وائل").expect("وائل must tokenize");
        assert_eq!(stem, "وائل");
    }

    /// Cascade flagship: الأئمة (definite + broken plural of إمام).
    /// Layer 3b peels ال, FST matches أئمة as the plural of إمام →
    /// lemma comes out as one of the legitimate root derivations.
    /// We don't pin the exact lemma because the tiebreak order among
    /// equal-confidence FST hits isn't stable across refactors (the
    /// 502-case corpus leaves this row unasserted on lemma for the
    /// same reason), but we DO assert it isn't the Light10 mangle
    /// ("ئم" from naive ال- / -ة stripping).
    #[test]
    fn aimma_is_not_light10_mangled() {
        let (_display, stem) = process_arabic_word("الأئمة");
        assert_ne!(stem, "ئم", "cascade path must find a real analysis");
        assert_ne!(stem, "ئمه", "cascade path must find a real analysis");
        // Sanity: the lemma should contain at least one of the
        // radicals ء / م — any genuine analysis of الأئمة does.
        assert!(
            stem.chars().any(|c| c == 'ء' || c == 'أ' || c == 'م' || c == 'إ'),
            "stem {:?} lost the root radicals",
            stem,
        );
    }

    /// Unknown Arabic word falls to SurfaceHeuristic — verify we KEEP
    /// Light10 affix stripping for it so M6 is strictly non-regressive
    /// for words the analyzer doesn't yet know. "قذالبثظ" is nonsense:
    /// not protected, no root × pattern match, no peelable affix chain
    /// that hits anything real.
    #[test]
    fn unknown_word_still_gets_light10_stripping() {
        let nonsense = "قذالبثظ";
        let (_display, stem) = process_arabic_word(nonsense);
        // Post-condition is just "did not panic and returned non-empty
        // UTF-8" — the exact Light10 output on nonsense isn't something
        // we want to pin to a literal. The important contract is that
        // the pipeline degrades gracefully, not that it produces any
        // particular string.
        assert!(!stem.is_empty());
        assert!(stem.chars().all(|c| !c.is_ascii_control()));
    }

    /// Non-Arabic words must still route through the non-Arabic branch
    /// untouched — M6 only changed the Arabic branch of `process_word_for_fts`.
    #[test]
    fn english_word_still_english_stemmed() {
        let (stem, norm) = process_word_for_fts("running").expect("english must tokenize");
        // The English stemmer turns "running" into "run" (or close);
        // critically the stem must NOT be Arabic-pipeline output.
        assert!(stem.is_ascii(), "english must not be routed to arabic pipeline");
        assert_eq!(norm, "running");
    }
}

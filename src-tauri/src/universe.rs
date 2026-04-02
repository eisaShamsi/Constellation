// ─── Constellation Universe — Portable User-Owned Data Storage ───

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

// ─── Data Structures ───

/// Metadata stored inside each universe's .constellation/universe.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    pub name: String,
    pub created: String,
    pub version: u32,
    #[serde(default)]
    pub children: Vec<String>,
    /// Relative folder name for universe-level notes (e.g., "كون عيسى")
    #[serde(default)]
    pub notes_folder: Option<String>,
}

/// Entry in the global registry (app_data_dir/universes.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created: String,
}

/// Global registry stored in app_data_dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UniverseRegistry {
    entries: Vec<UniverseEntry>,
    active_id: Option<String>,
}

/// Tauri managed state — holds the active universe path.
pub struct UniverseState {
    pub active_path: Mutex<Option<PathBuf>>,
}

impl UniverseState {
    pub fn new() -> Self {
        Self {
            active_path: Mutex::new(None),
        }
    }
}

// ─── .constellation/ Directory Helpers ───

/// Return the .constellation/ config directory inside a universe root.
pub fn constellation_dir(universe_root: &Path) -> PathBuf {
    universe_root.join(".constellation")
}

/// Return the .constellation/ directory for the active universe.
pub fn active_constellation_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = active_universe_dir(app)?;
    Ok(constellation_dir(&root))
}

// ─── Registry Helpers ───

/// Path to the global universe registry: {app_data_dir}/universes.json
fn registry_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_dir.join("universes.json"))
}

fn load_registry(app: &tauri::AppHandle) -> UniverseRegistry {
    let path = match registry_path(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[universe] Failed to get registry path: {}", e);
            return UniverseRegistry { entries: vec![], active_id: None };
        }
    };
    if path.exists() {
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[universe] Failed to read registry ({}): {}", path.display(), e);
                return UniverseRegistry { entries: vec![], active_id: None };
            }
        };
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("[universe] Corrupt registry JSON ({}): {}", path.display(), e);
            UniverseRegistry { entries: vec![], active_id: None }
        })
    } else {
        UniverseRegistry { entries: vec![], active_id: None }
    }
}

fn save_registry(app: &tauri::AppHandle, registry: &UniverseRegistry) -> Result<(), String> {
    let path = registry_path(app)?;
    let data = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Failed to save universe registry: {}", e))
}

// ─── Active Universe Helper ───

/// Get the active universe ROOT directory path from managed state.
pub fn active_universe_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let state = app.state::<UniverseState>();
    let lock = state.active_path.lock().map_err(|e| e.to_string())?;
    lock.clone().ok_or_else(|| "No active universe set.".to_string())
}

// ─── UUID Helper ───

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random: u32 = (timestamp as u32).wrapping_mul(2654435761) ^ std::process::id();
    format!("{:x}{:04x}", timestamp, random & 0xFFFF)
}

// ─── Migration: old flat format → .constellation/ ───

/// Auto-migrate a universe from the old flat layout to .constellation/ subdirectory.
/// Called when setting an active universe.
fn migrate_to_constellation(universe_root: &Path) -> Result<(), String> {
    let cdir = constellation_dir(universe_root);
    let old_meta = universe_root.join("universe.json");

    // Only migrate if old format exists but .constellation/ does not
    if !old_meta.exists() || cdir.exists() {
        return Ok(());
    }

    eprintln!("[universe] Migrating {} to .constellation/ format", universe_root.display());

    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create .constellation/bases/: {}", e))?;

    // Move config files into .constellation/
    let files_to_move = [
        "universe.json",
        "settings.json",
        "bookmarks.json",
        "workspaces.json",
        "property-types.json",
    ];
    for file in &files_to_move {
        let src = universe_root.join(file);
        if src.exists() {
            let dest = cdir.join(file);
            fs::rename(&src, &dest)
                .map_err(|e| format!("Failed to move {}: {}", file, e))?;
        }
    }

    // Rename vaults.json → libraries.json during migration
    let old_vaults = universe_root.join("vaults.json");
    if old_vaults.exists() {
        let dest = cdir.join("libraries.json");
        fs::rename(&old_vaults, &dest)
            .map_err(|e| format!("Failed to move vaults.json → libraries.json: {}", e))?;
    }

    // Move bases/ directory contents
    let old_bases = universe_root.join("bases");
    if old_bases.is_dir() {
        let new_bases = cdir.join("bases");
        if let Ok(entries) = fs::read_dir(&old_bases) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let dest = new_bases.join(entry.file_name());
                    fs::rename(&src, &dest).ok();
                }
            }
        }
        // Remove old empty bases dir
        fs::remove_dir(&old_bases).ok();
    }

    eprintln!("[universe] Migration complete for {}", universe_root.display());
    Ok(())
}

/// Ensure the universe notes folder exists for existing universes (migration).
fn ensure_universe_notes_folder(universe_root: &Path) -> Result<(), String> {
    let cdir = constellation_dir(universe_root);
    let meta_path = cdir.join("universe.json");

    if !meta_path.exists() {
        return Ok(());
    }

    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    // ─── Migration: flatten nested universe folder ───
    // Old behavior created UniverseName/UniverseName/ (nested). New behavior uses
    // the universe root directly as the library (Obsidian-style flat).
    if let Some(ref folder_name) = meta.notes_folder {
        let nested_path = universe_root.join(folder_name);
        // If nested folder exists AND has the same name as the universe → migrate to flat
        if nested_path.is_dir() && folder_name == &meta.name {
            // Walk down the chain of same-name nesting: root/Name/Name/Name/...
            // Find the deepest level, then move everything up to root.
            let mut deepest = nested_path.clone();
            loop {
                let next = deepest.join(folder_name);
                if next.is_dir() { deepest = next; } else { break; }
            }
            eprintln!("[universe] Migrating nested folders to flat (deepest: {})", deepest.display());

            // Move contents from deepest level up to universe root
            if let Ok(entries) = fs::read_dir(&deepest) {
                for entry in entries.flatten() {
                    let src = entry.path();
                    let fname = entry.file_name();
                    // Skip if it's another same-name directory (already traversed)
                    if src.is_dir() && fname.to_string_lossy() == folder_name.as_str() { continue; }
                    let dest = universe_root.join(&fname);
                    if !dest.exists() {
                        let _ = fs::rename(&src, &dest);
                    }
                }
            }

            // Also move contents from intermediate levels (they may have files too)
            let mut level = nested_path.clone();
            while level != *universe_root {
                if let Ok(entries) = fs::read_dir(&level) {
                    for entry in entries.flatten() {
                        let src = entry.path();
                        let fname = entry.file_name();
                        if src.is_dir() && fname.to_string_lossy() == folder_name.as_str() { continue; }
                        let dest = universe_root.join(&fname);
                        if !dest.exists() {
                            let _ = fs::rename(&src, &dest);
                        }
                    }
                }
                let next = level.join(folder_name);
                if next.is_dir() { level = next; } else { break; }
            }

            // Remove empty nested folders bottom-up
            let mut cleanup = deepest.clone();
            while cleanup != *universe_root {
                let _ = fs::remove_dir(&cleanup); // only removes if empty
                if let Some(parent) = cleanup.parent() {
                    cleanup = parent.to_path_buf();
                } else { break; }
            }

            // Update metadata to flat (notes_folder = None)
            meta.notes_folder = None;
            if let Ok(json) = serde_json::to_string_pretty(&meta) {
                let _ = fs::write(&meta_path, json);
            }

            // Update libraries.json: point universe_notes library to root
            let libs_path = cdir.join("libraries.json");
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.path = universe_root.to_string_lossy().to_string();
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        let _ = fs::write(&libs_path, json);
                    }
                }
            }
            eprintln!("[universe] Migration to flat structure complete");
            return Ok(());
        }

        // Non-matching subfolder (e.g. user renamed) — leave as-is, just ensure it exists
        if !nested_path.exists() {
            fs::create_dir_all(&nested_path)
                .map_err(|e| format!("Failed to create universe notes folder: {}", e))?;
        }
        // Ensure registered as library
        let libs_path = cdir.join("libraries.json");
        let folder_path_str = nested_path.to_string_lossy().to_string();
        if libs_path.exists() {
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    if !libs.iter().any(|l| l.is_universe_notes) {
                        let mut libs = libs;
                        libs.insert(0, crate::libraries::LibraryInfo {
                            id: format!("universe_notes_{}", uuid_simple()),
                            name: meta.name.clone(),
                            path: folder_path_str,
                            is_universe_notes: true,
                        });
                        if let Ok(json) = serde_json::to_string_pretty(&libs) {
                            let _ = fs::write(&libs_path, json);
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    // notes_folder is None → universe root IS the library (flat/Obsidian-style)
    // Just ensure it's registered as a library
    let libs_path = cdir.join("libraries.json");
    let root_path_str = universe_root.to_string_lossy().to_string();
    let mut libs: Vec<crate::libraries::LibraryInfo> = if libs_path.exists() {
        fs::read_to_string(&libs_path)
            .ok()
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };

    if !libs.iter().any(|l| l.is_universe_notes) {
        libs.insert(0, crate::libraries::LibraryInfo {
            id: format!("universe_notes_{}", uuid_simple()),
            name: meta.name.clone(),
            path: root_path_str,
            is_universe_notes: true,
        });
        if let Ok(json) = serde_json::to_string_pretty(&libs) {
            let _ = fs::write(&libs_path, json);
        }
    }

    Ok(())
}

// ─── Library Resolution (Universe of Universes) ───

/// Recursively resolve all libraries accessible from a universe directory.
/// Collects own libraries + child universe libraries, deduplicated by path.
fn resolve_libraries_recursive(universe_path: &Path, visited: &mut Vec<PathBuf>) -> Vec<crate::libraries::LibraryInfo> {
    // Prevent circular references
    if let Ok(canon) = fs::canonicalize(universe_path) {
        if visited.contains(&canon) {
            return vec![];
        }
        visited.push(canon);
    }

    let mut all_libraries: Vec<crate::libraries::LibraryInfo> = Vec::new();
    let cdir = constellation_dir(universe_path);

    // 1. Load own libraries from .constellation/libraries.json
    let libs_path = cdir.join("libraries.json");
    if libs_path.exists() {
        if let Ok(data) = fs::read_to_string(&libs_path) {
            if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&data) {
                all_libraries.extend(libs);
            }
        }
    } else {
        // Fallback: try old flat format (vaults.json at root)
        let old_path = universe_path.join("vaults.json");
        if old_path.exists() {
            if let Ok(data) = fs::read_to_string(&old_path) {
                if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&data) {
                    all_libraries.extend(libs);
                }
            }
        }
    }

    // 2. Load children from .constellation/universe.json and recurse
    let meta_path = cdir.join("universe.json");
    let meta_path = if meta_path.exists() { meta_path } else { universe_path.join("universe.json") };
    if meta_path.exists() {
        if let Ok(data) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<UniverseMeta>(&data) {
                for child_path_str in &meta.children {
                    let child_canon = match fs::canonicalize(child_path_str) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    if child_canon.is_dir() {
                        let child_libs = resolve_libraries_recursive(&child_canon, visited);
                        all_libraries.extend(child_libs);
                    }
                }
            }
        }
    }

    // 3. Deduplicate by path
    let mut seen = std::collections::HashSet::new();
    all_libraries.retain(|v| seen.insert(v.path.clone()));

    all_libraries
}

// ─── Tauri Commands ───

/// List all known universes from the registry.
#[tauri::command]
pub fn list_universes(app: tauri::AppHandle) -> Vec<UniverseEntry> {
    load_registry(&app).entries
}

/// Create a new universe: .constellation/ directory structure + config files.
#[tauri::command]
pub fn create_universe(
    app: tauri::AppHandle,
    name: String,
    path: String,
) -> Result<UniverseEntry, String> {
    let universe_dir = Path::new(&path).join(&name);

    if universe_dir.exists() {
        return Err("A directory with this name already exists at the chosen location.".to_string());
    }

    // Create directory structure
    let cdir = constellation_dir(&universe_dir);
    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/ directory: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;
    fs::create_dir_all(cdir.join("templates"))
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;

    // Universe root IS the notes folder (Obsidian-style flat structure).
    // No nested subfolder — notes go directly in the universe root.

    // Write universe.json into .constellation/
    let now = chrono::Local::now().to_rfc3339();
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 2,
        children: vec![],
        notes_folder: None, // None = universe root is the library (flat)
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(cdir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Register universe root as the notes library
    let notes_library = crate::libraries::LibraryInfo {
        id: format!("universe_notes_{}", uuid_simple()),
        name: name.clone(),
        path: universe_dir.to_string_lossy().to_string(), // root, not nested
        is_universe_notes: true,
    };
    let libraries_json = serde_json::to_string_pretty(&vec![&notes_library]).map_err(|e| e.to_string())?;
    fs::write(cdir.join("libraries.json"), &libraries_json)
        .map_err(|e| format!("Failed to write libraries.json: {}", e))?;
    fs::write(cdir.join("bookmarks.json"), "[]")
        .map_err(|e| format!("Failed to write bookmarks.json: {}", e))?;
    fs::write(cdir.join("settings.json"), "{}")
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;
    fs::write(cdir.join("workspaces.json"), "[]")
        .map_err(|e| format!("Failed to write workspaces.json: {}", e))?;
    fs::write(cdir.join("property-types.json"), "{}")
        .map_err(|e| format!("Failed to write property-types.json: {}", e))?;

    // Add to registry (path = universe ROOT, not .constellation/)
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name: name.clone(),
        path: universe_dir.to_string_lossy().to_string(),
        created: now,
    };

    let mut registry = load_registry(&app);
    registry.entries.push(entry.clone());
    if registry.active_id.is_none() {
        registry.active_id = Some(entry.id.clone());
    }
    save_registry(&app, &registry)?;

    Ok(entry)
}

/// Set the active universe by ID. Auto-migrates old format if needed.
#[tauri::command]
pub fn set_active_universe(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut registry = load_registry(&app);

    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| "Universe not found in registry.".to_string())?;

    let universe_path = PathBuf::from(&entry.path);
    if !universe_path.is_dir() {
        return Err(format!("Universe directory does not exist: {}", entry.path));
    }

    // Auto-migrate old flat format to .constellation/
    migrate_to_constellation(&universe_path)?;

    // Ensure universe notes folder exists (migration for existing universes)
    ensure_universe_notes_folder(&universe_path)?;

    // ─── Migration: consolidate same-name parent nesting ───
    // If universe is at C:\Name\Name\ and parent is C:\Name\, move everything up
    // and update the registry to point to C:\Name\.
    let mut final_path = universe_path.clone();
    if let (Some(parent), Some(dir_name)) = (universe_path.parent(), universe_path.file_name()) {
        let parent_name = parent.file_name().map(|n| n.to_string_lossy().to_string());
        let this_name = dir_name.to_string_lossy().to_string();
        if parent_name.as_deref() == Some(&this_name) && parent.join(".constellation").exists() == false {
            // Parent has same name and no .constellation of its own → consolidate
            eprintln!("[universe] Consolidating nested universe: {} → {}", universe_path.display(), parent.display());
            // Move .constellation/ and all contents up to parent
            if let Ok(entries) = fs::read_dir(&universe_path) {
                for entry in entries.flatten() {
                    let src = entry.path();
                    let dest = parent.join(entry.file_name());
                    if !dest.exists() {
                        let _ = fs::rename(&src, &dest);
                    }
                }
            }
            // Remove the now-empty nested directory
            let _ = fs::remove_dir(&universe_path);

            // Update registry path
            let parent_str = parent.to_string_lossy().to_string();
            for e in &mut registry.entries {
                if e.id == id {
                    e.path = parent_str.clone();
                }
            }
            // Update library paths in .constellation/libraries.json
            let cdir = constellation_dir(parent);
            let libs_path = cdir.join("libraries.json");
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.path = parent_str.clone();
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        let _ = fs::write(&libs_path, json);
                    }
                }
            }
            final_path = parent.to_path_buf();
            eprintln!("[universe] Consolidation complete: {}", final_path.display());
        }
    }

    // Update managed state
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(final_path);

    // Update registry
    registry.active_id = Some(id);
    save_registry(&app, &registry)?;

    Ok(())
}

/// Get the current active universe path.
#[tauri::command]
pub fn get_active_universe_path(app: tauri::AppHandle) -> Option<String> {
    let state = app.state::<UniverseState>();
    let lock = state.active_path.lock().ok()?;
    lock.as_ref().map(|p| p.to_string_lossy().to_string())
}

/// Remove a universe from the registry (does NOT delete files).
#[tauri::command]
pub fn remove_universe_from_registry(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut registry = load_registry(&app);
    registry.entries.retain(|e| e.id != id);
    if registry.active_id.as_deref() == Some(&id) {
        registry.active_id = registry.entries.first().map(|e| e.id.clone());
    }
    save_registry(&app, &registry)
}

/// Rename the active universe — updates registry, universe.json, notes folder, and library entry.
#[tauri::command]
pub fn rename_universe(app: tauri::AppHandle, new_name: String) -> Result<(), String> {
    let universe_dir = active_universe_dir(&app)?;
    let cdir = constellation_dir(&universe_dir);

    // 1. Read current universe.json
    let meta_path = cdir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    let old_name = meta.name.clone();

    // 2. Rename the notes folder on disk (only if a subfolder exists — legacy nested layout)
    if let Some(ref old_folder) = meta.notes_folder {
        let old_path = universe_dir.join(old_folder);
        let new_path = universe_dir.join(&new_name);
        if old_path.exists() && !new_path.exists() {
            fs::rename(&old_path, &new_path)
                .map_err(|e| format!("Failed to rename notes folder: {}", e))?;
        }
        // Update the library entry path
        let libs_path = cdir.join("libraries.json");
        if libs_path.exists() {
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.name = new_name.clone();
                            lib.path = new_path.to_string_lossy().to_string();
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        let _ = fs::write(&libs_path, json);
                    }
                }
            }
        }
    } else {
        // Flat layout (notes_folder = None): just update the library name
        let libs_path = cdir.join("libraries.json");
        if libs_path.exists() {
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.name = new_name.clone();
                            // Path stays as universe root — no subfolder to rename
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        let _ = fs::write(&libs_path, json);
                    }
                }
            }
        }
    }

    // 3. Update universe.json (preserve notes_folder — None for flat, Some for legacy)
    meta.name = new_name.clone();
    // Don't overwrite notes_folder — keep None if flat, keep old value if legacy subfolder
    if meta.notes_folder.is_some() {
        meta.notes_folder = Some(new_name.clone());
    }
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(&meta_path, &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // 4. Update global registry
    let mut registry = load_registry(&app);
    for entry in &mut registry.entries {
        if entry.path == universe_dir.to_string_lossy().to_string() {
            entry.name = new_name.clone();
        }
    }
    save_registry(&app, &registry)?;

    eprintln!("[universe] Renamed universe '{}' → '{}'", old_name, new_name);
    Ok(())
}

/// Open an existing universe directory (must contain .constellation/universe.json).
#[tauri::command]
pub fn open_existing_universe(app: tauri::AppHandle, path: String) -> Result<UniverseEntry, String> {
    let universe_dir = Path::new(&path);

    if !universe_dir.is_dir() {
        return Err("Path does not exist or is not a directory.".to_string());
    }

    // Auto-migrate if old flat format detected
    migrate_to_constellation(universe_dir)?;

    // Check for .constellation/universe.json (new format) or universe.json (fallback)
    let cdir = constellation_dir(universe_dir);
    let meta_path = if cdir.join("universe.json").exists() {
        cdir.join("universe.json")
    } else if universe_dir.join("universe.json").exists() {
        universe_dir.join("universe.json")
    } else {
        return Err("This folder does not contain a .constellation/ directory. It is not a valid Constellation universe.".to_string());
    };

    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    let mut registry = load_registry(&app);

    // Check for duplicates by path
    let canon = fs::canonicalize(universe_dir)
        .unwrap_or_else(|_| universe_dir.to_path_buf());
    for existing in &registry.entries {
        let existing_canon = fs::canonicalize(&existing.path)
            .unwrap_or_else(|_| PathBuf::from(&existing.path));
        if existing_canon == canon {
            // Already registered — just activate it
            let state = app.state::<UniverseState>();
            let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
            *lock = Some(canon);
            registry.active_id = Some(existing.id.clone());
            save_registry(&app, &registry)?;
            return Ok(existing.clone());
        }
    }

    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name: meta.name.clone(),
        path: path.clone(),
        created: meta.created.clone(),
    };

    registry.entries.push(entry.clone());
    registry.active_id = Some(entry.id.clone());
    save_registry(&app, &registry)?;

    // Set managed state
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(universe_dir.to_path_buf());

    Ok(entry)
}

/// Link an existing Markdown folder as a universe.
/// Creates .constellation/ inside the folder and registers it as a single-library universe.
#[tauri::command]
pub fn link_library_as_universe(app: tauri::AppHandle, path: String) -> Result<UniverseEntry, String> {
    let library_dir = Path::new(&path);

    if !library_dir.is_dir() {
        return Err("Path does not exist or is not a directory.".to_string());
    }

    let cdir = constellation_dir(library_dir);

    // If .constellation/ already exists, treat as "open existing"
    if cdir.join("universe.json").exists() {
        return open_existing_universe(app, path);
    }

    // Create .constellation/ inside the library folder
    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/ directory: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;

    // Derive name from folder name
    let name = library_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let name = if name.is_empty() { "My Library".to_string() } else { name };

    let now = chrono::Local::now().to_rfc3339();

    // Write universe.json
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 2,
        children: vec![],
        notes_folder: None,
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(cdir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Register the folder itself as the sole library
    let lib_id = format!("library_{}", uuid_simple());
    let library_entry = crate::libraries::LibraryInfo {
        id: lib_id,
        name: name.clone(),
        path: path.clone(),
        is_universe_notes: false,
    };
    let libs_json = serde_json::to_string_pretty(&vec![library_entry]).map_err(|e| e.to_string())?;
    fs::write(cdir.join("libraries.json"), &libs_json)
        .map_err(|e| format!("Failed to write libraries.json: {}", e))?;

    // Write empty data files
    fs::write(cdir.join("bookmarks.json"), "[]").ok();
    fs::write(cdir.join("settings.json"), "{}").ok();
    fs::write(cdir.join("workspaces.json"), "[]").ok();
    fs::write(cdir.join("property-types.json"), "{}").ok();

    // Register in global registry
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name,
        path: path.clone(),
        created: now,
    };

    let mut registry = load_registry(&app);
    registry.entries.push(entry.clone());
    registry.active_id = Some(entry.id.clone());
    save_registry(&app, &registry)?;

    // Set managed state
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(library_dir.to_path_buf());

    Ok(entry)
}

/// Check if migration from legacy app_data_dir storage is needed.
#[tauri::command]
pub fn check_migration_needed(app: tauri::AppHandle) -> bool {
    let reg_path = match registry_path(&app) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let old_vaults = app
        .path()
        .app_data_dir()
        .map(|d| d.join("vaults.json"))
        .unwrap_or_default();
    !reg_path.exists() && old_vaults.exists()
}

/// Add a child universe path to the active universe's children array.
#[tauri::command]
pub fn add_child_universe(app: tauri::AppHandle, child_path: String) -> Result<(), String> {
    let cdir = active_constellation_dir(&app)?;
    let meta_path = cdir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    // Validate child path exists and has .constellation/universe.json (or old universe.json)
    let child_dir = Path::new(&child_path);
    let child_cdir = constellation_dir(child_dir);
    if !child_cdir.join("universe.json").exists() && !child_dir.join("universe.json").exists() {
        return Err("The selected path is not a valid universe.".to_string());
    }

    // Prevent adding self
    let universe_dir = active_universe_dir(&app)?;
    if let (Ok(self_canon), Ok(child_canon)) = (fs::canonicalize(&universe_dir), fs::canonicalize(child_dir)) {
        if self_canon == child_canon {
            return Err("A universe cannot be a child of itself.".to_string());
        }
    }

    if !meta.children.contains(&child_path) {
        meta.children.push(child_path);
    }

    let json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(&meta_path, json).map_err(|e| format!("Failed to save universe.json: {}", e))
}

/// Remove a child universe path from the active universe's children array.
#[tauri::command]
pub fn remove_child_universe(app: tauri::AppHandle, child_path: String) -> Result<(), String> {
    let cdir = active_constellation_dir(&app)?;
    let meta_path = cdir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    meta.children.retain(|c| c != &child_path);

    let json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(&meta_path, json).map_err(|e| format!("Failed to save universe.json: {}", e))
}

/// Return the full merged library list for the active universe
/// (own + children, recursive, deduplicated).
#[tauri::command]
pub fn resolve_universe_libraries(app: tauri::AppHandle) -> Result<Vec<crate::libraries::LibraryInfo>, String> {
    let universe_dir = active_universe_dir(&app)?;
    let mut visited = Vec::new();
    Ok(resolve_libraries_recursive(&universe_dir, &mut visited))
}

/// Info about a child universe — name, path, and how many libraries it contributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildUniverseInfo {
    pub name: String,
    pub path: String,
    pub library_count: u32,
}

/// Return info about child universes of the active universe.
#[tauri::command]
pub fn get_child_universes(app: tauri::AppHandle) -> Result<Vec<ChildUniverseInfo>, String> {
    let cdir = active_constellation_dir(&app)?;
    let meta_path = cdir.join("universe.json");

    if !meta_path.exists() {
        return Ok(vec![]);
    }

    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    let mut children = Vec::new();
    for child_path_str in &meta.children {
        let child_path = Path::new(child_path_str);
        let child_cdir = constellation_dir(child_path);

        // Try .constellation/universe.json first, then old flat format
        let child_meta_path = if child_cdir.join("universe.json").exists() {
            child_cdir.join("universe.json")
        } else {
            child_path.join("universe.json")
        };

        let name = if child_meta_path.exists() {
            if let Ok(child_data) = fs::read_to_string(&child_meta_path) {
                if let Ok(child_meta) = serde_json::from_str::<UniverseMeta>(&child_data) {
                    child_meta.name
                } else {
                    child_path.file_name().unwrap_or_default().to_string_lossy().to_string()
                }
            } else {
                child_path.file_name().unwrap_or_default().to_string_lossy().to_string()
            }
        } else {
            child_path.file_name().unwrap_or_default().to_string_lossy().to_string()
        };

        // Count libraries in child
        let libs_path = if child_cdir.join("libraries.json").exists() {
            child_cdir.join("libraries.json")
        } else {
            child_path.join("vaults.json")
        };
        let library_count = if libs_path.exists() {
            if let Ok(vdata) = fs::read_to_string(&libs_path) {
                serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&vdata)
                    .map(|v| v.len() as u32)
                    .unwrap_or(0)
            } else { 0 }
        } else { 0 };

        children.push(ChildUniverseInfo {
            name,
            path: child_path_str.clone(),
            library_count,
        });
    }

    Ok(children)
}

/// Read library list from a child universe path (reads its .constellation/libraries.json).
#[tauri::command]
pub fn read_child_universe_libraries(_app: tauri::AppHandle, child_path: String) -> Result<Vec<crate::libraries::LibraryInfo>, String> {
    let cp = Path::new(&child_path);
    let cdir = constellation_dir(cp);

    let libs_path = if cdir.join("libraries.json").exists() {
        cdir.join("libraries.json")
    } else {
        cp.join("vaults.json")
    };

    if !libs_path.exists() {
        return Ok(vec![]);
    }

    let data = fs::read_to_string(&libs_path)
        .map_err(|e| format!("Failed to read libraries.json: {}", e))?;
    let libs: Vec<crate::libraries::LibraryInfo> = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse libraries.json: {}", e))?;
    Ok(libs)
}

// ─── Data File I/O Commands ───
// All data files live inside .constellation/

/// Read settings.json from the active universe.
#[tauri::command]
pub fn read_universe_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_constellation_dir(&app)?;
    let path = dir.join("settings.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse settings: {}", e))
    } else {
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    }
}

/// Save settings.json to the active universe.
#[tauri::command]
pub fn save_universe_settings(app: tauri::AppHandle, settings: serde_json::Value) -> Result<(), String> {
    let dir = active_constellation_dir(&app)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(dir.join("settings.json"), json)
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Read bookmarks.json from the active universe.
#[tauri::command]
pub fn read_universe_bookmarks(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_constellation_dir(&app)?;
    let path = dir.join("bookmarks.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read bookmarks: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse bookmarks: {}", e))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}

/// Save bookmarks.json to the active universe.
#[tauri::command]
pub fn save_universe_bookmarks(app: tauri::AppHandle, bookmarks: serde_json::Value) -> Result<(), String> {
    let dir = active_constellation_dir(&app)?;
    let json = serde_json::to_string_pretty(&bookmarks).map_err(|e| e.to_string())?;
    fs::write(dir.join("bookmarks.json"), json)
        .map_err(|e| format!("Failed to save bookmarks: {}", e))
}

/// Read workspaces.json from the active universe.
#[tauri::command]
pub fn read_universe_workspaces(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_constellation_dir(&app)?;
    let path = dir.join("workspaces.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read workspaces: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse workspaces: {}", e))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}

/// Save workspaces.json to the active universe.
#[tauri::command]
pub fn save_universe_workspaces(app: tauri::AppHandle, workspaces: serde_json::Value) -> Result<(), String> {
    let dir = active_constellation_dir(&app)?;
    let json = serde_json::to_string_pretty(&workspaces).map_err(|e| e.to_string())?;
    fs::write(dir.join("workspaces.json"), json)
        .map_err(|e| format!("Failed to save workspaces: {}", e))
}

/// Read property-types.json from the active universe.
#[tauri::command]
pub fn read_universe_property_types(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_constellation_dir(&app)?;
    let path = dir.join("property-types.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read property types: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse property types: {}", e))
    } else {
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    }
}

/// Save property-types.json to the active universe.
#[tauri::command]
pub fn save_universe_property_types(app: tauri::AppHandle, types: serde_json::Value) -> Result<(), String> {
    let dir = active_constellation_dir(&app)?;
    let json = serde_json::to_string_pretty(&types).map_err(|e| e.to_string())?;
    fs::write(dir.join("property-types.json"), json)
        .map_err(|e| format!("Failed to save property types: {}", e))
}

// ─── Legacy Migration ───

/// Migrate legacy data from app_data_dir to a new universe directory.
#[tauri::command]
pub fn migrate_legacy_data(app: tauri::AppHandle, name: String, universe_path: String) -> Result<UniverseEntry, String> {
    let universe_dir = PathBuf::from(&universe_path);
    let app_dir = app.path().app_data_dir()
        .map_err(|_| "Failed to get app data dir.".to_string())?;

    // Create universe directory structure with .constellation/
    let cdir = constellation_dir(&universe_dir);
    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/ directory: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;

    // Copy vaults.json → .constellation/libraries.json
    let old_vaults = app_dir.join("vaults.json");
    if old_vaults.exists() {
        fs::copy(&old_vaults, cdir.join("libraries.json"))
            .map_err(|e| format!("Failed to copy vaults.json: {}", e))?;
    } else {
        fs::write(cdir.join("libraries.json"), "[]").ok();
    }

    // Copy bases directory contents
    let old_bases = app_dir.join("bases");
    if old_bases.is_dir() {
        let target_bases = cdir.join("bases");
        if let Ok(entries) = fs::read_dir(&old_bases) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let dest = target_bases.join(entry.file_name());
                    fs::copy(&src, &dest).ok();
                }
            }
        }
    }

    // Write universe.json
    let now = chrono::Local::now().to_rfc3339();
    let name = if name.trim().is_empty() { "My Universe".to_string() } else { name };
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 2,
        children: vec![],
        notes_folder: None,
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(cdir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Write empty data files
    if !cdir.join("bookmarks.json").exists() {
        fs::write(cdir.join("bookmarks.json"), "[]").ok();
    }
    if !cdir.join("settings.json").exists() {
        fs::write(cdir.join("settings.json"), "{}").ok();
    }
    if !cdir.join("workspaces.json").exists() {
        fs::write(cdir.join("workspaces.json"), "[]").ok();
    }
    if !cdir.join("property-types.json").exists() {
        fs::write(cdir.join("property-types.json"), "{}").ok();
    }

    // Create registry with this universe
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name,
        path: universe_dir.to_string_lossy().to_string(),
        created: now,
    };

    let registry = UniverseRegistry {
        entries: vec![entry.clone()],
        active_id: Some(entry.id.clone()),
    };
    save_registry(&app, &registry)?;

    // Set as active
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(universe_dir);

    Ok(entry)
}

/// Scaffold a starter PKM structure in a library folder.
/// Creates Atlas/, Calendar/, Efforts/, + (inbox), and a Welcome.md note.
#[tauri::command]
pub fn scaffold_starter_library(library_path: String) -> Result<(), String> {
    let root = Path::new(&library_path);
    if !root.exists() {
        return Err("Library path does not exist.".to_string());
    }

    let folders = ["Atlas", "Calendar", "Efforts", "+"];
    for folder in &folders {
        let dir = root.join(folder);
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create {}: {}", folder, e))?;
    }

    let welcome_path = root.join("Welcome.md");
    if !welcome_path.exists() {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let content = format!(
            "---\ncreated: {}\nstatus: seedling\n---\n\n# Welcome to Constellation\n\nYour knowledge universe is ready. Here's a quick guide to get started:\n\n## Folder Structure\n\n- **Atlas** — Maps of Content, dashboards, and indexes\n- **Calendar** — Daily notes and time-based entries\n- **Efforts** — Active projects and tasks\n- **+** — Quick capture inbox (Ctrl+Shift+N)\n\n## Tips\n\n- Use `[[wikilinks]]` to connect your notes\n- Press `Ctrl+N` to create a new note\n- Press `Ctrl+Shift+N` to quick-capture into your inbox\n- Open the Star View to see your knowledge network\n\nHappy exploring!\n",
            now
        );
        fs::write(&welcome_path, &content)
            .map_err(|e| format!("Failed to write Welcome.md: {}", e))?;
    }

    Ok(())
}

// ─── Template Commands ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub name: String,
    pub path: String,
}

/// Get the path to the universe-level templates directory.
#[tauri::command]
pub fn get_templates_dir(app: tauri::AppHandle) -> Result<String, String> {
    let cdir = active_constellation_dir(&app)?;
    let templates_dir = cdir.join("templates");
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;
    Ok(templates_dir.to_string_lossy().to_string())
}

/// List all .md template files in the universe templates directory.
#[tauri::command]
pub fn list_templates(app: tauri::AppHandle) -> Result<Vec<TemplateEntry>, String> {
    let cdir = active_constellation_dir(&app)?;
    let templates_dir = cdir.join("templates");
    if !templates_dir.exists() {
        return Ok(vec![]);
    }
    let mut templates = Vec::new();
    collect_templates_recursive(&templates_dir, &mut templates);
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

fn collect_templates_recursive(dir: &Path, templates: &mut Vec<TemplateEntry>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_templates_recursive(&path, templates);
        } else if path.extension().map_or(false, |ext| ext == "md") {
            let name = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            templates.push(TemplateEntry {
                name,
                path: path.to_string_lossy().to_string(),
            });
        }
    }
}

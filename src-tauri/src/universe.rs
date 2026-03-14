// ─── Constellation Universe — Portable User-Owned Data Storage ───

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

// ─── Data Structures ───

/// Metadata stored inside each universe directory as universe.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    pub name: String,
    pub created: String,
    pub version: u32,
    #[serde(default)]
    pub children: Vec<String>,
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

/// Get the active universe directory path from managed state.
/// This is the central function that replaces all app_data_dir usage.
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
    // Add random component to avoid collisions on low-resolution clocks (Windows ~100ns)
    let random: u32 = (timestamp as u32).wrapping_mul(2654435761) ^ std::process::id();
    format!("{:x}{:04x}", timestamp, random & 0xFFFF)
}

// ─── Vault Resolution (Universe of Universes) ───

/// Recursively resolve all vaults accessible from a universe directory.
/// Collects own vaults + child universe vaults, deduplicated by path.
fn resolve_vaults_recursive(universe_path: &Path, visited: &mut Vec<PathBuf>) -> Vec<crate::vaults::VaultInfo> {
    // Prevent circular references
    if let Ok(canon) = fs::canonicalize(universe_path) {
        if visited.contains(&canon) {
            return vec![];
        }
        visited.push(canon);
    }

    let mut all_vaults: Vec<crate::vaults::VaultInfo> = Vec::new();

    // 1. Load own vaults from vaults.json
    let vaults_path = universe_path.join("vaults.json");
    if vaults_path.exists() {
        if let Ok(data) = fs::read_to_string(&vaults_path) {
            if let Ok(vaults) = serde_json::from_str::<Vec<crate::vaults::VaultInfo>>(&data) {
                all_vaults.extend(vaults);
            }
        }
    }

    // 2. Load children from universe.json and recurse
    let meta_path = universe_path.join("universe.json");
    if meta_path.exists() {
        if let Ok(data) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<UniverseMeta>(&data) {
                for child_path_str in &meta.children {
                    // Canonicalize before use to resolve any ".." or symlink components
                    let child_canon = match fs::canonicalize(child_path_str) {
                        Ok(p) => p,
                        Err(_) => continue, // Path doesn't exist or is invalid — skip
                    };
                    if child_canon.is_dir() {
                        let child_vaults = resolve_vaults_recursive(&child_canon, visited);
                        all_vaults.extend(child_vaults);
                    }
                }
            }
        }
    }

    // 3. Deduplicate by path
    let mut seen = std::collections::HashSet::new();
    all_vaults.retain(|v| seen.insert(v.path.clone()));

    all_vaults
}

// ─── Tauri Commands ───

/// List all known universes from the registry.
#[tauri::command]
pub fn list_universes(app: tauri::AppHandle) -> Vec<UniverseEntry> {
    load_registry(&app).entries
}

/// Create a new universe: directory structure + universe.json + empty data files.
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
    fs::create_dir_all(&universe_dir)
        .map_err(|e| format!("Failed to create universe directory: {}", e))?;
    fs::create_dir_all(universe_dir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;

    // Write universe.json
    let now = chrono::Local::now().to_rfc3339();
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 1,
        children: vec![],
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(universe_dir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Write empty data files
    fs::write(universe_dir.join("vaults.json"), "[]")
        .map_err(|e| format!("Failed to write vaults.json: {}", e))?;
    fs::write(universe_dir.join("bookmarks.json"), "[]")
        .map_err(|e| format!("Failed to write bookmarks.json: {}", e))?;
    fs::write(universe_dir.join("settings.json"), "{}")
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;
    fs::write(universe_dir.join("workspaces.json"), "[]")
        .map_err(|e| format!("Failed to write workspaces.json: {}", e))?;
    fs::write(universe_dir.join("property-types.json"), "{}")
        .map_err(|e| format!("Failed to write property-types.json: {}", e))?;

    // Add to registry
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name: name.clone(),
        path: universe_dir.to_string_lossy().to_string(),
        created: now,
    };

    let mut registry = load_registry(&app);
    registry.entries.push(entry.clone());
    // If this is the first universe, make it active
    if registry.active_id.is_none() {
        registry.active_id = Some(entry.id.clone());
    }
    save_registry(&app, &registry)?;

    Ok(entry)
}

/// Set the active universe by ID. Updates both the registry and managed state.
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

    // Update managed state
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(universe_path);

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

/// Open an existing universe directory (must contain universe.json).
/// Reads its metadata, registers it, and sets it as active.
#[tauri::command]
pub fn open_existing_universe(app: tauri::AppHandle, path: String) -> Result<UniverseEntry, String> {
    let universe_dir = Path::new(&path);

    if !universe_dir.is_dir() {
        return Err("Path does not exist or is not a directory.".to_string());
    }

    let meta_path = universe_dir.join("universe.json");
    if !meta_path.exists() {
        return Err("This folder does not contain a universe.json file. It is not a valid Constellation universe.".to_string());
    }

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
    let universe_dir = active_universe_dir(&app)?;
    let meta_path = universe_dir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    // Validate child path exists and has a universe.json
    let child_dir = Path::new(&child_path);
    if !child_dir.join("universe.json").exists() {
        return Err("The selected path is not a valid universe.".to_string());
    }

    // Prevent adding self
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
    let universe_dir = active_universe_dir(&app)?;
    let meta_path = universe_dir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    meta.children.retain(|c| c != &child_path);

    let json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(&meta_path, json).map_err(|e| format!("Failed to save universe.json: {}", e))
}

/// Return the full merged vault list for the active universe
/// (own + children, recursive, deduplicated).
#[tauri::command]
pub fn resolve_universe_vaults(app: tauri::AppHandle) -> Result<Vec<crate::vaults::VaultInfo>, String> {
    let universe_dir = active_universe_dir(&app)?;
    let mut visited = Vec::new();
    Ok(resolve_vaults_recursive(&universe_dir, &mut visited))
}

/// Info about a child universe — name, path, and how many vaults it contributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildUniverseInfo {
    pub name: String,
    pub path: String,
    pub vault_count: u32,
}

/// Return info about child universes of the active universe.
#[tauri::command]
pub fn get_child_universes(app: tauri::AppHandle) -> Result<Vec<ChildUniverseInfo>, String> {
    let universe_dir = active_universe_dir(&app)?;
    let meta_path = universe_dir.join("universe.json");

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
        let child_meta_path = child_path.join("universe.json");

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

        // Count vaults in child (non-recursive for display)
        let vaults_path = child_path.join("vaults.json");
        let vault_count = if vaults_path.exists() {
            if let Ok(vdata) = fs::read_to_string(&vaults_path) {
                serde_json::from_str::<Vec<crate::vaults::VaultInfo>>(&vdata)
                    .map(|v| v.len() as u32)
                    .unwrap_or(0)
            } else { 0 }
        } else { 0 };

        children.push(ChildUniverseInfo {
            name,
            path: child_path_str.clone(),
            vault_count,
        });
    }

    Ok(children)
}

// ─── Data File I/O Commands ───

/// Read settings.json from the active universe.
#[tauri::command]
pub fn read_universe_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_universe_dir(&app)?;
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
    let dir = active_universe_dir(&app)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(dir.join("settings.json"), json)
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Read bookmarks.json from the active universe.
#[tauri::command]
pub fn read_universe_bookmarks(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_universe_dir(&app)?;
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
    let dir = active_universe_dir(&app)?;
    let json = serde_json::to_string_pretty(&bookmarks).map_err(|e| e.to_string())?;
    fs::write(dir.join("bookmarks.json"), json)
        .map_err(|e| format!("Failed to save bookmarks: {}", e))
}

/// Read workspaces.json from the active universe.
#[tauri::command]
pub fn read_universe_workspaces(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_universe_dir(&app)?;
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
    let dir = active_universe_dir(&app)?;
    let json = serde_json::to_string_pretty(&workspaces).map_err(|e| e.to_string())?;
    fs::write(dir.join("workspaces.json"), json)
        .map_err(|e| format!("Failed to save workspaces: {}", e))
}

/// Read property-types.json from the active universe.
#[tauri::command]
pub fn read_universe_property_types(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_universe_dir(&app)?;
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
    let dir = active_universe_dir(&app)?;
    let json = serde_json::to_string_pretty(&types).map_err(|e| e.to_string())?;
    fs::write(dir.join("property-types.json"), json)
        .map_err(|e| format!("Failed to save property types: {}", e))
}

// ─── Migration ───

/// Migrate legacy data from app_data_dir to a new universe directory.
#[tauri::command]
pub fn migrate_legacy_data(app: tauri::AppHandle, name: String, universe_path: String) -> Result<UniverseEntry, String> {
    let universe_dir = PathBuf::from(&universe_path);
    let app_dir = app.path().app_data_dir()
        .map_err(|_| "Failed to get app data dir.".to_string())?;

    // Create universe directory structure
    fs::create_dir_all(&universe_dir)
        .map_err(|e| format!("Failed to create universe directory: {}", e))?;
    fs::create_dir_all(universe_dir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;

    // Copy vaults.json
    let old_vaults = app_dir.join("vaults.json");
    if old_vaults.exists() {
        fs::copy(&old_vaults, universe_dir.join("vaults.json"))
            .map_err(|e| format!("Failed to copy vaults.json: {}", e))?;
    } else {
        fs::write(universe_dir.join("vaults.json"), "[]").ok();
    }

    // Copy bases directory contents
    let old_bases = app_dir.join("bases");
    if old_bases.is_dir() {
        let target_bases = universe_dir.join("bases");
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
        version: 1,
        children: vec![],
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(universe_dir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Write empty data files (will be populated by frontend migration)
    if !universe_dir.join("bookmarks.json").exists() {
        fs::write(universe_dir.join("bookmarks.json"), "[]").ok();
    }
    if !universe_dir.join("settings.json").exists() {
        fs::write(universe_dir.join("settings.json"), "{}").ok();
    }
    if !universe_dir.join("workspaces.json").exists() {
        fs::write(universe_dir.join("workspaces.json"), "[]").ok();
    }
    if !universe_dir.join("property-types.json").exists() {
        fs::write(universe_dir.join("property-types.json"), "{}").ok();
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

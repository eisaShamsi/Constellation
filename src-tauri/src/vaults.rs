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

//! MIG-069 — Style Presets: app-GLOBAL, named bundles of style configuration.
//!
//! Stored at `{app_data_dir}/style-presets.json` (a sibling of the universe
//! registry, NOT inside any universe), so presets are reusable across every
//! universe. Rust is a dumb, robust JSON store here: the preset shape (which
//! sections each preset carries) is owned entirely by the frontend
//! (`stylePresets.ts`) — we persist and return the array verbatim, with a
//! graceful empty fallback on a missing/corrupt file (mirrors the universe
//! registry in `universe.rs`).

use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// Path to the global style-presets file: `{app_data_dir}/style-presets.json`.
fn presets_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_dir.join("style-presets.json"))
}

/// Load all style presets (the frontend-shaped array). Returns `[]` if the file is
/// missing or corrupt — a bad presets file must never block startup or the Settings UI.
#[tauri::command]
pub fn load_style_presets(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let path = presets_path(&app)?;
    if !path.exists() {
        return Ok(serde_json::json!([]));
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read style presets ({}): {}", path.display(), e))?;
    Ok(serde_json::from_str(&data).unwrap_or_else(|e| {
        eprintln!(
            "[style_presets] Corrupt style-presets.json ({}): {}",
            path.display(),
            e
        );
        serde_json::json!([])
    }))
}

/// Persist the full presets array (pretty-printed). The frontend owns the shape.
#[tauri::command]
pub fn save_style_presets(app: tauri::AppHandle, presets: serde_json::Value) -> Result<(), String> {
    let path = presets_path(&app)?;
    let data = serde_json::to_string_pretty(&presets).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Failed to save style presets: {}", e))
}

// ─── Export / import to a shareable .json file (MIG-069 §D) ───

/// Export ONE preset to a user-chosen `.json` file (the share story). Returns true if
/// saved, false if the user cancelled the dialog.
#[tauri::command]
pub async fn export_style_preset(
    preset: serde_json::Value,
    suggested_name: String,
) -> Result<bool, String> {
    let stem = if suggested_name.trim().is_empty() { "style".to_string() } else { suggested_name };
    let file = rfd::AsyncFileDialog::new()
        .set_title("Export Style")
        .set_file_name(format!("{stem}.constellation-style.json"))
        .add_filter("Constellation Style", &["json"])
        .save_file()
        .await;
    match file {
        Some(f) => {
            let data = serde_json::to_string_pretty(&preset).map_err(|e| e.to_string())?;
            fs::write(f.path(), data).map_err(|e| format!("Failed to write style file: {}", e))?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Import a preset from a user-chosen `.json` file. Returns the parsed JSON (the frontend
/// validates the shape), or `null` if the user cancelled.
#[tauri::command]
pub async fn import_style_preset() -> Result<serde_json::Value, String> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Import Style")
        .add_filter("Constellation Style", &["json"])
        .pick_file()
        .await;
    match file {
        Some(f) => {
            let data = fs::read_to_string(f.path()).map_err(|e| format!("Failed to read file: {}", e))?;
            serde_json::from_str(&data).map_err(|e| format!("Not a valid style file: {}", e))
        }
        None => Ok(serde_json::Value::Null),
    }
}

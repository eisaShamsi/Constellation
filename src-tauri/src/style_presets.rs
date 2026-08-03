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
    // 2026-08-02 triage concern #1 — corrupt USED to return `[]`, and the frontend owns the
    // whole array: it would then render "no saved styles" and the next Save would write that
    // single new style over every style the user had. The read error was already propagated;
    // the PARSE error was not. Both must be, because both mean "your data is on disk and we
    // could not see it" — the one case where writing back is destruction.
    Ok(crate::universe::read_persisted_json::<serde_json::Value>(&path)?
        .unwrap_or_else(|| serde_json::json!([])))
}

/// Persist the full presets array (pretty-printed). The frontend owns the shape.
#[tauri::command]
pub fn save_style_presets(app: tauri::AppHandle, presets: serde_json::Value) -> Result<(), String> {
    let path = presets_path(&app)?;
    let data = serde_json::to_string_pretty(&presets).map_err(|e| e.to_string())?;
    // 2026-08-03 inspection — this was a plain truncate-then-write, the last persisted-state
    // save that never got `atomic_write`. An interruption mid-write leaves the file partial,
    // and the loader above (now strict) reads that as corrupt — so the user is told their saved
    // styles are unreadable, by the very save that was meant to add one.
    //
    // Making the READ strict without making the WRITE atomic just relocates the failure. Same
    // half-a-sweep shape as the rest of this session; both halves belong together.
    crate::universe::atomic_write(&path, data.as_bytes())
        .map_err(|e| format!("Failed to save style presets: {}", e))
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

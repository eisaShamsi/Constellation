//! PJ-229 — **app-global preferences that survive a restart.**
//!
//! Stored at `{app_data_dir}/app-prefs.json`, a sibling of the universe registry and of
//! `style-presets.json` — deliberately NOT inside any universe.
//!
//! Its first tenant is the interface language. That lived only in the WebView's
//! `localStorage`, which this project has already proved non-durable (PJ-110, the leveldb
//! orphan-wipe), and on 2026-08-08 the Boss closed Constellation in Arabic and reopened
//! it in English. Everything else he sets is written to a real file; the language was the
//! one preference riding on browser storage.
//!
//! **Why not `.constellation/settings.json`:** that file is per-UNIVERSE, so the language
//! would change when he switched universes; it does not exist on the first-run screen,
//! which has no universe yet; and its save path is latched behind a successful read, so
//! on exactly that screen the write would be refused — reproducing PJ-229 through a
//! different door.
//!
//! Rust is a dumb JSON store here, as in `style_presets`: the frontend owns the shape.
//! The two disciplines that module paid for in blood are inherited deliberately —
//! **strict read** (a corrupt file errors instead of returning `{}` that the next save
//! would flatten) and **atomic write** (a partial file is not a corrupt one).

use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn prefs_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_dir.join("app-prefs.json"))
}

/// The whole preferences object. `{}` when the file has never been written — which is
/// every install before this shipped, and is not an error.
///
/// A file that EXISTS but cannot be read or parsed IS an error: returning `{}` there
/// would tell the frontend "you have no preferences", and the next save would write that
/// emptiness over preferences that are still on disk.
#[tauri::command]
pub fn load_app_prefs(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let path = prefs_path(&app)?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    Ok(crate::universe::read_persisted_json::<serde_json::Value>(&path)?
        .unwrap_or_else(|| serde_json::json!({})))
}

/// Persist the whole preferences object. The frontend merges; this only stores.
#[tauri::command]
pub fn save_app_prefs(app: tauri::AppHandle, prefs: serde_json::Value) -> Result<(), String> {
    let path = prefs_path(&app)?;
    let data = serde_json::to_string_pretty(&prefs).map_err(|e| e.to_string())?;
    crate::universe::atomic_write(&path, data.as_bytes())
        .map_err(|e| format!("Failed to save app preferences: {}", e))
}

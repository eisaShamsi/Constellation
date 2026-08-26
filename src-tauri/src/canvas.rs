//! Sense-Making Canvas — Cognitive Engine Phase 11 (لوحة الإدراك).
//!
//! Pre-structural space for capturing ambiguous, half-formed ideas.
//! Canvas files stored as `.canvas` JSON in the library.
//!
//! Cynefin quadrants: Clear, Complicated, Complex, Chaotic
//! Items: text snippets, wikilinks, free-form text

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasData {
    #[serde(default)]
    pub items: Vec<CanvasItem>,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,    // "text" | "link"
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub content: String,
    pub quadrant: Option<String>, // "clear" | "complicated" | "complex" | "chaotic" | null
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanvasInfo {
    pub path: String,
    pub name: String,
}

/// List all .canvas files in a library.
///
/// PJ-207 §15 — `(async)`: an unbounded recursive `read_dir` with a per-entry `is_dir()` stat
/// over the whole library — which for the default `universe_notes` library is the entire
/// Universe root. On the main thread that is a visible freeze on a large universe.
#[tauri::command(async)]
pub fn list_canvases(
    app: tauri::AppHandle,
    library_path: String,
) -> Result<Vec<CanvasInfo>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let mut canvases: Vec<CanvasInfo> = Vec::new();
    scan_canvases_recursive(Path::new(&library_path), &mut canvases);
    Ok(canvases)
}

/// Read a canvas file.
#[tauri::command]
pub fn read_canvas(
    _app: tauri::AppHandle,
    canvas_path: String,
) -> Result<CanvasData, String> {
    let content = fs::read_to_string(&canvas_path)
        .map_err(|e| format!("Failed to read canvas: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse canvas: {}", e))
}

/// Write a canvas file.
#[tauri::command]
pub fn write_canvas(
    app: tauri::AppHandle,
    canvas_path: String,
    data: CanvasData,
) -> Result<(), String> {
    // PJ-207 §15 — this was a bare `fs::write`: truncate-then-write, no temp file, no fsync.
    // A `.canvas` is the SOLE copy of that board — no SQLite mirror, no `.trash` copy, no
    // backup — so an interruption mid-write (power loss, a kill, an AV or sync client) left
    // the user's only copy zero-length or half-JSON; and with no fsync even an Ok-returning
    // write could land as zeros after a crash. `read_canvas` then failed to parse, and the
    // board was gone with nothing on screen able to say why. `atomic_write` (temp + fsync +
    // rename, universe.rs) is what every other persisted-JSON writer already uses;
    // style_presets.rs called itself "the last save that never got atomic_write" on
    // 2026-08-03, and canvas.rs was simply missed by that sweep.
    //
    // The unused `_app` was the same omission wearing a different hat: alone among the
    // commands in this file, the WRITE validated nothing and would write wherever it was
    // told. A refusal here surfaces in the canvas's save-error banner; it is never silent.
    crate::libraries::validate_path_in_any_library(&app, &canvas_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    crate::universe::atomic_write(Path::new(&canvas_path), json.as_bytes())
        .map_err(|e| format!("Failed to write canvas: {}", e))
}

/// Create a new canvas file.
#[tauri::command]
pub fn create_canvas(
    app: tauri::AppHandle,
    library_path: String,
    name: String,
) -> Result<String, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let file_name = format!("{}.canvas", name.trim().replace(['<', '>', ':', '"', '/', '\\', '|', '?', '*'], "_"));
    let path = Path::new(&library_path).join(&file_name);

    if path.exists() {
        return Err("Canvas with this name already exists.".to_string());
    }

    let data = CanvasData {
        items: vec![],
        title: name,
    };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    // PJ-207 §15 — Whole-Ecosystem: the same truncate-then-write shape as `write_canvas`
    // above, on the same file kind. A create interrupted mid-write leaves a half-JSON
    // `.canvas` that `list_canvases` will happily keep listing (it matches on extension
    // only, line 118) and that `read_canvas` can never open — a canvas born unreadable.
    crate::universe::atomic_write(&path, json.as_bytes())
        .map_err(|e| format!("Failed to create canvas: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

fn scan_canvases_recursive(dir: &Path, canvases: &mut Vec<CanvasInfo>) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            // MIG-112 — a universe is never content of another universe.
            if crate::libraries::carries_universe_manifest(&path, crate::libraries::BareManifest::MustLookLikeOne) { continue; }
            scan_canvases_recursive(&path, canvases);
        } else if path.extension().and_then(|e| e.to_str()) == Some("canvas") {
            let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            canvases.push(CanvasInfo {
                path: path.to_string_lossy().to_string(),
                name,
            });
        }
    }
}

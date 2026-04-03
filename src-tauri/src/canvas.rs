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
#[tauri::command]
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
    _app: tauri::AppHandle,
    canvas_path: String,
    data: CanvasData,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(&canvas_path, json).map_err(|e| format!("Failed to write canvas: {}", e))
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
    fs::write(&path, json).map_err(|e| format!("Failed to create canvas: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

fn scan_canvases_recursive(dir: &Path, canvases: &mut Vec<CanvasInfo>) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
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

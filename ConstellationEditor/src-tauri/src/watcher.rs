use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct WatcherState {
    watchers: Mutex<HashMap<String, RecommendedWatcher>>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            watchers: Mutex::new(HashMap::new()),
        }
    }
}

#[tauri::command]
pub fn watch_library(app: AppHandle, library_id: String, library_path: String) -> Result<(), String> {
    let state = app.state::<WatcherState>();
    let mut watchers = state.watchers.lock().map_err(|e| e.to_string())?;

    // Don't watch if already watching
    if watchers.contains_key(&library_id) {
        return Ok(());
    }

    let app_clone = app.clone();
    let lid = library_id.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            // Only care about create, modify, remove, rename events
            let dominated = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            );
            if !dominated {
                return;
            }

            let changed_paths: Vec<String> = event
                .paths
                .iter()
                .filter(|p| {
                    p.is_dir()
                        || p.extension()
                            .map(|e| e == "md")
                            .unwrap_or(false)
                })
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            if !changed_paths.is_empty() {
                let _ = app_clone.emit(
                    "library-changed",
                    serde_json::json!({
                        "libraryId": lid,
                        "paths": changed_paths
                    }),
                );
            }
        }
    })
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    let path = PathBuf::from(&library_path);
    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {}", e))?;

    watchers.insert(library_id, watcher);
    Ok(())
}

#[tauri::command]
pub fn unwatch_library(app: AppHandle, library_id: String) -> Result<(), String> {
    let state = app.state::<WatcherState>();
    let mut watchers = state.watchers.lock().map_err(|e| e.to_string())?;
    watchers.remove(&library_id);
    Ok(())
}

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::watcher_suppress;

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

/// Installing a recursive filesystem watch is blocking I/O: `notify`'s Windows
/// backend calls `ReadDirectoryChangesW` and creates kernel structures for the
/// subtree. On boot the frontend fans out one `watch_library` per library (16
/// on the trial Universe) alongside ~16 `get_library_appearance`, stats, and
/// recent-files calls. Without the `(async)` attribute below, every one of
/// those `#[tauri::command]` sync bodies runs **on the WebView2 UI thread**
/// (wry-0.54.2/src/webview2/mod.rs:950 delivers IPC messages via a single
/// `WebResourceRequested` handler, serialized by COM STA; tauri-2.10.3/src/
/// webview/mod.rs:1888 then calls the command body inline). Under that model
/// each library's watch-install + appearance read + …runs back-to-back on one
/// thread, and `cache_boot_snapshot_core` — which fires in the same fan-out
/// window — has to wait ~20 s for its turn. That's Boot Criterion 2's failure
/// mode (docs/LESSONS-LEARNED.md LL-021 post-Round-3 investigation).
///
/// `#[tauri::command(async)] pub fn` tells the Tauri macro
/// (tauri-macros-2.5.5/src/command/wrapper.rs:241, "sync_threadpool" kind) to
/// route this command through `respond_async_serialized` →
/// `tauri::async_runtime::spawn` (tauri-2.10.3/src/ipc/mod.rs:375). The UI
/// thread pays only the spawn cost (microseconds) and is freed to drain the
/// next IPC message; the sync body runs on a Tokio async-runtime worker.
#[tauri::command(async)]
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
                // Watcher index-freshness (2026-07-08): also pass paths that no
                // longer exist (a removed / renamed-away file or DIRECTORY). A
                // renamed-away folder's OLD path is gone, so without this the
                // old-side signal is lost and the frontend reindex can't purge that
                // folder's stale note_meta rows. Still emit-only — the reindex runs
                // off-thread via the frontend's `reindex_changed_paths`. ONE stat
                // per path (kept cheap on the notify watch thread): gone → pass
                // (removed); existing dir → pass; existing `.md` → pass; existing
                // non-`.md` file → ignored.
                .filter(|p| match p.metadata() {
                    Err(_) => true,
                    Ok(m) => m.is_dir() || p.extension().map(|e| e == "md").unwrap_or(false),
                })
                // §3-redo.2: skip paths just written by the wikilink rename
                // cascade. Without this, the cascade's fs::write bubbles back
                // as an external edit, the frontend reloads the file, and the
                // cascade and watcher loop. See watcher_suppress.rs.
                .filter(|p| !watcher_suppress::was_recent(p))
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

/// See `watch_library` above for the rationale behind `(async)`. Dropping a
/// watcher in `notify`'s Windows backend also calls into the OS (the
/// `RecommendedWatcher` drop releases the `ReadDirectoryChangesW` handle) so
/// we treat it the same way for symmetry and to keep the UI thread unblocked
/// even if multiple libraries are closed concurrently.
#[tauri::command(async)]
pub fn unwatch_library(app: AppHandle, library_id: String) -> Result<(), String> {
    let state = app.state::<WatcherState>();
    let mut watchers = state.watchers.lock().map_err(|e| e.to_string())?;
    watchers.remove(&library_id);
    Ok(())
}

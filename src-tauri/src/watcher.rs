use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
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

/// MIG-006 §3 — Rust-side recent-writes suppression for the file watcher.
///
/// Cascades and other intentional Rust-side writes (`fs::write` from the
/// wikilink rename walker, frontmatter title rewrites, etc.) call
/// `mark_recent_write(path)` immediately before writing. The watcher
/// closure then drops any change event whose paths are all `was_recent`,
/// so our own writes don't bubble back through the `library-changed`
/// channel as "external edits" and race the editor's read-back.
///
/// TTL is 2500 ms — covers two debounced autosave cycles plus margin,
/// short enough that a genuine external edit landing on the same path
/// shortly after our write isn't permanently masked.
const RECENT_WRITE_TTL: Duration = Duration::from_millis(2500);

fn recent_writes() -> &'static Mutex<HashMap<PathBuf, Instant>> {
    static CELL: OnceLock<Mutex<HashMap<PathBuf, Instant>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Mark `path` as a self-write. The watcher will skip emit for any change
/// event on this path within the TTL window.
pub fn mark_recent_write(path: &Path) {
    let now = Instant::now();
    if let Ok(mut map) = recent_writes().lock() {
        // Opportunistic cleanup — drop entries older than 2× the TTL.
        let cutoff = now - RECENT_WRITE_TTL * 2;
        map.retain(|_, &mut t| t > cutoff);
        map.insert(path.to_path_buf(), now);
    }
}

/// Returns true if `path` was recently marked by `mark_recent_write` and
/// is still within the TTL window. Stale entries are removed on read.
fn was_recent(path: &Path) -> bool {
    let now = Instant::now();
    if let Ok(mut map) = recent_writes().lock() {
        match map.get(path).copied() {
            Some(t) if now.duration_since(t) < RECENT_WRITE_TTL => true,
            Some(_) => {
                map.remove(path);
                false
            }
            None => false,
        }
    } else {
        false
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
                .filter(|p| {
                    p.is_dir()
                        || p.extension()
                            .map(|e| e == "md")
                            .unwrap_or(false)
                })
                // §3: drop paths Rust just wrote itself (cascade, in-place
                // frontmatter rewrites). Without this, the cascade's own
                // `fs::write` round-trips back through this channel as an
                // "external edit," fighting the editor's read-back.
                .filter(|p| !was_recent(p))
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

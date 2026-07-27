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

/// True when `p` lies inside Constellation's own bookkeeping directory — i.e. any path
/// having a `.constellation` **component**, including the directory itself.
///
/// MIG-104 Slice 1. `EXCLUDED_DIRS` (`file_kinds.rs`) already names `.constellation`, but it
/// is referenced only by the importers and `canonical.rs` — never by the watcher and never by
/// `reindex_changed_paths`. This is the watcher's own gate.
///
/// Deliberately keyed on the exact segment, NOT on "starts with a dot": `.trash` carries real
/// `note_meta` rows (62 measured live) and a note restored from it must still reach the
/// indexer, so excluding all dot-dirs here would hide the user's own knowledge.
///
/// Component-wise (never a substring compare) so a legitimately-named user folder such as
/// `My .constellation notes` can never be swallowed.
fn is_app_bookkeeping_path(p: &std::path::Path) -> bool {
    p.components()
        .any(|c| c.as_os_str() == std::ffi::OsStr::new(".constellation"))
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
                // MIG-104 Slice 1 — the app's own bookkeeping folder must never look like
                // the user's knowledge changing. FIRST in the chain, because it is the only
                // filter that can reject the two shapes the checks below deliberately PASS:
                // a bare directory event (`m.is_dir()` → pass) and any vanished path
                // (`Err(_)` → pass). The Universe root IS a registered library and the watch
                // is Recursive, so every write inside `<universe>/.constellation/` lands here.
                // Left unfiltered it costs a full `refreshLibraryTree` re-walk + loadAllStats,
                // and a vanished non-`.md` path additionally drives `delete_rows_under_prefix`
                // → the writer lock plus a lowercase scan of all 7,817 `note_meta` paths to
                // find zero victims. Live today (D3) via `cece/reliability.rs`'s tempfile
                // persist, independent of this migration.
                // Scoped to this ONE segment — NOT all dot-dirs: `.trash` holds real
                // `note_meta` rows and is a separate design question (handled at the indexer,
                // never here). Safe: zero `.md` files and zero `note_meta` rows live under any
                // `.constellation` dir (both measured).
                .filter(|p| !is_app_bookkeeping_path(p))
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

#[cfg(test)]
mod tests_mig104_watcher_excludes_constellation {
    //! MIG-104 Slice 1 — the app's own folder must never look like the user's knowledge
    //! changing. Measured with a ctypes probe replicating notify's exact
    //! CreateFileW + ReadDirectoryChangesW call: an append inside `.constellation`
    //! reports a bare-directory event NON-DETERMINISTICALLY, and a temp+replace
    //! (snapshot/compaction) or a rename-aside (corrupt-store contract) reports BOTH a
    //! bare-directory event AND a vanished path. Those are exactly the two shapes the
    //! filter below this predicate deliberately PASSES, so the predicate must come first.
    use super::is_app_bookkeeping_path;
    use std::path::Path;

    #[test]
    fn rejects_every_shape_the_ledger_writes() {
        for p in [
            // the bare directory event — hits `m.is_dir()` → would pass
            r"E:\U\Eisa Cognitive Knowledge\.constellation",
            // the tail's own path
            r"E:\U\Eisa Cognitive Knowledge\.constellation\earned.jsonl",
            // a vanished temp from a snapshot compaction — hits `Err(_)` → would pass
            r"E:\U\Eisa Cognitive Knowledge\.constellation\earned.tmp",
            // the corrupt-store rename-aside
            r"E:\U\Eisa Cognitive Knowledge\.constellation\earned.corrupt-2026.jsonl",
            // the live D3 case: cece/reliability.rs persists a tempfile here on every save
            r"E:\Cognitive Knowledge\Eisa Test\.constellation\cataloger_reliability.json",
            // nested deeper, and forward slashes (macOS / notify normalization)
            "E:/U/Eisa Cognitive Knowledge/.constellation/bases/All Notes.base",
        ] {
            assert!(is_app_bookkeeping_path(Path::new(p)), "must be rejected: {p}");
        }
    }

    #[test]
    fn accepts_the_users_knowledge_including_trash_and_vanished_folders() {
        for p in [
            r"E:\U\Eisa Cognitive Knowledge\Notes\a.md",
            // .trash holds real note_meta rows — a restore must reach the indexer.
            r"E:\U\Eisa Cognitive Knowledge\.trash\b.md",
            // a vanished user folder — the old-side signal the Err(_) arm exists to keep.
            r"E:\U\Eisa Cognitive Knowledge\Folder",
            r"E:\U\Eisa Cognitive Knowledge\Daily Notes\2026-06-17.md",
        ] {
            assert!(!is_app_bookkeeping_path(Path::new(p)), "must be accepted: {p}");
        }
    }

    #[test]
    fn matches_a_whole_component_never_a_substring() {
        // A user folder whose NAME merely contains the word must survive.
        assert!(!is_app_bookkeeping_path(Path::new(
            r"E:\U\Eisa Cognitive Knowledge\My .constellation notes\a.md"
        )));
        assert!(!is_app_bookkeeping_path(Path::new(
            r"E:\U\Eisa Cognitive Knowledge\constellation\a.md"
        )));
    }
}

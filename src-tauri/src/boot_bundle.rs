//! Boot bundle — a single IPC that returns everything the frontend needs
//! at startup.
//!
//! Why this exists
//! ---------------
//! Before this module, the frontend fired 10+ separate Tauri commands at
//! boot (read_universe_settings, read_universe_bookmarks, list_workspaces,
//! get_property_types, list_workspace_bases, get_child_universes,
//! read_child_universe_libraries × N, resolve_universe_libraries). Each
//! IPC call has non-trivial overhead — especially in dev mode on Windows
//! where Tauri v2 + Vite + WebView2 amplify the per-call cost to ~37s
//! (see LL-015 in docs/LESSONS-LEARNED.md).
//!
//! This bundle returns them all in one call. In production the speedup
//! is ~5× (10+ IPC round-trips → 1). In dev mode the speedup is ~11×
//! because the per-call overhead dominates and we pay it once instead of
//! eleven times.
//!
//! The Rust work itself is all file reads of small JSON files — total
//! Rust-side time is sub-millisecond. All the latency lived in the
//! serialized IPC round-trips we're now collapsing.

use serde::Serialize;
use std::collections::HashMap;

use crate::libraries::LibraryInfo;
use crate::universe::ChildUniverseInfo;

#[derive(Debug, Serialize)]
pub struct BootBundle {
    pub libraries: Vec<LibraryInfo>,
    pub settings: serde_json::Value,
    pub bookmarks: serde_json::Value,
    pub workspaces: serde_json::Value,
    pub property_types: serde_json::Value,
    pub workspace_bases: Vec<crate::bases::WorkspaceBaseEntry>,
    pub child_universes: Vec<ChildUniverseInfo>,
    /// Keyed by child universe path → list of library paths (normalized
    /// lowercase with forward slashes) that belong to that child.
    pub child_universe_lib_paths: HashMap<String, Vec<String>>,
}

/// Return everything the frontend needs at boot in one IPC call. This
/// replaces ~10 serialized calls during initializeApp with a single
/// round-trip. See module docs for rationale.
#[tauri::command]
pub fn constellation_boot_bundle(app: tauri::AppHandle) -> Result<BootBundle, String> {
    // Run each step fault-tolerantly — a missing settings.json or
    // universe.json shouldn't block the rest of the boot from succeeding.
    let libraries = crate::libraries::load_all_libraries(&app);

    let settings = crate::universe::read_universe_settings(app.clone())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let bookmarks = crate::universe::read_universe_bookmarks(app.clone())
        .unwrap_or(serde_json::Value::Array(vec![]));
    let workspaces = crate::universe::read_universe_workspaces(app.clone())
        .unwrap_or(serde_json::Value::Array(vec![]));
    let property_types = crate::universe::read_universe_property_types(app.clone())
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let workspace_bases = crate::bases::list_workspace_bases(app.clone())
        .unwrap_or_default();
    let child_universes = crate::universe::get_child_universes(app.clone())
        .unwrap_or_default();

    // Resolve each child's library paths in the same pass, so the frontend
    // doesn't have to do per-child round-trips.
    let mut child_universe_lib_paths: HashMap<String, Vec<String>> = HashMap::new();
    for cu in &child_universes {
        let paths = crate::universe::read_child_universe_libraries(app.clone(), cu.path.clone())
            .map(|libs| {
                libs.into_iter()
                    .map(|l| l.path.replace('\\', "/").to_lowercase())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        child_universe_lib_paths.insert(cu.path.clone(), paths);
    }

    Ok(BootBundle {
        libraries,
        settings,
        bookmarks,
        workspaces,
        property_types,
        workspace_bases,
        child_universes,
        child_universe_lib_paths,
    })
}

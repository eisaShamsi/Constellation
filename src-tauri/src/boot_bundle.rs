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
use std::time::Instant;

use crate::libraries::LibraryInfo;
use crate::universe::ChildUniverseInfo;

#[derive(Debug, Serialize)]
pub struct BootBundle {
    pub libraries: Vec<LibraryInfo>,
    pub settings: serde_json::Value,
    pub workspaces: serde_json::Value,
    /// MIG-100 — the auto-session snapshot (`.constellation/session.json`),
    /// or null when none exists. Delivered in the bundle so the boot-time
    /// tab restore costs zero extra IPC round-trips.
    pub session: serde_json::Value,
    /// MIG-100 hotfix (Boss Stage-2 failure 4) — the universe root `session`
    /// was READ FROM. The frontend restore refuses a payload whose origin
    /// differs from the root it is arming for: the active universe can flip
    /// between this read and the post-hydration restore (the Scratch switch
    /// incident restored Eisa-Cognitive-Knowledge's tabs inside Scratch and
    /// then persisted them into Scratch's file). Null when session is null.
    pub session_root: Option<String>,
    pub property_types: serde_json::Value,
    /// MIG-067 §C — the resolved Link-Type Registry (8 seeds + custom, ordered +
    /// nested) so the frontend `linkTypeRegistry` seeds without a separate IPC.
    pub link_types: Vec<crate::link_types::LinkTypeDef>,
    pub workspace_bases: Vec<crate::bases::WorkspaceBaseEntry>,
    pub child_universes: Vec<ChildUniverseInfo>,
    /// Keyed by child universe path → list of library paths (normalized
    /// lowercase with forward slashes) that belong to that child.
    pub child_universe_lib_paths: HashMap<String, Vec<String>>,
    /// Per-step wall-clock timings captured inside `constellation_boot_bundle`.
    /// Ordered the same way the steps run. The frontend writes these into
    /// `boot-perf.latest.json` as `boot_bundle_timings` so cold-boot
    /// bottlenecks can be attributed without instrumentation rebuilds.
    pub timings_ms: Vec<(String, u64)>,
}

/// Return everything the frontend needs at boot in one IPC call. This
/// replaces ~10 serialized calls during initializeApp with a single
/// round-trip. See module docs for rationale.
///
/// Also emits per-step timings (`timings_ms`) so cold-boot attribution is
/// possible without rebuilding — each step is wall-clock-measured with
/// `Instant::now()` and shipped alongside the payload.
// App-freeze audit Batch-S (2026-07-03): `(async)` — this command reaches
// ensure_search_db_ready (or a multi-second walk/read) and used to PARK the
// WebView2 dispatch thread for the whole 20-40s cold init after a universe
// switch / boot (the Boss-reproduced switch freeze). Off-thread, the init
// still runs exactly once (init_lock) but the app stays responsive.
#[tauri::command(async)]
pub fn constellation_boot_bundle(app: tauri::AppHandle) -> Result<BootBundle, String> {
    let mut timings: Vec<(String, u64)> = Vec::new();

    // Small helper: run `body`, push elapsed ms to `timings` under `label`,
    // return the body's value. A local closure is cleaner than a macro here
    // because the closure captures `timings` by &mut and each call-site is
    // straight-line code.
    macro_rules! time_step {
        ($label:expr, $body:expr) => {{
            let __t = Instant::now();
            let __v = $body;
            timings.push(($label.to_string(), __t.elapsed().as_millis() as u64));
            __v
        }};
    }

    // Run each step fault-tolerantly — a missing settings.json or
    // universe.json shouldn't block the rest of the boot from succeeding.
    let libraries = time_step!(
        "load_all_libraries",
        crate::libraries::load_all_libraries(&app)
    );

    let settings = time_step!(
        "read_universe_settings",
        // None = ambient active universe — correct at boot (the pointer is stable here).
        crate::universe::read_universe_settings(app.clone(), None)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
    );
    let workspaces = time_step!(
        "read_universe_workspaces",
        crate::universe::read_universe_workspaces(app.clone(), None)
            .unwrap_or(serde_json::Value::Array(vec![]))
    );
    // MIG-100 — the auto-session snapshot. The root is resolved HERE, passed
    // explicitly (the session IPCs never key off the ambient active pointer —
    // see universe.rs), and RETURNED alongside the payload so the frontend can
    // verify the payload's origin at restore time. No-active-universe or a
    // bad file both degrade to null = "no session", never a bundle failure.
    let (session, session_root) = time_step!("read_universe_session", {
        match crate::universe::active_universe_dir(&app) {
            Ok(root) => {
                let root_str = root.to_string_lossy().to_string();
                let v = crate::universe::read_universe_session(root_str.clone())
                    .unwrap_or(serde_json::Value::Null);
                if v.is_null() {
                    (serde_json::Value::Null, None)
                } else {
                    (v, Some(root_str))
                }
            }
            Err(_) => (serde_json::Value::Null, None),
        }
    });
    let property_types = time_step!(
        "read_universe_property_types",
        crate::universe::read_universe_property_types(app.clone(), None)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
    );
    // MIG-067 §C — resolved link-type vocabulary (self-loads the active universe's
    // deltas, so it's correct regardless of whether ensure_search_db_ready has run).
    // 2026-08-03 per-build inspection — `list_link_types` is deliberately STRICT (it refuses to
    // present an unreadable link-type file as "no custom types", because the editor writes back
    // from what it reads). This `unwrap_or_default()` collapses that refusal into an empty Vec,
    // which the frontend cannot distinguish from a real answer.
    //
    // The bundle keeps degrading rather than failing the whole boot — one unreadable settings
    // file must not stop the app from starting. What makes that safe is the FRONTEND latch:
    // `linkTypeRegistry.seedFromBundle` treats an empty list as proof of failure and refuses to
    // latch (a successful read always carries at least the 8 built-in seeds). Keep those two
    // facts together — if this ever returns a legitimately empty list, that latch breaks.
    let link_types = time_step!(
        "list_link_types",
        crate::link_types::list_link_types(app.clone()).unwrap_or_default()
    );
    let workspace_bases = time_step!(
        "list_workspace_bases",
        crate::bases::list_workspace_bases(app.clone()).unwrap_or_default()
    );
    let child_universes = time_step!(
        "get_child_universes",
        crate::universe::get_child_universes(app.clone()).unwrap_or_default()
    );

    // Resolve each child's library paths in the same pass, so the frontend
    // doesn't have to do per-child round-trips.
    let child_universe_lib_paths = time_step!("child_universes_lib_paths_loop", {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for cu in &child_universes {
            let paths = crate::universe::read_child_universe_libraries(app.clone(), cu.path.clone())
                .map(|libs| {
                    libs.into_iter()
                        .map(|l| l.path.replace('\\', "/").to_lowercase())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            map.insert(cu.path.clone(), paths);
        }
        map
    });

    Ok(BootBundle {
        libraries,
        settings,
        workspaces,
        session,
        session_root,
        property_types,
        link_types,
        workspace_bases,
        child_universes,
        child_universe_lib_paths,
        timings_ms: timings,
    })
}

//! Constellation Sight v3 — projection foundation (MIG-018).
//!
//! Computes a deterministic 2D Landmark-MDS embedding of the user's
//! knowledge graph and persists it to the `sight_v3_layout` SQLite
//! table. The frontend reads the cache at Sight-v3 toggle time and
//! projects to screen coordinates via either Lambert azimuthal
//! equal-area or stereographic projection (user-toggle in Settings).
//!
//! See: `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` §3.
//! Architect: `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-ARCHITECT.md`.
//! Plan:      `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-PLAN.md`.
//!
//! ── §1A status (this commit) ──
//! Stub only. The IPC is registered, the SQLite schema is in place
//! (created in `search.rs::init_db`), and the data type contracts are
//! defined. Returns an empty `Vec<LayoutPoint>` for now. The actual
//! Landmark-MDS algorithm + persistence + invalidation lands in §1B.
//!
//! Why stub-first: it lets §1A commit cleanly with `cargo check`
//! green, the SQLite schema applied on next boot, and the IPC routable
//! end-to-end — without committing partially-written analytics that
//! could mislead the frontend. §1B replaces the body with the real
//! compute.

use serde::Serialize;

/// One row in `sight_v3_layout`. Returned by `compute_layout_embedding`
/// to the frontend; one entry per note in the queried library set.
///
/// Coordinate system: `(embed_x, embed_y)` lie on the unit disk
/// (`embed_x² + embed_y² ≤ 1.0`). The frontend's `projection.ts`
/// applies either Lambert or stereographic to map disk → screen.
///
/// `centrality_norm` is normalized to `[0, 1]`: `1.0` = highest
/// betweenness centrality node in the universe; `0.0` = a leaf
/// (degree 1 with no shortest paths through it).
///
/// `community_id` is the Louvain community assignment; the frontend
/// maps id → palette color (Suwaidi warm-cream + gold cycle, per
/// Eisa's design call 2026-05-07).
#[derive(Debug, Clone, Serialize)]
pub struct LayoutPoint {
    pub note_path: String,
    pub embed_x: f32,
    pub embed_y: f32,
    pub community_id: i32,
    pub centrality_norm: f32,
}

/// `constellation_sight_v3_layout` — the v3 layout-cache IPC.
///
/// Frontend calls this on Sight-v3 toggle. Implementation behavior:
/// - **Cache hit** (graph_version matches current): SELECT from
///   `sight_v3_layout` and return the cached rows. Sub-50ms.
/// - **Cache miss** (graph_version stale or no rows): run Landmark-MDS,
///   persist, return. Sub-500ms on Boss's 7,600-note universe.
///
/// `library_paths` — set of library directory paths to include in the
/// graph (mirrors v2's `constellation_sight_centrality` shape; passed
/// from frontend's `$libraryStats` via the same flow).
///
/// `k_landmarks` — number of MDS landmarks. Default 50 in §1B
/// (frontend will pass this; pinned in the IPC signature so future
/// tuning is non-breaking).
///
/// ── §1A behavior ──
/// Returns `Ok(vec![])`. The frontend's layout-cache wrapper logs the
/// count and falls through to the (also-stubbed) §1C empty render.
/// §1B replaces this body with the real compute.
#[tauri::command]
pub fn constellation_sight_v3_layout(
    library_paths: Vec<String>,
    k_landmarks: usize,
) -> Result<Vec<LayoutPoint>, String> {
    // §1A: log the call shape so a §1B implementer can see whether the
    // frontend wiring is correct in isolation. No DB I/O, no compute.
    let _ = (library_paths, k_landmarks);
    Ok(Vec::new())
}

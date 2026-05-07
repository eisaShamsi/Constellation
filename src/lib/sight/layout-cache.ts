/**
 * Sight v3 — frontend wrapper for the `constellation_sight_v3_layout` IPC.
 *
 * Calls the Rust IPC, optionally caches the result in memory, and
 * exposes the layout to `SightV3.svelte`. §1B persisted the layout to
 * SQLite Rust-side; this module is the JS-side handle to that cached
 * data + a thin recompute trigger.
 *
 * The frontend invalidates by calling `invalidateLayout()` — which
 * drops the in-memory cache AND calls the Rust
 * `constellation_sight_v3_invalidate_layout` IPC to bump the
 * SQLite-side `sight_v3_graph_version`.
 *
 * §1C (this commit): basic IPC wrapper + module-level Map cache.
 * §1D will tune for warm-cache speed (skip the IPC entirely if the
 * in-memory cache hit is ≤ 5 minutes old).
 */
import { invoke } from '@tauri-apps/api/core';

export interface LayoutPoint {
    note_path: string;
    embed_x: number;
    embed_y: number;
    /** §1B persists 0; §1E populates from clusterEngine.ts Louvain output. */
    community_id: number;
    /** Normalized betweenness centrality, [0, 1]. */
    centrality_norm: number;
}

/** Module-level cache, keyed by sorted `library_paths.join('|')`. */
const memoryCache = new Map<string, LayoutPoint[]>();

function cacheKey(libraryPaths: Array<[string, string]>): string {
    return libraryPaths
        .map(([p]) => p)
        .slice()
        .sort()
        .join('|');
}

/**
 * Fetch the v3 layout for the given library set. Returns cached
 * in-memory result if present; otherwise hits the IPC.
 *
 * §1C semantics: in-memory cache is process-lifetime; cleared via
 * `invalidateLayout()`. §1D tightens to 5-minute TTL plus version
 * comparison.
 *
 * @param libraryPaths Tuples of `(library_path, library_name)` —
 *     same shape as `constellation_sight_centrality` to ease v2 → v3
 *     wiring at call sites.
 * @param kLandmarks Number of MDS landmarks. Default 50. Larger k =
 *     higher fidelity but more BFS passes (sub-linear cost on most
 *     graphs).
 */
export async function fetchLayout(
    libraryPaths: Array<[string, string]>,
    kLandmarks: number = 50,
): Promise<LayoutPoint[]> {
    const key = cacheKey(libraryPaths);
    const cached = memoryCache.get(key);
    if (cached) {
        return cached;
    }
    const points = await invoke<LayoutPoint[]>('constellation_sight_v3_layout', {
        libraryPaths,
        kLandmarks,
    });
    memoryCache.set(key, points);
    return points;
}

/**
 * Invalidate the in-memory cache + bump the SQLite-side graph_version.
 * Call after any operation that mutates the graph (note rename, link
 * create/delete, etc.). §1B's IPC bumps the version for ALL library
 * sets — coarse-grained but cheap.
 */
export async function invalidateLayout(): Promise<void> {
    memoryCache.clear();
    await invoke('constellation_sight_v3_invalidate_layout');
}

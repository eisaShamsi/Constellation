/**
 * Sight v3 — frontend wrapper for the `constellation_sight_v3_similarity_field` IPC.
 *
 * Calls the Rust IPC, caches in-memory by sorted library_paths, exposes
 * the high-similarity edge list to `SightV3.svelte` for the Milky Way
 * density-wash render.
 *
 * Mirrors `layout-cache.ts`. Module-level `Map` cache + `invalidateSimilarity()`
 * to drop the cache (and let next call hit the IPC + recompute).
 *
 * §2A (Rust) ships cache-aware semantics — the Rust IPC returns cached
 * SQLite rows when the (library_set_hash, graph_version) hits. So a
 * fresh `fetchSimilarity()` call after invalidation does the cold compute
 * once and then re-caches.
 */
import { invoke } from '@tauri-apps/api/core';

export interface SimilarityEdge {
    note_path_a: string;
    note_path_b: string;
    /** Cosine similarity ∈ [0, 1]. */
    similarity: number;
}

const memoryCache = new Map<string, SimilarityEdge[]>();

function cacheKey(libraryPaths: Array<[string, string]>): string {
    return libraryPaths
        .map(([p]) => p)
        .slice()
        .sort()
        .join('|');
}

/**
 * Fetch the v3 similarity edge list. Returns cached in-memory result
 * if present; otherwise hits the IPC.
 *
 * @param libraryPaths Tuples of `(library_path, library_name)` —
 *     same shape as `fetchLayout`.
 * @param kTopTerms Number of top TF×IDF terms per note for the
 *     inverted-index candidate filter. Default 50.
 * @param similarityThreshold Cosine similarity floor. Pairs below this
 *     are dropped. Default 0.3 (per Concept Paper v1.1 §3.3).
 */
export async function fetchSimilarity(
    libraryPaths: Array<[string, string]>,
    kTopTerms: number = 50,
    similarityThreshold: number = 0.3,
): Promise<SimilarityEdge[]> {
    const key = cacheKey(libraryPaths);
    const cached = memoryCache.get(key);
    if (cached) return cached;
    const edges = await invoke<SimilarityEdge[]>('constellation_sight_v3_similarity_field', {
        libraryPaths,
        kTopTerms,
        similarityThreshold,
    });
    memoryCache.set(key, edges);
    return edges;
}

/** Drop the in-memory cache. The Rust SQLite cache stays — version-bump
 *  is the way to invalidate that. Call this when the graph mutates and
 *  you want a fresh fetch on the next `fetchSimilarity()` call. */
export function invalidateSimilarity(): void {
    memoryCache.clear();
}

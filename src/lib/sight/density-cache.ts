/**
 * Sight v3 — frontend wrapper for the `constellation_sight_v3_density_field` IPC.
 *
 * MIG-019 §2A+§2B redesign (Eisa's 2026-05-07 directive: "Don't patch
 * it. Solve it."). Replaces `similarity-cache.ts` after the edge-list
 * approach was exposed as fundamentally OOM-prone on large universes.
 *
 * The IPC returns a fixed-size 2D density grid (256×256 f32 by default,
 * ~256 KB regardless of universe size) that captures the TF-IDF
 * content-similarity field per Concept Paper v1.1 §5.1. The grid is
 * the IPC payload; the frontend builds a Pixi Texture from it and
 * renders as a single Sprite — one draw call.
 *
 * Memory & perf: input-size invariant. 7,600 notes / 656k links: same
 * 256 KB payload as 100 notes. JS heap impact: ~256 KB Float32Array +
 * one Pixi Texture (~512 KB tops).
 */
import { invoke } from '@tauri-apps/api/core';

export interface DensityField {
    /** Grid width in cells. */
    width: number;
    /** Grid height in cells. */
    height: number;
    /** Maximum cell value across the grid. Used for alpha normalization
     *  in the renderer ([0, max] → [0, 1] alpha). */
    max_value: number;
    /** Length = width × height. Row-major: `values[y * width + x]`. */
    values: number[];
}

const memoryCache = new Map<string, DensityField>();

function cacheKey(libraryPaths: Array<[string, string]>): string {
    return libraryPaths
        .map(([p]) => p)
        .slice()
        .sort()
        .join('|');
}

/**
 * Fetch the v3 density field. Returns cached in-memory result if
 * present; otherwise hits the IPC.
 *
 * @param libraryPaths Tuples of `(library_path, library_name)` —
 *     same shape as `fetchLayout`.
 * @param kTopTerms Number of top TF×IDF terms per note for the
 *     inverted-index candidate filter. Default 50.
 * @param similarityThreshold Cosine similarity floor. Pairs below this
 *     don't contribute to the grid. Default 0.3 (Concept Paper §3.3).
 */
export async function fetchDensityField(
    libraryPaths: Array<[string, string]>,
    kTopTerms: number = 50,
    similarityThreshold: number = 0.3,
): Promise<DensityField> {
    const key = cacheKey(libraryPaths);
    const cached = memoryCache.get(key);
    if (cached) return cached;
    const field = await invoke<DensityField>('constellation_sight_v3_density_field', {
        libraryPaths,
        kTopTerms,
        similarityThreshold,
    });
    memoryCache.set(key, field);
    return field;
}

/** Drop the in-memory cache. The Rust SQLite cache stays — version-bump
 *  is the way to invalidate that. Call after a graph mutation if you
 *  want a fresh fetch on the next call. */
export function invalidateDensity(): void {
    memoryCache.clear();
}

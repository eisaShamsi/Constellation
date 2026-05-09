// MIG-021v2 §1A' — Frontend wrapper for the horizontal-axis taxonomy.
//
// Single source of truth lives in `src-tauri/src/sources/horizontal_taxonomy.rs`
// (Rust). This file fetches via Tauri IPC + caches in-memory. The tree picker
// and Source Review panel consume the cached data.
//
// Anchored against `docs/sources-of-knowledge-diagram.html` (Eisa-canonical).

import { invoke } from '@tauri-apps/api/core';

export type HorizontalNode = {
  id: string;
  en: string;
  ar: string;
  tr: string | null;
  /** 1 = universally accepted, 2 = broadly accepted, 3 = school-specific. 0 = leaf or opt-out token (no own tier). */
  tier: number;
  parent_id: string | null;
};

let _cache: HorizontalNode[] | null = null;

/**
 * Fetch the full horizontal-axis taxonomy. Cached after first call;
 * the data is static-as-shipped and never changes within a session.
 */
export async function getHorizontalTaxonomy(): Promise<HorizontalNode[]> {
  if (_cache) return _cache;
  _cache = await invoke<HorizontalNode[]>('sources_get_horizontal_taxonomy');
  return _cache;
}

/** Reset the cache (test-only; production code should never call this). */
export function _resetHorizontalCache(): void {
  _cache = null;
}

/** Lookup helpers (call after `getHorizontalTaxonomy()` has resolved). */
export function findNode(taxonomy: HorizontalNode[], id: string): HorizontalNode | undefined {
  return taxonomy.find((n) => n.id === id);
}

export function childrenOf(taxonomy: HorizontalNode[], parentId: string): HorizontalNode[] {
  return taxonomy.filter((n) => n.parent_id === parentId);
}

export function topLevelParents(taxonomy: HorizontalNode[]): HorizontalNode[] {
  return taxonomy.filter((n) => n.parent_id === null && n.id !== 'unclassifiable');
}

/** Effective tier — for leaves, returns the parent's tier. Used for tier-coloring in tree picker. */
export function effectiveTier(taxonomy: HorizontalNode[], id: string): number {
  const node = findNode(taxonomy, id);
  if (!node) return 0;
  if (node.tier > 0) return node.tier;
  if (node.parent_id) {
    const parent = findNode(taxonomy, node.parent_id);
    if (parent) return parent.tier;
  }
  return 0;
}

/** Suwaidi-aligned tier color (matches sources-of-knowledge-diagram.html). */
export function tierColor(tier: number): string | null {
  switch (tier) {
    case 1:
      return '#0f6e56'; // teal — universally accepted
    case 2:
      return '#534ab7'; // purple — broadly accepted
    case 3:
      return '#854f0b'; // amber — school-specific or contested
    default:
      return null;
  }
}

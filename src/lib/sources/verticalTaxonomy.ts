// MIG-021v2 §1A' — Frontend wrapper for the vertical-axis taxonomy.
//
// Single source of truth lives in `src-tauri/src/sources/vertical_taxonomy.rs`
// (Rust). This file fetches via Tauri IPC + caches in-memory.
//
// Anchored against `docs/epistemic-content-taxonomy-chart.html` (Eisa-canonical).

import { invoke } from '@tauri-apps/api/core';

export type VerticalNode = {
  id: string;
  en: string;
  ar: string;
  parent_id: string | null;
  /** 1-5 for the five top-level branches; 0 for the root. */
  branch: number;
};

let _cache: VerticalNode[] | null = null;

export async function getVerticalTaxonomy(): Promise<VerticalNode[]> {
  if (_cache) return _cache;
  _cache = await invoke<VerticalNode[]>('sources_get_vertical_taxonomy');
  return _cache;
}

export function _resetVerticalCache(): void {
  _cache = null;
}

export function findNode(taxonomy: VerticalNode[], id: string): VerticalNode | undefined {
  return taxonomy.find((n) => n.id === id);
}

export function childrenOf(taxonomy: VerticalNode[], parentId: string): VerticalNode[] {
  return taxonomy.filter((n) => n.parent_id === parentId);
}

export function topLevelBranches(taxonomy: VerticalNode[]): VerticalNode[] {
  return taxonomy.filter((n) => n.parent_id === 'epistemic-content');
}

/** Branch color matching the chart's per-branch color scheme. */
export function branchColor(branch: number): string | null {
  switch (branch) {
    case 1:
      return '#534ab7'; // purple — Sensory inputs
    case 2:
      return '#0f6e56'; // teal — Symbolic entities
    case 3:
      return '#993556'; // pink — Semantic contents
    case 4:
      return '#993c1d'; // coral — Epistemic states
    case 5:
      return '#854f0b'; // amber — Higher-order constructs
    default:
      return null;
  }
}

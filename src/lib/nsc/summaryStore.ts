/**
 * NSC summary store — the app-wide "what is this note about?" provider.
 *
 * MIG-043 Phase 1: generalizes the batched cache-first fetch that previously
 * lived only inside `SourceReviewPanel.svelte` (direct `invoke` of
 * `nsc_get_summaries_for_notes`) into a shared module any surface can use.
 *
 * Discipline (CLAUDE.md Rule 3 + Concept Paper §6):
 *   - Cache-first (in-memory `Map<path, NoteSummaryEntry>`).
 *   - Batched: one IPC per call regardless of how many paths.
 *   - Coalesced: concurrent requests for the same path share one in-flight
 *     promise — no duplicate IPC.
 *   - Self-invalidating: a single `library-changed` watcher drops cached
 *     entries when their file changes on disk.
 *   - Zero per-item IPC on render — surfaces ask for a *batch* of visible
 *     paths and update when the promise resolves.
 *
 * Consumers (post-MIG-043 Phase 1): SourceReviewPanel.svelte (migrated from
 * its inline invoke); search results + editor header land in Steps C/D.
 * Phase 2 (MIG-044) wires the remaining surfaces. Phase 3 (MIG-045) is the
 * Universe Digest, which uses this store via the same API.
 */
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * One cached/computed summary for a note. Mirrors the Rust
 * `nsc::NoteSummaryEntry` shape (post-MIG-043 Phase 1; `headline` is the
 * 1-line top-1 TextRank / first-sentence-of-author-summary variant).
 *
 * `headline` is `#[serde(default)]` on the Rust side, so an older binary that
 * doesn't return it yields `''` here — the store stays usable, just without
 * the headline field populated until the cache row recomputes.
 */
export type NoteSummaryEntry = {
  path: string;
  summary: string;
  source: string;
  headline: string;
};

// In-memory cache: path -> the most recent entry seen for that path.
const cache = new Map<string, NoteSummaryEntry>();

// In-flight per-path promises so concurrent callers coalesce onto one IPC.
// `undefined` is a valid resolved value (the backend returned no entry for
// that path — typically an empty body).
const inflight = new Map<string, Promise<NoteSummaryEntry | undefined>>();

// File-watcher unlisten handle — set once on first use, kept for the app's
// lifetime (no per-surface re-listen).
let watcherUnlisten: UnlistenFn | null = null;
let watcherInit: Promise<void> | null = null;

/**
 * Get summaries for a batch of note paths. Cache-first, batched, coalesced.
 * Returns a `Map<path, NoteSummaryEntry>` containing only the paths that had
 * a non-empty summary (mirrors `nsc_get_summaries_for_notes` which omits
 * notes with no body / no result).
 *
 * Safe to call from a `$effect` / `$derived` — does not throw on IPC failure
 * (logs to console; the returned map omits the failed paths).
 */
export async function getSummariesFor(notePaths: string[]): Promise<Map<string, NoteSummaryEntry>> {
  if (notePaths.length === 0) return new Map();
  // Lazily set up the invalidation listener on first use (idempotent).
  await ensureInvalidationListener();

  const result = new Map<string, NoteSummaryEntry>();
  const waitOn: Array<Promise<NoteSummaryEntry | undefined>> = [];
  const toFetch: string[] = [];

  for (const p of notePaths) {
    const cached = cache.get(p);
    if (cached) {
      result.set(p, cached);
      continue;
    }
    const inf = inflight.get(p);
    if (inf) {
      // Coalesce: another caller is already fetching this path; wait on it.
      waitOn.push(inf);
      continue;
    }
    toFetch.push(p);
  }

  if (toFetch.length > 0) {
    // One batched IPC for everything not in cache and not already in flight.
    const batch = invoke<NoteSummaryEntry[]>('nsc_get_summaries_for_notes', {
      notePaths: toFetch,
    })
      .then((entries) => {
        const byPath = new Map<string, NoteSummaryEntry>();
        for (const e of entries) byPath.set(e.path, e);
        // Populate the long-lived cache for every fetched path (even those
        // with no result — we'd just remap them to `undefined` below; but
        // we deliberately DON'T cache misses so a future cache fill via the
        // backfill can succeed without an explicit invalidate from here).
        for (const e of entries) cache.set(e.path, e);
        // Clear in-flight markers for everything in this batch.
        for (const p of toFetch) inflight.delete(p);
        return byPath;
      })
      .catch((err) => {
        console.error('[NSC summaryStore] fetch failed:', err);
        for (const p of toFetch) inflight.delete(p);
        return new Map<string, NoteSummaryEntry>();
      });

    // Register per-path in-flight promises so any concurrent caller asking
    // for the same path coalesces onto this batch.
    for (const p of toFetch) {
      const perPath = batch.then((byPath) => byPath.get(p));
      inflight.set(p, perPath);
      waitOn.push(perPath);
    }
  }

  // Wait on every in-flight (this batch + any prior coalesced ones) and merge.
  const settled = await Promise.all(waitOn);
  for (const entry of settled) {
    if (entry) result.set(entry.path, entry);
  }

  return result;
}

/** Convenience: single-path get-or-compute. */
export async function getSummaryFor(notePath: string): Promise<NoteSummaryEntry | undefined> {
  const map = await getSummariesFor([notePath]);
  return map.get(notePath);
}

/**
 * Drop cached entries for the given paths. Call manually if a surface knows
 * a path's content has changed in a way the file watcher won't see (rare).
 * The file watcher invalidation fires automatically on `library-changed`.
 */
export function invalidate(notePaths: string[]): void {
  for (const p of notePaths) cache.delete(p);
}

/**
 * Drop the entire cache. Useful on universe switch — though, in practice,
 * after switch the new universe's note_paths simply won't be in the cache
 * yet, so this is mostly a memory-hygiene call.
 */
export function clearAll(): void {
  cache.clear();
  inflight.clear();
}

/** Lazy-init the file-watcher invalidation listener. Idempotent. */
function ensureInvalidationListener(): Promise<void> {
  if (watcherUnlisten) return Promise.resolve();
  if (watcherInit) return watcherInit;
  watcherInit = listen<{ libraryId: string; paths: string[] }>(
    'library-changed',
    (event) => {
      const paths = event.payload?.paths;
      if (Array.isArray(paths) && paths.length > 0) invalidate(paths);
    },
  )
    .then((unlisten) => {
      watcherUnlisten = unlisten;
    })
    .catch((err) => {
      console.warn('[NSC summaryStore] could not subscribe to library-changed:', err);
      // Reset so a future getSummariesFor call retries — best-effort.
      watcherInit = null;
    });
  return watcherInit;
}

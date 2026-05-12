/**
 * Sight v5 — D-V3 scope filter.
 *
 * Per Eisa's D-V3 lock (2026-05-12), Sight v5 supports three scope
 * levels — Universe (all notes), Library (current Library context),
 * or Folder (current Folder context). The user toggles in-canvas
 * via the U/L/F button row.
 *
 * The scope filter applies BEFORE wedge computation: filter rows
 * first, then modes.ts runs against the filtered set. Spatial memory
 * is still preserved within the filtered universe because radial
 * position (strata) doesn't change with scope.
 */

import type { LayoutCacheRow, SightV5Scope } from './types';

/** Filter the layout cache rows by the requested scope.
 *
 *  universe → all rows.
 *  library  → rows whose libraryName === scopeId.
 *  folder   → rows whose folderPath === scopeId OR is nested under it.
 *
 *  If scopeId is null/undefined for library/folder, returns the full
 *  set (graceful fallback — the U/L/F button row dims L/F when no
 *  scope target is available). */
export function filterNotesByScope(
	rows: LayoutCacheRow[],
	scope: SightV5Scope,
	scopeId: string | null | undefined,
): LayoutCacheRow[] {
	if (scope === 'universe') return rows;
	if (!scopeId) return rows;

	if (scope === 'library') {
		return rows.filter(r => r.libraryName === scopeId);
	}
	// folder: scopeId is a folder path; include the folder itself + nested.
	const prefix = scopeId.endsWith('/') ? scopeId : `${scopeId}/`;
	return rows.filter(r => r.folderPath === scopeId || (r.folderPath ?? '').startsWith(prefix));
}

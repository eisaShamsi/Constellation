/**
 * MIG-036 P2 (2026-05-19) — Sight v7 stack primitive.
 *
 * Per the Form-Aligns-To-Purpose rule + Architect §4 + §7.2: when
 * the user drills into a cell (click → expand), the cell's notes
 * render as a STACK (one row per note, identifiable + clickable)
 * instead of as positions inside the cell. The angular position
 * inside the cell is never used; the cell's stack is the per-note
 * view.
 *
 * This module is pure-functional: takes a filtered note list +
 * sort options, returns the ordered stack a per-shape renderer
 * (or the Cell View component in P7) can paint.
 *
 * Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md §7.2
 */

import type { LayoutCacheRow } from '../v6/types';

// ════════════════════════════════════════════════════════════════════
// Stack model
// ════════════════════════════════════════════════════════════════════

/** Sort key options for a cell's stack. Most cells sort by stratum
 *  (most foundational first) so the stack reads inside-out from the
 *  most-foundational to the edge-of-knowing. Time-axis sort is the
 *  Time Dome's natural order. Hash is a fallback for traditions
 *  whose grammar implies no internal ordering at all (a deterministic
 *  shuffle that stays stable between renders but carries no analytical
 *  meaning — kept here as the named fallback rather than as filler
 *  inside the visual position itself). */
export type StackSort = 'stratum' | 'created' | 'modified' | 'title' | 'hash';

/** Single row in a cell's drill-in stack. */
export interface StackRow {
	notePath: string;
	title: string; // display title (already-resolved via row.name fallback chain)
	libraryName: string | null;
	stratum: number | null;
	confidenceAlpha: number | null;
	stage: string | null;
	createdMonth: number | null;
}

/** Build a stack from a filtered note list + sort key. The cell-
 *  membership filter is the per-shape renderer's responsibility
 *  (e.g., masādir's drill-in passes only notes whose masadirSource
 *  === clickedCellId); this function orders + projects to the
 *  display shape. */
export function buildStack(
	rows: LayoutCacheRow[],
	titleResolver: (row: LayoutCacheRow) => string,
	sort: StackSort = 'stratum',
): StackRow[] {
	const projected: StackRow[] = rows.map((r) => ({
		notePath: r.notePath,
		title: titleResolver(r),
		libraryName: r.libraryName,
		stratum: r.stratum,
		confidenceAlpha: r.confidenceAlpha,
		stage: r.stage,
		createdMonth: r.createdMonth,
	}));
	return sortStack(projected, sort);
}

/** Sort a stack in place by the chosen key. Stable: ties preserve
 *  pre-existing order (caller's input order). */
export function sortStack(stack: StackRow[], sort: StackSort): StackRow[] {
	switch (sort) {
		case 'stratum': {
			// Most foundational first (lowest stratum number = most foundational).
			// Null stratum goes to the END (treat as Edge of Knowing).
			return stack.slice().sort((a, b) => {
				const av = a.stratum ?? 999;
				const bv = b.stratum ?? 999;
				return av - bv;
			});
		}
		case 'created': {
			// By createdMonth ascending. Null last.
			return stack.slice().sort((a, b) => {
				const av = a.createdMonth ?? 999;
				const bv = b.createdMonth ?? 999;
				return av - bv;
			});
		}
		case 'modified':
			// LayoutCacheRow doesn't carry modified; placeholder for
			// future enhancement when sight_v7's IPC returns it.
			return stack.slice();
		case 'title':
			return stack.slice().sort((a, b) => a.title.localeCompare(b.title));
		case 'hash':
			return stack.slice().sort((a, b) => pathHash01(a.notePath) - pathHash01(b.notePath));
	}
}

/** Filter a row set to the notes belonging to a specific cell of
 *  the active tradition. Per-shape renderers will use this with
 *  their tradition's cell-membership predicate (e.g., for masādir:
 *  `(r) => r.masadirSource === 'sunnah'`).
 *
 *  Kept here as a thin helper so the per-shape renderers can stay
 *  focused on geometry; this module owns the row → stack pipeline. */
export function filterCell(
	rows: LayoutCacheRow[],
	predicate: (row: LayoutCacheRow) => boolean,
): LayoutCacheRow[] {
	return rows.filter(predicate);
}

// ════════════════════════════════════════════════════════════════════
// Internal helpers
// ════════════════════════════════════════════════════════════════════

/** Deterministic [0, 1) hash for a notePath. Same FNV-1a pattern as
 *  the v6 tradition modules; duplicated locally so v7 stays free of
 *  v6 dependencies. */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

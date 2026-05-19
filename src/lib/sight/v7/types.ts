/**
 * MIG-036 P3 (2026-05-19) — Sight v7 type contracts.
 *
 * The v7 tradition contract is intentionally different from v6's:
 * v6 traditions exposed a `remapStarPosition()` that moved every
 * star into the tradition's grammar. v7 traditions instead expose
 * `cellRegions()` — they declare the categorical cells of their
 * grammar + provide a per-note membership predicate. Notes don't
 * have within-cell positions in v7; the cell IS the position.
 *
 * v7 reuses v6's `LayoutCacheRow` shape (per Architect §8) — no
 * new schema. Just a different rendering contract.
 *
 * Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md §5
 */

import type { LayoutCacheRow } from '../v6/types';

// Re-export so v7 modules can import everything from one place.
export type { LayoutCacheRow };

// ════════════════════════════════════════════════════════════════════
// Cell regions — the heart of the v7 tradition contract
// ════════════════════════════════════════════════════════════════════

/** Geometric center + bounding circle for a single cell in a
 *  tradition's grammar. The renderer paints the density blob at
 *  this center; the bounding radius is used for hit-testing
 *  (click within bound → drill into this cell). */
export interface CellGeometry {
	/** Logical id. Stable across renders. Used by membership
	 *  predicates + drill-in state. */
	id: string;
	/** Display label (the renderer translates via `labelize(key)`
	 *  the same way v6 did). May be a literal string for user-
	 *  defined plugins. */
	label: string;
	/** Center position in dome local coordinates (centerX, centerY,
	 *  radius from `DomeLayout`). The renderer offsets from the
	 *  dome center. */
	cx: number;
	cy: number;
	/** Hit-test radius in dome local coordinates. Click inside this
	 *  bound → drill into this cell. Should be larger than the
	 *  density blob's max radius so users have a generous click
	 *  target. */
	hitRadius: number;
}

/** Dome layout the renderer hands traditions for their cellRegions
 *  computation. Same shape as v6's `DomeLayout` so traditions can
 *  reuse v6's geometry math (radius fractions, angular offsets,
 *  etc.) wholesale. */
export interface TraditionLayoutV7 {
	centerX: number;
	centerY: number;
	radius: number;
}

/** Per-note cell-membership: given a row, return the id of the cell
 *  it belongs to under this tradition (or null if the note doesn't
 *  belong to any cell — e.g. a note with no `masadir_source`
 *  frontmatter under the masādir tradition). The renderer counts
 *  populations by calling this for every row.
 *
 *  A tradition with a strict-default policy (every note must land
 *  somewhere) returns its default cell id for unknown values. A
 *  tradition with a strict-opt-in policy returns null for unknown
 *  values; those notes don't render in the universe view (they're
 *  visible in the Time Dome anyway). Per-tradition choice. */
export type CellMembership = (row: LayoutCacheRow) => string | null;

/** v7 tradition module contract. Pure data + small callbacks; no
 *  per-note position remap (that was the v6 contract that v7 is
 *  replacing per Form-Aligns-To-Purpose). */
export interface TraditionModuleV7 {
	/** Stable id — used in settings.activeTradition and i18n keys. */
	id: string;

	/** Display name (i18n key resolved at render time). */
	name: string;

	/** Tradition shape — informational; the dispatcher uses
	 *  cellRegions() + cellMembership() uniformly across all shapes
	 *  in v7. */
	shape:
		| 'sectoral'
		| 'concentric'
		| 'grid'
		| 'ladder'
		| 'horizontal-bands'
		| 'binary-flow'
		| 'relational'
		| 'gradient'
		| 'radial-tower'
		| 'stratum-time';

	/** Categorical cells of this tradition. Called once per render
	 *  pass; the returned geometries drive density placement +
	 *  hit-testing. */
	cellRegions(layout: TraditionLayoutV7): CellGeometry[];

	/** Per-note cell-membership predicate. */
	cellMembership: CellMembership;

	/** Whether this tradition shows the calendar rim. Only the Time
	 *  Dome returns true; all other traditions return false (per
	 *  Form-Aligns-To-Purpose: time is the Time Dome's grammar,
	 *  not a universal chrome). */
	showCalendarRim?: boolean;

	/** Whether this tradition uses Universe-view density rendering
	 *  (true for all categorical traditions) vs. per-note rendering
	 *  (true only for Time Dome where both axes are meaningful and
	 *  individual stars carry the answer). Default: true. */
	useDensityView?: boolean;
}

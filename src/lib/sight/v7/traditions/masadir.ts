/**
 * MIG-036 P3 (2026-05-19) — masādir tradition under Sight v7.
 *
 * v6 ancestor: src/lib/sight/v6/traditions/masadir.ts (4-quadrant
 * sectoral with within-quadrant month-fraction angular positioning).
 *
 * v7 redesign per Form-Aligns-To-Purpose: each of the 4 sources
 * (Qur'an, sunnah, ijmā', qiyās) is a categorical cell. The cell
 * IS the unit at universe view (density-blob magnitude encodes
 * population); the cell expands to a stack of individual notes
 * on drill-in. No within-quadrant angular positions — those were
 * the canonical Form-Aligns-To-Purpose violation v7 fixes.
 *
 * Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md §5.1
 */

import type {
	TraditionModuleV7,
	TraditionLayoutV7,
	CellGeometry,
	CellMembership,
	LayoutCacheRow,
} from '../types';

// ════════════════════════════════════════════════════════════════════
// Geometry
// ════════════════════════════════════════════════════════════════════

/** Cell centers at diagonal NE / SE / SW / NW positions per Concept
 *  Paper §4.1.3 (the original geometry before v6's +π/4 rotation
 *  shifted them to cardinal E/S/W/N).
 *
 *  Why diagonals and not cardinals: stratum labels (FOUNDATION /
 *  WORKING / CONNECTION / SYNTHESIS / EDGE OF KNOWING) sit on the
 *  +y vertical axis at multiple radii. Any cell on the vertical
 *  axis collides with them — v6 avoided this by spreading individual
 *  stars across each wedge with hash jitter, so the stratum labels
 *  showed BESIDE the stars rather than on top of them. v7 collapses
 *  each wedge to one density blob at the cell center; that blob
 *  MUST live off the +y axis to stay clear of the stratum labels.
 *  Diagonals are equidistant from both axes and avoid the +x axis
 *  too (where calendar rim labels would sit if a tradition opted
 *  into them — masādir doesn't, but the geometry is forward-
 *  compatible).
 *
 *  Why v7 doesn't need v6's wedge-divider rotation: v7 doesn't
 *  draw wedge dividers at all (per the Form-Aligns-To-Purpose
 *  redesign — the cell IS the unit, not a bounded slice of the
 *  dome). So the +π/4 offset v6 added (§θ-fix-1) to push dividers
 *  off the cardinal axes is unnecessary here; cells land at the
 *  Concept Paper's original NE/SE/SW/NW positions directly.
 *
 *  Canvas math convention: 0 = EAST, +π/2 = SOUTH (canvas y is
 *  inverted from math), +π = WEST, -π/2 = NORTH.
 *
 *  MIG-036 P3-fix-1 (Eisa Boss test 2026-05-19): pre-fix had cells
 *  at cardinal E/S/W/N (v6's rotation preserved); qiyās blob at N
 *  overlapped with the CONNECTION stratum label. */
const CELL_CENTER_ANGLES = {
	quran: -Math.PI / 4, // NE (top-right)
	sunnah: Math.PI / 4, // SE (bottom-right)
	ijma: (3 * Math.PI) / 4, // SW (bottom-left)
	qiyas: -(3 * Math.PI) / 4, // NW (top-left)
} as const;

/** Cell centers sit at 55% of the dome radius from the dome center
 *  — far enough out that the central stratum labels stay visible
 *  but close enough in that the largest density blob (max 48px)
 *  doesn't crowd the dome rim. */
const CELL_RADIUS_FRAC = 0.55;

/** Hit-test radius around each cell center (in dome local coords).
 *  Generous so the click target is forgiving — 35% of dome radius
 *  means quadrant clicks don't fall through to the dome. */
const CELL_HIT_RADIUS_FRAC = 0.35;

// ════════════════════════════════════════════════════════════════════
// i18n keys (same namespace as v6 — v7 doesn't introduce its own
// since the cell labels are stable across the v6 → v7 redesign)
// ════════════════════════════════════════════════════════════════════

const CELL_LABEL_KEYS = {
	quran: 'sight.v6.tradition.canvas.masadir.quran',
	sunnah: 'sight.v6.tradition.canvas.masadir.sunnah',
	ijma: 'sight.v6.tradition.canvas.masadir.ijma',
	qiyas: 'sight.v6.tradition.canvas.masadir.qiyas',
} as const;

// ════════════════════════════════════════════════════════════════════
// Tradition module
// ════════════════════════════════════════════════════════════════════

function cellRegions(layout: TraditionLayoutV7): CellGeometry[] {
	const r = layout.radius * CELL_RADIUS_FRAC;
	const hitR = layout.radius * CELL_HIT_RADIUS_FRAC;
	return (['quran', 'sunnah', 'ijma', 'qiyas'] as const).map((id) => {
		const angle = CELL_CENTER_ANGLES[id];
		return {
			id,
			label: CELL_LABEL_KEYS[id],
			cx: layout.centerX + Math.cos(angle) * r,
			cy: layout.centerY + Math.sin(angle) * r,
			hitRadius: hitR,
		};
	});
}

const cellMembership: CellMembership = (row: LayoutCacheRow) => {
	// Per v6's MIG-029 §ν.3 fallback: invalid / absent → 'quran' (the
	// canonical default per Plan §C.4). v7 inherits this policy.
	switch (row.masadirSource) {
		case 'quran':
		case 'sunnah':
		case 'ijma':
		case 'qiyas':
			return row.masadirSource;
		default:
			return 'quran';
	}
};

export const masadirV7: TraditionModuleV7 = {
	id: 'masadir',
	name: 'masādir',
	shape: 'sectoral',
	cellRegions,
	cellMembership,
	showCalendarRim: false,
	useDensityView: true,
};

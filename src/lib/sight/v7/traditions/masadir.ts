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

/** Same 4-quadrant rotation v6 used (+π/4 offset so quadrant
 *  dividers don't run through the vertical/horizontal axes where
 *  stratum labels live). Cell centers land at compass East / South
 *  / West / North after this rotation. */
const QUADRANT_ROTATION_OFFSET = Math.PI / 4;
const CELL_CENTER_ANGLES = {
	quran: -Math.PI / 2 + QUADRANT_ROTATION_OFFSET + Math.PI / 4, // E
	sunnah: 0 + QUADRANT_ROTATION_OFFSET + Math.PI / 4, // S
	ijma: Math.PI / 2 + QUADRANT_ROTATION_OFFSET + Math.PI / 4, // W
	qiyas: Math.PI + QUADRANT_ROTATION_OFFSET + Math.PI / 4, // N
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

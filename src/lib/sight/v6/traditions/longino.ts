/**
 * MIG-026 Phase δ.2 — Longino tradition (4 critical-contextual-
 * empiricism norms, sectoral shape).
 *
 * Per Concept Paper §4.1 (Modern Western family):
 *   Geometry         4 sectors of 90° each, rotated 45° CW from the
 *                    cardinal vertical axis so dividers don't collide
 *                    with the stratum labels (same +π/8 = 22.5°
 *                    offset principle from §δ.1-fix-1, applied with
 *                    π/4 here because 4-sector wedges need a half-
 *                    wedge offset to avoid cardinal directions):
 *                      East (1-5 o'clock):  venues
 *                      South (4-8 o'clock): uptake
 *                      West (7-11 o'clock): public standards
 *                      North (10-2 o'clock): tempered equality
 *                    Radial within sector = stratum (same encoding
 *                    as Aristotelian); angular within sector = time +
 *                    deterministic jitter.
 *   Cultural framing Modern Western feminist philosophy of science;
 *                    Helen Longino's 4 norms of "Critical Contextual
 *                    Empiricism" (CCE) — the social conditions that
 *                    must obtain for a community's empirical practice
 *                    to count as objective inquiry rather than mere
 *                    consensus.
 *   Citation         Longino, *The Fate of Knowledge* (2002), ch. 5
 *                    (the 4 CCE norms); *Science as Social Knowledge*
 *                    (1990) ch. 4 (the constitutive vs. contextual
 *                    values distinction).
 *
 * 4 CCE norms (Longino 2002):
 *   - venues             — public forums where critiques can be voiced
 *   - uptake             — the community must actually respond to criticism
 *   - public standards   — shared criteria for evaluating theories
 *   - tempered equality  — credentialed disagreement weighted equitably
 *
 * Star sector assignment (Plan §6.2): from a frontmatter `longino_norm`
 * field; default 'venues' if absent. §δ.2 ships with the default-all-
 * to-venues fallback because LayoutCacheRow doesn't yet carry
 * `longinoNorm`; per-note opt-in ships as a §δ.2-fix-N follow-up.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §6.2
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type LonginoNorm = 'venues' | 'uptake' | 'publicStandards' | 'temperedEquality';

/** Sector start angles in canvas math convention. Four 90° sectors
 *  rotated 45° (π/4) CW from the cardinal vertical axis so dividers
 *  fall at NE/SE/SW/NW positions (not at cardinal N/E/S/W axes that
 *  would collide with stratum labels on the +y vertical axis). Same
 *  off-axis principle as §δ.1-fix-1 for Peirce + Habermas, just with
 *  a π/4 offset (half a 90° wedge) instead of π/6 (half a 120° wedge). */
const SECTOR_ARC = Math.PI / 2;
const SECTOR_ROTATION_OFFSET = Math.PI / 4;
const SECTOR_START: Record<LonginoNorm, number> = {
	venues: -Math.PI / 2 + SECTOR_ROTATION_OFFSET,
	uptake: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,
	publicStandards: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,
	temperedEquality: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 3 * SECTOR_ARC,
};

const SECTOR_LABELS: Record<LonginoNorm, string> = {
	venues: 'venues',
	uptake: 'uptake',
	publicStandards: 'public standards',
	temperedEquality: 'tempered equality',
};

const SECTOR_ORDER: LonginoNorm[] = ['venues', 'uptake', 'publicStandards', 'temperedEquality'];

function longinoNormOf(_row: LayoutCacheRow): LonginoNorm {
	return 'venues';
}

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

export const longino: TraditionModule = {
	id: 'longino',
	name: 'Longino',
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const norm = longinoNormOf(row);
		const startAngle = SECTOR_START[norm];

		// Preserve radial distance so stratum encoding reads.
		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const radial = Math.hypot(dx, dy);

		const month = row.createdMonth ?? 0;
		const jitter = pathHash01(row.notePath);
		const monthFraction = (month + jitter) / 12;
		const clamped = 0.03 + monthFraction * 0.94;
		const angle = startAngle + clamped * SECTOR_ARC;

		return {
			x: layout.centerX + Math.cos(angle) * radial,
			y: layout.centerY + Math.sin(angle) * radial,
		};
	},

	sectorDividers: (_layout: TraditionLayout): SectorSpec[] => {
		return SECTOR_ORDER.map((norm): SectorSpec => {
			const start = SECTOR_START[norm];
			return {
				angleStart: start,
				angleEnd: start + SECTOR_ARC,
				label: SECTOR_LABELS[norm],
			};
		});
	},
};

/**
 * MIG-026 Phase ε.1 — Ibn Rushd burhān ladder (4 concentric rings).
 *
 * Per Concept Paper §4.1 (Arabic / Islamic family beyond uṣūl):
 *   Geometry         4 concentric ring zones from center out:
 *                      burhān   (innermost disc, 0-25%) — apodictic
 *                                                          demonstration
 *                      jadal    (inner ring, 25-50%)   — dialectical
 *                                                          argument
 *                      khaṭāba  (middle ring, 50-75%)  — rhetorical
 *                                                          persuasion
 *                      shiʿr    (outer ring, 75-100%)  — poetic
 *                                                          imagination
 *                    Each note is assigned to a ring zone by its
 *                    intended argumentative register.
 *   Cultural framing Islamic Aristotelian commentary tradition; Ibn
 *                    Rushd (Averroes, 1126-1198) ranks Aristotle's
 *                    five demonstrative arts as a hierarchy of
 *                    epistemic certitude, from rigorous proof
 *                    (burhān) inward to imaginative persuasion (shiʿr)
 *                    outward. The geometry visualizes the inward-to-
 *                    outward decay of demonstrative force.
 *   Citation         Ibn Rushd, *Faṣl al-Maqāl* (The Decisive
 *                    Treatise, 1180) §§ 7-15; *Talkhīṣ Manṭiq Arisṭū*
 *                    (Commentary on Aristotle's Organon); Black,
 *                    *Logic and Aristotle's Rhetoric and Poetics in
 *                    Medieval Arabic Philosophy* (1990) ch. 2.
 *
 * Star zone assignment (Plan §7.1): from a frontmatter `burhan_kind`
 * field (values: 'burhan' | 'jadal' | 'khataba' | 'shir'); default
 * 'shir' (outermost, lowest demonstrative force) if absent. §ε.1
 * ships with the default-all-to-shir fallback because LayoutCacheRow
 * doesn't yet carry `burhanKind`; per-note opt-in ships as a §ε.1-fix-N
 * follow-up. Defensible default: most note-taking starts as
 * imaginative association (poetic register) before being reflectively
 * elevated to rhetorical / dialectical / demonstrative status.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §7.1
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	RingSpec,
} from '../types';

type BurhanKind = 0 | 1 | 2 | 3; // burhan / jadal / khataba / shir

const RING_BOUNDARIES_FRAC = [0.25, 0.50, 0.75];

const ZONE_LABELS: Record<BurhanKind, string> = {
	0: 'burhān',
	1: 'jadal',
	2: 'khaṭāba',
	3: 'shiʿr',
};

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

function pathHash01Alt(path: string): number {
	let h = 0xcafebabe;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return (((h >>> 16) & 0xffff)) / 0xffff;
}

/** Determine a note's burhān ladder zone.
 *
 *  Per Plan §7.1: read from frontmatter `burhan_kind` field; default
 *  'shir' (outermost, lowest demonstrative force) if absent.
 *
 *  §ε.1 ship: LayoutCacheRow doesn't yet carry `burhanKind`, so
 *  this unconditionally returns 3 (shiʿr / outermost). Per-note
 *  opt-in ships as a §ε.1-fix-N follow-up.
 */
function burhanKindOf(_row: LayoutCacheRow): BurhanKind {
	return 3;
}

export const ibnRushdBurhan: TraditionModule = {
	id: 'ibn-rushd-burhan',
	name: 'Ibn Rushd burhān',
	shape: 'rings',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const zone = burhanKindOf(row);
		const innerFrac = zone === 0 ? 0 : RING_BOUNDARIES_FRAC[zone - 1];
		const outerFrac = zone === 3 ? 1.0 : RING_BOUNDARIES_FRAC[zone];
		const safeInner = innerFrac + (outerFrac - innerFrac) * 0.10;
		const safeOuter = innerFrac + (outerFrac - innerFrac) * 0.90;
		const jitterR = pathHash01(row.notePath);
		const radialFrac = safeInner + (safeOuter - safeInner) * jitterR;
		const radial = layout.radius * radialFrac;

		const jitterA = pathHash01Alt(row.notePath);
		const angle = jitterA * 2 * Math.PI - Math.PI / 2;

		return {
			x: layout.centerX + radial * Math.cos(angle),
			y: layout.centerY + radial * Math.sin(angle),
		};
	},

	ringBoundaries: (_layout: TraditionLayout): RingSpec[] => {
		// 4 zones requires 3 boundary arcs + a 4th RingSpec at
		// radiusFrac=1.0 to carry the outermost label (drawRing-
		// Boundaries doesn't auto-label the outer rim).
		return [
			{ radiusFrac: RING_BOUNDARIES_FRAC[0], label: ZONE_LABELS[0] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[1], label: ZONE_LABELS[1] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[2], label: ZONE_LABELS[2] },
			{ radiusFrac: 1.0, label: ZONE_LABELS[3] },
		];
	},
};

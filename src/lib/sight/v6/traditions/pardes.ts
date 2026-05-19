/**
 * MIG-026 Phase ζ.1 — PaRDeS (4 concentric rings).
 *
 * Per Concept Paper §4.1 (Jewish / Abrahamic family):
 *   Geometry         4 concentric ring zones from center out:
 *                      peshat  (innermost disc, 0-25%)
 *                                — literal / plain meaning
 *                      remez   (inner ring, 25-50%)
 *                                — allusion / hint
 *                      derash  (middle ring, 50-75%)
 *                                — interpretation / homiletical
 *                      sod     (outer ring, 75-100%)
 *                                — mystical / esoteric
 *                    Each note is assigned to one of the 4 levels of
 *                    interpretation. Inner = literal foundation;
 *                    outer = mystical periphery.
 *   Cultural framing Jewish hermeneutical tradition (medieval, attested
 *                    in Bahir, Zohar, and later Kabbalistic literature).
 *                    PaRDeS (פַּרְדֵּס "orchard", Hebrew acronym from
 *                    Peshat / Remez / Derash / Sod) ranks the four
 *                    levels of Torah interpretation from literal to
 *                    mystical. The concentric-ring geometry visualizes
 *                    the inward-to-outward progression from foundation
 *                    to mystery.
 *   Citation         Bahir §§ 1-12 (early Kabbalistic source for the
 *                    four-level scheme); Moses de León, *Zohar* (~1290)
 *                    introduces the PaRDeS acronym explicitly; Idel,
 *                    *Kabbalah: New Perspectives* (1988) ch. 6.
 *
 * Star zone assignment (Plan §8.1): from a frontmatter `pardes_level`
 * field (values: 'peshat' | 'remez' | 'derash' | 'sod'); default
 * 'peshat' (literal, foundation) if absent. §ζ.1 ships with the
 * default-all-to-peshat fallback because LayoutCacheRow doesn't yet
 * carry `pardesLevel`; per-note opt-in ships as §ζ.1-fix-N follow-up.
 * Defensible default: most note-taking starts as literal/plain
 * reading before being reflectively elevated to remez / derash / sod.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §8.1
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	RingSpec,
} from '../types';

type PardesLevel = 0 | 1 | 2 | 3; // peshat / remez / derash / sod

const RING_BOUNDARIES_FRAC = [0.25, 0.50, 0.75];

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const ZONE_LABELS: Record<PardesLevel, string> = {
	0: 'sight.v6.tradition.canvas.pardes.peshat',
	1: 'sight.v6.tradition.canvas.pardes.remez',
	2: 'sight.v6.tradition.canvas.pardes.derash',
	3: 'sight.v6.tradition.canvas.pardes.sod',
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

function pardesLevelOf(row: LayoutCacheRow): PardesLevel {
	// MIG-029 §ν.3 (2026-05-19) — read from frontmatter `pardes_level`
	// via row.pardesLevel. Allowed values: 'peshat' / 'remez' / 'derash'
	// / 'sod' mapped to ring indices 0..3 respectively. Falls back to
	// peshat (innermost, literal foundation) when absent or invalid.
	switch (row.pardesLevel) {
		case 'peshat': return 0;
		case 'remez':  return 1;
		case 'derash': return 2;
		case 'sod':    return 3;
		default:       return 0;
	}
}

export const pardes: TraditionModule = {
	id: 'pardes',
	name: 'PaRDeS',
	shape: 'rings',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const zone = pardesLevelOf(row);
		const innerFrac = zone === 0 ? 0 : RING_BOUNDARIES_FRAC[zone - 1];
		const outerFrac = zone === 3 ? 1.0 : RING_BOUNDARIES_FRAC[zone];
		const safeInner = innerFrac + (outerFrac - innerFrac) * 0.10;
		const safeOuter = innerFrac + (outerFrac - innerFrac) * 0.90;
		const jitterR = pathHash01(row.notePath);
		const radial = layout.radius * (safeInner + (safeOuter - safeInner) * jitterR);

		const jitterA = pathHash01Alt(row.notePath);
		const angle = jitterA * 2 * Math.PI - Math.PI / 2;

		return {
			x: layout.centerX + radial * Math.cos(angle),
			y: layout.centerY + radial * Math.sin(angle),
		};
	},

	ringBoundaries: (_layout: TraditionLayout): RingSpec[] => {
		return [
			{ radiusFrac: RING_BOUNDARIES_FRAC[0], label: ZONE_LABELS[0] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[1], label: ZONE_LABELS[1] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[2], label: ZONE_LABELS[2] },
			{ radiusFrac: 1.0, label: ZONE_LABELS[3] },
		];
	},
};

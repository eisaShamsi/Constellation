/**
 * MIG-026 Phase δ.2 — Husserl tradition (4 regional ontologies,
 * concentric-rings shape).
 *
 * Per Concept Paper §4.1 (Modern Western family) + orientation v2.10
 * geometric-shape audit:
 *   Geometry         4 concentric ring zones from center out:
 *                      Center (innermost disc): formal ontology
 *                      Inner ring (25-50%):     material nature
 *                      Middle ring (50-75%):    animal nature
 *                      Outer ring (75-100%):    spirit / Geist
 *                    Each note assigned to a zone is placed within
 *                    that zone's annulus at a deterministic angular
 *                    position (no specific angular encoding — the
 *                    sequence within an annulus is arbitrary).
 *   Cultural framing Phenomenological epistemology; Edmund Husserl's
 *                    regional ontologies introduced in *Ideen* (1913).
 *                    The formal ontology is the science of any-object-
 *                    whatsoever; the three material regional ontologies
 *                    (material nature, animal nature, Geist) are the
 *                    irreducible kinds of being whose essences are
 *                    given in eidetic intuition.
 *   Citation         Husserl, *Ideen I* (1913) §§ 9-17 (formal vs.
 *                    material ontology); *Ideen II* (1952) intro
 *                    (the three material regions); Smith & Smith,
 *                    *The Cambridge Companion to Husserl* (1995) ch. 3.
 *
 * Star zone assignment (Plan §6.2): from a frontmatter `husserl_region`
 * field (values: 'formal' | 'material-nature' | 'animal-nature' |
 * 'spirit'); default 'formal' if absent. §δ.2 ships with the default-
 * all-to-formal fallback because LayoutCacheRow doesn't yet carry
 * `husserlRegion`; per-note opt-in ships as a §δ.2-fix-N follow-up.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §6.2
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	RingSpec,
} from '../types';

type HusserlRegion = 0 | 1 | 2 | 3; // formal / material-nature / animal-nature / spirit

/** Ring boundary radial fractions. 4 zones requires 3 boundaries +
 *  the outer rim. Zone 0 (formal) sits in 0.00-0.25; zone 1 in
 *  0.25-0.50; zone 2 in 0.50-0.75; zone 3 in 0.75-1.00. */
const RING_BOUNDARIES_FRAC = [0.25, 0.50, 0.75];

const ZONE_LABELS: Record<HusserlRegion, string> = {
	0: 'formal ontology',
	1: 'material nature',
	2: 'animal nature',
	3: 'spirit · Geist',
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

/** Determine a note's Husserl regional ontology.
 *
 *  Per Plan §6.2: read from frontmatter `husserl_region` field;
 *  default 'formal' (zone 0) if absent.
 *
 *  §δ.2 ship: LayoutCacheRow doesn't yet carry `husserlRegion`, so
 *  this unconditionally returns 0. Defensible default: formal
 *  ontology is the science of any-object-whatsoever (Husserl 1913
 *  §10) — every note's content is at minimum "an object" subject
 *  to the formal categories before being further classified into
 *  a material region.
 */
function husserlRegionOf(_row: LayoutCacheRow): HusserlRegion {
	return 0;
}

export const husserl: TraditionModule = {
	id: 'husserl',
	name: 'Husserl',
	shape: 'rings',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const region = husserlRegionOf(row);
		// Place star within its zone's annulus:
		//   - radial: jittered within the zone's radial band
		//   - angular: jittered around full circle
		const innerFrac = region === 0 ? 0 : RING_BOUNDARIES_FRAC[region - 1];
		const outerFrac = region === 3 ? 1.0 : RING_BOUNDARIES_FRAC[region];
		// Inset 10% from boundaries so stars don't kiss the ring strokes.
		const safeInner = innerFrac + (outerFrac - innerFrac) * 0.10;
		const safeOuter = innerFrac + (outerFrac - innerFrac) * 0.90;
		const jitterR = pathHash01(row.notePath);
		const radialFrac = safeInner + (safeOuter - safeInner) * jitterR;
		const radial = layout.radius * radialFrac;

		const jitterA = pathHash01Alt(row.notePath);
		const angle = jitterA * 2 * Math.PI - Math.PI / 2; // full circle starting at top

		return {
			x: layout.centerX + radial * Math.cos(angle),
			y: layout.centerY + radial * Math.sin(angle),
		};
	},

	ringBoundaries: (_layout: TraditionLayout): RingSpec[] => {
		// 3 boundary arcs at 25/50/75% radius. Labels placed by
		// drawRingBoundaries at the midpoint of each annulus along the
		// +x axis (east, 3 o'clock) so they don't collide with stratum
		// labels on the +y axis. The first label (formal ontology) sits
		// inside the central disc; the rest sit inside their annuli.
		return [
			{ radiusFrac: RING_BOUNDARIES_FRAC[0], label: ZONE_LABELS[0] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[1], label: ZONE_LABELS[1] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[2], label: ZONE_LABELS[2] },
			{ radiusFrac: 1.0, label: ZONE_LABELS[3] },
		];
	},
};

/**
 * MIG-026 Phase θ.3 — Maldonado-Torres coloniality (rings, 3 concentric).
 *
 * Per Concept Paper §4.1 (Latin American decolonial family):
 *   Geometry         3 concentric ring zones, each one tier of
 *                    coloniality per Maldonado-Torres's three-fold
 *                    extension of Quijano's coloniality of power:
 *                      coloniality of power      (innermost, 0-33%)
 *                                                — political /
 *                                                  economic structure
 *                      coloniality of knowledge  (middle, 33-67%)
 *                                                — epistemic /
 *                                                  knowledge-producing
 *                      coloniality of being      (outermost, 67-100%)
 *                                                — ontological /
 *                                                  existential
 *   Cultural framing Latin American decolonial theory; Nelson
 *                    Maldonado-Torres extends Aníbal Quijano's
 *                    "coloniality of power" thesis into a three-tier
 *                    framework that captures how colonial structures
 *                    persist after formal decolonization at the
 *                    levels of political authority, knowledge
 *                    production, and being itself.
 *   Citation         Maldonado-Torres, "On the Coloniality of Being"
 *                    (*Cultural Studies* 21, 2007); Quijano,
 *                    "Coloniality of Power" (*Nepantla* 1, 2000).
 *
 * Star tier assignment (Plan §10.3): from a frontmatter
 * `coloniality_tier` field; default to hash-distributed across all 3
 * tiers if absent. §θ.3 ships with hash fallback so all 3 zones
 * populate visibly.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §10.3
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	RingSpec,
} from '../types';

type ColonialityTier = 0 | 1 | 2; // power / knowledge / being

const RING_BOUNDARIES_FRAC = [0.33, 0.67];

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const TIER_LABELS: Record<ColonialityTier, string> = {
	0: 'sight.v6.tradition.canvas.maldonado-torres.power',
	1: 'sight.v6.tradition.canvas.maldonado-torres.knowledge',
	2: 'sight.v6.tradition.canvas.maldonado-torres.being',
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

function tierOf(row: LayoutCacheRow): ColonialityTier {
	const bucket = Math.floor(pathHash01(row.notePath) * 3);
	return (bucket >= 2 ? 2 : (bucket <= 0 ? 0 : 1)) as ColonialityTier;
}

export const maldonadoTorres: TraditionModule = {
	id: 'maldonado-torres',
	name: 'Maldonado-Torres',
	shape: 'rings',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const tier = tierOf(row);
		const innerFrac = tier === 0 ? 0 : RING_BOUNDARIES_FRAC[tier - 1];
		const outerFrac = tier === 2 ? 1.0 : RING_BOUNDARIES_FRAC[tier];
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
			{ radiusFrac: RING_BOUNDARIES_FRAC[0], label: TIER_LABELS[0] },
			{ radiusFrac: RING_BOUNDARIES_FRAC[1], label: TIER_LABELS[1] },
			{ radiusFrac: 1.0, label: TIER_LABELS[2] },
		];
	},
};

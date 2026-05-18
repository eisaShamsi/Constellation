/**
 * MIG-026 Phase θ.1 — Mignolo pluriversal (relational hub-and-spoke).
 *
 * Per Concept Paper §4.1 (Latin American decolonial family):
 *   Geometry         Central hub disc (modernity / totality) + 5
 *                    outer cluster bubbles connected to hub by
 *                    spoke lines. Each cluster represents a
 *                    "decolonial option" — a position from which
 *                    modernity/coloniality can be challenged.
 *   Cultural framing Latin American decolonial theory; Walter
 *                    Mignolo (b. 1941) advances "pluriversality" —
 *                    the rejection of a single Western universal in
 *                    favor of multiple coexisting epistemic worlds.
 *                    The hub-and-spoke geometry visualizes modernity
 *                    as one position among many, not as the
 *                    encompassing whole.
 *   Citation         Mignolo, *The Darker Side of Western Modernity*
 *                    (2011) ch. 2 + 7; *Local Histories / Global
 *                    Designs* (2000); Mignolo & Walsh, *On
 *                    Decoloniality* (2018) intro.
 *
 * Five cluster labels (Mignolo's decolonial vocabulary, NOT specific
 * indigenous traditions — those are excluded from MIG-026 per Eisa's
 * blanket-exclusion ruling on the Indigenous family):
 *   epistemic disobedience  — refusal to accept modernity's claim
 *                              to universal validity
 *   border thinking          — knowing from the colonial difference
 *   delinking                 — releasing from imposed cosmology
 *   decolonial gnosis         — knowledge from the perspective of
 *                              the colonized
 *   pluriversal world         — the meta-position itself (many worlds)
 *
 * Star cluster assignment (Plan §10.1): from a frontmatter
 * `mignolo_position` field; default-distribute across all 5 clusters
 * by hash if absent. §θ.1 ships with hash fallback so every cluster
 * bubble has at least some stars near it for visual context.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §10.1
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	RelationalSpec,
} from '../types';

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const CLUSTER_LABELS = [
	'sight.v6.tradition.canvas.mignolo-pluriversal.epistemicDisobedience',
	'sight.v6.tradition.canvas.mignolo-pluriversal.borderThinking',
	'sight.v6.tradition.canvas.mignolo-pluriversal.delinking',
	'sight.v6.tradition.canvas.mignolo-pluriversal.decolonialGnosis',
	'sight.v6.tradition.canvas.mignolo-pluriversal.pluriversalWorld',
];

const N_CLUSTERS = CLUSTER_LABELS.length; // 5

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

function clusterIndexOf(row: LayoutCacheRow): number {
	const bucket = Math.floor(pathHash01(row.notePath) * N_CLUSTERS);
	return bucket >= N_CLUSTERS ? N_CLUSTERS - 1 : (bucket < 0 ? 0 : bucket);
}

export const mignoloPluriversal: TraditionModule = {
	id: 'mignolo-pluriversal',
	name: 'Mignolo pluriversal',
	shape: 'relational',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const cluster = clusterIndexOf(row);
		// Match the ring + cluster geometry used by drawRelationalGraph
		// so stars cluster near their bubble.
		const ringRadius = layout.radius * 0.65;
		const startAngle = -Math.PI / 2 + Math.PI / N_CLUSTERS;
		const a = startAngle + (cluster * 2 * Math.PI) / N_CLUSTERS;
		// Bubble centerpoint
		const bx = layout.centerX + ringRadius * Math.cos(a);
		const by = layout.centerY + ringRadius * Math.sin(a);
		// Star jitter within a small disc (~12% of dome radius) around
		// the bubble center
		const jitterTheta = pathHash01(row.notePath) * 2 * Math.PI;
		const jitterR = pathHash01Alt(row.notePath) * layout.radius * 0.12;
		return {
			x: bx + jitterR * Math.cos(jitterTheta),
			y: by + jitterR * Math.sin(jitterTheta),
		};
	},

	relationalSpec: (_layout: TraditionLayout): RelationalSpec => {
		return {
			variant: 'hub-and-spoke',
			hubLabel: 'sight.v6.tradition.canvas.mignolo-pluriversal.modernityTotality',
			clusters: CLUSTER_LABELS.map((label) => ({ label })),
		};
	},
};

/**
 * MIG-026 Phase θ.5 — Ibuanyidanda (relational hub-and-spoke).
 *
 * Per Concept Paper §4.1 (African philosophical family):
 *   Geometry         Central hub disc ("missing link") + 5 outer
 *                    cluster bubbles connected to hub by spoke lines.
 *                    Same relational primitive as Mignolo
 *                    pluriversal (Phase θ.1) — reuses
 *                    drawRelationalGraph. Conceptually inverse to
 *                    Mignolo: where Mignolo's hub is the dominant
 *                    pole (modernity/totality) and clusters challenge
 *                    it, Asouzu's hub is the UNIFYING absence
 *                    (missing link) that every entity must
 *                    complement in order to be.
 *   Cultural framing African philosophy of complementarity; Innocent
 *                    Asouzu (b. 1952) developed "ibuanyidanda"
 *                    (Igbo: "no load is insurmountable for ndida")
 *                    as a complementary ontology — every being is
 *                    constituted through its complementary relations
 *                    with other beings, mediated by the missing link.
 *                    The hub-and-spoke geometry visualizes each
 *                    entity-cluster's necessary relation to the
 *                    unifying complementarity.
 *   Citation         Asouzu, *Ibuanyidanda: New Complementary
 *                    Ontology* (2007); *The Method and Principles
 *                    of Complementary Reflection* (2004); Edeh,
 *                    *Igbo Metaphysics* (1985) — predecessor.
 *
 * Five cluster labels (Asouzu's complementary-ontology vocabulary,
 * abstracted into generic relational categories appropriate to the
 * hub-and-spoke geometric form):
 *   self           — the individuating pole
 *   other          — the dialogical pole
 *   community      — the collective pole
 *   tradition      — the temporal-historical pole
 *   transcendence  — the metaphysical-religious pole
 *
 * Star cluster assignment (Plan §10.5): from a frontmatter
 * `ibuanyidanda_pole` field; default to hash-distributed across all
 * 5 clusters if absent. §θ.5 ships with hash fallback so every
 * cluster bubble populates visibly.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §10.5
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	RelationalSpec,
} from '../types';

const CLUSTER_LABELS = [
	'self',
	'other',
	'community',
	'tradition',
	'transcendence',
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

export const ibuanyidanda: TraditionModule = {
	id: 'ibuanyidanda',
	name: 'Ibuanyidanda',
	shape: 'relational',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const cluster = clusterIndexOf(row);
		const ringRadius = layout.radius * 0.65;
		const startAngle = -Math.PI / 2 + Math.PI / N_CLUSTERS;
		const a = startAngle + (cluster * 2 * Math.PI) / N_CLUSTERS;
		const bx = layout.centerX + ringRadius * Math.cos(a);
		const by = layout.centerY + ringRadius * Math.sin(a);
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
			hubLabel: 'missing link',
			clusters: CLUSTER_LABELS.map((label) => ({ label })),
		};
	},
};

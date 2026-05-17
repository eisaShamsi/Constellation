/**
 * MIG-026 Phase θ.2 — Dussel transmodernity (binary-flow concentric).
 *
 * Per Concept Paper §4.1 (Latin American decolonial family):
 *   Geometry         Concentric binary-flow: inner disc = totality
 *                    (modernity's encompassing system) + outer ring
 *                    = exteriority (what totality excludes — the
 *                    poor, the colonized, the Other). Bidirectional
 *                    radial arrows convey the "analectic moment" —
 *                    the interruption by exteriority that opens
 *                    transmodernity, where excluded knowledge re-
 *                    enters and transforms totality.
 *   Cultural framing Latin American liberation philosophy; Enrique
 *                    Dussel (1934-2023) advances "transmodernity" as
 *                    going BEYOND postmodernism by recovering the
 *                    excluded exteriority of modernity. Not anti-
 *                    modern but post-Eurocentric; the analectic
 *                    method moves from exteriority back into
 *                    totality to transform it.
 *   Citation         Dussel, *Filosofía de la Liberación* (1977);
 *                    *Hacia un Marx desconocido* (1988); Mendieta
 *                    (ed.), *The Underside of Modernity* (1996).
 *
 * Star side assignment (Plan §10.2): from a frontmatter `dussel_pole`
 * field (values: 'totality' | 'exteriority'); default 'exteriority'
 * (the privileged pole in Dussel's framework — the source of
 * critique) if absent. §θ.2 ships with hash-bucket 50/50 so both
 * zones populate visibly for testing.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §10.2
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	BinaryFlowSpec,
} from '../types';

type DusselPole = 'totality' | 'exteriority';

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

function dusselPoleOf(row: LayoutCacheRow): DusselPole {
	return pathHash01(row.notePath) < 0.5 ? 'totality' : 'exteriority';
}

export const dusselTransmodernity: TraditionModule = {
	id: 'dussel-transmodernity',
	name: 'Dussel transmodernity',
	shape: 'binary-flow',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const pole = dusselPoleOf(row);
		// Match the concentric layout in drawBinaryFlowConcentric:
		// totality = inner disc (0-40%), exteriority = outer annulus (40-95%).
		const innerFrac = pole === 'totality' ? 0 : 0.40;
		const outerFrac = pole === 'totality' ? 0.40 : 1.0;
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

	binaryFlowSpec: (_layout: TraditionLayout): BinaryFlowSpec => {
		return {
			cellA: { label: 'totality' },
			cellB: { label: 'exteriority' },
			flowDirection: 'bidirectional',
			layout: 'concentric',
		};
	},
};

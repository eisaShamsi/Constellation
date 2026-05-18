/**
 * MIG-026 Phase δ.2 — Dewey tradition (5-stage pattern of inquiry,
 * cyclic-flow shape).
 *
 * Per Concept Paper §4.1 (Modern Western family):
 *   Geometry         5-segment ring with clockwise flow arrows
 *                    indicating sequence. The 5 stages are Dewey's
 *                    canonical pattern of inquiry, in order:
 *                      1. indeterminate situation (12 o'clock)
 *                      2. problem definition (2:30 area)
 *                      3. hypothesis (5 o'clock area)
 *                      4. reasoning (7:30 area)
 *                      5. testing (9:30 area, wraps to indeterminate)
 *                    Stars assigned to a stage are placed inside that
 *                    stage's arc-segment at a radial position within
 *                    the segment's band.
 *   Cultural framing American pragmatist epistemology; John Dewey's
 *                    pattern-of-inquiry from *Logic: The Theory of
 *                    Inquiry* (1938). Dewey's central claim: knowledge
 *                    is the resolution of an indeterminate situation
 *                    through reflective inquiry, with the 5 stages
 *                    forming a cyclic pattern (the testing stage
 *                    typically opens new indeterminate situations,
 *                    re-entering the cycle).
 *   Citation         Dewey, *Logic: The Theory of Inquiry* (1938),
 *                    ch. 6 ("The Pattern of Inquiry"); Hildebrand,
 *                    "John Dewey" (SEP, 2018) §3.4.
 *
 * Star stage assignment (Plan §6.2): from a frontmatter
 * `dewey_stage` field (values: 'indeterminate' | 'problem' |
 * 'hypothesis' | 'reasoning' | 'testing'); default 'indeterminate'
 * if absent. §δ.2 ships with the default-all-to-indeterminate
 * fallback because LayoutCacheRow doesn't yet carry `deweyStage`;
 * per-note opt-in ships as a §δ.2-fix-N follow-up.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §6.2
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	CyclicFlowSpec,
} from '../types';

type DeweyStage = 0 | 1 | 2 | 3 | 4;

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const STAGE_LABELS = [
	'sight.v6.tradition.canvas.dewey.indeterminate',
	'sight.v6.tradition.canvas.dewey.problem',
	'sight.v6.tradition.canvas.dewey.hypothesis',
	'sight.v6.tradition.canvas.dewey.reasoning',
	'sight.v6.tradition.canvas.dewey.testing',
] as const;

const STAGE_ARC = (2 * Math.PI) / 5;
const STAGE_START = -Math.PI / 2; // First stage starts at 12 o'clock

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

/** Determine a note's Dewey inquiry stage.
 *
 *  Per Plan §6.2: read from frontmatter `dewey_stage` field; default
 *  'indeterminate' (stage 0) if absent.
 *
 *  §δ.2 ship: LayoutCacheRow doesn't yet carry `deweyStage`, so this
 *  unconditionally returns 0. Defensible default: most note-taking
 *  starts as recording observations / encountering puzzles before
 *  reflectively classifying which inquiry stage applies.
 */
function deweyStageOf(_row: LayoutCacheRow): DeweyStage {
	return 0;
}

export const dewey: TraditionModule = {
	id: 'dewey',
	name: 'Dewey',
	shape: 'cyclic-flow',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const stage = deweyStageOf(row);
		const segmentStart = STAGE_START + stage * STAGE_ARC;
		// Within-segment: jittered angle within the stage's arc + slight
		// radial jitter around the 75% ring band so stars cluster
		// inside their stage's wedge of the ring.
		const jitterA = pathHash01(row.notePath);
		const angle = segmentStart + (0.1 + jitterA * 0.8) * STAGE_ARC; // 10-90% into wedge
		// Radial jitter: 65-85% of dome radius (band around the 75% ring).
		const jitterR = pathHash01(row.notePath + ':r');
		const radial = layout.radius * (0.65 + jitterR * 0.20);
		return {
			x: layout.centerX + radial * Math.cos(angle),
			y: layout.centerY + radial * Math.sin(angle),
		};
	},

	cyclicFlowSpec: (_layout: TraditionLayout): CyclicFlowSpec => {
		return {
			segments: STAGE_LABELS.map((label) => ({ label })),
		};
	},
};

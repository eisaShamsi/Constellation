/**
 * MIG-026 Phase ζ.2 — Maimonidean prophecy (spiral ladder, 11 steps).
 *
 * Per Concept Paper §4.1 (Jewish / Abrahamic family):
 *   Geometry         Logarithmic-style spiral from near-center
 *                    outward across 1.5 turns. 11 step-marks evenly
 *                    spaced along the spiral, each labeled by its
 *                    prophetic degree per Maimonides' enumeration in
 *                    the *Guide of the Perplexed*. Inner = lowest
 *                    degree (holy spirit); outer = highest degree
 *                    (face-to-face vision, reserved historically to
 *                    Moses alone per Maimonides).
 *   Cultural framing Medieval Jewish philosophical theology; Moses
 *                    Maimonides (1138-1204) hierarchically ranks 11
 *                    levels of prophetic experience in *Guide* II:45.
 *                    The spiral geometry visualizes the ascending
 *                    progression from minimal divine inspiration to
 *                    full apprehension.
 *   Citation         Maimonides, *Guide of the Perplexed* II:45;
 *                    tr. Pines (1963) pp. 395-403; Kreisel,
 *                    *Prophecy: The History of an Idea in Medieval
 *                    Jewish Philosophy* (2001) ch. 4.
 *
 * Eleven degrees of prophecy (Maimonides 1190, *Guide* II:45):
 *   1.  ruaḥ ha-qodesh    — holy spirit (divine inspiration)
 *   2.  sleep visions     — figured imagery in dream
 *   3.  daytime allegory  — figured imagery while awake
 *   4.  voice in vision   — auditory experience within vision
 *   5.  speaking person   — human-form figure addressing prophet
 *   6.  speaking angel    — angelic figure addressing prophet
 *   7.  speaking man-form — God-form figure (vision)
 *   8.  allegory awake    — full allegorical vision while awake
 *   9.  voice while awake — auditory prophecy not in vision
 *  10.  man-form awake    — anthropomorphic vision while awake
 *  11.  angel awake       — angel vision while awake (Moses' level)
 *
 * Star step assignment (Plan §8.2): from a frontmatter
 * `maimonidean_degree` field (1-11); default-distribute across all
 * 11 steps by hash if absent. §ζ.2 ships with HASH-BUCKETED fallback
 * so all 11 step positions populate visibly — a single-default would
 * crowd one step.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §8.2
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	LadderSpec,
} from '../types';

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const STEP_LABELS = [
	'sight.v6.tradition.canvas.maimonidean-prophecy.ruahHaQodesh',
	'sight.v6.tradition.canvas.maimonidean-prophecy.sleepVision',
	'sight.v6.tradition.canvas.maimonidean-prophecy.daytimeAllegory',
	'sight.v6.tradition.canvas.maimonidean-prophecy.voiceInVision',
	'sight.v6.tradition.canvas.maimonidean-prophecy.speakingPerson',
	'sight.v6.tradition.canvas.maimonidean-prophecy.speakingAngel',
	'sight.v6.tradition.canvas.maimonidean-prophecy.speakingManForm',
	'sight.v6.tradition.canvas.maimonidean-prophecy.allegoryAwake',
	'sight.v6.tradition.canvas.maimonidean-prophecy.voiceWhileAwake',
	'sight.v6.tradition.canvas.maimonidean-prophecy.manFormAwake',
	'sight.v6.tradition.canvas.maimonidean-prophecy.angelAwake',
];

const N_STEPS = STEP_LABELS.length; // 11

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

/** Determine a note's Maimonidean prophetic degree.
 *
 *  Per Plan §8.2: read from frontmatter `maimonidean_degree` field
 *  (integer 1-11); default to hash-distributed if absent. §ζ.2 ships
 *  with the hash fallback so all 11 step positions are visible at
 *  test time.
 */
function maimonideanDegreeOf(row: LayoutCacheRow): number {
	const bucket = Math.floor(pathHash01(row.notePath) * N_STEPS);
	return bucket >= N_STEPS ? N_STEPS - 1 : (bucket < 0 ? 0 : bucket);
}

export const maimonideanProphecy: TraditionModule = {
	id: 'maimonidean-prophecy',
	name: 'Maimonidean prophecy',
	shape: 'ladder',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		// Match the spiral parameters used by drawLadderSteps so stars
		// land near their step's position on the spiral curve.
		const totalAngle = 3 * Math.PI;
		const innerR = layout.radius * 0.05;
		const outerR = layout.radius * 0.95;
		const a = innerR;
		const b = (outerR - innerR) / totalAngle;

		const degree = maimonideanDegreeOf(row);
		const stepTheta = (degree / (N_STEPS - 1)) * totalAngle;

		// Jitter within ±0.4 rad of step theta and ±15% of radial range
		// so stars cluster near their step but don't perfectly stack.
		const jitterA = pathHash01Alt(row.notePath);
		const thetaJitter = (jitterA - 0.5) * 0.8; // ~±0.4 rad
		const theta = Math.max(0, Math.min(totalAngle, stepTheta + thetaJitter));

		const r = a + b * theta;
		const jitterR = pathHash01(row.notePath + ':r');
		const radial = r * (0.92 + jitterR * 0.16); // ~±8% radial wobble

		return {
			x: layout.centerX + radial * Math.cos(theta - Math.PI / 2),
			y: layout.centerY + radial * Math.sin(theta - Math.PI / 2),
		};
	},

	ladderSteps: (_layout: TraditionLayout): LadderSpec => {
		return {
			variant: 'spiral',
			steps: STEP_LABELS.map((label) => ({ label })),
		};
	},
};

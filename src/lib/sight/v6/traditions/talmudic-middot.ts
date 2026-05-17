/**
 * MIG-026 Phase ζ.3 — Talmudic 13 middot (spiral ladder, 13 steps).
 *
 * Per Concept Paper §4.1 (Jewish / Abrahamic family):
 *   Geometry         Spiral ladder (same primitive as Maimonidean
 *                    ζ.2) with 13 step-marks instead of 11. Each
 *                    step is one of Rabbi Yishmael's hermeneutical
 *                    rules for interpreting Torah. Inner = simplest
 *                    inference (kal va-chomer); outer = most
 *                    complex (reconciliation of contradictory verses).
 *   Cultural framing Classical Talmudic hermeneutics; the 13 middot
 *                    (rules) attributed to Rabbi Yishmael ben
 *                    Elisha (~3rd c. CE), preserved in the Baraita
 *                    d'Rabbi Yishmael at the opening of Sifra
 *                    (halakhic midrash on Leviticus). These rules
 *                    are recited daily in the morning liturgy as
 *                    part of the introductory section.
 *   Citation         Sifra, introduction (Baraita d'Rabbi Yishmael);
 *                    Strack & Stemberger, *Introduction to the
 *                    Talmud and Midrash* (1992) §§ 25-30; Yadin-
 *                    Israel, *Rabbi Yishmael* (2014) ch. 3.
 *
 * The 13 middot (Rabbi Yishmael, in canonical order):
 *   1.  kal va-chomer            — a fortiori inference (light → heavy)
 *   2.  gezera shava             — verbal analogy between verses
 *   3.  binyan av (1 verse)      — principle established from one verse
 *   4.  binyan av (2 verses)     — principle from two verses
 *   5.  kelal u-perat             — general → particular
 *   6.  perat u-kelal             — particular → general
 *   7.  kelal-perat-kelal         — general-particular-general
 *   8.  special purpose           — general extracted to teach specific case
 *   9.  extracted (lighter)       — extracted item with lighter rule
 *  10.  extracted (different)     — extracted item with different rule
 *  11.  from context              — meaning derived from passage context
 *  12.  from end                  — meaning derived from later passage
 *  13.  reconciliation            — two contradictory verses harmonized by a third
 *
 * Star step assignment (Plan §8.3): from a frontmatter `middah`
 * field (integer 1-13); default to hash-distributed if absent.
 * §ζ.3 ships with HASH-BUCKETED fallback so all 13 step positions
 * populate visibly.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §8.3
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	LadderSpec,
} from '../types';

const STEP_LABELS = [
	'1 · kal va-chomer',
	'2 · gezera shava',
	'3 · binyan av (1 verse)',
	'4 · binyan av (2 verses)',
	'5 · kelal u-perat',
	'6 · perat u-kelal',
	'7 · k-p-k',
	'8 · special purpose',
	'9 · extracted (lighter)',
	'10 · extracted (different)',
	'11 · from context',
	'12 · from end',
	'13 · reconciliation',
];

const N_STEPS = STEP_LABELS.length; // 13

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

function middahOf(row: LayoutCacheRow): number {
	const bucket = Math.floor(pathHash01(row.notePath) * N_STEPS);
	return bucket >= N_STEPS ? N_STEPS - 1 : (bucket < 0 ? 0 : bucket);
}

export const talmudicMiddot: TraditionModule = {
	id: 'talmudic-middot',
	name: 'Talmudic 13 middot',
	shape: 'ladder',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		// Match the spiral parameters used by drawLadderSteps in anchor.ts
		// so stars land near their step's position on the spiral curve.
		// Same parameters as Maimonidean (ζ.2): 1.5 turns, 0.05R → 0.95R.
		const totalAngle = 3 * Math.PI;
		const innerR = layout.radius * 0.05;
		const outerR = layout.radius * 0.95;
		const a = innerR;
		const b = (outerR - innerR) / totalAngle;

		const middah = middahOf(row);
		const stepTheta = (middah / (N_STEPS - 1)) * totalAngle;

		// Jitter within ±0.3 rad of step theta (tighter than Maimonidean's
		// ±0.4 because 13 steps are more densely packed along the same
		// spiral) and ±15% radial wobble.
		const jitterA = pathHash01Alt(row.notePath);
		const thetaJitter = (jitterA - 0.5) * 0.6; // ~±0.3 rad
		const theta = Math.max(0, Math.min(totalAngle, stepTheta + thetaJitter));

		const r = a + b * theta;
		const jitterR = pathHash01(row.notePath + ':r');
		const radial = r * (0.92 + jitterR * 0.16);

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

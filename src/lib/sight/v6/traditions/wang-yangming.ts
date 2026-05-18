/**
 * MIG-026 Phase η.2 — Wang Yangming (binary-flow vertical with central
 * liángzhī).
 *
 * Per Concept Paper §4.1 (East Asian Confucian family):
 *   Geometry         Vertical-layout binary flow: cellA on the LEFT
 *                    (zhī 知 — knowing) and cellB on the RIGHT
 *                    (xíng 行 — acting), with central liángzhī (良知
 *                    — innate moral knowing) mediating between them.
 *                    Bidirectional arrows on each side of center
 *                    convey the unity of knowing-and-acting through
 *                    innate moral apprehension.
 *   Cultural framing Late Ming Neo-Confucian philosophy; Wang
 *                    Yangming (1472-1529) opposed the bifurcation of
 *                    knowledge and action that he attributed to the
 *                    Zhu Xi school. His central doctrine —
 *                    *zhī-xíng héyī* (知行合一, "knowing and acting
 *                    are one") — holds that genuine moral knowledge
 *                    is constituted by action, and genuine action
 *                    embodies knowledge. The central liángzhī
 *                    (innate moral knowing) is what unifies them.
 *   Citation         Wang Yangming, *Chuán Xí Lù* (傳習錄
 *                    "Instructions for Practical Living", 1518),
 *                    Book I §§ 5-10, 26; Ivanhoe, *Readings from the
 *                    Lu-Wang School of Neo-Confucianism* (2009).
 *
 * Star side assignment (Plan §9.2): from a frontmatter
 * `wang_yangming_axis` field (values: 'zhi' | 'xing'); default to
 * hash-bucket 50/50 if absent. §η.2 ships with the hash fallback so
 * both sides populate visibly.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §9.2
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	BinaryFlowSpec,
} from '../types';

type WangAxis = 'zhi' | 'xing';

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

function wangAxisOf(row: LayoutCacheRow): WangAxis {
	return pathHash01(row.notePath) < 0.5 ? 'zhi' : 'xing';
}

export const wangYangming: TraditionModule = {
	id: 'wang-yangming',
	name: 'Wang Yangming',
	shape: 'binary-flow',

	remapStarPosition: (row: LayoutCacheRow, _defaultPos, layout: TraditionLayout) => {
		const axis = wangAxisOf(row);
		// Left half (zhī) = x in [centerX - r, centerX]
		// Right half (xíng) = x in [centerX, centerX + r]
		// Inset 10% from edges + small gap around center so stars
		// don't overlap with axis labels or center liángzhī.
		const sideSign = axis === 'zhi' ? -1 : 1;
		const jitterX = pathHash01(row.notePath);
		const xOffsetFrac = 0.10 + jitterX * 0.80; // 10-90% from center toward edge
		const x = layout.centerX + sideSign * layout.radius * xOffsetFrac;

		// Vertical: full dome height available. Clipped to circle bound
		// at the given x with 10% inset.
		const dx = x - layout.centerX;
		const halfHeightAtX = Math.sqrt(
			Math.max(0, layout.radius * layout.radius - dx * dx),
		);
		const safeHalfHeight = halfHeightAtX * 0.90;
		const jitterY = pathHash01Alt(row.notePath);
		const y = layout.centerY + (jitterY - 0.5) * 2 * safeHalfHeight;

		return { x, y };
	},

	binaryFlowSpec: (_layout: TraditionLayout): BinaryFlowSpec => {
		// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
		return {
			cellA: { label: 'sight.v6.tradition.canvas.wang-yangming.zhi' },
			cellB: { label: 'sight.v6.tradition.canvas.wang-yangming.xing' },
			flowDirection: 'bidirectional',
			centerLabel: 'sight.v6.tradition.canvas.wang-yangming.liangzhi',
			layout: 'vertical',
		};
	},
};

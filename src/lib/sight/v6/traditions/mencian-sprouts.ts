/**
 * MIG-026 Phase η.1 — Mencian 4 sprouts (sectoral 4-cell with optional
 * central 5th virtue).
 *
 * Per Concept Paper §4.1 (East Asian Confucian family):
 *   Geometry         4 sectors of 90° each rotated π/4 CW from the
 *                    cardinal vertical axis (Longino pattern), plus an
 *                    optional small central ring at 15% radius labeling
 *                    the 5th virtue xìn (信 trustworthiness, added by
 *                    later Han-dynasty commentators to Mencius's
 *                    original 4). Each sector represents one of the
 *                    4 moral sprouts (端 duān) per Mencius II.A.6.
 *   Cultural framing Classical Confucian moral psychology; Mengzi
 *                    (Mencius, 372-289 BCE) argued that humans
 *                    possess innate moral sprouts (端 duān) that,
 *                    when cultivated, develop into the four cardinal
 *                    virtues. The geometry visualizes each note as
 *                    expressing one of these sprouts.
 *   Citation         *Mencius* (孟子) II.A.6 ("Gōngsūn Chǒu" 公孫丑);
 *                    Van Norden, *Mengzi: With Selections from
 *                    Traditional Commentaries* (2008) pp. 46-49;
 *                    Ivanhoe, *Ethics in the Confucian Tradition*
 *                    (2002) ch. 1.
 *
 * The 4 sprouts (端) and corresponding cardinal virtues:
 *   惻隱  cèyǐn   — compassion       → 仁 rén  (benevolence)
 *   羞惡  xiūwù   — shame              → 義 yì   (righteousness)
 *   辭讓  círàng  — deference          → 禮 lǐ   (propriety)
 *   是非  shìfēi  — sense of right/wrong → 智 zhì  (wisdom)
 *
 * Central 5th virtue (Han addition):
 *   信   xìn     — trustworthiness (the binding virtue underlying
 *                                    the other four)
 *
 * Star sector assignment (Plan §9.1): from a frontmatter `mencian_sprout`
 * field; default 'ceyin' (compassion) if absent. §η.1 ships with the
 * default-all-to-cèyǐn fallback per the sectoral-tradition convention
 * established by Peirce / Habermas / Longino in §δ.1-§δ.2. Per-note
 * opt-in ships as §η.1-fix-N follow-up.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §9.1
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	SectorSpec,
	RingSpec,
} from '../types';

type MencianSprout = 'ceyin' | 'xiuwu' | 'cirang' | 'shifei';

/** Sector start angles in canvas math convention. 4 sectors of 90°
 *  each rotated π/4 (45°) CW from the cardinal vertical axis — same
 *  off-axis pattern as Longino (4-sector). Dividers land at NE/SE/SW/
 *  NW positions; the vertical axis (stratum labels) falls inside
 *  sector 3 (N) without a divider crossing it. */
const SECTOR_ARC = Math.PI / 2;
const SECTOR_ROTATION_OFFSET = Math.PI / 4;
const SECTOR_START: Record<MencianSprout, number> = {
	ceyin: -Math.PI / 2 + SECTOR_ROTATION_OFFSET,                       // E wedge
	xiuwu: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,          // S wedge
	cirang: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,     // W wedge
	shifei: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 3 * SECTOR_ARC,     // N wedge
};

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const SECTOR_LABELS: Record<MencianSprout, string> = {
	ceyin: 'sight.v6.tradition.canvas.mencian-sprouts.ceyin',
	xiuwu: 'sight.v6.tradition.canvas.mencian-sprouts.xiuwu',
	cirang: 'sight.v6.tradition.canvas.mencian-sprouts.cirang',
	shifei: 'sight.v6.tradition.canvas.mencian-sprouts.shifei',
};

const SECTOR_ORDER: MencianSprout[] = ['ceyin', 'xiuwu', 'cirang', 'shifei'];

/** Central xìn (信) ring at 15% radius. The label sits inside the
 *  central disc (per drawRingBoundaries placement logic: label for the
 *  first ring entry goes in the annulus from 0 to radiusFrac, i.e.,
 *  inside the center disc). */
const XIN_RING_FRAC = 0.15;

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

function mencianSproutOf(_row: LayoutCacheRow): MencianSprout {
	// TODO post-§η.1: read from frontmatter `mencian_sprout`.
	// Default to cèyǐn (compassion — first in Mencius's enumeration).
	return 'ceyin';
}

export const mencianSprouts: TraditionModule = {
	id: 'mencian-sprouts',
	name: 'Mencian sprouts',
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const sprout = mencianSproutOf(row);
		const startAngle = SECTOR_START[sprout];

		// Preserve radial distance so stratum encoding reads. Clamp
		// minimum radial to just outside the xìn central ring (15% +
		// 2% padding) so default-to-cèyǐn stars don't dive into the
		// center disc.
		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const rawRadial = Math.hypot(dx, dy);
		const minRadial = layout.radius * (XIN_RING_FRAC + 0.02);
		const radial = Math.max(rawRadial, minRadial);

		const month = row.createdMonth ?? 0;
		const jitter = pathHash01(row.notePath);
		const monthFraction = (month + jitter) / 12;
		const clamped = 0.03 + monthFraction * 0.94;
		const angle = startAngle + clamped * SECTOR_ARC;

		return {
			x: layout.centerX + Math.cos(angle) * radial,
			y: layout.centerY + Math.sin(angle) * radial,
		};
	},

	sectorDividers: (_layout: TraditionLayout): SectorSpec[] => {
		return SECTOR_ORDER.map((sprout): SectorSpec => {
			const start = SECTOR_START[sprout];
			return {
				angleStart: start,
				angleEnd: start + SECTOR_ARC,
				label: SECTOR_LABELS[sprout],
			};
		});
	},

	ringBoundaries: (_layout: TraditionLayout): RingSpec[] => {
		// Single small ring at 15% radius carrying the central 5th
		// virtue label (xìn / 信 / trustworthiness). drawRingBoundaries
		// places the label in the annulus from 0 to 15% — i.e., inside
		// the central disc — so it reads as "this innermost zone is xìn".
		return [{ radiusFrac: XIN_RING_FRAC, label: 'sight.v6.tradition.canvas.mencian-sprouts.xin' }];
	},
};

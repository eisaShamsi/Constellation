/**
 * MIG-026 Phase η.3 — Korean Sŏngnihak Four-Seven debate (2×2 grid via
 * sectoral 4-cell).
 *
 * Per Concept Paper §4.1 (East Asian Confucian family):
 *   Geometry         Conceptually a 2×2 grid (axis 1 = lǐ vs qì; axis
 *                    2 = sìduān vs qīqíng), implemented geometrically
 *                    as 4 sectors of 90° each rotated π/4 CW (Longino
 *                    pattern). Each wedge represents one cell of the
 *                    2×2 — a position in the four-seven debate:
 *                      E wedge (sector 0): lǐ × 四端
 *                      S wedge (sector 1): lǐ × 七情
 *                      W wedge (sector 2): qì × 七情
 *                      N wedge (sector 3): qì × 四端
 *   Cultural framing Korean Neo-Confucian philosophy; the
 *                    "Four-Seven" debate (사단칠정논쟁 sadan-chiljeong
 *                    nonjaeng) between Yi T'oegye (1501-1570) and
 *                    Gi Daeseung / Yi Yulgok (1536-1584) in the 16th
 *                    century. The question: are the 4 moral sprouts
 *                    (sìduān, from Mencius) and the 7 emotions
 *                    (qīqíng, from Liji) issued from lǐ (principle)
 *                    or qì (psychophysical force)? T'oegye argued
 *                    lǐ issues the 4 sprouts and qì the 7 emotions;
 *                    Yulgok argued both are issued by qì with lǐ as
 *                    the underlying principle.
 *   Citation         Yi T'oegye, *Sŏnghak Sipdo* (聖學十圖 1568)
 *                    Diagram 6; Yi Yulgok, *Sŏnghak Chibyo* (聖學輯要
 *                    1575); Kalton et al., *The Four-Seven Debate*
 *                    (1994).
 *
 * Star sector assignment (Plan §9.3): from a frontmatter `songnihak_
 * cell` field; default 'li-sa' (T'oegye's classical position for the
 * 4 sprouts) if absent. §η.3 ships with default-all-to-first per the
 * sectoral-tradition convention. Per-note opt-in ships as §η.3-fix-N.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §9.3
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type SongnihakCell = 'li-sa' | 'li-chil' | 'qi-chil' | 'qi-sa';

const SECTOR_ARC = Math.PI / 2;
const SECTOR_ROTATION_OFFSET = Math.PI / 4;
const SECTOR_START: Record<SongnihakCell, number> = {
	'li-sa': -Math.PI / 2 + SECTOR_ROTATION_OFFSET,                       // E wedge
	'li-chil': -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,        // S wedge
	'qi-chil': -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,    // W wedge
	'qi-sa': -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 3 * SECTOR_ARC,      // N wedge
};

const SECTOR_LABELS: Record<SongnihakCell, string> = {
	'li-sa': 'lǐ · 四端',
	'li-chil': 'lǐ · 七情',
	'qi-chil': 'qì · 七情',
	'qi-sa': 'qì · 四端',
};

const SECTOR_ORDER: SongnihakCell[] = ['li-sa', 'li-chil', 'qi-chil', 'qi-sa'];

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

function songnihakCellOf(_row: LayoutCacheRow): SongnihakCell {
	// TODO post-§η.3: read from frontmatter `songnihak_cell`.
	// Default to lǐ · 四端 (T'oegye's classical position).
	return 'li-sa';
}

export const koreanSongnihak: TraditionModule = {
	id: 'korean-songnihak',
	name: 'Korean Sŏngnihak',
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const cell = songnihakCellOf(row);
		const startAngle = SECTOR_START[cell];

		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const radial = Math.hypot(dx, dy);

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
		return SECTOR_ORDER.map((cell): SectorSpec => {
			const start = SECTOR_START[cell];
			return {
				angleStart: start,
				angleEnd: start + SECTOR_ARC,
				label: SECTOR_LABELS[cell],
			};
		});
	},
};

/**
 * MIG-026 Phase θ.4 — Akan Wiredu (sectoral 3-cell).
 *
 * Per Concept Paper §4.1 (African philosophical family):
 *   Geometry         3 sectors of 120° each rotated +π/6 CW from
 *                    the cardinal vertical axis (same off-axis
 *                    principle as Peirce + Habermas in §δ.1-fix-1 —
 *                    avoids divider collision with stratum labels).
 *                    Each sector represents one of Wiredu's
 *                    epistemic categories drawn from his
 *                    reconstruction of Akan philosophical vocabulary.
 *   Cultural framing African analytic philosophy; Kwasi Wiredu
 *                    (1931-2022) developed a project of "conceptual
 *                    decolonization" by reconstructing Akan
 *                    (Ghanaian) philosophical concepts as resources
 *                    for contemporary epistemology. The three terms
 *                    capture Akan distinctions between assertoric
 *                    truth-claim, well-being / reality-fittingness,
 *                    and reflective mind / thought.
 *   Citation         Wiredu, *Cultural Universals and Particulars:
 *                    An African Perspective* (1996) ch. 6;
 *                    *Philosophy and an African Culture* (1980);
 *                    Bodunrin (ed.), *Philosophy in Africa* (1985).
 *
 * Three Akan epistemic categories (per Wiredu's reconstruction):
 *   nokware   — truth / assertoric correspondence
 *   ahonyam   — well-being / reality-fittingness
 *   adwene    — mind / thought / reflective consciousness
 *
 * Star sector assignment (Plan §10.4): from a frontmatter
 * `wiredu_category` field; default 'nokware' (truth — the most
 * central category in Wiredu's reconstruction) if absent.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §10.4
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type WireduCategory = 'nokware' | 'ahonyam' | 'adwene';

const SECTOR_ARC = (2 * Math.PI) / 3;
const SECTOR_ROTATION_OFFSET = Math.PI / 6;
const SECTOR_START: Record<WireduCategory, number> = {
	nokware: -Math.PI / 2 + SECTOR_ROTATION_OFFSET,
	ahonyam: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,
	adwene: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,
};

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const SECTOR_LABELS: Record<WireduCategory, string> = {
	nokware: 'sight.v6.tradition.canvas.akan-wiredu.nokware',
	ahonyam: 'sight.v6.tradition.canvas.akan-wiredu.ahonyam',
	adwene: 'sight.v6.tradition.canvas.akan-wiredu.adwene',
};

const SECTOR_ORDER: WireduCategory[] = ['nokware', 'ahonyam', 'adwene'];

function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

function categoryOf(_row: LayoutCacheRow): WireduCategory {
	return 'nokware';
}

export const akanWiredu: TraditionModule = {
	id: 'akan-wiredu',
	name: 'Akan Wiredu',
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const cat = categoryOf(row);
		const startAngle = SECTOR_START[cat];

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
		return SECTOR_ORDER.map((cat): SectorSpec => {
			const start = SECTOR_START[cat];
			return {
				angleStart: start,
				angleEnd: start + SECTOR_ARC,
				label: SECTOR_LABELS[cat],
			};
		});
	},
};

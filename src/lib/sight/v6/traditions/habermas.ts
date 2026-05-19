/**
 * MIG-026 Phase δ.1 — Habermas tradition (3 knowledge-interests).
 *
 * Per Concept Paper §4.1 (Modern Western family):
 *   Geometry         3 sectors of 120° each, dividers + labels visible:
 *                    technical    (NE-ish, top-right wedge)
 *                    practical    (south wedge)
 *                    emancipatory (NW-ish, top-left wedge)
 *                    Radial within sector = stratum (same encoding as
 *                    Aristotelian); angular within sector = time +
 *                    deterministic jitter.
 *   Cultural framing Frankfurt School critical theory; Jürgen Habermas's
 *                    three "knowledge-constitutive interests" introduced
 *                    in *Knowledge and Human Interests* (1968) as the
 *                    irreducible orientations governing the sciences.
 *                    The sectoral geometry visualizes each note as
 *                    serving one of the three orientations.
 *   Citation         Habermas, *Erkenntnis und Interesse* (1968) /
 *                    *Knowledge and Human Interests* tr. Shapiro (1971),
 *                    Appendix: "Knowledge and Human Interests"; Bohman
 *                    & Rehg, "Jürgen Habermas" (SEP, 2017) §3.
 *
 * Three knowledge-interests (Habermas 1968, Appendix):
 *   - technical    — empirical-analytical sciences; orientation toward
 *                    prediction and control of objectified processes
 *   - practical    — historical-hermeneutic sciences; orientation toward
 *                    mutual understanding within intersubjective life-worlds
 *   - emancipatory — critical social sciences (psychoanalysis, ideology
 *                    critique); orientation toward reflection and
 *                    liberation from systematically distorted communication
 *
 * Star sector assignment (Plan §6.1): from a frontmatter
 * `habermas_interest` field (values: 'technical' | 'practical' |
 * 'emancipatory'); default 'technical' if absent. §δ.1 ships with the
 * default-all-to-technical fallback because LayoutCacheRow doesn't yet
 * carry `habermasInterest`. Defensible philosophical default: most
 * note-taking starts as record-keeping (technical) before being
 * reflectively reclassified as understanding (practical) or critique
 * (emancipatory).
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §6.1
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type HabermasInterest = 'technical' | 'practical' | 'emancipatory';

/** Sector start angles in canvas math convention (0 = east, increases
 *  clockwise because canvas y is inverted). Three 120° sectors offset
 *  by 30° CW from the cardinal vertical axis so dividers don't collide
 *  with the stratum labels on the +y vertical axis. Same geometric
 *  layout + same offset as Peirce; see peirce.ts SECTOR_START block
 *  for the §δ.1-fix-1 rationale.
 *
 *  technical:    -π/3 .. π/3      (1 o'clock → 5 o'clock, NE+E wedge)
 *  practical:    π/3 .. π         (5 o'clock → 9 o'clock, S+SW wedge)
 *  emancipatory: π .. -π/3        (9 o'clock → 1 o'clock, NW+N wedge, includes 12 o'clock) */
const SECTOR_ARC = (2 * Math.PI) / 3;
const SECTOR_ROTATION_OFFSET = Math.PI / 6;
const SECTOR_START: Record<HabermasInterest, number> = {
	technical: -Math.PI / 2 + SECTOR_ROTATION_OFFSET,
	practical: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,
	emancipatory: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,
};

// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const SECTOR_LABELS: Record<HabermasInterest, string> = {
	technical: 'sight.v6.tradition.canvas.habermas.technical',
	practical: 'sight.v6.tradition.canvas.habermas.practical',
	emancipatory: 'sight.v6.tradition.canvas.habermas.emancipatory',
};

const SECTOR_ORDER: HabermasInterest[] = ['technical', 'practical', 'emancipatory'];

/** Determine a note's Habermas knowledge-interest.
 *
 *  Reads frontmatter `habermas_interest` via `row.habermasInterest`
 *  (extracted Rust-side in MIG-029 §ν.2). Falls back to 'technical'
 *  when absent or invalid (Plan §6.1 default).
 */
function habermasInterestOf(row: LayoutCacheRow): HabermasInterest {
	// MIG-029 §ν.3 (2026-05-19) — read from frontmatter
	// `habermas_interest` via row.habermasInterest. Falls back to
	// 'technical' when absent or invalid (Plan §6.1 default).
	switch (row.habermasInterest) {
		case 'technical':
		case 'practical':
		case 'emancipatory':
			return row.habermasInterest;
		default:
			return 'technical';
	}
}

/** FNV-1a 32-bit hash of a string → normalized [0, 1) value. Local
 *  duplicate per the convention established in pramana / mohist /
 *  peirce modules (traditions/ stays free of renderer imports). */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

export const habermas: TraditionModule = {
	id: 'habermas',
	name: 'Habermas',
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const interest = habermasInterestOf(row);
		const startAngle = SECTOR_START[interest];

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
		return SECTOR_ORDER.map((interest): SectorSpec => {
			const start = SECTOR_START[interest];
			return {
				angleStart: start,
				angleEnd: start + SECTOR_ARC,
				label: SECTOR_LABELS[interest],
			};
		});
	},
};

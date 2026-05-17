/**
 * MIG-026 Phase δ.1 — Peirce tradition (3 phaneroscopic categories).
 *
 * Per Concept Paper §4.1 (Modern Western family):
 *   Geometry         3 sectors of 120° each, dividers + labels visible:
 *                    Firstness  (NE-ish, top-right wedge, 12→4 o'clock)
 *                    Secondness (south wedge, 4→8 o'clock)
 *                    Thirdness  (NW-ish, top-left wedge, 8→12 o'clock)
 *                    Radial within sector = stratum (same encoding as
 *                    Aristotelian); angular within sector = time +
 *                    deterministic jitter.
 *   Cultural framing American pragmatist epistemology; Charles Sanders
 *                    Peirce's three phaneroscopic categories (Firstness,
 *                    Secondness, Thirdness) introduced in "On a New List
 *                    of Categories" (1867) as the irreducible kinds of
 *                    elements present in any phenomenon. The sectoral
 *                    geometry visualizes each note as belonging to one
 *                    of the three categories.
 *   Citation         Peirce, "On a New List of Categories" (1867);
 *                    *Collected Papers* (1931–58), vol. 1 §§ 300–353
 *                    (phaneroscopy); Houser & Kloesel, *The Essential
 *                    Peirce* (1992), vol. 1 ch. 1.
 *
 * Star sector assignment (Plan §6.1): from a frontmatter `peirce_category`
 * field (values: 'firstness' | 'secondness' | 'thirdness'); default
 * 'firstness' if absent. §δ.1 ships with the default-all-to-Firstness
 * fallback because LayoutCacheRow doesn't yet carry `peirceCategory`;
 * per-note frontmatter integration is a §δ.1-fix-N follow-up once the
 * Rust-side extraction lands. Defensible philosophical default:
 * Firstness is the category of immediacy / quality — anything not yet
 * reflectively reclassified.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §6.1
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type PeirceCategory = 'firstness' | 'secondness' | 'thirdness';

/** Sector start angles in canvas math convention (0 = east, increases
 *  clockwise because canvas y is inverted). Three 120° sectors offset
 *  by 30° CW from the cardinal vertical axis so dividers don't collide
 *  with the stratum labels (FOUNDATION, WORKING, …, EDGE OF KNOWING)
 *  which sit on the +y vertical axis from dome center.
 *
 *  §δ.1-fix-1 (Eisa Boss test 2026-05-17): pre-fix, the first sector
 *  started at -π/2 (12 o'clock) — the top divider overlapped the
 *  stratum labels. Rotated by +π/6 so the top divider is at
 *  -π/3 (~1 o'clock), with the vertical axis falling safely INSIDE
 *  the Thirdness sector (no divider there). Same fix applied to
 *  habermas.ts which shares this geometry.
 *
 *  Firstness:  -π/3 .. -π/3 + 2π/3 = π/3      (1 o'clock → 5 o'clock, NE+E wedge)
 *  Secondness: π/3 .. π/3 + 2π/3 = π          (5 o'clock → 9 o'clock, S+SW wedge)
 *  Thirdness:  π .. π + 2π/3 ≡ -π/3           (9 o'clock → 1 o'clock, NW+N wedge, includes 12 o'clock) */
const SECTOR_ARC = (2 * Math.PI) / 3;
const SECTOR_ROTATION_OFFSET = Math.PI / 6;
const SECTOR_START: Record<PeirceCategory, number> = {
	firstness: -Math.PI / 2 + SECTOR_ROTATION_OFFSET,
	secondness: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + SECTOR_ARC,
	thirdness: -Math.PI / 2 + SECTOR_ROTATION_OFFSET + 2 * SECTOR_ARC,
};

const SECTOR_LABELS: Record<PeirceCategory, string> = {
	firstness: 'Firstness',
	secondness: 'Secondness',
	thirdness: 'Thirdness',
};

const SECTOR_ORDER: PeirceCategory[] = ['firstness', 'secondness', 'thirdness'];

/** Determine a note's Peirce category.
 *
 *  Per Plan §6.1: read from frontmatter `peirce_category` field; default
 *  Firstness if absent.
 *
 *  §δ.1 ship: LayoutCacheRow does not yet carry `peirceCategory`, so
 *  this unconditionally returns 'firstness'. Per-note frontmatter
 *  integration ships in a follow-up once Rust-side extraction lands.
 */
function peirceCategoryOf(_row: LayoutCacheRow): PeirceCategory {
	// TODO post-§δ.1: when LayoutCacheRow gains `peirceCategory: string | null`,
	// switch on the value here. For now, all notes default to Firstness.
	return 'firstness';
}

/** FNV-1a 32-bit hash of a string → normalized [0, 1) value. Local
 *  duplicate of the helper used in pramana.ts / mohist-san-biao.ts so
 *  traditions/ stays free of cross-module imports from the renderer. */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

export const peirce: TraditionModule = {
	id: 'peirce',
	name: 'Peirce',
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const cat = peirceCategoryOf(row);
		const startAngle = SECTOR_START[cat];

		// Preserve radial distance so stratum encoding (Foundation → Edge
		// of Knowing rings) reads identically within each sector. Users
		// can still tell which band a star belongs to in Peirce view just
		// as in Aristotelian.
		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const radial = Math.hypot(dx, dy);

		// Within-sector angular: month occupies one twelfth of the wedge;
		// jitter spreads notes within their month slot. Clamped to
		// [0.03, 0.97] of the wedge so stars don't kiss the divider lines.
		const month = row.createdMonth ?? 0;
		const jitter = pathHash01(row.notePath);
		const monthFraction = (month + jitter) / 12; // 0 .. 1
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

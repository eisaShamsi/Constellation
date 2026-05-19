/**
 * MIG-025 §C.3 — pramāṇa tradition (4 quadrants).
 * MIG-026 Phase 0 — K1 rename: "register" → "tradition" throughout.
 *
 * Per Concept Paper §4.1.2:
 *   Geometry         4 quadrants for the Nyāya valid means of knowing:
 *                    pratyakṣa (NE, perception), anumāna (SE, inference),
 *                    upamāna (SW, analogy/comparison), śabda (NW, testimony).
 *                    Quadrant dividers visible. Radial within quadrant =
 *                    stratum (same encoding as Aristotelian); angular
 *                    within quadrant = time.
 *   Cultural framing Indian Nyāya epistemology; honors pramāṇas as
 *                    *kinds, not levels*.
 *   Citation         Nyāya-Sūtra 1.1.3; Mohanty, *Classical Indian
 *                    Philosophy* (2000), pp. 17–34; Matilal, *Perception*
 *                    (1986), ch. 1.
 *   v4.1 polish      Per-quadrant radial-internal structure (e.g.,
 *                    pratyakṣa: indriya-artha-sannikarṣa loci; anumāna:
 *                    pakṣa/sādhya/hetu loci per the 5-membered syllogism).
 *
 * Star quadrant assignment (Plan §C.3): from a frontmatter `pramana_kind`
 * field; default `pratyakṣa` if absent. §C.3 ships with the default
 * behavior (all notes → pratyakṣa) since `pramana_kind` is not yet
 * extracted into LayoutCacheRow on the Rust side. This is per the Plan
 * verbatim. Per-note frontmatter integration + sidebar facet hint ship
 * in a follow-up (will appear as §C.3-fix-N or as part of a batched
 * post-§C polish pass). Once LayoutCacheRow gains `pramanaKind: string |
 * null`, `pramanaKindOf` reads it directly.
 *
 * Within each quadrant:
 *   - radial encoding (distance from center) preserved from defaultPos,
 *     so stratum bands stay legible — innermost ring = Foundation,
 *     outer rim = Edge of Knowing, just as in Aristotelian.
 *   - angular encoding (within the 90° wedge) is month + hash jitter,
 *     so notes spread out by creation time inside their quadrant rather
 *     than overlapping at a single point.
 *
 * Dignāga tradition is EXCLUDED entirely per §C.1-fix-1 — this is the
 * ONLY pramāṇa-tradition that ships in Constellation.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1.2
 * Plan:          lab/reports/MIG-025-SIGHT-V6-PLAN.md §C.3
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type PramanaKind = 'pratyaksha' | 'anumana' | 'upamana' | 'shabda';

/** Quadrant start angles in canvas math convention (0 = east, increases
 *  clockwise because canvas y is inverted). Each quadrant spans π/2 rad.
 *
 *  §δ.2-fix-1 (Eisa Boss test 2026-05-17 side discovery): pre-fix the
 *  quadrants started at the cardinal axes (-π/2 = north, 0 = east,
 *  etc.), which made the top divider stroke run straight up through
 *  the stratum labels (FOUNDATION, WORKING, CONNECTION, SYNTHESIS,
 *  EDGE OF KNOWING) on the +y axis. Eisa flagged this when testing
 *  pramana during the δ.2 cycle — same root cause as §δ.1-fix-1 for
 *  Peirce + Habermas (3-sector) and the deliberate Longino offset
 *  (4-sector). Applying the half-wedge offset (π/4 for a 4-sector
 *  pattern) shifts dividers off the cardinal axes to NE/SE/SW/NW
 *  positions; the vertical axis falls cleanly inside the śabda
 *  quadrant.
 *
 *  Post-fix quadrant positions (geometric, not cultural):
 *    pratyakṣa  = angles -π/4 .. π/4    (1:30 → 4:30, E wedge)
 *    anumāna    = angles π/4 .. 3π/4    (4:30 → 7:30, S wedge)
 *    upamāna    = angles 3π/4 .. 5π/4   (7:30 → 10:30, W wedge)
 *    śabda      = angles 5π/4 .. 7π/4   (10:30 → 1:30, N wedge, includes +y axis)
 *
 *  Cultural mapping (Concept Paper §4.1.2): the pramāṇas remain
 *  CATEGORICAL "kinds, not levels" — the visual quadrant they occupy
 *  is purely a layout choice. The Concept Paper's NE/SE/SW/NW
 *  description becomes E/S/W/N after this rotation, which is a doc-
 *  drift item flagged for §4.1.2 update at MIG-026 ship-gate. */
const QUADRANT_ROTATION_OFFSET = Math.PI / 4;
const QUADRANT_START_ANGLES: Record<PramanaKind, number> = {
	pratyaksha: -Math.PI / 2 + QUADRANT_ROTATION_OFFSET,
	anumana: 0 + QUADRANT_ROTATION_OFFSET,
	upamana: Math.PI / 2 + QUADRANT_ROTATION_OFFSET,
	shabda: Math.PI + QUADRANT_ROTATION_OFFSET,
};

/** Display labels rendered in the dome chrome at each quadrant's wedge
 *  center per Concept Paper §4.1.2. Sanskrit transliteration with proper
 *  diacritics. */
// MIG-026 §λ-fix-3 — i18n keys; English values live in src/lib/i18n/en.json
const QUADRANT_LABELS: Record<PramanaKind, string> = {
	pratyaksha: 'sight.v6.tradition.canvas.pramana.pratyaksha',
	anumana: 'sight.v6.tradition.canvas.pramana.anumana',
	upamana: 'sight.v6.tradition.canvas.pramana.upamana',
	shabda: 'sight.v6.tradition.canvas.pramana.shabda',
};

const QUADRANT_ORDER: PramanaKind[] = ['pratyaksha', 'anumana', 'upamana', 'shabda'];

/** Determine a note's pramāṇa quadrant.
 *
 *  Per Plan §C.3: read from frontmatter `pramana_kind` field; default
 *  pratyakṣa if absent.
 *
 *  §C.3 ship: LayoutCacheRow does not yet carry `pramanaKind`, so this
 *  unconditionally returns `pratyaksha`. The philosophical default is
 *  defensible: all knowledge starts as direct perception until
 *  reflectively reclassified as inference / analogy / testimony.
 *  Users opt their notes into other quadrants by adding
 *  `pramana_kind: anumana` (etc.) to frontmatter, once the Rust-side
 *  extraction ships in a follow-up.
 */
function pramanaKindOf(row: LayoutCacheRow): PramanaKind {
	// MIG-029 §ν.3 (2026-05-19) — read from frontmatter `pramana_kind`
	// via the layout-cache row's `pramanaKind` field. Falls back to
	// pratyakṣa (the philosophical default per Plan §C.3) when the
	// frontmatter key is absent OR its value is not one of the 4
	// allowed kinds. Defends against typos by treating unknown values
	// as default rather than crashing the renderer.
	switch (row.pramanaKind) {
		case 'pratyaksha':
		case 'anumana':
		case 'upamana':
		case 'shabda':
			return row.pramanaKind;
		default:
			return 'pratyaksha';
	}
}

/** FNV-1a 32-bit hash of a string → normalized [0, 1) value. Used for
 *  per-note angular jitter so multiple notes of the same month + same
 *  quadrant spread out within their wedge instead of stacking at a
 *  single point. Duplicates anchor.ts's pathHashJitter logic locally
 *  to keep traditions/ free of cross-module imports from the renderer. */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

export const pramana: TraditionModule = {
	id: 'pramana',
	name: 'pramāṇa',
	// MIG-026 Phase α — shape discriminator. 4 quadrants of Nyāya
	// pramāṇas = sectoral shape.
	shape: 'sectoral',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const kind = pramanaKindOf(row);
		const startAngle = QUADRANT_START_ANGLES[kind];

		// Preserve radial distance from center so the stratum encoding
		// (Foundation → Edge of Knowing rings) reads identically within
		// each quadrant. The user can still tell which band a star
		// belongs to in pramāṇa view just as in Aristotelian view.
		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const radial = Math.hypot(dx, dy);

		// Position within the 90° quadrant wedge: month occupies one
		// twelfth of the wedge; jitter spreads notes within their month
		// slot so co-month/co-stratum notes don't pile up at a single
		// point. Clamped to [0.03, 0.97] of the wedge so stars don't
		// kiss the divider line.
		const month = row.createdMonth ?? 0;
		const jitter = pathHash01(row.notePath);
		const monthFraction = (month + jitter) / 12; // 0 .. 1
		const clamped = 0.03 + monthFraction * 0.94;
		const angle = startAngle + clamped * (Math.PI / 2);

		return {
			x: layout.centerX + Math.cos(angle) * radial,
			y: layout.centerY + Math.sin(angle) * radial,
		};
	},

	sectorDividers: (_layout: TraditionLayout): SectorSpec[] => {
		return QUADRANT_ORDER.map((kind): SectorSpec => {
			const start = QUADRANT_START_ANGLES[kind];
			return {
				angleStart: start,
				angleEnd: start + Math.PI / 2,
				label: QUADRANT_LABELS[kind],
			};
		});
	},
};

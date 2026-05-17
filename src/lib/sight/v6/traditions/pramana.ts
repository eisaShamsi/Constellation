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
 *  NE (upper right) = pratyakṣa   = angles −π/2 .. 0
 *  SE (lower right) = anumāna     = angles 0    .. π/2
 *  SW (lower left)  = upamāna     = angles π/2  .. π
 *  NW (upper left)  = śabda       = angles π    .. 3π/2  (≡ −π .. −π/2) */
const QUADRANT_START_ANGLES: Record<PramanaKind, number> = {
	pratyaksha: -Math.PI / 2,
	anumana: 0,
	upamana: Math.PI / 2,
	shabda: Math.PI,
};

/** Display labels rendered in the dome chrome at each quadrant's wedge
 *  center per Concept Paper §4.1.2. Sanskrit transliteration with proper
 *  diacritics. */
const QUADRANT_LABELS: Record<PramanaKind, string> = {
	pratyaksha: 'pratyakṣa',
	anumana: 'anumāna',
	upamana: 'upamāna',
	shabda: 'śabda',
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
function pramanaKindOf(_row: LayoutCacheRow): PramanaKind {
	// TODO post-§C.3: when LayoutCacheRow gains `pramanaKind: string | null`,
	// switch on the value here. For now, all notes default to pratyakṣa
	// per Plan verbatim.
	return 'pratyaksha';
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

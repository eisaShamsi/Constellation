/**
 * MIG-025 §C.4 — masādir tradition (4 sectors + 4 extension chips).
 * MIG-026 Phase 0 — K1 rename: "register" → "tradition" throughout.
 *
 * Per Concept Paper §4.1.3:
 *   Geometry         4 categorical sectors (NOT concentric ladder):
 *                    Qur'an (NE), sunnah (SE), ijmāʿ (SW), qiyās (NW).
 *                    Each sector annotated with kind distinction
 *                    (naṣṣ vs. ijtihādī; qaṭʿī vs. ẓannī). Four
 *                    extension chips below the dome: istiḥsān,
 *                    istiṣḥāb, maṣlaḥa mursalah, ʿurf.
 *   Cultural framing Sunni *uṣūl al-fiqh*; sources as different kinds
 *                    of proof, not degrees-of-one-thing.
 *   Citation         al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*
 *                    (vol. 1, ed. Hafnawi), pp. 81–94; Rosenthal,
 *                    *Knowledge Triumphant* (1970).
 *   Note             ijmāʿ-as-ijtihādī is contested by Ash'arī/Māturīdī
 *                    kalām (which treats it as transmitted/binding). v6
 *                    ships the Mustaṣfā-aligned reading; alternative
 *                    kalām reading is a v4.1 variant.
 *
 * Star sector assignment (Plan §C.4): from a frontmatter `masadir_source`
 * field; default `quran` if absent. §C.4 ships with the default behavior
 * (all notes → Qur'an) since `masadirSource` is not yet extracted into
 * LayoutCacheRow on the Rust side. This is per the Plan verbatim. Per-note
 * frontmatter integration ships in a §C.4-fix-N follow-up.
 *
 * Within each sector: same as pramāṇa — radial preserved from defaultPos
 * (stratum bands stay legible), angular = month + hash jitter within the
 * 90° wedge.
 *
 * Sub-sector annotations (naṣṣ / ijtihādī / qaṭʿī / ẓannī) — §C.4 ships
 * without them. The Concept Paper §4.1.3 describes them as PER-SUBSECTOR
 * distinctions (different kinds within each sector), not as a single
 * annotation that fits cleanly under the main sector label. They are a
 * polish target for a §C.4-fix-N variant once the spec settles.
 *
 * Extension chips: 4 supplementary sources per §4.1.3, rendered as a row
 * of HTML badges below the canvas-host by SightV6.svelte. They are visual
 * reminders — for now — that the user can opt notes into istiḥsān /
 * istiṣḥāb / maṣlaḥa mursalah / ʿurf once Rust-side frontmatter
 * extraction lands. They have no effect on the dome layout in §C.4.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1.3
 * Plan:          lab/reports/MIG-025-SIGHT-V6-PLAN.md §C.4
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, SectorSpec } from '../types';

type MasadirSource = 'quran' | 'sunnah' | 'ijma' | 'qiyas';

/** Quadrant start angles in canvas math convention (0 = east, increases
 *  clockwise because canvas y is inverted). Each sector spans π/2 rad.
 *
 *  NE (upper right) = Qur'an  = angles −π/2 .. 0
 *  SE (lower right) = sunnah  = angles 0    .. π/2
 *  SW (lower left)  = ijmāʿ   = angles π/2  .. π
 *  NW (upper left)  = qiyās   = angles π    .. 3π/2  (≡ −π .. −π/2) */
const QUADRANT_START_ANGLES: Record<MasadirSource, number> = {
	quran: -Math.PI / 2,
	sunnah: 0,
	ijma: Math.PI / 2,
	qiyas: Math.PI,
};

/** Display labels rendered in the dome chrome at each sector's wedge
 *  center per Concept Paper §4.1.3. Arabic transliterations with proper
 *  diacritics (ʿ for ʿayn; macrons for long vowels). */
const QUADRANT_LABELS: Record<MasadirSource, string> = {
	quran: "Qur'an",
	sunnah: 'sunnah',
	ijma: 'ijmāʿ',
	qiyas: 'qiyās',
};

const QUADRANT_ORDER: MasadirSource[] = ['quran', 'sunnah', 'ijma', 'qiyas'];

/** Four supplementary sources beyond the 4 main sectors per Concept
 *  Paper §4.1.3. Rendered as a row of HTML badges below the anchor
 *  canvas-host by SightV6.svelte. These do not yet drive any layout
 *  behavior — they are visual reminders that the masādir vocabulary
 *  extends beyond the 4 main sectors. Per-note opt-in via frontmatter
 *  (`masadir_source: istihsan` etc.) ships in §C.4-fix-N once
 *  Rust-side extraction lands. */
const EXTENSION_CHIP_LABELS: readonly string[] = [
	'istiḥsān',
	'istiṣḥāb',
	'maṣlaḥa mursalah',
	'ʿurf',
];

/** Determine a note's masādir source sector.
 *
 *  Per Plan §C.4: read from frontmatter `masadir_source` field; default
 *  Qur'an if absent.
 *
 *  §C.4 ship: LayoutCacheRow does not yet carry `masadirSource`, so this
 *  unconditionally returns `quran`. The user can later opt notes into
 *  sunnah / ijmāʿ / qiyās by adding `masadir_source: sunnah` (etc.) to
 *  frontmatter, once the Rust-side extraction ships. */
function masadirSourceOf(_row: LayoutCacheRow): MasadirSource {
	// TODO post-§C.4: when LayoutCacheRow gains `masadirSource: string | null`,
	// switch on the value here. For now, all notes default to Qur'an
	// per Plan verbatim.
	return 'quran';
}

/** FNV-1a 32-bit hash → normalized [0, 1). Same as pramana.ts's
 *  pathHash01 — duplicated locally so traditions/ stays decoupled from
 *  anchor.ts. The mild duplication is intentional: tradition modules
 *  should be self-contained. */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

export const masadir: TraditionModule = {
	id: 'masadir',
	name: 'masādir',

	remapStarPosition: (row: LayoutCacheRow, defaultPos, layout: TraditionLayout) => {
		const source = masadirSourceOf(row);
		const startAngle = QUADRANT_START_ANGLES[source];

		// Preserve radial distance from center so the stratum encoding
		// (Foundation → Edge of Knowing rings) stays legible within each
		// sector. Same approach as pramana.ts.
		const dx = defaultPos.x - layout.centerX;
		const dy = defaultPos.y - layout.centerY;
		const radial = Math.hypot(dx, dy);

		// Angular position within the 90° wedge: month + per-note hash
		// jitter clamped to [0.03, 0.97] of the wedge so stars don't
		// kiss the divider lines.
		const month = row.createdMonth ?? 0;
		const jitter = pathHash01(row.notePath);
		const monthFraction = (month + jitter) / 12;
		const clamped = 0.03 + monthFraction * 0.94;
		const angle = startAngle + clamped * (Math.PI / 2);

		return {
			x: layout.centerX + Math.cos(angle) * radial,
			y: layout.centerY + Math.sin(angle) * radial,
		};
	},

	sectorDividers: (_layout: TraditionLayout): SectorSpec[] => {
		return QUADRANT_ORDER.map((source): SectorSpec => {
			const start = QUADRANT_START_ANGLES[source];
			return {
				angleStart: start,
				angleEnd: start + Math.PI / 2,
				label: QUADRANT_LABELS[source],
			};
		});
	},

	extensionChips: () => [...EXTENSION_CHIP_LABELS],
};

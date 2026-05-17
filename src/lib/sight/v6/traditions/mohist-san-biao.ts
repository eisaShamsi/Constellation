/**
 * MIG-026 Phase γ — Mohist sān biǎo (三表) tradition (3 horizontal bands).
 *
 * Per Concept Paper §4.2.1 (and the original MIG-025 §D.3 placeholder
 * that was never implemented):
 *   Geometry         3 horizontal zones stacked top-to-bottom:
 *                    top    = 本 (běn, root)   — historical precedent of the sage-kings
 *                    middle = 原 (yuán, origin) — direct observational evidence
 *                    bottom = 用 (yòng, use)    — practical social benefit
 *                    Each star is placed in its zone with deterministic
 *                    jitter; horizontal axis preserves no specific
 *                    encoding (Mohist's three standards are CATEGORICAL,
 *                    not ordinal).
 *   Cultural framing Classical Chinese pragmatist epistemology
 *                    (Mòzǐ 墨子, ~5th c. BCE). The sān biǎo (三表,
 *                    "three standards/marks") are tests applied to
 *                    doctrines to determine if they are worth holding:
 *                    is there historical precedent? does observation
 *                    support it? does adopting it benefit the people?
 *                    The horizontal-bands geometry visualizes each note
 *                    as having been evaluated against one of the three
 *                    tests.
 *   Citation         *Mòzǐ*, Book IX, "Fēi Mìng Shàng" 非命上
 *                    ("Anti-Fatalism, Part I"); Graham, *Disputers of
 *                    the Tao* (1989), ch. 1; Fraser, "Mohism" (SEP, 2020).
 *   v1 preview       Marked `preview` in TRADITIONS_META so the chip
 *                    carries a v1-preview badge — the philosophical
 *                    deeper structure (e.g., per-standard sub-criteria)
 *                    is a v4.1 polish target.
 *
 * Zone assignment (Plan §5): from a frontmatter `mohist_zone` field
 * with values `ben` / `yuan` / `yong`; default deterministic-hash to one
 * of the three if absent. §γ ships with the hash-based fallback because
 * LayoutCacheRow does not yet carry `mohistZone`; per-note frontmatter
 * integration ships as a follow-up once the Rust-side extraction lands.
 *
 * Within each band:
 *   - vertical: jittered within the band's y-range using a deterministic
 *     hash of the notePath so co-zone notes spread vertically instead
 *     of stacking
 *   - horizontal: distributed across the band's x-range, clipped to the
 *     dome circle at each y (so stars don't escape the dome bounds)
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.2.1
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §5 (Phase γ)
 */
import type {
	LayoutCacheRow,
	TraditionLayout,
	TraditionModule,
	HorizontalBandsSpec,
} from '../types';

type MohistZone = 0 | 1 | 2;

/** Display labels for the three sān biǎo bands per Concept Paper §4.2.1.
 *  Chinese characters first (the philosophical primary), transliteration
 *  + English gloss second. The renderer in anchor.ts uses these in the
 *  HorizontalBandsSpec.bands[].label field. */
const ZONE_LABELS: Record<MohistZone, string> = {
	0: '本 běn · root',     // top band — historical precedent
	1: '原 yuán · origin',  // middle band — observational evidence
	2: '用 yòng · use',     // bottom band — practical benefit
};

/** FNV-1a 32-bit hash of a string → normalized [0, 1) value. Used for
 *  deterministic zone assignment + within-band jitter. Duplicates
 *  pramana.ts's pathHash01 locally so traditions/ stays free of
 *  cross-module imports from the renderer. */
function pathHash01(path: string): number {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return ((h >>> 0) & 0xffff) / 0xffff;
}

/** Secondary hash (different prime mixing) for an independent jitter
 *  axis. Without two independent hashes the within-band placement would
 *  be perfectly correlated with the zone assignment, producing visible
 *  diagonal stripes in dense regions. */
function pathHash01Alt(path: string): number {
	let h = 0xcafebabe;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return (((h >>> 16) & 0xffff)) / 0xffff;
}

/** Determine a note's sān biǎo zone.
 *
 *  Per Plan §5: read from frontmatter `mohist_zone` field (values:
 *  'ben' | 'yuan' | 'yong'); default deterministic-hash to one of the
 *  three if absent.
 *
 *  §γ ship: LayoutCacheRow does not yet carry `mohistZone`, so this
 *  unconditionally hash-buckets the notePath into 0/1/2. Users opt
 *  notes into specific zones by adding `mohist_zone: ben` (etc.) to
 *  frontmatter once the Rust-side extraction lands as a §γ-fix-N.
 */
function mohistZoneOf(row: LayoutCacheRow): MohistZone {
	// TODO post-§γ: when LayoutCacheRow gains `mohistZone: string | null`,
	// switch on the value here. For now, hash-bucket into 3 zones.
	const bucket = Math.floor(pathHash01(row.notePath) * 3);
	// Defensive clamp — pathHash01 returns [0, 1) so * 3 yields [0, 3)
	// and floor yields 0/1/2; the clamp guards against exact-1.0 edge
	// cases from any future hash impl change.
	return (bucket >= 2 ? 2 : (bucket <= 0 ? 0 : 1)) as MohistZone;
}

export const mohistSanBiao: TraditionModule = {
	id: 'mohist-san-biao',
	name: 'Mohist sān biǎo',
	// MIG-026 Phase α/γ — shape discriminator. 3 horizontal zones.
	shape: 'horizontal-bands',

	remapStarPosition: (
		row: LayoutCacheRow,
		_defaultPos: { x: number; y: number },
		layout: TraditionLayout,
	) => {
		const zone = mohistZoneOf(row);

		// Vertical: divide the dome bounding circle into 3 equal-height
		// horizontal bands. Each band's y-center is at:
		//   zone 0 (top):    centerY - 2*r/3
		//   zone 1 (middle): centerY
		//   zone 2 (bottom): centerY + 2*r/3
		// Each band's half-height ≈ r/3.
		const bandHalfHeight = layout.radius / 3;
		const zoneCenterY = layout.centerY + (zone - 1) * (2 * layout.radius / 3);

		// Within-band vertical jitter: 80% of the band's half-height so
		// stars don't kiss the band dividers.
		const jitterY = pathHash01(row.notePath); // 0..1
		const y = zoneCenterY + (jitterY - 0.5) * bandHalfHeight * 1.6;

		// Horizontal: distribute uniformly across the dome width at this
		// y, clipped to the circle bound `(x - centerX)^2 + (y - centerY)^2 <= r^2`.
		// At y = zoneCenterY ± bandHalfHeight, the available half-width
		// shrinks because the circle narrows there. Compute the half-
		// width at the actual y, with a small 10% inset so stars don't
		// hug the dome edge.
		const dy = y - layout.centerY;
		const halfWidthAtY = Math.sqrt(
			Math.max(0, layout.radius * layout.radius - dy * dy),
		);
		const safeHalfWidth = halfWidthAtY * 0.90;
		const jitterX = pathHash01Alt(row.notePath); // 0..1 (independent of jitterY)
		const x = layout.centerX + (jitterX - 0.5) * 2 * safeHalfWidth;

		return { x, y };
	},

	horizontalBandsSpec: (_layout: TraditionLayout): HorizontalBandsSpec => {
		return {
			bands: [
				{ label: ZONE_LABELS[0] },
				{ label: ZONE_LABELS[1] },
				{ label: ZONE_LABELS[2] },
			],
		};
	},
};

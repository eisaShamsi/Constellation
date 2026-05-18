/**
 * MIG-025 §A.8 — Sight v6 dome geometry (pure functions).
 *
 * Per Concept Paper v4.0 §2.2 + the v0.3 visual contract
 * (`docs/sight-redesign-v0.3-full-layout.svg`): the anchor dome is a
 * circular field with **5 concentric strata bands** (Foundation at
 * the pole, Edge of Knowing at the rim) and a 12-month calendar rim
 * wrapping the outside. No Milky Way (v6 dropped v5's decorative
 * gradient — the Suwaidi reference is sparse).
 *
 * Reparameterized from `v5/dome.ts` (which had 8 strata bands). The
 * cache schema still stores raw stratum 1..8 from `sky_nodes` for
 * backward-compatibility; this module maps raw → v6 5-band scheme
 * via `bandForRawStratum`.
 *
 * All geometry is parameterized on `domeRadius` — the dome scales
 * to the container; the math is unitless. v0.3 mock uses
 * domeRadius=280 px.
 *
 * Pure functions; no DOM, no Canvas, no state.
 */

// ════════════════════════════════════════════════════════════════════
// Type contracts (mirror v5/dome.ts where applicable)
// ════════════════════════════════════════════════════════════════════

/** A single calendar rim month label position. The label itself is
 *  rendered as canvas text (no HTML overlay in v6 — the dome-only
 *  render path keeps the chrome on the canvas surface). */
export interface MonthLabel {
	monthIndex: number;          // 0..11
	angle: number;               // radians; 0 = top (north), clockwise
	x: number;                   // pixel offset from dome center
	y: number;                   // pixel offset (y grows DOWN per Canvas convention)
	label: string;               // locale-aware short month name
}

/** v6 5-band stratum identifier. Inner → outer.
 *  Per Concept Paper §2.2: Foundation = innermost (deepest); Edge of
 *  Knowing = outermost rim (frontier of thought). */
export type StratumBand =
	| 'foundation'
	| 'working'
	| 'connection'
	| 'synthesis'
	| 'edge-of-knowing';

/** The 5 v6 stratum bands in inner→outer order. */
export const STRATUM_BANDS: StratumBand[] = [
	'foundation',
	'working',
	'connection',
	'synthesis',
	'edge-of-knowing',
];

/** English-literal stratum labels. Kept as the canonical English
 *  source-of-truth that en.json mirrors at `sight.v6.stratum.<band>`,
 *  AND as a defensive fallback in `anchor.ts` (when the renderer's
 *  labelize call returns the raw key unchanged — i.e. when an
 *  unsupported locale + missing en entry leaves the i18n chain with
 *  nothing to resolve — anchor falls back to this map so the dome
 *  still reads as English instead of as raw key text).
 *
 *  MIG-026 §λ-fix-3 (2026-05-18): the on-canvas renderer resolves
 *  labels via $t() through STRATUM_LABEL_KEYS below.
 *  MIG-026 §μ drift audit (2026-05-18): an earlier version of this
 *  comment said the literal map was "kept ONLY for facets.ts's
 *  legacy consumer" — that claim was stale after facets.ts migrated
 *  to i18n keys in §λ-fix-4. The actual remaining consumer is
 *  anchor.ts's defensive fallback at the stratum-label draw call. */
export const STRATUM_LABELS: Record<StratumBand, string> = {
	foundation: 'FOUNDATION',
	working: 'WORKING',
	connection: 'CONNECTION',
	synthesis: 'SYNTHESIS',
	'edge-of-knowing': 'EDGE OF KNOWING',
};

/** i18n key per stratum band — for the on-canvas anchor renderer
 *  (anchor.ts). Resolves via $t() at draw time so the dome's
 *  vertical-axis labels (FOUNDATION → الأساس → 基础 → etc.) follow
 *  the user's active locale. The key path matches the en.json /
 *  ar.json / ... structure at sight.v6.stratum.<band>. */
export const STRATUM_LABEL_KEYS: Record<StratumBand, string> = {
	foundation: 'sight.v6.stratum.foundation',
	working: 'sight.v6.stratum.working',
	connection: 'sight.v6.stratum.connection',
	synthesis: 'sight.v6.stratum.synthesis',
	'edge-of-knowing': 'sight.v6.stratum.edge-of-knowing',
};

// ════════════════════════════════════════════════════════════════════
// Stratum geometry (5-band)
// ════════════════════════════════════════════════════════════════════

/** Returns the 5 strata band BOUNDARIES from outer rim to pole.
 *
 * Index 0 = outer rim radius (Edge of Knowing outer boundary).
 * Index 4 = innermost ring radius (Foundation outer boundary).
 * The pole itself is implicit at radius 0.
 *
 * Linear distribution: 4 rings divide the dome into 5 bands.
 *
 * v0.3 mock uses domeRadius=280 with bands at:
 *   [280, 224, 168, 112, 56] — close to evenly spaced.
 */
export function stratumBandBoundaries(domeRadius: number): number[] {
	return Array.from({ length: 5 }, (_, n) => (domeRadius * (5 - n)) / 5);
}

/** Returns the CENTER radius of band index n (0 = outermost
 *  Edge-of-Knowing, 4 = innermost Foundation). A star at this band
 *  is positioned at this radius. */
export function radiusForBandIndex(bandIndex: number, domeRadius: number): number {
	const n = Math.max(0, Math.min(4, Math.round(bandIndex)));
	const outer = (domeRadius * (5 - n)) / 5;
	const inner = (domeRadius * (5 - n - 1)) / 5;
	return (outer + inner) / 2;
}

/** Returns the center radius for the named stratum band. */
export function radiusForStratum(band: StratumBand, domeRadius: number): number {
	const idx = STRATUM_BANDS.indexOf(band);
	if (idx < 0) return radiusForBandIndex(0, domeRadius); // fallback to rim
	// STRATUM_BANDS is inner→outer; bandBoundaries is outer→inner.
	// Band 'foundation' (idx=0 in STRATUM_BANDS) sits at the innermost
	// ring → bandBoundaries index 4. Convert: bandIndex = 4 - idx.
	return radiusForBandIndex(4 - idx, domeRadius);
}

/** Maps the cache's raw stratum integer (1..8 from sky_nodes; NULL
 *  for unclassified) into the v6 5-band scheme.
 *
 *  Raw 1, 2 → Foundation (innermost; the deepest "I'm certain" notes)
 *  Raw 3, 4 → Working knowledge
 *  Raw 5, 6 → Connection layer
 *  Raw 7    → Synthesis
 *  Raw 8 OR NULL → Edge of Knowing (orphans + frontier notes)
 *
 *  Matches the §A.4 progressive backfill tier filter so notes land
 *  in the same band visually as they did in the backfill pass that
 *  populated them.
 */
export function bandForRawStratum(rawStratum: number | null): StratumBand {
	if (rawStratum === null) return 'edge-of-knowing';
	if (rawStratum <= 2) return 'foundation';
	if (rawStratum <= 4) return 'working';
	if (rawStratum <= 6) return 'connection';
	if (rawStratum === 7) return 'synthesis';
	return 'edge-of-knowing';
}

// ════════════════════════════════════════════════════════════════════
// Calendar rim (12 months around the outer rim)
// ════════════════════════════════════════════════════════════════════

/** Returns the 12 calendar rim month label positions (Gregorian per
 *  Concept Paper §2.2 default; multi-calendar deferred to v4.1).
 *  Each label sits at the wedge CENTER (15°, 45°, ..., 345° from
 *  north, clockwise).
 *
 *  Label radius is offset OUTSIDE the dome by `labelOffset` pixels.
 */
export function calendarRimMonths(
	domeRadius: number,
	locale: string,
	labelOffset = 18,
): MonthLabel[] {
	const fmt = new Intl.DateTimeFormat(locale, { month: 'short' });
	const labelRadius = domeRadius + labelOffset;
	return Array.from({ length: 12 }, (_, m) => {
		const angle = m * (Math.PI / 6) + Math.PI / 12 - Math.PI / 2;
		const x = Math.cos(angle) * labelRadius;
		const y = Math.sin(angle) * labelRadius;
		return {
			monthIndex: m,
			angle,
			x,
			y,
			label: fmt.format(new Date(Date.UTC(2024, m, 15))),
		};
	});
}

/** Returns the wedge spoke angles. Used for §A.9 angular positioning
 *  + any future §B mini-dome that wants to overlay rim spokes. */
export function calendarRimSpokes(): number[] {
	return Array.from({ length: 12 }, (_, m) => m * (Math.PI / 6) - Math.PI / 2);
}

// ════════════════════════════════════════════════════════════════════
// Suwaidi palette (v6 — neutral-fill star aesthetic)
// ════════════════════════════════════════════════════════════════════

/**
 * v6 palette per `docs/sight-redesign-v0.3-full-layout.svg` visual
 * contract. Library color is GONE (shape-only encoding per §4 commit);
 * stars render with neutral fill `#cdd5e0`. Hue is reserved for stage
 * (in mini-dome + anchor pip) and link line color.
 *
 * CIE Delta-E ≥30 between any two co-rendered hues — verified at
 * §D.4 build gate per Concept Paper §11 invariant 4.
 */
/**
 * MIG-027 (2026-05-17) — split PALETTE into theme-aware chrome
 * colors + theme-agnostic semantic colors.
 *
 * Before: PALETTE was a single const with hardcoded dark values for
 * everything. Sight always rendered in its dark "starfield" palette
 * regardless of the user's active interface theme.
 *
 * After: chrome colors (bg, strataRing, label text, starFill) are
 * derived at paint time from the document's CSS variables — they
 * automatically follow whatever theme is active. Semantic colors
 * (stage hues, link types, gold highlight ring) stay categorical
 * since their meaning depends on the hue, not the theme — cyan is
 * always 'spark' regardless of whether the UI is light or dark.
 *
 * The legacy `PALETTE` export below stays as a backwards-compat
 * default; it merges the dark-fallback chrome with semantic colors,
 * so callers that haven't migrated yet still get the original
 * appearance. New canvas-drawing code should use
 * `readChromePalette(el)` to get theme-aware values + import
 * `SEMANTIC_COLORS` directly for stage/link hues.
 */

/**
 * MIG-027 — Theme-agnostic categorical hues (stage, link-type).
 * These do NOT change with theme since their meaning is tied to
 * the hue itself (cyan = spark, green = established). Kept as a
 * single const for ergonomic import across canvas renderers.
 *
 * MIG-027 §-fix-2 (2026-05-17): `highlightedRing` moved OUT of
 * SEMANTIC_COLORS into ChromePalette. Rationale: it's an
 * interaction affordance (visual treatment when user hovers
 * something), not a data category like stage hue. It also needs
 * to adapt across themes — bright amber works on dark bg but
 * washes out on cream bg; the chrome side reads the
 * `--sight-highlight` CSS var which is theme-conditional via
 * +layout.svelte's theme classes.
 */
export const SEMANTIC_COLORS = {
	// Stage hues (6 categorical, used as inner pip on anchor + full
	// disk in mini). Per Concept Paper §3.4 + §11 invariant 4: CIE
	// Delta-E ≥30 between any two co-rendered hues.
	// 2026-05-15 §B.6-fix-2: added stageBirth (orange) per Eisa cycle-2
	// feedback — splitting birth from growth gives the two largest
	// categories maximally distinct hues (cyan vs orange).
	stageEstablished: '#4ade80',    // green
	stageFresh:        '#22d3ee',    // cyan — Living Link 'spark'
	stageBirth:        '#fb923c',    // orange — Living Link 'birth'
	stageGrowing:      '#a78bfa',    // violet
	stageAtRisk:       '#facc15',    // yellow
	stageDormant:      '#94a3b8',    // gray

	// Typed-link line colors (Concept Paper §3.4 — 9 kinds)
	linkSupports:     '#4ade80',
	linkContradicts:  '#f87171',    // dashed
	linkCauses:       '#fb923c',
	linkExemplifies:  '#60a5fa',
	linkGeneralizes:  '#a78bfa',
	linkDerivesFrom:  '#22d3ee',
	linkPartOf:       '#f472b6',
	linkAssociative:  '#94a3b8',
	linkSupersedes:   '#fde68a',
} as const;

/**
 * MIG-027 — Theme-aware chrome palette. The 8 fields below are
 * derived at paint time from the document body's CSS variables
 * (which the +layout.svelte $effect updates whenever activeThemeId
 * or system color-scheme changes — see store.ts deriveThemeVariables).
 *
 * `readChromePalette(el)` reads the live values; if CSS vars are
 * unavailable (e.g., very-early-boot, SSR), it falls back to the
 * dark constants below.
 */
export interface ChromePalette {
	bg: string;
	strataRing: string;
	calendarRimText: string;
	stratumLabel: string;
	titleText: string;
	subtitleText: string;
	statusText: string;
	starFill: string;
	/** MIG-027 §-fix-2: gold hover/selection ring used in linked
	 *  brushing across surfaces (anchor highlight, mini-dome hover ring).
	 *  Reads --sight-highlight CSS var which defaults to bright amber
	 *  (#fbbf24) on dark themes and a deeper amber (#b45309) on light
	 *  themes via body.theme-light override in SightV6.svelte. Promoted
	 *  from SEMANTIC_COLORS because it's an interaction affordance, not
	 *  a data-category color, and it needs to adapt across themes. */
	highlightedRing: string;
}

/** MIG-027 — Dark-theme fallback chrome palette. Original Sight v6
 *  cycle-1/cycle-2 values; used when CSS variables are unavailable
 *  (early boot, SSR) or when a tradition renderer doesn't pass a
 *  palette to the chrome-drawing helper.
 *
 *  2026-05-14 §A.14 fix-1 (Boss-test cycle 1): chrome bumped 40-80%.
 *  2026-05-14 §A.14 fix-6 (Boss-test cycle 2): Eisa "fonts need to
 *  be clearer. Change the font color to white at 100% opacity."
 *  Calendar months go to near-white; stratum italic labels go to
 *  readable mid-bright (still italic for hierarchy). Strata rings
 *  stay at the cycle-1 #2a3245 — they're geometry, not text. */
export const CHROME_PALETTE_DARK_FALLBACK: ChromePalette = {
	bg: '#080c16',                  // dome background (deep navy-black)
	strataRing: '#2a3245',          // 5 concentric guides
	calendarRimText: '#e8ebf2',     // 12 month labels — near-white
	stratumLabel: '#a0aabe',        // vertical-axis labels — mid-bright italic
	titleText: '#e8ebf2',           // header strip text
	subtitleText: '#5a6275',
	statusText: '#7a8295',
	starFill: '#cdd5e0',            // NEUTRAL — library encoded by shape only
	highlightedRing: '#fbbf24',     // §-fix-2: bright amber — default for dark themes
};

/**
 * MIG-027 — Read theme-aware chrome palette from CSS variables on a
 * DOM element (typically the Sight canvas-host element). Returns the
 * dark fallback when DOM/CSS-vars are unavailable.
 *
 * The CSS variables are applied to document.body by +layout.svelte's
 * theme $effect (which calls deriveThemeVariables on the active
 * theme's 5 colors). So any element in the document tree inherits
 * the active theme's vars; reading them with getComputedStyle gives
 * the live theme-correct values.
 *
 * Field mapping (MIG-027 design choices):
 *   bg              → --background-primary       (main app bg)
 *   strataRing      → --background-modifier-border (subtle geometry color)
 *   calendarRimText → --text-normal              (primary text — month labels)
 *   stratumLabel    → --text-muted               (secondary — italic labels)
 *   titleText       → --text-normal
 *   subtitleText    → --text-faint
 *   statusText      → --text-muted
 *   starFill        → --text-normal              (stars get theme text color
 *                                                 so they invert: cream on dark,
 *                                                 dark on light)
 */
export function readChromePalette(el: HTMLElement | null): ChromePalette {
	if (!el || typeof document === 'undefined') {
		return CHROME_PALETTE_DARK_FALLBACK;
	}
	const cs = getComputedStyle(el);
	const get = (name: string, fallback: string) =>
		cs.getPropertyValue(name).trim() || fallback;
	return {
		bg: get('--background-primary', CHROME_PALETTE_DARK_FALLBACK.bg),
		strataRing: get('--background-modifier-border', CHROME_PALETTE_DARK_FALLBACK.strataRing),
		calendarRimText: get('--text-normal', CHROME_PALETTE_DARK_FALLBACK.calendarRimText),
		stratumLabel: get('--text-muted', CHROME_PALETTE_DARK_FALLBACK.stratumLabel),
		titleText: get('--text-normal', CHROME_PALETTE_DARK_FALLBACK.titleText),
		subtitleText: get('--text-faint', CHROME_PALETTE_DARK_FALLBACK.subtitleText),
		statusText: get('--text-muted', CHROME_PALETTE_DARK_FALLBACK.statusText),
		starFill: get('--text-normal', CHROME_PALETTE_DARK_FALLBACK.starFill),
		// §-fix-2: --sight-highlight is set on .sight-v6-root + overridden
		// by :global(body.theme-light) .sight-v6-root in SightV6.svelte.
		// CSS vars cascade through DOM, so reading from the canvas host
		// (a descendant of .sight-v6-root) picks up the theme-conditional
		// value automatically.
		highlightedRing: get('--sight-highlight', CHROME_PALETTE_DARK_FALLBACK.highlightedRing),
	};
}

/**
 * Legacy unified palette. Merges the dark-fallback chrome with
 * semantic colors. Kept for backwards-compat with consumers that
 * haven't migrated to readChromePalette + SEMANTIC_COLORS yet.
 *
 * New code: import SEMANTIC_COLORS + use readChromePalette(el).
 * Mid-MIG-027 transition: existing consumers continue to work
 * unchanged via PALETTE; consumers are updated incrementally.
 */
export const PALETTE = {
	...CHROME_PALETTE_DARK_FALLBACK,
	...SEMANTIC_COLORS,
} as const;

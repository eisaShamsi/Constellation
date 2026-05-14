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

/** Human-readable label per stratum band (for the on-canvas label text
 *  along the vertical axis). i18n: these will move to $t() at v4.1
 *  polish; v6.0 ships English-only labels (Concept Paper §10 deferred). */
export const STRATUM_LABELS: Record<StratumBand, string> = {
	foundation: 'FOUNDATION',
	working: 'WORKING',
	connection: 'CONNECTION',
	synthesis: 'SYNTHESIS',
	'edge-of-knowing': 'EDGE OF KNOWING',
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
export const PALETTE = {
	// Background + chrome
	// 2026-05-14 §A.14 fix-1 (Boss-test #2 feedback): chrome too faint
	// at the install build. Strata rings, calendar text, and stratum
	// labels all bumped up roughly 40-80% in luminance so they read
	// clearly without dominating the data.
	bg: '#080c16',                  // dome background (deep navy-black)
	strataRing: '#2a3245',          // 5 concentric guides (was #1a1f2e)
	calendarRimText: '#5a6275',     // 12 month labels (was #3e4453)
	stratumLabel: '#4a5060',        // vertical-axis labels (was #252b3a)
	titleText: '#e8ebf2',           // header strip text
	subtitleText: '#5a6275',
	statusText: '#7a8295',

	// Star encoding
	starFill: '#cdd5e0',            // NEUTRAL — library encoded by shape only
	highlightedRing: '#fbbf24',     // gold ring on hover/selection (linked brushing)

	// Stage hues (5 categorical, used as inner pip on anchor + full disk in mini)
	stageEstablished: '#4ade80',    // green
	stageFresh:        '#22d3ee',    // cyan
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

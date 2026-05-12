/**
 * Sight v5 — dome geometry (pure functions).
 *
 * Per Concept Paper v3.1 §5.2 + Mock B1 (docs/Sight-vNext-MockB1-Toggle.svg):
 * the dome is a circular field with 8 concentric strata bands (L1 at the
 * rim, L8 at the pole), a 12-month calendar rim wrapping the outside,
 * and a soft Milky Way wash drifting across in two diffuse ellipses.
 *
 * All geometry is parameterized on `domeRadius` — the dome scales to
 * the container; the math is unitless. Mock B1 uses domeRadius=310 px.
 *
 * MIG-024 §3 deliverable. Pure functions; no DOM, no Canvas, no state.
 */

/** A single calendar rim month label position. The label itself is
 *  rendered as an HTML overlay (not canvas-drawn text) per v3
 *  invariant 12 — `dir="auto"` handles RTL locales automatically. */
export interface MonthLabel {
	monthIndex: number;          // 0..11
	angle: number;               // radians; 0 = top (north), clockwise
	x: number;                   // pixel offset from dome center
	y: number;                   // pixel offset (y grows DOWN per Canvas convention)
	label: string;               // locale-aware short month name
}

/** A Milky Way wash ellipse — the diffuse content-similarity density
 *  texture. Two ellipses combine to form the "drifting across the
 *  chart" pattern in Mock B1. */
export interface MilkyWayEllipse {
	cx: number;
	cy: number;
	rx: number;
	ry: number;
	rotationDeg: number;         // applied at (cx, cy) center
}

/** Returns the 8 strata band BOUNDARIES from outer rim to pole.
 *
 * Index 0 = outer rim radius (L1 boundary out).
 * Index 7 = innermost ring radius (L8 boundary out).
 * The pole itself (L8 innermost) is implicit at radius 0.
 *
 * Mock B1 uses domeRadius=310 with bands at:
 *   [310, 271, 232, 194, 155, 116, 78, 39] (close to evenly spaced).
 *
 * Pure linear distribution: 7 rings divide the dome into 8 bands.
 */
export function stratumBandBoundaries(domeRadius: number): number[] {
	// 8 boundaries (outer + 7 inner rings). Linear spacing: r_n = domeRadius * (8-n)/8 for n in [0..7],
	// where n=0 is the outer rim and n=7 is the innermost ring.
	return Array.from({ length: 8 }, (_, n) => (domeRadius * (8 - n)) / 8);
}

/** Returns the CENTER radius of band n (n = 1..8). A star at stratum n
 *  is positioned at this radius. Mode-invariant: this is what makes
 *  the four constants (Concept Paper §7) hold across mode toggles.
 *
 *  Band centers (for domeRadius=310):
 *    L1 ~ 290.6  (outer band: from 310 to 271)
 *    L2 ~ 251.8  (271 to 232)
 *    L3 ~ 213.1
 *    L4 ~ 174.4
 *    L5 ~ 135.6
 *    L6 ~ 96.9
 *    L7 ~ 58.1
 *    L8 ~ 19.4   (innermost band: 39 to 0 — the pole)
 *
 *  An invalid stratum (out of [1, 8]) clamps to [1, 8] then computes.
 *  Unrecognized strata (null/undefined from the cache) should not
 *  reach this function; callers fall back to "Unstratified" handling.
 */
export function radiusForStratum(stratum: number, domeRadius: number): number {
	const n = Math.max(1, Math.min(8, Math.round(stratum)));
	const outer = (domeRadius * (8 - (n - 1))) / 8;  // outer boundary of band n
	const inner = (domeRadius * (8 - n)) / 8;          // inner boundary of band n
	return (outer + inner) / 2;
}

/** Returns the 12 calendar rim month label positions (Gregorian default
 *  per Concept Paper §7.2). Each label sits at the wedge CENTER (15°,
 *  45°, ..., 345° from north, clockwise).
 *
 *  Label radius is offset OUTSIDE the dome by `labelOffset` pixels so
 *  the month text doesn't overlap the strata bands. Mock B1 uses
 *  ~12-22 px outside the rim.
 *
 *  Locale-aware month names via Intl.DateTimeFormat — handles all 15
 *  Constellation locales without per-locale lookup tables.
 */
export function calendarRimMonths(
	domeRadius: number,
	locale: string,
	labelOffset = 18,
): MonthLabel[] {
	const fmt = new Intl.DateTimeFormat(locale, { month: 'short' });
	const labelRadius = domeRadius + labelOffset;
	return Array.from({ length: 12 }, (_, m) => {
		// Wedge centers: 15°, 45°, 75°, ... = (m * 30 + 15) deg = (m * π/6 + π/12) rad.
		// We orient 0 = top (12 o'clock) so subtract π/2 from standard math angle.
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

/** Returns the wedge spoke angles (0°, 30°, ..., 330° in radians,
 *  oriented 0 = north). Used for drawing the faint month-wedge spokes
 *  in Time mode. Other modes may overlay different spoke sets.
 */
export function calendarRimSpokes(): number[] {
	return Array.from({ length: 12 }, (_, m) => m * (Math.PI / 6) - Math.PI / 2);
}

/** Returns the two Milky Way wash ellipses scaled to the given dome
 *  radius. The shape + placement matches Mock B1's visual contract;
 *  scaling preserves the relative geometry.
 *
 *  Mock B1 (domeRadius=310) values:
 *    [{ cx: -80, cy: -50, rx: 180, ry: 60, rotationDeg: 35 },
 *     { cx:  90, cy:  80, rx: 200, ry: 70, rotationDeg: -25 }]
 *  Scaled by domeRadius/310.
 */
export function milkyWayEllipses(domeRadius: number): MilkyWayEllipse[] {
	const k = domeRadius / 310;
	return [
		{ cx: -80 * k, cy: -50 * k, rx: 180 * k, ry: 60 * k, rotationDeg: 35 },
		{ cx: 90 * k, cy: 80 * k, rx: 200 * k, ry: 70 * k, rotationDeg: -25 },
	];
}

/** Suwaidi palette tokens used by render.ts. Hex values match Mock B1's
 *  embedded `<style>` block exactly. */
export const PALETTE = {
	parchment: '#faf6e8',         // dome background
	ink: '#1a1a1a',                // stars, primary text
	inkSoft: '#3a3a3a',            // softer near-black
	ruleFaint: '#b8a98a',          // strata band rings, calendar rim, grid
	gold: '#c9a227',               // active mode, current month tint, selection ring
	blueInk: '#2a4a8c',            // headers, captions
	redInk: '#a83232',             // contested stars
	milkyWay: '#e6dec0',           // Milky Way wash radial gradient
	// Connector-line colors (Concept Paper §5.4 — 9 typed-link kinds).
	linkGreen: '#3a8a4a',          // supports / derives-from
	linkRed: '#a83232',            // contradicts
	linkGold: '#c9a227',           // exemplifies / generalizes
	linkBlue: '#2a4a8c',           // causes / part-of
	linkSlateBlue: '#5B7A8A',      // supersedes (MIG-022 §A.2)
	linkGrey: '#888888',           // associative (untyped)
} as const;

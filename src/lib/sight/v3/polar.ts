/**
 * Sight v3 — pure polar layout math.
 *
 * MIG-019 §2G.2 (Eisa's 2026-05-07 redesign). Per `docs/SIGHT-V3-VISUAL-SPEC.md`
 * §2 (polar grammar), the chart is a single-pole polar projection where:
 *   • radius  = (1 − centrality) × DOME_R + edge_padding
 *   • azimuth = mode-dependent (Regions / Link Types / Time / etc.)
 *   • size    = magnitude bucket from centrality
 *   • alpha   = magnitude-tied opacity
 *
 * No allocations on the hot render path. Functions are stateless. Trig
 * is plain `Math.sin/cos`. Caller passes `cx, cy, dome_r` per-frame so
 * the same helpers work across canvas sizes (panel + second screen).
 */

/** Convention: theta=0 points to TOP (12 o'clock), CW positive.
 *  Matches the SVG mockup generator and the Suwaidi star chart. */
export function polarToCartesian(
    radius: number,
    thetaRad: number,
    cx: number,
    cy: number,
): { x: number; y: number } {
    return {
        x: cx + radius * Math.sin(thetaRad),
        y: cy - radius * Math.cos(thetaRad),
    };
}

/** Inverse of polarToCartesian — recovers (radius, theta) given a point. */
export function cartesianToPolar(
    x: number,
    y: number,
    cx: number,
    cy: number,
): { radius: number; thetaRad: number } {
    const dx = x - cx;
    const dy = cy - y;
    return {
        radius: Math.hypot(dx, dy),
        thetaRad: Math.atan2(dx, dy),  // note: (dx, dy) gives 0=top, CW+
    };
}

/** Star radius from centrality. High centrality → near pole. Low → near rim.
 *  Inner padding so the most-central star isn't on the pole exactly. */
export function radiusFromCentrality(
    centralityNorm: number,
    domeR: number,
    edgePadding: number = 8,
): number {
    const c = Math.max(0, Math.min(1, centralityNorm));
    return (1 - c) * (domeR - edgePadding) + edgePadding;
}

/** Six magnitude buckets, log-distributed by centrality. Matches the
 *  Suwaidi star chart's 6-magnitude convention. */
export function magnitudeSize(centralityNorm: number): number {
    const c = Math.max(0, Math.min(1, centralityNorm));
    if (c > 0.85) return 6.0;
    if (c > 0.65) return 4.0;
    if (c > 0.40) return 2.5;
    if (c > 0.20) return 1.5;
    if (c > 0.08) return 0.9;
    return 0.5;
}

/** Faint stars are slightly translucent; bright stars solid. */
export function magnitudeAlpha(centralityNorm: number): number {
    const c = Math.max(0, Math.min(1, centralityNorm));
    return Math.min(0.55 + c * 0.45, 1.0);
}

/** Interpolate angular position along the SHORTER arc between `from` and
 *  `to`. Used by the 600 ms mode-switch animation (§1.2 of the spec) so
 *  stars don't take the long way around the rim. `t` ∈ [0, 1]. */
export function interpolateAngle(fromRad: number, toRad: number, t: number): number {
    const TAU = Math.PI * 2;
    let delta = ((toRad - fromRad) % TAU + TAU) % TAU;
    if (delta > Math.PI) delta -= TAU;
    return fromRad + delta * t;
}

/** Pastel hue cycle from spec §2.3. 8 hues; communities map by `id % 8`. */
export const COMMUNITY_HUES: ReadonlyArray<string> = Object.freeze([
    '#7c8a9e',  // pale slate-blue
    '#9e8a7c',  // pale clay
    '#7c9e8a',  // pale moss
    '#8a7c9e',  // pale lavender
    '#9e9e7c',  // pale olive
    '#7c9e9e',  // pale teal
    '#9e7c8a',  // pale rose
    '#8a9e7c',  // pale sage
]);

/** Color a community by its assigned id. */
export function communityColor(communityId: number): string {
    const len = COMMUNITY_HUES.length;
    const idx = ((communityId % len) + len) % len;
    return COMMUNITY_HUES[idx];
}

/** Suwaidi palette tokens (spec §6). Re-exported here so render code
 *  doesn't need a separate palette module. */
export const PALETTE = Object.freeze({
    BG: '#faf6e8',
    INK: '#1a1a1a',
    INK_SOFT: '#3a3a3a',
    RULE_FAINT: '#b8a98a',
    GOLD: '#c9a227',
    CYAN: '#2b8fa8',
    RED_INK: '#a83232',
    BLUE_INK: '#2a4a8c',
    MILKY: '#e6dec0',
    HEALTHY: '#3a8a4a',
    CAUTION: '#c9831f',
    CRITICAL: '#a83232',
} as const);

/** Dome geometry tokens (spec §4.2). The dome is sized to viewport at
 *  render time; these are the *ratios* relative to dome radius. */
export const DOME_RATIOS = Object.freeze({
    DEC_RING_FRACS: [0.25, 0.50, 0.75, 1.00] as ReadonlyArray<number>,
    ECLIPTIC_FRAC: 0.40,
    EQUATOR_FRAC: 0.70,
    EDGE_PADDING_PX: 8,
});

/** Animation tokens (spec §1.2). */
export const ANIMATION = Object.freeze({
    /** Mode-switch eased migration. */
    MODE_SWITCH_MS: 600,
    /** CSS-equivalent: cubic-bezier(0.4, 0, 0.2, 1). Used both in
     *  CSS transitions and JS-driven animation frames. */
    EASE_MODE_SWITCH: (t: number) => {
        // Approximation of cubic-bezier(0.4, 0, 0.2, 1) via Hermite-like ease
        const p = Math.max(0, Math.min(1, t));
        return p < 0.5
            ? 2 * p * p
            : 1 - Math.pow(-2 * p + 2, 2) / 2;
    },
} as const);

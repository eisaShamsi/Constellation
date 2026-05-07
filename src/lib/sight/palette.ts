/**
 * Sight v3 — Suwaidi warm-cream + gold palette (Eisa's design call 2026-05-07).
 *
 * Cycled by Louvain community id (id mod 8 → palette index). Replaces the
 * generic 8-color cluster palette inherited from Sky View / clusterEngine.
 * The tones are inspired by 19th-century printed star charts: cream stars
 * on midnight ground, warm gold accents for major bright stars + the
 * mythological figures.
 *
 * User can override per-community via the existing Style Settings system
 * (see clusterEngine.ts overrideClusterColor pattern). MIG-018 §1E ships
 * the default cycle; per-community override surfaces in MIG-019.
 */
export const SUWAIDI_PALETTE: readonly string[] = [
    '#f5e6c8', // cream — primary star color, also community 0
    '#d4af37', // gold — bright accent, community 1
    '#c9a227', // amber — slightly darker gold, community 2
    '#c97f63', // dusty rose — warm earth-tone, community 3
    '#d2b48c', // sandy tan — neutral warm, community 4
    '#e8d8b8', // parchment — pale warm, community 5
    '#f0e0c0', // antique-white — cream variant, community 6
    '#b8860b', // dark goldenrod — deepest amber, community 7
];

/** Hex string ('#rrggbb') → 0xRRGGBB integer for Pixi tint/fill. */
export function hexToInt(hex: string): number {
    return parseInt(hex.slice(1), 16);
}

/** Cycle through the Suwaidi palette by community id. */
export function communityColor(communityId: number): string {
    const idx = ((communityId % SUWAIDI_PALETTE.length) + SUWAIDI_PALETTE.length) % SUWAIDI_PALETTE.length;
    return SUWAIDI_PALETTE[idx];
}

export function communityColorInt(communityId: number): number {
    return hexToInt(communityColor(communityId));
}

/** Background midnight blue (Suwaidi-chart deep sky). */
export const SKY_BACKGROUND = 0x0f1729;

/** Faint connector-line color at rest (Suwaidi cream, very low alpha). */
export const CONNECTOR_LINE_COLOR = 0xf5e6c8;

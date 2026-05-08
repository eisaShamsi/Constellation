/**
 * Sight v3 — per-library deterministic color palette.
 *
 * MIG-019 §2G.3f (Eisa's 2026-05-07 directive). Library colors aren't
 * centrally stored in Constellation today (impact-report finding), so
 * we derive them deterministically from the library's stable position
 * in the wedge order. That keeps colors consistent across renders /
 * sessions without requiring backend work — and the user can later
 * customise via Settings → Appearance if desired.
 *
 * Color goals on the cream parchment background (#faf6e8):
 *   • Saturated enough to stand out against cream
 *   • Not so dark they collide with pure-black stroke (we want the
 *     stroke to read as a contrast frame)
 *   • Spread evenly around the hue wheel for distinguishability
 *   • Stable per library — don't shuffle on every load
 */

const TAU = Math.PI * 2;

/** HSL → RGB integer (0xRRGGBB). h in [0, 360), s and l in [0, 1]. */
export function hslToRgbInt(h: number, s: number, l: number): number {
    const hp = ((h % 360) + 360) % 360 / 360;
    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs((hp * 6) % 2 - 1));
    const m = l - c / 2;
    let r1 = 0, g1 = 0, b1 = 0;
    const seg = Math.floor(hp * 6);
    if (seg === 0) { r1 = c; g1 = x; b1 = 0; }
    else if (seg === 1) { r1 = x; g1 = c; b1 = 0; }
    else if (seg === 2) { r1 = 0; g1 = c; b1 = x; }
    else if (seg === 3) { r1 = 0; g1 = x; b1 = c; }
    else if (seg === 4) { r1 = x; g1 = 0; b1 = c; }
    else { r1 = c; g1 = 0; b1 = x; }
    const r = Math.round((r1 + m) * 255);
    const g = Math.round((g1 + m) * 255);
    const b = Math.round((b1 + m) * 255);
    return (r << 16) | (g << 8) | b;
}

/** Color a library by its index in the wedge order.
 *
 *  Uses a low-discrepancy hue sequence (golden-angle increment) so
 *  even with N=20 libraries, neighbours on the hue wheel stay well
 *  separated. Saturation 55 %, lightness 45 % — readable on cream.
 */
export function libraryColorByIndex(
    libraryIndex: number,
    totalLibraries: number,
): { hex: number; css: string } {
    // Golden-angle = 137.50776...° in degrees → use this stride so
    // consecutive indices visit different hue regions.
    const goldenAngle = 137.50776405003785;
    // Anchor at 30° (warm-ish) so library #1 is in a comfortable
    // earth-tone band rather than pure red.
    const hue = (30 + libraryIndex * goldenAngle) % 360;
    const sat = 0.55;
    const lightness = 0.42;
    const hex = hslToRgbInt(hue, sat, lightness);
    const css = `hsl(${hue.toFixed(1)}, ${sat * 100}%, ${lightness * 100}%)`;
    return { hex, css };
}

/** Build a per-library color map for the active wedge order.
 *
 *  Caller is `SightV3.svelte` after the region layout is built —
 *  pass the wedges in their final sorted order (by note count desc).
 *  Returned map is keyed by library_path so render-loop lookups
 *  match the same key the polar dispatcher uses.
 */
export function buildLibraryColorMap(
    wedges: ReadonlyArray<{ libraryPath: string }>,
): Map<string, { hex: number; css: string; index: number }> {
    const map = new Map<string, { hex: number; css: string; index: number }>();
    const total = wedges.length;
    wedges.forEach((w, i) => {
        const { hex, css } = libraryColorByIndex(i, total);
        map.set(w.libraryPath, { hex, css, index: i + 1 });  // 1-indexed labels
    });
    return map;
}

/** Suppress unused TAU warning if a future revision needs it. */
void TAU;

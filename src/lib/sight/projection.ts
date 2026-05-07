/**
 * Sight v3 — projection from unit-disk embedding to screen coordinates.
 *
 * Two projection modes are supported (Eisa's design call §11 Q2,
 * 2026-05-07: ship both, user-toggle in Settings):
 *
 *   - Lambert azimuthal equal-area (default): preserves area, so
 *     constellation territory sizes are visually proportional to
 *     community node count.
 *   - Stereographic (alternative): preserves angles, so constellation
 *     shapes look "right" near the dome edge but community sizes
 *     mislead.
 *
 * Both projections take a unit-disk embedding `(embedX, embedY)` from
 * the Rust Landmark-MDS output (`sight_layout::compute_layout_embedding`)
 * and map it to screen-pixel coordinates given a viewport.
 *
 * The projection is applied at *render time*, not cached — switching
 * between Lambert and stereographic is free (no IPC, no recompute).
 *
 * See `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` §3.1 Stage B.
 */

export type ProjectionMode = 'lambert' | 'stereographic';

export interface Viewport {
    /** Center x in screen pixels. */
    cx: number;
    /** Center y in screen pixels. */
    cy: number;
    /** Half the smaller of (viewport width, viewport height). The
     *  unit-disk maps to a circle of this radius. */
    radius: number;
}

/**
 * Project a unit-disk embedding point to screen coordinates.
 *
 * Both modes share the angle θ = atan2(embedY, embedX). They differ
 * only in how the disk-radius r ∈ [0, 1] maps to a screen-radius r'.
 *
 * Lambert (equal-area):
 *   r' = √(2 · (1 − cos(θ_polar)))   where θ_polar = atan(r) · 2
 *   Equivalently: r' = 2 · sin(atan(r) / 2) · viewport.radius
 *
 * Stereographic (equal-angle):
 *   r' = 2 · tan(θ_polar / 2)   where θ_polar = atan(r) · 2
 *   Equivalently: r' = 2 · tan(atan(r) / 2) · viewport.radius
 *
 * Since the input is already on the unit disk (max radius ≈ 0.95 per
 * Rust's `normalize_to_unit_disk`), both projections behave well — no
 * point ever maps to infinity (which would happen for stereographic at
 * exactly r = 1).
 */
export function embedToScreen(
    embedX: number,
    embedY: number,
    mode: ProjectionMode,
    viewport: Viewport,
): { x: number; y: number } {
    const r = Math.hypot(embedX, embedY);
    if (r < 1e-9) {
        return { x: viewport.cx, y: viewport.cy };
    }
    const theta = Math.atan2(embedY, embedX);
    const polarTheta = Math.atan(r) * 2;

    let rPrime: number;
    if (mode === 'lambert') {
        rPrime = 2 * Math.sin(polarTheta / 2);
    } else {
        rPrime = 2 * Math.tan(polarTheta / 2);
    }

    // Both formulas produce r' in [0, ~1.4] for r ∈ [0, 0.95]; we want
    // them to map ~0.95 → viewport.radius. Empirical scale factor:
    //   Lambert at r=0.95: r' = 2·sin(atan(0.95)) = 2·0.6889 = 1.378  → / 1.45 ≈ 0.95
    //   Stereographic at r=0.95: r' = 2·tan(atan(0.95)/2) = 2·0.4382 = 0.876 → /0.92 ≈ 0.95
    // Choose a per-mode scale so 0.95 → viewport.radius:
    const scale = (mode === 'lambert' ? 1 / 1.378 : 1 / 0.876) * viewport.radius;

    return {
        x: viewport.cx + rPrime * Math.cos(theta) * scale,
        y: viewport.cy + rPrime * Math.sin(theta) * scale,
    };
}

/**
 * MIG-025 §A.6 — Sight v6 anchor dome renderer (skeleton).
 *
 * Stub exports for the §A.8 (chrome) + §A.9 (stars + lines)
 * implementation. Body is unimplemented until those steps land;
 * this skeleton exists so SightV6.svelte can import + reference
 * without compile errors during the phased build.
 *
 * Visual contract: docs/sight-redesign-v0.3-full-layout.svg
 *
 * References:
 *   docs/Constellation-Sight-Concept-Paper-v4.0.md  §2.2, §3
 *   docs/sight-redesign-v0.3-full-layout.svg
 *   src/lib/sight/v6/dome.ts                         (geometry helpers — §A.8)
 */

import type { LayoutCacheRow, LinkEdge, StarDerived } from './types';

/**
 * Render the anchor dome to a Canvas 2D context.
 *
 * §A.8 lands the chrome: 5 strata circles, calendar rim, stratum
 * labels. §A.9 lands the stars + connector lines per the channel
 * encoding (shape from library, opacity from confidence, pip from
 * stage, size from acts top-decile, line color from typed-link kind).
 *
 * Per Concept Paper §11 invariant 2 (Suwaidi-fidelity): the anchor
 * occupies ≥80% of the visible canvas in default state.
 */
export function renderAnchorDome(
	_ctx: CanvasRenderingContext2D,
	_stars: StarDerived[],
	_links: LinkEdge[],
	_width: number,
	_height: number
): void {
	// §A.8 + §A.9 implementation
	throw new Error('renderAnchorDome: not implemented until §A.8/§A.9');
}

/**
 * Compute per-star (x, y) positions from cache rows + Universe
 * context. Radial = stratum band, angular = month-of-creation
 * (January at top, clockwise). Library shape index, top-decile-acts
 * flag, and provenance sector are derived here from the raw row +
 * Universe-wide aggregates (e.g., the acts-density 90th-percentile
 * cutoff for the top-decile flag).
 */
export function computeStarPositions(
	_rows: LayoutCacheRow[],
	_centerX: number,
	_centerY: number,
	_outerRadius: number
): StarDerived[] {
	// §A.9 implementation — port v5/render.ts::computeStarPositions
	// adapted for v6's 5-band stratum + new derived-field computation.
	throw new Error('computeStarPositions: not implemented until §A.9');
}

/**
 * Hit-test for star clicks. Returns the note path of the closest
 * star within the click tolerance, or null if no hit. Used by the
 * gesture grammar (Concept Paper §5: hover-star, click-star).
 */
export function starHitTest(
	_stars: StarDerived[],
	_x: number,
	_y: number,
	_tolerancePx: number
): string | null {
	// §A.9 implementation — port v5/render.ts::wedgeKeyAtPoint
	// adapted for per-star Euclidean distance.
	throw new Error('starHitTest: not implemented until §A.9');
}

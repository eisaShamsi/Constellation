/**
 * MIG-025 §A.6/§A.8 — Sight v6 anchor dome renderer.
 *
 * §A.6 shipped the stub exports.
 * §A.8 (this commit) lands the chrome render: background fill,
 * 5 strata circles, calendar rim labels, vertical-axis stratum
 * labels. Per Concept Paper v4.0 §2.2 + the v0.3 visual contract.
 *
 * §A.9 (next step) lands the stars + connector lines per the channel
 * encoding (shape from library, opacity from confidence, pip from
 * stage, size from acts top-decile, line color from typed-link kind).
 *
 * Per §11 invariant 2 (Suwaidi-fidelity): the anchor occupies ≥80%
 * of the visible canvas in default state. The §A.13 CI test enforces
 * this at the layout level; this render code respects it by sizing
 * the dome to fit minus a small calendar-rim margin.
 *
 * Visual contract: docs/sight-redesign-v0.3-full-layout.svg
 */

import type { LayoutCacheRow, LinkEdge, StarDerived } from './types';
import {
	PALETTE,
	STRATUM_BANDS,
	STRATUM_LABELS,
	calendarRimMonths,
	radiusForStratum,
	stratumBandBoundaries,
} from './dome';

/**
 * Dome layout for a given canvas size. Pure function; no Canvas
 * touch. The renderer + hit-test both consume this so geometry
 * decisions live in one place.
 */
export interface DomeLayout {
	/** Center X in canvas coords. */
	centerX: number;
	/** Center Y in canvas coords. */
	centerY: number;
	/** Outer dome radius (Edge of Knowing rim). Excludes calendar
	 *  label margin. */
	radius: number;
}

/**
 * Compute the dome layout for a given canvas size. The dome is
 * centered horizontally and vertically; radius is sized to the
 * smaller dimension minus a margin for the calendar rim labels
 * (which sit OUTSIDE the dome at radius + 18 px per `calendarRimMonths`).
 *
 * The §11 invariant-2 ≥80%-anchor target is honored by occupying
 * the full canvas-host minus the small label margin.
 */
export function computeDomeLayout(width: number, height: number): DomeLayout {
	const labelMargin = 28; // 18 px rim offset + 10 px text bleed
	const radius = Math.max(40, Math.min(width, height) / 2 - labelMargin);
	return {
		centerX: width / 2,
		centerY: height / 2,
		radius,
	};
}

/**
 * Render the anchor dome to a Canvas 2D context.
 *
 * §A.8 — chrome only (background + strata rings + calendar rim +
 * stratum labels). §A.9 will add the stars + lines render call
 * after the chrome layer.
 *
 * Caller must clear the canvas (or call this with `clear=true`)
 * before rendering — this function does not assume an empty canvas.
 */
export function renderAnchorDome(
	ctx: CanvasRenderingContext2D,
	_stars: StarDerived[],
	_links: LinkEdge[],
	width: number,
	height: number,
	options: { locale?: string; clear?: boolean } = {},
): void {
	const { locale = 'en', clear = true } = options;
	const layout = computeDomeLayout(width, height);

	if (clear) {
		ctx.clearRect(0, 0, width, height);
	}

	// 1. Background fill (deep Suwaidi navy).
	ctx.fillStyle = PALETTE.bg;
	ctx.fillRect(0, 0, width, height);

	// 2. Five strata reference circles. Drawn at the band BOUNDARIES
	//    (5 outer-to-inner radii). 0.6 px stroke, very faint —
	//    Suwaidi-style guide rings, not heavy chrome.
	ctx.strokeStyle = PALETTE.strataRing;
	ctx.lineWidth = 0.6;
	const boundaries = stratumBandBoundaries(layout.radius);
	for (const r of boundaries) {
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}

	// 3. Calendar rim — 12 month labels at outer radius + 18 px.
	//    Locale-aware via Intl.DateTimeFormat (15 Constellation
	//    locales handled without per-locale tables).
	ctx.fillStyle = PALETTE.calendarRimText;
	ctx.font = '10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	for (const m of calendarRimMonths(layout.radius, locale, 18)) {
		ctx.fillText(m.label, layout.centerX + m.x, layout.centerY + m.y);
	}

	// 4. Stratum labels along the vertical axis (centered horizontally
	//    on the dome center; positioned at each band's center radius
	//    above the center, since the labels go up the page).
	//    Foundation = innermost (closest to center); Edge of Knowing
	//    = outermost (closest to top).
	ctx.fillStyle = PALETTE.stratumLabel;
	ctx.font = 'italic 9px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	for (const band of STRATUM_BANDS) {
		const r = radiusForStratum(band, layout.radius);
		// Place label ABOVE center (negative y in Canvas convention)
		// at the band's center radius. Slight inset so labels don't
		// touch the rings exactly.
		const y = layout.centerY - r;
		ctx.fillText(STRATUM_LABELS[band], layout.centerX, y);
	}

	// 5. Stars + lines — §A.9 implementation. Stub for now.
	// TODO §A.9: drawConnectorLines(ctx, _links, layout);
	// TODO §A.9: drawStars(ctx, _stars, layout);
}

/**
 * Compute per-star (x, y) positions from cache rows + Universe
 * context. §A.9 implementation — stub for now.
 */
export function computeStarPositions(
	rows: LayoutCacheRow[],
	_centerX: number,
	_centerY: number,
	_outerRadius: number,
): StarDerived[] {
	// §A.9 implementation stub — return empty array so the §A.8 chrome
	// renders without errors. The full computeStarPositions lands in
	// §A.9 (radial = bandForRawStratum + jitter; angular = month).
	return rows.length ? [] : [];
}

/**
 * Hit-test for star clicks. §A.9 implementation stub.
 */
export function starHitTest(
	_stars: StarDerived[],
	_x: number,
	_y: number,
	_tolerancePx: number,
): string | null {
	return null;
}

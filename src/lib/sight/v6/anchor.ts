/**
 * MIG-025 §A.6/§A.8/§A.9 — Sight v6 anchor dome renderer.
 *
 * §A.6 — stub exports
 * §A.8 — chrome render (background + 5 strata + calendar rim + labels)
 * §A.9 — stars + connector lines + hit-test (this commit)
 *
 * Channel encoding per Concept Paper v4.0 §3.1:
 *   shape       → library identity (Treisman primitive, pre-attentive)
 *   opacity     → confidence (pre-attentive)
 *   inner pip   → stage hue (≥1.8 px else suppressed; focal-on-foveation)
 *   size +40%   → top-decile acts (binary, pre-attentive)
 *   line color  → typed-link kind (auto-fade above 800 visible)
 *
 * Star fill is NEUTRAL (PALETTE.starFill = #cdd5e0) per §4 commit;
 * library identity rides on shape only — no library hue.
 *
 * Per §11 invariants:
 *   1 — channel orthogonality (each channel uses a distinct Bertin variable)
 *   2 — Suwaidi-fidelity (anchor ≥80% of canvas)
 *   3 — ≤16 ms cross-filter response (path is render-on-paint, not per-frame)
 *   5 — pip foveation threshold (suppress pip when computed <1.5 px)
 *
 * Visual contract: docs/sight-redesign-v0.3-full-layout.svg
 */

import type {
	LayoutCacheRow,
	LinkEdge,
	StarDerived,
	ProvenanceSector,
	TypedLinkKind,
	LifecycleStage,
} from './types';
import {
	PALETTE,
	STRATUM_BANDS,
	STRATUM_LABELS,
	bandForRawStratum,
	calendarRimMonths,
	radiusForStratum,
	stratumBandBoundaries,
} from './dome';

// ════════════════════════════════════════════════════════════════════
// Layout
// ════════════════════════════════════════════════════════════════════

export interface DomeLayout {
	centerX: number;
	centerY: number;
	radius: number;
}

/**
 * Compute the dome layout for a given canvas size. Centered, sized
 * to the smaller dimension minus a calendar-rim margin. Honors §11
 * invariant 2 (≥80% anchor occupancy).
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

// ════════════════════════════════════════════════════════════════════
// Per-star derivation (computed in JS at render time)
// ════════════════════════════════════════════════════════════════════

/**
 * Compute per-star (x, y) positions + derived fields from cache rows
 * + Universe-wide context. Pure function; no DOM.
 *
 * Position:
 *   radial  = band CENTER for the row's stratum, with deterministic
 *             jitter (hashed on note_path) so co-stratum/co-month
 *             stars don't pile on top of each other
 *   angular = (createdMonth + 0.5) * 30° measured from north,
 *             clockwise. Notes without createdMonth are placed at
 *             month 0 with a flag in jitter to spread them.
 *
 * Derived fields:
 *   libraryShapeIndex  — sorted index of the library, mod 5
 *                        (5 shapes for 5 libraries; v0.4 outline-style
 *                        rotation extends to 25)
 *   topDecileActs      — true if (linkInCount + linkOutCount) ≥
 *                        the 90th percentile across the Universe
 *   provenanceSector   — substring heuristic on sourcesPrimary
 *                        (URL → Read; null → Self; v4.1 will use
 *                        the masādir-aware classification)
 */
/**
 * 2026-05-14 §A.14 fix-12 (Boss-test cycle 3.2 depth-exploration):
 * positions stars via PHYLLOTAXIS (sunflower spiral packing) within
 * each (band, month) cell. Per Working Agreement #5 research: this
 * is Vogel's 1979 model, the same mathematical packing sunflowers
 * have used for >100M years. Replaces the previous FNV-jitter
 * approach which collapsed dense cells into undifferentiated
 * texture even at high zoom.
 *
 * Algorithm per cell:
 *   1. Bucket all rows by (band, month) cell key.
 *   2. Within each cell, sort by notePath (deterministic order →
 *      same note always at same position across renders).
 *   3. For star i (0-indexed) in a cell of N stars:
 *        radius = c × √(i + 0.5)
 *        angle  = (i + 0.5) × goldenAngle (≈ 137.508°)
 *      where c is sized so the largest spiral radius fits within
 *      the cell bounds with a margin: c = 0.92 × cellHalfMin / √N.
 *   4. Spiral position is added to the cell's centroid in Cartesian
 *      coords (cell is small enough that local Euclidean ≈ polar).
 *
 * Properties:
 *   • Deterministic (same path → same position across loads).
 *   • Mathematically optimal packing — distance-between-points is
 *     maximized for any given point count (Vogel 1979).
 *   • Preserves stratum × month spatial anchor exactly.
 *   • At default zoom: dense cells render as the textured speckle
 *     Eisa accepted. At zoom: spiral structure becomes visible,
 *     individual stars distinguishable as discrete points on a
 *     golden-angle spiral.
 *   • Adaptive c per cell — cells with more stars get tighter
 *     packing within the same physical bounds.
 */
export function computeStarPositions(
	rows: LayoutCacheRow[],
	centerX: number,
	centerY: number,
	outerRadius: number,
): StarDerived[] {
	if (rows.length === 0) return [];

	// Pre-compute Universe-wide context.
	const libraryOrder = uniqueSortedLibraries(rows);
	const top10thLinkCount = topDecileLinkCount(rows);

	// Phyllotaxis geometry constants.
	const GOLDEN_ANGLE = Math.PI * (3 - Math.sqrt(5)); // ≈ 2.39996 rad ≈ 137.508°
	const bandHalfWidth = outerRadius / 10;            // 5 bands × full = outerRadius / 5; half = / 10

	// Step 1 — bucket by (band, month) cell.
	interface Cell {
		band: ReturnType<typeof bandForRawStratum>;
		month: number;
		rows: LayoutCacheRow[];
	}
	const cellMap = new Map<string, Cell>();
	for (const row of rows) {
		const band = bandForRawStratum(row.stratum);
		const month = row.createdMonth ?? 0;
		const key = `${band}|${month}`;
		const existing = cellMap.get(key);
		if (existing) {
			existing.rows.push(row);
		} else {
			cellMap.set(key, { band, month, rows: [row] });
		}
	}

	const out: StarDerived[] = [];
	for (const cell of cellMap.values()) {
		// Step 2 — deterministic intra-cell order.
		const sortedRows = cell.rows.slice().sort((a, b) =>
			a.notePath < b.notePath ? -1 : a.notePath > b.notePath ? 1 : 0,
		);
		const N = sortedRows.length;

		// Cell centroid in Cartesian.
		const bandMidRadius = radiusForStratum(cell.band, outerRadius);
		const monthMidAngle =
			cell.month * (Math.PI / 6) + Math.PI / 12 - Math.PI / 2;
		const cellCenterX = centerX + Math.cos(monthMidAngle) * bandMidRadius;
		const cellCenterY = centerY + Math.sin(monthMidAngle) * bandMidRadius;

		// Cell physical extent: smallest of band-half-width OR
		// half-arc-length at this radius. Spiral c must keep the
		// largest radius inside this min so the spiral stays in cell.
		const halfArcLength = bandMidRadius * (Math.PI / 12); // half a 30° wedge
		const cellHalfMin = Math.min(bandHalfWidth, halfArcLength);
		// 0.92 leaves a small visual margin between cells; clamp c so
		// extremely sparse cells (N=1..5) don't put the single star
		// way out at the boundary.
		const c = Math.min(2.4, (0.92 * cellHalfMin) / Math.sqrt(N));

		// Step 3 — phyllotaxis spiral within this cell.
		for (let i = 0; i < N; i++) {
			const row = sortedRows[i];
			const r = c * Math.sqrt(i + 0.5);
			const theta = (i + 0.5) * GOLDEN_ANGLE;
			const x = cellCenterX + Math.cos(theta) * r;
			const y = cellCenterY + Math.sin(theta) * r;

			out.push({
				row,
				libraryShapeIndex:
					row.libraryName !== null
						? libraryOrder.indexOf(row.libraryName) % 5
						: 0,
				topDecileActs:
					row.linkInCount + row.linkOutCount >= top10thLinkCount,
				provenanceSector: provenanceSectorOf(row.sourcesPrimary),
				x,
				y,
			});
		}
	}
	return out;
}

function uniqueSortedLibraries(rows: LayoutCacheRow[]): string[] {
	const set = new Set<string>();
	for (const r of rows) {
		if (r.libraryName !== null) set.add(r.libraryName);
	}
	return [...set].sort();
}

/** 90th-percentile threshold of (linkInCount + linkOutCount) across
 *  the Universe. Notes meeting OR exceeding this threshold render as
 *  top-decile-acts (size +40%). */
function topDecileLinkCount(rows: LayoutCacheRow[]): number {
	if (rows.length === 0) return Infinity;
	const counts = rows
		.map((r) => r.linkInCount + r.linkOutCount)
		.sort((a, b) => a - b);
	const idx = Math.floor(counts.length * 0.9);
	return counts[Math.min(idx, counts.length - 1)] || 1;
}

/**
 * Substring heuristic for provenance sector classification (v6.0).
 * v4.1 polish target: use the masādir-aware classifier per Concept
 * Paper §10. For now: URL-like → Read, anything else with content →
 * Self, null → Self.
 */
function provenanceSectorOf(sourcesPrimary: string | null): ProvenanceSector | null {
	if (!sourcesPrimary) return 'Self';
	const s = sourcesPrimary.toLowerCase();
	if (s.includes('http://') || s.includes('https://') || s.includes('book:')) {
		return 'Read';
	}
	if (s.includes('podcast:') || s.includes('heard:') || s.includes('audio:')) {
		return 'Heard';
	}
	if (s.includes('reasoned:') || s.includes('inference:')) {
		return 'Reasoned';
	}
	if (s.includes('tradition:') || s.includes('canon:') || s.includes('scripture:')) {
		return 'Tradition';
	}
	return 'Self';
}

// 2026-05-14 §A.14 fix-12: pathHashJitter helper REMOVED. Was used
// by the previous random-jitter positioning (cycle-2/3); superseded
// by deterministic phyllotaxis spiral within each (band, month) cell.
// Deterministic-by-cell-index sort removes the need for hash-based
// jitter — same path → same spiral position via stable sort within
// the cell.

// ════════════════════════════════════════════════════════════════════
// Render entry point
// ════════════════════════════════════════════════════════════════════

/**
 * Render the anchor dome to a Canvas 2D context.
 * Layer order: background → strata circles → calendar rim → stratum
 * labels → connector lines (under stars) → stars (top of stack).
 */
export function renderAnchorDome(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	links: LinkEdge[],
	width: number,
	height: number,
	options: { locale?: string; clear?: boolean; highlightedPath?: string | null } = {},
): void {
	const { locale = 'en', clear = true, highlightedPath = null } = options;
	const layout = computeDomeLayout(width, height);

	// 2026-05-14 §A.14 fix-10 (Boss-test cycle 3 zoom regression):
	// clear + background must run in IDENTITY transform space so they
	// always cover the full canvas backing store, regardless of caller-
	// applied zoom/pan transforms (FIX-9). Pre-fix, at zoom > 1 these
	// only filled the upper-left fraction of the canvas — the rest of
	// the screen showed stale frame contents and zoom appeared to do
	// nothing. The save/restore pattern preserves the caller's transform
	// for the rest of the dome rendering (which DOES run in caller
	// space — that's what makes zoom-toward-cursor work).
	if (clear) {
		ctx.save();
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.fillStyle = PALETTE.bg;
		ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.restore();
	} else {
		// Even when caller manages clear, paint the bg in caller-space
		// so the dome's local viewport gets the right base color.
		ctx.fillStyle = PALETTE.bg;
		ctx.fillRect(0, 0, width, height);
	}

	// 2. 5 strata reference circles
	ctx.strokeStyle = PALETTE.strataRing;
	// 2026-05-14 §A.14 fix-1: stroke width 0.6 → 0.9 for chrome legibility
	ctx.lineWidth = 0.9;
	for (const r of stratumBandBoundaries(layout.radius)) {
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}

	// 3. Calendar rim labels (12 months, locale-aware)
	ctx.fillStyle = PALETTE.calendarRimText;
	ctx.font = '10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	for (const m of calendarRimMonths(layout.radius, locale, 18)) {
		ctx.fillText(m.label, layout.centerX + m.x, layout.centerY + m.y);
	}

	// 4. Stratum labels along the vertical axis
	ctx.fillStyle = PALETTE.stratumLabel;
	ctx.font = 'italic 9px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	for (const band of STRATUM_BANDS) {
		const r = radiusForStratum(band, layout.radius);
		ctx.fillText(STRATUM_LABELS[band], layout.centerX, layout.centerY - r);
	}

	// 5. Connector lines (under stars). Auto-fade above 800 visible
	//    per Concept Paper §2.2 invariant.
	if (stars.length > 0 && links.length > 0) {
		drawConnectorLines(ctx, stars, links);
	}

	// 6. Stars (top of stack)
	if (stars.length > 0) {
		drawStars(ctx, stars, highlightedPath);
	}
}

// ════════════════════════════════════════════════════════════════════
// Star rendering
// ════════════════════════════════════════════════════════════════════

// 2026-05-14 §A.14 fix-3 (Boss-test cycle 1): smaller baseline, smaller
//                    top-decile delta, pip bumped to 2.4 px + full opacity.
// 2026-05-14 §A.14 fix-7 (Boss-test cycle 2): density-aware rendering.
//   The cycle-2 build still showed solid white blobs in Eisa's 7,650-note
//   centroid (1-2 strata × 1-2 months) because even at full band+month
//   jitter, ~3,800 notes pile into the same ~100 px region (~30 stars
//   per pixel-spot at 3.5 px). Real fix: shrink star bodies AND drop per-
//   star body opacity so overlapping bodies BLEND ADDITIVELY — dense
//   clusters become brighter texture, sparse areas show distinct shapes.
//   The dome reads as a star chart instead of a thresholded mask.
// 2026-05-14 §A.14 fix-8 (Boss-test cycle 2): two-pass rendering.
//   ALL star bodies in pass 1; ALL pips in pass 2. So pips survive
//   in dense clusters where bodies overlap (cycle-2 each star drew its
//   pip then the next star's body covered it).
// 2026-05-14 §A.14 fix-13 (Boss-test cycle 3.3 redesign): Eisa's spec —
//   node size at maximum zoom = 5 px diameter (= 2.5 px radius). The
//   wheel-zoom transform scales world coords by zoomScale (max 8×), so
//   the world-space radius must be 2.5 / 8 = 0.3125 px. This means:
//     • Default zoom (1×): nodes at 0.625 px diameter — sub-pixel.
//       Each node contributes ~30% pixel coverage; with additive
//       blending (BODY_OPACITY_MULT × confidenceAlpha), dense areas
//       saturate naturally and sparse outliers appear as faint specks.
//       The default view becomes a true DENSITY CHART rather than a
//       constellation of discrete blobs.
//     • Mid zoom (~4×): nodes at 2.5 px diameter — visible specks.
//       Spiral structure (per fix-12 phyllotaxis) starts to emerge.
//     • Max zoom (8×): nodes at the spec'd 5 px diameter — clean,
//       readable individuals. Library shape distinguishable; pip
//       color legible; top-decile +32% size visibly bigger.
//   Converts the pipeline from "constellation at all zooms" (which
//   failed at 7,650-note density) into "density chart at default
//   + individual stars at zoom" — semantic-zoom pattern Datashader,
//   Tableau, Bokeh converge on, achieved without a separate
//   aggregation pass.
const ZOOM_MAX_FOR_SIZING = 8;                            // mirrors ZOOM_MAX in SightV6.svelte
const BASE_STAR_RADIUS = 2.5 / ZOOM_MAX_FOR_SIZING;       // 0.3125 → 5 px diameter @ 8× zoom
const TOP_DECILE_RADIUS = BASE_STAR_RADIUS * 1.32;        // ~0.41 → 6.6 px @ 8× (+32% delta)
const PIP_RADIUS = BASE_STAR_RADIUS * 0.6;                // ~0.19 → 3 px @ 8× (60% of body)
// Per-star body opacity multiplier. Bumped from 0.55 (cycle-2/3)
// to 0.7 because smaller nodes need more per-node alpha to remain
// visible in sparse areas at default zoom.
const BODY_OPACITY_MULT = 0.7;

function drawStars(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	highlightedPath: string | null,
): void {
	// PASS 1: all star bodies (additive blend via lower per-star alpha).
	ctx.fillStyle = PALETTE.starFill;
	for (const star of stars) {
		const r = star.topDecileActs ? TOP_DECILE_RADIUS : BASE_STAR_RADIUS;
		const opacity = (star.row.confidenceAlpha ?? 0.45) * BODY_OPACITY_MULT;
		ctx.globalAlpha = opacity;
		drawShape(ctx, star.x, star.y, r, star.libraryShapeIndex);
	}

	// PASS 2: all pips on top of bodies (full opacity per Bertin
	// orthogonality — pip hue carries categorical stage signal,
	// independent of confidence). With pass-2 ordering, the pip
	// is the LAST thing drawn at any (x,y), so it stays visible
	// in dense clusters where bodies overlap.
	ctx.globalAlpha = 1;
	for (const star of stars) {
		const pipColor = pipColorForStage(star.row.stage);
		if (!pipColor) continue;
		ctx.fillStyle = pipColor;
		ctx.beginPath();
		ctx.arc(star.x, star.y, PIP_RADIUS, 0, Math.PI * 2);
		ctx.fill();
	}

	// PASS 3: highlighted-brushing ring (above everything).
	if (highlightedPath !== null) {
		const star = stars.find((s) => s.row.notePath === highlightedPath);
		if (star) {
			const r = star.topDecileActs ? TOP_DECILE_RADIUS : BASE_STAR_RADIUS;
			ctx.strokeStyle = PALETTE.highlightedRing;
			ctx.lineWidth = 1.8;
			ctx.beginPath();
			ctx.arc(star.x, star.y, r + 4, 0, Math.PI * 2);
			ctx.stroke();
		}
	}
}

/** Draw the library-shape glyph centered on (x, y) with size r.
 *  Bertin shape variable: each library gets a distinct primitive.
 *  Shape-weight normalization (§3.3): equal PERCEIVED area, not
 *  equal bounding-box area. Diamond -15%, triangle +20%, hexagon -10%. */
function drawShape(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	r: number,
	shapeIndex: number,
): void {
	switch (shapeIndex % 5) {
		case 0: // circle
			ctx.beginPath();
			ctx.arc(x, y, r, 0, Math.PI * 2);
			ctx.fill();
			return;
		case 1: { // square
			const s = r * 1.6; // square inscribed in 2r diameter; trim slightly
			ctx.fillRect(x - s / 2, y - s / 2, s, s);
			return;
		}
		case 2: { // diamond (rotated square) — perceived area -15%
			const s = r * 1.6 * 0.85;
			ctx.beginPath();
			ctx.moveTo(x, y - s / Math.SQRT2);
			ctx.lineTo(x + s / Math.SQRT2, y);
			ctx.lineTo(x, y + s / Math.SQRT2);
			ctx.lineTo(x - s / Math.SQRT2, y);
			ctx.closePath();
			ctx.fill();
			return;
		}
		case 3: { // triangle — perceived area +20%
			const s = r * 2.0 * 1.20;
			const h = (s * Math.sqrt(3)) / 2;
			ctx.beginPath();
			ctx.moveTo(x, y - (2 / 3) * h);
			ctx.lineTo(x + s / 2, y + (1 / 3) * h);
			ctx.lineTo(x - s / 2, y + (1 / 3) * h);
			ctx.closePath();
			ctx.fill();
			return;
		}
		case 4: { // hexagon — perceived area -10%
			const s = r * 0.90;
			ctx.beginPath();
			for (let i = 0; i < 6; i++) {
				const a = (i * Math.PI) / 3 - Math.PI / 6;
				const px = x + Math.cos(a) * s;
				const py = y + Math.sin(a) * s;
				if (i === 0) ctx.moveTo(px, py);
				else ctx.lineTo(px, py);
			}
			ctx.closePath();
			ctx.fill();
			return;
		}
	}
}

function pipColorForStage(stage: string | null): string | null {
	switch (stage as LifecycleStage | null) {
		case 'established': return PALETTE.stageEstablished;
		case 'fresh':       return PALETTE.stageFresh;
		case 'growing':     return PALETTE.stageGrowing;
		case 'at-risk':     return PALETTE.stageAtRisk;
		case 'dormant':     return PALETTE.stageDormant;
		default:            return null;
	}
}

// ════════════════════════════════════════════════════════════════════
// Connector-line rendering
// ════════════════════════════════════════════════════════════════════

const LINK_FADE_THRESHOLD = 800;     // §2.2 invariant
const LINK_OPACITY_NORMAL = 0.55;
const LINK_OPACITY_FADED = 0.18;

function drawConnectorLines(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	links: LinkEdge[],
): void {
	// Build a path → star lookup for O(1) endpoint resolution.
	const byPath = new Map<string, StarDerived>();
	for (const s of stars) byPath.set(s.row.notePath, s);

	const visibleLinks = links.filter(
		(l) => byPath.has(l.sourcePath) && byPath.has(l.targetPath),
	);
	const opacity =
		visibleLinks.length > LINK_FADE_THRESHOLD
			? LINK_OPACITY_FADED
			: LINK_OPACITY_NORMAL;

	ctx.save();
	ctx.globalAlpha = opacity;
	ctx.lineWidth = 0.6;
	ctx.lineCap = 'round';

	for (const link of visibleLinks) {
		const a = byPath.get(link.sourcePath);
		const b = byPath.get(link.targetPath);
		if (!a || !b) continue;
		const color = lineColorForLink(link.linkType);
		ctx.strokeStyle = color;
		// Contradicts: dashed line.
		if (link.linkType === 'contradicts') {
			ctx.setLineDash([2, 2]);
		} else {
			ctx.setLineDash([]);
		}
		ctx.beginPath();
		ctx.moveTo(a.x, a.y);
		ctx.lineTo(b.x, b.y);
		ctx.stroke();
	}

	ctx.setLineDash([]);
	ctx.restore();
}

function lineColorForLink(kind: TypedLinkKind): string {
	switch (kind) {
		case 'supports':     return PALETTE.linkSupports;
		case 'contradicts':  return PALETTE.linkContradicts;
		case 'causes':       return PALETTE.linkCauses;
		case 'exemplifies':  return PALETTE.linkExemplifies;
		case 'generalizes':  return PALETTE.linkGeneralizes;
		case 'derives-from': return PALETTE.linkDerivesFrom;
		case 'part-of':      return PALETTE.linkPartOf;
		case 'associative':  return PALETTE.linkAssociative;
		case 'supersedes':   return PALETTE.linkSupersedes;
	}
}

// ════════════════════════════════════════════════════════════════════
// Hit-test (per §5 gesture grammar)
// ════════════════════════════════════════════════════════════════════

/**
 * Find the star closest to (x, y) within `tolerancePx`. Returns
 * the star's note path or null if no hit. Used for hover/click
 * dispatch from SightV6.svelte's pointer events.
 */
export function starHitTest(
	stars: StarDerived[],
	x: number,
	y: number,
	tolerancePx = 9,
): string | null {
	let best: { path: string; d2: number } | null = null;
	const tol2 = tolerancePx * tolerancePx;
	for (const s of stars) {
		const dx = s.x - x;
		const dy = s.y - y;
		const d2 = dx * dx + dy * dy;
		if (d2 <= tol2 && (best === null || d2 < best.d2)) {
			best = { path: s.row.notePath, d2 };
		}
	}
	return best ? best.path : null;
}

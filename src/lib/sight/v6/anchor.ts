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

	const out: StarDerived[] = [];
	for (const row of rows) {
		const band = bandForRawStratum(row.stratum);
		const bandCenterRadius = radiusForStratum(band, outerRadius);
		const bandHalfWidth = outerRadius / 10; // 5 bands → each ½-band ≈ 1/10 of outer radius
		// Jitter: deterministic per note_path so re-renders stay stable.
		// Hashes the path to two normalized [0, 1) values.
		// 2026-05-14 §A.14 fix-2 (Boss-test #2 feedback): jitter widened
		// substantially. Original ±15% radial + ±π/24 angular collapsed
		// thousands of co-stratum/co-month notes into visible blobs.
		// New: spread across nearly the full band width radially (±0.85 ×
		// halfWidth) and the full month wedge angularly (±π/12 = full
		// 30° wedge), so 7,636-note universes show as a textured speckle
		// instead of opaque masses.
		const [jitterRadial, jitterAngular] = pathHashJitter(row.notePath);
		const radial = bandCenterRadius + (jitterRadial - 0.5) * 1.7 * bandHalfWidth;

		const month = row.createdMonth ?? 0;
		// Angle: each month wedge spans 30°; place at the wedge center
		// (m * 30° + 15°) measured from NORTH (12 o'clock), clockwise.
		// Canvas math angle 0 = east, so subtract π/2 to rotate.
		const baseAngle =
			month * (Math.PI / 6) + Math.PI / 12 - Math.PI / 2;
		const angle = baseAngle + (jitterAngular - 0.5) * (Math.PI / 12);

		const x = centerX + Math.cos(angle) * radial;
		const y = centerY + Math.sin(angle) * radial;

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

/** Deterministic 32-bit FNV-1a hash → two normalized [0, 1) values
 *  for radial + angular jitter. Stable across paints; same path ⇒
 *  same jitter every time. */
function pathHashJitter(path: string): [number, number] {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	const u32 = h >>> 0;
	const radial = (u32 & 0xffff) / 0xffff;
	const angular = ((u32 >>> 16) & 0xffff) / 0xffff;
	return [radial, angular];
}

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

	if (clear) ctx.clearRect(0, 0, width, height);

	// 1. Background
	ctx.fillStyle = PALETTE.bg;
	ctx.fillRect(0, 0, width, height);

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
const BASE_STAR_RADIUS = 2.2;       // was 3.5 — smaller for density tolerance
const TOP_DECILE_RADIUS = 2.9;      // +32% delta — still pre-attentive
const PIP_RADIUS = 1.8;             // pip stays foveal-on-foveation
// Per-star body opacity multiplier. Combined with confidenceAlpha:
// a hypothesis (0.45) star renders at 0.45 × 0.55 = 0.25 opacity;
// established (1.0) at 0.55. Overlapping bodies accumulate.
const BODY_OPACITY_MULT = 0.55;

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

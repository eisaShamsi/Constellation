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
 * Star fill is NEUTRAL (_chrome.starFill = #cdd5e0) per §4 commit;
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
	TraditionModule,
	SectorSpec,
	RingSpec,
	LadderSpec,
	RelationalSpec,
	CyclicFlowSpec,
	BinaryFlowSpec,
	GradientSpec,
	HorizontalBandsSpec,
} from './types';
import {
	PALETTE,
	CHROME_PALETTE_DARK_FALLBACK,
	STRATUM_BANDS,
	STRATUM_LABELS,
	bandForRawStratum,
	calendarRimMonths,
	radiusForStratum,
	stratumBandBoundaries,
	type ChromePalette,
} from './dome';

// MIG-027 — Module-level chrome state. Set at the top of every
// renderAnchorDome call from the caller's chromePalette option (or
// the dark fallback). All helper functions in this file (drawStars,
// drawConnectorLines, drawSectorDividers, etc.) read chrome colors
// via this module-level reference so they pick up theme-aware values
// without per-call parameter plumbing.
//
// Safe because Sight renders one dome at a time; renderAnchorDome is
// the single entry point; helpers are called only within its call
// stack so the per-render assignment is stable for the duration of
// the paint.
let _chrome: ChromePalette = CHROME_PALETTE_DARK_FALLBACK;

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
	// §C.2 — active scholarly tradition. When provided, the default
	// Aristotelian position (computed below) is passed through
	// `tradition.remapStarPosition(row, defaultPos, layout)` before
	// being stored. For Aristotelian this is identity (no change);
	// for pramāṇa (§C.3) / masādir (§C.4) / Polanyi (Phase γ) etc.,
	// the remap moves the star into the tradition's geometric
	// vocabulary (quadrants / sectors / fog gradient). Optional so
	// existing callers and tests that don't yet pass a tradition
	// continue to render in default Aristotelian. Per Concept Paper
	// §11 invariant 6: tradition remap affects the anchor ONLY —
	// mini-domes never see this parameter.
	// MIG-026 Phase 0 — K1 rename: "tradition" → "tradition" throughout.
	tradition?: TraditionModule | null,
): StarDerived[] {
	if (rows.length === 0) return [];

	// Pre-compute Universe-wide context.
	const libraryOrder = uniqueSortedLibraries(rows);
	const top10thLinkCount = topDecileLinkCount(rows);

	// §C.2 — pre-build the layout payload passed to the tradition's
	// remap callback. Same shape as DomeLayout but plain object so
	// traditions don't need to import anchor.ts.
	const traditionLayout = { centerX, centerY, radius: outerRadius };

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

		const defaultX = centerX + Math.cos(angle) * radial;
		const defaultY = centerY + Math.sin(angle) * radial;

		// §C.2 — apply active tradition's remap. For Aristotelian this
		// returns the input unchanged (identity remap). For other
		// traditions (§C.3+), this redistributes the star into the
		// tradition's geometric vocabulary. When `tradition` is null/
		// undefined (e.g., chip-selected tradition has no module shipped
		// yet, or test caller didn't pass one), we fall back to the
		// default Aristotelian position.
		const remapped = tradition
			? tradition.remapStarPosition(row, { x: defaultX, y: defaultY }, traditionLayout)
			: { x: defaultX, y: defaultY };

		out.push({
			row,
			libraryShapeIndex:
				row.libraryName !== null
					? libraryOrder.indexOf(row.libraryName) % 5
					: 0,
			topDecileActs:
				row.linkInCount + row.linkOutCount >= top10thLinkCount,
			provenanceSector: provenanceSectorOf(row.sourcesPrimary),
			x: remapped.x,
			y: remapped.y,
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

/** §C.3 — Draw tradition-supplied sector dividers + wedge labels onto
 *  the anchor. Called once per paint with the active tradition's
 *  computed SectorSpec[]. For each sector, draws a stroke from the
 *  dome center to `angleStart` at the outer rim, then places `label`
 *  (if present) at the wedge midpoint at 88% of outer radius.
 *
 *  The angle convention is canvas math: 0 = east, increases clockwise
 *  (canvas y inverted). Per-tradition modules in `traditions/` produce
 *  SectorSpec[] in this convention. */
function drawSectorDividers(
	ctx: CanvasRenderingContext2D,
	layout: DomeLayout,
	sectors: SectorSpec[],
): void {
	if (sectors.length === 0) return;
	ctx.save();
	// Divider strokes — same color as strata rings (chrome family) but
	// slightly heavier so they read as "category boundary" not "axis tick".
	ctx.strokeStyle = _chrome.strataRing;
	ctx.lineWidth = 1.2;
	for (const sec of sectors) {
		const x2 = layout.centerX + Math.cos(sec.angleStart) * layout.radius;
		const y2 = layout.centerY + Math.sin(sec.angleStart) * layout.radius;
		ctx.beginPath();
		ctx.moveTo(layout.centerX, layout.centerY);
		ctx.lineTo(x2, y2);
		ctx.stroke();
	}
	// Wedge-center labels — same italic Inter as stratum labels but
	// slightly larger (10 px vs 9 px) so they read at one glance.
	ctx.fillStyle = _chrome.stratumLabel;
	ctx.font = 'italic 10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	const labelRadius = layout.radius * 0.88;
	for (const sec of sectors) {
		if (!sec.label) continue;
		const midAngle = (sec.angleStart + sec.angleEnd) / 2;
		const lx = layout.centerX + Math.cos(midAngle) * labelRadius;
		const ly = layout.centerY + Math.sin(midAngle) * labelRadius;
		ctx.fillText(sec.label, lx, ly);
	}
	ctx.restore();
}

// ════════════════════════════════════════════════════════════════════
// MIG-026 Phase α — Multi-shape tradition renderer stubs
// ════════════════════════════════════════════════════════════════════
//
// Seven stub renderers for the new tradition shapes introduced in
// MIG-026 (architecture foundation lands stubs only; subsequent
// phases fill in the actual drawing per the Plan):
//
//   drawRingBoundaries    — Phase ε.1 (Ibn Rushd burhān) implements
//   drawLadderSteps       — Phase ζ.2 (Maimonidean spiral) implements
//   drawRelationalGraph   — Phase θ.1 (Mignolo hub-and-spoke) implements
//   drawCyclicFlow        — Phase δ.2 (Dewey inquiry) implements
//   drawBinaryFlow        — Phase ε.3 (Ibn Khaldūn ʿumrān) implements
//   drawGradientFog       — Phase γ (Polanyi) implements
//   drawHorizontalBands   — Phase γ (Mohist sān biǎo) implements
//
// Each stub:
//   - Accepts the proper signature so the dispatch in step 2.5
//     type-checks against TraditionModule's spec callbacks
//   - Is a no-op (early return) so dispatching to a stub during
//     Phase α has zero visual effect — Aristotelian/pramāṇa/masādir
//     continue to render via the existing drawSectorDividers
//   - Logs `console.warn` on first call (mid-phase debugging aid;
//     removed in the phase that implements the shape) so we know
//     when a tradition we haven't built yet gets selected

/** MIG-026 Phase δ.2 — Concentric ring boundary strokes + labels.
 *
 *  Originally scheduled for Phase ε.1 (Ibn Rushd burhān ladder) but
 *  pulled forward to δ.2 because Husserl's regional ontologies is the
 *  first ring-shaped tradition to ship (geometric-shape audit in
 *  orientation v2.10 puts Husserl in the "Concentric rings" category
 *  alongside Ibn Rushd, PaRDeS, and Maldonado-Torres).
 *
 *  For each RingSpec, draws an arc at `radiusFrac * layout.radius` and
 *  places `label` (if present) at the midpoint of the annulus along
 *  the -y axis (top vertical) shifted slightly so it doesn't collide
 *  with the existing stratum labels (which sit at the centers of the
 *  5 strata bands at centerY - r). We offset the ring labels along the
 *  +x axis (east, 3 o'clock) instead — clean and consistent across
 *  ring-shape traditions.
 *
 *  Stroke color = _chrome.strataRing (chrome family — same as the
 *  base strata circles in step 2). Line width 1.2 px world-units
 *  (matches drawSectorDividers convention).
 */
function drawRingBoundaries(
	ctx: CanvasRenderingContext2D,
	layout: DomeLayout,
	rings: RingSpec[],
): void {
	if (rings.length === 0) return;
	ctx.save();

	// 1. Ring boundary arcs at the requested radial fractions.
	ctx.strokeStyle = _chrome.strataRing;
	ctx.lineWidth = 1.2;
	for (const ring of rings) {
		const r = ring.radiusFrac * layout.radius;
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}

	// 2. Annulus labels. Each ring's label sits at the midpoint between
	//    its boundary and the next-larger boundary (or the outer rim if
	//    this is the outermost ring), along the +x axis (east, 3 o'clock)
	//    so it doesn't collide with the stratum labels on the +y axis.
	//    Sort rings by radiusFrac to ensure midpoint math uses adjacent
	//    boundaries.
	const sortedRings = [...rings].sort((a, b) => a.radiusFrac - b.radiusFrac);
	ctx.fillStyle = _chrome.stratumLabel;
	ctx.font = 'italic 10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'left';
	ctx.textBaseline = 'middle';
	for (let i = 0; i < sortedRings.length; i++) {
		const ring = sortedRings[i];
		if (!ring.label) continue;
		const innerFrac = i === 0 ? 0 : sortedRings[i - 1].radiusFrac;
		const outerFrac = ring.radiusFrac;
		const midFrac = (innerFrac + outerFrac) / 2;
		const lx = layout.centerX + midFrac * layout.radius + 4; // +4 px world inset right
		const ly = layout.centerY;
		ctx.fillText(ring.label, lx, ly);
	}

	// 3. If the largest ring boundary is < 1.0 (i.e. there's an outer
	//    annulus beyond the last specified boundary), label that annulus
	//    too — its inner edge is the largest specified boundary, outer
	//    is the dome rim (radiusFrac 1.0). Caller decides what label to
	//    use by including a RingSpec with radiusFrac=1.0 and the desired
	//    label; this block intentionally does NOT auto-generate a label.

	ctx.restore();
}

/** STUB — Phase ζ.2 (Maimonidean prophecy spiral) ships the implementation.
 *  Per Eisa's locked D3 choice: render as logarithmic spiral from center
 *  to outer rim, with N step-marks along the spiral. */
function drawLadderSteps(
	_ctx: CanvasRenderingContext2D,
	_layout: DomeLayout,
	_ladder: LadderSpec,
): void {
	// TODO Phase ζ.2 — Maimonidean spiral spike: implement equiangular
	// spiral r(θ) = a·exp(b·θ) where:
	//   N = ladder.steps.length
	//   a = small inner offset (~5% of layout.radius)
	//   b chosen so step N lands at r ≈ 0.95 * layout.radius
	//   trace spiral path with ctx.beginPath / lineTo
	//   place step-marks at N equally-spaced θ values along the spiral
	//   label each mark tangent to the spiral or via short radial spokes
}

/** STUB — Phase θ.1 (Mignolo pluriversal hub-and-spoke) ships the impl.
 *  Per Eisa's locked E3 choice: render as central hub disc + N outer
 *  clusters with connecting lines. */
function drawRelationalGraph(
	_ctx: CanvasRenderingContext2D,
	_layout: DomeLayout,
	_relational: RelationalSpec,
): void {
	// TODO Phase θ.1 — hub-and-spoke spike: implement
	//   hub: small disc at layout center, label = relational.hubLabel
	//   clusters: N positions around the rim at angles 2π·i/N, each with
	//     a small cluster-bubble and a connecting line back to hub
	//   labels: outside each cluster bubble (radial-outward placement)
}

/** MIG-026 Phase δ.2 — Cyclic-flow ring + segment labels + flow arrows.
 *
 *  Draws a single ring path at ~75% radius divided into N equal arc-
 *  segments. Each segment gets a label at its midpoint (radial-outward
 *  position so labels sit between the ring path and the outer rim).
 *  Chevron arrows on the path indicate clockwise sequence flow
 *  (segment N → wraps to segment 1, matching the cyclic nature of
 *  Dewey's pattern of inquiry).
 *
 *  Stroke colors from _chrome (chrome family) so the cyclic ring +
 *  arrows track the active theme. Segment dividers (short radial
 *  ticks crossing the ring path) help separate adjacent segments
 *  visually so the sequence reads as N distinct stages, not one
 *  continuous loop.
 */
function drawCyclicFlow(
	ctx: CanvasRenderingContext2D,
	layout: DomeLayout,
	cyclic: CyclicFlowSpec,
): void {
	const n = cyclic.segments.length;
	if (n === 0) return;

	ctx.save();
	const ringRadius = layout.radius * 0.75;
	const segmentArc = (2 * Math.PI) / n;
	// Start at -π/2 (12 o'clock) so the first segment's leading edge
	// is at top. Sequence then proceeds clockwise.
	const startAngle = -Math.PI / 2;

	// 1. Main ring path at 75% radius.
	ctx.strokeStyle = _chrome.strataRing;
	ctx.lineWidth = 1.2;
	ctx.beginPath();
	ctx.arc(layout.centerX, layout.centerY, ringRadius, 0, Math.PI * 2);
	ctx.stroke();

	// 2. Segment divider ticks (short radial strokes at each segment
	//    boundary, crossing the ring path). 4 px world (~half on each
	//    side of the ring path).
	const tickHalfLen = 4;
	for (let i = 0; i < n; i++) {
		const a = startAngle + i * segmentArc;
		const cosA = Math.cos(a);
		const sinA = Math.sin(a);
		const x1 = layout.centerX + (ringRadius - tickHalfLen) * cosA;
		const y1 = layout.centerY + (ringRadius - tickHalfLen) * sinA;
		const x2 = layout.centerX + (ringRadius + tickHalfLen) * cosA;
		const y2 = layout.centerY + (ringRadius + tickHalfLen) * sinA;
		ctx.beginPath();
		ctx.moveTo(x1, y1);
		ctx.lineTo(x2, y2);
		ctx.stroke();
	}

	// 3. Segment labels at midpoint of each arc, placed slightly outside
	//    the ring path (at ~85% radius) so they don't overlap the ring
	//    stroke itself.
	ctx.fillStyle = _chrome.stratumLabel;
	ctx.font = 'italic 10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	const labelRadius = layout.radius * 0.85;
	for (let i = 0; i < n; i++) {
		const seg = cyclic.segments[i];
		if (!seg.label) continue;
		const midAngle = startAngle + (i + 0.5) * segmentArc;
		const lx = layout.centerX + labelRadius * Math.cos(midAngle);
		const ly = layout.centerY + labelRadius * Math.sin(midAngle);
		ctx.fillText(seg.label, lx, ly);
	}

	// 4. Flow chevron arrows. One small arrow at each segment midpoint
	//    on the ring path, pointing clockwise (tangent to the ring at
	//    that angle). The chevron is rendered as two short strokes
	//    forming a ">" shape pointing in the clockwise tangent direction.
	//    §δ.2-fix-1 (Eisa 2026-05-17): chevronSize bumped 4 → 8 (2× the
	//    initial Phase δ.2 value) per Boss feedback "Enlarge it 2x" —
	//    the smaller arrows didn't read as direction markers at 1× zoom.
	const chevronSize = 8;
	for (let i = 0; i < n; i++) {
		const midAngle = startAngle + (i + 0.5) * segmentArc;
		// Tangent direction at midAngle, clockwise = perpendicular to
		// the radial direction, +π/2 added to the radial angle.
		const tangentAngle = midAngle + Math.PI / 2;
		const cx = layout.centerX + ringRadius * Math.cos(midAngle);
		const cy = layout.centerY + ringRadius * Math.sin(midAngle);
		const tx = Math.cos(tangentAngle);
		const ty = Math.sin(tangentAngle);
		// Chevron tip at (cx, cy); two wings extending backward.
		const wingBackAngle1 = tangentAngle + Math.PI - Math.PI / 4;
		const wingBackAngle2 = tangentAngle + Math.PI + Math.PI / 4;
		const wx1 = cx + chevronSize * Math.cos(wingBackAngle1);
		const wy1 = cy + chevronSize * Math.sin(wingBackAngle1);
		const wx2 = cx + chevronSize * Math.cos(wingBackAngle2);
		const wy2 = cy + chevronSize * Math.sin(wingBackAngle2);
		// Push the chevron slightly forward of the segment midpoint so
		// the tip points into the next segment (suggests "flow into next").
		const tipPushX = cx + tx * 2;
		const tipPushY = cy + ty * 2;
		const wx1Adj = wx1 + tx * 2;
		const wy1Adj = wy1 + ty * 2;
		const wx2Adj = wx2 + tx * 2;
		const wy2Adj = wy2 + ty * 2;
		ctx.beginPath();
		ctx.moveTo(wx1Adj, wy1Adj);
		ctx.lineTo(tipPushX, tipPushY);
		ctx.lineTo(wx2Adj, wy2Adj);
		ctx.stroke();
	}

	ctx.restore();
}

/** STUB — Phase ε.3 (Ibn Khaldūn ʿumrān cyclic binary) ships the impl.
 *  Renders 2 cells (typically inner disc + outer ring, or top/bottom
 *  bands) with directional flow arrows. */
function drawBinaryFlow(
	_ctx: CanvasRenderingContext2D,
	_layout: DomeLayout,
	_binary: BinaryFlowSpec,
): void {
	// TODO Phase ε.3 — Ibn Khaldūn (cyclic), Phase θ.2 — Dussel (a→b),
	// Phase η.2 — Wang Yangming (bidirectional with center):
	//   cell A + cell B layout depends on tradition (horizontal split
	//   for ʿumrān; concentric for Dussel; left/right hemisphere for
	//   Wang Yangming)
	//   flow arrow per binary.flowDirection
	//   optional center label (Wang Yangming's liángzhī)
}

/** MIG-026 Phase γ — Polanyi tacit/explicit fog overlay.
 *
 *  Paints a radial alpha-modulated fog (bg-colored translucent overlay)
 *  across the dome circle. The fog is dense at center (low star
 *  visibility = the tacit pole — "acknowledged but inarticulable") and
 *  sparse at the edge (high star visibility = the explicit pole —
 *  "what you can articulate"). Per Polanyi (1966), tacit and explicit
 *  are a continuum, not a binary; the radial gradient encodes that
 *  continuum visually.
 *
 *  Dispatched AFTER stars (see anchor.ts step 6.5) so the fog actually
 *  overlays the star layer. Stratum labels and the calendar rim are
 *  drawn before stars (steps 4-5), so they also get fogged at center —
 *  conceptually consistent with the Polanyi metaphor (the inner
 *  stratum "FOUNDATION" is least articulable; the outer rim "EDGE OF
 *  KNOWING" is most articulable).
 *
 *  Fog color = _chrome.bg so it reads as "the background seeping into
 *  the stars" rather than as a colored overlay. Adapts to both dark
 *  and light themes since _chrome.bg is theme-aware (MIG-027).
 */
function drawGradientFog(
	ctx: CanvasRenderingContext2D,
	layout: DomeLayout,
	gradient: GradientSpec,
): void {
	// Fog alpha at each radial endpoint = 1 - (star visibility there).
	// Star visibility at center = gradient.centerOpacity (typically ~0.18).
	// Star visibility at edge   = gradient.edgeOpacity   (typically ~0.95).
	// So fog alpha at center    = ~0.82 (mostly opaque fog → stars hidden).
	// And fog alpha at edge     = ~0.05 (mostly clear → stars visible).
	const fogCenterAlpha = Math.max(0, Math.min(1, 1 - gradient.centerOpacity));
	const fogEdgeAlpha = Math.max(0, Math.min(1, 1 - gradient.edgeOpacity));

	const bg = parseRgb(_chrome.bg);

	ctx.save();
	const grad = ctx.createRadialGradient(
		layout.centerX, layout.centerY, 0,
		layout.centerX, layout.centerY, layout.radius,
	);
	grad.addColorStop(0, `rgba(${bg.r}, ${bg.g}, ${bg.b}, ${fogCenterAlpha})`);
	grad.addColorStop(1, `rgba(${bg.r}, ${bg.g}, ${bg.b}, ${fogEdgeAlpha})`);
	ctx.fillStyle = grad;
	ctx.beginPath();
	ctx.arc(layout.centerX, layout.centerY, layout.radius, 0, Math.PI * 2);
	ctx.fill();

	// Optional centerLabel / edgeLabel — placed at the radial endpoints
	// in the same italic font as stratum labels (chrome family). Center
	// label sits at the dome center; edge label sits just inside the
	// bottom rim (chosen over top rim because the +y direction is where
	// stratum labels do NOT live — top has FOUNDATION/WORKING/etc.).
	if (gradient.centerLabel || gradient.edgeLabel) {
		ctx.fillStyle = _chrome.stratumLabel;
		ctx.font = 'italic 10px Inter, system-ui, sans-serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		if (gradient.centerLabel) {
			ctx.fillText(gradient.centerLabel, layout.centerX, layout.centerY);
		}
		if (gradient.edgeLabel) {
			ctx.fillText(
				gradient.edgeLabel,
				layout.centerX,
				layout.centerY + layout.radius * 0.92,
			);
		}
	}
	ctx.restore();
}

/** MIG-026 Phase γ — Mohist sān biǎo 3 horizontal bands.
 *
 *  Divides the dome circle into N equal-height horizontal bands by
 *  drawing N−1 horizontal divider strokes across the dome chord. Each
 *  band's label is placed at the left edge just inside the dome rim
 *  (so it doesn't overlap with the calendar rim text outside the dome).
 *
 *  Divider strokes are clipped to the dome circle so they don't escape
 *  the rim. Stroke color matches strata rings (chrome family) per the
 *  drawSectorDividers convention — these are category boundaries, not
 *  text. Labels use the stratum-label chrome color and italic font for
 *  visual consistency with the rest of the dome chrome.
 *
 *  Per Mohist sān biǎo: 3 bands top-to-bottom are
 *    本 (běn, root)   — top
 *    原 (yuán, origin) — middle
 *    用 (yòng, use)    — bottom
 */
function drawHorizontalBands(
	ctx: CanvasRenderingContext2D,
	layout: DomeLayout,
	bands: HorizontalBandsSpec,
): void {
	const n = bands.bands.length;
	if (n === 0) return;

	ctx.save();

	// 1. Divider strokes — N−1 horizontal lines clipped to the dome.
	//    The dome spans y ∈ [centerY − radius, centerY + radius]. Each
	//    band has height 2r/N; dividers sit between bands at:
	//      y_i = centerY − r + i * (2r/N)   for i = 1 .. N−1
	//    At each divider y, the chord half-width is √(r² − (y − centerY)²).
	ctx.strokeStyle = _chrome.strataRing;
	ctx.lineWidth = 1.2;
	const bandHeight = (2 * layout.radius) / n;
	for (let i = 1; i < n; i++) {
		const y = layout.centerY - layout.radius + i * bandHeight;
		const dy = y - layout.centerY;
		const halfChord = Math.sqrt(
			Math.max(0, layout.radius * layout.radius - dy * dy),
		);
		ctx.beginPath();
		ctx.moveTo(layout.centerX - halfChord, y);
		ctx.lineTo(layout.centerX + halfChord, y);
		ctx.stroke();
	}

	// 2. Band labels — placed at the left edge of each band, ~92% of
	//    the chord half-width from the band's vertical center. Same
	//    italic font + chrome color as stratum labels for consistency.
	ctx.fillStyle = _chrome.stratumLabel;
	ctx.font = 'italic 10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'left';
	ctx.textBaseline = 'middle';
	for (let i = 0; i < n; i++) {
		const band = bands.bands[i];
		if (!band.label) continue;
		const yCenter = layout.centerY - layout.radius + (i + 0.5) * bandHeight;
		const dy = yCenter - layout.centerY;
		const halfChord = Math.sqrt(
			Math.max(0, layout.radius * layout.radius - dy * dy),
		);
		// Position label inside the band at ~92% of the left half-chord
		// so it sits clearly within the dome but doesn't crowd the rim.
		const lx = layout.centerX - halfChord * 0.92;
		ctx.fillText(band.label, lx, yCenter);
	}

	ctx.restore();
}

/** Parse a CSS color string (`#rgb`, `#rrggbb`, `rgb(r,g,b)`, or
 *  `rgba(r,g,b,a)`) into RGB components. Used by drawGradientFog to
 *  build alpha-varying gradient stops from the theme bg color (which
 *  arrives from getComputedStyle as either hex or `rgb(...)` depending
 *  on theme definition).
 *
 *  Falls back to the dark-fallback bg (#080c16) on parse failure —
 *  ensures the renderer always returns a valid color rather than NaN.
 */
function parseRgb(c: string): { r: number; g: number; b: number } {
	const trimmed = c.trim();
	// #rgb / #rrggbb
	if (trimmed.startsWith('#')) {
		const hex = trimmed.slice(1);
		if (hex.length === 3) {
			return {
				r: parseInt(hex[0] + hex[0], 16),
				g: parseInt(hex[1] + hex[1], 16),
				b: parseInt(hex[2] + hex[2], 16),
			};
		}
		if (hex.length === 6) {
			return {
				r: parseInt(hex.slice(0, 2), 16),
				g: parseInt(hex.slice(2, 4), 16),
				b: parseInt(hex.slice(4, 6), 16),
			};
		}
	}
	// rgb(r,g,b) / rgba(r,g,b,a)
	const m = trimmed.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/);
	if (m) {
		return { r: parseInt(m[1], 10), g: parseInt(m[2], 10), b: parseInt(m[3], 10) };
	}
	// Fallback: dark-theme bg
	return { r: 8, g: 12, b: 22 };
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
	options: {
		locale?: string;
		clear?: boolean;
		highlightedPath?: string | null;
		zoomScale?: number;
		matchedPaths?: Set<string> | null;
		densityMode?: boolean;
		// §C.2 — active epistemic tradition. When provided, the tradition's
		// optional `sectorDividers(layout)` callback is invoked and its
		// returned SectorSpec[] is drawn as quadrant/sector boundary
		// strokes + center-of-wedge labels. For Aristotelian and Polanyi
		// this is absent (no tradition-drawn dividers). For pramāṇa
		// (§C.3) and masādir (§C.4) this is 4 sectors. Star positions
		// themselves are remapped in computeStarPositions, not here.
		tradition?: TraditionModule | null;
		// MIG-027 — theme-aware chrome palette. Caller computes via
		// `readChromePalette(canvasHostEl)` (from dome.ts) so colors
		// track the active interface theme automatically. Falls back
		// to dark when absent (preserves pre-MIG-027 behavior for
		// any caller that hasn't migrated).
		chromePalette?: ChromePalette;
		/** MIG-026 §γ-fix-2 (2026-05-17) — per-paint star radius boost
		 *  in SCREEN pixels (zoom-invariant). Used by spread-shape
		 *  traditions (Mohist horizontal-bands; future grid/rings/
		 *  relational) where stars redistribute uniformly across the
		 *  dome and the default sub-pixel-at-1×-zoom size dissolves
		 *  into the bg. Boss-test cycle 2026-05-17 found Mohist stars
		 *  needed +2 px to read clearly. Aristotelian/pramāṇa/masādir/
		 *  Polanyi keep the default 0 boost (cluster-style layouts
		 *  benefit from sub-pixel sizes via additive blending). */
		starRadiusBoostScreenPx?: number;
	} = {},
): void {
	const {
		locale = 'en',
		clear = true,
		highlightedPath = null,
		zoomScale = 1,
		matchedPaths = null,
		densityMode: _densityMode = false,
		tradition = null,
		chromePalette = CHROME_PALETTE_DARK_FALLBACK,
		starRadiusBoostScreenPx = 0,
	} = options;
	// MIG-027 — set the module-level chrome state for the duration of
	// this paint. All helper functions in this file (drawStars,
	// drawConnectorLines, drawSectorDividers, etc.) read chrome colors
	// via the `_chrome` reference at the top of the file. Safe because
	// Sight renders one dome at a time + helpers are called only
	// within renderAnchorDome's call stack.
	_chrome = chromePalette;
	// §B.9 — densityMode accepted for API symmetry with renderMiniDome
	// but currently unused: the anchor already renders bodies at
	// BODY_OPACITY_MULT (0.7) for the existing density-via-additive-
	// blending baseline (see drawStars Pass 1 comment), so dense regions
	// already read as "more stars here" without an explicit mode switch.
	// Future polish (v6.2 hex-bin) could use this flag to swap to true
	// hex-bin rendering at very high counts.
	void _densityMode;
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
		ctx.fillStyle = _chrome.bg;
		ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.restore();
	} else {
		// Even when caller manages clear, paint the bg in caller-space
		// so the dome's local viewport gets the right base color.
		ctx.fillStyle = _chrome.bg;
		ctx.fillRect(0, 0, width, height);
	}

	// 2. 5 strata reference circles
	ctx.strokeStyle = _chrome.strataRing;
	// 2026-05-14 §A.14 fix-1: stroke width 0.6 → 0.9 for chrome legibility
	ctx.lineWidth = 0.9;
	for (const r of stratumBandBoundaries(layout.radius)) {
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}

	// 2.5 — Tradition-shape dispatch (CHROME shapes — under stars).
	//      Drawn AFTER strata circles so any shape overlays visibly cross
	//      the rings; drawn BEFORE the calendar rim and stratum labels so
	//      those text labels stay on top and remain legible.
	//
	//      §C.3/§C.4 (MIG-025): sectoral dispatch for pramāṇa + masādir.
	//      MIG-026 Phase α: extended with full multi-shape dispatch.
	//      MIG-026 Phase γ: gradient (Polanyi) moved to a second dispatch
	//      point at step 7 (AFTER stars) because gradient is an OVERLAY,
	//      not chrome — its purpose is to modulate star visibility, which
	//      requires painting after stars are drawn. All other shapes
	//      remain under stars as chrome.
	//
	//      Phase coverage:
	//        γ:    gradient (after stars) + horizontal-bands
	//        δ.2:  cyclic-flow
	//        ε.1:  rings
	//        ε.2:  grid
	//        ε.3 / θ.2 / η.2: binary-flow
	//        ζ.2 / ζ.3:       ladder
	//        θ.1 / θ.5:       relational
	if (tradition) {
		const traditionLayout = {
			centerX: layout.centerX,
			centerY: layout.centerY,
			radius: layout.radius,
		};
		if (tradition.sectorDividers) {
			drawSectorDividers(ctx, layout, tradition.sectorDividers(traditionLayout));
		}
		if (tradition.ringBoundaries) {
			drawRingBoundaries(ctx, layout, tradition.ringBoundaries(traditionLayout));
		}
		if (tradition.ladderSteps) {
			drawLadderSteps(ctx, layout, tradition.ladderSteps(traditionLayout));
		}
		if (tradition.relationalSpec) {
			drawRelationalGraph(ctx, layout, tradition.relationalSpec(traditionLayout));
		}
		if (tradition.cyclicFlowSpec) {
			drawCyclicFlow(ctx, layout, tradition.cyclicFlowSpec(traditionLayout));
		}
		if (tradition.binaryFlowSpec) {
			drawBinaryFlow(ctx, layout, tradition.binaryFlowSpec(traditionLayout));
		}
		if (tradition.horizontalBandsSpec) {
			drawHorizontalBands(ctx, layout, tradition.horizontalBandsSpec(traditionLayout));
		}
		// NOTE: gradient dispatch is at step 7 below (post-stars).
	}

	// 3. Calendar rim labels (12 months, locale-aware)
	ctx.fillStyle = _chrome.calendarRimText;
	ctx.font = '10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	for (const m of calendarRimMonths(layout.radius, locale, 18)) {
		ctx.fillText(m.label, layout.centerX + m.x, layout.centerY + m.y);
	}

	// 4. Stratum labels along the vertical axis
	ctx.fillStyle = _chrome.stratumLabel;
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

	// 6. Stars (top of stack — but see step 7 for overlay shapes)
	if (stars.length > 0) {
		drawStars(ctx, stars, highlightedPath, zoomScale, matchedPaths, starRadiusBoostScreenPx);
	}

	// 7. Tradition-shape dispatch (OVERLAY shapes — over stars).
	//    MIG-026 Phase γ — Polanyi's gradient fog. Painted AFTER stars
	//    so the fog actually modulates star visibility (a fog painted
	//    BEFORE stars at step 2.5 would be invisible — stars would just
	//    cover it). For Aristotelian/pramāṇa/masādir/Mohist this is a
	//    no-op because their modules don't define gradientSpec.
	if (tradition?.gradientSpec) {
		const traditionLayout = {
			centerX: layout.centerX,
			centerY: layout.centerY,
			radius: layout.radius,
		};
		drawGradientFog(ctx, layout, tradition.gradientSpec(traditionLayout));
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
	zoomScale: number = 1,
	matchedPaths: Set<string> | null = null,
	radiusBoostScreenPx: number = 0,
): void {
	// 2026-05-15 §B.7-fix-2 (Eisa cycle-2 ghost mode): when a facet
	// filter is active, render NON-matching stars at low opacity (ghost)
	// instead of hiding them entirely. Lets the user Shift+click a faded
	// star to ADD its category to the filter — multi-select within a
	// facet works directly from the dome instead of requiring sidebar
	// chip interaction. matchedPaths === null means no filter active
	// (all stars render at full encoding).
	const GHOST_ALPHA = 0.15;
	// MIG-026 §γ-fix-2 (Eisa Boss test 2026-05-17): per-paint star radius
	// boost in SCREEN pixels (zoom-invariant). Spread-shape traditions
	// (Mohist horizontal-bands; future grid/rings/relational) redistribute
	// stars uniformly across the dome instead of clustering them, which
	// destroys the additive-blending milky-way texture Aristotelian-style
	// concentration produces. The default star size (5 px ⌀ at 8× zoom →
	// ~0.6 px ⌀ at 1× zoom) was tuned for the cluster case; in spread
	// layouts individual sub-pixel dots dissolve into the bg even with
	// density mode off. The boost adds N CSS-pixels to the body+pip
	// radius (and the hover ring picks up the same boost so the brushing
	// halo stays visually correct). 1/zoomScale because the canvas
	// transform scales coordinates by zoomScale; dividing keeps the boost
	// constant in SCREEN px regardless of zoom level.
	const radiusBoostWorld = radiusBoostScreenPx / Math.max(zoomScale, 0.01);
	// PASS 1: all star bodies (additive blend via lower per-star alpha).
	// 2026-05-14 §A.14 fix-15: all notes render as CIRCLES per Eisa's
	// spec ("I want all the notes to take a circular shape"). Library
	// identity no longer encoded on the star body — at the new tiny
	// node sizes (5 px ⌀ at max zoom), shape distinguishability was
	// already weak. Library is still surfaced via hover tooltip + the
	// facet sidebar (with shape glyphs as legend keys, not on stars).
	// libraryShapeIndex stays in StarDerived for future surfaces.
	ctx.fillStyle = _chrome.starFill;
	for (const star of stars) {
		const r = (star.topDecileActs ? TOP_DECILE_RADIUS : BASE_STAR_RADIUS)
			+ radiusBoostWorld;
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		const opacity = isMatched
			? (star.row.confidenceAlpha ?? 0.45) * BODY_OPACITY_MULT
			: GHOST_ALPHA;
		ctx.globalAlpha = opacity;
		ctx.beginPath();
		ctx.arc(star.x, star.y, r, 0, Math.PI * 2);
		ctx.fill();
	}

	// PASS 2: all pips on top of bodies (full opacity per Bertin
	// orthogonality — pip hue carries categorical stage signal,
	// independent of confidence). With pass-2 ordering, the pip
	// is the LAST thing drawn at any (x,y), so it stays visible
	// in dense clusters where bodies overlap.
	// Ghost stars: pip drawn at GHOST_ALPHA so the encoding is faintly
	// visible (helpful context for "what category is this ghost in?")
	// without dominating.
	// MIG-026 §γ-fix-2: pip scales proportionally to body — 60% of the
	// body boost keeps the original pip:body 0.6 ratio in spread shapes.
	const pipBoostWorld = radiusBoostWorld * 0.6;
	for (const star of stars) {
		const pipColor = pipColorForStage(star.row.stage);
		if (!pipColor) continue;
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		ctx.globalAlpha = isMatched ? 1 : GHOST_ALPHA;
		ctx.fillStyle = pipColor;
		ctx.beginPath();
		ctx.arc(star.x, star.y, PIP_RADIUS + pipBoostWorld, 0, Math.PI * 2);
		ctx.fill();
	}
	ctx.globalAlpha = 1;

	// PASS 3: highlighted-brushing ring (above everything).
	if (highlightedPath !== null) {
		const star = stars.find((s) => s.row.notePath === highlightedPath);
		if (star) {
			const r = (star.topDecileActs ? TOP_DECILE_RADIUS : BASE_STAR_RADIUS)
				+ radiusBoostWorld;
			// 2026-05-14 §A.14 fix-16 (Boss-test cycle 3.6): ring screen
			// padding stays constant 4 px regardless of zoom. Pre-fix the
			// "+4" was world units, so at max zoom the ring became 32+ px
			// world × zoom = absurd halo around a 5-px node. Now: ring
			// world radius = node world radius + 4/zoomScale, yielding
			// node_screen_radius + 4 px screen halo at any zoom.
			// Linewidth 1.8 world also scales with zoom; divide by zoom
			// to keep stroke at constant ~1.8 px screen.
			const screenPadding = 4 / Math.max(zoomScale, 0.01);
			// MIG-027 §-fix-2: theme-aware hover ring (deep amber on light
			// themes, bright amber on dark; reads --sight-highlight CSS var)
			ctx.strokeStyle = _chrome.highlightedRing;
			ctx.lineWidth = 1.8 / Math.max(zoomScale, 0.01);
			ctx.beginPath();
			ctx.arc(star.x, star.y, r + screenPadding, 0, Math.PI * 2);
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

export function pipColorForStage(stage: string | null): string | null {
	// MIG-025 §B.6-fix-1 (2026-05-15): the Concept Paper v4.0 palette
	// originally recognized 5 abstract stages (established / fresh /
	// growing / at-risk / dormant). Eisa's universe (and the project's
	// own Living Link Architecture per CLAUDE.md) uses the 7-stage
	// lifecycle (Spark → Birth → Growth → Maturity → Dormancy →
	// Renewal → Archival). 99.3% of his 7,645 notes were falling back
	// to neutral gray because the renderer didn't recognize that
	// vocabulary, making the Stage mini visually indistinguishable
	// from the Confidence mini. Both vocabularies are now recognized;
	// the original 5 stay as fallbacks for any legacy frontmatter.
	//
	// Mapping (Living Link stage → Concept-Paper color slot):
	//   spark     → fresh (cyan)        — newly sparked idea
	//   birth     → growing (violet)    — taking form
	//   growth    → growing (violet)    — actively in motion (same energy as birth)
	//   maturity  → established (green) — fully formed
	//   dormancy  → dormant (gray)      — inactive
	//   renewal   → at-risk (yellow)    — recently revisited / re-emerging
	//   archival  → dormant (gray)      — closed
	//
	// `birth` and `growth` collapse to the same violet — distinguishing
	// them would require a 6th palette color (currently 5 slots per
	// Concept Paper §3.4 spec).
	if (stage === null) return null;
	switch (stage) {
		// Concept Paper v4.0 vocabulary (kept as fallbacks).
		case 'established': return PALETTE.stageEstablished;
		case 'fresh':       return PALETTE.stageFresh;
		case 'growing':     return PALETTE.stageGrowing;
		case 'at-risk':     return PALETTE.stageAtRisk;
		case 'dormant':     return PALETTE.stageDormant;
		// Living Link Architecture vocabulary (Eisa's actual data).
		// 2026-05-15 §B.6-fix-2 (Boss-test cycle 2): birth split out from
		// stageGrowing (violet) to stageBirth (orange) so spark/birth/growth
		// render as cyan/orange/violet instead of cyan/violet/violet. The two
		// dominant categories (spark 49% + birth 40% = 89%) now sit on
		// opposite sides of the warm-cool axis instead of blurring together.
		case 'spark':       return PALETTE.stageFresh;       // cyan
		case 'birth':       return PALETTE.stageBirth;       // orange (NEW)
		case 'growth':      return PALETTE.stageGrowing;     // violet
		case 'maturity':    return PALETTE.stageEstablished; // green
		case 'dormancy':    return PALETTE.stageDormant;     // gray
		case 'renewal':     return PALETTE.stageAtRisk;      // yellow
		case 'archival':    return PALETTE.stageDormant;     // gray
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

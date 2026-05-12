/**
 * Sight v5 — Canvas 2D render pipeline.
 *
 * Two-layer render strategy per Concept Paper v3.1 §11:
 *   - Base layer: dome chrome (8 strata bands, calendar rim, Milky Way
 *     wash) + faint connector lines + stars. Drawn ONCE per cache-warm
 *     cycle. Static; not redrawn per frame. Idle-Sight per-frame cost
 *     ≤ 1 ms (no redraw at rest).
 *   - Focus overlay: brightened constellation edges + selected-star
 *     ring + hover tooltips. Redrawn on hover/select state change
 *     ONLY (not per frame). Sparse element count even when focused.
 *
 * MIG-024 §3 ships the base-layer chrome (no stars yet; those land in
 * §5). The function signature accepts an empty `stars` array for §3.
 *
 * D-V1 lock (Eisa, 2026-05-12): Canvas 2D + D3-zoom. Inherits v4's
 * proven render path. Month labels are HTML overlay (NOT canvas-drawn
 * text) per v3 invariant 12 — `dir="auto"` handles RTL automatically.
 *
 * Coordinate system: Canvas y grows DOWN; we place the dome center at
 * (canvasWidth/2, canvasHeight/2) and offset all geometry from there.
 */

import {
	PALETTE,
	calendarRimSpokes,
	milkyWayEllipses,
	stratumBandBoundaries,
	type MilkyWayEllipse,
} from './dome';

/** Render the dome's static base layer onto the provided canvas
 *  context. Clears + redraws the full canvas; cheap (one CLEAR + ~30
 *  path operations on a 7,636-note universe with §5 stars added).
 *
 *  §3 deliverable: dome chrome only. §5 will add stars + connector
 *  lines (the function signature is forward-compatible — pass empty
 *  arrays today).
 *
 *  Performance: target ≤ 50 ms warm-cache draw on 1280×800 canvas at
 *  domeRadius ≈ 320 (the typical Sight-v5-full-screen size). The
 *  dome chrome itself is microsecond-scale; the budget headroom is
 *  for star rendering once §5 lands.
 */
export function renderBaseLayer(
	ctx: CanvasRenderingContext2D,
	canvasWidth: number,
	canvasHeight: number,
	domeRadius: number,
	currentMonthIndex: number,         // 0..11 — for the gold wedge tint
	modeWedgeAngles?: number[],        // §4: optional active-mode wedge dividers
): void {
	// Background fill (full canvas).
	ctx.fillStyle = PALETTE.parchment;
	ctx.fillRect(0, 0, canvasWidth, canvasHeight);

	// Translate to dome center for the rest of the draw.
	const cx = canvasWidth / 2;
	const cy = canvasHeight / 2;
	ctx.save();
	ctx.translate(cx, cy);

	// Soft Milky Way wash — two ellipses with radial gradient fade.
	drawMilkyWay(ctx, domeRadius);

	// Current-month wedge subtle highlight (gold @ 0.05 alpha).
	drawCurrentMonthWedge(ctx, domeRadius, currentMonthIndex);

	// Strata band rings (8 boundaries — outermost = solid rim, inner
	// 7 = faint guide lines).
	drawStrataBands(ctx, domeRadius);

	// Calendar rim outer accent line (dashed, just outside the
	// outermost strata boundary).
	drawCalendarRimAccent(ctx, domeRadius);

	// 12 month-wedge spokes (very faint) — always-on per Concept Paper
	// §5.2 (the calendar rim is the stable temporal reference).
	drawMonthSpokes(ctx, domeRadius);

	// §4 mode-specific wedge boundary spokes (gold, slightly more
	// visible than the calendar rim spokes — the active-mode cue).
	if (modeWedgeAngles && modeWedgeAngles.length > 0) {
		drawModeWedgeSpokes(ctx, domeRadius, modeWedgeAngles);
	}

	ctx.restore();
}

function drawMilkyWay(ctx: CanvasRenderingContext2D, domeRadius: number): void {
	const ellipses: MilkyWayEllipse[] = milkyWayEllipses(domeRadius);
	for (const e of ellipses) {
		ctx.save();
		ctx.translate(e.cx, e.cy);
		ctx.rotate((e.rotationDeg * Math.PI) / 180);

		// Radial gradient from center (alpha ~0.55) to edges (alpha 0).
		// Use the larger axis as the gradient extent.
		const gradient = ctx.createRadialGradient(0, 0, 0, 0, 0, Math.max(e.rx, e.ry));
		gradient.addColorStop(0, hexToRgba(PALETTE.milkyWay, 0.55));
		gradient.addColorStop(0.6, hexToRgba(PALETTE.milkyWay, 0.1));
		gradient.addColorStop(1, hexToRgba(PALETTE.milkyWay, 0));
		ctx.fillStyle = gradient;
		ctx.beginPath();
		ctx.ellipse(0, 0, e.rx, e.ry, 0, 0, Math.PI * 2);
		ctx.fill();
		ctx.restore();
	}
}

function drawCurrentMonthWedge(
	ctx: CanvasRenderingContext2D,
	domeRadius: number,
	currentMonthIndex: number,
): void {
	// Wedge from spoke[m] to spoke[m+1] (30° span). Clamp current month
	// to [0, 11].
	const m = Math.max(0, Math.min(11, currentMonthIndex));
	const startAngle = m * (Math.PI / 6) - Math.PI / 2;
	const endAngle = (m + 1) * (Math.PI / 6) - Math.PI / 2;
	ctx.save();
	ctx.fillStyle = hexToRgba(PALETTE.gold, 0.05);
	ctx.beginPath();
	ctx.moveTo(0, 0);
	ctx.arc(0, 0, domeRadius, startAngle, endAngle);
	ctx.closePath();
	ctx.fill();
	ctx.restore();
}

function drawStrataBands(ctx: CanvasRenderingContext2D, domeRadius: number): void {
	const radii = stratumBandBoundaries(domeRadius);
	for (let i = 0; i < radii.length; i++) {
		const r = radii[i];
		ctx.beginPath();
		ctx.arc(0, 0, r, 0, Math.PI * 2);
		if (i === 0) {
			// Outermost = solid rim
			ctx.strokeStyle = PALETTE.ruleFaint;
			ctx.lineWidth = 1.2;
			ctx.globalAlpha = 1;
		} else {
			// Inner rings = faint guides
			ctx.strokeStyle = PALETTE.ruleFaint;
			ctx.lineWidth = 0.5;
			ctx.globalAlpha = 0.55;
		}
		ctx.stroke();
	}
	ctx.globalAlpha = 1;
}

function drawCalendarRimAccent(ctx: CanvasRenderingContext2D, domeRadius: number): void {
	// Dashed ring just outside the dome — visual anchor for the
	// month-label HTML overlay layer.
	ctx.save();
	ctx.beginPath();
	ctx.arc(0, 0, domeRadius + 14, 0, Math.PI * 2);
	ctx.strokeStyle = PALETTE.ruleFaint;
	ctx.lineWidth = 0.5;
	ctx.setLineDash([2, 4]);
	ctx.globalAlpha = 0.5;
	ctx.stroke();
	ctx.restore();
}

function drawMonthSpokes(ctx: CanvasRenderingContext2D, domeRadius: number): void {
	ctx.save();
	ctx.strokeStyle = PALETTE.ruleFaint;
	ctx.lineWidth = 0.4;
	ctx.globalAlpha = 0.4;
	for (const angle of calendarRimSpokes()) {
		ctx.beginPath();
		ctx.moveTo(0, 0);
		ctx.lineTo(Math.cos(angle) * domeRadius, Math.sin(angle) * domeRadius);
		ctx.stroke();
	}
	ctx.restore();
}

function drawModeWedgeSpokes(
	ctx: CanvasRenderingContext2D,
	domeRadius: number,
	angles: number[],
): void {
	ctx.save();
	ctx.strokeStyle = PALETTE.gold;
	ctx.lineWidth = 0.6;
	ctx.globalAlpha = 0.4;
	for (const angle of angles) {
		ctx.beginPath();
		ctx.moveTo(0, 0);
		ctx.lineTo(Math.cos(angle) * domeRadius, Math.sin(angle) * domeRadius);
		ctx.stroke();
	}
	ctx.restore();
}

/** Hex `#RRGGBB` → `rgba(r, g, b, a)`. Tiny helper, no dependencies. */
function hexToRgba(hex: string, alpha: number): string {
	const h = hex.startsWith('#') ? hex.slice(1) : hex;
	const r = parseInt(h.slice(0, 2), 16);
	const g = parseInt(h.slice(2, 4), 16);
	const b = parseInt(h.slice(4, 6), 16);
	return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

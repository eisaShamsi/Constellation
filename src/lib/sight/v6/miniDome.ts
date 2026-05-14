/**
 * MIG-025 §B.1 — Sight v6 mini-dome renderer (skeleton).
 *
 * Single rendering function parameterized by `MiniDomeChannel`. Each
 * mini-dome shows the same notes as the anchor (in the same radial
 * position) but isolates ONE channel with its optimal visual property:
 *
 *   confidence  → opacity (channel only — uniform 2.8 px discs)
 *   stage       → full-disk hue (no pip; the mark IS the stage color)
 *   acts        → binary size (top-decile = 6 px filled; rest = 1.5 px dot)
 *   provenance  → 5 angular sectors (Self / Read / Heard / Reasoned / Tradition)
 *
 * Stratum bands preserved at 0.04 opacity in every mini so the radial-
 * anchor metaphor never disappears (Concept Paper §3.3 invariant).
 *
 * Per Concept Paper §2.3: mini-domes default to ≥320×320 px in
 * production; the Svelte wrapper sizes the canvas to its container.
 *
 * §B.1 deliverable: skeleton only — chrome (background + stratum
 * bands + channel-name title) + dispatch switch. Channel-specific
 * star rendering lands in §B.2 (confidence) → §B.3 (stage) → §B.4
 * (acts) → §B.5 (provenance).
 *
 * Per Plan-Approval cascade — does not include linked brushing
 * (§B.6) or cross-filter (§B.7) yet.
 */

import type { StarDerived, MiniDomeChannel, LayoutCacheRow } from './types';
import { PALETTE, stratumBandBoundaries } from './dome';

export interface MiniDomeLayout {
	centerX: number;
	centerY: number;
	radius: number;
}

/**
 * Compute layout for a mini-dome given canvas dimensions. Same
 * pattern as `computeDomeLayout` in anchor.ts but with a smaller
 * label margin (mini-domes don't show calendar month labels).
 */
export function computeMiniDomeLayout(width: number, height: number): MiniDomeLayout {
	const margin = 12; // smaller than anchor's 28 — no calendar rim
	const radius = Math.max(20, Math.min(width, height) / 2 - margin);
	return {
		centerX: width / 2,
		centerY: height / 2,
		radius,
	};
}

/**
 * Render a mini-dome for the given channel. Skeleton (§B.1):
 *   • Pass 0: clear + background (identity transform per fix-10 pattern)
 *   • Pass 1: stratum reference rings at 0.04 opacity
 *   • Pass 2: channel title text top-center
 *   • Pass 3: dispatch to channel-specific star renderer (filled in
 *             §B.2–§B.5)
 *   • Pass 4: highlighted-brushing ring (§B.6)
 *
 * Caller is responsible for setting an appropriate transform if
 * synchronizing zoom/pan with the anchor (Phase 2 may or may not
 * link these — to be decided in §B.6).
 */
export function renderMiniDome(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	channel: MiniDomeChannel,
	width: number,
	height: number,
	options: { highlightedPath?: string | null } = {},
): void {
	const { highlightedPath = null } = options;
	const layout = computeMiniDomeLayout(width, height);

	// Pass 0: clear + background (identity transform safe).
	ctx.save();
	ctx.setTransform(1, 0, 0, 1, 0, 0);
	ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
	ctx.fillStyle = PALETTE.bg;
	ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
	ctx.restore();

	// Pass 1: stratum reference rings (0.04 opacity per Concept Paper §3.3).
	// Thin gray strokes; serve as the radial-anchor metaphor's preserved
	// frame. Without them, mini-dome stars float without context.
	ctx.save();
	ctx.globalAlpha = 0.04;
	ctx.strokeStyle = PALETTE.strataRing;
	ctx.lineWidth = 0.6;
	for (const r of stratumBandBoundaries(layout.radius)) {
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}
	ctx.restore();

	// Pass 2: channel title text top-center.
	ctx.save();
	ctx.fillStyle = PALETTE.subtitleText;
	ctx.font = '10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'top';
	ctx.fillText(channelTitle(channel), layout.centerX, 4);
	ctx.restore();

	// Pass 3: dispatch to channel-specific renderer (skeletons stub
	// until §B.2–§B.5 fill them in). Each renderer reads the stars'
	// pre-computed (x, y) which were positioned by the anchor's
	// `computeStarPositions` — mini-domes inherit the same world-space
	// positions so linked brushing in §B.6 matches up trivially.
	switch (channel) {
		case 'confidence':
			renderConfidenceChannel(ctx, stars, layout);
			break;
		case 'stage':
			renderStageChannel(ctx, stars, layout);
			break;
		case 'acts':
			renderActsChannel(ctx, stars, layout);
			break;
		case 'provenance':
			renderProvenanceChannel(ctx, stars, layout);
			break;
	}

	// Pass 4: highlighted-brushing ring (§B.6 implementation).
	// Skeleton: matches anchor.ts pattern; renders gold ring around
	// the matching star if any.
	if (highlightedPath !== null) {
		const star = stars.find((s) => s.row.notePath === highlightedPath);
		if (star) {
			ctx.save();
			ctx.strokeStyle = PALETTE.highlightedRing;
			ctx.lineWidth = 1.4;
			ctx.beginPath();
			ctx.arc(star.x, star.y, 6, 0, Math.PI * 2);
			ctx.stroke();
			ctx.restore();
		}
	}
}

// ════════════════════════════════════════════════════════════════════
// Channel-specific renderers (§B.2–§B.5 stubs)
// ════════════════════════════════════════════════════════════════════

/** §B.2: opacity-only rendering — uniform 2.8 px discs, alpha = confidenceAlpha. */
function renderConfidenceChannel(
	_ctx: CanvasRenderingContext2D,
	_stars: StarDerived[],
	_layout: MiniDomeLayout,
): void {
	// §B.2 implementation pending.
}

/** §B.3: full-disk hue per stage (no pip). 5 categorical colors. */
function renderStageChannel(
	_ctx: CanvasRenderingContext2D,
	_stars: StarDerived[],
	_layout: MiniDomeLayout,
): void {
	// §B.3 implementation pending.
}

/** §B.4: binary size — top-decile = 6 px filled disc; rest = 1.5 px dot. */
function renderActsChannel(
	_ctx: CanvasRenderingContext2D,
	_stars: StarDerived[],
	_layout: MiniDomeLayout,
): void {
	// §B.4 implementation pending.
}

/** §B.5: 5 angular sectors (Self/Read/Heard/Reasoned/Tradition).
 *  Re-positions stars from anchor positions to provenance-sector
 *  layout (different geometry than other minis). */
function renderProvenanceChannel(
	_ctx: CanvasRenderingContext2D,
	_stars: StarDerived[],
	_layout: MiniDomeLayout,
): void {
	// §B.5 implementation pending.
}

// ════════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════════

function channelTitle(channel: MiniDomeChannel): string {
	switch (channel) {
		case 'confidence':
			return 'CONFIDENCE — opacity';
		case 'stage':
			return 'STAGE — hue (full-disk)';
		case 'acts':
			return 'ACTS — size (top decile)';
		case 'provenance':
			return 'PROVENANCE — 5 sectors';
	}
}

/** Compute Universe-wide context for channel rendering. Currently
 *  unused (skeletons stubbed); will be consumed by §B.4 (acts top-
 *  decile threshold) once that lands. */
export function topDecileLinkCount(rows: LayoutCacheRow[]): number {
	if (rows.length === 0) return Infinity;
	const counts = rows
		.map((r) => r.linkInCount + r.linkOutCount)
		.sort((a, b) => a - b);
	const idx = Math.floor(counts.length * 0.9);
	return counts[Math.min(idx, counts.length - 1)] || 1;
}

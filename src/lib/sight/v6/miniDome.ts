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

import type { StarDerived, MiniDomeChannel, LayoutCacheRow, ProvenanceSector } from './types';
import { PALETTE, stratumBandBoundaries } from './dome';
import { pipColorForStage, type DomeLayout } from './anchor';

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
	anchorLayout: DomeLayout,
	options: { highlightedPath?: string | null } = {},
): void {
	const { highlightedPath = null } = options;
	const layout = computeMiniDomeLayout(width, height);
	// Coordinate transform: anchor world coords → mini canvas coords.
	// Channel renderers consume StarDerived.x/y (anchor coords) and
	// re-project per star via this scale + offset. Provenance channel
	// (§B.5) ignores this scale and uses its own sector layout.
	const scale = layout.radius / anchorLayout.radius;
	const offsetX = layout.centerX - anchorLayout.centerX * scale;
	const offsetY = layout.centerY - anchorLayout.centerY * scale;

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
			renderConfidenceChannel(ctx, stars, scale, offsetX, offsetY);
			break;
		case 'stage':
			renderStageChannel(ctx, stars, scale, offsetX, offsetY);
			break;
		case 'acts':
			renderActsChannel(ctx, stars, scale, offsetX, offsetY);
			break;
		case 'provenance':
			renderProvenanceChannel(ctx, stars, layout);
			break;
	}

	// Pass 4: highlighted-brushing ring (§B.6 implementation).
	// Skeleton: matches anchor.ts pattern; renders gold ring around
	// the matching star if any. Uses anchor→mini scale for non-
	// provenance channels; provenance gets its own coords (§B.5).
	if (highlightedPath !== null) {
		const star = stars.find((s) => s.row.notePath === highlightedPath);
		if (star) {
			let hx: number;
			let hy: number;
			if (channel === 'provenance') {
				const p = provenancePositionFor(star, computeMiniDomeLayout(width, height));
				hx = p.x;
				hy = p.y;
			} else {
				hx = star.x * scale + offsetX;
				hy = star.y * scale + offsetY;
			}
			ctx.save();
			ctx.strokeStyle = PALETTE.highlightedRing;
			ctx.lineWidth = 1.4;
			ctx.beginPath();
			ctx.arc(hx, hy, 6, 0, Math.PI * 2);
			ctx.stroke();
			ctx.restore();
		}
	}
}

// ════════════════════════════════════════════════════════════════════
// Channel-specific renderers (§B.2–§B.5 stubs)
// ════════════════════════════════════════════════════════════════════

/** §B.2: opacity-only rendering — uniform 2.8 px discs, alpha =
 *  confidenceAlpha. Per Concept Paper §2.3: this mini-dome ISOLATES
 *  the confidence channel — no pip, no shape variation, no top-decile
 *  size delta. The visible signal is purely opacity gradient.
 *
 *  Star fill is neutral (PALETTE.starFill); the alpha multiplier
 *  is the channel. Hypothesis stars (alpha 0.45) appear faint;
 *  established stars (alpha 1.0) appear crisp. Pre-attentive opacity
 *  per Mackinlay encoding-effectiveness ranking. */
function renderConfidenceChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	scale: number,
	offsetX: number,
	offsetY: number,
): void {
	ctx.fillStyle = PALETTE.starFill;
	for (const star of stars) {
		const opacity = star.row.confidenceAlpha ?? 0.45;
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		ctx.globalAlpha = opacity;
		ctx.beginPath();
		ctx.arc(x, y, 2.8, 0, Math.PI * 2);
		ctx.fill();
	}
	ctx.globalAlpha = 1;
}

/** §B.3: full-disk hue per stage (no pip). 5 categorical colors per
 *  Concept Paper §3.4 stage palette:
 *    established → green   #4ade80
 *    fresh       → cyan    #22d3ee
 *    growing     → violet  #a78bfa
 *    at-risk     → yellow  #facc15
 *    dormant     → gray    #94a3b8
 *
 *  Per Concept Paper §2.3: this mini-dome ISOLATES stage. Full-disk
 *  hue (the entire mark IS the stage color) is pre-attentive — Stage
 *  pops here in a way it never does on the anchor (where it's a tiny
 *  ~3 px pip at max zoom). 2.8 px discs match the confidence channel's
 *  baseline size for visual consistency across minis. */
function renderStageChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	scale: number,
	offsetX: number,
	offsetY: number,
): void {
	ctx.globalAlpha = 1;
	for (const star of stars) {
		const stageColor = pipColorForStage(star.row.stage);
		if (!stageColor) continue;
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		ctx.fillStyle = stageColor;
		ctx.beginPath();
		ctx.arc(x, y, 2.8, 0, Math.PI * 2);
		ctx.fill();
	}
}

/** §B.4: binary size — top-decile = 6 px filled disc; rest = 1.5 px
 *  dot. Per Concept Paper §2.3: this mini-dome ISOLATES the acts
 *  channel — top-decile-acts notes (computed by anchor's
 *  computeStarPositions as `topDecileActs: boolean`) become visible
 *  hot-spots; the rest fade to background. Pre-attentive size delta
 *  per Treisman primitive (>30% threshold honored: 6/1.5 = 4× ratio). */
function renderActsChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	scale: number,
	offsetX: number,
	offsetY: number,
): void {
	ctx.fillStyle = PALETTE.starFill;
	ctx.globalAlpha = 1;
	for (const star of stars) {
		const r = star.topDecileActs ? 6 : 1.5;
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		ctx.beginPath();
		ctx.arc(x, y, r, 0, Math.PI * 2);
		ctx.fill();
	}
}

/** §B.5: 5 angular sectors (Self/Read/Heard/Reasoned/Tradition).
 *  Per Concept Paper §2.3: this mini-dome ISOLATES provenance via
 *  RE-POSITIONING — angular sector = source bucket; radial = stratum
 *  (preserved from anchor). Sector dividers visible at 0.2 opacity.
 *  Sector labels at outer rim.
 *
 *  This is the only mini-dome with its own positioning math (other
 *  three reuse anchor's stratum × time layout). Linked-brushing
 *  position lookup (§B.6) uses provenancePositionFor() helper. */
const PROVENANCE_SECTORS: ProvenanceSector[] = [
	'Self',
	'Read',
	'Heard',
	'Reasoned',
	'Tradition',
];

function renderProvenanceChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	layout: MiniDomeLayout,
): void {
	const sectorCount = PROVENANCE_SECTORS.length;
	const sectorAngle = (Math.PI * 2) / sectorCount;
	// Top of dome = -π/2 (canvas math); first sector starts there.

	// Sector dividers — 5 lines from center to outer radius.
	ctx.save();
	ctx.globalAlpha = 0.2;
	ctx.strokeStyle = PALETTE.strataRing;
	ctx.lineWidth = 0.6;
	for (let i = 0; i < sectorCount; i++) {
		const angle = -Math.PI / 2 + i * sectorAngle;
		ctx.beginPath();
		ctx.moveTo(layout.centerX, layout.centerY);
		ctx.lineTo(
			layout.centerX + Math.cos(angle) * layout.radius,
			layout.centerY + Math.sin(angle) * layout.radius,
		);
		ctx.stroke();
	}
	ctx.restore();

	// Sector labels at outer rim.
	ctx.save();
	ctx.fillStyle = PALETTE.subtitleText;
	ctx.font = '8px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	const labelRadius = layout.radius - 6;
	for (let i = 0; i < sectorCount; i++) {
		const angle = -Math.PI / 2 + i * sectorAngle + sectorAngle / 2;
		const lx = layout.centerX + Math.cos(angle) * labelRadius;
		const ly = layout.centerY + Math.sin(angle) * labelRadius;
		ctx.fillText(PROVENANCE_SECTORS[i], lx, ly);
	}
	ctx.restore();

	// Stars — re-positioned per provenance sector × stratum.
	ctx.fillStyle = PALETTE.starFill;
	ctx.globalAlpha = 1;
	for (const star of stars) {
		const pos = provenancePositionFor(star, layout);
		ctx.beginPath();
		ctx.arc(pos.x, pos.y, 2, 0, Math.PI * 2);
		ctx.fill();
	}
}

/** Compute provenance-mini-dome position for a star. Used by render
 *  + by linked-brushing ring-placement lookup. Deterministic per
 *  notePath via FNV-1a hash for intra-sector jitter. */
function provenancePositionFor(
	star: StarDerived,
	layout: MiniDomeLayout,
): { x: number; y: number } {
	const sector = star.provenanceSector ?? 'Self';
	const sectorIdx = PROVENANCE_SECTORS.indexOf(sector);
	const sectorAngle = (Math.PI * 2) / PROVENANCE_SECTORS.length;
	// Sector center angle (canvas math: -π/2 = top, clockwise).
	const sectorCenterAngle =
		-Math.PI / 2 + sectorIdx * sectorAngle + sectorAngle / 2;
	// Deterministic hash → jitter within sector.
	const [jitterRadial, jitterAngular] = pathHash2(star.row.notePath);
	// Radial: place between 0.15 × radius and 0.85 × radius (avoid
	// center crowding + give label space at rim). Stratum NOT preserved
	// here — the provenance mini visualizes by source primarily.
	const radial = layout.radius * (0.15 + jitterRadial * 0.7);
	// Angular: ±0.4 × sector half-arc (most of the wedge).
	const angularJitter = (jitterAngular - 0.5) * sectorAngle * 0.8;
	const angle = sectorCenterAngle + angularJitter;
	return {
		x: layout.centerX + Math.cos(angle) * radial,
		y: layout.centerY + Math.sin(angle) * radial,
	};
}

/** Local FNV-1a hash → two normalized [0,1) values. Internal helper
 *  for §B.5 provenance jitter; mirrors the now-restored pathHashJitter
 *  in anchor.ts but kept here to avoid an import cycle. */
function pathHash2(path: string): [number, number] {
	let h = 0x811c9dc5;
	for (let i = 0; i < path.length; i++) {
		h ^= path.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	const u32 = h >>> 0;
	return [(u32 & 0xffff) / 0xffff, ((u32 >>> 16) & 0xffff) / 0xffff];
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

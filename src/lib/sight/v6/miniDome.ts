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

import type { StarDerived, MiniDomeChannel, SlotChannel, LayoutCacheRow, ProvenanceSector } from './types';
import { PALETTE, CHROME_PALETTE_DARK_FALLBACK, stratumBandBoundaries, type ChromePalette } from './dome';
import { pipColorForStage, starHitTest, type DomeLayout } from './anchor';

// MIG-027 — module-level chrome palette (same pattern as anchor.ts).
// renderMiniDome sets this at the top of each paint from the caller's
// chromePalette option. Helper functions in this file read chrome
// colors via _chrome.* (theme-aware) and semantic colors via
// PALETTE.stageX / PALETTE.linkX (theme-agnostic categorical).
let _chrome: ChromePalette = CHROME_PALETTE_DARK_FALLBACK;

// MIG-026 §λ-fix-3 — module-level i18n label resolver (same pattern
// as anchor.ts _labelize). renderMiniDome sets this from the caller's
// `labelize` option; renderProvenanceChannel + the channel-title pass
// read via _labelize(key) to localize on-canvas text. Defaults to
// identity so the canvas still paints (with English keys) if a caller
// forgets to wire it.
let _labelize: (key: string) => string = (key) => key;

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
	channel: SlotChannel,
	width: number,
	height: number,
	anchorLayout: DomeLayout,
	options: {
		highlightedPath?: string | null;
		dotRadius?: number;
		zoomScale?: number;
		matchedPaths?: Set<string> | null;
		densityMode?: boolean;
		// MIG-027 — theme-aware chrome palette. MiniDome.svelte's
		// paint() reads via readChromePalette(canvasEl); if not
		// provided, defaults to dark fallback.
		chromePalette?: ChromePalette;
		// MIG-026 §λ-fix-3 — i18n label resolver. Same contract as
		// renderAnchorDome.labelize: when provided, the channel-title
		// pass and the provenance sector-label pass run text through
		// labelize(key). When absent, falls back to identity (English
		// key text). Callers should provide $t.
		labelize?: (key: string) => string;
	} = {},
): void {
	const {
		highlightedPath = null,
		dotRadius = 0.75,
		zoomScale = 1,
		matchedPaths = null,
		densityMode = false,
		chromePalette = CHROME_PALETTE_DARK_FALLBACK,
		labelize = (key: string) => key,
	} = options;
	// MIG-027 — set the module-level chrome state for this paint.
	_chrome = chromePalette;
	// MIG-026 §λ-fix-3 — same module-level pattern for i18n.
	_labelize = labelize;
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
	ctx.fillStyle = _chrome.bg;
	ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
	ctx.restore();

	// Pass 1: stratum reference rings.
	// 2026-05-14 §B-fix-2 (Eisa Boss-test §B-preview-2): "Should the
	// mini-domes have their own inner circles ... If yes, then why
	// isn't it visible?" — they were rendered at globalAlpha 0.04 per
	// Concept Paper §3.3 spec, which made them effectively invisible
	// against the dark background. Bumped to full opacity (PALETTE
	// .strataRing #2a3245 is dark enough that 1.0 alpha still reads
	// as "supporting actor", not as dominating chrome). Anchor uses
	// the same color at 0.9-px stroke; minis use 0.5-px stroke (since
	// minis are smaller, thinner stroke keeps the visual weight
	// proportional).
	// 2026-05-15 §B.6-fix-1 (Boss-test cycle 1.4): Eisa: "I want the
	// mini-domes' inner circles, font color, and opacity to match the
	// anchor dome exactly." Stroke width bumped 0.5 → 0.9 to match the
	// anchor's stratum-ring stroke (anchor.ts:267). Visual weight is
	// now identical between anchor and minis — the radial-anchor
	// metaphor reads consistently across all 5 surfaces.
	ctx.save();
	ctx.strokeStyle = _chrome.strataRing;
	ctx.lineWidth = 0.9;
	for (const r of stratumBandBoundaries(layout.radius)) {
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}
	ctx.restore();

	// Pass 2: channel title text top-center.
	// 2026-05-15 §B.6-fix-1 (Boss-test cycle 1.4): bumped from
	// _chrome.subtitleText (faint #5a6275) to _chrome.titleText
	// (bright cream #e8ebf2) to match the anchor's header-strip text
	// color. The mini titles were too faint to read against the dark
	// background; now they read as confident labels at the same
	// visual weight as the anchor's "Constellation Sight" title.
	ctx.save();
	ctx.fillStyle = _chrome.titleText;
	ctx.font = '10px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'top';
	ctx.fillText(_labelize(channelTitleKey(channel)), layout.centerX, 4);
	ctx.restore();

	// Pass 3: dispatch to channel-specific renderer (skeletons stub
	// until §B.2–§B.5 fill them in). Each renderer reads the stars'
	// pre-computed (x, y) which were positioned by the anchor's
	// `computeStarPositions` — mini-domes inherit the same world-space
	// positions so linked brushing in §B.6 matches up trivially.
	switch (channel) {
		case 'anchor':
			renderAnchorChannel(ctx, stars, scale, offsetX, offsetY, dotRadius, matchedPaths, densityMode);
			break;
		case 'confidence':
			renderConfidenceChannel(ctx, stars, scale, offsetX, offsetY, dotRadius, matchedPaths, densityMode);
			break;
		case 'stage':
			renderStageChannel(ctx, stars, scale, offsetX, offsetY, dotRadius, matchedPaths, densityMode);
			break;
		case 'acts':
			renderActsChannel(ctx, stars, scale, offsetX, offsetY, dotRadius, matchedPaths, densityMode);
			break;
		case 'provenance':
			renderProvenanceChannel(ctx, stars, layout, dotRadius, matchedPaths, densityMode);
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
			// 2026-05-15 §B.7-fix-1 (Eisa cycle-1 Stage 1.3): "Why is
			// the gold circle large? It has to match the node size,
			// even when zooming in." Previously: hardcoded radius 6
			// + line widths 4/2.2 in world coords. With promoted zoom
			// transform applied, those scaled multiplicatively → at
			// zoom 24× the ring became ~144px on screen (vs 48px node).
			// New formula: ring radius = dotRadius + 1.5/zoomScale and
			// stroke widths scale as 1/zoomScale, so the on-screen
			// ring sits ~1.5 screen-px outside the node + the halo and
			// gold strokes stay at ~constant 2/1px screen widths
			// regardless of zoom level. Ring "hugs" the node at any
			// zoom instead of ballooning. Compact slot passes
			// zoomScale=1 (no transform applied), so formula reduces
			// to dotRadius + 1.5 = ~2.25 world ≈ 4.5-px ⌀ ring around
			// 1.5-px ⌀ dots — visible marker without dominating.
			const ringRadiusWorld = dotRadius + 1.5 / zoomScale;
			const haloLineWidth = 2 / zoomScale;
			const goldLineWidth = 1 / zoomScale;
			ctx.save();
			// Halo (drawn first, underneath the gold).
			ctx.strokeStyle = _chrome.bg;
			ctx.lineWidth = haloLineWidth;
			ctx.beginPath();
			ctx.arc(hx, hy, ringRadiusWorld, 0, Math.PI * 2);
			ctx.stroke();
			// Gold ring on top. MIG-027 §-fix-2: theme-aware via _chrome
			// (reads --sight-highlight CSS var; deep amber on light, bright
			// amber on dark)
			ctx.strokeStyle = _chrome.highlightedRing;
			ctx.lineWidth = goldLineWidth;
			ctx.beginPath();
			ctx.arc(hx, hy, ringRadiusWorld, 0, Math.PI * 2);
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
 *  Star fill is neutral (_chrome.starFill); the alpha multiplier
 *  is the channel. Hypothesis stars (alpha 0.45) appear faint;
 *  established stars (alpha 1.0) appear crisp. Pre-attentive opacity
 *  per Mackinlay encoding-effectiveness ranking. */
function renderConfidenceChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	scale: number,
	offsetX: number,
	offsetY: number,
	dotRadius: number = 0.75,
	matchedPaths: Set<string> | null = null,
	densityMode: boolean = false,
): void {
	// 2026-05-15 §B.7-fix-2: ghost mode (non-matched at GHOST_ALPHA).
	// 2026-05-16 §B.9: density mode (matched count > threshold): use
	// MATCHED_DENSITY_ALPHA so overlapping stars additive-blend into
	// a perceptual density gradient instead of individual dots.
	const GHOST_ALPHA = 0.15;
	const MATCHED_DENSITY_ALPHA = 0.35;
	const GHOST_DENSITY_ALPHA = 0.05;
	ctx.fillStyle = _chrome.starFill;
	for (const star of stars) {
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		const opacity = isMatched
			? (densityMode ? MATCHED_DENSITY_ALPHA : (star.row.confidenceAlpha ?? 0.45))
			: (densityMode ? GHOST_DENSITY_ALPHA : GHOST_ALPHA);
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		ctx.globalAlpha = opacity;
		ctx.beginPath();
		ctx.arc(x, y, dotRadius, 0, Math.PI * 2);
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
	dotRadius: number = 0.75,
	matchedPaths: Set<string> | null = null,
	densityMode: boolean = false,
): void {
	// §B.9 density mode: lower alpha for additive blending when too many.
	const GHOST_ALPHA = 0.15;
	const MATCHED_DENSITY_ALPHA = 0.35;
	const GHOST_DENSITY_ALPHA = 0.05;
	for (const star of stars) {
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		const stageColor = pipColorForStage(star.row.stage) ?? _chrome.starFill;
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		ctx.globalAlpha = isMatched
			? (densityMode ? MATCHED_DENSITY_ALPHA : 1)
			: (densityMode ? GHOST_DENSITY_ALPHA : GHOST_ALPHA);
		ctx.fillStyle = stageColor;
		ctx.beginPath();
		ctx.arc(x, y, dotRadius, 0, Math.PI * 2);
		ctx.fill();
	}
	ctx.globalAlpha = 1;
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
	dotRadius: number = 0.75,
	matchedPaths: Set<string> | null = null,
	densityMode: boolean = false,
): void {
	// Acts: 4× top-decile ratio in compact, no ratio in promoted (§B.6-fix-7).
	// §B.7-fix-2: ghost non-matched. §B.9: density mode lowers alpha.
	const GHOST_ALPHA = 0.15;
	const MATCHED_DENSITY_ALPHA = 0.35;
	const GHOST_DENSITY_ALPHA = 0.05;
	ctx.fillStyle = _chrome.starFill;
	const topDecileRadius = dotRadius < 1 ? dotRadius * 4 : dotRadius;
	for (const star of stars) {
		const r = star.topDecileActs ? topDecileRadius : dotRadius;
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		ctx.globalAlpha = isMatched
			? (densityMode ? MATCHED_DENSITY_ALPHA : 1)
			: (densityMode ? GHOST_DENSITY_ALPHA : GHOST_ALPHA);
		ctx.beginPath();
		ctx.arc(x, y, r, 0, Math.PI * 2);
		ctx.fill();
	}
	ctx.globalAlpha = 1;
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

/** MIG-026 §λ-fix-3 — parallel i18n keys for the provenance sector
 *  labels above. The literal English bucket names stay in
 *  PROVENANCE_SECTORS because they're matched against StarDerived
 *  .provenanceSector (the same literal value lives on the row from
 *  the Rust side) — those identifiers must NOT be translated. Only
 *  the on-canvas label text is localized via this parallel array. */
const PROVENANCE_SECTOR_LABEL_KEYS: string[] = [
	'sight.v6.miniDome.provenance.self',
	'sight.v6.miniDome.provenance.read',
	'sight.v6.miniDome.provenance.heard',
	'sight.v6.miniDome.provenance.reasoned',
	'sight.v6.miniDome.provenance.tradition',
];

function renderProvenanceChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	layout: MiniDomeLayout,
	dotRadius: number = 0.75,
	matchedPaths: Set<string> | null = null,
	densityMode: boolean = false,
): void {
	const GHOST_ALPHA = 0.15;
	const MATCHED_DENSITY_ALPHA = 0.35;
	const GHOST_DENSITY_ALPHA = 0.05;
	const sectorCount = PROVENANCE_SECTORS.length;
	const sectorAngle = (Math.PI * 2) / sectorCount;
	// Top of dome = -π/2 (canvas math); first sector starts there.

	// Sector dividers — 5 lines from center to outer radius.
	ctx.save();
	ctx.globalAlpha = 0.2;
	ctx.strokeStyle = _chrome.strataRing;
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
	ctx.fillStyle = _chrome.subtitleText;
	ctx.font = '8px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	const labelRadius = layout.radius - 6;
	for (let i = 0; i < sectorCount; i++) {
		const angle = -Math.PI / 2 + i * sectorAngle + sectorAngle / 2;
		const lx = layout.centerX + Math.cos(angle) * labelRadius;
		const ly = layout.centerY + Math.sin(angle) * labelRadius;
		ctx.fillText(_labelize(PROVENANCE_SECTOR_LABEL_KEYS[i]), lx, ly);
	}
	ctx.restore();

	// Stars — re-positioned per provenance sector × stratum.
	// §B-fix-3: 0.75 radius (1.5 px ⌀). §B.6-fix-3: dotRadius parameterized.
	// §B.7-fix-2: ghost-mode. §B.9: density-mode lowers alpha.
	ctx.fillStyle = _chrome.starFill;
	for (const star of stars) {
		const pos = provenancePositionFor(star, layout);
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		ctx.globalAlpha = isMatched
			? (densityMode ? MATCHED_DENSITY_ALPHA : 1)
			: (densityMode ? GHOST_DENSITY_ALPHA : GHOST_ALPHA);
		ctx.beginPath();
		ctx.arc(pos.x, pos.y, dotRadius, 0, Math.PI * 2);
		ctx.fill();
	}
	ctx.globalAlpha = 1;
}

/** §B.6-fix-3: render the anchor view as a channel option (used when
 *  anchor is demoted into a mini slot, OR when 'anchor' is the
 *  promoted primary slot). Plain neutral cream stars — no encoding —
 *  matching the anchor dome's baseline appearance. Uses the same
 *  stratum × time positioning the other 3 spatial-shared channels
 *  use (confidence/stage/acts), so linked brushing still aligns. */
function renderAnchorChannel(
	ctx: CanvasRenderingContext2D,
	stars: StarDerived[],
	scale: number,
	offsetX: number,
	offsetY: number,
	dotRadius: number = 0.75,
	matchedPaths: Set<string> | null = null,
	densityMode: boolean = false,
): void {
	const GHOST_ALPHA = 0.15;
	const MATCHED_DENSITY_ALPHA = 0.35;
	const GHOST_DENSITY_ALPHA = 0.05;
	ctx.fillStyle = _chrome.starFill;
	for (const star of stars) {
		const x = star.x * scale + offsetX;
		const y = star.y * scale + offsetY;
		const isMatched = matchedPaths === null || matchedPaths.has(star.row.notePath);
		ctx.globalAlpha = isMatched
			? (densityMode ? MATCHED_DENSITY_ALPHA : 1)
			: (densityMode ? GHOST_DENSITY_ALPHA : GHOST_ALPHA);
		ctx.beginPath();
		ctx.arc(x, y, dotRadius, 0, Math.PI * 2);
		ctx.fill();
	}
	ctx.globalAlpha = 1;
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

/** MIG-026 §λ-fix-3 — return i18n key for the mini-dome channel title.
 *  Keys live under `sight.v6.miniDome.title.<channel>` in every locale
 *  file. The rendered text passes through `_labelize` (set from the
 *  caller's $t store) so it always matches the active interface
 *  language. The old `channelTitle` returned hardcoded English strings
 *  and was the entry-point for the Arabic Boss-test gap on
 *  2026-05-18 (mini-dome titles read "CONFIDENCE — opacity" instead
 *  of localized form). */
function channelTitleKey(channel: SlotChannel): string {
	switch (channel) {
		case 'anchor':
			return 'sight.v6.miniDome.title.anchor';
		case 'confidence':
			return 'sight.v6.miniDome.title.confidence';
		case 'stage':
			return 'sight.v6.miniDome.title.stage';
		case 'acts':
			return 'sight.v6.miniDome.title.acts';
		case 'provenance':
			return 'sight.v6.miniDome.title.provenance';
	}
}

/**
 * §B.6 — channel-aware hit-test for mini-dome canvases. Returns the
 * notePath of the star under the cursor (within `tolerance` canvas-
 * space pixels) or null. Used by MiniDome.svelte's pointermove handler
 * to dispatch hover events upward, completing the bidirectional linked-
 * brushing loop (anchor ↔ all 4 minis).
 *
 * Per-channel logic:
 *   • provenance — stars are positioned by sector × jittered radial
 *                  (provenancePositionFor); iterate stars and pick
 *                  the closest within tolerance in canvas space.
 *   • confidence/stage/acts — stars share the anchor's stratum × time
 *                  positions, scaled into mini space by `scale`. Convert
 *                  the cursor's canvas coords back to anchor world
 *                  coords (inverse of the render transform) and reuse
 *                  anchor's `starHitTest`. World tolerance scales as
 *                  tolerance/scale so the on-screen tolerance stays
 *                  constant in canvas pixels regardless of mini size.
 */
export function miniDomeHitTest(
	stars: StarDerived[],
	canvasX: number,
	canvasY: number,
	channel: SlotChannel,
	anchorLayout: DomeLayout,
	miniWidth: number,
	miniHeight: number,
	tolerance: number = 12,
): string | null {
	const layout = computeMiniDomeLayout(miniWidth, miniHeight);
	if (channel === 'provenance') {
		let bestPath: string | null = null;
		let bestDistSq = tolerance * tolerance;
		for (const star of stars) {
			const pos = provenancePositionFor(star, layout);
			const dx = canvasX - pos.x;
			const dy = canvasY - pos.y;
			const d2 = dx * dx + dy * dy;
			if (d2 < bestDistSq) {
				bestDistSq = d2;
				bestPath = star.row.notePath;
			}
		}
		return bestPath;
	}
	// Non-provenance: invert the render transform.
	// renderMiniDome maps (star.x, star.y) →
	//   (star.x * scale + offsetX, star.y * scale + offsetY)
	// where offsetX = layout.centerX - anchorLayout.centerX * scale,
	//       offsetY = layout.centerY - anchorLayout.centerY * scale.
	// Inverse: (canvasX, canvasY) →
	//   ((canvasX - layout.centerX) / scale + anchorLayout.centerX,
	//    (canvasY - layout.centerY) / scale + anchorLayout.centerY)
	const scale = layout.radius / anchorLayout.radius;
	if (scale === 0) return null;
	const ax = (canvasX - layout.centerX) / scale + anchorLayout.centerX;
	const ay = (canvasY - layout.centerY) / scale + anchorLayout.centerY;
	return starHitTest(stars, ax, ay, tolerance / scale);
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

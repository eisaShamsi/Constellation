/**
 * MIG-036 P3 (2026-05-19) — Sight v7 universe-view dispatcher.
 *
 * The v6 ancestor (`src/lib/sight/v6/anchor.ts::renderAnchorDome`)
 * computed a per-star position for every note and painted the dome
 * as a starfield. v7 replaces that with a CATEGORICAL render: each
 * tradition declares its cells (via `tradition.cellRegions(layout)`),
 * the dispatcher counts how many notes land in each cell (via
 * `tradition.cellMembership(row)`), and paints ONE density blob per
 * cell whose magnitude encodes the cell's population.
 *
 * This is the Form-Aligns-To-Purpose payoff: the universe view's
 * single question is "how is my knowledge distributed across this
 * tradition's grammar?" — one blob per cell answers it pre-attentively.
 * Per-note positions inside cells (the v6 within-quadrant angular
 * placement) carried no analytical signal; v7 drops them entirely.
 * Drill-in (P7, future) hands the user the per-note view via the
 * stack primitive.
 *
 * Layers painted (back → front):
 *   1. Background (theme bg)
 *   2. Stratum guide rings (UNIVERSAL — every tradition gets the
 *      5-ring maturity scaffold; per Architect §3 "what stays
 *      universal across all traditions")
 *   3. Stratum labels along vertical axis (UNIVERSAL)
 *   4. Calendar rim (CONDITIONAL — only when
 *      `tradition.showCalendarRim === true`; per the dropped-from-
 *      traditions decision, only the Time Dome opts in)
 *   5. Per-cell density blobs (the v7 essence)
 *   6. Per-cell labels
 *   7. Hover/selection rings (interaction affordance)
 *
 * Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md §3, §4, §5
 */

import {
	stratumBandBoundaries,
	STRATUM_BANDS,
	STRATUM_LABELS,
	STRATUM_LABEL_KEYS,
	radiusForStratum,
	calendarRimMonths,
	CHROME_PALETTE_DARK_FALLBACK,
	type ChromePalette,
} from '../v6/dome';
// computeDomeLayout lives in v6/anchor.ts (not dome.ts) — reusing it
// keeps v7 dome geometry pixel-identical to v6 so the Boss-test
// visual diff stays clean across the v6 ↔ v7 ship gate.
import { computeDomeLayout } from '../v6/anchor';
import {
	computeDensityScale,
	cellDensity,
	type CellDensity,
} from './density';
import type {
	TraditionModuleV7,
	CellGeometry,
	LayoutCacheRow,
} from './types';

// ════════════════════════════════════════════════════════════════════
// Hit-test return shape
// ════════════════════════════════════════════════════════════════════

/** Per-cell click target returned by the dispatcher. The caller
 *  (SightV7.svelte) uses this list for click detection: walk the
 *  list on click, find the cell whose (cx, cy, hitRadius) contains
 *  the click point, dispatch a drill-in for that cellId.
 *
 *  Coordinates are in CANVAS LOCAL space (the same space the
 *  dispatcher painted into) — caller must apply any zoom/pan
 *  transform if they wrap renderAnchorDomeV7 in one. */
export interface CellHitTestV7 {
	cellId: string;
	cellLabel: string;
	cx: number;
	cy: number;
	hitRadius: number;
	population: number;
	/** True when the cell has at least one note (i.e., the density
	 *  blob was painted at non-zero radius). Empty cells are still
	 *  click targets — clicking an empty cell drills in to show "no
	 *  notes in this category" rather than swallowing the click. */
	hasPopulation: boolean;
}

// ════════════════════════════════════════════════════════════════════
// Dispatcher
// ════════════════════════════════════════════════════════════════════

export interface RenderAnchorV7Options {
	/** Active tradition module — supplies cellRegions() + cellMembership(). */
	tradition: TraditionModuleV7;
	/** Locale for the calendar rim (only consulted when the tradition
	 *  opts into the rim). Defaults to 'en'. */
	locale?: string;
	/** Clear the canvas before painting. Defaults to true. Set false
	 *  when the caller manages clears itself (e.g., for compositing
	 *  multiple layers). */
	clear?: boolean;
	/** Theme-aware chrome palette. When omitted, falls back to the
	 *  v6 dark palette — same MIG-027 contract v6 uses. */
	chromePalette?: ChromePalette;
	/** i18n key → localized string resolver (typically `(k) => $t(k)`).
	 *  Used for stratum labels + cell labels. When absent, labels
	 *  render as their i18n keys (defensive fallback — visibly wrong
	 *  but not crashing; callers SHOULD pass this). */
	labelize?: (key: string) => string;
	/** Cell id currently hovered (for hover ring). Null when nothing
	 *  is hovered. */
	hoveredCellId?: string | null;
	/** Cell id currently in drill-in (for selection ring). Null when
	 *  no cell is drilled-in. P7 wires this; P3 leaves it null. */
	selectedCellId?: string | null;
}

export function renderAnchorDomeV7(
	ctx: CanvasRenderingContext2D,
	rows: LayoutCacheRow[],
	width: number,
	height: number,
	options: RenderAnchorV7Options,
): CellHitTestV7[] {
	const {
		tradition,
		locale = 'en',
		clear = true,
		chromePalette = CHROME_PALETTE_DARK_FALLBACK,
		labelize = (key: string) => key,
		hoveredCellId = null,
		selectedCellId = null,
	} = options;

	const layout = computeDomeLayout(width, height);

	// ─── 1. Background ───────────────────────────────────────────────
	// Same identity-transform pattern v6 uses (§A.14 fix-10): clears
	// + bg always cover the full backing store, regardless of any
	// caller-applied zoom/pan.
	if (clear) {
		ctx.save();
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.fillStyle = chromePalette.bg;
		ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
		ctx.restore();
	} else {
		ctx.fillStyle = chromePalette.bg;
		ctx.fillRect(0, 0, width, height);
	}

	// ─── 2. Stratum guide rings (UNIVERSAL) ──────────────────────────
	// The 5 maturity-band boundaries painted as faint strokes. Same
	// geometry helper v6 uses (stratumBandBoundaries) so the rings
	// align exactly with v6 — important for Boss-test visual diffing
	// across the v6 ↔ v7 ship gate.
	ctx.strokeStyle = chromePalette.strataRing;
	ctx.lineWidth = 0.9;
	for (const r of stratumBandBoundaries(layout.radius)) {
		ctx.beginPath();
		ctx.arc(layout.centerX, layout.centerY, r, 0, Math.PI * 2);
		ctx.stroke();
	}

	// ─── 3. Stratum labels (UNIVERSAL) ───────────────────────────────
	// Italic labels along the +y axis (above center). Same fallback
	// chain v6 uses: try labelize(STRATUM_LABEL_KEYS[band]) first; if
	// it returns the key unchanged (i18n miss), fall back to the
	// English STRATUM_LABELS literal.
	ctx.fillStyle = chromePalette.stratumLabel;
	ctx.font = 'italic 9px Inter, system-ui, sans-serif';
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	for (const band of STRATUM_BANDS) {
		const r = radiusForStratum(band, layout.radius);
		const key = STRATUM_LABEL_KEYS[band];
		const resolved = labelize(key);
		const text = resolved === key ? STRATUM_LABELS[band] : resolved;
		ctx.fillText(text, layout.centerX, layout.centerY - r);
	}

	// ─── 4. Calendar rim (CONDITIONAL — Time Dome only) ──────────────
	// Per the Form-Aligns-To-Purpose redesign: time is the Time Dome's
	// grammar, not a universal chrome. Categorical traditions (every
	// non-Time tradition) opt OUT by returning false from
	// `showCalendarRim`. The masādir P3 sample case returns false, so
	// this layer is a no-op for the smoke test; it's wired here in P3
	// so the Time Dome (P6) plugs in without dispatcher edits.
	if (tradition.showCalendarRim) {
		ctx.fillStyle = chromePalette.calendarRimText;
		ctx.font = '10px Inter, system-ui, sans-serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		for (const m of calendarRimMonths(layout.radius, locale, 18)) {
			ctx.fillText(m.label, layout.centerX + m.x, layout.centerY + m.y);
		}
	}

	// ─── 5. Per-cell density blobs (THE V7 ESSENCE) ──────────────────
	const traditionLayout = {
		centerX: layout.centerX,
		centerY: layout.centerY,
		radius: layout.radius,
	};
	const cells: CellGeometry[] = tradition.cellRegions(traditionLayout);

	// Count notes per cell via the tradition's membership predicate.
	// Build the population list in the SAME order as `cells` so the
	// downstream density-scale + per-cell paint can iterate in lockstep.
	const populations: number[] = new Array(cells.length).fill(0);
	const cellIdToIndex = new Map<string, number>();
	cells.forEach((cell, i) => cellIdToIndex.set(cell.id, i));
	for (const row of rows) {
		const cellId = tradition.cellMembership(row);
		if (cellId === null) continue; // strict-opt-in: doesn't land anywhere
		const idx = cellIdToIndex.get(cellId);
		if (idx === undefined) continue; // membership returned an id with no geometry
		populations[idx] += 1;
	}

	// Universe-wide density scale so all cells share one magnitude →
	// radius mapping. (Per-cell scales would make magnitudes
	// incomparable across cells — the whole point of the density
	// view is the at-a-glance comparison.)
	const scale = computeDensityScale(populations);

	// Paint each cell's density blob.
	const hitTests: CellHitTestV7[] = [];
	cells.forEach((cell, i) => {
		const population = populations[i];
		const label = labelize(cell.label);
		// labelize-returns-the-key sentinel: same fallback as the v6
		// renderer uses — if i18n missed, render the key literally so
		// the failure is visible instead of crashing.
		const displayLabel = label === cell.label ? cell.label : label;
		const density: CellDensity = cellDensity(cell.id, displayLabel, population, scale);

		hitTests.push({
			cellId: cell.id,
			cellLabel: displayLabel,
			cx: cell.cx,
			cy: cell.cy,
			hitRadius: cell.hitRadius,
			population,
			hasPopulation: population > 0,
		});

		// Blob itself: filled circle at the density-scaled radius.
		// Uses the chrome's neutral starFill so the blob inherits
		// theme (cream-on-dark, dark-on-light).
		if (density.radiusPx > 0) {
			ctx.globalAlpha = density.opacity;
			ctx.fillStyle = chromePalette.starFill;
			ctx.beginPath();
			ctx.arc(cell.cx, cell.cy, density.radiusPx, 0, Math.PI * 2);
			ctx.fill();
			ctx.globalAlpha = 1;
		}

		// ─── 6. Per-cell label (drawn next to the blob) ──────────
		// Sits at the cell center BELOW the blob — far enough out
		// that the largest blob (max 48px radius) doesn't cover the
		// label text. Same italic stratum-label font for visual
		// consistency with the universal chrome.
		ctx.fillStyle = chromePalette.stratumLabel;
		ctx.font = '11px Inter, system-ui, sans-serif';
		ctx.textAlign = 'center';
		ctx.textBaseline = 'top';
		const labelOffset = Math.max(density.radiusPx, 12) + 6;
		ctx.fillText(displayLabel, cell.cx, cell.cy + labelOffset);

		// Population subtitle below the label — useful at-a-glance.
		// Hidden when 0 (cells with no notes show the label alone).
		if (population > 0) {
			ctx.fillStyle = chromePalette.subtitleText;
			ctx.font = '10px Inter, system-ui, sans-serif';
			ctx.fillText(
				String(population),
				cell.cx,
				cell.cy + labelOffset + 14,
			);
		}
	});

	// ─── 7. Hover + selection rings ──────────────────────────────────
	// Drawn last so they sit on top of every other layer. Hover ring
	// is a thin highlight; selection ring (P7) is thicker + persistent.
	if (hoveredCellId !== null) {
		const cell = cells.find((c) => c.id === hoveredCellId);
		if (cell) {
			ctx.strokeStyle = chromePalette.highlightedRing;
			ctx.lineWidth = 1.5;
			ctx.beginPath();
			ctx.arc(cell.cx, cell.cy, cell.hitRadius, 0, Math.PI * 2);
			ctx.stroke();
		}
	}
	if (selectedCellId !== null) {
		const cell = cells.find((c) => c.id === selectedCellId);
		if (cell) {
			ctx.strokeStyle = chromePalette.highlightedRing;
			ctx.lineWidth = 2.5;
			ctx.beginPath();
			ctx.arc(cell.cx, cell.cy, cell.hitRadius, 0, Math.PI * 2);
			ctx.stroke();
		}
	}

	return hitTests;
}

// ════════════════════════════════════════════════════════════════════
// Hit-test helper
// ════════════════════════════════════════════════════════════════════

/** Click-point → cell-id resolver. Walks the hit-test list returned
 *  by `renderAnchorDomeV7` and returns the id of the cell whose
 *  bounding circle contains (px, py), or null if the click missed
 *  every cell. Distance-based — picks the closest hit when multiple
 *  cells overlap (shouldn't happen with well-spaced cell centers but
 *  the tie-break keeps the behavior deterministic). */
export function cellAtPoint(
	hitTests: CellHitTestV7[],
	px: number,
	py: number,
): CellHitTestV7 | null {
	let best: { cell: CellHitTestV7; dist: number } | null = null;
	for (const cell of hitTests) {
		const dx = px - cell.cx;
		const dy = py - cell.cy;
		const dist = Math.sqrt(dx * dx + dy * dy);
		if (dist <= cell.hitRadius) {
			if (best === null || dist < best.dist) {
				best = { cell, dist };
			}
		}
	}
	return best?.cell ?? null;
}

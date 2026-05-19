/**
 * MIG-036 P2 (2026-05-19) — Sight v7 density primitive.
 *
 * Per the Form-Aligns-To-Purpose rule + Architect §4: at universe
 * view, each categorical cell of a tradition renders as ONE visual
 * element whose magnitude encodes the cell's population. NO within-
 * cell positions, NO hash jitter, NO filler.
 *
 * This module is pure-functional: takes a count + canvas geometry,
 * returns the visual properties (radius, opacity, color) to draw.
 * The actual `ctx.arc()` / `ctx.fill()` calls happen in the per-
 * shape renderers (P4–P5).
 *
 * Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md §4
 */

// ════════════════════════════════════════════════════════════════════
// Density encoding
// ════════════════════════════════════════════════════════════════════

/** Universe-wide context the density encoding needs to scale
 *  consistently across all cells. Computed once per render pass and
 *  passed to each per-cell call so all density blobs share the same
 *  magnitude → radius mapping. */
export interface DensityScale {
	/** Maximum cell population in the current render. Used as the
	 *  upper anchor of the log scale; the cell with this count
	 *  reaches `maxRadiusPx`. */
	maxPopulation: number;
	/** Minimum on-screen radius for a non-zero cell (the "floor" so
	 *  cells with very small populations stay visible). */
	minRadiusPx: number;
	/** Maximum on-screen radius for the most-populated cell. Capped
	 *  to prevent the largest cell from dominating the dome. */
	maxRadiusPx: number;
}

/** Compute a DensityScale that fits the universe's range. Uses the
 *  population list across ALL cells of the active tradition so the
 *  scale is consistent within a render pass (not per-cell). */
export function computeDensityScale(
	populations: number[],
	options: { minRadiusPx?: number; maxRadiusPx?: number } = {},
): DensityScale {
	const minRadiusPx = options.minRadiusPx ?? 6;
	const maxRadiusPx = options.maxRadiusPx ?? 48;
	let max = 0;
	for (const p of populations) {
		if (p > max) max = p;
	}
	return {
		maxPopulation: max,
		minRadiusPx,
		maxRadiusPx,
	};
}

/** Map a single cell's population to its on-screen radius in px.
 *  Log-scaled so cells of widely different populations stay legible:
 *  a cell with 50 notes is bigger than one with 10, but not 5× bigger.
 *  A cell with population=0 returns 0 (don't render). */
export function densityRadius(population: number, scale: DensityScale): number {
	if (population <= 0) return 0;
	if (scale.maxPopulation <= 0) return scale.minRadiusPx;
	// log(p+1) keeps the curve well-defined at p=1 and shifts the
	// origin so p=0 → 0 (handled by the early return above; this
	// guarantees p>=1 → positive radius).
	const logCurrent = Math.log(population + 1);
	const logMax = Math.log(scale.maxPopulation + 1);
	const t = logCurrent / logMax; // ∈ (0, 1]
	return scale.minRadiusPx + t * (scale.maxRadiusPx - scale.minRadiusPx);
}

/** Compute an opacity for the density blob. Cells with population at
 *  the universe maximum render at full opacity; smaller cells fade
 *  slightly so the relative magnitudes read pre-attentively even when
 *  radius differences are subtle. */
export function densityOpacity(population: number, scale: DensityScale): number {
	if (population <= 0) return 0;
	if (scale.maxPopulation <= 0) return 0.6;
	const t = Math.log(population + 1) / Math.log(scale.maxPopulation + 1);
	// Map t ∈ (0, 1] → opacity ∈ [0.5, 1.0]. Keeps even small cells
	// visibly opaque while reserving full alpha for the maximum.
	return 0.5 + t * 0.5;
}

// ════════════════════════════════════════════════════════════════════
// Per-cell density properties
// ════════════════════════════════════════════════════════════════════

/** Result of evaluating density for one cell. Per-shape renderers
 *  in P4–P5 consume this + the cell's center position to draw the
 *  blob. */
export interface CellDensity {
	/** Cell's logical id (e.g., 'sunnah', 'pratyaksha', 'peshat',
	 *  'firstness'). Stable across renders; used by drill-in to
	 *  identify which cell was clicked. */
	cellId: string;
	/** Display label for the cell (already-translated via $t). Shown
	 *  in tooltips on hover. */
	cellLabel: string;
	/** Note count in this cell. */
	population: number;
	/** Density-scaled radius in screen pixels (0 if empty). */
	radiusPx: number;
	/** Density-scaled opacity ∈ [0, 1] (0 if empty). */
	opacity: number;
}

/** Build a CellDensity from a population + scale. Per-shape renderers
 *  call this once per cell during their draw loop. */
export function cellDensity(
	cellId: string,
	cellLabel: string,
	population: number,
	scale: DensityScale,
): CellDensity {
	return {
		cellId,
		cellLabel,
		population,
		radiusPx: densityRadius(population, scale),
		opacity: densityOpacity(population, scale),
	};
}

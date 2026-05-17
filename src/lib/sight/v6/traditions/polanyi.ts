/**
 * MIG-026 Phase γ — Polanyi tradition (tacit/explicit gradient).
 *
 * Per Concept Paper §4.1.4 (and the original MIG-025 §C.5 placeholder
 * that was never implemented):
 *   Geometry         No spatial redistribution — stars stay at their
 *                    default Aristotelian positions. Instead, a radial
 *                    OPACITY gradient is overlaid across the dome:
 *                    center = high fog (low star visibility) = the
 *                    tacit pole; edge = no fog (high star visibility)
 *                    = the explicit pole. The metaphor: knowledge
 *                    near the tacit core is "acknowledged but
 *                    inarticulable" (Polanyi 1966); knowledge near the
 *                    explicit periphery is "what you can articulate".
 *   Cultural framing Modern Western pluralist epistemology; Michael
 *                    Polanyi's *The Tacit Dimension* (1966) introduced
 *                    the tacit/explicit polarity as a continuous
 *                    spectrum, not a binary. The gradient encoding
 *                    visualizes this continuity.
 *   Citation         Polanyi, *The Tacit Dimension* (1966), ch. 1
 *                    ("Tacit Knowing"); *Personal Knowledge* (1958),
 *                    Part III.
 *   v4.1 polish      Per-note tacit-axis frontmatter (`tacit_proximity:
 *                    0.0 .. 1.0`) for finer-grained placement; once
 *                    LayoutCacheRow gains the field, individual star
 *                    alpha can be modulated by per-note value rather
 *                    than by radial position alone.
 *
 * Note on remap: per the Plan, `remapStarPosition = identity`. Polanyi
 * does NOT redistribute stars — it leaves them at default Aristotelian
 * positions and modulates their VISIBILITY via the fog overlay drawn
 * by `drawGradientFog` in anchor.ts. The metaphor preserves Aristotelian's
 * stratum × time encoding underneath; the gradient is a SECOND axis
 * (tacit ↔ explicit) layered on top.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1.4
 * Plan:          lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-PLAN.md §5 (Phase γ)
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule, GradientSpec } from '../types';

export const polanyi: TraditionModule = {
	id: 'polanyi',
	name: 'Polanyi',
	// MIG-026 Phase α/γ — shape discriminator. Polanyi is a `gradient`
	// shape: no spatial redistribution, just an opacity overlay.
	shape: 'gradient',

	remapStarPosition: (
		_row: LayoutCacheRow,
		defaultPos: { x: number; y: number },
		_layout: TraditionLayout,
	) => {
		// Identity per Plan: Polanyi preserves default Aristotelian
		// positions and modulates visibility via drawGradientFog overlay
		// (which paints AFTER stars are drawn — see anchor.ts step 6.5).
		return defaultPos;
	},

	gradientSpec: (_layout: TraditionLayout): GradientSpec => {
		// Per the GradientSpec docstring (types.ts):
		//   centerOpacity ≈ 0.14–0.18 (tacit core: stars there read as
		//     "acknowledged but inarticulable")
		//   edgeOpacity   ≈ 0.85–0.95 (explicit periphery: clearly readable)
		// Picked 0.18 / 0.95 as a balanced first cut — leaves enough
		// signal at center for hover/click to still hit, but unmistakably
		// fades stars compared to the periphery.
		return {
			centerOpacity: 0.18,
			edgeOpacity: 0.95,
			centerLabel: 'tacit',
			edgeLabel: 'explicit',
		};
	},
};

/**
 * MIG-025 §C.2 — Aristotelian register (default, identity remap).
 *
 * Per Concept Paper §4.1.1:
 *   Geometry         radial = stratum (Foundation → Edge of Knowing),
 *                    angular = time. Identical to the default Sight
 *                    grammar; this module exists to make the implicit
 *                    Western-classical frame EXPLICIT rather than
 *                    smuggled in as the unnamed default.
 *   Cultural framing Western-classical; knowledge as maturity gradient.
 *   Citation         Aristotle, *Posterior Analytics*; Lloyd, *The
 *                    Ambitions of Curiosity*.
 *   Why default      Makes the implicit Western frame visible to the
 *                    user as a choice, not as the unmarked baseline.
 *
 * Identity remap: returns the defaultPos unchanged. The default
 * position already IS the Aristotelian position (anchor.ts
 * computeStarPositions implements stratum × time per §4.1.1), so
 * this module's job is purely to NAME the geometry and provide
 * the chip's selectable entry — there is no transformation to apply.
 *
 * No sectorDividers: the 5 concentric stratum bands ARE the visual
 * structure of the Aristotelian frame, and those are drawn by
 * dome.ts as part of the always-on chrome, not by register code.
 *
 * Concept Paper: docs/Constellation-Sight-Concept-Paper-v4.0.md §4.1.1
 * Plan:          lab/reports/MIG-025-SIGHT-V6-PLAN.md §C.2
 */
import type { LayoutCacheRow, RegisterLayout, RegisterModule } from '../types';

export const aristotelian: RegisterModule = {
	id: 'aristotelian',
	name: 'Aristotelian',
	remapStarPosition: (_row: LayoutCacheRow, defaultPos, _layout: RegisterLayout) => defaultPos,
	// sectorDividers intentionally omitted (no register-drawn dividers).
};

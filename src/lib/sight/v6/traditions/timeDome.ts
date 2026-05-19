/**
 * MIG-037 P1 (2026-05-19) — Time Dome tradition.
 *
 * The proper home for the time-aware view that v6 had implicitly
 * inherited inside Aristotelian. Per Eisa's direction (post-MIG-036
 * P3 pivot): "If Aristotelian is just to display the time, then
 * why are we calling it this? Instead, the Traditions (including
 * Aristotelian) should look at the knowledge-cognition lens based
 * on their design. If I want to display time, I will call this
 * 'the Time Dome'."
 *
 * Phase 1 ships Time Dome as an identity-remap clone of current
 * Aristotelian behavior (stratum × time, calendar rim around the
 * outer edge). Visually identical to Aristotelian today.
 *
 * The architectural split happens in Phase 2 + 3:
 *   - Phase 2: calendar rim becomes opt-in via TraditionModule's
 *              new `showCalendarRim?: boolean`. Time Dome sets
 *              true; Aristotelian + the other 23 traditions
 *              default to false. Aristotelian's remap pivots
 *              to pure-radial (collapses stars to stratum-band
 *              centers; no angular spread) so Aristotelian becomes
 *              the maturity-ladder view without time semantics.
 *   - Phase 3: density-blob render mode opt-in for categorical
 *              wedge traditions (masādir, pramāṇa, etc.). Time
 *              Dome stays per-star because both axes carry data
 *              (stratum × time both meaningful).
 *
 * Why identity remap: the default `computeStarPositions` in
 * anchor.ts already computes the time-aware default
 * (angle = createdMonth wedge midpoint + jitter, radial =
 * stratum band center + jitter). Time Dome's job is to NAME this
 * grammar as a first-class view + provide the dropdown entry;
 * there is no transformation to apply on top of the default.
 *
 * Identical structure to aristotelian.ts. The difference between
 * the two surfaces in Phase 2 when:
 *   - Aristotelian gets a non-identity remap (pure-radial)
 *   - Time Dome stays identity (keeps time-angle)
 *   - Aristotelian sets showCalendarRim: false
 *   - Time Dome sets showCalendarRim: true
 *
 * No sectorDividers: same reasoning as Aristotelian — the 12
 * calendar rim labels themselves visually demark the month
 * wedges; adding strokes would be redundant chrome.
 */
import type { LayoutCacheRow, TraditionLayout, TraditionModule } from '../types';

export const timeDome: TraditionModule = {
	id: 'time-dome',
	name: 'Time Dome',
	// Sectoral by convention (12 month wedges + 5 stratum bands).
	// Same shape Aristotelian uses; the shape discriminator drives
	// the renderer dispatch in anchor.ts, but identity-remap
	// traditions take the default path either way.
	shape: 'sectoral',
	remapStarPosition: (_row: LayoutCacheRow, defaultPos, _layout: TraditionLayout) => defaultPos,
	// sectorDividers intentionally omitted — the calendar rim labels
	// at the outer edge already mark the 12 month wedges visually;
	// adding stroke dividers would be redundant chrome.
};

// @ts-nocheck
// ↑ vitest/playwright runners deferred to §D.4 per tests/sight-v6/README.md.
// This file is harness-ready source; type-check skipped here so svelte-check
// doesn't fail on absent vitest types. Wires into the type tree in §D.4
// when the runner devDep lands.

/**
 * MIG-025 §A.13 — Sight v6 render-budget performance test (vitest).
 *
 * Per Concept Paper v4.0 §8.3 + §11 invariants 2 and 3:
 *   - default render ≤100 ms on 1,000-note fixture
 *   - cross-filter response ≤16 ms on 7,636-note × 5 views
 *
 * Status: harness-ready. The vitest runner is deferred to §D.4 (CI
 * hardening phase) because adding it as a devDependency is a
 * project-level choice beyond MIG-025's scope. See README.md.
 *
 * The test exercises the PURE-FUNCTION render path (computeDomeLayout
 * + computeStarPositions) which is the dominant CPU cost in the
 * render pipeline. The Canvas drawing operations are GPU-bounded and
 * not measurable from JS in a stable way; their budget is verified
 * via Boss-test perception at the §A.14 ship gate.
 *
 * To run once vitest lands:
 *   npm run test:sight-v6:perf
 */

import { describe, it, expect } from 'vitest';
import {
	computeStarPositions,
	computeDomeLayout,
} from '../../src/lib/sight/v6/anchor';
import {
	applyFilters,
	emptyFilters,
	toggleFilter,
	computeFacetCounts,
} from '../../src/lib/sight/v6/facets';
import type { LayoutCacheRow } from '../../src/lib/sight/v6/types';

// ── Fixture builders ──────────────────────────────────────────────

function syntheticUniverse(n: number): LayoutCacheRow[] {
	const out: LayoutCacheRow[] = [];
	const libraries = ['Research', 'Projects', 'Personal', 'Reading', 'Reference'];
	const stages = ['established', 'fresh', 'growing', 'at-risk', 'dormant'];
	for (let i = 0; i < n; i++) {
		out.push({
			notePath: `note-${i}.md`,
			stratum: (i % 8) + 1,
			maturity: 'evergreen',
			confidenceAlpha: 0.45 + ((i % 4) * 0.18), // spreads 0.45 → 0.99
			contested: i % 23 === 0,
			libraryName: libraries[i % libraries.length],
			folderPath: `folder-${i % 12}`,
			createdMonth: i % 12,
			sourcesPrimary: i % 3 === 0 ? 'https://example.com' : null,
			stage: stages[i % stages.length],
			actsPrimary: i % 7 === 0 ? 'Synthesis' : null,
			dominantLinkType: 'supports',
			computedAt: 0,
			linkInCount: i % 11,
			linkOutCount: i % 13,
			frontmatterKeyCount: i % 5,
			bodyChars: 100 + (i % 200),
		});
	}
	return out;
}

// ── Tests ─────────────────────────────────────────────────────────

describe('Sight v6 render-budget', () => {
	it('computeStarPositions on 1,000 notes completes in ≤100 ms', () => {
		const rows = syntheticUniverse(1_000);
		const layout = computeDomeLayout(1200, 800);

		const t0 = performance.now();
		const stars = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		const elapsed = performance.now() - t0;

		expect(stars.length).toBe(1_000);
		expect(elapsed).toBeLessThan(100);
	});

	it('computeStarPositions on 7,636 notes completes in ≤200 ms (initial-load budget)', () => {
		// 7,636 = Eisa's actual Universe size. Initial layout has a
		// looser budget than cross-filter (200 ms vs 16 ms) because it
		// runs once on render-ready, not per gesture.
		const rows = syntheticUniverse(7_636);
		const layout = computeDomeLayout(1200, 800);

		const t0 = performance.now();
		const stars = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		const elapsed = performance.now() - t0;

		expect(stars.length).toBe(7_636);
		expect(elapsed).toBeLessThan(200);
	});

	it('cross-filter cycle on 7,636 notes completes in ≤16 ms', () => {
		// Concept Paper §11 invariant 3: cross-filter ≤16 ms on
		// 7,636-note × 5 views. The §A path tests the 1-view (anchor
		// only) case; §B tests will extend to 5 views (anchor + 4 minis).
		const rows = syntheticUniverse(7_636);
		const filters = toggleFilter(emptyFilters(), 'library', 'Research');

		const t0 = performance.now();
		const filtered = applyFilters(rows, filters);
		const elapsed = performance.now() - t0;

		expect(filtered.length).toBeGreaterThan(0);
		expect(filtered.length).toBeLessThan(rows.length);
		expect(elapsed).toBeLessThan(16);
	});

	it('Hearst facet-count rebalancing on 7,636 notes completes in ≤32 ms', () => {
		// 6 facets × Hearst preview (each excluding self) = 6 filter
		// passes. Budget is 2× the cross-filter budget per facet.
		const rows = syntheticUniverse(7_636);
		const filters = toggleFilter(emptyFilters(), 'stratum', 'foundation');

		const t0 = performance.now();
		const facets = computeFacetCounts(rows, filters);
		const elapsed = performance.now() - t0;

		expect(facets.length).toBe(6);
		expect(facets[0].id).toBe('folder'); // §11 invariant 8 — Folder TOP
		expect(elapsed).toBeLessThan(32);
	});
});

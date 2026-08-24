// MIG-030 (2026-05-18): vitest runner now installed. @ts-nocheck removed.
// Run with: npm run test:sight-v6:perf

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
			// MIG-029 §ν.1: tradition-kind frontmatter fields. Synthetic
			// universe leaves them null so all stars default to the
			// per-tradition fallback bucket — preserves pre-MIG-029
			// test expectations.
			masadirSource: null,
			pramanaKind: null,
			burhanKind: null,
			pardesLevel: null,
			peirceCategory: null,
			habermasInterest: null,
			mencianSprout: null,
			mohistZone: null,
			songnihakCell: null,
		});
	}
	return out;
}

// ── Tests ─────────────────────────────────────────────────────────

/**
 * PJ-380 — time the BEST of several runs, not one cold run.
 *
 * These budgets guard against an algorithmic regression. Timed once, cold, they were also
 * measuring JIT warm-up and whatever GC happened to land in that single call — so on
 * 2026-08-24 the 32 ms case went red three times in five full-suite runs, purely because
 * vitest was running other files in parallel and, on another occasion, because a
 * `cargo build --release` was saturating the CPU. Nothing in the diff touched Sight v6.
 *
 * A suite that can go red for reasons unrelated to the code teaches everyone to re-run red
 * tests instead of reading them, which is exactly how a real regression gets waved through.
 *
 * The minimum across runs is the standard estimator for this: scheduler noise and GC can only
 * ever ADD time, so the fastest run is the closest thing to the true cost — while a genuine
 * regression slows every run, including the fastest. The budget keeps its meaning; only the
 * noise is removed.
 */
function fastestMs(fn: () => unknown, runs = 5): number {
	let best = Infinity;
	for (let i = 0; i < runs; i++) {
		const t0 = performance.now();
		fn();
		best = Math.min(best, performance.now() - t0);
	}
	return best;
}

describe('Sight v6 render-budget', () => {
	it('computeStarPositions on 1,000 notes completes in ≤100 ms', () => {
		const rows = syntheticUniverse(1_000);
		const layout = computeDomeLayout(1200, 800);

		let stars!: ReturnType<typeof computeStarPositions>;
		const elapsed = fastestMs(() => {
			stars = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		});

		expect(stars.length).toBe(1_000);
		expect(elapsed).toBeLessThan(100);
	});

	it('computeStarPositions on 7,636 notes completes in ≤200 ms (initial-load budget)', () => {
		// 7,636 = Eisa's actual Universe size. Initial layout has a
		// looser budget than cross-filter (200 ms vs 16 ms) because it
		// runs once on render-ready, not per gesture.
		const rows = syntheticUniverse(7_636);
		const layout = computeDomeLayout(1200, 800);

		let stars!: ReturnType<typeof computeStarPositions>;
		const elapsed = fastestMs(() => {
			stars = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		});

		expect(stars.length).toBe(7_636);
		expect(elapsed).toBeLessThan(200);
	});

	it('cross-filter cycle on 7,636 notes completes in ≤16 ms', () => {
		// Concept Paper §11 invariant 3: cross-filter ≤16 ms on
		// 7,636-note × 5 views. The §A path tests the 1-view (anchor
		// only) case; §B tests will extend to 5 views (anchor + 4 minis).
		const rows = syntheticUniverse(7_636);
		const filters = toggleFilter(emptyFilters(), 'library', 'Research');

		let filtered!: ReturnType<typeof applyFilters>;
		const elapsed = fastestMs(() => {
			filtered = applyFilters(rows, filters);
		});

		expect(filtered.length).toBeGreaterThan(0);
		expect(filtered.length).toBeLessThan(rows.length);
		expect(elapsed).toBeLessThan(16);
	});

	it('Hearst facet-count rebalancing on 7,636 notes completes in ≤32 ms', () => {
		// 6 facets × Hearst preview (each excluding self) = 6 filter
		// passes. Budget is 2× the cross-filter budget per facet.
		const rows = syntheticUniverse(7_636);
		const filters = toggleFilter(emptyFilters(), 'stratum', 'foundation');

		let facets!: ReturnType<typeof computeFacetCounts>;
		const elapsed = fastestMs(() => {
			facets = computeFacetCounts(rows, filters);
		});

		expect(facets.length).toBe(6);
		expect(facets[0].id).toBe('folder'); // §11 invariant 8 — Folder TOP
		expect(elapsed).toBeLessThan(32);
	});
});

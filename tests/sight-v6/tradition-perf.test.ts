/**
 * MIG-030 (Plan §14.2) — Per-tradition switch perf test.
 *
 * Concept Paper v4.1 §11.3 — tradition switch on a 7,636-note universe
 * (Eisa's actual library size) must complete computeStarPositions in
 * ≤16ms so the chip-click → repaint cycle reads as instant.
 *
 * This test iterates all 24 curated baseline traditions and asserts
 * each remap completes within the budget.
 *
 * Note: this measures the JS computeStarPositions cost only — the
 * Canvas2D draw step that follows is GPU-bounded and not measurable
 * from JS in a stable way. Its budget is verified via Boss-test
 * perception at MIG-026 Phase μ ship gate.
 */

import { describe, it, expect } from 'vitest';
import { computeStarPositions, computeDomeLayout } from '../../src/lib/sight/v6/anchor';
import { allTraditions } from '../../src/lib/sight/v6/traditions';
import type { LayoutCacheRow } from '../../src/lib/sight/v6/types';

function syntheticUniverse(n: number): LayoutCacheRow[] {
	const out: LayoutCacheRow[] = [];
	const libraries = ['Research', 'Projects', 'Personal', 'Reading', 'Reference'];
	const stages = ['established', 'fresh', 'growing', 'at-risk', 'dormant'];
	for (let i = 0; i < n; i++) {
		out.push({
			notePath: `note-${i}.md`,
			stratum: (i % 8) + 1,
			maturity: 'evergreen',
			confidenceAlpha: 0.45 + ((i % 4) * 0.18),
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

describe('Sight v6 per-tradition switch perf (Plan §14.2)', () => {
	const rows = syntheticUniverse(7_636); // Eisa's library size
	const layout = computeDomeLayout(1200, 800);
	const traditions = allTraditions();

	// Warm-up — JIT compilation gets one free pass before measurement.
	it('warm-up pass', () => {
		for (const t of traditions) {
			computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius, t);
		}
		expect(true).toBe(true);
	});

	it.each(traditions.map((t) => [t.id, t]))(
		'tradition %s: switch completes in ≤16ms on 7,636 notes',
		(_id, tradition) => {
			// Single measurement — chip-click triggers a single recompute,
			// so the budget is per-call not amortized.
			const t0 = performance.now();
			const stars = computeStarPositions(
				rows,
				layout.centerX,
				layout.centerY,
				layout.radius,
				tradition,
			);
			const elapsed = performance.now() - t0;

			expect(stars.length).toBe(7_636);
			expect(elapsed).toBeLessThan(16);
		},
	);

	it('full cycle through all 24 traditions completes in ≤400ms (16ms × 24 + overhead)', () => {
		const t0 = performance.now();
		for (const t of traditions) {
			computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius, t);
		}
		const elapsed = performance.now() - t0;
		// Headroom over 16×24=384; allow 400 for orchestration overhead.
		expect(elapsed).toBeLessThan(400);
	});
});

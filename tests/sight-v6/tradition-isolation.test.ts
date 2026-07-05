/**
 * MIG-030 (Plan §14.1) — Channel-isolation test.
 *
 * Concept Paper v4.1 §11 invariant 6 — mini-domes never see the active
 * tradition; they always render `starsDefault` (the tradition-agnostic
 * computeStarPositions result). The anchor dome alone receives the
 * active-tradition remap.
 *
 * This test exercises ALL 24 curated baseline traditions and asserts:
 *
 *   1. `starsDefault = computeStarPositions(rows, …)` (no tradition arg)
 *      always returns the SAME positions regardless of which tradition
 *      a sibling call may have used in the same paint cycle.
 *   2. `starsTradition = computeStarPositions(rows, …, tradition)` for
 *      a non-identity tradition produces DIFFERENT positions from
 *      `starsDefault` (otherwise the tradition isn't actually remapping
 *      and the chip is a lie).
 *   3. Aristotelian (the identity tradition) DOES produce the same
 *      positions as `starsDefault`.
 *   4. Star count is preserved across all 24 traditions × default.
 */

import { describe, it, expect } from 'vitest';
import { computeStarPositions, computeDomeLayout } from '../../src/lib/sight/v6/anchor';
import { allTraditions, getTraditionById } from '../../src/lib/sight/v6/traditions';
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
			// MIG-029 §ν.1: tradition-kind frontmatter fields null →
			// per-tradition default bucket (preserves pre-MIG-029 behavior).
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

function fingerprintXY(stars: { x: number; y: number; row: { notePath: string } }[]): string {
	// Hash-equivalent: ordered concatenation of (path, x, y) at 2-decimal
	// precision. Suitable for stable-equality checks across runs without
	// sensitivity to floating-point noise below pixel resolution.
	return stars
		.map((s) => `${s.row.notePath}:${s.x.toFixed(2)},${s.y.toFixed(2)}`)
		.join('|');
}

describe('Sight v6 channel isolation (Plan §14.1)', () => {
	const rows = syntheticUniverse(500);
	const layout = computeDomeLayout(1200, 800);
	const traditions = allTraditions();

	it('registers all 24 curated baseline traditions', () => {
		expect(traditions.length).toBeGreaterThanOrEqual(24);
	});

	it('starsDefault is stable — no tradition argument always produces same output', () => {
		const a = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		const b = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		const c = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius, null);
		expect(fingerprintXY(a)).toBe(fingerprintXY(b));
		expect(fingerprintXY(a)).toBe(fingerprintXY(c));
	});

	it('Aristotelian is the identity tradition — same output as no-tradition', () => {
		const aristotelian = getTraditionById('aristotelian');
		expect(aristotelian).not.toBeNull();
		const def = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		const arist = computeStarPositions(
			rows,
			layout.centerX,
			layout.centerY,
			layout.radius,
			aristotelian,
		);
		expect(fingerprintXY(def)).toBe(fingerprintXY(arist));
	});

	it.each(traditions.map((t) => [t.id, t]))(
		'tradition %s: star count preserved + (if non-Aristotelian) positions differ from default',
		(id, tradition) => {
			const def = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
			const remapped = computeStarPositions(
				rows,
				layout.centerX,
				layout.centerY,
				layout.radius,
				tradition,
			);

			// Star count preserved across every tradition switch
			expect(remapped.length).toBe(def.length);

			// Non-identity traditions actually move stars (otherwise the
			// tradition's geometric vocabulary is a lie). The known
			// identity-remap shapes:
			//   - aristotelian — identity today (pivots to pure-radial in the
			//     MIG-037 Phase 2 split);
			//   - polanyi — modulates opacity via gradient, not position;
			//   - time-dome (MIG-037 P1) — NAMES the time-aware default grammar
			//     as a first-class view; remapStarPosition returns defaultPos by
			//     design (there is no transform on top of the default).
			const identityIds = new Set(['aristotelian', 'polanyi', 'time-dome']);
			if (!identityIds.has(id as string)) {
				expect(fingerprintXY(remapped)).not.toBe(fingerprintXY(def));
			}
		},
	);

	it('iterating all traditions does not mutate starsDefault between switches', () => {
		// The B2 dual-mount invariant in operational form: if the user
		// rapid-cycles through every tradition, the mini-domes' source
		// (starsDefault) must remain bit-identical from first call to
		// last. If a tradition's remap mutated shared state, the second
		// call to starsDefault would diverge.
		const before = fingerprintXY(
			computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius),
		);
		for (const t of traditions) {
			computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius, t);
		}
		const after = fingerprintXY(
			computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius),
		);
		expect(after).toBe(before);
	});
});

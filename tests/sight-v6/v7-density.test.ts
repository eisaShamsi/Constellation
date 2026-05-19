/**
 * MIG-036 P2 (2026-05-19) — Sight v7 density primitive unit tests.
 *
 * Located in tests/sight-v6/ because that's where the vitest runner
 * is wired (test:sight-v6 npm script). The directory name is now
 * slightly misleading — both v6 and v7 tests live here. Renaming
 * the directory is a polish item for v7 close-out.
 */

import { describe, it, expect } from 'vitest';
import {
	computeDensityScale,
	densityRadius,
	densityOpacity,
	cellDensity,
} from '../../src/lib/sight/v7/density';

describe('Sight v7 density primitive', () => {
	describe('computeDensityScale', () => {
		it('finds the max across populations', () => {
			const scale = computeDensityScale([10, 50, 5, 200, 30]);
			expect(scale.maxPopulation).toBe(200);
		});

		it('uses default min/max radius when not overridden', () => {
			const scale = computeDensityScale([10]);
			expect(scale.minRadiusPx).toBe(6);
			expect(scale.maxRadiusPx).toBe(48);
		});

		it('respects min/max overrides', () => {
			const scale = computeDensityScale([10], { minRadiusPx: 12, maxRadiusPx: 80 });
			expect(scale.minRadiusPx).toBe(12);
			expect(scale.maxRadiusPx).toBe(80);
		});

		it('handles empty populations', () => {
			const scale = computeDensityScale([]);
			expect(scale.maxPopulation).toBe(0);
		});

		it('handles all-zero populations', () => {
			const scale = computeDensityScale([0, 0, 0]);
			expect(scale.maxPopulation).toBe(0);
		});
	});

	describe('densityRadius', () => {
		it('returns 0 for empty cells', () => {
			const scale = computeDensityScale([100]);
			expect(densityRadius(0, scale)).toBe(0);
		});

		it('returns the max radius for the most-populated cell', () => {
			const scale = computeDensityScale([100]);
			expect(densityRadius(100, scale)).toBe(scale.maxRadiusPx);
		});

		it('returns the min radius for a single-note cell when max is the same', () => {
			const scale = computeDensityScale([1]);
			expect(densityRadius(1, scale)).toBe(scale.maxRadiusPx);
		});

		it('scales monotonically — bigger population → bigger radius', () => {
			const scale = computeDensityScale([1, 10, 100, 1000]);
			const r1 = densityRadius(1, scale);
			const r10 = densityRadius(10, scale);
			const r100 = densityRadius(100, scale);
			const r1000 = densityRadius(1000, scale);
			expect(r1).toBeLessThan(r10);
			expect(r10).toBeLessThan(r100);
			expect(r100).toBeLessThan(r1000);
		});

		it('uses log-scale (not linear) so extreme ratios stay legible', () => {
			// Cell with 1000 notes shouldn't be 1000× bigger than cell with 1.
			const scale = computeDensityScale([1, 1000]);
			const r1 = densityRadius(1, scale);
			const r1000 = densityRadius(1000, scale);
			// Linear would give ratio = 1000; log curve keeps ratio < 10
			// after the min-radius floor.
			expect(r1000 / r1).toBeLessThan(10);
			// And both are within the [min, max] band.
			expect(r1).toBeGreaterThanOrEqual(scale.minRadiusPx);
			expect(r1000).toBeLessThanOrEqual(scale.maxRadiusPx);
		});

		it('non-zero populations always render at least the minimum radius', () => {
			const scale = computeDensityScale([100]);
			expect(densityRadius(1, scale)).toBeGreaterThanOrEqual(scale.minRadiusPx);
		});
	});

	describe('densityOpacity', () => {
		it('returns 0 for empty cells', () => {
			const scale = computeDensityScale([100]);
			expect(densityOpacity(0, scale)).toBe(0);
		});

		it('returns full opacity for the most-populated cell', () => {
			const scale = computeDensityScale([100]);
			expect(densityOpacity(100, scale)).toBe(1);
		});

		it('keeps small cells visible at ≥0.5 opacity', () => {
			const scale = computeDensityScale([1000]);
			expect(densityOpacity(1, scale)).toBeGreaterThanOrEqual(0.5);
		});
	});

	describe('cellDensity (composite)', () => {
		it('packages id + label + magnitude into one struct', () => {
			const scale = computeDensityScale([10, 50, 100]);
			const d = cellDensity('sunnah', 'sunnah · السنة', 50, scale);
			expect(d.cellId).toBe('sunnah');
			expect(d.cellLabel).toBe('sunnah · السنة');
			expect(d.population).toBe(50);
			expect(d.radiusPx).toBeGreaterThan(0);
			expect(d.opacity).toBeGreaterThan(0);
		});

		it('handles empty cells gracefully', () => {
			const scale = computeDensityScale([100]);
			const d = cellDensity('empty-cell', 'empty', 0, scale);
			expect(d.radiusPx).toBe(0);
			expect(d.opacity).toBe(0);
		});
	});
});

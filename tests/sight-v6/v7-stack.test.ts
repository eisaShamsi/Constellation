/**
 * MIG-036 P2 (2026-05-19) — Sight v7 stack primitive unit tests.
 */

import { describe, it, expect } from 'vitest';
import { buildStack, sortStack, filterCell, type StackRow } from '../../src/lib/sight/v7/stack';
import type { LayoutCacheRow } from '../../src/lib/sight/v6/types';

function row(overrides: Partial<LayoutCacheRow>): LayoutCacheRow {
	return {
		notePath: 'note.md',
		stratum: 4,
		maturity: 'evergreen',
		confidenceAlpha: 0.5,
		contested: false,
		libraryName: 'Library',
		folderPath: null,
		createdMonth: 0,
		sourcesPrimary: null,
		stage: null,
		actsPrimary: null,
		dominantLinkType: null,
		computedAt: 0,
		linkInCount: 0,
		linkOutCount: 0,
		frontmatterKeyCount: 0,
		bodyChars: 0,
		masadirSource: null,
		pramanaKind: null,
		burhanKind: null,
		pardesLevel: null,
		peirceCategory: null,
		habermasInterest: null,
		mencianSprout: null,
		mohistZone: null,
		songnihakCell: null,
		...overrides,
	};
}

describe('Sight v7 stack primitive', () => {
	describe('buildStack + sort by stratum', () => {
		it('orders most-foundational first', () => {
			const rows = [
				row({ notePath: 'a.md', stratum: 5 }),
				row({ notePath: 'b.md', stratum: 2 }),
				row({ notePath: 'c.md', stratum: 8 }),
				row({ notePath: 'd.md', stratum: 1 }),
			];
			const stack = buildStack(rows, (r) => r.notePath, 'stratum');
			expect(stack.map((s) => s.notePath)).toEqual([
				'd.md',
				'b.md',
				'a.md',
				'c.md',
			]);
		});

		it('places null stratum at the end (Edge-of-Knowing rim)', () => {
			const rows = [
				row({ notePath: 'null-stratum.md', stratum: null }),
				row({ notePath: 'mid.md', stratum: 4 }),
			];
			const stack = buildStack(rows, (r) => r.notePath, 'stratum');
			expect(stack[0].notePath).toBe('mid.md');
			expect(stack[1].notePath).toBe('null-stratum.md');
		});
	});

	describe('sort by created month', () => {
		it('orders earlier months first', () => {
			const rows: StackRow[] = [
				{ notePath: 'mar.md', title: 'mar', libraryName: null, stratum: 4, confidenceAlpha: null, stage: null, createdMonth: 2 },
				{ notePath: 'jan.md', title: 'jan', libraryName: null, stratum: 4, confidenceAlpha: null, stage: null, createdMonth: 0 },
				{ notePath: 'jul.md', title: 'jul', libraryName: null, stratum: 4, confidenceAlpha: null, stage: null, createdMonth: 6 },
			];
			const sorted = sortStack(rows, 'created');
			expect(sorted.map((r) => r.notePath)).toEqual(['jan.md', 'mar.md', 'jul.md']);
		});
	});

	describe('sort by title', () => {
		it('orders alphabetically', () => {
			const rows: StackRow[] = [
				{ notePath: 'a.md', title: 'Zebra', libraryName: null, stratum: 4, confidenceAlpha: null, stage: null, createdMonth: 0 },
				{ notePath: 'b.md', title: 'Apple', libraryName: null, stratum: 4, confidenceAlpha: null, stage: null, createdMonth: 0 },
				{ notePath: 'c.md', title: 'Mango', libraryName: null, stratum: 4, confidenceAlpha: null, stage: null, createdMonth: 0 },
			];
			const sorted = sortStack(rows, 'title');
			expect(sorted.map((r) => r.title)).toEqual(['Apple', 'Mango', 'Zebra']);
		});
	});

	describe('sort by hash (deterministic shuffle)', () => {
		it('produces stable order between renders', () => {
			const rows: StackRow[] = ['a', 'b', 'c', 'd', 'e'].map((n) => ({
				notePath: `${n}.md`,
				title: n,
				libraryName: null,
				stratum: 4,
				confidenceAlpha: null,
				stage: null,
				createdMonth: 0,
			}));
			const first = sortStack(rows, 'hash').map((r) => r.notePath);
			const second = sortStack(rows, 'hash').map((r) => r.notePath);
			expect(first).toEqual(second);
		});
	});

	describe('filterCell', () => {
		it('keeps only rows matching the predicate', () => {
			const rows = [
				row({ notePath: 'a.md', masadirSource: 'sunnah' }),
				row({ notePath: 'b.md', masadirSource: 'quran' }),
				row({ notePath: 'c.md', masadirSource: 'sunnah' }),
				row({ notePath: 'd.md', masadirSource: null }),
			];
			const sunnah = filterCell(rows, (r) => r.masadirSource === 'sunnah');
			expect(sunnah.length).toBe(2);
			expect(sunnah.map((r) => r.notePath)).toEqual(['a.md', 'c.md']);
		});

		it('returns empty array when no rows match', () => {
			const rows = [row({ masadirSource: 'quran' })];
			const sunnah = filterCell(rows, (r) => r.masadirSource === 'sunnah');
			expect(sunnah).toEqual([]);
		});
	});

	describe('buildStack with custom title resolver', () => {
		it('uses the resolver for display title', () => {
			const rows = [
				row({ notePath: 'note.md' }),
			];
			const stack = buildStack(rows, (r) => `Title for ${r.notePath}`, 'stratum');
			expect(stack[0].title).toBe('Title for note.md');
		});
	});
});

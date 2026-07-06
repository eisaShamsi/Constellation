/**
 * MIG-090 §7 — the PINNED chip-intersection test. The invariant it guards
 * forever: chips NARROW the held set (pure AND intersection, client-side,
 * zero IPC). The verified engine landmine this pins against: hybrid-mode
 * structured filters APPEND matches to the fused result (search.rs "Merge
 * structured results") — if chips were ever "optimized" into engine filters,
 * they would EXPAND sets. This test fails the moment filtering stops being
 * a pure subset operation.
 */
import { describe, it, expect } from 'vitest';
import {
	filterByChips, isContested, isForming, isUnlinked, anyChipOn,
	type ChipFacts, type ChipToggles,
} from '$lib/components/collectionChips';

const facts = (over: Partial<ChipFacts> = {}): ChipFacts => ({
	stage: null,
	incoming_count: 1,
	outgoing_count: 1,
	incoming_link_types_json: '{}',
	outgoing_link_types_json: '{}',
	review_due: false,
	...over,
});

const none: ChipToggles = { due: false, unlinked: false, contested: false, forming: false };

describe('MIG-090 §7 — chips narrow, never expand', () => {
	const rows = [
		{ id: 'due', f: facts({ review_due: true }) },
		{ id: 'unlinked', f: facts({ incoming_count: 0, outgoing_count: 0 }) },
		{ id: 'contested', f: facts({ outgoing_link_types_json: '{"contradicts": 2}' }) },
		{ id: 'forming', f: facts({ stage: 'growth-deepening' }) },
		{ id: 'plain', f: facts() },
		{ id: 'missing', f: null as ChipFacts | null },
	];
	const byId = (list: typeof rows) => list.map(r => r.id);

	it('no chips → identity (everything passes, missing included)', () => {
		expect(filterByChips(rows, r => r.f, none)).toEqual(rows);
	});

	it('every filtered result is a SUBSET of the input — for all 16 toggle combinations', () => {
		for (let mask = 0; mask < 16; mask++) {
			const chips: ChipToggles = {
				due: !!(mask & 1), unlinked: !!(mask & 2),
				contested: !!(mask & 4), forming: !!(mask & 8),
			};
			const out = filterByChips(rows, r => r.f, chips);
			expect(out.length).toBeLessThanOrEqual(rows.length);
			for (const e of out) expect(rows).toContain(e); // subset — never expands
		}
	});

	it('single chips select exactly their rows', () => {
		expect(byId(filterByChips(rows, r => r.f, { ...none, due: true }))).toEqual(['due']);
		expect(byId(filterByChips(rows, r => r.f, { ...none, unlinked: true }))).toEqual(['unlinked']);
		expect(byId(filterByChips(rows, r => r.f, { ...none, contested: true }))).toEqual(['contested']);
		expect(byId(filterByChips(rows, r => r.f, { ...none, forming: true }))).toEqual(['forming']);
	});

	it('two chips = AND intersection (not union)', () => {
		const both = facts({ review_due: true, incoming_count: 0, outgoing_count: 0 });
		const all = [...rows, { id: 'due+unlinked', f: both }];
		const out = filterByChips(all, r => r.f, { ...none, due: true, unlinked: true });
		expect(byId(out)).toEqual(['due+unlinked']); // union would also keep 'due' and 'unlinked'
	});

	it('missing members (no facts) never match an active chip', () => {
		const out = filterByChips(rows, r => r.f, { ...none, due: true });
		expect(byId(out)).not.toContain('missing');
	});

	it('predicate details: forming honors the lifecycle prefix; unstaged is honest-false', () => {
		expect(isForming(facts({ stage: 'spark' }))).toBe(true);
		expect(isForming(facts({ stage: 'growth-deep' }))).toBe(true);
		expect(isForming(facts({ stage: 'maturity' }))).toBe(false);
		expect(isForming(facts({ stage: null }))).toBe(false);
		expect(isContested(facts({ incoming_link_types_json: '{"contradicts": 1}' }))).toBe(true);
		expect(isContested(facts({ incoming_link_types_json: 'not json' }))).toBe(false);
		expect(isUnlinked(facts({ incoming_count: 0, outgoing_count: 0 }))).toBe(true);
		expect(anyChipOn(none)).toBe(false);
	});
});

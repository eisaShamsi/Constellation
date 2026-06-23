import { describe, it, expect } from 'vitest';
import { computedPriority, effectivePriority, type PrioritySignals } from '../../src/lib/reviewer/priorities';

// todayDay 0 = 2020-01-01 in the schedule's days-since-2020 frame.
const TODAY = 0;
const base: PrioritySignals = {
	reason: 'interval_due', days_overdue: 0, stale_trigger_type: null, stale_changed_on: null,
	incoming_count: 0, outgoing_count: 0, maturity: 'seed',
};

describe('MIG-084 §F.2 computedPriority', () => {
	it('contributions always sum exactly to the score (the recipe-bar invariant)', () => {
		for (const n of [
			base,
			{ ...base, reason: 'stale', stale_trigger_type: 'contradicts', stale_changed_on: '2019-11-15', incoming_count: 12, outgoing_count: 4, maturity: 'canonical' },
			{ ...base, reason: 'orphan', days_overdue: 5, maturity: 'sapling' },
			{ ...base, reason: 'fragile', incoming_count: 8, outgoing_count: 1, maturity: 'evergreen' },
		] as PrioritySignals[]) {
			const r = computedPriority(n, TODAY);
			const sum = r.contributions.reduce((a, c) => a + c.points, 0);
			expect(Math.round(sum)).toBe(r.score);
		}
	});

	it('ranks a long-stale, contradicting, well-connected canonical note HIGH', () => {
		const n: PrioritySignals = {
			...base, reason: 'stale', stale_trigger_type: 'contradicts', stale_changed_on: '2019-11-15',
			incoming_count: 12, outgoing_count: 4, maturity: 'canonical',
		};
		const r = computedPriority(n, TODAY);
		expect(r.score).toBeGreaterThan(60);
		expect(r.contributions[0].axis).toBe('urgency');
	});

	it('ranks a fresh, disconnected seed orphan LOW', () => {
		const r = computedPriority({ ...base, reason: 'orphan', days_overdue: 3, maturity: 'seed' }, TODAY);
		expect(r.score).toBeLessThan(25);
	});

	it('a fragile hub outranks an equally-fresh non-fragile note (importance breaks the tie)', () => {
		const hub: PrioritySignals = { ...base, reason: 'fragile', incoming_count: 8, outgoing_count: 1, maturity: 'evergreen' };
		const plain: PrioritySignals = { ...base, reason: 'interval_due', incoming_count: 0, maturity: 'seed' };
		expect(computedPriority(hub, TODAY).score).toBeGreaterThan(computedPriority(plain, TODAY).score);
	});

	it('a contradiction is more urgent than a supporting-link change, all else equal', () => {
		const contradict: PrioritySignals = { ...base, reason: 'stale', stale_trigger_type: 'contradicts', stale_changed_on: '2020-01-01' };
		const support: PrioritySignals = { ...base, reason: 'stale', stale_trigger_type: 'supports', stale_changed_on: '2020-01-01' };
		expect(computedPriority(contradict, TODAY).score).toBeGreaterThan(computedPriority(support, TODAY).score);
	});

	it('more overdue ⇒ higher (monotonic decay)', () => {
		const mild = computedPriority({ ...base, days_overdue: 2 }, TODAY).score;
		const severe = computedPriority({ ...base, days_overdue: 90 }, TODAY).score;
		expect(severe).toBeGreaterThan(mild);
	});

	it('effectivePriority: a user override wins; otherwise the computed score', () => {
		expect(effectivePriority(90, 40)).toBe(90);
		expect(effectivePriority(0, 40)).toBe(0); // an explicit 0 override is honored
		expect(effectivePriority(null, 40)).toBe(40);
		expect(effectivePriority(undefined, 40)).toBe(40);
	});
});

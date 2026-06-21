import { describe, it, expect } from 'vitest';
import { taskDateCompletions } from '$lib/editor/taskDates';

// Fixed reference: Sunday, 21 June 2026.
const NOW = new Date(2026, 5, 21);

function comp(before: string) {
	return taskDateCompletions(before, NOW);
}
function dates(before: string): string[] {
	return (comp(before)?.options ?? []).map((o) => o.date);
}

describe('taskDateCompletions — @ trigger (now = Sun 2026-06-21)', () => {
	it('@tomorrow → a single pinned suggestion', () => {
		const r = comp('Call dentist @tomorrow')!;
		expect(r.options[0].date).toBe('2026-06-22');
		expect(r.options[0].label).toBe('\u{1F4C5} 2026-06-22');
	});

	it('@ partial narrows the menu (@to → today + tomorrow)', () => {
		const keys = (comp('x @to')?.options ?? []).map((o) => o.detail);
		expect(keys).toContain('today');
		expect(keys).toContain('tomorrow');
		expect(keys).not.toContain('yesterday');
	});

	it('@ alone offers the full menu', () => {
		const keys = (comp('x @')?.options ?? []).map((o) => o.detail);
		expect(keys).toContain('today');
		expect(keys.length).toBeGreaterThan(3);
	});

	it('@next week / @next month / @in 3 days', () => {
		expect(dates('x @next week')).toContain('2026-06-28');
		expect(dates('x @next month')).toContain('2026-07-21');
		expect(dates('x @in 3 days')).toContain('2026-06-24');
	});

	it('@friday vs @next friday', () => {
		expect(dates('x @friday')).toContain('2026-06-26');       // coming Friday
		expect(dates('x @next friday')).toContain('2026-07-03');  // Friday of next week
	});

	it('replace-from points at the @', () => {
		const before = 'Call dentist @tomorrow';
		const r = comp(before)!;
		expect(before.slice(r.from)).toBe('@tomorrow');
	});
});

describe('taskDateCompletions — bare keyword fallback (forgot the @)', () => {
	it('a complete trailing keyword suggests its date', () => {
		expect(dates('Call dentist tomorrow')).toEqual(['2026-06-22']);
		expect(dates('ship the draft next week')).toEqual(['2026-06-28']);
		expect(dates('follow up in 3 days')).toEqual(['2026-06-24']);
		expect(dates('standup Monday')).toEqual(['2026-06-22']);
	});

	it('replace-from points at the keyword', () => {
		const before = 'Call dentist tomorrow';
		const r = comp(before)!;
		expect(before.slice(r.from)).toBe('tomorrow');
	});

	it('no suggestion when the trailing text is not a date', () => {
		expect(comp('discuss the quarterly plans')).toBeNull();
		expect(comp('write the report')).toBeNull();
	});

	it('partial typing has no bare match until the keyword completes', () => {
		expect(comp('meet tomo')).toBeNull();        // mid-word
		expect(comp('meet tomorrow')?.options[0].date).toBe('2026-06-22');
	});
});

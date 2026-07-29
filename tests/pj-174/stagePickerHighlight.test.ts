/**
 * PJ-179 — the stage picker opened on the FIRST entry, not the note's current stage.
 *
 * Boss-found 2026-07-29 while testing MIG-107 Slice 5. A note at Growth opened its stage list with
 * "Spark" highlighted, so the offered default was two steps BACKWARDS and one careless Enter would
 * have taken it there. A picker for a value that already exists should show where you ARE.
 *
 * The index logic is duplicated here rather than imported because it lives inside a `.svelte`
 * component; this pins the rule the component now follows, including the fallback that is the only
 * case where "no current entry" is the truth.
 */
import { describe, it, expect } from 'vitest';

/** Mirrors PropertyEditor.stageIndexOf. */
function stageIndexOf(opts: Array<{ value: string }>, current: string): number {
	const c = (current ?? '').trim().toLowerCase();
	if (!c) return 0;
	const i = opts.findIndex((o) => o.value.toLowerCase() === c);
	return i >= 0 ? i : 0;
}

const BASELINE = ['spark', 'birth', 'growth', 'maturity', 'dormancy', 'archival']
	.map((value) => ({ value }));

describe('PJ-179 — the stage list opens on the note\'s current stage', () => {
	it('lands on the current stage, not the top of the list', () => {
		expect(stageIndexOf(BASELINE, 'growth')).toBe(2);   // the Boss\'s note
		expect(stageIndexOf(BASELINE, 'archival')).toBe(5); // the last entry, not index 0
		expect(stageIndexOf(BASELINE, 'spark')).toBe(0);    // genuinely first
	});

	it('ignores case and surrounding whitespace, as the note may have either', () => {
		expect(stageIndexOf(BASELINE, 'Growth')).toBe(2);
		expect(stageIndexOf(BASELINE, '  GROWTH  ')).toBe(2);
	});

	it('falls back to the first entry ONLY when there is no current stage to land on', () => {
		expect(stageIndexOf(BASELINE, '')).toBe(0);
		expect(stageIndexOf(BASELINE, '   ')).toBe(0);
		// A custom per-note term is not one of the offered options — no current entry exists.
		expect(stageIndexOf(BASELINE, 'growth-arabic')).toBe(0);
	});
});

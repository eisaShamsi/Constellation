/**
 * PJ-106 §A0/§A1 — the OFFSET-PURE direction-resolution recipes (headless).
 *
 * Scope honesty (the migration's core discipline): jsdom NEVER computes layout, so the
 * VISUAL defects (caret side, End/Home landing, arrow motion) FALSE-PASS headlessly —
 * `textDirectionAt`/`bidiSpans`/`coordsAtPos` silently return correct values without a
 * real layout. Those are the Boss's LIVE staged tests (his detailed symptom reports are
 * the on-demand reproduction; the fix is verified on the running app). What IS reliable
 * headlessly is the PURE direction logic the fix rests on — `detectDir` (the deterministic
 * per-note base that replaces the `dir='auto'` competitor) and the bidiPlugin's per-line /
 * empty-line-inheritance classification. This file locks those, so a future change can't
 * silently re-introduce the ambiguity.
 */
import { describe, it, expect } from 'vitest';
import { detectDir } from '$lib/utils';

describe('PJ-106 §A1 — deterministic per-note base direction (kills the dir=auto competitor)', () => {
	it('never returns "auto"/empty — always a concrete rtl|ltr (the determinism A1 relies on)', () => {
		for (const t of ['', '   ', '#', '123 456', '!@#$', 'hello', 'مرحبا', 'مرحبا hello', 'hello مرحبا']) {
			expect(['rtl', 'ltr']).toContain(detectDir(t));
		}
	});

	it('an Arabic-dominant note resolves RTL; an English note resolves LTR', () => {
		expect(detectDir('مرحبا بالعالم، هذا نص عربي طويل')).toBe('rtl');
		expect(detectDir('Hello world, this is an English note')).toBe('ltr');
	});

	it('a bilingual note is classified by dominance, not by the first character', () => {
		// Latin-first but Arabic-dominant → RTL (the bug class: first-char heuristics get this wrong)
		expect(detectDir('OK: هذا نص عربي طويل جدا مع القليل من اللاتينية')).toBe('rtl');
		// Arabic-first but Latin-dominant → LTR
		expect(detectDir('ملاحظة: this is a mostly-English note with a short Arabic label')).toBe('ltr');
	});

	it('frontmatter is ignored — the base reflects the BODY the user writes', () => {
		expect(detectDir('---\ntitle: Test\ncid_cn: X\n---\nمرحبا بالعالم هذا نص عربي')).toBe('rtl');
	});
});

/**
 * The T-recipe register (the Boss's symptoms → live staged tests). Kept here as the
 * canonical list the Stage-1/Stage-2 Boss tutorials are built from; the ones marked
 * [state] gain headless assertions as their commands land in Part B.
 *   T① empty-line/empty-note caret side            [live: caret visual side]
 *   T② End/Home target on an RTL line              [live: caret landing]
 *   T③ Latin run at the end of an Arabic line      [live: boundary affinity]
 *   T④ select word/sentence/paragraph/page/line    [state: ranges (Part B) + live: highlight]
 *   T⑤ logical arrow motion across a bidi boundary [state: offset (Part B) + live: matches Word]
 *   T⑥ Shift+Home/End line selection               [live]
 *   T⑦ bilingual variants of the above             [state where offset-pure + live]
 * Round-3: Right/Left-Ctrl+Shift → per-paragraph 100% RTL/LTR via an invisible RLM/LRM
 * mark (Part B). See lab/reports/PJ-106-RTL-Symptoms-BossReported.md.
 */

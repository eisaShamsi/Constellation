/**
 * PJ-106 §B3 — select-sentence: the OFFSET-PURE recipes.
 *
 * `sentenceRangeInLine` is deterministic string logic (Intl.Segmenter, present in the V8 test
 * runtime — the same engine family WebView2 ships), so these ARE the whole behavior of which
 * sentence gets picked; only the click/keymap plumbing is a live Boss test. The Arabic cases
 * lock the design-inspection H4 contract: break on ؟ ! ۔ and ., but NOT on ؛ (intra-sentence),
 * and never false-break a decimal.
 */
import { describe, it, expect } from 'vitest';
import { sentenceRangeInLine } from '$lib/editor/sentenceSelect';

/** The substring a range selects — convenient for asserting the picked sentence. */
const pick = (text: string, off: number) => {
	const r = sentenceRangeInLine(text, off);
	return r ? text.slice(r.from, r.to) : null;
};

describe('PJ-106 §B3 — sentenceRangeInLine (English)', () => {
	const t = 'Hello world. How are you?';
	it('picks the first sentence, terminator kept, trailing space trimmed', () => {
		expect(pick(t, 2)).toBe('Hello world.');
	});
	it('picks the second sentence from a caret inside it', () => {
		expect(pick(t, 16)).toBe('How are you?');
	});
	it('a caret at the very end selects the last sentence', () => {
		expect(pick(t, t.length)).toBe('How are you?');
	});
});

describe('PJ-106 §B3 — sentenceRangeInLine (Arabic, the H4 contract)', () => {
	it('breaks on ؟ and !: picks each Arabic sentence', () => {
		const t = 'هل هذا صحيح؟ نعم بالتأكيد!';
		expect(pick(t, 2)).toBe('هل هذا صحيح؟');
		expect(pick(t, 15)).toBe('نعم بالتأكيد!');
	});

	it('does NOT break on ؛ (Arabic semicolon = intra-sentence): whole clause is one sentence', () => {
		const t = 'الجملة الثالثة؛ ما زالت مستمرة.';
		// A caret before OR after the ؛ selects the entire sentence (؛ is not a terminator).
		expect(pick(t, 3)).toBe('الجملة الثالثة؛ ما زالت مستمرة.');
		expect(pick(t, 20)).toBe('الجملة الثالثة؛ ما زالت مستمرة.');
	});

	it('breaks on ۔ (Arabic full stop U+06D4)', () => {
		const t = 'الجملة الأولى۔ الجملة الثانية۔';
		expect(pick(t, 2)).toBe('الجملة الأولى۔');
	});
});

describe('PJ-106 §B3 — sentenceRangeInLine (mixed + guards)', () => {
	it('does not false-break a decimal number', () => {
		const t = 'Pi is 3.14 exactly.';
		expect(pick(t, 0)).toBe('Pi is 3.14 exactly.');
	});

	it('handles a bilingual line (Arabic clause embedded)', () => {
		const t = 'This is English. وهذه عربية؟ done.';
		expect(pick(t, 2)).toBe('This is English.');
		expect(pick(t, 20)).toBe('وهذه عربية؟');
		expect(pick(t, t.length - 1)).toBe('done.');
	});

	it('an empty line yields no sentence range', () => {
		expect(sentenceRangeInLine('', 0)).toBeNull();
	});

	it('a line with no terminator is one whole sentence', () => {
		const t = 'مرحبا بالعالم بدون نقطة';
		expect(pick(t, 5)).toBe('مرحبا بالعالم بدون نقطة');
	});
});

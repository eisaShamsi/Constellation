/**
 * PJ-106 §B1 — paragraph navigation: the OFFSET-PURE recipes.
 *
 * Paragraph motion is direction-BLIND (next = further down the document, prev = further up —
 * never a screen direction), so unlike the arrows of §A5 there is no visual half to defer to a
 * live Boss test: these pure-offset functions ARE the whole behavior. The Arabic and bilingual
 * cases assert the SAME offsets as the Latin case — that is the proof of parity (Rule: RTL and
 * bilingual notes navigate paragraphs identically to English ones).
 */
import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { paragraphForwardPos, paragraphBackwardPos } from '$lib/editor/paragraphNav';

const st = (doc: string) => EditorState.create({ doc });

describe('PJ-106 §B1 — paragraphForwardPos (Ctrl+↓ → next paragraph start)', () => {
	it('jumps from mid-paragraph to the next paragraph start', () => {
		//        line1  line2  line3 line4   line5
		const s = st('para1a\npara1b\n\npara2a\npara2b');
		// 'para1a' = 0..6, '\n' 6, 'para1b' 7..13, '\n' 13, '' 14, '\n' 14... let's compute via API:
		const l4 = s.doc.line(4); // 'para2a'
		// caret anywhere in para1 → start of para2
		expect(paragraphForwardPos(s, 0)).toBe(l4.from); // start of para1
		expect(paragraphForwardPos(s, 3)).toBe(l4.from); // mid para1 line1
		expect(paragraphForwardPos(s, s.doc.line(2).from)).toBe(l4.from); // para1 line2
	});

	it('from a blank line jumps to the following paragraph start', () => {
		const s = st('para1\n\npara2');
		const blank = s.doc.line(2);
		expect(paragraphForwardPos(s, blank.from)).toBe(s.doc.line(3).from);
	});

	it('in the last paragraph returns document end (nowhere further to go)', () => {
		const s = st('para1\n\npara2a\npara2b');
		expect(paragraphForwardPos(s, s.doc.line(3).from)).toBe(s.doc.length);
		expect(paragraphForwardPos(s, s.doc.length)).toBe(s.doc.length);
	});

	it('skips multiple consecutive blank lines to the next real paragraph', () => {
		const s = st('para1\n\n\n\npara2');
		expect(paragraphForwardPos(s, 0)).toBe(s.doc.line(5).from);
	});

	it('is direction-blind: Arabic paragraphs give the SAME offsets as Latin', () => {
		const s = st('مرحبا بالعالم\nسطر ثانٍ\n\nفقرة ثانية');
		const p2 = s.doc.line(4); // 'فقرة ثانية'
		expect(paragraphForwardPos(s, 0)).toBe(p2.from);
		expect(paragraphForwardPos(s, 5)).toBe(p2.from);
	});

	it('handles a bilingual paragraph (Arabic with an embedded Latin clause)', () => {
		const s = st('هذا نص مع كلمة English هنا\nتابع\n\nفقرة تالية');
		expect(paragraphForwardPos(s, 0)).toBe(s.doc.line(4).from);
	});
});

describe('PJ-106 §B1 — paragraphBackwardPos (Ctrl+↑ → current, else previous paragraph start)', () => {
	it('from mid-paragraph goes to the start of the CURRENT paragraph', () => {
		const s = st('para1a\npara1b\n\npara2a\npara2b');
		const p2start = s.doc.line(4).from;
		// caret in para2 line2 → start of para2 (line4)
		expect(paragraphBackwardPos(s, s.doc.line(5).from + 2)).toBe(p2start);
		// caret mid para2 line1 → start of para2
		expect(paragraphBackwardPos(s, p2start + 3)).toBe(p2start);
	});

	it('when already AT a paragraph start, goes to the PREVIOUS paragraph start', () => {
		const s = st('para1a\npara1b\n\npara2a\npara2b');
		const p1start = s.doc.line(1).from;
		const p2start = s.doc.line(4).from;
		expect(paragraphBackwardPos(s, p2start)).toBe(p1start);
	});

	it('from a blank line goes to the previous paragraph start', () => {
		const s = st('para1a\npara1b\n\npara2');
		expect(paragraphBackwardPos(s, s.doc.line(3).from)).toBe(s.doc.line(1).from);
	});

	it('at document start stays at 0', () => {
		const s = st('para1a\npara1b\n\npara2');
		expect(paragraphBackwardPos(s, 0)).toBe(0);
		expect(paragraphBackwardPos(s, 3)).toBe(0); // mid first line → its start (0)
	});

	it('climbs to the TOP of a multi-line current paragraph, not just the previous line', () => {
		const s = st('a\nb\nc\nd\n\nnext');
		// caret on line 'd' (line4) → start of the paragraph = line1 'a'
		expect(paragraphBackwardPos(s, s.doc.line(4).from + 1)).toBe(0);
	});

	it('is direction-blind: Arabic paragraphs give the SAME offsets as Latin', () => {
		const s = st('مرحبا بالعالم\nسطر ثانٍ\n\nفقرة ثانية');
		const p1start = s.doc.line(1).from;
		const p2start = s.doc.line(4).from;
		expect(paragraphBackwardPos(s, p2start + 4)).toBe(p2start); // current
		expect(paragraphBackwardPos(s, p2start)).toBe(p1start); // already-at-start → previous
	});
});

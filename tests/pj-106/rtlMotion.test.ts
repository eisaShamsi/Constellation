/**
 * PJ-106 §A5 — logical (Word/Windows) arrow motion: the OFFSET-PURE half.
 *
 * Scope honesty (same discipline as rtlDirection.test.ts): jsdom computes NO layout, so the
 * VISUAL half of arrow motion (which physical arrow maps to forward on an RTL line — it reads
 * `textDirectionAt`, i.e. getComputedStyle) FALSE-PASSES headlessly and is a LIVE Boss test.
 * What IS reliable here is `logicalCharPos` — the raw-offset stepping that makes the motion
 * "logical" at all. These tests lock T⑤'s core claim: every press advances EXACTLY ONE
 * character of the document, in writing order, including across an Arabic↔Latin boundary and
 * across a line break — the property CM6's visual default does NOT have.
 */
import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { logicalCharPos } from '$lib/editor/rtlMotion';

const st = (doc: string) => EditorState.create({ doc });

describe('PJ-106 §A5 — logicalCharPos: one character of the TEXT per press', () => {
	it('steps one char forward/backward in a pure-Latin line', () => {
		const s = st('abc');
		expect(logicalCharPos(s, 0, true)).toBe(1);
		expect(logicalCharPos(s, 2, false)).toBe(1);
	});

	it('steps one char forward/backward in a pure-Arabic line (offsets, not screen positions)', () => {
		const s = st('مرحبا');
		expect(logicalCharPos(s, 0, true)).toBe(1);
		expect(logicalCharPos(s, 3, false)).toBe(2);
	});

	it('crosses an Arabic↔Latin boundary one character at a time (T⑤ — the headline recipe)', () => {
		// 'abc' = 0..3, then 'مرحبا' = 3..8. The boundary is offset 3.
		const s = st('abcمرحبا');
		expect(logicalCharPos(s, 2, true)).toBe(3); // last Latin char → the boundary
		expect(logicalCharPos(s, 3, true)).toBe(4); // boundary → first Arabic char: ONE step, no stall
		expect(logicalCharPos(s, 4, false)).toBe(3); // and back again, symmetrically
		expect(logicalCharPos(s, 3, false)).toBe(2);
	});

	it('crosses a Latin run embedded INSIDE an Arabic line (the Boss writes bilingual)', () => {
		const s = st('هذا English نص');
		// Walking the whole line forward must visit every offset exactly once, never repeat or skip.
		const seen: number[] = [];
		let p = 0;
		while (p < s.doc.length) {
			const next = logicalCharPos(s, p, true);
			expect(next).toBeGreaterThan(p); // strictly monotonic — never stalls, never doubles back
			seen.push(next);
			p = next;
		}
		expect(seen[seen.length - 1]).toBe(s.doc.length);
		expect(seen.length).toBe(s.doc.length); // exactly one press per character
	});

	it('steps across a line break at the line edges', () => {
		const s = st('ab\nمرحبا'); // line1 = 0..2, newline at 2, line2 = 3..8
		expect(logicalCharPos(s, 2, true)).toBe(3); // end of line 1 → start of line 2
		expect(logicalCharPos(s, 3, false)).toBe(2); // start of line 2 → end of line 1
	});

	it('clamps at the document edges (never runs off either end)', () => {
		const s = st('مرحبا');
		expect(logicalCharPos(s, 0, false)).toBe(0);
		expect(logicalCharPos(s, s.doc.length, true)).toBe(s.doc.length);
	});

	it('treats a combining/diacritic cluster as ONE press (Arabic harakat)', () => {
		// بَ = base letter + fatha (U+064E) — a single grapheme cluster, so one arrow press.
		const s = st('بَا');
		expect(logicalCharPos(s, 0, true)).toBe(2); // skips the combining mark with the base letter
	});
});

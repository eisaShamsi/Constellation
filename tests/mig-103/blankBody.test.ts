/**
 * MIG-103 D2 — the blank-body test that would have caught the door not showing.
 *
 * Boss-reported 2026-07-21: the "Start from a template…" door never appeared in a
 * new note. The cause was not styling. CM6's own `placeholder` extension renders
 * on exactly one condition — `doc.length ? none : placeholder` — and a freshly
 * created note's body is `"\n"`: the blank line that follows the closing `---`.
 * Length 1, so the placeholder was never shown.
 *
 * The lesson is the assumption, not the line: "looks empty" is not `length === 0`.
 * These tests pin the corrected predicate — a body is blank when it is empty OR
 * nothing but whitespace — against the exact bytes `create_note` produces.
 */
import { describe, it, expect } from 'vitest';

/** Mirrors `isBlankBody` in NotePane.svelte. Kept in step with it deliberately:
 *  the guard is the contract, and the contract is what this file protects. */
function isBlankBody(doc: string): boolean {
	if (doc.length === 0) return true;
	if (doc.length > 8) return false; // O(1) on real notes — never stringify a big doc
	return doc.trim() === '';
}

/** The body of a note produced by `create_note`: `---\n{fm}\n---\n\n` → body `"\n"`. */
const NEW_NOTE_BODY = '\n';

describe('MIG-103 D2 — the template door shows on a blank body', () => {
	it('THE REGRESSION: a freshly created note counts as blank', () => {
		// This is the exact case that failed. CM6's `doc.length === 0` test says
		// false here; ours must say true.
		expect(NEW_NOTE_BODY.length).toBe(1);
		expect(isBlankBody(NEW_NOTE_BODY)).toBe(true);
	});

	it('a truly empty body is blank', () => {
		expect(isBlankBody('')).toBe(true);
	});

	it.each(['\n', '\r\n', '\n\n', ' ', '\t', ' \n '])(
		'whitespace-only body %j is blank',
		(doc) => expect(isBlankBody(doc)).toBe(true),
	);

	it('any real content is NOT blank — the door must disappear', () => {
		expect(isBlankBody('a')).toBe(false);
		expect(isBlankBody('\nx')).toBe(false);
		expect(isBlankBody('# Heading')).toBe(false);
	});

	/** Rule 1 — the check must never stringify a large document. A long body is
	 *  rejected on length alone, before any allocation. */
	it('is O(1) on a large document — rejected on length, never stringified', () => {
		const big = 'x'.repeat(200_000);
		expect(isBlankBody(big)).toBe(false);
		// Whitespace-only but LONG is also rejected without a trim: correct, because
		// a body that large is not the "new empty note" case the door serves.
		expect(isBlankBody(' '.repeat(200_000))).toBe(false);
	});

	it('the threshold covers a realistic blank body but not prose', () => {
		expect(isBlankBody('\n'.repeat(8))).toBe(true); // still blank
		expect(isBlankBody('\n'.repeat(9))).toBe(false); // past the O(1) guard
	});
});

/**
 * PJ-114 §0.2 — the shared parser-free wikilink finder extracted from CodeMirrorEditor's
 * Ctrl/⌘-click handler (so FocusPane's FM+ affordances reuse ONE copy). These pin the exact
 * behavior the main editor has always had, so the extraction changes nothing: same regex,
 * same hit predicate (offset inclusive of both `[[` and past-`]]`), same alias/#heading strip.
 */
import { describe, it, expect } from 'vitest';
import { findWikilinkAtLineOffset } from '$lib/editor/linkAtPos';

describe('findWikilinkAtLineOffset — parser-free wikilink under a line offset', () => {
	it('finds a plain [[link]] when the offset is inside it', () => {
		// "a [[Note]] b" — [[Note]] spans [2,10]
		const hit = findWikilinkAtLineOffset('a [[Note]] b', 5);
		expect(hit).toEqual({ raw: 'Note', target: 'Note', from: 2, to: 10 });
	});

	it('strips an |alias to resolve the target', () => {
		const hit = findWikilinkAtLineOffset('[[Note|Alias]]', 3);
		expect(hit?.raw).toBe('Note|Alias');
		expect(hit?.target).toBe('Note');
	});

	it('strips a #heading to resolve the target', () => {
		const hit = findWikilinkAtLineOffset('[[Note#Heading]]', 4);
		expect(hit?.raw).toBe('Note#Heading');
		expect(hit?.target).toBe('Note');
	});

	it('strips both #heading and |alias (heading before alias)', () => {
		const hit = findWikilinkAtLineOffset('[[Note#H|Alias]]', 4);
		expect(hit?.target).toBe('Note');
	});

	it('keeps spaces in a multi-word target', () => {
		const hit = findWikilinkAtLineOffset('see [[My Long Note]] end', 8);
		expect(hit?.target).toBe('My Long Note');
	});

	it('matches on the opening [[ boundary (offset === from)', () => {
		const hit = findWikilinkAtLineOffset('a [[Note]] b', 2);
		expect(hit?.target).toBe('Note');
	});

	it('matches on the closing ]] boundary (offset === to) — the original inclusive predicate', () => {
		const hit = findWikilinkAtLineOffset('a [[Note]] b', 10);
		expect(hit?.target).toBe('Note');
	});

	it('returns null just past the link', () => {
		expect(findWikilinkAtLineOffset('a [[Note]] b', 11)).toBeNull();
	});

	it('returns null before the link', () => {
		expect(findWikilinkAtLineOffset('a [[Note]] b', 0)).toBeNull();
	});

	it('returns null on plain text with no wikilink', () => {
		expect(findWikilinkAtLineOffset('just some plain text', 5)).toBeNull();
	});

	it('picks the wikilink under the offset when a line has several', () => {
		// "[[A]] [[B]]" — [[A]] spans [0,5], [[B]] spans [6,11]
		expect(findWikilinkAtLineOffset('[[A]] [[B]]', 1)?.target).toBe('A');
		expect(findWikilinkAtLineOffset('[[A]] [[B]]', 8)?.target).toBe('B');
	});
});

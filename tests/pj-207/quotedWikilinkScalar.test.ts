/**
 * PJ-207 §15 whole-app sweep (HIGH) — **a quoted wikilink is a SCALAR, not a list.**
 *
 * `parseFrontmatter` unquoted the value first and then asked "does it start with `[` and end
 * with `]`?" — true of every quoted wikilink. `parent: "[[Architecture]]"` was therefore routed
 * into the flow-sequence splitter and came back as `['[Architecture]']`, one bracket pair gone.
 * Any later edit of that key wrote the broken form to disk, destroying the link.
 *
 * Constellation writes this shape itself (`set_frontmatter_parent`), so it was corrupting its
 * own output — and a comma in the note title made it worse by splitting the title as well.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter } from '../../src/lib/libraries/store';

const fm = (body: string) => parseFrontmatter(`---
${body}
---

Body.
`).properties;
const keyed = (body: string, key: string) => fm(body).find((p) => p.key === key);

describe('PJ-207 §15 — a quoted wikilink stays a scalar', () => {
	it('keeps both bracket pairs and stays a scalar', () => {
		const p = keyed('parent: "[[Architecture]]"', 'parent');
		expect(p?.value).toBe('[[Architecture]]');
		expect(p?.listItems).toBeUndefined();
	});

	it('survives a comma in the note title', () => {
		const p = keyed('parent: "[[Foo, Bar]]"', 'parent');
		expect(p?.value).toBe('[[Foo, Bar]]');
		expect(p?.listItems).toBeUndefined();
	});

	it('an UNQUOTED inline list is still parsed as a list', () => {
		const p = keyed('tags: [alpha, beta]', 'tags');
		expect(p?.listItems).toEqual(['alpha', 'beta']);
	});

	it('a quoted item inside an unquoted list keeps its comma', () => {
		const p = keyed('aliases: [alpha, "Rosenthal, F."]', 'aliases');
		expect(p?.listItems).toEqual(['alpha', 'Rosenthal, F.']);
	});
});

/**
 * G4 Phase 3 — composeUpdatedContent (the byte-perfect replacement for
 * buildFullContent on a read→edit-props→write of an EXISTING note, e.g. adding a
 * tag/link to a CLOSED note). It must preserve rich frontmatter the lossy
 * buildFullContent would destroy.
 */
import { describe, it, expect } from 'vitest';
import { composeUpdatedContent, parseFrontmatter, buildFullContent } from '$lib/libraries/store';

const RICH = [
	'---',
	'title: Closed Note',
	'source:',
	'  author: Ibn Khaldun',
	'  year: 1377',
	'description: |',
	'  first line',
	'  second line',
	"quote: 'He said: \"hi\"'",
	'tags:',
	'  - a',
	'---',
	'body text',
	'',
].join('\n');

describe('G4 Phase 3 — composeUpdatedContent preserves rich frontmatter on a closed-note edit', () => {
	it('editing a scalar prop keeps nested map + block scalar + quoted value byte-perfect', () => {
		const { properties, body } = parseFrontmatter(RICH);
		const newProps = properties.map((p) => (p.key === 'title' ? { ...p, value: 'Closed Note EDITED' } : p));
		const out = composeUpdatedContent(RICH, newProps, body);
		expect(out).toContain('title: Closed Note EDITED');
		expect(out).toContain('author: Ibn Khaldun'); // nested map preserved
		expect(out).toContain('year: 1377');
		expect(out).toContain('first line'); // block scalar preserved
		expect(out).toContain('second line');
		expect(out).toContain("quote: 'He said: \"hi\"'"); // quoted value byte-perfect
		expect(out).not.toContain('description: "|"'); // NOT the corruption signature
		expect(out).toContain('body text');
	});

	it('a no-op update is byte-perfect', () => {
		const { properties, body } = parseFrontmatter(RICH);
		expect(composeUpdatedContent(RICH, properties, body)).toBe(RICH);
	});

	it('adds the first property to an EMPTY-fence note (review Finding 1 — was a no-op)', () => {
		const empty = '---\n---\nbody text\n';
		const out = composeUpdatedContent(empty, [{ key: 'tags', value: 'a', type: 'list', listItems: ['a'] }], 'body text\n');
		expect(out).toContain('tags:'); // the property landed
		expect(out).toContain('a');
		expect(out).toContain('body text');
		expect(parseFrontmatter(out).properties.find((p) => p.key === 'tags')).toBeTruthy();
	});

	/**
	 * This case locks in WHY Phase 3 exists: `buildFullContent` rebuilds frontmatter from
	 * the projection instead of editing the real bytes, so it is not byte-perfect.
	 *
	 * ★ UPDATED BY PJ-182 (2026-07-29). It used to assert the *content loss* directly —
	 * `expect(legacy).not.toContain('first line')`, i.e. the block scalar collapsed to a
	 * literal `"|"` and its prose was dropped. That loss is now FIXED at the source: the
	 * parser projects a block scalar READ-ONLY with its bytes verbatim, so the legacy
	 * composer can no longer destroy it. Asserting the old damage would now be pinning a
	 * defect in place.
	 *
	 * What remains true — and is the real reason to prefer `composeUpdatedContent` — is
	 * that the legacy path still RE-SERIALIZES: the single-quoted `'He said: "hi"'` comes
	 * back double-quoted with escapes. Same value, different bytes, on every write.
	 * (`buildFullContent` being a lossier composer than `compose()` is carried as PJ-180.)
	 */
	it('the legacy buildFullContent is still not byte-perfect on the same note', () => {
		const { properties, body } = parseFrontmatter(RICH);
		const legacy = buildFullContent(properties, body);

		// PJ-182 — no longer destroyed: the block scalar and the nested map both survive.
		expect(legacy).toContain('description: |');
		expect(legacy).toContain('first line');
		expect(legacy).toContain('second line');
		expect(legacy).toContain('author: Ibn Khaldun');
		expect(legacy).not.toContain('description: "|"'); // the old corruption signature

		// But it re-quotes, so it is NOT a byte-perfect round trip — which is the point.
		expect(legacy).not.toBe(RICH);
		expect(legacy).toContain('quote: "He said: \\"hi\\""');
		// …whereas the CST path IS byte-perfect (asserted above too).
		expect(composeUpdatedContent(RICH, properties, body)).toBe(RICH);
	});
});

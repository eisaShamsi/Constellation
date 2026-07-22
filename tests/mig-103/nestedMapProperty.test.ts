/**
 * PJ-136 — a nested MAP property shows as "Empty" in the property editor.
 *
 * `yamlDoc.projectProps` deliberately skips nested maps ("preserved in the CST, not
 * editable here — Boss decision"). But the note MODEL is projected by the hand-rolled
 * line parser `store.parseFrontmatter`, which has no such rule: a key whose value is
 * on the following lines yields `{ key: 'source', value: '' }`. That row is what the
 * user sees, labelled Empty, for a property that is not empty.
 *
 * These tests establish exactly how far the damage goes, because the answer decides
 * the severity: untouched is SAFE (proven on the Boss's disk), and typing into the
 * row is the open question.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter } from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter } from '$lib/editor/yamlDoc';

const NOTE = [
	'---',
	'title: Nested Property Probe',
	'tags:',
	'  - probe',
	'source:',
	'  title: Muqaddimah',
	'  author: Ibn Khaldun',
	'  year: 1377',
	'stage: seed',
	'---',
	'',
	'body text',
	'',
].join('\n');

function base() {
	const { yaml, hadFence } = splitFrontmatter(NOTE);
	const { properties, body } = parseFrontmatter(NOTE);
	return { yaml, hadFence, properties, body };
}

describe('PJ-136 — the nested map behind the "Empty" row', () => {
	it('the nested map is now TYPED, and carries its children for the summary', () => {
		const { properties } = base();
		const source = properties.find((p) => p.key === 'source');
		expect(source, 'the row the user sees').toBeDefined();
		expect(source!.type).toBe('nested-map');
		expect(source!.nestedKeys).toEqual(['title', 'author', 'year']);
		// `value` stays empty ON PURPOSE — the legacy `reconstructFrontmatter`, still
		// live behind `buildFullContent`, serializes from `value`, so putting the
		// summary there would make it write `source: "title, author, year"` over the
		// block. The summary rides in `nestedKeys`; no existing write behaviour moves.
		expect(source!.value).toBe('');
	});

	it('a real LIST is still a list — the nested-map branch must not steal it', () => {
		const { properties } = base();
		const tags = properties.find((p) => p.key === 'tags');
		expect(tags!.type).toBe('list');
		expect(tags!.listItems).toEqual(['probe']);
	});

	it('an ordinary scalar is untouched', () => {
		const { properties } = base();
		expect(properties.find((p) => p.key === 'stage')!.type).toBe('text');
		expect(properties.find((p) => p.key === 'title')!.value).toBe('Nested Property Probe');
	});

	it('SAFE — editing a DIFFERENT property leaves the nested map byte-intact', () => {
		const { yaml, hadFence, properties, body } = base();
		const next = properties.map((p) =>
			p.key === 'stage' ? { ...p, value: 'sapling' } : p,
		);
		const out = composeFrontmatter(yaml, hadFence, properties, next, body);

		expect(out).toContain('  title: Muqaddimah');
		expect(out).toContain('  author: Ibn Khaldun');
		expect(out).toContain('  year: 1377');
		expect(out).toContain('stage: sapling');
	});

	/**
	 * THE FIX, from the other side. This test asserted the bug before PJ-136 and now
	 * asserts its absence: the same edit that used to replace the whole block is now
	 * a no-op, because the WRITE PATH refuses the type — not because a widget happens
	 * to be read-only today.
	 *
	 * That distinction is the point. A read-only widget protects the data only while
	 * every caller keeps it read-only; refusing in `composeFrontmatter` means the
	 * block survives however the panel behaves.
	 */
	it('FIXED — a write against the nested map is refused, not applied', () => {
		const { yaml, hadFence, properties, body } = base();
		const next = properties.map((p) =>
			p.key === 'source' ? { ...p, value: 'typed by the user' } : p,
		);
		const out = composeFrontmatter(yaml, hadFence, properties, next, body);

		expect(out).not.toContain('typed by the user');
		expect(out).toContain('  title: Muqaddimah');
		expect(out).toContain('  author: Ibn Khaldun');
		expect(out).toContain('  year: 1377');
	});

	/**
	 * The other half of the refusal. `composeFrontmatter` SPLICES any key that is in
	 * oldProps but missing from newProps — so a UI that merely stopped listing the row
	 * would delete the block from disk. Dropping it must be a no-op too.
	 */
	it('FIXED — dropping the row from the props does not splice the block', () => {
		const { yaml, hadFence, properties, body } = base();
		const next = properties.filter((p) => p.key !== 'source');
		const out = composeFrontmatter(yaml, hadFence, properties, next, body);

		expect(out).toContain('  title: Muqaddimah');
		expect(out).toContain('  author: Ibn Khaldun');
	});

	it('a genuinely deleted ORDINARY property is still removed', () => {
		const { yaml, hadFence, properties, body } = base();
		const next = properties.filter((p) => p.key !== 'stage');
		const out = composeFrontmatter(yaml, hadFence, properties, next, body);

		expect(out).not.toContain('stage:');
		expect(out).toContain('  title: Muqaddimah'); // and the map is still fine
	});
});

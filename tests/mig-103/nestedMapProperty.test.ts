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
	it('the model projects the nested map as an EMPTY property — this is the wrong label', () => {
		const { properties } = base();
		const source = properties.find((p) => p.key === 'source');
		expect(source, 'the row the user sees').toBeDefined();
		expect(source!.value).toBe('');
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
	 * CHARACTERIZATION TEST — this asserts the BUG, deliberately.
	 *
	 * Typing into the row labelled "Empty" replaces the whole nested block with the
	 * typed scalar. That is what makes PJ-136 more than cosmetic: "Empty" is not a
	 * wrong label the user can shrug off, it is an invitation to fill in a field —
	 * and accepting the invitation destroys the data, silently.
	 *
	 * When PJ-136 is fixed this test MUST go red. Flip it then; do not delete it.
	 */
	it('BUG (PJ-136) — typing into the "Empty" row destroys the nested map', () => {
		const { yaml, hadFence, properties, body } = base();
		const next = properties.map((p) =>
			p.key === 'source' ? { ...p, value: 'typed by the user' } : p,
		);
		const out = composeFrontmatter(yaml, hadFence, properties, next, body);

		expect(out).toContain('source: typed by the user');
		expect(out).not.toContain('Muqaddimah');
		expect(out).not.toContain('Ibn Khaldun');
		expect(out).not.toContain('1377');
	});
});

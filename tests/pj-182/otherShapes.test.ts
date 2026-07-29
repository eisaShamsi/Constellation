/**
 * PJ-182 Slice 4 — the three OTHER shapes the same scanner mis-read, found by the
 * whole-ecosystem sweep and fixed in the same pass (WA#6: if we discover it, we fix it).
 *
 * 1. BLOCK SCALARS — `desc: |` / `desc: >` (with optional `-`, `+`, indentation digits).
 *    The multi-line branches all require an EMPTY inline value, but a block scalar's value
 *    IS the indicator, so none of them fired: the key projected as editable TEXT whose
 *    value was the literal character `|`, its prose was skipped by the top-level-key guard,
 *    and `buildFullContent` wrote `desc: "|"` into the tab.content cache — the prose gone.
 *
 * 2. FLOW SEQUENCE ON THE NEXT LINE — `tags:` then `  [alpha, beta]`. Valid YAML, read
 *    correctly by the CST parser, but projected read-only here, so the user could not edit
 *    their own tags. No data was lost; the property was simply unreachable.
 *
 * 3. A COMMENT INSIDE A BLOCK — covered in `zeroIndentBlockList.test.ts`; the zero-indent
 *    form used to project EMPTY (destructive), and now matches the indented form's
 *    read-only-with-bytes-verbatim answer.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter, buildFullContent } from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter } from '$lib/editor/yamlDoc';

describe('PJ-182 §4.1 — block scalars', () => {
	const LITERAL = [
		'---',
		'title: T',
		'desc: |',
		'  first line of prose',
		'  second line of prose',
		'stage: spark-seed',
		'---',
		'',
		'body',
		'',
	].join('\n');

	it('is projected READ-ONLY with its prose intact, never as the character "|"', () => {
		const { properties } = parseFrontmatter(LITERAL);
		const desc = properties.find((p) => p.key === 'desc')!;

		// Before: { value: '|', type: 'text' } — an editable row showing a pipe.
		expect(desc.type).toBe('nested-map');
		expect(desc.value).toBe('|');
		expect(desc.nestedRaw).toEqual(['  first line of prose', '  second line of prose']);

		// The keys around it are still ordinary top-level properties.
		expect(properties.map((p) => p.key)).toEqual(['title', 'desc', 'stage']);
	});

	it('survives the tab.content cache byte-for-byte', () => {
		const { properties, body } = parseFrontmatter(LITERAL);
		const cached = buildFullContent(properties, body);

		// Before: `desc: "|"` and both prose lines deleted.
		expect(cached).toContain('desc: |');
		expect(cached).toContain('  first line of prose');
		expect(cached).toContain('  second line of prose');

		// And it re-reads identically — no silent revert to an editable pipe.
		const again = parseFrontmatter(cached).properties.find((p) => p.key === 'desc')!;
		expect(again.type).toBe('nested-map');
		expect(again.nestedRaw).toEqual(['  first line of prose', '  second line of prose']);
	});

	/**
	 * Found by the `/simplify` altitude pass on this very fix. Rendering the row read-only
	 * is the WIDGET half; the half that actually protects the bytes is `composeFrontmatter`
	 * refusing to write the key at all — and `immutableBlockKeys` tested only `isMap` and
	 * seq-of-non-scalars. A block scalar is `isScalar` (node type `BLOCK_LITERAL` /
	 * `BLOCK_FOLDED`), so it was in neither set.
	 *
	 * Proven before the fix: a props array that merely OMITTED the row deleted `desc: |`
	 * and both prose lines from the file; one that changed it wrote the new value over the
	 * block. This module's own comment is the rule — *"refusing here means the block
	 * survives however the panel behaves."*
	 */
	it('the WRITE PATH refuses it, however the props array behaves', () => {
		const { yaml, hadFence } = splitFrontmatter(LITERAL);
		const { properties, body } = parseFrontmatter(LITERAL);

		const omitted = properties.filter((p) => p.key !== 'desc');
		const afterOmit = composeFrontmatter(yaml, hadFence, properties, omitted, body);
		expect(afterOmit).toContain('desc: |');
		expect(afterOmit).toContain('  first line');
		expect(afterOmit).toContain('  second line');

		const overwritten = properties.map((p) => (p.key === 'desc' ? { ...p, value: 'typed over' } : p));
		const afterEdit = composeFrontmatter(yaml, hadFence, properties, overwritten, body);
		expect(afterEdit).toContain('  first line');
		expect(afterEdit).toContain('  second line');
		expect(afterEdit).not.toContain('typed over');

		// A folded scalar is the same shape.
		const folded = LITERAL.replace('desc: |', 'desc: >');
		const f = splitFrontmatter(folded);
		const fp = parseFrontmatter(folded);
		const afterFold = composeFrontmatter(
			f.yaml, f.hadFence, fp.properties, fp.properties.filter((p) => p.key !== 'desc'), fp.body,
		);
		expect(afterFold).toContain('desc: >');
		expect(afterFold).toContain('  first line');
	});

	it('an edit to a neighbouring key leaves the block byte-identical', () => {
		const { yaml, hadFence } = splitFrontmatter(LITERAL);
		const { properties, body } = parseFrontmatter(LITERAL);
		const edited = properties.map((p) => (p.key === 'stage' ? { ...p, value: 'growth-seed' } : p));
		const out = composeFrontmatter(yaml, hadFence, properties, edited, body);

		expect(out).toContain('desc: |\n  first line of prose\n  second line of prose');
		expect(out).toContain('stage: growth-seed');
	});

	it.each(['|', '>', '|-', '>-', '|+', '|2', '>2-'])('handles the %s indicator', (ind) => {
		const note = ['---', `desc: ${ind}`, '  prose', 'stage: seed', '---', '', 'body', ''].join('\n');
		const desc = parseFrontmatter(note).properties.find((p) => p.key === 'desc')!;
		expect(desc.type).toBe('nested-map');
		expect(desc.value).toBe(ind);
		expect(desc.nestedRaw).toEqual(['  prose']);
	});

	it('REGRESSION — a value that merely CONTAINS a pipe stays ordinary editable text', () => {
		const note = ['---', 'title: a | b', 'note: piped |> arrow', '---', '', 'body', ''].join('\n');
		const props = parseFrontmatter(note).properties;
		expect(props.find((p) => p.key === 'title')!.type).toBe('text');
		expect(props.find((p) => p.key === 'title')!.value).toBe('a | b');
		expect(props.find((p) => p.key === 'note')!.type).toBe('text');
	});

	it('REGRESSION — a bare `key: |` with nothing indented after it is left alone', () => {
		const note = ['---', 'desc: |', 'stage: seed', '---', '', 'body', ''].join('\n');
		const props = parseFrontmatter(note).properties;
		expect(props.map((p) => p.key)).toEqual(['desc', 'stage']);
		expect(props.find((p) => p.key === 'stage')!.value).toBe('seed');
	});
});

describe('PJ-182 §4.2 — a flow sequence on the line after its key', () => {
	const FLOW = ['---', 'title: T', 'tags:', '  [alpha, beta]', 'stage: seed', '---', '', 'body', ''].join('\n');

	it('is an editable list, matching what the CST parser already read', () => {
		const { properties } = parseFrontmatter(FLOW);
		const tags = properties.find((p) => p.key === 'tags')!;

		// Before: { type: 'nested-map', nestedRaw: ['  [alpha, beta]'] } — uneditable.
		expect(tags.type).toBe('list');
		expect(tags.listItems).toEqual(['alpha', 'beta']);
		expect(properties.map((p) => p.key)).toEqual(['title', 'tags', 'stage']);
	});

	it('a tag added to it reaches the file, with the existing tags kept', () => {
		const { yaml, hadFence } = splitFrontmatter(FLOW);
		const { properties, body } = parseFrontmatter(FLOW);
		const edited = properties.map((p) =>
			p.key === 'tags' ? { ...p, listItems: ['alpha', 'beta', 'gamma'], value: 'alpha, beta, gamma' } : p,
		);
		const out = composeFrontmatter(yaml, hadFence, properties, edited, body);

		expect(parseFrontmatter(out).properties.find((p) => p.key === 'tags')!.listItems).toEqual([
			'alpha',
			'beta',
			'gamma',
		]);
		expect(out).toContain('stage: seed');
	});

	it('REGRESSION — a flow sequence on the SAME line still works', () => {
		const note = ['---', 'tags: [alpha, beta]', '---', '', 'body', ''].join('\n');
		const tags = parseFrontmatter(note).properties.find((p) => p.key === 'tags')!;
		expect(tags.type).toBe('list');
		expect(tags.listItems).toEqual(['alpha', 'beta']);
	});
});

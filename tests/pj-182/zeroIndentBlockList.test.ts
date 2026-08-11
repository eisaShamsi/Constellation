/**
 * PJ-182 — REPRODUCE-FIRST. A ZERO-INDENT YAML block list silently deletes the user's data.
 *
 * This is the FOURTH shape of a class this repo has already closed three times
 * (PJ-136 nested maps · MIG-101 nested-object-list · the 2026-07-24 inspection's
 * seq-of-maps). LL-014's three strikes on the class are spent, so the fix is
 * STRUCTURAL: `parseFrontmatter` stops requiring a list item to be INDENTED, and the
 * "a `- ` line is never a key" rule that `search.rs::parse_frontmatter` has carried all
 * along is brought over to the JS parser too.
 *
 * ── THE SHAPE ────────────────────────────────────────────────────────────────
 *   ---
 *   tags:
 *   - alpha            <- column 0. VALID YAML: a block sequence may sit at the
 *   - beta                same indentation as its parent mapping key.
 *   ---
 * Constellation reads vaults IN PLACE, and this is ordinary in imported and
 * hand-authored ones.
 *
 * ── WHAT WAS OBSERVED (2026-07-30, by running the code; lab/reports/pj182-observation*.txt)
 *   store.parseFrontmatter   -> tags {value:'', type:'list', listItems:[]}  ← items ABSENT
 *   yamlDoc.parseFrontmatterDoc -> tags ['alpha','beta']                    ← CORRECT
 * The two parsers DISAGREE, and the one that feeds the visible panel is the wrong one.
 * All three multi-line branches (store.ts:1900/:1981/:2009) tested the NEXT line with
 * /^\s+-\s/ or /^\s/ — every one of them REQUIRES a leading space — so none fired and the
 * key fell through to the scalar path as an empty list.
 *
 * Consequences, each observed rather than reasoned:
 *   - the panel draws `tags` as an EMPTY editable chip list;
 *   - adding one tag makes composeFrontmatter splice the block and append a fresh one:
 *     `alpha` and `beta` are gone from the .md, with no error, and the result re-parses
 *     cleanly so nothing downstream ever notices;
 *   - `buildFullContent` (the tab.content cache) writes `tags: ` — the whole list, gone;
 *   - `aliases` is hit identically, and it is a LINK-RESOLUTION key: losing it silently
 *     breaks every backlink that resolved through that alias;
 *   - a zero-indent SEQ-OF-MAPS is worse still — `- name: X` at column 0 has a colon and
 *     no leading space, so the outer guard admitted it as a TOP-LEVEL KEY named `- name`.
 *
 * Editing a NEIGHBOURING key is safe (both sides of the diff project the list identically,
 * so the CST is untouched) — the loss fires on a write to THAT key.
 */
import { describe, it, expect } from 'vitest';
import {
	parseFrontmatter,
	buildFullContent,
	composeUpdatedContent,
	type FrontmatterProperty,
} from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter, composeContent, parseFrontmatterDoc } from '$lib/editor/yamlDoc';

const ZERO_INDENT = [
	'---',
	'cid_cn: ABCD',
	'title: Imported Probe',
	'tags:',
	'- alpha',
	'- beta',
	'aliases:',
	'- Old Name',
	'- Older Name',
	'stage: spark-seed',
	'---',
	'',
	'body text',
	'',
].join('\n');

/** Verbatim copy of `+layout.svelte`'s addTagToProps (the `addTagToNote` writer). */
function addTagToProps(props: FrontmatterProperty[], tag: string): FrontmatterProperty[] {
	const idx = props.findIndex((p) => p.key.toLowerCase() === 'tags');
	if (idx >= 0) {
		const p = props[idx];
		const existing = p.listItems ?? (p.value ? String(p.value).split(',').map((s) => s.trim()).filter(Boolean) : []);
		if (existing.some((x) => x.toLowerCase() === tag.toLowerCase())) return props;
		const items = [...existing, tag];
		return props.map((q, i) =>
			i === idx ? { ...q, type: 'list' as const, listItems: items, value: items.join(', ') } : q,
		);
	}
	return [...props, { key: 'tags', value: tag, type: 'list', listItems: [tag] } as FrontmatterProperty];
}

describe('PJ-182 — a zero-indent block list must project, and survive, intact', () => {
	it('THE ROOT — the projection carries every item', () => {
		const { properties } = parseFrontmatter(ZERO_INDENT);

		const tags = properties.find((p) => p.key === 'tags')!;
		expect(tags.type).toBe('list');
		expect(tags.listItems).toEqual(['alpha', 'beta']);

		const aliases = properties.find((p) => p.key === 'aliases')!;
		expect(aliases.type).toBe('list');
		expect(aliases.listItems).toEqual(['Old Name', 'Older Name']);

		// The keys AFTER the block are still ordinary top-level properties — the extent
		// scan must not swallow them, and no phantom key may appear.
		expect(properties.map((p) => p.key)).toEqual(['cid_cn', 'title', 'tags', 'aliases', 'stage']);
	});

	it('THE LOSS — adding a tag to an OPEN note keeps the items that were already there', () => {
		const { yaml, hadFence } = splitFrontmatter(ZERO_INDENT);
		const { properties, body } = parseFrontmatter(ZERO_INDENT);

		const out = composeFrontmatter(yaml, hadFence, properties, addTagToProps(properties, 'gamma'), body);
		const tags = parseFrontmatter(out).properties.find((p) => p.key === 'tags')!;

		// Observed before the fix: ['gamma'] — alpha and beta deleted from the .md.
		expect(tags.listItems).toEqual(['alpha', 'beta', 'gamma']);
		// And the untouched neighbour list must not be collateral damage.
		expect(parseFrontmatter(out).properties.find((p) => p.key === 'aliases')!.listItems).toEqual([
			'Old Name',
			'Older Name',
		]);
	});

	it('THE LOSS — adding a tag to a CLOSED note keeps them too', () => {
		const { properties } = parseFrontmatter(ZERO_INDENT);
		const out = composeUpdatedContent(ZERO_INDENT, addTagToProps(properties, 'gamma'));

		expect(parseFrontmatter(out).properties.find((p) => p.key === 'tags')!.listItems).toEqual([
			'alpha',
			'beta',
			'gamma',
		]);
	});

	it('THE CACHE — buildFullContent does not empty the block', () => {
		const { properties, body } = parseFrontmatter(ZERO_INDENT);
		const cached = buildFullContent(properties, body);

		// Observed before the fix: `tags: ` and `aliases: ` — both lists deleted outright.
		expect(cached).toMatch(/tags:\s*\n\s*-\s*alpha/);
		expect(cached).toContain('beta');
		expect(cached).toContain('Old Name');
		expect(cached).toContain('Older Name');

		const reparsed = parseFrontmatter(cached).properties;
		expect(reparsed.find((p) => p.key === 'tags')!.listItems).toEqual(['alpha', 'beta']);
		expect(reparsed.find((p) => p.key === 'aliases')!.listItems).toEqual(['Old Name', 'Older Name']);
	});

	it('CRLF — the same note with Windows line endings behaves identically', () => {
		const crlf = ZERO_INDENT.replace(/\n/g, '\r\n');
		const tags = parseFrontmatter(crlf).properties.find((p) => p.key === 'tags')!;
		expect(tags.listItems).toEqual(['alpha', 'beta']);
	});

	it('a single-item zero-indent list is a list, not an empty one', () => {
		const note = ['---', 'title: T', 'aliases:', '- Only One', '---', '', 'body', ''].join('\n');
		const aliases = parseFrontmatter(note).properties.find((p) => p.key === 'aliases')!;
		expect(aliases.listItems).toEqual(['Only One']);
	});

	/**
	 * `- name: X` at column 0 HAS a colon and does NOT start with a space, so the
	 * top-level-key guard admitted it as a key called `- name` — twice, once per row.
	 * `search.rs::parse_frontmatter` has carried the correct rule all along:
	 * "a line beginning `- ` is a LIST ITEM, never a key — in YAML a key cannot start
	 * with a sequence dash." The JS parser never had it.
	 */
	it('a zero-indent SEQ-OF-MAPS produces no phantom `- name` property', () => {
		const note = [
			'---',
			'title: T',
			'authors:',
			'- name: Ibn Khaldun',
			'  role: author',
			'- name: Rosenthal',
			'  role: translator',
			'stage: spark-seed',
			'---',
			'',
			'body',
			'',
		].join('\n');

		const { properties, body } = parseFrontmatter(note);
		expect(properties.map((p) => p.key)).toEqual(['title', 'authors', 'stage']);

		// Boss ruling 2026-07-22: what we cannot round-trip is READ-ONLY, with its bytes
		// carried verbatim — never an editable projection missing half its content.
		const authors = properties.find((p) => p.key === 'authors')!;
		expect(authors.type).toBe('nested-map');
		expect(authors.nestedRaw).toEqual([
			'- name: Ibn Khaldun',
			'  role: author',
			'- name: Rosenthal',
			'  role: translator',
		]);

		// And the cache keeps every continuation line.
		const cached = buildFullContent(properties, body);
		expect(cached).toContain('  role: author');
		expect(cached).toContain('  role: translator');
	});

	it('a zero-indent ikhtilāf block still parses as the structured type', () => {
		const note = [
			'---',
			'title: T',
			'ikhtilāf:',
			'- school: Hanafī',
			'  position: permissible',
			'- school: Mālikī',
			'  position: discouraged',
			'stage: spark-seed',
			'---',
			'',
			'body',
			'',
		].join('\n');

		const { properties } = parseFrontmatter(note);
		expect(properties.map((p) => p.key)).toEqual(['title', 'ikhtilāf', 'stage']);

		const k = properties.find((p) => p.key === 'ikhtilāf')!;
		expect(k.type).toBe('nested-object-list');
		expect(k.nestedObjects).toEqual([
			{ school: 'Hanafī', position: 'permissible' },
			{ school: 'Mālikī', position: 'discouraged' },
		]);
	});

	/**
	 * A comment inside the block does not cost the user their list.
	 *
	 * PJ-182 answered this by projecting the block READ-ONLY with its bytes verbatim, which
	 * kept the comment — and PJ-252 then found the price: `store.parseFrontmatter` called it
	 * read-only while the WRITE path's own classifier called it an ordinary editable list, so
	 * adding a tag deleted the tags already there. One classifier now answers both, and this
	 * shape is an editable list whose comment is carried across the rewrite by the composer.
	 *
	 * The assertion moves with it: the invariant was always "the comment survives", never
	 * "the row is read-only". It is asserted against `composeContent` — the path that reaches
	 * disk (`SINGLE_OWNERSHIP`/`USE_YAML_DOC` are both on) — because the legacy
	 * `buildFullContent` rebuild has never carried a comment. Verified by running, 2026-08-11:
	 * a standalone `# comment` between two top-level keys is dropped by that path today, with
	 * no PJ-252 change involved. That lossiness is why G4 replaced it.
	 */
	it('a comment inside a zero-indent block keeps its list AND its comment', () => {
		const note = ['---', 'title: T', 'tags:', '# a comment', '- alpha', 'stage: seed', '---', '', 'body', ''].join('\n');
		const { properties } = parseFrontmatter(note);

		const tags = properties.find((p) => p.key === 'tags')!;
		expect(tags.type).toBe('list');
		expect(tags.listItems).toEqual(['alpha']);
		expect(properties.map((p) => p.key)).toEqual(['title', 'tags', 'stage']);

		// Edit that very key — the destructive moment — and both survive.
		const edited = properties.map((p) =>
			p.key === 'tags' ? { ...p, listItems: ['alpha', 'beta'], value: 'alpha, beta' } : p,
		);
		const out = composeContent(parseFrontmatterDoc(note), properties, edited);
		expect(out).toContain('# a comment');
		expect(out).toContain('- alpha');
		expect(out).toContain('- beta');
	});

	// ── CONTROLS — the fix must not change any of these ────────────────────────
	describe('REGRESSION CONTROLS — the indented forms are untouched', () => {
		const INDENTED = [
			'---',
			'title: T',
			'tags:',
			'  - alpha',
			'  - beta',
			'stage: spark-seed',
			'---',
			'',
			'body',
			'',
		].join('\n');

		it('an ordinary indented list is still an editable list that round-trips', () => {
			const { properties, body } = parseFrontmatter(INDENTED);
			const { yaml, hadFence } = splitFrontmatter(INDENTED);
			const tags = properties.find((p) => p.key === 'tags')!;
			expect(tags.type).toBe('list');
			expect(tags.listItems).toEqual(['alpha', 'beta']);

			const out = composeFrontmatter(yaml, hadFence, properties, addTagToProps(properties, 'gamma'), body);
			expect(parseFrontmatter(out).properties.find((p) => p.key === 'tags')!.listItems).toEqual([
				'alpha',
				'beta',
				'gamma',
			]);
		});

		it('an indented seq-of-maps is still read-only with verbatim bytes', () => {
			const note = [
				'---',
				'title: T',
				'authors:',
				'  - name: X',
				'    role: Y',
				'---',
				'',
				'body',
				'',
			].join('\n');
			const authors = parseFrontmatter(note).properties.find((p) => p.key === 'authors')!;
			expect(authors.type).toBe('nested-map');
			expect(authors.nestedRaw).toEqual(['  - name: X', '    role: Y']);
		});

		it('an indented nested MAP is still read-only with verbatim bytes', () => {
			const note = [
				'---',
				'title: T',
				'source:',
				'  title: Muqaddimah',
				'  author: Ibn Khaldun',
				'---',
				'',
				'body',
				'',
			].join('\n');
			const src = parseFrontmatter(note).properties.find((p) => p.key === 'source')!;
			expect(src.type).toBe('nested-map');
			expect(src.nestedKeys).toEqual(['title', 'author']);
			expect(src.nestedRaw).toEqual(['  title: Muqaddimah', '  author: Ibn Khaldun']);
		});

		it('editing a NEIGHBOURING key still leaves a zero-indent block byte-identical', () => {
			const { yaml, hadFence } = splitFrontmatter(ZERO_INDENT);
			const { properties, body } = parseFrontmatter(ZERO_INDENT);
			const edited = properties.map((p) => (p.key === 'stage' ? { ...p, value: 'growth-seed' } : p));
			const out = composeFrontmatter(yaml, hadFence, properties, edited, body);

			expect(out).toContain('tags:\n- alpha\n- beta\n');
			expect(out).toContain('aliases:\n- Old Name\n- Older Name\n');
			expect(out).toContain('stage: growth-seed');
		});

		it('a list item that merely CONTAINS a colon stays an editable scalar (zero-indent too)', () => {
			const note = ['---', 'links:', '- https://example.com/x', '- "a: b"', '---', '', 'body', ''].join('\n');
			const links = parseFrontmatter(note).properties.find((p) => p.key === 'links')!;
			expect(links.type).toBe('list');
			expect(links.listItems).toEqual(['https://example.com/x', 'a: b']);
		});

		it('an ordinary scalar key whose value starts with a dash is not mistaken for a list', () => {
			const note = ['---', 'title: -- dashes --', 'stage: seed', '---', '', 'body', ''].join('\n');
			const props = parseFrontmatter(note).properties;
			expect(props.find((p) => p.key === 'title')!.value).toBe('-- dashes --');
			expect(props.find((p) => p.key === 'title')!.type).toBe('text');
		});
	});
});

/**
 * 2026-07-24 SAFETY INSPECTION — APP-KILLER (content-loss), the THIRD shape of the
 * class PJ-136 and MIG-101 already closed twice.
 *
 * `store.parseFrontmatter`'s multi-line-list branch consumed ONLY consecutive
 * `- item` lines. Two very common, perfectly valid YAML shapes therefore reached the
 * property panel TRUNCATED:
 *
 *   authors:              seq-of-maps — `    role: Y` is not a `- ` item, so the scan
 *     - name: X           stopped there, and the outer `!line.startsWith(' ')` guard
 *       role: Y           then skipped that line entirely.
 *
 *   tags:                 a block list with an interior blank line — everything after
 *     - a                 the blank was dropped the same way.
 *
 *     - b
 *
 * A truncated projection of an EDITABLE type is not a display bug, it is a
 * data-destroying invitation: the panel drew `authors` as an editable chip list
 * showing one chip and no hint that `role: Y` existed, and the moment the user
 * added or removed a chip, `composeFrontmatter` spliced the whole block-seq out of
 * the CST and rewrote it from the truncated projection. `role: Y` was gone from the
 * .md with no error, the save set the disk baseline so the watcher saw its own echo,
 * and the rewritten YAML re-parsed cleanly — permanent, silent, unnoticeable.
 *
 * Constellation reads Obsidian vaults IN PLACE and seq-of-maps frontmatter is
 * ordinary there (citation/plugin-authored `authors:` / `sources:` blocks).
 *
 * Two independent fixes, either of which breaks the chain; both are kept:
 *   A. `immutableBlockKeys` (yamlDoc) treats a seq holding ANY non-scalar item as a
 *      block compose must never rewrite — read from the FILE, so it holds however
 *      any upstream projection behaves.
 *   B. `parseFrontmatter` takes the block's FULL extent and, when it cannot
 *      round-trip it as a flat list, projects it READ-ONLY (`nested-map`) with its
 *      bytes verbatim in `nestedRaw` — the Boss's 2026-07-22 rule: visible and
 *      honest, never editable.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter, buildFullContent } from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter } from '$lib/editor/yamlDoc';

const SEQ_OF_MAPS = [
	'---',
	'title: Citation Probe',
	'authors:',
	'  - name: Ibn Khaldun',
	'    role: author',
	'  - name: Rosenthal',
	'    role: translator',
	'stage: spark-seed',
	'---',
	'',
	'body text',
	'',
].join('\n');

const BLANK_LINE_LIST = [
	'---',
	'title: Spaced List Probe',
	'tags:',
	'  - alpha',
	'',
	'  - beta',
	'stage: spark-seed',
	'---',
	'',
	'body text',
	'',
].join('\n');

const ORDINARY_LIST = [
	'---',
	'title: Ordinary Probe',
	'tags:',
	'  - alpha',
	'  - beta',
	'stage: spark-seed',
	'---',
	'',
	'body text',
	'',
].join('\n');

describe('APP-KILLER 2026-07-24 — seq-of-maps / spaced block lists are never truncated', () => {
	it('FIX B — a seq-of-maps is projected READ-ONLY, not as a truncated chip list', () => {
		const authors = parseFrontmatter(SEQ_OF_MAPS).properties.find((p) => p.key === 'authors')!;

		// The bug: type 'list' with listItems ['name: Ibn Khaldun', 'name: Rosenthal'] —
		// editable chips, and both `role:` lines silently absent.
		expect(authors.type).toBe('nested-map');
		expect(authors.listItems).toBeUndefined();

		// Every byte of the block is carried verbatim.
		expect(authors.nestedRaw).toEqual([
			'  - name: Ibn Khaldun',
			'    role: author',
			'  - name: Rosenthal',
			'    role: translator',
		]);

		// The keys that follow the block are still parsed as top-level properties —
		// the extent scan must not swallow them.
		const keys = parseFrontmatter(SEQ_OF_MAPS).properties.map((p) => p.key);
		expect(keys).toEqual(['title', 'authors', 'stage']);
	});

	it('FIX B — the tab.content cache keeps every continuation line', () => {
		const { properties, body } = parseFrontmatter(SEQ_OF_MAPS);
		const cached = buildFullContent(properties, body);

		expect(cached).toContain('  - name: Ibn Khaldun');
		expect(cached).toContain('    role: author');
		expect(cached).toContain('  - name: Rosenthal');
		expect(cached).toContain('    role: translator');

		// Re-parsing the cache still recognises the type (no silent revert to editable).
		const reparsed = parseFrontmatter(cached).properties.find((p) => p.key === 'authors')!;
		expect(reparsed.type).toBe('nested-map');
	});

	it('FIX A — even a props array CLAIMING an editable list cannot destroy the block', () => {
		const { yaml, hadFence } = splitFrontmatter(SEQ_OF_MAPS);
		const { properties: base, body } = parseFrontmatter(SEQ_OF_MAPS);

		// Exactly what the OLD parser produced, plus the chip the user would add.
		const lying = base.map((p) =>
			p.key === 'authors'
				? {
						key: 'authors',
						value: 'name: Ibn Khaldun, name: Rosenthal',
						type: 'list' as const,
						listItems: ['name: Ibn Khaldun', 'name: Rosenthal'],
					}
				: p,
		);
		const edited = lying.map((p) =>
			p.key === 'authors'
				? { ...p, listItems: [...(p.listItems ?? []), 'Someone New'] }
				: p,
		);

		const out = composeFrontmatter(yaml, hadFence, lying, edited, body);

		expect(out).toContain('    role: author');
		expect(out).toContain('    role: translator');
		expect(out).not.toContain('- "name: Ibn Khaldun"');
	});

	it('an ordinary edit to a NEIGHBOURING property leaves the block byte-intact', () => {
		const { yaml, hadFence } = splitFrontmatter(SEQ_OF_MAPS);
		const { properties: base, body } = parseFrontmatter(SEQ_OF_MAPS);

		const edited = base.map((p) => (p.key === 'stage' ? { ...p, value: 'growth-seed' } : p));
		const out = composeFrontmatter(yaml, hadFence, base, edited, body);

		expect(out).toContain('  - name: Ibn Khaldun');
		expect(out).toContain('    role: author');
		expect(out).toContain('  - name: Rosenthal');
		expect(out).toContain('    role: translator');
		expect(out).toContain('stage: growth-seed'); // the real edit still lands
	});

	/**
	 * The blank-line block list stays an ordinary EDITABLE list — the extent scan now
	 * reaches past the blank, so both items are captured and neither can be written
	 * out of the file. (The blank line itself normalises away if the user edits the
	 * key; that is whitespace, not data. Making every spaced tags: list read-only
	 * would be a needless regression — the app-killer was the dropped ITEM.)
	 */
	it('a block list with an interior blank line keeps every item', () => {
		const { properties, body } = parseFrontmatter(BLANK_LINE_LIST);
		const tags = properties.find((p) => p.key === 'tags')!;

		// The bug captured only 'alpha'; the next tag edit then wrote 'beta' out of the file.
		expect(tags.listItems).toEqual(['alpha', 'beta']);

		const cached = buildFullContent(properties, body);
		expect(cached).toContain('  - alpha');
		expect(cached).toContain('  - beta');

		expect(properties.map((p) => p.key)).toEqual(['title', 'tags', 'stage']);

		// And the edit that used to destroy 'beta' now preserves it.
		const { yaml, hadFence } = splitFrontmatter(BLANK_LINE_LIST);
		const edited = properties.map((p) =>
			p.key === 'tags'
				? { ...p, listItems: ['alpha', 'beta', 'gamma'], value: 'alpha, beta, gamma' }
				: p,
		);
		const out = composeFrontmatter(yaml, hadFence, properties, edited, body);
		expect(parseFrontmatter(out).properties.find((p) => p.key === 'tags')!.listItems).toEqual([
			'alpha',
			'beta',
			'gamma',
		]);
	});

	/**
	 * The guard on the guard: an ORDINARY flat list must stay fully editable, parse
	 * identically to before, and still round-trip through a real chip edit. The fix
	 * would be worthless if it made every tags: list read-only.
	 */
	it('REGRESSION — an ordinary flat list is still an editable list and still round-trips', () => {
		const { yaml, hadFence } = splitFrontmatter(ORDINARY_LIST);
		const { properties: base, body } = parseFrontmatter(ORDINARY_LIST);
		const tags = base.find((p) => p.key === 'tags')!;

		expect(tags.type).toBe('list');
		expect(tags.listItems).toEqual(['alpha', 'beta']);

		const edited = base.map((p) =>
			p.key === 'tags'
				? { ...p, listItems: ['alpha', 'beta', 'gamma'], value: 'alpha, beta, gamma' }
				: p,
		);
		const out = composeFrontmatter(yaml, hadFence, base, edited, body);

		expect(out).toContain('gamma');
		expect(parseFrontmatter(out).properties.find((p) => p.key === 'tags')!.listItems).toEqual([
			'alpha',
			'beta',
			'gamma',
		]);
	});

	it('REGRESSION — list items that merely CONTAIN a colon stay editable scalars', () => {
		// `https://…` and quoted `"a: b"` are scalars, not map entries. YAML's own rule
		// (colon + space/EOL on an unquoted item) is what the detector uses.
		const note = [
			'---',
			'links:',
			'  - https://example.com/x',
			'  - "a: b"',
			'---',
			'',
			'body',
			'',
		].join('\n');
		const links = parseFrontmatter(note).properties.find((p) => p.key === 'links')!;
		expect(links.type).toBe('list');
		expect(links.listItems).toEqual(['https://example.com/x', 'a: b']);
	});
});

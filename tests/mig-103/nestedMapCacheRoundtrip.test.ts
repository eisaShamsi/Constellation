/**
 * PJ-136 REGRESSION — the app-killer I introduced in commit 98b71440 and the
 * 2026-07-22 inspection caught before any user hit it.
 *
 * The original PJ-136 fix filtered `nested-map` out of BOTH sides of
 * `composeFrontmatter`'s diff and then checked `immutableKeys` only in the REMOVE
 * loop. That is safe exactly as long as the props array always reports the type
 * honestly — and the app itself manufactures a props array that does not:
 *
 *   1. PropertyEditor caches `tab.content = buildFullContent(props, body)`.
 *      `reconstructFrontmatter` wrote a nested block as a bare `source:` and
 *      dropped its children — the CACHE is lossy (disk was still correct).
 *   2. Any re-derive from that cache (`parseFrontmatter(tab.content)` on a tab
 *      emission, a switch-away-and-back, a remount) re-projects `source` as
 *      ordinary EMPTY TEXT, and PropertyEditor's re-sync adopts it.
 *   3. The next edit of ANY property reaches compose with `source` absent from the
 *      old side (filtered as nested-map) and present on the new side (as text) →
 *      `op === undefined` → the SET/ADD branch spliced the block out of the CST
 *      and appended `source: ""`. Durable, silent, unrecoverable.
 *
 * Two independent fixes, either of which breaks the chain; both are kept:
 *   A. `composeFrontmatter` reads the immutable set from the FILE, not the props.
 *   B. `reconstructFrontmatter` re-emits the block verbatim, so the cache is lossless.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter, buildFullContent } from '$lib/libraries/store';
import { splitFrontmatter, composeFrontmatter } from '$lib/editor/yamlDoc';

const NOTE = [
	'---',
	'title: Nested Property Probe v2',
	'source:',
	'  title: Muqaddimah',
	'  author: Ibn Khaldun',
	'  year: 1377',
	'stage: spark-seed',
	'---',
	'',
	'body text',
	'',
].join('\n');

describe('PJ-136 regression — the lossy-cache chain', () => {
	it('FIX B — the tab.content cache keeps the nested block intact', () => {
		const { properties, body } = parseFrontmatter(NOTE);
		const cached = buildFullContent(properties, body);

		expect(cached).toContain('  title: Muqaddimah');
		expect(cached).toContain('  author: Ibn Khaldun');
		expect(cached).toContain('  year: 1377');

		// ...and re-parsing the cache still recognises the type, so the panel does not
		// silently revert to the editable "Empty" row it used to show.
		const reparsed = parseFrontmatter(cached).properties.find((p) => p.key === 'source');
		expect(reparsed!.type).toBe('nested-map');
		expect(reparsed!.nestedKeys).toEqual(['title', 'author', 'year']);
	});

	/**
	 * FIX A, proven independently of FIX B: hand compose the exact hostile input the
	 * lossy cache used to produce — `source` typed as empty TEXT on the new side while
	 * the base still holds the real block — and require the block to survive.
	 */
	it('FIX A — a props array that MISREPORTS the type cannot delete the block', () => {
		const { yaml, hadFence } = splitFrontmatter(NOTE);
		const { properties: base, body } = parseFrontmatter(NOTE);

		const lying = base.map((p) =>
			p.key === 'source'
				? { key: 'source', value: '', type: 'text' as const } // what the old cache yielded
				: p,
		);
		// ...plus a genuine edit to some OTHER property, which is what used to trigger it.
		const edited = lying.map((p) => (p.key === 'stage' ? { ...p, value: 'growth-seed' } : p));

		const out = composeFrontmatter(yaml, hadFence, base, edited, body);

		expect(out).toContain('  title: Muqaddimah');
		expect(out).toContain('  author: Ibn Khaldun');
		expect(out).toContain('  year: 1377');
		expect(out).not.toContain('source: ""');
		expect(out).toContain('stage: growth-seed'); // the real edit still lands
	});

	it('the full two-save chain leaves the block byte-intact on disk', () => {
		const { yaml, hadFence } = splitFrontmatter(NOTE);
		const { properties: base, body } = parseFrontmatter(NOTE);

		// Save #1 — edit a property; the panel then caches tab.content.
		const edit1 = base.map((p) => (p.key === 'stage' ? { ...p, value: 'growth-seed' } : p));
		const disk1 = composeFrontmatter(yaml, hadFence, base, edit1, body);
		const cache = buildFullContent(edit1, body);

		// Re-derive the props from the cache, exactly as NoteEditor does.
		const rederived = parseFrontmatter(cache).properties;

		// Save #2 — another ordinary edit, now working from the re-derived props.
		const { yaml: yaml1 } = splitFrontmatter(disk1);
		const edit2 = rederived.map((p) => (p.key === 'title' ? { ...p, value: 'Renamed' } : p));
		const disk2 = composeFrontmatter(yaml1, true, rederived, edit2, body);

		expect(disk2).toContain('  title: Muqaddimah');
		expect(disk2).toContain('  author: Ibn Khaldun');
		expect(disk2).toContain('  year: 1377');
	});
});

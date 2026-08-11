/**
 * PJ-249 §6i — **KNOWN, UNFIXED DEFECT.** Adding a tag to a note whose frontmatter list
 * carries a comment line, or an item wrapped across two lines, DELETES the entries that
 * were already in that list, from the `.md` file, with no error.
 *
 * These are `it.fails` — vitest reports them green **while the defect exists**. That is
 * deliberate and it is a debt marker, not a pass: **the moment §6i is fixed these turn RED**,
 * and whoever fixes it must flip them to `it`. A reproduction is worth more committed than
 * described, and the Reproduce-First rule wants it to exist before the fix does.
 *
 * ── The mechanism (both halves verified in source, then by running) ──────────────────────
 *
 * Two classifiers decide, for the same block, whether it is safe to edit — and they disagree:
 *
 *  - `store.parseFrontmatter` works on LINES. `blockExtent` / `isYamlBlockChild` pull a `#`
 *    comment (or a wrapped continuation) into the block, then require EVERY content line to
 *    be a bare `- item` before projecting an editable `list`. A comment fails that, so the
 *    key is projected READ-ONLY, with an EMPTY `value` and no `listItems`.
 *
 *  - `yamlDoc.immutableBlockKeys` asks the `yaml` library, which attaches the comment as a
 *    comment and folds a wrapped item into ONE scalar — so the seq is "all scalars", not a
 *    block, and NOT protected.
 *
 * Every list mutator then rebuilds from `p.listItems ?? (p.value ? split : [])`, which on
 * that read-only projection is `[]`. The block is spliced out and re-appended holding only
 * the new item.
 *
 * ── Exposure, measured rather than assumed ──────────────────────────────────────────────
 *
 * 10,077 `.md` files scanned across both live universes (2026-08-10): **1** matches the
 * shape — a probe note, on `authors`. Real defect, real class, negligible live exposure.
 *
 * ── The fix this is waiting for ─────────────────────────────────────────────────────────
 *
 * ONE predicate, shared. `yamlDoc`'s own comment already states the standard —
 * *"refusing here means the block survives however the panel behaves"* — and the 2026-07-24
 * inspection closed this shape for seq-of-maps, PJ-182 for block scalars. The comment-bearing
 * and wrapped-item scalar seqs are the shapes still open, and they are open precisely because
 * each closure re-answered the question in a second place instead of routing to one answer.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter } from '$lib/libraries/store';
import { composeContent, parseFrontmatterDoc } from '$lib/editor/yamlDoc';
import type { FrontmatterProperty } from '$lib/libraries/store';

/** `addTagToProps`, verbatim from `+layout.svelte:7016` — the real mutator, not a stand-in. */
function addTagToProps(props: FrontmatterProperty[], tag: string): FrontmatterProperty[] {
	const idx = props.findIndex((p) => p.key.toLowerCase() === 'tags');
	if (idx >= 0) {
		const p = props[idx];
		const existing =
			p.listItems ??
			(p.value
				? String(p.value)
						.split(',')
						.map((s) => s.trim())
						.filter(Boolean)
				: []);
		if (existing.some((x) => x.toLowerCase() === tag.toLowerCase())) return props;
		const items = [...existing, tag];
		return props.map((q, i) =>
			i === idx ? { ...q, type: 'list', listItems: items, value: items.join(', ') } : q
		);
	}
	return [
		...props,
		{ key: 'tags', value: tag, type: 'list', listItems: [tag] } as FrontmatterProperty
	];
}

/** The whole Add-tag path for a closed note, as the app runs it. */
function addTag(content: string, tag: string): string {
	const { properties } = parseFrontmatter(content);
	return composeContent(parseFrontmatterDoc(content), properties, addTagToProps(properties, tag));
}

const SHAPES: Array<[string, string, string]> = [
	[
		'a comment line after the block',
		'history',
		`---
title: T
tags:
  - history
  - islam
# a note to self about this file
cid_cn: 20260101T000000Z_NOTE_0001
---
body
`
	],
	[
		'a comment line inside the block',
		'history',
		`---
title: T
tags:
  # personal taxonomy
  - history
  - islam
cid_cn: 20260101T000000Z_NOTE_0001
---
body
`
	],
	[
		'a plain scalar item wrapped across two lines',
		'a very long tag that',
		`---
title: T
tags:
  - a very long tag that
    wraps to a second line
  - islam
cid_cn: 20260101T000000Z_NOTE_0001
---
body
`
	]
];

describe('PJ-249 §6i — adding a tag must never delete the tags already there', () => {
	for (const [shape, survivor, content] of SHAPES) {
		// `it.fails` — GREEN MEANS THE BUG IS STILL HERE. Flip to `it` when §6i lands.
		it.fails(`KNOWN DEFECT (unfixed): tags are destroyed when the block has ${shape}`, () => {
			const out = addTag(content, 'newtag');
			expect(out).toContain('newtag');
			expect(out).toContain('islam');
			expect(out).toContain(survivor);
		});
	}

	it('the ordinary all-scalar block still round-trips (the fix must not over-refuse)', () => {
		// The other half of the contract, and the reason the fix is a shared predicate rather
		// than "refuse every seq": the everyday tags list must stay editable.
		const plain = `---
title: T
tags:
  - history
  - islam
---
body
`;
		const out = addTag(plain, 'newtag');
		expect(out).toContain('history');
		expect(out).toContain('islam');
		expect(out).toContain('newtag');
	});
});

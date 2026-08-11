/**
 * PJ-252 (was PJ-249 §6i) — **FIXED.** Adding a tag to a note whose frontmatter list carried
 * a comment line, or an item wrapped across two lines, DELETED the entries that were already
 * in that list, from the `.md` file, with no error and a clean re-parse afterwards.
 *
 * These three were committed as `it.fails` while the defect was live — a reproduction is
 * worth more committed than described, and the Reproduce-First rule wants it to exist before
 * the fix does. They are `it` now: **they fail if the defect ever comes back.**
 *
 * ── The mechanism (both halves verified in source, then by running) ──────────────────────
 *
 * Two classifiers decided, for the same block, whether it was safe to edit — and they
 * disagreed:
 *
 *  - `store.parseFrontmatter` worked on LINES. `blockExtent` / `isYamlBlockChild` pulled a `#`
 *    comment (or a wrapped continuation) into the block, then required EVERY content line to
 *    be a bare `- item` before projecting an editable `list`. A comment fails that, so the key
 *    was projected READ-ONLY, with an EMPTY `value` and no `listItems`.
 *
 *  - `yamlDoc.immutableBlockKeys` asked the `yaml` library, which attaches the comment as a
 *    comment and folds a wrapped item into ONE scalar — so the seq was "all scalars", not a
 *    block, and NOT protected.
 *
 * Every list mutator then rebuilt from `p.listItems ?? (p.value ? split : [])`, which on that
 * read-only projection is `[]`. The block was spliced out and re-appended holding only the
 * new item.
 *
 * ── The fix ─────────────────────────────────────────────────────────────────────────────
 *
 * ONE classifier — `yamlDoc.classifyFrontmatterValues` — answers "what is this key's value?"
 * for the projection, for the write path's refusal, and for `projectProps`. This was the
 * FOURTH shape of one defect (2026-07-24 closed seq-of-maps, PJ-182 closed block scalars),
 * and it existed because each closure re-answered the question in a second place. There is no
 * longer a second opinion to disagree with.
 *
 * ── Exposure, measured rather than assumed ──────────────────────────────────────────────
 *
 * 10,077 `.md` files scanned across both live universes (2026-08-10): **1** matched the
 * shape — a probe note, on `authors`. Real defect, real class, negligible live exposure.
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
		'a very long tag that wraps to a second line',
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

describe('PJ-252 — adding a tag must never delete the tags already there', () => {
	for (const [shape, survivor, content] of SHAPES) {
		it(`the tags survive when the block has ${shape}`, () => {
			const out = addTag(content, 'newtag');
			expect(out).toContain('newtag');
			expect(out).toContain('islam');
			expect(out).toContain(survivor);
		});
	}

	it('the ordinary all-scalar block still round-trips (the fix must not over-refuse)', () => {
		// The other half of the contract, and the reason the fix is a shared classifier rather
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

	// ── Two more shapes of the same disagreement, found by RUNNING the path while fixing it ──

	it('an inline comment on an item is not swallowed into the tag value', () => {
		// The line scanner read `- history   # why` as a tag literally named `history   # why`
		// and wrote it back as one, quoted. The library reads the value and the comment apart.
		const content = `---
title: T
tags:
  - history   # why
  - islam
---
body
`;
		const { properties } = parseFrontmatter(content);
		expect(properties.find((p) => p.key === 'tags')?.listItems).toEqual(['history', 'islam']);

		const out = addTag(content, 'newtag');
		expect(out).toContain('- history # why'); // value AND comment, still apart
		expect(out).not.toContain('"history'); // never re-quoted as one scalar
		expect(out).toContain('- islam');
		expect(out).toContain('- newtag');
	});

	it("an inline flow list under an unrecognised key is a list, not the string 'a, b'", () => {
		// `detectPropertyType` types by KEY, so a flow sequence under a key it does not know
		// came back as `text` valued `a, b` — and editing the note wrote the sequence to disk
		// as that string.
		const content = `---
title: T
whatever: [alpha, beta]
---
body
`;
		const p = parseFrontmatter(content).properties.find((x) => x.key === 'whatever');
		expect(p?.type).toBe('list');
		expect(p?.listItems).toEqual(['alpha', 'beta']);
	});

	// ── The regression the fix itself nearly shipped ────────────────────────────────────────
	describe('a Windows (CRLF) note', () => {
		// Routing the projection through the `yaml` library made HOW the YAML text is extracted
		// load-bearing for the first time. `parseFrontmatter`'s own `rawYaml` is
		// `yamlLines.join('\n')`, and on a CRLF note every line still ends in `\r`; `join` puts a
		// separator only BETWEEN elements, so the LAST line's `\r` is unterminated and the library
		// reads it as DATA. The final property came back as `snurfle\r` and was written back as
		// the quoted tag `"snurfle\r"` — on an ordinary Notepad-saved note with a plain tags list,
		// which is far commoner than any shape this PJ set out to fix. Caught by the ui-inspector
		// gate before it reached the Boss; the classifier is handed `splitFrontmatter`'s text now,
		// the same bytes the writer composes from.
		const crlf = (s: string) => s.replace(/\n/g, '\r\n');

		for (const [name, body] of [
			['plain list', 'tags:\n  - blorptide\n  - snurfle'],
			['list with a comment', 'tags:\n  # personal taxonomy\n  - blorptide\n  - snurfle'],
		] as Array<[string, string]>) {
			it(`keeps no carriage return in the last property's value — ${name}`, () => {
				const content = crlf(`---\ntitle: T\n${body}\n---\nbody here\n`);
				const { properties } = parseFrontmatter(content);
				expect(properties.find((p) => p.key === 'tags')?.listItems).toEqual([
					'blorptide',
					'snurfle',
				]);

				const out = addTag(content, 'quixolite');
				expect(out).not.toContain('\\r'); // never re-quoted with an escaped CR
				expect(out).toContain('- snurfle\r\n');
				expect(out).toContain('- quixolite');
				expect(out.split('\n').every((l) => l === '' || l.endsWith('\r'))).toBe(true); // EOL intact
			});
		}
	});

	describe('a rewrite must not leave a blank line where it spliced', () => {
		// Splicing the block's FIRST item left its line break behind, so a note whose edited key
		// was its first property gained a blank line under the opening `---` on EVERY edit.
		// Pre-existing — measured at HEAD, byte-identical output — and the twin of the blank line
		// `ensure_cid_cn` was leaving in the Rust writer (canonical.rs, fixed the same day). One
		// concern, two surfaces.
		it('no blank line appears above a first-property list that was edited', () => {
			const out = addTag(`---\ntags:\n  - alpha\n  - beta\n---\nbody\n`, 'newtag');
			expect(out.startsWith('---\ntags:')).toBe(true);
			expect(out).toContain('- alpha');
			expect(out).toContain('- newtag');
		});

		it('a blank line the USER typed is preserved', () => {
			// The condition is what keeps the write byte-perfect: only a break the splice invented
			// is removed. This one is in the file, so it stays.
			const out = addTag(`---\n\ntags:\n  - alpha\n---\nbody\n`, 'newtag');
			expect(out.startsWith('---\n\n')).toBe(true);
			expect(out).toContain('- alpha');
			expect(out).toContain('- newtag');
		});
	});

	it('a comment the user wrote inside an edited list survives the rewrite', () => {
		// Making those lists editable must not trade a destroyed list for a destroyed comment.
		const content = `---
title: T
tags:
  # personal taxonomy
  - history
  - islam
---
body
`;
		expect(addTag(content, 'newtag')).toContain('# personal taxonomy');
	});

	it('a key written TWICE is refused wholesale rather than merged', () => {
		// The classifier and the compose diff are both keyed by NAME, so a rewrite would splice
		// the first block and append a merge of both. The `yaml` library calls this file
		// unparseable ("Map keys must be unique"), which routes it to the H1 passthrough — the
		// frontmatter bytes are re-emitted verbatim and the tag add is refused, not applied
		// destructively. Pinned because it is the safe outcome arriving by a route that is easy
		// to break: it depends on the library treating this as an ERROR, not a warning.
		const content = `---
tags:
  - history
tags:
  - islam
---
body
`;
		const out = addTag(content, 'newtag');
		expect(out).toContain('- history');
		expect(out).toContain('- islam');
		expect(out).not.toContain('newtag'); // refused, and the user is told by the save banner
	});
});

describe('PJ-252 — the block shapes earlier closures fixed must stay refused', () => {
	const cases: Array<[string, string]> = [
		[
			'seq-of-maps (2026-07-24 inspection)',
			`---
authors:
  - name: X
    role: Y
---
body
`
		],
		[
			'block scalar (PJ-182)',
			`---
desc: |
  first prose line
  second prose line
---
body
`
		],
		[
			'nested map (PJ-136)',
			`---
source:
  title: X
  year: 2020
---
body
`
		]
	];
	for (const [name, content] of cases) {
		it(`${name} stays read-only, and its bytes survive an unrelated edit`, () => {
			const { properties } = parseFrontmatter(content);
			expect(properties[0].type).toBe('nested-map');
			// Add an unrelated key — the block must come out byte-identical.
			const out = composeContent(parseFrontmatterDoc(content), properties, [
				...properties,
				{ key: 'stage', value: 'seed', type: 'text' } as FrontmatterProperty
			]);
			for (const line of content.split('\n').slice(1, -3)) expect(out).toContain(line);
		});
	}
});

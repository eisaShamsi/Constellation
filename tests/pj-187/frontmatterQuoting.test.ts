/**
 * PJ-187 (register) — TWO places hand-built a `key: value` frontmatter line by
 * concatenation and emitted invalid YAML for any value that needs quoting.
 *
 *   · `store.rebrandCopyFrontmatter` — the "Save a copy" LAST-RESORT recovery, used when a
 *     note's own file stays locked. It stripped the quotes off `title:` in its own match and
 *     re-emitted the value bare.
 *   · `+layout.svelte`'s template merge — `.map(p => \`${p.key}: ${p.value}\`)` over the
 *     canonical fields, so creating a note inside a folder with a template produced a
 *     malformed property block from birth.
 *
 * Measured before the fix, with the repo's own `yaml` parser:
 *   `"Plan: phase two"`  → 1 parse error; the title decodes as the MAP {Plan: "phase two …"}
 *   `"#hashtag start"`   → parses clean, but the value decodes as NULL (`#` starts a comment)
 *   `"- dash lead"`      → 2 parse errors
 *   `"[bracket]"`        → 1 parse error; the title decodes as the ARRAY ["bracket"]
 *
 * The quoter already existed — `quoteIfNeeded`, used by `reconstructFrontmatter` — it was
 * simply private. Both sites now use it. That is the same shape as PJ-182's LL-039: the rule
 * was written down in one of the places that needed it.
 */
import { describe, it, expect } from 'vitest';
import { parseDocument } from 'yaml';
import { quoteIfNeeded, parseFrontmatter } from '$lib/libraries/store';

/** The exact line-building both call sites now do. */
const line = (key: string, value: string) => `${key}: ${quoteIfNeeded(value)}`;

const HAZARDS = [
	'Plan: phase two',      // colon+space → a nested mapping
	'#hashtag start',       // leading # → a comment, value becomes null
	'- dash lead',          // leading dash → a sequence
	'[bracket]',            // flow sequence
	'{brace}',              // flow mapping
	'yes',                  // a YAML boolean-ish scalar
	'null',
	'He said "hi"',         // embedded quotes
	'trailing colon:',
	'@at-start',
	'*anchor',
	'&ref',
	'100%',
];

describe('PJ-187 — a hand-built frontmatter line must survive the YAML parser', () => {
	it.each(HAZARDS)('emits parseable YAML for a title containing %j', (value) => {
		const doc = parseDocument(`${line('title', value)}\n`);
		expect(doc.errors).toHaveLength(0);
		// …and round-trips to the SAME STRING — not a map, array, or null.
		expect(doc.get('title')).toBe(value);
	});

	it('the recovery-copy title survives — the whole point of Save a copy', () => {
		// Exactly what rebrandCopyFrontmatter now builds.
		for (const base of HAZARDS) {
			const rebuilt = line('title', `${base} (recovered copy)`);
			const doc = parseDocument(`${rebuilt}\n`);
			expect(doc.errors, `parse errors for ${base}`).toHaveLength(0);
			expect(doc.get('title')).toBe(`${base} (recovered copy)`);
		}
	});

	it('a full frontmatter block built this way re-reads through the app\'s own parser', () => {
		const fm = [
			'---',
			line('title', 'Plan: phase two (recovered copy)'),
			line('kind', 'note'),
			line('cid_cn', '20260729T000000Z_NOTE_0001'),
			'---',
			'',
			'body',
			'',
		].join('\n');

		expect(parseDocument(fm.split('---')[1]).errors).toHaveLength(0);
		const props = parseFrontmatter(fm).properties;
		expect(props.find((p) => p.key === 'title')!.value).toBe('Plan: phase two (recovered copy)');
		expect(props.find((p) => p.key === 'kind')!.value).toBe('note');
	});

	it('REGRESSION — an ordinary value is still emitted unquoted', () => {
		expect(line('title', 'Ordinary Title')).toBe('title: Ordinary Title');
		expect(line('kind', 'note')).toBe('kind: note');
	});
});

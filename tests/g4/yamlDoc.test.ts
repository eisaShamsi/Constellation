/**
 * G4 Phase 1 — the yamlDoc module (round-trip-safe frontmatter authority),
 * proven GREEN in isolation before the live noteModel swap (Phase 2).
 *
 * These are the same two app-killers as the Phase-0 RED harness, now asserted
 * against yamlDoc: Recipe A (nested map + block scalar must survive) and
 * Recipe B (embedded quote must not accumulate backslashes) — plus the
 * byte-perfect (untouched-key) and H1 (malformed-input) hardening requirements.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatterDoc, composeContent, splitFrontmatter } from '$lib/editor/yamlDoc';
import type { FrontmatterProperty } from '$lib/libraries/store';

const RECIPE_A = [
	'---',
	'title: Round Trip Probe',
	'cid_cn: 20260707T120000Z_NOTE_0001',
	'kind: note',
	'created: 2026-07-07T12:00:00+00:00',
	'source:',
	'    author: Ibn Khaldun', // 4-space indent — byte-fidelity probe
	'    year: 1377',
	'description: |',
	'  First line of a long note.',
	'  Second line stays too.',
	'tags:',
	'  - history',
	'---',
	'',
	'Body text.',
	'',
].join('\n');

describe('G4 Recipe A (yamlDoc) — nested map + block scalar survive', () => {
	it('a no-op round-trip preserves the nested map, block scalar, and body', () => {
		const fm = parseFrontmatterDoc(RECIPE_A);
		const out = composeContent(fm, fm.props, fm.props);
		expect(out).toContain('author: Ibn Khaldun');
		expect(out).toContain('year: 1377');
		expect(out).toContain('First line of a long note.');
		expect(out).toContain('Second line stays too.');
		expect(out).not.toContain('description: "|"'); // the corruption signature (block → quoted string)
		expect(out).toContain('Body text.');
	});

	it('is BYTE-EXACT on a no-op round-trip (byte-perfect tier)', () => {
		const fm = parseFrontmatterDoc(RECIPE_A);
		const out = composeContent(fm, fm.props, fm.props);
		expect(out).toBe(RECIPE_A);
	});

	it('editing ONE scalar keeps every untouched key byte-perfect (4-space map, block scalar, comment)', () => {
		const fm = parseFrontmatterDoc(RECIPE_A);
		const titleProp = fm.props.find((p) => p.key === 'title')!;
		const newProps = fm.props.map((p) => (p.key === 'title' ? { ...p, value: 'Edited Title' } : p));
		const out = composeContent(fm, fm.props, newProps);
		expect(out).toContain('title: Edited Title');
		// untouched keys unchanged, byte-for-byte:
		expect(out).toContain('\nsource:\n    author: Ibn Khaldun\n    year: 1377\n'); // 4-space preserved
		expect(out).toContain('description: |\n  First line of a long note.\n  Second line stays too.\n');
		expect(titleProp.value).toBe('Round Trip Probe'); // projection read the original
	});
});

describe('G4 Recipe B (yamlDoc) — embedded quotes round-trip stably', () => {
	const base = ['---', 'title: T', 'cid_cn: 20260707T120000Z_NOTE_0002', '---', 'body', ''].join('\n');

	it('adding a quoted-value property round-trips with no backslash doubling across cycles', () => {
		let content = base;
		// add the property
		const fm0 = parseFrontmatterDoc(content);
		const added: FrontmatterProperty = { key: 'quote', value: 'He said: "hi"', type: 'text' };
		content = composeContent(fm0, fm0.props, [...fm0.props, added]);

		// several further save cycles (no change) must be stable
		for (let i = 0; i < 4; i++) {
			const fm = parseFrontmatterDoc(content);
			content = composeContent(fm, fm.props, fm.props);
		}
		const finalProps = parseFrontmatterDoc(content).props;
		expect(finalProps.find((p) => p.key === 'quote')?.value).toBe('He said: "hi"');
		expect(content).not.toMatch(/\\\\/); // no doubled backslashes ever
	});
});

describe('G4 H1 — malformed YAML never loses content', () => {
	it('an unterminated flow sequence preserves both frontmatter and body verbatim', () => {
		const broken = ['---', 'title: My Note', 'tags: [a, b', 'status: draft', '---', 'An hour of body text.', ''].join('\n');
		const fm = parseFrontmatterDoc(broken);
		expect(fm.hasErrors).toBe(true);
		const out = composeContent(fm, fm.props, fm.props);
		expect(out).toContain('An hour of body text.');
		expect(out).toContain('title: My Note');
		expect(out).toContain('tags: [a, b'); // preserved verbatim, not thrown away
		expect(out).toContain('status: draft');
	});
});

describe('G4 projectProps — sequences are projected as editable lists (Finding 3)', () => {
	it('a block-list tags key projects to a list prop with listItems (not skipped)', () => {
		const fm = parseFrontmatterDoc(['---', 'title: T', 'tags:', '  - history', '  - science', '---', 'b', ''].join('\n'));
		const tags = fm.props.find((p) => p.key === 'tags');
		expect(tags?.type).toBe('list');
		expect(tags?.listItems).toEqual(['history', 'science']);
	});
	it('editing one scalar leaves an unedited list byte-perfect (no reorder/restyle churn)', () => {
		const src = ['---', 'title: Old', 'tags: [alpha, beta]', 'cid_cn: NOTE_1', '---', 'body', ''].join('\n');
		const fm = parseFrontmatterDoc(src);
		const newProps = fm.props.map((p) => (p.key === 'title' ? { ...p, value: 'New' } : p));
		const out = composeContent(fm, fm.props, newProps);
		expect(out).toContain('title: New');
		expect(out).toContain('tags: [alpha, beta]'); // flow list untouched, in place
		expect(out).toContain('cid_cn: NOTE_1'); // order preserved (tags NOT moved to end)
	});
});

describe('G4 fenceless notes — adding a property creates a frontmatter block (no drop)', () => {
	it('a plain .md gaining a property gets a valid fenced block', () => {
		const fm = parseFrontmatterDoc('just body text\nsecond line\n');
		expect(fm.hadFence).toBe(false);
		const added: FrontmatterProperty = { key: 'title', value: 'New', type: 'text' };
		const out = composeContent(fm, [], [added]);
		expect(out).toMatch(/^---\ntitle: New\n---\n/);
		expect(out).toContain('just body text');
		// round-trips: the new block parses back with the property
		expect(parseFrontmatterDoc(out).props.find((p) => p.key === 'title')?.value).toBe('New');
	});
	it('a plain .md with no properties stays fenceless (body only)', () => {
		const fm = parseFrontmatterDoc('body only\n');
		expect(composeContent(fm, [], [])).toBe('body only\n');
	});
});

describe('G4 splitFrontmatter — fence detection', () => {
	it('a note with no frontmatter is all body', () => {
		const r = splitFrontmatter('just a body\nno fence');
		expect(r.hadFence).toBe(false);
		expect(r.body).toBe('just a body\nno fence');
	});
	it('an unclosed fence is treated as body (parity with the old parser)', () => {
		const r = splitFrontmatter('---\ntitle: x\nno close');
		expect(r.hadFence).toBe(false);
	});
});

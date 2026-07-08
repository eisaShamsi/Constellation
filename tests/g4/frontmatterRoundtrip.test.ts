/**
 * G4 Phase 0 — Reproduce-First harness for the frontmatter round-trip app-killers.
 *
 * These assert the CORRECT round-trip behaviour, so against the current
 * hand-rolled `parseFrontmatter` / `reconstructFrontmatter` they FAIL RED —
 * that red baseline is the whole point (Reproduce-First). Phase 1 builds the
 * `yamlDoc` module and turns them GREEN; Phase 2 swaps the live save path onto
 * it and re-proves them on the running app (Editor-Surface Gate).
 *
 * Two confirmed app-killers (safety sweep + G4 Architect `wf_e553a7ff-b0d`):
 *   Recipe A — a nested map + a block scalar `|` are SILENTLY DROPPED on save.
 *   Recipe B — an embedded double-quote ACCUMULATES backslashes every save.
 */
import { describe, it, expect } from 'vitest';
import { parseFrontmatter, buildFullContent, type FrontmatterProperty } from '$lib/libraries/store';

// ─────────────────────────────────────────────────────────────────────────
// Recipe A — nested map (source.author/year) + block scalar (description: |)
// must survive a parse → serialize round-trip. (Obsidian/Git-authored note.)
// ─────────────────────────────────────────────────────────────────────────
const RECIPE_A = [
	'---',
	'title: Round Trip Probe',
	'cid_cn: 20260707T120000Z_NOTE_0001',
	'kind: note',
	'created: 2026-07-07T12:00:00+00:00',
	'source:',
	'  author: Ibn Khaldun',
	'  year: 1377',
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

describe('G4 Recipe A — nested map + block scalar survive round-trip', () => {
	it('preserves the nested map children (source.author / source.year)', () => {
		const { properties, body } = parseFrontmatter(RECIPE_A);
		const out = buildFullContent(properties, body);
		expect(out).toContain('Ibn Khaldun');
		expect(out).toContain('1377');
		// the corruption signature: `source:` reduced to an empty key
		expect(out).not.toMatch(/^source:\s*$/m);
	});

	it('preserves the block-scalar description body (both lines)', () => {
		const { properties, body } = parseFrontmatter(RECIPE_A);
		const out = buildFullContent(properties, body);
		expect(out).toContain('First line of a long note.');
		expect(out).toContain('Second line stays too.');
		// the corruption signature: description collapsed to the literal string "|"
		expect(out).not.toMatch(/description:\s*["']?\|["']?\s*$/m);
	});

	it('preserves the note body', () => {
		const { properties, body } = parseFrontmatter(RECIPE_A);
		const out = buildFullContent(properties, body);
		expect(out).toContain('Body text.');
	});
});

// ─────────────────────────────────────────────────────────────────────────
// Recipe B — an embedded double-quote value must be byte-stable across saves
// (no backslash doubling from the escape-on-write / no-unescape-on-read gap).
// ─────────────────────────────────────────────────────────────────────────
describe('G4 Recipe B — embedded quotes round-trip stably (no backslash doubling)', () => {
	const props0: FrontmatterProperty[] = [
		{ key: 'title', value: 'T', type: 'text' },
		{ key: 'cid_cn', value: '20260707T120000Z_NOTE_0002', type: 'text' },
		{ key: 'quote', value: 'He said: "hi"', type: 'text' },
	];

	it('a value with an embedded quote survives two save cycles unchanged', () => {
		const save1 = buildFullContent(props0, 'body');
		const save2 = buildFullContent(parseFrontmatter(save1).properties, parseFrontmatter(save1).body);
		const readback = parseFrontmatter(save2).properties.find((p) => p.key === 'quote')?.value;
		expect(readback).toBe('He said: "hi"');
	});

	it('does not accumulate backslashes on disk across saves', () => {
		let disk = buildFullContent(props0, 'body');
		for (let i = 0; i < 3; i++) {
			const { properties, body } = parseFrontmatter(disk);
			disk = buildFullContent(properties, body);
		}
		// after several cycles there must be no doubled backslash escape
		expect(disk).not.toMatch(/\\\\/);
	});
});

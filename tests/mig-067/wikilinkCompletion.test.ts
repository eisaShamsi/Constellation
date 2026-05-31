/**
 * MIG-067 §E — the type-first wikilink autocomplete (Option A).
 *
 * Verifies the phase logic + filtering of `createWikilinkCompletion` against a
 * mocked CompletionContext (the `apply` closures need a live EditorView, so the
 * actual insertion is covered by the Boss test; here we pin WHICH options appear
 * in each phase — the regression-prone part):
 *   - `[[`          → types (boosted) + notes
 *   - `[[sup`       → matching types only (no note matches "sup")
 *   - `[[type::ap`  → target notes only (no types)
 *   - `[[C++::x`    → null (unknown prefix = a real "::" in a name)
 *   - `[[Note|x`    → null (the legacy note|type menu owns the post-pipe region)
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import * as reg from '../../src/lib/libraries/linkTypeRegistry';
import { createWikilinkCompletion } from '../../src/lib/editor/completions';

const NOTES = [
	{ name: 'Apple', path: '/a.md' },
	{ name: 'Apricot', path: '/ap.md' },
	{ name: 'Banana', path: '/b.md' },
];

/** Minimal CompletionContext emulating CM6 `matchBefore` (anchored at end). */
function ctx(textBefore: string): any {
	return {
		pos: textBefore.length,
		matchBefore: (re: RegExp) => {
			const m = textBefore.match(re);
			if (!m) return null;
			return { from: textBefore.length - m[0].length, to: textBefore.length, text: m[0] };
		},
	};
}

const def = (id: string, order: number, color: string, desc: string) => ({
	id, label: id, parent: null, color, order, builtin: true, emoji: null, desc,
});

beforeEach(() => {
	reg.seedFromBundle([
		def('supports', 1, '#4A9EFF', 'Evidence'),
		def('contradicts', 2, '#FF4A4A', 'Tension'),
		def('supersedes', 8, '#5B7A8A', 'Replaces'),
	]);
});

describe('wikilink type-first autocomplete (§E)', () => {
	const complete = createWikilinkCompletion(() => NOTES);

	it('phase 1: [[ offers types (boosted) and notes', () => {
		const r: any = complete(ctx('[['));
		const labels = r.options.map((o: any) => o.label);
		expect(labels).toContain('supports');
		expect(labels).toContain('Apple');
		expect(r.options.find((o: any) => o.label === 'supports').boost).toBe(2);
		expect(r.options.find((o: any) => o.label === 'Apple').boost).toBeUndefined();
	});

	it('phase 1: [[sup narrows to the matching types, no note match', () => {
		const r: any = complete(ctx('[[sup'));
		const labels = r.options.map((o: any) => o.label);
		expect(labels).toEqual(expect.arrayContaining(['supports', 'supersedes']));
		expect(labels).not.toContain('Apple');
	});

	it('phase 2: [[supports::ap suggests target notes only', () => {
		const r: any = complete(ctx('[[supports::ap'));
		const labels = r.options.map((o: any) => o.label);
		expect(labels).toEqual(['Apple', 'Apricot']);
		expect(labels).not.toContain('supports');
	});

	it('phase 2: unknown prefix (real "::" in a name) yields no completion', () => {
		expect(complete(ctx('[[C++::vec'))).toBeNull();
	});

	it('does not fire after a pipe (legacy note|type menu owns that region)', () => {
		expect(complete(ctx('[[Apple|sup'))).toBeNull();
	});
});

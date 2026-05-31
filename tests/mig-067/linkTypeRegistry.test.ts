/**
 * MIG-067 §C — unit tests for the frontend Link-Type Registry.
 *
 * Covers the pure cache logic the editor (§E) and Base columns (§F) depend on:
 * seeding from the boot bundle, the ordered getters, the rank sentinel, the
 * color + label fallbacks, parent/child nesting, and the change-observer seam.
 * The invoke-backed paths (loadLinkTypes / saveLinkTypes) are thin wrappers
 * around Tauri commands and are exercised on the Rust side; here we only mock
 * `invoke` so the module's top-level import resolves in the node test env.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import * as reg from '../../src/lib/libraries/linkTypeRegistry';

type Def = reg.LinkTypeDef;
const def = (id: string, parent: string | null, order: number, color = '#123456'): Def => ({
	id, label: id, parent, color, order,
	builtin: parent === null && order <= 8, emoji: null, desc: null,
});

// A small resolved vocabulary: 2 seeds + a custom top-level + a child of supports.
const VOCAB: Def[] = [
	def('supports', null, 1, '#4A9EFF'),
	def('evidence-for', 'supports', 1, '#7FB8FF'),
	def('contradicts', null, 2, '#FF4A4A'),
	def('inspires', null, 9, '#33CC99'),
];

beforeEach(() => {
	reg.seedFromBundle(VOCAB);
});

describe('linkTypeRegistry §C', () => {
	it('seeds the resolved list verbatim + flags loaded', () => {
		expect(reg.getLinkTypes().map((t) => t.id)).toEqual([
			'supports', 'evidence-for', 'contradicts', 'inspires',
		]);
		expect(reg.isLoaded()).toBe(true);
	});

	it('looks up a type by id', () => {
		expect(reg.getLinkType('inspires')?.color).toBe('#33CC99');
		expect(reg.getLinkType('nope')).toBeUndefined();
	});

	it('rank is the 1-based position; unknown ids sort last (sentinel)', () => {
		expect(reg.linkTypeRank('supports')).toBe(1);
		expect(reg.linkTypeRank('inspires')).toBe(4);
		expect(reg.linkTypeRank('unknown')).toBe(5); // length + 1
	});

	it('color falls back to the neutral default for unknown ids', () => {
		expect(reg.linkTypeColor('contradicts')).toBe('#FF4A4A');
		expect(reg.linkTypeColor('unknown')).toBe('#AAAAAA');
	});

	it('label falls back to the id', () => {
		expect(reg.linkTypeLabel('supports')).toBe('supports');
		expect(reg.linkTypeLabel('unknown')).toBe('unknown');
	});

	it('separates top-level types from nested children', () => {
		expect(reg.topLevelLinkTypes().map((t) => t.id)).toEqual([
			'supports', 'contradicts', 'inspires',
		]);
		expect(reg.linkTypeChildren('supports').map((t) => t.id)).toEqual(['evidence-for']);
		expect(reg.linkTypeChildren('contradicts')).toEqual([]);
	});

	it('isKnownLinkType reflects the seeded vocabulary', () => {
		expect(reg.isKnownLinkType('evidence-for')).toBe(true);
		expect(reg.isKnownLinkType('unknown')).toBe(false);
	});

	it('isLinkTypeValue accepts typed acts + the null associative (§D)', () => {
		expect(reg.isLinkTypeValue('supports')).toBe(true);   // typed act
		expect(reg.isLinkTypeValue('evidence-for')).toBe(true); // custom
		expect(reg.isLinkTypeValue('associative')).toBe(true); // the null default
		expect(reg.isKnownLinkType('associative')).toBe(false); // but NOT a typed act
		expect(reg.isLinkTypeValue('unknown')).toBe(false);
	});

	it('toLinkTypeDeltas keeps custom + changed seeds, drops unchanged seeds (§G)', () => {
		const seed = (id: string, label: string, color: string, order: number): Def => ({
			id, label, parent: null, color, order, builtin: true, emoji: null, desc: null,
		});
		const types: Def[] = [
			seed('supports', 'Supports', '#4A9EFF', 1),       // == default → dropped
			seed('contradicts', 'Contradicts', '#00FF00', 2), // recoloured → kept
			def('inspires', null, 999, '#33CC99'),            // custom → kept
		];
		const ids = reg.toLinkTypeDeltas(types).map((d) => d.id);
		expect(ids).toContain('contradicts');
		expect(ids).toContain('inspires');
		expect(ids).not.toContain('supports');
	});

	it('stripLinkTypePrefix strips a known type:: prefix, preserves the rest', () => {
		// known types (seed + custom + associative) → stripped to the target
		expect(reg.stripLinkTypePrefix('supports::Apple')).toBe('Apple');
		expect(reg.stripLinkTypePrefix('evidence-for::Note')).toBe('Note');
		expect(reg.stripLinkTypePrefix('associative::X')).toBe('X');
		// display alias + fragment ride along (the caller splits them off after)
		expect(reg.stripLinkTypePrefix('supports::Apple|My alias')).toBe('Apple|My alias');
		// no prefix / unknown prefix / a real "::" in a name → untouched
		expect(reg.stripLinkTypePrefix('Apple')).toBe('Apple');
		expect(reg.stripLinkTypePrefix('Apple|supports')).toBe('Apple|supports');
		expect(reg.stripLinkTypePrefix('C++::vector')).toBe('C++::vector');
	});

	it('notifies subscribers on re-seed and stops after unsubscribe', () => {
		let n = 0;
		const off = reg.subscribe(() => { n++; });
		reg.seedFromBundle(VOCAB);
		expect(n).toBe(1);
		off();
		reg.seedFromBundle(VOCAB);
		expect(n).toBe(1);
	});
});

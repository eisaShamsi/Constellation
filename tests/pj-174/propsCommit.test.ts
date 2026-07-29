/**
 * MIG-107 Slice 4 — the commit planner, proven before it is wired.
 *
 * The rule this file exists to pin: **a commit may ADD and SET freely, but may only REMOVE a key the
 * panel actually knew about.** Everything else follows from it — most importantly that a key which
 * appeared after the panel last read the model cannot be deleted by that panel's next save, which is
 * the whole of AK-2 and AK-3.
 */
import { describe, it, expect } from 'vitest';
import { plan, apply, touchedSince, sameList, type PropOp, type IntentSink } from '$lib/editor/propsCommit';
import type { FrontmatterProperty, PropertyType } from '$lib/libraries/store';

const P = (key: string, value: string, type: PropertyType = 'text'): FrontmatterProperty => ({ key, value, type });
const L = (key: string, items: string[]): FrontmatterProperty =>
	({ key, value: items.join(', '), type: 'list', listItems: items });
const seed = (...keys: string[]) => new Set(keys);
const ofKind = (ops: PropOp[], kind: PropOp['op']) => ops.filter((o) => o.op === kind);

describe('MIG-107 Slice 4 — THE RULE: a commit cannot remove what the panel never saw', () => {
	/** AK-2, at the planner. The tag arrived after the panel seeded; the panel must not touch it. */
	it('leaves a key added by ANOTHER writer completely alone', () => {
		const local = [P('title', 'N'), P('stage', 'sapling')];          // the panel's view (no tags)
		const model = [P('title', 'N'), P('stage', 'seed'), L('tags', ['research'])]; // reality
		const ops = plan(local, model, seed('title', 'stage'));          // it never saw `tags`

		expect(ofKind(ops, 'remove')).toEqual([]);                        // nothing removed at all
		expect(JSON.stringify(ops)).not.toContain('tags');                // and `tags` is never named
		expect(ofKind(ops, 'set')).toEqual([
			{ op: 'set', key: 'stage', value: 'sapling', type: 'text' },   // exactly the edit made
		]);
	});

	it('removes a key the user DELETED — because the panel was showing it', () => {
		const local = [P('title', 'N')];
		const model = [P('title', 'N'), P('stage', 'seed')];
		const ops = plan(local, model, seed('title', 'stage'));
		expect(ofKind(ops, 'remove')).toEqual([{ op: 'remove', key: 'stage' }]);
	});

	/** The distinction the rule turns on, in one test. */
	it('tells a DELETED key apart from an UNSEEN one, in the same commit', () => {
		const local = [P('title', 'N')];                                   // user deleted `stage`
		const model = [P('title', 'N'), P('stage', 'seed'), L('tags', ['x'])]; // `tags` unseen
		const ops = plan(local, model, seed('title', 'stage'));
		expect(ofKind(ops, 'remove')).toEqual([{ op: 'remove', key: 'stage' }]);
	});
});

describe('MIG-107 Slice 4 — add / set / blank rows', () => {
	it('adds a key the model does not have', () => {
		const ops = plan([P('title', 'N'), P('sources', 's')], [P('title', 'N')], seed('title'));
		expect(ofKind(ops, 'add')).toEqual([{ op: 'add', prop: P('sources', 's') }]);
	});

	it('emits NOTHING for rows that did not change', () => {
		const same = [P('title', 'N'), P('stage', 'seed')];
		const ops = plan(same, same, seed('title', 'stage'));
		expect(ofKind(ops, 'set')).toEqual([]);
		expect(ofKind(ops, 'add')).toEqual([]);
		expect(ofKind(ops, 'remove')).toEqual([]);
	});

	/** PJ-178: a half-typed row is the panel's business and must never reach the note. */
	it('ignores a blank in-progress row entirely', () => {
		const ops = plan([P('title', 'N'), P('', '')], [P('title', 'N')], seed('title'));
		expect(ofKind(ops, 'add')).toEqual([]);
		expect(JSON.stringify(ops)).not.toContain('""');
	});

	it('carries listItems on a list change', () => {
		const ops = plan([L('tags', ['a', 'b'])], [L('tags', ['a'])], seed('tags'));
		expect(ofKind(ops, 'set')[0]).toMatchObject({ key: 'tags', listItems: ['a', 'b'] });
	});
});

describe('MIG-107 Slice 4 — ordering', () => {
	/**
	 * ★ This test previously asserted a trailing `{ key: 'a', beforeKey: null }` op AND claimed in its
	 * own name that "an unseen key is never moved". Both were wrong, and the inspection caught it:
	 * `beforeKey: null` means "move to the END of the model array", which pushes a foreign key sitting
	 * after it. A test that encodes the wrong contract is worse than no test — it makes the mistake
	 * look verified. Ordering is now anchored only BETWEEN two known keys.
	 */
	it('anchors only BETWEEN known keys, so an unseen key keeps its position', () => {
		const ops = plan([P('b', '2'), P('a', '1')], [P('a', '1'), P('b', '2'), L('tags', ['x'])], seed('a', 'b'));
		const orders = ofKind(ops, 'order');
		expect(orders).toEqual([{ op: 'order', key: 'b', beforeKey: 'a' }]);
		expect(JSON.stringify(orders)).not.toContain('tags');
	});
});

describe('MIG-107 Slice 4 — apply()', () => {
	function sink() {
		const calls: string[] = [];
		const s: IntentSink = {
			setValue: (k) => { calls.push(`set:${k}`); return true; },
			add: (p) => { calls.push(`add:${p.key}`); return true; },
			remove: (k) => { calls.push(`remove:${k}`); return true; },
			order: () => false, // an ordering no-op, the common case
		};
		return { s, calls };
	}

	it('drives each intent once, and reports whether anything changed', () => {
		const { s, calls } = sink();
		const ops = plan([P('title', 'N'), P('new', 'v')], [P('title', 'N'), P('gone', 'x')], seed('title', 'gone'));
		expect(apply(ops, s)).toBe(true);
		expect(calls).toEqual(['remove:gone', 'add:new']);
	});

	it('reports NO change when every intent no-ops — so a pointless disk write is skipped', () => {
		const s: IntentSink = { setValue: () => false, add: () => false, remove: () => false, order: () => false };
		const same = [P('title', 'N')];
		expect(apply(plan(same, same, seed('title')), s)).toBe(false);
	});
});

describe('MIG-107 #1e/#1f — the three findings the Slice-4 inspection returned', () => {
	/**
	 * #1e. `seededKeys` protects a key the panel never SAW. This protects a key it saw but never
	 * TOUCHED — whose value another writer may have moved on since. Without it the commit wrote back
	 * every row it held, silently reverting the other writer's change on a shared key.
	 */
	it('does NOT write back a key the user never edited, even though the panel is showing it', () => {
		const local = [P('title', 'N'), P('stage', 'seed')];   // panel still displays the OLD stage
		const model = [P('title', 'N'), P('stage', 'sapling')]; // the other panel already moved it on
		const ops = plan(local, model, seed('title', 'stage'), new Set(['title']));
		expect(ofKind(ops, 'set')).toEqual([]);                 // `stage` is left exactly as it is
	});

	it('still writes a key the user DID edit', () => {
		const local = [P('title', 'N'), P('stage', 'sapling')];
		const model = [P('title', 'N'), P('stage', 'seed')];
		const ops = plan(local, model, seed('title', 'stage'), new Set(['stage']));
		expect(ofKind(ops, 'set')).toEqual([{ op: 'set', key: 'stage', value: 'sapling', type: 'text' }]);
	});

	it('omitting touchedKeys keeps the legacy whole-array meaning (nothing else changes)', () => {
		const local = [P('title', 'N'), P('stage', 'sapling')];
		const model = [P('title', 'N'), P('stage', 'seed')];
		expect(ofKind(plan(local, model, seed('title', 'stage')), 'set')).toHaveLength(1);
	});

	/**
	 * #1f. My own comment claimed an unseen key is "never moved". It was false: a trailing
	 * `beforeKey: null` means "move to the END", which pushes any foreign key sitting after it.
	 * Ordering is now anchored only BETWEEN two known keys, so every op stays inside the panel's span.
	 */
	it('never emits an END anchor, so a foreign key cannot be pushed out of position', () => {
		const ops = plan([P('b', '2'), P('a', '1')], [P('a', '1'), P('b', '2'), L('tags', ['x'])],
			seed('a', 'b'), new Set(['a', 'b']));
		const orders = ofKind(ops, 'order');
		expect(orders.every((o) => o.op === 'order' && o.beforeKey !== null)).toBe(true);
		expect(orders).toEqual([{ op: 'order', key: 'b', beforeKey: 'a' }]);
	});

	it('emits no ordering at all for a single row (nothing to anchor between)', () => {
		expect(ofKind(plan([P('a', '1')], [P('a', '1')], seed('a'), new Set()), 'order')).toEqual([]);
	});
});

describe('MIG-107 #1g — touchedSince: derived, so it cannot be forgotten', () => {
	/**
	 * THE BOSS-FOUND REGRESSION. The first version had the panel hand-mark edited keys from its edit
	 * handlers, and it was wired at 3 of that component's 16 mutation sites. Adding a tag went
	 * through one of the 13 missed ones, so the commit skipped it: the tag appeared in the panel it
	 * was typed into and nowhere else — not the other panel, not the model, not the file.
	 */
	it('detects an edit made by ANY code path, including one it was never told about', () => {
		const seeded = [P('title', 'N'), L('tags', ['test'])];
		const local = [P('title', 'N'), L('tags', ['test', 'Eisa'])]; // the tag editor's own mutation
		expect(touchedSince(seeded, local)).toEqual(new Set(['tags']));
	});

	it('reports nothing when the panel has not been edited', () => {
		const rows = [P('title', 'N'), P('stage', 'seed'), L('tags', ['a'])];
		expect(touchedSince(rows, rows)).toEqual(new Set());
	});

	it('counts a NEW key and a DELETED key as edits', () => {
		expect(touchedSince([P('a', '1')], [P('a', '1'), P('b', '2')])).toEqual(new Set(['b']));
		expect(touchedSince([P('a', '1'), P('b', '2')], [P('a', '1')])).toEqual(new Set(['b']));
	});

	it('ignores a blank in-progress row', () => {
		expect(touchedSince([P('a', '1')], [P('a', '1'), P('', '')])).toEqual(new Set());
	});

	/** End to end: the tag edit now survives the planner too. */
	it('lets the tag edit through the planner while still shielding an unseen key', () => {
		const seeded = [P('title', 'N'), L('tags', ['test'])];
		const local = [P('title', 'N'), L('tags', ['test', 'Eisa'])];
		const model = [P('title', 'N'), L('tags', ['test']), P('stage', 'seed')]; // `stage` unseen
		const ops = plan(local, model, seed('title', 'tags'), touchedSince(seeded, local));
		expect(ofKind(ops, 'set')).toEqual([
			{ op: 'set', key: 'tags', value: 'test, Eisa', type: 'list', listItems: ['test', 'Eisa'] },
		]);
		expect(JSON.stringify(ops)).not.toContain('stage');
	});
});

describe('MIG-107 Slice 6 — /simplify: the quadratic ordering, removed', () => {
	/**
	 * `plan` used to emit an `order` op for EVERY adjacent row pair, unconditionally — and each one
	 * made the model deep-clone its whole array before discovering the move was a no-op. A single
	 * value edit on a 10-property note cost ~90 discarded object spreads, growing as N².
	 * Now the ops appear only when the order genuinely differs.
	 */
	it('emits NO ordering ops when the model already matches the panel', () => {
		const rows = [P('a', '1'), P('b', '2'), P('c', '3')];
		expect(ofKind(plan(rows, rows, seed('a', 'b', 'c'), new Set()), 'order')).toEqual([]);
	});

	it('still emits them when the order genuinely differs', () => {
		const model = [P('a', '1'), P('b', '2')];
		const rows = [P('b', '2'), P('a', '1')];
		expect(ofKind(plan(rows, model, seed('a', 'b'), new Set()), 'order').length).toBeGreaterThan(0);
	});

	it('a value edit alone costs no ordering work', () => {
		const model = [P('a', '1'), P('b', '2'), P('c', '3')];
		const rows = [P('a', '9'), P('b', '2'), P('c', '3')];
		const ops = plan(rows, model, seed('a', 'b', 'c'), new Set(['a']));
		expect(ofKind(ops, 'order')).toEqual([]);
		expect(ofKind(ops, 'set')).toHaveLength(1);
	});

	/** Element-wise now, not four JSON.stringify spellings across three modules. */
	it('sameList compares element-wise and treats empty/absent alike', () => {
		expect(sameList(['a', 'b'], ['a', 'b'])).toBe(true);
		expect(sameList(['a'], ['b'])).toBe(false);
		expect(sameList(['a'], ['a', 'b'])).toBe(false);
		expect(sameList(undefined, undefined)).toBe(true);
		expect(sameList(undefined, [])).toBe(true);
		expect(sameList(undefined, ['a'])).toBe(false);
	});
});

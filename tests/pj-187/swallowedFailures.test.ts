/**
 * PJ-187 (register) — the two decisions in the sweep that are PURE, and were therefore the two
 * that could be pinned without a component harness.
 *
 * Both are the same shape as LL-040: a boolean that *looks* like it answers the question, and
 * silently answers a different one.
 *
 *   1. `apply` returned `changed: false` for a REFUSED intent and for a genuine no-op alike, so
 *      the panel's "nothing to do, skip the write" branch swallowed a rejected property edit
 *      whole — the user's change never reached the file and nothing anywhere said so.
 *
 *   2. `cascadeFreeze` was set and cleared as a bare boolean by two call sites, so two
 *      overlapping renames in one library shared one flag: the first to finish lifted the
 *      read-only overlay while the second cascade was still rewriting files on disk. Its
 *      non-reactive twin (`cascadingLibraries`) has been refcounted since MIG-076 — this is the
 *      reactive half that was not.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { plan, apply, type IntentSink } from '$lib/editor/propsCommit';
import { markFreeze, clearFreeze, cascadeFreeze, isPathFrozen } from '$lib/libraries/store';
import type { FrontmatterProperty } from '$lib/libraries/store';

const prop = (key: string, value: string): FrontmatterProperty => ({ key, value, type: 'text' });
const seed = (...keys: string[]) => new Set(keys);

/** A sink that takes everything — the healthy model. */
const acceptAll = (): IntentSink => ({
	setValue: () => true,
	add: () => true,
	remove: () => true,
	order: () => true,
});

/** A sink that refuses everything — what an identity mismatch inside the model looks like. */
const refuseAll = (): IntentSink => ({
	setValue: () => false,
	add: () => false,
	remove: () => false,
	order: () => false,
});

describe('PJ-187 — a REFUSED property intent must not read as "nothing to do"', () => {
	it('reports every refused op, and reports changed:false alongside it', () => {
		const model = [prop('title', 'Old'), prop('stage', 'seed')];
		const rows = [prop('title', 'New'), prop('stage', 'seed')];
		const ops = plan(rows, model, seed('title', 'stage'), seed('title'));
		expect(ops).toHaveLength(1); // the title set

		const r = apply(ops, refuseAll());
		expect(r.changed).toBe(false);
		// …and the caller can now TELL the two apart, which is the whole point.
		expect(r.refused).toHaveLength(1);
		expect(r.refused[0]).toMatchObject({ op: 'set', key: 'title' });
	});

	it('a healthy commit refuses nothing', () => {
		const model = [prop('title', 'Old')];
		const rows = [prop('title', 'New')];
		const r = apply(plan(rows, model, seed('title'), seed('title')), acceptAll());
		expect(r.changed).toBe(true);
		expect(r.refused).toEqual([]);
	});

	it('an EMPTY plan is not a refusal — there was genuinely nothing to do', () => {
		const same = [prop('title', 'Same')];
		const ops = plan(same, same, seed('title'), seed('title'));
		expect(ops).toEqual([]);
		const r = apply(ops, refuseAll()); // the sink never runs
		expect(r.changed).toBe(false);
		expect(r.refused).toEqual([]);
	});

	it('an `order` op that no-ops is NOT counted as a refusal', () => {
		// plan emits one order op per adjacent pair when the two orders differ; a move that is
		// already in position legitimately reports no change, so it must not raise the alarm.
		const model = [prop('b', '2'), prop('a', '1')];
		const rows = [prop('a', '1'), prop('b', '2')];
		const ops = plan(rows, model, seed('a', 'b'), seed());
		expect(ops.some((o) => o.op === 'order')).toBe(true);

		const orderOnlyRefuser: IntentSink = { ...acceptAll(), order: () => false };
		const r = apply(ops, orderOnlyRefuser);
		expect(r.refused).toEqual([]);
	});
});

describe('PJ-187 — the cascade freeze must survive an overlapping rename', () => {
	const LIB = 'E:/Universe/Library';
	const NOTE = 'E:/Universe/Library/Folder/Note.md';
	const frozen = () => {
		let v!: Set<string>;
		cascadeFreeze.subscribe((s) => (v = s))();
		return v;
	};

	beforeEach(() => {
		// Drain any depth left by a previous test.
		for (let i = 0; i < 8 && frozen().size; i++) clearFreeze(LIB);
	});

	it('two overlapping cascades: the first to finish does NOT lift the overlay', () => {
		markFreeze(LIB); // rename A starts
		markFreeze(LIB); // rename B starts before A finishes
		expect(isPathFrozen(NOTE, frozen())).toBe(true);

		clearFreeze(LIB); // A finishes — B is still rewriting files
		// Before the fix this published the EMPTY set and the user could type into a note
		// the walker was mid-rewrite.
		expect(isPathFrozen(NOTE, frozen())).toBe(true);

		clearFreeze(LIB); // B finishes
		expect(isPathFrozen(NOTE, frozen())).toBe(false);
	});

	it('a single cascade still freezes and lifts exactly once', () => {
		expect(isPathFrozen(NOTE, frozen())).toBe(false);
		markFreeze(LIB);
		expect(isPathFrozen(NOTE, frozen())).toBe(true);
		clearFreeze(LIB);
		expect(isPathFrozen(NOTE, frozen())).toBe(false);
	});

	it('an unbalanced clear cannot drive the depth negative and wedge the next freeze', () => {
		clearFreeze(LIB); // stray clear, nothing frozen
		markFreeze(LIB);
		expect(isPathFrozen(NOTE, frozen())).toBe(true);
		clearFreeze(LIB);
		expect(isPathFrozen(NOTE, frozen())).toBe(false);
	});

	it('a sibling library sharing a name prefix is never frozen by the other', () => {
		markFreeze(LIB);
		expect(isPathFrozen('E:/Universe/Library2/Note.md', frozen())).toBe(false);
		clearFreeze(LIB);
	});
});

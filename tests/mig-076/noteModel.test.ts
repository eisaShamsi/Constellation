/**
 * MIG-076 §C — the ACCEPTANCE HARNESS for single content ownership.
 *
 * Each describe block is one named 2026-06 failure turned into an invariant
 * the cure must satisfy. Green here proves the cure's ENGINE is correct.
 * (Integration into the live components is gated separately by the runtime
 * view-vs-disk harness + a Boss test — see CLAUDE.md Editor-Surface Gate.)
 */
import { describe, it, expect, beforeEach } from 'vitest';
import {
	openModel, getModel, closeModel, clearAllModels, modelCount,
	setBody, setProps, compose, markSaved, isDirty, adoptDisk,
} from '$lib/editor/noteModel';
import { buildFullContent, parseFrontmatter, type FrontmatterProperty } from '$lib/libraries/store';

/** Build a minimal on-disk note string. */
const note = (cid: string, body: string, extra = '') =>
	`---\ntitle: T\ncid_cn: ${cid}\n${extra}---\n${body}`;

beforeEach(() => clearAllModels());

describe('I1 — always current (kills symptom 1: stale focus-seed → empty-body write)', () => {
	it('a body change is readable immediately, and a save right now carries it', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'hello'));
		setBody('t1', 'hello world');
		expect(getModel('t1')!.body.toString()).toBe('hello world');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) expect(r.content).toContain('hello world');
	});

	it('NotePane and Focus read ONE model — there is no second copy to go stale', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'v1'));
		setBody('t1', 'typed in notepane'); // notepane view pushes
		expect(getModel('t1')!.body.toString()).toBe('typed in notepane');
		setBody('t1', 'typed in focus'); // focus view pushes the same model
		expect(getModel('t1')!.body.toString()).toBe('typed in focus'); // notepane reads it back
	});
});

describe('I2 — freshness (kills symptom 2: stale restore over fresh content)', () => {
	it('unsaved local edits are never clobbered by a disk read', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'original'));
		setBody('t1', 'unsaved edit'); // dirty
		const adopted = adoptDisk('t1', note('NOTE_1', 'original'));
		expect(adopted).toBe(false);
		expect(getModel('t1')!.body.toString()).toBe('unsaved edit');
	});

	it("ignores the watcher echo of our own write (identical disk payload)", () => {
		openModel('t1', '/n.md', note('NOTE_1', 'same'));
		expect(adoptDisk('t1', note('NOTE_1', 'same'))).toBe(false);
		expect(getModel('t1')!.body.toString()).toBe('same');
	});

	it('a clean model adopts a genuine external change (second screen / another app)', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'v1'));
		expect(isDirty('t1')).toBe(false);
		expect(adoptDisk('t1', note('NOTE_1', 'v2 from second screen'))).toBe(true);
		expect(getModel('t1')!.body.toString()).toBe('v2 from second screen');
	});

	it('persists its latest content across a view switch (no resolve-from-snapshot)', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'start'));
		setBody('t1', 'edited then switched away');
		// switching away destroys the VIEW, not the model; returning reads the model
		expect(getModel('t1')!.body.toString()).toBe('edited then switched away');
	});
});

describe('I3 — identity-bound compose (kills the in-focus-switch cross-note write)', () => {
	it('composes for the matching path', () => {
		openModel('t1', '/notes/a.md', note('NOTE_A', 'body A'));
		const r = compose('t1', '/notes/a.md');
		expect(r.ok).toBe(true);
		if (r.ok) {
			expect(r.path).toBe('/notes/a.md');
			expect(r.content).toContain('body A');
		}
	});

	it('REFUSES when the target path is not this model’s identity', () => {
		openModel('t1', '/notes/a.md', note('NOTE_A', 'body A'));
		const r = compose('t1', '/notes/b.md'); // a callback that outlived its slot
		expect(r.ok).toBe(false);
		if (!r.ok) {
			expect(r.reason).toBe('path_mismatch');
			expect(r.modelPath).toBe('/notes/a.md');
		}
	});
});

describe('I4 — single deterministic composition', () => {
	it('content = buildFullContent(props, body), from the model alone', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'the body', 'mood: calm\n'));
		const m = getModel('t1')!;
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) expect(r.content).toBe(buildFullContent(m.props, m.body.toString()));
	});

	it('open → compose round-trips the note', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'round trip body'));
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) {
			const reparsed = parseFrontmatter(r.content);
			expect(reparsed.body).toBe('round trip body');
			expect(reparsed.properties.find((p) => p.key === 'cid_cn')?.value).toBe('NOTE_1');
		}
	});
});

describe('I5 — model independence (cross-wire impossible)', () => {
	it('touching note B never changes note A’s composed content', () => {
		openModel('a', '/a.md', note('NOTE_A', 'A body'));
		openModel('b', '/b.md', note('NOTE_B', 'B body'));
		setBody('b', 'B body edited while A is open');
		const ra = compose('a', '/a.md');
		expect(ra.ok).toBe(true);
		if (ra.ok) {
			expect(ra.content).toContain('A body');
			expect(ra.content).not.toContain('B body');
		}
	});

	it('props and body update independently — no assemble-from-two-stale-sources', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'body'));
		setProps('t1', [
			...getModel('t1')!.props,
			{ key: 'stage', value: 'spark', type: 'text' } as FrontmatterProperty,
		]);
		setBody('t1', 'new body');
		const r = compose('t1', '/n.md');
		expect(r.ok).toBe(true);
		if (r.ok) {
			expect(r.content).toContain('stage: spark');
			expect(r.content).toContain('new body');
		}
	});
});

describe('dirty tracking', () => {
	it('open is clean; edit is dirty; markSaved clears', () => {
		openModel('t1', '/n.md', note('NOTE_1', 'x'));
		expect(isDirty('t1')).toBe(false);
		setBody('t1', 'y');
		expect(isDirty('t1')).toBe(true);
		markSaved('t1', getModel('t1')!.version);
		expect(isDirty('t1')).toBe(false);
	});
});

describe('lifecycle', () => {
	it('close disposes; clearAll empties', () => {
		openModel('t1', '/a.md', note('N1', 'a'));
		openModel('t2', '/b.md', note('N2', 'b'));
		closeModel('t1');
		expect(getModel('t1')).toBeUndefined();
		expect(modelCount()).toBe(1);
		clearAllModels();
		expect(modelCount()).toBe(0);
	});
});

/**
 * MIG-076 §C — THE RUNTIME HARNESS (logic level).
 *
 * Plays each named 2026-06 failure as a full RECIPE SEQUENCE through the real
 * content-flow code the components will call (noteSession over noteModel),
 * against an in-memory fake disk, asserting **screen === disk** and **no
 * cross-note contamination** after every transition. This is the audit's
 * view-vs-disk parity gate, headless and permanent — it re-runs on every
 * future change to the editor's content layer.
 *
 * Scope honesty: this proves the content-flow LOGIC + GLUE end-to-end. It does
 * NOT mount Svelte components / CM6, so a pure-template wiring mistake at
 * integration (e.g. seeding a view from tab.content instead of bodyForView)
 * is the one residual the Boss test still closes. Single ownership removes the
 * stale alternative that made such a mistake possible, so the residual is small.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import * as S from '$lib/editor/noteSession';
import { parseFrontmatter } from '$lib/libraries/store';

const note = (cid: string, body: string, extra = '') =>
	`---\ntitle: T\ncid_cn: ${cid}\n${extra}---\n${body}`;

/** Fake disk + writer injected into the real save path. */
let disk: Map<string, string>;
const write = (path: string, content: string) => { disk.set(path, content); };
const diskBody = (path: string) => parseFrontmatter(disk.get(path) ?? '').body;
const diskCid = (path: string) =>
	parseFrontmatter(disk.get(path) ?? '').properties.find((p) => p.key === 'cid_cn')?.value ?? null;

beforeEach(() => {
	disk = new Map();
	S.closeAll();
});

describe('Recipe A — symptom 1: Focus opens on a freshly-typed note', () => {
	it('the focus seed is the CURRENT body, never empty, and no empty-body write occurs', async () => {
		S.open('t', '/n.md', note('N', '')); // brand-new note: empty body on disk
		S.editBody('t', 'hello world'); // type in NotePane
		await S.save('t', '/n.md', write);
		expect(diskBody('/n.md')).toBe('hello world');

		// ENTER FOCUS — the seed must be the model's current body (the bug: '')
		expect(S.bodyForView('t')).toBe('hello world');

		S.editBody('t', 'hello world\nadded in focus'); // type in focus
		await S.save('t', '/n.md', write, 'focus_pane');
		expect(diskBody('/n.md')).toBe('hello world\nadded in focus');
		expect(S.bodyForView('t')).toBe(diskBody('/n.md')); // screen === disk
		// the note never lost its body to an empty write at any point:
		expect(diskBody('/n.md').length).toBeGreaterThan(0);
	});
});

describe('Recipe B — symptom 2: switch away and return', () => {
	it('returning shows the latest content, never a stale snapshot', async () => {
		S.open('a', '/a.md', note('NA', 'A1'));
		S.open('b', '/b.md', note('NB', 'B1'));
		S.editBody('a', 'A2');
		await S.save('a', '/a.md', write);

		// switch to b, then back to a (switching carries NO content op)
		// returning reads the model, not a resolve-from-snapshot:
		expect(S.bodyForView('a')).toBe('A2');
		expect(S.bodyForView('a')).toBe(diskBody('/a.md')); // screen === disk
	});
});

describe('Recipe C — the landmine: tab switch WHILE in Focus', () => {
	it('the in-focus note saves its OWN content; the other file is never cross-written', async () => {
		S.open('a', '/a.md', note('NA', 'A body'));
		S.open('b', '/b.md', note('NB', 'B body'));
		await S.save('b', '/b.md', write); // B persisted as itself

		// Focus on A; user switches to B; A's view flushes A on teardown:
		S.editBody('a', 'A body edited');
		const rA = await S.save('a', '/a.md', write);
		expect(rA.ok).toBe(true);
		expect(diskBody('/a.md')).toBe('A body edited');
		expect(diskCid('/a.md')).toBe('NA');
		expect(disk.get('/a.md')).not.toContain('B body');

		// A stale callback trying to write A's content to B's path is REFUSED:
		const bad = await S.save('a', '/b.md', write);
		expect(bad.ok).toBe(false);
		if (!bad.ok) expect(bad.reason).toBe('path_mismatch');
		// B on disk still holds ONLY B's content + identity:
		expect(diskBody('/b.md')).toBe('B body');
		expect(diskCid('/b.md')).toBe('NB');
	});
});

describe('Recipe D — rename with a link (the BUG-023 shape)', () => {
	it('renamed note keeps its identity; the linking note keeps its own', async () => {
		S.open('a', '/a.md', note('NA', 'see [[B]]'));
		S.open('b', '/b.md', note('NB', 'B content'));

		// rename B (title-rename or file-rename): identity moves, content intact
		S.repath('b', '/B-renamed.md');
		const rb = await S.save('b', '/B-renamed.md', write);
		expect(rb.ok).toBe(true);
		expect(diskCid('/B-renamed.md')).toBe('NB');
		expect(diskBody('/B-renamed.md')).toBe('B content');

		// the cascade rewrites A's link — A saves its OWN content only:
		S.editBody('a', 'see [[B-renamed]]');
		await S.save('a', '/a.md', write);
		expect(diskCid('/a.md')).toBe('NA');
		expect(diskBody('/a.md')).toBe('see [[B-renamed]]');

		// neither file acquired the other's identity (the BUG-023 wound):
		expect(disk.get('/a.md')).not.toContain('NB');
		expect(disk.get('/B-renamed.md')).not.toContain('NA');
	});
});

describe('Recipe E — second screen / external change (freshness)', () => {
	it('a clean note adopts external edits; a dirty note rejects them (local wins)', () => {
		S.open('t', '/n.md', note('N', 'v1'));
		expect(S.externalChange('t', note('N', 'v2 from second screen'))).toBe(true);
		expect(S.bodyForView('t')).toBe('v2 from second screen');

		S.editBody('t', 'local unsaved edit'); // now dirty
		expect(S.externalChange('t', note('N', 'v3'))).toBe(false);
		expect(S.bodyForView('t')).toBe('local unsaved edit');
	});
});

describe('Recipe F — restart / workspace restore', () => {
	it('every note reopens with its OWN content from disk', async () => {
		S.open('a', '/a.md', note('NA', 'A'));
		S.open('b', '/b.md', note('NB', 'B'));
		S.editBody('a', 'A edited');
		S.editBody('b', 'B edited');
		await S.save('a', '/a.md', write);
		await S.save('b', '/b.md', write);

		S.closeAll(); // restart

		S.open('a', '/a.md', disk.get('/a.md')!);
		S.open('b', '/b.md', disk.get('/b.md')!);
		expect(S.bodyForView('a')).toBe('A edited');
		expect(S.bodyForView('b')).toBe('B edited');
		expect(S.bodyForView('a')).not.toBe(S.bodyForView('b'));
	});
});

describe('Global invariant — disk never holds a foreign identity', () => {
	it('after a mixed session, every file on disk carries only its own cid', async () => {
		S.open('a', '/a.md', note('NA', 'alpha'));
		S.open('b', '/b.md', note('NB', 'beta'));
		S.open('c', '/c.md', note('NC', 'gamma'));
		S.editBody('a', 'alpha-2');
		S.editBody('c', 'gamma-2');
		await S.save('a', '/a.md', write);
		await S.save('b', '/b.md', write);
		await S.save('c', '/c.md', write);

		expect(diskCid('/a.md')).toBe('NA');
		expect(diskCid('/b.md')).toBe('NB');
		expect(diskCid('/c.md')).toBe('NC');
		expect(diskBody('/a.md')).toBe('alpha-2');
		expect(diskBody('/b.md')).toBe('beta');
		expect(diskBody('/c.md')).toBe('gamma-2');
	});
});

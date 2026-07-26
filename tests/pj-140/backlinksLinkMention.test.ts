/**
 * PJ-140 [0] (HIGH, 2026-07-25) — the Backlinks panel "link it" action must forge an inline
 * wikilink WITHOUT the three silent failures the old `linkMention` had:
 *   1. Open-model overwrite — it read the mentioning note from DISK and wrote behind the open
 *      in-memory model, whose next autosave composed from the stale body and erased the link
 *      (and, for a dirty note, the user's unsaved edits with it).
 *   2. False success — a `catch {}` swallowed a failed write; the user believed it landed.
 *   3. Index divergence — no reindex, so the new backlink edge was invisible until a boot.
 *
 * The fix routes the write through the shared store primitive `linkMentionInNote`, which uses
 * the proven `toggleTaskReconciled` body-edit shape: gate → flush the open model to disk first
 * (or ABORT rather than clobber) → mutate disk → the model adopts the mutated disk → reindex;
 * with longest-root-wins library resolution and a throw (not a swallow) on a genuine write error.
 *
 * These drive the REAL primitive against a disk-backed mocked IPC bridge (the
 * reopenRecoveryClobber.test.ts harness pattern).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	linkMentionInNote,
	libraryStats,
	openTabs,
	activeTabId,
	focusedTabId,
	splitActive,
	splitDirection,
	openNoteTab,
	flushDisposeClearTabs,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let locked: Set<string>;
let calls: Array<{ cmd: string; args: any }>;

const note = (body: string) => `---\ntitle: T\ncid_cn: N\n---\n${body}`;
const bodyOf = (content: string) => content.split('\n---\n').slice(1).join('\n---\n');

function wireInvoke() {
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		calls.push({ cmd, args });
		switch (cmd) {
			case 'read_note': {
				const c = disk.get(args.filePath);
				if (c === undefined) throw new Error('missing');
				return c;
			}
			case 'write_note': {
				if (locked.has(args.filePath)) throw new Error('EBUSY: locked');
				disk.set(args.filePath, args.content);
				return undefined;
			}
			case 'ensure_cid_cn_cmd': {
				return disk.get(args.filePath) ?? '';
			}
			case 'constellation_search_reindex':
				return undefined;
			default:
				return undefined;
		}
	});
}

const tabs = () => {
	let v: any[] = [];
	openTabs.subscribe((x) => (v = x))();
	return v;
};

const reindexCalls = () => calls.filter((c) => c.cmd === 'constellation_search_reindex');
const writeCalls = () => calls.filter((c) => c.cmd === 'write_note');

beforeEach(async () => {
	await flushDisposeClearTabs('test-reset');
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	libraryStats.set([{ name: 'Lib', path: '/lib' } as any]);
	disk = new Map();
	locked = new Set();
	calls = [];
	wireInvoke();
});

describe('linkMentionInNote — CLOSED note', () => {
	it('T1: links the body mention AND reindexes (fixes index divergence)', async () => {
		const P = '/lib/b.md';
		disk.set(P, note('This note talks about Target Note in prose.'));

		const r = await linkMentionInNote(P, 'Target Note');

		expect(r).toBe(true);
		expect(bodyOf(disk.get(P)!)).toContain('[[Target Note]]');
		// The OLD linkMention wrote but NEVER reindexed — assert the reindex now fires, for THIS path.
		expect(reindexCalls().some((c) => c.args.notePath === P)).toBe(true);
	});

	it('T2: a genuine write failure THROWS — never the old silent catch{} (fixes false success)', async () => {
		const P = '/lib/b.md';
		disk.set(P, note('Mentions Target Note once.'));
		locked.add(P); // write_note will reject

		await expect(linkMentionInNote(P, 'Target Note')).rejects.toThrow();
		expect(bodyOf(disk.get(P)!)).not.toContain('[['); // disk untouched
	});

	it('T3: a mention ONLY in frontmatter is never linked (body-scoped — no YAML corruption)', async () => {
		const P = '/lib/c.md';
		// "Target Note" appears in the title but NOT in the body.
		disk.set(P, `---\ntitle: Target Note\ncid_cn: N\n---\nAn unrelated body with nothing to link.`);

		const r = await linkMentionInNote(P, 'Target Note');

		expect(r).toBe(false);
		expect(writeCalls().length).toBe(0); // nothing written
		expect(disk.get(P)).toContain('title: Target Note'); // frontmatter intact, un-wikilinked
	});

	it('T4: an already-linked occurrence is skipped; the PLAIN one is linked (no [[[[..]]]])', async () => {
		const P = '/lib/d.md';
		disk.set(P, note('See [[Target Note]] and also Target Note again.'));

		const r = await linkMentionInNote(P, 'Target Note');

		expect(r).toBe(true);
		const b = bodyOf(disk.get(P)!);
		expect(b).toBe('See [[Target Note]] and also [[Target Note]] again.');
		expect(b).not.toContain('[[[['); // never double-wrapped
	});

	it('T5: a nested-library note reindexes under the NESTED library, not the root (PJ-141 safety)', async () => {
		libraryStats.set([
			{ name: 'Root', path: '/lib' } as any,
			{ name: 'Nested', path: '/lib/nested' } as any,
		]);
		const P = '/lib/nested/e.md';
		disk.set(P, note('Mentions Target Note here.'));

		await linkMentionInNote(P, 'Target Note');

		const rc = reindexCalls().find((c) => c.args.notePath === P);
		expect(rc).toBeTruthy();
		expect(rc!.args.libraryName).toBe('Nested'); // longest-root-wins, NOT the first-match 'Root'
	});
});

describe('linkMentionInNote — OPEN note', () => {
	it('T6: an OPEN note whose dirty model cannot be flushed ABORTS — no clobber (the HIGH)', async () => {
		const P = '/lib/g.md';
		disk.set(P, note('Mentions Target Note in the body.'));
		await openNoteTab(P, 'Lib');
		const tab = tabs().find((t) => t.path === P)!;
		expect(tab).toBeTruthy();

		// The user has unsaved edits AND the .md is locked, so the pre-mutation flush cannot land.
		S.editBody(tab.id, note('Mentions Target Note in the body.\nUNSAVED user edit.'));
		locked.add(P);

		const r = await linkMentionInNote(P, 'Target Note');

		// It must NOT have written the link behind the open dirty model (the old overwrite bug):
		expect(r).toBe(false);
		// No SUCCESSFUL wikilink write reached disk — the flush write failed and the mutation aborted.
		expect(disk.get(P)).not.toContain('[[Target Note]]');
	});

	it('T7: an OPEN note with a clean model links + reindexes through the safe path', async () => {
		const P = '/lib/h.md';
		disk.set(P, note('Mentions Target Note in the body.'));
		await openNoteTab(P, 'Lib');
		const tab = tabs().find((t) => t.path === P)!;
		expect(tab).toBeTruthy();

		const r = await linkMentionInNote(P, 'Target Note');

		expect(r).toBe(true);
		expect(bodyOf(disk.get(P)!)).toContain('[[Target Note]]');
		expect(reindexCalls().some((c) => c.args.notePath === P)).toBe(true);
	});
});

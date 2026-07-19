/**
 * APP-KILLER — `moveItem` repaths open tabs one directory too high.
 *
 * THE DEFECT (store.ts:3537-3540, confirmed at HEAD db65d15b). Moving a FOLDER while a note
 * inside it is open repoints that tab — and its note model — at
 *     `targetFolder + relative`
 * where `relative` is the tab path with the SOURCE FOLDER PATH sliced off. That slice starts
 * *after* the moved folder's own name, so the folder's name is dropped from the result. Rust's
 * `move_item` returns `dest = target_dir.join(basename(source))` (libraries.rs:1710-1712), i.e.
 * the correct new root — and the sibling `renameItem` branch (store.ts:3507-3510) already uses
 * exactly that (`effectivePath + relative`). The asymmetry IS the bug.
 *
 * WHY NOTHING CATCHES IT. `compose(id, expectPath)` refuses on `m.path !== expectPath`
 * (noteModel.ts:297) — but both the tab AND the model are set to the same wrong string, so the
 * identity guard passes. `write_note` only rejects a missing PARENT directory, and the parent
 * exists. The write gate returns `WouldRefuseIdentity` when an unrelated note already occupies
 * that path, but `WRITE_GATE_ENFORCE = false` (write_gate.rs:41) so the write proceeds anyway.
 *
 * WHAT THE USER SEES: nothing. The tab looks normal and reports saved, while every keystroke
 * after the move lands in a phantom file one directory above the real note — or, if a note of
 * the same name already sits there, silently overwrites it.
 *
 * These tests are the headless half of the proof. They pin the repath CONTRACT — that a moved
 * folder's descendants follow the path Rust actually returned. They do NOT prove the on-disk
 * outcome; that is the Boss's live recipe (Reproduce-First: static checks are not runtime
 * verification for this class).
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';

import { openTabs, moveItem, type OpenTab } from '$lib/libraries/store';
import { open as mOpen, closeAll } from '$lib/editor/noteSession';
import { compose } from '$lib/editor/noteModel';

const mockInvoke = vi.mocked(invoke);

const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;
const tab = (id: string, path: string): OpenTab => ({
	id, path, content: note(id, 'v1'),
	libraryName: 'L', libraryPath: '/L', name: path.split(/[\\/]/).pop() ?? path,
	libraryColor: '#000', history: [path], historyIndex: 0,
});

function openOne(id: string, path: string) {
	openTabs.set([tab(id, path)]);
	mOpen(id, path, note(id, 'v1'));
}

/** The model's own idea of its path. `compose` skips its identity guard on a falsy
 *  expectPath (noteModel.ts:297), so this reads the model path without asserting it. */
function modelPath(id: string): string | undefined {
	const r = compose(id, '');
	return r.ok ? r.path : undefined;
}

function tabPath(id: string): string | undefined {
	let out: string | undefined;
	openTabs.subscribe(ts => { out = ts.find(t => t.id === id)?.path; })();
	return out;
}

beforeEach(() => { closeAll(); openTabs.set([]); mockInvoke.mockReset(); });
afterEach(() => { openTabs.set([]); });

describe('moveItem — a moved folder’s descendants follow the path Rust returned', () => {
	it('keeps the moved folder’s own name for a DIRECT child', async () => {
		// Moving /L/Alpha/Sub into /L/Beta. Rust returns the new root: /L/Beta/Sub.
		openOne('t1', '/L/Alpha/Sub/Ideas.md');
		mockInvoke.mockResolvedValueOnce('/L/Beta/Sub');

		await moveItem('/L/Alpha/Sub', '/L/Beta');

		// The bug produced '/L/Beta/Ideas.md' — the 'Sub' segment dropped.
		expect(tabPath('t1')).toBe('/L/Beta/Sub/Ideas.md');
		expect(modelPath('t1')).toBe('/L/Beta/Sub/Ideas.md');
	});

	it('keeps it for a DEEPER descendant too', async () => {
		openOne('t2', '/L/Alpha/Sub/Deep/More.md');
		mockInvoke.mockResolvedValueOnce('/L/Beta/Sub');

		await moveItem('/L/Alpha/Sub', '/L/Beta');

		expect(tabPath('t2')).toBe('/L/Beta/Sub/Deep/More.md');
		expect(modelPath('t2')).toBe('/L/Beta/Sub/Deep/More.md');
	});

	it('works with Windows separators — the real platform', async () => {
		openOne('t3', 'E:\\Lib\\Alpha\\Sub\\Ideas.md');
		mockInvoke.mockResolvedValueOnce('E:\\Lib\\Beta\\Sub');

		await moveItem('E:\\Lib\\Alpha\\Sub', 'E:\\Lib\\Beta');

		expect(tabPath('t3')).toBe('E:\\Lib\\Beta\\Sub\\Ideas.md');
		expect(modelPath('t3')).toBe('E:\\Lib\\Beta\\Sub\\Ideas.md');
	});

	it('honours a destination Rust CHOSE rather than the one requested (collision suffix)', async () => {
		// Rust owns the final name — if `Sub` already existed in Beta it may return `Sub 1`.
		// Deriving from `targetFolder` cannot see that; deriving from the returned path can.
		openOne('t4', '/L/Alpha/Sub/Ideas.md');
		mockInvoke.mockResolvedValueOnce('/L/Beta/Sub 1');

		await moveItem('/L/Alpha/Sub', '/L/Beta');

		expect(tabPath('t4')).toBe('/L/Beta/Sub 1/Ideas.md');
		expect(modelPath('t4')).toBe('/L/Beta/Sub 1/Ideas.md');
	});

	it('the tab and its model never disagree about where the note lives', async () => {
		// The divergence that defeats compose()'s identity guard is them agreeing on a WRONG
		// path; this asserts they agree on the RIGHT one.
		openOne('t5', '/L/Alpha/Sub/Ideas.md');
		mockInvoke.mockResolvedValueOnce('/L/Beta/Sub');

		await moveItem('/L/Alpha/Sub', '/L/Beta');

		expect(modelPath('t5')).toBe(tabPath('t5'));
	});

	it('still moves a note moved DIRECTLY (the exact-match branch, which was already correct)', async () => {
		openOne('t6', '/L/Alpha/Ideas.md');
		mockInvoke.mockResolvedValueOnce('/L/Beta/Ideas.md');

		await moveItem('/L/Alpha/Ideas.md', '/L/Beta');

		expect(tabPath('t6')).toBe('/L/Beta/Ideas.md');
		expect(modelPath('t6')).toBe('/L/Beta/Ideas.md');
	});

	it('leaves unrelated tabs alone, including a same-prefix sibling folder', async () => {
		// '/L/Alpha/Submarine' starts with '/L/Alpha/Sub' as a STRING but is not inside it.
		openTabs.set([tab('t7', '/L/Alpha/Sub/Ideas.md'), tab('t8', '/L/Alpha/Submarine/Other.md')]);
		mOpen('t7', '/L/Alpha/Sub/Ideas.md', note('t7', 'v1'));
		mOpen('t8', '/L/Alpha/Submarine/Other.md', note('t8', 'v1'));
		mockInvoke.mockResolvedValueOnce('/L/Beta/Sub');

		await moveItem('/L/Alpha/Sub', '/L/Beta');

		expect(tabPath('t7')).toBe('/L/Beta/Sub/Ideas.md');
		expect(tabPath('t8')).toBe('/L/Alpha/Submarine/Other.md');
	});
});

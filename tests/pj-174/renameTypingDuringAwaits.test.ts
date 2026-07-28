/**
 * PJ-174 #1b — `renameItem` destroyed keystrokes typed DURING the rename.
 *
 * Found by the safety inspection on the #1 fix build. It is the SAME concern as #1 — an
 * unconditional model re-seed after awaits — at a surface my own "whole ecosystem" sweep missed,
 * because I grepped `openNoteModel` while EXCLUDING the file I was editing. The Whole-Ecosystem Fix
 * Law's canonical failure shape, committed while applying the Whole-Ecosystem Fix Law.
 *
 * The window is real and the gesture is ordinary:
 *   - `markCascading` (set at the top of `renameItem`) gates disk WRITES — `handleSave` /
 *     `handleFlush` — but NOT `onDocChange → editBody`, so the model keeps taking keystrokes.
 *   - the cascade freeze overlay cannot cover it either: the caller raises that only AFTER
 *     `renameItem` returns.
 *   - `await invoke('rename_item')` + `await readNote()` is hundreds of ms, and NotePane's Enter
 *     handler focuses the BODY, so "rename the title, press Enter, keep writing" lands the caret
 *     exactly where the typing is unprotected.
 *
 * Then: `clearWriteAhead` wiped the recovery net, and `openNoteModel(...)` replaced the dirty model
 * with disk bytes. The text was gone from the model, the screen and the net, with no error — and
 * the only tripwire (`openModel`'s LL-023 warn) is DEV-only, i.e. invisible in the release build.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';

import { openTabs, renameItem, clearAllCascading, type OpenTab } from '$lib/libraries/store';
import { open as mOpen, editBody, isDirty, bodyForView, closeAll } from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);
const note = (title: string, body: string) => `---\ntitle: ${title}\ncid_cn: C1\n---\n${body}`;
const tab = (id: string, path: string, content: string): OpenTab => ({
	id, path, content,
	libraryName: 'L', libraryPath: '/L', name: path.split('/').pop() ?? path,
	libraryColor: '#000', history: [path], historyIndex: 0,
});

beforeEach(() => {
	closeAll();
	openTabs.set([]);
	clearAllCascading();
	mockInvoke.mockReset();
});
afterEach(() => { openTabs.set([]); closeAll(); });

describe('PJ-174 #1b — typing during a rename survives', () => {
	it('keeps text typed inside the rename await window, and keeps its recovery net', async () => {
		openTabs.set([tab('a', '/L/Old.md', note('Old', 'first line'))]);
		mOpen('a', '/L/Old.md', note('Old', 'first line'));

		// The user presses Enter on the title; NotePane focuses the BODY and they keep writing
		// while the rename IPC is still in flight. This is what `markCascading` does not gate.
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'rename_item') {
				editBody('a', 'first line\nthe next sentence'); // typed mid-await
				return '/L/New.md';
			}
			if (cmd === 'read_note') return note('New', 'first line'); // disk, WITHOUT the sentence
			return undefined;
		});

		await renameItem('/L/Old.md', '/L/New.md');

		// Before the fix the model was replaced by the disk bytes and the sentence was gone from
		// the model, the screen AND the write-ahead net, with no error of any kind.
		expect(bodyForView('a')).toContain('the next sentence');
		expect(isDirty('a')).toBe(true); // still unsaved work — the app KNOWS it, so save-health can act

		// The identity still followed the rename, which is the other half of the contract: without
		// it the next save's compose would refuse the new path.
		const t = (await import('svelte/store')).get(openTabs)[0];
		expect(t.path).toBe('/L/New.md');
	});

	it('a CLEAN model still adopts the renamed file from disk (the fix must not over-block)', async () => {
		openTabs.set([tab('a', '/L/Old.md', note('Old', 'first line'))]);
		mOpen('a', '/L/Old.md', note('Old', 'first line'));
		expect(isDirty('a')).toBe(false);

		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'rename_item') return '/L/New.md';
			if (cmd === 'read_note') return note('New', 'first line');
			return undefined;
		});

		await renameItem('/L/Old.md', '/L/New.md');

		const t = (await import('svelte/store')).get(openTabs)[0];
		expect(t.path).toBe('/L/New.md');
		expect(t.content).toContain('title: New'); // the rename's frontmatter rewrite IS adopted
		expect(t.reloadVersion ?? 0).toBeGreaterThan(0); // …and the pane remounts on it
		expect(isDirty('a')).toBe(false);
	});

	it('a canonical rename (path unchanged) also preserves mid-rename typing', async () => {
		openTabs.set([tab('a', '/L/Note.md', note('Old', 'body'))]);
		mOpen('a', '/L/Note.md', note('Old', 'body'));

		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'rename_item') {
				editBody('a', 'body\ntyped during');
				return '/L/Note.md'; // canonical: the file stays put, only the title changed
			}
			if (cmd === 'read_note') return note('New', 'body');
			return undefined;
		});

		await renameItem('/L/Note.md', '/L/New.md');
		expect(bodyForView('a')).toContain('typed during');
		expect(isDirty('a')).toBe(true);
	});
});

describe('PJ-174 #1c — a property edited during a cascade is kept, not dropped', () => {
	it('lands in the model even though the disk write is gated', async () => {
		const { saveTabContent, markCascadingLibrary, clearCascadingLibrary } =
			await import('$lib/libraries/store');
		const { getModel } = await import('$lib/editor/noteModel');

		openTabs.set([tab('a', '/L/a.md', note('A', 'body'))]);
		mOpen('a', '/L/a.md', note('A', 'body'));
		mockInvoke.mockImplementation(async () => undefined);

		// A rename cascade is running over the whole library (the LIVE gate from #1).
		markCascadingLibrary('/L');
		try {
			// The right-sidebar PropertyEditor is NOT under the freeze overlay, so this is reachable.
			await saveTabContent('a', '/L/a.md', [
				{ key: 'stage', value: 'sapling', type: 'text' } as never,
			], 'body');

			// Before the fix the gate returned BEFORE the model push: not written AND not kept.
			const props = getModel('a')?.props ?? [];
			expect(props.some((p: { key: string; value: unknown }) =>
				p.key === 'stage' && p.value === 'sapling')).toBe(true);
			expect(isDirty('a')).toBe(true); // held as unsaved work, so a later save persists it
		} finally {
			clearCascadingLibrary('/L');
		}
	});

	it('still does NOT write to disk during the cascade (the gate must keep gating)', async () => {
		const { saveTabContent, markCascadingLibrary, clearCascadingLibrary } =
			await import('$lib/libraries/store');
		openTabs.set([tab('a', '/L/a.md', note('A', 'body'))]);
		mOpen('a', '/L/a.md', note('A', 'body'));

		const writes: string[] = [];
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'write_note') writes.push(cmd);
			return undefined;
		});

		markCascadingLibrary('/L');
		try {
			await saveTabContent('a', '/L/a.md', [
				{ key: 'stage', value: 'sapling', type: 'text' } as never,
			], 'body');
			expect(writes).toEqual([]); // F2 post-cascade-stomp is still prevented
		} finally {
			clearCascadingLibrary('/L');
		}
	});
});

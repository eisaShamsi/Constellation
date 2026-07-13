/**
 * PJ-092 redo (flush-gate-exclude) — the frontend contract that feeds the
 * rename cascade's exclude-list.
 *
 * `flushAllTabsInLibrary` returns the paths whose flush was NOT durable. The
 * cascade passes those to the Rust walker as the exclude-list, so a note the
 * user couldn't flush is NEVER rewritten on disk → no model↔disk divergence →
 * no data-loss, no reactive freeze. Each dirty tab flushes through the BOUNDED
 * re-flush loop (H2: a keystroke during the awaited write is caught, so a note
 * is never reported clean while still dirty).
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';

import { openTabs, flushAllTabsInLibrary, type OpenTab } from '$lib/libraries/store';
import { open as mOpen, editBody, isDirty, closeAll } from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);
const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;
const tab = (id: string, path: string, content: string): OpenTab => ({
	id, path, content,
	libraryName: 'L', libraryPath: '/L', name: path.split('/').pop() ?? path,
	libraryColor: '#000', history: [path], historyIndex: 0,
});
function openOne(id: string, path: string, content: string) {
	openTabs.set([tab(id, path, content)]);
	mOpen(id, path, content);
}

beforeEach(() => { closeAll(); openTabs.set([]); });
afterEach(() => { openTabs.set([]); });

describe('flushAllTabsInLibrary — reports the not-durably-flushed paths (PJ-092)', () => {
	it('returns the path of a dirty tab whose write_note FAILED — edits preserved', async () => {
		openOne('b', '/L/b.md', note('B', 'v1'));
		editBody('b', 'v1\nmy unsaved work');
		expect(isDirty('b')).toBe(true);

		mockInvoke.mockReset();
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'write_note') throw new Error('locked .md');
			return undefined;
		});

		const failed = await flushAllTabsInLibrary('/L');
		expect(failed).toEqual(['/L/b.md']);   // the walker will EXCLUDE it → never rewritten
		expect(isDirty('b')).toBe(true);        // model stays dirty — edits are the sole copy, retried by save-health
	});

	it('returns EMPTY when the flush is durable', async () => {
		openOne('c', '/L/c.md', note('C', 'v1'));
		editBody('c', 'v1\nedit');
		expect(isDirty('c')).toBe(true);

		mockInvoke.mockReset();
		mockInvoke.mockImplementation(async () => undefined); // all writes succeed

		const failed = await flushAllTabsInLibrary('/L');
		expect(failed).toEqual([]);
		expect(isDirty('c')).toBe(false);       // durable → model clean → disk == model
	});

	it('a CLEAN tab is never flushed and never reported', async () => {
		openOne('d', '/L/d.md', note('D', 'v1'));
		expect(isDirty('d')).toBe(false);
		mockInvoke.mockReset();
		mockInvoke.mockImplementation(async () => undefined);
		expect(await flushAllTabsInLibrary('/L')).toEqual([]);
	});
});

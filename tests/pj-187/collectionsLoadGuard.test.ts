/**
 * PJ-187 (register) — a FAILED READ must never present as "you have nothing", because the
 * next write turns that emptiness into the truth on disk.
 *
 * Collections membership lives ONLY in `collections.json` — the store's own comment says so
 * (*"Membership ONLY lives here"*; adding a note to a collection never writes the note's
 * file). So `saveCollections` is the sole writer of that user-authored data, and it had two
 * independent ways to lose all of it:
 *
 *   1. it was fire-and-forget with a `console.error`, which release builds discard entirely —
 *      starring a note looked like it worked and was gone at the next launch;
 *   2. `loadCollections` swallowed a failed READ and left the store at its empty default, so
 *      the next star/unstar wrote that emptiness over a perfectly good file. A sync tool
 *      holding the file for a moment was enough to erase every collection the user had.
 *
 * This is the shape shared by five sites in the register — settings, workspaces, collections,
 * property types, and a universe manifest — and it is the one that destroys data the user
 * already has, rather than merely losing the change in hand.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import { loadCollections, toggleStarred, collectionSets, collectionsError } from '$lib/libraries/store';

const mockInvoke = vi.mocked(invoke);
const calls = () => mockInvoke.mock.calls.map((c) => c[0] as string);
const read = <T,>(s: { subscribe: (r: (v: T) => void) => () => void }): T => {
	let v!: T;
	s.subscribe((x) => (v = x))();
	return v;
};

beforeEach(() => {
	collectionSets.set([]);
	collectionsError.set(null);
	mockInvoke.mockReset();
});

describe('PJ-187 — a failed collections READ must disable writes', () => {
	it('a read failure never lets a later change overwrite the file', async () => {
		// The file is momentarily unreadable — a sync tool, antivirus, a half-written file.
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_collections') throw new Error('EBUSY: locked');
			return undefined;
		});

		await loadCollections();
		expect(read(collectionsError)).toBeTruthy();

		// The user now stars a note. Before the fix this wrote the EMPTY store over the
		// real file, destroying every collection they had.
		mockInvoke.mockClear();
		toggleStarred({ type: 'note', path: '/lib/n.md', name: 'n' });
		await new Promise((r) => setTimeout(r, 0));

		expect(calls()).not.toContain('save_universe_collections');
	});

	it('after a successful read, saving works normally', async () => {
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_collections') return [];
			if (cmd === 'read_universe_bookmarks') return [];
			return undefined;
		});

		await loadCollections();
		expect(read(collectionsError)).toBeNull();

		mockInvoke.mockClear();
		mockInvoke.mockImplementation(async () => undefined);
		toggleStarred({ type: 'note', path: '/lib/n.md', name: 'n' });
		await new Promise((r) => setTimeout(r, 0));

		expect(calls()).toContain('save_universe_collections');
	});

	it('a universe SWITCH whose read fails never lets the OLD universe leak through', async () => {
		// Universe A loads fine and holds one collection.
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_collections')
				return [{ id: 'col_a', name: 'From Universe A', createdAt: 1, items: [] }];
			if (cmd === 'read_universe_bookmarks') return [];
			return undefined;
		});
		await loadCollections();
		expect(read(collectionSets).some((c: any) => c.name === 'From Universe A')).toBe(true);

		// The user switches to universe B, whose collections file is momentarily unreadable.
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_collections') throw new Error('EBUSY: locked');
			if (cmd === 'read_universe_bookmarks') return [];
			return undefined;
		});
		await loadCollections();

		// Before the fix: the latch stayed TRUE from A and the store still held A's list —
		// so the next star wrote universe A's collections over universe B's file.
		expect(read(collectionSets)).toEqual([]); // B never shows A's collections
		mockInvoke.mockClear();
		toggleStarred({ type: 'note', path: '/lib/n.md', name: 'n' });
		await new Promise((r) => setTimeout(r, 0));
		expect(calls()).not.toContain('save_universe_collections'); // and writes stay refused
	});

	it('overlapping saves serialise, and the LAST write carries the newest list', async () => {
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_collections') return [];
			if (cmd === 'read_universe_bookmarks') return [];
			return undefined;
		});
		await loadCollections();

		// Every save call records the payload it actually sent.
		const sent: unknown[][] = [];
		let block: (() => void) | null = null;
		mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
			if (cmd === 'save_universe_collections') {
				sent.push(args.collections);
				// Hold the FIRST write open so the second toggle lands inside its window.
				if (sent.length === 1) await new Promise<void>((r) => (block = r));
			}
			return undefined;
		});

		toggleStarred({ type: 'note', path: '/lib/one.md', name: 'one' });
		await new Promise((r) => setTimeout(r, 0));
		toggleStarred({ type: 'note', path: '/lib/two.md', name: 'two' });
		block!();
		await new Promise((r) => setTimeout(r, 20));

		// The final write on the file must contain BOTH stars — before the fix, an
		// interleaved retry could land the older payload last and drop the second star.
		const last = sent[sent.length - 1] as Array<{ items: Array<{ path: string }> }>;
		const starred = last.find((c: any) => c.id === 'starred') ?? last[0];
		const paths = (starred?.items ?? []).map((i) => i.path);
		expect(paths).toContain('/lib/one.md');
		expect(paths).toContain('/lib/two.md');
	});

	it('a failed WRITE retries once and then surfaces, instead of logging into the void', async () => {
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_universe_collections') return [];
			if (cmd === 'read_universe_bookmarks') return [];
			return undefined;
		});
		await loadCollections();

		let attempts = 0;
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'save_universe_collections') { attempts++; throw new Error('EBUSY'); }
			return undefined;
		});
		toggleStarred({ type: 'note', path: '/lib/n.md', name: 'n' });
		await new Promise((r) => setTimeout(r, 0));

		expect(attempts).toBe(2); // one retry
		expect(read(collectionsError)).toBeTruthy(); // and it is VISIBLE, not a console line
	});
});

/**
 * PJ-187 (Boss Stage-1, 2026-07-29) — **every** way Constellation removes a note from its place
 * must put it where the user's "Deleted files" setting says.
 *
 * Found by the Boss failing a step of my own test: I sent him to look for two Overwrite-displaced
 * notes in the trash, and they were not there. They existed and were perfectly intact — in a
 * different tree entirely.
 *
 * `deleteWithSetting` read `trashDestination` + `trashFolderScope`. The three other displacement
 * paths — Overwrite on create, Overwrite on rename, and the PJ-088 conflict sidecar — called
 * `moveToTrash(path, libraryPath)`, which invoked a Rust command that derives its trash root from
 * the library path it also validates against. It could not consult the setting even in principle.
 *
 * Measured on the Boss's real universe: `trashDestination: 'local'`, `trashFolderScope:
 * 'universe'`. Delete filed to `E:\Constellation Universes\Eisa Cognitive Knowledge\.trash`;
 * Overwrite filed to `E:\Cognitive Knowledge\Eisa Test\.trash`. Because that universe's libraries
 * live OUTSIDE its root, those are not neighbouring folders — they are different trees. The note
 * stayed recoverable and stayed invisible, which for a user who goes looking is the same thing.
 *
 * With the DEFAULT `trashDestination: 'system'` the divergence is worse: Delete uses the Windows
 * Recycle Bin, while Overwrite silently creates a `.trash` folder inside the library that the
 * user never opted into and will never think to open.
 *
 * The fixture below is the Boss's actual layout — a universe root with its libraries somewhere
 * else on disk — because that is the arrangement in which the two answers differ most.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	moveToTrash,
	deleteWithSetting,
	resolveTrashDestination,
	appSettings,
	libraries,
	libraryStats,
} from '$lib/libraries/store';

const mockInvoke = vi.mocked(invoke);

/** The Boss's shape: the universe root is one tree, its libraries are on another path entirely. */
const UNIVERSE_ROOT = 'E:/Constellation Universes/Eisa Cognitive Knowledge';
const LIBRARY_ROOT = 'E:/Cognitive Knowledge/Eisa Test';
const NOTE = 'E:/Cognitive Knowledge/Eisa Test/Alpha/Overwrite Test.md';

const LIB_ROWS = [
	{ id: 'universe_notes_x', name: 'Eisa Cognitive Knowledge', path: UNIVERSE_ROOT, is_universe_notes: true },
	{ id: 'library_y', name: 'Eisa Test', path: LIBRARY_ROOT, is_universe_notes: false },
];

/** The one call that matters: what destination did we hand the backend? */
const deleteCall = () => {
	const c = mockInvoke.mock.calls.find((c) => c[0] === 'delete_path');
	return c?.[1] as { path: string; mode: string; trashRoot: string | null } | undefined;
};

const setSettings = (trashDestination: string, trashFolderScope: string) =>
	appSettings.update((s) => ({ ...s, trashDestination, trashFolderScope }) as typeof s);

beforeEach(() => {
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
	libraries.set(LIB_ROWS as never);
	libraryStats.set(LIB_ROWS as never);
});

describe('PJ-187 — Overwrite must file the displaced note where DELETE files it', () => {
	it('universe scope: Overwrite uses the universe root, exactly as Delete does', async () => {
		setSettings('local', 'universe');

		await moveToTrash(NOTE);
		const viaOverwrite = deleteCall();

		mockInvoke.mockClear();
		await deleteWithSetting(NOTE);
		const viaDelete = deleteCall();

		expect(viaOverwrite?.mode).toBe('trash');
		expect(viaOverwrite?.trashRoot).toBe(UNIVERSE_ROOT);
		// The property that actually matters — the two paths agree.
		expect(viaOverwrite?.trashRoot).toBe(viaDelete?.trashRoot);
		expect(viaOverwrite?.mode).toBe(viaDelete?.mode);
	});

	it('library scope: both use the note\'s own library root', async () => {
		setSettings('local', 'library');

		await moveToTrash(NOTE);
		const viaOverwrite = deleteCall();
		mockInvoke.mockClear();
		await deleteWithSetting(NOTE);
		const viaDelete = deleteCall();

		expect(viaOverwrite?.trashRoot).toBe(LIBRARY_ROOT);
		expect(viaOverwrite?.trashRoot).toBe(viaDelete?.trashRoot);
	});

	it('system trash (the DEFAULT): Overwrite goes to the Recycle Bin, not a surprise .trash folder', async () => {
		setSettings('system', 'library');

		await moveToTrash(NOTE);
		const viaOverwrite = deleteCall();

		expect(viaOverwrite?.mode).toBe('system');
		expect(viaOverwrite?.trashRoot).toBeNull();
		// Before the fix this path created `<library>/.trash` regardless — a folder the user,
		// having chosen the Recycle Bin, never asked for and would never look in.
	});

	it('Overwrite no longer reaches the library-scoped Rust command at all', async () => {
		setSettings('local', 'universe');
		await moveToTrash(NOTE);
		expect(mockInvoke.mock.calls.map((c) => c[0])).not.toContain('move_to_trash');
	});

	it('the resolver is the single source of truth for every displacement path', () => {
		setSettings('local', 'universe');
		expect(resolveTrashDestination(NOTE)).toEqual({ mode: 'trash', trashRoot: UNIVERSE_ROOT });

		setSettings('local', 'library');
		expect(resolveTrashDestination(NOTE)).toEqual({ mode: 'trash', trashRoot: LIBRARY_ROOT });

		setSettings('system', 'universe');
		expect(resolveTrashDestination(NOTE)).toEqual({ mode: 'system', trashRoot: null });
	});

	it('library scope picks the MOST SPECIFIC library when one nests inside another', () => {
		const NESTED = 'E:/Cognitive Knowledge/Eisa Test/Nested';
		const rows = [...LIB_ROWS, { id: 'library_z', name: 'Nested', path: NESTED, is_universe_notes: false }];
		libraries.set(rows as never);
		libraryStats.set(rows as never);
		setSettings('local', 'library');

		expect(resolveTrashDestination(NESTED + '/Deep/Note.md').trashRoot).toBe(NESTED);
	});

	it('refuses rather than guessing when no trash root can be resolved', () => {
		libraries.set([] as never);
		libraryStats.set([] as never);
		setSettings('local', 'universe');
		expect(() => resolveTrashDestination(NOTE)).toThrow(/trash location/i);
	});
});

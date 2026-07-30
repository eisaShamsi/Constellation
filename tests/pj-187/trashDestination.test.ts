/**
 * PJ-187 → MIG-108 — **every** way Constellation removes a note from its place files it where
 * the "Deleted files" setting says, and all displacement paths agree with each other.
 *
 * History: PJ-187's Stage-1 found four displacement paths disagreeing on the destination
 * (Delete honoured the setting; Overwrite ×2 and the conflict sidecar hardcoded the library's
 * `.trash`). The fix was ONE shared resolver. MIG-108 (Boss ruling, 2026-07-29) then collapsed
 * the scope axis entirely: a universe has ONE `.trash`, at its root, the way it has one root.
 * This suite pins the collapsed contract:
 *
 *   · `local`  → every path files to `<universe root>/.trash`, wherever the note lives;
 *   · `system` → every path uses the OS Recycle Bin, and no `.trash` folder is invented;
 *   · a legacy settings.json still carrying `trashFolderScope` loads clean, behaves
 *     universe-root, and the stale key is PURGED (not round-tripped forever by the
 *     spread-over-defaults load path);
 *   · Overwrite and Delete are indistinguishable at the backend boundary.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	moveToTrash,
	deleteWithSetting,
	resolveTrashDestination,
	applyParsedSettings,
	appSettings,
	libraries,
	libraryStats,
} from '$lib/libraries/store';

const mockInvoke = vi.mocked(invoke);

/** Post-MIG-108 shape: every library lives UNDER the universe root. */
const ROOT = 'E:/Constellation Universes/Eisa Cognitive Knowledge';
const LIB = `${ROOT}/Eisa Test`;
const NOTE = `${LIB}/Alpha/Note.md`;
const ROWS = [
	{ id: 'u', name: 'Eisa Cognitive Knowledge', path: ROOT, is_universe_notes: true },
	{ id: 'l', name: 'Eisa Test', path: LIB, is_universe_notes: false },
];

const deleteCall = () => {
	const c = mockInvoke.mock.calls.find((c) => c[0] === 'delete_path');
	return c?.[1] as { path: string; mode: string; trashRoot: string | null } | undefined;
};

const setDest = (trashDestination: string) =>
	appSettings.update((s) => ({ ...s, trashDestination }) as typeof s);

beforeEach(() => {
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
	libraries.set(ROWS as never);
	libraryStats.set(ROWS as never);
});

describe('MIG-108 — one universe, ONE trash', () => {
	it('local: every note, from any library, files to the universe-root .trash', () => {
		setDest('local');
		expect(resolveTrashDestination(NOTE)).toEqual({ mode: 'trash', trashRoot: ROOT });
		expect(resolveTrashDestination(`${ROOT}/loose.md`)).toEqual({ mode: 'trash', trashRoot: ROOT });
	});

	it('system (the default): the Recycle Bin, and no .trash folder is invented', () => {
		setDest('system');
		expect(resolveTrashDestination(NOTE)).toEqual({ mode: 'system', trashRoot: null });
	});

	it('Overwrite and Delete hand the backend IDENTICAL destinations', async () => {
		setDest('local');
		await moveToTrash(NOTE);
		const viaOverwrite = deleteCall();
		mockInvoke.mockClear();
		await deleteWithSetting(NOTE);
		const viaDelete = deleteCall();

		expect(viaOverwrite?.mode).toBe('trash');
		expect(viaOverwrite?.trashRoot).toBe(ROOT);
		expect(viaOverwrite?.mode).toBe(viaDelete?.mode);
		expect(viaOverwrite?.trashRoot).toBe(viaDelete?.trashRoot);
	});

	it('the retired move_to_trash command is never reached', async () => {
		setDest('local');
		await moveToTrash(NOTE);
		expect(mockInvoke.mock.calls.map((c) => c[0])).not.toContain('move_to_trash');
	});

	it('a LEGACY settings.json carrying trashFolderScope loads clean and the key is purged', () => {
		// The load path spreads parsed keys over the defaults, and saveSettings round-trips
		// the WHOLE object — an unpurged stale key would live in settings.json forever.
		applyParsedSettings({ trashDestination: 'local', trashFolderScope: 'library' } as never);
		const s = appSettings as unknown as { subscribe: (r: (v: unknown) => void) => () => void };
		let snapshot: Record<string, unknown> = {};
		s.subscribe((v) => (snapshot = v as Record<string, unknown>))();
		expect(snapshot.trashDestination).toBe('local');
		expect('trashFolderScope' in snapshot).toBe(false);
		// …and the old 'library' preference cannot resurrect a per-library destination:
		expect(resolveTrashDestination(NOTE).trashRoot).toBe(ROOT);
	});

	it('refuses rather than guessing when no universe root is registered', () => {
		libraries.set([] as never);
		setDest('local');
		expect(() => resolveTrashDestination(NOTE)).toThrow(/trash location/i);
	});
});

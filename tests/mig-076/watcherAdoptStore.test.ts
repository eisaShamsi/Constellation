/**
 * PJ-070 §2 — the STORE-BOUNDARY test for `adoptExternalChangeIntoTabs`.
 *
 * Recipe O (runtimeHarness.test.ts) proves the mechanism at the MODEL layer. This drives the
 * real store helper end-to-end against a seeded `openTabs` + an INJECTED disk reader (no Tauri),
 * so the "did the wiring actually adopt + remount" residual is now a genuine RED→GREEN store test:
 *   - clean tab → adopts the disk + bumps reloadVersion ONLY on the adopter;
 *   - dirty tab + genuine external change → routes to the conflict hook, never clobbers, no bump;
 *   - a cascade-owned path, an echo/spurious touch, a deleted file, and the two Focus branches.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { Text } from '@codemirror/state';
import { openTabs, adoptExternalChangeIntoTabs, isReseeding, markCascading, clearCascading, type OpenTab } from '$lib/libraries/store';
import { open as mOpen, editBody, isDirty, bodyForView, closeAll } from '$lib/editor/noteSession';

const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;

const tab = (id: string, path: string, content: string): OpenTab => ({
	id, path, content,
	libraryName: 'L', libraryPath: '/L', name: path.split('/').pop() ?? path,
	libraryColor: '#000', history: [path], historyIndex: 0,
});

/** Seed one open tab + its model from the same on-disk content. */
function openOne(id: string, path: string, content: string) {
	openTabs.set([tab(id, path, content)]);
	mOpen(id, path, content);
}
const reloadVersionOf = (path: string) => get(openTabs).find((t) => t.path === path)?.reloadVersion ?? 0;
const contentOf = (path: string) => get(openTabs).find((t) => t.path === path)?.content ?? '';

beforeEach(() => { closeAll(); openTabs.set([]); });
afterEach(() => { openTabs.set([]); });

describe('adoptExternalChangeIntoTabs — clean adopt (GREEN)', () => {
	it('adopts the external disk into the model, bumps reloadVersion, updates content, clears the reseed mark', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		const external = note('N', 'v1\nEXTERNAL');
		await adoptExternalChangeIntoTabs(['/n.md'], {}, async () => external);

		expect(bodyForView('t')).toBe('v1\nEXTERNAL');       // model adopted the disk
		expect(reloadVersionOf('/n.md')).toBe(1);            // NotePane will remount
		expect(contentOf('/n.md')).toBe(external);           // store content refreshed
		expect(isReseeding('/n.md')).toBe(false);            // mark cleared after tick()
	});
});

describe('adoptExternalChangeIntoTabs — dirty conflict (local wins, external preserved)', () => {
	it('routes a dirty tab with a genuine external change to the conflict hook, never adopts, no bump', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		editBody('t', 'v1\nmy unsaved work'); // dirty
		expect(isDirty('t')).toBe(true);
		const external = note('N', 'v1\nEXTERNAL');
		const conflict = vi.fn();
		await adoptExternalChangeIntoTabs(['/n.md'], { conflict }, async () => external);

		expect(conflict).toHaveBeenCalledTimes(1);
		expect(conflict).toHaveBeenCalledWith('/n.md', 'n.md', external);
		expect(bodyForView('t')).toBe('v1\nmy unsaved work'); // local NOT clobbered
		expect(reloadVersionOf('/n.md')).toBe(0);             // no remount
	});
});

describe('adoptExternalChangeIntoTabs — the guards', () => {
	it('a cascade-owned path is skipped (the rename cascade force-adopt owns it)', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		markCascading('/n.md');
		try {
			await adoptExternalChangeIntoTabs(['/n.md'], {}, async () => note('N', 'v1\nEXTERNAL'));
			expect(bodyForView('t')).toBe('v1');            // untouched
			expect(reloadVersionOf('/n.md')).toBe(0);
		} finally { clearCascading('/n.md'); }
	});

	it('an echo of our own content (disk === model) is a no-op (no bump, no conflict)', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		const conflict = vi.fn();
		await adoptExternalChangeIntoTabs(['/n.md'], { conflict }, async () => note('N', 'v1'));
		expect(reloadVersionOf('/n.md')).toBe(0);
		expect(conflict).not.toHaveBeenCalled();
	});

	it('a deleted file (read rejects) does not throw or mutate', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		await expect(adoptExternalChangeIntoTabs(['/n.md'], {}, async () => { throw new Error('ENOENT'); })).resolves.toBeUndefined();
		expect(bodyForView('t')).toBe('v1');
		expect(reloadVersionOf('/n.md')).toBe(0);
	});

	it('a path not open in any tab is ignored', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		const read = vi.fn(async () => note('N', 'x'));
		await adoptExternalChangeIntoTabs(['/other.md'], {}, read);
		expect(read).not.toHaveBeenCalled(); // never even read — O(open ∩ changed)
	});
});

describe('adoptExternalChangeIntoTabs — Focus-mode handoff (hazard #7)', () => {
	it('with NO focusReseed hook, the focus note is left untouched (never adopt-without-reseed)', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		await adoptExternalChangeIntoTabs(['/n.md'], { focusPath: '/n.md' }, async () => note('N', 'v1\nEXTERNAL'));
		expect(bodyForView('t')).toBe('v1');       // NOT adopted — Focus can't be reseeded yet
		expect(reloadVersionOf('/n.md')).toBe(0);  // and NO NotePane remount for a focus note
	});

	it('with a focusReseed hook, the model adopts, content refreshes, and focusReseed remounts FocusPane', async () => {
		openOne('t', '/n.md', note('N', 'v1'));
		const focusReseed = vi.fn();
		await adoptExternalChangeIntoTabs(['/n.md'], { focusPath: '/n.md', focusReseed }, async () => note('N', 'v1\nEXTERNAL'));
		expect(bodyForView('t')).toBe('v1\nEXTERNAL');       // model adopted
		expect(focusReseed).toHaveBeenCalledWith('/n.md');   // FocusPane remounts on the fresh model
		expect(contentOf('/n.md')).toBe(note('N', 'v1\nEXTERNAL')); // store tab refreshed uniformly
		// reloadVersion also bumps (uniform), but it's inert while NotePane is unmounted in Focus mode:
		expect(reloadVersionOf('/n.md')).toBe(1);
	});
});

describe('spurious-dirty class fix (safety-inspection finding [7] + altitude A1) — a no-op string push never dirties', () => {
	it('an identical-content string editBody (a merely-viewed note teardown / focus flush) is a NO-OP → the model stays clean → the external edit ADOPTS, no phantom conflict', async () => {
		// The class fix lives in setBody (noteModel.ts): the STRING form no-ops when content is unchanged.
		// This closes the whole class at the source — NotePane's handleFlush, FocusPane's onflush, and
		// flushAllDirtyTabs all push a string on teardown; a merely-viewed note must not go dirty.
		openOne('t', '/n.md', note('N', 'v1'));
		editBody('t', 'v1'); // the redundant teardown push (identical content)
		expect(isDirty('t')).toBe(false); // stays CLEAN (the fix; was spuriously dirty before)

		// So an external edit is cleanly ADOPTED — not refused into a phantom `.conflict` sidecar for a
		// note the user never edited (the exact downstream harm the fix eliminates):
		const conflict = vi.fn();
		await adoptExternalChangeIntoTabs(['/n.md'], { conflict }, async () => note('N', 'v2 external'));
		expect(conflict).not.toHaveBeenCalled();
		expect(bodyForView('t')).toBe('v2 external'); // adopted, not stale
	});

	it('a REAL edit (different string) still dirties', () => {
		openOne('t', '/n.md', note('N', 'v1'));
		editBody('t', 'v1 edited');
		expect(isDirty('t')).toBe(true);
	});

	it('the Text (keystroke) path stays O(1) ref-based — an identical-content Text still bumps (Rule 1, unchanged)', () => {
		openOne('t', '/n.md', note('N', 'v1'));
		editBody('t', Text.of(['v1'])); // a fresh Text object, same content — the per-keystroke onDocChange shape
		expect(isDirty('t')).toBe(true); // bumps: the hot path must NOT pay an O(N) content compare
	});
});

/**
 * Safety inspection 2026-08-01 (APP-KILLER, self-inflicted) — WINDOWS PATHS.
 *
 * Every test above drives POSIX `/n.md` paths, where `normPath` is the identity function.
 * That is precisely why the whole suite stayed green while the 2026-08-01 half-normalization
 * (byPath keyed by normPath, looked up with the RAW tab path) silently disabled the entire
 * external-adopt/conflict arbitration on Windows — the ONLY platform Constellation ships on
 * today, where every tab path is a backslash string from `read_dir_recursive`.
 *
 * These cases pin BOTH halves of the arbitration to a real Windows-shaped path. Under the
 * broken version the clean case adopts nothing (the model keeps v1, reloadVersion stays 0)
 * and the dirty case never reaches the conflict hook — a silent overwrite on the next save.
 */
describe('adoptExternalChangeIntoTabs — Windows backslash paths (the shipping platform)', () => {
	const WIN = 'E:\\Constellation Universes\\Eisa Cognitive Knowledge\\Lib\\Ideas.md';

	it('a CLEAN tab at a backslash path still adopts the external disk', async () => {
		openOne('t', WIN, note('N', 'v1'));
		const external = note('N', 'v1\nEXTERNAL');
		await adoptExternalChangeIntoTabs([WIN], {}, async () => external);

		expect(bodyForView('t')).toBe('v1\nEXTERNAL');
		expect(reloadVersionOf(WIN)).toBe(1);
		expect(contentOf(WIN)).toBe(external);
	});

	it('a DIRTY tab at a backslash path still reaches the conflict hook (never a silent clobber)', async () => {
		openOne('t', WIN, note('N', 'v1'));
		editBody('t', 'v1\nmy unsaved work');
		const external = note('N', 'v1\nEXTERNAL');
		const conflict = vi.fn();
		await adoptExternalChangeIntoTabs([WIN], { conflict }, async () => external);

		expect(conflict).toHaveBeenCalledTimes(1);
		expect(bodyForView('t')).toBe('v1\nmy unsaved work');
		expect(reloadVersionOf(WIN)).toBe(0);
	});

	it('a MIXED-separator announce path resolves to the same tab (the finding-37 producer shape)', async () => {
		openOne('t', WIN, note('N', 'v1'));
		const external = note('N', 'v1\nEXTERNAL');
		// The watcher/announce side may carry forward slashes for the same file.
		await adoptExternalChangeIntoTabs([WIN.replace(/\\/g, '/')], {}, async () => external);

		expect(bodyForView('t')).toBe('v1\nEXTERNAL');
		expect(reloadVersionOf(WIN)).toBe(1);
	});
});

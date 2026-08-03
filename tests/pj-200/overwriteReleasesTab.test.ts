/**
 * Triage concern #2 (2026-08-02), ranked APP-KILLER — **"Overwrite" could destroy the note the
 * user chose to KEEP.**
 *
 * The collision dialog's Overwrite does two things in order: trash the existing note, then
 * rename/create the other note onto that same path. The first step called `moveToTrash`, which
 * removed the FILE and left the tab open — model live, still believing it owned that path. One
 * flush later (a debounced save, a tab switch, app close) that stale model wrote its content
 * over the survivor. No error: from the app's point of view a tab saved itself.
 *
 * `deleteWithSetting` has always closed tabs and disposed models. `moveToTrash` never did, and
 * `moveToTrash` is what all three displacement paths use — Overwrite-on-create,
 * Overwrite-on-rename, and the PJ-088 conflict sidecar. PJ-187 had already unified these paths
 * on *where the displaced file goes* and left *what happens to its open tab* in only one of
 * them: half a sweep inside the very fix meant to make them agree. The cure is the shared
 * `releaseTabsForVacatedPath`, so there is nothing left for `moveToTrash` to fall behind.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	moveToTrash,
	deleteWithSetting,
	openTabs,
	libraries,
	appSettings,
	activeTabId,
	focusedTabId,
	splitActive,
	closeTab,
} from '$lib/libraries/store';
import { get } from 'svelte/store';

const mockInvoke = vi.mocked(invoke);

const ROOT = 'E:\\U';
const KEEP = 'E:\\U\\Keep.md';
const VICTIM = 'E:\\U\\Victim.md';

/** Put a tab on the board the way the app does, without dragging in openNoteTab's IPC. */
function seedTab(id: string, path: string) {
	openTabs.update((ts) => [
		...ts,
		{ id, path, name: path.split('\\').pop()!, content: `body of ${path}` } as never,
	]);
}

beforeEach(() => {
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
	openTabs.set([]);
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	// One universe root, so resolveTrashDestination can answer.
	libraries.set([{ path: ROOT, name: 'universe_notes', is_universe_notes: true } as never]);
	appSettings.update((s) => ({ ...s, trashDestination: 'local' }) as never);
});

describe('a note that leaves its path takes its tab with it', () => {
	it('moveToTrash releases the displaced note\'s tab — the Overwrite app-killer', () => {
		seedTab('t_victim', VICTIM);
		seedTab('t_keep', KEEP);
		expect(get(openTabs).map((t) => t.path)).toContain(VICTIM);

		return moveToTrash(VICTIM).then(() => {
			const paths = get(openTabs).map((t) => t.path);
			expect(paths).not.toContain(VICTIM); // ← the fix: no live model owning a vacated path
			expect(paths).toContain(KEEP); // and the survivor is untouched
		});
	});

	it('leaves unrelated tabs alone', async () => {
		seedTab('t_keep', KEEP);
		await moveToTrash(VICTIM);
		expect(get(openTabs).map((t) => t.path)).toEqual([KEEP]);
	});

	it('releases tabs BENEATH a vacated folder, not just an exact match', async () => {
		seedTab('t_child', 'E:\\U\\Folder\\Child.md');
		await moveToTrash('E:\\U\\Folder');
		expect(get(openTabs)).toHaveLength(0);
	});

	/**
	 * The invariant that actually matters, stated as a test: the two displacement paths must
	 * agree. If someone adds a third, or changes one of these, this is what fails.
	 */
	it('moveToTrash and deleteWithSetting agree about the tab', async () => {
		seedTab('t_a', VICTIM);
		await moveToTrash(VICTIM);
		const afterTrash = get(openTabs).length;

		seedTab('t_b', VICTIM);
		await deleteWithSetting(VICTIM);
		const afterDelete = get(openTabs).length;

		expect(afterTrash).toBe(afterDelete);
		expect(afterTrash).toBe(0);
	});

	/**
	 * RED-proof kept in the suite: the shape this replaced. `moveToTrash` used to be exactly
	 * `deletePath` and nothing else, so the tab survived — which is the whole defect.
	 */
	it('the old moveToTrash shape would have left the tab owning the path', async () => {
		seedTab('t_victim', VICTIM);
		const legacyMoveToTrash = async (_p: string) => {
			await invoke('delete_path', { path: _p }); // …and nothing else. No tab, no model.
		};
		await legacyMoveToTrash(VICTIM);
		expect(get(openTabs).map((t) => t.path)).toContain(VICTIM); // the bug, reproduced
		await moveToTrash(VICTIM); // ours
		expect(get(openTabs).map((t) => t.path)).not.toContain(VICTIM);
	});
});

/**
 * Boss-found 2026-08-02, in the FIRST live test of the fix above: after Overwrite the editor
 * collapsed to "Select a note from the sidebar" while a good tab sat in the bar.
 *
 * Removing tabs from the list is not the whole job. `activeTabId` and `focusedTabId` are
 * SEPARATE stores; leaving either pointing at a removed tab makes the derived active-tab
 * undefined and the pane renders its empty placeholder. `closeTab` has always picked a
 * replacement — this path never did, which is a PRE-EXISTING gap in `deleteWithSetting`
 * (delete the note you are looking at, and the pane could empty with other tabs still open).
 * Sharing the teardown is what made it visible.
 */
describe('the pane must never be left empty with tabs still open', () => {
	it('activates a survivor when the vacated tab was the ACTIVE one (the Boss repro)', async () => {
		seedTab('t_victim', VICTIM);
		seedTab('t_keep', KEEP);
		activeTabId.set('t_victim'); // exactly the Boss's state: the replaced note was active

		await moveToTrash(VICTIM);

		expect(get(activeTabId)).toBe('t_keep');
		expect(get(openTabs).map((t) => t.path)).toEqual([KEEP]);
	});

	it('clears the active tab only when nothing survives', async () => {
		seedTab('t_victim', VICTIM);
		activeTabId.set('t_victim');
		await moveToTrash(VICTIM);
		expect(get(activeTabId)).toBeNull();
		expect(get(openTabs)).toHaveLength(0);
	});

	it('leaves an unaffected active tab exactly where it was', async () => {
		seedTab('t_victim', VICTIM);
		seedTab('t_keep', KEEP);
		activeTabId.set('t_keep');
		await moveToTrash(VICTIM);
		expect(get(activeTabId)).toBe('t_keep'); // untouched — no gratuitous re-activation
	});

	it('repairs the split-view focus the same way', async () => {
		seedTab('t_victim', VICTIM);
		seedTab('t_keep', KEEP);
		focusedTabId.set('t_victim');
		await moveToTrash(VICTIM);
		expect(get(focusedTabId)).toBe('t_keep');
	});

	/** The pre-existing half of this: DELETE had the identical gap. One fix, both paths. */
	it('DELETE also activates a survivor — the gap that was there all along', async () => {
		seedTab('t_victim', VICTIM);
		seedTab('t_keep', KEEP);
		activeTabId.set('t_victim');
		await deleteWithSetting(VICTIM);
		expect(get(activeTabId)).toBe('t_keep');
	});
});

/**
 * Boss-ruled 2026-08-02, after passing the Close tests: *"When closing notes in split view, and
 * there is only one note remaining, the logic is to have it go back to the normal view, where
 * its tab is showing."*
 *
 * The first version collapsed only at ZERO tabs. That fixed the blank-window case and left the
 * one that actually happens: close one of two split notes and a lone pane sits in a split
 * layout whose tab bar is hidden — so the survivor has no tab, no close affordance, and no way
 * to switch. One note is not a split.
 */
describe('split view collapses when fewer than two notes remain', () => {
	it('closing one of two split notes returns to normal view', async () => {
		seedTab('t_a', KEEP);
		seedTab('t_b', VICTIM);
		splitActive.set(true);
		await closeTab('t_b');
		expect(get(splitActive)).toBe(false);
		expect(get(openTabs)).toHaveLength(1);
	});

	it('the surviving note is the focused one, so its tab shows', async () => {
		seedTab('t_a', KEEP);
		seedTab('t_b', VICTIM);
		splitActive.set(true);
		focusedTabId.set('t_b');
		await closeTab('t_b');
		expect(get(focusedTabId)).toBe('t_a');
	});

	it('stays split while two or more remain', async () => {
		seedTab('t_a', KEEP);
		seedTab('t_b', VICTIM);
		seedTab('t_c', 'E:\U\Third.md');
		splitActive.set(true);
		await closeTab('t_c');
		expect(get(splitActive)).toBe(true); // two left — still a split
	});

	it('collapses to the empty state when the last note goes', async () => {
		seedTab('t_a', KEEP);
		splitActive.set(true);
		await closeTab('t_a');
		expect(get(splitActive)).toBe(false);
		expect(get(openTabs)).toHaveLength(0);
	});

	/** The vacate path must obey the same rule — Delete and Overwrite, not just closeTab. */
	it('DELETE in split view collapses it too', async () => {
		seedTab('t_a', KEEP);
		seedTab('t_b', VICTIM);
		splitActive.set(true);
		await deleteWithSetting(VICTIM);
		expect(get(splitActive)).toBe(false);
	});
});

/**
 * Per-build inspection 2026-08-03 — an APP-KILLER in this session's OWN fix, found independently
 * by four hunters.
 *
 * `preserveWorkBeforeVacating` matched only the EXACT vacated path, while
 * `releaseTabsForVacatedPath` matches at-or-under it. Deleting a FOLDER therefore preserved
 * nothing, then disposed every open note inside it and wiped their write-ahead nets — the
 * unsaved work of every open note in that folder, gone, with no error.
 *
 * The single-note case had been fixed and its own generalisation had not: half a sweep, inside
 * the fix for half a sweep. Both halves now take the same predicate from `vacatedBy`.
 */
describe('vacating a FOLDER must treat the notes inside it like the note itself', () => {
	const FOLDER = 'E:\\U\\Folder';
	const INSIDE = 'E:\\U\\Folder\\Inner.md';

	it('releases tabs inside a deleted folder', async () => {
		seedTab('t_in', INSIDE);
		seedTab('t_keep', KEEP);
		await deleteWithSetting(FOLDER);
		expect(get(openTabs).map((t) => t.path)).toEqual([KEEP]);
	});

	it('activates a survivor when the active tab was inside the folder', async () => {
		seedTab('t_in', INSIDE);
		seedTab('t_keep', KEEP);
		activeTabId.set('t_in');
		await deleteWithSetting(FOLDER);
		expect(get(activeTabId)).toBe('t_keep');
	});

	it('the two halves agree on WHICH tabs a folder vacate affects', () => {
		// The defect in one line: release used at-or-under, preserve used exact-match, so the
		// set of tabs destroyed was strictly larger than the set of tabs preserved.
		const vacatedBy = (path: string) => (p: string) =>
			p === path || p.startsWith(path + '/') || p.startsWith(path + '\\');
		const exactOnly = (path: string) => (p: string) => p === path;

		expect(vacatedBy(FOLDER)(INSIDE)).toBe(true); // release destroys it
		expect(exactOnly(FOLDER)(INSIDE)).toBe(false); // preserve skipped it → work lost
		// Sharing one predicate is what makes the two sets identical by construction.
	});

	it('a folder that shares a name prefix is not swept in', async () => {
		seedTab('t_sib', 'E:\\U\\Folder Notes\\Other.md');
		await deleteWithSetting(FOLDER);
		expect(get(openTabs)).toHaveLength(1); // separator boundary holds
	});
});

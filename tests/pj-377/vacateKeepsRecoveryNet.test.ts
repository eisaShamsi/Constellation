/**
 * PJ-377 — **Delete could destroy the only copy of work a failed save left behind.**
 *
 * Found by the 2026-08-24 whole-app safety inspection, ranked APP-KILLER; the fix was then
 * found INCOMPLETE by the adversarial panel, which is why this file tests the net rather than
 * the tab.
 *
 * THE SHAPE. `preserveWorkBeforeVacating` decides what a delete may destroy. Three successive
 * fixes each asked a question about the TAB, while the thing being protected is keyed to the
 * PATH:
 *
 *   1. 2026-08-03 — matched the exact path while the release half matched at-or-under it, so a
 *      FOLDER delete preserved nothing.
 *   2. 2026-08-24 (a) — asked `isNoteDirty` alone, missing the model that is CLEAN yet holds
 *      write-ahead-recovered content: after a failed save the edit lives only in the net, and
 *      the next boot seeds a model from it that is clean by construction.
 *   3. 2026-08-24 (b) — asked `get(openTabs)`, and **the net outlives the tab**. `closeTab`
 *      clears neither the net nor the banner, by design ("preserve a failed write and restore
 *      it on reopen"). So the commonest real sequence — save fails, tab closed, note deleted
 *      later — walked past every guard, because by then there was no tab to ask about.
 *
 * The net is the record: `setNet` stashes before the write, `clearNetIf` clears only on durable
 * success. An entry still present, that is not a `snapshot` view-stash, means "work disk does
 * not have" — regardless of tabs, models, or sessions.
 *
 * WHY THE OBVIOUS FIX WAS WRONG, and why the control case matters. Adding
 * `|| hasUnsavedRecovery(t.id)` to the flush list looks right and is not: `flushIfDirty` returns
 * `{ok:true}` WITHOUT writing when the model is clean (noteSession.ts:392), so it would report
 * durability having written nothing. And a test seeded with a merely DIRTY model would pass
 * against that wrong fix — so these cases seed the states that actually distinguish it.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const dirty = new Set<string>();

vi.mock('$lib/editor/noteSession', async (orig) => ({
	...(await orig<Record<string, unknown>>()),
	isDirty: (id: string) => dirty.has(id),
	flushIfDirty: vi.fn(async () => ({ ok: true })),
}));
vi.mock('$lib/editor/noteModel', async (orig) => ({
	...(await orig<Record<string, unknown>>()),
	close: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import {
	deleteWithSetting,
	moveToTrash,
	openTabs,
	libraries,
	appSettings,
	activeTabId,
	focusedTabId,
	splitActive,
	setWriteAhead,
	getWriteAhead,
	clearWriteAhead,
} from '$lib/libraries/store';

const mockInvoke = vi.mocked(invoke);

const ROOT = 'E:\\U';
const FOLDER = 'E:\\U\\Notes';
const NOTE = 'E:\\U\\Notes\\Recovered.md';
const SIBLING = 'E:\\U\\Notes\\Sibling.md';
const RECOVERED_TEXT = 'the paragraph that never reached disk';

function seedTab(id: string, path: string) {
	openTabs.update((ts) => [
		...ts,
		{ id, path, name: path.split('\\').pop()!, content: RECOVERED_TEXT } as never,
	]);
}

beforeEach(() => {
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
	openTabs.set([]);
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	dirty.clear();
	// The write-ahead buffer is MODULE-level and survives between cases. Leaving it dirty made
	// an earlier draft of the control case fail for the wrong reason: a leftover real-work entry
	// caused `setWriteAhead(..., snapshot: true)` to correctly no-op (a view-stash may never
	// replace real work), so the control was asserting against a net the previous test wrote.
	// Isolate explicitly — a shared-state test that passes by accident proves nothing.
	clearWriteAhead(NOTE);
	clearWriteAhead(SIBLING);
	try { localStorage.removeItem('constellation-wab'); } catch {}
	libraries.set([{ path: ROOT, name: 'universe_notes', is_universe_notes: true } as never]);
	appSettings.update((s) => ({ ...s, trashDestination: 'local' }) as never);
});

describe('PJ-377 — vacating a path must not destroy work that is not on disk', () => {
	it('Deleting a note with NO TAB OPEN keeps its net — the panel-found hole', async () => {
		// The commonest real sequence: the save failed, the user closed the tab (which by design
		// keeps the net and the banner), and only later deleted the note. No tab exists to ask
		// about, which is exactly why the predicate cannot live on `openTabs`.
		setWriteAhead(NOTE, RECOVERED_TEXT, 0, 0);
		expect(getWriteAhead(NOTE)?.content).toBe(RECOVERED_TEXT);

		await deleteWithSetting(NOTE);

		expect(getWriteAhead(NOTE)?.content).toBe(RECOVERED_TEXT);
	});

	it('Deleting the ANCESTOR FOLDER keeps it too — the trigger that never touches the note', async () => {
		setWriteAhead(NOTE, RECOVERED_TEXT, 0, 0);

		await deleteWithSetting(FOLDER);

		expect(getWriteAhead(NOTE)?.content).toBe(RECOVERED_TEXT);
	});

	it('moveToTrash (the displacement primitive) keeps it too', async () => {
		// NOTE: this exercises `moveToTrash` ALONE. The full Overwrite-on-collision flow is
		// moveToTrash + renameItem on one click, which immediately RE-OCCUPIES the vacated path
		// — a case path-keyed preservation cannot make correct, and which is recorded as an open
		// product decision rather than claimed as covered here. An earlier version of this test
		// was titled "Overwrite-on-collision", which asserted coverage this assertion does not
		// have.
		setWriteAhead(NOTE, RECOVERED_TEXT, 0, 0);

		await moveToTrash(NOTE);

		expect(getWriteAhead(NOTE)?.content).toBe(RECOVERED_TEXT);
	});

	it('A CLEAN model whose net holds recovered content keeps it (the restored-session shape)', async () => {
		// `isDirty` is false — as `restoreSessionTabs` leaves a net-seeded model — yet the net
		// holds bytes disk never had.
		seedTab('t1', NOTE);
		dirty.clear();
		setWriteAhead(NOTE, RECOVERED_TEXT, 0, 0);

		await deleteWithSetting(NOTE);

		expect(getWriteAhead(NOTE)?.content).toBe(RECOVERED_TEXT);
	});

	it('SIBLINGS are still cleaned up — protecting one note must not leak the rest', async () => {
		// The panel's second finding: an all-or-nothing skip meant one recovering note under a
		// deleted folder spared every sibling's aux state, growing a map and a localStorage blob
		// whose quota overflow is swallowed by an empty catch.
		setWriteAhead(NOTE, RECOVERED_TEXT, 0, 0);          // real work — must survive
		setWriteAhead(SIBLING, 'already durable', 0, 0, true); // snapshot — must be cleared

		await deleteWithSetting(FOLDER);

		expect(getWriteAhead(NOTE)?.content).toBe(RECOVERED_TEXT);
		expect(getWriteAhead(SIBLING)).toBeUndefined();
	});

	it('CONTROL: an ordinary delete with nothing unsaved still clears its aux state', async () => {
		// Without this the fix could "pass" by never clearing anything — which would leak the net
		// for every ordinary delete and let a future note created at the same path load the
		// deleted note's content (the defect §140 added this cleanup for).
		// `snapshot: true` = content already durable on disk, recovers nothing (PJ-181).
		seedTab('t1', NOTE);
		dirty.clear();
		setWriteAhead(NOTE, 'already durable', 0, 0, true);

		await deleteWithSetting(NOTE);

		expect(getWriteAhead(NOTE)).toBeUndefined();
	});
});

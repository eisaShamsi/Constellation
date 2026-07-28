/**
 * PJ-174 / APP-KILLER #1 — the rename cascade's protection sets are PRE-WALK SNAPSHOTS.
 *
 * `handleRenameComplete` (+layout.svelte) builds all three of its protection sets **before** the
 * cascade's multi-second library walk:
 *
 *   :6756  cascadeFreeze.set(new Set(tabsInLibrary(lib.path).map(t => t.path)))   ← the overlay
 *   :6779  const tabs = tabsInLibrary(lib.path); for (const t of tabs) markCascading(t.path)
 *   :6783  flushAllTabsInLibrary(lib.path)                                        ← its own snapshot
 *
 * …and then, after `await updateLinksOnRename`, calls `reloadTabsFromDisk(result.rewritten)`, which
 * force-reseeds whatever is in `openTabs` **at reload time**.
 *
 * The sidebar tree is NOT blocked during the walk (the overlay covers editor panes only —
 * `FileTree.svelte` has zero cascade/freeze references), so the user can open a note mid-walk and
 * type in it. That tab is in none of the three snapshots, but IS in `openTabs` when the reload runs.
 *
 * `reloadTabsFromDisk`'s own docstring states the invariant it depends on:
 *
 *   > "must NEVER be handed a path whose open model is DIRTY … The guard lives UPSTREAM at every
 *   >  caller … A future edit MUST NOT leak a dirty path into this function."
 *
 * The upstream guard IS the stale snapshot, so the invariant leaks — and the function itself has no
 * dirty check to catch it.
 *
 * FIXED (PJ-174 #1), in the two places the defect actually lives:
 *   1. a LIVE library-scoped cascade mark, because a path snapshot can never cover a tab that does
 *      not exist yet at mark time;
 *   2. `reloadTabsFromDisk` enforcing its OWN documented invariant — a dirty model is never
 *      force-adopted; a genuine conflict routes to the same `.conflict` sidecar + banner the
 *      watcher's external-change path uses. Force-discard is now opt-in by name
 *      (`discardLocalEdits`), which exactly one caller wants: PJ-102c "Discard my changes".
 *
 * These tests were RED before that change: the paragraph came back as the disk text and `isDirty`
 * reported false, so nothing downstream could even tell work had been lost.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';

import {
	openTabs, tabsInLibrary, markCascading, clearCascading, isCascading,
	markCascadingLibrary, clearCascadingLibrary, clearAllCascading, isPathFrozen,
	reloadTabsFromDisk, type OpenTab,
} from '$lib/libraries/store';
import { open as mOpen, editBody, isDirty, bodyForView, closeAll } from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);
const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;
const tab = (id: string, path: string, content: string): OpenTab => ({
	id, path, content,
	libraryName: 'L', libraryPath: '/L', name: path.split('/').pop() ?? path,
	libraryColor: '#000', history: [path], historyIndex: 0,
});
/** Open a tab the way the app does: the store row AND the note model. */
function openTab(id: string, path: string, content: string) {
	openTabs.update((ts) => [...ts, tab(id, path, content)]);
	mOpen(id, path, content);
}

beforeEach(() => {
	closeAll();
	openTabs.set([]);
	clearAllCascading();
	mockInvoke.mockReset();
});
afterEach(() => {
	openTabs.set([]);
	closeAll();
});

describe('PJ-174 #1 — a note opened DURING the rename walk is unprotected', () => {
	it('IS marked cascading via the LIVE library mark, which a path snapshot can never be', () => {
		// The user has note A open and renames something. The orchestrator snapshots NOW.
		openTab('a', '/L/a.md', note('A', 'a body'));
		const snapshot = tabsInLibrary('/L');
		expect(snapshot.map((t) => t.path)).toEqual(['/L/a.md']);
		for (const t of snapshot) markCascading(t.path);

		// …the walk begins (seconds). The sidebar is not blocked, so the user opens B.
		openTab('b', '/L/b.md', note('B', 'links to [[Old]]'));

		// The path snapshot alone still misses B — that is the defect, and it cannot be repaired by
		// snapshotting later, because there is no "later" that is after every tab the user might open.
		expect(isCascading('/L/b.md')).toBe(false);

		// The fix: mark the LIBRARY. `isCascading` now answers "inside a cascading library?", which
		// is true for a tab that did not exist when the mark was taken.
		markCascadingLibrary('/L');
		expect(isCascading('/L/a.md')).toBe(true);
		expect(isCascading('/L/b.md')).toBe(true); // ← B's autosave and teardown flush are GATED

		// Separator boundary: a sibling library sharing a name prefix must NOT be gated.
		openTab('c', '/L2/c.md', note('C', 'other library'));
		expect(isCascading('/L2/c.md')).toBe(false);

		clearCascadingLibrary('/L');
		for (const t of snapshot) clearCascading(t.path);
		expect(isCascading('/L/b.md')).toBe(false); // fully lifted — no permanent save gate
	});

	it('OUTCOME (a): the mid-walk tab\'s unsaved paragraph is destroyed by the reload', async () => {
		openTab('a', '/L/a.md', note('A', 'a body'));
		const snapshot = tabsInLibrary('/L'); // the protection set — B is not in it
		for (const t of snapshot) markCascading(t.path);

		// Mid-walk: the user opens B (a backlinker) and types a paragraph. It lives in the model
		// only — B was not in flushAllTabsInLibrary's snapshot, so it was never flushed, and
		// isCascading(B) is false so nothing gated it either.
		openTab('b', '/L/b.md', note('B', 'links to [[Old]]'));
		editBody('b', 'links to [[Old]]\n\nmy unsaved paragraph');
		expect(isDirty('b')).toBe(true);
		expect(bodyForView('b')).toContain('my unsaved paragraph');

		// The walker rewrote B on disk ([[Old]] → [[New]]) and reports it in result.rewritten.
		// B was never excluded, because the exclude-list comes from the same stale snapshot.
		const rewritten = note('B', 'links to [[New]]');
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_note') return rewritten;
			return undefined;
		});

		const conflicts: string[] = [];
		const refused = await reloadTabsFromDisk(['/L/b.md'], {
			conflict: (path) => { conflicts.push(path); },
		});

		// Before the fix openNoteModel() reset the model wholesale and the paragraph was gone from
		// the model, the screen AND the write-ahead net, with isDirty then reporting false.
		expect(bodyForView('b')).toContain('my unsaved paragraph');
		expect(isDirty('b')).toBe(true);
		// Not silently skipped either — refused, reported, and raised as a real conflict so the
		// cascade's version is preserved in a sidecar instead of being dropped.
		expect(refused).toEqual(['/L/b.md']);
		expect(conflicts).toEqual(['/L/b.md']);

		for (const t of snapshot) clearCascading(t.path);
	});

	it('a CLEAN mid-walk tab must still adopt the cascade result (the fix must not over-block)', async () => {
		openTab('b', '/L/b.md', note('B', 'links to [[Old]]'));
		expect(isDirty('b')).toBe(false);

		const rewritten = note('B', 'links to [[New]]');
		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_note') return rewritten;
			return undefined;
		});
		await reloadTabsFromDisk(['/L/b.md']);

		expect(bodyForView('b')).toContain('[[New]]');
		expect(isDirty('b')).toBe(false);
	});

	it('enforces its OWN stated invariant — a dirty path is never force-adopted', async () => {
		// The docstring says a dirty path must never be handed to it and that the guard lives
		// upstream. Upstream is a snapshot that provably misses the mid-walk tab, so the invariant
		// has to be enforced where it is stated — at the point of damage.
		openTab('b', '/L/b.md', note('B', 'v1'));
		editBody('b', 'v1\nunsaved');
		expect(isDirty('b')).toBe(true);

		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_note') return note('B', 'v2 from disk');
			return undefined;
		});
		await reloadTabsFromDisk(['/L/b.md']);

		expect(isDirty('b')).toBe(true);
		expect(bodyForView('b')).toContain('unsaved');
	});
});

describe('PJ-174 #1 — force-discard stays available, but only by name', () => {
	it('discardLocalEdits: true still force-adopts (PJ-102c "Discard my changes")', async () => {
		openTab('b', '/L/b.md', note('B', 'v1'));
		editBody('b', 'v1\nwork the user chose to throw away');
		expect(isDirty('b')).toBe(true);

		mockInvoke.mockImplementation(async (cmd: string) => {
			if (cmd === 'read_note') return note('B', 'v2 from disk');
			return undefined;
		});
		// The ONE caller that wants this. A blanket dirty-refusal would have silently broken the
		// feature — which is why the destructive behaviour has to be asked for explicitly rather
		// than being the default every caller inherits.
		const refused = await reloadTabsFromDisk(['/L/b.md'], { discardLocalEdits: true });

		expect(refused).toEqual([]);
		expect(bodyForView('b')).toContain('v2 from disk');
		expect(bodyForView('b')).not.toContain('work the user chose to throw away');
		expect(isDirty('b')).toBe(false);
	});
});

describe('PJ-174 #1 — the read-only FREEZE covers the mid-walk tab too', () => {
	it('freezes any path inside the cascading library, including one opened after the mark', () => {
		// The overlay set now holds library ROOTS. Passed raw — `isPathFrozen` normalises both
		// sides, which is what stops each consumer hand-rolling the boundary rule.
		const frozen = new Set(['E:\\Lib\\History\\']);
		expect(isPathFrozen('E:\\Lib\\History\\Ancient\\Africa.md', frozen)).toBe(true);
		expect(isPathFrozen('E:/Lib/History/opened-mid-walk.md', frozen)).toBe(true);
		// Separator boundary — a sibling library sharing a name prefix is NOT frozen.
		expect(isPathFrozen('E:/Lib/History2/other.md', frozen)).toBe(false);
		expect(isPathFrozen('E:/Other/x.md', frozen)).toBe(false);
	});

	it('an empty set freezes nothing (the steady state, and the lift-the-freeze path)', () => {
		expect(isPathFrozen('E:/Lib/History/a.md', new Set())).toBe(false);
		expect(isPathFrozen('', new Set(['E:/Lib']))).toBe(false);
	});
});

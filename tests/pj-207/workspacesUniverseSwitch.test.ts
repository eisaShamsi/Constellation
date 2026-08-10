/**
 * PJ-207 §15 whole-app sweep — APP-KILLER: **a universe switch carries the OUTGOING universe's
 * named workspaces into the incoming one, and the first save writes them over its file.**
 *
 * The mechanism, verified in source before this test was written:
 *
 *   1. `loadWorkspaces` adopts the read only `if (data.length > 0)` — directly contradicting its
 *      own comment one line above ("An empty array from a SUCCESSFUL read is a real answer —
 *      adopt it"). A universe with no workspaces therefore leaves the PREVIOUS universe's list in
 *      the store…
 *   2. …while still setting `workspacesLoaded = true`, which is the ONLY thing gating writes. So
 *      saving is unlocked with the wrong universe's data loaded.
 *   3. Nothing resets the store or the latch on a universe switch — `handleUniverseSwitch` clears
 *      libraries, stats, sky, tags, appearances and the drift notice, but not these.
 *
 * `workspaces.json` has no `.prev` rotation and no second copy — its own comment calls it "the
 * precious file" — so the overwrite is unrecoverable.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';
import {
	workspaces,
	loadWorkspaces,
	markWorkspacesLoadedFromBundle,
	resetWorkspacesForUniverse,
} from '../../src/lib/libraries/store';

const A_LIST = [{ name: 'Universe A layout', tabs: ['/A/one.md'] }] as never[];

describe('PJ-207 §15 — workspaces must not survive a universe switch', () => {
	beforeEach(() => {
		vi.mocked(invoke).mockReset();
		resetWorkspacesForUniverse();
	});

	it("a SUCCESSFUL empty read adopts the empty answer — it does not keep the old universe's list", async () => {
		// Universe A: one saved workspace, latched.
		markWorkspacesLoadedFromBundle(A_LIST);
		expect(get(workspaces)).toHaveLength(1);

		// Switch to universe B, whose workspaces.json is an empty array (a freshly created
		// universe writes exactly "[]"). This is a SUCCESSFUL read of "you have none".
		vi.mocked(invoke).mockResolvedValueOnce([]);
		await loadWorkspaces();

		expect(get(workspaces)).toEqual([]); // B's real answer, not A's list
	});

	it('a switch clears both the list and the write latch, so a failed read cannot write A over B', async () => {
		markWorkspacesLoadedFromBundle(A_LIST);

		// The switch itself must clear, BEFORE any read of B is attempted.
		resetWorkspacesForUniverse();
		expect(get(workspaces)).toEqual([]);

		// Now B's read FAILS (an AV/sync lock). Writing must stay disabled: the latch was
		// cleared by the switch and only a successful read re-arms it.
		vi.mocked(invoke).mockRejectedValueOnce(new Error('locked'));
		await loadWorkspaces();
		expect(get(workspaces)).toEqual([]); // still empty — nothing of A's is present to write
	});

	it('a real non-empty read still loads normally', async () => {
		vi.mocked(invoke).mockResolvedValueOnce([{ name: 'B layout', tabs: ['/B/x.md'] }]);
		await loadWorkspaces();
		expect(get(workspaces)).toHaveLength(1);
		expect(get(workspaces)[0].name).toBe('B layout');
	});
});

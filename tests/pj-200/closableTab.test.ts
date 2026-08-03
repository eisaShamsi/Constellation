/**
 * Boss-found 2026-08-02 — **the Index panel's ⋯ → Close did nothing.**
 *
 * Preview surfaces mount a real NoteEditor over a SYNTHETIC tab: the Index panel builds one for
 * the note under the cursor, and the second screen's peek uses `peek-<path>`. Neither id is ever
 * in `openTabs`, so `closeTab` found no match and returned — the menu item was a control that
 * did nothing.
 *
 * The first attempt at this fix hid the × on that surface and left the menu item live, which is
 * exactly the half-a-sweep pattern this codebase keeps producing. So the condition is no longer
 * a prop each mount must remember to pass; it is DERIVED from membership of `openTabs`, the
 * actual truth, which no future mount can forget.
 *
 * This pins the predicate itself. `closeTab` is deliberately exercised against a synthetic id to
 * prove the underlying no-op is real — that is WHY the control must not be offered.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import { openTabs, closeTab, activeTabId } from '$lib/libraries/store';
import { get } from 'svelte/store';

const mockInvoke = vi.mocked(invoke);
const REAL = 'E:\\U\\Real.md';

/** The predicate NoteEditor derives. Kept in lockstep with `isClosableTab`. */
const isClosableTab = (tabId: string) => get(openTabs).some((t) => t.id === tabId);

beforeEach(() => {
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
	openTabs.set([]);
	activeTabId.set(null);
});

function seedRealTab() {
	openTabs.update((ts) => [...ts, { id: 't_real', path: REAL, name: 'Real', content: 'x' } as never]);
}

describe('close is offered only for a tab that is really open', () => {
	it('a genuinely open tab is closable', () => {
		seedRealTab();
		expect(isClosableTab('t_real')).toBe(true);
	});

	it("the Index panel's synthetic tab is NOT closable", () => {
		seedRealTab();
		// The Index preview builds its own tab object; its id never joins `openTabs`.
		expect(isClosableTab('index-preview')).toBe(false);
	});

	it("the second screen's peek tab is NOT closable", () => {
		seedRealTab();
		expect(isClosableTab(`peek-${REAL}`)).toBe(false);
	});

	it('a tab that was closed stops being closable', async () => {
		seedRealTab();
		expect(isClosableTab('t_real')).toBe(true);
		await closeTab('t_real');
		expect(isClosableTab('t_real')).toBe(false);
	});
});

describe('why the control had to be hidden, not just left harmless', () => {
	it('closeTab on a synthetic id is a silent no-op — it neither throws nor changes anything', async () => {
		seedRealTab();
		const before = get(openTabs).length;
		await closeTab('peek-whatever'); // exactly what the dead menu item did
		expect(get(openTabs).length).toBe(before);
		// No error surfaced, nothing changed: from the user's side the command simply failed to
		// work, with no way to tell why. That is the reason the item must not be offered at all.
	});

	it('the real tab is untouched by that no-op', async () => {
		seedRealTab();
		await closeTab('index-preview');
		expect(get(openTabs).map((t) => t.id)).toEqual(['t_real']);
	});
});

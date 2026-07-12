/**
 * PJ-092 (APP-KILLER) — `reloadTabsFromDisk` must NEVER force-adopt disk over a
 * DIRTY model.
 *
 * The rename wikilink cascade flushes every open tab, then the Rust walker
 * rewrites each backlink-source note's ON-DISK `[[Old]]`→`[[New]]`, then
 * `reloadTabsFromDisk` re-reads + force-reseeds the model from that disk. If a
 * tab's pre-cascade flush FAILED (a locked .md — Syncthing/OneDrive/Defender),
 * its model is still dirty and disk is STALE (missing the unsaved edits) yet now
 * differs from `tab.content` (the walker rewrote the link) — so the unguarded
 * reseed rebuilt the model CLEAN from stale disk and wiped the recovery net,
 * silently and permanently losing the user's unsaved edits while the save-health
 * banner self-healed to green. The sibling `renameItem` path was hardened
 * (`renameFlushOk`); this generalizes that guard to the shared reload primitive,
 * so EVERY flush-then-reload caller is covered at once — the rename cascade,
 * updateNoteProperty, resolveStructuralConflict, toggleTaskReconciled,
 * addLinkToNote, and any future one (resolveConflictMerge stays clean — it gates
 * on the durable save's outcome.ok before reloading).
 *
 * Reproduce-First: the first `it` is RED before the dirty-guard, GREEN after.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { invoke } from '@tauri-apps/api/core';

import {
	openTabs,
	reloadTabsFromDisk,
	setWriteAhead,
	getWriteAhead,
	type OpenTab,
} from '$lib/libraries/store';
import { open as mOpen, editBody, isDirty, bodyForView, closeAll } from '$lib/editor/noteSession';

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
const reloadVersionOf = (path: string) => get(openTabs).find((t) => t.path === path)?.reloadVersion ?? 0;
const contentOf = (path: string) => get(openTabs).find((t) => t.path === path)?.content ?? '';

/** Wire read_note to a fake disk map; all other IPCs no-op. */
function wireDisk(disk: Map<string, string>) {
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args: any) => {
		if (cmd === 'read_note') return disk.get(args.filePath) ?? '';
		return undefined;
	});
}

beforeEach(() => { closeAll(); openTabs.set([]); });
afterEach(() => { openTabs.set([]); });

describe('reloadTabsFromDisk — dirty-guard (PJ-092 APP-KILLER)', () => {
	it('PRESERVES a dirty model whose pre-cascade flush failed (never reseeds stale disk)', async () => {
		// Tab B links [[Old]] and is dirty with unsaved work; its flush FAILED so
		// disk never got the edit — but the cascade walker rewrote its stale disk
		// link to [[New]], so disk now DIFFERS from tab.content (the trigger).
		openOne('b', '/b.md', note('B', 'v1 [[Old]]'));
		editBody('b', 'v1 [[Old]] my-unsaved-work');
		expect(isDirty('b')).toBe(true);
		setWriteAhead('/b.md', note('B', 'v1 [[Old]] my-unsaved-work'), 0, 0);

		wireDisk(new Map([['/b.md', note('B', 'v1 [[New]]')]]));
		await reloadTabsFromDisk(['/b.md']);

		// The unsaved edit survives — NOT clobbered by the stale-disk reseed.
		expect(bodyForView('b')).toBe('v1 [[Old]] my-unsaved-work');
		expect(reloadVersionOf('/b.md')).toBe(0);          // no {#key} remount
		expect(getWriteAhead('/b.md')).not.toBeNull();     // recovery net kept — sole copy
	});

	it('STILL reseeds a CLEAN tab from disk (the guard is targeted, not a blanket no-op)', async () => {
		// Clean tab whose flush succeeded: disk is canonical; the reseed must fire.
		openOne('c', '/c.md', note('C', 'v1 [[Old]]'));
		expect(isDirty('c')).toBe(false);

		wireDisk(new Map([['/c.md', note('C', 'v1 [[New]]')]]));
		await reloadTabsFromDisk(['/c.md']);

		expect(bodyForView('c')).toBe('v1 [[New]]');        // model adopted the cascade result
		expect(reloadVersionOf('/c.md')).toBe(1);           // remounted
		expect(contentOf('/c.md')).toBe(note('C', 'v1 [[New]]'));
	});
});

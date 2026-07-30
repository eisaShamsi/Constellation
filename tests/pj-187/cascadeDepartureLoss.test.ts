/**
 * PJ-187 — safety-inspection APP-KILLER, found IN THE SWEEP'S OWN FIX before commit.
 *
 * ── THE DEFECT ───────────────────────────────────────────────────────────────
 * The sweep added a cascade gate to `flushOutgoing` (a note whose links a rename cascade
 * is rewriting on disk must not be flushed from memory — writing the pre-cascade body
 * back would revert the walker's corrections). Correct refusal, WRONG RETURN VALUE: the
 * first version returned `{ok:true}`, and the FlushResult contract (noteSession.ts:266)
 * defines `ok:true` as "safe to proceed with the nav/replace". Every departure site then
 * destroyed the dirty model — openNoteTab/loadTabHistoryEntry re-seeded it, closeTab and
 * the universe-switch sweep disposed it — and the unsaved edit existed NOWHERE:
 *
 *   · not on disk        — the flush was refused;
 *   · not in the net     — NoteEditor's own stash sites sit BELOW its cascade gate
 *                          (NoteEditor.svelte:344 returns before :391/:419), and
 *                          PropertyEditor's teardown flush is gated the same way;
 *   · no banner          — nothing failed, so saveHealth never heard of it.
 *
 * Reachable by an ordinary gesture: rename a heavily-linked note (the cascade walk runs
 * for seconds on a large library), edit a property of another note in the right sidebar
 * (which the freeze overlay deliberately does not cover), then click a third note or
 * close the tab while the walk is still running.
 *
 * ── THE FIX, and what this file pins ─────────────────────────────────────────
 * At the single choke point every departure routes through:
 *   1. the refusal is `{ok:false, reason:'cascading'}` — the nav sites abort and keep
 *      the user on the note until the cascade lifts (their documented failed-flush
 *      behaviour);
 *   2. the model's current content is stashed into the write-ahead net FIRST, unflagged
 *      (real unsaved work — the PJ-181 stale-snapshot check never discards it), because
 *      closeTab and the departure sweep proceed regardless of the result by contract,
 *      and the net is the only thing that preserves their content for reopen.
 *
 * This is LL-040's shape a fourth time: the gate tested the right condition and returned
 * the wrong FACT about it. The inspection caught it because the return value contradicted
 * the contract one file away.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	openNoteTab,
	closeTab,
	openTabs,
	activeTabId,
	focusedTabId,
	splitActive,
	getWriteAhead,
	clearWriteAhead,
	flushDisposeClearTabs,
	markCascading,
	clearCascading,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let calls: string[];

const PATH = '/lib/edited-during-cascade.md';
const OTHER = '/lib/other.md';
const NOTE = `---\ntitle: T\ncid_cn: N1\n---\noriginal body`;

function wireInvoke() {
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		calls.push(cmd);
		switch (cmd) {
			case 'read_note': {
				const c = disk.get(args.filePath);
				if (c === undefined) throw new Error('missing');
				return c;
			}
			case 'write_note': {
				disk.set(args.filePath, args.content);
				return undefined;
			}
			case 'ensure_cid_cn_cmd':
				return disk.get(args.filePath) ?? '';
			default:
				return undefined;
		}
	});
}

const tabs = () => {
	let v: any[] = [];
	openTabs.subscribe((x) => (v = x))();
	return v;
};

beforeEach(async () => {
	clearCascading(PATH); // a failed case must not leak its gate into the next
	await flushDisposeClearTabs('test-reset');
	clearWriteAhead(PATH);
	clearWriteAhead(OTHER);
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	S.closeAll();
	disk = new Map([[PATH, NOTE], [OTHER, `---\ntitle: O\ncid_cn: N2\n---\nother`]]);
	calls = [];
	wireInvoke();
});

describe('PJ-187 — an edit made during a cascade must survive every departure', () => {
	it('closeTab during a cascade: the edit reaches the write-ahead net, unflagged', async () => {
		// 1. Open the note and edit it (the sidebar PropertyEditor path pushes into the
		//    model exactly like this — dirty model, no disk write yet).
		await openNoteTab(PATH, 'lib', '#123456');
		const tab = tabs()[0];
		S.editBody(tab.id, `${NOTE} PLUS THE EDIT`, PATH);
		expect(S.isDirty(tab.id)).toBe(true);

		// 2. A rename cascade is rewriting this library — the gate is up.
		markCascading(PATH);
		try {
			// 3. The user closes the tab while the walk runs. closeTab proceeds regardless
			//    of the flush result (by contract: the tab is being dismissed).
			await closeTab(tab.id);
		} finally {
			clearCascading(PATH);
		}

		// The model is gone — that is closeTab's job. The edit must NOT be gone with it.
		expect(disk.get(PATH)).toBe(NOTE); // cascade protected: nothing wrote over disk
		const net = getWriteAhead(PATH);
		expect(net, 'the unsaved edit must be in the write-ahead net').toBeTruthy();
		expect(net!.content).toContain('PLUS THE EDIT');
		// Unflagged = real unsaved work: the PJ-181 stale-snapshot check must never
		// discard it on reopen, even though disk (rewritten by the cascade) differs.
		expect(net!.snapshot).not.toBe(true);
	});

	it('nav during a cascade: the departure ABORTS — the user stays on the dirty note', async () => {
		await openNoteTab(PATH, 'lib', '#123456');
		const tab = tabs()[0];
		S.editBody(tab.id, `${NOTE} PLUS THE EDIT`, PATH);

		markCascading(PATH);
		try {
			// Click another note mid-walk. The flush refuses (cascading) → the nav aborts.
			await openNoteTab(OTHER, 'lib', '#123456');
		} finally {
			clearCascading(PATH);
		}

		// Same tab, same path, model intact and still dirty — nothing was destroyed.
		expect(tabs()[0].path).toBe(PATH);
		expect(S.isDirty(tab.id)).toBe(true);
		expect(S.bodyForView(tab.id)).toContain('PLUS THE EDIT');
		expect(disk.get(PATH)).toBe(NOTE); // and nothing wrote over the cascade
	});

	it("an ABORTED nav puts the INCOMING note's consumed recovery net back", async () => {
		// OTHER carries a write-ahead entry — the only copy of a failed save's work.
		const netContent = `---
title: O
cid_cn: N2
---
other PLUS UNSAVED RECOVERY`;
		const { setWriteAhead } = await import('$lib/libraries/store');
		setWriteAhead(OTHER, netContent, 0, 0);

		// The current tab is dirty and its library is mid-cascade, so the departure flush
		// will refuse — the nav aborts AFTER resolveNoteContent has consumed OTHER's net.
		await openNoteTab(PATH, 'lib', '#123456');
		const tab = tabs()[0];
		S.editBody(tab.id, `${NOTE} PLUS THE EDIT`, PATH);
		markCascading(PATH);
		try {
			await openNoteTab(OTHER, 'lib', '#123456');
		} finally {
			clearCascading(PATH);
		}

		expect(tabs()[0].path).toBe(PATH); // the nav aborted
		// Before the fix, OTHER's net entry was consumed by the read and never restored —
		// the recovery existed nowhere. It must be back, byte for byte.
		const net = getWriteAhead(OTHER);
		expect(net, 'the consumed net entry must be re-stashed on abort').toBeTruthy();
		expect(net!.content).toBe(netContent);
		expect(net!.snapshot).not.toBe(true); // still real unsaved work
	});

	it('CONTROL — outside a cascade, a departure flushes to disk exactly as before', async () => {
		await openNoteTab(PATH, 'lib', '#123456');
		const tab = tabs()[0];
		S.editBody(tab.id, `${NOTE} PLUS THE EDIT`, PATH);

		await openNoteTab(OTHER, 'lib', '#123456');

		expect(disk.get(PATH)).toContain('PLUS THE EDIT'); // the normal durable flush
		expect(tabs()[0].path).toBe(OTHER); // and the nav proceeded
	});
});

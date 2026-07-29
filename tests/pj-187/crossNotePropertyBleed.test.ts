/**
 * PJ-187 (headline of the 2026-07-29 whole-app register) — CROSS-NOTE PROPERTY BLEED.
 * APP-KILLER: note A's properties written onto note B's `.md`, durably and silently.
 *
 * ── THE RECIPE ───────────────────────────────────────────────────────────────
 *   1. Note A is open, with the RIGHT SIDEBAR on the Properties tab.
 *   2. Edit a property there. `debouncedSave()` arms an 800 ms timer; the edit lives ONLY
 *      in the panel's local `editableProps` — it has not reached the model, so the model
 *      is CLEAN.
 *   3. Within 800 ms, click a `[[wikilink]]` (or a file-tree row, or Alt+Left). That is an
 *      IN-PLACE navigation: `openNoteTab` reuses the SAME tab — same `id`, new `path` —
 *      and re-points the model with `openNoteModel(currentTab.id, filePath, content)`.
 *   4. The timer fires.
 *
 * ── WHY EVERY EXISTING GUARD MISSES IT ───────────────────────────────────────
 *  · The nav-flush at `store.ts:2739` is gated on `isNoteDirty(currentTab.id)` — it flushes
 *    the MODEL. A pending property debounce never reached the model, so the model is clean
 *    and the flush is skipped.
 *  · The panel's seed `$effect` re-seeds on `tabChanged || propsChanged`, but `tabChanged`
 *    is `tabId !== prevTabId` and the tab id DID NOT CHANGE — only the path did. The
 *    `localEditPending` guard (a pending timer) then blocks the propsChanged re-seed too.
 *    So `editableProps` still hold NOTE A's rows.
 *  · The timer reads the LIVE `tabId` / `filePath` props — now note B's.
 *  · Every `expectPath` guard passes: they check that the id and the path AGREE, which they
 *    do. Nothing anywhere checks that the ROWS belong to that note.
 *  · The NotePane-embedded twin of this panel is protected — `NoteEditor.svelte` wraps it in
 *    `{#key tab.id + '|' + tab.path + '|' + reloadVersion}`, so an in-place nav DESTROYS it
 *    and its onDestroy identity gate applies. The right-sidebar instance
 *    (`+layout.svelte`, under a bare `{#if rightSidebarTab === 'properties' && sidebarTab}`)
 *    has no `{#key}` — the only one in that file is FocusPane's.
 *
 * ── WHAT THIS TEST DRIVES, AND WHAT IT REPLICATES ────────────────────────────
 * It drives the REAL `openNoteTab` in-place reuse, the REAL `plan`/`apply` intents, and the
 * REAL `saveTabContent` against a mocked IPC bridge. What it REPLICATES — deliberately and
 * in three lines — is the component's retained state (`editableProps` / `seededRows` still
 * holding note A's rows), because this repo has no component-test harness (no jsdom, no
 * testing-library, no component tests anywhere) and adding one is not this fix's job.
 *
 * That is stated plainly because LL-036 warns that a fixture more privileged than production
 * cannot fail the way production fails. Here the replication is of the DEFECT's precondition,
 * not of a guard — the commit chain under test is entirely real.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	openNoteTab,
	openTabs,
	activeTabId,
	focusedTabId,
	splitActive,
	splitDirection,
	saveTabContent,
	parseFrontmatter,
	flushDisposeClearTabs,
	clearWriteAhead,
	type FrontmatterProperty,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';
import * as M from '$lib/editor/noteModel';
import { plan, apply, touchedSince, seededKeysOf, rowsBelongToTarget } from '$lib/editor/propsCommit';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let calls: Array<{ cmd: string; args: any }>;

const A = '/lib/note-a.md';
const B = '/lib/note-b.md';

const NOTE_A = '---\ncid_cn: AAAA\ntitle: Note A\nstage: spark-seed\nsecret_a: only-on-A\n---\n\nbody of A\n';
const NOTE_B = '---\ncid_cn: BBBB\ntitle: Note B\nstage: growth-seed\n---\n\nbody of B\n';

beforeEach(async () => {
	await flushDisposeClearTabs('test-reset');
	clearWriteAhead(A);
	clearWriteAhead(B);
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	disk = new Map([[A, NOTE_A], [B, NOTE_B]]);
	calls = [];
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		calls.push({ cmd, args });
		if (cmd === 'read_note') {
			const c = disk.get(args.filePath);
			if (c === undefined) throw new Error('missing');
			return c;
		}
		if (cmd === 'write_note') { disk.set(args.filePath, args.content); return undefined; }
		if (cmd === 'ensure_cid_cn_cmd') return disk.get(args.filePath) ?? '';
		return undefined;
	});
});

const tabs = () => {
	let v: any[] = [];
	openTabs.subscribe((x) => (v = x))();
	return v;
};

/** Verbatim `PropertyEditor.commitAndSave`, minus the component wrapper. */
async function commitAndSave(
	id: string,
	path: string,
	editableProps: FrontmatterProperty[],
	seededRows: FrontmatterProperty[],
	body: string,
	seededForPath: string,
) {
	// The REAL guard, called here exactly as the component calls it — not a copy of it.
	// Replicating the guard would make this test prove nothing (LL-040).
	if (!rowsBelongToTarget(seededForPath, path)) return;
	const model = M.getModel(id);
	if (!model) return;
	const touched = touchedSince(seededRows, editableProps);
	const ops = plan(editableProps, model.props, seededKeysOf(seededRows), touched);
	apply(ops, {
		setValue: (k, v, o) => S.editPropValue(id, k, v, o, path),
		add: (pr) => S.addPropTo(id, pr, path),
		remove: (k) => S.removePropFrom(id, k, path),
		order: () => false,
	});
	await saveTabContent(id, path, editableProps, body, false, true);
}

describe('PJ-187 — a pending property commit must never land on a different note', () => {
	it("RECIPE — note A's properties must not reach note B's file", async () => {
		// 1. Open A. The sidebar panel seeds from A's model.
		await openNoteTab(A, 'Lib');
		const tab = tabs().find((t) => t.path === A)!;
		const tabIdAtSeed = tab.id;

		const seededRows = M.getModel(tab.id)!.props.map((p) => ({ ...p }));
		const bodyAtSeed = parseFrontmatter(NOTE_A).body;

		// 2. The user edits `stage` in the sidebar. The timer is armed; the model is CLEAN
		//    because nothing has committed yet.
		const editableProps = seededRows.map((p) =>
			p.key === 'stage' ? { ...p, value: 'EDITED-ON-A' } : { ...p },
		);
		expect(S.isDirty(tab.id)).toBe(false); // ← why the nav-flush guard is skipped

		// 3. Within 800 ms the user clicks a wikilink → IN-PLACE nav, same tab id.
		await openNoteTab(B, 'Lib');
		const after = tabs().find((t) => t.id === tabIdAtSeed)!;
		expect(after.path).toBe(B); // the tab was reused, not replaced
		expect(M.getModel(tabIdAtSeed)!.path).toBe(B); // the model now IS note B

		// 4. The timer fires with the LIVE props (note B) and note A's retained rows.
		await commitAndSave(tabIdAtSeed, B, editableProps, seededRows, bodyAtSeed, /* seededForPath */ A);

		// ── THE ASSERTIONS ───────────────────────────────────────────────────────────
		const bOnDisk = disk.get(B)!;
		const bProps = parseFrontmatter(bOnDisk).properties;
		const byKey = (k: string) => bProps.find((p) => p.key === k)?.value;

		// A's identity must never land on B.
		expect(byKey('cid_cn')).toBe('BBBB');
		expect(byKey('title')).toBe('Note B');
		// A-only keys must not be ADDED to B.
		expect(bProps.some((p) => p.key === 'secret_a')).toBe(false);
		// A's edited value must not overwrite B's own.
		expect(byKey('stage')).toBe('growth-seed');
		// And A's BODY must not have replaced B's.
		expect(bOnDisk).toContain('body of B');
		expect(bOnDisk).not.toContain('body of A');
	});

	/** CONTROL — the ordinary case must keep working: no navigation, the edit lands on A. */
	it('CONTROL — with no navigation, the edit lands on the note it was made in', async () => {
		await openNoteTab(A, 'Lib');
		const tab = tabs().find((t) => t.path === A)!;
		const seededRows = M.getModel(tab.id)!.props.map((p) => ({ ...p }));
		const body = parseFrontmatter(NOTE_A).body;
		const editableProps = seededRows.map((p) =>
			p.key === 'stage' ? { ...p, value: 'EDITED-ON-A' } : { ...p },
		);

		await commitAndSave(tab.id, A, editableProps, seededRows, body, /* seededForPath */ A);

		const aProps = parseFrontmatter(disk.get(A)!).properties;
		expect(aProps.find((p) => p.key === 'stage')?.value).toBe('EDITED-ON-A');
		expect(aProps.find((p) => p.key === 'secret_a')?.value).toBe('only-on-A');
		expect(disk.get(B)).toBe(NOTE_B); // B untouched
	});
});

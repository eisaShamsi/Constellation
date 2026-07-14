/**
 * Recipe S (PJ-102, 2026-07-14) — THE MANUAL-REOPEN RECOVERY CLOBBER (APP-KILLER).
 *
 * The app's documented last-resort recovery: a failed durable save (locked .md) or a
 * crash mid-debounce leaves the user's edits ONLY in the write-ahead net (memory +
 * localStorage); closeTab proceeds past a failed close-flush citing exactly this net
 * ("…preserve a failed write and restore it on reopen"). The reopen path then:
 *   1. resolveNoteContent finds the identity-proven net, CONSUMES it (manual-open
 *      semantics — clearWriteAhead), and returns the recovered content;
 *   2. openNoteTab's ensure_cid_cn step (store.ts:2039) calls the Rust cmd, which
 *      reads DISK and — when cid_cn already exists — returns the DISK content
 *      verbatim (canonical.rs:1226);
 *   3. `updated !== content` is true precisely BECAUSE the recovery differed from
 *      stale disk → `content = updated` discards the recovered edits.
 * Screen, net, and disk all end on the stale version — silent, 100% reproducible
 * whenever net ≠ disk on a manual open. This recipe drives the REAL openNoteTab
 * against a mocked IPC bridge (the restore.test.ts harness pattern; the
 * ensure_cid_cn_cmd stub mirrors the real Rust: cid present → disk verbatim).
 *
 * THE FIX under test: openNoteTab adopts ensure_cid_cn's result ONLY when the
 * in-hand content LACKS a cid_cn (an identity-proven recovery ALWAYS carries one —
 * that is what identity-proven means). The cmd's only content-changing jobs
 * (inject a missing cid / migrate legacy `cid:`) both operate on cid_cn-less
 * content, so the legitimate flows are untouched (Recipe S2).
 *
 * The sibling call site drainCidEnsure (store.ts:2281) was CHECKED and REFUTED as a
 * second instance: it flushes a dirty model before the ensure, adopts only while
 * clean, and a pending-cid tab (cid-less disk) can never hold net-recovered content
 * (a cid-less disk fails resolveNoteContent's identity proof by construction).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	openNoteTab,
	adoptExternalChangeIntoTabs,
	restoreSessionTabs,
	saveRecoveredCopy,
	discardFailedSave,
	reportSaveFailure,
	saveHealth,
	openTabs,
	activeTabId,
	focusedTabId,
	splitActive,
	splitDirection,
	setWriteAhead,
	getWriteAhead,
	flushDisposeClearTabs,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let locked: Set<string>;
let calls: Array<{ cmd: string; args: any }>;

const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;
const cidlessNote = (body: string) => `---\ntitle: T\n---\n${body}`;

function wireInvoke() {
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		calls.push({ cmd, args });
		switch (cmd) {
			case 'read_note': {
				const c = disk.get(args.filePath);
				if (c === undefined) throw new Error('missing');
				return c;
			}
			case 'write_note': {
				if (locked.has(args.filePath)) throw new Error('EBUSY: locked');
				disk.set(args.filePath, args.content);
				return undefined;
			}
			case 'ensure_cid_cn_cmd': {
				// Mirrors the REAL Rust (canonical.rs): reads DISK; cid present → disk
				// verbatim; absent → inject + write back.
				const c = disk.get(args.filePath) ?? '';
				if (!/cid_cn:/.test(c)) {
					const updated = c.startsWith('---')
						? c.replace(/^---\n/, '---\ncid_cn: INJECTED\n')
						: `---\ncid_cn: INJECTED\n---\n${c}`;
					disk.set(args.filePath, updated);
					return updated;
				}
				return c;
			}
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
	await flushDisposeClearTabs('test-reset');
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	disk = new Map();
	locked = new Set();
	calls = [];
	wireInvoke();
});

describe('Recipe S — manual reopen must deliver the recovery copy intact (PJ-102)', () => {
	it('S1: net-recovered content survives the reopen (ensure_cid_cn must not swap in stale disk)', async () => {
		const PATH = '/lib/n.md';
		// Disk holds the STALE last-saved version; the write-ahead net holds the
		// user's unsaved edits (the failed-save / crash recovery copy). Same cid —
		// identity-proven, so resolveNoteContent restores the net.
		disk.set(PATH, note('N', 'stale disk body'));
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);

		await openNoteTab(PATH, 'Lib');

		const t = tabs().find((x) => x.path === PATH);
		expect(t).toBeTruthy();
		// THE APP-KILLER assertion — the screen shows the RECOVERED content:
		expect(t!.content).toContain('RECOVERED unsaved edits');
		// …and the single-ownership model (the save source) holds it too:
		expect(S.bodyForView(t!.id)).toContain('RECOVERED unsaved edits');
	});

	it('S2: the legitimate cid-injection flow is untouched (a cid-less note adopts the injected version)', async () => {
		const PATH = '/lib/fresh.md';
		disk.set(PATH, cidlessNote('fresh body'));

		await openNoteTab(PATH, 'Lib');

		const t = tabs().find((x) => x.path === PATH);
		expect(t).toBeTruthy();
		// The injected cid_cn IS adopted (the step's whole purpose):
		expect(t!.content).toContain('cid_cn: INJECTED');
		expect(t!.content).toContain('fresh body');
	});

	it('S4 (the Boss-hit sequence): after a recovery-reopen, a phantom watcher event must NOT clobber the recovered content back to stale disk', async () => {
		const PATH = '/lib/n.md';
		disk.set(PATH, note('N', 'stale disk body'));
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);
		await openNoteTab(PATH, 'Lib');
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('RECOVERED unsaved edits'); // the S1 fix holds

		// The live wound: disk is UNCHANGED (my lock blocked every write; the event is a
		// phantom — AV/indexer/our own suppressed-echo leak). The watcher flush fires:
		await adoptExternalChangeIntoTabs([PATH], undefined, async (fp) => disk.get(fp)!);

		const t2 = tabs().find((x) => x.path === PATH)!;
		// THE assertion — the recovered content survives (pre-fix: swapped to stale disk):
		expect(t2.content).toContain('RECOVERED unsaved edits');
		expect(S.bodyForView(t2.id)).toContain('RECOVERED unsaved edits');
		// And the recovered delta is honestly DIRTY — the autosave/retry will persist it,
		// and the save-health banner stays red until a REAL durable save:
		expect(S.isDirty(t2.id)).toBe(true);
	});

	it('S5: a GENUINE external edit after a recovery-reopen goes to the dirty-conflict hook, never a silent clobber', async () => {
		const PATH = '/lib/n.md';
		disk.set(PATH, note('N', 'stale disk body'));
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);
		await openNoteTab(PATH, 'Lib');

		// A REAL external edit lands (disk actually changes):
		disk.set(PATH, note('N', 'stale disk body\nEXTERNAL edit'));
		const conflicts: string[] = [];
		await adoptExternalChangeIntoTabs(
			[PATH],
			{ conflict: (p) => { conflicts.push(p); } },
			async (fp) => disk.get(fp)!,
		);
		const t = tabs().find((x) => x.path === PATH)!;
		// The recovered local work is never silently clobbered — the conflict hook fires
		// (the PJ-070 sidecar path) and the view keeps the local copy:
		expect(t.content).toContain('RECOVERED unsaved edits');
		expect(conflicts).toEqual([PATH]);
	});

	it('S6 (the Q4 hole): a SESSION-RESTORED wab-recovered tab survives a phantom event — content AND net intact', async () => {
		const PATH = '/lib/n.md';
		disk.set(PATH, note('N', 'stale disk body'));
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);

		// The MIG-100 session restore (preserveNet: the net must survive a restore):
		const r = await restoreSessionTabs({
			tabs: [{ path: PATH, libraryName: 'Lib', libraryColor: '#123' }],
			activeTabPath: PATH,
			splitActive: false,
			splitDir: 'vertical',
		} as any);
		expect(r.restored).toBe(1);
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('RECOVERED unsaved edits'); // restored from the net
		expect(getWriteAhead(PATH)).toBeTruthy();               // net preserved (Gate #8 contract)

		// A phantom watcher event (disk unchanged) fires:
		await adoptExternalChangeIntoTabs([PATH], undefined, async (fp) => disk.get(fp)!);

		const t2 = tabs().find((x) => x.path === PATH)!;
		// Pre-fix: the clean model "adopted" stale disk AND clearWriteAhead destroyed the net.
		expect(t2.content).toContain('RECOVERED unsaved edits');
		expect(S.bodyForView(t2.id)).toContain('RECOVERED unsaved edits');
		expect(getWriteAhead(PATH)).toBeTruthy(); // the recovery copy SURVIVES
	});

	it('S7 (disk-unreachable corner): a wab-trusted open with no readable disk is born dirty and cannot be clobbered', async () => {
		const PATH = '/lib/gone.md';
		// No disk entry at all — read_note throws; resolveNoteContent trusts the wab.
		setWriteAhead(PATH, note('G', 'only copy — RECOVERED'), 2, 0);
		await openNoteTab(PATH, 'Lib');
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('only copy — RECOVERED');
		expect(S.isDirty(t.id)).toBe(true); // unsaved work, honestly dirty

		// The file reappears on disk (sync catches up) with old content + a watcher event:
		disk.set(PATH, note('G', 'old synced body'));
		await adoptExternalChangeIntoTabs([PATH], undefined, async (fp) => disk.get(fp)!);
		const t2 = tabs().find((x) => x.path === PATH)!;
		expect(t2.content).toContain('only copy — RECOVERED'); // dirty model wins — never clobbered
	});

	it('S8 (PJ-102c): "Save a copy" writes the unsaved content verbatim to a fresh-identity sibling and opens it', async () => {
		const PATH = '/lib/n.md';
		disk.set(PATH, note('N', 'stale disk body'));
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);
		await openNoteTab(PATH, 'Lib');
		locked.add(PATH); // the persistent lock — every write to the original fails
		reportSaveFailure(PATH, 'n', 'EBUSY: locked');

		const copyPath = await saveRecoveredCopy(PATH);
		expect(copyPath).toBe('/lib/n (recovered copy).md');
		const copy = disk.get(copyPath!)!;
		expect(copy).toContain('RECOVERED unsaved edits'); // the unsaved work, durably on disk
		expect(copy).not.toContain('cid_cn: N'); // a copy is a NEW note — never the original identity
		expect(copy).toContain('title: T (recovered copy)'); // the tab label distinguishes it (Boss remark)
		expect(tabs().some((t) => t.path === copyPath)).toBe(true); // opened as a real tab
		// The locked original is untouched and its banner entry remains (still failing — honest):
		expect(disk.get(PATH)).toBe(note('N', 'stale disk body'));
		let health: Map<string, unknown> = new Map();
		saveHealth.subscribe((v) => (health = v))();
		expect(health.has(PATH)).toBe(true);

		// Collision safety — a second copy gets the next suffix:
		const copy2 = await saveRecoveredCopy(PATH);
		expect(copy2).toBe('/lib/n (recovered copy 2).md');
	});

	it('S9 (PJ-102c): "Discard" reverts the note to disk, clears the net + the banner entry — deliberately', async () => {
		const PATH = '/lib/n.md';
		disk.set(PATH, note('N', 'stale disk body'));
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);
		await openNoteTab(PATH, 'Lib');
		locked.add(PATH);
		reportSaveFailure(PATH, 'n', 'EBUSY: locked');
		expect(tabs().find((t) => t.path === PATH)!.content).toContain('RECOVERED unsaved edits');

		await discardFailedSave(PATH);

		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toBe(note('N', 'stale disk body')); // the disk version, kept by choice
		expect(S.bodyForView(t.id)).toBe('stale disk body');
		expect(getWriteAhead(PATH)).toBeFalsy(); // net dropped — an EXPLICIT discard
		let health: Map<string, unknown> = new Map();
		saveHealth.subscribe((v) => (health = v))();
		expect(health.has(PATH)).toBe(false); // banner entry cleared
	});

	it('S3: a normal reopen with net === disk stays a no-op (no spurious divergence)', async () => {
		const PATH = '/lib/same.md';
		disk.set(PATH, note('M', 'settled body'));
		setWriteAhead(PATH, note('M', 'settled body'), 3, 10); // teardown re-stash, identical

		await openNoteTab(PATH, 'Lib');

		const t = tabs().find((x) => x.path === PATH);
		expect(t!.content).toBe(note('M', 'settled body'));
	});
});

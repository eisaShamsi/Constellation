/**
 * PJ-181 — REPRODUCE-FIRST. A merely-VIEWED note's cached copy overwrites a NEWER
 * external edit. APP-KILLER (silent source-of-truth loss).
 *
 * ── THE RECIPE ───────────────────────────────────────────────────────────────
 *   1. Open a note. Type NOTHING. Close it.
 *      NoteEditor's teardown still stashes a write-ahead entry — `NoteEditor.svelte:370`,
 *      the `!needsDiskSave` branch: *"No disk write (nothing changed since last save) —
 *      still stash the current buffer for crash recovery."* Nothing clears it for a
 *      CLOSED note.
 *   2. The note is edited OUTSIDE Constellation — Syncthing, a second device, `git pull`.
 *      `cid_cn` is unchanged by construction (it is the note's identity, not its version),
 *      and the file watcher ignores the change because the note is closed.
 *   3. Reopen it.
 *
 * `resolveNoteContent` (store.ts) restores the net whenever the two `cid_cn`s match. It
 * compares IDENTITY and never FRESHNESS — so the stale view wins over the newer file, the
 * model is born DIRTY with it (`markModelRecoveredFromNet`), and the first tab switch
 * flushes that stale content over the newer file and reindexes on it. No error, no
 * conflict sidecar, no save-health entry.
 *
 * ── WHY THE CID CHECK CANNOT DECIDE THIS ─────────────────────────────────────
 * Two situations are IDENTICAL under the current test (net cid === disk cid, net ≠ disk):
 *
 *   Recipe S (PJ-102, `tests/mig-076/reopenRecoveryClobber.test.ts`) — a failed save left
 *     the user's ONLY copy in the net. The net is NEWER. **The net must win.**
 *   Recipe V (this file) — a mere view left a copy identical to what was on disk at the
 *     time; disk has since moved on. The net is STALE. **Disk must win.**
 *
 * The distinguishing fact is not identity but whether the net ever held anything the disk
 * did not: in Recipe S the stash differed from the last durable bytes, in Recipe V it did
 * not. Any fix must keep Recipe S green — that is what the control below is for.
 *
 * The codebase already arbitrates this correctly on the SIBLING path: `restoreSessionTabs`
 * passes `preserveNet` and seeds the model with the TRUE disk baseline, so a restored tab
 * is born CLEAN and can never write its recovered view over disk.
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
	setWriteAhead,
	getWriteAhead,
	clearWriteAhead,
	flushDisposeClearTabs,
	standardSaveEnv,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';
import * as M from '$lib/editor/noteModel';
import { SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let locked: Set<string>;
let calls: Array<{ cmd: string; args: any }>;

const PATH = '/lib/viewed.md';

const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;

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
				const c = disk.get(args.filePath) ?? '';
				if (!/cid_cn:/.test(c)) {
					const updated = `---\ncid_cn: INJECTED\n---\n${c}`;
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
	// Unlock BEFORE the reset flush: a previous case may have left the file locked to
	// simulate a failed save, and the reset flush would then fail and RETAIN a net entry
	// into the next case. And clear the net explicitly — it lives in a module-level Map
	// plus localStorage, neither of which vitest resets between cases.
	locked = new Set();
	await flushDisposeClearTabs('test-reset');
	clearWriteAhead(PATH);
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	disk = new Map();
	calls = [];
	wireInvoke();
});

describe('PJ-181 — a merely-VIEWED note must never overwrite a newer external edit', () => {
	it('RECIPE V — reopening after an external edit shows DISK, and the model is CLEAN', async () => {
		// 1. The note as it was when the user viewed it.
		const asViewed = note('N', 'the original body');
		disk.set(PATH, asViewed);
		// The teardown stash left by merely viewing it (NoteEditor.svelte:370) — the
		// stashed bytes are IDENTICAL to what was on disk, because nothing was typed.
		setWriteAhead(PATH, asViewed, 0, 0, true); // flagged a SNAPSHOT, as NoteEditor now does

		// 2. Edited outside Constellation. Same cid_cn — identity never changes.
		const externallyEdited = note('N', 'the original body\n\nA PARAGRAPH ADDED ON ANOTHER DEVICE');
		disk.set(PATH, externallyEdited);

		// 3. Reopen.
		await openNoteTab(PATH, 'Lib');

		const t = tabs().find((x) => x.path === PATH)!;
		expect(t).toBeTruthy();

		// The screen must show the NEWER file, not the stale view.
		expect(t.content).toContain('A PARAGRAPH ADDED ON ANOTHER DEVICE');
		expect(S.bodyForView(t.id)).toContain('A PARAGRAPH ADDED ON ANOTHER DEVICE');

		// And the model must be CLEAN — a dirty model here is the loaded gun: the next
		// tab switch flushes it over the newer file.
		expect(S.isDirty(t.id)).toBe(false);
	});

	it('RECIPE V — the stale content never reaches disk, even after a flush', async () => {
		const asViewed = note('N', 'the original body');
		disk.set(PATH, asViewed);
		setWriteAhead(PATH, asViewed, 0, 0, true); // flagged a SNAPSHOT, as NoteEditor now does

		const externallyEdited = note('N', 'the original body\n\nEXTERNAL WORK');
		disk.set(PATH, externallyEdited);

		await openNoteTab(PATH, 'Lib');
		const t = tabs().find((x) => x.path === PATH)!;

		// Whatever a departure does (tab switch, close, app close), it must not put the
		// stale bytes back on disk.
		//
		// NOTE the signature: `flushIfDirty(id, ENV, origin)`. The first version of this
		// test passed a path string as the env, so `e.write` was undefined, the save
		// returned `write_failed`, and the test went green while the defect was fully
		// live — LL-037 rule 3 exactly. Measured with a real env, the flush returns
		// `{ok:true}` and the external paragraph is gone from disk.
		await S.flushIfDirty(t.id, standardSaveEnv({ origin: 'test-departure' }), 'test-departure');

		expect(disk.get(PATH)).toContain('EXTERNAL WORK');
		expect(disk.get(PATH)).toBe(externallyEdited);
	});

	/**
	 * THE CONTROL THAT MUST NOT BREAK. Recipe S (PJ-102): the net holds the user's ONLY
	 * copy after a failed save. Disk is STALE, the net is NEWER — the net must still win,
	 * and the model must still be born DIRTY so the recovery gets written.
	 *
	 * Any fix that makes Recipe V pass by simply "always preferring disk" fails here, and
	 * that failure is data loss of a different kind — the user's unsaved work.
	 */
	it('CONTROL / Recipe S — a genuine recovery copy still wins and is still born dirty', async () => {
		const lastSaved = note('N', 'stale disk body');
		disk.set(PATH, lastSaved);
		// The net holds MORE than disk ever had — unsaved edits from a failed save.
		setWriteAhead(PATH, note('N', 'stale disk body\nRECOVERED unsaved edits'), 7, 0);

		await openNoteTab(PATH, 'Lib');

		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('RECOVERED unsaved edits');
		expect(S.bodyForView(t.id)).toContain('RECOVERED unsaved edits');
		// Born DIRTY — otherwise nothing would ever write the recovery to disk.
		expect(S.isDirty(t.id)).toBe(true);
	});

	/**
	 * THE CONTROL THE FIRST FIX WAS MISSING — and the build's own safety inspection caught
	 * the APP-KILLER it left open, measured, before it shipped.
	 *
	 * The Recipe-S control above stashes an UNFLAGGED net directly, so it never exercises
	 * the code that DECIDES the flag. The first version of the fix decided it from
	 * `!needsDiskSave`, on the reasoning that reaching that branch means "nothing changed
	 * since the last durable save". It does not. `needsDiskSave` is NotePane's view-level
	 * `dirty`, and `doSave()` clears it at save-REQUEST time (NotePane.svelte:340) — before
	 * the write is attempted — and never restores it on failure.
	 *
	 * So after a FAILED save (the documented `.md`-locked case) any teardown — a tab switch,
	 * the app-close `beforeunload`, switching to Focus — re-stashed the user's ONLY copy
	 * flagged "already durable", and this file's own new branch then rejected it and CLEARED
	 * it. The fix would have deleted precisely what the net exists to protect.
	 *
	 * This case pins the predicate at the point of decision: a model that is DIRTY (work not
	 * yet durably written) must never be stashed as a snapshot, whatever the view flag says.
	 */
	it('a FAILED save left unwritten work — the teardown predicate must be FALSE', async () => {
		const lastSaved = note('N', 'saved body');
		disk.set(PATH, lastSaved);
		await openNoteTab(PATH, 'Lib');
		const t = tabs().find((x) => x.path === PATH)!;

		// The user types and the save FAILS — the documented case: the .md is momentarily
		// locked by Syncthing / OneDrive / a virus scanner. The model stays dirty and the
		// net becomes the ONLY copy of the work.
		locked.add(PATH);
		S.editBody(t.id, 'saved body\nUNSAVED WORK');
		await S.flushIfDirty(t.id, standardSaveEnv({ origin: 'failed-save' }), 'failed-save');

		// The save was ATTEMPTED and failed, so NotePane's view flag is already false here —
		// which is exactly why it must not be what decides. The MODEL still knows the truth.
		expect(S.isDirty(t.id)).toBe(true);
		expect(disk.get(PATH)).toBe(lastSaved); // nothing durable was written

		// ── THE ASSERTION ────────────────────────────────────────────────────────────────
		// This is the predicate `NoteEditor.handleFlush` now passes as `snapshot`. It is
		// asserted directly, and deliberately NOT through a reopen round-trip: once the flag
		// is wrong the downstream paths CANNOT tell a stale snapshot from a recovery copy —
		// that indistinguishability is the whole reason the flag exists. A round-trip test
		// here passed with the flag hard-coded to `true`, i.e. it proved nothing.
		expect(SINGLE_OWNERSHIP && !M.isDirty(t.id)).toBe(false);

		// The consequence of getting it wrong is not re-demonstrated here on purpose. It is
		// mechanically identical to Recipe V above — a snapshot-flagged entry differing from
		// disk is rejected and cleared — and the two cases differ only in WHICH side is
		// newer, which is precisely what the downstream code cannot know. Re-testing it here
		// only produced a case that passed with the flag hard-coded to `true`.
	});

	/** CONTROL — no net at all: ordinary open reads disk and is clean. */
	it('CONTROL — a note with no net entry opens clean from disk', async () => {
		disk.set(PATH, note('N', 'just disk'));
		await openNoteTab(PATH, 'Lib');
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('just disk');
		expect(S.isDirty(t.id)).toBe(false);
	});

	/** CONTROL — the identity guard still rejects a net entry from a reused path. */
	it('CONTROL — a net entry whose cid differs is still rejected in favour of disk', async () => {
		disk.set(PATH, note('NEW', 'the new note at this path'));
		setWriteAhead(PATH, note('OLD', 'a different note that used to live here'), 3, 0);

		await openNoteTab(PATH, 'Lib');
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('the new note at this path');
		expect(getWriteAhead(PATH)).toBeUndefined();
	});
});

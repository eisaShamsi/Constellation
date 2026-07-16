/**
 * Recipe RO (PJ-108, 2026-07-15) — A READ-ONLY HOST FOLLOWING A WIKILINK MUST NOT
 * CONSUME THE SHARED CRASH-RECOVERY NET (APP-KILLER).
 *
 * The write-ahead net (memory + localStorage) is the ONLY copy of an unsaved, save-failed
 * note's edits — the app's documented last-resort recovery (a locked .md leaves the edits here;
 * closeTab proceeds past a failed flush citing exactly this net). localStorage is shared across
 * the same origin, so the SECOND SCREEN's editor sees the MAIN window's net.
 *
 * The wound: the second screen mounts `<NoteEditor readOnly>` with NO onLinkClick override
 * (SecondScreenPage.svelte:1124/:1290), so a `[[wikilink]]` click there falls through to the
 * default handleLinkClick → openNoteTab → resolveNoteContent, which CONSUMES the net
 * (clearWriteAhead, store.ts:2004/2014) unless preserveNet is set. A read-only surface never
 * mounts a writable editor to re-stash it, so the net is destroyed with nothing surfaced: disk
 * still holds the pre-edit body, and a crash before the main window's retry loses the edits.
 * The SS's own openTabs is a SEPARATE window store, so openNoteTab's dedup (which protects the
 * main window / Index preview) never fires for it — the SS is the unique hole.
 *
 * THE FIX under test: openNoteTab gains a `preserveNet` option threaded to resolveNoteContent;
 * handleLinkClick passes its own `readOnly` as preserveNet (and never CREATES a note from a
 * read-only display — a second, smaller Display-not-Domain leak fixed in the same pass).
 *
 * This drives the REAL openNoteTab against a mocked IPC bridge (the reopenRecoveryClobber.test.ts
 * harness pattern). The RO1 RED assertion documents the app-killer mechanism (a plain open still
 * consumes the net — correct for WRITABLE hosts, which re-stash); RO2 proves the preserveNet
 * contract that the read-only path now uses.
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
	setDisplayOnlyWindow,
	flushDisposeClearTabs,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;
let calls: Array<{ cmd: string; args: any }>;

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
			case 'ensure_cid_cn_cmd': {
				const c = disk.get(args.filePath) ?? '';
				return c; // cid present in every fixture → disk verbatim (mirrors canonical.rs)
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
	calls = [];
	setDisplayOnlyWindow(false); // reset the window flag (module-level) between tests
	wireInvoke();
});

describe('Recipe RO — a read-only host must not consume the shared recovery net (PJ-108)', () => {
	it('RO1 (the wound): a plain openNoteTab CONSUMES the net — the writable manual-open semantics', async () => {
		const PATH = '/lib/x.md';
		// Main window holds X open, dirty, save FAILED — its ONLY unsaved copy is the net.
		disk.set(PATH, note('X', 'last saved body'));
		setWriteAhead(PATH, note('X', 'last saved body\nUNSAVED edits'), 3, 0);
		expect(getWriteAhead(PATH)).toBeTruthy(); // the recovery net exists

		await openNoteTab(PATH, 'Lib'); // no preserveNet — today's read-only link-click path

		// The net is consumed. For a WRITABLE host this is fine (its editor re-stashes on
		// teardown); for the read-only second screen there is no re-stash → the net is GONE.
		expect(getWriteAhead(PATH)).toBeUndefined();
		// The tab still shows the recovered content (resolveNoteContent returns it either way).
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('UNSAVED edits');
	});

	it('RO2 (the fix): openNoteTab with preserveNet leaves the net intact AND still shows the note', async () => {
		const PATH = '/lib/x.md';
		disk.set(PATH, note('X', 'last saved body'));
		setWriteAhead(PATH, note('X', 'last saved body\nUNSAVED edits'), 3, 0);

		// The read-only host's fixed path: preserveNet = true (the 8th positional arg).
		await openNoteTab(PATH, 'Lib', '#7c3aed', undefined, undefined, undefined, undefined, true);

		// THE APP-KILLER assertion — the shared crash-recovery net SURVIVES:
		const net = getWriteAhead(PATH);
		expect(net).toBeTruthy();
		expect(net!.content).toContain('UNSAVED edits');
		// …and the note is still displayed (a read-only peek that harms nothing):
		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('UNSAVED edits');
	});

	it('RO3: preserveNet is inert when there is no net (a normal note opens exactly as before)', async () => {
		const PATH = '/lib/plain.md';
		disk.set(PATH, note('P', 'plain body'));

		await openNoteTab(PATH, 'Lib', '#7c3aed', undefined, undefined, undefined, undefined, true);

		const t = tabs().find((x) => x.path === PATH)!;
		expect(t.content).toContain('plain body');
		expect(getWriteAhead(PATH)).toBeUndefined(); // never had a net; none created
	});

	it('RO4 (Solve-the-Class): in a display-only window, a PLAIN openNoteTab preserves the net — every SS call site is covered', async () => {
		const PATH = '/lib/x.md';
		disk.set(PATH, note('X', 'last saved body'));
		setWriteAhead(PATH, note('X', 'last saved body\nUNSAVED edits'), 3, 0);

		// The second screen marks its whole store display-only at init — so a bare openNoteTab
		// (the note-list click at SecondScreenPage:1261, the restore at :717, etc.) preserves the
		// net with NO explicit preserveNet arg. This is what makes the fix miss-proof.
		setDisplayOnlyWindow();
		await openNoteTab(PATH, 'Lib'); // no explicit preserveNet — the window flag supplies it

		const net = getWriteAhead(PATH);
		expect(net).toBeTruthy();
		expect(net!.content).toContain('UNSAVED edits');
	});

	it('RO5: an explicit preserveNet:false still wins over the window flag (an intentional consume)', async () => {
		const PATH = '/lib/x.md';
		disk.set(PATH, note('X', 'body'));
		setWriteAhead(PATH, note('X', 'body\nUNSAVED'), 3, 0);
		setDisplayOnlyWindow();

		await openNoteTab(PATH, 'Lib', '#7c3aed', undefined, undefined, undefined, undefined, false);

		// preserveNet ?? displayOnlyWindow → explicit false wins → net consumed (documents the
		// precedence; no SS caller passes false, but the contract must be unambiguous).
		expect(getWriteAhead(PATH)).toBeUndefined();
	});
});

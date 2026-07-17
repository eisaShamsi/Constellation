/**
 * Sweep-2026-07-18 finding #2 (APP-KILLER) + #10 — `loadTabHistoryEntry` (Alt+←/→ history nav).
 *
 * The wound (two facets of one gap):
 *   #2  loadTabHistoryEntry has NO one-path-one-tab dedup (the B1 DEDUP_ALL_TABS guard lives
 *       only in openNoteTab), so Alt+Back can land a tab on a path ALREADY open in another tab,
 *       minting a SECOND independent NoteModel for the same file — the exact two-models-one-path
 *       clobber class B1 was built to kill (committed on-disk content lost, no error, no sidecar).
 *   #10 loadTabHistoryEntry raw-reads via invoke('read_note'), BYPASSING resolveNoteContent — a
 *       note whose only unsaved copy is the write-ahead net is re-seeded CLEAN from stale disk on
 *       an Alt-nav, and the documented reopen-restore route never runs.
 *
 * These drive the REAL store helpers end-to-end against a seeded openTabs + a mocked invoke
 * (mirrors readonlyLinkPreservesNet.test.ts). RED on the pre-fix code; GREEN after the fix routes
 * history nav through the same dedup + resolveNoteContent path openNoteTab already uses.
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
	navigateBack,
	setWriteAhead,
	getWriteAhead,
	setDisplayOnlyWindow,
	flushDisposeClearTabs,
} from '$lib/libraries/store';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

let disk: Map<string, string>;

const note = (cid: string, body: string) => `---\ntitle: T\ncid_cn: ${cid}\n---\n${body}`;

function wireInvoke() {
	mockInvoke.mockReset();
	mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
		switch (cmd) {
			case 'read_note': {
				const c = disk.get(args.filePath);
				if (c === undefined) throw new Error('missing');
				return c;
			}
			case 'ensure_cid_cn_cmd': {
				return disk.get(args.filePath) ?? ''; // cid present in every fixture → disk verbatim
			}
			case 'write_note':
			case 'save_note':
				// A durable write lands on our fake disk (used by the flush path).
				if (args?.filePath && typeof args?.content === 'string') disk.set(args.filePath, args.content);
				return undefined;
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
const tabsOn = (path: string) => tabs().filter((t) => t.path === path);

beforeEach(async () => {
	await flushDisposeClearTabs('test-reset');
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	disk = new Map();
	setDisplayOnlyWindow(false);
	wireInvoke();
});

describe('Recipe HN — history nav must honor one-path-one-tab + the recovery net', () => {
	it('HN1 (the wound #2): Alt+Back onto a path already open in ANOTHER tab must not mint a second tab/model', async () => {
		disk.set('/N.md', note('N', 'N-body'));
		disk.set('/M.md', note('M', 'M-body'));

		// Tab B: open N, then navigate in-place to M → B.history = [N, M], B now on M.
		await openNoteTab('/N.md', 'Lib');
		const bId = tabs()[0].id;
		await openNoteTab('/M.md', 'Lib'); // in-place reuse of B
		expect(tabs().find((t) => t.id === bId)?.path).toBe('/M.md');

		// Open N again in a SEPARATE tab A (Ctrl+click → newTab). N is not open anywhere now, so
		// dedup passes and a fresh tab A is created on N.
		await openNoteTab('/N.md', 'Lib', '#7c3aed', undefined, true);
		expect(tabsOn('/N.md').length).toBe(1); // only tab A on N so far

		// User presses Alt+Back in tab B → target = N, which is ALREADY open in tab A.
		activeTabId.set(bId);
		navigateBack();
		await Promise.resolve(); await Promise.resolve(); await new Promise((r) => setTimeout(r, 0));

		// THE INVARIANT: one path → one tab. Pre-fix this is 2 (B also became N → two models → clobber).
		expect(tabsOn('/N.md').length).toBe(1);
	});

	it('HN2 (the wound #10): Alt+Back must recover from the write-ahead net, not raw-read stale disk', async () => {
		disk.set('/N.md', note('N', 'saved body'));
		disk.set('/M.md', note('M', 'M-body'));

		// The ONLY copy of N's unsaved edits lives in the net — a prior failed save whose tab was
		// CLOSED (closeTab retains the net per its documented reopen-restore contract). N is open
		// nowhere; it merely sits in tab B's back-history.
		setWriteAhead('/N.md', note('N', 'saved body\nUNSAVED edits'), 5, 0);

		// Seed tab B directly: currently on M, history = [N, M] (index 1). Model for M only.
		const bTab = {
			id: 'B', path: '/M.md', content: disk.get('/M.md')!,
			libraryName: 'Lib', libraryPath: '/Lib', name: 'M.md', libraryColor: '#000',
			history: ['/N.md', '/M.md'], historyIndex: 1,
		} as any;
		openTabs.set([bTab]);
		S.open('B', '/M.md', disk.get('/M.md')!);
		expect(getWriteAhead('/N.md')).toBeTruthy(); // orphaned net present

		// Alt+Back to N.
		activeTabId.set('B');
		navigateBack();
		await Promise.resolve(); await Promise.resolve(); await new Promise((r) => setTimeout(r, 0));

		// GREEN: the recovered (net) content is on screen; pre-fix the raw read shows stale disk.
		const t = tabs().find((x) => x.id === 'B')!;
		expect(t.path).toBe('/N.md');
		expect(t.content).toContain('UNSAVED edits');
	});
});

/**
 * MIG-100 §7 — the boot-restore recipes (R1–R7 from the Plan).
 *
 * Proves, headlessly, against the REAL store + session modules with a mocked
 * IPC bridge:
 *   R1 — a batch restore performs ZERO write-class IPCs (Gate #8),
 *   R2 — 0-of-N restored = failure: snapshot preserved, arm deferred to the
 *        first user tab mutation,
 *   R3 — a universe switch mid-restore aborts with zero store mutations and
 *        no arm,
 *   R4 — restore appends and never steals focus from a live user,
 *   R5 — a restored tab's model is the view/save source (screen === disk
 *        after edit + save),
 *   R6 — deferred cid-ensure drains on first USER activation, never at boot,
 *   R7 — a mid-restore throw still arms tracking (journal-marked, never a
 *        silently dead tracker) — plus the crash-loop sentinel skip.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	restoreSessionTabs,
	openTabs,
	activeTabId,
	focusedTabId,
	splitActive,
	splitDirection,
	setWriteAhead,
	getWriteAhead,
	clearWriteAhead,
} from '$lib/libraries/store';
import {
	restoreSessionThenTrack,
	stopSessionTracking,
	isSessionTracking,
	SESSION_DEBOUNCE_MS,
} from '$lib/libraries/session';
import * as S from '$lib/editor/noteSession';

const mockInvoke = vi.mocked(invoke);

/** Fake disk + IPC ledger. */
let disk: Map<string, string>;
let calls: Array<{ cmd: string; args: any }>;

const note = (cid: string, body: string) => `---\ntitle: T-${cid}\ncid_cn: ${cid}\n---\n${body}`;

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
				if (!/cid_cn:/.test(c)) {
					const updated = c.startsWith('---')
						? c.replace(/^---\n/, '---\ncid_cn: INJECTED\n')
						: `---\ncid_cn: INJECTED\n---\n${c}`;
					disk.set(args.filePath, updated);
					return updated;
				}
				return c;
			}
			case 'read_universe_session':
			case 'save_universe_session':
			case 'journal_frontend_marker':
			case 'write_note':
				return undefined;
			default:
				return undefined;
		}
	});
}

const writes = () =>
	calls.filter((c) =>
		['write_note', 'ensure_cid_cn_cmd', 'save_universe_session'].includes(c.cmd)
	);
const journalMarks = () =>
	calls.filter((c) => c.cmd === 'journal_frontend_marker').map((c) => c.args.surface);

const snapOf = (paths: string[], active = paths[0] ?? null) => ({
	tabs: paths.map((p) => ({ path: p, libraryName: 'Lib', libraryColor: '#123' })),
	activeTabPath: active,
	splitActive: false,
	splitDir: 'vertical' as const,
});

beforeEach(async () => {
	await stopSessionTracking();
	openTabs.set([]);
	activeTabId.set(null);
	focusedTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	S.closeAll();
	disk = new Map();
	calls = [];
	wireInvoke();
	try { localStorage.removeItem('constellation-session-restoring'); } catch { /* no localStorage */ }
});

describe('R1 — Gate #8: a restore performs zero write-class IPCs', () => {
	it('restores N tabs with reads only; active + split applied', async () => {
		disk.set('/lib/a.md', note('A', 'body A'));
		disk.set('/lib/b.md', note('B', 'body B'));
		disk.set('/lib/c.md', note('C', 'body C'));
		const r = await restoreSessionTabs(snapOf(['/lib/a.md', '/lib/b.md', '/lib/c.md'], '/lib/b.md'));
		expect(r).toMatchObject({ restored: 3, requested: 3 });
		expect(writes()).toHaveLength(0); // THE Gate #8 assertion
		const tabs = openTabs;
		let current: any[] = [];
		tabs.subscribe((v) => (current = v))();
		expect(current.map((t) => t.path)).toEqual(['/lib/a.md', '/lib/b.md', '/lib/c.md']);
		let active: string | null = null;
		activeTabId.subscribe((v) => (active = v))();
		expect(current.find((t) => t.id === active)?.path).toBe('/lib/b.md');
	});

	it('a missing file is skipped without aborting the rest', async () => {
		disk.set('/lib/a.md', note('A', 'body A'));
		// /lib/gone.md not on disk
		const r = await restoreSessionTabs(snapOf(['/lib/gone.md', '/lib/a.md']));
		expect(r).toMatchObject({ restored: 1, requested: 2 });
		expect(writes()).toHaveLength(0);
	});
});

describe('R2 — 0-of-N restored = failure: preserve, defer arm', () => {
	it('unmounted-drive shape: nothing restored → not tracking; first user mutation arms', async () => {
		vi.useFakeTimers();
		try {
			const raw = {
				version: 1, savedAt: 1,
				tabs: [
					{ path: '/drive/x.md', libraryName: 'L', libraryColor: '#1' },
					{ path: '/drive/y.md', libraryName: 'L', libraryColor: '#1' },
				],
				activeTabPath: '/drive/x.md', splitActive: false, splitDir: 'vertical',
			};
			await restoreSessionThenTrack(raw, 'E:/U/A', { enabled: true, safeBootMode: false });
			expect(journalMarks()).toContain('session_restore_failed');
			expect(isSessionTracking()).toBe(false);
			// The snapshot was never overwritten:
			expect(calls.filter((c) => c.cmd === 'save_universe_session')).toHaveLength(0);
			// First USER tab mutation arms tracking and persists fresh state.
			openTabs.set([{ id: 't1', path: '/lib/new.md', content: '', libraryName: 'L', libraryPath: '', name: 'new', libraryColor: '#1', history: [], historyIndex: 0 }]);
			expect(isSessionTracking()).toBe(true);
			await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
			const saves = calls.filter((c) => c.cmd === 'save_universe_session');
			expect(saves).toHaveLength(1);
			expect(saves[0].args.session.tabs.map((t: any) => t.path)).toEqual(['/lib/new.md']);
		} finally {
			vi.useRealTimers();
		}
	});
});

describe('R3 — a universe switch mid-restore aborts with zero mutations', () => {
	it('stillValid=false at commit time → no tabs inserted, aborted flag', async () => {
		disk.set('/lib/a.md', note('A', 'body A'));
		const r = await restoreSessionTabs(snapOf(['/lib/a.md']), () => false);
		expect(r.aborted).toBe(true);
		expect(r.restored).toBe(0);
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		expect(current).toHaveLength(0);
	});
});

describe('R4 — restore never steals focus from a live user', () => {
	it('user tab already open+active → restored tabs append, focus unchanged, no dup', async () => {
		disk.set('/lib/a.md', note('A', 'body A'));
		disk.set('/lib/user.md', note('U', 'user note'));
		openTabs.set([{ id: 'user1', path: '/lib/user.md', content: note('U', 'user note'), libraryName: 'L', libraryPath: '', name: 'user', libraryColor: '#1', history: [], historyIndex: 0 }]);
		activeTabId.set('user1');
		const r = await restoreSessionTabs(snapOf(['/lib/a.md', '/lib/user.md'], '/lib/a.md'));
		expect(r.restored).toBe(1); // user.md deduped — one path, one tab
		let active: string | null = null;
		activeTabId.subscribe((v) => (active = v))();
		expect(active).toBe('user1'); // focus NOT stolen
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		expect(current.map((t) => t.path)).toEqual(['/lib/user.md', '/lib/a.md']);
	});
});

describe('R5 — a restored tab’s model is the view/save source', () => {
	it('screen === disk after restore; edit + save writes the model body', async () => {
		disk.set('/lib/a.md', note('A', 'original body'));
		await restoreSessionTabs(snapOf(['/lib/a.md']));
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		const tab = current[0];
		expect(S.bodyForView(tab.id)).toBe('original body'); // model seeded from disk
		S.editBody(tab.id, 'edited body');
		await S.save(tab.id, tab.path, (p, c) => { disk.set(p, c); });
		expect(disk.get('/lib/a.md')).toContain('edited body');
		expect(S.bodyForView(tab.id)).toBe('edited body'); // screen === disk
	});
});

describe('R6 — deferred cid-ensure: zero at boot, drains on first user activation', () => {
	it('cid-less restored note: no ensure at restore; ensure + adopt on activation', async () => {
		disk.set('/lib/nocid.md', '---\ntitle: NoCid\n---\nplain body');
		disk.set('/lib/a.md', note('A', 'body A'));
		await restoreSessionTabs(snapOf(['/lib/a.md', '/lib/nocid.md'], '/lib/a.md'));
		expect(calls.filter((c) => c.cmd === 'ensure_cid_cn_cmd')).toHaveLength(0); // Gate #8
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		const noCidTab = current.find((t) => t.path === '/lib/nocid.md')!;
		// USER activates the cid-less tab:
		activeTabId.set(noCidTab.id);
		await vi.waitFor(() => {
			expect(calls.filter((c) => c.cmd === 'ensure_cid_cn_cmd')).toHaveLength(1);
		});
		await vi.waitFor(() => {
			// the model ADOPTED the cid-bearing disk (reloadTabsFromDisk ran)
			expect(disk.get('/lib/nocid.md')).toContain('cid_cn: INJECTED');
		});
	});
});

describe('R8 — restore honors the crash-recovery net WITHOUT consuming it', () => {
	it('write-ahead content is shown, and the net survives the restore (inspection APP-KILLER fix)', async () => {
		disk.set('/lib/x.md', note('X', 'stale disk body'));
		setWriteAhead('/lib/x.md', note('X', 'recovered newer body'), 5, 10);
		try {
			await restoreSessionTabs(snapOf(['/lib/x.md']));
			let current: any[] = [];
			openTabs.subscribe((v) => (current = v))();
			// Recovery honored — the tab shows the write-ahead content …
			expect(current[0].content).toContain('recovered newer body');
			// … and the net was NOT destroyed: the model is born clean, so the
			// net is the ONLY durable copy until a real save replaces it.
			expect(getWriteAhead('/lib/x.md')?.content).toContain('recovered newer body');
		} finally {
			clearWriteAhead('/lib/x.md');
		}
	});
});

describe('R9 — cid drain never discards keystrokes typed during the drain', () => {
	it('a model dirtied mid-drain keeps the user content; no adopt over it', async () => {
		disk.set('/lib/typed.md', '---\ntitle: Typed\n---\noriginal body');
		disk.set('/lib/other.md', note('O', 'other'));
		await restoreSessionTabs(snapOf(['/lib/other.md', '/lib/typed.md'], '/lib/other.md'));
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		const typedTab = current.find((t) => t.path === '/lib/typed.md')!;
		// Simulate the race: the ensure IPC lands WHILE the user is typing —
		// the mock dirties the model synchronously inside the drain window.
		const base = mockInvoke.getMockImplementation()!;
		mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
			if (cmd === 'ensure_cid_cn_cmd') {
				S.editBody(typedTab.id, 'user typed during drain');
			}
			return base(cmd, args);
		});
		activeTabId.set(typedTab.id); // user activates → drain fires
		await vi.waitFor(() => {
			expect(calls.filter((c) => c.cmd === 'ensure_cid_cn_cmd')).toHaveLength(1);
		});
		await new Promise((r) => setTimeout(r, 30)); // let the guarded adopt settle
		// The user's keystrokes survive — the dirty model was never re-seeded.
		expect(S.bodyForView(typedTab.id)).toBe('user typed during drain');
		let after: any[] = [];
		openTabs.subscribe((v) => (after = v))();
		expect(after.find((t) => t.path === '/lib/typed.md')!.content).not.toContain('cid_cn: INJECTED');
	});
});

describe('R10 — Boss Stage-2 failure 4: a payload from universe A must never restore under universe B', () => {
	it('cross-root payload is discarded (re-read of the CORRECT root finds nothing)', async () => {
		// The real incident (2026-07-11 19:05): the boot bundle's session was
		// read while Eisa Cognitive Knowledge was active; the restore then
		// applied it inside the freshly-activated Scratch and the tracker
		// wrote the foreign tab into Scratch's session.json.
		disk.set('/eck/note.md', note('E', 'ECK note'));
		const eckPayload = {
			version: 1, savedAt: 1,
			tabs: [
				{ path: '/eck/note.md', libraryName: 'ECK', libraryColor: '#1' },
				{ path: '/eck/other.md', libraryName: 'ECK', libraryColor: '#1' },
			],
			activeTabPath: '/eck/note.md', splitActive: false, splitDir: 'vertical',
		};
		await restoreSessionThenTrack(eckPayload, 'E:/U/Scratch', {
			enabled: true,
			safeBootMode: false,
			bundleRoot: 'E:/U/EisaCognitiveKnowledge', // payload origin ≠ arm root
		});
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		expect(current).toHaveLength(0); // NOTHING from A restores under B
		expect(journalMarks()).toContain('session_restore_payload_mismatch');
		// The fallback re-read targeted the ARM root, not the payload's:
		const reads = calls.filter((c) => c.cmd === 'read_universe_session');
		expect(reads).toHaveLength(1);
		expect(reads[0].args.universeRoot).toBe('E:/U/Scratch');
		expect(isSessionTracking()).toBe(true); // still arms — for Scratch, clean
	});

	it('matching roots restore normally (normalized comparison — slashes/case)', async () => {
		disk.set('/lib/a.md', note('A', 'body A'));
		const payload = {
			version: 1, savedAt: 1,
			tabs: [{ path: '/lib/a.md', libraryName: 'L', libraryColor: '#1' }],
			activeTabPath: '/lib/a.md', splitActive: false, splitDir: 'vertical',
		};
		await restoreSessionThenTrack(payload, 'E:\\U\\A', {
			enabled: true,
			safeBootMode: false,
			bundleRoot: 'e:/u/a/', // same universe, different separators/case
		});
		let current: any[] = [];
		openTabs.subscribe((v) => (current = v))();
		expect(current.map((t) => t.path)).toEqual(['/lib/a.md']);
	});
});

describe('R11 — Boss Stage-2 failure 3: a missing file must not resurrect from the crash net', () => {
	it('wab exists but disk is gone → tab skipped, net preserved', async () => {
		// The real incident: "Testing opened note.md" was moved away while the
		// app was closed; its teardown-re-stashed write-ahead entry then
		// resurrected a ghost tab at the dead path.
		setWriteAhead('/lib/moved.md', note('M', 'ghost content'), 0, 0);
		disk.set('/lib/a.md', note('A', 'body A'));
		try {
			const r = await restoreSessionTabs(snapOf(['/lib/moved.md', '/lib/a.md'], '/lib/a.md'));
			expect(r).toMatchObject({ restored: 1, requested: 2 }); // ghost SKIPPED
			let current: any[] = [];
			openTabs.subscribe((v) => (current = v))();
			expect(current.map((t) => t.path)).toEqual(['/lib/a.md']);
			// The net is NOT consumed — real recovery stays possible later.
			expect(getWriteAhead('/lib/moved.md')?.content).toContain('ghost content');
		} finally {
			clearWriteAhead('/lib/moved.md');
		}
	});
});

describe('R12 — the wab REJECT branch honors preserveNet (hotfix-inspection MED)', () => {
	it('identity-unproven wab at restore: disk content used, net NOT destroyed', async () => {
		// A cid-less recovery copy (deferred-cid note whose save failed) with a
		// cid-bearing disk: identity unproven → the restore must fall back to
		// disk WITHOUT destroying the possibly-only copy of unsaved edits.
		disk.set('/lib/x.md', note('X', 'disk body'));
		setWriteAhead('/lib/x.md', 'no frontmatter — cid-less unsaved edits', 0, 0);
		try {
			await restoreSessionTabs(snapOf(['/lib/x.md']));
			let current: any[] = [];
			openTabs.subscribe((v) => (current = v))();
			expect(current[0].content).toContain('disk body'); // disk wins the VIEW
			// …but the net survives for manual recovery:
			expect(getWriteAhead('/lib/x.md')?.content).toContain('cid-less unsaved edits');
		} finally {
			clearWriteAhead('/lib/x.md');
		}
	});
});

describe('R13 — a transiently unreadable tab is carried forward, not silently pruned (hotfix-inspection LOW)', () => {
	it('k-of-N restore: the skipped tab stays in the next persisted snapshot (one-boot grace)', async () => {
		vi.useFakeTimers();
		try {
			disk.set('/lib/ok.md', note('K', 'ok'));
			const payload = {
				version: 1, savedAt: 1,
				tabs: [
					{ path: '/lib/ok.md', libraryName: 'L', libraryColor: '#1' },
					{ path: '/lib/transient.md', libraryName: 'L', libraryColor: '#1' }, // read fails
				],
				activeTabPath: '/lib/ok.md', splitActive: false, splitDir: 'vertical',
			};
			await restoreSessionThenTrack(payload, 'E:/U/A', { enabled: true, safeBootMode: false });
			// arrangement change → persist: the snapshot must still CONTAIN the
			// transiently-missing tab (carried), not prune it.
			openTabs.update((ts) => ts.map((t) => ({ ...t, pinned: true })));
			await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
			const s = calls.filter((c) => c.cmd === 'save_universe_session');
			expect(s.length).toBeGreaterThan(0);
			const savedTabs = s.at(-1)!.args.session.tabs;
			expect(savedTabs.map((t: any) => t.path)).toContain('/lib/transient.md');
			expect(savedTabs.find((t: any) => t.path === '/lib/transient.md')?.carried).toBe(true);
		} finally {
			vi.useRealTimers();
		}
	});

	it('a CARRIED tab that fails again is dropped (two strikes — no immortal ghosts)', async () => {
		vi.useFakeTimers();
		try {
			disk.set('/lib/ok.md', note('K', 'ok'));
			const payload = {
				version: 1, savedAt: 1,
				tabs: [
					{ path: '/lib/ok.md', libraryName: 'L', libraryColor: '#1' },
					{ path: '/lib/gone.md', libraryName: 'L', libraryColor: '#1', carried: true }, // 2nd strike
				],
				activeTabPath: '/lib/ok.md', splitActive: false, splitDir: 'vertical',
			};
			await restoreSessionThenTrack(payload, 'E:/U/A', { enabled: true, safeBootMode: false });
			openTabs.update((ts) => ts.map((t) => ({ ...t, pinned: true })));
			await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
			const s = calls.filter((c) => c.cmd === 'save_universe_session');
			const savedTabs = s.at(-1)!.args.session.tabs;
			expect(savedTabs.map((t: any) => t.path)).not.toContain('/lib/gone.md');
		} finally {
			vi.useRealTimers();
		}
	});
});

describe('R7 — a broken restore still arms tracking (never silently dead)', () => {
	it('mid-restore throw → journal error marker + tracking armed', async () => {
		// read_note resolves a NON-STRING → content.match throws inside the
		// batch loop, outside the per-tab catch — the restore itself dies.
		mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
			calls.push({ cmd, args });
			if (cmd === 'read_note') return 42 as unknown as string;
			return undefined;
		});
		const raw = {
			version: 1, savedAt: 1,
			tabs: [{ path: '/lib/a.md', libraryName: 'L', libraryColor: '#1' }],
			activeTabPath: null, splitActive: false, splitDir: 'vertical',
		};
		await restoreSessionThenTrack(raw, 'E:/U/A', { enabled: true, safeBootMode: false });
		expect(journalMarks()).toContain('session_restore_error');
		expect(isSessionTracking()).toBe(true); // armed in finally
	});

	it('crash sentinel present → restore skipped once, arm deferred', async () => {
		let lsOk = true;
		try { localStorage.setItem('constellation-session-restoring', '1'); } catch { lsOk = false; }
		if (!lsOk) return; // env without localStorage — the guard path is a no-op there
		const raw = {
			version: 1, savedAt: 1,
			tabs: [{ path: '/lib/a.md', libraryName: 'L', libraryColor: '#1' }],
			activeTabPath: null, splitActive: false, splitDir: 'vertical',
		};
		disk.set('/lib/a.md', note('A', 'body A'));
		await restoreSessionThenTrack(raw, 'E:/U/A', { enabled: true, safeBootMode: false });
		expect(journalMarks()).toContain('session_restore_skipped');
		expect(calls.filter((c) => c.cmd === 'read_note')).toHaveLength(0); // nothing restored
		expect(isSessionTracking()).toBe(false); // deferred, not dead
		expect(localStorage.getItem('constellation-session-restoring')).toBeNull(); // cleared — next boot tries again
	});

	it('toggle OFF → no restore, no tracking, snapshot untouched', async () => {
		const raw = {
			version: 1, savedAt: 1,
			tabs: [{ path: '/lib/a.md', libraryName: 'L', libraryColor: '#1' }],
			activeTabPath: null, splitActive: false, splitDir: 'vertical',
		};
		await restoreSessionThenTrack(raw, 'E:/U/A', { enabled: false, safeBootMode: false });
		expect(calls).toHaveLength(0);
		expect(isSessionTracking()).toBe(false);
	});

	it('safeBootMode → skip + journal, no arm (snapshot preserved)', async () => {
		const raw = {
			version: 1, savedAt: 1,
			tabs: [{ path: '/lib/a.md', libraryName: 'L', libraryColor: '#1' }],
			activeTabPath: null, splitActive: false, splitDir: 'vertical',
		};
		await restoreSessionThenTrack(raw, 'E:/U/A', { enabled: true, safeBootMode: true });
		expect(journalMarks()).toContain('session_restore_skipped');
		expect(calls.filter((c) => c.cmd === 'read_note')).toHaveLength(0);
		expect(isSessionTracking()).toBe(false);
	});
});

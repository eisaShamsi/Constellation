/**
 * MIG-100 §2 — the session tracker (auto-restore tabs on relaunch).
 *
 * Proves the tracker's safety contract headlessly against the REAL stores:
 * signature stability (content/cursor churn never writes), debounce
 * coalescing, the unarmed/seed guards (the empty-overwrite race is
 * structural), in-flight serialization, failed-write retry (never a silent
 * drop), and stop's cancel-and-flush to the root captured at arm time.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import {
	startSessionTracking,
	stopSessionTracking,
	persistSessionNow,
	captureSessionSnapshot,
	sessionSignature,
	sessionGeneration,
	isSessionTracking,
	deleteSessionOnDisk,
	SESSION_DEBOUNCE_MS,
	type SessionSnapshot,
} from '$lib/libraries/session';
import { openTabs, activeTabId, splitActive, splitDirection, type OpenTab } from '$lib/libraries/store';

const mockInvoke = vi.mocked(invoke);

const tab = (id: string, path: string, extra: Partial<OpenTab> = {}): OpenTab => ({
	id,
	path,
	content: '',
	libraryName: 'Lib',
	libraryPath: '/lib',
	name: path,
	libraryColor: '#123456',
	history: [],
	historyIndex: 0,
	...extra,
});

/** The save_universe_session calls only (root, snapshot). */
const saves = () =>
	mockInvoke.mock.calls.filter(([cmd]) => cmd === 'save_universe_session') as Array<
		[string, { universeRoot: string; session: SessionSnapshot | null }]
	>;

beforeEach(async () => {
	vi.useFakeTimers();
	await stopSessionTracking();
	openTabs.set([]);
	activeTabId.set(null);
	splitActive.set(false);
	splitDirection.set('vertical');
	mockInvoke.mockReset();
	mockInvoke.mockResolvedValue(undefined);
});

afterEach(async () => {
	await stopSessionTracking();
	vi.useRealTimers();
});

describe('MIG-100 §2 — session tracker', () => {
	it('unarmed: store mutations never write (the empty-overwrite guard is structural)', async () => {
		expect(isSessionTracking()).toBe(false);
		openTabs.set([tab('t1', '/lib/a.md')]);
		openTabs.set([]);
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS * 3);
		expect(saves()).toHaveLength(0);
	});

	it('armed: a tab mutation debounces into exactly one write with the captured root', async () => {
		startSessionTracking('E:/U/A');
		openTabs.set([tab('t1', '/lib/a.md')]);
		activeTabId.set('t1');
		openTabs.update((ts) => [...ts, tab('t2', '/lib/b.md')]);
		expect(saves()).toHaveLength(0); // nothing before the debounce window
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
		const s = saves();
		expect(s).toHaveLength(1);
		expect(s[0][1].universeRoot).toBe('E:/U/A');
		expect(s[0][1].session?.tabs.map((t) => t.path)).toEqual(['/lib/a.md', '/lib/b.md']);
		expect(s[0][1].session?.activeTabPath).toBe('/lib/a.md');
	});

	it('seed signature: arming after restore does not rewrite the identical arrangement', async () => {
		openTabs.set([tab('t1', '/lib/a.md')]);
		activeTabId.set('t1');
		startSessionTracking('E:/U/A', sessionSignature(captureSessionSnapshot()));
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS * 3);
		expect(saves()).toHaveLength(0);
		// …but a REAL change still writes.
		openTabs.update((ts) => [...ts, tab('t2', '/lib/b.md')]);
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
		expect(saves()).toHaveLength(1);
	});

	it('Rule 8: content / cursor / scroll churn never schedules a write', async () => {
		openTabs.set([tab('t1', '/lib/a.md')]);
		startSessionTracking('E:/U/A', sessionSignature(captureSessionSnapshot()));
		openTabs.update((ts) => ts.map((t) => ({ ...t, content: 'typed text', cursorPos: 42, scrollTop: 300 })));
		openTabs.update((ts) => ts.map((t) => ({ ...t, content: 'more typed text' })));
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS * 3);
		expect(saves()).toHaveLength(0);
	});

	it('empty tabs (no file) are excluded from the snapshot', () => {
		openTabs.set([tab('t1', '/lib/a.md'), tab('empty', '')]);
		const snap = captureSessionSnapshot();
		expect(snap.tabs.map((t) => t.path)).toEqual(['/lib/a.md']);
	});

	it('a failed write is retained and retried on the next mutation — never dropped', async () => {
		startSessionTracking('E:/U/A');
		mockInvoke.mockRejectedValueOnce(new Error('disk lock'));
		openTabs.set([tab('t1', '/lib/a.md')]);
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
		expect(saves()).toHaveLength(1); // attempted, failed
		openTabs.update((ts) => ts.map((t) => ({ ...t, pinned: true })));
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
		expect(saves()).toHaveLength(2); // retried
	});

	it('writes are serialized through one in-flight promise (ordered, never interleaved)', async () => {
		const order: string[] = [];
		let releaseFirst!: () => void;
		mockInvoke
			.mockImplementationOnce(async () => {
				order.push('first:start');
				await new Promise<void>((r) => (releaseFirst = r));
				order.push('first:end');
			})
			.mockImplementationOnce(async () => {
				order.push('second:start');
			});
		startSessionTracking('E:/U/A');
		openTabs.set([tab('t1', '/lib/a.md')]);
		const p1 = persistSessionNow();
		openTabs.update((ts) => [...ts, tab('t2', '/lib/b.md')]);
		const p2 = persistSessionNow();
		await vi.advanceTimersByTimeAsync(5);
		releaseFirst();
		await p1;
		await p2;
		expect(order).toEqual(['first:start', 'first:end', 'second:start']);
	});

	it('stop cancel-and-flushes the pending change to the ARM-time root, then goes inert', async () => {
		startSessionTracking('E:/U/A');
		openTabs.set([tab('t1', '/lib/a.md')]);
		// Debounce still pending — stop must flush it, not lose it.
		const stopped = stopSessionTracking();
		// The switch handler clears stores right after stop; the flush already
		// captured its snapshot synchronously, so this cannot empty the write.
		openTabs.set([]);
		await vi.advanceTimersByTimeAsync(5);
		await stopped;
		const s = saves();
		expect(s).toHaveLength(1);
		expect(s[0][1].universeRoot).toBe('E:/U/A');
		expect(s[0][1].session?.tabs.map((t) => t.path)).toEqual(['/lib/a.md']);
		// Inert after stop: further mutations write nothing.
		openTabs.set([tab('t9', '/lib/z.md')]);
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS * 3);
		expect(saves()).toHaveLength(1);
		expect(isSessionTracking()).toBe(false);
	});

	it('stop bumps the restore generation (aborts an in-flight boot restore)', async () => {
		const g = sessionGeneration();
		await stopSessionTracking();
		expect(sessionGeneration()).toBe(g + 1);
	});

	it('re-arming for a different universe writes to the new root only', async () => {
		startSessionTracking('E:/U/A');
		openTabs.set([tab('a', '/libA/a.md')]);
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
		await stopSessionTracking();
		openTabs.set([tab('b', '/libB/b.md')]);
		startSessionTracking('E:/U/B', sessionSignature(captureSessionSnapshot()));
		openTabs.update((ts) => ts.map((t) => ({ ...t, pinned: true })));
		await vi.advanceTimersByTimeAsync(SESSION_DEBOUNCE_MS + 10);
		const s = saves();
		expect(s[0][1].universeRoot).toBe('E:/U/A');
		expect(s.at(-1)![1].universeRoot).toBe('E:/U/B');
		expect(s.at(-1)![1].session?.tabs[0].path).toBe('/libB/b.md');
	});

	it('deleteSessionOnDisk sends the null tombstone to the explicit root', async () => {
		await deleteSessionOnDisk('E:/U/A');
		const s = saves();
		expect(s).toHaveLength(1);
		expect(s[0][1]).toEqual({ universeRoot: 'E:/U/A', session: null });
	});
});

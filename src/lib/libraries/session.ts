/**
 * MIG-100 — Auto-restore tabs on relaunch: the session tracker.
 *
 * Watches the tab-arrangement stores (openTabs / activeTabId / splitActive /
 * splitDirection) and persists a small disposable snapshot to the universe's
 * `.constellation/session.json` ~1s after the arrangement changes. The boot
 * restore (§3) reads it back. Deliberately DECOUPLED from the user's named
 * workspaces — different file, different lifecycle: this one is machine-
 * written and disposable; workspaces.json is precious.
 *
 * Safety contract (the MIG-100 Architect corrections):
 * - The universe ROOT is captured at arm time and passed EXPLICITLY on every
 *   IPC — never resolved from the ambient active-universe pointer, which
 *   flips BEFORE the frontend switch handler runs (UniverseManager awaits
 *   set_active_universe first). Universe A's session can never land in B's file.
 * - Rule 8: the signature covers paths + pinned + active + split only. Typing
 *   mutates noteModel — and even the OpenTab.content sync is excluded here —
 *   so a keystroke can never schedule a write.
 * - The tracker is armed only AFTER the boot restore resolves (or is skipped),
 *   seeded with the restored arrangement's signature. While `openTabs` is
 *   still boot-empty there is no subscription — an empty snapshot can never
 *   clobber the last session (the empty-overwrite race is structural, not
 *   guarded by timing).
 * - `persistSessionNow()` captures the snapshot SYNCHRONOUSLY at call time
 *   (a store clear one tick later can't change what gets written), is
 *   signature-guarded, and serializes all writes through one in-flight
 *   promise. A failed write keeps a dirty flag and retries on the next
 *   mutation — never silently dropped.
 * - `stopSessionTracking()` cancel-AND-flushes the pending debounce to the
 *   captured root (a stray timer can never fire after a universe switch),
 *   then bumps the generation token that aborts an in-flight boot restore.
 */
import { get, type Unsubscriber } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { subscribeSkipInitial, normalizePathKey } from '$lib/utils';
import {
	openTabs,
	activeTabId,
	splitActive,
	splitDirection,
	type SplitDirection,
} from '$lib/libraries/store';

export interface SessionTab {
	path: string;
	libraryName: string;
	libraryColor: string;
	pinned?: boolean;
	/** One-boot grace (hotfix-inspection LOW): set on a tab whose file was
	 *  unreadable at restore — it stays in persisted snapshots this session
	 *  so a TRANSIENT failure (AV lock, cloud placeholder, drive hiccup)
	 *  isn't pruned; a carried tab that fails a SECOND boot is dropped. */
	carried?: boolean;
}

/** The persisted shape (`.constellation/session.json`). Paths + arrangement
 *  metadata only — never note content (File-Over-App). Unknown `version` at
 *  read time means "no session", never an error. */
export interface SessionSnapshot {
	version: 1;
	savedAt: number;
	tabs: SessionTab[];
	activeTabPath: string | null;
	splitActive: boolean;
	splitDir: SplitDirection;
}

export const SESSION_DEBOUNCE_MS = 1000;

let trackedRoot: string | null = null; // captured at arm time — never ambient
let lastWrittenSig: string | null = null;
/** Tabs unreadable at THIS boot's restore, carried into persisted snapshots
 *  for one boot of grace (see SessionTab.carried). Reset per restore/stop. */
let carriedTabs: SessionTab[] = [];
let dirtyRetry = false; // a failed write → retry on the next mutation
let timer: ReturnType<typeof setTimeout> | null = null;
let unsubs: Unsubscriber[] = [];
let inFlight: Promise<void> = Promise.resolve();
let generation = 0;

/** The restore generation. The boot restore captures this before its awaits
 *  and re-checks it before committing tabs — a universe switch mid-restore
 *  bumps it (via stopSessionTracking) and the stale restore aborts. */
export function sessionGeneration(): number {
	return generation;
}

export function isSessionTracking(): boolean {
	return trackedRoot !== null;
}

/** Snapshot the current tab arrangement. Empty tabs (no file) are excluded —
 *  there is nothing to reopen. */
export function captureSessionSnapshot(): SessionSnapshot {
	const all = get(openTabs);
	const activeId = get(activeTabId);
	const active = all.find((t) => t.id === activeId);
	const openPaths = new Set(all.map((t) => t.path));
	return {
		version: 1,
		savedAt: Date.now(),
		tabs: [
			...all
				.filter((t) => t.path)
				.map((t) => ({
					path: t.path,
					libraryName: t.libraryName,
					libraryColor: t.libraryColor,
					...(t.pinned ? { pinned: true as const } : {}),
				})),
			// One-boot grace: transiently-unreadable tabs stay in the snapshot
			// (a manual reopen supersedes the carried entry).
			...carriedTabs.filter((c) => !openPaths.has(c.path)),
		],
		activeTabPath: active?.path ?? null,
		splitActive: get(splitActive),
		splitDir: get(splitDirection),
	};
}

/** The equality signature two snapshots are compared by: arrangement only —
 *  `savedAt` (and everything not in the snapshot: content, cursor, scroll)
 *  excluded, so only a real arrangement change schedules a write. */
export function sessionSignature(snap: SessionSnapshot): string {
	return JSON.stringify([snap.tabs, snap.activeTabPath, snap.splitActive, snap.splitDir]);
}

function schedulePersist(): void {
	if (trackedRoot === null) return;
	if (sessionSignature(captureSessionSnapshot()) === lastWrittenSig && !dirtyRetry) return;
	if (timer !== null) clearTimeout(timer);
	timer = setTimeout(() => {
		timer = null;
		void persistSessionNow();
	}, SESSION_DEBOUNCE_MS);
}

/** Persist the CURRENT arrangement immediately (debounce cancelled). The
 *  snapshot is captured synchronously — callers may clear the stores right
 *  after (universe switch) without affecting what gets written. Serialized:
 *  concurrent calls write in order, last state wins on disk. */
export function persistSessionNow(): Promise<void> {
	const root = trackedRoot;
	if (root === null) return inFlight;
	if (timer !== null) {
		clearTimeout(timer);
		timer = null;
	}
	const snap = captureSessionSnapshot();
	const sig = sessionSignature(snap);
	if (sig === lastWrittenSig && !dirtyRetry) return inFlight;
	inFlight = inFlight.then(async () => {
		try {
			await invoke('save_universe_session', { universeRoot: root, session: snap });
			lastWrittenSig = sig;
			dirtyRetry = false;
		} catch (e) {
			// Retained, not dropped: the next mutation (or stop-flush) retries.
			dirtyRetry = true;
			console.error('[session] persist failed (will retry on next change):', e);
		}
	});
	return inFlight;
}

/** Arm the tracker for `universeRoot`. `seedSignature` is the signature of
 *  the arrangement that was just restored (or skipped-as-current) — the
 *  synchronous first fire of each subscription then no-ops instead of
 *  rewriting an identical snapshot. */
export function startSessionTracking(universeRoot: string, seedSignature?: string | null): void {
	if (trackedRoot !== null) void stopSessionTracking();
	trackedRoot = universeRoot;
	lastWrittenSig = seedSignature ?? null;
	dirtyRetry = false;
	unsubs = [
		openTabs.subscribe(schedulePersist),
		activeTabId.subscribe(schedulePersist),
		splitActive.subscribe(schedulePersist),
		splitDirection.subscribe(schedulePersist),
	];
}

/** Disarm: unsubscribe, cancel-and-flush any pending change to the root
 *  captured at arm time, bump the restore generation. Returns the flush
 *  promise so a universe switch can await the old universe's final write. */
export function stopSessionTracking(): Promise<void> {
	generation++;
	for (const u of unsubs) u();
	unsubs = [];
	if (deferredArmUnsub) {
		// A pending 0-of-N / sentinel deferred-arm watcher belongs to the
		// universe being left — detach it (its generation check would make it
		// inert anyway; this just avoids the leaked subscription).
		deferredArmUnsub();
		deferredArmUnsub = null;
	}
	let flushed: Promise<void> = inFlight;
	if (trackedRoot !== null && (timer !== null || dirtyRetry)) {
		flushed = persistSessionNow();
	}
	if (timer !== null) {
		clearTimeout(timer);
		timer = null;
	}
	trackedRoot = null;
	lastWrittenSig = null;
	dirtyRetry = false;
	carriedTabs = []; // grace entries belong to the universe being left
	return flushed;
}

/** Delete the session from disk (both generations) — the toggle-off "stop
 *  remembering" primitive (§6). Explicit root, same as every session IPC. */
export async function deleteSessionOnDisk(universeRoot: string): Promise<void> {
	await invoke('save_universe_session', { universeRoot, session: null });
}

// ─── §3 — the boot restore ───

const RESTORE_SENTINEL_KEY = 'constellation-session-restoring';

let deferredArmUnsub: Unsubscriber | null = null;

/** Arm tracking only on the first USER tab mutation (the 0-of-N /
 *  crash-sentinel outcome: the snapshot on disk is preserved untouched until
 *  the user actually does something with tabs — an unmounted drive at boot
 *  can never wipe a good session). */
function armOnFirstTabMutation(universeRoot: string): void {
	if (deferredArmUnsub) deferredArmUnsub();
	const gen = sessionGeneration();
	const unsub = subscribeSkipInitial(openTabs, () => {
		unsub();
		deferredArmUnsub = null;
		if (sessionGeneration() !== gen) return; // universe switched meanwhile
		startSessionTracking(universeRoot, null);
	});
	deferredArmUnsub = unsub;
}

/**
 * The ONE toggle-lifecycle primitive (Settings and any future entry point):
 * ON arms the live tracker; OFF stops it AND deletes the stored session —
 * "off means stop remembering" — re-arming if the delete fails so the live
 * state always matches the retained setting. Throws on failure; the caller
 * owns reverting its UI.
 */
export async function setSessionEnabled(universeRoot: string, enabled: boolean): Promise<void> {
	if (enabled) {
		if (!isSessionTracking()) startSessionTracking(universeRoot, null);
		return;
	}
	await stopSessionTracking();
	try {
		await deleteSessionOnDisk(universeRoot);
	} catch (e) {
		// Deletion failed (file locked) — re-arm so the live state still
		// matches the setting the caller will retain, then surface.
		startSessionTracking(universeRoot, null);
		throw e;
	}
}

function validateSnapshot(raw: unknown): SessionSnapshot | null {
	if (!raw || typeof raw !== 'object') return null;
	const s = raw as Record<string, unknown>;
	// Unknown version = no session, never an error (forward compatibility).
	if (s.version !== 1 || !Array.isArray(s.tabs)) return null;
	const tabs: SessionTab[] = [];
	for (const t of s.tabs) {
		if (!t || typeof t !== 'object') continue;
		const tt = t as Record<string, unknown>;
		if (typeof tt.path !== 'string' || !tt.path) continue;
		tabs.push({
			path: tt.path,
			libraryName: typeof tt.libraryName === 'string' ? tt.libraryName : '',
			libraryColor: typeof tt.libraryColor === 'string' ? tt.libraryColor : '#7c3aed',
			...(tt.pinned === true ? { pinned: true as const } : {}),
			...(tt.carried === true ? { carried: true as const } : {}),
		});
	}
	return {
		version: 1,
		savedAt: typeof s.savedAt === 'number' ? s.savedAt : 0,
		tabs,
		activeTabPath: typeof s.activeTabPath === 'string' ? s.activeTabPath : null,
		splitActive: s.splitActive === true,
		splitDir: s.splitDir === 'horizontal' ? 'horizontal' : 'vertical',
	};
}

export interface RestoreGates {
	/** appSettings.restoreTabsOnRelaunch — OFF means no restore AND no tracking. */
	enabled: boolean;
	/** appSettings.safeBootMode — skip restore AND don't arm (arming over the
	 *  empty boot state would overwrite the snapshot the user may want back). */
	safeBootMode: boolean;
	/** The universe root the bundle payload was READ FROM (bundle.session_root).
	 *  The active universe can flip between the bundle read and this restore
	 *  (the 2026-07-11 Scratch incident); a payload whose origin differs from
	 *  `universeRoot` is DISCARDED and the correct root is re-read instead. */
	bundleRoot?: string | null;
}

/** Same-universe comparison: separators, case, and trailing slashes vary
 *  between the Rust-resolved root and getActiveUniversePath. */
function sameRoot(a: string, b: string): boolean {
	const n = (p: string) => normalizePathKey(p).replace(/\/+$/, '');
	return n(a) === n(b);
}

/**
 * MIG-100 §3 — the boot restore + tracker arming, one call, fire-and-forget
 * from initializeApp AFTER boot:hydrated (zero awaited work on the boot path).
 *
 * Ordering contract: the tracker is armed in `finally` — a failed restore can
 * never leave persistence silently dead (journal-marked instead), and it is
 * never armed while a universe switch superseded this restore (generation
 * check). Gate order: toggle → safeBootMode → crash sentinel → validate →
 * batch restore (journal-bracketed) → arm.
 */
export async function restoreSessionThenTrack(
	bundleSession: unknown,
	universeRoot: string,
	gates: RestoreGates,
): Promise<void> {
	const gen = sessionGeneration();
	const journal = (surface: string, detail: string) =>
		invoke('journal_frontend_marker', { surface, detail }).catch(() => {});

	if (!gates.enabled) return;
	if (gates.safeBootMode) {
		void journal('session_restore_skipped', 'safe_boot_mode');
		return;
	}

	let deferArm = false;
	try {
		// Crash-loop breaker: a sentinel still present means the LAST restore
		// never finished (webview death mid-restore — possibly caused by a
		// pathological restored note). Skip ONCE; the sentinel clears so the
		// next boot tries again, and the toggle stays reachable meanwhile.
		let sentinel: string | null = null;
		try {
			sentinel = localStorage.getItem(RESTORE_SENTINEL_KEY);
		} catch { /* no localStorage (harness) → no sentinel */ }
		if (sentinel) {
			try { localStorage.removeItem(RESTORE_SENTINEL_KEY); } catch { /* ignore */ }
			void journal('session_restore_skipped', 'crash_sentinel');
			deferArm = true; // preserve the snapshot until the user acts
			return;
		}

		let raw = bundleSession;
		// Origin check (Boss Stage-2 failure 4): a payload read from another
		// universe must never restore here — discard it and fall through to
		// the direct re-read of THIS root.
		if (raw !== undefined && raw !== null && gates.bundleRoot && !sameRoot(gates.bundleRoot, universeRoot)) {
			void journal('session_restore_payload_mismatch', `${gates.bundleRoot} ≠ ${universeRoot}`);
			raw = null;
		}
		if (raw === undefined || raw === null) {
			// Bundle-failure fallback: read directly. An Err degrades to null →
			// no restore, but tracking still arms (deliberate: a transient read
			// error must not kill persistence for the whole session — the first
			// real arrangement change rewrites a current snapshot anyway).
			try {
				raw = await invoke('read_universe_session', { universeRoot });
			} catch {
				raw = null;
			}
		}
		const snap = validateSnapshot(raw);
		if (!snap || snap.tabs.length === 0) return; // nothing to restore

		try { localStorage.setItem(RESTORE_SENTINEL_KEY, String(snap.savedAt)); } catch { /* ignore */ }
		void journal('session_restore_begin', `${snap.tabs.length} tabs`);
		const { restoreSessionTabs } = await import('$lib/libraries/store');
		const result = await restoreSessionTabs(snap, () => sessionGeneration() === gen);
		try { localStorage.removeItem(RESTORE_SENTINEL_KEY); } catch { /* ignore */ }
		if (result.aborted) {
			void journal('session_restore_aborted', 'superseded_by_switch');
			return; // gen changed — finally's check skips arming; the new universe arms itself
		}
		void journal('session_restore_end', `${result.restored}/${result.requested} restored`);
		if (result.requested > 0 && result.restored === 0) {
			// 0-of-N (the unmounted-drive shape): restore FAILED. Preserve the
			// file; don't arm until the user actually mutates tabs.
			void journal('session_restore_failed', `0/${result.requested} — arm deferred`);
			deferArm = true;
			return;
		}
		// Partial restore: carry each FIRST-strike unreadable tab into this
		// session's snapshots (transient failures survive one boot); a tab
		// already carried — a second strike — is dropped for good.
		carriedTabs = result.skipped
			.filter((t) => !t.carried)
			.map((t) => ({
				path: t.path,
				libraryName: t.libraryName,
				libraryColor: t.libraryColor,
				...(t.pinned ? { pinned: true as const } : {}),
				carried: true as const,
			}));
		if (carriedTabs.length > 0) {
			void journal('session_restore_carried', `${carriedTabs.length} unreadable tab(s) on one-boot grace`);
		}
	} catch (e) {
		try { localStorage.removeItem(RESTORE_SENTINEL_KEY); } catch { /* ignore */ }
		void journal('session_restore_error', String(e));
		// fall through — the finally still arms (never a silently dead tracker)
	} finally {
		if (sessionGeneration() === gen) {
			if (deferArm) {
				armOnFirstTabMutation(universeRoot);
			} else {
				startSessionTracking(universeRoot, sessionSignature(captureSessionSnapshot()));
			}
		}
	}
}

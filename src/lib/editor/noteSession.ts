/**
 * noteSession — MIG-076 §C: the behavior layer over noteModel.
 *
 * The thin, TESTABLE glue the editor components call for every content-flow
 * event. The components hold NO content logic — they translate user / CM6
 * events into these calls and render what the model reports. This is the
 * structural reason the §CB attempts failed and this one won't be allowed to:
 * the bugs lived in glue tangled inside .svelte lifecycle, invisible to any
 * test. Pulling the glue into a plain module drags it into the light, where a
 * headless harness drives every failure recipe through the REAL code path the
 * components will use (tests/mig-076/runtimeHarness.test.ts).
 *
 * Disk writes go through an injected `DiskWriter`, so the harness uses an
 * in-memory fake disk and the live app injects the real WriteGate (`writeNote`).
 * The component layer that calls these stays a thin view — that thin wiring is
 * the only thing the Boss test still has to confirm; the logic is proven here.
 */
import * as M from './noteModel';
import { bumpProps } from './propsSignal';
import type { Text } from '@codemirror/state';
import type { FrontmatterProperty } from '$lib/libraries/store';
import type { SetPropOpts } from './propRow';

export type DiskWriter = (path: string, content: string, origin: string) => void | Promise<void>;
export type SaveResult = M.ComposeResult;

/**
 * The full durability contract for a save (Save-Durability migration, 2026-07-08).
 * A save may only mark the model clean AFTER the disk write is proven durable, and
 * a failed write must (a) NOT mark clean (model stays dirty → retried), (b) RETAIN
 * a recovery net (never a silent loss), and (c) SURFACE the error (never swallowed).
 * All side effects are injected so the headless harness drives the REAL path.
 *   - `setNet`      — write-ahead buffer set BEFORE the write (retained on failure).
 *   - `clearNetIf`  — compare-and-clear: only wipe the net if what we wrote is still
 *                     the buffered content (a newer edit's net is never clobbered).
 *   - `onSuccess`   — reindex / embed / broadcast — runs ONLY on a durable write.
 *   - `onError`     — surface the failure (save-health banner) — never silent.
 */
export interface SaveEnv {
	write: DiskWriter;
	setNet?: (path: string, content: string, cursorPos: number, scrollTop: number) => void;
	clearNetIf?: (path: string, content: string) => void;
	onSuccess?: (info: { path: string; content: string; version: number }) => void;
	onError?: (info: { path: string; content: string; version: number; error: unknown }) => void;
	cursorPos?: number;
	scrollTop?: number;
}

/** compose refusal | write failure | success. `ok:false` covers both non-durable outcomes. */
export type SaveOutcome =
	| M.ComposeResult
	| { ok: false; reason: 'write_failed'; path: string; version: number; error: unknown };

/** Open a note's session from its on-disk content (tab open / nav / reload). */
export function open(id: string, path: string, diskContent: string): void {
	M.openModel(id, path, diskContent);
	bumpProps(); // MIG-107 — a freshly-seeded model has new props; panels must look again
}

/**
 * Create the model if absent, or re-seed it when the host has demonstrably
 * moved this session id to a DIFFERENT note (path changed) without going
 * through open(). Hosts that never call open() — index preview, dashboard,
 * a second-screen NoteEditor — get a correct model here. An existing model on
 * the SAME path is left untouched, so its live edits always win.
 */
export function ensure(id: string, path: string, diskContent: string): void {
	const m = M.getModel(id);
	if (m && m.path === path) return;
	M.openModel(id, path, diskContent);
	bumpProps(); // MIG-107 — see `open`
}

/** Any editor view (NotePane OR FocusPane) → the one model. Accepts the CM6
 *  `Text` rope (O(1), the keystroke hot path) or a plain string. `expectPath`
 *  (the caller's target note) identity-guards the write: a stale caller
 *  addressing a repurposed model is rejected, never poisons it. */
export function editBody(id: string, text: string | Text, expectPath?: string): void {
	M.setBody(id, text, expectPath);
}

/**
 * Property editor → the one model, WHOLESALE. `expectPath` identity-guards the write (see editBody).
 *
 * ⚠ MIG-107: a wholesale replace is only correct for a caller that genuinely means "replace
 * everything" and has just read the model. For "change one property", use the intents below —
 * a whole array assembled from a stale projection is what silently deletes another writer's
 * frontmatter (§3.5.1).
 */
export function editProps(id: string, props: FrontmatterProperty[], expectPath?: string): void {
	M.setProps(id, props, expectPath);
	bumpProps();
}

// ─── MIG-107 Slice 2 — the property INTENTS ─────────────────────────────────────────────────────
//
// Thin wrappers whose only job beyond the model call is to ANNOUNCE. The announcement lives here
// rather than inside `noteModel` because that module is deliberately non-reactive (§C-2), so the
// set of call sites that tick the signal has to stay explicit and reviewable — see `propsSignal.ts`.
//
// Each returns whether the model actually changed, so a caller can skip a pointless save, and the
// signal is ticked ONLY on a real change — an announcement for a no-op would wake every panel for
// nothing.

/** Set ONE property's value. */
export function editPropValue(
	id: string,
	key: string,
	value: string,
	opts?: SetPropOpts,
	expectPath?: string,
): boolean {
	const changed = M.setPropValue(id, key, value, opts, expectPath);
	if (changed) bumpProps();
	return changed;
}

/** Add a NEW property. Refuses an empty key, and refuses to overwrite an existing one. */
export function addPropTo(id: string, prop: FrontmatterProperty, expectPath?: string): boolean {
	const changed = M.addProp(id, prop, expectPath);
	if (changed) bumpProps();
	return changed;
}

/** Remove ONE property. */
export function removePropFrom(id: string, key: string, expectPath?: string): boolean {
	const changed = M.removeProp(id, key, expectPath);
	if (changed) bumpProps();
	return changed;
}

/** Rename a property's key. Returns false on a collision — the caller surfaces it to the user. */
export function renamePropKeyIn(id: string, oldKey: string, newKey: string, expectPath?: string): boolean {
	const changed = M.renamePropKey(id, oldKey, newKey, expectPath);
	if (changed) bumpProps();
	return changed;
}

/** Move a property before another (or to the end when `beforeKey` is null). */
export function reorderPropsIn(id: string, key: string, beforeKey: string | null, expectPath?: string): boolean {
	const changed = M.reorderProps(id, key, beforeKey, expectPath);
	if (changed) bumpProps();
	return changed;
}

/** PJ-088 — replace the model's whole content (frontmatter + body) from a merged source, re-basing
 *  so compose emits it verbatim. `expectPath` identity-guards the write (see editBody). */
export function replaceContent(id: string, content: string, expectPath?: string): void {
	M.replaceContent(id, content, expectPath);
	bumpProps(); // MIG-107 — a merge rebase replaces props wholesale (legitimately); panels re-read
}

/**
 * The body a view must SEED from when it mounts — ALWAYS the model, never a
 * stale tab copy. This is the symptom-1 fix made explicit and testable: it
 * cannot return an empty/old body for a note that has current content, no
 * matter when the view's branch renders relative to the old view's teardown.
 */
export function bodyForView(id: string): string {
	return M.getModel(id)?.body.toString() ?? '';
}

/**
 * Seed a view safely: the model's body when a model for THIS id exists AND it
 * holds THIS path; otherwise the caller's fallback (the host's own content).
 * The path guard means a not-yet-ensured or repurposed slot falls back to the
 * host content instead of seeding another note's body — robust at every mount
 * timing (the model may be created by an $effect that runs after first render).
 */
export function seedBody(id: string, path: string, fallback: string): string {
	const m = M.getModel(id);
	return m && m.path === path ? m.body.toString() : fallback;
}

/**
 * Save: compose from THIS id for THIS path (REFUSES a mismatch — the identity
 * guard that kills the cross-note write), then run the durability contract:
 * net-before-write → await write → ON SUCCESS mark clean + compare-and-clear net
 * + onSuccess; ON FAILURE surface via onError, RETAIN the net, and leave the model
 * DIRTY (never a false-clean; the next debounce/retry re-attempts). Never throws on
 * a write failure — returns `{write_failed}` so callers proceed without an unhandled
 * rejection (the failure is already netted + surfaced). The caller never assembles
 * content itself. `env` may be a bare `DiskWriter` (back-compat: the headless harness
 * and any minimal caller) or the full `SaveEnv` (production sites, with net + surface).
 */
/**
 * PJ-103 adversarial-review fix (2026-07-16) — per-id in-flight serialization.
 * Two concurrent save() calls for the SAME model (the app-close flush racing
 * the 1500ms debounced editor_save; a double-X close re-pass) must never have
 * two writes of DIFFERENT versions in flight at once: IPC arrival order is not
 * contractually FIFO, so an older body could land on disk LAST while
 * markSaved(newer) reports clean and the net compare-and-clears — the newest
 * keystrokes would then exist nowhere. Chaining makes each save COMPOSE only
 * after the prior write settles, so the newest content always writes last.
 * The chain entry is deleted when its tail settles (no unbounded growth); the
 * chained runner never rejects for write failures (save returns {ok:false}),
 * and both settle-branches proceed so a rejected predecessor can't wedge the
 * chain.
 */
const saveChains = new Map<string, Promise<unknown>>();

export function save(
	id: string,
	expectPath: string,
	env: DiskWriter | SaveEnv,
	origin = 'editor_save',
): Promise<SaveOutcome> {
	const e: SaveEnv = typeof env === 'function' ? { write: env } : env;
	const prev = saveChains.get(id);
	let run: Promise<SaveOutcome>;
	if (prev) {
		// CHAINED: compose only after the predecessor settles (newest-last).
		// But the crash-net stash must NOT wait — a beforeunload flush issued
		// while a debounced save is in flight would otherwise never stash if
		// the webview dies before the predecessor settles. Stash the CURRENT
		// composition eagerly; the chained write's own setNet (same-or-newer
		// content) overwrites it, and the predecessor's compare-and-clear
		// can only remove it when the contents are identical (nothing lost).
		const eager = M.compose(id, expectPath);
		if (eager.ok) e.setNet?.(eager.path, eager.content, e.cursorPos ?? 0, e.scrollTop ?? 0);
		run = prev.then(
			() => saveUnchained(id, expectPath, e, origin),
			() => saveUnchained(id, expectPath, e, origin),
		);
	} else {
		// FAST PATH (no in-flight save for this id): run directly — an async
		// fn executes synchronously to its first await, preserving the
		// original contract that compose + setNet complete BEFORE the first
		// await (the beforeunload sync-stash guarantee, and the recipes'
		// type-during-await version semantics).
		run = saveUnchained(id, expectPath, e, origin);
	}
	const tail = run.then(() => undefined, () => undefined);
	saveChains.set(id, tail);
	void tail.then(() => {
		if (saveChains.get(id) === tail) saveChains.delete(id);
	});
	return run;
}

async function saveUnchained(
	id: string,
	expectPath: string,
	e: SaveEnv,
	origin: string,
): Promise<SaveOutcome> {
	const r = M.compose(id, expectPath);
	if (!r.ok) return r; // identity refusal (path_mismatch) — nothing composed, nothing written
	// NET BEFORE THE WRITE — a failed/interrupted write is recoverable from the buffer.
	e.setNet?.(r.path, r.content, e.cursorPos ?? 0, e.scrollTop ?? 0);
	try {
		await e.write(r.path, r.content, origin);
	} catch (error) {
		// SURFACE (never silent) + net RETAINED + model NOT marked clean (stays dirty → retried).
		e.onError?.({ path: r.path, content: r.content, version: r.version, error });
		return { ok: false, reason: 'write_failed', path: r.path, version: r.version, error };
	}
	M.markSaved(id, r.version, r.path); // clean trails durability — path-guarded so a save that resolves after an id-swap can't poison the new model (APP-KILLER #2)
	M.noteDiskSynced(id, r.content, r.path); // PJ-070 — re-baseline: the model now knows the exact on-disk bytes (path-guarded, same reason as markSaved)
	e.clearNetIf?.(r.path, r.content); // compare-and-clear: never wipe a newer edit's net
	e.onSuccess?.({ path: r.path, content: r.content, version: r.version });
	return r;
}

/** flushIfDirty outcome. `ok:true` = safe to proceed with the nav/replace; `ok:false` =
 *  ABORT (keep the user on the outgoing note) — a durable write could not be proven. */
export type FlushResult = { ok: true } | { ok: false; reason: string; error?: unknown };

/**
 * APP-KILLER #2 (2026-07-08) — the ONE nav-flush choke point. Before the store re-seeds a
 * tab's id-slot with the next note (openNoteModel), it flushes the OUTGOING dirty model to
 * disk through the shipped durability gate (`save`), so a navigation (a DEPARTURE) never
 * discards in-debounce edits. The old path is read FROM THE MODEL (never the tab — the caller
 * has already rewritten tab.path to the new note by the time it replaces the model).
 *   - no model / not dirty → {ok:true} immediately (no spurious write) → proceed.
 *   - write failure → {ok:false} → ABORT: the caller keeps the user on the note; the net is
 *     retained and the save-health banner is surfaced by `save`'s onError (never a silent loss).
 *   - still dirty after a bounded loop → {ok:false, still_dirty} → ABORT.
 * The bounded loop re-composes each pass, so a keystroke that lands during the awaited write
 * (the await-window race) is caught by the next flush instead of being lost to the swap.
 */
export async function flushIfDirty(
	id: string,
	env: DiskWriter | SaveEnv,
	origin = 'nav_flush',
): Promise<FlushResult> {
	const m = M.getModel(id);
	if (!m || !M.isDirty(id)) return { ok: true }; // nothing to flush → safe to proceed
	const oldPath = m.path; // the model's OWN identity (setBody never moves it; only rename does)
	const MAX = 4;
	for (let i = 0; i < MAX && M.isDirty(id); i++) {
		const r = await save(id, oldPath, env, origin);
		if (!r.ok) return { ok: false, reason: r.reason, error: (r as { error?: unknown }).error };
	}
	if (M.isDirty(id)) return { ok: false, reason: 'still_dirty' };
	return { ok: true };
}

/** Rename / move: update identity; the next save targets the new path. */
export function repath(id: string, newPath: string): void {
	M.setPath(id, newPath);
}

/** PJ-102b — a model seeded from the write-ahead NET is truthfully DIRTY with the
 *  ACTUAL disk bytes as its baseline (see noteModel.markRecoveredFromNet). */
export function recoveredFromNet(id: string, trueDiskContent: string | null): void {
	M.markRecoveredFromNet(id, trueDiskContent);
}

/** PJ-102b (restore half) — truth-set a clean model's disk baseline (see noteModel). */
export function setDiskBaseline(id: string, trueDiskContent: string): void {
	M.setDiskBaseline(id, trueDiskContent);
}

/** External change (file watcher / second screen) — freshness-gated; returns whether adopted. */
export function externalChange(id: string, diskContent: string, expectPath?: string): boolean {
	const adopted = M.adoptDisk(id, diskContent, expectPath);
	// MIG-107 — an adopted external change rewrites props wholesale (legitimately, and it re-bases).
	// Ticked ONLY when it actually adopted: adoptDisk returns false for an echo or a dirty refusal,
	// and announcing those would wake every panel for a change that did not happen.
	if (adopted) bumpProps();
	return adopted;
}

/** Unsaved edits beyond the last persisted version. */
export function isDirty(id: string): boolean {
	return M.isDirty(id);
}

/** Tab / session close. */
export function close(id: string): void {
	M.closeModel(id);
}

/** Universe switch / restart / workspace restore. */
export function closeAll(): void {
	M.clearAllModels();
}

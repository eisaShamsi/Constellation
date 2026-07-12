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
import type { Text } from '@codemirror/state';
import type { FrontmatterProperty } from '$lib/libraries/store';

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
}

/** Any editor view (NotePane OR FocusPane) → the one model. Accepts the CM6
 *  `Text` rope (O(1), the keystroke hot path) or a plain string. `expectPath`
 *  (the caller's target note) identity-guards the write: a stale caller
 *  addressing a repurposed model is rejected, never poisons it. */
export function editBody(id: string, text: string | Text, expectPath?: string): void {
	M.setBody(id, text, expectPath);
}

/** Property editor → the one model. `expectPath` identity-guards the write (see editBody). */
export function editProps(id: string, props: FrontmatterProperty[], expectPath?: string): void {
	M.setProps(id, props, expectPath);
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
export async function save(
	id: string,
	expectPath: string,
	env: DiskWriter | SaveEnv,
	origin = 'editor_save',
): Promise<SaveOutcome> {
	const e: SaveEnv = typeof env === 'function' ? { write: env } : env;
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

/** External change (file watcher / second screen) — freshness-gated; returns whether adopted. */
export function externalChange(id: string, diskContent: string): boolean {
	return M.adoptDisk(id, diskContent);
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

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
import type { FrontmatterProperty } from '$lib/libraries/store';

export type DiskWriter = (path: string, content: string, origin: string) => void | Promise<void>;
export type SaveResult = M.ComposeResult;

/** Open a note's session from its on-disk content (tab open / nav / reload). */
export function open(id: string, path: string, diskContent: string): void {
	M.openModel(id, path, diskContent);
}

/** Any editor view (NotePane OR FocusPane) → the one model. */
export function editBody(id: string, text: string): void {
	M.setBody(id, text);
}

/** Property editor → the one model. */
export function editProps(id: string, props: FrontmatterProperty[]): void {
	M.setProps(id, props);
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
 * Save: compose from THIS id for THIS path (REFUSES a mismatch — the identity
 * guard that kills the cross-note write), write through the injected writer,
 * record the saved version. Returns the compose result so the caller can
 * journal a refusal. The caller never assembles content itself.
 */
export async function save(
	id: string,
	expectPath: string,
	write: DiskWriter,
	origin = 'editor_save',
): Promise<SaveResult> {
	const r = M.compose(id, expectPath);
	if (!r.ok) return r;
	await write(r.path, r.content, origin);
	M.markSaved(id, r.version);
	return r;
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

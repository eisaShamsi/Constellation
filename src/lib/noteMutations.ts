// MIG-096 §1 — the refresh-after-mutate broadcast (the note-lists right-click
// cluster's central contract). The full right-click menu lets a note be
// renamed / moved / deleted from ANY of the ~26 note-list surfaces; those lists
// are results of a computation that never re-runs, so a mutation would leave
// them stale or dangling (the "non-refreshing surface" hazard the Search results
// and IndexPanel already document). This module is the fix: the gated write path
// announces every rename/move/delete once, and every list subscribes to keep
// itself current.
//
// Transport: Tauri `emit()` reaches ALL windows, so second-screen companions
// hear the same broadcast for free (Display-not-Domain — they never re-write).
//
// CASCADE-SAFE ORDERING (MIG-096 invariant 2 / BUG-023): `note-renamed` is
// emitted by the RENAME HANDLER only after the universe-wide wikilink cascade
// has fully settled — NEVER from inside `renameItem`. A list that re-runs its
// IPC on this event must therefore never see a half-rewritten universe. Move and
// delete have no cascade; they fire on handler resolve. Batch loops emit their
// granular events once at the tail (after the single heavy tree refresh), so the
// only storm-able cost — a list re-running its query — is coalesced by
// `onAnyChange` below.

import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';

/** A rename settled. `oldPath`→`newPath` may be equal for canonical-filename
 *  notes (rename rewrites the frontmatter title in place, filename stable) —
 *  hence `newName` is always carried so a splice/re-title subscriber can update
 *  the visible title even when the path did not change. */
export interface NoteRenamedEvent {
	oldPath: string;
	newPath: string;
	/** The new human display title. */
	newName: string;
}
export interface NoteMovedEvent {
	oldPath: string;
	newPath: string;
}
export interface NoteDeletedEvent {
	path: string;
}

export const NOTE_RENAMED = 'note-renamed';
export const NOTE_MOVED = 'note-moved';
export const NOTE_DELETED = 'note-deleted';

// Fire-and-forget: a missing listener is a no-op, and the mutation itself has
// already succeeded through the gated wrapper — a failed emit must never surface
// as a mutation failure.
export function emitNoteRenamed(e: NoteRenamedEvent): void {
	emit(NOTE_RENAMED, e).catch(() => {});
}
export function emitNoteMoved(e: NoteMovedEvent): void {
	emit(NOTE_MOVED, e).catch(() => {});
}
export function emitNoteDeleted(e: NoteDeletedEvent): void {
	emit(NOTE_DELETED, e).catch(() => {});
}

export interface NoteMutationHandlers {
	/** Fires immediately per event — for cheap, precise splice / re-title. */
	onRenamed?: (e: NoteRenamedEvent) => void;
	onMoved?: (e: NoteMovedEvent) => void;
	onDeleted?: (e: NoteDeletedEvent) => void;
	/** Coalesced (300 ms) "something changed" — for surfaces that answer a
	 *  membership question and must RE-RUN their own IPC (tag / reviewer / search
	 *  / tension lists, where a rename can change WHICH rows belong). Fires once
	 *  after a burst so a batch of N mutations triggers exactly one re-run. */
	onAnyChange?: () => void;
}

/**
 * Subscribe to the note-mutation broadcast. Returns an unlisten function to call
 * in `onDestroy` (Rule 4 — no leaks: it clears the coalescing timer AND drops
 * all three Tauri listeners). Granular callbacks fire immediately; `onAnyChange`
 * is debounced 300 ms.
 */
export async function onNoteMutation(h: NoteMutationHandlers): Promise<UnlistenFn> {
	let anyTimer: ReturnType<typeof setTimeout> | null = null;
	const fireAny = () => {
		if (!h.onAnyChange) return;
		if (anyTimer) clearTimeout(anyTimer);
		anyTimer = setTimeout(() => {
			anyTimer = null;
			h.onAnyChange!();
		}, 300);
	};
	// Push each listener into the set AS it resolves — never an array literal that
	// aborts atomically: if the 2nd/3rd listen() rejects (transient webview IPC),
	// the array literal would drop the already-registered listener on the floor,
	// leaking it for the window's lifetime (Rule 4). Instead, on any failure we
	// unwind the ones that DID register, then re-throw so the caller knows.
	const unlisteners: UnlistenFn[] = [];
	const cleanup: UnlistenFn = () => {
		if (anyTimer) clearTimeout(anyTimer);
		for (const u of unlisteners) u();
	};
	try {
		unlisteners.push(await listen<NoteRenamedEvent>(NOTE_RENAMED, (ev) => { h.onRenamed?.(ev.payload); fireAny(); }));
		unlisteners.push(await listen<NoteMovedEvent>(NOTE_MOVED, (ev) => { h.onMoved?.(ev.payload); fireAny(); }));
		unlisteners.push(await listen<NoteDeletedEvent>(NOTE_DELETED, (ev) => { h.onDeleted?.(ev.payload); fireAny(); }));
	} catch (e) {
		cleanup();
		throw e;
	}
	return cleanup;
}

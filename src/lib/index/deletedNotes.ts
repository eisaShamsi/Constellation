/**
 * PJ-385 — reading the delete archive back.
 *
 * # Concept
 *
 * *When Constellation destroys something permanently, the person must be able to see what it
 * destroyed.*
 *
 * Every delete in this app writes an envelope to `note-history.jsonl` before anything is purged,
 * and refuses to purge if that write fails. That guarantee was true and useless in the same
 * breath: the only reader took a content id the caller had to already know and returned only the
 * change-events — it could not enumerate what had been deleted at all. So the app could say "its
 * history was kept" while offering no way to look at it.
 *
 * The Boss ruled on 2026-08-25, asked whether to proceed with a 603-row removal or build this
 * first: **build this first.** A record nobody can read is not a record.
 *
 * # What this is NOT
 *
 * It is not a restore. Nothing here writes a note back to disk. The archive is a durable account
 * of what was removed — read it, copy from it — and the question of whether Constellation should
 * be able to put a note back is a separate decision with its own write path, collision rules and
 * re-indexing.
 */

import { invoke } from '@tauri-apps/api/core';

/** Mirrors `deleted_notes::DeletedNote` (serde `rename_all = "camelCase"`). */
export interface DeletedNote {
	/** The archive's key. Also what change-events are joined on. */
	cid: string;
	/** Milliseconds since the epoch, recorded at the moment of deletion. */
	at: number;
	path: string;
	name: string;
	library: string;
	/** `trash` | `system_trash` | `permanent` | `vanished` | `reconcile_gone` | `phantom_prune`. */
	reason: string;
	/** Where the file went, when that was knowable. */
	dest: string | null;
	/**
	 * Characters of note text held in the archive. **Zero is a real answer**, not a failure: the
	 * archive keeps what the search index held, which is empty for a note with nothing beyond its
	 * frontmatter. It does NOT mean the note's file was missing — an earlier version of this
	 * comment claimed that, and the measurement is the other way round: 601 of the 603 phantom
	 * rows carry body text (median 18,984 characters across those 601; 18,944 is the median of
	 * all 603, and this line previously attached that figure to the 601).
	 *
	 * Note what this count is OF: a stripped search rendering, not the note. The index stores
	 * `strip_markdown` + Arabic-normalised text, so headings, code fences and `[[ ]]` brackets
	 * are already gone from the number as well as from the text.
	 */
	bodyChars: number;
	/** How many change-history events were archived alongside it. */
	historyEvents: number;
}

/** Mirrors `deleted_notes::DeletedNotesPage`. */
export interface DeletedNotesPage {
	notes: DeletedNote[];
	/** Envelopes in the file, before any limit. */
	total: number;
	/**
	 * Lines the parser could not read. Surfaced rather than swallowed — an archive that is partly
	 * unreadable must never look like a complete one, least of all when it is the last remaining
	 * account of a destroyed note.
	 */
	unreadableLines: number;
	/** False when the archive file does not exist — distinct from "exists and is empty", which is
	 *  a real answer meaning nothing has ever been deleted here. */
	archivePresent: boolean;
}

/** Every deletion this universe has archived, newest first. Reads a file; opens no database. */
export async function loadDeletedNotes(limit?: number): Promise<DeletedNotesPage> {
	// THROWS on failure rather than returning null. A caller that cannot tell "the archive is
	// empty" from "the archive could not be read" will eventually render one as the other, and on
	// this surface that means telling someone nothing was destroyed when the record simply could
	// not be opened (2026-08-25 inspection).
	return await invoke<DeletedNotesPage>('deleted_notes_list', { limit });
}

/**
 * The archived text of one deletion, addressed by content id AND time.
 *
 * Both are needed: one note can have several deletion envelopes (a sync agent removing and
 * re-adding a file archives a `vanished` envelope, and the same note may be deleted again later).
 * Addressed by cid alone, a row could display a DIFFERENT deletion's text as though it were what
 * was destroyed — on the last surviving copy.
 *
 * Returns `{ text: null }` when the envelope genuinely carried no body, and THROWS when the read
 * failed. Those are different facts and the first version returned `null` for both, so an
 * unreadable archive was indistinguishable from an empty entry — the exact confusion this module
 * exists to prevent (2026-08-25 inspection).
 */
export async function loadDeletedNoteBody(cid: string, at: number): Promise<{ text: string | null }> {
	return { text: await invoke<string | null>('deleted_note_body', { cid, at }) };
}

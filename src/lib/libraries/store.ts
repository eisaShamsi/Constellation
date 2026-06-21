/**
 * Library (Universe) state management.
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { normalizePathKey } from '$lib/utils';
import { getLinkTypes, isLinkTypeValue } from './linkTypeRegistry';
// MIG-076 §C — single content ownership. noteModel/noteSession use this
// module's parseFrontmatter/buildFullContent (hoisted fn declarations, so the
// import cycle is eval-safe) and are used here only inside functions (never at
// module init). The model is the save source when SINGLE_OWNERSHIP is on.
import { editProps as editNoteProps, close as closeNoteModel, repath as repathNoteModel, open as openNoteModel, save as saveNoteSession, isDirty as isNoteDirty } from '$lib/editor/noteSession';
import { compose as composeNoteModel, markSaved as markNoteSaved, getModel as getNoteModel } from '$lib/editor/noteModel';
import { SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';
import { setPendingLineJump } from '$lib/editor/lineJump'; // §A.2 — one-shot line jump (CM6-free)
import { toggleTask } from '$lib/tasks/store'; // §A.3 — reconciled task toggle (tasks/store has no store dep → no cycle)

export interface LibraryInfo {
	id: string;
	name: string;
	path: string;
	is_universe_notes?: boolean;
	/** "native" | "canonical" | "compatible" */
	canonical_mode?: string;
}

export interface StarInfo {
	name: string;
	path: string;
	library_id: string;
	library_name: string;
	modified: number;
	preview: string;
}

export interface LibraryStats {
	library_id: string;
	name: string;
	path: string;
	star_count: number;
	folder_count: number;
	recent_stars: StarInfo[];
	is_universe_notes?: boolean;
}

export interface FileEntry {
	name: string;
	path: string;
	is_dir: boolean;
	children: FileEntry[] | null;
	extension: string | null;
	modified: number | null;
	status: string | null;
	isCUniverse?: boolean;
	/** For canonical files: human-readable title from frontmatter. */
	display_title?: string | null;
}

export interface OpenTab {
	id: string;
	path: string;
	content: string;
	libraryName: string;
	libraryPath: string;
	name: string;
	libraryColor: string;
	highlightTerm?: string;
	history: string[];
	historyIndex: number;
	cursorPos?: number;
	scrollTop?: number;
	pinned?: boolean;
	/** §3-redo.4 — incremented by `reloadTabsFromDisk` after the cascade
	 *  rewrites this tab's file. Used in NoteEditor's `{#key}` to force
	 *  NotePane to destroy + remount with fresh disk content. Per Concept
	 *  Paper D6, recreate is the safe primitive — `$effect`-driven
	 *  view.dispatch is forbidden. */
	reloadVersion?: number;
}

export type PropertyType = 'text' | 'number' | 'date' | 'datetime' | 'list' | 'link' | 'checkbox' | 'nested-object-list';

export interface FrontmatterProperty {
	key: string;
	value: string;
	type: PropertyType;
	listItems?: string[];
	/** MIG-022 §A.1 (PJ-041 cluster, 2026-05-11) — for the
	 *  `nested-object-list` type, holds the structured row data.
	 *  Each row is a `{ field: value }` map. Used today by the
	 *  `ikhtilāf` field which stores `[{ school, position }, ...]`;
	 *  generalized so future schema additions (e.g. `ḥadīth_chain`,
	 *  `citation_block`) can reuse the parser + serializer.
	 *
	 *  When this field is present, `value` carries a compact
	 *  one-line summary suitable for legacy display + search
	 *  matching ("Hanafī: permissible | Mālikī: discouraged"); the
	 *  authoritative source-of-truth is `nestedObjects`.
	 */
	nestedObjects?: Array<Record<string, string>>;
}

// Well-known property key sets (English + Arabic)
const LIST_KEYS = new Set([
	'tags', 'aliases', 'cssclasses', 'cssclass', 'related', 'categories', 'group',
	'الوسم', 'وسوم', 'المجموعة', 'ذات صلة', 'أسماء بديلة', 'تصنيفات',
	// MIG-022 §A.1 — gap-analysis §6.1 metadata extensions.
	// `domain` is a list ("[fiqh, photography, overland-travel]") of
	// per-note subject-matter tags. Per the gap analysis, the existing
	// `tags` field is the user's free-form folksonomy; `domain` is the
	// structured discipline/topic field for retrieval.
	'domain',
]);
const CHECKBOX_KEYS = new Set([
	'done', 'completed', 'draft', 'publish', 'published', 'pinned', 'archived', 'starred', 'todo',
	'favorite', 'featured', 'hidden',
	'مكتمل', 'منشور', 'مسودة', 'مثبت', 'مؤرشف', 'مميز', 'مخفي',
]);
const DATE_KEYS = new Set([
	'date', 'created', 'updated', 'modified', 'due', 'start', 'end', 'deadline', 'completed_date',
	'أنشئ', 'حُدث', 'تاريخ', 'تعديل', 'موعد', 'بداية', 'نهاية',
	// MIG-022 §A.1 — gap-analysis §6.1: ISO date of last epistemic
	// state revision. Distinct from `updated`/`modified` (file-system
	// touch); `updated_at` is the user's deliberate stance-revision
	// timestamp.
	'updated_at',
]);
// MIG-022 §A.1 — gap-analysis §6.1: list-of-objects support, primarily
// for `ikhtilāf` (structured scholarly disagreement). Each entry has a
// `school` field + a `position` field. The parser detects these by
// (a) the key matching IKHTILAF_KEYS, AND (b) the next line being an
// indented `- field: value` line. The §A.3 Properties panel renders
// these via the custom ikhtilāf widget per D-A4.α; raw consumers can
// read `nestedObjects` directly.
const IKHTILAF_KEYS = new Set([
	'ikhtilāf', 'ikhtilaf', 'الاختلاف',
]);

/** Normalize DD/MM/YYYY → YYYY-MM-DD for storage */
export function normalizeDateValue(value: string): string {
	const ddmmyyyy = value.match(/^(\d{1,2})\/(\d{1,2})\/(\d{4})$/);
	if (ddmmyyyy) {
		const [, d, m, y] = ddmmyyyy;
		return `${y}-${m.padStart(2, '0')}-${d.padStart(2, '0')}`;
	}
	return value;
}

function detectPropertyType(key: string, value: string): PropertyType {
	const k = key.toLowerCase();

	// MIG-022 §A.1 — nested-object-list detection (highest priority).
	// `ikhtilāf` and its transliterations route to the structured
	// nested-object-list parser. The parseFrontmatter caller checks
	// the same set BEFORE entering the simple-list branch.
	if (IKHTILAF_KEYS.has(key) || IKHTILAF_KEYS.has(k)) return 'nested-object-list';

	// List detection (highest priority for known keys)
	if (LIST_KEYS.has(k)) return 'list';
	if (value.startsWith('[') && value.endsWith(']')) return 'list';

	// Link detection
	if (/^\[\[.*\]\]$/.test(value)) return 'link';

	// Checkbox / boolean detection
	const lv = value.toLowerCase();
	if (lv === 'true' || lv === 'false') return 'checkbox';
	if (CHECKBOX_KEYS.has(k) && value === '') return 'checkbox';

	// Datetime detection (with time component)
	if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?$/.test(value)) return 'datetime';

	// Date detection (date only, including DD/MM/YYYY)
	if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return 'date';
	if (/^\d{1,2}\/\d{1,2}\/\d{4}$/.test(value)) return 'date';
	if (DATE_KEYS.has(k) && value) return 'date';

	// Number detection
	if (/^-?\d+(\.\d+)?$/.test(value) && value !== '') return 'number';

	return 'text';
}

// ─── Core state ───
export const libraries = writable<LibraryInfo[]>([]);
export const libraryStats = writable<LibraryStats[]>([]);
export const searchResults = writable<StarInfo[]>([]);
export const universeNotesLibrary = derived(libraries, ($libs) =>
	$libs.find(l => l.is_universe_notes) ?? null
);

// ─── Editing mode state ───
export const editingTabIds = writable<Set<string>>(new Set());

export function toggleEditMode(tabId: string) {
	editingTabIds.update(set => {
		const next = new Set(set);
		if (next.has(tabId)) next.delete(tabId);
		else next.add(tabId);
		return next;
	});
}

// ─── Centralized save with lock ───
const saveLocks = new Map<string, boolean>();
const recentWrites = new Map<string, number>();

/** Mark a file as recently written so the file watcher ignores it. */
export function markRecentWrite(filePath: string) {
	recentWrites.set(filePath, Date.now());
	setTimeout(() => recentWrites.delete(filePath), 2000);
}

/** Write-ahead buffer: holds content/cursor/scroll that hasn't been written to disk yet.
 *  When opening a note, check this first — it's synchronous and always has the latest data. */
const writeAheadBuffer = new Map<string, { content: string; cursorPos: number; scrollTop: number }>();
/** localStorage key for the crash-safe wab backup. Single source of truth so
 *  the five readers/writers can't drift apart on a typo. */
const WAB_LS_KEY = 'constellation-wab';

export function setWriteAhead(filePath: string, content: string, cursorPos: number, scrollTop: number) {
	const entry = { content, cursorPos, scrollTop };
	writeAheadBuffer.set(filePath, entry);
	/* Also persist to localStorage as crash-safe backup (survives app restart).
	   This is synchronous and fast for single-note content. */
	try {
		const existing = JSON.parse(localStorage.getItem(WAB_LS_KEY) || '{}');
		existing[filePath] = entry;
		localStorage.setItem(WAB_LS_KEY, JSON.stringify(existing));
	} catch {}
}

export function getWriteAhead(filePath: string): { content: string; cursorPos: number; scrollTop: number } | undefined {
	/* Check in-memory buffer first (faster), fall back to localStorage */
	const mem = writeAheadBuffer.get(filePath);
	if (mem) return mem;
	try {
		const all = JSON.parse(localStorage.getItem(WAB_LS_KEY) || '{}');
		return all[filePath];
	} catch {}
	return undefined;
}

export function clearWriteAhead(filePath: string) {
	writeAheadBuffer.delete(filePath);
	try {
		const all = JSON.parse(localStorage.getItem(WAB_LS_KEY) || '{}');
		delete all[filePath];
		localStorage.setItem(WAB_LS_KEY, JSON.stringify(all));
	} catch {}
}

/**
 * §140 — extract `cid_cn` from a note's frontmatter, or `null` if absent.
 * Used as a cheap "same logical note" signature for the openNoteTab
 * write-ahead-buffer freshness check (see `openNoteTab` for the why).
 *
 * §141: regex bounded to the first `---…---` block so a 10MB note doesn't
 * scan the body. `parseFrontmatter` would also work but allocates the full
 * property array; this is the cheap-signature path.
 */
function extractCidCn(content: string): string | null {
	if (!content.startsWith('---')) return null;
	const fmEnd = content.indexOf('\n---', 4);
	const fm = fmEnd > 0 ? content.slice(0, fmEnd) : content.slice(0, 4096);
	const match = fm.match(/^cid_cn:\s*"?([^"\s\n]+)"?/m);
	return match?.[1] ?? null;
}

/**
 * §140 — path-keyed aux-state lifecycle: rename + delete propagation for
 * `writeAheadBuffer` (in-memory + localStorage backup) and `recentWrites`.
 *
 * Without this, a buffer entry under `oldPath` survives a rename/delete,
 * and a later note created at the same path hits the stale entry on
 * `openNoteTab` — tab loads with the old note's content, title, and
 * cid_cn (the corruption Boss reported in §140's discovery turn). Same
 * Rule 8 / write-time-derivation discipline §137 applied to `stageMap` /
 * `maturityMap`.
 *
 * §141: rename and delete share a single walker. The walker visits every
 * key in every aux-state container (in-memory wab, in-memory recentWrites,
 * localStorage wab) and asks `decide(key, isExact, isDescendant)` whether
 * to delete the entry, or rename it to a new key. The two public functions
 * are thin wrappers over the walker.
 *
 * Path matching is normalised via `normalizePathKey` (forward-slash +
 * case-insensitive) so a buffer key written under `C:\Foo.md` migrates
 * correctly when the rename target is `C:/Foo v2.md`. Folder renames /
 * deletes apply to every descendant via prefix match.
 */
/** Decision returned by a `walkAuxStatePaths` callback. */
type AuxStateAction =
	| { kind: 'delete' }
	| { kind: 'rename'; newKey: string };

/**
 * §141 — single walker shared by `migratePathKeyedAuxStateOnRename` and
 * `clearPathKeyedAuxStateOnDelete`. Iterates every key in the in-memory
 * `writeAheadBuffer`, in-memory `recentWrites`, and the localStorage
 * `constellation-wab` backup; for each key whose normalised form either
 * matches `targetPath` exactly or sits under it as a folder descendant,
 * calls `decide(originalKey, isExact)` and applies the returned action.
 *
 * The callback receives the ORIGINAL (non-normalised) key so folder-rename
 * suffix preservation works on case-mixed Windows paths.
 */
function walkAuxStatePaths(targetPath: string, decide: (originalKey: string, isExact: boolean) => AuxStateAction): void {
	const targetNorm = normalizePathKey(targetPath);
	const prefix = targetNorm + '/';

	const matchAndAct = <V>(
		container: { keys(): IterableIterator<string>; get(k: string): V | undefined; set(k: string, v: V): void; delete(k: string): boolean }
	): boolean => {
		const moves: Array<[string, string]> = [];
		const dels: string[] = [];
		for (const k of container.keys()) {
			const keyNorm = normalizePathKey(k);
			const isExact = keyNorm === targetNorm;
			const isDescendant = !isExact && keyNorm.startsWith(prefix);
			if (!isExact && !isDescendant) continue;
			const action = decide(k, isExact);
			if (action.kind === 'rename') moves.push([k, action.newKey]);
			else dels.push(k);
		}
		for (const [oldK, newK] of moves) {
			const v = container.get(oldK)!;
			container.delete(oldK);
			container.set(newK, v);
		}
		for (const k of dels) container.delete(k);
		return moves.length > 0 || dels.length > 0;
	};

	matchAndAct(writeAheadBuffer);
	matchAndAct(recentWrites);

	// localStorage backup — same shape, plain object instead of Map
	try {
		const all = JSON.parse(localStorage.getItem(WAB_LS_KEY) || '{}');
		let mutated = false;
		const moves: Array<[string, string]> = [];
		const dels: string[] = [];
		for (const k of Object.keys(all)) {
			const keyNorm = normalizePathKey(k);
			const isExact = keyNorm === targetNorm;
			const isDescendant = !isExact && keyNorm.startsWith(prefix);
			if (!isExact && !isDescendant) continue;
			const action = decide(k, isExact);
			if (action.kind === 'rename') moves.push([k, action.newKey]);
			else dels.push(k);
		}
		for (const [oldK, newK] of moves) {
			all[newK] = all[oldK];
			delete all[oldK];
			mutated = true;
		}
		for (const k of dels) {
			delete all[k];
			mutated = true;
		}
		if (mutated) localStorage.setItem(WAB_LS_KEY, JSON.stringify(all));
	} catch {}
}

/**
 * §140 — migrate `writeAheadBuffer` + `recentWrites` (in-memory + localStorage
 * backup) from `oldPath` to `newPath` on rename / move. Folder rename migrates
 * every descendant; the suffix is preserved from the ORIGINAL key so mixed
 * separators across the JS layer don't break.
 *
 * Without this, a buffer entry under `oldPath` survives the rename and a
 * later note created at the same path hits it on `openNoteTab` — tab loads
 * with the old note's content (the corruption Boss reported in §140's
 * discovery turn). Same Rule 8 / write-time-derivation discipline §137
 * applied to `stageMap` / `maturityMap`.
 */
function migratePathKeyedAuxStateOnRename(oldPath: string, newPath: string): void {
	if (oldPath === newPath || normalizePathKey(oldPath) === normalizePathKey(newPath)) return;
	walkAuxStatePaths(oldPath, (originalKey, isExact) => ({
		kind: 'rename',
		newKey: isExact ? newPath : newPath + originalKey.substring(oldPath.length),
	}));
}

/**
 * §140 — drop `writeAheadBuffer` + `recentWrites` entries for a deleted
 * file (or every descendant under a deleted folder). Without this, a
 * future note created at the same path hits the dead entry on
 * `openNoteTab` and loads the deleted note's content.
 */
function clearPathKeyedAuxStateOnDelete(path: string): void {
	walkAuxStatePaths(path, () => ({ kind: 'delete' }));
}

/** §3-redo.5 — paths currently inside a wikilink rename cascade window.
 *  Refcounted (Map<path, count>) so overlapping cascades — e.g. the user
 *  spam-renames two notes in the same library — don't pop each other's
 *  marks: `markCascading` increments, `clearCascading` decrements, the
 *  entry is dropped only when the count reaches zero.
 *
 *  Paths are normalised to forward-slash form on insert / lookup so a
 *  Windows tab path that travels through the JS layer with mixed
 *  separators (`C:\Foo\bar.md` vs `C:/Foo/bar.md`) still matches.
 */
const cascadingPaths = new Map<string, number>();
function normPath(p: string): string {
	return p.replace(/\\/g, '/');
}
export function markCascading(path: string) {
	const key = normPath(path);
	cascadingPaths.set(key, (cascadingPaths.get(key) ?? 0) + 1);
}
export function clearCascading(path: string) {
	const key = normPath(path);
	const n = cascadingPaths.get(key);
	if (n === undefined) return;
	if (n <= 1) cascadingPaths.delete(key);
	else cascadingPaths.set(key, n - 1);
}
/** §3-redo.5 — true if `path` is currently being rewritten by a wikilink
 *  rename cascade. NoteEditor's handleSave / handleFlush and the
 *  saveTabContent gate bail out when this returns true — writing the
 *  editor's pre-cascade doc back during the cascade window would silently
 *  undo the cascade rewrite (Concept Paper P4 / D2 / F2 post-cascade-stomp).
 *  Cheap O(1) early-exit on the steady-state empty map skips the path
 *  normalisation cost on the keystroke flush hot path. */
export function isCascading(path: string): boolean {
	if (cascadingPaths.size === 0) return false;
	return cascadingPaths.has(normPath(path));
}
/** §3-redo.7 — clear every cascading entry. Used by the Universe-switch
 *  path so a cascade in flight in the previous Universe doesn't leave
 *  stale entries that gate edits in the new Universe. */
export function clearAllCascading() { cascadingPaths.clear(); }

/** MIG-076 §D1: the REACTIVE freeze signal for the quiesce overlay. Holds the
 *  `tab.path` strings of every open pane currently inside a rename/cascade window.
 *  The rename orchestrator (`handleRenameComplete`) sets it inside the cascade
 *  block and clears it in the inner `finally`. SEPARATE from `cascadingPaths` (the
 *  non-reactive write-gate Map) on purpose: the editor surfaces subscribe to this
 *  for the read-only overlay, while the hot-path save gate stays a plain Map
 *  lookup (no reactivity on the keystroke path, Rule 1). */
export const cascadeFreeze = writable<Set<string>>(new Set());

/** §3-redo.7 — open tabs whose path lies under `libraryPath`. Both sides
 *  are normalised to forward-slash form, and the prefix match enforces a
 *  separator boundary so a sibling library with a shared name prefix
 *  (e.g. `/Foo/Bar` vs `/Foo/Bar2`) does not falsely match. Used by the
 *  cascade-orchestration path (mark + clear cascading) and by
 *  `flushAllTabsInLibrary`. */
export function tabsInLibrary(libraryPath: string): OpenTab[] {
	const libNorm = normPath(libraryPath).replace(/\/+$/, '');
	return get(openTabs).filter((t) => {
		if (!t.path) return false;
		const tabNorm = normPath(t.path);
		return tabNorm === libNorm || tabNorm.startsWith(libNorm + '/');
	});
}

/** §3-redo.4 — re-read each path from disk and bump the matching tab's
 *  `reloadVersion`. The bump flips NoteEditor's `{#key}` so NotePane
 *  destroys and remounts with the fresh `tab.content` — the recreate
 *  primitive (Option A) chosen by Boss in the §3-redo Architect. Per
 *  Concept Paper D6, recreate is the only safe way to push new body
 *  content into a CodeMirror EditorView from a parent reactive system;
 *  `$effect`-driven `view.dispatch` is forbidden (it races `{#key}`
 *  `onDestroy` and corrupts target body content — BUG-015).
 *
 *  Reads run in parallel; the resulting store update is batched into a
 *  single `openTabs.update` so N affected tabs cost one subscription
 *  notification, not N. Idempotent: if disk content already matches the
 *  tab's content (e.g. orchestrator already reloaded before the
 *  `cascade:rewrote` listener fires), no version bump and no store
 *  notification. Per-path read failures are logged and skipped — a
 *  single bad read does not block reloads for the rest. The recreate
 *  primitive loses CodeMirror undo history for the affected tabs; that
 *  trade is accepted by Concept Paper D6.
 */
export async function reloadTabsFromDisk(filePaths: string[]): Promise<void> {
	if (filePaths.length === 0) return;
	const tabs = get(openTabs);
	const targets = filePaths.filter((fp) => tabs.some((t) => t.path === fp));
	if (targets.length === 0) return;

	const reads = await Promise.all(
		targets.map((fp) =>
			readNote(fp)
				.then((content) => ({ fp, content }) as const)
				.catch((err) => {
					console.error('[reloadTabsFromDisk] read failed for', fp, err);
					return null;
				})
		)
	);
	const byPath = new Map<string, string>();
	for (const r of reads) if (r) byPath.set(r.fp, r.content);
	if (byPath.size === 0) return;

	openTabs.update((ts) => {
		let mutated = false;
		const next = ts.map((t) => {
			const newContent = byPath.get(t.path);
			if (newContent === undefined || newContent === t.content) return t;
			mutated = true;
			// MIG-076 §C — the cascade authored canonical disk content; force the
			// model to adopt it so the next save composes from the cascade result.
			openNoteModel(t.id, t.path, newContent);
			return { ...t, content: newContent, reloadVersion: (t.reloadVersion ?? 0) + 1 };
		});
		return mutated ? next : ts;
	});
	// The cascade just authored canonical disk content; any in-flight
	// write-ahead buffer for these paths is now stale.
	for (const fp of byPath.keys()) clearWriteAhead(fp);
}

/**
 * MIG-082 §A.3 — toggle a task checkbox SAFELY when its note may be open.
 *
 * `toggle_task` (Rust) reads the file FROM DISK, flips the checkbox, and gate-writes it.
 * Under Single-Ownership (MIG-076) the open editor's in-memory model is the save source — so a
 * naked toggle would (1) operate on stale disk if the note has unsaved edits and (2) let the next
 * debounced save REVERT the toggle (the model never learned about it). This helper closes both:
 *   1. if the note is open AND dirty, FLUSH its model to disk first (so toggle reads the latest);
 *   2. toggle on disk;
 *   3. reloadTabsFromDisk → the model ADOPTS the toggled disk content + the {#key} remounts.
 * Reuse this from EVERY toggle site (calendar, Tasks panel, GlobalTasksView) — it also fixes the
 * pre-existing latent reconcile gap those sites had.
 */
export async function toggleTaskReconciled(filePath: string, lineNumber: number): Promise<void> {
	const openTab = get(openTabs).find((t) => t.path === filePath);
	// CRITICAL (BUG-015 F2 class): gate the WHOLE op — mark BEFORE the flush + toggle + reload, exactly
	// like the rename cascade (mark-then-flush-then-mutate-then-reload). Otherwise the open dirty note's
	// armed NotePane autosave can fire during the `await toggleTask(...)` window, pass its un-gated
	// handleSave, write the editor's PRE-toggle body, and REVERT the toggle on disk. The explicit flush
	// below uses saveNoteSession (a DIRECT write, NOT gated by isCascading), so it still flushes correctly
	// inside the cascading window — identical to how flushAllTabsInLibrary runs inside the rename cascade.
	if (openTab) markCascading(openTab.path);
	try {
		if (openTab && isNoteDirty(openTab.id)) {
			markRecentWrite(openTab.path);
			await saveNoteSession(openTab.id, openTab.path, (p, c) => writeNote(p, c, 'task_toggle_flush'), 'task_toggle_flush');
		}
		await toggleTask(filePath, lineNumber);
		await reloadTabsFromDisk([filePath]); // model ADOPTS the toggled disk + {#key} remount
	} finally {
		if (openTab) clearCascading(openTab.path);
	}
}

/** §3-redo.1 — flush every dirty tab in the affected library to disk
 *  before a wikilink rename cascade walks them. Tabs are "dirty" if
 *  they have a writeAheadBuffer entry. Without this, the cascade reads
 *  stale disk state for tabs that haven't autosaved their current
 *  edits yet — the F2-pre-cascade-staleness failure mode defined in
 *  the Rename Function Concept Paper (P4 / D2).
 *
 *  Reads from writeAheadBuffer (the canonical in-flight state, fed by
 *  the editor on every keystroke) and writes via writeNote. Bypasses
 *  the property-auto-update logic from saveTabContent — that's
 *  acceptable for flush-before-cascade because the next legitimate
 *  edit refreshes the modified date.
 *
 *  Errors are logged, not thrown. A failed flush on one tab does not
 *  block the cascade for the rest; the user sees cascade results in
 *  the toast, and any flush failures are recorded for forensics.
 *
 *  `markRecentWrite` is called for each path to suppress the file
 *  watcher's external-edit emit during the write.
 */
export async function flushAllTabsInLibrary(libraryPath: string): Promise<void> {
	const writes: Promise<void>[] = [];
	for (const tab of tabsInLibrary(libraryPath)) {
		if (SINGLE_OWNERSHIP) {
			// MIG-076 §D1 — under single ownership the MODEL is the keystroke
			// authority; the write-ahead buffer is filled only on tab teardown
			// (NoteEditor.handleFlush), NEVER per keystroke. Reading the WAB here
			// would miss a tab being actively edited, so the cascade walker would
			// rewrite that tab's STALE disk content and leave freshly-typed
			// [[links]] to the renamed note broken (the sub-1.5s pre-autosave
			// race the §D1 review surfaced). Compose the pre-cascade flush from
			// the model instead: `saveNoteSession` composes + writes + marks
			// saved, and REFUSES (skips) any tab whose model identity no longer
			// matches its path — the same identity guard the rest of §C uses.
			if (!isNoteDirty(tab.id)) continue; // clean — disk already current
			markRecentWrite(tab.path);
			writes.push(
				saveNoteSession(tab.id, tab.path, (p, c) => writeNote(p, c, 'flush_all'), 'flush_all')
					.then(() => {})
					.catch((err) => {
						console.error('[flushAllTabsInLibrary] model flush failed for', tab.path, err);
					})
			);
		} else {
			const wab = getWriteAhead(tab.path);
			if (!wab) continue; // not dirty — nothing to flush
			markRecentWrite(tab.path);
			writes.push(
				writeNote(tab.path, wab.content, 'flush_all')
					.then(() => clearWriteAhead(tab.path))
					.catch((err) => {
						console.error('[flushAllTabsInLibrary] write failed for', tab.path, err);
					})
			);
		}
	}
	await Promise.all(writes);
}

export async function saveTabContent(
	tabId: string,
	filePath: string,
	properties: FrontmatterProperty[],
	body: string
): Promise<void> {
	if (saveLocks.get(tabId)) return;
	// PropertyEditor's frontmatter edits land here directly, so the same
	// F2 post-cascade-stomp gate NoteEditor uses must apply here too. See
	// `isCascading` for the full rationale.
	if (isCascading(filePath)) return;
	saveLocks.set(tabId, true);
	try {
		// Auto-update the "updated" / "حُدث" property if it exists
		const now = new Date();
		const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
		const updatedProps = properties.map(p => {
			const k = p.key.toLowerCase();
			if ((k === 'updated' || k === 'modified' || k === 'حُدث' || k === 'تعديل') && p.type === 'date') {
				return { ...p, value: dateStr };
			}
			return p;
		});

		// MIG-076 §C — props go to the model (with the auto-date applied); the
		// disk write is composed from the model ALONE, so the body is the live
		// one the editor pane maintains, NOT the (possibly stale) `body` param
		// PropertyEditor passed. A path mismatch is REFUSED, never composed.
		let newContent: string;
		let embedBody = body;
		if (SINGLE_OWNERSHIP) {
			// expectPath guards the write: a stale PropertyEditor teardown for the
			// PREVIOUS note (its filePath) is rejected once the tab's model has been
			// repurposed to a new note — the new-note-while-open poison fix.
			editNoteProps(tabId, updatedProps, filePath);
			const r = composeNoteModel(tabId, filePath);
			if (!r.ok) return; // identity refusal — finally{} still releases the lock
			newContent = r.content;
			markNoteSaved(tabId, r.version);
			embedBody = getNoteModel(tabId)?.body.toString() ?? body;
		} else {
			newContent = buildFullContent(updatedProps, body);
		}
		// Do NOT update the store during autosave — it triggers full reactivity cascade.
		// The editor owns the content. Store is synced on tab switch / note reload.
		recentWrites.set(filePath, Date.now());
		await writeNote(filePath, newContent, 'prop_save');
		emit('screen:note-saved', { path: filePath }).catch(() => {});
		// Reindex for search (non-blocking) — updates FTS5, tags, links
		const tab = get(openTabs).find(t => t.path === filePath);
		if (tab) {
			invoke('constellation_search_reindex', { notePath: filePath, libraryName: tab.libraryName }).catch(() => {});
		}
		// Re-embed for semantic search via Rust ONNX (non-blocking)
		if (get(appSettings).enabledFeatures?.semanticSearch) {
			const tab = get(openTabs).find(t => t.path === filePath);
			if (tab) {
				invoke('constellation_embed_notes', {
					notes: [{ path: filePath, name: tab.name, content: embedBody }],
					force: true
				}).catch(() => {});
			}
		}
		// Track as recently edited in localStorage for second screen dashboard
		try {
			const key = 'constellation-recent-edited';
			const existing: { name: string; path: string; libraryName: string; editedAt: number }[] = JSON.parse(localStorage.getItem(key) || '[]');
			const tab = get(openTabs).find(t => t.path === filePath);
			if (tab) {
				const filtered = existing.filter(n => n.path !== filePath);
				filtered.unshift({ name: tab.name, path: filePath, libraryName: tab.libraryName, editedAt: Date.now() });
				localStorage.setItem(key, JSON.stringify(filtered.slice(0, 20)));
			}
		} catch {}
		setTimeout(() => recentWrites.delete(filePath), 2000);
	} finally {
		saveLocks.set(tabId, false);
	}
}

export function wasRecentlyWritten(filePath: string): boolean {
	const timestamp = recentWrites.get(filePath);
	if (!timestamp) return false;
	return Date.now() - timestamp < 2000;
}

// ─── Multi-tab state ───
export const openTabs = writable<OpenTab[]>([]);
export const activeTabId = writable<string | null>(null);

export const activeTab = derived(
	[openTabs, activeTabId],
	([$tabs, $id]) => $tabs.find(t => t.id === $id) ?? null
);

// ─── Split pane state ───
export type SplitDirection = 'vertical' | 'horizontal';

export const splitActive = writable<boolean>(false);
export const splitDirection = writable<SplitDirection>('vertical');
export const focusedTabId = writable<string | null>(null);

export const focusedTab = derived(
	[splitActive, openTabs, focusedTabId, activeTab],
	([$split, $tabs, $fid, $active]) => {
		if (!$split) return $active;
		return $tabs.find(t => t.id === $fid) ?? $active;
	}
);

// Backward compat: selectedNote derived from activeTab
export const selectedNote = derived(
	activeTab,
	($tab) => $tab ? { path: $tab.path, content: $tab.content, libraryName: $tab.libraryName } : null
);

export const libraryCount = derived(libraries, ($v) => $v.length);
export const totalStars = derived(libraryStats, ($s) => $s.reduce((sum, v) => sum + v.star_count, 0));

// ─── Per-tab navigation ───
// Supersede token per tab — a later call overwrites an in-flight earlier one
// so rapid Alt+Left / Alt+Right keypresses don't race openTabs.update with
// stale content/name/path combinations (the "tab title ≠ body" bug).
const _navTokens = new Map<string, number>();

// Ring-buffer trace of navigation calls exposed as `window.__navTrace` for
// debugging the wikilink-click cycle. 200 entries is plenty for a session.
const _navTrace: Array<{ t: number; fn: string; tabId?: string; from?: string; to?: string; stack?: string }> = [];
if (typeof window !== 'undefined') (window as unknown as Record<string, unknown>).__navTrace = _navTrace;
function _traceNav(fn: string, tabId?: string, to?: string, from?: string) {
	_navTrace.push({
		t: Date.now(), fn, tabId, to, from,
		stack: new Error().stack?.split('\n').slice(2, 6).join(' ← '),
	});
	if (_navTrace.length > 200) _navTrace.shift();
}

export function navigateBack() {
	const tab = get(splitActive) ? get(focusedTab) : get(activeTab);
	if (!tab || tab.historyIndex <= 0) return;
	const newIndex = tab.historyIndex - 1;
	const targetPath = tab.history[newIndex];
	_traceNav('navigateBack', tab.id, targetPath, tab.path);
	loadTabHistoryEntry(tab.id, targetPath, newIndex);
}

export function navigateForward() {
	const tab = get(splitActive) ? get(focusedTab) : get(activeTab);
	if (!tab || tab.historyIndex >= tab.history.length - 1) return;
	const newIndex = tab.historyIndex + 1;
	const targetPath = tab.history[newIndex];
	_traceNav('navigateForward', tab.id, targetPath, tab.path);
	loadTabHistoryEntry(tab.id, targetPath, newIndex);
}

async function loadTabHistoryEntry(tabId: string, filePath: string, newHistoryIndex: number) {
	const myToken = (_navTokens.get(tabId) ?? 0) + 1;
	_navTokens.set(tabId, myToken);
	try {
		const content: string = await invoke('read_note', { filePath });
		// If a later nav has superseded this one, don't stomp its result.
		if (_navTokens.get(tabId) !== myToken) return;

		// Name: mirror openNoteTab — prefer frontmatter `title:`, fall back to
		// the filename stem. Without this parity the tab label flips between
		// the two conventions as the user navigates forward (click) vs back
		// (history), producing visible "title ≠ body" desync.
		let name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';
		const fmTitleMatch = content.match(/^---[\s\S]*?^title:\s*"?([^"\n]+)"?\s*$/m);
		if (fmTitleMatch?.[1]) name = fmTitleMatch[1].trim();

		// Resolve library for the new path so cross-library history entries
		// (or any future cross-library nav) don't keep the old library's
		// name/path on the tab.
		const allLibs = get(libraries);
		const normalize = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
		const filePathNorm = normalize(filePath);
		let resolvedLibrary: typeof allLibs[number] | undefined;
		let bestLen = -1;
		for (const v of allLibs) {
			const libNorm = normalize(v.path);
			if (filePathNorm === libNorm || filePathNorm.startsWith(libNorm + '/')) {
				if (libNorm.length > bestLen) { bestLen = libNorm.length; resolvedLibrary = v; }
			}
		}

		openTabs.update(tabs => tabs.map(t => {
			if (t.id !== tabId) return t;
			return {
				...t,
				path: filePath,
				content,
				name,
				historyIndex: newHistoryIndex,
				highlightTerm: undefined,
				...(resolvedLibrary ? { libraryName: resolvedLibrary.name, libraryPath: resolvedLibrary.path } : {}),
			};
		}));
		openNoteModel(tabId, filePath, content); // MIG-076 §C — Alt-nav reuse drives the model synchronously
		_traceNav('loadTabHistoryEntry:applied', tabId, filePath);
	} catch { /* file may have been deleted */ }
}

// ─── Bookmarks ───
export interface Bookmark {
	id: string;
	type: 'note' | 'folder' | 'search';
	path: string;
	name: string;
	libraryName: string;
}

export const bookmarks = writable<Bookmark[]>([]);

export function addBookmark(bm: Omit<Bookmark, 'id'>) {
	const id = `bm_${Date.now()}`;
	bookmarks.update(list => [...list, { ...bm, id }]);
	saveBookmarks();
}

export function removeBookmark(id: string) {
	bookmarks.update(list => list.filter(b => b.id !== id));
	saveBookmarks();
}

export function isBookmarked(path: string): boolean {
	return get(bookmarks).some(b => b.path === path);
}

function saveBookmarks() {
	invoke('save_universe_bookmarks', { bookmarks: get(bookmarks) }).catch(e => console.error('[save] bookmarks failed:', e));
}

export async function loadBookmarks() {
	try {
		const data = await invoke<unknown[]>('read_universe_bookmarks');
		if (data && Array.isArray(data) && data.length > 0) bookmarks.set(data as Bookmark[]);
	} catch { /* ignore */ }
}

// ─── Frontmatter parsing ───
export function parseFrontmatter(content: string): { properties: FrontmatterProperty[]; body: string; rawYaml?: string } {
	const lines = content.split('\n');
	if (lines[0]?.trim() !== '---') {
		return { properties: [], body: content };
	}

	let endIndex = -1;
	for (let i = 1; i < lines.length; i++) {
		if (lines[i].trim() === '---') {
			endIndex = i;
			break;
		}
	}

	if (endIndex === -1) {
		return { properties: [], body: content };
	}

	const yamlLines = lines.slice(1, endIndex);
	const rawYaml = yamlLines.join('\n');
	const properties: FrontmatterProperty[] = [];

	let i = 0;
	while (i < yamlLines.length) {
		const line = yamlLines[i];
		const colonIdx = line.indexOf(':');

		if (colonIdx > 0 && !line.startsWith(' ') && !line.startsWith('\t')) {
			const key = line.substring(0, colonIdx).trim();
			let value = line.substring(colonIdx + 1).trim();

			// MIG-022 §A.1 — nested-object-list (e.g. ikhtilāf):
			//   ikhtilāf:
			//     - school: Hanafī
			//       position: permissible
			//     - school: Mālikī
			//       position: discouraged
			// Detect when the key is in IKHTILAF_KEYS AND the next line
			// is an indented `- field:` start. Each item gathers its
			// continuation lines (also indented but without `- `) into
			// a single Record<string, string>.
			if (
				!value &&
				(IKHTILAF_KEYS.has(key) || IKHTILAF_KEYS.has(key.toLowerCase())) &&
				i + 1 < yamlLines.length &&
				/^\s+-\s/.test(yamlLines[i + 1])
			) {
				i++;
				const nestedObjects: Array<Record<string, string>> = [];
				while (i < yamlLines.length) {
					const cur = yamlLines[i];
					if (/^\s+-\s/.test(cur)) {
						// New row begins. The first field is on this line:
						// "    - school: Hanafī"
						const obj: Record<string, string> = {};
						const firstFieldLine = cur.replace(/^\s+-\s*/, '');
						const firstColon = firstFieldLine.indexOf(':');
						if (firstColon > 0) {
							const fkey = firstFieldLine.substring(0, firstColon).trim();
							let fval = firstFieldLine.substring(firstColon + 1).trim();
							if ((fval.startsWith('"') && fval.endsWith('"')) || (fval.startsWith("'") && fval.endsWith("'"))) {
								fval = fval.slice(1, -1);
							}
							if (fkey) obj[fkey] = fval;
						}
						i++;
						// Gather continuation lines (indented, no leading dash) until
						// either next list-item starts or non-indented line appears.
						while (i < yamlLines.length) {
							const cont = yamlLines[i];
							if (/^\s+-\s/.test(cont)) break; // next row
							if (!/^\s/.test(cont)) break; // back to top-level key
							const contColon = cont.indexOf(':');
							if (contColon > 0) {
								const fkey = cont.substring(0, contColon).trim();
								let fval = cont.substring(contColon + 1).trim();
								if ((fval.startsWith('"') && fval.endsWith('"')) || (fval.startsWith("'") && fval.endsWith("'"))) {
									fval = fval.slice(1, -1);
								}
								if (fkey) obj[fkey] = fval;
							}
							i++;
						}
						nestedObjects.push(obj);
					} else {
						break;
					}
				}
				if (key) {
					// Compact display string for legacy consumers + search:
					// "Hanafī: permissible | Mālikī: discouraged"
					const summary = nestedObjects
						.map((o) => Object.entries(o).map(([k, v]) => `${k}: ${v}`).join(' / '))
						.join(' | ');
					properties.push({
						key,
						value: summary,
						type: 'nested-object-list',
						nestedObjects,
					});
				}
				continue;
			}

			// Multi-line list: key:\n  - item1\n  - item2
			const listItems: string[] = [];
			if (!value && i + 1 < yamlLines.length && /^\s+-\s/.test(yamlLines[i + 1])) {
				i++;
				while (i < yamlLines.length && /^\s+-\s/.test(yamlLines[i])) {
					let item = yamlLines[i].replace(/^\s+-\s*/, '').trim();
					if ((item.startsWith('"') && item.endsWith('"')) || (item.startsWith("'") && item.endsWith("'"))) {
						item = item.slice(1, -1);
					}
					listItems.push(item);
					i++;
				}
				if (key) {
					properties.push({ key, value: listItems.join(', '), type: 'list', listItems });
				}
				continue;
			}

			// Strip quotes
			if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
				value = value.slice(1, -1);
			}

			// Inline list: [a, b, c]
			let parsedListItems: string[] | undefined;
			if (value.startsWith('[') && value.endsWith(']')) {
				parsedListItems = value.slice(1, -1)
					.split(',')
					.map(s => s.trim().replace(/^["']|["']$/g, ''))
					.filter(Boolean);
				value = parsedListItems.join(', ');
			}

			const type = detectPropertyType(key, value);
			// Normalize DD/MM/YYYY dates to YYYY-MM-DD for storage
			if ((type === 'date' || type === 'datetime') && value) {
				value = normalizeDateValue(value);
			}
			if (key) {
				properties.push({
					key,
					value,
					type,
					listItems: parsedListItems ?? (type === 'list' ? value.split(',').map(s => s.trim()).filter(Boolean) : undefined)
				});
			}
		}
		i++;
	}

	const body = lines.slice(endIndex + 1).join('\n');
	return { properties, body, rawYaml };
}

/** MIG-022 §A.1 — shared YAML value quoter. Used by reconstructFrontmatter
 *  for both flat values and nested-object-list field values. Strings with
 *  YAML special chars get double-quoted with embedded `"` escaped. */
function quoteIfNeeded(v: string): string {
	if (v === '') return '""';
	const needsQuoting = /[:{}\[\],&*?|>!%@`#]/.test(v) ||
		v.startsWith("'") || v.startsWith('"') ||
		v === 'true' || v === 'false' ||
		v === 'null' || v === 'yes' || v === 'no';
	if (needsQuoting) return `"${v.replace(/"/g, '\\"')}"`;
	return v;
}

export function reconstructFrontmatter(properties: FrontmatterProperty[]): string {
	if (properties.length === 0) return '';

	const lines: string[] = ['---'];
	for (const prop of properties) {
		if (prop.type === 'nested-object-list' && prop.nestedObjects && prop.nestedObjects.length > 0) {
			// MIG-022 §A.1 — write nested-object-list back as YAML:
			//   ikhtilāf:
			//     - school: Hanafī
			//       position: permissible
			//     - school: Mālikī
			//       position: discouraged
			// Field order within each object follows insertion order
			// (Object.entries preserves the order parseFrontmatter
			// captured). Quote values that contain YAML special chars.
			lines.push(`${prop.key}:`);
			for (const obj of prop.nestedObjects) {
				const entries = Object.entries(obj);
				if (entries.length === 0) continue;
				const [firstKey, firstVal] = entries[0];
				lines.push(`  - ${firstKey}: ${quoteIfNeeded(firstVal)}`);
				for (const [k, v] of entries.slice(1)) {
					lines.push(`    ${k}: ${quoteIfNeeded(v)}`);
				}
			}
		} else if (prop.type === 'list' && prop.listItems && prop.listItems.length > 0) {
			lines.push(`${prop.key}:`);
			for (const item of prop.listItems) {
				lines.push(`  - ${item}`);
			}
		} else if (prop.type === 'checkbox') {
			// Write bare YAML boolean (unquoted true/false)
			lines.push(`${prop.key}: ${prop.value === 'true' ? 'true' : 'false'}`);
		} else if (prop.type === 'date' || prop.type === 'datetime' || prop.type === 'number' || prop.type === 'link') {
			lines.push(`${prop.key}: ${prop.value}`);
		} else {
			const v = prop.value;
			const needsQuoting = /[:{}\[\],&*?|>!%@`#]/.test(v) ||
				v.startsWith("'") || v.startsWith('"') ||
				v === '' || v === 'true' || v === 'false' ||
				v === 'null' || v === 'yes' || v === 'no';
			if (needsQuoting && v !== '') {
				lines.push(`${prop.key}: "${v.replace(/"/g, '\\"')}"`);
			} else {
				lines.push(`${prop.key}: ${v}`);
			}
		}
	}
	lines.push('---');
	return lines.join('\n');
}

export function buildFullContent(properties: FrontmatterProperty[], body: string): string {
	const frontmatter = reconstructFrontmatter(properties);
	if (!frontmatter) return body;
	return frontmatter + '\n' + body;
}

export async function writeNote(filePath: string, content: string, origin?: string): Promise<void> {
	// MIG-076 — `origin` labels the writer in the write journal so an anomaly
	// names its author in one line (the Stage-1 lesson: "write_note" alone
	// hid five different writers behind one tag).
	await invoke('write_note', { filePath, content, origin });
}

/**
 * MIG-006 §1: resolve the OLD human title for a note about to be renamed,
 * so the wikilink cascade can search source bodies for `[[old_title]]`
 * instead of `[[<canonical-filename-stem>]]`.
 *
 * Strategy:
 *   1. If the note is open in a tab, use `tab.name` directly — it was
 *      set from frontmatter `title:` by `index_note` and is exactly the
 *      string the cascade walker needs to scan for. Zero IPC.
 *   2. Otherwise (right-click rename in the file tree), call the
 *      `read_note_title` IPC to peek the file's frontmatter without
 *      indexing.
 *   3. If neither produces a title, fall back to the filename stem — for
 *      legacy human-named notes that don't carry a `title:` field, the
 *      filename IS the display name.
 *
 * Must be awaited BEFORE `renameItem` runs, because (a) for canonical
 * files the rename mutates frontmatter (so reading after-the-fact gives
 * the NEW title) and (b) for legacy files the filename itself changes.
 */
export async function getOldTitleForCascade(oldPath: string): Promise<string> {
	const tab = get(openTabs).find(t => t.path === oldPath);
	if (tab && tab.name) return tab.name;
	try {
		const t = await invoke<string | null>('read_note_title', { filePath: oldPath });
		if (t && t.length > 0) return t;
	} catch { /* fall through to filename stem */ }
	return oldPath.split(/[\\/]/).pop()?.replace(/\.md$/, '') ?? '';
}

export function updateTabContent(tabId: string, newContent: string) {
	const tabs = get(openTabs);
	const tab = tabs.find(t => t.id === tabId);
	/* Skip store update if tab doesn't exist or content is unchanged —
	   avoids triggering a full reactivity cascade (3800+ line layout) */
	if (!tab || tab.content === newContent) return;
	openTabs.update(ts =>
		ts.map(t => t.id === tabId ? { ...t, content: newContent } : t)
	);
}

// ─── Outline (headings) extraction ───
export interface HeadingItem {
	level: number;
	text: string;
	id: string;
}

export function extractHeadings(markdown: string): HeadingItem[] {
	const headings: HeadingItem[] = [];
	const lines = markdown.split('\n');
	for (const line of lines) {
		const match = line.match(/^(#{1,6})\s+(.+)/);
		if (match) {
			const level = match[1].length;
			const text = match[2].replace(/[#*`\[\]]/g, '').trim();
			const id = text.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '');
			headings.push({ level, text, id });
		}
	}
	return headings;
}

// ─── Split functions ───
export function toggleSplit() {
	const current = get(splitActive);
	if (!current) {
		splitActive.set(true);
		focusedTabId.set(get(activeTabId));
	} else {
		splitActive.set(false);
		// Keep the focused tab as the active one
		const fid = get(focusedTabId);
		if (fid) activeTabId.set(fid);
	}
}

export function toggleSplitDirection() {
	splitDirection.update(d => d === 'vertical' ? 'horizontal' : 'vertical');
}

export function setFocusedTab(tabId: string) {
	focusedTabId.set(tabId);
	if (!get(splitActive)) {
		activeTabId.set(tabId);
	}
}

// ─── Tab functions ───
let tabCounter = 0;

export function createEmptyTab() {
	const id = `tab_${++tabCounter}_${Date.now()}`;
	const tab: OpenTab = {
		id, path: '', content: '', libraryName: '', libraryPath: '', name: 'New tab', libraryColor: '#7c3aed',
		history: [], historyIndex: -1,
	};
	openTabs.update(tabs => [...tabs, tab]);
	// Auto-enable editing mode (WYSIWYG is always edit-ready)
	editingTabIds.update(set => { const next = new Set(set); next.add(id); return next; });
	if (get(splitActive)) {
		focusedTabId.set(id);
	} else {
		activeTabId.set(id);
	}
}

// Living Link traversal throttle: coalesce rapid repeat clicks on the same
// (source, target) pair so double-clicks don't inflate traversal_count.
const TRAVERSAL_THROTTLE_MS = 2000;
const traversalLastWrite = new Map<string, number>();

// P4.2 live refresh: optimistic per-pair traversal increments applied on top
// of the boot-graph counts. When the user follows a wikilink and we fire
// `constellation_link_traverse`, we ALSO bump this local map so the in-prose
// `×N` chip and the sidebar chips update immediately without waiting for
// the next boot-graph re-fetch. The bumps are cleared whenever the graph
// payload arrives fresh — at that point the DB has already absorbed the
// bumps, so keeping them around would double-count.
export const linkTraversalBumps = writable<Map<string, number>>(new Map());

// MIG-060 §C-fix-2 — Set of note paths that participate in the CNS gravity
// well (i.e., the linked-subgraph). LensBlockWidget reads this to decide
// whether to render the CNS gesture button per row: orphan notes get the
// 360.3D + Cataloger buttons but NOT the CNS button, because CNS has no
// node to focus for an orphan. Mirrored from `skyNodes` in +layout.svelte
// whenever the sky data refreshes.
//
// Empty set means "we don't know yet" (boot hasn't finished). During that
// window, the consumer should permissively show the CNS button — better
// to risk an occasional NO-MATCH fallback (handled gracefully by §C-fix)
// than to hide a working button.
export const skyNodePathSet = writable<Set<string>>(new Set());

/** Increment the optimistic bump for a (source, target) pair by 1. Key
 *  format matches the consumers' lookup: `source_path.toLowerCase()|target.toLowerCase()`. */
export function bumpLinkTraversal(sourcePath: string, targetLower: string) {
	const key = sourcePath.toLowerCase() + '|' + targetLower;
	linkTraversalBumps.update(m => {
		const next = new Map(m);
		next.set(key, (next.get(key) ?? 0) + 1);
		return next;
	});
}

/** Reset the bumps — call this immediately after a fresh boot-graph load
 *  lands in `allLibraryLinks`, otherwise the optimistic increments will
 *  double-count against the (now-updated) server counts. */
export function clearLinkTraversalBumps() {
	linkTraversalBumps.set(new Map());
}

/**
 * §141 — resolve `filePath`'s content + cursor/scroll for a tab open.
 * Returns `null` if the file is unreadable (caller bails). Encapsulates
 * the §140 wab/disk choice: when the write-ahead buffer hits, also read
 * disk and compare the `cid_cn` signature; on mismatch, the wab entry is
 * stale (path was reused after a rename/delete) and disk wins — including
 * dropping the wab's cursor/scroll values, which were for the old note.
 * The §140 prospective migrations in `renameItem` / `moveItem` /
 * `deleteItem` keep the buffer clean going forward; this check self-heals
 * any historical localStorage entry from before §140.
 *
 * Cost: one extra `read_note` IPC per wab-hit tab open. wab-hit is rare
 * (only when the editor flushed and the user reopens before disk save
 * completes, or after an app restart with a localStorage backup entry).
 * Negligible against the corruption it prevents.
 */
async function resolveNoteContent(filePath: string): Promise<{ content: string; cursorPos: number; scrollTop: number } | null> {
	const wab = getWriteAhead(filePath);
	if (!wab) {
		try {
			const content = await invoke<string>('read_note', { filePath });
			return { content, cursorPos: 0, scrollTop: 0 };
		} catch {
			return null;
		}
	}
	let diskContent: string | null = null;
	try { diskContent = await invoke<string>('read_note', { filePath }); } catch { /* disk unreachable; trust wab */ }
	if (diskContent !== null) {
		// MIG-076 §C-1 — FAIL-CLOSED restore. The old policy rejected the
		// buffer only when BOTH identities were readable AND differed, so any
		// unreadable identity let a stale buffer through (the W2 fail-open
		// finding). The buffer is restored ONLY when both cids are present
		// AND equal AND it isn't resurrecting an empty body over real disk
		// content (★Stage-1 finding #2). Anything less proven → disk wins
		// (the buffer's cursor/scroll are dropped with it — they belong to
		// the rejected snapshot).
		const wabCid = extractCidCn(wab.content);
		const diskCid = extractCidCn(diskContent);
		const identityProven = !!wabCid && !!diskCid && wabCid === diskCid;
		const wabBody = parseFrontmatter(wab.content).body.trim();
		const diskBody = parseFrontmatter(diskContent).body.trim();
		const emptyResurrection = wabBody === '' && diskBody !== '';
		if (!identityProven || emptyResurrection) {
			console.warn(
				'[resolveNoteContent] write-ahead-buffer rejected for', filePath,
				identityProven ? '(empty-body resurrection)' : '(identity unproven)',
				'— preferring disk',
			);
			clearWriteAhead(filePath);
			return { content: diskContent, cursorPos: 0, scrollTop: 0 };
		}
	}
	clearWriteAhead(filePath);
	return { content: wab.content, cursorPos: wab.cursorPos, scrollTop: wab.scrollTop };
}

export async function openNoteTab(filePath: string, libraryName: string, color: string = '#7c3aed', highlightTerm?: string, newTab?: boolean, fromNotePath?: string, targetLine?: number) {
	const tabs = get(openTabs);

	// If the same file is already the active tab, just update highlight
	const currentTab = get(splitActive) ? get(focusedTab) : get(activeTab);
	_traceNav('openNoteTab:entry', currentTab?.id, filePath, fromNotePath ?? currentTab?.path);
	if (currentTab && currentTab.path === filePath) {
		if (highlightTerm) {
			openTabs.update(tabs => tabs.map(t => t.id === currentTab.id ? { ...t, highlightTerm } : t));
		}
		// §A.2 — note already active (no remount): jump the live editor to the line imperatively
		// (path-verified inside; selection-only, no save).
		if (targetLine && targetLine > 0) {
			import('$lib/editor/activeEditor').then(m => m.goToLineIfActive(filePath, targetLine!)).catch(() => {});
		}
		_traceNav('openNoteTab:earlyReturn', currentTab.id, filePath);
		return;
	}

	// Living Link System: record traversal when following a wikilink (fire-and-forget)
	// Deferred until we have the note's display name (extracted from content below)
	const _fromNotePath = fromNotePath;

	const resolved = await resolveNoteContent(filePath);
	if (resolved === null) return; // File unreadable
	let content = resolved.content;
	const cursorPos = resolved.cursorPos;
	const scrollTop = resolved.scrollTop;

	// Living Link identifier (cid_cn): injected lazily the first time a note is
	// opened. Adds a `cid_cn:` property to the note's YAML frontmatter with a
	// timestamp derived from the file's creation time. Migrates any legacy
	// `cid:` to `cid_cn:` on the same pass. Only markdown files get this; the
	// vault's original filenames are never touched.
	if (filePath.endsWith('.md') || filePath.endsWith('.markdown')) {
		try {
			const updated = await invoke<string>('ensure_cid_cn_cmd', { filePath });
			if (updated && updated !== content) content = updated;
		} catch { /* non-fatal: CID stays absent, note still opens */ }
	}
	// For canonical files, extract title from frontmatter; fallback to filename stem
	let name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';
	const fmTitleMatch = content.match(/^---[\s\S]*?^title:\s*"?([^"\n]+)"?\s*$/m);
	if (fmTitleMatch?.[1]) {
		name = fmTitleMatch[1].trim();
	}

	// Living Link System: record traversal now that we have the display name
	if (_fromNotePath) {
		const nameLower = name.toLowerCase();
		const key = `${_fromNotePath}|${nameLower}`;
		const now = Date.now();
		if (now - (traversalLastWrite.get(key) ?? 0) >= TRAVERSAL_THROTTLE_MS) {
			traversalLastWrite.set(key, now);
			// Stale entries (older than TRAVERSAL_THROTTLE_MS) are already inert;
			// clearing on overflow is equivalent to letting them age out.
			if (traversalLastWrite.size > 500) traversalLastWrite.clear();
			// P4.2 live refresh: bump the optimistic counter so the chips
			// render the new count immediately. The server-side write fires
			// fire-and-forget below — if it fails, the bump remains (the user
			// clicked, they expect feedback) and gets reconciled on the next
			// boot-graph fetch anyway.
			//
			// Deferred via queueMicrotask so the bump's reactive cascade
			// (linkTraversalBumps → linkTraversalMap → NotePane $effect →
			// view.dispatch → LivePreviewPlugin rebuild) fires AFTER this
			// openNoteTab call has finished its `{#key}`-remount-triggering
			// tab update. Running the cascade synchronously mid-navigation
			// would race with the in-flight editor teardown/mount and
	// risk the same class of desync the mountedFilePath /
			// supersede-token guards defend against.
			queueMicrotask(() => bumpLinkTraversal(_fromNotePath, nameLower));
			invoke('constellation_link_traverse', { sourcePath: _fromNotePath, targetName: name }).catch(() => {});
		}
	}

	// Derive library path from registered libraries.
	// Use a path-normalized, case-insensitive prefix match so Windows paths
	// (\ vs / separators) and case differences don't silently lose the
	// library anchor — which would break embed resolution for any note.
	// Pick the LONGEST matching prefix so nested libraries
	// (e.g. "Universe" and "Universe/Project" both registered) route each
	// note to its immediate containing library.
	const allLibraries = get(libraries);
	const normalize = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
	const filePathNorm = normalize(filePath);
	let library: typeof allLibraries[number] | undefined;
	let bestLen = -1;
	for (const v of allLibraries) {
		const libNorm = normalize(v.path);
		if (filePathNorm === libNorm || filePathNorm.startsWith(libNorm + '/')) {
			if (libNorm.length > bestLen) { bestLen = libNorm.length; library = v; }
		}
	}
	const libraryPath = library?.path ?? '';
	// Derive libraryName locally from the same normalized match used for
	// libraryPath. The caller's `libraryName` arg comes from
	// `resolve_wikilink_cross_library` whose current-library branch uses
	// strict string equality against the registered library path — which
	// silently drops to "" on Windows slash/trailing-slash drift, leaving
	// the tab with an empty library chip AND poisoning the next
	// wikilink resolution (empty `currentLibraryPath` skips the
	// current-library branch and picks the first matching library in
	// store order, loading the wrong same-named note from another
	// library). Trust the local derivation; only fall back to the
	// caller's value when we genuinely couldn't match.
	const resolvedLibraryName = library?.name ?? libraryName;

	// Default: replace active tab content
	if (!newTab && currentTab) {
		// Push to tab's history (trim forward history)
		const trimmedHistory = currentTab.history.slice(0, currentTab.historyIndex + 1);
		trimmedHistory.push(filePath);
		if (trimmedHistory.length > 50) trimmedHistory.shift();
		const newHistoryIndex = trimmedHistory.length - 1;

		openTabs.update(tabs => tabs.map(t => {
			if (t.id !== currentTab.id) return t;
			return {
				...t,
				path: filePath,
				content,
				name,
				libraryName: resolvedLibraryName,
				libraryPath,
				libraryColor: color,
				highlightTerm,
				history: trimmedHistory,
				historyIndex: newHistoryIndex,
				/* Restore cursor/scroll from write-ahead buffer if it was
				   the same logical note; otherwise 0 (resolveNoteContent
				   already gated on cid_cn match). */
				cursorPos,
				scrollTop,
			};
		}));
		// MIG-076 §C — drive the model from THIS explicit open, synchronously, so
		// the reused tab's model carries the NEW note's identity immediately. Not
		// relying on NoteEditor's async ensure $effect closes the window where a
		// stale teardown save could land between reuse and ensure.
		openNoteModel(currentTab.id, filePath, content);
		// §A.2 — the {#key} remount (path changed) re-runs NotePane's mount; arm the one-shot line jump.
		if (targetLine && targetLine > 0) setPendingLineJump(currentTab.id, targetLine);
		// Auto-enable editing mode (WYSIWYG is always edit-ready)
		editingTabIds.update(set => { const next = new Set(set); next.add(currentTab.id); return next; });
		_traceNav('openNoteTab:applied', currentTab.id, filePath);
		return;
	}

	// Ctrl+click / new tab: create a new tab
	const id = `tab_${++tabCounter}_${Date.now()}`;
	const tab: OpenTab = {
		id, path: filePath, content, libraryName: resolvedLibraryName, libraryPath, name, libraryColor: color, highlightTerm,
		history: [filePath], historyIndex: 0,
		cursorPos,
		scrollTop,
	};
	openTabs.update(tabs => [...tabs, tab]);
	openNoteModel(id, filePath, content); // MIG-076 §C — model born with the tab, synchronously
	if (targetLine && targetLine > 0) setPendingLineJump(id, targetLine); // §A.2 — arm the one-shot jump for the new tab's mount

	// Auto-enable editing mode (WYSIWYG is always edit-ready)
	editingTabIds.update(set => { const next = new Set(set); next.add(id); return next; });

	// Only focus the new tab if alwaysFocusNewTabs is enabled
	const settings = get(appSettings);
	if (settings.alwaysFocusNewTabs !== false) {
		if (get(splitActive)) {
			focusedTabId.set(id);
		} else {
			activeTabId.set(id);
		}
	}
}

export function closeTab(tabId: string) {
	const tabs = get(openTabs);
	const idx = tabs.findIndex(t => t.id === tabId);
	if (idx === -1) return;

	const currentActive = get(activeTabId);
	const newTabs = tabs.filter(t => t.id !== tabId);

	// Clean up non-reactive state first (no cascade)
	saveLocks.delete(tabId);
	closeNoteModel(tabId); // MIG-076 §C — dispose this tab's content model
	const editSet = get(editingTabIds);
	if (editSet.has(tabId)) {
		const next = new Set(editSet);
		next.delete(tabId);
		editingTabIds.set(next);
	}

	// Determine new active tab BEFORE touching openTabs
	let newActiveId: string | null = currentActive;
	if (currentActive === tabId) {
		newActiveId = newTabs.length > 0
			? newTabs[Math.min(idx, newTabs.length - 1)].id
			: null;
	}

	/* Batch: set activeTabId FIRST (so $activeTab derives correctly when openTabs fires),
	   then set openTabs. This reduces from 3 cascades to at most 2,
	   and the activeTabId change is a cheap scalar update. */
	if (newActiveId !== currentActive) {
		activeTabId.set(newActiveId);
	}
	openTabs.set(newTabs);
}

/** Reorder tabs by moving a tab from one index to another */
export function reorderTab(fromId: string, toId: string) {
	const tabs = [...get(openTabs)];
	const fromIdx = tabs.findIndex(t => t.id === fromId);
	const toIdx = tabs.findIndex(t => t.id === toId);
	if (fromIdx === -1 || toIdx === -1 || fromIdx === toIdx) return;
	const [moved] = tabs.splice(fromIdx, 1);
	tabs.splice(toIdx, 0, moved);
	openTabs.set(tabs);
}

export function switchTab(tabId: string) {
	if (get(splitActive)) {
		focusedTabId.set(tabId);
	} else {
		activeTabId.set(tabId);
	}
}

/** Load libraries including child universe libraries. */
export async function loadLibraries() {
	let list: LibraryInfo[];
	try {
		// Resolve own libraries + child universe libraries (recursive, deduplicated)
		list = await invoke('resolve_universe_libraries');
	} catch {
		// Fallback to own libraries only
		list = await invoke('list_libraries');
	}
	libraries.set(list);
}

/** Load stats for all libraries (star counts, recent stars). */
export async function loadAllStats() {
	const stats: LibraryStats[] = await invoke('get_all_library_stats');
	libraryStats.set(stats);
}

/** Open folder picker and add the selected library. */
export async function addLibrary(): Promise<LibraryInfo | null> {
	const folderPath: string | null = await invoke('pick_folder');
	if (!folderPath) return null;

	const library: LibraryInfo = await invoke('add_library', { path: folderPath });
	await loadLibraries();
	await loadAllStats();
	return library;
}

/** Create a new empty library folder and register it. */
export async function createNewLibrary(name: string): Promise<LibraryInfo | null> {
	const library: LibraryInfo | null = await invoke('create_new_library', { name });
	if (library) {
		await loadLibraries();
		await loadAllStats();
	}
	return library;
}

/**
 * MIG-008 §Build.5 — create a library at an explicit parent path. Used by
 * the shared `CreateItemDialog` which collects the parent location via its
 * own "Pick…" affordance (so the user sees the location IN the dialog
 * before confirming). Refreshes libraries + stats on success.
 */
export async function createNewLibraryAt(parentPath: string, name: string): Promise<LibraryInfo> {
	const library: LibraryInfo = await invoke('create_new_library_at', { parentPath, name });
	await loadLibraries();
	await loadAllStats();
	return library;
}

/** Remove a library (does NOT delete files). */
export async function removeLibrary(libraryId: string) {
	await invoke('remove_library', { libraryId });
	await loadLibraries();
	await loadAllStats();
}

/** Remove a library with full cleanup: close tabs, stop watcher, refresh caches. */
export async function removeLibraryWithCleanup(libraryId: string) {
	// Close all open tabs from this library
	const tabs = get(openTabs);
	const library = get(libraries).find(v => v.id === libraryId);
	if (library) {
		const libraryTabs = tabs.filter(t => t.path.startsWith(library.path));
		for (const tab of libraryTabs) closeTab(tab.id);
	}
	// Stop file watcher
	try { await stopWatchingLibrary(libraryId); } catch { /* ignore */ }
	// Remove from registry
	await removeLibrary(libraryId);
}

/** Search across all libraries. */
export async function searchAllStars(query: string) {
	if (!query.trim()) {
		searchResults.set([]);
		return;
	}
	const results: StarInfo[] = await invoke('search_stars', { query });
	searchResults.set(results);
}

// ─── Constellation Search Engine (Phase 1) ───

export interface ConstellationSearchRequest {
	query?: string;
	query_embedding?: number[];  // pre-computed embedding for semantic mode
	mode: 'lexical' | 'structured' | 'semantic' | 'hybrid';
	filters?: {
		properties?: { key: string; op: string; value?: string }[];
		tags?: string[];
		wikilinks_to?: string[];
		wikilinks_from?: string[];
		mutual?: string[];
		mentions?: string[];
		orphans?: boolean;
		links_between?: string[];
		links_all?: string[];
		typed_links?: { link_type: string; target: string }[];
		library_names?: string[];
		maturity?: string[];
		path_prefix?: string;
	};
	limit?: number;
	include_snippet?: boolean;
	include_headings?: boolean;
}

export interface ConstellationSearchResult {
	name: string;
	path: string;
	library_name: string;
	score: number;
	match_type: string;
	snippet?: string;
	heading_breadcrumb?: string[];
	modified: number;
	/**
	 * M13 — cross-lingual match badge. Populated when the result was
	 * found via a Lexical Bridge expansion to a language other than the
	 * query's source language. Example: query "tree" matches an Arabic
	 * note containing "شجرة" → `match_via: "شجرة"`. UI renders as
	 * "via شجرة" next to the result title.
	 *
	 * Absent (undefined) when the hit was same-language, a title match,
	 * or the expansion didn't produce cross-language terms.
	 */
	match_via?: string;
}

/** Initialize the search index (builds SQLite FTS5 database). */
export async function initSearchIndex(): Promise<{ note_count: number; index_size_bytes: number }> {
	return invoke('constellation_search_init');
}

/** Main search command — supports lexical, structured, and hybrid modes. */
export async function constellationSearch(request: ConstellationSearchRequest): Promise<ConstellationSearchResult[]> {
	return invoke('constellation_search', { request });
}

/** Reindex a single note after file change. */
export async function reindexNote(notePath: string, libraryName: string): Promise<void> {
	return invoke('constellation_search_reindex', { notePath, libraryName });
}

/** Store a pre-computed embedding vector for a note (from JS semantic engine). */
export async function storeNoteEmbedding(notePath: string, embedding: number[]): Promise<void> {
	return invoke('constellation_search_store_embedding', { notePath, embedding });
}

/** Find notes semantically similar to a given note. */
export async function searchSimilarNotes(notePath: string, limit?: number): Promise<ConstellationSearchResult[]> {
	return invoke('constellation_search_similar', { notePath, limit: limit ?? 20 });
}

/** Universal categorized search — searches everywhere at once. */
export interface UniversalSearchResponse {
	titles: ConstellationSearchResult[];
	contents: ConstellationSearchResult[];
	tags: ConstellationSearchResult[];
	properties: ConstellationSearchResult[];
	wikilinks: ConstellationSearchResult[];
	semantic: ConstellationSearchResult[];
}

export async function universalSearch(query: string, queryEmbedding?: number[] | null, limit?: number): Promise<UniversalSearchResponse> {
	return invoke('constellation_search_universal', { query, queryEmbedding: queryEmbedding ?? null, limit: limit ?? 0 });
}

// ─── Living Link System: P3-P5 wrappers ────────────────────────────────

export interface LinkStats {
	total_links: number;
	by_type: Record<string, number>;
	by_confidence: Record<string, number>;
	with_annotation: number;
	sample_links: Array<{ source: string; target: string; type: string; annotation: string; confidence: string; weight: number }>;
}

export interface FormulationInsight {
	source_name: string;
	target_name: string;
	link_type: string;
	annotation: string;
	weight: number;
	confidence: string;
	traversal_count: number;
	last_traversed: string;
	library_name: string;
}

export type FormulationQueryType =
	| 'strongest_evidence' | 'weak_foundations' | 'tensions' | 'stagnating'
	| 'abandoned' | 'emerging' | 'bias_check' | 'most_connected';

export async function linkStats(): Promise<LinkStats> {
	return invoke('constellation_link_stats');
}

// (linkDecay() wrapper removed 2026-06-10 with MIG-073: its only caller — the
// 24h decay job — was deleted when stored decay was retired; decay is
// display-only via effectiveLinkWeight. The Rust IPC remains, read-only.)

export type LinkConfidence = 'hypothesis' | 'evidence' | 'established' | 'contested';

/**
 * Set a link's confidence level. Accepts all 4 tiers. The Rust side will
 * overwrite whatever the auto-promotion rule decided — use this for
 * user-driven contest / force-promote actions.
 */
export async function setLinkConfidence(sourcePath: string, targetName: string, confidence: LinkConfidence): Promise<void> {
	await invoke('constellation_link_set_confidence', { sourcePath, targetName, confidence });
}

export interface LinkConfidenceBackfillResult {
	promoted_to_established: number;
	promoted_to_evidence: number;
	total: number;
}

/**
 * One-shot: age-assign confidence on existing rows that already meet the
 * traversal thresholds (≥10 → established, ≥3 → evidence). Never downgrades;
 * preserves user-set `contested`.
 */
export async function backfillLinkConfidence(): Promise<LinkConfidenceBackfillResult> {
	return invoke('constellation_link_backfill_confidence');
}

/** Archive a link (soft delete — status='archived', weight zeroed). Reversible via unarchiveLink. */
export async function archiveLink(sourcePath: string, targetName: string): Promise<void> {
	await invoke('constellation_link_archive', { sourcePath, targetName });
}

/** Resurrect an archived link — status back to 'active', weight reset to 1.0. Traversal count + confidence preserved. */
export async function unarchiveLink(sourcePath: string, targetName: string): Promise<void> {
	await invoke('constellation_link_unarchive', { sourcePath, targetName });
}

export interface ArchivedLink {
	source_path: string;
	source_name: string;
	target_name: string;
	link_type: string;
	annotation: string;
	confidence: string;
	traversal_count: number;
	last_traversed: string;
	library_name: string;
}

export async function listArchivedLinks(): Promise<ArchivedLink[]> {
	return invoke('constellation_link_archived');
}

export async function formulationAnalysis(queryType: FormulationQueryType, target?: string): Promise<FormulationInsight[]> {
	return invoke('constellation_formulation_analysis', { queryType, target: target ?? null });
}

/**
 * Map a NoteLink-like row (with weight, traversal_count, last_traversed, status fields)
 * to its current lifecycle stage. Mirrors the Rust classification in
 * `compute_lifecycle_distribution` (search.rs) so the UI doesn't need a
 * roundtrip per row.
 *
 *   spark      — created < 7 days ago, no traversal yet
 *   birth      — traversal_count = 0 (or weight ≤ 1)
 *   growth     — traversed at least once, weight < 5
 *   maturity   — weight ≥ 5
 *   dormancy   — status = 'dormant' (historical rows only: the write-decay job
 *                that set it was retired 2026-06-10 — decay is display-only)
 *   archival   — status = 'archived'
 */
export type LinkStage = 'spark' | 'birth' | 'growth' | 'maturity' | 'dormancy' | 'archival';

export function getLinkStage(link: { weight?: number; traversal_count?: number; status?: string; created?: string; last_traversed?: string }): LinkStage {
	if (link.status === 'archived') return 'archival';
	if (link.status === 'dormant') return 'dormancy';
	const w = link.weight ?? 1;
	const tc = link.traversal_count ?? 0;
	if (tc === 0) {
		// Within 7 days = spark; otherwise birth (waiting for first use)
		if (link.created) {
			const ageDays = (Date.now() - new Date(link.created).getTime()) / 86400000;
			if (ageDays < 7) return 'spark';
		}
		return 'birth';
	}
	if (w >= 5) return 'maturity';
	return 'growth';
}

const STAGE_META: Record<LinkStage, { icon: string; color: string; label: string }> = {
	spark:    { icon: '✨', color: '#a78bfa', label: 'Spark' },
	birth:    { icon: '🌱', color: '#86efac', label: 'Birth' },
	growth:   { icon: '🌿', color: '#22c55e', label: 'Growth' },
	maturity: { icon: '🌳', color: '#15803d', label: 'Maturity' },
	dormancy: { icon: '🌙', color: '#94a3b8', label: 'Dormant' },
	archival: { icon: '📦', color: '#64748b', label: 'Archived' },
};

export function getLinkStageMeta(stage: LinkStage) { return STAGE_META[stage]; }

// ─── MIG-014 — Note stage taxonomy (Living Link baseline + per-note custom term) ───

/**
 * A baseline lifecycle stage. Six of these form the canonical chain;
 * the structure is value + emoji + the (i18n-keyed) label callers
 * resolve via $t(`notePane.stage.${name}`). Used inside
 * `LIVING_LINK_BASELINE` only — there is no user-extensible analogue
 * since per-note custom terms are encoded into stage values directly
 * (see Stages Concept Paper v1.2 + Plan v4 commit 5782f15).
 */
export interface BaselineStage {
	name: string;
	emoji: string;
}

/**
 * The Living Link baseline — six stages every Constellation Universe
 * always exposes. Order is the canonical promotion path
 * (spark → birth → growth → maturity → dormancy → archival), which
 * NotePane's breadcrumb arrows step through. Custom note-types are
 * encoded as a dash suffix (`spark-concept`) per-note; nothing
 * Universe-wide stores them.
 */
export const LIVING_LINK_BASELINE: ReadonlyArray<BaselineStage> = [
	{ name: 'spark',    emoji: '✨' },
	{ name: 'birth',    emoji: '🌱' },
	{ name: 'growth',   emoji: '🌿' },
	{ name: 'maturity', emoji: '🌳' },
	{ name: 'dormancy', emoji: '😴' },
	{ name: 'archival', emoji: '📦' },
] as const;

/**
 * Old Zettelkasten emoji map. Notes saved before MIG-014 may still
 * carry these values (`fleeting/literature/permanent/synthesis`).
 * The on-disk values are preserved verbatim — the map keeps display
 * recognisable without forcing a silent migration.
 */
export const LEGACY_ZETTELKASTEN_EMOJI: Readonly<Record<string, string>> = {
	fleeting: '🌱',
	literature: '📖',
	permanent: '🔗',
	synthesis: '✨',
};

/**
 * Split a stage value into its lifecycle prefix and (optional) custom-term
 * suffix. The dash separator is canonical: `spark-concept` →
 * `{ lifecycle: 'spark', suffix: 'concept' }`. Values without a dash
 * have an empty suffix. Trailing dash (`spark-`) yields empty suffix.
 */
export function splitStage(stage: string): { lifecycle: string; suffix: string } {
	const i = stage.indexOf('-');
	return i < 0
		? { lifecycle: stage, suffix: '' }
		: { lifecycle: stage.slice(0, i), suffix: stage.slice(i + 1) };
}

/**
 * Build the human-readable display label for a stage value. Lifecycle
 * goes through i18n; suffix is rendered with first-letter-capitalised.
 * Pure — same input always yields same output.
 */
export function stageLabel(stage: string, t: (k: string) => string): string {
	const { lifecycle, suffix } = splitStage(stage);
	const isBaseline = LIVING_LINK_BASELINE.some(b => b.name === lifecycle);
	const lifecycleLabel = isBaseline
		? t(`notePane.stage.${lifecycle}`)
		: lifecycle.charAt(0).toUpperCase() + lifecycle.slice(1);
	if (!suffix) return lifecycleLabel;
	const suffixDisplay = suffix.charAt(0).toUpperCase() + suffix.slice(1);
	return `${lifecycleLabel}-${suffixDisplay}`;
}

/**
 * Resolve a stage value's emoji. Lifecycle-only — the dash-encoded
 * custom term suffix carries no emoji of its own. Falls back to the
 * legacy Zettelkasten map for old-on-disk values, then to `''`.
 */
export function lookupStageEmoji(stage: string): string {
	const { lifecycle } = splitStage(stage);
	if (!lifecycle) return '';
	const baseline = LIVING_LINK_BASELINE.find(b => b.name === lifecycle);
	if (baseline) return baseline.emoji;
	if (LEGACY_ZETTELKASTEN_EMOJI[lifecycle]) return LEGACY_ZETTELKASTEN_EMOJI[lifecycle];
	return '';
}

/**
 * Compute the next stage in the promote chain. The chain length is
 * always 6 (the Living Link baseline) — custom suffixes carry across.
 * `spark-concept` → `birth-concept`; `archival-concept` → null.
 * Values whose lifecycle isn't a baseline (legacy Zettelkasten,
 * malformed, etc.) yield null — promote arrow hidden in those cases.
 */
export function nextStage(stage: string): string | null {
	const { lifecycle, suffix } = splitStage(stage);
	const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
	if (idx < 0 || idx === LIVING_LINK_BASELINE.length - 1) return null;
	const next = LIVING_LINK_BASELINE[idx + 1].name;
	return suffix ? `${next}-${suffix}` : next;
}

/** Symmetric to `nextStage`. `spark-concept` → null; `birth-concept` → `spark-concept`. */
export function prevStage(stage: string): string | null {
	const { lifecycle, suffix } = splitStage(stage);
	const idx = LIVING_LINK_BASELINE.findIndex(b => b.name === lifecycle);
	if (idx <= 0) return null;
	const prev = LIVING_LINK_BASELINE[idx - 1].name;
	return suffix ? `${prev}-${suffix}` : prev;
}

/** Initialize the Rust-native ONNX embedding engine. */
export async function initEmbeddingEngine(): Promise<string> {
	return invoke('constellation_init_embeddings');
}

/** Embed text using the Rust ONNX engine. Returns 384-dim vector. */
export async function embedText(text: string): Promise<number[]> {
	return invoke('constellation_embed_text', { text });
}

/** Batch embed notes and store in search DB. Returns count embedded. */
export async function embedNotes(notes: { path: string; name: string; content: string }[]): Promise<number> {
	return invoke('constellation_embed_notes', { notes });
}

/** Get embedding engine status. */
export async function embeddingStatus(): Promise<{ ready: boolean; embedded_count: number; model_loaded: boolean }> {
	return invoke('constellation_embedding_status');
}

/**
 * Strip invisible Unicode characters that browsers inject in bidi text inputs.
 * This is the ROOT fix for manual Arabic typing: RTL inputs insert directional
 * marks (LRM, RLM, ALM) and joiners (ZWJ, ZWNJ) that are invisible but break
 * string matching. Must be applied to ALL search input before any processing.
 */
export function stripInvisibleChars(text: string): string {
	return text.replace(/[\u200B-\u200F\u2028-\u202F\u2060-\u2069\u061C\uFEFF\u00AD]/g, '');
}

/**
 * Normalize Arabic text for fuzzy matching: strip diacritics (tashkeel),
 * normalize Alef variants (أإآٱ→ا), normalize Teh marbuta (ة→ه),
 * normalize Alef Maksura (ى→ي). This ensures manual typing matches
 * regardless of keyboard/input method differences.
 */
function normalizeArabicLight(text: string): string {
	return stripInvisibleChars(text)
		// Strip Arabic diacritics (Fathah, Dammah, Kasrah, Shadda, Sukun, etc.)
		.replace(/[\u0610-\u061A\u064B-\u065F\u0670\u06D6-\u06DC\u06DF-\u06E4\u06E7\u06E8\u06EA-\u06ED]/g, '')
		// Normalize Alef variants → bare Alef
		.replace(/[أإآٱ]/g, 'ا')
		// Normalize Teh marbuta → Heh
		.replace(/ة/g, 'ه')
		// Normalize Alef Maksura → Yeh
		.replace(/ى/g, 'ي');
}

/**
 * Canonicalize a search query: replace localized operators with English equivalents.
 * Always accepts English in any locale. Only translates the current locale's keywords.
 * Uses simple string matching with Arabic normalization — no complex regex lookbehinds.
 * Pattern: Excel/LibreOffice — canonical internal + locale display layer.
 */
export function canonicalizeSearchQuery(raw: string, ops: Record<string, string> | null): string {
	if (!ops) return raw;

	// ROOT FIX: strip invisible bidi characters browsers inject in RTL text inputs
	let result = stripInvisibleChars(raw);

	// Build replacement pairs: [localized, canonical]
	// Sorted by localized string length (longest first) to prevent partial matches
	const replacements: [string, string][] = [
		[ops.linksBetween, 'links between'],
		[ops.linksAll, 'links all'],
		[ops.linksTo, 'links to'],
		[ops.linksFrom, 'links from'],
		[ops.mutual, 'mutual'],
		[ops.mentions, 'mentions'],
		[ops.orphans, 'orphans'],
		// Cognitive typed link operators
		[ops.supports, 'supports'],
		[ops.contradicts, 'contradicts'],
		[ops.causes, 'causes'],
		[ops.exemplifies, 'exemplifies'],
		[ops.generalizes, 'generalizes'],
		[ops.derivesFrom, 'derives from'],
		[ops.partOf, 'part of'],
	].filter(([loc, can]) => loc && can && loc !== can) as [string, string][];

	replacements.sort((a, b) => b[0].length - a[0].length);

	const normalizedResult = normalizeArabicLight(result);

	for (const [localized, canonical] of replacements) {
		// Try exact match first
		if (result.includes(localized)) {
			result = result.split(localized).join(canonical);
			continue;
		}
		// Try normalized match (Arabic: أ→ا, ة→ه, ى→ي, strip diacritics)
		const normalizedOp = normalizeArabicLight(localized);
		if (normalizedResult.includes(normalizedOp)) {
			// Find the position in the normalized string
			const idx = normalizedResult.indexOf(normalizedOp);
			// Map back to original string position
			const before = result.slice(0, idx);
			const after = result.slice(idx);
			// Find the original span that corresponds to the normalized match
			let consumed = 0, origLen = 0;
			for (let i = 0; i < after.length && consumed < normalizedOp.length; i++) {
				origLen++;
				if (!/[\u0610-\u061A\u064B-\u065F\u0670\u06D6-\u06DC\u06DF-\u06E4\u06E7\u06E8\u06EA-\u06ED]/.test(after[i])) {
					consumed++;
				}
			}
			result = before + canonical + after.slice(origLen);
		}
	}

	// Handle "and" keyword (used in "links between [[X]] and [[Y]]")
	if (ops.and && ops.and !== 'and') {
		const andPattern = `]] ${ops.and} [[`;
		const andNormalized = `]] ${normalizeArabicLight(ops.and)} [[`;
		if (result.includes(andPattern)) {
			result = result.replace(andPattern, ']] and [[');
		} else if (normalizeArabicLight(result).includes(andNormalized)) {
			result = result.replace(new RegExp(`\\]\\]\\s*${ops.and.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*\\[\\[`, 'g'), ']] and [[');
		}
	}

	// Handle scope prefix: في: → in:
	if (ops.scope && ops.scope !== 'in') {
		if (result.includes(ops.scope + ':')) {
			result = result.split(ops.scope + ':').join('in:');
		} else {
			const normalizedScope = normalizeArabicLight(ops.scope);
			if (normalizeArabicLight(result).includes(normalizedScope + ':')) {
				result = result.replace(new RegExp(normalizedScope.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + ':', 'g'), 'in:');
			}
		}
	}

	return result;
}

/**
 * Check if a query contains advanced syntax in any supported language.
 * Uses simple string matching with Arabic normalization.
 */
export function hasAdvancedSyntaxMultilingual(q: string, ops: Record<string, string> | null): boolean {
	// ROOT FIX: strip invisible bidi characters before checking
	const clean = stripInvisibleChars(q);
	// English operators (always checked) — includes cognitive typed link operators
	if (/[#=]|links?\s+(to|from|between|all)|mutual\s|mentions?\s|orphans?|\bin:|supports\s+\[\[|contradicts\s+\[\[|causes\s+\[\[|exemplifies\s+\[\[|generalizes\s+\[\[|derives[- ]from\s+\[\[|part[- ]of\s+\[\[/i.test(clean)) return true;
	// Localized operators for current locale
	if (!ops) return false;
	const normalized = normalizeArabicLight(clean);
	return Object.values(ops).some(op => {
		if (!op || op.length < 2) return false;
		// Exact match (on clean text, no invisible chars)
		if (clean.includes(op)) return true;
		// Normalized match (Arabic fuzzy)
		if (normalized.includes(normalizeArabicLight(op))) return true;
		return false;
	});
}

/**
 * Parse a search query string into a SearchRequest.
 * Recognizes: #tag, property=value, links to [[X]], in:Library, free text.
 */
export function parseSearchQuery(raw: string): ConstellationSearchRequest {
	const filters: ConstellationSearchRequest['filters'] = {};
	let freeText = '';

	const parts = raw.split(/\s+/);
	for (const part of parts) {
		// Tag: #project
		if (part.startsWith('#') && part.length > 1) {
			if (!filters.tags) filters.tags = [];
			filters.tags.push(part.slice(1).toLowerCase());
			continue;
		}
		// Library scope: in:LibraryName
		if (part.startsWith('in:') && part.length > 3) {
			if (!filters.library_names) filters.library_names = [];
			filters.library_names.push(part.slice(3));
			continue;
		}
		// Property: key=value
		if (part.includes('=') && !part.startsWith('=')) {
			const [key, ...valueParts] = part.split('=');
			const value = valueParts.join('=');
			if (!filters.properties) filters.properties = [];
			filters.properties.push({ key, op: '=', value: value || undefined });
			continue;
		}
		freeText += (freeText ? ' ' : '') + part;
	}

	// Wikilink: "links to [[X]]"
	const wikiToRe = /links?\s+to\s+\[\[([^\]]+)\]\]/gi;
	let match;
	while ((match = wikiToRe.exec(raw)) !== null) {
		if (!filters.wikilinks_to) filters.wikilinks_to = [];
		filters.wikilinks_to.push(match[1].toLowerCase());
		freeText = freeText.replace(match[0], '').trim();
	}

	// Wikilink: "links from [[X]]"
	const wikiFromRe = /links?\s+from\s+\[\[([^\]]+)\]\]/gi;
	while ((match = wikiFromRe.exec(raw)) !== null) {
		if (!filters.wikilinks_from) filters.wikilinks_from = [];
		filters.wikilinks_from.push(match[1].toLowerCase());
		freeText = freeText.replace(match[0], '').trim();
	}

	// Mutual: "mutual [[X]]"
	const mutualRe = /mutual\s+\[\[([^\]]+)\]\]/gi;
	while ((match = mutualRe.exec(raw)) !== null) {
		if (!filters.mutual) filters.mutual = [];
		filters.mutual.push(match[1].toLowerCase());
		freeText = freeText.replace(match[0], '').trim();
	}

	// Mentions: "mentions [[X]]"
	const mentionsRe = /mentions?\s+\[\[([^\]]+)\]\]/gi;
	while ((match = mentionsRe.exec(raw)) !== null) {
		if (!filters.mentions) filters.mentions = [];
		filters.mentions.push(match[1].toLowerCase());
		freeText = freeText.replace(match[0], '').trim();
	}

	// Orphans: standalone keyword
	if (/\borphans?\b/i.test(freeText)) {
		filters.orphans = true;
		freeText = freeText.replace(/\borphans?\b/gi, '').trim();
	}

	// Links between: "links between [[X]] and [[Y]]"
	const betweenRe = /links?\s+between\s+\[\[([^\]]+)\]\]\s+and\s+\[\[([^\]]+)\]\]/gi;
	while ((match = betweenRe.exec(raw)) !== null) {
		if (!filters.links_between) filters.links_between = [];
		filters.links_between.push(match[1].toLowerCase());
		filters.links_between.push(match[2].toLowerCase());
		freeText = freeText.replace(match[0], '').trim();
	}

	// Links all: "links all [[X]]" — both incoming and outgoing
	const allLinksRe = /links?\s+all\s+\[\[([^\]]+)\]\]/gi;
	while ((match = allLinksRe.exec(raw)) !== null) {
		if (!filters.links_all) filters.links_all = [];
		filters.links_all.push(match[1].toLowerCase());
		freeText = freeText.replace(match[0], '').trim();
	}

	// Cognitive typed link operators: "supports [[X]]", "contradicts [[X]]", etc.
	const typedLinkTypes = ['supports', 'contradicts', 'causes', 'exemplifies', 'generalizes', 'derives[- ]from', 'part[- ]of'];
	const typedLinkRe = new RegExp(`(${typedLinkTypes.join('|')})\\s+\\[\\[([^\\]]+)\\]\\]`, 'gi');
	while ((match = typedLinkRe.exec(raw)) !== null) {
		if (!filters.typed_links) filters.typed_links = [];
		// Normalize type: "derives from" → "derives-from", "part of" → "part-of"
		const linkType = match[1].toLowerCase().replace(/\s+/g, '-');
		filters.typed_links.push({ link_type: linkType, target: match[2].toLowerCase() });
		freeText = freeText.replace(match[0], '').trim();
	}

	const hasFilters = Object.values(filters).some(v => v && (Array.isArray(v) ? v.length > 0 : true));
	const hasQuery = freeText.trim().length > 0;

	return {
		query: hasQuery ? freeText.trim() : undefined,
		mode: hasQuery && hasFilters ? 'hybrid' : hasQuery ? 'lexical' : 'structured',
		filters: hasFilters ? filters : undefined,
		limit: 0,
		include_snippet: true,
		include_headings: true,
	};
}

/** Close the current note (closes active tab). */
export function closeNote() {
	const id = get(activeTabId);
	if (id) closeTab(id);
}

/** Format a timestamp to relative time. */
export function timeAgo(timestamp: number): string {
	const now = Math.floor(Date.now() / 1000);
	const diff = now - timestamp;
	if (diff < 60) return 'just now';
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
	return new Date(timestamp * 1000).toLocaleDateString();
}

// ─── File operations ───
export async function createNote(folderPath: string, fileName: string, initialFrontmatter?: string): Promise<string> {
	const newPath: string = await invoke('create_note', { folderPath, fileName, initialFrontmatter: initialFrontmatter ?? null });
	return newPath;
}

/** Search notes by property key/value across all libraries */
export async function searchByProperty(key: string, value: string): Promise<any[]> {
	return await invoke('search_by_property', { key, value });
}

/** Build default frontmatter YAML for new notes (auto-dates + user-defined defaults) */
export function buildDefaultFrontmatter(settings: AppSettings): string {
	const lines: string[] = [];
	const now = new Date();
	const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;

	// Auto-populate created date
	lines.push(`created: ${dateStr}`);

	// Add user-defined default properties (skip canonical fields — handled by Rust)
	const canonicalKeys = new Set(['created', 'title', 'cid', 'kind']);
	if (settings.defaultProperties) {
		for (const prop of settings.defaultProperties) {
			if (prop.key && !canonicalKeys.has(prop.key)) {
				lines.push(`${prop.key}: ${prop.value}`);
			}
		}
	}

	return lines.join('\n');
}

export async function createFolder(parentPath: string, folderName: string): Promise<string> {
	const newPath: string = await invoke('create_folder', { parentPath, folderName });
	return newPath;
}

export async function renameItem(oldPath: string, newPath: string): Promise<string> {
	// Rust returns the EFFECTIVE path. For canonical notes, rename updates
	// the frontmatter title in-place and the file stays at oldPath — so the
	// returned path equals oldPath even though we requested newPath. Trusting
	// the requested newPath would point the tab at a non-existent file and
	// the next write_note call would create a phantom duplicate (BUG-001).
	const effectivePath = await invoke<string>('rename_item', { oldPath, newPath });
	// §140: migrate write-ahead-buffer + recent-writes from oldPath to
	// effectivePath. Without this, a stale wab entry under oldPath survives
	// the rename and a future note created at the same path on
	// `openNoteTab` hits the stale entry and loads the OLD note's content
	// (cid_cn, title, body) — the corruption Boss reported.
	migratePathKeyedAuxStateOnRename(oldPath, effectivePath);
	const derivedName =
		newPath.split(/[\\/]/).pop()?.replace(/\.md$/, '') ?? '';
	// MIG-076 ★Stage-1 findings (journal-proven, 2026-06-11 22:01 + 22:20):
	// rename_item rewrites the frontmatter title on disk, so any PRE-rename
	// state for this file is stale BY DEFINITION — the migrated write-ahead
	// buffer and the tab's in-memory content both still carry the OLD title,
	// and the next flush writes them back over the renamed file (the 349 ms
	// 176 B stomp). Cure: drop the stale buffer (the recentWrites migration
	// above stands — §140's old-path defense is preserved by clearing BOTH
	// keys), read the renamed file ONCE, and fold path+name+content+
	// reloadVersion into a SINGLE tab update → exactly ONE {#key} remount
	// with fresh disk content. (Finding #2: the first fix updated the tab
	// twice — path, then content — and the instantly-destroyed middle editor
	// instance was a zombie that could flush an EMPTY initial doc over the
	// renamed file: the 159 B body-less write at 22:20:08.)
	clearWriteAhead(oldPath);
	clearWriteAhead(effectivePath);
	let fresh: string | null = null;
	try {
		fresh = await readNote(effectivePath);
	} catch { /* folder rename / unreadable target — path/name update only */ }

	openTabs.update(tabs => tabs.map(t => {
		if (t.path === oldPath) {
			// MIG-076 §C — the model's identity must follow the rename, or the
			// next save's compose would refuse the new path. A rename rewrites
			// frontmatter (title), so re-seed from fresh disk content when we
			// have it; otherwise just move the path.
			if (fresh !== null) openNoteModel(t.id, effectivePath, fresh);
			else repathNoteModel(t.id, effectivePath);
			// Path comes from Rust (may equal oldPath for canonical files).
			// Display name follows the user's intent — for canonical files
			// the title changed even though the filename didn't.
			return {
				...t,
				path: effectivePath,
				name: derivedName || t.name,
				...(fresh !== null
					? { content: fresh, reloadVersion: (t.reloadVersion ?? 0) + 1 }
					: {}),
			};
		}
		// If a folder was renamed, update paths that start with the old folder path
		if (t.path.startsWith(oldPath + '/') || t.path.startsWith(oldPath + '\\')) {
			const relative = t.path.substring(oldPath.length);
			repathNoteModel(t.id, effectivePath + relative); // MIG-076 §C
			return { ...t, path: effectivePath + relative };
		}
		return t;
	}));
	return effectivePath;
}

export async function moveItem(sourcePath: string, targetFolder: string): Promise<string> {
	const newPath = await invoke<string>('move_item', { sourcePath, targetFolder });
	// §140: same path-keyed migration as renameItem — buffer follows file.
	migratePathKeyedAuxStateOnRename(sourcePath, newPath);
	// Update any open tabs that reference the old path
	openTabs.update(tabs => tabs.map(t => {
		if (t.path === sourcePath) {
			const newName = newPath.split(/[\\/]/).pop()?.replace('.md', '') ?? t.name;
			repathNoteModel(t.id, newPath); // MIG-076 §C — model identity follows the move
			return { ...t, path: newPath, name: newName };
		}
		// If a folder was moved, update paths under it
		if (t.path.startsWith(sourcePath + '/') || t.path.startsWith(sourcePath + '\\')) {
			const relative = t.path.substring(sourcePath.length);
			repathNoteModel(t.id, targetFolder + relative); // MIG-076 §C
			return { ...t, path: targetFolder + relative };
		}
		return t;
	}));
	return newPath;
}

export async function deleteItem(path: string, permanent = false): Promise<void> {
	await invoke('delete_item', { path, permanent });
	// §140: drop the path's wab + recentWrites entries (and any descendants
	// for a folder delete). Without this, a future note created at the same
	// path hits the dead entry and loads the deleted note's content.
	clearPathKeyedAuxStateOnDelete(path);
	// Close any tabs with this path or under this folder
	openTabs.update(tabs => tabs.filter(t => {
		if (t.path === path) return false;
		if (t.path.startsWith(path + '/') || t.path.startsWith(path + '\\')) return false;
		return true;
	}));
}

/** MIG-076 §E-follow-up — the raw routing command (see `delete_path` in
 *  libraries.rs). Most callers want `deleteWithSetting`. */
export async function deletePath(path: string, mode: 'permanent' | 'trash' | 'system', trashRoot: string | null): Promise<void> {
	await invoke('delete_path', { path, mode, trashRoot });
}

/** MIG-076 §E-follow-up — delete a note/folder HONORING the user's "Deleted
 *  files" setting (permanent · .trash folder [library|universe scope] · OS
 *  Recycle Bin). Replaces the old always-permanent delete at every call site.
 *  Resolves the .trash root from the libraries list, then closes any open tabs
 *  + clears path-keyed aux state, exactly as `deleteItem` did. */
export async function deleteWithSetting(path: string): Promise<void> {
	const s = get(appSettings);
	// 'permanent' is no longer a user choice (Boss 2026-06-14 — deletes are always
	// recoverable); anything not 'local' resolves to System trash.
	const dest = s.trashDestination === 'local' ? 'local' : 'system';
	let mode: 'trash' | 'system' = 'system';
	let trashRoot: string | null = null;
	if (dest === 'local') {
		mode = 'trash';
		if (s.trashFolderScope === 'universe') {
			trashRoot = get(libraries).find(v => v.is_universe_notes)?.path ?? null;
		} else {
			const matches = get(libraryStats).filter(v => path.startsWith(v.path));
			trashRoot = matches.length
				? matches.reduce((a, b) => (b.path.length > a.path.length ? b : a)).path
				: null;
		}
		if (!trashRoot) throw new Error('Could not resolve a .trash location for this path.');
	}
	await deletePath(path, mode, trashRoot);
	// §140 — drop the path's aux state + close any tabs at/under it (as deleteItem did).
	clearPathKeyedAuxStateOnDelete(path);
	openTabs.update(tabs => tabs.filter(t => {
		if (t.path === path) return false;
		if (t.path.startsWith(path + '/') || t.path.startsWith(path + '\\')) return false;
		return true;
	}));
}

/** MIG-076 §E1b — move an existing note to the library's `.trash` (recoverable),
 *  used by the collision dialog's "Overwrite" before the create/rename proceeds. */
export async function moveToTrash(path: string, libraryPath: string): Promise<void> {
	await invoke('move_to_trash', { path, libraryPath });
}

/** MIG-076 §E-2 — write-journal diagnostics snapshot for Settings → Security &
 *  Privacy. `anomalies` = the shadow-mode would-refuse verdicts (identity +
 *  stale); they must be 0 before the §F enforcement flip. */
export interface WriteJournalStats {
	writes: number;
	anomalies: number;
	would_refuse_identity: number;
	would_refuse_stale: number;
	last_anomaly_ts: number | null;
	refused_exists: number;
	unverified_no_cid: number;
	created: number;
	enforce: boolean;
	exists: boolean;
	rotated: boolean;
	dir: string;
}

export async function readWriteJournalStats(): Promise<WriteJournalStats> {
	return await invoke('read_write_journal_stats');
}

/** Open a file or folder in the OS file manager (tauri_plugin_opener). */
export async function openPath(path: string): Promise<void> {
	await invoke('open_path', { path });
}

// ─── Wikilink resolution ───
export interface ResolvedLink {
	path: string;
	library_name: string;
	library_path: string;
	fragment: string | null;
}

export async function resolveWikilink(libraryPath: string, target: string): Promise<string | null> {
	return await invoke('resolve_wikilink', { libraryPath, target });
}

export async function resolveWikilinkCrossLibrary(currentLibraryPath: string, target: string): Promise<ResolvedLink | null> {
	const libraryList = get(libraries).map(v => [v.id, v.name, v.path] as [string, string, string]);
	return await invoke('resolve_wikilink_cross_library', { libraries: libraryList, currentLibraryPath, target });
}

export async function getNoteHeadings(filePath: string): Promise<string[]> {
	return await invoke('get_note_headings', { filePath });
}

// ─── File watcher ───
export async function startWatchingLibrary(libraryId: string, libraryPath: string): Promise<void> {
	await invoke('watch_library', { libraryId, libraryPath });
}

export async function stopWatchingLibrary(libraryId: string): Promise<void> {
	await invoke('unwatch_library', { libraryId });
}

// ─── Library appearance ───
export interface LibraryAppearance {
	accent_color: string | null;
	base_font_size: number | null;
	text_font_family: string | null;
	monospace_font_family: string | null;
	interface_font_family: string | null;
	css_theme: string | null;
}

export const libraryAppearances = writable<Record<string, LibraryAppearance>>({});

export async function loadLibraryAppearance(libraryPath: string, libraryId: string): Promise<void> {
	try {
		const appearance: LibraryAppearance = await invoke('read_library_appearance', { libraryPath });
		libraryAppearances.update(map => ({ ...map, [libraryId]: appearance }));
	} catch {
		// Silently fail — use defaults
	}
}

// ─── Backlinks scanning ───
export interface NoteLink {
	source_path: string;
	source_name: string;
	target: string;
	context: string;
	library_name: string;
	link_type: string | null;
	/** User's typed annotation from `[[target|annotation]]`. The second
	 *  parser (`extract_typed_links` in search.rs) stores the semantic
	 *  tag here and leaves `link_type` at the default "relates". Used by
	 *  `displayLinkType` to resolve which name to show on the badge. */
	annotation?: string;
	/** Living Link weight = 1 + ln(1 + traversal_count). Default 1 for
	 *  untraversed links; higher values indicate worn paths. Used by
	 *  `getBacklinks` to prioritize heavily-travelled connections. */
	weight?: number;
	/** How many times the user has traversed this link. Default 0. */
	traversal_count?: number;
	/** ISO-8601 timestamp of the most recent traversal, or empty for
	 *  never-followed links. Powers the P5 lifecycle helpers (decay,
	 *  stale flagging, confidence tiering). */
	last_traversed?: string;
	/** Confidence tier stored in the DB: "hypothesis" (default) or a
	 *  user-promoted tier. The UI derives a richer lifecycle state from
	 *  (traversal_count + last_traversed + confidence) via `linkLifecycle()`. */
	confidence?: string;
	/** Archival status: 'active' (default) or 'archived'. Archived links
	 *  are hidden from backlinks/outgoing panels but preserved on disk. */
	status?: 'active' | 'archived';
}

/** P5 — Living Link lifecycle state computed client-side from the raw
 *  DB fields. Four tiers:
 *
 *  - `fresh`: never traversed (traversal_count === 0)
 *  - `emerging`: 1–2 traversals, regardless of age — just-found paths
 *  - `established`: 3+ traversals AND touched within LINK_STALE_DAYS
 *  - `load-bearing`: 10+ traversals AND touched within LINK_STALE_DAYS
 *  - `stale`: previously traversed but untouched for LINK_STALE_DAYS+
 *
 *  The UI uses this to color-tier chips, list stale links in the
 *  CCS registers, and (future) apply weight decay. No DB write yet —
 *  tier is recomputed on every read so threshold changes apply
 *  immediately without a migration. */
export type LinkLifecycle = 'fresh' | 'emerging' | 'established' | 'load-bearing' | 'stale';

export const LINK_STALE_DAYS = 90;

/** ms in 24 hours, cached so we don't recompute per call. */
const MS_PER_DAY = 86_400_000;

/** Compute lifecycle tier for a link. Pure function of the link's
 *  traversal fields + current time; no side effects.
 *
 *  `nowMs` is a param (not `Date.now()` inline) so callers that batch
 *  many links in a derived can snapshot `Date.now()` once upstream. */
export function linkLifecycle(link: NoteLink, nowMs: number = Date.now()): LinkLifecycle {
	const tc = link.traversal_count ?? 0;
	if (tc === 0) return 'fresh';

	// Age check: parse last_traversed (ISO-8601); fall back to "active" if
	// the field is missing or malformed — never-traversed links already
	// returned above, so an empty string here means pre-P5 data that was
	// still flagged active, which we treat as fresh-looking rather than
	// stale (principle of least destruction).
	const lt = link.last_traversed ?? '';
	let ageDays = 0;
	if (lt) {
		const parsed = Date.parse(lt);
		if (!Number.isNaN(parsed)) ageDays = (nowMs - parsed) / MS_PER_DAY;
	}

	if (ageDays > LINK_STALE_DAYS) return 'stale';
	if (tc >= 10) return 'load-bearing';
	if (tc >= 3) return 'established';
	return 'emerging';
}

/** P5 slice 2 — read-time weight decay.
 *
 *  `effectiveWeight = weight * exp(-ln(2) * daysSinceTraversal / halfLifeDays)`
 *
 *  The DB column `weight = 1 + ln(1 + traversal_count)` is a pure integral
 *  of the user's traversal activity. Decay is a display/ordering concern
 *  only, never a write: that way threshold tuning (half-life slider) takes
 *  effect immediately, and the ground-truth weight never loses fidelity
 *  against a user's future revisits.
 *
 *  Callers that sort large lists should pass `nowMs` and the settings
 *  block once rather than reading them per iteration. */
export function effectiveLinkWeight(
	link: NoteLink,
	nowMs: number = Date.now(),
	halfLifeDays: number = 60,
	decayEnabled: boolean = true
): number {
	const raw = link.weight ?? 1;
	if (!decayEnabled) return raw;

	const tc = link.traversal_count ?? 0;
	if (tc === 0) return raw; // never traversed — no age to decay from

	const lt = link.last_traversed ?? '';
	if (!lt) return raw;
	const parsed = Date.parse(lt);
	if (Number.isNaN(parsed)) return raw;

	const ageDays = Math.max(0, (nowMs - parsed) / MS_PER_DAY);
	const lambda = Math.LN2 / Math.max(1, halfLifeDays);
	return raw * Math.exp(-lambda * ageDays);
}

/** Known typed-link names shared across the Backlinks/Outgoing panels,
 *  GraphMind, and the livePreview decorator. Kept in sync with the
 *  `KNOWN_LINK_TYPES` slice in `src-tauri/src/libraries.rs` and the
 *  `TYPED_LINK_TYPES` set in `src/lib/editor/livePreview.ts`. */
/** Resolve which typed name to show on a link's badge. Prefers the
 *  `annotation` field (populated by the DB-indexed parser) when it
 *  matches a known link-type value; falls back to `link_type` if THAT
 *  matches; otherwise returns `undefined` so the UI can skip the badge.
 *  Drops the vacuous default `"relates"` that every DB row carries at rest.
 *  MIG-067 §D — membership reads the Link-Type Registry (isLinkTypeValue: the
 *  8 typed acts + custom + `associative`), so `supersedes` now badges too and
 *  custom types are recognized. */
function displayLinkType(l: NoteLink): string | undefined {
	const ann = l.annotation?.trim().toLowerCase();
	if (ann && isLinkTypeValue(ann)) return ann;
	const lt = l.link_type?.trim().toLowerCase();
	if (lt && isLinkTypeValue(lt)) return lt;
	return undefined;
}

/**
 * The annotation field doubles as a typed-link slot when the user writes
 * `[[Note|supports]]`: the parser stores "supports" in `annotation`, and
 * `displayLinkType` promotes it to the rendered badge. In that case the
 * annotation is already on screen as the badge — rendering it again as
 * italic prose underneath is pure redundancy. Suppress it here so the
 * panel's `{#if bl.annotation}` block never fires for typed-link-only
 * annotations. Real prose annotations like
 * `[[Note|supports my health goal]]` survive — they're not in
 * KNOWN_LINK_TYPES and so won't equal `displayedType`.
 */
function displayAnnotation(l: NoteLink, displayedType: string | undefined): string {
	const ann = (l.annotation ?? '').trim();
	if (!ann) return '';
	if (displayedType && ann.toLowerCase() === displayedType) return '';
	return ann;
}

export async function scanLibraryLinks(libraryPath: string, libraryName: string): Promise<NoteLink[]> {
	return await invoke('scan_library_links', { libraryPath, libraryName });
}

/** Optional P5 decay config for the sort helpers. Passing it in (rather
 *  than reading the store here) keeps these functions pure and testable
 *  while letting callers batch-snapshot `Date.now()` + settings once. */
export interface LinkDecayConfig {
	nowMs: number;
	halfLifeDays: number;
	decayEnabled: boolean;
}

function sortWeight(link: NoteLink, cfg?: LinkDecayConfig): number {
	if (!cfg) return link.weight ?? 1;
	return effectiveLinkWeight(link, cfg.nowMs, cfg.halfLifeDays, cfg.decayEnabled);
}

/**
 * Group an array of per-link rows by a caller-chosen key (source path for
 * backlinks, target name for outgoing) so a single source/target with
 * multiple distinct link types collapses to ONE row whose `linkTypes`
 * array carries every type-badge that should render.
 *
 * Why this exists: a note like Lunch Plan can contain BOTH `[[X]]`
 * (regular wikilink) AND `[[X|supports]]` (typed link) targeting the
 * same note. Each becomes a separate `note_links` row server-side.
 * Without this dedupe the Backlinks / Outgoing Links panels render
 * the same source twice — once with no type badge and once with a
 * `supports` badge — distorting the user's count of how many notes
 * actually engage with the active note.
 *
 * Merging rules:
 * - `linkTypes`: union of distinct, non-empty link types across grouped rows.
 * - `traversalCount`: sum (total engagement from this source/target).
 * - `lastTraversed`: most recent ISO timestamp.
 * - `tier`: highest tier (load-bearing > established > emerging; stale beats
 *   nothing). The first non-`emerging` value seen wins because tier is
 *   already monotonic in traversal count + recency.
 * - `confidence`: strongest tier wins (`established` > `evidence` >
 *   `hypothesis`). `contested` is preserved if any row has it (it's a
 *   user-set override, never auto-overwritten).
 * - Other fields (name, path, context, etc.): kept from the first row.
 */
type DedupableRow = {
	linkType?: string;
	traversalCount?: number;
	lastTraversed?: string;
	tier?: LinkLifecycle;
	confidence?: LinkConfidence;
	annotation?: string;
};

function dedupeBySource<T extends DedupableRow>(
	rows: T[],
	keyFn: (row: T) => string,
): Array<T & { linkTypes: string[] }> {
	const CONFIDENCE_RANK: Record<LinkConfidence, number> = {
		hypothesis: 1,
		evidence: 2,
		established: 3,
		contested: 4,
	};
	// MIG-071 audit HIGH — 'fresh' was missing (svelte-check ERROR + NaN sort). Option B from
	// project_link_lifecycle_dedupe_fix: full lifecycle ladder, freshest/load-bearing wins on dedupe.
	const TIER_RANK: Record<LinkLifecycle, number> = {
		stale: 0,
		fresh: 1,
		emerging: 2,
		established: 3,
		'load-bearing': 4,
	};
	const map = new Map<string, T & { linkTypes: string[] }>();
	for (const row of rows) {
		const key = keyFn(row);
		const existing = map.get(key);
		if (existing) {
			if (row.linkType && !existing.linkTypes.includes(row.linkType)) {
				existing.linkTypes.push(row.linkType);
			}
			existing.traversalCount = (existing.traversalCount ?? 0) + (row.traversalCount ?? 0);
			if (row.lastTraversed && (!existing.lastTraversed || row.lastTraversed > existing.lastTraversed)) {
				existing.lastTraversed = row.lastTraversed;
			}
			if (row.tier && (!existing.tier || (TIER_RANK[row.tier] ?? 0) > (TIER_RANK[existing.tier] ?? 0))) {
				existing.tier = row.tier;
			}
			if (row.confidence && (!existing.confidence ||
				(CONFIDENCE_RANK[row.confidence] ?? 0) > (CONFIDENCE_RANK[existing.confidence] ?? 0))) {
				existing.confidence = row.confidence;
			}
			if (row.annotation && !existing.annotation) {
				existing.annotation = row.annotation;
			}
		} else {
			map.set(key, { ...row, linkTypes: row.linkType ? [row.linkType] : [] });
		}
	}
	return Array.from(map.values());
}

export function getBacklinks(
	allLinks: NoteLink[],
	noteName: string,
	decay?: LinkDecayConfig,
	noteAliases?: string[],
) {
	// MIG-004 §9: alias-aware backlinks. A wikilink targeting any of
	// the active note's aliases (frontmatter or rename-stamped) counts
	// as a backlink. Aliases are pre-lowercased + Arabic-normalized
	// in the DB; pass them through as-is.
	const target = noteName.toLowerCase();
	const targets = new Set<string>([target]);
	if (noteAliases) {
		for (const a of noteAliases) {
			if (a) targets.add(a.toLowerCase());
		}
	}
	const linked = allLinks.filter(l => targets.has(l.target.toLowerCase()) && l.status !== 'archived');
	// Sort by Living Link weight (desc), decayed if caller opted in. Ties
	// break alphabetically by source name so fresh vaults (all weights == 1)
	// stay in a stable order across boots.
	linked.sort((a, b) => {
		const wDiff = sortWeight(b, decay) - sortWeight(a, decay);
		if (wDiff !== 0) return wDiff;
		return a.source_name.localeCompare(b.source_name);
	});
	const nowMs = decay?.nowMs ?? Date.now();
	const rows = linked.map(l => {
		const dt = displayLinkType(l);
		return {
			name: l.source_name,
			path: l.source_path,
			context: l.context,
			libraryName: l.library_name,
			linkType: dt,
			traversalCount: l.traversal_count ?? 0,
			/** ISO-8601 timestamp of last traversal — empty string if never traversed. */
			lastTraversed: l.last_traversed ?? '',
			// P5 slice 3: precompute the lifecycle tier here so panels don't
			// need to import / re-derive on every row render.
			tier: linkLifecycle(l, nowMs) as LinkLifecycle,
			confidence: (l.confidence ?? 'hypothesis') as LinkConfidence,
			annotation: displayAnnotation(l, dt),
		};
	});
	// Group by source path so the same source-note never appears twice
	// because it has both a regular wikilink and a typed wikilink to the
	// active note. Type badges accumulate into `linkTypes`.
	return dedupeBySource(rows, r => r.path);
}

export function getOutgoingLinks(allLinks: NoteLink[], notePath: string, decay?: LinkDecayConfig) {
	const outgoing = allLinks.filter(l => l.source_path === notePath && l.status !== 'archived');
	// Same contract as getBacklinks — weight-desc with decay optional.
	outgoing.sort((a, b) => {
		const wDiff = sortWeight(b, decay) - sortWeight(a, decay);
		if (wDiff !== 0) return wDiff;
		return a.target.localeCompare(b.target);
	});
	const nowMs = decay?.nowMs ?? Date.now();
	const rows = outgoing.map(l => {
		const dt = displayLinkType(l);
		return {
			target: l.target,
			context: l.context,
			libraryName: l.library_name,
			linkType: dt,
			traversalCount: l.traversal_count ?? 0,
			/** ISO-8601 timestamp of last traversal — empty string if never traversed. */
			lastTraversed: l.last_traversed ?? '',
			tier: linkLifecycle(l, nowMs) as LinkLifecycle,
			confidence: (l.confidence ?? 'hypothesis') as LinkConfidence,
			annotation: displayAnnotation(l, dt),
		};
	});
	// Group by target name so a source note with both `[[X]]` and `[[X|type]]`
	// shows X once with both type chips, not twice.
	return dedupeBySource(rows, r => r.target);
}

export async function scanUnlinkedMentions(noteName: string, notePath: string): Promise<{ name: string; path: string; context: string; libraryName: string }[]> {
	const libraryList = get(libraries).map(v => [v.name, v.path] as [string, string]);
	const links: NoteLink[] = await invoke('scan_unlinked_mentions', { noteName, notePath, libraryPaths: libraryList });
	return links.map(l => ({
		name: l.source_name,
		path: l.source_path,
		context: l.context,
		libraryName: l.library_name
	}));
}

// ─── Tags scanning ───
export async function scanLibraryTags(libraryPath: string): Promise<Record<string, number>> {
	return await invoke('scan_library_tags', { libraryPath });
}

// ─── Index: Word Index ───
/** Sentinel chars that wrap matched tokens inside an `IndexMention.snippet`.
 *  The Rust backend emits them via `snippet(notes_fts, …, CHAR(2), CHAR(3), …)`
 *  so the JS side can split safely and render `<mark>` spans without ever
 *  interpreting user note content as HTML. Keep both sides in sync — the
 *  Rust literal lives in `src-tauri/src/libraries.rs`. */
export const SNIPPET_MARK_START = '\x02';
export const SNIPPET_MARK_END = '\x03';

export interface IndexMention {
	note_path: string;
	note_name: string;
	/** One-line context around the matched term. Matched tokens are wrapped
	 *  in {@link SNIPPET_MARK_START}…{@link SNIPPET_MARK_END} sentinels.
	 *  Optional: empty/absent when FTS5 produced no snippet (title-only
	 *  match against an empty body). */
	snippet?: string | null;
	/** Cross-language bridge lemma when this row surfaced because of M11
	 *  Lexical Bridge expansion (only when `expandCrossLanguage: true`
	 *  was passed to {@link readTermMentions} AND the matched token is
	 *  a non-source-language equivalent of the queried term). The
	 *  IndexPanel renders it as a small "via {lemma}" badge. Absent for
	 *  direct matches.
	 *
	 *  IMPORTANT: snake_case to match Tauri's wire format. Tauri auto-
	 *  converts case for command PARAMETERS but NOT for struct fields
	 *  in return values — same convention as `note_path` / `note_name`
	 *  / `snippet`. (MIG-010.) */
	via_lemma?: string | null;
}

export interface IndexEntry {
	term: string;
	count: number;
	mentions: IndexMention[];
	is_compound: boolean;
}

export async function scanLibraryIndex(libraryPath: string): Promise<IndexEntry[]> {
	return await invoke('scan_library_index', { libraryPath });
}

/**
 * Read the full Universe vocabulary from the FTS5 term dictionary.
 *
 * Backed by the `notes_vocab` virtual table — a `fts5vocab(notes_fts)` view
 * over the term dictionary that FTS5 already maintains as notes are added,
 * edited, or deleted (via existing triggers on `note_meta`).
 *
 * Returns one `IndexEntry` per term with display and count. `mentions` is
 * empty — the UI lazy-fetches the notes for a term via `readTermMentions`
 * when the user expands it, so we don't move millions of rows across IPC.
 *
 * No progress bar needed: the dictionary is already built. Result arrives
 * in tens of milliseconds.
 */
export async function readIndexEntries(): Promise<IndexEntry[]> {
	return await invoke('read_index_entries');
}

/**
 * Lazy-load the list of notes mentioning a given term. Called on expand.
 * Uses FTS5 MATCH against the term dictionary — sub-10 ms per call.
 *
 * `expandCrossLanguage`: when true, expand across languages via the M11
 * Lexical Bridge (MIG-010). Each row's `viaLemma` carries the bridge
 * lemma that surfaced it (None for direct matches). Off by default.
 * The IndexPanel reads `$appSettings.index.expandCrossLanguage` and
 * forwards the flag here.
 */
export async function readTermMentions(
	term: string,
	limit?: number,
	expandCrossLanguage?: boolean,
): Promise<IndexMention[]> {
	return await invoke('read_term_mentions', {
		term,
		limit: limit ?? null,
		expandCrossLanguage: expandCrossLanguage ?? null,
	});
}

/**
 * A vocabulary term that co-occurs with a query term. "co-occurs" means
 * "appears in the same note as". Note count is across the sampled matching
 * set (defaults to 200 notes, cheap on large vaults, statistically stable
 * for ranking by the time you reach a few hundred hits).
 */
export interface CooccurringTerm {
	/** Display form. Bigrams are space-joined (the `\x1f` sentinel is
	 *  unwrapped on the Rust side so the UI never sees a control char). */
	term: string;
	/** Number of sampled notes in which this term appears alongside the
	 *  query term. Never exceeds `sample_limit`. */
	note_count: number;
}

/**
 * Lazy-load co-occurring terms for an Index term. Called when the user
 * expands a row; cached per term so re-expanding is free.
 *
 * `sampleLimit` caps how many matching notes we re-tokenize (default 200,
 * max 2000). `resultLimit` caps how many co-occurring terms we return
 * (default 20, max 100). These are advisory — for rare query terms we
 * return everything; for common ones, sampling gives a stable top-K by
 * law of large numbers.
 */
export async function readCooccurringTerms(
	term: string,
	sampleLimit?: number,
	resultLimit?: number
): Promise<CooccurringTerm[]> {
	return await invoke('read_cooccurring_terms', {
		term,
		sampleLimit: sampleLimit ?? null,
		resultLimit: resultLimit ?? null,
	});
}

// ─── MIG-011 — Index filter cross-language bridge ───

/** Single cross-language lemma surfaced by the Index filter bridge.
 *  See `MIG-011-INDEX-FILTER-BRIDGE-ARCHITECT.md`. */
export interface FilterLemma {
	lemma_lower: string;
	lang: string;
}

/** Result of `lexiconExpandForFilter` — `null` when the query is out
 *  of corpus or has no cross-language equivalents. The frontend treats
 *  null as "no bridge expansion this keystroke; substring filter only."
 *  Returned `lemmas` are filtered to non-source-language only (M13
 *  same-language exclusion rule). */
export interface FilterExpansion {
	source_lemma: string;
	source_lang: string;
	lemmas: FilterLemma[];
}

/** Invoke the M11 Lexical Bridge for the Index filter box. Per-keystroke
 *  callers MUST debounce (≥300ms) to avoid spamming the IPC; the wrapper
 *  itself is just the round-trip. The result is suitable for caching by
 *  the lower-cased query string for the duration of the session. */
export async function lexiconExpandForFilter(query: string): Promise<FilterExpansion | null> {
	return await invoke('lexicon_expand_for_filter', { query });
}

// ─── MIG-013 §1D — CTSE Bridge Adapter (Index panel cross-language `≈ similar`) ───

/** One row from [`ctseSearchTermsByConcept`]. Same shape as the
 *  retired MIG-012 `TermSimilarity` so the IndexPanel filter UX is a
 *  drop-in: the dropdown adds `≈ similar` annotations to existing
 *  vocabulary terms whose M11 concept matches the user's query. */
export interface CtseTermSimilarity {
	/** Stem as stored in `term_vocab.term` (already in the FTS5
	 *  tokenizer namespace, so it matches `IndexEntry.term` exactly). */
	term: string;
	/** Cosine score of the highest M11 concept that brought this
	 *  term into the result, in [min_score, 1.0]. */
	score: number;
}

/** Embed the user's filter query, find top-K nearest M11 concepts,
 *  expand each concept to its multilingual lemmas, tokenize those
 *  lemmas through Constellation's FTS5 tokenizer, and return the
 *  subset that exists in the user's `term_vocab` — i.e., terms the
 *  user's library actually contains.
 *
 *  Replaces the retired MIG-012 `searchTermsSemantic`. All concept
 *  expansion happens at query time (Lucene `SynonymGraphFilter` /
 *  SQLite FTS5 Method 2 / CLIR query-translation pattern); there is
 *  no per-term backfill, no first-fill, no boot wait.
 *
 *  Per-keystroke callers MUST debounce (≥300 ms; CLAUDE.md Rule 3) —
 *  each call is one e5 inference + cosine sweep + indexed lookup. */
export async function ctseSearchTermsByConcept(
	query: string,
	topK?: number,
	minScore?: number,
): Promise<CtseTermSimilarity[]> {
	return await invoke('ctse_search_terms_by_concept', {
		query,
		topK: topK ?? null,
		minScore: minScore ?? null,
	});
}

// ─── MIG-012 — Index search history ───

/** One row from `read_index_history`. */
export interface IndexHistoryEntry {
	query: string;
	last_used: number;
	use_count: number;
}

export async function readIndexHistory(limit?: number): Promise<IndexHistoryEntry[]> {
	return await invoke('read_index_history', { limit: limit ?? null });
}

export async function writeIndexHistoryEntry(query: string): Promise<void> {
	return await invoke('write_index_history_entry', { query });
}

export async function clearIndexHistory(): Promise<void> {
	return await invoke('clear_index_history');
}

// ─── Navigator data ───
export interface NoteWithMeta {
	name: string;
	path: string;
	modified: number; // epoch ms
	size: number; // bytes
	preview: string; // first 200 chars, frontmatter stripped
	tags: string[];
	folder: string; // relative folder path within library
	libraryName?: string; // set by frontend after loading
}

export async function collectLibraryNotesWithMeta(libraryPath: string): Promise<NoteWithMeta[]> {
	return await invoke<NoteWithMeta[]>('collect_library_notes_with_metadata', { libraryPath });
}

// ─── Graph data ───
export interface SkyNode {
	id: string;
	name: string;
	path: string;
	libraryName: string;
	linkCount: number;
	outgoingCount: number;
	createdAt?: number; // epoch ms from file metadata
	stratum?: number;   // 1–8, from compute_note_strata (CE Phase 2)
	maturity?: string;  // seed|sapling|evergreen|canonical|wilting (CE Phase 3)
	originType?: string; // received|discovered|mixed|none (CE Phase 5)
}

export interface SkyLink {
	source: string;
	target: string;
	linkType?: string;
}

export function buildSkyData(
	allLinks: NoteLink[],
	allNotes: { name: string; path: string; libraryName: string }[],
	notePathToAliases?: Map<string, string[]>
) {
	const nodeMap = new Map<string, SkyNode>();
	// path → current note name lowercase. Used by the alias-fallback below.
	const pathToCurrentName = new Map<string, string>();
	// Add all notes as nodes
	for (const note of allNotes) {
		const id = note.name.toLowerCase();
		nodeMap.set(id, {
			id,
			name: note.name,
			path: note.path,
			libraryName: note.libraryName,
			linkCount: 0,
			outgoingCount: 0
		});
		pathToCurrentName.set(note.path, id);
	}

	// MIG-005-PARITY: Build alias_lower → current note id (lowercased name) so
	// a wikilink targeting a renamed note's old title still resolves to the
	// renamed note. Mirrors the 3-tier resolution in
	// `cache.rs::read_sky_links_raw` (MIG-004 §8). Without this, the
	// buildSkyData fallback path silently drops every edge whose target was
	// renamed since the wikilink was last reindexed — exactly the
	// "renaming-shrinks-the-bubble" symptom MIG-005 is meant to eliminate.
	const aliasToCurrentId = new Map<string, string>();
	if (notePathToAliases) {
		for (const [path, aliases] of notePathToAliases) {
			const currentId = pathToCurrentName.get(path);
			if (!currentId) continue;
			for (const alias of aliases) {
				if (!aliasToCurrentId.has(alias)) {
					aliasToCurrentId.set(alias, currentId);
				}
			}
		}
	}

	const resolveTarget = (raw: string): string | undefined => {
		if (nodeMap.has(raw)) return raw;
		const viaAlias = aliasToCurrentId.get(raw);
		if (viaAlias && nodeMap.has(viaAlias)) return viaAlias;
		return undefined;
	};

	const links: SkyLink[] = [];
	const seen = new Set<string>();

	for (const link of allLinks) {
		const sourceId = link.source_name.toLowerCase();
		const targetCandidate = link.target.toLowerCase();
		if (!nodeMap.has(sourceId)) continue;
		const targetId = resolveTarget(targetCandidate);
		if (!targetId) continue;

		// Include link_type in key so typed links with different types are kept as distinct edges
		const key = `${sourceId}->${targetId}:${link.link_type ?? ''}`;
		if (seen.has(key)) continue;
		seen.add(key);

		links.push({ source: sourceId, target: targetId, linkType: link.link_type || undefined });
		nodeMap.get(sourceId)!.linkCount++;
		nodeMap.get(sourceId)!.outgoingCount++;
		nodeMap.get(targetId)!.linkCount++;
	}

	// Include ALL notes — every note in the universe is a node in Sky View.
	// Orphans appear as smaller dots at the periphery, ready to be connected.
	const nodes = Array.from(nodeMap.values());

	return { nodes, links };
}

// ─── Daily notes ───
export async function getDailyNotePath(libraryPath: string, format = '%Y-%m-%d', folder = '', date?: string, culturalDate?: string): Promise<string> {
	// MIG-079 §D: `date` (YYYY-MM-DD) opens that day's daily note; omit it for today.
	// MIG-082 §C: `culturalDate` (e.g. "hijri: 1447-12-03") is stamped into the frontmatter ON CREATION only.
	return await invoke('get_daily_note_path', { libraryPath, format, folder, date, culturalDate });
}

// ─── Link update on rename ───
/** §3-redo.3 — what the cascade walker returns. `rewritten` is the list of
 *  absolute paths the walker successfully rewrote; `failed` is `[path, error]`
 *  pairs for files that couldn't be written, capped at the Rust-side limit
 *  (see `MAX_FAILED_REPORTED` in libraries.rs); `failed_truncated` is the
 *  count of additional failures dropped past that cap. The frontend uses
 *  `rewritten` to drive the §3-redo.4 reload of affected open tabs. The
 *  cascade also emits a `cascade:rewrote` Tauri event with the same paths
 *  array, so frontend listeners can react without awaiting the full IPC
 *  return. */
export interface CascadeResult {
	rewritten: string[];
	failed: Array<[string, string]>;
	failed_truncated: number;
}

export async function updateLinksOnRename(libraryPath: string, libraryName: string, oldName: string, newName: string): Promise<CascadeResult> {
	return await invoke('update_links_on_rename', { libraryPath, libraryName, oldName, newName });
}

// ─── Quick Capture ───
export async function quickCapture(libraryPath: string, inboxFolder: string): Promise<string> {
	return await invoke('quick_capture', { libraryPath, inboxFolder });
}

// ─── Note reading ───
export async function readNote(filePath: string): Promise<string> {
	return await invoke('read_note', { filePath });
}

export async function readNotePreview(filePath: string, maxChars = 500): Promise<string> {
	return await invoke('read_note_preview', { filePath, maxChars });
}

// ─── Font Sets ───
export interface FontSet {
	id: string;
	name: string;
	interfaceFont: string;
	textFont: string;
	monoFont: string;
	isBuiltIn: boolean;
}

const DEFAULT_UI_STACK = '-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, "Noto Sans Arabic", "Noto Sans Hebrew", "Noto Sans CJK SC", sans-serif';
const DEFAULT_MONO_STACK = '"Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace';

export const BUILTIN_FONT_SETS: FontSet[] = [
	{ id: 'system', name: 'System Default', interfaceFont: '', textFont: '', monoFont: '', isBuiltIn: true },
	{ id: 'modern', name: 'Modern', interfaceFont: 'Inter, sans-serif', textFont: 'Inter, sans-serif', monoFont: '"Fira Code", monospace', isBuiltIn: true },
	{ id: 'serif', name: 'Classic Serif', interfaceFont: 'Inter, sans-serif', textFont: 'Georgia, "Times New Roman", serif', monoFont: 'Consolas, monospace', isBuiltIn: true },
	{ id: 'arabic-traditional', name: 'Arabic Traditional', interfaceFont: '"Sakkal Majalla", "Traditional Arabic", sans-serif', textFont: '"Traditional Arabic", "Sakkal Majalla", serif', monoFont: '"Cascadia Code", Consolas, monospace', isBuiltIn: true },
	{ id: 'arabic-modern', name: 'Arabic Modern', interfaceFont: 'Dubai, "Segoe UI", sans-serif', textFont: 'Dubai, "Segoe UI", sans-serif', monoFont: '"Cascadia Code", Consolas, monospace', isBuiltIn: true },
	{ id: 'arabic-sakkal', name: 'Arabic Sakkal', interfaceFont: '"Sakkal Majalla", sans-serif', textFont: '"Sakkal Majalla", sans-serif', monoFont: '"Cascadia Code", Consolas, monospace', isBuiltIn: true },
	{ id: 'cjk', name: 'CJK', interfaceFont: '"Microsoft YaHei", "Malgun Gothic", "Yu Gothic", sans-serif', textFont: '"Microsoft YaHei", "Malgun Gothic", "Yu Gothic", serif', monoFont: '"MS Gothic", monospace', isBuiltIn: true },
	{ id: 'hebrew', name: 'Hebrew', interfaceFont: '"Segoe UI", "Arial Hebrew", sans-serif', textFont: '"Segoe UI", "Arial Hebrew", serif', monoFont: '"Cascadia Code", Consolas, monospace', isBuiltIn: true },
	{ id: 'persian', name: 'Persian', interfaceFont: '"Sakkal Majalla", "Arabic Typesetting", sans-serif', textFont: '"Arabic Typesetting", "Sakkal Majalla", sans-serif', monoFont: '"Cascadia Code", Consolas, monospace', isBuiltIn: true },
];

/** Typewriter font preset — authentic pre-PC-era fonts for each script */
export const TYPEWRITER_FONTS: { textFont: string; scriptFonts: Record<string, string> } = {
	textFont: '"Courier Prime", "PT Mono", monospace',
	scriptFonts: {
		arabic:     'Noto Naskh Arabic, serif',
		hebrew:     '"Miriam Libre", serif',
		devanagari: '"Tiro Devanagari Hindi", serif',
		cyrillic:   '"PT Mono", monospace',
		// CJK: system typewriter fonts — MS Mincho (Windows), Hiragino Mincho (macOS), Batang (Korean)
		japanese:   '"Shippori Mincho", "MS Mincho", "Hiragino Mincho Pro", serif',
		korean:     '"Gowun Batang", Batang, "AppleMyungjo", serif',
		chinese:    '"ZCOOL XiaoWei", NSimSun, "Songti SC", serif',
	},
};

export const SCRIPT_UNICODE_RANGES: Record<string, string> = {
	latin: 'U+0000-024F, U+1E00-1EFF, U+2000-206F',
	arabic: 'U+0600-06FF, U+0750-077F, U+08A0-08FF, U+FB50-FDFF, U+FE70-FEFF',
	hebrew: 'U+0590-05FF, U+FB1D-FB4F',
	cjk: 'U+4E00-9FFF, U+3000-303F, U+30A0-30FF, U+3040-309F, U+AC00-D7AF',
	devanagari: 'U+0900-097F',
	cyrillic: 'U+0400-04FF, U+0500-052F',
};

export const SCRIPT_LABELS: Record<string, string> = {
	latin: 'Latin / English',
	arabic: 'Arabic / العربية',
	hebrew: 'Hebrew / עברית',
	cjk: 'CJK / 中日韩',
	devanagari: 'Devanagari / देवनागरी',
	cyrillic: 'Cyrillic / Кириллица',
};

export const SCRIPT_SAMPLES: Record<string, string> = {
	latin: 'The quick brown fox jumps over the lazy dog',
	arabic: 'نص عربي تجريبي لعرض الخط',
	hebrew: 'טקסט עברי לדוגמה',
	cjk: '中文日本語한국어示例文字',
	devanagari: 'हिन्दी में नमूना पाठ',
	cyrillic: 'Пример текста на кириллице',
};

/** Returns the effective per-script fonts — typewriter preset overrides user scriptFonts */
export function getEffectiveScriptFonts(s: AppSettings): Record<string, string> {
	return s.fontTheme === 'typewriter' ? TYPEWRITER_FONTS.scriptFonts : (s.scriptFonts || {});
}

export function getFontSetById(id: string, customSets: FontSet[] = []): FontSet | undefined {
	return BUILTIN_FONT_SETS.find(s => s.id === id) || customSets.find(s => s.id === id);
}

export function getAllFontSets(customSets: FontSet[] = []): FontSet[] {
	return [...BUILTIN_FONT_SETS, ...customSets];
}

// ─── Settings store ───
// ─── Theme System ─────────────────────────────────────────
export interface ConstellationTheme {
	id: string;
	name: string;
	type: 'light' | 'dark';
	pairedThemeId?: string; // ID of the light/dark counterpart (for auto-switching)
	author?: string;        // Original theme author
	source?: 'custom' | 'obsidian' | 'builtin'; // Where this theme came from
	colors: {
		background: string;
		surface: string;
		text: string;
		accent: string;
		border: string;
	};
	customCSS?: string;
	styleSettingsBlocks?: import('$lib/theme/styleSettings').StyleSettingsBlock[];
	styleSettingsValues?: Record<string, string>;
	styleSettingsClasses?: Record<string, boolean>;
}

/** Convert hex color to HSL components */
export function hexToHSL(hex: string): { h: number; s: number; l: number } {
	const r = parseInt(hex.slice(1, 3), 16) / 255;
	const g = parseInt(hex.slice(3, 5), 16) / 255;
	const b = parseInt(hex.slice(5, 7), 16) / 255;
	const max = Math.max(r, g, b), min = Math.min(r, g, b);
	let h = 0, s = 0;
	const l = (max + min) / 2;
	if (max !== min) {
		const d = max - min;
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
		if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
		else if (max === g) h = ((b - r) / d + 2) / 6;
		else h = ((r - g) / d + 4) / 6;
	}
	return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
}

/** Lighten or darken a hex color by percentage */
function adjustLightness(hex: string, amount: number): string {
	const hsl = hexToHSL(hex);
	const newL = Math.max(0, Math.min(100, hsl.l + amount));
	return `hsl(${hsl.h}, ${hsl.s}%, ${newL}%)`;
}

// MIG-071 — `deriveThemeVariables` + `BUILTIN_THEMES` removed (Eisa, 2026-06-07): the theme layer is
// retired. The plain default base now comes from theme.css `:root`; the Style Setter's styleOverride
// applies on top (see the single apply $effect in +layout.svelte). No code derives theme vars anymore.

/**
 * Panel-Placement system (Tier 1 of the "note as organism" redesign).
 *
 * Each movable UI panel has a stable {@link PanelId}. It can be assigned
 * to one of a small set of {@link PanelSlot}s via {@link AppSettings.panelPlacements}.
 * Workspaces persist this map so users can save/switch entire layouts.
 *
 * Tier 1 ships: slot picker in Settings, static flanking positions for
 * Backlinks / Outgoing Links around the editor, right-sidebar tab strip
 * remains the default host for everything else.
 *
 * Tier 2 will add drag-and-drop rearrangement on top of the same schema.
 * Tier 3 will add detachable floating windows (Tauri multi-window) and
 * stack-as-tabs / split-within-slot.
 */
export type PanelId =
	| 'backlinks'
	| 'outgoing'
	| 'properties'
	| 'tags'
	| 'sky'
	| 'tasks'
	| 'calendar'
	| 'health'
	| 'provenance'
	| 'review'
	| 'inspector360';

export type PanelSlot =
	| 'left-of-note'    // inside editor area, logical-left flanking column
	| 'right-of-note'   // inside editor area, logical-right flanking column
	| 'right-sidebar'   // existing right sidebar tab strip
	| 'hidden';         // user chose to hide this panel entirely

/** MIG-070 §C polish (Item B) — a saved colour swatch: the hex plus an optional user-given name
 *  (e.g. "Brand teal"). Legacy palettes stored bare hex strings; `applyParsedSettings` coerces
 *  those to `{ hex, name: '' }` on load (back-compat, idempotent). */
export interface StyleSwatch {
	hex: string;
	name?: string;
}

export interface AppSettings {
	// Editor
	showLineNumbers: boolean;
	readableLineLength: boolean;
	tabSize: number;
	indentWithTabs: boolean;
	smartLists: boolean;
	autoPairBrackets: boolean;
	autoPairMarkdown: boolean;
	/** MIG-080 §C.2 — task due-date autosuggest (@today / bare keyword → 📅 date). */
	naturalLanguageTaskDates: boolean;
	spellcheck: boolean;
	showFloatingToolbar: boolean;
	foldHeading: boolean;
	foldIndent: boolean;
	indentationGuides: boolean;
	alwaysFocusNewTabs: boolean;
	propertiesInDocument: 'visible' | 'hidden' | 'source';

	// Files & Links
	defaultNoteLocation: 'root' | 'current' | 'folder';
	defaultNoteFolder: string;
	defaultAttachmentFolder: string;
	linkFormat: 'shortest' | 'relative' | 'absolute';
	autoUpdateLinks: boolean;
	useWikilinks: boolean;
	/** MIG-067 §E.2 — tint each typed link with its type's colour in the editor.
	 *  Off → typed links use the standard wikilink colour (type carried by the label). */
	colourTypedLinks: boolean;
	/** MIG-067 §E.2 — show the type name as a small label above each typed link. */
	showTypedLinkLabels: boolean;
	confirmDelete: boolean;
	/** Boss 2026-06-14 — 'permanent' dropped as a user choice; deletes are always
	 *  recoverable. Legacy 'permanent' is migrated to 'system' on load + at use. */
	trashDestination: 'system' | 'local';
	/** MIG-076 §E-follow-up — when trashDestination is 'local' (.trash folder),
	 *  whether .trash lives in the note's own library or at the universe root. */
	trashFolderScope: 'library' | 'universe';

	// Appearance & Themes
	titleAlignment: 'start' | 'center';
	colorScheme: 'light' | 'dark' | 'system';
	accentColor: string;
	activeThemeId: string;
	customThemes: ConstellationTheme[];
	/** MIG-070 §C — per-Universe style override: CSS-var → value, applied ON TOP of the
	 *  active theme + its styleSettingsValues (the Style Setter's persisted look). Survives
	 *  theme switches; an empty map = the pure theme look. */
	styleOverride: Record<string, string>;
	/** MIG-070 §C — the user's saved colour palette, built up in the Style Setter for re-use across
	 *  controls. Most-recent first. §C-polish Item B: each swatch can carry an optional name. */
	styleSwatches: StyleSwatch[];
	/** MIG-070 §C — the Style Setter's per-script font choices (script → font family), e.g.
	 *  { arabic: 'Amiri', cjk: '"Noto Sans CJK SC"' }. Applied via @font-face unicode-range in the
	 *  +layout font effect so each script renders in its own font, independent of the UI language.
	 *  The Latin base comes from the styleOverride font vars; empty entry = system/default. */
	perScriptFonts: Record<string, string>;
	/** Emoji & Icon Library (core plug-in): per-slot icon overrides.
	 *  Map<slot, ref> where ref is an emoji char or a namespaced icon id
	 *  ("lucide:heart", "phosphor:heart", ...). Unset = use built-in default. */
	iconOverrides?: Record<string, string>;
	interfaceFont: string;
	interfaceFontSize: number;
	textFont: string;
	monoFont: string;
	fontSize: number;
	numeralStyle: 'arabic' | 'hindi';
	dateFormat: string;
	scriptDateFormats: Record<string, string>;
	contextualDates: Record<string, boolean>;
	scriptFonts: Record<string, string>;

	// Font Sets
	fontMode: 'universal' | 'per-language';
	fontTheme: 'default' | 'typewriter';
	activeFontSetId: string;
	languageFontSets: Record<string, string>;
	customFontSets: FontSet[];
	primaryScript: string;
	enableSecondaryScript: boolean;
	secondaryScript: string;

	// Script toolbars — language-specific symbol/tool panels
	enableScriptToolbar: boolean;
	scriptToolbarScripts: string[];  // which scripts to show toolbars for

	// Dashboard
	showDashboard: boolean;

	// Focus (writing modes)
	focus: 'none' | 'blankPage' | 'typewriter' | 'manuscript' | 'flow';

	// Quick Capture
	inboxFolder: string;

	// Daily notes
	dailyNoteFormat: string;
	dailyNoteFolder: string;
	dailyNoteTemplate: string;

	// MIG-081 — Calendar systems (cultural calendars). Primary switches the whole grid
	// (standalone); secondary shows a date alongside (integrated). Filenames stay
	// Gregorian ISO regardless. Hijri uses Eisa's engine; Persian/Hebrew via Temporal.
	calendarPrimarySystem: 'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew' | 'indian' | 'buddhist' | 'chinese' | 'korean';
	calendarSecondarySystem: 'none' | 'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew' | 'indian' | 'buddhist' | 'chinese' | 'korean';
	calendarWeekStart: 0 | 1; // 0 = Sunday, 1 = Monday
	calendarShowWeekNumbers: boolean; // MIG-081 §C.2b — the "Wk" column
	// MIG-082 §B — the lunisolar (Chinese/Korean) YEAR-display preference. The two calendars share
	// identical lunar dates, so the year is what gives each its identity. Chinese: sexagenary cycle
	// (丙午年) ± Gregorian year. Korean: Dangi era (단기 4359) | Gregorian | sexagenary (병오년).
	calendarChineseYearStyle: 'sexagenary-gregorian' | 'sexagenary' | 'gregorian';
	calendarKoreanYearStyle: 'dangi' | 'dangi-gregorian' | 'gregorian' | 'sexagenary';
	// MIG-082 §B.2 — lunisolar month names: native script (五月/5월) or PHONETIC, the pronunciation in
	// the UI script (Wǔyuè / Owol; Arabic pending Boss-verified table). The Hijri "Muharram" pattern.
	calendarMonthNameStyle: 'native' | 'phonetic';
	// MIG-082 §C — stamp the non-authoritative HIJRI date into NEW daily notes' frontmatter (filename
	// stays Gregorian). Hijri-only, and only effective when the Hijri calendar is the main or secondary.
	calendarStampCulturalDate: 'off' | 'hijri';
	// MIG-081 §C.2f — Hijri engine prefs, stored here (synced with the universe) NOT in the
	// engine's per-device localStorage. Pushed into the engine on load via applyCalendarPrefs.
	// Corrections: key "year-month" (Hijri) → ±day offset (moon-sighting adjustment, cumulative
	// from that month forward). Mode: astronomical (lunar conjunction) | tabular (al-Tawfīqāt).
	calendarCorrections: Record<string, number>;
	calendarCalculationMode: 'astronomical' | 'tabular';

	// Templates
	templateFolder: string;
	folderTemplates: Record<string, string>;
	templateHotkeys: Record<string, string>;

	// Default properties for new notes
	defaultProperties: { key: string; value: string }[];

	// Updates
	autoUpdate: boolean;
	githubToken: string;

	// Security
	security: {
		libraryEncryption: boolean;
		lockOnIdle: boolean;
		lockIdleTimeout: number;
		lockPinHash: string;
		apiKeyProtection: boolean;
	};

	// Custom keyboard shortcut overrides (command ID → shortcut string, empty = unbound)
	customShortcuts: Record<string, string>;

	// Living Link pill appearance — per-type fill + text colors and shared
	// shape. Consumed reactively by BacklinksPanel + OutgoingLinksPanel so
	// the user can tune the sidebar pills without editing CSS. Defaults
	// mirror the palette that shipped with P3/P4.1.
	linkPills: {
		fill: Record<string, string>;    // type name → fill hex
		text: Record<string, string>;    // type name → text hex
		shape: {
			radius: number;              // px border-radius
			height: number;              // px explicit pill height
			fontWeight: number;          // 400..900
		};
	};

	// P5 — Living Link lifecycle. Weight decay applied at sort-time only
	// (no DB write); the raw `weight` column stays as the traversal
	// integral and the UI picks `effectiveLinkWeight(link)` for ordering.
	linkLifecycle: {
		decayEnabled: boolean;
		// Days after which decayed weight halves. λ = ln(2) / halfLifeDays.
		// 60 is a middle ground — faster than 90-day stale threshold so
		// sort order drifts before the link is flagged, giving a gradient
		// of decay rather than a cliff.
		halfLifeDays: number;
	};

	// Index panel preferences. New nested block (vs flat keys) so future
	// Index settings — term-exclusion list migration from localStorage,
	// script-filter defaults, etc. — land here without re-architecting.
	// MIG-010 introduces only the cross-language expansion toggle.
	index: {
		/** When true, clicking a term in the Index panel surfaces notes in
		 *  other languages too via the M11 Lexical Bridge, with a
		 *  "via {lemma}" badge per cross-language row. Off by default to
		 *  preserve pre-MIG-010 exact-match behaviour. */
		expandCrossLanguage: boolean;
		/** MIG-012 — when true, the Index filter ALSO does semantic search
		 *  over `term_embeddings`, surfacing conceptually-related terms
		 *  with a `≈ similar` badge. First-time on triggers an embed-all
		 *  job (~10–20 min on a 7,600-note Universe). Off by default. */
		semanticSearchEnabled: boolean;
		/** MIG-012 — when true, the Index filter box shows a dropdown of
		 *  recently-used queries on focus + saves each committed query.
		 *  Per-Universe storage in SQLite. Off by default. */
		searchHistoryEnabled: boolean;
	};

	// Sky View graph settings
	skyView: {
		nodeSize: number;
		labelVisibility: 'hover' | 'always' | 'none';
		labelFontSize: number;
		linkThickness: number;
		repelForce: number;
		linkForce: number;
		linkDistance: number;
		showOrphans: boolean;
		colorByLibrary: boolean;
	};

	// Panel Placements — Tier 1 of the "note as organism" redesign.
	// Every movable panel has a stable ID and can be assigned to a slot.
	// Default: Backlinks flanks the note on the logical-left (reading-
	// order: before), Outgoing on the logical-right. Other panels still
	// live in the right sidebar tab strip until Tier 2 exposes drag-and-
	// drop rearrangement.
	panelPlacements: Record<PanelId, PanelSlot>;
	/** Width of the left-of-note flanking column in px. Tier 1b — set by drag handle. */
	leftOfNoteWidth: number;
	/** Width of the right-of-note flanking column in px. Tier 1b — set by drag handle. */
	rightOfNoteWidth: number;

	/** MIG-079 §B — minimal/safe boot. When true, boot skips the satellite IPCs
	 * (the graph snapshot, federation, Five Acts, federation warnings) so the app
	 * comes up editor + file-tree only — the clean bring-up baseline. Satellites
	 * re-enable on demand / on the next normal boot. Default false. */
	safeBootMode: boolean;

	/** MIG-079 §C.2c — when true, the Backlinks/Outgoing panels (and the editor
	 * ×N traversal chips) fetch ONLY the active note's links via per-note SQLite
	 * queries (`get_backlink_rows`/`get_outgoing_rows`) instead of filtering the
	 * full in-memory 234k-edge array — so that array is never loaded (kills the
	 * scroll-freeze + thrashing). Default true. The old array path stays behind
	 * `false` for rollback until §C.2c-4 removes it. */
	perNoteLinkQueries: boolean;

	/** When true, the Backlinks/Outgoing panels (and other surfaces) show the
	 * one-line NSC note-summary "headline" under each row, fetched on demand.
	 * Default FALSE — the per-row summary fetch is the second-biggest cost on a
	 * hub note (SME-1), and many users don't want the extra line. Opt-in via
	 * Settings → Panels → Note summaries. */
	noteSummariesEnabled: boolean;

	/** When true, the editor shows the note's own one-line NSC summary "headline"
	 * directly under its title. Default FALSE — consistent with the sibling
	 * `noteSummariesEnabled` (panel rows) and the leaner-by-default editor. Toggle
	 * in Settings → Editor → Note title summary. */
	noteTitleSummaryEnabled: boolean;

	// Built-in features
	enabledFeatures: {
		dailyNotes: boolean;
		templates: boolean;
		skyView: boolean;
		backlinks: boolean;
		outgoingLinks: boolean;
		tags: boolean;
		pagePreview: boolean;
		search: boolean;
		quickSwitcher: boolean;
		commandPalette: boolean;
		wordCount: boolean;
		workspaces: boolean;
		index: boolean;
		semanticSearch: boolean;
		notesNavigator: boolean;
		orgChart: boolean;
		aiSkills: boolean;
		secondScreen: boolean;
		constellationMap: boolean;
		constellationSight: boolean;
		// MIG-018 (PJ-038): v3 Sight is a separate plugin from v2's
		// `constellationSight`. New field name avoids overloading the
		// v2 setting (Eisa's design call 2026-05-07 — fresh field over
		// reuse). Independent toggle: a user can enable v3 without
		// touching v2's vestigial flag.
		constellationSightV3: boolean;
		// MIG-025 §A.7: v6 user-settings flag. Fresh name (not the
		// v3-era `constellationSightV3` quirk that v5 reused) per
		// Eisa's locked decision 2026-05-14. v6 dock button + mount
		// in `+layout.svelte` gate on `SIGHT_V6_ENABLED && $appSettings
		// .enabledFeatures?.constellationSightV6 !== false`.
		constellationSightV6: boolean;
		emojiIconPicker: boolean;
		inspector360: boolean;
		// MIG-039: CECE ("The Cataloger") left-dock Core Plug-in. Gates the
		// dock button + full-page view in +layout.svelte. Distinct from the
		// `cece` settings object below (engine config). Default ON.
		cece: boolean;
		// MIG-074: CCS (Constellation Circulatory System) left-dock Core
		// Plug-in. Gates the dock button + full-page view. Default ON.
		ccs: boolean;
	};
	/**
	 * MIG-018 (PJ-038) — Sight v3 settings namespace.
	 *
	 * `projection` — Lambert (equal-area, default; community sizes
	 * accurate) or Stereographic (equal-angle; constellation shapes
	 * accurate). Per Eisa's design call §11 Q2 2026-05-07: ship both
	 * with user toggle. Switching is a frontend-only operation; the
	 * MDS embedding doesn't change.
	 *
	 * Future fields land in MIG-019 / MIG-020:
	 *   - `alwaysOnLabels`: boolean — show constellation labels at rest
	 *   - `calendarSystems`: string[] — Gregorian + user-added systems
	 *   - `magnitudeThreshold`: number — layer-peeling slider state
	 */
	sight?: {
		projection?: 'lambert' | 'stereographic';
		/** MIG-019 §2B: toggle the Milky Way density wash (PJ-035).
		 *  Default true per Eisa's design call 2026-05-07 — the band IS
		 *  the InfraNodus mechanic the v3 paper §13 describes as core
		 *  to the visual grammar. */
		showMilkyWay?: boolean;
		/** MIG-019 §2C: enabled calendar systems for the rim. Each
		 *  system renders as its own concentric ring; first in the list
		 *  is innermost. Eisa-approved §11 Q3 (2026-05-07): Gregorian
		 *  default; users add Hijri / Solar Hijri / Hebrew via Settings.
		 *  Solar-Hijri / Hebrew are placeholders for PJ-014 backfill. */
		calendarSystems?: Array<'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew'>;
		/** MIG-019 §2E: render constellation labels at territory
		 *  centroids at-rest (in addition to hover/select). Default false
		 *  per Eisa's §11 Q6 — hover/select-only is the cleaner default. */
		alwaysOnLabels?: boolean;
		/** MIG-024 §1 — Sight v5 last-used mode (per Concept Paper v3.1
		 *  §6.1; persisted per-Universe). Default 'R' (Regions) — the
		 *  lowest-cognitive-load mode for first-launch users. If saved
		 *  value is unrecognized (e.g. legacy v4 mode name), code falls
		 *  back to 'R' at read time. */
		lastMode?: 'R' | 'L' | 'T' | 'C' | 'S' | 'A' | 'P';
		/** MIG-024 §1 + D-V3 (Eisa, 2026-05-12) — Sight v5 last-used scope.
		 *  Filters the visible note set BEFORE wedge computation. Default
		 *  'universe' (whole-universe view per Concept Paper §1). 'library'
		 *  scopes to the active sidebar Library; 'folder' scopes to the
		 *  active Folder. If saved value's scope target is no longer in
		 *  context (e.g., active Library closed), falls back to 'universe'. */
		lastScope?: 'universe' | 'library' | 'folder';
		/** MIG-025 §A.11 — Sight v6 first-boot tour state. Per Concept
		 *  Paper v4.0 §11 invariant 10: tour fires on first-ever Sight v6
		 *  open and is auto-skipped on subsequent opens. Help → Sight tour
		 *  re-fires by clearing this flag (wired in §C.10 alongside the
		 *  register chip). Default false (tour shows on first launch). */
		tourSeen?: boolean;
		/** MIG-025 §A.12 — Sight v6 extended-view persistence (Concept
		 *  Paper §6.4). Cmd-Shift-D toggles. Default false: Default-
		 *  simple state on every open. True = sidebar + register chip
		 *  + mini-domes all expanded by default.
		 *  §B.10-fix-1 (Eisa cycle-1, 2026-05-16): renamed from
		 *  `proMode` because "Pro" overpromised — the feature only
		 *  controls default view extent, not professional capabilities.
		 *  Migration in `applyParsedSettings` copies old `proMode`
		 *  value into `extended` for existing users. */
		extended?: boolean;
		/** MIG-025 §A.12 — active scholarly tradition (Concept Paper §4).
		 *  5 traditions shipped in v6.2 (4 production + 1 v1-preview) per
		 *  Eisa's locked decision. Anchor-dome only; mini-domes stay
		 *  culturally neutral per §7. MIG-026 Phases γ–θ extend to 24
		 *  curated traditions.
		 *  §C.1-fix-1 (Eisa 2026-05-16): Dignāga tradition EXCLUDED.
		 *  §C.4-religious-rule (Eisa 2026-05-16): Suhrawardi Ishrāqī
		 *  also EXCLUDED per the top-principal religious-lineage
		 *  rule (orientation v2.09). The 'dignaga' and 'ishraqi'
		 *  literals are both removed from this union.
		 *  MIG-026 Phase 0 — K1 full rename: field renamed from
		 *  `activeRegister` to `activeTradition`. Migration block in
		 *  applyParsedSettings rewrites legacy `activeRegister` → new
		 *  `activeTradition` field; subsequent dignaga/ishraqi blocks
		 *  rewrite to `activeTradition`. */
		activeTradition?:
			| 'aristotelian'
			| 'pramana'
			| 'masadir'
			| 'polanyi'
			| 'mohist-san-biao'
			| 'peirce'
			| 'habermas'
			| 'dewey'
			| 'husserl'
			| 'longino'
			| 'ibn-rushd-burhan'
			| 'shatibi-maqasid'
			| 'ibn-khaldun-umran'
			| 'pardes'
			| 'maimonidean-prophecy'
			| 'talmudic-middot'
			| 'mencian-sprouts'
			| 'wang-yangming'
			| 'korean-songnihak'
			| 'mignolo-pluriversal'
			| 'dussel-transmodernity'
			| 'maldonado-torres'
			| 'akan-wiredu'
			| 'ibuanyidanda'
			// MIG-026 Phase κ.1 — user-defined tradition ids (matched by
			// the v1 JSON schema's `^user-[a-z0-9][a-z0-9-]{2,40}$`
			// pattern). The `string` widener admits any user-prefixed id;
			// runtime validation in userDefinedLoader enforces the actual
			// pattern + de-dupes against curated literals.
			| (string & {});
		/** MIG-026 Phase β — Favorite traditions pinned to the inline
		 *  chip row (A6 hybrid: 4 favorites visible inline + dropdown
		 *  for rest). Array order = display order. The chip UI shows
		 *  the first 4 elements inline; the rest live in the dropdown
		 *  with a pin/unpin toggle. Default: the 4 production-polish
		 *  traditions currently shipping (Aristotelian, pramāṇa,
		 *  masādir, Polanyi). User can re-order or replace via pin/
		 *  unpin in the dropdown UI.
		 *
		 *  When MIG-026 Phases γ–θ add 19 more traditions, those new
		 *  ones do NOT auto-pin — they appear in the dropdown by
		 *  default; user opts in by pinning. */
		favoriteTraditions?: string[];
		/** MIG-026 Phase κ.2 — user-approved tradition plugin filenames.
		 *  When a `.js` plugin file is detected in
		 *  `<Universe>/.constellation/traditions/`, the first-detection
		 *  consent banner asks the user to enable it by filename. Enabled
		 *  filenames persist here; the loader auto-imports them on each
		 *  Sight mount. Removing a filename here disables the plugin
		 *  (no uninstall — the plugin just stops loading). Default empty
		 *  per the Obsidian-trust security model (Architect §3.H). */
		enabledTraditionPlugins?: string[];
		/** MIG-025 §A.12 + Plan §A.4/§A.10 — mini-dome hex-bin aggregation
		 *  threshold. Above this many visible notes per mini, the mini
		 *  switches to hex-bin rendering with count badges. Default 5000
		 *  per Concept Paper §3.4. Tunable per Plan UX-SME condition. */
		hexBinThreshold?: number;
		/** MIG-025 §A.12 + Concept Paper §2.2 — connector-line auto-fade
		 *  threshold on the anchor dome. Above this many visible link
		 *  edges, line opacity drops to 0.18 (vs default 0.55) to prevent
		 *  overplotting. Default 800. */
		linkFadeThreshold?: number;
		/** MIG-025 §A.12 — sentinel: stamps true after the one-shot v5→v6
		 *  settings migration runs. Migration is quiet (no user dialog);
		 *  drops `lastMode` (no v6 equivalent) and stamps this flag.
		 *  `lastScope` stays in the file as a dead key (harmless; v5 still
		 *  reads it correctly during the B2 dual-mount window). */
		v6MigrationDone?: boolean;
	};
	/** AI/LLM integration preferences */
	ai?: {
		contextLines?: number;
		libraryAccess?: 'all' | 'active' | 'none';
		[key: string]: unknown;
	};
	/** MIG-021v3 V3-§10.A — Constellation Epistemic Content Engine
	 *  user preferences. Defaults preserve pre-V3-§10 behavior:
	 *  trail visibility = 'on_disagreement' (the trust-cal default
	 *  + r5.3 fall-through behavior); background scan = 'off'
	 *  (manual classification scan only). */
	cece?: {
		/** When to auto-expand the per-cataloger reasoning trail on
		 *  Source Review cards.
		 *  - 'always': open on every card
		 *  - 'on_disagreement' (default): open on Split / Strong-
		 *    Majority cards + during the first 50-review trust-cal
		 *    period; collapsed otherwise
		 *  - 'never': always collapsed (manual click required) */
		reasoningTrailVisibility?: 'always' | 'on_disagreement' | 'never';
		/** When to auto-trigger the classifier scan on notes that
		 *  don't yet have sources/content_type set.
		 *  - 'off' (default): manual scan only via Settings button
		 *  - 'on_save': fire classifier_suggest_for_note when the
		 *    NotePane debounced save fires (≥1500ms after typing)
		 *  - 'on_startup': trigger classifier_scan_start once per
		 *    app boot */
		backgroundScan?: 'off' | 'on_save' | 'on_startup';
	};
}

export const DEFAULT_SETTINGS: AppSettings = {
	showLineNumbers: true,
	readableLineLength: true,
	tabSize: 4,
	indentWithTabs: true,
	smartLists: true,
	autoPairBrackets: true,
	autoPairMarkdown: true,
	naturalLanguageTaskDates: true,
	spellcheck: false,
	showFloatingToolbar: true,
	foldHeading: true,
	foldIndent: true,
	indentationGuides: false,
	alwaysFocusNewTabs: true,
	propertiesInDocument: 'visible',
	defaultNoteLocation: 'root',
	defaultNoteFolder: '',
	defaultAttachmentFolder: '',
	linkFormat: 'shortest',
	autoUpdateLinks: true,
	useWikilinks: true,
	colourTypedLinks: true,
	showTypedLinkLabels: true,
	confirmDelete: true,
	trashDestination: 'system',
	trashFolderScope: 'library',
	titleAlignment: 'center',
	colorScheme: 'light',
	accentColor: '#7c3aed',
	activeThemeId: '',
	customThemes: [],
	styleOverride: {},
	styleSwatches: [],
	perScriptFonts: {},
	iconOverrides: {},
	interfaceFont: '',
	interfaceFontSize: 14,
	textFont: '',
	monoFont: '',
	fontSize: 17,
	scriptFonts: {},
	numeralStyle: 'arabic' as 'arabic' | 'hindi',
	dateFormat: 'DD/MM/YYYY',
	scriptDateFormats: {} as Record<string, string>,
	contextualDates: {} as Record<string, boolean>,
	fontMode: 'per-language',
	fontTheme: 'default',
	activeFontSetId: 'system',
	languageFontSets: { latin: 'system', arabic: 'arabic-modern', hebrew: 'hebrew', cjk: 'cjk' },
	customFontSets: [],
	primaryScript: 'latin',
	enableSecondaryScript: false,
	secondaryScript: 'arabic',
	enableScriptToolbar: true,
	scriptToolbarScripts: ['arabic', 'latin'],
	showDashboard: false,
	focus: 'none' as const,
	inboxFolder: '+',
	dailyNoteFormat: '%Y-%m-%d',
	dailyNoteFolder: '',
	dailyNoteTemplate: '',
	calendarPrimarySystem: 'gregorian' as const,
	calendarSecondarySystem: 'none' as const,
	calendarWeekStart: 0 as const,
	calendarShowWeekNumbers: true,
	calendarCorrections: {},
	calendarCalculationMode: 'astronomical' as const,
	calendarChineseYearStyle: 'sexagenary-gregorian' as const,
	calendarKoreanYearStyle: 'dangi' as const,
	calendarMonthNameStyle: 'native' as const,
	calendarStampCulturalDate: 'off' as const,
	templateFolder: 'Templates',
	folderTemplates: {},
	templateHotkeys: {},
	defaultProperties: [],
	autoUpdate: true,
	githubToken: '',
	security: {
		libraryEncryption: false,
		lockOnIdle: false,
		lockIdleTimeout: 5,
		lockPinHash: '',
		apiKeyProtection: false,
	},
	index: {
		expandCrossLanguage: false,
		semanticSearchEnabled: false,
		searchHistoryEnabled: false,
	},
	skyView: {
		nodeSize: 1.5,
		labelVisibility: 'hover' as const,
		labelFontSize: 12,
		linkThickness: 1,
		repelForce: 80,
		linkForce: 0.05,
		linkDistance: 30,
		showOrphans: true,
		colorByLibrary: true,
	},
	safeBootMode: false,
	perNoteLinkQueries: true,
	noteSummariesEnabled: false,
	noteTitleSummaryEnabled: false,
	enabledFeatures: {
		dailyNotes: true,
		templates: true,
		skyView: true,
		backlinks: true,
		outgoingLinks: true,
		tags: true,
		pagePreview: true,
		search: true,
		quickSwitcher: true,
		commandPalette: true,
		wordCount: true,
		workspaces: true,
		index: true,
		semanticSearch: false,
		notesNavigator: true,
		orgChart: true,
		aiSkills: true,
		secondScreen: true,
		constellationMap: false,
		constellationSight: true,
		// MIG-018 (PJ-038): v3 default-on once SIGHT_V3_ENABLED flips
		// to true in §1F (after Boss-test passes). Until then the
		// SIGHT_V3_ENABLED const short-circuits the gate regardless
		// of this user setting, mirroring v2's MIG-017 disable pattern.
		constellationSightV3: true,
		// MIG-025 §A.7: v6 default-on once SIGHT_V6_ENABLED flips
		// to true at the §A.14 ship gate (after Eisa's Boss-test of
		// Sight v6.0 passes). Until then the SIGHT_V6_ENABLED const
		// short-circuits the gate regardless of this user setting.
		constellationSightV6: true,
		emojiIconPicker: true,
		inspector360: true,
		// MIG-039: The Cataloger ships ON by default (first Core Plug-in).
		cece: true,
		// MIG-074: CCS ships ON by default (peer of CNS / the Cataloger).
		ccs: true,
	},
	// MIG-018 §1D + MIG-019 §2B/§2C/§2E: Sight v3 settings.
	// projection: Lambert (equal-area) by default.
	// showMilkyWay: TRUE by default.
	// calendarSystems: ['gregorian'] default per Eisa's §11 Q3.
	// alwaysOnLabels: false default per Eisa's §11 Q6 (hover/select-only).
	sight: {
		projection: 'lambert',
		showMilkyWay: true,
		calendarSystems: ['gregorian'],
		alwaysOnLabels: false,
		// MIG-024 §1 — Sight v5 mode + scope persistence. Defaults match
		// Concept Paper v3.1 §6.1 (R = Regions = lowest-cognitive-load
		// first-launch mode) + D-V3 universe-default scope.
		lastMode: 'R',
		lastScope: 'universe',
		// MIG-025 §A.11 — Sight v6 tour default false: first-ever Sight
		// v6 open shows the orientation overlay (Concept Paper §11
		// invariant 10). User dismissal sets this true; Help → Sight
		// tour clears it back to false.
		tourSeen: false,
		// MIG-025 §A.12 — Sight v6 v6.0 defaults per Concept Paper.
		// Default-simple chrome (extended=false) + Aristotelian register
		// (the explicit Western-classical default per §4.1.1).
		// Hex-bin / link-fade thresholds match Concept Paper §2.2/§3.4.
		// §B.10-fix-1 (2026-05-16): `extended` was named `proMode` until
		// Eisa's cycle-1 review; migration in applyParsedSettings carries
		// existing users' values forward.
		extended: false,
		activeTradition: 'aristotelian',
		// MIG-026 Phase β default favorites: the 4 production-polish
		// traditions currently shipping. User can re-order via chip
		// dropdown pin/unpin.
		favoriteTraditions: ['aristotelian', 'pramana', 'masadir', 'polanyi'],
		hexBinThreshold: 5000,
		linkFadeThreshold: 800,
		// v6MigrationDone left undefined intentionally — the §A.12
		// migration sets it to true on first run. Existing v5-installed
		// Universes will run the migration once on first v6.0 boot.
	},
	customShortcuts: {},
	linkPills: {
		fill: {
			supports:       '#4A9EFF',
			contradicts:    '#FF4A4A',
			causes:         '#FF8C42',
			exemplifies:    '#4AFF88',
			generalizes:    '#A44AFF',
			'derives-from': '#FFD700',
			'part-of':      '#AAAAAA',
			associative:    '#888888',
			// MIG-022 §A.2 (D-A1.β) — `supersedes` color: slate
			// blue-gray, distinct from all eight existing pills,
			// suggesting "this replaces something older". User can
			// override via Settings → Link pills color picker.
			supersedes:     '#5B7A8A',
		},
		text: {
			supports:       '#ffffff',
			contradicts:    '#ffffff',
			causes:         '#ffffff',
			exemplifies:    '#000000',
			generalizes:    '#ffffff',
			'derives-from': '#000000',
			'part-of':      '#ffffff',
			associative:    '#ffffff',
			supersedes:     '#ffffff',  // MIG-022 §A.2 — white text on slate-blue fill
		},
		shape: { radius: 10, height: 20, fontWeight: 700 },
	},
	linkLifecycle: {
		decayEnabled: true,
		halfLifeDays: 60,
	},
	panelPlacements: {
		// "Nervous system" default: the two link panels flank the note.
		backlinks: 'left-of-note',
		outgoing:  'right-of-note',
		// Everything else keeps its current right-sidebar home until Tier 2
		// drag-and-drop lands and the user can rearrange freely.
		properties: 'right-sidebar',
		tags:       'right-sidebar',
		sky:        'right-sidebar',
		tasks:      'right-sidebar',
		calendar:   'right-sidebar',
		health:     'right-sidebar',
		provenance: 'right-sidebar',
		review:     'right-sidebar',
		// MIG-074 §D — 'links' (the Link Dashboard tab) retired into CCS; a
		// stored user value for it stays inert via the spread-merge below.
		inspector360: 'right-sidebar',
	},
	leftOfNoteWidth: 280,
	rightOfNoteWidth: 280,
	// MIG-021v3 V3-§10.A — CECE user preferences. Defaults preserve
	// pre-V3-§10 behavior: trail visibility = 'on_disagreement' (the
	// trust-cal default + r5.3 fall-through behavior); background
	// scan = 'off' (manual classification scan only). These defaults
	// are the contract every appSettings.cece consumer reads via `??`.
	cece: {
		reasoningTrailVisibility: 'on_disagreement',
		backgroundScan: 'off',
	},
};

/** Shared metadata for the nine typed-link names — used by Settings UI
 *  and the two panels so the iteration order is stable. Kept in sync
 *  with `KNOWN_LINK_TYPES` (Rust: tension.rs / strata.rs / libraries.rs)
 *  and `TYPED_LINK_TYPES` in livePreview.
 *
 *  MIG-022 §A.2 (D-A1.β, 2026-05-11) — `supersedes` added as the 9th.
 *  Per the gap analysis §6.1, "this note replaces an earlier stance"
 *  is a first-class typed relationship, not a flat YAML scalar. */
/** Link-type names for Settings iteration — the registry's typed acts (8 seeds
 *  + custom, in canonical order) plus the null `associative`. MIG-067 §D: was a
 *  hardcoded 9-tuple; now registry-derived so custom types appear. Read at call
 *  time (the registry is boot-seeded; a vocabulary edit re-seeds it). */
export function linkTypeNames(): string[] {
	return [...getLinkTypes().map((t) => t.id), 'associative'];
}

export const appSettings = writable<AppSettings>(DEFAULT_SETTINGS);

/**
 * §A.14 fix-5 (Boss-test #7) — single source of truth for parsed-
 * settings hydration. Both `loadSettings()` (per-call path) AND the
 * `+layout.svelte` boot-bundle handler call this so they cannot drift.
 *
 * Pre-fix: `loadSettings()` had a comprehensive merge with all nested
 * defaults (sight, cece, panelPlacements, index, etc.) AND the §A.12
 * v5→v6 migration block. The boot-bundle path at +layout.svelte:1880
 * had its OWN inline merge that was missing four nested-key merges
 * AND missing the migration entirely. Result: on every Boss user's
 * machine, the v6 defaults never landed in-memory and the migration
 * never ran. Surfaced 2026-05-14 by Eisa's screenshot showing only
 * 4 sight keys persisted.
 */
export function applyParsedSettings(parsed: Record<string, unknown>): void {
	if (!parsed || Object.keys(parsed).length === 0) return;

	// Migrate: old default nodeSize was 4, new default is 1.5
	const savedSkyView = (parsed.skyView as Record<string, unknown>) || {};
	if (savedSkyView.nodeSize === 4) savedSkyView.nodeSize = 1.5;

	// Boss 2026-06-14 — 'permanent' is no longer a valid delete destination
	// (deletes are always recoverable). Migrate any saved 'permanent' to System trash.
	if (parsed.trashDestination === 'permanent') parsed.trashDestination = 'system';

	appSettings.set({
		...DEFAULT_SETTINGS,
		...(parsed as Partial<AppSettings>),
		skyView: { ...DEFAULT_SETTINGS.skyView, ...savedSkyView },
		index: { ...DEFAULT_SETTINGS.index, ...((parsed.index as Record<string, unknown>) || {}) },
		security: { ...DEFAULT_SETTINGS.security, ...((parsed.security as Record<string, unknown>) || {}) },
		// MIG-038 (2026-05-19) — Map DISABLED pre-Wings. The trailing
		// `constellationMap: false` override force-disables Constellation
		// Map even for users who previously enabled it (the dock button +
		// command-palette entry both gate on enabledFeatures.constellationMap
		// === true). Sight is disabled separately via SIGHT_V6_ENABLED=false
		// in sight/engine.ts. Both keep their code intact for later
		// detachment into standalone Constellation Plugins under the
		// "Constellation Wings" sub-project. Reversible: delete the override.
		enabledFeatures: { ...DEFAULT_SETTINGS.enabledFeatures, ...((parsed.enabledFeatures as Record<string, boolean>) ?? (parsed.enabledPlugins as Record<string, boolean>) ?? {}), constellationMap: false },
		customShortcuts: { ...((parsed.customShortcuts as Record<string, string>) || {}) },
		// Merge panel placements so existing users get new-panel defaults
		// automatically when we add a new PanelId in a future release.
		panelPlacements: { ...DEFAULT_SETTINGS.panelPlacements, ...((parsed.panelPlacements as Record<PanelId, PanelSlot>) || {}) },
		// MIG-021v3 V3-§11 audit fix — preserve cece sub-keys across
		// settings round-trip. Without this merge, a user who sets
		// reasoningTrailVisibility but leaves backgroundScan implicit
		// would get the saved object overwriting the default for
		// any newly-added sub-keys in a future release.
		cece: { ...DEFAULT_SETTINGS.cece, ...((parsed.cece as Record<string, unknown>) || {}) },
		// MIG-024 §1 — same V3-§11 AT RISK pattern for sight: a
		// user who sets projection but leaves lastMode implicit
		// would otherwise get the saved object overwriting the
		// new lastMode/lastScope defaults.
		sight: { ...DEFAULT_SETTINGS.sight, ...((parsed.sight as Record<string, unknown>) || {}) },
	});

	// ── MIG-070 §C-polish Item B — styleSwatches shape upgrade ──
	// Legacy palettes stored bare hex strings; the named-swatch UI needs { hex, name }.
	// Coerce any string entries on load (idempotent: silent once every entry is an object).
	const rawSwatches = parsed.styleSwatches;
	if (Array.isArray(rawSwatches) && rawSwatches.some((sw) => typeof sw === 'string')) {
		appSettings.update((s) => ({
			...s,
			styleSwatches: (s.styleSwatches as unknown as Array<string | StyleSwatch>).map((sw) =>
				typeof sw === 'string' ? { hex: String(sw).toLowerCase(), name: '' } : sw,
			),
		}));
		saveSettings();
	}

	// ── MIG-071 §D — remove the theme subsystem: wipe all theme data (Eisa, 2026-06-07) ──
	// The Appearance theme layer (built-ins + custom themes) is retired; the Style Setter is the sole
	// styling home. Clear every theme + reset the active pointers to the plain-default base. The
	// Setter's saved Styles (style-presets.json), styleOverride (current look), and styleSwatches are
	// NOT touched. Self-limiting + idempotent: only fires while theme data exists; after the wipe the
	// fields are empty so it never re-runs (and there's no UI to create new themes). No backup (Eisa).
	if ((parsed.customThemes as unknown[] | undefined)?.length || parsed.activeThemeId) {
		appSettings.update((s) => ({ ...s, customThemes: [], activeThemeId: '' }));
		saveSettings();
	}

	// ── §B.10-fix-1 — proMode → extended rename migration ───────
	// 2026-05-16, Eisa cycle-1: rename for honest naming ("Pro"
	// overpromised). One-shot per-user; idempotent (subsequent loads
	// have no proMode key in parsed.sight so the branch is silent).
	// Order: runs BEFORE the v6MigrationDone block below so that any
	// existing extended value already set by the user (rare race
	// condition) takes precedence over a legacy proMode read.
	const sightSnapshotForRename = (parsed.sight as Record<string, unknown>) || {};
	if ('proMode' in sightSnapshotForRename) {
		appSettings.update((s) => {
			const nextSight = { ...s.sight } as Record<string, unknown>;
			// Only copy if extended wasn't already explicitly set (avoid
			// overwriting a value the user set on a later build).
			if (!('extended' in sightSnapshotForRename) || nextSight.extended === undefined) {
				nextSight.extended = sightSnapshotForRename.proMode as boolean;
			}
			// Drop the legacy key; saveSettings persists the deletion.
			delete nextSight.proMode;
			return { ...s, sight: nextSight as typeof s.sight };
		});
		saveSettings();
	}

	// ── §B.11 — lastMode dead-key cleanup ───────────────────────
	// 2026-05-16 (Eisa observation while verifying §B.10-fix-1
	// migration): Eisa's settings.json still had `lastMode: "R"`
	// despite v6MigrationDone=true. The §A.12 migration was supposed
	// to delete lastMode on first v6 boot, but its `if (!v6MigrationDone)`
	// gate means it never re-runs for users whose v6MigrationDone was
	// stamped without the deletion succeeding (likely a partial write
	// in an early v6 build). This idempotent cleanup runs on every
	// load: deletes lastMode unconditionally if present (v6 ignores
	// the field entirely; no v6 code path reads it). saveSettings
	// persists the deletion. Branch silent on subsequent loads.
	if ('lastMode' in sightSnapshotForRename) {
		appSettings.update((s) => {
			const nextSight = { ...s.sight } as Record<string, unknown>;
			delete nextSight.lastMode;
			return { ...s, sight: nextSight as typeof s.sight };
		});
		saveSettings();
	}

	// ── MIG-026 Phase 0 — K1 rename: activeRegister → activeTradition ─
	// 2026-05-17, MIG-026 Phase 0 (K1 full rename "register" →
	// "tradition" throughout). Legacy users have `activeRegister: <id>`
	// in settings.json. This block copies it to `activeTradition` and
	// deletes `activeRegister`. Runs BEFORE the dignaga/ishraqi
	// migration blocks below so those see the snapshot's
	// `activeRegister` field (still present in snapshot since the
	// snapshot was read once at top), but write to the new
	// `activeTradition` field name.
	// Idempotent: subsequent loads have no `activeRegister` key in
	// the snapshot so the branch is silent. saveSettings persists.
	if ('activeRegister' in sightSnapshotForRename) {
		appSettings.update((s) => {
			const nextSight = { ...s.sight } as Record<string, unknown>;
			if (!('activeTradition' in sightSnapshotForRename) || nextSight.activeTradition === undefined) {
				nextSight.activeTradition = sightSnapshotForRename.activeRegister;
			}
			delete nextSight.activeRegister;
			return { ...s, sight: nextSight as typeof s.sight };
		});
		saveSettings();
	}

	// ── §C.1-fix-1 — Dignāga tradition exclusion migration ──────
	// 2026-05-16, Eisa Stage 2 review of §C.1: "don't include the
	// 'Dignāga' at all in any of Constellation functions". The
	// 'dignaga' literal is removed from TraditionId and from the
	// activeTradition union above. This block rewrites any persisted
	// activeRegister: 'dignaga' (could exist if a user clicked the
	// Dignāga chip during §C.1 testing before this fix shipped) back
	// to 'aristotelian' so the chip's blue-dot indicator resolves to
	// a valid tradition entry. Idempotent: subsequent loads have no
	// 'dignaga' value so the branch is silent. saveSettings persists
	// the rewrite to disk.
	// MIG-026 Phase 0: this block now WRITES to `activeTradition`
	// (the renamed field) but READS from the snapshot's still-present
	// legacy `activeRegister` key. The rename block above already
	// migrated the field name in the store.
	if (sightSnapshotForRename.activeRegister === 'dignaga' ||
	    sightSnapshotForRename.activeTradition === 'dignaga') {
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, activeTradition: 'aristotelian' },
		}));
		saveSettings();
	}

	// ── §C.4-religious-rule — Ishrāqī register exclusion migration ─
	// 2026-05-16, Eisa direction post-§C.4: new top-principal rule —
	// "when dealing with religious references, no non-Abrahamic; for
	// Islamic, Sunni only." The Ishrāqī tradition (Suhrawardi 1154–
	// 1191) was overwhelmingly absorbed into Twelver Shīʿī ḥikma
	// (Mulla Sadra, Sabzavari, modern Qom curriculum) — failing the
	// Sunni-only restriction — and is fundamentally religious-mystical
	// rather than philosophical-epistemological. Same shape as the
	// Dignāga block above: if a user clicked the Ishrāqī chip during
	// v6.2-pre-religious-rule testing, rewrite back to 'aristotelian'.
	// Idempotent: subsequent loads have no 'ishraqi' value so the
	// branch is silent. saveSettings persists the rewrite to disk.
	// MIG-026 Phase 0: also accepts persisted 'ishraqi' under the new
	// `activeTradition` key (edge case: user downgraded after migration
	// then re-upgraded).
	if (sightSnapshotForRename.activeRegister === 'ishraqi' ||
	    sightSnapshotForRename.activeTradition === 'ishraqi') {
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, activeTradition: 'aristotelian' },
		}));
		saveSettings();
	}

	// ── §A.12 — Sight v5 → v6 settings migration ────────────────
	// One-shot, quiet (no user dialog). Stamps v6MigrationDone=true
	// to prevent re-running. Per Architect Option G1 (locked):
	//   - lastMode is dropped (v6 has no modes; the field has no
	//     v6 equivalent). Removed from in-memory state; saveSettings
	//     persists the deletion to disk on the next debounced write.
	//   - lastScope is intentionally LEFT IN THE FILE as a dead key.
	//     v6 ignores it; v5 still reads it correctly during the B2
	//     dual-mount window (Phases 1-3). §D.6 (Phase 4) deletes
	//     the field along with the v5 module set.
	// Idempotent: if v6MigrationDone is already true (from a prior
	// session), the block short-circuits.
	const sightSnapshot = (parsed.sight as Record<string, unknown>) || {};
	if (!sightSnapshot.v6MigrationDone) {
		appSettings.update((s) => {
			const nextSight = { ...s.sight, v6MigrationDone: true };
			// Drop lastMode if present (no v6 equivalent).
			delete (nextSight as Record<string, unknown>).lastMode;
			return { ...s, sight: nextSight };
		});
		saveSettings();
	}
}

export async function loadSettings() {
	try {
		const parsed = await invoke<Record<string, unknown>>('read_universe_settings');
		applyParsedSettings(parsed);
	} catch { /* ignore */ }
}

let saveSettingsTimer: ReturnType<typeof setTimeout> | null = null;

export function saveSettings() {
	if (saveSettingsTimer) clearTimeout(saveSettingsTimer);
	saveSettingsTimer = setTimeout(() => {
		invoke('save_universe_settings', { settings: get(appSettings) }).catch(e => console.error('[save] settings failed:', e));
	}, 300);
}

export function updateSettings(partial: Partial<AppSettings>) {
	appSettings.update(s => ({ ...s, ...partial }));
	saveSettings();
	// Notify second screen of settings change
	emit('screen:settings-changed', get(appSettings)).catch(() => {});
}

/** MIG-070 §C — set one per-Universe style override (CSS-var → value) and persist.
 *  Debounced via saveSettings (300ms) so dragging a Style-Setter slider can't hammer IPC. */
export function setStyleOverride(key: string, value: string) {
	appSettings.update(s => ({ ...s, styleOverride: { ...(s.styleOverride ?? {}), [key]: value } }));
	saveSettings();
	emit('screen:settings-changed', get(appSettings)).catch(() => {});
}

/** MIG-070 §C — remove one per-Universe style override (revert to the theme look for that var). */
export function clearStyleOverride(key: string) {
	appSettings.update(s => {
		if (!(s.styleOverride && key in s.styleOverride)) return s;
		const next = { ...s.styleOverride };
		delete next[key];
		return { ...s, styleOverride: next };
	});
	saveSettings();
	emit('screen:settings-changed', get(appSettings)).catch(() => {});
}

/** MIG-070 §C — merge many overrides at once (the Style Setter's "Apply"): ONE settings
 *  update + one save + one emit, so applying a whole draft doesn't re-run the apply effect
 *  per key. */
export function mergeStyleOverride(partial: Record<string, string>) {
	appSettings.update(s => ({ ...s, styleOverride: { ...(s.styleOverride ?? {}), ...partial } }));
	saveSettings();
	emit('screen:settings-changed', get(appSettings)).catch(() => {});
}

/** MIG-070 §C — set/clear one per-script font (empty family clears → engine/system default). */
export function setPerScriptFont(script: string, family: string) {
	appSettings.update(s => {
		const next = { ...(s.perScriptFonts ?? {}) };
		if (family) next[script] = family; else delete next[script];
		return { ...s, perScriptFonts: next };
	});
	saveSettings();
	emit('screen:settings-changed', get(appSettings)).catch(() => {});
}

/** MIG-070 §C — clear ALL per-Universe overrides (the Style Setter's "Reset" → pure theme look). */
export function clearAllStyleOverride() {
	appSettings.update(s => ({ ...s, styleOverride: {} }));
	saveSettings();
	emit('screen:settings-changed', get(appSettings)).catch(() => {});
}

/** MIG-070 §C Option E — a transient, IN-MEMORY top layer for the Style Setter's LIVE preview.
 *  While the Setter is open in live mode, the draft is pushed here (NOT persisted); the shared
 *  `+layout` apply `$effect` merges it LAST (above `styleOverride`), so the REAL app restyles live.
 *  Keep promotes it to `styleOverride`; Discard/close clears it (the same `$effect` reverts). Being
 *  in-memory means a slider drag does ZERO IPC/disk (Rule 3) and the saved look is never touched
 *  until Keep — and it rides the existing single apply path, so there is no second-writer race. */
export const liveStyleDraft = writable<Record<string, string>>({});
export function setLiveStyleDraft(vars: Record<string, string>) { liveStyleDraft.set({ ...vars }); }
export function clearLiveStyleDraft() { liveStyleDraft.set({}); }

/** MIG-070 §C — add a hex colour to the saved palette (deduped by hex, most-recent first, capped).
 *  §C-polish Item B: an optional name can be supplied; auto-saved picks come in unnamed. */
export function addStyleSwatch(hex: string, name = '') {
	const h = (hex || '').trim().toLowerCase();
	if (!/^#[0-9a-f]{6}$/.test(h)) return;
	appSettings.update(s => {
		const cur = s.styleSwatches ?? [];
		if (cur.some(sw => sw.hex === h)) return s;
		return { ...s, styleSwatches: [{ hex: h, name: name.trim() }, ...cur].slice(0, 24) };
	});
	saveSettings();
}

/** MIG-070 §C — remove a swatch from the saved palette (matched by hex). */
export function removeStyleSwatch(hex: string) {
	const h = (hex || '').trim().toLowerCase();
	appSettings.update(s => ({ ...s, styleSwatches: (s.styleSwatches ?? []).filter(sw => sw.hex !== h) }));
	saveSettings();
}

/** MIG-070 §C polish (Item B) — name (or rename) a saved swatch, matched by hex. Blank clears it. */
export function renameStyleSwatch(hex: string, name: string) {
	const h = (hex || '').trim().toLowerCase();
	appSettings.update(s => ({ ...s, styleSwatches: (s.styleSwatches ?? []).map(sw => sw.hex === h ? { ...sw, name: name.trim() } : sw) }));
	saveSettings();
}

export function updateSecuritySettings(partial: Partial<AppSettings['security']>) {
	appSettings.update(s => ({
		...s,
		security: { ...s.security, ...partial }
	}));
	saveSettings();
}

// ─── Workspaces ───
export interface WorkspaceLayout {
	leftSidebarOpen: boolean;
	leftSidebarWidth: number;
	rightSidebarOpen: boolean;
	rightSidebarTab: string;
	rightSidebarWidth: number;
	/** Tier 1 panel-placement snapshot. Workspaces that predate this field
	 *  load without it; restoring such a workspace leaves the current
	 *  panelPlacements untouched. Saving writes the current map. */
	panelPlacements?: Record<PanelId, PanelSlot>;
	/** Flanking column widths. Resizable via drag handles in Tier 1b; for
	 *  now they hold the defaults so the schema can accept them when the
	 *  drag handle ships without a migration. */
	leftOfNoteWidth?: number;
	rightOfNoteWidth?: number;
}

export interface WorkspaceSecondScreen {
	open: boolean;
	mode: string;
	linkedBrowsing: boolean;
	tabs: { path: string; libraryName: string; libraryColor: string }[];
	activeTabPath: string | null;
}

export interface Workspace {
	id: string;
	name: string;
	tabs: { path: string; libraryName: string; libraryColor: string }[];
	activeTabPath: string | null;
	splitActive: boolean;
	splitDir: SplitDirection;
	timestamp: number;
	layout?: WorkspaceLayout;
	secondScreen?: WorkspaceSecondScreen;
}

export const workspaces = writable<Workspace[]>([]);

export async function loadWorkspaces() {
	try {
		const data = await invoke<unknown[]>('read_universe_workspaces');
		if (data && Array.isArray(data) && data.length > 0) workspaces.set(data as Workspace[]);
	} catch { /* ignore */ }
}

function persistWorkspaces() {
	invoke('save_universe_workspaces', { workspaces: get(workspaces) }).catch(e => console.error('[save] workspaces failed:', e));
}

export function saveWorkspace(name: string, layout?: WorkspaceLayout, secondScreenState?: WorkspaceSecondScreen) {
	const tabs = get(openTabs).map(t => ({
		path: t.path,
		libraryName: t.libraryName,
		libraryColor: t.libraryColor,
	}));
	const activeTab = get(activeTabId);
	const currentTab = get(openTabs).find(t => t.id === activeTab);
	const ws: Workspace = {
		id: `ws_${Date.now()}`,
		name,
		tabs,
		activeTabPath: currentTab?.path ?? null,
		splitActive: get(splitActive),
		splitDir: get(splitDirection),
		timestamp: Date.now(),
		layout,
		secondScreen: secondScreenState,
	};

	workspaces.update(list => {
		// Replace if same name exists
		const filtered = list.filter(w => w.name !== name);
		return [...filtered, ws];
	});
	persistWorkspaces();
}

export async function restoreWorkspace(ws: Workspace): Promise<{ layout?: WorkspaceLayout; secondScreen?: WorkspaceSecondScreen }> {
	// Close all current tabs
	openTabs.set([]);
	activeTabId.set(null);
	focusedTabId.set(null);

	// Open saved tabs
	for (const saved of ws.tabs) {
		try {
			await openNoteTab(saved.path, saved.libraryName, saved.libraryColor);
		} catch { /* file may not exist anymore */ }
	}

	// Restore active tab
	if (ws.activeTabPath) {
		const tabs = get(openTabs);
		const match = tabs.find(t => t.path === ws.activeTabPath);
		if (match) {
			activeTabId.set(match.id);
			focusedTabId.set(match.id);
		}
	}

	// Restore split state
	splitActive.set(ws.splitActive);
	splitDirection.set(ws.splitDir);

	// Return layout and second screen state for the caller to apply
	return { layout: ws.layout, secondScreen: ws.secondScreen };
}

export function deleteWorkspace(id: string) {
	workspaces.update(list => list.filter(w => w.id !== id));
	persistWorkspaces();
}

// ─── Clipboard image paste ───
export async function saveClipboardImage(libraryPath: string, imageData: string): Promise<string> {
	return await invoke('save_clipboard_image', { libraryPath, imageData });
}

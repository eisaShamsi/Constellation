/**
 * Library (Universe) state management.
 */

import { writable, derived, get } from 'svelte/store';
import { tick } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { normalizePathKey, subscribeSkipInitial } from '$lib/utils';
import { t } from '$lib/i18n'; // PJ-102c — the localized recovered-copy suffix (leaf module, no cycle)
import { getLinkTypes, isLinkTypeValue } from './linkTypeRegistry';
import * as CL from './collectionsLogic'; // MIG-092 — pure Collections reducers
// MIG-076 §C — single content ownership. noteModel/noteSession use this
// module's parseFrontmatter/buildFullContent (hoisted fn declarations, so the
// import cycle is eval-safe) and are used here only inside functions (never at
// module init). The model is the save source when SINGLE_OWNERSHIP is on.
import { editProps as editNoteProps, editBody as editNoteBody, bodyForView as modelBodyForView, replaceContent as replaceContentInModel, close as closeNoteModel, repath as repathNoteModel, open as openNoteModel, save as saveNoteSession, isDirty as isNoteDirty, flushIfDirty as flushOutgoingModel, externalChange as externalChangeNoteModel, recoveredFromNet as markModelRecoveredFromNet, setDiskBaseline as setModelDiskBaseline, type SaveEnv, type FlushResult } from '$lib/editor/noteSession';
import { compose as composeNoteModel, getModel as getNoteModel, diskDiffersFromBaseline } from '$lib/editor/noteModel';
import { splitFrontmatter, composeFrontmatter, STRUCTURED_LIST_KEYS } from '$lib/editor/yamlDoc'; // G4 Phase 3 — byte-perfect round-trip write
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
	/** MIG-091 §A — created + size for the File Explorer's richer sort. */
	created?: number | null;
	size?: number | null;
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

export type PropertyType = 'text' | 'number' | 'date' | 'datetime' | 'list' | 'link' | 'checkbox' | 'nested-object-list' | 'nested-map';

export interface FrontmatterProperty {
	key: string;
	value: string;
	type: PropertyType;
	listItems?: string[];
	/** PJ-136 — for the `nested-map` type, the child lines VERBATIM. The legacy
	 *  `reconstructFrontmatter` re-emits them unchanged so `buildFullContent`'s
	 *  `tab.content` cache stays lossless; a cache that dropped them re-parsed the key
	 *  as ordinary text, and that lie is what let `composeFrontmatter` delete the block. */
	nestedRaw?: string[];
	/** PJ-136 — for the `nested-map` type, the child keys in document order
	 *  (`['title', 'author', 'year']`). The panel renders them as the row's
	 *  read-only summary; the authoritative bytes remain in the CST, which
	 *  `composeFrontmatter` never rewrites for this type. */
	nestedKeys?: string[];
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
// The set lives in yamlDoc, next to the serializer that makes "losslessly editable"
// true for these keys — see STRUCTURED_LIST_KEYS there. Aliased so the many call
// sites below read unchanged.
const IKHTILAF_KEYS = STRUCTURED_LIST_KEYS;

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

// APP-KILLER #2 (2026-07-08) — the nav-flush safety valve. When true, a navigation
// that reuses a tab (wikilink / file-tree click, Alt+←/→) flushes the OUTGOING dirty
// model to disk through the durability gate BEFORE re-seeding the tab's model, so
// in-debounce edits are never silently discarded. One-line revert to the prior
// (defective) behavior. Frontend-only; models are ephemeral, so toggling is safe.
const NAV_FLUSH_ENABLED = true;

// APP-KILLER #2 (B1, Boss-ruled 2026-07-08) — one path → one tab. When true, opening a
// note already open in ANY tab (incl. a background tab, and regardless of Ctrl+click)
// activates that tab instead of minting a SECOND tab + SECOND model for the same path
// (two independent models for one path = a save from one clobbers the other's disk edits).
// Trade-off: the same note can no longer be open in two panes at once. One-line revert.
const DEDUP_ALL_TABS_ENABLED = true;

// PJ-070 (2026-07-12) — the watcher external-change ADOPT valve. When true, an external .md
// edit to an OPEN note (git-pull / Syncthing / Obsidian) is ADOPTED into the single-ownership
// note model (freshness-gated: a clean model adopts + remounts; a dirty model keeps its unsaved
// work and the incoming edit is preserved to a `.conflict` sidecar) instead of only updating
// tab.content — so the next keystroke can never silently overwrite the external edit.
// WARNING: false = content-only update for BOTH ingress paths (the watcher flush AND the
// second-screen onNoteSaved) = the KNOWN Recipe-O clobber (external edit lost on the next
// keystroke). It is a one-line ROLLBACK lever, NOT a safe steady state — and note it is NOT
// byte-identical to pre-PJ-070 (the old onNoteSaved adopted the model without a remount); for an
// exact code-level rollback use `git revert` of the PJ-070 commit, not this flag.
const WATCHER_ADOPT_ENABLED = true;

// ─── Centralized save with lock ───
const saveLocks = new Map<string, boolean>();
const recentWrites = new Map<string, number>();

/** Mark a file as recently written so the file watcher ignores it. */
export function markRecentWrite(filePath: string) {
	recentWrites.set(filePath, Date.now());
	setTimeout(() => recentWrites.delete(filePath), 2000);
}

/** One entry in the write-ahead net.
 *
 *  `snapshot` (PJ-181) records WHY the entry exists — the thing it never recorded before:
 *
 *    snapshot !== true  → the net holds work the disk never had (a real recovery copy)
 *    snapshot === true  → `content` was ALREADY DURABLE on disk when it was stashed
 *
 *  Without it the entry recorded WHAT it held and never WHY, and `resolveNoteContent` was
 *  asked to tell a recovery copy from a stale snapshot using information that was never
 *  written down. It answered with `cid_cn`, which is the note's IDENTITY and says nothing
 *  about its VERSION — so the copy left by merely VIEWING a note beat a newer file.
 *
 *  A flag rather than a copy of the baseline bytes, deliberately: for a snapshot the
 *  baseline IS `content`, so the bytes would be stored twice. This blob is persisted to
 *  localStorage, is never pruned or capped, and `setWriteAhead` swallows a quota exception
 *  with an empty catch — so doubling every viewed note's entry would have silently pushed
 *  the whole net toward the point where it stops persisting at all, which is exactly the
 *  crash recovery it exists to provide.
 *
 *  Optional on purpose: a legacy localStorage entry has no `snapshot`, and its absence
 *  means "cannot prove this is a snapshot" → treated as real work, which is the pre-PJ-181
 *  behaviour and the direction that never discards the user's edits.
 */
type WriteAheadEntry = { content: string; cursorPos: number; scrollTop: number; snapshot?: boolean };

/** Write-ahead buffer: holds content/cursor/scroll that hasn't been written to disk yet.
 *  When opening a note, check this first — it's synchronous and always has the latest data. */
const writeAheadBuffer = new Map<string, WriteAheadEntry>();
/** localStorage key for the crash-safe wab backup. Single source of truth so
 *  the five readers/writers can't drift apart on a typo. */
const WAB_LS_KEY = 'constellation-wab';

/**
 * @param snapshot PJ-181 — pass `true` when `content` is ALREADY DURABLE on disk (a mere
 *        view, nothing typed). Such an entry recovers nothing, and must never outrank a
 *        newer file on reopen. Omit it for a real recovery copy (net-before-write, or any
 *        stash of work not yet on disk) — omission is the safe default.
 */
export function setWriteAhead(filePath: string, content: string, cursorPos: number, scrollTop: number, snapshot?: boolean) {
	const entry: WriteAheadEntry = snapshot
		? { content, cursorPos, scrollTop, snapshot: true }
		: { content, cursorPos, scrollTop };
	writeAheadBuffer.set(filePath, entry);
	/* Also persist to localStorage as crash-safe backup (survives app restart).
	   This is synchronous and fast for single-note content. */
	try {
		const existing = JSON.parse(localStorage.getItem(WAB_LS_KEY) || '{}');
		existing[filePath] = entry;
		localStorage.setItem(WAB_LS_KEY, JSON.stringify(existing));
	} catch {}
}

export function getWriteAhead(filePath: string): WriteAheadEntry | undefined {
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
 * Save-Durability (2026-07-08) — compare-and-clear the write-ahead net. A completed
 * save clears the net ONLY when the buffered content still equals what it wrote; if
 * a newer edit (concurrent typing, or a second window sharing the localStorage
 * origin) has already replaced the buffer, that newer net is left intact. Prevents
 * an older resolved write from wiping a newer in-flight write's recovery buffer (INV-3).
 */
export function clearWriteAheadIf(filePath: string, content: string) {
	const cur = getWriteAhead(filePath);
	if (cur && cur.content !== content) return; // a newer net exists — never clobber it
	clearWriteAhead(filePath);
}

/**
 * PJ-108 (APP-KILLER) — this store instance runs in a DISPLAY-ONLY window (the second screen).
 * Such a window never mounts a writable editor, so it must NEVER consume the shared crash-recovery
 * net when it opens a note: without a writable editor to re-stash it, consuming the net silently
 * destroys the ONLY copy of a main-window note's unsaved, save-failed edits (localStorage is shared
 * across same-origin windows). When set, EVERY openNoteTab in this window defaults to preserveNet —
 * the Solve-the-Class fix, so no individual SS call site (restore, list-click, wikilink) can leak.
 * Separate JS context per window → the main window's flag stays false. (`on` param lets tests reset.)
 */
let displayOnlyWindow = false;
export function setDisplayOnlyWindow(on: boolean = true) {
	displayOnlyWindow = on;
}
export function isDisplayOnlyWindow(): boolean {
	return displayOnlyWindow;
}

/**
 * Save-Durability (2026-07-08) — the "save health" surface. A save write that fails
 * (a `.md` momentarily locked by Syncthing/OneDrive/Defender, disk full, offline
 * drive) records a persistent, path-KEYED entry here — coalesced by construction, so
 * a failing drive at the 1.5 s debounce produces ONE entry per note, never a per-tick
 * storm. The banner (main + second-screen `+layout`) renders one row per entry; a
 * later successful save for that path auto-dismisses it. INV-5: no save failure is
 * ever silently swallowed.
 */
export const saveHealth = writable<Map<string, { name: string; error: string; since: number }>>(new Map());

export function reportSaveFailure(path: string, name: string, error: unknown) {
	saveHealth.update((m) => {
		const n = new Map(m);
		n.set(path, { name, error: String((error as { message?: string })?.message ?? error), since: Date.now() });
		return n;
	});
}

export function clearSaveFailure(path: string) {
	saveHealth.update((m) => {
		if (!m.has(path)) return m;
		const n = new Map(m);
		n.delete(path);
		return n;
	});
}

/**
 * PJ-070 — the "external conflict" surface. When a note's file is edited OUTSIDE Constellation
 * while its open model has UNSAVED local edits, the local work is kept (never clobbered) and the
 * incoming disk copy is preserved to a `.conflict` sidecar. Each conflict records one entry here,
 * KEYED BY SIDECAR PATH (unique per conflict), rendered as a banner row with a "Show copy" action.
 * Unlike saveHealth this is NOT a failure and does NOT auto-clear — a conflict is a decision the
 * user makes, so the row persists until they dismiss it (or reveal + resolve the copy).
 */
export const saveConflicts = writable<Map<string, { noteName: string; notePath: string; since: number }>>(new Map());

export function reportConflict(sidecarPath: string, noteName: string, notePath: string) {
	saveConflicts.update((m) => {
		const n = new Map(m);
		n.set(sidecarPath, { noteName, notePath, since: Date.now() });
		return n;
	});
}

export function dismissConflict(sidecarPath: string) {
	saveConflicts.update((m) => {
		if (!m.has(sidecarPath)) return m;
		const n = new Map(m);
		n.delete(sidecarPath);
		return n;
	});
}

/**
 * PJ-070 — the conflict hook `adoptExternalChangeIntoTabs` calls for a DIRTY tab whose file was
 * genuinely changed externally: persist the incoming disk copy to a `.conflict` sidecar (never
 * lost) and surface a banner row. Best-effort — a sidecar-write failure is logged, never thrown
 * (the local unsaved work is already safe in the model; this only preserves the REMOTE copy).
 */
export async function reportExternalConflict(notePath: string, noteName: string, diskContent: string): Promise<void> {
	try {
		const sidecarPath = await invoke<string>('write_conflict_sidecar', { notePath, diskContent });
		reportConflict(sidecarPath, noteName, notePath);
	} catch (e) {
		console.error('[PJ-070] failed to write conflict sidecar for', notePath, e);
	}
}

/**
 * Save-Durability (2026-07-08) — the one place a component/flush save site assembles
 * its `SaveEnv`. Wires the write-ahead net (set-before-write + compare-and-clear), the
 * save-health surface, and an optional `onSaved` hook for the component-owned
 * on-DURABLE-write side effects (reindex / embed / broadcast / CECE). No site
 * hand-rolls the mark-clean ordering again (INV-1) — they all go through
 * `noteSession.save()` with this env.
 */
export function standardSaveEnv(opts: {
	origin: string;
	name?: string;
	cursorPos?: number;
	scrollTop?: number;
	onSaved?: (path: string, content: string) => void;
}): SaveEnv {
	return {
		write: (p, c) => writeNote(p, c, opts.origin),
		setNet: setWriteAhead,
		clearNetIf: clearWriteAheadIf,
		cursorPos: opts.cursorPos,
		scrollTop: opts.scrollTop,
		onSuccess: ({ path, content }) => {
			clearSaveFailure(path);
			opts.onSaved?.(path, content);
		},
		onError: ({ path, error }) => reportSaveFailure(path, opts.name ?? path, error),
	};
}

/**
 * Save-Durability — retry a previously-failed save. The save-health banner's "Retry"
 * button and the ~10 s auto-retry timer both call this. If the note is still OPEN and
 * dirty, re-run the durability gate (a durable write clears its save-health entry via
 * `standardSaveEnv`'s onSuccess); if it is already clean, drop the stale entry. If the
 * note is CLOSED, its edit is preserved in the write-ahead net (restored on reopen),
 * so there is nothing to re-drive from here.
 */
export async function retrySaveFailure(path: string): Promise<void> {
	const tab = get(openTabs).find((t) => t.path === path);
	if (!tab) return; // closed — the write-ahead net holds it; restored on reopen
	if (!isNoteDirty(tab.id)) { clearSaveFailure(path); return; } // already clean
	await saveNoteSession(tab.id, path, standardSaveEnv({
		origin: 'retry_save',
		name: tab.name,
		onSaved: (savedPath) => {
			invoke('constellation_search_reindex', { notePath: savedPath, libraryName: tab.libraryName }).catch(() => {});
		},
	}), 'retry_save');
}

/** PJ-102c — make a recovered copy's FRONTMATTER own its NEW identity (body untouched):
 *  strip the cid_cn (a copy is a NEW note — the duplicate-cid_cn class; a fresh cid is
 *  injected by ensure_cid_cn on first open) AND retitle `title:` with the copy suffix —
 *  the tab label derives from the title, so an unretitled copy is indistinguishable
 *  from the locked original on the tab bar (the Boss's 2026-07-14 live-test remark). */
function rebrandCopyFrontmatter(content: string, copySuffix: string): string {
	const m = content.match(/^(---\r?\n[\s\S]*?\r?\n---)/);
	if (!m) return content;
	let fm = m[1].replace(/^cid_cn:[^\n]*\r?\n/m, '');
	// PJ-187 — RE-QUOTE the rebuilt title. This used to concatenate `title: ${title} (…)`
	// after stripping the surrounding quotes off in the match, so any title needing quotes
	// produced frontmatter that no longer parses — or that parses into the wrong TYPE.
	// Measured: `"Plan: phase two"` came back with 1 parse error and the title decoding as
	// the map `{Plan: "phase two (recovered copy)"}`; `"#hashtag"` parsed clean but decoded
	// as NULL (the `#` starts a comment); `"[bracket]"` decoded as an array.
	// This is the LAST-RESORT recovery path — the copy the user makes when their note is
	// locked — so a broken title here lands on the one file they were trying to rescue.
	fm = fm.replace(
		/^title:[ \t]*"?([^"\r\n]*?)"?[ \t]*$/m,
		(_, title: string) => `title: ${quoteIfNeeded(`${title} (${copySuffix})`)}`,
	);
	return fm + content.slice(m[1].length);
}

/**
 * PJ-102c — "Save a copy": the user's explicit exit when a note's own file stays
 * unwritable (a persistent lock — sync tool / antivirus / another app). Takes the
 * CURRENT unsaved content (the open model via the identity-guarded compose; the
 * write-ahead net when the tab is closed) and writes it VERBATIM (minus the cid_cn —
 * a copy is a new note) to a collision-safe sibling `<stem> (recovered copy).md`
 * through the normal gated write, then opens it as a real tab. The locked original
 * is never touched — its banner entry keeps retrying until the lock clears or the
 * user discards. Returns the copy's path, or null when there was nothing to copy
 * or every write attempt failed (the banner stays — never a silent swallow).
 */
export async function saveRecoveredCopy(path: string): Promise<string | null> {
	const tab = get(openTabs).find((t) => t.path === path);
	let content: string | null = null;
	if (tab) {
		const r = composeNoteModel(tab.id, path);
		if (r.ok) content = r.content;
	}
	if (content === null) content = getWriteAhead(path)?.content ?? null;
	if (content === null) return null;
	// Localized suffix (the full-localization standing order): filename, title, and tab
	// all carry the SAME suffix in the user's language.
	let copySuffix = 'recovered copy';
	try { const tr = get(t)('saveHealth.copySuffix'); if (tr && tr !== 'saveHealth.copySuffix') copySuffix = tr; } catch { /* fallback stays */ }
	const copyContent = rebrandCopyFrontmatter(content, copySuffix);
	const dirPath = path.replace(/[\\/][^\\/]+$/, '');
	const stem = (path.split(/[\\/]/).pop() ?? 'note').replace(/\.(md|markdown)$/, '');
	for (let i = 0; i < 20; i++) {
		const candidate = `${dirPath}/${stem} (${copySuffix}${i === 0 ? '' : ' ' + (i + 1)}).md`;
		const exists = await invoke<string>('read_note', { filePath: candidate }).then(() => true, () => false);
		if (exists) continue;
		try {
			await writeNote(candidate, copyContent, 'recovered_copy');
		} catch {
			return null; // write to the sibling failed too (folder-wide problem) — banner stays, nothing lost
		}
		emit('note-created', { path: candidate }).catch(() => {}); // gated creates are watcher-suppressed — announce
		const lib = get(libraries).find((v) => normPath(path).startsWith(normPath(v.path)));
		// NEW tab deliberately: a same-slot reuse would flush the outgoing dirty (locked)
		// original first — which fails and ABORTS the nav. A new tab opens the copy beside
		// the original with no outgoing flush; the user sees both.
		await openNoteTab(candidate, lib?.name ?? tab?.libraryName ?? '', tab?.libraryColor ?? '#7c3aed', undefined, true);
		return candidate;
	}
	return null;
}

/**
 * PJ-102c — "Discard my changes": the user's explicit decision to drop the unsaved
 * work and keep what is on disk. The open tab force-reseeds from disk through the
 * shared reload primitive (model re-seed + {#key} remount + net cleared); a closed
 * note just drops its net. The saveHealth entry clears — this is the DELIBERATE
 * counterpart of the silent discard the PJ-102 arc eliminated.
 */
export async function discardFailedSave(path: string): Promise<void> {
	const tab = get(openTabs).find((t) => t.path === path);
	// `discardLocalEdits` is REQUIRED here and nowhere else: since PJ-174 #1 the reload primitive
	// refuses to force-adopt over a dirty model, and this is the single path where discarding the
	// dirty model IS the user's stated intent. Saying so explicitly is what keeps that refusal safe
	// to apply everywhere else by default.
	if (tab) await reloadTabsFromDisk([path], { discardLocalEdits: true });
	clearWriteAhead(path); // belt — the reload clears it only on its success path
	clearSaveFailure(path);
}

/**
 * APP-KILLER #2 — the SaveEnv for a nav-flush of the OUTGOING note (the note the user is
 * navigating away from). A bare flush (like flushAllTabsInLibrary / rename_flush) omits the
 * onSaved side effects, which would leave the just-flushed note's SEARCH INDEX stale
 * (index↔disk divergence) and the second screen un-notified. This mirrors NoteEditor's
 * handleSave onSaved: broadcast (second-screen sync) + FTS5/note_meta reindex + re-embed.
 */
function navFlushEnv(tab: { id: string; name: string; libraryName: string }, origin = 'nav_flush'): SaveEnv {
	return standardSaveEnv({
		origin,
		name: tab.name,
		onSaved: (savedPath) => {
			emit('screen:note-saved', { path: savedPath }).catch(() => {});
			invoke('constellation_search_reindex', { notePath: savedPath, libraryName: tab.libraryName }).catch(() => {});
			if (get(appSettings).enabledFeatures?.semanticSearch) {
				const body = getNoteModel(tab.id)?.body.toString() ?? '';
				invoke('constellation_embed_notes', {
					notes: [{ path: savedPath, name: tab.name, content: body }],
					force: true,
				}).catch(() => {});
			}
		},
	});
}

/**
 * APP-KILLER #2 — the shared prelude for the THREE departure-flush sites (openNoteTab
 * in-place reuse, loadTabHistoryEntry, closeTab): look the tab up, and if its model is
 * dirty, flush it to disk through the durability gate before its id-slot is re-seeded /
 * disposed. Returns {ok:true} when there is nothing to flush. The per-site supersede-token
 * dance and the abort-vs-best-effort decision stay at each call site (they genuinely differ),
 * so this factors only the identical find + dirty-gate + markRecentWrite + env assembly.
 */
function flushOutgoing(tabId: string, origin = 'nav_flush'): Promise<FlushResult> {
	// PJ-130 Batch 1 (APP-KILLER) — a display-only window NEVER writes to disk.
	// "Additional screens are displays, not domains": the second screen mounts
	// core components to show them, it does not own save/load. But PJ-108 makes
	// every second-screen note-open PRESERVE the shared crash-recovery net, and
	// resolveNoteContent still returns `recoveredFromNet: true` on that path, so
	// openNoteTab calls markModelRecoveredFromNet → `m.version++` and the model is
	// born DIRTY. A departure from the second screen (closeTab, tab switch,
	// history nav) would then flushOutgoing that stale snapshot durably over the
	// note — and because the MAIN window's model for the same note is clean, the
	// watcher ADOPTS the revert instead of raising a conflict. Silent loss of the
	// main window's newer content, on screen and on disk. This is the single
	// choke point every departure-flush routes through, so one guard here closes
	// the class. The net stays preserved for the main window either way.
	if (displayOnlyWindow) return Promise.resolve({ ok: true });
	const tab = get(openTabs).find((t) => t.id === tabId);
	if (!tab || !isNoteDirty(tabId)) return Promise.resolve({ ok: true });
	// PJ-187 — a note whose links a rename cascade is REWRITING on disk must not be flushed
	// from memory: the in-memory body still carries the OLD link text, and writing it back
	// silently reverts the cascade's corrections. NoteEditor already refuses to save under
	// this condition at four sites; the departure path is the fifth, and it was open.
	//
	// ★ Safety inspection 2026-07-29 (APP-KILLER, found in THIS fix's first version): the
	// refusal returned `{ok:true}` — but the contract (noteSession.ts:266) is explicit that
	// `ok:true` means "safe to proceed with the nav/replace", and every departure site then
	// DESTROYED the dirty model: openNoteTab/loadTabHistoryEntry re-seeded it, closeTab and
	// the universe-switch sweep disposed it. The unsaved edit existed nowhere afterwards —
	// not on disk, not in the net (NoteEditor's own stash sites sit BELOW its cascade gate),
	// no banner. A refusal-to-write is `ok:false` — that is precisely "a durable write could
	// not be proven". The nav sites then keep the user on the note until the cascade lifts.
	//
	// And because closeTab / the departure sweep proceed REGARDLESS of the result (by
	// contract — the tab is being dismissed), the model's current content is stashed into
	// the write-ahead net FIRST, unflagged (it is real unsaved work, so the PJ-181 stale-
	// snapshot check never discards it): reopen restores it, the documented recovery route.
	if (isCascading(tab.path)) {
		const r = composeNoteModel(tabId, tab.path);
		if (r.ok) setWriteAhead(tab.path, r.content, tab.cursorPos ?? 0, tab.scrollTop ?? 0);
		return Promise.resolve({ ok: false, reason: 'cascading' });
	}
	markRecentWrite(tab.path);
	return flushOutgoingModel(tabId, navFlushEnv(tab, origin));
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
	if (cascadingPaths.size === 0 && cascadingLibraries.size === 0) return false;
	const key = normPath(path);
	if (cascadingPaths.has(key)) return true;
	// PJ-174 #1 — the LIVE half: a tab opened after the path snapshot was taken is still inside a
	// cascading library. Separator boundary, so `/Foo/Bar2` never matches a cascade on `/Foo/Bar`
	// (same rule as `tabsInLibrary`).
	for (const lib of cascadingLibraries.keys()) {
		if (key === lib || key.startsWith(lib + '/')) return true;
	}
	return false;
}
/** §3-redo.7 — clear every cascading entry. Used by the Universe-switch
 *  path so a cascade in flight in the previous Universe doesn't leave
 *  stale entries that gate edits in the new Universe. */
export function clearAllCascading() { cascadingPaths.clear(); cascadingLibraries.clear(); }

/** PJ-174 #1 — LIBRARY-scoped cascade marks, refcounted like `cascadingPaths`.
 *
 *  ★ Why a second map instead of more entries in the first one. `cascadingPaths` is populated from
 *  `tabsInLibrary(lib.path)` — a SNAPSHOT taken before a multi-second library walk. The sidebar
 *  tree is not blocked during that walk (the freeze overlay covers editor panes only), so the user
 *  can open a note mid-walk; that tab is in no snapshot, so `isCascading` returned FALSE for it and
 *  its autosave ran ungated against a file the walker was rewriting.
 *
 *  A path snapshot cannot be repaired by taking it later — there is no "later" that is after every
 *  tab that might be opened. The predicate has to stop being a snapshot: marking the LIBRARY makes
 *  `isCascading` answer "is this path inside a library currently cascading?", which is true for a
 *  tab that does not exist yet at mark time. Reproduced by
 *  `tests/pj-174/renameCascadeMidWalkTab.test.ts`.
 *
 *  Hot-path cost: `isCascading` keeps its O(1) early-exit while BOTH maps are empty — the steady
 *  state. The prefix loop runs only inside a cascade window, over a map that holds one entry. */
const cascadingLibraries = new Map<string, number>();
export function markCascadingLibrary(libraryPath: string) {
	const key = normPath(libraryPath).replace(/\/+$/, '');
	cascadingLibraries.set(key, (cascadingLibraries.get(key) ?? 0) + 1);
}
export function clearCascadingLibrary(libraryPath: string) {
	const key = normPath(libraryPath).replace(/\/+$/, '');
	const n = cascadingLibraries.get(key);
	if (n === undefined) return;
	if (n <= 1) cascadingLibraries.delete(key);
	else cascadingLibraries.set(key, n - 1);
}

/** MIG-076 §D1: the REACTIVE freeze signal for the quiesce overlay. Holds the
 *  `tab.path` strings of every open pane currently inside a rename/cascade window.
 *  The rename orchestrator (`handleRenameComplete`) sets it inside the cascade
 *  block and clears it in the inner `finally`. SEPARATE from `cascadingPaths` (the
 *  non-reactive write-gate Map) on purpose: the editor surfaces subscribe to this
 *  for the read-only overlay, while the hot-path save gate stays a plain Map
 *  lookup (no reactivity on the keystroke path, Rule 1). */
/**
 *  PJ-174 #1 — this now holds the cascading LIBRARY ROOTS, not a snapshot of tab paths.
 *
 *  It was `new Set(tabsInLibrary(lib.path).map(t => t.path))`, taken before a multi-second walk, so
 *  a note opened mid-walk got no read-only overlay — the user could type into a file the walker was
 *  actively rewriting. Same defect as the save gate, same cure, and holding roots means the freeze
 *  and the gate are now ONE concept with one boundary rule instead of two representations that can
 *  disagree. Membership goes through `isPathFrozen`.
 */
export const cascadeFreeze = writable<Set<string>>(new Set());

/**
 * PJ-187 — REFCOUNTED freeze, mirroring `markCascadingLibrary` / `clearCascadingLibrary`.
 *
 * The two call sites used to be bare `cascadeFreeze.set(new Set([lib.path]))` and
 * `cascadeFreeze.set(new Set())`, so two overlapping renames in the same library shared one
 * boolean: the first to finish published the EMPTY set and lifted the overlay while the second
 * cascade was still rewriting files. The user could then type into a note the walker was
 * mid-rewrite — exactly the window the overlay exists to close. The non-reactive save-gate
 * beside it (`cascadingLibraries`) has been refcounted since MIG-076; this is its reactive twin
 * and it was not. A Map of depths, published as its key set, makes the pairing structural.
 */
const freezeDepth = new Map<string, number>();
const publishFreeze = () => cascadeFreeze.set(new Set(freezeDepth.keys()));
export function markFreeze(libraryPath: string) {
	const key = normPath(libraryPath).replace(/\/+$/, '');
	freezeDepth.set(key, (freezeDepth.get(key) ?? 0) + 1);
	publishFreeze();
}
export function clearFreeze(libraryPath: string) {
	const key = normPath(libraryPath).replace(/\/+$/, '');
	const n = freezeDepth.get(key);
	if (n === undefined) return;
	if (n <= 1) freezeDepth.delete(key);
	else freezeDepth.set(key, n - 1);
	publishFreeze();
}

/** Is `path` inside one of the frozen library roots? Separator-boundary matched, so a cascade on
 *  `/Foo/Bar` never freezes `/Foo/Bar2`. Shared by every consumer so the rule cannot drift. */
export function isPathFrozen(path: string, frozen: Set<string>): boolean {
	if (!path || frozen.size === 0) return false;
	const key = normPath(path);
	for (const raw of frozen) {
		// Normalise BOTH sides here rather than trusting each caller to do it — the one place this
		// was done by hand produced a broken regex, which is the argument for it living in one
		// function that every consumer shares.
		const root = normPath(raw).replace(/\/+$/, '');
		if (key === root || key.startsWith(root + '/')) return true;
	}
	return false;
}

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
 *
 *  PJ-092 INVARIANT: this primitive FORCE-adopts disk over the model (openNoteModel,
 *  no dirty-guard — by design). It must NEVER be handed a path whose open model is
 *  DIRTY: force-reseeding a dirty model from divergent disk loses the edits (or hangs
 *  the reactive layer). The guard lives UPSTREAM at every caller — the rename cascade
 *  excludes not-durably-flushed notes (`flushAllTabsInLibrary`'s returned paths → the
 *  walker never rewrites them → they're never in `result.rewritten`), and the single-
 *  note callers abort on a non-durable flush. A future edit MUST NOT leak a dirty path
 *  into this function.
 */
export async function reloadTabsFromDisk(
	filePaths: string[],
	opts?: {
		/**
		 * The user's EXPLICIT decision to throw away unsaved work and keep what is on disk —
		 * today only PJ-102c "Discard my changes". Without it a dirty model is never force-adopted.
		 *
		 * This is opt-in rather than the default because exactly one of the nine call sites wants
		 * it, and a blanket dirty-refusal would have silently broken that feature (caught by the
		 * WA#4 consumer sweep before it shipped). Destroying a user's edits should have to be
		 * asked for by name.
		 */
		discardLocalEdits?: boolean;
		/**
		 * Where a genuine conflict goes: the model is dirty AND disk moved underneath it. Same
		 * signature and same handler as the watcher's external-change path, so a cascade conflict
		 * and a Syncthing conflict produce the same `.conflict` sidecar and the same banner
		 * instead of two policies for one situation.
		 */
		conflict?: (notePath: string, noteName: string, diskContent: string) => void | Promise<void>;
	},
): Promise<string[]> {
	if (filePaths.length === 0) return [];
	const tabs = get(openTabs);
	const targets = filePaths.filter((fp) => tabs.some((t) => t.path === fp));
	if (targets.length === 0) return [];

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
	if (byPath.size === 0) return [];

	// ★ PJ-174 #1 — ENFORCE THE INVARIANT THIS FUNCTION ALREADY DOCUMENTS.
	//
	// The docstring above says a dirty path must never be handed here and that "the guard lives
	// UPSTREAM at every caller". Upstream for the rename cascade is a path snapshot taken before a
	// multi-second walk, and a note opened mid-walk is in no snapshot — so the invariant leaked,
	// and this function force-adopted over a dirty model: the user's unsaved paragraph erased from
	// the model, the screen AND the write-ahead net, with `isDirty` then reporting false so nothing
	// downstream could even tell work had been lost.
	//
	// An invariant that every caller must uphold, in a function whose only job is destructive
	// re-seeding, is a promise waiting to be broken. It is enforced here, where the damage happens.
	// (`linkMentionInNote` already carried a comment asserting this guard existed — it did not.)
	const refused: string[] = [];
	const conflicted: Array<{ path: string; name: string; disk: string }> = [];
	if (!opts?.discardLocalEdits) {
		for (const t of get(openTabs)) {
			const disk = byPath.get(t.path);
			if (disk === undefined || disk === t.content) continue;
			if (!isNoteDirty(t.id)) continue;
			byPath.delete(t.path); // never force-adopted below
			refused.push(t.path);
			// Dirty AND disk genuinely moved → a real conflict. Route it to the SAME handler the
			// watcher uses (clean→adopt, dirty→`.conflict` sidecar + banner) so the user's edits
			// stay live in the editor and the incoming version is preserved rather than either one
			// being silently dropped.
			if (diskDiffersFromBaseline(t.id, disk)) conflicted.push({ path: t.path, name: t.name, disk });
		}
	}
	if (byPath.size === 0 && conflicted.length === 0) return refused;

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
	// The cascade just authored canonical disk content; any in-flight write-ahead buffer for these
	// paths is now stale. ONLY for paths we actually adopted — a refused dirty tab KEEPS its net,
	// because that net is now part of the only copy of the user's work (mirrors the same rule in
	// `adoptExternalChangeIntoTabs`, invariant #10).
	for (const fp of byPath.keys()) clearWriteAhead(fp);

	for (const c of conflicted) await opts?.conflict?.(c.path, c.name, c.disk);
	return refused;
}

/** PJ-070 — paths whose NotePane is mid-REMOUNT from a watcher/external adopt (a reloadVersion
 *  bump). DEDICATED (not the shared cascade Map) so a concurrent rename cascade's clear — even
 *  clearAllCascading — can never lift it. NoteEditor's teardown flush gates (handleFlush /
 *  handleSave) bail while a path is reseeding: otherwise the OLD, stale-displaying editor's
 *  {#key} teardown flush would push its pre-adopt body back into the freshly-adopted model —
 *  the hazard-#6 re-stale (setBody has no staleness guard). O(1) empty-set early-exit keeps it
 *  off the keystroke hot path. */
const reseedingPaths = new Map<string, number>();
export function isReseeding(path: string): boolean {
	if (reseedingPaths.size === 0) return false;
	return reseedingPaths.has(normPath(path));
}
// Refcounted (like cascadingPaths) so two overlapping adopts of the SAME path — e.g. the 300ms
// watcher flush racing an onNoteSaved within one tick() bracket — don't pop each other's mark: the
// first clear must not lift the gate while the second is still mid-teardown.
function markReseeding(path: string) {
	const key = normPath(path);
	reseedingPaths.set(key, (reseedingPaths.get(key) ?? 0) + 1);
}
function clearReseeding(path: string) {
	const key = normPath(path);
	const n = reseedingPaths.get(key);
	if (n === undefined) return;
	if (n <= 1) reseedingPaths.delete(key);
	else reseedingPaths.set(key, n - 1);
}

/**
 * PJ-070 — adopt an EXTERNAL disk change (file watcher / second-screen save) into every open tab
 * whose file changed, freshness-gated. The main-window twin of SecondScreenPage.adoptFreshDiskIntoSS,
 * and the ONE home both main-window ingress paths call (the watcher flush + onNoteSaved). Per changed
 * open tab:
 *   - CLEAN model → adoptDisk: the model adopts the disk, and (unless it is the Focus note) the tab's
 *     reloadVersion bumps so NoteEditor's {#key} remounts NotePane on the fresh model. The bump is
 *     bracketed by a DEDICATED reseed mark spanning the async {#key} teardown (await tick), so the
 *     outgoing editor's teardown flush can't re-stale the adopted model (hazard #6). When the note is
 *     in FOCUS mode, hooks.focusReseed remounts FocusPane instead (hazard #7) — and if no focusReseed
 *     is wired yet, the focus tab is LEFT UNTOUCHED (never adopt-without-reseed).
 *   - DIRTY model + a GENUINE external change (disk !== the model's synced baseline) → hooks.conflict
 *     (writes the `.conflict` sidecar + banner). Local unsaved work is NEVER clobbered (adoptDisk refuses
 *     on dirty). An echo of our own write / a spurious touch (disk === baseline) is ignored.
 * Never force-adopts (never openNoteModel / reloadTabsFromDisk here — that would discard local edits, the
 * APP-KILLER #2 / LL-014 class). When WATCHER_ADOPT_ENABLED is off, degrades to the pre-PJ-070 content-only
 * update (the Recipe-O clobber). `readDisk` is injectable so the harness drives the store boundary sans Tauri.
 */
export async function adoptExternalChangeIntoTabs(
	paths: string[],
	hooks?: {
		conflict?: (notePath: string, noteName: string, diskContent: string) => void | Promise<void>;
		focusReseed?: (path: string) => void;
		focusPath?: string | null;
	},
	readDisk: (filePath: string) => Promise<string> = readNote,
): Promise<void> {
	if (paths.length === 0) return;
	const tabs = get(openTabs);
	const openPaths = new Set(tabs.map((t) => t.path));
	const targets = paths.filter((fp) => openPaths.has(fp)); // invariant #9 — O(changed + open), not O(changed × open)
	if (targets.length === 0) return;

	// Read each changed open path once (reuse the 300ms-flush read; a deleted file's rejection is
	// caught per-path so it can't sink the batch — invariant #8).
	const reads = await Promise.all(
		targets.map((fp) => readDisk(fp).then((content) => ({ fp, content }) as const).catch(() => null)),
	);
	const byPath = new Map<string, string>();
	for (const r of reads) if (r) byPath.set(r.fp, r.content);
	if (byPath.size === 0) return;

	// ── ROLLBACK path (WATCHER_ADOPT_ENABLED off): today's content-only update, no adopt, no remount. ──
	if (!WATCHER_ADOPT_ENABLED) {
		openTabs.update((ts) => ts.map((t) => {
			const c = byPath.get(t.path);
			return c !== undefined ? { ...t, content: c } : t;
		}));
		return;
	}

	const focusPathNorm = hooks?.focusPath ? normPath(hooks.focusPath) : null;
	const adopted = new Set<string>();          // paths whose CLEAN model adopted the disk (path↔id is 1:1 under DEDUP)
	let focusReseedPath: string | null = null;  // the at-most-one focus note that also needs a FocusPane remount
	const conflicts: Array<{ path: string; name: string; disk: string }> = [];

	// Re-read the tab list AFTER the awaited disk reads. The snapshot taken before them
	// (line ~894) can be stale by the time they resolve: an in-place navigation landing
	// in the read window rebinds a tab's path→id, and adopting on the old pairing writes
	// one note's disk content into another note's model. Iterating a snapshot taken
	// immediately before this synchronous loop means every (path, id) pair is current at
	// the moment it is used. (2026-07-21 inspection, APP-KILLER; `adoptDisk`'s new
	// expectPath guard is the second line of defence, not the first.)
	for (const t of get(openTabs)) {
		const disk = byPath.get(t.path);
		if (disk === undefined) continue;
		if (isCascading(t.path)) continue; // invariant #3 — a rename cascade owns this path; its force-adopt runs there
		const isFocusTab = focusPathNorm !== null && normPath(t.path) === focusPathNorm;
		// Never adopt a Focus note without a way to reseed FocusPane (it ignores `value` after mount):
		// a fresh model behind a stale Focus view would let Focus's teardown write stale. Leave as today.
		if (isFocusTab && !hooks?.focusReseed) continue;
		if (externalChangeNoteModel(t.id, disk, t.path)) {
			// CLEAN model adopted the disk (invariant #2 — adoptDisk, never a force re-seed). Refresh the
			// store tab (content + reloadVersion) uniformly; the FocusPane one ALSO gets focusReseed below.
			adopted.add(t.path);
			if (isFocusTab) focusReseedPath = t.path; // hazard #7 — remount FocusPane too (not under the reloadVersion {#key})
		} else if (isNoteDirty(t.id) && diskDiffersFromBaseline(t.id, disk)) {
			// DIRTY + a genuine external change → preserve the incoming edit; never clobber local work.
			conflicts.push({ path: t.path, name: t.name, disk });
		}
		// else: our own write echoing back / a spurious touch (disk === baseline) → nothing to do (invariant #1).
	}

	// Batched remount for the clean adopters, bracketed by the dedicated reseed mark across the async
	// {#key} teardown (hazard #6): mark BEFORE the store update, clear only AFTER tick() has flushed the
	// remount + the outgoing editor's onDestroy teardown flush (which the mark makes a no-op). The reseed
	// mark is inert for a Focus tab (NotePane isn't mounted); FocusPane's own teardown is gated by
	// focusReseedSuppress inside focusReseed.
	if (adopted.size > 0) {
		for (const p of adopted) markReseeding(p);
		try {
			openTabs.update((ts) => ts.map((t) =>
				adopted.has(t.path)
					? { ...t, content: byPath.get(t.path)!, reloadVersion: (t.reloadVersion ?? 0) + 1 } // invariant #4/#5 — only adopters
					: t,
			));
			for (const p of adopted) clearWriteAhead(p); // invariant #10 — only adopters (a dirty refuser keeps its net)
			if (focusReseedPath) hooks!.focusReseed!(focusReseedPath); // remount FocusPane on the freshly-adopted model
			await tick();
		} finally {
			// ALWAYS lift the gate — a throw in the flush/reseed above must never leave a path
			// permanently reseeding, which would disable handleSave/handleFlush for it (a save-loss).
			for (const p of adopted) clearReseeding(p);
		}
	}

	// Sidecar + banner for each genuine dirty conflict (async IPC — a stub no-ops if unwired).
	for (const c of conflicts) {
		await hooks?.conflict?.(c.path, c.name, c.disk);
	}
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
		// PJ-092 H4 — bounded flush; if not durable, DON'T toggle+reload (clobber/hang guard); save-health retries.
		if (openTab && !(await flushOpenTabOrAbort(openTab, 'task_toggle_flush'))) return;
		await toggleTask(filePath, lineNumber);
		await reloadTabsFromDisk([filePath]); // model ADOPTS the toggled disk + {#key} remount
	} finally {
		if (openTab) clearCascading(openTab.path);
	}
}

/**
 * MIG-086 Part 2 §F2 — add a typed link to a note's FRONTMATTER as a type-as-property
 * entry (`<type>:` list of `[[wikilink]]`). Mirrors `addTagToProps`' list semantics
 * (dedup, `list` type). The property KEY is the link-type id; the value list holds
 * `[[target]]` wikilinks (reconstructFrontmatter quotes them → valid, interoperable
 * YAML). Returns the updated props, or `null` when the link is already present so the
 * caller can skip the write.
 */
export function addTypedLinkToProps(
	props: FrontmatterProperty[],
	linkType: string,
	wikilink: string
): FrontmatterProperty[] | null {
	const key = linkType.toLowerCase();
	const idx = props.findIndex((p) => p.key.toLowerCase() === key);
	if (idx >= 0) {
		const p = props[idx];
		const existing =
			p.listItems ?? (p.value ? String(p.value).split(',').map((s) => s.trim()).filter(Boolean) : []);
		if (existing.some((x) => x.trim().toLowerCase() === wikilink.toLowerCase())) return null; // already linked
		const items = [...existing, wikilink];
		return props.map((q, i) => (i === idx ? { ...q, type: 'list', listItems: items, value: items.join(', ') } : q));
	}
	return [...props, { key: linkType, value: wikilink, type: 'list', listItems: [wikilink] } as FrontmatterProperty];
}

/**
 * MIG-086 Part 2 §F2 — connect notes by declaring a typed Living Link in the SOURCE
 * note's FRONTMATTER (type-as-property), content-integrity-safe whether the source is
 * OPEN or CLOSED. The link is born ONLY as frontmatter `<type>: ["[[target]]"]` text;
 * `index_note` derives the `note_links` row (single-writer), defaulting it to
 * `hypothesis` confidence + weight 1.0 (concept invariant C-4, automatic). This helper
 * NEVER writes `note_links`.
 *
 * Direction (de-orphan semantics): to give note O an incoming link, pass the related
 * CANDIDATE as `sourcePath` and O's display name as `target` — the wikilink lives in the
 * candidate's frontmatter, pointing at O. Reindexing the candidate bumps O's
 * `incoming_count` (`maintain_incoming_after_save`), so O leaves the orphan lens.
 *
 * Open/closed safety: this is the proven PROPS save path (exactly `addTagToNote`). OPEN →
 * `composeNoteModel` (identity-guarded; refuse on `!ok`) → `saveTabContent` (composes from
 * the model + reindexes; the body is NEVER touched, so there is no §C dangling-append and
 * no BUG-015 surface). CLOSED → `readNote` → add prop → `writeNote` → `reindexNote`.
 * Supersedes the §C body-append (Boss ruling 2026-06-24: typed links live in frontmatter).
 */
export async function addLinkToNote(sourcePath: string, linkType: string, target: string): Promise<void> {
	const type = (linkType || 'associative').trim();
	const tgt = (target || '').trim();
	if (!tgt) return;
	const srcNorm = normPath(sourcePath);
	// MIG-105 Stage-0 C7 (PJ-156 E1): boundary-guarded LONGEST-prefix = most-specific
	// (nested/federated) — NOT first-match, which returned the ROOT library whose path
	// prefixes every nested one; the CLOSED branch below feeds lib.name to reindexNote,
	// so first-match WROTE the wrong library into note_meta. Same shape as
	// linkMentionInNote's resolver.
	const lib = get(libraryStats)
		.filter((l) => srcNorm === normPath(l.path) || srcNorm.startsWith(normPath(l.path) + '/'))
		.sort((a, b) => normPath(b.path).length - normPath(a.path).length)[0];
	if (!lib) throw new Error(`addLinkToNote: no library found for ${sourcePath}`);
	const wikilink = `[[${tgt}]]`;
	const openTab = get(openTabs).find((t) => t.path === sourcePath);
	if (openTab) {
		// OPEN: the props save path (identity-guarded), exactly like addTagToNote.
		// saveTabContent edits the model's props, composes from the model, and reindexes;
		// the body is untouched — no dangling append, no BUG-015 surface.
		const r = composeNoteModel(openTab.id, sourcePath);
		if (!r.ok) return; // identity refusal — never write behind a mismatched model
		const { properties, body } = parseFrontmatter(r.content);
		const updated = addTypedLinkToProps(properties, type, wikilink);
		if (!updated) return; // already linked
		// PJ-066 follow-up: bodyUnchanged=true — a typed-link connect only adds a
		// frontmatter property; the body (hence its embedding) is unchanged, so skip
		// the redundant ~32s re-embed (the felt connect-freeze on link-dense notes).
		await saveTabContent(openTab.id, sourcePath, updated, body, true);
		// Refresh the OPEN note's view so the new property shows immediately. The props save
		// path updates the model + disk but does NOT re-render an already-open note's
		// PropertyEditor (Boss finding: the link only appeared on reopen). reloadTabsFromDisk
		// re-seeds + {#key}-remounts the tab from the just-written disk — cheap (one file read,
		// no reindex), the proven pattern toggleTaskReconciled uses to refresh an open note
		// after an external change.
		// PJ-092 H4 — addLinkToNote is largely safe-by-construction (saveTabContent IS the write;
		// no independent disk-rewriter, so a failed write leaves disk unchanged and this reload no-ops).
		// Belt: if the model is still dirty (the write didn't land), skip the view-refresh — the new
		// property already shows in the dirty model; save-health retries the write.
		if (isNoteDirty(openTab.id)) return;
		await reloadTabsFromDisk([sourcePath]);
	} else {
		// CLOSED: gated writeNote + reindex; index_note derives the note_links row (and
		// bumps the target's incoming_count). Adding a frontmatter property necessarily
		// re-serializes the frontmatter — the SAME proven path addTagToNote uses for tags.
		// MIG-090 §9 (discovered defect, fix-what-we-discover): the OPEN branch
		// inherits the cascade gate via saveTabContent; this branch reached
		// writeNote WITHOUT it — a connect fired mid-rename-cascade could write
		// a pre-cascade read back over the walker's rewrite. Same guard, same
		// refusal shape as NoteEditor.handlePromote.
		if (isCascading(sourcePath)) return;
		const content = await readNote(sourcePath);
		const { properties, body } = parseFrontmatter(content);
		const updated = addTypedLinkToProps(properties, type, wikilink);
		if (!updated) return; // already linked
		markRecentWrite(sourcePath);
		// G4 Phase 3 — byte-perfect round-trip: preserve the closed note's rich
		// frontmatter (nested maps / block scalars / quoted values) instead of
		// rebuilding it lossily with buildFullContent.
		await writeNote(sourcePath, composeUpdatedContent(content, updated, body), 'reviewer_connect');
		// Fire-and-forget the reindex (exactly like saveTabContent does for every save): a
		// link-dense source can be slow to reindex because index_note's per-edge sky triggers
		// re-fire for ALL its edges (pre-existing cost — PJ-066), and the CONNECT must not
		// block on it. The frontmatter (source of truth) is already on disk; the note_links
		// derivation + the target's incoming_count catch up in the background (or on the next
		// boot's reindex if interrupted). The Reviewer updates optimistically (refreshAfterConnect).
		reindexNote(sourcePath, lib.name).catch((e) => console.error('[addLinkToNote] background reindex failed:', e));
	}
}

/**
 * Replace the FIRST plain-text occurrence of `name` in `body` with `[[name]]`, skipping any
 * occurrence that already sits inside an existing `[[wikilink]]` (so a note that already links
 * the target elsewhere can't produce `[[[[name]]]]`). Scoped to the BODY only — never the
 * frontmatter — so a title/tag/alias match can't inject `[[..]]` into YAML and corrupt it.
 * Returns the new body, or `null` when there is no plain mention to link.
 */
function firstPlainMentionReplace(body: string, name: string): string | null {
	const esc = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	const re = new RegExp(`\\b${esc}\\b`, 'gi');
	let m: RegExpExecArray | null;
	while ((m = re.exec(body)) !== null) {
		const before = body.slice(0, m.index);
		// Inside an open `[[…` that hasn't closed yet? → already-linked occurrence, try the next.
		if (before.lastIndexOf('[[') > before.lastIndexOf(']]')) continue;
		return before + `[[${name}]]` + body.slice(m.index + m[0].length);
	}
	return null;
}

/**
 * PJ-140 [0] (HIGH — 2026-07-25). Turn a plain-text mention of `targetName` in `mentionPath`'s
 * BODY into an inline `[[wikilink]]`, content-integrity-safe whether the mentioning note is OPEN
 * or CLOSED. This replaces `BacklinksPanel.linkMention`'s raw `invoke('write_note')`, which had
 * THREE silent failure modes the "link it" button must never have:
 *   1. Open-model overwrite — it read the note from DISK and wrote behind the open in-memory
 *      model, whose next autosave composed from the stale (pre-link) body and durably erased the
 *      `[[link]]` (and, for a dirty note, the user's unsaved edits with it).
 *   2. False success — a `catch {}` swallowed a failed write, so the user believed the mention
 *      was linked while disk was unchanged.
 *   3. Index divergence — no reindex, so the new backlink edge stayed invisible until a boot.
 *
 * The fix is the proven `toggleTaskReconciled` body-edit shape (single content ownership): gate
 * the whole op → flush the open model to disk first (or ABORT rather than clobber) → mutate disk
 * → the model adopts the mutated disk (remount) → reindex. Longest-root-wins library resolution
 * (nested-library-correct — the PJ-140/PJ-141 class). Returns `true` when a link was written,
 * `false` when there was no plain mention to link; THROWS on a genuine write failure so the
 * caller can surface it (never swallowed).
 */
export async function linkMentionInNote(mentionPath: string, targetName: string): Promise<boolean> {
	const name = (targetName || '').trim();
	if (!name) return false;
	const mNorm = normPath(mentionPath);
	// longest-prefix = most-specific (nested/federated) — NOT first-match, which returns the
	// root library whose path prefixes every nested one (the PJ-141 resolver bug).
	const lib = get(libraryStats)
		.filter((l) => mNorm === normPath(l.path) || mNorm.startsWith(normPath(l.path) + '/'))
		.sort((a, b) => normPath(b.path).length - normPath(a.path).length)[0];
	if (!lib) throw new Error(`linkMentionInNote: no library for ${mentionPath}`);

	const openTab = get(openTabs).find((t) => t.path === mentionPath);
	// Gate the WHOLE op (toggleTaskReconciled's F2 guard): mark BEFORE flush+mutate+reload so an
	// open note's armed NotePane autosave can't fire mid-mutation and REVERT the new wikilink on disk.
	if (openTab) markCascading(openTab.path);
	try {
		// Flush the open model to disk FIRST so the user's unsaved edits are never lost — and if the
		// flush did NOT land (locked .md / keystroke-during-await), ABORT instead of mutating behind a
		// dirty model (the exact clobber this HIGH is about). The ~10s save-health loop retries the flush.
		if (openTab && !(await flushOpenTabOrAbort(openTab, 'link_mention_flush'))) return false;
		// Closed-note cascade gate (addLinkToNote parity): don't write a pre-cascade read back over a
		// rename walker's in-flight rewrite.
		if (!openTab && isCascading(mentionPath)) return false;

		const content = await readNote(mentionPath);
		const { properties, body } = parseFrontmatter(content);
		const newBody = firstPlainMentionReplace(body, name);
		if (newBody === null) return false; // no plain mention in the BODY (already linked / frontmatter-only)

		markRecentWrite(mentionPath);
		// THROWS on a genuine write failure — the caller surfaces it, never the old silent catch{}.
		await writeNote(mentionPath, composeUpdatedContent(content, properties, newBody), 'link_mention');

		// Open note: the model ADOPTS the mutated disk + {#key} remount so the inline [[link]] shows at
		// once. reloadTabsFromDisk skips the reseed if a keystroke re-dirtied the model (PJ-092), so a
		// during-write edit is preserved rather than clobbered.
		if (openTab) await reloadTabsFromDisk([mentionPath]);

		// AWAIT the reindex so the caller can refresh the Backlinks / Unlinked-mentions panels
		// deterministically the instant the new note_links edge exists — instead of waiting for
		// an incidental trigger to re-run the panel effect (the felt 5-10s lag the Boss saw). This
		// single-note reindex is fast: O(changed-edges) incoming/sky maintenance (only the one new
		// target recomputes) and NO re-embed (index_note never embeds). A reindex FAILURE is
		// non-fatal — the wikilink is already durable on disk and the index self-heals on the next
		// boot reconcile — so it is logged, not thrown.
		await reindexNote(mentionPath, lib.name).catch((e) => console.error('[linkMentionInNote] background reindex failed:', e));
		return true;
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
export async function flushAllTabsInLibrary(libraryPath: string): Promise<string[]> {
	// PJ-092 (redo, flush-gate-exclude) — RETURNS the paths whose flush was NOT
	// durable. The rename cascade excludes these from the on-disk link rewrite so a
	// note we couldn't flush is NEVER touched on disk → no model↔disk divergence →
	// no silent data-loss, no reactive freeze. Mirrors renameItem's `renameFlushOk`
	// gate, generalized to the whole-library cascade.
	//
	// H2 — each dirty tab is flushed through the BOUNDED re-flush loop
	// (`flushOutgoingModel` = flushIfDirty, MAX=4): a keystroke that lands DURING the
	// awaited write advances the model, so a single `save` would return ok:true with
	// the model still dirty (the walker would then rewrite the note and the reload
	// would clobber the during-write keystroke). The loop re-composes until clean or
	// a genuine failure, closing that await-window race.
	//
	// Hardening — a flush that THROWS is recorded as failed (never propagated): a
	// thrown `flushAllTabsInLibrary` would leave the caller's excluded set undefined
	// and skip the whole cascade. Fail-safe = never rewrite a note we couldn't prove
	// we flushed.
	const failed: string[] = [];
	const flushes: Promise<void>[] = [];
	for (const tab of tabsInLibrary(libraryPath)) {
		if (SINGLE_OWNERSHIP) {
			if (!isNoteDirty(tab.id)) continue; // clean — disk already current
			markRecentWrite(tab.path);
			const p = tab.path;
			flushes.push(
				flushOutgoingModel(tab.id, standardSaveEnv({ origin: 'flush_all', name: tab.name }), 'flush_all')
					.then((r) => { if (!r.ok) failed.push(p); })
					.catch((err) => {
						console.error('[flushAllTabsInLibrary] model flush threw for', p, err);
						failed.push(p);
					})
			);
		} else {
			// Legacy (!SINGLE_OWNERSHIP) rollback path: the write-ahead-buffer write.
			// Same fail-safe contract — a failed write records the path so the cascade excludes it.
			const wab = getWriteAhead(tab.path);
			if (!wab) continue; // not dirty — nothing to flush
			markRecentWrite(tab.path);
			const p = tab.path;
			flushes.push(
				writeNote(tab.path, wab.content, 'flush_all')
					.then(() => clearWriteAhead(p))
					.catch((err) => {
						console.error('[flushAllTabsInLibrary] write failed for', p, err);
						failed.push(p);
					})
			);
		}
	}
	await Promise.all(flushes);
	return failed;
}

/** PJ-092 — flush ONE open tab's dirty model to disk through the BOUNDED re-flush loop
 *  before an operation rewrites its file + reloads it. Returns `true` when durable (or the
 *  tab isn't dirty), `false` when the flush did NOT land. On `false` the caller MUST NOT
 *  rewrite + reload the note — force-reseeding a still-dirty model is the loss/freeze class
 *  this migration closes; the edit stays dirty + netted and the ~10 s save-health retry
 *  persists it. The bounded loop (`flushOutgoingModel`) closes the H2 await-window race — a
 *  single `save` would return ok:true with the model dirty again if a keystroke lands during
 *  the write. One primitive so a future flush-then-reload caller can't forget the gate. */
export async function flushOpenTabOrAbort(openTab: OpenTab, origin: string): Promise<boolean> {
	if (!isNoteDirty(openTab.id)) return true;
	markRecentWrite(openTab.path);
	const r = await flushOutgoingModel(openTab.id, standardSaveEnv({ origin, name: openTab.name }), origin);
	return r.ok;
}

/**
 * MIG-107 — the "updated" / "modified" auto-date rule, extracted so the intent commit path applies
 * exactly the same rule as the legacy whole-array path. One definition, not two that can drift.
 */
export function withAutoUpdatedDate(properties: FrontmatterProperty[]): FrontmatterProperty[] {
	const now = new Date();
	const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
	return properties.map((p) => {
		const k = p.key.toLowerCase();
		if ((k === 'updated' || k === 'modified' || k === 'حُدث' || k === 'تعديل') && p.type === 'date') {
			return { ...p, value: dateStr };
		}
		return p;
	});
}

export async function saveTabContent(
	tabId: string,
	filePath: string,
	properties: FrontmatterProperty[],
	body: string,
	// PJ-066 follow-up (2026-06-27): when the caller KNOWS the body is unchanged
	// (a frontmatter-only save — e.g. a typed-link connect), the e5 embedding of the
	// body is identical to what's stored, so re-embedding is pure waste (~32s of ONNX
	// on the 533-link note, measured). Passing `bodyUnchanged` flips the embed to
	// `force: false`, which makes the Rust command skip when the note is already
	// embedded (and still embed it if it never was — safe). Body edits leave it false.
	bodyUnchanged: boolean = false,
	/**
	 * MIG-107 Slice 4 — the caller has ALREADY put its edit into the model, one property at a time
	 * (`propsCommit`), so this must not replace the model's array from `properties`. That replace is
	 * the defect: an array assembled from one panel's view deletes whatever another writer changed
	 * in the meantime. Everything downstream is unchanged — compose still reads the model.
	 */
	propsAlreadyInModel: boolean = false
): Promise<void> {
	const updatedProps = withAutoUpdatedDate(properties);
	// Save-Durability (2026-07-08) — push props to the MODEL (the source of truth)
	// BEFORE the single-flight write guard, so a concurrent property edit arriving while
	// a slow write is in flight is NEVER dropped: it lands in the model (dirty) and the
	// next save/flush persists it. The guard serializes the WRITE, never the model update.
	// (Fixes the saveLocks-drop silent-loss — same class as this migration; wf_5f9b257d.)
	if (SINGLE_OWNERSHIP && !propsAlreadyInModel) editNoteProps(tabId, updatedProps, filePath);
	// PropertyEditor's frontmatter edits land here directly, so the same F2 post-cascade-stomp gate
	// NoteEditor uses must apply here too (see `isCascading`) — it gates the DISK WRITE.
	//
	// ★ PJ-174 #1c — this gate used to sit ABOVE `editNoteProps`, so a property edited during a
	// rename-cascade window was not written AND not kept: it never reached the model, and the panel
	// holds no record of its own, so it was simply gone. That is precisely the drop the comment
	// directly above rules out for the write-lock — "the guard serializes the WRITE, never the model
	// update" — and the cascade gate was breaking the same rule two lines earlier.
	//
	// It matters more now than it did: the cascade gate became LIVE and library-scoped in this same
	// change (PJ-174 #1), so it covers far more of the app for the duration. Widening a gate that
	// silently discards edits would have turned a fix into a regression. The right-sidebar
	// PropertyEditor is reachable during the window too — the freeze overlay covers editor panes,
	// not the sidebar.
	//
	// Now the edit lands in the model (dirty) and only the write waits. The cascade's own reload
	// then sees a dirty model and — since PJ-174 #1 — refuses to force-adopt over it, raising a
	// conflict instead. So the property survives and the user is told, rather than the edit
	// vanishing with the app reporting success.
	if (isCascading(filePath)) return;
	if (saveLocks.get(tabId)) return; // a write is already in flight; the model has this edit
	saveLocks.set(tabId, true);
	try {

		// MIG-076 §C — props go to the model (with the auto-date applied); the
		// disk write is composed from the model ALONE, so the body is the live
		// one the editor pane maintains, NOT the (possibly stale) `body` param
		// PropertyEditor passed. A path mismatch is REFUSED, never composed.
		// Body for the embed comes from the live model (a prop-save doesn't touch it).
		const embedBody = SINGLE_OWNERSHIP ? (getNoteModel(tabId)?.body.toString() ?? body) : body;
		// The post-DURABLE-write side effects — screen sync, reindex, embed, recents.
		// Run ONLY after the write lands (onSaved) — never on a failed write.
		const runPostSave = (savedPath: string) => {
			emit('screen:note-saved', { path: savedPath }).catch(() => {});
			const tab = get(openTabs).find(t => t.path === savedPath);
			if (tab) {
				invoke('constellation_search_reindex', { notePath: savedPath, libraryName: tab.libraryName }).catch(() => {});
			}
			if (get(appSettings).enabledFeatures?.semanticSearch && tab) {
				invoke('constellation_embed_notes', {
					notes: [{ path: savedPath, name: tab.name, content: embedBody }],
					force: !bodyUnchanged,
				}).catch(() => {});
			}
			// Track as recently edited for the second-screen dashboard.
			try {
				const key = 'constellation-recent-edited';
				const existing: { name: string; path: string; libraryName: string; editedAt: number }[] = JSON.parse(localStorage.getItem(key) || '[]');
				if (tab) {
					const filtered = existing.filter(n => n.path !== savedPath);
					filtered.unshift({ name: tab.name, path: savedPath, libraryName: tab.libraryName, editedAt: Date.now() });
					localStorage.setItem(key, JSON.stringify(filtered.slice(0, 20)));
				}
			} catch {}
			setTimeout(() => recentWrites.delete(savedPath), 2000);
		};
		// Do NOT update the store during autosave — it triggers a full reactivity
		// cascade. The editor owns the content; the store syncs on tab switch / reload.
		recentWrites.set(filePath, Date.now());
		if (SINGLE_OWNERSHIP) {
			// props already pushed to the model above (before the guard). The write
			// composes from the model ALONE (the live body, not the possibly-stale
			// `body` param). Save-Durability gate: mark clean ONLY on a durable write;
			// on failure the model stays dirty, the net is retained, and the save-health
			// banner surfaces it (no false-clean, no silent loss).
			const tab = get(openTabs).find(t => t.path === filePath);
			await saveNoteSession(tabId, filePath, standardSaveEnv({
				origin: 'prop_save',
				name: tab?.name ?? filePath,
				onSaved: (savedPath) => runPostSave(savedPath),
			}), 'prop_save');
		} else {
			const newContent = buildFullContent(updatedProps, body);
			await writeNote(filePath, newContent, 'prop_save');
			runPostSave(filePath);
		}
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
		// Sweep-2026-07-18 #2 (B1) — one path → one tab, ALSO on Alt+←/→. If the target is
		// ALREADY open in ANOTHER tab, activate that tab instead of re-seeding THIS tab onto
		// the same path — which would mint a SECOND independent NoteModel for one file (models
		// are keyed by tab id), the exact two-writers-last-wins clobber B1 kills for openNoteTab.
		// Runs BEFORE the flush: on a dedup this tab keeps its own note (it doesn't depart), so
		// there is nothing to flush. Mirrors openNoteTab's dedup (store.ts:2100).
		// (Residual, shared with openNoteTab: this check is a single synchronous snapshot; a
		// concurrent open of the SAME path during the awaited flush/resolve below could still
		// mint a second model — an extremely narrow disk-latency TOCTOU, not introduced here.)
		if (DEDUP_ALL_TABS_ENABLED) {
			const existing = get(openTabs).find(t => t.path === filePath && t.id !== tabId);
			if (existing) {
				if (get(splitActive)) focusedTabId.set(existing.id); else activeTabId.set(existing.id);
				_traceNav('loadTabHistoryEntry:dedupExisting', existing.id, filePath);
				return;
			}
		}
		// Sweep-2026-07-18 #10 — resolveNoteContent (NOT a raw read) so a note whose ONLY copy
		// of unsaved edits lives in the write-ahead net (a failed save whose tab was closed) is
		// recovered on Alt-nav — the documented reopen-restore route, previously bypassed here.
		// Mirrors openNoteTab (store.ts:2121); displayOnlyWindow preserves the shared net (PJ-108).
		//
		// PJ-187 — the READ comes FIRST, exactly as openNoteTab orders it. When the flush ran
		// first, the incoming note's disk read sat between the flush and the model re-seed, and
		// every keystroke typed during that read landed in a model that was about to be replaced
		// — typed after the flush captured the body, discarded by openNoteModel below. The flush
		// now sits immediately before the synchronous swap, which is the audited ordering.
		const resolved = await resolveNoteContent(filePath, { preserveNet: displayOnlyWindow });
		if (resolved === null) { _traceNav('loadTabHistoryEntry:unreadable', tabId, filePath); return; }
		// ★ Inspection 2026-07-29 — the read above CONSUMED the incoming note's write-ahead
		// entry (its only copy of a failed save's work). Every abort below this line must put
		// it back, or an aborted nav silently destroys the recovery the net exists to provide.
		// Unflagged on re-stash: it is real unsaved work, exactly as it was stored.
		const restashConsumedNet = () => {
			if (resolved.recoveredFromNet) {
				setWriteAhead(filePath, resolved.content, resolved.cursorPos ?? 0, resolved.scrollTop ?? 0);
			}
		};
		// If a later nav has superseded this one, don't stomp its result.
		if (_navTokens.get(tabId) !== myToken) { restashConsumedNet(); return; }
		// Alt+←/→ is a DEPARTURE: flush the OUTGOING dirty model to disk before this tab's
		// id-slot is re-seeded below (openNoteModel). Sources the old path from the model. A
		// failed flush ABORTS the nav (user stays on the note + save-health banner); a nav that
		// superseded us during the flush also bails.
		if (NAV_FLUSH_ENABLED) {
			const f = await flushOutgoing(tabId, 'nav_flush');
			if (!f.ok) { restashConsumedNet(); _traceNav('loadTabHistoryEntry:flushAbort', tabId, filePath); return; }
			if (_navTokens.get(tabId) !== myToken) { restashConsumedNet(); return; } // superseded during the flush await
		}
		const content = resolved.content;

		// Name: mirror openNoteTab (shared deriveTabName). Without this parity
		// the tab label flips between conventions as the user navigates
		// forward (click) vs back (history).
		const name = deriveTabName(filePath, content);

		// Resolve library for the new path so cross-library history entries
		// (or any future cross-library nav) don't keep the old library's
		// name/path on the tab.
		const { library: resolvedLibrary } = deriveLibraryForPath(filePath);

		openTabs.update(tabs => tabs.map(t => {
			if (t.id !== tabId) return t;
			return {
				...t,
				path: filePath,
				content,
				name,
				historyIndex: newHistoryIndex,
				cursorPos: resolved.cursorPos,
				scrollTop: resolved.scrollTop,
				highlightTerm: undefined,
				...(resolvedLibrary ? { libraryName: resolvedLibrary.name, libraryPath: resolvedLibrary.path } : {}),
			};
		}));
		openNoteModel(tabId, filePath, content); // MIG-076 §C — Alt-nav reuse drives the model synchronously
		// #10 — net-recovered content is UNSAVED work: born DIRTY with the true disk baseline, so
		// the autosave/retry persists it and switching away can't silently lose it (mirrors openNoteTab).
		if (resolved.recoveredFromNet && !displayOnlyWindow) markModelRecoveredFromNet(tabId, resolved.diskContent ?? null);
		_traceNav('loadTabHistoryEntry:applied', tabId, filePath);
	} catch { /* file may have been deleted */ }
}

// ─── Bookmarks ─── (MIG-092: unified into Collections as the pinned "Starred"
// collection — see collectionSets / toggleStarred below. The legacy
// bookmarks.json is read once by loadCollections for the one-time migration and
// then retained as a backup; nothing writes it anymore.)

// ─── MIG-092 — Collections (persistent hand-picked working sets) ───
// The ONE hand-picked-set mechanism (unifies the former Bookmarks as the
// pinned "Starred" collection). Membership ONLY lives here (the MIG-077 B1
// stale-snapshot cure): every displayed NOTE fact is re-read from the index
// via collections_hydrate. Note members are added path-keyed and SELF-UPGRADE
// to cid-keyed on first hydration (adoptCollectionIdentities) — cid_cn
// survives renames AND folder moves; adding a note NEVER writes its file.
// Folder / saved-search members (from the former Bookmarks) carry inline
// facts and are never hydrated. The pure reducers live in ./collectionsLogic
// (unit-tested there); this shell adds the writable + save-on-change IPC.
export type { CollectionItemType, CollectionItem, Collection } from './collectionsLogic';
export { STARRED_ID, COLLECTION_ITEM_CAP, collectionKey } from './collectionsLogic';

export const collectionSets = writable<CL.Collection[]>([]);

/**
 * PJ-187 — collections membership lives ONLY here (the comment above says so: *"Membership
 * ONLY lives here"*, and adding a note never writes its file). So this is the sole writer of
 * user-authored data, and it had two independent ways to lose it:
 *
 *  1. it was FIRE-AND-FORGET with a `console.error` — which release builds discard entirely
 *     (`feedback_devtools_dev_only`). Starring a note appeared to work and was gone next boot.
 *  2. worse, `loadCollections` swallowed a failed READ and left the store at its empty
 *     default — so the next star/unstar wrote that emptiness over a perfectly good file.
 *     A momentary lock from a sync tool was enough to erase every collection.
 *
 * Now: the load must SUCCEED before any write is permitted (`collectionsLoaded`), the write
 * is awaited with one retry, and a final failure raises a visible state instead of a log line
 * nobody sees. Refusing to write is the safe direction — it can only ever lose the change the
 * user just made, never the ones they made before.
 */
let collectionsLoaded = false;

/** PJ-187 — surfaced when collections could not be read or written; the panel shows it
 *  rather than silently presenting an empty list as the truth. */
export const collectionsError = writable<string | null>(null);

// ★ Inspection 2026-07-29 — saves are SINGLE-FLIGHT, and each reads the store at ITS turn.
// The first version snapshotted the payload at call time: two rapid toggles could interleave
// as save1(old) → save2(new) → save1's RETRY(old), leaving the file missing the newer change.
// Chaining serialises the writes, and reading `collectionSets` inside the queued task means
// the last write to land always carries the newest list.
let _collectionsSaveChain: Promise<void> = Promise.resolve();

function saveCollections(): Promise<void> {
	if (!collectionsLoaded) {
		// Never overwrite a file we failed to read — that is how the whole set is lost.
		collectionsError.set('not-loaded');
		console.warn('[collections] refusing to save: the collections file was never read successfully');
		return Promise.resolve();
	}
	const run = _collectionsSaveChain.then(async () => {
		const payload = get(collectionSets); // read at WRITE time, after prior writes settled
		for (let attempt = 0; attempt < 2; attempt++) {
			try {
				await invoke('save_universe_collections', { collections: payload });
				collectionsError.set(null);
				return;
			} catch (e) {
				if (attempt === 1) {
					collectionsError.set(String(e));
					console.error('[save] collections failed after retry:', e);
				}
			}
		}
	});
	_collectionsSaveChain = run.catch(() => {}); // a failed save never wedges the chain
	return run;
}

/** Create a new named collection; returns its id. */
export function createCollection(name: string): string {
	const id = `col_${Date.now()}`;
	collectionSets.update(list => CL.createSet(list, id, name, Date.now()));
	saveCollections();
	return id;
}

export function renameCollection(id: string, name: string) {
	collectionSets.update(list => CL.renameSet(list, id, name));
	saveCollections();
}

/** Delete a collection. The pinned Starred collection cannot be deleted. */
export function deleteCollection(id: string) {
	collectionSets.update(list => CL.deleteSet(list, id));
	saveCollections();
}

/** Add an item to a collection. Dedupes by (type,path); capped. Returns true if added. */
export function addToCollection(
	setId: string,
	item: { type?: CL.CollectionItemType; path: string; name?: string; libraryName?: string }
): boolean {
	let added = false;
	collectionSets.update(list => {
		const r = CL.addItem(list, setId, item, Date.now());
		added = r.added;
		return r.list;
	});
	if (added) saveCollections();
	return added;
}

export function removeFromCollection(setId: string, key: string) {
	collectionSets.update(list => CL.removeItem(list, setId, key));
	saveCollections();
}

// ── Starred (the former Bookmarks) — the quick-add favorites shelf ──
export function addToStarred(item: {
	type?: CL.CollectionItemType;
	path: string;
	name?: string;
	libraryName?: string;
}): boolean {
	return addToCollection(CL.STARRED_ID, item);
}
export function removeFromStarred(key: string) {
	removeFromCollection(CL.STARRED_ID, key);
}
export function isInStarred(path: string): boolean {
	return get(collectionSets).some(c => c.id === CL.STARRED_ID && c.items.some(i => i.path === path));
}
/** Toggle a target's Starred membership by PATH (the ⭐ / bookmark gesture).
 *  Removes by the existing item's key (cid or path) so cid-adopted notes still
 *  toggle off correctly. */
export function toggleStarred(item: { type?: CL.CollectionItemType; path: string; name?: string; libraryName?: string }) {
	const starred = get(collectionSets).find(c => c.id === CL.STARRED_ID);
	const existing = starred?.items.find(i => i.path === item.path);
	if (existing) removeFromCollection(CL.STARRED_ID, CL.collectionKey(existing));
	else addToStarred(item);
}

/** Hydration returned authoritative rows: adopt cids for path-keyed NOTE items
 *  (self-upgrade to rename-proof identity) and refresh moved paths. Saves only
 *  when something changed. */
export function adoptCollectionIdentities(rows: { key: string; path: string; cid_cn: string }[]) {
	let changed = false;
	collectionSets.update(list => {
		const r = CL.adoptIdentities(list, rows);
		changed = r.changed;
		return r.list;
	});
	if (changed) saveCollections();
}

/** Rename hook — path-keyed membership follows in-app renames/moves (wired in
 *  handleRenameComplete beside the other path-keyed migrations). cid-keyed note
 *  items self-heal via hydration anyway. */
export function migrateCollectionPath(oldPath: string, newPath: string) {
	let changed = false;
	collectionSets.update(list => {
		const r = CL.migratePath(list, oldPath, newPath);
		changed = r.changed;
		return r.list;
	});
	if (changed) saveCollections();
}

/** Load collections membership post-paint. One-time migration: when there is no
 *  Starred collection yet, seed it from the former bookmarks.json (preserving
 *  note/folder/search targets) — idempotent (CL.migrateBookmarks no-ops when
 *  Starred exists); the legacy bookmarks.json is left intact as a backup. */
export async function loadCollections() {
	// ★ Inspection 2026-07-29 — reset BOTH the permission latch and the list BEFORE reading.
	// Without this, a universe SWITCH whose read fails kept `collectionsLoaded = true` and the
	// store still holding the PREVIOUS universe's collections — so the next star wrote
	// universe A's collections over universe B's file. The latch must mean "THIS universe's
	// file was read", never "some universe's file was once read". (No subscriber auto-saves
	// on `collectionSets`, so the reset itself can never trigger a write.)
	collectionsLoaded = false;
	collectionSets.set([]);
	try {
		const data = await invoke<unknown[]>('read_universe_collections');
		const base = (Array.isArray(data) ? data : []) as CL.Collection[];
		let bms: Array<{ type?: CL.CollectionItemType; path: string; name?: string; libraryName?: string }> = [];
		try {
			const b = await invoke<typeof bms>('read_universe_bookmarks');
			if (Array.isArray(b)) bms = b;
		} catch { /* no bookmarks to migrate */ }
		const r = CL.migrateBookmarks(base, bms, Date.now());
		collectionSets.set(r.list);
		// PJ-187 — the read SUCCEEDED, so writes are now permitted. Until this line runs,
		// `saveCollections` refuses: an empty store after a FAILED read is indistinguishable
		// from a genuinely empty one, and writing it back destroys every collection the user
		// had. This flag is the difference between "you have none" and "I could not read it".
		collectionsLoaded = true;
		collectionsError.set(null);
		if (r.migrated) void saveCollections(); // persist the seeded Starred so it never re-runs
	} catch (e) {
		// Do NOT set collectionsLoaded. The store keeps its default and the panel shows the
		// error state instead of presenting emptiness as the truth.
		collectionsError.set(String(e));
		console.error('[collections] read failed — saves are disabled until a successful load:', e);
	}
}

/** Hydrate a collection's NOTE members' live facts (folder/search members are
 *  skipped — they render from inline facts). Adopts cids for path-keyed notes
 *  as a side effect (rename-proof self-upgrade). Empty on any failure. */
export async function hydrateCollectionNotes(items: CL.CollectionItem[]): Promise<CL.HydratedNoteRow[]> {
	const { cids, paths } = CL.noteHydrationKeys(items);
	if (cids.length === 0 && paths.length === 0) return [];
	try {
		const rows = await invoke<CL.HydratedNoteRow[]>('collections_hydrate', { cids, paths });
		if (Array.isArray(rows) && rows.length > 0) {
			adoptCollectionIdentities(rows.map(r => ({ key: r.key, path: r.path, cid_cn: r.cid_cn })));
		}
		return Array.isArray(rows) ? rows : [];
	} catch {
		return [];
	}
}

// ─── Frontmatter parsing ───

/** Strip one trailing CR so a CRLF note's lines test identically to an LF note's.
 *  `content.split('\n')` leaves the `\r` attached, and `\r` IS `\s`, which is how a
 *  CR-only line used to satisfy the old `/^\s/` nested-map probe. */
const noCr = (l: string) => (l.endsWith('\r') ? l.slice(0, -1) : l);

/**
 * PJ-182 — a YAML block-SEQUENCE item: `- x`, `  - x`, `-\tx`, or a bare `-`.
 *
 * **Indentation is deliberately NOT part of this test.** YAML 1.2 lets a block sequence
 * sit at the SAME indentation as its parent mapping key —
 *
 *     tags:
 *     - alpha        ← column 0. Valid, and ordinary in hand-authored and imported vaults.
 *
 * What identifies a sequence item is the DASH: a mapping key can never begin with one.
 * Every scanner in the app that asked "is this line indented?" instead of "does it start
 * with a dash?" mis-read that shape — the frontend read the list as EMPTY (so the next
 * write to that key deleted every item from the .md), and the Rust writers spliced their
 * own indented item in beside the user's column-0 ones, producing frontmatter that no
 * longer parses at all.
 *
 * `search.rs::parse_frontmatter` has always used the dash rule, and says so in a comment:
 * *"a line beginning `- ` is a LIST ITEM, never a key."* PJ-182 is what it cost to have
 * written that rule in exactly one of the nine places that needed it. This helper exists
 * so the rule has ONE home on the JS side (LL-038 rule 5 — two representations of one
 * truth will eventually disagree); `is_seq_item` in `src-tauri/src/yaml_lines.rs` is its
 * Rust twin. Keep the two honest about each other — the `/simplify` pass on this very
 * change caught them already disagreeing about comments inside a block.
 */
export function isYamlSeqItem(line: string): boolean {
	return yamlSeqItemValue(line) !== null;
}

/**
 * The PAYLOAD of a sequence item — the text after the dash — or `null` if `line` is not
 * one. A bare `-` (an empty entry) yields `''`.
 *
 * Paired with the predicate on purpose: the dash-strip was spelled out four separate times
 * in this file, which is the same "several representations of one truth" the predicate
 * exists to end. Its Rust twin is `yaml_lines::seq_item_value`.
 */
export function yamlSeqItemValue(line: string): string | null {
	const m = /^[ \t]*-([ \t]|$)/.exec(noCr(line));
	if (!m) return null;
	return noCr(line).replace(/^[ \t]*-[ \t]*/, '');
}

/** A YAML comment line, at any indentation. */
const isYamlComment = (l: string) => /^[ \t]*#/.test(noCr(l));

/** Leading whitespace — an indented continuation line, not a top-level key. */
const isIndentedLine = (l: string) => /^[ \t]/.test(noCr(l));

/** A line that belongs to the block under a top-level `key:` — indented, a zero-indent
 *  sequence item, or a comment. Blank lines are handled by the caller (they may be
 *  interior to a block, so they neither continue nor end it on their own). */
function isYamlBlockChild(line: string): boolean {
	return isIndentedLine(line) || isYamlSeqItem(line) || isYamlComment(line);
}

/** Blank (or whitespace/CR only). */
const isBlankLine = (l: string) => /^\s*$/.test(l);

/** A YAML block-scalar header: `|` or `>`, with optional indentation and chomping
 *  indicators (`|-`, `>2`, `|2-`, `|+`). The BODY is on the indented lines that follow. */
const isBlockScalarHeader = (v: string) => /^[|>][0-9+-]{0,2}$/.test(v.trim());

/** Strip one layer of matching quotes, if present. */
const unquote = (v: string) =>
	(v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'")) ? v.slice(1, -1) : v;

/**
 * Index one past the block's last non-blank child line, with trailing blanks dropped.
 * `isChild` decides what belongs — the block-scalar body wants indented lines only, an
 * ordinary block wants sequence items and comments too.
 */
function blockExtent(lines: string[], start: number, isChild: (l: string) => boolean): number {
	let last = start - 1;
	for (let e = start; e < lines.length; e++) {
		if (isBlankLine(lines[e])) continue; // blank: may be interior
		if (!isChild(lines[e])) break; // a top-level key — the block is over
		last = e;
	}
	return last + 1;
}

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
		const idx = i;
		const line = yamlLines[idx];
		// Advance FIRST, so a branch that consumes a block just says how far it got and
		// every `continue` below is correct without a trailing increment to reconcile
		// against. (The alternative — `i = end - 1` so a tail `i++` lands right — is the
		// kind of off-by-one that only reads correctly if you hold the whole loop in mind.)
		i = idx + 1;
		const colonIdx = line.indexOf(':');

		// PJ-182 — a line beginning with a dash is a SEQUENCE ITEM, never a key, at any
		// indentation. Without this, `- name: X` at column 0 has a colon and no leading
		// space, so it was admitted here as a top-level property literally named `- name`
		// — one phantom row per list item, and the real key left showing empty.
		if (colonIdx > 0 && !isYamlSeqItem(line) && !line.startsWith(' ') && !line.startsWith('\t')) {
			const key = line.substring(0, colonIdx).trim();
			let value = line.substring(colonIdx + 1).trim();

			// ── BLOCK SCALAR (`key: |`, `key: >`, with optional chomp/indent indicators) ──
			// PJ-182 — the block gate below requires an EMPTY inline value, but a block
			// scalar's value is the indicator itself (`|`), so none of it ran: the key
			// projected as editable TEXT whose value was the literal character `|`, and its
			// body lines were skipped by the top-level-key guard. `buildFullContent` then
			// wrote `desc: "|"` into the tab.content cache — the prose, gone.
			//
			// Same ruling as every other shape we cannot round-trip: READ-ONLY, bytes
			// verbatim. `value` keeps the indicator so `reconstructFrontmatter` can re-emit
			// `key: |` exactly; the body rides in `nestedRaw`.
			if (isBlockScalarHeader(value) && idx + 1 < yamlLines.length && isIndentedLine(yamlLines[idx + 1])) {
				const blockStart = idx + 1;
				const end = blockExtent(yamlLines, blockStart, isIndentedLine);
				if (key && end > blockStart) {
					const raw = yamlLines.slice(blockStart, end);
					i = end;
					// One chip previewing the first line, so the row is not drawn as "Empty"
					// for a property that holds prose.
					const first = noCr(raw[0]).trim();
					properties.push({
						key,
						value,
						type: 'nested-map',
						nestedKeys: first ? [first.length > 40 ? `${first.slice(0, 40)}…` : first] : [],
						nestedRaw: raw,
					});
					continue;
				}
			}

			// ── The BLOCK under `key:` ────────────────────────────────────────────────
			// PJ-182 — take the block's FULL extent ONCE, then decide what it is. This
			// used to be three separate branches (ikhtilāf / nested-map / flat list), each
			// with its OWN probe of the next line, and all three probes required a LEADING
			// SPACE (`/^\s+-\s/`, `/^\s/`). A zero-indent block sequence satisfied none of
			// them, so the key fell through to the scalar path and projected EMPTY — and
			// an empty EDITABLE list is a data-destroying invitation: the next write to
			// that key spliced the block out and rewrote it from nothing, deleting every
			// item from the .md with no error and re-parsing cleanly afterwards.
			//
			// One extent scan, one classification. Three probes of one truth are three
			// chances to disagree (LL-038 rule 5), and they did.
			if (!value) {
				const blockStart = idx + 1;
				const end = blockExtent(yamlLines, blockStart, isYamlBlockChild);
				const blockLines = yamlLines.slice(blockStart, end);
				const contentLines = blockLines.filter((l) => !isBlankLine(l));

				// A run of comments alone is NOT a block — a `#` line between two
				// top-level keys belongs to the file, not to the key above it. Comments
				// that sit among real content DO belong to the block, and their presence
				// is what makes it un-round-trippable, hence read-only. (An EMPTY block
				// satisfies `every` vacuously, which is the same answer: not a block.)
				const onlyComments = contentLines.every(isYamlComment);

				// PJ-182 — a FLOW sequence written on the line after its key
				// (`tags:` then `  [alpha, beta]`) is an ordinary list of scalars. It was
				// reaching the read-only branch, which left the user unable to edit their
				// own tags while the CST parser read them perfectly well. Hand it to the
				// inline-list path below by treating it as the key's value.
				const flowOnly = contentLines.length === 1 ? noCr(contentLines[0]).trim() : '';
				if (key && flowOnly.startsWith('[') && flowOnly.endsWith(']')) {
					value = flowOnly;
					i = end; // consumed — the scalar path below reads the new `value`
				} else if (key && !onlyComments) {
					i = end;

					// A flat list is every content line being a `- item` whose text is not
					// itself a `key: value` map entry. `- https://x` and `- "a: b"` are
					// scalars, not maps — the test requires a colon followed by space/EOL
					// on an UNQUOTED item, which is YAML's own rule. A comment line is
					// neither, and it is user data the flat projection cannot round-trip,
					// so its presence alone demotes the block to read-only.
					const allSeqItems = contentLines.every(isYamlSeqItem);
					const anyMapItem = contentLines.some((l) =>
						/^[ \t]*-[ \t]*[^"'\s][^:]*:([ \t]|$)/.test(noCr(l)),
					);

					// MIG-022 §A.1 — nested-object-list (e.g. ikhtilāf):
					//   ikhtilāf:
					//     - school: Hanafī
					//       position: permissible
					//     - school: Mālikī
					//       position: discouraged
					// A seq-of-maps under one of the keys the system round-trips LOSSLESSLY
					// (STRUCTURED_LIST_KEYS — the serializer has a real branch for it and
					// the panel edits it through a structured widget). Each row gathers its
					// continuation lines — indented, no leading dash — into one record.
					const isIkhtilaf = IKHTILAF_KEYS.has(key) || IKHTILAF_KEYS.has(key.toLowerCase());
					if (isIkhtilaf && anyMapItem) {
						const nestedObjects: Array<Record<string, string>> = [];
						let r = 0;
						const readField = (text: string, obj: Record<string, string>) => {
							const c = text.indexOf(':');
							if (c <= 0) return;
							const fkey = text.substring(0, c).trim();
							if (fkey) obj[fkey] = unquote(text.substring(c + 1).trim());
						};
						while (r < blockLines.length) {
							const cur = blockLines[r];
							const rowStart = yamlSeqItemValue(cur);
							if (rowStart === null) { r++; continue; }
							const obj: Record<string, string> = {};
							readField(rowStart, obj);
							r++;
							// Continuation lines: indented, no leading dash, until the next row.
							while (r < blockLines.length) {
								const cont = blockLines[r];
								if (isYamlSeqItem(cont)) break; // next row
								if (!isIndentedLine(cont)) break; // back to top level
								readField(noCr(cont), obj);
								r++;
							}
							nestedObjects.push(obj);
						}
						// Compact display string for legacy consumers + search:
						// "Hanafī: permissible | Mālikī: discouraged"
						const summary = nestedObjects
							.map((o) => Object.entries(o).map(([k, v]) => `${k}: ${v}`).join(' / '))
							.join(' | ');
						properties.push({ key, value: summary, type: 'nested-object-list', nestedObjects });
						continue;
					}

					if (allSeqItems && !anyMapItem) {
						const listItems = contentLines.map((l) => unquote((yamlSeqItemValue(l) ?? '').trim()));
						properties.push({ key, value: listItems.join(', '), type: 'list', listItems });
						continue;
					}

					// Anything we cannot round-trip as a flat list — a nested MAP, a
					// seq-of-maps, a block holding a comment — is projected READ-ONLY with
					// its bytes carried verbatim in `nestedRaw`.
					//
					// PJ-136 / the 2026-07-24 inspection, and the Boss's 2026-07-22 ruling:
					// a TRUNCATED or EMPTY projection of an EDITABLE type is not a display
					// bug, it is an invitation — the panel draws a chip list missing half
					// its content, and the moment the user touches it `composeFrontmatter`
					// splices the whole block out and rewrites it from what we captured.
					// Visible and honest beats hidden or editable. `value` stays EMPTY so
					// the legacy `reconstructFrontmatter` (still live behind
					// `buildFullContent`) serializes this key byte-identically; the summary
					// rides in `nestedKeys`.
					const children = blockLines
						.map((l) => {
							const m = /^[ \t]*(?:-[ \t]*)?([^:]+):(?:[ \t]|$)/.exec(noCr(l));
							return m ? m[1].trim() : '';
						})
						.filter(Boolean);
					properties.push({ key, value: '', type: 'nested-map', nestedKeys: children, nestedRaw: blockLines });
					continue;
				}
			}

			// Strip quotes
			value = unquote(value);

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
	}

	const body = lines.slice(endIndex + 1).join('\n');
	return { properties, body, rawYaml };
}

/** MIG-022 §A.1 — shared YAML value quoter. Used by reconstructFrontmatter
 *  for both flat values and nested-object-list field values. Strings with
 *  YAML special chars get double-quoted with embedded `"` escaped.
 *
 *  PJ-187 — EXPORTED, because two other places were hand-building a `key: value`
 *  frontmatter line by concatenation and emitting invalid YAML for any value needing
 *  quotes: `rebrandCopyFrontmatter` above, and the template merge in `+layout.svelte`.
 *  One quoter for everything that emits a frontmatter line. */
export function quoteIfNeeded(v: string): string {
	if (v === '') return '""';
	const needsQuoting = /[:{}\[\],&*?|>!%@`#]/.test(v) ||
		// PJ-187 — a leading SEQUENCE INDICATOR. `- ` (dash + space) or a bare `-` opens a
		// block sequence, so `title: - dash lead` is not a string at all; the parser reports
		// two errors on it. Found by routing the recovery-copy title through this quoter and
		// then through the real parser — the quoter had covered every other indicator
		// character and not this one. Note `--dashes--` is a perfectly ordinary plain scalar
		// (only dash-SPACE and a lone dash are indicators), so it stays unquoted.
		/^-(\s|$)/.test(v) ||
		// Leading or trailing whitespace does not survive a plain scalar — it is stripped on
		// read, so a value that depends on it must be quoted to round-trip.
		v !== v.trim() ||
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
				// MIG-086 Part 2 §F2 — quote list items that need it (e.g. wikilinks
				// `[[X]]`, whose `[` would otherwise start a YAML flow sequence → invalid
				// YAML). quoteIfNeeded is a no-op for plain items (tags etc.), and the
				// block-list parser strips the quotes back on read, so the round-trip is
				// byte-stable. This also fixes pre-existing special-char list items (`#`, `:`).
				lines.push(`  - ${quoteIfNeeded(item)}`);
			}
		} else if (prop.type === 'nested-map') {
			// PJ-136 — re-emit the block EXACTLY as it was read. Serializing this prop
			// from `value` would write a bare `key:` and drop every child, which is what
			// made the cached `tab.content` lossy in the first place.
			//
			// PJ-182 — a BLOCK SCALAR is carried by this same read-only type, and its
			// header indicator lives in `value` (`|`, `>-`, …). Emitting a bare `key:`
			// there would turn prose into a mis-parsed nested block, so the indicator is
			// re-emitted when present. Ordinary nested maps have an empty `value` and are
			// unaffected.
			lines.push(prop.value ? `${prop.key}: ${prop.value}` : `${prop.key}:`);
			for (const raw of prop.nestedRaw ?? []) lines.push(raw);
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

/**
 * G4 Phase 3 — the byte-perfect replacement for `buildFullContent` on a ROUND-TRIP
 * write: a note READ from disk whose props are edited and written back (closed-note
 * tag/link adds, etc.). `buildFullContent` rebuilds the frontmatter from the lossy
 * projection and silently drops nested maps / block scalars / corrupts quoted values;
 * this instead diffs `newProps` against the ORIGINAL content's projection and applies
 * ONLY the changes to the real bytes via the yamlDoc CST, so unedited rich frontmatter
 * is preserved verbatim (the SAME engine the open-editor save path uses). Consistency,
 * not losslessness: both sides use this module's `parseFrontmatter` projection, so an
 * unedited key produces no diff and is left byte-perfect.
 *
 * Use for any read→edit-props→write of an EXISTING note. Use `buildFullContent` only
 * for a genuinely NEW note (no original bytes to preserve).
 */
export function composeUpdatedContent(originalContent: string, newProps: FrontmatterProperty[], newBody?: string): string {
	const { yaml, hadFence } = splitFrontmatter(originalContent);
	const parsed = parseFrontmatter(originalContent);
	return composeFrontmatter(yaml, hadFence, parsed.properties, newProps, newBody ?? parsed.body);
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
			// PJ-106 §B4 — direction marks (RLM/LRM) are stripped: a heading forced RTL keeps
			// its identity in the outline, anchors, and [[note#heading]] targets.
			const text = match[2].replace(/[#*`\[\]‎‏]/g, '').trim();
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
async function resolveNoteContent(
	filePath: string,
	opts: { preserveNet?: boolean; requireDisk?: boolean } = {}
): Promise<{ content: string; cursorPos: number; scrollTop: number; recoveredFromNet?: boolean; diskContent?: string | null } | null> {
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
	// MIG-100 hotfix (Boss Stage-2 failure 3): the RESTORE path requires the
	// FILE to exist — a wab entry alone must not resurrect a tab at a dead
	// path (every viewed note leaves a teardown-re-stashed wab; a note moved
	// while the app was closed came back as a ghost). The net is left intact:
	// a manual open (no requireDisk) can still recover real unsaved content.
	if (opts.requireDisk && diskContent === null) return null;
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
		// PJ-181 (APP-KILLER) — IDENTITY is not FRESHNESS. `cid_cn` is the note's identity
		// and is unchanged by an edit, so every check above passes for a note edited
		// OUTSIDE Constellation (Syncthing, a second device, `git pull`) — while the net
		// still holds the copy left by merely VIEWING it (NoteEditor's teardown stashes one
		// even when nothing was typed, and nothing clears it for a closed note). The stale
		// view then won the screen, the model was born DIRTY with it, and the first tab
		// switch wrote it over the newer file and reindexed on it. Reproduced end to end:
		// the flush returned `{ok:true}` and the externally-added paragraph was gone.
		//
		// The entry now says which it is. `snapshot` means its content was ALREADY DURABLE
		// when it was stashed — so the bytes it holds ARE the baseline it was taken against,
		// and if disk no longer matches them, someone else has written since and is simply
		// newer. A net holding real unsaved work (the PJ-102 failed-save recovery) is
		// untouched by this, and so is a legacy entry with no flag: unprovable → real work.
		const staleSnapshot = wab.snapshot === true && diskContent !== wab.content;
		if (!identityProven || emptyResurrection || staleSnapshot) {
			console.warn(
				'[resolveNoteContent] write-ahead-buffer rejected for', filePath,
				staleSnapshot ? '(stale snapshot — disk moved on)'
					: identityProven ? '(empty-body resurrection)' : '(identity unproven)',
				'— preferring disk',
			);
			// MIG-100 hotfix-inspection: the RESTORE path must not destroy the
			// net even on rejection — a cid-less recovery copy (deferred-cid
			// note whose save failed) is identity-unproven by construction,
			// yet may be the ONLY copy of unsaved edits. Disk wins the view;
			// the net stays for manual recovery.
			if (!opts.preserveNet) clearWriteAhead(filePath);
			return { content: diskContent, cursorPos: 0, scrollTop: 0 };
		}
	}
	// MIG-100 inspection fix (APP-KILLER): the RESTORE path must NOT consume
	// the net. A restored tab's model is born CLEAN — nothing would ever write
	// the recovered content to disk, and a background tab never mounts the
	// editor whose teardown re-stashes the net. With preserveNet the recovery
	// copy survives until a real durable save replaces it (the manual-open
	// path keeps today's consume semantics — its mounted editor re-stashes).
	if (!opts.preserveNet) clearWriteAhead(filePath);
	// PJ-102b — tell the caller this content came from the NET and what disk truly
	// holds, so the model can be born DIRTY with the real baseline (never "clean" on
	// content disk never had — the lie behind the recovery-then-clobber arc).
	return { content: wab.content, cursorPos: wab.cursorPos, scrollTop: wab.scrollTop, recoveredFromNet: true, diskContent };
}

/**
 * Derive a note's containing library from the registered libraries.
 * Path-normalized, case-insensitive prefix match so Windows paths (\ vs /)
 * and case differences don't silently lose the library anchor — which would
 * break embed resolution for any note. Picks the LONGEST matching prefix so
 * nested libraries (e.g. "Universe" and "Universe/Project" both registered)
 * route each note to its immediate containing library.
 * Shared by openNoteTab and the MIG-100 session restore.
 */
function deriveLibraryForPath(filePath: string): { library: LibraryInfo | undefined; libraryPath: string } {
	const allLibraries = get(libraries);
	const normalize = (p: string) => normalizePathKey(p).replace(/\/+$/, '');
	const filePathNorm = normalize(filePath);
	let library: LibraryInfo | undefined;
	let bestLen = -1;
	for (const v of allLibraries) {
		const libNorm = normalize(v.path);
		if (filePathNorm === libNorm || filePathNorm.startsWith(libNorm + '/')) {
			if (libNorm.length > bestLen) { bestLen = libNorm.length; library = v; }
		}
	}
	return { library, libraryPath: library?.path ?? '' };
}

/**
 * Derive a tab's display name: prefer the frontmatter `title:`, fall back to
 * the filename stem. One helper — the label convention must not drift between
 * click-nav, history-nav, and session restore (a drift shows as the tab label
 * flipping as the user navigates forward vs back).
 */
function deriveTabName(filePath: string, content: string): string {
	let name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';
	const fmTitleMatch = content.match(/^---[\s\S]*?^title:\s*"?([^"\n]+)"?\s*$/m);
	if (fmTitleMatch?.[1]) name = fmTitleMatch[1].trim();
	return name;
}

export async function openNoteTab(filePath: string, libraryName: string, color: string = '#7c3aed', highlightTerm?: string, newTab?: boolean, fromNotePath?: string, targetLine?: number, preserveNet?: boolean) {
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

	// APP-KILLER #2 (B1) — one path → one tab. If this note is already open in ANY OTHER tab
	// (a background tab; the active-tab case is handled by the early-return above), activate
	// that tab instead of minting a second tab + second model for the same path — regardless
	// of newTab / Ctrl+click. Runs before the file read (no I/O needed to focus an open tab).
	if (DEDUP_ALL_TABS_ENABLED) {
		const existing = get(openTabs).find(t => t.path === filePath);
		if (existing) {
			if (get(splitActive)) focusedTabId.set(existing.id); else activeTabId.set(existing.id);
			if (highlightTerm) openTabs.update(tabs => tabs.map(t => t.id === existing.id ? { ...t, highlightTerm } : t));
			// The activated (previously background) tab mounts its editor → the one-shot jump fires on mount.
			if (targetLine && targetLine > 0) setPendingLineJump(existing.id, targetLine);
			_traceNav('openNoteTab:dedupExisting', existing.id, filePath);
			return;
		}
	}

	// Living Link System: record traversal when following a wikilink (fire-and-forget)
	// Deferred until we have the note's display name (extracted from content below)
	const _fromNotePath = fromNotePath;

	// PJ-108 (APP-KILLER) — the second screen is a DISPLAY-ONLY window: it never mounts a writable
	// editor to re-stash the net, so consuming the shared crash-recovery net here would silently
	// destroy a main-window note's only unsaved-save-failed copy. Every openNoteTab in such a
	// window preserves the net (restore, list-click, wikilink alike — Solve-the-Class); an explicit
	// preserveNet arg still wins. Writable hosts keep the manual-open consume-and-re-stash default.
	const resolved = await resolveNoteContent(filePath, { preserveNet: preserveNet ?? displayOnlyWindow });
	if (resolved === null) {
		// §B2-4 forensics: this silent return is what "I click the note and
		// nothing happens" looks like when a stale tree row points at a dead
		// path (e.g. a rename whose tree refresh never ran). Log it loudly.
		console.warn('[openNoteTab] unreadable path — stale tree row?', filePath);
		return;
	}
	let content = resolved.content;
	const cursorPos = resolved.cursorPos;
	const scrollTop = resolved.scrollTop;

	// Living Link identifier (cid_cn): injected lazily the first time a note is
	// opened. Adds a `cid_cn:` property to the note's YAML frontmatter with a
	// timestamp derived from the file's creation time. Migrates any legacy
	// `cid:` to `cid_cn:` on the same pass. Only markdown files get this; the
	// vault's original filenames are never touched.
	// MIG-TPL §1 — templates are EXEMPT: a mold must never be stamped with an identity (see
	// `isTemplatePath`). Opening one to edit it would otherwise inject `cid_cn:` into the file.
	if ((filePath.endsWith('.md') || filePath.endsWith('.markdown')) && !isTemplatePath(filePath)) {
		try {
			const updated = await invoke<string>('ensure_cid_cn_cmd', { filePath });
			// PJ-102 (APP-KILLER) — adopt the cmd's result ONLY when the content we
			// already hold LACKS a cid_cn. The cmd reads DISK and, when the cid already
			// exists, returns the disk content verbatim (canonical.rs) — so an
			// unconditional adopt silently swapped a net-RECOVERED content (which
			// resolveNoteContent had already consumed the recovery buffer for) back to
			// stale disk: the user's last unsaved edits vanished from screen, net, and
			// disk on the app's own documented recovery route. An identity-proven
			// recovery ALWAYS carries a cid_cn (that is what identity-proven means), so
			// this predicate protects exactly the recovery case while leaving the cmd's
			// two real jobs — inject a missing cid / migrate legacy `cid:` — untouched
			// (both only ever apply to cid_cn-less content).
			if (updated && updated !== content && !extractCidCn(content)) {
				content = updated;
				// PJ-187 — the injection WROTE the note's permanent identity to disk through the
				// gate, which marks the path watcher-suppressed, so nothing else tells the index
				// it changed. Until the user happened to edit and save that note, note_meta held
				// no cid_cn for it and every identity-keyed lookup — collection membership,
				// link resolution by identity — silently missed it. Fire-and-forget: the note is
				// already open and correct on screen; this only catches the index up.
				invoke('constellation_search_reindex', { notePath: filePath, libraryName }).catch(() => {});
			}
		} catch { /* non-fatal: CID stays absent, note still opens */ }
	}
	// For canonical files, extract title from frontmatter; fallback to filename stem
	const name = deriveTabName(filePath, content);

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

	const { library, libraryPath } = deriveLibraryForPath(filePath);
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
		// APP-KILLER #2 — this in-place reuse is a DEPARTURE from currentTab's note. Flush
		// its OUTGOING dirty model to disk BEFORE the openTabs.update + openNoteModel below
		// re-seed the tab to the new note (the old path is sourced from the model, which
		// still holds it here). A failed flush ABORTS the nav — keep the user on the note +
		// the save-health banner (never a silent discard). Shares _navTokens with
		// loadTabHistoryEntry so a click-nav and an Alt-nav on this tab supersede each other.
		if (NAV_FLUSH_ENABLED && isNoteDirty(currentTab.id)) {
			const navToken = (_navTokens.get(currentTab.id) ?? 0) + 1;
			_navTokens.set(currentTab.id, navToken);
			const f = await flushOutgoing(currentTab.id, 'nav_flush');
			// ★ Inspection 2026-07-29 — resolveNoteContent (above) already consumed the INCOMING
			// note's write-ahead entry; an abort here must put it back or the recovery is lost.
			const restashConsumedNet = () => {
				if (resolved.recoveredFromNet) {
					setWriteAhead(filePath, resolved.content, resolved.cursorPos ?? 0, resolved.scrollTop ?? 0);
				}
			};
			if (!f.ok) { restashConsumedNet(); _traceNav('openNoteTab:flushAbort', currentTab.id, filePath); return; }
			if (_navTokens.get(currentTab.id) !== navToken) { restashConsumedNet(); _traceNav('openNoteTab:superseded', currentTab.id, filePath); return; }
		}
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
		// PJ-102b — net-recovered content is UNSAVED work: born dirty + true disk baseline.
		// PJ-130 Batch 1 — but NOT in a display-only window. The net is shared crash-
		// recovery state OWNED by the main window that created it; a display exists to
		// SHOW disk content, so it opens clean and never claims the unsaved work as its
		// own. This keeps the second-screen model from being born dirty at all — the
		// belt to flushOutgoing's braces. Per-window flag: the main window is unaffected.
		if (resolved.recoveredFromNet && !displayOnlyWindow) markModelRecoveredFromNet(currentTab.id, resolved.diskContent ?? null);
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
	// PJ-102b — net-recovered content is UNSAVED work: born dirty + true disk baseline.
	if (resolved.recoveredFromNet && !displayOnlyWindow) markModelRecoveredFromNet(id, resolved.diskContent ?? null);
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

/**
 * MIG-100 — the ONE departure primitive for "leave every open tab" moments
 * (workspace restore, universe switch, universe create). Flush each dirty
 * departing model through the durability gate (a failed flush proceeds — the
 * write-ahead net + save-health banner hold the edit, closeTab's contract),
 * clear the tab stores, wait a tick so the unmounting panes' own teardown
 * flush has run, then dispose the models (a raw `openTabs.set([])` alone
 * leaks every NoteModel for the process lifetime).
 */
export async function flushDisposeClearTabs(origin: string): Promise<void> {
	const departing = get(openTabs);
	await flushAllDirtyTabs(origin);
	openTabs.set([]);
	activeTabId.set(null);
	focusedTabId.set(null);
	// Close-audit fix: the departing universe's deferred-cid watchers +
	// pending paths must not survive into the next universe (2 leaked store
	// subscriptions per switch otherwise, firing on every later activation).
	disposeDeferredCidEnsure();
	await tick();
	for (const t of departing) closeNoteModel(t.id);
}

/**
 * MIG-100 switch-flush fix (Boss re-test finding): flush every dirty open
 * model to disk — WITHOUT clearing or disposing anything. This must run
 * BEFORE `set_active_universe` flips the Rust active universe, because
 * `write_note` validates paths against the ACTIVE universe's libraries — a
 * departure flush that runs after the flip is rejected ("Couldn't save X"
 * banners for every dirty departing tab; the write-ahead net held the
 * content, but the write itself never landed). The Universe Manager calls
 * this via its onBeforeSwitch hook; the post-flip teardown then finds every
 * model clean and performs no writes at all.
 */
export async function flushAllDirtyTabs(origin: string): Promise<void> {
	for (const t of get(openTabs)) {
		if (NAV_FLUSH_ENABLED && isNoteDirty(t.id)) {
			try { await flushOutgoing(t.id, origin); } catch { /* net + banner hold it */ }
		}
	}
}

/**
 * PJ-103 — the app-close flush: the body of the 'session:final-flush'
 * graceful-close handshake. flushAllDirtyTabs plus the adversarial-review
 * hardening (2026-07-16, wf_5bb5c713):
 *  - BOUNDED RE-PASS: the window stays interactive during Rust's ≤5s close
 *    hold, so keystrokes can land in a tab AFTER its flush pass — one re-scan
 *    catches them before the ack.
 *  - RESIDUAL JOURNAL MARKER: a write that fails fast (locked .md) leaves the
 *    model dirty with only the save-health banner — which dies with the
 *    window milliseconds later. The marker makes a dirty-at-ack close
 *    journal-decidable (Charter class-1: never a silent loss).
 *  - AWAITED FTS REINDEX for the flushed notes: navFlushEnv's fire-and-forget
 *    reindex is killed by the destroy, leaving disk NEWER than the index
 *    across the boot (the app's own write is watcher-suppressed, so no later
 *    event heals it). Embeddings stay fire-and-forget — bounded staleness,
 *    healed at the note's next save.
 * Instant when clean: zero dirty models → returns after one synchronous scan.
 */
export async function flushAllForAppClose(): Promise<void> {
	const dirty = get(openTabs).filter((t) => NAV_FLUSH_ENABLED && isNoteDirty(t.id));
	if (dirty.length === 0) return;
	await flushAllDirtyTabs('final_flush');
	if (get(openTabs).some((t) => isNoteDirty(t.id))) {
		await flushAllDirtyTabs('final_flush_repass');
	}
	const residual = get(openTabs).filter((t) => isNoteDirty(t.id)).length;
	if (residual > 0) {
		// Awaited — a fire-and-forget marker could be cut by the destroy,
		// which would defeat its whole purpose (journal-decidability).
		await invoke('journal_frontend_marker', {
			surface: 'final_flush_residual_dirty',
			detail: `${residual} note(s) still dirty at close ack`,
		}).catch(() => {});
	}
	await Promise.all(
		dirty.map((t) =>
			invoke('constellation_search_reindex', { notePath: t.path, libraryName: t.libraryName })
				.catch(() => {})
		)
	);
}

// ─── MIG-100 §3 — session restore: the batch-insert path ───

export interface SessionRestoreInput {
	tabs: { path: string; libraryName: string; libraryColor: string; pinned?: boolean; carried?: boolean }[];
	activeTabPath: string | null;
	splitActive: boolean;
	splitDir: SplitDirection;
}

/** MIG-100 §3 — paths restored WITHOUT a cid_cn. Gate #8 forbids the boot
 *  restore from writing user files, so the cid injection every manual open
 *  performs is DEFERRED: drained on the first USER activation of that tab,
 *  through the toggleTaskReconciled recipe (mark-cascading → flush-if-dirty →
 *  disk ensure → model ADOPTS). A skipped drain stays pending; the manual
 *  open path heals it eventually. */
const pendingCidEnsure = new Set<string>();
let cidEnsureUnsubs: Array<() => void> = [];

/** Test/diagnostic — count of tabs awaiting deferred cid-ensure. The
 *  activation watchers are alive iff this is > 0 (the close-audit invariant:
 *  the set empties → the watchers are torn down), so it's the observable for
 *  the leak fixes. */
export function pendingCidEnsureCount(): number {
	return pendingCidEnsure.size;
}

/** MIG-100 close-audit fix (drift/leak class): tear down the deferred-cid
 *  activation watchers + clear the pending set. Called when the set empties
 *  (drain), when a pending tab is CLOSED (else its path wedges the set and
 *  the watchers leak for the session), and on universe departure (the tabs
 *  are gone — their pending paths belong to the universe being left). */
function disposeDeferredCidEnsure(): void {
	for (const u of cidEnsureUnsubs) u();
	cidEnsureUnsubs = [];
	pendingCidEnsure.clear();
}

/** Drop a single path from the pending set (a tab closed before its deferred
 *  cid-ensure drained); tear the watchers down if that empties the set. */
function dropPendingCidEnsure(path: string): void {
	if (!pendingCidEnsure.delete(path)) return;
	if (pendingCidEnsure.size === 0) disposeDeferredCidEnsure();
}

function armDeferredCidEnsure(): void {
	if (cidEnsureUnsubs.length > 0) return;
	const onActivate = (tabId: string | null) => {
		if (!tabId) return;
		const tab = get(openTabs).find((t) => t.id === tabId);
		if (!tab || !pendingCidEnsure.has(tab.path)) return;
		pendingCidEnsure.delete(tab.path);
		void drainCidEnsure(tab);
		if (pendingCidEnsure.size === 0) disposeDeferredCidEnsure();
	};
	// Skip each subscription's synchronous first fire — that's the CURRENT
	// activation (set by the restore itself), not a user action.
	cidEnsureUnsubs = [
		subscribeSkipInitial(activeTabId, onActivate),
		subscribeSkipInitial(focusedTabId, onActivate),
	];
}

async function drainCidEnsure(tab: OpenTab): Promise<void> {
	// Same gate shape as toggleTaskReconciled: the disk op runs under the
	// cascading mark so the open note's armed autosave can't interleave.
	markCascading(tab.path);
	try {
		if (isNoteDirty(tab.id)) {
			markRecentWrite(tab.path);
			await saveNoteSession(tab.id, tab.path, standardSaveEnv({ origin: 'cid_ensure_flush', name: tab.name }), 'cid_ensure_flush');
		}
		// MIG-TPL §1 — never stamp a template (identity belongs to the cast, not the mold).
		if (!isTemplatePath(tab.path)) {
			await invoke('ensure_cid_cn_cmd', { filePath: tab.path });
			// ★ Inspection 2026-07-29 — the ensure WRITES the note's permanent identity through
			// the gate (watcher-suppressed), so nothing else tells the index. The sweep closed
			// this exact gap in openNoteTab and missed this sibling. Fire-and-forget: the tab
			// is already correct on screen; this only catches note_meta up.
			invoke('constellation_search_reindex', { notePath: tab.path, libraryName: tab.libraryName }).catch(() => {});
		}
		// Guarded adopt — deliberately NOT reloadTabsFromDisk: its
		// unconditional model re-seed would discard keystrokes typed during
		// the awaits above (markCascading blocks saves, not typing). The model
		// adopts the cid-bearing disk ONLY while it is still clean; if the
		// user typed, their model wins and the path re-pends for a later
		// clean activation.
		const diskContent = await readNote(tab.path).catch(() => null);
		if (diskContent === null) return;
		let adopted = false;
		openTabs.update((ts) =>
			ts.map((t) => {
				if (t.id !== tab.id || t.content === diskContent || isNoteDirty(t.id)) return t;
				adopted = true;
				openNoteModel(t.id, t.path, diskContent);
				return { ...t, content: diskContent, reloadVersion: (t.reloadVersion ?? 0) + 1 };
			})
		);
		if (adopted) {
			clearWriteAhead(tab.path);
		} else if (isNoteDirty(tab.id)) {
			// cid landed on disk but the dirty model will overwrite it on its
			// next save — re-pend so a later clean activation re-ensures.
			pendingCidEnsure.add(tab.path);
			armDeferredCidEnsure();
		}
	} catch { /* non-fatal: cid stays absent; the next manual open heals it */ }
	finally {
		clearCascading(tab.path);
	}
}

/**
 * MIG-100 §3 — restore a session snapshot's tabs in ONE batch.
 *
 * NOT a loop over openNoteTab: that would activate every tab in sequence
 * (alwaysFocusNewTabs) — N CM6 mount/teardown cycles, wab pollution, focus
 * churn — and would run ensure_cid_cn disk writes at boot (Gate #8). This
 * path reads each note (write-ahead-buffer-aware via resolveNoteContent),
 * builds all OpenTabs, commits them in ONE openTabs update (append — tabs
 * the user already opened stay untouched, one-path-one-tab preserved), and
 * activates at most ONE tab, only when the user hasn't navigated yet.
 *
 * `stillValid` is checked immediately before the commit — a universe switch
 * mid-restore aborts with zero store mutations.
 */
export async function restoreSessionTabs(
	snap: SessionRestoreInput,
	stillValid?: () => boolean,
): Promise<{
	restored: number;
	requested: number;
	aborted?: boolean;
	activatedId?: string;
	/** Requested tabs whose file was unreadable — the caller decides whether
	 *  to carry them forward one boot (transient failure) or drop them. */
	skipped: SessionRestoreInput['tabs'];
}> {
	const requested = snap.tabs.length;
	const built: OpenTab[] = [];
	const restoredTrueDisk = new Map<string, string>(); // PJ-102b — path → true disk bytes for wab-recovered tabs
	const seen = new Set<string>();
	const skipped: SessionRestoreInput['tabs'] = [];
	for (const saved of snap.tabs) {
		if (!saved.path || seen.has(saved.path)) continue;
		seen.add(saved.path);
		// preserveNet: the restore may show write-ahead-recovered content but
		// must never DESTROY the recovery copy (the model is born clean; only
		// a real durable save may replace the net). requireDisk: a wab entry
		// alone must not resurrect a tab whose file is gone (the ghost-tab
		// failure) — restore restores FILES, the net stays for real recovery.
		const resolved = await resolveNoteContent(saved.path, { preserveNet: true, requireDisk: true });
		if (resolved === null) {
			skipped.push(saved); // unreadable → skip, never abort the rest
			continue;
		}
		const { content, cursorPos, scrollTop } = resolved;
		// PJ-102b (restore half) — remember the TRUE disk bytes for wab-recovered tabs;
		// applied to the model after the one-commit seed below (clean + true baseline).
		if (resolved.recoveredFromNet && resolved.diskContent != null) {
			restoredTrueDisk.set(saved.path, resolved.diskContent);
		}
		if ((saved.path.endsWith('.md') || saved.path.endsWith('.markdown')) && !extractCidCn(content)) {
			pendingCidEnsure.add(saved.path); // deferred — no boot-time write
		}
		let name = saved.path.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';
		const fmTitleMatch = content.match(/^---[\s\S]*?^title:\s*"?([^"\n]+)"?\s*$/m);
		if (fmTitleMatch?.[1]) name = fmTitleMatch[1].trim();
		const { library, libraryPath } = deriveLibraryForPath(saved.path);
		built.push({
			id: `tab_${++tabCounter}_${Date.now()}`,
			path: saved.path,
			content,
			libraryName: library?.name ?? saved.libraryName,
			libraryPath,
			name,
			libraryColor: saved.libraryColor,
			history: [saved.path],
			historyIndex: 0,
			cursorPos,
			scrollTop,
			...(saved.pinned ? { pinned: true } : {}),
		});
	}
	if (stillValid && !stillValid()) return { restored: 0, requested, aborted: true, skipped };
	if (built.length === 0) return { restored: 0, requested, skipped };

	// ONE commit. Dedup against tabs the user opened DURING the reads
	// (one path → one tab, the B1 invariant), then models born with their
	// tabs, synchronously — same discipline as openNoteTab.
	let appended: OpenTab[] = [];
	openTabs.update((ts) => {
		const have = new Set(ts.map((t) => t.path));
		appended = built.filter((b) => !have.has(b.path));
		return appended.length ? [...ts, ...appended] : ts;
	});
	for (const t of appended) {
		openNoteModel(t.id, t.path, t.content);
		// PJ-102b (restore half) — a wab-recovered restore tab stays BORN-CLEAN (Gate #8:
		// a restore performs zero write-class IPCs) but gets the TRUE disk baseline, so a
		// phantom watcher event (disk === baseline) is REFUSED by adoptDisk instead of
		// "adopting" stale disk and destroying the preserved net (the Q4 hole: store.ts
		// clearWriteAhead-on-adopt). A genuinely-changed disk still adopts (clean model).
		const trueDisk = restoredTrueDisk.get(t.path);
		if (trueDisk !== undefined) setModelDiskBaseline(t.id, trueDisk);
	}
	editingTabIds.update((set) => {
		const next = new Set(set);
		for (const t of appended) next.add(t.id);
		return next;
	});

	// ONE activation — only when the user hasn't opened/focused anything yet
	// (restore must never steal focus from a live user).
	let activatedId: string | undefined;
	if (appended.length > 0 && get(activeTabId) === null && get(focusedTabId) === null) {
		const match =
			(snap.activeTabPath && appended.find((t) => t.path === snap.activeTabPath)) || appended[0];
		splitActive.set(snap.splitActive);
		splitDirection.set(snap.splitDir);
		activeTabId.set(match.id);
		if (snap.splitActive) focusedTabId.set(match.id);
		activatedId = match.id;
	}
	if (pendingCidEnsure.size > 0) armDeferredCidEnsure();
	return { restored: appended.length, requested, activatedId, skipped };
}

export async function closeTab(tabId: string) {
	// APP-KILLER #2 — a tab CLOSE is a DEPARTURE that DISPOSES the model (closeNoteModel
	// below). Flush its dirty edits to disk FIRST through the durability gate, or they
	// vanish when the model is deleted (the teardown flush can't help — the model is
	// already gone). Unlike a nav we do NOT abort on a failed flush (the tab is being
	// dismissed): the write-ahead net + save-health banner preserve a failed write and
	// restore it on reopen. Re-read openTabs AFTER the await so a concurrent tab op is
	// respected. (Gap in the APP-KILLER #2 nav-flush — closeTab is the third departure.)
	if (NAV_FLUSH_ENABLED) await flushOutgoing(tabId, 'close_flush');

	const tabs = get(openTabs);
	const idx = tabs.findIndex(t => t.id === tabId);
	if (idx === -1) return;

	const currentActive = get(activeTabId);
	const newTabs = tabs.filter(t => t.id !== tabId);

	// Clean up non-reactive state first (no cascade)
	saveLocks.delete(tabId);
	// Close-audit fix: a restored tab closed before its deferred cid-ensure
	// drained would wedge `pendingCidEnsure` (never empties → the activation
	// watchers leak for the session). Drop its path here.
	if (idx >= 0) dropPendingCidEnsure(tabs[idx].path);
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
	// PJ-065 §8 (cold-start) — add_library only REGISTERS the folder; its pre-existing
	// .md files are not in the index yet (no boot walk; the watcher only catches live
	// edits). Index them in the background so search, backlinks AND the structural
	// spine work immediately. Fire-and-forget; refresh stats when it finishes.
	invoke('reindex_library', { libraryPath: library.path, libraryName: library.name, onlyIfUnindexed: false })
		.then(() => loadAllStats())
		.catch((e) => console.error('[addLibrary] background reindex failed:', e));
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
	// PJ-065 — this list is COGNITIVE-ONLY by design. The structural (parent/TOC)
	// lane (parent / contains) must NEVER be added here: a TOC relationship is not a
	// search operator, and `parent [[X]]` must stay plain free text, not a typed_links
	// filter. (Structural search lives in the TOC panel, not the query grammar.)
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

// ─── File operations ───
export async function createNote(folderPath: string, fileName: string, initialFrontmatter?: string, initialBody?: string): Promise<string> {
	const newPath: string = await invoke('create_note', { folderPath, fileName, initialFrontmatter: initialFrontmatter ?? null, initialBody: initialBody ?? null });
	// F2′ — gated creates are watcher-suppressed (write_gate marks the path),
	// so the file tree never hears about them; announce the birth explicitly.
	// A Tauri emit reaches the main window from any window's JS context.
	emit('note-created', { path: newPath }).catch(() => {});
	return newPath;
}

// MIG-091 — searchByProperty removed with the retired Notes Navigator (its
// sole caller). Property search lives in Search Hub's `properties` category.

/** Build default frontmatter YAML for new notes (auto-dates + user-defined defaults) */
/**
 * MIG-TPL §1 — is this path inside the user's templates folder?
 *
 * Boss ruling 2026-07-19: **a template never carries `cid_cn` or a creation date.** A template is
 * a MOLD; identity and birth belong to the CAST. Without this guard the rule would be broken by
 * the app itself: simply OPENING a template to edit it runs `ensure_cid_cn_cmd`, which injects a
 * `cid_cn:` into the file — so every mold would acquire an identity on first edit, and every note
 * cast from it would inherit the mold's identity line.
 *
 * Compared case-insensitively on normalized separators, since the setting may be relative
 * ("Templates") or an absolute path from the folder picker.
 */
export function isTemplatePath(filePath: string): boolean {
	// TRIM FIRST, then fall back — mirroring `resolve_templates_dir` (universe.rs) exactly. It
	// trims and treats the result as empty, so a whitespace-only setting resolves to "Templates"
	// there. Falling back differently here would make the app list a folder whose files it then
	// stamps with an identity. (Caught by tests/mig-tpl/isTemplatePath.test.ts.)
	const folder = ((get(appSettings)?.templateFolder ?? '').trim()) || 'Templates';
	const norm = (p: string) => p.replace(/\\/g, '/').toLowerCase();
	const f = norm(filePath);
	const needle = norm(folder).replace(/\/+$/, '');
	// Absolute setting: a straight prefix test. Relative: match the folder as a PATH SEGMENT so
	// "Templates" does not also match "MyTemplatesArchive".
	if (/^([a-z]:)?\//.test(needle)) return f.startsWith(needle + '/');
	return f.includes('/' + needle + '/');
}

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
	// Note-open-freeze Batch-2 §B2-4 (2026-07-03) — flush-before-rename. The
	// (now async + dual-locked) rename reads DISK; a dirty open tab's last
	// ≤1.5 s of typing lives only in the editor model, and the ★Stage-1 tab
	// re-seed below replaces the tab with fresh disk content — so unsaved
	// keystrokes were dropped (pre-existing) and, once async, could race the
	// rename. Cure (the toggleTaskReconciled recipe): markCascading gates the
	// armed autosave, flush the model to disk, THEN rename — the rename's
	// locked read sees the user's latest keystrokes. Applied HERE in the one
	// wrapper both callers share (main-window handleRenameComplete + the
	// second-screen title fallback).
	const renamedTab = get(openTabs).find((t) => t.path === oldPath);
	if (renamedTab) markCascading(renamedTab.path);
	let effectivePath: string;
	// APP-KILLER #2 — track whether the pre-rename flush was DURABLE (see the guard below).
	let renameFlushOk = true;
	try {
		if (renamedTab && isNoteDirty(renamedTab.id)) {
			markRecentWrite(renamedTab.path);
			const rf = await saveNoteSession(renamedTab.id, renamedTab.path, standardSaveEnv({ origin: 'rename_flush', name: renamedTab.name }), 'rename_flush');
			renameFlushOk = rf.ok; // a FAILED write (locked .md) must NOT lose the dirty edit below
		}
		// Rust returns the EFFECTIVE path. For canonical notes, rename updates
		// the frontmatter title in-place and the file stays at oldPath — so the
		// returned path equals oldPath even though we requested newPath. Trusting
		// the requested newPath would point the tab at a non-existent file and
		// the next write_note call would create a phantom duplicate (BUG-001).
		effectivePath = await invoke<string>('rename_item', { oldPath, newPath });
	} finally {
		if (renamedTab) clearCascading(renamedTab.path);
	}
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
	// APP-KILLER #2 — only wipe the recovery net + re-seed the model from disk when the
	// pre-rename flush was DURABLE. If it FAILED (locked .md), the dirty model + its net
	// (migrated to effectivePath above) are the ONLY copy of the user's unsaved edits: keep
	// BOTH — no clearWriteAhead, and fresh stays null so the tab re-seed below repaths the
	// model (identity follows the rename) WITHOUT openNoteModel replacing it from stale disk.
	// The save-health auto-retry persists it once the lock clears.
	let fresh: string | null = null;
	if (renameFlushOk) {
		try {
			fresh = await readNote(effectivePath);
		} catch { /* folder rename / unreadable target — path/name update only */ }
	}

	// ★ PJ-174 #1b (safety inspection, 2026-07-28) — the net is cleared ONLY for a tab that
	// actually adopts fresh disk, and that decision is made below, AFTER every await. It used to
	// be cleared here, before `readNote`, purely on `renameFlushOk` — so a keystroke typed during
	// the rename lost its recovery net a moment before the model that held it was overwritten.
	// Same rule as `drainCidEnsure` and `reloadTabsFromDisk`: only adopters clear their net.
	let adoptedFreshDisk = false;

	openTabs.update(tabs => tabs.map(t => {
		if (t.path === oldPath) {
			// MIG-076 §C — the model's identity must follow the rename, or the
			// next save's compose would refuse the new path. A rename rewrites
			// frontmatter (title), so re-seed from fresh disk content when we
			// have it; otherwise just move the path.
			//
			// ★ PJ-174 #1b — …but ONLY when the model is still CLEAN. `markCascading` (set at the
			// top of this function) gates disk WRITES — `handleSave` / `handleFlush` — it does NOT
			// gate `onDocChange → editBody`, so the editor keeps accepting keystrokes into the
			// model throughout `invoke('rename_item')` and `readNote()`. And the freeze overlay
			// cannot cover this window either: it is raised by the caller only AFTER this function
			// returns. "Rename the title, press Enter, keep writing" is the natural gesture, and
			// the caret is already back in the body — so an unconditional re-seed here silently
			// destroyed whatever was typed, from the model, the screen AND (above) the net.
			//
			// The correct behaviour was already in this function as the flush-failed branch: keep
			// the user's model and just move its identity. Its sibling `drainCidEnsure` (:2841)
			// carries the identical guard with the identical reasoning; this site never got it.
			const keepUserModel = fresh === null || isNoteDirty(t.id);
			if (!keepUserModel) {
				openNoteModel(t.id, effectivePath, fresh!);
				adoptedFreshDisk = true;
			} else if (fresh !== null) {
				// ★ PJ-174 #1d (safety inspection, 2026-07-28) — APP-KILLER, introduced by #1b itself.
				//
				// #1b correctly stopped destroying the user's typing, but it kept the model by calling
				// `repathNoteModel` ALONE — and `setPath` moves `m.path` and nothing else. The model's
				// G4 write-base stayed at the PRE-rename bytes, while `rename_item` had just rewritten
				// `title:` on disk and appended the old title to `aliases`. compose then diffs
				// base.props against props, sees ZERO difference, and re-emits the OLD frontmatter
				// verbatim — so the next debounced save silently reverted the title, deleted the alias,
				// and left every wikilink the cascade had just rewritten pointing at a title that
				// existed nowhere. For a canonical note (title lives only in frontmatter) that undoes
				// the whole rename. Nothing surfaced it: the write is watcher-suppressed, and markSaved
				// + diskBaseline then made the stale model permanently self-consistent.
				//
				// Keep the user's BODY — the point of #1b — but RE-BASE the frontmatter to the renamed
				// file, which is what the clean branch above gets for free from `openNoteModel`.
				const dirtyBody = modelBodyForView(t.id);
				replaceContentInModel(t.id, fresh, t.path); // props + base ← the post-rename disk
				repathNoteModel(t.id, effectivePath);
				editNoteBody(t.id, dirtyBody, effectivePath); // …and the user's unsaved body back on top
			} else {
				// Flush FAILED: there is no trustworthy disk to re-base from, and the dirty model plus
				// its retained net are the only copy of the user's work. Move the identity and leave
				// everything else alone; save-health retries the write.
				repathNoteModel(t.id, effectivePath);
			}
			// Path comes from Rust (may equal oldPath for canonical files).
			// Display name follows the user's intent — for canonical files
			// the title changed even though the filename didn't.
			return {
				...t,
				path: effectivePath,
				name: derivedName || t.name,
				...(!keepUserModel
					? { content: fresh!, reloadVersion: (t.reloadVersion ?? 0) + 1 }
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
	// PJ-174 #1b — the net goes ONLY when its content was superseded by disk we actually adopted.
	// A tab that kept the user's model keeps its net too: that buffer is now part of the only copy
	// of unsaved work, exactly as in `adoptExternalChangeIntoTabs` (invariant #10).
	if (adoptedFreshDisk) {
		clearWriteAhead(oldPath);
		clearWriteAhead(effectivePath);
	}
	// APP-KILLER #2 — on a FAILED pre-rename flush, re-point the save-health failure from the
	// old path to the renamed path so the banner's Retry + the ~10 s auto-retry target the
	// note's CURRENT (tab-owned) path (retrySaveFailure looks the tab up BY path). For a
	// canonical rename (effectivePath === oldPath) this is a harmless no-op.
	if (!renameFlushOk) {
		const stale = get(saveHealth).get(oldPath);
		if (stale) { clearSaveFailure(oldPath); reportSaveFailure(effectivePath, stale.name, stale.error); }
	}
	return effectivePath;
}

export async function moveItem(sourcePath: string, targetFolder: string): Promise<string> {
	// PJ-187 — the same flush-before-cascade envelope `renameItem` has carried since Batch-2
	// §B2-4, which this sibling never got. `move_item` is async and reads DISK; a dirty open
	// tab's last ≤1.5 s of typing lives only in the model, and the armed autosave was still
	// free to fire mid-move — writing the newest words to the note's OLD path, which the move
	// then leaves behind. The result on disk is two notes: the moved one, and a stray copy in
	// the old folder carrying the user's latest text, with nothing to say which is real.
	// markCascading gates the armed autosave; the explicit flush puts the newest bytes on disk
	// BEFORE the move reads them; the finally releases the gate on every exit path.
	const movedTab = get(openTabs).find((t) => t.path === sourcePath);
	if (movedTab) markCascading(movedTab.path);
	let newPath: string;
	try {
		if (movedTab && isNoteDirty(movedTab.id)) {
			markRecentWrite(movedTab.path);
			await saveNoteSession(movedTab.id, movedTab.path, standardSaveEnv({ origin: 'move_flush', name: movedTab.name }), 'move_flush');
		}
		newPath = await invoke<string>('move_item', { sourcePath, targetFolder });
	} finally {
		if (movedTab) clearCascading(movedTab.path);
	}
	// §140: same path-keyed migration as renameItem — buffer follows file.
	migratePathKeyedAuxStateOnRename(sourcePath, newPath);
	// Update any open tabs that reference the old path
	openTabs.update(tabs => tabs.map(t => {
		if (t.path === sourcePath) {
			const newName = newPath.split(/[\\/]/).pop()?.replace('.md', '') ?? t.name;
			repathNoteModel(t.id, newPath); // MIG-076 §C — model identity follows the move
			return { ...t, path: newPath, name: newName };
		}
		// If a folder was moved, update paths under it.
		//
		// APP-KILLER FIX (2026-07-18): this derives from `newPath` — the destination Rust
		// actually created — NOT from `targetFolder`. `relative` is sliced at
		// `sourcePath.length`, i.e. AFTER the moved folder's own name, so pairing it with the
		// parent directory dropped that name from every descendant: moving `A/Sub` into `B`
		// repointed an open `A/Sub/Ideas.md` at `B/Ideas.md`. Both the tab and the model were
		// set to the same wrong string, so `compose`'s path-identity guard could not fire, and
		// every later save created a phantom note beside the real one — or overwrote an
		// unrelated note of the same name, since WRITE_GATE_ENFORCE is false.
		//
		// `newPath` is also the only correct source when Rust CHOSE a different final name than
		// requested (a collision suffix): `targetFolder` cannot know about that, the returned
		// path can. This now matches `renameItem`'s equivalent branch above, which has always
		// used the returned path (`effectivePath + relative`) — that asymmetry was the bug.
		if (t.path.startsWith(sourcePath + '/') || t.path.startsWith(sourcePath + '\\')) {
			const relative = t.path.substring(sourcePath.length);
			const movedPath = newPath + relative;
			repathNoteModel(t.id, movedPath); // MIG-076 §C
			return { ...t, path: movedPath };
		}
		return t;
	}));
	return newPath;
}

// `deleteItem` wrapper RETIRED with its `delete_item` command (Batch-2 §B2-4,
// Boss-ruled 2026-07-03) — zero component callers; `deleteWithSetting` /
// `deletePath` are the live delete surface (MIG-076 §E-follow-up).

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
/**
 * PJ-187 — the ONE answer to *"where does a note go when Constellation removes it from its
 * place?"*, honouring **Settings → Universe & Libraries → Deleted files**.
 *
 * It was the private opening of `deleteWithSetting`, so DELETE honoured the setting and the three
 * other displacement paths — Overwrite on create, Overwrite on rename, and the PJ-088 conflict
 * sidecar — did not: they called `moveToTrash(path, libraryPath)`, which hardcoded
 * `<library>/.trash` and never read the setting at all.
 *
 * Measured on the Boss's universe (2026-07-29, Stage 1): with `trashDestination: 'local'` and
 * `trashFolderScope: 'universe'`, Delete filed to the universe root while Overwrite filed to the
 * library — and because that universe's libraries live OUTSIDE its root, those are different
 * TREES, not neighbouring folders. The note stayed recoverable, but not where the app says it
 * puts things, which is the same thing as lost to anyone who goes looking. With the DEFAULT
 * `trashDestination: 'system'` it is worse: Delete uses the Recycle Bin while Overwrite silently
 * creates a `.trash` folder inside the library the user never opted into.
 *
 * Exported so every displacement path shares one implementation and they cannot drift again.
 */
export function resolveTrashDestination(path: string): { mode: 'trash' | 'system'; trashRoot: string | null } {
	const s = get(appSettings);
	// 'permanent' is no longer a user choice (Boss 2026-06-14 — deletes are always
	// recoverable); anything not 'local' resolves to System trash.
	if (s.trashDestination !== 'local') return { mode: 'system', trashRoot: null };
	let trashRoot: string | null;
	if (s.trashFolderScope === 'universe') {
		trashRoot = get(libraries).find(v => v.is_universe_notes)?.path ?? null;
	} else {
		const matches = get(libraryStats).filter(v => path.startsWith(v.path));
		trashRoot = matches.length
			? matches.reduce((a, b) => (b.path.length > a.path.length ? b : a)).path
			: null;
	}
	if (!trashRoot) throw new Error('Could not resolve a .trash location for this path.');
	return { mode: 'trash', trashRoot };
}

export async function deleteWithSetting(path: string): Promise<void> {
	const { mode, trashRoot } = resolveTrashDestination(path);
	await deletePath(path, mode, trashRoot);
	// §140 — drop the path's aux state + close any tabs at/under it (as deleteItem did).
	clearPathKeyedAuxStateOnDelete(path);
	// PJ-187 — dropping a tab from the list is NOT disposing its model. Every other departure
	// (closeTab, the universe-switch sweep) calls closeNoteModel; this one filtered the tabs and
	// left each deleted note's full body, base and props resident for the rest of the session.
	// Collect what the filter drops, then dispose it.
	const removed: OpenTab[] = [];
	openTabs.update(tabs => tabs.filter(t => {
		const gone = t.path === path || t.path.startsWith(path + '/') || t.path.startsWith(path + '\\');
		if (gone) removed.push(t);
		return !gone;
	}));
	for (const t of removed) closeNoteModel(t.id);
}

/**
 * MIG-076 §E1b — displace an existing note to the trash (recoverable), used by the collision
 * dialog's "Overwrite" before the create/rename proceeds, and by the PJ-088 conflict sidecar.
 *
 * PJ-187 — now routed through `resolveTrashDestination` + `deletePath`, the SAME pair Delete
 * uses, so a displaced note lands exactly where the user's "Deleted files" setting says it will.
 * It previously invoked `move_to_trash`, which derives its trash root from the library path it
 * also validates against — so it can never honour universe scope for a library that lives outside
 * the universe root, which is the ordinary case here. The Rust command keeps its own caller
 * (`universe.rs` Template-Studio undo) and is untouched.
 */
export async function moveToTrash(path: string): Promise<void> {
	const { mode, trashRoot } = resolveTrashDestination(path);
	await deletePath(path, mode, trashRoot);
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

// MIG-099 §6 — purpose-built create/rename TITLE-collision check (MIG-076 §E1b).
// Index-only for own libraries (no filename read_dir) → sub-10 ms vs the full
// resolver. Same ResolvedLink shape so the collision dialog + Overwrite are
// unchanged. Use ONLY for the "does this title already exist?" guard — to
// open/navigate a [[wikilink]] use resolveWikilinkCrossLibrary (that keeps the
// filename-stem stage link resolution needs).
export async function resolveTitleCollision(currentLibraryPath: string, target: string): Promise<ResolvedLink | null> {
	const libraryList = get(libraries).map(v => [v.id, v.name, v.path] as [string, string, string]);
	return await invoke('resolve_title_collision', { libraries: libraryList, currentLibraryPath, target });
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
	/** PJ-114 §3 — ISO-8601 timestamp the link row was created, or '' for legacy
	 *  rows written before the column was populated. Surfaced by the per-link
	 *  inspector (§6), which renders an honest "unknown" rather than a guessed
	 *  date. Empty for entries not sourced from `note_links`. */
	created?: string;
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
	// MIG-085 §B.0 — fold the match key the SAME way the Rust side does
	// (fold_match_key = NFC → lower → NFC), so this panel's backlink count
	// matches incoming_count / the badge / the Reviewer even for accented
	// titles ("Île-de-France") and NFD (macOS-origin) names.
	const foldKey = (s: string) => s.normalize('NFC').toLowerCase().normalize('NFC');
	const target = foldKey(noteName);
	const targets = new Set<string>([target]);
	if (noteAliases) {
		for (const a of noteAliases) {
			if (a) targets.add(foldKey(a));
		}
	}
	const linked = allLinks.filter(l => targets.has(foldKey(l.target)) && l.status !== 'archived');
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
			/** PJ-114 §3 — ISO-8601 creation stamp ('' on legacy rows). Carried through for
			 *  the per-link inspector (§6); this map REBUILDS rows, so an unmapped field is
			 *  silently dropped. */
			created: l.created ?? '',
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
			/** PJ-114 §3 — ISO-8601 creation stamp ('' on legacy rows). Carried through for
			 *  the per-link inspector (§6); this map REBUILDS rows, so an unmapped field is
			 *  silently dropped. */
			created: l.created ?? '',
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

// ─── MIG-086 — related-note suggestions (BM25 "More Like This") ───

/** One related-but-UNLINKED note suggested for an orphan/fragile note.
 *  `shared_terms` is the *why* — the distinctive terms it shares with the
 *  source (rendered as chips); never empty. From `suggest_related_notes`. */
export interface RelatedCandidate {
	note_path: string;
	note_name: string;
	score: number;          // |bm25| — higher = more related (optional UI bar)
	shared_terms: string[]; // the legible reason they relate
	snippet: string;        // short body preview (may be empty)
}

/** Suggest related-but-unlinked notes for the given note (orphan/fragile triage →
 *  action). Query-time over the live FTS index — call ON DEMAND (panel/detail open,
 *  note change), NEVER per-keystroke. Returns `[]` when nothing clears the bar. */
export async function suggestRelatedNotes(
	libraryPath: string,
	notePath: string,
	limit?: number
): Promise<RelatedCandidate[]> {
	return await invoke('suggest_related_notes', {
		libraryPath,
		notePath,
		limit: limit ?? null,
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

// MIG-091 — NoteWithMeta + collectLibraryNotesWithMeta removed with the retired
// Notes Navigator (their sole consumer). The empowered File Explorer reads the
// tree via read_library_tree; note lists elsewhere read the write-time index.

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

export async function updateLinksOnRename(libraryPath: string, libraryName: string, oldName: string, newName: string, excludePaths: string[] = []): Promise<CascadeResult> {
	// PJ-092 — `excludePaths` = open notes whose flush was NOT durable; the Rust walker
	// must NEVER rewrite them (rewriting a note we couldn't flush diverges disk from the
	// dirty model → data-loss / freeze). Pre-normalize to forward slashes as a belt; Rust
	// re-keys by full FILE IDENTITY (canonicalize + NFC), the primary guard.
	return await invoke('update_links_on_rename', { libraryPath, libraryName, oldName, newName, excludePaths: excludePaths.map(normPath) });
}

// PJ-065 §D9 — one-click resolution of a contested structural parent. Edits ONE frontmatter
// field on `notePath` (field 'parent' → set `parent: [[targetName]]`; field 'contains' → remove
// `[[targetName]]` from `contains:`) via the gated RMW path; the backend emits
// cascade:rewrote so any open tab reloads from disk.
// Note-open-freeze Batch-2 §B2-3 (2026-07-03): the command is `(async)` + gate_rmw now; this
// wrapper gains the proven toggleTaskReconciled recipe — if the note is open AND dirty, FLUSH
// its model to disk first (so the resolve reads the latest keystrokes, which no lock can
// recover from editor memory), gate the window with markCascading (so the armed autosave can't
// revert the resolve), then reload the model from disk.
export async function resolveStructuralConflict(notePath: string, field: 'parent' | 'contains', targetName: string): Promise<void> {
	const openTab = get(openTabs).find((t) => t.path === notePath);
	if (openTab) markCascading(openTab.path);
	try {
		// PJ-092 H4 — bounded flush; if not durable, DON'T rewrite+reload (clobber/hang guard); save-health retries.
		if (openTab && !(await flushOpenTabOrAbort(openTab, 'structural_resolve_flush'))) return;
		await invoke('resolve_structural_conflict', { notePath, field, targetName });
		await reloadTabsFromDisk([notePath]); // model ADOPTS the resolved disk + {#key} remount
	} finally {
		if (openTab) clearCascading(openTab.path);
	}
}

/**
 * PJ-088 — resolve a conflict by SAVING a user-reconciled MERGE safely. The note is OPEN under
 * Single-Ownership, so the merged text is pushed INTO the one model (editProps+editBody) and saved
 * through the durability gate — NEVER a raw write over the open note (a raw write would reintroduce
 * the PJ-070 Recipe-O clobber: the still-dirty model's next autosave composes the stale body over
 * the merge). The `.conflict` sidecar is moved to trash (reversible) + the banner dismissed ONLY
 * after a proven durable save; on failure nothing irreversible runs (zero-loss, retryable). Returns
 * {ok} so the caller closes the overlay + trashes only on success. Cancel never calls this — the
 * merged text lives in the merge view and reaches the model only here, at Save.
 */
export async function resolveConflictMerge(
	notePath: string,
	sidecarPath: string,
	mergedText: string,
	hooks?: { focusReseed?: (path: string) => void },
): Promise<{ ok: boolean; reason?: string }> {
	const tab = get(openTabs).find((t) => t.path === notePath);
	if (!tab) return { ok: false, reason: 'note_not_open' }; // the entry point opens the note first — a model must exist

	markCascading(notePath); // gate the armed autosave + the outgoing {#key} teardown flush for the whole op
	markReseeding(notePath); // hazard #6 — span the async remount so the outgoing (stale) editor can't re-stale the merge
	try {
		// Push the merged content INTO the one model, path-guarded + RE-BASED (replaceContent), so
		// compose emits the merged frontmatter verbatim — not a G4 diff against the stale open-time base
		// (which would silently drop non-projectable frontmatter, e.g. nested maps, the merge changed).
		// The model IS now the merge — no stale second copy remains to overwrite it (the core defense).
		replaceContentInModel(tab.id, mergedText, notePath);
		markRecentWrite(notePath); // suppress the watcher echo of our own write (no phantom sidecar)
		const outcome = await saveNoteSession(
			tab.id,
			notePath,
			standardSaveEnv({
				origin: 'merge_resolve',
				name: tab.name,
				onSaved: (savedPath) => {
					emit('screen:note-saved', { path: savedPath }).catch(() => {}); // second screen refresh
					reindexNote(savedPath, tab.libraryName).catch(() => {}); // search/backlinks reflect the merge
				},
			}),
			'merge_resolve',
		);
		// Durability gate: on failure the model stays DIRTY, the net is RETAINED, the save-health banner
		// surfaces it (onError). Nothing irreversible has run — the sidecar + conflict banner remain.
		if (!outcome.ok) return { ok: false, reason: (outcome as { reason?: string }).reason ?? 'write_failed' };

		// Durable success → remount the editor surfaces on the merged disk, then resolve the sidecar.
		await reloadTabsFromDisk([notePath]); // NotePane: force-reseed clean model + reloadVersion {#key} remount
		hooks?.focusReseed?.(notePath); // FocusPane is NOT under the {#key} — reseed it too (Editor-Surface Gate #2/#4)
		await tick(); // let the remounts + the gated outgoing teardown flush settle before we release the gate

		// Sidecar → trash (reversible, never hard-delete), banner row → dismiss. ONLY after durable success.
		const libNorm = normPath(sidecarPath);
		// MIG-105 Stage-0 C7 (PJ-156 E2): `+ '/'` boundary guard — without it a root like
		// ".../Research" matched a ".../Research Notes" sidecar and trashed it via the
		// WRONG library root. No-match now leaves the sidecar on disk (the banner is still
		// dismissed) — strictly safer than trashing into another library's trash.
		const lib = get(libraryStats)
			.filter((l) => libNorm === normPath(l.path) || libNorm.startsWith(normPath(l.path) + '/'))
			.sort((a, b) => normPath(b.path).length - normPath(a.path).length)[0]; // longest-prefix = most-specific (nested/federated)
		if (lib) {
			try { await moveToTrash(sidecarPath); } catch (e) { console.error('[PJ-088] conflict sidecar trash failed', e); }
		}
		dismissConflict(sidecarPath);
		return { ok: true };
	} finally {
		clearReseeding(notePath);
		clearCascading(notePath);
	}
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
	/** PJ-114 — FM+ (Focus Mode Plus): opt-in Focus right-click menu + in-Focus link
	 *  navigation. Additionally gated by the FM_PLUS_ENABLED build flag. Default off;
	 *  Focus stays byte-for-byte itself when off. */
	focusModePlus: boolean;
	foldIndent: boolean;
	indentationGuides: boolean;
	alwaysFocusNewTabs: boolean;
	/** MIG-100 — reopen the last session's tabs at launch (the auto-session,
	 *  `.constellation/session.json`). OFF also deletes the stored session —
	 *  off means "stop remembering". */
	restoreTabsOnRelaunch: boolean;
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
	/** MIG-089 — per-Universe user-defined callout types ([!slug] → colour + icon).
	 *  See src/lib/theme/customCallouts.ts (CustomCallout). Inline type here to keep
	 *  store.ts free of a circular import. */
	customCallouts?: { slug: string; name: string; color: string; icon: string }[];
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

	// Living Link pill SHAPE (radius/height/weight), shared by every LinkTypePill.
	// Per-type COLOURS moved to the Link-Type Registry (MIG-067 — linkTypeRegistry.ts,
	// user-edited via Style Setter → Links); the old fill/text maps here had no readers.
	linkPills: {
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

	/** MIG-083 §D — Review Pulse (the resurfacing/staleness queue) settings. */
	review: {
		/** Staleness grace period, in DAYS (minimum 1). A note is flagged "stale"
		 *  only when a load-bearing dependency's content changed at least this many
		 *  days AFTER the note's last explicit review. 1 = the next day onward
		 *  (most patient minimum); higher = more patient. Boss 2026-06-22. */
		staleGraceDays: number;
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

	/** PJ-068 v2 — which note-graph "lens" the second screen renders for the open note:
	 *  'butterfly' (facing blooms, default) · 'ledger' (balance sheet) · 'heartwood' · 'orrery'.
	 *  The retired 'aster' normalizes to 'butterfly' on read (normalizeGraphStyle in cockpitFlag).
	 *  Chosen in Settings; coloured via the Style Setter (--rel-* vars). */
	noteGraphStyle: 'butterfly' | 'ledger' | 'heartwood' | 'orrery';

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
	focusModePlus: false, // PJ-114 — FM+ opt-in, default off
	foldIndent: true,
	indentationGuides: false,
	alwaysFocusNewTabs: true,
	restoreTabsOnRelaunch: true,
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
	customCallouts: [],
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
	review: {
		staleGraceDays: 1,
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
	noteGraphStyle: 'butterfly',
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
	// PJ-065 note: this currently returns ALL types (incl. the structural parent/TOC
	// lane). It has no live caller. If revived for a COGNITIVE surface, switch to
	// cognitiveLinkTypes(); if revived for the Style Setter colour controls, all
	// types (incl. structural) are intentionally correct (its teal is refinable there).
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
		review: { ...DEFAULT_SETTINGS.review, ...((parsed.review as Record<string, unknown>) || {}) },
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

/** MIG-088 §3b-pre — clear MANY per-Universe overrides at once (the Style Setter's PER-ELEMENT
 *  "Reset this element"): one settings update + one save + one emit. Reverts exactly the named
 *  vars to the theme/fallback look, leaving every other element's overrides untouched. */
export function clearStyleOverrideKeys(keys: string[]) {
	appSettings.update(s => {
		const cur = s.styleOverride ?? {};
		if (!keys.some(k => k in cur)) return s;
		const next = { ...cur };
		for (const k of keys) delete next[k];
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
	// MIG-100 §5 — the old shape here had two latent defects the Architect
	// census documented: (1) `openTabs.set([])` discarded possibly-dirty
	// models with NO flush (the nav-loss class v3.34 closed everywhere else);
	// (2) the sequential openNoteTab loop with `newTab` undefined collapsed a
	// multi-tab workspace through the in-place-replace branch. Now: the ONE
	// departure primitive, then the shared batch-insert.
	await flushDisposeClearTabs('workspace_restore_flush');

	// Batch-insert (one store commit, one activation — restoreSessionTabs
	// activates ws.activeTabPath or the first restored tab, and applies the
	// split state from the snapshot).
	const result = await restoreSessionTabs({
		tabs: ws.tabs.map((t) => ({
			path: t.path,
			libraryName: t.libraryName,
			libraryColor: t.libraryColor,
		})),
		activeTabPath: ws.activeTabPath,
		splitActive: ws.splitActive,
		splitDir: ws.splitDir,
	});

	// The old loop applied focusedTabId unconditionally; keep that behavior
	// for manual restores (restoreSessionTabs sets it only in split mode).
	if (result.activatedId) focusedTabId.set(result.activatedId);

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

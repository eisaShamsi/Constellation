/**
 * Library (Universe) state management.
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';

export interface LibraryInfo {
	id: string;
	name: string;
	path: string;
	is_universe_notes?: boolean;
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
}

export type PropertyType = 'text' | 'number' | 'date' | 'datetime' | 'list' | 'link' | 'checkbox';

export interface FrontmatterProperty {
	key: string;
	value: string;
	type: PropertyType;
	listItems?: string[];
}

// Well-known property key sets (English + Arabic)
const LIST_KEYS = new Set([
	'tags', 'aliases', 'cssclasses', 'cssclass', 'related', 'categories', 'group',
	'الوسم', 'وسوم', 'المجموعة', 'ذات صلة', 'أسماء بديلة', 'تصنيفات',
]);
const CHECKBOX_KEYS = new Set([
	'done', 'completed', 'draft', 'publish', 'published', 'pinned', 'archived', 'starred', 'todo',
	'favorite', 'featured', 'hidden',
	'مكتمل', 'منشور', 'مسودة', 'مثبت', 'مؤرشف', 'مميز', 'مخفي',
]);
const DATE_KEYS = new Set([
	'date', 'created', 'updated', 'modified', 'due', 'start', 'end', 'deadline', 'completed_date',
	'أنشئ', 'حُدث', 'تاريخ', 'تعديل', 'موعد', 'بداية', 'نهاية',
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

export function setWriteAhead(filePath: string, content: string, cursorPos: number, scrollTop: number) {
	const entry = { content, cursorPos, scrollTop };
	writeAheadBuffer.set(filePath, entry);
	/* Also persist to localStorage as crash-safe backup (survives app restart).
	   This is synchronous and fast for single-note content. */
	try {
		const key = 'constellation-wab';
		const existing = JSON.parse(localStorage.getItem(key) || '{}');
		existing[filePath] = entry;
		localStorage.setItem(key, JSON.stringify(existing));
	} catch {}
}

export function getWriteAhead(filePath: string): { content: string; cursorPos: number; scrollTop: number } | undefined {
	/* Check in-memory buffer first (faster), fall back to localStorage */
	const mem = writeAheadBuffer.get(filePath);
	if (mem) return mem;
	try {
		const key = 'constellation-wab';
		const all = JSON.parse(localStorage.getItem(key) || '{}');
		return all[filePath];
	} catch {}
	return undefined;
}

export function clearWriteAhead(filePath: string) {
	writeAheadBuffer.delete(filePath);
	try {
		const key = 'constellation-wab';
		const all = JSON.parse(localStorage.getItem(key) || '{}');
		delete all[filePath];
		localStorage.setItem(key, JSON.stringify(all));
	} catch {}
}

export async function saveTabContent(
	tabId: string,
	filePath: string,
	properties: FrontmatterProperty[],
	body: string
): Promise<void> {
	if (saveLocks.get(tabId)) return;
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

		const newContent = buildFullContent(updatedProps, body);
		// Do NOT update the store during autosave — it triggers full reactivity cascade.
		// The editor owns the content. Store is synced on tab switch / note reload.
		recentWrites.set(filePath, Date.now());
		await writeNote(filePath, newContent);
		emit('screen:note-saved', { path: filePath }).catch(() => {});
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
export function navigateBack() {
	const tab = get(splitActive) ? get(focusedTab) : get(activeTab);
	if (!tab || tab.historyIndex <= 0) return;
	const newIndex = tab.historyIndex - 1;
	const targetPath = tab.history[newIndex];
	loadTabHistoryEntry(tab.id, targetPath, newIndex);
}

export function navigateForward() {
	const tab = get(splitActive) ? get(focusedTab) : get(activeTab);
	if (!tab || tab.historyIndex >= tab.history.length - 1) return;
	const newIndex = tab.historyIndex + 1;
	const targetPath = tab.history[newIndex];
	loadTabHistoryEntry(tab.id, targetPath, newIndex);
}

async function loadTabHistoryEntry(tabId: string, filePath: string, newHistoryIndex: number) {
	try {
		const content: string = await invoke('read_note', { filePath });
		const name = filePath.split(/[\\/]/).pop()?.replace('.md', '') ?? '';
		openTabs.update(tabs => tabs.map(t => {
			if (t.id !== tabId) return t;
			return { ...t, path: filePath, content, name, historyIndex: newHistoryIndex, highlightTerm: undefined };
		}));
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

export function reconstructFrontmatter(properties: FrontmatterProperty[]): string {
	if (properties.length === 0) return '';

	const lines: string[] = ['---'];
	for (const prop of properties) {
		if (prop.type === 'list' && prop.listItems && prop.listItems.length > 0) {
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

export async function writeNote(filePath: string, content: string): Promise<void> {
	await invoke('write_note', { filePath, content });
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

export async function openNoteTab(filePath: string, libraryName: string, color: string = '#7c3aed', highlightTerm?: string, newTab?: boolean) {
	const tabs = get(openTabs);

	// If the same file is already the active tab, just update highlight
	const currentTab = get(splitActive) ? get(focusedTab) : get(activeTab);
	if (currentTab && currentTab.path === filePath) {
		if (highlightTerm) {
			openTabs.update(tabs => tabs.map(t => t.id === currentTab.id ? { ...t, highlightTerm } : t));
		}
		return;
	}

	/* Check write-ahead buffer first — it has the latest content if the
	   async disk write from a previous close hasn't completed yet. */
	const wab = getWriteAhead(filePath);
	let content: string;
	if (wab) {
		content = wab.content;
		clearWriteAhead(filePath);
	} else {
		try {
			content = await invoke('read_note', { filePath });
		} catch {
			return; // File may not exist or be readable
		}
	}
	const name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';

	// Derive library path from registered libraries
	const allLibraries = get(libraries);
	const library = allLibraries.find(v => filePath.startsWith(v.path));
	const libraryPath = library?.path ?? '';

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
				libraryName,
				libraryPath,
				libraryColor: color,
				highlightTerm,
				history: trimmedHistory,
				historyIndex: newHistoryIndex,
				/* Restore cursor/scroll from write-ahead buffer if available */
				cursorPos: wab?.cursorPos ?? 0,
				scrollTop: wab?.scrollTop ?? 0,
			};
		}));
		// Auto-enable editing mode (WYSIWYG is always edit-ready)
		editingTabIds.update(set => { const next = new Set(set); next.add(currentTab.id); return next; });
		return;
	}

	// Ctrl+click / new tab: create a new tab
	const id = `tab_${++tabCounter}_${Date.now()}`;
	const tab: OpenTab = {
		id, path: filePath, content, libraryName, libraryPath, name, libraryColor: color, highlightTerm,
		history: [filePath], historyIndex: 0,
		/* Restore cursor/scroll from write-ahead buffer if available */
		cursorPos: wab?.cursorPos ?? 0,
		scrollTop: wab?.scrollTop ?? 0,
	};
	openTabs.update(tabs => [...tabs, tab]);

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
}

export async function universalSearch(query: string, limit?: number): Promise<UniversalSearchResponse> {
	return invoke('constellation_search_universal', { query, limit: limit ?? 15 });
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

	const hasFilters = Object.values(filters).some(v => v && (Array.isArray(v) ? v.length > 0 : true));
	const hasQuery = freeText.trim().length > 0;

	return {
		query: hasQuery ? freeText.trim() : undefined,
		mode: hasQuery && hasFilters ? 'hybrid' : hasQuery ? 'lexical' : 'structured',
		filters: hasFilters ? filters : undefined,
		limit: 50,
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

	// Add user-defined default properties
	if (settings.defaultProperties) {
		for (const prop of settings.defaultProperties) {
			if (prop.key && prop.key !== 'created') {
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

export async function renameItem(oldPath: string, newPath: string): Promise<void> {
	await invoke('rename_item', { oldPath, newPath });
	// Update any open tabs that reference the old path
	openTabs.update(tabs => tabs.map(t => {
		if (t.path === oldPath) {
			const newName = newPath.split(/[\\/]/).pop()?.replace('.md', '') ?? t.name;
			return { ...t, path: newPath, name: newName };
		}
		// If a folder was renamed, update paths that start with the old folder path
		if (t.path.startsWith(oldPath + '/') || t.path.startsWith(oldPath + '\\')) {
			const relative = t.path.substring(oldPath.length);
			return { ...t, path: newPath + relative };
		}
		return t;
	}));
}

export async function moveItem(sourcePath: string, targetFolder: string): Promise<string> {
	const newPath = await invoke<string>('move_item', { sourcePath, targetFolder });
	// Update any open tabs that reference the old path
	openTabs.update(tabs => tabs.map(t => {
		if (t.path === sourcePath) {
			const newName = newPath.split(/[\\/]/).pop()?.replace('.md', '') ?? t.name;
			return { ...t, path: newPath, name: newName };
		}
		// If a folder was moved, update paths under it
		if (t.path.startsWith(sourcePath + '/') || t.path.startsWith(sourcePath + '\\')) {
			const relative = t.path.substring(sourcePath.length);
			return { ...t, path: targetFolder + relative };
		}
		return t;
	}));
	return newPath;
}

export async function deleteItem(path: string, permanent = false): Promise<void> {
	await invoke('delete_item', { path, permanent });
	// Close any tabs with this path or under this folder
	openTabs.update(tabs => tabs.filter(t => {
		if (t.path === path) return false;
		if (t.path.startsWith(path + '/') || t.path.startsWith(path + '\\')) return false;
		return true;
	}));
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
}

export async function scanLibraryLinks(libraryPath: string, libraryName: string): Promise<NoteLink[]> {
	return await invoke('scan_library_links', { libraryPath, libraryName });
}

export function getBacklinks(allLinks: NoteLink[], noteName: string) {
	const linked = allLinks.filter(l =>
		l.target.toLowerCase() === noteName.toLowerCase()
	);
	return linked.map(l => ({
		name: l.source_name,
		path: l.source_path,
		context: l.context,
		libraryName: l.library_name,
		linkType: l.link_type ?? undefined,
	}));
}

export function getOutgoingLinks(allLinks: NoteLink[], notePath: string) {
	return allLinks.filter(l => l.source_path === notePath);
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
export interface IndexMention {
	note_path: string;
	note_name: string;
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
export interface StarNode {
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

export interface StarLink {
	source: string;
	target: string;
	linkType?: string;
}

export function buildStarData(allLinks: NoteLink[], allNotes: { name: string; path: string; libraryName: string }[]) {
	const nodeMap = new Map<string, StarNode>();
	// Add all notes as nodes
	for (const note of allNotes) {
		nodeMap.set(note.name.toLowerCase(), {
			id: note.name.toLowerCase(),
			name: note.name,
			path: note.path,
			libraryName: note.libraryName,
			linkCount: 0,
			outgoingCount: 0
		});
	}

	const links: StarLink[] = [];
	const seen = new Set<string>();

	for (const link of allLinks) {
		const sourceId = link.source_name.toLowerCase();
		const targetId = link.target.toLowerCase();

		if (!nodeMap.has(sourceId) || !nodeMap.has(targetId)) continue;

		// Include link_type in key so typed links with different types are kept as distinct edges
		const key = `${sourceId}->${targetId}:${link.link_type ?? ''}`;
		if (seen.has(key)) continue;
		seen.add(key);

		links.push({ source: sourceId, target: targetId, linkType: link.link_type || undefined });
		nodeMap.get(sourceId)!.linkCount++;
		nodeMap.get(sourceId)!.outgoingCount++;
		nodeMap.get(targetId)!.linkCount++;
	}

	// Only include nodes that have at least one link
	const nodes = Array.from(nodeMap.values()).filter(n => n.linkCount > 0);

	return { nodes, links };
}

// ─── Daily notes ───
export async function getDailyNotePath(libraryPath: string, format = '%Y-%m-%d', folder = ''): Promise<string> {
	return await invoke('get_daily_note_path', { libraryPath, format, folder });
}

// ─── Link update on rename ───
export async function updateLinksOnRename(libraryPath: string, oldName: string, newName: string): Promise<number> {
	return await invoke('update_links_on_rename', { libraryPath, oldName, newName });
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
export interface AppSettings {
	// Editor
	showLineNumbers: boolean;
	readableLineLength: boolean;
	tabSize: number;
	indentWithTabs: boolean;
	smartLists: boolean;
	autoPairBrackets: boolean;
	autoPairMarkdown: boolean;
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
	confirmDelete: boolean;
	trashDestination: 'system' | 'local' | 'permanent';

	// Appearance
	titleAlignment: 'start' | 'center';
	colorScheme: 'light' | 'dark' | 'system';
	accentColor: string;
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

	// Built-in features
	enabledFeatures: {
		dailyNotes: boolean;
		templates: boolean;
		starView: boolean;
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
	};
}

const DEFAULT_SETTINGS: AppSettings = {
	showLineNumbers: true,
	readableLineLength: true,
	tabSize: 4,
	indentWithTabs: true,
	smartLists: true,
	autoPairBrackets: true,
	autoPairMarkdown: true,
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
	confirmDelete: true,
	trashDestination: 'system',
	titleAlignment: 'center',
	colorScheme: 'light',
	accentColor: '#7c3aed',
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
	enabledFeatures: {
		dailyNotes: true,
		templates: true,
		starView: true,
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
	},
	customShortcuts: {},
};

export const appSettings = writable<AppSettings>(DEFAULT_SETTINGS);

export async function loadSettings() {
	try {
		const parsed = await invoke<Record<string, unknown>>('read_universe_settings');
		if (parsed && Object.keys(parsed).length > 0) {
			// Migrate: old default nodeSize was 4, new default is 1.5
		const savedSkyView = (parsed.skyView as Record<string, unknown>) || {};
		if (savedSkyView.nodeSize === 4) savedSkyView.nodeSize = 1.5;

		appSettings.set({
				...DEFAULT_SETTINGS,
				...(parsed as Partial<AppSettings>),
				skyView: { ...DEFAULT_SETTINGS.skyView, ...savedSkyView },
				security: { ...DEFAULT_SETTINGS.security, ...((parsed.security as Record<string, unknown>) || {}) },
				enabledFeatures: { ...DEFAULT_SETTINGS.enabledFeatures, ...((parsed.enabledFeatures as Record<string, boolean>) ?? (parsed.enabledPlugins as Record<string, boolean>) ?? {}) },
				customShortcuts: { ...((parsed.customShortcuts as Record<string, string>) || {}) },
			});
		}
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

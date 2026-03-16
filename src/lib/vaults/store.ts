/**
 * Vault (Universe) state management.
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface VaultInfo {
	id: string;
	name: string;
	path: string;
}

export interface StarInfo {
	name: string;
	path: string;
	vault_id: string;
	vault_name: string;
	modified: number;
	preview: string;
}

export interface VaultStats {
	vault_id: string;
	name: string;
	path: string;
	star_count: number;
	folder_count: number;
	recent_stars: StarInfo[];
}

export interface FileEntry {
	name: string;
	path: string;
	is_dir: boolean;
	children: FileEntry[] | null;
	extension: string | null;
	modified: number | null;
}

export interface OpenTab {
	id: string;
	path: string;
	content: string;
	vaultName: string;
	vaultPath: string;
	name: string;
	vaultColor: string;
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
export const vaults = writable<VaultInfo[]>([]);
export const vaultStats = writable<VaultStats[]>([]);
export const searchResults = writable<StarInfo[]>([]);

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
		updateTabContent(tabId, newContent);
		recentWrites.set(filePath, Date.now());
		await writeNote(filePath, newContent);
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
	($tab) => $tab ? { path: $tab.path, content: $tab.content, vaultName: $tab.vaultName } : null
);

export const vaultCount = derived(vaults, ($v) => $v.length);
export const totalStars = derived(vaultStats, ($s) => $s.reduce((sum, v) => sum + v.star_count, 0));

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
	vaultName: string;
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
	openTabs.update(tabs =>
		tabs.map(t => t.id === tabId ? { ...t, content: newContent } : t)
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
		id, path: '', content: '', vaultName: '', vaultPath: '', name: 'New tab', vaultColor: '#7c3aed',
		history: [], historyIndex: -1,
	};
	openTabs.update(tabs => [...tabs, tab]);
	if (get(splitActive)) {
		focusedTabId.set(id);
	} else {
		activeTabId.set(id);
	}
}

export async function openNoteTab(filePath: string, vaultName: string, color: string = '#7c3aed', highlightTerm?: string, newTab?: boolean) {
	const tabs = get(openTabs);

	// If the same file is already the active tab, just update highlight
	const currentTab = get(splitActive) ? get(focusedTab) : get(activeTab);
	if (currentTab && currentTab.path === filePath) {
		if (highlightTerm) {
			openTabs.update(tabs => tabs.map(t => t.id === currentTab.id ? { ...t, highlightTerm } : t));
		}
		return;
	}

	const content: string = await invoke('read_note', { filePath });
	const name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';

	// Derive vault path from registered vaults
	const allVaults = get(vaults);
	const vault = allVaults.find(v => filePath.startsWith(v.path));
	const vaultPath = vault?.path ?? '';

	// Default: replace active tab content (single-tab navigation)
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
				vaultName,
				vaultPath,
				vaultColor: color,
				highlightTerm,
				history: trimmedHistory,
				historyIndex: newHistoryIndex,
			};
		}));
		return;
	}

	// Ctrl+click / new tab: create a new tab
	const id = `tab_${++tabCounter}_${Date.now()}`;
	const tab: OpenTab = {
		id, path: filePath, content, vaultName, vaultPath, name, vaultColor: color, highlightTerm,
		history: [filePath], historyIndex: 0,
	};
	openTabs.update(tabs => [...tabs, tab]);

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
	openTabs.set(newTabs);

	// Clean up editing state and save locks for closed tab
	editingTabIds.update(set => {
		if (set.has(tabId)) {
			const next = new Set(set);
			next.delete(tabId);
			return next;
		}
		return set;
	});
	saveLocks.delete(tabId);

	if (currentActive === tabId) {
		if (newTabs.length > 0) {
			const newIdx = Math.min(idx, newTabs.length - 1);
			activeTabId.set(newTabs[newIdx].id);
		} else {
			activeTabId.set(null);
		}
	}

	// Update focused tab if it was closed
	if (get(focusedTabId) === tabId) {
		if (newTabs.length > 0) {
			const newIdx = Math.min(idx, newTabs.length - 1);
			focusedTabId.set(newTabs[newIdx].id);
		} else {
			focusedTabId.set(null);
		}
	}
}

export function switchTab(tabId: string) {
	if (get(splitActive)) {
		focusedTabId.set(tabId);
	} else {
		activeTabId.set(tabId);
	}
}

/** Load vaults including child universe vaults. */
export async function loadVaults() {
	let list: VaultInfo[];
	try {
		// Resolve own vaults + child universe vaults (recursive, deduplicated)
		list = await invoke('resolve_universe_vaults');
	} catch {
		// Fallback to own vaults only
		list = await invoke('list_vaults');
	}
	vaults.set(list);
}

/** Load stats for all vaults (star counts, recent stars). */
export async function loadAllStats() {
	const stats: VaultStats[] = await invoke('get_all_vault_stats');
	vaultStats.set(stats);
}

/** Open folder picker and add the selected vault. */
export async function addVault(): Promise<VaultInfo | null> {
	const folderPath: string | null = await invoke('pick_folder');
	if (!folderPath) return null;

	const vault: VaultInfo = await invoke('add_vault', { path: folderPath });
	await loadVaults();
	await loadAllStats();
	return vault;
}

/** Remove a vault (does NOT delete files). */
export async function removeVault(vaultId: string) {
	await invoke('remove_vault', { vaultId });
	await loadVaults();
	await loadAllStats();
}

/** Remove a vault with full cleanup: close tabs, stop watcher, refresh caches. */
export async function removeVaultWithCleanup(vaultId: string) {
	// Close all open tabs from this vault
	const tabs = get(openTabs);
	const vault = get(vaults).find(v => v.id === vaultId);
	if (vault) {
		const vaultTabs = tabs.filter(t => t.path.startsWith(vault.path));
		for (const tab of vaultTabs) closeTab(tab.id);
	}
	// Stop file watcher
	try { await stopWatchingVault(vaultId); } catch { /* ignore */ }
	// Remove from registry
	await removeVault(vaultId);
}

/** Search across all vaults. */
export async function searchAllStars(query: string) {
	if (!query.trim()) {
		searchResults.set([]);
		return;
	}
	const results: StarInfo[] = await invoke('search_stars', { query });
	searchResults.set(results);
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

/** Search notes by property key/value across all vaults */
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
	vault_name: string;
	vault_path: string;
	fragment: string | null;
}

export async function resolveWikilink(vaultPath: string, target: string): Promise<string | null> {
	return await invoke('resolve_wikilink', { vaultPath, target });
}

export async function resolveWikilinkCrossVault(currentVaultPath: string, target: string): Promise<ResolvedLink | null> {
	const vaultList = get(vaults).map(v => [v.id, v.name, v.path] as [string, string, string]);
	return await invoke('resolve_wikilink_cross_vault', { vaults: vaultList, currentVaultPath, target });
}

export async function getNoteHeadings(filePath: string): Promise<string[]> {
	return await invoke('get_note_headings', { filePath });
}

// ─── File watcher ───
export async function startWatchingVault(vaultId: string, vaultPath: string): Promise<void> {
	await invoke('watch_vault', { vaultId, vaultPath });
}

export async function stopWatchingVault(vaultId: string): Promise<void> {
	await invoke('unwatch_vault', { vaultId });
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

export const vaultAppearances = writable<Record<string, LibraryAppearance>>({});

export async function loadVaultAppearance(vaultPath: string, vaultId: string): Promise<void> {
	try {
		const appearance: LibraryAppearance = await invoke('read_library_appearance', { vaultPath });
		vaultAppearances.update(map => ({ ...map, [vaultId]: appearance }));
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
	vault_name: string;
	link_type: string | null;
}

export async function scanVaultLinks(vaultPath: string, vaultName: string): Promise<NoteLink[]> {
	return await invoke('scan_vault_links', { vaultPath, vaultName });
}

export function getBacklinks(allLinks: NoteLink[], noteName: string) {
	const linked = allLinks.filter(l =>
		l.target.toLowerCase() === noteName.toLowerCase()
	);
	return linked.map(l => ({
		name: l.source_name,
		path: l.source_path,
		context: l.context,
		vaultName: l.vault_name
	}));
}

export function getOutgoingLinks(allLinks: NoteLink[], notePath: string) {
	return allLinks.filter(l => l.source_path === notePath);
}

export async function scanUnlinkedMentions(noteName: string, notePath: string): Promise<{ name: string; path: string; context: string; vaultName: string }[]> {
	const vaultList = get(vaults).map(v => [v.name, v.path] as [string, string]);
	const links: NoteLink[] = await invoke('scan_unlinked_mentions', { noteName, notePath, vaultPaths: vaultList });
	return links.map(l => ({
		name: l.source_name,
		path: l.source_path,
		context: l.context,
		vaultName: l.vault_name
	}));
}

// ─── Tags scanning ───
export async function scanVaultTags(vaultPath: string): Promise<Record<string, number>> {
	return await invoke('scan_vault_tags', { vaultPath });
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

export async function scanVaultIndex(vaultPath: string): Promise<IndexEntry[]> {
	return await invoke('scan_vault_index', { vaultPath });
}

// ─── Graph data ───
export interface GraphNode {
	id: string;
	name: string;
	path: string;
	vaultName: string;
	linkCount: number;
}

export interface GraphLink {
	source: string;
	target: string;
	linkType?: string;
}

export function buildGraphData(allLinks: NoteLink[], allNotes: { name: string; path: string; vaultName: string }[]) {
	const nodeMap = new Map<string, GraphNode>();
	// Add all notes as nodes
	for (const note of allNotes) {
		nodeMap.set(note.name.toLowerCase(), {
			id: note.name.toLowerCase(),
			name: note.name,
			path: note.path,
			vaultName: note.vaultName,
			linkCount: 0
		});
	}

	const links: GraphLink[] = [];
	const seen = new Set<string>();

	for (const link of allLinks) {
		const sourceId = link.source_name.toLowerCase();
		const targetId = link.target.toLowerCase();

		if (!nodeMap.has(sourceId) || !nodeMap.has(targetId)) continue;

		const key = `${sourceId}->${targetId}`;
		if (seen.has(key)) continue;
		seen.add(key);

		links.push({ source: sourceId, target: targetId, linkType: link.link_type || undefined });
		nodeMap.get(sourceId)!.linkCount++;
		nodeMap.get(targetId)!.linkCount++;
	}

	// Only include nodes that have at least one link
	const nodes = Array.from(nodeMap.values()).filter(n => n.linkCount > 0);

	return { nodes, links };
}

// ─── Daily notes ───
export async function getDailyNotePath(vaultPath: string, format = '%Y-%m-%d', folder = ''): Promise<string> {
	return await invoke('get_daily_note_path', { vaultPath, format, folder });
}

// ─── Link update on rename ───
export async function updateLinksOnRename(vaultPath: string, oldName: string, newName: string): Promise<number> {
	return await invoke('update_links_on_rename', { vaultPath, oldName, newName });
}

// ─── Note reading ───
export async function readNote(filePath: string): Promise<string> {
	return await invoke('read_note', { filePath });
}

export async function readNotePreview(filePath: string, maxChars = 500): Promise<string> {
	return await invoke('read_note_preview', { filePath, maxChars });
}

// ─── Settings store ───
export interface AppSettings {
	// Editor
	defaultView: 'reading' | 'editing';
	defaultEditingMode: 'livePreview' | 'source';
	showLineNumbers: boolean;
	readableLineLength: boolean;
	tabSize: number;
	indentWithTabs: boolean;
	smartLists: boolean;
	autoPairBrackets: boolean;
	autoPairMarkdown: boolean;
	spellcheck: boolean;
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
	colorScheme: 'light' | 'dark' | 'system';
	accentColor: string;
	interfaceFont: string;
	textFont: string;
	monoFont: string;
	fontSize: number;

	// Daily notes
	dailyNoteFormat: string;
	dailyNoteFolder: string;
	dailyNoteTemplate: string;

	// Templates
	templateFolder: string;

	// Default properties for new notes
	defaultProperties: { key: string; value: string }[];

	// Updates
	autoUpdate: boolean;

	// Security
	security: {
		vaultEncryption: boolean;
		lockOnIdle: boolean;
		lockIdleTimeout: number;
		lockPinHash: string;
		apiKeyProtection: boolean;
	};

	// Built-in features
	enabledPlugins: {
		dailyNotes: boolean;
		templates: boolean;
		graphView: boolean;
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
	defaultView: 'reading',
	defaultEditingMode: 'livePreview',
	showLineNumbers: true,
	readableLineLength: true,
	tabSize: 4,
	indentWithTabs: true,
	smartLists: true,
	autoPairBrackets: true,
	autoPairMarkdown: true,
	spellcheck: false,
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
	colorScheme: 'light',
	accentColor: '#7c3aed',
	interfaceFont: '',
	textFont: '',
	monoFont: '',
	fontSize: 15,
	dailyNoteFormat: '%Y-%m-%d',
	dailyNoteFolder: '',
	dailyNoteTemplate: '',
	templateFolder: 'Templates',
	defaultProperties: [],
	autoUpdate: true,
	security: {
		vaultEncryption: false,
		lockOnIdle: false,
		lockIdleTimeout: 5,
		lockPinHash: '',
		apiKeyProtection: false,
	},
	enabledPlugins: {
		dailyNotes: true,
		templates: true,
		graphView: true,
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
};

export const appSettings = writable<AppSettings>(DEFAULT_SETTINGS);

export async function loadSettings() {
	try {
		const parsed = await invoke<Record<string, unknown>>('read_universe_settings');
		if (parsed && Object.keys(parsed).length > 0) {
			appSettings.set({
				...DEFAULT_SETTINGS,
				...(parsed as Partial<AppSettings>),
				security: { ...DEFAULT_SETTINGS.security, ...((parsed.security as Record<string, unknown>) || {}) },
				enabledPlugins: { ...DEFAULT_SETTINGS.enabledPlugins, ...((parsed.enabledPlugins as Record<string, boolean>) || {}) },
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
	tabs: { path: string; vaultName: string; vaultColor: string }[];
	activeTabPath: string | null;
}

export interface Workspace {
	id: string;
	name: string;
	tabs: { path: string; vaultName: string; vaultColor: string }[];
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
		vaultName: t.vaultName,
		vaultColor: t.vaultColor,
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
			await openNoteTab(saved.path, saved.vaultName, saved.vaultColor);
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
export async function saveClipboardImage(vaultPath: string, imageData: string): Promise<string> {
	return await invoke('save_clipboard_image', { vaultPath, imageData });
}

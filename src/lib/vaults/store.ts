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
}

export interface OpenTab {
	id: string;
	path: string;
	content: string;
	vaultName: string;
	vaultPath: string;
	name: string;
	vaultColor: string;
}

export type PropertyType = 'text' | 'number' | 'date' | 'list' | 'link';

export interface FrontmatterProperty {
	key: string;
	value: string;
	type: PropertyType;
	listItems?: string[];
}

function detectPropertyType(key: string, value: string): PropertyType {
	const listKeys = ['tags', 'aliases', 'cssclasses', 'cssclass'];
	if (listKeys.includes(key.toLowerCase())) return 'list';
	if (value.startsWith('[') && value.endsWith(']')) return 'list';

	if (/^\[\[.*\]\]$/.test(value)) return 'link';

	if (/^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}(:\d{2})?)?$/.test(value)) return 'date';
	const dateKeys = ['date', 'created', 'updated', 'modified', 'due', 'published', 'start', 'end'];
	if (dateKeys.includes(key.toLowerCase()) && value) return 'date';

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
		const newContent = buildFullContent(properties, body);
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

// ─── Navigation history ───
const navHistory = writable<string[]>([]);
const navHistoryIndex = writable<number>(-1);

export function pushNavHistory(tabId: string) {
	navHistory.update(h => {
		const idx = get(navHistoryIndex);
		const trimmed = h.slice(0, idx + 1);
		trimmed.push(tabId);
		if (trimmed.length > 50) trimmed.shift();
		navHistoryIndex.set(trimmed.length - 1);
		return trimmed;
	});
}

export function navigateBack() {
	const idx = get(navHistoryIndex);
	const hist = get(navHistory);
	if (idx > 0) {
		navHistoryIndex.set(idx - 1);
		const tabId = hist[idx - 1];
		if (get(splitActive)) focusedTabId.set(tabId);
		else activeTabId.set(tabId);
	}
}

export function navigateForward() {
	const idx = get(navHistoryIndex);
	const hist = get(navHistory);
	if (idx < hist.length - 1) {
		navHistoryIndex.set(idx + 1);
		const tabId = hist[idx + 1];
		if (get(splitActive)) focusedTabId.set(tabId);
		else activeTabId.set(tabId);
	}
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
	try {
		localStorage.setItem('constellation-bookmarks', JSON.stringify(get(bookmarks)));
	} catch { /* ignore */ }
}

export function loadBookmarks() {
	try {
		const data = localStorage.getItem('constellation-bookmarks');
		if (data) bookmarks.set(JSON.parse(data));
	} catch { /* ignore */ }
}

// ─── Frontmatter parsing ───
export function parseFrontmatter(content: string): { properties: FrontmatterProperty[]; body: string } {
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
	return { properties, body };
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
		} else if (prop.type === 'date' || prop.type === 'number' || prop.type === 'link') {
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

export async function openNoteTab(filePath: string, vaultName: string, color: string = '#7c3aed') {
	const tabs = get(openTabs);

	const existing = tabs.find(t => t.path === filePath);
	if (existing) {
		if (get(splitActive)) {
			focusedTabId.set(existing.id);
		} else {
			activeTabId.set(existing.id);
		}
		pushNavHistory(existing.id);
		return;
	}

	const content: string = await invoke('read_note', { filePath });
	const name = filePath.split(/[\\/]/).pop()?.replace('.md', '') ?? '';
	const id = `tab_${++tabCounter}_${Date.now()}`;

	// Derive vault path from registered vaults
	const allVaults = get(vaults);
	const vault = allVaults.find(v => filePath.startsWith(v.path));
	const vaultPath = vault?.path ?? '';

	const tab: OpenTab = { id, path: filePath, content, vaultName, vaultPath, name, vaultColor: color };
	openTabs.update(tabs => [...tabs, tab]);

	if (get(splitActive)) {
		focusedTabId.set(id);
	} else {
		activeTabId.set(id);
	}
	pushNavHistory(id);
}

export function closeTab(tabId: string) {
	const tabs = get(openTabs);
	const idx = tabs.findIndex(t => t.id === tabId);
	if (idx === -1) return;

	const currentActive = get(activeTabId);
	const newTabs = tabs.filter(t => t.id !== tabId);
	openTabs.set(newTabs);

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
	pushNavHistory(tabId);
}

/** Load vaults and their stats. */
export async function loadVaults() {
	const list: VaultInfo[] = await invoke('list_vaults');
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
export async function createNote(folderPath: string, fileName: string): Promise<string> {
	const newPath: string = await invoke('create_note', { folderPath, fileName });
	return newPath;
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
export async function resolveWikilink(vaultPath: string, target: string): Promise<string | null> {
	return await invoke('resolve_wikilink', { vaultPath, target });
}

// ─── File watcher ───
export async function startWatchingVault(vaultId: string, vaultPath: string): Promise<void> {
	await invoke('watch_vault', { vaultId, vaultPath });
}

export async function stopWatchingVault(vaultId: string): Promise<void> {
	await invoke('unwatch_vault', { vaultId });
}

// ─── Vault appearance ───
export interface ObsidianAppearance {
	accent_color: string | null;
	base_font_size: number | null;
	text_font_family: string | null;
	monospace_font_family: string | null;
	interface_font_family: string | null;
	css_theme: string | null;
}

export const vaultAppearances = writable<Record<string, ObsidianAppearance>>({});

export async function loadVaultAppearance(vaultPath: string, vaultId: string): Promise<void> {
	try {
		const appearance: ObsidianAppearance = await invoke('read_obsidian_appearance', { vaultPath });
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
}

export async function scanVaultLinks(vaultPath: string): Promise<NoteLink[]> {
	return await invoke('scan_vault_links', { vaultPath });
}

export function getBacklinks(allLinks: NoteLink[], noteName: string) {
	const linked = allLinks.filter(l =>
		l.target.toLowerCase() === noteName.toLowerCase()
	);
	return linked.map(l => ({
		name: l.source_name,
		path: l.source_path,
		context: l.context,
		vaultName: ''
	}));
}

export function getOutgoingLinks(allLinks: NoteLink[], notePath: string) {
	return allLinks.filter(l => l.source_path === notePath);
}

// ─── Tags scanning ───
export async function scanVaultTags(vaultPath: string): Promise<Record<string, number>> {
	return await invoke('scan_vault_tags', { vaultPath });
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

		links.push({ source: sourceId, target: targetId });
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

// ─── Note preview ───
export async function readNotePreview(filePath: string, maxChars = 500): Promise<string> {
	return await invoke('read_note_preview', { filePath, maxChars });
}

// ─── Settings store ───
export interface AppSettings {
	// Editor
	defaultView: 'reading' | 'editing';
	showLineNumbers: boolean;
	readableLineLength: boolean;
	tabSize: number;
	smartLists: boolean;
	autoPairBrackets: boolean;
	spellcheck: boolean;

	// Files & Links
	defaultNoteLocation: 'root' | 'current' | 'folder';
	defaultNoteFolder: string;
	defaultAttachmentFolder: string;
	linkFormat: 'shortest' | 'relative' | 'absolute';
	autoUpdateLinks: boolean;
	useWikilinks: boolean;
	confirmDelete: boolean;
	trashDestination: 'system' | 'obsidian' | 'permanent';

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
}

const DEFAULT_SETTINGS: AppSettings = {
	defaultView: 'reading',
	showLineNumbers: true,
	readableLineLength: true,
	tabSize: 4,
	smartLists: true,
	autoPairBrackets: true,
	spellcheck: false,
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
};

export const appSettings = writable<AppSettings>(DEFAULT_SETTINGS);

export function loadSettings() {
	try {
		const data = localStorage.getItem('constellation-settings');
		if (data) {
			const parsed = JSON.parse(data);
			appSettings.set({ ...DEFAULT_SETTINGS, ...parsed });
		}
	} catch { /* ignore */ }
}

export function saveSettings() {
	try {
		localStorage.setItem('constellation-settings', JSON.stringify(get(appSettings)));
	} catch { /* ignore */ }
}

export function updateSettings(partial: Partial<AppSettings>) {
	appSettings.update(s => ({ ...s, ...partial }));
	saveSettings();
}

// ─── Workspaces ───
export interface Workspace {
	id: string;
	name: string;
	tabs: { path: string; vaultName: string; vaultColor: string }[];
	activeTabPath: string | null;
	splitActive: boolean;
	splitDir: SplitDirection;
	timestamp: number;
}

export const workspaces = writable<Workspace[]>([]);

export function loadWorkspaces() {
	try {
		const data = localStorage.getItem('constellation-workspaces');
		if (data) workspaces.set(JSON.parse(data));
	} catch { /* ignore */ }
}

function persistWorkspaces() {
	try {
		localStorage.setItem('constellation-workspaces', JSON.stringify(get(workspaces)));
	} catch { /* ignore */ }
}

export function saveWorkspace(name: string) {
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
	};

	workspaces.update(list => {
		// Replace if same name exists
		const filtered = list.filter(w => w.name !== name);
		return [...filtered, ws];
	});
	persistWorkspaces();
}

export async function restoreWorkspace(ws: Workspace) {
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
}

export function deleteWorkspace(id: string) {
	workspaces.update(list => list.filter(w => w.id !== id));
	persistWorkspaces();
}

// ─── Clipboard image paste ───
export async function saveClipboardImage(vaultPath: string, imageData: string): Promise<string> {
	return await invoke('save_clipboard_image', { vaultPath, imageData });
}

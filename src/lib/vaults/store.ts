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
	name: string;
}

export interface FrontmatterProperty {
	key: string;
	value: string;
}

// ─── Core state ───
export const vaults = writable<VaultInfo[]>([]);
export const vaultStats = writable<VaultStats[]>([]);
export const searchResults = writable<StarInfo[]>([]);

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

	for (const line of yamlLines) {
		const colonIdx = line.indexOf(':');
		if (colonIdx > 0) {
			const key = line.substring(0, colonIdx).trim();
			let value = line.substring(colonIdx + 1).trim();
			// Strip quotes
			if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
				value = value.slice(1, -1);
			}
			if (key) {
				properties.push({ key, value });
			}
		}
	}

	const body = lines.slice(endIndex + 1).join('\n');
	return { properties, body };
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

export async function openNoteTab(filePath: string, vaultName: string) {
	const tabs = get(openTabs);

	const existing = tabs.find(t => t.path === filePath);
	if (existing) {
		if (get(splitActive)) {
			focusedTabId.set(existing.id);
		} else {
			activeTabId.set(existing.id);
		}
		return;
	}

	const content: string = await invoke('read_note', { filePath });
	const name = filePath.split(/[\\/]/).pop()?.replace('.md', '') ?? '';
	const id = `tab_${++tabCounter}_${Date.now()}`;

	const tab: OpenTab = { id, path: filePath, content, vaultName, name };
	openTabs.update(tabs => [...tabs, tab]);

	if (get(splitActive)) {
		focusedTabId.set(id);
	} else {
		activeTabId.set(id);
	}
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

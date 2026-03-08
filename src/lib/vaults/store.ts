/**
 * Vault (Universe) state management.
 */

import { writable, derived } from 'svelte/store';
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

// Core state
export const vaults = writable<VaultInfo[]>([]);
export const vaultStats = writable<VaultStats[]>([]);
export const searchResults = writable<StarInfo[]>([]);
export const selectedNote = writable<{ path: string; content: string; vaultName: string } | null>(null);

export const vaultCount = derived(vaults, ($v) => $v.length);
export const totalStars = derived(vaultStats, ($s) => $s.reduce((sum, v) => sum + v.star_count, 0));

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

/** Read a note's content. */
export async function openNote(star: StarInfo) {
	const content: string = await invoke('read_note', { filePath: star.path });
	selectedNote.set({ path: star.path, content, vaultName: star.vault_name });
}

/** Close the current note. */
export function closeNote() {
	selectedNote.set(null);
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

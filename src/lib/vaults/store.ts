/**
 * Vault state management — tracks registered vaults and selected vault.
 */

import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface VaultInfo {
	id: string;
	name: string;
	path: string;
}

export interface FileEntry {
	name: string;
	path: string;
	is_dir: boolean;
	children: FileEntry[] | null;
	extension: string | null;
}

export const vaults = writable<VaultInfo[]>([]);
export const selectedVaultId = writable<string | null>(null);
export const vaultTree = writable<FileEntry[]>([]);
export const selectedNote = writable<{ path: string; content: string } | null>(null);

export const selectedVault = derived(
	[vaults, selectedVaultId],
	([$vaults, $id]) => $vaults.find((v) => v.id === $id) ?? null
);

export const vaultCount = derived(vaults, ($v) => $v.length);

/** Load vaults from Rust backend. */
export async function loadVaults() {
	const list: VaultInfo[] = await invoke('list_vaults');
	vaults.set(list);
}

/** Open folder picker and add the selected vault. */
export async function addVault(): Promise<VaultInfo | null> {
	const folderPath: string | null = await invoke('pick_folder');
	if (!folderPath) return null;

	const vault: VaultInfo = await invoke('add_vault', { path: folderPath });
	await loadVaults();
	return vault;
}

/** Remove a vault (does NOT delete files). */
export async function removeVault(vaultId: string) {
	await invoke('remove_vault', { vaultId });
	await loadVaults();
	selectedVaultId.update((current) => (current === vaultId ? null : current));
}

/** Load the file tree for a vault. */
export async function loadVaultTree(vaultPath: string, maxDepth?: number) {
	const tree: FileEntry[] = await invoke('read_vault_tree', {
		path: vaultPath,
		maxDepth: maxDepth ?? 3
	});
	vaultTree.set(tree);
}

/** Read a note's content. */
export async function openNote(filePath: string) {
	const content: string = await invoke('read_note', { filePath });
	selectedNote.set({ path: filePath, content });
}

/** Select a vault and load its tree. */
export async function selectVault(vault: VaultInfo) {
	selectedVaultId.set(vault.id);
	await loadVaultTree(vault.path);
	selectedNote.set(null);
}

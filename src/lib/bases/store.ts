// ─── Constellation Bases — Workspace store (IPC bridge) ───
//
// MIG-065 §I — the old live-scan Base MVP (`query_base` + the orphaned
// `BaseView` family) is retired. What remains is the SIDEBAR's workspace-base
// management: list / create / delete the `.base` files shown under the "Bases"
// section. READING and EDITING a base is the unified lens engine's job
// (`$lib/lens/store` + `BaseTab.svelte`). Base creation (§I-b) writes a minimal
// `LensDefinition` YAML — there is no `BaseDefinition` on the live path anymore.

import { invoke } from '@tauri-apps/api/core';

/** Create a new `.base` file in a library folder. Returns the full path. */
export async function createBase(folderPath: string, fileName: string): Promise<string> {
	return await invoke('create_base', { folderPath, fileName });
}

// ─── Workspace-level Base Operations ───

export interface WorkspaceBaseEntry {
	id: string;
	name: string;
	path: string;
	modified: number;
	/** MIG-062 — undefined for the active universe; the cUniverse's display
	 *  name for a federated entry. The sidebar groups by this. */
	universe_name?: string;
}

/** List all bases in the active universe's workspace (+ federated cUniverses). */
export async function listWorkspaceBases(): Promise<WorkspaceBaseEntry[]> {
	return await invoke('list_workspace_bases');
}

/**
 * MIG-065 §I-b — create a new `.base` in the workspace bases directory as a
 * minimal `LensDefinition` YAML, scoped to the chosen libraries (empty = all).
 * Returns the full path. Opens directly in BaseTab.
 */
export async function createWorkspaceBase(
	fileName: string,
	selectedLibraries: string[] = [],
): Promise<string> {
	return await invoke('create_workspace_base', { fileName, selectedLibraries });
}

/** Delete a workspace base file. */
export async function deleteWorkspaceBase(filePath: string): Promise<void> {
	return await invoke('delete_workspace_base', { filePath });
}

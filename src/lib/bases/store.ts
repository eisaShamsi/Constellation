// ─── Constellation Bases — Store (IPC bridge) ───

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { BaseDefinition, BaseQueryResult, BaseRow } from './types';

/**
 * Parse a .base file from disk.
 */
export async function parseBaseFile(filePath: string): Promise<BaseDefinition> {
	return await invoke('parse_base_file', { filePath });
}

/**
 * Query a Base: scan notes, apply filters/sorts, return rows.
 * @param definition The Base definition (parsed from .base file)
 * @param libraryPaths Array of [libraryName, libraryPath] tuples
 */
export async function queryBase(
	definition: BaseDefinition,
	libraryPaths: [string, string][],
): Promise<BaseQueryResult> {
	return await invoke('query_base', { definition, libraryPaths });
}

/**
 * Save a Base definition to disk as JSON.
 */
export async function saveBaseFile(filePath: string, definition: BaseDefinition): Promise<void> {
	return await invoke('save_base_file', { filePath, definition });
}

/**
 * Update a single property in a note's YAML frontmatter.
 */
export async function updateNoteProperty(filePath: string, key: string, value: string): Promise<void> {
	return await invoke('update_note_property', { filePath, key, value });
}

/**
 * Create a new .base file in a folder with default definition.
 * Returns the full path of the created file.
 */
export async function createBase(folderPath: string, fileName: string): Promise<string> {
	return await invoke('create_base', { folderPath, fileName });
}

// ─── Workspace-level Base Operations ───

export interface WorkspaceBaseEntry {
	id: string;
	name: string;
	path: string;
	modified: number;
}

/**
 * List all bases stored in the Constellation workspace directory.
 */
export async function listWorkspaceBases(): Promise<WorkspaceBaseEntry[]> {
	return await invoke('list_workspace_bases');
}

/**
 * Create a new .base file in the Constellation workspace directory.
 * Returns the full path of the created file.
 */
export async function createWorkspaceBase(fileName: string): Promise<string> {
	return await invoke('create_workspace_base', { fileName });
}

/**
 * Save a workspace base definition.
 */
export async function saveWorkspaceBase(filePath: string, definition: BaseDefinition): Promise<void> {
	return await invoke('save_workspace_base', { filePath, definition });
}

/**
 * Delete a workspace base file.
 */
export async function deleteWorkspaceBase(filePath: string): Promise<void> {
	return await invoke('delete_workspace_base', { filePath });
}

/**
 * Parse a workspace base file.
 */
export async function parseWorkspaceBase(filePath: string): Promise<BaseDefinition> {
	return await invoke('parse_workspace_base', { filePath });
}

// ─── MIG-054 §E — Cell-edit refresh event ───

/**
 * Payload emitted by `update_note_property` after a cell-edit.
 * Any Bases view listening can refresh / re-query to reflect the new value.
 */
export interface BasesNoteUpdatedPayload {
	path: string;
	changed_keys: string[];
}

/**
 * Listen for `bases:note_updated` events fired after a cell-edit completes
 * (Rust side updates `note_meta.properties_json` immediately + emits this event).
 *
 * Returns an unlisten function — call it on component destroy.
 *
 * Typical use in a Bases view:
 *   let unlisten: UnlistenFn | null = null;
 *   $effect(() => {
 *     onBasesNoteUpdated(() => runQuery()).then(fn => { unlisten = fn; });
 *     return () => unlisten?.();
 *   });
 */
export async function onBasesNoteUpdated(
	callback: (payload: BasesNoteUpdatedPayload) => void,
): Promise<UnlistenFn> {
	return await listen<BasesNoteUpdatedPayload>('bases:note_updated', (event) => {
		callback(event.payload);
	});
}

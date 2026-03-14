import { invoke } from '@tauri-apps/api/core';
import type { DataviewResult } from './types';

/**
 * Execute a Dataview query against the given vaults.
 * @param queryText - DQL query string (e.g., 'TABLE title, date FROM "folder" WHERE status = "done"')
 * @param vaultPaths - Array of [vaultName, vaultPath] tuples
 */
export async function executeDataviewQuery(
	queryText: string,
	vaultPaths: [string, string][]
): Promise<DataviewResult> {
	return await invoke('execute_dataview_query', {
		queryText,
		vaultPaths,
	});
}

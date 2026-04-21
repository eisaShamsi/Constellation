import { invoke } from '@tauri-apps/api/core';
import type { DataviewResult } from './types';

/**
 * Execute a Dataview query against the given libraries.
 * @param queryText - DQL query string (e.g., 'TABLE title, date FROM "folder" WHERE status = "done"')
 * @param libraryPaths - Array of [libraryName, libraryPath] tuples
 */
export async function executeDataviewQuery(
	queryText: string,
	libraryPaths: [string, string][]
): Promise<DataviewResult> {
	return await invoke('execute_dataview_query', {
		queryText,
		libraryPaths,
	});
}

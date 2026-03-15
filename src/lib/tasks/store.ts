// ─── Constellation Tasks — IPC Store ───

import { invoke } from '@tauri-apps/api/core';
import type { TaskScanResult, NoteDateEntry } from './types';

/** Scan an entire vault for tasks. */
export async function scanVaultTasks(libraryPath: string, libraryName: string): Promise<TaskScanResult> {
	return await invoke('scan_vault_tasks', { libraryPath, libraryName });
}

/** Scan a single note for tasks. */
export async function scanNoteTasks(filePath: string, libraryName: string, libraryPath: string): Promise<TaskScanResult> {
	return await invoke('scan_note_tasks', { filePath, libraryName, libraryPath });
}

/** Toggle a task checkbox at a specific line. Returns updated file content. */
export async function toggleTask(filePath: string, lineNumber: number): Promise<string> {
	return await invoke('toggle_task', { filePath, lineNumber });
}

/** Scan vault for note dates (modified + frontmatter). Returns map of date -> entries. */
export async function scanVaultNoteDates(libraryPath: string, libraryName: string): Promise<Record<string, NoteDateEntry[]>> {
	return await invoke('scan_vault_note_dates', { libraryPath, libraryName });
}

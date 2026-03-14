// ─── Constellation Tasks — IPC Store ───

import { invoke } from '@tauri-apps/api/core';
import type { TaskScanResult, NoteDateEntry } from './types';

/** Scan an entire vault for tasks. */
export async function scanVaultTasks(vaultPath: string, vaultName: string): Promise<TaskScanResult> {
	return await invoke('scan_vault_tasks', { vaultPath, vaultName });
}

/** Scan a single note for tasks. */
export async function scanNoteTasks(filePath: string, vaultName: string, vaultPath: string): Promise<TaskScanResult> {
	return await invoke('scan_note_tasks', { filePath, vaultName, vaultPath });
}

/** Toggle a task checkbox at a specific line. Returns updated file content. */
export async function toggleTask(filePath: string, lineNumber: number): Promise<string> {
	return await invoke('toggle_task', { filePath, lineNumber });
}

/** Scan vault for note dates (modified + frontmatter). Returns map of date -> entries. */
export async function scanVaultNoteDates(vaultPath: string, vaultName: string): Promise<Record<string, NoteDateEntry[]>> {
	return await invoke('scan_vault_note_dates', { vaultPath, vaultName });
}

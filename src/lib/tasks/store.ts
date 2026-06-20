// ─── Constellation Tasks — IPC Store ───

import { invoke } from '@tauri-apps/api/core';
import type { TaskScanResult, NoteDateEntry } from './types';

/** Scan an entire library for tasks. */
export async function scanLibraryTasks(libraryPath: string, libraryName: string): Promise<TaskScanResult> {
	return await invoke('scan_library_tasks', { libraryPath, libraryName });
}

/** Scan a single note for tasks. */
export async function scanNoteTasks(filePath: string, libraryName: string, libraryPath: string): Promise<TaskScanResult> {
	return await invoke('scan_note_tasks', { filePath, libraryName, libraryPath });
}

/** Toggle a task checkbox at a specific line. Returns updated file content. */
export async function toggleTask(filePath: string, lineNumber: number): Promise<string> {
	return await invoke('toggle_task', { filePath, lineNumber });
}

/** Scan library for note dates (modified + frontmatter). Returns map of date -> entries.
 *  MIG-082 §A.1 — pass the daily-note format/folder so each entry's `is_daily` is correct. */
export async function scanLibraryNoteDates(libraryPath: string, libraryName: string, dailyFormat?: string, dailyFolder?: string): Promise<Record<string, NoteDateEntry[]>> {
	return await invoke('scan_library_note_dates', { libraryPath, libraryName, dailyFormat, dailyFolder });
}

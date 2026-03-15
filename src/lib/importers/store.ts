import { invoke } from '@tauri-apps/api/core';
import type { ImportPreview, ImportResult } from './types';

export async function importPickSource(format: string): Promise<string> {
	return invoke<string>('import_pick_source', { format });
}

export async function importPreview(source: string, format: string): Promise<ImportPreview> {
	return invoke<ImportPreview>('import_preview', { source, format });
}

export async function importExecute(
	source: string,
	format: string,
	targetLibrary: string,
	subfolder: string
): Promise<ImportResult> {
	return invoke<ImportResult>('import_execute', {
		source,
		format,
		targetLibrary,
		subfolder,
	});
}

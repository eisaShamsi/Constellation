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

// ─── Canonical Filename System ──────────────────────────────────────

export interface CanonicalName {
	timestamp: string;
	kind: string;
	suffix: string;
	extension: string;
	full: string;
	stem: string; // = cid
}

export interface CanonicalizeResult {
	total_files: number;
	renamed: number;
	sidecars_created: number;
	errors: string[];
	rename_map: Record<string, string>;
}

/** Classify a file by its content and extension. Returns kind code (e.g., "NOTE", "IMG"). */
export async function classifyFile(path: string): Promise<string> {
	return invoke<string>('classify_file_cmd', { path });
}

/** Generate a canonical filename for a new file. */
export async function generateCanonicalName(kind: string, created?: string): Promise<CanonicalName> {
	return invoke<CanonicalName>('generate_canonical_name', { kind, created });
}

/** Preview what canonicalization would do to a library (no changes made). */
export async function canonicalizePreview(libraryPath: string): Promise<CanonicalizeResult> {
	return invoke<CanonicalizeResult>('canonicalize_preview', { libraryPath });
}

/** Execute canonicalization: rename files, inject frontmatter, create sidecars. */
export async function canonicalizeExecute(libraryPath: string): Promise<CanonicalizeResult> {
	return invoke<CanonicalizeResult>('canonicalize_execute', { libraryPath });
}

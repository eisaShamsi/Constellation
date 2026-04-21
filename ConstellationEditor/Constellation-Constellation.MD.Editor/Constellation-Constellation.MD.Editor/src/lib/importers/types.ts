export interface ImportPreviewEntry {
	source_name: string;
	target_name: string;
	size_bytes: number;
}

export interface ImportPreview {
	file_count: number;
	format: string;
	files: ImportPreviewEntry[];
}

export interface ImportResult {
	imported: number;
	skipped: number;
	errors: string[];
	files: string[];
}

export type ImportFormat = 'obsidian' | 'markdown' | 'notion' | 'bear' | 'enex' | 'html' | 'csv' | 'txt';

export interface ImportFormatOption {
	id: ImportFormat;
	label: string;
	description: string;
	icon: string;
	pickType: 'folder' | string;
}

// ─── Constellation Bases — Type Definitions ───

export interface BaseSource {
	type: 'folder' | 'tag' | 'all';
	path?: string;
	tag?: string;
	includeSubfolders?: boolean;
	selectedLibraries?: string[]; // empty/undefined = all libraries; populated = only these library names
}

export interface ColumnDef {
	property: string;
	label?: string;
	width?: number;
	visible?: boolean;
	direction?: 'ltr' | 'rtl';
}

export interface FilterRule {
	property: string;
	operator: 'is' | 'is_not' | 'contains' | 'not_contains' | 'gt' | 'lt' | 'is_empty' | 'is_not_empty';
	value?: string;
}

export interface SortRule {
	property: string;
	direction: 'asc' | 'desc';
}

export interface BaseDefinition {
	version: number;
	name: string;
	source: BaseSource;
	columns: ColumnDef[];
	filters: FilterRule[];
	sorts: SortRule[];
	view: 'table' | 'card' | 'list';
	direction: 'auto' | 'rtl' | 'ltr';
}

export interface BaseRow {
	file_path: string;
	file_name: string;
	library_name: string;
	library_path: string;
	properties: Record<string, string>;
	modified: number;
}

export interface BaseQueryResult {
	rows: BaseRow[];
	total_count: number;
	query_time_ms: number;
	columns_detected: string[];
}

// ─── Defaults ───

export function createDefaultBase(name: string, folderPath: string): BaseDefinition {
	return {
		version: 1,
		name,
		source: {
			type: 'folder',
			path: folderPath,
			includeSubfolders: true,
		},
		columns: [],
		filters: [],
		sorts: [],
		view: 'table',
		direction: 'auto',
	};
}

export function createDefaultColumn(property: string): ColumnDef {
	return {
		property,
		width: 150,
		visible: true,
	};
}

// ─── Property type detection (mirrors store.ts logic) ───

export type PropertyType = 'text' | 'number' | 'date' | 'datetime' | 'checkbox' | 'list' | 'link';

export function detectCellType(key: string, value: string): PropertyType {
	if (!value) return 'text';

	const keyLower = key.toLowerCase();

	// Checkbox
	if (value === 'true' || value === 'false') return 'checkbox';

	// Link
	if (value.startsWith('[[') && value.endsWith(']]')) return 'link';

	// List (comma-separated with 2+ items)
	if (value.includes(',') && value.split(',').length >= 2) return 'list';

	// Date patterns
	if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return 'date';
	if (/^\d{4}-\d{2}-\d{2}T/.test(value)) return 'datetime';

	// Number
	if (/^-?\d+(\.\d+)?$/.test(value)) return 'number';

	// Date-like key names
	const dateKeys = ['date', 'created', 'modified', 'updated', 'due', 'published', 'تاريخ'];
	if (dateKeys.some(dk => keyLower.includes(dk))) {
		if (/\d{4}/.test(value)) return 'date';
	}

	return 'text';
}

// ─── Constellation Bases — Workspace base types ───
//
// MIG-065 §I — trimmed to what the SIDEBAR's workspace-base management still
// needs (`BaseDefinition` for create/save). The old MVP's row / cell / query
// types + cell-type detection went out with `query_base` and the `BaseView`
// family. §I-b retires `BaseDefinition` itself once base creation writes a
// `LensDefinition` YAML directly.

export interface BaseSource {
	type: 'folder' | 'tag' | 'all';
	path?: string;
	tag?: string;
	includeSubfolders?: boolean;
	selectedLibraries?: string[]; // empty/undefined = all libraries; populated = only these names
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

// ─── Defaults ───

export function createDefaultBase(name: string, folderPath: string): BaseDefinition {
	return {
		version: 1,
		name,
		source: { type: 'folder', path: folderPath, includeSubfolders: true },
		columns: [],
		filters: [],
		sorts: [],
		view: 'table',
		direction: 'auto',
	};
}

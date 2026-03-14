/** Dataview query result from Rust backend */
export interface DataviewResult {
	query_type: 'table' | 'list' | 'task' | 'calendar' | 'error';
	rows: DataviewRow[];
	columns: string[];
	total_count: number;
	query_time_ms: number;
	group_by: string | null;
	error: string | null;
}

/** A single row in a dataview result (matches BaseRow from Rust) */
export interface DataviewRow {
	file_path: string;
	file_name: string;
	vault_name: string;
	vault_path: string;
	properties: Record<string, string>;
	modified: number;
}

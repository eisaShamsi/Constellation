// ─── Constellation Tasks — Type Definitions ───

export interface TaskItem {
	text: string;
	completed: boolean;
	file_path: string;
	file_name: string;
	vault_name: string;
	vault_path: string;
	line_number: number;
	due_date: string | null;
	priority: 'high' | 'medium' | 'low' | null;
	tags: string[];
	created_date: string | null;
	done_date: string | null;
}

export interface TaskScanResult {
	tasks: TaskItem[];
	total_count: number;
	scan_time_ms: number;
}

export interface NoteDateEntry {
	file_path: string;
	file_name: string;
	date: string;
	date_source: string;
}

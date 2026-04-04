/**
 * Tag utilities — shared tag scanning and merging logic.
 * One source of truth for dashboard tag aggregation.
 */
import { get } from 'svelte/store';
import { libraries, scanLibraryTags } from './store';

export interface DashboardTag {
	tag: string;
	count: number;
}

/** Scan all libraries and merge tags with counts, sorted by count descending */
export async function scanAllLibraryTags(): Promise<DashboardTag[]> {
	const merged: Record<string, number> = {};
	for (const lib of get(libraries)) {
		try {
			const tags = await scanLibraryTags(lib.path);
			for (const [tag, count] of Object.entries(tags)) {
				merged[tag] = (merged[tag] || 0) + count;
			}
		} catch {}
	}
	return Object.entries(merged)
		.map(([tag, count]) => ({ tag, count }))
		.sort((a, b) => b.count - a.count);
}

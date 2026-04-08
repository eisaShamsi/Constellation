/**
 * Search history — localStorage read/write for recent search queries.
 * Follows the same pattern as recentNotes.ts.
 */

export interface SearchHistoryEntry {
	query: string;
	timestamp: number;
}

const KEY = 'constellation-search-history';
const MAX_ENTRIES = 20;

export function readSearchHistory(): SearchHistoryEntry[] {
	try { return JSON.parse(localStorage.getItem(KEY) || '[]'); }
	catch { return []; }
}

export function addSearchHistory(query: string): void {
	const trimmed = query.trim();
	if (!trimmed) return;
	try {
		const existing = readSearchHistory().filter(e => e.query !== trimmed);
		existing.unshift({ query: trimmed, timestamp: Date.now() });
		localStorage.setItem(KEY, JSON.stringify(existing.slice(0, MAX_ENTRIES)));
	} catch {}
}

export function clearSearchHistory(): void {
	try { localStorage.removeItem(KEY); }
	catch {}
}

/** Human-readable relative time for history entries. */
export function relativeTime(ts: number, now: number = Date.now()): string {
	const diff = now - ts;
	const mins = Math.floor(diff / 60000);
	if (mins < 1) return '<1m';
	if (mins < 60) return `${mins}m`;
	const hours = Math.floor(mins / 60);
	if (hours < 24) return `${hours}h`;
	const days = Math.floor(hours / 24);
	if (days < 7) return `${days}d`;
	const weeks = Math.floor(days / 7);
	return `${weeks}w`;
}

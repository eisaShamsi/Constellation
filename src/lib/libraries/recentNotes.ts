/**
 * Recent notes — shared localStorage read/write for recently opened/edited notes.
 * One source of truth. Used by main window, second screen, and dashboard.
 */

export interface RecentOpenedNote {
	name: string;
	path: string;
	libraryName: string;
	openedAt: number;
}

export interface RecentEditedNote {
	name: string;
	path: string;
	libraryName: string;
	editedAt: number;
}

const KEY_OPENED = 'constellation-recent-opened';
const KEY_EDITED = 'constellation-recent-edited';
const MAX_RECENT = 20;

export function readRecentOpened(): RecentOpenedNote[] {
	try { return JSON.parse(localStorage.getItem(KEY_OPENED) || '[]'); }
	catch { return []; }
}

export function readRecentEdited(): RecentEditedNote[] {
	try { return JSON.parse(localStorage.getItem(KEY_EDITED) || '[]'); }
	catch { return []; }
}

export function addRecentOpened(note: { name: string; path: string; libraryName: string }) {
	try {
		const existing = readRecentOpened().filter(n => n.path !== note.path);
		existing.unshift({ ...note, openedAt: Date.now() });
		localStorage.setItem(KEY_OPENED, JSON.stringify(existing.slice(0, MAX_RECENT)));
	} catch {}
}

export function addRecentEdited(note: { name: string; path: string; libraryName: string }) {
	try {
		const existing = readRecentEdited().filter(n => n.path !== note.path);
		existing.unshift({ ...note, editedAt: Date.now() });
		localStorage.setItem(KEY_EDITED, JSON.stringify(existing.slice(0, MAX_RECENT)));
	} catch {}
}

/**
 * Get display-ready recent lists:
 * - recentlyEdited: last 10 edited
 * - recentlyOpened: last 10 opened but NOT in edited list (no duplicates)
 */
export function getRecentLists(): {
	recentlyEdited: RecentEditedNote[];
	recentlyOpened: RecentOpenedNote[];
} {
	const edited = readRecentEdited().slice(0, 10);
	const editedPaths = new Set(edited.map(n => n.path));
	const opened = readRecentOpened().filter(n => !editedPaths.has(n.path)).slice(0, 10);
	return { recentlyEdited: edited, recentlyOpened: opened };
}

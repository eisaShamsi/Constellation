/**
 * MIG-092 — pure reducers for Collections membership.
 *
 * Zero Tauri / Svelte dependencies so the membership logic is unit-testable in
 * isolation (the tests/mig-090/chips.test.ts pattern). `store.ts` wraps these
 * with a `writable` + save-on-change persistence; every function here is a pure
 * transform of an immutable `Collection[]` (callers pass `now` so there is no
 * hidden clock — deterministic tests, and safe for resume/replay).
 *
 * Collections are the ONE hand-picked-set mechanism. `starred` is a pinned,
 * undeletable collection (the former Bookmarks). Note members are keyed by
 * cid (self-upgraded at hydration) or path; folder / saved-search members
 * (unified from Bookmarks) carry inline facts and are never hydrated.
 */

export type CollectionItemType = 'note' | 'folder' | 'search';

export interface CollectionItem {
	/** Membership kind. Absent ≡ 'note' (back-compat with MIG-090 items). */
	type?: CollectionItemType;
	/** The stable canonical id once known (note members) — survives renames/moves. */
	cid?: string;
	/** Display path; the membership key while a note lacks a cid, and the key for folder/search members. */
	path: string;
	/** Inline label for folder/search members (no note_meta row to hydrate). */
	name?: string;
	/** Inline library for folder/search members. */
	libraryName?: string;
	addedAt: number;
	done?: boolean;
}
export interface Collection {
	id: string;
	name: string;
	created: number;
	items: CollectionItem[];
}

/** The pinned, undeletable "Starred" collection (the former Bookmarks). */
export const STARRED_ID = 'starred';
export const COLLECTION_ITEM_CAP = 100;

/** The item's membership identity: cid when known (notes), else its path. */
export function collectionKey(item: CollectionItem): string {
	return item.cid || item.path;
}

/** Guarantee a pinned Starred collection at the head of the list. */
export function ensureStarred(list: Collection[], now: number): Collection[] {
	if (list.some(c => c.id === STARRED_ID)) return list;
	return [{ id: STARRED_ID, name: 'Starred', created: now, items: [] }, ...list];
}

export function createSet(list: Collection[], id: string, name: string, now: number): Collection[] {
	return [...ensureStarred(list, now), { id, name: name.trim() || 'Untitled', created: now, items: [] }];
}

export function renameSet(list: Collection[], id: string, name: string): Collection[] {
	return list.map(c => (c.id === id ? { ...c, name: name.trim() || c.name } : c));
}

/** Delete a collection. The pinned Starred collection is never deletable. */
export function deleteSet(list: Collection[], id: string): Collection[] {
	if (id === STARRED_ID) return list;
	return list.filter(c => c.id !== id);
}

/** Add an item to a set. Dedupes by (type, path); capped at COLLECTION_ITEM_CAP. */
export function addItem(
	list: Collection[],
	setId: string,
	item: { type?: CollectionItemType; path: string; name?: string; libraryName?: string },
	now: number
): { list: Collection[]; added: boolean } {
	const sets = ensureStarred(list, now);
	const set = sets.find(c => c.id === setId);
	if (!set) return { list: sets, added: false };
	const type = item.type ?? 'note';
	if (set.items.some(i => i.path === item.path && (i.type ?? 'note') === type)) return { list: sets, added: false };
	if (set.items.length >= COLLECTION_ITEM_CAP) return { list: sets, added: false };
	const nextItem: CollectionItem = {
		type: item.type,
		path: item.path,
		name: item.name,
		libraryName: item.libraryName,
		addedAt: now,
	};
	return {
		list: sets.map(c => (c.id === setId ? { ...c, items: [...c.items, nextItem] } : c)),
		added: true,
	};
}

export function removeItem(list: Collection[], setId: string, key: string): Collection[] {
	return list.map(c => (c.id === setId ? { ...c, items: c.items.filter(i => collectionKey(i) !== key) } : c));
}

export function toggleDone(list: Collection[], setId: string, key: string): Collection[] {
	return list.map(c =>
		c.id === setId
			? { ...c, items: c.items.map(i => (collectionKey(i) === key ? { ...i, done: !i.done } : i)) }
			: c
	);
}

export function sweepDone(list: Collection[], setId: string): Collection[] {
	return list.map(c => (c.id === setId ? { ...c, items: c.items.filter(i => !i.done) } : c));
}

/** Adopt cids for path-keyed NOTE items (self-upgrade to rename-proof identity)
 *  and refresh display paths for cid-keyed items whose note moved. */
export function adoptIdentities(
	list: Collection[],
	rows: { key: string; path: string; cid_cn: string }[]
): { list: Collection[]; changed: boolean } {
	let changed = false;
	const next = list.map(set => ({
		...set,
		items: set.items.map(i => {
			const row = rows.find(r => r.key === collectionKey(i));
			if (!row) return i;
			const cid = row.cid_cn && row.cid_cn !== '' ? row.cid_cn : i.cid;
			if (i.cid !== cid || i.path !== row.path) {
				changed = true;
				return { ...i, cid, path: row.path };
			}
			return i;
		}),
	}));
	return { list: changed ? next : list, changed };
}

/** Rename hook — path-keyed membership follows in-app renames/moves. */
export function migratePath(
	list: Collection[],
	oldPath: string,
	newPath: string
): { list: Collection[]; changed: boolean } {
	let changed = false;
	const next = list.map(set => ({
		...set,
		items: set.items.map(i => {
			if (i.path === oldPath) {
				changed = true;
				return { ...i, path: newPath };
			}
			return i;
		}),
	}));
	return { list: changed ? next : list, changed };
}

/** The one-time Bookmarks→Starred migration mapping. Idempotent: when a Starred
 *  collection already exists the list is returned unchanged (migrated=false),
 *  so the seed can never overwrite user edits on a subsequent load. Preserves
 *  each bookmark's type/path/name/library (folders + saved searches included). */
export function migrateBookmarks(
	list: Collection[],
	bookmarks: Array<{ type?: CollectionItemType; path: string; name?: string; libraryName?: string }>,
	now: number
): { list: Collection[]; migrated: boolean } {
	if (list.some(c => c.id === STARRED_ID)) return { list, migrated: false };
	const items: CollectionItem[] = bookmarks.map(bm => ({
		type: bm.type ?? 'note',
		path: bm.path,
		name: bm.name,
		libraryName: bm.libraryName,
		addedAt: now,
	}));
	return { list: [{ id: STARRED_ID, name: 'Starred', created: now, items }, ...list], migrated: true };
}

// ── Mixed-member hydration (§3) ──
// Notes carry live facts (re-read from the index); folder / saved-search
// members carry inline facts and are never hydrated. These pure helpers split
// the note keys for the `collections_hydrate` command and merge the returned
// rows back into ordered display rows.

/** The note-fact row returned by the `collections_hydrate` Tauri command. */
export interface HydratedNoteRow {
	key: string;
	path: string;
	cid_cn: string;
	name: string;
	library_name: string;
	modified: number;
	word_count: number;
	stage: string | null;
	incoming_count: number;
	outgoing_count: number;
	incoming_link_types_json: string;
	outgoing_link_types_json: string;
	review_reason: string | null;
	review_due: boolean;
	snoozed: boolean;
}

/** One ordered row for display: a note (live or missing) or an inline folder/search. */
export interface CollectionDisplayRow {
	key: string;
	item: CollectionItem;
	type: CollectionItemType;
	name: string;
	libraryName: string;
	/** A note member with no hydrated row (deleted externally / universe detached). */
	missing: boolean;
	hydrated: HydratedNoteRow | null;
}

/** Split ONLY note members into hydration keys (cid preferred, else path).
 *  Folder/search members are excluded — they have no note_meta row. */
export function noteHydrationKeys(items: CollectionItem[]): { cids: string[]; paths: string[] } {
	const cids: string[] = [];
	const paths: string[] = [];
	for (const i of items) {
		if ((i.type ?? 'note') !== 'note') continue;
		if (i.cid) cids.push(i.cid);
		else paths.push(i.path);
	}
	return { cids, paths };
}

/** Merge membership items + hydrated note rows into ordered display rows.
 *  Note members resolve to live facts (or `missing` when absent); folder/search
 *  members render from their inline stored facts. Preserves item order. */
export function buildDisplayRows(items: CollectionItem[], rows: HydratedNoteRow[]): CollectionDisplayRow[] {
	const byKey = new Map(rows.map(r => [r.key, r]));
	return items.map(item => {
		const key = collectionKey(item);
		const type = item.type ?? 'note';
		if (type !== 'note') {
			return {
				key,
				item,
				type,
				name: item.name ?? item.path,
				libraryName: item.libraryName ?? '',
				missing: false,
				hydrated: null,
			};
		}
		const hydrated = byKey.get(key) ?? null;
		return {
			key,
			item,
			type,
			name: hydrated?.name ?? item.name ?? item.path,
			libraryName: hydrated?.library_name ?? item.libraryName ?? '',
			missing: !hydrated,
			hydrated,
		};
	});
}

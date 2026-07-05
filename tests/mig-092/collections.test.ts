/**
 * MIG-092 §2 — Collections membership reducers (pure, deterministic).
 *
 * Guards the invariants the migration rests on:
 *  - Bookmarks → Starred preserves note/folder/search targets, and the seed is
 *    idempotent (never overwrites an existing Starred on a later load).
 *  - Starred is pinned (undeletable).
 *  - Named sets add/remove/rename/delete independently; adds dedupe by
 *    (type,path) and respect the item cap.
 *  - Note identity self-upgrades to cid (rename-proof); renames follow paths.
 */
import { describe, it, expect } from 'vitest';
import {
	type Collection,
	STARRED_ID,
	COLLECTION_ITEM_CAP,
	collectionKey,
	ensureStarred,
	createSet,
	renameSet,
	deleteSet,
	addItem,
	removeItem,
	toggleDone,
	sweepDone,
	adoptIdentities,
	migratePath,
	migrateBookmarks,
	noteHydrationKeys,
	buildDisplayRows,
	type HydratedNoteRow,
} from '$lib/libraries/collectionsLogic';

const hydrated = (over: Partial<HydratedNoteRow> & { key: string }): HydratedNoteRow => ({
	path: over.key,
	cid_cn: '',
	name: over.key,
	library_name: 'Lib',
	modified: 0,
	word_count: 0,
	stage: null,
	incoming_count: 0,
	outgoing_count: 0,
	incoming_link_types_json: '{}',
	outgoing_link_types_json: '{}',
	review_reason: null,
	review_due: false,
	snoozed: false,
	...over,
});

const NOW = 1_700_000_000_000;

describe('MIG-092 §2 — Bookmarks → Starred migration', () => {
	it('seeds Starred with note/folder/search targets preserved', () => {
		const bms = [
			{ type: 'note' as const, path: 'a.md', name: 'A', libraryName: 'Lib' },
			{ type: 'folder' as const, path: 'sub', name: 'sub', libraryName: 'Lib' },
			{ type: 'search' as const, path: '#tag topic', name: '#tag topic', libraryName: '' },
		];
		const { list, migrated } = migrateBookmarks([], bms, NOW);
		expect(migrated).toBe(true);
		const starred = list.find(c => c.id === STARRED_ID)!;
		expect(starred.items).toHaveLength(3);
		expect(starred.items.map(i => i.type)).toEqual(['note', 'folder', 'search']);
		expect(starred.items[1]).toMatchObject({ path: 'sub', name: 'sub', type: 'folder' });
		expect(starred.items[2]).toMatchObject({ path: '#tag topic', type: 'search' });
	});

	it('bookmark without an explicit type defaults to note', () => {
		const { list } = migrateBookmarks([], [{ path: 'x.md', name: 'X' }], NOW);
		expect(list.find(c => c.id === STARRED_ID)!.items[0].type).toBe('note');
	});

	it('is idempotent — a second run never overwrites an existing Starred', () => {
		const first = migrateBookmarks([], [{ path: 'a.md', name: 'A' }], NOW);
		const second = migrateBookmarks(first.list, [{ path: 'b.md', name: 'B' }], NOW + 1);
		expect(second.migrated).toBe(false);
		expect(second.list).toBe(first.list); // untouched reference
		expect(second.list.find(c => c.id === STARRED_ID)!.items.map(i => i.path)).toEqual(['a.md']);
	});

	it('an empty bookmark set still creates an empty Starred', () => {
		const { list, migrated } = migrateBookmarks([], [], NOW);
		expect(migrated).toBe(true);
		expect(list.find(c => c.id === STARRED_ID)!.items).toHaveLength(0);
	});
});

describe('MIG-092 §2 — Starred is pinned', () => {
	it('cannot be deleted', () => {
		const list = ensureStarred([], NOW);
		expect(deleteSet(list, STARRED_ID)).toBe(list); // no-op, same reference
		expect(deleteSet(list, STARRED_ID).some(c => c.id === STARRED_ID)).toBe(true);
	});

	it('ensureStarred is a no-op when Starred already exists', () => {
		const list = ensureStarred([], NOW);
		expect(ensureStarred(list, NOW + 5)).toBe(list);
	});
});

describe('MIG-092 §2 — named sets add/remove/rename/delete independently', () => {
	it('creates a set, adds to it, and leaves the other set untouched', () => {
		let list: Collection[] = ensureStarred([], NOW);
		list = createSet(list, 'A', 'Task A', NOW);
		list = createSet(list, 'B', 'Task B', NOW);
		list = addItem(list, 'A', { path: 'n1.md', name: 'N1' }, NOW).list;
		list = addItem(list, 'A', { path: 'n2.md', name: 'N2' }, NOW).list;
		list = addItem(list, 'B', { path: 'n3.md', name: 'N3' }, NOW).list;
		expect(list.find(c => c.id === 'A')!.items.map(i => i.path)).toEqual(['n1.md', 'n2.md']);
		expect(list.find(c => c.id === 'B')!.items.map(i => i.path)).toEqual(['n3.md']);
	});

	it('dedupes by (type,path) but allows the same path with a different type', () => {
		let list = createSet(ensureStarred([], NOW), 'A', 'A', NOW);
		const r1 = addItem(list, 'A', { path: 'dup.md' }, NOW);
		expect(r1.added).toBe(true);
		list = r1.list;
		const r2 = addItem(list, 'A', { path: 'dup.md' }, NOW);
		expect(r2.added).toBe(false); // same note, refused
		list = r2.list;
		const r3 = addItem(list, 'A', { type: 'folder', path: 'dup.md' }, NOW);
		expect(r3.added).toBe(true); // a folder at the same path is a distinct member
		expect(r3.list.find(c => c.id === 'A')!.items).toHaveLength(2);
	});

	it('respects the item cap', () => {
		let list = createSet(ensureStarred([], NOW), 'A', 'A', NOW);
		for (let i = 0; i < COLLECTION_ITEM_CAP; i++) list = addItem(list, 'A', { path: `n${i}.md` }, NOW).list;
		const over = addItem(list, 'A', { path: 'one-too-many.md' }, NOW);
		expect(over.added).toBe(false);
		expect(over.list.find(c => c.id === 'A')!.items).toHaveLength(COLLECTION_ITEM_CAP);
	});

	it('rename and delete a named set (Starred stays)', () => {
		let list = createSet(ensureStarred([], NOW), 'A', 'Task A', NOW);
		list = renameSet(list, 'A', 'Renamed');
		expect(list.find(c => c.id === 'A')!.name).toBe('Renamed');
		list = deleteSet(list, 'A');
		expect(list.some(c => c.id === 'A')).toBe(false);
		expect(list.some(c => c.id === STARRED_ID)).toBe(true);
	});

	it('remove + done + sweep operate per-set', () => {
		let list = createSet(ensureStarred([], NOW), 'A', 'A', NOW);
		list = addItem(list, 'A', { path: 'keep.md' }, NOW).list;
		list = addItem(list, 'A', { path: 'drop.md' }, NOW).list;
		list = toggleDone(list, 'A', 'drop.md');
		expect(list.find(c => c.id === 'A')!.items.find(i => i.path === 'drop.md')!.done).toBe(true);
		list = sweepDone(list, 'A');
		expect(list.find(c => c.id === 'A')!.items.map(i => i.path)).toEqual(['keep.md']);
		list = removeItem(list, 'A', 'keep.md');
		expect(list.find(c => c.id === 'A')!.items).toHaveLength(0);
	});
});

describe('MIG-092 §2 — identity: rename-proof notes, inline folders', () => {
	it('collectionKey uses cid for notes, path for typeless/folder members', () => {
		expect(collectionKey({ cid: 'CID1', path: 'a.md', addedAt: NOW })).toBe('CID1');
		expect(collectionKey({ path: 'sub', type: 'folder', addedAt: NOW })).toBe('sub');
	});

	it('adoptIdentities upgrades a path-keyed note to its cid', () => {
		let list = createSet(ensureStarred([], NOW), 'A', 'A', NOW);
		list = addItem(list, 'A', { path: 'a.md' }, NOW).list;
		const { list: next, changed } = adoptIdentities(list, [{ key: 'a.md', path: 'a.md', cid_cn: 'CID_A' }]);
		expect(changed).toBe(true);
		expect(next.find(c => c.id === 'A')!.items[0].cid).toBe('CID_A');
	});

	it('migratePath follows an in-app rename', () => {
		let list = createSet(ensureStarred([], NOW), 'A', 'A', NOW);
		list = addItem(list, 'A', { path: 'old.md' }, NOW).list;
		const { list: next, changed } = migratePath(list, 'old.md', 'new.md');
		expect(changed).toBe(true);
		expect(next.find(c => c.id === 'A')!.items[0].path).toBe('new.md');
	});
});

describe('MIG-092 §3 — mixed-member hydration (notes live, folder/search inline)', () => {
	const items = [
		{ cid: 'CID_A', path: 'a.md', type: 'note' as const, addedAt: NOW },
		{ path: 'b.md', type: 'note' as const, addedAt: NOW }, // no cid yet → path key
		{ path: 'sub', type: 'folder' as const, name: 'sub', libraryName: 'Lib', addedAt: NOW },
		{ path: '#tag topic', type: 'search' as const, name: '#tag topic', addedAt: NOW },
	];

	it('noteHydrationKeys sends only note members (cid preferred, else path)', () => {
		const { cids, paths } = noteHydrationKeys(items);
		expect(cids).toEqual(['CID_A']);
		expect(paths).toEqual(['b.md']); // folder/search excluded
	});

	it('buildDisplayRows merges live note facts, flags missing, and inlines folder/search — order preserved', () => {
		const rows = [hydrated({ key: 'CID_A', name: 'Alpha', library_name: 'L1', stage: 'growth' })];
		const display = buildDisplayRows(items, rows);
		expect(display.map(d => d.type)).toEqual(['note', 'note', 'folder', 'search']);
		// note a: hydrated live facts
		expect(display[0]).toMatchObject({ name: 'Alpha', libraryName: 'L1', missing: false });
		expect(display[0].hydrated?.stage).toBe('growth');
		// note b: no row returned → missing, falls back to inline path
		expect(display[1]).toMatchObject({ missing: true, name: 'b.md', hydrated: null });
		// folder + search: inline name, never missing, never hydrated
		expect(display[2]).toMatchObject({ name: 'sub', libraryName: 'Lib', missing: false, hydrated: null });
		expect(display[3]).toMatchObject({ name: '#tag topic', missing: false, hydrated: null });
	});
});

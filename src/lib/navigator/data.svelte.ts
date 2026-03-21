/**
 * Shared data layer for the Unified Navigator.
 * Loads libraries, child universes, folder trees, tags, and notes once,
 * shared across Tree, List, and Sky View modes.
 */

import { invoke } from '@tauri-apps/api/core';
import { libraries, collectLibraryNotesWithMeta, type FileEntry, type NoteWithMeta } from '$lib/libraries/store';
import { getChildUniverses, type ChildUniverseInfo } from '$lib/universe/store';
import { get } from 'svelte/store';

// ─── TreeNode type (used by Sky View) ────────────────────
export type TreeNode = {
	name: string;
	path: string;
	isDir: boolean;
	isLibrary?: boolean;
	isRoot?: boolean;
	isCUniverse?: boolean;
	isNote?: boolean;
	libraryName?: string;
	libraryPath?: string;
	color?: string;
	status?: string;
	children?: TreeNode[];
	expanded?: boolean;
	noteCount?: number;
};

// ─── Navigator Data State ────────────────────────────────
export interface NavigatorData {
	loading: boolean;
	childUniverses: ChildUniverseInfo[];
	childUniverseLibPaths: Map<string, Set<string>>;
	folderTrees: FileEntry[];
	allNotesWithMeta: NoteWithMeta[];
	tagMap: Record<string, number>;
	skyViewRoot: TreeNode | null;
}

export function normalizePath(p: string): string {
	return p.replace(/\\/g, '/').toLowerCase();
}

function buildTreeNodes(
	entries: FileEntry[],
	libraryName: string,
	libraryPath: string,
	libraryColorMap: Record<string, string>,
): TreeNode[] {
	return entries.map((e) => {
		const node: TreeNode = {
			name: e.name.replace(/\.md$/, ''),
			path: e.path,
			isDir: e.is_dir,
			isNote: !e.is_dir,
			libraryName,
			libraryPath,
			color: libraryColorMap[libraryName] || '#7c3aed',
			status: e.status ?? undefined,
			children:
				e.is_dir && e.children
					? buildTreeNodes(e.children, libraryName, libraryPath, libraryColorMap)
					: undefined,
			expanded: false,
		};
		if (e.is_dir) node.noteCount = countNotes(node);
		return node;
	});
}

export function countNotes(node: TreeNode): number {
	if (!node.children) return 0;
	let n = 0;
	for (const c of node.children) {
		if (c.isNote) n++;
		else n += countNotes(c);
	}
	return n;
}

/**
 * Load all navigator data in one pass.
 */
export async function loadNavigatorData(
	libraryColorMap: Record<string, string>,
	universeName: string,
): Promise<NavigatorData> {
	const libs = get(libraries);
	if (libs.length === 0) {
		return {
			loading: false,
			childUniverses: [],
			childUniverseLibPaths: new Map(),
			folderTrees: [],
			allNotesWithMeta: [],
			tagMap: {},
			skyViewRoot: null,
		};
	}

	// 1. Child universes
	let childUniverses: ChildUniverseInfo[] = [];
	try {
		childUniverses = await getChildUniverses();
	} catch {
		/* no child universes */
	}

	// 2. Resolve child universe library paths
	const childUniverseLibPaths = new Map<string, Set<string>>();
	for (const cu of childUniverses) {
		try {
			const childLibs = await invoke<{ id: string; name: string; path: string }[]>(
				'read_child_universe_libraries',
				{ childPath: cu.path },
			).catch(() => []);
			childUniverseLibPaths.set(cu.path, new Set(childLibs.map((l) => normalizePath(l.path))));
		} catch {
			childUniverseLibPaths.set(cu.path, new Set());
		}
	}

	// 3. Load per-library data
	const allNotes: NoteWithMeta[] = [];
	const allTags: Record<string, number> = {};
	const libTreeMap = new Map<string, FileEntry>();
	const libTreeNodeMap = new Map<string, TreeNode>();

	for (const lib of libs) {
		// Notes
		const notes = await collectLibraryNotesWithMeta(lib.path).catch(() => []);
		for (const n of notes) n.libraryName = lib.name;
		allNotes.push(...notes);

		// Tags
		const libTags = await invoke<Record<string, number>>('scan_library_tags', {
			libraryPath: lib.path,
		}).catch(() => ({}));
		for (const [tag, count] of Object.entries(libTags)) {
			allTags[tag] = (allTags[tag] || 0) + count;
		}

		// Folder tree (FileEntry format for List mode browser)
		const tree = await invoke<FileEntry[]>('read_library_tree', {
			libraryPath: lib.path,
			maxDepth: 10,
		}).catch(() => []);
		libTreeMap.set(normalizePath(lib.path), {
			name: lib.name,
			path: lib.path,
			is_dir: true,
			children: tree,
		} as FileEntry);

		// TreeNode format for Sky View
		const treeNodes = buildTreeNodes(tree, lib.name, lib.path, libraryColorMap);
		const libNode: TreeNode = {
			name: lib.name,
			path: lib.path,
			isDir: true,
			isLibrary: true,
			libraryName: lib.name,
			libraryPath: lib.path,
			color: libraryColorMap[lib.name] || '#7c3aed',
			children: treeNodes,
			expanded: false,
		};
		libNode.noteCount = countNotes(libNode);
		libTreeNodeMap.set(normalizePath(lib.path), libNode);
	}

	// 4. Group into cUniverse vs own
	const assignedPaths = new Set<string>();

	// Build folderTrees (FileEntry[]) for List mode browser
	const folderTrees: FileEntry[] = [];
	// Build skyViewChildren (TreeNode[]) for Sky View
	const skyViewChildren: TreeNode[] = [];

	for (const cu of childUniverses) {
		const paths = childUniverseLibPaths.get(cu.path) || new Set();

		// FileEntry children
		const cuFolderChildren: FileEntry[] = [];
		// TreeNode children
		const cuTreeChildren: TreeNode[] = [];

		for (const p of paths) {
			const feEntry = libTreeMap.get(p);
			if (feEntry) {
				cuFolderChildren.push(feEntry);
				assignedPaths.add(p);
			}
			const tnEntry = libTreeNodeMap.get(p);
			if (tnEntry) cuTreeChildren.push(tnEntry);
		}

		if (cuFolderChildren.length > 0) {
			folderTrees.push({
				name: cu.name,
				path: cu.path,
				is_dir: true,
				children: cuFolderChildren,
				isCUniverse: true,
			} as FileEntry);
		}

		skyViewChildren.push({
			name: cu.name,
			path: cu.path,
			isDir: true,
			isCUniverse: true,
			color: '#6366f1',
			children: cuTreeChildren,
			expanded: false,
			noteCount: cuTreeChildren.reduce((s, l) => s + (l.noteCount || 0), 0),
		});
	}

	// Own libraries
	for (const lib of libs) {
		const np = normalizePath(lib.path);
		if (!assignedPaths.has(np)) {
			const feEntry = libTreeMap.get(np);
			if (feEntry) folderTrees.push(feEntry);

			const tnEntry = libTreeNodeMap.get(np);
			if (tnEntry) skyViewChildren.push(tnEntry);
		}
	}

	// Sky View root
	const skyViewRoot: TreeNode = {
		name: universeName || 'Universe',
		path: '__root__',
		isDir: true,
		isRoot: true,
		children: skyViewChildren,
		expanded: true,
		noteCount: skyViewChildren.reduce((s, c) => s + (c.noteCount || 0), 0),
	};

	return {
		loading: false,
		childUniverses,
		childUniverseLibPaths,
		folderTrees,
		allNotesWithMeta: allNotes,
		tagMap: allTags,
		skyViewRoot,
	};
}

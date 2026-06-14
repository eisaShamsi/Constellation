<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import { libraries, appSettings, deleteWithSetting, collectLibraryNotesWithMeta, type NoteWithMeta, type FileEntry } from '$lib/libraries/store';
	import { getChildUniverses, type ChildUniverseInfo } from '$lib/universe/store';
	import NavBrowserPane from './navigator/NavBrowserPane.svelte';
	import NavFileList from './navigator/NavFileList.svelte';
	import NavBatchBar from './navigator/NavBatchBar.svelte';

	let {
		mode = 'main' as 'main' | 'second',
		libraryColorMap = {} as Record<string, string>,
		/** Pre-computed tags from the boot snapshot (SQLite-backed, write-time derived).
		 *  When non-empty, skips the expensive scan_library_tags filesystem scan on mount.
		 *  The boot snapshot is populated before first paint and kept current via note-save hooks. */
		initialTags = {} as Record<string, number>,
		onNoteClick,
		onNoteDoubleClick,
		onNotePreview,
		onFolderSelect,
	}: {
		mode?: 'main' | 'second';
		libraryColorMap?: Record<string, string>;
		initialTags?: Record<string, number>;
		onNoteClick?: (path: string, name: string, libraryName: string) => void;
		onNoteDoubleClick?: (path: string, name: string, libraryName: string) => void;
		onNotePreview?: (path: string, name: string, libraryName: string) => void;
		onFolderSelect?: (path: string | string[] | null) => void;
	} = $props();

	// Data
	let allNotesWithMeta: NoteWithMeta[] = $state([]);
	let folderTrees: FileEntry[] = $state([]);
	let tagMap: Record<string, number> = $state({});
	// MIG-062 §A: re-sync tagMap when the federated `initialTags` prop updates.
	// The federation:ready re-fetch in +layout.svelte refreshes allLibraryTags
	// → the initialTags prop. Without this, the tag browser stayed frozen at
	// the mount-time (parent-only) snapshot — federated cUniverse tags never
	// appeared. Guard on non-empty so a transient empty boot payload never
	// clobbers a good map. No $effect loop: reads a prop, writes a *different*
	// $state var, never writes initialTags.
	$effect(() => {
		if (Object.keys(initialTags).length > 0) {
			tagMap = initialTags;
		}
	});
	let loading = $state(true);

	// Browser state
	let browseMode = $state<'folders' | 'tags' | 'properties'>('folders');
	let selectedFolder = $state<string | null>(null);
	let selectedTag = $state<string | null>(null);
	let propertyKey = $state('');
	let propertyValue = $state('');
	let propertyResults: NoteWithMeta[] = $state([]);

	// Selection
	let selectedPaths = $state<Set<string>>(new Set());
	let focusedIndex = $state(-1);

	// Derived: filter notes based on browser selection
	const filteredNotes = $derived.by(() => {
		if (browseMode === 'properties' && propertyResults.length > 0) {
			return propertyResults;
		}
		let result = allNotesWithMeta;
		if (browseMode === 'folders' && selectedFolder) {
			result = result.filter(n => {
				const notePath = n.path.replace(/\\/g, '/');
				const folderPath = selectedFolder!.replace(/\\/g, '/');
				return notePath.startsWith(folderPath + '/') || notePath.startsWith(folderPath + '\\');
			});
		}
		if (browseMode === 'tags' && selectedTag) {
			result = result.filter(n => n.tags.some(t => t === selectedTag || t.startsWith(selectedTag + '/')));
		}
		return result;
	});

	function normalizePath(p: string): string { return p.replace(/\\/g, '/').toLowerCase(); }

	// Load data — process 2 libraries at a time to avoid IPC flooding
	onMount(async () => {
		try {
			const libs = $libraries;
			const allNotes: NoteWithMeta[] = [];
			const allTags: Record<string, number> = {};
			const libTreeMap = new Map<string, FileEntry>();

			// Timeout wrapper: prevents infinite hang on any IPC call
			function withTimeout<T>(promise: Promise<T>, ms: number, fallback: T): Promise<T> {
				return Promise.race([
					promise,
					new Promise<T>(resolve => setTimeout(() => resolve(fallback), ms)),
				]);
			}

			// Process 2 libraries at a time
			for (let i = 0; i < libs.length; i += 2) {
				const batch = libs.slice(i, i + 2);
				const results = await Promise.all(batch.map(async (lib) => {
					const [notes, libTags, tree] = await Promise.all([
						withTimeout(collectLibraryNotesWithMeta(lib.path).catch(() => []), 15000, [] as NoteWithMeta[]),
						// Skip the O(N-file-reads) scan if the boot snapshot already has tags.
						// initialTags comes from cache_boot_snapshot_graph (SQLite-backed, write-time derived).
						Object.keys(initialTags).length > 0
							? Promise.resolve({} as Record<string, number>)
							: withTimeout(invoke<Record<string, number>>('scan_library_tags', { libraryPath: lib.path }).catch(() => ({})), 10000, {}),
						withTimeout(invoke<FileEntry[]>('read_library_tree', { libraryPath: lib.path, maxDepth: 10 }).catch(() => []), 10000, []),
					]);
					return { lib, notes, libTags, tree };
				}));
				for (const { lib, notes, libTags, tree } of results) {
					for (const n of notes) n.libraryName = lib.name;
					allNotes.push(...notes);
					for (const [tag, count] of Object.entries(libTags)) {
						allTags[tag] = (allTags[tag] || 0) + count;
					}
					libTreeMap.set(normalizePath(lib.path), { name: lib.name, path: lib.path, is_dir: true, children: tree } as FileEntry);
				}
			}

			// Group under child universes
			let childUniverses: ChildUniverseInfo[] = [];
			try { childUniverses = await getChildUniverses(); } catch {}

			const childLibPathSets = new Map<string, Set<string>>();
			for (const cu of childUniverses) {
				try {
					const childLibs = await invoke<{ id: string; name: string; path: string }[]>(
						'read_child_universe_libraries', { childPath: cu.path }
					).catch(() => []);
					childLibPathSets.set(cu.path, new Set(childLibs.map(l => normalizePath(l.path))));
				} catch {
					childLibPathSets.set(cu.path, new Set());
				}
			}

			const assignedPaths = new Set<string>();
			const trees: FileEntry[] = [];

			// Child universes first
			for (const cu of childUniverses) {
				const paths = childLibPathSets.get(cu.path) || new Set();
				const cuChildren: FileEntry[] = [];
				for (const p of paths) {
					const entry = libTreeMap.get(p);
					if (entry) {
						cuChildren.push(entry);
						assignedPaths.add(p);
					}
				}
				if (cuChildren.length > 0) {
					trees.push({ name: cu.name, path: cu.path, is_dir: true, children: cuChildren, isCUniverse: true } as FileEntry);
				}
			}

			// Own libraries (not assigned to any child universe)
			for (const lib of libs) {
				if (!assignedPaths.has(normalizePath(lib.path))) {
					const entry = libTreeMap.get(normalizePath(lib.path));
					if (entry) trees.push(entry);
				}
			}

			// Deduplicate notes by path (prevents Svelte each_key_duplicate crash)
			const seen = new Set<string>();
			const deduped: NoteWithMeta[] = [];
			for (const n of allNotes) {
				if (!seen.has(n.path)) { seen.add(n.path); deduped.push(n); }
			}
			allNotesWithMeta = deduped;
			// Use boot-snapshot tags when available — eliminates the filesystem scan cost.
			// Fall back to the locally-merged scan result (allTags) only when
			// initialTags is empty (e.g. second screen, or very early mount before graph loads).
			tagMap = Object.keys(initialTags).length > 0 ? initialTags : allTags;
			folderTrees = trees;
		} finally {
			loading = false;
		}
	});

	function handleNoteClick(note: NoteWithMeta) {
		if (mode === 'main') {
			onNoteClick?.(note.path, note.name.replace(/\.md$/, ''), note.libraryName ?? '');
		} else {
			onNotePreview?.(note.path, note.name.replace(/\.md$/, ''), note.libraryName ?? '');
		}
	}

	function handleNoteDoubleClick(note: NoteWithMeta) {
		if (mode === 'second') {
			onNoteDoubleClick?.(note.path, note.name.replace(/\.md$/, ''), note.libraryName ?? '');
		}
	}

	async function handlePropertySearch(key: string, value: string) {
		if (!key) return;
		try {
			const results = await invoke<any[]>('search_by_property', { key, value: value || '' });
			propertyResults = results.map((r: any) => ({
				name: r.name || '',
				path: r.path || '',
				modified: 0,
				size: 0,
				preview: r.preview || '',
				tags: [],
				folder: '',
				libraryName: r.library_name || '',
			}));
		} catch {
			propertyResults = [];
		}
	}

	async function handleBatchTag() {
		const tag = prompt('Enter tag to add:');
		if (!tag || !tag.trim()) return;
		const tagClean = tag.trim().replace(/^#/, '');
		let success = 0;
		for (const path of selectedPaths) {
			try {
				const content = await invoke<string>('read_note', { filePath: path });
				let newContent: string;
				if (content.startsWith('---')) {
					const endIdx = content.indexOf('---', 3);
					if (endIdx > 0) {
						const yaml = content.substring(3, endIdx);
						const body = content.substring(endIdx + 3);
						if (yaml.includes('tags:')) {
							// Append to existing tags list
							newContent = '---' + yaml.replace(/^(tags:.*)/m, `$1\n  - ${tagClean}`) + '---' + body;
						} else {
							// Add tags section
							newContent = '---' + yaml + `tags:\n  - ${tagClean}\n` + '---' + body;
						}
					} else {
						newContent = `---\ntags:\n  - ${tagClean}\n---\n` + content;
					}
				} else {
					newContent = `---\ntags:\n  - ${tagClean}\n---\n` + content;
				}
				await invoke('write_note', { filePath: path, content: newContent, origin: 'bulk_tag' });
				success++;
			} catch { /* skip failed */ }
		}
		// Refresh data
		if (success > 0) {
			selectedPaths = new Set();
			await refreshData();
		}
	}

	async function handleBatchMove() {
		const folder = await invoke<string | null>('pick_folder').catch(() => null);
		if (!folder) return;
		let success = 0;
		for (const path of selectedPaths) {
			try {
				await invoke('move_item', { sourcePath: path, targetFolder: folder });
				success++;
			} catch { /* skip failed */ }
		}
		if (success > 0) {
			selectedPaths = new Set();
			await refreshData();
		}
	}

	async function handleBatchDelete() {
		const dest = $appSettings.trashDestination ?? 'system';
		const where = dest === 'local' ? 'moved to the .trash folder' : 'moved to the Recycle Bin';
		if (!confirm(`Delete ${selectedPaths.size} notes? They will be ${where}.`)) return;
		let success = 0;
		for (const path of selectedPaths) {
			try {
				await deleteWithSetting(path);
				success++;
			} catch { /* skip failed */ }
		}
		if (success > 0) {
			selectedPaths = new Set();
			await refreshData();
		}
	}

	async function refreshData() {
		const libs = $libraries;
		const allNotes: NoteWithMeta[] = [];
		const allTags: Record<string, number> = {};
		for (const lib of libs) {
			const notes = await collectLibraryNotesWithMeta(lib.path).catch(() => []);
			for (const n of notes) n.libraryName = lib.name;
			allNotes.push(...notes);
			const libTags = await invoke<Record<string, number>>('scan_library_tags', { libraryPath: lib.path }).catch(() => ({}));
			for (const [tag, count] of Object.entries(libTags)) {
				allTags[tag] = (allTags[tag] || 0) + count;
			}
		}
		allNotesWithMeta = allNotes;
		tagMap = allTags;
	}

	// Keyboard handler
	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			focusedIndex = Math.min(focusedIndex + 1, filteredNotes.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			focusedIndex = Math.max(focusedIndex - 1, 0);
		} else if (e.key === 'Enter' && focusedIndex >= 0) {
			e.preventDefault();
			handleNoteClick(filteredNotes[focusedIndex]);
		} else if (e.key === ' ' && focusedIndex >= 0) {
			e.preventDefault();
			const path = filteredNotes[focusedIndex]?.path;
			if (path) {
				const next = new Set(selectedPaths);
				if (next.has(path)) next.delete(path); else next.add(path);
				selectedPaths = next;
			}
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="notebook-navigator" onkeydown={handleKeydown} tabindex="-1">
	{#if loading}
		<div class="nav-loading">
			<div class="nav-spinner"></div>
			<span>{$t('secondScreen.loading') || 'Loading...'}</span>
		</div>
	{:else}
		<div class="nav-panes">
			<div class="nav-browser-pane">
				<NavBrowserPane
					{browseMode}
					folderTree={folderTrees}
					{tagMap}
					{selectedFolder}
					{selectedTag}
					{propertyKey}
					{propertyValue}
					onModeChange={(m) => { browseMode = m; selectedFolder = null; selectedTag = null; propertyResults = []; }}
					onFolderSelect={(p) => {
						selectedFolder = p; focusedIndex = -1;
						// Propagate to Star View
						if (p && onFolderSelect) {
							// Check if this is a cUniverse entry — pass all child library paths
							const cuEntry = folderTrees.find(e => e.isCUniverse && e.path === p);
							if (cuEntry && cuEntry.children) {
								onFolderSelect(cuEntry.children.map(c => c.path));
							} else {
								onFolderSelect(p);
							}
						} else if (onFolderSelect) {
							onFolderSelect(null);
						}
					}}
					onTagSelect={(t) => { selectedTag = t; focusedIndex = -1; }}
					onPropertySearch={handlePropertySearch}
				/>
			</div>
			<div class="nav-divider"></div>
			<div class="nav-list-pane">
				<NavFileList
					files={filteredNotes}
					{libraryColorMap}
					{selectedPaths}
					{focusedIndex}
					onNoteClick={handleNoteClick}
					onNoteDoubleClick={handleNoteDoubleClick}
					onSelectionChange={(s) => selectedPaths = s}
				/>
			</div>
		</div>
		<NavBatchBar
			count={selectedPaths.size}
			onBatchTag={handleBatchTag}
			onBatchMove={handleBatchMove}
			onBatchDelete={handleBatchDelete}
			onClearSelection={() => selectedPaths = new Set()}
		/>
	{/if}
</div>

<style>
	.notebook-navigator {
		display: flex; flex-direction: column; height: 100%; overflow: hidden;
		background: var(--background-primary);
		font-family: var(--font-interface-theme), sans-serif;
	}

	.nav-loading {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		height: 100%; gap: 8px; color: var(--text-muted); font-size: 13px;
	}
	.nav-spinner {
		width: 20px; height: 20px; border: 2px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent); border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	.nav-panes { display: flex; flex: 1; overflow: hidden; }

	.nav-browser-pane { width: 40%; min-width: 140px; overflow: hidden; border-inline-end: 1px solid var(--background-modifier-border); }
	.nav-list-pane { flex: 1; overflow: hidden; }

	.nav-divider {
		width: 3px; flex-shrink: 0; cursor: col-resize;
		background: transparent;
	}
	.nav-divider:hover { background: var(--interactive-accent); }

	/* Responsive: stack vertically when narrow */
	@container (max-width: 350px) {
		.nav-panes { flex-direction: column; }
		.nav-browser-pane { width: 100%; max-height: 40%; border-inline-end: none; border-bottom: 1px solid var(--background-modifier-border); }
	}
</style>

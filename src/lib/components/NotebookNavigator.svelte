<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import { libraries, collectLibraryNotesWithMeta, type NoteWithMeta, type FileEntry } from '$lib/libraries/store';
	import NavBrowserPane from './navigator/NavBrowserPane.svelte';
	import NavFileList from './navigator/NavFileList.svelte';
	import NavBatchBar from './navigator/NavBatchBar.svelte';

	let {
		mode = 'main' as 'main' | 'second',
		libraryColorMap = {} as Record<string, string>,
		onNoteClick,
		onNoteDoubleClick,
		onNotePreview,
	}: {
		mode?: 'main' | 'second';
		libraryColorMap?: Record<string, string>;
		onNoteClick?: (path: string, name: string, libraryName: string) => void;
		onNoteDoubleClick?: (path: string, name: string, libraryName: string) => void;
		onNotePreview?: (path: string, name: string, libraryName: string) => void;
	} = $props();

	// Data
	let allNotesWithMeta: NoteWithMeta[] = $state([]);
	let folderTrees: FileEntry[] = $state([]);
	let tagMap: Record<string, number> = $state({});
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

	// Load data
	onMount(async () => {
		try {
			const libs = $libraries;
			const allNotes: NoteWithMeta[] = [];
			const allTags: Record<string, number> = {};
			const trees: FileEntry[] = [];

			for (const lib of libs) {
				// Notes with metadata
				const notes = await collectLibraryNotesWithMeta(lib.path);
				for (const n of notes) n.libraryName = lib.name;
				allNotes.push(...notes);

				// Tags
				const libTags = await invoke<Record<string, number>>('scan_library_tags', { libraryPath: lib.path }).catch(() => ({}));
				for (const [tag, count] of Object.entries(libTags)) {
					allTags[tag] = (allTags[tag] || 0) + count;
				}

				// Folder tree
				const tree = await invoke<FileEntry[]>('read_library_tree', { libraryPath: lib.path, maxDepth: 10 }).catch(() => []);
				trees.push({ name: lib.name, path: lib.path, is_dir: true, children: tree } as FileEntry);
			}

			allNotesWithMeta = allNotes;
			tagMap = allTags;
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

	function handleBatchTag() {
		const tag = prompt('Enter tag to add:');
		if (!tag) return;
		// TODO: Add tag to all selected notes' frontmatter
		alert(`Would add tag "${tag}" to ${selectedPaths.size} notes (not yet implemented)`);
	}

	function handleBatchMove() {
		alert(`Would move ${selectedPaths.size} notes (not yet implemented)`);
	}

	function handleBatchDelete() {
		if (confirm(`Delete ${selectedPaths.size} notes? This cannot be undone.`)) {
			alert(`Would delete ${selectedPaths.size} notes (not yet implemented)`);
		}
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
					onFolderSelect={(p) => { selectedFolder = p; focusedIndex = -1; }}
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

	.nav-browser-pane { width: 35%; min-width: 120px; overflow: hidden; border-inline-end: 1px solid var(--background-modifier-border); }
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

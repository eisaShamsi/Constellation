<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { marked } from 'marked';
	import {
		libraries, libraryStats, selectedNote, searchResults, appSettings,
		loadLibraries, loadAllStats, addLibrary, createNewLibraryAt, removeLibrary,
		searchAllStars, closeNote, timeAgo, openNoteTab
	} from '$lib/libraries/store';
	import type { LibraryStats, FileEntry } from '$lib/libraries/store';
	import { getChildUniverses, type ChildUniverseInfo } from '$lib/universe/store';
	import FileTree from '$lib/components/FileTree.svelte';
	import CreateItemDialog from '$lib/components/CreateItemDialog.svelte';

	let error = $state('');
	let adding = $state(false);
	// MIG-008 §152 — `creatingNew` / `newLibraryName` removed; the inline
	// "Create new library" form on this Library Manager screen now opens the
	// shared CreateItemDialog (kind='library') for consistency with the rest
	// of the create surfaces (sidebar, command palette, welcome card).
	let createLibraryOpen = $state(false);
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Per-library file trees (loaded on expand)
	let libraryTrees = $state<Record<string, FileEntry[]>>({});
	let expandedLibraries = $state<Set<string>>(new Set());

	// Font from settings
	const uiFont = $derived($appSettings.interfaceFont || 'system-ui, sans-serif');

	// Child universes
	let childUniverses = $state<ChildUniverseInfo[]>([]);
	let childUniverseLibPaths = $state<Map<string, Set<string>>>(new Map());
	let expandedChildUniverses = $state<Set<string>>(new Set());

	function normalizePath(p: string): string {
		return p.replace(/\\/g, '/').toLowerCase();
	}

	function isChildUniverseLib(libPath: string): boolean {
		const norm = normalizePath(libPath);
		for (const paths of childUniverseLibPaths.values()) {
			if (paths.has(norm)) return true;
		}
		return false;
	}

	function getChildUniverseLibs(cuPath: string): LibraryStats[] {
		const paths = childUniverseLibPaths.get(cuPath);
		if (!paths) return [];
		return $libraryStats.filter(lib => paths.has(normalizePath(lib.path)));
	}

	const ownLibraries = $derived($libraryStats.filter(lib => !isChildUniverseLib(lib.path)));

	// Configure marked for safe rendering
	marked.setOptions({ breaks: true, gfm: true });

	onDestroy(() => {
		clearTimeout(searchTimeout);
	});

	onMount(async () => {
		await loadLibraries();
		await loadAllStats();

		// Load child universes
		try {
			childUniverses = await getChildUniverses();
			const map = new Map<string, Set<string>>();
			for (const cu of childUniverses) {
				try {
					const childLibs = await invoke<{ id: string; name: string; path: string }[]>(
						'read_child_universe_libraries', { childPath: cu.path }
					);
					map.set(cu.path, new Set(childLibs.map(l => normalizePath(l.path))));
				} catch {
					map.set(cu.path, new Set());
				}
			}
			childUniverseLibPaths = map;
		} catch { /* no child universes */ }

		// Auto-expand first library if only one
		if ($libraryStats.length === 1) {
			await toggleLibrary($libraryStats[0]);
		}
	});

	async function toggleLibrary(lib: LibraryStats) {
		const id = lib.library_id;
		if (expandedLibraries.has(id)) {
			expandedLibraries.delete(id);
			expandedLibraries = new Set(expandedLibraries);
		} else {
			// Load tree if not loaded
			if (!libraryTrees[id]) {
				const tree: FileEntry[] = await invoke('read_library_tree', {
					path: lib.path,
					maxDepth: 4
				});
				libraryTrees[id] = tree;
				libraryTrees = { ...libraryTrees };
			}
			expandedLibraries.add(id);
			expandedLibraries = new Set(expandedLibraries);
		}
	}

	async function handleAddLibrary() {
		adding = true;
		error = '';
		try {
			await addLibrary();
			await loadAllStats();
		} catch (e) { error = String(e); }
		adding = false;
	}

	// MIG-008 §152 — opens the shared CreateItemDialog. The dialog handles
	// name + location collection + validation; on commit it invokes
	// `createNewLibraryAt(parentPath, name)`.
	function handleCreateNew() {
		error = '';
		createLibraryOpen = true;
	}

	async function handleRemove(e: Event, id: string) {
		e.stopPropagation();
		try { await removeLibrary(id); } catch (e2) { error = String(e2); }
	}

	function handleSearch(e: Event) {
		searchQuery = (e.target as HTMLInputElement).value;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => searchAllStars(searchQuery), 300);
	}

	async function handleNoteClick(filePath: string, noteName: string) {
		const lib = $libraries.find(v => filePath.startsWith(v.path));
		await openNoteTab(filePath, lib?.name ?? '');
	}

	async function handleSearchResultClick(path: string, libraryName: string) {
		await openNoteTab(path, libraryName);
	}

	function renderMarkdown(md: string): string {
		return marked.parse(md) as string;
	}

	const colors = ['#7c3aed', '#3b82f6', '#06b6d4', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#8b5cf6'];
</script>

<div class="workspace-layout" style="font-family:{uiFont};">
	<!-- Sidebar -->
	<aside class="sidebar">
		<!-- Search -->
		<div class="sidebar-search">
			<input
				type="text"
				placeholder={$t('libraries.searchPlaceholder')}
				value={searchQuery}
				oninput={handleSearch}
			/>
		</div>

		<!-- Search Results -->
		{#if searchQuery}
			<div class="search-panel">
				{#if $searchResults.length > 0}
					<div class="section-label">{$searchResults.length} {$t('libraries.results')}</div>
					{#each $searchResults as star}
						<button class="search-result" class:active={$selectedNote?.path === star.path} onclick={() => handleSearchResultClick(star.path, star.library_name)}>
							<div class="sr-name">{star.name}</div>
							<div class="sr-meta">
								<span class="sr-lib-name">{star.library_name}</span>
								<span class="sr-preview">{star.preview}</span>
							</div>
						</button>
					{/each}
				{:else}
					<div class="no-results">{$t('libraries.noResults')}</div>
				{/if}
			</div>
		{:else}
			<!-- Library Tree -->
			<div class="lib-list">
				<!-- Child Universes first -->
				{#each childUniverses as child}
					<div class="lib-section">
						<button class="lib-header cu-header" onclick={() => {
							const next = new Set(expandedChildUniverses);
							if (next.has(child.path)) next.delete(child.path); else next.add(child.path);
							expandedChildUniverses = next;
						}}>
							<svg class="lib-chevron" class:expanded={expandedChildUniverses.has(child.path)} width="10" height="10" viewBox="0 0 10 10">
								<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
							</svg>
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#6366f1" stroke-width="1.5" style="flex-shrink: 0;">
								<circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/>
								<path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/>
								<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2"/>
							</svg>
							<span class="lib-name">{child.name}</span>
							<span class="library-count">{child.library_count}</span>
						</button>

						{#if expandedChildUniverses.has(child.path)}
							<div class="cu-libs">
								{#each getChildUniverseLibs(child.path) as lib, i}
									<div class="lib-section">
										<button class="lib-header" onclick={() => toggleLibrary(lib)} style="--accent: {lib.path ? colors[(colors.length - 1 - i) % colors.length] : '#7c3aed'}">
											<svg class="lib-chevron" class:expanded={expandedLibraries.has(lib.library_id)} width="10" height="10" viewBox="0 0 10 10">
												<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
											</svg>
											<span class="lib-dot" style="background: {colors[(colors.length - 1 - i) % colors.length]}"></span>
											<span class="lib-name">{lib.name}</span>
											<span class="library-count">{lib.star_count}</span>
										</button>
										{#if expandedLibraries.has(lib.library_id) && libraryTrees[lib.library_id]}
											<div class="lib-tree">
												<FileTree entries={libraryTrees[lib.library_id]} libraryId={lib.library_id} libraryName={lib.name} onNoteClick={handleNoteClick} />
											</div>
										{/if}
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/each}

				<!-- Own Libraries -->
				{#each ownLibraries as lib, i}
					<div class="lib-section">
						<button class="lib-header" onclick={() => toggleLibrary(lib)} style="--accent: {colors[i % colors.length]}">
							<svg class="lib-chevron" class:expanded={expandedLibraries.has(lib.library_id)} width="10" height="10" viewBox="0 0 10 10">
								<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
							</svg>
							<span class="lib-dot" style="background: {colors[i % colors.length]}"></span>
							<span class="lib-name">{lib.name}</span>
							<span class="library-count">{lib.star_count}</span>
						</button>

						{#if expandedLibraries.has(lib.library_id) && libraryTrees[lib.library_id]}
							<div class="lib-tree">
								<FileTree
									entries={libraryTrees[lib.library_id]}
									libraryId={lib.library_id}
									libraryName={lib.name}
									onNoteClick={handleNoteClick}
								/>
							</div>
						{/if}
					</div>
				{/each}

				<!-- Add Library -->
				<button class="add-lib-btn" onclick={handleAddLibrary} disabled={adding}>
					{adding ? '...' : $t('libraries.addLibrary')}
				</button>
			</div>
		{/if}

		{#if error}
			<div class="sidebar-error">{error}</div>
		{/if}
	</aside>

	<!-- Main Content -->
	<main class="main-content">
		{#if $selectedNote}
			<!-- Note View -->
			<div class="note-view">
				<div class="note-topbar">
					<div class="note-breadcrumb">
						<span class="breadcrumb-lib">{$selectedNote.libraryName}</span>
						<span class="breadcrumb-sep">/</span>
						<span class="breadcrumb-name">{$selectedNote.path.split(/[\\/]/).pop()?.replace('.md', '')}</span>
					</div>
					<button class="close-note" onclick={closeNote}>×</button>
				</div>
				<div class="note-content">
					{@html renderMarkdown($selectedNote.content)}
				</div>
			</div>
		{:else}
			<!-- Empty State -->
			<div class="empty-state">
				{#if $libraryStats.length === 0}
					<div class="empty-icon">✦</div>
					<h2>{$t('libraries.welcomeTitle')}</h2>
					<p class="empty-subtitle">{$t('libraries.welcomeSubtitle')}</p>

					<div class="option-cards">
						<!-- Option 1: New Library -->
						<div class="option-card">
							<div class="option-icon">📁</div>
							<h3>{$t('libraries.newLibrary')}</h3>
							<p>{$t('libraries.newLibraryDesc')}</p>
							<button class="option-btn primary" onclick={handleCreateNew}>
								+ {$t('libraries.newLibrary')}
							</button>
						</div>

						<!-- Option 2: Link Existing -->
						<div class="option-card">
							<div class="option-icon">🔗</div>
							<h3>{$t('libraries.linkLibrary')}</h3>
							<p>{$t('libraries.linkLibraryDesc')}</p>
							<button class="option-btn secondary" onclick={handleAddLibrary} disabled={adding}>
								{adding ? '...' : '📂 Browse'}
							</button>
						</div>
					</div>

					{#if error}
						<div class="empty-error">{error}</div>
					{/if}
				{:else}
					<div class="empty-hint">
						<p>{$t('libraries.selectNote')}</p>
						<div class="stats-row">
							{#each $libraryStats as lib, i}
								<div class="stat-chip" style="--accent: {colors[i % colors.length]}">
									<span class="chip-dot" style="background: {colors[i % colors.length]}"></span>
									{lib.name}
									<span class="chip-count">{lib.star_count}</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</main>
</div>

{#if createLibraryOpen}
	<CreateItemDialog
		open={true}
		kind="library"
		parentPath=""
		onClose={() => createLibraryOpen = false}
		onCreate={async ({ name, location }) => {
			await createNewLibraryAt(location, name);
			await loadAllStats();
		}}
	/>
{/if}

<style>
	.workspace-layout {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	/* ─── Sidebar ─── */
	.sidebar {
		width: 280px;
		min-width: 280px;
		background: #f6f8fa;
		border-inline-end: 1px solid #d0d7de;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.sidebar-search {
		padding: 0.5rem;
		border-bottom: 1px solid #d0d7de;
	}
	.sidebar-search input {
		width: 100%;
		padding: 0.45rem 0.6rem;
		background: #ffffff;
		border: 1px solid #d0d7de;
		border-radius: 6px;
		color: #1f2328;
		font-size: 0.85rem;
		font-family: inherit;
	}
	.sidebar-search input:focus { border-color: #7c3aed; outline: none; }
	.sidebar-search input::placeholder { color: #656d76; }

	.lib-list {
		flex: 1;
		overflow-y: auto;
		padding: 0.3rem 0;
	}

	.lib-section { }

	.lib-header {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		width: 100%;
		padding: 0.4rem 0.6rem;
		background: none;
		border: none;
		color: #24292f;
		font-size: 0.85rem;
		font-weight: 600;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
	}
	.lib-header:hover { background: #eaeef2; }

	.lib-chevron {
		color: #656d76;
		flex-shrink: 0;
		transition: transform 0.15s ease;
	}
	.lib-chevron.expanded { transform: rotate(90deg); }

	.lib-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.lib-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.library-count {
		color: #656d76;
		font-size: 0.75rem;
		font-weight: 400;
	}

	.lib-tree {
		padding-inline-start: 0.8rem;
		padding-bottom: 0.3rem;
	}

	.cu-header {
		font-weight: 600;
	}
	.cu-libs {
		padding-inline-start: 1rem;
	}

	.add-lib-btn {
		display: block;
		width: calc(100% - 1rem);
		margin: 0.5rem;
		padding: 0.4rem;
		background: none;
		border: 1px dashed #d0d7de;
		border-radius: 6px;
		color: #656d76;
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
		text-align: center;
	}
	.add-lib-btn:hover { border-color: #7c3aed; color: #7c3aed; }

	.sidebar-error {
		padding: 0.5rem;
		color: #cf222e;
		font-size: 0.8rem;
		border-top: 1px solid #d0d7de;
	}

	/* ─── Search Panel ─── */
	.search-panel {
		flex: 1;
		overflow-y: auto;
		padding: 0.3rem;
	}

	.section-label {
		padding: 0.3rem 0.5rem;
		font-size: 0.75rem;
		color: #656d76;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.search-result {
		display: block;
		width: 100%;
		padding: 0.5rem 0.6rem;
		background: none;
		border: none;
		color: #24292f;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
		border-radius: 4px;
	}
	.search-result:hover { background: #eaeef2; }
	.search-result.active { background: #7c3aed18; }

	.sr-name { font-size: 0.85rem; font-weight: 500; }
	.sr-meta { display: flex; gap: 0.4rem; font-size: 0.75rem; color: #656d76; margin-top: 2px; }
	.sr-lib-name { color: #7c3aed; flex-shrink: 0; }
	.sr-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.no-results { padding: 1rem; text-align: center; color: #656d76; font-size: 0.85rem; }

	/* ─── Main Content ─── */
	.main-content {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.note-view {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.note-topbar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 1rem;
		border-bottom: 1px solid #d0d7de;
		background: #f6f8fa;
		min-height: 38px;
	}

	.note-breadcrumb { font-size: 0.8rem; color: #57606a; }
	.breadcrumb-lib { color: #7c3aed; }
	.breadcrumb-sep { margin: 0 0.3rem; color: #d0d7de; }
	.breadcrumb-name { color: #24292f; }

	.close-note {
		background: none;
		border: none;
		color: #656d76;
		font-size: 1.3rem;
		cursor: pointer;
		padding: 0.2rem 0.4rem;
		line-height: 1;
		border-radius: 4px;
	}
	.close-note:hover { color: #24292f; background: #eaeef2; }

	.note-content {
		flex: 1;
		overflow-y: auto;
		padding: 2rem 3rem;
		max-width: 800px;
		line-height: 1.75;
	}

	/* Markdown Styles */
	.note-content :global(h1) { font-size: 1.8rem; margin: 1.5rem 0 0.75rem; border-bottom: 1px solid #d0d7de; padding-bottom: 0.3rem; }
	.note-content :global(h2) { font-size: 1.4rem; margin: 1.3rem 0 0.5rem; }
	.note-content :global(h3) { font-size: 1.15rem; margin: 1rem 0 0.4rem; }
	.note-content :global(p) { margin: 0.5rem 0; }
	.note-content :global(a) { color: #7c3aed; }
	.note-content :global(code) {
		background: #f0f2f5;
		padding: 0.15em 0.35em;
		border-radius: 4px;
		font-size: 0.9em;
		color: #24292f;
	}
	.note-content :global(pre) {
		background: #f6f8fa;
		border: 1px solid #d0d7de;
		border-radius: 6px;
		padding: 1rem;
		overflow-x: auto;
	}
	.note-content :global(pre code) { background: none; padding: 0; }
	.note-content :global(blockquote) {
		border-inline-start: 3px solid #7c3aed;
		padding: 0.25rem 1rem;
		margin: 0.5rem 0;
		color: #57606a;
	}
	.note-content :global(ul), .note-content :global(ol) { padding-inline-start: 1.5rem; }
	.note-content :global(li) { margin: 0.2rem 0; }
	.note-content :global(hr) { border: none; border-top: 1px solid #d0d7de; margin: 1.5rem 0; }
	.note-content :global(table) { border-collapse: collapse; width: 100%; margin: 0.75rem 0; }
	.note-content :global(th), .note-content :global(td) {
		border: 1px solid #d0d7de;
		padding: 0.4rem 0.7rem;
		text-align: start;
	}
	.note-content :global(th) { background: #f6f8fa; }
	.note-content :global(img) { max-width: 100%; border-radius: 6px; }
	.note-content :global(input[type="checkbox"]) { margin-inline-end: 0.4rem; }

	/* ─── Empty State ─── */
	.empty-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		padding: 2rem;
		color: #656d76;
	}

	.empty-icon { font-size: 2.5rem; margin-bottom: 0.75rem; color: #7c3aed; }
	.empty-state h2 { color: #24292f; font-size: 1.4rem; margin-bottom: 0.4rem; }
	.empty-subtitle { margin-bottom: 2rem; font-size: 0.95rem; }

	.option-cards {
		display: flex;
		gap: 1.25rem;
		max-width: 620px;
		width: 100%;
	}

	.option-card {
		flex: 1;
		background: #ffffff;
		border: 1px solid #d0d7de;
		border-radius: 12px;
		padding: 1.5rem;
		text-align: center;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.4rem;
		transition: border-color 0.15s, box-shadow 0.15s;
	}
	.option-card:hover {
		border-color: #7c3aed;
		box-shadow: 0 2px 12px rgba(124, 58, 237, 0.08);
	}

	.option-icon { font-size: 1.8rem; margin-bottom: 0.25rem; }
	.option-card h3 { color: #24292f; font-size: 1rem; font-weight: 600; margin: 0; }
	.option-card p { color: #656d76; font-size: 0.82rem; margin: 0 0 0.75rem 0; line-height: 1.45; }

	.new-lib-form {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		width: 100%;
	}

	.lib-name-input {
		width: 100%;
		padding: 0.45rem 0.6rem;
		border: 1px solid #d0d7de;
		border-radius: 6px;
		font-size: 0.85rem;
		font-family: inherit;
		text-align: center;
		color: #24292f;
		background: #f6f8fa;
	}
	.lib-name-input:focus { border-color: #7c3aed; outline: none; background: #fff; }

	.option-btn {
		padding: 0.5rem 1rem;
		border-radius: 8px;
		font-size: 0.88rem;
		font-weight: 600;
		font-family: inherit;
		cursor: pointer;
		border: none;
		width: 100%;
		transition: background 0.15s;
	}
	.option-btn.primary { background: #7c3aed; color: white; }
	.option-btn.primary:hover { background: #6d28d9; }
	.option-btn.secondary { background: #f6f8fa; color: #24292f; border: 1px solid #d0d7de; }
	.option-btn.secondary:hover { border-color: #7c3aed; color: #7c3aed; }
	.option-btn:disabled { opacity: 0.6; cursor: default; }

	.empty-error { margin-top: 1rem; color: #cf222e; font-size: 0.85rem; }

	.empty-hint p { font-size: 1rem; margin-bottom: 1.5rem; }

	.stats-row {
		display: flex;
		gap: 0.75rem;
		flex-wrap: wrap;
		justify-content: center;
	}

	.stat-chip {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		background: #f6f8fa;
		border: 1px solid #d0d7de;
		padding: 0.4em 0.8em;
		border-radius: 20px;
		font-size: 0.85rem;
		color: #57606a;
	}
	.chip-dot { width: 6px; height: 6px; border-radius: 50%; }
	.chip-count { color: #656d76; }
</style>

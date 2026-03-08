<script lang="ts">
	import { onMount } from 'svelte';
	import { locale } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { marked } from 'marked';
	import {
		vaults, vaultStats, selectedNote, searchResults,
		loadVaults, loadAllStats, addVault, removeVault,
		searchAllStars, closeNote, timeAgo
	} from '$lib/vaults/store';
	import type { VaultStats, FileEntry } from '$lib/vaults/store';
	import FileTree from '$lib/components/FileTree.svelte';

	let error = $state('');
	let adding = $state(false);
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Per-vault file trees (loaded on expand)
	let vaultTrees = $state<Record<string, FileEntry[]>>({});
	let expandedVaults = $state<Set<string>>(new Set());

	const ar = $derived($locale === 'ar');

	// Configure marked for safe rendering
	marked.setOptions({ breaks: true, gfm: true });

	onMount(async () => {
		await loadVaults();
		await loadAllStats();
		// Auto-expand first vault if only one
		if ($vaultStats.length === 1) {
			await toggleVault($vaultStats[0]);
		}
	});

	async function toggleVault(vault: VaultStats) {
		const id = vault.vault_id;
		if (expandedVaults.has(id)) {
			expandedVaults.delete(id);
			expandedVaults = new Set(expandedVaults);
		} else {
			// Load tree if not loaded
			if (!vaultTrees[id]) {
				const tree: FileEntry[] = await invoke('read_vault_tree', {
					path: vault.path,
					maxDepth: 4
				});
				vaultTrees[id] = tree;
				vaultTrees = { ...vaultTrees };
			}
			expandedVaults.add(id);
			expandedVaults = new Set(expandedVaults);
		}
	}

	async function handleAddVault() {
		adding = true;
		error = '';
		try {
			await addVault();
			await loadAllStats();
		} catch (e) { error = String(e); }
		adding = false;
	}

	async function handleRemove(e: Event, id: string) {
		e.stopPropagation();
		try { await removeVault(id); } catch (e2) { error = String(e2); }
	}

	function handleSearch(e: Event) {
		searchQuery = (e.target as HTMLInputElement).value;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => searchAllStars(searchQuery), 300);
	}

	async function handleNoteClick(filePath: string, noteName: string) {
		const content: string = await invoke('read_note', { filePath });
		// Find which vault this belongs to
		const vault = $vaults.find(v => filePath.startsWith(v.path));
		selectedNote.set({
			path: filePath,
			content,
			vaultName: vault?.name ?? ''
		});
	}

	async function handleSearchResultClick(path: string, vaultName: string) {
		const content: string = await invoke('read_note', { filePath: path });
		selectedNote.set({ path, content, vaultName });
	}

	function renderMarkdown(md: string): string {
		return marked.parse(md) as string;
	}

	const colors = ['#7c3aed', '#3b82f6', '#06b6d4', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#8b5cf6'];
</script>

<div class="workspace-layout">
	<!-- Sidebar -->
	<aside class="sidebar">
		<!-- Search -->
		<div class="sidebar-search">
			<input
				type="text"
				placeholder={ar ? 'بحث في جميع الخزائن...' : 'Search all vaults...'}
				value={searchQuery}
				oninput={handleSearch}
			/>
		</div>

		<!-- Search Results -->
		{#if searchQuery}
			<div class="search-panel">
				{#if $searchResults.length > 0}
					<div class="section-label">{$searchResults.length} {ar ? 'نتيجة' : 'results'}</div>
					{#each $searchResults as star}
						<button class="search-result" class:active={$selectedNote?.path === star.path} onclick={() => handleSearchResultClick(star.path, star.vault_name)}>
							<div class="sr-name">{star.name}</div>
							<div class="sr-meta">
								<span class="sr-vault">{star.vault_name}</span>
								<span class="sr-preview">{star.preview}</span>
							</div>
						</button>
					{/each}
				{:else}
					<div class="no-results">{ar ? 'لا توجد نتائج' : 'No results'}</div>
				{/if}
			</div>
		{:else}
			<!-- Vault Tree -->
			<div class="vault-list">
				{#each $vaultStats as vault, i}
					<div class="vault-section">
						<button class="vault-header" onclick={() => toggleVault(vault)} style="--accent: {colors[i % colors.length]}">
							<svg class="vault-chevron" class:expanded={expandedVaults.has(vault.vault_id)} width="10" height="10" viewBox="0 0 10 10">
								<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
							</svg>
							<span class="vault-dot" style="background: {colors[i % colors.length]}"></span>
							<span class="vault-name">{vault.name}</span>
							<span class="vault-count">{vault.star_count}</span>
						</button>

						{#if expandedVaults.has(vault.vault_id) && vaultTrees[vault.vault_id]}
							<div class="vault-tree">
								<FileTree
									entries={vaultTrees[vault.vault_id]}
									vaultId={vault.vault_id}
									vaultName={vault.name}
									onNoteClick={handleNoteClick}
								/>
							</div>
						{/if}
					</div>
				{/each}

				<!-- Add Vault -->
				<button class="add-vault-btn" onclick={handleAddVault} disabled={adding}>
					{adding ? '...' : (ar ? '+ إضافة خزينة' : '+ Add vault')}
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
						<span class="breadcrumb-vault">{$selectedNote.vaultName}</span>
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
				{#if $vaultStats.length === 0}
					<div class="empty-icon">🌌</div>
					<h2>{ar ? 'مرحبا بك في كونستليشن' : 'Welcome to Constellation'}</h2>
					<p>{ar ? 'أضف خزينة أوبسيديان الأولى من الشريط الجانبي' : 'Add your first Obsidian vault from the sidebar'}</p>
					<button class="empty-add-btn" onclick={handleAddVault}>
						{ar ? '+ إضافة خزينة' : '+ Add Vault'}
					</button>
				{:else}
					<div class="empty-hint">
						<p>{ar ? 'اختر ملاحظة من الشريط الجانبي' : 'Select a note from the sidebar'}</p>
						<div class="stats-row">
							{#each $vaultStats as vault, i}
								<div class="stat-chip" style="--accent: {colors[i % colors.length]}">
									<span class="chip-dot" style="background: {colors[i % colors.length]}"></span>
									{vault.name}
									<span class="chip-count">{vault.star_count}</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</main>
</div>

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
		background: #161b22;
		border-inline-end: 1px solid #21262d;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.sidebar-search {
		padding: 0.5rem;
		border-bottom: 1px solid #21262d;
	}
	.sidebar-search input {
		width: 100%;
		padding: 0.45rem 0.6rem;
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 6px;
		color: #e0e0e0;
		font-size: 0.85rem;
		font-family: inherit;
	}
	.sidebar-search input:focus { border-color: #7c3aed; outline: none; }
	.sidebar-search input::placeholder { color: #484f58; }

	.vault-list {
		flex: 1;
		overflow-y: auto;
		padding: 0.3rem 0;
	}

	.vault-section { }

	.vault-header {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		width: 100%;
		padding: 0.4rem 0.6rem;
		background: none;
		border: none;
		color: #c9d1d9;
		font-size: 0.85rem;
		font-weight: 600;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
	}
	.vault-header:hover { background: #1c2128; }

	.vault-chevron {
		color: #484f58;
		flex-shrink: 0;
		transition: transform 0.15s ease;
	}
	.vault-chevron.expanded { transform: rotate(90deg); }

	.vault-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.vault-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.vault-count {
		color: #484f58;
		font-size: 0.75rem;
		font-weight: 400;
	}

	.vault-tree {
		padding-inline-start: 0.8rem;
		padding-bottom: 0.3rem;
	}

	.add-vault-btn {
		display: block;
		width: calc(100% - 1rem);
		margin: 0.5rem;
		padding: 0.4rem;
		background: none;
		border: 1px dashed #30363d;
		border-radius: 6px;
		color: #484f58;
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
		text-align: center;
	}
	.add-vault-btn:hover { border-color: #7c3aed; color: #7c3aed; }

	.sidebar-error {
		padding: 0.5rem;
		color: #f85149;
		font-size: 0.8rem;
		border-top: 1px solid #21262d;
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
		color: #484f58;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.search-result {
		display: block;
		width: 100%;
		padding: 0.5rem 0.6rem;
		background: none;
		border: none;
		color: #c9d1d9;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
		border-radius: 4px;
	}
	.search-result:hover { background: #1c2128; }
	.search-result.active { background: #7c3aed22; }

	.sr-name { font-size: 0.85rem; font-weight: 500; }
	.sr-meta { display: flex; gap: 0.4rem; font-size: 0.75rem; color: #484f58; margin-top: 2px; }
	.sr-vault { color: #7c3aed; flex-shrink: 0; }
	.sr-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.no-results { padding: 1rem; text-align: center; color: #484f58; font-size: 0.85rem; }

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
		border-bottom: 1px solid #21262d;
		background: #161b22;
		min-height: 38px;
	}

	.note-breadcrumb { font-size: 0.8rem; color: #8b949e; }
	.breadcrumb-vault { color: #7c3aed; }
	.breadcrumb-sep { margin: 0 0.3rem; color: #30363d; }
	.breadcrumb-name { color: #c9d1d9; }

	.close-note {
		background: none;
		border: none;
		color: #484f58;
		font-size: 1.3rem;
		cursor: pointer;
		padding: 0.2rem 0.4rem;
		line-height: 1;
		border-radius: 4px;
	}
	.close-note:hover { color: #e0e0e0; background: #21262d; }

	.note-content {
		flex: 1;
		overflow-y: auto;
		padding: 2rem 3rem;
		max-width: 800px;
		line-height: 1.75;
	}

	/* Markdown Styles */
	.note-content :global(h1) { font-size: 1.8rem; margin: 1.5rem 0 0.75rem; border-bottom: 1px solid #21262d; padding-bottom: 0.3rem; }
	.note-content :global(h2) { font-size: 1.4rem; margin: 1.3rem 0 0.5rem; }
	.note-content :global(h3) { font-size: 1.15rem; margin: 1rem 0 0.4rem; }
	.note-content :global(p) { margin: 0.5rem 0; }
	.note-content :global(a) { color: #7c3aed; }
	.note-content :global(code) {
		background: #1c2128;
		padding: 0.15em 0.35em;
		border-radius: 4px;
		font-size: 0.9em;
		color: #e0e0e0;
	}
	.note-content :global(pre) {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 6px;
		padding: 1rem;
		overflow-x: auto;
	}
	.note-content :global(pre code) { background: none; padding: 0; }
	.note-content :global(blockquote) {
		border-inline-start: 3px solid #7c3aed;
		padding: 0.25rem 1rem;
		margin: 0.5rem 0;
		color: #8b949e;
	}
	.note-content :global(ul), .note-content :global(ol) { padding-inline-start: 1.5rem; }
	.note-content :global(li) { margin: 0.2rem 0; }
	.note-content :global(hr) { border: none; border-top: 1px solid #21262d; margin: 1.5rem 0; }
	.note-content :global(table) { border-collapse: collapse; width: 100%; margin: 0.75rem 0; }
	.note-content :global(th), .note-content :global(td) {
		border: 1px solid #21262d;
		padding: 0.4rem 0.7rem;
		text-align: start;
	}
	.note-content :global(th) { background: #161b22; }
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
		color: #484f58;
	}

	.empty-icon { font-size: 3rem; margin-bottom: 1rem; }
	.empty-state h2 { color: #c9d1d9; font-size: 1.3rem; margin-bottom: 0.5rem; }
	.empty-state p { margin-bottom: 1.5rem; }

	.empty-add-btn {
		background: #7c3aed;
		border: none;
		color: white;
		padding: 0.6em 1.5em;
		border-radius: 8px;
		cursor: pointer;
		font-size: 0.95rem;
		font-weight: 600;
	}
	.empty-add-btn:hover { background: #6d28d9; }

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
		background: #161b22;
		border: 1px solid #21262d;
		padding: 0.4em 0.8em;
		border-radius: 20px;
		font-size: 0.85rem;
		color: #8b949e;
	}
	.chip-dot { width: 6px; height: 6px; border-radius: 50%; }
	.chip-count { color: #484f58; }
</style>

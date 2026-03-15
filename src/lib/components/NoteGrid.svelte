<script lang="ts">
	import { dir, t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';

	let {
		notes = [],
		libraryColorMap = {},
		onNoteClick,
		onNoteDoubleClick,
	}: {
		notes: { name: string; path: string; libraryName: string }[];
		libraryColorMap: Record<string, string>;
		onNoteClick: (note: { name: string; path: string; libraryName: string }) => void;
		onNoteDoubleClick?: (note: { name: string; path: string; libraryName: string }) => void;
	} = $props();

	let searchQuery = $state('');
	let sortBy = $state<'name' | 'vault' | 'modified'>('name');
	let selectedVault = $state<string>('all');

	// Get unique library names
	const libraryNames = $derived([...new Set(notes.map(n => n.libraryName))].sort());

	// Filter and sort
	const filteredNotes = $derived.by(() => {
		let result = notes;

		// Library filter
		if (selectedVault !== 'all') {
			result = result.filter(n => n.libraryName === selectedVault);
		}

		// Search filter
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			result = result.filter(n =>
				n.name.toLowerCase().includes(q) || n.path.toLowerCase().includes(q)
			);
		}

		// Sort
		result = [...result].sort((a, b) => {
			if (sortBy === 'name') return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
			if (sortBy === 'vault') return a.libraryName.localeCompare(b.libraryName) || a.name.localeCompare(b.name);
			return a.name.localeCompare(b.name); // fallback
		});

		return result;
	});

	// Extract folder from path
	function getFolder(path: string): string {
		const parts = path.replace(/\\/g, '/').split('/');
		if (parts.length <= 1) return '';
		return parts.slice(0, -1).join('/');
	}
</script>

<div class="note-grid-wrapper" dir={$dir}>
	<!-- Filter bar -->
	<div class="grid-toolbar">
		<div class="grid-search">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
			</svg>
			<input
				type="text"
				bind:value={searchQuery}
				placeholder={$t('secondScreen.searchNotes')}
				class="grid-search-input"
			/>
			{#if searchQuery}
				<button class="clear-btn" onclick={() => searchQuery = ''}>×</button>
			{/if}
		</div>

		<select class="grid-filter" bind:value={selectedVault}>
			<option value="all">{$t('secondScreen.allVaults')}</option>
			{#each libraryNames as vault}
				<option value={vault}>{vault}</option>
			{/each}
		</select>

		<select class="grid-sort" bind:value={sortBy}>
			<option value="name">{$t('secondScreen.sortName')}</option>
			<option value="vault">{$t('secondScreen.sortVault')}</option>
		</select>

		<span class="grid-count">{filteredNotes.length} / {notes.length}</span>
	</div>

	<!-- Card grid -->
	<div class="note-cards">
		{#each filteredNotes as note (note.path)}
			<button
				class="note-card"
				onclick={() => onNoteClick(note)}
				ondblclick={() => onNoteDoubleClick?.(note)}
			>
				<div class="card-header">
					<span class="card-dot" style="background:{libraryColorMap[note.libraryName] || '#7c3aed'}"></span>
					<span class="card-title">{note.name.replace(/\.md$/, '')}</span>
				</div>
				<div class="card-folder">{getFolder(note.path)}</div>
				<div class="card-library">{note.libraryName}</div>
			</button>
		{/each}

		{#if filteredNotes.length === 0}
			<div class="grid-empty">
				<p>{searchQuery ? $t('secondScreen.noResults') : $t('secondScreen.noNotes')}</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.note-grid-wrapper {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	/* ─── Toolbar ─── */
	.grid-toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}

	.grid-search {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: 1;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 0 8px;
	}

	.grid-search-input {
		flex: 1;
		border: none;
		background: transparent;
		padding: 6px 0;
		color: var(--text-normal);
		font-size: 13px;
		outline: none;
	}

	.clear-btn {
		border: none;
		background: none;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: 16px;
		padding: 0 2px;
	}

	.grid-filter, .grid-sort {
		padding: 5px 8px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 12px;
		cursor: pointer;
	}

	.grid-count {
		font-size: 11px;
		color: var(--text-muted);
		white-space: nowrap;
	}

	/* ─── Cards ─── */
	.note-cards {
		flex: 1;
		overflow-y: auto;
		padding: 12px;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 8px;
		align-content: start;
	}

	.note-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 10px 12px;
		background: var(--bg-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		cursor: pointer;
		text-align: start;
		transition: all 0.15s;
		min-height: 70px;
	}
	.note-card:hover {
		border-color: var(--interactive-accent);
		background: var(--bg-hover);
		transform: translateY(-1px);
		box-shadow: 0 2px 8px rgba(0,0,0,0.1);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.card-dot {
		width: 8px; height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.card-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-normal);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.card-folder {
		font-size: 11px;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding-inline-start: 14px;
	}

	.card-library {
		font-size: 10px;
		color: var(--text-secondary);
		padding-inline-start: 14px;
		margin-top: auto;
	}

	.grid-empty {
		grid-column: 1 / -1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 48px;
		color: var(--text-muted);
		font-size: 14px;
	}
</style>

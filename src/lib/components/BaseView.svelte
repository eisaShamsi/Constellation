<script lang="ts">
	import { onMount } from 'svelte';
	import { queryBase, saveBaseFile, updateNoteProperty } from '$lib/bases/store';
	import { createDefaultColumn, detectCellType, type BaseDefinition, type BaseQueryResult, type BaseRow, type ColumnDef, type BaseSource } from '$lib/bases/types';
	import { libraries } from '$lib/libraries/store';
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import BaseTableView from './BaseTableView.svelte';
	import BaseCardView from './BaseCardView.svelte';
	import BaseListView from './BaseListView.svelte';
	import BaseFilterBuilder from './BaseFilterBuilder.svelte';
	import BaseSortBuilder from './BaseSortBuilder.svelte';

	let {
		definition,
		filePath,
		onOpenNote,
		onCreateNote,
	}: {
		definition: BaseDefinition;
		filePath: string;
		onOpenNote: (path: string, libraryName: string) => void;
		onCreateNote?: (folderPath: string, properties: Record<string, string>) => void;
	} = $props();

	let result: BaseQueryResult | null = $state(null);
	let loading = $state(false);
	let error = $state('');
	let showFilters = $state(false);
	let showSorts = $state(false);
	let showSource = $state(false);

	// Source editing state
	let sourceType = $state(definition.source.type);
	let sourcePath = $state(definition.source.path ?? '');
	let sourceTag = $state(definition.source.tag ?? '');
	let sourceSubfolders = $state(definition.source.includeSubfolders ?? true);
	let selectedLibraryNames: string[] = $state(definition.source.selectedLibraries ?? []);

	// Available libraries for the picker
	const availableLibraries = $derived($libraries);

	// Whether all libraries are selected (empty = all)
	const allLibrariesSelected = $derived(selectedLibraryNames.length === 0);

	// Source description for display
	const sourceLabel = $derived.by(() => {
		const librarySuffix = selectedLibraryNames.length > 0
			? ` (${selectedLibraryNames.length})`
			: '';
		switch (definition.source.type) {
			case 'folder': return (definition.source.path || '/') + librarySuffix;
			case 'tag': return `#${definition.source.tag || ''}` + librarySuffix;
			case 'all': return selectedLibraryNames.length > 0
				? `${selectedLibraryNames.length} ${$t('bases.source.libraries')}`
				: $t('bases.source.allVaults');
			default: return '';
		}
	});

	const sourceIcon = $derived.by(() => {
		switch (definition.source.type) {
			case 'folder': return 'folder' as const;
			case 'tag': return 'tag' as const;
			case 'all': return 'all' as const;
			default: return 'all' as const;
		}
	});

	// Count unique libraries in results
	const libraryCount = $derived.by(() => {
		if (!result) return 0;
		const librarySet = new Set(result.rows.map(r => r.library_name));
		return librarySet.size;
	});

	// Effective columns: user-defined or auto-detected
	const effectiveColumns = $derived.by(() => {
		if (definition.columns.length > 0) {
			return definition.columns;
		}
		// Auto-detect from query results
		if (result?.columns_detected) {
			return result.columns_detected.map(prop => createDefaultColumn(prop));
		}
		return [];
	});

	// Determine direction
	const baseDir = $derived.by((): 'ltr' | 'rtl' => {
		if (definition.direction === 'rtl') return 'rtl';
		if (definition.direction === 'ltr') return 'ltr';
		// auto: detect from name
		return detectDir(definition.name || '') as 'ltr' | 'rtl';
	});

	async function runQuery() {
		loading = true;
		error = '';
		try {
			const libraryPaths: [string, string][] = $libraries.map(v => [v.name, v.path]);
			result = await queryBase(definition, libraryPaths);
		} catch (e: any) {
			error = e?.toString() ?? 'Query failed';
		}
		loading = false;
	}

	async function handleCellEdit(row: BaseRow, key: string, value: string) {
		try {
			await updateNoteProperty(row.file_path, key, value);
			// Update local state immediately for responsiveness
			row.properties[key] = value;
			result = result; // trigger reactivity
		} catch (e: any) {
			console.error('Failed to update property:', e);
		}
	}

	function setView(view: 'table' | 'card' | 'list') {
		definition.view = view;
		saveBaseFile(filePath, definition);
	}

	async function handleFiltersChange(filters: typeof definition.filters) {
		definition.filters = filters;
		await saveBaseFile(filePath, definition);
		await runQuery();
	}

	async function handleSortsChange(sorts: typeof definition.sorts) {
		definition.sorts = sorts;
		await saveBaseFile(filePath, definition);
		await runQuery();
	}

	async function handleColumnReorder(columns: ColumnDef[]) {
		definition.columns = columns;
		await saveBaseFile(filePath, definition);
	}

	function toggleLibrary(name: string) {
		if (selectedLibraryNames.includes(name)) {
			selectedLibraryNames = selectedLibraryNames.filter(v => v !== name);
		} else {
			selectedLibraryNames = [...selectedLibraryNames, name];
		}
	}

	function toggleAllLibraries() {
		if (allLibrariesSelected) {
			// Can't deselect "all" — do nothing (it's already all)
		} else {
			selectedLibraryNames = [];
		}
	}

	async function handleSourceChange() {
		const newSource: BaseSource = {
			type: sourceType,
			includeSubfolders: sourceSubfolders,
			selectedLibraries: selectedLibraryNames.length > 0 ? selectedLibraryNames : undefined,
		};
		if (sourceType === 'folder') newSource.path = sourcePath;
		if (sourceType === 'tag') newSource.tag = sourceTag;

		definition.source = newSource;
		await saveBaseFile(filePath, definition);
		showSource = false;
		await runQuery();
	}

	function handleNewNote() {
		if (!onCreateNote) return;
		const folderPath = definition.source.path ?? '';
		// Pre-fill properties from active filters
		const prefill: Record<string, string> = {};
		for (const f of definition.filters) {
			if (f.operator === 'is' && f.value) {
				prefill[f.property] = f.value;
			}
		}
		onCreateNote(folderPath, prefill);
	}

	onMount(() => {
		runQuery();
	});
</script>

<div class="base-view" dir={baseDir}>
	<!-- Toolbar -->
	<div class="base-toolbar">
		<div class="base-toolbar-start">
			<h2 class="base-name">{definition.name || $t('bases.untitled')}</h2>
			{#if result}
				<span class="base-count">
					{result.rows.length} / {result.total_count}
					{#if libraryCount > 1}
						<span class="library-count" title="{libraryCount} {$t('bases.source.libraries')}">
							· {libraryCount} {$t('bases.source.libraries')}
						</span>
					{/if}
				</span>
			{/if}
		</div>
		<div class="base-toolbar-end">
			<!-- Source config toggle -->
			<button class="toolbar-btn source-btn" class:active={showSource} onclick={() => showSource = !showSource} title={$t('bases.source.title')}>
				{#if sourceIcon === 'folder'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>
				{:else if sourceIcon === 'tag'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/><path d="M7 7h.01"/></svg>
				{:else if sourceIcon === 'all'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 20h20"/><path d="M5 20V8l7-5 7 5v12"/><path d="M9 20v-4h6v4"/></svg>
				{:else}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
				{/if}
				<span class="source-label">{sourceLabel}</span>
			</button>

			<!-- View switcher -->
			<div class="view-switcher">
				<button class="view-btn" class:active={definition.view === 'table'} onclick={() => setView('table')} title={$t('bases.table')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
				</button>
				<button class="view-btn" class:active={definition.view === 'card'} onclick={() => setView('card')} title={$t('bases.cards')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="8" height="8" rx="1"/><rect x="14" y="3" width="8" height="8" rx="1"/><rect x="2" y="13" width="8" height="8" rx="1"/><rect x="14" y="13" width="8" height="8" rx="1"/></svg>
				</button>
				<button class="view-btn" class:active={definition.view === 'list'} onclick={() => setView('list')} title={$t('bases.list')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
				</button>
			</div>

			<!-- Filter toggle -->
			<button class="toolbar-btn" class:active={showFilters} onclick={() => showFilters = !showFilters}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/></svg>
				{#if definition.filters.length > 0}<span class="badge">{definition.filters.length}</span>{/if}
			</button>

			<!-- Sort toggle -->
			<button class="toolbar-btn" class:active={showSorts} onclick={() => showSorts = !showSorts}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="4" y1="6" x2="13" y2="6"/><line x1="4" y1="12" x2="10" y2="12"/><line x1="4" y1="18" x2="7" y2="18"/><polyline points="15 9 18 6 21 9"/><line x1="18" y1="6" x2="18" y2="18"/></svg>
				{#if definition.sorts.length > 0}<span class="badge">{definition.sorts.length}</span>{/if}
			</button>

			<!-- New note -->
			{#if onCreateNote}
				<button class="toolbar-btn new-note-btn" onclick={handleNewNote}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
				</button>
			{/if}

			<!-- Refresh -->
			<button class="toolbar-btn" onclick={runQuery} title={$t('bases.refresh')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
			</button>
		</div>
	</div>

	<!-- Source configuration panel -->
	{#if showSource}
		<div class="source-panel">
			<div class="source-row">
				<!-- Left: source type + options -->
				<div class="source-left">
					<div class="source-section-label">{$t('bases.source.title')}</div>
					<div class="source-type-selector">
						<button class="source-type-btn" class:active={sourceType === 'folder'} onclick={() => sourceType = 'folder'}>
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>
							{$t('bases.source.folder')}
						</button>
						<button class="source-type-btn" class:active={sourceType === 'tag'} onclick={() => sourceType = 'tag'}>
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/><path d="M7 7h.01"/></svg>
							{$t('bases.source.tag')}
						</button>
						<button class="source-type-btn" class:active={sourceType === 'all'} onclick={() => sourceType = 'all'}>
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
							{$t('bases.source.allVaults')}
						</button>
					</div>

					{#if sourceType === 'folder'}
						<label class="source-field">
							<span>{$t('bases.source.folderPath')}</span>
							<input type="text" bind:value={sourcePath} placeholder={$t('bases.source.folderPlaceholder')} />
						</label>
						<label class="source-checkbox">
							<input type="checkbox" bind:checked={sourceSubfolders} />
							{$t('bases.source.includeSubfolders')}
						</label>
					{:else if sourceType === 'tag'}
						<label class="source-field">
							<span>{$t('bases.source.tagName')}</span>
							<input type="text" bind:value={sourceTag} placeholder={$t('bases.source.tagPlaceholder')} />
						</label>
					{/if}
				</div>

				<!-- Right: library checkboxes -->
				<div class="source-right">
					<div class="source-section-label">{$t('bases.source.vaultsLabel')}</div>
					<div class="library-checklist">
						<label class="library-check" class:active={allLibrariesSelected}>
							<input type="checkbox" checked={allLibrariesSelected} onchange={toggleAllLibraries} />
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
							<span>{$t('bases.source.allVaults')}</span>
						</label>
						{#each availableLibraries as v}
							<label class="library-check" class:active={selectedLibraryNames.includes(v.name)}>
								<input type="checkbox" checked={allLibrariesSelected || selectedLibraryNames.includes(v.name)} onchange={() => toggleLibrary(v.name)} />
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 20h20"/><path d="M5 20V8l7-5 7 5v12"/><path d="M9 20v-4h6v4"/></svg>
								<span>{v.name}</span>
							</label>
						{/each}
					</div>
				</div>
			</div>

			<div class="source-actions">
				<button class="source-apply" onclick={handleSourceChange}>{$t('bases.source.apply')}</button>
				<button class="source-cancel" onclick={() => { showSource = false; sourceType = definition.source.type; sourcePath = definition.source.path ?? ''; sourceTag = definition.source.tag ?? ''; sourceSubfolders = definition.source.includeSubfolders ?? true; selectedLibraryNames = definition.source.selectedLibraries ?? []; }}>{$t('bases.source.cancel')}</button>
			</div>
		</div>
	{/if}

	<!-- Filter builder -->
	{#if showFilters}
		<BaseFilterBuilder
			filters={definition.filters}
			availableProperties={['file_name', ...(result?.columns_detected ?? [])]}
			onchange={handleFiltersChange}
		/>
	{/if}

	<!-- Sort builder -->
	{#if showSorts}
		<BaseSortBuilder
			sorts={definition.sorts}
			availableProperties={['file_name', 'modified', ...(result?.columns_detected ?? [])]}
			onchange={handleSortsChange}
		/>
	{/if}

	<!-- Content -->
	<div class="base-content">
		{#if loading}
			<div class="base-loading">{$t('bases.loading')}</div>
		{:else if error}
			<div class="base-error">{error}</div>
		{:else if result && result.rows.length === 0}
			<div class="base-empty">
				<p>{$t('bases.noNotes')}</p>
				<p class="base-empty-hint">
					{#if definition.filters.length > 0}
						{$t('bases.adjustFilters')}
					{:else}
						{$t('bases.noFrontmatter')}
					{/if}
				</p>
			</div>
		{:else if result}
			{#if definition.view === 'table'}
				<BaseTableView
					rows={result.rows}
					columns={effectiveColumns}
					dir={baseDir}
					onCellEdit={handleCellEdit}
					onOpenNote={onOpenNote}
					onColumnReorder={handleColumnReorder}
				/>
			{:else if definition.view === 'card'}
				<BaseCardView
					rows={result.rows}
					columns={effectiveColumns}
					dir={baseDir}
					onOpenNote={onOpenNote}
				/>
			{:else if definition.view === 'list'}
				<BaseListView
					rows={result.rows}
					columns={effectiveColumns}
					dir={baseDir}
					onCellEdit={handleCellEdit}
					onOpenNote={onOpenNote}
				/>
			{/if}

			{#if result.query_time_ms > 0}
				<div class="base-footer">
					<span class="query-time">{result.rows.length} {$t('bases.resultsIn')} {result.query_time_ms}ms</span>
					{#if libraryCount > 1}
						<span class="query-libraries">· {libraryCount} {$t('bases.source.libraries')}</span>
					{/if}
				</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	.base-view {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		font-size: 0.88rem;
	}

	.base-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
		gap: 12px;
	}

	.base-toolbar-start {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.base-toolbar-end {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-shrink: 0;
	}

	.base-name {
		font-size: 1rem;
		font-weight: 600;
		margin: 0;
		color: var(--text-normal);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.base-count {
		font-size: 0.75rem;
		color: var(--text-muted);
		white-space: nowrap;
	}

	.library-count {
		color: var(--interactive-accent);
	}

	.view-switcher {
		display: flex;
		gap: 1px;
		background: var(--background-modifier-border);
		border-radius: 6px;
		overflow: hidden;
	}

	.view-btn {
		background: var(--background-primary);
		border: none;
		padding: 5px 8px;
		cursor: pointer;
		color: var(--text-muted);
		display: flex;
		align-items: center;
	}
	.view-btn:hover { color: var(--text-normal); }
	.view-btn.active {
		background: var(--interactive-accent);
		color: white;
	}

	.toolbar-btn {
		background: none;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 5px 8px;
		cursor: pointer;
		color: var(--text-muted);
		display: flex;
		align-items: center;
		gap: 4px;
		position: relative;
	}
	.toolbar-btn:hover { color: var(--text-normal); background: var(--background-secondary); }
	.toolbar-btn.active { color: var(--interactive-accent); border-color: var(--interactive-accent); }

	.source-btn {
		max-width: 200px;
	}
	.source-label {
		font-size: 0.75rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.badge {
		font-size: 0.65rem;
		background: var(--interactive-accent);
		color: white;
		border-radius: 50%;
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.new-note-btn {
		background: var(--interactive-accent);
		color: white;
		border-color: var(--interactive-accent);
	}
	.new-note-btn:hover { opacity: 0.9; background: var(--interactive-accent); color: white; }

	/* ─── Source Panel ─── */
	.source-panel {
		padding: 12px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		flex-shrink: 0;
	}

	.source-row {
		display: flex;
		gap: 20px;
		margin-bottom: 10px;
	}

	.source-left {
		flex: 1;
		min-width: 0;
	}

	.source-right {
		flex: 0 0 auto;
		min-width: 140px;
		max-width: 220px;
	}

	.source-section-label {
		font-size: 0.72rem;
		font-weight: 600;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-bottom: 6px;
	}

	.source-type-selector {
		display: flex;
		gap: 4px;
		margin-bottom: 10px;
		flex-wrap: wrap;
	}

	.source-type-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-muted);
		cursor: pointer;
		font-size: 0.8rem;
		transition: all 0.15s;
	}
	.source-type-btn:hover {
		color: var(--text-normal);
		border-color: var(--text-muted);
	}
	.source-type-btn.active {
		background: var(--interactive-accent);
		color: white;
		border-color: var(--interactive-accent);
	}

	.source-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-bottom: 6px;
	}
	.source-field span {
		font-size: 0.75rem;
		color: var(--text-muted);
	}
	.source-field input, .source-field select {
		padding: 5px 8px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 0.82rem;
	}

	.source-checkbox {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.8rem;
		color: var(--text-normal);
		cursor: pointer;
	}
	.source-checkbox input[type="checkbox"] {
		cursor: pointer;
	}

	/* Library checklist */
	.library-checklist {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 160px;
		overflow-y: auto;
	}

	.library-check {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 3px 6px;
		border-radius: 4px;
		font-size: 0.8rem;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.1s;
	}
	.library-check:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.library-check.active {
		color: var(--text-normal);
	}
	.library-check input[type="checkbox"] {
		cursor: pointer;
		accent-color: var(--interactive-accent);
	}
	.library-check span {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.source-actions {
		display: flex;
		gap: 6px;
	}
	.source-apply {
		padding: 4px 14px;
		border: none;
		border-radius: 4px;
		background: var(--interactive-accent);
		color: white;
		cursor: pointer;
		font-size: 0.8rem;
	}
	.source-apply:hover { opacity: 0.9; }
	.source-cancel {
		padding: 4px 14px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 0.8rem;
	}
	.source-cancel:hover { color: var(--text-normal); }

	.base-content {
		flex: 1;
		overflow: auto;
		min-height: 0;
	}

	.base-loading, .base-error, .base-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 200px;
		color: var(--text-muted);
		gap: 8px;
	}
	.base-error { color: var(--text-error, #e53e3e); }
	.base-empty-hint { font-size: 0.78rem; }

	.base-footer {
		padding: 6px 16px;
		border-top: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
		display: flex;
		gap: 6px;
	}
	.query-time {
		font-size: 0.72rem;
		color: var(--text-faint);
	}
	.query-libraries {
		font-size: 0.72rem;
		color: var(--interactive-accent);
	}
</style>

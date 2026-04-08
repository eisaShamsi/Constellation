<script lang="ts">
	import { t, dir } from '$lib/i18n';
	import {
		universalSearch,
		type UniversalSearchResponse,
		type ConstellationSearchResult
	} from '$lib/libraries/store';
	import { readSearchHistory, addSearchHistory, clearSearchHistory, relativeTime } from '$lib/libraries/searchHistory';

	let {
		initialQuery = '',
		allNotes = [] as { name: string; path: string; libraryName: string }[],
		onNoteClick = (_path: string, _name: string, _libraryName: string, _query: string) => {},
		onClose = () => {},
	} = $props();

	let query = $state(initialQuery);
	let response = $state<UniversalSearchResponse | null>(null);
	let loading = $state(false);
	let searchTimeout: ReturnType<typeof setTimeout>;
	let showHistory = $state(false);
	let history = $state(readSearchHistory());

	// Category collapse state
	let collapsed = $state<Record<string, boolean>>({});

	let searchInput: HTMLInputElement;

	const categories = ['titles', 'contents', 'tags', 'properties', 'wikilinks'] as const;

	const categoryIcons: Record<string, string> = {
		titles: 'T', contents: 'C', tags: '#', properties: 'P', wikilinks: 'W'
	};

	const categoryColors: Record<string, string> = {
		titles: '#3b82f6', contents: '#16a34a', tags: '#f472b6', properties: '#f59e0b', wikilinks: '#60a5fa'
	};

	let lastAppliedInitial = '';
	$effect(() => {
		// Only apply initialQuery if it changed and is non-empty (new search from outside)
		if (initialQuery && initialQuery !== lastAppliedInitial) {
			lastAppliedInitial = initialQuery;
			query = initialQuery;
			triggerSearch(initialQuery);
		}
	});

	function triggerSearch(q: string) {
		clearTimeout(searchTimeout);
		if (!q.trim()) { response = null; return; }
		loading = true;
		searchTimeout = setTimeout(async () => {
			try {
				response = await universalSearch(q, 15);
				addSearchHistory(q);
				history = readSearchHistory();
			} catch { response = null; }
			loading = false;
		}, 300);
	}

	function handleInput(e: Event) {
		query = (e.target as HTMLInputElement).value;
		showHistory = false;
		triggerSearch(query);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { onClose(); return; }
	}

	function handleResultClick(r: ConstellationSearchResult) {
		onNoteClick(r.path, r.name, r.library_name, query);
	}

	function selectHistory(q: string) {
		query = q;
		showHistory = false;
		triggerSearch(q);
		requestAnimationFrame(() => searchInput?.focus());
	}

	function toggleCategory(cat: string) {
		collapsed[cat] = !collapsed[cat];
	}

	function getCategoryResults(cat: string): ConstellationSearchResult[] {
		if (!response) return [];
		return (response as any)[cat] ?? [];
	}

	function totalResults(): number {
		if (!response) return 0;
		return categories.reduce((sum, c) => sum + getCategoryResults(c).length, 0);
	}

	function formatSnippetForCategory(cat: string, r: ConstellationSearchResult): string {
		if (cat === 'tags' && r.snippet) {
			try {
				const tags = JSON.parse(r.snippet) as string[];
				return tags.map(t => `#${t}`).join('  ');
			} catch { return r.snippet; }
		}
		if (cat === 'properties' && r.snippet) {
			try {
				const props = JSON.parse(r.snippet) as Record<string, string>;
				return Object.entries(props).map(([k, v]) => `${k}: ${v}`).join('  ·  ');
			} catch { return r.snippet; }
		}
		if (cat === 'wikilinks' && r.snippet) {
			try {
				const links = JSON.parse(r.snippet) as string[];
				return links.map(l => `→ [[${l}]]`).join('  ');
			} catch { return r.snippet; }
		}
		return r.snippet ?? '';
	}

	function matchBadgeKey(type: string): string {
		const cap = type.charAt(0).toUpperCase() + type.slice(1);
		return `sidebar.match${cap}`;
	}

	function highlightInText(text: string, cssClass: string = ''): string {
		if (!query || query.length < 2) return escapeHtml(text);
		const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
		const re = new RegExp(`(${escaped})`, 'gi');
		const tag = cssClass ? `<mark class="${cssClass}">$1</mark>` : '<mark>$1</mark>';
		return escapeHtml(text).replace(re, tag);
	}

	function escapeHtml(s: string): string {
		return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
	}
</script>

<div class="sh-root" dir={$dir}>
	<!-- Header bar with search -->
	<div class="sh-header">
		<svg class="sh-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
		<input bind:this={searchInput} class="sh-input" type="text"
			placeholder={$t('sidebar.searchPlaceholder')}
			value={query} oninput={handleInput} onkeydown={handleKeydown}
			onfocus={() => { if (!query) showHistory = true; }}
			onblur={() => setTimeout(() => showHistory = false, 200)} />
		{#if query}
			<button class="sh-clear" onclick={() => { query = ''; response = null; previewContent = ''; previewName = ''; }}>×</button>
		{/if}
		{#if response}
			<span class="sh-total">{totalResults()}</span>
		{/if}
		<button class="sh-close" onclick={onClose}>×</button>
	</div>

	<!-- History dropdown -->
	{#if showHistory && !query && history.length > 0}
		<div class="sh-history-drop">
			<div class="sh-section-label">{$t('sidebar.searchHistory')}</div>
			{#each history.slice(0, 10) as entry}
				<button class="sh-hist-item" onclick={() => selectHistory(entry.query)}>
					<span dir="auto">{entry.query}</span>
					<span class="sh-hist-time">{relativeTime(entry.timestamp)}</span>
				</button>
			{/each}
			<button class="sh-hist-clear" onclick={() => { clearSearchHistory(); history = []; }}>
				{$t('sidebar.clearHistory')}
			</button>
		</div>
	{/if}

	<div class="sh-body">
		<div class="sh-results-area">
			{#if loading}
				<div class="sh-loading">...</div>
			{:else if response}
				{#each categories as cat}
					{@const items = getCategoryResults(cat)}
					{#if items.length > 0}
						<div class="sh-category">
							<button class="sh-cat-header" onclick={() => toggleCategory(cat)}>
								<svg class="sh-chevron" class:sh-collapsed={collapsed[cat]} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
								<span class="sh-cat-badge" style:background={categoryColors[cat]}>{categoryIcons[cat]}</span>
								<span class="sh-cat-name">{$t(`searchHub.${cat}`)}</span>
								<span class="sh-cat-count">{items.length}</span>
							</button>
							{#if !collapsed[cat]}
								<div class="sh-cat-items">
									{#each items as r}
										<button class="sh-item"
											onclick={() => handleResultClick(r)}>
											<div class="sh-item-top">
												<span class="sh-item-name" dir="auto">{@html highlightInText(r.name)}</span>
												<span class="sh-item-lib">{r.library_name}</span>
											</div>
											{#if r.snippet}
												<div class="sh-item-snippet" dir="auto">
													{#if cat === 'contents'}
														{@html r.snippet}
													{:else if cat === 'tags'}
														{@html highlightInText(formatSnippetForCategory(cat, r), 'sh-hl-tag')}
													{:else}
														{@html highlightInText(formatSnippetForCategory(cat, r))}
													{/if}
												</div>
											{/if}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				{/each}
				{#if totalResults() === 0}
					<div class="sh-empty">{$t('sidebar.noResults')}</div>
				{/if}
			{:else if !query}
				<div class="sh-empty">{$t('searchHub.preview')}</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.sh-root {
		display: flex; flex-direction: column; height: 100%; width: 100%;
		background: var(--background-primary, #fff); color: var(--text);
	}

	/* Header */
	.sh-header {
		display: flex; align-items: center; gap: 8px; padding: 8px 16px;
		border-bottom: 1px solid var(--border); flex-shrink: 0;
	}
	.sh-icon { color: var(--text-muted); flex-shrink: 0; }
	.sh-input {
		flex: 1; border: none; background: none; padding: 6px 0;
		font-size: 1rem; color: var(--text); font-family: inherit; outline: none;
		min-width: 0;
	}
	.sh-input::placeholder { color: var(--text-faint); }
	.sh-clear, .sh-close {
		border: none; background: none; color: var(--text-muted); cursor: pointer;
		font-size: 1.1rem; padding: 2px 6px; border-radius: 4px;
	}
	.sh-clear:hover, .sh-close:hover { background: var(--bg-hover); color: var(--text); }
	.sh-total {
		font-size: 0.72rem; color: var(--interactive-accent); font-weight: 600;
		background: color-mix(in srgb, var(--interactive-accent) 12%, transparent);
		padding: 2px 8px; border-radius: 10px;
	}

	/* History dropdown */
	.sh-history-drop {
		border-bottom: 1px solid var(--border); padding: 4px 0; max-height: 300px; overflow-y: auto;
	}
	.sh-section-label {
		padding: 4px 16px; font-size: 0.7rem; color: var(--text-muted);
		text-transform: uppercase; letter-spacing: 0.04em;
	}
	.sh-hist-item {
		display: flex; align-items: center; gap: 4px; width: 100%; padding: 4px 16px;
		background: none; border: none; color: var(--text); font-family: inherit;
		cursor: pointer; text-align: start; font-size: 0.82rem;
	}
	.sh-hist-item:hover { background: var(--bg-hover); }
	.sh-hist-item span:first-child { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.sh-hist-time { font-size: 0.65rem; color: var(--text-faint); flex-shrink: 0; }
	.sh-hist-clear {
		display: block; width: 100%; padding: 3px 16px; background: none; border: none;
		color: var(--text-faint); font-size: 0.68rem; cursor: pointer; text-align: start; font-family: inherit;
	}
	.sh-hist-clear:hover { text-decoration: underline; }

	/* Body */
	.sh-body { flex: 1; display: flex; flex-direction: column; min-height: 0; }

	/* Results area — fills entire body */
	.sh-results-area { flex: 1; overflow-y: auto; }

	/* Category */
	.sh-category { border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent); }
	.sh-cat-header {
		display: flex; align-items: center; gap: 6px; width: 100%; padding: 6px 16px;
		background: var(--background-secondary, var(--bg)); border: none; color: var(--text);
		font-family: inherit; cursor: pointer; text-align: start; font-size: 0.82rem;
		position: sticky; top: 0; z-index: 1;
	}
	.sh-cat-header:hover { background: var(--bg-hover); }
	.sh-chevron { transition: transform 0.15s; flex-shrink: 0; color: var(--text-muted); }
	.sh-chevron.sh-collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .sh-chevron.sh-collapsed { transform: rotate(90deg); }
	.sh-cat-badge {
		min-width: 16px; height: 16px; border-radius: 3px;
		font-size: 10px; font-weight: 700; line-height: 16px; text-align: center;
		color: #fff; display: inline-block; flex-shrink: 0;
	}
	.sh-cat-name { font-weight: 600; flex: 1; }
	.sh-cat-count {
		font-size: 0.7rem; color: var(--text-muted); background: var(--bg);
		padding: 1px 6px; border-radius: 8px; flex-shrink: 0;
	}

	/* Items */
	.sh-cat-items { }
	.sh-item {
		display: block; width: 100%; padding: 4px 16px 4px 40px;
		background: none; border: none; color: var(--text); font-family: inherit;
		cursor: pointer; text-align: start;
	}
	.sh-item:hover { background: var(--bg-hover); }
	.sh-item.sh-item-active { background: var(--accent-bg); }
	.sh-item-top { display: flex; align-items: center; gap: 6px; }
	.sh-item-name { font-size: 0.82rem; font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.sh-item-lib { font-size: 0.68rem; color: var(--accent); flex-shrink: 0; }
	.sh-item-snippet {
		font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap; line-height: 1.4;
	}
	.sh-item-snippet :global(mark) {
		background: color-mix(in srgb, var(--interactive-accent) 25%, transparent);
		color: var(--text-normal); border-radius: 2px; padding: 0 1px;
	}
	.sh-item-snippet :global(mark.sh-hl-tag) {
		background: color-mix(in srgb, #f472b6 30%, transparent);
		color: var(--text-normal);
	}
	.sh-item-name :global(mark) {
		background: color-mix(in srgb, var(--interactive-accent) 25%, transparent);
		color: var(--text-normal); border-radius: 2px; padding: 0 1px;
	}
	.sh-empty, .sh-loading {
		padding: 40px; text-align: center; color: var(--text-faint); font-size: 0.88rem;
	}
</style>

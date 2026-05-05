<script lang="ts">
	import { t, dir, getSearchOps } from '$lib/i18n';
	import {
		universalSearch, appSettings, embedText, constellationSearch, parseSearchQuery,
		canonicalizeSearchQuery, hasAdvancedSyntaxMultilingual, stripInvisibleChars,
		ctseSearchByConcept,
		type UniversalSearchResponse,
		type ConstellationSearchResult,
		type CtseConceptHit
	} from '$lib/libraries/store';
	import { readSearchHistory, addSearchHistory, clearSearchHistory, relativeTime } from '$lib/libraries/searchHistory';
	import { detectDir } from '$lib/utils';

	let {
		initialQuery = '',
		allNotes = [] as { name: string; path: string; libraryName: string }[],
		linkCounts = new Map<string, { incoming: number }>(),
		onNoteClick = (_path: string, _name: string, _libraryName: string, _query: string) => {},
		onClose = () => {},
		onResults = (_matchIds: Set<string>) => {},
	} = $props();

	let query = $state(initialQuery);
	let response = $state<UniversalSearchResponse | null>(null);
	let filteredResults = $state<ConstellationSearchResult[]>([]);
	let isAdvancedMode = $state(false);
	let advancedGroups = $state<{query: string; results: ConstellationSearchResult[]}[]>([]);
	let selectedResultIdx = $state(-1);
	let loading = $state(false);
	let searchTimeout: ReturnType<typeof setTimeout>;
	let showHistory = $state(false);
	let showChips = $state(false);
	let history = $state(readSearchHistory());

	// Wikilink autocomplete
	let wikiAuto = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let wikiAutoIdx = $state(-1);

	// Category collapse state
	let collapsed = $state<Record<string, boolean>>({});

	let searchInput: HTMLInputElement;

	// MIG-013 \u00a71D \u2014 `concept` is the cross-language semantic category
	// powered by the CTSE Bridge Adapter. Embeds the query, finds the
	// nearest M11 concepts, joins term_vocab to get every script's
	// equivalent terms, FTS5 MATCHes them, returns notes regardless of
	// the language they're written in. The "semantic" category remains
	// for note-level cosine over `note_embeddings` (different feature,
	// kept for users who have it populated).
	const categories = ['titles', 'contents', 'tags', 'properties', 'wikilinks', 'semantic', 'concept'] as const;

	const categoryIcons = $derived.by(() => {
		const b = (key: string, fallback: string) => $t(`searchBadges.${key}`) !== `searchBadges.${key}` ? $t(`searchBadges.${key}`) : fallback;
		return {
			titles: b('title', 'T'), contents: b('content', 'C'), tags: '#', properties: b('property', 'P'), wikilinks: b('wikilink', 'W'), semantic: b('semantic', 'S'), concept: b('concept', '\u2248'),
			title: b('title', 'T'), content: b('content', 'C'), tag: '#', property: b('property', 'P'), wikilink: b('wikilink', 'W'), structured: '0\u0336'
		} as Record<string, string>;
	});

	const categoryColors: Record<string, string> = {
		titles: '#3b82f6', contents: '#16a34a', tags: '#f472b6', properties: '#f59e0b', wikilinks: '#60a5fa', semantic: '#7c3aed', concept: '#0891b2',
		// match_type values from advanced/structured search
		title: '#3b82f6', content: '#16a34a', tag: '#f472b6', property: '#f59e0b', wikilink: '#60a5fa', structured: '#ef4444'
	};

	const syntaxChips = $derived.by(() => {
		const _localeTracker = $t('searchHub.linksTo'); // reactive dependency on locale
		const ops = getSearchOps();
		return [
			{ label: 'linksTo', syntax: (ops?.linksTo ?? 'links to') + ' [[' },
			{ label: 'linksFrom', syntax: (ops?.linksFrom ?? 'links from') + ' [[' },
			{ label: 'mutual', syntax: (ops?.mutual ?? 'mutual') + ' [[' },
			{ label: 'mentions', syntax: (ops?.mentions ?? 'mentions') + ' [[' },
			{ label: 'orphans', syntax: ops?.orphans ?? 'orphans' },
			{ label: 'linksBetween', syntax: (ops?.linksBetween ?? 'links between') + ' [[' },
			{ label: 'linksAll', syntax: (ops?.linksAll ?? 'links all') + ' [[' },
			// Cognitive typed link operators
			{ label: 'supports', syntax: (ops?.supports ?? 'supports') + ' [[' },
			{ label: 'contradicts', syntax: (ops?.contradicts ?? 'contradicts') + ' [[' },
			{ label: 'causes', syntax: (ops?.causes ?? 'causes') + ' [[' },
			{ label: 'exemplifies', syntax: (ops?.exemplifies ?? 'exemplifies') + ' [[' },
			{ label: 'generalizes', syntax: (ops?.generalizes ?? 'generalizes') + ' [[' },
			{ label: 'derivesFrom', syntax: (ops?.derivesFrom ?? 'derives from') + ' [[' },
			{ label: 'partOf', syntax: (ops?.partOf ?? 'part of') + ' [[' },
			// Structural
			{ label: 'tag', syntax: '#' },
			{ label: 'property', syntax: 'key=value' },
			{ label: 'scope', syntax: (ops?.scope ?? 'in') + ':' },
		];
	});

	/** Detect if query uses advanced syntax (in English or current locale) */
	function hasAdvancedSyntax(q: string): boolean {
		return hasAdvancedSyntaxMultilingual(q, getSearchOps());
	}

	/** Detect link direction from query for arrow display (handles localized operators) */
	function queryDirection(q: string): '↑' | '↓' | '↑↓' | null {
		const cq = canonicalizeSearchQuery(q, getSearchOps());
		if (/links?\s+all\s/i.test(cq)) return null; // per-result from Rust
		if (/mutual\s/i.test(cq)) return '↑↓';
		if (/links?\s+to\s/i.test(cq)) return '↑';
		if (/links?\s+from\s/i.test(cq)) return '↓';
		if (/links?\s+between\s/i.test(cq)) return '↑';
		return null;
	}

	let lastAppliedInitial = '';
	$effect(() => {
		if (initialQuery && initialQuery !== lastAppliedInitial) {
			lastAppliedInitial = initialQuery;
			query = initialQuery;
			triggerSearch(initialQuery);
		}
	});

	function triggerSearch(q: string) {
		clearTimeout(searchTimeout);
		// q is already clean — stripInvisibleChars applied in handleInput
		if (!q.trim()) { response = null; filteredResults = []; isAdvancedMode = false; return; }
		loading = true;
		searchTimeout = setTimeout(async () => {
			try {
				if (hasAdvancedSyntax(q)) {
					// Advanced mode: split by commas, parse each sub-query
					isAdvancedMode = true;
					response = null;
					const ops = getSearchOps();
					const canonicalized = canonicalizeSearchQuery(q, ops);
					const subQueries = canonicalized.split(/[,،、]/).map(s => s.trim()).filter(s => s.length > 0);
					if (subQueries.length > 1) {
						// Multiple sub-queries: group results by query
						advancedGroups = [];
						for (const sub of subQueries) {
							const req = parseSearchQuery(sub);
							const results = await constellationSearch(req);
							advancedGroups.push({ query: sub, results });
						}
						filteredResults = [];
					} else {
						// Single advanced query: flat list
						advancedGroups = [];
						const req = parseSearchQuery(canonicalized);
						const raw = await constellationSearch(req);
					filteredResults = raw.sort((a, b) => {
						const sd = b.score - a.score;
						if (Math.abs(sd) > 0.001) return sd;
						return (linkCounts.get(b.name.toLowerCase())?.incoming ?? 0) - (linkCounts.get(a.name.toLowerCase())?.incoming ?? 0);
					});
					}
				} else {
					// Universal mode: search everywhere, categorize results
					isAdvancedMode = false;
					filteredResults = [];
					let qEmbed: number[] | null = null;
					if ($appSettings.enabledFeatures?.semanticSearch) {
						try { qEmbed = await embedText(q); } catch {}
					}
					// MIG-013 §1D — fire the universal search and the CTSE
					// concept search in parallel; merge concept hits into
					// the response under a `concept` key. CTSE failures
					// (engine init failure, empty term_vocab) degrade
					// gracefully — the rest of the categories still render.
					const [universal, conceptHits] = await Promise.all([
						universalSearch(q, qEmbed, 0),
						ctseSearchByConcept(q).catch((err) => {
							console.warn('[SearchHub] ctseSearchByConcept failed:', err);
							return [] as CtseConceptHit[];
						}),
					]);
					response = universal;
					// Map CTSE hits into the same shape as the other
					// categories. score 1.0 is a placeholder — concept-
					// based ranking is by FTS5 rank server-side, so the
					// row order already reflects relevance.
					const conceptResults: ConstellationSearchResult[] = conceptHits.map((h) => ({
						name: h.name,
						path: h.path,
						library_name: h.library_name,
						score: 1.0,
						match_type: 'concept',
						snippet: h.snippet ?? undefined,
						modified: 0,
					}));
					(response as unknown as Record<string, ConstellationSearchResult[]>).concept = conceptResults;
				}
				addSearchHistory(q);
				history = readSearchHistory();
				// Emit matched note IDs for graph view highlighting
				const ids = new Set<string>();
				if (isAdvancedMode) {
					filteredResults.forEach(r => ids.add(r.name.toLowerCase()));
					advancedGroups.forEach(g => g.results.forEach(r => ids.add(r.name.toLowerCase())));
				} else if (response) {
					for (const cat of categories) {
						((response as any)[cat] ?? []).forEach((r: ConstellationSearchResult) => ids.add(r.name.toLowerCase()));
					}
				}
				onResults(ids);
				selectedResultIdx = -1;
			} catch { response = null; filteredResults = []; }
			loading = false;
		}, 300);
	}

	function handleInput(e: Event) {
		query = stripInvisibleChars((e.target as HTMLInputElement).value);
		showHistory = false;

		// Wikilink autocomplete detection — works with both English and localized operators
		const canonicalized = canonicalizeSearchQuery(query, getSearchOps());
		const wikiMatch = canonicalized.match(/(?:links?\s+(?:to|from|between|all)|mutual|mentions?)\s+(?:.*\[\[(?:[^\]]+\]\]\s+and\s+)?)?\[\[([^\]]*)$/i);
		if (wikiMatch) {
			const partial = wikiMatch[1].toLowerCase();
			wikiAuto = allNotes
				.filter(n => !partial || n.name.toLowerCase().includes(partial))
				.sort((a, b) => (linkCounts.get(b.name.toLowerCase())?.incoming ?? 0) - (linkCounts.get(a.name.toLowerCase())?.incoming ?? 0))
				.slice(0, 20);
			wikiAutoIdx = -1;
			return; // Don't trigger search while composing wikilink
		}
		wikiAuto = [];
		triggerSearch(query);
	}

	function handleKeydown(e: KeyboardEvent) {
		// Wikilink autocomplete navigation
		if (wikiAuto.length > 0) {
			if (e.key === 'ArrowDown') { e.preventDefault(); wikiAutoIdx = Math.min(wikiAutoIdx + 1, wikiAuto.length - 1); }
			else if (e.key === 'ArrowUp') { e.preventDefault(); wikiAutoIdx = Math.max(wikiAutoIdx - 1, 0); }
			else if (e.key === 'Enter') {
				e.preventDefault();
				insertWikiName(wikiAuto[Math.max(wikiAutoIdx, 0)].name);
			}
			else if (e.key === 'Escape') { e.preventDefault(); wikiAuto = []; }
			return;
		}
		if (e.key === 'Escape') { onClose(); return; }
		// No auto-bracket pairing in search inputs — causes extra brackets in RTL/bidi contexts
		// Result navigation
		if (allFlatResults.length > 0) {
			if (e.key === 'ArrowDown') {
				e.preventDefault();
				selectedResultIdx = Math.min(selectedResultIdx + 1, allFlatResults.length - 1);
				scrollSelectedIntoView();
			} else if (e.key === 'ArrowUp') {
				e.preventDefault();
				selectedResultIdx = Math.max(selectedResultIdx - 1, 0);
				scrollSelectedIntoView();
			} else if (e.key === 'Enter' && selectedResultIdx >= 0) {
				e.preventDefault();
				handleResultClick(allFlatResults[selectedResultIdx]);
			}
		}
	}

	function scrollSelectedIntoView() {
		requestAnimationFrame(() => {
			const el = document.querySelector('.sh-item.sh-item-selected');
			el?.scrollIntoView({ block: 'nearest' });
		});
	}

	function insertWikiName(name: string) {
		query = query.replace(/\[\[[^\]]*$/, `[[${name}]]`);
		wikiAuto = [];
		wikiAutoIdx = -1;
		triggerSearch(query);
		requestAnimationFrame(() => searchInput?.focus());
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

	function insertSyntax(syntax: string) {
		query = query ? query + ' ' + syntax : syntax;
		showChips = false;
		requestAnimationFrame(() => {
			searchInput?.focus();
			// Move cursor to end of input
			if (searchInput) {
				searchInput.selectionStart = searchInput.selectionEnd = searchInput.value.length;
			}
			if (syntax.endsWith('[[')) {
				wikiAuto = [...allNotes].sort((a, b) => (linkCounts.get(b.name.toLowerCase())?.incoming ?? 0) - (linkCounts.get(a.name.toLowerCase())?.incoming ?? 0)).slice(0, 20);
				wikiAutoIdx = -1;
			} else if (syntax === 'orphans') {
				triggerSearch(query);
			}
		});
	}

	/** Flat list of all results for keyboard navigation */
	const allFlatResults = $derived.by(() => {
		if (isAdvancedMode) return filteredResults;
		if (!response) return [];
		const flat: ConstellationSearchResult[] = [];
		for (const cat of categories) {
			const items = (response as any)[cat] ?? [];
			if (!collapsed[cat]) flat.push(...items);
		}
		return flat;
	});

	function toggleCategory(cat: string) {
		collapsed[cat] = !collapsed[cat];
	}

	function getCategoryResults(cat: string): ConstellationSearchResult[] {
		if (!response) return [];
		return (response as any)[cat] ?? [];
	}

	function totalResults(): number {
		if (isAdvancedMode) {
			if (advancedGroups.length > 0) return advancedGroups.reduce((sum, g) => sum + g.results.length, 0);
			return filteredResults.length;
		}
		if (!response) return 0;
		return categories.reduce((sum, c) => sum + getCategoryResults(c).length, 0);
	}

	function uniqueNoteCount(): number {
		const paths = new Set<string>();
		if (isAdvancedMode) {
			if (advancedGroups.length > 0) {
				for (const g of advancedGroups) for (const r of g.results) paths.add(r.path);
			} else {
				for (const r of filteredResults) paths.add(r.path);
			}
		} else if (response) {
			for (const cat of categories) {
				for (const r of getCategoryResults(cat)) paths.add(r.path);
			}
		}
		return paths.size;
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

	/** Split query by comma variants for multi-term highlight */
	function getSearchTerms(): string[] {
		// For advanced mode, extract the actual search words (strip operators)
		const raw = query.replace(/links?\s+(to|from|between)\s+\[\[[^\]]*\]\]/gi, '')
			.replace(/mutual\s+\[\[[^\]]*\]\]/gi, '')
			.replace(/mentions?\s+\[\[[^\]]*\]\]/gi, '')
			.replace(/orphans?/gi, '')
			.replace(/in:\S+/gi, '')
			.replace(/#(\S+)/g, '$1')
			.replace(/\S+=\S+/g, '');
		return raw.split(/[,،、\s]+/).map(s => s.trim()).filter(s => s.length > 1);
	}

	function highlightInText(text: string, cssClass: string = ''): string {
		const terms = getSearchTerms();
		if (!terms.length) return escapeHtml(text);
		const escaped = terms.map(t => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|');
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
	<div class="sh-header-wrap">
		<div class="sh-header">
			<svg class="sh-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			<input bind:this={searchInput} class="sh-input" type="text" dir="auto"
				placeholder={$t('sidebar.searchPlaceholder')}
				value={query} oninput={handleInput} onkeydown={handleKeydown}
				onfocus={() => { if (!query) showHistory = true; }}
				onblur={() => setTimeout(() => { showHistory = false; }, 200)} />
			{#if query}
				<button class="sh-clear" onclick={() => { query = ''; response = null; filteredResults = []; isAdvancedMode = false; }}>×</button>
			{/if}
			<button class="sh-chips-toggle" class:active={showChips} onclick={() => showChips = !showChips} title={$t('searchHub.syntaxHelpers')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
			</button>
			{#if totalResults() > 0}
				{@const unique = uniqueNoteCount()}
				<span class="sh-total">{totalResults()} {#if unique < totalResults()}<span class="sh-unique">{$t('searchHub.from')} {unique} {$t('searchHub.notes')}</span>{/if}</span>
			{/if}
			<button class="sh-close" onclick={() => onClose()}>×</button>
		</div>

		<!-- Wikilink autocomplete dropdown (absolute overlay below header) -->
		{#if wikiAuto.length > 0}
			<div class="sh-wiki-drop">
				{#each wikiAuto as note, idx}
					{@const counts = linkCounts.get(note.name.toLowerCase())}
					{@const incoming = counts?.incoming ?? 0}
					<button class="sh-wa-item" class:selected={idx === wikiAutoIdx}
						onclick={() => insertWikiName(note.name)} dir={detectDir(note.name)}>
						<span class="sh-wa-name">{note.name}</span>
						<span class="sh-wa-links" class:sh-wa-zero={incoming === 0}>
							<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>
							{incoming}
						</span>
						<span class="sh-wa-lib">{note.libraryName}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Syntax helper chips -->
	{#if showChips}
		<div class="sh-chips-bar">
			{#each syntaxChips as chip}
				<button class="sh-chip" onclick={() => insertSyntax(chip.syntax)}>
					{$t(`searchHub.${chip.label}`)}
				</button>
			{/each}
		</div>
	{/if}

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

			{:else if isAdvancedMode && advancedGroups.length > 0}
				<!-- Advanced mode: grouped by sub-query -->
				{#each advancedGroups as group}
					{#if group.results.length > 0}
						<div class="sh-category">
							<button class="sh-cat-header" dir={detectDir(group.query)} onclick={() => toggleCategory(group.query)}>
								<svg class="sh-chevron" class:sh-collapsed={collapsed[group.query]} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
								<span class="sh-cat-badge" style:background={categoryColors[group.results[0]?.match_type] ?? '#94a3b8'}>{categoryIcons[group.results[0]?.match_type] ?? '?'}</span>
								<span class="sh-cat-name">{group.query}</span>
								<span class="sh-cat-count">{group.results.length}</span>
							</button>
							{#if !collapsed[group.query]}
								<div class="sh-cat-items">
									{#each group.results as r}
										<button class="sh-item" onclick={() => handleResultClick(r)} dir={detectDir(r.name)}>
											<div class="sh-item-top">
												<span class="sh-item-name">{@html highlightInText(r.name)}</span>
												{#if r.match_via}
													<span class="sh-match-via" dir={detectDir(r.match_via)} title="{$t('searchHub.matchVia')} {r.match_via}">
														{$t('searchHub.matchVia')} {r.match_via}
													</span>
												{/if}
												<span class="sh-item-lib">{r.library_name}</span>
											</div>
											{#if r.snippet}
												<div class="sh-item-snippet">{@html highlightInText(r.snippet)}</div>
											{/if}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				{/each}

			{:else if isAdvancedMode && filteredResults.length > 0}
				<!-- Advanced mode: single query, flat result list -->
				<div class="sh-section-label">{filteredResults.length} {$t('sidebar.results')}</div>
				{#each filteredResults as r, idx}
					{@const rustDir = r.snippet === '↑' || r.snippet === '↓' || r.snippet === '↑↓' ? r.snippet : null}
					{@const dir = rustDir ?? queryDirection(query)}
					{@const rCounts = linkCounts.get(r.name.toLowerCase())}
					{@const rIncoming = rCounts?.incoming ?? 0}
					<button class="sh-item" style="padding-inline-start:16px" class:sh-item-selected={idx === selectedResultIdx} onclick={() => handleResultClick(r)} dir={detectDir(r.name)}>
						<div class="sh-item-top">
							{#if dir}
								{#if dir === '↑' || dir === '↑↓'}
									<span class="sh-dir-arrow sh-dir-in" title="Incoming">▲</span>
								{/if}
								{#if dir === '↓' || dir === '↑↓'}
									<span class="sh-dir-arrow sh-dir-out" title="Outgoing">▼</span>
								{/if}
							{/if}
							{#if r.match_type}
								<span class="sh-cat-badge" style:background={categoryColors[r.match_type] ?? '#94a3b8'}>{categoryIcons[r.match_type] ?? '?'}</span>
							{/if}
							<span class="sh-item-name">{@html highlightInText(r.name)}</span>
							{#if r.match_via}
								<span class="sh-match-via" dir={detectDir(r.match_via)} title="{$t('searchHub.matchVia')} {r.match_via}">
									{$t('searchHub.matchVia')} {r.match_via}
								</span>
							{/if}
							<span class="sh-wa-links" class:sh-wa-zero={rIncoming === 0}>
								<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>
								{rIncoming}
							</span>
							<span class="sh-item-lib">{r.library_name}</span>
						</div>
						{#if r.snippet && !rustDir}
							<div class="sh-item-snippet">{@html highlightInText(r.snippet)}</div>
						{/if}
					</button>
				{/each}

			{:else if isAdvancedMode && filteredResults.length === 0 && advancedGroups.length === 0}
				<div class="sh-empty">{$t('sidebar.noResults')}</div>

			{:else if response}
				<!-- Universal mode: categorized results -->
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
										<button class="sh-item" class:sh-item-selected={selectedResultIdx >= 0 && allFlatResults[selectedResultIdx] === r}
											onclick={() => handleResultClick(r)} dir={detectDir(r.name)}>
											<div class="sh-item-top">
												<span class="sh-item-name">{@html highlightInText(r.name)}</span>
												{#if r.match_via}
													<span class="sh-match-via" dir={detectDir(r.match_via)} title="{$t('searchHub.matchVia')} {r.match_via}">
														{$t('searchHub.matchVia')} {r.match_via}
													</span>
												{/if}
												<span class="sh-item-lib">{r.library_name}</span>
											</div>
											{#if r.snippet}
												<div class="sh-item-snippet">
													{#if cat === 'contents'}
														{@html highlightInText(r.snippet ?? '')}
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
	.sh-header-wrap {
		position: relative; flex-shrink: 0; z-index: 10; /* anchor for absolute dropdown */
	}
	.sh-header {
		display: flex; align-items: center; gap: 8px; padding: 8px 16px;
		border-bottom: 1px solid var(--border);
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
	.sh-chips-toggle {
		border: none; background: none; color: var(--text-faint); cursor: pointer;
		padding: 3px; border-radius: 4px; display: flex; align-items: center;
	}
	.sh-chips-toggle:hover, .sh-chips-toggle.active { color: var(--interactive-accent); background: var(--bg-hover); }
	.sh-total {
		font-size: 0.72rem; color: var(--interactive-accent); font-weight: 600;
		background: color-mix(in srgb, var(--interactive-accent) 12%, transparent);
		padding: 2px 8px; border-radius: 10px; flex-shrink: 0;
	}
	.sh-unique { font-weight: 400; opacity: 0.8; }

	/* Syntax chips bar */
	.sh-chips-bar {
		display: flex; flex-wrap: wrap; gap: 4px; padding: 6px 16px;
		border-bottom: 1px solid var(--border); background: var(--background-secondary, var(--bg));
	}
	.sh-chip {
		padding: 3px 10px; border-radius: 12px; border: 1px solid var(--border);
		background: var(--bg); color: var(--text-secondary); font-size: 0.72rem;
		cursor: pointer; font-family: inherit; white-space: nowrap;
	}
	.sh-chip:hover { background: var(--bg-hover); color: var(--text); border-color: var(--interactive-accent); }

	/* Wikilink autocomplete */
	.sh-wiki-drop {
		position: absolute; top: 100%; left: 16px; right: 16px; z-index: 200;
		max-height: 300px; overflow-y: auto;
		border: 1px solid var(--interactive-accent); border-radius: 8px;
		background: var(--background-primary, #fff);
		box-shadow: 0 8px 24px rgba(0,0,0,0.2);
		margin-top: 4px;
	}
	.sh-wa-item {
		display: flex; align-items: center; gap: 6px; width: 100%; padding: 4px 16px;
		background: none; border: none; color: var(--text); font-family: inherit;
		cursor: pointer; text-align: start; font-size: 0.82rem;
	}
	.sh-wa-item:hover, .sh-wa-item.selected { background: var(--bg-hover); }
	.sh-wa-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.sh-wa-links {
		display: flex; align-items: center; gap: 2px; flex-shrink: 0;
		font-size: 0.65rem; color: var(--text-muted);
		background: var(--bg); padding: 1px 5px; border-radius: 8px;
	}
	.sh-wa-links.sh-wa-zero { color: #ef4444; background: color-mix(in srgb, #ef4444 10%, transparent); }
	.sh-wa-lib { font-size: 0.65rem; color: var(--text-muted); flex-shrink: 0; }

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

	/* Results area */
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
		display: block; width: 100%; padding: 4px 16px; padding-inline-start: 40px;
		background: none; border: none; color: var(--text); font-family: inherit;
		cursor: pointer; text-align: start;
	}
	.sh-item:hover { background: var(--bg-hover); }
	.sh-dir-arrow { font-size: 1rem; font-weight: 900; flex-shrink: 0; line-height: 1; }
	.sh-dir-in { color: #16a34a; } /* green = incoming (notes link TO X) */
	.sh-dir-out { color: #ef4444; } /* red = outgoing (X links TO this note) */
	.sh-item.sh-item-selected { background: transparent; outline: 2px solid var(--interactive-accent); outline-offset: -2px; border-radius: 4px; }
	.sh-item-top { display: flex; align-items: center; gap: 6px; }
	.sh-item-name { font-size: 0.82rem; font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.sh-item-lib { font-size: 0.68rem; color: var(--accent); flex-shrink: 0; }
	/* M13 — cross-lingual match badge: "via {lemma}". Rendered next to
	   the result name when the hit came through a translation rather
	   than the source lemma itself. Uses a subtle accent-tinted chip so
	   it reads as metadata, not as the headline. `dir` is set on the
	   span so Arabic / Hebrew lemmas render right-to-left regardless of
	   the host row's direction. */
	.sh-match-via {
		font-size: 0.68rem; color: var(--text-muted); flex-shrink: 0;
		padding: 1px 6px; border-radius: 8px;
		background: color-mix(in srgb, var(--interactive-accent) 12%, transparent);
		max-width: 12ch; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
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

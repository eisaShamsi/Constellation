<script lang="ts">
	/**
	 * DigestPane — MIG-045 Phase 3 of the NSC Core Plug-in.
	 *
	 * The left-dock view that lets the user skim the whole knowledge base at
	 * summary-headline level without opening notes. Tiered Library → Folder
	 * → Headline; expandable to full summary; recency-sorted by default;
	 * searchable; virtualized; cUniverse-federated.
	 *
	 * Step A (this commit): skeleton only — top-bar shell + empty list
	 * placeholder. No tree derivation, no headline fetch, no virtualization.
	 * Steps B–E add tree derivation, headline fetch + render, sort/filter,
	 * and VirtualList wiring respectively (per MIG-045-ARCHITECT §6).
	 *
	 * Discipline (CLAUDE.md Rule 3 + Concept Paper §6):
	 *   - Zero new IPC (consumes the shared `summaryStore` from MIG-043).
	 *   - Zero new schema (uses `note_summaries.headline` from MIG-043).
	 *   - Cache-first + batched fetch (per BacklinksPanel / IndexPanel pattern).
	 *   - Virtualized (VirtualList — per IndexPanel).
	 *   - i18n (full 15-locale string-set at PCS-4).
	 */
	import { untrack } from 'svelte';
	import { t } from '$lib/i18n';
	import type { SkyNode, LibraryInfo } from '$lib/libraries/store';
	// MIG-045 Phase 3 Step C — NSC summary headlines + full summaries from
	// the shared store (Phase 1). Cache-first + batched; no new IPC.
	import { getSummariesFor } from '$lib/nsc/summaryStore';
	// Step E — virtualize the list so 7k+-note Universes scroll at 60fps.
	import VirtualList from '$lib/components/VirtualList.svelte';

	let {
		nodes = [] as SkyNode[],
		libraries = [] as LibraryInfo[],
		onNoteClick = (_path: string, _libraryName: string) => {},
	}: {
		nodes: SkyNode[];
		libraries: LibraryInfo[];
		onNoteClick?: (path: string, libraryName: string) => void;
	} = $props();

	// Toolbar state — filter input (wired in Step D), sort mode (used in
	// treeRows derivation, toggled by the sort button).
	//
	// cUniverse toggle: DEFERRED from v1 because identifying which
	// library belongs to a child universe vs the current one requires a
	// `universe_id` field on `LibraryInfo` that doesn't exist (and adding
	// it is a Rust change, violating the frontend-only invariant of MIG-045).
	// Federation still works without the toggle: `resolve_libraries_recursive`
	// already flattens child-universe libraries into the `libraries` array,
	// so they render inline as peers — just without an on/off switch.
	// A follow-up MIG can add `universe_id` and surface the toggle then.
	let filterQuery = $state('');
	let sortMode = $state<'recency' | 'alpha'>('recency');

	// ─── Step B — Tiered tree derivation (Library → Folder → Note) ───
	//
	// VRow is the flattened virtualized-list row type. The list shows
	// library headers, folder headers, and note rows interleaved. Step C
	// adds the 'expanded-summary' variant when a row is expanded.
	type VRow =
		| { kind: 'library-header'; libraryName: string; libraryColor?: string; count: number }
		| { kind: 'folder-header'; libraryName: string; folder: string }
		| { kind: 'note'; node: SkyNode };

	// Path-separator-agnostic folder extraction: returns the path slice
	// relative to the library root, then takes its dirname. The function
	// is defensive — `libraryPath` may be missing (the node's
	// `libraryName` could be a child-universe library we haven't fully
	// resolved yet), in which case we fall back to dirname of the full
	// path. Empty folder = library root.
	function folderOf(node: SkyNode, libraryPath: string | undefined): string {
		const p = node.path;
		let rel = p;
		if (libraryPath && p.startsWith(libraryPath)) {
			rel = p.slice(libraryPath.length).replace(/^[\\/]+/, '');
		}
		const lastSep = Math.max(rel.lastIndexOf('/'), rel.lastIndexOf('\\'));
		return lastSep < 0 ? '' : rel.slice(0, lastSep);
	}

	// Library lookups by name (the only join key SkyNode carries).
	const libByName = $derived.by(() => {
		const m = new Map<string, LibraryInfo>();
		for (const lib of libraries) m.set(lib.name, lib);
		return m;
	});

	// Step D — filter predicate. Substring match (case-insensitive) on
	// note name + headline + full summary. Empty query = no filter (all
	// rows pass). Tracks summaryHeadlines + fullSummaries so the filter
	// also catches notes whose headline mentions the query term, even if
	// the note's own name doesn't.
	function passesFilter(node: SkyNode): boolean {
		const q = filterQuery.trim().toLowerCase();
		if (!q) return true;
		if (node.name.toLowerCase().includes(q)) return true;
		const h = summaryHeadlines.get(node.path);
		if (h && h.toLowerCase().includes(q)) return true;
		const s = fullSummaries.get(node.path);
		if (s && s.toLowerCase().includes(q)) return true;
		return false;
	}

	// The flat row sequence rendered by the virtualized list. Built
	// top-down: for each library (sorted by name asc), bucket its notes
	// by folder, sort folders + notes per sortMode, emit headers + rows.
	// Filter applies at the per-node level — folders/libraries with zero
	// passing notes are skipped entirely (no empty headers).
	const treeRows = $derived.by((): VRow[] => {
		if (libraries.length === 0 || nodes.length === 0) return [];
		// Bucket nodes by libraryName → folder → notes[].
		const buckets = new Map<string, Map<string, SkyNode[]>>();
		for (const node of nodes) {
			if (!passesFilter(node)) continue;
			const libName = node.libraryName ?? '';
			let folderMap = buckets.get(libName);
			if (!folderMap) { folderMap = new Map(); buckets.set(libName, folderMap); }
			const lib = libByName.get(libName);
			const folder = folderOf(node, lib?.path);
			let list = folderMap.get(folder);
			if (!list) { list = []; folderMap.set(folder, list); }
			list.push(node);
		}
		// Emit rows. Libraries first by alpha (stable); folders within a
		// library sort by recency (max child createdAt) when sortMode is
		// 'recency', alpha otherwise. Notes within a folder sort by the
		// same rule. Step D promotes sortMode to a UI control.
		const out: VRow[] = [];
		const libNames = Array.from(buckets.keys()).sort((a, b) => a.localeCompare(b));
		for (const libName of libNames) {
			const folderMap = buckets.get(libName)!;
			const libNoteCount = Array.from(folderMap.values()).reduce((n, list) => n + list.length, 0);
			out.push({ kind: 'library-header', libraryName: libName, count: libNoteCount });

			const folderEntries = Array.from(folderMap.entries());
			if (sortMode === 'recency') {
				// Folders sorted by max child createdAt desc; undefined → -Infinity.
				folderEntries.sort((a, b) => {
					const am = Math.max(...a[1].map(n => n.createdAt ?? -Infinity));
					const bm = Math.max(...b[1].map(n => n.createdAt ?? -Infinity));
					if (am === bm) return a[0].localeCompare(b[0]);
					return bm - am;
				});
			} else {
				folderEntries.sort((a, b) => a[0].localeCompare(b[0]));
			}
			for (const [folder, list] of folderEntries) {
				if (folder !== '') {
					out.push({ kind: 'folder-header', libraryName: libName, folder });
				}
				if (sortMode === 'recency') {
					list.sort((a, b) => (b.createdAt ?? -Infinity) - (a.createdAt ?? -Infinity));
				} else {
					list.sort((a, b) => a.name.localeCompare(b.name));
				}
				for (const node of list) out.push({ kind: 'note', node });
			}
		}
		return out;
	});

	const hasContent = $derived(treeRows.length > 0);

	// ─── Step C — Headlines + full summaries + inline expansion ───
	//
	// summaryHeadlines: per-path 1-line headline (Phase 1's `headline` column).
	// fullSummaries:    per-path full multi-sentence summary (Phase 1's `summary`).
	// expanded:         set of paths the user clicked to expand inline.
	// All three are populated lazily via the shared store's batched fetch.
	let summaryHeadlines = $state<Map<string, string>>(new Map());
	let fullSummaries = $state<Map<string, string>>(new Map());
	let expanded = $state<Set<string>>(new Set());

	// Fetch summaries for every note path in the current treeRows view.
	// Cache-first via the shared store: re-rendering is free, and the
	// `changed` guard avoids unnecessary state writes that would re-fire
	// downstream renders (CLAUDE.md Rule 2). For Step C we fetch the
	// entire treeRows path-set in one batched IPC; Step E (virtualization)
	// will narrow this to the visible viewport for very-large universes.
	//
	// Rule-2 discipline (post-audit fix): the only tracked dep is
	// `treeRows`. The body reads `summaryHeadlines` / `fullSummaries`
	// via `new Map(...)` and writes to them — both happen inside
	// `untrack` so the effect doesn't self-fire on its own writes.
	// Same shape as IndexPanel.svelte:90-101 / 134-163.
	$effect(() => {
		void treeRows; // explicit dep — re-fire when the row set reshapes
		untrack(() => {
			const paths: string[] = [];
			for (const row of treeRows) if (row.kind === 'note') paths.push(row.node.path);
			if (paths.length === 0) return;
			(async () => {
				try {
					const entries = await getSummariesFor(paths);
					let changed = false;
					const nextH = new Map(summaryHeadlines);
					const nextS = new Map(fullSummaries);
					for (const [path, entry] of entries) {
						const h = entry.headline ?? '';
						if (h && nextH.get(path) !== h) { nextH.set(path, h); changed = true; }
						const s = entry.summary ?? '';
						if (s && nextS.get(path) !== s) { nextS.set(path, s); changed = true; }
					}
					if (changed) {
						summaryHeadlines = nextH;
						fullSummaries = nextS;
					}
				} catch { /* ignore — rows still render without headline */ }
			})();
		});
	});

	function toggleExpand(path: string) {
		if (expanded.has(path)) {
			expanded.delete(path);
		} else {
			expanded.add(path);
		}
		expanded = new Set(expanded); // re-trigger reactivity
	}

	// ─── Step E — Virtualized row heights ───
	//
	// Per-row heights for VirtualList. Closes over `summaryHeadlines`
	// + `expanded` + `fullSummaries` so changes to those Maps/Sets cause
	// the `heights` derivation inside VirtualList to re-run and re-layout
	// rows. Same shape as IndexPanel.getRowHeight (the proven 7k+-term
	// reference implementation).
	const ROW_LIBRARY_HEADER = 30;
	const ROW_FOLDER_HEADER = 24;
	const ROW_NOTE_BASE = 22;          // name only
	const ROW_NOTE_HEADLINE = 14;       // + headline line (when loaded)
	const ROW_EXPAND_PAD = 8;
	const ROW_EXPAND_LINE_PX = 18;      // ~one line of full-summary text
	const ROW_EXPAND_MAX_LINES = 14;    // cap so a runaway summary can't blow the layout
	const SUMMARY_CHARS_PER_LINE = 50;  // rough estimate for typical panel width

	function getRowHeight(row: VRow): number {
		if (row.kind === 'library-header') return ROW_LIBRARY_HEADER;
		if (row.kind === 'folder-header') return ROW_FOLDER_HEADER;
		// note row
		let h = ROW_NOTE_BASE;
		const hasHead = summaryHeadlines.has(row.node.path);
		if (hasHead) h += ROW_NOTE_HEADLINE;
		if (expanded.has(row.node.path)) {
			const summary = fullSummaries.get(row.node.path) ?? '';
			if (summary) {
				const lines = Math.min(
					ROW_EXPAND_MAX_LINES,
					Math.max(1, Math.ceil(summary.length / SUMMARY_CHARS_PER_LINE)),
				);
				h += lines * ROW_EXPAND_LINE_PX + ROW_EXPAND_PAD;
			} else {
				h += 20 + ROW_EXPAND_PAD; // "No summary yet." single line
			}
		}
		return h + 4; // small per-row padding
	}

	// scrollResetKey: anything that reshapes treeRows order/contents
	// resets the scroll to top so users aren't stranded mid-list after
	// a filter or sort change. Same pattern as IndexPanel.
	const scrollResetKey = $derived(`${sortMode}|${filterQuery}`);
</script>

<div class="digest-pane">
	<!-- Top bar — filter input + sort toggle + cUniverse toggle. -->
	<div class="dg-toolbar">
		<div class="dg-search">
			<svg class="dg-search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
			</svg>
			<input
				type="text"
				dir="auto"
				placeholder={$t('digest.filterPlaceholder') || 'Filter...'}
				bind:value={filterQuery}
			/>
			{#if filterQuery}
				<button class="dg-clear" onclick={() => filterQuery = ''} aria-label="Clear filter">×</button>
			{/if}
		</div>
		<div class="dg-actions">
			<button
				class="dg-icon-btn"
				class:active={sortMode === 'recency'}
				onclick={() => sortMode = sortMode === 'recency' ? 'alpha' : 'recency'}
				title={sortMode === 'recency'
					? ($t('digest.sortAlpha') || 'Sort alphabetically')
					: ($t('digest.sortRecency') || 'Sort by recency')}
				aria-label={sortMode === 'recency'
					? ($t('digest.sortAlpha') || 'Sort alphabetically')
					: ($t('digest.sortRecency') || 'Sort by recency')}
			>
				{#if sortMode === 'recency'}
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
				{:else}
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h12M3 12h9M3 18h6"/></svg>
				{/if}
			</button>
			<!-- cUniverse toggle removed from v1 — needs Rust `universe_id` on
			     LibraryInfo to distinguish current-universe libraries from
			     federated child-universe ones. Federation still works: child
			     libraries render as peer top-level rows via the existing
			     resolve_libraries_recursive flatten. Toggle returns in a
			     follow-up MIG when the Rust field is added. -->
		</div>
	</div>

	<!-- List area. Step E uses VirtualList — only visible rows render
	     regardless of total count. Per-row heights via getRowHeight;
	     scroll resets when filter/sort changes via scrollResetKey. -->
	<div class="dg-list" dir="auto">
		{#if !hasContent}
			<div class="dg-empty">{$t('digest.empty') || 'No notes to show yet.'}</div>
		{:else}
			<VirtualList items={treeRows} getItemHeight={getRowHeight} {scrollResetKey} overscan={10}>
				{#snippet row(row, _idx)}
				{#if row.kind === 'library-header'}
					<div class="dg-library-header" dir="auto">
						<span class="dg-library-name">{row.libraryName}</span>
						<span class="dg-library-count">{row.count}</span>
					</div>
				{:else if row.kind === 'folder-header'}
					<div class="dg-folder-header" dir="auto" title={row.folder}>{row.folder}</div>
				{:else if row.kind === 'note'}
					{@const isExpanded = expanded.has(row.node.path)}
					{@const headline = summaryHeadlines.get(row.node.path) ?? ''}
					{@const full = fullSummaries.get(row.node.path) ?? ''}
					<!-- Row is a plain layout <div> (no role) with TWO sibling
					     buttons: chevron (toggles expand) + name (opens note).
					     Each button is keyboard-accessible on its own. No
					     nested-interactive a11y warning (post-audit fix). -->
					<div class="dg-note" dir="auto">
						<button type="button"
							class="dg-chev-btn"
							class:expanded={isExpanded}
							onclick={() => toggleExpand(row.node.path)}
							aria-expanded={isExpanded}
							aria-label={isExpanded ? ($t('digest.collapse') || 'Collapse') : ($t('digest.expand') || 'Expand')}
							title={isExpanded ? ($t('digest.collapse') || 'Collapse') : ($t('digest.expand') || 'Expand')}>
							<svg class="dg-chev" class:expanded={isExpanded} width="8" height="8" viewBox="0 0 10 10"><path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/></svg>
						</button>
						<div class="dg-note-body">
							<button type="button"
								class="dg-note-name"
								dir="auto"
								onclick={() => onNoteClick(row.node.path, row.node.libraryName)}
								title={row.node.name}>
								{row.node.name}
							</button>
							{#if headline}
								<button type="button"
									class="dg-note-headline"
									dir="auto"
									onclick={() => toggleExpand(row.node.path)}
									title={isExpanded ? ($t('digest.collapse') || 'Collapse') : ($t('digest.expand') || 'Expand')}>{headline}</button>
							{/if}
							{#if isExpanded && full}
								<div class="dg-note-summary" dir="auto">{full}</div>
							{:else if isExpanded && !full}
								<div class="dg-note-summary dg-note-summary-empty">{$t('digest.noSummary') || 'No summary yet.'}</div>
							{/if}
						</div>
					</div>
				{/if}
				{/snippet}
			</VirtualList>
		{/if}
	</div>
</div>

<style>
	/* MIG-045 Phase 3, Step A — DigestPane skeleton.
	   Visual shape mirrors IndexPanel's top-bar + list layout so the user
	   learns the gesture once. Class names are namespaced `.dg-*` to keep
	   Svelte's CSS pruner from clipping rules (LL-029 follow-up). */
	.digest-pane {
		font-size: 0.8rem;
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.dg-toolbar {
		padding: 6px 8px;
		border-bottom: 1px solid var(--border);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.dg-search {
		position: relative;
		display: flex;
		align-items: center;
		gap: 4px;
		background: var(--background-modifier-form-field);
		border-radius: 4px;
		padding: 2px 6px;
	}
	.dg-search input {
		border: none;
		background: none;
		outline: none;
		font-family: inherit;
		font-size: 0.78rem;
		color: var(--text-normal);
		flex: 1;
		min-width: 0;
	}
	.dg-search-icon {
		color: var(--text-faint);
		flex-shrink: 0;
	}
	.dg-clear {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-faint);
		font-size: 0.9rem;
		padding: 0 2px;
	}

	.dg-actions {
		display: flex;
		gap: 2px;
		justify-content: flex-end;
	}
	.dg-icon-btn {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-faint);
		padding: 2px 4px;
		border-radius: 3px;
		display: flex;
		align-items: center;
	}
	.dg-icon-btn:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.dg-icon-btn.active {
		color: var(--interactive-accent);
	}

	/* VirtualList owns its own scroll container (.vlist). The .dg-list
	   wrapper just provides the flex sizing context so VirtualList can
	   fill the remaining height of the pane. No own overflow. */
	.dg-list {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.dg-empty {
		color: var(--color-base-40);
		font-size: 0.78rem;
		padding: 12px 8px;
		text-align: center;
	}

	/* Step B — tiered row styles. Library header is the most prominent;
	   folder header is a small inline label; note is a clickable button
	   with the headline (Step C) appearing as a faint italic line below
	   the name (same shape as Phase 2 surfaces). */
	.dg-library-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
		padding: 8px 10px 4px;
		font-weight: 700;
		font-size: 0.75rem;
		color: var(--interactive-accent);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
		position: sticky;
		top: 0;
		z-index: 1;
	}
	.dg-library-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.dg-library-count {
		flex-shrink: 0;
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--text-faint);
		font-variant-numeric: tabular-nums;
	}

	.dg-folder-header {
		padding: 4px 10px 2px 18px;
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Note row — plain layout div containing two sibling buttons
	   (chevron + name) inside a content column. No nested interactives. */
	.dg-note {
		display: flex;
		align-items: flex-start;
		gap: 4px;
		width: 100%;
		padding: 4px 10px 4px 14px;
		text-align: start;
		border-radius: 3px;
	}
	.dg-note:hover {
		background: var(--background-modifier-hover);
	}
	/* Chevron toggle button — its OWN interactive element. Click expands. */
	.dg-chev-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 18px;
		flex-shrink: 0;
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-faint);
		padding: 0;
		border-radius: 3px;
	}
	.dg-chev-btn:hover {
		background: var(--background-modifier-active);
		color: var(--text-normal);
	}
	.dg-chev {
		transition: transform 0.15s ease;
		flex-shrink: 0;
	}
	.dg-chev.expanded {
		transform: rotate(90deg);
	}
	.dg-note-body {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
		flex: 1;
		text-align: start;
	}
	/* Name button — its OWN interactive element. Click opens the note. */
	.dg-note-name {
		display: block;
		width: 100%;
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		padding: 0;
		text-align: start;
		color: var(--interactive-accent);
		font-size: 0.78rem;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.dg-note-name:hover {
		text-decoration: underline;
	}
	/* Headline button — clicking it ALSO toggles expand (the visible
	   click-area for the expand gesture, paired with the chevron).
	   Same italic-muted-ellipsis grammar as every Phase 1/2 surface. */
	.dg-note-headline {
		display: block;
		width: 100%;
		background: none;
		border: none;
		cursor: pointer;
		font-family: inherit;
		padding: 0;
		text-align: start;
		font-size: 0.7rem;
		font-style: italic;
		color: var(--text-faint);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.dg-note-headline:hover {
		color: var(--text-muted);
	}
	/* Expanded full summary: multi-line, wraps naturally; muted color so
	   the click-to-collapse cue is the chevron orientation, not visual weight. */
	.dg-note-summary {
		display: block;
		margin-top: 2px;
		font-size: 0.72rem;
		line-height: 1.4;
		color: var(--text-muted);
		white-space: normal;
		word-wrap: break-word;
		overflow-wrap: anywhere;
	}
	.dg-note-summary-empty {
		font-style: italic;
		color: var(--text-faint);
	}
</style>

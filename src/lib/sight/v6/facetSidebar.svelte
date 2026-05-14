<!--
  MIG-025 §A.10 — Sight v6 facet sidebar.

  Collapsed-by-default tab on the left edge (20 px); expands to 180 px
  on click showing 6 facet groups with live Hearst-Flamenco counts.

  Folder is the TOP facet per Concept Paper §11 invariant 8 (round-2/3
  LIS critique demanded explicit Folder visibility — flagged as
  missing from v0.2's mocks).

  Each category click toggles the filter; the parent recomputes the
  filtered row set and re-paints the anchor dome. Counts in OTHER
  facets rebalance to show what's available given the current filter
  set (Hearst preview pattern; see facets.ts::rowsExcluding).

  Visual contract: docs/sight-redesign-v0.3-full-layout.svg
-->
<script lang="ts">
	import type { Facet, FacetId } from './types';
	import type { FacetFilters } from './facets';

	let {
		facets,
		filters,
		onToggle,
		expanded,
		onExpandToggle,
	}: {
		facets: Facet[];
		filters: FacetFilters;
		onToggle: (facet: FacetId, categoryId: string) => void;
		expanded: boolean;
		onExpandToggle: () => void;
	} = $props();

	function isActive(facetId: FacetId, categoryId: string): boolean {
		const set = filters[facetId] as Set<string>;
		return set.has(categoryId);
	}
</script>

{#if !expanded}
	<button
		type="button"
		class="facet-tab"
		onclick={onExpandToggle}
		aria-label="Expand facet filters"
		title="Filters"
	>
		<span class="facet-tab-glyph" aria-hidden="true">▶</span>
	</button>
{:else}
	<aside class="facet-sidebar" aria-label="Sight facet filters">
		<div class="facet-sidebar-header">
			<span class="facet-sidebar-title">FACETS</span>
			<button
				type="button"
				class="facet-sidebar-collapse"
				onclick={onExpandToggle}
				aria-label="Collapse facet filters"
				title="Collapse"
			>◀</button>
		</div>

		<div class="facet-sidebar-body">
			{#each facets as facet (facet.id)}
				<div class="facet-group">
					<div class="facet-group-label">▼ {facet.label}</div>
					<ul class="facet-cat-list">
						{#each facet.categories as cat (cat.id)}
							<li>
								<button
									type="button"
									class="facet-cat-row"
									class:active={isActive(facet.id, cat.id)}
									onclick={() => onToggle(facet.id, cat.id)}
									title={`${facet.label}: ${cat.label} (${cat.count})`}
								>
									<span class="facet-cat-label">{cat.label}</span>
									<span class="facet-cat-count">{cat.count.toLocaleString()}</span>
								</button>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	</aside>
{/if}

<style>
	/* ── Collapsed tab ────────────────────────────────────────── */
	.facet-tab {
		flex: 0 0 auto;
		width: 20px;
		height: 100%;
		background: #0c1322;
		border: 0;
		border-right: 1px solid #1a1f2e;
		color: #5a6275;
		cursor: pointer;
		font-size: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		transition: background 0.12s;
	}
	.facet-tab:hover {
		background: #13192b;
		color: #a0aabe;
	}
	.facet-tab-glyph {
		writing-mode: vertical-rl;
		font-size: 10px;
		letter-spacing: 1px;
	}

	/* ── Expanded panel ───────────────────────────────────────── */
	.facet-sidebar {
		flex: 0 0 180px;
		display: flex;
		flex-direction: column;
		background: #0c1322;
		border-right: 1px solid #1a1f2e;
		color: #cdd5e0;
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);
		font-size: 11px;
		overflow: hidden;
	}

	.facet-sidebar-header {
		flex: 0 0 auto;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 14px;
		border-bottom: 1px solid #1a1f2e;
	}
	.facet-sidebar-title {
		font-size: 10px;
		font-weight: 500;
		letter-spacing: 0.5px;
		color: #a0aabe;
	}
	.facet-sidebar-collapse {
		background: transparent;
		border: 0;
		color: #5a6275;
		cursor: pointer;
		font-size: 12px;
		padding: 0;
		line-height: 1;
	}
	.facet-sidebar-collapse:hover { color: #a0aabe; }

	.facet-sidebar-body {
		flex: 1 1 auto;
		overflow-y: auto;
		padding: 8px 0 16px;
	}

	.facet-group {
		padding: 8px 14px 12px;
	}
	.facet-group-label {
		font-size: 11px;
		font-weight: 500;
		color: #cdd5e0;
		margin-bottom: 4px;
	}

	.facet-cat-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}
	.facet-cat-row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		width: 100%;
		padding: 3px 0;
		background: transparent;
		border: 0;
		color: #7a8295;
		font-size: 10px;
		text-align: left;
		cursor: pointer;
		transition: color 0.12s;
		font-family: inherit;
	}
	.facet-cat-row:hover {
		color: #cdd5e0;
	}
	.facet-cat-row.active {
		color: #7dd3fc;
	}
	.facet-cat-row.active .facet-cat-count {
		color: #7dd3fc;
	}
	.facet-cat-label {
		flex: 1 1 auto;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding-right: 6px;
	}
	.facet-cat-count {
		flex: 0 0 auto;
		color: #a0aabe;
		font-variant-numeric: tabular-nums;
	}
</style>

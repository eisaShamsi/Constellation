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
	// MIG-026 §λ-fix-4 — facet sidebar labels (FACETS title, facet
	// group names, category names like Foundation/Hypothesis/Self/
	// Established) all flow through $t. facets.ts now writes i18n keys
	// in label fields; $t resolves them per active locale and falls
	// back to the literal string for unknown keys (which is what
	// folder/library/custom-stage names are — user data passes through
	// unchanged).
	import { t } from '$lib/i18n';

	let {
		facets,
		filters,
		onToggle,
		expanded,
		onExpandToggle,
		hoveredFacetValues = null,
	}: {
		facets: Facet[];
		filters: FacetFilters;
		onToggle: (facet: FacetId, categoryId: string) => void;
		expanded: boolean;
		onExpandToggle: () => void;
		/** §B.7-fix-3 — when the user hovers a star anywhere in Sight, the
		 *  parent (SightV6) computes the hovered star's value per facet
		 *  and passes it here. Each chip whose (facetId, categoryId) matches
		 *  the hovered values gets a `is-hovered` class for subtle gold tint.
		 *  Reverse companion to the forward direction (click chip → filter
		 *  dome). null when no star is hovered. */
		hoveredFacetValues?: Partial<Record<FacetId, string>> | null;
	} = $props();

	function isActive(facetId: FacetId, categoryId: string): boolean {
		const set = filters[facetId] as Set<string>;
		return set.has(categoryId);
	}

	function isHovered(facetId: FacetId, categoryId: string): boolean {
		return hoveredFacetValues?.[facetId] === categoryId;
	}
</script>

{#if !expanded}
	<button
		type="button"
		class="facet-tab"
		onclick={onExpandToggle}
		aria-label={$t('sight.v6.facetSidebar.expandAriaLabel')}
		title={$t('sight.v6.facetSidebar.filtersTooltip')}
	>
		<span class="facet-tab-glyph" aria-hidden="true">▶</span>
	</button>
{:else}
	<aside class="facet-sidebar" aria-label={$t('sight.v6.facetSidebar.ariaLabel')}>
		<div class="facet-sidebar-header">
			<span class="facet-sidebar-title">{$t('sight.v6.facetSidebar.title')}</span>
			<button
				type="button"
				class="facet-sidebar-collapse"
				onclick={onExpandToggle}
				aria-label={$t('sight.v6.facetSidebar.collapseAriaLabel')}
				title={$t('sight.v6.facetSidebar.collapseTooltip')}
			>◀</button>
		</div>

		<div class="facet-sidebar-body">
			{#each facets as facet (facet.id)}
				<div class="facet-group">
					<div class="facet-group-label" dir="auto">▼ {$t(facet.label)}</div>
					<ul class="facet-cat-list">
						{#each facet.categories as cat (cat.id)}
							<li>
								<button
									type="button"
									class="facet-cat-row"
									class:active={isActive(facet.id, cat.id)}
									class:is-hovered={isHovered(facet.id, cat.id)}
									onclick={() => onToggle(facet.id, cat.id)}
									title={`${$t(facet.label)}: ${$t(cat.label)} (${cat.count})`}
								>
									<span class="facet-cat-label" dir="auto">{$t(cat.label)}</span>
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
		background: var(--background-secondary, #0c1322);
		border: 0;
		border-right: 1px solid var(--background-modifier-border, #1a1f2e);
		color: var(--text-faint, #5a6275);
		cursor: pointer;
		font-size: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		transition: background 0.12s;
	}
	.facet-tab:hover {
		background: var(--background-modifier-hover, #13192b);
		color: var(--text-muted, #a0aabe);
	}
	.facet-tab-glyph {
		writing-mode: vertical-rl;
		font-size: 10px;
		letter-spacing: 1px;
	}
	/* MIG-026 §λ-fix-4b — mirror the expand-arrow chevron in RTL.
	   In LTR the sidebar lives at the left edge and `▶` points into
	   the expand direction. In RTL the sidebar lives at the right
	   edge so the same semantic ("click to expand") points the other
	   way. scaleX(-1) flips the Unicode triangle glyph in-place. */
	:global([dir="rtl"]) .facet-tab-glyph {
		transform: scaleX(-1);
	}

	/* ── Expanded panel ───────────────────────────────────────── */
	.facet-sidebar {
		flex: 0 0 180px;
		display: flex;
		flex-direction: column;
		background: var(--background-secondary, #0c1322);
		border-right: 1px solid var(--background-modifier-border, #1a1f2e);
		color: var(--text-normal, #cdd5e0);
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
		border-bottom: 1px solid var(--background-modifier-border, #1a1f2e);
	}
	.facet-sidebar-title {
		font-size: 10px;
		font-weight: 500;
		letter-spacing: 0.5px;
		color: var(--text-muted, #a0aabe);
	}
	.facet-sidebar-collapse {
		background: transparent;
		border: 0;
		color: var(--text-faint, #5a6275);
		cursor: pointer;
		font-size: 12px;
		padding: 0;
		line-height: 1;
	}
	.facet-sidebar-collapse:hover { color: var(--text-muted, #a0aabe); }
	/* MIG-026 §λ-fix-4b — same mirror logic for the collapse chevron.
	   In LTR `◀` points back toward the left edge (where the sidebar
	   collapses to). In RTL the sidebar collapses to the right edge,
	   so the same semantic should point right; flip the glyph. */
	:global([dir="rtl"]) .facet-sidebar-collapse {
		transform: scaleX(-1);
	}

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
		color: var(--text-normal, #cdd5e0);
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
		color: var(--text-muted, #7a8295);
		font-size: 10px;
		text-align: left;
		cursor: pointer;
		transition: color 0.12s;
		font-family: inherit;
	}
	.facet-cat-row:hover {
		color: var(--text-normal, #cdd5e0);
	}
	.facet-cat-row.active {
		color: var(--text-accent, #7dd3fc);
	}
	.facet-cat-row.active .facet-cat-count {
		color: var(--text-accent, #7dd3fc);
	}
	/* §B.7-fix-3 — hover-linked chip. When the user hovers a star in
	   any dome, the chips matching that star's facet values get a gold
	   tint (same hue as the hover ring) so the user can read the
	   star's identity at a glance from the sidebar. Stacks with .active
	   — if a chip is both active AND matches the hovered star, both
	   styles apply (the gold border on top of the cyan active text).
	   MIG-027 §-fix-2: text + bg use --sight-highlight (theme-aware
	   gold). The vars cascade from .sight-v6-root which is the ancestor
	   in DOM. Light theme → deeper amber that reads on cream. */
	.facet-cat-row.is-hovered {
		color: var(--sight-highlight);
		background: var(--sight-highlight-bg-soft);
	}
	.facet-cat-row.is-hovered .facet-cat-count {
		color: var(--sight-highlight);
	}
	.facet-cat-row.active.is-hovered {
		color: var(--sight-highlight);
		background: var(--sight-highlight-bg-strong);
	}
	.facet-cat-label {
		flex: 1 1 auto;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		/* MIG-026 §λ-fix-4 — logical property so the gap between the
		   label and the count flips correctly in RTL. The old
		   padding-right kept the gap on the right side of the label,
		   which in Arabic put the count flush against the label
		   producing the "549Biology" mash that the Boss flagged on
		   2026-05-18. */
		padding-inline-end: 6px;
	}
	.facet-cat-count {
		flex: 0 0 auto;
		color: var(--text-muted, #a0aabe);
		font-variant-numeric: tabular-nums;
	}
</style>

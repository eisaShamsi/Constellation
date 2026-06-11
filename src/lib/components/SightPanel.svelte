<script lang="ts">
	/**
	 * SightPanel — the CNS analytics sidebar (the registers' home).
	 *
	 * Answers: "What is the SHAPE of my thinking?" — structure only,
	 * per the ratified Constellation-Nervous-System-Concept-Paper §5/§6.
	 * Shows: overview, top bridges, knowledge insights (§B2 recomposes),
	 * and the "Circulation → CCS" hand-off for everything flow-side
	 * (MIG-075 §B1 shed the BY TYPE / BY CONFIDENCE / dormant blocks —
	 * they duplicated CCS's Acts-of-Inquiry + Conviction-&-Doubt registers).
	 */
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t, dir } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';

	import type { UniverseHealth } from '$lib/graph/clusterEngine';

	let {
		nodeCount = 0,
		linkCount = 0,
		orphanCount = 0,
		health = null as UniverseHealth | null,
		bridges = [] as { id: string; name: string; centrality: number }[],
		regions = [] as { id: number; name: string; color: string; count: number; maturity: string | null }[],
		onRegionHover,
		gapRows = [] as { c1: string; c2: string; bridges: string[] }[],
		libraryBreakdown = [] as { name: string; color: string; count: number }[],
		onNoteClick,
	}: {
		nodeCount?: number;
		linkCount?: number;
		orphanCount?: number;
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		// §C1 — the Regions register (top communities by size, with the
		// dominant-maturity character hint). Hover dims the well to the region.
		regions?: { id: number; name: string; color: string; count: number; maturity: string | null }[];
		onRegionHover?: (id: number | null) => void;
		// §C2 — the Blind Spots register: region pairs that should touch and
		// don't, each with suggested bridge notes (clickable).
		gapRows?: { c1: string; c2: string; bridges: string[] }[];
		libraryBreakdown?: { name: string; color: string; count: number }[];
		onNoteClick?: (name: string) => void;
	} = $props();

	// Dominant-maturity → the localized Sky View vocabulary (graphView.ns*,
	// present in all 15 locales — verified before use).
	const MATURITY_KEY: Record<string, string> = {
		seed: 'graphView.nsSeed', sapling: 'graphView.nsSapling',
		evergreen: 'graphView.nsEvergreen', canonical: 'graphView.nsCanonical',
		wilting: 'graphView.nsWilting',
	};

	function healthColor(score: number): string {
		return score >= 70 ? '#16a34a' : score >= 40 ? '#f59e0b' : '#ef4444';
	}

	const isRTL = $derived($dir === 'rtl');

	// ─── Collapsible sections ──────────────────────────────
	let showRegions = $state(true);
	let showBridges = $state(true);
	let showHubs = $state(true);
	let showBlindSpots = $state(true);

	// CCS hand-off gate — mirrors the +layout open-ccs listener's gate.
	const ccsEnabled = $derived($appSettings.enabledFeatures?.ccs !== false);

	// Hubs — the most-connected notes (in-degree; topology). The canonical
	// home per the ratified CNS paper Q6 (KH's card retired in the same
	// commit). Read from the CACHED snapshot — zero live aggregate queries.
	let hubs = $state<any[]>([]);

	onMount(async () => {
		// (§B1 removed the constellation_link_stats + constellation_link_dormant
		// fetches; §B2 replaced the live formulation_analysis insights strip
		// with the cached Hubs read. Zero live link IPCs on the panel open.)
		try {
			const snap: any = await invoke('constellation_knowledge_health_snapshot');
			if (snap?.ready) hubs = (snap.most_connected ?? []).slice(0, 10);
		} catch {}
	});

	function openCcs() {
		// The +layout listener is registered on document (the MIG-007 hub
		// dispatches there too) — window-dispatched events never reach it.
		document.dispatchEvent(new CustomEvent('constellation:open-ccs'));
	}

	function linksPerNote(): string {
		return nodeCount > 0 ? (linkCount / nodeCount).toFixed(1) : '0';
	}
</script>

<div class="sp-root" dir={isRTL ? 'rtl' : 'ltr'}>
	<!-- Section 1: Overview -->
	<div class="sp-overview">
		{#if health}
			<!-- §B3: renamed per the ratified paper Q3 — "Structural Cohesion"
			     (the topology composite; ends the collision with the Knowledge
			     Health plug-in) — and all FOUR sub-metrics readable (paper §5). -->
			<div class="sp-health">
				<div class="sp-health-score">
					<span class="sp-health-num" style="color:{healthColor(health.score)}">{Math.round(health.score)}</span>
					<span class="sp-health-label">{$t('lens.structuralCohesion') || 'Structural Cohesion'}</span>
				</div>
				<div class="sp-health-grid">
					<div class="sp-health-metric">
						<span class="sp-health-val">{health.modularity.toFixed(2)}</span>
						<span class="sp-health-key">{$t('lens.modularity') || 'Modularity'}</span>
					</div>
					<div class="sp-health-metric">
						<span class="sp-health-val">{Math.round(health.dominance * 100)}%</span>
						<span class="sp-health-key">{$t('lens.dominance') || 'Dominance'}</span>
					</div>
					<div class="sp-health-metric">
						<span class="sp-health-val">{(health.connectivity).toFixed(1)}</span>
						<span class="sp-health-key">{$t('lens.connectivity') || 'Links/Note'}</span>
					</div>
					<div class="sp-health-metric">
						<span class="sp-health-val">{Math.round(health.entropy * 100)}%</span>
						<span class="sp-health-key">{$t('lens.entropy') || 'Diversity'}</span>
					</div>
				</div>
			</div>
		{/if}
		<div class="sp-overview-title">{$t('sightPanel.overview') || 'Overview'}</div>
		<div class="sp-stats-grid">
			<div class="sp-stat">
				<span class="sp-stat-val">{nodeCount}</span>
				<span class="sp-stat-label">{$t('sightPanel.totalNodes') || 'Notes'}</span>
			</div>
			<div class="sp-stat">
				<span class="sp-stat-val">{linkCount}</span>
				<span class="sp-stat-label">{$t('sightPanel.totalLinks') || 'Links'}</span>
			</div>
			<div class="sp-stat">
				<span class="sp-stat-val">{orphanCount}</span>
				<span class="sp-stat-label">{$t('sightPanel.orphans') || 'Orphans'}</span>
			</div>
			<div class="sp-stat">
				<span class="sp-stat-val">{linksPerNote()}</span>
				<span class="sp-stat-label">{$t('sightPanel.linksPerNote') || 'Links/Note'}</span>
			</div>
		</div>
		<!-- Library breakdown -->
		{#if libraryBreakdown.length > 0}
			<div class="sp-libs">
				{#each libraryBreakdown as lib}
					<div class="sp-lib-row">
						<span class="sp-lib-dot" style="background:{lib.color}"></span>
						<span class="sp-lib-name">{lib.name}</span>
						<span class="sp-lib-count">{lib.count}</span>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Section 2: Circulation → CCS (the §B1 hand-off — the BY TYPE /
	     BY CONFIDENCE / dormant blocks now live in the Circulatory System) -->
	{#if ccsEnabled}
		<div class="sp-section">
			<button class="sp-header sp-ccs-link" onclick={openCcs}>
				<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12h4l2-7 4 14 2-7h6"/></svg>
				<span>{$t('sightPanel.openCcs') || 'Circulation → Circulatory System'}</span>
				<span class="sp-ccs-arrow" aria-hidden="true">›</span>
			</button>
		</div>
	{/if}

	<!-- Section: Regions (§C1 — the emergent neighborhoods of thought,
	     Louvain over the link graph; rendered at last after computing dark
	     since the v2 era. Hover a row → the well dims to that region. -->
	{#if regions.length > 0}
		<div class="sp-section">
			<button class="sp-header" onclick={() => showRegions = !showRegions}>
				<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!showRegions}><polyline points="6 9 12 15 18 9"/></svg>
				<span>{$t('sightPanel.regions') || 'Regions'}</span>
				<span class="sp-count">{regions.length}</span>
			</button>
			{#if showRegions}
				<div class="sp-list" role="list" onmouseleave={() => onRegionHover?.(null)}>
					{#each regions as region}
						<div class="sp-item sp-region-row" role="listitem"
							onmouseenter={() => onRegionHover?.(region.id)}>
							<span class="sp-region-dot" style="background:{region.color}"></span>
							<span class="sp-item-name" dir="auto">{region.name}</span>
							<span class="sp-item-score">
								{region.count}{#if region.maturity && MATURITY_KEY[region.maturity]}&nbsp;· {$t(MATURITY_KEY[region.maturity]) || region.maturity}{/if}
							</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Section 3: Top Bridges -->
	<div class="sp-section">
		<button class="sp-header" onclick={() => showBridges = !showBridges}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!showBridges}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('sightPanel.topBridges') || 'Top Bridges'}</span>
			<span class="sp-count">{bridges.length}</span>
		</button>
		{#if showBridges}
			<div class="sp-list">
				{#each bridges.slice(0, 10) as bridge}
					{@const maxC = bridges[0]?.centrality ?? 1}
					<button class="sp-item" onclick={() => onNoteClick?.(bridge.name)} dir="auto">
						<div class="sp-item-bar" style="width:{(bridge.centrality / maxC) * 100}%"></div>
						<span class="sp-item-name">{bridge.name}</span>
						<span class="sp-item-score">{(bridge.centrality * 100).toFixed(0)}%</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Section 4: Hubs — most-connected notes (topology; the canonical home
	     per the ratified CNS paper Q6). §B2 replaced the six-tab insights
	     strip: strongest_evidence → CCS Load-Bearing · weak_foundations →
	     KH's card · stagnating → data-dead (CCS Cooling is the live read) ·
	     tensions → the health tab's TensionPanel · knowledge_gaps → retired. -->
	<div class="sp-section">
		<button class="sp-header" onclick={() => showHubs = !showHubs}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!showHubs}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('sightPanel.hubs') || 'Hubs'}</span>
			<span class="sp-count">{hubs.length}</span>
		</button>
		{#if showHubs}
			<div class="sp-list">
				{#each hubs as hub}
					<button class="sp-item" onclick={() => onNoteClick?.(hub.target_name)} dir="auto">
						<span class="sp-item-name">{hub.target_name}</span>
						<span class="sp-item-score">{$t('knowledgeHealth.incomingLinks', { n: (hub.traversal_count ?? 0).toLocaleString() })}</span>
					</button>
				{:else}
					<div class="sp-empty">{$t('sightPanel.noResults') || 'No results'}</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Section: Blind Spots (§C2 — the founding register restored: region
	     pairs with dense interiors and no connecting tissue, each with the
	     notes that could bridge them. An EMPTY list is good news. -->
	<div class="sp-section">
		<button class="sp-header" onclick={() => showBlindSpots = !showBlindSpots}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!showBlindSpots}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('sightPanel.blindSpots') || 'Blind Spots'}</span>
			<span class="sp-count">{gapRows.length}</span>
		</button>
		{#if showBlindSpots}
			<div class="sp-list">
				{#each gapRows as gap}
					<div class="sp-gap-row">
						<div class="sp-gap-pair" dir="auto">
							<span class="sp-gap-name">{gap.c1}</span>
							<span class="sp-gap-sep" aria-hidden="true">↮</span>
							<span class="sp-gap-name">{gap.c2}</span>
						</div>
						{#if gap.bridges.length > 0}
							<div class="sp-gap-bridges">
								<span class="sp-gap-hint">{$t('sightPanel.suggestedBridges') || 'Suggested bridges'}:</span>
								{#each gap.bridges as bridge}
									<button class="sp-bridge-chip" onclick={() => onNoteClick?.(bridge)} dir="auto">{bridge}</button>
								{/each}
							</div>
						{/if}
					</div>
				{:else}
					<div class="sp-empty">{$t('sightPanel.noResults') || 'No results'}</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.sp-root {
		/* §C-fix-1 (Eisa, Stage 2): 280 → 380px — the Blind Spots pair rows
		   need the breath; matches the app right-sidebar width convention. */
		width: 380px; height: 100%; overflow-y: auto; overflow-x: hidden;
		background: var(--background-primary, #fff);
		border-inline-start: 1px solid var(--background-modifier-border, #e5e7eb);
		font-size: 11px; display: flex; flex-direction: column; gap: 0;
	}
	/* Health */
	.sp-health { text-align: center; padding-bottom: 10px; margin-bottom: 8px; border-bottom: 1px solid var(--background-modifier-border, #e5e7eb); }
	.sp-health-score { display: flex; align-items: baseline; justify-content: center; gap: 6px; }
	.sp-health-num { font-size: 36px; font-weight: 800; line-height: 1; }
	.sp-health-label { font-size: 11px; color: var(--text-muted, #64748b); }
	.sp-health-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; margin-top: 6px; }
	.sp-health-metric { text-align: center; padding: 3px; background: var(--background-secondary, #f8fafc); border-radius: 4px; }
	.sp-health-val { display: block; font-size: 13px; font-weight: 700; color: var(--text-normal, #1a1a1a); }
	.sp-health-key { font-size: 9px; color: var(--text-faint, #94a3b8); text-transform: uppercase; letter-spacing: 0.5px; }
	/* Overview */
	.sp-overview { padding: 12px; border-bottom: 1px solid var(--background-modifier-border, #e5e7eb); }
	.sp-overview-title { font-size: 12px; font-weight: 700; color: var(--text-normal, #1a1a1a); margin-bottom: 8px; }
	.sp-stats-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
	.sp-stat { text-align: center; padding: 6px 4px; background: var(--background-secondary, #f8fafc); border-radius: 4px; }
	.sp-stat-val { display: block; font-size: 16px; font-weight: 700; color: var(--text-normal, #1a1a1a); }
	.sp-stat-label { font-size: 9px; color: var(--text-faint, #94a3b8); text-transform: uppercase; letter-spacing: 0.5px; }
	.sp-libs { margin-top: 8px; display: flex; flex-direction: column; gap: 2px; }
	.sp-lib-row { display: flex; align-items: center; gap: 6px; padding: 2px 0; }
	.sp-lib-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.sp-lib-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-muted, #64748b); }
	.sp-lib-count { font-size: 10px; color: var(--text-faint, #94a3b8); }
	/* Sections */
	.sp-section { border-bottom: 1px solid var(--background-modifier-border, #e5e7eb); }
	.sp-header {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 8px 12px; border: none; background: none; cursor: pointer;
		font-size: 11px; font-weight: 600; color: var(--text-normal, #1a1a1a);
		font-family: inherit; text-align: start;
	}
	.sp-header:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.sp-header svg { color: var(--text-muted, #64748b); transition: transform 0.15s; }
	.sp-header svg.rotated { transform: rotate(-90deg); }
	.sp-count { font-size: 10px; color: var(--text-faint); margin-inline-start: auto; background: var(--background-modifier-border); padding: 0 5px; border-radius: 6px; }
	/* The CCS hand-off row */
	.sp-ccs-link { color: var(--interactive-accent, #7c3aed); }
	.sp-ccs-arrow { margin-inline-start: auto; font-size: 13px; color: var(--text-faint, #94a3b8); }
	/* Regions rows (§C1) */
	.sp-region-row { cursor: default; }
	.sp-region-dot { width: 9px; height: 9px; border-radius: 50%; flex-shrink: 0; }
	/* Blind Spots rows (§C2) */
	.sp-gap-row { padding: 4px 8px 6px; display: flex; flex-direction: column; gap: 3px; }
	.sp-gap-pair { display: flex; align-items: center; gap: 5px; font-size: 11px; color: var(--text-normal, #1a1a1a); }
	.sp-gap-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.sp-gap-sep { color: #ef4444; font-weight: 700; flex-shrink: 0; }
	.sp-gap-bridges { display: flex; flex-wrap: wrap; align-items: center; gap: 4px; }
	.sp-gap-hint { font-size: 9px; color: var(--text-faint, #94a3b8); }
	.sp-bridge-chip {
		font-size: 10px; padding: 1px 7px; border-radius: 8px; cursor: pointer;
		border: 1px solid var(--background-modifier-border, #e5e7eb);
		background: none; color: var(--interactive-accent, #7c3aed); font-family: inherit;
		max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.sp-bridge-chip:hover { background: var(--background-modifier-hover, #f1f5f9); }
	/* List items */
	.sp-list { display: flex; flex-direction: column; gap: 1px; padding: 0 4px 6px; }
	.sp-item {
		display: flex; align-items: center; gap: 6px; padding: 4px 8px;
		border: none; background: none; cursor: pointer; font-size: 11px;
		color: var(--text-normal, #1a1a1a); font-family: inherit;
		border-radius: 3px; position: relative; text-align: start; width: 100%;
	}
	.sp-item:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.sp-item-bar { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; background: var(--interactive-accent, #7c3aed); opacity: 0.06; border-radius: 3px; }
	.sp-item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; position: relative; }
	.sp-item-score { font-size: 10px; color: var(--text-faint, #94a3b8); position: relative; flex-shrink: 0; }
	.sp-empty { padding: 12px; text-align: center; color: var(--text-faint, #94a3b8); font-size: 11px; }
</style>

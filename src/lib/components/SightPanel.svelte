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
		libraryBreakdown = [] as { name: string; color: string; count: number }[],
		onNoteClick,
	}: {
		nodeCount?: number;
		linkCount?: number;
		orphanCount?: number;
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		libraryBreakdown?: { name: string; color: string; count: number }[];
		onNoteClick?: (name: string) => void;
	} = $props();

	function healthColor(score: number): string {
		return score >= 70 ? '#16a34a' : score >= 40 ? '#f59e0b' : '#ef4444';
	}

	const isRTL = $derived($dir === 'rtl');

	// ─── Collapsible sections ──────────────────────────────
	let showBridges = $state(true);
	let showInsights = $state(false);

	// CCS hand-off gate — mirrors the +layout open-ccs listener's gate.
	const ccsEnabled = $derived($appSettings.enabledFeatures?.ccs !== false);

	let insights = $state<any[]>([]);
	let insightType = $state('strongest_evidence');

	const LINK_TYPE_COLORS: Record<string, string> = {
		supports: '#4A9EFF', contradicts: '#FF4A4A', causes: '#FF8C42',
		exemplifies: '#4AFF88', generalizes: '#C084FC', 'derives-from': '#FACC15',
		'part-of': '#94A3B8', associative: '#A78BFA',
	};

	const INSIGHT_TYPES = [
		'strongest_evidence', 'weak_foundations', 'tensions',
		'stagnating', 'most_connected', 'knowledge_gaps',
	];

	onMount(async () => {
		// (§B1 removed the constellation_link_stats + constellation_link_dormant
		// fetches — live full-table aggregates feeding circulatory blocks that
		// now live in CCS. Zero live link IPCs remain on the panel-open path
		// besides the insights query, which §B2 re-points to the cache.)
		loadInsights(insightType);
	});

	function openCcs() {
		window.dispatchEvent(new CustomEvent('constellation:open-ccs'));
	}

	async function loadInsights(type: string) {
		insightType = type;
		try {
			const result: any[] = await invoke('constellation_formulation_analysis', { queryType: type });
			insights = result?.slice(0, 10) ?? [];
		} catch { insights = []; }
	}

	function linksPerNote(): string {
		return nodeCount > 0 ? (linkCount / nodeCount).toFixed(1) : '0';
	}
</script>

<div class="sp-root" dir={isRTL ? 'rtl' : 'ltr'}>
	<!-- Section 1: Overview -->
	<div class="sp-overview">
		{#if health}
			<div class="sp-health">
				<div class="sp-health-score">
					<span class="sp-health-num" style="color:{healthColor(health.score)}">{Math.round(health.score)}</span>
					<span class="sp-health-label">{$t('lens.universeHealth') || 'Universe Health'}</span>
				</div>
				<div class="sp-health-grid">
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

	<!-- Section 4: Knowledge Insights -->
	<div class="sp-section">
		<button class="sp-header" onclick={() => showInsights = !showInsights}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!showInsights}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('sightPanel.insights') || 'Knowledge Insights'}</span>
		</button>
		{#if showInsights}
			<div class="sp-insight-tabs">
				{#each INSIGHT_TYPES as itype}
					<button class="sp-insight-tab" class:active={insightType === itype} onclick={() => loadInsights(itype)}>
						{$t(`sightPanel.${itype}`) || itype.replace(/_/g, ' ')}
					</button>
				{/each}
			</div>
			<div class="sp-list">
				{#each insights as insight}
					<button class="sp-item" onclick={() => onNoteClick?.(insight.source_name)} dir="auto">
						<span class="sp-item-name">{insight.source_name} → {insight.target_name}</span>
						<span class="sp-item-score" style="color:{LINK_TYPE_COLORS[insight.link_type] ?? '#94a3b8'}">{insight.link_type ?? ''}</span>
					</button>
				{/each}
				{#if insights.length === 0}
					<div class="sp-empty">{$t('sightPanel.noResults') || 'No results'}</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.sp-root {
		width: 280px; height: 100%; overflow-y: auto; overflow-x: hidden;
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
	/* Insight tabs */
	.sp-insight-tabs { display: flex; flex-wrap: wrap; gap: 4px; padding: 4px 12px; }
	.sp-insight-tab {
		padding: 2px 8px; border-radius: 4px; border: 1px solid var(--background-modifier-border, #e5e7eb);
		background: none; color: var(--text-muted, #64748b); font-size: 9px; cursor: pointer;
		font-family: inherit; white-space: nowrap;
	}
	.sp-insight-tab:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.sp-insight-tab.active { background: var(--interactive-accent, #7c3aed); color: white; border-color: var(--interactive-accent); }
	.sp-empty { padding: 12px; text-align: center; color: var(--text-faint, #94a3b8); font-size: 11px; }
</style>

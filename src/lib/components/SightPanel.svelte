<script lang="ts">
	/**
	 * SightPanel — Knowledge analytics sidebar for Constellation Sight.
	 *
	 * Answers: "How healthy is my knowledge system?"
	 * Shows: overview, link health, top bridges, knowledge insights.
	 */
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t, dir } from '$lib/i18n';

	let {
		nodeCount = 0,
		linkCount = 0,
		orphanCount = 0,
		bridges = [] as { id: string; name: string; centrality: number }[],
		libraryBreakdown = [] as { name: string; color: string; count: number }[],
		onNoteClick,
	}: {
		nodeCount?: number;
		linkCount?: number;
		orphanCount?: number;
		bridges?: { id: string; name: string; centrality: number }[];
		libraryBreakdown?: { name: string; color: string; count: number }[];
		onNoteClick?: (name: string) => void;
	} = $props();

	const isRTL = $derived($dir === 'rtl');

	// ─── Collapsible sections ──────────────────────────────
	let showLinkHealth = $state(true);
	let showBridges = $state(true);
	let showInsights = $state(false);

	// ─── Link Stats (fetched from Rust) ────────────────────
	let linkStats = $state<any>(null);
	let dormantCount = $state(0);
	let insights = $state<any[]>([]);
	let insightType = $state('strongest_evidence');

	const LINK_TYPE_COLORS: Record<string, string> = {
		supports: '#4A9EFF', contradicts: '#FF4A4A', causes: '#FF8C42',
		exemplifies: '#4AFF88', generalizes: '#C084FC', 'derives-from': '#FACC15',
		'part-of': '#94A3B8', associative: '#A78BFA',
	};

	const CONFIDENCE_COLORS: Record<string, string> = {
		hypothesis: '#94a3b8', evidence: '#3b82f6', established: '#16a34a', contested: '#ef4444',
	};

	const INSIGHT_TYPES = [
		'strongest_evidence', 'weak_foundations', 'tensions',
		'stagnating', 'most_connected', 'knowledge_gaps',
	];

	onMount(async () => {
		// Fetch link stats
		try { linkStats = await invoke('constellation_link_stats'); } catch {}
		// Fetch dormant count
		try {
			const dormant: any[] = await invoke('constellation_link_dormant');
			dormantCount = dormant?.length ?? 0;
		} catch {}
		// Fetch initial insights
		loadInsights(insightType);
	});

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

	<!-- Section 2: Link Health -->
	<div class="sp-section">
		<button class="sp-header" onclick={() => showLinkHealth = !showLinkHealth}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={!showLinkHealth}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('sightPanel.linkHealth') || 'Link Health'}</span>
		</button>
		{#if showLinkHealth && linkStats}
			<div class="sp-bars">
				<!-- By Type -->
				<div class="sp-bar-title">{$t('sightPanel.byType') || 'By Type'}</div>
				{#each Object.entries(linkStats.by_type ?? {}) as [type, count]}
					{@const maxCount = Math.max(...Object.values(linkStats.by_type ?? {}).map(Number))}
					<div class="sp-bar-row">
						<span class="sp-bar-label">{$t(`lens.link${type.charAt(0).toUpperCase() + type.slice(1).replace(/-./g, c => c[1].toUpperCase())}`) || type}</span>
						<div class="sp-bar-track">
							<div class="sp-bar-fill" style="width:{Math.max(2, (Number(count) / Math.max(maxCount, 1)) * 100)}%;background:{LINK_TYPE_COLORS[type] ?? '#94a3b8'}"></div>
						</div>
						<span class="sp-bar-val">{count}</span>
					</div>
				{/each}
				<!-- By Confidence -->
				<div class="sp-bar-title" style="margin-top:8px">{$t('sightPanel.byConfidence') || 'By Confidence'}</div>
				{#each Object.entries(linkStats.by_confidence ?? {}) as [conf, count]}
					{@const maxConf = Math.max(...Object.values(linkStats.by_confidence ?? {}).map(Number))}
					<div class="sp-bar-row">
						<span class="sp-bar-label">{conf}</span>
						<div class="sp-bar-track">
							<div class="sp-bar-fill" style="width:{Math.max(2, (Number(count) / Math.max(maxConf, 1)) * 100)}%;background:{CONFIDENCE_COLORS[conf] ?? '#94a3b8'}"></div>
						</div>
						<span class="sp-bar-val">{count}</span>
					</div>
				{/each}
				<!-- Dormant -->
				{#if dormantCount > 0}
					<div class="sp-dormant">
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
						<span>{dormantCount} {$t('sightPanel.dormantLinks') || 'dormant links (90+ days)'}</span>
					</div>
				{/if}
			</div>
		{/if}
	</div>

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
	/* Bars */
	.sp-bars { padding: 4px 12px 10px; }
	.sp-bar-title { font-size: 9px; font-weight: 600; color: var(--text-faint, #94a3b8); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
	.sp-bar-row { display: flex; align-items: center; gap: 6px; height: 18px; }
	.sp-bar-label { width: 70px; font-size: 10px; color: var(--text-muted, #64748b); text-align: end; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex-shrink: 0; }
	.sp-bar-track { flex: 1; height: 6px; background: var(--background-modifier-border, #e5e7eb); border-radius: 3px; overflow: hidden; }
	.sp-bar-fill { height: 100%; border-radius: 3px; min-width: 2px; }
	.sp-bar-val { width: 28px; font-size: 10px; color: var(--text-faint, #94a3b8); text-align: end; flex-shrink: 0; }
	.sp-dormant { display: flex; align-items: center; gap: 6px; margin-top: 6px; padding: 4px 6px; background: rgba(245, 158, 11, 0.08); border-radius: 4px; color: #f59e0b; font-size: 10px; }
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

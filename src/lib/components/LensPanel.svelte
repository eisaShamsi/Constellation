<script lang="ts">
	/**
	 * LensPanel — Constellation Lens analytics sidebar.
	 *
	 * Shows: Universe Health, Top Bridges, Communities, Structural Gaps.
	 * Displayed as an overlay panel inside GraphMind when the Lens is active.
	 */
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import type { ClusterInfo } from '$lib/graph/clusterEngine';
	import type { StructuralGap, UniverseHealth } from '$lib/graph/clusterEngine';

	let {
		health = null as UniverseHealth | null,
		bridges = [] as { id: string; name: string; centrality: number }[],
		communities = [] as ClusterInfo[],
		gaps = [] as StructuralGap[],
		nodeCount = 0,
		edgeCount = 0,
		onNoteClick,
		onNodeHover,
	}: {
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		communities?: ClusterInfo[];
		gaps?: StructuralGap[];
		nodeCount?: number;
		edgeCount?: number;
		onNoteClick?: (id: string, name: string) => void;
		onNodeHover?: (id: string | null) => void;
	} = $props();

	let showBridges = $state(true);
	let showCommunities = $state(true);
	let showGaps = $state(false);

	function healthColor(score: number): string {
		if (score >= 70) return '#16a34a'; // green
		if (score >= 40) return '#f59e0b'; // amber
		return '#ef4444'; // red
	}
</script>

<div class="lens-panel">
	<!-- Universe Health -->
	{#if health}
		<div class="lp-section lp-health">
			<div class="lp-health-score" style="color:{healthColor(health.score)}">
				<span class="lp-health-num">{health.score}</span>
				<span class="lp-health-label">{$t('lens.universeHealth') || 'Universe Health'}</span>
			</div>
			<div class="lp-health-grid">
				<div class="lp-metric">
					<span class="lp-metric-val">{health.modularity.toFixed(2)}</span>
					<span class="lp-metric-label">{$t('lens.modularity') || 'Modularity'}</span>
				</div>
				<div class="lp-metric">
					<span class="lp-metric-val">{(health.dominance * 100).toFixed(0)}%</span>
					<span class="lp-metric-label">{$t('lens.dominance') || 'Dominance'}</span>
				</div>
				<div class="lp-metric">
					<span class="lp-metric-val">{health.entropy.toFixed(2)}</span>
					<span class="lp-metric-label">{$t('lens.entropy') || 'Entropy'}</span>
				</div>
				<div class="lp-metric">
					<span class="lp-metric-val">{health.connectivity.toFixed(1)}</span>
					<span class="lp-metric-label">{$t('lens.connectivity') || 'Links/Note'}</span>
				</div>
			</div>
			<div class="lp-stats">
				{nodeCount} {$t('lens.nodes') || 'nodes'} · {edgeCount} {$t('lens.edges') || 'edges'}
			</div>
		</div>
	{/if}

	<!-- Top Bridges -->
	<div class="lp-section">
		<button class="lp-header" onclick={() => showBridges = !showBridges}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={showBridges}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('lens.topBridges') || 'Top Bridges'}</span>
			<span class="lp-count">{bridges.length}</span>
		</button>
		{#if showBridges}
			<div class="lp-list">
				{#each bridges as b (b.id)}
					<button class="lp-item"
						dir={detectDir(b.name)}
						onclick={() => onNoteClick?.(b.id, b.name)}
						onmouseenter={() => onNodeHover?.(b.id)}
						onmouseleave={() => onNodeHover?.(null)}>
						<span class="lp-bar" style="width:{Math.max(4, b.centrality * 100)}%"></span>
						<span class="lp-item-name">{b.name}</span>
						<span class="lp-item-score">{(b.centrality * 100).toFixed(0)}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Communities -->
	<div class="lp-section">
		<button class="lp-header" onclick={() => showCommunities = !showCommunities}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={showCommunities}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('lens.communities') || 'Communities'}</span>
			<span class="lp-count">{communities.length}</span>
		</button>
		{#if showCommunities}
			<div class="lp-list">
				{#each communities as c (c.id)}
					<div class="lp-comm">
						<span class="lp-comm-dot" style="background:{c.color}"></span>
						<span class="lp-comm-name" dir={detectDir(c.suggestedName)}>{c.suggestedName}</span>
						<span class="lp-comm-count">{c.memberIds.length}</span>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Structural Gaps -->
	{#if gaps.length > 0}
		<div class="lp-section">
			<button class="lp-header" onclick={() => showGaps = !showGaps}>
				<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={showGaps}><polyline points="6 9 12 15 18 9"/></svg>
				<span>{$t('lens.structuralGaps') || 'Blind Spots'}</span>
				<span class="lp-count">{gaps.length}</span>
			</button>
			{#if showGaps}
				<div class="lp-list">
					{#each gaps as gap}
						<div class="lp-gap">
							<span class="lp-gap-names">{gap.community1Name} ↔ {gap.community2Name}</span>
							{#if gap.interLinkCount === 0}
								<span class="lp-gap-badge lp-gap-zero">{$t('lens.noLinks') || 'No links'}</span>
							{:else}
								<span class="lp-gap-badge">{gap.interLinkCount} {$t('lens.links') || 'links'}</span>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.lens-panel {
		width: 280px; max-height: 100%; overflow-y: auto;
		background: var(--background-primary); border-inline-start: 1px solid var(--background-modifier-border);
		padding: 12px; display: flex; flex-direction: column; gap: 8px;
		font-size: 12px;
	}

	/* Universe Health */
	.lp-health { text-align: center; padding-bottom: 8px; border-bottom: 1px solid var(--background-modifier-border); }
	.lp-health-score { display: flex; align-items: baseline; justify-content: center; gap: 6px; }
	.lp-health-num { font-size: 36px; font-weight: 800; line-height: 1; }
	.lp-health-label { font-size: 11px; color: var(--text-muted); }
	.lp-health-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; margin-top: 8px; }
	.lp-metric { text-align: center; padding: 4px; background: var(--background-secondary); border-radius: 4px; }
	.lp-metric-val { display: block; font-size: 13px; font-weight: 700; color: var(--text-normal); }
	.lp-metric-label { font-size: 9px; color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.5px; }
	.lp-stats { font-size: 10px; color: var(--text-faint); margin-top: 6px; }

	/* Section headers */
	.lp-section { display: flex; flex-direction: column; }
	.lp-header {
		display: flex; align-items: center; gap: 6px;
		padding: 6px 0; border: none; background: none; cursor: pointer;
		font-family: inherit; font-size: 12px; font-weight: 600; color: var(--text-normal);
		text-align: start;
	}
	.lp-header svg { color: var(--text-muted); transition: transform 0.15s; }
	.lp-header svg.rotated { transform: rotate(180deg); }
	.lp-count { font-size: 10px; color: var(--text-faint); margin-inline-start: auto; background: var(--background-modifier-border); padding: 0 5px; border-radius: 6px; }

	/* Lists */
	.lp-list { display: flex; flex-direction: column; gap: 1px; }

	/* Bridge items */
	.lp-item {
		display: flex; align-items: center; gap: 6px; padding: 4px 6px;
		border: none; border-radius: 4px; background: none; cursor: pointer;
		font-family: inherit; font-size: 12px; color: var(--text-normal); text-align: start;
		position: relative; overflow: hidden;
	}
	.lp-item:hover { background: var(--background-modifier-hover); }
	.lp-bar {
		position: absolute; inset-inline-start: 0; top: 0; bottom: 0;
		background: var(--interactive-accent); opacity: 0.08; border-radius: 4px;
	}
	.lp-item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; position: relative; }
	.lp-item-score { font-size: 10px; color: var(--text-faint); position: relative; }

	/* Community items */
	.lp-comm { display: flex; align-items: center; gap: 6px; padding: 3px 6px; }
	.lp-comm-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.lp-comm-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-normal); }
	.lp-comm-count { font-size: 10px; color: var(--text-faint); }

	/* Gap items */
	.lp-gap { display: flex; align-items: center; gap: 6px; padding: 4px 6px; }
	.lp-gap-names { flex: 1; font-size: 11px; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.lp-gap-badge { font-size: 10px; color: var(--text-muted); background: var(--background-secondary); padding: 1px 5px; border-radius: 4px; }
	.lp-gap-zero { color: #ef4444; background: color-mix(in srgb, #ef4444 10%, transparent); }
</style>

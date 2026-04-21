<script lang="ts">
	/**
	 * LensPanel — Constellation Lens analytics sidebar.
	 *
	 * Shows: Universe Health, Top Bridges, Communities, Structural Gaps.
	 * Displayed as an overlay panel inside GraphMind when the Lens is active.
	 */
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import type { ClusterInfo, CommunityProfile } from '$lib/graph/clusterEngine';
	import type { StructuralGap, UniverseHealth } from '$lib/graph/clusterEngine';

	let {
		health = null as UniverseHealth | null,
		bridges = [] as { id: string; name: string; centrality: number }[],
		communities = [] as ClusterInfo[],
		communityProfiles = [] as CommunityProfile[],
		contradictions = [] as [string, string][],
		gaps = [] as StructuralGap[],
		nodeCount = 0,
		edgeCount = 0,
		onNoteClick,
		onNodeHover,
		showTagEdges = $bindable(false),
		peelCount = $bindable(0),
		onPeelChange,
		onTagEdgesToggle,
	}: {
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		communities?: ClusterInfo[];
		communityProfiles?: CommunityProfile[];
		contradictions?: [string, string][];
		gaps?: StructuralGap[];
		nodeCount?: number;
		edgeCount?: number;
		onNoteClick?: (id: string, name: string) => void;
		onNodeHover?: (id: string | null) => void;
		showTagEdges?: boolean;
		peelCount?: number;
		onPeelChange?: (count: number) => void;
		onTagEdgesToggle?: (show: boolean) => void;
	} = $props();

	let showBridges = $state(true);
	let showCommunities = $state(true);
	let showGaps = $state(false);
	let showAdvanced = $state(false);
	let showContradictions = $state(false);
	let showLegend = $state(false);

	function healthColor(score: number): string {
		if (score >= 70) return '#16a34a'; // green
		if (score >= 40) return '#f59e0b'; // amber
		return '#ef4444'; // red
	}
</script>

<div class="lens-panel">
	<!-- Universe Health — frozen/sticky -->
	<div class="lp-frozen">
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
	</div>

	<!-- Scrollable categories -->
	<div class="lp-scrollable">
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
				{#each communities as c, i (c.id)}
					{@const profile = communityProfiles.find(p => p.id === c.id)}
					<div class="lp-comm">
						<div class="lp-comm-header">
							<span class="lp-comm-dot" style="background:{c.color}"></span>
							<span class="lp-comm-name" dir={detectDir(c.suggestedName)}>{c.suggestedName}</span>
							<span class="lp-comm-count">{c.memberIds.length}</span>
						</div>
						{#if profile}
							<!-- Maturity bar -->
							<div class="lp-comm-bar">
								{#each ['seed','sapling','evergreen','canonical','wilting'] as m}
									{@const count = profile.maturityBreakdown[m] ?? 0}
									{#if count > 0}
										<div class="lp-comm-bar-seg" style="width:{(count/profile.memberCount)*100}%; background:{({'seed':'#d1d5db','sapling':'#86efac','evergreen':'#16a34a','canonical':'#f59e0b','wilting':'#a3e635'})[m]}" title="{m}: {count}"></div>
									{/if}
								{/each}
							</div>
							<div class="lp-comm-meta">
								<span>L{profile.avgStratum.toFixed(0)}</span>
								{#if profile.wiltingPercent > 20}<span class="lp-comm-warning">⚠ {profile.wiltingPercent.toFixed(0)}% wilting</span>{/if}
								{#if profile.provenanceBreakdown['received'] > profile.memberCount * 0.7}<span class="lp-comm-warning">📥 mostly received</span>{/if}
							</div>
						{/if}
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

	<!-- Contradictions -->
	{#if contradictions.length > 0}
		<div class="lp-section">
			<button class="lp-header" onclick={() => showContradictions = !showContradictions}>
				<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={showContradictions}><polyline points="6 9 12 15 18 9"/></svg>
				<span>{$t('lens.contradictions') || 'Contradictions'}</span>
				<span class="lp-count">{contradictions.length}</span>
			</button>
			{#if showContradictions}
				<div class="lp-list">
					{#each contradictions.slice(0, 10) as [a, b]}
						<div class="lp-contradiction">
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
							<span class="lp-contradiction-text">{a} ↔ {b}</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Advanced: Layer Peeling + Tag Edges -->
	<div class="lp-section">
		<button class="lp-header" onclick={() => showAdvanced = !showAdvanced}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={showAdvanced}><polyline points="6 9 12 15 18 9"/></svg>
			<span>{$t('lens.advanced') || 'Advanced'}</span>
		</button>
		{#if showAdvanced}
			<div class="lp-advanced">
				<!-- Layer Peeling -->
				<div class="lp-adv-row">
					<label class="lp-adv-label">{$t('lens.layerPeeling') || 'Layer Peeling'}</label>
					<div class="lp-adv-slider">
						<input type="range" min="0" max="20" step="1" value={peelCount}
							oninput={(e) => { const v = parseInt((e.target as HTMLInputElement).value); onPeelChange?.(v); }} />
						<span class="lp-adv-val">{peelCount === 0 ? 'Off' : `Hide top ${peelCount}`}</span>
					</div>
				</div>
				<!-- Tag Edges Toggle -->
				<div class="lp-adv-row">
					<label class="lp-adv-label">{$t('lens.tagEdges') || 'Tag Edges'}</label>
					<button class="lp-adv-toggle" class:active={showTagEdges} onclick={() => onTagEdgesToggle?.(!showTagEdges)}>
						{showTagEdges ? 'On' : 'Off'}
					</button>
				</div>
			</div>
		{/if}
	</div>
t</div>
</div>

<style>
	.lens-panel {
		width: 280px; height: 100%; overflow: hidden;
		background: var(--background-primary); border-inline-start: 1px solid var(--background-modifier-border);
		display: flex; flex-direction: column;
		font-size: 12px;
	}
	.lp-frozen { flex-shrink: 0; padding: 12px 12px 0; }
	.lp-scrollable { flex: 1; overflow-y: auto; padding: 8px 12px 12px; display: flex; flex-direction: column; gap: 8px; }

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

	/* Community profile enhancements */
	.lp-comm { display: flex; flex-direction: column; gap: 2px; padding: 4px 6px; }
	.lp-comm-header { display: flex; align-items: center; gap: 6px; }
	.lp-comm-bar { display: flex; height: 3px; border-radius: 2px; overflow: hidden; margin-top: 1px; }
	.lp-comm-bar-seg { height: 100%; }
	.lp-comm-meta { display: flex; gap: 6px; font-size: 9px; color: var(--text-faint); }
	.lp-comm-warning { color: #ef4444; font-weight: 600; }

	/* Contradictions */
	.lp-contradiction { display: flex; align-items: center; gap: 6px; padding: 3px 6px; }
	.lp-contradiction-text { font-size: 11px; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	/* Advanced controls */
	.lp-advanced { display: flex; flex-direction: column; gap: 8px; padding: 4px 0; }
	.lp-adv-row { display: flex; align-items: center; gap: 8px; }
	.lp-adv-label { font-size: 11px; color: var(--text-muted); min-width: 80px; }
	.lp-adv-slider { display: flex; align-items: center; gap: 6px; flex: 1; }
	.lp-adv-slider input[type="range"] { flex: 1; height: 4px; }
	.lp-adv-val { font-size: 10px; color: var(--text-faint); white-space: nowrap; min-width: 70px; }
	.lp-adv-toggle {
		padding: 3px 10px; border-radius: 4px; font-size: 11px; cursor: pointer;
		border: 1px solid var(--background-modifier-border);
		background: none; color: var(--text-muted); font-family: inherit;
	}
	.lp-adv-toggle.active { background: var(--interactive-accent); color: white; border-color: var(--interactive-accent); }
</style>

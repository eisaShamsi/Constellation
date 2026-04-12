<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	let {
		onClose,
	}: {
		onClose: () => void;
	} = $props();

	interface LinkStats {
		total_links: number;
		by_type: Record<string, number>;
		by_confidence: Record<string, number>;
		with_annotation: number;
		sample_links: any[];
	}

	interface LifecycleData {
		decayed: number;
		new_dormant: number;
		lifecycle: Record<string, number>;
	}

	interface FormulationInsight {
		source_name: string;
		target_name: string;
		link_type: string;
		annotation: string;
		weight: number;
		confidence: string;
		traversal_count: number;
		last_traversed: string;
		library_name: string;
	}

	let stats = $state<LinkStats | null>(null);
	let lifecycle = $state<LifecycleData | null>(null);
	let emerging = $state<FormulationInsight[]>([]);
	let biasAlerts = $state<FormulationInsight[]>([]);
	let mostConnected = $state<FormulationInsight[]>([]);
	let weakFoundations = $state<FormulationInsight[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			const [s, l, e, b, m, w] = await Promise.all([
				invoke<LinkStats>('constellation_link_stats'),
				invoke<LifecycleData>('constellation_link_decay'),
				invoke<FormulationInsight[]>('constellation_formulation_analysis', { queryType: 'emerging', target: null }),
				invoke<FormulationInsight[]>('constellation_formulation_analysis', { queryType: 'bias_check', target: null }),
				invoke<FormulationInsight[]>('constellation_formulation_analysis', { queryType: 'most_connected', target: null }),
				invoke<FormulationInsight[]>('constellation_formulation_analysis', { queryType: 'weak_foundations', target: null }),
			]);
			stats = s;
			lifecycle = l;
			emerging = e.slice(0, 10);
			biasAlerts = b.slice(0, 10);
			mostConnected = m.slice(0, 10);
			weakFoundations = w.slice(0, 10);
		} catch (e) { console.error('[KHD]', e); }
		loading = false;
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	const typeColors: Record<string, string> = {
		relates: '#94a3b8', supports: '#16a34a', contradicts: '#ef4444',
		causes: '#f59e0b', exemplifies: '#3b82f6', generalizes: '#7c3aed',
		'derives-from': '#06b6d4', 'part-of': '#ec4899',
	};

	const confidenceColors: Record<string, string> = {
		hypothesis: '#94a3b8', evidence: '#3b82f6', established: '#16a34a', contested: '#ef4444',
	};

	const stageColors: Record<string, string> = {
		birth: '#94a3b8', growth: '#16a34a', maturity: '#7c3aed', dormancy: '#f59e0b', archived: '#ef4444',
	};
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="khd-overlay" onkeydown={handleKeydown} tabindex="-1" role="dialog">
	<div class="khd-container">
		<div class="khd-header">
			<h1>Knowledge Health</h1>
			<button class="khd-close" onclick={onClose}>
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
			</button>
		</div>

		{#if loading}
			<div class="khd-loading">Loading diagnostics...</div>
		{:else if stats && lifecycle}
			<!-- Lifecycle Cards -->
			<div class="khd-cards">
				{#each Object.entries(lifecycle.lifecycle) as [stage, count]}
					<div class="khd-card">
						<div class="khd-card-num" style:color={stageColors[stage] ?? '#666'}>{count.toLocaleString()}</div>
						<div class="khd-card-label">{stage}</div>
					</div>
				{/each}
				<div class="khd-card">
					<div class="khd-card-num" style:color="#7c3aed">{stats.total_links.toLocaleString()}</div>
					<div class="khd-card-label">Total Links</div>
				</div>
				<div class="khd-card">
					<div class="khd-card-num" style:color="#3b82f6">{stats.with_annotation.toLocaleString()}</div>
					<div class="khd-card-label">Annotated</div>
				</div>
			</div>

			<div class="khd-grid">
				<!-- Link Types -->
				<div class="khd-section">
					<h3>Link Types</h3>
					{#each Object.entries(stats.by_type).sort((a, b) => b[1] - a[1]) as [type, count]}
						<div class="khd-bar-row">
							<span class="khd-bar-label">{type}</span>
							<div class="khd-bar-track">
								<div class="khd-bar-fill" style:width="{Math.max(2, (count / stats.total_links) * 100)}%" style:background={typeColors[type] ?? '#94a3b8'}></div>
							</div>
							<span class="khd-bar-num">{count.toLocaleString()}</span>
						</div>
					{/each}
				</div>

				<!-- Confidence -->
				<div class="khd-section">
					<h3>Confidence Distribution</h3>
					{#each Object.entries(stats.by_confidence).sort((a, b) => b[1] - a[1]) as [conf, count]}
						<div class="khd-bar-row">
							<span class="khd-bar-label">{conf}</span>
							<div class="khd-bar-track">
								<div class="khd-bar-fill" style:width="{Math.max(2, (count / stats.total_links) * 100)}%" style:background={confidenceColors[conf] ?? '#94a3b8'}></div>
							</div>
							<span class="khd-bar-num">{count.toLocaleString()}</span>
						</div>
					{/each}
				</div>

				<!-- Most Connected -->
				<div class="khd-section">
					<h3>Knowledge Hubs</h3>
					{#if mostConnected.length === 0}
						<p class="khd-empty">No knowledge hubs yet</p>
					{:else}
						{#each mostConnected as item}
							<div class="khd-insight-row">
								<span class="khd-insight-name" dir="auto">{item.target_name}</span>
								<span class="khd-insight-meta">{item.annotation}</span>
							</div>
						{/each}
					{/if}
				</div>

				<!-- Emerging Ideas -->
				<div class="khd-section">
					<h3>Emerging Ideas</h3>
					{#if emerging.length === 0}
						<p class="khd-empty">No emerging ideas yet — follow more links!</p>
					{:else}
						{#each emerging as item}
							<div class="khd-insight-row">
								<span class="khd-insight-name" dir="auto">{item.source_name}</span>
								<span class="khd-insight-badge" style:background={typeColors[item.link_type] ?? '#94a3b8'}>{item.link_type}</span>
								<span class="khd-insight-name" dir="auto">{item.target_name}</span>
								<span class="khd-insight-meta">w:{item.weight.toFixed(1)}</span>
							</div>
						{/each}
					{/if}
				</div>

				<!-- Weak Foundations -->
				<div class="khd-section">
					<h3>Weak Foundations</h3>
					{#if weakFoundations.length === 0}
						<p class="khd-empty">No weak foundations detected</p>
					{:else}
						{#each weakFoundations as item}
							<div class="khd-insight-row">
								<span class="khd-insight-name" dir="auto">{item.source_name} → {item.target_name}</span>
								<span class="khd-insight-meta">hypothesis, w:{item.weight.toFixed(1)}</span>
							</div>
						{/each}
					{/if}
				</div>

				<!-- Bias Alerts -->
				<div class="khd-section">
					<h3>Bias Alerts</h3>
					{#if biasAlerts.length === 0}
						<p class="khd-empty">No echo chambers detected</p>
					{:else}
						{#each biasAlerts as item}
							<div class="khd-insight-row">
								<span class="khd-insight-name" dir="auto">{item.target_name}</span>
								<span class="khd-insight-meta">{item.annotation}</span>
							</div>
						{/each}
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.khd-overlay {
		position: fixed;
		inset: 0;
		z-index: 9998;
		background: var(--background-primary);
		overflow-y: auto;
	}
	.khd-container {
		max-width: 1000px;
		margin: 0 auto;
		padding: 24px 32px;
	}
	.khd-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
	}
	.khd-header h1 {
		margin: 0;
		font-size: 1.4rem;
		font-weight: 700;
	}
	.khd-close {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-muted);
		padding: 4px;
		border-radius: 4px;
	}
	.khd-close:hover { color: var(--text-normal); }
	.khd-loading {
		text-align: center;
		padding: 60px;
		color: var(--text-muted);
	}
	.khd-cards {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
		margin-bottom: 24px;
	}
	.khd-card {
		flex: 1;
		min-width: 100px;
		background: var(--background-secondary);
		border-radius: 10px;
		padding: 16px;
		text-align: center;
	}
	.khd-card-num {
		font-size: 1.6rem;
		font-weight: 700;
		line-height: 1.2;
	}
	.khd-card-label {
		font-size: 0.72rem;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-top: 4px;
	}
	.khd-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}
	.khd-section {
		background: var(--background-secondary);
		border-radius: 10px;
		padding: 16px;
	}
	.khd-section h3 {
		margin: 0 0 12px;
		font-size: 0.88rem;
		font-weight: 600;
	}
	.khd-bar-row {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
	}
	.khd-bar-label {
		width: 90px;
		font-size: 0.75rem;
		color: var(--text-muted);
		text-align: end;
		flex-shrink: 0;
	}
	.khd-bar-track {
		flex: 1;
		height: 8px;
		background: var(--background-modifier-border);
		border-radius: 4px;
		overflow: hidden;
	}
	.khd-bar-fill {
		height: 100%;
		border-radius: 4px;
		transition: width 0.3s ease;
	}
	.khd-bar-num {
		width: 60px;
		font-size: 0.72rem;
		font-weight: 600;
		color: var(--text-muted);
		text-align: end;
	}
	.khd-empty {
		font-size: 0.82rem;
		color: var(--text-faint);
		font-style: italic;
		margin: 8px 0;
	}
	.khd-insight-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 0;
		font-size: 0.78rem;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.khd-insight-row:last-child { border-bottom: none; }
	.khd-insight-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.khd-insight-badge {
		font-size: 0.65rem;
		color: white;
		padding: 1px 6px;
		border-radius: 4px;
		font-weight: 600;
		flex-shrink: 0;
	}
	.khd-insight-meta {
		font-size: 0.68rem;
		color: var(--text-faint);
		flex-shrink: 0;
	}
</style>

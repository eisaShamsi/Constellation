<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import LinkTypePill from './LinkTypePill.svelte';
	import { linkTypeColor } from '$lib/libraries/linkTypeRegistry';

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

	// MIG-073 — the panel reads ONE cached snapshot (6 tiny rows) instead of
	// firing six live aggregates at note_links on every open (the old path
	// cold-read a 1.7 GB table for ~11s while holding the DB mutex). The
	// backend keeps the snapshot fresh (stale-while-revalidate + reconcile
	// hooks); `ready: false` only happens on the first-ever open while the
	// one-off background population runs.
	interface KhSnapshot {
		ready: boolean;
		stats?: LinkStats | null;
		lifecycle?: LifecycleData | null;
		emerging?: FormulationInsight[] | null;
		bias_check?: FormulationInsight[] | null;
		most_connected?: FormulationInsight[] | null;
		weak_foundations?: FormulationInsight[] | null;
	}

	async function loadSnapshot(): Promise<boolean> {
		const snap = await invoke<KhSnapshot>('constellation_knowledge_health_snapshot');
		if (!snap.ready) return false;
		stats = snap.stats ?? null;
		lifecycle = snap.lifecycle ?? null;
		emerging = (snap.emerging ?? []).slice(0, 10);
		biasAlerts = (snap.bias_check ?? []).slice(0, 10);
		mostConnected = (snap.most_connected ?? []).slice(0, 10);
		weakFoundations = (snap.weak_foundations ?? []).slice(0, 10);
		return true;
	}

	onMount(() => {
		let unlisten: (() => void) | null = null;
		let retryTimer: ReturnType<typeof setInterval> | null = null;
		let disposed = false;
		const stopRetry = () => {
			if (retryTimer) { clearInterval(retryTimer); retryTimer = null; }
		};
		const tryLoad = async () => {
			try {
				if (await loadSnapshot()) { loading = false; stopRetry(); }
			} catch (e) { console.error('[KHD]', e); loading = false; stopRetry(); }
		};
		(async () => {
			// §P3 — listen the whole time the panel is open (registered BEFORE
			// the first fetch so no event is missed): a stale-while-revalidate
			// refresh that lands mid-view updates the numbers in place.
			const { listen } = await import('@tauri-apps/api/event');
			const un = await listen('kh-snapshot-ready', tryLoad);
			if (disposed) { un(); return; }
			unlisten = un;
			await tryLoad();
			if (disposed || !loading) return;
			// First-ever open: the cache is populating in the background. The
			// event above delivers it; the slow poll is a self-healing fallback
			// (the snapshot IPC re-kicks a failed recompute on every call).
			retryTimer = setInterval(tryLoad, 5000);
		})();
		return () => { disposed = true; stopRetry(); unlisten?.(); };
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	// MIG-067 §H.3 — link-type colours come from the §G registry (the single colour source),
	// via linkTypeColor() for the bar fills + the shared LinkTypePill for the badges. The old
	// hardcoded map drifted from the registry (supports was green here, blue everywhere else).

	const confidenceColors: Record<string, string> = {
		hypothesis: '#94a3b8', evidence: '#3b82f6', established: '#16a34a', contested: '#ef4444',
	};

	// MIG-014 §2F — Living Link 6-stage colours. Was missing `spark`; had
	// `archived` instead of the canonical `archival`. Aligned now with the
	// LIVING_LINK_BASELINE order.
	const stageColors: Record<string, string> = {
		spark: '#a78bfa', birth: '#94a3b8', growth: '#16a34a', maturity: '#7c3aed', dormancy: '#f59e0b', archival: '#ef4444',
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
								<div class="khd-bar-fill" style:width="{Math.max(2, (count / stats.total_links) * 100)}%" style:background={linkTypeColor(type)}></div>
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
								<LinkTypePill id={item.link_type} />
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
		/* 1 1 auto: basis = content width, so a 7-digit count widens its own
		   card instead of bleeding past the rounded edge (flex:1 squeezed all
		   cards equal regardless of content); flex-wrap handles overflow. */
		flex: 1 1 auto;
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
	.khd-insight-meta {
		font-size: 0.68rem;
		color: var(--text-faint);
		flex-shrink: 0;
	}
</style>

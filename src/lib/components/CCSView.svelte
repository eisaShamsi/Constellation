<script lang="ts">
	// MIG-074 — CCS, the Constellation Circulatory System (الجهاز الدوري).
	// The circulatory register of the Cognitive-Engine Connection question:
	// the seven ratified registers (Concept Paper v1.1 §6) read from ONE
	// cached snapshot (`constellation_ccs_snapshot`, the MIG-073
	// link_stats_cache layer) — no note_links scan on open (Perf Rule 8).
	// Writes: none in this view (Restore lands in §C via the existing
	// lifecycle commands). Opening a note from a register NEVER fires
	// `constellation_link_traverse` — CCS observes circulation; it must not
	// feed the metric it displays (invariant I2b).
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import LinkTypePill from './LinkTypePill.svelte';
	import { linkTypesStore, linkTypeColor, getLinkType, getLinkTypes } from '$lib/libraries/linkTypeRegistry';

	let {
		onClose,
		onNoteClick,
		onOpenKnowledgeHealth,
	}: {
		onClose: () => void;
		onNoteClick?: (path: string, libraryName: string) => void;
		onOpenKnowledgeHealth?: () => void;
	} = $props();

	interface CcsRow {
		source_name: string;
		source_path: string;
		target_name: string;
		target_path: string;
		link_type: string;
		annotation: string;
		weight: number;
		confidence: string;
		traversal_count: number;
		last_traversed: string;
		library_name: string;
	}
	interface CcsList { total: number; rows: CcsRow[] }
	interface CcsTiers { fresh: number; emerging: number; established: number; load_bearing: number; stale: number }
	interface CcsSnapshot {
		ready: boolean;
		stats?: { total_links: number; by_type: Record<string, number>; by_confidence: Record<string, number>; with_annotation: number } | null;
		living?: CcsList | null;
		load_bearing?: CcsList | null;
		cooling?: CcsList | null;
		contested?: CcsList | null;
		tiers?: CcsTiers | null;
		retired?: CcsList | null;
	}

	let totalLinks = $state(0);
	let byType = $state<Record<string, number>>({});
	let byConfidence = $state<Record<string, number>>({});
	let living = $state<CcsList>({ total: 0, rows: [] });
	let loadBearing = $state<CcsList>({ total: 0, rows: [] });
	let cooling = $state<CcsList>({ total: 0, rows: [] });
	let contested = $state<CcsList>({ total: 0, rows: [] });
	let tiers = $state<CcsTiers | null>(null);
	let retired = $state<CcsList>({ total: 0, rows: [] });
	let loading = $state(true);

	async function loadSnapshot(): Promise<boolean> {
		const snap = await invoke<CcsSnapshot>('constellation_ccs_snapshot');
		if (!snap.ready) return false;
		totalLinks = snap.stats?.total_links ?? 0;
		byType = snap.stats?.by_type ?? {};
		byConfidence = snap.stats?.by_confidence ?? {};
		living = snap.living ?? { total: 0, rows: [] };
		loadBearing = snap.load_bearing ?? { total: 0, rows: [] };
		cooling = snap.cooling ?? { total: 0, rows: [] };
		contested = snap.contested ?? { total: 0, rows: [] };
		tiers = snap.tiers ?? null;
		retired = snap.retired ?? { total: 0, rows: [] };
		return true;
	}

	onMount(() => {
		// The KHD §P3 flow: register the snapshot-ready listener BEFORE the
		// first fetch (no missed-event race); a stale snapshot renders
		// instantly and updates in place when the background refresh lands;
		// a slow poll self-heals a failed first population. Everything is
		// cleaned on destroy (Rule 4) — and the component only exists while
		// open ({#if showCCS}), so a closed CCS does zero IPC (LL-022).
		let unlisten: (() => void) | null = null;
		let retryTimer: ReturnType<typeof setInterval> | null = null;
		let disposed = false;
		const stopRetry = () => {
			if (retryTimer) { clearInterval(retryTimer); retryTimer = null; }
		};
		const tryLoad = async () => {
			try {
				if (await loadSnapshot()) { loading = false; stopRetry(); }
			} catch (e) { console.error('[CCS]', e); loading = false; stopRetry(); }
		};
		(async () => {
			const { listen } = await import('@tauri-apps/api/event');
			const un = await listen('kh-snapshot-ready', tryLoad);
			if (disposed) { un(); return; }
			unlisten = un;
			await tryLoad();
			if (disposed || !loading) return;
			retryTimer = setInterval(tryLoad, 5000);
		})();
		return () => { disposed = true; stopRetry(); unlisten?.(); };
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	function openRow(row: CcsRow) {
		if (row.source_path && onNoteClick) onNoteClick(row.source_path, row.library_name);
	}

	// ── label resolvers (the KHD pattern: locale → registry label → raw id) ──
	const typeLabel = $derived((id: string) => {
		void $linkTypesStore;
		const k = `linkTypes.${id.toLowerCase()}`;
		const tr = $t(k);
		return tr === k ? (getLinkType(id)?.label ?? id) : tr;
	});
	const confLabel = $derived((id: string) => {
		const k = `knowledgeHealth.confidence.${id}`;
		const tr = $t(k);
		return tr === k ? id : tr;
	});

	const confidenceColors: Record<string, string> = {
		hypothesis: '#94a3b8', evidence: '#3b82f6', established: '#16a34a', contested: '#ef4444',
	};
	const CONFIDENCE_ORDER = ['hypothesis', 'evidence', 'established', 'contested'];

	const TIER_ORDER: Array<keyof CcsTiers> = ['fresh', 'emerging', 'established', 'load_bearing', 'stale'];
	const tierColors: Record<string, string> = {
		fresh: '#94a3b8', emerging: '#16a34a', established: '#3b82f6', load_bearing: '#7c3aed', stale: '#f59e0b',
	};
	const tierKey: Record<string, string> = {
		fresh: 'fresh', emerging: 'emerging', established: 'established', load_bearing: 'loadBearing', stale: 'stale',
	};
	const tiersTotal = $derived(
		tiers ? TIER_ORDER.reduce((s, k) => s + (tiers![k] ?? 0), 0) : 0
	);

	// The Acts of Inquiry — REGISTRY canonical order (never count-sorted; the
	// vocabulary IS the picture). Every by_type id outside the registry
	// (legacy 'relates', 'associative', '', strays) aggregates into ONE
	// "Open inquiries" line — the untyped link is the question, never a
	// defect (Living-Link canon §4 / CCS guardrail 1).
	const actsRows = $derived.by(() => {
		void $linkTypesStore;
		const counts = { ...byType };
		const rows: Array<{ id: string; count: number }> = [];
		for (const def of getLinkTypes()) {
			const n = counts[def.id] ?? 0;
			delete counts[def.id];
			if (n > 0) rows.push({ id: def.id, count: n });
		}
		const open = Object.values(counts).reduce((s, n) => s + n, 0);
		return { rows, open };
	});

	function idleDays(lastTraversed: string): number {
		const parsed = Date.parse(lastTraversed);
		if (Number.isNaN(parsed)) return 0;
		return Math.max(0, Math.round((Date.now() - parsed) / 86_400_000));
	}
	function walkedDate(lastTraversed: string): string {
		const parsed = Date.parse(lastTraversed);
		if (Number.isNaN(parsed)) return '';
		return new Date(parsed).toLocaleDateString();
	}
	const pct = (n: number, total: number) => `${Math.max(2, total > 0 ? (n / total) * 100 : 0)}%`;
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="ccs-overlay" onkeydown={handleKeydown} tabindex="-1" role="dialog">
	<div class="ccs-container">
		<div class="ccs-header">
			<div class="ccs-header-titles">
				<h1>{$t('ccs.title')}</h1>
				<span class="ccs-scope">{$t('ccs.scope')} · {$t('ccs.meta.total', { n: totalLinks.toLocaleString() })}</span>
			</div>
			<div class="ccs-header-actions">
				{#if onOpenKnowledgeHealth}
					<button class="ccs-kh-link" onclick={onOpenKnowledgeHealth}>{$t('knowledgeHealth.title')} →</button>
				{/if}
				<button class="ccs-close" onclick={onClose} title={$t('common.close')} aria-label={$t('common.close')}>
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
				</button>
			</div>
		</div>

		{#if loading}
			<div class="ccs-loading">{$t('ccs.computing')}</div>
		{:else}
			<div class="ccs-grid">
				<!-- 1 · Living Connections -->
				<div class="ccs-section">
					<h3>{$t('ccs.living.title')}</h3>
					<p class="ccs-question">{$t('ccs.living.question')}</p>
					{#if living.rows.length === 0}
						<p class="ccs-empty">{$t('ccs.living.empty')}</p>
					{:else}
						{#each living.rows as row}
							<button class="ccs-row" class:ccs-row-link={!!row.source_path} onclick={() => openRow(row)}>
								<span class="ccs-row-name" dir="auto">{row.source_name}</span>
								<LinkTypePill id={row.link_type} />
								<span class="ccs-row-name" dir="auto">{row.target_name}</span>
								<span class="ccs-row-meta">{$t('ccs.meta.walks', { n: row.traversal_count.toLocaleString() })}{#if walkedDate(row.last_traversed)} · {$t('ccs.meta.lastWalked', { date: walkedDate(row.last_traversed) })}{/if}</span>
							</button>
						{/each}
					{/if}
				</div>

				<!-- 2 · Load-Bearing Reasoning -->
				<div class="ccs-section">
					<h3>{$t('ccs.loadBearing.title')}</h3>
					<p class="ccs-question">{$t('ccs.loadBearing.question')}</p>
					{#if loadBearing.rows.length === 0}
						<p class="ccs-empty">{$t('ccs.loadBearing.empty')}</p>
					{:else}
						{#each loadBearing.rows as row}
							<button class="ccs-row" class:ccs-row-link={!!row.source_path} onclick={() => openRow(row)}>
								<span class="ccs-row-name" dir="auto">{row.source_name}</span>
								<LinkTypePill id={row.link_type} />
								<span class="ccs-row-name" dir="auto">{row.target_name}</span>
								<span class="ccs-row-meta">{$t('knowledgeHealth.weightAbbrev')}:{row.weight.toFixed(1)} · {$t('ccs.meta.walks', { n: row.traversal_count.toLocaleString() })}</span>
							</button>
						{/each}
					{/if}
				</div>

				<!-- 3 · Cooling Inquiries -->
				<div class="ccs-section">
					<h3>{$t('ccs.cooling.title')}</h3>
					<p class="ccs-question">{$t('ccs.cooling.question')}</p>
					{#if cooling.rows.length === 0}
						<p class="ccs-empty">{$t('ccs.cooling.empty')}</p>
					{:else}
						{#each cooling.rows as row}
							<button class="ccs-row" class:ccs-row-link={!!row.source_path} onclick={() => openRow(row)}>
								<span class="ccs-row-name" dir="auto">{row.source_name}</span>
								<LinkTypePill id={row.link_type} />
								<span class="ccs-row-name" dir="auto">{row.target_name}</span>
								<span class="ccs-row-meta">{$t('ccs.meta.idleDays', { n: idleDays(row.last_traversed).toLocaleString() })}</span>
							</button>
						{/each}
						{#if cooling.total > cooling.rows.length}
							<p class="ccs-more-note">{$t('ccs.meta.total', { n: cooling.total.toLocaleString() })}</p>
						{/if}
					{/if}
				</div>

				<!-- 4 · Conviction & Doubt -->
				<div class="ccs-section">
					<h3>{$t('ccs.conviction.title')}</h3>
					<p class="ccs-question">{$t('ccs.conviction.question')}</p>
					{#each CONFIDENCE_ORDER as conf}
						{@const count = byConfidence[conf] ?? 0}
						<div class="ccs-bar-row">
							<span class="ccs-bar-label" dir="auto">{confLabel(conf)}</span>
							<div class="ccs-bar-track">
								<div class="ccs-bar-fill" style:width={pct(count, totalLinks)} style:background={confidenceColors[conf]}></div>
							</div>
							<span class="ccs-bar-num">{count.toLocaleString()}</span>
						</div>
					{/each}
					{#if contested.rows.length === 0}
						<p class="ccs-empty">{$t('ccs.conviction.empty')}</p>
					{:else}
						{#each contested.rows.slice(0, 8) as row}
							<button class="ccs-row" class:ccs-row-link={!!row.source_path} onclick={() => openRow(row)}>
								<span class="ccs-row-name" dir="auto">{row.source_name}</span>
								<LinkTypePill id={row.link_type} />
								<span class="ccs-row-name" dir="auto">{row.target_name}</span>
								<span class="ccs-row-meta">{confLabel('contested')}</span>
							</button>
						{/each}
					{/if}
				</div>

				<!-- 5 · The Life of a Connection -->
				<div class="ccs-section">
					<h3>{$t('ccs.life.title')}</h3>
					<p class="ccs-question">{$t('ccs.life.question')}</p>
					{#if tiers}
						{#each TIER_ORDER as k}
							{@const count = tiers[k] ?? 0}
							<div class="ccs-bar-row">
								<span class="ccs-bar-label" dir="auto">{$t(`ccs.tier.${tierKey[k]}`)}</span>
								<div class="ccs-bar-track">
									<div class="ccs-bar-fill" style:width={pct(count, tiersTotal)} style:background={tierColors[k]}></div>
								</div>
								<span class="ccs-bar-num">{count.toLocaleString()}</span>
							</div>
						{/each}
					{/if}
				</div>

				<!-- 6 · Retired Reasoning -->
				<div class="ccs-section">
					<h3>{$t('ccs.retired.title')}</h3>
					<p class="ccs-question">{$t('ccs.retired.question')}</p>
					{#if retired.rows.length === 0}
						<p class="ccs-empty">{$t('ccs.retired.empty')}</p>
					{:else}
						{#each retired.rows.slice(0, 20) as row}
							<button class="ccs-row" class:ccs-row-link={!!row.source_path} onclick={() => openRow(row)}>
								<span class="ccs-row-name" dir="auto">{row.source_name}</span>
								<LinkTypePill id={row.link_type} />
								<span class="ccs-row-name" dir="auto">{row.target_name}</span>
								<span class="ccs-row-meta">{#if walkedDate(row.last_traversed)}{$t('ccs.meta.lastWalked', { date: walkedDate(row.last_traversed) })}{/if}</span>
							</button>
						{/each}
						{#if retired.total > 20}
							<p class="ccs-more-note">{$t('ccs.meta.total', { n: retired.total.toLocaleString() })}</p>
						{/if}
					{/if}
				</div>

				<!-- 7 · The Acts of Inquiry -->
				<div class="ccs-section ccs-section-wide">
					<h3>{$t('ccs.acts.title')}</h3>
					<p class="ccs-question">{$t('ccs.acts.question')}</p>
					{#each actsRows.rows as { id, count }}
						<div class="ccs-bar-row">
							<span class="ccs-bar-label" dir="auto">{typeLabel(id)}</span>
							<div class="ccs-bar-track">
								<div class="ccs-bar-fill" style:width={pct(count, totalLinks)} style:background={linkTypeColor(id)}></div>
							</div>
							<span class="ccs-bar-num">{count.toLocaleString()}</span>
						</div>
					{/each}
					{#if actsRows.open > 0}
						<div class="ccs-bar-row">
							<span class="ccs-bar-label ccs-open-label" dir="auto">{$t('ccs.openInquiries')}</span>
							<div class="ccs-bar-track">
								<div class="ccs-bar-fill" style:width={pct(actsRows.open, totalLinks)} style:background="#94a3b8"></div>
							</div>
							<span class="ccs-bar-num">{actsRows.open.toLocaleString()}</span>
						</div>
						<p class="ccs-hint">{$t('ccs.openInquiriesHint')}</p>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.ccs-overlay {
		position: fixed;
		inset: 0;
		z-index: 9998;
		background: var(--background-primary);
		overflow-y: auto;
	}
	.ccs-container {
		max-width: 1000px;
		margin: 0 auto;
		padding: 24px 32px;
	}
	.ccs-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
		gap: 12px;
	}
	.ccs-header h1 {
		margin: 0;
		font-size: 1.4rem;
		font-weight: 700;
	}
	.ccs-scope {
		display: block;
		font-size: 0.72rem;
		color: var(--text-muted);
		margin-top: 2px;
	}
	.ccs-header-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}
	.ccs-kh-link {
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		color: var(--text-muted);
		font-size: 0.75rem;
		padding: 5px 10px;
		cursor: pointer;
	}
	.ccs-kh-link:hover { color: var(--text-normal); }
	.ccs-close {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-muted);
		padding: 4px;
		border-radius: 4px;
	}
	.ccs-close:hover { color: var(--text-normal); }
	.ccs-loading {
		text-align: center;
		padding: 60px;
		color: var(--text-muted);
	}
	.ccs-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}
	.ccs-section {
		background: var(--background-secondary);
		border-radius: 10px;
		padding: 16px;
	}
	.ccs-section-wide { grid-column: 1 / -1; }
	.ccs-section h3 {
		margin: 0;
		font-size: 0.88rem;
		font-weight: 600;
	}
	.ccs-question {
		margin: 2px 0 12px;
		font-size: 0.72rem;
		color: var(--text-faint);
		font-style: italic;
	}
	.ccs-bar-row {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
	}
	.ccs-bar-label {
		width: 110px;
		font-size: 0.75rem;
		color: var(--text-muted);
		text-align: end;
		flex-shrink: 0;
	}
	.ccs-open-label { font-style: italic; }
	.ccs-bar-track {
		flex: 1;
		height: 8px;
		background: var(--background-modifier-border);
		border-radius: 4px;
		overflow: hidden;
	}
	.ccs-bar-fill {
		height: 100%;
		border-radius: 4px;
		transition: width 0.3s ease;
	}
	.ccs-bar-num {
		width: 60px;
		font-size: 0.72rem;
		font-weight: 600;
		color: var(--text-muted);
		text-align: end;
	}
	.ccs-empty {
		font-size: 0.82rem;
		color: var(--text-faint);
		font-style: italic;
		margin: 8px 0;
	}
	.ccs-hint {
		font-size: 0.7rem;
		color: var(--text-faint);
		font-style: italic;
		margin: 6px 0 0;
	}
	.ccs-more-note {
		font-size: 0.7rem;
		color: var(--text-faint);
		margin: 6px 0 0;
		text-align: end;
	}
	.ccs-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 0;
		font-size: 0.78rem;
		border-bottom: 1px solid var(--background-modifier-border);
		/* rows are buttons (a11y + keyboard) styled as list lines */
		width: 100%;
		background: none;
		border-inline: none;
		border-top: none;
		color: inherit;
		text-align: start;
		cursor: default;
		font-family: inherit;
	}
	.ccs-row-link { cursor: pointer; }
	.ccs-row-link:hover .ccs-row-name { color: var(--text-accent, var(--interactive-accent)); }
	.ccs-row:last-of-type { border-bottom: none; }
	.ccs-row-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		/* The span stretches (flex:1) and carries dir="auto" for the note
		   name's own bidi — but the LINE must align to the ROW's direction
		   (the KHD RTL lesson, 2026-06-10). */
		text-align: match-parent;
	}
	.ccs-row-meta {
		font-size: 0.68rem;
		color: var(--text-faint);
		flex-shrink: 0;
	}
</style>

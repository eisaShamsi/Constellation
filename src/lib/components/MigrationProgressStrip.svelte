<script lang="ts">
	// MIG-015 §1C — Status-bar strip that listens for the deferred
	// term_vocab v2 sentinel migration's progress events. The strip is
	// hidden until a `start` event arrives, shows running counts via
	// `progress` events, and disappears 4 seconds after `done`.
	//
	// Per Law 2.7 — single source of truth: state lives entirely in
	// this component (the migration is a transient UI signal, not a
	// persisted property). The Tauri event channel is the canonical
	// owner; this component is a derived view of those events.
	import { onMount, onDestroy } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { t } from '$lib/i18n';

	// MIG-041: `vacuum_start` / `vacuum_done` are the one-time disk-compaction
	// (VACUUM) phase that runs after the bigram purge. VACUUM is opaque (no
	// chunk progress), so those phases show an indeterminate "Compacting…"
	// label with no counts.
	type Phase = 'start' | 'progress' | 'done' | 'vacuum_start' | 'vacuum_done';
	interface ProgressEvent {
		phase: Phase;
		total?: number;
		completed?: number;
	}

	let visible = $state(false);
	let phase = $state<Phase | null>(null);
	let total = $state(0);
	let completed = $state(0);

	let hideTimer: ReturnType<typeof setTimeout> | null = null;
	let unlisten: UnlistenFn | null = null;

	const fmt = (n: number) => n.toLocaleString();
	const cancelHide = () => { if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; } };
	const scheduleHide = () => {
		cancelHide();
		hideTimer = setTimeout(() => { visible = false; phase = null; }, 4000);
	};

	onMount(async () => {
		unlisten = await listen<ProgressEvent>('migration:term_vocab_v2', (ev) => {
			const p = ev.payload;
			phase = p.phase;
			if (typeof p.total === 'number') total = p.total;
			if (p.phase === 'start') {
				completed = 0;
				visible = true;
				cancelHide();
			} else if (p.phase === 'progress') {
				completed = p.completed ?? completed;
			} else if (p.phase === 'vacuum_start') {
				// Compaction begins — keep the strip up (cancel any pending
				// hide scheduled by the purge's `done`) with no counts.
				visible = true;
				cancelHide();
			} else if (p.phase === 'done' || p.phase === 'vacuum_done') {
				if (p.phase === 'done') completed = total;
				scheduleHide();
			}
		});
	});

	onDestroy(() => {
		unlisten?.();
		if (hideTimer) clearTimeout(hideTimer);
	});
</script>

{#if visible}
	<div class="mig-progress-strip" role="status" aria-live="polite">
		<span class="mig-progress-label">
			{#if phase === 'vacuum_start'}
				{$t('migrationProgress.termVocabV2.compacting') || 'Compacting search index…'}
			{:else if phase === 'done' || phase === 'vacuum_done'}
				{$t('migrationProgress.termVocabV2.done')}
			{:else}
				{$t('migrationProgress.termVocabV2.label')}
			{/if}
		</span>
		{#if phase === 'start' || phase === 'progress'}
			<span class="mig-progress-counts">— {fmt(completed)} / {fmt(total)}</span>
		{/if}
	</div>
{/if}

<style>
	.mig-progress-strip {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.75rem;
		color: var(--text-muted);
	}
	.mig-progress-counts {
		font-variant-numeric: tabular-nums;
	}
</style>

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

	interface ProgressEvent {
		phase: 'start' | 'progress' | 'done';
		total: number;
		completed?: number;
	}

	let visible = $state(false);
	let phase = $state<'start' | 'progress' | 'done' | null>(null);
	let total = $state(0);
	let completed = $state(0);

	let hideTimer: ReturnType<typeof setTimeout> | null = null;
	let unlisten: UnlistenFn | null = null;

	const fmt = (n: number) => n.toLocaleString();

	onMount(async () => {
		unlisten = await listen<ProgressEvent>('migration:term_vocab_v2', (ev) => {
			const p = ev.payload;
			phase = p.phase;
			total = p.total;
			if (p.phase === 'start') {
				completed = 0;
				visible = true;
			} else if (p.phase === 'progress') {
				completed = p.completed ?? completed;
			} else if (p.phase === 'done') {
				completed = total;
				if (hideTimer) clearTimeout(hideTimer);
				hideTimer = setTimeout(() => {
					visible = false;
					phase = null;
				}, 4000);
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
			{phase === 'done'
				? $t('migrationProgress.termVocabV2.done')
				: $t('migrationProgress.termVocabV2.label')}
		</span>
		{#if phase !== 'done'}
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

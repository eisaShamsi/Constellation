<script lang="ts">
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import VirtualList from './VirtualList.svelte';

	interface DueNote {
		note_path: string;
		note_name: string;
		reason: string;
		days_overdue: number;
		stratum: number;
		last_reviewed: string | null;
		// MIG-083 §D — Mode-2 staleness "why" (present only when reason === 'stale').
		// The §F two-lens reviewer renders the full "stale because …" line from these.
		stale_trigger_name?: string | null;
		stale_trigger_type?: string | null;
		stale_changed_on?: string | null;
	}

	let {
		dueNotes = [] as DueNote[],
		onNoteClick,
		onRefresh,
	}: {
		dueNotes?: DueNote[];
		onNoteClick?: (path: string, name: string) => void;
		onRefresh?: () => void;
	} = $props();

	let showResurfacing = $state(true);
	let showCheckpoints = $state(true);
	let showNeverReviewed = $state(true);

	// MIG-080 — show the FULL list (Boss: no "+N more"). Plain render for normal-size
	// sections; fall back to a VirtualList window ONLY when a section is huge — a fresh
	// universe can make "never reviewed" the whole library (get_due_notes has no cap), which
	// would re-introduce the §C.2c freeze if rendered plain (Rule 3). (§F will split this
	// panel into the open note's status + a full-page reviewer.)
	const VLIST_THRESHOLD = 80;
	function rpRowHeight(): number { return 30; }

	const resurfacing = $derived(dueNotes.filter(n => n.reason === 'interval_due' || n.reason === 'stale'));
	const checkpoints = $derived(dueNotes.filter(n => n.reason === 'checkpoint'));
	const neverReviewed = $derived(dueNotes.filter(n => n.reason === 'never_reviewed'));

	async function markReviewed(path: string) {
		try {
			await invoke('mark_reviewed', { notePath: path });
			onRefresh?.();
		} catch {}
	}

	async function snooze(path: string) {
		try {
			await invoke('snooze_note', { notePath: path, days: 7 });
			onRefresh?.();
		} catch {}
	}

	async function dismiss(path: string) {
		try {
			await invoke('dismiss_note', { notePath: path });
			onRefresh?.();
		} catch {}
	}

	function reasonIcon(reason: string): string {
		return reason === 'interval_due' ? '🔄' : reason === 'stale' ? '🥀' : reason === 'checkpoint' ? '🧠' : '📝';
	}
</script>

{#snippet reviewItem(note: DueNote, kind: string, withDismiss: boolean)}
	<div class="rp-item">
		<button class="rp-name" onclick={() => onNoteClick?.(note.note_path, note.note_name)}>
			<span class="rp-reason">{reasonIcon(note.reason)}</span>
			{note.note_name}
		</button>
		<span class="rp-detail">{kind === 'checkpoint' ? ($t('reviewPanel.checkpointHint') || 'Do you still hold this view?') : kind === 'overdue' ? `${note.days_overdue}d overdue` : `${note.days_overdue}d old`}</span>
		<div class="rp-actions">
			<button class="rp-action" title={$t('reviewPanel.reviewed') || 'Reviewed'} onclick={() => markReviewed(note.note_path)}>✓</button>
			<button class="rp-action" title={$t('reviewPanel.snooze') || 'Snooze 7d'} onclick={() => snooze(note.note_path)}>👁</button>
			{#if withDismiss}<button class="rp-action" title={$t('reviewPanel.dismiss') || 'Dismiss'} onclick={() => dismiss(note.note_path)}>🗄️</button>{/if}
		</div>
	</div>
{/snippet}

{#snippet section(items: DueNote[], kind: string, withDismiss: boolean)}
	{#if items.length > VLIST_THRESHOLD}
		<div class="rp-vlist-wrap">
			<VirtualList items={items} getItemHeight={rpRowHeight} overscan={10}>
				{#snippet row(note, _i)}{@render reviewItem(note, kind, withDismiss)}{/snippet}
			</VirtualList>
		</div>
	{:else}
		{#each items as note}{@render reviewItem(note, kind, withDismiss)}{/each}
	{/if}
{/snippet}

<div class="rp-panel">
	{#if dueNotes.length === 0}
		<div class="rp-empty">
			<div class="rp-empty-icon">✅</div>
			<div class="rp-empty-text">{$t('reviewPanel.allCaughtUp') || 'All caught up! No notes due for review.'}</div>
		</div>
	{:else}
	{#if resurfacing.length > 0}
	<div class="rp-section">
		<button class="rp-header" onclick={() => showResurfacing = !showResurfacing}>
			<span class="rp-chevron" class:collapsed={!showResurfacing}>▾</span>
			<span>🔄 {$t('reviewPanel.resurfacing') || 'Due for Review'}</span>
			<span class="rp-count">{resurfacing.length}</span>
		</button>
		{#if showResurfacing}{@render section(resurfacing, 'overdue', true)}{/if}
	</div>
	{/if}
	{#if checkpoints.length > 0}
	<div class="rp-section">
		<button class="rp-header" onclick={() => showCheckpoints = !showCheckpoints}>
			<span class="rp-chevron" class:collapsed={!showCheckpoints}>▾</span>
			<span>🧠 {$t('reviewPanel.checkpoints') || 'Mental Model Checkpoints'}</span>
			<span class="rp-count">{checkpoints.length}</span>
		</button>
		{#if showCheckpoints}{@render section(checkpoints, 'checkpoint', false)}{/if}
	</div>
	{/if}
	{#if neverReviewed.length > 0}
	<div class="rp-section">
		<button class="rp-header" onclick={() => showNeverReviewed = !showNeverReviewed}>
			<span class="rp-chevron" class:collapsed={!showNeverReviewed}>▾</span>
			<span>📝 {$t('reviewPanel.neverReviewed') || 'Never Reviewed'}</span>
			<span class="rp-count">{neverReviewed.length}</span>
		</button>
		{#if showNeverReviewed}{@render section(neverReviewed, 'old', true)}{/if}
	</div>
	{/if}
	{/if}
</div>

<style>
	.rp-panel { padding: 8px 0; flex: 1; min-height: 0; overflow-y: auto; } /* self-scrolling — host column is overflow:hidden */
	.rp-empty { text-align: center; padding: 24px 16px; }
	.rp-empty-icon { font-size: calc(2rem * var(--rs-scale, 1)); margin-bottom: 8px; }
	.rp-empty-text { font-size: calc(0.82rem * var(--rs-scale, 1)); color: var(--text-muted); line-height: 1.4; }
	.rp-section { margin-bottom: 4px; }
	.rp-header {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 6px 12px; border: none; background: none; cursor: pointer;
		font-size: calc(0.78rem * var(--rs-scale, 1)); font-weight: 600; color: var(--text-normal); font-family: inherit;
		text-align: start;
	}
	.rp-header:hover { background: var(--background-modifier-hover); }
	.rp-chevron { font-size: calc(0.65rem * var(--rs-scale, 1)); transition: transform 0.15s; flex-shrink: 0; }
	.rp-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .rp-chevron.collapsed { transform: rotate(90deg); }
	.rp-count { margin-inline-start: auto; font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-faint); font-weight: 400; }
	.rp-item {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 12px 4px 20px; font-size: calc(0.78rem * var(--rs-scale, 1));
	}
	.rp-name {
		flex: 1; display: flex; align-items: center; gap: 4px;
		border: none; background: none; cursor: pointer; padding: 2px 4px;
		border-radius: 3px; font-family: inherit; font-size: calc(0.78rem * var(--rs-scale, 1));
		color: var(--text-normal); text-align: start; min-width: 0;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.rp-name:hover { background: var(--background-modifier-hover); }
	.rp-reason { font-size: calc(0.7rem * var(--rs-scale, 1)); flex-shrink: 0; }
	.rp-detail { font-size: calc(0.68rem * var(--rs-scale, 1)); color: var(--text-faint); white-space: nowrap; flex-shrink: 0; }
	.rp-actions { display: flex; gap: 2px; flex-shrink: 0; }
	.rp-action {
		width: 22px; height: 22px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; cursor: pointer;
		font-size: calc(0.7rem * var(--rs-scale, 1)); color: var(--text-muted);
	}
	.rp-action:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	/* MIG-080 — bounded scroller so VirtualList can window a huge section (the §C.2c pattern;
	   only used when a section exceeds VLIST_THRESHOLD). Normal lists render plain above this. */
	.rp-vlist-wrap { display: flex; flex-direction: column; max-height: 60vh; min-height: 0; }
</style>

<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		count = 0,
		onBatchTag,
		onBatchMove,
		onBatchDelete,
		onClearSelection,
	}: {
		count: number;
		onBatchTag?: () => void;
		onBatchMove?: () => void;
		onBatchDelete?: () => void;
		onClearSelection?: () => void;
	} = $props();
</script>

{#if count > 0}
	<div class="nav-batch-bar">
		<span class="nav-batch-count">{count} {$t('navigator.selected') || 'selected'}</span>
		<div class="nav-batch-actions">
			<button class="nav-batch-btn" onclick={onBatchTag} title={$t('navigator.batchTag') || 'Add tag'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 5 6.3 6.3a2.4 2.4 0 0 1 0 3.4L12 24"/><path d="M9.586 5.586A2 2 0 0 0 8.172 5H3a1 1 0 0 0-1 1v5.172a2 2 0 0 0 .586 1.414L8 18"/></svg>
				{$t('navigator.batchTag') || 'Tag'}
			</button>
			<button class="nav-batch-btn" onclick={onBatchMove} title={$t('navigator.batchMove') || 'Move'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
				{$t('navigator.batchMove') || 'Move'}
			</button>
			<button class="nav-batch-btn danger" onclick={onBatchDelete} title={$t('navigator.batchDelete') || 'Delete'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
				{$t('navigator.batchDelete') || 'Delete'}
			</button>
		</div>
		<button class="nav-batch-clear" onclick={onClearSelection}>✕</button>
	</div>
{/if}

<style>
	.nav-batch-bar {
		display: flex; align-items: center; gap: 8px;
		padding: 6px 10px; background: var(--background-secondary);
		border-top: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.nav-batch-count { font-size: 12px; color: var(--text-muted); white-space: nowrap; }
	.nav-batch-actions { display: flex; gap: 4px; flex: 1; }
	.nav-batch-btn {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 8px; border: none; border-radius: 4px;
		background: var(--background-modifier-hover); color: var(--text-normal);
		font-size: 11px; cursor: pointer;
	}
	.nav-batch-btn:hover { background: var(--interactive-accent); color: white; }
	.nav-batch-btn.danger:hover { background: #ef4444; }
	.nav-batch-clear {
		border: none; background: transparent; color: var(--text-muted);
		cursor: pointer; font-size: 14px; padding: 2px 6px;
	}
</style>

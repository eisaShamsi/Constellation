<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		x = 0,
		y = 0,
		onAddRow,
		onAddColumn,
		onDeleteRow,
		onDeleteColumn,
		onAlignLeft,
		onAlignCenter,
		onAlignRight,
	}: {
		x: number;
		y: number;
		onAddRow: () => void;
		onAddColumn: () => void;
		onDeleteRow: () => void;
		onDeleteColumn: () => void;
		onAlignLeft: () => void;
		onAlignCenter: () => void;
		onAlignRight: () => void;
	} = $props();

	function prevent(e: MouseEvent) { e.preventDefault(); }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="table-toolbar" style="left: {x}px; top: {y}px;" onmousedown={prevent}>
	<button class="tt-btn" title={$t('tableToolbar.addRow')} onmousedown={prevent} onclick={onAddRow}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
		<span class="tt-label">Row</span>
	</button>
	<button class="tt-btn" title={$t('tableToolbar.addColumn')} onmousedown={prevent} onclick={onAddColumn}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
		<span class="tt-label">Col</span>
	</button>
	<div class="tt-separator"></div>
	<button class="tt-btn danger" title={$t('tableToolbar.deleteRow')} onmousedown={prevent} onclick={onDeleteRow}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 12h14"/></svg>
		<span class="tt-label">Row</span>
	</button>
	<button class="tt-btn danger" title={$t('tableToolbar.deleteColumn')} onmousedown={prevent} onclick={onDeleteColumn}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 12h14"/></svg>
		<span class="tt-label">Col</span>
	</button>
	<div class="tt-separator"></div>
	<button class="tt-btn" title={$t('tableToolbar.alignLeft')} onmousedown={prevent} onclick={onAlignLeft}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M17 10H3M21 6H3M21 14H3M17 18H3"/></svg>
	</button>
	<button class="tt-btn" title={$t('tableToolbar.alignCenter')} onmousedown={prevent} onclick={onAlignCenter}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 10H6M21 6H3M21 14H3M18 18H6"/></svg>
	</button>
	<button class="tt-btn" title={$t('tableToolbar.alignRight')} onmousedown={prevent} onclick={onAlignRight}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 10H7M21 6H3M21 14H3M21 18H7"/></svg>
	</button>
</div>

<style>
	.table-toolbar {
		position: absolute;
		z-index: 100;
		display: flex;
		align-items: center;
		gap: 1px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.18);
		padding: 3px 4px;
		transform: translateX(-50%);
	}

	.tt-btn {
		display: flex;
		align-items: center;
		gap: 3px;
		height: 26px;
		border: none;
		border-radius: 5px;
		background: transparent;
		color: var(--text-normal);
		cursor: pointer;
		font-size: 0.72rem;
		padding: 0 5px;
	}
	.tt-btn:hover {
		background: var(--background-modifier-hover);
	}
	.tt-btn.danger:hover {
		background: color-mix(in srgb, var(--color-red) 15%, transparent);
		color: var(--color-red);
	}

	.tt-label {
		font-size: 0.68rem;
		opacity: 0.7;
	}

	.tt-separator {
		width: 1px;
		height: 18px;
		background: var(--background-modifier-border);
		margin: 0 2px;
	}
</style>

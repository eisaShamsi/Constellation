<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		x = 0,
		y = 0,
		dir = 'ltr' as 'ltr' | 'rtl',
		onAddRow,
		onAddColumn,
		onDeleteRow,
		onDeleteColumn,
		onAlignLeft,
		onAlignCenter,
		onAlignRight,
		onMoveRowUp,
		onMoveRowDown,
		onMoveColLeft,
		onMoveColRight,
		onSortAsc,
		onSortDesc,
		onInsertFormula,
		onEvaluateFormulas,
		canDeleteRow = true,
		canDeleteColumn = true,
	}: {
		x: number;
		y: number;
		dir?: 'ltr' | 'rtl';
		onAddRow: () => void;
		onAddColumn: () => void;
		onDeleteRow: () => void;
		onDeleteColumn: () => void;
		canDeleteRow?: boolean;
		canDeleteColumn?: boolean;
		onAlignLeft: () => void;
		onAlignCenter: () => void;
		onAlignRight: () => void;
		onMoveRowUp: () => void;
		onMoveRowDown: () => void;
		onMoveColLeft: () => void;
		onMoveColRight: () => void;
		onSortAsc: () => void;
		onSortDesc: () => void;
		onInsertFormula: () => void;
		onEvaluateFormulas: () => void;
	} = $props();

	let showMore = $state(false);

	function prevent(e: MouseEvent) { e.preventDefault(); }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="table-toolbar" {dir} style="left: {x}px; top: {y}px;" onmousedown={prevent}>
	<button class="tt-btn" title={$t('tableToolbar.addRow')} onmousedown={prevent} onclick={onAddRow}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
		<span class="tt-label">Row</span>
	</button>
	<button class="tt-btn" title={$t('tableToolbar.addColumn')} onmousedown={prevent} onclick={onAddColumn}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
		<span class="tt-label">Col</span>
	</button>
	<div class="tt-separator"></div>
	<button class="tt-btn danger" title={$t('tableToolbar.deleteRow')} onmousedown={prevent} onclick={onDeleteRow} disabled={!canDeleteRow}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 12h14"/></svg>
		<span class="tt-label">Row</span>
	</button>
	<button class="tt-btn danger" title={$t('tableToolbar.deleteColumn')} onmousedown={prevent} onclick={onDeleteColumn} disabled={!canDeleteColumn}>
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
	<div class="tt-separator"></div>
	<!-- More actions toggle -->
	<button class="tt-btn" title={$t('tableToolbar.moreActions')} onmousedown={prevent} onclick={() => showMore = !showMore}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
	</button>
</div>

{#if showMore}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="tt-more-menu" {dir} style="left: {x}px; top: {y + 36}px;" onmousedown={prevent}>
		<!-- Move -->
		<div class="tt-menu-group">
			<span class="tt-menu-label">{$t('tableToolbar.move')}</span>
			<div class="tt-menu-row">
				<button class="tt-btn" title={$t('tableToolbar.moveRowUp')} onmousedown={prevent} onclick={onMoveRowUp}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
					<span class="tt-label">Row</span>
				</button>
				<button class="tt-btn" title={$t('tableToolbar.moveRowDown')} onmousedown={prevent} onclick={onMoveRowDown}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12l7 7 7-7"/></svg>
					<span class="tt-label">Row</span>
				</button>
				<button class="tt-btn" title={$t('tableToolbar.moveColLeft')} onmousedown={prevent} onclick={onMoveColLeft}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M19 12H5M12 5l-7 7 7 7"/></svg>
					<span class="tt-label">Col</span>
				</button>
				<button class="tt-btn" title={$t('tableToolbar.moveColRight')} onmousedown={prevent} onclick={onMoveColRight}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
					<span class="tt-label">Col</span>
				</button>
			</div>
		</div>
		<div class="tt-menu-sep"></div>
		<!-- Sort -->
		<div class="tt-menu-group">
			<span class="tt-menu-label">{$t('tableToolbar.sort')}</span>
			<div class="tt-menu-row">
				<button class="tt-btn" title={$t('tableToolbar.sortAsc')} onmousedown={prevent} onclick={() => { onSortAsc(); showMore = false; }}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 20V4M5 11l7-7 7 7"/></svg>
					<span class="tt-label">A→Z</span>
				</button>
				<button class="tt-btn" title={$t('tableToolbar.sortDesc')} onmousedown={prevent} onclick={() => { onSortDesc(); showMore = false; }}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 4v16M5 13l7 7 7-7"/></svg>
					<span class="tt-label">Z→A</span>
				</button>
			</div>
		</div>
		<div class="tt-menu-sep"></div>
		<!-- Formulas -->
		<div class="tt-menu-group">
			<span class="tt-menu-label">{$t('tableToolbar.formulas')}</span>
			<div class="tt-menu-row">
				<button class="tt-btn" title={$t('tableToolbar.insertFormula')} onmousedown={prevent} onclick={() => { onInsertFormula(); showMore = false; }}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><text x="4" y="18" font-size="16" fill="currentColor" stroke="none" font-style="italic">f</text><text x="12" y="12" font-size="10" fill="currentColor" stroke="none">(x)</text></svg>
					<span class="tt-label">=SUM</span>
				</button>
				<button class="tt-btn" title={$t('tableToolbar.evaluateFormulas')} onmousedown={prevent} onclick={() => { onEvaluateFormulas(); showMore = false; }}>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 12h14"/><path d="M12 5l7 7-7 7"/></svg>
					<span class="tt-label">{$t('tableToolbar.evaluate')}</span>
				</button>
			</div>
		</div>
	</div>
{/if}

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
	.tt-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}
	.tt-btn:disabled:hover {
		background: transparent;
		color: var(--text-normal);
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

	.tt-more-menu {
		position: absolute;
		z-index: 101;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.18);
		padding: 6px 8px;
		transform: translateX(-50%);
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 180px;
	}

	.tt-menu-group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.tt-menu-label {
		font-size: 0.65rem;
		font-weight: 600;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		padding: 0 4px;
	}

	.tt-menu-row {
		display: flex;
		gap: 1px;
	}

	.tt-menu-sep {
		height: 1px;
		background: var(--background-modifier-border);
		margin: 2px 0;
	}

	/* RTL: flip move col arrows and alignment icons */
	.table-toolbar[dir="rtl"] .tt-btn svg { transform: scaleX(-1); }
	.tt-more-menu[dir="rtl"] .tt-btn svg { transform: scaleX(-1); }
</style>

<script lang="ts">
	/**
	 * MIG-065 §G.2b — the multi-sort panel for the unified Base.
	 *
	 * The "Strong" layer on top of click-header sorting (§G.2a): sort by several
	 * columns at once (e.g. `library`, then `name`). Lists the active `order:`
	 * entries with per-entry direction toggle, priority reorder (↑/↓), and
	 * remove (×); plus a list of sortable columns to add. Every change calls
	 * `onChange(order)` — the host persists via `update_base_order` and the
	 * panel re-renders from the fresh `order` prop. The panel stays open so the
	 * user can build a multi-level sort in one place.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { columnLabel, isSortable } from '$lib/lens/tableModel';
	import type { LensSort } from '$lib/lens/store';

	let {
		order,
		columns,
		onChange,
		onClose,
	}: {
		/** Current sort clauses (priority = array order). */
		order: LensSort[];
		/** The base's declared columns — the pool of sort candidates. */
		columns: string[];
		onChange: (order: LensSort[]) => void;
		onClose: () => void;
	} = $props();

	let panelEl = $state<HTMLDivElement | null>(null);

	// Columns that can be added as a sort key: note.name (always) + the base's
	// sortable columns, minus those already in the sort.
	const addable = $derived.by(() => {
		const inSort = new Set(order.map((o) => o.dimension));
		const pool = ['note.name', ...columns];
		const seen = new Set<string>();
		return pool.filter((d) => {
			if (seen.has(d) || inSort.has(d) || !isSortable(d)) return false;
			seen.add(d);
			return true;
		});
	});

	function toggleDir(i: number) {
		onChange(
			order.map((o, j) =>
				j === i ? { ...o, direction: o.direction === 'asc' ? 'desc' : 'asc' } : o,
			),
		);
	}
	function move(i: number, delta: number) {
		const j = i + delta;
		if (j < 0 || j >= order.length) return;
		const next = [...order];
		[next[i], next[j]] = [next[j], next[i]];
		onChange(next);
	}
	function removeAt(i: number) {
		onChange(order.filter((_, j) => j !== i));
	}
	function add(dim: string) {
		onChange([...order, { dimension: dim, direction: 'asc' }]);
	}

	function onWindowMouseDown(e: MouseEvent) {
		if (panelEl && e.target instanceof Node && !panelEl.contains(e.target)) onClose();
	}
	function onKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.stopPropagation();
			onClose();
		}
	}
	onMount(() => {
		const id = setTimeout(() => window.addEventListener('mousedown', onWindowMouseDown), 0);
		window.addEventListener('keydown', onKeyDown, true);
		return () => clearTimeout(id);
	});
	onDestroy(() => {
		window.removeEventListener('mousedown', onWindowMouseDown);
		window.removeEventListener('keydown', onKeyDown, true);
	});
</script>

<div class="sort-panel" bind:this={panelEl} role="dialog" aria-label={$t('lensBlock.sortTitle') || 'Sort'}>
	{#if order.length === 0}
		<div class="sp-state">{$t('lensBlock.notSorted') || 'Not sorted yet.'}</div>
	{:else}
		{#each order as o, i (o.dimension)}
			<div class="sp-row">
				<span class="sp-prio">{i + 1}</span>
				<span class="sp-label" dir="auto">{columnLabel(o.dimension, $t)}</span>
				<button
					class="sp-dir"
					title={$t('lensBlock.toggleDirection') || 'Toggle direction'}
					onclick={() => toggleDir(i)}
				>
					{o.direction === 'asc' ? '↑ A–Z' : '↓ Z–A'}
				</button>
				<button class="sp-icon" disabled={i === 0} title={$t('lensBlock.moveUp') || 'Move up'} onclick={() => move(i, -1)}>↑</button>
				<button class="sp-icon" disabled={i === order.length - 1} title={$t('lensBlock.moveDown') || 'Move down'} onclick={() => move(i, 1)}>↓</button>
				<button class="sp-icon sp-remove" title={$t('lensBlock.removeSort') || 'Remove'} onclick={() => removeAt(i)}>×</button>
			</div>
		{/each}
	{/if}

	<div class="sp-divider"></div>
	<div class="sp-add-head">{$t('lensBlock.addSort') || 'Add a column to sort by'}</div>
	{#if addable.length === 0}
		<div class="sp-state">{$t('lensBlock.allSorted') || 'Every sortable column is already in the sort.'}</div>
	{:else}
		{#each addable as dim (dim)}
			<button class="sp-add-item" onclick={() => add(dim)}>
				<span dir="auto">{columnLabel(dim, $t)}</span>
			</button>
		{/each}
	{/if}
</div>

<style>
	.sort-panel {
		position: absolute;
		top: calc(100% + 4px);
		inset-inline-end: 0;
		z-index: 50;
		width: 300px;
		max-width: 86vw;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 8px 28px rgba(0, 0, 0, 0.28);
		padding: 8px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 360px;
		overflow-y: auto;
	}
	.sp-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 3px 4px;
		border-radius: 5px;
	}
	.sp-row:hover {
		background: var(--background-modifier-hover);
	}
	.sp-prio {
		flex-shrink: 0;
		width: 1.4em;
		height: 1.4em;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: 999px;
		background: var(--interactive-accent);
		color: #fff;
		font-size: 0.7rem;
		font-weight: 700;
	}
	.sp-label {
		flex: 1 1 auto;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 0.86rem;
		color: var(--text-normal);
	}
	.sp-dir {
		flex-shrink: 0;
		background: var(--background-modifier-hover);
		border: none;
		border-radius: 5px;
		padding: 2px 7px;
		font: inherit;
		font-size: 0.74rem;
		color: var(--text-muted);
		cursor: pointer;
		white-space: nowrap;
	}
	.sp-dir:hover {
		color: var(--text-normal);
	}
	.sp-icon {
		flex-shrink: 0;
		background: none;
		border: none;
		padding: 0 3px;
		font: inherit;
		font-size: 0.95rem;
		line-height: 1;
		color: var(--text-faint);
		cursor: pointer;
	}
	.sp-icon:hover:not(:disabled) {
		color: var(--text-normal);
	}
	.sp-icon:disabled {
		opacity: 0.3;
		cursor: default;
	}
	.sp-remove:hover:not(:disabled) {
		color: var(--text-error, #e53e3e);
	}
	.sp-divider {
		height: 1px;
		background: var(--background-modifier-border);
		margin: 5px 0;
	}
	.sp-add-head {
		font-size: 0.7rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-muted);
		padding: 2px 4px 4px;
	}
	.sp-add-item {
		display: block;
		width: 100%;
		text-align: start;
		background: none;
		border: none;
		border-radius: 5px;
		padding: 5px 7px;
		font: inherit;
		font-size: 0.86rem;
		color: var(--text-normal);
		cursor: pointer;
	}
	.sp-add-item:hover {
		background: var(--background-modifier-hover);
	}
	.sp-state {
		color: var(--text-muted);
		font-size: 0.82rem;
		padding: 6px 4px;
	}
</style>

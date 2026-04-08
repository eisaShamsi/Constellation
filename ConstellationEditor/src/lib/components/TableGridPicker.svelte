<script lang="ts">
	import { t } from '$lib/i18n';

	let {
		x = 0,
		y = 0,
		onInsert,
		onClose,
	}: {
		x: number;
		y: number;
		onInsert: (rows: number, cols: number) => void;
		onClose: () => void;
	} = $props();

	const MAX_ROWS = 8;
	const MAX_COLS = 8;

	let hoverRow = $state(0);
	let hoverCol = $state(0);

	function handleCellHover(r: number, c: number) {
		hoverRow = r;
		hoverCol = c;
	}

	function handleCellClick(r: number, c: number) {
		onInsert(r + 1, c + 1);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	function prevent(e: MouseEvent) { e.preventDefault(); }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="grid-picker" style="left: {x}px; top: {y}px;" onmousedown={prevent}>
	<div class="grid-label">{hoverRow + 1} × {hoverCol + 1} {$t('toolbar.table')}</div>
	<div class="grid" role="grid">
		{#each Array(MAX_ROWS) as _, r}
			<div class="grid-row" role="row">
				{#each Array(MAX_COLS) as _, c}
					<button
						class="grid-cell"
						class:active={r <= hoverRow && c <= hoverCol}
						role="gridcell"
						aria-label="{r + 1} × {c + 1}"
						onmouseenter={() => handleCellHover(r, c)}
						onclick={() => handleCellClick(r, c)}
						onmousedown={prevent}
					></button>
				{/each}
			</div>
		{/each}
	</div>
</div>

<style>
	.grid-picker {
		position: absolute;
		z-index: 120;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.18);
		padding: 8px;
		transform: translateX(-50%);
	}

	.grid-label {
		text-align: center;
		font-size: 0.75rem;
		color: var(--text-muted);
		margin-bottom: 6px;
		font-weight: 500;
		user-select: none;
	}

	.grid {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.grid-row {
		display: flex;
		gap: 2px;
	}

	.grid-cell {
		width: 18px;
		height: 18px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 3px;
		background: var(--background-secondary);
		cursor: pointer;
		padding: 0;
		transition: background 0.05s, border-color 0.05s;
	}

	.grid-cell.active {
		background: var(--interactive-accent, var(--color-accent));
		border-color: var(--interactive-accent, var(--color-accent));
		opacity: 0.7;
	}

	.grid-cell:hover {
		opacity: 1;
	}
</style>

<script lang="ts">
	import type { SortRule } from '$lib/bases/types';

	let {
		sorts,
		availableProperties,
		onchange,
	}: {
		sorts: SortRule[];
		availableProperties: string[];
		onchange: (sorts: SortRule[]) => void;
	} = $props();

	function addSort() {
		const property = availableProperties[0] ?? 'file_name';
		const updated = [...sorts, { property, direction: 'asc' as const }];
		onchange(updated);
	}

	function removeSort(idx: number) {
		const updated = sorts.filter((_, i) => i !== idx);
		onchange(updated);
	}

	function updateSort(idx: number, field: keyof SortRule, value: string) {
		const updated = sorts.map((s, i) => {
			if (i !== idx) return s;
			return { ...s, [field]: value };
		});
		onchange(updated);
	}

	function moveSort(idx: number, dir: -1 | 1) {
		const newIdx = idx + dir;
		if (newIdx < 0 || newIdx >= sorts.length) return;
		const updated = [...sorts];
		[updated[idx], updated[newIdx]] = [updated[newIdx], updated[idx]];
		onchange(updated);
	}
</script>

<div class="sort-builder">
	{#each sorts as sort, idx (idx)}
		<div class="sort-row">
			<div class="sort-order-btns">
				<button class="sort-move" disabled={idx === 0} onclick={() => moveSort(idx, -1)} title="Move up">
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="18 15 12 9 6 15"/></svg>
				</button>
				<button class="sort-move" disabled={idx === sorts.length - 1} onclick={() => moveSort(idx, 1)} title="Move down">
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>
				</button>
			</div>

			<select
				class="sort-select"
				value={sort.property}
				onchange={(e) => updateSort(idx, 'property', e.currentTarget.value)}
			>
				{#each availableProperties as prop}
					<option value={prop}>{prop}</option>
				{/each}
			</select>

			<select
				class="sort-select sort-dir"
				value={sort.direction}
				onchange={(e) => updateSort(idx, 'direction', e.currentTarget.value)}
			>
				<option value="asc">Ascending ↑</option>
				<option value="desc">Descending ↓</option>
			</select>

			<button class="sort-remove" onclick={() => removeSort(idx)} title="Remove sort">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
				</svg>
			</button>
		</div>
	{/each}

	<button class="sort-add" onclick={addSort}>
		<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
			<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
		</svg>
		Add sort
	</button>
</div>

<style>
	.sort-builder {
		padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
		display: flex;
		flex-direction: column;
		gap: 6px;
		background: var(--background-secondary-alt, var(--background-secondary));
	}

	.sort-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.sort-order-btns {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.sort-move {
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 1px;
		display: flex;
		align-items: center;
		border-radius: 3px;
	}
	.sort-move:hover:not(:disabled) {
		color: var(--text-normal);
		background: var(--background-modifier-hover);
	}
	.sort-move:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.sort-select {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		padding: 4px 8px;
		font-size: 0.78rem;
		color: var(--text-normal);
		cursor: pointer;
		min-width: 100px;
	}
	.sort-dir {
		min-width: 130px;
	}

	.sort-remove {
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 4px;
		border-radius: 4px;
		display: flex;
		align-items: center;
	}
	.sort-remove:hover {
		color: var(--text-error, #e53e3e);
		background: var(--background-modifier-hover);
	}

	.sort-add {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: none;
		border: 1px dashed var(--background-modifier-border);
		border-radius: 4px;
		padding: 4px 10px;
		font-size: 0.76rem;
		color: var(--text-muted);
		cursor: pointer;
		align-self: flex-start;
	}
	.sort-add:hover {
		color: var(--text-normal);
		border-color: var(--text-muted);
	}
</style>

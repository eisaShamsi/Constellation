<script lang="ts">
	import type { FilterRule } from '$lib/bases/types';

	let {
		filters,
		availableProperties,
		onchange,
	}: {
		filters: FilterRule[];
		availableProperties: string[];
		onchange: (filters: FilterRule[]) => void;
	} = $props();

	const OPERATORS: { value: FilterRule['operator']; label: string }[] = [
		{ value: 'is', label: 'is' },
		{ value: 'is_not', label: 'is not' },
		{ value: 'contains', label: 'contains' },
		{ value: 'not_contains', label: 'does not contain' },
		{ value: 'gt', label: 'greater than' },
		{ value: 'lt', label: 'less than' },
		{ value: 'is_empty', label: 'is empty' },
		{ value: 'is_not_empty', label: 'is not empty' },
	];

	const needsValue = (op: string) => !['is_empty', 'is_not_empty'].includes(op);

	function addFilter() {
		const property = availableProperties[0] ?? 'file_name';
		const updated = [...filters, { property, operator: 'contains' as const, value: '' }];
		onchange(updated);
	}

	function removeFilter(idx: number) {
		const updated = filters.filter((_, i) => i !== idx);
		onchange(updated);
	}

	function updateFilter(idx: number, field: keyof FilterRule, value: string) {
		const updated = filters.map((f, i) => {
			if (i !== idx) return f;
			return { ...f, [field]: value };
		});
		onchange(updated);
	}
</script>

<div class="filter-builder">
	{#each filters as filter, idx (idx)}
		<div class="filter-row">
			<select
				class="filter-select"
				value={filter.property}
				onchange={(e) => updateFilter(idx, 'property', e.currentTarget.value)}
			>
				{#each availableProperties as prop}
					<option value={prop}>{prop}</option>
				{/each}
			</select>

			<select
				class="filter-select filter-op"
				value={filter.operator}
				onchange={(e) => updateFilter(idx, 'operator', e.currentTarget.value)}
			>
				{#each OPERATORS as op}
					<option value={op.value}>{op.label}</option>
				{/each}
			</select>

			{#if needsValue(filter.operator)}
				<input
					class="filter-input"
					type="text"
					value={filter.value ?? ''}
					oninput={(e) => updateFilter(idx, 'value', e.currentTarget.value)}
					placeholder="Value..."
				/>
			{/if}

			<button class="filter-remove" onclick={() => removeFilter(idx)} title="Remove filter">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
				</svg>
			</button>
		</div>
	{/each}

	<button class="filter-add" onclick={addFilter}>
		<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
			<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
		</svg>
		Add filter
	</button>
</div>

<style>
	.filter-builder {
		padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
		display: flex;
		flex-direction: column;
		gap: 6px;
		background: var(--background-secondary-alt, var(--background-secondary));
	}

	.filter-row {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}

	.filter-select {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		padding: 4px 8px;
		font-size: 0.78rem;
		color: var(--text-normal);
		cursor: pointer;
		min-width: 100px;
	}
	.filter-op {
		min-width: 130px;
	}

	.filter-input {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		padding: 4px 8px;
		font-size: 0.78rem;
		color: var(--text-normal);
		flex: 1;
		min-width: 100px;
	}
	.filter-input:focus {
		border-color: var(--interactive-accent);
		outline: none;
	}

	.filter-remove {
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 4px;
		border-radius: 4px;
		display: flex;
		align-items: center;
	}
	.filter-remove:hover {
		color: var(--text-error, #e53e3e);
		background: var(--background-modifier-hover);
	}

	.filter-add {
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
	.filter-add:hover {
		color: var(--text-normal);
		border-color: var(--text-muted);
	}
</style>

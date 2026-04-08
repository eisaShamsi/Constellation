<script lang="ts">
	import { detectCellType, type BaseRow, type ColumnDef } from '$lib/bases/types';
	import { detectDir } from '$lib/utils';

	let {
		rows,
		columns,
		dir = 'ltr',
		onOpenNote,
	}: {
		rows: BaseRow[];
		columns: ColumnDef[];
		dir: 'ltr' | 'rtl';
		onOpenNote: (path: string, libraryName: string) => void;
	} = $props();

	const visibleColumns = $derived(columns.filter(c => c.visible !== false).slice(0, 6));

	function getCellDir(value: string): 'ltr' | 'rtl' {
		return detectDir(value) as 'ltr' | 'rtl';
	}
</script>

<div class="base-card-grid" dir={dir}>
	{#each rows as row (row.file_path)}
		<button class="base-card" onclick={() => onOpenNote(row.file_path, row.library_name)}>
			<div class="card-header">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
					<polyline points="14 2 14 8 20 8"/>
				</svg>
				<span class="card-title" dir={getCellDir(row.file_name)}>{row.file_name}</span>
			</div>
			<div class="card-props">
				{#each visibleColumns as col (col.property)}
					{@const value = row.properties[col.property] ?? ''}
					{#if value}
						{@const cellType = detectCellType(col.property, value)}
						<div class="card-prop">
							<span class="card-prop-key">{col.label || col.property}</span>
							{#if cellType === 'checkbox'}
								<span class="card-prop-value">
									{#if value === 'true'}
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--interactive-accent)" stroke-width="2.5">
											<rect x="3" y="3" width="18" height="18" rx="3" fill="var(--interactive-accent)" stroke="none"/>
											<polyline points="7 13 10 16 17 9" stroke="white" stroke-width="2.5"/>
										</svg>
									{:else}
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="2">
											<rect x="3" y="3" width="18" height="18" rx="3"/>
										</svg>
									{/if}
								</span>
							{:else if cellType === 'list'}
								<div class="card-tags">
									{#each value.split(',').map(s => s.trim()).filter(Boolean) as tag}
										<span class="card-tag">{tag}</span>
									{/each}
								</div>
							{:else}
								<span class="card-prop-value" dir={getCellDir(value)}>
									{cellType === 'link' ? value.replace(/^\[\[|\]\]$/g, '') : value}
								</span>
							{/if}
						</div>
					{/if}
				{/each}
			</div>
			{#if row.library_name}
				<div class="card-library">{row.library_name}</div>
			{/if}
		</button>
	{/each}
</div>

<style>
	.base-card-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 10px;
		padding: 12px 16px;
	}

	.base-card {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		padding: 12px;
		cursor: pointer;
		text-align: start;
		transition: border-color 0.15s, box-shadow 0.15s;
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-family: inherit;
		font-size: inherit;
		color: inherit;
	}
	.base-card:hover {
		border-color: var(--interactive-accent);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.card-header svg {
		flex-shrink: 0;
		color: var(--text-faint);
	}

	.card-title {
		font-weight: 600;
		font-size: 0.88rem;
		color: var(--text-normal);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.card-props {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.card-prop {
		display: flex;
		align-items: baseline;
		gap: 6px;
		font-size: 0.78rem;
		min-width: 0;
	}

	.card-prop-key {
		color: var(--text-faint);
		flex-shrink: 0;
		max-width: 80px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.card-prop-value {
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: inline-flex;
		align-items: center;
	}

	.card-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
	}
	.card-tag {
		background: var(--background-modifier-hover);
		color: var(--text-muted);
		padding: 0 6px;
		border-radius: 8px;
		font-size: 0.72rem;
		white-space: nowrap;
	}

	.card-library {
		font-size: 0.7rem;
		color: var(--text-faint);
		margin-top: auto;
		padding-top: 4px;
		border-top: 1px solid var(--background-modifier-border);
	}
</style>

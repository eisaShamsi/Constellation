<script lang="ts">
	import { detectCellType, type BaseRow, type ColumnDef } from '$lib/bases/types';
	import { detectDir } from '$lib/utils';

	let {
		rows,
		columns,
		dir = 'ltr',
		onCellEdit,
		onOpenNote,
	}: {
		rows: BaseRow[];
		columns: ColumnDef[];
		dir: 'ltr' | 'rtl';
		onCellEdit: (row: BaseRow, key: string, value: string) => void;
		onOpenNote: (path: string, vaultName: string) => void;
	} = $props();

	const visibleColumns = $derived(columns.filter(c => c.visible !== false).slice(0, 4));

	let editingCell: { rowIdx: number; colKey: string } | null = $state(null);
	let editValue = $state('');

	function startEdit(rowIdx: number, colKey: string, currentValue: string) {
		editingCell = { rowIdx, colKey };
		editValue = currentValue;
	}

	function commitEdit(row: BaseRow) {
		if (!editingCell) return;
		const key = editingCell.colKey;
		if (editValue !== (row.properties[key] ?? '')) {
			onCellEdit(row, key, editValue);
		}
		editingCell = null;
	}

	function cancelEdit() {
		editingCell = null;
	}

	function getCellDir(value: string): 'ltr' | 'rtl' {
		return detectDir(value) as 'ltr' | 'rtl';
	}
</script>

<div class="base-list" dir={dir}>
	{#each rows as row, rowIdx (row.file_path)}
		<div class="list-item">
			<div class="list-item-main">
				<button class="list-file-link" onclick={() => onOpenNote(row.file_path, row.vault_name)}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
						<polyline points="14 2 14 8 20 8"/>
					</svg>
					<span dir={getCellDir(row.file_name)}>{row.file_name}</span>
				</button>
				{#if row.vault_name}
					<span class="list-vault">{row.vault_name}</span>
				{/if}
			</div>
			<div class="list-item-props">
				{#each visibleColumns as col (col.property)}
					{@const value = row.properties[col.property] ?? ''}
					{@const cellType = detectCellType(col.property, value)}
					<div class="list-prop">
						<span class="list-prop-key">{col.label || col.property}:</span>
						{#if editingCell?.rowIdx === rowIdx && editingCell?.colKey === col.property}
							<input
								class="list-prop-input"
								type="text"
								bind:value={editValue}
								onblur={() => commitEdit(row)}
								onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); commitEdit(row); } else if (e.key === 'Escape') cancelEdit(); }}
								autofocus
							/>
						{:else if cellType === 'checkbox'}
							<button class="list-checkbox" onclick={() => onCellEdit(row, col.property, value === 'true' ? 'false' : 'true')}>
								{#if value === 'true'}
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none">
										<rect x="3" y="3" width="18" height="18" rx="3" fill="var(--interactive-accent)"/>
										<polyline points="7 13 10 16 17 9" stroke="white" stroke-width="2.5"/>
									</svg>
								{:else}
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="2">
										<rect x="3" y="3" width="18" height="18" rx="3"/>
									</svg>
								{/if}
							</button>
						{:else if cellType === 'list'}
							<span class="list-prop-tags" ondblclick={() => startEdit(rowIdx, col.property, value)}>
								{#each value.split(',').map(s => s.trim()).filter(Boolean) as tag}
									<span class="list-tag">{tag}</span>
								{/each}
							</span>
						{:else}
							<span
								class="list-prop-value"
								dir={getCellDir(value)}
								ondblclick={() => startEdit(rowIdx, col.property, value)}
							>
								{cellType === 'link' ? value.replace(/^\[\[|\]\]$/g, '') : value || '—'}
							</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/each}
</div>

<style>
	.base-list {
		display: flex;
		flex-direction: column;
		padding: 4px 0;
	}

	.list-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 7px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
		transition: background 0.1s;
	}
	.list-item:hover {
		background: var(--background-modifier-hover);
	}

	.list-item-main {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		flex: 1;
	}

	.list-file-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--text-normal);
		cursor: pointer;
		padding: 0;
		font-size: 0.84rem;
		text-align: start;
		min-width: 0;
	}
	.list-file-link span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.list-file-link:hover {
		color: var(--interactive-accent);
	}
	.list-file-link svg {
		flex-shrink: 0;
		color: var(--text-faint);
	}

	.list-vault {
		font-size: 0.7rem;
		color: var(--text-faint);
		background: var(--background-secondary);
		padding: 1px 6px;
		border-radius: 4px;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.list-item-props {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-shrink: 0;
	}

	.list-prop {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.78rem;
	}

	.list-prop-key {
		color: var(--text-faint);
		white-space: nowrap;
	}

	.list-prop-value {
		color: var(--text-muted);
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		cursor: text;
		padding: 1px 4px;
		border-radius: 3px;
	}
	.list-prop-value:hover {
		background: var(--background-modifier-hover);
	}

	.list-checkbox {
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		display: inline-flex;
	}

	.list-prop-tags {
		display: flex;
		gap: 3px;
		cursor: text;
	}
	.list-tag {
		background: var(--background-modifier-hover);
		color: var(--text-muted);
		padding: 0 6px;
		border-radius: 8px;
		font-size: 0.72rem;
		white-space: nowrap;
	}

	.list-prop-input {
		width: 100px;
		background: var(--background-primary);
		border: 1.5px solid var(--interactive-accent);
		border-radius: 3px;
		padding: 1px 4px;
		font-size: inherit;
		font-family: inherit;
		color: var(--text-normal);
		outline: none;
	}
</style>

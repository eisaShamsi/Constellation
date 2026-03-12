<script lang="ts">
	import { detectCellType, type BaseRow, type ColumnDef, type PropertyType } from '$lib/bases/types';
	import { detectDir } from '$lib/utils';

	let {
		rows,
		columns,
		dir = 'ltr',
		onCellEdit,
		onOpenNote,
		onColumnReorder,
	}: {
		rows: BaseRow[];
		columns: ColumnDef[];
		dir: 'ltr' | 'rtl';
		onCellEdit: (row: BaseRow, key: string, value: string) => void;
		onOpenNote: (path: string, vaultName: string) => void;
		onColumnReorder?: (columns: ColumnDef[]) => void;
	} = $props();

	// ─── Editing state ───
	let editingCell: { rowIdx: number; colKey: string } | null = $state(null);
	let editValue = $state('');

	// ─── Column resize state ───
	let resizingCol: string | null = $state(null);
	let resizeStartX = 0;
	let resizeStartWidth = 0;

	// ─── Column drag state ───
	let draggingCol: string | null = $state(null);
	let dragOverCol: string | null = $state(null);

	// Visible columns only
	const visibleColumns = $derived(columns.filter(c => c.visible !== false));

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

	function handleKeydown(e: KeyboardEvent, row: BaseRow) {
		if (e.key === 'Enter') {
			e.preventDefault();
			commitEdit(row);
		} else if (e.key === 'Escape') {
			cancelEdit();
		}
	}

	function handleCheckboxToggle(row: BaseRow, key: string, currentValue: string) {
		const newVal = currentValue === 'true' ? 'false' : 'true';
		onCellEdit(row, key, newVal);
	}

	// ─── Column resize ───
	function startResize(e: MouseEvent, col: ColumnDef) {
		e.preventDefault();
		e.stopPropagation();
		resizingCol = col.property;
		resizeStartX = e.clientX;
		resizeStartWidth = col.width ?? 150;
		window.addEventListener('mousemove', onResizeMove);
		window.addEventListener('mouseup', onResizeEnd);
	}

	function onResizeMove(e: MouseEvent) {
		if (!resizingCol) return;
		const diff = dir === 'rtl' ? resizeStartX - e.clientX : e.clientX - resizeStartX;
		const col = columns.find(c => c.property === resizingCol);
		if (col) {
			col.width = Math.max(60, resizeStartWidth + diff);
		}
	}

	function onResizeEnd() {
		resizingCol = null;
		window.removeEventListener('mousemove', onResizeMove);
		window.removeEventListener('mouseup', onResizeEnd);
		if (onColumnReorder) onColumnReorder(columns);
	}

	// ─── Column drag reorder ───
	function onColDragStart(e: DragEvent, col: ColumnDef) {
		draggingCol = col.property;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setData('text/plain', col.property);
		}
	}

	function onColDragOver(e: DragEvent, col: ColumnDef) {
		e.preventDefault();
		dragOverCol = col.property;
	}

	function onColDrop(e: DragEvent, targetCol: ColumnDef) {
		e.preventDefault();
		if (!draggingCol || draggingCol === targetCol.property) {
			draggingCol = null;
			dragOverCol = null;
			return;
		}
		const fromIdx = columns.findIndex(c => c.property === draggingCol);
		const toIdx = columns.findIndex(c => c.property === targetCol.property);
		if (fromIdx >= 0 && toIdx >= 0) {
			const [moved] = columns.splice(fromIdx, 1);
			columns.splice(toIdx, 0, moved);
			if (onColumnReorder) onColumnReorder([...columns]);
		}
		draggingCol = null;
		dragOverCol = null;
	}

	function renderCellValue(key: string, value: string, type: PropertyType): string {
		if (!value) return '';
		if (type === 'checkbox') return value === 'true' ? '✓' : '✗';
		if (type === 'link') return value.replace(/^\[\[|\]\]$/g, '');
		return value;
	}

	function getCellDir(value: string): 'ltr' | 'rtl' {
		return detectDir(value) as 'ltr' | 'rtl';
	}
</script>

<div class="base-table-wrapper" dir={dir}>
	<table class="base-table">
		<thead>
			<tr>
				<!-- File name column (always first) -->
				<th class="col-filename sticky-col" style="min-width: 180px">
					<span class="th-label">Name</span>
				</th>
				{#each visibleColumns as col (col.property)}
					<th
						class="col-header"
						class:drag-over={dragOverCol === col.property}
						style="width: {col.width ?? 150}px; min-width: {col.width ?? 150}px"
						draggable="true"
						ondragstart={(e) => onColDragStart(e, col)}
						ondragover={(e) => onColDragOver(e, col)}
						ondrop={(e) => onColDrop(e, col)}
						ondragend={() => { draggingCol = null; dragOverCol = null; }}
					>
						<span class="th-label">{col.label || col.property}</span>
						<!-- Resize handle -->
						<div
							class="resize-handle"
							onmousedown={(e) => startResize(e, col)}
							role="separator"
							aria-orientation="vertical"
						></div>
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row, rowIdx (row.file_path)}
				<tr class="base-row">
					<!-- File name cell -->
					<td class="cell-filename sticky-col">
						<button class="file-link" onclick={() => onOpenNote(row.file_path, row.vault_name)}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
								<polyline points="14 2 14 8 20 8"/>
							</svg>
							<span dir={getCellDir(row.file_name)}>{row.file_name}</span>
						</button>
					</td>
					<!-- Property cells -->
					{#each visibleColumns as col (col.property)}
						{@const value = row.properties[col.property] ?? ''}
						{@const cellType = detectCellType(col.property, value)}
						<td
							class="cell"
							class:cell-number={cellType === 'number'}
							class:cell-checkbox={cellType === 'checkbox'}
							style="width: {col.width ?? 150}px; min-width: {col.width ?? 150}px"
						>
							{#if editingCell?.rowIdx === rowIdx && editingCell?.colKey === col.property}
								<!-- Editing -->
								<input
									class="cell-input"
									type={cellType === 'number' ? 'number' : cellType === 'date' ? 'date' : 'text'}
									bind:value={editValue}
									onblur={() => commitEdit(row)}
									onkeydown={(e) => handleKeydown(e, row)}
									autofocus
								/>
							{:else if cellType === 'checkbox'}
								<button class="cell-checkbox-btn" onclick={() => handleCheckboxToggle(row, col.property, value)}>
									{#if value === 'true'}
										<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--interactive-accent)" stroke-width="2.5">
											<rect x="3" y="3" width="18" height="18" rx="3" fill="var(--interactive-accent)" stroke="none"/>
											<polyline points="7 13 10 16 17 9" stroke="white" stroke-width="2.5"/>
										</svg>
									{:else}
										<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="2">
											<rect x="3" y="3" width="18" height="18" rx="3"/>
										</svg>
									{/if}
								</button>
							{:else if cellType === 'link'}
								<button class="cell-link" onclick={() => {
									const target = value.replace(/^\[\[|\]\]$/g, '');
									onOpenNote(target, row.vault_name);
								}}>
									{value.replace(/^\[\[|\]\]$/g, '')}
								</button>
							{:else if cellType === 'list'}
								<div class="cell-tags" ondblclick={() => startEdit(rowIdx, col.property, value)}>
									{#each value.split(',').map(s => s.trim()).filter(Boolean) as tag}
										<span class="cell-tag">{tag}</span>
									{/each}
								</div>
							{:else}
								<!-- Text, number, date -->
								<span
									class="cell-text"
									dir={getCellDir(value)}
									ondblclick={() => startEdit(rowIdx, col.property, value)}
									role="button"
									tabindex="0"
									onkeydown={(e) => { if (e.key === 'Enter') startEdit(rowIdx, col.property, value); }}
								>
									{renderCellValue(col.property, value, cellType)}
								</span>
							{/if}
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.base-table-wrapper {
		overflow: auto;
		height: 100%;
	}

	.base-table {
		width: max-content;
		min-width: 100%;
		border-collapse: collapse;
		font-size: 0.84rem;
	}

	thead {
		position: sticky;
		top: 0;
		z-index: 2;
	}

	th {
		background: var(--background-secondary);
		border-bottom: 2px solid var(--background-modifier-border);
		padding: 6px 10px;
		text-align: start;
		font-weight: 600;
		font-size: 0.78rem;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.03em;
		white-space: nowrap;
		position: relative;
		user-select: none;
		cursor: grab;
	}
	th.drag-over {
		background: var(--background-modifier-hover);
		box-shadow: inset 2px 0 0 var(--interactive-accent);
	}

	.th-label {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.col-filename {
		cursor: default;
	}

	.sticky-col {
		position: sticky;
		left: 0;
		z-index: 1;
		background: var(--background-primary);
	}
	[dir="rtl"] .sticky-col {
		left: auto;
		right: 0;
	}
	thead .sticky-col {
		z-index: 3;
		background: var(--background-secondary);
	}

	.resize-handle {
		position: absolute;
		top: 0;
		right: 0;
		width: 4px;
		height: 100%;
		cursor: col-resize;
		opacity: 0;
		background: var(--interactive-accent);
		transition: opacity 0.15s;
	}
	[dir="rtl"] .resize-handle {
		right: auto;
		left: 0;
	}
	th:hover .resize-handle {
		opacity: 0.5;
	}
	.resize-handle:hover, .resize-handle:active {
		opacity: 1 !important;
	}

	tr {
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.base-row:hover td {
		background: var(--background-modifier-hover);
	}
	.base-row:hover .sticky-col {
		background: var(--background-modifier-hover);
	}

	td {
		padding: 5px 10px;
		vertical-align: middle;
		background: var(--background-primary);
		max-width: 300px;
	}

	.cell-filename {
		min-width: 180px;
		max-width: 280px;
	}

	.file-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--text-normal);
		cursor: pointer;
		padding: 2px 0;
		font-size: 0.84rem;
		text-align: start;
		max-width: 100%;
	}
	.file-link span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.file-link:hover {
		color: var(--interactive-accent);
	}
	.file-link svg {
		flex-shrink: 0;
		color: var(--text-faint);
	}

	.cell-text {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		cursor: text;
		padding: 2px 4px;
		border-radius: 4px;
		min-height: 1.4em;
	}
	.cell-text:hover {
		background: var(--background-modifier-hover);
	}

	.cell-number {
		text-align: end;
		font-variant-numeric: tabular-nums;
	}

	.cell-checkbox {
		text-align: center;
	}
	.cell-checkbox-btn {
		background: none;
		border: none;
		cursor: pointer;
		padding: 2px;
		display: inline-flex;
		align-items: center;
	}
	.cell-checkbox-btn:hover {
		opacity: 0.8;
	}

	.cell-link {
		background: none;
		border: none;
		color: var(--interactive-accent);
		cursor: pointer;
		padding: 2px 4px;
		text-decoration: underline;
		text-decoration-style: dotted;
		text-underline-offset: 2px;
		font-size: inherit;
		text-align: start;
	}
	.cell-link:hover {
		text-decoration-style: solid;
	}

	.cell-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
		cursor: text;
		padding: 1px 0;
	}
	.cell-tag {
		display: inline-block;
		background: var(--background-modifier-hover);
		color: var(--text-muted);
		padding: 1px 7px;
		border-radius: 10px;
		font-size: 0.76rem;
		white-space: nowrap;
	}

	.cell-input {
		width: 100%;
		background: var(--background-primary);
		border: 1.5px solid var(--interactive-accent);
		border-radius: 4px;
		padding: 2px 6px;
		font-size: inherit;
		font-family: inherit;
		color: var(--text-normal);
		outline: none;
		box-sizing: border-box;
	}
</style>

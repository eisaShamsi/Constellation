<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { executeDataviewQuery } from '$lib/dataview/store';
	import type { DataviewResult, DataviewRow } from '$lib/dataview/types';
	import { detectDir } from '$lib/utils';

	let {
		query,
		libraryPaths,
		onNoteClick,
	}: {
		query: string;
		libraryPaths: [string, string][];
		onNoteClick?: (path: string, libraryName: string) => void;
	} = $props();

	let result = $state<DataviewResult | null>(null);
	let loading = $state(true);
	let collapsed = $state(false);

	async function runQuery() {
		loading = true;
		try {
			result = await executeDataviewQuery(query, libraryPaths);
		} catch (e: any) {
			result = {
				query_type: 'error',
				rows: [],
				columns: [],
				total_count: 0,
				query_time_ms: 0,
				group_by: null,
				error: e?.message || String(e),
			};
		}
		loading = false;
	}

	onMount(() => {
		runQuery();
	});

	function handleNoteClick(row: DataviewRow) {
		onNoteClick?.(row.file_path, row.library_name);
	}

	function formatValue(value: string): string {
		if (!value) return '';
		// Render wikilinks as clickable text
		return value.replace(/\[\[([^\]]+)\]\]/g, '$1');
	}

	function stripMdExtension(name: string): string {
		return name.replace(/\.md$/, '');
	}
</script>

<div class="dataview-block" class:collapsed>
	<!-- Header -->
	<div class="dv-header">
		<button class="dv-toggle" onclick={() => collapsed = !collapsed}>
			<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				{#if collapsed}
					<polyline points="9 18 15 12 9 6"/>
				{:else}
					<polyline points="6 9 12 15 18 9"/>
				{/if}
			</svg>
		</button>
		<span class="dv-label">{$t('dataview.label')}</span>
		<code class="dv-query-preview">{query.length > 60 ? query.slice(0, 60) + '...' : query}</code>
		<button class="dv-refresh" onclick={runQuery} title={$t('dataview.refresh')}>
			<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polyline points="23 4 23 10 17 10"/>
				<path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
			</svg>
		</button>
	</div>

	{#if !collapsed}
		<div class="dv-content">
			{#if loading}
				<div class="dv-loading">{$t('dataview.loading')}</div>
			{:else if result?.error}
				<div class="dv-error">{result.error}</div>
			{:else if result}
				{#if result.query_type === 'table'}
					<!-- Table view -->
					<div class="dv-table-wrap">
						<table class="dv-table">
							<thead>
								<tr>
									<th class="dv-th-file">{$t('dataview.file')}</th>
									{#each result.columns as col}
										<th>{col}</th>
									{/each}
								</tr>
							</thead>
							<tbody>
								{#each result.rows as row}
									<tr>
										<td class="dv-td-file">
											<button class="dv-file-link" onclick={() => handleNoteClick(row)}>
												{stripMdExtension(row.file_name)}
											</button>
										</td>
										{#each result.columns as col}
											<td dir={detectDir(row.properties[col] || '')}>
												{formatValue(row.properties[col] || '')}
											</td>
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else if result.query_type === 'list'}
					<!-- List view -->
					<ul class="dv-list">
						{#each result.rows as row}
							<li>
								<button class="dv-file-link" onclick={() => handleNoteClick(row)}>
									{stripMdExtension(row.file_name)}
								</button>
							</li>
						{/each}
					</ul>
				{:else}
					<div class="dv-empty">{$t('dataview.unsupportedType')}: {result.query_type}</div>
				{/if}

				<!-- Footer -->
				<div class="dv-footer">
					{result.rows.length} {$t('dataview.results')}
					{#if result.total_count > result.rows.length}
						{$t('dataview.of')} {result.total_count} {$t('dataview.total')}
					{/if}
					&middot; {result.query_time_ms}ms
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.dataview-block {
		margin: 12px 0;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		overflow: hidden;
		font-size: 13px;
	}

	.dv-header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		user-select: none;
	}

	.collapsed .dv-header {
		border-bottom: none;
	}

	.dv-toggle {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-muted);
		padding: 2px;
		display: flex;
		align-items: center;
	}
	.dv-toggle:hover { color: var(--text-normal); }

	.dv-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--interactive-accent);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.dv-query-preview {
		flex: 1;
		font-size: 11px;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		background: none;
		padding: 0;
	}

	.dv-refresh {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--text-muted);
		padding: 2px;
		display: flex;
		align-items: center;
		opacity: 0;
		transition: opacity 0.15s;
	}
	.dv-header:hover .dv-refresh { opacity: 1; }
	.dv-refresh:hover { color: var(--text-normal); }

	.dv-content {
		padding: 0;
	}

	.dv-loading, .dv-error, .dv-empty {
		padding: 16px;
		text-align: center;
		color: var(--text-muted);
		font-size: 12px;
	}

	.dv-error {
		color: var(--text-error);
		background: rgba(255, 0, 0, 0.05);
	}

	/* Table */
	.dv-table-wrap {
		overflow-x: auto;
	}

	.dv-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	.dv-table th {
		text-align: start;
		padding: 6px 10px;
		font-weight: 600;
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.3px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		position: sticky;
		top: 0;
	}

	.dv-table td {
		padding: 5px 10px;
		border-bottom: 1px solid var(--background-modifier-border-hover, rgba(0,0,0,0.04));
		color: var(--text-normal);
	}

	.dv-table tr:hover td {
		background: var(--background-modifier-hover);
	}

	/* File link */
	.dv-file-link {
		background: none;
		border: none;
		color: var(--interactive-accent);
		cursor: pointer;
		padding: 0;
		font-size: inherit;
		text-decoration: none;
	}
	.dv-file-link:hover {
		text-decoration: underline;
	}

	/* List */
	.dv-list {
		list-style: none;
		padding: 8px 12px;
		margin: 0;
	}
	.dv-list li {
		padding: 3px 0;
	}

	/* Footer */
	.dv-footer {
		padding: 4px 10px;
		font-size: 11px;
		color: var(--text-faint);
		border-top: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		text-align: end;
	}
</style>

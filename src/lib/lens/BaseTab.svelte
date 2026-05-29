<script lang="ts">
	/**
	 * MIG-065 §F.2 — the standalone `.base` file rendered as a full-tab table.
	 *
	 * The "Simple" default face of the Unified Progressive Base: an Obsidian-
	 * familiar editable-name table, fed by the SQL `execute_lens` engine (NOT the
	 * retiring `query_base` live-scan). A standalone `.base` file is just lens
	 * YAML on disk — identical to an inline ` ```base ` block's body — so we hand
	 * its text straight to `executeLens`.
	 *
	 * Column semantics (which columns, header labels, cell formatting) come from
	 * the shared `$lib/lens/tableModel` so this tab and the inline
	 * `LensBlockWidget._renderTable` can never drift (CLAUDE.md "secure the
	 * winning — one source of truth").
	 *
	 * §F.2 scope = read-only familiar table (Strong-yet-Simple default). The
	 * "+ Add column" picker + resize/reorder + `columns:` save path land in §G;
	 * edit-in-place in §H.
	 */
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { executeLens, type LensResult, type LensRow } from '$lib/lens/store';
	import { dataColumns, columnLabel, renderCellValue } from '$lib/lens/tableModel';

	let {
		path,
		content,
	}: {
		/** Absolute path of the `.base` file (used as a stable id; the `columns:`
		 *  write-back in §G/§H targets it). */
		path: string;
		/** Raw `.base` YAML text, loaded by `openNoteTab`. */
		content: string;
	} = $props();

	let result = $state<LensResult | null>(null);
	let error = $state<string | null>(null);
	let loading = $state(true);

	// Re-evaluate whenever the `.base` YAML changes. Per CLAUDE.md Rule 2 (no
	// $effect loops): the effect's only reactive dependency is `content`; it
	// writes to disjoint state (`result`/`error`/`loading`) and never reads them
	// reactively. `lastRun` is a plain (non-$state) var so the guard creates no
	// reactivity. The captured `reqYaml` check drops out-of-order async results.
	let lastRun: string | null = null;
	$effect(() => {
		const yaml = content ?? '';
		if (yaml === lastRun) return;
		lastRun = yaml;
		const reqYaml = yaml;
		loading = true;
		error = null;
		executeLens(reqYaml)
			.then((res) => {
				if (lastRun !== reqYaml) return; // a newer evaluation superseded this one
				result = res;
				error = null;
			})
			.catch((e: unknown) => {
				if (lastRun !== reqYaml) return;
				error = typeof e === 'string' ? e : (e as Error)?.message ?? String(e);
				result = null;
			})
			.finally(() => {
				if (lastRun === reqYaml) loading = false;
			});
	});

	const cols = $derived(result ? dataColumns(result.columns) : []);

	/** Open a row's note. Dispatches the same `constellation:open-note` event the
	 *  Boss-validated inline table uses (handled in `+layout.svelte`), so every
	 *  base/lens surface shares one open-note path. */
	function openNote(row: LensRow) {
		window.dispatchEvent(
			new CustomEvent('constellation:open-note', {
				detail: {
					path: row.note_path,
					libraryName: row.library_name,
					libraryPath: row.library_path,
				},
			}),
		);
	}
</script>

<div class="base-tab" dir={$dir} data-base-path={path}>
	{#if loading && !result}
		<div class="base-state">{$t('lensBlock.loading') || 'Loading…'}</div>
	{:else if error}
		<div class="base-state base-error">
			<span class="base-error-label">{$t('lensBlock.errorLabel') || 'Base error'}</span>
			<span class="base-error-msg" dir="auto">{error}</span>
		</div>
	{:else if result}
		<div class="base-header">
			<h2 class="base-name" dir={detectDir(result.lens_name)}>{result.lens_name}</h2>
			<span class="base-count">{result.total_count}</span>
		</div>

		{#if result.rows.length === 0}
			<div class="base-state">{$t('lensBlock.empty') || 'No notes match this base.'}</div>
		{:else}
			<div class="base-table-scroll">
				<table class="base-table">
					<thead>
						<tr>
							<th class="th-name">{$t('lensBlock.colName') || 'Name'}</th>
							{#each cols as c (c)}
								<th dir="auto">{columnLabel(c, $t)}</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each result.rows as row (row.note_path)}
							<tr class="base-trow">
								<td class="cell-name" dir={detectDir(row.name)}>
									<button
										type="button"
										class="row-name"
										title={row.note_path}
										dir={detectDir(row.name)}
										onclick={() => openNote(row)}
									>
										{row.name}
									</button>
								</td>
								{#each cols as c (c)}
									{@const text = renderCellValue(row.dimensions[c], c)}
									<td dir={text ? detectDir(text) : undefined}>{text}</td>
								{/each}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}

		<div class="base-footer">
			<span class="base-time">{result.query_time_ms}ms</span>
		</div>
	{/if}
</div>

<style>
	.base-tab {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		padding: 18px 24px;
		box-sizing: border-box;
	}

	.base-header {
		display: flex;
		align-items: baseline;
		gap: 10px;
		margin-bottom: 12px;
		flex-shrink: 0;
	}
	.base-name {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 650;
		color: var(--text-normal);
	}
	.base-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 1.4em;
		height: 1.4em;
		padding: 0 0.5em;
		border-radius: 999px;
		background: var(--interactive-accent);
		color: #fff;
		font-size: 0.78rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}

	.base-table-scroll {
		flex: 1 1 auto;
		min-height: 0;
		overflow: auto;
	}
	.base-table {
		border-collapse: collapse;
		width: max-content;
		min-width: 100%;
		font-size: 0.9rem;
	}
	.base-table thead {
		position: sticky;
		top: 0;
		z-index: 1;
	}
	.base-table th {
		text-align: start;
		padding: 7px 12px;
		background: var(--background-secondary);
		border-bottom: 2px solid var(--background-modifier-border);
		color: var(--text-muted);
		font-weight: 600;
		font-size: 0.8rem;
		white-space: nowrap;
	}
	.base-table td {
		padding: 6px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
		vertical-align: top;
		max-width: 360px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--text-normal);
	}
	.base-trow:hover td {
		background: var(--background-modifier-hover);
	}

	.cell-name {
		font-weight: 500;
	}
	.row-name {
		background: none;
		border: none;
		padding: 0;
		margin: 0;
		font: inherit;
		font-weight: 500;
		color: var(--text-normal);
		cursor: pointer;
		text-align: start;
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row-name:hover {
		color: var(--interactive-accent);
		text-decoration: underline;
		text-underline-offset: 2px;
	}

	.base-footer {
		flex-shrink: 0;
		padding-top: 8px;
		color: var(--text-faint);
		font-size: 0.75rem;
	}

	.base-state {
		color: var(--text-muted);
		padding: 12px 0;
		font-size: 0.9rem;
	}
	.base-error {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.base-error-label {
		color: var(--text-error, #e53e3e);
		font-weight: 600;
	}
	.base-error-msg {
		color: var(--text-muted);
		font-family: var(--font-monospace, monospace);
		font-size: 0.82rem;
		white-space: pre-wrap;
	}
</style>

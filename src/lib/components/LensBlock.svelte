<!--
	MIG-055 §D — LensBlock renderer.

	Mounts inside a CM6 widget when the editor encounters a ` ```base `
	fenced code block (wired in `src/lib/editor/livePreview.ts`). Reads
	the block text (YAML lens definition), calls `execute_lens` via the
	`$lib/lens/store` bridge, and renders the resulting `LensRow[]` as a
	list view (v1 ships `view: list` only per Architect §11 #4 lock).

	UX states (per Plan §D):
	- Loading  → small "Loading lens…" muted text
	- Error    → red text with the validator's error message
	- Empty    → muted "No notes match this lens"
	- Success  → list of rows, each showing `name — headline`

	Clicking a row's name dispatches `constellation:open-note` (the same
	custom event used by `UniversalEmbedWidget` for `![[wikilink]]`
	transclusions). The app-shell handler routes it to the active pane.

	LL-022 (Lazy mount). The component attaches no global subscriptions;
	the only side effect is the one-shot `executeLens` call on mount. No
	cleanup needed beyond what `$effect` already gives us.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { executeLens, type LensResult, type LensRow, type DimensionValue } from '$lib/lens/store';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	let { lensYaml }: { lensYaml: string } = $props();

	let result = $state<LensResult | null>(null);
	let error = $state<string>('');
	let loading = $state<boolean>(true);

	onMount(() => {
		let cancelled = false;
		executeLens(lensYaml)
			.then((res) => {
				if (cancelled) return;
				result = res;
				loading = false;
			})
			.catch((err: unknown) => {
				if (cancelled) return;
				error = typeof err === 'string' ? err : (err as Error)?.message ?? String(err);
				loading = false;
			});
		return () => {
			cancelled = true;
		};
	});

	function openNote(row: LensRow) {
		// Same custom event UniversalEmbedWidget uses for ![[wikilink]]
		// transclusions — the app-shell layout listens for this and opens
		// the note in the active pane (or a new tab on modifier-click).
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

	function getHeadline(row: LensRow): string {
		// The lens may or may not have requested `note.headline` as a column.
		// If it did, it'll be in `dimensions["note.headline"]` (Text or Null).
		const v = row.dimensions['note.headline'] as DimensionValue | undefined;
		if (typeof v === 'string') return v;
		return '';
	}
</script>

<div class="lens-block">
	{#if loading}
		<div class="lens-loading" aria-live="polite">{$t('lensBlock.loading') || 'Loading lens…'}</div>
	{:else if error}
		<div class="lens-error" role="alert">
			<span class="lens-error-label">{$t('lensBlock.errorLabel') || 'Lens error'}:</span>
			<span class="lens-error-msg">{error}</span>
		</div>
	{:else if result}
		<header class="lens-header">
			<h3 class="lens-name" dir={detectDir(result.lens_name)}>{result.lens_name}</h3>
			<span class="lens-count">{result.total_count}</span>
		</header>

		{#if result.rows.length === 0}
			<div class="lens-empty">{$t('lensBlock.empty') || 'No notes match this lens.'}</div>
		{:else}
			<ul class="lens-rows">
				{#each result.rows as row (row.note_path)}
					{@const headline = getHeadline(row)}
					<li class="lens-row">
						<button
							type="button"
							class="lens-row-name"
							dir={detectDir(row.name)}
							onclick={() => openNote(row)}
							title={row.note_path}
						>
							{row.name}
						</button>
						{#if headline}
							<span class="lens-row-sep">—</span>
							<span class="lens-row-headline" dir={detectDir(headline)}>{headline}</span>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}

		<footer class="lens-footer">
			<span class="lens-time">{result.query_time_ms}ms</span>
		</footer>
	{/if}
</div>

<style>
	.lens-block {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: var(--background-secondary);
		padding: 10px 14px;
		margin: 8px 0;
		font-size: 0.9em;
	}

	.lens-loading {
		color: var(--text-muted);
		font-size: 0.85em;
		padding: 4px 0;
	}

	.lens-error {
		color: var(--text-error, #e53e3e);
		font-size: 0.85em;
		padding: 4px 0;
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.lens-error-label {
		font-weight: 600;
	}

	.lens-error-msg {
		font-family: var(--font-monospace);
		white-space: pre-wrap;
	}

	.lens-header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 8px;
		padding-bottom: 6px;
		border-bottom: 1px solid var(--background-modifier-border);
	}

	.lens-name {
		margin: 0;
		font-size: 0.95em;
		font-weight: 600;
		color: var(--text-normal);
	}

	.lens-count {
		font-size: 0.75em;
		color: var(--text-muted);
		background: var(--background-modifier-border);
		padding: 1px 8px;
		border-radius: 10px;
	}

	.lens-empty {
		color: var(--text-muted);
		font-style: italic;
		font-size: 0.85em;
		padding: 6px 0;
		text-align: center;
	}

	.lens-rows {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.lens-row {
		display: flex;
		align-items: baseline;
		gap: 6px;
		padding: 3px 0;
		flex-wrap: wrap;
		border-bottom: 1px dotted transparent;
	}

	.lens-row:hover {
		border-bottom-color: var(--background-modifier-border);
	}

	.lens-row-name {
		background: none;
		border: none;
		padding: 0;
		font: inherit;
		color: var(--interactive-accent, var(--text-accent));
		cursor: pointer;
		text-decoration: none;
		font-weight: 500;
	}

	.lens-row-name:hover {
		text-decoration: underline;
	}

	.lens-row-sep {
		color: var(--text-faint);
	}

	.lens-row-headline {
		color: var(--text-muted);
		font-style: italic;
	}

	.lens-footer {
		display: flex;
		justify-content: flex-end;
		margin-top: 8px;
		padding-top: 6px;
		border-top: 1px solid var(--background-modifier-border);
	}

	.lens-time {
		font-size: 0.7em;
		color: var(--text-faint);
	}
</style>

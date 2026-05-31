<script lang="ts">
	/**
	 * MIG-065 §G — the tiered "+ Add column" picker for the unified Base.
	 *
	 * The literal embodiment of "Strong yet Simple": power = *add a column*,
	 * never *learn to code*. Two sections, clearly separated (the design the
	 * "two Createds" Boss-test question motivated):
	 *
	 *   • Your fields        — frontmatter keys present in the collection
	 *                          (`discover_base_properties`), shown by their raw
	 *                          names. Editable later (§H). The Simple default.
	 *   • Constellation      — registered dimensions (friendly labels), marked
	 *                          read-only/computed. The Strong tier (grows
	 *                          MIG-066+: links, epistemics, cognitive).
	 *
	 * Selecting a field calls `onAdd(dimension)` with `prop.<key>` (your fields)
	 * or the registered dimension name (Constellation). The host (`BaseTab`)
	 * persists via `update_base_columns` and re-renders.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { discoverBaseProperties } from '$lib/lens/store';
	import { getLinkTypes } from '$lib/libraries/linkTypeRegistry';
	import { ADDABLE_REGISTERED_DIMS, columnLabel } from '$lib/lens/tableModel';

	let {
		currentColumns,
		onAdd,
		onClose,
	}: {
		/** Columns already in the base — excluded from both tiers. */
		currentColumns: string[];
		/** Append a column: `prop.<key>` (your field) or a registered dim name. */
		onAdd: (dimension: string) => void;
		onClose: () => void;
	} = $props();

	let allKeys = $state<string[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let search = $state('');
	let panelEl = $state<HTMLDivElement | null>(null);

	// Your fields = discovered frontmatter keys not already shown as a column,
	// matching the search term.
	const yourFields = $derived.by(() => {
		const q = search.trim().toLowerCase();
		return allKeys
			.filter((k) => !currentColumns.includes('prop.' + k))
			.filter((k) => !q || k.toLowerCase().includes(q));
	});

	// Constellation fields = registered dims not already shown, matching search
	// (against the friendly label).
	const constellationFields = $derived.by(() => {
		const q = search.trim().toLowerCase();
		return ADDABLE_REGISTERED_DIMS.filter((d) => !currentColumns.includes(d)).filter((d) => {
			if (!q) return true;
			return columnLabel(d, $t).toLowerCase().includes(q) || d.toLowerCase().includes(q);
		});
	});

	// MIG-067 §F — Link types = the registry's types (8 + custom) not already shown
	// as a `note.link.<id>` per-type count column, matching the search.
	const linkTypeFields = $derived.by(() => {
		const q = search.trim().toLowerCase();
		return getLinkTypes()
			.map((lt) => 'note.link.' + lt.id)
			.filter((d) => !currentColumns.includes(d))
			.filter((d) => !q || columnLabel(d, $t).toLowerCase().includes(q) || d.toLowerCase().includes(q));
	});

	const nothingLeft = $derived(!loading && yourFields.length === 0 && constellationFields.length === 0 && linkTypeFields.length === 0);

	function pick(dim: string) {
		onAdd(dim);
		onClose();
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
		discoverBaseProperties()
			.then((keys) => {
				allKeys = keys;
				loadError = null;
			})
			.catch((err: unknown) => {
				loadError = typeof err === 'string' ? err : (err as Error)?.message ?? String(err);
			})
			.finally(() => {
				loading = false;
			});
		// Defer so the opening click doesn't immediately close the panel.
		const id = setTimeout(() => window.addEventListener('mousedown', onWindowMouseDown), 0);
		window.addEventListener('keydown', onKeyDown, true);
		return () => clearTimeout(id);
	});

	onDestroy(() => {
		window.removeEventListener('mousedown', onWindowMouseDown);
		window.removeEventListener('keydown', onKeyDown, true);
	});
</script>

<div class="col-picker" bind:this={panelEl} role="dialog" aria-label={$t('lensBlock.addColumn') || 'Add column'}>
	<input
		class="cp-search"
		type="text"
		bind:value={search}
		placeholder={$t('lensBlock.searchFields') || 'Search fields…'}
		aria-label={$t('lensBlock.searchFields') || 'Search fields…'}
	/>

	<div class="cp-scroll">
		{#if loading}
			<div class="cp-state">{$t('lensBlock.loading') || 'Loading…'}</div>
		{:else if loadError}
			<div class="cp-state cp-error" dir="auto">{loadError}</div>
		{:else if nothingLeft}
			<div class="cp-state">{$t('lensBlock.allFieldsAdded') || 'All fields added.'}</div>
		{:else}
			{#if yourFields.length > 0}
				<div class="cp-section-head">{$t('lensBlock.yourFields') || 'Your fields'}</div>
				{#each yourFields as key (key)}
					<button class="cp-item" onclick={() => pick('prop.' + key)}>
						<span class="cp-item-name" dir={detectDir(key)}>{key}</span>
					</button>
				{/each}
			{/if}

			{#if constellationFields.length > 0}
				<div class="cp-section-head cp-section-power">
					{$t('lensBlock.constellationFields') || 'Constellation'}
				</div>
				{#each constellationFields as dim (dim)}
					<button class="cp-item cp-item-power" onclick={() => pick(dim)}>
						<span class="cp-item-name">{columnLabel(dim, $t)}</span>
						<span class="cp-readonly">{$t('lensBlock.readOnly') || 'read-only'}</span>
					</button>
				{/each}
			{/if}

			{#if linkTypeFields.length > 0}
				<div class="cp-section-head cp-section-power">
					{$t('lensBlock.linkTypeFields') || 'Link types'}
				</div>
				{#each linkTypeFields as dim (dim)}
					<button class="cp-item cp-item-power" onclick={() => pick(dim)}>
						<span class="cp-item-name">{columnLabel(dim, $t)}</span>
						<span class="cp-readonly">{$t('lensBlock.count') || 'count'}</span>
					</button>
				{/each}
			{/if}
		{/if}
	</div>
</div>

<style>
	.col-picker {
		position: absolute;
		top: calc(100% + 4px);
		inset-inline-end: 0;
		z-index: 50;
		width: 260px;
		max-width: 80vw;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 8px 28px rgba(0, 0, 0, 0.28);
		padding: 8px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.cp-search {
		width: 100%;
		box-sizing: border-box;
		padding: 5px 8px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-normal);
		font: inherit;
		font-size: 0.85rem;
		outline: none;
	}
	.cp-search:focus {
		border-color: var(--interactive-accent);
	}
	.cp-scroll {
		max-height: 320px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}
	.cp-section-head {
		font-size: 0.7rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-muted);
		padding: 8px 6px 3px;
	}
	.cp-section-power {
		color: var(--interactive-accent);
	}
	.cp-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
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
	.cp-item:hover {
		background: var(--background-modifier-hover);
	}
	.cp-item-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.cp-item-power .cp-item-name {
		color: var(--text-normal);
	}
	.cp-readonly {
		flex-shrink: 0;
		font-size: 0.66rem;
		color: var(--text-faint);
		background: var(--background-modifier-hover);
		padding: 1px 6px;
		border-radius: 999px;
		text-transform: lowercase;
	}
	.cp-state {
		color: var(--text-muted);
		font-size: 0.82rem;
		padding: 10px 6px;
	}
	.cp-error {
		color: var(--text-error, #e53e3e);
	}
</style>

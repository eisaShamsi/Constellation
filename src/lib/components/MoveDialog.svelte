<script lang="ts">
	// MIG-077 A3-R3 — the destination-folder picker for "Move". moveItem() existed
	// but was drag-drop-only; right-click Move needs a picker. Scoped to the
	// source's own library (move_item rejects cross-library/outside-universe
	// targets anyway) — a flat, indented, searchable folder list. The host loads
	// the candidate folders (source + its descendants + its current parent already
	// excluded) and commits the move; onConfirm may throw -> shown inline (e.g. the
	// "name already exists in the target folder" collision from move_item).
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';

	let {
		sourceName,
		folders,
		onConfirm,
		onCancel,
	}: {
		sourceName: string;
		folders: { path: string; name: string; depth: number }[];
		onConfirm: (targetFolder: string) => Promise<void> | void;
		onCancel: () => void;
	} = $props();

	let filter = $state('');
	let selected = $state<string | null>(null);
	let error = $state('');
	let submitting = $state(false);
	let filterEl: HTMLInputElement | undefined;

	const shown = $derived(
		filter.trim()
			? folders.filter((f) => f.name.toLowerCase().includes(filter.trim().toLowerCase()))
			: folders
	);

	async function submit() {
		if (!selected || submitting) return;
		submitting = true;
		error = '';
		try {
			await onConfirm(selected);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			submitting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onCancel(); }
		else if (e.key === 'Enter' && selected) { e.preventDefault(); submit(); }
	}

	onMount(() => filterEl?.focus());
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="dialog-overlay" onclick={onCancel} onkeydown={handleKeydown}>
	<div class="dialog" onclick={(e) => e.stopPropagation()}>
		<p class="dialog-title">{$t('contextMenu.move')}<span class="dialog-source" dir={detectDir(sourceName)}>{sourceName}</span></p>
		<input
			bind:this={filterEl}
			bind:value={filter}
			class="dialog-input"
			placeholder={$t('layout.search') || 'Search...'}
			dir={detectDir(filter)}
		/>
		<div class="folder-list">
			{#each shown as f (f.path)}
				<button
					class="folder-row"
					class:selected={selected === f.path}
					style="padding-inline-start: {8 + f.depth * 14}px"
					onclick={() => { selected = f.path; error = ''; }}
					ondblclick={() => { selected = f.path; submit(); }}
				>
					<svg class="folder-ic" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
					<span class="folder-row-name" dir={detectDir(f.name)}>{f.name}</span>
				</button>
			{:else}
				<p class="folder-empty">—</p>
			{/each}
		</div>
		{#if error}<p class="dialog-error">{error}</p>{/if}
		<div class="dialog-actions">
			<button class="dialog-btn cancel" onclick={onCancel}>{$t('dialogs.cancel')}</button>
			<button class="dialog-btn" onclick={submit} disabled={!selected || submitting}>{$t('contextMenu.move')}</button>
		</div>
	</div>
</div>

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		z-index: 2000;
		background: var(--background-modifier-cover);
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.dialog {
		background: var(--background-primary);
		border-radius: 8px;
		box-shadow: var(--shadow-l);
		padding: 20px 24px;
		width: 440px;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
	}
	.dialog-title {
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--text-normal);
		margin: 0 0 12px;
		display: flex;
		align-items: baseline;
		gap: 8px;
	}
	.dialog-source {
		font-weight: 400;
		font-size: 0.85rem;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.dialog-input {
		width: 100%;
		box-sizing: border-box;
		padding: 8px 10px;
		margin-bottom: 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 0.9rem;
		font-family: inherit;
	}
	.dialog-input:focus { outline: none; border-color: var(--interactive-accent); }
	.folder-list {
		max-height: 320px;
		overflow-y: auto;
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px;
		padding: 4px;
	}
	.folder-row {
		display: flex;
		align-items: center;
		gap: 7px;
		width: 100%;
		padding: 6px 8px;
		border: none;
		background: none;
		color: var(--text-normal);
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
		border-radius: 4px;
		text-align: start;
	}
	.folder-row:hover { background: var(--background-modifier-hover); }
	.folder-row.selected { background: color-mix(in srgb, var(--interactive-accent) 18%, transparent); color: var(--interactive-accent); }
	.folder-ic { flex-shrink: 0; opacity: 0.7; }
	.folder-row-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.folder-empty { text-align: center; color: var(--text-faint); margin: 12px 0; }
	.dialog-error { color: var(--text-error); font-size: 0.8rem; margin: 10px 0 0; }
	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 16px;
	}
	.dialog-btn {
		padding: 6px 16px;
		border: none;
		border-radius: 5px;
		font-size: 0.82rem;
		font-family: inherit;
		cursor: pointer;
		font-weight: 500;
		background: var(--interactive-accent);
		color: var(--text-on-accent);
	}
	.dialog-btn:hover { background: var(--interactive-accent-hover); }
	.dialog-btn:disabled { opacity: 0.5; cursor: default; }
	.dialog-btn.cancel { background: var(--background-secondary-alt); color: var(--text-muted); }
	.dialog-btn.cancel:hover { background: var(--background-modifier-border); }
</style>

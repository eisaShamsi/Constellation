<script lang="ts">
	import { parseFrontmatter, extractHeadings } from '$lib/vaults/store';
	import type { OpenTab } from '$lib/vaults/store';
	import { detectDir, renderMarkdown } from '$lib/utils';
	import { dir } from '$lib/i18n';

	let {
		tab,
		isFocused = false,
		onFocus,
		ar = false,
		color = '#7c3aed',
		splitView = false
	}: {
		tab: OpenTab | null;
		isFocused?: boolean;
		onFocus: () => void;
		ar?: boolean;
		color?: string;
		splitView?: boolean;
	} = $props();

	const parsed = $derived(tab ? parseFrontmatter(tab.content) : null);
	const properties = $derived(parsed?.properties ?? []);
	const noteBody = $derived(parsed?.body ?? '');
	const noteDir = $derived(noteBody ? detectDir(noteBody) : $dir);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pane" class:focused={isFocused} onclick={onFocus}>
	{#if tab}
		{#if splitView}
			<div class="pane-tab-bar" style:--vault-color={color}>
				<div class="pane-tab">
					<span class="pane-tab-vault">{tab.vaultName}</span>
					<span class="pane-tab-title">{tab.name}</span>
				</div>
			</div>
		{:else}
			<div class="pane-breadcrumb">
				<span class="bc-vault">{tab.vaultName}</span>
				<span class="bc-sep">/</span>
				<span class="bc-note">{tab.name}</span>
			</div>
		{/if}
		<div class="note-scroll" dir={noteDir}>
			{#if properties.length > 0}
				<div class="note-properties">
					<div class="props-header">{ar ? 'الخصائص' : 'Properties'}</div>
					{#each properties as prop}
						<div class="prop-row">
							<span class="prop-key">{prop.key}</span>
							<span class="prop-val">{prop.value || '—'}</span>
						</div>
					{/each}
				</div>
				<hr class="props-divider"/>
			{/if}
			<div class="note-content">
				{@html renderMarkdown(noteBody)}
			</div>
		</div>
	{:else}
		<div class="pane-empty">
			{ar ? 'اختر ملاحظة' : 'Select a note'}
		</div>
	{/if}
</div>

<style>
	.pane {
		flex: 1; display: flex; flex-direction: column;
		overflow: hidden; min-width: 0; min-height: 0;
	}
	.pane.focused { box-shadow: inset 0 0 0 2px #7c3aed33; }

	.pane-tab-bar {
		display: flex; align-items: flex-end;
		background: #f0f0f4; border-bottom: 1px solid #e0e0e4;
		padding: 10px 4px 0; flex-shrink: 0;
	}
	.pane-tab {
		position: relative;
		background: #fff; color: #1f2328;
		border: 1px solid #e0e0e4;
		border-top: 3px solid var(--vault-color, #7c3aed);
		border-bottom: 1px solid #fff;
		margin-bottom: -1px;
		border-radius: 6px 6px 0 0;
		padding: 4px 10px;
		font-size: 0.8rem;
		max-width: 200px;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.pane-tab-vault {
		position: absolute; bottom: 100%; inset-inline-end: 8px;
		font-size: 0.55rem; line-height: 1.3; letter-spacing: 0.02em;
		color: #1f2328;
		background: #f0f0f4;
		padding: 0 5px;
		border-radius: 3px 3px 0 0;
		border: 1px solid #e0e0e4; border-bottom: none;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%; pointer-events: none;
	}
	.pane-tab-title {
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}

	.pane-breadcrumb {
		padding: 4px 16px; border-bottom: 1px solid #f0f0f4;
		font-size: 0.78rem; color: #8b8b96; flex-shrink: 0;
		display: flex; align-items: center; min-height: 28px;
	}
	.bc-vault { color: #5c5c66; }
	.bc-sep { margin: 0 4px; color: #d0d0d6; }
	.bc-note { color: #1f2328; }

	.note-scroll { flex: 1; overflow-y: auto; padding: 1.5rem 3rem; max-width: 800px; }

	.note-properties {
		background: #f8f8fb; border: 1px solid #e8e8ec; border-radius: 6px;
		padding: 10px 14px; margin-bottom: 4px;
	}
	.props-header { font-size: 0.78rem; font-weight: 600; color: #5c5c66; margin-bottom: 6px; }
	.prop-row { display: flex; gap: 12px; padding: 3px 0; font-size: 0.82rem; border-bottom: 1px solid #f0f0f4; }
	.prop-row:last-child { border-bottom: none; }
	.prop-key { color: #5c5c66; font-weight: 500; min-width: 80px; text-align: end; }
	.prop-val { color: #1f2328; flex: 1; }
	.props-divider { border: none; border-top: 1px solid #e8e8ec; margin: 12px 0; }

	.note-content { line-height: 1.8; color: #1f2328; }

	.note-content :global(h1) { font-size: 1.8rem; margin: 1.5rem 0 0.75rem; color: #1f2328; }
	.note-content :global(h2) { font-size: 1.4rem; margin: 1.3rem 0 0.5rem; }
	.note-content :global(h3) { font-size: 1.15rem; margin: 1rem 0 0.4rem; }
	.note-content :global(p) { margin: 0.5rem 0; }
	.note-content :global(a) { color: #7c3aed; }
	.note-content :global(code) { background: #f0f0f4; padding: 0.15em 0.35em; border-radius: 3px; font-size: 0.9em; }
	.note-content :global(pre) { background: #f6f6f9; border: 1px solid #e0e0e4; border-radius: 6px; padding: 1rem; overflow-x: auto; }
	.note-content :global(pre code) { background: none; padding: 0; }
	.note-content :global(blockquote) { border-inline-start: 3px solid #7c3aed; padding: 0.25rem 1rem; margin: 0.5rem 0; color: #5c5c66; }
	.note-content :global(ul), .note-content :global(ol) { padding-inline-start: 1.5rem; }
	.note-content :global(li) { margin: 0.2rem 0; }
	.note-content :global(hr) { border: none; border-top: 1px solid #e0e0e4; margin: 1.5rem 0; }
	.note-content :global(table) { border-collapse: collapse; width: 100%; margin: 0.75rem 0; }
	.note-content :global(th), .note-content :global(td) { border: 1px solid #e0e0e4; padding: 0.4rem 0.7rem; text-align: start; }
	.note-content :global(th) { background: #f6f6f9; }
	.note-content :global(img) { max-width: 100%; border-radius: 4px; }
	.note-content :global(input[type="checkbox"]) { margin-inline-end: 0.4rem; }
	.note-content :global(strong) { font-weight: 600; }

	.pane-empty {
		flex: 1; display: flex; align-items: center; justify-content: center;
		color: #b0b0b8; font-size: 0.85rem;
	}
</style>

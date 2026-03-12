<script lang="ts">
	import { openNoteTab, vaults } from '$lib/vaults/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';

	let {
		outgoingLinks = [] as { target: string; context: string }[],
	}: {
		outgoingLinks: { target: string; context: string }[];
	} = $props();
</script>

<div class="outgoing-panel">
	<div class="ol-header">
		{$t('outgoingLinksPanel.header')}
		<span class="ol-count">{outgoingLinks.length}</span>
	</div>
	{#if outgoingLinks.length > 0}
		{#each outgoingLinks as link}
			<div class="ol-item">
				<span class="ol-target">{link.target}</span>
				<span class="ol-context">{link.context}</span>
			</div>
		{/each}
	{:else}
		<div class="ol-empty">{$t('outgoingLinksPanel.noLinks')}</div>
	{/if}
</div>

<style>
	.outgoing-panel { font-size: 0.8rem; }
	.ol-header {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 0; font-weight: 600; color: var(--text-muted); font-size: 0.75rem;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.ol-count { background: var(--background-modifier-border-focus); border-radius: 8px; padding: 0 5px; font-size: 0.7rem; color: var(--text-faint); }
	.ol-item {
		padding: 4px 8px; border-radius: 3px;
	}
	.ol-item:hover { background: var(--background-modifier-hover); }
	.ol-target { display: block; color: var(--interactive-accent); font-size: 0.8rem; }
	.ol-context { display: block; color: var(--text-faint); font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ol-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 4px 0; }
</style>

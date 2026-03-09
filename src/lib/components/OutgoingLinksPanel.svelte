<script lang="ts">
	import { openNoteTab, vaults } from '$lib/vaults/store';
	import { get } from 'svelte/store';

	let {
		outgoingLinks = [] as { target: string; context: string }[],
		ar = false,
	}: {
		outgoingLinks: { target: string; context: string }[];
		ar?: boolean;
	} = $props();
</script>

<div class="outgoing-panel">
	<div class="ol-header">
		{ar ? 'الروابط الصادرة' : 'Outgoing links'}
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
		<div class="ol-empty">{ar ? 'لا توجد روابط صادرة' : 'No outgoing links'}</div>
	{/if}
</div>

<style>
	.outgoing-panel { font-size: 0.8rem; }
	.ol-header {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 0; font-weight: 600; color: #5c5c66; font-size: 0.75rem;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.ol-count { background: var(--bg-tertiary, #e8e8ec); border-radius: 8px; padding: 0 5px; font-size: 0.7rem; color: var(--text-muted, #8b8b96); }
	.ol-item {
		padding: 4px 8px; border-radius: 3px;
	}
	.ol-item:hover { background: var(--bg-hover, #f0f0f4); }
	.ol-target { display: block; color: var(--accent, #7c3aed); font-size: 0.8rem; }
	.ol-context { display: block; color: var(--text-muted, #8b8b96); font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ol-empty { color: var(--text-faint, #b0b0b8); font-size: 0.78rem; padding: 4px 0; }
</style>

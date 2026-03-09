<script lang="ts">
	import { openNoteTab, vaults } from '$lib/vaults/store';
	import { get } from 'svelte/store';

	let {
		backlinks = [] as { name: string; path: string; context: string; vaultName: string }[],
		unlinkedMentions = [] as { name: string; path: string; context: string; vaultName: string }[],
		ar = false,
	}: {
		backlinks: { name: string; path: string; context: string; vaultName: string }[];
		unlinkedMentions: { name: string; path: string; context: string; vaultName: string }[];
		ar?: boolean;
	} = $props();

	let showUnlinked = $state(false);

	async function openLink(path: string, vaultName: string) {
		const vault = get(vaults).find(v => path.startsWith(v.path));
		await openNoteTab(path, vaultName, '#7c3aed');
	}
</script>

<div class="backlinks-panel">
	<div class="bl-section">
		<div class="bl-header">
			{ar ? 'الروابط الواردة' : 'Linked mentions'}
			<span class="bl-count">{backlinks.length}</span>
		</div>
		{#if backlinks.length > 0}
			{#each backlinks as bl}
				<button class="bl-item" onclick={() => openLink(bl.path, bl.vaultName)}>
					<span class="bl-name">{bl.name}</span>
					<span class="bl-context">{bl.context}</span>
				</button>
			{/each}
		{:else}
			<div class="bl-empty">{ar ? 'لا توجد روابط واردة' : 'No backlinks'}</div>
		{/if}
	</div>

	<div class="bl-section">
		<button class="bl-header bl-toggle" onclick={() => showUnlinked = !showUnlinked}>
			<svg class="bl-chev" class:expanded={showUnlinked} width="8" height="8" viewBox="0 0 10 10">
				<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
			</svg>
			{ar ? 'إشارات غير مرتبطة' : 'Unlinked mentions'}
			<span class="bl-count">{unlinkedMentions.length}</span>
		</button>
		{#if showUnlinked && unlinkedMentions.length > 0}
			{#each unlinkedMentions as ul}
				<button class="bl-item" onclick={() => openLink(ul.path, ul.vaultName)}>
					<span class="bl-name">{ul.name}</span>
					<span class="bl-context">{ul.context}</span>
				</button>
			{/each}
		{/if}
	</div>
</div>

<style>
	.backlinks-panel { font-size: 0.8rem; }
	.bl-section { margin-bottom: 4px; }
	.bl-header {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 0; font-weight: 600; color: #5c5c66; font-size: 0.75rem;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.bl-toggle {
		background: none; border: none; cursor: pointer; font-family: inherit; width: 100%; text-align: start;
	}
	.bl-toggle:hover { color: #1f2328; }
	.bl-count { background: #e8e8ec; border-radius: 8px; padding: 0 5px; font-size: 0.7rem; color: #8b8b96; }
	.bl-chev { transition: transform 0.15s ease; flex-shrink: 0; }
	.bl-chev.expanded { transform: rotate(90deg); }
	.bl-item {
		display: block; width: 100%; padding: 4px 8px;
		background: none; border: none; cursor: pointer; text-align: start;
		border-radius: 3px; font-family: inherit;
	}
	.bl-item:hover { background: #f0f0f4; }
	.bl-name { display: block; color: #7c3aed; font-size: 0.8rem; }
	.bl-context { display: block; color: #8b8b96; font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.bl-empty { color: #b0b0b8; font-size: 0.78rem; padding: 4px 0; }
</style>

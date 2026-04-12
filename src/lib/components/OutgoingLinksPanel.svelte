<script lang="ts">
	import { openNoteTab, libraries, resolveWikilinkCrossLibrary } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';

	let {
		outgoingLinks = [] as { target: string; context: string }[],
		activeNotePath = '',
		libraryPath = '',
		libraryColorMap = {} as Record<string, string>,
	}: {
		outgoingLinks: { target: string; context: string }[];
		activeNotePath?: string;
		libraryPath?: string;
		libraryColorMap?: Record<string, string>;
	} = $props();

	function getLibraryColor(name: string): string {
		return libraryColorMap[name] ?? '#7c3aed';
	}

	async function openLink(target: string, e?: MouseEvent) {
		if (!libraryPath) return;
		try {
			const resolved = await resolveWikilinkCrossLibrary(libraryPath, target);
			if (resolved) {
				const newTab = e ? (e.ctrlKey || e.metaKey) : false;
				await openNoteTab(resolved.path, resolved.libraryName, getLibraryColor(resolved.libraryName), undefined, newTab, activeNotePath || undefined);
			}
		} catch {}
	}
</script>

<div class="outgoing-panel">
	<div class="ol-header">
		{$t('outgoingLinksPanel.header')}
		<span class="ol-count">{outgoingLinks.length}</span>
	</div>
	{#if outgoingLinks.length > 0}
		{#each outgoingLinks as link}
			<button class="ol-item" onclick={(e) => openLink(link.target, e)} dir="auto">
				<span class="ol-target">{link.target}</span>
				<span class="ol-context">{link.context}</span>
			</button>
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
		display: block; width: 100%; text-align: start;
		background: none; border: none; cursor: pointer;
	}
	.ol-item:hover { background: var(--background-modifier-hover); }
	.ol-target { display: block; color: var(--interactive-accent); font-size: 0.8rem; }
	.ol-context { display: block; color: var(--text-faint); font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ol-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 4px 0; }
</style>

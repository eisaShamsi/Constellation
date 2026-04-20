<script lang="ts">
	import { openNoteTab, libraries, resolveWikilinkCrossLibrary } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';

	// Link-type color palette — mirrors Backlinks + GraphMind + livePreview.
	const LINK_TYPE_COLORS: Record<string, string> = {
		supports:     '#4A9EFF',
		contradicts:  '#FF4A4A',
		causes:       '#FF8C42',
		exemplifies:  '#4AFF88',
		generalizes:  '#A44AFF',
		'derives-from': '#FFD700',
		'part-of':    '#AAAAAA',
		associative:  '#888888',
	};
	// Text color paired with each solid fill for AA contrast — matches
	// BacklinksPanel. Bright fills take dark text; saturated fills take white.
	const LINK_TYPE_TEXT: Record<string, string> = {
		supports:     '#ffffff',
		contradicts:  '#ffffff',
		causes:       '#ffffff',
		exemplifies:  '#0b2e18',
		generalizes:  '#ffffff',
		'derives-from': '#3d2e00',
		'part-of':    '#1a1a1a',
		associative:  '#ffffff',
	};

	let {
		outgoingLinks = [] as { target: string; context: string; traversalCount?: number; linkType?: string }[],
		activeNotePath = '',
		libraryPath = '',
		libraryColorMap = {} as Record<string, string>,
	}: {
		outgoingLinks: { target: string; context: string; traversalCount?: number; linkType?: string }[];
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
				<span class="ol-target-row">
					<span class="ol-target">{link.target}</span>
					{#if link.linkType}
						{@const fill = LINK_TYPE_COLORS[link.linkType] ?? '#888'}
						{@const txt = LINK_TYPE_TEXT[link.linkType] ?? '#ffffff'}
						<span class="ol-link-type-badge"
							style="color:{txt};background:{fill};border-color:{fill}"
						>{link.linkType}</span>
					{/if}
					{#if (link.traversalCount ?? 0) > 0}
						<span class="ol-traversal-chip" title={`Traversed ${link.traversalCount} time${link.traversalCount === 1 ? '' : 's'}`}>×{link.traversalCount}</span>
					{/if}
				</span>
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
	.ol-count {
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent, #7c3aed) 30%, transparent);
		color: var(--interactive-accent, #7c3aed);
		border-radius: 8px; padding: 0 6px;
		font-size: 0.7rem; font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.ol-item {
		padding: 4px 8px; border-radius: 3px;
		display: block; width: 100%; text-align: start;
		background: none; border: none; cursor: pointer;
	}
	.ol-item:hover { background: var(--background-modifier-hover); }
	.ol-target-row { display: flex; align-items: center; gap: 4px; }
	.ol-target { color: var(--interactive-accent); font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ol-context { display: block; color: var(--text-faint); font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ol-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 4px 0; }
	.ol-link-type-badge {
		font-size: 0.65rem; font-weight: 500; padding: 0 5px;
		border-radius: 8px; border: 1px solid; white-space: nowrap; flex-shrink: 0;
		text-transform: lowercase; letter-spacing: 0.02em;
	}
	.ol-traversal-chip {
		font-size: 0.65rem; font-weight: 600; padding: 0 5px;
		border-radius: 8px; white-space: nowrap; flex-shrink: 0;
		color: var(--interactive-accent, #7c3aed);
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent, #7c3aed) 30%, transparent);
		letter-spacing: 0.02em; font-variant-numeric: tabular-nums;
	}
</style>

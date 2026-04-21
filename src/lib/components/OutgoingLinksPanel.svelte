<script lang="ts">
	import { openNoteTab, libraries, resolveWikilinkCrossLibrary, appSettings } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';

	// Pill colors + shape come from $appSettings.linkPills — matches
	// BacklinksPanel. User-configurable via Settings → Appearance.
	const LINK_TYPE_COLORS = $derived($appSettings.linkPills?.fill ?? {});
	const LINK_TYPE_TEXT   = $derived($appSettings.linkPills?.text ?? {});
	const pillShape        = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });

	let {
		outgoingLinks = [] as { target: string; context: string; traversalCount?: number; linkType?: string; tier?: string }[],
		activeNotePath = '',
		libraryPath = '',
		libraryColorMap = {} as Record<string, string>,
	}: {
		outgoingLinks: { target: string; context: string; traversalCount?: number; linkType?: string; tier?: string }[];
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

<div class="outgoing-panel" style="--pill-radius:{pillShape.radius}px;--pill-height:{pillShape.height}px;--pill-weight:{pillShape.fontWeight}">
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
						>{$t(`linkTypes.${link.linkType}`) || link.linkType}</span>
					{/if}
					{#if (link.traversalCount ?? 0) > 0}
						<span class="ol-traversal-chip ol-tier-{link.tier ?? 'emerging'}" title={`Traversed ${link.traversalCount} time${link.traversalCount === 1 ? '' : 's'} · ${link.tier ?? 'emerging'}`}>×{link.traversalCount}</span>
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
		display: inline-flex; align-items: center;
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent, #7c3aed) 30%, transparent);
		color: var(--interactive-accent, #7c3aed);
		border-radius: var(--pill-radius, 10px); padding: 0 8px;
		height: var(--pill-height, 20px); line-height: 1;
		font-size: 0.7rem; font-weight: var(--pill-weight, 700);
		font-variant-numeric: tabular-nums;
		box-sizing: border-box;
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
		display: inline-flex; align-items: center;
		font-size: 0.65rem; font-weight: var(--pill-weight, 700); line-height: 1;
		padding: 0 8px; height: var(--pill-height, 20px);
		border-radius: var(--pill-radius, 10px); border: 1px solid;
		white-space: nowrap; flex-shrink: 0;
		text-transform: lowercase; letter-spacing: 0.02em;
		box-sizing: border-box;
	}
	.ol-traversal-chip {
		display: inline-flex; align-items: center;
		font-size: 0.65rem; font-weight: var(--pill-weight, 700); line-height: 1;
		padding: 0 8px; height: var(--pill-height, 20px);
		border-radius: var(--pill-radius, 10px); white-space: nowrap; flex-shrink: 0;
		color: var(--interactive-accent, #7c3aed);
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent, #7c3aed) 30%, transparent);
		letter-spacing: 0.02em; font-variant-numeric: tabular-nums;
		box-sizing: border-box;
	}
	/* P5 slice 3 — per-tier gradient, mirrors BacklinksPanel. */
	.ol-tier-emerging { /* default */ }
	.ol-tier-established {
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 26%, transparent);
		border-color: color-mix(in srgb, var(--interactive-accent, #7c3aed) 55%, transparent);
	}
	.ol-tier-load-bearing {
		background: var(--interactive-accent, #7c3aed);
		border-color: var(--interactive-accent, #7c3aed);
		color: #fff;
	}
	.ol-tier-stale {
		background: color-mix(in srgb, #d97706 14%, transparent);
		border-color: color-mix(in srgb, #d97706 30%, transparent);
		color: #d97706;
	}
</style>

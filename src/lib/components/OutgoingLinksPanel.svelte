<script lang="ts">
	import { openNoteTab, libraries, resolveWikilinkCrossLibrary, appSettings, setLinkConfidence, archiveLink, type LinkConfidence } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';

	// Pill colors + shape come from $appSettings.linkPills — matches
	// BacklinksPanel. User-configurable via Settings → Appearance.
	const LINK_TYPE_COLORS = $derived($appSettings.linkPills?.fill ?? {});
	const LINK_TYPE_TEXT   = $derived($appSettings.linkPills?.text ?? {});
	const pillShape        = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });

	/** Format ISO-8601 last_traversed to a short relative label for the tooltip. */
	function fmtTraversed(iso: string): string {
		if (!iso) return '';
		const d = new Date(iso);
		if (isNaN(d.getTime())) return '';
		const days = Math.floor((Date.now() - d.getTime()) / 86400000);
		if (days === 0) return 'today';
		if (days === 1) return 'yesterday';
		if (days < 7) return `${days}d ago`;
		if (days < 30) return `${Math.floor(days / 7)}w ago`;
		if (days < 365) return `${Math.floor(days / 30)}mo ago`;
		return `${Math.floor(days / 365)}y ago`;
	}

	let {
		outgoingLinks = [] as { target: string; context: string; traversalCount?: number; lastTraversed?: string; linkType?: string; tier?: string; confidence?: LinkConfidence; annotation?: string }[],
		activeNotePath = '',
		libraryPath = '',
		libraryColorMap = {} as Record<string, string>,
		onConfidenceChange = undefined as undefined | ((sourcePath: string, targetName: string, confidence: LinkConfidence) => void),
		onArchive = undefined as undefined | ((sourcePath: string, targetName: string) => void),
	}: {
		outgoingLinks: { target: string; context: string; traversalCount?: number; lastTraversed?: string; linkType?: string; tier?: string; confidence?: LinkConfidence; annotation?: string }[];
		activeNotePath?: string;
		libraryPath?: string;
		libraryColorMap?: Record<string, string>;
		onConfidenceChange?: (sourcePath: string, targetName: string, confidence: LinkConfidence) => void;
		onArchive?: (sourcePath: string, targetName: string) => void;
	} = $props();

	let showOutgoing = $state(true);

	// Confidence popover — mirrors BacklinksPanel.
	let confMenu = $state<{ x: number; y: number; sourcePath: string; targetName: string; current: LinkConfidence } | null>(null);
	const CONFIDENCE_LEVELS: LinkConfidence[] = ['hypothesis', 'evidence', 'established', 'contested'];
	function openConfMenu(e: MouseEvent, sourcePath: string, targetName: string, current: LinkConfidence) {
		e.preventDefault();
		e.stopPropagation();
		confMenu = { x: e.clientX, y: e.clientY, sourcePath, targetName, current };
	}
	async function applyConf(level: LinkConfidence) {
		if (!confMenu) return;
		const { sourcePath, targetName } = confMenu;
		confMenu = null;
		try {
			await setLinkConfidence(sourcePath, targetName, level);
			onConfidenceChange?.(sourcePath, targetName, level);
		} catch { /* ignore */ }
	}
	async function applyArchive() {
		if (!confMenu) return;
		const { sourcePath, targetName } = confMenu;
		confMenu = null;
		try {
			await archiveLink(sourcePath, targetName);
			onArchive?.(sourcePath, targetName);
		} catch { /* ignore */ }
	}

	function getLibraryColor(name: string): string {
		return libraryColorMap[name] ?? '#7c3aed';
	}

	async function openLink(target: string, e?: MouseEvent) {
		if (!libraryPath) return;
		try {
			const resolved = await resolveWikilinkCrossLibrary(libraryPath, target);
			if (resolved) {
				const newTab = e ? (e.ctrlKey || e.metaKey) : false;
				await openNoteTab(resolved.path, resolved.library_name, getLibraryColor(resolved.library_name), undefined, newTab, activeNotePath || undefined);
			}
		} catch {}
	}
</script>

<div class="outgoing-panel" style="--pill-radius:{pillShape.radius}px;--pill-height:{pillShape.height}px;--pill-weight:{pillShape.fontWeight}">
	<button class="ol-header ol-toggle" onclick={() => showOutgoing = !showOutgoing}>
		<svg class="ol-chev" class:expanded={showOutgoing} width="8" height="8" viewBox="0 0 10 10">
			<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
		</svg>
		{$t('outgoingLinksPanel.header')}
		<span class="ol-count">{outgoingLinks.length}</span>
	</button>
	{#if showOutgoing && outgoingLinks.length > 0}
		{#each outgoingLinks as link}
			<button class="ol-item" onclick={(e) => openLink(link.target, e)} dir="auto"
				oncontextmenu={(e) => openConfMenu(e, activeNotePath, link.target, link.confidence ?? 'hypothesis')}
				title={$t('linkConfidence.rightClickHint') || 'Right-click to set confidence'}>
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
						{@const ltLabel = fmtTraversed(link.lastTraversed ?? '')}
						<span class="ol-traversal-chip ol-tier-{link.tier ?? 'emerging'}"
							title={`Traversed ${link.traversalCount} time${link.traversalCount === 1 ? '' : 's'} · ${link.tier ?? 'emerging'}${ltLabel ? ' · Last: ' + ltLabel : ''}`}>×{link.traversalCount}</span>
					{/if}
				</span>
				<span class="ol-context">{link.context}</span>
				{#if link.annotation}
					<span class="ol-annotation" title={link.annotation}>“{link.annotation}”</span>
				{/if}
			</button>
		{/each}
	{:else if showOutgoing}
		<div class="ol-empty">{$t('outgoingLinksPanel.noLinks')}</div>
	{/if}
</div>

{#if confMenu}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="conf-overlay" onclick={() => confMenu = null} oncontextmenu={(e) => { e.preventDefault(); confMenu = null; }}></div>
	<div class="conf-menu" style="left:{confMenu.x}px;top:{confMenu.y}px">
		<div class="conf-menu-header">{$t('linkConfidence.setConfidence') || 'Set confidence'}</div>
		{#each CONFIDENCE_LEVELS as level}
			<button class="conf-menu-item" class:active={level === confMenu.current} onclick={() => applyConf(level)}>
				<span class="conf-dot conf-dot-{level}"></span>
				{$t(`linkConfidence.${level}`) || level}
			</button>
		{/each}
		<div class="conf-menu-sep"></div>
		<button class="conf-menu-item conf-menu-archive" onclick={applyArchive}>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 8v13H3V8M1 3h22v5H1zM10 12h4"/></svg>
			{$t('linkConfidence.archive') || 'Archive link'}
		</button>
	</div>
{/if}

<style>
	.outgoing-panel { font-size: 0.8rem; }
	.ol-header {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 0; font-weight: 600; color: var(--text-muted); font-size: 0.75rem;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.ol-toggle {
		background: none; border: none; cursor: pointer; font-family: inherit;
		width: 100%; text-align: start;
	}
	.ol-toggle:hover { color: var(--text-normal); }
	.ol-chev { transition: transform 0.15s ease; flex-shrink: 0; }
	.ol-chev.expanded { transform: rotate(90deg); }
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
	.ol-annotation {
		display: block; margin-top: 2px;
		color: var(--interactive-accent); font-size: 0.7rem; font-style: italic;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
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

	.conf-overlay { position: fixed; inset: 0; z-index: 99; background: transparent; }
	.conf-menu {
		position: fixed; z-index: 100;
		background: var(--bg-secondary, #fff);
		border: 1px solid var(--border); border-radius: 6px;
		box-shadow: 0 8px 20px rgba(0,0,0,0.18);
		padding: 4px; min-width: 160px;
		font-size: 0.78rem;
	}
	.conf-menu-header {
		padding: 6px 8px 4px; color: var(--text-muted); font-size: 0.68rem;
		text-transform: uppercase; letter-spacing: 0.04em; font-weight: 600;
	}
	.conf-menu-item {
		display: flex; align-items: center; gap: 8px;
		width: 100%; padding: 6px 8px; border: none; background: none;
		cursor: pointer; border-radius: 4px; text-align: start;
		color: var(--text-normal); font-family: inherit; font-size: 0.78rem;
	}
	.conf-menu-item:hover { background: var(--background-modifier-hover); }
	.conf-menu-item.active { font-weight: 600; color: var(--interactive-accent); }
	.conf-dot {
		width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
		border: 1px solid var(--border);
	}
	.conf-dot-hypothesis { background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent); }
	.conf-dot-evidence   { background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 40%, transparent); }
	.conf-dot-established{ background: var(--interactive-accent, #7c3aed); border-color: var(--interactive-accent, #7c3aed); }
	.conf-dot-contested  { background: #d97706; border-color: #d97706; }
	.conf-menu-sep { height: 1px; margin: 4px 4px; background: var(--border-light, var(--border)); }
	.conf-menu-archive { color: var(--text-muted); }
	.conf-menu-archive:hover { color: #d97706; }
	.conf-menu-archive svg { flex-shrink: 0; }
</style>

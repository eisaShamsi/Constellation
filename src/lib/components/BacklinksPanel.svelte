<script lang="ts">
	import { openNoteTab, libraries, readNote, appSettings, setLinkConfidence, archiveLink, type LinkConfidence } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';

	// Pill colors + shape now come from $appSettings.linkPills so the user
	// can tune them from Settings → Appearance → Living Link Pills. The
	// `?? '#...'` fallbacks keep the panel rendering during the brief
	// window between boot and settings-loaded, and cover any type the user
	// might remove from the settings object.
	const LINK_TYPE_COLORS = $derived($appSettings.linkPills?.fill ?? {});
	const LINK_TYPE_TEXT   = $derived($appSettings.linkPills?.text ?? {});
	const pillShape        = $derived($appSettings.linkPills?.shape ?? { radius: 10, height: 20, fontWeight: 700 });

	/** MIG-022 §A.4.d — same shape as OutgoingLinksPanel.displayAnnotation.
	 *  Translates known link-type names that land in the annotation
	 *  slot via $t('linkTypes.<name>'); raw fallback otherwise. */
	function displayAnnotation(annotation: string): string {
		if (!annotation) return annotation;
		const key = `linkTypes.${annotation.toLowerCase()}`;
		const translated = $t(key);
		if (translated && translated !== key) return translated;
		return annotation;
	}

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

	// `linkTypes` is the post-dedupe array of distinct typed-link badges
	// for a single source note. `linkType` is kept as a back-compat
	// optional (some legacy callers haven't updated to the deduped shape
	// yet) — the template prefers `linkTypes` when present.
	type BacklinkRow = {
		name: string; path: string; context: string; libraryName: string;
		linkType?: string;
		linkTypes?: string[];
		traversalCount?: number; lastTraversed?: string; tier?: string;
		confidence?: LinkConfidence; annotation?: string;
	};
	let {
		backlinks = [] as BacklinkRow[],
		unlinkedMentions = [] as { name: string; path: string; context: string; libraryName: string }[],
		activeNoteName = '',
		activeNotePath = '',
		libraryColorMap = {} as Record<string, string>,
		onConfidenceChange = undefined as undefined | ((sourcePath: string, targetName: string, confidence: LinkConfidence) => void),
		onArchive = undefined as undefined | ((sourcePath: string, targetName: string) => void),
	}: {
		backlinks: BacklinkRow[];
		unlinkedMentions: { name: string; path: string; context: string; libraryName: string }[];
		activeNoteName?: string;
		activeNotePath?: string;
		libraryColorMap?: Record<string, string>;
		onConfidenceChange?: (sourcePath: string, targetName: string, confidence: LinkConfidence) => void;
		onArchive?: (sourcePath: string, targetName: string) => void;
	} = $props();

	/** Resolve the row's link-type list — prefers the post-dedupe
	 *  `linkTypes[]`; falls back to wrapping a single `linkType`. */
	function rowLinkTypes(bl: BacklinkRow): string[] {
		if (bl.linkTypes && bl.linkTypes.length > 0) return bl.linkTypes;
		return bl.linkType ? [bl.linkType] : [];
	}

	// Confidence popover state. Opened via right-click on a backlink row.
	// Position is absolute-positioned relative to the viewport (fixed).
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

	let showUnlinked = $state(false);
	let showLinked = $state(true);
	let filterQuery = $state('');
	const filteredBacklinks = $derived(
		filterQuery.trim()
			? backlinks.filter(bl => bl.name.toLowerCase().includes(filterQuery.toLowerCase()) || bl.context.toLowerCase().includes(filterQuery.toLowerCase()))
			: backlinks
	);
	const filteredUnlinked = $derived(
		filterQuery.trim()
			? unlinkedMentions.filter(m => m.name.toLowerCase().includes(filterQuery.toLowerCase()) || m.context.toLowerCase().includes(filterQuery.toLowerCase()))
			: unlinkedMentions
	);

	function getLibraryColor(libraryName: string): string {
		return libraryColorMap[libraryName] || '#7c3aed';
	}

	async function openLink(path: string, libraryName: string, e?: MouseEvent) {
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(path, libraryName, getLibraryColor(libraryName), undefined, newTab, activeNotePath || undefined);
	}

	async function linkMention(mentionPath: string, e: MouseEvent) {
		e.stopPropagation();
		if (!activeNoteName) return;
		try {
			const content = await readNote(mentionPath);
			// Replace first plain-text occurrence with [[wikilink]]
			const re = new RegExp(`\\b(${activeNoteName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})\\b`, 'i');
			const newContent = content.replace(re, `[[${activeNoteName}]]`);
			if (newContent !== content) {
				await invoke('write_note', { filePath: mentionPath, content: newContent });
			}
		} catch { /* ignore */ }
	}
</script>

<div class="backlinks-panel" style="--pill-radius:{pillShape.radius}px;--pill-height:{pillShape.height}px;--pill-weight:{pillShape.fontWeight}">
	{#if backlinks.length + unlinkedMentions.length > 3}
		<div class="bl-filter">
			<input type="text" dir="auto" placeholder="Filter..." value={filterQuery} oninput={(e) => filterQuery = (e.target as HTMLInputElement).value} />
		</div>
	{/if}
	<div class="bl-section">
		<button class="bl-header bl-toggle" onclick={() => showLinked = !showLinked}>
			<svg class="bl-chev" class:expanded={showLinked} width="8" height="8" viewBox="0 0 10 10">
				<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
			</svg>
			{$t('backlinksPanel.linkedMentions')}
			<span class="bl-count">{filteredBacklinks.length}</span>
		</button>
		{#if showLinked && filteredBacklinks.length > 0}
			{#each filteredBacklinks as bl}
				<button class="bl-item" onclick={(e) => openLink(bl.path, bl.libraryName, e)}
					oncontextmenu={(e) => openConfMenu(e, bl.path, activeNoteName, bl.confidence ?? 'hypothesis')}
					title={$t('linkConfidence.rightClickHint') || 'Right-click to set confidence'}>
					<span class="bl-name-row">
						{#if bl.libraryName}
							<span class="bl-library-dot" style="background:{getLibraryColor(bl.libraryName)}"></span>
						{/if}
						<span class="bl-name">{bl.name}</span>
						{#each rowLinkTypes(bl) as lt (lt)}
							{@const fill = LINK_TYPE_COLORS[lt] ?? '#888'}
							{@const txt = LINK_TYPE_TEXT[lt] ?? '#ffffff'}
							<span class="bl-link-type-badge"
								style="color:{txt};background:{fill};border-color:{fill}"
							>{$t(`linkTypes.${lt}`) || lt}</span>
						{/each}
						{#if (bl.traversalCount ?? 0) > 0}
							{@const ltLabel = fmtTraversed(bl.lastTraversed ?? '')}
							<span class="bl-traversal-chip bl-tier-{bl.tier ?? 'emerging'}"
								title={`Traversed ${bl.traversalCount} time${bl.traversalCount === 1 ? '' : 's'} · ${bl.tier ?? 'emerging'}${ltLabel ? ' · Last: ' + ltLabel : ''}`}>×{bl.traversalCount}</span>
						{/if}
						{#if bl.libraryName}
							<span class="bl-library-label">{bl.libraryName}</span>
						{/if}
					</span>
					<span class="bl-context">{bl.context}</span>
					{#if bl.annotation}
						<span class="bl-annotation" title={bl.annotation}>“{displayAnnotation(bl.annotation)}”</span>
					{/if}
				</button>
			{/each}
		{:else if showLinked}
			<div class="bl-empty">{$t('backlinksPanel.noBacklinks')}</div>
		{/if}
	</div>

	<div class="bl-section">
		<button class="bl-header bl-toggle" onclick={() => showUnlinked = !showUnlinked}>
			<svg class="bl-chev" class:expanded={showUnlinked} width="8" height="8" viewBox="0 0 10 10">
				<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
			</svg>
			{$t('backlinksPanel.unlinkedMentions')}
			<span class="bl-count">{filteredUnlinked.length}</span>
		</button>
		{#if showUnlinked && filteredUnlinked.length > 0}
			{#each filteredUnlinked as ul}
				<div class="bl-item-row">
					<button class="bl-item" onclick={(e) => openLink(ul.path, ul.libraryName, e)}>
						<span class="bl-name-row">
							{#if ul.libraryName}
								<span class="bl-library-dot" style="background:{getLibraryColor(ul.libraryName)}"></span>
							{/if}
							<span class="bl-name">{ul.name}</span>
							{#if ul.libraryName}
								<span class="bl-library-label">{ul.libraryName}</span>
							{/if}
						</span>
						<span class="bl-context">{ul.context}</span>
					</button>
					<button class="bl-link-btn" title="Link it" onclick={(e) => linkMention(ul.path, e)}>
						<svg width="12" height="12" viewBox="0 0 16 16" fill="none">
							<path d="M6.5 10.5L9.5 7.5M5 8.5L3.5 10a2.12 2.12 0 003 3L8 11.5M8 7.5l1.5-1.5a2.12 2.12 0 013 3L11 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
						</svg>
					</button>
				</div>
			{/each}
		{/if}
	</div>
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
	.backlinks-panel { font-size: 0.8rem; }
	.bl-filter { padding: 2px 8px 4px; }
	.bl-filter input {
		width: 100%; padding: 3px 6px; border: 1px solid var(--border); border-radius: 4px;
		background: var(--bg); color: var(--text); font-size: 0.75rem; font-family: inherit; outline: none;
	}
	.bl-filter input:focus { border-color: var(--interactive-accent); }
	.bl-filter input::placeholder { color: var(--text-faint); }
	.bl-section { margin-bottom: 4px; }
	.bl-header {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 0; font-weight: 600; color: var(--text-muted); font-size: 0.75rem;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.bl-toggle {
		background: none; border: none; cursor: pointer; font-family: inherit; width: 100%; text-align: start;
	}
	.bl-toggle:hover { color: var(--text-normal); }
	.bl-count {
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
	.bl-chev { transition: transform 0.15s ease; flex-shrink: 0; }
	.bl-chev.expanded { transform: rotate(90deg); }
	.bl-item-row { display: flex; align-items: flex-start; gap: 2px; }
	.bl-item-row .bl-item { flex: 1; min-width: 0; }
	.bl-item {
		display: block; width: 100%; padding: 4px 8px;
		background: none; border: none; cursor: pointer; text-align: start;
		border-radius: 3px; font-family: inherit;
	}
	.bl-item:hover { background: var(--background-modifier-hover); }
	.bl-name-row { display: flex; align-items: center; gap: 4px; }
	.bl-library-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
	.bl-library-label { font-size: 0.68rem; color: var(--text-faint); flex-shrink: 0; }
	.bl-name { color: var(--interactive-accent); font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.bl-context { display: block; color: var(--text-faint); font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.bl-annotation {
		display: block; margin-top: 2px;
		color: var(--interactive-accent); font-size: 0.7rem; font-style: italic;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.bl-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 4px 0; }
	.bl-link-btn {
		flex-shrink: 0; background: none; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; padding: 3px 4px; cursor: pointer;
		color: var(--text-muted); margin-top: 4px;
	}
	.bl-link-btn:hover { color: var(--interactive-accent); border-color: var(--interactive-accent); }
	.bl-link-type-badge {
		display: inline-flex; align-items: center;
		font-size: 0.65rem; font-weight: var(--pill-weight, 700); line-height: 1;
		padding: 0 8px; height: var(--pill-height, 20px);
		border-radius: var(--pill-radius, 10px); border: 1px solid;
		white-space: nowrap; flex-shrink: 0;
		text-transform: lowercase; letter-spacing: 0.02em;
		box-sizing: border-box;
	}
	.bl-traversal-chip {
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
	/* P5 slice 3 — per-tier visual gradient on the ×N chip.
	   The tier class is appended to .bl-traversal-chip so each step up
	   saturates further without changing shape or size. Subtle by design:
	   the chip should signal "wear" at a glance without screaming. */
	.bl-tier-emerging {
		/* default — matches the base class above */
	}
	.bl-tier-established {
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 26%, transparent);
		border-color: color-mix(in srgb, var(--interactive-accent, #7c3aed) 55%, transparent);
	}
	.bl-tier-load-bearing {
		background: var(--interactive-accent, #7c3aed);
		border-color: var(--interactive-accent, #7c3aed);
		color: #fff;
	}
	.bl-tier-stale {
		background: color-mix(in srgb, #d97706 14%, transparent);
		border-color: color-mix(in srgb, #d97706 30%, transparent);
		color: #d97706;
	}

	/* Confidence popover (shared visual grammar with OutgoingLinksPanel). */
	.conf-overlay {
		position: fixed; inset: 0; z-index: 99; background: transparent;
	}
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

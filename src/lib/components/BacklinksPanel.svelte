<script lang="ts">
	import { openNoteTab, libraries, readNote } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';

	// Link type color palette — matches GraphMind + livePreview typed link colors
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

	let {
		backlinks = [] as { name: string; path: string; context: string; libraryName: string; linkType?: string; traversalCount?: number }[],
		unlinkedMentions = [] as { name: string; path: string; context: string; libraryName: string }[],
		activeNoteName = '',
		activeNotePath = '',
		libraryColorMap = {} as Record<string, string>,
	}: {
		backlinks: { name: string; path: string; context: string; libraryName: string; linkType?: string; traversalCount?: number }[];
		unlinkedMentions: { name: string; path: string; context: string; libraryName: string }[];
		activeNoteName?: string;
		activeNotePath?: string;
		libraryColorMap?: Record<string, string>;
	} = $props();

	let showUnlinked = $state(false);
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

<div class="backlinks-panel">
	{#if backlinks.length + unlinkedMentions.length > 3}
		<div class="bl-filter">
			<input type="text" dir="auto" placeholder="Filter..." value={filterQuery} oninput={(e) => filterQuery = (e.target as HTMLInputElement).value} />
		</div>
	{/if}
	<div class="bl-section">
		<div class="bl-header">
			{$t('backlinksPanel.linkedMentions')}
			<span class="bl-count">{filteredBacklinks.length}</span>
		</div>
		{#if filteredBacklinks.length > 0}
			{#each filteredBacklinks as bl}
				<button class="bl-item" onclick={(e) => openLink(bl.path, bl.libraryName, e)}>
					<span class="bl-name-row">
						{#if bl.libraryName}
							<span class="bl-library-dot" style="background:{getLibraryColor(bl.libraryName)}"></span>
						{/if}
						<span class="bl-name">{bl.name}</span>
						{#if bl.linkType}
							<span class="bl-link-type-badge"
								style="color:{LINK_TYPE_COLORS[bl.linkType] ?? '#888'};border-color:{LINK_TYPE_COLORS[bl.linkType] ?? '#888'}20"
							>{bl.linkType}</span>
						{/if}
						{#if (bl.traversalCount ?? 0) > 0}
							<span class="bl-traversal-chip" title={`Traversed ${bl.traversalCount} time${bl.traversalCount === 1 ? '' : 's'}`}>×{bl.traversalCount}</span>
						{/if}
						{#if bl.libraryName}
							<span class="bl-library-label">{bl.libraryName}</span>
						{/if}
					</span>
					<span class="bl-context">{bl.context}</span>
				</button>
			{/each}
		{:else}
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
	.bl-count { background: var(--background-modifier-border-focus); border-radius: 8px; padding: 0 5px; font-size: 0.7rem; color: var(--text-faint); }
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
	.bl-library-label { font-size: 0.68rem; color: var(--text-faint); margin-inline-start: auto; flex-shrink: 0; }
	.bl-name { color: var(--interactive-accent); font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.bl-context { display: block; color: var(--text-faint); font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.bl-empty { color: var(--color-base-40); font-size: 0.78rem; padding: 4px 0; }
	.bl-link-btn {
		flex-shrink: 0; background: none; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; padding: 3px 4px; cursor: pointer;
		color: var(--text-muted); margin-top: 4px;
	}
	.bl-link-btn:hover { color: var(--interactive-accent); border-color: var(--interactive-accent); }
	.bl-link-type-badge {
		font-size: 0.65rem; font-weight: 500; padding: 0 5px;
		border-radius: 8px; border: 1px solid; white-space: nowrap; flex-shrink: 0;
		text-transform: lowercase; letter-spacing: 0.02em;
	}
	.bl-traversal-chip {
		font-size: 0.65rem; font-weight: 600; padding: 0 5px;
		border-radius: 8px; white-space: nowrap; flex-shrink: 0;
		color: var(--interactive-accent, #7c3aed);
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--interactive-accent, #7c3aed) 30%, transparent);
		letter-spacing: 0.02em; font-variant-numeric: tabular-nums;
	}
</style>

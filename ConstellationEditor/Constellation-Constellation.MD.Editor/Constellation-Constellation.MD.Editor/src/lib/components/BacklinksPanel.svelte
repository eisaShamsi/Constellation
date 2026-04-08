<script lang="ts">
	import { openNoteTab, libraries, readNote } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';

	let {
		backlinks = [] as { name: string; path: string; context: string; libraryName: string }[],
		unlinkedMentions = [] as { name: string; path: string; context: string; libraryName: string }[],
		activeNoteName = '',
		libraryColorMap = {} as Record<string, string>,
	}: {
		backlinks: { name: string; path: string; context: string; libraryName: string }[];
		unlinkedMentions: { name: string; path: string; context: string; libraryName: string }[];
		activeNoteName?: string;
		libraryColorMap?: Record<string, string>;
	} = $props();

	let showUnlinked = $state(false);

	function getLibraryColor(libraryName: string): string {
		return libraryColorMap[libraryName] || '#7c3aed';
	}

	async function openLink(path: string, libraryName: string, e?: MouseEvent) {
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(path, libraryName, getLibraryColor(libraryName), undefined, newTab);
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
	<div class="bl-section">
		<div class="bl-header">
			{$t('backlinksPanel.linkedMentions')}
			<span class="bl-count">{backlinks.length}</span>
		</div>
		{#if backlinks.length > 0}
			{#each backlinks as bl}
				<button class="bl-item" onclick={(e) => openLink(bl.path, bl.libraryName, e)}>
					<span class="bl-name-row">
						{#if bl.libraryName}
							<span class="bl-library-dot" style="background:{getLibraryColor(bl.libraryName)}"></span>
						{/if}
						<span class="bl-name">{bl.name}</span>
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
			<span class="bl-count">{unlinkedMentions.length}</span>
		</button>
		{#if showUnlinked && unlinkedMentions.length > 0}
			{#each unlinkedMentions as ul}
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
</style>

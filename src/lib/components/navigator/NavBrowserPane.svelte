<script lang="ts">
	import type { FileEntry } from '$lib/libraries/store';
	import { t } from '$lib/i18n';

	let {
		browseMode = 'folders' as 'folders' | 'tags' | 'properties',
		folderTree = [] as FileEntry[],
		tagMap = {} as Record<string, number>,
		selectedFolder = null as string | null,
		selectedTag = null as string | null,
		propertyKey = '',
		propertyValue = '',
		onModeChange,
		onFolderSelect,
		onTagSelect,
		onPropertySearch,
	}: {
		browseMode?: 'folders' | 'tags' | 'properties';
		folderTree?: FileEntry[];
		tagMap?: Record<string, number>;
		selectedFolder?: string | null;
		selectedTag?: string | null;
		propertyKey?: string;
		propertyValue?: string;
		onModeChange?: (mode: 'folders' | 'tags' | 'properties') => void;
		onFolderSelect?: (path: string | null) => void;
		onTagSelect?: (tag: string | null) => void;
		onPropertySearch?: (key: string, value: string) => void;
	} = $props();

	// Build tag tree from flat map
	interface TagNode { name: string; fullPath: string; count: number; children: TagNode[] }
	const tagTree = $derived.by(() => {
		const roots: TagNode[] = [];
		const sorted = Object.entries(tagMap).sort((a, b) => a[0].localeCompare(b[0]));
		const nodeMap = new Map<string, TagNode>();
		for (const [tag, count] of sorted) {
			const parts = tag.split('/');
			let path = '';
			let parent: TagNode[] = roots;
			for (let i = 0; i < parts.length; i++) {
				path += (i > 0 ? '/' : '') + parts[i];
				let existing = nodeMap.get(path);
				if (!existing) {
					existing = { name: parts[i], fullPath: path, count: i === parts.length - 1 ? count : 0, children: [] };
					nodeMap.set(path, existing);
					parent.push(existing);
				}
				if (i === parts.length - 1) existing.count = count;
				parent = existing.children;
			}
		}
		return roots;
	});

	let expandedFolders = $state(new Set<string>());
	let expandedTags = $state(new Set<string>());

	function toggleFolder(path: string) {
		const next = new Set(expandedFolders);
		if (next.has(path)) next.delete(path); else next.add(path);
		expandedFolders = next;
	}

	function toggleTag(path: string) {
		const next = new Set(expandedTags);
		if (next.has(path)) next.delete(path); else next.add(path);
		expandedTags = next;
	}
</script>

<div class="nav-browser">
	<!-- Mode tabs -->
	<div class="nav-mode-tabs">
		<button class="nav-mode-tab" class:active={browseMode === 'folders'} onclick={() => onModeChange?.('folders')} title={$t('navigator.folders') || 'Folders'}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
		</button>
		<button class="nav-mode-tab" class:active={browseMode === 'tags'} onclick={() => onModeChange?.('tags')} title={$t('navigator.tags') || 'Tags'}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 5 6.3 6.3a2.4 2.4 0 0 1 0 3.4L12 24"/><path d="M9.586 5.586A2 2 0 0 0 8.172 5H3a1 1 0 0 0-1 1v5.172a2 2 0 0 0 .586 1.414L8 18"/><circle cx="6.5" cy="9.5" r=".5" fill="currentColor"/></svg>
		</button>
		<button class="nav-mode-tab" class:active={browseMode === 'properties'} onclick={() => onModeChange?.('properties')} title={$t('navigator.properties') || 'Properties'}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z"/></svg>
		</button>
	</div>

	<div class="nav-browser-scroll">
		<!-- Folders mode -->
		{#if browseMode === 'folders'}
			<button class="nav-tree-item" class:active={selectedFolder === null} onclick={() => onFolderSelect?.(null)}>
				<span class="nav-tree-label">{$t('navigator.allNotes') || 'All notes'}</span>
			</button>
			<!-- cUniverse entries first -->
			{#each folderTree.filter(e => e.isCUniverse) as entry}
				{@render folderNode(entry, 0)}
			{/each}
			<!-- Separator if there are both cUniverses and own libraries -->
			{#if folderTree.some(e => e.isCUniverse) && folderTree.some(e => !e.isCUniverse && e.is_dir)}
				<div class="nav-tree-separator"></div>
			{/if}
			<!-- Own libraries -->
			{#each folderTree.filter(e => !e.isCUniverse) as entry}
				{#if entry.is_dir}
					{@render folderNode(entry, 0)}
				{/if}
			{/each}

		<!-- Tags mode -->
		{:else if browseMode === 'tags'}
			<button class="nav-tree-item" class:active={selectedTag === null} onclick={() => onTagSelect?.(null)}>
				<span class="nav-tree-label">{$t('navigator.allTags') || 'All tags'}</span>
			</button>
			{#each tagTree as node}
				{@render tagNode(node, 0)}
			{/each}

		<!-- Properties mode -->
		{:else}
			<div class="nav-prop-form">
				<input class="nav-prop-input" dir="auto" placeholder="Key (e.g., author)" bind:value={propertyKey} />
				<input class="nav-prop-input" dir="auto" placeholder="Value" bind:value={propertyValue} />
				<button class="nav-prop-btn" onclick={() => onPropertySearch?.(propertyKey, propertyValue)}>
					{$t('navigator.search') || 'Search'}
				</button>
			</div>
		{/if}
	</div>
</div>

{#snippet folderNode(entry: FileEntry, depth: number)}
	<div class="nav-tree-row" style="padding-inline-start: {12 + depth * 16}px">
		{#if entry.children && entry.children.some(c => c.is_dir)}
			<button class="nav-tree-chevron" onclick={() => toggleFolder(entry.path)}>
				<svg width="10" height="10" viewBox="0 0 10 10" style="transform: rotate({expandedFolders.has(entry.path) ? '90deg' : '0deg'})">
					<path d="M3 1l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
				</svg>
			</button>
		{:else}
			<span class="nav-tree-spacer"></span>
		{/if}
		<button class="nav-tree-item" class:active={selectedFolder === entry.path} class:cu-entry={entry.isCUniverse} onclick={() => onFolderSelect?.(entry.path)} dir="auto" title={entry.name}>
			{#if entry.isCUniverse}
				<svg class="nav-tree-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
					<circle cx="12" cy="12" r="3"/>
					<ellipse cx="12" cy="12" rx="10" ry="4"/>
					<ellipse cx="12" cy="12" rx="10" ry="4" transform="rotate(60 12 12)"/>
					<ellipse cx="12" cy="12" rx="10" ry="4" transform="rotate(120 12 12)"/>
				</svg>
			{/if}
			<span class="nav-tree-label">{entry.name}</span>
		</button>
	</div>
	{#if expandedFolders.has(entry.path) && entry.children}
		{#each entry.children as child}
			{#if child.is_dir}
				{@render folderNode(child, depth + 1)}
			{/if}
		{/each}
	{/if}
{/snippet}

{#snippet tagNode(node: TagNode, depth: number)}
	<div class="nav-tree-row" style="padding-inline-start: {12 + depth * 16}px">
		{#if node.children.length > 0}
			<button class="nav-tree-chevron" onclick={() => toggleTag(node.fullPath)}>
				<svg width="10" height="10" viewBox="0 0 10 10" style="transform: rotate({expandedTags.has(node.fullPath) ? '90deg' : '0deg'})">
					<path d="M3 1l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
				</svg>
			</button>
		{:else}
			<span class="nav-tree-spacer"></span>
		{/if}
		<button class="nav-tree-item" class:active={selectedTag === node.fullPath} onclick={() => onTagSelect?.(node.fullPath)} dir="auto">
			<span class="nav-tree-label">#{node.name}</span>
			{#if node.count > 0}
				<span class="nav-tree-count">{node.count}</span>
			{/if}
		</button>
	</div>
	{#if expandedTags.has(node.fullPath) && node.children.length > 0}
		{#each node.children as child}
			{@render tagNode(child, depth + 1)}
		{/each}
	{/if}
{/snippet}

<style>
	.nav-browser { display: flex; flex-direction: column; height: 100%; overflow: hidden; font-family: var(--font-interface-theme), sans-serif; }

	.nav-mode-tabs {
		display: flex; gap: 2px; padding: 6px 8px; flex-shrink: 0;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.nav-mode-tab {
		flex: 1; display: flex; align-items: center; justify-content: center;
		padding: 5px; border: none; border-radius: 4px; background: transparent;
		color: var(--text-muted); cursor: pointer;
	}
	.nav-mode-tab:hover { background: var(--background-modifier-hover); }
	.nav-mode-tab.active { background: var(--interactive-accent); color: white; }

	.nav-browser-scroll { flex: 1; overflow-y: auto; padding: 4px 0; }

	.nav-tree-row { display: flex; align-items: center; gap: 2px; min-width: 0; }
	.nav-tree-chevron {
		flex-shrink: 0; width: 16px; height: 16px; display: flex; align-items: center; justify-content: center;
		border: none; background: transparent; color: var(--text-muted); cursor: pointer; padding: 0;
	}
	.nav-tree-chevron svg { transition: transform 0.15s; }
	.nav-tree-spacer { flex-shrink: 0; width: 16px; }

	.nav-tree-item {
		flex: 1; min-width: 0; display: flex; align-items: center; gap: 6px;
		padding: 4px 8px; border: none; border-radius: 3px;
		background: transparent; color: var(--text-normal); cursor: pointer;
		font-size: 12px; text-align: start;
		font-family: var(--font-interface-theme), sans-serif;
	}
	.nav-tree-item:hover { background: var(--background-modifier-hover); }
	.nav-tree-item.active { background: color-mix(in srgb, var(--interactive-accent) 15%, transparent); color: var(--interactive-accent); }

	.nav-tree-label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.nav-tree-icon { flex-shrink: 0; color: var(--text-muted); }
	.cu-entry { color: var(--interactive-accent); font-weight: 600; }
	.cu-entry .nav-tree-label { white-space: normal; word-break: break-word; }
	.nav-tree-separator { height: 1px; margin: 4px 12px; background: var(--background-modifier-border); }
	.nav-tree-count { flex-shrink: 0; font-size: 10px; color: var(--text-faint); background: var(--background-secondary); padding: 0 5px; border-radius: 8px; }

	.nav-prop-form { display: flex; flex-direction: column; gap: 6px; padding: 8px; }
	.nav-prop-input {
		height: 28px; padding: 0 8px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; outline: none;
	}
	.nav-prop-input:focus { border-color: var(--interactive-accent); }
	.nav-prop-btn {
		padding: 6px; border: none; border-radius: 4px;
		background: var(--interactive-accent); color: white; font-size: 12px; cursor: pointer;
	}
</style>

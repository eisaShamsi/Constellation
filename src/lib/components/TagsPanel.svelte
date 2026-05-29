<script lang="ts">
	import { t } from '$lib/i18n';
	let {
		tags = {} as Record<string, number>,
		onTagClick,
	}: {
		tags: Record<string, number>;
		onTagClick: (tag: string) => void;
	} = $props();

	// Build tag tree from flat tags (e.g., "parent/child" → nested)
	interface TagNode {
		name: string;
		fullPath: string;
		count: number;
		children: TagNode[];
	}

	// Tag Browser (#12): sort mode for the tag tree — A→Z, Z→A, or by count.
	let sortMode = $state<'az' | 'za' | 'count'>('az');

	const tagTree = $derived.by(() => {
		const root: TagNode[] = [];
		// Build the tree (insertion order is irrelevant — sorted below).
		for (const [tag, count] of Object.entries(tags)) {
			const parts = tag.split('/');
			let current = root;
			let path = '';
			for (let i = 0; i < parts.length; i++) {
				path = path ? path + '/' + parts[i] : parts[i];
				let existing = current.find(n => n.name === parts[i]);
				if (!existing) {
					existing = { name: parts[i], fullPath: path, count: i === parts.length - 1 ? count : 0, children: [] };
					current.push(existing);
				} else if (i === parts.length - 1) {
					existing.count += count;
				}
				current = existing.children;
			}
		}
		// Recursively sort every level by the chosen mode. Count uses a
		// name tie-break so equal-count tags stay alphabetical.
		const sortNodes = (nodes: TagNode[]) => {
			nodes.sort((a, b) => {
				if (sortMode === 'count') return (b.count - a.count) || a.name.localeCompare(b.name);
				if (sortMode === 'za') return b.name.localeCompare(a.name);
				return a.name.localeCompare(b.name);
			});
			for (const n of nodes) sortNodes(n.children);
		};
		sortNodes(root);
		return root;
	});

	let expanded = $state<Set<string>>(new Set());
	let filterQuery = $state('');

	function toggle(path: string) {
		if (expanded.has(path)) expanded.delete(path);
		else expanded.add(path);
		expanded = new Set(expanded);
	}

	function matchesFilter(node: TagNode): boolean {
		if (!filterQuery.trim()) return true;
		const q = filterQuery.toLowerCase();
		if (node.fullPath.toLowerCase().includes(q)) return true;
		return node.children.some(c => matchesFilter(c));
	}
</script>

<div class="tags-panel">
	{#if Object.keys(tags).length > 0}
		<div class="tp-sort">
			<button class:active={sortMode === 'az'} onclick={() => sortMode = 'az'} title={$t('tagsPanel.sortAz') || 'A → Z'}>A→Z</button>
			<button class:active={sortMode === 'za'} onclick={() => sortMode = 'za'} title={$t('tagsPanel.sortZa') || 'Z → A'}>Z→A</button>
			<button class:active={sortMode === 'count'} onclick={() => sortMode = 'count'} title={$t('tagsPanel.sortCount') || 'By count'}>#</button>
		</div>
	{/if}
	{#if Object.keys(tags).length > 5}
		<div class="tp-filter">
			<input type="text" dir="auto" placeholder="Filter tags..." value={filterQuery} oninput={(e) => filterQuery = (e.target as HTMLInputElement).value} />
		</div>
	{/if}
	{#if Object.keys(tags).length === 0}
		<div class="tp-empty">{$t('tagsPanel.noTags')}</div>
	{:else}
		{#each tagTree.filter(n => matchesFilter(n)) as node}
			{@render tagNode(node, 0)}
		{/each}
	{/if}
</div>

{#snippet tagNode(node: TagNode, depth: number)}
	<div class="tp-node" style="padding-inline-start: {depth * 12 + 4}px">
		{#if node.children.length > 0}
			<button class="tp-toggle" onclick={() => toggle(node.fullPath)}>
				<svg class="tp-chev" class:expanded={expanded.has(node.fullPath)} width="8" height="8" viewBox="0 0 10 10">
					<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
				</svg>
			</button>
		{:else}
			<span class="tp-spacer"></span>
		{/if}
		<button class="tp-tag" onclick={() => onTagClick(node.fullPath)}>
			<span class="tp-hash">#</span>
			<span class="tp-name" dir="auto">{node.name}</span>
			{#if node.count > 0}
				<span class="tp-count">{node.count}</span>
			{/if}
		</button>
	</div>
	{#if expanded.has(node.fullPath)}
		{#each node.children as child}
			{@render tagNode(child, depth + 1)}
		{/each}
	{/if}
{/snippet}

<style>
	.tags-panel { font-size: 0.8rem; }
	.tp-filter { padding: 2px 4px 4px; }
	.tp-sort { display: flex; gap: 3px; padding: 4px 4px 6px; }
	.tp-sort button {
		flex: 1; padding: 3px 4px; font-size: 0.66rem; font-family: inherit;
		background: var(--bg-hover); border: 1px solid var(--border); color: var(--text-muted);
		cursor: pointer; border-radius: 4px;
	}
	.tp-sort button.active { background: var(--interactive-accent, var(--accent)); color: #fff; border-color: transparent; }
	.tp-filter input {
		width: 100%; padding: 3px 6px; border: 1px solid var(--border); border-radius: 4px;
		background: var(--bg); color: var(--text); font-size: 0.75rem; font-family: inherit; outline: none;
	}
	.tp-filter input:focus { border-color: var(--interactive-accent); }
	.tp-filter input::placeholder { color: var(--text-faint); }
	.tp-empty { color: var(--color-base-40); font-size: 0.78rem; }
	.tp-node { display: flex; align-items: center; gap: 2px; }
	.tp-toggle {
		width: 16px; height: 16px; display: flex; align-items: center; justify-content: center;
		background: none; border: none; cursor: pointer; color: var(--text-faint); flex-shrink: 0;
	}
	.tp-spacer { width: 16px; flex-shrink: 0; }
	.tp-chev { transition: transform 0.15s ease; }
	.tp-chev.expanded { transform: rotate(90deg); }
	.tp-tag {
		display: flex; align-items: center; gap: 2px;
		background: var(--tag-bg, none); border: none; cursor: pointer;
		font-family: inherit; padding: 2px 4px;
		border-radius: var(--tag-radius, 3px);
		color: var(--tag-color, var(--text-normal));
		font-size: 0.8rem;
	}
	.tp-tag:hover { background: var(--background-modifier-hover); }
	.tp-hash { color: var(--interactive-accent); font-weight: 600; }
	.tp-count {
		background: var(--background-modifier-border-focus); color: var(--text-faint); border-radius: 8px;
		padding: 0 5px; font-size: 0.68rem; margin-inline-start: 4px;
	}
</style>

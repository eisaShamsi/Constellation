<script lang="ts">
	import type { FileEntry } from '$lib/vaults/store';
	import { openNote, selectedNote } from '$lib/vaults/store';

	let { entries, depth = 0 }: { entries: FileEntry[]; depth?: number } = $props();

	function handleClick(entry: FileEntry) {
		if (!entry.is_dir) {
			openNote(entry.path);
		}
	}

	function isSelected(entry: FileEntry): boolean {
		return $selectedNote?.path === entry.path;
	}
</script>

<ul class="file-tree" style="padding-inline-start: {depth > 0 ? '1.2rem' : '0'}">
	{#each entries as entry}
		<li>
			{#if entry.is_dir}
				<details open={depth < 1}>
					<summary class="folder">
						<span class="icon">📁</span>
						<span>{entry.name}</span>
					</summary>
					{#if entry.children && entry.children.length > 0}
						<svelte:self entries={entry.children} depth={depth + 1} />
					{/if}
				</details>
			{:else}
				<button
					class="file"
					class:selected={isSelected(entry)}
					onclick={() => handleClick(entry)}
				>
					<span class="icon">📄</span>
					<span>{entry.name}</span>
				</button>
			{/if}
		</li>
	{/each}
</ul>

<style>
	.file-tree {
		list-style: none;
		margin: 0;
	}

	li { margin: 1px 0; }

	summary {
		cursor: pointer;
		padding: 0.3em 0.5em;
		border-radius: 4px;
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.9rem;
		color: #c9d1d9;
	}
	summary:hover { background: #21262d; }
	summary::marker { color: #484f58; }

	.file {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		width: 100%;
		padding: 0.3em 0.5em;
		border: none;
		background: none;
		color: #c9d1d9;
		font-size: 0.9rem;
		font-family: inherit;
		cursor: pointer;
		border-radius: 4px;
		text-align: start;
	}
	.file:hover { background: #21262d; }
	.file.selected { background: #1f2937; color: #7c3aed; }

	.icon { font-size: 0.85rem; flex-shrink: 0; }
</style>

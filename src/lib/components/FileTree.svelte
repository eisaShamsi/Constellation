<script lang="ts">
	import type { FileEntry } from '$lib/vaults/store';
	import { selectedNote } from '$lib/vaults/store';
	import { invoke } from '@tauri-apps/api/core';

	let {
		entries,
		depth = 0,
		vaultId = '',
		vaultName = '',
		onNoteClick
	}: {
		entries: FileEntry[];
		depth?: number;
		vaultId?: string;
		vaultName?: string;
		onNoteClick?: (path: string, name: string) => void;
	} = $props();

	function handleClick(entry: FileEntry) {
		if (!entry.is_dir && onNoteClick) {
			onNoteClick(entry.path, entry.name.replace('.md', ''));
		}
	}
</script>

<ul class="tree" style="padding-inline-start: {depth > 0 ? '0.9rem' : '0'}">
	{#each entries as entry}
		<li>
			{#if entry.is_dir}
				<details open={depth < 1}>
					<summary class="folder">
						<svg class="chevron" width="10" height="10" viewBox="0 0 10 10">
							<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
						</svg>
						<span class="folder-name">{entry.name}</span>
					</summary>
					{#if entry.children && entry.children.length > 0}
						<svelte:self entries={entry.children} depth={depth + 1} {vaultId} {vaultName} {onNoteClick} />
					{/if}
				</details>
			{:else}
				<button
					class="note"
					class:active={$selectedNote?.path === entry.path}
					onclick={() => handleClick(entry)}
				>
					<span class="note-name">{entry.name.replace('.md', '')}</span>
				</button>
			{/if}
		</li>
	{/each}
</ul>

<style>
	.tree {
		list-style: none;
		margin: 0;
	}

	li { margin: 0; }

	details > summary { list-style: none; }
	details > summary::-webkit-details-marker { display: none; }
	details[open] > summary .chevron { transform: rotate(90deg); }

	.folder {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		padding: 0.2rem 0.4rem;
		border-radius: 4px;
		cursor: pointer;
		color: #8b949e;
		font-size: 0.85rem;
		user-select: none;
	}
	.folder:hover { background: #1c2128; color: #c9d1d9; }

	.chevron {
		flex-shrink: 0;
		color: #484f58;
		transition: transform 0.15s ease;
	}

	.folder-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.note {
		display: block;
		width: 100%;
		padding: 0.2rem 0.4rem 0.2rem 1.4rem;
		border: none;
		background: none;
		color: #c9d1d9;
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
		border-radius: 4px;
		text-align: start;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.note:hover { background: #1c2128; }
	.note.active { background: #7c3aed22; color: #a78bfa; }

	.note-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>

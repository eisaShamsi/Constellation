<script lang="ts">
	import type { FileEntry } from '$lib/vaults/store';
	import { activeTab, splitActive, openTabs } from '$lib/vaults/store';

	let {
		entries,
		depth = 0,
		vaultId = '',
		vaultName = '',
		color = '#7c3aed',
		onNoteClick
	}: {
		entries: FileEntry[];
		depth?: number;
		vaultId?: string;
		vaultName?: string;
		color?: string;
		onNoteClick?: (path: string, name: string) => void;
	} = $props();

	function handleClick(entry: FileEntry) {
		if (!entry.is_dir && onNoteClick) {
			onNoteClick(entry.path, entry.name.replace('.md', ''));
		}
	}
</script>

<ul class="tree" style="padding-inline-start: {depth > 0 ? '12px' : '0'}">
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
						<svelte:self entries={entry.children} depth={depth + 1} {vaultId} {vaultName} {color} {onNoteClick} />
					{/if}
				</details>
			{:else}
				<button
					class="note"
					class:active={$splitActive ? $openTabs.some(t => t.path === entry.path) : $activeTab?.path === entry.path}
					style:--vault-color={color}
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
		gap: 3px;
		padding: 2px 6px;
		border-radius: 3px;
		cursor: pointer;
		color: #57606a;
		font-size: 0.82rem;
		user-select: none;
	}
	.folder:hover { background: #eaeef2; color: #24292f; }

	.chevron {
		flex-shrink: 0;
		color: #8b949e;
		transition: transform 0.15s ease;
	}

	.folder-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.note {
		display: block;
		width: 100%;
		padding: 2px 6px 2px 20px;
		border: none;
		background: none;
		color: #24292f;
		font-size: 0.82rem;
		font-family: inherit;
		cursor: pointer;
		border-radius: 3px;
		text-align: start;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.note:hover { background: #eaeef2; }
	.note.active { background: color-mix(in srgb, var(--vault-color) 8%, transparent); color: var(--vault-color); }

	.note-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>

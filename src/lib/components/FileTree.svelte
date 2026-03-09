<script lang="ts">
	import type { FileEntry } from '$lib/vaults/store';
	import { activeTab, splitActive, openTabs } from '$lib/vaults/store';

	let {
		entries,
		depth = 0,
		vaultId = '',
		vaultName = '',
		color = '#7c3aed',
		onNoteClick,
		onContextMenu,
		renamingPath = '',
		onRenameComplete
	}: {
		entries: FileEntry[];
		depth?: number;
		vaultId?: string;
		vaultName?: string;
		color?: string;
		onNoteClick?: (path: string, name: string) => void;
		onContextMenu?: (entry: FileEntry, x: number, y: number) => void;
		renamingPath?: string;
		onRenameComplete?: (oldPath: string, newName: string) => void;
	} = $props();

	function handleClick(entry: FileEntry) {
		if (!entry.is_dir && onNoteClick) {
			onNoteClick(entry.path, entry.name.replace('.md', ''));
		}
	}

	function handleRightClick(e: MouseEvent, entry: FileEntry) {
		e.preventDefault();
		onContextMenu?.(entry, e.clientX, e.clientY);
	}

	let renameValue = $state('');

	function startRename(entry: FileEntry) {
		renameValue = entry.is_dir ? entry.name : entry.name.replace('.md', '');
	}

	function finishRename(entry: FileEntry) {
		const newName = renameValue.trim();
		if (newName && newName !== (entry.is_dir ? entry.name : entry.name.replace('.md', ''))) {
			onRenameComplete?.(entry.path, newName);
		} else {
			onRenameComplete?.('', ''); // Cancel
		}
	}

	function handleRenameKeydown(e: KeyboardEvent, entry: FileEntry) {
		if (e.key === 'Enter') {
			finishRename(entry);
		} else if (e.key === 'Escape') {
			onRenameComplete?.('', ''); // Cancel
		}
	}
</script>

<ul class="tree" style="padding-inline-start: {depth > 0 ? '12px' : '0'}">
	{#each entries as entry}
		<li>
			{#if entry.is_dir}
				<details open={depth < 1}>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<summary class="folder" oncontextmenu={(e) => handleRightClick(e, entry)}>
						<svg class="chevron" width="10" height="10" viewBox="0 0 10 10">
							<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
						</svg>
						{#if renamingPath === entry.path}
							<!-- svelte-ignore a11y_autofocus -->
							<input
								class="rename-input"
								type="text"
								bind:value={renameValue}
								onblur={() => finishRename(entry)}
								onkeydown={(e) => handleRenameKeydown(e, entry)}
								onfocus={() => startRename(entry)}
								autofocus
								onclick={(e) => e.stopPropagation()}
							/>
						{:else}
							<span class="folder-name">{entry.name}</span>
						{/if}
					</summary>
					{#if entry.children && entry.children.length > 0}
						<svelte:self entries={entry.children} depth={depth + 1} {vaultId} {vaultName} {color} {onNoteClick} {onContextMenu} {renamingPath} {onRenameComplete} />
					{/if}
				</details>
			{:else}
				{#if renamingPath === entry.path}
					<div class="rename-row">
						<!-- svelte-ignore a11y_autofocus -->
						<input
							class="rename-input"
							type="text"
							bind:value={renameValue}
							onblur={() => finishRename(entry)}
							onkeydown={(e) => handleRenameKeydown(e, entry)}
							onfocus={() => startRename(entry)}
							autofocus
						/>
					</div>
				{:else}
					<button
						class="note"
						class:active={$splitActive ? $openTabs.some(t => t.path === entry.path) : $activeTab?.path === entry.path}
						style:--vault-color={color}
						onclick={() => handleClick(entry)}
						oncontextmenu={(e) => handleRightClick(e, entry)}
					>
						<span class="note-name">{entry.name.replace('.md', '')}</span>
					</button>
				{/if}
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
		color: var(--text-muted);
		font-size: 0.82rem;
		user-select: none;
	}
	.folder:hover { background: var(--background-modifier-hover); color: var(--text-normal); }

	.chevron {
		flex-shrink: 0;
		color: var(--text-faint);
		transition: transform 0.15s ease;
	}

	.folder-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.note {
		display: block;
		width: 100%;
		padding: 2px 6px 2px 20px;
		border: none;
		background: none;
		color: var(--text-normal);
		font-size: 0.82rem;
		font-family: inherit;
		cursor: pointer;
		border-radius: 3px;
		text-align: start;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.note:hover { background: var(--background-modifier-hover); }
	.note.active { background: color-mix(in srgb, var(--vault-color) 8%, transparent); color: var(--vault-color); }

	.note-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.rename-input {
		flex: 1;
		min-width: 0;
		border: 1px solid var(--interactive-accent);
		border-radius: 3px;
		padding: 1px 4px;
		font-size: 0.82rem;
		font-family: inherit;
		outline: none;
		background: var(--background-primary);
		color: var(--text-normal);
	}

	.rename-row {
		padding: 2px 6px 2px 20px;
	}
</style>

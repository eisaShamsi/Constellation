<script lang="ts">
	import type { FileEntry } from '$lib/libraries/store';
	import { activeTab, splitActive, openTabs } from '$lib/libraries/store';

	let {
		entries,
		depth = 0,
		libraryId = '',
		libraryName = '',
		color = '#7c3aed',
		onNoteClick,
		onFolderClick,
		onContextMenu,
		renamingPath = '',
		onRenameComplete,
		allExpanded = true,
		maturityMap = new Map() as Map<string, string>,
		stageMap = new Map() as Map<string, string>,
	}: {
		entries: FileEntry[];
		depth?: number;
		libraryId?: string;
		libraryName?: string;
		color?: string;
		onNoteClick?: (path: string, name: string, highlightTerm?: string, e?: MouseEvent) => void;
		onFolderClick?: (path: string) => void;
		onContextMenu?: (entry: FileEntry, x: number, y: number) => void;
		renamingPath?: string;
		onRenameComplete?: (oldPath: string, newName: string) => void;
		allExpanded?: boolean;
		maturityMap?: Map<string, string>;
		stageMap?: Map<string, string>;
	} = $props();

	const MATURITY_COLORS: Record<string, string> = {
		sapling: '#4ade80', evergreen: '#16a34a', canonical: '#f59e0b', wilting: 'rgba(22, 163, 74, 0.4)',
	};

	function handleClick(entry: FileEntry, e: MouseEvent) {
		if (!entry.is_dir && onNoteClick) {
			onNoteClick(entry.path, entry.name.replace(/\.(md|base)$/, ''), undefined, e);
		}
	}

	function handleRightClick(e: MouseEvent, entry: FileEntry) {
		e.preventDefault();
		onContextMenu?.(entry, e.clientX, e.clientY);
	}

	let renameValue = $state('');

	function startRename(entry: FileEntry) {
		renameValue = entry.is_dir ? entry.name : (entry.display_title || entry.name.replace(/\.(md|base)$/, ''));
	}

	function finishRename(entry: FileEntry) {
		const newName = renameValue.trim();
		const currentName = entry.is_dir ? entry.name : (entry.display_title || entry.name.replace(/\.(md|base)$/, ''));
		if (newName && newName !== currentName) {
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
				<details open={allExpanded && depth < 2}>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<summary class="folder" oncontextmenu={(e) => handleRightClick(e, entry)} onclick={() => onFolderClick?.(entry.path)}>
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
						<svelte:self entries={entry.children} depth={depth + 1} {libraryId} {libraryName} {color} {onNoteClick} {onFolderClick} {onContextMenu} {renamingPath} {onRenameComplete} {allExpanded} />
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
					{@const entryMat = maturityMap.get(entry.path.replace(/\\/g, '/').toLowerCase()) ?? ''}
					<button
						class="note"
						class:active={$splitActive ? $openTabs.some(t => t.path === entry.path) : $activeTab?.path === entry.path}
						class:base-file={entry.name.endsWith('.base')}
						class:mat-sapling={entryMat === 'sapling'}
						class:mat-evergreen={entryMat === 'evergreen'}
						class:mat-canonical={entryMat === 'canonical'}
						class:mat-wilting={entryMat === 'wilting'}
						style:--library-color={color}
						onclick={(e) => handleClick(entry, e)}
						oncontextmenu={(e) => handleRightClick(e, entry)}
					>
						{#if entry.name.endsWith('.base')}
							<svg class="base-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
						{/if}
						{#if stageMap.get(entry.path.replace(/\\/g, '/').toLowerCase()) === 'fleeting'}<span class="note-stage">🌱</span>{:else if stageMap.get(entry.path.replace(/\\/g, '/').toLowerCase()) === 'literature'}<span class="note-stage">📖</span>{:else if stageMap.get(entry.path.replace(/\\/g, '/').toLowerCase()) === 'permanent'}<span class="note-stage">🔗</span>{:else if stageMap.get(entry.path.replace(/\\/g, '/').toLowerCase()) === 'synthesis'}<span class="note-stage">✨</span>{/if}
						<span class="note-name">{entry.display_title || entry.name.replace(/\.(md|base)$/, '')}</span>
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
		padding: var(--ft-master-row-padding-y, 2px) 6px;
		border-radius: 3px;
		cursor: pointer;
		color: var(--ft-master-color, var(--text-muted));
		font-size: var(--ft-master-font-size, 0.82rem);
		font-weight: var(--ft-master-weight, 400);
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
		padding: var(--ft-master-row-padding-y, 2px) 6px var(--ft-master-row-padding-y, 2px) 20px;
		border: none;
		background: none;
		color: var(--ft-master-color, var(--text-normal));
		font-size: var(--ft-master-font-size, 0.82rem);
		font-weight: var(--ft-master-weight, 400);
		font-family: inherit;
		cursor: pointer;
		border-radius: 3px;
		text-align: start;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.note:hover { background: var(--background-modifier-hover); }
	.note.active { background: color-mix(in srgb, var(--library-color) 8%, transparent); color: var(--library-color); }

	.note-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.note-stage { font-size: 0.7rem; flex-shrink: 0; margin-inline-end: 2px; }
	/* CE Phase 3: Maturity left border */
	.note.mat-sapling   { border-inline-start: 3px solid #4ade80 !important; }
	.note.mat-evergreen  { border-inline-start: 3px solid #16a34a !important; }
	.note.mat-canonical  { border-inline-start: 3px solid #f59e0b !important; }
	.note.mat-wilting    { border-inline-start: 3px solid rgba(22, 163, 74, 0.4) !important; }
	.note-status { font-size: 0.75rem; flex-shrink: 0; margin-inline-end: 1px; }

	.base-file {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.base-icon {
		flex-shrink: 0;
		color: var(--interactive-accent);
		opacity: 0.7;
	}

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

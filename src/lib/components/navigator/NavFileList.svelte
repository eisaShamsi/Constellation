<script lang="ts">
	import type { NoteWithMeta } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import NavFileItem from './NavFileItem.svelte';

	let {
		files = [] as NoteWithMeta[],
		libraryColorMap = {} as Record<string, string>,
		selectedPaths = new Set<string>(),
		focusedIndex = -1,
		onNoteClick,
		onNoteDoubleClick,
		onSelectionChange,
	}: {
		files: NoteWithMeta[];
		libraryColorMap?: Record<string, string>;
		selectedPaths?: Set<string>;
		focusedIndex?: number;
		onNoteClick?: (note: NoteWithMeta) => void;
		onNoteDoubleClick?: (note: NoteWithMeta) => void;
		onSelectionChange?: (paths: Set<string>) => void;
	} = $props();

	let filterText = $state('');
	let sortBy = $state<'modified' | 'name' | 'created' | 'size'>('modified');
	let sortDir = $state<'asc' | 'desc'>('desc');

	const filtered = $derived.by(() => {
		let result = files;
		if (filterText) {
			const q = filterText.toLowerCase();
			result = result.filter(n =>
				n.name.toLowerCase().includes(q) ||
				n.preview.toLowerCase().includes(q) ||
				n.tags.some(t => t.toLowerCase().includes(q))
			);
		}
		// Sort
		const dir = sortDir === 'asc' ? 1 : -1;
		result = [...result].sort((a, b) => {
			switch (sortBy) {
				case 'name': return dir * a.name.localeCompare(b.name);
				case 'modified': return dir * (a.modified - b.modified);
				case 'created': return dir * (a.modified - b.modified); // fallback to modified
				case 'size': return dir * (a.size - b.size);
				default: return 0;
			}
		});
		return result;
	});

	function toggleSelect(path: string, checked: boolean) {
		const next = new Set(selectedPaths);
		if (checked) next.add(path); else next.delete(path);
		onSelectionChange?.(next);
	}

	function toggleSort(field: typeof sortBy) {
		if (sortBy === field) {
			sortDir = sortDir === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = field;
			sortDir = field === 'name' ? 'asc' : 'desc';
		}
	}
</script>

<div class="nav-file-list">
	<div class="nav-list-toolbar">
		<input
			class="nav-filter"
			type="text"
			dir="auto"
			placeholder={$t('navigator.filter') || 'Filter notes...'}
			bind:value={filterText}
		/>
		<div class="nav-sort-btns">
			<button class="nav-sort-btn" class:active={sortBy === 'name'} onclick={() => toggleSort('name')} title="Sort by name">
				A{sortBy === 'name' ? (sortDir === 'asc' ? '↑' : '↓') : ''}
			</button>
			<button class="nav-sort-btn" class:active={sortBy === 'modified'} onclick={() => toggleSort('modified')} title="Sort by modified">
				⏱{sortBy === 'modified' ? (sortDir === 'asc' ? '↑' : '↓') : ''}
			</button>
			<button class="nav-sort-btn" class:active={sortBy === 'size'} onclick={() => toggleSort('size')} title="Sort by size">
				##{sortBy === 'size' ? (sortDir === 'asc' ? '↑' : '↓') : ''}
			</button>
		</div>
	</div>
	<div class="nav-list-count">{filtered.length} notes</div>
	<div class="nav-list-scroll">
		{#each filtered as note, i (note.path)}
			<NavFileItem
				{note}
				color={libraryColorMap[note.libraryName ?? ''] || '#7c3aed'}
				selected={selectedPaths.has(note.path)}
				focused={i === focusedIndex}
				onSelect={toggleSelect}
				onClick={onNoteClick}
				onDoubleClick={onNoteDoubleClick}
			/>
		{/each}
		{#if filtered.length === 0}
			<div class="nav-no-results">{$t('navigator.noResults') || 'No matching notes'}</div>
		{/if}
	</div>
</div>

<style>
	.nav-file-list { display: flex; flex-direction: column; height: 100%; overflow: hidden; font-family: var(--font-interface-theme), sans-serif; }

	.nav-list-toolbar { display: flex; gap: 6px; padding: 8px; align-items: center; flex-shrink: 0; }
	.nav-filter {
		flex: 1; height: 28px; padding: 0 8px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; outline: none; font-family: inherit;
	}
	.nav-filter:focus { border-color: var(--interactive-accent); }

	.nav-sort-btns { display: flex; gap: 2px; }
	.nav-sort-btn {
		padding: 2px 6px; border: none; border-radius: 3px; background: transparent;
		color: var(--text-muted); font-size: 11px; cursor: pointer;
	}
	.nav-sort-btn:hover { background: var(--background-modifier-hover); }
	.nav-sort-btn.active { color: var(--interactive-accent); font-weight: 600; }

	.nav-list-count { padding: 0 10px 4px; font-size: 11px; color: var(--text-faint); flex-shrink: 0; }
	.nav-list-scroll { flex: 1; overflow-y: auto; padding: 0 4px; }

	.nav-no-results { padding: 20px; text-align: center; color: var(--text-faint); font-size: 13px; }
</style>

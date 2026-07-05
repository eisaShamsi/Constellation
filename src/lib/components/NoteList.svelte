<script lang="ts" generics="T">
	/**
	 * MIG-090 §4 — the shared note-LIST wrapper: composes the existing shared
	 * VirtualList (the proven in-repo virtualizer — Rule 3) with the NoteRow
	 * height contract, so hosts never guess row heights. The host supplies the
	 * row snippet (rendering NoteRow or anything row-shaped at the same height).
	 */
	import VirtualList from './VirtualList.svelte';
	import { NOTE_ROW_HEIGHT } from './NoteRow.svelte';
	import type { Snippet } from 'svelte';

	let {
		items,
		row,
		rowHeight = NOTE_ROW_HEIGHT,
		overscan = 8,
		scrollResetKey,
	}: {
		items: T[];
		row: Snippet<[T, number]>;
		rowHeight?: number;
		overscan?: number;
		scrollResetKey?: unknown;
	} = $props();
</script>

<div class="note-list">
	<VirtualList {items} getItemHeight={() => rowHeight} {row} {overscan} {scrollResetKey} />
</div>

<style>
	.note-list {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	/* VirtualList's own container fills this box. */
	.note-list :global(> *) { flex: 1; min-height: 0; }
</style>

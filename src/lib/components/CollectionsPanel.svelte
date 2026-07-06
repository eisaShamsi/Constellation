<script lang="ts">
	/**
	 * MIG-092 §4–§5 — the Collections tab inside the Search Hub.
	 *
	 * Renders one active collection: note members carry LIVE facts (re-read from
	 * the index via collections_hydrate on membership change); folder / saved-
	 * search members (unified from the former Bookmarks) render from inline
	 * facts and are never hydrated. Management: switch / create / rename / delete
	 * collections (Starred is pinned), and per-member open / done / remove /
	 * sweep-done. Membership only — nothing here writes note content.
	 */
	import { t, dir } from '$lib/i18n';
	import { untrack, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import {
		collectionSets, createCollection, renameCollection, deleteCollection,
		removeFromCollection, hydrateCollectionNotes, STARRED_ID,
	} from '$lib/libraries/store';
	import { buildDisplayRows, collectionKey, type CollectionDisplayRow, type HydratedNoteRow } from '$lib/libraries/collectionsLogic';
	import { filterByChips, type ChipToggles } from './collectionChips';
	import NoteList from './NoteList.svelte';
	import NoteRow from './NoteRow.svelte';
	import { detectDir } from '$lib/utils';

	let {
		onNoteClick = (_path: string, _name: string, _libraryName: string) => {},
		onRunSearch = (_query: string) => {},
		onRevealPath = (_path: string) => {},
	}: {
		onNoteClick?: (path: string, name: string, libraryName: string) => void;
		onRunSearch?: (query: string) => void;
		onRevealPath?: (path: string) => void;
	} = $props();

	/** Locale lookup with an English fallback while non-EN keys land at close-out. */
	const L = (key: string, fb: string): string => {
		const v = $t(key);
		return v === key ? fb : v;
	};

	let activeId = $state(STARRED_ID);
	let hydratedRows = $state<HydratedNoteRow[]>([]);
	let renaming = $state(false);
	let draft = $state('');
	let creating = $state(false);

	const sets = $derived($collectionSets);
	const active = $derived(sets.find(c => c.id === activeId) ?? sets[0]);
	// Selected collection was deleted → fall back to the first (Starred).
	$effect(() => {
		if (sets.length > 0 && !sets.some(c => c.id === activeId)) activeId = sets[0].id;
	});

	const items = $derived(active?.items ?? []);
	// Re-hydrate only when the membership KEY SET changes (add/remove/cid-adopt/
	// switch) — never on a mere done-toggle. Snapshot read is untracked so the
	// effect depends on the signature + activeId only (Rule 2/3: no loop, no
	// per-toggle IPC).
	const keySig = $derived(items.map(i => i.cid || i.path).join('|'));
	$effect(() => {
		keySig;
		activeId;
		const snapshot = untrack(() => items);
		hydrateCollectionNotes(snapshot).then(rows => { hydratedRows = rows; });
	});

	// §8 — liveness: when notes change elsewhere (edit / rename cascade / cross-
	// window save / cache reconcile), re-read the active collection's facts. One
	// debounced re-hydrate on the EXISTING mutation events (membership unchanged
	// → the display-only stale-snapshot cure). Listeners unlistened on destroy.
	let rehydrateTimer: ReturnType<typeof setTimeout> | null = null;
	function scheduleRehydrate() {
		if (rehydrateTimer) clearTimeout(rehydrateTimer);
		rehydrateTimer = setTimeout(() => {
			hydrateCollectionNotes(items).then(rows => { hydratedRows = rows; });
		}, 500);
	}
	onMount(() => {
		const events = ['note-created', 'cascade:rewrote', 'cache-reconciled', 'screen:note-saved'];
		const unlistens: UnlistenFn[] = [];
		let alive = true;
		for (const ev of events) {
			listen(ev, scheduleRehydrate).then(un => { if (alive) unlistens.push(un); else un(); });
		}
		return () => {
			alive = false;
			if (rehydrateTimer) clearTimeout(rehydrateTimer);
			for (const un of unlistens) un();
		};
	});

	const displayRows = $derived(buildDisplayRows(items, hydratedRows));
	// §7 — four state chips NARROW the shown members (pure client-side
	// intersection over the one hydration read — zero IPC; folder/search members
	// carry no state facts, so any active chip drops them).
	let chips = $state<ChipToggles>({ due: false, unlinked: false, contested: false, forming: false });
	const filteredRows = $derived(filterByChips(displayRows, d => d.hydrated, chips));
	const label = (c: { id: string; name: string }) => (c.id === STARRED_ID ? L('collections.starred', 'Starred') : c.name);

	function memberMeta(d: CollectionDisplayRow): string {
		if (d.type === 'folder') return L('collections.typeFolder', 'Folder');
		if (d.type === 'search') return L('collections.typeSearch', 'Search');
		if (d.missing) return L('collections.missing', 'missing');
		return d.libraryName;
	}

	function activate(d: CollectionDisplayRow) {
		if (d.type === 'search') { onRunSearch(d.item.path); return; }
		if (d.type === 'folder') { onRevealPath(d.item.path); return; }
		const path = d.hydrated?.path ?? d.item.path;
		onNoteClick(path, d.name, d.libraryName);
	}

	function startCreate() { creating = true; renaming = false; draft = ''; }
	function commitCreate() {
		const name = draft.trim();
		if (name) activeId = createCollection(name);
		creating = false;
		draft = '';
	}
	function startRename() {
		if (!active || active.id === STARRED_ID) return;
		renaming = true; creating = false; draft = active.name;
	}
	function commitRename() {
		if (active && draft.trim()) renameCollection(active.id, draft.trim());
		renaming = false;
	}
	function doDelete() {
		if (active && active.id !== STARRED_ID) deleteCollection(active.id);
	}
	function onDraftKey(e: KeyboardEvent, commit: () => void) {
		if (e.key === 'Enter') { e.preventDefault(); commit(); }
		else if (e.key === 'Escape') { e.preventDefault(); creating = false; renaming = false; }
	}
</script>

<div class="cp-root" dir={$dir}>
	<div class="cp-head">
		{#if renaming}
			<!-- svelte-ignore a11y_autofocus -->
			<input class="cp-input" bind:value={draft} dir="auto" autofocus
				onkeydown={(e) => onDraftKey(e, commitRename)} onblur={commitRename} />
		{:else if creating}
			<!-- svelte-ignore a11y_autofocus -->
			<input class="cp-input" bind:value={draft} dir="auto" autofocus
				placeholder={L('collections.newPlaceholder', 'Collection name…')}
				onkeydown={(e) => onDraftKey(e, commitCreate)} onblur={commitCreate} />
		{:else}
			<select class="cp-select" bind:value={activeId}>
				{#each sets as c}
					<option value={c.id}>{label(c)} ({c.items.length})</option>
				{/each}
			</select>
			<button class="cp-btn" title={L('collections.newCollection', 'New collection')} onclick={startCreate}>+ {L('collections.new', 'New')}</button>
			{#if active && active.id !== STARRED_ID}
				<button class="cp-btn" title={L('collections.rename', 'Rename')} onclick={startRename} aria-label={L('collections.rename', 'Rename')}>✎</button>
				<button class="cp-btn cp-btn-danger" title={L('collections.delete', 'Delete')} onclick={doDelete} aria-label={L('collections.delete', 'Delete')}>🗑</button>
			{/if}
		{/if}
	</div>

	{#if !active || items.length === 0}
		<div class="cp-empty">
			<div class="cp-empty-title">{L('collections.empty', 'This collection is empty')}</div>
			<div class="cp-empty-hint">{L('collections.emptyHint', 'Search, then add results to a collection to keep them.')}</div>
		</div>
	{:else}
		<div class="cp-chips">
			<button class="cp-chip" class:on={chips.due} onclick={() => (chips = { ...chips, due: !chips.due })}>{L('collections.chipDue', 'Due')}</button>
			<button class="cp-chip" class:on={chips.unlinked} onclick={() => (chips = { ...chips, unlinked: !chips.unlinked })}>{L('collections.chipUnlinked', 'Unlinked')}</button>
			<button class="cp-chip" class:on={chips.contested} onclick={() => (chips = { ...chips, contested: !chips.contested })}>{L('collections.chipContested', 'Contested')}</button>
			<button class="cp-chip" class:on={chips.forming} onclick={() => (chips = { ...chips, forming: !chips.forming })}>{L('collections.chipForming', 'Forming')}</button>
		</div>
		{#if filteredRows.length === 0}
			<div class="cp-empty">
				<div class="cp-empty-hint">{L('collections.noneMatch', 'No members match the active filters.')}</div>
			</div>
		{:else}
			<NoteList items={filteredRows} scrollResetKey={activeId} row={memberRow} />
		{/if}
	{/if}
</div>

{#snippet memberRow(d: CollectionDisplayRow)}
	<NoteRow
		name={d.name}
		meta={memberMeta(d)}
		missing={d.missing}
		onActivate={() => activate(d)}
	>
		{#snippet trailing()}
			<button class="cp-act" title={L('collections.remove', 'Remove')} aria-label={L('collections.remove', 'Remove')}
				onclick={(e) => { e.stopPropagation(); active && removeFromCollection(active.id, d.key); }}>×</button>
		{/snippet}
	</NoteRow>
{/snippet}

<style>
	.cp-root {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.cp-head {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
		flex-wrap: wrap;
	}
	.cp-select {
		flex: 0 1 auto;
		max-width: 260px;
		padding: 4px 8px;
		border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-primary);
		color: var(--text-normal);
		font-family: var(--font-interface-theme);
		font-size: 13px;
	}
	.cp-input {
		flex: 1;
		min-width: 120px;
		padding: 4px 8px;
		border-radius: 6px;
		border: 1px solid var(--interactive-accent);
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 13px;
	}
	.cp-btn {
		padding: 4px 10px;
		border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		color: var(--text-normal);
		cursor: pointer;
		font-size: 12px;
	}
	.cp-btn:hover { background: var(--background-modifier-hover); }
	.cp-btn-danger:hover { color: var(--text-error); }
	.cp-act {
		width: 22px;
		height: 22px;
		border-radius: 5px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 13px;
		line-height: 1;
	}
	.cp-act:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.cp-chips {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		padding: 6px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.cp-chip {
		padding: 3px 10px;
		border-radius: 12px;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		color: var(--text-muted);
		cursor: pointer;
		font-size: 11px;
	}
	.cp-chip:hover { color: var(--text-normal); }
	.cp-chip.on {
		background: color-mix(in srgb, var(--interactive-accent) 16%, transparent);
		border-color: var(--interactive-accent);
		color: var(--interactive-accent);
	}
	.cp-empty {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 32px;
		text-align: center;
	}
	.cp-empty-title { font-size: 14px; color: var(--text-normal); }
	.cp-empty-hint { font-size: 12px; color: var(--text-muted); max-width: 320px; }
</style>

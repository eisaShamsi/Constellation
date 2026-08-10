<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import type { LibraryInfo } from '$lib/libraries/store';
	import LibraryIcon from '$lib/components/LibraryIcon.svelte';

	/**
	 * Boss finding 2026-08-10 — `items` is REQUIRED and must be the OWN-universe libraries.
	 *
	 * This component used to read the `libraries` store directly, which is the FEDERATED list:
	 * Ctrl+N offered all 25 libraries including a linked universe's (Architecture, Film,
	 * Philosophy… belong to Eisa Cognitive Knowledge), and picking one created a note INSIDE
	 * that other universe. The sidebar has always filtered them out (`ownLibraries` →
	 * `isChildUniverseLib`); only this picker had drifted. Reading a universe-wide list is
	 * right for RESOLVING a name and wrong for CHOOSING somewhere to write — the caller owns
	 * that distinction, so it passes the list rather than this component guessing.
	 */
	let {
		items,
		colorMap,
		onSelect,
		onClose,
	}: {
		items: LibraryInfo[];
		colorMap: Record<string, string>;
		onSelect: (library: LibraryInfo) => void;
		onClose: () => void;
	} = $props();

	let overlayEl: HTMLDivElement;

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onClose(); }
	}

	function handleOverlayClick(e: MouseEvent) {
		if (e.target === overlayEl) onClose();
	}

	onMount(() => {
		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="vp-overlay" bind:this={overlayEl} onclick={handleOverlayClick}>
	<div class="vp-modal">
		<div class="vp-title">{$t('notePane.selectLibrary')}</div>
		<div class="vp-list">
			{#each items.filter(l => l.is_universe_notes) as lib}
				<button class="vp-item" onclick={() => { onSelect(lib); onClose(); }}>
					<!-- Universe root: no icon (Boss 2026-07-25). -->
					<span class="vp-name">{lib.name}</span>
				</button>
			{/each}
			{#each items.filter(l => !l.is_universe_notes) as lib}
				<button class="vp-item" onclick={() => { onSelect(lib); onClose(); }}>
					<LibraryIcon kind="library" size={12} color={colorMap[lib.name] || '#7c3aed'} />
					<span class="vp-name">{lib.name}</span>
				</button>
			{/each}
		</div>
		<button class="vp-cancel" onclick={onClose}>{$t('notePane.close')}</button>
	</div>
</div>

<style>
	.vp-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, var(--modal-overlay-alpha, 0.4)); /* MIG-088 §4c — shared dialog-backdrop opacity */
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}
	.vp-modal {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 10px;
		box-shadow: var(--modal-shadow, 0 8px 32px rgba(0, 0, 0, 0.2));
		width: 280px;
		max-height: 60vh;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
	.vp-title {
		padding: 14px 16px 10px;
		font-size: 0.9rem;
		font-weight: 600;
		color: var(--text-normal);
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.vp-list {
		overflow-y: auto;
		padding: 4px 0;
	}
	.vp-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 16px;
		background: none;
		border: none;
		color: var(--text-normal);
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
	}
	.vp-item:hover {
		background: var(--background-modifier-hover);
	}
	.vp-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.vp-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.vp-cancel {
		padding: 10px 16px;
		background: none;
		border: none;
		border-top: 1px solid var(--background-modifier-border);
		color: var(--text-muted);
		font-size: 0.82rem;
		font-family: inherit;
		cursor: pointer;
		text-align: center;
	}
	.vp-cancel:hover {
		background: var(--background-modifier-hover);
	}
</style>

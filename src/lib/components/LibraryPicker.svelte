<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { libraries } from '$lib/libraries/store';
	import type { LibraryInfo } from '$lib/libraries/store';

	let {
		colorMap,
		onSelect,
		onClose,
	}: {
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
		<div class="vp-title">{$t('notePane.selectVault')}</div>
		<div class="vp-list">
			{#each $libraries as vault}
				<button class="vp-item" onclick={() => { onSelect(vault); onClose(); }}>
					<span class="vp-dot" style="background: {colorMap[vault.name] || '#7c3aed'}"></span>
					<span class="vp-name">{vault.name}</span>
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
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}
	.vp-modal {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 10px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
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

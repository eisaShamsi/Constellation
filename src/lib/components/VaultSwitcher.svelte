<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { vaultStats } from '$lib/vaults/store';
	import type { VaultStats } from '$lib/vaults/store';

	let {
		colorMap,
		onClose,
		onAddVault,
		onManage,
	}: {
		colorMap: Record<string, string>;
		onClose: () => void;
		onAddVault: () => void;
		onManage: () => void;
	} = $props();

	let popupEl: HTMLDivElement;

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onClose(); }
	}

	function handleClickOutside(e: MouseEvent) {
		if (popupEl && !popupEl.contains(e.target as Node)) onClose();
	}

	onMount(() => {
		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);
		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="vault-switcher" bind:this={popupEl}>
	<button class="vs-item vs-add" onclick={() => { onClose(); onAddVault(); }}>
		<span>{$t('vaultManager.openVault')}</span>
	</button>

	<div class="vs-divider"></div>

	{#if $vaultStats.length === 0}
		<div class="vs-empty">{$t('vaultManager.noVaults')}</div>
	{:else}
		<div class="vs-list">
			{#each $vaultStats as vault}
				<div class="vs-item vs-vault">
					<span class="vs-dot" style="background: {colorMap[vault.name] || '#7c3aed'}"></span>
					<span class="vs-name">{vault.name}</span>
					<span class="vs-count">{vault.star_count}</span>
				</div>
			{/each}
		</div>
	{/if}

	<div class="vs-divider"></div>

	<button class="vs-item vs-manage" onclick={() => { onClose(); onManage(); }}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<path d="M12 3h7a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7"/>
			<path d="M16 3v4"/><path d="M8 3v4"/>
		</svg>
		<span>{$t('vaultManager.manageVaults')}</span>
	</button>
</div>

<style>
	.vault-switcher {
		position: absolute;
		bottom: 100%;
		inset-inline-start: 0;
		inset-inline-end: 0;
		margin-bottom: 4px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
		z-index: 100;
		overflow: hidden;
	}
	.vs-list {
		max-height: 300px;
		overflow-y: auto;
	}
	.vs-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 14px;
		background: none;
		border: none;
		color: var(--text-normal);
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
	}
	.vs-item:hover {
		background: var(--background-modifier-hover);
	}
	.vs-add {
		color: var(--interactive-accent);
		font-weight: 500;
	}
	.vs-manage {
		color: var(--text-muted);
		font-size: 0.82rem;
	}
	.vs-vault {
		cursor: default;
	}
	.vs-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.vs-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.vs-count {
		font-size: 0.75rem;
		color: var(--text-faint);
	}
	.vs-divider {
		height: 1px;
		background: var(--background-modifier-border);
	}
	.vs-empty {
		padding: 12px 14px;
		color: var(--text-faint);
		font-size: 0.82rem;
		text-align: center;
	}
</style>

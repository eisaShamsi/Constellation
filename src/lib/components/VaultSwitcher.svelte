<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { vaultStats, loadVaults, loadAllStats } from '$lib/vaults/store';
	import { listUniverses, addChildUniverse, getChildUniverses } from '$lib/universe/store';
	import type { ChildUniverseInfo } from '$lib/universe/store';

	let {
		colorMap,
		onClose,
		onAddVault,
		onManage,
		onManageUniverse,
		onChildUniverseChanged,
		activeUniverseName = '',
	}: {
		colorMap: Record<string, string>;
		onClose: () => void;
		onAddVault: () => void;
		onManage: () => void;
		onManageUniverse: () => void;
		onChildUniverseChanged?: () => void;
		activeUniverseName?: string;
	} = $props();

	async function handleAddChildUniverse() {
		try {
			const result = await invoke<string | null>('pick_folder');
			if (result) {
				await addChildUniverse(result);
				childUniverses = await getChildUniverses();
				onClose();
				onChildUniverseChanged?.();
			}
		} catch { /* cancelled or error */ }
	}

	let popupEl: HTMLDivElement;
	let universeCount = $state(0);
	let childUniverses: ChildUniverseInfo[] = $state([]);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); onClose(); }
	}

	function handleClickOutside(e: MouseEvent) {
		if (popupEl && !popupEl.contains(e.target as Node)) onClose();
	}

	onMount(async () => {
		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);

		try {
			const universes = await listUniverses();
			universeCount = universes.length;
			childUniverses = await getChildUniverses();
		} catch { /* ignore */ }

		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="vault-switcher" bind:this={popupEl}>
	<!-- Universe section -->
	<div class="vs-section-header">{$t('universe.title') ?? 'Universe'}</div>
	<div class="vs-item vs-universe-info">
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<circle cx="12" cy="12" r="10"/>
			<path d="M2 12h20"/>
			<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
		</svg>
		<span class="vs-name">{activeUniverseName || 'Universe'}</span>
		{#if universeCount > 1}
			<span class="vs-count">{universeCount}</span>
		{/if}
	</div>
	<button class="vs-item vs-action" onclick={() => { onClose(); onManageUniverse(); }}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
			<circle cx="12" cy="12" r="3"/>
		</svg>
		<span>{$t('universe.manager.heading') ?? 'Universe Manager'}</span>
	</button>
	<button class="vs-item vs-action" onclick={handleAddChildUniverse}>
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/>
		</svg>
		<span>{$t('universe.manager.addChild') ?? 'Add Child Universe'}</span>
	</button>

	<div class="vs-divider"></div>

	<!-- Own Vaults section -->
	<div class="vs-section-header">{$t('universe.manager.ownVaults') ?? 'Own Vaults'}</div>
	<button class="vs-item vs-add" onclick={() => { onClose(); onAddVault(); }}>
		<span>{$t('vaultManager.openVault')}</span>
	</button>

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

	<!-- Child Universes section (only if there are children) -->
	{#if childUniverses.length > 0}
		<div class="vs-divider"></div>
		<div class="vs-section-header">{$t('universe.manager.children') ?? 'Child Universes'}</div>
		<div class="vs-list">
			{#each childUniverses as child}
				<div class="vs-item vs-child-universe">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink:0; color: var(--text-muted)">
						<circle cx="12" cy="12" r="10"/>
						<path d="M2 12h20"/>
						<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
					</svg>
					<span class="vs-name">{child.name}</span>
					<span class="vs-count">{child.vault_count} {child.vault_count === 1 ? 'vault' : 'vaults'}</span>
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
	.vs-section-header {
		padding: 6px 14px 2px;
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.04em;
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
	.vs-action {
		color: var(--text-muted);
		font-size: 0.82rem;
	}
	.vs-action:hover {
		color: var(--text-normal);
	}
	.vs-manage {
		color: var(--text-muted);
		font-size: 0.82rem;
	}
	.vs-vault {
		cursor: default;
	}
	.vs-child-universe {
		cursor: default;
		font-size: 0.82rem;
	}
	.vs-universe-info {
		cursor: default;
		color: var(--text-normal);
		font-weight: 500;
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

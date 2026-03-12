<script lang="ts">
	import { t } from '$lib/i18n';
	import { vaultStats, addVault, removeVaultWithCleanup } from '$lib/vaults/store';
	import type { VaultStats } from '$lib/vaults/store';
	import { invoke } from '@tauri-apps/api/core';

	let {
		colorMap,
		onClose,
		onRefresh,
	}: {
		colorMap: Record<string, string>;
		onClose: () => void;
		onRefresh: () => void;
	} = $props();

	let confirmingRemove = $state<VaultStats | null>(null);
	let adding = $state(false);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			if (confirmingRemove) { confirmingRemove = null; }
			else { onClose(); }
		}
	}

	async function handleAddVault() {
		adding = true;
		try {
			await addVault();
			onRefresh();
		} catch { /* ignore */ }
		adding = false;
	}

	async function handleRemove(vault: VaultStats) {
		confirmingRemove = null;
		await removeVaultWithCleanup(vault.vault_id);
		onRefresh();
	}

	async function handleOpenFolder(path: string) {
		try {
			await invoke('plugin:opener|open_path', { path });
		} catch {
			// Fallback: try shell open
			try { await invoke('open_folder', { path }); } catch { /* ignore */ }
		}
	}

	function truncatePath(path: string, max = 45): string {
		if (path.length <= max) return path;
		const parts = path.replace(/\\/g, '/').split('/');
		if (parts.length <= 3) return '...' + path.slice(-(max - 3));
		return parts[0] + '/.../' + parts.slice(-2).join('/');
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="vm-overlay" onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="vm-backdrop" onclick={onClose}></div>
	<div class="vm-modal" role="dialog" aria-label={$t('vaultManager.title')}>
		<div class="vm-header">
			<h2>{$t('vaultManager.title')}</h2>
			<button class="vm-close" onclick={onClose}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
			</button>
		</div>

		<div class="vm-body">
			{#if $vaultStats.length === 0}
				<div class="vm-empty">{$t('vaultManager.noVaults')}</div>
			{:else}
				{#each $vaultStats as vault}
					<div class="vm-vault">
						<div class="vm-vault-info">
							<span class="vm-dot" style="background: {colorMap[vault.name] || '#7c3aed'}"></span>
							<div class="vm-vault-text">
								<div class="vm-vault-name">{vault.name}</div>
								<div class="vm-vault-path" title={vault.path}>{truncatePath(vault.path)}</div>
							</div>
							<span class="vm-vault-count">{vault.star_count} {$t('vaultManager.notes')}</span>
						</div>
						<div class="vm-vault-actions">
							<button class="vm-action" title={$t('vaultManager.openFolder')} onclick={() => handleOpenFolder(vault.path)}>
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>
							</button>
							<button class="vm-action vm-danger" title={$t('vaultManager.remove')} onclick={() => confirmingRemove = vault}>
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<div class="vm-footer">
			<button class="vm-add-btn" onclick={handleAddVault} disabled={adding}>
				{$t('vaultManager.addVault')}
			</button>
		</div>
	</div>

	{#if confirmingRemove}
		<div class="vm-confirm-overlay">
			<div class="vm-confirm">
				<p>{$t('vaultManager.confirmRemove', { name: confirmingRemove.name })}</p>
				<div class="vm-confirm-actions">
					<button class="vm-btn-cancel" onclick={() => confirmingRemove = null}>{$t('common.cancel')}</button>
					<button class="vm-btn-remove" onclick={() => handleRemove(confirmingRemove!)}>{$t('vaultManager.remove')}</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.vm-overlay {
		position: fixed; inset: 0; z-index: 1000;
		display: flex; align-items: center; justify-content: center;
	}
	.vm-backdrop {
		position: absolute; inset: 0;
		background: rgba(0, 0, 0, 0.4);
	}
	.vm-modal {
		position: relative;
		width: 90vw; max-width: 520px;
		max-height: 70vh;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 12px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
		display: flex; flex-direction: column;
		overflow: hidden;
	}
	.vm-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.vm-header h2 {
		margin: 0; font-size: 1.1rem; font-weight: 700;
		color: var(--text-normal);
	}
	.vm-close {
		width: 28px; height: 28px;
		display: flex; align-items: center; justify-content: center;
		background: none; border: none; border-radius: 4px;
		color: var(--text-muted); cursor: pointer;
	}
	.vm-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }

	.vm-body {
		flex: 1; overflow-y: auto; padding: 8px 0;
	}
	.vm-empty {
		padding: 32px 20px; text-align: center;
		color: var(--text-faint); font-size: 0.88rem;
	}
	.vm-vault {
		display: flex; align-items: center; justify-content: space-between;
		padding: 10px 20px; gap: 12px;
	}
	.vm-vault:hover { background: var(--background-modifier-hover); }
	.vm-vault-info {
		display: flex; align-items: center; gap: 10px;
		flex: 1; min-width: 0;
	}
	.vm-dot {
		width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0;
	}
	.vm-vault-text {
		flex: 1; min-width: 0;
	}
	.vm-vault-name {
		font-size: 0.9rem; font-weight: 600; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.vm-vault-path {
		font-size: 0.75rem; color: var(--text-faint);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.vm-vault-count {
		font-size: 0.75rem; color: var(--text-muted);
		flex-shrink: 0; white-space: nowrap;
	}
	.vm-vault-actions {
		display: flex; gap: 4px; flex-shrink: 0;
	}
	.vm-action {
		width: 28px; height: 28px;
		display: flex; align-items: center; justify-content: center;
		background: none; border: none; border-radius: 4px;
		color: var(--text-muted); cursor: pointer;
	}
	.vm-action:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.vm-danger:hover { color: var(--text-error); }

	.vm-footer {
		padding: 12px 20px;
		border-top: 1px solid var(--background-modifier-border);
	}
	.vm-add-btn {
		width: 100%; padding: 8px;
		background: none;
		border: 1px dashed var(--background-modifier-border);
		border-radius: 6px;
		color: var(--text-muted);
		font-size: 0.85rem; font-family: inherit;
		cursor: pointer;
	}
	.vm-add-btn:hover { border-color: var(--interactive-accent); color: var(--interactive-accent); }
	.vm-add-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	/* Confirm dialog */
	.vm-confirm-overlay {
		position: absolute; inset: 0;
		background: rgba(0, 0, 0, 0.3);
		display: flex; align-items: center; justify-content: center;
		z-index: 10;
	}
	.vm-confirm {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 20px;
		max-width: 360px; width: 90%;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
	}
	.vm-confirm p {
		margin: 0 0 16px; font-size: 0.9rem;
		color: var(--text-normal); line-height: 1.5;
	}
	.vm-confirm-actions {
		display: flex; justify-content: flex-end; gap: 8px;
	}
	.vm-btn-cancel {
		padding: 6px 14px; background: var(--background-modifier-border);
		border: none; border-radius: 4px;
		color: var(--text-muted); font-size: 0.82rem;
		cursor: pointer; font-family: inherit;
	}
	.vm-btn-cancel:hover { opacity: 0.85; }
	.vm-btn-remove {
		padding: 6px 14px; background: var(--text-error);
		border: none; border-radius: 4px;
		color: white; font-size: 0.82rem;
		cursor: pointer; font-family: inherit;
	}
	.vm-btn-remove:hover { opacity: 0.9; }
</style>

<script lang="ts">
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import {
		vaults, selectedVault, vaultTree, selectedNote,
		loadVaults, addVault, removeVault, selectVault
	} from '$lib/vaults/store';
	import FileTree from '$lib/components/FileTree.svelte';

	let error = $state('');
	let adding = $state(false);

	onMount(() => { loadVaults(); });

	async function handleAddVault() {
		adding = true;
		error = '';
		try {
			await addVault();
		} catch (e) {
			error = String(e);
		}
		adding = false;
	}

	async function handleRemove(id: string) {
		try {
			await removeVault(id);
		} catch (e) {
			error = String(e);
		}
	}
</script>

<div class="vaults-page">
	<div class="header">
		<h1>{$locale === 'ar' ? 'الخزائن' : 'Universes'}</h1>
		<button class="add-btn" onclick={handleAddVault} disabled={adding}>
			{adding
				? ($locale === 'ar' ? 'جارٍ الإضافة...' : 'Adding...')
				: ($locale === 'ar' ? '+ إضافة خزينة' : '+ Add Vault')}
		</button>
	</div>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	{#if $vaults.length === 0}
		<div class="empty">
			<p>{$locale === 'ar' ? 'لم تتم إضافة أي خزائن بعد.' : 'No vaults added yet.'}</p>
			<p class="hint">
				{$locale === 'ar'
					? 'انقر على "+ إضافة خزينة" لربط مجلد خزينة أوبسيديان.'
					: 'Click "+ Add Vault" to link an Obsidian vault folder.'}
			</p>
		</div>
	{:else}
		<div class="layout">
			<!-- Vault Sidebar -->
			<div class="vault-sidebar">
				{#each $vaults as vault}
					<div
						class="vault-card"
						class:active={$selectedVault?.id === vault.id}
					>
						<button class="vault-btn" onclick={() => selectVault(vault)}>
							<span class="vault-icon">🌌</span>
							<div class="vault-info">
								<div class="vault-name">{vault.name}</div>
								<div class="vault-path">{vault.path}</div>
							</div>
						</button>
						<button
							class="remove-btn"
							onclick={() => handleRemove(vault.id)}
							title={$locale === 'ar' ? 'إزالة' : 'Remove'}
						>×</button>
					</div>
				{/each}
			</div>

			<!-- File Browser -->
			<div class="browser">
				{#if $selectedVault}
					<div class="browser-header">
						<h2>{$selectedVault.name}</h2>
					</div>

					<div class="browser-content">
						<!-- File Tree -->
						<div class="tree-panel">
							{#if $vaultTree.length > 0}
								<FileTree entries={$vaultTree} />
							{:else}
								<p class="muted">
									{$locale === 'ar' ? 'لا توجد ملاحظات في هذه الخزينة.' : 'No notes in this vault.'}
								</p>
							{/if}
						</div>

						<!-- Note Preview -->
						<div class="preview-panel">
							{#if $selectedNote}
								<div class="note-header">
									<span>{$selectedNote.path.split(/[\\/]/).pop()}</span>
								</div>
								<pre class="note-content">{$selectedNote.content}</pre>
							{:else}
								<p class="muted center">
									{$locale === 'ar'
										? 'اختر ملاحظة لعرضها'
										: 'Select a note to preview'}
								</p>
							{/if}
						</div>
					</div>
				{:else}
					<p class="muted center">
						{$locale === 'ar'
							? 'اختر خزينة من القائمة'
							: 'Select a universe from the sidebar'}
					</p>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.vaults-page { max-width: 100%; }

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
	}
	h1 { font-size: 1.8rem; margin: 0; }

	.add-btn {
		background: #7c3aed;
		border: none;
		color: white;
		padding: 0.6em 1.2em;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.9rem;
		font-weight: 600;
		transition: background 0.2s;
	}
	.add-btn:hover { background: #6d28d9; }
	.add-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.error {
		background: #1c1917;
		border: 1px solid #991b1b;
		color: #f85149;
		padding: 0.75rem 1rem;
		border-radius: 8px;
		margin-bottom: 1rem;
		font-size: 0.9rem;
	}

	.empty {
		text-align: center;
		padding: 4rem 2rem;
		color: #8b949e;
	}
	.hint { font-size: 0.9rem; margin-top: 0.5rem; }

	.layout {
		display: flex;
		gap: 1rem;
		min-height: 70vh;
	}

	.vault-sidebar {
		flex: 0 0 240px;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.vault-card {
		display: flex;
		align-items: center;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 8px;
		transition: border-color 0.2s;
	}
	.vault-card:hover { border-color: #30363d; }
	.vault-card.active { border-color: #7c3aed; }

	.vault-btn {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 0.6rem;
		padding: 0.7rem;
		background: none;
		border: none;
		color: #e0e0e0;
		cursor: pointer;
		text-align: start;
		font-family: inherit;
	}

	.vault-icon { font-size: 1.3rem; }
	.vault-name { font-weight: 600; font-size: 0.9rem; }
	.vault-path {
		font-size: 0.7rem;
		color: #8b949e;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 160px;
	}

	.remove-btn {
		background: none;
		border: none;
		color: #484f58;
		font-size: 1.2rem;
		cursor: pointer;
		padding: 0.5rem;
		line-height: 1;
	}
	.remove-btn:hover { color: #f85149; }

	.browser {
		flex: 1;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 8px;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.browser-header {
		padding: 0.75rem 1rem;
		border-bottom: 1px solid #21262d;
	}
	.browser-header h2 { margin: 0; font-size: 1.1rem; }

	.browser-content {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.tree-panel {
		flex: 0 0 280px;
		border-inline-end: 1px solid #21262d;
		padding: 0.5rem;
		overflow-y: auto;
	}

	.preview-panel {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	.note-header {
		padding: 0.5rem 1rem;
		border-bottom: 1px solid #21262d;
		font-size: 0.85rem;
		color: #8b949e;
	}

	.note-content {
		padding: 1rem;
		margin: 0;
		font-size: 0.9rem;
		white-space: pre-wrap;
		word-break: break-word;
		color: #c9d1d9;
		line-height: 1.6;
		flex: 1;
	}

	.muted { color: #484f58; font-size: 0.9rem; padding: 1rem; }
	.center { text-align: center; padding-top: 4rem; }
</style>

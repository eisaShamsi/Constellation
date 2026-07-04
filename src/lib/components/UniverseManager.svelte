<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import {
		listUniverses, setActiveUniverse, removeUniverseFromRegistry,
		createUniverse, addChildUniverse, removeChildUniverse,
		openExistingUniverse,
		type UniverseEntry
	} from '$lib/universe/store';

	let {
		onClose,
		onSwitch,
		onRemoveLast,
	}: {
		onClose: () => void;
		onSwitch: () => void;
		onRemoveLast: () => void;
	} = $props();

	let universes: UniverseEntry[] = $state([]);
	let activeId = $state('');
	let showCreateForm = $state(false);
	let newName = $state('');
	let newPath = $state('');
	let creating = $state(false);
	let error = $state('');
	let confirmRemoveId: string | null = $state(null);
	let overlayEl: HTMLDivElement;

	onMount(() => {
		refresh();
		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	});

	async function refresh() {
		universes = await listUniverses();
		// Find the active one (it's the first one since set_active_universe reorders)
		const activePath = await invoke<string | null>('get_active_universe_path');
		if (activePath) {
			const active = universes.find(u => u.path === activePath);
			if (active) activeId = active.id;
		}
	}

	async function handleSwitch(id: string) {
		if (id === activeId) return;
		// App-freeze audit R2 (2026-07-04): measured ~9ms end-to-end on a settled
		// switch — the switch handler is not the felt lag (that was the departing
		// universe's still-warming background boot; its own reproduce-first pass).
		try {
			await setActiveUniverse(id);
			activeId = id;
			onSwitch();
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleRemove(id: string) {
		confirmRemoveId = id;
	}

	async function confirmRemove() {
		const id = confirmRemoveId;
		if (!id) return;
		confirmRemoveId = null;
		const wasActive = id === activeId;
		await removeUniverseFromRegistry(id);
		await refresh();

		if (universes.length === 0) {
			// Last universe removed — go back to setup
			onRemoveLast();
			return;
		}

		if (wasActive && universes.length > 0) {
			// Removed the active universe but others remain — switch to first
			await handleSwitch(universes[0].id);
		}
	}

	async function pickFolder() {
		try {
			const result = await invoke<string | null>('pick_folder');
			if (result) newPath = result;
		} catch { /* cancelled */ }
	}

	async function handleCreate() {
		const name = newName.trim() || 'My Universe';
		if (!newPath) return;
		creating = true;
		error = '';
		try {
			await createUniverse(name, newPath);
			await refresh();
			showCreateForm = false;
			newName = '';
			newPath = '';
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
		creating = false;
	}

	async function handleAddChild(parentId: string) {
		try {
			const result = await invoke<string | null>('pick_folder');
			if (result) {
				await addChildUniverse(result);
				await refresh();
			}
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleOpenExisting() {
		try {
			const result = await invoke<string | null>('pick_folder');
			if (!result) return;
			error = '';
			creating = true;
			await openExistingUniverse(result);
			await refresh();
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
		creating = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			if (confirmRemoveId) {
				confirmRemoveId = null;
			} else {
				onClose();
			}
		}
	}

	function handleOverlayClick(e: MouseEvent) {
		if (e.target === overlayEl) onClose();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="um-overlay" bind:this={overlayEl} onclick={handleOverlayClick}>
	<div class="um-modal" dir={$dir}>
		<div class="um-header">
			<span class="um-title">{$t('universe.manager.heading')}</span>
			<button class="um-close" onclick={onClose}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
			</button>
		</div>

		<div class="um-body">
			{#if universes.length === 0}
				<div class="um-empty">{$t('universe.manager.noUniverses')}</div>
			{:else}
				{#each universes as u}
					<div class="um-entry" class:active={u.id === activeId}>
						<div class="um-entry-info">
							<div class="um-entry-name">
								{u.name}
								{#if u.id === activeId}
									<span class="um-badge">{$t('universe.manager.active')}</span>
								{/if}
							</div>
							<div class="um-entry-path">{u.path}</div>
							{#if u.id === activeId}
								<button class="um-add-child" onclick={() => handleAddChild(u.id)}>
									<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
									</svg>
									{$t('universe.manager.addChild')}
								</button>
							{/if}
						</div>
						<div class="um-entry-actions">
							{#if u.id !== activeId}
								<button class="um-btn" onclick={() => handleSwitch(u.id)}>{$t('universe.manager.switch')}</button>
							{/if}
							<button class="um-btn um-btn-danger" onclick={() => handleRemove(u.id)}>{$t('universe.manager.remove')}</button>
						</div>
					</div>
				{/each}
			{/if}

			{#if confirmRemoveId}
				<div class="um-confirm">
					<span class="um-confirm-text">{$t('universe.manager.removeConfirm')}</span>
					<div class="um-confirm-actions">
						<button class="um-btn um-btn-danger" onclick={confirmRemove}>{$t('universe.manager.remove')}</button>
						<button class="um-btn" onclick={() => confirmRemoveId = null}>{$t('bases.source.cancel')}</button>
					</div>
				</div>
			{/if}

			{#if error}
				<div class="um-error">{error}</div>
			{/if}
		</div>

		<div class="um-footer">
			{#if showCreateForm}
				<div class="um-create-form">
					<input
						type="text"
						dir="auto"
						bind:value={newName}
						placeholder={$t('universe.setup.namePlaceholder')}
					/>
					<div class="um-folder-row">
						<span class="um-path">{newPath || '—'}</span>
						<button class="um-btn" onclick={pickFolder}>{$t('universe.setup.chooseFolder')}</button>
					</div>
					<div class="um-create-actions">
						<button class="um-btn um-btn-accent" onclick={handleCreate} disabled={creating || !newPath}>
							{creating ? $t('universe.setup.creating') : $t('universe.setup.create')}
						</button>
						<button class="um-btn" onclick={() => showCreateForm = false}>{$t('bases.source.cancel')}</button>
					</div>
				</div>
			{:else}
				<div class="um-footer-buttons">
					<button class="um-btn um-btn-accent" onclick={() => showCreateForm = true}>
						+ {$t('universe.manager.createNew')}
					</button>
					<button class="um-btn" onclick={handleOpenExisting} disabled={creating}>
						{$t('universe.manager.openExisting') ?? 'Open Existing'}
					</button>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.um-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, var(--modal-overlay-alpha, 0.4)); /* MIG-088 §4c — shared dialog-backdrop opacity */
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}
	.um-modal {
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 12px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
		width: 460px;
		max-height: 70vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}
	.um-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.um-title {
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--text-normal);
	}
	.um-close {
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: 4px;
	}
	.um-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.um-body {
		padding: 12px 16px;
		overflow-y: auto;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.um-empty {
		text-align: center;
		color: var(--text-muted);
		font-size: 0.85rem;
		padding: 20px;
	}
	.um-entry {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 12px;
		border-radius: 8px;
		border: 1px solid var(--background-modifier-border);
		gap: 12px;
	}
	.um-entry.active {
		border-color: #22c55e;
		background: rgba(34, 197, 94, 0.10);
		box-shadow: inset 3px 0 0 #22c55e;
	}
	.um-entry-info {
		flex: 1;
		min-width: 0;
	}
	.um-entry-name {
		font-size: 0.88rem;
		font-weight: 600;
		color: var(--text-normal);
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.um-badge {
		font-size: 0.65rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: white;
		background: #22c55e;
		padding: 2px 8px;
		border-radius: 4px;
	}
	.um-entry-path {
		font-size: 0.75rem;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		direction: ltr;
	}
	.um-add-child {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 6px;
		padding: 4px 10px;
		border: 1px dashed var(--background-modifier-border);
		border-radius: 6px;
		background: none;
		color: var(--text-muted);
		font-size: 0.75rem;
		font-family: inherit;
		cursor: pointer;
	}
	.um-add-child:hover {
		color: var(--interactive-accent);
		border-color: var(--interactive-accent);
		background: rgba(124, 58, 237, 0.05);
	}
	.um-entry-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}
	.um-btn {
		padding: 4px 12px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: none;
		color: var(--text-muted);
		font-size: 0.78rem;
		font-family: inherit;
		cursor: pointer;
	}
	.um-btn:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
	.um-btn.um-btn-accent {
		background: var(--interactive-accent);
		color: white;
		border-color: var(--interactive-accent);
	}
	.um-btn.um-btn-accent:hover { opacity: 0.9; }
	.um-btn.um-btn-accent:disabled { opacity: 0.5; cursor: not-allowed; }
	.um-btn.um-btn-danger:hover { color: var(--text-error, #f38ba8); border-color: var(--text-error, #f38ba8); }
	.um-confirm {
		padding: 10px 12px;
		border-radius: 8px;
		border: 1px solid var(--text-error, #f38ba8);
		background: rgba(243, 139, 168, 0.08);
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.um-confirm-text {
		font-size: 0.82rem;
		color: var(--text-normal);
	}
	.um-confirm-actions {
		display: flex;
		gap: 6px;
		justify-content: flex-end;
	}
	.um-footer {
		padding: 12px 16px;
		border-top: 1px solid var(--background-modifier-border);
	}
	.um-create-form {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.um-create-form input {
		padding: 6px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-secondary);
		color: var(--text-normal);
		font-size: 0.85rem;
		font-family: inherit;
	}
	.um-create-form input:focus { outline: none; border-color: var(--interactive-accent); }
	.um-folder-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-secondary);
	}
	.um-path {
		flex: 1;
		font-size: 0.78rem;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		direction: ltr;
	}
	.um-create-actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
	}
	.um-footer-buttons {
		display: flex;
		gap: 8px;
	}
	.um-footer-buttons .um-btn {
		flex: 1;
	}
	.um-error {
		font-size: 0.8rem;
		color: var(--text-error, #f38ba8);
		padding: 4px 0;
	}
</style>

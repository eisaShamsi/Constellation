<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import {
		createUniverse, migrateLegacyData, checkMigrationNeeded,
		openExistingUniverse, setActiveUniverse, addChildUniverse,
		type UniverseEntry
	} from '$lib/universe/store';

	let {
		onCreated,
		migrationMode = false,
	}: {
		onCreated: (entry: UniverseEntry) => void;
		migrationMode?: boolean;
	} = $props();

	// Wizard state
	let step = $state<0 | 1 | 2>(0);
	let universeName = $state('');
	let folderPath = $state('');
	let creating = $state(false);
	let error = $state('');
	let nameInput: HTMLInputElement;
	let needsMigration = $state(false);
	let createdEntry = $state<UniverseEntry | null>(null);

	// Step 2 state — added vaults and child universes
	let addedVaults = $state<{ id: string; name: string; path: string }[]>([]);
	let addedChildren = $state<{ name: string; path: string }[]>([]);
	let adding = $state(false);

	onMount(async () => {
		needsMigration = await checkMigrationNeeded();
		if (needsMigration) {
			// Skip welcome, go straight to name+location
			step = 1;
		}
	});

	$effect(() => {
		if (step === 1) {
			setTimeout(() => nameInput?.focus(), 50);
		}
	});

	async function pickFolder() {
		try {
			const result = await invoke<string | null>('pick_folder');
			if (result) folderPath = result;
		} catch { /* cancelled */ }
	}

	async function handleOpenExisting() {
		error = '';
		try {
			const result = await invoke<string | null>('pick_folder');
			if (!result) return;
			creating = true;
			const entry = await openExistingUniverse(result);
			onCreated(entry);
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
			creating = false;
		}
	}

	async function handleNext() {
		const name = universeName.trim() || $t('universe.setup.namePlaceholder');
		if (!folderPath) {
			error = $t('universe.setup.locationLabel');
			return;
		}

		creating = true;
		error = '';

		try {
			let entry: UniverseEntry;
			if (needsMigration) {
				const universePath = folderPath.replace(/[\\/]+$/, '') + '/' + name;
				entry = await migrateLegacyData(name, universePath);
				await migrateLocalStorage();
			} else {
				entry = await createUniverse(name, folderPath);
			}
			await setActiveUniverse(entry.id);
			createdEntry = entry;
			creating = false;
			step = 2;
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
			creating = false;
		}
	}

	async function handleAddVault() {
		adding = true;
		error = '';
		try {
			const folderPath: string | null = await invoke('pick_folder');
			if (!folderPath) { adding = false; return; }
			const vault: { id: string; name: string; path: string } = await invoke('add_vault', { path: folderPath });
			addedVaults = [...addedVaults, vault];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
		adding = false;
	}

	async function handleRemoveVault(vaultId: string) {
		try {
			await invoke('remove_vault', { vaultId });
			addedVaults = addedVaults.filter(v => v.id !== vaultId);
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleAddChild() {
		adding = true;
		error = '';
		try {
			const result: string | null = await invoke('pick_folder');
			if (!result) { adding = false; return; }
			await addChildUniverse(result);
			// Read child's universe.json for name
			const childName = result.split(/[\\/]/).pop() || result;
			addedChildren = [...addedChildren, { name: childName, path: result }];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
		adding = false;
	}

	async function handleRemoveChild(childPath: string) {
		try {
			await invoke('remove_child_universe', { childPath });
			addedChildren = addedChildren.filter(c => c.path !== childPath);
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	function handleFinish() {
		if (createdEntry) {
			onCreated(createdEntry);
		}
	}

	async function migrateLocalStorage() {
		try {
			const settingsData = localStorage.getItem('constellation-settings');
			if (settingsData) {
				const settings = JSON.parse(settingsData);
				await invoke('save_universe_settings', { settings });
			}
		} catch { /* ignore */ }

		try {
			const bookmarksData = localStorage.getItem('constellation-bookmarks');
			if (bookmarksData) {
				const bookmarks = JSON.parse(bookmarksData);
				await invoke('save_universe_bookmarks', { bookmarks });
			}
		} catch { /* ignore */ }

		try {
			const workspacesData = localStorage.getItem('constellation-workspaces');
			if (workspacesData) {
				const workspaces = JSON.parse(workspacesData);
				await invoke('save_universe_workspaces', { workspaces });
			}
		} catch { /* ignore */ }

		try {
			const types: Record<string, Record<string, string>> = {};
			for (let i = 0; i < localStorage.length; i++) {
				const key = localStorage.key(i);
				if (key && key.startsWith('constellation-prop-types-')) {
					const vaultName = key.replace('constellation-prop-types-', '');
					const data = localStorage.getItem(key);
					if (data) types[vaultName] = JSON.parse(data);
				}
			}
			if (Object.keys(types).length > 0) {
				await invoke('save_universe_property_types', { types });
			}
		} catch { /* ignore */ }

		const keysToRemove = [];
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			if (key && (key.startsWith('constellation-') || key.startsWith('constellation-prop-types-'))) {
				keysToRemove.push(key);
			}
		}
		keysToRemove.forEach(k => localStorage.removeItem(k));
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') { e.preventDefault(); handleNext(); }
	}
</script>

<div class="us-overlay" dir={$dir}>
	<div class="us-container">
		<!-- Logo -->
		<div class="us-logo">
			<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<circle cx="12" cy="12" r="10"/>
				<circle cx="12" cy="12" r="4"/>
				<path d="M12 2v4M12 18v4M2 12h4M18 12h4"/>
				<path d="M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
			</svg>
		</div>

		{#if step === 0}
			<!-- ═══ STEP 0: Welcome ═══ -->
			<h1 class="us-heading">{$t('universe.setup.welcome')}</h1>
			<p class="us-description">{$t('universe.setup.description')}</p>

			<div class="us-choices">
				<button class="us-choice" onclick={() => { step = 1; error = ''; }}>
					<div class="us-choice-icon">
						<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/>
						</svg>
					</div>
					<div class="us-choice-text">
						<span class="us-choice-title">{$t('universe.setup.createNew')}</span>
						<span class="us-choice-desc">{$t('universe.setup.createNewDesc')}</span>
					</div>
				</button>

				<button class="us-choice" onclick={handleOpenExisting} disabled={creating}>
					<div class="us-choice-icon">
						<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
						</svg>
					</div>
					<div class="us-choice-text">
						<span class="us-choice-title">{$t('universe.setup.openExisting')}</span>
						<span class="us-choice-desc">{$t('universe.setup.openExistingDesc')}</span>
					</div>
				</button>
			</div>

			{#if error}
				<div class="us-error">{error}</div>
			{/if}

		{:else if step === 1}
			<!-- ═══ STEP 1: Name & Location ═══ -->
			<h1 class="us-heading">{$t('universe.setup.heading')}</h1>
			<p class="us-description">{$t('universe.setup.description')}</p>

			<!-- Step indicator -->
			<div class="us-steps">
				<span class="us-step active">1</span>
				<span class="us-step-line"></span>
				<span class="us-step">2</span>
			</div>

			<div class="us-form" onkeydown={handleKeydown}>
				<label class="us-field">
					<span class="us-label">{$t('universe.setup.nameLabel')}</span>
					<input
						type="text"
						bind:this={nameInput}
						bind:value={universeName}
						placeholder={$t('universe.setup.namePlaceholder')}
					/>
				</label>

				<label class="us-field">
					<span class="us-label">{$t('universe.setup.locationLabel')}</span>
					<div class="us-folder-row">
						<span class="us-path">{folderPath || '—'}</span>
						<button class="us-browse" onclick={pickFolder}>{$t('universe.setup.chooseFolder')}</button>
					</div>
				</label>

				{#if error}
					<div class="us-error">{error}</div>
				{/if}

				<div class="us-nav-row">
					{#if !needsMigration}
						<button class="us-back" onclick={() => { step = 0; error = ''; }}>{$t('universe.setup.back')}</button>
					{/if}
					<button class="us-create" onclick={handleNext} disabled={creating || !folderPath}>
						{creating ? $t('universe.setup.creating') : $t('universe.setup.next')}
					</button>
				</div>
			</div>

		{:else if step === 2}
			<!-- ═══ STEP 2: Add Vaults & Child Universes ═══ -->
			<h1 class="us-heading">{$t('universe.setup.addVaultsHeading')}</h1>
			<p class="us-description">{$t('universe.setup.addVaultsDescription')}</p>

			<!-- Step indicator -->
			<div class="us-steps">
				<span class="us-step done">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
				</span>
				<span class="us-step-line"></span>
				<span class="us-step active">2</span>
			</div>

			<div class="us-form">
				<!-- Added Vaults List -->
				{#if addedVaults.length > 0}
					<div class="us-list">
						{#each addedVaults as vault (vault.id)}
							<div class="us-list-item">
								<svg class="us-list-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"/>
								</svg>
								<span class="us-list-name">{vault.name}</span>
								<button class="us-list-remove" onclick={() => handleRemoveVault(vault.id)} title={$t('universe.setup.remove')}>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
								</button>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Added Children List -->
				{#if addedChildren.length > 0}
					<div class="us-list">
						{#each addedChildren as child (child.path)}
							<div class="us-list-item">
								<svg class="us-list-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
									<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/>
								</svg>
								<span class="us-list-name">{child.name}</span>
								<button class="us-list-remove" onclick={() => handleRemoveChild(child.path)} title={$t('universe.setup.remove')}>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
								</button>
							</div>
						{/each}
					</div>
				{/if}

				{#if addedVaults.length === 0 && addedChildren.length === 0}
					<div class="us-empty">{$t('universe.setup.noVaultsYet')}</div>
				{/if}

				{#if error}
					<div class="us-error">{error}</div>
				{/if}

				<!-- Action buttons -->
				<div class="us-add-buttons">
					<button class="us-add-btn" onclick={handleAddVault} disabled={adding}>
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"/>
						</svg>
						{$t('universe.setup.addVault')}
					</button>
					<button class="us-add-btn" onclick={handleAddChild} disabled={adding}>
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/>
						</svg>
						{$t('universe.setup.addChildUniverse')}
					</button>
				</div>

				<!-- Finish / Skip -->
				<div class="us-nav-row">
					<button class="us-skip" onclick={handleFinish}>{$t('universe.setup.skip')}</button>
					<button class="us-create" onclick={handleFinish} disabled={addedVaults.length === 0 && addedChildren.length === 0}>
						{$t('universe.setup.finish')}
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.us-overlay {
		position: fixed;
		inset: 0;
		background: var(--background-primary, #1e1e2e);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 9999;
	}
	.us-container {
		max-width: 460px;
		width: 100%;
		padding: 40px;
		text-align: center;
	}
	.us-logo {
		margin-bottom: 20px;
		color: var(--interactive-accent, #7c3aed);
	}
	.us-heading {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--text-normal, #cdd6f4);
		margin: 0 0 8px;
	}
	.us-description {
		font-size: 0.85rem;
		color: var(--text-muted, #a6adc8);
		margin: 0 0 24px;
		line-height: 1.5;
	}

	/* ─── Step indicator ─── */
	.us-steps {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0;
		margin-bottom: 24px;
	}
	.us-step {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 0.75rem;
		font-weight: 700;
		background: var(--background-secondary, #313244);
		color: var(--text-muted, #a6adc8);
		border: 2px solid var(--background-modifier-border, #45475a);
	}
	.us-step.active {
		background: var(--interactive-accent, #7c3aed);
		color: white;
		border-color: var(--interactive-accent, #7c3aed);
	}
	.us-step.done {
		background: var(--text-success, #a6e3a1);
		color: var(--background-primary, #1e1e2e);
		border-color: var(--text-success, #a6e3a1);
	}
	.us-step-line {
		width: 40px;
		height: 2px;
		background: var(--background-modifier-border, #45475a);
	}

	/* ─── Step 0: Choices ─── */
	.us-choices {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 8px;
	}
	.us-choice {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 16px 18px;
		border: 1px solid var(--background-modifier-border, #45475a);
		border-radius: 10px;
		background: var(--background-secondary, #313244);
		cursor: pointer;
		text-align: start;
		font-family: inherit;
		transition: border-color 0.15s, background 0.15s;
	}
	.us-choice:hover:not(:disabled) {
		border-color: var(--interactive-accent, #7c3aed);
		background: var(--background-modifier-hover, #45475a);
	}
	.us-choice:disabled { opacity: 0.5; cursor: not-allowed; }
	.us-choice-icon {
		flex-shrink: 0;
		color: var(--interactive-accent, #7c3aed);
	}
	.us-choice-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.us-choice-title {
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--text-normal, #cdd6f4);
	}
	.us-choice-desc {
		font-size: 0.78rem;
		color: var(--text-muted, #a6adc8);
		line-height: 1.4;
	}

	/* ─── Form ─── */
	.us-form {
		display: flex;
		flex-direction: column;
		gap: 16px;
		text-align: start;
	}
	.us-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.us-label {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-muted, #a6adc8);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.us-field input {
		padding: 8px 12px;
		border: 1px solid var(--background-modifier-border, #45475a);
		border-radius: 8px;
		background: var(--background-secondary, #313244);
		color: var(--text-normal, #cdd6f4);
		font-size: 0.9rem;
		font-family: inherit;
	}
	.us-field input:focus {
		outline: none;
		border-color: var(--interactive-accent, #7c3aed);
	}
	.us-folder-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		border: 1px solid var(--background-modifier-border, #45475a);
		border-radius: 8px;
		background: var(--background-secondary, #313244);
	}
	.us-path {
		flex: 1;
		font-size: 0.82rem;
		color: var(--text-muted, #a6adc8);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		direction: ltr;
	}
	.us-browse {
		padding: 4px 12px;
		border: 1px solid var(--background-modifier-border, #45475a);
		border-radius: 6px;
		background: none;
		color: var(--text-normal, #cdd6f4);
		font-size: 0.78rem;
		font-family: inherit;
		cursor: pointer;
		white-space: nowrap;
	}
	.us-browse:hover {
		background: var(--background-modifier-hover, #45475a);
	}
	.us-error {
		font-size: 0.8rem;
		color: var(--text-error, #f38ba8);
	}

	/* ─── Navigation row ─── */
	.us-nav-row {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 10px;
		margin-top: 8px;
	}
	.us-back {
		padding: 8px 16px;
		border: 1px solid var(--background-modifier-border, #45475a);
		border-radius: 8px;
		background: none;
		color: var(--text-muted, #a6adc8);
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
	}
	.us-back:hover { background: var(--background-modifier-hover, #45475a); }
	.us-skip {
		padding: 8px 16px;
		border: none;
		border-radius: 8px;
		background: none;
		color: var(--text-muted, #a6adc8);
		font-size: 0.85rem;
		font-family: inherit;
		cursor: pointer;
	}
	.us-skip:hover { color: var(--text-normal, #cdd6f4); }
	.us-create {
		padding: 10px 20px;
		border: none;
		border-radius: 8px;
		background: var(--interactive-accent, #7c3aed);
		color: white;
		font-size: 0.9rem;
		font-weight: 600;
		font-family: inherit;
		cursor: pointer;
	}
	.us-create:hover:not(:disabled) { opacity: 0.9; }
	.us-create:disabled { opacity: 0.5; cursor: not-allowed; }

	/* ─── Step 2: Lists ─── */
	.us-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.us-list-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 12px;
		border-radius: 8px;
		background: var(--background-secondary, #313244);
		border: 1px solid var(--background-modifier-border, #45475a);
	}
	.us-list-icon {
		flex-shrink: 0;
		color: var(--text-muted, #a6adc8);
	}
	.us-list-name {
		flex: 1;
		font-size: 0.85rem;
		color: var(--text-normal, #cdd6f4);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.us-list-remove {
		flex-shrink: 0;
		padding: 2px;
		border: none;
		border-radius: 4px;
		background: none;
		color: var(--text-muted, #a6adc8);
		cursor: pointer;
		display: flex;
	}
	.us-list-remove:hover { color: var(--text-error, #f38ba8); }
	.us-empty {
		padding: 24px;
		text-align: center;
		font-size: 0.82rem;
		color: var(--text-faint, #6c7086);
		border: 1px dashed var(--background-modifier-border, #45475a);
		border-radius: 8px;
	}

	/* ─── Add buttons ─── */
	.us-add-buttons {
		display: flex;
		gap: 8px;
	}
	.us-add-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 8px 14px;
		border: 1px dashed var(--background-modifier-border, #45475a);
		border-radius: 8px;
		background: none;
		color: var(--text-muted, #a6adc8);
		font-size: 0.8rem;
		font-family: inherit;
		cursor: pointer;
		transition: border-color 0.15s, color 0.15s;
	}
	.us-add-btn:hover:not(:disabled) {
		border-color: var(--interactive-accent, #7c3aed);
		color: var(--text-normal, #cdd6f4);
	}
	.us-add-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>

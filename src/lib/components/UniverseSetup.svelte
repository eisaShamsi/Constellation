<script lang="ts">
	import { onMount } from 'svelte';
	import { t, dir } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import {
		createUniverse, migrateLegacyData, checkMigrationNeeded,
		openExistingUniverse, setActiveUniverse, addChildUniverse,
		linkLibraryAsUniverse,
		type UniverseEntry
	} from '$lib/universe/store';
	import { importPickSource, importPreview, importExecute } from '$lib/importers/store';
	import type { ImportFormat, ImportPreview } from '$lib/importers/types';
	import { bringInLibrary } from '$lib/libraries/store';
	import { normalizePathKey } from '$lib/utils';
	import BringInDialog from './BringInDialog.svelte';

	let {
		onCreated,
		migrationMode = false,
	}: {
		onCreated: (entry: UniverseEntry) => void;
		migrationMode?: boolean;
	} = $props();

	// Wizard state
	let step = $state<0 | 1 | 2 | 3 | 4>(0);
	let universeName = $state('');
	let folderPath = $state('');
	let creating = $state(false);
	let error = $state('');
	let nameInput: HTMLInputElement;
	let needsMigration = $state(false);
	let createdEntry = $state<UniverseEntry | null>(null);

	// Step 2 state — added libraries and child universes
	let addedLibraries = $state<{ id: string; name: string; path: string }[]>([]);
	let addedChildren = $state<{ name: string; path: string }[]>([]);
	let adding = $state(false);
	let starterKit = $state(true);
	/** MIG-108 — an external pick awaiting the Copy/Move choice (Boss D2: ask each time). */
	let bringInSource = $state<string | null>(null);

	// Import state (steps 3 & 4)
	let selectedImportFormat = $state<ImportFormat>('obsidian');
	let importSourcePath = $state('');
	let importPreviewData = $state<ImportPreview | null>(null);
	let importUniverseLocation = $state('');

	const importFormats: { id: ImportFormat; icon: string; labelKey: string; descKey: string }[] = [
		{ id: 'obsidian', icon: 'M12 2L3 7v10l9 5 9-5V7l-9-5zM12 22V12M3 7l9 5 9-5', labelKey: 'importer.formats.obsidian', descKey: 'importer.formats.obsidianDesc' },
		{ id: 'markdown', icon: 'M3 3h18v18H3zM7 15V9l3 4 3-4v6M17 9v6', labelKey: 'importer.formats.markdown', descKey: 'importer.formats.markdownDesc' },
		{ id: 'notion', icon: 'M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z', labelKey: 'importer.formats.notion', descKey: 'importer.formats.notionDesc' },
		{ id: 'bear', icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z', labelKey: 'importer.formats.bear', descKey: 'importer.formats.bearDesc' },
		{ id: 'enex', icon: 'M9 3H5a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2V5a2 2 0 00-2-2h-4M9 3v4a1 1 0 001 1h4a1 1 0 001-1V3M9 3h6', labelKey: 'importer.formats.evernote', descKey: 'importer.formats.evernoteDesc' },
		{ id: 'html', icon: 'M4 7l4-4 4 4M4 17l4 4 4-4M14 3l4 9-4 9', labelKey: 'importer.formats.html', descKey: 'importer.formats.htmlDesc' },
		{ id: 'csv', icon: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8zM14 2v6h6M8 13h2M8 17h2M14 13h2M14 17h2', labelKey: 'importer.formats.csv', descKey: 'importer.formats.csvDesc' },
		{ id: 'txt', icon: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8zM14 2v6h6M16 13H8M16 17H8M10 9H8', labelKey: 'importer.formats.txt', descKey: 'importer.formats.txtDesc' },
	];

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

	async function handleLinkLibrary() {
		error = '';
		try {
			const result = await invoke<string | null>('pick_folder');
			if (!result) return;
			creating = true;
			const entry = await linkLibraryAsUniverse(result);
			await setActiveUniverse(entry.id);
			createdEntry = entry;
			creating = false;
			step = 2;
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
			creating = false;
		}
	}

	async function handleAddLibrary() {
		adding = true;
		error = '';
		try {
			const folderPath: string | null = await invoke('pick_folder');
			if (!folderPath) { adding = false; return; }
			// MIG-108 One Universe, One Location — an external pick is not a dead-end error:
			// it opens the same Copy/Move choice the main window offers (Boss D2).
			const rn = normalizePathKey(createdEntry?.path ?? '');
			const pn = normalizePathKey(folderPath);
			if (rn && !(pn === rn || pn.startsWith(rn + '/'))) {
				bringInSource = folderPath;
				adding = false;
				return;
			}
			const lib: { id: string; name: string; path: string } = await invoke('add_library', { path: folderPath });
			addedLibraries = [...addedLibraries, lib];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
		adding = false;
	}

	async function handleBringInChoice(mode: 'copy' | 'move') {
		const src = bringInSource;
		bringInSource = null;
		if (!src) return;
		adding = true;
		error = '';
		try {
			const lib = await bringInLibrary(src, mode);
			addedLibraries = [...addedLibraries, { id: lib.id, name: lib.name, path: lib.path }];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
		adding = false;
	}

	async function handleRemoveLibrary(libraryId: string) {
		try {
			await invoke('remove_library', { libraryId });
			addedLibraries = addedLibraries.filter(v => v.id !== libraryId);
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

	async function handleFinish() {
		if (starterKit && addedLibraries.length > 0) {
			try {
				await invoke('scaffold_starter_library', { libraryPath: addedLibraries[0].path });
			} catch (e) {
				console.error('Starter kit scaffold failed:', e);
			}
		}
		if (createdEntry) {
			onCreated(createdEntry);
		}
	}

	async function migrateLocalStorage() {
		// PJ-207 §15 — DELETE ONLY WHAT ACTUALLY ARRIVED.
		//
		// Each leg below wrote to the universe inside `catch { /* ignore */ }`, and the cleanup
		// at the end then removed the localStorage source keys UNCONDITIONALLY. So a single
		// failed write — the universe folder not yet writable, an AV lock on first run — deleted
		// the only remaining copy of the user's settings, bookmarks or saved workspaces. A
		// migration that discards the source before confirming the destination is not a
		// migration. A key is now removed only if its own write succeeded, and a failure is
		// logged instead of vanishing.
		const migrated = new Set<string>();

		try {
			const settingsData = localStorage.getItem('constellation-settings');
			if (settingsData) {
				const settings = JSON.parse(settingsData);
				await invoke('save_universe_settings', { settings });
				migrated.add('constellation-settings');
			}
		} catch (e) { console.error('[migrate] settings did NOT move to the universe — keeping the local copy:', e); }

		try {
			// PJ-207 §15 — this called `save_universe_bookmarks`, which does not exist: MIG-092
			// retired the writer and lib.rs registers only the reader. So the leg threw "command
			// not found" every single time and the bookmarks never left localStorage — while the
			// one-time Bookmarks→Starred adoption downstream spent its only run against the empty
			// bookmarks.json and persisted that emptiness, so it can never re-run.
			const bookmarksData = localStorage.getItem('constellation-bookmarks');
			if (bookmarksData) {
				const bookmarks = JSON.parse(bookmarksData);
				await invoke('migrate_universe_bookmarks', { bookmarks });
				migrated.add('constellation-bookmarks');
			}
		} catch (e) { console.error('[migrate] bookmarks did NOT move to the universe — keeping the local copy:', e); }

		try {
			const workspacesData = localStorage.getItem('constellation-workspaces');
			if (workspacesData) {
				const workspaces = JSON.parse(workspacesData);
				await invoke('save_universe_workspaces', { workspaces });
				migrated.add('constellation-workspaces');
			}
		} catch (e) { console.error('[migrate] workspaces did NOT move to the universe — keeping the local copy:', e); }

		try {
			const types: Record<string, Record<string, string>> = {};
			const typeKeys: string[] = [];
			for (let i = 0; i < localStorage.length; i++) {
				const key = localStorage.key(i);
				if (key && key.startsWith('constellation-prop-types-')) {
					const libraryName = key.replace('constellation-prop-types-', '');
					const data = localStorage.getItem(key);
					if (data) {
						types[libraryName] = JSON.parse(data);
						typeKeys.push(key);
					}
				}
			}
			if (Object.keys(types).length > 0) {
				await invoke('save_universe_property_types', { types });
				for (const k of typeKeys) migrated.add(k); // one write for the batch — all or none
			}
		} catch (e) { console.error('[migrate] property types did NOT move to the universe — keeping the local copies:', e); }

		// Clear ONLY what this migration just moved into the universe.
		//
		// 2026-08-08 (PJ-229 work, WA#6): this swept every key starting with
		// `constellation-`, which is far more than it migrated. It took
		// `constellation-wab` with it — the write-ahead backup of note content that has
		// not reached disk yet (`libraries/store.ts`), i.e. the recovery net for exactly
		// the work a user would most hate to lose — and it would now also take
		// `constellation-locale`, resetting the interface language on this one path.
		// A cleanup that deletes things it did not create is not a cleanup.
		const MIGRATED_KEYS = [
			'constellation-settings',
			'constellation-bookmarks',
			'constellation-workspaces',
		];
		const keysToRemove = [];
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			// …and ONLY if this run actually wrote it into the universe (see the top of this
			// function). `MIGRATED_KEYS` says which keys this migration is allowed to touch;
			// `migrated` says which of them made it.
			const isOurs = key && (MIGRATED_KEYS.includes(key) || key.startsWith('constellation-prop-types-'));
			if (isOurs && migrated.has(key)) {
				keysToRemove.push(key);
			}
		}
		keysToRemove.forEach(k => localStorage.removeItem(k));
	}

	async function handleImportPickSource() {
		error = '';
		try {
			const pickType = ['markdown', 'obsidian', 'notion', 'bear'].includes(selectedImportFormat) ? 'folder' : selectedImportFormat;
			importSourcePath = await importPickSource(pickType);
			if (!importSourcePath) return;
			importPreviewData = await importPreview(importSourcePath, selectedImportFormat);
			// Derive universe name from source folder/file name
			const sourceName = importSourcePath.replace(/[\\/]+$/, '').split(/[\\/]/).pop() || 'Imported';
			universeName = sourceName;
			step = 4;
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleImportPickLocation() {
		try {
			const result = await invoke<string | null>('pick_folder');
			if (result) importUniverseLocation = result;
		} catch { /* cancelled */ }
	}

	async function handleImportConfirm() {
		if (!importUniverseLocation || !importSourcePath) return;
		creating = true;
		error = '';
		try {
			// 1. Create universe
			const name = universeName.trim() || 'Imported';
			const entry = await createUniverse(name, importUniverseLocation);
			await setActiveUniverse(entry.id);
			createdEntry = entry;

			// 2. Build library path inside the universe directory
			const libraryName = importSourcePath.replace(/[\\/]+$/, '').split(/[\\/]/).pop() || 'Library';
			const universePath = importUniverseLocation.replace(/[\\/]+$/, '') + '/' + name;
			const libraryPath = universePath + '/Libraries/' + libraryName;

			// 3. Import files — this creates the destination directory automatically
			await importExecute(importSourcePath, selectedImportFormat, libraryPath, '');

			// 4. Register the library
			const lib: { id: string; name: string; path: string } = await invoke('add_library', { path: libraryPath });

			addedLibraries = [lib];
			creating = false;
			step = 2;
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
			creating = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') { e.preventDefault(); handleNext(); }
	}
</script>

<div class="us-overlay" dir={$dir}>
	<div class="us-container">
		<!-- Logo -->
		<div class="us-logo">
			<svg width="48" height="48" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
				<circle cx="100" cy="100" r="30" fill="#534AB7"/>
				<circle cx="100" cy="100" r="19" fill="#3C3489"/>
				<circle cx="45" cy="42" r="24" fill="#378ADD"/>
				<circle cx="130" cy="52" r="20" fill="#7F77DD"/>
				<circle cx="162" cy="110" r="16" fill="#1D9E75"/>
				<circle cx="80" cy="158" r="13" fill="#D85A30"/>
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

				<button class="us-choice" onclick={handleLinkLibrary} disabled={creating}>
					<div class="us-choice-icon">
						<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
							<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
						</svg>
					</div>
					<div class="us-choice-text">
						<span class="us-choice-title">{$t('universe.setup.linkLibrary')}</span>
						<span class="us-choice-desc">{$t('universe.setup.linkLibraryDesc')}</span>
					</div>
				</button>

				<button class="us-choice" onclick={() => { step = 3; error = ''; }} disabled={creating}>
					<div class="us-choice-icon">
						<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
						</svg>
					</div>
					<div class="us-choice-text">
						<span class="us-choice-title">{$t('universe.setup.importApp')}</span>
						<span class="us-choice-desc">{$t('universe.setup.importAppDesc')}</span>
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
						dir="auto"
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
			<!-- ═══ STEP 2: Add Libraries & Child Universes ═══ -->
			<h1 class="us-heading">{$t('universe.setup.addLibrariesHeading')}</h1>
			<p class="us-description">{$t('universe.setup.addLibrariesDescription')}</p>

			<!-- Step indicator -->
			<div class="us-steps">
				<span class="us-step done">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
				</span>
				<span class="us-step-line"></span>
				<span class="us-step active">2</span>
			</div>

			<div class="us-form">
				<!-- Added Libraries List -->
				{#if addedLibraries.length > 0}
					<div class="us-list">
						{#each addedLibraries as lib (lib.id)}
							<div class="us-list-item">
								<svg class="us-list-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"/>
								</svg>
								<span class="us-list-name">{lib.name}</span>
								<button class="us-list-remove" onclick={() => handleRemoveLibrary(lib.id)} title={$t('universe.setup.remove')}>
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
								<svg class="us-list-icon" width="16" height="16" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
									<circle cx="100" cy="100" r="30" fill="#534AB7"/><circle cx="100" cy="100" r="19" fill="#3C3489"/><circle cx="45" cy="42" r="24" fill="#378ADD"/><circle cx="130" cy="52" r="20" fill="#7F77DD"/><circle cx="162" cy="110" r="16" fill="#1D9E75"/><circle cx="80" cy="158" r="13" fill="#D85A30"/>
								</svg>
								<span class="us-list-name">{child.name}</span>
								<button class="us-list-remove" onclick={() => handleRemoveChild(child.path)} title={$t('universe.setup.remove')}>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
								</button>
							</div>
						{/each}
					</div>
				{/if}

				{#if addedLibraries.length === 0 && addedChildren.length === 0}
					<div class="us-empty">{$t('universe.setup.noLibrariesYet')}</div>
				{/if}

				{#if error}
					<div class="us-error">{error}</div>
				{/if}

				<!-- Action buttons -->
				<div class="us-add-buttons">
					<button class="us-add-btn" onclick={handleAddLibrary} disabled={adding}>
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"/>
						</svg>
						{$t('universe.setup.addLibrary')}
					</button>
					<button class="us-add-btn" onclick={handleAddChild} disabled={adding}>
						<svg width="16" height="16" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
							<circle cx="100" cy="100" r="30" fill="#534AB7"/><circle cx="100" cy="100" r="19" fill="#3C3489"/><circle cx="45" cy="42" r="24" fill="#378ADD"/><circle cx="130" cy="52" r="20" fill="#7F77DD"/><circle cx="162" cy="110" r="16" fill="#1D9E75"/><circle cx="80" cy="158" r="13" fill="#D85A30"/>
						</svg>
						{$t('universe.setup.addChildUniverse')}
					</button>
				</div>

				<!-- Starter Kit checkbox -->
				{#if addedLibraries.length > 0}
					<label class="us-starter-check">
						<input type="checkbox" bind:checked={starterKit} />
						<span class="us-starter-label">{$t('universe.setup.starterKit')}</span>
						<span class="us-starter-desc">{$t('universe.setup.starterKitDesc')}</span>
					</label>
				{/if}

				<!-- Finish / Skip -->
				<div class="us-nav-row">
					<button class="us-skip" onclick={handleFinish}>{$t('universe.setup.skip')}</button>
					<button class="us-create" onclick={handleFinish} disabled={addedLibraries.length === 0 && addedChildren.length === 0}>
						{$t('universe.setup.finish')}
					</button>
				</div>
			</div>
		{:else if step === 3}
			<!-- ═══ STEP 3: Import — Choose Format ═══ -->
			<h1 class="us-heading">{$t('universe.setup.importHeading')}</h1>
			<p class="us-description">{$t('universe.setup.importDescription')}</p>

			<div class="us-import-formats">
				{#each importFormats as fmt}
					<button
						class="us-import-format"
						class:active={selectedImportFormat === fmt.id}
						onclick={() => selectedImportFormat = fmt.id}
					>
						<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d={fmt.icon}/></svg>
						<span class="us-import-format-name">{$t(fmt.labelKey)}</span>
					</button>
				{/each}
			</div>

			{#if error}
				<div class="us-error">{error}</div>
			{/if}

			<div class="us-nav-row">
				<button class="us-back" onclick={() => { step = 0; error = ''; }}>{$t('universe.setup.back')}</button>
				<button class="us-create" onclick={handleImportPickSource}>{$t('importer.selectSource')}</button>
			</div>

		{:else if step === 4}
			<!-- ═══ STEP 4: Import — Preview & Confirm ═══ -->
			<h1 class="us-heading">{$t('universe.setup.importPreviewHeading')}</h1>

			{#if importPreviewData}
				<div class="us-import-summary">
					<div class="us-import-stat">
						<span class="us-import-stat-num">{importPreviewData.file_count}</span>
						<span class="us-import-stat-label">{$t('importer.files')}</span>
					</div>
				</div>
			{/if}

			<div class="us-form">
				<label class="us-field">
					<span class="us-label">{$t('universe.setup.nameLabel')}</span>
					<input type="text" dir="auto" bind:value={universeName} placeholder={$t('universe.setup.namePlaceholder')} />
				</label>

				<label class="us-field">
					<span class="us-label">{$t('universe.setup.importLocationLabel')}</span>
					<div class="us-folder-row">
						<span class="us-path">{importUniverseLocation || '—'}</span>
						<button class="us-browse" onclick={handleImportPickLocation}>{$t('universe.setup.chooseFolder')}</button>
					</div>
				</label>

				{#if error}
					<div class="us-error">{error}</div>
				{/if}

				<div class="us-nav-row">
					<button class="us-back" onclick={() => { step = 3; error = ''; }}>{$t('universe.setup.back')}</button>
					<button class="us-create" onclick={handleImportConfirm} disabled={creating || !importUniverseLocation}>
						{creating ? $t('universe.setup.importingNotes') : $t('universe.setup.importConfirm')}
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

{#if bringInSource}
	<BringInDialog
		sourcePath={bringInSource}
		onChoose={handleBringInChoice}
		onCancel={() => (bringInSource = null)}
	/>
{/if}

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

	/* ─── Starter Kit checkbox ─── */
	.us-starter-check {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 12px;
		border: 1px solid var(--background-modifier-border, #45475a);
		border-radius: 8px;
		background: var(--background-secondary, #313244);
		cursor: pointer;
		flex-wrap: wrap;
	}
	.us-starter-check input[type="checkbox"] {
		margin-top: 2px;
		accent-color: var(--interactive-accent, #7c3aed);
	}
	.us-starter-label {
		font-size: 0.85rem;
		font-weight: 600;
		color: var(--text-normal, #cdd6f4);
		flex: 1;
	}
	.us-starter-desc {
		width: 100%;
		font-size: 0.78rem;
		color: var(--text-muted, #a6adc8);
		padding-inline-start: 24px;
		line-height: 1.4;
	}

	/* ─── Step 3: Import format grid ─── */
	.us-import-formats {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
		margin-bottom: 20px;
	}
	.us-import-format {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 12px 6px;
		border: 2px solid var(--background-modifier-border, #45475a);
		border-radius: 10px;
		background: var(--background-secondary, #313244);
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
		font-family: inherit;
		color: var(--text-muted, #a6adc8);
	}
	.us-import-format:hover {
		border-color: var(--interactive-accent, #7c3aed);
		color: var(--text-normal, #cdd6f4);
	}
	.us-import-format.active {
		border-color: var(--interactive-accent, #7c3aed);
		background: color-mix(in srgb, var(--interactive-accent, #7c3aed) 10%, transparent);
		color: var(--text-normal, #cdd6f4);
	}
	.us-import-format-name {
		font-size: 0.72rem;
		font-weight: 600;
		text-align: center;
	}

	/* ─── Step 4: Import summary ─── */
	.us-import-summary {
		display: flex;
		justify-content: center;
		gap: 24px;
		margin-bottom: 20px;
	}
	.us-import-stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}
	.us-import-stat-num {
		font-size: 2rem;
		font-weight: 700;
		color: var(--interactive-accent, #7c3aed);
	}
	.us-import-stat-label {
		font-size: 0.75rem;
		color: var(--text-muted, #a6adc8);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
</style>

<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { dir, t } from '$lib/i18n';
	import {
		vaults, loadVaults, appSettings, loadSettings,
		openNoteTab, openTabs, activeTabId, activeTab,
		switchTab, closeTab, createEmptyTab,
		parseFrontmatter, editingTabIds, toggleEditMode,
		navigateBack, navigateForward,
		scanVaultLinks, scanVaultTags, scanVaultIndex,
		buildGraphData, readNotePreview,
		type NoteLink, type GraphNode, type GraphLink, type IndexEntry
	} from '$lib/vaults/store';
	import { get } from 'svelte/store';
	import { detectDir } from '$lib/utils';
	import NotePane from '$lib/components/NotePane.svelte';
	import GraphView from '$lib/components/GraphView.svelte';
	import NoteGrid from '$lib/components/NoteGrid.svelte';
	import {
		onNoteToScreen, onNoteSaved, onUniverseSwitch, onSettingsChanged,
		sendNoteToMain, notifyScreenClosed,
		type ScreenNote, type ScreenMode
	} from '$lib/secondScreen';
	import {
		setActiveUniverse, listUniverses
	} from '$lib/universe/store';

	// ─── Mode state ───
	let currentMode = $state<ScreenMode>('grid');
	let linkedBrowsing = $state(true);

	// ─── Data state ───
	let allNotes = $state<{ name: string; path: string; vaultName: string }[]>([]);
	let graphNodes = $state<GraphNode[]>([]);
	let graphLinks = $state<GraphLink[]>([]);
	let loading = $state(true);
	let universeName = $state('');

	// ─── Vault color map ───
	const vaultColors = [
		'#7c3aed', '#3b82f6', '#10b981', '#f59e0b', '#ef4444',
		'#ec4899', '#8b5cf6', '#06b6d4', '#84cc16', '#f97316'
	];
	let vaultColorMap = $derived.by(() => {
		const map: Record<string, string> = {};
		const v = $vaults;
		v.forEach((vault, i) => { map[vault.name] = vaultColors[i % vaultColors.length]; });
		return map;
	});

	// ─── Theme sync ───
	if (typeof document !== 'undefined') {
		// Ensure theme class is set immediately
		if (!document.body.classList.contains('theme-light') && !document.body.classList.contains('theme-dark')) {
			document.body.classList.add('theme-light');
		}
	}
	const colorScheme = $derived($appSettings.colorScheme);
	$effect(() => {
		if (typeof document !== 'undefined') {
			const resolved = colorScheme === 'system'
				? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
				: (colorScheme || 'light');
			document.body.classList.remove('theme-light', 'theme-dark');
			document.body.classList.add(`theme-${resolved}`);
		}
	});

	// ─── Data loading ───
	async function loadAllData() {
		loading = true;
		try {
			await loadSettings();
			await loadVaults();

			const vaultList = $vaults;
			const links: NoteLink[] = [];
			const notes: { name: string; path: string; vaultName: string }[] = [];

			for (const vault of vaultList) {
				const [vaultLinks, vaultNotes] = await Promise.all([
					scanVaultLinks(vault.path, vault.name).catch(() => [] as NoteLink[]),
					invoke('collect_vault_notes', { vaultPath: vault.path }).catch(() => []) as Promise<any[]>,
				]);
				links.push(...vaultLinks);
				notes.push(...vaultNotes.map((n: any) => ({ name: n.name, path: n.path, vaultName: vault.name })));
			}

			allNotes = notes;

			if (vaultList.length > 0) {
				const { nodes, links: gLinks } = buildGraphData(links, notes);
				graphNodes = nodes;
				graphLinks = gLinks;
			}
		} finally {
			loading = false;
		}
	}

	// ─── Event listeners ───
	let unlisteners: (() => void)[] = [];

	// ─── Close handler ───
	async function handleClose() {
		notifyScreenClosed();
		try { await invoke('close_second_screen'); } catch {}
	}

	onMount(async () => {
		const win = getCurrentWindow();
		try { await win.setTitle('Constellation'); } catch {}

		// Listen for screen-hidden event (Rust hides instead of closing)
		const unlistenHidden = await listen('screen-hidden', () => {
			notifyScreenClosed();
		});
		unlisteners.push(unlistenHidden);

		// Ensure universe is active (shared Rust state from main window)
		try {
			const universes = await listUniverses();
			if (universes.length > 0) {
				await setActiveUniverse(universes[0].id);
				universeName = universes[0].name || '';
				await win.setTitle(`Constellation - ${universeName}`).catch(() => {});
			}
		} catch {}

		// Load data
		await loadAllData();

		// Listen for notes sent from main window
		const u1 = await onNoteToScreen(async (note) => {
			await openNoteTab(note.path, note.vaultName, note.vaultPath, note.vaultColor);
			if (linkedBrowsing) {
				currentMode = 'detail';
			}
		});
		unlisteners.push(u1);

		// Listen for note saves
		const u2 = await onNoteSaved(async () => {
			const tab = get(activeTab);
			if (tab?.path) {
				try {
					const content = await invoke<string>('read_note', { notePath: tab.path });
					tab.content = content;
				} catch {}
			}
		});
		unlisteners.push(u2);

		// Listen for universe switch
		const u3 = await onUniverseSwitch(async () => {
			// Update universe name
			try {
				const universes = await listUniverses();
				if (universes.length > 0) {
					universeName = universes[0].name || '';
					getCurrentWindow().setTitle(`Constellation - ${universeName}`).catch(() => {});
				}
			} catch {}
			await loadAllData();
		});
		unlisteners.push(u3);

		// Listen for settings changes
		const u4 = await onSettingsChanged(async () => {
			await loadSettings();
		});
		unlisteners.push(u4);

		// Listen for vault file changes
		const u5 = await listen<{ vaultId: string; paths: string[] }>('vault-changed', async () => {
			setTimeout(() => loadAllData(), 3000);
		});
		unlisteners.push(u5);
	});

	onDestroy(() => {
		unlisteners.forEach(u => u());
	});

	// ─── Handlers ───
	function handleGridNoteClick(note: { name: string; path: string; vaultName: string }) {
		const vault = $vaults.find(v => v.name === note.vaultName);
		if (!vault) return;
		const color = vaultColorMap[note.vaultName] || '#7c3aed';
		sendNoteToMain({
			path: note.path,
			name: note.name,
			vaultName: note.vaultName,
			vaultPath: vault.path,
			vaultColor: color,
		});
	}

	function handleGridNoteDoubleClick(note: { name: string; path: string; vaultName: string }) {
		const vault = $vaults.find(v => v.name === note.vaultName);
		if (!vault) return;
		const color = vaultColorMap[note.vaultName] || '#7c3aed';
		openNoteTab(note.path, note.vaultName, vault.path, color);
		currentMode = 'detail';
	}

	function handleGraphNodeClick(path: string, vaultName: string) {
		const vault = $vaults.find(v => v.name === vaultName);
		if (!vault) return;
		const color = vaultColorMap[vaultName] || '#7c3aed';
		sendNoteToMain({
			path, name: path.split('/').pop()?.replace('.md', '') ?? path,
			vaultName, vaultPath: vault.path, vaultColor: color,
		});
	}

	function switchMode(mode: ScreenMode) {
		currentMode = mode;
	}
</script>

<div class="second-screen" dir={$dir}>
	<!-- Top bar -->
	<div class="screen-toolbar">
		<div class="screen-title">
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
			</svg>
			{#if universeName}
				<span>{universeName} <span class="screen-badge">(Screen 2)</span></span>
			{:else}
				<span>Screen 2</span>
			{/if}
		</div>

		<div class="mode-switcher">
			<button
				class="mode-btn" class:active={currentMode === 'grid'}
				onclick={() => switchMode('grid')}
				title={$t('secondScreen.grid')}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
				</svg>
				<span class="mode-key">G</span>
			</button>
			<button
				class="mode-btn" class:active={currentMode === 'graph'}
				onclick={() => switchMode('graph')}
				title={$t('secondScreen.graph')}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<circle cx="12" cy="12" r="3"/><circle cx="4" cy="6" r="2"/><circle cx="20" cy="6" r="2"/><circle cx="4" cy="18" r="2"/><circle cx="20" cy="18" r="2"/>
					<line x1="6" y1="7" x2="10" y2="10"/><line x1="14" y1="10" x2="18" y2="7"/><line x1="6" y1="17" x2="10" y2="14"/><line x1="14" y1="14" x2="18" y2="17"/>
				</svg>
				<span class="mode-key">E</span>
			</button>
			<button
				class="mode-btn" class:active={currentMode === 'detail'}
				onclick={() => switchMode('detail')}
				title={$t('secondScreen.detail')}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8"/><line x1="7" y1="12" x2="17" y2="12"/><line x1="7" y1="16" x2="13" y2="16"/>
				</svg>
				<span class="mode-key">D</span>
			</button>
		</div>

		<div class="screen-actions">
			<button
				class="linked-btn" class:active={linkedBrowsing}
				onclick={() => linkedBrowsing = !linkedBrowsing}
				title={linkedBrowsing ? $t('secondScreen.linkedOn') : $t('secondScreen.linkedOff')}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					{#if linkedBrowsing}
						<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
						<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
					{:else}
						<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
						<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
						<line x1="2" y1="2" x2="22" y2="22" stroke-width="2.5"/>
					{/if}
				</svg>
			</button>
			<button
				class="close-btn"
				onclick={handleClose}
				title="Close"
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
				</svg>
			</button>
		</div>
	</div>

	<!-- Content area -->
	<div class="screen-content">
		{#if loading}
			<div class="screen-loading">
				<div class="spinner"></div>
				<p>{$t('secondScreen.loading')}</p>
			</div>
		{:else if currentMode === 'grid'}
			<NoteGrid
				notes={allNotes}
				{vaultColorMap}
				onNoteClick={handleGridNoteClick}
				onNoteDoubleClick={handleGridNoteDoubleClick}
			/>
		{:else if currentMode === 'graph'}
			<div class="graph-container">
				<GraphView
					nodes={graphNodes}
					links={graphLinks}
					onNodeClick={handleGraphNodeClick}
				/>
			</div>
		{:else if currentMode === 'detail'}
			{#if $activeTab}
				<div class="detail-container">
					{#if $openTabs.length > 0}
						<div class="detail-tabs">
							{#each $openTabs as tab (tab.id)}
								<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
								<div
									class="detail-tab" class:active={$activeTabId === tab.id}
									onclick={() => switchTab(tab.id)}
								>
									<span class="tab-dot" style="background:{vaultColorMap[tab.vaultName] || '#7c3aed'}"></span>
									<span class="tab-name">{tab.name || $t('tabs.newTab')}</span>
									<button class="tab-close" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}>×</button>
								</div>
							{/each}
						</div>
					{/if}
					<NotePane
						tab={$activeTab}
						isFocused={true}
						onFocus={() => {}}
						color={vaultColorMap[$activeTab.vaultName] || '#7c3aed'}
						allNotes={allNotes}
						{vaultColorMap}
					/>
				</div>
			{:else}
				<div class="detail-empty">
					<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
						<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8"/><line x1="7" y1="12" x2="17" y2="12"/><line x1="7" y1="16" x2="13" y2="16"/>
					</svg>
					<p>{$t('secondScreen.detailEmpty')}</p>
				</div>
			{/if}
		{/if}
	</div>

	<!-- Bottom status bar -->
	<div class="screen-status">
		<span class="status-mode">
			{currentMode === 'grid' ? $t('secondScreen.grid') : currentMode === 'graph' ? $t('secondScreen.graph') : $t('secondScreen.detail')}
		</span>
		<span class="status-count">{allNotes.length} {$t('statusBar.notes')}</span>
		{#if linkedBrowsing}
			<span class="status-linked">{$t('secondScreen.linked')}</span>
		{/if}
	</div>
</div>

<svelte:window onkeydown={(e) => {
	if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
	if (e.key === 'g' || e.key === 'G') { currentMode = 'grid'; e.preventDefault(); }
	if (e.key === 'e' || e.key === 'E') { currentMode = 'graph'; e.preventDefault(); }
	if (e.key === 'd' || e.key === 'D') { currentMode = 'detail'; e.preventDefault(); }
}} />

<style>
	.second-screen {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--background-primary);
		color: var(--text-normal);
		font-family: var(--font-interface-theme, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif);
	}

	/* ─── Toolbar ─── */
	.screen-toolbar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 6px 12px;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		user-select: none;
		-webkit-user-select: none;
	}

	.screen-title {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 600;
		color: var(--text-muted);
		margin-inline-end: auto;
	}
	.screen-badge {
		font-weight: 400;
		opacity: 0.6;
		font-size: 11px;
	}

	.mode-switcher {
		display: flex;
		gap: 2px;
		background: var(--background-secondary-alt);
		border-radius: 6px;
		padding: 2px;
	}

	.mode-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 5px 10px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 11px;
		transition: all 0.15s;
	}
	.mode-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.mode-btn.active { background: var(--interactive-accent); color: white; }

	.mode-key {
		font-size: 10px;
		font-weight: 700;
		opacity: 0.6;
		border: 1px solid currentColor;
		border-radius: 3px;
		padding: 0 3px;
		line-height: 1.4;
	}
	.mode-btn.active .mode-key { border-color: rgba(255,255,255,0.4); }

	.screen-actions { display: flex; gap: 4px; }

	.linked-btn {
		display: flex;
		align-items: center;
		padding: 5px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s;
	}
	.linked-btn:hover { background: var(--background-modifier-hover); }
	.linked-btn.active { color: var(--interactive-accent); }

	.close-btn {
		display: flex;
		align-items: center;
		padding: 5px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s;
	}
	.close-btn:hover { background: var(--text-error); color: white; }

	/* ─── Content ─── */
	.screen-content {
		flex: 1;
		overflow: hidden;
		position: relative;
	}

	.screen-loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
		color: var(--text-muted);
	}

	.spinner {
		width: 28px; height: 28px;
		border: 3px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	.graph-container {
		height: 100%;
		position: relative;
	}

	/* ─── Detail mode ─── */
	.detail-container {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.detail-tabs {
		display: flex;
		gap: 1px;
		padding: 4px 8px 0;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		overflow-x: auto;
	}

	.detail-tab {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border: none;
		border-radius: 6px 6px 0 0;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 12px;
		white-space: nowrap;
		transition: all 0.15s;
	}
	.detail-tab:hover { background: var(--background-modifier-hover); }
	.detail-tab.active {
		background: var(--background-primary);
		color: var(--text-normal);
		border-bottom: 2px solid var(--interactive-accent);
	}

	.tab-dot {
		width: 8px; height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tab-name { max-width: 150px; overflow: hidden; text-overflow: ellipsis; }

	.tab-close {
		border: none;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 14px;
		padding: 0 2px;
		line-height: 1;
		opacity: 0;
		transition: opacity 0.15s;
	}
	.detail-tab:hover .tab-close { opacity: 1; }

	.detail-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
		color: var(--text-faint);
		font-size: 14px;
	}

	/* ─── Status bar ─── */
	.screen-status {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 4px 12px;
		background: var(--background-secondary);
		border-top: 1px solid var(--background-modifier-border);
		font-size: 11px;
		color: var(--text-muted);
	}

	.status-mode {
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.status-linked {
		margin-inline-start: auto;
		display: flex;
		align-items: center;
		gap: 4px;
		color: var(--interactive-accent);
	}
	.status-linked::before {
		content: '';
		width: 6px; height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
</style>

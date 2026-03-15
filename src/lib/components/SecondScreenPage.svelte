<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { dir, t } from '$lib/i18n';
	import {
		libraries, loadLibraries, appSettings, loadSettings, updateSettings,
		loadLibraryAppearance,
		openNoteTab, openTabs, activeTabId, activeTab,
		switchTab, closeTab, createEmptyTab,
		parseFrontmatter, editingTabIds, toggleEditMode,
		navigateBack, navigateForward,
		scanLibraryLinks, scanLibraryTags, scanLibraryIndex,
		buildStarData, readNotePreview,
		type NoteLink, type StarNode, type StarLink, type IndexEntry
	} from '$lib/libraries/store';
	import { get } from 'svelte/store';
	import { detectDir } from '$lib/utils';
	import NotePane from '$lib/components/NotePane.svelte';
	import LocalStarView from '$lib/components/LocalStarView.svelte';
	import {
		onNoteToScreen, onNoteSaved, onUniverseSwitch, onSettingsChanged,
		onStateRequest, onWorkspaceRestore,
		sendNoteToMain, notifyScreenClosed, sendScreenState,
		type ScreenNote, type ScreenMode, type ScreenState
	} from '$lib/secondScreen';
	import {
		setActiveUniverse, listUniverses
	} from '$lib/universe/store';

	// ─── State ───
	let sidebarOpen = $state(true);
	let noteWidth = $state(100); // percentage 50-100
	let skyViewOpen = $state(false);
	// Sky view: RTL note → sky on left, LTR note → sky on right
	let skyViewPosition = $derived.by(() => {
		const tab = $activeTab;
		if (!tab?.content) return 'right';
		const d = detectDir(tab.content);
		return d === 'rtl' ? 'left' : 'right';
	});

	// ─── Data state ───
	let allNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let loading = $state(true);
	let universeName = $state('');

	// ─── Sidebar data ───
	let backlinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let forwardLinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let noteTags = $state<string[]>([]);

	// ─── Sky view (local star) data ───
	let localStarNodes = $state<StarNode[]>([]);
	let localStarLinks = $state<StarLink[]>([]);

	// ─── Library color map ───
	const libraryColors = [
		'#7c3aed', '#3b82f6', '#10b981', '#f59e0b', '#ef4444',
		'#ec4899', '#8b5cf6', '#06b6d4', '#84cc16', '#f97316'
	];
	let libraryColorMap = $derived.by(() => {
		const map: Record<string, string> = {};
		const v = $libraries;
		v.forEach((lib, i) => { map[lib.name] = libraryColors[i % libraryColors.length]; });
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
			await loadLibraries();

			// Load library appearances (fonts, colors) for each library
			for (const lib of $libraries) {
				await loadLibraryAppearance(lib.path, lib.id);
			}

			const libraryList = $libraries;
			const notes: { name: string; path: string; libraryName: string }[] = [];

			for (const lib of libraryList) {
				const libNotes = await (invoke('collect_library_notes', { libraryPath: lib.path }).catch(() => []) as Promise<any[]>);
				notes.push(...libNotes.map((n: any) => ({ name: n.name, path: n.path, libraryName: lib.name })));
			}

			allNotes = notes;
		} finally {
			loading = false;
		}
	}

	// ─── Load sidebar data for active note ───
	async function loadSidebarData() {
		const tab = get(activeTab);
		if (!tab?.path) {
			backlinks = [];
			forwardLinks = [];
			noteTags = [];
			return;
		}

		try {
			// Parse frontmatter for tags
			const fm = parseFrontmatter(tab.content || '');
			const tagProp = fm.properties.find(p => p.key === 'tags');
			noteTags = Array.isArray(tagProp?.value) ? tagProp.value : [];

			// Scan links for the note's library
			const lib = $libraries.find(v => v.name === tab.libraryName);
			if (!lib) return;

			const links = await scanLibraryLinks(lib.path, lib.name).catch(() => [] as NoteLink[]);
			const tabName = tab.name?.toLowerCase() || '';

			// Forward links: links FROM this note
			forwardLinks = links
				.filter(l => l.source_path === tab.path)
				.map(l => {
					// l.target is the link text, find matching note
					const match = allNotes.find(n => n.name.toLowerCase() === l.target.toLowerCase());
					return match || { name: l.target, path: '', libraryName: lib.name };
				})
				.filter(l => l.path) // only include notes that exist
				.filter((v, i, a) => a.findIndex(x => x.path === v.path) === i);

			// Backlinks: links TO this note (where target matches our note name)
			backlinks = links
				.filter(l => l.target.toLowerCase() === tabName)
				.map(l => {
					const match = allNotes.find(n => n.path === l.source_path);
					return match || { name: l.source_path.split(/[\\/]/).pop()?.replace(/\.md$/, '') ?? '', path: l.source_path, libraryName: lib.name };
				})
				.filter((v, i, a) => a.findIndex(x => x.path === v.path) === i);
			// Build local star data for sky view
			const { nodes: starNodes, links: starLinks } = buildStarData(links, allNotes);
			const activeId = tabName;
			const connectedIds = new Set<string>();
			connectedIds.add(activeId);
			for (const link of starLinks) {
				if (link.source === activeId || link.target === activeId) {
					connectedIds.add(link.source);
					connectedIds.add(link.target);
				}
			}
			localStarNodes = starNodes.filter(n => connectedIds.has(n.id));
			localStarLinks = starLinks.filter(l => connectedIds.has(l.source) && connectedIds.has(l.target));
		} catch {
			backlinks = [];
			forwardLinks = [];
			noteTags = [];
			localStarNodes = [];
			localStarLinks = [];
		}
	}

	// ─── Event listeners ───
	let unlisteners: (() => void)[] = [];
	let pendingTimers: ReturnType<typeof setTimeout>[] = [];
	let libraryChangeTimer: ReturnType<typeof setTimeout> | null = null;

	// ─── Apply global font settings (mirrors +layout.svelte logic) ───
	$effect(() => {
		if (typeof document === 'undefined') return;
		const s = $appSettings;
		const root = document.documentElement.style;

		const defaultUI = '-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif';
		const defaultMono = '"Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace';
		const uiFont = s.interfaceFont || defaultUI;
		const txtFont = s.textFont || uiFont;
		const mono = s.monoFont || defaultMono;

		root.setProperty('--font-text-size', s.fontSize + 'px');
		root.setProperty('--font-monospace-theme', mono);

		// Build per-script @font-face rules using unicode-range
		const scripts = s.scriptFonts || {};
		let css = '';
		const ranges: Record<string, string> = {
			arabic: 'U+0600-06FF, U+0750-077F, U+08A0-08FF, U+FB50-FDFF, U+FE70-FEFF',
			hebrew: 'U+0590-05FF, U+FB1D-FB4F',
			cjk: 'U+4E00-9FFF, U+3000-303F, U+30A0-30FF, U+3040-309F, U+AC00-D7AF',
		};

		let hasScriptFont = false;
		for (const [script, range] of Object.entries(ranges)) {
			const fontName = scripts[script];
			if (fontName) {
				hasScriptFont = true;
				css += `@font-face { font-family: "ConstellationUI"; src: local("${fontName}"); unicode-range: ${range}; }\n`;
				css += `@font-face { font-family: "ConstellationText"; src: local("${fontName}"); unicode-range: ${range}; }\n`;
			}
		}

		if (hasScriptFont) {
			root.setProperty('--font-interface-theme', `"ConstellationUI", ${uiFont}`);
			root.setProperty('--font-text-theme', `"ConstellationText", ${txtFont}`);
		} else {
			root.setProperty('--font-interface-theme', uiFont);
			root.setProperty('--font-text-theme', txtFont);
		}

		// Inject or update the style element for script fonts
		let styleEl = document.getElementById('constellation-script-fonts');
		if (!styleEl) {
			styleEl = document.createElement('style');
			styleEl.id = 'constellation-script-fonts';
			document.head.appendChild(styleEl);
		}
		styleEl.textContent = css;
	});

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
			await openNoteTab(note.path, note.libraryName, note.libraryPath, note.libraryColor);
			await loadSidebarData();
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

		// Listen for settings changes (payload contains current settings from main window)
		const u4 = await onSettingsChanged(async (settings) => {
			if (settings && Object.keys(settings).length > 0) {
				appSettings.set({ ...get(appSettings), ...settings });
			} else {
				await loadSettings();
			}
		});
		unlisteners.push(u4);

		// Listen for library file changes
		const u5 = await listen<{ libraryId: string; paths: string[] }>('library-changed', async () => {
			if (libraryChangeTimer) clearTimeout(libraryChangeTimer);
			libraryChangeTimer = setTimeout(() => loadAllData(), 3000);
		});
		unlisteners.push(u5);

		// Listen for state request from main (workspace save)
		const u6 = await onStateRequest(() => {
			const tabs = get(openTabs).map(t => ({
				path: t.path,
				libraryName: t.libraryName,
				libraryColor: t.libraryColor,
			}));
			const currentTab = get(activeTab);
			sendScreenState({
				mode: 'detail',
				linkedBrowsing: true,
				tabs,
				activeTabPath: currentTab?.path ?? null,
			});
		});
		unlisteners.push(u6);

		// Listen for workspace restore from main
		const u7 = await onWorkspaceRestore(async (state: ScreenState) => {
			// Close current tabs and open saved ones
			openTabs.set([]);
			activeTabId.set(null);

			for (const saved of state.tabs) {
				try {
					await openNoteTab(saved.path, saved.libraryName, saved.libraryColor);
				} catch { /* file may not exist anymore */ }
			}

			// Restore active tab
			if (state.activeTabPath) {
				const tabs = get(openTabs);
				const match = tabs.find(t => t.path === state.activeTabPath);
				if (match) {
					activeTabId.set(match.id);
				}
			}
			await loadSidebarData();
		});
		unlisteners.push(u7);
	});

	onDestroy(() => {
		unlisteners.forEach(u => u());
		pendingTimers.forEach(t => clearTimeout(t));
		if (libraryChangeTimer) clearTimeout(libraryChangeTimer);
	});

	// ─── Handlers ───
	async function handleSidebarLinkClick(path: string, libraryName: string) {
		const lib = $libraries.find(v => v.name === libraryName);
		if (!lib) return;
		const color = libraryColorMap[libraryName] || '#7c3aed';
		await openNoteTab(path, libraryName, lib.path, color);
		await loadSidebarData();
	}

	async function handleTabSwitch(tabId: string) {
		switchTab(tabId);
		// Small delay to let activeTab update
		const t = setTimeout(() => loadSidebarData(), 50);
		pendingTimers.push(t);
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

		<div class="screen-actions">
			<div class="width-control" title="Note width: {noteWidth}%">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M21 12H3M21 12l-4-4m4 4l-4 4M3 12l4-4m-4 4l4 4"/>
				</svg>
				<input type="range" class="width-slider" min="50" max="100" step="5" bind:value={noteWidth} />
			</div>
			<div class="sky-controls">
				<button
					class="sky-toggle" class:active={skyViewOpen}
					onclick={() => skyViewOpen = !skyViewOpen}
					title="Toggle sky view"
				>
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<circle cx="12" cy="12" r="2"/><circle cx="5" cy="6" r="1.5"/><circle cx="19" cy="6" r="1.5"/><circle cx="5" cy="18" r="1.5"/><circle cx="19" cy="18" r="1.5"/>
						<line x1="12" y1="12" x2="5" y2="6"/><line x1="12" y1="12" x2="19" y2="6"/><line x1="12" y1="12" x2="5" y2="18"/><line x1="12" y1="12" x2="19" y2="18"/>
					</svg>
				</button>
				</div>
			<button
				class="sidebar-toggle" class:active={sidebarOpen}
				onclick={() => sidebarOpen = !sidebarOpen}
				title="Toggle sidebar"
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="15" y1="3" x2="15" y2="21"/>
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
		{:else if $activeTab}
			<div class="detail-layout">
				<div class="detail-container" style="--note-width: {noteWidth}%">
					{#if $openTabs.length > 0}
						<div class="detail-tabs">
							{#each $openTabs as tab (tab.id)}
								<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
								<div
									class="detail-tab" class:active={$activeTabId === tab.id}
									onclick={() => handleTabSwitch(tab.id)}
								>
									<span class="tab-dot" style="background:{libraryColorMap[tab.libraryName] || '#7c3aed'}"></span>
									<span class="tab-name">{tab.name || $t('tabs.newTab')}</span>
									<button class="tab-close" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}>×</button>
								</div>
							{/each}
						</div>
					{/if}
					<div class="note-and-sky" class:sky-open={skyViewOpen} data-sky-pos={skyViewPosition} style="--sky-size: {Math.min(50, Math.max(20, localStarNodes.length * 4 + 10))}%">
					<div class="note-area">
						<NotePane
							tab={$activeTab}
							isFocused={true}
							onFocus={() => {}}
							color={libraryColorMap[$activeTab.libraryName] || '#7c3aed'}
							allNotes={allNotes}
							{libraryColorMap}
						/>
					</div>
					{#if skyViewOpen && localStarNodes.length > 0}
						<div class="sky-view-panel">
							<LocalStarView
								nodes={localStarNodes}
								links={localStarLinks}
								activeNodeId={$activeTab.name?.replace(/\.md$/, '').toLowerCase() || ''}
								onNodeClick={async (id) => {
									const note = allNotes.find(n => n.name.toLowerCase() === id);
									if (note) await handleSidebarLinkClick(note.path, note.libraryName);
								}}
							/>
						</div>
					{/if}
				</div>
				</div>

				{#if sidebarOpen}
					<div class="screen-sidebar">
						<!-- Backlinks -->
						<div class="sidebar-section">
							<h3 class="sidebar-heading">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/></svg>
								Backlinks
								<span class="sidebar-count">{backlinks.length}</span>
							</h3>
							{#if backlinks.length > 0}
								<ul class="sidebar-links">
									{#each backlinks as link}
										<li>
											<button class="sidebar-link" dir="auto" onclick={() => handleSidebarLinkClick(link.path, link.libraryName)}>
												<span class="link-dot" style="background:{libraryColorMap[link.libraryName] || '#7c3aed'}"></span>
												{link.name}
											</button>
										</li>
									{/each}
								</ul>
							{:else}
								<p class="sidebar-empty">No backlinks</p>
							{/if}
						</div>

						<!-- Forward links -->
						<div class="sidebar-section">
							<h3 class="sidebar-heading">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
								Forward links
								<span class="sidebar-count">{forwardLinks.length}</span>
							</h3>
							{#if forwardLinks.length > 0}
								<ul class="sidebar-links">
									{#each forwardLinks as link}
										<li>
											<button class="sidebar-link" dir="auto" onclick={() => handleSidebarLinkClick(link.path, link.libraryName)}>
												<span class="link-dot" style="background:{libraryColorMap[link.libraryName] || '#7c3aed'}"></span>
												{link.name}
											</button>
										</li>
									{/each}
								</ul>
							{:else}
								<p class="sidebar-empty">No forward links</p>
							{/if}
						</div>

						<!-- Tags -->
						<div class="sidebar-section">
							<h3 class="sidebar-heading">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
								Tags
								<span class="sidebar-count">{noteTags.length}</span>
							</h3>
							{#if noteTags.length > 0}
								<div class="sidebar-tags">
									{#each noteTags as tag}
										<span class="sidebar-tag">#{tag}</span>
									{/each}
								</div>
							{:else}
								<p class="sidebar-empty">No tags</p>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{:else}
			<div class="detail-empty">
				<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
					<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8"/><line x1="7" y1="12" x2="17" y2="12"/><line x1="7" y1="16" x2="13" y2="16"/>
				</svg>
				<p>{$t('secondScreen.detailEmpty')}</p>
			</div>
		{/if}
	</div>

	<!-- Bottom status bar -->
	<div class="screen-status">
		<span class="status-count">{allNotes.length} {$t('statusBar.notes')}</span>
		<span class="status-linked">{$t('secondScreen.linked')}</span>
	</div>
</div>

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

	.screen-actions { display: flex; gap: 4px; }

	.sky-toggle, .sidebar-toggle, .close-btn {
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
	.sky-toggle:hover, .sidebar-toggle:hover { background: var(--background-modifier-hover); }
	.sky-toggle.active, .sidebar-toggle.active { color: var(--interactive-accent); }

	.sky-controls {
		display: flex;
		align-items: center;
		gap: 2px;
	}
	.close-btn:hover { background: var(--text-error); color: white; }

	.width-control {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--text-muted);
		padding: 0 4px;
	}
	.width-slider {
		width: 80px;
		height: 4px;
		accent-color: var(--interactive-accent);
		cursor: pointer;
	}

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

	/* ─── Detail layout ─── */
	.detail-layout {
		display: flex;
		height: 100%;
	}

	.detail-container {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		height: 100%;
		overflow: hidden;
	}
	/* Override NotePane's hardcoded max-width: 800px — controlled by width slider */
	.note-area {
		overflow: hidden !important;
		contain: size layout;
	}
	.note-area :global(*) {
		max-width: 100%;
		box-sizing: border-box;
	}
	.note-area :global(.note-scroll) {
		max-width: 100% !important;
		overflow-x: hidden !important;
	}
	.note-area :global(.note-content) {
		overflow-wrap: break-word;
		word-break: break-word;
	}

	.note-and-sky {
		display: flex;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
		direction: ltr; /* isolate from page RTL so flex-direction works predictably */
	}
	.note-and-sky[data-sky-pos="right"] {
		flex-direction: row;
	}
	.note-and-sky[data-sky-pos="left"] {
		flex-direction: row-reverse;
	}
	.note-area {
		flex: 1;
		min-height: 0;
		min-width: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}
	.sky-view-panel {
		flex: 0 0 var(--sky-size);
		aspect-ratio: 1;
		align-self: center;
		background: var(--background-primary);
		min-width: 120px;
		min-height: 120px;
		max-width: 100%;
		max-height: 100%;
		border: 2px solid var(--background-modifier-border);
		border-radius: 6px;
		margin: 4px;
		overflow: hidden;
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

	/* ─── Right sidebar ─── */
	.screen-sidebar {
		width: 260px;
		flex-shrink: 0;
		border-inline-start: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		overflow-y: auto;
		padding: 8px 0;
	}

	.sidebar-section {
		padding: 8px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.sidebar-section:last-child { border-bottom: none; }

	.sidebar-heading {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin: 0 0 6px;
	}

	.sidebar-count {
		font-weight: 400;
		opacity: 0.6;
		font-size: 0.7rem;
	}

	.sidebar-links {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.sidebar-links li { margin: 1px 0; }

	.sidebar-link {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 4px 6px;
		border: none;
		border-radius: 4px;
		background: none;
		color: var(--interactive-accent);
		font-size: 0.78rem;
		font-family: inherit;
		cursor: pointer;
		text-align: start;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.sidebar-link:hover {
		background: var(--background-modifier-hover);
		text-decoration: underline;
	}

	.link-dot {
		width: 6px; height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.sidebar-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.sidebar-tag {
		font-size: 0.72rem;
		padding: 2px 8px;
		border-radius: 10px;
		background: var(--background-modifier-hover);
		color: var(--text-muted);
	}

	.sidebar-empty {
		font-size: 0.75rem;
		color: var(--text-faint);
		margin: 4px 0 0;
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

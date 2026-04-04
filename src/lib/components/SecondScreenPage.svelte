<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { dir, t, setLocale, type Locale } from '$lib/i18n';
	import {
		libraries, loadLibraries, appSettings, loadSettings,
		loadLibraryAppearance,
		openNoteTab, openTabs, activeTabId, activeTab,
		switchTab, closeTab,
		parseFrontmatter, buildFullContent,
		writeNote, markRecentWrite, setWriteAhead, clearWriteAhead,
		renameItem,
		scanLibraryLinks, scanLibraryTags,
		buildStarData,
		libraryStats, loadAllStats,
		SCRIPT_UNICODE_RANGES, getFontSetById,
		type StarNode, type StarLink
	} from '$lib/libraries/store';
	import { detectDir } from '$lib/utils';
	import { get } from 'svelte/store';
	import NotePane from '$lib/components/NotePane.svelte';
	import DashboardView from '$lib/components/DashboardView.svelte';
	import NotebookNavigator from '$lib/components/NotebookNavigator.svelte';
	import OrgChart from '$lib/components/OrgChart.svelte';
	import LocalStarView from '$lib/components/LocalStarView.svelte';
	import {
		onNoteToScreen, onNoteSaved, onUniverseSwitch, onSettingsChanged,
		onStateRequest, onWorkspaceRestore,
		onContextChanged, onSkyViewHover, onSkyViewClick,
		onSidebarModeChanged, onSplitModeChanged,
		sendNoteToMain, notifyScreenClosed, sendScreenState,
		type ScreenNote, type ScreenMode, type ScreenState, type ContextMode, type SkyViewNodeInfo, type SidebarMode,
		type SplitCompanionData
	} from '$lib/secondScreen';
	import {
		setActiveUniverse, listUniverses, getChildUniverses,
		type ChildUniverseInfo
	} from '$lib/universe/store';
	import { marked } from 'marked';

	function renderMarkdownPreview(raw: string): string {
		const body = raw.replace(/^---[\s\S]*?---\n?/, '').slice(0, 2000);
		try {
			return marked.parse(body, { async: false }) as string;
		} catch {
			return `<p>${body.slice(0, 800)}</p>`;
		}
	}

	// ─── State ───
	let noteWidth = $state(100);
	let mainSidebarMode = $state<SidebarMode>('tree');
	let splitCompanionActive = $state(false);
	let splitCompanionData = $state<SplitCompanionData | null>(null);
	let splitCompanionTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks'>('properties');
	let preSplitSidebarMode = $state<SidebarMode>('tree');

	// ─── Data state ───
	let allNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let loading = $state(true);
	let universeName = $state('');

	// ─── Dashboard state ───
	let childUniverses = $state<ChildUniverseInfo[]>([]);
	let childUniverseLibs = $state<Record<string, { name: string; path: string }[]>>({});
	let dashboardTags = $state<{ tag: string; count: number }[]>([]);
	let selectedTag = $state<string | null>(null);
	let selectedTagNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let loadingTagNotes = $state(false);

	async function selectTag(tag: string) {
		if (selectedTag === tag) {
			selectedTag = null;
			selectedTagNotes = [];
			return;
		}
		selectedTag = tag;
		loadingTagNotes = true;
		const notes: { name: string; path: string; libraryName: string }[] = [];
		try {
			for (const lib of get(libraries)) {
				const results = await invoke<any[]>('notes_by_tag', { libraryPath: lib.path, tag });
				notes.push(...results.map((n: any) => ({ name: n.name, path: n.path, libraryName: n.library_name || lib.name })));
			}
			selectedTagNotes = notes.sort((a, b) => a.name.localeCompare(b.name));
		} catch {
			selectedTagNotes = [];
		}
		loadingTagNotes = false;
	}

	let totalNotes = $derived($libraryStats.reduce((sum: number, s: any) => sum + s.star_count, 0));
	let totalFolders = $derived($libraryStats.reduce((sum: number, s: any) => sum + s.folder_count, 0));
	let cuLibNames = $derived.by(() => {
		const names = new Set<string>();
		for (const libs of Object.values(childUniverseLibs)) {
			for (const lib of libs) names.add(lib.name);
		}
		return names;
	});
	let topLevelStats = $derived($libraryStats.filter((s: any) => !cuLibNames.has(s.name) && !s.is_universe_notes));
	let universeNotesStats = $derived($libraryStats.find((s: any) => s.is_universe_notes) ?? null);

	// ─── Recently opened/edited (read from localStorage, shared with main window) ───
	let recentOpenedRaw = $state<{ name: string; path: string; libraryName: string; openedAt: number }[]>([]);
	let recentEditedRaw = $state<{ name: string; path: string; libraryName: string; editedAt: number }[]>([]);

	function refreshRecentLists() {
		try { recentOpenedRaw = JSON.parse(localStorage.getItem('constellation-recent-opened') || '[]'); } catch { recentOpenedRaw = []; }
		try { recentEditedRaw = JSON.parse(localStorage.getItem('constellation-recent-edited') || '[]'); } catch { recentEditedRaw = []; }
	}

	// Opened = in opened list but NOT in edited list
	let editedPathSet = $derived(new Set(recentEditedRaw.map(n => n.path)));
	let recentlyOpened = $derived(recentOpenedRaw.filter(n => !editedPathSet.has(n.path)).slice(0, 10));
	let recentlyEdited = $derived(recentEditedRaw.slice(0, 10));

	async function loadDashboardData() {
		try {
			await loadAllStats();
			try {
				childUniverses = await getChildUniverses();
				const libMap: Record<string, { name: string; path: string }[]> = {};
				for (const cu of childUniverses) {
					try {
						const libs = await invoke<{ name: string; path: string }[]>('read_child_universe_libraries', { childPath: cu.path });
						libMap[cu.path] = libs.map(l => ({ name: l.name, path: l.path }));
					} catch { libMap[cu.path] = []; }
				}
				childUniverseLibs = libMap;
			} catch { childUniverses = []; childUniverseLibs = {}; }
			// Collect tags from all libraries
			try {
				const merged: Record<string, number> = {};
				for (const lib of get(libraries)) {
					const tags = await scanLibraryTags(lib.path);
					for (const [tag, count] of Object.entries(tags)) {
						merged[tag] = (merged[tag] || 0) + count;
					}
				}
				dashboardTags = Object.entries(merged)
					.map(([tag, count]) => ({ tag, count }))
					.sort((a, b) => b.count - a.count);
			} catch { dashboardTags = []; }
		} catch {}
	}

	// ─── Sky View Companion state ───
	let contextMode = $state<ContextMode>('editor');
	let skyviewNode = $state<SkyViewNodeInfo | null>(null);
	let pinnedSkyviewNode = $state<SkyViewNodeInfo | null>(null);
	let isHoverPreview = $state(false);

	// Peek preview: full editable note in left panel
	let peekNote = $state<{ name: string; path: string; libraryName: string; libraryColor: string } | null>(null);
	let peekTab = $state<import('$lib/libraries/store').OpenTab | null>(null);
	let peekGeneration = 0;

	// Skyview companion data
	let skyviewPreview = $state('');
	let skyviewBacklinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let skyviewForwardLinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let skyviewTags = $state<string[]>([]);
	let skyviewLocalNodes = $state<StarNode[]>([]);
	let skyviewLocalLinks = $state<StarLink[]>([]);
	let skyviewGeneration = 0;

	// Navigation history for sky view companion
	let skyviewHistory = $state<SkyViewNodeInfo[]>([]);
	let skyviewHistoryIdx = $state(-1);
	let isNavigatingHistory = false;

	function pushSkyviewHistory(node: SkyViewNodeInfo) {
		if (isNavigatingHistory) return;
		skyviewHistory = [...skyviewHistory.slice(0, skyviewHistoryIdx + 1), node];
		skyviewHistoryIdx = skyviewHistory.length - 1;
	}

	function canGoBack() { return skyviewHistoryIdx > 0; }
	function canGoForward() { return skyviewHistoryIdx < skyviewHistory.length - 1; }

	async function goBack() {
		if (!canGoBack()) return;
		isNavigatingHistory = true;
		skyviewHistoryIdx--;
		const node = skyviewHistory[skyviewHistoryIdx];
		pinnedSkyviewNode = node;
		isHoverPreview = false;
		await loadSkyViewCompanionData(node);
		isNavigatingHistory = false;
	}

	async function goForward() {
		if (!canGoForward()) return;
		isNavigatingHistory = true;
		skyviewHistoryIdx++;
		const node = skyviewHistory[skyviewHistoryIdx];
		pinnedSkyviewNode = node;
		isHoverPreview = false;
		await loadSkyViewCompanionData(node);
		isNavigatingHistory = false;
	}

	async function loadSkyViewCompanionData(node: SkyViewNodeInfo) {
		const gen = ++skyviewGeneration;
		skyviewNode = node;

		try {
			const content = await invoke<string>('read_note', { notePath: node.path });
			if (gen !== skyviewGeneration) return;
			skyviewPreview = content.slice(0, 2000);
			const fm = parseFrontmatter(content);
			const tagProp = fm.properties.find(p => p.key === 'tags');
			skyviewTags = tagProp?.listItems ?? [];
		} catch {
			if (gen !== skyviewGeneration) return;
			skyviewPreview = '';
			skyviewTags = [];
		}

		const lib = $libraries.find(v => v.name === node.libraryName);
		if (lib) {
			try {
				const links = await scanLibraryLinks(lib.path, lib.name);
				if (gen !== skyviewGeneration) return;
				const nodeName = node.name.toLowerCase();

				skyviewForwardLinks = links
					.filter(l => l.source_name.toLowerCase() === nodeName)
					.map(l => {
						const target = allNotes.find(n => n.name.replace(/\.md$/, '').toLowerCase() === l.target.toLowerCase());
						return target ? { name: target.name, path: target.path, libraryName: target.libraryName } : null;
					})
					.filter(Boolean) as { name: string; path: string; libraryName: string }[];

				skyviewBacklinks = links
					.filter(l => l.target.toLowerCase() === nodeName)
					.map(l => {
						const src = allNotes.find(n => n.path === l.source_path);
						return src ? { name: src.name, path: src.path, libraryName: src.libraryName } : null;
					})
					.filter(Boolean) as { name: string; path: string; libraryName: string }[];

				const { nodes: sn, links: sl } = buildStarData(links, allNotes.map(n => ({
					id: n.name.replace(/\.md$/, '').toLowerCase(),
					name: n.name,
					path: n.path,
					libraryName: n.libraryName,
					linkCount: 0,
					outgoingCount: 0,
				})));
				const connectedIds = new Set<string>([nodeName]);
				for (const link of sl) {
					if (link.source === nodeName || link.target === nodeName) {
						connectedIds.add(link.source);
						connectedIds.add(link.target);
					}
				}
				skyviewLocalNodes = sn.filter(n => connectedIds.has(n.id));
				skyviewLocalLinks = sl.filter(l => connectedIds.has(l.source) && connectedIds.has(l.target));
			} catch {
				if (gen !== skyviewGeneration) return;
				skyviewBacklinks = [];
				skyviewForwardLinks = [];
				skyviewLocalNodes = [];
				skyviewLocalLinks = [];
			}
		}
	}

	async function loadPeekPreview(note: { name: string; path: string; libraryName: string; libraryColor: string }) {
		const gen = ++peekGeneration;
		peekNote = note;
		try {
			const content = await invoke<string>('read_note', { filePath: note.path });
			if (gen !== peekGeneration) return;
			const lib = $libraries.find(v => v.name === note.libraryName);
			peekTab = {
				id: `peek-${note.path}`,
				path: note.path,
				content,
				libraryName: note.libraryName,
				libraryPath: lib?.path ?? '',
				name: note.name.endsWith('.md') ? note.name : note.name + '.md',
				libraryColor: note.libraryColor,
				history: [note.path],
				historyIndex: 0,
			};
		} catch {
			if (gen !== peekGeneration) return;
			peekTab = null;
		}
	}

	function closePeek() {
		peekNote = null;
		peekTab = null;
	}

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

	// ─── Apply global font settings (mirrors +layout.svelte) ───
	$effect(() => {
		if (typeof document === 'undefined') return;
		const s = $appSettings;
		const root = document.documentElement.style;
		const defaultUI = '-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, "Noto Sans Arabic", "Noto Sans Hebrew", "Noto Sans CJK SC", sans-serif';
		const defaultMono = '"Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace';

		root.setProperty('--font-text-size', s.fontSize + 'px');
		root.setProperty('--font-ui-size', (s.interfaceFontSize || 14) + 'px');
		document.documentElement.style.fontSize = (s.interfaceFontSize || 14) + 'px';
		const fontMode = s.fontMode || 'per-language';
		let css = '';

		if (fontMode === 'universal') {
			const set = getFontSetById(s.activeFontSetId || 'system', s.customFontSets || []);
			const uiFont = set?.interfaceFont || s.interfaceFont || defaultUI;
			const txtFont = set?.textFont || s.textFont || uiFont;
			const mono = set?.monoFont || s.monoFont || defaultMono;
			root.setProperty('--font-interface-theme', uiFont);
			root.setProperty('--font-text-theme', txtFont);
			root.setProperty('--font-monospace-theme', mono);
		} else {
			const langSets = s.languageFontSets || {};
			const customSets = s.customFontSets || [];
			let hasPerScript = false;
			const latinSet = getFontSetById(langSets.latin || 'system', customSets);
			const baseUI = latinSet?.interfaceFont || s.interfaceFont || defaultUI;
			const baseTxt = latinSet?.textFont || s.textFont || baseUI;
			const baseMono = latinSet?.monoFont || s.monoFont || defaultMono;
			root.setProperty('--font-monospace-theme', baseMono);

			for (const [script, range] of Object.entries(SCRIPT_UNICODE_RANGES)) {
				if (script === 'latin') continue;
				const setId = langSets[script];
				if (!setId || setId === 'system') continue;
				const set = getFontSetById(setId, customSets);
				if (!set) continue;
				hasPerScript = true;
				if (set.interfaceFont) {
					css += `@font-face { font-family: "ConstellationUI"; src: local("${set.interfaceFont.split(',')[0].trim().replace(/"/g, '')}"); unicode-range: ${range}; }\n`;
				}
				if (set.textFont) {
					css += `@font-face { font-family: "ConstellationText"; src: local("${set.textFont.split(',')[0].trim().replace(/"/g, '')}"); unicode-range: ${range}; }\n`;
				}
			}

			if (hasPerScript) {
				root.setProperty('--font-interface-theme', `"ConstellationUI", ${baseUI}`);
				root.setProperty('--font-text-theme', `"ConstellationText", ${baseTxt}`);
			} else {
				root.setProperty('--font-interface-theme', baseUI);
				root.setProperty('--font-text-theme', baseTxt);
			}
		}

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

	// ─── Event listeners ───
	let unlisteners: (() => void)[] = [];
	let libraryChangeTimer: ReturnType<typeof setTimeout> | null = null;

	onMount(async () => {
		const win = getCurrentWindow();
		try { await win.setTitle('Constellation'); } catch {}

		const unlistenHidden = await listen('screen-hidden', () => {
			notifyScreenClosed();
		});
		unlisteners.push(unlistenHidden);

		// Load recent lists from localStorage and listen for changes
		refreshRecentLists();
		const handleStorage = (e: StorageEvent) => {
			if (e.key === 'constellation-recent-opened' || e.key === 'constellation-recent-edited') {
				refreshRecentLists();
			}
		};
		window.addEventListener('storage', handleStorage);
		unlisteners.push(() => window.removeEventListener('storage', handleStorage));

		// Also poll periodically since storage events don't fire for same-origin same-window writes
		const recentPollTimer = setInterval(refreshRecentLists, 2000);
		unlisteners.push(() => clearInterval(recentPollTimer));

		// Listen for notes sent from main window
		const u1 = await onNoteToScreen(async (note) => {
			// No-op for tracking (localStorage handles it), but keep listener for other modes
		});
		unlisteners.push(u1);

		// Listen for note saves
		const u2 = await onNoteSaved(async (path) => {
			refreshRecentLists();
			const tab = get(activeTab);
			if (tab?.path === path) {
				try {
					const content = await invoke<string>('read_note', { notePath: tab.path });
					tab.content = content;
				} catch {}
			}
		});
		unlisteners.push(u2);

		// Listen for universe switch
		const u3 = await onUniverseSwitch(async () => {
			try {
				const universes = await listUniverses();
				if (universes.length > 0) {
					universeName = universes[0].name || '';
					getCurrentWindow().setTitle(`Constellation - ${universeName}`).catch(() => {});
				}
			} catch {}
			await loadAllData();
			await loadDashboardData();
		});
		unlisteners.push(u3);

		// Listen for settings changes
		const u4 = await onSettingsChanged(async (settings) => {
			if (settings && Object.keys(settings).length > 0) {
				if (settings.locale) {
					setLocale(settings.locale as Locale);
				}
				appSettings.set({ ...get(appSettings), ...settings });
			} else {
				await loadSettings();
			}
		});
		unlisteners.push(u4);

		// Listen for library file changes
		const u5 = await listen<{ libraryId: string; paths: string[] }>('library-changed', async () => {
			if (libraryChangeTimer) clearTimeout(libraryChangeTimer);
			libraryChangeTimer = setTimeout(async () => { await loadAllData(); await loadDashboardData(); }, 3000);
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
			openTabs.set([]);
			activeTabId.set(null);
			for (const saved of state.tabs) {
				try {
					await openNoteTab(saved.path, saved.libraryName, saved.libraryColor);
				} catch {}
			}
			if (state.activeTabPath) {
				const tabs = get(openTabs);
				const match = tabs.find(t => t.path === state.activeTabPath);
				if (match) activeTabId.set(match.id);
			}
		});
		unlisteners.push(u7);

		// Sky View companion: listen for context mode changes
		const u8 = await onContextChanged((mode) => {
			contextMode = mode;
			if (mode !== 'skyview') {
				skyviewNode = null;
			}
		});
		unlisteners.push(u8);

		// Sky View companion: listen for hover events
		const u9 = await onSkyViewHover(async (node) => {
			if (contextMode !== 'skyview') return;
			if (!node) {
				isHoverPreview = false;
				if (pinnedSkyviewNode) {
					await loadSkyViewCompanionData(pinnedSkyviewNode);
				} else {
					skyviewNode = null;
				}
				return;
			}
			isHoverPreview = true;
			await loadSkyViewCompanionData(node);
		});
		unlisteners.push(u9);

		// Sky View companion: listen for click events
		const u10 = await onSkyViewClick(async (node) => {
			if (contextMode !== 'skyview') return;
			isHoverPreview = false;
			pinnedSkyviewNode = node;
			pushSkyviewHistory(node);
			await loadSkyViewCompanionData(node);
		});
		unlisteners.push(u10);

		// Listen for sidebar mode changes from main window
		const u11 = await onSidebarModeChanged((mode) => {
			mainSidebarMode = mode;
			if (mode === 'tree') loadDashboardData();
		});
		unlisteners.push(u11);

		const u12 = await onSplitModeChanged((data) => {
			if (data.active) {
				if (!splitCompanionActive) preSplitSidebarMode = mainSidebarMode;
				splitCompanionActive = true;
				splitCompanionData = data;
			} else {
				splitCompanionActive = false;
				splitCompanionData = null;
				mainSidebarMode = preSplitSidebarMode;
			}
		});
		unlisteners.push(u12);

		// Now load data (after listeners are set up so no events are missed)
		try {
			const universes = await listUniverses();
			if (universes.length > 0) {
				await setActiveUniverse(universes[0].id);
				universeName = universes[0].name || '';
				await win.setTitle(`Constellation - ${universeName}`).catch(() => {});
			}
		} catch {}

		await loadAllData();
		await loadDashboardData();
	});

	onDestroy(() => {
		unlisteners.forEach(u => u());
		if (libraryChangeTimer) clearTimeout(libraryChangeTimer);
	});

	// ─── Handlers ───
	async function handleTabSwitch(tabId: string) {
		switchTab(tabId);
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
			{#if mainSidebarMode !== 'tree'}
				<div class="width-control" title="Note width: {noteWidth}%">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M21 12H3M21 12l-4-4m4 4l-4 4M3 12l4-4m-4 4l4 4"/>
					</svg>
					<input type="range" class="width-slider" min="50" max="100" step="5" bind:value={noteWidth} />
				</div>
			{/if}
			<button class="close-btn" onclick={handleClose} title="Close">
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
		{:else if splitCompanionActive && splitCompanionData}
			<!-- Split View Panels Companion -->
			<div class="split-companion">
				<div class="split-companion-header">
					<span class="split-companion-label">{$t('secondScreen.splitCompanion') || 'Panels Companion'}</span>
					{#if splitCompanionData.noteName}
						<span class="split-companion-note" dir="auto">{splitCompanionData.noteName.replace(/\.md$/, '')}</span>
					{/if}
				</div>
				<div class="split-companion-tabs">
					{#each [
						{ id: 'properties', icon: '⚙', label: $t('panels.properties') || 'Properties' },
						{ id: 'backlinks', icon: '🔗', label: $t('panels.backlinks') || 'Backlinks' },
						{ id: 'tags', icon: '🏷', label: $t('panels.tags') || 'Tags' },
						{ id: 'star', icon: '⭐', label: $t('panels.starView') || 'Star' },
						{ id: 'tasks', icon: '☑', label: $t('panels.tasks') || 'Tasks' },
					] as tab}
						<button class="sc-tab" class:active={splitCompanionTab === tab.id}
							onclick={() => splitCompanionTab = tab.id as any}>
							{tab.icon} {tab.label}
						</button>
					{/each}
				</div>
				<div class="split-companion-body">
					{#if splitCompanionData.notePath}
						{#if splitCompanionTab === 'properties'}
							{@const parsed = parseFrontmatter(splitCompanionData.content || '')}
							<div class="sc-panel">
								<div class="sc-props-list">
									{#each parsed.properties as prop}
										<div class="sc-prop">
											<span class="sc-prop-key">{prop.key}</span>
											<span class="sc-prop-val">{prop.value}</span>
										</div>
									{/each}
									{#if parsed.properties.length === 0}
										<p class="sc-empty">{$t('secondScreen.noProperties') || 'No properties'}</p>
									{/if}
								</div>
							</div>
						{:else if splitCompanionTab === 'backlinks'}
							<div class="sc-panel">
								<p class="sc-info">{$t('secondScreen.backlinksFor') || 'Backlinks for'} <strong>{splitCompanionData.noteName?.replace(/\.md$/, '')}</strong></p>
								<!-- Backlinks loaded from library scan -->
							</div>
						{:else if splitCompanionTab === 'tags'}
							{@const parsed = parseFrontmatter(splitCompanionData.content || '')}
							{@const tags = parsed.properties.filter(p => p.key.toLowerCase() === 'tags').map(p => String(p.value)).flatMap(v => v.split(',').map(t => t.trim())).filter(Boolean)}
							<div class="sc-panel">
								{#if tags.length > 0}
									<div class="sc-tags">
										{#each tags as tag}
											<span class="sc-tag">#{tag}</span>
										{/each}
									</div>
								{:else}
									<p class="sc-empty">{$t('panels.noTags') || 'No tags'}</p>
								{/if}
							</div>
						{:else if splitCompanionTab === 'star'}
							<div class="sc-panel sc-star-panel">
								<LocalStarView
									nodes={[]}
									links={[]}
									activeNodeId={splitCompanionData.notePath}
									onNodeClick={(id) => {
										const note = allNotes.find(n => n.path === id);
										if (note) sendNoteToMain({ path: note.path, name: note.name, libraryName: note.libraryName, libraryPath: '', libraryColor: libraryColorMap[note.libraryName] || '#7c3aed' });
									}}
								/>
							</div>
						{:else if splitCompanionTab === 'tasks'}
							<div class="sc-panel">
								<p class="sc-empty">{$t('panels.noTasks') || 'No tasks in this note'}</p>
							</div>
						{/if}
					{:else}
						<p class="sc-empty">{$t('panels.noNoteSelected') || 'No note selected'}</p>
					{/if}
				</div>
			</div>

		{:else if contextMode === 'skyview'}
			<!-- Sky View Companion — kept as-is -->
			<div class="skyview-companion">
				{#if skyviewNode}
					<div class="skyview-layout">
						<div class="skyview-peek-area">
							{#if peekTab}
								<div class="peek-preview">
									<div class="peek-header">
										<span class="skyview-dot" style="background:{peekNote?.libraryColor}"></span>
										<h3 class="peek-name" dir="auto">{peekNote?.name?.replace(/\.md$/, '')}</h3>
										<button class="peek-close" onclick={closePeek} title="Close">
											<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
										</button>
									</div>
									<div class="peek-editor">
										{#if peekTab.path}
											{@const _pp = parseFrontmatter(peekTab.content || '')}
											{@const _pbody = _pp.body}
											{@const _pdir = detectDir(_pbody) || $dir}
											{@const _pGuard = { saving: false }}
											{#key peekTab.id + '|' + peekTab.path}
											<NotePane
												value={_pbody}
												title={peekTab.name.replace(/\.md$/, '')}
												dir={_pdir}
												libraryName={peekTab.libraryName}
												tabId={peekTab.id}
												filePath={peekTab.path}
												libraryPath={peekTab.libraryPath || ''}
												noteNames={allNotes}
												allTags={[]}
												properties={_pp.properties}
												rawYaml={_pp.rawYaml ?? ''}
												stage={_pp.properties.find(p => p.key.toLowerCase() === 'stage')?.value ?? ''}
												onchange={() => {}}
												onsave={(text) => {
													if (_pGuard.saving) return;
													_pGuard.saving = true;
													const pr = parseFrontmatter(peekTab?.content || '').properties;
													markRecentWrite(peekTab!.path);
													const content = buildFullContent(pr, text);
													writeNote(peekTab!.path, content).catch(() => {}).finally(() => { _pGuard.saving = false; });
												}}
												onflush={(text, needsDiskSave, cursorPos, scrollTop) => {
													const pr = parseFrontmatter(peekTab?.content || '').properties;
													const content = buildFullContent(pr, text);
													if (peekTab) { peekTab.content = content; }
													if (needsDiskSave) {
														markRecentWrite(peekTab!.path);
														writeNote(peekTab!.path, content).catch(() => {});
													}
												}}
												ontitlechange={(newTitle) => {
													if (peekTab && newTitle !== peekTab.name.replace(/\.md$/, '')) {
														renameItem(peekTab.path, peekTab.path.replace(/[^/\\]+$/, newTitle + '.md'));
													}
												}}
												onpropschange={() => {}}
											/>
											{/key}
										{/if}
									</div>
								</div>
							{:else}
								<div class="peek-empty">
									<p class="sidebar-empty">{$t('secondScreen.skyviewHint') || 'Click a link to preview it here'}</p>
								</div>
							{/if}
						</div>

						<div class="skyview-detail">
							<div class="skyview-nav">
								<button class="skyview-nav-btn" disabled={!canGoBack()} onclick={goBack} title="Back">
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
								</button>
								<button class="skyview-nav-btn" disabled={!canGoForward()} onclick={goForward} title="Forward">
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
								</button>
								<span class="skyview-nav-count">{skyviewHistoryIdx + 1}/{skyviewHistory.length}</span>
							</div>
							<div class="skyview-header">
								<span class="skyview-dot" style="background:{skyviewNode.libraryColor}"></span>
								<h2 class="skyview-name" dir="auto">{skyviewNode.name}</h2>
								<span class="skyview-lib">{skyviewNode.libraryName}</span>
								{#if !isHoverPreview && pinnedSkyviewNode}
									<span class="skyview-pinned" title="Pinned">📌</span>
								{/if}
							</div>

							{#if skyviewPreview}
								<div class="skyview-preview markdown-rendered" dir="auto">
									{@html renderMarkdownPreview(skyviewPreview)}
								</div>
							{/if}

							<div class="sidebar-section">
								<h3 class="sidebar-heading">
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/></svg>
									Backlinks <span class="sidebar-count">{skyviewBacklinks.length}</span>
								</h3>
								{#if skyviewBacklinks.length > 0}
									<ul class="sidebar-links">
										{#each skyviewBacklinks as link}
											<li>
												<button class="sidebar-link" dir="auto" onclick={() => {
													loadPeekPreview({ name: link.name.replace(/\.md$/, ''), path: link.path, libraryName: link.libraryName, libraryColor: libraryColorMap[link.libraryName] || '#7c3aed' });
												}}>
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

							<div class="sidebar-section">
								<h3 class="sidebar-heading">
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
									Forward links <span class="sidebar-count">{skyviewForwardLinks.length}</span>
								</h3>
								{#if skyviewForwardLinks.length > 0}
									<ul class="sidebar-links">
										{#each skyviewForwardLinks as link}
											<li>
												<button class="sidebar-link" dir="auto" onclick={() => {
													loadPeekPreview({ name: link.name.replace(/\.md$/, ''), path: link.path, libraryName: link.libraryName, libraryColor: libraryColorMap[link.libraryName] || '#7c3aed' });
												}}>
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

							{#if skyviewTags.length > 0}
								<div class="sidebar-section">
									<h3 class="sidebar-heading">
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
										Tags <span class="sidebar-count">{skyviewTags.length}</span>
									</h3>
									<div class="sidebar-tags">
										{#each skyviewTags as tag}
											<span class="sidebar-tag">#{tag}</span>
										{/each}
									</div>
								</div>
							{/if}
						</div>

						{#if skyviewLocalNodes.length > 0}
							<div class="skyview-graph">
								<LocalStarView
									nodes={skyviewLocalNodes}
									links={skyviewLocalLinks}
									activeNodeId={skyviewNode.name.replace(/\.md$/, '').toLowerCase()}
									onNodeClick={async (id) => {
										const note = allNotes.find(n => n.name.replace(/\.md$/, '').toLowerCase() === id);
										if (note) {
											const lib = $libraries.find(v => v.name === note.libraryName);
											await loadSkyViewCompanionData({
												path: note.path,
												name: note.name,
												libraryName: note.libraryName,
												libraryPath: lib?.path ?? '',
												libraryColor: libraryColorMap[note.libraryName] ?? '#7c3aed',
											});
										}
									}}
								/>
							</div>
						{/if}
					</div>
				{:else}
					<div class="detail-empty">
						<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
							<circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
						</svg>
						<p>{$t('secondScreen.skyviewHint') || 'Hover over a node in Sky View to see its details here'}</p>
					</div>
				{/if}
			</div>

		{:else if mainSidebarMode === 'list'}
			<!-- Notes Navigator companion -->
			<div class="navigator-fullscreen">
				<NotebookNavigator
					mode="second"
					{libraryColorMap}
					onNoteClick={(path, name, lib) => {
						const libObj = $libraries.find(v => v.name === lib);
						const color = libraryColorMap[lib] || '#7c3aed';
						openNoteTab(path, lib, color);
					}}
					onNoteDoubleClick={(path, name, lib) => {
						sendNoteToMain({ path, name: name + '.md', libraryName: lib, libraryPath: '', libraryColor: '' });
					}}
				/>
			</div>

		{:else if mainSidebarMode === 'skyview'}
			<!-- OrgChart companion -->
			<div class="navigator-fullscreen">
				<OrgChart
					{libraryColorMap}
					universeName={universeName}
					embedded={true}
					onNoteClick={(path, name) => {
						const note = allNotes.find(n => n.path === path);
						const lib = note ? $libraries.find(v => v.name === note.libraryName) : null;
						const color = note ? libraryColorMap[note.libraryName] || '#7c3aed' : '#7c3aed';
						openNoteTab(path, note?.libraryName ?? '', color);
					}}
				/>
			</div>

		{:else if mainSidebarMode === 'tree'}
			<!-- File Explorer companion — Universe Dashboard -->
			<DashboardView
				{universeName}
				{libraryColorMap}
				onNoteClick={(path, name, libraryName) => {
					openNoteTab(path, libraryName, libraryColorMap[libraryName] || '#7c3aed');
				}}
				onNoteToMain={(note) => {
					sendNoteToMain(note);
				}}
			/>

		{:else if $activeTab}
			<!-- Fallback note companion -->
			<div class="note-companion">
				{#if $openTabs.length > 0}
					<div class="detail-tabs">
						{#each $openTabs as tab (tab.id)}
							<button
								class="detail-tab"
								class:active={$activeTabId === tab.id}
								onclick={() => handleTabSwitch(tab.id)}
								dir="auto"
							>
								<span class="tab-dot" style="background:{tab.libraryColor || '#7c3aed'}"></span>
								<span class="tab-name">{tab.name?.replace(/\.md$/, '')}</span>
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<span class="tab-close" role="button" tabindex="-1" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }} onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); closeTab(tab.id); } }}>×</span>
							</button>
						{/each}
					</div>
				{/if}
				<div class="note-area" style="--note-width:{noteWidth}%">
					{#if $activeTab?.path}
						{@const _dp = parseFrontmatter($activeTab.content || '')}
						{@const _dbody = _dp.body}
						{@const _ddir = detectDir(_dbody) || $dir}
						{@const _dGuard = { saving: false }}
						{#key $activeTab.id + '|' + $activeTab.path}
						<NotePane
							value={_dbody}
							title={$activeTab.name.replace(/\.md$/, '')}
							dir={_ddir}
							initialCursorPos={$activeTab.cursorPos ?? 0}
							initialScrollTop={$activeTab.scrollTop ?? 0}
							libraryName={$activeTab.libraryName}
							tabId={$activeTab.id}
							filePath={$activeTab.path}
							libraryPath={$activeTab.libraryPath || ''}
							noteNames={allNotes}
							allTags={[]}
							properties={_dp.properties}
							rawYaml={_dp.rawYaml ?? ''}
							stage={_dp.properties.find(p => p.key.toLowerCase() === 'stage')?.value ?? ''}
							onchange={() => {}}
							onpromote={(nextStage) => {
								const ct = get(activeTab);
								if (!ct) return;
								const pr = parseFrontmatter(ct.content || '').properties;
								const bd = parseFrontmatter(ct.content || '').body;
								let np;
								if (!nextStage) { np = pr.filter(p => p.key.toLowerCase() !== 'stage'); }
								else {
									let u = false;
									np = pr.map(p => { if (p.key.toLowerCase() === 'stage') { u = true; return { ...p, value: nextStage }; } return p; });
									if (!u) np.push({ key: 'stage', value: nextStage, type: 'text' as any });
								}
								const fc = buildFullContent(np, bd);
								ct.content = fc;
								openTabs.update(tabs => tabs);
								markRecentWrite(ct.path);
								writeNote(ct.path, fc).catch(() => {});
							}}
							onsave={(text) => {
								if (_dGuard.saving) return;
								_dGuard.saving = true;
								const ct = get(activeTab);
								if (!ct) { _dGuard.saving = false; return; }
								const pr = parseFrontmatter(ct.content || '').properties;
								markRecentWrite(ct.path);
								const content = buildFullContent(pr, text);
								writeNote(ct.path, content).catch(() => {}).finally(() => { _dGuard.saving = false; });
							}}
							onflush={(text, needsDiskSave, cursorPos, scrollTop) => {
								const ct = get(activeTab);
								if (!ct) return;
								const pr = parseFrontmatter(ct.content || '').properties;
								const content = buildFullContent(pr, text);
								ct.content = content;
								ct.cursorPos = cursorPos;
								ct.scrollTop = scrollTop;
								setWriteAhead(ct.path, content, cursorPos, scrollTop);
								if (needsDiskSave) {
									markRecentWrite(ct.path);
									writeNote(ct.path, content).then(() => clearWriteAhead(ct.path)).catch(() => {});
								}
							}}
							ontitlechange={(newTitle) => {
								const ct = get(activeTab);
								if (ct && newTitle !== ct.name.replace(/\.md$/, '')) {
									renameItem(ct.path, ct.path.replace(/[^/\\]+$/, newTitle + '.md'));
								}
							}}
							onpropschange={() => { openTabs.update(tabs => tabs); }}
						/>
						{/key}
					{/if}
				</div>
			</div>

		{:else}
			<div class="detail-empty">
				<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
					<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8"/><line x1="7" y1="12" x2="17" y2="12"/><line x1="7" y1="16" x2="13" y2="16"/>
				</svg>
				<p>{$t('secondScreen.selectNote') || 'Select a note in the main window to preview it here'}</p>
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
	/* ─── Dashboard ─── */
	.dashboard-companion { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
	.dashboard-scroll { flex: 1; overflow-y: auto; padding: 24px 28px; }

	.dashboard-header { display: flex; align-items: center; gap: 10px; margin-bottom: 24px; }
	.dashboard-header h2 { font-size: 20px; font-weight: 700; color: var(--text-normal); margin: 0; }

	.dashboard-stats {
		display: grid; grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
		gap: 12px; margin-bottom: 28px;
	}
	.stat-card {
		display: flex; flex-direction: column; align-items: center; gap: 4px;
		padding: 16px 12px; border-radius: 10px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
	}
	.stat-value { font-size: 28px; font-weight: 700; color: var(--interactive-accent); line-height: 1; }
	.stat-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; text-align: center; }

	.dashboard-section { margin-bottom: 24px; }
	.dashboard-section-title {
		font-size: 12px; font-weight: 600; color: var(--text-muted);
		text-transform: uppercase; letter-spacing: 0.5px;
		margin: 0 0 12px 0; padding-bottom: 8px;
		border-bottom: 1px solid var(--background-modifier-border);
	}

	.library-list { display: flex; flex-direction: column; gap: 10px; }
	.library-card {
		background: var(--background-secondary);
		border-radius: 10px; padding: 10px 14px;
		border: 1px solid var(--background-modifier-border);
	}
	.library-card-header {
		display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
	}
	.lib-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
	.lib-name { flex: 1; font-size: 14px; font-weight: 600; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.library-card-stats { display: flex; gap: 8px; }
	.lib-stat-box {
		flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px;
		padding: 8px 6px; border-radius: 8px;
		background: color-mix(in srgb, var(--lib-color) 8%, var(--background-primary));
		border: 1px solid color-mix(in srgb, var(--lib-color) 18%, transparent);
	}
	.lib-stat-value { font-size: 18px; font-weight: 700; color: var(--lib-color); line-height: 1; }
	.lib-stat-label { font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }

	.recent-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }
	.recent-column { min-width: 0; }
	.recent-empty { color: var(--text-faint); font-size: 13px; padding: 8px 12px; margin: 0; }
	.recent-list { display: flex; flex-direction: column; gap: 2px; }
	.recent-note {
		display: flex; align-items: center; gap: 10px;
		padding: 8px 12px; border-radius: 8px; border: none; width: 100%;
		background: transparent; color: var(--text-normal); cursor: pointer;
		text-align: start; font-family: inherit; transition: background 0.15s;
	}
	.recent-note:hover { background: var(--background-modifier-hover); }
	.recent-name { flex: 1; font-size: 14px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
	.recent-time { font-size: 11px; color: var(--text-faint); white-space: nowrap; margin-inline-start: auto; }

	.cu-list { display: flex; flex-direction: column; gap: 8px; }
	.cu-group {
		background: var(--background-secondary);
		border-radius: 10px; overflow: hidden;
		border: 1px solid var(--background-modifier-border);
	}
	.cu-header {
		display: flex; align-items: center; gap: 10px;
		padding: 10px 14px 0;
	}
	.cu-name { flex: 1; font-size: 14px; font-weight: 600; color: var(--text-normal); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cu-stat-boxes { display: flex; gap: 8px; padding: 10px 14px; }
	.cu-stat-box {
		flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px;
		padding: 8px 6px; border-radius: 8px;
		background: color-mix(in srgb, #6366f1 8%, var(--background-primary));
		border: 1px solid color-mix(in srgb, #6366f1 18%, transparent);
	}
	.cu-stat-value { font-size: 18px; font-weight: 700; color: #6366f1; line-height: 1; }
	.cu-stat-label { font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.4px; }
	.cu-libs {
		padding: 0 14px 10px; margin-top: 0;
		display: flex; flex-direction: column; gap: 8px;
	}

	.tags-layout { display: block; }
	.tags-layout.tags-split { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
	.tags-list-col { min-width: 0; }
	.tags-notes-col {
		min-width: 0; border-left: 1px solid var(--background-modifier-border);
		padding-left: 20px; font-family: var(--font-interface-theme);
	}
	.tags-notes-title {
		display: flex; align-items: center; gap: 8px;
		margin: 0 0 10px 0; font-size: 13px; font-weight: 600; color: var(--text-normal);
	}
	.tags-notes-close {
		margin-left: auto; background: none; border: none; cursor: pointer;
		color: var(--text-muted); padding: 2px; border-radius: 4px;
		display: flex; align-items: center;
	}
	.tags-notes-close:hover { color: var(--text-normal); background: var(--background-modifier-hover); }
	.tag-badge {
		padding: 2px 10px; border-radius: 12px;
		background: var(--interactive-accent); color: white; font-size: 12px;
	}
	.tags-notes-count {
		font-size: 11px; color: var(--text-faint);
		background: var(--background-modifier-border);
		padding: 1px 6px; border-radius: 8px;
	}

	.dashboard-tags { display: flex; flex-wrap: wrap; gap: 6px; }
	.dashboard-tag {
		display: inline-flex; align-items: center; gap: 4px;
		padding: 4px 10px; border-radius: 12px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		font-family: var(--font-interface-theme);
		font-size: 12px; color: var(--text-muted);
		cursor: pointer; transition: all 0.15s;
	}
	.dashboard-tag:hover { border-color: var(--interactive-accent); }
	.dashboard-tag.tag-selected {
		background: var(--interactive-accent); border-color: var(--interactive-accent);
	}
	.dashboard-tag.tag-selected .tag-name { color: white; }
	.dashboard-tag.tag-selected .tag-count { background: rgba(255,255,255,0.25); color: white; }
	.tag-name { color: var(--text-normal); }
	.tag-count {
		font-size: 10px; color: var(--text-faint);
		background: var(--background-modifier-border);
		padding: 1px 5px; border-radius: 8px; min-width: 16px; text-align: center;
	}

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
		display: flex; align-items: center; gap: 6px;
		font-size: 13px; font-weight: 600; color: var(--text-muted);
		margin-inline-end: auto;
	}
	.screen-badge { font-weight: 400; opacity: 0.6; font-size: 11px; }

	.screen-actions { display: flex; gap: 4px; }

	.close-btn {
		display: flex; align-items: center;
		padding: 5px 8px; border: none; border-radius: 4px;
		background: transparent; color: var(--text-muted); cursor: pointer;
		transition: all 0.15s;
	}
	.close-btn:hover { background: var(--text-error); color: white; }

	.width-control {
		display: flex; align-items: center; gap: 6px;
		color: var(--text-muted); padding: 0 4px;
	}
	.width-slider {
		width: 80px; height: 4px;
		accent-color: var(--interactive-accent); cursor: pointer;
	}

	/* ─── Content ─── */
	.screen-content { flex: 1; overflow: hidden; position: relative; }

	.navigator-fullscreen {
		position: absolute; top: 0; left: 0; right: 0; bottom: 0; overflow: hidden;
	}

	.screen-loading {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		height: 100%; gap: 12px; color: var(--text-muted);
	}

	.spinner {
		width: 28px; height: 28px;
		border: 3px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	/* ─── Note companion ─── */
	.note-companion {
		display: flex; flex-direction: column; height: 100%; overflow: hidden;
	}
	.note-area {
		flex: 1; min-height: 0; display: flex; flex-direction: column; overflow: hidden;
	}
	.note-area :global(.pane) { max-width: var(--note-width, 100%) !important; }
	.note-area :global(.note-scroll) { max-width: 100% !important; overflow-x: hidden !important; }
	.note-area :global(.note-content) { overflow-wrap: break-word; word-break: break-word; }

	.detail-tabs {
		display: flex; gap: 1px; padding: 4px 8px 0;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		overflow-x: auto;
	}

	.detail-tab {
		display: flex; align-items: center; gap: 6px;
		padding: 6px 10px; border: none; border-radius: 6px 6px 0 0;
		background: transparent; color: var(--text-muted); cursor: pointer;
		font-size: 12px; white-space: nowrap; transition: all 0.15s;
	}
	.detail-tab:hover { background: var(--background-modifier-hover); }
	.detail-tab.active {
		background: var(--background-primary); color: var(--text-normal);
		border-bottom: 2px solid var(--interactive-accent);
	}

	.tab-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.tab-name { max-width: 150px; overflow: hidden; text-overflow: ellipsis; }
	.tab-close {
		border: none; background: none; color: var(--text-muted); cursor: pointer;
		font-size: 14px; padding: 0 2px; line-height: 1; opacity: 0; transition: opacity 0.15s;
	}
	.detail-tab:hover .tab-close { opacity: 1; }

	.detail-empty {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		height: 100%; gap: 12px; color: var(--text-faint); font-size: 14px;
	}

	/* ─── Sky View Companion ─── */
	.skyview-companion { width: 100%; height: 100%; overflow: hidden; }
	.skyview-layout { display: flex; height: 100%; gap: 0; }
	.skyview-peek-area {
		flex: 1; overflow: hidden; padding: 0;
		border-inline-end: 1px solid var(--background-modifier-border);
		min-width: 0; display: flex; flex-direction: column;
	}
	.peek-empty { display: flex; align-items: center; justify-content: center; height: 100%; opacity: 0.5; }
	.skyview-detail { flex: 1; overflow-y: auto; padding: 16px 20px; min-width: 0; }
	.skyview-graph {
		width: 35%; flex-shrink: 0;
		border-inline-start: 1px solid var(--background-modifier-border);
	}
	.skyview-nav { display: flex; align-items: center; gap: 4px; margin-bottom: 8px; }
	.skyview-nav-btn {
		display: flex; align-items: center; justify-content: center;
		width: 28px; height: 28px; border: none; border-radius: 6px;
		background: var(--background-modifier-hover); color: var(--text-muted);
		cursor: pointer; transition: all 0.15s;
	}
	.skyview-nav-btn:hover:not(:disabled) { background: var(--interactive-accent); color: white; }
	.skyview-nav-btn:disabled { opacity: 0.3; cursor: default; }
	.skyview-nav-count { font-size: 11px; color: var(--text-faint); margin-inline-start: 4px; }
	.skyview-pinned { font-size: 14px; }

	.peek-preview { height: 100%; display: flex; flex-direction: column; }
	.peek-header {
		display: flex; align-items: center; gap: 8px; padding: 10px 24px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
	}
	.peek-name { flex: 1; font-size: 15px; font-weight: 600; margin: 0; color: var(--text-normal); }
	.peek-close {
		display: flex; align-items: center; justify-content: center;
		width: 24px; height: 24px; border: none; border-radius: 4px;
		background: transparent; color: var(--text-muted); cursor: pointer;
	}
	.peek-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.peek-editor { flex: 1; overflow: hidden; min-height: 0; }
	.peek-editor :global(.pane) { height: 100% !important; border: none !important; }
	.peek-editor :global(*) { max-width: 100%; box-sizing: border-box; }
	.peek-editor :global(.note-scroll) { max-width: 100% !important; overflow-x: hidden !important; padding: 1.5rem 3rem !important; }
	.peek-editor :global(.note-content) { overflow-wrap: break-word; word-break: break-word; }

	.skyview-header { display: flex; align-items: center; gap: 10px; margin-bottom: 16px; flex-wrap: wrap; }
	.skyview-dot { width: 12px; height: 12px; border-radius: 50%; flex-shrink: 0; }
	.skyview-name {
		font-size: 18px; font-weight: 600; color: var(--text-normal); margin: 0;
		flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis;
	}
	.skyview-lib { font-size: 12px; color: var(--text-faint); white-space: nowrap; }
	.skyview-preview {
		font-size: 13px; color: var(--text-muted); line-height: 1.6;
		margin-bottom: 16px; white-space: pre-wrap; max-height: 300px; overflow-y: auto;
	}

	/* ─── Sidebar sections ─── */
	.sidebar-section { padding: 8px 12px; border-bottom: 1px solid var(--background-modifier-border); }
	.sidebar-section:last-child { border-bottom: none; }
	.sidebar-heading {
		display: flex; align-items: center; gap: 6px;
		font-size: 0.75rem; font-weight: 600; color: var(--text-muted);
		text-transform: uppercase; letter-spacing: 0.5px; margin: 0 0 6px;
	}
	.sidebar-count { font-weight: 400; opacity: 0.6; font-size: 0.7rem; }
	.sidebar-links { list-style: none; margin: 0; padding: 0; }
	.sidebar-links li { margin: 1px 0; }
	.sidebar-link {
		display: flex; align-items: center; gap: 6px;
		width: 100%; padding: 4px 6px; border: none; border-radius: 4px;
		background: none; color: var(--interactive-accent);
		font-size: 0.78rem; font-family: inherit; cursor: pointer;
		text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
	}
	.sidebar-link:hover { background: var(--background-modifier-hover); text-decoration: underline; }
	.link-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
	.sidebar-tags { display: flex; flex-wrap: wrap; gap: 4px; }
	.sidebar-tag {
		font-size: 0.72rem; padding: 2px 8px; border-radius: 10px;
		background: var(--background-modifier-hover); color: var(--text-muted);
	}
	.sidebar-empty { font-size: 0.75rem; color: var(--text-faint); margin: 4px 0 0; }

	/* ─── Status bar ─── */
	.screen-status {
		display: flex; align-items: center; gap: 16px; padding: 4px 12px;
		background: var(--background-secondary);
		border-top: 1px solid var(--background-modifier-border);
		font-size: 11px; color: var(--text-muted);
	}
	.status-linked {
		margin-inline-start: auto; display: flex; align-items: center; gap: 4px;
		color: var(--interactive-accent);
	}
	.status-linked::before {
		content: ''; width: 6px; height: 6px; border-radius: 50%; background: currentColor;
	}

	/* Split Companion */
	.split-companion { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
	.split-companion-header {
		padding: 12px 16px; display: flex; align-items: center; gap: 10px;
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.split-companion-label {
		font-size: 11px; font-weight: 700; color: var(--interactive-accent);
		text-transform: uppercase; letter-spacing: 1px;
	}
	.split-companion-note {
		font-size: 14px; font-weight: 600; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.split-companion-tabs {
		display: flex; gap: 2px; padding: 6px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0; overflow-x: auto;
	}
	.sc-tab {
		padding: 5px 10px; border-radius: 6px; border: none;
		background: transparent; color: var(--text-muted);
		font-size: 12px; cursor: pointer; white-space: nowrap;
	}
	.sc-tab:hover { background: var(--background-modifier-hover); }
	.sc-tab.active { background: var(--interactive-accent); color: white; }
	.split-companion-body { flex: 1; overflow-y: auto; padding: 12px 16px; }
	.sc-panel { min-height: 100px; }
	.sc-star-panel { height: 300px; }
	.sc-empty { color: var(--text-faint); font-size: 13px; padding: 16px 0; text-align: center; }
	.sc-info { font-size: 13px; color: var(--text-muted); margin-bottom: 12px; }
	.sc-props-list { display: flex; flex-direction: column; gap: 6px; }
	.sc-prop {
		display: flex; gap: 10px; padding: 6px 10px; border-radius: 6px;
		background: var(--background-secondary);
	}
	.sc-prop-key { font-size: 12px; font-weight: 600; color: var(--text-muted); min-width: 80px; }
	.sc-prop-val { font-size: 12px; color: var(--text-normal); flex: 1; word-break: break-word; }
	.sc-tags { display: flex; flex-wrap: wrap; gap: 6px; }
	.sc-tag {
		padding: 3px 10px; border-radius: 12px; font-size: 12px;
		background: var(--background-secondary); color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
	}
</style>

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
		onContextChanged, onSkyViewHover, onSkyViewClick,
		onClipboardCopy, onNoteContentUpdate,
		sendNoteToMain, notifyScreenClosed, sendScreenState,
		type ScreenNote, type ScreenMode, type ScreenState, type ContextMode, type SkyViewNodeInfo
	} from '$lib/secondScreen';
	import {
		setActiveUniverse, listUniverses
	} from '$lib/universe/store';
	import { marked } from 'marked';

	function renderMarkdownPreview(raw: string): string {
		// Strip frontmatter
		const body = raw.replace(/^---[\s\S]*?---\n?/, '').slice(0, 2000);
		try {
			return marked.parse(body, { async: false }) as string;
		} catch {
			return `<p>${body.slice(0, 800)}</p>`;
		}
	}

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

	// ─── Sky View Companion state ───
	let contextMode = $state<ContextMode>('editor');
	let skyviewNode = $state<SkyViewNodeInfo | null>(null);
	let pinnedSkyviewNode = $state<SkyViewNodeInfo | null>(null); // Stays until next click
	let isHoverPreview = $state(false); // true = temporary hover, false = pinned click

	// Peek preview: full editable note in left panel
	let peekNote = $state<{ name: string; path: string; libraryName: string; libraryColor: string } | null>(null);
	let peekTab = $state<import('$lib/libraries/store').OpenTab | null>(null);
	let peekGeneration = 0;

	// ─── Editor Mode: Smart Panels ───

	// Reference Shelf: pin a different note for reading while writing
	let refShelfNote = $state<{ name: string; path: string; libraryName: string } | null>(null);
	let refShelfContent = $state('');
	let refShelfLoading = $state(false);
	let refShelfSearch = $state('');
	let refShelfResults = $derived.by(() => {
		if (!refShelfSearch || refShelfSearch.length < 2) return [];
		const q = refShelfSearch.toLowerCase();
		return allNotes.filter(n => n.name.toLowerCase().includes(q)).slice(0, 15);
	});

	async function pinReferenceNote(note: { name: string; path: string; libraryName: string }) {
		refShelfNote = note;
		refShelfLoading = true;
		refShelfSearch = '';
		try {
			refShelfContent = await invoke<string>('read_note', { filePath: note.path });
		} catch {
			refShelfContent = '(Failed to load)';
		}
		refShelfLoading = false;
	}

	// Clipboard Timeline: track copied text
	let clipboardItems = $state<{ text: string; time: number; source: string }[]>([]);
	function addToClipboard(text: string, source: string) {
		if (!text.trim()) return;
		// Deduplicate
		clipboardItems = [{ text: text.slice(0, 2000), time: Date.now(), source }, ...clipboardItems.filter(c => c.text !== text)].slice(0, 50);
	}

	// Writing Dashboard
	let dashSessionStart = $state(0);
	let dashWordCount = $state(0);
	let dashCharCount = $state(0);
	let dashSessionWords = $state(0); // words written this session
	let dashInitialWords = $state(0);
	let pomodoroRunning = $state(false);
	let pomodoroTimeLeft = $state(25 * 60); // 25 min in seconds
	let pomodoroInterval: ReturnType<typeof setInterval> | null = null;

	function startPomodoro() {
		pomodoroRunning = true;
		pomodoroTimeLeft = 25 * 60;
		if (pomodoroInterval) clearInterval(pomodoroInterval);
		pomodoroInterval = setInterval(() => {
			pomodoroTimeLeft--;
			if (pomodoroTimeLeft <= 0) {
				pomodoroRunning = false;
				if (pomodoroInterval) clearInterval(pomodoroInterval);
				pomodoroInterval = null;
			}
		}, 1000);
	}
	function stopPomodoro() {
		pomodoroRunning = false;
		if (pomodoroInterval) clearInterval(pomodoroInterval);
		pomodoroInterval = null;
		pomodoroTimeLeft = 25 * 60;
	}
	function formatPomodoroTime(sec: number): string {
		const m = Math.floor(sec / 60);
		const s = sec % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	// Note Diff: track last saved vs current
	let diffLastSaved = $state('');
	let diffCurrent = $state('');
	let diffLines = $derived.by(() => {
		if (!diffLastSaved && !diffCurrent) return [];
		const oldLines = diffLastSaved.split('\n');
		const newLines = diffCurrent.split('\n');
		const result: { type: 'same' | 'added' | 'removed'; text: string }[] = [];
		const maxLen = Math.max(oldLines.length, newLines.length);
		for (let i = 0; i < maxLen; i++) {
			const oldLine = oldLines[i] ?? '';
			const newLine = newLines[i] ?? '';
			if (oldLine === newLine) {
				result.push({ type: 'same', text: newLine });
			} else {
				if (i < oldLines.length && oldLine !== '') {
					result.push({ type: 'removed', text: oldLine });
				}
				if (i < newLines.length && newLine !== '') {
					result.push({ type: 'added', text: newLine });
				}
			}
		}
		return result;
	});
	let diffChangeCount = $derived(diffLines.filter(l => l.type !== 'same').length);

	// AI Context Radar: semantic suggestions (uses existing embedding infrastructure)
	let radarSuggestions = $state<{ name: string; path: string; libraryName: string; score: number }[]>([]);
	let radarLoading = $state(false);
	let radarLastQuery = '';

	// Editor mode active panel (which sections are expanded)
	let editorPanels = $state<Record<string, boolean>>({
		reference: true,
		radar: true,
		clipboard: true,
		dashboard: true,
		diff: true,
	});

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

	// Navigation history for second screen (back/forward)
	let skyviewHistory = $state<SkyViewNodeInfo[]>([]);
	let skyviewHistoryIdx = $state(-1); // current position in history
	let isNavigatingHistory = false; // prevent pushes during back/forward

	function pushSkyviewHistory(node: SkyViewNodeInfo) {
		if (isNavigatingHistory) return;
		// Truncate forward history when navigating to a new node
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
	let skyviewPreview = $state('');
	let skyviewBacklinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let skyviewForwardLinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let skyviewTags = $state<string[]>([]);
	let skyviewLocalNodes = $state<StarNode[]>([]);
	let skyviewLocalLinks = $state<StarLink[]>([]);
	let skyviewGeneration = 0;

	async function loadSkyViewCompanionData(node: SkyViewNodeInfo) {
		const gen = ++skyviewGeneration;
		skyviewNode = node;

		// Load note preview
		try {
			const content = await invoke<string>('read_note', { notePath: node.path });
			if (gen !== skyviewGeneration) return;
			skyviewPreview = content.slice(0, 2000);

			// Parse tags from frontmatter
			const fm = parseFrontmatter(content);
			const tagProp = fm.properties.find(p => p.key === 'tags');
			skyviewTags = tagProp?.listItems ?? [];
		} catch {
			if (gen !== skyviewGeneration) return;
			skyviewPreview = '';
			skyviewTags = [];
		}

		// Scan for backlinks and forward links
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

				// Build local star for this node
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

		const defaultUI = '-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, "Noto Sans Arabic", "Noto Sans Hebrew", "Noto Sans CJK SC", sans-serif';
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

		// Sky View companion: listen for context mode changes
		const u8 = await onContextChanged((mode) => {
			contextMode = mode;
			if (mode !== 'skyview') {
				skyviewNode = null;
			}
		});
		unlisteners.push(u8);

		// Sky View companion: listen for hover events (temporary preview)
		const u9 = await onSkyViewHover(async (node) => {
			if (contextMode !== 'skyview') return;
			if (!node) {
				// Hover ended — revert to pinned note if one exists
				isHoverPreview = false;
				if (pinnedSkyviewNode) {
					await loadSkyViewCompanionData(pinnedSkyviewNode);
				} else {
					skyviewNode = null;
				}
				return;
			}
			// Show temporary hover preview
			isHoverPreview = true;
			await loadSkyViewCompanionData(node);
		});
		unlisteners.push(u9);

		// Sky View companion: listen for click events (pinned — stays until next click)
		const u10 = await onSkyViewClick(async (node) => {
			if (contextMode !== 'skyview') return;
			isHoverPreview = false;
			pinnedSkyviewNode = node;
			pushSkyviewHistory(node);
			await loadSkyViewCompanionData(node);
		});
		unlisteners.push(u10);

		// Editor mode: clipboard copy events
		const u11 = await onClipboardCopy((text, source) => {
			addToClipboard(text, source);
		});
		unlisteners.push(u11);

		// Editor mode: note content updates (for diff + dashboard)
		const u12 = await onNoteContentUpdate((content, savedContent, noteName) => {
			diffCurrent = content;
			if (!diffLastSaved || diffLastSaved !== savedContent) {
				diffLastSaved = savedContent;
			}
			// Update dashboard word counts
			const words = content.replace(/^---[\s\S]*?---\n?/, '').trim().split(/\s+/).filter(w => w.length > 0).length;
			const chars = content.length;
			if (dashSessionStart === 0) {
				dashSessionStart = Date.now();
				dashInitialWords = words;
			}
			dashWordCount = words;
			dashCharCount = chars;
			dashSessionWords = Math.max(0, words - dashInitialWords);
		});
		unlisteners.push(u12);
	});

	onDestroy(() => {
		unlisteners.forEach(u => u());
		pendingTimers.forEach(t => clearTimeout(t));
		if (libraryChangeTimer) clearTimeout(libraryChangeTimer);
		if (pomodoroInterval) clearInterval(pomodoroInterval);
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
		{:else if contextMode === 'skyview'}
			<div class="skyview-companion">
				{#if skyviewNode}
					<div class="skyview-layout">
						<!-- Left: Peek preview (fills empty space) -->
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
										<NotePane
											tab={peekTab}
											isFocused={true}
											onFocus={() => {}}
											color={peekNote?.libraryColor || '#7c3aed'}
											allNotes={allNotes}
											{libraryColorMap}
										/>
									</div>
								</div>
							{:else}
								<div class="peek-empty">
									<p class="sidebar-empty">{$t('secondScreen.skyviewHint') || 'Click a link to preview it here'}</p>
								</div>
							{/if}
						</div>

						<!-- Right: Links, tags, graph -->
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
													loadPeekPreview({
														name: link.name.replace(/\.md$/, ''),
														path: link.path,
														libraryName: link.libraryName,
														libraryColor: libraryColorMap[link.libraryName] || '#7c3aed',
													});
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
													loadPeekPreview({
														name: link.name.replace(/\.md$/, ''),
														path: link.path,
														libraryName: link.libraryName,
														libraryColor: libraryColorMap[link.libraryName] || '#7c3aed',
													});
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
		{:else if contextMode === 'editor' || $activeTab}
			<div class="editor-companion" style="min-height: 100%; background: var(--background-primary);">
				<!-- Writing Dashboard (top bar) -->
				<div class="ec-dashboard">
					<div class="ec-stats">
						<span class="ec-stat"><strong>{dashWordCount}</strong> words</span>
						<span class="ec-stat"><strong>{dashCharCount}</strong> chars</span>
						<span class="ec-stat"><strong>+{dashSessionWords}</strong> this session</span>
						{#if dashSessionStart > 0}
							<span class="ec-stat">{Math.round((Date.now() - dashSessionStart) / 60000)}m elapsed</span>
						{/if}
					</div>
					<div class="ec-pomodoro">
						{#if pomodoroRunning}
							<span class="ec-timer">{formatPomodoroTime(pomodoroTimeLeft)}</span>
							<button class="ec-btn ec-btn-stop" onclick={stopPomodoro}>Stop</button>
						{:else}
							<button class="ec-btn ec-btn-start" onclick={startPomodoro}>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
								Pomodoro 25m
							</button>
						{/if}
					</div>
				</div>

				<div class="ec-panels">
					<!-- Reference Shelf -->
					<div class="ec-panel" class:collapsed={!editorPanels.reference}>
						<button class="ec-panel-header" onclick={() => editorPanels.reference = !editorPanels.reference}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/></svg>
							<span>Reference Shelf</span>
							<svg class="ec-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
						</button>
						{#if editorPanels.reference}
							<div class="ec-panel-body">
								{#if refShelfNote}
									<div class="ref-pinned">
										<span class="ref-name" dir="auto">{refShelfNote.name.replace(/\.md$/, '')}</span>
										<button class="ec-btn-icon" onclick={() => { refShelfNote = null; refShelfContent = ''; }} title="Unpin">×</button>
									</div>
									{#if refShelfLoading}
										<p class="ec-muted">Loading...</p>
									{:else}
										<div class="ref-content markdown-rendered" dir="auto">
											{@html renderMarkdownPreview(refShelfContent)}
										</div>
									{/if}
								{:else}
									<input class="ec-search" type="text" dir="auto" placeholder="Search notes to pin..." bind:value={refShelfSearch} />
									{#if refShelfResults.length > 0}
										<ul class="ec-results">
											{#each refShelfResults as note}
												<li>
													<button class="ec-result-btn" dir="auto" onclick={() => pinReferenceNote(note)}>
														<span class="link-dot" style="background:{libraryColorMap[note.libraryName] || '#7c3aed'}"></span>
														{note.name.replace(/\.md$/, '')}
													</button>
												</li>
											{/each}
										</ul>
									{/if}
								{/if}
							</div>
						{/if}
					</div>

					<!-- AI Context Radar -->
					<div class="ec-panel" class:collapsed={!editorPanels.radar}>
						<button class="ec-panel-header" onclick={() => editorPanels.radar = !editorPanels.radar}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 2a10 10 0 0110 10"/><path d="M12 2a10 10 0 016.93 4"/><circle cx="12" cy="12" r="2"/></svg>
							<span>AI Context Radar</span>
							{#if radarLoading}
								<span class="ec-badge">...</span>
							{:else if radarSuggestions.length > 0}
								<span class="ec-badge">{radarSuggestions.length}</span>
							{/if}
							<svg class="ec-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
						</button>
						{#if editorPanels.radar}
							<div class="ec-panel-body">
								{#if radarSuggestions.length > 0}
									<ul class="ec-results">
										{#each radarSuggestions as sug}
											<li>
												<button class="ec-result-btn" dir="auto" onclick={() => pinReferenceNote(sug)}>
													<span class="link-dot" style="background:{libraryColorMap[sug.libraryName] || '#7c3aed'}"></span>
													<span class="ec-result-name">{sug.name.replace(/\.md$/, '')}</span>
													<span class="ec-score">{Math.round(sug.score * 100)}%</span>
												</button>
											</li>
										{/each}
									</ul>
								{:else}
									<p class="ec-muted">Semantic suggestions will appear as you write</p>
								{/if}
							</div>
						{/if}
					</div>

					<!-- Clipboard Timeline -->
					<div class="ec-panel" class:collapsed={!editorPanels.clipboard}>
						<button class="ec-panel-header" onclick={() => editorPanels.clipboard = !editorPanels.clipboard}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2"/><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>
							<span>Clipboard Timeline</span>
							{#if clipboardItems.length > 0}
								<span class="ec-badge">{clipboardItems.length}</span>
							{/if}
							<svg class="ec-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
						</button>
						{#if editorPanels.clipboard}
							<div class="ec-panel-body">
								{#if clipboardItems.length > 0}
									<ul class="ec-clipboard">
										{#each clipboardItems as item}
											<li class="ec-clip-item" dir="auto">
												<button class="ec-clip-btn" onclick={async () => {
													try { await navigator.clipboard.writeText(item.text); } catch {}
												}} title="Click to copy back">
													<span class="ec-clip-text">{item.text.slice(0, 120)}{item.text.length > 120 ? '...' : ''}</span>
													<span class="ec-clip-meta">{item.source} · {new Date(item.time).toLocaleTimeString()}</span>
												</button>
											</li>
										{/each}
									</ul>
								{:else}
									<p class="ec-muted">Copy text in the editor — it will appear here</p>
								{/if}
							</div>
						{/if}
					</div>

					<!-- Note Diff -->
					<div class="ec-panel" class:collapsed={!editorPanels.diff}>
						<button class="ec-panel-header" onclick={() => editorPanels.diff = !editorPanels.diff}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v18"/><path d="M18 9l-6-6-6 6"/><path d="M6 15l6 6 6-6"/></svg>
							<span>Note Diff</span>
							{#if diffChangeCount > 0}
								<span class="ec-badge ec-badge-warn">{diffChangeCount} changes</span>
							{/if}
							<svg class="ec-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
						</button>
						{#if editorPanels.diff}
							<div class="ec-panel-body">
								{#if diffLines.length > 0 && diffChangeCount > 0}
									<div class="ec-diff">
										{#each diffLines as line}
											{#if line.type !== 'same'}
												<div class="ec-diff-line" class:ec-diff-added={line.type === 'added'} class:ec-diff-removed={line.type === 'removed'} dir="auto">
													<span class="ec-diff-marker">{line.type === 'added' ? '+' : '-'}</span>
													{line.text || ' '}
												</div>
											{/if}
										{/each}
									</div>
								{:else}
									<p class="ec-muted">No unsaved changes</p>
								{/if}
							</div>
						{/if}
					</div>
				</div>
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

	/* ─── Sky View Companion ─── */
	.skyview-companion {
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
	.skyview-layout {
		display: flex;
		height: 100%;
		gap: 0;
	}
	.skyview-peek-area {
		flex: 1;
		overflow: hidden;
		padding: 0;
		border-inline-end: 1px solid var(--background-modifier-border, #e0e0e0);
		min-width: 0;
		display: flex;
		flex-direction: column;
	}
	.peek-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		opacity: 0.5;
	}
	.skyview-detail {
		flex: 1;
		overflow-y: auto;
		padding: 16px 20px;
		min-width: 0;
	}
	.skyview-graph {
		width: 35%;
		flex-shrink: 0;
		border-inline-start: 1px solid var(--background-modifier-border);
	}
	.skyview-nav {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-bottom: 8px;
	}
	.skyview-nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: var(--background-modifier-hover, #f0f0f0);
		color: var(--text-muted, #666);
		cursor: pointer;
		transition: all 0.15s;
	}
	.skyview-nav-btn:hover:not(:disabled) {
		background: var(--interactive-accent, #7c3aed);
		color: white;
	}
	.skyview-nav-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}
	.skyview-nav-count {
		font-size: 11px;
		color: var(--text-faint, #999);
		margin-inline-start: 4px;
	}
	.skyview-pinned {
		font-size: 14px;
	}

	/* Peek preview */
	.peek-preview {
		height: 100%;
		display: flex;
		flex-direction: column;
	}
	.peek-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 24px;
		border-bottom: 1px solid var(--background-modifier-border, #e0e0e0);
		background: var(--background-secondary, #f8f8f8);
	}
	.peek-name {
		flex: 1;
		font-size: 15px;
		font-weight: 600;
		margin: 0;
		color: var(--text-normal);
	}
	.peek-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}
	.peek-close:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.peek-editor {
		flex: 1;
		overflow: hidden;
		min-height: 0;
	}
	.peek-editor :global(.pane) {
		height: 100% !important;
		border: none !important;
	}
	.peek-editor :global(*) {
		max-width: 100%;
		box-sizing: border-box;
	}
	.peek-editor :global(.note-scroll) {
		max-width: 100% !important;
		overflow-x: hidden !important;
		padding: 1.5rem 3rem !important;
	}
	.peek-editor :global(.note-content) {
		overflow-wrap: break-word;
		word-break: break-word;
	}
	.skyview-header {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 16px;
		flex-wrap: wrap;
	}
	.skyview-dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.skyview-name {
		font-size: 18px;
		font-weight: 600;
		color: var(--text-normal);
		margin: 0;
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.skyview-lib {
		font-size: 12px;
		color: var(--text-faint);
		white-space: nowrap;
	}
	.skyview-preview {
		font-size: 13px;
		color: var(--text-muted);
		line-height: 1.6;
		margin-bottom: 16px;
		white-space: pre-wrap;
		max-height: 300px;
		overflow-y: auto;
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

	/* ─── Editor Companion Panels ─── */
	.editor-companion {
		position: absolute;
		top: 0; left: 0; right: 0; bottom: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.ec-dashboard {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 16px;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.ec-stats { display: flex; gap: 16px; font-size: 12px; color: var(--text-muted); }
	.ec-stat strong { color: var(--text-normal); font-variant-numeric: tabular-nums; }
	.ec-pomodoro { display: flex; align-items: center; gap: 8px; }
	.ec-timer { font-size: 18px; font-weight: 700; color: var(--interactive-accent); font-variant-numeric: tabular-nums; }
	.ec-btn {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 10px; border: none; border-radius: 4px;
		font-size: 12px; cursor: pointer;
		background: var(--background-modifier-hover); color: var(--text-muted);
	}
	.ec-btn:hover { background: var(--interactive-accent); color: white; }
	.ec-btn-start { background: var(--interactive-accent); color: white; }
	.ec-btn-stop { background: #ef4444; color: white; }

	.ec-panels {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.ec-panel {
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		overflow: hidden;
		background: var(--background-primary);
	}
	.ec-panel.collapsed .ec-chevron { transform: rotate(-90deg); }

	.ec-panel-header {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 12px;
		border: none;
		background: var(--background-secondary);
		color: var(--text-normal);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		text-align: start;
	}
	.ec-panel-header:hover { background: var(--background-modifier-hover); }
	.ec-panel-header span:first-of-type { flex: 1; }
	.ec-chevron { transition: transform 0.15s; flex-shrink: 0; }

	.ec-panel-body {
		padding: 8px 12px;
		max-height: 300px;
		overflow-y: auto;
	}

	.ec-badge {
		font-size: 10px;
		padding: 1px 6px;
		border-radius: 10px;
		background: var(--interactive-accent);
		color: white;
	}
	.ec-badge-warn { background: #f59e0b; }

	.ec-muted { font-size: 12px; color: var(--text-faint); margin: 4px 0; }

	.ec-search {
		width: 100%;
		padding: 6px 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-secondary);
		color: var(--text-normal);
		font-size: 13px;
		outline: none;
	}
	.ec-search:focus { border-color: var(--interactive-accent); }

	.ec-results { list-style: none; padding: 0; margin: 6px 0 0; }
	.ec-results li { margin: 0; }
	.ec-result-btn {
		display: flex; align-items: center; gap: 8px;
		width: 100%; padding: 5px 8px; border: none; border-radius: 4px;
		background: transparent; color: var(--text-normal);
		font-size: 12px; cursor: pointer; text-align: start;
	}
	.ec-result-btn:hover { background: var(--background-modifier-hover); }
	.ec-result-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.ec-score { font-size: 10px; color: var(--text-faint); font-variant-numeric: tabular-nums; }

	/* Reference Shelf */
	.ref-pinned {
		display: flex; align-items: center; gap: 8px;
		padding-bottom: 6px; border-bottom: 1px solid var(--background-modifier-border);
		margin-bottom: 8px;
	}
	.ref-name { flex: 1; font-weight: 600; font-size: 13px; }
	.ec-btn-icon {
		width: 24px; height: 24px; border: none; border-radius: 4px;
		background: transparent; color: var(--text-muted); cursor: pointer;
		display: flex; align-items: center; justify-content: center; font-size: 16px;
	}
	.ec-btn-icon:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.ref-content { font-size: 13px; line-height: 1.6; max-height: 250px; overflow-y: auto; }
	.ref-content :global(h1) { font-size: 1.2em; }
	.ref-content :global(h2) { font-size: 1.1em; }
	.ref-content :global(h3) { font-size: 1em; }

	/* Clipboard Timeline */
	.ec-clipboard { list-style: none; padding: 0; margin: 0; }
	.ec-clip-item { margin-bottom: 4px; }
	.ec-clip-btn {
		display: flex; flex-direction: column; gap: 2px;
		width: 100%; padding: 6px 8px; border: none; border-radius: 4px;
		background: var(--background-secondary); color: var(--text-normal);
		font-size: 12px; cursor: pointer; text-align: start;
	}
	.ec-clip-btn:hover { background: var(--background-modifier-hover); }
	.ec-clip-text { white-space: pre-wrap; word-break: break-word; line-height: 1.4; }
	.ec-clip-meta { font-size: 10px; color: var(--text-faint); }

	/* Note Diff */
	.ec-diff {
		font-family: var(--font-monospace, 'Fira Code', monospace);
		font-size: 11px;
		line-height: 1.5;
	}
	.ec-diff-line {
		padding: 1px 8px;
		border-radius: 2px;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.ec-diff-added { background: rgba(34, 197, 94, 0.15); color: #16a34a; }
	.ec-diff-removed { background: rgba(239, 68, 68, 0.15); color: #ef4444; text-decoration: line-through; }
	.ec-diff-marker {
		display: inline-block; width: 14px;
		font-weight: 700; user-select: none;
	}
</style>

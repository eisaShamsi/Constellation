<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { dir, t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { getVersion } from '@tauri-apps/api/app';
	import {
		libraries, libraryStats, searchResults, totalStars, libraryCount,
		activeTab, openTabs, activeTabId,
		splitActive, splitDirection, focusedTabId, focusedTab,
		loadLibraries, loadAllStats, addLibrary, createNewLibrary, searchAllStars,
		openNoteTab, closeTab, switchTab, closeNote, createEmptyTab,
		toggleSplit, toggleSplitDirection, setFocusedTab,
		parseFrontmatter, extractHeadings,
		createNote, createFolder, renameItem, deleteItem,
		startWatchingLibrary, wasRecentlyWritten,
		loadLibraryAppearance, libraryAppearances,
		toggleEditMode, editingTabIds,
		navigateBack, navigateForward,
		scanLibraryLinks, scanLibraryTags, getBacklinks, getOutgoingLinks, scanUnlinkedMentions,
		scanLibraryIndex,
		buildStarData, readNotePreview,
		getDailyNotePath, updateLinksOnRename, quickCapture,
		loadBookmarks, addBookmark, removeBookmark, isBookmarked, bookmarks,
		loadSettings, updateSettings, appSettings,
		loadWorkspaces, workspaces,
		resolveWikilinkCrossLibrary,
		buildDefaultFrontmatter, searchByProperty,
		type FrontmatterProperty, type HeadingItem, type NoteLink, type StarNode, type StarLink,
		type IndexEntry
	} from '$lib/libraries/store';
	import type { LibraryStats, FileEntry, WorkspaceLayout, WorkspaceSecondScreen } from '$lib/libraries/store';
	import { get } from 'svelte/store';
	import { detectDir, eventToShortcut, normalizeShortcut, getResolvedShortcut, formatShortcut } from '$lib/utils';
	import { createBase, saveBaseFile, listWorkspaceBases, createWorkspaceBase, saveWorkspaceBase, deleteWorkspaceBase } from '$lib/bases/store';
	import type { WorkspaceBaseEntry } from '$lib/bases/store';
	import type { BaseDefinition } from '$lib/bases/types';
	import FileTree from '$lib/components/FileTree.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import QuickSwitcher from '$lib/components/QuickSwitcher.svelte';
	import TemplatePicker from '$lib/components/TemplatePicker.svelte';
	import TemplatePrompt from '$lib/components/TemplatePrompt.svelte';
	import TemplateSuggester from '$lib/components/TemplateSuggester.svelte';
	import { processTemplate, processTemplateAsync, extractTemplateBody, type TemplateCallbacks } from '$lib/templates/engine';
	import GraphMindView from '$lib/components/GraphMindView.svelte';
	import LocalStarView from '$lib/components/LocalStarView.svelte';
	import NoteGrid from '$lib/components/NoteGrid.svelte';
	import BacklinksPanel from '$lib/components/BacklinksPanel.svelte';
	import TagsPanel from '$lib/components/TagsPanel.svelte';
	import LinkDashboard from '$lib/components/LinkDashboard.svelte';
	import TasksPanel from '$lib/components/TasksPanel.svelte';
	import CalendarPanel from '$lib/components/CalendarPanel.svelte';
	import GlobalTasksView from '$lib/components/GlobalTasksView.svelte';
	import { scanNoteTasks, toggleTask, scanLibraryNoteDates } from '$lib/tasks/store';
	import type { TaskItem } from '$lib/tasks/types';
	import PropertyEditor from '$lib/components/PropertyEditor.svelte';
	import PagePreview from '$lib/components/PagePreview.svelte';
	import WorkspaceManager from '$lib/components/WorkspaceManager.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import LockScreen from '$lib/components/LockScreen.svelte';
	import LibrarySwitcher from '$lib/components/LibrarySwitcher.svelte';
	import LibraryManager from '$lib/components/LibraryManager.svelte';
	import LibraryPicker from '$lib/components/LibraryPicker.svelte';
	import NewBaseDialog from '$lib/components/NewBaseDialog.svelte';
	import OutgoingLinksPanel from '$lib/components/OutgoingLinksPanel.svelte';
	import IndexPanel from '$lib/components/IndexPanel.svelte';
	import UniverseSetup from '$lib/components/UniverseSetup.svelte';
	import UniverseManager from '$lib/components/UniverseManager.svelte';
	import ImporterModal from '$lib/components/ImporterModal.svelte';
	import {
		listUniverses, createUniverse, setActiveUniverse,
		checkMigrationNeeded, migrateLegacyData,
		getChildUniverses,
		type UniverseEntry, type ChildUniverseInfo
	} from '$lib/universe/store';
	import { loadPropertyTypes } from '$lib/libraries/propertyTypeRegistry';
	import { openSecondScreen, closeSecondScreen, isSecondScreenOpen, sendNoteToScreen, onNoteToMain, onScreenClosed, notifyUniverseSwitch, notifySettingsChanged, requestScreenState, onStateResponse, sendWorkspaceRestore, emitContextChanged, emitSkyViewHover, emitSkyViewClick, emitClipboardCopy, emitNoteContentUpdate, type ScreenNote, type ScreenState, type SkyViewNodeInfo } from '$lib/secondScreen';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	// Sidebar state
	let sidebarOpen = $state(true);
	let searchMode = $state(false);
	// indexMode removed - index now opens as full page view
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;
	let sortOrder = $state<'name-asc' | 'name-desc' | 'modified-desc' | 'modified-asc'>('name-asc');
	let libraryPickerAction = $state<'note' | 'folder' | 'base'>('note');
	let allExpanded = $state(true);

	// Universe state
	let showUniverseSetup = $state(false);
	let showUniverseManager = $state(false);
	let activeUniverseName = $state('');
	let appVersion = $state('');
	let appReady = $state(false);

	// Load app version
	getVersion().then(v => appVersion = v).catch(() => {});

	// Update window title when active universe changes
	$effect(() => {
		const name = activeUniverseName;
		const ver = appVersion;
		if (name && ver) {
			getCurrentWindow().setTitle(`Constellation v${ver} - ${name}`).catch(() => {});
		} else if (name) {
			getCurrentWindow().setTitle(`Constellation - ${name}`).catch(() => {});
		}
	});

	// Right sidebar
	let rightSidebarOpen = $state(false);
	let rightSidebarTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks' | 'calendar'>('properties');

	// Sidebar resizing
	let leftSidebarWidth = $state(240);
	let rightSidebarWidth = $state(260);
	let resizing = $state<'left' | 'right' | null>(null);

	// Command palette & quick switcher
	let showCommandPalette = $state(false);
	let showQuickSwitcher = $state(false);
	let showTemplatePicker = $state(false);
	let templatePickerMode = $state<'insert' | 'newNote'>('insert');

	// Template prompt/suggester state for async template processing
	let activePrompt = $state<{ question: string; defaultValue?: string; resolve: (v: string | null) => void } | null>(null);
	let activeSuggester = $state<{ options: string[]; resolve: (v: string | null) => void } | null>(null);

	/** Build TemplateCallbacks with promise-based bridges for interactive variables */
	function buildTemplateCallbacks(): TemplateCallbacks {
		return {
			getClipboard: async () => {
				try { return await navigator.clipboard.readText(); } catch { return ''; }
			},
			getFileMetadata: async (filePath: string) => {
				try { return await invoke('get_file_metadata', { filePath }); } catch { return null; }
			},
			promptUser: (question: string, defaultValue?: string) => {
				return new Promise<string | null>((resolve) => {
					activePrompt = { question, defaultValue, resolve };
				});
			},
			suggestOptions: (options: string[]) => {
				return new Promise<string | null>((resolve) => {
					activeSuggester = { options, resolve };
				});
			},
		};
	}
	let showStarView = $state(false);
	// Emit context change to second screen when Sky View toggles or active tab changes
	let skyviewHoverTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		if (secondScreenOpen) {
			const mode = showStarView ? 'skyview' : 'editor';
			emitContextChanged(mode);
			// When switching to editor mode, send the current note to second screen
			if (mode === 'editor' && $activeTab?.path) {
				sendNoteToScreen({
					path: $activeTab.path,
					body: '',
					libraryName: $activeTab.libraryName,
					name: $activeTab.name,
				});
			}
		}
	});
	// Also sync when the user switches tabs in editor mode
	$effect(() => {
		if (secondScreenOpen && !showStarView && $activeTab?.path) {
			sendNoteToScreen({
				path: $activeTab.path,
				body: '',
				libraryName: $activeTab.libraryName,
				name: $activeTab.name,
			});
		}
	});
	// Clipboard monitoring: send copy events to second screen
	let lastSavedContent = $state('');
	$effect(() => {
		if (!secondScreenOpen || showStarView) return;
		const tab = $activeTab;
		if (!tab?.content) return;
		// Send content updates for diff + word count (debounced by Svelte's batching)
		console.log('[Main] Sending note content to second screen:', tab.name, 'words:', tab.content.split(/\s+/).length);
		emitNoteContentUpdate(tab.content, lastSavedContent || tab.content, tab.name);
		// Also emit editor context mode
		emitContextChanged('editor');
	});
	// Track initial content for diff baseline
	$effect(() => {
		const tab = $activeTab;
		if (tab?.path) {
			invoke<string>('read_note', { filePath: tab.path }).then(saved => {
				lastSavedContent = saved;
			}).catch(() => {});
		}
	});

	let showGlobalTasks = $state(false);
	let showIndex = $state(false);
	let indexNoteTab = $state<import('$lib/libraries/store').OpenTab | null>(null);
	let indexActiveNotePath = $state('');

	// Tasks sidebar data
	let sidebarTasks = $state<TaskItem[]>([]);
	// Calendar sidebar data
	let calendarNoteDates = $state<Record<string, number>>({});
	let calendarTaskDates = $state<Record<string, number>>({});

	// Workspace manager
	let showWorkspaces = $state(false);

	// Settings modal
	let showSettings = $state(false);
	// Importer modal
	let showImporter = $state(false);
	let secondScreenOpen = $state(false);

	// Library management
	let showLibrarySwitcher = $state(false);
	let showLibraryManager = $state(false);
	let showLibraryPicker = $state(false);
	let showNewBaseDialog = $state(false);

	// Lock screen
	let isLocked = $state(false);
	let idleTimer: ReturnType<typeof setTimeout> | null = null;

	// Page preview (hover)
	let pagePreview = $state<{ content: string; x: number; y: number; visible: boolean }>({ content: '', x: 0, y: 0, visible: false });
	let previewTimeout: ReturnType<typeof setTimeout>;

	// Cache refresh debounce (for file watcher)
	let cacheRefreshDebounce: ReturnType<typeof setTimeout>;
	// Watcher debounce (hoisted so onDestroy can clear it)
	let watcherDebounce: ReturnType<typeof setTimeout>;

	// Library data caches
	let allLibraryLinks = $state<NoteLink[]>([]);
	let allLibraryTags = $state<Record<string, number>>({});
	let allNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let allIndexEntries = $state<IndexEntry[]>([]);
	// Star data stored as plain (non-reactive) arrays to avoid $state proxy overhead
	// on potentially tens of thousands of items. Use starVersion to signal changes.
	let starNodes: StarNode[] = [];
	let starLinks: StarLink[] = [];
	let starVersion = $state(0);
	// Star data is passed to StarView as plain arrays.
	// We avoid $state/$derived for large arrays (1885+ nodes) because Svelte 5 proxies
	// make iteration extremely slow. Instead, starVersion ($state) triggers re-render
	// and StarView reads the plain starNodes/starLinks directly.

	let resizeCleanup: (() => void) | null = null;

	function startResize(side: 'left' | 'right', e: MouseEvent) {
		e.preventDefault();
		resizing = side;
		const startX = e.clientX;
		const startWidth = side === 'left' ? leftSidebarWidth : rightSidebarWidth;
		const isRtl = $dir === 'rtl';

		function onMouseMove(ev: MouseEvent) {
			const delta = ev.clientX - startX;
			if (side === 'left') {
				leftSidebarWidth = Math.max(160, Math.min(500, startWidth + (isRtl ? -delta : delta)));
			} else {
				rightSidebarWidth = Math.max(160, Math.min(500, startWidth + (isRtl ? delta : -delta)));
			}
		}

		function onMouseUp() {
			resizing = null;
			document.removeEventListener('mousemove', onMouseMove);
			document.removeEventListener('mouseup', onMouseUp);
			resizeCleanup = null;
		}

		document.addEventListener('mousemove', onMouseMove);
		document.addEventListener('mouseup', onMouseUp);
		resizeCleanup = () => {
			document.removeEventListener('mousemove', onMouseMove);
			document.removeEventListener('mouseup', onMouseUp);
		};
	}

	// Library trees
	let libraryTrees = $state<Record<string, FileEntry[]>>({});
	let expandedLibraries = $state<Set<string>>(new Set());

	// Workspace bases
	let workspaceBases = $state<WorkspaceBaseEntry[]>([]);
	let workspaceBasesExpanded = $state(true);

	// Child universes for sidebar
	let childUniverses = $state<ChildUniverseInfo[]>([]);
	let childUniversesExpanded = $state(true);

	let error = $state('');
	let adding = $state(false);
	let creatingNew = $state(false);
	let newLibraryName = $state('');

	const isHome = $derived(page.url.pathname === '/');

	// Library color palette
	const LIBRARY_COLORS = ['#7c3aed', '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#06b6d4', '#8b5cf6'];
	const libraryColorMap = $derived.by(() => {
		const map: Record<string, string> = {};
		$libraries.forEach((v, i) => { map[v.name] = LIBRARY_COLORS[i % LIBRARY_COLORS.length]; });
		return map;
	});

	// Sidebar data: derived from focused tab (whichever pane has focus)
	const sidebarTab = $derived($focusedTab);
	const sidebarParsed = $derived(sidebarTab ? parseFrontmatter(sidebarTab.content) : null);
	const sidebarProperties = $derived<FrontmatterProperty[]>(sidebarParsed?.properties ?? []);
	const sidebarBody = $derived(sidebarParsed?.body ?? '');
	const sidebarHeadings = $derived<HeadingItem[]>(sidebarBody ? extractHeadings(sidebarBody) : []);
	const noteDir = $derived(sidebarBody ? detectDir(sidebarBody) : $dir);

	// Backlinks for current note
	const currentBacklinks = $derived.by(() => {
		if (!sidebarTab) return [];
		return getBacklinks(allLibraryLinks, sidebarTab.name);
	});

	// Unlinked mentions for current note
	let currentUnlinkedMentions: { name: string; path: string; context: string; libraryName: string }[] = $state([]);
	let unlinkedDebounce: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		const tab = sidebarTab;
		clearTimeout(unlinkedDebounce);
		if (!tab) { currentUnlinkedMentions = []; return; }
		unlinkedDebounce = setTimeout(async () => {
			try {
				currentUnlinkedMentions = await scanUnlinkedMentions(tab.name, tab.path);
			} catch { currentUnlinkedMentions = []; }
		}, 500);
	});

	// Outgoing links for current note
	const currentOutgoing = $derived.by(() => {
		if (!sidebarTab) return [];
		return getOutgoingLinks(allLibraryLinks, sidebarTab.path).map(l => ({
			target: l.target,
			context: l.context,
		}));
	});

	// Tags for the active note (from frontmatter + inline #tags)
	const activeNoteTags = $derived.by(() => {
		if (!sidebarTab) return [];
		const tags: string[] = [];
		// From frontmatter properties
		for (const p of sidebarProperties) {
			if (p.key === 'tags' || p.key === 'tag') {
				if (Array.isArray(p.value)) {
					tags.push(...p.value.map((v: string) => String(v).trim()).filter(Boolean));
				} else if (typeof p.value === 'string') {
					tags.push(...p.value.split(',').map(v => v.trim()).filter(Boolean));
				}
			}
		}
		// From inline #tags in body
		const bodyText = sidebarBody || '';
		const inlineMatches = bodyText.match(/(?:^|\s)#([a-zA-Z\u0600-\u06FF][\w\u0600-\u06FF/\-]*)/g);
		if (inlineMatches) {
			for (const m of inlineMatches) {
				tags.push(m.trim().replace(/^#/, ''));
			}
		}
		return [...new Set(tags)];
	});

	// Local star: nodes/links for the active note and its direct connections
	// Uses deferred state to avoid blocking the main thread with heavy iteration
	let localStarNodes = $state<StarNode[]>([]);
	let localStarLinks = $state<StarLink[]>([]);
	let _localStarTimer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		// Track reactive dependencies (starVersion signals when plain arrays change)
		const isVisible = rightSidebarOpen && rightSidebarTab === 'star';
		const tab = sidebarTab;
		const _ver = starVersion; // reactive trigger for non-reactive starNodes/starLinks

		clearTimeout(_localStarTimer);

		if (!isVisible || !tab) {
			localStarNodes = [];
			localStarLinks = [];
			return;
		}

		// Defer computation to avoid blocking UI
		_localStarTimer = setTimeout(() => {
			const activeId = tab.name.replace(/\.md$/, '').toLowerCase();
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
		}, 50);
	});

	// Tasks sidebar: load tasks from the active note when Tasks tab is visible
	let _tasksTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		const isVisible = rightSidebarOpen && rightSidebarTab === 'tasks';
		const tab = sidebarTab;
		clearTimeout(_tasksTimer);
		if (!isVisible || !tab?.path) {
			return;
		}
		console.log('[Tasks] Effect fired, scheduling scan for:', tab.path);
		_tasksTimer = setTimeout(async () => {
			try {
				console.log('[Tasks] Scanning note tasks...');
				const result = await scanNoteTasks(tab.path, tab.libraryName, tab.libraryPath);
				console.log('[Tasks] Scan complete, found', result.tasks.length, 'tasks');
				sidebarTasks = result.tasks;
			} catch (e) {
				console.error('[Tasks] Scan failed:', e);
				sidebarTasks = [];
			}
		}, 100);
	});

	// Calendar sidebar: load note dates when Calendar tab is visible
	let _calTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		const isVisible = rightSidebarOpen && rightSidebarTab === 'calendar';
		clearTimeout(_calTimer);
		if (!isVisible) return;
		console.log('[Calendar] Effect fired, scheduling scan...');
		_calTimer = setTimeout(async () => {
			try {
				const libraryList = get(libraries);
				console.log('[Calendar] Scanning', libraryList.length, 'libraries...');
				const dateCounts: Record<string, number> = {};
				const taskCounts: Record<string, number> = {};
				const results = await Promise.all(
					libraryList.map(v => scanLibraryNoteDates(v.path, v.name))
				);
				for (const dateMap of results) {
					for (const [date, entries] of Object.entries(dateMap)) {
						dateCounts[date] = (dateCounts[date] || 0) + entries.length;
					}
				}
				// Also scan tasks for due dates
				const { scanLibraryTasks } = await import('$lib/tasks/store');
				const taskResults = await Promise.all(
					libraryList.map(v => scanLibraryTasks(v.path, v.name))
				);
				for (const result of taskResults) {
					for (const task of result.tasks) {
						if (task.due_date && !task.completed) {
							taskCounts[task.due_date] = (taskCounts[task.due_date] || 0) + 1;
						}
					}
				}
				calendarNoteDates = dateCounts;
				calendarTaskDates = taskCounts;
				console.log('[Calendar] Scan complete');
			} catch (e) { console.error('[Calendar] Scan failed:', e); }
		}, 200);
	});

	// Status bar stats from focused tab (debounced to avoid per-keystroke recompute)
	let wordCount = $state(0);
	let charCount = $state(0);
	let _wcTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		const content = sidebarTab?.content ?? '';
		charCount = content.length;
		clearTimeout(_wcTimer);
		_wcTimer = setTimeout(() => {
			wordCount = content ? (content.match(/\S+/g)?.length ?? 0) : 0;
		}, 300);
	});

	// All note names across all libraries (for quick switcher)
	const allSwitcherNotes = $derived(allNotes);

	// Dark mode
	const colorScheme = $derived($appSettings.colorScheme);
	$effect(() => {
		if (typeof document !== 'undefined') {
			const resolved = colorScheme === 'system'
				? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
				: colorScheme;
			document.body.classList.remove('theme-light', 'theme-dark');
			document.body.classList.add(`theme-${resolved}`);
		}
	});

	// Apply custom fonts at runtime
	$effect(() => {
		if (typeof document === 'undefined') return;
		const s = $appSettings;
		const root = document.documentElement.style;

		// Apply base font settings to CSS variables
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

		// Prepend virtual font families so script-specific fonts are tried first
		if (hasScriptFont) {
			root.setProperty('--font-interface-theme', `"ConstellationUI", ${uiFont}`);
			root.setProperty('--font-text-theme', `"ConstellationText", ${txtFont}`);
		} else {
			root.setProperty('--font-interface-theme', uiFont);
			root.setProperty('--font-text-theme', txtFont);
		}

		// Inject or update the style element
		let styleEl = document.getElementById('constellation-script-fonts');
		if (!styleEl) {
			styleEl = document.createElement('style');
			styleEl.id = 'constellation-script-fonts';
			document.head.appendChild(styleEl);
		}
		styleEl.textContent = css;
	});

	// Re-init idle timer when lock settings change
	$effect(() => {
		const _lockOn = $appSettings.security?.lockOnIdle;
		const _timeout = $appSettings.security?.lockIdleTimeout;
		resetIdleTimer();
	});

	// ─── Commands for command palette ───
	function sc(id: string): string { return formatShortcut(getResolvedShortcut(id, $appSettings.customShortcuts)); }

	function getCommands() {
		return [
			{ id: 'command-palette', name: $t('settings.plugins.commandPalette'), shortcut: sc('command-palette'), icon: '🚀', action: () => { showCommandPalette = !showCommandPalette; showQuickSwitcher = false; }, category: 'Navigation' },
			{ id: 'new-note', name: $t('commands.newNote'), shortcut: sc('new-note'), icon: '📄', action: handleNewNote, category: 'File' },
			{ id: 'quick-capture', name: $t('commands.quickCapture'), shortcut: sc('quick-capture'), icon: '⚡', action: handleQuickCapture, category: 'File' },
			{ id: 'new-base', name: $t('commands.newBase'), shortcut: sc('new-base'), icon: '▦', action: handleNewBase, category: 'File' },
			{ id: 'quick-switch', name: $t('commands.quickSwitcher'), shortcut: sc('quick-switch'), icon: '🔍', action: () => { showCommandPalette = false; showQuickSwitcher = true; }, category: 'Navigation' },
			{ id: 'search', name: $t('commands.searchLibrary'), shortcut: sc('search'), icon: '🔎', action: () => { sidebarOpen = true; searchMode = true; }, category: 'Navigation' },
			{ id: 'daily-note', name: $t('commands.dailyNote'), shortcut: sc('daily-note'), icon: '📅', action: handleOpenDailyNote, category: 'Daily Notes' },
			{ id: 'toggle-edit', name: $t('commands.toggleEdit'), shortcut: sc('toggle-edit'), icon: '✏️', action: () => { const tab = get(focusedTab); if (tab) toggleEditMode(tab.id); }, category: 'Editor' },
			{ id: 'star-view', name: $t('commands.starView'), shortcut: sc('star-view'), icon: '🕸️', action: () => showStarView = !showStarView, category: 'View' },
			{ id: 'global-tasks', name: $t('commands.globalTasks'), shortcut: sc('global-tasks'), icon: '☑️', action: () => { showGlobalTasks = !showGlobalTasks; showStarView = false; }, category: 'View' },
			{ id: 'insert-template', name: $t('commands.insertTemplate'), shortcut: sc('insert-template'), icon: '📋', action: () => { templatePickerMode = 'insert'; refreshTemplates(); showTemplatePicker = true; }, category: 'Templates' },
			{ id: 'toggle-bold', name: $t('commands.toggleBold'), shortcut: sc('toggle-bold'), icon: '𝐁', action: () => {}, category: 'Editor' },
			{ id: 'toggle-italic', name: $t('commands.toggleItalic'), shortcut: sc('toggle-italic'), icon: '𝐼', action: () => {}, category: 'Editor' },
			{ id: 'split-view', name: $t('commands.splitView'), shortcut: sc('split-view'), icon: '⊞', action: cycleSplit, category: 'View' },
			{ id: 'close-note', name: $t('commands.closeNote'), shortcut: sc('close-note'), icon: '✕', action: closeNote, category: 'File' },
			{ id: 'toggle-left', name: $t('commands.toggleLeftSidebar'), shortcut: sc('toggle-left'), icon: '◧', action: () => sidebarOpen = !sidebarOpen, category: 'View' },
			{ id: 'toggle-right', name: $t('commands.toggleRightSidebar'), shortcut: sc('toggle-right'), icon: '◨', action: () => rightSidebarOpen = !rightSidebarOpen, category: 'View' },
			{ id: 'add-library', name: $t('commands.addLibrary'), shortcut: sc('add-library'), icon: '📁', action: handleAddLibrary, category: 'Library' },
			{ id: 'toggle-bookmark', name: $t('commands.toggleBookmark'), shortcut: sc('toggle-bookmark'), icon: '⭐', action: handleToggleBookmark, category: 'Bookmarks' },
			{ id: 'random-note', name: $t('commands.randomNote'), shortcut: sc('random-note'), icon: '🎲', action: handleRandomNote, category: 'Navigation' },
			{ id: 'toggle-theme', name: $t('commands.toggleTheme'), shortcut: sc('toggle-theme'), icon: '🌗', action: handleToggleTheme, category: 'Appearance' },
			{ id: 'second-screen', name: $t('secondScreen.title'), shortcut: sc('second-screen'), icon: '🖥️', action: handleToggleSecondScreen, category: 'View' },
			{ id: 'send-to-screen', name: $t('secondScreen.sendToScreen'), shortcut: sc('send-to-screen'), icon: '📤', action: handleSendToSecondScreen, category: 'View' },
			{ id: 'nav-back', name: $t('commands.navBack'), shortcut: sc('nav-back'), icon: '←', action: navigateBack, category: 'Navigation' },
			{ id: 'nav-forward', name: $t('commands.navForward'), shortcut: sc('nav-forward'), icon: '→', action: navigateForward, category: 'Navigation' },
			{ id: 'workspaces', name: $t('commands.workspaces'), shortcut: sc('workspaces'), icon: '🗂️', action: () => { showCommandPalette = false; showWorkspaces = true; }, category: 'View' },
			{ id: 'index', name: $t('commands.index'), shortcut: sc('index'), icon: '📖', action: () => { showCommandPalette = false; showIndex = !showIndex; showStarView = false; showGlobalTasks = false; }, category: 'Navigation' },
			{ id: 'import-notes', name: $t('commands.importNotes'), shortcut: sc('import-notes'), icon: '📥', action: () => { showCommandPalette = false; showImporter = true; }, category: 'App' },
			{ id: 'settings', name: $t('commands.settings'), shortcut: sc('settings'), icon: '⚙️', action: () => { showCommandPalette = false; showSettings = true; }, category: 'App' },
			{ id: 'add-property', name: $t('commands.addProperty'), shortcut: sc('add-property'), icon: '✎', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:add-property')); }, category: 'Editor' },
			{ id: 'insert-link', name: $t('commands.insertLink'), shortcut: sc('insert-link'), icon: '🔗', action: () => {}, category: 'Editor' },
			{ id: 'duplicate-line', name: $t('commands.duplicateLine'), shortcut: sc('duplicate-line'), icon: '📋', action: () => {}, category: 'Editor' },
			{ id: 'toggle-comment', name: $t('commands.toggleComment'), shortcut: sc('toggle-comment'), icon: '💬', action: () => {}, category: 'Editor' },
			{ id: 'select-next', name: $t('commands.selectNextOccurrence'), shortcut: sc('select-next'), icon: '🔤', action: () => {}, category: 'Editor' },
			{ id: 'fold-all', name: $t('commands.foldAll'), shortcut: sc('fold-all'), icon: '🔽', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:fold-all')); }, category: 'Editor' },
			{ id: 'unfold-all', name: $t('commands.unfoldAll'), shortcut: sc('unfold-all'), icon: '🔼', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:unfold-all')); }, category: 'Editor' },
			{ id: 'toggle-live-preview', name: $t('commands.toggleLivePreview'), shortcut: sc('toggle-live-preview'), icon: '📖', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:toggle-live-preview')); }, category: 'Editor' },
		];
	}

	// ─── Lock screen idle detection ───
	function resetIdleTimer() {
		if (idleTimer) clearTimeout(idleTimer);
		if ($appSettings.security?.lockOnIdle && $appSettings.security?.lockPinHash) {
			const timeoutMs = ($appSettings.security.lockIdleTimeout || 5) * 60 * 1000;
			idleTimer = setTimeout(() => { isLocked = true; }, timeoutMs);
		}
	}

	let lastActivityTime = 0;
	function handleActivity() {
		if (isLocked) return;
		const now = Date.now();
		// Throttle: only reset idle timer at most once per 10 seconds
		if (now - lastActivityTime < 10_000) return;
		lastActivityTime = now;
		resetIdleTimer();
	}

	function handleUnlock() {
		isLocked = false;
		resetIdleTimer();
	}

	// ─── Universe initialization ───
	async function initializeApp() {
		// Load essential data from the active universe in parallel (each fault-tolerant)
		await Promise.all([
			loadSettings().catch(() => {}),
			loadBookmarks().catch(() => {}),
			loadWorkspaces().catch(() => {}),
			loadPropertyTypes().catch(() => {}),
			listWorkspaceBases().then(b => workspaceBases = b).catch(() => {}),
			getChildUniverses().then(c => childUniverses = c).catch(() => {}),
		]);

		// Idle detection for lock screen
		const activityEvents = ['mousemove', 'mousedown', 'keydown', 'scroll', 'touchstart'] as const;
		for (const event of activityEvents) {
			document.addEventListener(event, handleActivity, { passive: true });
		}
		resetIdleTimer();
		cleanupFns.push(() => {
			for (const event of activityEvents) {
				document.removeEventListener(event, handleActivity);
			}
			if (idleTimer) clearTimeout(idleTimer);
		});

		// Load libraries — this is what the sidebar needs
		try { await loadLibraries(); } catch { /* ignore */ }
		try { await loadAllStats(); } catch { /* ignore */ }

		// App is usable now — show UI immediately
		appReady = true;

		// Start file watchers and build caches in the background
		for (const lib of $libraries) {
			try { await startWatchingLibrary(lib.id, lib.path); } catch { /* ignore */ }
			await loadLibraryAppearance(lib.path, lib.id);
		}
		await refreshLibraryCaches();
	}

	async function handleUniverseCreated(entry: UniverseEntry) {
		await setActiveUniverse(entry.id);
		activeUniverseName = entry.name;
		showUniverseSetup = false;
		await initializeApp();
	}

	async function handleUniverseSwitch() {
		// Save current state, clear everything, re-init
		appReady = false;
		showUniverseManager = false;

		// Unwatch all libraries
		for (const lib of $libraries) {
			try { await invoke('unwatch_library', { libraryId: lib.id }); } catch { /* ignore */ }
		}

		// Clear in-memory state
		$openTabs = [];
		$activeTabId = null;
		$focusedTabId = null;
		workspaceBases = [];

		// Clear library stores so sidebar resets
		libraries.set([]);
		libraryStats.set([]);
		allLibraryLinks = [];
		allLibraryTags = {};
		allNotes = [];
		allIndexEntries = [];
		libraryTrees = {};
		expandedLibraries = new Set();
		editingTabIds.set(new Set());
		libraryAppearances.set({});
		bookmarks.set([]);

		// Reset cache guard so refreshLibraryCaches can run for the new universe
		cacheRefreshing = false;

		// Update active universe name for title bar and status bar
		try {
			const universes = await listUniverses();
			const activePath = await invoke<string | null>('get_active_universe_path');
			const active = universes.find(u => u.path === activePath);
			if (active) activeUniverseName = active.name;
		} catch { /* ignore */ }

		// Re-initialize
		await initializeApp();
	}

	// ─── Lifecycle ───
	function handleUnhandledRejection(e: PromiseRejectionEvent) {
		console.error('[Constellation] Unhandled rejection:', e.reason);
		e.preventDefault();
	}
	function handleUncaughtError(e: ErrorEvent) {
		console.error('[Constellation] Uncaught error:', e.error);
	}
	function handleTemplatePicker() { templatePickerMode = 'insert'; refreshTemplates(); showTemplatePicker = true; }

	onMount(async () => {
		// Global error handlers to prevent WebView crashes
		window.addEventListener('unhandledrejection', handleUnhandledRejection);
		window.addEventListener('error', handleUncaughtError);

		// Listen for template picker requests from CodeMirrorEditor /template slash command
		window.addEventListener('constellation:open-template-picker', handleTemplatePicker);

		// Clipboard monitoring for second screen
		document.addEventListener('copy', () => {
			if (!secondScreenOpen) return;
			setTimeout(async () => {
				try {
					const text = await navigator.clipboard.readText();
					if (text) emitClipboardCopy(text, $activeTab?.name?.replace(/\.md$/, '') || 'unknown');
				} catch {}
			}, 100);
		});

		// 1. Check universe state
		let universes: UniverseEntry[] = [];
		let needsMigration = false;
		try {
			universes = await listUniverses();
			needsMigration = await checkMigrationNeeded();
		} catch {
			// IPC not available (browser preview) — show setup
			showUniverseSetup = true;
			return;
		}

		if (universes.length === 0 && !needsMigration) {
			// First launch — show universe setup
			showUniverseSetup = true;
			return;
		}

		if (universes.length === 0 && needsMigration) {
			// Legacy data exists — migrate to default universe
			// Show setup so user picks location
			showUniverseSetup = true;
			return;
		}

		// Activate the last-active universe (or first one)
		const activeEntry = universes[0]; // registry stores active_id, set_active_universe handles it
		try {
			await setActiveUniverse(activeEntry.id);
			activeUniverseName = activeEntry.name;
		} catch {
			showUniverseSetup = true;
			return;
		}

		await initializeApp();

		// Listen for file change events from the watcher
		let pendingTreeRefresh: Set<string> = new Set();
		let pendingTabReloads: Set<string> = new Set();
		const unlistenWatcher = await listen<{ libraryId: string; paths: string[] }>('library-changed', (event) => {
			const { libraryId, paths } = event.payload;
			pendingTreeRefresh.add(libraryId);
			for (const p of paths) {
				if (!wasRecentlyWritten(p)) pendingTabReloads.add(p);
			}
			// Batch rapid file changes (300ms window)
			clearTimeout(watcherDebounce);
			watcherDebounce = setTimeout(async () => {
				const libraryIds = [...pendingTreeRefresh];
				const tabPaths = [...pendingTabReloads];
				pendingTreeRefresh.clear();
				pendingTabReloads.clear();

				// Refresh trees for changed libraries
				for (const vid of libraryIds) {
					await refreshLibraryTree(vid);
				}
				await loadAllStats();

				// Reload open tabs whose files changed
				const tabs = get(openTabs);
				for (const changedPath of tabPaths) {
					const tab = tabs.find(t => t.path === changedPath);
					if (tab) {
						try {
							const content: string = await invoke('read_note', { filePath: changedPath });
							openTabs.update(ts => ts.map(t =>
								t.path === changedPath ? { ...t, content } : t
							));
						} catch { /* file may have been deleted */ }
					}
				}

				// Debounced cache refresh for links, tags, index
				clearTimeout(cacheRefreshDebounce);
				cacheRefreshDebounce = setTimeout(() => refreshLibraryCaches(), 5000);
			}, 300);
		});

		if ($libraryStats.length === 1) {
			await toggleLibrary($libraryStats[0]);
		}

		// Second screen event listeners
		const unlistenScreenNote = await onNoteToMain(async (note: ScreenNote) => {
			await openNoteTab(note.path, note.libraryName, note.libraryPath, note.libraryColor);
		});
		const unlistenScreenClosed = await onScreenClosed(() => {
			secondScreenOpen = false;
		});

		// Global keyboard shortcuts
		document.addEventListener('keydown', handleGlobalKeydown);

		// Cleanup on destroy
		cleanupFns.push(
			() => document.removeEventListener('keydown', handleGlobalKeydown),
			unlistenWatcher,
			unlistenScreenNote,
			unlistenScreenClosed,
		);
	});

	const cleanupFns: (() => void)[] = [];
	onDestroy(() => {
		clearTimeout(searchTimeout);
		clearTimeout(previewTimeout);
		clearTimeout(cacheRefreshDebounce);
		clearTimeout(watcherDebounce);
		clearTimeout(unlinkedDebounce);
		clearTimeout(_wcTimer);
		clearTimeout(_localStarTimer);
		clearTimeout(_tasksTimer);
		clearTimeout(_calTimer);
		resizeCleanup?.();
		window.removeEventListener('unhandledrejection', handleUnhandledRejection);
		window.removeEventListener('error', handleUncaughtError);
		window.removeEventListener('constellation:open-template-picker', handleTemplatePicker);
		for (const fn of cleanupFns) fn();
	});

	let cacheRefreshing = false;
	async function refreshLibraryCaches() {
		// Prevent concurrent scans — skip if one is already in progress
		if (cacheRefreshing) return;
		cacheRefreshing = true;
		try {
			const links: NoteLink[] = [];
			const tags: Record<string, number> = {};
			const notes: { name: string; path: string; libraryName: string }[] = [];
			const indexRaw: IndexEntry[] = [];

			// Process libraries sequentially (2 at a time) to avoid IPC flood
			const libraryList = $libraries;
			for (let i = 0; i < libraryList.length; i += 2) {
				const batch = libraryList.slice(i, i + 2);
				const batchResults = await Promise.all(batch.map(async (lib) => {
					const [libLinks, libTags, libNotes, libIndex] = await Promise.all([
						scanLibraryLinks(lib.path, lib.name).catch(() => [] as NoteLink[]),
						scanLibraryTags(lib.path).catch(() => ({} as Record<string, number>)),
						invoke('collect_library_notes', { libraryPath: lib.path }).catch(() => []) as Promise<any[]>,
						scanLibraryIndex(lib.path).catch(() => [] as IndexEntry[]),
					]);
					return { lib, libLinks, libTags, libNotes, libIndex };
				}));

				for (const { lib, libLinks, libTags, libNotes, libIndex } of batchResults) {
					links.push(...libLinks);
					for (const [tag, count] of Object.entries(libTags)) {
						tags[tag] = (tags[tag] || 0) + count;
					}
					notes.push(...libNotes.map((n: any) => ({ name: n.name, path: n.path, libraryName: lib.name })));
					indexRaw.push(...libIndex);
				}
			}

			allLibraryLinks = links;
			allLibraryTags = tags;
			allNotes = notes;
			allIndexEntries = mergeIndexEntries(indexRaw);

			// Build star data from all libraries combined
			if (libraryList.length > 0) {
				const { nodes, links: gLinks } = buildStarData(links, notes);
				starNodes = nodes;
				starLinks = gLinks;
				starVersion++;
			}
		} finally {
			cacheRefreshing = false;
		}
	}

	function mergeIndexEntries(entries: IndexEntry[]): IndexEntry[] {
		const map = new Map<string, IndexEntry>();
		for (const entry of entries) {
			const key = entry.term.toLowerCase();
			const existing = map.get(key);
			if (existing) {
				// Sum counts across libraries
				existing.count += entry.count;
				// Merge mentions, deduplicate by note_path
				const existingPaths = new Set(existing.mentions.map(m => m.note_path));
				for (const m of entry.mentions) {
					if (!existingPaths.has(m.note_path)) {
						existing.mentions.push(m);
						existingPaths.add(m.note_path);
					}
				}
			} else {
				map.set(key, { ...entry, mentions: [...entry.mentions] });
			}
		}
		return Array.from(map.values()).sort((a, b) =>
			a.term.toLowerCase().localeCompare(b.term.toLowerCase())
		);
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		if (isLocked) { e.preventDefault(); e.stopPropagation(); return; }

		// Escape always closes overlays (not remappable)
		if (e.key === 'Escape') {
			if (showCommandPalette) { showCommandPalette = false; return; }
			if (showQuickSwitcher) { showQuickSwitcher = false; return; }
			if (showStarView) { showStarView = false; return; }
			if (showGlobalTasks) { showGlobalTasks = false; return; }
			if (showIndex) { showIndex = false; return; }
			if (showTemplatePicker) { showTemplatePicker = false; return; }
			if (showWorkspaces) { showWorkspaces = false; return; }
			if (showSettings) { showSettings = false; return; }
			if (showImporter) { showImporter = false; return; }
			return;
		}

		// Build lookup from shortcut combo → action
		const combo = eventToShortcut(e);
		if (!combo) return;

		const cmds = getCommands();
		for (const cmd of cmds) {
			if (cmd.shortcut && normalizeShortcut(cmd.shortcut) === combo) {
				e.preventDefault();
				cmd.action();
				return;
			}
		}

		// Check template hotkeys
		const tplHotkeys = $appSettings.templateHotkeys || {};
		for (const [shortcut, tplPath] of Object.entries(tplHotkeys)) {
			if (normalizeShortcut(shortcut) === combo) {
				e.preventDefault();
				handleTemplateSelect(tplPath, '');
				return;
			}
		}
	}

	// ─── Actions ───
	async function handleNewNote() {
		if ($libraries.length === 0) return;
		if ($libraries.length === 1) {
			await createNoteInLibrary($libraries[0]);
		} else {
			libraryPickerAction = 'note';
			showLibraryPicker = true;
		}
	}

	async function createNoteInLibrary(lib: { id: string; name: string; path: string }) {
		try {
			const baseName = $t('actions.untitled');
			let name = baseName;
			let newPath: string | null = null;

			// Build default frontmatter with auto-dates + user defaults
			const defaultFM = buildDefaultFrontmatter($appSettings);

			// Template will be resolved after path is determined (for folder templates)
			let templateBody = '';

			for (let i = 0; i < 100; i++) {
				try {
					newPath = await createNote(lib.path, name, defaultFM);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			if (!newPath) return;

			// Resolve template: check folder templates first, then default.md
			if ($appSettings.enabledFeatures?.templates) {
				try {
					const tplDir: string = await invoke('get_templates_dir');
					const noteFolder = newPath.replace(/\\/g, '/').split('/').slice(0, -1).join('/');

					// Check folder templates (deepest match wins)
					const folderTpls = $appSettings.folderTemplates || {};
					let matchedTpl = '';
					let matchDepth = -1;
					for (const [folder, tplName] of Object.entries(folderTpls)) {
						const normFolder = folder.replace(/\\/g, '/');
						if (noteFolder.includes(normFolder) || noteFolder.endsWith(normFolder)) {
							const depth = normFolder.split('/').length;
							if (depth > matchDepth) {
								matchDepth = depth;
								matchedTpl = tplName;
							}
						}
					}

					const tplFile = matchedTpl || 'default';
					const tplPath = `${tplDir}/${tplFile.endsWith('.md') ? tplFile : tplFile + '.md'}`;
					const tpl: string = await invoke('read_note', { filePath: tplPath });
					if (tpl) {
						const tplParsed = parseFrontmatter(tpl);
						templateBody = tplParsed.body;
					}
				} catch { /* no template — OK */ }
			}

			// Apply template if found
			if (templateBody.trim()) {
				try {
					const noteFolder = newPath.replace(/\\/g, '/').split('/').slice(-2, -1)[0] || '';
					const ctx = { title: name, folder: noteFolder, library: lib.name, filePath: newPath };
					const result = await processTemplateAsync(templateBody, ctx, buildTemplateCallbacks());
					const fullContent = `---\n${defaultFM}\n---\n${result.content}`;
					await invoke('write_note', { filePath: newPath, content: fullContent });
				} catch { /* template write failed — note still created */ }
			}

			await refreshLibraryTree(lib.id);
			const libraryColor = libraryColorMap[lib.name] ?? '#7c3aed';
			await openNoteTab(newPath, lib.name, libraryColor);
			// Auto-enter edit mode
			const tab = get(focusedTab);
			if (tab) toggleEditMode(tab.id);
		} catch (e) {
			console.error('Failed to create note:', e);
		}
	}

	async function handleQuickCapture() {
		if ($libraries.length === 0) return;
		const lib = $libraries[0];
		try {
			const inboxFolder = $appSettings.inboxFolder ?? '+';
			const newPath = await quickCapture(lib.path, inboxFolder);
			await refreshLibraryTree(lib.id);
			const libraryColor = libraryColorMap[lib.name] ?? '#7c3aed';
			await openNoteTab(newPath, lib.name, libraryColor);
			const tab = get(focusedTab);
			if (tab) toggleEditMode(tab.id);
		} catch (e) {
			console.error('Quick capture failed:', e);
		}
	}

	async function handleNewBase() {
		showNewBaseDialog = true;
	}

	async function createWorkspaceBaseWithLibraries(
		baseName: string,
		selectedLibraries: string[],
	) {
		try {
			let name = baseName;
			let newPath: string | null = null;

			for (let i = 0; i < 100; i++) {
				try {
					newPath = await createWorkspaceBase(name);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			if (!newPath) return;

			// Overwrite with the user's selected libraries + name
			const definition: BaseDefinition = {
				version: 1,
				name,
				source: {
					type: 'all',
					includeSubfolders: true,
					selectedLibraries: selectedLibraries.length > 0 ? selectedLibraries : undefined,
				},
				columns: [],
				filters: [],
				sorts: [],
				view: 'table',
				direction: 'auto',
			};
			await saveWorkspaceBase(newPath, definition);

			// Refresh workspace bases list
			workspaceBases = await listWorkspaceBases();

			// Open in a tab — workspace bases use the active universe name
			await openNoteTab(newPath, activeUniverseName || 'Constellation', '#7c3aed');
		} catch (e) {
			console.error('Failed to create workspace base:', e);
		}
	}

	async function handleNewFolder() {
		if ($libraries.length === 0) return;
		if ($libraries.length === 1) {
			await createFolderInLibrary($libraries[0]);
		} else {
			libraryPickerAction = 'folder';
			showLibraryPicker = true;
		}
	}

	async function createFolderInLibrary(lib: { id: string; name: string; path: string }) {
		try {
			const baseName = $t('actions.newFolder');
			let name = baseName;
			for (let i = 0; i < 100; i++) {
				try {
					await createFolder(lib.path, name);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			await refreshLibraryTree(lib.id);
			// Expand the library if not already
			if (!expandedLibraries.has(lib.id)) {
				expandedLibraries.add(lib.id);
				expandedLibraries = new Set(expandedLibraries);
			}
		} catch (e) {
			console.error('Failed to create folder:', e);
		}
	}

	function cycleSortOrder() {
		const orders: typeof sortOrder[] = ['name-asc', 'name-desc', 'modified-desc', 'modified-asc'];
		const idx = orders.indexOf(sortOrder);
		sortOrder = orders[(idx + 1) % orders.length];
	}

	function getSortTooltip(): string {
		switch (sortOrder) {
			case 'name-asc': return $t('sidebar.sortNameAsc');
			case 'name-desc': return $t('sidebar.sortNameDesc');
			case 'modified-desc': return $t('sidebar.sortModifiedDesc');
			case 'modified-asc': return $t('sidebar.sortModifiedAsc');
		}
	}

	function sortEntries(entries: FileEntry[]): FileEntry[] {
		const sorted = [...entries].sort((a, b) => {
			// Folders first always
			if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
			switch (sortOrder) {
				case 'name-asc': return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
				case 'name-desc': return b.name.localeCompare(a.name, undefined, { sensitivity: 'base' });
				case 'modified-desc': return (b.modified ?? 0) - (a.modified ?? 0);
				case 'modified-asc': return (a.modified ?? 0) - (b.modified ?? 0);
				default: return 0;
			}
		});
		return sorted.map(e => e.children ? { ...e, children: sortEntries(e.children) } : e);
	}

	function toggleCollapseAll() {
		if (expandedLibraries.size > 0) {
			expandedLibraries = new Set();
			allExpanded = false;
		} else {
			for (const lib of $libraryStats) {
				expandedLibraries.add(lib.library_id);
			}
			expandedLibraries = new Set(expandedLibraries);
			allExpanded = true;
		}
	}

	async function handleOpenDailyNote() {
		const firstLib = $libraries[0];
		if (!firstLib) return;
		try {
			const path = await getDailyNotePath(firstLib.path, $appSettings.dailyNoteFormat, $appSettings.dailyNoteFolder);
			const libraryColor = libraryColorMap[firstLib.name] ?? '#7c3aed';

			// Apply daily note template if configured and note was just created (has only date frontmatter)
			const dailyTpl = $appSettings.dailyNoteTemplate;
			if (dailyTpl && $appSettings.enabledFeatures?.templates) {
				try {
					const noteContent: string = await invoke('read_note', { filePath: path });
					if (noteContent.length < 50) {
						const tplDir: string = await invoke('get_templates_dir');
						const tplName = dailyTpl.endsWith('.md') ? dailyTpl : dailyTpl + '.md';
						const tplRaw: string = await invoke('read_note', { filePath: `${tplDir}/${tplName}` });
						const tplBody = extractTemplateBody(tplRaw);
						const fileName = path.split(/[/\\]/).pop()?.replace(/\.md$/, '') || '';
						const ctx = { title: fileName, folder: $appSettings.dailyNoteFolder || '', library: firstLib.name, filePath: path };
						const result = await processTemplateAsync(tplBody, ctx, buildTemplateCallbacks());
						const newContent = noteContent.trimEnd() + '\n' + result.content;
						await invoke('write_note', { filePath: path, content: newContent });
					}
				} catch { /* template not found — OK */ }
			}

			await openNoteTab(path, firstLib.name, libraryColor);
		} catch (e) {
			console.error('Failed to open daily note:', e);
		}
	}

	/** Cached universe-level templates list */
	let cachedTemplates = $state<{ name: string; path: string; libraryName: string }[]>([]);

	/** Refresh templates from universe .constellation/templates/ directory */
	async function refreshTemplates() {
		try {
			const entries: { name: string; path: string }[] = await invoke('list_templates');
			cachedTemplates = entries.map(e => ({ name: e.name, path: e.path, libraryName: '' }));
		} catch {
			cachedTemplates = [];
		}
	}

	/** Get list of template files (from cache, refreshes on open) */
	function getTemplateFiles(): { name: string; path: string; libraryName: string }[] {
		return cachedTemplates;
	}

	/** Handle template selection — insert content into active note */
	async function handleTemplateSelect(templatePath: string, _libraryName: string) {
		try {
			const raw: string = await invoke('read_note', { filePath: templatePath });
			const body = extractTemplateBody(raw);
			const tab = get(focusedTab);
			if (!tab) return;

			// Build context with frontmatter for async engine
			const fm = tab.content ? parseFrontmatter(tab.content) : null;
			const fmRecord: Record<string, string> = {};
			if (fm?.properties) for (const p of fm.properties) fmRecord[p.key] = p.value;

			const ctx = {
				title: tab.name.replace(/\.md$/, ''),
				folder: tab.path.split(/[/\\]/).slice(-2, -1)[0] || '',
				library: tab.libraryName,
				filePath: tab.path,
				frontmatter: fmRecord,
			};
			const result = await processTemplateAsync(body, ctx, buildTemplateCallbacks());

			if (templatePickerMode === 'insert') {
				// Insert at cursor in editor
				const pane = document.querySelector('.note-pane.active .cm-editor') as HTMLElement | null;
				if (pane) {
					// Dispatch to CodeMirror
					const cmView = (pane as any)?.cmView?.view;
					if (cmView) {
						const pos = cmView.state.selection.main.head;
						cmView.dispatch({
							changes: { from: pos, insert: result.content },
							selection: result.cursorOffset != null
								? { anchor: pos + result.cursorOffset }
								: { anchor: pos + result.content.length }
						});
						return;
					}
				}
				// Fallback: append to note content
				const currentContent = tab.content || '';
				const newContent = currentContent + '\n' + result.content;
				await invoke('write_note', { filePath: tab.path, content: newContent });
				openTabs.update(tabs => tabs.map(t => t.id === tab.id ? { ...t, content: newContent } : t));
			}
		} catch (e) {
			console.error('Failed to insert template:', e);
		}
	}

	function handleToggleBookmark() {
		const tab = get(focusedTab);
		if (!tab) return;
		if (isBookmarked(tab.path)) {
			const bm = get(bookmarks).find(b => b.path === tab.path);
			if (bm) removeBookmark(bm.id);
		} else {
			addBookmark({ type: 'note', path: tab.path, name: tab.name, libraryName: tab.libraryName });
		}
	}

	function handleRandomNote() {
		if (allNotes.length === 0) return;
		const randomNote = allNotes[Math.floor(Math.random() * allNotes.length)];
		const libraryColor = libraryColorMap[randomNote.libraryName] ?? '#7c3aed';
		openNoteTab(randomNote.path, randomNote.libraryName, libraryColor);
	}

	function handleToggleTheme() {
		const current = get(appSettings).colorScheme;
		if (current === 'dark') {
			updateSettings({ colorScheme: 'light' });
		} else {
			// 'light' or 'system' → toggle to dark
			updateSettings({ colorScheme: 'dark' });
		}
	}

	async function handleToggleSecondScreen() {
		// Check actual window state (not just local flag) to handle native X close
		const isOpen = await invoke<boolean>('is_second_screen_open');
		if (isOpen) {
			await invoke('close_second_screen');
			secondScreenOpen = false;
		} else {
			await openSecondScreen();
			secondScreenOpen = true;
		}
	}

	async function handleSendToSecondScreen() {
		const tab = get(activeTab);
		if (!tab?.path) return;
		if (!secondScreenOpen) {
			await openSecondScreen();
			secondScreenOpen = true;
			// Small delay to let window open
			await new Promise(r => setTimeout(r, 500));
		}
		await sendNoteToScreen({
			path: tab.path,
			name: tab.name,
			libraryName: tab.libraryName,
			libraryPath: tab.libraryPath,
			libraryColor: tab.libraryColor,
		});
	}

	async function handleQuickSwitchSelect(path: string, libraryName: string) {
		const libraryColor = libraryColorMap[libraryName] ?? '#7c3aed';
		await openNoteTab(path, libraryName, libraryColor);
	}

	function handleStarNodeClick(path: string, libraryName: string) {
		const lib = $libraries.find(v => v.name === libraryName);
		const color = libraryColorMap[libraryName] ?? '#7c3aed';

		if (secondScreenOpen) {
			// Pin the note on second screen — stay in Sky View
			emitSkyViewClick({
				path,
				name: path.split(/[\\/]/).pop()?.replace(/\.md$/, '') ?? '',
				libraryName,
				libraryPath: lib?.path ?? '',
				libraryColor: color,
			});
			return; // Don't leave Sky View
		}

		// No second screen — open note in main and exit Sky View
		openNoteTab(path, libraryName, color);
		showStarView = false;
	}

	function handleTagClick(tag: string) {
		searchQuery = `#${tag}`;
		searchMode = true;
		sidebarOpen = true;
		searchAllStars(`#${tag}`);
	}

	// ─── Page Preview (hover) ───
	let previewGeneration = 0;
	async function handleWikilinkHover(e: MouseEvent) {
		const target = e.target as HTMLElement;
		const link = target.closest('a.wikilink') as HTMLAnchorElement | null;
		if (!link) {
			clearTimeout(previewTimeout);
			pagePreview = { ...pagePreview, visible: false };
			return;
		}
		const linkTarget = decodeURIComponent(link.dataset.wikilink ?? '');
		if (!linkTarget || !sidebarTab) return;

		clearTimeout(previewTimeout);
		const gen = ++previewGeneration;
		previewTimeout = setTimeout(async () => {
			try {
				const libraryList = $libraries.map(v => [v.id, v.name, v.path] as [string, string, string]);
				const resolved = await invoke<{ path: string; library_name: string; library_path: string } | null>('resolve_wikilink_cross_library', { libraries: libraryList, currentLibraryPath: sidebarTab!.libraryPath, target: linkTarget });
				if (gen !== previewGeneration) return;
				if (resolved) {
					const content = await readNotePreview(resolved.path, 800);
					if (gen !== previewGeneration) return;
					pagePreview = { content, x: e.clientX, y: e.clientY, visible: true };
				}
			} catch { /* ignore */ }
		}, 400);
	}

	function handleWikilinkLeave() {
		clearTimeout(previewTimeout);
		pagePreview = { ...pagePreview, visible: false };
	}

	// ─── Index note hover preview ───
	async function handleIndexNoteHover(filePath: string, e: MouseEvent) {
		clearTimeout(previewTimeout);
		const gen = ++previewGeneration;
		const cx = e.clientX;
		const cy = e.clientY;
		previewTimeout = setTimeout(async () => {
			try {
				const content = await readNotePreview(filePath, 2000);
				if (gen !== previewGeneration) return;
				pagePreview = { content, x: cx, y: cy, visible: true };
			} catch { /* ignore */ }
		}, 300);
	}

	function handleIndexNoteLeave() {
		clearTimeout(previewTimeout);
		pagePreview = { ...pagePreview, visible: false };
	}

	// ─── Library tree operations ───
	async function toggleLibrary(lib: LibraryStats) {
		const id = lib.library_id;
		if (expandedLibraries.has(id)) {
			expandedLibraries.delete(id);
			expandedLibraries = new Set(expandedLibraries);
		} else {
			if (!libraryTrees[id]) {
				const tree: FileEntry[] = await invoke('read_library_tree', { path: lib.path, maxDepth: 4 });
				libraryTrees[id] = tree;
				libraryTrees = { ...libraryTrees };
			}
			expandedLibraries.add(id);
			expandedLibraries = new Set(expandedLibraries);
		}
	}

	async function handleAddLibrary() {
		adding = true;
		error = '';
		try { await addLibrary(); await loadAllStats(); await refreshLibraryCaches(); }
		catch (e) { error = String(e); }
		adding = false;
	}

	async function handleCreateNewLibrary() {
		const name = newLibraryName.trim() || 'My Library';
		creatingNew = true;
		error = '';
		try {
			await createNewLibrary(name);
			newLibraryName = '';
			await loadAllStats();
			await refreshLibraryCaches();
		} catch (e) { error = String(e); }
		creatingNew = false;
	}

	/** Reload everything after a child universe is added/removed. */
	async function handleChildUniverseChanged() {
		try {
			await loadLibraries();
			await loadAllStats();
			childUniverses = await getChildUniverses();
			// Start watchers for any new libraries and refresh caches
			for (const lib of $libraries) {
				try { await startWatchingLibrary(lib.id, lib.path); } catch { /* ignore */ }
			}
			await refreshLibraryCaches();
		} catch { /* ignore */ }
	}

	function handleSearch(e: Event) {
		searchQuery = (e.target as HTMLInputElement).value;
		clearTimeout(searchTimeout);
		if (searchQuery.trim()) {
			searchMode = true;
			// Property-based search: [key:value] or [key]
			const propMatch = searchQuery.trim().match(/^\[([^\]:]+)(?::(.+))?\]$/);
			if (propMatch) {
				const key = propMatch[1].trim();
				const value = propMatch[2]?.trim() ?? '';
				searchTimeout = setTimeout(async () => {
					const results = await searchByProperty(key, value);
					searchResults.set(results);
				}, 300);
			} else {
				searchTimeout = setTimeout(() => searchAllStars(searchQuery), 300);
			}
		} else {
			searchMode = false;
			searchResults.set([]);
		}
	}

	function clearSearch() {
		searchQuery = '';
		searchMode = false;
		searchResults.set([]);
	}

	function cycleSplit() {
		if (!$splitActive) {
			toggleSplit(); // off → vertical
		} else if ($splitDirection === 'vertical') {
			toggleSplitDirection(); // vertical → horizontal
		} else {
			toggleSplit(); // horizontal → off (resets to vertical for next cycle)
			splitDirection.set('vertical');
		}
	}

	// ─── Context menu state ───
	let contextMenu = $state<{ x: number; y: number; entry: FileEntry; libraryId: string } | null>(null);
	let confirmDelete = $state<{ path: string; name: string } | null>(null);
	let renamingPath = $state('');

	function handleContextMenu(entry: FileEntry, x: number, y: number, libraryId: string) {
		contextMenu = { x, y, entry, libraryId };
	}

	function getContextMenuItems(entry: FileEntry, libraryId: string) {
		const items: { label: string; icon?: string; action: () => void; danger?: boolean }[] = [];

		// Workspace bases have a simplified context menu
		if (libraryId === '__workspace__') {
			items.push({
				label: $t('actions.delete'),
				icon: '🗑️',
				action: () => handleDeleteWorkspaceBase(entry.path),
				danger: true,
			});
			return items;
		}

		if (entry.is_dir) {
			items.push({
				label: $t('actions.newNote'),
				icon: '📄',
				action: () => handleCreateNote(entry.path, libraryId)
			});
			items.push({
				label: $t('actions.newFolder'),
				icon: '📁',
				action: () => handleCreateFolder(entry.path, libraryId)
			});
		}
		items.push({
			label: $t('actions.rename'),
			icon: '✏️',
			action: () => { renamingPath = entry.path; }
		});
		items.push({
			label: $t('actions.delete'),
			icon: '🗑️',
			action: () => { confirmDelete = { path: entry.path, name: entry.name }; },
			danger: true
		});
		return items;
	}

	async function handleDeleteWorkspaceBase(filePath: string) {
		try {
			// Close tab if open
			const tab = get(openTabs).find(t => t.path === filePath);
			if (tab) closeTab(tab.id);

			await deleteWorkspaceBase(filePath);
			workspaceBases = await listWorkspaceBases();
		} catch (e) {
			console.error('Failed to delete workspace base:', e);
		}
	}

	async function handleCreateNote(folderPath: string, libraryId: string) {
		try {
			const name = $t('actions.untitled');
			const newPath = await createNote(folderPath, name);
			await refreshLibraryTree(libraryId);
			const lib = $libraries.find(v => v.id === libraryId);
			if (lib) {
				const libraryColor = libraryColorMap[lib.name] ?? '#7c3aed';
				await openNoteTab(newPath, lib.name, libraryColor);
			}
		} catch (e) {
			console.error('Failed to create note:', e);
		}
	}

	async function handleCreateBase(folderPath: string, libraryId: string) {
		try {
			const baseName = $t('bases.untitled');
			let name = baseName;
			let newPath: string | null = null;

			for (let i = 0; i < 100; i++) {
				try {
					newPath = await createBase(folderPath, name);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			if (!newPath) return;

			await refreshLibraryTree(libraryId);
			const lib = $libraries.find(v => v.id === libraryId);
			if (lib) {
				const libraryColor = libraryColorMap[lib.name] ?? '#7c3aed';
				await openNoteTab(newPath, lib.name, libraryColor);
			}
		} catch (e) {
			console.error('Failed to create base:', e);
		}
	}

	async function handleCreateFolder(parentPath: string, libraryId: string) {
		try {
			const name = $t('actions.newFolder');
			await createFolder(parentPath, name);
			await refreshLibraryTree(libraryId);
		} catch (e) {
			console.error('Failed to create folder:', e);
		}
	}

	async function handleDeleteConfirm() {
		if (!confirmDelete) return;
		try {
			const lib = $libraryStats.find(v => confirmDelete!.path.startsWith(v.path));
			await deleteItem(confirmDelete.path, true);
			if (lib) await refreshLibraryTree(lib.library_id);
			await loadAllStats();
		} catch (e) {
			console.error('Failed to delete:', e);
		}
		confirmDelete = null;
	}

	async function handleRenameComplete(oldPath: string, newName: string) {
		renamingPath = '';
		if (!oldPath || !newName) return;

		try {
			const parentDir = oldPath.substring(0, oldPath.lastIndexOf('\\') === -1 ? oldPath.lastIndexOf('/') : oldPath.lastIndexOf('\\'));
			const isDir = !oldPath.endsWith('.md');
			const newPath = parentDir + (oldPath.includes('\\') ? '\\' : '/') + (isDir ? newName : newName + '.md');

			// Get old note name for link updates
			const oldName = oldPath.split(/[\\/]/).pop()?.replace('.md', '') ?? '';

			await renameItem(oldPath, newPath);
			const lib = $libraryStats.find(v => oldPath.startsWith(v.path));
			if (lib) {
				await refreshLibraryTree(lib.library_id);
				// Auto-update links
				if ($appSettings.autoUpdateLinks && !isDir) {
					await updateLinksOnRename(lib.path, oldName, newName);
				}
			}
		} catch (e) {
			console.error('Failed to rename:', e);
		}
	}

	async function refreshLibraryTree(libraryId: string) {
		const lib = $libraryStats.find(v => v.library_id === libraryId);
		if (lib) {
			const tree: FileEntry[] = await invoke('read_library_tree', { path: lib.path, maxDepth: 4 });
			libraryTrees[libraryId] = tree;
			libraryTrees = { ...libraryTrees };
		}
	}

	async function handleNoteClick(filePath: string, _noteName: string, highlightTerm?: string, e?: MouseEvent) {
		const lib = $libraries.find(v => filePath.startsWith(v.path));
		const libraryColor = lib ? libraryColorMap[lib.name] : '#7c3aed';
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(filePath, lib?.name ?? '', libraryColor, highlightTerm, newTab);
		if (!isHome) window.location.href = '/';
	}

	async function handleIndexNoteClick(filePath: string, noteName: string, highlightTerm?: string, e?: MouseEvent) {
		// Ctrl+click or middle-click: open in a real tab (existing behavior)
		if (e && (e.ctrlKey || e.metaKey || e.button === 1)) {
			return handleNoteClick(filePath, noteName, highlightTerm, e);
		}
		// Bold the active link in the index
		indexActiveNotePath = filePath;
		// If second screen is open, send the note there
		const scOpen = secondScreenOpen || await isSecondScreenOpen().catch(() => false);
		if (scOpen) {
			secondScreenOpen = true; // sync local state
			const name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';
			const lib = $libraries.find(v => filePath.startsWith(v.path));
			await sendNoteToScreen({
				path: filePath,
				name,
				libraryName: lib?.name ?? '',
				libraryPath: lib?.path ?? '',
				libraryColor: lib ? libraryColorMap[lib.name] : '#7c3aed',
			});
			return;
		}
		// Normal click: build a standalone tab for the index split pane (no store mutation)
		try {
			const content: string = await invoke('read_note', { filePath });
			const name = filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') ?? '';
			const lib = $libraries.find(v => filePath.startsWith(v.path));
			const libraryColor = lib ? libraryColorMap[lib.name] : '#7c3aed';
			indexNoteTab = {
				id: `index_preview_${Date.now()}`,
				path: filePath,
				content,
				name,
				libraryName: lib?.name ?? '',
				libraryPath: lib?.path ?? '',
				libraryColor,
				highlightTerm,
				history: [filePath],
				historyIndex: 0,
			};
		} catch { /* ignore read errors */ }
	}

	async function handleSearchResultClick(path: string, libraryName: string, e?: MouseEvent) {
		const libraryColor = libraryColorMap[libraryName] ?? '#7c3aed';
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(path, libraryName, libraryColor, undefined, newTab);
		clearSearch();
		if (!isHome) window.location.href = '/';
	}

	// Get all tags as flat array for editor autocomplete
	const allTagsList = $derived(Object.keys(allLibraryTags));
</script>

{#if showUniverseSetup}
	<UniverseSetup
		onCreated={handleUniverseCreated}
		migrationMode={false}
	/>
{:else if !appReady}
	<div class="app-loading" dir={$dir}>
		<div class="loading-spinner"></div>
	</div>
{:else}
<div class="app" dir={$dir} class:resizing={resizing !== null} class:no-sidebar={!sidebarOpen} class:dark={colorScheme === 'dark'}>
	<!-- ═══ DOCK ═══ -->
	<div class="dock">
		<div class="dock-top">
			<button class="dock-btn" onclick={() => { sidebarOpen = true; searchMode = false; }} title={$t('ribbon.fileExplorer')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
			</button>
			<button class="dock-btn" onclick={() => { sidebarOpen = true; searchMode = true; }} title={$t('ribbon.search')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<button class="dock-btn" onclick={() => showStarView = !showStarView} title={$t('ribbon.graphView')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>
			</button>
			<button class="dock-btn" onclick={() => { showGlobalTasks = !showGlobalTasks; showStarView = false; }} title={$t('ribbon.globalTasks')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
			</button>
			<button class="dock-btn" onclick={handleOpenDailyNote} title={$t('ribbon.dailyNote')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
			</button>
			<a href="/skills" class="dock-btn" class:active={page.url.pathname === '/skills'} title={$t('ribbon.aiSkills')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01z"/></svg>
			</a>
			<button class="dock-btn" class:active={showIndex} onclick={() => { showIndex = !showIndex; showStarView = false; showGlobalTasks = false; }} title={$t('ribbon.index')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/><path d="M8 7h6"/><path d="M8 11h8"/></svg>
			</button>
		</div>
		<div class="dock-bottom">
			<button class="dock-btn" class:active={secondScreenOpen} onclick={handleToggleSecondScreen} title={$t('secondScreen.title')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="1" y="2" width="14" height="10" rx="1.5" fill="var(--background-secondary, #1e1e2e)"/><rect x="9" y="10" width="14" height="10" rx="1.5" fill="var(--background-secondary, #1e1e2e)"/></svg>
			</button>
			<button class="dock-btn" onclick={() => showUniverseManager = true} title={$t('universe.title')}>
				<svg width="16" height="16" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"><circle cx="100" cy="100" r="30" fill="#534AB7"/><circle cx="100" cy="100" r="19" fill="#3C3489"/><circle cx="45" cy="42" r="24" fill="#378ADD"/><circle cx="130" cy="52" r="20" fill="#7F77DD"/><circle cx="162" cy="110" r="16" fill="#1D9E75"/><circle cx="80" cy="158" r="13" fill="#D85A30"/></svg>
			</button>
			<button class="dock-btn" onclick={handleToggleTheme} title={$t('ribbon.toggleTheme')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
			</button>
<button class="dock-btn" onclick={() => showImporter = true} title={$t('ribbon.importNotes')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
			</button>
<button class="dock-btn" class:active={showSettings} onclick={() => showSettings = !showSettings} title={$t('ribbon.settings')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
			</button>
		</div>
	</div>

	<!-- ═══ LEFT SIDEBAR ═══ -->
	{#if sidebarOpen}
		<aside class="sidebar" style:width="{leftSidebarWidth}px">
			<div class="sidebar-toolbar">
				{#if searchMode}
					<div class="search-box">
						<svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
						<input type="text" placeholder={$t('sidebar.searchPlaceholder')} value={searchQuery} oninput={handleSearch}/>
						<button class="search-clear" onclick={clearSearch}>×</button>
					</div>
				{:else}
					<div class="toolbar-actions">
						<button class="tb-btn" onclick={handleNewNote} title={$t('sidebar.newNote')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M12 18v-6"/><path d="M9 15h6"/></svg>
						</button>
						<button class="tb-btn" onclick={handleNewBase} title={$t('sidebar.newBase')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
						</button>
						<button class="tb-btn" onclick={handleNewFolder} title={$t('sidebar.newFolder')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 10v6"/><path d="M9 13h6"/><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>
						</button>
						<button class="tb-btn" onclick={cycleSortOrder} title={getSortTooltip()}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/><path d="m21 16-4 4-4-4"/><path d="M17 20V4"/></svg>
						</button>
						<button class="tb-btn" onclick={toggleCollapseAll} title={expandedLibraries.size > 0 ? $t('sidebar.collapseAll') : $t('sidebar.expandAll')}>
							{#if expandedLibraries.size > 0}
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m7 20 5-5 5 5"/><path d="m7 4 5 5 5-5"/></svg>
							{:else}
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m7 15 5 5 5-5"/><path d="m7 9 5-5 5 5"/></svg>
							{/if}
						</button>
					</div>
				{/if}
			</div>

			<div class="sidebar-content">
				{#if searchMode && searchQuery}
					{#if $searchResults.length > 0}
						<div class="section-label">{$searchResults.length} {$t('sidebar.results')}</div>
						{#each $searchResults as star}
							<button class="s-result" class:active={$activeTab?.path === star.path} onclick={(e) => handleSearchResultClick(star.path, star.library_name, e)}>
								<div class="s-name">{star.name}</div>
								<div class="s-meta">
									<span class="s-lib-name">{star.library_name}</span>
									<span class="s-preview">{star.preview}</span>
								</div>
							</button>
						{/each}
					{:else}
						<div class="no-results">{$t('sidebar.noResults')}</div>
					{/if}
				{:else}
					<!-- Bookmarks section -->
					{#if $bookmarks.length > 0}
						<div class="section-label">{$t('sidebar.bookmarks')}</div>
						{#each $bookmarks as bm}
							<button class="s-result" onclick={(e) => handleNoteClick(bm.path, bm.name, undefined, e)}>
								<div class="s-name">⭐ {bm.name}</div>
								<div class="s-meta"><span class="s-lib-name">{bm.libraryName}</span></div>
							</button>
						{/each}
					{/if}

					<!-- Workspace Bases section -->
					{#if workspaceBases.length > 0}
						<div class="library-section">
							<button class="library-header" onclick={() => workspaceBasesExpanded = !workspaceBasesExpanded}>
								<svg class="v-chev" class:expanded={workspaceBasesExpanded} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-inline-end: 4px; opacity: 0.5;"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
								<span class="library-name">{$t('sidebar.bases')}</span>
							</button>
							{#if workspaceBasesExpanded}
								<div class="library-tree">
									{#each workspaceBases as base}
										<button
											class="ws-base-item"
											class:active={$activeTab?.path === base.path}
											onclick={() => openNoteTab(base.path, activeUniverseName || 'Constellation', '#7c3aed')}
											oncontextmenu={(e: MouseEvent) => {
												e.preventDefault();
												contextMenu = {
													x: e.clientX,
													y: e.clientY,
													entry: { name: base.name + '.base', path: base.path, is_dir: false, children: null, extension: 'base', modified: base.modified, status: null },
													libraryId: '__workspace__',
												};
											}}
										>
											<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity: 0.5; flex-shrink: 0;"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
											<span class="ws-base-name">{base.name}</span>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					{/if}

					<!-- Child Universes — listed first with globe icon -->
					{#each childUniverses as child}
						<div class="library-section">
							<div class="library-header child-universe-item">
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink: 0; opacity: 0.5;">
									<circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
								</svg>
								<span class="library-name">{child.name}</span>
								<span class="child-universe-count">{child.library_count}</span>
							</div>
						</div>
					{/each}

					<!-- Divider between universes and libraries -->
					{#if childUniverses.length > 0 && $libraryStats.length > 0}
						<div class="sidebar-divider"></div>
					{/if}

					<!-- Libraries — listed after universes with chevron for file tree -->
					{#each $libraryStats as lib}
						<div class="library-section">
							<button class="library-header" onclick={() => toggleLibrary(lib)}>
								<svg class="v-chev" class:expanded={expandedLibraries.has(lib.library_id)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<span class="library-name">{lib.name}</span>
							</button>
							{#if expandedLibraries.has(lib.library_id) && libraryTrees[lib.library_id]}
								<div class="library-tree">
									<FileTree
									entries={sortEntries(libraryTrees[lib.library_id])}
									libraryId={lib.library_id}
									libraryName={lib.name}
									color={libraryColorMap[lib.name]}
									onNoteClick={handleNoteClick}
									onContextMenu={(entry, x, y) => handleContextMenu(entry, x, y, lib.library_id)}
									{renamingPath}
									onRenameComplete={handleRenameComplete}
									{allExpanded}
								/>
								</div>
							{/if}
						</div>
					{/each}

					{#if $libraryStats.length === 0}
						<div class="empty-sidebar">
							<p>{$t('sidebar.noLibraries')}</p>
							<button class="add-first-btn" onclick={handleAddLibrary}>{$t('sidebar.addLibraryButton')}</button>
						</div>
					{/if}
				{/if}
			</div>

			{#if error}
				<div class="sidebar-error">{error}</div>
			{/if}

			<div class="sidebar-footer-wrap">
				{#if showLibrarySwitcher}
					<LibrarySwitcher
						colorMap={libraryColorMap}
						onClose={() => showLibrarySwitcher = false}
						onAddLibrary={handleAddLibrary}
						onManage={() => showLibraryManager = true}
						onManageUniverse={() => showUniverseManager = true}
						onChildUniverseChanged={handleChildUniverseChanged}
						activeUniverseName={activeUniverseName}
					/>
				{/if}
				<button class="sidebar-footer" onmousedown={(e) => e.stopPropagation()} onclick={() => showLibrarySwitcher = !showLibrarySwitcher}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 10l5-5 5 5"/><path d="M7 14l5 5 5-5"/></svg>
					<span class="footer-name">{$t('universe.title') ?? 'Universe'}</span>
				</button>
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="sidebar-resize" onmousedown={(e) => startResize('left', e)}></div>
		</aside>
	{/if}

	<!-- ═══ MAIN AREA ═══ -->
	<div class="main-area">
		<!-- Tab Bar -->
		<div class="tab-bar">
			<button class="tab-action" class:active={sidebarOpen} onclick={() => sidebarOpen = !sidebarOpen} title={$t('layout.leftSidebar')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/></svg>
			</button>
			{#if !$splitActive}
				<div class="tab-scroll">
					{#each $openTabs as tab (tab.id)}
						<button class="tab"
							class:active={$activeTabId === tab.id}
							class:pinned={tab.pinned}
							style:--library-color={libraryColorMap[tab.libraryName]}
							onclick={() => switchTab(tab.id)}
							onauxclick={(e) => { if (e.button === 1 && !tab.pinned) { e.preventDefault(); closeTab(tab.id); } }}
							oncontextmenu={(e) => { e.preventDefault(); tab.pinned = !tab.pinned; }}>
							{#if tab.pinned}
								<span class="tab-pin" title={$t('layout.pinned')}>📌</span>
							{:else if tab.libraryName}
								<span class="tab-lib-name">{tab.libraryName}</span>
							{/if}
							<span class="tab-title">{tab.name}</span>
							{#if !tab.pinned}
								<span class="tab-close" role="button" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}>×</span>
							{/if}
						</button>
					{/each}
					{#if !isHome}
						<div class="tab active">
							<span class="tab-title">{$t('layout.skills')}</span>
							<a class="tab-close" href="/">×</a>
						</div>
					{/if}
				</div>
				<button class="tab-new-btn" onclick={() => createEmptyTab()} title="New tab">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
				</button>
				<div class="tab-spacer"></div>
			{:else}
				<div class="tab-spacer"></div>
			{/if}
			<button class="tab-action" class:active={$splitActive} onclick={cycleSplit} title={$t('layout.splitView')}>
				{#if $splitActive && $splitDirection === 'horizontal'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 12h18"/></svg>
				{:else}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 3v18"/></svg>
				{/if}
			</button>
			<button class="tab-action" class:active={rightSidebarOpen} onclick={() => rightSidebarOpen = !rightSidebarOpen} title={$t('layout.rightSidebar')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M15 3v18"/></svg>
			</button>
		</div>

		<!-- Content -->
		<div class="content-area" onmouseover={handleWikilinkHover} onmouseout={handleWikilinkLeave}>
			{#if showStarView}
				<div class="star-fullscreen">
					<div class="star-header">
						<span class="star-title">{$t('layout.starViewTitle')}</span>
						<button class="star-close" onclick={() => showStarView = false}>×</button>
					</div>
					<GraphMindView
					nodes={starNodes}
					links={starLinks}
					onNodeClick={handleStarNodeClick}
					onNodeHover={(node) => {
						if (!secondScreenOpen) return;
						if (skyviewHoverTimer) clearTimeout(skyviewHoverTimer);
						skyviewHoverTimer = setTimeout(() => {
							if (!node) { emitSkyViewHover(null); return; }
							const lib = $libraries.find(v => v.name === node.libraryName);
							emitSkyViewHover({
								path: node.path,
								name: node.name,
								libraryName: node.libraryName,
								libraryPath: lib?.path ?? '',
								libraryColor: libraryColorMap[node.libraryName] ?? '#7c3aed',
							});
						}, 100);
					}}
					activeNodeId={sidebarTab?.name?.toLowerCase() ?? ''}
					skyViewSettings={$appSettings.skyView}
					{libraryColorMap}
				/>
				</div>
			{:else if showGlobalTasks}
				<GlobalTasksView
					{libraryColorMap}
					onClose={() => showGlobalTasks = false}
				/>
			{:else if showIndex}
				<div class="index-split" class:has-note={indexNoteTab}>
					{#if indexNoteTab}
						<div class="index-note-pane">
							<div class="index-note-header">
								<span class="index-note-name" dir="auto">{indexNoteTab.name}</span>
								<button class="index-close" onclick={() => { indexNoteTab = null; indexActiveNotePath = ''; }} title="Close note">×</button>
							</div>
							<NotePane tab={indexNoteTab} isFocused={true} onFocus={() => {}} color={libraryColorMap[indexNoteTab.libraryName]} splitView {libraryTrees} allTags={allTagsList} {allNotes} {libraryColorMap} />
						</div>
						<div class="index-split-divider"></div>
					{/if}
					<div class="index-panel-pane">
						<div class="index-header">
							<span class="index-title">{$t('ribbon.index')}</span>
							<button class="index-close" onclick={() => { showIndex = false; indexNoteTab = null; indexActiveNotePath = ''; }}>×</button>
						</div>
						<div class="index-body">
							<IndexPanel
								entries={allIndexEntries}
								onNoteClick={handleIndexNoteClick}
								onNoteHover={handleIndexNoteHover}
								onNoteLeave={handleIndexNoteLeave}
								activeNotePath={indexActiveNotePath}
							/>
						</div>
					</div>
				</div>
			{:else if isHome && ($activeTab || $splitActive)}
				<div class="pane-container" class:horizontal={$splitActive && $splitDirection === 'horizontal'}>
					{#if $splitActive}
						{#each $openTabs as tab, i (tab.id)}
							{#if i > 0}
								<div class="pane-divider"></div>
							{/if}
							<NotePane {tab} isFocused={$focusedTabId === tab.id} onFocus={() => setFocusedTab(tab.id)} color={libraryColorMap[tab.libraryName]} splitView {libraryTrees} allTags={allTagsList} {allNotes} {libraryColorMap} />
						{/each}
					{:else}
						<NotePane tab={$activeTab} isFocused={true} onFocus={() => {}} {libraryTrees} allTags={allTagsList} {allNotes} {libraryColorMap}
						onCreateNote={handleNewNote}
						onQuickSwitch={() => showQuickSwitcher = true}
						onCloseTab={() => { if ($activeTab) closeTab($activeTab.id); }}
					/>
					{/if}
				</div>
			{:else if isHome}
				<div class="welcome">
					{#if $libraryStats.length === 0}
						<svg class="w-icon" width="80" height="80" viewBox="0 0 160 160" fill="none">
								<defs>
									<linearGradient id="wStarGrad" x1="0%" y1="0%" x2="100%" y2="100%">
										<stop offset="0%" stop-color="#8b5cf6" />
										<stop offset="100%" stop-color="#7c3aed" />
									</linearGradient>
								</defs>
								<path d="M80,64 L85,75 L96,80 L85,85 L80,96 L75,85 L64,80 L75,75 Z" fill="url(#wStarGrad)" />
								<path d="M80,42 L82.5,47 L88,50 L82.5,53 L80,58 L77.5,53 L72,50 L77.5,47 Z" fill="url(#wStarGrad)" />
								<path d="M108,58 L110.5,63 L116,66 L110.5,69 L108,74 L105.5,69 L100,66 L105.5,63 Z" fill="url(#wStarGrad)" />
								<path d="M104,96 L106.5,101 L112,104 L106.5,107 L104,112 L101.5,107 L96,104 L101.5,101 Z" fill="url(#wStarGrad)" />
								<path d="M60,102 L62,106 L67,109 L62,112 L60,116 L58,112 L53,109 L58,106 Z" fill="url(#wStarGrad)" />
								<path d="M50,66 L52.5,71 L58,74 L52.5,77 L50,82 L47.5,77 L42,74 L47.5,71 Z" fill="url(#wStarGrad)" />
							</svg>
						<p class="w-title">{$t('libraries.welcomeTitle')}</p>
						<p class="w-sub">{$t('libraries.welcomeSubtitle')}</p>

						<div class="w-options">
							<!-- Option 1: New Library -->
							<div class="w-option-card">
								<div class="w-option-icon">📁</div>
								<p class="w-option-title">{$t('libraries.newLibrary')}</p>
								<p class="w-option-desc">{$t('libraries.newLibraryDesc')}</p>
								<div class="w-option-form">
									<input
										type="text"
										class="w-option-input"
										placeholder="My Library"
										bind:value={newLibraryName}
										onkeydown={(e) => e.key === 'Enter' && handleCreateNewLibrary()}
									/>
									<button class="w-option-btn primary" onclick={handleCreateNewLibrary} disabled={creatingNew}>
										{creatingNew ? '...' : '+ Create'}
									</button>
								</div>
							</div>

							<!-- Option 2: Link Existing -->
							<div class="w-option-card">
								<div class="w-option-icon">🔗</div>
								<p class="w-option-title">{$t('libraries.linkLibrary')}</p>
								<p class="w-option-desc">{$t('libraries.linkLibraryDesc')}</p>
								<button class="w-option-btn secondary" onclick={handleAddLibrary} disabled={adding}>
									{adding ? '...' : '📂 Browse'}
								</button>
							</div>
						</div>

						{#if error}
							<p class="w-error">{error}</p>
						{/if}
					{:else}
						<p class="w-hint">{$t('welcome.selectNote')}</p>
						<p class="w-hint-sub">{$t('welcome.quickSwitchHint')}</p>
					{/if}
				</div>
			{:else}
				<div class="page-scroll">
					{@render children()}
				</div>
			{/if}
		</div>
	</div>

	<!-- ═══ RIGHT SIDEBAR ═══ -->
	<aside class="right-sidebar" class:collapsed={!rightSidebarOpen} style:width={rightSidebarOpen ? rightSidebarWidth + 'px' : undefined}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="rs-resize" onmousedown={(e) => startResize('right', e)}></div>
		<div class="rs-inner" dir={noteDir}>
			<!-- Right sidebar tab bar -->
			<div class="rs-tabs">
				<button class="rs-tab" class:active={rightSidebarTab === 'properties'} onclick={() => rightSidebarTab = 'properties'} title={$t('panels.properties')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'backlinks'} onclick={() => rightSidebarTab = 'backlinks'} title={$t('panels.backlinks')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'tags'} onclick={() => rightSidebarTab = 'tags'} title={$t('panels.tags')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'star'} onclick={() => rightSidebarTab = 'star'} title={$t('panels.starView')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="18" r="3"/><path d="M8.5 8.5l7 7M15.5 8.5l-7 7"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'tasks'} onclick={() => rightSidebarTab = 'tasks'} title={$t('panels.tasks')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'calendar'} onclick={() => rightSidebarTab = 'calendar'} title={$t('panels.calendar')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
				</button>
			</div>

			{#if isHome && sidebarTab}
				{#if rightSidebarTab === 'properties'}
					<!-- Properties Panel (interactive editor) -->
					<div class="rs-section">
						<PropertyEditor
							properties={sidebarProperties}
							body={sidebarBody}
							tabId={sidebarTab.id}
							filePath={sidebarTab.path}
							libraryName={sidebarTab.libraryName}
							onNoteClick={async (noteName) => {
								if (!sidebarTab) return;
								const resolved = await resolveWikilinkCrossLibrary(sidebarTab.libraryPath, noteName);
								if (resolved) {
									const vc = libraryColorMap[resolved.library_name] || '#7c3aed';
									await openNoteTab(resolved.path, resolved.library_name, vc);
								}
							}}
						/>
					</div>

					<!-- Outline Panel -->
					<div class="rs-section">
						<div class="rs-header">{$t('panels.outline')}</div>
						{#if sidebarHeadings.length > 0}
							{#each sidebarHeadings as h}
								<button class="rs-heading" style="padding-inline-start: {(h.level - 1) * 12 + 8}px" onclick={() => {
									const el = document.getElementById(h.id);
									if (el) el.scrollIntoView({ behavior: 'smooth' });
								}}>
									{h.text}
								</button>
							{/each}
						{:else}
							<div class="rs-empty">{$t('panels.noHeadings')}</div>
						{/if}
					</div>
				{:else if rightSidebarTab === 'backlinks'}
					<div class="rs-section">
						<div class="rs-header">{$t('panels.backlinksHeader')}</div>
						<BacklinksPanel
							backlinks={currentBacklinks}
							unlinkedMentions={currentUnlinkedMentions}
							activeNoteName={sidebarTab?.name ?? ''}
							{libraryColorMap}
						/>
					</div>
					<div class="rs-section">
						<div class="rs-header">{$t('panels.outgoingLinksHeader')}</div>
						<OutgoingLinksPanel
							outgoingLinks={currentOutgoing}
						/>
					</div>
				{:else if rightSidebarTab === 'tags'}
					<!-- Tags for the active note -->
					<div class="rs-section">
						<div class="rs-header">{$t('panels.tags')}</div>
						{#if activeNoteTags.length > 0}
							<div class="rs-note-tags">
								{#each activeNoteTags as tag}
									<button class="rs-tag-chip" onclick={() => handleTagClick(tag)}>
										<span class="rs-tag-hash">#</span>{tag}
									</button>
								{/each}
							</div>
						{:else}
							<div class="rs-empty">{$t('panels.noTags')}</div>
						{/if}
					</div>
				{:else if rightSidebarTab === 'star'}
					<!-- Local star centered on the active note -->
					<div class="rs-section rs-full-height">
						{#if localStarNodes.length > 0}
							<LocalStarView
								nodes={localStarNodes}
								links={localStarLinks}
								onNodeClick={(nodeId) => {
									const note = allNotes.find(n => n.path === nodeId || n.name.replace(/\.md$/, '').toLowerCase() === nodeId);
									if (note) openNoteTab(note.path, note.libraryName, libraryColorMap[note.libraryName] || '#7c3aed');
								}}
								activeNodeId={sidebarTab?.name?.replace(/\.md$/, '').toLowerCase()}
							/>
						{:else}
							<div class="rs-empty-full">{$t('panels.noConnections')}</div>
						{/if}
					</div>
				{:else if rightSidebarTab === 'tasks'}
					<div class="rs-section rs-full-height">
						{#if sidebarTasks.length > 0}
							<TasksPanel
								tasks={sidebarTasks}
								onToggle={async (filePath, lineNumber) => {
									try {
										const newContent = await toggleTask(filePath, lineNumber);
										const activeTab = get(focusedTab);
										if (activeTab && activeTab.path === filePath) {
											openTabs.update(tabs => tabs.map(t => t.path === filePath ? { ...t, content: newContent } : t));
										}
										if (sidebarTab?.path) {
											const result = await scanNoteTasks(sidebarTab.path, sidebarTab.libraryName, sidebarTab.libraryPath);
											sidebarTasks = result.tasks;
										}
									} catch (e) { console.error('Toggle task failed:', e); }
								}}
								{libraryColorMap}
							/>
						{:else}
							<div class="rs-empty-full">{$t('tasksPanel.noTasks')}</div>
						{/if}
					</div>
				{:else if rightSidebarTab === 'calendar'}
					<div class="rs-section rs-full-height">
						<CalendarPanel
							noteDates={calendarNoteDates}
							taskDueDates={calendarTaskDates}
							onDayClick={async (dateStr) => {
								const libraryList = get(libraries);
								if (libraryList.length === 0) return;
								const lib = libraryList[0];
								try {
									const dailyPath: string = await invoke('get_daily_note_path', {
										libraryPath: lib.path,
										format: get(appSettings).dailyNoteFormat || '%Y-%m-%d',
										folder: get(appSettings).dailyNoteFolder || '',
									});
									const vc = libraryColorMap[lib.name] || '#7c3aed';
									await openNoteTab(dailyPath, lib.name, vc);
								} catch (e) { console.error('Daily note failed:', e); }
							}}
						/>
					</div>
				{/if}
			{:else}
				<div class="rs-empty-full">{$t('panels.noNoteSelected')}</div>
			{/if}
		</div>
	</aside>

	<!-- ═══ OVERLAYS ═══ -->
	{#if showCommandPalette}
		<CommandPalette
			commands={getCommands()}
			onClose={() => showCommandPalette = false}
		/>
	{/if}

	{#if showQuickSwitcher}
		<QuickSwitcher
			notes={allSwitcherNotes}
			onSelect={handleQuickSwitchSelect}
			onClose={() => showQuickSwitcher = false}
		/>
	{/if}

	{#if showTemplatePicker}
		<TemplatePicker
			templates={getTemplateFiles()}
			onSelect={handleTemplateSelect}
			onClose={() => showTemplatePicker = false}
		/>
	{/if}

	{#if activePrompt}
		<TemplatePrompt
			question={activePrompt.question}
			defaultValue={activePrompt.defaultValue}
			onSubmit={(val) => { activePrompt?.resolve(val); activePrompt = null; }}
			onCancel={() => { activePrompt?.resolve(null); activePrompt = null; }}
		/>
	{/if}

	{#if activeSuggester}
		<TemplateSuggester
			options={activeSuggester.options}
			onSelect={(val) => { activeSuggester?.resolve(val); activeSuggester = null; }}
			onCancel={() => { activeSuggester?.resolve(null); activeSuggester = null; }}
		/>
	{/if}

	{#if showWorkspaces}
		<WorkspaceManager
			onClose={() => showWorkspaces = false}
			getLayoutState={async () => {
				const layout: WorkspaceLayout = {
					leftSidebarOpen: sidebarOpen,
					leftSidebarWidth,
					rightSidebarOpen,
					rightSidebarTab,
					rightSidebarWidth,
				};
				let secondScreen: WorkspaceSecondScreen | undefined;
				if (secondScreenOpen) {
					// Request state from second screen via IPC
					try {
						const screenState = await new Promise<ScreenState>((resolve, reject) => {
							const timeout = setTimeout(() => reject(new Error('timeout')), 2000);
							onStateResponse((state) => {
								clearTimeout(timeout);
								resolve(state);
							}).then((unlisten) => {
								setTimeout(() => unlisten(), 2500);
							});
							requestScreenState();
						});
						secondScreen = {
							open: true,
							mode: screenState.mode,
							linkedBrowsing: screenState.linkedBrowsing,
							tabs: screenState.tabs,
							activeTabPath: screenState.activeTabPath,
						};
					} catch {
						// Fallback if second screen doesn't respond
						secondScreen = { open: true, mode: 'grid', linkedBrowsing: false, tabs: [], activeTabPath: null };
					}
				}
				return { layout, secondScreen };
			}}
			onRestore={async (layout, screen) => {
				if (layout) {
					sidebarOpen = layout.leftSidebarOpen;
					leftSidebarWidth = layout.leftSidebarWidth;
					rightSidebarOpen = layout.rightSidebarOpen;
					const validTabs = ['properties', 'backlinks', 'tags', 'star', 'tasks', 'calendar'] as const;
					rightSidebarTab = validTabs.includes(layout.rightSidebarTab as any) ? layout.rightSidebarTab as typeof rightSidebarTab : 'properties';
					rightSidebarWidth = layout.rightSidebarWidth;
				}
				if (screen?.open) {
					if (!secondScreenOpen) {
						await openSecondScreen();
						secondScreenOpen = true;
					}
					// Give second screen time to initialize, then send restore
					setTimeout(() => {
						sendWorkspaceRestore({
							mode: screen.mode as any,
							linkedBrowsing: screen.linkedBrowsing,
							tabs: screen.tabs,
							activeTabPath: screen.activeTabPath,
						});
					}, 500);
				} else if (screen && !screen.open && secondScreenOpen) {
					await closeSecondScreen();
					secondScreenOpen = false;
				}
			}}
		/>
	{/if}

	{#if showSettings}
		<SettingsModal
			onClose={() => showSettings = false}
			commands={getCommands()}
		/>
	{/if}

	{#if showLibraryManager}
		<LibraryManager
			colorMap={libraryColorMap}
			onClose={() => showLibraryManager = false}
			onRefresh={refreshLibraryCaches}
		/>
	{/if}

	{#if showImporter}
		<ImporterModal
			libraries={$libraries.map(v => ({ name: v.name, path: v.path }))}
			onClose={() => showImporter = false}
			onImportComplete={refreshLibraryCaches}
		/>
	{/if}

	{#if showLibraryPicker}
		<LibraryPicker
			colorMap={libraryColorMap}
			onSelect={(lib) => libraryPickerAction === 'folder' ? createFolderInLibrary(lib) : createNoteInLibrary(lib)}
			onClose={() => showLibraryPicker = false}
		/>
	{/if}

	{#if showNewBaseDialog}
		<NewBaseDialog
			colorMap={libraryColorMap}
			onCreate={(_lib, name, selectedLibraries) => createWorkspaceBaseWithLibraries(name, selectedLibraries)}
			onClose={() => showNewBaseDialog = false}
		/>
	{/if}

	{#if contextMenu}
		<ContextMenu
			x={contextMenu.x}
			y={contextMenu.y}
			items={getContextMenuItems(contextMenu.entry, contextMenu.libraryId)}
			onClose={() => contextMenu = null}
		/>
	{/if}

	{#if confirmDelete}
		<ConfirmDialog
			message={$t('dialogs.confirmDelete', { name: confirmDelete.name })}
			confirmLabel={$t('dialogs.delete')}
			cancelLabel={$t('dialogs.cancel')}
			onConfirm={handleDeleteConfirm}
			onCancel={() => confirmDelete = null}
		/>
	{/if}

	<PagePreview
		content={pagePreview.content}
		x={pagePreview.x}
		y={pagePreview.y}
		visible={pagePreview.visible}
	/>

	{#if isLocked}
		<LockScreen onUnlock={handleUnlock} />
	{/if}

	<!-- ═══ STATUS BAR ═══ -->
	<div class="status-bar">
		<div class="sb-left">
			{#if sidebarTab}
				<span class="sb-item">{sidebarTab.libraryName}</span>
				<span class="sb-dot">·</span>
				<span class="sb-item">{sidebarTab.name}</span>
			{:else}
				<span class="sb-item">{$t('libraryManager.manageLibraries')}</span>
			{/if}
		</div>
		<div class="sb-right">
			{#if sidebarTab}
				{#if sidebarProperties.length > 0}
					<span class="sb-item">{sidebarProperties.length} {$t('statusBar.properties')}</span>
					<span class="sb-dot">·</span>
				{/if}
				<span class="sb-item">{wordCount} {$t('statusBar.words')}</span>
				<span class="sb-dot">·</span>
				<span class="sb-item">{charCount} {$t('statusBar.characters')}</span>
				<span class="sb-dot">·</span>
			{/if}
			<span class="sb-item">{$libraryCount} {$t('statusBar.libraries')}</span>
			<span class="sb-dot">·</span>
			<span class="sb-item">{$totalStars} {$t('statusBar.notes')}</span>
			{#if activeUniverseName}
				<span class="sb-dot">·</span>
				<button class="sb-universe" onclick={() => showUniverseManager = true} title={$t('universe.manager.heading')}>
					<svg width="10" height="10" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"><circle cx="100" cy="100" r="30" fill="#534AB7"/><circle cx="100" cy="100" r="19" fill="#3C3489"/><circle cx="45" cy="42" r="24" fill="#378ADD"/><circle cx="130" cy="52" r="20" fill="#7F77DD"/><circle cx="162" cy="110" r="16" fill="#1D9E75"/><circle cx="80" cy="158" r="13" fill="#D85A30"/></svg>
					{activeUniverseName}
				</button>
			{/if}
		</div>
	</div>
</div>
{/if}

{#if showUniverseManager}
	<UniverseManager
		onClose={() => showUniverseManager = false}
		onSwitch={handleUniverseSwitch}
		onRemoveLast={() => { showUniverseManager = false; appReady = false; showUniverseSetup = true; }}
	/>
{/if}

<style>
	:global(html) { margin: 0; padding: 0; overflow: hidden; }
	:global(body) {
		margin: 0; padding: 0;
		font-family: var(--font-interface-theme);
		font-size: var(--font-text-size); line-height: var(--line-height-normal); overflow: hidden;
		background: var(--background-primary);
		color: var(--text-normal);
	}
	:global(a) { text-decoration: none; color: var(--text-accent); }
	:global(a:hover) { color: var(--text-accent-hover); }
	:global(*) { box-sizing: border-box; }

	/* ═══ APP GRID ═══ */
	.app {
		height: 100vh;
		display: grid;
		grid-template-columns: auto auto 1fr auto;
		grid-template-rows: 1fr 24px;
		overflow: hidden;
	}
	.app.no-sidebar {
		grid-template-columns: auto 1fr auto;
	}

	/* ═══ DOCK ═══ */
	.dock {
		grid-row: 1; width: 40px; background: var(--bg-tertiary);
		border-inline-end: 1px solid var(--border);
		display: flex; flex-direction: column;
		justify-content: space-between; align-items: center; padding: 6px 0;
	}
	.dock-top, .dock-bottom { display: flex; flex-direction: column; align-items: center; gap: 1px; }
	.dock-btn {
		width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
		border-radius: 4px; border: none; background: none;
		color: var(--text-secondary); cursor: pointer; text-decoration: none; transition: all 0.1s;
	}
	.dock-btn:hover { background: var(--border); color: var(--text); }
	.dock-btn.active { color: var(--accent); }

	/* ═══ LEFT SIDEBAR ═══ */
	.sidebar {
		grid-row: 1; background: var(--bg-secondary);
		border-inline-end: 1px solid var(--border);
		display: flex; flex-direction: column; overflow: hidden;
		position: relative;
	}
	.sidebar-toolbar {
		padding: 4px 6px; border-bottom: 1px solid var(--border);
		min-height: 34px; display: flex; align-items: center;
	}
	.toolbar-actions { display: flex; gap: 2px; align-items: center; }
	.tb-btn {
		width: 26px; height: 26px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
	}
	.tb-btn:hover { background: var(--border); color: var(--text); }

	.search-box {
		display: flex; align-items: center; gap: 4px; width: 100%;
		background: var(--bg); border: 1px solid var(--border); border-radius: 4px; padding: 0 6px;
	}
	.search-icon { color: var(--text-muted); flex-shrink: 0; }
	.search-box input {
		flex: 1; border: none; background: none; padding: 4px 0;
		font-size: 0.82rem; color: var(--text); font-family: inherit; outline: none;
	}
	.search-box input::placeholder { color: var(--text-faint); }
	.search-clear { border: none; background: none; color: var(--text-muted); cursor: pointer; font-size: 1rem; padding: 0 2px; }

	.sidebar-content { flex: 1; overflow-y: auto; padding: 2px 0; }
	.section-label { padding: 4px 12px; font-size: 0.7rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.04em; }
	.s-result {
		display: block; width: 100%; padding: 4px 12px;
		background: none; border: none; color: var(--text);
		font-family: inherit; cursor: pointer; text-align: start;
	}
	.s-result:hover { background: var(--bg-hover); }
	.s-result.active { background: var(--accent-bg); }
	.s-name { font-size: 0.82rem; font-weight: 500; }
	.s-meta { display: flex; gap: 4px; font-size: 0.7rem; color: var(--text-muted); margin-top: 1px; }
	.s-lib-name { color: var(--accent); flex-shrink: 0; }
	.s-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.no-results { padding: 20px; text-align: center; color: var(--text-muted); font-size: 0.82rem; }

	.library-header {
		display: flex; align-items: center; gap: 4px; width: 100%; padding: 3px 12px;
		background: none; border: none; color: var(--text-secondary);
		font-size: 0.8rem; font-weight: 600; font-family: inherit; cursor: pointer; text-align: start;
	}
	.library-header:hover { background: var(--bg-hover); }
	.v-chev { color: var(--text-muted); flex-shrink: 0; transition: transform 0.15s ease; }
	.v-chev.expanded { transform: rotate(90deg); }
	.library-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.library-tree { padding-inline-start: 8px; }

	/* Sidebar divider between libraries and child universes */
	.sidebar-divider {
		height: 1px;
		background: var(--background-modifier-border);
		margin: 6px 12px;
	}

	/* Child universe items in sidebar */
	.child-universe-header {
		opacity: 0.85;
	}
	.child-universe-item {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 3px 12px 3px 20px;
		font-size: 0.8rem;
		color: var(--text-muted);
	}
	.child-universe-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.child-universe-count {
		font-size: 0.7rem;
		color: var(--text-faint);
	}

	.ws-base-item {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 3px 12px 3px 20px; background: none; border: none;
		color: var(--text-secondary); font-size: 0.8rem; font-family: inherit;
		cursor: pointer; text-align: start; border-radius: 3px;
	}
	.ws-base-item:hover { background: var(--bg-hover); color: var(--text-normal); }
	.ws-base-item.active { background: var(--bg-active); color: var(--text-normal); }
	.ws-base-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.empty-sidebar { padding: 20px 16px; text-align: center; }
	.empty-sidebar p { color: var(--text-muted); font-size: 0.85rem; margin-bottom: 10px; }
	.add-first-btn {
		background: none; border: 1px dashed var(--border); border-radius: 4px;
		padding: 4px 12px; color: var(--text-muted); font-size: 0.82rem; cursor: pointer; font-family: inherit;
	}
	.add-first-btn:hover { border-color: var(--accent); color: var(--accent); }
	.sidebar-error { padding: 6px 12px; color: var(--danger); font-size: 0.75rem; }

	.sidebar-footer-wrap {
		position: relative;
	}
	.sidebar-footer {
		width: 100%;
		border-top: 1px solid var(--border); padding: 4px 12px;
		display: flex; align-items: center; gap: 6px; min-height: 30px;
		background: none; border-inline: none; border-bottom: none;
		cursor: pointer; font-family: inherit;
	}
	.sidebar-footer:hover { background: var(--bg-hover); }
	.sidebar-footer svg { color: var(--text-muted); flex-shrink: 0; }
	.footer-name { font-size: 0.78rem; font-weight: 600; color: var(--text-secondary); }

	/* ═══ MAIN AREA ═══ */
	.main-area {
		grid-row: 1; display: flex; flex-direction: column;
		overflow: hidden; background: var(--bg);
	}

	/* Tab bar */
	.tab-bar {
		display: flex; align-items: flex-end;
		background: var(--bg-tertiary); border-bottom: 1px solid var(--border);
		min-height: 36px; flex-shrink: 0;
	}
	.tab-scroll {
		min-width: 0; display: flex; align-items: flex-end;
		gap: 1px; padding: 12px 4px 0; overflow-x: auto;
	}
	.tab-scroll::-webkit-scrollbar { height: 0; }
	.tab {
		display: flex; align-items: center; gap: 6px;
		padding: 5px 10px; font-size: 0.8rem; color: var(--text-secondary);
		background: var(--bg-hover); border-radius: 6px 6px 0 0;
		cursor: pointer; min-width: 0;
		border: none; font-family: inherit; flex-shrink: 0;
		border-top: 3px solid var(--library-color, transparent);
		position: relative;
	}
	.tab.active, .tab.focused {
		background: var(--bg); color: var(--text);
		border: 1px solid var(--border);
		border-top: 3px solid var(--library-color, var(--accent));
		border-bottom: 1px solid var(--bg);
		margin-bottom: -1px;
	}
	.tab-lib-name {
		position: absolute; bottom: 100%; inset-inline-end: 8px;
		font-size: 0.55rem; line-height: 1.3; letter-spacing: 0.02em;
		color: var(--text);
		background: var(--bg-tertiary);
		padding: 0 5px;
		border-radius: 3px 3px 0 0;
		border: 1px solid var(--border); border-bottom: none;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%; pointer-events: none;
	}
	.tab-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.tab.pinned { min-width: 36px; padding: 0 8px; }
	.tab-pin { font-size: 0.65rem; flex-shrink: 0; }
	.tab-close {
		background: none; border: none; color: var(--text-muted);
		cursor: pointer; font-size: 0.85rem; padding: 0; line-height: 1;
		border-radius: 3px; text-decoration: none;
		display: flex; align-items: center; justify-content: center;
		width: 16px; height: 16px; flex-shrink: 0;
	}
	.tab-close:hover { background: var(--border); color: var(--text); }
	.tab-action {
		width: 28px; height: 28px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 4px;
		color: var(--text-muted); cursor: pointer; flex-shrink: 0; margin: auto 2px;
	}
	.tab-action:hover { background: var(--border); color: var(--text); }
	.tab-action.active { color: var(--accent); }
	.tab-spacer { flex: 1; }
	.tab-new-btn {
		width: 28px; height: 28px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 4px;
		color: var(--text-muted); cursor: pointer; flex-shrink: 0; align-self: flex-end; margin-bottom: 2px;
	}
	.tab-new-btn:hover { background: var(--border); color: var(--text); }

	/* Content */
	.content-area { flex: 1; overflow: hidden; display: flex; flex-direction: column; }

	/* Pane container */
	.pane-container {
		flex: 1; display: flex; flex-direction: row; overflow: hidden;
	}
	.pane-container.horizontal { flex-direction: column; }
	.pane-divider { flex-shrink: 0; background: var(--border); }
	.pane-container:not(.horizontal) > .pane-divider { width: 3px; cursor: col-resize; }
	.pane-container.horizontal > .pane-divider { height: 3px; cursor: row-resize; }
	.pane-divider:hover { background: var(--accent); }

	/* Star fullscreen */
	.star-fullscreen {
		flex: 1; display: flex; flex-direction: column; overflow: hidden;
	}
	.star-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 8px 16px; border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
	}
	.star-title { font-weight: 600; font-size: 0.9rem; }
	.star-close {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
		font-size: 1.2rem;
	}
	.star-close:hover { background: var(--border); color: var(--text); }

	/* Index split view */
	.index-split {
		flex: 1; display: flex; flex-direction: row; overflow: hidden;
	}
	.index-panel-pane {
		flex: 1; display: flex; flex-direction: column; overflow: hidden;
		min-width: 0;
	}
	.index-split.has-note .index-panel-pane {
		flex: 1;
	}
	.index-note-pane {
		flex: 1; display: flex; flex-direction: column; overflow: hidden;
		min-width: 0;
	}
	.index-note-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 4px 12px; border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
	}
	.index-note-name {
		font-size: 0.8rem; font-weight: 500; color: var(--text-muted);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.index-split-divider {
		flex-shrink: 0; width: 3px; background: var(--border); cursor: col-resize;
	}
	.index-split-divider:hover { background: var(--interactive-accent); }
	.index-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 8px 16px; border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
	}
	.index-title { font-weight: 600; font-size: 0.9rem; }
	.index-close {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
		font-size: 1.2rem;
	}
	.index-close:hover { background: var(--border); color: var(--text); }
	.index-body {
		flex: 1; overflow: auto;
	}

	/* Welcome */
	.welcome {
		flex: 1; display: flex; flex-direction: column;
		align-items: center; justify-content: center; text-align: center; padding: 2rem; color: var(--text-muted);
	}
	.w-icon { margin-bottom: 20px; opacity: 0.7; }
	.w-title { color: var(--text); font-size: 1.2rem; font-weight: 600; margin: 0 0 4px; }
	.w-sub { font-size: 0.9rem; margin: 0 0 16px; }
	.w-hint { font-size: 0.9rem; }
	.w-hint-sub { font-size: 0.78rem; color: var(--text-faint); margin-top: 4px; }
	.w-btn {
		background: var(--accent); border: none; color: var(--text-on-accent);
		padding: 8px 20px; border-radius: 6px; cursor: pointer;
		font-size: 0.9rem; font-weight: 600;
	}
	.w-btn:hover { background: var(--accent-hover); }

	/* Welcome option cards */
	.w-options {
		display: flex; gap: 1.25rem; max-width: 580px; width: 100%; margin-top: 0.5rem;
	}
	.w-option-card {
		flex: 1; background: var(--bg-primary); border: 1px solid var(--border);
		border-radius: 12px; padding: 1.5rem; text-align: center;
		display: flex; flex-direction: column; align-items: center; gap: 0.3rem;
		transition: border-color 0.15s, box-shadow 0.15s;
	}
	.w-option-card:hover {
		border-color: var(--accent);
		box-shadow: 0 2px 12px color-mix(in srgb, var(--accent) 10%, transparent);
	}
	.w-option-icon { font-size: 1.8rem; margin-bottom: 0.2rem; }
	.w-option-title { color: var(--text); font-size: 0.95rem; font-weight: 600; margin: 0; }
	.w-option-desc { color: var(--text-muted); font-size: 0.8rem; margin: 0 0 0.75rem 0; line-height: 1.45; }
	.w-option-form { display: flex; flex-direction: column; gap: 0.5rem; width: 100%; }
	.w-option-input {
		width: 100%; padding: 0.45rem 0.6rem; border: 1px solid var(--border);
		border-radius: 6px; font-size: 0.85rem; font-family: inherit; text-align: center;
		color: var(--text); background: var(--bg-secondary);
	}
	.w-option-input:focus { border-color: var(--accent); outline: none; background: var(--bg-primary); }
	.w-option-btn {
		padding: 0.5rem 1rem; border-radius: 8px; font-size: 0.88rem; font-weight: 600;
		font-family: inherit; cursor: pointer; border: none; width: 100%; transition: background 0.15s;
	}
	.w-option-btn.primary { background: var(--accent); color: var(--text-on-accent); }
	.w-option-btn.primary:hover { background: var(--accent-hover); }
	.w-option-btn.secondary { background: var(--bg-secondary); color: var(--text); border: 1px solid var(--border); }
	.w-option-btn.secondary:hover { border-color: var(--accent); color: var(--accent); }
	.w-option-btn:disabled { opacity: 0.6; cursor: default; }
	.w-error { margin-top: 1rem; color: var(--danger, #cf222e); font-size: 0.85rem; }

	.page-scroll { flex: 1; overflow-y: auto; padding: 2rem; max-width: 900px; margin: 0 auto; width: 100%; }

	/* ═══ RIGHT SIDEBAR ═══ */
	.right-sidebar {
		grid-row: 1; background: var(--bg-secondary);
		border-inline-start: 1px solid var(--border);
		overflow: hidden;
		transition: width 0.2s ease;
		position: relative;
	}
	.right-sidebar.collapsed { width: 0 !important; border-inline-start: none; }
	.rs-inner {
		height: 100%;
		display: flex; flex-direction: column; overflow-y: auto; overflow-x: hidden;
	}

	.rs-tabs {
		display: flex; border-bottom: 1px solid var(--border); flex-shrink: 0;
	}
	.rs-tab {
		flex: 1; display: flex; align-items: center; justify-content: center;
		height: 30px; border: none; background: none;
		color: var(--text-muted); cursor: pointer;
		border-bottom: 2px solid transparent;
	}
	.rs-tab:hover { background: var(--bg-hover); color: var(--text); }
	.rs-tab.active { color: var(--accent); border-bottom-color: var(--accent); }

	.rs-section {
		padding: 12px; border-bottom: 1px solid var(--border-light);
	}
	.rs-section.rs-full-height {
		flex: 1; display: flex; flex-direction: column;
		padding: 0; border-bottom: none; overflow: hidden;
		min-height: 0;
	}
	.rs-header {
		font-size: 0.78rem; font-weight: 600; color: var(--accent);
		margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.03em;
	}
	.rs-prop {
		display: flex; justify-content: space-between; gap: 8px;
		padding: 3px 0; font-size: 0.8rem;
	}
	.rs-prop-icon { color: var(--text-muted); font-size: 0.75rem; flex-shrink: 0; width: 16px; text-align: center; }
	.rs-prop-key { color: var(--text-secondary); font-weight: 500; }
	.rs-prop-val { color: var(--text); text-align: end; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.rs-heading {
		display: block; width: 100%; border: none; background: none;
		text-align: start; font-family: inherit;
		padding: 3px 8px; font-size: 0.8rem; color: var(--text-secondary);
		cursor: pointer; border-radius: 3px;
	}
	.rs-heading:hover { background: var(--bg-hover); color: var(--text); }
	.rs-empty { font-size: 0.8rem; color: var(--text-faint); }
	.rs-empty-full {
		padding: 24px; text-align: center; color: var(--text-faint); font-size: 0.85rem;
	}
	.rs-note-tags {
		display: flex; flex-wrap: wrap; gap: 6px;
	}
	.rs-tag-chip {
		display: inline-flex; align-items: center; gap: 2px;
		padding: 3px 8px; border-radius: 12px;
		background: var(--bg-hover); border: 1px solid var(--border-light);
		font-size: 0.78rem; color: var(--text-secondary);
		cursor: pointer; font-family: inherit;
	}
	.rs-tag-chip:hover { background: var(--accent-bg, rgba(124, 58, 237, 0.1)); color: var(--accent); border-color: var(--accent); }
	.rs-tag-hash { color: var(--accent); font-weight: 600; }

	/* ═══ STATUS BAR ═══ */
	.status-bar {
		grid-column: 1 / -1; grid-row: 2; height: 24px;
		background: var(--bg-tertiary); border-top: 1px solid var(--border);
		display: flex; align-items: center; justify-content: space-between;
		padding: 0 10px; font-size: 0.7rem; color: var(--text-muted);
	}
	.sb-left, .sb-right { display: flex; align-items: center; gap: 4px; }
	.sb-dot { color: var(--border); }
	.sb-universe {
		display: flex; align-items: center; gap: 3px;
		border: none; background: none; color: var(--text-secondary);
		font-size: inherit; font-family: inherit; cursor: pointer; padding: 0;
	}
	.sb-universe:hover { color: var(--interactive-accent); }

	/* ═══ RESIZE HANDLES ═══ */
	.app.resizing { user-select: none; cursor: col-resize; }
	.sidebar-resize {
		position: absolute; top: 0; inset-inline-end: 0;
		width: 4px; height: 100%;
		cursor: col-resize; z-index: 10;
	}
	.sidebar-resize:hover, .app.resizing .sidebar-resize { background: var(--accent); }
	.rs-resize {
		position: absolute; top: 0; inset-inline-start: -3px;
		width: 7px; height: 100%;
		cursor: col-resize; z-index: 10;
	}
	.rs-resize:hover, .app.resizing .rs-resize { background: var(--accent); opacity: 0.5; }

	/* ═══ APP LOADING ═══ */
	.app-loading {
		height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--background-primary, #1e1e2e);
	}
	.loading-spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--background-modifier-border, #333);
		border-top-color: var(--interactive-accent, #7c3aed);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>

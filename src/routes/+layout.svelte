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
		openNoteTab, closeTab, switchTab, reorderTab, closeNote, createEmptyTab,
		toggleSplit, toggleSplitDirection, setFocusedTab,
		parseFrontmatter, extractHeadings, saveTabContent, updateTabContent, buildFullContent, writeNote, markRecentWrite, setWriteAhead, getWriteAhead, clearWriteAhead,
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
	import type { LibraryStats, FileEntry, WorkspaceLayout, WorkspaceSecondScreen, FontSet } from '$lib/libraries/store';
	import { BUILTIN_FONT_SETS, SCRIPT_UNICODE_RANGES, TYPEWRITER_FONTS, getFontSetById } from '$lib/libraries/store';
	import { get } from 'svelte/store';
	import { detectDir, eventToShortcut, normalizeShortcut, getResolvedShortcut, formatShortcut } from '$lib/utils';
	import { createBase, saveBaseFile, listWorkspaceBases, createWorkspaceBase, saveWorkspaceBase, deleteWorkspaceBase } from '$lib/bases/store';
	import type { WorkspaceBaseEntry } from '$lib/bases/store';
	import type { BaseDefinition } from '$lib/bases/types';
	import FileTree from '$lib/components/FileTree.svelte';
	import NotebookNavigator from '$lib/components/NotebookNavigator.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import FocusPane from '$lib/components/FocusPane.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import QuickSwitcher from '$lib/components/QuickSwitcher.svelte';
	import TemplatePicker from '$lib/components/TemplatePicker.svelte';
	import TemplatePrompt from '$lib/components/TemplatePrompt.svelte';
	import TemplateSuggester from '$lib/components/TemplateSuggester.svelte';
	import { processTemplate, processTemplateAsync, extractTemplateBody, type TemplateCallbacks } from '$lib/templates/engine';
	import GraphMindView from '$lib/components/GraphMindView.svelte';
	import OrgChart from '$lib/components/OrgChart.svelte';
	import LocalStarView from '$lib/components/LocalStarView.svelte';
	import NoteGrid from '$lib/components/NoteGrid.svelte';
	import BacklinksPanel from '$lib/components/BacklinksPanel.svelte';
	import TagsPanel from '$lib/components/TagsPanel.svelte';

	import TasksPanel from '$lib/components/TasksPanel.svelte';
	import CalendarPanel from '$lib/components/CalendarPanel.svelte';
	import GlobalTasksView from '$lib/components/GlobalTasksView.svelte';
	import TensionPanel from '$lib/components/TensionPanel.svelte';
	import ProvenancePanel from '$lib/components/ProvenancePanel.svelte';
	import ReviewPulsePanel from '$lib/components/ReviewPulsePanel.svelte';
	import ExpressionForge from '$lib/components/ExpressionForge.svelte';
	import SenseMakingCanvas from '$lib/components/SenseMakingCanvas.svelte';
	// import Inspector360 from '$lib/components/Inspector360.svelte'; // CE Phase 12: disabled — revisit later
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
	import { openSecondScreen, closeSecondScreen, isSecondScreenOpen, sendNoteToScreen, onNoteToMain, onScreenClosed, notifyUniverseSwitch, notifySettingsChanged, requestScreenState, onStateResponse, sendWorkspaceRestore, emitContextChanged, emitSkyViewHover, emitSkyViewClick, emitSidebarModeChanged, type ScreenNote, type ScreenState, type SkyViewNodeInfo } from '$lib/secondScreen';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	// Sidebar state
	let sidebarOpen = $state(true);
	let dragTabId: string | null = $state(null);
	let dragOverTabId: string | null = $state(null);
	let dragStartX = 0;
	let isDragging = false;
	let tabScrollEl: HTMLDivElement;
	let canScrollStart = $state(false);
	let canScrollEnd = $state(false);

	function updateTabScrollArrows() {
		if (!tabScrollEl) return;
		const { scrollLeft, scrollWidth, clientWidth } = tabScrollEl;
		const isRTL = document.dir === 'rtl' || document.documentElement.dir === 'rtl';
		if (isRTL) {
			canScrollEnd = scrollLeft < 0;
			canScrollStart = scrollLeft > -(scrollWidth - clientWidth);
		} else {
			canScrollStart = scrollLeft > 0;
			canScrollEnd = scrollLeft + clientWidth < scrollWidth - 1;
		}
	}

	function scrollTabs(direction: 'start' | 'end') {
		if (!tabScrollEl) return;
		const amount = direction === 'end' ? 150 : -150;
		tabScrollEl.scrollBy({ left: amount, behavior: 'smooth' });
		setTimeout(updateTabScrollArrows, 200);
	}

	let _tabScrollTimer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		// Re-check arrows when tabs change
		const _tabs = $openTabs;
		clearTimeout(_tabScrollTimer);
		_tabScrollTimer = setTimeout(updateTabScrollArrows, 50);
	});

	function startTabDrag(e: MouseEvent, tabId: string) {
		if (e.button !== 0) return; // left click only
		dragTabId = tabId;
		dragStartX = e.clientX;
		isDragging = false;

		const onMove = (me: MouseEvent) => {
			if (!isDragging && Math.abs(me.clientX - dragStartX) > 5) {
				isDragging = true;
				document.body.style.cursor = 'grabbing';
				// Add dragging class to the source tab
				const srcEl = document.querySelector(`.tab[data-tab-id="${dragTabId}"]`) as HTMLElement;
				if (srcEl) srcEl.classList.add('tab-dragging');
			}
			if (!isDragging) return;
			// Find which tab element we're over
			const els = document.querySelectorAll('.tab-scroll .tab');
			let found: string | null = null;
			els.forEach((el) => {
				const rect = el.getBoundingClientRect();
				if (me.clientX >= rect.left && me.clientX <= rect.right && me.clientY >= rect.top && me.clientY <= rect.bottom) {
					const id = el.getAttribute('data-tab-id');
					if (id && id !== dragTabId) found = id;
				}
			});
			dragOverTabId = found;
		};

		const onUp = () => {
			if (isDragging && dragTabId && dragOverTabId) {
				reorderTab(dragTabId, dragOverTabId);
			}
			// Clean up dragging visuals
			document.body.style.cursor = '';
			const srcEl = document.querySelector('.tab-dragging');
			if (srcEl) srcEl.classList.remove('tab-dragging');
			dragTabId = null;
			dragOverTabId = null;
			isDragging = false;
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		};

		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	// Tab context menu
	let tabCtxMenu = $state<{ x: number; y: number; tabId: string } | null>(null);

	function showTabContextMenu(e: MouseEvent, tabId: string) {
		e.preventDefault();
		tabCtxMenu = { x: e.clientX, y: e.clientY, tabId };
		const close = () => { tabCtxMenu = null; window.removeEventListener('click', close); };
		setTimeout(() => window.addEventListener('click', close), 0);
	}

	function tabCtxAction(action: string) {
		if (!tabCtxMenu) return;
		const id = tabCtxMenu.tabId;
		const tabs = get(openTabs);
		const tab = tabs.find(t => t.id === id);
		switch (action) {
			case 'close':
				if (tab && !tab.pinned) closeTab(id);
				break;
			case 'closeOthers':
				tabs.filter(t => t.id !== id && !t.pinned).forEach(t => closeTab(t.id));
				break;
			case 'closeRight': {
				const idx = tabs.findIndex(t => t.id === id);
				tabs.filter((t, i) => i > idx && !t.pinned).forEach(t => closeTab(t.id));
				break;
			}
			case 'closeLeft': {
				const idx = tabs.findIndex(t => t.id === id);
				tabs.filter((t, i) => i < idx && !t.pinned).forEach(t => closeTab(t.id));
				break;
			}
			case 'closeAll':
				tabs.filter(t => !t.pinned).forEach(t => closeTab(t.id));
				break;
			case 'pin':
				if (tab) tab.pinned = !tab.pinned;
				break;
			case 'copyPath':
				if (tab) navigator.clipboard.writeText(tab.path).catch(() => {});
				break;
			case 'copyName':
				if (tab) navigator.clipboard.writeText(tab.name).catch(() => {});
				break;
		}
		tabCtxMenu = null;
	}
	let searchMode = $state(false);
	let sidebarMode = $state<'tree' | 'list' | 'skyview'>('tree');
	// CE Phase 9: Multi-Lens Views
	let availableLenses = $state<any[]>([]);
	let activeLensId = $state('');
	let lensGroups = $state<any[]>([]); // LensGroup[] when a lens is active
	let lensEntries = $derived.by(() => {
		if (!activeLensId || lensGroups.length === 0) return null;
		// Build virtual FileEntry tree from lens groups
		return lensGroups.map((g: any) => ({
			name: g.name,
			path: '',
			is_dir: true,
			children: g.notes.map((n: any) => ({
				name: n.name + '.md',
				path: n.path,
				is_dir: false,
				children: null,
				extension: 'md',
				modified: null,
				status: null,
			})),
			extension: null,
			modified: null,
			status: null,
		}));
	});
	let preTreeWidth = 240; // Saved sidebar width before wider modes expanded it

	/** Measure the pixel width needed to display the longest cUniverse/library name */
	function calcContentWidth(extraPadding: number = 80): number {
		const allNames = [...childUniverses.map(c => c.name), ...$libraryStats.map(l => l.name)];
		if (allNames.length === 0) return 240;
		const canvas = document.createElement('canvas');
		const ctx = canvas.getContext('2d');
		if (!ctx) return 280;
		const font = $appSettings?.interfaceFont || 'system-ui, sans-serif';
		ctx.font = `13px ${font}`;
		const maxTextWidth = Math.max(...allNames.map(n => ctx.measureText(n).width));
		// Add padding for: tree indent (~40px), icons (~20px), count badge (~50px), scrollbar (~16px), extra
		return Math.min(Math.max(Math.ceil(maxTextWidth) + extraPadding, 200), 500);
	}
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
	let rightSidebarTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks' | 'calendar' | 'health' | 'provenance' | 'review'>('properties');
	let dueNotes = $state<any[]>([]); // CE Phase 7: ReviewPulse due notes
	let activeTrail = $state<any>(null); // CE Phase 8: active trail data
	let showExpressionForge = $state(false); // CE Phase 10
	let showSenseMakingCanvas = $state(false); // CE Phase 11
	// let inspector360Data = $state<any>(null); // CE Phase 12: disabled — revisit later
	let trailIndex = $state(0); // CE Phase 8: current note index in trail
	let tensionReport = $state<any>(null); // CE Phase 4: TensionReport
	let provenanceChain = $state<any>(null); // CE Phase 5: ProvenanceChain
	let _lastProvenancePath = ''; // cache guard — only re-fetch when note changes

	// Sidebar resizing
	let leftSidebarWidth = $state(300);
	let rightSidebarWidth = $state(300);
	let resizing = $state<'left' | 'right' | null>(null);
	let splitPaneSizes = $state<number[]>([]); // flex values per pane in split view

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
	// showOrgChart removed — now sidebarMode === 'skyview'
	// Shared selection path — when an item is clicked in any sidebar mode, OrgChart highlights it
	let skyViewSelectedPath = $state<string | string[] | null>(null);

	// WiW (Window in Window) overlay state
	let showWiW = $state(false);
	let wiwX = $state(0); // will be set on first show
	let wiwY = $state(0);
	let wiwW = $state(420);
	let wiwH = $state(320);
	let wiwInitialized = $state(false);
	let wiwEnabled = $state(true);

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
					name: $activeTab.name,
					libraryName: $activeTab.libraryName,
					libraryPath: $activeTab.libraryPath ?? '',
					libraryColor: $activeTab.libraryColor ?? '#7c3aed',
				});
			}
		}
	});
	// Also sync when the user switches tabs in editor mode
	$effect(() => {
		if (secondScreenOpen && !showStarView && $activeTab?.path) {
			sendNoteToScreen({
				path: $activeTab.path,
				name: $activeTab.name,
				libraryName: $activeTab.libraryName,
				libraryPath: $activeTab.libraryPath ?? '',
				libraryColor: $activeTab.libraryColor ?? '#7c3aed',
			});
		}
	});
	// Track recently opened notes in localStorage for the second screen dashboard
	$effect(() => {
		const tab = $activeTab;
		if (!tab?.path) return;
		try {
			const key = 'constellation-recent-opened';
			const existing: { name: string; path: string; libraryName: string; openedAt: number }[] = JSON.parse(localStorage.getItem(key) || '[]');
			const filtered = existing.filter(n => n.path !== tab.path);
			filtered.unshift({ name: tab.name, path: tab.path, libraryName: tab.libraryName, openedAt: Date.now() });
			localStorage.setItem(key, JSON.stringify(filtered.slice(0, 20)));
		} catch {}
	});
	// Clipboard monitoring: send copy events to second screen
	let lastSavedContent = $state('');
	$effect(() => {
		if (!secondScreenOpen || showStarView) return;
		// Emit editor context mode when not in Star View
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
	let maturityMap = $state(new Map<string, string>()); // path → maturity state (CE Phase 3)
	let stageMap = $state(new Map<string, string>()); // path → stage (CE Phase 6)
	// Star data is passed to StarView as plain arrays.
	// We avoid $state/$derived for large arrays (1885+ nodes) because Svelte 5 proxies
	// make iteration extremely slow. Instead, starVersion ($state) triggers re-render
	// and StarView reads the plain starNodes/starLinks directly.

	// WiW filtered data — recomputed when selection or star data changes
	// Uses starVersion as reactive trigger since starNodes/starLinks are plain arrays
	const wiwFilteredNodes = $derived.by(() => {
		const _ver = starVersion; // reactive trigger
		if (!skyViewSelectedPath || !showStarView) return [];
		const paths = Array.isArray(skyViewSelectedPath) ? skyViewSelectedPath : [skyViewSelectedPath];
		const norms = paths.map(p => p.replace(/\\/g, '/').toLowerCase());
		return starNodes.filter(n => {
			const np = n.path.replace(/\\/g, '/').toLowerCase();
			return norms.some(norm => np.startsWith(norm + '/') || np === norm);
		});
	});
	const wiwFilteredNodeIds = $derived(new Set(wiwFilteredNodes.map(n => n.id)));
	const wiwFilteredLinks = $derived.by(() => {
		const _ver = starVersion;
		if (wiwFilteredNodes.length === 0) return [];
		return starLinks.filter(l => wiwFilteredNodeIds.has(l.source) && wiwFilteredNodeIds.has(l.target));
	});

	// WiW legend — only libraries present in filtered nodes
	const wiwLibraryColorMap = $derived.by(() => {
		const libs = new Set(wiwFilteredNodes.map(n => n.libraryName));
		const map: Record<string, string> = {};
		for (const lib of libs) {
			if (libraryColorMap[lib]) map[lib] = libraryColorMap[lib];
		}
		return map;
	});

	// WiW title derived from selection
	const wiwTitle = $derived.by(() => {
		const sel = skyViewSelectedPath;
		if (!sel) return '';
		if (Array.isArray(sel)) {
			// cUniverse — find its name from childUniverses
			const cu = childUniverses.find(c => {
				const libs = getChildUniverseLibs(c.path);
				return libs.some(l => sel.includes(l.path));
			});
			return cu ? cu.name : `${sel.length} libraries`;
		}
		// Single path — find library or folder name
		const lib = $libraryStats.find(l => l.path.replace(/\\/g, '/').toLowerCase() === sel.replace(/\\/g, '/').toLowerCase());
		if (lib) return lib.name;
		// Folder — extract last segment
		const parts = sel.replace(/\\/g, '/').split('/');
		return parts[parts.length - 1] || sel;
	});

	// Auto-show/hide WiW (guarded to avoid redundant writes)
	$effect(() => {
		const shouldShow = showStarView && wiwEnabled && skyViewSelectedPath && wiwFilteredNodes.length > 0;
		if (shouldShow) {
			if (!wiwInitialized) {
				wiwX = Math.max(50, window.innerWidth - wiwW - 30);
				wiwY = Math.max(50, window.innerHeight - wiwH - 30);
				wiwInitialized = true;
			}
			if (!showWiW) showWiW = true;
		} else {
			if (showWiW) showWiW = false;
		}
	});

	// WiW drag handlers
	function startWiWDrag(e: MouseEvent) {
		e.preventDefault();
		const startX = e.clientX, startY = e.clientY;
		const startPipX = wiwX, startPipY = wiwY;
		function onMove(ev: MouseEvent) {
			wiwX = startPipX + (ev.clientX - startX);
			wiwY = startPipY + (ev.clientY - startY);
		}
		function onUp() {
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		}
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	function startWiWResize(e: MouseEvent, edge: string) {
		e.preventDefault(); e.stopPropagation();
		const startX = e.clientX, startY = e.clientY;
		const sW = wiwW, sH = wiwH, sX = wiwX, sY = wiwY;
		function onMove(ev: MouseEvent) {
			const dx = ev.clientX - startX, dy = ev.clientY - startY;
			if (edge.includes('e')) wiwW = Math.max(280, sW + dx);
			if (edge.includes('s')) wiwH = Math.max(200, sH + dy);
			if (edge.includes('w')) { wiwW = Math.max(280, sW - dx); wiwX = sX + (sW - wiwW); }
			if (edge.includes('n')) { wiwH = Math.max(200, sH - dy); wiwY = sY + (sH - wiwH); }
		}
		function onUp() {
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		}
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

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

	function ensureSplitSizes(count: number) {
		if (splitPaneSizes.length !== count) {
			splitPaneSizes = Array(count).fill(1);
		}
	}

	function startSplitResize(dividerIndex: number, e: MouseEvent) {
		e.preventDefault();
		const container = (e.target as HTMLElement).parentElement;
		if (!container) return;
		ensureSplitSizes(get(openTabs).length);
		const isHoriz = $splitActive && $splitDirection === 'horizontal';
		const isRtl = $dir === 'rtl';
		// Get the two pane elements adjacent to this divider
		const panes = container.querySelectorAll('.split-pane-wrap');
		const paneA = panes[dividerIndex] as HTMLElement;
		const paneB = panes[dividerIndex + 1] as HTMLElement;
		if (!paneA || !paneB) return;
		const totalSize = isHoriz
			? paneA.offsetHeight + paneB.offsetHeight
			: paneA.offsetWidth + paneB.offsetWidth;
		const startPos = isHoriz ? e.clientY : e.clientX;
		const startSizeA = isHoriz ? paneA.offsetHeight : paneA.offsetWidth;

		function onMouseMove(ev: MouseEvent) {
			let delta = (isHoriz ? ev.clientY : ev.clientX) - startPos;
			if (!isHoriz && isRtl) delta = -delta;
			const newA = Math.max(100, Math.min(totalSize - 100, startSizeA + delta));
			const newB = totalSize - newA;
			// Update flex ratios for these two panes only
			const sizes = [...splitPaneSizes];
			const totalFlex = sizes[dividerIndex] + sizes[dividerIndex + 1];
			sizes[dividerIndex] = (newA / totalSize) * totalFlex;
			sizes[dividerIndex + 1] = (newB / totalSize) * totalFlex;
			splitPaneSizes = sizes;
		}

		function onMouseUp() {
			document.removeEventListener('mousemove', onMouseMove);
			document.removeEventListener('mouseup', onMouseUp);
		}

		document.addEventListener('mousemove', onMouseMove);
		document.addEventListener('mouseup', onMouseUp);
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
	// Track which libraries belong to which child universe (childUniversePath → Set of normalized library paths)
	let childUniverseLibPaths = $state<Map<string, Set<string>>>(new Map());
	// Track which child universes are expanded in the file explorer
	let expandedChildUniverses = $state<Set<string>>(new Set());

	let error = $state('');
	let adding = $state(false);
	let creatingNew = $state(false);
	let newLibraryName = $state('');

	const isHome = $derived(page.url.pathname === '/');

	// Separate own libraries from child universe libraries
	function isChildUniverseLib(libPath: string): boolean {
		const norm = libPath.replace(/\\/g, '/').toLowerCase();
		for (const paths of childUniverseLibPaths.values()) {
			if (paths.has(norm)) return true;
		}
		return false;
	}

	function getChildUniverseLibs(cuPath: string) {
		const paths = childUniverseLibPaths.get(cuPath);
		if (!paths) return [];
		return $libraryStats.filter(lib => paths.has(lib.path.replace(/\\/g, '/').toLowerCase()));
	}

	const ownLibraries = $derived($libraryStats.filter(lib => !isChildUniverseLib(lib.path) && !lib.is_universe_notes));
	const universeNotesStats = $derived($libraryStats.find(lib => lib.is_universe_notes) ?? null);

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

	// ─── Debounced sidebar computations (BLOCKING-001 fix) ───
	// These are expensive and sidebar-only — they don't need to run synchronously.
	// Debouncing at 500ms prevents them from firing on every keystroke.
	let sidebarHeadings = $state<HeadingItem[]>([]);
	let noteDir = $state<'ltr' | 'rtl'>($dir as 'ltr' | 'rtl');
	let focusMode = $state(false);
	let _focusModeTabId = '';
	$effect(() => { const id = $activeTab?.id ?? ''; if (id !== _focusModeTabId) { _focusModeTabId = id; focusMode = false; } });
	let currentBacklinks = $state<{ name: string; path: string; context: string; libraryName: string; linkType?: string }[]>([]);
	let currentOutgoing = $state<{ target: string; context: string }[]>([]);
	let activeNoteTags = $state<string[]>([]);
	let _sidebarDebounce: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		const body = sidebarBody;
		const tab = sidebarTab;
		const props = sidebarProperties;
		const dirFallback = $dir as 'ltr' | 'rtl';
		clearTimeout(_sidebarDebounce);

		// Immediate reset when no tab
		if (!tab) {
			sidebarHeadings = [];
			noteDir = dirFallback;
			currentBacklinks = [];
			currentOutgoing = [];
			activeNoteTags = [];
			return;
		}

		_sidebarDebounce = setTimeout(() => {
			// Headings
			sidebarHeadings = body ? extractHeadings(body) : [];
			// Direction
			noteDir = body ? detectDir(body) : dirFallback;
			// Backlinks
			currentBacklinks = getBacklinks(allLibraryLinks, tab.name);
			// CE Phase 5: Provenance fetched on tab click only (not here — no IPC on typing path)
			// Outgoing links
			currentOutgoing = getOutgoingLinks(allLibraryLinks, tab.path).map(l => ({
				target: l.target,
				context: l.context,
			}));
			// Tags (from frontmatter + inline)
			const tags: string[] = [];
			for (const p of props) {
				if (p.key === 'tags' || p.key === 'tag') {
					if (Array.isArray(p.value)) {
						tags.push(...p.value.map((v: string) => String(v).trim()).filter(Boolean));
					} else if (typeof p.value === 'string') {
						tags.push(...p.value.split(',').map(v => v.trim()).filter(Boolean));
					}
				}
			}
			const bodyText = body || '';
			const inlineMatches = bodyText.match(/(?:^|\s)#([a-zA-Z\u0600-\u06FF][\w\u0600-\u06FF/\-]*)/g);
			if (inlineMatches) {
				for (const m of inlineMatches) {
					tags.push(m.trim().replace(/^#/, ''));
				}
			}
			activeNoteTags = [...new Set(tags)];
		}, 500);
	});

	// Unlinked mentions for current note (already debounced — no change needed)
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
		_tasksTimer = setTimeout(async () => {
			try {
				const result = await scanNoteTasks(tab.path, tab.libraryName, tab.libraryPath);
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
		_calTimer = setTimeout(async () => {
			try {
				const libraryList = get(libraries);
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
			} catch { /* Calendar scan failed */ }
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

	// Focus — toggle body class to hide sidebar/tabs/statusbar
	$effect(() => {
		if (typeof document === 'undefined') return;
		const atm = $appSettings.focus || 'none';
		document.body.classList.toggle('focus-active', atm !== 'none');
	});

	// Apply custom fonts at runtime
	$effect(() => {
		if (typeof document === 'undefined') return;
		const s = $appSettings;
		// Set font variables on <body> to override .theme-light/.theme-dark class rules
		const root = document.body.style;

		// Apply base font settings to CSS variables
		const defaultUI = '-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, "Noto Sans Arabic", "Noto Sans Hebrew", "Noto Sans CJK SC", sans-serif';
		const defaultMono = '"Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace';

		root.setProperty('--font-text-size', s.fontSize + 'px');
		root.setProperty('--font-ui-size', (s.interfaceFontSize || 14) + 'px');
		document.documentElement.style.fontSize = (s.interfaceFontSize || 14) + 'px';

		const fontMode = s.fontMode || 'per-language';
		const isTypewriter = s.fontTheme === 'typewriter';
		let css = '';

		// Typewriter preset: inject @font-face rules for all scripts
		if (isTypewriter) {
			const twRanges: Record<string, string> = {
				arabic:     SCRIPT_UNICODE_RANGES.arabic,
				hebrew:     SCRIPT_UNICODE_RANGES.hebrew,
				devanagari: SCRIPT_UNICODE_RANGES.devanagari,
				cyrillic:   SCRIPT_UNICODE_RANGES.cyrillic,
				cjk:        SCRIPT_UNICODE_RANGES.cjk,
			};
			for (const [script, range] of Object.entries(twRanges)) {
				const fontStack = TYPEWRITER_FONTS.scriptFonts[script];
				if (fontStack && range) {
					const firstName = fontStack.split(',')[0].trim();
					css += `@font-face { font-family: "ConstellationTypewriter"; src: local(${firstName}); unicode-range: ${range}; }\n`;
				}
			}
			const twStack = `"ConstellationTypewriter", ${TYPEWRITER_FONTS.textFont}`;
			root.setProperty('--font-text-theme', twStack);
			root.setProperty('--font-interface-theme', twStack);
			css += `.cm-editor .cm-content { font-family: ${twStack} !important; }\n`;
			css += `.cm-editor .cm-scroller { font-family: ${twStack}; }\n`;
		}

		if (fontMode === 'universal') {
			// Universal mode: one font set for everything
			const set = getFontSetById(s.activeFontSetId || 'system', s.customFontSets || []);
			const uiFont = set?.interfaceFont || defaultUI;
			const txtFont = set?.textFont || defaultUI;
			const mono = set?.monoFont || defaultMono;
			root.setProperty('--font-interface-theme', uiFont);
			root.setProperty('--font-text-theme', txtFont);
			root.setProperty('--font-monospace-theme', mono);
		} else {
			// Per-language mode: each script gets its own font set via unicode-range
			const langSets = s.languageFontSets || {};
			const customSets = s.customFontSets || [];
			let hasPerScript = false;

			// Determine the base (Latin) font set for defaults
			const latinSet = getFontSetById(langSets.latin || 'system', customSets);
			const baseUI = latinSet?.interfaceFont || defaultUI;
			const baseTxt = latinSet?.textFont || defaultUI;
			const baseMono = latinSet?.monoFont || defaultMono;
			root.setProperty('--font-monospace-theme', baseMono);

			// Generate @font-face rules for non-latin scripts
			for (const [script, range] of Object.entries(SCRIPT_UNICODE_RANGES)) {
				if (script === 'latin') continue;
				const setId = langSets[script];
				if (!setId || setId === 'system') continue;
				const set = getFontSetById(setId, customSets);
				if (!set) continue;

				hasPerScript = true;
				const uiName = (set.interfaceFont || set.name || '').split(',')[0].trim().replace(/"/g, '');
				const txtName = (set.textFont || set.name || '').split(',')[0].trim().replace(/"/g, '');
				if (uiName) {
					css += `@font-face { font-family: "ConstellationUI"; src: local("${uiName}"); unicode-range: ${range}; }\n`;
				}
				if (txtName) {
					css += `@font-face { font-family: "ConstellationText"; src: local("${txtName}"); unicode-range: ${range}; }\n`;
				}
			}

			// Also support legacy scriptFonts for backward compatibility
			const legacyScripts = s.scriptFonts || {};
			const legacyRanges: Record<string, string> = {
				arabic: SCRIPT_UNICODE_RANGES.arabic,
				hebrew: SCRIPT_UNICODE_RANGES.hebrew,
				cjk: SCRIPT_UNICODE_RANGES.cjk,
			};
			for (const [script, range] of Object.entries(legacyRanges)) {
				const fontName = legacyScripts[script];
				if (fontName && !langSets[script]) {
					hasPerScript = true;
					css += `@font-face { font-family: "ConstellationUI"; src: local("${fontName}"); unicode-range: ${range}; }\n`;
					css += `@font-face { font-family: "ConstellationText"; src: local("${fontName}"); unicode-range: ${range}; }\n`;
				}
			}

			if (hasPerScript) {
				const uiStack = `"ConstellationUI", ${baseUI}`;
				const txtStack = `"ConstellationText", ${baseTxt}`;
				root.setProperty('--font-interface-theme', uiStack);
				root.setProperty('--font-text-theme', txtStack);
				// Directly target CM6 editor elements — CSS variables don't cascade
				// into CodeMirror's scoped styles for @font-face virtual fonts
				css += `.cm-editor .cm-content { font-family: ${txtStack} !important; }\n`;
				css += `.cm-editor .cm-scroller { font-family: ${txtStack}; }\n`;
			} else {
				root.setProperty('--font-interface-theme', baseUI);
				root.setProperty('--font-text-theme', baseTxt);
				css += `.cm-editor .cm-content { font-family: ${baseTxt} !important; }\n`;
				css += `.cm-editor .cm-scroller { font-family: ${baseTxt}; }\n`;
			}
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
			{ id: 'star-view', name: $t('commands.starView'), shortcut: sc('star-view'), icon: '🕸️', action: () => { showStarView = !showStarView; }, category: 'View' },
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
			{ id: 'review-pulse', name: $t('commands.reviewDueNotes') || 'Review due notes', icon: '📋', action: () => { showCommandPalette = false; rightSidebarOpen = true; rightSidebarTab = 'review'; const lib = get(libraries)[0]; if (lib) invoke<any[]>('get_due_notes', { libraryPath: lib.path }).then(notes => { dueNotes = notes; }).catch(() => {}); }, category: 'View' },
			{ id: 'open-trail', name: $t('commands.openTrail') || 'Open Trail', icon: '🛤️', action: async () => {
				showCommandPalette = false;
				const lib = get(libraries)[0];
				if (!lib) return;
				try {
					const trails = await invoke<any[]>('list_trails', { libraryPath: lib.path });
					if (trails.length === 0) return;
					// For now, open first trail found (TODO: trail picker)
					const trail = await invoke<any>('read_trail', { trailPath: trails[0].trail_path, libraryPath: lib.path });
					activeTrail = trail;
					trailIndex = 0;
					if (trail.notes.length > 0 && trail.notes[0].exists) {
						await openNoteTab(trail.notes[0].path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
					}
				} catch {}
			}, category: 'Navigation' },
			{ id: 'create-lens', name: $t('commands.createLens') || 'Create Lens', icon: '🔍', action: () => { showCommandPalette = false; showSettings = true; }, category: 'View' },
			{ id: 'expression-forge', name: $t('commands.expressionForge') || 'Expression Forge', icon: '✨', action: () => { showCommandPalette = false; showExpressionForge = !showExpressionForge; showStarView = false; showGlobalTasks = false; showIndex = false; showSenseMakingCanvas = false; }, category: 'View' },

			{ id: 'sense-making-canvas', name: $t('commands.senseMakingCanvas') || 'Sense-Making Canvas', icon: '🎨', action: () => { showCommandPalette = false; showSenseMakingCanvas = !showSenseMakingCanvas; showStarView = false; showGlobalTasks = false; showIndex = false; showExpressionForge = false; }, category: 'View' },
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
			getChildUniverses().then(async (c) => {
				childUniverses = c;
				// Resolve which libraries belong to each child universe
				const map = new Map<string, Set<string>>();
				for (const cu of c) {
					try {
						const childLibs = await invoke<{ id: string; name: string; path: string }[]>(
							'read_child_universe_libraries', { childPath: cu.path }
						);
						map.set(cu.path, new Set(childLibs.map(l => l.path.replace(/\\/g, '/').toLowerCase())));
					} catch {
						map.set(cu.path, new Set());
					}
				}
				childUniverseLibPaths = map;
			}).catch(() => {}),
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

		// Activate the last-active universe — try each entry until one succeeds.
		// If a universe was moved/deleted, skip it and try the next.
		let activated = false;
		for (const entry of universes) {
			try {
				await setActiveUniverse(entry.id);
				activeUniverseName = entry.name;
				activated = true;
				break;
			} catch {
				// This universe's path doesn't exist — try next
				continue;
			}
		}
		if (!activated) {
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
			await openNoteTab(note.path, note.libraryName, note.libraryColor);
		});
		const unlistenScreenClosed = await onScreenClosed(() => {
			secondScreenOpen = false;
		});

		// Global keyboard shortcuts — capture phase to beat browser defaults
		document.addEventListener('keydown', handleGlobalKeydown, true);

		// Cleanup on destroy
		cleanupFns.push(
			() => document.removeEventListener('keydown', handleGlobalKeydown, true),
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
		if (skyviewHoverTimer) clearTimeout(skyviewHoverTimer);
		if (idleTimer) clearTimeout(idleTimer);
		clearTimeout(_sidebarDebounce);
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

				// CE Phase 2: Fetch Knowledge Strata per library, merge into starNodes
				for (const lib of libraryList) {
					try {
						const strata = await invoke<{ note_path: string; stratum: number }[]>(
							'compute_note_strata', { libraryPath: lib.path, libraryName: lib.name }
						);
						const strataMap = new Map(strata.map(s => [s.note_path.replace(/\\/g, '/').toLowerCase(), s.stratum]));
						for (const node of starNodes) {
							const key = node.path.replace(/\\/g, '/').toLowerCase();
							const s = strataMap.get(key);
							if (s !== undefined) node.stratum = s;
						}
					} catch { /* strata computation failed — nodes stay without stratum */ }
				}
				// CE Phase 3: Fetch Maturity Lifecycle per library, merge into starNodes + maturityMap
				const newMatMap = new Map<string, string>();
				for (const lib of libraryList) {
					try {
						const maturities = await invoke<{ note_path: string; state: string }[]>(
							'compute_note_maturity', { libraryPath: lib.path, libraryName: lib.name }
						);
						for (const m of maturities) {
							const key = m.note_path.replace(/\\/g, '/').toLowerCase();
							newMatMap.set(key, m.state);
						}
						for (const node of starNodes) {
							const key = node.path.replace(/\\/g, '/').toLowerCase();
							const m = newMatMap.get(key);
							if (m) node.maturity = m;
						}
					} catch { /* maturity computation failed */ }
				}
				maturityMap = newMatMap;

				// CE Phase 5: Fetch provenance origins per library, merge into starNodes
				for (const lib of libraryList) {
					try {
						const origins = await invoke<{ note_path: string; origin_type: string }[]>(
							'compute_note_origins', { libraryPath: lib.path, libraryName: lib.name }
						);
						const originMap = new Map(origins.map(o => [o.note_path.replace(/\\/g, '/').toLowerCase(), o.origin_type]));
						for (const node of starNodes) {
							const key = node.path.replace(/\\/g, '/').toLowerCase();
							const o = originMap.get(key);
							if (o && o !== 'none') node.originType = o;
						}
					} catch { /* origins failed */ }
				}

				// CE Phase 6: Scan note stages per library
				const newStageMap = new Map<string, string>();
				for (const lib of libraryList) {
					try {
						const stages = await invoke<[string, string][]>('scan_note_stages', { libraryPath: lib.path });
						for (const [path, stage] of stages) {
							newStageMap.set(path.replace(/\\/g, '/').toLowerCase(), stage);
						}
					} catch { /* stages scan failed */ }
				}
				stageMap = newStageMap;

				// CE Phase 9: Load available lenses
				try {
					availableLenses = await invoke('list_lenses');
				} catch { availableLenses = []; }

				// CE Phase 4: Detect tensions (first library only for performance)
				if (libraryList.length > 0) {
					try {
						tensionReport = await invoke('detect_tensions', { libraryPath: libraryList[0].path, libraryName: libraryList[0].name });
					} catch { tensionReport = null; }
				}
				starVersion++; // signal strata + maturity + tension data ready
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

		// Block browser defaults for known shortcuts (e.g., Ctrl+P = print → command palette)
		if ((e.ctrlKey || e.metaKey) && e.key === 'p' && !e.shiftKey && !e.altKey) {
			e.preventDefault();
		}

		// Escape always closes overlays (not remappable)
		if (e.key === 'Escape') {
			if (showCommandPalette) { showCommandPalette = false; return; }
			if (showQuickSwitcher) { showQuickSwitcher = false; return; }
			if (showStarView) { showStarView = false; return; }
			if (sidebarMode === 'skyview') { sidebarMode = 'tree'; return; }
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

	async function handleStarNodeClick(path: string, libraryName: string) {
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

		// No second screen — open note in main, THEN exit Sky View so the tab
		// has a path before the new-tab guard renders an empty screen.
		await openNoteTab(path, libraryName, color);
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
		skyViewSelectedPath = lib.path;
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
		skyViewSelectedPath = filePath;
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
			<button class="dock-btn" onclick={() => { sidebarOpen = true; searchMode = true; }} title={$t('ribbon.search')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<button class="dock-btn" class:active={showGlobalTasks} onclick={() => { showGlobalTasks = !showGlobalTasks; showStarView = false; showIndex = false; }} title={$t('ribbon.globalTasks')}>
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
			<!-- Universe manager moved to sidebar -->
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
				{/if}
				<!-- Row 1: New Elements — always visible -->
				<div class="toolbar-actions new-elements">
					<button class="tb-btn" onclick={handleNewNote} title={$t('sidebar.newNote')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M12 18v-6"/><path d="M9 15h6"/></svg>
					</button>
					<button class="tb-btn" onclick={handleNewBase} title={$t('sidebar.newBase')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
					</button>
					<button class="tb-btn" onclick={handleNewFolder} title={$t('sidebar.newFolder')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 10v6"/><path d="M9 13h6"/><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>
					</button>
				</div>
				<!-- Row 2: Notes Management — always visible, even during search -->
				<div class="toolbar-modes notes-management">
					<button class="mode-tab" class:active={sidebarMode === 'tree'} onclick={() => { searchMode = false; searchQuery = ''; if (sidebarMode !== 'tree') { sidebarMode = 'tree'; leftSidebarWidth = calcContentWidth(100); emitSidebarModeChanged('tree'); } }} title={$t('navigator.fileExplorer') || 'File Explorer'}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
					</button>
					<button class="mode-tab" class:active={sidebarMode === 'list'} onclick={() => { searchMode = false; searchQuery = ''; if (sidebarMode !== 'list') { if (sidebarMode === 'tree') preTreeWidth = leftSidebarWidth; sidebarMode = 'list'; leftSidebarWidth = Math.max(leftSidebarWidth, 450); emitSidebarModeChanged('list'); } }} title={$t('navigator.notesNavigator') || 'Notes Navigator'}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="18" rx="1"/><rect x="14" y="3" width="7" height="18" rx="1"/></svg>
					</button>
					<button class="mode-tab" class:active={sidebarMode === 'skyview'} onclick={() => { searchMode = false; searchQuery = ''; if (sidebarMode !== 'skyview') { if (sidebarMode === 'tree') preTreeWidth = leftSidebarWidth; sidebarMode = 'skyview'; leftSidebarWidth = calcContentWidth(130); emitSidebarModeChanged('skyview'); } }} title={$t('navigator.orgChart') || 'Organization Chart'}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="8" y="2" width="8" height="5" rx="1"/><rect x="1" y="17" width="8" height="5" rx="1"/><rect x="15" y="17" width="8" height="5" rx="1"/><path d="M12 7v4"/><path d="M5 17v-2h14v2"/></svg>
					</button>
					<button class="mode-tab" class:active={showStarView} onclick={() => { showStarView = !showStarView; showGlobalTasks = false; showIndex = false; }} title={$t('ribbon.graphView')}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>
					</button>
					{#if sidebarMode === 'tree' && !searchMode}
						<button class="mode-tab" onclick={cycleSortOrder} title={getSortTooltip()}>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/><path d="m21 16-4 4-4-4"/><path d="M17 20V4"/></svg>
						</button>
						<button class="mode-tab" onclick={toggleCollapseAll} title={expandedLibraries.size > 0 ? $t('sidebar.collapseAll') : $t('sidebar.expandAll')}>
							{#if expandedLibraries.size > 0}
								<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m7 20 5-5 5 5"/><path d="m7 4 5 5 5-5"/></svg>
							{:else}
								<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m7 15 5 5 5-5"/><path d="m7 9 5-5 5 5"/></svg>
							{/if}
						</button>
					{/if}
						<!-- CE Phase 9: Lens switcher -->
						{#if sidebarMode === 'tree' && !searchMode}
							<select class="mode-tab lens-select" value={activeLensId}
								onchange={async (e) => {
									const id = (e.target as HTMLSelectElement).value;
									activeLensId = id;
									if (!id) { lensGroups = []; return; }
									const lens = availableLenses.find((l: any) => l.id === id);
									const lib = get(libraries)[0];
									if (lens && lib) {
										try {
											lensGroups = await invoke('apply_lens', { libraryPath: lib.path, lens });
										} catch { lensGroups = []; }
									}
								}}>
								<option value="">📁 {$t('lensPanel.default') || 'Folders'}</option>
								{#each availableLenses as lens}
									<option value={lens.id}>🔍 {lens.name}</option>
								{/each}
							</select>
						{/if}
					</div>
				</div>

			<div class="sidebar-content">
				{#if sidebarMode === 'list'}
					<NotebookNavigator
						mode="main"
						{libraryColorMap}
						onNoteClick={(path, name, lib) => handleNoteClick(path, name, undefined)}
						onFolderSelect={(path) => { skyViewSelectedPath = path; }}
					/>
				{:else if sidebarMode === 'skyview'}
					<OrgChart
						{libraryColorMap}
						universeName={activeUniverseName}
						bind:selectedPath={skyViewSelectedPath}
						onNoteClick={(path, name) => handleNoteClick(path, name)}
						onClose={() => sidebarMode = 'tree'}
						embedded={true}
					/>
				{:else if searchMode && searchQuery}
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
				{:else if activeLensId && lensEntries}
					<!-- CE Phase 9: Lens view -->
					<div class="section-label">🔍 {availableLenses.find((l: any) => l.id === activeLensId)?.name ?? 'Lens'}</div>
					<FileTree
						entries={lensEntries}
						libraryName={get(libraries)[0]?.name ?? ''}
						color={libraryColorMap[get(libraries)[0]?.name ?? ''] ?? '#7c3aed'}
						onNoteClick={(path, name, ht, e) => handleNoteClick(path, name, ht, e)}
						{maturityMap}
						{stageMap}
					/>
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

					<!-- Universe Notes — folder named after the universe, shown above everything -->
					{#if universeNotesStats}
						<div class="library-section">
							<button class="library-header universe-notes-item" onclick={() => toggleLibrary(universeNotesStats)}>
								<svg class="v-chev" class:expanded={expandedLibraries.has(universeNotesStats.library_id)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--interactive-accent)" stroke-width="1.5" style="flex-shrink: 0;">
									<circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/>
									<path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/>
									<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2"/>
								</svg>
								<span class="library-name">{universeNotesStats.name}</span>
								{#if universeNotesStats.star_count > 0}
									<span class="child-universe-count">{universeNotesStats.star_count}</span>
								{/if}
							</button>
							{#if expandedLibraries.has(universeNotesStats.library_id) && libraryTrees[universeNotesStats.library_id]}
								<div class="library-tree">
									<FileTree
									entries={sortEntries(libraryTrees[universeNotesStats.library_id])}
									libraryId={universeNotesStats.library_id}
									libraryName={universeNotesStats.name}
									color={libraryColorMap[universeNotesStats.name] || 'var(--interactive-accent)'}
									{maturityMap}
									{stageMap}
									onNoteClick={handleNoteClick}
									onFolderClick={(path) => { skyViewSelectedPath = path; }}
									onContextMenu={(entry, x, y) => handleContextMenu(entry, x, y, universeNotesStats.library_id)}
									{renamingPath}
									onRenameComplete={handleRenameComplete}
									{allExpanded}
								/>
								</div>
							{/if}
						</div>
					{/if}

					<!-- Child Universes — expandable, with their libraries nested inside -->
					{#each childUniverses as child}
						<div class="library-section">
							<button class="library-header child-universe-item" onclick={() => {
								// Pass all child library paths for Star View highlighting
								const libPaths = getChildUniverseLibs(child.path).map(l => l.path);
								skyViewSelectedPath = libPaths.length > 0 ? libPaths : child.path;
								const next = new Set(expandedChildUniverses);
								if (next.has(child.path)) next.delete(child.path); else next.add(child.path);
								expandedChildUniverses = next;
							}}>
								<svg class="v-chev" class:expanded={expandedChildUniverses.has(child.path)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#6366f1" stroke-width="1.5" style="flex-shrink: 0;">
									<circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/>
									<path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/>
									<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2"/>
								</svg>
								<span class="library-name">{child.name}</span>
								<span class="child-universe-count">{child.library_count}</span>
							</button>

							{#if expandedChildUniverses.has(child.path)}
								<div class="child-universe-libs">
									{#each getChildUniverseLibs(child.path) as lib}
										<div class="library-section library-section-nested">
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
													{maturityMap}
									{stageMap}
													onNoteClick={handleNoteClick}
													onFolderClick={(path) => { skyViewSelectedPath = path; }}
													onContextMenu={(entry, x, y) => handleContextMenu(entry, x, y, lib.library_id)}
													{renamingPath}
													onRenameComplete={handleRenameComplete}
													{allExpanded}
												/>
												</div>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/each}

					<!-- Own Libraries — only libraries NOT belonging to a child universe -->
					{#each ownLibraries as lib}
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
									{maturityMap}
									{stageMap}
									onNoteClick={handleNoteClick}
									onFolderClick={(path) => { skyViewSelectedPath = path; }}
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
		<!-- Tab Bar (unified with layout controls) -->
		<!-- Layout bar: sidebar + split controls (independent from tabs/paper) -->
		<div class="layout-bar">
			<button class="tab-action" class:active={sidebarOpen} onclick={() => sidebarOpen = !sidebarOpen} title={$t('layout.leftSidebar')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/></svg>
			</button>
			<div style="flex:1"></div>
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

		<!-- Tab bar (locked to paper, hidden when full-screen overlay is active) -->
		<div class="tab-bar" class:tab-bar-hidden={showStarView || showGlobalTasks || showIndex || showExpressionForge || showSenseMakingCanvas}>
			{#if !$splitActive}
				<div class="tab-scroll-wrap">
				{#if canScrollStart}
					<button class="tab-scroll-arrow tab-scroll-start" onclick={() => scrollTabs('start')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
					</button>
				{/if}
				<div class="tab-scroll" class:no-tabs={$openTabs.length === 0} bind:this={tabScrollEl} onscroll={updateTabScrollArrows}>
					{#each $openTabs as tab (tab.id)}
						<button class="tab"
							class:active={$activeTabId === tab.id}
							class:pinned={tab.pinned}
							class:drag-over={dragOverTabId === tab.id}
							style:--library-color={libraryColorMap[tab.libraryName]}
							data-tab-id={tab.id}
							onmousedown={(e) => startTabDrag(e, tab.id)}
							onclick={() => { if (!isDragging) switchTab(tab.id); }}
							onauxclick={(e) => { if (e.button === 1 && !tab.pinned) { e.preventDefault(); closeTab(tab.id); } }}
							oncontextmenu={(e) => showTabContextMenu(e, tab.id)}>
							{#if tab.pinned}
								<span class="tab-pin" title={$t('layout.pinned')}>📌</span>
							{:else if tab.libraryName}
								<span class="tab-lib-name">{tab.libraryName}</span>
							{/if}
							{#if (() => { const m = maturityMap.get(tab.path?.replace(/\\/g, '/').toLowerCase() ?? ''); return m && m !== 'seed'; })()}<span class="tab-maturity" class:mat-sapling={maturityMap.get(tab.path?.replace(/\\/g, '/').toLowerCase() ?? '') === 'sapling'} class:mat-evergreen={maturityMap.get(tab.path?.replace(/\\/g, '/').toLowerCase() ?? '') === 'evergreen'} class:mat-canonical={maturityMap.get(tab.path?.replace(/\\/g, '/').toLowerCase() ?? '') === 'canonical'} class:mat-wilting={maturityMap.get(tab.path?.replace(/\\/g, '/').toLowerCase() ?? '') === 'wilting'}>●</span>{/if}
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
					<div class="tab-new-wrap">
					<button class="tab tab-new" onclick={() => createEmptyTab()} title="New tab">
						<svg class="tab-plus-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
						<svg class="tab-bulb-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18h6"/><path d="M10 22h4"/><path d="M12 2a7 7 0 0 0-4 12.7V17h8v-2.3A7 7 0 0 0 12 2z"/></svg>
					</button>
					{#if $openTabs.length === 0}
						<span class="tab-new-hint">{$t('layout.clickToStart')}</span>
					{/if}
				</div>
				</div>
				{#if canScrollEnd}
					<button class="tab-scroll-arrow tab-scroll-end" onclick={() => scrollTabs('end')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
					</button>
				{/if}
				</div>

				<!-- Tab context menu -->
				{#if tabCtxMenu}
					{#each [$openTabs.find(t => t.id === tabCtxMenu.tabId)] as ctxTab}
						<div class="tab-ctx-menu" style="left:{tabCtxMenu.x}px;top:{tabCtxMenu.y}px">
							<button class="tab-ctx-item" onclick={() => tabCtxAction('pin')}>
								{ctxTab?.pinned ? 'Unpin' : 'Pin'}
							</button>
							<div class="tab-ctx-sep"></div>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('close')} disabled={ctxTab?.pinned}>Close</button>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('closeOthers')}>Close Others</button>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('closeRight')}>Close to the Right</button>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('closeLeft')}>Close to the Left</button>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('closeAll')}>Close All</button>
							<div class="tab-ctx-sep"></div>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('copyPath')}>Copy Path</button>
							<button class="tab-ctx-item" onclick={() => tabCtxAction('copyName')}>Copy Name</button>
						</div>
					{/each}
				{/if}
			{/if}
		</div>

		<!-- Content -->
		<div class="content-area" onmouseover={handleWikilinkHover} onmouseout={handleWikilinkLeave}>
			{#if showStarView}
				<div class="star-fullscreen">
					<div class="star-header">
						<span class="star-title">{$t('layout.starViewTitle')}</span>
						<button class="star-wiw-toggle" class:active={wiwEnabled} onclick={() => { wiwEnabled = !wiwEnabled; if (!wiwEnabled) showWiW = false; }} title="Window in Window">
							<svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
								<rect x="0.75" y="1.75" width="12.5" height="9" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
								<rect x="7" y="5" width="5" height="3.5" rx="0.75" fill="currentColor"/>
							</svg>
						</button>
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
					highlightPath={skyViewSelectedPath}
					highlightColor={(() => {
						if (!skyViewSelectedPath) return 0x7c3aed;
						const firstPath = Array.isArray(skyViewSelectedPath) ? skyViewSelectedPath[0] : skyViewSelectedPath;
						if (!firstPath) return 0x7c3aed;
						const lib = $libraryStats.find(l => {
							const norm = firstPath.replace(/\\/g, '/').toLowerCase();
							return l.path.replace(/\\/g, '/').toLowerCase() === norm || norm.startsWith(l.path.replace(/\\/g, '/').toLowerCase() + '/');
						});
						if (lib) {
							const hex = libraryColorMap[lib.name] || '#7c3aed';
							return parseInt(hex.replace('#', ''), 16);
						}
						return 0x7c3aed;
					})()}
					skyViewSettings={$appSettings.skyView}
					{libraryColorMap}
				/>
				<!-- WiW Overlay -->
				{#if showWiW && wiwFilteredNodes.length > 0}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="wiw-overlay" style="left:{wiwX}px; top:{wiwY}px; width:{wiwW}px; height:{wiwH}px;">
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-header" onmousedown={startWiWDrag}>
							<div class="wiw-header-row">
								<span class="wiw-title" dir="auto">{wiwTitle}</span>
								<span class="wiw-count">{wiwFilteredNodes.length} nodes</span>
								<button class="wiw-close" onclick={() => { showWiW = false; skyViewSelectedPath = null; }}>×</button>
							</div>
							<span class="wiw-subtitle">{$t('layout.starViewWiWHint')}</span>
						</div>
						<div class="wiw-graph">
							<GraphMindView
								nodes={wiwFilteredNodes}
								links={wiwFilteredLinks}
								libraryColorMap={wiwLibraryColorMap}
								onNodeClick={handleStarNodeClick}
							/>
						</div>
						<!-- Resize handles -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-n" onmousedown={(e) => startWiWResize(e, 'n')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-s" onmousedown={(e) => startWiWResize(e, 's')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-e" onmousedown={(e) => startWiWResize(e, 'e')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-w" onmousedown={(e) => startWiWResize(e, 'w')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-se" onmousedown={(e) => startWiWResize(e, 'se')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-sw" onmousedown={(e) => startWiWResize(e, 'sw')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-ne" onmousedown={(e) => startWiWResize(e, 'ne')}></div>
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="wiw-resize wiw-resize-nw" onmousedown={(e) => startWiWResize(e, 'nw')}></div>
					</div>
				{/if}
				</div>
			{:else if showGlobalTasks}
				<GlobalTasksView
					{libraryColorMap}
					onClose={() => showGlobalTasks = false}
				/>
			{:else if showSenseMakingCanvas}
				<SenseMakingCanvas
					libraryPath={get(libraries)[0]?.path ?? ''}
					libraryName={get(libraries)[0]?.name ?? ''}
					libraryColor={libraryColorMap[get(libraries)[0]?.name ?? ''] ?? '#7c3aed'}
					onClose={() => showSenseMakingCanvas = false}
				/>
			{:else if showExpressionForge}
				<ExpressionForge
					notes={starNodes}
					{activeTrail}
					libraryPath={get(libraries)[0]?.path ?? ''}
					libraryName={get(libraries)[0]?.name ?? ''}
					onClose={() => showExpressionForge = false}
				/>
			{:else if showIndex}
				<div class="index-split" class:has-note={indexNoteTab}>
					{#if indexNoteTab}
						{@const _ip = parseFrontmatter(indexNoteTab.content || '')}
						{@const _ibody = _ip.body}
						{@const _idir = detectDir(_ibody) || $dir}
						{@const _iGuard = { saving: false }}
						<div class="index-note-pane">
							<div class="index-note-header">
								<span class="index-note-name" dir="auto">{indexNoteTab.name}</span>
								<button class="index-close" onclick={() => { indexNoteTab = null; indexActiveNotePath = ''; }} title="Close note">×</button>
							</div>
							{#key indexNoteTab.id + '|' + indexNoteTab.path}
							<NotePane
								value={_ibody}
								title={indexNoteTab.name.replace(/\.md$/, '')}
								dir={_idir}
								initialCursorPos={indexNoteTab.cursorPos ?? 0}
								initialScrollTop={indexNoteTab.scrollTop ?? 0}
								libraryName={indexNoteTab.libraryName}
								tabId={indexNoteTab.id}
								filePath={indexNoteTab.path}
								libraryPath={indexNoteTab.libraryPath || ''}
								noteNames={allNotes}
								allTags={allTagsList}
								properties={_ip.properties}
								rawYaml={_ip.rawYaml ?? ''}
								stage={_ip.properties.find(p => p.key.toLowerCase() === 'stage')?.value ?? ''}
								onchange={() => {}}
								onpromote={(nextStage) => {
									const pr = parseFrontmatter(indexNoteTab?.content || '').properties;
									const bd = parseFrontmatter(indexNoteTab?.content || '').body;
									let np;
									if (!nextStage) { np = pr.filter(p => p.key.toLowerCase() !== 'stage'); }
									else {
										let u = false;
										np = pr.map(p => { if (p.key.toLowerCase() === 'stage') { u = true; return { ...p, value: nextStage }; } return p; });
										if (!u) np.push({ key: 'stage', value: nextStage, type: 'text' as any });
									}
									const fc = buildFullContent(np, bd);
									if (indexNoteTab) indexNoteTab.content = fc;
									markRecentWrite(indexNoteTab!.path);
									writeNote(indexNoteTab!.path, fc).catch(() => {});
								}}
								onsave={(text) => {
									if (_iGuard.saving) return;
									_iGuard.saving = true;
									const pr = parseFrontmatter(indexNoteTab?.content || '').properties;
									markRecentWrite(indexNoteTab!.path);
									const content = buildFullContent(pr, text);
									writeNote(indexNoteTab!.path, content).catch(() => {}).finally(() => { _iGuard.saving = false; });
								}}
								onflush={(text, needsDiskSave, cursorPos, scrollTop) => {
									const pr = parseFrontmatter(indexNoteTab?.content || '').properties;
									const content = buildFullContent(pr, text);
									if (indexNoteTab) { indexNoteTab.content = content; indexNoteTab.cursorPos = cursorPos; indexNoteTab.scrollTop = scrollTop; }
									if (needsDiskSave) {
										markRecentWrite(indexNoteTab!.path);
										writeNote(indexNoteTab!.path, content).catch(() => {});
									}
								}}
								ontitlechange={(newTitle) => {
									if (indexNoteTab && newTitle !== indexNoteTab.name.replace(/\.md$/, '')) {
										renameItem(indexNoteTab.path, indexNoteTab.path.replace(/[^/\\]+$/, newTitle + '.md'));
									}
								}}
								onpropschange={() => {}}
							/>
							{/key}
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
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div class="pane-divider" onmousedown={(e) => startSplitResize(i - 1, e)}></div>
							{/if}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div class="split-pane-wrap" style="flex:{splitPaneSizes[i] ?? 1}" onclick={() => setFocusedTab(tab.id)}>
							{#if tab.path}
								{@const _sp = parseFrontmatter(tab.content || '')}
								{@const _sbody = _sp.body}
								{@const _sdir = detectDir(_sbody) || $dir}
								{@const _sGuard = { saving: false }}
								{#key tab.id + '|' + tab.path}
								<NotePane
									value={_sbody}
									title={tab.name.replace(/\.md$/, '')}
									dir={_sdir}
									initialCursorPos={tab.cursorPos ?? 0}
									initialScrollTop={tab.scrollTop ?? 0}
									libraryName={tab.libraryName}
									tabId={tab.id}
									filePath={tab.path}
									libraryPath={tab.libraryPath || ''}
									noteNames={allNotes}
									allTags={allTagsList}
									properties={_sp.properties}
									rawYaml={_sp.rawYaml ?? ''}
									stage={_sp.properties.find(p => p.key.toLowerCase() === 'stage')?.value ?? ''}
									canGoBack={(tab.historyIndex ?? 0) > 0}
									canGoForward={(tab.historyIndex ?? 0) < (tab.history?.length ?? 1) - 1}
									onchange={() => {}}
									onpromote={(nextStage) => {
										const ct = get(openTabs).find(x => x.id === tab.id);
										const pr = ct ? parseFrontmatter(ct.content || '').properties : _sp.properties;
										const bd = ct ? parseFrontmatter(ct.content || '').body : _sbody;
										let np;
										if (!nextStage) { np = pr.filter(p => p.key.toLowerCase() !== 'stage'); }
										else {
											let u = false;
											np = pr.map(p => { if (p.key.toLowerCase() === 'stage') { u = true; return { ...p, value: nextStage }; } return p; });
											if (!u) np.push({ key: 'stage', value: nextStage, type: 'text' as any });
										}
										const fc = buildFullContent(np, bd);
										if (ct) { ct.content = fc; openTabs.update(tabs => tabs); }
										markRecentWrite(tab.path);
										writeNote(tab.path, fc).catch(() => {});
										const key = tab.path.replace(/\\/g, '/').toLowerCase();
										const nm = new Map(stageMap);
										if (nextStage) { nm.set(key, nextStage); } else { nm.delete(key); }
										stageMap = nm;
									}}
									onsave={(text) => {
										if (_sGuard.saving) return;
										_sGuard.saving = true;
										const ct = get(openTabs).find(x => x.id === tab.id);
										const pr = ct ? parseFrontmatter(ct.content || '').properties : _sp.properties;
										markRecentWrite(tab.path);
										const content = buildFullContent(pr, text);
										writeNote(tab.path, content).catch(() => {}).finally(() => { _sGuard.saving = false; });
									}}
									onflush={(text, needsDiskSave, cursorPos, scrollTop) => {
										const ct = get(openTabs).find(x => x.id === tab.id);
										const pr = ct ? parseFrontmatter(ct.content || '').properties : _sp.properties;
										const content = buildFullContent(pr, text);
										if (ct) { ct.content = content; ct.cursorPos = cursorPos; ct.scrollTop = scrollTop; }
										setWriteAhead(tab.path, content, cursorPos, scrollTop);
										if (needsDiskSave) {
											markRecentWrite(tab.path);
											writeNote(tab.path, content).then(() => clearWriteAhead(tab.path)).catch(() => {});
										}
									}}
									ontitlechange={(newTitle) => {
										if (newTitle !== tab.name.replace(/\.md$/, '')) {
											renameItem(tab.path, tab.path.replace(/[^/\\]+$/, newTitle + '.md'));
										}
									}}
									onnavigateback={() => { setFocusedTab(tab.id); navigateBack(); }}
									onnavigateforward={() => { setFocusedTab(tab.id); navigateForward(); }}
									onpropschange={() => { openTabs.update(tabs => tabs); }}
								/>
								{/key}
							{:else}
								<div class="new-tab-screen"><p>{$t('tabs.newTab')}</p></div>
							{/if}
							</div>
						{/each}
					{:else if $activeTab && !$activeTab.path}
						<div class="new-tab-screen">
							<div class="nt-commands">
								<button class="nt-command" onclick={handleNewNote}>
									{$t('commands.newNote')} ({sc('new-note')})
								</button>
								<button class="nt-command" onclick={() => { showQuickSwitcher = true; }}>
									{$t('tabs.openNote')} ({sc('quick-switch')})
								</button>
								<button class="nt-command" onclick={closeNote}>
									{$t('tabs.closeTab')}
								</button>
							</div>
						</div>
					{:else if $activeTab && $activeTab.path}
						{@const _parsed = parseFrontmatter($activeTab.content || '')}
						{@const _body = _parsed.body}
						{#key $activeTab.id + '|' + $activeTab.path}
						{@const _mountedTab = $activeTab}
						{@const _saveGuard = { saving: false }}
						{#if focusMode}
							<FocusPane
								value={_body}
								title={_mountedTab.name.replace(/\.md$/, '')}
								dir={noteDir}
								onchange={(text) => {
									const currentTab = get(openTabs).find(x => x.id === _mountedTab.id);
									const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
									const fc = buildFullContent(props, text);
									if (currentTab) currentTab.content = fc;
									markRecentWrite(_mountedTab.path);
									writeNote(_mountedTab.path, fc).catch(() => {});
								}}
								onexit={(promote) => {
									focusMode = false;
									if (promote) {
										// CE Phase 6: promote to permanent on FocusPane exit
										const currentTab = get(openTabs).find(x => x.id === _mountedTab.id);
										const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
										const body = currentTab ? parseFrontmatter(currentTab.content || '').body : _parsed.body;
										let updated = false;
										const newProps = props.map(p => {
											if (p.key.toLowerCase() === 'stage') { updated = true; return { ...p, value: promote }; }
											return p;
										});
										if (!updated) newProps.push({ key: 'stage', value: promote, type: 'text' as any });
										const fc = buildFullContent(newProps, body);
										if (currentTab) {
											currentTab.content = fc;
											openTabs.update(tabs => tabs);
										}
										markRecentWrite(_mountedTab.path);
										writeNote(_mountedTab.path, fc).catch(() => {});
									}
								}}
							/>
						{:else}
						<NotePane
							value={_body}
							title={_mountedTab.name.replace(/\.md$/, '')}
							dir={noteDir}
							initialCursorPos={_mountedTab.cursorPos ?? 0}
							initialScrollTop={_mountedTab.scrollTop ?? 0}
							libraryName={_mountedTab.libraryName}
							tabId={_mountedTab.id}
							filePath={_mountedTab.path}
							libraryPath={_mountedTab.libraryPath || ''}
							noteNames={allNotes}
							allTags={allTagsList}
							properties={_parsed.properties}
							rawYaml={_parsed.rawYaml ?? ''}
							stage={_parsed.properties.find(p => p.key.toLowerCase() === 'stage')?.value ?? ''}
							trail={activeTrail ? activeTrail.title : ''}
							trailIndex={trailIndex}
							trailTotal={activeTrail ? activeTrail.notes.length : 0}
							onTrailPrev={async () => {
								if (activeTrail && trailIndex > 0) {
									trailIndex--;
									const note = activeTrail.notes[trailIndex];
									if (note.exists) {
										const lib = get(libraries)[0];
										if (lib) await openNoteTab(note.path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
									}
								}
							}}
							onTrailNext={async () => {
								if (activeTrail && trailIndex < activeTrail.notes.length - 1) {
									trailIndex++;
									const note = activeTrail.notes[trailIndex];
									if (note.exists) {
										const lib = get(libraries)[0];
										if (lib) await openNoteTab(note.path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
									}
								}
							}}
							canGoBack={(_mountedTab.historyIndex ?? 0) > 0}
							canGoForward={(_mountedTab.historyIndex ?? 0) < (_mountedTab.history?.length ?? 1) - 1}
							onchange={() => {}}
							onpromote={(nextStage) => {
								const currentTab = get(openTabs).find(x => x.id === _mountedTab.id);
								const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
								const body = currentTab ? parseFrontmatter(currentTab.content || '').body : _parsed.body;
								let newProps;
								if (!nextStage) {
									// Remove stage property when "— Stage —" selected
									newProps = props.filter(p => p.key.toLowerCase() !== 'stage');
								} else {
									let updated = false;
									newProps = props.map(p => {
										if (p.key.toLowerCase() === 'stage') { updated = true; return { ...p, value: nextStage }; }
										return p;
									});
									if (!updated) newProps.push({ key: 'stage', value: nextStage, type: 'text' as any });
								}
								const fc = buildFullContent(newProps, body);
								if (currentTab) {
									currentTab.content = fc;
									// Force store reactivity so Properties panel refreshes
									openTabs.update(tabs => tabs);
								}
								markRecentWrite(_mountedTab.path);
								writeNote(_mountedTab.path, fc).catch(() => {});
								// Update stageMap immediately (don't wait for full library refresh)
								const key = _mountedTab.path.replace(/\\/g, '/').toLowerCase();
								const newMap = new Map(stageMap);
								if (nextStage) { newMap.set(key, nextStage); } else { newMap.delete(key); }
								stageMap = newMap;
							}}
							onsave={(text) => {
								if (_saveGuard.saving) return;
								_saveGuard.saving = true;
								/* Re-read properties from store (PropertyEditor may have updated them) */
								const currentTab = get(openTabs).find(x => x.id === _mountedTab.id);
								const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
								markRecentWrite(_mountedTab.path);
								const content = buildFullContent(props, text);
								writeNote(_mountedTab.path, content)
									.catch(() => {})
									.finally(() => { _saveGuard.saving = false; });
							}}
							onflush={(text, needsDiskSave, cursorPos, scrollTop) => {
								/* Re-read properties from store (PropertyEditor may have updated them) */
								const currentTab = get(openTabs).find(x => x.id === _mountedTab.id);
								const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
								const content = buildFullContent(props, text);
								if (currentTab) {
									currentTab.content = content;
									currentTab.cursorPos = cursorPos;
									currentTab.scrollTop = scrollTop;
								}
								setWriteAhead(_mountedTab.path, content, cursorPos, scrollTop);
								if (needsDiskSave) {
									markRecentWrite(_mountedTab.path);
									writeNote(_mountedTab.path, content)
										.then(() => clearWriteAhead(_mountedTab.path))
										.catch(() => {});
								}
							}}
							ontitlechange={(newTitle) => {
								if (newTitle !== _mountedTab.name.replace(/\.md$/, '')) {
									renameItem(_mountedTab.path, _mountedTab.path.replace(/[^/\\]+$/, newTitle + '.md'));
								}
							}}
							onnavigateback={() => navigateBack()}
							onnavigateforward={() => navigateForward()}
							onmoreaction={async (action) => {
								switch (action) {
									case 'rename': {
										const input = document.querySelector('.e-title') as HTMLInputElement;
										if (input) { input.focus(); input.select(); }
										break;
									}
									case 'showInExplorer':
										try { await invoke('constellation_show_in_folder', { path: _mountedTab.path }); } catch {}
										break;
									case 'openDefaultApp':
										try { await invoke('open_path', { path: _mountedTab.path }); } catch {}
										break;
									case 'revealInTree':
										window.dispatchEvent(new CustomEvent('constellation:reveal-in-tree', { detail: { path: _mountedTab.path } }));
										break;
									case 'copyPath':
										navigator.clipboard.writeText(_mountedTab.path).catch(() => {});
										break;
									case 'copyName':
										navigator.clipboard.writeText(_mountedTab.name).catch(() => {});
										break;
									case 'delete':
										window.dispatchEvent(new CustomEvent('constellation:delete-note', { detail: { path: _mountedTab.path, name: _mountedTab.name } }));
										break;
									case 'addProperty':
										window.dispatchEvent(new CustomEvent('constellation:add-property', { detail: { path: _mountedTab.path } }));
										break;
									case 'switchToFocus':
										focusMode = true;
										break;
								}
							}}
						/>
						{/if}
						{/key}
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
				<button class="rs-tab" class:active={rightSidebarTab === 'health'} onclick={() => rightSidebarTab = 'health'} title={$t('panels.health') || 'Knowledge Health'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'provenance'} onclick={() => {
					rightSidebarTab = 'provenance';
					_lastProvenancePath = ''; // reset cache to force fresh fetch
					const tab = $focusedTab;
					if (tab?.path && tab?.libraryPath) {
						_lastProvenancePath = tab.path;
						invoke<any>('get_provenance_chain', { libraryPath: tab.libraryPath, notePath: tab.path, maxDepth: 10 })
							.then(chain => { provenanceChain = chain; }).catch(() => { provenanceChain = null; });
					}
				}} title={$t('panels.provenance') || 'Provenance'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v6M12 22v-6M2 12h6M22 12h-6"/><circle cx="12" cy="12" r="3"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'review'} onclick={() => {
					rightSidebarTab = 'review';
					const lib = get(libraries)[0];
					if (lib) invoke<any[]>('get_due_notes', { libraryPath: lib.path })
						.then(notes => { dueNotes = notes; }).catch(() => { dueNotes = []; });
				}} title={$t('panels.review') || 'Review Pulse'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
					{#if dueNotes.length > 0}<span class="rs-tab-badge">{dueNotes.length}</span>{/if}
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
				{:else if rightSidebarTab === 'health'}
					<div class="rs-section rs-full-height">
						<TensionPanel
							report={tensionReport}
							{libraryColorMap}
							onNoteClick={(path, name) => {
								const lib = $libraryStats.find(l => path.startsWith(l.path));
								if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
							}}
						/>
					</div>
				{:else if rightSidebarTab === 'provenance'}
					<div class="rs-section rs-full-height">
						<ProvenancePanel
							chain={provenanceChain}
							{libraryColorMap}
							onNoteClick={(path, name) => {
								const lib = $libraryStats.find(l => path.startsWith(l.path));
								if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
							}}
						/>
					</div>
				{:else if rightSidebarTab === 'review'}
					<div class="rs-section rs-full-height">
						<ReviewPulsePanel
							{dueNotes}
							onNoteClick={(path, name) => {
								const lib = $libraryStats.find(l => path.startsWith(l.path));
								if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
							}}
							onRefresh={() => {
								const lib = get(libraries)[0];
								if (lib) invoke<any[]>('get_due_notes', { libraryPath: lib.path })
									.then(notes => { dueNotes = notes; }).catch(() => {});
							}}
						/>
					</div>
				{/if}
			{:else if rightSidebarTab === 'review'}
				<!-- Review Pulse works without a note open (library-level feature) -->
				<div class="rs-section rs-full-height">
					<ReviewPulsePanel
						{dueNotes}
						onNoteClick={(path, name) => {
							const lib = $libraryStats.find(l => path.startsWith(l.path));
							if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
						}}
						onRefresh={() => {
							const lib = get(libraries)[0];
							if (lib) invoke<any[]>('get_due_notes', { libraryPath: lib.path })
								.then(notes => { dueNotes = notes; }).catch(() => {});
						}}
					/>
				</div>
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
							let unlistenFn: (() => void) | null = null;
							const timeout = setTimeout(() => { unlistenFn?.(); reject(new Error('timeout')); }, 2000);
							onStateResponse((state) => {
								clearTimeout(timeout);
								unlistenFn?.();
								resolve(state);
							}).then((unlisten) => {
								unlistenFn = unlisten;
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
					const validTabs = ['properties', 'backlinks', 'tags', 'star', 'tasks', 'calendar', 'health', 'provenance', 'review'] as const;
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
		min-height: 34px; display: flex; flex-direction: column;
	}
	.toolbar-actions { display: flex; gap: 2px; align-items: center; padding: 2px 0; }
	.tb-btn {
		width: 26px; height: 26px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
	}
	.tb-btn:hover { background: var(--border); color: var(--text); }
	.tb-btn.active { color: var(--interactive-accent); }

	.toolbar-section-label {
		font-size: 10px; color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.5px;
		padding: 4px 8px 0; font-family: inherit;
	}

	.toolbar-modes {
		display: flex; gap: 2px; padding: 2px 6px 4px; border-top: 1px solid var(--background-modifier-border);
	}
	.mode-tab {
		flex: 1; display: flex; align-items: center; justify-content: center; gap: 4px;
		padding: 3px 4px; border: none; border-radius: 4px; background: transparent;
		color: var(--text-muted); cursor: pointer; font-size: 11px; font-family: inherit;
	}
	.mode-tab:hover { background: var(--background-modifier-hover); }
	.mode-tab.active { background: color-mix(in srgb, var(--interactive-accent) 15%, transparent); color: var(--interactive-accent); font-weight: 600; }
	.lens-select {
		font-size: 10px; padding: 2px 4px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-muted);
		cursor: pointer; outline: none; max-width: 100px; font-family: inherit;
	}
	.lens-select:hover { border-color: var(--interactive-accent); }

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
	.universe-notes-item {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 12px;
		font-size: 0.8rem;
		color: var(--interactive-accent);
		font-weight: 600;
	}
	.child-universe-item {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 12px;
		font-size: 0.8rem;
		color: var(--text-secondary);
		font-weight: 600;
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
		margin-inline-start: auto;
	}
	.child-universe-libs {
		padding-inline-start: 16px;
	}
	.library-section-nested {
	}
	.library-section-nested .library-header {
		font-weight: 500;
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
		overflow: hidden; background: #e8e8ec;
	}

	/* Layout bar (sidebar + split controls, independent from tabs) */
	.layout-bar {
		display: flex; align-items: center;
		background: var(--bg-secondary);
		padding: 4px 8px;
		flex-shrink: 0;
		gap: 4px;
		box-shadow: 0 -1px 0 0 rgba(0,0,0,0.08) inset;
	}

	/* Tab bar (locked to paper edge) */
	.tab-bar {
		display: flex; flex-direction: column; align-items: center;
		background: #e8e8ec; border-bottom: none;
		flex-shrink: 0;
		padding: 5px 32px 0;
	}
	.tab-scroll-wrap {
		display: flex; align-items: center;
		width: 100%;
		max-width: 1200px;
		gap: 2px;
		margin-inline-start: -9px;
	}
	.tab-scroll {
		min-width: 0; display: flex; align-items: flex-end;
		gap: 1px; padding: 12px 4px 0; overflow-x: auto;
		flex: 1;
	}
	.tab-scroll::-webkit-scrollbar { height: 0; }
	.tab-scroll { scrollbar-width: none; }
	.tab-scroll-arrow {
		flex-shrink: 0; width: 28px; height: 32px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: rgba(0,0,0,0.08); border-radius: 6px;
		color: var(--text-normal); cursor: pointer;
	}
	.tab-scroll-end { margin-inline-end: 8px; }
	.tab-scroll-arrow svg { width: 16px; height: 16px; }
	.tab-scroll-arrow:hover { background: rgba(0,0,0,0.15); }
	:global([dir="rtl"]) .tab-scroll-arrow svg { transform: scaleX(-1); }
	.tab-scroll.no-tabs { margin-inline-start: 0; padding: 0; }
	.tab {
		display: flex; align-items: center; gap: 6px;
		padding: 5px 10px; font-size: 0.8rem; color: var(--text-secondary);
		background: #dcdce0; border-radius: 6px 6px 0 0;
		cursor: pointer; min-width: 0;
		border: none; font-family: inherit; flex-shrink: 0;
		border-top: 3px solid var(--library-color, transparent);
		position: relative;
	}
	.tab.active, .tab.focused {
		background: #ffffff; color: var(--text);
		border: 1px solid #d0d0d0;
		border-top: 3px solid var(--library-color, var(--accent));
		border-bottom: 1px solid #ffffff;
		margin-bottom: -1px;
	}
	.tab.drag-over {
		border-inline-start: 3px solid var(--interactive-accent);
		background: color-mix(in srgb, var(--interactive-accent) 10%, var(--background-primary));
	}
	.tab[draggable="true"] { cursor: grab; }
	.tab[draggable="true"]:active { cursor: grabbing; }
	.tab-lib-name {
		position: absolute; bottom: calc(100% + 4px); inset-inline-end: 8px;
		font-size: 0.55rem; line-height: 1.3; letter-spacing: 0.02em;
		color: var(--text);
		background: #e8e8ec;
		padding: 0 5px;
		border-radius: 3px 3px 0 0;
		border: none;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%; pointer-events: none;
		z-index: 1;
	}
	.tab-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; pointer-events: none; }
	.tab-maturity { font-size: 12px; flex-shrink: 0; margin-inline-end: 3px; }
	.tab-maturity.mat-sapling { color: #4ade80; }
	.tab-maturity.mat-evergreen { color: #16a34a; }
	.tab-maturity.mat-canonical { color: #f59e0b; }
	.tab-maturity.mat-wilting { color: #16a34a; opacity: 0.4; }
	.tab.pinned { min-width: 36px; padding: 0 8px; }
	.tab-pin { font-size: 0.65rem; flex-shrink: 0; pointer-events: none; }
	.tab { cursor: grab; }
	.tab:active { cursor: grabbing; }
	.tab-dragging { opacity: 0.5; transform: scale(0.95); transition: opacity 0.1s, transform 0.1s; cursor: grabbing !important; }
	.tab.drag-over { border-inline-start: 3px solid var(--interactive-accent); background: color-mix(in srgb, var(--interactive-accent) 10%, transparent); }
	.tab-close {
		background: none; border: none; color: var(--text-muted);
		cursor: pointer; font-size: 0.85rem; padding: 0; line-height: 1;
		border-radius: 3px; text-decoration: none;
		display: flex; align-items: center; justify-content: center;
		width: 16px; height: 16px; flex-shrink: 0;
		pointer-events: auto;
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
	.tab-ctx-menu {
		position: fixed; z-index: 9999;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px 0;
		min-width: 180px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.15);
		direction: ltr;
	}
	.tab-ctx-item {
		display: block; width: 100%; padding: 6px 14px;
		border: none; background: none; cursor: pointer;
		font-size: 13px; color: var(--text-normal);
		text-align: left; font-family: var(--font-interface-theme);
	}
	.tab-ctx-item:hover { background: var(--background-modifier-hover); }
	.tab-ctx-item:disabled { opacity: 0.4; cursor: default; }
	.tab-ctx-item:disabled:hover { background: none; }
	.tab-ctx-sep { height: 1px; margin: 4px 8px; background: var(--background-modifier-border); }
	.tab-new {
		min-width: 32px !important; max-width: 32px;
		padding: 4px 0 !important; justify-content: center;
		background: transparent !important; color: var(--text-muted);
		border: 1px solid #4caf50 !important; border-bottom: none !important; border-radius: 6px 6px 0 0;
		cursor: pointer; position: relative;
	}
	.tab-bulb-icon {
		position: absolute; opacity: 0;
		color: #ff9800;
	}
	.tab-new:hover { background: color-mix(in srgb, #4caf50 10%, transparent) !important; color: #4caf50; }

	/* When no tabs open: full circle centered in bar */
	.tab-scroll.no-tabs {
		justify-content: center; align-items: center;
	}
	.tab-scroll.no-tabs ~ .tab-scroll-arrow { display: none; }
	.tab-bar:has(.tab-scroll.no-tabs) .tab-scroll-wrap {
		flex: 1;
		margin-inline-start: 0 !important;
		max-width: none !important;
		justify-content: center;
	}
	.tab-new-wrap {
		display: flex; flex-direction: column; align-items: center;
		flex-shrink: 0;
	}
	.tab-new-hint {
		font-size: 11px; color: var(--text-faint); margin-top: 4px;
		white-space: nowrap;
	}
	.tab-scroll.no-tabs .tab-new {
		min-width: 26px !important; max-width: 26px;
		height: 26px;
		border-radius: 50% !important;
		border: 1px solid #4caf50 !important;
		border-bottom: 1px solid #4caf50 !important;
		padding: 0 !important;
		margin-top: 20px;
	}

	/* Slow pulse: 3 blinks of +, then 2 blinks of bulb. Total cycle = 15s (5 blinks × 3s each) */
	.tab-scroll.no-tabs .tab-new svg.tab-plus-icon {
		animation: plus-cycle 15s ease-in-out infinite;
	}
	.tab-scroll.no-tabs .tab-new svg.tab-bulb-icon {
		animation: bulb-cycle 15s ease-in-out infinite;
	}
	/* Plus: blinks at 0-9s (3 blinks), hidden at 9-15s */
	@keyframes plus-cycle {
		0% { opacity: 0.3; }
		3% { opacity: 1; }
		10% { opacity: 0.3; }
		13% { opacity: 0.3; }
		16% { opacity: 1; }
		23% { opacity: 0.3; }
		26% { opacity: 0.3; }
		29% { opacity: 1; }
		36% { opacity: 0.3; }
		58% { opacity: 0.3; }
		59% { opacity: 0; }
		99% { opacity: 0; }
		100% { opacity: 0.3; }
	}
	/* Bulb: hidden at 0-9s, blinks at 9-15s (2 blinks) */
	@keyframes bulb-cycle {
		0%, 59% { opacity: 0; }
		62% { opacity: 1; }
		72% { opacity: 0.2; }
		75% { opacity: 0.2; }
		78% { opacity: 1; }
		88% { opacity: 0.2; }
		95% { opacity: 0; }
		100% { opacity: 0; }
	}

	/* Content */
	.content-area { flex: 1; overflow: hidden; display: flex; flex-direction: column; background: #e8e8ec; }

	/* Pane container */
	.pane-container {
		flex: 1; display: flex; flex-direction: row; overflow: hidden; background: #e8e8ec;
	}
	.pane-container > :global(*) { flex: 1; min-width: 0; min-height: 0; }
	.pane-container.horizontal { flex-direction: column; }
	.pane-divider { flex: 0 0 auto !important; background: var(--border); }
	.pane-container:not(.horizontal) > .pane-divider { width: 3px; cursor: col-resize; }
	.pane-container.horizontal > .pane-divider { height: 3px; cursor: row-resize; }
	.pane-divider:hover { background: var(--accent); }
	.split-pane-wrap { display: flex; flex-direction: column; flex: 1; min-width: 0; min-height: 0; overflow: hidden; }
	.split-pane-wrap :global(.e-desk) { padding-inline: 8px !important; }

	/* Star fullscreen */
	.star-fullscreen {
		flex: 1; display: flex; flex-direction: column; overflow: hidden;
		background: var(--background-primary, #fff);
	}
	.tab-bar-hidden { display: none !important; }
	.star-header {
		display: flex; align-items: center; gap: 4px;
		padding: 8px 16px; border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
	}
	.star-title { font-weight: 600; font-size: 0.9rem; flex: 1; }
	.star-wiw-toggle {
		width: 26px; height: 26px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 4px; color: var(--text-muted); cursor: pointer;
		opacity: 0.5; transition: opacity 0.12s, color 0.12s; margin-inline-end: 2px;
	}
	.star-wiw-toggle:hover { opacity: 0.9; }
	.star-wiw-toggle.active { color: var(--interactive-accent); opacity: 1; }
	.star-close {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
		font-size: 1.2rem;
	}
	.star-close:hover { background: var(--border); color: var(--text); }

	/* WiW overlay */
	.wiw-overlay {
		position: fixed; z-index: 1000; display: flex; flex-direction: column;
		border: 2px solid var(--interactive-accent);
		border-radius: 10px; overflow: hidden;
		box-shadow: 0 8px 32px rgba(0,0,0,0.3);
		background: var(--background-primary);
	}
	.wiw-header {
		display: flex; flex-direction: column; align-items: stretch; gap: 2px;
		padding: 5px 10px;
		background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		cursor: move; user-select: none;
	}
	.wiw-header-row { display: flex; align-items: center; gap: 8px; }
	.wiw-title {
		flex: 1; font-size: 12px; font-weight: 600;
		color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.wiw-subtitle { font-size: 10px; color: var(--text-muted); opacity: 0.65; pointer-events: none; line-height: 1.2; }
	.wiw-count {
		font-size: 10px; color: var(--text-faint);
		background: var(--background-modifier-hover);
		padding: 1px 6px; border-radius: 8px; flex-shrink: 0;
	}
	.wiw-close {
		width: 20px; height: 20px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px;
		color: var(--text-muted); cursor: pointer; font-size: 14px; flex-shrink: 0;
	}
	.wiw-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.wiw-graph { flex: 1; overflow: hidden; }
	.wiw-resize { position: absolute; z-index: 1; }
	.wiw-resize-n { top: 0; left: 6px; right: 6px; height: 4px; cursor: n-resize; }
	.wiw-resize-s { bottom: 0; left: 6px; right: 6px; height: 4px; cursor: s-resize; }
	.wiw-resize-e { top: 6px; right: 0; bottom: 6px; width: 4px; cursor: e-resize; }
	.wiw-resize-w { top: 6px; left: 0; bottom: 6px; width: 4px; cursor: w-resize; }
	.wiw-resize-se { bottom: 0; right: 0; width: 10px; height: 10px; cursor: se-resize; }
	.wiw-resize-sw { bottom: 0; left: 0; width: 10px; height: 10px; cursor: sw-resize; }
	.wiw-resize-ne { top: 0; right: 0; width: 10px; height: 10px; cursor: ne-resize; }
	.wiw-resize-nw { top: 0; left: 0; width: 10px; height: 10px; cursor: nw-resize; }

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

	/* New tab screen */
	.new-tab-screen {
		flex: 1; display: flex; flex-direction: column;
		align-items: center; justify-content: center;
		padding: 2rem;
	}
	.nt-commands {
		display: flex; flex-direction: column; align-items: center; gap: 14px;
	}
	.nt-command {
		background: none; border: none; padding: 0;
		color: var(--accent); font-size: 1rem;
		font-family: inherit; cursor: pointer; opacity: 0.75;
		transition: opacity 0.12s;
	}
	.nt-command:hover { opacity: 1; }

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
	.rs-tab-badge {
		position: absolute; top: -2px; inset-inline-end: -2px;
		font-size: 0.55rem; background: var(--interactive-accent); color: white;
		border-radius: 6px; padding: 0 3px; min-width: 12px; text-align: center;
		line-height: 1.3;
	}
	.rs-tab { position: relative; }

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

	/* ═══ Focus — hide UI elements ═══ */
	:global(body.focus-active) .sidebar { display: none !important; }
	:global(body.focus-active) .ribbon { display: none !important; }
	/* layout-ribbon removed — controls merged into tab-bar */
	:global(body.focus-active) .tab-bar { display: none !important; }
	:global(body.focus-active) .status-bar { display: none !important; }
	:global(body.focus-active) .wiw-overlay { display: none !important; }
</style>

<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { dir, t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { getVersion } from '@tauri-apps/api/app';
	import {
		libraries, libraryStats, totalStars, libraryCount,
		activeTab, openTabs, activeTabId,
		splitActive, splitDirection, focusedTabId, focusedTab,
		loadLibraries, loadAllStats, addLibrary, createNewLibrary,
		initSearchIndex,
		type ConstellationSearchResult,
		openNoteTab, closeTab, switchTab, reorderTab, closeNote, createEmptyTab,
		toggleSplit, toggleSplitDirection, setFocusedTab,
		parseFrontmatter, extractHeadings, saveTabContent, updateTabContent, buildFullContent, writeNote, markRecentWrite, setWriteAhead, getWriteAhead, clearWriteAhead,
		createNote, createFolder, renameItem, deleteItem,
		startWatchingLibrary, wasRecentlyWritten,
		loadLibraryAppearance, libraryAppearances,
		toggleEditMode, editingTabIds,
		navigateBack, navigateForward,
		scanLibraryLinks, scanLibraryTags, getBacklinks, getOutgoingLinks, scanUnlinkedMentions,
		scanLibraryIndex, readIndexEntries, readTermMentions, readCooccurringTerms,
		buildSkyData, readNotePreview,
		getDailyNotePath, updateLinksOnRename, getOldTitleForCascade, quickCapture,
		loadBookmarks, addBookmark, removeBookmark, isBookmarked, bookmarks,
		loadSettings, updateSettings, appSettings, DEFAULT_SETTINGS,
		loadWorkspaces, workspaces,
		resolveWikilinkCrossLibrary,
		buildDefaultFrontmatter,
		linkTraversalBumps, clearLinkTraversalBumps,
		type FrontmatterProperty, type HeadingItem, type NoteLink, type SkyNode, type SkyLink,
		type IndexEntry
	} from '$lib/libraries/store';
	import type { LibraryStats, FileEntry, WorkspaceLayout, WorkspaceSecondScreen, FontSet, PanelId } from '$lib/libraries/store';
	import { BUILTIN_FONT_SETS, SCRIPT_UNICODE_RANGES, TYPEWRITER_FONTS, getFontSetById, BUILTIN_THEMES, deriveThemeVariables, hexToHSL } from '$lib/libraries/store';
	import { generateStyleSettingsCSS } from '$lib/theme/styleSettings';
	import { CORE_BLOCK_IDS, getEffectiveStyleBlocks } from '$lib/theme/constellationStyleSettings';
	import { get } from 'svelte/store';
	import { detectDir, eventToShortcut, normalizeShortcut, getResolvedShortcut, formatShortcut } from '$lib/utils';
	import { createBase, saveBaseFile, listWorkspaceBases, createWorkspaceBase, saveWorkspaceBase, deleteWorkspaceBase } from '$lib/bases/store';
	import type { WorkspaceBaseEntry } from '$lib/bases/store';
	import type { BaseDefinition } from '$lib/bases/types';
	import FileTree from '$lib/components/FileTree.svelte';
	import NotebookNavigator from '$lib/components/NotebookNavigator.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
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
	import ConstellationSight from '$lib/components/ConstellationSight2.svelte';
	import { detectClusters, computeStructuralGaps, computeUniverseHealth, buildCommunityProfiles, stratumWeightedCentrality, suggestBridges, type StructuralGap, type UniverseHealth, type ClusterInfo, type CommunityProfile } from '$lib/graph/clusterEngine';
	import OrgChart from '$lib/components/OrgChart.svelte';
	import EmojiIconPicker from '$lib/components/EmojiIconPicker.svelte';
	import SlotIcon from '$lib/components/SlotIcon.svelte';
	import SearchHub from '$lib/components/SearchHub.svelte';
	import LocalSkyView from '$lib/components/LocalSkyView.svelte';
	import NoteGrid from '$lib/components/NoteGrid.svelte';
	import BacklinksPanel from '$lib/components/BacklinksPanel.svelte';
	import TagsPanel from '$lib/components/TagsPanel.svelte';

	import { embedNotes, embeddingStatus } from '$lib/libraries/store';
	import DashboardView from '$lib/components/DashboardView.svelte';
	import KnowledgeHealthDashboard from '$lib/components/KnowledgeHealthDashboard.svelte';
	import TasksPanel from '$lib/components/TasksPanel.svelte';
	import CalendarPanel from '$lib/components/CalendarPanel.svelte';
	import GlobalTasksView from '$lib/components/GlobalTasksView.svelte';
	import TensionPanel from '$lib/components/TensionPanel.svelte';
	import ProvenancePanel from '$lib/components/ProvenancePanel.svelte';
	import ReviewPulsePanel from '$lib/components/ReviewPulsePanel.svelte';
	import LinkDashboard from '$lib/components/LinkDashboard.svelte';
	import ExpressionForge from '$lib/components/ExpressionForge.svelte';
	import SenseMakingCanvas from '$lib/components/SenseMakingCanvas.svelte';
	import ConstellationMap from '$lib/components/ConstellationMap.svelte';
	import Inspector360 from '$lib/components/Inspector360.svelte';
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
	import CanonicalChoiceDialog from '$lib/components/CanonicalChoiceDialog.svelte';
	import {
		listUniverses, createUniverse, setActiveUniverse,
		checkMigrationNeeded, migrateLegacyData,
		getChildUniverses,
		type UniverseEntry, type ChildUniverseInfo
	} from '$lib/universe/store';
	import { loadPropertyTypes } from '$lib/libraries/propertyTypeRegistry';
	import { openSecondScreen, openSecondScreenSmart, closeSecondScreen, isSecondScreenOpen, hasMultipleMonitors, waitForScreenReady, sendNoteToScreen, onNoteToMain, onScreenClosed, onNoteSaved, broadcastNoteSaved, notifyUniverseSwitch, notifySettingsChanged, requestScreenState, onStateResponse, sendWorkspaceRestore, emitContextChanged, emitSkyViewHover, emitSkyViewClick, emitSidebarModeChanged, emitSplitModeChanged, emitDashboardOpenNote, emitDashboardTagSelected, emitIndexTermSelected, emitIndexCompare, emitMapCompanion, emitEditorPanels, type ScreenNote, type ScreenState, type SkyViewNodeInfo } from '$lib/secondScreen';
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
	// searchMode removed — Search Hub is the single search experience
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
	let searchEngineReady = $state(false); // true when SQLite FTS5 index is built
	let semanticIndexProgress = $state('');
	let semanticIndexing = $state(false);

	// Canonical system state
	let canonicalizing = $state(false);
	let canonicalProgress = $state({ current: 0, total: 0, currentFile: '', libraryName: '', phase: '' });
	let showCanonicalChoice = $state(false);
	let pendingLibraryPath = $state('');
	let sortOrder = $state<'name-asc' | 'name-desc' | 'modified-desc' | 'modified-asc'>('name-asc');
	let libraryPickerAction = $state<'note' | 'folder' | 'base'>('note');
	let allExpanded = $state(true);

	// Universe state
	let showUniverseSetup = $state(false);
	let showUniverseManager = $state(false);
	let activeUniverseName = $state('');
	let appVersion = $state('');
	let appReady = $state(false);
	// Distinguishes "hydration in progress" from "genuinely has no libraries".
	// Flips to true ONLY after loadLibraries() completes (success or empty).
	// Used to gate the Welcome/Create screen — we don't want that screen to
	// flash during the window between paint and libraries loading.
	let librariesLoaded = $state(false);

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
	let rightSidebarTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks' | 'calendar' | 'health' | 'provenance' | 'review' | 'links' | 'inspector360'>('properties');

	// ── Sidebar overlay snapshots ───────────────────────────────────────────
	// Every overlay mode that takes over the editor area (full-page view,
	// Search Hub, OrgChart, Lens, SV inspect mode) hides both sidebars on
	// entry and restores them on exit. Before this helper there were 5 pairs
	// of `sidebarBeforeX` / `rightSidebarBeforeX` variables with copy-pasted
	// save/restore logic at ~15 sites. The Map below is the single snapshot
	// store; push/pop are the single save/restore API. Adding a new overlay
	// mode is now a one-liner `pushSidebars('newKey')`.
	// Only two distinct snapshot domains:
	// - 'fullPage' covers every overlay included in fullPageActive (Sky View,
	//   Search Hub, OrgChart, Lens, Dashboard, etc.). The fullPageActive
	//   $effect pushes once on first true, pops once on first false — so
	//   overlays inside this group share one snapshot and can't stomp it.
	// - 'skyInspect' is separate because it coexists with normal note-editor
	//   flow (not part of fullPageActive) and has its own dismiss path.
	type OverlayKey = 'fullPage' | 'skyInspect';
	const sidebarSnapshots = new Map<OverlayKey, { left: boolean; right: boolean }>();
	function pushSidebars(key: OverlayKey) {
		// Idempotent: re-entering the same overlay doesn't overwrite the
		// original snapshot. The first push wins so the exit restores the
		// state the user had before the first entry.
		if (!sidebarSnapshots.has(key)) {
			sidebarSnapshots.set(key, { left: sidebarOpen, right: rightSidebarOpen });
		}
		sidebarOpen = false;
		rightSidebarOpen = false;
	}
	function popSidebars(key: OverlayKey) {
		const snap = sidebarSnapshots.get(key);
		if (!snap) return;
		sidebarOpen = snap.left;
		rightSidebarOpen = snap.right;
		sidebarSnapshots.delete(key);
	}
	// For workspace-restore fired while an overlay is active: update the
	// stored target state so the eventual pop restores the workspace's
	// intended layout, not the pre-load state.
	function updateSidebarSnapshot(key: OverlayKey, left: boolean, right: boolean) {
		if (sidebarSnapshots.has(key)) {
			sidebarSnapshots.set(key, { left, right });
		}
	}
	let dueNotes = $state<any[]>([]); // CE Phase 7: ReviewPulse due notes
	let activeTrail = $state<any>(null); // CE Phase 8: active trail data
	let showExpressionForge = $state(false); // CE Phase 10
	let showSenseMakingCanvas = $state(false); // CE Phase 11
	let showConstellationMap = $state(false);
	let showSearchHub = $state(false);
	let showKnowledgeHealth = $state(false);
	let showPicker = $state(false);
	let searchHubMatchIds = $state<Set<string> | null>(null);
	let searchHubReturnPending = $state(false);
	let searchHubInitialQuery = $state('');
	// CE Phase 12 — 360° Inspector. Re-enabled 2026-04-29 (compact sidebar tab + full-window overlay).
	let showInspector360 = $state(false);
	let inspector360EverOpened = $state(false); // LL-022 lazy-mount sticky flag
	let inspector360Data = $state<any>(null);    // Note360View
	let inspector360FetchTimer: ReturnType<typeof setTimeout> | null = null;
	let inspector360RequestSeq = 0;
	let lastFetchedInspectorKey: string | null = null;
	let trailIndex = $state(0); // CE Phase 8: current note index in trail
	let tensionReport = $state<any>(null); // CE Phase 4: TensionReport
	let provenanceChain = $state<any>(null); // CE Phase 5: ProvenanceChain
	let _lastProvenancePath = ''; // cache guard — only re-fetch when note changes

	// Sidebar resizing
	let leftSidebarWidth = $state(300);
	let rightSidebarWidth = $state(340);
	let resizing = $state<'left' | 'right' | null>(null);

	// Flanking column widths — Tier 1b drag-resize. Initialized from appSettings on load.
	let leftFlankWidth = $state(280);
	let rightFlankWidth = $state(280);
	/** Tier 1b: collapsed flanks — width goes to 0, panel content hidden. */
	let leftFlankCollapsed = $state(false);
	let rightFlankCollapsed = $state(false);
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
	let showSkyView = $state(false);
	let showOrgChart = $state(false);
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
			const mode = showSkyView ? 'skyview' : 'editor';
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
		if (secondScreenOpen && !showSkyView && $activeTab?.path) {
			sendNoteToScreen({
				path: $activeTab.path,
				name: $activeTab.name,
				libraryName: $activeTab.libraryName,
				libraryPath: $activeTab.libraryPath ?? '',
				libraryColor: $activeTab.libraryColor ?? '#7c3aed',
			});
			// Send editor panels data so SS can show properties/backlinks/tags
			emitEditorPanels({
				active: true,
				notePath: $activeTab.path,
				noteName: $activeTab.name,
				libraryName: $activeTab.libraryName,
				libraryPath: $activeTab.libraryPath ?? '',
				content: $activeTab.content,
			});
		}
	});
	// Track recently opened notes in localStorage (shared utility)
	import { addRecentOpened } from '$lib/libraries/recentNotes';
	$effect(() => {
		const tab = $activeTab;
		if (!tab?.path) return;
		addRecentOpened({ name: tab.name, path: tab.path, libraryName: tab.libraryName });
	});
	// Track initial content for diff baseline
	let lastSavedContent = $state('');
	$effect(() => {
		const tab = $activeTab;
		if (tab?.path) {
			invoke<string>('read_note', { filePath: tab.path }).then(saved => {
				lastSavedContent = saved;
			}).catch(() => {});
		}
	});


	// Sync split view state to second screen — send ALL open tabs for comparison
	$effect(() => {
		if (!secondScreenOpen) return;
		const active = $splitActive;
		const tabs = $openTabs;
		if (active && tabs.length > 0) {
			emitSplitModeChanged({
				active: true,
				notes: tabs.filter(t => t.path).map(t => ({
					notePath: t.path,
					noteName: t.name,
					libraryName: t.libraryName,
					libraryPath: t.libraryPath ?? '',
					content: t.content ?? '',
				})),
			});
		} else if (!active) {
			emitSplitModeChanged({ active: false });
		}
	});

	let showGlobalTasks = $state(false);
	let showIndex = $state(false);
	let indexNoteTab = $state<import('$lib/libraries/store').OpenTab | null>(null);
	let indexActiveNotePath = $state('');
	let indexSelectedTerms = $state<Set<string>>(new Set());
	let indexReturnPending = $state(false); // show "Return to Index" button on note tab
	let mapReturnPending = $state(false); // show "Return to Map" button on note tab
	let orgChartReturnPending = $state(false); // show "Return to OrgChart" button on note tab
	let lensReturnPending = $state(false);
	let skyViewReturnPending = $state(false);

	// "Note as organism" — flanking Backlinks/Outgoing panels only render
	// when the user arrived at the note by clicking a Sky-View node. This
	// is the "inspect mode" — SV → node → note-with-flanks → maybe return
	// to SV. Clicking Sky-View nodes sets it; explicit dismiss clears it.
	// A user who just opens a note via tree / quick-switcher / wikilink
	// sees the regular editor without flanks.
	let skyViewInspectMode = $state(false);
	// Sidebar snapshot for SV inspect mode: managed via pushSidebars/popSidebars('skyInspect')

	// Flank-side predicates for the tab-bar alignment in inspect mode.
	// Derived so changes to panelPlacements or inspect-mode flip immediately.
	const tabBarFlankLeft  = $derived(
		skyViewInspectMode && (
			$appSettings.panelPlacements?.backlinks === 'left-of-note' ||
			$appSettings.panelPlacements?.outgoing === 'left-of-note'
		)
	);
	const tabBarFlankRight = $derived(
		skyViewInspectMode && (
			$appSettings.panelPlacements?.backlinks === 'right-of-note' ||
			$appSettings.panelPlacements?.outgoing === 'right-of-note'
		)
	);
	// layoutCtrlDisabled / layoutCtrlDisabledReason defined after fullPageActive
	// (depends on it) — see the block right after fullPageActive declaration.
	let mapColorMode = $state<'maturity' | 'stratum' | 'library'>('maturity');
	let mapFocusNode = $state<any>(null); // current MapNode being viewed
	// Sticky lazy-mount flags — stay true after first open so drill-down state survives
	// close/reopen, reset on Universe switch. See LL-022.
	let mapEverOpened = $state(false);
	let orgChartEverOpened = $state(false);
	$effect(() => { if (showConstellationMap) mapEverOpened = true; });
	$effect(() => { if (showOrgChart) orgChartEverOpened = true; });
	$effect(() => { if (showInspector360) inspector360EverOpened = true; });
	// Inspector 360 IPC-fetch $effect is co-located with `sidebarTab`'s
	// declaration further down the file (TDZ); see "CE Phase 12 — fetch"
	// block.

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
	let hasMultipleDisplays = $state(false); // gate SS features behind 2+ monitors
	let rightSidebarBeforeSS = $state(false); // remember right sidebar state before SS hid it

	// Library management
	let showLibrarySwitcher = $state(false);
	let showLibraryManager = $state(false);
	let showLibraryPicker = $state(false);
	let showNewBaseDialog = $state(false);
	let showNewLibraryDropdown = $state(false);
	let newLibName = $state('');

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
	// MIG-004 §9: path → aliases[] map populated from cache_boot_snapshot_graph.
	// Used by getBacklinks to resolve wikilinks targeting an alias (rename'd
	// or frontmatter) to the canonical note. Also useful for any future
	// frontend code that needs to know a note's full identity set.
	let notePathToAliases = $state<Map<string, string[]>>(new Map());

	/** P5 deferred: after a user contests/force-promotes a link's confidence,
	 *  mirror the DB write into the in-memory NoteLink so the right-click
	 *  menu's "current" marker reflects the new value without a full rescan.
	 *  Everything else reads from this array via `effectiveLibraryLinks`. */
	function applyConfidenceLocally(sourcePath: string, targetName: string, confidence: string) {
		const target = targetName.toLowerCase();
		allLibraryLinks = allLibraryLinks.map(l =>
			l.source_path === sourcePath && l.target.toLowerCase() === target
				? { ...l, confidence }
				: l
		);
	}

	/** Living Link final: archived links are excluded from the Backlinks /
	 *  Outgoing panels and from the Most-Traveled / Stale tabs (via status
	 *  filters elsewhere). Mirror the DB write into memory so the row
	 *  disappears immediately without a full rescan. */
	function applyArchiveLocally(sourcePath: string, targetName: string) {
		const target = targetName.toLowerCase();
		allLibraryLinks = allLibraryLinks.map(l =>
			l.source_path === sourcePath && l.target.toLowerCase() === target
				? { ...l, status: 'archived', weight: 0 }
				: l
		);
	}
	// P4.2: per-(source,target) traversal counts, derived from the boot
	// graph PLUS any optimistic bumps fired by openNoteTab since the last
	// fetch. Key = `${sourcePath.toLowerCase()}|${target.toLowerCase()}`.
	// livePreview.ts consumes this via the linkTraversalMapField StateField
	// to render a `×N` chip after each traversed wikilink in the note body.
	// The bumps are cleared by clearLinkTraversalBumps() right after the
	// boot-graph payload lands so live increments don't double-count against
	// the server's already-updated values.
	const linkTraversalMap = $derived.by(() => {
		const m = new Map<string, number>();
		for (const l of allLibraryLinks) {
			const count = l.traversal_count ?? 0;
			if (count <= 0 || !l.source_path || !l.target) continue;
			m.set(l.source_path.toLowerCase() + '|' + l.target.toLowerCase(), count);
		}
		for (const [key, bump] of $linkTraversalBumps) {
			m.set(key, (m.get(key) ?? 0) + bump);
		}
		return m;
	});
	// Same fold as `linkTraversalMap` but projected back onto the NoteLink
	// list so consumers (`getBacklinks` / `getOutgoingLinks`) who read
	// `l.traversal_count` directly still see the live value. Cloning only
	// the entries that actually have a bump keeps the hot path at
	// `O(bumps)` per derivation, not `O(links)`.
	const effectiveLibraryLinks = $derived.by(() => {
		if ($linkTraversalBumps.size === 0) return allLibraryLinks;
		return allLibraryLinks.map(l => {
			const bump = $linkTraversalBumps.get(
				l.source_path.toLowerCase() + '|' + l.target.toLowerCase()
			);
			return bump ? { ...l, traversal_count: (l.traversal_count ?? 0) + bump } : l;
		});
	});
	let allLibraryTags = $state<Record<string, number>>({});
	let allNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let allIndexEntries = $state<IndexEntry[]>([]);
	// Index panel lazy-load state. The index is built by a filesystem walk in
	// `scan_library_index` (libraries.rs), so it is NOT on the boot path (per
	// commit 039ac66 "kill ALL filesystem walkers from boot"). Instead we
	// scan lazily when the user opens the panel for the first time. The key
	// combines universe + library count so adding/removing libraries also
	// triggers a re-scan on next panel open. Reset to null on universe switch.
	let indexLoading = $state(false);
	let indexLoadedKey = $state<string | null>(null);
	// Star data uses $state.raw — tracks reassignment (required for the main Sky View
	// <GraphMindView nodes={skyNodes}> binding and the Lens <ConstellationSight> binding
	// to re-run when refreshLibraryCaches populates these arrays after mount) but
	// skips the per-element proxy wrap, preserving iteration perf on 10k+ arrays.
	// Plain `let` is non-reactive in Svelte 5 runes — confirmed by Sky View boot
	// investigation, see docs/LESSONS-LEARNED.md LL-017.
	let skyNodes = $state.raw<SkyNode[]>([]);
	let skyLinks = $state.raw<SkyLink[]>([]);

	// Constellation Lens state
	let lensActive = $state(false);
	let lensLoading = $state(false);
	let lensCentrality = $state<Map<string, number>>(new Map());
	let lensCommunities = $state<ClusterInfo[]>([]);
	let lensCommunityAssignments = $state<Map<string, number>>(new Map());
	let lensGaps = $state<StructuralGap[]>([]);
	let lensHealth = $state<UniverseHealth | null>(null);
	let lensBridges = $state<{ id: string; name: string; centrality: number }[]>([]);
	let lensShowTagEdges = $state(false);
	let lensPeelCount = $state(0);
	let lensTagEdges = $state<{ source: string; target: string; shared_tags: string[]; weight: number }[]>([]);
	let lensCommunityProfiles = $state<CommunityProfile[]>([]);
	let lensContradictions = $state<[string, string][]>([]);
	/** WTD cache flag: true = lens data is stale and must be recomputed on next open.
	 *  Starts true (no data yet). Flipped to false after a successful computation,
	 *  back to true whenever skyVersion increments (graph topology changed). */
	let lensDataStale = $state(true);
	let skyVersion = $state(0);
	// Boot Criterion 2: `graphReady` flips to true once the deferred link+tag
	// payload (Phase 2 of refreshLibraryCaches) lands. Views that render
	// degraded state while the graph is still loading (Sky View shell, tag
	// browser) can read this flag to flip to the full UI when it arrives.
	let graphReady = $state(false);
	let searchLinkCounts = $state(new Map<string, { incoming: number }>());
	let maturityMap = $state(new Map<string, string>()); // path → maturity state (CE Phase 3)
	let stageMap = $state(new Map<string, string>()); // path → stage (CE Phase 6)
	// Star data is passed to SkyView as plain arrays.
	// We avoid $state/$derived for large arrays (1885+ nodes) because Svelte 5 proxies
	// make iteration extremely slow. Instead, skyVersion ($state) triggers re-render
	// and SkyView reads the plain skyNodes/skyLinks directly.

	// WiW filtered data — recomputed when selection or star data changes
	// Uses skyVersion as reactive trigger since skyNodes/skyLinks are plain arrays
	const wiwFilteredNodes = $derived.by(() => {
		const _ver = skyVersion; // reactive trigger
		if (!skyViewSelectedPath || !showSkyView) return [];
		const paths = Array.isArray(skyViewSelectedPath) ? skyViewSelectedPath : [skyViewSelectedPath];
		const norms = paths.map(p => p.replace(/\\/g, '/').toLowerCase());
		return skyNodes.filter(n => {
			const np = n.path.replace(/\\/g, '/').toLowerCase();
			return norms.some(norm => np.startsWith(norm + '/') || np === norm);
		});
	});
	const wiwFilteredNodeIds = $derived(new Set(wiwFilteredNodes.map(n => n.id)));
	const wiwFilteredLinks = $derived.by(() => {
		const _ver = skyVersion;
		if (wiwFilteredNodes.length === 0) return [];
		return skyLinks.filter(l => wiwFilteredNodeIds.has(l.source) && wiwFilteredNodeIds.has(l.target));
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
		const shouldShow = showSkyView && wiwEnabled && skyViewSelectedPath && wiwFilteredNodes.length > 0;
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

	// ── Flanking column resize (Tier 1b) ────────────────────────────────────
	// Initialize widths from persisted settings on first load; only run once
	// so user drags aren't cancelled by subsequent appSettings reactions.
	let flankWidthsLoaded = false;
	$effect(() => {
		const lw = $appSettings.leftOfNoteWidth;
		const rw = $appSettings.rightOfNoteWidth;
		if (!flankWidthsLoaded && (lw ?? 0) > 0) {
			flankWidthsLoaded = true;
			leftFlankWidth = lw ?? 280;
			rightFlankWidth = rw ?? 280;
		}
	});

	let flankResizing = $state<'left' | 'right' | null>(null);

	function startFlankResize(side: 'left' | 'right', e: MouseEvent) {
		e.preventDefault();
		flankResizing = side;
		const startX = e.clientX;
		const startWidth = side === 'left' ? leftFlankWidth : rightFlankWidth;
		const isRtl = $dir === 'rtl';

		function onMouseMove(ev: MouseEvent) {
			const delta = ev.clientX - startX;
			if (side === 'left') {
				// Left flank: grows rightward in LTR, leftward in RTL
				leftFlankWidth = Math.max(180, Math.min(500, startWidth + (isRtl ? -delta : delta)));
			} else {
				// Right flank: grows leftward in LTR, rightward in RTL
				rightFlankWidth = Math.max(180, Math.min(500, startWidth + (isRtl ? delta : -delta)));
			}
		}

		function onMouseUp() {
			flankResizing = null;
			document.removeEventListener('mousemove', onMouseMove);
			document.removeEventListener('mouseup', onMouseUp);
			// Persist to appSettings on drag end
			updateSettings({ leftOfNoteWidth: leftFlankWidth, rightOfNoteWidth: rightFlankWidth });
		}

		document.addEventListener('mousemove', onMouseMove);
		document.addEventListener('mouseup', onMouseUp);
	}

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
				rightSidebarWidth = Math.max(320, Math.min(600, startWidth + (isRtl ? delta : -delta)));
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
	const isDashboardVisible = $derived(isHome && !$activeTab && $libraries.length > 0 && $appSettings.showDashboard);
	/** True when any full-page function is active — disables sidebars and split pane */
	const fullPageActive = $derived(showSkyView || showGlobalTasks || showIndex || showExpressionForge || showSenseMakingCanvas || showConstellationMap || showOrgChart || showKnowledgeHealth || lensActive || showSearchHub || showInspector360 || isDashboardVisible);

	// Shared disable/title logic for the three layout-bar buttons (left sidebar,
	// split-view, right sidebar). Any overlay mode that takes over the editor
	// area — full-page view (OrgChart/Lens/Search Hub) or SV inspect mode —
	// disables all three. Adding a fourth overlay mode touches only this pair,
	// not three title ternaries.
	const layoutCtrlDisabled = $derived(fullPageActive || skyViewInspectMode);
	const layoutCtrlDisabledReason = $derived(
		fullPageActive
			? $t('layout.disabledFullPage') || 'Disabled in full-page view'
			: skyViewInspectMode
				? $t('layout.disabledInSkyInspect') || 'Disabled while inspecting a Sky View note'
				: ''
	);

	// Auto-collapse sidebars when full-page becomes active, restore when deactivated
	// Sidebar snapshot for full-page view: managed via pushSidebars/popSidebars('fullPage')
	let fullPageWasActive = false;
	$effect(() => {
		if (fullPageActive && !fullPageWasActive) {
			pushSidebars('fullPage');
		} else if (!fullPageActive && fullPageWasActive) {
			popSidebars('fullPage');
		}
		fullPageWasActive = fullPageActive;
	});

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

	// Library color palette (shared utility)
	import { buildLibraryColorMap } from '$lib/libraries/colors';
	const libraryColorMap = $derived(buildLibraryColorMap($libraries));

	// Sidebar data: derived from focused tab (whichever pane has focus)
	const sidebarTab = $derived($focusedTab);

	// CE Phase 12 — fetch Note360View when Inspector 360 is visible.
	// Read through $derived string values so identity-change of sidebarTab
	// on tab-content updates doesn't re-fire the IPC effect. Debounce
	// 200 ms; sequence number discards stale results when fetches overlap;
	// last-key guard skips re-fetching the same note across tab toggles.
	const inspector360Path = $derived(sidebarTab?.path ?? '');
	const inspector360LibPath = $derived(sidebarTab?.libraryPath ?? '');
	$effect(() => {
		const shouldFetch = (rightSidebarOpen && rightSidebarTab === 'inspector360') || showInspector360;
		const path = inspector360Path;
		const libPath = inspector360LibPath;
		if (!shouldFetch || !path || !libPath) {
			if (!shouldFetch && inspector360Data !== null) inspector360Data = null;
			return;
		}
		const key = `${libPath}::${path}`;
		if (key === lastFetchedInspectorKey && inspector360Data) return;
		const seq = ++inspector360RequestSeq;
		if (inspector360FetchTimer) clearTimeout(inspector360FetchTimer);
		inspector360FetchTimer = setTimeout(async () => {
			try {
				const data = await invoke('get_360_view', { libraryPath: libPath, notePath: path });
				if (seq === inspector360RequestSeq) {
					inspector360Data = data;
					lastFetchedInspectorKey = key;
				}
			} catch (e) {
				console.error('Inspector 360 fetch failed:', e);
				if (seq === inspector360RequestSeq) inspector360Data = null;
			}
		}, 200);
		return () => {
			if (inspector360FetchTimer) clearTimeout(inspector360FetchTimer);
		};
	});
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
	let currentBacklinks = $state<{ name: string; path: string; context: string; libraryName: string; linkType?: string; linkTypes?: string[]; traversalCount?: number }[]>([]);
	let currentOutgoing = $state<{ target: string; context: string; traversalCount?: number; linkType?: string; linkTypes?: string[] }[]>([]);
	let activeNoteTags = $state<string[]>([]);
	let _sidebarDebounce: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		const body = sidebarBody;
		const tab = sidebarTab;
		const props = sidebarProperties;
		const dirFallback = $dir as 'ltr' | 'rtl';
		// Track allLibraryLinks as a dep so this effect re-runs when the
		// deferred graph payload (Phase 2 of refreshLibraryCaches) lands —
		// otherwise a tab focused BEFORE the graph arrives would show an
		// empty backlinks/outgoing panel and never auto-refresh. Reading
		// `.length` at top level is enough to establish the dependency.
		// Also track linkTraversalBumps so the `×N` chips in the sidebar
		// refresh live when the user follows a wikilink (P4.2 follow-up).
		void allLibraryLinks.length;
		void $linkTraversalBumps.size;
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
			// P5 slice 2: snapshot the decay config once so both sort helpers
			// get the same `nowMs` (no skew between backlinks and outgoing).
			const lifecycle = $appSettings.linkLifecycle;
			const decayCfg = {
				nowMs: Date.now(),
				halfLifeDays: lifecycle?.halfLifeDays ?? 60,
				decayEnabled: lifecycle?.decayEnabled ?? true,
			};
			// Backlinks (MIG-004 §9: alias-aware via notePathToAliases lookup).
			const aliasesForActive = notePathToAliases.get(tab.path) ?? [];
			currentBacklinks = getBacklinks(effectiveLibraryLinks, tab.name, decayCfg, aliasesForActive);
			// CE Phase 5: Provenance fetched on tab click only (not here — no IPC on typing path)
			// Outgoing links
			currentOutgoing = getOutgoingLinks(effectiveLibraryLinks, tab.path, decayCfg);
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
	let localSkyNodes = $state<SkyNode[]>([]);
	let localSkyLinks = $state<SkyLink[]>([]);
	let _localStarTimer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		// Track reactive dependencies (skyVersion signals when plain arrays change)
		const isVisible = rightSidebarOpen && rightSidebarTab === 'star';
		const tab = sidebarTab;
		const _ver = skyVersion; // reactive trigger for non-reactive skyNodes/skyLinks

		clearTimeout(_localStarTimer);

		if (!isVisible || !tab) {
			localSkyNodes = [];
			localSkyLinks = [];
			return;
		}

		// Defer computation to avoid blocking UI
		_localStarTimer = setTimeout(() => {
			const activeId = tab.name.replace(/\.md$/, '').toLowerCase();
			const connectedIds = new Set<string>();
			connectedIds.add(activeId);
			for (const link of skyLinks) {
				if (link.source === activeId || link.target === activeId) {
					connectedIds.add(link.source);
					connectedIds.add(link.target);
				}
			}
			localSkyNodes = skyNodes.filter(n => connectedIds.has(n.id));
			localSkyLinks = skyLinks.filter(l => connectedIds.has(l.source) && connectedIds.has(l.target));
		}, 50);
	});

	// WTD: Invalidate cached Constellation Lens data whenever the sky graph changes.
	// skyVersion increments each time skyNodes/skyLinks are rebuilt (on library load,
	// note save, link scan). If lens data was previously computed, mark it stale so
	// the next toggleLens() call triggers a fresh computation instead of showing
	// outdated graph analytics.
	$effect(() => {
		const ver = skyVersion; // reactive trigger
		if (ver > 0 && lensHealth !== null) {
			// Graph changed after a successful computation — mark data stale.
			lensDataStale = true;
		}
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

	// Safety: if the active right sidebar tab's panel has been moved away from
	// right-sidebar (via Settings → Panels), switch to the first tab that IS
	// still in the sidebar. Runs reactively whenever panelPlacements changes.
	// Falls back to 'right-sidebar' when no placement is saved (new install /
	// first-time user), so all tabs remain visible by default.
	$effect(() => {
		const p = $appSettings.panelPlacements;
		const inSidebar = (id: PanelId): boolean =>
			(p?.[id] ?? 'right-sidebar') === 'right-sidebar';

		const tabVisible: Record<string, boolean> = {
			properties: inSidebar('properties'),
			// Backlinks tab is unconditionally rendered in the sidebar as an
			// alternative access path to the backlinks/outgoing panels, even
			// when they're placed in the flanking slots. Force visible so the
			// safety reset below doesn't steal the user's click.
			backlinks:  true,
			tags:       inSidebar('tags'),
			star:       inSidebar('sky'),
			tasks:      inSidebar('tasks'),
			calendar:   inSidebar('calendar'),
			health:     inSidebar('health'),
			provenance: inSidebar('provenance'),
			review:     inSidebar('review'),
			links:      inSidebar('links'),
		};

		if (!tabVisible[rightSidebarTab]) {
			const order = ['properties', 'backlinks', 'tags', 'star', 'tasks', 'calendar', 'health', 'provenance', 'review', 'links'] as const;
			const first = order.find(tab => tabVisible[tab]);
			rightSidebarTab = (first ?? 'properties') as typeof rightSidebarTab;
		}
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

	// One-time cleanup: earlier versions stored core blocks on custom themes and
	// wrote hard-coded per-tier File-Explorer defaults into styleSettingsValues.
	// Both defeat the current cascade. Runs exactly once per session.
	let _coreBlockCleanupDone = false;
	// Any per-tier File-Explorer value stored from earlier sessions would
	// defeat the new master cascade (CSS var fallbacks can't distinguish
	// "explicitly set" from "not set"). Clear them all on startup. Master
	// ids (ft-master-*) and non-file-explorer ids are preserved.
	const LEGACY_FT_TIER_IDS = new Set([
		'ft-universe-font-size', 'ft-universe-weight', 'ft-universe-color',
		'ft-cuniverse-font-size', 'ft-cuniverse-weight', 'ft-cuniverse-color',
		'ft-library-font-size', 'ft-library-weight', 'ft-library-color',
		'ft-font-size', 'ft-folder-weight', 'ft-folder-color',
		'ft-file-weight', 'ft-file-color', 'ft-row-padding-y',
	]);
	$effect(() => {
		if (_coreBlockCleanupDone) return;
		const customs = $appSettings.customThemes;
		if (!customs || customs.length === 0) { _coreBlockCleanupDone = true; return; }
		let changed = false;
		const cleaned = customs.map(ct => {
			let next = ct;
			// (a) strip stored core blocks
			if (ct.styleSettingsBlocks && ct.styleSettingsBlocks.length > 0) {
				const filtered = ct.styleSettingsBlocks.filter(b => !CORE_BLOCK_IDS.has(b.id));
				if (filtered.length !== ct.styleSettingsBlocks.length) {
					changed = true;
					next = { ...next, styleSettingsBlocks: filtered };
				}
			}
			// (b) clear legacy per-tier File-Explorer overrides so master cascades
			if (ct.styleSettingsValues) {
				const values = { ...ct.styleSettingsValues };
				let valuesChanged = false;
				for (const id of Object.keys(values)) {
					if (LEGACY_FT_TIER_IDS.has(id)) { delete values[id]; valuesChanged = true; }
				}
				if (valuesChanged) {
					changed = true;
					next = { ...next, styleSettingsValues: values };
				}
			}
			return next;
		});
		if (changed) updateSettings({ customThemes: cleaned });
		_coreBlockCleanupDone = true;
	});

	// Track which Style-Settings CSS vars the last theme-apply wrote, so
	// stale overrides are cleared when the user resets a row (empty values
	// are skipped by the generator, so the property would otherwise persist
	// forever).
	let _lastStyleSettingsKeys: string[] = [];

	// Apply custom theme colors (responds to both activeThemeId and colorScheme changes)
	$effect(() => {
		if (typeof document === 'undefined') return;
		const s = $appSettings;
		let themeId = s.activeThemeId;

		// Auto-pair: if the active theme has a counterpart for the current scheme, switch to it
		if (themeId) {
			const resolved = colorScheme === 'system'
				? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
				: colorScheme;
			const allThemes = [...BUILTIN_THEMES, ...(s.customThemes ?? [])];
			const current = allThemes.find(t => t.id === themeId);
			if (current && current.type !== resolved && current.pairedThemeId) {
				const paired = allThemes.find(t => t.id === current.pairedThemeId);
				if (paired) themeId = paired.id;
			}
		}

		if (!themeId) {
			// No custom theme — but still apply accent color if set
			if (s.accentColor && s.accentColor !== '#7c3aed') {
				const hsl = hexToHSL(s.accentColor);
				document.body.style.setProperty('--accent-h', String(hsl.h));
				document.body.style.setProperty('--accent-s', `${hsl.s}%`);
				document.body.style.setProperty('--accent-l', `${hsl.l}%`);
			}
			return;
		}

		// Find theme — customs take precedence over built-ins so a user's
		// auto-cloned copy of a built-in theme (same id, now with their
		// styleSettingsValues attached) is the one that gets applied.
		const theme = s.customThemes?.find(t => t.id === themeId) || BUILTIN_THEMES.find(t => t.id === themeId);
		if (!theme) return;

		// Apply derived CSS variables
		const vars = deriveThemeVariables(theme.colors, theme.type);
		const root = document.body.style;
		for (const [key, value] of Object.entries(vars)) {
			root.setProperty(key, value);
		}

		// Apply theme type class
		document.body.classList.remove('theme-light', 'theme-dark');
		document.body.classList.add(`theme-${theme.type}`);

		// Apply custom CSS if present
		let customStyleEl = document.getElementById('constellation-custom-theme-css');
		if (theme.customCSS) {
			if (!customStyleEl) {
				customStyleEl = document.createElement('style');
				customStyleEl.id = 'constellation-custom-theme-css';
				document.head.appendChild(customStyleEl);
			}
			customStyleEl.textContent = theme.customCSS;
		} else if (customStyleEl) {
			customStyleEl.remove();
		}

		// Apply Style Settings: core blocks + theme's own blocks (dedup guarded).
		const ssBlocks = getEffectiveStyleBlocks(theme);
		if (ssBlocks.length > 0) {
			const ssResult = generateStyleSettingsCSS(
				ssBlocks,
				theme.styleSettingsValues ?? {},
				theme.type
			);
			// Clear any Style-Settings var from the previous apply that is NOT
			// in the new set (user reset it, or cleared a tier override).
			const newKeys = new Set(Object.keys(ssResult.variables));
			for (const prevKey of _lastStyleSettingsKeys) {
				if (!newKeys.has(prevKey)) root.removeProperty(prevKey);
			}
			_lastStyleSettingsKeys = [...newKeys];
			// Apply CSS variables with proper format units
			for (const [key, value] of Object.entries(ssResult.variables)) {
				root.setProperty(key, value);
			}
			// Apply body classes from class-toggle and class-select
			// Remove old style settings classes first
			document.body.className = document.body.className
				.split(' ')
				.filter(c => !c.startsWith('css-settings-'))
				.join(' ');
			for (const cls of ssResult.classes) {
				document.body.classList.add(cls);
			}
		} else if (theme.styleSettingsValues) {
			// Fallback: simple key→value application
			for (const [id, value] of Object.entries(theme.styleSettingsValues)) {
				root.setProperty(`--${id}`, value);
			}
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
			{ id: 'search', name: $t('commands.searchLibrary'), shortcut: sc('search'), icon: '🔎', action: () => { showSearchHub = true; searchHubInitialQuery = ''; showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; lensActive = false; showInspector360 = false; /* fullPageActive $effect handles sidebar snapshot */ }, category: 'Navigation' },
			{ id: 'daily-note', name: $t('commands.dailyNote'), shortcut: sc('daily-note'), icon: '📅', action: handleOpenDailyNote, category: 'Daily Notes' },
			{ id: 'toggle-edit', name: $t('commands.toggleEdit'), shortcut: sc('toggle-edit'), icon: '✏️', action: () => { const tab = get(focusedTab); if (tab) toggleEditMode(tab.id); }, category: 'Editor' },
			{ id: 'star-view', name: $t('commands.skyView'), shortcut: sc('star-view'), icon: '🕸️', action: () => { showSkyView = !showSkyView; showConstellationMap = false; }, category: 'View' },
			{ id: 'global-tasks', name: $t('commands.globalTasks'), shortcut: sc('global-tasks'), icon: '☑️', action: () => { showGlobalTasks = !showGlobalTasks; showSkyView = false; showConstellationMap = false; showInspector360 = false; }, category: 'View' },
			{ id: 'insert-template', name: $t('commands.insertTemplate'), shortcut: sc('insert-template'), icon: '📋', action: () => { templatePickerMode = 'insert'; refreshTemplates(); showTemplatePicker = true; }, category: 'Templates' },
			{ id: 'toggle-bold', name: $t('commands.toggleBold'), shortcut: sc('toggle-bold'), icon: '𝐁', action: () => {}, category: 'Editor' },
			{ id: 'toggle-italic', name: $t('commands.toggleItalic'), shortcut: sc('toggle-italic'), icon: '𝐼', action: () => {}, category: 'Editor' },
			{ id: 'split-view', name: $t('commands.splitView'), shortcut: sc('split-view'), icon: '⊞', action: cycleSplit, category: 'View' },
			{ id: 'close-note', name: $t('commands.closeNote'), shortcut: sc('close-note'), icon: '✕', action: closeNote, category: 'File' },
			{ id: 'toggle-left', name: $t('commands.toggleLeftSidebar'), shortcut: sc('toggle-left'), icon: '◧', action: () => { if (!fullPageActive && !skyViewInspectMode) sidebarOpen = !sidebarOpen; }, category: 'View' },
			{ id: 'toggle-right', name: $t('commands.toggleRightSidebar'), shortcut: sc('toggle-right'), icon: '◨', action: () => { if (!fullPageActive && !skyViewInspectMode) rightSidebarOpen = !rightSidebarOpen; }, category: 'View' },
			{ id: 'add-library', name: $t('commands.addLibrary'), shortcut: sc('add-library'), icon: '📁', action: handleAddLibrary, category: 'Library' },
			{ id: 'new-library', name: $t('commands.newLibrary'), icon: '📚', action: handleNewLibrary, category: 'Library' },
			{ id: 'toggle-bookmark', name: $t('commands.toggleBookmark'), shortcut: sc('toggle-bookmark'), icon: '⭐', action: handleToggleBookmark, category: 'Bookmarks' },
			{ id: 'random-note', name: $t('commands.randomNote'), shortcut: sc('random-note'), icon: '🎲', action: handleRandomNote, category: 'Navigation' },
			{ id: 'toggle-theme', name: $t('commands.toggleTheme'), shortcut: sc('toggle-theme'), icon: '🌗', action: handleToggleTheme, category: 'Appearance' },
			...(hasMultipleDisplays ? [
				{ id: 'second-screen', name: $t('secondScreen.title'), shortcut: sc('second-screen'), icon: '🖥️', action: handleToggleSecondScreen, category: 'View' },
				{ id: 'send-to-screen', name: $t('secondScreen.sendToScreen'), shortcut: sc('send-to-screen'), icon: '📤', action: handleSendToSecondScreen, category: 'View' },
			] : []),
			{ id: 'nav-back', name: $t('commands.navBack'), shortcut: sc('nav-back'), icon: '←', action: navigateBack, category: 'Navigation' },
			{ id: 'nav-forward', name: $t('commands.navForward'), shortcut: sc('nav-forward'), icon: '→', action: navigateForward, category: 'Navigation' },
			{ id: 'workspaces', name: $t('commands.workspaces'), shortcut: sc('workspaces'), icon: '🗂️', action: () => { showCommandPalette = false; showWorkspaces = true; }, category: 'View' },
			{ id: 'index', name: $t('commands.index'), shortcut: sc('index'), icon: '📖', action: () => { showCommandPalette = false; showIndex = !showIndex; showSkyView = false; showGlobalTasks = false; showConstellationMap = false; showInspector360 = false; indexReturnPending = false; }, category: 'Navigation' },
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
			{ id: 'expression-forge', name: $t('commands.expressionForge') || 'Expression Forge', icon: '✨', action: () => { showCommandPalette = false; showExpressionForge = !showExpressionForge; showSkyView = false; showGlobalTasks = false; showIndex = false; showSenseMakingCanvas = false; showConstellationMap = false; showInspector360 = false; }, category: 'View' },
			...($appSettings.enabledFeatures?.constellationMap === true ? [{ id: 'constellation-map', name: $t('commands.constellationMap') || 'Constellation Map', icon: '🗺️', action: () => { showCommandPalette = false; showConstellationMap = !showConstellationMap; showSkyView = false; showGlobalTasks = false; showIndex = false; showExpressionForge = false; showSenseMakingCanvas = false; showInspector360 = false; mapReturnPending = false; }, category: 'View' }] : []),
			{ id: 'sense-making-canvas', name: $t('commands.senseMakingCanvas') || 'Sense-Making Canvas', icon: '🎨', action: () => { showCommandPalette = false; showSenseMakingCanvas = !showSenseMakingCanvas; showSkyView = false; showGlobalTasks = false; showIndex = false; showExpressionForge = false; showConstellationMap = false; showInspector360 = false; }, category: 'View' },
			{ id: 'knowledge-health', name: 'Knowledge Health', icon: '🧠', action: () => { showCommandPalette = false; showKnowledgeHealth = true; }, category: 'View' },
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
		// ═══ BOOT ARCHITECTURE — paint-first, single-bundle IPC ════════════
		// Per 2026-04-15 expert panel + LL-015 (dev-mode per-IPC overhead is
		// ~37s on Windows): the frontend now makes ONE IPC call at boot that
		// returns the full bundle of config + libraries + child universes,
		// rather than ~10 serialized calls. This collapses boot-IPC latency
		// from 10× the per-call overhead to 1×.
		//
		// appReady still flips synchronously at the top so the UI shell
		// paints instantly. The bundle call populates all reactive stores in
		// one shot when it returns.
		performance.mark('boot:paint');
		appReady = true;

		// ── Round 5 follow-up diagnostic: JS-event-loop heartbeat ──
		// Samples the event loop every 100 ms. If the JS thread is blocked
		// during the core-snapshot queue window, `bootHeartbeatMaxGapMs`
		// will be large; if the event loop stays responsive, it stays small.
		// Value is dumped into `buildBootPerfReport()` alongside the other
		// per-phase timings.
		bootHeartbeatLastFire = performance.now();
		bootHeartbeatMaxGapMs = 0;
		bootHeartbeatInterval = setInterval(() => {
			const now = performance.now();
			const gap = now - bootHeartbeatLastFire;
			if (gap > bootHeartbeatMaxGapMs) bootHeartbeatMaxGapMs = gap;
			bootHeartbeatLastFire = now;
		}, 100);
		cleanupFns.push(() => {
			if (bootHeartbeatInterval !== undefined) {
				clearInterval(bootHeartbeatInterval);
				bootHeartbeatInterval = undefined;
			}
		});

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

		// One IPC call returns everything: libraries, settings, bookmarks,
		// workspaces, property types, workspace bases, child universes.
		type BootBundle = {
			libraries: any[];
			settings: Record<string, unknown>;
			bookmarks: unknown[];
			workspaces: unknown[];
			property_types: Record<string, unknown>;
			workspace_bases: any[];
			child_universes: ChildUniverseInfo[];
			child_universe_lib_paths: Record<string, string[]>;
			/** Per-step wall-clock timings measured inside the Rust command.
			 *  Attributed into `boot-perf.latest.json#boot_bundle_timings`
			 *  so cold-boot bottlenecks are diagnosable without rebuilds. */
			timings_ms?: Array<[string, number]>;
		};
		let bundle: BootBundle | null = null;
		try {
			bundle = await invoke<BootBundle>('constellation_boot_bundle');
		} catch (e) {
			console.warn('[boot] boot bundle failed, falling back to per-call loads', e);
		}

		if (bundle) {
			// Populate every store directly from the bundle response. No
			// additional IPCs — the whole point of collapsing into one call.
			// Each store-setter mirrors what the individual loader functions
			// do in $lib/libraries/store.ts + propertyTypeRegistry.ts.
			libraries.set(bundle.libraries);

			// Settings — merge with DEFAULT_SETTINGS (same logic as loadSettings).
			try {
				const parsed = bundle.settings as any;
				if (parsed && Object.keys(parsed).length > 0) {
					const savedSkyView = parsed.skyView || {};
					if (savedSkyView.nodeSize === 4) savedSkyView.nodeSize = 1.5;
					appSettings.set({
						...DEFAULT_SETTINGS,
						...parsed,
						skyView: { ...DEFAULT_SETTINGS.skyView, ...savedSkyView },
						security: { ...DEFAULT_SETTINGS.security, ...(parsed.security || {}) },
						enabledFeatures: { ...DEFAULT_SETTINGS.enabledFeatures, ...(parsed.enabledFeatures ?? parsed.enabledPlugins ?? {}) },
						customShortcuts: { ...(parsed.customShortcuts || {}) },
					});
				}
			} catch { /* settings schema mismatch — fall through with defaults */ }

			// Bookmarks / Workspaces — arrays set directly.
			if (Array.isArray(bundle.bookmarks) && bundle.bookmarks.length > 0) {
				bookmarks.set(bundle.bookmarks as any);
			}
			if (Array.isArray(bundle.workspaces) && bundle.workspaces.length > 0) {
				workspaces.set(bundle.workspaces as any);
			}

			// Property types — seed the registry cache (avoids a separate IPC).
			try {
				const reg = await import('$lib/libraries/propertyTypeRegistry');
				reg.seedFromBundle(bundle.property_types);
			} catch { /* on-demand load on first use */ }

			workspaceBases = bundle.workspace_bases;
			childUniverses = bundle.child_universes;
			const map = new Map<string, Set<string>>();
			for (const cu of bundle.child_universes) {
				map.set(cu.path, new Set(bundle.child_universe_lib_paths[cu.path] ?? []));
			}
			childUniverseLibPaths = map;

			// Stash Rust-side per-step timings so buildBootPerfReport can write
			// them into boot-perf.latest.json. Investigation target per
			// lab/boot-perf/boot-bundle-cold-start.md.
			if (Array.isArray(bundle.timings_ms)) {
				bootBundleTimings = bundle.timings_ms;
			}
		} else {
			// Fallback: old per-IPC pattern. Only runs if the bundle command
			// is unavailable (shouldn't happen post-dc46683 but defensive).
			await Promise.all([
				loadSettings().catch(() => {}),
				loadBookmarks().catch(() => {}),
				loadWorkspaces().catch(() => {}),
				loadPropertyTypes().catch(() => {}),
				listWorkspaceBases().then(b => workspaceBases = b).catch(() => {}),
				getChildUniverses().then(async (c) => {
					childUniverses = c;
					const m = new Map<string, Set<string>>();
					for (const cu of c) {
						try {
							const childLibs = await invoke<{ id: string; name: string; path: string }[]>(
								'read_child_universe_libraries', { childPath: cu.path }
							);
							m.set(cu.path, new Set(childLibs.map(l => l.path.replace(/\\/g, '/').toLowerCase())));
						} catch {
							m.set(cu.path, new Set());
						}
					}
					childUniverseLibPaths = m;
				}).catch(() => {}),
				loadLibraries().catch(() => {}),
			]);
		}

		librariesLoaded = true;
		performance.mark('boot:libraries-loaded');

		// ═══ BOOT RULE: ZERO FILESYSTEM WALKS ════════════════════════════
		// Per the 2026-04-15 expert panel, nothing walks the filesystem on
		// boot. Everything below is fire-and-forget — the UI never waits.
		//
		// Removed from the boot path in this rewrite:
		//   - cache_reconcile() — full-filesystem mtime walk. Now only
		//     triggered by the file watcher (per-file) or by the user
		//     clicking Settings → Rebuild Index.
		//   - enrichNodesBackground() — 4 per-library walks for strata /
		//     maturity / origins / stages. Will be persisted into SQLite
		//     at index time and read from cache in a future commit.
		//
		// loadAllStats remains because its Rust side is already cache-
		// fast (metadata-only walk + per-library thread parallelism).
		// It's fire-and-forget so the sidebar star counts populate
		// without blocking anything.
		// ═══ BOOT ORDER: hydrate first, fan-out after ═══════════════════
		// Critical finding (2026-04-16): all 34 boot IPCs (16 watchers +
		// 16 appearances + stats + snapshot) were racing into Tauri's
		// command queue in the same tick. On Windows/NTFS the I/O
		// scheduler round-robined them, so `cache_boot_snapshot_core` —
		// the ONE thing that gates `boot:hydrated` — took 27 s wall-clock
		// despite only 8 s of Rust execution time. Everything else
		// finished at the same ~27 s endpoint.
		//
		// Fix: await the ship-gate IPC first. The core snapshot owns the
		// I/O queue until `boot:hydrated` fires; only THEN do watchers,
		// appearances, and stats fan out. They don't gate anything the
		// user can see, so deferring them is free.
		// See lab/boot-perf/boot-bundle-cold-start.md.
		await refreshLibraryCaches().catch(() => {});

		// Post-hydration fan-out — populate sidebar badges and enable
		// the file watcher. Fire-and-forget; the UI is already live.
		{
			const t0 = performance.now();
			Promise.all($libraries.map(lib =>
				startWatchingLibrary(lib.id, lib.path).catch(() => {})
			)).then(() => { startWatchingAllWallMs = Math.round(performance.now() - t0); })
			  .catch(() => { startWatchingAllWallMs = Math.round(performance.now() - t0); });
		}
		{
			const t0 = performance.now();
			Promise.all($libraries.map(lib =>
				loadLibraryAppearance(lib.path, lib.id).catch(() => {})
			)).then(() => { loadAllAppearancesWallMs = Math.round(performance.now() - t0); })
			  .catch(() => { loadAllAppearancesWallMs = Math.round(performance.now() - t0); });
		}
		{
			const t0 = performance.now();
			loadAllStats()
				.then(() => { loadAllStatsWallMs = Math.round(performance.now() - t0); })
				.catch(() => { loadAllStatsWallMs = Math.round(performance.now() - t0); });
		}
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
		librariesLoaded = false;
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
		clearLinkTraversalBumps();
		allIndexEntries = [];
		indexLoadedKey = null;
		libraryTrees = {};
		expandedLibraries = new Set();
		editingTabIds.set(new Set());
		libraryAppearances.set({});
		bookmarks.set([]);

		// Force Map + OrgChart to re-mount on next open for the new Universe.
		// Their IPC (constellation_map_universe) fires only from onMount, so without
		// this reset the user would see stale data from the prior Universe.
		mapEverOpened = false;
		orgChartEverOpened = false;
		inspector360EverOpened = false;
		inspector360Data = null;
		mapFocusNode = null;

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
		document.addEventListener('constellation:show-importer', () => { showImporter = true; });

		// Universal Embed: "open this note" (from transclusion header click)
		window.addEventListener('constellation:open-note', (e: Event) => {
			const detail = (e as CustomEvent).detail as { path?: string };
			if (!detail?.path) return;
			const libs = get(libraries);
			const lib = libs.find(l => detail.path!.startsWith(l.path));
			if (lib) openNoteTab(detail.path!, lib.name, libraryColorMap[lib.name] || '#7c3aed');
		});
		// Universal Embed: "open this file externally" (from generic file card)
		window.addEventListener('constellation:open-external', (e: Event) => {
			const detail = (e as CustomEvent).detail as { path?: string };
			if (!detail?.path) return;
			invoke('constellation_show_in_folder', { path: detail.path }).catch(() => {});
		});

		// Safety net: de-canonicalize any external library that earlier builds
		// may have renamed en-masse on import. Runs at most ONCE per install —
		// after the first successful sweep, we mark it done in localStorage and
		// skip on subsequent boots (the import path that created canonical
		// filenames has been removed, so they cannot reappear).
		//
		// This matters at scale: with a 7,600-note Universe across 16 libraries
		// the repair walks ~12,000 files via IPC on every launch, which bogged
		// down startup by several seconds even when every library was clean.
		// Canonical repair: fire-and-forget, gated by a one-shot localStorage
		// flag. The Rust side walks every library's filesystem checking for
		// canonical-format filenames to revert; the flag ensures this only
		// runs once per install, then never again. Paint time is unaffected
		// because the call doesn't block.
		try {
			if (localStorage.getItem('constellation:canonical-repair-done') !== '1') {
				invoke<string[]>('repair_external_libraries_on_startup').then((repaired) => {
					if (repaired.length > 0) {
						console.log('[Constellation] Restored original filenames in libraries:', repaired);
					}
					localStorage.setItem('constellation:canonical-repair-done', '1');
				}).catch(err => console.error('[Constellation] Startup repair failed:', err));
			}
		} catch { /* localStorage unavailable */ }

		// Living Link P3: run weight decay job once per 24h (fire-and-forget,
		// gated by localStorage timestamp). Operates on the SQLite
		// note_links table; does not block any IPC.
		try {
			const lastDecay = Number(localStorage.getItem('constellation:last-link-decay') ?? '0');
			if (Date.now() - lastDecay > 86_400_000) {
				const { linkDecay } = await import('$lib/libraries/store');
				linkDecay().then(() => localStorage.setItem('constellation:last-link-decay', String(Date.now())))
					.catch(() => { /* index not ready yet — try again next launch */ });
			}
		} catch { /* localStorage unavailable */ }

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

		// initializeApp paints the shell (appReady=true) at its very first
		// step, then loads data in the background. We `await` its full
		// completion here so later onMount steps (watcher setup, etc.)
		// run with a populated libraries store.
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

		// Detect multiple monitors — gate SS features
		hasMultipleDisplays = await hasMultipleMonitors().catch(() => false);

		// Canonical system: no startup rename. Canonicalization happens only when
		// the user explicitly links/imports with "Adopt Constellation Format".
		// Native libraries (created by Constellation) are born canonical.

		// When the background reconcile finishes (filesystem walk complete,
		// any stale cache rows refreshed), re-read the cache snapshot so the
		// UI picks up any notes/links/tags that changed outside Constellation
		// since the last launch. Cheap: SQLite queries only.
		const unlistenCacheReconciled = await listen('cache-reconciled', () => {
			// Re-read snapshot without kicking off another reconcile —
			// cacheRefreshing gate prevents the reconcile from running twice.
			if (!cacheRefreshing) {
				refreshLibraryCaches().catch(() => {});
			}
		});
		cleanupFns.push(() => { try { unlistenCacheReconciled(); } catch {} });

		// Search engine init is driven by cache_reconcile() (which invokes
		// constellation_search_init on a background thread). When it finishes
		// the cache-reconciled event fires; we load link counts then.
		const unlistenSearchReady = await listen('cache-reconciled', async () => {
			searchEngineReady = true;
			try {
				const counts: Record<string, number> = await invoke('constellation_search_link_counts');
				searchLinkCounts = new Map(Object.entries(counts).map(([k, v]) => [k, { incoming: v }]));
			} catch {}
			// Living Link System: apply weight decay on startup (background)
			invoke('constellation_link_decay').catch(() => {});
		});
		cleanupFns.push(() => { try { unlistenSearchReady(); } catch {} });

		// Semantic search: ONNX engine lazy-loads on first search/embed call

		// Second screen event listeners
		const unlistenScreenNote = await onNoteToMain(async (note: ScreenNote) => {
			await openNoteTab(note.path, note.libraryName, note.libraryColor);
		});
		const unlistenScreenClosed = await onScreenClosed(() => {
			secondScreenOpen = false;
			// Restore right sidebar when SS is closed via its own × button
			if (rightSidebarBeforeSS) rightSidebarOpen = true;
			emitEditorPanels({ active: false });
		});
		// When the second screen saves a note, reload it in the main window if open
		const unlistenNoteSaved = await onNoteSaved(async (path) => {
			if (wasRecentlyWritten(path)) return; // we wrote it ourselves
			const tab = get(openTabs).find(t => t.path === path);
			if (tab) {
				try {
					const content = await invoke<string>('read_note', { filePath: path });
					tab.content = content;
					openTabs.update(tabs => tabs);
				} catch {}
			}
		});

		// Global keyboard shortcuts — capture phase to beat browser defaults
		document.addEventListener('keydown', handleGlobalKeydown, true);

		// Cleanup on destroy
		cleanupFns.push(
			() => document.removeEventListener('keydown', handleGlobalKeydown, true),
			unlistenWatcher,
			unlistenScreenNote,
			unlistenScreenClosed,
			unlistenNoteSaved,
		);
	});

	const cleanupFns: (() => void)[] = [];
	onDestroy(() => {
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
	/** Yield to the browser event loop so UI clicks can preempt heavy work. */
	const yieldToUI = () => new Promise<void>(r => setTimeout(r, 0));

	/**
	 * Write the boot-perf scorecard for `lab/boot-perf/BOOT-BUDGET.md`.
	 * Called twice: once when `boot:hydrated` is marked (Criteria 1+2), and
	 * again when `boot:graph-ready` resolves so `graph_ready_ms` is recorded
	 * even though it isn't a ship-gate. Both writes are idempotent — the
	 * first write fills paint/hydrated; the second overwrites with the full
	 * scorecard (paint/hydrated/graph-ready).
	 */
	let bootPerfCorePhaseWritten = false;
	let bootPerfGraphPhaseWritten = false;
	/** Per-step timings captured inside Rust's constellation_boot_bundle.
	 *  Populated once on first paint; written to boot-perf.latest.json so
	 *  cold-boot attribution is possible without rebuilds. See
	 *  lab/boot-perf/boot-bundle-cold-start.md. */
	let bootBundleTimings: Array<[string, number]> = [];
	/** Diagnostic — time between `await invoke(...)` issue and resolution
	 *  for each awaited boot IPC. Paired with the Rust `timings_ms` inside
	 *  each response: if `wall_ms >> sum(timings_ms)`, the difference is
	 *  queue/contention time, not Rust execution time. */
	let cacheSnapshotCoreWallMs = 0;
	let cacheSnapshotCoreServerTimings: Array<[string, number]> = [];
	let cacheSnapshotGraphWallMs = 0;
	let cacheSnapshotGraphServerTimings: Array<[string, number]> = [];
	/** Criterion 2 IPC-overhead diagnostic (2026-04-19). We split the
	 *  previously-undifferentiated "wall time" bucket into three:
	 *
	 *   1. transport_ms  = clientRecvUnixMs - server_return_unix_ms
	 *      Pure IPC: the time between Rust returning the struct and the
	 *      JS `await invoke(...)` resolving. Isolates Tauri serialize +
	 *      WebView2 pipe + JS deserialize — independent of what the
	 *      caller does with the payload.
	 *
	 *   2. assign_ms     = perf.now() after allNotes=... - perf.now() after invoke
	 *      Time to apply the response to reactive state. On a 7,600-note
	 *      Universe this is the Svelte 5 reactive cascade triggered by
	 *      the `allNotes = ...` assignment — if it dominates, the fix is
	 *      to chunk the assignment across `requestAnimationFrame` rather
	 *      than attack IPC.
	 *
	 *   3. The remainder of wall_ms is everything else (promise micro-task
	 *      scheduling, other JS running in between, etc.).
	 *
	 *  If core_wall = 22,614 ms but server_timings sum to only ~48 ms, we
	 *  need to know whether the missing ~22,500 ms lives in transport or
	 *  assign. The raw unix timestamps are also shipped so we can sanity-
	 *  check clock drift between Rust and JS clocks. */
	let cacheSnapshotCoreTransportMs = 0;
	let cacheSnapshotCoreAssignMs = 0;
	let cacheSnapshotCoreServerReturnUnixMs = 0;
	let cacheSnapshotCoreClientRecvUnixMs = 0;
	let cacheSnapshotGraphTransportMs = 0;
	let cacheSnapshotGraphAssignMs = 0;
	let cacheSnapshotGraphServerReturnUnixMs = 0;
	let cacheSnapshotGraphClientRecvUnixMs = 0;
	/** Round 2 of IPC-overhead diagnostic (2026-04-19). Adds queue-time
	 *  attribution — the time between JS issuing `invoke(...)` and the
	 *  Rust command body actually starting execution. If `queue_ms`
	 *  dominates `wall_ms`, the bottleneck is Tauri's dispatcher or the
	 *  blocking-pool scheduler, not any work we do inside the command. */
	let cacheSnapshotCoreInvokeStartUnixMs = 0;
	let cacheSnapshotCoreServerStartUnixMs = 0;
	let cacheSnapshotCoreQueueMs = 0;
	let cacheSnapshotCoreBodyMs = 0;
	let cacheSnapshotGraphInvokeStartUnixMs = 0;
	let cacheSnapshotGraphServerStartUnixMs = 0;
	let cacheSnapshotGraphQueueMs = 0;
	let cacheSnapshotGraphBodyMs = 0;
	/** Round 5 follow-up (2026-04-19). JS-event-loop heartbeat. After two
	 *  rounds of converting sync commands to `(async)` failed to move
	 *  `core_queue_ms` (stayed at ~19.5 s even with DashboardView fully gated
	 *  off), the live hypothesis is that the JS thread itself is blocked
	 *  between `invoke('cache_boot_snapshot_core')` and the Rust dispatcher
	 *  picking up the message. A `setInterval(…, 100)` samples the event
	 *  loop: if max-gap-between-fires ≪ 500 ms, JS is alive during the
	 *  window (next diagnostic: Rust-side arrival tracing). If max-gap
	 *  ≫ 5,000 ms, JS is blocked (next diagnostic: find the blocker). */
	let bootHeartbeatMaxGapMs = 0;
	let bootHeartbeatLastFire = 0;
	let bootHeartbeatInterval: ReturnType<typeof setInterval> | undefined = undefined;
	/** Round 6 diagnostic (2026-04-19). Rust-side IPC arrival log.
	 *  Populated once at boot:hydrated by `invoke('get_perf_trace_log')`,
	 *  which returns `[command_name, unix_ms]` tuples captured by the
	 *  `invoke_handler` wrapper in lib.rs on every command dispatch. If
	 *  during the 18.6 s queue window the log shows many arrivals →
	 *  dispatcher serialization; if it shows NO arrivals → the delay
	 *  is upstream of Rust (WebView2 / wry level). */
	let ipcArrivalLog: Array<[string, number]> = [];
	/** Wall-clock for the fire-and-forget chain issued right before
	 *  `refreshLibraryCaches()`. These race into Tauri's command queue
	 *  alongside `cache_boot_snapshot_core`; if any is slow it may starve
	 *  the core snapshot. */
	let loadAllStatsWallMs = 0;
	let startWatchingAllWallMs = 0;
	let loadAllAppearancesWallMs = 0;
	function buildBootPerfReport(includeGraphPhase: boolean): Record<string, unknown> {
		const paint = performance.getEntriesByName('boot:paint')[0]?.startTime ?? 0;
		const libs = performance.getEntriesByName('boot:libraries-loaded')[0]?.startTime ?? 0;
		const hyd = performance.getEntriesByName('boot:hydrated')[0]?.startTime ?? 0;
		const graphReadyMark = performance.getEntriesByName('boot:graph-ready')[0]?.startTime ?? 0;
		return {
			paint_ms: Math.round(paint),
			libraries_loaded_ms: Math.round(libs),
			hydrated_ms: Math.round(hyd),
			// graph_ready_ms is informational — not a ship-gate criterion.
			graph_ready_ms: includeGraphPhase ? Math.round(graphReadyMark) : null,
			note_count: allNotes.length,
			timestamp: new Date().toISOString(),
			// Criteria from lab/boot-perf/BOOT-BUDGET.md
			criterion_1_paint: paint <= 2500 ? 'PASS' : 'FAIL',
			criterion_2_hydrated: hyd <= 6000 ? 'PASS' : 'FAIL',
			// Per-step timings from constellation_boot_bundle — diagnostic only.
			// Empty on first paint before the bundle resolves; populated on the
			// graph-phase write (and any subsequent writes).
			boot_bundle_timings: bootBundleTimings,
			// ── Deep attribution for Criterion 2 cold-boot regression ──
			// `*_wall_ms` is the frontend-side elapsed from `await invoke(...)`
			// to resolution. `*_server_timings` is the Rust-side per-phase
			// breakdown returned inside the response. If wall >> sum(server),
			// the time is queue/contention; if read_notes dominates server,
			// the fix is the SQLite row-scan.
			cache_snapshot_core_wall_ms: cacheSnapshotCoreWallMs,
			cache_snapshot_core_server_timings: cacheSnapshotCoreServerTimings,
			cache_snapshot_graph_wall_ms: cacheSnapshotGraphWallMs,
			cache_snapshot_graph_server_timings: cacheSnapshotGraphServerTimings,
			// ── IPC-overhead attribution (2026-04-19 Criterion 2 diagnostic) ──
			// Splits the single wall_ms bucket into transport (pure IPC) and
			// assign (reactive cascade cost of applying payload to state).
			// Raw unix timestamps are included so clock skew between Rust and
			// JS can be ruled out if transport_ms looks implausible.
			cache_snapshot_core_transport_ms: cacheSnapshotCoreTransportMs,
			cache_snapshot_core_assign_ms: cacheSnapshotCoreAssignMs,
			cache_snapshot_core_server_return_unix_ms: cacheSnapshotCoreServerReturnUnixMs,
			cache_snapshot_core_client_recv_unix_ms: cacheSnapshotCoreClientRecvUnixMs,
			cache_snapshot_graph_transport_ms: cacheSnapshotGraphTransportMs,
			cache_snapshot_graph_assign_ms: cacheSnapshotGraphAssignMs,
			cache_snapshot_graph_server_return_unix_ms: cacheSnapshotGraphServerReturnUnixMs,
			cache_snapshot_graph_client_recv_unix_ms: cacheSnapshotGraphClientRecvUnixMs,
			// ── Round 2: queue-time diagnostic (2026-04-19) ──
			// If queue_ms dominates wall_ms but body_ms is small, the
			// bottleneck is Tauri's dispatcher / blocking-pool scheduler,
			// NOT anything in the SQL or IPC codepath. body_ms =
			// server_return_unix_ms - server_start_unix_ms (pure in-Rust
			// execution); queue_ms = server_start_unix_ms - invoke_start_unix_ms.
			cache_snapshot_core_invoke_start_unix_ms: cacheSnapshotCoreInvokeStartUnixMs,
			cache_snapshot_core_server_start_unix_ms: cacheSnapshotCoreServerStartUnixMs,
			cache_snapshot_core_queue_ms: cacheSnapshotCoreQueueMs,
			cache_snapshot_core_body_ms: cacheSnapshotCoreBodyMs,
			cache_snapshot_graph_invoke_start_unix_ms: cacheSnapshotGraphInvokeStartUnixMs,
			cache_snapshot_graph_server_start_unix_ms: cacheSnapshotGraphServerStartUnixMs,
			cache_snapshot_graph_queue_ms: cacheSnapshotGraphQueueMs,
			cache_snapshot_graph_body_ms: cacheSnapshotGraphBodyMs,
			// Fire-and-forget chain that races alongside the core snapshot.
			load_all_stats_wall_ms: loadAllStatsWallMs,
			start_watching_all_wall_ms: startWatchingAllWallMs,
			load_all_appearances_wall_ms: loadAllAppearancesWallMs,
			// Round 5 follow-up: JS-event-loop heartbeat. Max gap between
			// `setInterval(…, 100)` firings from boot:paint onward. Small
			// (< 500) → JS alive; large (> 5000) → JS blocked for that long.
			boot_heartbeat_max_gap_ms: Math.round(bootHeartbeatMaxGapMs),
			// Round 6 diagnostic: Rust-side IPC arrival log. Each entry is
			// `[command_name, unix_ms]` captured by the `invoke_handler`
			// wrapper in lib.rs the moment a command reaches the Rust
			// dispatcher. Cross-reference with `cache_snapshot_core_*_unix_ms`
			// to see what (if anything) ran between JS `postMessage` and the
			// core snapshot's Rust body starting.
			ipc_arrival_log: ipcArrivalLog,
		};
	}
	async function recordBootPerf() {
		if (bootPerfCorePhaseWritten) return;
		bootPerfCorePhaseWritten = true;
		// Freeze the heartbeat max-gap to the boot:paint → boot:hydrated window.
		if (bootHeartbeatInterval !== undefined) {
			clearInterval(bootHeartbeatInterval);
			bootHeartbeatInterval = undefined;
		}
		// Round 6 diagnostic: fetch the Rust-side IPC arrival log before
		// writing the report. Captures every command that reached the
		// Rust dispatcher up to this moment, with a Unix-ms timestamp.
		try {
			const log = await invoke<Array<[string, number]>>('get_perf_trace_log');
			if (Array.isArray(log)) ipcArrivalLog = log;
		} catch (e) {
			console.warn('[boot-perf] failed to fetch IPC arrival log', e);
		}
		try {
			const report = buildBootPerfReport(false);
			console.log('[boot-perf]', report);
			// Persist to .constellation/boot-perf.latest.json so the
			// Settings → Debug panel and the lab harness can read it.
			await invoke('write_boot_perf_report', { reportJson: JSON.stringify(report) }).catch(() => {});
		} catch (e) {
			console.warn('[boot-perf] recording failed', e);
		}
	}
	async function recordBootPerfGraphPhase() {
		if (bootPerfGraphPhaseWritten) return;
		bootPerfGraphPhaseWritten = true;
		try {
			const report = buildBootPerfReport(true);
			console.log('[boot-perf] graph-ready', report);
			await invoke('write_boot_perf_report', { reportJson: JSON.stringify(report) }).catch(() => {});
		} catch (e) {
			console.warn('[boot-perf] graph-phase recording failed', e);
		}
	}
	async function refreshLibraryCaches() {
		// Prevent concurrent scans — skip if one is already in progress.
		// The guard spans BOTH phases (core await + deferred graph load) so
		// re-entrant callers during the idle-callback window still short-circuit.
		if (cacheRefreshing) return;
		cacheRefreshing = true;

		// ── Phase 1 (awaited): CORE snapshot ────────────────────────────
		// Minimal payload (notes + is_cold) needed to paint the sidebar /
		// file tree / Sight. Returns in low-millis on a 7,600-note Universe.
		// The heavy link graph + tag aggregation is deferred to Phase 2
		// via requestIdleCallback so `boot:hydrated` fires before the
		// ~656k-row link payload crosses IPC.
		let core: {
			notes: { name: string; path: string; library_name: string }[];
			is_cold: boolean;
			timings_ms?: Array<[string, number]>;
			server_return_unix_ms?: number;
			server_start_unix_ms?: number;
		};
		const coreInvokeStart = performance.now();
		// Round 2 (2026-04-19) — capture the wall-clock instant the invoke
		// was issued. Paired with Rust `server_start_unix_ms` (stamped at
		// the first line of the command body), the delta is pure dispatcher
		// queue time. If queue_ms is huge but body_ms is small, Tauri's
		// blocking-pool scheduler is where the 22.9 s disappears.
		const coreInvokeStartUnixMs = Date.now();
		try {
			core = await invoke('cache_boot_snapshot_core');
		} catch {
			core = { notes: [], is_cold: true };
		}
		// Capture the instant `await invoke(...)` resolved — before any
		// reactive work. Paired with `core.server_return_unix_ms` (captured
		// in Rust right before `Ok(...)`), the delta is pure transport cost.
		const coreClientRecvUnixMs = Date.now();
		const corePostInvokePerfMs = performance.now();
		cacheSnapshotCoreWallMs = Math.round(corePostInvokePerfMs - coreInvokeStart);
		if (Array.isArray(core.timings_ms)) {
			cacheSnapshotCoreServerTimings = core.timings_ms;
		}
		cacheSnapshotCoreServerReturnUnixMs = core.server_return_unix_ms ?? 0;
		cacheSnapshotCoreClientRecvUnixMs = coreClientRecvUnixMs;
		cacheSnapshotCoreTransportMs = core.server_return_unix_ms
			? Math.max(0, coreClientRecvUnixMs - core.server_return_unix_ms)
			: 0;
		// Round-2 queue + body attribution.
		cacheSnapshotCoreInvokeStartUnixMs = coreInvokeStartUnixMs;
		cacheSnapshotCoreServerStartUnixMs = core.server_start_unix_ms ?? 0;
		cacheSnapshotCoreQueueMs = core.server_start_unix_ms
			? Math.max(0, core.server_start_unix_ms - coreInvokeStartUnixMs)
			: 0;
		cacheSnapshotCoreBodyMs = core.server_start_unix_ms && core.server_return_unix_ms
			? Math.max(0, core.server_return_unix_ms - core.server_start_unix_ms)
			: 0;

		if (!core.is_cold) {
			allNotes = core.notes.map(n => ({
				name: n.name,
				path: n.path,
				libraryName: n.library_name,
			}));
			// Measure how long the reactive cascade from `allNotes = ...` took.
			// On Svelte 5, this includes re-running every `$derived` / `$effect`
			// that reads `allNotes` (file tree, sidebar, Sight). If this number
			// is large, the fix path is chunking via requestAnimationFrame, not
			// attacking IPC.
			cacheSnapshotCoreAssignMs = Math.round(performance.now() - corePostInvokePerfMs);

			// Boot-perf Criterion 2: fully responsive. Reached when the
			// core snapshot has populated the notes list — the UI can paint
			// the sidebar/file tree/Sight immediately even while the link
			// graph is still streaming in.
			performance.mark('boot:hydrated');
			recordBootPerf();
		}

		// ── Phase 2 (deferred): GRAPH snapshot ──────────────────────────
		// Fires via requestIdleCallback so it never competes with the paint
		// that just happened. Populates link/tag state and rebuilds Sky View
		// data; bumps `skyVersion` so open views re-derive off the new data.
		const loadGraph = async (): Promise<void> => {
			try {
				let graph: {
					links: NoteLink[];
					tags: Record<string, number>;
					timings_ms?: Array<[string, number]>;
					server_return_unix_ms?: number;
					server_start_unix_ms?: number;
				};
				const graphInvokeStart = performance.now();
				const graphInvokeStartUnixMs = Date.now();
				// MIG-001 Step 9 (parallel): kick off the pre-shaped sky_*
				// payload invoke CONCURRENTLY with the classic graph IPC.
				// SQLite WAL allows both read commands to hit their own
				// dedicated connections without contention. Total wall time
				// becomes max(graph_ipc, sky_ipc) instead of their sum — on
				// the target universe this is bounded by the slower graph
				// IPC (~7s) and makes the sky IPC effectively free.
				type SkySnapshot = {
					nodes: SkyNode[];
					links: SkyLink[];
					isReady: boolean;
					timingsMs: Array<[string, number]>;
				};
				const skyPromise = (async (): Promise<SkySnapshot | null> => {
					try {
						return await invoke<SkySnapshot>('cache_boot_snapshot_sky');
					} catch (err) {
						console.warn('[sky] cache_boot_snapshot_sky failed, falling back to buildSkyData:', err);
						return null;
					}
				})();
				try {
					graph = await invoke('cache_boot_snapshot_graph');
				} catch {
					graph = { links: [], tags: {} };
				}
				const graphClientRecvUnixMs = Date.now();
				const graphPostInvokePerfMs = performance.now();
				cacheSnapshotGraphWallMs = Math.round(graphPostInvokePerfMs - graphInvokeStart);
				if (Array.isArray(graph.timings_ms)) {
					cacheSnapshotGraphServerTimings = graph.timings_ms;
				}
				cacheSnapshotGraphServerReturnUnixMs = graph.server_return_unix_ms ?? 0;
				cacheSnapshotGraphClientRecvUnixMs = graphClientRecvUnixMs;
				cacheSnapshotGraphTransportMs = graph.server_return_unix_ms
					? Math.max(0, graphClientRecvUnixMs - graph.server_return_unix_ms)
					: 0;
				// Round-2 queue + body attribution (see core phase above).
				cacheSnapshotGraphInvokeStartUnixMs = graphInvokeStartUnixMs;
				cacheSnapshotGraphServerStartUnixMs = graph.server_start_unix_ms ?? 0;
				cacheSnapshotGraphQueueMs = graph.server_start_unix_ms
					? Math.max(0, graph.server_start_unix_ms - graphInvokeStartUnixMs)
					: 0;
				cacheSnapshotGraphBodyMs = graph.server_start_unix_ms && graph.server_return_unix_ms
					? Math.max(0, graph.server_return_unix_ms - graph.server_start_unix_ms)
					: 0;

				allLibraryLinks = graph.links;
				allLibraryTags = graph.tags;
				// MIG-004 §9: build path → aliases[] map for alias-aware
				// Backlinks / Outgoing / Map / Sight queries. Frontend
				// consumers look up the active note's aliases by path
				// and pass them to getBacklinks (and similar). ~1.4k
				// entries on the reference universe; insertion is
				// <1ms total.
				notePathToAliases = new Map();
				if (Array.isArray((graph as any).aliases)) {
					for (const a of (graph as any).aliases as { path: string; aliasLower: string }[]) {
						const list = notePathToAliases.get(a.path) ?? [];
						list.push(a.aliasLower);
						notePathToAliases.set(a.path, list);
					}
				}
				// Boot graph carries the canonical counts from the DB, which
				// already absorbed every traversal that was fired since the
				// last fetch — drop our optimistic bumps so they don't add
				// on top.
				clearLinkTraversalBumps();

				const libraryList = $libraries;
				// Resolve the parallel sky snapshot kicked off earlier.
				// If the graph IPC took longer (common), this await is
				// free — the sky payload is already in hand. Fallback
				// to buildSkyData when the back-fill hasn't stamped
				// schema_versions.sky yet or the IPC errored.
				const sky = await skyPromise;
				// Enter this block if EITHER source can produce a graph:
				// graph IPC returned links, OR sky IPC is ready. The old
				// guard skipped sky assignment entirely when graph failed
				// (links=[]) even if sky had data — the graph-failure
				// fallback path would leave Sky View empty.
				if (libraryList.length > 0 && (graph.links.length > 0 || (sky && sky.isReady))) {
					if (sky && sky.isReady) {
						// Trust the readiness flag — a zero-node Universe
						// with isReady=true is legitimately empty; falling
						// back would produce the same empty result.
						skyNodes = sky.nodes;
						skyLinks = sky.links;
						skyVersion++;
					} else {
						// MIG-005-PARITY: pass notePathToAliases so the
						// buildSkyData fallback can resolve renamed-target
						// wikilinks via the alias map — same alias-aware
						// resolution the cache_boot_snapshot_sky path
						// performs server-side. Without this, the fallback
						// silently drops every edge whose target was renamed.
						const { nodes, links: gLinks } = buildSkyData(graph.links, allNotes, notePathToAliases);
						skyNodes = nodes;
						skyLinks = gLinks;
						skyVersion++;
					}
				}

				// Measure how long applying the graph to reactive state took.
				// Captured AFTER buildSkyData since that synchronously iterates
				// the full link set on the main thread (store.ts:1504-1543) and
				// is part of the "cost of receiving the graph payload" bucket.
				cacheSnapshotGraphAssignMs = Math.round(performance.now() - graphPostInvokePerfMs);

				// Signal: Sky View / backlinks / tag browser can now use the
				// full graph. Components that render a degraded "Loading…"
				// state while waiting can read this flag to flip to full UI.
				graphReady = true;
				performance.mark('boot:graph-ready');
				recordBootPerfGraphPhase();
			} finally {
				cacheRefreshing = false;
			}
		};

		const schedule = (fn: () => void): void => {
			// requestIdleCallback is a browser primitive; fall back to
			// setTimeout(0) on WebKit (Safari/iOS) where it isn't implemented.
			const w = window as unknown as { requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => number };
			if (typeof w.requestIdleCallback === 'function') {
				w.requestIdleCallback(fn, { timeout: 3000 });
			} else {
				setTimeout(fn, 0);
			}
		};
		schedule(() => { loadGraph().catch(() => { cacheRefreshing = false; }); });

		// ═══ ZERO BOOT-TIME WALKS — see initializeApp() comment ════════
		// `cache_reconcile` and `enrichNodesBackground` used to fire here.
		// Both walked every library on every boot, causing the audible
		// disk thrashing and IPC saturation that made the app
		// unresponsive for minutes after first paint.
		//
		// They are now triggered ONLY by:
		//   - The runtime file watcher (per-file, incremental).
		//   - The user clicking Settings → Rebuild Index.
		//   - First-ever launch when the cache is empty (one-time modal).
		//
		// External edits made while the app was closed (git pull, sync
		// clients) are detected by a future cheap stat-only sweep — see
		// Criterion 4 in lab/boot-perf/BOOT-BUDGET.md.
	}

	async function enrichNodesBackground(libraryList: typeof $libraries) {
		if (libraryList.length === 0) return;
		const libPaths = libraryList.map(l => l.path);
		const libNames = libraryList.map(l => l.name);

		// Strata — per library, yield between
		for (let i = 0; i < libPaths.length; i++) {
			try {
				const strata = await invoke<{ note_path: string; stratum: number }[]>(
					'compute_note_strata', { libraryPath: libPaths[i], libraryName: libNames[i] }
				);
				const sMap = new Map(strata.map(s => [s.note_path.replace(/\\/g, '/').toLowerCase(), s.stratum]));
				for (const node of skyNodes) {
					const key = node.path.replace(/\\/g, '/').toLowerCase();
					const s = sMap.get(key);
					if (s !== undefined) node.stratum = s;
				}
			} catch { /* skip */ }
			await yieldToUI();
		}
		// Maturity
		const newMatMap = new Map<string, string>();
		for (let i = 0; i < libPaths.length; i++) {
			try {
				const maturities = await invoke<{ note_path: string; state: string }[]>(
					'compute_note_maturity', { libraryPath: libPaths[i], libraryName: libNames[i] }
				);
				for (const m of maturities) newMatMap.set(m.note_path.replace(/\\/g, '/').toLowerCase(), m.state);
				for (const node of skyNodes) {
					const key = node.path.replace(/\\/g, '/').toLowerCase();
					const m = newMatMap.get(key);
					if (m) node.maturity = m;
				}
			} catch { /* skip */ }
			await yieldToUI();
		}
		maturityMap = newMatMap;
		// Origins
		for (let i = 0; i < libPaths.length; i++) {
			try {
				const origins = await invoke<{ note_path: string; origin_type: string }[]>(
					'compute_note_origins', { libraryPath: libPaths[i], libraryName: libNames[i] }
				);
				const oMap = new Map(origins.map(o => [o.note_path.replace(/\\/g, '/').toLowerCase(), o.origin_type]));
				for (const node of skyNodes) {
					const key = node.path.replace(/\\/g, '/').toLowerCase();
					const o = oMap.get(key);
					if (o && o !== 'none') node.originType = o;
				}
			} catch { /* skip */ }
			await yieldToUI();
		}
		// Stages
		const newStageMap = new Map<string, string>();
		for (let i = 0; i < libPaths.length; i++) {
			try {
				const stages = await invoke<[string, string][]>('scan_note_stages', { libraryPath: libPaths[i] });
				for (const [path, stage] of stages) {
					newStageMap.set(path.replace(/\\/g, '/').toLowerCase(), stage);
				}
			} catch { /* skip */ }
			await yieldToUI();
		}
		stageMap = newStageMap;
		// Lenses + tensions (cheap, once)
		try { availableLenses = await invoke('list_lenses'); } catch { availableLenses = []; }
		try {
			tensionReport = await invoke('detect_tensions', { libraryPath: libPaths[0], libraryName: libNames[0] });
		} catch { tensionReport = null; }
		skyVersion++;
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

		// Ctrl+. → open the Emoji & Icon picker (Obsidian-parity shortcut).
		// Gated on the Core Plug-In toggle.
		if ((e.ctrlKey || e.metaKey) && e.key === '.' && !e.shiftKey && !e.altKey
			&& $appSettings.enabledFeatures?.emojiIconPicker !== false) {
			e.preventDefault();
			showPicker = true;
			return;
		}

		// Escape always closes overlays (not remappable)
		if (e.key === 'Escape') {
			if (showNewLibraryDropdown) { showNewLibraryDropdown = false; newLibName = ''; return; }
			if (showCommandPalette) { showCommandPalette = false; return; }
			if (showQuickSwitcher) { showQuickSwitcher = false; return; }
			if (showSkyView) { showSkyView = false; return; }
			if (lensActive) { lensActive = false; return; }
			if (showOrgChart) { showOrgChart = false; return; }
			if (sidebarMode === 'skyview') { sidebarMode = 'tree'; return; }
			if (showGlobalTasks) { showGlobalTasks = false; return; }
			if (showIndex) { showIndex = false; return; }
			if (showTemplatePicker) { showTemplatePicker = false; return; }
			if (showWorkspaces) { showWorkspaces = false; return; }
			if (showSettings) { showSettings = false; return; }
			if (showImporter) { showImporter = false; return; }
			if (showPicker) { showPicker = false; return; }
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

			// Apply template if found — preserve canonical frontmatter fields (cid, kind, title)
			if (templateBody.trim()) {
				try {
					// Read back the canonical frontmatter that create_note injected
					const noteContent: string = await invoke('read_note', { filePath: newPath });
					const canonicalFields: string[] = [];
					const fmMatch = noteContent.match(/^---\n([\s\S]*?)\n---/);
					if (fmMatch) {
						for (const line of fmMatch[1].split('\n')) {
							const t = line.trim();
							if (t.startsWith('title:') || t.startsWith('cid:') || t.startsWith('kind:')) {
								canonicalFields.push(t);
							}
						}
					}

					const noteFolder = newPath.replace(/\\/g, '/').split('/').slice(-2, -1)[0] || '';
					const ctx = { title: name, folder: noteFolder, library: lib.name, filePath: newPath };
					const result = await processTemplateAsync(templateBody, ctx, buildTemplateCallbacks());
					// Merge: canonical fields first, then user defaults (skip duplicates)
					const mergedFM = [...canonicalFields, ...defaultFM.split('\n').filter(l => {
						const key = l.split(':')[0]?.trim();
						return key && !canonicalFields.some(cf => cf.startsWith(key + ':'));
					})].join('\n');
					const fullContent = `---\n${mergedFM}\n---\n${result.content}`;
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

	function handleNewLibrary() {
		showNewLibraryDropdown = !showNewLibraryDropdown;
		newLibName = '';
	}

	async function handleCreateNewLib() {
		if (!newLibName.trim()) return;
		showNewLibraryDropdown = false;
		try {
			await createNewLibrary(newLibName.trim());
			await loadLibraries();
			await refreshLibraryCaches();
			// Rebuild search index to include new library's notes
			initSearchIndex().then(() => { searchEngineReady = true; }).catch(() => {});
			newLibName = '';
		} catch (e) { console.error('[Library] Create failed:', e); }
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

	async function toggleLens() {
		if (lensActive) {
			// Toggle OFF: hide the overlay but keep computed data in memory.
			// The next toggle-on will reuse it if the graph hasn't changed.
			// Data is only discarded when skyVersion increments (graph changed)
			// — see the WTD $effect above.
			lensActive = false;
			return;
		}

		// Toggle ON — serve from cache if data is still fresh.
		if (!lensDataStale && lensHealth !== null) {
			lensActive = true;
			return;
		}

		lensLoading = true;
		// Snapshot the graph version so we can detect a mid-computation change.
		const computeVersion = skyVersion;
		try {
			// 1. Compute centrality in Rust
			const libPaths = $libraries.map(l => [l.path, l.name] as [string, string]);
			const result = await invoke<{ centrality: Record<string, number>; node_count: number; edge_count: number }>(
				'constellation_sight_centrality', { libraryPaths: libPaths }
			);
			lensCentrality = new Map(Object.entries(result.centrality));

			// 2. Run community detection (existing JS Louvain)
			const clusterResult = detectClusters(
				skyNodes.map(n => ({ id: n.id, name: n.name })),
				skyLinks.map(l => ({ source: l.source, target: l.target })),
			);
			lensCommunities = clusterResult.clusters;
			lensCommunityAssignments = clusterResult.assignments;

			// 3. Compute structural gaps
			lensGaps = computeStructuralGaps(
				clusterResult.clusters,
				skyLinks.map(l => ({ source: l.source, target: l.target })),
				clusterResult.assignments,
			);

			// 4. Compute universe health
			lensHealth = computeUniverseHealth(
				clusterResult.modularity,
				clusterResult.clusters,
				skyNodes.length,
				skyLinks.length,
				lensGaps.length,
			);

			// 5. Stratum-weighted centrality (Feature 2)
			const weightedCentrality = stratumWeightedCentrality(
				lensCentrality,
				skyLinks.map(l => ({ source: l.source, target: l.target })),
				skyNodes,
			);
			lensCentrality = weightedCentrality; // Replace with stratum-weighted version

			// 6. Build top bridges list (top 10 by weighted centrality)
			lensBridges = [...lensCentrality.entries()]
				.sort((a, b) => b[1] - a[1])
				.slice(0, 10)
				.map(([id, centrality]) => {
					const node = skyNodes.find(n => n.id === id);
					return { id, name: node?.name ?? id, centrality };
				});

			// 7. Community profiles (Features 4 & 5: maturity + provenance)
			lensCommunityProfiles = buildCommunityProfiles(
				clusterResult.clusters,
				clusterResult.assignments,
				skyNodes,
			);

			// 8. Bridge suggestions for gaps (Feature 7)
			lensGaps = suggestBridges(
				lensGaps,
				clusterResult.clusters,
				skyNodes.map(n => ({ id: n.id, name: n.name })),
				skyLinks.map(l => ({ source: l.source, target: l.target })),
			);

			// 9. Contradictions from Rust (Feature 3)
			lensContradictions = (result as any).contradictions ?? [];

			// Mark cache fresh only if the graph didn't change mid-computation.
			if (skyVersion === computeVersion) {
				lensDataStale = false;
			}
			lensActive = true;
		} catch (e) {
			console.error('[Lens] Failed to compute:', e);
		}
		lensLoading = false;
	}

	async function handleToggleSecondScreen() {
		if (!hasMultipleDisplays) return; // SS requires 2+ monitors
		const isOpen = await invoke<boolean>('is_second_screen_open');
		if (isOpen) {
			await invoke('close_second_screen');
			secondScreenOpen = false;
			// Restore right sidebar if it was open before SS
			if (rightSidebarBeforeSS) rightSidebarOpen = true;
			emitEditorPanels({ active: false });
		} else {
			// Remember sidebar state, then hide it — main window becomes clean writing space
			rightSidebarBeforeSS = rightSidebarOpen;
			rightSidebarOpen = false;
			await openSecondScreenSmart();
			secondScreenOpen = true;
			// Wait for SS to signal it has registered all listeners
			await waitForScreenReady();
			const tab = get(activeTab);
			if (tab?.path) {
				emitEditorPanels({
					active: true,
					notePath: tab.path,
					noteName: tab.name,
					libraryName: tab.libraryName,
					libraryPath: tab.libraryPath ?? '',
					content: tab.content,
				});
			}
		}
	}

	async function handleSendToSecondScreen() {
		if (!hasMultipleDisplays) return; // SS requires 2+ monitors
		const tab = get(activeTab);
		if (!tab?.path) return;
		if (!secondScreenOpen) {
			rightSidebarBeforeSS = rightSidebarOpen;
			rightSidebarOpen = false;
			await openSecondScreenSmart();
			secondScreenOpen = true;
			await waitForScreenReady();
		}
		await sendNoteToScreen({
			path: tab.path,
			name: tab.name,
			libraryName: tab.libraryName,
			libraryPath: tab.libraryPath,
			libraryColor: tab.libraryColor,
		});
		// Also emit editor panels so SS shows context
		emitEditorPanels({
			active: true,
			notePath: tab.path,
			noteName: tab.name,
			libraryName: tab.libraryName,
			libraryPath: tab.libraryPath ?? '',
			content: tab.content,
		});
	}

	async function handleQuickSwitchSelect(path: string, libraryName: string) {
		const libraryColor = libraryColorMap[libraryName] ?? '#7c3aed';
		await openNoteTab(path, libraryName, libraryColor);
	}

	async function handleSkyNodeClick(path: string, libraryName: string, highlightTerm?: string) {
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
		// Pass the currently-active tab as _fromNotePath so Living Link records
		// the traversal from the note the user was reading through Sky View to
		// the node they clicked.
		const fromPath = $activeTab?.path && $activeTab.path !== path ? $activeTab.path : undefined;
		await openNoteTab(path, libraryName, color, highlightTerm, false, fromPath);
		// Entering inspect mode from SV: sidebars were already zeroed by the
		// fullPage snapshot when SV opened. Steal that snapshot so the
		// eventual dismiss restores the user's pre-SV layout (not {false,
		// false}). Idempotent via the `!skyViewInspectMode` guard — re-
		// entering from another SV click doesn't overwrite.
		if (!skyViewInspectMode) {
			const fp = sidebarSnapshots.get('fullPage');
			sidebarSnapshots.set('skyInspect', fp
				? { ...fp }
				: { left: sidebarOpen, right: rightSidebarOpen }
			);
		}
		sidebarOpen = false;
		rightSidebarOpen = false;
		showSkyView = false;
		skyViewInspectMode = true;
		if (highlightTerm) skyViewReturnPending = true;
	}

	function handleTagClick(tag: string) {
		searchHubInitialQuery = `#${tag}`;
		showSearchHub = true;
		showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; lensActive = false; showInspector360 = false;
		/* fullPageActive $effect handles sidebar snapshot */
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

	// ─── Index panel: FTS5 vocab read ───
	// The Index panel is now backed by the `notes_vocab` virtual table
	// (fts5vocab over `notes_fts`). FTS5 already maintains the term dictionary
	// incrementally as notes change — no build step, no progress bar, no
	// batch loop. We just read when the data is ready.
	//
	// Triggered on `graphReady` so the first read doesn't fight the boot
	// snapshot IPC queue. Subsequent universe switches trigger a re-read.
	// One `invoke` round-trip, tens of milliseconds on a 50k-term Universe.
	$effect(() => {
		if (!graphReady) return;
		if ($libraries.length === 0) return;
		const key = `${activeUniverseName}|${$libraries.length}`;
		if (indexLoadedKey === key) return;
		if (indexLoading) return;

		indexLoading = true;
		(async () => {
			try {
				allIndexEntries = await readIndexEntries();
				indexLoadedKey = key;
			} catch {
				/* leave allIndexEntries as-is */
			} finally {
				indexLoading = false;
			}
		})();
	});

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
		try {
			// Step 1: Pick folder
			const folderPath: string | null = await invoke('pick_folder');
			if (!folderPath) { adding = false; return; }
			// Step 2: Add as a compatible library — ALWAYS. External files
			// (existing Obsidian vaults, etc.) are never renamed on import.
			// We only inject a `cid` property into the frontmatter of each
			// markdown note so Constellation's Living Link system has a
			// stable identifier, leaving the filename and the rest of the
			// vault exactly as the user had it.
			pendingLibraryPath = folderPath;
			await handleKeepIntact();
		} catch (e) { error = String(e); adding = false; }
	}

	// Deprecated: external library canonicalization is disabled entirely.
	// Constellation MUST NOT rename files in libraries the user imports or
	// links to the app. External files keep their original filenames; a
	// `cid` property is added to each markdown note's frontmatter so the
	// Living Link system has a stable identifier without touching the vault.
	// Redirects to the compatible-mode path. Kept as a stub so any stale
	// call-site (e.g. the old CanonicalChoiceDialog) behaves identically
	// to keep-intact. Will be removed once the dialog is too.
	async function handleCanonicalAdopt() {
		await handleKeepIntact();
	}

	async function handleKeepIntact() {
		showCanonicalChoice = false;
		try {
			// Register library with "compatible" mode (the only mode for external
			// vaults). No bulk writes to the vault on import — the Living Link
			// identifier (cid_cn) is injected lazily on a per-note basis the first
			// time Constellation actually opens a note. This keeps the user's
			// filesystem untouched until the note is genuinely accessed.
			await invoke('add_library', { path: pendingLibraryPath });

			await loadLibraries();
			await loadAllStats();
			await refreshLibraryCaches();
			initSearchIndex().then(() => { searchEngineReady = true; }).catch(() => {});
		} catch (e) { error = String(e); }
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
			initSearchIndex().then(() => { searchEngineReady = true; }).catch(() => {});
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

	// Trigger semantic indexing when enabled, search DB ready, and notes loaded
	// Engine lazy-loads on first embed call — no eager init needed
	let semanticIndexTriggered = false;
	$effect(() => {
		const enabled = $appSettings.enabledFeatures?.semanticSearch;
		const notesReady = allNotes.length > 0;
		const dbReady = searchEngineReady;
		if (enabled && dbReady && notesReady && !semanticIndexTriggered && !semanticIndexing) {
			// Check how many are already embedded
			embeddingStatus().then(status => {
				if (status.embedded_count < allNotes.length) {
					semanticIndexTriggered = true;
					console.log(`[Semantic] Indexing: ${status.embedded_count}/${allNotes.length} embedded. Starting...`);
					startSemanticIndexing();
				} else {
					console.log(`[Semantic] All ${allNotes.length} notes already embedded.`);
				}
			}).catch(() => {});
		}
	});

	async function startSemanticIndexing() {
		if (semanticIndexing || allNotes.length === 0) return;
		semanticIndexing = true;
		semanticIndexProgress = 'Loading notes for embedding...';
		try {
			// Load note contents for embedding — batch in chunks to avoid memory spike
			const CHUNK = 50;
			for (let i = 0; i < allNotes.length; i += CHUNK) {
				const chunk = allNotes.slice(i, i + CHUNK);
				const notesWithContent = await Promise.all(
					chunk.map(async (n) => {
						try {
							const content: string = await invoke('read_note', { filePath: n.path });
							return { path: n.path, name: n.name, content };
						} catch { return { path: n.path, name: n.name, content: '' }; }
					})
				);
				const done = await embedNotes(notesWithContent);
				semanticIndexProgress = `Embedding ${Math.min(i + CHUNK, allNotes.length)}/${allNotes.length} notes`;
			}
		} catch (err) { console.error('[Semantic] Indexing failed:', err); }
		semanticIndexing = false;
		semanticIndexProgress = '';
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

			// MIG-006 §1: resolve the OLD human title BEFORE the rename.
			// Pre-§1, this line was `oldPath.split(...).pop()?.replace('.md','')`,
			// which for canonical-filename notes (e.g. `20260424T054559Z_NOTE_C3A4.md`)
			// produced the canonical stem — NEVER the human title — and the
			// downstream `update_links_on_rename` walker would silently match
			// nothing in any source body. The cascade has been dead for every
			// canonical-named note since canonical filenames shipped.
			//
			// Now: pull from the open tab if present (zero IPC), fall back to
			// the new `read_note_title` IPC for closed notes, fall back to the
			// filename stem only for legacy human-named notes without `title:`.
			const oldName = isDir
				? (oldPath.split(/[\\/]/).pop() ?? '')
				: await getOldTitleForCascade(oldPath);

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
		// Ctrl+click or middle-click: open in a real tab, hide Index but keep state for return
		if (e && (e.ctrlKey || e.metaKey || e.button === 1)) {
			showIndex = false;
			indexReturnPending = true; // show "Return to Index" button
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
			<button class="dock-btn" class:active={showSearchHub} onclick={() => {
				showSearchHub = !showSearchHub;
				if (showSearchHub) {
					searchHubInitialQuery = '';
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; lensActive = false; showInspector360 = false;
				}
				searchHubReturnPending = false;
				/* fullPageActive $effect handles sidebar snapshot on entry/exit */
			}} title={$t('searchHub.title')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			{#if $appSettings.enabledFeatures?.orgChart !== false}
			<button class="dock-btn" class:active={showOrgChart} onclick={() => {
				showOrgChart = !showOrgChart;
				orgChartReturnPending = false;
				if (showOrgChart) {
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showInspector360 = false;
				}
				/* fullPageActive $effect handles sidebar snapshot */
			}} title={$t('navigator.orgChart') || 'Organization Chart'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="8" y="2" width="8" height="5" rx="1"/><rect x="1" y="17" width="8" height="5" rx="1"/><rect x="15" y="17" width="8" height="5" rx="1"/><path d="M12 7v4"/><path d="M5 17v-2h14v2"/></svg>
			</button>
			{/if}
			<button class="dock-btn" class:active={showKnowledgeHealth} onclick={() => {
				showKnowledgeHealth = !showKnowledgeHealth;
				if (showKnowledgeHealth) {
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showInspector360 = false;
				}
			}} title={$t('ribbon.knowledgeHealth') || 'Knowledge Health'}>
				<SlotIcon slot="dock.knowledgeHealth">
					{#snippet children()}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
							<path d="M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z"/>
							<path d="M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z"/>
							<path d="M15 13a4.5 4.5 0 0 1-3-4"/>
							<path d="M12 5v13"/>
						</svg>
					{/snippet}
				</SlotIcon>
			</button>
			{#if $appSettings.enabledFeatures?.skyView !== false}
			<button class="dock-btn" class:active={showSkyView} onclick={() => { showSkyView = !showSkyView; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showKnowledgeHealth = false; showInspector360 = false; }} title={$t('ribbon.graphView') || 'Sky View'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.dailyNotes !== false}
			<button class="dock-btn" onclick={handleOpenDailyNote} title={$t('ribbon.dailyNote')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.aiSkills !== false}
			<a href="/skills" class="dock-btn" class:active={page.url.pathname === '/skills'} title={$t('ribbon.aiSkills')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01z"/></svg>
			</a>
			{/if}
			{#if $appSettings.enabledFeatures?.index !== false}
			<button class="dock-btn" class:active={showIndex} onclick={() => { showIndex = !showIndex; showSkyView = false; showGlobalTasks = false; showConstellationMap = false; showInspector360 = false; indexReturnPending = false; }} title={$t('ribbon.index')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/><path d="M8 7h6"/><path d="M8 11h8"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.constellationMap === true}
			<button class="dock-btn" class:active={showConstellationMap} onclick={() => { showConstellationMap = !showConstellationMap; showSkyView = false; showGlobalTasks = false; showIndex = false; showInspector360 = false; mapReturnPending = false; }} title={$t('ribbon.constellationMap') || 'Constellation Map'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.constellationSight !== false}
			<button class="dock-btn" class:active={lensActive} onclick={() => {
				if (!lensActive) {
					toggleLens(); showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showInspector360 = false; lensReturnPending = false;
				} else {
					lensActive = false;
				}
				/* fullPageActive $effect handles sidebar snapshot */
			}} title={$t('lens.title') || 'Constellation Sight'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.inspector360 !== false}
			<button class="dock-btn" class:active={showInspector360} onclick={() => {
				showInspector360 = !showInspector360;
				if (showInspector360) {
					showSkyView = false; showGlobalTasks = false; showIndex = false;
					showConstellationMap = false; showOrgChart = false; lensActive = false;
					showKnowledgeHealth = false; showSearchHub = false;
					showExpressionForge = false; showSenseMakingCanvas = false;
				}
			}} title={$t('ribbon.inspector360') || $t('inspector360.title') || '360° Inspector'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="3"/><line x1="12" y1="3" x2="12" y2="9"/><line x1="12" y1="15" x2="12" y2="21"/><line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/></svg>
			</button>
			{/if}
		</div>
		<div class="dock-bottom">
			{#if hasMultipleDisplays && $appSettings.enabledFeatures?.secondScreen !== false}
				<button class="dock-btn" class:active={secondScreenOpen} onclick={handleToggleSecondScreen} title={$t('secondScreen.title')}>
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="1" y="2" width="14" height="10" rx="1.5" fill="var(--background-secondary, #1e1e2e)"/><rect x="9" y="10" width="14" height="10" rx="1.5" fill="var(--background-secondary, #1e1e2e)"/></svg>
				</button>
			{/if}
<button class="dock-btn" class:active={showSettings} onclick={() => showSettings = !showSettings} title={$t('ribbon.settings')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
			</button>
		</div>
	</div>

	<!-- ═══ LEFT SIDEBAR ═══ -->
	{#if sidebarOpen && !skyViewInspectMode}
		<aside class="sidebar" style:width="{leftSidebarWidth}px">
			<div class="sidebar-toolbar">
				<!-- Sidebar search removed — Search Hub is the single search experience -->
				<!-- Row 1: New Elements — always visible -->
				<div class="toolbar-actions new-elements" style="position:relative">
					<button class="tb-btn" onclick={handleNewNote} title={$t('sidebar.newNote')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M12 18v-6"/><path d="M9 15h6"/></svg>
					</button>
					<button class="tb-btn" onclick={handleNewBase} title={$t('sidebar.newBase')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
					</button>
					<button class="tb-btn" onclick={handleNewFolder} title={$t('sidebar.newFolder')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 10v6"/><path d="M9 13h6"/><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>
					</button>
					<button class="tb-btn" onclick={handleNewLibrary} title={$t('sidebar.newLibrary')}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/><path d="M12 10v4"/><path d="M10 12h4"/></svg>
					</button>

					<!-- New Library dropdown -->
					{#if showNewLibraryDropdown}
						<div class="new-lib-drop">
							<div class="nld-option">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14"/><path d="M5 12h14"/></svg>
								<div class="nld-text">
									<span class="nld-title">{$t('libraries.newLibrary')}</span>
									<div class="nld-input-row">
										<input class="nld-input" type="text" dir="auto" placeholder={$t('libraries.newLibrary')}
											bind:value={newLibName}
											onkeydown={(e) => { if (e.key === 'Enter') handleCreateNewLib(); if (e.key === 'Escape') showNewLibraryDropdown = false; }} />
										<button class="nld-create" onclick={handleCreateNewLib}>+</button>
									</div>
								</div>
							</div>
							<button class="nld-option" onclick={async () => { showNewLibraryDropdown = false; await handleAddLibrary(); }}>
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
								<div class="nld-text">
									<span class="nld-title">{$t('libraries.linkLibrary')}</span>
									<span class="nld-desc">{$t('libraries.linkLibraryDesc')}</span>
								</div>
							</button>
						</div>
					{/if}
				</div>
				<!-- Row 2: Notes Management — always visible, even during search -->
				<div class="toolbar-modes notes-management">
					<button class="mode-tab" class:active={sidebarMode === 'tree'} onclick={() => { if (sidebarMode !== 'tree') { sidebarMode = 'tree'; leftSidebarWidth = calcContentWidth(100); emitSidebarModeChanged('tree'); } }} title={$t('navigator.fileExplorer') || 'File Explorer'}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
					</button>
					{#if $appSettings.enabledFeatures?.notesNavigator !== false}
					<button class="mode-tab" class:active={sidebarMode === 'list'} onclick={() => { if (sidebarMode !== 'list') { if (sidebarMode === 'tree') preTreeWidth = leftSidebarWidth; sidebarMode = 'list'; leftSidebarWidth = Math.max(leftSidebarWidth, 450); emitSidebarModeChanged('list'); } }} title={$t('navigator.notesNavigator') || 'Notes Navigator'}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="18" rx="1"/><rect x="14" y="3" width="7" height="18" rx="1"/></svg>
					</button>
					{/if}
					<!-- OrgChart and Sky View buttons moved to left dock bar -->
					{#if sidebarMode === 'tree' }
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
					</div>
				</div>

			<div class="sidebar-content">
				{#if sidebarMode === 'list'}
					<NotebookNavigator
						mode="main"
						{libraryColorMap}
						initialTags={allLibraryTags}
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

					{#if $libraries.length === 0 && librariesLoaded}
						<div class="empty-sidebar">
							<p>{$t('sidebar.noLibraries')}</p>
							<button class="add-first-btn" onclick={handleAddLibrary}>{$t('sidebar.addLibraryButton')}</button>
						</div>
					{:else if !librariesLoaded}
						<div class="empty-sidebar">
							<div class="loading-spinner" aria-label="Loading libraries"></div>
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
		<!-- Layout bar: sidebar + split controls (disabled when full-page overlay active) -->
		<div class="layout-bar">
			<button class="tab-action" class:active={sidebarOpen} disabled={layoutCtrlDisabled} onclick={() => sidebarOpen = !sidebarOpen} title={layoutCtrlDisabled ? layoutCtrlDisabledReason : $t('layout.leftSidebar')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/></svg>
			</button>
			<div style="flex:1"></div>
			<button class="tab-action" class:active={$splitActive} disabled={layoutCtrlDisabled} onclick={cycleSplit} title={layoutCtrlDisabled ? layoutCtrlDisabledReason : $t('layout.splitView')}>
				{#if $splitActive && $splitDirection === 'horizontal'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 12h18"/></svg>
				{:else}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 3v18"/></svg>
				{/if}
			</button>
			<button class="tab-action" class:active={rightSidebarOpen} disabled={layoutCtrlDisabled} onclick={() => rightSidebarOpen = !rightSidebarOpen} title={layoutCtrlDisabled ? layoutCtrlDisabledReason : $t('layout.rightSidebar')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M15 3v18"/></svg>
			</button>
		</div>

		<!-- Tab bar (locked to paper, hidden when full-screen overlay is active).
		     When SV inspect mode is active the pane-container below gets flanked
		     by Backlinks/Outgoing columns; padding-inline here shifts the tab
		     strip so it aligns with the center (editor) column instead of sitting
		     flush against the screen edge. Padding respects per-side placement;
		     uses logical CSS so RTL flips automatically.
		     `tabBarFlankLeft` / `tabBarFlankRight` derived in the script above. -->
		<div class="tab-bar" class:tab-bar-hidden={fullPageActive} class:tab-bar-flanked-start={tabBarFlankLeft} class:tab-bar-flanked-end={tabBarFlankRight}>
			{#if indexReturnPending}
				<button class="index-return-btn" onclick={() => { showIndex = true; indexReturnPending = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('indexPanel.returnToIndex') || 'Return to Index'}
				</button>
			{/if}
			{#if mapReturnPending}
				<button class="index-return-btn" onclick={() => { showConstellationMap = true; mapReturnPending = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('constellationMap.returnToMap') || 'Return to Map'}
				</button>
			{/if}
			{#if orgChartReturnPending}
				<button class="index-return-btn" onclick={() => { showOrgChart = true; orgChartReturnPending = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('orgChart.returnToOrgChart') || 'Return to OrgChart'}
				</button>
			{/if}
			{#if lensReturnPending}
				<button class="index-return-btn" onclick={() => { lensActive = true; lensReturnPending = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('lens.returnToLens') || 'Return to Lens'}
				</button>
			{/if}
			{#if searchHubReturnPending}
				<button class="index-return-btn" onclick={() => { showSearchHub = true; searchHubReturnPending = false; showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; lensActive = false; showInspector360 = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('searchHub.title')}
				</button>
			{/if}
			{#if skyViewReturnPending}
				<button class="index-return-btn" onclick={() => { showSkyView = true; skyViewReturnPending = false; showSearchHub = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; lensActive = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('layout.skyViewTitle') || 'Sky View'}
				</button>
			{:else if skyViewInspectMode && $activeTab?.path && !showSkyView && !fullPageActive && ($appSettings.enabledFeatures?.skyView ?? true)}
				<!-- "Return to Sky View" pill — only visible while in SV inspect mode
				     (user arrived at this note by clicking a Sky-View node). Pair of
				     buttons: clicking the main body returns to SV, clicking × exits
				     inspect mode entirely (flanks hide, pill disappears). -->
				<span class="sv-pill-group">
					<button class="index-return-btn sv-return-pill" onclick={() => { showSkyView = true; }} title={$t('layout.skyViewTitle') || 'Sky View'}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
						{$t('layout.skyViewTitle') || 'Sky View'}
					</button>
					<button class="sv-pill-dismiss" onclick={() => {
						popSidebars('skyInspect');
						skyViewInspectMode = false;
					}} title={$t('layout.exitSkyViewMode') || 'Exit Sky View inspect mode'}>×</button>
				</span>
			{/if}
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
					{#each [$openTabs.find(t => t.id === tabCtxMenu!.tabId)] as ctxTab}
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

		<!-- Index (always rendered, hidden with CSS to preserve state) -->
		<div class="index-overlay" class:index-visible={showIndex}>
			<div class="index-split" class:has-note={indexNoteTab}>
				{#if indexNoteTab}
					<div class="index-note-pane">
						<div class="index-note-header">
							<span class="index-note-name" dir="auto">{indexNoteTab.name}</span>
							<button class="index-close" onclick={() => { indexNoteTab = null; indexActiveNotePath = ''; }} title="Close note">×</button>
						</div>
						<NoteEditor tab={indexNoteTab} noteNames={allNotes} allTags={allTagsList} {linkTraversalMap} />
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
							isLoading={indexLoading}
							onNoteClick={handleIndexNoteClick}
							loadMentions={(term) => readTermMentions(term, 500)}
							loadCooccurrence={(term) => readCooccurringTerms(term)}
							onNoteHover={handleIndexNoteHover}
							onNoteLeave={handleIndexNoteLeave}
							activeNotePath={indexActiveNotePath}
							selectedTerms={indexSelectedTerms}
							onTermClick={(term, mentions) => {
								if (secondScreenOpen) {
									emitIndexTermSelected({ term, notes: mentions });
								}
							}}
							onTermSelect={(term, mentions, selected) => {
								const next = new Set(indexSelectedTerms);
								if (selected) { next.add(term); } else { next.delete(term); }
								indexSelectedTerms = next;
								if (secondScreenOpen) {
									const terms = allIndexEntries
										.filter(e => next.has(e.term))
										.map(e => ({ term: e.term, notes: e.mentions }));
									emitIndexCompare({ terms });
								}
							}}
						/>
					</div>
				</div>
			</div>
		</div>

		<!-- Constellation Map — lazy-mounted (LL-022). -->
		{#if mapEverOpened}
			<div class="map-overlay" class:map-visible={showConstellationMap}>
				<ConstellationMap
					universeName={activeUniverseName}
					libraryPath={get(libraries)[0]?.path ?? ''}
					libraryName={get(libraries)[0]?.name ?? ''}
					libraryColor={libraryColorMap[get(libraries)[0]?.name ?? ''] ?? '#7c3aed'}
					{libraryColorMap}
					onNoteClick={(path, name) => {
						const lib = $libraryStats.find(l => path.startsWith(l.path));
						if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
						showConstellationMap = false;
						mapReturnPending = true;
						if (secondScreenOpen) {
							emitMapCompanion({ active: true, colorMode: mapColorMode, focusNode: mapFocusNode, parentNode: null, clickedNote: { path, name, libraryName: lib?.name ?? '', libraryPath: lib?.path ?? '' } });
						}
					}}
					onDrillDown={(node, bcNames) => {
						mapFocusNode = node;
						if (secondScreenOpen && showConstellationMap) {
							emitMapCompanion({ active: true, colorMode: mapColorMode, focusNode: node, parentNode: null, clickedNote: null });
						}
					}}
					onColorModeChange={(mode) => {
						mapColorMode = mode as any;
						if (secondScreenOpen && showConstellationMap) {
							emitMapCompanion({ active: true, colorMode: mode as any, focusNode: mapFocusNode, parentNode: null, clickedNote: null });
						}
					}}
					onClose={() => {
						showConstellationMap = false;
						mapReturnPending = false;
						if (secondScreenOpen) {
							emitMapCompanion({ active: false, colorMode: mapColorMode, focusNode: null, parentNode: null, clickedNote: null });
						}
					}}
				/>
			</div>
		{/if}

		<!-- 360° Inspector overlay — lazy-mounted (LL-022). CE Phase 12. -->
		{#if inspector360EverOpened}
			<div class="inspector360-overlay" class:inspector360-visible={showInspector360}>
				<Inspector360
					data={inspector360Data}
					compact={false}
					onNoteClick={(path, name) => {
						const lib = $libraryStats.find(l => path.startsWith(l.path));
						if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
						showInspector360 = false;
					}}
					onClose={() => { showInspector360 = false; }}
				/>
			</div>
		{/if}

		<!-- OrgChart overlay — lazy-mounted (LL-022). -->
		{#if orgChartEverOpened}
			<div class="orgchart-overlay" class:orgchart-visible={showOrgChart}>
				<OrgChart
					universeName={activeUniverseName}
					{libraryColorMap}
					fullscreen={true}
					onNoteClick={(path, name) => {
						const lib = $libraryStats.find(l => path.startsWith(l.path));
						if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
						showOrgChart = false;
						orgChartReturnPending = true;
					}}
					onClose={() => { showOrgChart = false; orgChartReturnPending = false; }}
				/>
			</div>
		{/if}

		<!-- Constellation Lens — standalone D3+Canvas component -->
		<div class="lens-overlay" class:lens-visible={lensActive}>
			{#if lensActive}
				<ConstellationSight
					nodes={skyNodes}
					links={skyLinks}
					centrality={lensCentrality}
					communityAssignments={lensCommunityAssignments}
					communityColors={new Map(lensCommunities.map(c => [c.id, c.color]))}
					gaps={lensGaps}
					health={lensHealth}
					bridges={lensBridges}
					communities={lensCommunities}
					communityProfiles={lensCommunityProfiles}
					contradictions={lensContradictions}
					{libraryColorMap}
					searchMatchIds={searchHubMatchIds}
					onNoteClick={(path, name, highlightTerm) => {
						const lib = $libraryStats.find(l => path.startsWith(l.path));
						if (lib) {
							let hl = highlightTerm || '';
							hl = hl.replace(/(?:links?\s+(?:to|from|between|all)|mutual|mentions?|supports|contradicts|causes|exemplifies|generalizes|derives[- ]from|part[- ]of)\s*/gi, '');
							const hlFinal = hl.replace(/\[\[|\]\]/g, '').replace(/^\s*#/, '').trim() || undefined;
							openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed', hlFinal);
						}
						lensActive = false;
						lensReturnPending = true;
					}}
					onClose={() => { lensActive = false; lensReturnPending = false; }}
				/>
			{/if}
		</div>

		<!-- Search Hub overlay -->
		<div class="searchhub-overlay" class:searchhub-visible={showSearchHub}>
			<SearchHub
				initialQuery={searchHubInitialQuery}
				{allNotes}
				linkCounts={searchLinkCounts}
				onNoteClick={(path: string, name: string, libraryName: string, hubQuery: string) => {
					const libraryColor = libraryColorMap[libraryName] ?? '#7c3aed';
					// Strip operator syntax from query for clean highlighting
					let hl = hubQuery || '';
					hl = hl.replace(/(?:links?\s+(?:to|from|between|all)|mutual|mentions?)\s*/gi, '');
					hl = hl.replace(/\[\[|\]\]/g, '');
					hl = hl.replace(/\band\b/gi, ',');
					hl = hl.replace(/^\s*#/, '');
					hl = hl.replace(/^\s*in:\S+\s*/, '');
					hl = hl.replace(/^\s*\S+=\S+\s*/, '');
					const hlFinal = hl.trim() || undefined;
					openNoteTab(path, libraryName, libraryColor, hlFinal);
					showSearchHub = false;
					searchHubReturnPending = true;
				}}
				onClose={() => {
					showSearchHub = false;
					searchHubReturnPending = false;
				}}
				onResults={(ids: Set<string>) => { searchHubMatchIds = ids.size > 0 ? ids : null; }}
			/>
		</div>

		<!-- Content -->
		<div class="content-area" class:content-hidden={showIndex || showConstellationMap || showOrgChart || lensActive || showSearchHub || showInspector360} onmouseover={handleWikilinkHover} onmouseout={handleWikilinkLeave}>
			{#if showSkyView}
				<div class="star-fullscreen">
					<div class="star-header">
						<span class="star-title">{$t('layout.skyViewTitle')}</span>
						<button class="star-wiw-toggle" class:active={wiwEnabled} onclick={() => { wiwEnabled = !wiwEnabled; if (!wiwEnabled) showWiW = false; }} title="Window in Window">
							<svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
								<rect x="0.75" y="1.75" width="12.5" height="9" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
								<rect x="7" y="5" width="5" height="3.5" rx="0.75" fill="currentColor"/>
							</svg>
						</button>
						<button class="star-close" onclick={() => showSkyView = false}>×</button>
					</div>
					<GraphMindView
					nodes={skyNodes}
					links={skyLinks}
					onNodeClick={handleSkyNodeClick}
					onRequestEnrichment={async () => {
						// Triggered from GraphMindView's legend empty-state
						// when the user picks Stratum/Maturity but the data
						// was never computed (per-boot compute removed for
						// boot-perf — see enrichNodesBackground comment).
						// Runs the full enrichment pass once on demand so
						// the legend populates. skyNodes is mutated in-place;
						// the svelte reactivity tripwire (skyVersion bump
						// inside enrichNodesBackground's yield points) lets
						// the graph reactive updates propagate.
						await enrichNodesBackground($libraries);
					}}
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
					searchMatchIds={searchHubMatchIds}
					{allNotes}
				/>
				{#if !graphReady}
					<div class="sky-loading" role="status" aria-live="polite" dir="auto">
						<svg class="sky-loading-spinner" width="16" height="16" viewBox="0 0 24 24" aria-hidden="true">
							<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2.5" fill="none" opacity="0.25"/>
							<path d="M22 12a10 10 0 0 1-10 10" stroke="currentColor" stroke-width="2.5" fill="none" stroke-linecap="round"/>
						</svg>
						<span>{$t('layout.skyViewLoading')}</span>
					</div>
				{/if}
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
							<span class="wiw-subtitle">{$t('layout.skyViewWiWHint')}</span>
						</div>
						<div class="wiw-graph">
							<GraphMindView
								nodes={wiwFilteredNodes}
								links={wiwFilteredLinks}
								libraryColorMap={wiwLibraryColorMap}
								onNodeClick={handleSkyNodeClick}
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
					notes={skyNodes}
					{activeTrail}
					libraryPath={get(libraries)[0]?.path ?? ''}
					libraryName={get(libraries)[0]?.name ?? ''}
					onClose={() => showExpressionForge = false}
				/>
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
								<NoteEditor
									{tab}
									noteNames={allNotes}
									allTags={allTagsList}
									{linkTraversalMap}
									onnavigateback={() => { setFocusedTab(tab.id); navigateBack(); }}
									onnavigateforward={() => { setFocusedTab(tab.id); navigateForward(); }}
									onStageChanged={(path, stage) => {
										const key = path.replace(/\\/g, '/').toLowerCase();
										const nm = new Map(stageMap);
										if (stage) { nm.set(key, stage); } else { nm.delete(key); }
										stageMap = nm;
									}}
								/>
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
						{#if focusMode}
							{@const _parsed = parseFrontmatter($activeTab.content || '')}
							<FocusPane
								value={_parsed.body}
								title={$activeTab.name.replace(/\.md$/, '')}
								dir={noteDir}
								onchange={(text) => {
									const currentTab = get(openTabs).find(x => x.id === $activeTab!.id);
									const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
									const fc = buildFullContent(props, text);
									if (currentTab) currentTab.content = fc;
									markRecentWrite($activeTab!.path);
									writeNote($activeTab!.path, fc).catch(() => {});
								}}
								onexit={(promote) => {
									focusMode = false;
									if (promote) {
										const currentTab = get(openTabs).find(x => x.id === $activeTab!.id);
										const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
										const body = currentTab ? parseFrontmatter(currentTab.content || '').body : _parsed.body;
										let updated = false;
										const newProps = props.map(p => {
											if (p.key.toLowerCase() === 'stage') { updated = true; return { ...p, value: promote }; }
											return p;
										});
										if (!updated) newProps.push({ key: 'stage', value: promote, type: 'text' as any });
										const fc = buildFullContent(newProps, body);
										if (currentTab) { currentTab.content = fc; openTabs.update(tabs => tabs); }
										markRecentWrite($activeTab!.path);
										writeNote($activeTab!.path, fc).catch(() => {});
									}
								}}
							/>
						{:else}
							<!--
								"Note as organism" — Tier 1 flanking panels.
								Backlinks placed on logical-left, Outgoing on
								logical-right, both user-configurable via
								appSettings.panelPlacements. Flanks hide in
								focus mode (handled above) and split mode
								(the {$splitActive} branch renders a different
								code path that doesn't reach here). RTL is
								handled by the dir attribute on the wrapper:
								flex-direction:row + dir:rtl reverses the
								visual order so Backlinks stays on the reading-
								start side.
							-->
							<!-- Flanks only render in SV "inspect mode" — when the user arrived
							     by clicking a Sky-View node. Regular navigation shows the plain
							     editor. See skyViewInspectMode declaration for lifecycle. -->
							{@const backlinksOnLeft  = skyViewInspectMode && $appSettings.panelPlacements?.backlinks === 'left-of-note'}
							{@const backlinksOnRight = skyViewInspectMode && $appSettings.panelPlacements?.backlinks === 'right-of-note'}
							{@const outgoingOnLeft   = skyViewInspectMode && $appSettings.panelPlacements?.outgoing === 'left-of-note'}
							{@const outgoingOnRight  = skyViewInspectMode && $appSettings.panelPlacements?.outgoing === 'right-of-note'}
							<div class="editor-with-flanks" class:flank-resizing={flankResizing !== null} dir={$dir}>
								{#if backlinksOnLeft || outgoingOnLeft}
									<div class="flank flank-start"
										class:flank-collapsed={leftFlankCollapsed}
										style:flex-basis="{leftFlankCollapsed ? 0 : leftFlankWidth}px">
										{#if !leftFlankCollapsed}
											{#if backlinksOnLeft}
												<BacklinksPanel
													backlinks={currentBacklinks}
													unlinkedMentions={currentUnlinkedMentions}
													activeNoteName={$activeTab?.name ?? ''}
													activeNotePath={$activeTab?.path ?? ''}
													{libraryColorMap}
													onConfidenceChange={applyConfidenceLocally}
													onArchive={applyArchiveLocally}
												/>
											{/if}
											{#if outgoingOnLeft}
												<OutgoingLinksPanel
													outgoingLinks={currentOutgoing}
													activeNotePath={$activeTab?.path ?? ''}
													libraryPath={$activeTab?.libraryPath ?? ''}
													{libraryColorMap}
													onConfidenceChange={applyConfidenceLocally}
													onArchive={applyArchiveLocally}
												/>
											{/if}
										{/if}
									</div>
									<!-- Drag handle + collapse toggle between left flank and center editor -->
									<div class="flank-handle-wrap flank-handle-wrap-start">
										<button class="flank-collapse-btn"
											aria-label={leftFlankCollapsed ? 'Expand left panel' : 'Collapse left panel'}
											title={leftFlankCollapsed ? 'Expand' : 'Collapse'}
											onclick={() => leftFlankCollapsed = !leftFlankCollapsed}>
											<!-- Chevron points right when collapsed (show), left when expanded (hide) -->
											{#if $dir === 'rtl'}
												<svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
													<path d={leftFlankCollapsed ? 'M7 5L3 1v8z' : 'M3 5l4-4v8z'}/>
												</svg>
											{:else}
												<svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
													<path d={leftFlankCollapsed ? 'M3 5l4-4v8z' : 'M7 5L3 1v8z'}/>
												</svg>
											{/if}
										</button>
										<div
											class="flank-handle flank-handle-start"
											role="separator"
											aria-label="Resize left panel"
											onmousedown={(e) => !leftFlankCollapsed && startFlankResize('left', e)}
										></div>
									</div>
								{/if}
								<div class="flank-center">
							<NoteEditor
								tab={$activeTab}
								noteNames={allNotes}
								allTags={allTagsList}
								{linkTraversalMap}
								trail={activeTrail ? activeTrail.title : ''}
								{trailIndex}
								trailTotal={activeTrail ? activeTrail.notes.length : 0}
								onTrailPrev={async () => {
									if (activeTrail && trailIndex > 0) {
										const fromPath = activeTrail.notes[trailIndex]?.path;
										trailIndex--;
										const note = activeTrail.notes[trailIndex];
										if (note.exists) {
											const lib = get(libraries)[0];
											if (lib) await openNoteTab(note.path, lib.name, libraryColorMap[lib.name] || '#7c3aed', undefined, false, fromPath);
										}
									}
								}}
								onTrailNext={async () => {
									if (activeTrail && trailIndex < activeTrail.notes.length - 1) {
										const fromPath = activeTrail.notes[trailIndex]?.path;
										trailIndex++;
										const note = activeTrail.notes[trailIndex];
										if (note.exists) {
											const lib = get(libraries)[0];
											if (lib) await openNoteTab(note.path, lib.name, libraryColorMap[lib.name] || '#7c3aed', undefined, false, fromPath);
										}
									}
								}}
								onnavigateback={() => navigateBack()}
								onnavigateforward={() => navigateForward()}
								onStageChanged={(path, stage) => {
									const key = path.replace(/\\/g, '/').toLowerCase();
									const nm = new Map(stageMap);
									if (stage) { nm.set(key, stage); } else { nm.delete(key); }
									stageMap = nm;
								}}
								onmoreaction={async (action) => {
									switch (action) {
										case 'rename': {
											const input = document.querySelector('.e-title') as HTMLInputElement;
											if (input) { input.focus(); input.select(); }
											break;
										}
										case 'revealInTree':
											window.dispatchEvent(new CustomEvent('constellation:reveal-in-tree', { detail: { path: $activeTab!.path } }));
											break;
										case 'delete':
											window.dispatchEvent(new CustomEvent('constellation:delete-note', { detail: { path: $activeTab!.path, name: $activeTab!.name } }));
											break;
										case 'addProperty':
											window.dispatchEvent(new CustomEvent('constellation:add-property', { detail: { path: $activeTab!.path } }));
											break;
										case 'switchToFocus':
											focusMode = true;
											break;
									}
								}}
							/>
								</div>
								{#if backlinksOnRight || outgoingOnRight}
									<!-- Drag handle + collapse toggle between center editor and right flank -->
									<div class="flank-handle-wrap flank-handle-wrap-end">
										<div
											class="flank-handle flank-handle-end"
											role="separator"
											aria-label="Resize right panel"
											onmousedown={(e) => !rightFlankCollapsed && startFlankResize('right', e)}
										></div>
										<button class="flank-collapse-btn"
											aria-label={rightFlankCollapsed ? 'Expand right panel' : 'Collapse right panel'}
											title={rightFlankCollapsed ? 'Expand' : 'Collapse'}
											onclick={() => rightFlankCollapsed = !rightFlankCollapsed}>
											{#if $dir === 'rtl'}
												<svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
													<path d={rightFlankCollapsed ? 'M3 5l4-4v8z' : 'M7 5L3 1v8z'}/>
												</svg>
											{:else}
												<svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
													<path d={rightFlankCollapsed ? 'M7 5L3 1v8z' : 'M3 5l4-4v8z'}/>
												</svg>
											{/if}
										</button>
									</div>
									<div class="flank flank-end"
										class:flank-collapsed={rightFlankCollapsed}
										style:flex-basis="{rightFlankCollapsed ? 0 : rightFlankWidth}px">
										{#if !rightFlankCollapsed}
											{#if outgoingOnRight}
												<OutgoingLinksPanel
													outgoingLinks={currentOutgoing}
													activeNotePath={$activeTab?.path ?? ''}
													libraryPath={$activeTab?.libraryPath ?? ''}
													{libraryColorMap}
													onConfidenceChange={applyConfidenceLocally}
													onArchive={applyArchiveLocally}
												/>
											{/if}
											{#if backlinksOnRight}
												<BacklinksPanel
													backlinks={currentBacklinks}
													unlinkedMentions={currentUnlinkedMentions}
													activeNoteName={$activeTab?.name ?? ''}
													activeNotePath={$activeTab?.path ?? ''}
													{libraryColorMap}
													onConfidenceChange={applyConfidenceLocally}
													onArchive={applyArchiveLocally}
												/>
											{/if}
										{/if}
									</div>
								{/if}
							</div>
						{/if}
					{/if}
				</div>
			{:else if isHome}
				<div class="welcome" class:welcome-dashboard={$libraries.length > 0 && $appSettings.showDashboard}>
					{#if !librariesLoaded}
						<!-- Hydration window: spinner, not the Create-Library screen. -->
						<div class="app-loading" style="min-height: 400px;">
							<div class="loading-spinner"></div>
						</div>
					{:else if $libraries.length === 0}
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
						{#if $appSettings.showDashboard}
							<div class="home-dashboard">
								<div class="home-dashboard-header">
									<button class="home-dashboard-toggle" onclick={() => updateSettings({ showDashboard: false })} title={$t('dashboard.hide') || 'Hide Dashboard'}>
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
									</button>
								</div>
								<DashboardView
									universeName={activeUniverseName}
									{libraryColorMap}
									onNoteClick={(path, name, libraryName) => openNoteTab(path, libraryName, libraryColorMap[libraryName] || '#7c3aed')}
									onNoteToScreen={(note) => {
										if (secondScreenOpen) {
											emitDashboardOpenNote(note);
										} else {
											openNoteTab(note.path, note.libraryName, note.libraryColor);
										}
									}}
									onTagSelect={secondScreenOpen ? (tag, notes) => {
										emitDashboardTagSelected({ tag, notes });
									} : undefined}
								/>
							</div>
						{:else}
							<p class="w-hint">{$t('welcome.selectNote')}</p>
							<p class="w-hint-sub">{$t('welcome.quickSwitchHint')}</p>
							<button class="w-dashboard-btn" onclick={() => updateSettings({ showDashboard: true })}>
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
								{$t('dashboard.show') || 'Show Dashboard'}
							</button>
						{/if}
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
	<aside class="right-sidebar" class:collapsed={!rightSidebarOpen || skyViewInspectMode} style:width={(rightSidebarOpen && !skyViewInspectMode) ? rightSidebarWidth + 'px' : undefined}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="rs-resize" onmousedown={(e) => startResize('right', e)}></div>
		<div class="rs-inner" dir={noteDir}>
			<!-- Right sidebar tab bar.
			     Each tab button is gated on its panel being placed in 'right-sidebar'.
			     Falls back to showing the tab when no placement is saved (new install),
			     so all tabs remain visible by default. The safety $effect above resets
			     rightSidebarTab if the active tab's panel is moved away. -->
			<div class="rs-tabs">
				{#if ($appSettings.panelPlacements?.properties ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'properties'} onclick={() => rightSidebarTab = 'properties'} title={$t('panels.properties')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>
					</button>
				{/if}
				<!-- Backlinks tab always present in the right sidebar regardless of panelPlacements.
				     Placement controls whether the flanks render next to the editor; the sidebar
				     tab is an alternative access path that stays available for regular navigation
				     (not just in SV inspect mode). -->
				<button class="rs-tab" class:active={rightSidebarTab === 'backlinks'} onclick={() => rightSidebarTab = 'backlinks'} title={$t('panels.backlinks')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
				</button>
				{#if ($appSettings.panelPlacements?.tags ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'tags'} onclick={() => rightSidebarTab = 'tags'} title={$t('panels.tags')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.sky ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'star'} onclick={() => rightSidebarTab = 'star'} title={$t('panels.skyView')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="18" r="3"/><path d="M8.5 8.5l7 7M15.5 8.5l-7 7"/></svg>
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.tasks ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'tasks'} onclick={() => rightSidebarTab = 'tasks'} title={$t('panels.tasks')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.calendar ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'calendar'} onclick={() => rightSidebarTab = 'calendar'} title={$t('panels.calendar')}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.health ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'health'} onclick={() => rightSidebarTab = 'health'} title={$t('panels.health') || 'Knowledge Health'}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.provenance ?? 'right-sidebar') === 'right-sidebar'}
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
				{/if}
				{#if ($appSettings.panelPlacements?.review ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'review'} onclick={() => {
						rightSidebarTab = 'review';
						const lib = get(libraries)[0];
						if (lib) invoke<any[]>('get_due_notes', { libraryPath: lib.path })
							.then(notes => { dueNotes = notes; }).catch(() => { dueNotes = []; });
					}} title={$t('panels.review') || 'Review Pulse'}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
						{#if dueNotes.length > 0}<span class="rs-tab-badge">{dueNotes.length}</span>{/if}
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.links ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'links'} onclick={() => rightSidebarTab = 'links'} title={$t('panels.links') || 'Link Dashboard'}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
					</button>
				{/if}
				{#if ($appSettings.panelPlacements?.inspector360 ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'inspector360'} onclick={() => rightSidebarTab = 'inspector360'} title={$t('panels.inspector360') || $t('inspector360.title') || '360° Inspector'}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="3"/><line x1="12" y1="3" x2="12" y2="9"/><line x1="12" y1="15" x2="12" y2="21"/><line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/></svg>
					</button>
				{/if}
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
				{:else if rightSidebarTab === 'backlinks' && !skyViewInspectMode}
					<!-- Skip mounting in inspect mode: the right sidebar is force-collapsed
					     and cannot be opened, so this would be an invisible mount paying
					     render cost over currentBacklinks/currentOutgoing for nothing. -->
					<div class="rs-section rs-section--flush">
						<div class="rs-header">{$t('panels.backlinksHeader')}</div>
						<BacklinksPanel
							backlinks={currentBacklinks}
							unlinkedMentions={currentUnlinkedMentions}
							activeNoteName={sidebarTab?.name ?? ''}
							activeNotePath={sidebarTab?.path ?? ''}
							{libraryColorMap}
							onConfidenceChange={applyConfidenceLocally}
							onArchive={applyArchiveLocally}
						/>
					</div>
					<div class="rs-section rs-section--flush">
						<div class="rs-header">{$t('panels.outgoingLinksHeader')}</div>
						<OutgoingLinksPanel
							outgoingLinks={currentOutgoing}
							activeNotePath={sidebarTab?.path ?? ''}
							libraryPath={sidebarTab?.libraryPath ?? ''}
							{libraryColorMap}
							onConfidenceChange={applyConfidenceLocally}
							onArchive={applyArchiveLocally}
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
						{#if localSkyNodes.length > 0}
							<LocalSkyView
								nodes={localSkyNodes}
								links={localSkyLinks}
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
										// Notify SS so it can refresh its panels
										if (secondScreenOpen) broadcastNoteSaved(filePath);
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
				{:else if rightSidebarTab === 'links'}
					<div class="rs-section rs-full-height">
						<LinkDashboard
							allLinks={effectiveLibraryLinks}
							allNotes={allNotes}
							visible={rightSidebarOpen && rightSidebarTab === 'links'}
							onNoteClick={(path, libraryName) => {
								openNoteTab(path, libraryName, libraryColorMap[libraryName] || '#7c3aed');
							}}
						/>
					</div>
				{:else if rightSidebarTab === 'inspector360'}
					<div class="rs-section rs-full-height">
						<Inspector360
							data={inspector360Data}
							compact={true}
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
					// In SV inspect mode: force sidebars hidden, but update the
					// snapshot so the eventual pop restores the workspace's
					// intended state (not the pre-workspace-load state).
					if (skyViewInspectMode) {
						updateSidebarSnapshot('skyInspect', layout.leftSidebarOpen, layout.rightSidebarOpen);
						sidebarOpen = false;
						rightSidebarOpen = false;
					} else {
						sidebarOpen = layout.leftSidebarOpen;
						rightSidebarOpen = layout.rightSidebarOpen;
					}
					leftSidebarWidth = layout.leftSidebarWidth;
					const validTabs = ['properties', 'backlinks', 'tags', 'star', 'tasks', 'calendar', 'health', 'provenance', 'review'] as const;
					rightSidebarTab = validTabs.includes(layout.rightSidebarTab as any) ? layout.rightSidebarTab as typeof rightSidebarTab : 'properties';
					rightSidebarWidth = layout.rightSidebarWidth;
				}
				// SS always starts closed — never auto-restore from workspace.
				// User deliberately opens it when needed.
				if (secondScreenOpen) {
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

	{#if showCanonicalChoice}
		<CanonicalChoiceDialog
			onAdopt={handleCanonicalAdopt}
			onKeepIntact={handleKeepIntact}
			onCancel={() => { showCanonicalChoice = false; adding = false; }}
		/>
	{/if}

	{#if canonicalizing}
		<div class="canonical-overlay">
			<div class="canonical-modal">
				<div class="canonical-icon">
					<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--text-accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
						<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
						<polyline points="14 2 14 8 20 8"/>
						<path d="M12 18v-6"/>
						<path d="M9 15l3 3 3-3"/>
					</svg>
				</div>
				<h2>{$t('canonical.migrationTitle')}</h2>
				<p class="canonical-desc">{$t('canonical.migrationDesc')}</p>

				{#if canonicalProgress.phase === 'scanning'}
					<div class="canonical-status">{$t('canonical.scanning')}</div>
					<div class="canonical-bar-track"><div class="canonical-bar-fill scanning"></div></div>
				{:else if canonicalProgress.phase === 'canonicalizing'}
					<div class="canonical-status">
						{canonicalProgress.current} / {canonicalProgress.total}
						<span class="canonical-lib">— {canonicalProgress.libraryName}</span>
					</div>
					<div class="canonical-bar-track">
						<div class="canonical-bar-fill" style="width: {Math.round((canonicalProgress.current / Math.max(canonicalProgress.total, 1)) * 100)}%"></div>
					</div>
					<div class="canonical-file">{canonicalProgress.currentFile}</div>
				{/if}

				<p class="canonical-note">{$t('canonical.migrationNote')}</p>
			</div>
		</div>
	{/if}

	{#if showImporter}
		<ImporterModal
			libraries={$libraries.map(v => ({ name: v.name, path: v.path }))}
			onClose={() => showImporter = false}
			onImportComplete={refreshLibraryCaches}
		/>
	{/if}

	{#if showKnowledgeHealth}
		<KnowledgeHealthDashboard onClose={() => showKnowledgeHealth = false} />
	{/if}

	{#if showPicker}
		<EmojiIconPicker
			onClose={() => showPicker = false}
			onPick={async (insertion) => {
				// Insert directly into the last-focused CM6 editor via the
				// active-editor registry. The picker's own focus doesn't
				// interfere because registration happens on every editor
				// focusin, and the picker opens without stealing that record.
				const { getActiveEditor } = await import('$lib/editor/activeEditor');
				const view = getActiveEditor();
				if (view) {
					const sel = view.state.selection.main;
					view.dispatch({
						changes: { from: sel.from, to: sel.to, insert: insertion },
						selection: { anchor: sel.from + insertion.length },
					});
					// Defer focus so Svelte finishes unmounting the picker first
					setTimeout(() => view.focus(), 0);
				}
				showPicker = false;
			}}
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
	/* ─── Canonical Migration Overlay ─── */
	.canonical-overlay {
		position: fixed;
		inset: 0;
		z-index: 99999;
		background: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: canonFadeIn 0.3s ease;
	}
	.canonical-modal {
		background: var(--background-primary, #1e1e2e);
		border-radius: 16px;
		padding: 40px 48px;
		max-width: 520px;
		width: 90vw;
		text-align: center;
		box-shadow: 0 24px 80px rgba(0, 0, 0, 0.5);
	}
	.canonical-icon { margin-bottom: 16px; }
	.canonical-modal h2 {
		margin: 0 0 8px;
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--text-normal);
	}
	.canonical-desc {
		margin: 0 0 24px;
		font-size: 0.88rem;
		color: var(--text-muted);
		line-height: 1.6;
	}
	.canonical-status {
		font-size: 0.9rem;
		font-weight: 600;
		color: var(--text-normal);
		margin-bottom: 8px;
	}
	.canonical-lib {
		font-weight: 400;
		color: var(--text-muted);
		font-size: 0.82rem;
	}
	.canonical-bar-track {
		width: 100%;
		height: 6px;
		background: var(--background-modifier-border);
		border-radius: 3px;
		overflow: hidden;
		margin-bottom: 12px;
	}
	.canonical-bar-fill {
		height: 100%;
		background: var(--text-accent);
		border-radius: 3px;
		transition: width 0.15s ease;
	}
	.canonical-bar-fill.scanning {
		width: 30%;
		animation: canonScan 1.5s ease-in-out infinite;
	}
	.canonical-file {
		font-size: 0.78rem;
		color: var(--text-faint);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		margin-bottom: 16px;
	}
	.canonical-note {
		margin: 16px 0 0;
		font-size: 0.78rem;
		color: var(--text-faint);
		line-height: 1.5;
		font-style: italic;
	}
	@keyframes canonFadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}
	@keyframes canonScan {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(400%); }
	}

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
		grid-template-rows: 1fr var(--statusbar-height, 24px);
		overflow: hidden;
	}
	.app.no-sidebar {
		grid-template-columns: auto 1fr auto;
	}

	/* ═══ DOCK ═══ */
	.dock {
		grid-row: 1;
		width: var(--dock-width, 40px);
		background: var(--dock-bg, var(--bg-tertiary));
		border-inline-end: 1px solid var(--border);
		display: flex; flex-direction: column;
		justify-content: space-between; align-items: center; padding: 6px 0;
	}
	.dock-top, .dock-bottom { display: flex; flex-direction: column; align-items: center; gap: 1px; }
	.dock-btn {
		width: var(--dock-btn-size, 32px); height: var(--dock-btn-size, 32px);
		display: flex; align-items: center; justify-content: center;
		border-radius: var(--dock-btn-radius, 4px); border: none; background: none;
		color: var(--dock-btn-color, var(--text-secondary));
		cursor: pointer; text-decoration: none; transition: all 0.1s;
	}
	.dock-btn svg {
		width: var(--dock-icon-size, 18px);
		height: var(--dock-icon-size, 18px);
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
		padding: 4px 6px;
		border-bottom: 1px solid var(--border);
		min-height: var(--sidebar-toolbar-height, 34px);
		background: var(--sidebar-toolbar-bg, transparent);
		display: flex; flex-direction: column;
	}
	.toolbar-actions { display: flex; gap: 2px; align-items: center; padding: 2px 0; }
	.tb-btn {
		width: var(--sidebar-btn-size, 26px); height: var(--sidebar-btn-size, 26px);
		display: flex; align-items: center; justify-content: center;
		border: none; background: none;
		border-radius: var(--sidebar-btn-radius, 3px);
		color: var(--sidebar-btn-color, var(--text-muted));
		cursor: pointer;
	}
	.tb-btn svg {
		width: var(--sidebar-icon-size, 16px);
		height: var(--sidebar-icon-size, 16px);
	}
	.tb-btn:hover { background: var(--border); color: var(--text); }
	.tb-btn.active { color: var(--interactive-accent); }

	/* New Library dropdown */
	.new-lib-drop {
		position: absolute; top: 100%; inset-inline: 0; z-index: 100;
		background: var(--background-primary, #fff); border: 1px solid var(--border);
		border-radius: 6px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);
		padding: 4px; margin-top: 2px; min-width: 220px;
	}
	.nld-option {
		display: flex; gap: 8px; align-items: flex-start; padding: 8px;
		border-radius: 4px; cursor: pointer; border: none; background: none;
		width: 100%; text-align: start; font-family: inherit; color: var(--text);
	}
	.nld-option:hover { background: var(--bg-hover); }
	.nld-option svg { flex-shrink: 0; margin-top: 2px; color: var(--text-muted); }
	.nld-text { flex: 1; min-width: 0; }
	.nld-title { font-size: 0.78rem; font-weight: 500; display: block; }
	.nld-desc { font-size: 0.68rem; color: var(--text-muted); display: block; margin-top: 1px; }
	.nld-input-row { display: flex; gap: 4px; margin-top: 4px; }
	.nld-input {
		flex: 1; min-width: 0; padding: 3px 6px; border: 1px solid var(--border); border-radius: 4px;
		background: var(--bg); color: var(--text); font-size: 0.75rem; font-family: inherit; outline: none;
	}
	.nld-input:focus { border-color: var(--interactive-accent); }
	.nld-create {
		padding: 3px 8px; border: 1px solid var(--interactive-accent); border-radius: 4px;
		background: var(--interactive-accent); color: #fff; font-size: 0.75rem;
		cursor: pointer; font-weight: 600;
	}
	.nld-create:hover { opacity: 0.9; }

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
		flex: 1; min-width: 0; border: none; background: none; padding: 4px 0;
		font-size: 0.82rem; color: var(--text); font-family: inherit; outline: none;
		text-overflow: ellipsis;
	}
	.search-box input::placeholder { color: var(--text-faint); }
	.search-expand {
		border: none; background: none; color: var(--text-faint); cursor: pointer; padding: 2px;
		display: flex; align-items: center; justify-content: center; border-radius: 3px;
	}
	.search-expand:hover { color: var(--interactive-accent); background: var(--bg-hover); }
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
	.s-result-top { display: flex; align-items: center; gap: 4px; }
	.s-result.s-selected { background: var(--bg-hover); outline: 1px solid var(--interactive-accent); outline-offset: -1px; }
	.s-match-badge {
		min-width: 14px; height: 14px; border-radius: 3px; flex-shrink: 0;
		font-size: 9px; font-weight: 700; line-height: 14px; text-align: center;
		color: #fff; display: inline-block;
	}
	.s-match-title { background: #3b82f6; }
	.s-match-content { background: #16a34a; }
	.s-match-semantic { background: #7c3aed; }
	.s-match-property { background: #f59e0b; }
	.s-match-tag { background: #f472b6; }
	.s-match-wikilink { background: #60a5fa; }
	.s-match-structured { background: #94a3b8; }
	/* Search history */
	.sh-icon { color: var(--text-faint); flex-shrink: 0; }
	.sh-time { margin-inline-start: auto; font-size: 0.65rem; color: var(--text-faint); flex-shrink: 0; }
	.s-history-clear {
		display: block; width: 100%; padding: 4px 12px; background: none; border: none;
		color: var(--text-faint); font-size: 0.7rem; cursor: pointer; text-align: start;
		font-family: inherit;
	}
	.s-history-clear:hover { color: var(--text-muted); text-decoration: underline; }
	/* Wikilink autocomplete */
	.wiki-autocomplete { max-height: 300px; overflow-y: auto; }
	.wa-item {
		display: flex; align-items: center; gap: 6px; width: 100%; padding: 4px 12px;
		background: none; border: none; color: var(--text); font-family: inherit;
		cursor: pointer; text-align: start; font-size: 0.82rem;
	}
	.wa-item:hover, .wa-item.wa-selected { background: var(--bg-hover); }
	.wa-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.wa-lib { font-size: 0.7rem; color: var(--text-muted); flex-shrink: 0; }
	.s-breadcrumb { font-size: 0.65rem; color: var(--text-faint); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.s-snippet { font-size: 0.7rem; color: var(--text-muted); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.s-snippet :global(mark) { background: color-mix(in srgb, var(--interactive-accent) 25%, transparent); color: var(--text-normal); border-radius: 2px; padding: 0 1px; }
	.no-results { padding: 20px; text-align: center; color: var(--text-muted); font-size: 0.82rem; }

	.library-header {
		display: flex; align-items: center; gap: 4px; width: 100%;
		padding: var(--ft-master-row-padding-y, 3px) 12px;
		background: none; border: none;
		color: var(--ft-master-color, var(--text-secondary));
		font-size: var(--ft-master-font-size, 0.8rem);
		font-weight: var(--ft-master-weight, 600);
		font-family: inherit; cursor: pointer; text-align: start;
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
		padding: var(--ft-master-row-padding-y, 3px) 12px;
		font-size: var(--ft-master-font-size, 0.8rem);
		color: var(--ft-master-color, var(--interactive-accent));
		font-weight: var(--ft-master-weight, 600);
	}
	.child-universe-item {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: var(--ft-master-row-padding-y, 3px) 12px;
		font-size: var(--ft-master-font-size, 0.8rem);
		color: var(--ft-master-color, var(--text-secondary));
		font-weight: var(--ft-master-weight, 600);
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
		background: var(--layout-bar-bg, var(--bg-secondary));
		padding: 4px 8px;
		min-height: var(--layout-bar-height, auto);
		flex-shrink: 0;
		gap: 4px;
		box-shadow: 0 -1px 0 0 rgba(0,0,0,0.08) inset;
	}

	/* Tab bar (locked to paper edge) */
	.tab-bar {
		display: flex; flex-direction: column; align-items: center;
		background: var(--topbar-bg, #e8e8ec); border-bottom: none;
		flex-shrink: 0;
		min-height: var(--topbar-height, auto);
		padding: 5px 32px 0;
	}
	/* SV inspect mode: shift the tab-bar to align with the editor's
	   .flank-center column so tabs visually sit above the note content
	   instead of being flush left/right. Values must match the 280px
	   .flank width defined below. */
	.tab-bar.tab-bar-flanked-start { padding-inline-start: calc(32px + 280px + 5px); }
	.tab-bar.tab-bar-flanked-end   { padding-inline-end:   calc(32px + 280px); }
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
		padding: 5px 10px;
		font-size: var(--tab-font-size, 0.8rem);
		height: var(--tab-height, auto);
		color: var(--tab-color, var(--text-secondary));
		background: var(--tab-bg, #dcdce0);
		border-radius: var(--tab-radius, 6px) var(--tab-radius, 6px) 0 0;
		cursor: pointer; min-width: 0;
		border: none; font-family: inherit; flex-shrink: 0;
		border-top: 3px solid var(--library-color, transparent);
		position: relative;
	}
	.tab.active, .tab.focused {
		background: var(--tab-active-bg, #ffffff); color: var(--tab-active-color, var(--text));
		border: 1px solid var(--tab-border, #d0d0d0);
		border-top: 3px solid var(--library-color, var(--accent));
		border-bottom: 1px solid var(--tab-active-bg, #ffffff);
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
		width: var(--layout-btn-size, 28px); height: var(--layout-btn-size, 28px);
		display: flex; align-items: center; justify-content: center;
		border: none; background: none;
		border-radius: var(--layout-btn-radius, 4px);
		color: var(--layout-btn-color, var(--text-muted));
		cursor: pointer; flex-shrink: 0; margin: auto 2px;
	}
	.tab-action svg {
		width: var(--layout-icon-size, 14px);
		height: var(--layout-icon-size, 14px);
	}
	.tab-action:hover:not(:disabled) { background: var(--border); color: var(--text); }
	.tab-action.active { color: var(--layout-btn-active-color, var(--accent)); }
	.tab-action:disabled { opacity: 0.3; cursor: not-allowed; }
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
	.content-area.content-hidden { display: none; }

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

	/* ──────────────────────────────────────────────────────────────
	   Tier 1 flanking panels — "note as organism"
	   Backlinks / Outgoing Links live beside the editor instead of
	   inside the right sidebar. Wrapper inherits dir from $dir so
	   flank-start/end flip automatically in RTL.
	   ──────────────────────────────────────────────────────────── */
	.editor-with-flanks {
		flex: 1;
		display: flex;
		flex-direction: row;
		min-width: 0;
		min-height: 0;
		overflow: hidden;
	}
	.flank {
		/* flex-basis is set via inline style (dynamic width from drag-resize).
		   The flex shorthand below must NOT set flex-basis — it's overridden per-instance. */
		flex: 0 0 auto;
		min-width: 180px;
		max-width: 500px;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 12px 8px;
		background: var(--bg-secondary);
		border: 0 solid var(--border);
		transition: flex-basis 120ms ease, min-width 120ms ease;
	}
	.flank-collapsed {
		/* min-width overrides the 180px floor so the flank can truly vanish */
		min-width: 0 !important;
		padding: 0;
		overflow: hidden;
	}
	.flank-start { border-inline-end-width: 1px; }
	.flank-end   { border-inline-start-width: 1px; }

	/* Wrapper that stacks the drag handle and collapse button vertically */
	.flank-handle-wrap {
		display: flex;
		flex-direction: column;
		align-items: center;
		flex: 0 0 14px;
		position: relative;
		z-index: 10;
		gap: 0;
	}
	/* Drag handles between flanking columns and the center editor */
	.flank-handle {
		flex: 1;
		width: 4px;
		cursor: col-resize;
		background: transparent;
		position: relative;
		transition: background 120ms ease;
	}
	.flank-handle:hover,
	.flank-resizing .flank-handle {
		background: var(--accent, #7c3aed);
		opacity: 0.5;
	}
	/* Widen the hit-target without widening the visual bar */
	.flank-handle::before {
		content: '';
		display: block;
		position: absolute;
		inset-block: 0;
		inset-inline: -5px;   /* 5px extra on each side → 14px total hit area */
	}

	/* Collapse/expand toggle button on each drag-handle strip */
	.flank-collapse-btn {
		flex: 0 0 auto;
		width: 14px; height: 20px;
		display: flex; align-items: center; justify-content: center;
		background: var(--background-modifier-border);
		border: none; border-radius: 2px;
		color: var(--text-muted);
		cursor: pointer;
		padding: 0;
		opacity: 0.6;
		transition: opacity 120ms, color 120ms;
	}
	.flank-collapse-btn:hover {
		opacity: 1;
		color: var(--interactive-accent);
	}

	.flank-center {
		flex: 1;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.index-overlay, .map-overlay, .orgchart-overlay, .inspector360-overlay {
		display: none; flex: 1; overflow: hidden;
		background: var(--background-primary, #fff);
		min-height: 0;
	}
	.index-overlay.index-visible, .map-overlay.map-visible, .orgchart-overlay.orgchart-visible, .inspector360-overlay.inspector360-visible { display: flex; flex-direction: column; }

	.index-return-btn {
		display: flex; align-items: center; gap: 4px;
		padding: 3px 10px; margin-inline-start: 8px; margin-inline-end: 4px;
		border: 1px solid var(--interactive-accent);
		background: color-mix(in srgb, var(--interactive-accent) 10%, transparent);
		color: var(--interactive-accent); font-size: 11px; font-weight: 600;
		border-radius: 4px; cursor: pointer; white-space: nowrap;
		flex-shrink: 0;
	}
	.index-return-btn:hover {
		background: var(--interactive-accent); color: white;
	}
	:global([dir="rtl"]) .index-return-btn svg { transform: scaleX(-1); }

	/* "Return to Sky View" paired pill + dismiss button for inspect mode. */
	.sv-pill-group { display: inline-flex; align-items: center; gap: 0; }
	.sv-pill-group .index-return-btn { margin-inline-end: 0; border-end-end-radius: 0; border-start-end-radius: 0; }
	.sv-pill-dismiss {
		padding: 3px 7px; margin-inline-end: 4px;
		border: 1px solid var(--interactive-accent);
		border-inline-start-width: 0;
		background: color-mix(in srgb, var(--interactive-accent) 10%, transparent);
		color: var(--interactive-accent); font-size: 13px; font-weight: 600;
		line-height: 1; cursor: pointer;
		border-end-end-radius: 4px; border-start-end-radius: 4px;
	}
	.sv-pill-dismiss:hover { background: var(--interactive-accent); color: white; }

	/* Star fullscreen */
	.star-fullscreen {
		flex: 1; display: flex; flex-direction: column; overflow: hidden;
		background: var(--background-primary, #fff);
		position: relative;
	}
	.sky-loading {
		position: absolute;
		top: 56px;
		inset-inline-end: 16px;
		display: flex; align-items: center; gap: 8px;
		padding: 6px 12px;
		background: rgba(0, 0, 0, 0.55);
		color: #fff;
		border-radius: 999px;
		font-size: 0.8rem;
		pointer-events: none;
		z-index: 15;
		backdrop-filter: blur(4px);
	}
	.sky-loading-spinner { animation: sky-loading-spin 0.9s linear infinite; }
	@keyframes sky-loading-spin { to { transform: rotate(360deg); } }
	.tab-bar-hidden { display: none !important; }
	.star-header {
		display: flex; align-items: center; gap: 4px;
		padding: 8px 16px; border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
		position: relative; z-index: 20;
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
	.lens-overlay {
		display: none; flex: 1; overflow: hidden;
		background: var(--background-primary); min-height: 0;
	}
	.lens-overlay.lens-visible { display: flex; }
	.searchhub-overlay {
		display: none; flex: 1; overflow: hidden;
		background: var(--background-primary); min-height: 0;
	}
	.searchhub-overlay.searchhub-visible { display: flex; }
	@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
	:global(.spin) { animation: spin 1s linear infinite; }

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
		overflow: hidden;
	}
	.welcome.welcome-dashboard {
		align-items: stretch; justify-content: stretch;
		text-align: start; padding: 0; overflow: hidden;
	}
	.w-icon { margin-bottom: 20px; opacity: 0.7; }
	.w-title { color: var(--text); font-size: 1.2rem; font-weight: 600; margin: 0 0 4px; }
	.w-sub { font-size: 0.9rem; margin: 0 0 16px; }
	.w-hint { font-size: 0.9rem; }
	.w-hint-sub { font-size: 0.78rem; color: var(--text-faint); margin-top: 4px; }
	.w-dashboard-btn {
		margin-top: 16px; padding: 8px 16px; border-radius: 8px;
		border: 1px solid var(--border); background: var(--bg-secondary);
		color: var(--text-muted); cursor: pointer; font-size: 0.82rem;
		display: flex; align-items: center; gap: 6px;
		transition: all 0.15s;
	}
	.w-dashboard-btn:hover { border-color: var(--accent); color: var(--text); }
	.home-dashboard { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
	.home-dashboard-header {
		display: flex; justify-content: flex-end; padding: 6px 12px 0;
		flex-shrink: 0;
	}
	.home-dashboard-toggle {
		width: 28px; height: 28px; border-radius: 6px;
		border: 1px solid var(--border); background: transparent;
		color: var(--text-muted); cursor: pointer;
		display: flex; align-items: center; justify-content: center;
	}
	.home-dashboard-toggle:hover { background: var(--border); color: var(--text); }
	.w-btn {
		background: var(--accent); border: none; color: var(--text-on-accent);
		padding: var(--button-padding-y, 8px) var(--button-padding-x, 20px);
		border-radius: var(--button-radius, 6px);
		cursor: pointer;
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
		grid-row: 1; background: var(--right-sidebar-bg, var(--bg-secondary));
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
		background: var(--rs-tabs-bg, var(--right-sidebar-bg, var(--bg-secondary)));
		position: sticky; top: 0; z-index: 3;
	}
	.rs-tab {
		flex: 1; display: flex; align-items: center; justify-content: center;
		height: var(--rs-tab-height, 30px);
		border: none; background: none;
		color: var(--rs-tab-color, var(--text-muted));
		cursor: pointer;
		border-bottom: 2px solid transparent;
	}
	.rs-tab svg {
		width: var(--rs-icon-size, 16px);
		height: var(--rs-icon-size, 16px);
	}
	.rs-tab:hover { background: var(--bg-hover); color: var(--text); }
	.rs-tab.active {
		color: var(--rs-tab-active-color, var(--accent));
		border-bottom-color: var(--rs-tab-active-color, var(--accent));
	}
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
	.rs-section.rs-section--flush { border-bottom: none; }
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
		grid-column: 1 / -1; grid-row: 2;
		height: var(--statusbar-height, 24px);
		background: var(--statusbar-bg, var(--bg-tertiary));
		border-top: 1px solid var(--border);
		display: flex; align-items: center; justify-content: space-between;
		padding: 0 10px;
		font-size: var(--statusbar-font-size, 0.7rem);
		color: var(--statusbar-color, var(--text-muted));
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

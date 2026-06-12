<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy, untrack, tick } from 'svelte';
	import { dir, t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { getVersion } from '@tauri-apps/api/app';
	import {
		libraries, libraryStats, totalStars, libraryCount,
		activeTab, openTabs, activeTabId,
		splitActive, splitDirection, focusedTabId, focusedTab,
		loadLibraries, loadAllStats, addLibrary, createNewLibrary, createNewLibraryAt,
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
		getDailyNotePath, updateLinksOnRename, getOldTitleForCascade, reloadTabsFromDisk,
		flushAllTabsInLibrary, markCascading, clearCascading, clearAllCascading,
		tabsInLibrary, quickCapture,
		loadBookmarks, addBookmark, removeBookmark, isBookmarked, bookmarks,
		loadSettings, updateSettings, appSettings, DEFAULT_SETTINGS, applyParsedSettings,
		loadWorkspaces, workspaces,
		resolveWikilinkCrossLibrary,
		buildDefaultFrontmatter,
		linkTraversalBumps, clearLinkTraversalBumps,
		skyNodePathSet,
		type FrontmatterProperty, type HeadingItem, type NoteLink, type SkyNode, type SkyLink,
		type IndexEntry
	} from '$lib/libraries/store';
	import type { LibraryStats, FileEntry, WorkspaceLayout, WorkspaceSecondScreen, FontSet, PanelId } from '$lib/libraries/store';
	import { BUILTIN_FONT_SETS, SCRIPT_UNICODE_RANGES, TYPEWRITER_FONTS, getFontSetById, hexToHSL } from '$lib/libraries/store';
	import { liveStyleDraft } from '$lib/libraries/store'; // MIG-070 §C Option E — Style Setter live-preview layer
	// MIG-076 §C — single content ownership (FocusPane seeds from / saves through the model).
	import { editBody as editNoteBody, seedBody, externalChange as externalChangeNoteModel } from '$lib/editor/noteSession';
	import { compose as composeNoteModel, markSaved as markNoteSaved } from '$lib/editor/noteModel';
	import { SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';
	import { CORE_BLOCK_IDS } from '$lib/theme/constellationStyleSettings';
	import { get } from 'svelte/store';
	import { SvelteMap } from 'svelte/reactivity';
	import { detectDir, eventToShortcut, normalizeShortcut, getResolvedShortcut, formatShortcut, migratePathKeyedMap, migratePathKeyedMapInPlace, normalizePathKey } from '$lib/utils';
	import { createBase, listWorkspaceBases, createWorkspaceBase, deleteWorkspaceBase } from '$lib/bases/store';
	import type { WorkspaceBaseEntry } from '$lib/bases/store';
	// MIG-055 §F — Five Acts sidebar section (Constellation Base v1).
	import { listFiveActsNotes, type FiveActsNoteEntry } from '$lib/lens/store';
	// MIG-056 §H — Cross-universe federation warning surface.
	import { getFederationWarnings, type FederationWarning } from '$lib/federation/store';
	import FileTree from '$lib/components/FileTree.svelte';
	// MIG-045 Phase 3 — Universe Digest left-dock pane.
	import DigestPane from '$lib/components/DigestPane.svelte';
	import NotebookNavigator from '$lib/components/NotebookNavigator.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import FocusPane from '$lib/components/FocusPane.svelte';
	import BaseTab from '$lib/lens/BaseTab.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import CreateItemDialog, { type CreateKind } from '$lib/components/CreateItemDialog.svelte';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import QuickSwitcher from '$lib/components/QuickSwitcher.svelte';
	import TemplatePicker from '$lib/components/TemplatePicker.svelte';
	import TemplatePrompt from '$lib/components/TemplatePrompt.svelte';
	import TemplateSuggester from '$lib/components/TemplateSuggester.svelte';
	import { processTemplate, processTemplateAsync, extractTemplateBody, type TemplateCallbacks } from '$lib/templates/engine';
	import GraphMindView from '$lib/components/GraphMindView.svelte';
	import ConstellationSight from '$lib/components/ConstellationSight2.svelte';
	import SightV3 from '$lib/sight/v3/SightV3.svelte';
	import SightV4 from '$lib/sight/v4/SightV4.svelte';
	// MIG-028 (2026-05-18): SightV5 import retired with the v5 module set.
	import SightV6 from '$lib/sight/v6/SightV6.svelte';
	// MIG-036 P1 (2026-05-19): SightV7 parallel mount per B2 dual-mount.
	import SightV7 from '$lib/sight/v7/SightV7.svelte';
	import { SIGHT_V2_ENABLED, SIGHT_V3_ENABLED, SIGHT_V4_ENABLED, SIGHT_V6_ENABLED, SIGHT_V7_ENABLED } from '$lib/sight/engine';
	import { detectClusters, computeStructuralGaps, computeUniverseHealth, buildCommunityProfiles, stratumWeightedCentrality, suggestBridges, type StructuralGap, type UniverseHealth, type ClusterInfo, type CommunityProfile } from '$lib/graph/clusterEngine';
	import OrgChart from '$lib/components/OrgChart.svelte';
	import CatalogerView from '$lib/components/CatalogerView.svelte';
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
	import CCSView from '$lib/components/CCSView.svelte';
	import TasksPanel from '$lib/components/TasksPanel.svelte';
	import CalendarPanel from '$lib/components/CalendarPanel.svelte';
	import GlobalTasksView from '$lib/components/GlobalTasksView.svelte';
	import TensionPanel from '$lib/components/TensionPanel.svelte';
	import ProvenancePanel from '$lib/components/ProvenancePanel.svelte';
	import ReviewPulsePanel from '$lib/components/ReviewPulsePanel.svelte';
	import SourceReviewPanel from '$lib/components/SourceReviewPanel.svelte';
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
	import StyleSetter from '$lib/components/StyleSetter.svelte';
	import { openStyleSetterInspect } from '$lib/stores/styleSetter'; // MIG-070 §C item D — dock inspect shortcut
	import LockScreen from '$lib/components/LockScreen.svelte';
	import MigrationProgressStrip from '$lib/components/MigrationProgressStrip.svelte';
	import ClassifierScanProgressStrip from '$lib/components/ClassifierScanProgressStrip.svelte';
	import NscBackfillProgressStrip from '$lib/components/NscBackfillProgressStrip.svelte';
	import LibrarySwitcher from '$lib/components/LibrarySwitcher.svelte';
	import LibraryManager from '$lib/components/LibraryManager.svelte';
	import LibraryPicker from '$lib/components/LibraryPicker.svelte';
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
	let sidebarMode = $state<'tree' | 'list' | 'skyview' | 'digest'>('tree');
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
	let rightSidebarTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks' | 'calendar' | 'health' | 'provenance' | 'review' | 'inspector360' | 'sourceReview'>('properties');
	// MIG-007 follow-up (task_19b5319d) — which right-sidebar tabs require an open note. The rest
	// (tags / links / review / sourceReview) are universe-wide and render with or without a note.
	// Gated ONCE in the content area below, replacing the old `isHome && sidebarTab` note-gate that
	// hoisted tags/links above it and duplicated review/sourceReview inside + outside it.
	const NOTE_SCOPED_TABS = new Set(['properties', 'backlinks', 'star', 'tasks', 'calendar', 'health', 'provenance', 'inspector360']);
	// Tag Browser (#12): the right-sidebar Tags tab toggles between the open
	// note's tags ('note') and the universe-wide federated tag tree ('all',
	// fed by allLibraryTags via the reusable TagsPanel).
	let tagView = $state<'note' | 'all'>('note');

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
	let showCCS = $state(false); // MIG-074 — CCS (Constellation Circulatory System) left-dock Core Plug-in
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
	// Multi-hop back-nav stack from the compact widget (§98 single-step,
	// upgraded to multi-hop in §99 per Boss request). Each forward node-
	// click pushes the current note onto the stack; each back click pops
	// one entry. Click-back walks all the way to the original source.
	let inspector360BackStack = $state<Array<{path: string; name: string}>>([]);
	let trailIndex = $state(0); // CE Phase 8: current note index in trail
	let tensionReport = $state<any>(null); // CE Phase 4: TensionReport
	let tensionLoading = $state(false); // analyzing state for the health tab
	let _tensionLibPath: string | null = null; // cache guard — re-fetch only when the library changes
	let provenanceChain = $state<any>(null); // CE Phase 5: ProvenanceChain
	let _lastProvenancePath = ''; // cache guard — only re-fetch when note changes

	// Sidebar resizing
	let leftSidebarWidth = $state(300);
	let rightSidebarWidth = $state(380); // Tag Browser (#12) + general breathing room (was 340)
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
	// MIG-039 — The Cataloger (CECE) left-dock full-page view.
	let showCataloger = $state(false);
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
	// MIG-025 §B.6-fix-5 — show "Return to Sight" button on note tab
	// after the user clicks a star in Sight v6 (anchor or promoted mini)
	// to open a note. Mirrors lensReturnPending pattern.
	let sightV6ReturnPending = $state(false);

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
	let catalogerEverOpened = $state(false);
	$effect(() => { if (showConstellationMap) mapEverOpened = true; });
	$effect(() => { if (showOrgChart) orgChartEverOpened = true; });
	$effect(() => { if (showCataloger) catalogerEverOpened = true; });
	$effect(() => { if (showInspector360) inspector360EverOpened = true; });

	// Sky View inspect-mode lockout recovery. When the user clicks an SV node,
	// the SV-opened note becomes a tab with skyViewInspectMode=true and both
	// sidebars hidden. The intended exit is the "Return to Sky View" pill or
	// its `×` dismiss button — both of which only render while `$activeTab?.path`
	// is truthy. If instead the user closes the note tab via the tab's own ×,
	// `$activeTabId` goes null while skyViewInspectMode stays true: the
	// dismiss pill disappears with the tab, and the sidebar toggles are gated
	// behind `!skyViewInspectMode` (lines ~1660-1661) so they refuse to open
	// either flank. Result: app locked until restart. Recovery: mirror the
	// dismiss-pill cleanup when the active tab disappears mid-inspect.
	$effect(() => {
		if (skyViewInspectMode && $activeTabId === null) {
			popSidebars('skyInspect');
			skyViewInspectMode = false;
		}
	});
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
	// MIG-008 §Build.6 — orphaned state vars from pre-MIG-008 dropdowns and
	// dialogs removed: `showNewBaseDialog` (NewBaseDialog block deleted in
	// §Build.4), `showNewLibraryDropdown` + `newLibName` (sidebar inline
	// dropdown UI deleted in §Build.5).

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

	// MIG-060 §C-fix-2 — keep the `skyNodePathSet` store in sync with the
	// local `skyNodes` array. LensBlockWidget reads the store to decide
	// whether to render the CNS gesture button per row (orphans hide it,
	// since CNS has no node to focus for orphans). Reactive: any time the
	// graph data refreshes, the store updates and lens widgets get the
	// new orphan-set on their next render.
	$effect(() => {
		skyNodePathSet.set(new Set(skyNodes.map(n => n.path)));
	});

	// Constellation Lens state
	let lensActive = $state(false);
	let lensLoading = $state(false);
	// MIG-060 §C-fix — when CNS is opened via a lens-row threading gesture,
	// the gesture sets this to the clicked note's path so ConstellationSight2
	// can center the gravity well on that node at mount time. Null when CNS
	// is opened via the dock button (no specific focus target — default view).
	let pendingCnsFocusPath = $state<string | null>(null);
	// MIG-018 (PJ-038) — v3 Sight active flag. Independent from lensActive
	// so a developer flipping SIGHT_V2_ENABLED + SIGHT_V3_ENABLED both true
	// in a custom build can A/B them. In production exactly one of v2 / v3 / v4
	// renders at a time; the SIGHT_V*_ENABLED gates enforce this.
	let sightV3Active = $state(false);
	let sightV4Active = $state(false);
	// MIG-024 §1 — Sight v5 mount state. RETIRED MIG-028 (2026-05-18) —
	// the v5 module set is fully removed from the build. Variable pinned
	// false; remaining references in mutex-clear lines are harmless no-ops.
	// Cleanup of those references is a future polish item.
	let sightV5Active = $state(false);
	// MIG-025 §A.7 — Sight v6 mount state. Gated false until §A.14 ship gate
	// clears (Phase 1 of MIG-025 / Concept Paper v4.0). Mutually exclusive
	// with v5 per B2 dual-mount; one engine renders at a time.
	let sightV6Active = $state(false);
	// MIG-036 P1 (2026-05-19) — Sight v7 mount state. B2 dual-mount with
	// v6 during the Form-Aligns-To-Purpose redesign cascade.
	let sightV7Active = $state(false);
	let lensCentrality = $state<Map<string, number>>(new Map());
	let lensCommunities = $state<ClusterInfo[]>([]);
	let lensCommunityAssignments = $state<Map<string, number>>(new Map());
	let lensGaps = $state<StructuralGap[]>([]);
	let lensHealth = $state<UniverseHealth | null>(null);
	let lensBridges = $state<{ id: string; name: string; centrality: number }[]>([]);
	// (MIG-075 §A3 removed the dead lensShowTagEdges / lensPeelCount /
	// lensTagEdges states — the tag-edges/layer-peeling era leftovers with
	// zero readers; layer peeling stays on the CNS roadmap per paper Q5.)
	let lensCommunityProfiles = $state<CommunityProfile[]>([]);
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
	// §139: SvelteMap (Svelte 5 explicitly-reactive Map). Mutations like
	// .set() / .delete() trigger reactivity at the operation level — no
	// reassign-to-force-identity dance required. The Rule 8 derived views
	// (file-tree stage emoji + maturity dot) re-render the moment the map
	// mutates. Replaces the prior `$state(new Map())` pattern, which had
	// a prop-propagation quirk visible after promote/demote: the parent's
	// reassignment didn't always reach the FileTree's template binding.
	let maturityMap = new SvelteMap<string, string>(); // path → maturity state (CE Phase 3)
	let stageMap = new SvelteMap<string, string>(); // path → stage (CE Phase 6)
	/** §141 — single onStageChanged handler shared by every NoteEditor instance
	 *  (main, split, second-screen). Mutates the SvelteMap directly so the file
	 *  tree's stage emoji updates reactively for every consumer. */
	function handleStageChanged(path: string, stage: string) {
		const key = normalizePathKey(path);
		if (stage) stageMap.set(key, stage);
		else stageMap.delete(key);
	}
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

	// MIG-055 §F — Five Acts host notes ({universe}/Five Acts/*.md).
	// Populated by `listFiveActsNotes()` on universe activation; auto-created
	// by Rust `init_five_acts_system_notes` at boot (idempotent, edit-preserving).
	let fiveActsNotes = $state<FiveActsNoteEntry[]>([]);
	let fiveActsExpanded = $state(true);

	// MIG-062 §E — federated grouping for the Five Acts + Workspace Bases
	// sidebar sections. Active-universe entries (no universe_name) render
	// normally; cUniverse entries group under collapsible per-universe
	// sub-headers (collapsed by default — Boss "maybe just hide it"). The
	// expanded-Sets track which cUniverse sub-groups are open.
	let expandedCuFiveActs = $state<Set<string>>(new Set());
	let expandedCuBases = $state<Set<string>>(new Set());
	const fiveActsActive = $derived(fiveActsNotes.filter(n => !n.universe_name));
	const fiveActsByCu = $derived.by(() => {
		const m = new Map<string, FiveActsNoteEntry[]>();
		for (const n of fiveActsNotes) {
			if (!n.universe_name) continue;
			(m.get(n.universe_name) ?? m.set(n.universe_name, []).get(n.universe_name)!).push(n);
		}
		return m;
	});
	const basesActive = $derived(workspaceBases.filter(b => !b.universe_name));
	const basesByCu = $derived.by(() => {
		const m = new Map<string, WorkspaceBaseEntry[]>();
		for (const b of workspaceBases) {
			if (!b.universe_name) continue;
			(m.get(b.universe_name) ?? m.set(b.universe_name, []).get(b.universe_name)!).push(b);
		}
		return m;
	});
	function toggleCuGroup(set: Set<string>, key: string): Set<string> {
		const next = new Set(set);
		if (next.has(key)) next.delete(key); else next.add(key);
		return next;
	}

	// MIG-056 §H — federation warnings (skip_unavailable cUniverses).
	// Refreshed on boot + on universe switch; surfaces in the status bar
	// when length > 0. Click → popup with details.
	let federationWarnings = $state<FederationWarning[]>([]);
	let showFederationWarningsPopup = $state(false);

	// Child universes for sidebar
	let childUniverses = $state<ChildUniverseInfo[]>([]);
	let childUniversesExpanded = $state(true);
	// Track which libraries belong to which child universe (childUniversePath → Set of normalized library paths)
	let childUniverseLibPaths = $state<Map<string, Set<string>>>(new Map());
	// Track which child universes are expanded in the file explorer
	let expandedChildUniverses = $state<Set<string>>(new Set());

	let error = $state('');
	let adding = $state(false);
	// MIG-008 §Build.6 — orphaned state vars from welcome screen's pre-MIG-008
	// inline create form removed: `creatingNew`, `newLibraryName`. The card
	// now opens the shared CreateItemDialog which manages its own state.

	const isHome = $derived(page.url.pathname === '/');
	const isDashboardVisible = $derived(isHome && !$activeTab && $libraries.length > 0 && $appSettings.showDashboard);
	/** True when any full-page function is active — disables sidebars and split pane */
	const fullPageActive = $derived(showSkyView || showGlobalTasks || showIndex || showExpressionForge || showSenseMakingCanvas || showConstellationMap || showOrgChart || showCataloger || showKnowledgeHealth || lensActive || sightV3Active || sightV4Active || sightV5Active || sightV6Active || showSearchHub || showInspector360 || isDashboardVisible);

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
	// MIG-076 §C — the focus note's identity, captured at focus entry and held
	// for the whole session so FocusPane's body push + save always target the
	// note focus was opened for, even as the active tab changes underneath.
	let focusSessionId = $state('');
	let focusSessionPath = $state('');
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
			inspector360: inSidebar('inspector360'),
			// MIG-021 §1C — Source Review panel is always visible (not yet
			// in panelPlacements; force-visible until the Settings → Panels
			// integration ships in a follow-up).
			sourceReview: true,
		};

		if (!tabVisible[rightSidebarTab]) {
			const order = ['properties', 'backlinks', 'tags', 'star', 'tasks', 'calendar', 'health', 'provenance', 'review', 'inspector360', 'sourceReview'] as const;
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

	// MIG-071 — the theme subsystem was removed; all styling is the Style Setter's per-Universe
	// styleOverride applied on top of the plain default base (theme.css :root). This remains the
	// SINGLE writer of body CSS vars (the BUG-015 guard): styleOverride, then the Setter's transient
	// live layer, last. Responds to settings changes + live Setter edits.
	$effect(() => {
		if (typeof document === 'undefined') return;
		const s = $appSettings;
		const root = document.body.style;
		const trackedVars: Record<string, string> = {};

		// Accent colour (untracked) — applied when set and not the app default.
		if (s.accentColor && s.accentColor !== '#7c3aed') {
			const hsl = hexToHSL(s.accentColor);
			root.setProperty('--accent-h', String(hsl.h));
			root.setProperty('--accent-s', `${hsl.s}%`);
			root.setProperty('--accent-l', `${hsl.l}%`);
		}

		// The saved per-Universe look (Style Setter "Keep") — tracked, so a removed key clears next apply.
		Object.assign(trackedVars, s.styleOverride ?? {});
		// The Style Setter's transient LIVE layer wins while the Setter is open (same single writer;
		// cleared on Keep/Discard/close). Reading $liveStyleDraft re-runs THIS effect on a live edit.
		Object.assign(trackedVars, $liveStyleDraft);

		// Clear any tracked var from the previous apply that is gone now (reset row / removed override).
		const newKeys = new Set(Object.keys(trackedVars));
		for (const prevKey of _lastStyleSettingsKeys) {
			if (!newKeys.has(prevKey)) root.removeProperty(prevKey);
		}
		_lastStyleSettingsKeys = [...newKeys];
		for (const [key, value] of Object.entries(trackedVars)) {
			root.setProperty(key, value);
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
			css += `.cm-editor .cm-content { font-family: var(--font-text-theme, ${twStack}) !important; }\n`;
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
				css += `.cm-editor .cm-content { font-family: var(--font-text-theme, ${txtStack}) !important; }\n`;
				css += `.cm-editor .cm-scroller { font-family: ${txtStack}; }\n`;
			} else {
				root.setProperty('--font-interface-theme', baseUI);
				root.setProperty('--font-text-theme', baseTxt);
				css += `.cm-editor .cm-content { font-family: var(--font-text-theme, ${baseTxt}) !important; }\n`;
				css += `.cm-editor .cm-scroller { font-family: ${baseTxt}; }\n`;
			}
		}

		// MIG-070 §C — the Style Setter composes the final font vars when it has any font picks:
		// the Latin base comes from the styleOverride font vars, and per-script fonts layer on via
		// @font-face unicode-range using a DISTINCT virtual family ("CnSetterText"/"CnSetterUI") so
		// it never collides with the engine's "ConstellationText". This WINS over the per-language
		// engine above (runs last). When the Setter sets no fonts, the engine's result stands.
		{
			const ov = s.styleOverride ?? {};
			const psf = s.perScriptFonts ?? {};
			const ranges = SCRIPT_UNICODE_RANGES as Record<string, string>;
			const psfActive = ['arabic', 'hebrew', 'cjk', 'devanagari', 'cyrillic'].filter((sc) => psf[sc] && ranges[sc]);
			if (ov['--font-text-theme'] || ov['--font-interface-theme'] || ov['--font-monospace-theme'] || psfActive.length) {
				const latinText = ov['--font-text-theme'] || defaultUI;
				const latinUI = ov['--font-interface-theme'] || defaultUI;
				const latinMono = ov['--font-monospace-theme'] || defaultMono;
				let setterCss = '';
				for (const sc of psfActive) {
					const name = psf[sc].split(',')[0].trim().replace(/"/g, '');
					setterCss += `@font-face { font-family: "CnSetterText"; src: local("${name}"); unicode-range: ${ranges[sc]}; }\n`;
					setterCss += `@font-face { font-family: "CnSetterUI"; src: local("${name}"); unicode-range: ${ranges[sc]}; }\n`;
				}
				if (psfActive.length) {
					root.setProperty('--font-text-theme', `"CnSetterText", ${latinText}`);
					root.setProperty('--font-interface-theme', `"CnSetterUI", ${latinUI}`);
				} else {
					root.setProperty('--font-text-theme', latinText);
					root.setProperty('--font-interface-theme', latinUI);
				}
				root.setProperty('--font-monospace-theme', latinMono);
				setterCss += `.cm-editor .cm-content { font-family: var(--font-text-theme) !important; }\n`;
				css += setterCss;
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
			{ id: 'search', name: $t('commands.searchLibrary'), shortcut: sc('search'), icon: '🔎', action: () => { showSearchHub = true; searchHubInitialQuery = ''; showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false; showInspector360 = false; /* fullPageActive $effect handles sidebar snapshot */ }, category: 'Navigation' },
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
			{ id: 'cataloger', name: $t('commands.cataloger') || 'The Cataloger', icon: '🗃️', action: () => { showCommandPalette = false; showCataloger = !showCataloger; if (showCataloger) { showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showKnowledgeHealth = false; showInspector360 = false; showSearchHub = false; showExpressionForge = false; showSenseMakingCanvas = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false; } }, category: 'View' },
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
			{ id: 'knowledge-health', name: 'Knowledge Health', icon: '🧠', action: () => { showCommandPalette = false; showKnowledgeHealth = true; showCCS = false; }, category: 'View' },
			...($appSettings.enabledFeatures?.ccs !== false ? [{ id: 'ccs', name: $t('ccs.title') || 'Constellation Circulatory System', icon: '🫀', action: () => { showCommandPalette = false; showCCS = true; showKnowledgeHealth = false; }, category: 'View' }] : []),
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
			link_types?: unknown[];
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

			// Warm-boot fix (2026-05-21): seed `libraryStats` from the library
			// list NOW so the sidebar's library sections render immediately
			// (~0.4 s). The sidebar derives `ownLibraries`/`universeNotesStats`
			// from `$libraryStats`, which was previously empty until the
			// `get_all_library_stats` IPC finished (~1.5–3 s on a cold DB) — the
			// measured cause of "the universe is blank until ~2.5 s, then comes
			// to life at once" (boot-perf round 2: sidebar_populated_ms dropped
			// 2452 → 423 ms with this seed). star_count/folder_count/recent_stars
			// are placeholders (the count badge is hidden when 0); the
			// fire-and-forget `loadAllStats()` in the post-hydration fan-out
			// fills the real values a moment later, so badges just pop in.
			libraryStats.set(bundle.libraries.map(lib => ({
				library_id: lib.id,
				name: lib.name,
				path: lib.path,
				star_count: 0,
				folder_count: 0,
				recent_stars: [],
				is_universe_notes: lib.is_universe_notes,
			})));

			// Settings — single source of truth via applyParsedSettings
			// (store.ts). Pre-§A.14-fix-5 this was an inline merge that
			// drifted from loadSettings() — missed sight, cece,
			// panelPlacements, index nested merges + the §A.12 migration.
			// Result: v6 defaults never landed + migration never ran on
			// real user settings (only loadSettings() hit them, but
			// loadSettings() has zero callers — boot-bundle is the only
			// load path). Eisa's 2026-05-14 Boss-test surfaced this.
			try {
				const parsed = (bundle.settings as Record<string, unknown>) || {};
				applyParsedSettings(parsed);
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

			// Link types (MIG-067 §C) — seed the link-type registry from the
			// bundle so the editor colors + Base per-type columns read the
			// resolved vocabulary (8 seeds + custom) without a separate IPC.
			try {
				const ltReg = await import('$lib/libraries/linkTypeRegistry');
				ltReg.seedFromBundle(bundle.link_types);
			} catch { /* falls back to on-demand loadLinkTypes() */ }

			workspaceBases = bundle.workspace_bases;
			// MIG-055 §F — load Five Acts host notes alongside the boot bundle.
			// Not yet part of the bundle IPC (deferred to §J or a future MIG);
			// fire-and-forget here keeps boot time unchanged (single fs read).
			listFiveActsNotes().then(n => fiveActsNotes = n).catch(() => {});
			// MIG-056 §H — load federation warnings. Fire-and-forget; the
			// status-bar badge appears once warnings.length > 0. Refreshed
			// later via setTimeout to catch the background-attach completion.
			loadFederationWarnings();
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
				// MIG-055 §F — load Five Acts host notes in the fallback path too.
				listFiveActsNotes().then(n => fiveActsNotes = n).catch(() => {}),
				// MIG-056 §H — load federation warnings in the fallback path too.
				getFederationWarnings().then(w => federationWarnings = w).catch(() => {}),
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
				.then(() => {
					loadAllStatsWallMs = Math.round(performance.now() - t0);
					// BUG-022 — auto-recover an empty search index. If the active
					// universe has libraries but ZERO indexed notes, its index
					// never finished building (a prior failed init like BUG-021, a
					// wiped/restored DB, or files synced in while the app was
					// closed). Build it once in the background — the SAME builder
					// `add_library` uses (`constellation_search_init` →
					// `reconcile_filesystem`). Runs on boot AND universe-switch
					// (both go through initializeApp).
					//
					// GATED ON EMPTY: an already-indexed universe has star_count>0,
					// so this never fires for it — the ZERO BOOT-TIME WALKS rule is
					// preserved for the common case. After the build, re-load stats
					// so the sidebar/status bar note count updates from 0 → N.
					const stats = get(libraryStats);
					const totalIndexed = stats.reduce((sum, s) => sum + (s.star_count || 0), 0);
					if (totalIndexed === 0 && stats.length > 0) {
						initSearchIndex()
							.then(() => { searchEngineReady = true; return loadAllStats(); })
							.catch(() => {});
					}
				})
				.catch(() => { loadAllStatsWallMs = Math.round(performance.now() - t0); });
		}
		// MIG-067 — search-ready WITHOUT a filesystem walk. A §B-era cache_reconcile()
		// fired here re-walked every file on EVERY boot (the audible thrash, violating
		// the ZERO BOOT-TIME WALKS rule above). cache_mark_search_ready just ensures the
		// DB + emits 'cache-reconciled', so the listeners below load incoming link counts
		// + mark search ready — no walk. Bulk closed-time changes: Settings → Rebuild
		// Index; live changes: the file watcher.
		setTimeout(() => { invoke('cache_mark_search_ready').catch(() => {}); }, 800);
	}

	async function handleUniverseCreated(entry: UniverseEntry) {
		await setActiveUniverse(entry.id);
		activeUniverseName = entry.name;
		showUniverseSetup = false;
		await initializeApp();
	}

	/**
	 * MIG-056 §H — Load federation warnings + re-poll once after a delay
	 * to catch the background-attach completion. Background-attach happens
	 * in a separate Rust thread spawned from `ensure_search_db_ready`;
	 * the initial fire-and-forget call may run before attach completes
	 * (returns empty warnings); the delayed re-poll catches anything that
	 * surfaced during attach.
	 */
	async function loadFederationWarnings() {
		try {
			federationWarnings = await getFederationWarnings();
		} catch {
			federationWarnings = [];
		}
		// Re-poll once after ~3s to catch the background-attach completion.
		// Per Architect §6.3 the attach takes tens-to-low-hundreds ms per
		// cUniverse; 3s is generous headroom for ≤25 cUniverses.
		//
		// §K.2 — Also re-fire loadAllStats() so the status-bar notes
		// count + sidebar library badges pick up federated rows. The
		// initial loadAllStats() runs in the post-hydration fan-out
		// BEFORE federation attaches (federation runs in a background
		// Rust thread); without this re-fire the stats stay frozen at
		// the pre-attach (active-universe-only) snapshot and the
		// status bar shows 1101 notes instead of 8751 with cUniverses.
		setTimeout(async () => {
			try {
				federationWarnings = await getFederationWarnings();
			} catch {
				// Keep existing state on error.
			}
			try {
				await loadAllStats();
			} catch {
				// Keep stale stats on error rather than zeroing.
			}
		}, 3000);
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
		// MIG-055 §F — clear Five Acts notes; reloaded by initApp() for the
		// new universe (init_db on that universe re-creates the system note
		// if absent, edit-preserving otherwise).
		fiveActsNotes = [];
		// MIG-056 §H — clear federation warnings; reloaded by initApp() for
		// the new universe's cUniverse attach result.
		federationWarnings = [];
		showFederationWarningsPopup = false;
		// A cascade in flight in the previous Universe could leave entries
		// in cascadingPaths that gate edits in the new one if any path
		// happens to collide — start the new Universe with a clean slate.
		clearAllCascading();

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
		catalogerEverOpened = false;
		inspector360EverOpened = false;
		inspector360Data = null;
		inspector360BackStack = [];
		mapFocusNode = null;
		showCCS = false; // MIG-074 — close CCS on universe switch ({#if} unmount re-reads the new universe's snapshot on next open)

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

		// MIG-021v3 V3-§10.A — CECE background scan on app start.
		// When enabled in Settings, fire classifier_scan_start once
		// per app boot. Best-effort + non-blocking; the scan runs on
		// a background thread per V3-§1F'.b, so it doesn't impact
		// the user's first-paint or first-interaction latency.
		// Defer by 5 seconds so it doesn't compete with the boot
		// path's other startup IPCs.
		if (get(appSettings).cece?.backgroundScan === 'on_startup') {
			setTimeout(() => {
				invoke('classifier_scan_start').catch((err) => {
					console.warn('[CECE on-startup] classifier_scan_start failed:', err);
				});
			}, 5000);
		}

		// MIG-040 — NSC summary backfill is MANUAL (Boss decision 2026-05-21):
		// it is NOT started on boot. The auto-after-paint trigger here used to
		// force the embedding-model load + a full-Universe embed pass ~8 s in,
		// which made the app feel like it was still booting for ~20 s. Instant
		// boot is a hard requirement, so the backfill now runs only when the
		// user presses "Build all summaries" in the Cataloger. Summaries still
		// fill lazily on-demand as cards scroll into view (no boot cost).

		// Listen for template picker requests from CodeMirrorEditor /template slash command
		window.addEventListener('constellation:open-template-picker', handleTemplatePicker);
		document.addEventListener('constellation:show-importer', () => { showImporter = true; });
		// MIG-007 hub, re-pointed by MIG-074 §D (Architect ruling 7): the Settings →
		// Links button now opens CCS — the Link Dashboard tab is retired into it.
		document.addEventListener('constellation:open-ccs', () => {
			if ($appSettings.enabledFeatures?.ccs === false) return;
			showCCS = true;
			showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showKnowledgeHealth = false; showInspector360 = false; showSearchHub = false; showExpressionForge = false; showSenseMakingCanvas = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false;
		});

		// Universal Embed: "open this note" (from transclusion header click,
		// or MIG-055 §D LensBlock row-click).
		//
		// Detail shape:
		//   - `path` is REQUIRED (the note's filesystem path).
		//   - `libraryName` is OPTIONAL — provided by LensBlock so federated
		//     cUniverse rows resolve to the right library even when their
		//     path prefix coincidentally matches a different library's path.
		//     Surfaced by the MIG-055 §H drift audit (P2-2): path-prefix-only
		//     matching could silently mis-route or fail to open federated
		//     rows. Prefer the explicit libraryName when the dispatcher
		//     provides it; fall back to prefix matching for legacy
		//     dispatchers (UniversalEmbedWidget) that don't.
		window.addEventListener('constellation:open-note', (e: Event) => {
			const detail = (e as CustomEvent).detail as { path?: string; libraryName?: string; libraryPath?: string };
			if (!detail?.path) return;
			const libs = get(libraries);
			let lib = detail.libraryName ? libs.find(l => l.name === detail.libraryName) : undefined;
			if (!lib) lib = libs.find(l => detail.path!.startsWith(l.path));
			if (lib) openNoteTab(detail.path!, lib.name, libraryColorMap[lib.name] || '#7c3aed');
		});
		// MIG-060 §C — Threading-gesture listener.
		// LensBlockWidget._renderRow (§B) dispatches this custom event when
		// the user clicks one of the three threading-gesture buttons on a
		// lens row. The handler:
		//   1. Opens the host note in the active pane (same library-resolve
		//      flow as `constellation:open-note`).
		//   2. Awaits one tick so the reactive cascade settles.
		//   3. Clears all other full-page-surface flags, then activates the
		//      requested surface — same "exclusive surface" pattern the dock
		//      button onclick handlers use.
		window.addEventListener('constellation:open-note-in-surface', async (e: Event) => {
			const detail = (e as CustomEvent).detail as {
				surface?: '360.3d' | 'cns' | 'cataloger';
				path?: string;
				libraryName?: string;
				libraryPath?: string;
			};
			if (!detail?.path || !detail?.surface) return;
			// Step 1 — open the host note.
			const libs = get(libraries);
			let lib = detail.libraryName ? libs.find(l => l.name === detail.libraryName) : undefined;
			if (!lib) lib = libs.find(l => detail.path!.startsWith(l.path));
			if (lib) await openNoteTab(detail.path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
			// Step 2 — settle the reactive cascade so `sidebarTab` / active
			// tab state is consistent before flipping surface flags.
			await tick();
			// Step 3 — activate the requested surface. Each branch clears
			// every other full-page flag first (mirrors dock-button onclick).
			switch (detail.surface) {
				case '360.3d':
					showSkyView = false; showGlobalTasks = false; showIndex = false;
					showConstellationMap = false; showOrgChart = false; showCataloger = false;
					showKnowledgeHealth = false; showCCS = false; showSearchHub = false;
					showExpressionForge = false; showSenseMakingCanvas = false;
					lensActive = false; sightV3Active = false; sightV4Active = false;
					sightV5Active = false; sightV6Active = false;
					showInspector360 = true;
					break;
				case 'cns':
					// CNS gating is enforced at the gesture-render site (§B):
					// if `constellationSight === false`, the button is not
					// shown, so this branch should not fire for disabled
					// users. Defensive re-check kept for safety.
					if (get(appSettings).enabledFeatures?.constellationSight === false) break;
					showSkyView = false; showGlobalTasks = false; showIndex = false;
					showConstellationMap = false; showOrgChart = false; showCataloger = false;
					showInspector360 = false;
					showKnowledgeHealth = false; showCCS = false; showSearchHub = false;
					showExpressionForge = false; showSenseMakingCanvas = false;
					sightV3Active = false; sightV4Active = false;
					sightV5Active = false; sightV6Active = false;
					// MIG-060 §C-fix — set the focus target BEFORE toggleLens()
					// so ConstellationSight2 reads it as a $prop at mount time.
					// (The {#if lensActive && SIGHT_V2_ENABLED} block remounts the
					// component when lensActive flips true, so prop bindings are
					// fresh on every open.)
					pendingCnsFocusPath = detail.path;
					if (!lensActive) toggleLens();
					break;
				case 'cataloger':
					showSkyView = false; showGlobalTasks = false; showIndex = false;
					showConstellationMap = false; showOrgChart = false; showInspector360 = false;
					showKnowledgeHealth = false; showCCS = false; showSearchHub = false;
					showExpressionForge = false; showSenseMakingCanvas = false;
					lensActive = false; sightV3Active = false; sightV4Active = false;
					sightV5Active = false; sightV6Active = false;
					showCataloger = true;
					// MIG-060 §C-fix-3 — focus The Cataloger on the clicked note.
					// CataloferView contains a SourceReviewPanel that listens for
					// `constellation:classify-and-show` with `detail.notePath` and
					// classifies + scrolls to that specific note. Defer the
					// dispatch one animation frame so CataloferView's panel has
					// mounted (matches the established pattern at
					// `handleSuggestSourcesForNote`).
					{
						const focusPath = detail.path;
						requestAnimationFrame(() => {
							window.dispatchEvent(new CustomEvent('constellation:classify-and-show', {
								detail: { notePath: focusPath },
							}));
						});
					}
					break;
			}
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

		// Living Link P3 periodic decay job REMOVED (2026-06-10). Weight decay is DISPLAY-ONLY
		// (`effectiveLinkWeight` computes it at read time; the stored `weight` is the immutable
		// integral of traversals) — there is no stored-decay job to run. The old `linkDecay()` call
		// mutated + compounded raw weights on a 24h timer. See `constellation_link_decay` (now read-only).

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

		// MIG-061 §J.2 — federation:ready listener MUST register BEFORE
		// initializeApp() runs. initializeApp triggers the boot snapshot IPCs,
		// which call ensure_search_db_ready → spawn the federation attach
		// thread. On a fast cUniverse setup, attach_all completes + emits
		// federation:ready BEFORE initializeApp returns. If the listener
		// registered after that point (as the earlier §J revision did), the
		// event was dropped — CNS stayed stuck at parent-only data.
		//
		// Defensive: on listener registration, immediately invoke
		// cache_boot_snapshot_sky once. If federation already completed
		// (event fired and missed), this re-fetch picks up the federated
		// data. If federation is still pending, the call returns the same
		// parent-only data the boot path got — no harm — and the live
		// listener catches the event when it eventually fires.
		// MIG-061 §N — federation:ready listener (re-fetches BOTH sky AND graph).
		// §J originally only re-invoked _sky → Backlinks/Outgoing stayed stuck
		// with parent-only data from boot. §N re-invokes _graph too so
		// allLibraryLinks updates with federated note_links.
		// MIG-061 §P — empty-overwrite guard: if a federation:ready re-fire
		// returns empty data (e.g., federated_conn became None mid-universe-
		// switch race per Audit S6 / D4 finding), don't clobber the good
		// data we already have. The guard preserves prior allLibraryLinks
		// when the new payload is smaller AND the current is non-empty.
		const unlistenFederationReady = await listen('federation:ready', async () => {
			// Refresh sky (CNS / Sky View) — listener already guards isReady.
			try {
				type SkySnapshot = {
					nodes: SkyNode[];
					links: SkyLink[];
					isReady: boolean;
					timingsMs: Array<[string, number]>;
				};
				const sky = await invoke<SkySnapshot>('cache_boot_snapshot_sky');
				if (sky && sky.isReady && sky.nodes.length >= skyNodes.length) {
					skyNodes = sky.nodes;
					skyLinks = sky.links;
					skyVersion++;
				}
			} catch {}
			// Refresh graph (Backlinks / Outgoing / Tags / Aliases).
			// BootSnapshotGraph has no isReady field; the §M code returns
			// empty arrays when federated_conn is None mid-race. So we
			// guard by "new data must not be empty when current isn't"
			// (D4 audit finding).
			try {
				type GraphSnapshot = {
					links: NoteLink[];
					tags: Record<string, number>;
					aliases?: Array<{ path: string; aliasLower: string }>;
				};
				const graph = await invoke<GraphSnapshot>('cache_boot_snapshot_graph');
				const newLinksLen = graph?.links?.length ?? 0;
				const newIsValid = newLinksLen > 0 || allLibraryLinks.length === 0;
				if (graph && newIsValid) {
					allLibraryLinks = graph.links ?? [];
					allLibraryTags = graph.tags ?? {};
					notePathToAliases = new Map();
					if (Array.isArray(graph.aliases)) {
						for (const a of graph.aliases) {
							const list = notePathToAliases.get(a.path) ?? [];
							list.push(a.aliasLower);
							notePathToAliases.set(a.path, list);
						}
					}
				}
			} catch {}
			// MIG-062 §E — re-fetch the federated filesystem-walk surfaces so
			// cUniverse Five Acts notes + Workspace Bases appear once federation
			// settles. Manifest-based enum means they're usually present at the
			// first call, but this re-fetch is cheap insurance + covers the
			// universe-switch case.
			try { listFiveActsNotes().then(n => fiveActsNotes = n).catch(() => {}); } catch {}
			try { listWorkspaceBases().then(b => workspaceBases = b).catch(() => {}); } catch {}
		});
		cleanupFns.push(() => { try { unlistenFederationReady(); } catch {} });

		// initializeApp paints the shell (appReady=true) at its very first
		// step, then loads data in the background. We `await` its full
		// completion here so later onMount steps (watcher setup, etc.)
		// run with a populated libraries store.
		await initializeApp();

		// MIG-061 §N — defensive re-invoke after initializeApp.
		// Refreshes BOTH sky and graph (§N extends §J.2 to cover the graph
		// payload that feeds Backlinks/Outgoing).
		try {
			type SkySnapshot2 = {
				nodes: SkyNode[];
				links: SkyLink[];
				isReady: boolean;
				timingsMs: Array<[string, number]>;
			};
			const sky2 = await invoke<SkySnapshot2>('cache_boot_snapshot_sky');
			if (sky2 && sky2.isReady && sky2.nodes.length > skyNodes.length) {
				skyNodes = sky2.nodes;
				skyLinks = sky2.links;
				skyVersion++;
			}
		} catch {}
		try {
			type GraphSnapshot2 = {
				links: NoteLink[];
				tags: Record<string, number>;
				aliases?: Array<{ path: string; aliasLower: string }>;
			};
			const graph2 = await invoke<GraphSnapshot2>('cache_boot_snapshot_graph');
			if (graph2 && graph2.links.length > allLibraryLinks.length) {
				allLibraryLinks = graph2.links;
				allLibraryTags = graph2.tags ?? {};
				if (Array.isArray(graph2.aliases)) {
					notePathToAliases = new Map();
					for (const a of graph2.aliases) {
						const list = notePathToAliases.get(a.path) ?? [];
						list.push(a.aliasLower);
						notePathToAliases.set(a.path, list);
					}
				}
			}
		} catch {}

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

		// §3-redo.4 — wikilink rename cascade reload listener.
		// The watcher_suppress map (§3-redo.2) keeps the cascade's fs::write
		// from re-firing as a `library-changed` event, so the cascade has its
		// own dedicated event. The orchestration path in handleRenameComplete
		// already awaits reloadTabsFromDisk directly, so for the rename
		// cascade this listener is defence-in-depth: reloadTabsFromDisk is
		// idempotent (no-ops when disk content already matches the tab),
		// so the second pass costs one parallel batch of read_note IPCs and
		// no store update. The listener exists so any future Rust path that
		// emits cascade:rewrote (without going through update_links_on_rename
		// from the frontend) still triggers the recreate-primitive reload —
		// per Concept Paper D6, $effect on value/editBody is forbidden
		// (BUG-015's class).
		const unlistenCascadeRewrote = await listen<{ paths: string[] }>('cascade:rewrote', async (event) => {
			await reloadTabsFromDisk(event.payload?.paths ?? []);
		});
		cleanupFns.push(() => { try { unlistenCascadeRewrote(); } catch {} });

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
			// (Living Link decay-on-startup REMOVED 2026-06-10 — decay is display-only; no boot job.)
		});
		cleanupFns.push(() => { try { unlistenSearchReady(); } catch {} });

		// MIG-061 §J — federation:ready listener (registration moved earlier in §J.2).
		// See the listener block above `await initializeApp()` for the actual
		// registration. Without that earlier registration, the event fires while
		// initializeApp is still running (federation completes faster than boot
		// finishes on a 24-cUniverse universe) and the listener — if registered
		// here — would miss it.

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
					// MIG-076 §C — the second screen wrote this note; the main
					// window's model adopts it (freshness-gated: a dirty local
					// model is never clobbered).
					if (SINGLE_OWNERSHIP) externalChangeNoteModel(tab.id, content);
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
		// §139: SvelteMap — replace contents in-place. Mutations are
		// reactive at the operation level, so the file tree re-renders
		// for each .set() / .delete().
		maturityMap.clear();
		for (const [k, v] of newMatMap) maturityMap.set(k, v);
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
		// §139: SvelteMap — replace contents in-place (see maturityMap above).
		stageMap.clear();
		for (const [k, v] of newStageMap) stageMap.set(k, v);
		// Lenses (cheap, once). The tension load that used to sit here was
		// re-homed to loadTensionReport() — tab-activated, active-note-scoped
		// (this function only runs from the on-demand Sky View enrichment, so
		// a boot-era tension load here left the health tab "Loading…" forever).
		try { availableLenses = await invoke('list_lenses'); } catch { availableLenses = []; }
		skyVersion++;
	}

	// CE Phase 4 — Tension report for the right-sidebar Knowledge Health tab.
	// Loaded lazily when the tab activates with a note open (the boot-time
	// loader was removed by the zero-boot-walks rule and the tab was never
	// re-pointed — it showed "Loading…" forever). Scoped to the ACTIVE note's
	// library (the old wiring scanned libPaths[0] — the wrong library in any
	// multi-library universe); cached per library path; a failed run clears
	// the guard so the next activation retries.
	async function loadTensionReport(notePath: string) {
		const lib = $libraryStats.find(l => notePath.startsWith(l.path));
		if (!lib) return;
		if (_tensionLibPath === lib.path && (tensionReport || tensionLoading)) return;
		_tensionLibPath = lib.path;
		tensionLoading = true;
		try {
			tensionReport = await invoke('detect_tensions', { libraryPath: lib.path, libraryName: lib.name });
		} catch (e) {
			console.error('detect_tensions failed:', e);
			tensionReport = null;
			_tensionLibPath = null; // allow retry on next activation
		}
		tensionLoading = false;
	}
	$effect(() => {
		if (rightSidebarTab === 'health' && isHome && sidebarTab) {
			const p = sidebarTab.path;
			untrack(() => { void loadTensionReport(p); });
		}
	});

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
			if (showCommandPalette) { showCommandPalette = false; return; }
			if (showQuickSwitcher) { showQuickSwitcher = false; return; }
			if (showSkyView) { showSkyView = false; return; }
			if (lensActive) { lensActive = false; return; }
			if (sightV3Active) { sightV3Active = false; return; }
			if (sightV4Active) { sightV4Active = false; return; }
			if (sightV5Active) { sightV5Active = false; return; }
			if (sightV6Active) { sightV6Active = false; return; }
			if (showOrgChart) { showOrgChart = false; return; }
			if (showCataloger) { showCataloger = false; return; }
			if (sidebarMode === 'skyview') { sidebarMode = 'tree'; return; }
			if (sidebarMode === 'digest') { sidebarMode = 'tree'; return; }
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

	// MIG-008 §Build.3 — sidebar "+ New Note" toolbar opens the shared
	// create-dialog with the library root pre-filled as the parent. Pre-MIG-008
	// this auto-incremented "Untitled", "Untitled 1", … via a 100-iter loop;
	// the user now names it upfront. Template + frontmatter + edit-mode logic
	// preserved unchanged inside the onCreate callback.
	// §152 — single helper that BOTH right-click and toolbar paths invoke. Boss
	// directive 2026-05-03 (right-click should respect folder templates the
	// same way toolbar does). Pre-§152 right-click did createNote+open only;
	// toolbar did create + template + frontmatter merge + open + edit-mode.
	// Now both paths apply templates uniformly. The location parameter is
	// the parent folder (the dialog's chosen location).
	async function createNoteWithTemplate(
		lib: { id: string; name: string; path: string },
		location: string,
		name: string,
	): Promise<void> {
		const defaultFM = buildDefaultFrontmatter($appSettings);
		const newPath = await createNote(location, name, defaultFM);
		if (!newPath) return;

		// Resolve template: check folder templates first, then default.md.
		let templateBody = '';
		if ($appSettings.enabledFeatures?.templates) {
			try {
				const tplDir: string = await invoke('get_templates_dir');
				const noteFolder = newPath.replace(/\\/g, '/').split('/').slice(0, -1).join('/');
				const folderTpls = $appSettings.folderTemplates || {};
				let matchedTpl = '';
				let matchDepth = -1;
				for (const [folder, tplName] of Object.entries(folderTpls)) {
					const normFolder = folder.replace(/\\/g, '/');
					if (noteFolder.includes(normFolder) || noteFolder.endsWith(normFolder)) {
						const depth = normFolder.split('/').length;
						if (depth > matchDepth) { matchDepth = depth; matchedTpl = tplName; }
					}
				}
				const tplFile = matchedTpl || 'default';
				const tplPath = `${tplDir}/${tplFile.endsWith('.md') ? tplFile : tplFile + '.md'}`;
				const tpl: string = await invoke('read_note', { filePath: tplPath });
				if (tpl) templateBody = parseFrontmatter(tpl).body;
			} catch { /* no template — OK */ }
		}

		// Apply template if found — preserve canonical frontmatter fields (cid_cn, kind, title).
		// §152 — uses parseFrontmatter (canonical YAML reader) instead of the prior
		// hand-rolled regex, eliminating drift surface.
		if (templateBody.trim()) {
			try {
				const noteContent: string = await invoke('read_note', { filePath: newPath });
				const parsed = parseFrontmatter(noteContent);
				const canonicalKeys = new Set(['title', 'cid', 'cid_cn', 'kind']);
				const canonicalFields: string[] = parsed.properties
					.filter(p => canonicalKeys.has(p.key.toLowerCase()))
					.map(p => `${p.key}: ${p.value}`);
				const noteFolder = newPath.replace(/\\/g, '/').split('/').slice(-2, -1)[0] || '';
				const ctx = { title: name, folder: noteFolder, library: lib.name, filePath: newPath };
				const result = await processTemplateAsync(templateBody, ctx, buildTemplateCallbacks());
				const mergedFM = [...canonicalFields, ...defaultFM.split('\n').filter(l => {
					const key = l.split(':')[0]?.trim();
					return key && !canonicalFields.some(cf => cf.startsWith(key + ':'));
				})].join('\n');
				const fullContent = `---\n${mergedFM}\n---\n${result.content}`;
				await invoke('write_note', { filePath: newPath, content: fullContent, origin: 'template_create' });
			} catch { /* template write failed — note still created */ }
		}

		await refreshLibraryTree(lib.id);
		const libraryColor = libraryColorMap[lib.name] ?? '#7c3aed';
		await openNoteTab(newPath, lib.name, libraryColor);
		const tab = get(focusedTab);
		if (tab) toggleEditMode(tab.id);
	}

	function createNoteInLibrary(lib: { id: string; name: string; path: string }) {
		createDialog = {
			kind: 'note',
			parentPath: lib.path,
			onCreate: ({ name, location }) => createNoteWithTemplate(lib, location, name),
		};
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

	// MIG-008 §Build.4 — workspace base creation. Opens the shared dialog with
	// hideLocation (workspace bases always live in the workspace dir, no parent
	// to pick) and the library multi-select rendered via the extras snippet.
	function handleNewBase() {
		baseSelectedLibraries = []; // reset to "all libraries"
		createDialog = {
			kind: 'base',
			parentPath: '',
			hideLocation: true,
			extrasKind: 'baseLibraryPicker',
			onCreate: async ({ name }) => {
				await createWorkspaceBaseWithLibraries(name, baseSelectedLibraries);
			},
		};
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
					// MIG-065 §I-b — creates a minimal LensDefinition YAML scoped to
					// the chosen libraries (empty = all); opens directly in BaseTab.
					newPath = await createWorkspaceBase(name, selectedLibraries);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			if (!newPath) return;

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

	// MIG-008 §Build.5 — sidebar "+ New Library" toolbar button (and the
	// command palette item) opens the shared create-dialog with empty
	// parentPath, which surfaces the dialog's "Pick…" affordance for
	// location. The user picks the parent folder + types the name in
	// one place; on Create, the new `create_new_library_at` IPC builds
	// the library at the chosen path. Pre-MIG-008 this toggled an inline
	// dropdown in the sidebar that took just the name then the Rust
	// IPC opened its own folder picker AFTER Create.
	function handleNewLibrary() {
		createDialog = {
			kind: 'library',
			parentPath: '',
			onCreate: async ({ name, location }) => {
				await createNewLibraryAt(location, name);
				await refreshLibraryCaches();
				initSearchIndex().then(() => { searchEngineReady = true; }).catch(() => {});
			},
		};
	}

	// MIG-008 §Build.2 — sidebar "+ New Folder" toolbar button (and the
	// library-picker fallback when multiple libraries are present) opens the
	// shared create-dialog with the library root pre-filled as the parent.
	// Pre-MIG-008 this auto-incremented "New Folder", "New Folder 1", … via
	// a 100-iter loop. Now the user names it upfront; collisions surface as
	// the IPC error in the dialog's inline error region.
	function createFolderInLibrary(lib: { id: string; name: string; path: string }) {
		createDialog = {
			kind: 'folder',
			parentPath: lib.path,
			onCreate: async ({ name, location }) => {
				await createFolder(location, name);
				await refreshLibraryTree(lib.id);
				if (!expandedLibraries.has(lib.id)) {
					expandedLibraries.add(lib.id);
					expandedLibraries = new Set(expandedLibraries);
				}
			},
		};
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

	// MIG-062 §E.2 — hide the system "Five Acts" folder from library file trees.
	// Per Boss: the Observation note is surfaced via the dedicated top "Five
	// Acts" sidebar section (+ federated cUniverse groups), so the folder in
	// the tree is redundant duplication. READ-ONLY hide — the files stay on
	// disk. The dedicated section always reflects the ACTIVE universe, so when
	// the user detaches a cUniverse or switches to it (it becomes active), its
	// Five Acts is shown there by default. Only the universe-root system folder
	// is filtered (top level); nested user content is untouched.
	function hideFiveActsFolder(entries: FileEntry[] | undefined): FileEntry[] {
		if (!entries) return [];
		// Recursive: the system folder sits at the universe root, which shows
		// at the TOP level in the active universe's tree but NESTED (under a
		// universe-name wrapper) in a federated cUniverse's tree. Strip it at
		// any depth so both displays are clean. Only the exact system name
		// "Five Acts" is removed; everything else (incl. nested user content)
		// is preserved.
		return entries
			.filter(e => !(e.is_dir && e.name === 'Five Acts'))
			.map(e => e.children ? { ...e, children: hideFiveActsFolder(e.children) } : e);
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
						await invoke('write_note', { filePath: path, content: newContent, origin: 'daily_template' });
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
				await invoke('write_note', { filePath: tab.path, content: newContent, origin: 'template_insert' });
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
		// MIG-016 §1A — performance.mark instrumentation. Calibrates the
		// per-phase budgets for §1B (edges-on-hover) / §1C (worker offload)
		// / §1D (post-paint prewarm) / §1E (SQLite cache). Console.table
		// dump after lensActive = true. Boss data-collection gate: Eisa
		// runs the build, opens DevTools console, copies the table to me.
		performance.mark('sight:toggle:start');
		try {
			// 1. Compute centrality in Rust — MIG-075 §A1: DB-sourced (note_links)
			// + async; no library paths, no fs walk. Scope = the active universe.
			performance.mark('sight:rust-centrality:start');
			const result = await invoke<{ centrality: Record<string, number>; node_count: number; edge_count: number }>(
				'constellation_sight_centrality'
			);
			lensCentrality = new Map(Object.entries(result.centrality));
			performance.mark('sight:rust-centrality:end');
			performance.measure('sight:rust-centrality', 'sight:rust-centrality:start', 'sight:rust-centrality:end');

			// 2. Run community detection (existing JS Louvain)
			performance.mark('sight:louvain:start');
			const clusterResult = detectClusters(
				skyNodes.map(n => ({ id: n.id, name: n.name })),
				skyLinks.map(l => ({ source: l.source, target: l.target })),
			);
			lensCommunities = clusterResult.clusters;
			lensCommunityAssignments = clusterResult.assignments;
			performance.mark('sight:louvain:end');
			performance.measure('sight:louvain', 'sight:louvain:start', 'sight:louvain:end');

			// 3. Compute structural gaps
			performance.mark('sight:structural-gaps:start');
			lensGaps = computeStructuralGaps(
				clusterResult.clusters,
				skyLinks.map(l => ({ source: l.source, target: l.target })),
				clusterResult.assignments,
			);
			performance.mark('sight:structural-gaps:end');
			performance.measure('sight:structural-gaps', 'sight:structural-gaps:start', 'sight:structural-gaps:end');

			// 4. Compute universe health
			performance.mark('sight:universe-health:start');
			lensHealth = computeUniverseHealth(
				clusterResult.modularity,
				clusterResult.clusters,
				skyNodes.length,
				skyLinks.length,
				lensGaps.length,
			);
			performance.mark('sight:universe-health:end');
			performance.measure('sight:universe-health', 'sight:universe-health:start', 'sight:universe-health:end');

			// 5. Stratum-weighted centrality (Feature 2)
			performance.mark('sight:stratum-weighted:start');
			const weightedCentrality = stratumWeightedCentrality(
				lensCentrality,
				skyLinks.map(l => ({ source: l.source, target: l.target })),
				skyNodes,
			);
			lensCentrality = weightedCentrality; // Replace with stratum-weighted version
			performance.mark('sight:stratum-weighted:end');
			performance.measure('sight:stratum-weighted', 'sight:stratum-weighted:start', 'sight:stratum-weighted:end');

			// 6. Build top bridges list (top 10 by weighted centrality)
			performance.mark('sight:top-bridges:start');
			lensBridges = [...lensCentrality.entries()]
				.sort((a, b) => b[1] - a[1])
				.slice(0, 10)
				.map(([id, centrality]) => {
					const node = skyNodes.find(n => n.id === id);
					return { id, name: node?.name ?? id, centrality };
				});
			performance.mark('sight:top-bridges:end');
			performance.measure('sight:top-bridges', 'sight:top-bridges:start', 'sight:top-bridges:end');

			// 7. Community profiles (Features 4 & 5: maturity + provenance)
			performance.mark('sight:community-profiles:start');
			lensCommunityProfiles = buildCommunityProfiles(
				clusterResult.clusters,
				clusterResult.assignments,
				skyNodes,
			);
			performance.mark('sight:community-profiles:end');
			performance.measure('sight:community-profiles', 'sight:community-profiles:start', 'sight:community-profiles:end');

			// 8. Bridge suggestions for gaps (Feature 7)
			performance.mark('sight:bridge-suggestions:start');
			lensGaps = suggestBridges(
				lensGaps,
				clusterResult.clusters,
				skyNodes.map(n => ({ id: n.id, name: n.name })),
				skyLinks.map(l => ({ source: l.source, target: l.target })),
			);
			performance.mark('sight:bridge-suggestions:end');
			performance.measure('sight:bridge-suggestions', 'sight:bridge-suggestions:start', 'sight:bridge-suggestions:end');

			// (The old step 9 — contradiction pairs from the centrality IPC —
			// was removed in MIG-075 §A1: its only consumer was a dead prop.
			// The pair list is detect_tensions' per the ratified CNS paper §5.)

			// Mark cache fresh only if the graph didn't change mid-computation.
			if (skyVersion === computeVersion) {
				lensDataStale = false;
			}
			lensActive = true;
			performance.mark('sight:toggle:end');
			performance.measure('sight:toggle:total', 'sight:toggle:start', 'sight:toggle:end');

			// MIG-016 §1A — performance.marks retained for any future DevTools
			// session. Alert/clipboard prompts removed at §1B start (the
			// toggle trace was never reachable through the cache-warm
			// path; the mount trace alone confirmed mount is fast at
			// ~175-370ms; no further data needed to ship §1B's edges-on-
			// hover gate). Console.log retained for the rare DevTools session.
			const sightMeasures = performance.getEntriesByType('measure')
				.filter(m => m.name.startsWith('sight:'))
				.map(m => ({ phase: m.name, duration_ms: Math.round(m.duration) }));
			console.log('[MIG-016 §1A] Sight perf trace:');
			console.table(sightMeasures);
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
		showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false; showInspector360 = false;
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

	// (MIG-013 §1D originally added a boot-time CTSE auto-fire that
	// ran a first-fill + slow-path backfill on the existing library
	// before the user could query. That index-time approach was
	// retired — the dominant industry pattern (Lucene
	// SynonymGraphFilter, SQLite FTS5 Method 2, CLIR query
	// translation, Primo controlled-vocabulary expansion) does
	// concept expansion at query time, not at index time. CTSE now
	// follows that pattern: `ctse_search_by_concept` embeds the user
	// query, finds top-K M11 concepts, and expands them to
	// multilingual lemmas in memory — no per-term backfill, no
	// status-bar strip, no boot wait. See `src-tauri/src/ctse/search.rs`.)

	// ─── Index panel: FTS5 vocab read ───
	// The Index panel is now backed by the `notes_vocab` virtual table
	// (fts5vocab over `notes_fts`). FTS5 already maintains the term dictionary
	// incrementally as notes change — no build step, no progress bar, no
	// batch loop. We just read when the data is ready.
	//
	// Triggered on `graphReady` so the first read doesn't fight the boot
	// snapshot IPC queue. Subsequent universe switches trigger a re-read.
	// One `invoke` round-trip, tens of milliseconds on a 50k-term Universe.
	// MIG-010 Phase D: load lazily on first Index-panel open, not at boot.
	// Pre-MIG-010 this fired on `graphReady` regardless of whether the user
	// would actually open the Index — paying the cost (tens of ms on a 50k-
	// term Universe, 100-200 ms on 100k+) for every boot, even when the
	// user never expanded the panel that session. Gating on `showIndex`
	// makes the cost on-demand: free at boot, paid the first time the
	// user opens the panel. Subsequent opens (within the same Universe)
	// hit the cached `allIndexEntries`. Universe switches keep the existing
	// `indexLoadedKey` invalidation path so the data refreshes correctly.
	$effect(() => {
		if (!showIndex) return;
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
				// §138 (Rule 8 — Write-Time Derivation): load this library's
				// stage + maturity metadata so the file tree shows the stage
				// emoji 🌱📖🔗✨ + maturity dot ● immediately. enrichNodesBackground
				// was removed from boot for boot-perf (zero boot-time walks);
				// before §138 the only path that populated stageMap / maturityMap
				// was the Sky View legend's onRequestEnrichment, so the file
				// tree never showed indicators on a normal boot. Library expand
				// is user-action-triggered (respects the boot-perf discipline)
				// and natural — the user is showing they want this library's
				// contents. Fire-and-forget; the maps are reactive `$state`,
				// so the file tree re-renders when each scan returns. Failures
				// are silent — a missing emoji is preferable to blocking the
				// expand.
				// SvelteMap mutations are reactive at the .set() level; the
				// file tree re-renders the moment a key changes. The per-key
				// `if (… !== stage)` guard skips no-op writes so unchanged
				// entries don't fire spurious reactivity.
				invoke<[string, string][]>('scan_note_stages', { libraryPath: lib.path })
					.then((stages) => {
						for (const [path, stage] of stages) {
							const key = normalizePathKey(path);
							if (stageMap.get(key) !== stage) stageMap.set(key, stage);
						}
					})
					.catch(() => {});
				invoke<{ note_path: string; state: string }[]>(
					'compute_note_maturity', { libraryPath: lib.path, libraryName: lib.name }
				)
					.then((maturities) => {
						for (const m of maturities) {
							const key = normalizePathKey(m.note_path);
							if (maturityMap.get(key) !== m.state) maturityMap.set(key, m.state);
						}
					})
					.catch(() => {});
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

	// MIG-008 §Build.6 — handleCreateNewLibrary removed; the welcome screen's
	// "Create new library" card now invokes handleNewLibrary (opens the shared
	// CreateItemDialog). The dialog handles its own submitting/disabled state.

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
	let contextMenu = $state<{ x: number; y: number; entry: FileEntry; libraryId: string; isLibraryRoot?: boolean } | null>(null);
	let confirmDelete = $state<{ path: string; name: string } | null>(null);

	// MIG-008 §Build.1: shared create dialog state. Single component for
	// Folder / Note / Base / Library so the four flows don't drift.
	// Each affordance handler sets this and the template renders the
	// dialog when non-null. The `onCreate` callback is the affordance's
	// per-kind commit logic (createFolder + refresh, createNote + open
	// tab, etc.) — passed in so the dialog stays kind-agnostic.
	let createDialog = $state<{
		kind: CreateKind;
		parentPath: string;
		hideLocation?: boolean;
		/** When `kind === 'base'` workspace, this is `'baseLibraryPicker'` so the
		 *  template renders the library multi-select snippet (snippets are
		 *  template-scoped in Svelte 5; this discriminator is the bridge from
		 *  state to the template's snippet table). Other kinds leave it undefined. */
		extrasKind?: 'baseLibraryPicker';
		onCreate: (args: { name: string; location: string }) => Promise<boolean | void>;
	} | null>(null);

	// State for Base's library multi-select extras. Reset on each base-create
	// open. Empty list means "all libraries". `baseSelectedSet` is a $derived
	// O(1)-lookup view used by the snippet's per-library `class:active` and
	// `checked` bindings — avoids O(N²) `.includes` at federation scale.
	let baseSelectedLibraries = $state<string[]>([]);
	let baseSelectedSet = $derived(new Set(baseSelectedLibraries));
	function toggleBaseLibrary(name: string) {
		if (baseSelectedSet.has(name)) {
			baseSelectedLibraries = baseSelectedLibraries.filter(v => v !== name);
		} else {
			baseSelectedLibraries = [...baseSelectedLibraries, name];
		}
	}
	let renamingPath = $state('');

	function handleContextMenu(entry: FileEntry, x: number, y: number, libraryId: string) {
		contextMenu = { x, y, entry, libraryId };
	}

	// MIG-008 §Build.6 follow-up — library/universe row right-click handler.
	// Suppresses the WebView's default context menu (Back/Refresh/Save as/Print)
	// and shows a Constellation menu with the create affordances (New Note /
	// New Folder / New Base) at the library root. The synthetic FileEntry
	// gives downstream handlers the library's path + name; isLibraryRoot=true
	// tells `getContextMenuItems` to suppress Rename/Delete (which would be
	// nonsensical at the library level — those operations live in Library
	// Manager / command palette).
	function handleLibraryHeaderContextMenu(
		e: MouseEvent,
		lib: { library_id: string; name: string; path: string },
	) {
		e.preventDefault();
		const synthetic = {
			path: lib.path,
			name: lib.name,
			is_dir: true,
			children: undefined,
			display_title: lib.name,
		} as unknown as FileEntry;
		contextMenu = {
			x: e.clientX,
			y: e.clientY,
			entry: synthetic,
			libraryId: lib.library_id,
			isLibraryRoot: true,
		};
	}

	function getContextMenuItems(entry: FileEntry, libraryId: string, isLibraryRoot = false) {
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

		// MIG-008 §Build.6 follow-up — library/universe root right-click was
		// falling through to the WebView default context menu (Back/Refresh/
		// Save as/Print) because no oncontextmenu was wired. Now wired via
		// `handleLibraryHeaderContextMenu`; we render a slim version of the
		// folder menu (New Note / New Folder / New Base — the create
		// affordances) but suppress Rename/Delete since library-management
		// operations live elsewhere (Library Manager, command palette).
		if (isLibraryRoot) {
			items.push({
				label: $t('actions.newNote'),
				icon: '📄',
				action: () => handleCreateNote(entry.path, libraryId),
			});
			items.push({
				label: $t('actions.newFolder'),
				icon: '📁',
				action: () => handleCreateFolder(entry.path, libraryId),
			});
			items.push({
				label: $t('actions.newBase'),
				icon: '▦',
				action: () => handleCreateBase(entry.path, libraryId),
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
			// MIG-008 §Build.6 follow-up — New Base was missing from the folder
			// context menu (Boss flagged during §Build verification). Folder-based
			// bases are a real flow but had no right-click affordance, only the
			// command palette / sidebar toolbar. Adding here for parity with the
			// other create operations.
			items.push({
				label: $t('actions.newBase'),
				icon: '▦',
				action: () => handleCreateBase(entry.path, libraryId)
			});
		}
		items.push({
			label: $t('actions.rename'),
			icon: '✏️',
			action: () => { renamingPath = entry.path; }
		});
		// MIG-021v2 §1E' — right-click "Suggest sources & content type" on
		// any markdown note. Triggers Tier-2 classification and surfaces the
		// result in the Source Review panel. Files only, not folders.
		if (!entry.is_dir && entry.name.toLowerCase().endsWith('.md')) {
			items.push({
				label: $t('sources.contextMenu.suggest') || 'Suggest sources & content type',
				icon: '✨',
				action: () => handleSuggestSourcesForNote(entry.path),
			});
		}
		items.push({
			label: $t('actions.delete'),
			icon: '🗑️',
			action: () => { confirmDelete = { path: entry.path, name: entry.name }; },
			danger: true
		});
		return items;
	}

	// MIG-021v2 §1E' — right-click action handler. Opens the Source Review
	// panel and emits a window event the panel listens for; the panel fires
	// the classifier IPC + prepends the resulting record to the visible queue.
	function handleSuggestSourcesForNote(notePath: string) {
		rightSidebarOpen = true;
		rightSidebarTab = 'sourceReview';
		// Defer the dispatch one frame so the panel has mounted (the listener
		// is attached in onMount).
		requestAnimationFrame(() => {
			window.dispatchEvent(new CustomEvent('constellation:classify-and-show', {
				detail: { notePath },
			}));
		});
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

	// MIG-008 §Build.3 — sidebar right-click "+ New Note" opens the shared
	// create-dialog with the right-clicked folder pre-filled as the parent.
	// §152 — Boss-approved: right-click now ALSO applies folder templates
	// (via the shared `createNoteWithTemplate` helper). Pre-§152 the
	// right-click path skipped templates while toolbar applied them — that
	// inconsistency is closed.
	function handleCreateNote(folderPath: string, libraryId: string) {
		const lib = $libraries.find(v => v.id === libraryId);
		if (!lib) return;
		createDialog = {
			kind: 'note',
			parentPath: folderPath,
			onCreate: ({ name, location }) => createNoteWithTemplate(lib, location, name),
		};
	}

	// MIG-008 §Build.4 — folder-based base creation (right-click on a folder).
	// Library is implicit from the parent folder, so no library multi-select.
	function handleCreateBase(folderPath: string, libraryId: string) {
		const lib = $libraries.find(v => v.id === libraryId);
		createDialog = {
			kind: 'base',
			parentPath: folderPath,
			onCreate: async ({ name, location }) => {
				const newPath = await createBase(location, name);
				if (!newPath) return;
				await refreshLibraryTree(libraryId);
				if (lib) {
					const libraryColor = libraryColorMap[lib.name] ?? '#7c3aed';
					await openNoteTab(newPath, lib.name, libraryColor);
				}
			},
		};
	}

	// MIG-008 §Build.2 — opens the shared create-dialog with the right-clicked
	// folder pre-filled as the parent location. Replaces the pre-MIG-008 inline
	// auto-create-as-"New Folder" flow per Boss directive 2026-05-03.
	function handleCreateFolder(parentPath: string, libraryId: string) {
		createDialog = {
			kind: 'folder',
			parentPath,
			onCreate: async ({ name, location }) => {
				await createFolder(location, name);
				await refreshLibraryTree(libraryId);
			},
		};
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

			const effectivePath = await renameItem(oldPath, newPath);
			// §137 (Rule 8 — Write-Time Derivation): every reactive Map keyed
			// by a file path must follow that path in the same transaction as
			// the rename. Without this, derived UI surfaces (file-tree stage
			// emoji + maturity dot, alias index, search-hub link counts) fall
			// out of sync — the symptom is "the stage icon disappeared after I
			// renamed it." `migratePathKeyedMap` returns null for no-op renames
			// (canonical-file path-stable case) so we skip the store update
			// entirely and don't fire spurious reactivity.
			// §139: stageMap + maturityMap are SvelteMap — in-place mutation
			// is reactive at the operation level. notePathToAliases +
			// searchLinkCounts are still $state(new Map()) so they need the
			// reassign-to-fresh-Map pattern to fire reactivity.
			migratePathKeyedMapInPlace(stageMap, oldPath, effectivePath);
			migratePathKeyedMapInPlace(maturityMap, oldPath, effectivePath);
			const aliasNext = migratePathKeyedMap(notePathToAliases, oldPath, effectivePath);
			if (aliasNext) notePathToAliases = aliasNext;
			const linkCountNext = migratePathKeyedMap(searchLinkCounts, oldPath, effectivePath);
			if (linkCountNext) searchLinkCounts = linkCountNext;
			const lib = $libraryStats.find(v => oldPath.startsWith(v.path));
			if (lib) {
				await refreshLibraryTree(lib.library_id);
				// Auto-update links — wikilink rename cascade.
				if ($appSettings.autoUpdateLinks && !isDir) {
					// §3-redo.5: orchestrate the cascade with full open-editor
					// coherence per Rename Function Concept Paper P4 / D2 / D6.
					// (a) Mark every open tab in the library as "cascading"
					//     so NoteEditor's handleSave/handleFlush and
					//     saveTabContent bail out for the duration. Without
					//     this gate, the reload's {#key}-bump destroy would
					//     call doFlush → handleFlush → writeNote with the
					//     editor's pre-cascade doc, undoing the cascade
					//     (BUG-015's F2 post-cascade-stomp class).
					// (b) flushAllTabsInLibrary writes any in-flight buffered
					//     edits to disk so the walker reads consistent state
					//     (closes F2 pre-cascade-staleness).
					// (c) updateLinksOnRename runs the cascade walker.
					// (d) reloadTabsFromDisk re-reads + batch-updates every
					//     rewritten tab. Awaiting it gives a real completion
					//     signal — no magic timeout, no listener race.
					// (e) Clear cascading marks in `finally` so an error
					//     anywhere above doesn't leave editors silenced.
					const tabs = tabsInLibrary(lib.path);
					for (const t of tabs) markCascading(t.path);
					try {
						await flushAllTabsInLibrary(lib.path);
						const result = await updateLinksOnRename(lib.path, lib.name, oldName, newName);
						await reloadTabsFromDisk(result.rewritten);
						// §144 (supersedes §143's targeted update — the targeted
						// approach only worked when the in-memory link's target
						// matched the rename's oldName exactly. After several
						// renames in a session (Hub v4 → v5 → v6 → v7), the
						// in-memory snapshot held v4 while subsequent renames
						// passed v6/v7 as oldName — so the targeted update kept
						// missing. §142 made SQLite the truth; the simplest
						// way to push that truth into the frontend is to
						// re-fetch the graph snapshot. Cost: same as boot's
						// graph fetch (~10-50ms per the cache.rs timings on
						// the reference Universe). Acceptable because rename
						// is already a multi-step user-initiated action.
						//
						// Catches: not just the just-renamed target's links,
						// but ANY drift accumulated during the session
						// (since allLibraryLinks isn't refreshed between
						// boots and renames before §144 left silent staleness
						// behind every time).
						if (result.rewritten.length > 0) {
							try {
								const graph = await invoke<{
									links: NoteLink[];
									tags: Record<string, number>;
									aliases: Array<{ note_path: string; alias: string }>;
								}>('cache_boot_snapshot_graph');
								allLibraryLinks = graph.links;
								// Refresh the alias index too — renames stamp a
								// new alias entry on the renamed file, which the
								// alias-aware Backlinks reader needs to see.
								const nextAliases = new Map<string, string[]>();
								for (const a of graph.aliases ?? []) {
									const existing = nextAliases.get(a.note_path);
									if (existing) existing.push(a.alias);
									else nextAliases.set(a.note_path, [a.alias]);
								}
								notePathToAliases = nextAliases;
							} catch (e) {
								console.error('[handleRenameComplete] graph re-fetch failed:', e);
							}
						}
					} finally {
						for (const t of tabs) clearCascading(t.path);
					}
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
	<div class="dock" data-style-target="cDock">
		<div class="dock-top">
			<button class="dock-btn" class:active={showSearchHub} onclick={() => {
				showSearchHub = !showSearchHub;
				if (showSearchHub) {
					searchHubInitialQuery = '';
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false; showInspector360 = false;
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
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; showCCS = false;
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
			<button class="dock-btn" class:active={showSkyView} onclick={() => { showSkyView = !showSkyView; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showKnowledgeHealth = false; showCCS = false; showInspector360 = false; showCataloger = false; }} title={$t('ribbon.graphView') || 'Sky View'}>
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
			<button class="dock-btn" class:active={showIndex} onclick={() => { showIndex = !showIndex; showSkyView = false; showGlobalTasks = false; showConstellationMap = false; showInspector360 = false; showCataloger = false; indexReturnPending = false; }} title={$t('ribbon.index')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/><path d="M8 7h6"/><path d="M8 11h8"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.cece !== false}
			<button class="dock-btn" class:active={showCataloger} onclick={() => {
				showCataloger = !showCataloger;
				if (showCataloger) {
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showKnowledgeHealth = false; showCCS = false; showInspector360 = false; showSearchHub = false; showExpressionForge = false; showSenseMakingCanvas = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false;
				}
				/* fullPageActive $effect handles sidebar snapshot */
			}} title={$t('ribbon.cataloger') || 'The Cataloger'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.constellationMap === true}
			<button class="dock-btn" class:active={showConstellationMap} onclick={() => { showConstellationMap = !showConstellationMap; showSkyView = false; showGlobalTasks = false; showIndex = false; showInspector360 = false; showCataloger = false; mapReturnPending = false; }} title={$t('ribbon.constellationMap') || 'Constellation Map'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>
			</button>
			{/if}
			<!-- MIG-017 (PJ-039): v2 Sight dock button gated behind SIGHT_V2_ENABLED.
			     v2 is preserved on disk as a known-good fallback while v3 (PJ-038) is built fresh. -->
			{#if SIGHT_V2_ENABLED && $appSettings.enabledFeatures?.constellationSight !== false}
			<button class="dock-btn" class:active={lensActive} onclick={() => {
				if (!lensActive) {
					toggleLens(); showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; lensReturnPending = false;
				} else {
					lensActive = false;
				}
				/* fullPageActive $effect handles sidebar snapshot */
			}} title={$t('lens.title') || 'Constellation Sight'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="2.5"/><line x1="12" y1="9.5" x2="12" y2="5"/><circle cx="12" cy="3.5" r="1.5"/><line x1="10.2" y1="13.8" x2="5.5" y2="18.5"/><circle cx="4" cy="20" r="1.5"/><line x1="13.8" y1="13.8" x2="18.5" y2="18.5"/><circle cx="20" cy="20" r="1.5"/></svg>
			</button>
			{/if}
			<!-- MIG-074 — CCS (Constellation Circulatory System), the circulatory
			     complement of CNS: placed directly after it (Q1 ruling). ECG
			     pulse-waveform icon — "the pulse of your thinking". -->
			{#if $appSettings.enabledFeatures?.ccs !== false}
			<button class="dock-btn" class:active={showCCS} onclick={() => {
				showCCS = !showCCS;
				if (showCCS) {
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showKnowledgeHealth = false; showInspector360 = false; showSearchHub = false; showExpressionForge = false; showSenseMakingCanvas = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false;
				}
			}} title={$t('ccs.title') || 'Constellation Circulatory System'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12h4l2-6 4 12 2-6h2"/><path d="M19.5 12a2.5 2.5 0 1 0 0-.01"/></svg>
			</button>
			{/if}
			<!-- MIG-018 (PJ-038): v3 Sight dock button (star-chart engine) — RETIRED.
			     v3 used position:fixed overlay; close button failed 13 times. Kept for
			     A/B testing but SIGHT_V3_ENABLED is false in production. -->
			{#if SIGHT_V3_ENABLED && $appSettings.enabledFeatures?.constellationSightV3 !== false}
			<button class="dock-btn" class:active={sightV3Active} onclick={() => {
				if (!sightV3Active) {
					sightV3Active = true;
					sightV4Active = false; showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; lensActive = false; lensReturnPending = false;
				} else {
					sightV3Active = false;
				}
			}} title={$t('sightV3.title') || 'Constellation Sight'} aria-label="Constellation Sight v3">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
			</button>
			{/if}
			<!-- MIG-019 v4 Sight dock button — clean-slate rebuild with SkyView's
			     mount pattern. Replaces v3 in production (SIGHT_V4_ENABLED = true). -->
			{#if SIGHT_V4_ENABLED && $appSettings.enabledFeatures?.constellationSightV3 !== false}
			<button class="dock-btn" class:active={sightV4Active} onclick={() => {
				if (!sightV4Active) {
					sightV4Active = true;
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; lensActive = false; sightV3Active = false; sightV5Active = false; sightV6Active = false; lensReturnPending = false;
				} else {
					sightV4Active = false;
				}
			}} title={$t('sightV3.title') || 'Constellation Sight'} aria-label="Constellation Sight">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
			</button>
			{/if}
			<!-- MIG-028 (2026-05-18): v5 Sight dock button retired with the v5 module set. -->

			<!-- MIG-025 §A.7 — Sight v6 dock button. Coordinated Views per
			     Concept Paper v4.0. B2 dual-mount: appears alongside v5 only
			     when SIGHT_V6_ENABLED is true (dev-flag gating per Architect
			     Option B2). User-settings flag is `constellationSightV6`
			     (fresh name; not the v3-era `constellationSightV3` quirk). -->
			{#if SIGHT_V6_ENABLED && $appSettings.enabledFeatures?.constellationSightV6 !== false}
			<button class="dock-btn" class:active={sightV6Active} onclick={() => {
				if (!sightV6Active) {
					sightV6Active = true;
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; lensReturnPending = false; sightV6ReturnPending = false;
				} else {
					sightV6Active = false;
					sightV6ReturnPending = false;
				}
			}} title={$t('sight.v6.title') || 'Constellation Sight'} aria-label="Constellation Sight">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
			</button>
			{/if}
			<!-- MIG-036 P1 (2026-05-19) — Sight v7 dock button. B2 dual-mount
			     with v6 during the Form-Aligns-To-Purpose redesign cascade.
			     Visible only when SIGHT_V7_ENABLED flag is true (dev flag,
			     false on release builds during v7 dev). -->
			{#if SIGHT_V7_ENABLED}
			<button class="dock-btn" class:active={sightV7Active} onclick={() => {
				if (!sightV7Active) {
					sightV7Active = true;
					showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false; lensReturnPending = false; sightV6ReturnPending = false;
				} else {
					sightV7Active = false;
				}
			}} title="Constellation Sight (v7 — Form-Aligns-To-Purpose)" aria-label="Constellation Sight v7">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.5" fill="currentColor"/></svg>
			</button>
			{/if}
			{#if $appSettings.enabledFeatures?.inspector360 !== false}
			<button class="dock-btn" class:active={showInspector360} onclick={() => {
				showInspector360 = !showInspector360;
				if (showInspector360) {
					showSkyView = false; showGlobalTasks = false; showIndex = false;
					showConstellationMap = false; showOrgChart = false; showCataloger = false; lensActive = false;
					showKnowledgeHealth = false; showCCS = false; showSearchHub = false;
					showExpressionForge = false; showSenseMakingCanvas = false;
				}
			}} title={$t('inspector360.title') || '360° Inspector'}>
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
				<!-- MIG-070 §C item D — Style Setter / inspect-to-restyle shortcut (above Settings). Opens
				     the Setter straight into inspect mode so you can click any chrome element to style it. -->
				<button class="dock-btn" onclick={() => openStyleSetterInspect()} title="Style Setter — click any element to restyle it" aria-label="Style Setter — inspect & restyle">
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/></svg>
				</button>
				<button class="dock-btn" class:active={showSettings} onclick={() => showSettings = !showSettings} title={$t('ribbon.settings')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
			</button>
		</div>
	</div>

	<!-- ═══ LEFT SIDEBAR ═══ -->
	{#if sidebarOpen && !skyViewInspectMode}
		<aside class="sidebar" data-style-target="cSidebar" style:width="{leftSidebarWidth}px">
			<div class="sidebar-toolbar" data-style-target="cToolbar">
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

					<!-- §Build.5 — New Library dropdown removed; the toolbar "+ Library"
					     button now opens the shared CreateItemDialog directly via
					     handleNewLibrary. "Link existing library" remains reachable
					     via the command palette (id 'add-library') and the Library
					     Manager screen. -->

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
					<!-- MIG-045 Phase 3 — Universe Digest mode tab. Reads through the
					     same summaryStore as every other Phase 1/2 surface; no new IPC. -->
					<button class="mode-tab" class:active={sidebarMode === 'digest'} onclick={() => { if (sidebarMode !== 'digest') { if (sidebarMode === 'tree') preTreeWidth = leftSidebarWidth; sidebarMode = 'digest'; leftSidebarWidth = Math.max(leftSidebarWidth, 360); emitSidebarModeChanged('digest'); } }} title={$t('navigator.digest') || 'Universe Digest'}>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7h18M3 12h12M3 17h6"/><circle cx="19" cy="17" r="2"/></svg>
					</button>
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
				{:else if sidebarMode === 'digest'}
					<!-- MIG-045 Phase 3 — Universe Digest. Reads existing
					     skyNodes + libraries arrays (already populated for
					     the Sky View); no new IPC, no new schema. -->
					<DigestPane
						nodes={skyNodes}
						libraries={$libraries}
						onNoteClick={(path, libName) => handleNoteClick(path, libName)}
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

					<!-- MIG-055 §F — Five Acts host notes section.
					     Lists `.md` files in `{universe}/Five Acts/`. The v1 set
					     contains "Observation — Recent Captures" auto-created by
					     Rust `init_five_acts_system_notes` at boot. Clicking an
					     entry opens the host note as a regular `.md` tab; the
					     embedded ` ```base ` lens block renders via §D's
					     LensBlockWidget. -->
					{#if fiveActsNotes.length > 0}
						<div class="library-section">
							<button class="library-header" onclick={() => fiveActsExpanded = !fiveActsExpanded}>
								<svg class="v-chev" class:expanded={fiveActsExpanded} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-inline-end: 4px; opacity: 0.6;"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
								<span class="library-name">{$t('sidebar.fiveActs')}</span>
							</button>
							{#if fiveActsExpanded}
								<div class="library-tree">
									{#each fiveActsActive as note (note.absolute_path)}
										<button
											class="ws-base-item"
											class:active={$activeTab?.path === note.absolute_path}
											onclick={() => openNoteTab(note.absolute_path, universeNotesStats?.name || activeUniverseName || 'Constellation', libraryColorMap[universeNotesStats?.name || ''] || 'var(--interactive-accent)')}
											title={note.relative_path}
										>
											<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity: 0.5; flex-shrink: 0;"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
											<span class="ws-base-name" dir={detectDir(note.display_name)}>{note.display_name}</span>
										</button>
									{/each}
									<!-- MIG-062 §E — federated cUniverse Five Acts, collapsible per universe -->
									{#each [...fiveActsByCu] as [cuName, notes] (cuName)}
										<button class="ws-cu-group" onclick={() => expandedCuFiveActs = toggleCuGroup(expandedCuFiveActs, cuName)} title={cuName}>
											<svg class="v-chev" class:expanded={expandedCuFiveActs.has(cuName)} width="8" height="8" viewBox="0 0 10 10"><path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/></svg>
											<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity: 0.5; flex-shrink: 0;"><circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/><path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/></svg>
											<span class="ws-cu-group-name" dir={detectDir(cuName)}>{cuName}</span>
											<span class="child-universe-count">{notes.length}</span>
										</button>
										{#if expandedCuFiveActs.has(cuName)}
											{#each notes as note (note.absolute_path)}
												<button
													class="ws-base-item ws-cu-item"
													class:active={$activeTab?.path === note.absolute_path}
													onclick={() => window.dispatchEvent(new CustomEvent('constellation:open-note', { detail: { path: note.absolute_path } }))}
													title={note.relative_path}
												>
													<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity: 0.5; flex-shrink: 0;"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
													<span class="ws-base-name" dir={detectDir(note.display_name)}>{note.display_name}</span>
												</button>
											{/each}
										{/if}
									{/each}
								</div>
							{/if}
						</div>
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
									{#each basesActive as base}
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
									<!-- MIG-062 §E — federated cUniverse bases, collapsible per universe.
									     READ-ONLY: open-only, NO context menu (deleting/renaming a
									     cUniverse's base would violate the read-only guarantee). -->
									{#each [...basesByCu] as [cuName, cuBases] (cuName)}
										<button class="ws-cu-group" onclick={() => expandedCuBases = toggleCuGroup(expandedCuBases, cuName)} title={cuName}>
											<svg class="v-chev" class:expanded={expandedCuBases.has(cuName)} width="8" height="8" viewBox="0 0 10 10"><path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/></svg>
											<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity: 0.5; flex-shrink: 0;"><circle cx="12" cy="12" r="6"/><line x1="6" y1="12" x2="18" y2="12"/><path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/></svg>
											<span class="ws-cu-group-name" dir={detectDir(cuName)}>{cuName}</span>
											<span class="child-universe-count">{cuBases.length}</span>
										</button>
										{#if expandedCuBases.has(cuName)}
											{#each cuBases as base (base.path)}
												<button
													class="ws-base-item ws-cu-item"
													class:active={$activeTab?.path === base.path}
													onclick={() => openNoteTab(base.path, cuName, '#7c3aed')}
													title={base.name}
												>
													<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity: 0.5; flex-shrink: 0;"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
													<span class="ws-base-name">{base.name}</span>
												</button>
											{/each}
										{/if}
									{/each}
								</div>
							{/if}
						</div>
					{/if}

					<!-- Universe Notes — folder named after the universe, shown above everything -->
					{#if universeNotesStats}
						<div class="library-section">
							<button class="library-header universe-notes-item" data-style-target="library" onclick={() => toggleLibrary(universeNotesStats)} oncontextmenu={(e) => handleLibraryHeaderContextMenu(e, universeNotesStats)}>
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
									entries={sortEntries(hideFiveActsFolder(libraryTrees[universeNotesStats.library_id]))}
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
							<button class="library-header child-universe-item" data-style-target="cuniverse" onclick={() => {
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
											<button class="library-header" data-style-target="library" onclick={() => toggleLibrary(lib)} oncontextmenu={(e) => handleLibraryHeaderContextMenu(e, lib)}>
												<svg class="v-chev" class:expanded={expandedLibraries.has(lib.library_id)} width="8" height="8" viewBox="0 0 10 10">
													<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
												</svg>
												<span class="library-name">{lib.name}</span>
											</button>
											{#if expandedLibraries.has(lib.library_id) && libraryTrees[lib.library_id]}
												<div class="library-tree">
													<FileTree
													entries={sortEntries(hideFiveActsFolder(libraryTrees[lib.library_id]))}
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
							<button class="library-header" data-style-target="library" onclick={() => toggleLibrary(lib)} oncontextmenu={(e) => handleLibraryHeaderContextMenu(e, lib)}>
								<svg class="v-chev" class:expanded={expandedLibraries.has(lib.library_id)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<span class="library-name">{lib.name}</span>
							</button>
							{#if expandedLibraries.has(lib.library_id) && libraryTrees[lib.library_id]}
								<div class="library-tree">
									<FileTree
									entries={sortEntries(hideFiveActsFolder(libraryTrees[lib.library_id]))}
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
							<button class="add-first-btn" data-style-target="cButtons" onclick={handleAddLibrary}>{$t('sidebar.addLibraryButton')}</button>
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
				<button class="sidebar-footer" data-style-target="universe" onmousedown={(e) => e.stopPropagation()} onclick={() => showLibrarySwitcher = !showLibrarySwitcher}>
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
		<div class="layout-bar" data-style-target="cLayoutBar">
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
		<div class="tab-bar" data-style-target="cTabs" class:tab-bar-hidden={fullPageActive} class:tab-bar-flanked-start={tabBarFlankLeft} class:tab-bar-flanked-end={tabBarFlankRight}>
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
			{#if lensReturnPending && SIGHT_V2_ENABLED}
				<button class="index-return-btn" onclick={() => { lensActive = true; lensReturnPending = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('lens.returnToLens') || 'Return to CNS'}
				</button>
			{/if}
			<!-- MIG-025 §B.6-fix-5 — Return to Sight (v6) button. Eisa cycle-3
			     ask: "I want a dedicated 'Return-to-Sight button' in-editor button."
			     Mirrors the lensReturnPending pattern. Visible only when the user
			     opened a note from Sight v6 (anchor click OR promoted-mini click);
			     clearing sets sightV6Active=true to re-open Sight. -->
			{#if sightV6ReturnPending && SIGHT_V6_ENABLED}
				<button class="index-return-btn" onclick={() => { sightV6Active = true; sightV6ReturnPending = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('sight.v6.returnToSight') || 'Return to Sight'}
				</button>
			{/if}
			{#if searchHubReturnPending}
				<button class="index-return-btn" onclick={() => { showSearchHub = true; searchHubReturnPending = false; showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; lensActive = false; sightV3Active = false; sightV4Active = false; sightV5Active = false; sightV6Active = false; showInspector360 = false; }}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					{$t('searchHub.title')}
				</button>
			{/if}
			{#if skyViewReturnPending}
				<button class="index-return-btn" onclick={() => { showSkyView = true; skyViewReturnPending = false; showSearchHub = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; lensActive = false; }}>
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
							loadMentions={(term) => readTermMentions(term, 500, $appSettings.index.expandCrossLanguage)}
							cacheKey={$appSettings.index.expandCrossLanguage}
							bridgeFilterEnabled={$appSettings.index.expandCrossLanguage}
							searchHistoryEnabled={$appSettings.index.searchHistoryEnabled}
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
					previousNoteName={inspector360BackStack.length > 0 ? inspector360BackStack[inspector360BackStack.length - 1].name : null}
					onNoteClick={(path, name) => {
						if (sidebarTab?.path && sidebarTab?.name) {
							inspector360BackStack = [...inspector360BackStack, { path: sidebarTab.path, name: sidebarTab.name }];
						}
						const lib = $libraryStats.find(l => path.startsWith(l.path));
						if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
					}}
					onBack={() => {
						if (inspector360BackStack.length === 0) return;
						const next = [...inspector360BackStack];
						const target = next.pop()!;
						inspector360BackStack = next;
						const lib = $libraryStats.find(l => target.path.startsWith(l.path));
						if (lib) openNoteTab(target.path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
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

		<!-- The Cataloger overlay (MIG-039) — CECE left-dock Core Plug-in,
		     lazy-mounted (LL-022). Right-sidebar Source Review tab is unchanged. -->
		{#if catalogerEverOpened}
			<div class="cataloger-overlay" class:cataloger-visible={showCataloger}>
				<CatalogerView
					visible={showCataloger}
					onNoteClick={(path, name) => {
						const lib = $libraryStats.find(l => path.startsWith(l.path));
						if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
						showCataloger = false;
					}}
					onClose={() => { showCataloger = false; }}
				/>
			</div>
		{/if}

		<!-- Constellation Sight v3 (MIG-018, PJ-038) — Pixi-based star-chart engine.
		     Outer SIGHT_V3_ENABLED gate prevents the overlay from mounting when v3
		     is disabled in committed source (default through §1A–§1E; flipped to
		     true in §1F after Boss-test passes). -->
		{#if sightV3Active && SIGHT_V3_ENABLED}
			<SightV3
				nodes={skyNodes}
				links={skyLinks}
				searchMatchIds={searchHubMatchIds}
				universeName={activeUniverseName}
				onClose={() => { sightV3Active = false; }}
				onOpenNote={(path: string, libraryName: string) => {
					const lib = $libraryStats.find(l => l.name === libraryName);
					const color = lib ? (libraryColorMap[libraryName] || '#7c3aed') : '#7c3aed';
					openNoteTab(path, libraryName, color);
				}}
			/>
			<!-- §2G.3p: External close button — lives in +layout.svelte,
			     NOT inside SightV3. Replicates SkyView's star-close pattern
			     (line 5108): the button directly sets the reactive variable,
			     no callback crossing component boundaries, no Pixi/Svelte
			     delegation interference. z-index 1001 sits above SightV3's
			     fixed overlay (z-index 1000). -->
			<button class="sight-v3-ext-close" onclick={() => { sightV3Active = false; }}>×</button>
		{/if}

		<!-- Constellation Sight (v2) — standalone D3+Canvas component.
		     MIG-017 (PJ-039): outer SIGHT_V2_ENABLED gate ensures the overlay never mounts
		     when v2 is disabled, even if `lensActive` is forced via DevTools. -->
		<div class="lens-overlay" class:lens-visible={lensActive && SIGHT_V2_ENABLED}>
			{#if lensActive && SIGHT_V2_ENABLED}
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
					{libraryColorMap}
					searchMatchIds={searchHubMatchIds}
					focusNoteId={pendingCnsFocusPath ?? undefined}
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
						pendingCnsFocusPath = null;
					}}
					onClose={() => { lensActive = false; lensReturnPending = false; pendingCnsFocusPath = null; }}
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
		<div class="content-area" class:content-hidden={showIndex || showConstellationMap || showOrgChart || showCataloger || lensActive || showSearchHub || showInspector360} onmouseover={handleWikilinkHover} onmouseout={handleWikilinkLeave}>
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
			{:else if sightV4Active && SIGHT_V4_ENABLED}
				<!-- Constellation Sight v4 — mounted INSIDE content-area as a
				     normal flex child, exactly like SkyView. This is the
				     architectural fix that eliminates the v3 close-button failure
				     (position:fixed overlay swallowed all pointer events). -->
				<div class="star-fullscreen sight-v4-fullscreen">
					<div class="star-header">
						<span class="star-title">{$t('sightV3.title') || 'Constellation Sight'}</span>
						<button class="star-close" onclick={() => sightV4Active = false}>×</button>
					</div>
					<SightV4
						nodes={skyNodes}
						links={skyLinks}
						searchMatchIds={searchHubMatchIds}
						universeName={activeUniverseName}
						onOpenNote={(path: string, libraryName: string) => {
							const lib = $libraryStats.find(l => l.name === libraryName);
							const color = lib ? (libraryColorMap[libraryName] || '#7c3aed') : '#7c3aed';
							openNoteTab(path, libraryName, color);
							sightV4Active = false;
						}}
					/>
				</div>
			<!-- MIG-028 (2026-05-18): v5 Sight modal block retired with the v5 module set. -->
			{:else if sightV6Active && SIGHT_V6_ENABLED}
				<!-- MIG-025 §A.7 — Constellation Sight v6 mount block.
				     Coordinated Views per Concept Paper v4.0: anchor dome +
				     facet sidebar + 4 mini-domes + tradition chip. §A.6
				     ships the placeholder; §A.8/§A.9 land the anchor render;
				     §A.10 lands the sidebar; §A.11 lands the tour. -->
				<div class="star-fullscreen sight-v6-fullscreen">
					<div class="star-header">
						<span class="star-title">{$t('sight.v6.title') || 'Constellation Sight'}</span>
						<button class="star-close" onclick={() => sightV6Active = false}>×</button>
					</div>
					<SightV6
						onOpenNote={(path: string, libraryName: string) => {
							const lib = $libraryStats.find(l => l.name === libraryName);
							const color = lib ? (libraryColorMap[libraryName] || '#7c3aed') : '#7c3aed';
							openNoteTab(path, libraryName, color);
							sightV6Active = false;
							// §B.6-fix-5: surface the Return-to-Sight button on
							// the note tab so the user can jump back to v6
							// without hunting for the dock icon.
							sightV6ReturnPending = true;
						}}
					/>
				</div>
			{:else if sightV7Active && SIGHT_V7_ENABLED}
				<!-- MIG-036 P1 (2026-05-19) — Sight v7 placeholder mount.
				     B2 dual-mount with v6 during the Form-Aligns-To-Purpose
				     redesign. P1 ships scaffolding only; subsequent phases
				     fill in the Hybrid X+Y rendering. -->
				<div class="star-fullscreen sight-v7-fullscreen">
					<div class="star-header">
						<span class="star-title">{$t('sight.v6.title') || 'Constellation Sight'} (v7)</span>
						<button class="star-close" onclick={() => sightV7Active = false}>×</button>
					</div>
					<SightV7
						onOpenNote={(path: string, libraryName: string) => {
							const lib = $libraryStats.find(l => l.name === libraryName);
							const color = lib ? (libraryColorMap[libraryName] || '#7c3aed') : '#7c3aed';
							openNoteTab(path, libraryName, color);
							sightV7Active = false;
						}}
					/>
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
							{#if tab.path && tab.path.endsWith('.base')}
								<!-- MIG-065 §F.2 — a standalone `.base` file renders as a
								     full-tab table on the unified `execute_lens` engine. -->
								<BaseTab path={tab.path} content={tab.content ?? ''} />
							{:else if tab.path}
								<NoteEditor
									{tab}
									noteNames={allNotes}
									allTags={allTagsList}
									{linkTraversalMap}
									onnavigateback={() => { setFocusedTab(tab.id); navigateBack(); }}
									onnavigateforward={() => { setFocusedTab(tab.id); navigateForward(); }}
									onStageChanged={handleStageChanged}
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
						{#if $activeTab.path.endsWith('.base')}
							<!-- MIG-065 §F.2 — standalone `.base` → full-tab table (no
							     note-flanks/focus; it's a base, not a note). -->
							<BaseTab path={$activeTab.path} content={$activeTab.content ?? ''} />
						{:else if focusMode}
							{@const _parsed = parseFrontmatter($activeTab.content || '')}
							<FocusPane
								value={SINGLE_OWNERSHIP ? seedBody(focusSessionId, focusSessionPath, _parsed.body) : _parsed.body}
								title={$activeTab.name.replace(/\.md$/, '')}
								dir={noteDir}
								onchange={(text) => {
									// MIG-076 §C — push the body to the model for the note FOCUS
									// WAS OPENED FOR (captured session id/path), never the live
									// $activeTab. The save composes from the model and is REFUSED on
									// a path mismatch, so a tab switch while in focus can never write
									// this body under another note's identity (the in-focus-switch
									// cross-note write, closed structurally).
									if (SINGLE_OWNERSHIP) {
										if (!focusSessionId) return;
										editNoteBody(focusSessionId, text);
										const r = composeNoteModel(focusSessionId, focusSessionPath);
										if (!r.ok) return;
										markNoteSaved(focusSessionId, r.version);
										const ft = get(openTabs).find(x => x.id === focusSessionId);
										if (ft) ft.content = r.content;
										markRecentWrite(focusSessionPath);
										writeNote(focusSessionPath, r.content, 'focus_pane').catch(() => {});
									} else {
										const currentTab = get(openTabs).find(x => x.id === $activeTab!.id);
										const props = currentTab ? parseFrontmatter(currentTab.content || '').properties : _parsed.properties;
										const fc = buildFullContent(props, text);
										if (currentTab) currentTab.content = fc;
										markRecentWrite($activeTab!.path);
										writeNote($activeTab!.path, fc, 'focus_pane').catch(() => {});
									}
								}}
								onexit={() => { focusMode = false; }}
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
													activeNoteName={$activeTab?.name ?? ''}
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
								onStageChanged={handleStageChanged}
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
											// MIG-076 §C — capture the focus note's identity for the
											// whole focus session. The auto-exit $effect resets
											// _focusModeTabId during a switch, so these dedicated
											// captures are what bind FocusPane's writes to the note it
											// was opened for — never the note the user switched to
											// (the in-focus-switch cross-note write).
											focusSessionId = $activeTab?.id ?? '';
											focusSessionPath = $activeTab?.path ?? '';
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
													activeNoteName={$activeTab?.name ?? ''}
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
								<button class="w-option-btn primary" onclick={handleNewLibrary}>
									+ {$t('libraries.newLibrary')}
								</button>
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
							<button class="w-dashboard-btn" data-style-target="cButtons" onclick={() => updateSettings({ showDashboard: true })}>
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
	<aside class="right-sidebar" data-style-target="cRightSidebar" class:collapsed={!rightSidebarOpen || skyViewInspectMode} style:width={(rightSidebarOpen && !skyViewInspectMode) ? rightSidebarWidth + 'px' : undefined}>
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
				{#if ($appSettings.panelPlacements?.inspector360 ?? 'right-sidebar') === 'right-sidebar'}
					<button class="rs-tab" class:active={rightSidebarTab === 'inspector360'} onclick={() => rightSidebarTab = 'inspector360'} title={$t('panels.inspector360') || $t('inspector360.title') || '360° Inspector'}>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="3"/><line x1="12" y1="3" x2="12" y2="9"/><line x1="12" y1="15" x2="12" y2="21"/><line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/></svg>
				</button>
				{/if}
				<!-- MIG-021 §1C — Source Review tab. Force-visible (not yet in panelPlacements). -->
				<button class="rs-tab" class:active={rightSidebarTab === 'sourceReview'} onclick={() => rightSidebarTab = 'sourceReview'} title={$t('sources.review.title') || 'Source Review'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
				</button>
			</div>

			{#if NOTE_SCOPED_TABS.has(rightSidebarTab) && !(isHome && sidebarTab)}
				<div class="rs-empty-full">{$t('panels.noNoteSelected')}</div>
			{:else if rightSidebarTab === 'tags'}
				<!-- Tags tab — universe-wide; pulled OUT of the note-gate so it
				     renders with or without an open note, and so the gate below
				     keeps narrowing sidebarTab non-null for the other panels. -->
				<div class="rs-section rs-full-height">
					<div class="rs-header rs-header-with-toggle rs-tags-header">
						<span>{$t('panels.tags')}{#if tagView === 'all' && Object.keys(allLibraryTags).length > 0} <span class="rs-tags-total">{Object.keys(allLibraryTags).length}</span>{/if}</span>
						<span class="rs-tag-toggle">
							<button class:active={tagView === 'note'} onclick={() => tagView = 'note'}>{$t('panels.tagsThisNote') || 'This note'}</button>
							<button class:active={tagView === 'all'} onclick={() => tagView = 'all'}>{$t('panels.tagsAll') || 'All tags'}</button>
						</span>
					</div>
					<div class="rs-tags-body">
					{#if tagView === 'note'}
						{#if sidebarTab && activeNoteTags.length > 0}
							<div class="rs-note-tags">
								{#each activeNoteTags as tag}
									<button class="rs-tag-chip" onclick={() => handleTagClick(tag)}>
										<span class="rs-tag-hash">#</span>{tag}
									</button>
								{/each}
							</div>
						{:else}
							<div class="rs-empty">{sidebarTab ? $t('panels.noTags') : $t('panels.noNoteSelected')}</div>
						{/if}
					{:else}
						<!-- Universe-wide federated tags (allLibraryTags). Click →
						     handleTagClick → federated Search Hub. -->
						{#if Object.keys(allLibraryTags).length > 0}
							<TagsPanel tags={allLibraryTags} onTagClick={handleTagClick} />
						{:else}
							<div class="rs-empty">{$t('panels.noTags')}</div>
						{/if}
					{/if}
					</div>
				</div>
			{:else if rightSidebarTab === 'properties' && sidebarTab}
					<!-- note-scoped: the empty-guard above already shows "no note" when !(isHome && sidebarTab);
					     the `&& sidebarTab` here narrows it non-null for the panel's tabId/path/libraryName. -->
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
							activeNoteName={sidebarTab?.name ?? ''}
							activeNotePath={sidebarTab?.path ?? ''}
							libraryPath={sidebarTab?.libraryPath ?? ''}
							{libraryColorMap}
							onConfidenceChange={applyConfidenceLocally}
							onArchive={applyArchiveLocally}
						/>
					</div>
				{:else if rightSidebarTab === 'star'}
					<!-- Local star centered on the active note -->
					<div class="rs-section rs-full-height">
						{#if localSkyNodes.length > 0}
							<LocalSkyView
								nodes={localSkyNodes}
								links={localSkyLinks}
								libraryColorMap={libraryColorMap}
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
							loading={tensionLoading}
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
				{:else if rightSidebarTab === 'inspector360'}
					<div class="rs-section rs-full-height">
						<Inspector360
							data={inspector360Data}
							compact={true}
							previousNoteName={inspector360BackStack.length > 0 ? inspector360BackStack[inspector360BackStack.length - 1].name : null}
							onNoteClick={(path, name) => {
								if (sidebarTab?.path && sidebarTab?.name) {
									inspector360BackStack = [...inspector360BackStack, { path: sidebarTab.path, name: sidebarTab.name }];
								}
								const lib = $libraryStats.find(l => path.startsWith(l.path));
								if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
							}}
							onBack={() => {
								if (inspector360BackStack.length === 0) return;
								const next = [...inspector360BackStack];
								const target = next.pop()!;
								inspector360BackStack = next;
								const lib = $libraryStats.find(l => target.path.startsWith(l.path));
								if (lib) openNoteTab(target.path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
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
				{:else if rightSidebarTab === 'sourceReview'}
					<!-- MIG-021 §1C — Source Review queue (works with or without an active note). -->
					<div class="rs-section rs-full-height">
						<SourceReviewPanel
							activeNotePath={sidebarTab?.path ?? null}
							onNoteClick={(path, name) => {
								const lib = $libraryStats.find(l => path.startsWith(l.path));
								if (lib) openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
							}}
						/>
					</div>
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

	<!-- MIG-070 — standalone Style Setter (full-page; self-shows via its store) -->
	<StyleSetter />

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
		<KnowledgeHealthDashboard
			onClose={() => showKnowledgeHealth = false}
			onOpenCcs={$appSettings.enabledFeatures?.ccs !== false ? (() => { showKnowledgeHealth = false; showCCS = true; }) : undefined}
		/>
	{/if}

	<!-- MIG-074 — CCS full-page surface. Plain {#if} mount (the KHD pattern):
	     a closed CCS is unmounted, so it does ZERO IPC while closed (LL-022);
	     reopening re-reads the cached snapshot (one ~ms call). -->
	{#if showCCS}
		<CCSView
			onClose={() => showCCS = false}
			onNoteClick={(path, libraryName) => {
				const lib = $libraryStats.find(l => l.name === libraryName) ?? $libraryStats.find(l => path.startsWith(l.path));
				if (lib) {
					openNoteTab(path, lib.name, libraryColorMap[lib.name] || '#7c3aed');
					showCCS = false;
				}
			}}
			onOpenKnowledgeHealth={() => { showCCS = false; showKnowledgeHealth = true; }}
			onOpenCns={SIGHT_V2_ENABLED && $appSettings.enabledFeatures?.constellationSight !== false ? (() => {
				// MIG-075 §B2 (Eisa, Stage 1): the way back — mirrors the CNS
				// dock button's open path (toggleLens serves cache or computes).
				showCCS = false;
				if (!lensActive) {
					toggleLens(); showSkyView = false; showGlobalTasks = false; showIndex = false; showConstellationMap = false; showOrgChart = false; showCataloger = false; showInspector360 = false; lensReturnPending = false;
				}
			}) : undefined}
		/>
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



	{#if contextMenu}
		<ContextMenu
			x={contextMenu.x}
			y={contextMenu.y}
			items={getContextMenuItems(contextMenu.entry, contextMenu.libraryId, contextMenu.isLibraryRoot)}
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

	{#snippet baseLibraryPickerExtras()}
		{@const allSelected = baseSelectedLibraries.length === 0}
		<div class="cd-base-libs">
			<div class="cd-base-libs-label">{$t('bases.source.librariesLabel') || 'Libraries to query'}</div>
			<div class="cd-base-libs-list">
				<label class="cd-base-libs-item" class:cd-base-libs-active={allSelected}>
					<input type="checkbox" checked={allSelected} onchange={() => baseSelectedLibraries = []} />
					<span>{$t('bases.source.allLibraries') || 'All libraries'}</span>
				</label>
				{#each $libraries as v}
					{@const isSelected = baseSelectedSet.has(v.name)}
					<label class="cd-base-libs-item" class:cd-base-libs-active={isSelected}>
						<input
							type="checkbox"
							checked={allSelected || isSelected}
							onchange={() => toggleBaseLibrary(v.name)}
						/>
						<span class="cd-base-libs-dot" style="background: {libraryColorMap[v.name] || '#7c3aed'}"></span>
						<span>{v.name}</span>
					</label>
				{/each}
			</div>
		</div>
	{/snippet}

	{#if createDialog}
		<CreateItemDialog
			open={true}
			kind={createDialog.kind}
			parentPath={createDialog.parentPath}
			hideLocation={createDialog.hideLocation}
			extras={createDialog.extrasKind === 'baseLibraryPicker' ? baseLibraryPickerExtras : undefined}
			onClose={() => createDialog = null}
			onCreate={createDialog.onCreate}
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
	<div class="status-bar" data-style-target="statusbar">
		<div class="sb-left">
			{#if sidebarTab}
				<span class="sb-item">{sidebarTab.libraryName}</span>
				<span class="sb-dot">·</span>
				<span class="sb-item">{sidebarTab.name}</span>
			{:else}
				<span class="sb-item">{$t('libraryManager.manageLibraries')}</span>
			{/if}
		</div>
		<!-- MIG-015 §1C — center slot for migration progress strips. Hidden
		     when no migration is in flight (the component returns nothing).  -->
		<div class="sb-center">
			<MigrationProgressStrip />
			<ClassifierScanProgressStrip />
				<NscBackfillProgressStrip />
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
			{#if federationWarnings.length > 0}
				<!-- MIG-056 §H — Federation warning badge. Surfaces when one
				     or more cUniverses failed to attach (skip_unavailable
				     model). Click to see details. -->
				<span class="sb-dot">·</span>
				<button class="sb-federation-warning" onclick={() => showFederationWarningsPopup = !showFederationWarningsPopup} title={$t('federation.warningBadge') || 'cUniverses unavailable'}>
					<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
					{federationWarnings.length}
				</button>
			{/if}
			{#if activeUniverseName}
				<span class="sb-dot">·</span>
				<button class="sb-universe" onclick={() => showUniverseManager = true} title={$t('universe.manager.heading')}>
					<svg width="10" height="10" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"><circle cx="100" cy="100" r="30" fill="#534AB7"/><circle cx="100" cy="100" r="19" fill="#3C3489"/><circle cx="45" cy="42" r="24" fill="#378ADD"/><circle cx="130" cy="52" r="20" fill="#7F77DD"/><circle cx="162" cy="110" r="16" fill="#1D9E75"/><circle cx="80" cy="158" r="13" fill="#D85A30"/></svg>
					{activeUniverseName}
				</button>
			{/if}

			<!-- MIG-056 §H — Federation warning popup. Lists each
			     unavailable cUniverse with its path + reason. -->
			{#if showFederationWarningsPopup && federationWarnings.length > 0}
				<div class="federation-popup" role="dialog" onclick={(e) => e.stopPropagation()}>
					<div class="federation-popup-header">
						<span>{$t('federation.popupTitle') || 'Federation warnings'}</span>
						<button class="federation-popup-close" onclick={() => showFederationWarningsPopup = false} title="Close">×</button>
					</div>
					<ul class="federation-popup-list">
						{#each federationWarnings as w}
							<li class="federation-popup-item">
								<div class="federation-popup-path" title={w.cuniverse_path}>
									<span class="federation-popup-label">{$t('federation.cuniverseLabel') || 'cUniverse'}:</span>
									{w.cuniverse_path}
								</div>
								<div class="federation-popup-reason">
									<span class="federation-popup-label">{$t('federation.reasonLabel') || 'Reason'}:</span>
									{w.reason}
								</div>
							</li>
						{/each}
					</ul>
				</div>
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
		border-inline-end: var(--border-width, 1px) solid var(--border);
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
		/* §C Phase 9 wiring-audit — sidebar bg is an override layered over the global panel bg
		   (no duplication: "Panel background" is the default, "Sidebar background" the specific override). */
		grid-row: 1; background: var(--sidebar-bg, var(--bg-secondary));
		border-inline-end: var(--border-width, 1px) solid var(--border);
		display: flex; flex-direction: column; overflow: hidden;
		position: relative;
	}
	.sidebar-toolbar {
		padding: 4px 6px;
		border-bottom: var(--border-width, 1px) solid var(--border);
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
		background: var(--background-primary, #fff); border: var(--border-width, 1px) solid var(--border);
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
		flex: 1; min-width: 0; padding: 3px 6px; border: var(--border-width, 1px) solid var(--border); border-radius: 4px;
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
		background: var(--bg); border: var(--border-width, 1px) solid var(--border); border-radius: 4px; padding: 0 6px;
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
		/* MIG-070 §3B G1 — libraries are their own element: --ft-library-* overrides the
		   File-tree master (--ft-master-*), which falls back to today's defaults. */
		color: var(--ft-library-color, var(--ft-master-color, var(--text-secondary)));
		font-size: var(--ft-library-font-size, var(--ft-master-font-size, 0.8rem));
		font-weight: var(--ft-library-weight, var(--ft-master-weight, 600));
		font-family: var(--ft-library-font-family, var(--ft-master-font-family, inherit)); cursor: pointer; text-align: start;
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
		/* §3B G1 — universe-notes is a library row → --ft-library-* (then master, then accent). */
		font-size: var(--ft-library-font-size, var(--ft-master-font-size, 0.8rem));
		color: var(--ft-library-color, var(--ft-master-color, var(--interactive-accent)));
		font-weight: var(--ft-library-weight, var(--ft-master-weight, 600));
		font-family: var(--ft-library-font-family, var(--ft-master-font-family, inherit));
	}
	.child-universe-item {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: var(--ft-master-row-padding-y, 3px) 12px;
		/* §3B G1 — child-universe (cUniverse) row → --ft-cuniverse-* (then master, then secondary). */
		font-size: var(--ft-cuniverse-font-size, var(--ft-master-font-size, 0.8rem));
		color: var(--ft-cuniverse-color, var(--ft-master-color, var(--text-secondary)));
		font-weight: var(--ft-cuniverse-weight, var(--ft-master-weight, 600));
		font-family: var(--ft-cuniverse-font-family, var(--ft-master-font-family, inherit));
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

	/* MIG-062 §E — federated cUniverse sub-group header (Five Acts / Bases). */
	.ws-cu-group {
		display: flex; align-items: center; gap: 5px; width: 100%;
		padding: 2px 12px 2px 18px; background: none; border: none;
		color: var(--text-muted); font-size: 0.72rem; font-family: inherit;
		cursor: pointer; text-align: start; border-radius: 3px;
	}
	.ws-cu-group:hover { background: var(--bg-hover); color: var(--text-normal); }
	.ws-cu-group .v-chev { transition: transform 0.12s; flex-shrink: 0; }
	.ws-cu-group .v-chev.expanded { transform: rotate(90deg); }
	.ws-cu-group-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	/* cUniverse items nest one level deeper than active-universe items. */
	.ws-cu-item { padding-inline-start: 32px; }

	.empty-sidebar { padding: 20px 16px; text-align: center; }
	.empty-sidebar p { color: var(--text-muted); font-size: 0.85rem; margin-bottom: 10px; }
	.add-first-btn {
		/* MIG-070 §C-polish Item C — also honours the "Buttons" element (--button-*); current
		   values as fallbacks so the empty-sidebar button is unchanged until styled. */
		background: none; border: 1px dashed var(--border); border-radius: var(--button-radius, 4px);
		padding: var(--button-padding-y, 4px) var(--button-padding-x, 12px); color: var(--text-muted); font-size: 0.82rem; cursor: pointer; font-family: inherit;
	}
	.add-first-btn:hover { border-color: var(--accent); color: var(--accent); }
	.sidebar-error { padding: 6px 12px; color: var(--danger); font-size: 0.75rem; }

	.sidebar-footer-wrap {
		position: relative;
	}
	.sidebar-footer {
		width: 100%;
		border-top: var(--border-width, 1px) solid var(--border); padding: 4px 12px;
		display: flex; align-items: center; gap: 6px; min-height: 30px;
		/* §3B — the "◊ Universe" switcher is its own element (colour cascades to the chevron + label). */
		color: var(--universe-bar-color, var(--text-normal));
		background: var(--universe-bar-bg, none); border-inline: none; border-bottom: none;
		cursor: pointer; font-family: var(--universe-bar-font-family, inherit);
	}
	.sidebar-footer:hover { background: var(--bg-hover); }
	.sidebar-footer svg { color: var(--text-muted); flex-shrink: 0; }
	/* §3B — the "Universe" label follows the Universe bar element (it had its own colour that
	   overrode the button's, so the colour control appeared to do nothing). */
	.footer-name { font-size: var(--universe-bar-font-size, 0.78rem); font-weight: 600; color: var(--universe-bar-color, var(--text-secondary)); }

	/* ═══ MAIN AREA ═══ */
	.main-area {
		grid-row: 1; display: flex; flex-direction: column;
		overflow: hidden; background: var(--center-zone-bg, #e8e8ec);
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
		gap: 1px; padding: 18px 4px 0; overflow-x: auto;
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
		background: var(--tab-active-bg, var(--background-primary, #ffffff)); color: var(--tab-active-color, var(--text-normal, var(--text)));
		border: 1px solid var(--tab-border, #d0d0d0);
		border-top: 3px solid var(--library-color, var(--accent));
		border-bottom: 1px solid var(--tab-active-bg, var(--background-primary, #ffffff));
		margin-bottom: -1px;
	}
	.tab.drag-over {
		border-inline-start: 3px solid var(--interactive-accent);
		background: color-mix(in srgb, var(--interactive-accent) 10%, var(--background-primary));
	}
	.tab[draggable="true"] { cursor: grab; }
	.tab[draggable="true"]:active { cursor: grabbing; }
	.tab-lib-name {
		position: absolute; bottom: calc(100% + 6px); inset-inline-end: 8px;
		font-size: 0.72rem; line-height: 1.3; letter-spacing: 0.02em;
		/* MIG-070 §3B — the library label above the tab is chrome → follows the interface
		   text colour (--text-normal), not the note (it had no colour, so it inherited the tab's). */
		color: var(--text-normal, var(--text));
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
	.content-area { flex: 1; overflow: hidden; display: flex; flex-direction: column; background: var(--center-zone-bg, #e8e8ec); }
	.content-area.content-hidden { display: none; }

	/* Pane container */
	.pane-container {
		flex: 1; display: flex; flex-direction: row; overflow: hidden; background: var(--center-zone-bg, #e8e8ec);
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

	.index-overlay, .map-overlay, .orgchart-overlay, .inspector360-overlay, .cataloger-overlay {
		display: none; flex: 1; overflow: hidden;
		background: var(--background-primary, #fff);
		min-height: 0;
	}
	.index-overlay.index-visible, .map-overlay.map-visible, .orgchart-overlay.orgchart-visible, .inspector360-overlay.inspector360-visible, .cataloger-overlay.cataloger-visible { display: flex; flex-direction: column; }

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
		padding: 8px 16px; border-bottom: var(--border-width, 1px) solid var(--border);
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
	/* §2G.3p: External Sight v3 close button — position:fixed above the
	   Sight overlay (z-index 1000). Replicates the SkyView star-close
	   pattern but positioned over a fixed full-screen overlay. */
	.sight-v3-ext-close {
		position: fixed;
		top: 8px;
		inset-inline-end: 16px;
		z-index: 1001;
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		background: rgba(250, 246, 232, 0.85);
		cursor: pointer;
		font-size: 22px;
		line-height: 28px;
		border-radius: 50%;
		color: rgba(26, 26, 26, 0.7);
		padding: 0;
		font-family: serif;
		font-weight: 600;
	}
	.sight-v3-ext-close:hover {
		color: #1a1a1a;
		background: rgba(201, 162, 39, 0.35);
	}
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
		padding: 4px 12px; border-bottom: var(--border-width, 1px) solid var(--border);
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
		padding: 8px 16px; border-bottom: var(--border-width, 1px) solid var(--border);
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
		/* MIG-070 §C-polish Item C — a real main-chrome generic button: honours the Style Setter's
		   "Buttons" element (--button-*). Current values kept as fallbacks → no change until edited. */
		margin-top: 16px; padding: var(--button-padding-y, 8px) var(--button-padding-x, 16px); border-radius: var(--button-radius, 8px);
		border: var(--border-width, 1px) solid var(--border); background: var(--bg-secondary);
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
		border: var(--border-width, 1px) solid var(--border); background: transparent;
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
		flex: 1; background: var(--bg-primary); border: var(--border-width, 1px) solid var(--border);
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
		width: 100%; padding: 0.45rem 0.6rem; border: var(--border-width, 1px) solid var(--border);
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
	.w-option-btn.secondary { background: var(--bg-secondary); color: var(--text); border: var(--border-width, 1px) solid var(--border); }
	.w-option-btn.secondary:hover { border-color: var(--accent); color: var(--accent); }
	.w-option-btn:disabled { opacity: 0.6; cursor: default; }
	.w-error { margin-top: 1rem; color: var(--danger, #cf222e); font-size: 0.85rem; }

	.page-scroll { flex: 1; overflow-y: auto; padding: 2rem; max-width: 900px; margin: 0 auto; width: 100%; }

	/* ═══ RIGHT SIDEBAR ═══ */
	.right-sidebar {
		grid-row: 1; background: var(--right-sidebar-bg, var(--bg-secondary));
		border-inline-start: var(--border-width, 1px) solid var(--border);
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
		display: flex; flex-wrap: wrap;
		border-bottom: var(--border-width, 1px) solid var(--border); flex-shrink: 0;
		background: var(--rs-tabs-bg, var(--right-sidebar-bg, var(--bg-secondary)));
		position: sticky; top: 0; z-index: 3;
	}
	.rs-tab {
		flex: 1 1 28px; min-width: 24px;
		display: flex; align-items: center; justify-content: center;
		height: var(--rs-tab-height, 30px);
		padding: 0;
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
	/* Tag Browser (#12) — Tags-tab header with the This-note ⇄ All-tags toggle. */
	.rs-header-with-toggle {
		display: flex; align-items: center; justify-content: space-between; gap: 8px;
	}
	.rs-tag-toggle {
		display: inline-flex; gap: 2px; text-transform: none; letter-spacing: 0;
	}
	.rs-tag-toggle button {
		padding: 2px 8px; font-size: 0.68rem; font-family: inherit;
		background: var(--bg-hover); border: var(--border-width, 1px) solid var(--border); color: var(--text-muted);
		cursor: pointer; border-radius: 4px;
	}
	.rs-tag-toggle button.active {
		background: var(--interactive-accent, var(--accent)); color: #fff; border-color: transparent;
	}
	.rs-tags-header { padding: 12px 12px 8px; margin-bottom: 0; flex-shrink: 0; }
		.rs-tags-total { font-size: 0.72rem; font-weight: 400; color: var(--text-faint); text-transform: none; letter-spacing: 0; margin-inline-start: 4px; }
		.rs-tags-body { flex: 1; min-height: 0; overflow-y: auto; padding: 0 10px 10px; }
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
		border-top: var(--border-width, 1px) solid var(--border);
		display: flex; align-items: center; justify-content: space-between;
		padding: 0 10px;
		font-size: var(--statusbar-font-size, 0.7rem);
		color: var(--statusbar-color, var(--text-muted));
	}
	.sb-left, .sb-right { display: flex; align-items: center; gap: 4px; }
	/* MIG-015 §1C — center group expands to fill available space and
	   centers its content. Empty (zero-width) when no migration strip
	   is rendered, so the existing left/right space-between layout is
	   visually unchanged in the common case. */
	.sb-center { flex: 1; display: flex; justify-content: center; align-items: center; }
	.sb-dot { color: var(--border); }
	.sb-universe {
		display: flex; align-items: center; gap: 3px;
		/* §3B — the status-bar universe name follows the Status bar text colour. */
		border: none; background: none; color: var(--statusbar-color, var(--text-secondary));
		font-size: inherit; font-family: inherit; cursor: pointer; padding: 0;
	}
	.sb-universe:hover { color: var(--interactive-accent); }

	/* MIG-056 §H — Federation warning badge + popup */
	.sb-federation-warning {
		display: inline-flex; align-items: center; gap: 4px;
		border: none; background: none; color: var(--text-warning, #d97706);
		font-size: inherit; font-family: inherit; cursor: pointer; padding: 0;
	}
	.sb-federation-warning:hover { color: var(--text-error, #e53e3e); }
	.federation-popup {
		position: fixed; bottom: 28px; inset-inline-end: 16px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.15);
		min-width: 280px; max-width: 480px; max-height: 320px;
		overflow-y: auto;
		z-index: 1000;
		font-size: 0.85em;
	}
	.federation-popup-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--background-modifier-border);
		font-weight: 600;
		color: var(--text-warning, #d97706);
	}
	.federation-popup-close {
		background: none; border: none; cursor: pointer;
		font-size: 1.2em; line-height: 1; padding: 0 4px;
		color: var(--text-muted);
	}
	.federation-popup-close:hover { color: var(--text-normal); }
	.federation-popup-list {
		list-style: none; padding: 0; margin: 0;
	}
	.federation-popup-item {
		padding: 8px 12px;
		border-bottom: 1px dotted var(--background-modifier-border);
	}
	.federation-popup-item:last-child { border-bottom: none; }
	.federation-popup-path {
		font-family: var(--font-monospace);
		font-size: 0.85em;
		word-break: break-all;
		margin-bottom: 4px;
	}
	.federation-popup-reason {
		color: var(--text-muted);
		font-style: italic;
	}
	.federation-popup-label {
		font-weight: 600;
		color: var(--text-faint);
		margin-inline-end: 4px;
	}

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

	/* MIG-008 §Build.4 — Base library multi-select extras inside CreateItemDialog */
	.cd-base-libs { display: flex; flex-direction: column; gap: 6px; }
	.cd-base-libs-label {
		font-size: 0.78rem;
		color: var(--text-muted);
		font-weight: 500;
	}
	.cd-base-libs-list {
		display: flex; flex-direction: column; gap: 2px;
		max-height: 160px; overflow-y: auto;
		border: 1px solid var(--background-modifier-border);
		border-radius: 5px; padding: 4px;
	}
	.cd-base-libs-item {
		display: flex; align-items: center; gap: 8px;
		padding: 4px 6px; border-radius: 4px;
		font-size: 0.82rem; color: var(--text-muted);
		cursor: pointer;
	}
	.cd-base-libs-item:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.cd-base-libs-item.cd-base-libs-active { color: var(--text-normal); }
	.cd-base-libs-item input[type="checkbox"] { cursor: pointer; accent-color: var(--interactive-accent); }
	.cd-base-libs-dot {
		width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
	}
</style>

<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { dir, t, tn, setLocale, type Locale } from '$lib/i18n';
	import {
		libraries, loadLibraries, appSettings, loadSettings,
		loadLibraryAppearance,
		openNoteTab, openTabs, activeTabId, activeTab,
		switchTab, closeTab,
		parseFrontmatter, buildFullContent,
		writeNote, markRecentWrite, wasRecentlyWritten, setWriteAhead, clearWriteAhead,
		renameItem,
		scanLibraryLinks, scanLibraryTags,
		toggleTaskReconciled,
		buildSkyData,
		libraryStats, loadAllStats,
		SCRIPT_UNICODE_RANGES, getFontSetById, hexToHSL,
		type SkyNode, type SkyLink
	} from '$lib/libraries/store';
	import { detectDir, renderMarkdown } from '$lib/utils';
	// G3 — the freshness-gated adopt primitive (adoptDisk): a clean model takes the
	// fresh disk content, a dirty model is never clobbered. Same primitive the main
	// window uses to adopt the SS's own writes (+layout.svelte:3329).
	import { externalChange as externalChangeNoteModel } from '$lib/editor/noteSession';
	import { SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';
	import { scanNoteTasks } from '$lib/tasks/store';
	import type { TaskItem } from '$lib/tasks/types';
	import TasksPanel from '$lib/components/TasksPanel.svelte';
	import { get } from 'svelte/store';
	import NotePane from '$lib/components/NotePane.svelte';
	import NoteEditor from '$lib/components/NoteEditor.svelte';
	import SaveHealthBanner from '$lib/components/SaveHealthBanner.svelte';
	import ConstellationMap from '$lib/components/ConstellationMap.svelte';
	import DashboardView from '$lib/components/DashboardView.svelte';
	import OrgChart from '$lib/components/OrgChart.svelte';
	import LocalSkyView from '$lib/components/LocalSkyView.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import { buildContextMenu, type ContextTarget, type ContextActions } from '$lib/components/contextMenuBuilder';
	import { onNoteMutation } from '$lib/noteMutations';
	import {
		onNoteToScreen, onNoteSaved, onUniverseSwitch, onSettingsChanged,
		onStateRequest, onWorkspaceRestore,
		onContextChanged, onSkyViewHover, onSkyViewClick,
		onSidebarModeChanged, onSplitModeChanged,
		onDashboardOpenNote, onDashboardTagSelected,
		onIndexTermSelected, onIndexCompare,
		onMapCompanion, onEditorPanels,
		listMonitors,
		sendNoteToMain, requestNoteActionOnMain, notifyScreenClosed, sendScreenState, emitScreenReady,
		type ScreenNote, type ScreenMode, type ScreenState, type ContextMode, type SkyViewNodeInfo, type SidebarMode,
		type SplitCompanionData, type DashboardTagData,
		type IndexTermData, type IndexCompareData,
		type MapCompanionData, type EditorPanelsData, type MonitorInfo
	} from '$lib/secondScreen';
	import {
		listUniverses, getChildUniverses,
		type ChildUniverseInfo
	} from '$lib/universe/store';
	function renderMarkdownPreview(raw: string): string {
		// MIG-071 audit HIGH (XSS) — render through the DOMPurify-sanitized renderMarkdown. Was raw
		// marked.parse(), which let a note body's <img onerror=…>/<script> execute on the 2nd screen.
		const body = raw.replace(/^---[\s\S]*?---\n?/, '').slice(0, 2000);
		try {
			return renderMarkdown(body);
		} catch {
			return `<p>${body.slice(0, 800).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')}</p>`;
		}
	}

	// ─── State ───
	let noteWidth = $state(100);
	let mainSidebarMode = $state<SidebarMode>('tree');
	let splitCompanionActive = $state(false);
	let splitCompanionData = $state<SplitCompanionData | null>(null);
	let splitCompanionTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks'>('properties');
	let preSplitSidebarMode = $state<SidebarMode>('tree');

	// Dashboard companion mode
	let dashboardMode = $state<'none' | 'note' | 'tag'>('none');
	let dashboardNoteTab = $state<any>(null); // OpenTab-like object for note editor
	let dashboardTagName = $state('');
	let dashboardTagNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let dashboardSelectedNote = $state<any>(null); // OpenTab-like for selected tag note

	// Index companion mode
	let indexMode = $state<'none' | 'term' | 'compare'>('none');
	let indexTermData = $state<IndexTermData | null>(null);
	let indexCompareData = $state<IndexCompareData | null>(null);
	let indexSelectedNote = $state<any>(null); // OpenTab-like for selected note
	let indexActiveCompareIdx = $state(0); // which term column is active in compare mode

	// Map companion mode
	let mapCompanionActive = $state(false);
	let mapCompanionData = $state<MapCompanionData | null>(null);
	let mapCompanionNoteTab = $state<any>(null); // for note click
	let mapCompanionColorMode = $state<'maturity' | 'stratum' | 'library'>('maturity');


	// Editor panels companion mode (migrated right sidebar)
	let editorPanelsActive = $state(false);
	let editorPanelsData = $state<EditorPanelsData | null>(null);
	let editorPanelsTab = $state<'properties' | 'backlinks' | 'tags' | 'star' | 'tasks'>('properties');
	let epBacklinks = $state<{ name: string; path: string; context: string; libraryName: string }[]>([]);
	let epForwardLinks = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let epTags = $state<string[]>([]);
	let epProperties = $state<{ key: string; value: any }[]>([]);
	let epLocalSkyNodes = $state<SkyNode[]>([]);
	let epLocalSkyLinks = $state<SkyLink[]>([]);
	let epTasks = $state<TaskItem[]>([]);

	// ─── MIG-096 §2 — the second screen's note right-click menu ───
	// Display-not-Domain: the menu is SHOWN here, but every mutating action is
	// PERFORMED by the main window (requestNoteActionOnMain → main's
	// handleOrgNodeMenuAction; the rename/move/delete dialog opens on main). Only
	// copy-path/copy-name act locally (a pure clipboard read). The menu is built
	// once via the SAME shared builder every other surface uses — no bespoke copy.
	let ssCtxMenu = $state<{ x: number; y: number; items: ReturnType<typeof buildContextMenu> } | null>(null);
	function showSSNoteMenu(path: string, name: string, e: MouseEvent) {
		e.preventDefault();
		const isMd = path.toLowerCase().endsWith('.md');
		const target: ContextTarget = { kind: 'note', path, name, isMarkdown: isMd };
		const fwd = (action: string) => () => { requestNoteActionOnMain(action, path, name); };
		const actions: ContextActions = {
			open: fwd('open'),
			openInNewTab: fwd('openInNewTab'),
			revealInTree: fwd('revealInTree'),
			bookmark: fwd('bookmark'),
			addTag: fwd('addTag'),
			copyPath: () => { navigator.clipboard.writeText(path).catch(() => {}); },
			copyName: () => { navigator.clipboard.writeText(name).catch(() => {}); },
			rename: fwd('rename'),
			move: fwd('move'),
			delete: fwd('delete'),
		};
		if (isMd) actions.suggestSources = fwd('suggestSources');
		ssCtxMenu = { x: e.clientX, y: e.clientY, items: buildContextMenu(target, actions) };
	}

	// Refresh-after-mutate: the companion lists derive from a scan of the
	// displayed note(s); after any rename/move/delete (here-forwarded or from the
	// main window), re-run the last panel scan so no stale name/dead row lingers.
	// Coalesced (onAnyChange, 300 ms). A stale 2nd-screen row is only ever a dead
	// click — the 2nd screen never writes — so best-effort re-scan is sufficient.
	let unlistenSSMutations: (() => void) | null = null;
	let ssMutationsDestroyed = false;
	onNoteMutation({
		onAnyChange: () => {
			if (splitCompanionData) loadSplitCompanionPanelData(splitCompanionData);
			if (editorPanelsData) loadEditorPanelsData(editorPanelsData);
		},
	}).then(u => { if (ssMutationsDestroyed) u(); else unlistenSSMutations = u; }).catch(() => {});
	onDestroy(() => { ssMutationsDestroyed = true; unlistenSSMutations?.(); });

	// Split companion — comparison panel data for all notes
	interface SplitPanelEntry {
		notePath: string;
		noteName: string;
		libraryName: string;
		libraryPath: string;
		backlinks: { name: string; path: string; context: string; libraryName: string }[];
		forwardLinks: { name: string; path: string; libraryName: string }[];
		tags: string[];
		properties: { key: string; value: any }[];
		localSkyNodes: SkyNode[];
		localSkyLinks: SkyLink[];
		tasks: TaskItem[];
	}
	let scPanels = $state<SplitPanelEntry[]>([]);

	// Monitor info
	let monitorCount = $state(0);

	// ─── Data state ───
	let allNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let loading = $state(true);
	// The second screen is a READ-ONLY display, always (Boss ruling 2026-07-09; PJ-068 v2).
	// It is a contextual complement, never an editing domain — every NoteEditor mount below
	// is read-only. Kept as a named constant so the 7 mounts read one source of truth.
	const ssReadOnly = true;
	let universeName = $state('');

	// ─── Dashboard state ───
	let childUniverses = $state<ChildUniverseInfo[]>([]);
	let childUniverseLibs = $state<Record<string, { name: string; path: string }[]>>({});
	let dashboardTags = $state<{ tag: string; count: number }[]>([]);
	let selectedTag = $state<string | null>(null);
	let selectedTagNotes = $state<{ name: string; path: string; libraryName: string }[]>([]);
	let loadingTagNotes = $state(false);

	// Batch-W — same stale-result shape as DashboardView.selectTag (notes_by_tag
	// is `(async)`; success AND failure of a stale call must not write).
	let tagLoadSeq = 0;

	async function selectTag(tag: string) {
		const seq = ++tagLoadSeq;
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
			if (seq !== tagLoadSeq) return; // a newer selection owns the write
			selectedTagNotes = notes.sort((a, b) => a.name.localeCompare(b.name));
		} catch {
			if (seq !== tagLoadSeq) return; // a newer selection owns the write
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
	let skyviewLocalNodes = $state<SkyNode[]>([]);
	let skyviewLocalLinks = $state<SkyLink[]>([]);
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
			const content = await invoke<string>('read_note', { filePath: node.path });
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

				const { nodes: sn, links: sl } = buildSkyData(links, allNotes.map(n => ({
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

	// ─── Library color map (shared utility) ───
	import { buildLibraryColorMap } from '$lib/libraries/colors';
	let libraryColorMap = $derived(buildLibraryColorMap($libraries));

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

	// ─── Load editor panels data for a note ───
	// Batch-W — scan_library_links is `(async)`: rapid main-window tab
	// switches can leave an older note's slow scan resolving after a newer
	// note's fast one; only the newest load may write the panels.
	let epGeneration = 0;
	/** Check if any descendant of a node is in the search match set. */
	async function loadEditorPanelsData(data: EditorPanelsData) {
		const gen = ++epGeneration;
		if (!data.notePath || !data.libraryPath) {
			epBacklinks = []; epForwardLinks = []; epTags = []; epProperties = [];
			epLocalSkyNodes = []; epLocalSkyLinks = [];
			return;
		}
		try {
			// Parse frontmatter for properties and tags
			const fm = parseFrontmatter(data.content || '');
			epProperties = fm.properties;
			const tagProp = fm.properties.find(p => p.key === 'tags');
			epTags = Array.isArray(tagProp?.value) ? tagProp.value : [];

			// Scan links
			const links = await scanLibraryLinks(data.libraryPath, data.libraryName || '').catch(() => []);
			if (gen !== epGeneration) return; // a newer note owns the panels
			const noteName = data.noteName?.replace(/\.md$/, '').toLowerCase() || '';

			// Backlinks
			epBacklinks = links
				.filter(l => l.target.toLowerCase() === noteName)
				.map(l => {
					const match = allNotes.find(n => n.path === l.source_path);
					return {
						name: match?.name || l.source_path.split(/[\\/]/).pop()?.replace(/\.md$/, '') || '',
						path: l.source_path,
						context: '',
						libraryName: data.libraryName || '',
					};
				})
				.filter((v, i, a) => a.findIndex(x => x.path === v.path) === i);

			// Forward links
			epForwardLinks = links
				.filter(l => l.source_path === data.notePath)
				.map(l => {
					const match = allNotes.find(n => n.name.toLowerCase() === l.target.toLowerCase());
					return match || { name: l.target, path: '', libraryName: data.libraryName || '' };
				})
				.filter(l => l.path)
				.filter((v, i, a) => a.findIndex(x => x.path === v.path) === i);

			// Local star
			const { nodes, links: skyLinks } = buildSkyData(links, allNotes);
			const connectedIds = new Set<string>([noteName]);
			for (const link of skyLinks) {
				if (link.source === noteName || link.target === noteName) {
					connectedIds.add(link.source);
					connectedIds.add(link.target);
				}
			}
			epLocalSkyNodes = nodes.filter(n => connectedIds.has(n.id));
			epLocalSkyLinks = skyLinks.filter(l => connectedIds.has(l.source) && connectedIds.has(l.target));
		// Tasks
			if (data.notePath && data.libraryPath) {
				const taskResult = await scanNoteTasks(data.notePath, data.libraryName || '', data.libraryPath).catch(() => null);
				if (gen !== epGeneration) return; // a newer note owns the panels
				epTasks = taskResult?.tasks ?? [];
			} else {
				epTasks = [];
			}
		} catch (e) {
			if (gen !== epGeneration) return; // a stale failure must not blank a newer note's panels
			console.error('[SS] loadEditorPanelsData failed:', e);
			epBacklinks = []; epForwardLinks = []; epTags = []; epProperties = [];
			epLocalSkyNodes = []; epLocalSkyLinks = []; epTasks = [];
		}
	}

	// Batch-W — same stale-result shape as epGeneration, for the split view.
	let scGeneration = 0;
	/** Load panel data for all split notes in parallel. */
	async function loadSplitCompanionPanelData(data: SplitCompanionData) {
		const gen = ++scGeneration;
		const notes = data.notes ?? [];
		if (notes.length === 0) { scPanels = []; return; }

		const results = await Promise.all(notes.map(async (note): Promise<SplitPanelEntry> => {
			const entry: SplitPanelEntry = {
				notePath: note.notePath, noteName: note.noteName,
				libraryName: note.libraryName, libraryPath: note.libraryPath,
				backlinks: [], forwardLinks: [], tags: [], properties: [],
				localSkyNodes: [], localSkyLinks: [], tasks: [],
			};
			try {
				const lib = $libraries.find(l => l.name === note.libraryName);
				const libraryPath = note.libraryPath || lib?.path || '';
				if (!libraryPath) return entry;

				const fm = parseFrontmatter(note.content || '');
				entry.properties = fm.properties;
				const tagProp = fm.properties.find(p => p.key === 'tags');
				entry.tags = Array.isArray(tagProp?.value) ? tagProp.value : [];

				const links = await scanLibraryLinks(libraryPath, note.libraryName || '').catch(() => []);
				const noteName = note.noteName?.replace(/\.md$/, '').toLowerCase() || '';

				entry.backlinks = links
					.filter(l => l.target.toLowerCase() === noteName)
					.map(l => {
						const match = allNotes.find(n => n.path === l.source_path);
						return { name: match?.name || l.source_path.split(/[\\/]/).pop()?.replace(/\.md$/, '') || '', path: l.source_path, context: '', libraryName: note.libraryName || '' };
					})
					.filter((v, i, a) => a.findIndex(x => x.path === v.path) === i);

				entry.forwardLinks = links
					.filter(l => l.source_path === note.notePath)
					.map(l => {
						const match = allNotes.find(n => n.name.toLowerCase() === l.target.toLowerCase());
						return match || { name: l.target, path: '', libraryName: note.libraryName || '' };
					})
					.filter(l => l.path)
					.filter((v, i, a) => a.findIndex(x => x.path === v.path) === i);

				const { nodes, links: skyLinks } = buildSkyData(links, allNotes);
				const connectedIds = new Set<string>([noteName]);
				for (const link of skyLinks) {
					if (link.source === noteName || link.target === noteName) { connectedIds.add(link.source); connectedIds.add(link.target); }
				}
				entry.localSkyNodes = nodes.filter(n => connectedIds.has(n.id));
				entry.localSkyLinks = skyLinks.filter(l => connectedIds.has(l.source) && connectedIds.has(l.target));

				const taskResult = await scanNoteTasks(note.notePath, note.libraryName || '', libraryPath).catch(() => null);
				entry.tasks = taskResult?.tasks ?? [];
			} catch (e) { console.error('[SS] loadSplitPanel failed for', note.noteName, e); }
			return entry;
		}));
		if (gen !== scGeneration) return; // a newer split set owns the write
		scPanels = results;
	}

	// ─── Data loading ───
	let initialLoadDone = false;
	async function loadAllData() {
		// Only show loading spinner on first startup — never interrupt an active editor
		if (!initialLoadDone) loading = true;
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
			initialLoadDone = true;
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

		// MIG-071 audit HIGH — the second screen must show the Style Setter look too (after MIG-071,
		// per-Universe styleOverride is the SOLE styling mechanism). Mirror the main window: standalone
		// accent, then the saved styleOverride applied LAST so it wins over the font defaults above.
		// styleOverride already carries the Setter's full accent decomposition (--interactive-accent /
		// --accent-h/s/l / --text-accent), so picking colours in the Setter restyles this window too.
		if (s.accentColor && s.accentColor !== '#7c3aed') {
			const hsl = hexToHSL(s.accentColor);
			root.setProperty('--accent-h', String(hsl.h));
			root.setProperty('--accent-s', `${hsl.s}%`);
			root.setProperty('--accent-l', `${hsl.l}%`);
		}
		for (const [k, v] of Object.entries(s.styleOverride ?? {})) {
			root.setProperty(k, v as string);
		}
	});

	// ─── Close handler ───
	async function handleClose() {
		notifyScreenClosed();
		try { await invoke('close_second_screen'); } catch {}
	}

	// ─── Event listeners ───
	let unlisteners: (() => void)[] = [];
	let libraryChangeTimer: ReturnType<typeof setTimeout> | null = null;

	/**
	 * G3 §2/§3 — adopt fresh disk content for `path` into EVERY second-screen note
	 * view that shows it, freshness-gated. The SS holds up to 7 NoteEditor mounts: the
	 * store `openTabs` (active editor + tab list) plus 5 companion `$state` tabs. For
	 * each matching view, `externalChangeNoteModel` (noteModel.adoptDisk) adopts ONLY
	 * when the model is clean AND the disk genuinely differs — a dirty editable-mode
	 * edit is never clobbered, an echo of our own write is ignored. On a real adopt we
	 * bump the tab's `reloadVersion` so NoteEditor's {#key} remounts NotePane and
	 * re-seeds from the freshly-adopted model (the display shows the new truth).
	 *
	 * SS mirror of the main window's onNoteSaved adopt (+layout.svelte:3320) and its
	 * cascade reload (+layout.svelte:3223) — but freshness-gated per the G3 Plan (NOT
	 * reloadTabsFromDisk, which force-adopts and would clobber a dirty editable SS tab).
	 */
	async function adoptFreshDiskIntoSS(path: string): Promise<void> {
		if (!SINGLE_OWNERSHIP || !path) return;
		// Skip the disk read entirely when NO SS view shows this note. A rename cascade can
		// rewrite dozens of backlinks (§3 loops over all of them), but the SS shows at most 7
		// distinct notes — reading every rewritten path would be mostly wasted IPC (Rule 3).
		const shownHere = get(openTabs).some((t) => t.path === path)
			|| [dashboardNoteTab, dashboardSelectedNote, indexSelectedNote, mapCompanionNoteTab, peekTab].some((c) => c?.path === path);
		if (!shownHere) return;
		let content: string;
		try {
			content = await invoke<string>('read_note', { filePath: path });
		} catch { return; }
		// Store openTabs (active editor + tab list). Adopt per tab (freshness-gated),
		// then bump reloadVersion ONLY on the tabs that actually adopted — so a mounted
		// view remounts to show the new truth while a dirty editable-mode edit is never
		// disrupted. (One-path-one-tab dedup means at most one match today.)
		const adoptedIds = new Set<string>();
		for (const t of get(openTabs)) {
			if (t.path === path && externalChangeNoteModel(t.id, content)) adoptedIds.add(t.id);
		}
		if (adoptedIds.size > 0) {
			openTabs.update((ts) => ts.map((t) =>
				adoptedIds.has(t.id) ? { ...t, content, reloadVersion: (t.reloadVersion ?? 0) + 1 } : t
			));
		}
		// Companion $state tabs — each adopts independently (freshness-gated).
		dashboardNoteTab = adoptCompanionTab(dashboardNoteTab, path, content);
		dashboardSelectedNote = adoptCompanionTab(dashboardSelectedNote, path, content);
		indexSelectedNote = adoptCompanionTab(indexSelectedNote, path, content);
		mapCompanionNoteTab = adoptCompanionTab(mapCompanionNoteTab, path, content);
		peekTab = adoptCompanionTab(peekTab, path, content);
	}

	/** G3 — adopt fresh disk into one companion tab (freshness-gated). Returns the
	 *  same object untouched when it doesn't match / the model is dirty / the payload is
	 *  an echo; otherwise a NEW object with the fresh content + a bumped reloadVersion so
	 *  Svelte re-renders and NoteEditor's {#key} remounts with the adopted model. */
	function adoptCompanionTab(tab: any, path: string, content: string): any {
		if (!tab || tab.path !== path) return tab;
		if (!externalChangeNoteModel(tab.id, content)) return tab; // dirty edit or echo → leave it
		return { ...tab, content, reloadVersion: (tab.reloadVersion ?? 0) + 1 };
	}

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
			if (wasRecentlyWritten(path)) return; // we saved it ourselves — skip reload
			// Reload editor panels if the saved note is the one we're displaying
			if (editorPanelsActive && editorPanelsData?.notePath === path) {
				// Re-read content and reload panels
				try {
					const content = await invoke<string>('read_note', { filePath: path });
					const updated = { ...editorPanelsData, content };
					editorPanelsData = updated;
					await loadEditorPanelsData(updated);
				} catch {}
			}
			if (splitCompanionActive && splitCompanionData?.notes?.some(n => n.notePath === path)) {
				// Re-read the changed note's content and reload all panels
				try {
					const content = await invoke<string>('read_note', { filePath: path });
					const updatedNotes = splitCompanionData.notes!.map(n =>
						n.notePath === path ? { ...n, content } : n
					);
					splitCompanionData = { ...splitCompanionData, notes: updatedNotes };
					await loadSplitCompanionPanelData(splitCompanionData);
				} catch {}
			}
			// G3 §2 — adopt the main window's save into EVERY SS editor view of this
			// note (active tab + companions), freshness-gated so an editable-mode edit
			// on the SS is never clobbered. Fixes HIGH #2 (SS never adopted main→SS saves).
			await adoptFreshDiskIntoSS(path);
		});
		unlisteners.push(u2);

		// G3 §3 — react to a main-window rename cascade. `cascade:rewrote` is emitted
		// app-wide from Rust (libraries.rs:1618/:5483) so it reaches this separate SS
		// realm; the rewrite listener only lives in +layout (main window). For each
		// rewritten path, adopt the canonical cascade result into every SS view of it
		// (freshness-gated). Fixes HIGH #1 (SS blind to the cascade → stale [[wikilink]]).
		const uCascade = await listen<{ paths: string[] }>('cascade:rewrote', async (event) => {
			for (const p of event.payload?.paths ?? []) {
				await adoptFreshDiskIntoSS(p);
			}
		});
		unlisteners.push(uCascade);

		// Listen for universe switch
		const u3 = await onUniverseSwitch(async () => {
			try {
				const universes = await listUniverses();
				if (universes.length > 0) {
					universeName = universes[0].name || '';
					getCurrentWindow().setTitle(`Constellation - ${universeName}`).catch(() => {});
				}
			} catch {}
			allNotes = []; // Clear stale notes before rebuild
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

		// F2′ — the app's own gated creates are watcher-suppressed, so
		// `library-changed` never fires for them; `note-created` (emitted by
		// the store's createNote, any window) keeps this screen's note list
		// and dashboard in step. Same debounced reload as u5.
		const u5b = await listen<{ path: string }>('note-created', async () => {
			if (libraryChangeTimer) clearTimeout(libraryChangeTimer);
			libraryChangeTimer = setTimeout(async () => { await loadAllData(); await loadDashboardData(); }, 3000);
		});
		unlisteners.push(u5b);

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

		const u13 = await onDashboardOpenNote(async (note) => {
			dashboardMode = 'note';
			dashboardTagName = '';
			dashboardTagNotes = [];
			dashboardSelectedNote = null;
			try {
				const content = await invoke<string>('read_note', { filePath: note.path });
				dashboardNoteTab = {
					id: `dash-note-${Date.now()}`,
					path: note.path,
					content,
					name: note.name.endsWith('.md') ? note.name : note.name + '.md',
					libraryName: note.libraryName,
					libraryPath: note.libraryPath,
					libraryColor: note.libraryColor,
					history: [note.path],
					historyIndex: 0,
				};
			} catch { dashboardNoteTab = null; }
		});
		unlisteners.push(u13);

		const u14 = await onDashboardTagSelected(async (data) => {
			dashboardMode = 'tag';
			dashboardNoteTab = null;
			dashboardSelectedNote = null;
			dashboardTagName = data.tag;
			dashboardTagNotes = data.notes;
		});
		unlisteners.push(u14);

		const u15 = await onIndexTermSelected(async (data) => {
			indexMode = 'term';
			indexTermData = data;
			indexCompareData = null;
			indexSelectedNote = null;
			dashboardMode = 'none';
		});
		unlisteners.push(u15);

		const u16 = await onIndexCompare(async (data) => {
			indexMode = 'compare';
			indexCompareData = data;
			indexTermData = null;
			indexSelectedNote = null;
			indexActiveCompareIdx = 0;
			dashboardMode = 'none';
		});
		unlisteners.push(u16);

		const u17 = await onMapCompanion(async (data) => {
			if (!data.active) {
				mapCompanionActive = false;
				mapCompanionData = null;
				mapCompanionNoteTab = null;
				return;
			}
			mapCompanionActive = true;
			mapCompanionData = data;
			if (data.colorMode) mapCompanionColorMode = data.colorMode as any;
			// Reset other modes
			dashboardMode = 'none';
			indexMode = 'none';

			// If a note was clicked, load it for editing
			if (data.clickedNote) {
				try {
					const content = await invoke<string>('read_note', { filePath: data.clickedNote.path });
					mapCompanionNoteTab = {
						id: `map-note-${Date.now()}`,
						path: data.clickedNote.path,
						content,
						name: data.clickedNote.name.endsWith('.md') ? data.clickedNote.name : data.clickedNote.name + '.md',
						libraryName: data.clickedNote.libraryName,
						libraryPath: data.clickedNote.libraryPath,
					};
				} catch { mapCompanionNoteTab = null; }
			} else {
				mapCompanionNoteTab = null;
			}
		});
		unlisteners.push(u17);

		const u12 = await onSplitModeChanged(async (data) => {
			if (data.active) {
				if (!splitCompanionActive) preSplitSidebarMode = mainSidebarMode;
				splitCompanionActive = true;
				splitCompanionData = data;
				await loadSplitCompanionPanelData(data);
			} else {
				splitCompanionActive = false;
				splitCompanionData = null;
				mainSidebarMode = preSplitSidebarMode;
			}
		});
		unlisteners.push(u12);

		// Editor panels companion (migrated right sidebar)
		const u18 = await onEditorPanels(async (data) => {
			if (!data.active) {
				editorPanelsActive = false;
				editorPanelsData = null;
				return;
			}
			editorPanelsActive = true;
			editorPanelsData = data;
			// Reset other companion modes
			dashboardMode = 'none';
			indexMode = 'none';
			mapCompanionActive = false;
			mapCompanionData = null;
			mapCompanionNoteTab = null;
			// Load panel data
			await loadEditorPanelsData(data);
		});
		unlisteners.push(u18);

		// Detect monitors
		try { monitorCount = (await listMonitors()).length; } catch { monitorCount = 1; }

		// Signal main window that all listeners are ready
		await emitScreenReady();

		// Now load data (after listeners are set up so no events are missed)
		try {
			const universes = await listUniverses();
			if (universes.length > 0) {
				// MIG-079 §A — Display-Not-Domain: the second screen must NOT activate
				// the universe (that re-inits the search DB = the double-init). The main
				// window owns activation and listUniverses() returns the active universe
				// first, so we only READ its name to title this display window.
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
	<!-- Save-Durability — the save-failure surface (second-screen JS context has its own saveHealth) -->
	<SaveHealthBanner />
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
		{:else if dashboardMode === 'note' && dashboardNoteTab}
			<!-- Dashboard: single note editor -->
			<div class="dash-note-companion" dir={detectDir(dashboardNoteTab?.name || '')}>
				<div class="dash-note-header">
					<button class="dash-back-btn" onclick={() => { dashboardMode = 'none'; dashboardNoteTab = null; }} title={$t('notePane.back') || 'Back to Dashboard'}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					</button>
				</div>
				<div class="dash-note-editor">
					<NoteEditor tab={dashboardNoteTab} noteNames={allNotes} readOnly={ssReadOnly} />
				</div>
			</div>

		{:else if dashboardMode === 'tag' && dashboardTagNotes.length > 0}
			<!-- Dashboard: tag notes split view -->
			<div class="dash-tag-companion" dir={detectDir(dashboardTagName)}>
				<div class="dash-tag-header">
					<button class="dash-back-btn" onclick={() => { dashboardMode = 'none'; dashboardTagName = ''; dashboardTagNotes = []; dashboardSelectedNote = null; }}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					</button>
					<span class="dash-tag-badge">#{dashboardTagName}</span>
					<span class="dash-tag-count">{$tn('plurals.notes', dashboardTagNotes.length)}</span>
				</div>
				<div class="dash-tag-split">
					<div class="dash-tag-list">
						{#each dashboardTagNotes as note}
							<button class="dash-tag-note" class:active={dashboardSelectedNote?.path === note.path}
								onclick={async () => {
									try {
										const content = await invoke('read_note', { filePath: note.path });
										const lib = $libraries.find(l => l.name === note.libraryName);
										dashboardSelectedNote = {
											id: `dash-tag-${Date.now()}`,
											path: note.path,
											content,
											name: note.name.endsWith('.md') ? note.name : note.name + '.md',
											libraryName: note.libraryName,
											libraryPath: lib?.path ?? '',
											libraryColor: libraryColorMap[note.libraryName] || '#7c3aed',
										};
									} catch { dashboardSelectedNote = null; }
								}}>
								<span class="lib-dot" style="background:{libraryColorMap[note.libraryName] || '#7c3aed'}"></span>
								<span class="dash-tag-note-name" dir="auto">{note.name.replace(/\.md$/, '')}</span>
							</button>
						{/each}
					</div>
					<div class="dash-tag-editor">
						{#if dashboardSelectedNote}
							<NoteEditor tab={dashboardSelectedNote} noteNames={allNotes} readOnly={ssReadOnly} />
						{:else}
							<div class="dash-tag-empty">
								<p>{$t('secondScreen.selectNote') || 'Select a note to view here'}</p>
							</div>
						{/if}
					</div>
				</div>
			</div>

		{:else if indexMode === 'term' && indexTermData}
			<!-- Index: single term → note list + editor -->
			<div class="dash-tag-companion" dir={detectDir(indexTermData.term)}>
				<div class="dash-tag-header">
					<button class="dash-back-btn" onclick={() => { indexMode = 'none'; indexTermData = null; indexSelectedNote = null; }}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					</button>
					<span class="dash-tag-badge">{indexTermData.term}</span>
					<span class="dash-tag-count">{$tn('plurals.notes', indexTermData.notes.length)}</span>
				</div>
				<div class="dash-tag-split">
					<div class="dash-tag-list">
						{#each indexTermData.notes as note}
							{@const matched = allNotes.find(n => n.path === note.note_path)}
							{@const lib = matched ? $libraries.find(l => l.name === matched.libraryName) : null}
							<button class="dash-tag-note" class:active={indexSelectedNote?.path === note.note_path}
								onclick={async () => {
									try {
										const content = await invoke('read_note', { filePath: note.note_path });
										indexSelectedNote = {
											id: `idx-term-${Date.now()}`,
											path: note.note_path,
											content,
											name: note.note_name.endsWith('.md') ? note.note_name : note.note_name + '.md',
											libraryName: lib?.name ?? '',
											libraryPath: lib?.path ?? '',
											highlightTerm: indexTermData?.term ?? '',
										};
									} catch { indexSelectedNote = null; }
								}}>
								<span class="lib-dot" style="background:{libraryColorMap[matched?.libraryName ?? ''] || '#7c3aed'}"></span>
								<span class="dash-tag-note-name" dir="auto">{note.note_name}</span>
							</button>
						{/each}
					</div>
					<div class="dash-tag-editor">
						{#if indexSelectedNote}
							<NoteEditor tab={indexSelectedNote} noteNames={allNotes} readOnly={ssReadOnly} />
						{:else}
							<div class="dash-tag-empty">
								<p>{$t('secondScreen.selectNote') || 'Select a note to view here'}</p>
							</div>
						{/if}
					</div>
				</div>
			</div>

		{:else if indexMode === 'compare' && indexCompareData && indexCompareData.terms.length > 0}
			<!-- Index: multi-term compare -->
			<div class="index-compare">
				<div class="dash-tag-header">
					<button class="dash-back-btn" onclick={() => { indexMode = 'none'; indexCompareData = null; indexSelectedNote = null; }}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
					</button>
					<span class="index-compare-label">{$t('secondScreen.comparingTerms') || 'Comparing'} {$tn('plurals.terms', indexCompareData.terms.length)}</span>
				</div>
				<div class="index-compare-body">
					<div class="index-compare-columns">
						{#each indexCompareData.terms as termData, idx}
							<div class="index-compare-col" class:active={indexActiveCompareIdx === idx} dir={detectDir(termData.term)}>
								<div class="index-compare-col-header" onclick={() => indexActiveCompareIdx = idx}>
									<span class="dash-tag-badge">{termData.term}</span>
									<span class="dash-tag-count">{termData.notes.length}</span>
								</div>
								<div class="index-compare-col-list">
									{#each termData.notes as note}
										{@const matched = allNotes.find(n => n.path === note.note_path)}
										{@const lib = matched ? $libraries.find(l => l.name === matched.libraryName) : null}
										<button class="dash-tag-note" class:active={indexSelectedNote?.path === note.note_path}
											onclick={async () => {
												indexActiveCompareIdx = idx;
												try {
													const content = await invoke('read_note', { filePath: note.note_path });
													indexSelectedNote = {
														id: `idx-cmp-${Date.now()}`,
														path: note.note_path,
														content,
														name: note.note_name.endsWith('.md') ? note.note_name : note.note_name + '.md',
														libraryName: lib?.name ?? '',
														libraryPath: lib?.path ?? '',
														highlightTerm: termData.term,
													};
												} catch { indexSelectedNote = null; }
											}}>
											<span class="dash-tag-note-name" dir="auto">{note.note_name}</span>
										</button>
									{/each}
								</div>
							</div>
						{/each}
					</div>
					<div class="index-compare-editor">
						{#if indexSelectedNote}
							<NoteEditor tab={indexSelectedNote} noteNames={allNotes} readOnly={ssReadOnly} />
						{:else}
							<div class="dash-tag-empty">
								<p>{$t('secondScreen.selectNote') || 'Select a note to view here'}</p>
							</div>
						{/if}
					</div>
				</div>
			</div>

		{:else if mapCompanionActive && mapCompanionData}
			<!-- Constellation Map companion -->
			<div class="map-companion" dir={detectDir(mapCompanionData?.focusNode?.name || '')}>
				{#if mapCompanionData.clickedNote && mapCompanionNoteTab}
					<!-- Note view: editor + context mini-map -->
					<div class="map-companion-note">
						<div class="map-companion-editor">
							<NoteEditor tab={mapCompanionNoteTab} noteNames={allNotes} readOnly={ssReadOnly} />
						</div>
						{#if mapCompanionData.focusNode}
							<div class="map-companion-context">
								<ConstellationMap
									initialData={mapCompanionData.focusNode}
									{libraryColorMap}
									compact={true}
									initialColorMode={mapCompanionColorMode}
									onNoteClick={(path, name) => {
										const n = allNotes.find(x => x.path === path);
										const lb = n ? $libraries.find(l => l.name === n.libraryName) : null;
										sendNoteToMain({ path, name, libraryName: n?.libraryName ?? '', libraryPath: lb?.path ?? '', libraryColor: libraryColorMap[n?.libraryName ?? ''] || '#7c3aed' });
									}}
								/>
							</div>
						{/if}
					</div>
				{:else if mapCompanionData.focusNode?.children}
					<!-- Drill-down view: grid of child mini-maps -->
					{@const focusNode = mapCompanionData.focusNode}
					{@const children = focusNode.children || []}
					{@const dirChildren = children.filter((c: any) => c.is_dir)}
					<div class="map-companion-grid-header">
						<span class="map-companion-title" dir="auto">{focusNode.name}</span>
						<span class="map-companion-stats">{focusNode.note_count} notes · {focusNode.word_count?.toLocaleString()} words</span>
						<select class="map-companion-color-select" bind:value={mapCompanionColorMode}>
							<option value="maturity">{$t('constellationMap.colorByMaturity') || 'Color by Maturity'}</option>
							<option value="stratum">{$t('constellationMap.colorByStratum') || 'Color by Stratum'}</option>
							<option value="library">{$t('constellationMap.colorByLibrary') || 'Color by Library'}</option>
						</select>
					</div>
					<div class="map-companion-legend">
						{#if mapCompanionColorMode === 'maturity'}
							{#each [['seed','#d1d5db'],['sapling','#86efac'],['evergreen','#16a34a'],['canonical','#f59e0b'],['wilting','#a3e635']] as [label, color]}
								<span class="map-legend-item"><span class="map-legend-dot" style="background:{color}"></span>{label}</span>
							{/each}
						{:else if mapCompanionColorMode === 'stratum'}
							{#each ['#3b82f6','#6366f1','#8b5cf6','#a855f7','#d946ef','#ec4899','#f43f5e','#ef4444'] as color, i}
								<span class="map-legend-item"><span class="map-legend-dot" style="background:{color}"></span>L{i + 1}</span>
							{/each}
						{:else if mapCompanionColorMode === 'library'}
							{#each Object.entries(libraryColorMap) as [name, color]}
								<span class="map-legend-item"><span class="map-legend-dot" style="background:{color}"></span>{name}</span>
							{/each}
						{/if}
					</div>
					{#if dirChildren.length > 0}
						<div class="map-companion-grid">
							{#each dirChildren as child (child.path || child.name)}
								<div class="map-companion-card">
									<div class="map-companion-card-label" dir="auto">{child.name}</div>
									<div class="map-companion-card-chart">
										<ConstellationMap
											initialData={child}
											{libraryColorMap}
											compact={true}
											initialColorMode={mapCompanionColorMode}
										/>
									</div>
									<div class="map-companion-card-stats">{child.note_count} notes</div>
								</div>
							{/each}
						</div>
					{:else}
						<div class="map-companion-full">
							<ConstellationMap
								initialData={focusNode}
								{libraryColorMap}
								compact={true}
								initialColorMode={mapCompanionColorMode}
							/>
						</div>
					{/if}
				{:else}
					<div class="dash-tag-empty">
						<p>Map companion</p>
					</div>
				{/if}
			</div>

		{:else if splitCompanionActive && splitCompanionData}
			<!-- Split View — Comparison Panels (all notes side by side) -->
			<div class="split-companion">
				<div class="split-companion-header">
					<span class="split-companion-label">{$t('secondScreen.splitCompanion') || 'Split Comparison'}</span>
					<span class="split-companion-count">{$tn('plurals.notes', scPanels.length)}</span>
				</div>
				<div class="split-companion-tabs">
					{#each [
						{ id: 'properties', icon: '⚙', label: $t('panels.properties') || 'Properties' },
						{ id: 'backlinks', icon: '🔗', label: $t('panels.backlinks') || 'Backlinks' },
						{ id: 'tags', icon: '🏷', label: $t('panels.tags') || 'Tags' },
						{ id: 'star', icon: '', label: $t('panels.skyView') || 'Sky View' },
						{ id: 'tasks', icon: '☑', label: $t('panels.tasks') || 'Tasks' },
					] as tab}
						<button class="sc-tab" class:active={splitCompanionTab === tab.id}
							onclick={() => splitCompanionTab = tab.id as any}>
							{#if tab.id === 'star'}<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>{:else}{tab.icon}{/if} {tab.label}
						</button>
					{/each}
				</div>
				<div class="split-compare-columns">
					{#each scPanels as panel (panel.notePath)}
						<div class="split-compare-col" dir={detectDir(panel.noteName)}>
							<div class="split-compare-col-header">
								<span class="sc-dot" style="background:{libraryColorMap[panel.libraryName] || '#7c3aed'}"></span>
								<span class="split-compare-col-name">{panel.noteName.replace(/\.md$/, '')}</span>
							</div>
							<div class="split-compare-col-body">
								{#if splitCompanionTab === 'properties'}
									<div class="sc-props-list">
										{#each panel.properties as prop}
											<div class="sc-prop">
												<span class="sc-prop-key">{prop.key}</span>
												<span class="sc-prop-val">
													{#if Array.isArray(prop.value)}
														{prop.value.join(', ')}
													{:else}
														{prop.value}
													{/if}
												</span>
											</div>
										{/each}
										{#if panel.properties.length === 0}
											<p class="sc-empty">{$t('secondScreen.noProperties') || 'No properties'}</p>
										{/if}
									</div>
								{:else if splitCompanionTab === 'backlinks'}
									{#if panel.backlinks.length > 0}
										<ul class="sc-link-list">
											{#each panel.backlinks as bl}
												<li>
													<button class="sc-link-item" oncontextmenu={(e) => showSSNoteMenu(bl.path, bl.name, e)} onclick={() => sendNoteToMain({ path: bl.path, name: bl.name, libraryName: bl.libraryName, libraryPath: '', libraryColor: libraryColorMap[bl.libraryName] || '#7c3aed' })}>
														<span class="sc-dot" style="background:{libraryColorMap[bl.libraryName] || '#7c3aed'}"></span>
														{bl.name}
													</button>
												</li>
											{/each}
										</ul>
									{:else}
										<p class="sc-empty">{$t('backlinksPanel.noBacklinks') || 'No backlinks'}</p>
									{/if}
									{#if panel.forwardLinks.length > 0}
										<h4 class="sc-section-title">{$t('secondScreen.forwardLinks') || 'Forward Links'} <span class="sc-count">{panel.forwardLinks.length}</span></h4>
										<ul class="sc-link-list">
											{#each panel.forwardLinks as fl}
												<li>
													<button class="sc-link-item" oncontextmenu={(e) => showSSNoteMenu(fl.path, fl.name, e)} onclick={() => sendNoteToMain({ path: fl.path, name: fl.name, libraryName: fl.libraryName, libraryPath: '', libraryColor: libraryColorMap[fl.libraryName] || '#7c3aed' })}>
														<span class="sc-dot" style="background:{libraryColorMap[fl.libraryName] || '#7c3aed'}"></span>
														{fl.name}
													</button>
												</li>
											{/each}
										</ul>
									{/if}
								{:else if splitCompanionTab === 'tags'}
									{#if panel.tags.length > 0}
										<div class="sc-tags">
											{#each panel.tags as tag}
												<span class="sc-tag">#{tag}</span>
											{/each}
										</div>
									{:else}
										<p class="sc-empty">{$t('panels.noTags') || 'No tags'}</p>
									{/if}
								{:else if splitCompanionTab === 'star'}
									<div class="sc-star-panel">
										{#if panel.localSkyNodes.length > 0}
											<LocalSkyView
												nodes={panel.localSkyNodes}
												links={panel.localSkyLinks}
												libraryColorMap={libraryColorMap}
												activeNodeId={panel.noteName.replace(/\.md$/, '').toLowerCase()}
												onNodeClick={(id) => {
													const note = allNotes.find(n => n.name.toLowerCase() === id);
													if (note) sendNoteToMain({ path: note.path, name: note.name, libraryName: note.libraryName, libraryPath: '', libraryColor: libraryColorMap[note.libraryName] || '#7c3aed' });
												}}
											/>
										{:else}
											<p class="sc-empty">{$t('panels.noConnections') || 'No connections'}</p>
										{/if}
									</div>
								{:else if splitCompanionTab === 'tasks'}
									{#if panel.tasks.length > 0}
										<TasksPanel
											tasks={panel.tasks}
											{libraryColorMap}
											onToggle={async (filePath, lineNumber) => {
												try {
													await toggleTaskReconciled(filePath, lineNumber);
													if (splitCompanionData) await loadSplitCompanionPanelData(splitCompanionData);
												} catch {}
											}}
										/>
									{:else}
										<p class="sc-empty">{$t('panels.noTasks') || 'No tasks'}</p>
									{/if}
								{/if}
							</div>
						</div>
					{/each}
					{#if scPanels.length === 0}
						<p class="sc-empty" style="padding: 24px;">{$t('panels.noNoteSelected') || 'No notes in split view'}</p>
					{/if}
				</div>
			</div>

		{:else if editorPanelsActive && editorPanelsData}
			<!-- Editor Panels Companion (migrated right sidebar) -->
			<div class="split-companion" dir={detectDir(editorPanelsData.noteName || editorPanelsData.content || '')}>
				<div class="split-companion-header">
					<span class="split-companion-label">{$t('secondScreen.editorPanels') || 'Panels'}</span>
					{#if editorPanelsData.noteName}
						<span class="split-companion-note">{editorPanelsData.noteName.replace(/\.md$/, '')}</span>
					{/if}
					{#if monitorCount > 1}
						<span class="monitor-badge">2nd Display</span>
					{/if}
				</div>
				<div class="split-companion-tabs">
					{#each [
						{ id: 'properties', icon: '⚙', label: $t('panels.properties') || 'Properties' },
						{ id: 'backlinks', icon: '🔗', label: $t('panels.backlinks') || 'Backlinks' },
						{ id: 'tags', icon: '🏷', label: $t('panels.tags') || 'Tags' },
						{ id: 'star', icon: '', label: $t('panels.skyView') || 'Sky View' },
						{ id: 'tasks', icon: '☑', label: $t('panels.tasks') || 'Tasks' },
					] as tab}
						<button class="sc-tab" class:active={editorPanelsTab === tab.id}
							onclick={() => editorPanelsTab = tab.id as any}>
							{#if tab.id === 'star'}<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>{:else}{tab.icon}{/if} {tab.label}
						</button>
					{/each}
				</div>
				<div class="split-companion-body">
					{#if editorPanelsData.notePath}
						{#if editorPanelsTab === 'properties'}
							<div class="sc-panel">
								<div class="sc-props-list">
									{#each epProperties as prop}
										<div class="sc-prop">
											<span class="sc-prop-key">{prop.key}</span>
											<span class="sc-prop-val">
												{#if Array.isArray(prop.value)}
													{prop.value.join(', ')}
												{:else}
													{prop.value}
												{/if}
											</span>
										</div>
									{/each}
									{#if epProperties.length === 0}
										<p class="sc-empty">{$t('secondScreen.noProperties') || 'No properties'}</p>
									{/if}
								</div>
							</div>
						{:else if editorPanelsTab === 'backlinks'}
							<div class="sc-panel">
								{#if epBacklinks.length > 0}
									<ul class="sc-link-list">
										{#each epBacklinks as bl}
											<li>
												<button class="sc-link-item" oncontextmenu={(e) => showSSNoteMenu(bl.path, bl.name, e)} onclick={() => sendNoteToMain({ path: bl.path, name: bl.name, libraryName: bl.libraryName, libraryPath: '', libraryColor: libraryColorMap[bl.libraryName] || '#7c3aed' })}>
													<span class="sc-dot" style="background:{libraryColorMap[bl.libraryName] || '#7c3aed'}"></span>
													{bl.name}
												</button>
											</li>
										{/each}
									</ul>
								{:else}
									<p class="sc-empty">{$t('backlinksPanel.noBacklinks') || 'No backlinks'}</p>
								{/if}
								{#if epForwardLinks.length > 0}
									<h4 class="sc-section-title">{$t('secondScreen.forwardLinks') || 'Forward Links'} <span class="sc-count">{epForwardLinks.length}</span></h4>
									<ul class="sc-link-list">
										{#each epForwardLinks as fl}
											<li>
												<button class="sc-link-item" oncontextmenu={(e) => showSSNoteMenu(fl.path, fl.name, e)} onclick={() => sendNoteToMain({ path: fl.path, name: fl.name, libraryName: fl.libraryName, libraryPath: '', libraryColor: libraryColorMap[fl.libraryName] || '#7c3aed' })}>
													<span class="sc-dot" style="background:{libraryColorMap[fl.libraryName] || '#7c3aed'}"></span>
													{fl.name}
												</button>
											</li>
										{/each}
									</ul>
								{/if}
							</div>
						{:else if editorPanelsTab === 'tags'}
							<div class="sc-panel">
								{#if epTags.length > 0}
									<div class="sc-tags">
										{#each epTags as tag}
											<span class="sc-tag">#{tag}</span>
										{/each}
									</div>
								{:else}
									<p class="sc-empty">{$t('panels.noTags') || 'No tags'}</p>
								{/if}
							</div>
						{:else if editorPanelsTab === 'star'}
							<div class="sc-panel sc-star-panel">
								{#if epLocalSkyNodes.length > 0}
									<LocalSkyView
										nodes={epLocalSkyNodes}
										links={epLocalSkyLinks}
										libraryColorMap={libraryColorMap}
										activeNodeId={editorPanelsData.noteName?.replace(/\.md$/, '').toLowerCase() || ''}
										onNodeClick={(id) => {
											const note = allNotes.find(n => n.name.toLowerCase() === id);
											if (note) sendNoteToMain({ path: note.path, name: note.name, libraryName: note.libraryName, libraryPath: '', libraryColor: libraryColorMap[note.libraryName] || '#7c3aed' });
										}}
									/>
								{:else}
									<p class="sc-empty">{$t('panels.noConnections') || 'No connections'}</p>
								{/if}
							</div>
						{:else if editorPanelsTab === 'tasks'}
							<div class="sc-panel">
								{#if epTasks.length > 0}
									<TasksPanel
										tasks={epTasks}
										{libraryColorMap}
										onToggle={async (filePath, lineNumber) => {
											try {
												await toggleTaskReconciled(filePath, lineNumber);
												if (editorPanelsData) await loadEditorPanelsData(editorPanelsData);
											} catch {}
										}}
									/>
								{:else}
									<p class="sc-empty">{$t('panels.noTasks') || 'No tasks in this note'}</p>
								{/if}
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
											<NoteEditor tab={peekTab} noteNames={allNotes} readOnly={ssReadOnly} />
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
								<LocalSkyView
									nodes={skyviewLocalNodes}
									links={skyviewLocalLinks}
									libraryColorMap={libraryColorMap}
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
					onNoteContext={(path, name, e) => showSSNoteMenu(path, name, e)}
				/>
			</div>

		{:else if mainSidebarMode === 'tree' && $activeTab}
			<!-- File Explorer companion — Universe Dashboard (only when main window has a note open) -->
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
						<NoteEditor tab={$activeTab} noteNames={allNotes} readOnly={ssReadOnly} />
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
		<span class="status-count">{$tn('plurals.notes', allNotes.length)}</span>
		<span class="status-linked">{$t('secondScreen.linked')}</span>
	</div>
</div>

<!-- MIG-096 §2 — the second screen's shared note menu. The mutating items
     (rename/move/delete) forward to the main window (Display-not-Domain). -->
{#if ssCtxMenu}
	<ContextMenu x={ssCtxMenu.x} y={ssCtxMenu.y} items={ssCtxMenu.items} onClose={() => ssCtxMenu = null} />
{/if}

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
		min-width: 0; border-inline-start: 1px solid var(--background-modifier-border);
		padding-inline-start: 20px; font-family: var(--font-interface-theme);
	}
	.tags-notes-title {
		display: flex; align-items: center; gap: 8px;
		margin: 0 0 10px 0; font-size: 13px; font-weight: 600; color: var(--text-normal);
	}
	.tags-notes-close {
		margin-inline-start: auto; background: none; border: none; cursor: pointer;
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

	/* Dashboard Companion — note editor mode */
	.dash-note-companion, .dash-tag-companion {
		display: flex; flex-direction: column; height: 100%; overflow: hidden;
	}
	.dash-note-header, .dash-tag-header {
		display: flex; align-items: center; gap: 10px;
		padding: 6px 16px; flex-shrink: 0;
		background: #e8e8ec;
	}
	.dash-back-btn {
		width: 28px; height: 28px; border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		background: transparent; color: var(--text-muted);
		cursor: pointer; display: flex; align-items: center; justify-content: center;
	}
	.dash-back-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.dash-note-name {
		font-size: 15px; font-weight: 600; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.dash-note-editor { flex: 1; overflow: hidden; display: flex; flex-direction: column; }

	/* Dashboard Companion — tag split view */
	.dash-tag-badge {
		padding: 2px 10px; border-radius: 12px;
		background: var(--interactive-accent); color: white; font-size: 13px; font-weight: 600;
	}
	.dash-tag-count { font-size: 12px; color: var(--text-faint); }
	.dash-tag-split {
		flex: 1; display: flex; overflow: hidden;
	}
	.dash-tag-list {
		width: 280px; min-width: 220px; max-width: 360px;
		border-right: 1px solid var(--background-modifier-border);
		overflow-y: auto; flex-shrink: 0;
	}
	.dash-tag-note {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 14px; width: 100%; border: none;
		background: transparent; color: var(--text-normal);
		cursor: pointer; text-align: start; font-family: inherit;
		transition: background 0.12s;
	}
	.dash-tag-note:hover { background: var(--background-modifier-hover); }
	.dash-tag-note.active { background: var(--interactive-accent); color: white; }
	.dash-tag-note.active .lib-dot { box-shadow: 0 0 0 2px rgba(255,255,255,0.4); }
	.dash-tag-note-name {
		flex: 1; font-size: 13px; overflow: hidden;
		text-overflow: ellipsis; white-space: nowrap;
	}
	.dash-tag-editor {
		flex: 1; overflow: hidden; display: flex; flex-direction: column;
	}
	.dash-tag-empty {
		flex: 1; display: flex; align-items: center; justify-content: center;
		color: var(--text-faint); font-size: 14px;
	}

	/* Map Companion */
	.map-companion { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
	.map-companion-note { display: flex; flex: 1; overflow: hidden; }
	.map-companion-editor { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
	.map-companion-context { width: 300px; min-width: 250px; border-left: 1px solid var(--background-modifier-border); overflow: hidden; }
	.map-companion-grid-header {
		padding: 12px 16px; flex-shrink: 0;
		border-bottom: 1px solid var(--background-modifier-border);
		display: flex; align-items: center; gap: 10px;
	}
	.map-companion-title { font-size: 16px; font-weight: 700; color: var(--text-normal); }
	.map-companion-stats { font-size: 12px; color: var(--text-muted); flex: 1; }
	.map-companion-color-select {
		font-size: 12px; padding: 3px 8px; border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-primary); color: var(--text-normal);
		cursor: pointer;
	}
	.map-companion-legend {
		display: flex; flex-wrap: wrap; gap: 8px; padding: 6px 16px;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.map-legend-item { display: flex; align-items: center; gap: 4px; font-size: 11px; color: var(--text-muted); }
	.map-legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.map-companion-grid {
		flex: 1; overflow-y: auto; padding: 12px;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		grid-auto-rows: 1fr;
		gap: 12px; align-content: stretch;
	}
	.map-companion-card {
		border: 1px solid var(--background-modifier-border);
		border-radius: 10px; overflow: hidden;
		background: var(--background-secondary);
		display: flex; flex-direction: column;
		min-height: 300px;
	}
	.map-companion-card-label {
		padding: 8px 14px; font-size: 14px; font-weight: 700;
		color: var(--text-normal);
		border-bottom: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.map-companion-card-chart { flex: 1; min-height: 0; }
	.map-companion-card-stats {
		padding: 6px 14px; font-size: 12px; color: var(--text-muted);
		border-top: 1px solid var(--background-modifier-border);
		flex-shrink: 0;
	}
	.map-companion-full { flex: 1; overflow: hidden; }

	/* Index Compare */
	.index-compare { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
	.index-compare-label { font-size: 13px; font-weight: 600; color: var(--text-normal); }
	.index-compare-body { flex: 1; display: flex; overflow: hidden; }
	.index-compare-columns {
		width: 320px; min-width: 240px; max-width: 400px;
		display: flex; flex-direction: column; overflow-y: auto;
		border-right: 1px solid var(--background-modifier-border);
	}
	.index-compare-col { border-bottom: 1px solid var(--background-modifier-border); }
	.index-compare-col.active { background: color-mix(in srgb, var(--interactive-accent) 5%, transparent); }
	.index-compare-col-header {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 12px; cursor: pointer;
		border-bottom: 1px solid var(--background-modifier-border);
	}
	.index-compare-col-header:hover { background: var(--background-modifier-hover); }
	.index-compare-col-list { max-height: 200px; overflow-y: auto; }
	.index-compare-editor { flex: 1; overflow: hidden; display: flex; flex-direction: column; }

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
	.split-companion-count { font-size: 12px; color: var(--text-muted); margin-inline-start: auto; }
	.split-compare-columns {
		display: flex; flex: 1; overflow-x: auto; overflow-y: hidden;
	}
	.split-compare-col {
		flex: 1; min-width: 220px; overflow-y: auto;
		border-inline-end: 1px solid var(--background-modifier-border);
		display: flex; flex-direction: column;
	}
	.split-compare-col:last-child { border-inline-end: none; }
	.split-compare-col-header {
		display: flex; align-items: center; gap: 6px;
		padding: 8px 12px; font-size: 13px; font-weight: 600;
		color: var(--text-normal); background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		position: sticky; top: 0; z-index: 1; flex-shrink: 0;
	}
	.split-compare-col-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.split-compare-col-body { flex: 1; overflow-y: auto; padding: 10px 12px; }
	.sc-panel { min-height: 100px; }
	/* MIG-072 §5 — the 2nd-screen Sky View companion fills the whole panel body (was a fixed
	   300px cap that left most of the centre zone empty). Parents are definite-height flex items
	   (.split-compare-col-body / .split-companion-body), so height:100% resolves; the min-height
	   floor keeps the graph usable if the window is very short. */
	.sc-star-panel { height: 100%; min-height: 240px; }
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
	.sc-link-list { list-style: none; margin: 0; padding: 0; }
	.sc-link-list li { margin: 1px 0; }
	.sc-link-item {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 5px 8px; border: none; border-radius: 4px;
		background: none; color: var(--interactive-accent); font-size: 13px;
		font-family: inherit; cursor: pointer; text-align: start;
	}
	.sc-link-item:hover { background: var(--background-modifier-hover); text-decoration: underline; }
	.sc-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
	.sc-section-title {
		font-size: 11px; font-weight: 600; color: var(--text-muted);
		text-transform: uppercase; letter-spacing: 0.5px; margin: 12px 0 4px; padding: 0;
		display: flex; align-items: center; gap: 4px;
	}
	.sc-count { font-weight: 400; opacity: 0.6; font-size: 10px; }

	.monitor-badge {
		font-size: 12px; color: var(--text-muted); margin-inline-start: auto;
		background: var(--background-modifier-border); border-radius: 10px; padding: 3px 10px;
		font-weight: 500;
	}
</style>

<script lang="ts">
	import '$lib/theme.css';
	import { onMount, onDestroy } from 'svelte';
	import { dir, t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import {
		vaults, vaultStats, searchResults, totalStars, vaultCount,
		activeTab, openTabs, activeTabId,
		splitActive, splitDirection, focusedTabId, focusedTab,
		loadVaults, loadAllStats, addVault, searchAllStars,
		openNoteTab, closeTab, switchTab, closeNote, createEmptyTab,
		toggleSplit, toggleSplitDirection, setFocusedTab,
		parseFrontmatter, extractHeadings,
		createNote, createFolder, renameItem, deleteItem,
		startWatchingVault, wasRecentlyWritten,
		loadVaultAppearance, vaultAppearances,
		toggleEditMode, editingTabIds,
		navigateBack, navigateForward,
		scanVaultLinks, scanVaultTags, getBacklinks, getOutgoingLinks, scanUnlinkedMentions,
		scanVaultIndex,
		buildGraphData, readNotePreview,
		getDailyNotePath, updateLinksOnRename,
		loadBookmarks, addBookmark, removeBookmark, isBookmarked, bookmarks,
		loadSettings, updateSettings, appSettings,
		loadWorkspaces, workspaces,
		resolveWikilinkCrossVault,
		buildDefaultFrontmatter, searchByProperty,
		type FrontmatterProperty, type HeadingItem, type NoteLink, type GraphNode, type GraphLink,
		type IndexEntry
	} from '$lib/vaults/store';
	import type { VaultStats, FileEntry } from '$lib/vaults/store';
	import { get } from 'svelte/store';
	import { detectDir } from '$lib/utils';
	import { createBase, saveBaseFile, listWorkspaceBases, createWorkspaceBase, saveWorkspaceBase, deleteWorkspaceBase } from '$lib/bases/store';
	import type { WorkspaceBaseEntry } from '$lib/bases/store';
	import type { BaseDefinition } from '$lib/bases/types';
	import FileTree from '$lib/components/FileTree.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import QuickSwitcher from '$lib/components/QuickSwitcher.svelte';
	import GraphView from '$lib/components/GraphView.svelte';
	import BacklinksPanel from '$lib/components/BacklinksPanel.svelte';
	import TagsPanel from '$lib/components/TagsPanel.svelte';
	import LinkDashboard from '$lib/components/LinkDashboard.svelte';
	import PropertyEditor from '$lib/components/PropertyEditor.svelte';
	import PagePreview from '$lib/components/PagePreview.svelte';
	import WorkspaceManager from '$lib/components/WorkspaceManager.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import LockScreen from '$lib/components/LockScreen.svelte';
	import VaultSwitcher from '$lib/components/VaultSwitcher.svelte';
	import VaultManager from '$lib/components/VaultManager.svelte';
	import VaultPicker from '$lib/components/VaultPicker.svelte';
	import NewBaseDialog from '$lib/components/NewBaseDialog.svelte';
	import OutgoingLinksPanel from '$lib/components/OutgoingLinksPanel.svelte';
	import IndexPanel from '$lib/components/IndexPanel.svelte';
	import UniverseSetup from '$lib/components/UniverseSetup.svelte';
	import UniverseManager from '$lib/components/UniverseManager.svelte';
	import {
		listUniverses, createUniverse, setActiveUniverse,
		checkMigrationNeeded, migrateLegacyData,
		type UniverseEntry
	} from '$lib/universe/store';
	import { loadPropertyTypes } from '$lib/vaults/propertyTypeRegistry';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	// Sidebar state
	let sidebarOpen = $state(true);
	let searchMode = $state(false);
	let indexMode = $state(false);
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;
	let sortOrder = $state<'name-asc' | 'name-desc' | 'modified-desc' | 'modified-asc'>('name-asc');
	let vaultPickerAction = $state<'note' | 'folder' | 'base'>('note');
	let allExpanded = $state(true);

	// Universe state
	let showUniverseSetup = $state(false);
	let showUniverseManager = $state(false);
	let activeUniverseName = $state('');
	let appReady = $state(false);

	// Update window title when active universe changes
	$effect(() => {
		const name = activeUniverseName;
		if (name) {
			getCurrentWindow().setTitle(`Constellation - ${name}`).catch(() => {});
		}
	});

	// Right sidebar
	let rightSidebarOpen = $state(false);
	let rightSidebarTab = $state<'properties' | 'backlinks' | 'tags' | 'graph' | 'links'>('properties');

	// Sidebar resizing
	let leftSidebarWidth = $state(240);
	let rightSidebarWidth = $state(260);
	let resizing = $state<'left' | 'right' | null>(null);

	// Command palette & quick switcher
	let showCommandPalette = $state(false);
	let showQuickSwitcher = $state(false);
	let showGraphView = $state(false);

	// Workspace manager
	let showWorkspaces = $state(false);

	// Settings modal
	let showSettings = $state(false);

	// Vault management
	let showVaultSwitcher = $state(false);
	let showVaultManager = $state(false);
	let showVaultPicker = $state(false);
	let showNewBaseDialog = $state(false);

	// Lock screen
	let isLocked = $state(false);
	let idleTimer: ReturnType<typeof setTimeout> | null = null;

	// Page preview (hover)
	let pagePreview = $state<{ content: string; x: number; y: number; visible: boolean }>({ content: '', x: 0, y: 0, visible: false });
	let previewTimeout: ReturnType<typeof setTimeout>;

	// Cache refresh debounce (for file watcher)
	let cacheRefreshDebounce: ReturnType<typeof setTimeout>;

	// Vault data caches
	let allVaultLinks = $state<NoteLink[]>([]);
	let allVaultTags = $state<Record<string, number>>({});
	let allNotes = $state<{ name: string; path: string; vaultName: string }[]>([]);
	let allIndexEntries = $state<IndexEntry[]>([]);
	let graphNodes = $state<GraphNode[]>([]);
	let graphLinks = $state<GraphLink[]>([]);

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

	// Vault trees
	let vaultTrees = $state<Record<string, FileEntry[]>>({});
	let expandedVaults = $state<Set<string>>(new Set());

	// Workspace bases
	let workspaceBases = $state<WorkspaceBaseEntry[]>([]);
	let workspaceBasesExpanded = $state(true);

	let error = $state('');
	let adding = $state(false);

	const isHome = $derived(page.url.pathname === '/');

	// Vault color palette
	const VAULT_COLORS = ['#7c3aed', '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#06b6d4', '#8b5cf6'];
	const vaultColorMap = $derived.by(() => {
		const map: Record<string, string> = {};
		$vaults.forEach((v, i) => { map[v.name] = VAULT_COLORS[i % VAULT_COLORS.length]; });
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
		return getBacklinks(allVaultLinks, sidebarTab.name);
	});

	// Unlinked mentions for current note
	let currentUnlinkedMentions: { name: string; path: string; context: string; vaultName: string }[] = $state([]);
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
		return getOutgoingLinks(allVaultLinks, sidebarTab.path).map(l => ({
			target: l.target,
			context: l.context,
		}));
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

	// All note names across all vaults (for quick switcher)
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

	// Re-init idle timer when lock settings change
	$effect(() => {
		const _lockOn = $appSettings.security?.lockOnIdle;
		const _timeout = $appSettings.security?.lockIdleTimeout;
		resetIdleTimer();
	});

	// ─── Commands for command palette ───
	function getCommands() {
		return [
			{ id: 'new-note', name: $t('commands.newNote'), shortcut: 'Ctrl+N', icon: '📄', action: handleNewNote, category: 'File' },
			{ id: 'new-base', name: $t('commands.newBase'), shortcut: 'Ctrl+Shift+B', icon: '▦', action: handleNewBase, category: 'File' },
			{ id: 'quick-switch', name: $t('commands.quickSwitcher'), shortcut: 'Ctrl+O', icon: '🔍', action: () => { showCommandPalette = false; showQuickSwitcher = true; }, category: 'Navigation' },
			{ id: 'search', name: $t('commands.searchVault'), shortcut: 'Ctrl+Shift+F', icon: '🔎', action: () => { sidebarOpen = true; searchMode = true; }, category: 'Navigation' },
			{ id: 'daily-note', name: $t('commands.dailyNote'), icon: '📅', action: handleOpenDailyNote, category: 'Daily Notes' },
			{ id: 'toggle-edit', name: $t('commands.toggleEdit'), shortcut: 'Ctrl+E', icon: '✏️', action: () => { const tab = get(focusedTab); if (tab) toggleEditMode(tab.id); }, category: 'Editor' },
			{ id: 'graph-view', name: $t('commands.graphView'), icon: '🕸️', action: () => showGraphView = !showGraphView, category: 'View' },
			{ id: 'toggle-bold', name: $t('commands.toggleBold'), shortcut: 'Ctrl+B', icon: '𝐁', action: () => {}, category: 'Editor' },
			{ id: 'toggle-italic', name: $t('commands.toggleItalic'), shortcut: 'Ctrl+I', icon: '𝐼', action: () => {}, category: 'Editor' },
			{ id: 'split-view', name: $t('commands.splitView'), shortcut: '', icon: '⊞', action: cycleSplit, category: 'View' },
			{ id: 'close-note', name: $t('commands.closeNote'), shortcut: 'Ctrl+W', icon: '✕', action: closeNote, category: 'File' },
			{ id: 'toggle-left', name: $t('commands.toggleLeftSidebar'), shortcut: 'Ctrl+\\', icon: '◧', action: () => sidebarOpen = !sidebarOpen, category: 'View' },
			{ id: 'toggle-right', name: $t('commands.toggleRightSidebar'), icon: '◨', action: () => rightSidebarOpen = !rightSidebarOpen, category: 'View' },
			{ id: 'add-vault', name: $t('commands.addVault'), icon: '📁', action: handleAddVault, category: 'Vault' },
			{ id: 'toggle-bookmark', name: $t('commands.toggleBookmark'), icon: '⭐', action: handleToggleBookmark, category: 'Bookmarks' },
			{ id: 'random-note', name: $t('commands.randomNote'), icon: '🎲', action: handleRandomNote, category: 'Navigation' },
			{ id: 'toggle-theme', name: $t('commands.toggleTheme'), icon: '🌗', action: handleToggleTheme, category: 'Appearance' },
			{ id: 'nav-back', name: $t('commands.navBack'), shortcut: 'Alt+←', icon: '←', action: navigateBack, category: 'Navigation' },
			{ id: 'nav-forward', name: $t('commands.navForward'), shortcut: 'Alt+→', icon: '→', action: navigateForward, category: 'Navigation' },
			{ id: 'workspaces', name: $t('commands.workspaces'), icon: '🗂️', action: () => { showCommandPalette = false; showWorkspaces = true; }, category: 'View' },
			{ id: 'index', name: $t('commands.index'), icon: '📖', action: () => { showCommandPalette = false; sidebarOpen = true; searchMode = false; indexMode = true; }, category: 'Navigation' },
			{ id: 'settings', name: $t('commands.settings'), shortcut: 'Ctrl+,', icon: '⚙️', action: () => { showCommandPalette = false; showSettings = true; }, category: 'App' },
			{ id: 'add-property', name: $t('commands.addProperty'), shortcut: 'Ctrl+;', icon: '✎', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:add-property')); }, category: 'Editor' },
			{ id: 'insert-link', name: $t('commands.insertLink'), shortcut: 'Ctrl+K', icon: '🔗', action: () => {}, category: 'Editor' },
			{ id: 'duplicate-line', name: $t('commands.duplicateLine'), shortcut: 'Ctrl+Shift+D', icon: '📋', action: () => {}, category: 'Editor' },
			{ id: 'toggle-comment', name: $t('commands.toggleComment'), shortcut: 'Ctrl+/', icon: '💬', action: () => {}, category: 'Editor' },
			{ id: 'select-next', name: $t('commands.selectNextOccurrence'), shortcut: 'Ctrl+D', icon: '🔤', action: () => {}, category: 'Editor' },
			{ id: 'fold-all', name: $t('commands.foldAll'), icon: '🔽', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:fold-all')); }, category: 'Editor' },
			{ id: 'unfold-all', name: $t('commands.unfoldAll'), icon: '🔼', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:unfold-all')); }, category: 'Editor' },
			{ id: 'toggle-live-preview', name: $t('commands.toggleLivePreview'), icon: '📖', action: () => { showCommandPalette = false; document.dispatchEvent(new CustomEvent('constellation:toggle-live-preview')); }, category: 'Editor' },
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

		// Load vaults — this is what the sidebar needs
		try { await loadVaults(); } catch { /* ignore */ }
		try { await loadAllStats(); } catch { /* ignore */ }

		// App is usable now — show UI immediately
		appReady = true;

		// Start file watchers and build caches in the background
		for (const vault of $vaults) {
			try { await startWatchingVault(vault.id, vault.path); } catch { /* ignore */ }
			await loadVaultAppearance(vault.path, vault.id);
		}
		await refreshVaultCaches();
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

		// Unwatch all vaults
		for (const vault of $vaults) {
			try { await invoke('unwatch_vault', { vaultId: vault.id }); } catch { /* ignore */ }
		}

		// Clear in-memory state
		$openTabs = [];
		$activeTabId = null;
		$focusedTabId = null;
		workspaceBases = [];

		// Clear vault stores so sidebar resets
		vaults.set([]);
		vaultStats.set([]);
		allVaultLinks = [];
		allVaultTags = {};
		allNotes = [];
		allIndexEntries = [];
		vaultTrees = {};
		expandedVaults = new Set();

		// Reset cache guard so refreshVaultCaches can run for the new universe
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
	onMount(async () => {
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
		let watcherDebounce: ReturnType<typeof setTimeout>;
		const unlistenWatcher = await listen<{ vaultId: string; paths: string[] }>('vault-changed', (event) => {
			const { vaultId, paths } = event.payload;
			pendingTreeRefresh.add(vaultId);
			for (const p of paths) {
				if (!wasRecentlyWritten(p)) pendingTabReloads.add(p);
			}
			// Batch rapid file changes (300ms window)
			clearTimeout(watcherDebounce);
			watcherDebounce = setTimeout(async () => {
				const vaultIds = [...pendingTreeRefresh];
				const tabPaths = [...pendingTabReloads];
				pendingTreeRefresh.clear();
				pendingTabReloads.clear();

				// Refresh trees for changed vaults
				for (const vid of vaultIds) {
					await refreshVaultTree(vid);
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
				cacheRefreshDebounce = setTimeout(() => refreshVaultCaches(), 5000);
			}, 300);
		});

		if ($vaultStats.length === 1) {
			await toggleVault($vaultStats[0]);
		}

		// Global keyboard shortcuts
		document.addEventListener('keydown', handleGlobalKeydown);

		// Cleanup on destroy
		cleanupFns.push(
			() => document.removeEventListener('keydown', handleGlobalKeydown),
			unlistenWatcher,
		);
	});

	const cleanupFns: (() => void)[] = [];
	onDestroy(() => {
		clearTimeout(searchTimeout);
		clearTimeout(previewTimeout);
		clearTimeout(cacheRefreshDebounce);
		clearTimeout(unlinkedDebounce);
		resizeCleanup?.();
		for (const fn of cleanupFns) fn();
	});

	let cacheRefreshing = false;
	async function refreshVaultCaches() {
		// Prevent concurrent scans — skip if one is already in progress
		if (cacheRefreshing) return;
		cacheRefreshing = true;
		try {
			const links: NoteLink[] = [];
			const tags: Record<string, number> = {};
			const notes: { name: string; path: string; vaultName: string }[] = [];
			const indexRaw: IndexEntry[] = [];

			// Process vaults sequentially (2 at a time) to avoid IPC flood
			const vaultList = $vaults;
			for (let i = 0; i < vaultList.length; i += 2) {
				const batch = vaultList.slice(i, i + 2);
				const batchResults = await Promise.all(batch.map(async (vault) => {
					const [vaultLinks, vaultTags, vaultNotes, vaultIndex] = await Promise.all([
						scanVaultLinks(vault.path, vault.name).catch(() => [] as NoteLink[]),
						scanVaultTags(vault.path).catch(() => ({} as Record<string, number>)),
						invoke('collect_vault_notes', { vaultPath: vault.path }).catch(() => []) as Promise<any[]>,
						scanVaultIndex(vault.path).catch(() => [] as IndexEntry[]),
					]);
					return { vault, vaultLinks, vaultTags, vaultNotes, vaultIndex };
				}));

				for (const { vault, vaultLinks, vaultTags, vaultNotes, vaultIndex } of batchResults) {
					links.push(...vaultLinks);
					for (const [tag, count] of Object.entries(vaultTags)) {
						tags[tag] = (tags[tag] || 0) + count;
					}
					notes.push(...vaultNotes.map((n: any) => ({ name: n.name, path: n.path, vaultName: vault.name })));
					indexRaw.push(...vaultIndex);
				}
			}

			allVaultLinks = links;
			allVaultTags = tags;
			allNotes = notes;
			allIndexEntries = mergeIndexEntries(indexRaw);

			// Build graph data from all vaults combined
			if (vaultList.length > 0) {
				const { nodes, links: gLinks } = buildGraphData(links, notes);
				graphNodes = nodes;
				graphLinks = gLinks;
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
				// Sum counts across vaults
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
		// Command palette
		if ((e.ctrlKey || e.metaKey) && e.key === 'p') {
			e.preventDefault();
			showCommandPalette = !showCommandPalette;
			showQuickSwitcher = false;
			return;
		}
		// Quick switcher
		if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
			e.preventDefault();
			showQuickSwitcher = !showQuickSwitcher;
			showCommandPalette = false;
			return;
		}
		// New note
		if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
			e.preventDefault();
			handleNewNote();
			return;
		}
		// New base
		if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'B') {
			e.preventDefault();
			handleNewBase();
			return;
		}
		// Toggle edit mode
		if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
			e.preventDefault();
			const tab = get(focusedTab);
			if (tab) toggleEditMode(tab.id);
			return;
		}
		// Close tab
		if ((e.ctrlKey || e.metaKey) && e.key === 'w') {
			e.preventDefault();
			closeNote();
			return;
		}
		// Search
		if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
			e.preventDefault();
			sidebarOpen = true;
			searchMode = true;
			return;
		}
		// Toggle left sidebar
		if ((e.ctrlKey || e.metaKey) && e.key === '\\') {
			e.preventDefault();
			sidebarOpen = !sidebarOpen;
			return;
		}
		// Settings
		if ((e.ctrlKey || e.metaKey) && e.key === ',') {
			e.preventDefault();
			showSettings = !showSettings;
			return;
		}
		// Add property (Ctrl+;)
		if ((e.ctrlKey || e.metaKey) && e.key === ';') {
			e.preventDefault();
			document.dispatchEvent(new CustomEvent('constellation:add-property'));
			return;
		}
		// Navigate back/forward
		if (e.altKey && e.key === 'ArrowLeft') {
			e.preventDefault();
			navigateBack();
			return;
		}
		if (e.altKey && e.key === 'ArrowRight') {
			e.preventDefault();
			navigateForward();
			return;
		}
		// Escape closes overlays
		if (e.key === 'Escape') {
			if (showCommandPalette) { showCommandPalette = false; return; }
			if (showQuickSwitcher) { showQuickSwitcher = false; return; }
			if (showGraphView) { showGraphView = false; return; }
			if (showWorkspaces) { showWorkspaces = false; return; }
			if (showSettings) { showSettings = false; return; }
		}
	}

	// ─── Actions ───
	async function handleNewNote() {
		if ($vaults.length === 0) return;
		if ($vaults.length === 1) {
			await createNoteInVault($vaults[0]);
		} else {
			vaultPickerAction = 'note';
			showVaultPicker = true;
		}
	}

	async function createNoteInVault(vault: { id: string; name: string; path: string }) {
		try {
			const baseName = $t('actions.untitled');
			let name = baseName;
			let newPath: string | null = null;

			// Build default frontmatter with auto-dates + user defaults
			const defaultFM = buildDefaultFrontmatter($appSettings);

			// Try loading template content if configured
			let templateBody = '';
			const templateFolder = $appSettings.templateFolder;
			if (templateFolder && $appSettings.enabledPlugins?.templates) {
				try {
					const templatePath = `${vault.path}/${templateFolder}/default.md`;
					const tpl: string = await invoke('read_note', { filePath: templatePath });
					if (tpl) {
						// Extract body from template (skip its own frontmatter)
						const tplParsed = parseFrontmatter(tpl);
						templateBody = tplParsed.body;
					}
				} catch { /* no default template — OK */ }
			}

			for (let i = 0; i < 100; i++) {
				try {
					newPath = await createNote(vault.path, name, defaultFM);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			if (!newPath) return;

			// If we have a template body, append it after the frontmatter
			if (templateBody.trim()) {
				try {
					const { invoke: inv } = await import('@tauri-apps/api/core');
					const currentContent: string = await inv('read_note', { filePath: newPath });
					const parsed = parseFrontmatter(currentContent);
					const fullContent = `---\n${defaultFM}\n---\n${templateBody}`;
					await inv('write_note', { filePath: newPath, content: fullContent });
				} catch { /* template write failed — note still created */ }
			}

			await refreshVaultTree(vault.id);
			const vaultColor = vaultColorMap[vault.name] ?? '#7c3aed';
			await openNoteTab(newPath, vault.name, vaultColor);
			// Auto-enter edit mode
			const tab = get(focusedTab);
			if (tab) toggleEditMode(tab.id);
		} catch (e) {
			console.error('Failed to create note:', e);
		}
	}

	async function handleNewBase() {
		showNewBaseDialog = true;
	}

	async function createWorkspaceBaseWithVaults(
		baseName: string,
		selectedVaults: string[],
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

			// Overwrite with the user's selected vaults + name
			const definition: BaseDefinition = {
				version: 1,
				name,
				source: {
					type: 'all',
					includeSubfolders: true,
					selectedVaults: selectedVaults.length > 0 ? selectedVaults : undefined,
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

			// Open in a tab — workspace bases use "Constellation" as vault name
			await openNoteTab(newPath, 'Constellation', '#7c3aed');
		} catch (e) {
			console.error('Failed to create workspace base:', e);
		}
	}

	async function handleNewFolder() {
		if ($vaults.length === 0) return;
		if ($vaults.length === 1) {
			await createFolderInVault($vaults[0]);
		} else {
			vaultPickerAction = 'folder';
			showVaultPicker = true;
		}
	}

	async function createFolderInVault(vault: { id: string; name: string; path: string }) {
		try {
			const baseName = $t('actions.newFolder');
			let name = baseName;
			for (let i = 0; i < 100; i++) {
				try {
					await createFolder(vault.path, name);
					break;
				} catch {
					name = `${baseName} ${i + 1}`;
				}
			}
			await refreshVaultTree(vault.id);
			// Expand the vault if not already
			if (!expandedVaults.has(vault.id)) {
				expandedVaults.add(vault.id);
				expandedVaults = new Set(expandedVaults);
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
		if (expandedVaults.size > 0) {
			expandedVaults = new Set();
			allExpanded = false;
		} else {
			for (const vault of $vaultStats) {
				expandedVaults.add(vault.vault_id);
			}
			expandedVaults = new Set(expandedVaults);
			allExpanded = true;
		}
	}

	async function handleOpenDailyNote() {
		const firstVault = $vaults[0];
		if (!firstVault) return;
		try {
			const path = await getDailyNotePath(firstVault.path, $appSettings.dailyNoteFormat, $appSettings.dailyNoteFolder);
			const vaultColor = vaultColorMap[firstVault.name] ?? '#7c3aed';
			await openNoteTab(path, firstVault.name, vaultColor);
		} catch (e) {
			console.error('Failed to open daily note:', e);
		}
	}

	function handleToggleBookmark() {
		const tab = get(focusedTab);
		if (!tab) return;
		if (isBookmarked(tab.path)) {
			const bm = get(bookmarks).find(b => b.path === tab.path);
			if (bm) removeBookmark(bm.id);
		} else {
			addBookmark({ type: 'note', path: tab.path, name: tab.name, vaultName: tab.vaultName });
		}
	}

	function handleRandomNote() {
		if (allNotes.length === 0) return;
		const randomNote = allNotes[Math.floor(Math.random() * allNotes.length)];
		const vaultColor = vaultColorMap[randomNote.vaultName] ?? '#7c3aed';
		openNoteTab(randomNote.path, randomNote.vaultName, vaultColor);
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

	async function handleQuickSwitchSelect(path: string, vaultName: string) {
		const vaultColor = vaultColorMap[vaultName] ?? '#7c3aed';
		await openNoteTab(path, vaultName, vaultColor);
	}

	function handleGraphNodeClick(path: string, vaultName: string) {
		const vaultColor = vaultColorMap[vaultName] ?? '#7c3aed';
		openNoteTab(path, vaultName, vaultColor);
		showGraphView = false; // Switch to note view
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
				const vaultList = $vaults.map(v => [v.id, v.name, v.path] as [string, string, string]);
				const resolved = await invoke<{ path: string; vault_name: string; vault_path: string } | null>('resolve_wikilink_cross_vault', { vaults: vaultList, currentVaultPath: sidebarTab!.vaultPath, target: linkTarget });
				if (gen !== previewGeneration) return; // stale — user moved to a different link
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

	// ─── Vault tree operations ───
	async function toggleVault(vault: VaultStats) {
		const id = vault.vault_id;
		if (expandedVaults.has(id)) {
			expandedVaults.delete(id);
			expandedVaults = new Set(expandedVaults);
		} else {
			if (!vaultTrees[id]) {
				const tree: FileEntry[] = await invoke('read_vault_tree', { path: vault.path, maxDepth: 4 });
				vaultTrees[id] = tree;
				vaultTrees = { ...vaultTrees };
			}
			expandedVaults.add(id);
			expandedVaults = new Set(expandedVaults);
		}
	}

	async function handleAddVault() {
		adding = true;
		error = '';
		try { await addVault(); await loadAllStats(); await refreshVaultCaches(); }
		catch (e) { error = String(e); }
		adding = false;
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
	let contextMenu = $state<{ x: number; y: number; entry: FileEntry; vaultId: string } | null>(null);
	let confirmDelete = $state<{ path: string; name: string } | null>(null);
	let renamingPath = $state('');

	function handleContextMenu(entry: FileEntry, x: number, y: number, vaultId: string) {
		contextMenu = { x, y, entry, vaultId };
	}

	function getContextMenuItems(entry: FileEntry, vaultId: string) {
		const items: { label: string; icon?: string; action: () => void; danger?: boolean }[] = [];

		// Workspace bases have a simplified context menu
		if (vaultId === '__workspace__') {
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
				action: () => handleCreateNote(entry.path, vaultId)
			});
			items.push({
				label: $t('actions.newFolder'),
				icon: '📁',
				action: () => handleCreateFolder(entry.path, vaultId)
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

	async function handleCreateNote(folderPath: string, vaultId: string) {
		try {
			const name = $t('actions.untitled');
			const newPath = await createNote(folderPath, name);
			await refreshVaultTree(vaultId);
			const vault = $vaults.find(v => v.id === vaultId);
			if (vault) {
				const vaultColor = vaultColorMap[vault.name] ?? '#7c3aed';
				await openNoteTab(newPath, vault.name, vaultColor);
			}
		} catch (e) {
			console.error('Failed to create note:', e);
		}
	}

	async function handleCreateBase(folderPath: string, vaultId: string) {
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

			await refreshVaultTree(vaultId);
			const vault = $vaults.find(v => v.id === vaultId);
			if (vault) {
				const vaultColor = vaultColorMap[vault.name] ?? '#7c3aed';
				await openNoteTab(newPath, vault.name, vaultColor);
			}
		} catch (e) {
			console.error('Failed to create base:', e);
		}
	}

	async function handleCreateFolder(parentPath: string, vaultId: string) {
		try {
			const name = $t('actions.newFolder');
			await createFolder(parentPath, name);
			await refreshVaultTree(vaultId);
		} catch (e) {
			console.error('Failed to create folder:', e);
		}
	}

	async function handleDeleteConfirm() {
		if (!confirmDelete) return;
		try {
			const vault = $vaultStats.find(v => confirmDelete!.path.startsWith(v.path));
			await deleteItem(confirmDelete.path, true);
			if (vault) await refreshVaultTree(vault.vault_id);
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
			const vault = $vaultStats.find(v => oldPath.startsWith(v.path));
			if (vault) {
				await refreshVaultTree(vault.vault_id);
				// Auto-update links
				if ($appSettings.autoUpdateLinks && !isDir) {
					await updateLinksOnRename(vault.path, oldName, newName);
				}
			}
		} catch (e) {
			console.error('Failed to rename:', e);
		}
	}

	async function refreshVaultTree(vaultId: string) {
		const vault = $vaultStats.find(v => v.vault_id === vaultId);
		if (vault) {
			const tree: FileEntry[] = await invoke('read_vault_tree', { path: vault.path, maxDepth: 4 });
			vaultTrees[vaultId] = tree;
			vaultTrees = { ...vaultTrees };
		}
	}

	async function handleNoteClick(filePath: string, _noteName: string, highlightTerm?: string, e?: MouseEvent) {
		const vault = $vaults.find(v => filePath.startsWith(v.path));
		const vaultColor = vault ? vaultColorMap[vault.name] : '#7c3aed';
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(filePath, vault?.name ?? '', vaultColor, highlightTerm, newTab);
		if (!isHome) window.location.href = '/';
	}

	async function handleSearchResultClick(path: string, vaultName: string, e?: MouseEvent) {
		const vaultColor = vaultColorMap[vaultName] ?? '#7c3aed';
		const newTab = e ? (e.ctrlKey || e.metaKey || e.button === 1) : false;
		await openNoteTab(path, vaultName, vaultColor, undefined, newTab);
		clearSearch();
		if (!isHome) window.location.href = '/';
	}

	// Get all tags as flat array for editor autocomplete
	const allTagsList = $derived(Object.keys(allVaultTags));
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
	<!-- ═══ RIBBON ═══ -->
	<div class="ribbon">
		<div class="ribbon-top">
			<button class="r-btn" onclick={() => { sidebarOpen = true; searchMode = false; indexMode = false; }} title={$t('ribbon.fileExplorer')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
			</button>
			<button class="r-btn" onclick={() => { sidebarOpen = true; searchMode = true; indexMode = false; }} title={$t('ribbon.search')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<button class="r-btn" onclick={() => showGraphView = !showGraphView} title={$t('ribbon.graphView')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>
			</button>
			<button class="r-btn" onclick={handleOpenDailyNote} title={$t('ribbon.dailyNote')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
			</button>
			<a href="/skills" class="r-btn" class:active={page.url.pathname === '/skills'} title={$t('ribbon.aiSkills')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01z"/></svg>
			</a>
			<button class="r-btn" class:active={indexMode} onclick={() => { sidebarOpen = true; searchMode = false; indexMode = !indexMode; }} title={$t('ribbon.index')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/><path d="M8 7h6"/><path d="M8 11h8"/></svg>
			</button>
		</div>
		<div class="ribbon-bottom">
			<button class="r-btn" onclick={() => showUniverseManager = true} title={$t('universe.title')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/></svg>
			</button>
			<button class="r-btn" onclick={handleToggleTheme} title={$t('ribbon.toggleTheme')}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
			</button>
<button class="r-btn" class:active={showSettings} onclick={() => showSettings = !showSettings} title={$t('ribbon.settings')}>
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
						<button class="tb-btn" onclick={toggleCollapseAll} title={expandedVaults.size > 0 ? $t('sidebar.collapseAll') : $t('sidebar.expandAll')}>
							{#if expandedVaults.size > 0}
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m7 20 5-5 5 5"/><path d="m7 4 5 5 5-5"/></svg>
							{:else}
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m7 15 5 5 5-5"/><path d="m7 9 5-5 5 5"/></svg>
							{/if}
						</button>
					</div>
				{/if}
			</div>

			<div class="sidebar-content">
				{#if indexMode}
					<IndexPanel
						entries={allIndexEntries}
						onNoteClick={handleNoteClick}
					/>
				{:else if searchMode && searchQuery}
					{#if $searchResults.length > 0}
						<div class="section-label">{$searchResults.length} {$t('sidebar.results')}</div>
						{#each $searchResults as star}
							<button class="s-result" class:active={$activeTab?.path === star.path} onclick={(e) => handleSearchResultClick(star.path, star.vault_name, e)}>
								<div class="s-name">{star.name}</div>
								<div class="s-meta">
									<span class="s-vault">{star.vault_name}</span>
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
								<div class="s-meta"><span class="s-vault">{bm.vaultName}</span></div>
							</button>
						{/each}
					{/if}

					<!-- Workspace Bases section -->
					{#if workspaceBases.length > 0}
						<div class="vault-section">
							<button class="vault-header" onclick={() => workspaceBasesExpanded = !workspaceBasesExpanded}>
								<svg class="v-chev" class:expanded={workspaceBasesExpanded} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-inline-end: 4px; opacity: 0.5;"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
								<span class="vault-name">{$t('sidebar.bases')}</span>
							</button>
							{#if workspaceBasesExpanded}
								<div class="vault-tree">
									{#each workspaceBases as base}
										<button
											class="ws-base-item"
											class:active={$activeTab?.path === base.path}
											onclick={() => openNoteTab(base.path, 'Constellation', '#7c3aed')}
											oncontextmenu={(e: MouseEvent) => {
												e.preventDefault();
												contextMenu = {
													x: e.clientX,
													y: e.clientY,
													entry: { name: base.name + '.base', path: base.path, is_dir: false, children: null, extension: 'base', modified: base.modified },
													vaultId: '__workspace__',
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

					{#each $vaultStats as vault}
						<div class="vault-section">
							<button class="vault-header" onclick={() => toggleVault(vault)}>
								<svg class="v-chev" class:expanded={expandedVaults.has(vault.vault_id)} width="8" height="8" viewBox="0 0 10 10">
									<path d="M3 1 L7 5 L3 9" stroke="currentColor" fill="none" stroke-width="1.5"/>
								</svg>
								<span class="vault-name">{vault.name}</span>
							</button>
							{#if expandedVaults.has(vault.vault_id) && vaultTrees[vault.vault_id]}
								<div class="vault-tree">
									<FileTree
									entries={sortEntries(vaultTrees[vault.vault_id])}
									vaultId={vault.vault_id}
									vaultName={vault.name}
									color={vaultColorMap[vault.name]}
									onNoteClick={handleNoteClick}
									onContextMenu={(entry, x, y) => handleContextMenu(entry, x, y, vault.vault_id)}
									{renamingPath}
									onRenameComplete={handleRenameComplete}
									{allExpanded}
								/>
								</div>
							{/if}
						</div>
					{/each}
					{#if $vaultStats.length === 0}
						<div class="empty-sidebar">
							<p>{$t('sidebar.noVaults')}</p>
							<button class="add-first-btn" onclick={handleAddVault}>{$t('sidebar.addVaultButton')}</button>
						</div>
					{/if}
				{/if}
			</div>

			{#if error}
				<div class="sidebar-error">{error}</div>
			{/if}

			<div class="sidebar-footer-wrap">
				{#if showVaultSwitcher}
					<VaultSwitcher
						colorMap={vaultColorMap}
						onClose={() => showVaultSwitcher = false}
						onAddVault={handleAddVault}
						onManage={() => showVaultManager = true}
					/>
				{/if}
				<button class="sidebar-footer" onmousedown={(e) => e.stopPropagation()} onclick={() => showVaultSwitcher = !showVaultSwitcher}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 10l5-5 5 5"/><path d="M7 14l5 5 5-5"/></svg>
					<span class="footer-name">{$t('vaultManager.manageVaults')}</span>
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
							style:--vault-color={vaultColorMap[tab.vaultName]}
							onclick={() => switchTab(tab.id)}>
							{#if tab.vaultName}<span class="tab-vault">{tab.vaultName}</span>{/if}
							<span class="tab-title">{tab.name}</span>
							<span class="tab-close" role="button" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}>×</span>
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
			{#if showGraphView}
				<div class="graph-fullscreen">
					<div class="graph-header">
						<span class="graph-title">{$t('layout.graphViewTitle')}</span>
						<button class="graph-close" onclick={() => showGraphView = false}>×</button>
					</div>
					<GraphView
						nodes={graphNodes}
						links={graphLinks}
						onNodeClick={handleGraphNodeClick}
						activeNodeId={sidebarTab?.name?.toLowerCase() ?? ''}
					/>
				</div>
			{:else if isHome && ($activeTab || $splitActive)}
				<div class="pane-container" class:horizontal={$splitActive && $splitDirection === 'horizontal'}>
					{#if $splitActive}
						{#each $openTabs as tab, i (tab.id)}
							{#if i > 0}
								<div class="pane-divider"></div>
							{/if}
							<NotePane {tab} isFocused={$focusedTabId === tab.id} onFocus={() => setFocusedTab(tab.id)} color={vaultColorMap[tab.vaultName]} splitView {vaultTrees} allTags={allTagsList} {allNotes} {vaultColorMap} />
						{/each}
					{:else}
						<NotePane tab={$activeTab} isFocused={true} onFocus={() => {}} {vaultTrees} allTags={allTagsList} {allNotes} {vaultColorMap}
						onCreateNote={handleNewNote}
						onQuickSwitch={() => showQuickSwitcher = true}
						onCloseTab={() => { if ($activeTab) closeTab($activeTab.id); }}
					/>
					{/if}
				</div>
			{:else if isHome}
				<div class="welcome">
					{#if $vaultStats.length === 0}
						<svg class="w-icon" width="80" height="80" viewBox="0 0 160 160" fill="none">
								<defs>
									<linearGradient id="wStarGrad" x1="0%" y1="0%" x2="100%" y2="100%">
										<stop offset="0%" stop-color="#8b5cf6" />
										<stop offset="100%" stop-color="#7c3aed" />
									</linearGradient>
								</defs>
								<!-- Center star (largest) -->
								<path d="M80,64 L85,75 L96,80 L85,85 L80,96 L75,85 L64,80 L75,75 Z" fill="url(#wStarGrad)" />
								<!-- Top star -->
								<path d="M80,42 L82.5,47 L88,50 L82.5,53 L80,58 L77.5,53 L72,50 L77.5,47 Z" fill="url(#wStarGrad)" />
								<!-- Top-right star -->
								<path d="M108,58 L110.5,63 L116,66 L110.5,69 L108,74 L105.5,69 L100,66 L105.5,63 Z" fill="url(#wStarGrad)" />
								<!-- Bottom-right star -->
								<path d="M104,96 L106.5,101 L112,104 L106.5,107 L104,112 L101.5,107 L96,104 L101.5,101 Z" fill="url(#wStarGrad)" />
								<!-- Bottom-left star -->
								<path d="M60,102 L62,106 L67,109 L62,112 L60,116 L58,112 L53,109 L58,106 Z" fill="url(#wStarGrad)" />
								<!-- Left star -->
								<path d="M50,66 L52.5,71 L58,74 L52.5,77 L50,82 L47.5,77 L42,74 L47.5,71 Z" fill="url(#wStarGrad)" />
							</svg>
						<p class="w-title">{$t('welcome.title')}</p>
						<p class="w-sub">{$t('welcome.subtitle')}</p>
						<button class="w-btn" onclick={handleAddVault}>{$t('welcome.addVault')}</button>
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
		<div class="rs-inner" style:width="{rightSidebarWidth}px" dir={noteDir}>
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
				<button class="rs-tab" class:active={rightSidebarTab === 'links'} onclick={() => rightSidebarTab = 'links'} title={$t('panels.linkDashboard')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M8 12h8M12 8v8"/></svg>
				</button>
			</div>

			{#if isHome && sidebarTab}
				{#if rightSidebarTab === 'properties'}
					<!-- Properties Panel (interactive editor) -->
					<div class="rs-section">
						{#if sidebarTab}
							<PropertyEditor
								properties={sidebarProperties}
								body={sidebarBody}
								tabId={sidebarTab.id}
								filePath={sidebarTab.path}
								vaultName={sidebarTab.vaultName}
								onNoteClick={async (noteName) => {
									if (!sidebarTab) return;
									const resolved = await resolveWikilinkCrossVault(sidebarTab.vaultPath, noteName);
									if (resolved) {
										const vc = $vaults.find(v => v.name === resolved.vault_name)?.color || '#7c3aed';
										await openNoteTab(resolved.path, resolved.vault_name, vc);
									}
								}}
							/>
						{:else}
							<div class="rs-empty">{$t('panels.noProperties')}</div>
						{/if}
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
						/>
					</div>
									<div class="rs-section">
						<div class="rs-header">{$t('panels.outgoingLinksHeader')}</div>
						<OutgoingLinksPanel
							outgoingLinks={currentOutgoing}
						/>
					</div>
{:else if rightSidebarTab === 'tags'}
					<div class="rs-section">
						<div class="rs-header">{$t('panels.tagsHeader')}</div>
						<TagsPanel tags={allVaultTags} onTagClick={handleTagClick} />
					</div>
				{:else if rightSidebarTab === 'links'}
					<div class="rs-section">
						<div class="rs-header">{$t('panels.linkDashboard')}</div>
						<LinkDashboard
							allLinks={allVaultLinks}
							allNotes={allNotes}
							visible={rightSidebarTab === 'links'}
							onNoteClick={(path, vaultName) => openNoteTab(path, vaultName, $vaults.find(v => v.name === vaultName)?.color || '#7c3aed')}
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

	{#if showWorkspaces}
		<WorkspaceManager
			onClose={() => showWorkspaces = false}
		/>
	{/if}

	{#if showSettings}
		<SettingsModal
			onClose={() => showSettings = false}
			commands={getCommands()}
		/>
	{/if}

	{#if showVaultManager}
		<VaultManager
			colorMap={vaultColorMap}
			onClose={() => showVaultManager = false}
			onRefresh={refreshVaultCaches}
		/>
	{/if}

	{#if showVaultPicker}
		<VaultPicker
			colorMap={vaultColorMap}
			onSelect={(vault) => vaultPickerAction === 'folder' ? createFolderInVault(vault) : createNoteInVault(vault)}
			onClose={() => showVaultPicker = false}
		/>
	{/if}

	{#if showNewBaseDialog}
		<NewBaseDialog
			colorMap={vaultColorMap}
			onCreate={(_vault, name, selectedVaults) => createWorkspaceBaseWithVaults(name, selectedVaults)}
			onClose={() => showNewBaseDialog = false}
		/>
	{/if}

	{#if contextMenu}
		<ContextMenu
			x={contextMenu.x}
			y={contextMenu.y}
			items={getContextMenuItems(contextMenu.entry, contextMenu.vaultId)}
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
				<span class="sb-item">{sidebarTab.vaultName}</span>
				<span class="sb-dot">·</span>
				<span class="sb-item">{sidebarTab.name}</span>
			{:else}
				<span class="sb-item">{$t('vaultManager.manageVaults')}</span>
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
			<span class="sb-item">{$vaultCount} {$t('statusBar.vaults')}</span>
			<span class="sb-dot">·</span>
			<span class="sb-item">{$totalStars} {$t('statusBar.notes')}</span>
			{#if activeUniverseName}
				<span class="sb-dot">·</span>
				<button class="sb-universe" onclick={() => showUniverseManager = true} title={$t('universe.manager.heading')}>
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/></svg>
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

	/* ═══ RIBBON ═══ */
	.ribbon {
		grid-row: 1; width: 40px; background: var(--bg-tertiary);
		border-inline-end: 1px solid var(--border);
		display: flex; flex-direction: column;
		justify-content: space-between; align-items: center; padding: 6px 0;
	}
	.ribbon-top, .ribbon-bottom { display: flex; flex-direction: column; align-items: center; gap: 1px; }
	.r-btn {
		width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
		border-radius: 4px; border: none; background: none;
		color: var(--text-secondary); cursor: pointer; text-decoration: none; transition: all 0.1s;
	}
	.r-btn:hover { background: var(--border); color: var(--text); }
	.r-btn.active { color: var(--accent); }

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
	.s-vault { color: var(--accent); flex-shrink: 0; }
	.s-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.no-results { padding: 20px; text-align: center; color: var(--text-muted); font-size: 0.82rem; }

	.vault-header {
		display: flex; align-items: center; gap: 4px; width: 100%; padding: 3px 12px;
		background: none; border: none; color: var(--text-secondary);
		font-size: 0.8rem; font-weight: 600; font-family: inherit; cursor: pointer; text-align: start;
	}
	.vault-header:hover { background: var(--bg-hover); }
	.v-chev { color: var(--text-muted); flex-shrink: 0; transition: transform 0.15s ease; }
	.v-chev.expanded { transform: rotate(90deg); }
	.vault-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.vault-tree { padding-inline-start: 8px; }

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
		border-top: 3px solid var(--vault-color, transparent);
		position: relative;
	}
	.tab.active, .tab.focused {
		background: var(--bg); color: var(--text);
		border: 1px solid var(--border);
		border-top: 3px solid var(--vault-color, var(--accent));
		border-bottom: 1px solid var(--bg);
		margin-bottom: -1px;
	}
	.tab-vault {
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

	/* Graph fullscreen */
	.graph-fullscreen {
		flex: 1; display: flex; flex-direction: column; overflow: hidden;
	}
	.graph-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 8px 16px; border-bottom: 1px solid var(--border);
		background: var(--bg-secondary);
	}
	.graph-title { font-weight: 600; font-size: 0.9rem; }
	.graph-close {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-muted); cursor: pointer;
		font-size: 1.2rem;
	}
	.graph-close:hover { background: var(--border); color: var(--text); }

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
		display: flex; flex-direction: column; overflow-y: auto;
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
		position: absolute; top: 0; inset-inline-start: 0;
		width: 4px; height: 100%;
		cursor: col-resize; z-index: 10;
	}
	.rs-resize:hover, .app.resizing .rs-resize { background: var(--accent); }

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

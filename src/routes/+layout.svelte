<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { dir, locale, toggleLocale } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import {
		vaults, vaultStats, searchResults, totalStars, vaultCount,
		activeTab, openTabs, activeTabId,
		splitActive, splitDirection, focusedTabId, focusedTab,
		loadVaults, loadAllStats, addVault, searchAllStars,
		openNoteTab, closeTab, switchTab, closeNote,
		toggleSplit, toggleSplitDirection, setFocusedTab,
		parseFrontmatter, extractHeadings,
		createNote, createFolder, renameItem, deleteItem,
		startWatchingVault, wasRecentlyWritten,
		loadVaultAppearance, vaultAppearances,
		toggleEditMode, editingTabIds,
		navigateBack, navigateForward,
		scanVaultLinks, scanVaultTags, getBacklinks, getOutgoingLinks,
		buildGraphData, readNotePreview,
		getDailyNotePath, updateLinksOnRename,
		loadBookmarks, addBookmark, removeBookmark, isBookmarked, bookmarks,
		loadSettings, updateSettings, appSettings,
		loadWorkspaces, workspaces,
		type FrontmatterProperty, type HeadingItem, type NoteLink, type GraphNode, type GraphLink
	} from '$lib/vaults/store';
	import type { VaultStats, FileEntry, PropertyType } from '$lib/vaults/store';
	import { get } from 'svelte/store';
	import { detectDir } from '$lib/utils';
	import FileTree from '$lib/components/FileTree.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import QuickSwitcher from '$lib/components/QuickSwitcher.svelte';
	import GraphView from '$lib/components/GraphView.svelte';
	import BacklinksPanel from '$lib/components/BacklinksPanel.svelte';
	import TagsPanel from '$lib/components/TagsPanel.svelte';
	import PagePreview from '$lib/components/PagePreview.svelte';
	import WorkspaceManager from '$lib/components/WorkspaceManager.svelte';
	import OutgoingLinksPanel from '$lib/components/OutgoingLinksPanel.svelte';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	// Sidebar state
	let sidebarOpen = $state(true);
	let searchMode = $state(false);
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Right sidebar
	let rightSidebarOpen = $state(false);
	let rightSidebarTab = $state<'properties' | 'backlinks' | 'tags' | 'graph'>('properties');

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

	// Page preview (hover)
	let pagePreview = $state<{ content: string; x: number; y: number; visible: boolean }>({ content: '', x: 0, y: 0, visible: false });
	let previewTimeout: ReturnType<typeof setTimeout>;

	// Vault data caches
	let allVaultLinks = $state<NoteLink[]>([]);
	let allVaultTags = $state<Record<string, number>>({});
	let allNotes = $state<{ name: string; path: string; vaultName: string }[]>([]);
	let graphNodes = $state<GraphNode[]>([]);
	let graphLinks = $state<GraphLink[]>([]);

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
		}

		document.addEventListener('mousemove', onMouseMove);
		document.addEventListener('mouseup', onMouseUp);
	}

	// Vault trees
	let vaultTrees = $state<Record<string, FileEntry[]>>({});
	let expandedVaults = $state<Set<string>>(new Set());

	let error = $state('');
	let adding = $state(false);

	const ar = $derived($locale === 'ar');
	const isHome = $derived(page.url.pathname === '/');

	// Vault color palette
	const VAULT_COLORS = ['#7c3aed', '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#06b6d4', '#8b5cf6'];
	const vaultColorMap = $derived.by(() => {
		const map: Record<string, string> = {};
		$vaults.forEach((v, i) => { map[v.name] = VAULT_COLORS[i % VAULT_COLORS.length]; });
		return map;
	});

	const TYPE_ICONS: Record<PropertyType, string> = { text: '\u2261', number: '#', date: '\uD83D\uDCC5', list: '\u2255', link: '\uD83D\uDD17' };

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

	// Outgoing links for current note
	const currentOutgoing = $derived.by(() => {
		if (!sidebarTab) return [];
		return getOutgoingLinks(allVaultLinks, sidebarTab.path).map(l => ({
			target: l.target,
			context: l.context,
		}));
	});

	// Status bar stats from focused tab
	const wordCount = $derived(sidebarTab ? sidebarTab.content.split(/\s+/).filter(w => w.length > 0).length : 0);
	const charCount = $derived(sidebarTab ? sidebarTab.content.length : 0);

	// All note names across all vaults (for quick switcher)
	const allSwitcherNotes = $derived(allNotes);

	// Dark mode
	const colorScheme = $derived($appSettings.colorScheme);
	$effect(() => {
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-theme', colorScheme === 'system'
				? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
				: colorScheme
			);
		}
	});

	// ─── Commands for command palette ───
	function getCommands() {
		return [
			{ id: 'new-note', name: ar ? 'ملاحظة جديدة' : 'New note', shortcut: 'Ctrl+N', icon: '📄', action: handleNewNote, category: 'File' },
			{ id: 'quick-switch', name: ar ? 'التبديل السريع' : 'Quick switcher', shortcut: 'Ctrl+O', icon: '🔍', action: () => { showCommandPalette = false; showQuickSwitcher = true; }, category: 'Navigation' },
			{ id: 'search', name: ar ? 'بحث في الخزينة' : 'Search vault', shortcut: 'Ctrl+Shift+F', icon: '🔎', action: () => { sidebarOpen = true; searchMode = true; }, category: 'Navigation' },
			{ id: 'daily-note', name: ar ? 'ملاحظة اليوم' : 'Open daily note', icon: '📅', action: handleOpenDailyNote, category: 'Daily Notes' },
			{ id: 'toggle-edit', name: ar ? 'تبديل وضع التحرير' : 'Toggle edit/reading mode', shortcut: 'Ctrl+E', icon: '✏️', action: () => { const tab = get(focusedTab); if (tab) toggleEditMode(tab.id); }, category: 'Editor' },
			{ id: 'graph-view', name: ar ? 'عرض الرسم البياني' : 'Open graph view', icon: '🕸️', action: () => showGraphView = !showGraphView, category: 'View' },
			{ id: 'toggle-bold', name: ar ? 'تبديل الخط العريض' : 'Toggle bold', shortcut: 'Ctrl+B', icon: '𝐁', action: () => {}, category: 'Editor' },
			{ id: 'toggle-italic', name: ar ? 'تبديل المائل' : 'Toggle italic', shortcut: 'Ctrl+I', icon: '𝐼', action: () => {}, category: 'Editor' },
			{ id: 'split-view', name: ar ? 'تقسيم العرض' : 'Toggle split view', icon: '⊞', action: cycleSplit, category: 'View' },
			{ id: 'close-note', name: ar ? 'إغلاق الملاحظة' : 'Close current note', shortcut: 'Ctrl+W', icon: '✕', action: closeNote, category: 'File' },
			{ id: 'toggle-left', name: ar ? 'تبديل الشريط الأيسر' : 'Toggle left sidebar', shortcut: 'Ctrl+\\', icon: '◧', action: () => sidebarOpen = !sidebarOpen, category: 'View' },
			{ id: 'toggle-right', name: ar ? 'تبديل الشريط الأيمن' : 'Toggle right sidebar', icon: '◨', action: () => rightSidebarOpen = !rightSidebarOpen, category: 'View' },
			{ id: 'add-vault', name: ar ? 'إضافة خزينة' : 'Add vault', icon: '📁', action: handleAddVault, category: 'Vault' },
			{ id: 'toggle-bookmark', name: ar ? 'تبديل المفضلة' : 'Toggle bookmark', icon: '⭐', action: handleToggleBookmark, category: 'Bookmarks' },
			{ id: 'random-note', name: ar ? 'ملاحظة عشوائية' : 'Open random note', icon: '🎲', action: handleRandomNote, category: 'Navigation' },
			{ id: 'toggle-theme', name: ar ? 'تبديل المظهر' : 'Toggle dark/light mode', icon: '🌗', action: handleToggleTheme, category: 'Appearance' },
			{ id: 'nav-back', name: ar ? 'رجوع' : 'Navigate back', shortcut: 'Alt+←', icon: '←', action: navigateBack, category: 'Navigation' },
			{ id: 'nav-forward', name: ar ? 'تقدم' : 'Navigate forward', shortcut: 'Alt+→', icon: '→', action: navigateForward, category: 'Navigation' },
			{ id: 'workspaces', name: ar ? 'مساحات العمل' : 'Manage workspaces', icon: '🗂️', action: () => { showCommandPalette = false; showWorkspaces = true; }, category: 'View' },
		];
	}

	// ─── Lifecycle ───
	onMount(async () => {
		loadSettings();
		loadBookmarks();
		loadWorkspaces();
		await loadVaults();
		await loadAllStats();

		// Start file watchers and load appearance for all vaults
		for (const vault of $vaults) {
			try { await startWatchingVault(vault.id, vault.path); } catch { /* ignore */ }
			await loadVaultAppearance(vault.path, vault.id);
		}

		// Build vault data caches
		await refreshVaultCaches();

		// Listen for file change events from the watcher
		const unlistenWatcher = await listen<{ vaultId: string; paths: string[] }>('vault-changed', async (event) => {
			const { vaultId, paths } = event.payload;
			await refreshVaultTree(vaultId);
			await loadAllStats();
			// Reload any open tabs whose files changed (if not self-triggered)
			const tabs = get(openTabs);
			for (const changedPath of paths) {
				if (wasRecentlyWritten(changedPath)) continue;
				const tab = tabs.find(t => t.path === changedPath);
				if (tab) {
					try {
						const content: string = await invoke('read_note', { filePath: changedPath });
						openTabs.update(tabs => tabs.map(t =>
							t.path === changedPath ? { ...t, content } : t
						));
					} catch { /* file may have been deleted */ }
				}
			}
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
		for (const fn of cleanupFns) fn();
	});

	async function refreshVaultCaches() {
		const links: NoteLink[] = [];
		const tags: Record<string, number> = {};
		const notes: { name: string; path: string; vaultName: string }[] = [];

		for (const vault of $vaults) {
			try {
				const vaultLinks = await scanVaultLinks(vault.path);
				links.push(...vaultLinks);
			} catch { /* ignore */ }
			try {
				const vaultTags = await scanVaultTags(vault.path);
				for (const [tag, count] of Object.entries(vaultTags)) {
					tags[tag] = (tags[tag] || 0) + count;
				}
			} catch { /* ignore */ }
			try {
				const vaultNotes: any[] = await invoke('collect_vault_notes', { vaultPath: vault.path });
				notes.push(...vaultNotes.map((n: any) => ({ name: n.name, path: n.path, vaultName: vault.name })));
			} catch { /* ignore */ }
		}

		allVaultLinks = links;
		allVaultTags = tags;
		allNotes = notes;

		// Build graph data from all vaults combined
		if ($vaults.length > 0) {
			const { nodes, links: gLinks } = buildGraphData(links, notes);
			graphNodes = nodes;
			graphLinks = gLinks;
		}
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
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
		}
	}

	// ─── Actions ───
	async function handleNewNote() {
		const firstVault = $vaults[0];
		if (!firstVault) return;
		try {
			const name = ar ? 'بدون عنوان' : 'Untitled';
			const newPath = await createNote(firstVault.path, name);
			await refreshVaultTree(firstVault.id);
			const vaultColor = vaultColorMap[firstVault.name] ?? '#7c3aed';
			await openNoteTab(newPath, firstVault.name, vaultColor);
			// Auto-enter edit mode
			const tab = get(focusedTab);
			if (tab) toggleEditMode(tab.id);
		} catch (e) {
			console.error('Failed to create note:', e);
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

		previewTimeout = setTimeout(async () => {
			try {
				const resolved = await invoke<string | null>('resolve_wikilink', { vaultPath: sidebarTab!.vaultPath, target: linkTarget });
				if (resolved) {
					const content = await readNotePreview(resolved, 800);
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
			searchTimeout = setTimeout(() => searchAllStars(searchQuery), 300);
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
		if (entry.is_dir) {
			items.push({
				label: ar ? 'ملاحظة جديدة' : 'New Note',
				icon: '📄',
				action: () => handleCreateNote(entry.path, vaultId)
			});
			items.push({
				label: ar ? 'مجلد جديد' : 'New Folder',
				icon: '📁',
				action: () => handleCreateFolder(entry.path, vaultId)
			});
		}
		items.push({
			label: ar ? 'إعادة تسمية' : 'Rename',
			icon: '✏️',
			action: () => { renamingPath = entry.path; }
		});
		items.push({
			label: ar ? 'حذف' : 'Delete',
			icon: '🗑️',
			action: () => { confirmDelete = { path: entry.path, name: entry.name }; },
			danger: true
		});
		return items;
	}

	async function handleCreateNote(folderPath: string, vaultId: string) {
		try {
			const name = ar ? 'بدون عنوان' : 'Untitled';
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

	async function handleCreateFolder(parentPath: string, vaultId: string) {
		try {
			const name = ar ? 'مجلد جديد' : 'New Folder';
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

	async function handleNoteClick(filePath: string, _noteName: string) {
		const vault = $vaults.find(v => filePath.startsWith(v.path));
		const vaultColor = vault ? vaultColorMap[vault.name] : '#7c3aed';
		await openNoteTab(filePath, vault?.name ?? '', vaultColor);
		if (!isHome) window.location.href = '/';
	}

	async function handleSearchResultClick(path: string, vaultName: string) {
		const vaultColor = vaultColorMap[vaultName] ?? '#7c3aed';
		await openNoteTab(path, vaultName, vaultColor);
		clearSearch();
		if (!isHome) window.location.href = '/';
	}

	// Get all tags as flat array for editor autocomplete
	const allTagsList = $derived(Object.keys(allVaultTags));
</script>

<div class="app" dir={$dir} class:resizing={resizing !== null} class:no-sidebar={!sidebarOpen} class:dark={colorScheme === 'dark'}>
	<!-- ═══ RIBBON ═══ -->
	<div class="ribbon">
		<div class="ribbon-top">
			<button class="r-btn" onclick={() => { sidebarOpen = true; searchMode = false; }} title={ar ? 'المستكشف' : 'File explorer'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
			</button>
			<button class="r-btn" onclick={() => { sidebarOpen = true; searchMode = true; }} title={ar ? 'بحث' : 'Search'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<button class="r-btn" onclick={() => showGraphView = !showGraphView} title={ar ? 'الرسم البياني' : 'Graph view'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><circle cx="18" cy="6" r="3"/><path d="M6 9v6M9 6h6M15 18h-6"/></svg>
			</button>
			<button class="r-btn" onclick={handleOpenDailyNote} title={ar ? 'ملاحظة اليوم' : 'Daily note'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
			</button>
			<a href="/skills" class="r-btn" class:active={page.url.pathname === '/skills'} title={ar ? 'المهارات' : 'AI Skills'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01z"/></svg>
			</a>
		</div>
		<div class="ribbon-bottom">
			<button class="r-btn" onclick={handleToggleTheme} title={ar ? 'تبديل المظهر' : 'Toggle theme'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
			</button>
			<button class="r-btn" onclick={toggleLocale} title="Language / اللغة">
				<span class="r-lang">{$locale === 'en' ? 'ع' : 'En'}</span>
			</button>
			<a href="/settings" class="r-btn" class:active={page.url.pathname === '/settings'} title={ar ? 'الإعدادات' : 'Settings'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
			</a>
		</div>
	</div>

	<!-- ═══ LEFT SIDEBAR ═══ -->
	{#if sidebarOpen}
		<aside class="sidebar" style:width="{leftSidebarWidth}px">
			<div class="sidebar-toolbar">
				{#if searchMode}
					<div class="search-box">
						<svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
						<input type="text" placeholder={ar ? 'بحث...' : 'Search...'} value={searchQuery} oninput={handleSearch}/>
						<button class="search-clear" onclick={clearSearch}>×</button>
					</div>
				{:else}
					<div class="toolbar-actions">
						<button class="tb-btn" onclick={handleAddVault} disabled={adding} title={ar ? 'إضافة خزينة' : 'Add vault'}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
						</button>
					</div>
				{/if}
			</div>

			<div class="sidebar-content">
				{#if searchMode && searchQuery}
					{#if $searchResults.length > 0}
						<div class="section-label">{$searchResults.length} {ar ? 'نتيجة' : 'results'}</div>
						{#each $searchResults as star}
							<button class="s-result" class:active={$activeTab?.path === star.path} onclick={() => handleSearchResultClick(star.path, star.vault_name)}>
								<div class="s-name">{star.name}</div>
								<div class="s-meta">
									<span class="s-vault">{star.vault_name}</span>
									<span class="s-preview">{star.preview}</span>
								</div>
							</button>
						{/each}
					{:else}
						<div class="no-results">{ar ? 'لا توجد نتائج' : 'No results found'}</div>
					{/if}
				{:else}
					<!-- Bookmarks section -->
					{#if $bookmarks.length > 0}
						<div class="section-label">{ar ? 'المفضلة' : 'Bookmarks'}</div>
						{#each $bookmarks as bm}
							<button class="s-result" onclick={() => handleNoteClick(bm.path, bm.name)}>
								<div class="s-name">⭐ {bm.name}</div>
								<div class="s-meta"><span class="s-vault">{bm.vaultName}</span></div>
							</button>
						{/each}
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
									entries={vaultTrees[vault.vault_id]}
									vaultId={vault.vault_id}
									vaultName={vault.name}
									color={vaultColorMap[vault.name]}
									onNoteClick={handleNoteClick}
									onContextMenu={(entry, x, y) => handleContextMenu(entry, x, y, vault.vault_id)}
									{renamingPath}
									onRenameComplete={handleRenameComplete}
								/>
								</div>
							{/if}
						</div>
					{/each}
					{#if $vaultStats.length === 0}
						<div class="empty-sidebar">
							<p>{ar ? 'لا توجد خزائن' : 'No vaults yet'}</p>
							<button class="add-first-btn" onclick={handleAddVault}>{ar ? '+ إضافة خزينة' : '+ Add vault'}</button>
						</div>
					{/if}
				{/if}
			</div>

			{#if error}
				<div class="sidebar-error">{error}</div>
			{/if}

			<div class="sidebar-footer">
				<span class="footer-name">Constellation</span>
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="sidebar-resize" onmousedown={(e) => startResize('left', e)}></div>
		</aside>
	{/if}

	<!-- ═══ MAIN AREA ═══ -->
	<div class="main-area">
		<!-- Tab Bar -->
		<div class="tab-bar">
			<button class="tab-action" class:active={sidebarOpen} onclick={() => sidebarOpen = !sidebarOpen} title={ar ? 'القائمة الجانبية' : 'Left sidebar'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/></svg>
			</button>
			{#if !$splitActive}
				<div class="tab-scroll">
					{#each $openTabs as tab (tab.id)}
						<button class="tab"
							class:active={$activeTabId === tab.id}
							style:--vault-color={vaultColorMap[tab.vaultName]}
							onclick={() => switchTab(tab.id)}>
							<span class="tab-vault">{tab.vaultName}</span>
							<span class="tab-title">{tab.name}</span>
							<span class="tab-close" role="button" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}>×</span>
						</button>
					{/each}
					{#if !isHome}
						<div class="tab active">
							<span class="tab-title">{page.url.pathname === '/settings' ? (ar ? 'الإعدادات' : 'Settings') : (ar ? 'المهارات' : 'Skills')}</span>
							<a class="tab-close" href="/">×</a>
						</div>
					{/if}
				</div>
			{:else}
				<div class="tab-scroll"></div>
			{/if}
			<button class="tab-action" class:active={$splitActive} onclick={cycleSplit} title={ar ? 'تقسيم' : 'Split view'}>
				{#if $splitActive && $splitDirection === 'horizontal'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 12h18"/></svg>
				{:else}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 3v18"/></svg>
				{/if}
			</button>
			<button class="tab-action" class:active={rightSidebarOpen} onclick={() => rightSidebarOpen = !rightSidebarOpen} title={ar ? 'اللوحة الجانبية' : 'Right sidebar'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M15 3v18"/></svg>
			</button>
		</div>

		<!-- Content -->
		<div class="content-area" onmouseover={handleWikilinkHover} onmouseout={handleWikilinkLeave}>
			{#if showGraphView}
				<div class="graph-fullscreen">
					<div class="graph-header">
						<span class="graph-title">{ar ? 'الرسم البياني' : 'Graph View'}</span>
						<button class="graph-close" onclick={() => showGraphView = false}>×</button>
					</div>
					<GraphView
						nodes={graphNodes}
						links={graphLinks}
						onNodeClick={handleGraphNodeClick}
						activeNodeId={sidebarTab?.name?.toLowerCase() ?? ''}
						{ar}
					/>
				</div>
			{:else if isHome && ($activeTab || $splitActive)}
				<div class="pane-container" class:horizontal={$splitActive && $splitDirection === 'horizontal'}>
					{#if $splitActive}
						{#each $openTabs as tab, i (tab.id)}
							{#if i > 0}
								<div class="pane-divider"></div>
							{/if}
							<NotePane {tab} isFocused={$focusedTabId === tab.id} onFocus={() => setFocusedTab(tab.id)} {ar} color={vaultColorMap[tab.vaultName]} splitView {vaultTrees} allTags={allTagsList} />
						{/each}
					{:else}
						<NotePane tab={$activeTab} isFocused={true} onFocus={() => {}} {ar} {vaultTrees} allTags={allTagsList} />
					{/if}
				</div>
			{:else if isHome}
				<div class="welcome">
					{#if $vaultStats.length === 0}
						<svg class="w-icon" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#7c3aed" stroke-width="1.5"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01z"/></svg>
						<p class="w-title">{ar ? 'مرحبا بك في كونستليشن' : 'Welcome to Constellation'}</p>
						<p class="w-sub">{ar ? 'أضف خزينة أوبسيديان الأولى للبدء' : 'Add your first Obsidian vault to get started'}</p>
						<button class="w-btn" onclick={handleAddVault}>{ar ? '+ إضافة خزينة' : '+ Add Vault'}</button>
					{:else}
						<p class="w-hint">{ar ? 'اختر ملاحظة من الشريط الجانبي' : 'Select a note from the sidebar'}</p>
						<p class="w-hint-sub">{ar ? 'أو اضغط Ctrl+O للتبديل السريع' : 'or press Ctrl+O to quick switch'}</p>
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
				<button class="rs-tab" class:active={rightSidebarTab === 'properties'} onclick={() => rightSidebarTab = 'properties'} title={ar ? 'خصائص' : 'Properties'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'backlinks'} onclick={() => rightSidebarTab = 'backlinks'} title={ar ? 'روابط واردة' : 'Backlinks'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
				</button>
				<button class="rs-tab" class:active={rightSidebarTab === 'tags'} onclick={() => rightSidebarTab = 'tags'} title={ar ? 'وسوم' : 'Tags'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
				</button>
			</div>

			{#if isHome && sidebarTab}
				{#if rightSidebarTab === 'properties'}
					<!-- Properties Panel -->
					<div class="rs-section">
						<div class="rs-header">{ar ? 'الخصائص' : 'Properties'}</div>
						{#if sidebarProperties.length > 0}
							{#each sidebarProperties as prop}
								<div class="rs-prop">
									<span class="rs-prop-icon">{TYPE_ICONS[prop.type]}</span>
									<span class="rs-prop-key">{prop.key}</span>
									<span class="rs-prop-val">{prop.value || '—'}</span>
								</div>
							{/each}
						{:else}
							<div class="rs-empty">{ar ? 'لا توجد خصائص' : 'No properties'}</div>
						{/if}
					</div>

					<!-- Outline Panel -->
					<div class="rs-section">
						<div class="rs-header">{ar ? 'المحتويات' : 'Outline'}</div>
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
							<div class="rs-empty">{ar ? 'لا توجد عناوين' : 'No headings'}</div>
						{/if}
					</div>
				{:else if rightSidebarTab === 'backlinks'}
					<div class="rs-section">
						<div class="rs-header">{ar ? 'الروابط الواردة' : 'Backlinks'}</div>
						<BacklinksPanel
							backlinks={currentBacklinks}
							unlinkedMentions={[]}
							{ar}
						/>
					</div>
									<div class="rs-section">
						<div class="rs-header">{ar ? 'الروابط الصادرة' : 'Outgoing Links'}</div>
						<OutgoingLinksPanel
							outgoingLinks={currentOutgoing}
							{ar}
						/>
					</div>
{:else if rightSidebarTab === 'tags'}
					<div class="rs-section">
						<div class="rs-header">{ar ? 'الوسوم' : 'Tags'}</div>
						<TagsPanel tags={allVaultTags} onTagClick={handleTagClick} {ar} />
					</div>
				{/if}
			{:else}
				<div class="rs-empty-full">{ar ? 'لا توجد ملاحظة مفتوحة' : 'No note selected'}</div>
			{/if}
		</div>
	</aside>

	<!-- ═══ OVERLAYS ═══ -->
	{#if showCommandPalette}
		<CommandPalette
			commands={getCommands()}
			onClose={() => showCommandPalette = false}
			{ar}
		/>
	{/if}

	{#if showQuickSwitcher}
		<QuickSwitcher
			notes={allSwitcherNotes}
			onSelect={handleQuickSwitchSelect}
			onClose={() => showQuickSwitcher = false}
			{ar}
		/>
	{/if}

	{#if showWorkspaces}
		<WorkspaceManager
			onClose={() => showWorkspaces = false}
			{ar}
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
			message={ar ? `هل أنت متأكد من حذف "${confirmDelete.name}"؟` : `Are you sure you want to delete "${confirmDelete.name}"?`}
			confirmLabel={ar ? 'حذف' : 'Delete'}
			cancelLabel={ar ? 'إلغاء' : 'Cancel'}
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

	<!-- ═══ STATUS BAR ═══ -->
	<div class="status-bar">
		<div class="sb-left">
			{#if sidebarTab}
				<span class="sb-item">{sidebarTab.vaultName}</span>
				<span class="sb-dot">·</span>
				<span class="sb-item">{sidebarTab.name}</span>
			{:else}
				<span class="sb-item">Constellation</span>
			{/if}
		</div>
		<div class="sb-right">
			{#if sidebarTab}
				{#if sidebarProperties.length > 0}
					<span class="sb-item">{sidebarProperties.length} {ar ? 'خصائص' : 'properties'}</span>
					<span class="sb-dot">·</span>
				{/if}
				<span class="sb-item">{wordCount} {ar ? 'كلمة' : 'words'}</span>
				<span class="sb-dot">·</span>
				<span class="sb-item">{charCount} {ar ? 'حرف' : 'characters'}</span>
				<span class="sb-dot">·</span>
			{/if}
			<span class="sb-item">{$vaultCount} {ar ? 'خزائن' : 'vaults'}</span>
			<span class="sb-dot">·</span>
			<span class="sb-item">{$totalStars} {ar ? 'ملاحظة' : 'notes'}</span>
		</div>
	</div>
</div>

<style>
	:global(html) { margin: 0; padding: 0; overflow: hidden; }
	:global(body) {
		margin: 0; padding: 0;
		font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif;
		font-size: 15px; line-height: 1.5; overflow: hidden;
	}
	:global(a) { text-decoration: none; }
	:global(*) { box-sizing: border-box; }

	/* ═══ THEME VARIABLES ═══ */
	:global(html), :global([data-theme="light"]) {
		--bg: #fff;
		--bg-secondary: #f6f6f9;
		--bg-tertiary: #f0f0f4;
		--bg-hover: #ebebef;
		--border: #e0e0e4;
		--border-light: #e8e8ec;
		--text: #1f2328;
		--text-secondary: #5c5c66;
		--text-muted: #8b8b96;
		--text-faint: #b0b0b8;
		--accent: #7c3aed;
		--accent-hover: #6d28d9;
		--accent-bg: #d5ccf7;
		--danger: #cf222e;
		color-scheme: light;
	}
	:global([data-theme="dark"]) {
		--bg: #1e1e2e;
		--bg-secondary: #181825;
		--bg-tertiary: #252536;
		--bg-hover: #313244;
		--border: #313244;
		--border-light: #3a3a4e;
		--text: #cdd6f4;
		--text-secondary: #a6adc8;
		--text-muted: #6c7086;
		--text-faint: #585b70;
		--accent: #b4befe;
		--accent-hover: #cba6f7;
		--accent-bg: #313244;
		--danger: #f38ba8;
		color-scheme: dark;
	}

	:global(body) {
		background: var(--bg);
		color: var(--text);
	}
	:global(a) { color: var(--accent); }
	:global(a:hover) { color: var(--accent-hover); }

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
	.r-lang { font-size: 0.75rem; font-weight: 700; font-family: inherit; }

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
	.toolbar-actions { display: flex; gap: 2px; align-items: center; margin-inline-start: auto; }
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

	.empty-sidebar { padding: 20px 16px; text-align: center; }
	.empty-sidebar p { color: var(--text-muted); font-size: 0.85rem; margin-bottom: 10px; }
	.add-first-btn {
		background: none; border: 1px dashed var(--border); border-radius: 4px;
		padding: 4px 12px; color: var(--text-muted); font-size: 0.82rem; cursor: pointer; font-family: inherit;
	}
	.add-first-btn:hover { border-color: var(--accent); color: var(--accent); }
	.sidebar-error { padding: 6px 12px; color: var(--danger); font-size: 0.75rem; }

	.sidebar-footer {
		border-top: 1px solid var(--border); padding: 4px 12px;
		display: flex; align-items: center; min-height: 30px;
	}
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
		flex: 1; min-width: 0; display: flex; align-items: flex-end;
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
	.w-icon { margin-bottom: 16px; opacity: 0.5; }
	.w-title { color: var(--text); font-size: 1.2rem; font-weight: 600; margin: 0 0 4px; }
	.w-sub { font-size: 0.9rem; margin: 0 0 16px; }
	.w-hint { font-size: 0.9rem; }
	.w-hint-sub { font-size: 0.78rem; color: var(--text-faint); margin-top: 4px; }
	.w-btn {
		background: var(--accent); border: none; color: #fff;
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
</style>

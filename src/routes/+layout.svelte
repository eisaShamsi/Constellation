<script lang="ts">
	import { onMount } from 'svelte';
	import { dir, locale, toggleLocale } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import {
		vaults, vaultStats, searchResults, totalStars, vaultCount,
		activeTab, openTabs, activeTabId,
		splitActive, splitDirection, focusedTabId, focusedTab,
		loadVaults, loadAllStats, addVault, searchAllStars,
		openNoteTab, closeTab, switchTab,
		toggleSplit, toggleSplitDirection, setFocusedTab,
		parseFrontmatter, extractHeadings,
		createNote, createFolder, renameItem, deleteItem,
		startWatchingVault, wasRecentlyWritten,
		loadVaultAppearance, vaultAppearances,
		type FrontmatterProperty, type HeadingItem
	} from '$lib/vaults/store';
	import type { VaultStats, FileEntry, PropertyType } from '$lib/vaults/store';
	import { get } from 'svelte/store';
	import { detectDir } from '$lib/utils';
	import FileTree from '$lib/components/FileTree.svelte';
	import NotePane from '$lib/components/NotePane.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
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

	// Sidebar resizing
	let leftSidebarWidth = $state(240);
	let rightSidebarWidth = $state(260);
	let resizing = $state<'left' | 'right' | null>(null);

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

	// Status bar stats from focused tab
	const wordCount = $derived(sidebarTab ? sidebarTab.content.split(/\s+/).filter(w => w.length > 0).length : 0);
	const charCount = $derived(sidebarTab ? sidebarTab.content.length : 0);

	onMount(async () => {
		await loadVaults();
		await loadAllStats();

		// Start file watchers and load appearance for all vaults
		for (const vault of $vaults) {
			try { await startWatchingVault(vault.id, vault.path); } catch { /* ignore */ }
			await loadVaultAppearance(vault.path, vault.id);
		}

		// Listen for file change events from the watcher
		await listen<{ vaultId: string; paths: string[] }>('vault-changed', async (event) => {
			const { vaultId, paths } = event.payload;
			// Refresh the tree for this vault
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
	});

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
		try { await addVault(); await loadAllStats(); }
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
			// Find which vault this belongs to
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
			await renameItem(oldPath, newPath);
			const vault = $vaultStats.find(v => oldPath.startsWith(v.path));
			if (vault) await refreshVaultTree(vault.vault_id);
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
</script>

<div class="app" dir={$dir} class:resizing={resizing !== null} class:no-sidebar={!sidebarOpen}>
	<!-- ═══ RIBBON ═══ -->
	<div class="ribbon">
		<div class="ribbon-top">
			<button class="r-btn" onclick={() => { sidebarOpen = true; searchMode = false; }} title={ar ? 'المستكشف' : 'File explorer'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
			</button>
			<button class="r-btn" onclick={() => { sidebarOpen = true; searchMode = true; }} title={ar ? 'بحث' : 'Search'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<a href="/skills" class="r-btn" class:active={page.url.pathname === '/skills'} title={ar ? 'المهارات' : 'AI Skills'}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01z"/></svg>
			</a>
		</div>
		<div class="ribbon-bottom">
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
		<div class="content-area">
			{#if isHome && ($activeTab || $splitActive)}
				<div class="pane-container" class:horizontal={$splitActive && $splitDirection === 'horizontal'}>
					{#if $splitActive}
						{#each $openTabs as tab, i (tab.id)}
							{#if i > 0}
								<div class="pane-divider"></div>
							{/if}
							<NotePane {tab} isFocused={$focusedTabId === tab.id} onFocus={() => setFocusedTab(tab.id)} {ar} color={vaultColorMap[tab.vaultName]} splitView />
						{/each}
					{:else}
						<NotePane tab={$activeTab} isFocused={true} onFocus={() => {}} {ar} />
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
			{#if isHome && sidebarTab}
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
			{:else}
				<div class="rs-empty-full">{ar ? 'لا توجد ملاحظة مفتوحة' : 'No note selected'}</div>
			{/if}
		</div>
	</aside>

	<!-- ═══ CONTEXT MENU ═══ -->
	{#if contextMenu}
		<ContextMenu
			x={contextMenu.x}
			y={contextMenu.y}
			items={getContextMenuItems(contextMenu.entry, contextMenu.vaultId)}
			onClose={() => contextMenu = null}
		/>
	{/if}

	<!-- ═══ CONFIRM DIALOG ═══ -->
	{#if confirmDelete}
		<ConfirmDialog
			message={ar ? `هل أنت متأكد من حذف "${confirmDelete.name}"؟` : `Are you sure you want to delete "${confirmDelete.name}"?`}
			confirmLabel={ar ? 'حذف' : 'Delete'}
			cancelLabel={ar ? 'إلغاء' : 'Cancel'}
			onConfirm={handleDeleteConfirm}
			onCancel={() => confirmDelete = null}
		/>
	{/if}

	<!-- ═══ STATUS BAR ═══ -->
	<div class="status-bar">
		<div class="sb-left">
			{#if sidebarTab}
				<span class="sb-item">{sidebarTab.vaultName}</span>
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
		margin: 0; padding: 0; background: #fff; color: #1f2328;
		font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif;
		font-size: 15px; line-height: 1.5; overflow: hidden;
	}
	:global(a) { color: #7c3aed; text-decoration: none; }
	:global(a:hover) { color: #6d28d9; }
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
		grid-row: 1; width: 40px; background: #ebebef;
		border-inline-end: 1px solid #dcdce0;
		display: flex; flex-direction: column;
		justify-content: space-between; align-items: center; padding: 6px 0;
	}
	.ribbon-top, .ribbon-bottom { display: flex; flex-direction: column; align-items: center; gap: 1px; }
	.r-btn {
		width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
		border-radius: 4px; border: none; background: none;
		color: #5c5c66; cursor: pointer; text-decoration: none; transition: all 0.1s;
	}
	.r-btn:hover { background: #dcdce0; color: #1f2328; }
	.r-btn.active { color: #7c3aed; }
	.r-lang { font-size: 0.75rem; font-weight: 700; font-family: inherit; }

	/* ═══ LEFT SIDEBAR ═══ */
	.sidebar {
		grid-row: 1; background: #f6f6f9;
		border-inline-end: 1px solid #e0e0e4;
		display: flex; flex-direction: column; overflow: hidden;
		position: relative;
	}
	.sidebar-toolbar {
		padding: 4px 6px; border-bottom: 1px solid #e0e0e4;
		min-height: 34px; display: flex; align-items: center;
	}
	.toolbar-actions { display: flex; gap: 2px; align-items: center; margin-inline-start: auto; }
	.tb-btn {
		width: 26px; height: 26px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: #8b8b96; cursor: pointer;
	}
	.tb-btn:hover { background: #e0e0e4; color: #1f2328; }

	.search-box {
		display: flex; align-items: center; gap: 4px; width: 100%;
		background: #fff; border: 1px solid #d0d0d6; border-radius: 4px; padding: 0 6px;
	}
	.search-icon { color: #8b8b96; flex-shrink: 0; }
	.search-box input {
		flex: 1; border: none; background: none; padding: 4px 0;
		font-size: 0.82rem; color: #1f2328; font-family: inherit; outline: none;
	}
	.search-box input::placeholder { color: #b0b0b8; }
	.search-clear { border: none; background: none; color: #8b8b96; cursor: pointer; font-size: 1rem; padding: 0 2px; }

	.sidebar-content { flex: 1; overflow-y: auto; padding: 2px 0; }
	.section-label { padding: 4px 12px; font-size: 0.7rem; color: #8b8b96; text-transform: uppercase; letter-spacing: 0.04em; }
	.s-result {
		display: block; width: 100%; padding: 4px 12px;
		background: none; border: none; color: #1f2328;
		font-family: inherit; cursor: pointer; text-align: start;
	}
	.s-result:hover { background: #ebebef; }
	.s-result.active { background: #d5ccf7; }
	.s-name { font-size: 0.82rem; font-weight: 500; }
	.s-meta { display: flex; gap: 4px; font-size: 0.7rem; color: #8b8b96; margin-top: 1px; }
	.s-vault { color: #7c3aed; flex-shrink: 0; }
	.s-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.no-results { padding: 20px; text-align: center; color: #8b8b96; font-size: 0.82rem; }

	.vault-header {
		display: flex; align-items: center; gap: 4px; width: 100%; padding: 3px 12px;
		background: none; border: none; color: #5c5c66;
		font-size: 0.8rem; font-weight: 600; font-family: inherit; cursor: pointer; text-align: start;
	}
	.vault-header:hover { background: #ebebef; }
	.v-chev { color: #8b8b96; flex-shrink: 0; transition: transform 0.15s ease; }
	.v-chev.expanded { transform: rotate(90deg); }
	.vault-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.vault-tree { padding-inline-start: 8px; }

	.empty-sidebar { padding: 20px 16px; text-align: center; }
	.empty-sidebar p { color: #8b8b96; font-size: 0.85rem; margin-bottom: 10px; }
	.add-first-btn {
		background: none; border: 1px dashed #d0d0d6; border-radius: 4px;
		padding: 4px 12px; color: #8b8b96; font-size: 0.82rem; cursor: pointer; font-family: inherit;
	}
	.add-first-btn:hover { border-color: #7c3aed; color: #7c3aed; }
	.sidebar-error { padding: 6px 12px; color: #cf222e; font-size: 0.75rem; }

	.sidebar-footer {
		border-top: 1px solid #e0e0e4; padding: 4px 12px;
		display: flex; align-items: center; min-height: 30px;
	}
	.footer-name { font-size: 0.78rem; font-weight: 600; color: #5c5c66; }

	/* ═══ MAIN AREA ═══ */
	.main-area {
		grid-row: 1; display: flex; flex-direction: column;
		overflow: hidden; background: #fff;
	}

	/* Tab bar */
	.tab-bar {
		display: flex; align-items: flex-end;
		background: #f0f0f4; border-bottom: 1px solid #e0e0e4;
		min-height: 36px; flex-shrink: 0;
	}
	.tab-scroll {
		flex: 1; min-width: 0; display: flex; align-items: flex-end;
		gap: 1px; padding: 12px 4px 0; overflow-x: auto;
	}
	.tab-scroll::-webkit-scrollbar { height: 0; }
	.tab {
		display: flex; align-items: center; gap: 6px;
		padding: 5px 10px; font-size: 0.8rem; color: #5c5c66;
		background: #e8e8ec; border-radius: 6px 6px 0 0;
		cursor: pointer; min-width: 0;
		border: none; font-family: inherit; flex-shrink: 0;
		border-top: 3px solid var(--vault-color, transparent);
		position: relative;
	}
	.tab.active, .tab.focused {
		background: #fff; color: #1f2328;
		border: 1px solid #e0e0e4;
		border-top: 3px solid var(--vault-color, #7c3aed);
		border-bottom: 1px solid #fff;
		margin-bottom: -1px;
	}
	.tab-vault {
		position: absolute; bottom: 100%; inset-inline-end: 8px;
		font-size: 0.55rem; line-height: 1.3; letter-spacing: 0.02em;
		color: #1f2328;
		background: #f0f0f4;
		padding: 0 5px;
		border-radius: 3px 3px 0 0;
		border: 1px solid #e0e0e4; border-bottom: none;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%; pointer-events: none;
	}
	.tab-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.tab-close {
		background: none; border: none; color: #8b8b96;
		cursor: pointer; font-size: 0.85rem; padding: 0; line-height: 1;
		border-radius: 3px; text-decoration: none;
		display: flex; align-items: center; justify-content: center;
		width: 16px; height: 16px; flex-shrink: 0;
	}
	.tab-close:hover { background: #e0e0e4; color: #1f2328; }
	.tab-action {
		width: 28px; height: 28px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 4px;
		color: #8b8b96; cursor: pointer; flex-shrink: 0; margin: auto 2px;
	}
	.tab-action:hover { background: #e0e0e4; color: #1f2328; }
	.tab-action.active { color: #7c3aed; }

	/* Content */
	.content-area { flex: 1; overflow: hidden; display: flex; flex-direction: column; }

	/* Pane container */
	.pane-container {
		flex: 1; display: flex; flex-direction: row; overflow: hidden;
	}
	.pane-container.horizontal { flex-direction: column; }
	.pane-divider { flex-shrink: 0; background: #e0e0e4; }
	.pane-container:not(.horizontal) > .pane-divider { width: 3px; cursor: col-resize; }
	.pane-container.horizontal > .pane-divider { height: 3px; cursor: row-resize; }
	.pane-divider:hover { background: #7c3aed; }

	/* Welcome */
	.welcome {
		flex: 1; display: flex; flex-direction: column;
		align-items: center; justify-content: center; text-align: center; padding: 2rem; color: #8b8b96;
	}
	.w-icon { margin-bottom: 16px; opacity: 0.5; }
	.w-title { color: #1f2328; font-size: 1.2rem; font-weight: 600; margin: 0 0 4px; }
	.w-sub { font-size: 0.9rem; margin: 0 0 16px; }
	.w-hint { font-size: 0.9rem; }
	.w-btn {
		background: #7c3aed; border: none; color: #fff;
		padding: 8px 20px; border-radius: 6px; cursor: pointer;
		font-size: 0.9rem; font-weight: 600;
	}
	.w-btn:hover { background: #6d28d9; }

	.page-scroll { flex: 1; overflow-y: auto; padding: 2rem; max-width: 900px; margin: 0 auto; width: 100%; }

	/* ═══ RIGHT SIDEBAR ═══ */
	.right-sidebar {
		grid-row: 1; background: #f8f8fb;
		border-inline-start: 1px solid #e0e0e4;
		overflow: hidden;
		transition: width 0.2s ease;
		position: relative;
	}
	.right-sidebar.collapsed { width: 0 !important; border-inline-start: none; }
	.rs-inner {
		height: 100%;
		display: flex; flex-direction: column; overflow-y: auto;
	}

	.rs-section {
		padding: 12px; border-bottom: 1px solid #e8e8ec;
	}
	.rs-header {
		font-size: 0.78rem; font-weight: 600; color: #7c3aed;
		margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.03em;
	}
	.rs-prop {
		display: flex; justify-content: space-between; gap: 8px;
		padding: 3px 0; font-size: 0.8rem;
	}
	.rs-prop-icon { color: #8b8b96; font-size: 0.75rem; flex-shrink: 0; width: 16px; text-align: center; }
	.rs-prop-key { color: #5c5c66; font-weight: 500; }
	.rs-prop-val { color: #1f2328; text-align: end; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.rs-heading {
		display: block; width: 100%; border: none; background: none;
		text-align: start; font-family: inherit;
		padding: 3px 8px; font-size: 0.8rem; color: #5c5c66;
		cursor: pointer; border-radius: 3px;
	}
	.rs-heading:hover { background: #ebebef; color: #1f2328; }
	.rs-empty { font-size: 0.8rem; color: #b0b0b8; }
	.rs-empty-full {
		padding: 24px; text-align: center; color: #b0b0b8; font-size: 0.85rem;
	}

	/* ═══ STATUS BAR ═══ */
	.status-bar {
		grid-column: 1 / -1; grid-row: 2; height: 24px;
		background: #f0f0f4; border-top: 1px solid #e0e0e4;
		display: flex; align-items: center; justify-content: space-between;
		padding: 0 10px; font-size: 0.7rem; color: #8b8b96;
	}
	.sb-left, .sb-right { display: flex; align-items: center; gap: 4px; }
	.sb-dot { color: #d0d0d6; }

	/* ═══ RESIZE HANDLES ═══ */
	.app.resizing { user-select: none; cursor: col-resize; }
	.sidebar-resize {
		position: absolute; top: 0; inset-inline-end: 0;
		width: 4px; height: 100%;
		cursor: col-resize; z-index: 10;
	}
	.sidebar-resize:hover, .app.resizing .sidebar-resize { background: #7c3aed; }
	.rs-resize {
		position: absolute; top: 0; inset-inline-start: 0;
		width: 4px; height: 100%;
		cursor: col-resize; z-index: 10;
	}
	.rs-resize:hover, .app.resizing .rs-resize { background: #7c3aed; }
</style>

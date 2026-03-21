<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import { dir } from '$lib/i18n';
	import { libraries, appSettings, type FileEntry } from '$lib/libraries/store';
	import { getChildUniverses, type ChildUniverseInfo } from '$lib/universe/store';
	import { detectDir } from '$lib/utils';

	let {
		libraryColorMap = {} as Record<string, string>,
		universeName = '',
		embedded = false,
		selectedPath = $bindable(null as string | string[] | null),
		onNoteClick,
		onClose,
	}: {
		libraryColorMap?: Record<string, string>;
		universeName?: string;
		embedded?: boolean;
		selectedPath?: string | string[] | null;
		onNoteClick?: (path: string, name: string, highlightTerm?: string, e?: MouseEvent) => void;
		onClose?: () => void;
	} = $props();

	// ─── State ─────────────────────────────────────────────
	let loading = $state(true);
	let searchQuery = $state('');
	let searchVisible = $state(false);

	// Font from settings
	const uiFont = $derived($appSettings.interfaceFont || 'system-ui, sans-serif');
	const isRTL = $derived($dir === 'rtl');

	// Tree data: Universe → Libraries → Folders → Notes
	type TreeNode = {
		name: string;
		path: string;
		isDir: boolean;
		isLibrary?: boolean;
		isRoot?: boolean;
		isCUniverse?: boolean;
		isNote?: boolean;
		libraryName?: string;
		libraryPath?: string;
		color?: string;
		status?: string;
		children?: TreeNode[];
		expanded?: boolean;
		noteCount?: number;
	};

	let rootNode: TreeNode | null = $state(null);

	// Drag state
	let dragSource: TreeNode | null = $state(null);
	let dropTarget: TreeNode | null = $state(null);
	let dragOverPath: string = $state('');

	// ─── Data Loading ──────────────────────────────────────
	async function loadData() {
		loading = true;
		const libs = $libraries;
		if (libs.length === 0) { loading = false; return; }

		// 1. Get child universes to identify which libraries belong to them
		let childUniverses: ChildUniverseInfo[] = [];
		try {
			childUniverses = await getChildUniverses();
		} catch { /* no child universes */ }

		// 2. For each child universe, read its libraries.json to get library paths
		const childLibPaths = new Map<string, Set<string>>(); // childUniversePath → set of library paths
		for (const cu of childUniverses) {
			try {
				const childLibs = await invoke<{ id: string; name: string; path: string }[]>(
					'read_child_universe_libraries', { childPath: cu.path }
				).catch(() => []);
				const paths = new Set(childLibs.map(l => normalizePath(l.path)));
				childLibPaths.set(cu.path, paths);
			} catch {
				childLibPaths.set(cu.path, new Set());
			}
		}

		// 3. Build library nodes
		async function buildLibNode(lib: typeof libs[0]): Promise<TreeNode> {
			const tree = await invoke<FileEntry[]>('read_library_tree', { path: lib.path, maxDepth: 20 }).catch(() => []);
			const libNode: TreeNode = {
				name: lib.name,
				path: lib.path,
				isDir: true,
				isLibrary: true,
				libraryName: lib.name,
				libraryPath: lib.path,
				color: libraryColorMap[lib.name] || '#7c3aed',
				children: buildTree(tree, lib.name, lib.path),
				expanded: false,
			};
			libNode.noteCount = countNotes(libNode);
			return libNode;
		}

		// 4. Separate own libraries from child universe libraries
		const ownLibs: TreeNode[] = [];
		const childLibNodes = new Map<string, TreeNode[]>(); // childUniversePath → library nodes

		for (const cu of childUniverses) {
			childLibNodes.set(cu.path, []);
		}

		for (const lib of libs) {
			const libPathNorm = normalizePath(lib.path);
			let assignedToChild = false;

			for (const cu of childUniverses) {
				const paths = childLibPaths.get(cu.path);
				if (paths && paths.has(libPathNorm)) {
					const node = await buildLibNode(lib);
					childLibNodes.get(cu.path)!.push(node);
					assignedToChild = true;
					break;
				}
			}

			if (!assignedToChild) {
				ownLibs.push(await buildLibNode(lib));
			}
		}

		// 5. Build root children: child universes first, then own libraries
		const rootChildren: TreeNode[] = [];

		for (const cu of childUniverses) {
			const cuLibs = childLibNodes.get(cu.path) || [];
			const cuNode: TreeNode = {
				name: cu.name,
				path: cu.path,
				isDir: true,
				isCUniverse: true,
				color: '#6366f1',
				children: cuLibs,
				expanded: false,
				noteCount: cuLibs.reduce((s, l) => s + (l.noteCount || 0), 0),
			};
			rootChildren.push(cuNode);
		}

		rootChildren.push(...ownLibs);

		rootNode = {
			name: universeName || $t('orgChart.universe') || 'Universe',
			path: '__root__',
			isDir: true,
			isRoot: true,
			children: rootChildren,
			expanded: true,
			noteCount: rootChildren.reduce((s, c) => s + (c.noteCount || 0), 0),
		};

		loading = false;
	}

	function normalizePath(p: string): string {
		return p.replace(/\\/g, '/').toLowerCase();
	}

	function buildTree(entries: FileEntry[], libraryName: string, libraryPath: string): TreeNode[] {
		return entries.map(e => {
			const node: TreeNode = {
				name: e.name.replace(/\.md$/, ''),
				path: e.path,
				isDir: e.is_dir,
				isNote: !e.is_dir,
				libraryName,
				libraryPath,
				color: libraryColorMap[libraryName] || '#7c3aed',
				status: e.status ?? undefined,
				children: e.is_dir && e.children ? buildTree(e.children, libraryName, libraryPath) : undefined,
				expanded: false,
			};
			if (e.is_dir) node.noteCount = countNotes(node);
			return node;
		});
	}

	function countNotes(node: TreeNode): number {
		if (!node.children) return 0;
		let n = 0;
		for (const c of node.children) {
			if (c.isNote) n++;
			else n += countNotes(c);
		}
		return n;
	}

	// ─── Toggle expand/collapse ────────────────────────────
	function toggleNode(node: TreeNode) {
		node.expanded = !node.expanded;
		// Force re-render by reassigning rootNode
		rootNode = { ...rootNode! };
	}

	// ─── Drag & Drop ──────────────────────────────────────
	function onDragStart(e: DragEvent, node: TreeNode) {
		if (node.isRoot) { e.preventDefault(); return; }
		dragSource = node;
		e.dataTransfer!.effectAllowed = 'move';
		e.dataTransfer!.setData('text/plain', node.path);
	}

	function onDragOver(e: DragEvent, node: TreeNode) {
		if (!dragSource || dragSource.path === node.path) return;
		if (!node.isDir && !node.isLibrary && !node.isRoot) return;
		// Don't allow dropping into self or own children
		if (isDescendant(dragSource, node)) return;
		e.preventDefault();
		e.dataTransfer!.dropEffect = 'move';
		dragOverPath = node.path;
	}

	function onDragLeave(e: DragEvent) {
		dragOverPath = '';
	}

	async function onDrop(e: DragEvent, targetNode: TreeNode) {
		e.preventDefault();
		dragOverPath = '';
		if (!dragSource || dragSource.path === targetNode.path) { dragSource = null; return; }
		if (!targetNode.isDir && !targetNode.isLibrary) { dragSource = null; return; }
		if (isDescendant(dragSource, targetNode)) { dragSource = null; return; }

		try {
			await invoke('move_item', {
				sourcePath: dragSource.path,
				targetFolder: targetNode.path,
			});
			// Reload tree after move
			await loadData();
		} catch (err: any) {
			console.error('[OrgChart] move failed:', err);
			alert(err?.toString() || 'Move failed');
		}
		dragSource = null;
	}

	function onDragEnd() {
		dragSource = null;
		dragOverPath = '';
	}

	function isDescendant(source: TreeNode, target: TreeNode): boolean {
		return target.path.startsWith(source.path + '\\') || target.path.startsWith(source.path + '/');
	}

	// ─── Search ────────────────────────────────────────────
	function matchesSearch(node: TreeNode): boolean {
		if (!searchQuery) return true;
		const q = searchQuery.toLowerCase();
		if (node.name.toLowerCase().includes(q)) return true;
		if (node.children) {
			return node.children.some(c => matchesSearch(c));
		}
		return false;
	}

	// ─── Selection sync ────────────────────────────────────
	// When selectedPath changes, expand ancestors and scroll into view
	function normPath(p: string): string { return p.replace(/\\/g, '/').toLowerCase(); }

	/** Check if node is exactly selected */
	function isExactSelected(node: TreeNode): boolean {
		if (!selectedPath) return false;
		const np = normPath(node.path);
		if (Array.isArray(selectedPath)) {
			return selectedPath.some(p => normPath(p) === np);
		}
		return np === normPath(selectedPath);
	}

	/** Check if node is a descendant of the selected path (parent selected) */
	function isGroupSelected(node: TreeNode): boolean {
		if (!selectedPath) return false;
		const np = normPath(node.path);
		if (Array.isArray(selectedPath)) {
			// Check if node is under any of the selected paths
			return selectedPath.some(p => {
				const sel = normPath(p);
				return np !== sel && np.startsWith(sel + '/');
			});
		}
		const sel = normPath(selectedPath);
		if (np === sel) return false;
		return np.startsWith(sel + '/');
	}

	/** Get the color for the selection group frame */
	function getSelectionColor(node: TreeNode): string {
		if (!selectedPath) return '';
		// Find the library color for this node
		if (node.color) return node.color;
		if (node.libraryName && libraryColorMap[node.libraryName]) return libraryColorMap[node.libraryName];
		return 'var(--interactive-accent)';
	}

	function expandToPath(node: TreeNode, targetPath: string): boolean {
		const tp = normPath(targetPath);
		if (normPath(node.path) === tp) return true;
		if (node.children) {
			for (const child of node.children) {
				if (expandToPath(child, targetPath)) {
					node.expanded = true;
					return true;
				}
			}
		}
		return false;
	}

	$effect(() => {
		if (selectedPath && rootNode) {
			const paths = Array.isArray(selectedPath) ? selectedPath : [selectedPath];
			for (const p of paths) expandToPath(rootNode, p);
			rootNode = rootNode; // trigger reactivity
			// Scroll the selected element into view after DOM update
			requestAnimationFrame(() => {
				const el = document.querySelector('.tree-node-selected');
				el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
			});
		}
	});

	// ─── Keyboard ──────────────────────────────────────────
	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
			e.preventDefault();
			searchVisible = !searchVisible;
			if (!searchVisible) searchQuery = '';
		}
		if (e.key === 'Escape') {
			if (searchVisible) { searchVisible = false; searchQuery = ''; }
			else onClose?.();
		}
	}

	// ─── Icons ─────────────────────────────────────────────
	// Using SVG icons to avoid emoji rendering issues

	// ─── Lifecycle ─────────────────────────────────────────
	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
		loadData();
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
	});
</script>

<div class="oc-container" style="font-family:{uiFont};" dir={isRTL ? 'rtl' : 'ltr'}>
	<!-- Header -->
	<div class="oc-header" class:oc-header-embedded={embedded}>
		{#if !embedded}
			<span class="oc-title">{$t('orgChart.title') || 'Sky View'}</span>
		{/if}
		<div class="oc-toolbar">
			{#if searchVisible}
				<input class="oc-search" type="text" dir="auto" placeholder={$t('layout.search') || 'Search...'} bind:value={searchQuery} autofocus />
			{/if}
			<button class="oc-btn" class:active={searchVisible} onclick={() => { searchVisible = !searchVisible; if (!searchVisible) searchQuery = ''; }} title="Search (Ctrl+F)">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
			</button>
			<button class="oc-btn" onclick={() => { if (rootNode) { function expandAll(n: TreeNode) { n.expanded = true; n.children?.forEach(expandAll); } expandAll(rootNode); rootNode = { ...rootNode }; } }} title="Expand all">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M3 12h18M3 18h18"/></svg>
			</button>
			<button class="oc-btn" onclick={() => { if (rootNode) { function collapseAll(n: TreeNode) { if (!n.isRoot) n.expanded = false; n.children?.forEach(collapseAll); } collapseAll(rootNode); rootNode = { ...rootNode }; } }} title="Collapse all">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h10M4 18h6"/></svg>
			</button>
			{#if !embedded}
				<button class="oc-close" onclick={onClose}>
					<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
				</button>
			{/if}
		</div>
	</div>

	<!-- Tree -->
	{#if loading}
		<div class="oc-loading">
			<div class="oc-spinner"></div>
			<span>Loading...</span>
		</div>
	{:else if rootNode}
		<div class="oc-tree-scroll">
			{#snippet treeItem(node: TreeNode, isLast: boolean, depth: number)}
				{@const isSearching = !!searchQuery}
				{@const visible = !isSearching || matchesSearch(node)}
				{#if visible}
					<div class="tree-row" class:tree-root={node.isRoot}>
						<!-- Connector lines -->
						{#if !node.isRoot}
							<span class="tree-connector">
								<span class="tree-vline" class:tree-vline-last={isLast}></span>
								<span class="tree-hline"></span>
							</span>
						{/if}

						<!-- Node content -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<span
							class="tree-node"
							class:tree-node-selected={isExactSelected(node)}
							class:tree-node-group-selected={isGroupSelected(node)}
							style:--selection-color={isExactSelected(node) || isGroupSelected(node) ? getSelectionColor(node) : ''}
							class:tree-drop-target={dragOverPath === node.path}
							class:tree-dragging={dragSource?.path === node.path}
							draggable={!node.isRoot}
							ondragstart={(e) => onDragStart(e, node)}
							ondragover={(e) => onDragOver(e, node)}
							ondragleave={onDragLeave}
							ondrop={(e) => onDrop(e, node)}
							ondragend={onDragEnd}
							onclick={() => {
								if (node.isDir || node.isLibrary || node.isRoot || node.isCUniverse) {
									toggleNode(node);
								} else if (node.isNote) {
									onNoteClick?.(node.path, node.name);
								}
								// Update selection for all node types (except root)
								if (!node.isRoot) {
									if (node.isCUniverse && node.children) {
										// Pass all child library paths for Star View highlighting
										selectedPath = node.children.map(c => c.path);
									} else {
										selectedPath = node.path;
									}
								}
							}}
						>
							<!-- Icon -->
							{#if node.isRoot}
								<svg class="tree-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
									<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><line x1="12" y1="2" x2="12" y2="22"/>
									<path d="M4.93 4.93a15.7 15.7 0 010 14.14"/><path d="M19.07 4.93a15.7 15.7 0 000 14.14"/>
								</svg>
							{:else if node.isCUniverse}
								<!-- cUniverse: small globe with orbit ring -->
								<svg class="tree-icon" style="color:#6366f1" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
									<circle cx="12" cy="12" r="6"/>
									<line x1="6" y1="12" x2="18" y2="12"/>
									<path d="M9.5 6.5a8.5 8.5 0 010 11"/><path d="M14.5 6.5a8.5 8.5 0 000 11"/>
									<ellipse cx="12" cy="12" rx="11" ry="3.5" transform="rotate(-25 12 12)" stroke-dasharray="2,2"/>
								</svg>
							{:else if node.isLibrary}
								<svg class="tree-icon tree-icon-lib" style="color:{node.color}" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
									<path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/>
									<line x1="8" y1="7" x2="16" y2="7"/><line x1="8" y1="11" x2="14" y2="11"/>
								</svg>
							{:else if node.isDir}
								{#if node.expanded}
									<svg class="tree-icon tree-icon-folder" width="16" height="16" viewBox="0 0 24 24" fill="#f59e0b" stroke="#d97706" stroke-width="1.5">
										<path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
									</svg>
								{:else}
									<svg class="tree-icon tree-icon-folder" width="16" height="16" viewBox="0 0 24 24" fill="#fbbf24" stroke="#d97706" stroke-width="1.5">
										<path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
									</svg>
								{/if}
							{:else}
								<svg class="tree-icon tree-icon-note" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
									<path d="M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z"/>
									<polyline points="14 2 14 8 20 8"/>
									<line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="13" y2="17"/>
								</svg>
							{/if}

							<!-- Label -->
							<span class="tree-label" dir="auto">{node.name}</span>

							<!-- Note count badge -->
							{#if (node.isDir || node.isLibrary || node.isRoot || node.isCUniverse) && node.noteCount}
								<span class="tree-badge" style:color={node.color || 'var(--text-faint)'}>{node.noteCount}</span>
							{/if}
						</span>
					</div>

					<!-- Children (indented) -->
					{#if node.expanded && node.children && node.children.length > 0}
						<div class="tree-children" class:tree-children-root={node.isRoot}>
							{#each node.children as child, idx (child.path)}
								{@render treeItem(child, idx === node.children.length - 1, depth + 1)}
							{/each}
						</div>
					{/if}
				{/if}
			{/snippet}

			{@render treeItem(rootNode, true, 0)}
		</div>
	{/if}

	<!-- Status bar -->
	{#if rootNode}
		<div class="oc-status">
			<span>{$libraries.length} {$t('orgChart.libraries') || 'libraries'}</span>
			<span class="oc-sep">·</span>
			<span>{rootNode.noteCount || 0} {$t('orgChart.notes') || 'notes'}</span>
		</div>
	{/if}
</div>

<style>
	.oc-container {
		position: relative; width: 100%; height: 100%; overflow: hidden;
		background: var(--background-primary); display: flex; flex-direction: column;
	}

	/* ─── Header ─── */
	.oc-header {
		display: flex; justify-content: space-between; align-items: center;
		padding: 8px 12px; border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary); flex-shrink: 0; z-index: 10;
	}
	.oc-header-embedded { padding: 4px 8px; }
	.oc-header-embedded .oc-toolbar { flex: 1; justify-content: flex-end; }
	.oc-title { font-size: 14px; font-weight: 600; color: var(--text-normal); }
	.oc-toolbar { display: flex; gap: 4px; align-items: center; }
	.oc-btn {
		display: flex; align-items: center; justify-content: center;
		width: 28px; height: 28px; border: none; border-radius: 5px;
		background: transparent; color: var(--text-muted); cursor: pointer;
	}
	.oc-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.oc-btn.active { background: var(--interactive-accent); color: white; }
	.oc-close {
		display: flex; align-items: center; justify-content: center;
		width: 28px; height: 28px; border: none; border-radius: 5px;
		background: transparent; color: var(--text-muted); cursor: pointer; margin-inline-start: 6px;
	}
	.oc-close:hover { background: #ef4444; color: white; }
	.oc-search {
		height: 26px; padding: 0 8px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; outline: none; min-width: 160px;
	}
	.oc-search:focus { border-color: var(--interactive-accent); }

	/* ─── Loading ─── */
	.oc-loading {
		flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
		gap: 10px; color: var(--text-muted);
	}
	.oc-spinner {
		width: 20px; height: 20px; border: 2px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent); border-radius: 50%;
		animation: ocspin 0.6s linear infinite;
	}
	@keyframes ocspin { to { transform: rotate(360deg); } }

	/* ─── Tree ─── */
	.oc-tree-scroll {
		flex: 1; overflow: auto; padding: 12px 16px;
	}

	.tree-row {
		display: flex; align-items: center;
		min-height: 28px;
		position: relative;
	}

	.tree-root {
		margin-bottom: 2px;
	}

	/* ─── Connector lines ─── */
	.tree-connector {
		display: flex; align-items: center;
		width: 20px; height: 28px;
		position: relative;
		flex-shrink: 0;
	}

	.tree-vline {
		position: absolute;
		inset-inline-start: 0;
		top: 0;
		width: 0;
		height: 100%;
		border-inline-start: 1px solid var(--background-modifier-border-hover, #555);
	}
	.tree-vline.tree-vline-last {
		height: 50%;
	}

	.tree-hline {
		position: absolute;
		inset-inline-start: 0;
		top: 50%;
		width: 100%;
		height: 0;
		border-top: 1px solid var(--background-modifier-border-hover, #555);
	}

	/* Children container — adds the continuing vertical line */
	.tree-children {
		padding-inline-start: 20px;
		position: relative;
	}

	/* Vertical connecting line along the left for non-root children */
	.tree-children:not(.tree-children-root)::before {
		content: '';
		position: absolute;
		inset-inline-start: 0;
		top: 0;
		bottom: 14px; /* Stop at the last child's connector midpoint */
		width: 0;
		border-inline-start: 1px solid var(--background-modifier-border-hover, #555);
	}

	/* ─── Node ─── */
	.tree-node {
		display: inline-flex; align-items: center; gap: 6px;
		padding: 3px 8px;
		border-radius: 4px;
		cursor: pointer;
		user-select: none;
		white-space: nowrap;
		transition: background 0.1s;
	}
	.tree-node:hover {
		background: var(--background-modifier-hover);
	}
	.tree-node.tree-node-selected {
		background: color-mix(in srgb, var(--selection-color, var(--interactive-accent)) 15%, transparent);
		outline: 2px solid var(--selection-color, var(--interactive-accent));
		outline-offset: -1px;
		border-radius: 4px;
	}
	.tree-node.tree-node-group-selected {
		background: color-mix(in srgb, var(--selection-color, var(--interactive-accent)) 10%, transparent);
		outline: 1px dashed var(--selection-color, var(--interactive-accent));
		outline-offset: -1px;
		border-radius: 4px;
	}
	.tree-node.tree-drop-target {
		background: color-mix(in srgb, var(--interactive-accent) 20%, transparent);
		outline: 2px dashed var(--interactive-accent);
		outline-offset: -2px;
	}
	.tree-node.tree-dragging {
		opacity: 0.4;
	}

	.tree-icon {
		flex-shrink: 0;
		color: var(--text-muted);
	}
	.tree-icon-lib { color: var(--interactive-accent); }
	.tree-icon-folder { /* color set by fill/stroke */ }
	.tree-icon-note { color: var(--text-faint); }

	.tree-label {
		font-size: 13px;
		color: var(--text-normal);
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.tree-badge {
		font-size: 10px;
		opacity: 0.6;
		margin-inline-start: 4px;
	}

	/* Root node styling */
	.tree-root .tree-node {
		font-weight: 600;
		font-size: 14px;
	}

	/* ─── Status bar ─── */
	.oc-status {
		display: flex; gap: 8px; align-items: center; padding: 6px 12px;
		background: var(--background-secondary); border-top: 1px solid var(--background-modifier-border);
		font-size: 11px; color: var(--text-faint); flex-shrink: 0;
	}
	.oc-sep { opacity: 0.3; }
</style>

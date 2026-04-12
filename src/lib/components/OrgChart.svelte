<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t, dir, getSearchOps } from '$lib/i18n';
	import { libraries, appSettings, type FileEntry, stripInvisibleChars, canonicalizeSearchQuery, hasAdvancedSyntaxMultilingual } from '$lib/libraries/store';
	import { getChildUniverses, type ChildUniverseInfo } from '$lib/universe/store';
	import { detectDir } from '$lib/utils';
	import { readSearchHistory, addSearchHistory } from '$lib/libraries/searchHistory';

	let {
		libraryColorMap = {} as Record<string, string>,
		universeName = '',
		embedded = false,
		fullscreen = false,
		selectedPath = $bindable(null as string | string[] | null),
		onNoteClick,
		onClose,
	}: {
		libraryColorMap?: Record<string, string>;
		universeName?: string;
		embedded?: boolean;
		fullscreen?: boolean;
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

	// ─── Fullscreen Mode (Card-Based) ──────────────────────
	interface MapNode {
		name: string;
		path: string;
		is_dir: boolean;
		node_type: string;
		weight: number;
		note_count: number;
		word_count: number;
		link_count: number;
		maturity: string | null;
		stratum: number | null;
		modified: number | null;
		children: MapNode[] | null;
	}

	let mapRoot = $state<MapNode | null>(null);
	let fsSearchQuery = $state('');
	let fsMaturityFilter = $state<Set<string>>(new Set());
	let fsExpandedPaths = $state<Set<string>>(new Set());

	// Pan & zoom state
	let canvasEl: HTMLDivElement | undefined;
	let innerEl: HTMLDivElement | undefined;
	let panX = $state(0);
	let panY = $state(0);
	let zoom = $state(1);
	let isPanning = $state(false);
	let userHasPanned = false;
	let hasAutoFit = false;
	let userHasZoomed = false; // true once user manually scrolls to zoom
	let panStartX = 0;
	let panStartY = 0;

	function onCanvasMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		if ((e.target as HTMLElement).closest('.oc-org-box')) return;
		isPanning = true;
		panStartX = e.clientX - panX;
		panStartY = e.clientY - panY;
		e.preventDefault();
	}
	function onCanvasMouseMove(e: MouseEvent) {
		if (!isPanning) return;
		userHasPanned = true;
		panX = e.clientX - panStartX;
		panY = e.clientY - panStartY;
	}
	function onCanvasMouseUp() { isPanning = false; }
	function onCanvasWheel(e: WheelEvent) {
		e.preventDefault();
		userHasZoomed = true;
		const delta = e.deltaY > 0 ? -0.08 : 0.08;
		zoom = Math.max(0.2, Math.min(3, zoom + delta));
	}

	// Context menu
	let ctxMenu = $state<{ x: number; y: number; node: MapNode } | null>(null);

	const MATURITY_COLORS: Record<string, string> = {
		seed: '#d1d5db', sapling: '#86efac', evergreen: '#16a34a',
		canonical: '#f59e0b', wilting: '#a3e635',
	};
	const ALL_MATURITIES = ['seed', 'sapling', 'evergreen', 'canonical', 'wilting'];

	async function loadFullscreenData() {
		loading = true;
		hasAutoFit = false;
		userHasPanned = false;
		userHasZoomed = false;
		try {
			mapRoot = await invoke<MapNode>('constellation_map_universe', {
				universeName: universeName || 'Universe',
				maxDepth: 20,
			});
			// Auto-expand the root
			if (mapRoot) {
				fsExpandedPaths = new Set([mapRoot.path]);
			}
		} catch (e) {
			console.error('[OrgChart] loadFullscreenData failed:', e);
		}
		loading = false;
	}

	function toggleFsExpand(path: string) {
		const s = new Set(fsExpandedPaths);
		if (s.has(path)) s.delete(path); else s.add(path);
		fsExpandedPaths = s;
		userHasPanned = false;
		userHasZoomed = false;
		// After DOM updates, re-fit width to accommodate new content, then center on node
		requestAnimationFrame(() => requestAnimationFrame(() => {
			if (innerEl) fitToWidth(innerEl);
			panX = 0; panY = 0;
			requestAnimationFrame(() => centerOnNode(path));
		}));
	}

	/** Reset: collapse all, clear search, re-fit to initial view. */
	function resetView() {
		if (!mapRoot) return;
		// Clear search — all state
		fsSearchQuery = '';
		searchMatches = [];
		searchMatchPaths = new Set();
		searchVisiblePaths = new Set();
		searchMatchIdx = 0;
		searchExecuted = false;
		// Clear filters
		fsMaturityFilter = new Set();
		// Collapse to root only
		fsExpandedPaths = new Set([mapRoot.path]);
		userHasPanned = false;
		userHasZoomed = false;
		panX = 0; panY = 0;
		requestAnimationFrame(() => requestAnimationFrame(() => {
			if (innerEl) fitToWidth(innerEl);
		}));
	}

	/** Center the canvas viewport on a specific node by data-path. */
	function centerOnNode(path: string) {
		if (!canvasEl) return;
		const el = canvasEl.querySelector(`[data-path="${CSS.escape(path)}"]`) as HTMLElement | null;
		const innerEl = canvasEl.querySelector('.oc-canvas-inner') as HTMLElement | null;
		if (!el || !innerEl) return;
		const innerRect = innerEl.getBoundingClientRect();
		const elRect = el.getBoundingClientRect();
		// Element center relative to inner element center (in screen coords)
		const innerCX = innerRect.left + innerRect.width / 2;
		const innerCY = innerRect.top + innerRect.height / 2;
		const elCX = elRect.left + elRect.width / 2;
		const elCY = elRect.top + elRect.height / 2;
		// Offset pan so clicked element moves to where the inner center currently is
		panX += (innerCX - elCX);
		panY += (innerCY - elCY);
	}


	// ─── Search (Enter-driven, proven OrgChart pattern) ───
	// Pattern: collect → expand ancestors → highlight → center (one at a time)
	let searchMatches = $state<MapNode[]>([]);  // ordered list of direct matches
	let searchMatchIdx = $state(0);
	let searchMatchPaths = $state<Set<string>>(new Set());  // O(1) highlight lookup
	let searchVisiblePaths = $state<Set<string>>(new Set()); // O(1) visibility lookup (matches + ancestors)
	let searchExecuted = $state(false);

	// Syntax chips + search enhancements
	let showChips = $state(false);
	let showSearchHistory = $state(false);
	let searchHistoryItems = $state<{ query: string; timestamp: number }[]>([]);
	let searchCategoryCounts = $state<Record<string, number>>({});

	const syntaxChips = $derived.by(() => {
		const _locale = $t('searchHub.linksTo');
		const ops = getSearchOps();
		return [
			{ label: 'linksTo', syntax: (ops?.linksTo ?? 'links to') + ' [[' },
			{ label: 'linksFrom', syntax: (ops?.linksFrom ?? 'links from') + ' [[' },
			{ label: 'mutual', syntax: (ops?.mutual ?? 'mutual') + ' [[' },
			{ label: 'mentions', syntax: (ops?.mentions ?? 'mentions') + ' [[' },
			{ label: 'orphans', syntax: ops?.orphans ?? 'orphans' },
			{ label: 'linksBetween', syntax: (ops?.linksBetween ?? 'links between') + ' [[' },
			{ label: 'linksAll', syntax: (ops?.linksAll ?? 'links all') + ' [[' },
			{ label: 'supports', syntax: (ops?.supports ?? 'supports') + ' [[' },
			{ label: 'contradicts', syntax: (ops?.contradicts ?? 'contradicts') + ' [[' },
			{ label: 'causes', syntax: (ops?.causes ?? 'causes') + ' [[' },
			{ label: 'exemplifies', syntax: (ops?.exemplifies ?? 'exemplifies') + ' [[' },
			{ label: 'generalizes', syntax: (ops?.generalizes ?? 'generalizes') + ' [[' },
			{ label: 'derivesFrom', syntax: (ops?.derivesFrom ?? 'derives from') + ' [[' },
			{ label: 'partOf', syntax: (ops?.partOf ?? 'part of') + ' [[' },
			{ label: 'tag', syntax: '#' },
			{ label: 'property', syntax: 'key=value' },
			{ label: 'scope', syntax: (ops?.scope ?? 'in') + ':' },
		];
	});

	function insertChipSyntax(syntax: string) {
		fsSearchQuery = fsSearchQuery ? fsSearchQuery + ' ' + syntax : syntax;
		showChips = false;
	}

	function selectSearchHistory(q: { query: string }) {
		fsSearchQuery = q.query;
		showSearchHistory = false;
		executeSearch();
	}

	/** O(1) visibility check — precomputed set, no tree walking on render. */
	function fsMatchesSearch(node: MapNode): boolean {
		if (searchVisiblePaths.size === 0) return true; // no active search = show all
		return searchVisiblePaths.has(node.path);
	}

	/** O(1) highlight check. */
	function isDirectMatch(node: MapNode): boolean {
		return searchMatchPaths.has(node.path);
	}

	/** Execute search on Enter: Rust full-text search + tree walk to find matches. */
	async function executeSearch() {
		if (!mapRoot || !fsSearchQuery.trim()) {
			searchMatches = []; searchMatchPaths = new Set();
			searchVisiblePaths = new Set(); searchMatchIdx = 0;
			searchExecuted = false;
			return;
		}
		searchExecuted = true;

		// 1. Call advanced search (supports #tags, property=value, links to [[]], etc.)
		let bodyMatchPaths = new Set<string>();
		try {
			const { constellationSearch, parseSearchQuery } = await import('$lib/libraries/store');
			const cleanQuery = stripInvisibleChars(fsSearchQuery);
			const canonicalized = canonicalizeSearchQuery(cleanQuery, getSearchOps());
			const req = parseSearchQuery(canonicalized);
			req.limit = 200;
			req.include_snippet = true;
			const results = await constellationSearch(req);
			// Track category counts
			const cats: Record<string, number> = {};
			for (const r of results) {
				bodyMatchPaths.add(r.path);
				cats[r.match_type] = (cats[r.match_type] || 0) + 1;
			}
			searchCategoryCounts = cats;
		} catch { /* fallback to title-only */ }
		// Save to search history
		addSearchHistory(fsSearchQuery);
		searchHistoryItems = readSearchHistory();

		// 2. Walk tree: mark nodes whose path is in bodyMatchPaths OR whose name matches
		const q = fsSearchQuery.toLowerCase();
		const matches: MapNode[] = [];
		const matchPaths = new Set<string>();
		const visiblePaths = new Set<string>();
		const expandPaths = new Set<string>();

		function walk(node: MapNode, ancestors: string[]): boolean {
			const nameMatch = node.name.toLowerCase().includes(q);
			const bodyMatch = bodyMatchPaths.has(node.path);
			const isMatch = (nameMatch || bodyMatch) && node.node_type === 'note';
			// Also match non-note nodes by name (libraries, folders)
			const isNameMatch = nameMatch && node.node_type !== 'note';
			let hasDescendantMatch = false;

			if (node.children) {
				for (const c of node.children) {
					if (walk(c, [...ancestors, node.path])) hasDescendantMatch = true;
				}
			}

			if (isMatch || isNameMatch) {
				matches.push(node);
				matchPaths.add(node.path);
				visiblePaths.add(node.path);
				for (const a of ancestors) { visiblePaths.add(a); expandPaths.add(a); }
			} else if (hasDescendantMatch) {
				visiblePaths.add(node.path);
			}

			return isMatch || isNameMatch || hasDescendantMatch;
		}
		walk(mapRoot, []);

		// 3. Batch state updates
		searchMatches = matches;
		searchMatchPaths = matchPaths;
		searchVisiblePaths = visiblePaths;
		searchMatchIdx = 0;

		// Expand ancestors of matches
		const merged = new Set(fsExpandedPaths);
		for (const p of expandPaths) merged.add(p);
		fsExpandedPaths = merged;

		// Notify SS of search update
		// Re-fit chart
		userHasPanned = false;
		userHasZoomed = false;
		requestAnimationFrame(() => requestAnimationFrame(() => {
			if (innerEl) fitToWidth(innerEl);
			panX = 0; panY = 0;
		}));
	}

	function goToMatch(idx: number) {
		if (searchMatches.length === 0) return;
		searchMatchIdx = idx;
		const match = searchMatches[idx];
		if (match) requestAnimationFrame(() => centerOnNode(match.path));
	}
	function nextMatch() { goToMatch((searchMatchIdx + 1) % searchMatches.length); }
	function prevMatch() { goToMatch((searchMatchIdx - 1 + searchMatches.length) % searchMatches.length); }

	function clearSearch() {
		fsSearchQuery = '';
		searchMatches = []; searchMatchPaths = new Set();
		searchVisiblePaths = new Set(); searchMatchIdx = 0;
		searchExecuted = false;
		searchCategoryCounts = {};
		// Collapse back to root + first-level children and fit to screen
		if (mapRoot) {
			fsExpandedPaths = new Set([mapRoot.path]);
			userHasPanned = false;
			userHasZoomed = false;
		}
	}

	function fsMatchesFilter(node: MapNode): boolean {
		if (fsMaturityFilter.size === 0) return true;
		if (node.node_type === 'note') return fsMaturityFilter.has(node.maturity || 'seed');
		if (node.children) return node.children.some(c => fsMatchesFilter(c));
		return true;
	}

	function toggleMaturityFilter(m: string) {
		const s = new Set(fsMaturityFilter);
		if (s.has(m)) s.delete(m); else s.add(m);
		fsMaturityFilter = s;
	}

	function formatDate(ts: number | null): string {
		if (!ts) return '';
		return new Date(ts * 1000).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function countMaturities(node: MapNode): Record<string, number> {
		const counts: Record<string, number> = {};
		function walk(n: MapNode) {
			if (n.node_type === 'note') {
				const m = n.maturity || 'seed';
				counts[m] = (counts[m] || 0) + 1;
			}
			if (n.children) n.children.forEach(walk);
		}
		walk(node);
		return counts;
	}

	/** Split an array into chunks of max size n. */
	function chunkArray<T>(arr: T[], n: number): T[][] {
		const chunks: T[][] = [];
		for (let i = 0; i < arr.length; i += n) chunks.push(arr.slice(i, i + n));
		return chunks;
	}

	function formatWordCount(wc: number): string {
		if (wc >= 1000) return (wc / 1000).toFixed(1) + 'k';
		return String(wc);
	}

	function handleContextMenu(e: MouseEvent, node: MapNode) {
		e.preventDefault();
		ctxMenu = { x: e.clientX, y: e.clientY, node };
	}

	function closeContextMenu() {
		ctxMenu = null;
	}

	async function handleCtxAction(action: string) {
		if (!ctxMenu) return;
		const node = ctxMenu.node;
		closeContextMenu();
		if (action === 'open') {
			onNoteClick?.(node.path, node.name + '.md');
		} else if (action === 'open-tab') {
			onNoteClick?.(node.path, node.name + '.md', undefined, undefined);
		}
	}

	// Load fullscreen data on mount if fullscreen mode
	$effect(() => {
		if (fullscreen && !mapRoot && !loading) loadFullscreenData();
	});

	/** Compute zoom to fit inner content to canvas width. */
	function fitToWidth(innerNode: HTMLElement) {
		if (!canvasEl) return;
		const canvasW = canvasEl.clientWidth;
		const innerW = innerNode.scrollWidth;
		if (canvasW > 0 && innerW > 0) {
			zoom = Math.min(3, Math.max(0.2, (canvasW * 0.92) / innerW));
		}
	}

	/** Fit both width AND height to fill the viewport. */
	function fitToScreen() {
		if (!canvasEl || !innerEl) return;
		const canvasW = canvasEl.clientWidth;
		const canvasH = canvasEl.clientHeight;
		const innerW = innerEl.scrollWidth;
		const innerH = innerEl.scrollHeight;
		if (canvasW > 0 && canvasH > 0 && innerW > 0 && innerH > 0) {
			const scaleW = (canvasW * 0.92) / innerW;
			const scaleH = (canvasH * 0.92) / innerH;
			zoom = Math.min(3, Math.max(0.2, Math.min(scaleW, scaleH)));
			panX = 0; panY = 0;
			userHasPanned = false;
			userHasZoomed = false;
		}
	}

	/** Svelte action: auto-fit on mount + re-fit on canvas resize. */
	function autoFitWidth(node: HTMLElement) {
		requestAnimationFrame(() => {
			fitToWidth(node);
			hasAutoFit = true;
		});
		// Re-fit when canvas resizes (window maximize/minimize) unless user manually zoomed
		const obs = new ResizeObserver(() => {
			if (!userHasZoomed && hasAutoFit) {
				fitToWidth(node);
				panX = 0; panY = 0; // reset pan to re-center via CSS
			}
		});
		if (canvasEl) obs.observe(canvasEl);
		return { destroy() { obs.disconnect(); } };
	}

	// Close context menu on click outside
	function handleGlobalClick() { if (ctxMenu) closeContextMenu(); }
</script>

{#if fullscreen}
<!-- ═══════ FULLSCREEN VISUAL ORG CHART ═══════ -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="oc-fs" style="font-family:{uiFont};" dir={isRTL ? 'rtl' : 'ltr'} onclick={handleGlobalClick}>
	<!-- Header bar -->
	<div class="oc-fs-header">
		<svg class="oc-fs-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="8" y="2" width="8" height="5" rx="1"/><rect x="1" y="17" width="8" height="5" rx="1"/><rect x="15" y="17" width="8" height="5" rx="1"/><path d="M12 7v4"/><path d="M5 17v-2h14v2"/></svg>
		<span class="oc-fs-title">{universeName || $t('orgChart.universe') || 'Universe'}</span>
		{#if mapRoot}
			<span class="oc-fs-stat">{mapRoot.note_count} {$t('secondScreen.dashboard.notes') || 'notes'} · {formatWordCount(mapRoot.word_count)} words</span>
		{/if}
		<div class="oc-fs-actions">
			<div class="oc-search-box">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
				<input type="text" dir="auto" placeholder={$t('layout.search') || 'Search... (Enter)'}
					bind:value={fsSearchQuery}
					oninput={() => { searchExecuted = false; searchMatches = []; searchMatchPaths = new Set(); searchVisiblePaths = new Set(); showSearchHistory = false; }}
					onfocus={() => { if (!fsSearchQuery) { searchHistoryItems = readSearchHistory(); showSearchHistory = true; } }}
					onblur={() => setTimeout(() => { showSearchHistory = false; }, 200)}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							e.preventDefault();
							if (!searchExecuted || searchMatches.length === 0) executeSearch();
							else e.shiftKey ? prevMatch() : nextMatch();
						}
						if (e.key === 'Escape') { clearSearch(); e.stopPropagation(); }
					}} />
				<button class="oc-search-clear" onclick={clearSearch}>×</button>
				<button class="oc-chips-btn" class:active={showChips} onclick={() => showChips = !showChips} title={$t('searchHub.syntaxHelpers')}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
				</button>
				<!-- Search history dropdown (inside search box for correct positioning) -->
				{#if showSearchHistory && searchHistoryItems.length > 0 && !fsSearchQuery}
					<div class="oc-search-history">
						{#each searchHistoryItems.slice(0, 8) as item}
							<button class="oc-history-item" onclick={() => selectSearchHistory(item)} dir="auto">{item.query}</button>
						{/each}
					</div>
				{/if}
				<!-- Syntax chips panel (inside search box for correct positioning) -->
				{#if showChips}
					<div class="oc-chips-panel">
						{#each syntaxChips as chip}
							<button class="oc-chip" onclick={() => insertChipSyntax(chip.syntax)}>{$t(`searchHub.${chip.label}`)}</button>
						{/each}
					</div>
				{/if}
			</div>
			{#if searchMatches.length > 0}
				<span class="oc-fs-match-count">{searchMatchIdx + 1}/{searchMatches.length}</span>
				{#if Object.keys(searchCategoryCounts).length > 0}
					<span class="oc-fs-cats">
						{#each Object.entries(searchCategoryCounts) as [type, count]}
							<span class="oc-fs-cat-badge" class:oc-cat-title={type==='title'} class:oc-cat-content={type==='content'||type==='structured'} class:oc-cat-tag={type==='tag'} class:oc-cat-wikilink={type==='wikilink'} class:oc-cat-property={type==='property'}>{type[0].toUpperCase()} {count}</span>
						{/each}
					</span>
				{/if}
				<button class="oc-fs-match-nav" onclick={prevMatch} title={$t('common.previous') || 'Previous'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
				</button>
				<button class="oc-fs-match-nav" onclick={nextMatch} title={$t('common.next') || 'Next'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 6 15 12 9 18"/></svg>
				</button>
				<button class="oc-fs-match-nav" onclick={clearSearch} title={$t('common.clear') || 'Clear'}>✕</button>
			{:else if searchExecuted && searchMatches.length === 0}
				<span class="oc-fs-match-count oc-fs-no-match">{$t('searchHub.noResults') || '0 matches'}</span>
			{/if}

			{#each ALL_MATURITIES as m}
				<button class="oc-fs-chip" class:active={fsMaturityFilter.has(m)} onclick={() => toggleMaturityFilter(m)}>
					<span class="oc-fs-chip-dot" style="background:{MATURITY_COLORS[m]}"></span>
					{m}
				</button>
			{/each}
			{#if fsMaturityFilter.size > 0}
				<button class="oc-fs-chip oc-fs-chip-clear" onclick={() => { fsMaturityFilter = new Set(); }}>✕</button>
			{/if}
		</div>
		<button class="oc-fs-reset" onclick={fitToScreen} title="Fit to screen">
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M21 8V5a2 2 0 0 0-2-2h-3"/><path d="M3 16v3a2 2 0 0 0 2 2h3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
		</button>
		<button class="oc-fs-reset" onclick={resetView} title={$t('orgChart.reset') || 'Reset view'}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
		</button>
		<button class="oc-fs-close" onclick={() => onClose?.()}>✕</button>
	</div>

	<!-- Pannable org chart canvas -->
	<div class="oc-canvas" bind:this={canvasEl}
		onmousedown={onCanvasMouseDown} onmousemove={onCanvasMouseMove} onmouseup={onCanvasMouseUp}
		onwheel={onCanvasWheel}>
		{#if loading}
			<div class="oc-fs-loading">{$t('layout.loading') || 'Loading...'}</div>
		{:else if mapRoot}
			<div class="oc-canvas-inner" role="tree" aria-label="Knowledge hierarchy" bind:this={innerEl} use:autoFitWidth style="transform: translate({panX}px, {panY}px) scale({zoom}); transform-origin: center center;">
				{#snippet orgNode(node: MapNode, depth: number)}
					{#if fsMatchesSearch(node) && fsMatchesFilter(node)}
						<div class="oc-org-group">
							<!-- The box -->
							<div class="oc-org-box" role="treeitem"
								data-path={node.path}
								class:oc-org-root={node.node_type === 'universe'}
								class:oc-org-lib={node.node_type === 'library' || node.node_type === 'child_universe'}
								class:oc-org-folder={node.node_type === 'folder'}
								class:oc-org-note={node.node_type === 'note'}
								class:oc-org-match={isDirectMatch(node)}
								class:oc-org-match-active={searchMatches.length > 0 && searchMatches[searchMatchIdx]?.path === node.path}
								style="--node-color: {node.node_type === 'library' ? (libraryColorMap[node.name] || '#7c3aed') : node.node_type === 'child_universe' ? '#6366f1' : node.node_type === 'note' ? (MATURITY_COLORS[node.maturity || 'seed']) : 'var(--interactive-accent)'}"
								onclick={(e) => {
									e.stopPropagation();
									if (node.node_type === 'note') { onNoteClick?.(node.path, node.name + '.md'); }
									else { toggleFsExpand(node.path); }
								}}
								oncontextmenu={(e) => handleContextMenu(e, node)}
								dir={detectDir(node.name)}
							>
								<div class="oc-org-box-name">{node.name}</div>
								{#if node.node_type !== 'note'}
									<div class="oc-org-box-meta">
										<span>{node.note_count} notes</span>
										{#if node.word_count > 0}<span>· {formatWordCount(node.word_count)}w</span>{/if}
									</div>
								{:else}
									<div class="oc-org-box-meta">
										{#if node.word_count > 0}<span>{formatWordCount(node.word_count)}w</span>{/if}
										{#if node.link_count > 0}<span class="oc-link-indicator">🔗{node.link_count}</span>{/if}
										{#if node.stratum && node.stratum > 1}<span class="oc-stratum-indicator">S{node.stratum}</span>{/if}
										{#if node.maturity}<span class="oc-org-maturity-dot" style="background:{MATURITY_COLORS[node.maturity]}"></span>{/if}
									</div>
								{/if}
							</div>
							<!-- Children -->
							{#if fsExpandedPaths.has(node.path) && node.children && node.children.length > 0 && node.node_type !== 'note'}
								{@const visibleChildren = node.children.filter(c => fsMatchesSearch(c) && fsMatchesFilter(c))}
								{@const dirChildren = visibleChildren.filter(c => c.node_type !== 'note')}
								{@const noteChildren = visibleChildren.filter(c => c.node_type === 'note')}

								<!-- Note children: render as a vertical list (not chart boxes) -->
								{#if noteChildren.length > 0}
									<div class="oc-org-connector"><div class="oc-org-vline"></div></div>
									<div class="oc-org-notelist">
										{#each noteChildren as note (note.path)}
											<button class="oc-org-noterow" aria-label="{note.name}"
												class:oc-org-match={isDirectMatch(note)}
												dir={detectDir(note.name)}
												onclick={() => onNoteClick?.(note.path, note.name + '.md')}
												oncontextmenu={(e) => handleContextMenu(e, note)}>
												<span class="oc-org-noterow-dot" style="background:{MATURITY_COLORS[note.maturity || 'seed']}"></span>
												<span class="oc-org-noterow-name">{note.name}</span>
																	{#if note.link_count > 0}<span class="oc-link-indicator-sm">🔗{note.link_count}</span>{/if}
												{#if note.word_count > 0}<span class="oc-org-noterow-meta">{formatWordCount(note.word_count)}w</span>{/if}
												{#if note.modified}<span class="oc-org-noterow-meta">{formatDate(note.modified)}</span>{/if}
											</button>
										{/each}
									</div>
								{/if}

								<!-- Directory children: render as chart boxes with connectors (max 5 per row) -->
								{#if dirChildren.length > 0}
									{@const rows = chunkArray(dirChildren, 5)}
									{#each rows as row, rowIdx (rowIdx)}
										<div class="oc-org-connector">
											<div class="oc-org-vline"></div>
											<div class="oc-org-hline-row">
												{#each row as child, i (child.path)}
													<div class="oc-org-hline-seg" class:first={i === 0} class:last={i === row.length - 1} class:single={row.length === 1}></div>
												{/each}
											</div>
										</div>
										<div class="oc-org-children" style="--child-count:{row.length}">
											{#each row as child (child.path)}
												<div class="oc-org-child-wrap">
													<div class="oc-org-child-vline"></div>
													{@render orgNode(child, depth + 1)}
												</div>
											{/each}
										</div>
									{/each}
								{/if}
							{/if}
						</div>
					{/if}
				{/snippet}

				{@render orgNode(mapRoot, 0)}
			</div>
		{/if}
	</div>

	<!-- Context menu -->
	{#if ctxMenu}
		<div class="oc-fs-ctx" style="left:{ctxMenu.x}px; top:{ctxMenu.y}px">
			{#if ctxMenu.node.node_type === 'note'}
				<button onclick={() => handleCtxAction('open')}>{$t('contextMenu.open') || 'Open'}</button>
			{:else}
				<button onclick={() => toggleFsExpand(ctxMenu.node.path)}>{fsExpandedPaths.has(ctxMenu.node.path) ? ($t('sidebar.collapseAll') || 'Collapse') : ($t('sidebar.expandAll') || 'Expand')}</button>
			{/if}
		</div>
	{/if}
</div>

{:else}
<!-- ═══════ SIDEBAR VIEW (Original) ═══════ -->
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
			<span>{$t('layout.loading') || 'Loading...'}</span>
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
{/if}

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

	/* ═══════════════════════════════════════════ */
	/* Fullscreen visual org chart                */
	/* ═══════════════════════════════════════════ */

	.oc-fs {
		width: 100%; height: 100%; display: flex; flex-direction: column;
		background: var(--background-primary); overflow: hidden;
	}
	.oc-fs-header {
		display: flex; align-items: center; gap: 10px; padding: 10px 16px;
		border-bottom: 1px solid var(--background-modifier-border); flex-shrink: 0;
	}
	.oc-fs-icon { color: var(--text-muted); flex-shrink: 0; }
	.oc-fs-title { font-size: 16px; font-weight: 700; color: var(--text-normal); }
	.oc-fs-stat { font-size: 12px; color: var(--text-muted); }
	.oc-fs-actions { display: flex; align-items: center; gap: 6px; margin-inline-start: auto; flex-wrap: wrap; }
	.oc-search-box {
		display: flex; align-items: center; gap: 4px;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 0 6px;
		position: relative;
		min-width: 350px; flex: 1; max-width: 600px;
	}
	.oc-search-box:focus-within { border-color: var(--interactive-accent); }
	.oc-search-box svg { color: var(--text-muted); flex-shrink: 0; }
	.oc-search-box input {
		flex: 1; border: none; background: none; padding: 4px 0;
		font-size: 12px; color: var(--text-normal); font-family: inherit;
		outline: none; min-width: 140px;
	}
	.oc-search-box input::placeholder { color: var(--text-faint); }
	.oc-search-clear { border: none; background: none; color: var(--text-muted); cursor: pointer; font-size: 1rem; padding: 0 2px; }
	.oc-chips-btn { border: none; background: none; cursor: pointer; color: var(--text-muted); padding: 2px; border-radius: 3px; }
	.oc-chips-btn:hover, .oc-chips-btn.active { color: var(--interactive-accent); background: var(--background-modifier-hover); }
	.oc-chips-panel { position: absolute; top: 100%; inset-inline-start: 0; z-index: 100; display: flex; flex-wrap: wrap; gap: 4px; padding: 6px; background: var(--background-primary); border: 1px solid var(--background-modifier-border); border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12); max-width: 500px; margin-top: 4px; }
	.oc-chip { padding: 2px 8px; border-radius: 12px; border: 1px solid var(--background-modifier-border); background: var(--background-primary); color: var(--text-muted); font-size: 0.7rem; cursor: pointer; white-space: nowrap; }
	.oc-chip:hover { border-color: var(--interactive-accent); color: var(--interactive-accent); }
	.oc-search-history { position: absolute; top: 100%; inset-inline-start: 0; z-index: 100; background: var(--background-primary); border: 1px solid var(--background-modifier-border); border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12); min-width: 200px; margin-top: 4px; max-height: 200px; overflow-y: auto; }
	.oc-history-item { display: block; width: 100%; text-align: start; padding: 6px 12px; border: none; background: none; cursor: pointer; font-size: 0.78rem; color: var(--text-normal); }
	.oc-history-item:hover { background: var(--background-modifier-hover); }
	.oc-fs-cats { display: flex; gap: 4px; align-items: center; }
	.oc-fs-cat-badge { font-size: 0.62rem; padding: 1px 5px; border-radius: 4px; font-weight: 600; color: white; }
	.oc-cat-title { background: #3b82f6; }
	.oc-cat-content { background: #16a34a; }
	.oc-cat-tag { background: #f472b6; }
	.oc-cat-wikilink { background: #60a5fa; }
	.oc-cat-property { background: #f59e0b; }
	/* Living Link indicators on tree nodes */
	.oc-link-indicator { font-size: 0.6rem; color: var(--interactive-accent); opacity: 0.8; }
	.oc-stratum-indicator { font-size: 0.55rem; background: var(--interactive-accent); color: white; padding: 0 3px; border-radius: 3px; font-weight: 600; }
	.oc-link-indicator-sm { font-size: 0.55rem; color: var(--text-faint); margin-inline-start: 4px; }
	.oc-fs-match-count { font-size: 11px; color: var(--text-muted); white-space: nowrap; }
	.oc-fs-no-match { color: #ef4444; }
	.oc-fs-match-nav {
		width: 22px; height: 22px; border: none; border-radius: 4px;
		background: none; color: var(--text-muted); cursor: pointer;
		display: flex; align-items: center; justify-content: center; padding: 0;
	}
	.oc-fs-match-nav:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.oc-fs-chip {
		display: flex; align-items: center; gap: 3px; padding: 2px 8px;
		border-radius: 10px; font-size: 11px; cursor: pointer;
		border: 1px solid var(--background-modifier-border);
		background: var(--background-primary); color: var(--text-muted);
		font-family: inherit; text-transform: capitalize;
	}
	.oc-fs-chip:hover { background: var(--background-modifier-hover); }
	.oc-fs-chip.active { background: var(--interactive-accent); color: white; border-color: var(--interactive-accent); }
	.oc-fs-chip-dot { width: 6px; height: 6px; border-radius: 50%; }
	.oc-fs-chip-clear { font-size: 11px; padding: 2px 6px; }
	.oc-fs-reset, .oc-fs-close {
		width: 28px; height: 28px; border: none; border-radius: 6px;
		background: none; color: var(--text-muted); cursor: pointer; font-size: 16px;
		display: flex; align-items: center; justify-content: center;
	}
	.oc-fs-reset:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.oc-fs-close:hover { background: #ef4444; color: white; }
	.oc-fs-loading { color: var(--text-muted); font-size: 14px; padding: 40px; text-align: center; }

	/* Pannable canvas — CSS centers content; JS pan offsets from center */
	.oc-canvas {
		flex: 1; overflow: hidden; cursor: grab; position: relative;
		display: flex; align-items: center; justify-content: center;
	}
	.oc-canvas:active { cursor: grabbing; }
	.oc-canvas-inner {
		padding: 40px; flex-shrink: 0;
	}

	/* ─── Org chart boxes and connectors ─── */
	.oc-org-group {
		display: flex; flex-direction: column; align-items: center;
	}

	/* The box itself */
	.oc-org-box {
		border: 2px solid var(--node-color, var(--background-modifier-border));
		border-radius: 8px; padding: 8px 16px; cursor: pointer;
		background: var(--background-primary);
		min-width: 100px; max-width: 200px; text-align: center;
		transition: box-shadow 0.15s, transform 0.1s;
		box-shadow: 0 1px 3px rgba(0,0,0,0.08);
		user-select: none;
	}
	.oc-org-box:hover {
		box-shadow: 0 3px 10px rgba(0,0,0,0.15);
		transform: translateY(-1px);
	}
	/* Search match highlight */
	.oc-org-match {
		outline: 2px solid var(--interactive-accent);
		outline-offset: 2px;
		background: color-mix(in srgb, var(--interactive-accent) 8%, var(--background-primary)) !important;
	}
	.oc-org-match-active {
		outline: 3px solid var(--interactive-accent);
		outline-offset: 2px;
		box-shadow: 0 0 12px color-mix(in srgb, var(--interactive-accent) 40%, transparent) !important;
	}
	.oc-org-root {
		background: var(--interactive-accent); color: white;
		border-color: var(--interactive-accent);
		min-width: 140px; font-weight: 700;
	}
	.oc-org-root .oc-org-box-meta { color: rgba(255,255,255,0.8); }
	.oc-org-lib {
		border-width: 2px;
		background: color-mix(in srgb, var(--node-color) 8%, var(--background-primary));
	}
	.oc-org-folder {
		border-width: 1px; border-style: dashed;
		background: var(--background-secondary);
	}
	.oc-org-note {
		border-width: 1px; min-width: 80px; max-width: 160px;
		padding: 5px 10px; font-size: 12px;
	}
	.oc-org-box-name {
		font-size: 13px; font-weight: 600; color: var(--text-normal);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.oc-org-root .oc-org-box-name { color: white; font-size: 14px; }
	.oc-org-note .oc-org-box-name { font-weight: 500; font-size: 12px; }
	.oc-org-box-meta {
		font-size: 10px; color: var(--text-muted); margin-top: 2px;
		display: flex; align-items: center; justify-content: center; gap: 4px;
	}
	.oc-org-maturity-dot { width: 6px; height: 6px; border-radius: 50%; }

	/* ─── Connector lines ─── */
	.oc-org-connector {
		display: flex; flex-direction: column; align-items: center;
	}
	/* Vertical line from parent box down */
	.oc-org-vline {
		width: 1px; height: 16px;
		background: var(--background-modifier-border);
	}
	/* Horizontal line row — hidden, connectors drawn on child-wraps */
	.oc-org-hline-row { display: none; }
	.oc-org-hline-seg { display: none; }

	/* Children row */
	.oc-org-children {
		display: flex; align-items: flex-start;
	}

	/* Each child-wrap: vertical drop line + horizontal connector via ::before/::after.
	   Proven org chart technique from Envato Tuts+ / CodeHim:
	   - ::before draws left half of horizontal line (border-top from left edge to center)
	   - ::after draws right half (border-top from center to right edge)
	   - First child: no ::before (no line to the left)
	   - Last child: no ::after (no line to the right)
	   - Single child: neither (just vertical line) */
	.oc-org-child-wrap {
		display: flex; flex-direction: column; align-items: center;
		position: relative;
		padding: 0 8px; /* spacing between children */
	}
	/* Left half of horizontal line */
	.oc-org-child-wrap::before {
		content: ''; position: absolute; top: 0;
		width: 50%; height: 0;
		border-top: 1px solid var(--background-modifier-border);
		inset-inline-start: 0;
	}
	/* Right half of horizontal line */
	.oc-org-child-wrap::after {
		content: ''; position: absolute; top: 0;
		width: 50%; height: 0;
		border-top: 1px solid var(--background-modifier-border);
		inset-inline-end: 0;
	}
	/* First child: no line to the left */
	.oc-org-child-wrap:first-child::before { display: none; }
	/* Last child: no line to the right */
	.oc-org-child-wrap:last-child::after { display: none; }
	/* Single child: no horizontal line at all */
	.oc-org-child-wrap:only-child::before,
	.oc-org-child-wrap:only-child::after { display: none; }

	/* Vertical line from horizontal connector down to child box */
	.oc-org-child-vline {
		width: 1px; height: 16px;
		background: var(--background-modifier-border);
	}

	/* ─── Note list (under parent boxes) ─── */
	.oc-org-notelist {
		display: flex; flex-direction: column; align-items: stretch;
		background: var(--background-secondary); border-radius: 6px;
		border: 1px solid var(--background-modifier-border);
		min-width: 200px; max-width: 400px; max-height: 300px; overflow-y: auto;
	}
	.oc-org-noterow {
		display: flex; align-items: center; gap: 8px;
		padding: 5px 12px; width: 100%;
		border: none; border-bottom: 1px solid var(--background-modifier-border);
		background: none; cursor: pointer;
		font-family: inherit; font-size: 12px; color: var(--text-normal);
		text-align: start; transition: background 0.1s;
	}
	.oc-org-noterow:last-child { border-bottom: none; }
	.oc-org-noterow:hover { background: var(--background-modifier-hover); }
	.oc-org-noterow.oc-org-match { background: color-mix(in srgb, var(--interactive-accent) 10%, transparent); }
	.oc-org-noterow-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
	.oc-org-noterow-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.oc-org-noterow-meta { font-size: 10px; color: var(--text-faint); flex-shrink: 0; }

	/* ─── Search results (VS Code grouped list pattern) ─── */
	.oc-search-results {
		flex: 1; overflow-y: auto; padding: 16px 24px;
	}
	.oc-sr-group {
		margin-bottom: 16px;
	}
	.oc-sr-group-header {
		display: flex; align-items: center; gap: 8px;
		padding: 8px 12px; font-size: 14px; font-weight: 700;
		color: var(--text-normal);
		background: var(--background-secondary); border-radius: 6px;
		margin-bottom: 4px;
	}
	.oc-sr-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
	.oc-sr-group-name { flex: 1; }
	.oc-sr-group-count {
		font-size: 11px; font-weight: 400; color: var(--text-muted);
		background: var(--background-modifier-border); padding: 1px 6px; border-radius: 8px;
	}
	.oc-sr-folder {
		display: flex; align-items: center; gap: 6px;
		padding: 4px 12px 4px 28px; font-size: 12px; font-weight: 600;
		color: var(--text-muted);
	}
	.oc-sr-note {
		display: flex; align-items: center; gap: 8px;
		padding: 5px 12px 5px 44px; width: 100%;
		border: none; border-radius: 4px; background: none; cursor: pointer;
		font-family: inherit; font-size: 13px; color: var(--text-normal);
		text-align: start; transition: background 0.1s;
	}
	.oc-sr-note:hover { background: var(--background-modifier-hover); }
	.oc-sr-note-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
	.oc-sr-note-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.oc-sr-note-meta { font-size: 11px; color: var(--text-faint); flex-shrink: 0; }

	/* Context menu */
	.oc-fs-ctx {
		position: fixed; z-index: 9999; min-width: 140px;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 6px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); padding: 4px;
	}
	.oc-fs-ctx button {
		display: block; width: 100%; text-align: start; padding: 6px 12px;
		border: none; border-radius: 4px; background: none; cursor: pointer;
		font-size: 13px; color: var(--text-normal); font-family: inherit;
	}
	.oc-fs-ctx button:hover { background: var(--background-modifier-hover); }
</style>

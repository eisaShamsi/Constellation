<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import { libraries, type FileEntry } from '$lib/libraries/store';
	import * as d3 from 'd3';

	let {
		libraryColorMap = {} as Record<string, string>,
		onNoteClick,
		onClose,
	}: {
		libraryColorMap?: Record<string, string>;
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
	} = $props();

	// ─── State ─────────────────────────────────────────────
	let svgEl: SVGSVGElement;
	let containerEl: HTMLDivElement;
	let loading = $state(true);
	let searchQuery = $state('');
	let searchVisible = $state(false);
	let hierarchySource = $state<'folders' | 'tags' | 'moc' | 'parent'>('folders');
	let layoutMode = $state<'tree' | 'radial' | 'sunburst' | 'treemap'>('tree');
	let treeOrientation = $state<'vertical' | 'horizontal'>('vertical');
	let colorMode = $state<'library' | 'status' | 'depth'>('library');

	// Breadcrumb drill-down
	let breadcrumb: { name: string; path: string }[] = $state([]);
	let drillRootPath: string | null = $state(null); // null = show full tree

	// D3 data
	let rootData: any = null;
	let d3Root: d3.HierarchyNode<any> | null = null;
	let treeNodes: { x: number; y: number; data: any; depth: number; children?: any[]; _collapsed?: boolean }[] = $state([]);
	let treeLinks: { source: { x: number; y: number }; target: { x: number; y: number } }[] = $state([]);

	// Pan/Zoom
	let transform = $state({ x: 0, y: 0, k: 1 });
	let zoomBehavior: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;

	// Stats
	let totalNodes = $state(0);
	let totalFolders = $state(0);
	let maxDepth = $state(0);

	// Collapsed nodes tracking
	let collapsedPaths = $state(new Set<string>());

	// Sunburst/Treemap data
	let sunburstArcs: { x0: number; x1: number; y0: number; y1: number; data: any; depth: number }[] = $state([]);
	let treemapRects: { x0: number; y0: number; x1: number; y1: number; data: any; depth: number }[] = $state([]);

	// Hover
	let hoveredNode: any = $state(null);

	// Search matches
	const searchMatches = $derived.by(() => {
		if (!searchQuery || !d3Root) return new Set<string>();
		const q = searchQuery.toLowerCase();
		const matches = new Set<string>();
		d3Root.each((d: any) => {
			if (d.data.name?.toLowerCase().includes(q)) {
				matches.add(d.data.path || d.data.name);
				// Also highlight ancestors
				let p = d.parent;
				while (p) {
					matches.add(p.data.path || p.data.name);
					p = p.parent;
				}
			}
		});
		return matches;
	});

	// ─── Data Loading ──────────────────────────────────────
	async function loadFolderHierarchy() {
		const libs = $libraries;
		if (libs.length === 0) return null;

		const children: any[] = [];
		for (const lib of libs) {
			const tree = await invoke<FileEntry[]>('read_library_tree', { libraryPath: lib.path, maxDepth: 20 }).catch(() => []);
			children.push({
				name: lib.name,
				path: lib.path,
				isDir: true,
				isLibrary: true,
				color: libraryColorMap[lib.name] || '#7c3aed',
				children: flattenTree(tree, lib.name),
			});
		}

		return {
			name: 'Universe',
			path: '__root__',
			isDir: true,
			isRoot: true,
			children,
		};
	}

	function flattenTree(entries: FileEntry[], libraryName: string): any[] {
		return entries.map(e => ({
			name: e.name.replace(/\.md$/, ''),
			path: e.path,
			isDir: e.is_dir,
			isNote: !e.is_dir,
			libraryName,
			color: libraryColorMap[libraryName] || '#7c3aed',
			status: e.status,
			children: e.is_dir && e.children ? flattenTree(e.children, libraryName) : undefined,
		}));
	}

	async function loadTagHierarchy() {
		const libs = $libraries;
		const allTags: Record<string, number> = {};
		for (const lib of libs) {
			const tags = await invoke<Record<string, number>>('scan_library_tags', { libraryPath: lib.path }).catch(() => ({}));
			for (const [tag, count] of Object.entries(tags)) {
				allTags[tag] = (allTags[tag] || 0) + count;
			}
		}

		// Build tag tree
		function buildTagTree(tags: Record<string, number>): any[] {
			const roots: any[] = [];
			const nodeMap = new Map<string, any>();
			const sorted = Object.entries(tags).sort((a, b) => a[0].localeCompare(b[0]));
			for (const [tag, count] of sorted) {
				const parts = tag.split('/');
				let path = '';
				let parent = roots;
				for (let i = 0; i < parts.length; i++) {
					path += (i > 0 ? '/' : '') + parts[i];
					let existing = nodeMap.get(path);
					if (!existing) {
						existing = { name: '#' + parts[i], path: 'tag:' + path, isDir: true, isTag: true, color: '#7c3aed', children: [] };
						nodeMap.set(path, existing);
						parent.push(existing);
					}
					if (i === parts.length - 1) existing._count = count;
					parent = existing.children;
				}
			}
			return roots;
		}

		return {
			name: 'Tags',
			path: '__tags_root__',
			isDir: true,
			isRoot: true,
			children: buildTagTree(allTags),
		};
	}

	async function loadMOCHierarchy() {
		const libs = $libraries;
		const allLinks: { source: string; target: string; sourceName: string }[] = [];
		const noteNames = new Map<string, string>();

		for (const lib of libs) {
			const links = await invoke<any[]>('scan_library_links', { libraryPath: lib.path }).catch(() => []);
			for (const l of links) {
				allLinks.push({ source: l.source_name, target: l.target, sourceName: l.source_name });
				noteNames.set(l.source_name, l.source_path);
			}
		}

		// Find MOC notes (5+ outgoing links)
		const outgoing = new Map<string, Set<string>>();
		for (const l of allLinks) {
			if (!outgoing.has(l.source)) outgoing.set(l.source, new Set());
			outgoing.get(l.source)!.add(l.target);
		}

		const mocs: any[] = [];
		for (const [name, targets] of outgoing) {
			if (targets.size >= 5) {
				mocs.push({
					name,
					path: noteNames.get(name) || name,
					isDir: true,
					isMOC: true,
					color: '#d97706',
					children: [...targets].map(t => ({
						name: t,
						path: t,
						isNote: true,
						color: '#64748b',
					})),
				});
			}
		}

		return {
			name: 'Maps of Content',
			path: '__moc_root__',
			isDir: true,
			isRoot: true,
			children: mocs.sort((a, b) => (b.children?.length || 0) - (a.children?.length || 0)),
		};
	}

	// ─── Layout Computation ────────────────────────────────
	function computeLayout() {
		if (!rootData) return;

		// Filter out collapsed nodes
		function filterCollapsed(node: any): any {
			if (!node.children) return { ...node };
			if (collapsedPaths.has(node.path)) {
				return { ...node, children: undefined, _collapsed: true, _childCount: countDescendants(node) };
			}
			return {
				...node,
				_collapsed: false,
				children: node.children.map(filterCollapsed).filter(Boolean),
			};
		}

		// Find drill-down subtree if active
		let sourceData = rootData;
		if (drillRootPath) {
			const findNode = (node: any, path: string): any => {
				if (node.path === path) return node;
				if (!node.children) return null;
				for (const c of node.children) {
					const found = findNode(c, path);
					if (found) return found;
				}
				return null;
			};
			sourceData = findNode(rootData, drillRootPath) || rootData;
		}

		const filtered = filterCollapsed(sourceData);
		d3Root = d3.hierarchy(filtered);

		// Compute layout
		const nodeCount = d3Root.descendants().length;
		const nodeSize: [number, number] = treeOrientation === 'vertical' ? [180, 80] : [40, 220];

		// Clear all layout data
		sunburstArcs = [];
		treemapRects = [];

		if (layoutMode === 'sunburst') {
			// Sunburst: partition layout in polar coordinates
			d3Root.sum((d: any) => d.children ? 0 : 1);
			const partition = d3.partition().size([2 * Math.PI, 300]);
			partition(d3Root as any);
			sunburstArcs = d3Root.descendants().filter((d: any) => d.depth > 0).map((d: any) => ({
				x0: d.x0, x1: d.x1, y0: d.y0, y1: d.y1,
				data: d.data, depth: d.depth,
			}));
			treeNodes = [];
			treeLinks = [];
		} else if (layoutMode === 'treemap') {
			// Treemap: rectangle packing
			const w = containerEl?.clientWidth || 800;
			const h = containerEl?.clientHeight || 600;
			d3Root.sum((d: any) => d.children ? 0 : 1);
			const treemap = d3.treemap().size([w - 40, h - 100]).padding(2).round(true);
			treemap(d3Root as any);
			treemapRects = d3Root.leaves().map((d: any) => ({
				x0: d.x0, y0: d.y0, x1: d.x1, y1: d.y1,
				data: d.data, depth: d.depth,
			}));
			treeNodes = [];
			treeLinks = [];
		} else if (layoutMode === 'tree') {
			const treeLayout = d3.tree().nodeSize(nodeSize);
			treeLayout(d3Root as any);
		} else {
			// Radial
			const treeLayout = d3.tree().size([2 * Math.PI, Math.min(400, nodeCount * 8)]);
			treeLayout(d3Root as any);
		}

		if (layoutMode === 'tree' || layoutMode === 'radial') {
			// Extract nodes and links for tree/radial
			const nodes = d3Root.descendants().map((d: any) => ({
				x: layoutMode === 'radial' ? d.y * Math.cos(d.x - Math.PI / 2) : (treeOrientation === 'vertical' ? d.x : d.y),
				y: layoutMode === 'radial' ? d.y * Math.sin(d.x - Math.PI / 2) : (treeOrientation === 'vertical' ? d.y : d.x),
				data: d.data,
				depth: d.depth,
				children: d.children,
				_collapsed: d.data._collapsed,
			}));

			const links = d3Root.links().map((l: any) => ({
				source: {
					x: layoutMode === 'radial' ? l.source.y * Math.cos(l.source.x - Math.PI / 2) : (treeOrientation === 'vertical' ? l.source.x : l.source.y),
					y: layoutMode === 'radial' ? l.source.y * Math.sin(l.source.x - Math.PI / 2) : (treeOrientation === 'vertical' ? l.source.y : l.source.x),
				},
				target: {
					x: layoutMode === 'radial' ? l.target.y * Math.cos(l.target.x - Math.PI / 2) : (treeOrientation === 'vertical' ? l.target.x : l.target.y),
					y: layoutMode === 'radial' ? l.target.y * Math.sin(l.target.x - Math.PI / 2) : (treeOrientation === 'vertical' ? l.target.y : l.target.x),
				},
			}));

			treeNodes = nodes;
			treeLinks = links;
		}

		// Stats
		totalNodes = d3Root.descendants().length;
		totalFolders = d3Root.descendants().filter((d: any) => d.data.isDir).length;
		maxDepth = d3Root.height;
	}

	function countDescendants(node: any): number {
		if (!node.children) return 0;
		let count = node.children.length;
		for (const c of node.children) count += countDescendants(c);
		return count;
	}

	// ─── Interactions ──────────────────────────────────────
	function toggleCollapse(path: string) {
		const next = new Set(collapsedPaths);
		if (next.has(path)) next.delete(path); else next.add(path);
		collapsedPaths = next;
		computeLayout();
	}

	function handleNodeClick(node: any) {
		if (node.data.isDir || node.data.isRoot || node.data.isLibrary) {
			toggleCollapse(node.data.path);
		} else if (node.data.isNote) {
			onNoteClick?.(node.data.path, node.data.name);
		}
	}

	function handleNodeDoubleClick(node: any) {
		// Drill down: re-root the chart at this node
		if (node.data.isDir || node.data.isLibrary) {
			drillRootPath = node.data.path;
			breadcrumb = [...breadcrumb, { name: node.data.name, path: node.data.path }];
			computeLayout();
			setTimeout(fitToScreen, 100);
		}
	}

	function drillUp(idx: number) {
		if (idx < 0) {
			// Go to root
			drillRootPath = null;
			breadcrumb = [];
		} else {
			drillRootPath = breadcrumb[idx].path;
			breadcrumb = breadcrumb.slice(0, idx + 1);
		}
		computeLayout();
		setTimeout(fitToScreen, 100);
	}

	// Depth-based color palette
	const depthColors = ['#7c3aed', '#2563eb', '#0891b2', '#059669', '#d97706', '#dc2626', '#9333ea', '#4f46e5'];

	function nodeColor(node: any): string {
		if (colorMode === 'library') {
			return node.data.color || '#7c3aed';
		} else if (colorMode === 'status') {
			if (node.data.status === 'evergreen') return '#059669';
			if (node.data.status === 'growing') return '#d97706';
			if (node.data.status === 'seedling') return '#2563eb';
			if (node.data.isDir) return '#64748b';
			return '#94a3b8';
		} else {
			return depthColors[node.depth % depthColors.length];
		}
	}

	function fitToScreen() {
		if (!svgEl || !zoomBehavior || treeNodes.length === 0) return;
		const w = containerEl.clientWidth;
		const h = containerEl.clientHeight;

		let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
		for (const n of treeNodes) {
			if (n.x < minX) minX = n.x;
			if (n.x > maxX) maxX = n.x;
			if (n.y < minY) minY = n.y;
			if (n.y > maxY) maxY = n.y;
		}

		const padding = 100;
		const dx = maxX - minX + padding * 2;
		const dy = maxY - minY + padding * 2;
		const scale = Math.min(w / dx, h / dy, 1.5);
		const cx = (minX + maxX) / 2;
		const cy = (minY + maxY) / 2;

		const t = d3.zoomIdentity
			.translate(w / 2 - cx * scale, h / 2 - cy * scale)
			.scale(scale);

		d3.select(svgEl).transition().duration(500).call(zoomBehavior.transform as any, t);
	}

	function linkPath(link: { source: { x: number; y: number }; target: { x: number; y: number } }): string {
		const s = link.source;
		const t = link.target;
		if (treeOrientation === 'vertical' && layoutMode === 'tree') {
			const my = (s.y + t.y) / 2;
			return `M${s.x},${s.y} C${s.x},${my} ${t.x},${my} ${t.x},${t.y}`;
		} else if (layoutMode === 'tree') {
			const mx = (s.x + t.x) / 2;
			return `M${s.x},${s.y} C${mx},${s.y} ${mx},${t.y} ${t.x},${t.y}`;
		}
		return `M${s.x},${s.y} L${t.x},${t.y}`;
	}

	function nodeIcon(node: any): string {
		if (node.data.isRoot) return '🌐';
		if (node.data.isLibrary) return '📚';
		if (node.data.isDir) return node._collapsed ? '📁' : '📂';
		if (node.data.status === 'evergreen') return '🌲';
		if (node.data.status === 'growing') return '🌿';
		if (node.data.status === 'seedling') return '🌱';
		return '📄';
	}

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

	// ─── Reactivity ────────────────────────────────────────
	$effect(() => {
		// Recompute when collapsed paths change
		if (rootData) computeLayout();
	});

	async function loadHierarchy() {
		loading = true;
		drillRootPath = null;
		breadcrumb = [];
		collapsedPaths = new Set();
		switch (hierarchySource) {
			case 'folders': rootData = await loadFolderHierarchy(); break;
			case 'tags': rootData = await loadTagHierarchy(); break;
			case 'moc': rootData = await loadMOCHierarchy(); break;
			default: rootData = await loadFolderHierarchy();
		}
		loading = false;
		if (rootData) {
			computeLayout();
			setTimeout(fitToScreen, 100);
		}
	}

	// React to hierarchy source change
	let prevSource = hierarchySource;
	$effect(() => {
		if (hierarchySource !== prevSource) {
			prevSource = hierarchySource;
			loadHierarchy();
		}
	});

	// ─── Lifecycle ─────────────────────────────────────────
	onMount(async () => {
		window.addEventListener('keydown', handleKeydown);

		// Load initial data
		await loadHierarchy();

		// Setup zoom
		zoomBehavior = d3.zoom<SVGSVGElement, unknown>()
			.scaleExtent([0.05, 5])
			.on('zoom', (event) => {
				transform = { x: event.transform.x, y: event.transform.y, k: event.transform.k };
			});

		d3.select(svgEl).call(zoomBehavior);

		// Initial fit
		setTimeout(fitToScreen, 100);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
	});
</script>

<div class="oc-container" bind:this={containerEl}>
	<!-- Header -->
	<div class="oc-header">
		<span class="oc-title">{$t('orgChart.title') || 'Organization Chart'}</span>
		<div class="oc-toolbar">
			{#if searchVisible}
				<input class="oc-search" type="text" dir="auto" placeholder={$t('layout.search') || 'Search...'} bind:value={searchQuery} autofocus />
			{/if}
			<button class="oc-btn" class:active={searchVisible} onclick={() => { searchVisible = !searchVisible; if (!searchVisible) searchQuery = ''; }} title="Search (Ctrl+F)">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
			</button>
			<select class="oc-select" bind:value={hierarchySource}>
				<option value="folders">{$t('orgChart.folders') || 'Folders'}</option>
				<option value="tags">{$t('orgChart.tags') || 'Tags'}</option>
				<option value="moc">{$t('orgChart.moc') || 'MOC Links'}</option>
				<option value="parent">{$t('orgChart.parent') || 'Parent Property'}</option>
			</select>
			<select class="oc-select" bind:value={layoutMode} onchange={() => { computeLayout(); setTimeout(fitToScreen, 100); }}>
				<option value="tree">{$t('orgChart.tree') || 'Tree'}</option>
				<option value="radial">{$t('orgChart.radial') || 'Radial'}</option>
				<option value="sunburst">{$t('orgChart.sunburst') || 'Sunburst'}</option>
				<option value="treemap">{$t('orgChart.treemap') || 'Treemap'}</option>
			</select>
			{#if layoutMode === 'tree'}
				<button class="oc-btn" onclick={() => { treeOrientation = treeOrientation === 'vertical' ? 'horizontal' : 'vertical'; computeLayout(); setTimeout(fitToScreen, 100); }} title="Toggle orientation">
					{#if treeOrientation === 'vertical'}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v18"/><path d="M3 12h18"/></svg>
					{:else}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12h18"/><path d="M12 3v18"/></svg>
					{/if}
				</button>
			{/if}
			<select class="oc-select" bind:value={colorMode}>
				<option value="library">{$t('orgChart.colorLibrary') || 'Color: Library'}</option>
				<option value="status">{$t('orgChart.colorStatus') || 'Color: Status'}</option>
				<option value="depth">{$t('orgChart.colorDepth') || 'Color: Depth'}</option>
			</select>
			<button class="oc-btn" onclick={fitToScreen} title="Fit to screen">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="M9 21H3v-6"/><path d="M21 3l-7 7"/><path d="M3 21l7-7"/></svg>
			</button>
			<button class="oc-close" onclick={onClose}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
			</button>
		</div>
	</div>

	<!-- Breadcrumb -->
	{#if breadcrumb.length > 0}
		<div class="oc-breadcrumb">
			<button class="oc-crumb" onclick={() => drillUp(-1)}>🌐 Universe</button>
			{#each breadcrumb as crumb, i}
				<span class="oc-crumb-sep">›</span>
				<button class="oc-crumb" class:active={i === breadcrumb.length - 1} onclick={() => drillUp(i)}>{crumb.name}</button>
			{/each}
		</div>
	{/if}

	<!-- SVG Canvas -->
	{#if loading}
		<div class="oc-loading">
			<div class="oc-spinner"></div>
			<span>Loading hierarchy...</span>
		</div>
	{:else}
		<svg bind:this={svgEl} class="oc-svg" width="100%" height="100%">
			<!-- Sunburst arcs (no zoom transform — fills viewport) -->
			{#if layoutMode === 'sunburst' && sunburstArcs.length > 0}
				<g transform="translate({(containerEl?.clientWidth || 800) / 2},{(containerEl?.clientHeight || 600) / 2})">
					{#each sunburstArcs as arc}
						{@const innerR = arc.y0}
						{@const outerR = arc.y1}
						{@const startAngle = arc.x0 - Math.PI / 2}
						{@const endAngle = arc.x1 - Math.PI / 2}
						{@const arcGen = d3.arc()({innerRadius: innerR, outerRadius: outerR, startAngle: arc.x0, endAngle: arc.x1})}
						<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
						<path
							d={arcGen}
							fill={nodeColor({ data: arc.data, depth: arc.depth })}
							stroke="var(--background-primary)"
							stroke-width="1"
							opacity="0.85"
							class="oc-arc"
							onmouseenter={() => hoveredNode = arc}
							onmouseleave={() => hoveredNode = null}
							onclick={() => {
								if (arc.data.isDir) toggleCollapse(arc.data.path);
								else if (arc.data.isNote) onNoteClick?.(arc.data.path, arc.data.name);
							}}
						/>
						{#if (arc.x1 - arc.x0) > 0.1}
							<text
								transform="translate({d3.arc().centroid({innerRadius: innerR, outerRadius: outerR, startAngle: arc.x0, endAngle: arc.x1})})"
								text-anchor="middle" font-size="9" fill="white" style="pointer-events:none;"
							>{arc.data.name?.length > 12 ? arc.data.name.slice(0, 11) + '…' : arc.data.name}</text>
						{/if}
					{/each}
				</g>
			{/if}

			<!-- Treemap rects (no zoom transform — fills viewport) -->
			{#if layoutMode === 'treemap' && treemapRects.length > 0}
				<g transform="translate(20,20)">
					{#each treemapRects as rect}
						<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
						<g class="oc-treemap-cell"
							onmouseenter={() => hoveredNode = rect}
							onmouseleave={() => hoveredNode = null}
							onclick={() => { if (rect.data.isNote) onNoteClick?.(rect.data.path, rect.data.name); }}
						>
							<rect
								x={rect.x0} y={rect.y0}
								width={rect.x1 - rect.x0} height={rect.y1 - rect.y0}
								fill={nodeColor({ data: rect.data, depth: rect.depth })}
								stroke="var(--background-primary)" stroke-width="1"
								rx="2" opacity="0.8"
							/>
							{#if (rect.x1 - rect.x0) > 40 && (rect.y1 - rect.y0) > 14}
								<text
									x={rect.x0 + 4} y={rect.y0 + 12}
									font-size="10" fill="white" style="pointer-events:none;"
								>{rect.data.name?.length > 18 ? rect.data.name.slice(0, 17) + '…' : rect.data.name}</text>
							{/if}
						</g>
					{/each}
				</g>
			{/if}

			<!-- Tree/Radial nodes and links -->
			<g transform="translate({transform.x},{transform.y}) scale({transform.k})">
				<!-- Links -->
				{#each treeLinks as link}
					<path
						d={linkPath(link)}
						fill="none"
						stroke="var(--text-faint, #94a3b8)"
						stroke-width={1 / transform.k}
						opacity="0.4"
					/>
				{/each}

				<!-- Nodes -->
				{#each treeNodes as node}
					{@const isMatch = searchQuery && searchMatches.has(node.data.path || node.data.name)}
					{@const isDim = searchQuery && searchMatches.size > 0 && !isMatch}
					<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
					<g
						class="oc-node"
						transform="translate({node.x},{node.y})"
						onclick={() => handleNodeClick(node)}
						ondblclick={() => handleNodeDoubleClick(node)}
						onmouseenter={() => hoveredNode = node}
						onmouseleave={() => hoveredNode = null}
						opacity={isDim ? 0.2 : 1}
					>
						<!-- Background rect -->
						<rect
							x="-70" y="-16" width="140" height="32" rx="6"
							fill={isMatch ? 'color-mix(in srgb, var(--interactive-accent) 20%, var(--background-primary))' : 'var(--background-primary)'}
							stroke={nodeColor(node)}
							stroke-width={hoveredNode === node ? 2 : 1}
						/>
						<!-- Icon -->
						<text x="-58" y="5" font-size="14" text-anchor="start">{nodeIcon(node)}</text>
						<!-- Name -->
						<text
							x="-42" y="5" font-size={Math.min(12, 12 / transform.k * transform.k)} font-family="system-ui"
							fill="var(--text-normal)" text-anchor="start"
							style="pointer-events: none;"
						>
							{node.data.name?.length > 16 ? node.data.name.slice(0, 15) + '…' : node.data.name}
						</text>
						<!-- Collapsed indicator -->
						{#if node._collapsed && node.data._childCount}
							<text x="62" y="5" font-size="9" fill="var(--text-faint)" text-anchor="end">+{node.data._childCount}</text>
						{/if}
					</g>
				{/each}
			</g>
		</svg>
	{/if}

	<!-- Hover tooltip -->
	{#if hoveredNode && !loading}
		<div class="oc-tooltip" style="left:{transform.x + hoveredNode.x * transform.k + 80}px; top:{transform.y + hoveredNode.y * transform.k - 10}px">
			<div class="oc-tip-name" dir="auto">{hoveredNode.data.name}</div>
			{#if hoveredNode.data.isDir}
				<div class="oc-tip-meta">Folder · Depth {hoveredNode.depth}</div>
			{:else if hoveredNode.data.isNote}
				<div class="oc-tip-meta">{hoveredNode.data.libraryName} · {hoveredNode.data.status || 'note'}</div>
			{/if}
			{#if hoveredNode._collapsed}
				<div class="oc-tip-meta">Click to expand ({hoveredNode.data._childCount} children)</div>
			{/if}
		</div>
	{/if}

	<!-- Status bar -->
	<div class="oc-status">
		<span>{totalNodes} nodes</span>
		<span class="oc-sep">·</span>
		<span>{totalFolders} folders</span>
		<span class="oc-sep">·</span>
		<span>Depth: {maxDepth}</span>
	</div>
</div>

<style>
	.oc-container {
		position: relative; width: 100%; height: 100%; overflow: hidden;
		background: var(--background-primary); display: flex; flex-direction: column;
	}

	.oc-header {
		display: flex; justify-content: space-between; align-items: center;
		padding: 8px 12px; border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary); flex-shrink: 0; z-index: 10;
	}
	.oc-title { font-size: 14px; font-weight: 600; color: var(--text-normal); }
	.oc-toolbar { display: flex; gap: 6px; align-items: center; }

	.oc-btn {
		display: flex; align-items: center; justify-content: center;
		width: 30px; height: 30px; border: none; border-radius: 5px;
		background: transparent; color: var(--text-muted); cursor: pointer;
	}
	.oc-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.oc-btn.active { background: var(--interactive-accent); color: white; }

	.oc-select {
		height: 28px; padding: 0 6px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; cursor: pointer;
	}

	.oc-search {
		height: 28px; padding: 0 8px; border: 1px solid var(--background-modifier-border);
		border-radius: 4px; background: var(--background-primary); color: var(--text-normal);
		font-size: 12px; outline: none; min-width: 180px;
	}
	.oc-search:focus { border-color: var(--interactive-accent); }

	.oc-close {
		display: flex; align-items: center; justify-content: center;
		width: 30px; height: 30px; border: none; border-radius: 5px;
		background: transparent; color: var(--text-muted); cursor: pointer; margin-inline-start: 8px;
	}
	.oc-close:hover { background: #ef4444; color: white; }

	.oc-svg { flex: 1; cursor: grab; }
	.oc-svg:active { cursor: grabbing; }

	.oc-node { cursor: pointer; transition: opacity 0.2s; }
	.oc-node:hover rect { stroke-width: 2 !important; }

	.oc-loading {
		flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
		gap: 10px; color: var(--text-muted);
	}
	.oc-spinner {
		width: 24px; height: 24px; border: 2px solid var(--background-modifier-border);
		border-top-color: var(--interactive-accent); border-radius: 50%;
		animation: ocspin 0.6s linear infinite;
	}
	@keyframes ocspin { to { transform: rotate(360deg); } }

	.oc-tooltip {
		position: absolute; z-index: 20; pointer-events: none;
		background: var(--background-secondary); border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 6px 10px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);
		max-width: 250px;
	}
	.oc-tip-name { font-size: 13px; font-weight: 600; color: var(--text-normal); }
	.oc-tip-meta { font-size: 11px; color: var(--text-muted); margin-top: 2px; }

	.oc-status {
		display: flex; gap: 8px; align-items: center; padding: 6px 12px;
		background: var(--background-secondary); border-top: 1px solid var(--background-modifier-border);
		font-size: 11px; color: var(--text-faint); flex-shrink: 0;
	}
	.oc-sep { opacity: 0.3; }

	/* Breadcrumb */
	.oc-breadcrumb {
		display: flex; align-items: center; gap: 4px;
		padding: 4px 12px; background: var(--background-secondary);
		border-bottom: 1px solid var(--background-modifier-border);
		font-size: 12px; flex-shrink: 0; overflow-x: auto;
	}
	.oc-crumb {
		border: none; background: transparent; color: var(--text-muted);
		cursor: pointer; padding: 2px 6px; border-radius: 3px; white-space: nowrap;
	}
	.oc-crumb:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.oc-crumb.active { color: var(--interactive-accent); font-weight: 600; }
	.oc-crumb-sep { color: var(--text-faint); font-size: 10px; }
</style>

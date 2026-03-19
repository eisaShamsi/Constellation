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
	let layoutMode = $state<'tree' | 'radial'>('tree');
	let treeOrientation = $state<'vertical' | 'horizontal'>('vertical');

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

		const filtered = filterCollapsed(rootData);
		d3Root = d3.hierarchy(filtered);

		// Compute layout
		const nodeCount = d3Root.descendants().length;
		const nodeSize: [number, number] = treeOrientation === 'vertical' ? [180, 80] : [40, 220];

		if (layoutMode === 'tree') {
			const treeLayout = d3.tree().nodeSize(nodeSize);
			treeLayout(d3Root as any);
		} else {
			// Radial
			const treeLayout = d3.tree().size([2 * Math.PI, Math.min(400, nodeCount * 8)]);
			treeLayout(d3Root as any);
		}

		// Extract nodes and links
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

	// ─── Lifecycle ─────────────────────────────────────────
	onMount(async () => {
		window.addEventListener('keydown', handleKeydown);

		// Load data
		rootData = await loadFolderHierarchy();
		loading = false;

		if (!rootData) return;
		computeLayout();

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
			<button class="oc-btn" onclick={fitToScreen} title="Fit to screen">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="M9 21H3v-6"/><path d="M21 3l-7 7"/><path d="M3 21l7-7"/></svg>
			</button>
			<button class="oc-close" onclick={onClose}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
			</button>
		</div>
	</div>

	<!-- SVG Canvas -->
	{#if loading}
		<div class="oc-loading">
			<div class="oc-spinner"></div>
			<span>Loading hierarchy...</span>
		</div>
	{:else}
		<svg bind:this={svgEl} class="oc-svg" width="100%" height="100%">
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
						onmouseenter={() => hoveredNode = node}
						onmouseleave={() => hoveredNode = null}
						opacity={isDim ? 0.2 : 1}
					>
						<!-- Background rect -->
						<rect
							x="-70" y="-16" width="140" height="32" rx="6"
							fill={isMatch ? 'color-mix(in srgb, var(--interactive-accent) 20%, var(--background-primary))' : 'var(--background-primary)'}
							stroke={node.data.color || 'var(--text-faint)'}
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
</style>

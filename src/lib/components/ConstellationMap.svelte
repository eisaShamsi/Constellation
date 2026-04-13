<script lang="ts">
	/**
	 * ConstellationMap — CE Layer 2.
	 * Radial sunburst visualization of knowledge structure, density, and maturity.
	 * Inspired by Goalscape. Uses D3.js d3.partition() for layout.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t, dir, getSearchOps } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import * as d3 from 'd3';
	import { get } from 'svelte/store';
	import {
		stripInvisibleChars, canonicalizeSearchQuery, hasAdvancedSyntaxMultilingual,
		universalSearch, constellationSearch, parseSearchQuery,
		embedText, appSettings, type UniversalSearchResponse,
	} from '$lib/libraries/store';
	import { readSearchHistory, addSearchHistory } from '$lib/libraries/searchHistory';

	interface MapNode {
		name: string;
		path: string;
		is_dir: boolean;
		node_type: string; // "universe" | "child_universe" | "library" | "folder" | "note"
		weight: number;
		note_count: number;
		word_count: number;
		link_count: number;
		maturity: string | null;
		stratum: number | null;
		modified: number | null;
		children: MapNode[] | null;
	}

	let {
		universeName = '',
		libraryPath = '',
		libraryName = '',
		libraryColor = '#7c3aed',
		libraryColorMap = {} as Record<string, string>,
		initialData = null as MapNode | null,
		compact = false,
		initialColorMode,
		onNoteClick,
		onClose,
		onDrillDown,
		onColorModeChange,
	}: {
		universeName?: string;
		libraryPath?: string;
		libraryName?: string;
		libraryColor?: string;
		libraryColorMap?: Record<string, string>;
		initialData?: MapNode | null;
		compact?: boolean;
		initialColorMode?: 'maturity' | 'stratum' | 'library';
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
		onDrillDown?: (node: MapNode, breadcrumbNames: string[]) => void;
		onColorModeChange?: (mode: string) => void;
	} = $props();

	let containerEl: HTMLDivElement | undefined;
	let svgEl: SVGSVGElement | undefined;
	let loading = $state(true);
	let error = $state('');
	let mapData = $state<MapNode | null>(null);
	let colorMode = $state<'maturity' | 'stratum' | 'library'>(initialColorMode ?? 'maturity');
	let breadcrumb = $state<{ name: string; node: any }[]>([]);
	let tooltip = $state<{ x: number; y: number; node: MapNode; visible: boolean }>({ x: 0, y: 0, node: null as any, visible: false });

	// Search
	let searchVisible = $state(false);
	let searchQuery = $state('');
	let searchResults = $state<MapNode[]>([]);
	let searchCats = $state<Map<string, string[]>>(new Map()); // path → categories
	let searchIdx = $state(0);
	let showHistory = $state(false);
	let historyItems = $state<{ query: string; timestamp: number }[]>([]);

	const CAT_COLORS: Record<string, string> = {
		T: '#3b82f6', C: '#16a34a', '#': '#f472b6', P: '#f59e0b', S: '#7c3aed', W: '#94a3b8',
	};

	// Sidebar resize
	let sidebarWidth = $state(260);
	let isResizing = false;

	function onResizeStart(e: MouseEvent) {
		isResizing = true;
		e.preventDefault();
		const onMove = (ev: MouseEvent) => {
			if (!isResizing) return;
			// For RTL, measure from right edge
			const container = (e.target as Element).closest('.cmap-content-wrap');
			if (!container) return;
			const rect = container.getBoundingClientRect();
			const isR = $dir === 'rtl';
			const newWidth = isR ? rect.right - ev.clientX : ev.clientX - rect.left;
			sidebarWidth = Math.max(180, Math.min(500, newWidth));
		};
		const onUp = () => {
			isResizing = false;
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		};
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	// Syntax chips
	let showChips = $state(false);
	const syntaxChips = $derived.by(() => {
		const _locale = $t('searchHub.linksTo');
		const ops = getSearchOps();
		return [
			{ label: 'linksTo', syntax: (ops?.linksTo ?? 'links to') + ' [[' },
			{ label: 'linksFrom', syntax: (ops?.linksFrom ?? 'links from') + ' [[' },
			{ label: 'orphans', syntax: ops?.orphans ?? 'orphans' },
			{ label: 'tag', syntax: '#' },
			{ label: 'supports', syntax: (ops?.supports ?? 'supports') + ' [[' },
			{ label: 'contradicts', syntax: (ops?.contradicts ?? 'contradicts') + ' [[' },
		];
	});

	// Settings (persisted across remounts)
	const _mapDefaults = { arcOpacity: 0.75, depthLimit: 5, showLabels: true };
	const _mapSaved = (globalThis as any).__mapSettings ?? { ..._mapDefaults };
	let settingsVisible = $state(false);
	let arcOpacity = $state(_mapSaved.arcOpacity);
	let depthLimit = $state(_mapSaved.depthLimit);
	let showLabels = $state(_mapSaved.showLabels);

	function persistMapSettings() {
		(globalThis as any).__mapSettings = { arcOpacity, depthLimit, showLabels };
	}

	// Maturity colors
	const MATURITY_COLORS: Record<string, string> = {
		seed: '#d1d5db',
		sapling: '#86efac',
		evergreen: '#16a34a',
		canonical: '#f59e0b',
		wilting: '#a3e635',
	};

	// Stratum colors (1-8, cool→warm)
	const STRATUM_COLORS = ['#3b82f6', '#6366f1', '#8b5cf6', '#a855f7', '#d946ef', '#ec4899', '#f43f5e', '#ef4444'];

	// Folder colors (depth-based)
	const DEPTH_COLORS = ['#7c3aed', '#6366f1', '#8b5cf6', '#a78bfa', '#c4b5fd'];

	/** Find the library name for a node by walking up to depth 1 */
	function getLibraryName(d: any): string {
		let node = d;
		while (node && node.depth > 1) node = node.parent;
		return node?.data?.name || '';
	}

	function getNodeColor(d: any): string {
		const data = d.data as MapNode;

		// Child universe: distinct purple tint
		if (data.node_type === 'child_universe') return '#6366f1';

		// Library: use library color
		if (data.node_type === 'library') {
			return libraryColorMap[data.name] || libraryColor;
		}

		// Folder: inherit from parent library color, with slight depth fade
		if (data.is_dir) {
			const libName = getLibraryName(d);
			const libColor = libraryColorMap[libName];
			if (libColor) return libColor;
			return DEPTH_COLORS[Math.min(d.depth, DEPTH_COLORS.length - 1)];
		}

		// Notes: depends on color mode
		if (colorMode === 'maturity') {
			return MATURITY_COLORS[data.maturity || 'seed'] || '#d1d5db';
		}
		if (colorMode === 'stratum') {
			const s = (data.stratum || 1) - 1;
			return STRATUM_COLORS[Math.min(s, 7)];
		}
		// Library color — inherit from parent library
		const libName = getLibraryName(d);
		return libraryColorMap[libName] || libraryColor;
	}

	function getNodeOpacity(d: any): number {
		const data = d.data as MapNode;
		if (data.is_dir) return 0.85;
		if (colorMode === 'maturity') {
			const m = data.maturity || 'seed';
			if (m === 'seed') return 0.4 * arcOpacity;
			if (m === 'sapling') return 0.6 * arcOpacity;
			if (m === 'evergreen') return 0.8 * arcOpacity;
			if (m === 'canonical') return 0.95 * arcOpacity;
			if (m === 'wilting') return 0.5 * arcOpacity;
		}
		return arcOpacity;
	}

	let currentRoot: any = null;

	function renderSunburst(data: MapNode, focusNode?: any) {
		if (!svgEl || !containerEl) return;

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		// Skip render if container is hidden (display:none gives 0 dimensions)
		if (width < 10 || height < 10) return;
		const radius = Math.min(width, height) / 2 - 20;

		// Build hierarchy
		const root = d3.hierarchy(data)
			.sum(d => d.is_dir ? 0 : Math.max(d.weight, 0.1))
			.sort((a, b) => (b.value || 0) - (a.value || 0));

		const partition = d3.partition<MapNode>()
			.size([2 * Math.PI, radius]);

		partition(root);
		currentRoot = root;

		// Determine focus
		const focus = focusNode || root;

		// Arc generator
		const arc = d3.arc<any>()
			.startAngle((d: any) => d.x0)
			.endAngle((d: any) => d.x1)
			.innerRadius((d: any) => d.y0)
			.outerRadius((d: any) => d.y1)
			.padAngle(0.002)
			.padRadius(radius / 2);

		// Clear previous
		const svg = d3.select(svgEl);
		svg.selectAll('*').remove();

		svg.attr('viewBox', `${-width / 2} ${-height / 2} ${width} ${height}`);

		const g = svg.append('g');

		// Draw arcs
		const nodes = root.descendants().filter(d => d.depth > 0); // skip root

		g.selectAll('path')
			.data(nodes)
			.join('path')
			.attr('d', arc as any)
			.attr('fill', (d: any) => getNodeColor(d))
			.attr('fill-opacity', (d: any) => getNodeOpacity(d))
			.attr('stroke', '#fff')
			.attr('stroke-width', 0.5)
			.style('cursor', 'pointer')
			.on('click', (_event: MouseEvent, d: any) => {
				const data = d.data as MapNode;
				if (data.is_dir && data.children && data.children.length > 0) {
					// Drill down
					breadcrumb = [...breadcrumb, { name: data.name, node: d }];
					renderSunburst(data);
					onDrillDown?.(data, breadcrumb.map(b => b.name));
				} else if (!data.is_dir && onNoteClick) {
					onNoteClick(data.path, data.name);
				}
			})
			.on('mouseenter', (event: MouseEvent, d: any) => {
				// Only highlight on hover if no search is active
				if (searchResults.length === 0) {
					d3.select(event.currentTarget as Element).attr('fill-opacity', 1).attr('stroke-width', 2);
				}
			})
			.on('mouseleave', (event: MouseEvent, d: any) => {
				// Only restore on leave if no search is active
				if (searchResults.length === 0) {
					d3.select(event.currentTarget as Element)
						.attr('fill-opacity', (d: any) => getNodeOpacity(d))
						.attr('stroke-width', 0.5);
				}
			});

		// Center circle radius = inner edge of the first ring
		const innerRadius = nodes.length > 0 ? (nodes[0] as any).y0 : radius * 0.3;

		// Center background circle
		g.append('circle')
			.attr('cx', 0).attr('cy', 0).attr('r', innerRadius)
			.attr('fill', 'var(--background-primary, #fff)');

		// Center label — clipped to inner ring boundary
		const clipId = `center-clip-${Date.now()}`;
		svg.append('defs').append('clipPath').attr('id', clipId)
			.append('circle').attr('cx', 0).attr('cy', 0).attr('r', innerRadius * 0.85);

		const centerG = g.append('g').attr('clip-path', `url(#${clipId})`);

		// Adaptive font size based on name length and available space
		const maxTextWidth = innerRadius * 1.6;
		const nameFontSize = Math.min(16, Math.max(9, maxTextWidth / Math.max(data.name.length * 0.55, 1)));

		// Line 1: Title
		centerG.append('text')
			.attr('text-anchor', 'middle')
			.attr('y', -nameFontSize * 0.6)
			.attr('font-size', `${nameFontSize}px`)
			.attr('font-weight', '700')
			.attr('fill', 'var(--text-normal, #333)')
			.attr('dir', 'auto')
			.text(data.name);

		// Line 2: Note count
		centerG.append('text')
			.attr('text-anchor', 'middle')
			.attr('y', nameFontSize * 0.5)
			.attr('font-size', '10px')
			.attr('fill', 'var(--text-muted, #888)')
			.text(`${data.note_count} ${$t('constellationMap.notes') || 'notes'}`);

		// Line 3: Word count
		centerG.append('text')
			.attr('text-anchor', 'middle')
			.attr('y', nameFontSize * 0.5 + 14)
			.attr('font-size', '10px')
			.attr('fill', 'var(--text-muted, #888)')
			.text(`${data.word_count.toLocaleString()} ${$t('constellationMap.words') || 'words'}`);
	}

	function zoomToRoot() {
		breadcrumb = [];
		if (mapData) {
			renderSunburst(mapData);
			onDrillDown?.(mapData, []);
		}
	}

	function zoomToBreadcrumb(idx: number) {
		if (idx === -1) {
			zoomToRoot();
			return;
		}
		breadcrumb = breadcrumb.slice(0, idx + 1);
		const target = breadcrumb[idx];
		if (target) {
			renderSunburst(target.node.data);
			onDrillDown?.(target.node.data as MapNode, breadcrumb.map(b => b.name));
		}
	}

	async function loadData() {
		loading = true;
		error = '';
		try {
			let data: MapNode;
			if (initialData) {
				// Use pre-provided data (for SS companion mini-maps)
				data = initialData;
			} else {
				// Load universe-level map (all libraries)
				data = await invoke<MapNode>('constellation_map_universe', {
					universeName: universeName || 'Universe',
					maxDepth: depthLimit,
				});
			}
			mapData = data;
			breadcrumb = [];
			requestAnimationFrame(() => renderSunburst(data));
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	// Resize handler
	let resizeObserver: ResizeObserver | null = null;
	let resizeTimer: ReturnType<typeof setTimeout> | null = null;

	onMount(() => {
		loadData();
		if (containerEl) {
			resizeObserver = new ResizeObserver(() => {
				if (resizeTimer) clearTimeout(resizeTimer);
				resizeTimer = setTimeout(() => {
					if (mapData) {
						const current = breadcrumb.length > 0 ? breadcrumb[breadcrumb.length - 1].node.data : mapData;
						renderSunburst(current);
					}
				}, 200);
			});
			resizeObserver.observe(containerEl);
		}
	});

	onDestroy(() => {
		if (resizeObserver) resizeObserver.disconnect();
		if (resizeTimer) clearTimeout(resizeTimer);
	});

	// Sync external color mode prop
	$effect(() => {
		if (initialColorMode && initialColorMode !== colorMode) colorMode = initialColorMode;
	});

	// Re-render when color mode changes
	$effect(() => {
		colorMode; // track
		if (mapData && svgEl) {
			const current = breadcrumb.length > 0 ? breadcrumb[breadcrumb.length - 1].node.data : mapData;
			renderSunburst(current);
			onColorModeChange?.(colorMode);
		}
	});

	// ─── Search within Map (same engine as Sight) ───────────
	function collectAllNodes(node: MapNode, results: MapNode[] = []): MapNode[] {
		results.push(node);
		if (node.children) for (const c of node.children) collectAllNodes(c, results);
		return results;
	}

	async function executeMapSearch() {
		if (!searchQuery.trim() || !mapData) { searchResults = []; searchIdx = 0; searchCats = new Map(); highlightAllResults(); return; }

		addSearchHistory(searchQuery);
		historyItems = readSearchHistory();

		const cleanQ = stripInvisibleChars(searchQuery);
		const ops = getSearchOps();
		const canonicalized = canonicalizeSearchQuery(cleanQ, ops);
		const isAdvanced = hasAdvancedSyntaxMultilingual(canonicalized, ops);

		const all = collectAllNodes(mapData);
		const nameMap = new Map(all.map(n => [n.name.toLowerCase(), n]));
		const matchedNodes: MapNode[] = [];
		const cats = new Map<string, string[]>();

		if (isAdvanced) {
			try {
				const req = parseSearchQuery(canonicalized);
				const results = await constellationSearch(req);
				const CAT_MAP: Record<string, string> = { wikilink: 'W', title: 'T', content: 'C', tag: '#', property: 'P', semantic: 'S', hybrid: 'C' };
				for (const r of results) {
					const node = nameMap.get(r.name.toLowerCase());
					if (node) {
						matchedNodes.push(node);
						const cat = CAT_MAP[r.match_type] ?? r.match_type.charAt(0).toUpperCase();
						cats.set(node.path, [cat]);
					}
				}
			} catch {}
		} else {
			const q = searchQuery.toLowerCase();
			const titleMatchPaths = new Set<string>();
			const contentMatchPaths = new Set<string>();
			const tagMatchPaths = new Set<string>();
			const propMatchPaths = new Set<string>();
			const semMatchPaths = new Set<string>();

			// Local title matches
			for (const n of all) {
				if (n.name.toLowerCase().includes(q)) { matchedNodes.push(n); titleMatchPaths.add(n.path); }
			}

			// Backend search
			try {
				let qEmbed: number[] | null = null;
				if (get(appSettings).enabledFeatures?.semanticSearch) {
					try { qEmbed = await embedText(canonicalized); } catch {}
				}
				const resp: any = await universalSearch(canonicalized, qEmbed, 200);
				for (const r of resp?.titles ?? []) { const n = nameMap.get(r.name.toLowerCase()); if (n) { titleMatchPaths.add(n.path); if (!matchedNodes.includes(n)) matchedNodes.push(n); } }
				for (const r of resp?.contents ?? []) { const n = nameMap.get(r.name.toLowerCase()); if (n) { contentMatchPaths.add(n.path); if (!matchedNodes.includes(n)) matchedNodes.push(n); } }
				for (const r of resp?.tags ?? []) { const n = nameMap.get(r.name.toLowerCase()); if (n) { tagMatchPaths.add(n.path); if (!matchedNodes.includes(n)) matchedNodes.push(n); } }
				for (const r of resp?.properties ?? []) { const n = nameMap.get(r.name.toLowerCase()); if (n) { propMatchPaths.add(n.path); if (!matchedNodes.includes(n)) matchedNodes.push(n); } }
				for (const r of resp?.semantic ?? []) { const n = nameMap.get(r.name.toLowerCase()); if (n) { semMatchPaths.add(n.path); if (!matchedNodes.includes(n)) matchedNodes.push(n); } }
			} catch {}

			// Build category map
			for (const n of matchedNodes) {
				const c: string[] = [];
				if (titleMatchPaths.has(n.path)) c.push('T');
				if (contentMatchPaths.has(n.path)) c.push('C');
				if (tagMatchPaths.has(n.path)) c.push('#');
				if (propMatchPaths.has(n.path)) c.push('P');
				if (semMatchPaths.has(n.path)) c.push('S');
				if (c.length > 0) cats.set(n.path, c);
			}
		}

		searchResults = matchedNodes;
		searchCats = cats;
		searchIdx = 0;
		if (searchResults.length > 0) highlightSearchResult();
		else highlightAllResults();
	}

	function highlightSearchResult() {
		if (!svgEl) return;
		const match = searchResults[searchIdx];
		const matchSet = new Set(searchResults.map(n => n.path));

		// Highlight arcs: current = black border, other matches = blue, rest = dimmed
		d3.select(svgEl).selectAll('path').each(function(d: any) {
			const isMatch = d?.data?.path && matchSet.has(d.data.path);
			const isCurrent = match && d?.data?.path === match.path && d?.data?.name === match.name;
			d3.select(this)
				.attr('stroke', isCurrent ? '#000000' : isMatch ? '#3b82f6' : '#fff')
				.attr('stroke-width', isCurrent ? 3.5 : isMatch ? 2 : 0.5)
				.attr('fill-opacity', isMatch || isCurrent ? 1 : 0.3);
		});
	}

	function selectSearchResult(idx: number) {
		searchIdx = idx;
		highlightSearchResult();
	}

	function highlightAllResults() {
		if (!svgEl) return;
		// Reset all arcs to normal
		d3.select(svgEl).selectAll('path').each(function(d: any) {
			d3.select(this).attr('stroke', '#fff').attr('stroke-width', 0.5).attr('fill-opacity', (d: any) => getNodeOpacity(d));
		});
	}

	function nextMapResult() {
		if (searchResults.length === 0) return;
		searchIdx = (searchIdx + 1) % searchResults.length;
		highlightSearchResult();
	}

	function prevMapResult() {
		if (searchResults.length === 0) return;
		searchIdx = (searchIdx - 1 + searchResults.length) % searchResults.length;
		highlightSearchResult();
	}

	function resetMapSearch() {
		searchQuery = '';
		searchResults = [];
		searchCats = new Map();
		searchIdx = 0;
		highlightAllResults();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (breadcrumb.length > 0) {
				zoomToBreadcrumb(breadcrumb.length - 2);
			} else {
				onClose?.();
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="cmap" class:cmap-compact={compact}>
	<!-- Header -->
	{#if !compact}
	<div class="cmap-header">
		<div class="cmap-header-left">
			<span class="cmap-icon">🗺️</span>
			<span class="cmap-title">{$t('constellationMap.title') || 'Constellation Map'}</span>
			<span class="cmap-lib" dir="auto">{libraryName}</span>
		</div>
		<div class="cmap-header-right">
			<select class="cmap-color-select" bind:value={colorMode}>
				<option value="maturity">{$t('constellationMap.colorByMaturity') || 'Maturity'}</option>
				<option value="stratum">{$t('constellationMap.colorByStratum') || 'Stratum'}</option>
				<option value="library">{$t('constellationMap.colorByLibrary') || 'Library'}</option>
			</select>
			<!-- Search toggle -->
			<button class="cmap-toolbar-btn" class:active={searchVisible} onclick={() => { searchVisible = !searchVisible; if (!searchVisible) resetMapSearch(); }} title={$t('layout.search') || 'Search'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<!-- Fit to Screen (zoom to root) -->
			<button class="cmap-toolbar-btn" onclick={zoomToRoot} title={$t('lens.fitToScreen') || 'Fit to screen'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M21 8V5a2 2 0 0 0-2-2h-3"/><path d="M3 16v3a2 2 0 0 0 2 2h3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
			</button>
			<!-- Settings toggle -->
			<button class="cmap-toolbar-btn" class:active={settingsVisible} onclick={() => settingsVisible = !settingsVisible} title={$t('ribbon.settings') || 'Settings'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/></svg>
			</button>
			{#if onClose}
				<button class="cmap-close" onclick={onClose}>×</button>
			{/if}
		</div>
	</div>
	{/if}

	<!-- Breadcrumb -->
	{#if breadcrumb.length > 0}
		<div class="cmap-breadcrumb">
			<button class="cmap-bc-item" onclick={zoomToRoot} dir="auto">{mapData?.name || libraryName}</button>
			{#each breadcrumb as bc, i}
				<span class="cmap-bc-sep">/</span>
				<button class="cmap-bc-item" class:active={i === breadcrumb.length - 1} onclick={() => zoomToBreadcrumb(i)} dir="auto">{bc.name}</button>
			{/each}
		</div>
	{/if}

	<!-- Search bar -->
	{#if searchVisible}
		<div class="cmap-search">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			<div class="cmap-search-input-wrap">
				<input type="text" dir="auto"
					placeholder={$t('lens.searchAll') || 'Search... (Enter)'}
					bind:value={searchQuery}
					onfocus={() => { if (!searchQuery) { historyItems = readSearchHistory(); showHistory = true; } }}
					onblur={() => setTimeout(() => { showHistory = false; }, 200)}
					oninput={() => { showHistory = false; }}
					onkeydown={(e) => {
						if (e.key === 'Enter') { e.preventDefault(); searchResults.length > 0 ? (e.shiftKey ? prevMapResult() : nextMapResult()) : executeMapSearch(); }
						if (e.key === 'Escape') { searchVisible = false; resetMapSearch(); e.stopPropagation(); }
					}} />
				{#if showHistory && historyItems.length > 0 && !searchQuery}
					<div class="cmap-history-dropdown">
						{#each historyItems.slice(0, 8) as item}
							<button class="cmap-history-item" onclick={() => { searchQuery = item.query; showHistory = false; executeMapSearch(); }} dir="auto">{item.query}</button>
						{/each}
					</div>
				{/if}
			</div>
			<button class="cmap-search-clear" onclick={resetMapSearch}>×</button>
				<button class="cmap-chips-btn" class:active={showChips} onclick={() => showChips = !showChips} title={$t('searchHub.syntaxHelpers') || 'Syntax'}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
				</button>
				{#if showChips}
					<div class="cmap-chips-dropdown">
						{#each syntaxChips as chip}
							<button class="cmap-chip" onclick={() => { searchQuery = searchQuery ? searchQuery + ' ' + chip.syntax : chip.syntax; showChips = false; }}>{$t(`searchHub.${chip.label}`)}</button>
						{/each}
					</div>
				{/if}
			{#if searchResults.length > 0}
				{@const curCats = searchCats.get(searchResults[searchIdx]?.path) ?? []}
				{#each curCats as cat}
					<span class="cmap-search-cat" style="background:{CAT_COLORS[cat] ?? '#94a3b8'}">{cat}</span>
				{/each}
				<span class="cmap-search-count">{searchIdx + 1}/{searchResults.length}</span>
				<button class="cmap-search-nav" onclick={prevMapResult}>
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
				</button>
				<button class="cmap-search-nav" onclick={nextMapResult}>
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 6 15 12 9 18"/></svg>
				</button>
			{:else if searchQuery}
				<span class="cmap-search-none">0</span>
			{/if}
		</div>
	{/if}

	<!-- Settings panel -->
	{#if settingsVisible}
		<div class="cmap-settings">
			<label class="cmap-settings-slider">
				<span>{$t('lens.settingOpacity') || 'Opacity'}: {Math.round(arcOpacity * 100)}%</span>
				<input type="range" min="0.3" max="1" step="0.05" bind:value={arcOpacity} oninput={() => {
					persistMapSettings();
					const current = breadcrumb.length > 0 ? breadcrumb[breadcrumb.length - 1].node.data : mapData;
					if (current) renderSunburst(current);
				}} />
			</label>
			<label class="cmap-settings-slider">
				<span>Depth: {depthLimit}</span>
				<input type="range" min="2" max="8" step="1" bind:value={depthLimit} oninput={() => {
					persistMapSettings();
					loadData();
				}} />
			</label>
		</div>
	{/if}

	<!-- Content: results sidebar (left) + sunburst (right) -->
	<div class="cmap-content-wrap">
		<!-- Search results sidebar -->
		{#if searchResults.length > 0}
			<div class="cmap-results" style="width:{sidebarWidth}px">
				<div class="cmap-results-header">{searchResults.length} {$t('sightPanel.totalNodes') || 'results'}</div>
				<div class="cmap-results-list">
					{#each searchResults as result, i}
						<button class="cmap-result-row" class:active={i === searchIdx}
							onclick={() => selectSearchResult(i)}
							ondblclick={() => { if (!result.is_dir && onNoteClick) onNoteClick(result.path, result.name); }}
							dir="auto">
							<span class="cmap-result-name">{result.name}</span>
							<span class="cmap-result-badges">
								{#each (searchCats.get(result.path) ?? []) as cat}
									<span class="cmap-result-badge" style="background:{CAT_COLORS[cat] ?? '#94a3b8'}">{cat}</span>
								{/each}
							</span>
						</button>
					{/each}
				</div>
			</div>
			<!-- Resize handle -->
			<div class="cmap-resize-handle" onmousedown={onResizeStart}></div>
		{/if}

		<!-- Sunburst -->
		<div class="cmap-body" bind:this={containerEl}>
			{#if loading}
				<div class="cmap-loading">
					<div class="cmap-spinner"></div>
					<p>{$t('constellationMap.loading') || 'Building knowledge map...'}</p>
				</div>
			{:else if error}
				<div class="cmap-error">
					<p>{$t('constellationMap.noData') || 'No data available'}</p>
				</div>
			{:else}
				<svg bind:this={svgEl} class="cmap-svg"></svg>
			{/if}
		</div>
	</div>

	<!-- Tooltip -->
	{#if tooltip.visible && tooltip.node}
		<div class="cmap-tooltip" style="left:{tooltip.x + 20}px;top:{tooltip.y - 60}px" dir="auto">
			<div class="cmap-tt-name">{tooltip.node.name}</div>
			{#if tooltip.node.node_type === 'child_universe'}
				<div class="cmap-tt-type">{$t('constellationMap.childUniverse') || 'Child Universe'}</div>
				<div class="cmap-tt-row">{tooltip.node.note_count} {$t('constellationMap.notes') || 'notes'}</div>
				<div class="cmap-tt-row">{tooltip.node.word_count.toLocaleString()} {$t('constellationMap.words') || 'words'}</div>
			{:else if tooltip.node.node_type === 'library'}
				<div class="cmap-tt-type">{$t('constellationMap.library') || 'Library'}</div>
				<div class="cmap-tt-row">{tooltip.node.note_count} {$t('constellationMap.notes') || 'notes'}</div>
				<div class="cmap-tt-row">{tooltip.node.word_count.toLocaleString()} {$t('constellationMap.words') || 'words'}</div>
				<div class="cmap-tt-row">{tooltip.node.link_count} {$t('constellationMap.links') || 'links'}</div>
			{:else if tooltip.node.is_dir}
				<div class="cmap-tt-row">{tooltip.node.note_count} {$t('constellationMap.notes') || 'notes'}</div>
				<div class="cmap-tt-row">{tooltip.node.word_count.toLocaleString()} {$t('constellationMap.words') || 'words'}</div>
				<div class="cmap-tt-row">{tooltip.node.link_count} {$t('constellationMap.links') || 'links'}</div>
			{:else}
				<div class="cmap-tt-row">{tooltip.node.word_count.toLocaleString()} {$t('constellationMap.words') || 'words'} · {tooltip.node.link_count} {$t('constellationMap.links') || 'links'}</div>
				{#if tooltip.node.maturity}
					<div class="cmap-tt-row">
						<span class="cmap-tt-dot" style="background:{MATURITY_COLORS[tooltip.node.maturity] || '#999'}"></span>
						{tooltip.node.maturity}
					</div>
				{/if}
				{#if tooltip.node.stratum}
					<div class="cmap-tt-row">L{tooltip.node.stratum}</div>
				{/if}
			{/if}
		</div>
	{/if}

	<!-- Legend -->
	{#if !compact}
	<div class="cmap-legend">
		{#if colorMode === 'maturity'}
			<span class="cmap-legend-title">{$t('constellationMap.maturity') || 'Maturity'}</span>
			{#each Object.entries(MATURITY_COLORS) as [label, color]}
				<span class="cmap-legend-item"><span class="cmap-legend-dot" style="background:{color}"></span>{label}</span>
			{/each}
		{:else if colorMode === 'stratum'}
			<span class="cmap-legend-title">{$t('constellationMap.stratum') || 'Stratum'}</span>
			{#each STRATUM_COLORS as color, i}
				<span class="cmap-legend-item"><span class="cmap-legend-dot" style="background:{color}"></span>L{i + 1}</span>
			{/each}
		{:else if colorMode === 'library'}
			<span class="cmap-legend-title">{$t('constellationMap.library') || 'Library'}</span>
			{#each Object.entries(libraryColorMap) as [name, color]}
				<span class="cmap-legend-item"><span class="cmap-legend-dot" style="background:{color}"></span>{name}</span>
			{/each}
		{/if}
	</div>
	{/if}
</div>

<style>
	.cmap { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: var(--background-primary, #fff); }

	.cmap-header {
		display: flex; align-items: center; justify-content: space-between;
		padding: 10px 20px; border-bottom: 1px solid var(--border, #e0e0e0);
		flex-shrink: 0;
	}
	.cmap-header-left { display: flex; align-items: center; gap: 10px; }
	.cmap-icon { font-size: 18px; }
	.cmap-title { font-size: 13px; font-weight: 700; color: var(--interactive-accent, #7c3aed); text-transform: uppercase; letter-spacing: 1px; }
	.cmap-lib { font-size: 15px; font-weight: 600; color: var(--text-normal, #333); }
	.cmap-header-right { display: flex; align-items: center; gap: 10px; }
	.cmap-color-select {
		padding: 4px 8px; border-radius: 6px; border: 1px solid var(--border, #ddd);
		background: var(--background-secondary, #f5f5f5); font-size: 12px; cursor: pointer;
		color: var(--text-normal, #333);
	}
	.cmap-toolbar-btn {
		width: 28px; height: 28px; border: none; border-radius: 4px;
		background: none; color: var(--text-muted, #888); cursor: pointer;
		display: flex; align-items: center; justify-content: center;
	}
	.cmap-toolbar-btn:hover { background: var(--background-modifier-hover, #f1f5f9); color: var(--text-normal, #333); }
	.cmap-toolbar-btn.active { background: var(--interactive-accent, #7c3aed); color: white; }
	.cmap-close {
		width: 32px; height: 32px; border-radius: 50%; border: 1px solid var(--border, #ddd);
		background: transparent; color: var(--text-muted, #888); font-size: 18px;
		cursor: pointer; display: flex; align-items: center; justify-content: center;
	}
	.cmap-close:hover { background: var(--border, #eee); color: var(--text-normal, #333); }

	/* Search bar */
	.cmap-search {
		display: flex; align-items: center; gap: 6px;
		padding: 6px 20px; border-bottom: 1px solid var(--border, #e0e0e0);
		flex-shrink: 0; position: relative;
	}
	.cmap-search svg { color: var(--text-muted, #888); flex-shrink: 0; }
	.cmap-search-input-wrap { position: relative; min-width: 200px; max-width: 400px; }
	.cmap-search-input-wrap input {
		border: none; outline: none; background: none; font-size: 12px;
		font-family: inherit; color: var(--text-normal, #333); width: 100%;
	}
	.cmap-history-dropdown {
		position: absolute; top: 100%; inset-inline-start: 0; z-index: 100;
		background: var(--background-primary, #fff); border: 1px solid var(--border, #e0e0e0);
		border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);
		min-width: 200px; max-height: 250px; overflow-y: auto; margin-top: 4px;
	}
	.cmap-history-item {
		display: block; width: 100%; text-align: start; padding: 6px 12px;
		border: none; background: none; cursor: pointer; font-size: 11px;
		color: var(--text-normal, #333); font-family: inherit;
	}
	.cmap-history-item:hover { background: var(--background-modifier-hover, #f1f5f9); }
	.cmap-search-clear { border: none; background: none; color: var(--text-muted); cursor: pointer; font-size: 14px; padding: 0 2px; }
	.cmap-search-cat { font-size: 9px; color: #fff; padding: 1px 5px; border-radius: 4px; white-space: nowrap; }
	.cmap-search-count { font-size: 10px; color: var(--text-muted); white-space: nowrap; }
	.cmap-search-none { font-size: 10px; color: #ef4444; white-space: nowrap; }
	.cmap-search-nav { border: none; background: none; color: var(--text-muted); cursor: pointer; padding: 0 2px; display: flex; align-items: center; }
	.cmap-search-nav:hover { color: var(--text-normal); }
	.cmap-chips-btn { border: none; background: none; cursor: pointer; color: var(--text-muted, #888); padding: 2px; border-radius: 3px; }
	.cmap-chips-btn:hover, .cmap-chips-btn.active { color: var(--interactive-accent, #7c3aed); background: rgba(124,58,237,0.1); }
	.cmap-chips-dropdown {
		position: absolute; top: 100%; inset-inline-start: 0; z-index: 100;
		display: flex; flex-wrap: wrap; gap: 6px; padding: 10px;
		background: var(--background-primary, #fff); border: 1px solid var(--border, #e0e0e0);
		border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12); max-width: 350px; margin-top: 4px;
	}
	.cmap-chip {
		padding: 4px 10px; border-radius: 6px; border: 1.5px solid var(--border, #d1d5db);
		background: var(--background-secondary, #f9fafb); color: var(--text-normal, #374151);
		font-size: 11px; font-weight: 500; cursor: pointer; white-space: nowrap; font-family: inherit;
	}
	.cmap-chip:hover { border-color: var(--interactive-accent, #7c3aed); color: var(--interactive-accent, #7c3aed); }

	/* Settings panel */
	.cmap-settings {
		display: flex; gap: 16px; padding: 8px 20px; flex-shrink: 0;
		border-bottom: 1px solid var(--border, #e0e0e0); align-items: center;
	}
	.cmap-settings-slider { display: flex; flex-direction: column; gap: 2px; min-width: 120px; }
	.cmap-settings-slider span { font-size: 10px; color: var(--text-muted, #888); }
	.cmap-settings-slider input[type="range"] { width: 100%; height: 14px; cursor: pointer; }

	.cmap-breadcrumb {
		display: flex; align-items: center; gap: 4px; padding: 6px 20px;
		border-bottom: 1px solid var(--border, #e0e0e0); flex-shrink: 0;
		font-size: 12px; color: var(--text-muted, #888);
	}
	.cmap-bc-item {
		background: none; border: none; cursor: pointer; font-size: 12px;
		color: var(--interactive-accent, #7c3aed); padding: 2px 4px; border-radius: 3px;
	}
	.cmap-bc-item:hover { background: var(--background-modifier-hover, #f0f0f0); }
	.cmap-bc-item.active { font-weight: 700; color: var(--text-normal, #333); cursor: default; }
	.cmap-bc-sep { color: var(--text-faint, #ccc); }

	.cmap-content-wrap { flex: 1; display: flex; overflow: hidden; }
	.cmap-body { flex: 1; position: relative; overflow: hidden; }

	/* Search results sidebar */
	.cmap-results {
		flex-shrink: 0; display: flex; flex-direction: column;
		background: var(--background-primary, #fff); overflow: hidden;
	}
	.cmap-resize-handle {
		width: 4px; flex-shrink: 0; cursor: col-resize;
		background: var(--border, #e0e0e0);
	}
	.cmap-resize-handle:hover { background: var(--interactive-accent, #7c3aed); }
	.cmap-results-header {
		padding: 8px 12px; font-size: 11px; font-weight: 700;
		color: var(--text-muted, #888); border-bottom: 1px solid var(--border, #e0e0e0);
		flex-shrink: 0;
	}
	.cmap-results-list { flex: 1; overflow-y: auto; }
	.cmap-result-row {
		display: flex; align-items: center; gap: 6px; width: 100%;
		padding: 6px 12px; border: none; background: none; cursor: pointer;
		font-size: 11px; color: var(--text-normal, #333); font-family: inherit;
		text-align: start; border-bottom: 1px solid var(--background-modifier-border, #f0f0f0);
	}
	.cmap-result-row:hover { background: var(--background-modifier-hover, #f5f5f5); }
	.cmap-result-row.active { background: var(--background-modifier-active-hover, #e8e8ff); font-weight: 600; }
	.cmap-result-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.cmap-result-badges { display: flex; gap: 2px; flex-shrink: 0; }
	.cmap-result-badge {
		font-size: 8px; color: #fff; padding: 1px 4px; border-radius: 3px;
		font-weight: 700; line-height: 1.2;
	}
	.cmap-svg { width: 100%; height: 100%; }

	.cmap-loading {
		position: absolute; inset: 0; display: flex; flex-direction: column;
		align-items: center; justify-content: center; gap: 12px;
	}
	.cmap-spinner {
		width: 32px; height: 32px; border: 3px solid var(--border, #ddd);
		border-top-color: var(--interactive-accent, #7c3aed);
		border-radius: 50%; animation: cmap-spin 0.8s linear infinite;
	}
	@keyframes cmap-spin { to { transform: rotate(360deg); } }
	.cmap-loading p { font-size: 13px; color: var(--text-muted, #888); }
	.cmap-error { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; color: var(--text-error, #e53935); }

	.cmap-tooltip {
		position: fixed; z-index: 1000; padding: 8px 12px; border-radius: 8px;
		background: var(--background-primary, #fff); border: 1px solid var(--border, #ddd);
		box-shadow: 0 4px 12px rgba(0,0,0,0.15); font-size: 12px;
		pointer-events: none; max-width: 250px;
	}
	.cmap-tt-name { font-weight: 700; margin-bottom: 2px; color: var(--text-normal, #333); }
	.cmap-tt-type { font-size: 10px; color: var(--interactive-accent, #7c3aed); font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
	.cmap-tt-row { color: var(--text-muted, #666); display: flex; align-items: center; gap: 4px; }
	.cmap-tt-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }

	.cmap-legend {
		display: flex; gap: 12px; padding: 8px 20px; flex-shrink: 0;
		border-top: 1px solid var(--border, #e0e0e0); flex-wrap: wrap;
		justify-content: center;
	}
	.cmap-legend-title { font-size: 10px; font-weight: 700; color: var(--text-normal, #333); text-transform: uppercase; letter-spacing: 0.5px; }
	.cmap-legend-item { display: flex; align-items: center; gap: 4px; font-size: 11px; color: var(--text-muted, #888); }
	.cmap-legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
</style>

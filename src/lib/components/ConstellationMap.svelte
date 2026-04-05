<script lang="ts">
	/**
	 * ConstellationMap — CE Layer 2.
	 * Radial sunburst visualization of knowledge structure, density, and maturity.
	 * Inspired by Goalscape. Uses D3.js d3.partition() for layout.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import * as d3 from 'd3';

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
	let colorMode = $state<'maturity' | 'stratum' | 'library'>('maturity');
	let breadcrumb = $state<{ name: string; node: any }[]>([]);
	let tooltip = $state<{ x: number; y: number; node: MapNode; visible: boolean }>({ x: 0, y: 0, node: null as any, visible: false });

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
			if (m === 'seed') return 0.4;
			if (m === 'sapling') return 0.6;
			if (m === 'evergreen') return 0.8;
			if (m === 'canonical') return 0.95;
			if (m === 'wilting') return 0.5;
		}
		return 0.75;
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
				const data = d.data as MapNode;
				tooltip = {
					x: event.clientX,
					y: event.clientY,
					node: data,
					visible: true,
				};
				// Highlight
				d3.select(event.currentTarget as Element).attr('fill-opacity', 1).attr('stroke-width', 2);
			})
			.on('mousemove', (event: MouseEvent) => {
				tooltip.x = event.clientX;
				tooltip.y = event.clientY;
			})
			.on('mouseleave', (event: MouseEvent, d: any) => {
				tooltip.visible = false;
				d3.select(event.currentTarget as Element)
					.attr('fill-opacity', (d: any) => getNodeOpacity(d))
					.attr('stroke-width', 0.5);
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
					maxDepth: 5,
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

	// Re-render when color mode changes
	$effect(() => {
		colorMode; // track
		if (mapData && svgEl) {
			const current = breadcrumb.length > 0 ? breadcrumb[breadcrumb.length - 1].node.data : mapData;
			renderSunburst(current);
			onColorModeChange?.(colorMode);
		}
	});

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

	<!-- Content -->
	<div class="cmap-body" bind:this={containerEl}>
		{#if loading}
			<div class="cmap-loading">
				<div class="cmap-spinner"></div>
				<p>{$t('constellationMap.loading') || 'Building knowledge map...'}</p>
			</div>
		{:else if error}
			<div class="cmap-error">
				<p>{error}</p>
			</div>
		{:else}
			<svg bind:this={svgEl} class="cmap-svg"></svg>
		{/if}
	</div>

	<!-- Tooltip -->
	{#if tooltip.visible && tooltip.node}
		<div class="cmap-tooltip" style="left:{tooltip.x + 12}px;top:{tooltip.y - 8}px" dir="auto">
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
			{#each Object.entries(MATURITY_COLORS) as [label, color]}
				<span class="cmap-legend-item"><span class="cmap-legend-dot" style="background:{color}"></span>{label}</span>
			{/each}
		{:else if colorMode === 'stratum'}
			{#each STRATUM_COLORS as color, i}
				<span class="cmap-legend-item"><span class="cmap-legend-dot" style="background:{color}"></span>L{i + 1}</span>
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
	.cmap-close {
		width: 32px; height: 32px; border-radius: 50%; border: 1px solid var(--border, #ddd);
		background: transparent; color: var(--text-muted, #888); font-size: 18px;
		cursor: pointer; display: flex; align-items: center; justify-content: center;
	}
	.cmap-close:hover { background: var(--border, #eee); color: var(--text-normal, #333); }

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

	.cmap-body { flex: 1; position: relative; overflow: hidden; }
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
	.cmap-legend-item { display: flex; align-items: center; gap: 4px; font-size: 11px; color: var(--text-muted, #888); }
	.cmap-legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
</style>

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Graph from 'graphology';
	import Sigma from 'sigma';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import type { StarNode, StarLink } from '$lib/libraries/store';
	import type { GraphMindSettings } from '$lib/graph/types';

	let {
		nodes = [] as StarNode[],
		links = [] as StarLink[],
		onNodeClick = undefined as ((path: string, libraryName: string) => void) | undefined,
		activeNodeId = '',
		skyViewSettings,
		libraryColorMap = {} as Record<string, string>,
	}: {
		nodes: StarNode[];
		links: StarLink[];
		onNodeClick?: (path: string, libraryName: string) => void;
		activeNodeId?: string;
		skyViewSettings?: GraphMindSettings;
		libraryColorMap?: Record<string, string>;
	} = $props();

	const DEFAULTS: GraphMindSettings = {
		nodeSize: 4,
		labelVisibility: 'hover',
		labelFontSize: 12,
		linkThickness: 1,
		repelForce: 80,
		linkForce: 0.05,
		linkDistance: 30,
		showOrphans: true,
		colorByLibrary: true,
	};

	const DEFAULT_NODE_COLOR = '#7c3aed';
	const HIGHLIGHT_EDGE_COLOR = '#ffffff';
	const DIM_COLOR = '#1a1a2e';
	const DIM_EDGE_COLOR = '#0a0a1a';

	let containerEl: HTMLDivElement;
	let sigma: Sigma | null = null;
	let graph: Graph | null = null;
	let worker: Worker | null = null;
	let hoveredNode: string | null = $state(null);
	let highlightedNeighbors: Set<string> = new Set();
	let searchQuery = $state('');
	let searchVisible = $state(false);
	let showSettings = $state(false);
	let layoutSettled = false;

	// Local settings copy for inline controls
	let localSettings = $state({ ...DEFAULTS, ...skyViewSettings });

	// Settings panel controls
	let settingsPanel: 'appearance' | 'physics' = $state('appearance');

	function getSettings(): GraphMindSettings {
		return { ...DEFAULTS, ...skyViewSettings, ...localSettings };
	}

	function isRTL(text: string): boolean {
		// Check for Arabic/Hebrew characters
		return /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/.test(text);
	}

	function buildGraph() {
		const g = new Graph({ multi: false, type: 'directed', allowSelfLoops: false });
		const settings = getSettings();

		const filteredNodes = settings.showOrphans
			? nodes
			: nodes.filter((n) => n.linkCount > 0);

		const nodeIdSet = new Set(filteredNodes.map((n) => n.id));

		// Add nodes
		for (const node of filteredNodes) {
			const size = (2 + Math.sqrt(node.linkCount) * 1.5) * (settings.nodeSize / 4);
			const color = settings.colorByLibrary
				? (libraryColorMap[node.libraryName] || DEFAULT_NODE_COLOR)
				: DEFAULT_NODE_COLOR;

			g.addNode(node.id, {
				label: node.name,
				size,
				color,
				x: (Math.random() - 0.5) * 1000,
				y: (Math.random() - 0.5) * 1000,
				// Custom attributes
				path: node.path,
				libraryName: node.libraryName,
				linkCount: node.linkCount,
				outgoingCount: node.outgoingCount,
				isRTL: isRTL(node.name),
			});
		}

		// Add edges
		for (const link of links) {
			if (!nodeIdSet.has(link.source) || !nodeIdSet.has(link.target)) continue;
			if (link.source === link.target) continue;
			// Avoid duplicate edges
			const edgeKey = `${link.source}->${link.target}`;
			if (g.hasEdge(edgeKey)) continue;

			g.addEdgeWithKey(edgeKey, link.source, link.target, {
				size: settings.linkThickness,
				color: '#334155',
				linkType: link.linkType,
			});
		}

		return g;
	}

	function startForceWorker() {
		if (worker) {
			worker.terminate();
			worker = null;
		}
		if (!graph || graph.order === 0) return;

		const settings = getSettings();

		try {
			worker = new Worker(
				new URL('$lib/graph/forceWorker.ts', import.meta.url),
				{ type: 'module' }
			);
		} catch {
			// Fallback: inline simple force layout
			applySimpleLayout();
			return;
		}

		const workerNodes = graph.mapNodes((id, attrs) => ({
			id,
			x: attrs.x,
			y: attrs.y,
		}));
		const workerEdges = graph.mapEdges((_, attrs, source, target) => ({
			source,
			target,
		}));

		worker.onmessage = (e: MessageEvent) => {
			if (e.data.type === 'positions' && graph && sigma) {
				const positions = e.data.positions as Float64Array;
				const nodeIds = graph.nodes();
				for (let i = 0; i < nodeIds.length && i * 2 + 1 < positions.length; i++) {
					graph.setNodeAttribute(nodeIds[i], 'x', positions[i * 2]);
					graph.setNodeAttribute(nodeIds[i], 'y', positions[i * 2 + 1]);
				}
				sigma.refresh();

				// Auto-fit on early iterations
				if (!layoutSettled && e.data.settled) {
					layoutSettled = true;
					fitToScreen();
				}
			}
		};

		worker.postMessage({
			type: 'init',
			nodes: workerNodes,
			edges: workerEdges,
			settings: {
				repelForce: settings.repelForce,
				linkForce: settings.linkForce,
				linkDistance: settings.linkDistance,
				centerForce: 0.1,
			},
		});
	}

	function applySimpleLayout() {
		// Fallback: circular layout
		if (!graph) return;
		const nodeIds = graph.nodes();
		const radius = Math.max(200, nodeIds.length * 3);
		nodeIds.forEach((id, i) => {
			const angle = (2 * Math.PI * i) / nodeIds.length;
			graph!.setNodeAttribute(id, 'x', Math.cos(angle) * radius);
			graph!.setNodeAttribute(id, 'y', Math.sin(angle) * radius);
		});
		sigma?.refresh();
		setTimeout(fitToScreen, 100);
	}

	function fitToScreen() {
		if (!sigma || !graph || graph.order === 0) return;
		const camera = sigma.getCamera();
		camera.animatedReset({ duration: 300 });
	}

	function initSigma() {
		if (!containerEl) return;
		if (sigma) {
			sigma.kill();
			sigma = null;
		}

		graph = buildGraph();
		if (graph.order === 0) return;

		const settings = getSettings();
		const dir = detectDir(nodes[0]?.name ?? '');

		sigma = new Sigma(graph, containerEl, {
			renderLabels: settings.labelVisibility !== 'none',
			labelRenderedSizeThreshold: settings.labelVisibility === 'always' ? 0 : 8,
			labelSize: settings.labelFontSize,
			labelColor: { color: '#e2e8f0' },
			labelFont: 'system-ui, -apple-system, sans-serif',
			defaultEdgeColor: '#334155',
			defaultNodeColor: DEFAULT_NODE_COLOR,
			edgeLabelSize: 10,
			minCameraRatio: 0.02,
			maxCameraRatio: 20,
			// Node & edge rendering reducers for hover/search highlighting
			nodeReducer: (nodeId, data) => {
				const res = { ...data };

				// Active node glow
				if (nodeId === activeNodeId) {
					res.color = '#ffffff';
					res.size = (data.size ?? 5) * 1.3;
				}

				// Hover highlighting
				if (hoveredNode) {
					if (nodeId === hoveredNode) {
						res.size = (data.size ?? 5) * 1.5;
						res.zIndex = 10;
					} else if (highlightedNeighbors.has(nodeId)) {
						res.zIndex = 5;
					} else {
						res.color = DIM_COLOR;
						res.label = '';
					}
				}

				// Search highlighting
				if (searchQuery && searchVisible) {
					const q = searchQuery.toLowerCase();
					const label = (data.label ?? '').toLowerCase();
					if (!label.includes(q)) {
						if (!hoveredNode) {
							res.color = DIM_COLOR;
							res.label = '';
						}
					} else {
						res.highlighted = true;
						res.zIndex = 10;
					}
				}

				return res;
			},
			edgeReducer: (edgeId, data) => {
				const res = { ...data };

				if (hoveredNode && graph) {
					const source = graph.source(edgeId);
					const target = graph.target(edgeId);
					if (source === hoveredNode || target === hoveredNode) {
						res.color = HIGHLIGHT_EDGE_COLOR;
						res.size = (data.size ?? 1) * 2;
						res.zIndex = 10;
					} else {
						res.color = DIM_EDGE_COLOR;
						res.hidden = true;
					}
				}

				if (searchQuery && searchVisible && graph) {
					const q = searchQuery.toLowerCase();
					const source = graph.source(edgeId);
					const target = graph.target(edgeId);
					const sourceLabel = (graph.getNodeAttribute(source, 'label') ?? '').toLowerCase();
					const targetLabel = (graph.getNodeAttribute(target, 'label') ?? '').toLowerCase();
					if (!sourceLabel.includes(q) && !targetLabel.includes(q)) {
						res.hidden = true;
					}
				}

				return res;
			},
		});

		// --- Event handlers ---

		// Hover
		sigma.on('enterNode', ({ node }) => {
			hoveredNode = node;
			highlightedNeighbors = new Set(graph!.neighbors(node));
			sigma?.refresh();
			containerEl.style.cursor = 'pointer';
		});

		sigma.on('leaveNode', () => {
			hoveredNode = null;
			highlightedNeighbors.clear();
			sigma?.refresh();
			containerEl.style.cursor = 'grab';
		});

		// Click: open note
		sigma.on('clickNode', ({ node }) => {
			if (!graph) return;
			const attrs = graph.getNodeAttributes(node);
			onNodeClick?.(attrs.path, attrs.libraryName);
		});

		// Double-click: focus on node
		sigma.on('doubleClickNode', ({ node }) => {
			if (!graph || !sigma) return;
			const attrs = graph.getNodeAttributes(node);
			const camera = sigma.getCamera();
			camera.animate({ x: attrs.x, y: attrs.y, ratio: 0.3 }, { duration: 300 });
		});

		// Right-click: future context menu
		sigma.on('rightClickNode', ({ node, event }) => {
			event.original.preventDefault();
			// TODO Phase 3: context menu
		});

		// Drag behavior
		let draggedNode: string | null = null;
		let isDragging = false;

		sigma.on('downNode', ({ node, event }) => {
			draggedNode = node;
			isDragging = false;
			// Disable camera panning while dragging
			sigma!.getCamera().disable();
		});

		sigma.getMouseCaptor().on('mousemovebody', (e: any) => {
			if (!draggedNode || !sigma || !graph) return;
			isDragging = true;

			// Get new position from viewport coordinates
			const pos = sigma.viewportToGraph(e);
			graph.setNodeAttribute(draggedNode, 'x', pos.x);
			graph.setNodeAttribute(draggedNode, 'y', pos.y);

			// Pin node in worker
			worker?.postMessage({ type: 'pinNode', id: draggedNode, x: pos.x, y: pos.y });

			sigma.refresh();
		});

		sigma.getMouseCaptor().on('mouseup', () => {
			if (draggedNode) {
				sigma?.getCamera().enable();
				if (!isDragging) {
					// Was a click, not a drag — unpin
					worker?.postMessage({ type: 'unpinNode', id: draggedNode });
				}
				draggedNode = null;
				isDragging = false;
			}
		});

		containerEl.style.cursor = 'grab';

		// Start force layout
		layoutSettled = false;
		startForceWorker();
	}

	// Keyboard shortcuts
	function handleKeydown(e: KeyboardEvent) {
		// Ctrl+F — search
		if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
			e.preventDefault();
			searchVisible = !searchVisible;
			if (!searchVisible) {
				searchQuery = '';
				sigma?.refresh();
			}
		}
		// Escape — close search or settings
		if (e.key === 'Escape') {
			if (searchVisible) {
				searchVisible = false;
				searchQuery = '';
				sigma?.refresh();
			}
			if (showSettings) showSettings = false;
		}
	}

	// Search reactivity
	$effect(() => {
		if (searchQuery !== undefined && sigma) {
			sigma.refresh();
		}
	});

	// Re-init when data changes
	$effect(() => {
		const _n = nodes.length;
		const _l = links.length;
		if (containerEl && nodes.length > 0) {
			initSigma();
		}
	});

	// Settings change → update graph
	$effect(() => {
		const s = localSettings;
		if (!graph || !sigma) return;

		// Update node sizes
		const sizeMul = s.nodeSize / 4;
		graph.forEachNode((id, attrs) => {
			const lc = attrs.linkCount ?? 0;
			graph!.setNodeAttribute(id, 'size', (2 + Math.sqrt(lc) * 1.5) * sizeMul);
		});

		// Update edge thickness
		graph.forEachEdge((id) => {
			graph!.setEdgeAttribute(id, 'size', s.linkThickness);
		});

		// Update sigma settings
		sigma.setSetting('renderLabels', s.labelVisibility !== 'none');
		sigma.setSetting('labelRenderedSizeThreshold', s.labelVisibility === 'always' ? 0 : 8);
		sigma.setSetting('labelSize', s.labelFontSize);

		sigma.refresh();

		// Update worker physics
		worker?.postMessage({
			type: 'updateSettings',
			settings: {
				repelForce: s.repelForce,
				linkForce: s.linkForce,
				linkDistance: s.linkDistance,
				centerForce: 0.1,
			},
		});
	});

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
		worker?.terminate();
		worker = null;
		sigma?.kill();
		sigma = null;
		graph = null;
	});
</script>

<div class="graphmind-container">
	<!-- Toolbar -->
	<div class="gm-toolbar" dir="auto">
		<div class="gm-toolbar-left">
			<button class="gm-btn" class:active={searchVisible} title="{$t('layout.search')} (Ctrl+F)" onclick={() => { searchVisible = !searchVisible; if (!searchVisible) { searchQuery = ''; sigma?.refresh(); } }}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
			</button>
			{#if searchVisible}
				<input
					class="gm-search"
					type="text"
					dir="auto"
					placeholder={$t('layout.search')}
					bind:value={searchQuery}
					autofocus
				/>
			{/if}
		</div>
		<div class="gm-toolbar-right">
			<button class="gm-btn" title="Fit to screen" onclick={fitToScreen}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="M9 21H3v-6"/><path d="M21 3l-7 7"/><path d="M3 21l7-7"/></svg>
			</button>
			<button class="gm-btn" class:active={showSettings} title="Settings" onclick={() => showSettings = !showSettings}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
			</button>
		</div>
	</div>

	<!-- Settings panel -->
	{#if showSettings}
		<div class="gm-settings" dir="auto">
			<div class="gm-settings-tabs">
				<button class="gm-tab" class:active={settingsPanel === 'appearance'} onclick={() => settingsPanel = 'appearance'}>
					{$t('settings.skyview.graphAppearance') || 'Appearance'}
				</button>
				<button class="gm-tab" class:active={settingsPanel === 'physics'} onclick={() => settingsPanel = 'physics'}>
					{$t('settings.skyview.physics') || 'Physics'}
				</button>
			</div>

			{#if settingsPanel === 'appearance'}
				<label class="gm-setting">
					<span>{$t('settings.skyview.nodeSize') || 'Node size'}</span>
					<input type="range" min="1" max="10" step="0.5" bind:value={localSettings.nodeSize} />
					<span class="gm-val">{localSettings.nodeSize}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.labelVisibility') || 'Labels'}</span>
					<select bind:value={localSettings.labelVisibility}>
						<option value="hover">{$t('settings.skyview.labelHover') || 'On hover'}</option>
						<option value="always">{$t('settings.skyview.labelAlways') || 'Always'}</option>
						<option value="none">{$t('settings.skyview.labelNone') || 'None'}</option>
					</select>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.labelFontSize') || 'Label size'}</span>
					<input type="range" min="8" max="24" step="1" bind:value={localSettings.labelFontSize} />
					<span class="gm-val">{localSettings.labelFontSize}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.linkThickness') || 'Link width'}</span>
					<input type="range" min="0.5" max="5" step="0.5" bind:value={localSettings.linkThickness} />
					<span class="gm-val">{localSettings.linkThickness}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.showOrphans') || 'Show orphans'}</span>
					<input type="checkbox" bind:checked={localSettings.showOrphans} />
				</label>
			{:else}
				<label class="gm-setting">
					<span>{$t('settings.skyview.repelForce') || 'Repulsion'}</span>
					<input type="range" min="10" max="300" step="5" bind:value={localSettings.repelForce} />
					<span class="gm-val">{localSettings.repelForce}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.linkForce') || 'Link force'}</span>
					<input type="range" min="0.01" max="0.3" step="0.01" bind:value={localSettings.linkForce} />
					<span class="gm-val">{localSettings.linkForce.toFixed(2)}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.linkDistance') || 'Link distance'}</span>
					<input type="range" min="10" max="300" step="5" bind:value={localSettings.linkDistance} />
					<span class="gm-val">{localSettings.linkDistance}</span>
				</label>
				<button class="gm-btn gm-reset" onclick={() => {
					worker?.postMessage({ type: 'restart' });
					layoutSettled = false;
				}}>
					Reheat simulation
				</button>
			{/if}
		</div>
	{/if}

	<!-- Stats bar -->
	<div class="gm-stats" dir="auto">
		<span>{graph?.order ?? 0} {$t('graphView.nodes') || 'nodes'}</span>
		<span class="gm-sep">·</span>
		<span>{graph?.size ?? 0} {$t('graphView.edges') || 'edges'}</span>
		{#if hoveredNode && graph}
			<span class="gm-sep">·</span>
			<span class="gm-hovered" dir="auto">{graph.getNodeAttribute(hoveredNode, 'label')}</span>
		{/if}
	</div>

	<!-- Legend -->
	{#if Object.keys(libraryColorMap).length > 1}
		<div class="gm-legend" dir="auto">
			{#each Object.entries(libraryColorMap) as [name, color]}
				<div class="gm-legend-item">
					<span class="gm-legend-dot" style="background:{color}"></span>
					<span class="gm-legend-name" dir="auto">{name}</span>
				</div>
			{/each}
		</div>
	{/if}

	<!-- WebGL canvas container -->
	<div class="gm-renderer" bind:this={containerEl}></div>
</div>

<style>
	.graphmind-container {
		position: relative;
		width: 100%;
		height: 100%;
		overflow: hidden;
		background: var(--background-primary, #0f0f1a);
	}

	.gm-renderer {
		width: 100%;
		height: 100%;
	}
	.gm-renderer :global(canvas) {
		display: block;
	}

	/* Toolbar */
	.gm-toolbar {
		position: absolute;
		top: 8px;
		left: 8px;
		right: 8px;
		z-index: 10;
		display: flex;
		justify-content: space-between;
		align-items: center;
		pointer-events: none;
	}
	.gm-toolbar-left, .gm-toolbar-right {
		display: flex;
		gap: 4px;
		align-items: center;
		pointer-events: auto;
	}

	.gm-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 6px;
		background: rgba(15, 15, 26, 0.85);
		color: var(--text-muted, #94a3b8);
		cursor: pointer;
		backdrop-filter: blur(8px);
		transition: all 0.15s;
	}
	.gm-btn:hover { background: rgba(30, 30, 50, 0.95); color: var(--text-normal, #e2e8f0); }
	.gm-btn.active { background: var(--interactive-accent, #7c3aed); color: white; }

	.gm-search {
		height: 32px;
		padding: 0 10px;
		border: 1px solid var(--background-modifier-border, #334155);
		border-radius: 6px;
		background: rgba(15, 15, 26, 0.9);
		color: var(--text-normal, #e2e8f0);
		font-size: 13px;
		outline: none;
		min-width: 200px;
		backdrop-filter: blur(8px);
	}
	.gm-search:focus { border-color: var(--interactive-accent, #7c3aed); }

	/* Settings panel */
	.gm-settings {
		position: absolute;
		top: 48px;
		right: 8px;
		z-index: 20;
		width: 260px;
		background: rgba(15, 15, 26, 0.95);
		border: 1px solid var(--background-modifier-border, #334155);
		border-radius: 8px;
		padding: 8px;
		backdrop-filter: blur(12px);
	}
	[dir="rtl"] .gm-settings {
		right: auto;
		left: 8px;
	}
	.gm-settings-tabs {
		display: flex;
		gap: 4px;
		margin-bottom: 8px;
	}
	.gm-tab {
		flex: 1;
		padding: 5px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-muted, #94a3b8);
		font-size: 12px;
		cursor: pointer;
	}
	.gm-tab.active { background: var(--interactive-accent, #7c3aed); color: white; }

	.gm-setting {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 0;
		font-size: 12px;
		color: var(--text-muted, #94a3b8);
	}
	.gm-setting span:first-child { flex: 1; white-space: nowrap; }
	.gm-setting input[type="range"] { flex: 1; max-width: 100px; accent-color: var(--interactive-accent, #7c3aed); }
	.gm-setting select {
		background: rgba(30, 30, 50, 0.9);
		color: var(--text-normal, #e2e8f0);
		border: 1px solid var(--background-modifier-border, #334155);
		border-radius: 4px;
		padding: 2px 6px;
		font-size: 12px;
	}
	.gm-setting input[type="checkbox"] { accent-color: var(--interactive-accent, #7c3aed); }
	.gm-val { width: 30px; text-align: end; font-variant-numeric: tabular-nums; }

	.gm-reset {
		width: 100%;
		margin-top: 8px;
		font-size: 12px;
		padding: 6px;
	}

	/* Stats bar */
	.gm-stats {
		position: absolute;
		bottom: 8px;
		left: 8px;
		z-index: 10;
		display: flex;
		gap: 6px;
		align-items: center;
		font-size: 11px;
		color: var(--text-faint, #64748b);
		background: rgba(15, 15, 26, 0.8);
		padding: 4px 10px;
		border-radius: 6px;
		backdrop-filter: blur(8px);
	}
	[dir="rtl"] .gm-stats {
		left: auto;
		right: 8px;
	}
	.gm-sep { opacity: 0.3; }
	.gm-hovered { color: var(--text-normal, #e2e8f0); font-weight: 500; }

	/* Legend */
	.gm-legend {
		position: absolute;
		bottom: 8px;
		right: 8px;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 3px;
		background: rgba(15, 15, 26, 0.8);
		padding: 6px 10px;
		border-radius: 6px;
		backdrop-filter: blur(8px);
	}
	[dir="rtl"] .gm-legend {
		right: auto;
		left: 8px;
	}
	[dir="rtl"] .gm-stats {
		left: auto;
		right: 8px;
	}
	.gm-legend-item {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--text-muted, #94a3b8);
	}
	.gm-legend-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.gm-legend-name {
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>

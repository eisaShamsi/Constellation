<script lang="ts">
	/**
	 * GraphMind — Layer 1: Thin Svelte Wrapper
	 *
	 * This component owns ONLY:
	 *   - UI controls (settings panel, search bar, toolbar)
	 *   - Stats display (node count, edge count, hovered name)
	 *
	 * It does NOT own:
	 *   - Node positions (owned by GraphEngine)
	 *   - Hover state (owned by GraphEngine)
	 *   - Simulation state (owned by forceWorker)
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t, isRTL as isRTLStore } from '$lib/i18n';
	import { GraphEngine, type EngineConfig, type LayoutMode } from '$lib/graph/graphEngine';
	import type { StarNode, StarLink } from '$lib/libraries/store';
	import { computeSemanticLinks, type EmbeddingProgress } from '$lib/graph/semanticEngine';
	import { detectClusters, type ClusterResult } from '$lib/graph/clusterEngine';
	import { invoke } from '@tauri-apps/api/core';

	// RTL-aware directional symbols
	const arrowIncoming = $derived($isRTLStore ? '→' : '←');
	const arrowOutgoing = $derived($isRTLStore ? '←' : '→');
	const breadcrumbSep = $derived($isRTLStore ? '‹' : '›');

	const DEFAULTS: EngineConfig = {
		nodeSize: 1.5,
		labelVisibility: 'hover',
		labelFontSize: 12,
		linkThickness: 1,
		repelForce: 80,
		linkForce: 0.05,
		linkDistance: 30,
		showOrphans: true,
		colorByLibrary: true,
		layoutMode: 'organic',
		showSemanticLinks: false,
		semanticThreshold: 0.5,
	};

	let {
		nodes = [] as StarNode[],
		links = [] as StarLink[],
		onNodeClick = undefined as ((path: string, libraryName: string) => void) | undefined,
		onNodeHover = undefined as ((node: { name: string; path: string; libraryName: string } | null) => void) | undefined,
		activeNodeId = '',
		highlightPath = null as string | string[] | null,
		highlightColor = 0x7c3aed,
		skyViewSettings,
		libraryColorMap = {} as Record<string, string>,
		lensCentrality = null as Map<string, number> | null,
		lensCommunityAssignments = null as Map<string, number> | null,
		lensCommunityColors = null as Map<number, string> | null,
		lensGaps = [] as { community1: number; community2: number }[],
	}: {
		nodes: StarNode[];
		links: StarLink[];
		onNodeClick?: (path: string, libraryName: string) => void;
		onNodeHover?: (node: { name: string; path: string; libraryName: string } | null) => void;
		activeNodeId?: string;
		highlightPath?: string | string[] | null;
		highlightColor?: number;
		skyViewSettings?: Partial<EngineConfig>;
		libraryColorMap?: Record<string, string>;
		lensCentrality?: Map<string, number> | null;
		lensCommunityAssignments?: Map<string, number> | null;
		lensCommunityColors?: Map<number, string> | null;
		lensGaps?: { community1: number; community2: number }[];
	} = $props();

	// ─── Layer 1 state: UI only ─────────────────────────────
	let settingsOpen = $state(false);
	let settingsTab: 'appearance' | 'physics' | 'intelligence' = $state('appearance');
	let searchVisible = $state(false);
	let searchQuery = $state('');
	let hoveredName = $state<string | null>(null);
	let nodeCount = $state(0);
	let edgeCount = $state(0);
	let mocCount = $state(0);

	// Context menu
	let contextMenu = $state<{ node: { id: string; name: string; path: string; libraryName: string }; x: number; y: number } | null>(null);

	// Focus mode
	let focusActive = $state(false);
	let focusNodeName = $state('');
	let focusDepth = $state(2);

	// Local graph mode
	let localGraph = $state(false);

	// Layout mode
	let layoutMode = $state<LayoutMode>('organic');

	// Focus direction
	let focusDirection = $state<'all' | 'incoming' | 'outgoing'>('all');

	// Navigation breadcrumb
	let breadcrumb = $state<{ id: string; name: string }[]>([]);

	// Tilt state
	let isTilted = $state(false);

	// Semantic links (Phase 2)
	let semanticComputing = $state(false);
	let semanticProgress = $state('');
	let semanticLinkCount = $state(0);
	let uiShowSemanticLinks = $state(false);
	let uiSemanticThreshold = $state(0.5);
	let clusterResult = $state<ClusterResult | null>(null);
	let showClusters = $state(false);

	// Color-by mode for legend
	let colorBy = $state<'library' | 'folder' | 'tag'>('library');

	// Hidden groups — separate sets for library and folder modes
	let hiddenLibraries = $state(new Set<string>());
	let hiddenFolders = $state(new Set<string>());

	// Effective hidden groups: current mode's set + library→folder cascade
	const hiddenGroups = $derived.by(() => {
		const set = new Set(colorBy === 'folder' ? hiddenFolders : hiddenLibraries);
		// Cascade: hidden libraries → hide their folders too
		if (colorBy === 'folder') {
			for (const lib of hiddenLibraries) {
				const folders = libraryFolderMap[lib];
				if (folders) for (const f of folders) set.add(f);
			}
		}
		return set;
	});

	// Visible folder count (only folders from visible libraries)
	const visibleFolderCount = $derived.by(() => {
		let count = 0;
		for (const [lib, folders] of Object.entries(libraryFolderMap)) {
			if (!hiddenLibraries.has(lib)) count += folders.size;
		}
		return count;
	});

	// Hidden count
	let hiddenCount = $state(0);

	// ─── NOT $state: plain JS config (Law 3) ─────────────────
	let engineConfig: EngineConfig = { ...DEFAULTS, ...skyViewSettings };

	// Local copies for settings UI (these ARE $state for input binding)
	let uiNodeSize = $state(engineConfig.nodeSize);
	let uiLabelVisibility = $state(engineConfig.labelVisibility);
	let uiLabelFontSize = $state(engineConfig.labelFontSize);
	let uiLinkThickness = $state(engineConfig.linkThickness);
	let uiShowOrphans = $state(engineConfig.showOrphans);
	let uiRepelForce = $state(engineConfig.repelForce);
	let uiLinkForce = $state(engineConfig.linkForce);
	let uiLinkDistance = $state(engineConfig.linkDistance);

	const GROUP_COLORS = [
		'#a78bfa', '#34d399', '#60a5fa', '#f472b6', '#fbbf24',
		'#f87171', '#2dd4bf', '#818cf8', '#fb923c', '#a3e635',
		'#e879f9', '#38bdf8', '#facc15', '#4ade80', '#f43f5e',
	];

	// Compute folder-based color map from note paths
	const folderColorMap = $derived.by(() => {
		const map: Record<string, string> = {};
		const folders = new Set<string>();
		for (const n of nodes) {
			const parts = n.path.replace(/\\/g, '/').split('/');
			const folder = parts.length >= 3 ? parts[parts.length - 2] : '(root)';
			folders.add(folder);
		}
		const sorted = [...folders].sort();
		sorted.forEach((f, i) => { map[f] = GROUP_COLORS[i % GROUP_COLORS.length]; });
		return map;
	});

	// Map: library name → set of folder names belonging to that library
	const libraryFolderMap = $derived.by(() => {
		const map: Record<string, Set<string>> = {};
		for (const n of nodes) {
			const parts = n.path.replace(/\\/g, '/').split('/');
			const folder = parts.length >= 3 ? parts[parts.length - 2] : '(root)';
			if (!map[n.libraryName]) map[n.libraryName] = new Set();
			map[n.libraryName].add(folder);
		}
		return map;
	});

	// Get the folder for a node
	function getNodeFolder(path: string): string {
		const parts = path.replace(/\\/g, '/').split('/');
		return parts.length >= 3 ? parts[parts.length - 2] : '(root)';
	}

	// Active color map based on colorBy mode
	const activeColorMap = $derived.by(() => {
		if (colorBy === 'folder') return folderColorMap;
		return libraryColorMap; // 'library' default
	});

	let containerEl: HTMLDivElement;
	let engine: GraphEngine | null = null;

	function handleSettingChange(key: keyof EngineConfig, value: any) {
		(engineConfig as any)[key] = value;
		engine?.updateConfig({ [key]: value });
	}

	/** Compute semantic links for all notes (Phase 2) */
	async function computeSemantic() {
		if (semanticComputing || nodes.length < 2) return;
		semanticComputing = true;
		semanticProgress = 'Loading AI model...';

		try {
			// Read note contents from disk via Tauri
			const noteContents: { id: string; name: string; content: string }[] = [];
			for (const n of nodes) {
				try {
					const content: string = await invoke('read_file', { path: n.path });
					noteContents.push({ id: n.id, name: n.name, content });
				} catch {
					noteContents.push({ id: n.id, name: n.name, content: '' });
				}
			}

			const semanticResults = await computeSemanticLinks(
				noteContents,
				links,
				uiSemanticThreshold,
				500,
				(p: EmbeddingProgress) => {
					if (p.stage === 'loading-model') semanticProgress = 'Loading AI model...';
					else if (p.stage === 'embedding') semanticProgress = `Embedding notes: ${p.current}/${p.total}`;
					else if (p.stage === 'computing-links') semanticProgress = 'Computing similarities...';
					else semanticProgress = '';
				}
			);

			semanticLinkCount = semanticResults.length;
			engine?.setSemanticLinks(semanticResults);

			// Auto-enable display
			uiShowSemanticLinks = true;
			handleSettingChange('showSemanticLinks', true);
		} catch (err) {
			semanticProgress = `Error: ${err}`;
			setTimeout(() => { semanticProgress = ''; }, 5000);
		} finally {
			semanticComputing = false;
		}
	}

	/** Detect note clusters using Louvain algorithm (Phase 2) */
	function runClusterDetection() {
		if (nodes.length < 3) return;
		const result = detectClusters(
			nodes.map(n => ({ id: n.id, name: n.name })),
			links
		);
		clusterResult = result;
		showClusters = true;

		// Send to engine
		const colorMap = new Map<number, string>();
		for (const c of result.clusters) {
			colorMap.set(c.id, c.color);
		}
		engine?.setClusters(result.assignments, colorMap);
	}

	// Keyboard shortcuts
	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
			e.preventDefault();
			searchVisible = !searchVisible;
			if (!searchVisible) searchQuery = '';
		}
		if (e.key === 'Escape') {
			// Priority: close context menu → exit focus → close search → close settings
			if (contextMenu) { contextMenu = null; return; }
			if (focusActive) { engine?.setFocusNode(null); return; }
			if (searchVisible) { searchVisible = false; searchQuery = ''; return; }
			if (settingsOpen) { settingsOpen = false; return; }
		}
		// Space — toggle local graph mode
		if (e.key === ' ' && !searchVisible && !(e.target instanceof HTMLInputElement)) {
			e.preventDefault();
			engine?.toggleLocalGraph();
			localGraph = engine?.getLocalGraphMode() ?? false;
		}
		// Ctrl+L removed — only organic layout mode used
		// 0 key — reset rotation back to 2D
		if (e.key === '0' && !(e.target instanceof HTMLInputElement) && isTilted) {
			e.preventDefault();
			engine?.resetTilt();
			isTilted = false;
		}
		// WASD / Arrow keys — fly through 3D star field (only when in 3D mode)
		if (isTilted && !(e.target instanceof HTMLInputElement) && !e.ctrlKey && !e.metaKey) {
			const speed = 15;
			switch (e.key.toLowerCase()) {
				case 'w': case 'arrowup':    e.preventDefault(); engine?.moveCamera(0, 0, speed); break;
				case 's': case 'arrowdown':  e.preventDefault(); engine?.moveCamera(0, 0, -speed); break;
				case 'a': case 'arrowleft':  e.preventDefault(); engine?.moveCamera(-speed, 0, 0); break;
				case 'd': case 'arrowright': e.preventDefault(); engine?.moveCamera(speed, 0, 0); break;
				case 'q':                    e.preventDefault(); engine?.moveCamera(0, -speed, 0); break;
				case 'e':                    e.preventDefault(); engine?.moveCamera(0, speed, 0); break;
			}
		}
	}

	// Close context menu on any click outside
	function handleGlobalClick() {
		if (contextMenu) contextMenu = null;
	}

	// Context menu actions
	function ctxOpen() {
		if (!contextMenu) return;
		onNodeClick?.(contextMenu.node.path, contextMenu.node.libraryName);
		contextMenu = null;
	}
	function ctxFocus() {
		if (!contextMenu) return;
		engine?.setFocusNode(contextMenu.node.id);
		contextMenu = null;
	}
	function ctxPin() {
		if (!contextMenu) return;
		engine?.pinNode(contextMenu.node.id);
		contextMenu = null;
	}
	function ctxHide() {
		if (!contextMenu) return;
		engine?.hideNode(contextMenu.node.id);
		contextMenu = null;
	}

	// Search → engine (one-way)
	let prevSearch = '';
	$effect(() => {
		const q = searchQuery;
		if (q !== prevSearch) {
			prevSearch = q;
			engine?.setSearch(q);
		}
	});

	// Active node → engine
	$effect(() => {
		const id = activeNodeId;
		engine?.setActiveNode(id);
	});

	// Highlight filter → engine
	$effect(() => {
		const p = highlightPath;
		const c = highlightColor;
		engine?.setHighlightFilter(p, c);
	});

	// Data changes → engine
	let prevNodeLen = 0;
	let prevColorBy = 'library';
	let prevHiddenKey = '';
	$effect(() => {
		const len = nodes.length;
		const cb = colorBy;
		const cmap = activeColorMap;
		const hiddenKey = [...hiddenGroups].sort().join(',');
		if ((len !== prevNodeLen || cb !== prevColorBy || hiddenKey !== prevHiddenKey) && len > 0 && engine) {
			prevNodeLen = len;
			prevColorBy = cb;
			prevHiddenKey = hiddenKey;

			let dataNodes = nodes;
			let dataLinks = links;

			// When colorBy is 'folder', remap nodes to use folder as their grouping key
			if (cb === 'folder') {
				dataNodes = nodes.map(n => ({ ...n, libraryName: getNodeFolder(n.path) }));
			}

			// Filter out hidden groups (with library→folder cascade)
			if (hiddenGroups.size > 0) {
				// Build effective hidden set: if a library is hidden, also hide all its folders
				const effectiveHidden = new Set(hiddenGroups);
				if (cb === 'folder') {
					for (const lib of hiddenGroups) {
						const folders = libraryFolderMap[lib];
						if (folders) {
							for (const f of folders) effectiveHidden.add(f);
						}
					}
				}
				const groupKey = cb === 'folder' ? (n: typeof dataNodes[0]) => getNodeFolder(n.path) : (n: typeof dataNodes[0]) => n.libraryName;
				const visibleIds = new Set(dataNodes.filter(n => !effectiveHidden.has(groupKey(n))).map(n => n.id));
				dataNodes = dataNodes.filter(n => visibleIds.has(n.id));
				dataLinks = links.filter(l => visibleIds.has(l.source) && visibleIds.has(l.target));
			}

			engine.setData(dataNodes, dataLinks, cmap);
		}
	});

	// Focus depth → engine
	let prevFocusDepth = 2;
	$effect(() => {
		const d = focusDepth;
		if (d !== prevFocusDepth) {
			prevFocusDepth = d;
			engine?.setFocusDepth(d);
		}
	});

	// Constellation Lens overlay — react to lens data changes
	$effect(() => {
		if (lensCentrality && lensCommunityAssignments && lensCommunityColors) {
			engine?.setLensOverlay(lensCentrality, lensCommunityAssignments, lensCommunityColors, lensGaps ?? []);
		} else {
			engine?.clearLensOverlay();
		}
	});

	onMount(async () => {
		window.addEventListener('keydown', handleKeydown);
		window.addEventListener('click', handleGlobalClick);

		engine = new GraphEngine(containerEl, engineConfig, {
			onNodeClick: (path, lib) => {
				// Add to breadcrumb if in focus mode
				const clickedNode = nodes.find(n => n.path === path);
				if (clickedNode && focusActive) {
					breadcrumb = [...breadcrumb.filter(b => b.id !== clickedNode.id), { id: clickedNode.id, name: clickedNode.name }].slice(-8);
				}
				onNodeClick?.(path, lib);
			},
			onNodeHover: (node) => { hoveredName = node?.name ?? null; onNodeHover?.(node); },
			onStatsReady: (nc, ec, mc) => { nodeCount = nc; edgeCount = ec; mocCount = mc; },
			onContextMenu: (node, x, y) => { contextMenu = { node, x, y }; },
			onFocusChange: (active, name) => { focusActive = active; focusNodeName = name ?? ''; if (!active) { breadcrumb = []; focusDirection = 'all'; } },
			onHiddenCountChange: (count) => { hiddenCount = count; },
			onTiltChange: (tilted) => { isTilted = tilted; },
		});

		await engine.init();

		if (nodes.length > 0) {
			prevNodeLen = nodes.length;
			engine.setData(nodes, links, activeColorMap);
		}
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
		window.removeEventListener('click', handleGlobalClick);
		// Defer PIXI teardown — app.destroy() is synchronous and expensive (~100ms).
		// Running it on the same frame as unmount causes a visible freeze.
		// Capture and null the reference first so nothing can call it after this point.
		const capturedEngine = engine;
		engine = null;
		setTimeout(() => capturedEngine?.destroy(), 0);
	});
</script>

<div class="gm-container" bind:this={containerEl}>
	<!-- Toolbar -->
	<div class="gm-toolbar" dir="auto">
		<div class="gm-toolbar-left">
			<button class="gm-btn" class:active={searchVisible} title="{$t('graphView.controls.searchPlaceholder')} (Ctrl+F)"
				onclick={() => { searchVisible = !searchVisible; if (!searchVisible) searchQuery = ''; }}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
			</button>
			{#if searchVisible}
				<input class="gm-search" type="text" dir="auto" placeholder={$t('graphView.controls.searchPlaceholder')}
					bind:value={searchQuery} autofocus />
			{/if}
		</div>
		<div class="gm-toolbar-right">
			{#if isTilted}
				<button class="gm-btn" title="Reset rotation (0)" onclick={() => { engine?.resetTilt(); isTilted = false; }}>
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9"/><polyline points="3 3 3 12 12 12"/></svg>
				</button>
			{/if}
			<button class="gm-btn" title="Fit to screen" onclick={() => engine?.fitToScreen()}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/></svg>
			</button>
			<button class="gm-btn" class:active={settingsOpen} title="Settings"
				onclick={() => settingsOpen = !settingsOpen}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
			</button>
		</div>
	</div>

	<!-- Settings panel -->
	{#if settingsOpen}
		<div class="gm-settings" dir="auto">
			<div class="gm-settings-tabs">
				<button class="gm-tab" class:active={settingsTab === 'appearance'} onclick={() => settingsTab = 'appearance'}>
					{$t('settings.skyview.graphAppearance') || 'Appearance'}
				</button>
				<button class="gm-tab" class:active={settingsTab === 'physics'} onclick={() => settingsTab = 'physics'}>
					{$t('settings.skyview.physics') || 'Physics'}
				</button>
				<button class="gm-tab" class:active={settingsTab === 'intelligence'} onclick={() => settingsTab = 'intelligence'}>
					AI
				</button>
			</div>

			{#if settingsTab === 'appearance'}
				<label class="gm-setting">
					<span>{$t('settings.skyview.nodeSize') || 'Node size'}</span>
					<input type="range" min="1" max="10" step="0.5" bind:value={uiNodeSize}
						oninput={() => handleSettingChange('nodeSize', uiNodeSize)} />
					<span class="gm-val">{uiNodeSize}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.labelVisibility') || 'Labels'}</span>
					<select bind:value={uiLabelVisibility}
						onchange={() => handleSettingChange('labelVisibility', uiLabelVisibility)}>
						<option value="hover">{$t('settings.skyview.labelHover') || 'On hover'}</option>
						<option value="always">{$t('settings.skyview.labelAlways') || 'Always'}</option>
						<option value="none">{$t('settings.skyview.labelNone') || 'None'}</option>
					</select>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.labelFontSize') || 'Label size'}</span>
					<input type="range" min="8" max="24" step="1" bind:value={uiLabelFontSize}
						oninput={() => handleSettingChange('labelFontSize', uiLabelFontSize)} />
					<span class="gm-val">{uiLabelFontSize}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.linkThickness') || 'Link width'}</span>
					<input type="range" min="0.5" max="5" step="0.5" bind:value={uiLinkThickness}
						oninput={() => handleSettingChange('linkThickness', uiLinkThickness)} />
					<span class="gm-val">{uiLinkThickness}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.showOrphans') || 'Show orphans'}</span>
					<input type="checkbox" bind:checked={uiShowOrphans}
						onchange={() => { handleSettingChange('showOrphans', uiShowOrphans); engine?.setData(nodes, links, libraryColorMap); }} />
				</label>
			{:else if settingsTab === 'physics'}
				<label class="gm-setting">
					<span>{$t('settings.skyview.repelForce') || 'Repulsion'}</span>
					<input type="range" min="10" max="300" step="5" bind:value={uiRepelForce}
						oninput={() => handleSettingChange('repelForce', uiRepelForce)} />
					<span class="gm-val">{uiRepelForce}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.linkForce') || 'Link force'}</span>
					<input type="range" min="0.01" max="0.3" step="0.01" bind:value={uiLinkForce}
						oninput={() => handleSettingChange('linkForce', uiLinkForce)} />
					<span class="gm-val">{uiLinkForce.toFixed(2)}</span>
				</label>
				<label class="gm-setting">
					<span>{$t('settings.skyview.linkDistance') || 'Link distance'}</span>
					<input type="range" min="10" max="300" step="5" bind:value={uiLinkDistance}
						oninput={() => handleSettingChange('linkDistance', uiLinkDistance)} />
					<span class="gm-val">{uiLinkDistance}</span>
				</label>
			{:else if settingsTab === 'intelligence'}
				<!-- Semantic Links (Phase 2) -->
				<div class="gm-ai-section">
					<div class="gm-ai-header">Semantic Links</div>
					<p class="gm-ai-desc">AI detects hidden connections between notes that share conceptual overlap but aren't explicitly linked.</p>

					{#if semanticLinkCount > 0}
						<label class="gm-setting">
							<span>Show semantic links</span>
							<input type="checkbox" bind:checked={uiShowSemanticLinks}
								onchange={() => handleSettingChange('showSemanticLinks', uiShowSemanticLinks)} />
						</label>
						<div class="gm-ai-stat">{semanticLinkCount} connections found</div>
					{/if}

					<label class="gm-setting">
						<span>Threshold</span>
						<input type="range" min="0.3" max="0.9" step="0.05" bind:value={uiSemanticThreshold} />
						<span class="gm-val">{uiSemanticThreshold.toFixed(2)}</span>
					</label>

					<button class="gm-btn gm-compute-btn"
						disabled={semanticComputing}
						onclick={computeSemantic}>
						{#if semanticComputing}
							{semanticProgress}
						{:else if semanticLinkCount > 0}
							Recompute
						{:else}
							Compute Semantic Links
						{/if}
					</button>

					<p class="gm-ai-note">Runs locally — nothing leaves your machine. First run downloads a ~23MB model.</p>
				</div>

				<!-- Cluster Detection -->
				<div class="gm-ai-section" style="margin-top: 12px; padding-top: 8px; border-top: 1px solid var(--background-modifier-border);">
					<div class="gm-ai-header">Cluster Detection</div>
					<p class="gm-ai-desc">Groups notes into communities based on link structure using the Louvain algorithm.</p>

					{#if clusterResult && clusterResult.clusters.length > 0}
						<label class="gm-setting">
							<span>Show clusters</span>
							<input type="checkbox" bind:checked={showClusters}
								onchange={() => { if (showClusters && clusterResult) { const cm = new Map<number, string>(); clusterResult.clusters.forEach(c => cm.set(c.id, c.color)); engine?.setClusters(clusterResult.assignments, cm); } else { engine?.clearClusters(); } }} />
						</label>
						<div class="gm-ai-stat">{clusterResult.clusters.length} clusters (Q={clusterResult.modularity.toFixed(2)})</div>
						<div class="gm-cluster-list">
							{#each clusterResult.clusters as c}
								<div class="gm-cluster-item">
									<span class="gm-legend-dot" style="background:{c.color}"></span>
									<span class="gm-cluster-name">{c.suggestedName}</span>
									<span class="gm-cluster-count">{c.memberIds.length}</span>
								</div>
							{/each}
						</div>
					{/if}

					<button class="gm-btn gm-compute-btn"
						onclick={runClusterDetection}>
						{clusterResult ? 'Recompute' : 'Detect Clusters'}
					</button>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Focus mode bar -->
	{#if focusActive}
		<div class="gm-focus-bar" dir="auto">
			<span class="gm-focus-label">🔍 {focusNodeName}</span>
			<div class="gm-focus-direction">
				<button class="gm-dir-btn" class:active={focusDirection === 'all'}
					title={$t('graphView.directionAll')} onclick={() => { focusDirection = 'all'; engine?.setFocusDirection('all'); }}>↔</button>
				<button class="gm-dir-btn" class:active={focusDirection === 'incoming'}
					title={$t('graphView.directionIncoming')} onclick={() => { focusDirection = 'incoming'; engine?.setFocusDirection('incoming'); }}>{arrowIncoming}</button>
				<button class="gm-dir-btn" class:active={focusDirection === 'outgoing'}
					title={$t('graphView.directionOutgoing')} onclick={() => { focusDirection = 'outgoing'; engine?.setFocusDirection('outgoing'); }}>{arrowOutgoing}</button>
			</div>
			<label class="gm-focus-depth">
				<span>{$t('graphView.depth')}: {focusDepth}</span>
				<input type="range" min="1" max="6" step="1" bind:value={focusDepth} />
			</label>
			<button class="gm-btn gm-focus-exit" onclick={() => engine?.setFocusNode(null)}>✕</button>
		</div>
	{/if}

	<!-- Breadcrumb trail -->
	{#if breadcrumb.length > 0}
		<div class="gm-breadcrumb" dir="auto">
			{#each breadcrumb as item, i}
				{#if i > 0}<span class="gm-bc-sep">{breadcrumbSep}</span>{/if}
				<button class="gm-bc-item" onclick={() => { engine?.setFocusNode(item.id); }}
					dir="auto">{item.name}</button>
			{/each}
			<button class="gm-bc-clear" onclick={() => breadcrumb = []}>✕</button>
		</div>
	{/if}

	<!-- Local graph indicator -->
	{#if localGraph}
		<div class="gm-local-indicator" dir="auto">
			<span>📍 {$t('graphView.localGraph')}</span>
			<button class="gm-btn" style="width:auto;padding:0 8px;height:24px;font-size:11px" onclick={() => { engine?.toggleLocalGraph(); localGraph = false; }}>
				{$t('graphView.showAll')}
			</button>
		</div>
	{/if}

	<!-- Hidden nodes indicator -->
	{#if hiddenCount > 0}
		<div class="gm-hidden-bar" dir="auto">
			<span>{hiddenCount} {$t('graphView.hidden')}</span>
			<button class="gm-btn" style="width:auto;padding:0 8px;height:24px;font-size:11px" onclick={() => engine?.showAllHidden()}>
				{$t('graphView.showAll')}
			</button>
		</div>
	{/if}

	<!-- Context menu -->
	{#if contextMenu}
		<div class="gm-context-menu" style="{$isRTLStore ? 'right' : 'left'}:{$isRTLStore ? (window.innerWidth - contextMenu.x) : contextMenu.x}px;top:{contextMenu.y}px" dir="auto">
			<button class="gm-ctx-item" onclick={ctxOpen}>📄 {$t('graphView.open')}</button>
			<button class="gm-ctx-item" onclick={ctxFocus}>🔍 {$t('graphView.focus')}</button>
			<button class="gm-ctx-item" onclick={ctxPin}>📌 {engine?.isNodePinned(contextMenu.node.id) ? $t('graphView.unpin') : $t('graphView.pin')}</button>
			<button class="gm-ctx-item gm-ctx-danger" onclick={ctxHide}>👁 {$t('graphView.hide')}</button>
		</div>
	{/if}

	<!-- Stats bar -->
<div class="gm-stats" dir="auto">
		<span>{nodeCount} {$t('graphView.nodes') || 'nodes'}</span>
		<span class="gm-sep">&middot;</span>
		<span>{edgeCount} {$t('graphView.edges') || 'edges'}</span>
		{#if mocCount > 0}
			<span class="gm-sep">&middot;</span>
			<span>{mocCount} MOCs</span>
		{/if}
		{#if hoveredName}
			<span class="gm-sep">&middot;</span>
			<span class="gm-hovered" dir="auto">{hoveredName}</span>
		{/if}
	</div>

	<!-- Legend -->
	{#if Object.keys(activeColorMap).length > 0}
		<div class="gm-legend" dir="auto">
			<div class="gm-legend-header">
				<button class="gm-legend-toggle" class:active={colorBy === 'library'}
					onclick={() => { colorBy = 'library'; }}>
					{$t('graphView.colorByLibrary') || 'Library'}
				</button>
				<button class="gm-legend-toggle" class:active={colorBy === 'folder'}
					onclick={() => { colorBy = 'folder'; }}>
					{$t('graphView.colorByFolder') || 'Folder'} ({visibleFolderCount})
				</button>
			</div>
			<div class="gm-legend-items">
				{#each Object.entries(activeColorMap).filter(([name]) => {
					// In folder mode, only show folders from visible libraries
					if (colorBy === 'folder') {
						for (const [lib, folders] of Object.entries(libraryFolderMap)) {
							if (hiddenLibraries.has(lib)) continue;
							if (folders.has(name)) return true;
						}
						return false;
					}
					return true;
				}) as [name, color]}
					{@const nameIsRTL = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/.test(name)}
					{@const isHidden = hiddenGroups.has(name)}
					<label class="gm-legend-item" style:flex-direction={nameIsRTL ? 'row-reverse' : 'row'} style:opacity={isHidden ? 0.4 : 1}>
						<input type="checkbox" class="gm-legend-check" checked={!isHidden}
							onchange={() => {
								if (colorBy === 'folder') {
									const next = new Set(hiddenFolders);
									if (next.has(name)) next.delete(name); else next.add(name);
									hiddenFolders = next;
								} else {
									const next = new Set(hiddenLibraries);
									if (next.has(name)) next.delete(name); else next.add(name);
									hiddenLibraries = next;
								}
							}} />
						<span class="gm-legend-dot" style="background:{color}"></span>
						<span class="gm-legend-name" dir="auto" style:text-align={nameIsRTL ? 'right' : 'left'}>{name}</span>
					</label>
				{/each}
			</div>
			{#if hiddenGroups.size > 0}
				<button class="gm-legend-clear" onclick={() => { hiddenLibraries = new Set(); hiddenFolders = new Set(); }}>
					{$t('graphView.showAll') || 'Show all'}
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.gm-container {
		position: relative;
		width: 100%;
		height: 100%;
		overflow: hidden;
		background: var(--background-secondary);
	}

	/* Toolbar */
	.gm-toolbar {
		position: absolute;
		top: 8px; inset-inline-start: 8px; inset-inline-end: 8px;
		z-index: 10;
		display: flex;
		justify-content: space-between;
		align-items: center;
		pointer-events: none;
	}
	.gm-toolbar-left, .gm-toolbar-right {
		display: flex; gap: 4px; align-items: center;
		pointer-events: auto;
	}

	.gm-btn {
		display: flex; align-items: center; justify-content: center;
		width: 32px; height: 32px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s;
	}
	.gm-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.gm-btn.active { background: var(--interactive-accent); color: white; }

	.gm-search {
		height: 32px; padding: 0 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 13px; outline: none;
		min-width: 200px;
	}
	.gm-search:focus { border-color: var(--interactive-accent); }

	/* Settings panel */
	.gm-settings {
		position: absolute;
		top: 48px; inset-inline-end: 8px;
		z-index: 20; width: 260px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 8px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.15);
	}
	.gm-settings-tabs { display: flex; gap: 4px; margin-bottom: 8px; }
	.gm-tab {
		flex: 1; padding: 5px 8px;
		border: none; border-radius: 4px;
		background: transparent;
		color: var(--text-muted); font-size: 12px; cursor: pointer;
	}
	.gm-tab.active { background: var(--interactive-accent); color: white; }

	.gm-setting {
		display: flex; align-items: center; gap: 8px;
		padding: 4px 0; font-size: 12px; color: var(--text-muted);
	}
	.gm-setting span:first-child { flex: 1; white-space: nowrap; }
	.gm-setting input[type="range"] { flex: 1; max-width: 100px; accent-color: var(--interactive-accent); }
	.gm-setting select {
		background: var(--background-secondary); color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px; padding: 2px 6px; font-size: 12px;
	}
	.gm-setting input[type="checkbox"] { accent-color: var(--interactive-accent); }
	.gm-val { width: 30px; text-align: end; font-variant-numeric: tabular-nums; }

	/* Stats bar */
	.gm-stats {
		position: absolute;
		bottom: 8px; inset-inline-start: 8px;
		z-index: 10;
		display: flex; gap: 6px; align-items: center;
		font-size: 11px; color: var(--text-faint);
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		padding: 4px 10px; border-radius: 6px;
	}
	.gm-sep { opacity: 0.3; }
	.gm-hovered { color: var(--text-normal); font-weight: 500; }

	/* Legend */
	.gm-legend {
		position: absolute;
		bottom: 8px; inset-inline-end: 8px;
		z-index: 10;
		display: flex; flex-direction: column; gap: 3px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		padding: 6px 10px; border-radius: 6px;
		max-height: 240px;
		min-width: 100px;
	}
	.gm-legend-items {
		display: flex; flex-direction: column; gap: 3px;
		overflow-y: auto; max-height: 180px;
		scrollbar-width: thin;
	}
	.gm-legend-items::-webkit-scrollbar { width: 4px; }
	.gm-legend-items::-webkit-scrollbar-thumb { background: var(--background-modifier-border); border-radius: 2px; }
	.gm-legend-item {
		display: flex; align-items: center; gap: 6px;
		font-size: 11px; color: var(--text-muted);
	}
	.gm-legend-header {
		display: flex; gap: 2px; margin-bottom: 4px;
		border-bottom: 1px solid var(--background-modifier-border);
		padding-bottom: 4px;
	}
	.gm-legend-toggle {
		flex: 1; padding: 2px 6px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: transparent; color: var(--text-muted);
		font-size: 10px; cursor: pointer;
		white-space: nowrap;
	}
	.gm-legend-toggle.active {
		background: var(--interactive-accent); color: white;
		border-color: var(--interactive-accent);
	}
	.gm-legend-toggle:hover:not(.active) { background: var(--background-modifier-hover); }
	.gm-legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.gm-legend-check {
		width: 12px; height: 12px; margin: 0; cursor: pointer;
		accent-color: var(--interactive-accent, #7c3aed);
	}
	.gm-legend-name { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: start; }
	.gm-legend-clear {
		width: 100%; border: none; background: transparent;
		color: var(--interactive-accent, #7c3aed); font-size: 10px;
		cursor: pointer; padding: 2px 0; text-align: center;
		border-top: 1px solid var(--background-modifier-border);
		margin-top: 2px;
	}
	.gm-legend-clear:hover { text-decoration: underline; }

	/* AI Intelligence tab */
	.gm-ai-section { display: flex; flex-direction: column; gap: 6px; }
	.gm-ai-header { font-size: 12px; font-weight: 600; color: var(--text-normal); }
	.gm-ai-desc { font-size: 10px; color: var(--text-faint); margin: 0; line-height: 1.4; }
	.gm-ai-note { font-size: 9px; color: var(--text-faint); margin: 0; opacity: 0.7; }
	.gm-ai-stat { font-size: 11px; color: var(--interactive-accent, #7c3aed); font-weight: 500; }
	.gm-compute-btn {
		width: 100% !important; padding: 6px !important; font-size: 11px !important;
		margin-top: 4px;
	}
	.gm-compute-btn:disabled { opacity: 0.6; cursor: wait !important; }
	.gm-cluster-list { display: flex; flex-direction: column; gap: 2px; max-height: 100px; overflow-y: auto; }
	.gm-cluster-item { display: flex; align-items: center; gap: 4px; font-size: 10px; color: var(--text-muted); }
	.gm-cluster-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.gm-cluster-count { font-size: 9px; color: var(--text-faint); }

	/* 3D Axis gizmo labels */
/* Context menu */
	.gm-context-menu {
		position: fixed; z-index: 100;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px;
		box-shadow: 0 8px 24px rgba(0,0,0,0.2);
		min-width: 140px;
	}
	.gm-ctx-item {
		display: flex; align-items: center; gap: 8px;
		width: 100%; padding: 6px 10px;
		border: none; border-radius: 4px;
		background: transparent; color: var(--text-normal);
		font-size: 12px; cursor: pointer; text-align: start;
	}
	.gm-ctx-item:hover { background: var(--background-modifier-hover); }
	.gm-ctx-danger { color: var(--text-error, #ef4444); }
	.gm-ctx-danger:hover { background: rgba(239, 68, 68, 0.1); }

	/* Focus bar */
	.gm-focus-bar {
		position: absolute;
		top: 48px; inset-inline-start: 8px;
		z-index: 15;
		display: flex; align-items: center; gap: 8px;
		background: var(--background-primary);
		border: 1px solid var(--interactive-accent);
		border-radius: 8px; padding: 6px 12px;
		font-size: 12px; color: var(--text-normal);
		box-shadow: 0 2px 8px rgba(0,0,0,0.1);
	}
	.gm-focus-label { font-weight: 600; max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.gm-focus-depth { display: flex; align-items: center; gap: 6px; color: var(--text-muted); }
	.gm-focus-depth input[type="range"] { width: 80px; accent-color: var(--interactive-accent); }
	.gm-focus-exit { width: 24px !important; height: 24px !important; font-size: 14px; }

	/* Local graph indicator */
	.gm-local-indicator {
		position: absolute;
		top: 48px; inset-inline-start: 8px;
		z-index: 15;
		display: flex; align-items: center; gap: 8px;
		background: var(--background-primary);
		border: 1px solid var(--interactive-accent);
		border-radius: 8px; padding: 4px 10px;
		font-size: 11px; color: var(--text-muted);
	}

	/* Direction filter buttons */
	.gm-focus-direction { display: flex; gap: 2px; }
	.gm-dir-btn {
		width: 26px; height: 24px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: transparent; color: var(--text-muted);
		font-size: 14px; cursor: pointer;
		display: flex; align-items: center; justify-content: center;
	}
	.gm-dir-btn.active { background: var(--interactive-accent); color: white; border-color: var(--interactive-accent); }
	.gm-dir-btn:hover:not(.active) { background: var(--background-modifier-hover); }

	/* Breadcrumb trail */
	.gm-breadcrumb {
		position: absolute;
		bottom: 36px; inset-inline-start: 8px; inset-inline-end: 8px;
		z-index: 12;
		display: flex; align-items: center; gap: 4px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 4px 8px;
		overflow-x: auto;
	}
	.gm-bc-item {
		border: none; border-radius: 4px;
		background: transparent; color: var(--text-accent);
		font-size: 11px; cursor: pointer; white-space: nowrap;
		padding: 2px 6px;
	}
	.gm-bc-item:hover { background: var(--background-modifier-hover); }
	.gm-bc-sep { color: var(--text-faint); font-size: 10px; }
	.gm-bc-clear {
		border: none; background: transparent;
		color: var(--text-faint); cursor: pointer;
		font-size: 12px; margin-inline-start: auto;
	}
	.gm-bc-clear:hover { color: var(--text-normal); }

	/* Layout button */
	.gm-layout-btn { position: relative; }
	.gm-layout-label { font-weight: 500; text-transform: capitalize; color: var(--text-accent); }

	/* Hidden nodes indicator */
	.gm-hidden-bar {
		position: absolute;
		top: 48px; inset-inline-end: 8px;
		z-index: 15;
		display: flex; align-items: center; gap: 8px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px 10px;
		font-size: 11px; color: var(--text-muted);
	}
</style>

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
	import { t } from '$lib/i18n';
	import { GraphEngine, type EngineConfig, type LayoutMode } from '$lib/graph/graphEngine';
	import type { StarNode, StarLink } from '$lib/libraries/store';

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
	};

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
		skyViewSettings?: Partial<EngineConfig>;
		libraryColorMap?: Record<string, string>;
	} = $props();

	// ─── Layer 1 state: UI only ─────────────────────────────
	let settingsOpen = $state(false);
	let settingsTab: 'appearance' | 'physics' = $state('appearance');
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

	let containerEl: HTMLDivElement;
	let engine: GraphEngine | null = null;

	function handleSettingChange(key: keyof EngineConfig, value: any) {
		(engineConfig as any)[key] = value;
		engine?.updateConfig({ [key]: value });
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
		// Ctrl+L — cycle layout modes
		if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
			e.preventDefault();
			if (engine) {
				layoutMode = engine.cycleLayoutMode();
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

	// Data changes → engine
	let prevNodeLen = 0;
	$effect(() => {
		const len = nodes.length;
		if (len !== prevNodeLen && len > 0 && engine) {
			prevNodeLen = len;
			engine.setData(nodes, links, libraryColorMap);
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
			onNodeHover: (name) => { hoveredName = name; },
			onStatsReady: (nc, ec, mc) => { nodeCount = nc; edgeCount = ec; mocCount = mc; },
			onContextMenu: (node, x, y) => { contextMenu = { node, x, y }; },
			onFocusChange: (active, name) => { focusActive = active; focusNodeName = name ?? ''; if (!active) { breadcrumb = []; focusDirection = 'all'; } },
			onHiddenCountChange: (count) => { hiddenCount = count; },
		});

		await engine.init();

		if (nodes.length > 0) {
			prevNodeLen = nodes.length;
			engine.setData(nodes, links, libraryColorMap);
		}
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
		window.removeEventListener('click', handleGlobalClick);
		engine?.destroy();
		engine = null;
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
			<button class="gm-btn gm-layout-btn" title="Layout mode (Ctrl+L)"
				onclick={() => { if (engine) layoutMode = engine.cycleLayoutMode(); }}>
				{#if layoutMode === 'organic'}
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="2"/><circle cx="6" cy="6" r="2"/><circle cx="18" cy="6" r="2"/><circle cx="6" cy="18" r="2"/><circle cx="18" cy="18" r="2"/><line x1="12" y1="12" x2="6" y2="6"/><line x1="12" y1="12" x2="18" y2="6"/><line x1="12" y1="12" x2="6" y2="18"/><line x1="12" y1="12" x2="18" y2="18"/></svg>
				{:else if layoutMode === 'hierarchical'}
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="3" r="2"/><circle cx="6" cy="12" r="2"/><circle cx="18" cy="12" r="2"/><circle cx="4" cy="21" r="2"/><circle cx="12" cy="21" r="2"/><circle cx="20" cy="21" r="2"/><line x1="12" y1="5" x2="6" y2="10"/><line x1="12" y1="5" x2="18" y2="10"/><line x1="6" y1="14" x2="4" y2="19"/><line x1="18" y1="14" x2="20" y2="19"/><line x1="6" y1="14" x2="12" y2="19"/></svg>
				{:else}
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="21" x2="21" y2="21"/><circle cx="5" cy="16" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="13" cy="14" r="2"/><circle cx="17" cy="8" r="2"/><circle cx="21" cy="6" r="2"/></svg>
				{/if}
			</button>
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
			{:else}
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
			{/if}
		</div>
	{/if}

	<!-- Focus mode bar -->
	{#if focusActive}
		<div class="gm-focus-bar" dir="auto">
			<span class="gm-focus-label">🔍 {focusNodeName}</span>
			<div class="gm-focus-direction">
				<button class="gm-dir-btn" class:active={focusDirection === 'all'}
					title="All links" onclick={() => { focusDirection = 'all'; engine?.setFocusDirection('all'); }}>↔</button>
				<button class="gm-dir-btn" class:active={focusDirection === 'incoming'}
					title="Incoming only" onclick={() => { focusDirection = 'incoming'; engine?.setFocusDirection('incoming'); }}>←</button>
				<button class="gm-dir-btn" class:active={focusDirection === 'outgoing'}
					title="Outgoing only" onclick={() => { focusDirection = 'outgoing'; engine?.setFocusDirection('outgoing'); }}>→</button>
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
				{#if i > 0}<span class="gm-bc-sep">›</span>{/if}
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
		<div class="gm-context-menu" style="left:{contextMenu.x}px;top:{contextMenu.y}px" dir="auto">
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
		{#if layoutMode !== 'organic'}
			<span class="gm-sep">&middot;</span>
			<span class="gm-layout-label">{layoutMode === 'hierarchical' ? '🏛️' : '📅'} {layoutMode}</span>
		{/if}
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
		top: 8px; left: 8px; right: 8px;
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
		top: 48px; right: 8px;
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
		bottom: 8px; left: 8px;
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
		bottom: 8px; right: 8px;
		z-index: 10;
		display: flex; flex-direction: column; gap: 3px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		padding: 6px 10px; border-radius: 6px;
	}
	.gm-legend-item {
		display: flex; align-items: center; gap: 6px;
		font-size: 11px; color: var(--text-muted);
	}
	.gm-legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
	.gm-legend-name { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

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
		top: 48px; left: 8px;
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
		top: 48px; left: 8px;
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
		bottom: 36px; left: 8px; right: 8px;
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
		top: 48px; right: 8px;
		z-index: 15;
		display: flex; align-items: center; gap: 8px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px 10px;
		font-size: 11px; color: var(--text-muted);
	}
</style>

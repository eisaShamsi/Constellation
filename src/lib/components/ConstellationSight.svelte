<script lang="ts">
	/**
	 * ConstellationSight — Standalone network analysis visualization.
	 * D3.js force simulation + HTML5 Canvas rendering.
	 * Visually distinct from Sky View (GraphMind/Pixi.js).
	 */
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import * as d3 from 'd3';
	import { t, dir } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import type { StarNode, StarLink } from '$lib/libraries/store';
	import type { ClusterInfo, StructuralGap, UniverseHealth } from '$lib/graph/clusterEngine';
	import LensPanel from './LensPanel.svelte';

	let {
		nodes = [] as StarNode[],
		links = [] as StarLink[],
		centrality = new Map<string, number>(),
		communityAssignments = new Map<string, number>(),
		communityColors = new Map<number, string>(),
		gaps = [] as StructuralGap[],
		health = null as UniverseHealth | null,
		bridges = [] as { id: string; name: string; centrality: number }[],
		communities = [] as ClusterInfo[],
		communityProfiles = [] as any[],
		contradictions = [] as [string, string][],
		libraryColorMap = {} as Record<string, string>,
		onNoteClick,
		onClose,
	}: {
		nodes?: StarNode[];
		links?: StarLink[];
		centrality?: Map<string, number>;
		communityAssignments?: Map<string, number>;
		communityColors?: Map<number, string>;
		gaps?: StructuralGap[];
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		communities?: ClusterInfo[];
		communityProfiles?: any[];
		contradictions?: [string, string][];
		libraryColorMap?: Record<string, string>;
		onNoteClick?: (path: string, name: string) => void;
		onClose?: () => void;
		searchMatchIds?: Set<string> | null;
	} = $props();

	// ─── Canvas state ───
	let canvasEl: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D;
	let width = 0;
	let height = 0;
	let animFrame = 0;
	let destroyed = false;

	// ─── Simulation state ───
	interface SimNode extends d3.SimulationNodeDatum {
		id: string;
		name: string;
		path: string;
		libraryName: string;
		centrality: number;
		communityId: number;
		communityColor: string;
		r: number;
	}
	interface SimLink extends d3.SimulationLinkDatum<SimNode> {
		linkType?: string;
	}

	let simNodes: SimNode[] = [];
	let simLinks: SimLink[] = [];
	let simulation: d3.Simulation<SimNode, SimLink> | null = null;

	// ─── Interaction state ───
	let panX = 0, panY = 0, zoom = 1;
	let isPanning = false;
	let panStartX = 0, panStartY = 0;
	let hoveredNode: SimNode | null = null;
	let searchVisible = $state(false);
	let searchQuery = $state('');
	let searchScope = $state<'all' | 'title' | 'content'>('all');
	interface SearchMatch { node: SimNode; matchType: 'title' | 'content' | 'both'; }
	let searchResults = $state<SearchMatch[]>([]);
	let searchIdx = $state(0);
	let settingsVisible = $state(false);
	let showRegions = $state(true);
	let forceStrength = $state(-60);
	let linkDistance = $state(40);
	let quadtree: d3.Quadtree<SimNode> | null = null;

	// ─── Community hulls cache ───
	let communityHulls: Map<number, { points: [number, number][]; cx: number; cy: number; rx: number; ry: number; color: string }> = new Map();

	// ─── Colors ───
	const LINK_TYPE_COLORS: Record<string, string> = {
		supports: '#4A9EFF', contradicts: '#FF4A4A', causes: '#FF8C42',
		exemplifies: '#4AFF88', generalizes: '#C084FC', 'derives-from': '#FACC15',
		'part-of': '#94A3B8', associative: '#A78BFA',
	};
	const GRID_COLOR = 'rgba(148, 163, 184, 0.08)';
	const GAP_COLOR = '#ef4444';

	// ─── Build simulation data ───
	function buildSimData() {
		const nodeMap = new Map<string, SimNode>();
		simNodes = nodes.map(n => {
			const c = centrality.get(n.id) ?? 0;
			const cid = communityAssignments.get(n.id) ?? 0;
			const color = communityColors.get(cid) ?? '#94a3b8';
			const sn: SimNode = {
				id: n.id, name: n.name, path: n.path, libraryName: n.libraryName,
				centrality: c, communityId: cid, communityColor: color,
				r: Math.max(3, 3 + c * 18),
				x: (Math.random() - 0.5) * width * 0.8,
				y: (Math.random() - 0.5) * height * 0.8,
			};
			nodeMap.set(n.id, sn);
			return sn;
		});

		simLinks = [];
		for (const l of links) {
			const src = nodeMap.get(l.source);
			const tgt = nodeMap.get(l.target);
			if (src && tgt && src !== tgt) {
				simLinks.push({ source: src, target: tgt, linkType: l.linkType });
			}
		}
	}

	// ─── Start D3 force simulation ───
	function startSimulation() {
		simulation = d3.forceSimulation(simNodes)
			.force('link', d3.forceLink(simLinks).id((d: any) => d.id).distance(40).strength(0.3))
			.force('charge', d3.forceManyBody().strength(-60).distanceMax(300))
			.force('center', d3.forceCenter(0, 0))
			.force('collide', d3.forceCollide<SimNode>().radius(d => d.r + 2))
			.alphaDecay(0.02)
			.on('tick', () => { updateQuadtree(); });

		// Run 200 ticks synchronously for initial layout, then let it animate
		for (let i = 0; i < 200; i++) simulation.tick();
		simulation.alpha(0.1).restart();

		updateQuadtree();
		computeHulls();
	}

	function updateQuadtree() {
		quadtree = d3.quadtree<SimNode>()
			.x(d => d.x ?? 0).y(d => d.y ?? 0)
			.addAll(simNodes);
	}

	// ─── Compute convex hulls for communities ───
	function computeHulls() {
		communityHulls.clear();
		const groups = new Map<number, SimNode[]>();
		for (const n of simNodes) {
			if (!groups.has(n.communityId)) groups.set(n.communityId, []);
			groups.get(n.communityId)!.push(n);
		}
		for (const [cid, members] of groups) {
			if (members.length < 3) continue;
			const cx = d3.mean(members, n => n.x ?? 0) ?? 0;
			const cy = d3.mean(members, n => n.y ?? 0) ?? 0;
			// Compute ellipse radii from standard deviation of positions + padding
			let sumDx2 = 0, sumDy2 = 0;
			for (const n of members) {
				sumDx2 += ((n.x ?? 0) - cx) ** 2;
				sumDy2 += ((n.y ?? 0) - cy) ** 2;
			}
			const rx = Math.sqrt(sumDx2 / members.length) * 2 + 30;
			const ry = Math.sqrt(sumDy2 / members.length) * 2 + 30;
			communityHulls.set(cid, { points: [], cx, cy, rx, ry, color: communityColors.get(cid) ?? '#94a3b8' });
		}
	}

	// ─── Canvas rendering ───
	function render() {
		if (destroyed || !ctx) return;
		const w = width, h = height;
		const dpr = window.devicePixelRatio || 1;

		ctx.save();
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, w, h);

		// Background
		ctx.fillStyle = $dir === 'rtl' ? '#fafafa' : '#fafafa';
		ctx.fillRect(0, 0, w, h);

		// Grid
		ctx.strokeStyle = GRID_COLOR;
		ctx.lineWidth = 1;
		const gridSize = 40 * zoom;
		const offX = (w / 2 + panX) % gridSize;
		const offY = (h / 2 + panY) % gridSize;
		for (let x = offX; x < w; x += gridSize) {
			ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
		}
		for (let y = offY; y < h; y += gridSize) {
			ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
		}

		// Transform for pan/zoom
		ctx.save();
		ctx.translate(w / 2 + panX, h / 2 + panY);
		ctx.scale(zoom, zoom);

		// 1. Community regions (convex hulls)
		if (showRegions) for (const [, hull] of communityHulls) {
			ctx.beginPath();
			ctx.ellipse(hull.cx, hull.cy, hull.rx, hull.ry, 0, 0, Math.PI * 2);
			ctx.fillStyle = hull.color + '22';
			ctx.fill();
			ctx.strokeStyle = hull.color + '88';
			ctx.lineWidth = 2 / zoom;
			ctx.stroke();
		}

		// 2. Gap lines (red dashed)
		for (const gap of gaps) {
			const h1 = communityHulls.get(gap.community1);
			const h2 = communityHulls.get(gap.community2);
			if (!h1 || !h2) continue;
			ctx.beginPath();
			ctx.setLineDash([8 / zoom, 6 / zoom]);
			ctx.moveTo(h1.cx, h1.cy);
			ctx.lineTo(h2.cx, h2.cy);
			ctx.strokeStyle = GAP_COLOR + 'AA';
			ctx.lineWidth = 2.5 / zoom;
			ctx.stroke();
			ctx.setLineDash([]);
		}

		// 3. Edges — typed links: 2px, 70% opacity, solid. Untyped: dashed, subtle.
		for (const link of simLinks) {
			const src = link.source as SimNode;
			const tgt = link.target as SimNode;
			const typed = link.linkType && LINK_TYPE_COLORS[link.linkType];
			const color = typed ? LINK_TYPE_COLORS[link.linkType!] : '#94a3b8';
			ctx.beginPath();
			if (!typed) ctx.setLineDash([4 / zoom, 3 / zoom]); // untyped = dashed
			ctx.moveTo(src.x ?? 0, src.y ?? 0);
			ctx.lineTo(tgt.x ?? 0, tgt.y ?? 0);
			ctx.strokeStyle = typed ? color + 'B3' : color + '4D'; // typed=70%, untyped=25%
			ctx.lineWidth = typed ? 1 / zoom : 0.7 / zoom;
			ctx.stroke();
			if (!typed) ctx.setLineDash([]); // reset dash
		}

		// 4. Nodes
		for (const n of simNodes) {
			const x = n.x ?? 0, y = n.y ?? 0;
			ctx.beginPath();
			ctx.arc(x, y, n.r, 0, Math.PI * 2);
			ctx.fillStyle = n.communityColor;
			ctx.fill();
			// Centrality ring for top bridges
			if (n.centrality > 0.5) {
				ctx.strokeStyle = '#ffffff';
				ctx.lineWidth = 1.5 / zoom;
				ctx.stroke();
			}
			// Hover highlight
			if (hoveredNode === n) {
				ctx.strokeStyle = '#000000';
				ctx.lineWidth = 2 / zoom;
				ctx.stroke();
			}
		}

		// 5. Search results highlight — drawn ON TOP of all nodes
			// Colors: title = blue, content = green, both = purple
			if (searchResults.length > 0) {
				const currentId = searchResults[searchIdx]?.node.id;
				const matchMap = new Map(searchResults.map(m => [m.node.id, m.matchType]));
				const MATCH_COLORS = { title: '#3b82f6', content: '#16a34a', both: '#7c3aed' };

				for (const sn of simNodes) {
					const matchType = matchMap.get(sn.id);
					if (!matchType) continue;
					const sx = sn.x ?? 0, sy = sn.y ?? 0;
					const isCurrent = sn.id === currentId;
					const color = MATCH_COLORS[matchType];

					if (isCurrent) {
						// Current match: bouncing arrow + thick ring in match color
						const arrowSize = 14 / zoom;
						const gap = sn.r + 8 / zoom;
						const bounce = Math.sin(Date.now() / 200) * (5 / zoom);

						ctx.beginPath();
						ctx.moveTo(sx, sy - gap - bounce);
						ctx.lineTo(sx - arrowSize * 0.7, sy - gap - arrowSize - bounce);
						ctx.lineTo(sx + arrowSize * 0.7, sy - gap - arrowSize - bounce);
						ctx.closePath();
						ctx.fillStyle = color;
						ctx.fill();
						ctx.strokeStyle = '#ffffff';
						ctx.lineWidth = 1 / zoom;
						ctx.stroke();

						ctx.beginPath();
						ctx.arc(sx, sy, sn.r + 5 / zoom, 0, Math.PI * 2);
						ctx.strokeStyle = color;
						ctx.lineWidth = 3 / zoom;
						ctx.stroke();

						ctx.beginPath();
						ctx.arc(sx, sy, sn.r + 10 / zoom, 0, Math.PI * 2);
						ctx.strokeStyle = color + '66';
						ctx.lineWidth = 2 / zoom;
						ctx.stroke();
					} else {
						// Other matches: colored ring by match type
						ctx.beginPath();
						ctx.arc(sx, sy, sn.r + 4 / zoom, 0, Math.PI * 2);
						ctx.strokeStyle = color;
						ctx.lineWidth = 2 / zoom;
						ctx.stroke();
					}
				}
			}

		// 6. Hover label (RTL-aware)
		if (hoveredNode) {
			const x = hoveredNode.x ?? 0, y = hoveredNode.y ?? 0;
			const label = hoveredNode.name.replace(/\.md$/, '');
			const isRTL = detectDir(label) === 'rtl';
			const fontSize = Math.max(10, 12 / zoom);
			ctx.font = `600 ${fontSize}px system-ui, sans-serif`;
			ctx.direction = isRTL ? 'rtl' : 'ltr';
			const metrics = ctx.measureText(label);
			const tw = metrics.width;
			const th = fontSize + 4;
			const lx = x - tw / 2, ly = y - hoveredNode.r - th - 4;
			// Background
			ctx.fillStyle = 'rgba(0,0,0,0.8)';
			ctx.beginPath();
			ctx.roundRect(lx - 4, ly - 2, tw + 8, th + 4, 4 / zoom);
			ctx.fill();
			// Text
			ctx.fillStyle = '#ffffff';
			ctx.textAlign = 'center';
			ctx.textBaseline = 'top';
			ctx.fillText(label, x, ly + 2);
		ctx.direction = "ltr"; // reset after RTL label
		}

		ctx.restore(); // pop pan/zoom
		ctx.restore(); // pop DPR
		animFrame = requestAnimationFrame(render);
	}

	// ─── Mouse interaction ───
	function screenToWorld(sx: number, sy: number): [number, number] {
		return [(sx - width / 2 - panX) / zoom, (sy - height / 2 - panY) / zoom];
	}

	function onMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		isPanning = true;
		panStartX = e.clientX - panX;
		panStartY = e.clientY - panY;
	}
	function onMouseMove(e: MouseEvent) {
		const rect = canvasEl.getBoundingClientRect();
		const sx = e.clientX - rect.left, sy = e.clientY - rect.top;

		if (isPanning) {
			panX = e.clientX - panStartX;
			panY = e.clientY - panStartY;
			return;
		}

		// Hit test
		const [wx, wy] = screenToWorld(sx, sy);
		if (quadtree) {
			const found = quadtree.find(wx, wy, 20 / zoom);
			hoveredNode = found ?? null;
			canvasEl.style.cursor = hoveredNode ? 'pointer' : 'grab';
		}
	}
	function onMouseUp() { isPanning = false; }
	function onClick(e: MouseEvent) {
		if (hoveredNode && onNoteClick) {
			onNoteClick(hoveredNode.path, hoveredNode.name);
		}
	}
	function onWheel(e: WheelEvent) {
		e.preventDefault();
		const delta = e.deltaY > 0 ? -0.08 : 0.08;
		zoom = Math.max(0.1, Math.min(5, zoom + delta));
	}

	// ─── Resize ───
	// ─── Search ───
	async function executeSearch() {
		if (!searchQuery.trim()) { searchResults = []; searchIdx = 0; return; }
		const q = searchQuery.toLowerCase();

		// 1. Title matches (instant, local)
		const titleMatchIds = new Set<string>();
		if (searchScope !== 'content') {
			for (const n of simNodes) {
				if (n.name.toLowerCase().includes(q)) titleMatchIds.add(n.id);
			}
		}

		// 2. Content + advanced matches via Constellation search (supports #tags, properties, links)
		const contentMatchIds = new Set<string>();
		if (searchScope !== 'title') {
			try {
				const { constellationSearch, parseSearchQuery } = await import('$lib/libraries/store');
				const req = parseSearchQuery(searchQuery);
				req.limit = 200;
				const results = await constellationSearch(req);
				for (const r of results) contentMatchIds.add(r.name.toLowerCase());
			} catch { /* fallback */ }
		}

		// 3. Classify each match
		const allIds = new Set([...titleMatchIds, ...contentMatchIds]);
		const nodeMap = new Map(simNodes.map(n => [n.id, n]));
		const matches: SearchMatch[] = [];

		for (const id of allIds) {
			const node = nodeMap.get(id);
			if (!node) continue;
			const inTitle = titleMatchIds.has(id);
			const inContent = contentMatchIds.has(id);
			const matchType = inTitle && inContent ? 'both' : inTitle ? 'title' : 'content';
			matches.push({ node, matchType });
		}

		matches.sort((a, b) => {
			const order = { both: 0, title: 1, content: 2 };
			return order[a.matchType] - order[b.matchType];
		});

		searchResults = matches;
		searchIdx = 0;
		if (searchResults.length > 0) centerOnSearchResult();
	}

	function centerOnSearchResult() {
		const match = searchResults[searchIdx];
		if (!match) return;
		const nx = match.node.x ?? 0, ny = match.node.y ?? 0;
		zoom = 2;
		panX = -nx * zoom;
		panY = -ny * zoom;
	}

	function nextSearchResult() {
		if (searchResults.length === 0) return;
		searchIdx = (searchIdx + 1) % searchResults.length;
		centerOnSearchResult();
	}

	function prevSearchResult() {
		if (searchResults.length === 0) return;
		searchIdx = (searchIdx - 1 + searchResults.length) % searchResults.length;
		centerOnSearchResult();
	}

	function clearSearch() {
		searchQuery = '';
		searchResults = [];
		searchIdx = 0;
		searchVisible = false;
	}

	// ─── Settings apply ───
	function applySettings() {
		if (!simulation) return;
		simulation.force('charge', d3.forceManyBody().strength(forceStrength).distanceMax(300));
		simulation.force('link', d3.forceLink(simLinks).id((d: any) => d.id).distance(linkDistance).strength(0.3));
		simulation.alpha(0.5).restart();
	}

	function resize() {
		if (!canvasEl) return;
		const rect = canvasEl.parentElement?.getBoundingClientRect();
		if (!rect) return;
		width = rect.width;
		height = rect.height;
		const dpr = window.devicePixelRatio || 1;
		canvasEl.width = width * dpr;
		canvasEl.height = height * dpr;
		canvasEl.style.width = width + 'px';
		canvasEl.style.height = height + 'px';
	}

	let resizeObs: ResizeObserver;

	// ─── Lifecycle ───
	onMount(() => {
		ctx = canvasEl.getContext('2d')!;
		resizeObs = new ResizeObserver(() => resize());
		resizeObs.observe(canvasEl.parentElement!);
		resize();
		buildSimData();
		startSimulation();
		animFrame = requestAnimationFrame(render);
	});

	// External search match highlighting (from Search Hub)
	$effect(() => {
		if (searchMatchIds && searchMatchIds.size > 0 && simNodes.length > 0) {
			const matches: SearchMatch[] = [];
			for (const sn of simNodes) {
				if (searchMatchIds.has(sn.id) || searchMatchIds.has(sn.name?.toLowerCase())) {
					matches.push({ node: sn, matchType: 'both' });
				}
			}
			if (matches.length > 0) {
				searchResults = matches;
				searchIdx = 0;
			}
		}
	});

	onDestroy(() => {
		destroyed = true;
		if (animFrame) cancelAnimationFrame(animFrame);
		simulation?.stop();
		resizeObs?.disconnect();
	});
</script>

<div class="cl-root" dir={$dir}>
	<div class="cl-header">
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
		<span class="cl-title">{$t('lens.title') || 'Constellation Sight'}</span>
		<span class="cl-stat">{nodes.length} {$t('lens.nodes') || 'nodes'} · {links.length} {$t('lens.edges') || 'edges'}</span>
		<div class="cl-toolbar">
			<button class="cl-toolbar-btn" class:active={searchVisible} onclick={() => { searchVisible = !searchVisible; if (!searchVisible) clearSearch(); }} title={$t('layout.search') || 'Search'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
			</button>
			<button class="cl-toolbar-btn" onclick={() => {
				if (simNodes.length === 0) return;
				let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
				for (const n of simNodes) {
					const x = n.x ?? 0, y = n.y ?? 0;
					if (x - n.r < minX) minX = x - n.r;
					if (x + n.r > maxX) maxX = x + n.r;
					if (y - n.r < minY) minY = y - n.r;
					if (y + n.r > maxY) maxY = y + n.r;
				}
				const graphW = maxX - minX || 1;
				const graphH = maxY - minY || 1;
				const scaleX = (width * 0.9) / graphW;
				const scaleY = (height * 0.9) / graphH;
				zoom = Math.min(scaleX, scaleY, 3);
				panX = -(minX + maxX) / 2 * zoom;
				panY = -(minY + maxY) / 2 * zoom;
			}} title={$t('lens.fitToScreen') || 'Fit to screen'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M21 8V5a2 2 0 0 0-2-2h-3"/><path d="M3 16v3a2 2 0 0 0 2 2h3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
			</button>
			<button class="cl-toolbar-btn" class:active={settingsVisible} onclick={() => settingsVisible = !settingsVisible} title={$t('ribbon.settings') || 'Settings'}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/></svg>
			</button>
		</div>
		<button class="cl-close" onclick={() => onClose?.()}>×</button>
	</div>
	<div class="cl-body">
		<div class="cl-canvas-wrap">
			<!-- Search bar -->
			{#if searchVisible}
				<div class="cl-search-bar">
					<div class="cl-search-scope">
						<button class:active={searchScope === 'all'} onclick={() => searchScope = 'all'}>{$t("lens.scopeAll") || "All"}</button>
						<button class:active={searchScope === 'title'} onclick={() => searchScope = 'title'}>{$t("lens.scopeTitle") || "Title"}</button>
						<button class:active={searchScope === 'content'} onclick={() => searchScope = 'content'}>{$t("lens.scopeContent") || "Content"}</button>
					</div>
					<input type="text" dir="auto" placeholder={searchScope === 'title' ? ($t('lens.searchTitles') || 'Search titles...') : searchScope === 'content' ? ($t('lens.searchContent') || 'Search content...') : ($t('lens.searchAll') || 'Search all... (Enter)')} bind:value={searchQuery}
						onkeydown={(e) => {
							if (e.key === 'Enter') { e.preventDefault(); searchResults.length > 0 ? (e.shiftKey ? prevSearchResult() : nextSearchResult()) : executeSearch(); }
							if (e.key === 'Escape') clearSearch();
						}} />
					<button onclick={clearSearch}>×</button>
					{#if searchResults.length > 0}
						{@const currentMatch = searchResults[searchIdx]}
						<span class="cl-search-type" style="background:{currentMatch?.matchType === 'title' ? '#3b82f6' : currentMatch?.matchType === 'content' ? '#16a34a' : '#7c3aed'}">{currentMatch?.matchType}</span>
						<span class="cl-search-found">{searchIdx + 1}/{searchResults.length}</span>
						<button class="cl-search-nav" onclick={prevSearchResult}>
							<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
						</button>
						<button class="cl-search-nav" onclick={nextSearchResult}>
							<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 6 15 12 9 18"/></svg>
						</button>
					{:else if searchQuery}
						<span class="cl-search-none">0</span>
					{/if}
				</div>
			{/if}
			<!-- Settings panel -->
			{#if settingsVisible}
				<div class="cl-settings">
					<div class="cl-settings-title">{$t("lens.display") || "Display"}</div>
					<label class="cl-settings-toggle">
						<span>{$t("lens.regions") || "Regions"}</span>
						<button class:active={showRegions} onclick={() => showRegions = !showRegions}>{showRegions ? 'On' : 'Off'}</button>
					</label>
					<div class="cl-settings-title">{$t("lens.physics") || "Physics"}</div>
					<label>
						<span>{$t("lens.repulsion") || "Repulsion"}: {forceStrength}</span>
						<input type="range" min="-200" max="-10" step="5" bind:value={forceStrength} oninput={applySettings} />
					</label>
					<label>
						<span>{$t("lens.linkDistance") || "Link Distance"}: {linkDistance}</span>
						<input type="range" min="10" max="150" step="5" bind:value={linkDistance} oninput={applySettings} />
					</label>
				</div>
			{/if}
			<canvas bind:this={canvasEl}
				onmousedown={onMouseDown} onmousemove={onMouseMove}
				onmouseup={onMouseUp} onclick={onClick}
				onwheel={onWheel}></canvas>
			<!-- Legend — lower-left, always visible -->
			<div class="cl-legend">
				<div class="cl-legend-title">{$t('lens.legend') || 'Legend'}</div>
				<div class="cl-legend-row">
					<span class="cl-lg-circle cl-lg-big"></span>
					<span><strong>{$t("lens.largeNode") || "Large node"}</strong> — {$t("lens.bridgeDesc") || "bridge"}</span>
				</div>
				<div class="cl-legend-row">
					<span class="cl-lg-circle cl-lg-small"></span>
					<span><strong>{$t("lens.smallNode") || "Small node"}</strong> — {$t("lens.peripheralDesc") || "peripheral"}</span>
				</div>
				<div class="cl-legend-row">
					<span class="cl-lg-circle" style="background:#a78bfa"></span>
					<span class="cl-lg-circle" style="background:#34d399"></span>
					<span class="cl-lg-circle" style="background:#60a5fa"></span>
					<span><strong>{$t("lens.legendCommunityColor") || "Color"}</strong> — {$t("lens.communityDesc") || "community"}</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#94a3b8" stroke-width="1" stroke-dasharray="4,3"/></svg>
					<span><strong>{$t("lens.dashedLine") || "Dashed"}</strong> — {$t("lens.untypedWikilink") || "untyped wikilink"}</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#ef4444" stroke-width="2" stroke-dasharray="4,3"/></svg>
					<span><strong>{$t("lens.redDashed") || "Red dashed"}</strong> — {$t("lens.blindSpotDesc") || "blind spot"}</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="12"><polygon points="2,6 6,1 14,1 18,6 14,11 6,11" fill="rgba(124,58,237,0.15)" stroke="#7c3aed" stroke-width="1.5"/></svg>
					<span><strong>{$t("lens.regionLabel") || "Region"}</strong> — {$t("lens.communityDesc") || "community"}</span>
				</div>
				<div class="cl-legend-divider"></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#4A9EFF" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkSupports') || 'supports'}</strong></span></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#FF4A4A" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkContradicts') || 'contradicts'}</strong></span></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#FF8C42" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkCauses') || 'causes'}</strong></span></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#4AFF88" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkExemplifies') || 'exemplifies'}</strong></span></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#C084FC" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkGeneralizes') || 'generalizes'}</strong></span></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#FACC15" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkDerivesFrom') || 'derives-from'}</strong></span></div>
				<div class="cl-legend-row"><svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#94A3B8" stroke-width="1.5"/></svg><span><strong>{$t('lens.linkPartOf') || 'part-of'}</strong></span></div>
			</div>
		</div>
		<div class="cl-panel-wrap">
			<LensPanel
				{health} {bridges} {communities} {communityProfiles} {contradictions} {gaps}
				nodeCount={nodes.length} edgeCount={links.length}
				onNoteClick={(id, name) => {
					const node = nodes.find(n => n.id === id);
					if (node) onNoteClick?.(node.path, node.name);
				}}
			/>
		</div>
	</div>
</div>

<style>
	.cl-root { display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; background: #fafafa; }
	.cl-header {
		display: flex; align-items: center; gap: 8px; padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border, #e5e7eb);
		background: var(--background-secondary, #f8fafc); flex-shrink: 0;
	}
	.cl-header svg { color: var(--text-muted, #64748b); }
	.cl-title { font-weight: 700; font-size: 14px; color: var(--text-normal, #1a1a1a); }
	.cl-stat { font-size: 12px; color: var(--text-muted, #64748b); }
	.cl-toolbar { display: flex; gap: 2px; margin-inline-start: auto; }
	.cl-toolbar-btn {
		width: 28px; height: 28px; border: none; border-radius: 4px;
		background: none; color: var(--text-muted); cursor: pointer;
		display: flex; align-items: center; justify-content: center;
	}
	.cl-toolbar-btn:hover { background: var(--background-modifier-hover, #f1f5f9); color: var(--text-normal); }
	.cl-toolbar-btn.active { background: var(--interactive-accent, #7c3aed); color: white; }

	/* Search bar */
	.cl-search-bar {
		position: absolute; top: 8px; inset-inline-start: 8px; z-index: 10;
		display: flex; align-items: center; gap: 6px;
		background: rgba(255,255,255,0.95); border: 1px solid #e5e7eb;
		border-radius: 8px; padding: 6px 10px; box-shadow: 0 2px 8px rgba(0,0,0,0.08);
	}
	.cl-search-bar svg { color: #64748b; flex-shrink: 0; }
	.cl-search-bar input {
		border: none; outline: none; background: none; font-size: 12px;
		font-family: inherit; color: #1a1a1a; width: 160px;
	}
	.cl-search-bar button {
		border: none; background: none; color: #64748b; cursor: pointer; font-size: 14px; padding: 0 2px;
	}
	.cl-search-scope { display: flex; gap: 1px; flex-shrink: 0; }
	.cl-search-scope button {
		padding: 2px 6px; font-size: 9px; border: 1px solid #e5e7eb; border-radius: 3px;
		background: none; color: #64748b; cursor: pointer; font-family: inherit;
	}
	.cl-search-scope button:hover { background: #f1f5f9; }
	.cl-search-scope button.active { background: #7c3aed; color: white; border-color: #7c3aed; }
	.cl-search-found { font-size: 10px; color: #64748b; white-space: nowrap; }
	.cl-search-type { font-size: 9px; color: white; padding: 1px 5px; border-radius: 4px; white-space: nowrap; text-transform: capitalize; }
	.cl-search-none { font-size: 10px; color: #ef4444; white-space: nowrap; }
	.cl-search-nav { border: none; background: none; color: #64748b; cursor: pointer; padding: 0 2px; display: flex; align-items: center; }
	.cl-search-nav:hover { color: #1a1a1a; }

	/* Settings panel */
	.cl-settings {
		position: absolute; top: 8px; inset-inline-end: 8px; z-index: 10;
		background: rgba(255,255,255,0.95); border: 1px solid #e5e7eb;
		border-radius: 8px; padding: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.08);
		width: 200px; display: flex; flex-direction: column; gap: 8px;
	}
	.cl-settings-title { font-size: 12px; font-weight: 700; color: #1a1a1a; }
	.cl-settings label { display: flex; flex-direction: column; gap: 2px; }
	.cl-settings label span { font-size: 10px; color: #64748b; }
	.cl-settings input[type="range"] { width: 100%; }
	.cl-settings-toggle { flex-direction: row; align-items: center; justify-content: space-between; }
	.cl-settings-toggle button {
		padding: 2px 10px; border-radius: 4px; font-size: 10px; cursor: pointer;
		border: 1px solid #e5e7eb; background: none; color: #64748b; font-family: inherit;
	}
	.cl-settings-toggle button.active { background: #7c3aed; color: white; border-color: #7c3aed; }
	.cl-close {
		width: 26px; height: 26px; border: none; border-radius: 4px;
		background: none; color: var(--text-muted); cursor: pointer; font-size: 16px;
		display: flex; align-items: center; justify-content: center;
	}
	.cl-close:hover { background: #ef4444; color: white; }

	.cl-body { flex: 1; display: flex; overflow: hidden; }
	.cl-canvas-wrap { flex: 1; position: relative; overflow: hidden; }
	.cl-canvas-wrap canvas { display: block; width: 100%; height: 100%; cursor: grab; }
	.cl-canvas-wrap canvas:active { cursor: grabbing; }
	.cl-panel-wrap {
		flex-shrink: 0; overflow-y: auto; overflow-x: hidden;
		border-inline-start: 1px solid var(--background-modifier-border, #e5e7eb);
	}

	/* Legend */
	.cl-legend {
		position: absolute; bottom: 16px; inset-inline-start: 16px; z-index: 5;
		background: rgba(255,255,255,0.95); border: 1px solid #e5e7eb;
		border-radius: 8px; padding: 10px 14px; font-size: 10px;
		box-shadow: 0 2px 8px rgba(0,0,0,0.08);
		display: flex; flex-direction: column; gap: 4px; max-width: 220px;
	}
	.cl-legend-title { font-size: 11px; font-weight: 700; color: #1a1a1a; margin-bottom: 2px; }
	.cl-legend-row { display: flex; align-items: center; gap: 5px; color: #64748b; font-size: 10px; }
	.cl-legend-row strong { color: #1a1a1a; font-weight: 600; }
	.cl-lg-circle { width: 8px; height: 8px; border-radius: 50%; background: #7c3aed; flex-shrink: 0; display: inline-block; }
	.cl-lg-big { width: 14px; height: 14px; }
	.cl-lg-small { width: 5px; height: 5px; }
	.cl-legend-divider { height: 1px; background: #e5e7eb; margin: 2px 0; }
</style>

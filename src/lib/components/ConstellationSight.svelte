<script lang="ts">
	/**
	 * ConstellationSight — Standalone network analysis visualization.
	 * D3.js force simulation + HTML5 Canvas rendering.
	 * Visually distinct from Sky View (GraphMind/Pixi.js).
	 */
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import * as d3 from 'd3';
	import { t, dir, getSearchOps } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { readSearchHistory, addSearchHistory } from '$lib/libraries/searchHistory';
	import type { SkyNode, SkyLink } from '$lib/libraries/store';
	import type { ClusterInfo, StructuralGap, UniverseHealth } from '$lib/graph/clusterEngine';
	import LensPanel from './LensPanel.svelte';

	let {
		nodes = [] as SkyNode[],
		links = [] as SkyLink[],
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
		searchMatchIds = null as Set<string> | null,
	}: {
		nodes?: SkyNode[];
		links?: SkyLink[];
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
		onNoteClick?: (path: string, name: string, highlightTerm?: string) => void;
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
		weight?: number;
		confidence?: string;
		annotation?: string;
		traversalCount?: number;
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
	let searchScope = $state<'all' | 'title' | 'content' | 'tag' | 'property' | 'semantic'>('all');
	interface SearchMatch { node: SimNode; matchType: string; matchCategories: string[]; }
	let searchResults = $state<SearchMatch[]>([]);
	let searchIdx = $state(0);
	let showChips = $state(false);
	let showHistory = $state(false);
	let historyItems = $state<{ query: string; timestamp: number }[]>([]);

	const syntaxChips = $derived.by(() => {
		const _locale = $t('searchHub.linksTo');
		const ops = getSearchOps();
		return [
			{ label: 'linksTo', syntax: (ops?.linksTo ?? 'links to') + ' [[' },
			{ label: 'linksFrom', syntax: (ops?.linksFrom ?? 'links from') + ' [[' },
			{ label: 'mutual', syntax: (ops?.mutual ?? 'mutual') + ' [[' },
			{ label: 'mentions', syntax: (ops?.mentions ?? 'mentions') + ' [[' },
			{ label: 'orphans', syntax: ops?.orphans ?? 'orphans' },
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

	// ─── Enrich links with Living Link data from note_links table ───
	async function enrichLinksFromDB() {
		try {
			const stats: any = await invoke('constellation_link_stats');
			if (!stats?.sample_links) return;
			// Build lookup: "source::target" → link data
			// Query all links for visible notes
			const allLinkData: any[] = await invoke('constellation_formulation_analysis', {
				queryType: 'most_connected', target: null
			});
			// For now, use link stats to get a sense of the data
			// Full enrichment: query note_links for all displayed connections
		} catch {}
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

		// 3. Edges — Living Link visualization:
		//    Typed: solid colored lines with arrowheads, thickness = weight
		//    Untyped: dashed subtle lines
		//    Confidence: hypothesis=thin, evidence=medium, established=thick
		for (const link of simLinks) {
			const src = link.source as SimNode;
			const tgt = link.target as SimNode;
			const sx = src.x ?? 0, sy = src.y ?? 0;
			const tx = tgt.x ?? 0, ty = tgt.y ?? 0;
			const typed = link.linkType && LINK_TYPE_COLORS[link.linkType];
			const color = typed ? LINK_TYPE_COLORS[link.linkType!] : '#94a3b8';

			// Weight-based thickness (default 1.0, grows with traversal)
			const w = link.weight ?? 1.0;
			const baseWidth = typed ? Math.max(0.8, Math.min(3, w * 0.6)) : 0.5;

			// Confidence-based style
			const conf = link.confidence ?? 'hypothesis';
			if (conf === 'hypothesis' && typed) {
				ctx.setLineDash([4 / zoom, 3 / zoom]); // hypothesis = dashed even for typed
			} else if (!typed) {
				ctx.setLineDash([3 / zoom, 3 / zoom]);
			}

			ctx.beginPath();
			ctx.moveTo(sx, sy);
			ctx.lineTo(tx, ty);
			ctx.strokeStyle = typed ? color + 'B3' : color + '4D';
			ctx.lineWidth = baseWidth / zoom;
			ctx.stroke();
			ctx.setLineDash([]);

			// Arrowhead for typed links (shows direction)
			if (typed) {
				const dx = tx - sx, dy = ty - sy;
				const len = Math.sqrt(dx * dx + dy * dy);
				if (len > 0) {
					const ux = dx / len, uy = dy / len;
					// Arrow at 70% along the line (not at the target node)
					const ax = sx + dx * 0.7, ay = sy + dy * 0.7;
					const arrowSize = 4 / zoom;
					ctx.beginPath();
					ctx.moveTo(ax + ux * arrowSize, ay + uy * arrowSize);
					ctx.lineTo(ax - uy * arrowSize * 0.5 - ux * arrowSize * 0.3, ay + ux * arrowSize * 0.5 - uy * arrowSize * 0.3);
					ctx.lineTo(ax + uy * arrowSize * 0.5 - ux * arrowSize * 0.3, ay - ux * arrowSize * 0.5 - uy * arrowSize * 0.3);
					ctx.closePath();
					ctx.fillStyle = color + 'CC';
					ctx.fill();
				}
			}
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
				const MATCH_COLORS: Record<string, string> = { title: '#3b82f6', content: '#16a34a', both: '#7c3aed', T: '#3b82f6', C: '#16a34a', '#': '#f472b6', P: '#f59e0b', S: '#7c3aed', match: '#94a3b8' };

				for (const sn of simNodes) {
					const matchType = matchMap.get(sn.id);
					if (!matchType) continue;
					const sx = sn.x ?? 0, sy = sn.y ?? 0;
					const isCurrent = sn.id === currentId;
					const firstCat = matchType.split('·')[0] || 'match';
					const color = MATCH_COLORS[firstCat] || '#7c3aed';

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
			// Pass search query as highlight term if search is active
			const hl = searchQuery.trim() || undefined;
			onNoteClick(hoveredNode.path, hoveredNode.name, hl);
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
		addSearchHistory(searchQuery);
		historyItems = readSearchHistory();
		const q = searchQuery.toLowerCase();

		// 1. Title matches (instant, local) — for 'all' and 'title' scopes
		const titleMatchIds = new Set<string>();
		if (searchScope === 'all' || searchScope === 'title') {
			for (const n of simNodes) {
				if (n.name.toLowerCase().includes(q)) titleMatchIds.add(n.id);
			}
		}

		// 2. Backend search — uses universalSearch for categorized results
		const contentMatchIds = new Set<string>();
		const tagMatchIds = new Set<string>();
		const propertyMatchIds = new Set<string>();
		const semanticMatchIds = new Set<string>();
		if (searchScope !== 'title') {
			try {
				const { universalSearch, embedText, appSettings, canonicalizeSearchQuery, stripInvisibleChars } = await import('$lib/libraries/store');
				const { getSearchOps } = await import('$lib/i18n');
				const { get } = await import('svelte/store');
				const cleanQ = stripInvisibleChars(searchQuery);
				const canonicalized = canonicalizeSearchQuery(cleanQ, getSearchOps());

				// Get semantic embedding if enabled
				let qEmbed: number[] | null = null;
				if ((searchScope === 'all' || searchScope === 'semantic') && get(appSettings).enabledFeatures?.semanticSearch) {
					try { qEmbed = await embedText(canonicalized); } catch {}
				}

				const resp = await universalSearch(canonicalized, qEmbed, 200);
				// Categorize by scope
				if (searchScope === 'all' || searchScope === 'content') {
					for (const r of (resp as any).contents ?? []) contentMatchIds.add(r.name.toLowerCase());
				}
				if (searchScope === 'all' || searchScope === 'tag') {
					for (const r of (resp as any).tags ?? []) tagMatchIds.add(r.name.toLowerCase());
				}
				if (searchScope === 'all' || searchScope === 'property') {
					for (const r of (resp as any).properties ?? []) propertyMatchIds.add(r.name.toLowerCase());
				}
				if (searchScope === 'all' || searchScope === 'semantic') {
					for (const r of (resp as any).semantic ?? []) semanticMatchIds.add(r.name.toLowerCase());
				}
				if (searchScope === 'all') {
					// Also include title hits from backend
					for (const r of (resp as any).titles ?? []) titleMatchIds.add(r.name.toLowerCase());
				}
			} catch { /* fallback */ }
		}

		// 3. Classify each match
		const allIds = new Set([...titleMatchIds, ...contentMatchIds, ...tagMatchIds, ...propertyMatchIds, ...semanticMatchIds]);
		const nodeMap = new Map(simNodes.map(n => [n.id, n]));
		const matches: SearchMatch[] = [];

		for (const id of allIds) {
			const node = nodeMap.get(id);
			if (!node) continue;
			const cats: string[] = [];
			if (titleMatchIds.has(id)) cats.push('T');
			if (contentMatchIds.has(id)) cats.push('C');
			if (tagMatchIds.has(id)) cats.push('#');
			if (propertyMatchIds.has(id)) cats.push('P');
			if (semanticMatchIds.has(id)) cats.push('S');
			const matchType = cats.join('·') || 'match';
			matches.push({ node, matchType, matchCategories: cats });
		}

		matches.sort((a, b) => {
			const order: Record<string, number> = {}; // sort by category count (more = higher)
			return (b.matchCategories?.length ?? 0) - (a.matchCategories?.length ?? 0);
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

	function resetSearch() {
		searchQuery = '';
		searchResults = [];
		searchIdx = 0;
	}
	function closeSearch() {
		resetSearch();
		showChips = false;
		showHistory = false;
		searchVisible = false;
	}

	function insertChipSyntax(syntax: string) {
		searchQuery = searchQuery ? searchQuery + ' ' + syntax : syntax;
		showChips = false;
	}

	function selectHistory(item: { query: string }) {
		searchQuery = item.query;
		showHistory = false;
		executeSearch();
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
					matches.push({ node: sn, matchType: 'match', matchCategories: ['T'] });
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
			<button class="cl-toolbar-btn" class:active={searchVisible} onclick={() => { searchVisible = !searchVisible; if (!searchVisible) closeSearch(); }} title={$t('layout.search') || 'Search'}>
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
						<button class:active={searchScope === 'title'} onclick={() => searchScope = 'title'}>{$t("searchHub.titles") || "Title"}</button>
						<button class:active={searchScope === 'content'} onclick={() => searchScope = 'content'}>{$t("searchHub.contents") || "Content"}</button>
						<button class:active={searchScope === 'tag'} onclick={() => searchScope = 'tag'}>{$t("searchHub.tags") || "Tags"}</button>
						<button class:active={searchScope === 'property'} onclick={() => searchScope = 'property'}>{$t("searchHub.properties") || "Props"}</button>
						<button class:active={searchScope === 'semantic'} onclick={() => searchScope = 'semantic'}>{$t("searchHub.semantic") || "Semantic"}</button>
					</div>
					<div class="cl-search-input-wrap">
						<input type="text" dir="auto" placeholder={searchScope === 'title' ? ($t('lens.searchTitles') || 'Search titles...') : searchScope === 'content' ? ($t('lens.searchContent') || 'Search content...') : ($t('lens.searchAll') || 'Search all... (Enter)')} bind:value={searchQuery}
							onfocus={() => { if (!searchQuery) { historyItems = readSearchHistory(); showHistory = true; } }}
							onblur={() => setTimeout(() => { showHistory = false; }, 200)}
							oninput={() => { showHistory = false; }}
							onkeydown={(e) => {
								if (e.key === 'Enter') { e.preventDefault(); searchResults.length > 0 ? (e.shiftKey ? prevSearchResult() : nextSearchResult()) : executeSearch(); }
								if (e.key === 'Escape') { closeSearch(); e.stopPropagation(); }
							}} />
						<button onclick={resetSearch} title={$t('common.clear') || 'Reset'}>×</button>
						<button class="cl-chips-btn" class:active={showChips} onclick={() => showChips = !showChips} title={$t('searchHub.syntaxHelpers') || 'Syntax'}>
							<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
						</button>
						<!-- Search history -->
						{#if showHistory && historyItems.length > 0 && !searchQuery}
							<div class="cl-search-dropdown">
								{#each historyItems.slice(0, 8) as item}
									<button class="cl-dropdown-item" onclick={() => selectHistory(item)} dir="auto">{item.query}</button>
								{/each}
							</div>
						{/if}
						<!-- Syntax chips -->
						{#if showChips}
							<div class="cl-search-dropdown cl-chips-grid">
								{#each syntaxChips as chip}
									<button class="cl-chip" onclick={() => insertChipSyntax(chip.syntax)}>{$t(`searchHub.${chip.label}`)}</button>
								{/each}
							</div>
						{/if}
					</div>
					{#if searchResults.length > 0}
						{@const currentMatch = searchResults[searchIdx]}
						{#each (currentMatch?.matchCategories ?? []) as cat}
							<span class="cl-search-type" style="background:{cat === 'T' ? '#3b82f6' : cat === 'C' ? '#16a34a' : cat === '#' ? '#f472b6' : cat === 'P' ? '#f59e0b' : cat === 'S' ? '#7c3aed' : '#94a3b8'}">{cat}</span>
						{/each}
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
				<div class="cl-legend-title" style="margin-top:4px">{$t('searchHub.linksTo') || 'Link Types'}</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#4A9EFF" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#4A9EFF"/></svg>
					<span>supports</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#FF4A4A" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#FF4A4A"/></svg>
					<span>contradicts</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#FF8C42" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#FF8C42"/></svg>
					<span>causes</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#4AFF88" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#4AFF88"/></svg>
					<span>exemplifies</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#C084FC" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#C084FC"/></svg>
					<span>generalizes</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="16" y2="2" stroke="#FACC15" stroke-width="2"/><polygon points="16,0 20,2 16,4" fill="#FACC15"/></svg>
					<span>derives-from</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#94A3B8" stroke-width="1" stroke-dasharray="3,2"/></svg>
					<span>hypothesis (unverified)</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#4A9EFF" stroke-width="3"/></svg>
					<span>established (high weight)</span>
				</div>
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
		font-family: inherit; color: #1a1a1a; min-width: 200px;
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
	.cl-search-input-wrap { position: relative; display: flex; align-items: center; gap: 4px; flex: 1; min-width: 200px; }
	.cl-search-input-wrap input { flex: 1; min-width: 0; }
	.cl-chips-btn { border: none; background: none; cursor: pointer; color: #64748b; padding: 2px; border-radius: 3px; }
	.cl-chips-btn:hover, .cl-chips-btn.active { color: var(--interactive-accent, #7c3aed); background: rgba(124,58,237,0.1); }
	.cl-search-dropdown { position: absolute; top: 100%; inset-inline-start: 0; z-index: 100; background: rgba(255,255,255,0.98); border: 1px solid #e5e7eb; border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12); min-width: 200px; max-height: 250px; overflow-y: auto; margin-top: 4px; }
	.cl-dropdown-item { display: block; width: 100%; text-align: start; padding: 6px 12px; border: none; background: none; cursor: pointer; font-size: 11px; color: #1a1a1a; }
	.cl-dropdown-item:hover { background: #f1f5f9; }
	.cl-chips-grid { display: flex; flex-wrap: wrap; gap: 6px; padding: 10px; max-width: 450px; }
	.cl-chip { padding: 4px 10px; border-radius: 6px; border: 1.5px solid #d1d5db; background: #f9fafb; color: #374151; font-size: 11px; font-weight: 500; cursor: pointer; white-space: nowrap; transition: all 0.12s; }
	.cl-chip:hover { border-color: var(--interactive-accent, #7c3aed); color: var(--interactive-accent, #7c3aed); background: rgba(124,58,237,0.06); box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
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

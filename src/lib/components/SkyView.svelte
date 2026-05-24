<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { t } from '$lib/i18n';
	import * as d3 from 'd3';
	// MIG-044 Phase 2 — NSC summary headline in the hover tooltip.
	import { getSummaryFor } from '$lib/nsc/summaryStore';

	interface SkyNode extends d3.SimulationNodeDatum {
		id: string;
		name: string;
		path: string;
		libraryName: string;
		group?: string;
		linkCount: number;
	}

	interface SkyLink {
		source: string;
		target: string;
		linkType?: string;
	}

	const LINK_TYPE_COLORS: Record<string, string> = {
		'related-to': '#3b82f6',
		'prerequisite': '#ef4444',
		'see-also': '#10b981',
		'contradicts': '#f59e0b',
		'supports': '#8b5cf6',
		'extends': '#ec4899',
	};

	const GROUP_COLORS = [
		'#3b82f6', '#10b981', '#f59e0b', '#ef4444',
		'#8b5cf6', '#ec4899', '#06b6d4', '#84cc16',
	];

	let {
		nodes = [] as SkyNode[],
		links = [] as SkyLink[],
		onNodeClick,
		activeNodeId = '',
		compact = false,
	}: {
		nodes: SkyNode[];
		links: SkyLink[];
		onNodeClick: (path: string, libraryName: string) => void;
		activeNodeId?: string;
		compact?: boolean;
	} = $props();

	let containerEl: HTMLDivElement;
	let canvasEl: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let simulation: d3.Simulation<any, any> | null = null;
	let currentTransform = d3.zoomIdentity;
	let dpr = 1;

	let nodeData: any[] = [];
	let linkData: any[] = [];
	let hoveredNode: any = null;
	let draggedNode: any = null;
	let dragMoved = false;
	let prevActiveNodeId = '';
	let mounted = false;
	let centerRaf: number | null = null;

	// Tooltip
	let tooltipText = $state('');
	let tooltipX = $state(0);
	let tooltipY = $state(0);
	let tooltipVisible = $state(false);
	// MIG-044 Phase 2 — NSC summary headline shown under the node name in
	// the hover tooltip. The architect doc mentioned an "inspector"; SkyView
	// has no rich inspector — its on-canvas node info IS the tooltip, so the
	// headline lands there. Click still opens the note in the editor (which
	// has the Phase-1 summary band).
	let tooltipHeadline = $state('');
	let tooltipHeadlinePath = ''; // last node path fetched — skip refetch on jitter
	let tooltipHeadlineToken = 0; // monotonic; stale promises ignored

	// Legend click bounds
	let legendBounds: { x: number; y: number; w: number; h: number; lineH: number; padY: number } | null = null;

	// Library colors
	const LIBRARY_COLORS = [
		'#8b5cf6', '#3b82f6', '#10b981', '#f59e0b',
		'#ef4444', '#ec4899', '#06b6d4', '#84cc16',
	];
	let libraryColorMap = new Map<string, string>();
	let libraryList: { name: string; color: string; count: number }[] = $state([]);
	let nodeCountInfo = $state(''); // "Showing 400 of 1228 nodes"
	let hiddenLibraries: Set<string> = $state(new Set());

	// ─── Controls Panel State ───

	let showControls = $state(false);

	// Sections collapsed state
	let filtersOpen = $state(true);
	let groupsOpen = $state(true);
	let displayOpen = $state(true);
	let forcesOpen = $state(true);

	// Filters
	let filterQuery = $state('');
	let showOrphans = $state(true);
	let existingOnly = $state(false);

	// Groups
	let groups: { query: string; color: string }[] = $state([]);

	// Display
	let showArrows = $state(false);
	let textFadeThreshold = $state(1.5);
	let nodeSizeMultiplier = $state(4);
	let linkThicknessMultiplier = $state(1);
	let animating = $state(!compact);

	// Forces
	let centerForce = $state(0.5);
	let repelForce = $state(80);
	let linkForce = $state(1);
	let linkDistance = $state(60);

	function resetControls() {
		filterQuery = '';
		showOrphans = true;
		existingOnly = false;
		groups = [];
		showArrows = false;
		textFadeThreshold = 1.5;
		nodeSizeMultiplier = 4;
		linkThicknessMultiplier = 1;
		animating = true;
		centerForce = 0.5;
		repelForce = 80;
		linkForce = 1;
		linkDistance = 60;
		renderGraph();
	}

	function addGroup() {
		groups = [...groups, { query: '', color: GROUP_COLORS[groups.length % GROUP_COLORS.length] }];
	}

	function removeGroup(index: number) {
		groups = groups.filter((_, i) => i !== index);
		renderGraph();
	}

	function updateGroupQuery(index: number, query: string) {
		groups = groups.map((g, i) => i === index ? { ...g, query } : g);
		renderGraph();
	}

	function updateGroupColor(index: number, color: string) {
		groups = groups.map((g, i) => i === index ? { ...g, color } : g);
		renderGraph();
	}

	// ─── Reactivity: re-render on control changes ───

	// Filter changes need full re-render (rebuild nodeData/linkData)
	// Skip in compact mode — filters are hidden
	$effect(() => {
		const _ = [filterQuery, showOrphans, existingOnly];
		if (mounted && !compact && nodes.length > 0) untrack(() => renderGraph());
	});

	// Display changes only need redraw (no simulation rebuild)
	$effect(() => {
		const _ = [showArrows, textFadeThreshold, nodeSizeMultiplier, linkThicknessMultiplier];
		if (mounted && !compact && nodeData.length > 0) untrack(() => draw());
	});

	// Force changes update simulation dynamically
	$effect(() => {
		const _ = [centerForce, repelForce, linkForce, linkDistance];
		if (!compact && simulation) untrack(() => updateForces());
	});

	// Animation toggle — skip in compact mode (always static)
	$effect(() => {
		const anim = animating;
		if (compact) return;
		untrack(() => {
			if (!anim && simulation) {
				simulation.stop();
				// Use batched async ticks to avoid blocking the main thread
				batchedTick(simulation, 300, () => draw());
			} else if (anim && simulation) {
				simulation.alpha(0.3).restart();
			}
		});
	});

	/** Run N simulation ticks in small batches to avoid freezing the UI */
	function batchedTick(sim: d3.Simulation<any, any>, total: number, onDone: () => void) {
		const BATCH = 30;
		let done = 0;
		function step() {
			const end = Math.min(done + BATCH, total);
			for (let i = done; i < end; i++) sim.tick();
			done = end;
			draw(); // progressive render
			if (done < total) {
				requestAnimationFrame(step);
			} else {
				onDone();
			}
		}
		requestAnimationFrame(step);
	}

	function updateForces() {
		if (!simulation || !containerEl) return;
		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;

		const center = simulation.force('center') as d3.ForceCenter<any> | undefined;
		if (center) {
			center.strength(centerForce);
		}

		const charge = simulation.force('charge') as d3.ForceManyBody<any> | undefined;
		if (charge) {
			charge.strength(-repelForce);
		}

		const link = simulation.force('link') as d3.ForceLink<any, any> | undefined;
		if (link) {
			link.strength(linkForce);
			link.distance(linkDistance);
		}

		simulation.alpha(0.3).restart();
	}

	// ─── Library colors ───

	function assignLibraryColors() {
		const libraryCounts = new Map<string, number>();
		const rawNodes = Array.from(nodes);
		for (const n of rawNodes) {
			libraryCounts.set(n.libraryName, (libraryCounts.get(n.libraryName) || 0) + 1);
		}
		const libraryNames = [...libraryCounts.keys()];
		libraryColorMap = new Map();
		const list: { name: string; color: string; count: number }[] = [];
		libraryNames.forEach((v, i) => {
			const color = LIBRARY_COLORS[i % LIBRARY_COLORS.length];
			libraryColorMap.set(v, color);
			list.push({ name: v, color, count: libraryCounts.get(v) || 0 });
		});
		libraryList = list;
	}

	function toggleLibraryVisibility(libraryName: string) {
		const next = new Set(hiddenLibraries);
		if (next.has(libraryName)) next.delete(libraryName);
		else next.add(libraryName);
		hiddenLibraries = next;
		renderGraph();
	}

	function getNodeRadius(d: any): number {
		const base = Math.max(4, Math.min(14, 3 + d.linkCount * 1.5));
		return base * (nodeSizeMultiplier / 4);
	}

	function getNodeColor(d: any): string {
		// Check groups first — first matching group wins
		for (const g of groups) {
			if (g.query.trim() && matchesQuery(d, g.query)) {
				return g.color;
			}
		}
		return libraryColorMap.get(d.libraryName) || '#6b7280';
	}

	function matchesQuery(node: any, query: string): boolean {
		const q = query.toLowerCase().trim();
		if (!q) return false;

		// Support path: prefix
		if (q.startsWith('path:')) {
			const pathFilter = q.slice(5).trim();
			return node.path.toLowerCase().includes(pathFilter);
		}
		// Support tag: prefix (if we had tag data on nodes)
		if (q.startsWith('tag:')) {
			return false; // tags not on nodes currently
		}
		// Default: match name or path
		return node.name.toLowerCase().includes(q) || node.path.toLowerCase().includes(q);
	}

	function isDarkTheme(): boolean {
		return document.body.classList.contains('theme-dark');
	}

	function getCSSVar(name: string): string {
		return getComputedStyle(document.body).getPropertyValue(name).trim();
	}

	// ─── Lifecycle ───

	let themeObserver: MutationObserver | null = null;
	let resizeObserver: ResizeObserver | null = null;

	function handleResize() {
		if (!containerEl || !canvasEl || !ctx || nodeData.length === 0) return;
		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		dpr = window.devicePixelRatio || 1;
		canvasEl.width = width * dpr;
		canvasEl.height = height * dpr;
		canvasEl.style.width = width + 'px';
		canvasEl.style.height = height + 'px';
		draw();
	}

	onMount(() => {
		mounted = true;
		ctx = canvasEl?.getContext('2d') || null;

		themeObserver = new MutationObserver(() => {
			if (nodeData.length > 0) draw();
		});
		themeObserver.observe(document.body, {
			attributes: true,
			attributeFilter: ['class'],
		});

		// Resize observer to handle window maximize/resize
		resizeObserver = new ResizeObserver(() => {
			handleResize();
		});
		if (containerEl) resizeObserver.observe(containerEl);

		// Render once on mount (deferred to avoid blocking)
		if (nodes.length > 0) {
			requestAnimationFrame(() => renderGraph());
		}
	});

	$effect(() => {
		if (compact) return; // In compact mode, active node is always centered by renderGraph
		if (activeNodeId && activeNodeId !== prevActiveNodeId && nodeData.length > 0) {
			prevActiveNodeId = activeNodeId;
			centerOnNode(activeNodeId);
		}
	});

	// ─── Center on node ───

	function centerOnNode(nodeId: string) {
		const target = nodeData.find((n: any) => n.id === nodeId);
		if (!target || target.x == null || target.y == null) return;

		// Cancel any in-progress centering animation
		if (centerRaf !== null) { cancelAnimationFrame(centerRaf); centerRaf = null; }

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		const scale = 1.5;
		const tx = width / 2 - target.x * scale;
		const ty = height / 2 - target.y * scale;

		const from = { x: currentTransform.x, y: currentTransform.y, k: currentTransform.k };
		const to = { x: tx, y: ty, k: scale };
		const duration = 750;
		const start = performance.now();

		function animate(time: number) {
			centerRaf = null;
			const t = Math.min(1, (time - start) / duration);
			const e = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
			currentTransform = d3.zoomIdentity
				.translate(from.x + (to.x - from.x) * e, from.y + (to.y - from.y) * e)
				.scale(from.k + (to.k - from.k) * e);
			draw();
			if (t < 1) centerRaf = requestAnimationFrame(animate);
		}
		centerRaf = requestAnimationFrame(animate);
	}

	// ─── Cleanup helper ───

	function cleanup() {
		if (centerRaf !== null) { cancelAnimationFrame(centerRaf); centerRaf = null; }
		if (simulation) {
			simulation.stop();
			simulation.on('tick', null);
			simulation.on('end', null);
			simulation = null;
		}
		if (canvasEl) {
			d3.select(canvasEl).on('.zoom', null);
		}
		hoveredNode = null;
		draggedNode = null;
	}

	// ─── Main render ───

	function renderGraph() {
		if (!containerEl || !canvasEl || !ctx) return;

		cleanup();

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		dpr = window.devicePixelRatio || 1;

		canvasEl.width = width * dpr;
		canvasEl.height = height * dpr;
		canvasEl.style.width = width + 'px';
		canvasEl.style.height = height + 'px';

		assignLibraryColors();

		// In compact mode, use a simple radial layout — no d3 simulation
		if (compact) {
			nodeData = nodes.map(n => ({ ...n }));
			linkData = links.map(l => ({ ...l }));

			// Place active node at center, others in a circle around it
			const cx = width / 2;
			const cy = height / 2;
			const radius = Math.min(width, height) * 0.3;
			const others = nodeData.filter(n => n.id !== activeNodeId);
			const center = nodeData.find(n => n.id === activeNodeId);

			if (center) {
				center.x = cx;
				center.y = cy;
			}
			others.forEach((n, i) => {
				const a = (2 * Math.PI * i) / Math.max(others.length, 1) - Math.PI / 2;
				n.x = cx + Math.cos(a) * radius;
				n.y = cy + Math.sin(a) * radius;
			});

			// Resolve link references (d3-style: source/target become node objects)
			const nodeById = new Map(nodeData.map(n => [n.id, n]));
			for (const l of linkData) {
				const s = nodeById.get(l.source as string);
				const t = nodeById.get(l.target as string);
				if (s) l.source = s;
				if (t) l.target = t;
			}

			draw();
			return;
		}

		// ─── Full mode: d3 force simulation ───
		// Defer ALL heavy work to avoid blocking the UI thread.
		// d3.forceSimulation() + forceLink() initialization is O(N+E) and
		// synchronously blocks with 1885 nodes + thousands of links.
		// Cap nodes to prevent UI freeze. d3 handles ~400 nodes smoothly.
		const MAX_INITIAL_NODES = 400;

		let filteredNodes = nodes.filter(n => !hiddenLibraries.has(n.libraryName));
		if (filterQuery.trim()) {
			filteredNodes = filteredNodes.filter(n => matchesQuery(n, filterQuery));
		}
		if (!showOrphans) {
			filteredNodes = filteredNodes.filter(n => n.linkCount > 0);
		}

		// Sort by link count (most connected first) and cap
		const totalFiltered = filteredNodes.length;
		if (filteredNodes.length > MAX_INITIAL_NODES) {
			filteredNodes = [...filteredNodes].sort((a, b) => b.linkCount - a.linkCount).slice(0, MAX_INITIAL_NODES);
			nodeCountInfo = `${MAX_INITIAL_NODES} / ${totalFiltered}`;
		} else {
			nodeCountInfo = `${totalFiltered}`;
		}

		const visibleIds = new Set(filteredNodes.map(n => n.id));
		const visibleLinks = links.filter(l => visibleIds.has(l.source as string) && visibleIds.has(l.target as string));

		nodeData = filteredNodes.map(n => ({ ...n }));
		linkData = visibleLinks.map(l => ({ ...l }));

		// Compute library cluster centers
		const libraryNames = [...new Set(filteredNodes.map(n => n.libraryName))];
		const libraryCenters = new Map<string, { x: number; y: number }>();
		const clusterAngle = (2 * Math.PI) / Math.max(libraryNames.length, 1);
		const clusterRadius = Math.min(width, height) * 0.2;
		libraryNames.forEach((v, i) => {
			libraryCenters.set(v, {
				x: width / 2 + Math.cos(clusterAngle * i) * clusterRadius,
				y: height / 2 + Math.sin(clusterAngle * i) * clusterRadius,
			});
		});

		simulation = d3.forceSimulation(nodeData)
			.stop()
			.force('link', d3.forceLink(linkData).id((d: any) => d.id).distance(linkDistance).strength(linkForce))
			.force('charge', d3.forceManyBody().strength(-repelForce).theta(1.2))
			.force('center', d3.forceCenter(width / 2, height / 2).strength(centerForce))
			.force('collision', d3.forceCollide().radius(6))
			.force('clusterX', d3.forceX((d: any) => libraryCenters.get(d.libraryName)?.x ?? width / 2).strength(0.05))
			.force('clusterY', d3.forceY((d: any) => libraryCenters.get(d.libraryName)?.y ?? height / 2).strength(0.05))
			.alphaDecay(0.08)
			.velocityDecay(0.5);

		simulation.on('end', () => {
			draw();
			if (activeNodeId) {
				prevActiveNodeId = activeNodeId;
				centerOnNode(activeNodeId);
			}
		});

		// Zoom
		const zoomBehavior = d3.zoom<HTMLCanvasElement, unknown>()
			.scaleExtent([0.1, 8])
			.filter((event) => {
				if (event.type === 'mousedown') {
					const [sx, sy] = currentTransform.invert([event.offsetX, event.offsetY]);
					const node = findNodeAt(sx, sy);
					if (node) return false;
				}
				return !event.ctrlKey && event.button !== 2;
			})
			.on('zoom', (event) => {
				currentTransform = event.transform;
				draw();
			});

		d3.select(canvasEl).call(zoomBehavior as any);
		canvasEl.onmousedown = handleMouseDown;
		canvasEl.onmousemove = handleMouseMove;
		canvasEl.onmouseup = handleMouseUp;
		canvasEl.onmouseleave = handleMouseLeave;
		canvasEl.onclick = handleClick;

		// Start ticking progressively
		if (animating) {
			batchedTick(simulation, 150, () => {
				if (simulation) {
					simulation.on('tick', () => draw());
					simulation.alpha(0.1).restart();
				}
			});
		} else {
			batchedTick(simulation, 200, () => draw());
		}
	}

	function findNodeAt(sx: number, sy: number): any {
		return simulation?.find(sx, sy, 20) || null;
	}

	// ─── Mouse interaction ───

	function handleMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		dragMoved = false;
		const [sx, sy] = currentTransform.invert([e.offsetX, e.offsetY]);
		const node = findNodeAt(sx, sy);
		if (node) {
			draggedNode = node;
			draggedNode.fx = draggedNode.x;
			draggedNode.fy = draggedNode.y;
			simulation?.alphaTarget(0.3).restart();
			canvasEl.style.cursor = 'grabbing';
		}
	}

	function handleMouseMove(e: MouseEvent) {
		if (draggedNode) {
			dragMoved = true;
			const [sx, sy] = currentTransform.invert([e.offsetX, e.offsetY]);
			draggedNode.fx = sx;
			draggedNode.fy = sy;
			return;
		}

		const [sx, sy] = currentTransform.invert([e.offsetX, e.offsetY]);
		const node = findNodeAt(sx, sy);

		if (node !== hoveredNode) {
			hoveredNode = node;
			draw();
		}

		if (node) {
			tooltipText = node.name;
			tooltipX = e.offsetX + 14;
			tooltipY = e.offsetY - 10;
			tooltipVisible = true;
			canvasEl.style.cursor = 'pointer';
			// MIG-044 Phase 2 — fetch the headline lazily for the hovered
			// node. Cache-first via the shared store, so re-hover is free.
			// Stale-promise guard via a monotonic token so the LATEST hover
			// wins regardless of fetch latency.
			if (node.path !== tooltipHeadlinePath) {
				tooltipHeadlinePath = node.path;
				tooltipHeadline = ''; // clear stale text immediately
				const myToken = ++tooltipHeadlineToken;
				const targetPath = node.path;
				getSummaryFor(targetPath).then((entry) => {
					if (myToken !== tooltipHeadlineToken) return; // superseded
					tooltipHeadline = entry?.headline ?? '';
				}).catch(() => { /* ignore */ });
			}
		} else {
			tooltipVisible = false;
			canvasEl.style.cursor = 'grab';
		}
	}

	function handleMouseUp(_e: MouseEvent) {
		if (draggedNode) {
			simulation?.alphaTarget(0);
			draggedNode.fx = null;
			draggedNode.fy = null;
			draggedNode = null;
			canvasEl.style.cursor = 'grab';
		}
	}

	function handleMouseLeave(_e: MouseEvent) {
		hoveredNode = null;
		tooltipVisible = false;
		// MIG-044 Phase 2 — drop the cached path so re-hover refetches.
		tooltipHeadlinePath = '';
		tooltipHeadline = '';
		draw();
	}

	function handleClick(e: MouseEvent) {
		if (dragMoved) { dragMoved = false; return; }

		// Check legend click
		if (legendBounds && libraryList.length > 1) {
			const mx = e.offsetX, my = e.offsetY;
			const lb = legendBounds;
			if (mx >= lb.x && mx <= lb.x + lb.w && my >= lb.y && my <= lb.y + lb.h) {
				const idx = Math.floor((my - lb.y - lb.padY) / lb.lineH);
				if (idx >= 0 && idx < libraryList.length) {
					toggleLibraryVisibility(libraryList[idx].name);
					return;
				}
			}
		}

		const [sx, sy] = currentTransform.invert([e.offsetX, e.offsetY]);
		const node = findNodeAt(sx, sy);
		if (node) {
			onNodeClick(node.path, node.libraryName);
		}
	}

	// ─── Canvas draw ───

	function draw() {
		if (!ctx || !canvasEl) return;
		const dark = isDarkTheme();

		const linkColor = getCSSVar('--star-line') || (dark ? '#585b70' : '#8b8b96');
		const linkAlpha = 0.4;
		const linkDimAlpha = dark ? 0.06 : 0.08;
		const ringColor = getCSSVar('--text-normal') || (dark ? '#ffffff' : '#1f2328');
		const ringConnectedColor = getCSSVar('--text-muted') || (dark ? '#ffffffaa' : '#5c5c66');
		const labelColor = getCSSVar('--star-text') || (dark ? '#cdd6f4' : '#1f2328');
		const labelShadow = getCSSVar('--star-text-shadow') || (dark ? '#000000' : '#ffffff');
		const legendBg = dark ? 'rgba(24, 24, 36, 0.9)' : 'rgba(255, 255, 255, 0.94)';
		const legendBorder = dark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)';
		const legendText = getCSSVar('--star-text') || (dark ? '#cdd6f4' : '#1f2328');

		ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);
		ctx.save();

		ctx.setTransform(
			currentTransform.k * dpr, 0, 0,
			currentTransform.k * dpr,
			currentTransform.x * dpr,
			currentTransform.y * dpr
		);

		const highlightId = hoveredNode?.id || activeNodeId || null;
		let connectedIds: Set<string> | null = null;
		let highlightColor = '#8b5cf6';

		if (highlightId) {
			connectedIds = new Set();
			for (const l of linkData) {
				const sid = typeof l.source === 'object' ? l.source.id : l.source;
				const tid = typeof l.target === 'object' ? l.target.id : l.target;
				if (sid === highlightId) connectedIds.add(tid);
				if (tid === highlightId) connectedIds.add(sid);
			}
			const hn = nodeData.find(n => n.id === highlightId);
			if (hn) highlightColor = getNodeColor(hn);
		}

		// Convex hulls for library clusters
		if (libraryList.length > 1) {
			const libraryPoints = new Map<string, [number, number][]>();
			for (const n of nodeData) {
				if (n.x == null) continue;
				const pts = libraryPoints.get(n.libraryName) || [];
				pts.push([n.x, n.y]);
				libraryPoints.set(n.libraryName, pts);
			}
			for (const [vName, pts] of libraryPoints) {
				if (pts.length < 3) continue;
				const hull = d3.polygonHull(pts);
				if (!hull) continue;
				const color = libraryColorMap.get(vName) || '#6b7280';
				ctx.beginPath();
				const cx = d3.mean(hull, p => p[0]) || 0;
				const cy = d3.mean(hull, p => p[1]) || 0;
				const pad = 20 / currentTransform.k;
				const expanded = hull.map(([px, py]) => {
					const dx = px - cx, dy = py - cy;
					const dist = Math.sqrt(dx * dx + dy * dy);
					return [px + (dx / dist) * pad, py + (dy / dist) * pad] as [number, number];
				});
				ctx.moveTo(expanded[0][0], expanded[0][1]);
				for (let i = 1; i < expanded.length; i++) {
					ctx.lineTo(expanded[i][0], expanded[i][1]);
				}
				ctx.closePath();
				ctx.fillStyle = color;
				ctx.globalAlpha = 0.04;
				ctx.fill();
				ctx.strokeStyle = color;
				ctx.lineWidth = 1 / currentTransform.k;
				ctx.globalAlpha = 0.15;
				ctx.setLineDash([]);
				ctx.stroke();
			}
		}

		// Links
		for (const l of linkData) {
			const src = l.source;
			const tgt = l.target;
			if (src.x == null || tgt.x == null) continue;

			const isHL = highlightId && (src.id === highlightId || tgt.id === highlightId);
			const isCrossLibrary = src.libraryName !== tgt.libraryName;
			const typedColor = l.linkType ? LINK_TYPE_COLORS[l.linkType] : null;

			ctx.beginPath();
			ctx.moveTo(src.x, src.y);
			ctx.lineTo(tgt.x, tgt.y);

			if (isCrossLibrary && !isHL) {
				ctx.setLineDash([4 / currentTransform.k, 3 / currentTransform.k]);
			} else {
				ctx.setLineDash([]);
			}

			const thickness = linkThicknessMultiplier;

			if (isHL) {
				ctx.strokeStyle = highlightColor;
				ctx.lineWidth = (2 * thickness) / currentTransform.k;
				ctx.globalAlpha = 0.9;
				ctx.setLineDash([]);
			} else if (typedColor) {
				ctx.strokeStyle = typedColor;
				ctx.lineWidth = (1.2 * thickness) / currentTransform.k;
				ctx.globalAlpha = highlightId ? linkDimAlpha : 0.6;
			} else if (isCrossLibrary) {
				ctx.strokeStyle = linkColor;
				ctx.lineWidth = (1 * thickness) / currentTransform.k;
				ctx.globalAlpha = highlightId ? linkDimAlpha : linkAlpha * 0.7;
			} else {
				ctx.strokeStyle = linkColor;
				ctx.lineWidth = (0.6 * thickness) / currentTransform.k;
				ctx.globalAlpha = highlightId ? linkDimAlpha : linkAlpha;
			}
			ctx.stroke();
			ctx.setLineDash([]);

			// Arrows
			if (showArrows && !isHL) {
				const dx = tgt.x - src.x;
				const dy = tgt.y - src.y;
				const len = Math.sqrt(dx * dx + dy * dy);
				if (len > 0) {
					const tgtR = getNodeRadius(tgt);
					const arrowLen = 6 / currentTransform.k;
					const endX = tgt.x - (dx / len) * tgtR;
					const endY = tgt.y - (dy / len) * tgtR;
					const angle = Math.atan2(dy, dx);
					ctx.beginPath();
					ctx.moveTo(endX, endY);
					ctx.lineTo(endX - arrowLen * Math.cos(angle - 0.4), endY - arrowLen * Math.sin(angle - 0.4));
					ctx.lineTo(endX - arrowLen * Math.cos(angle + 0.4), endY - arrowLen * Math.sin(angle + 0.4));
					ctx.closePath();
					ctx.fillStyle = ctx.strokeStyle;
					ctx.globalAlpha = 0.6;
					ctx.fill();
				}
			}
		}

		// Nodes
		for (const n of nodeData) {
			if (n.x == null) continue;
			const r = getNodeRadius(n);
			const color = getNodeColor(n);
			const isActive = n.id === activeNodeId;
			const isHovered = n.id === hoveredNode?.id;
			const isConnected = connectedIds?.has(n.id);
			const isFocused = isActive || isHovered || n.id === highlightId;
			const isDimmed = highlightId && !isFocused && !isConnected;

			ctx.globalAlpha = isDimmed ? 0.07 : 1;

			ctx.beginPath();
			ctx.arc(n.x, n.y, r, 0, Math.PI * 2);
			ctx.fillStyle = color;
			ctx.fill();

			if (isFocused) {
				ctx.strokeStyle = ringColor;
				ctx.lineWidth = 2.5 / currentTransform.k;
				ctx.globalAlpha = 1;
				ctx.stroke();
			} else if (isConnected) {
				ctx.strokeStyle = ringConnectedColor;
				ctx.lineWidth = 1.2 / currentTransform.k;
				ctx.globalAlpha = 0.7;
				ctx.stroke();
			}
		}

		// Labels — show based on text fade threshold
		const showLabels = currentTransform.k >= textFadeThreshold;
		if (showLabels || highlightId) {
			const fontSize = Math.max(9, 12 / currentTransform.k);
			ctx.font = `600 ${fontSize}px system-ui, -apple-system, sans-serif`;
			ctx.textAlign = 'center';
			ctx.textBaseline = 'top';

			for (const n of nodeData) {
				if (n.x == null) continue;
				const isFocused = n.id === highlightId;
				const isConnected = connectedIds?.has(n.id);

				// When zoomed in enough, show all labels; otherwise only focused/connected
				if (!showLabels && !isFocused && !isConnected) continue;
				if (showLabels && highlightId && !isFocused && !isConnected) {
					// Dim non-related labels when something is highlighted
					continue;
				}

				const r = getNodeRadius(n);
				const yOff = n.y + r + 5 / currentTransform.k;
				const shadowOff = 1 / currentTransform.k;

				ctx.globalAlpha = isFocused ? 0.7 : (showLabels ? 0.4 : 0.5);
				ctx.fillStyle = labelShadow;
				ctx.fillText(n.name, n.x + shadowOff, yOff + shadowOff);

				ctx.globalAlpha = isFocused ? 1 : (showLabels ? 0.7 : 0.8);
				ctx.fillStyle = labelColor;
				ctx.fillText(n.name, n.x, yOff);
			}
		}

		ctx.restore();

		// Legend
		if (libraryList.length > 1) {
			ctx.save();
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

			const x = 14, y = 14, lineH = 22, dotR = 6, padX = 12, padY = 8, fs = 11;
			ctx.font = `500 ${fs}px system-ui, -apple-system, sans-serif`;

			let maxW = 0;
			for (const v of libraryList) {
				const label = `${v.name} (${v.count})`;
				const w = ctx.measureText(label).width;
				if (w > maxW) maxW = w;
			}
			const boxW = dotR * 2 + 12 + maxW + padX * 2;
			const boxH = libraryList.length * lineH + padY * 2;

			ctx.globalAlpha = 0.92;
			ctx.fillStyle = legendBg;
			ctx.strokeStyle = legendBorder;
			ctx.lineWidth = 1;
			roundRect(ctx, x, y, boxW, boxH, 8);
			ctx.fill();
			ctx.stroke();

			ctx.globalAlpha = 1;
			for (let i = 0; i < libraryList.length; i++) {
				const v = libraryList[i];
				const isHidden = hiddenLibraries.has(v.name);
				const iy = y + padY + i * lineH + lineH / 2;

				ctx.globalAlpha = isHidden ? 0.3 : 1;
				ctx.beginPath();
				ctx.arc(x + padX + dotR, iy, dotR, 0, Math.PI * 2);
				ctx.fillStyle = v.color;
				ctx.fill();

				ctx.fillStyle = legendText;
				ctx.font = `500 ${fs}px system-ui, -apple-system, sans-serif`;
				ctx.textAlign = 'left';
				ctx.textBaseline = 'middle';
				ctx.fillText(`${v.name} (${v.count})`, x + padX + dotR * 2 + 12, iy);
			}

			legendBounds = { x, y, w: boxW, h: boxH, lineH, padY };

			ctx.restore();
		}
	}

	function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
		ctx.beginPath();
		ctx.moveTo(x + r, y);
		ctx.lineTo(x + w - r, y);
		ctx.quadraticCurveTo(x + w, y, x + w, y + r);
		ctx.lineTo(x + w, y + h - r);
		ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
		ctx.lineTo(x + r, y + h);
		ctx.quadraticCurveTo(x, y + h, x, y + h - r);
		ctx.lineTo(x, y + r);
		ctx.quadraticCurveTo(x, y, x + r, y);
		ctx.closePath();
	}

	// ─── Full cleanup on destroy ───

	onDestroy(() => {
		cleanup();
		themeObserver?.disconnect();
		themeObserver = null;
		resizeObserver?.disconnect();
		resizeObserver = null;
		ctx = null;
		nodeData = [];
		linkData = [];
		if (canvasEl) {
			canvasEl.onmousedown = null;
			canvasEl.onmousemove = null;
			canvasEl.onmouseup = null;
			canvasEl.onmouseleave = null;
			canvasEl.onclick = null;
		}
	});
</script>

<div class="graph-container" bind:this={containerEl}>
	{#if nodes.length === 0}
		<div class="graph-empty">{$t('skyView.noNotes')}</div>
	{:else}
		<!-- Controls toggle button -->
		{#if !compact}
		<button
			class="controls-toggle"
			class:active={showControls}
			onclick={() => showControls = !showControls}
			title={$t('skyView.controls.title') ?? 'Sky View Settings'}
		>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="3"/>
				<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
			</svg>
		</button>

		<!-- Controls Panel -->
		{#if showControls}
			<div class="controls-panel">
				<div class="controls-header">
					<button class="controls-reset" onclick={resetControls} title={$t('skyView.controls.reset') ?? 'Reset'}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<polyline points="1 4 1 10 7 10"/>
							<path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
						</svg>
					</button>
					<button class="controls-close" onclick={() => showControls = false}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
						</svg>
					</button>
				</div>

				<div class="controls-body">
					<!-- Filters -->
					<details bind:open={filtersOpen}>
						<summary>{$t('skyView.controls.filters') ?? 'Filters'}</summary>
						<div class="control-section">
							<div class="search-input">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
								</svg>
								<input
									type="text"
									placeholder={$t('skyView.controls.searchPlaceholder') ?? 'Search files...'}
									bind:value={filterQuery}
								/>
							</div>
							<label class="toggle-row">
								<span>{$t('skyView.controls.existingOnly') ?? 'Existing files only'}</span>
								<input type="checkbox" bind:checked={existingOnly} class="toggle" />
							</label>
							<label class="toggle-row">
								<span>{$t('skyView.controls.orphans') ?? 'Orphans'}</span>
								<input type="checkbox" bind:checked={showOrphans} class="toggle" />
							</label>
						</div>
					</details>

					<!-- Groups -->
					<details bind:open={groupsOpen}>
						<summary>{$t('skyView.controls.groups') ?? 'Groups'}</summary>
						<div class="control-section">
							{#each groups as group, i}
								<div class="group-row">
									<input
										type="color"
										value={group.color}
										oninput={(e) => updateGroupColor(i, (e.target as HTMLInputElement).value)}
										class="color-picker"
									/>
									<input
										type="text"
										placeholder="path:folder or tag:#name"
										value={group.query}
										oninput={(e) => updateGroupQuery(i, (e.target as HTMLInputElement).value)}
										class="group-query"
									/>
									<button class="group-remove" onclick={() => removeGroup(i)}>
										<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
											<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
										</svg>
									</button>
								</div>
							{/each}
							<button class="new-group-btn" onclick={addGroup}>
								{$t('skyView.controls.newGroup') ?? 'New group'}
							</button>
						</div>
					</details>

					<!-- Display -->
					<details bind:open={displayOpen}>
						<summary>{$t('skyView.controls.display') ?? 'Display'}</summary>
						<div class="control-section">
							<label class="toggle-row">
								<span>{$t('skyView.controls.arrows') ?? 'Arrows'}</span>
								<input type="checkbox" bind:checked={showArrows} class="toggle" />
							</label>
							<div class="slider-row">
								<span>{$t('skyView.controls.textFade') ?? 'Text fade threshold'}</span>
								<input type="range" min="0" max="5" step="0.1" bind:value={textFadeThreshold} />
							</div>
							<div class="slider-row">
								<span>{$t('skyView.controls.nodeSize') ?? 'Node size'}</span>
								<input type="range" min="1" max="10" step="0.5" bind:value={nodeSizeMultiplier} />
							</div>
							<div class="slider-row">
								<span>{$t('skyView.controls.linkThickness') ?? 'Link thickness'}</span>
								<input type="range" min="0.5" max="5" step="0.25" bind:value={linkThicknessMultiplier} />
							</div>
							<button
								class="animate-btn"
								class:active={animating}
								onclick={() => animating = !animating}
							>
								{$t('skyView.controls.animate') ?? 'Animate'}
							</button>
						</div>
					</details>

					<!-- Forces -->
					<details bind:open={forcesOpen}>
						<summary>{$t('skyView.controls.forces') ?? 'Forces'}</summary>
						<div class="control-section">
							<div class="slider-row">
								<span>{$t('skyView.controls.centerForce') ?? 'Center force'}</span>
								<input type="range" min="0" max="1" step="0.05" bind:value={centerForce} />
							</div>
							<div class="slider-row">
								<span>{$t('skyView.controls.repelForce') ?? 'Repel force'}</span>
								<input type="range" min="0" max="300" step="5" bind:value={repelForce} />
							</div>
							<div class="slider-row">
								<span>{$t('skyView.controls.linkForce') ?? 'Link force'}</span>
								<input type="range" min="0" max="1" step="0.05" bind:value={linkForce} />
							</div>
							<div class="slider-row">
								<span>{$t('skyView.controls.linkDistance') ?? 'Link distance'}</span>
								<input type="range" min="10" max="500" step="10" bind:value={linkDistance} />
							</div>
						</div>
					</details>
				</div>
			</div>
		{/if}
		{/if}

		<canvas bind:this={canvasEl}></canvas>
		{#if tooltipVisible}
			<div class="graph-tooltip" style="left: {tooltipX}px; top: {tooltipY}px;" dir="auto">
				<div class="graph-tooltip-name">{tooltipText}</div>
				{#if tooltipHeadline}
					<div class="graph-tooltip-headline" title={tooltipHeadline}>{tooltipHeadline}</div>
				{/if}
			</div>
		{/if}
		{#if !compact && nodeCountInfo}
			<div class="node-count-badge">{nodeCountInfo} nodes</div>
		{/if}
	{/if}
</div>

<style>
	.star-container {
		width: 100%; height: 100%;
		background: var(--background-secondary);
		position: relative;
		overflow: hidden;
	}
	.node-count-badge {
		position: absolute;
		bottom: 8px;
		right: 8px;
		padding: 2px 8px;
		font-size: 11px;
		color: var(--text-muted);
		background: var(--background-primary);
		border-radius: 4px;
		opacity: 0.8;
		pointer-events: none;
	}
	canvas {
		display: block;
		width: 100%;
		height: 100%;
	}
	.star-empty {
		position: absolute; inset: 0;
		display: flex; align-items: center; justify-content: center;
		color: var(--text-faint); font-size: 0.85rem;
	}
	/* Hover tooltip — was a dead `.star-tooltip` selector (no matching
	   class in the template); renamed in MIG-044 Phase 2 to match the
	   actual `.graph-tooltip` element + extended for the NSC headline
	   subline. The container is now two stacked lines; both ellipsis at
	   max-width so a long headline doesn't blow the canvas. */
	.graph-tooltip {
		position: absolute;
		pointer-events: none;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 4px 10px;
		font-size: 0.8rem;
		color: var(--text-normal);
		box-shadow: var(--shadow-s);
		max-width: 320px;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.graph-tooltip-name {
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.graph-tooltip-headline {
		font-size: 0.7rem;
		font-style: italic;
		color: var(--text-faint);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* ─── Controls Toggle Button ─── */
	.controls-toggle {
		position: absolute;
		top: 10px;
		right: 10px;
		z-index: 20;
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s;
	}
	.controls-toggle:hover, .controls-toggle.active {
		color: var(--text-normal);
		background: var(--background-modifier-hover);
	}

	/* ─── Controls Panel ─── */
	.controls-panel {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 272px;
		z-index: 15;
		background: var(--background-primary);
		border-right: 1px solid var(--background-modifier-border);
		display: flex;
		flex-direction: column;
		overflow: hidden;
		box-shadow: var(--shadow-s);
	}

	.controls-header {
		display: flex;
		justify-content: flex-end;
		gap: 4px;
		padding: 8px 10px 4px;
	}

	.controls-reset, .controls-close {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		border-radius: 4px;
		color: var(--text-muted);
		cursor: pointer;
	}
	.controls-reset:hover, .controls-close:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}

	.controls-body {
		flex: 1;
		overflow-y: auto;
		padding: 0 0 12px;
	}

	/* Sections */
	details {
		border-bottom: 1px solid var(--background-modifier-border);
	}
	summary {
		padding: 10px 14px;
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--text-normal);
		cursor: pointer;
		user-select: none;
		list-style: none;
		display: flex;
		align-items: center;
		gap: 6px;
	}
	summary::before {
		content: '▸';
		font-size: 0.7rem;
		transition: transform 0.15s;
	}
	details[open] > summary::before {
		transform: rotate(90deg);
	}
	summary:hover {
		background: var(--background-modifier-hover);
	}

	.control-section {
		padding: 4px 14px 12px;
	}

	/* Search input */
	.search-input {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		margin-bottom: 8px;
		color: var(--text-muted);
	}
	.search-input input {
		flex: 1;
		background: none;
		border: none;
		outline: none;
		font-size: 0.78rem;
		color: var(--text-normal);
	}
	.search-input input::placeholder {
		color: var(--text-faint);
	}

	/* Toggle rows */
	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 5px 0;
		font-size: 0.78rem;
		color: var(--text-normal);
		cursor: pointer;
	}
	.toggle {
		appearance: none;
		width: 36px;
		height: 20px;
		background: var(--background-modifier-border);
		border-radius: 10px;
		position: relative;
		cursor: pointer;
		transition: background 0.2s;
		flex-shrink: 0;
	}
	.toggle::after {
		content: '';
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
	}
	.toggle:checked {
		background: var(--interactive-accent);
	}
	.toggle:checked::after {
		transform: translateX(16px);
	}

	/* Slider rows */
	.slider-row {
		padding: 4px 0;
	}
	.slider-row span {
		display: block;
		font-size: 0.78rem;
		color: var(--text-normal);
		margin-bottom: 4px;
	}
	.slider-row input[type="range"] {
		width: 100%;
		height: 4px;
		appearance: none;
		background: var(--background-modifier-border);
		border-radius: 2px;
		outline: none;
		cursor: pointer;
	}
	.slider-row input[type="range"]::-webkit-slider-thumb {
		appearance: none;
		width: 16px;
		height: 16px;
		background: white;
		border: 2px solid var(--background-modifier-border);
		border-radius: 50%;
		cursor: pointer;
	}
	.slider-row input[type="range"]::-webkit-slider-thumb:hover {
		border-color: var(--interactive-accent);
	}

	/* Groups */
	.group-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 6px;
	}
	.color-picker {
		width: 24px;
		height: 24px;
		padding: 0;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		cursor: pointer;
		background: none;
		flex-shrink: 0;
	}
	.color-picker::-webkit-color-swatch-wrapper {
		padding: 2px;
	}
	.color-picker::-webkit-color-swatch {
		border: none;
		border-radius: 2px;
	}
	.group-query {
		flex: 1;
		padding: 4px 8px;
		font-size: 0.75rem;
		background: var(--background-secondary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		color: var(--text-normal);
		outline: none;
		min-width: 0;
	}
	.group-query:focus {
		border-color: var(--interactive-accent);
	}
	.group-remove {
		width: 22px;
		height: 22px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		border-radius: 4px;
		color: var(--text-muted);
		cursor: pointer;
		flex-shrink: 0;
	}
	.group-remove:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.new-group-btn {
		width: 100%;
		padding: 6px;
		font-size: 0.78rem;
		font-weight: 500;
		background: var(--interactive-accent);
		color: white;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		transition: opacity 0.15s;
	}
	.new-group-btn:hover {
		opacity: 0.9;
	}

	/* Animate button */
	.animate-btn {
		width: 100%;
		padding: 6px;
		font-size: 0.78rem;
		font-weight: 500;
		background: var(--background-secondary);
		color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		cursor: pointer;
		margin-top: 4px;
		transition: all 0.15s;
	}
	.animate-btn.active {
		background: var(--interactive-accent);
		color: white;
		border-color: var(--interactive-accent);
	}
	.animate-btn:hover {
		opacity: 0.9;
	}
</style>

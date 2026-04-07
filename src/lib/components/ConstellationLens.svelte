<script lang="ts">
	/**
	 * ConstellationLens — Standalone network analysis visualization.
	 * D3.js force simulation + HTML5 Canvas rendering.
	 * Visually distinct from Sky View (GraphMind/Pixi.js).
	 */
	import { onMount, onDestroy } from 'svelte';
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
	let quadtree: d3.Quadtree<SimNode> | null = null;

	// ─── Community hulls cache ───
	let communityHulls: Map<number, { points: [number, number][]; cx: number; cy: number; color: string }> = new Map();

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
			const pts: [number, number][] = members.map(n => [n.x ?? 0, n.y ?? 0]);
			const hull = d3.polygonHull(pts);
			if (!hull) continue;
			const cx = d3.mean(pts, p => p[0]) ?? 0;
			const cy = d3.mean(pts, p => p[1]) ?? 0;
			// Expand hull outward from centroid by padding
			const expanded = hull.map(([hx, hy]): [number, number] => {
				const dx = hx - cx, dy = hy - cy;
				const dist = Math.sqrt(dx * dx + dy * dy);
				const pad = 25;
				return [hx + (dx / dist) * pad, hy + (dy / dist) * pad];
			});
			communityHulls.set(cid, { points: expanded, cx, cy, color: communityColors.get(cid) ?? '#94a3b8' });
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
		for (const [, hull] of communityHulls) {
			if (hull.points.length < 3) continue;
			ctx.beginPath();
			ctx.moveTo(hull.points[0][0], hull.points[0][1]);
			for (let i = 1; i < hull.points.length; i++) {
				ctx.lineTo(hull.points[i][0], hull.points[i][1]);
			}
			ctx.closePath();
			ctx.fillStyle = hull.color + '22'; // ~13% alpha
			ctx.fill();
			ctx.strokeStyle = hull.color + '88'; // ~53% alpha
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

		// 3. Edges
		ctx.lineWidth = 0.5 / zoom;
		for (const link of simLinks) {
			const src = link.source as SimNode;
			const tgt = link.target as SimNode;
			const color = LINK_TYPE_COLORS[link.linkType ?? ''] ?? '#cbd5e1';
			ctx.beginPath();
			ctx.moveTo(src.x ?? 0, src.y ?? 0);
			ctx.lineTo(tgt.x ?? 0, tgt.y ?? 0);
			ctx.strokeStyle = color + '33'; // ~20% alpha
			ctx.stroke();
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

		// 5. Hover label
		if (hoveredNode) {
			const x = hoveredNode.x ?? 0, y = hoveredNode.y ?? 0;
			const label = hoveredNode.name.replace(/\.md$/, '');
			const fontSize = Math.max(10, 12 / zoom);
			ctx.font = `600 ${fontSize}px system-ui, sans-serif`;
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

	onDestroy(() => {
		destroyed = true;
		if (animFrame) cancelAnimationFrame(animFrame);
		simulation?.stop();
		resizeObs?.disconnect();
	});
</script>

<div class="cl-root" dir={$dir}>
	<div class="cl-header">
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/><path d="M11 8v6"/><path d="M8 11h6"/></svg>
		<span class="cl-title">{$t('lens.title') || 'Constellation Lens'}</span>
		<span class="cl-stat">{simNodes.length} {$t('lens.nodes') || 'nodes'} · {simLinks.length} {$t('lens.edges') || 'edges'}</span>
		<button class="cl-close" onclick={() => onClose?.()}>×</button>
	</div>
	<div class="cl-body">
		<div class="cl-canvas-wrap">
			<canvas bind:this={canvasEl}
				onmousedown={onMouseDown} onmousemove={onMouseMove}
				onmouseup={onMouseUp} onclick={onClick}
				onwheel={onWheel}></canvas>
			<!-- Legend — lower-left, always visible -->
			<div class="cl-legend">
				<div class="cl-legend-title">{$t('lens.legend') || 'Legend'}</div>
				<div class="cl-legend-row">
					<span class="cl-lg-circle cl-lg-big"></span>
					<span><strong>Large node</strong> — bridge</span>
				</div>
				<div class="cl-legend-row">
					<span class="cl-lg-circle cl-lg-small"></span>
					<span><strong>Small node</strong> — peripheral</span>
				</div>
				<div class="cl-legend-row">
					<span class="cl-lg-circle" style="background:#a78bfa"></span>
					<span class="cl-lg-circle" style="background:#34d399"></span>
					<span class="cl-lg-circle" style="background:#60a5fa"></span>
					<span><strong>Color</strong> — community</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#94a3b8" stroke-width="1.5"/></svg>
					<span><strong>Line</strong> — wikilink</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="4"><line x1="0" y1="2" x2="20" y2="2" stroke="#ef4444" stroke-width="2" stroke-dasharray="4,3"/></svg>
					<span><strong>Red dashed</strong> — blind spot</span>
				</div>
				<div class="cl-legend-row">
					<svg width="20" height="12"><polygon points="2,6 6,1 14,1 18,6 14,11 6,11" fill="rgba(124,58,237,0.15)" stroke="#7c3aed" stroke-width="1.5"/></svg>
					<span><strong>Region</strong> — community</span>
				</div>
			</div>
		</div>
		<div class="cl-panel-wrap">
			<LensPanel
				{health} {bridges} {communities} {communityProfiles} {contradictions} {gaps}
				nodeCount={simNodes.length} edgeCount={simLinks.length}
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
	.cl-stat { font-size: 12px; color: var(--text-muted, #64748b); flex: 1; }
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
</style>

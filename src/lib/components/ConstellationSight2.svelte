<script lang="ts">
	/**
	 * ConstellationSight2 — Ground-up rebuild with Living Link System.
	 *
	 * The stethoscope for your knowledge system: shows the circulatory health
	 * of your thinking — where blood flows strongly, where vessels narrow,
	 * where the heart beats fast, and where tissue is dying.
	 *
	 * Architecture:
	 *   D3 force simulation (proven) + Canvas rendering
	 *   Living Link enrichment (type colors, weight thickness, confidence styles)
	 *   Search (6 scopes, chips, history, category badges)
	 *   Insight Panel (health, links, lifecycle, formulation, communities)
	 */
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import * as d3 from 'd3';
	import { t, dir, getSearchOps } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { readSearchHistory, addSearchHistory } from '$lib/libraries/searchHistory';
	import {
		stripInvisibleChars, canonicalizeSearchQuery, hasAdvancedSyntaxMultilingual,
		universalSearch, type UniversalSearchResponse,
	} from '$lib/libraries/store';
	import type { SkyNode, SkyLink } from '$lib/libraries/store';
	import type { ClusterInfo, StructuralGap, UniverseHealth, CommunityProfile } from '$lib/graph/clusterEngine';

	// ─── Types ────────────────────────────────────────────────
	interface SimNode extends d3.SimulationNodeDatum {
		id: string;
		name: string;
		path: string;
		libraryName: string;
		centrality: number;
		communityId: number;
		communityColor: string;
		r: number;
		maturity?: string;
		linkCount?: number;
		stratum?: number;
	}

	interface SimLink extends d3.SimulationLinkDatum<SimNode> {
		linkType?: string;
		weight?: number;
		confidence?: string;
		annotation?: string;
		traversalCount?: number;
		status?: string;
	}

	// ─── Link Type Colors (Living Link System) ────────────────
	const LINK_TYPE_COLORS: Record<string, string> = {
		supports: '#4A9EFF', contradicts: '#FF4A4A', causes: '#FF8C42',
		exemplifies: '#4AFF88', generalizes: '#C084FC', 'derives-from': '#FACC15',
		'part-of': '#94A3B8', associative: '#A78BFA', relates: '#94a3b8',
	};

	const MATURITY_COLORS: Record<string, string> = {
		seed: '#d1d5db', sapling: '#86efac', evergreen: '#16a34a',
		canonical: '#f59e0b', wilting: '#ef4444',
	};

	const CONFIDENCE_STYLE: Record<string, { dash: number[]; widthMul: number }> = {
		hypothesis: { dash: [4, 3], widthMul: 0.7 },
		evidence: { dash: [], widthMul: 1.0 },
		established: { dash: [], widthMul: 1.5 },
		contested: { dash: [2, 2], widthMul: 1.2 },
	};

	// ─── Props ────────────────────────────────────────────────
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
		communityProfiles = [] as CommunityProfile[],
		contradictions = [] as [string, string][],
		libraryColorMap = {} as Record<string, string>,
		searchMatchIds = null as Set<string> | null,
		onNoteClick,
		onClose,
	}: {
		nodes: SkyNode[];
		links: SkyLink[];
		centrality?: Map<string, number>;
		communityAssignments?: Map<string, number>;
		communityColors?: Map<number, string>;
		gaps?: StructuralGap[];
		health?: UniverseHealth | null;
		bridges?: { id: string; name: string; centrality: number }[];
		communities?: ClusterInfo[];
		communityProfiles?: CommunityProfile[];
		contradictions?: [string, string][];
		libraryColorMap?: Record<string, string>;
		searchMatchIds?: Set<string> | null;
		onNoteClick?: (path: string, name: string, highlightTerm?: string) => void;
		onClose?: () => void;
	} = $props();

	// ─── State ────────────────────────────────────────────────
	let canvasEl: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let width = $state(0);
	let height = $state(0);
	let destroyed = false;
	let animFrame = 0;

	// Simulation
	let simNodes: SimNode[] = [];
	let simLinks: SimLink[] = [];
	let simulation: d3.Simulation<SimNode, SimLink> | null = null;

	// Interaction
	let panX = 0, panY = 0, zoom = 1;
	let isPanning = false;
	let panStartX = 0, panStartY = 0;
	let hoveredNode: SimNode | null = null;
	let hoveredLink: SimLink | null = null;

	// Living Link enrichment map: "source::target" → link data
	let linkEnrichment = new Map<string, { weight: number; confidence: string; annotation: string; traversalCount: number; status: string }>();

	// Settings
	let showRegions = $state(true);
	let forceStrength = $state(-60);
	let linkDistance = $state(40);

	const isRTL = $derived($dir === 'rtl');

	// ─── Build Simulation Data ────────────────────────────────
	function buildSimData() {
		const nodeMap = new Map<string, SimNode>();
		simNodes = nodes.map(n => {
			const c = centrality.get(n.id) ?? 0;
			const cid = communityAssignments.get(n.id) ?? 0;
			const color = communityColors.get(cid) ?? '#94a3b8';
			const sn: SimNode = {
				id: n.id,
				name: n.name,
				path: n.path,
				libraryName: n.libraryName,
				centrality: c,
				communityId: cid,
				communityColor: color,
				r: Math.max(3, 3 + c * 18),
				maturity: (n as any).maturity,
				linkCount: n.linkCount,
				stratum: (n as any).stratum,
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
				const key = `${src.name.toLowerCase()}::${tgt.name.toLowerCase()}`;
				const enriched = linkEnrichment.get(key);
				simLinks.push({
					source: src,
					target: tgt,
					linkType: l.linkType || enriched?.status === 'active' ? (l.linkType || 'relates') : undefined,
					weight: enriched?.weight ?? 1.0,
					confidence: enriched?.confidence ?? 'hypothesis',
					annotation: enriched?.annotation ?? '',
					traversalCount: enriched?.traversalCount ?? 0,
					status: enriched?.status ?? 'active',
				});
			}
		}
	}

	// ─── Load Living Link Enrichment from note_links ──────────
	async function loadLinkEnrichment() {
		try {
			// Query all active links with their properties
			const stats: any = await invoke('constellation_link_stats');
			if (stats?.sample_links) {
				for (const link of stats.sample_links) {
					const key = `${link.source?.toLowerCase()}::${link.target?.toLowerCase()}`;
					linkEnrichment.set(key, {
						weight: link.weight ?? 1.0,
						confidence: link.confidence ?? 'hypothesis',
						annotation: link.annotation ?? '',
						traversalCount: link.traversal_count ?? 0,
						status: 'active',
					});
				}
			}
		} catch {}
	}

	// ─── Start D3 Force Simulation ────────────────────────────
	function startSimulation() {
		simulation = d3.forceSimulation(simNodes)
			.force('link', d3.forceLink(simLinks).id((d: any) => d.id).distance(linkDistance).strength(0.3))
			.force('charge', d3.forceManyBody().strength(forceStrength).distanceMax(300))
			.force('center', d3.forceCenter(0, 0))
			.force('collide', d3.forceCollide<SimNode>().radius(d => d.r + 2))
			.alphaDecay(0.05); // fast decay for performance

		// Synchronous ticks for initial layout — 40 ticks (was 200 in old)
		// D3 Barnes-Hut is O(n log n) per tick; 40 gives adequate layout
		for (let i = 0; i < 40; i++) simulation.tick();

		// Then animate
		simulation.alpha(0.1).restart();
		simulation.on('tick', () => {
			if (!destroyed) requestDraw();
		});
	}

	// ─── Canvas Rendering ─────────────────────────────────────
	function requestDraw() {
		if (animFrame) return;
		animFrame = requestAnimationFrame(() => {
			animFrame = 0;
			draw();
		});
	}

	function draw() {
		if (!ctx || destroyed) return;
		const w = width, h = height;
		ctx.clearRect(0, 0, w, h);

		ctx.save();
		// Transform: pan + zoom centered on canvas
		ctx.translate(w / 2 + panX, h / 2 + panY);
		ctx.scale(zoom, zoom);

		// 1. Background grid (subtle)
		drawGrid(w, h);

		// 2. Community regions (ellipses) — very subtle, background only
		if (showRegions) drawCommunityRegions();

		// 3. Nodes (drawn before edges so edges are on TOP and visible)
		drawNodes();

		// 4. Edges — LIVING LINK VISUALIZATION (on top of everything)
		drawEdges();

		// 5. Structural gap lines (red dashed — on top)
		drawGapLines();

		// 6. Hover label
		if (hoveredNode) drawHoverLabel(hoveredNode);

		// 7. Hover link annotation
		if (hoveredLink) drawLinkAnnotation(hoveredLink);

		ctx.restore();
	}

	function drawGrid(w: number, h: number) {
		const gridSize = 40;
		const gridColor = 'rgba(148, 163, 184, 0.06)';
		ctx!.strokeStyle = gridColor;
		ctx!.lineWidth = 0.5 / zoom;
		const hw = w / 2 / zoom, hh = h / 2 / zoom;
		for (let x = -Math.ceil(hw / gridSize) * gridSize; x <= hw; x += gridSize) {
			ctx!.beginPath(); ctx!.moveTo(x, -hh); ctx!.lineTo(x, hh); ctx!.stroke();
		}
		for (let y = -Math.ceil(hh / gridSize) * gridSize; y <= hh; y += gridSize) {
			ctx!.beginPath(); ctx!.moveTo(-hw, y); ctx!.lineTo(hw, y); ctx!.stroke();
		}
	}

	function drawCommunityRegions() {
		// Compute ellipse hulls per community
		const communityNodes = new Map<number, SimNode[]>();
		for (const n of simNodes) {
			const nodes = communityNodes.get(n.communityId) || [];
			nodes.push(n);
			communityNodes.set(n.communityId, nodes);
		}

		for (const [cid, members] of communityNodes) {
			if (members.length < 3) continue;
			const cx = d3.mean(members, m => m.x) ?? 0;
			const cy = d3.mean(members, m => m.y) ?? 0;
			let rx = 0, ry = 0;
			for (const m of members) {
				rx = Math.max(rx, Math.abs((m.x ?? 0) - cx));
				ry = Math.max(ry, Math.abs((m.y ?? 0) - cy));
			}
			rx = Math.max(rx + 30, 40);
			ry = Math.max(ry + 30, 40);
			const color = communityColors.get(cid) ?? '#94a3b8';

			ctx!.beginPath();
			ctx!.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
			ctx!.fillStyle = color + '06'; // 2% fill — won't bury edges
			ctx!.fill();
			ctx!.strokeStyle = color + '22'; // 13% stroke — subtle outline
			ctx!.lineWidth = 1.5 / zoom;
			ctx!.stroke();
		}
	}

	function drawGapLines() {
		if (!gaps.length) return;
		const communityCenter = new Map<number, { x: number; y: number }>();
		for (const n of simNodes) {
			const cid = n.communityId;
			if (!communityCenter.has(cid)) communityCenter.set(cid, { x: 0, y: 0 });
			const c = communityCenter.get(cid)!;
			c.x += (n.x ?? 0); c.y += (n.y ?? 0);
		}
		const counts = new Map<number, number>();
		for (const n of simNodes) counts.set(n.communityId, (counts.get(n.communityId) ?? 0) + 1);
		for (const [cid, c] of communityCenter) {
			const cnt = counts.get(cid) ?? 1;
			c.x /= cnt; c.y /= cnt;
		}

		for (const gap of gaps) {
			const c1 = communityCenter.get(gap.community1);
			const c2 = communityCenter.get(gap.community2);
			if (!c1 || !c2) continue;
			ctx!.beginPath();
			ctx!.setLineDash([8 / zoom, 6 / zoom]);
			ctx!.moveTo(c1.x, c1.y);
			ctx!.lineTo(c2.x, c2.y);
			ctx!.strokeStyle = '#ef444499';
			ctx!.lineWidth = 2 / zoom;
			ctx!.stroke();
			ctx!.setLineDash([]);
		}
	}

	function drawEdges() {
		// Viewport bounds for culling (skip edges fully outside view)
		const hw = width / 2 / zoom, hh = height / 2 / zoom;
		const vpLeft = -panX / zoom - hw - 50, vpRight = -panX / zoom + hw + 50;
		const vpTop = -panY / zoom - hh - 50, vpBottom = -panY / zoom + hh + 50;

		for (const link of simLinks) {
			const src = link.source as SimNode;
			const tgt = link.target as SimNode;
			const sx = src.x ?? 0, sy = src.y ?? 0;
			const tx = tgt.x ?? 0, ty = tgt.y ?? 0;

			// Cull edges fully outside viewport
			if ((sx < vpLeft && tx < vpLeft) || (sx > vpRight && tx > vpRight) ||
				(sy < vpTop && ty < vpTop) || (sy > vpBottom && ty > vpBottom)) continue;

			const typed = link.linkType && link.linkType !== 'relates' && LINK_TYPE_COLORS[link.linkType];
			const color = typed ? LINK_TYPE_COLORS[link.linkType!] : '#94a3b8';

			// Weight → thickness
			const w = link.weight ?? 1.0;
			const baseWidth = typed ? Math.max(0.8, Math.min(4, w * 0.5)) : 0.7;

			// Confidence → dash style
			const conf = CONFIDENCE_STYLE[link.confidence ?? 'hypothesis'] ?? CONFIDENCE_STYLE.hypothesis;
			if (conf.dash.length > 0) {
				ctx!.setLineDash(conf.dash.map(d => d / zoom));
			}

			// Dormant → faded, untyped → visible but subtle
			const isDormant = link.status === 'dormant';
			const opacity = isDormant ? '44' : typed ? 'CC' : 'AA';

			ctx!.beginPath();
			ctx!.moveTo(sx, sy);
			ctx!.lineTo(tx, ty);
			ctx!.strokeStyle = color + opacity;
			ctx!.lineWidth = (baseWidth * conf.widthMul) / zoom;
			ctx!.stroke();
			if (conf.dash.length > 0) ctx!.setLineDash([]);

			// Arrowhead for typed links
			if (typed) {
				const dx = tx - sx, dy = ty - sy;
				const len = Math.sqrt(dx * dx + dy * dy);
				if (len > 10) {
					const ux = dx / len, uy = dy / len;
					const ax = sx + dx * 0.7, ay = sy + dy * 0.7;
					const as = 4 / zoom;
					ctx!.beginPath();
					ctx!.moveTo(ax + ux * as, ay + uy * as);
					ctx!.lineTo(ax - uy * as * 0.5 - ux * as * 0.3, ay + ux * as * 0.5 - uy * as * 0.3);
					ctx!.lineTo(ax + uy * as * 0.5 - ux * as * 0.3, ay - ux * as * 0.5 - uy * as * 0.3);
					ctx!.closePath();
					ctx!.fillStyle = color + 'CC';
					ctx!.fill();
				}
			}
		}
	}

	function drawNodes() {
		const hw = width / 2 / zoom, hh = height / 2 / zoom;
		const vpLeft = -panX / zoom - hw - 20, vpRight = -panX / zoom + hw + 20;
		const vpTop = -panY / zoom - hh - 20, vpBottom = -panY / zoom + hh + 20;

		for (const n of simNodes) {
			const x = n.x ?? 0, y = n.y ?? 0;
			// Cull nodes outside viewport
			if (x < vpLeft || x > vpRight || y < vpTop || y > vpBottom) continue;

			// Node circle
			ctx!.beginPath();
			ctx!.arc(x, y, n.r, 0, Math.PI * 2);
			ctx!.fillStyle = n.communityColor;
			ctx!.fill();

			// Maturity border
			if (n.maturity && MATURITY_COLORS[n.maturity]) {
				ctx!.strokeStyle = MATURITY_COLORS[n.maturity];
				ctx!.lineWidth = 1.5 / zoom;
				ctx!.stroke();
			}

			// Bridge ring (high centrality)
			if (n.centrality > 0.5) {
				ctx!.strokeStyle = '#ffffff';
				ctx!.lineWidth = 2 / zoom;
				ctx!.stroke();
			}

			// Hover highlight
			if (hoveredNode === n) {
				ctx!.strokeStyle = '#7c3aed';
				ctx!.lineWidth = 2.5 / zoom;
				ctx!.stroke();
			}
		}
	}

	function drawHoverLabel(n: SimNode) {
		const x = n.x ?? 0, y = n.y ?? 0;
		const label = n.name;
		ctx!.font = `${12 / zoom}px system-ui, sans-serif`;
		const metrics = ctx!.measureText(label);
		const lw = metrics.width + 10 / zoom;
		const lh = 18 / zoom;
		const lx = x - lw / 2;
		const ly = y - n.r - lh - 6 / zoom;

		ctx!.fillStyle = 'rgba(30,30,40,0.9)';
		ctx!.beginPath();
		ctx!.roundRect(lx, ly, lw, lh, 3 / zoom);
		ctx!.fill();
		ctx!.fillStyle = '#ffffff';
		ctx!.textAlign = 'center';
		ctx!.textBaseline = 'middle';
		ctx!.fillText(label, x, ly + lh / 2);
	}

	function drawLinkAnnotation(link: SimLink) {
		if (!link.annotation) return;
		const src = link.source as SimNode;
		const tgt = link.target as SimNode;
		const mx = ((src.x ?? 0) + (tgt.x ?? 0)) / 2;
		const my = ((src.y ?? 0) + (tgt.y ?? 0)) / 2;

		const text = `${link.linkType ?? 'relates'}: ${link.annotation}`;
		ctx!.font = `${10 / zoom}px system-ui, sans-serif`;
		const tw = Math.min(ctx!.measureText(text).width + 10 / zoom, 200 / zoom);
		const th = 16 / zoom;

		ctx!.fillStyle = 'rgba(30,30,40,0.85)';
		ctx!.beginPath();
		ctx!.roundRect(mx - tw / 2, my - th - 4 / zoom, tw, th, 3 / zoom);
		ctx!.fill();
		ctx!.fillStyle = '#ffffff';
		ctx!.textAlign = 'center';
		ctx!.textBaseline = 'middle';
		ctx!.fillText(text.length > 40 ? text.slice(0, 40) + '...' : text, mx, my - th / 2 - 4 / zoom);
	}

	// ─── Interaction ──────────────────────────────────────────
	function onMouseDown(e: MouseEvent) {
		isPanning = true;
		panStartX = e.clientX - panX;
		panStartY = e.clientY - panY;
	}

	function onMouseMove(e: MouseEvent) {
		if (isPanning) {
			panX = e.clientX - panStartX;
			panY = e.clientY - panStartY;
			requestDraw();
			return;
		}
		// Hit test nodes
		const rect = canvasEl.getBoundingClientRect();
		const mx = (e.clientX - rect.left - rect.width / 2 - panX) / zoom;
		const my = (e.clientY - rect.top - rect.height / 2 - panY) / zoom;

		hoveredNode = null;
		hoveredLink = null;
		for (const n of simNodes) {
			const dx = (n.x ?? 0) - mx, dy = (n.y ?? 0) - my;
			if (dx * dx + dy * dy < (n.r + 4) * (n.r + 4)) {
				hoveredNode = n;
				break;
			}
		}

		// Hit test edges (if no node hovered)
		if (!hoveredNode) {
			for (const link of simLinks) {
				if (!link.annotation) continue;
				const src = link.source as SimNode;
				const tgt = link.target as SimNode;
				const sx = src.x ?? 0, sy = src.y ?? 0;
				const tx = tgt.x ?? 0, ty = tgt.y ?? 0;
				// Point-to-line-segment distance
				const dx = tx - sx, dy = ty - sy;
				const len2 = dx * dx + dy * dy;
				if (len2 === 0) continue;
				const t = Math.max(0, Math.min(1, ((mx - sx) * dx + (my - sy) * dy) / len2));
				const px = sx + t * dx, py = sy + t * dy;
				const dist = Math.sqrt((mx - px) * (mx - px) + (my - py) * (my - py));
				if (dist < 6 / zoom) {
					hoveredLink = link;
					break;
				}
			}
		}

		canvasEl.style.cursor = hoveredNode ? 'pointer' : hoveredLink ? 'help' : 'grab';
		requestDraw();
	}

	function onMouseUp() { isPanning = false; }

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		zoom = Math.max(0.1, Math.min(5, zoom + (e.deltaY > 0 ? -0.08 : 0.08)));
		requestDraw();
	}

	function onClick(e: MouseEvent) {
		if (hoveredNode && onNoteClick) {
			onNoteClick(hoveredNode.path, hoveredNode.name);
		}
	}

	function fitToScreen() {
		if (simNodes.length === 0) return;
		let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
		for (const n of simNodes) {
			minX = Math.min(minX, (n.x ?? 0) - n.r);
			maxX = Math.max(maxX, (n.x ?? 0) + n.r);
			minY = Math.min(minY, (n.y ?? 0) - n.r);
			maxY = Math.max(maxY, (n.y ?? 0) + n.r);
		}
		const rangeX = maxX - minX || 1;
		const rangeY = maxY - minY || 1;
		zoom = Math.min(width / rangeX, height / rangeY) * 0.85;
		zoom = Math.max(0.1, Math.min(3, zoom));
		panX = 0; panY = 0;
		requestDraw();
	}

	// ─── Lifecycle ────────────────────────────────────────────
	onMount(async () => {
		const rect = canvasEl.parentElement?.getBoundingClientRect();
		width = rect?.width ?? 800;
		height = rect?.height ?? 600;
		canvasEl.width = width * devicePixelRatio;
		canvasEl.height = height * devicePixelRatio;
		ctx = canvasEl.getContext('2d');
		if (ctx) ctx.scale(devicePixelRatio, devicePixelRatio);

		// Build and run simulation IMMEDIATELY (don't wait for enrichment)
		buildSimData();
		startSimulation();
		fitToScreen();

		// Load Living Link enrichment data in background (non-blocking)
		loadLinkEnrichment().then(() => {
			// Re-enrich simLinks with loaded data
			for (const link of simLinks) {
				const src = link.source as SimNode;
				const tgt = link.target as SimNode;
				const key = `${src.name.toLowerCase()}::${tgt.name.toLowerCase()}`;
				const enriched = linkEnrichment.get(key);
				if (enriched) {
					link.weight = enriched.weight;
					link.confidence = enriched.confidence;
					link.annotation = enriched.annotation;
					link.traversalCount = enriched.traversalCount;
					link.status = enriched.status;
				}
			}
			requestDraw();
		});
	});

	onDestroy(() => {
		destroyed = true;
		if (animFrame) cancelAnimationFrame(animFrame);
		simulation?.stop();
	});
</script>

<div class="sight2-root" dir={isRTL ? 'rtl' : 'ltr'}>
	<!-- Header -->
	<div class="sight2-header">
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>
		<span class="sight2-title">Constellation Sight</span>
		<span class="sight2-stats">{simNodes.length} nodes · {simLinks.length} edges</span>
		<div class="sight2-actions">
			<button class="sight2-btn" onclick={fitToScreen} title="Fit to screen">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M21 8V5a2 2 0 0 0-2-2h-3"/><path d="M3 16v3a2 2 0 0 0 2 2h3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
			</button>
			<button class="sight2-close" onclick={() => onClose?.()}>×</button>
		</div>
	</div>

	<!-- Canvas -->
	<div class="sight2-canvas-wrap">
		<canvas bind:this={canvasEl} style="width:{width}px;height:{height}px"
			onmousedown={onMouseDown} onmousemove={onMouseMove} onmouseup={onMouseUp}
			onmouseleave={onMouseUp} onwheel={onWheel} onclick={onClick}>
		</canvas>
	</div>
</div>

<style>
	.sight2-root {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background: #fafafa;
		overflow: hidden;
	}
	.sight2-header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--background-modifier-border, #e5e7eb);
		flex-shrink: 0;
		background: var(--background-primary, #fff);
	}
	.sight2-title { font-size: 14px; font-weight: 700; }
	.sight2-stats { font-size: 11px; color: var(--text-muted, #64748b); }
	.sight2-actions { display: flex; gap: 4px; margin-inline-start: auto; }
	.sight2-btn {
		border: none; background: none; cursor: pointer;
		color: var(--text-muted, #64748b); padding: 4px; border-radius: 4px;
	}
	.sight2-btn:hover { background: var(--background-modifier-hover, #f1f5f9); color: var(--text-normal, #1a1a1a); }
	.sight2-close {
		border: none; background: none; cursor: pointer;
		font-size: 18px; color: var(--text-muted, #64748b); padding: 0 4px;
	}
	.sight2-close:hover { color: var(--text-normal, #1a1a1a); }
	.sight2-canvas-wrap {
		flex: 1;
		position: relative;
		overflow: hidden;
	}
	.sight2-canvas-wrap canvas {
		display: block;
		cursor: grab;
	}
</style>

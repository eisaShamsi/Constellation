<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import * as d3 from 'd3';

	interface GraphNode extends d3.SimulationNodeDatum {
		id: string;
		name: string;
		path: string;
		vaultName: string;
		group?: string;
		linkCount: number;
	}

	interface GraphLink {
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

	let {
		nodes = [] as GraphNode[],
		links = [] as GraphLink[],
		onNodeClick,
		activeNodeId = '',
	}: {
		nodes: GraphNode[];
		links: GraphLink[];
		onNodeClick: (path: string, vaultName: string) => void;
		activeNodeId?: string;
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

	// Tooltip
	let tooltipText = $state('');
	let tooltipX = $state(0);
	let tooltipY = $state(0);
	let tooltipVisible = $state(false);

	// Legend click bounds
	let legendBounds: { x: number; y: number; w: number; h: number; lineH: number; padY: number } | null = null;

	// Vault colors
	const VAULT_COLORS = [
		'#8b5cf6', '#3b82f6', '#10b981', '#f59e0b',
		'#ef4444', '#ec4899', '#06b6d4', '#84cc16',
	];
	let vaultColorMap = new Map<string, string>();
	let vaultList: { name: string; color: string; count: number }[] = $state([]);
	let hiddenVaults: Set<string> = $state(new Set());

	function assignVaultColors() {
		const vaultCounts = new Map<string, number>();
		for (const n of nodes) {
			vaultCounts.set(n.vaultName, (vaultCounts.get(n.vaultName) || 0) + 1);
		}
		const vaultNames = [...vaultCounts.keys()];
		vaultColorMap = new Map();
		const list: { name: string; color: string; count: number }[] = [];
		vaultNames.forEach((v, i) => {
			const color = VAULT_COLORS[i % VAULT_COLORS.length];
			vaultColorMap.set(v, color);
			list.push({ name: v, color, count: vaultCounts.get(v) || 0 });
		});
		vaultList = list;
	}

	function toggleVaultVisibility(vaultName: string) {
		const next = new Set(hiddenVaults);
		if (next.has(vaultName)) next.delete(vaultName);
		else next.add(vaultName);
		hiddenVaults = next;
		renderGraph();
	}

	function getNodeRadius(d: any): number {
		return Math.max(4, Math.min(14, 3 + d.linkCount * 1.5));
	}

	function getNodeColor(d: any): string {
		return vaultColorMap.get(d.vaultName) || '#6b7280';
	}

	function isDarkTheme(): boolean {
		return document.body.classList.contains('theme-dark');
	}

	function getCSSVar(name: string): string {
		return getComputedStyle(document.body).getPropertyValue(name).trim();
	}

	// ─── Lifecycle ───

	let themeObserver: MutationObserver | null = null;

	onMount(() => {
		mounted = true;
		ctx = canvasEl?.getContext('2d') || null;

		// Theme change observer — just redraws, no heavy work
		themeObserver = new MutationObserver(() => {
			if (nodeData.length > 0) draw();
		});
		themeObserver.observe(document.body, {
			attributes: true,
			attributeFilter: ['class'],
		});

		if (nodes.length > 0 && canvasEl) {
			renderGraph();
		}
	});

	// Re-render only when nodes actually change (not on every reactive tick)
	let prevNodesLen = 0;
	$effect(() => {
		const len = nodes.length;
		if (mounted && len > 0 && len !== prevNodesLen && canvasEl) {
			prevNodesLen = len;
			ctx = canvasEl.getContext('2d');
			renderGraph();
		}
	});

	$effect(() => {
		if (activeNodeId && activeNodeId !== prevActiveNodeId && nodeData.length > 0) {
			prevActiveNodeId = activeNodeId;
			centerOnNode(activeNodeId);
		}
	});

	// ─── Center on node ───

	function centerOnNode(nodeId: string) {
		const target = nodeData.find((n: any) => n.id === nodeId);
		if (!target || target.x == null || target.y == null) return;

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
			const t = Math.min(1, (time - start) / duration);
			const e = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
			currentTransform = d3.zoomIdentity
				.translate(from.x + (to.x - from.x) * e, from.y + (to.y - from.y) * e)
				.scale(from.k + (to.k - from.k) * e);
			draw();
			if (t < 1) requestAnimationFrame(animate);
		}
		requestAnimationFrame(animate);
	}

	// ─── Cleanup helper ───

	function cleanup() {
		if (simulation) {
			simulation.stop();
			simulation.on('tick', null);
			simulation.on('end', null);
			simulation = null;
		}
		// Remove d3 zoom listeners from canvas
		if (canvasEl) {
			d3.select(canvasEl).on('.zoom', null);
		}
		hoveredNode = null;
		draggedNode = null;
	}

	// ─── Main render ───

	function renderGraph() {
		if (!containerEl || !canvasEl || !ctx) return;

		// Full cleanup of previous simulation and listeners
		cleanup();

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		dpr = window.devicePixelRatio || 1;

		canvasEl.width = width * dpr;
		canvasEl.height = height * dpr;
		canvasEl.style.width = width + 'px';
		canvasEl.style.height = height + 'px';

		assignVaultColors();

		// Filter hidden vaults
		const visibleNodes = nodes.filter(n => !hiddenVaults.has(n.vaultName));
		const visibleIds = new Set(visibleNodes.map(n => n.id));
		const visibleLinks = links.filter(l => visibleIds.has(l.source as string) && visibleIds.has(l.target as string));

		nodeData = visibleNodes.map(n => ({ ...n }));
		linkData = visibleLinks.map(l => ({ ...l }));

		// Compute vault cluster centers for clustering force
		const vaultNames = [...new Set(visibleNodes.map(n => n.vaultName))];
		const vaultCenters = new Map<string, { x: number; y: number }>();
		const angle = (2 * Math.PI) / Math.max(vaultNames.length, 1);
		const clusterRadius = Math.min(width, height) * 0.2;
		vaultNames.forEach((v, i) => {
			vaultCenters.set(v, {
				x: width / 2 + Math.cos(angle * i) * clusterRadius,
				y: height / 2 + Math.sin(angle * i) * clusterRadius,
			});
		});

		simulation = d3.forceSimulation(nodeData)
			.force('link', d3.forceLink(linkData).id((d: any) => d.id).distance(60))
			.force('charge', d3.forceManyBody().strength(-80).theta(0.9))
			.force('center', d3.forceCenter(width / 2, height / 2))
			.force('collision', d3.forceCollide().radius(8))
			.force('clusterX', d3.forceX((d: any) => vaultCenters.get(d.vaultName)?.x ?? width / 2).strength(0.05))
			.force('clusterY', d3.forceY((d: any) => vaultCenters.get(d.vaultName)?.y ?? height / 2).strength(0.05))
			.alphaDecay(0.06)
			.velocityDecay(0.45);

		let tickCount = 0;
		simulation.on('tick', () => {
			tickCount++;
			if (tickCount % 2 === 0 || simulation!.alpha() < 0.05) {
				draw();
			}
		});

		simulation.on('end', () => {
			draw();
			if (activeNodeId) {
				prevActiveNodeId = activeNodeId;
				centerOnNode(activeNodeId);
			}
		});

		// Zoom — applied fresh (old ones removed in cleanup)
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

		// Mouse listeners — add once, remove in onDestroy
		canvasEl.onmousedown = handleMouseDown;
		canvasEl.onmousemove = handleMouseMove;
		canvasEl.onmouseup = handleMouseUp;
		canvasEl.onmouseleave = handleMouseLeave;
		canvasEl.onclick = handleClick;
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
		draw();
	}

	function handleClick(e: MouseEvent) {
		if (dragMoved) { dragMoved = false; return; }

		// Check legend click
		if (legendBounds && vaultList.length > 1) {
			const mx = e.offsetX, my = e.offsetY;
			const lb = legendBounds;
			if (mx >= lb.x && mx <= lb.x + lb.w && my >= lb.y && my <= lb.y + lb.h) {
				const idx = Math.floor((my - lb.y - lb.padY) / lb.lineH);
				if (idx >= 0 && idx < vaultList.length) {
					toggleVaultVisibility(vaultList[idx].name);
					return;
				}
			}
		}

		const [sx, sy] = currentTransform.invert([e.offsetX, e.offsetY]);
		const node = findNodeAt(sx, sy);
		if (node) {
			onNodeClick(node.path, node.vaultName);
		}
	}

	// ─── Canvas draw ───

	function draw() {
		if (!ctx || !canvasEl) return;
		const dark = isDarkTheme();

		const linkColor = getCSSVar('--graph-line') || (dark ? '#585b70' : '#8b8b96');
		const linkAlpha = 0.4;
		const linkDimAlpha = dark ? 0.06 : 0.08;
		const ringColor = getCSSVar('--text-normal') || (dark ? '#ffffff' : '#1f2328');
		const ringConnectedColor = getCSSVar('--text-muted') || (dark ? '#ffffffaa' : '#5c5c66');
		const labelColor = getCSSVar('--graph-text') || (dark ? '#cdd6f4' : '#1f2328');
		const labelShadow = getCSSVar('--graph-text-shadow') || (dark ? '#000000' : '#ffffff');
		const legendBg = dark ? 'rgba(24, 24, 36, 0.9)' : 'rgba(255, 255, 255, 0.94)';
		const legendBorder = dark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)';
		const legendText = getCSSVar('--graph-text') || (dark ? '#cdd6f4' : '#1f2328');

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

		// Convex hulls for vault clusters
		if (vaultList.length > 1) {
			const vaultPoints = new Map<string, [number, number][]>();
			for (const n of nodeData) {
				if (n.x == null) continue;
				const pts = vaultPoints.get(n.vaultName) || [];
				pts.push([n.x, n.y]);
				vaultPoints.set(n.vaultName, pts);
			}
			for (const [vName, pts] of vaultPoints) {
				if (pts.length < 3) continue;
				const hull = d3.polygonHull(pts);
				if (!hull) continue;
				const color = vaultColorMap.get(vName) || '#6b7280';
				ctx.beginPath();
				// Expand hull points outward for padding
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
			const isCrossVault = src.vaultName !== tgt.vaultName;
			const typedColor = l.linkType ? LINK_TYPE_COLORS[l.linkType] : null;

			ctx.beginPath();
			ctx.moveTo(src.x, src.y);
			ctx.lineTo(tgt.x, tgt.y);

			if (isCrossVault && !isHL) {
				ctx.setLineDash([4 / currentTransform.k, 3 / currentTransform.k]);
			} else {
				ctx.setLineDash([]);
			}

			if (isHL) {
				ctx.strokeStyle = highlightColor;
				ctx.lineWidth = 2 / currentTransform.k;
				ctx.globalAlpha = 0.9;
				ctx.setLineDash([]);
			} else if (typedColor) {
				ctx.strokeStyle = typedColor;
				ctx.lineWidth = 1.2 / currentTransform.k;
				ctx.globalAlpha = highlightId ? linkDimAlpha : 0.6;
			} else if (isCrossVault) {
				ctx.strokeStyle = linkColor;
				ctx.lineWidth = 1 / currentTransform.k;
				ctx.globalAlpha = highlightId ? linkDimAlpha : linkAlpha * 0.7;
			} else {
				ctx.strokeStyle = linkColor;
				ctx.lineWidth = 0.6 / currentTransform.k;
				ctx.globalAlpha = highlightId ? linkDimAlpha : linkAlpha;
			}
			ctx.stroke();
			ctx.setLineDash([]);
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

		// Labels
		if (highlightId) {
			const fontSize = Math.max(9, 12 / currentTransform.k);
			ctx.font = `600 ${fontSize}px system-ui, -apple-system, sans-serif`;
			ctx.textAlign = 'center';
			ctx.textBaseline = 'top';

			for (const n of nodeData) {
				if (n.x == null) continue;
				const isFocused = n.id === highlightId;
				const isConnected = connectedIds?.has(n.id);
				if (!isFocused && !isConnected) continue;

				const r = getNodeRadius(n);
				const yOff = n.y + r + 5 / currentTransform.k;
				const shadowOff = 1 / currentTransform.k;

				ctx.globalAlpha = isFocused ? 0.7 : 0.5;
				ctx.fillStyle = labelShadow;
				ctx.fillText(n.name, n.x + shadowOff, yOff + shadowOff);

				ctx.globalAlpha = isFocused ? 1 : 0.8;
				ctx.fillStyle = labelColor;
				ctx.fillText(n.name, n.x, yOff);
			}
		}

		ctx.restore();

		// Legend
		if (vaultList.length > 1) {
			ctx.save();
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

			const x = 14, y = 14, lineH = 22, dotR = 6, padX = 12, padY = 8, fs = 11;
			ctx.font = `500 ${fs}px system-ui, -apple-system, sans-serif`;

			let maxW = 0;
			for (const v of vaultList) {
				const label = `${v.name} (${v.count})`;
				const w = ctx.measureText(label).width;
				if (w > maxW) maxW = w;
			}
			const boxW = dotR * 2 + 12 + maxW + padX * 2;
			const boxH = vaultList.length * lineH + padY * 2;

			ctx.globalAlpha = 0.92;
			ctx.fillStyle = legendBg;
			ctx.strokeStyle = legendBorder;
			ctx.lineWidth = 1;
			roundRect(ctx, x, y, boxW, boxH, 8);
			ctx.fill();
			ctx.stroke();

			ctx.globalAlpha = 1;
			for (let i = 0; i < vaultList.length; i++) {
				const v = vaultList[i];
				const isHidden = hiddenVaults.has(v.name);
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

			// Store legend bounds for click detection
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
		<div class="graph-empty">{$t('graphView.noNotes')}</div>
	{:else}
		<canvas bind:this={canvasEl}></canvas>
		{#if tooltipVisible}
			<div class="graph-tooltip" style="left: {tooltipX}px; top: {tooltipY}px;">
				{tooltipText}
			</div>
		{/if}
	{/if}
</div>

<style>
	.graph-container {
		width: 100%; height: 100%;
		background: var(--background-secondary);
		position: relative;
		overflow: hidden;
	}
	canvas {
		display: block;
		width: 100%;
		height: 100%;
	}
	.graph-empty {
		position: absolute; inset: 0;
		display: flex; align-items: center; justify-content: center;
		color: var(--text-faint); font-size: 0.85rem;
	}
	.graph-tooltip {
		position: absolute;
		pointer-events: none;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 4px 10px;
		font-size: 0.8rem;
		font-weight: 500;
		color: var(--text-normal);
		box-shadow: var(--shadow-s);
		white-space: nowrap;
		z-index: 10;
	}
</style>

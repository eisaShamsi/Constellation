<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';

	interface GraphNode {
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
	}

	let {
		nodes,
		links,
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
	let resizeObserver: ResizeObserver | null = null;
	let themeObserver: MutationObserver | null = null;
	let animFrame = 0;

	// Layout state
	let nodePos: { x: number; y: number; vx: number; vy: number; node: GraphNode; r: number }[] = [];
	let linkIdx: { si: number; ti: number }[] = [];
	let hoveredIdx = -1;
	let showControls = false;

	// View transform (pan + zoom)
	let viewX = 0;
	let viewY = 0;
	let viewScale = 1;
	let dragging = false;
	let dragStartX = 0;
	let dragStartY = 0;
	let dragViewX = 0;
	let dragViewY = 0;

	// Simulation state
	let simRunning = false;
	let simAlpha = 1;
	let simIterations = 0;

	const VAULT_COLORS = ['#8b5cf6', '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#06b6d4', '#84cc16'];
	let vaultColorMap = new Map<string, string>();

	function isDark(): boolean {
		return document.body.classList.contains('theme-dark');
	}

	function initLayout() {
		if (!containerEl || !canvasEl) return;

		// Snapshot data to avoid proxy overhead
		const rawNodes: GraphNode[] = [];
		for (let i = 0; i < nodes.length; i++) {
			const n = nodes[i];
			rawNodes.push({ id: n.id, name: n.name, path: n.path, vaultName: n.vaultName, group: n.group, linkCount: n.linkCount });
		}
		const rawLinks: GraphLink[] = [];
		for (let i = 0; i < links.length; i++) {
			const l = links[i];
			rawLinks.push({ source: l.source, target: l.target });
		}

		// Assign vault colors
		const vaultNames = [...new Set(rawNodes.map(n => n.vaultName))];
		vaultColorMap = new Map();
		vaultNames.forEach((v, i) => vaultColorMap.set(v, VAULT_COLORS[i % VAULT_COLORS.length]));

		// Build node index
		const nodeIdxMap = new Map<string, number>();
		rawNodes.forEach((n, i) => nodeIdxMap.set(n.id, i));

		// Initial positions: cluster by vault, random within cluster
		const vaultClusters = new Map<string, { cx: number; cy: number }>();
		const clusterRadius = Math.sqrt(rawNodes.length) * 8;
		vaultNames.forEach((v, i) => {
			const angle = (2 * Math.PI * i) / vaultNames.length;
			vaultClusters.set(v, {
				cx: Math.cos(angle) * clusterRadius * 0.4,
				cy: Math.sin(angle) * clusterRadius * 0.4,
			});
		});

		nodePos = rawNodes.map(n => {
			const cluster = vaultClusters.get(n.vaultName) || { cx: 0, cy: 0 };
			const spread = clusterRadius * 0.3;
			return {
				x: cluster.cx + (Math.random() - 0.5) * spread,
				y: cluster.cy + (Math.random() - 0.5) * spread,
				vx: 0,
				vy: 0,
				node: n,
				r: Math.max(2, Math.min(8, 2 + Math.sqrt(n.linkCount))),
			};
		});

		// Build link index (resolve source/target to array indices)
		linkIdx = [];
		for (const l of rawLinks) {
			const si = nodeIdxMap.get(l.source);
			const ti = nodeIdxMap.get(l.target);
			if (si !== undefined && ti !== undefined) {
				linkIdx.push({ si, ti });
			}
		}

		// Center view
		viewX = 0;
		viewY = 0;
		viewScale = 1;

		// Start simulation
		simAlpha = 1;
		simIterations = 0;
		simRunning = true;
		runSimulation();
	}

	function runSimulation() {
		if (!simRunning) return;

		const n = nodePos.length;
		if (n === 0) return;

		// Run a few iterations per frame
		const itersPerFrame = 3;
		for (let iter = 0; iter < itersPerFrame; iter++) {
			simAlpha *= 0.995;
			if (simAlpha < 0.001) {
				simRunning = false;
				break;
			}

			// Center gravity
			for (let i = 0; i < n; i++) {
				nodePos[i].vx -= nodePos[i].x * 0.01 * simAlpha;
				nodePos[i].vy -= nodePos[i].y * 0.01 * simAlpha;
			}

			// Repulsion (Barnes-Hut approximation: simple grid-based)
			const repulse = 80 * simAlpha;
			// Use spatial hashing for efficiency
			const cellSize = 50;
			const cells = new Map<string, number[]>();
			for (let i = 0; i < n; i++) {
				const cx = Math.floor(nodePos[i].x / cellSize);
				const cy = Math.floor(nodePos[i].y / cellSize);
				const key = `${cx},${cy}`;
				if (!cells.has(key)) cells.set(key, []);
				cells.get(key)!.push(i);
			}

			for (let i = 0; i < n; i++) {
				const px = nodePos[i].x;
				const py = nodePos[i].y;
				const cx = Math.floor(px / cellSize);
				const cy = Math.floor(py / cellSize);

				// Check nearby cells
				for (let dx = -2; dx <= 2; dx++) {
					for (let dy = -2; dy <= 2; dy++) {
						const key = `${cx + dx},${cy + dy}`;
						const cell = cells.get(key);
						if (!cell) continue;
						for (const j of cell) {
							if (j <= i) continue;
							let ddx = px - nodePos[j].x;
							let ddy = py - nodePos[j].y;
							let dist = Math.sqrt(ddx * ddx + ddy * ddy) || 1;
							if (dist > cellSize * 3) continue;
							const force = repulse / (dist * dist);
							const fx = ddx / dist * force;
							const fy = ddy / dist * force;
							nodePos[i].vx += fx;
							nodePos[i].vy += fy;
							nodePos[j].vx -= fx;
							nodePos[j].vy -= fy;
						}
					}
				}
			}

			// Link attraction
			const linkForce = 0.05 * simAlpha;
			const idealDist = 30;
			for (const { si, ti } of linkIdx) {
				const dx = nodePos[ti].x - nodePos[si].x;
				const dy = nodePos[ti].y - nodePos[si].y;
				const dist = Math.sqrt(dx * dx + dy * dy) || 1;
				const force = (dist - idealDist) * linkForce;
				const fx = (dx / dist) * force;
				const fy = (dy / dist) * force;
				nodePos[si].vx += fx;
				nodePos[si].vy += fy;
				nodePos[ti].vx -= fx;
				nodePos[ti].vy -= fy;
			}

			// Apply velocity with damping
			for (let i = 0; i < n; i++) {
				nodePos[i].vx *= 0.6;
				nodePos[i].vy *= 0.6;
				nodePos[i].x += nodePos[i].vx;
				nodePos[i].y += nodePos[i].vy;
			}

			simIterations++;
		}

		draw();
		if (simRunning) {
			animFrame = requestAnimationFrame(runSimulation);
		}
	}

	function setupCanvas() {
		if (!containerEl || !canvasEl) return;
		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		if (width === 0 || height === 0) return;

		const dpr = window.devicePixelRatio || 1;
		canvasEl.width = width * dpr;
		canvasEl.height = height * dpr;
		canvasEl.style.width = width + 'px';
		canvasEl.style.height = height + 'px';
		ctx = canvasEl.getContext('2d');
	}

	function draw() {
		if (!ctx || !canvasEl || !containerEl) return;

		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		const dpr = window.devicePixelRatio || 1;
		const dark = isDark();

		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, width, height);

		// Apply view transform (center + pan + zoom)
		ctx.save();
		ctx.translate(width / 2 + viewX, height / 2 + viewY);
		ctx.scale(viewScale, viewScale);

		// Draw links
		ctx.strokeStyle = dark ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.06)';
		ctx.lineWidth = 0.5 / viewScale;
		ctx.beginPath();
		for (const { si, ti } of linkIdx) {
			ctx.moveTo(nodePos[si].x, nodePos[si].y);
			ctx.lineTo(nodePos[ti].x, nodePos[ti].y);
		}
		ctx.stroke();

		// Draw nodes
		for (let i = 0; i < nodePos.length; i++) {
			const p = nodePos[i];
			const isActive = p.node.id === activeNodeId;
			const isHovered = i === hoveredIdx;
			const color = vaultColorMap.get(p.node.vaultName) || '#6b7280';
			const r = isHovered ? p.r * 1.5 : isActive ? p.r * 1.3 : p.r;

			ctx.beginPath();
			ctx.arc(p.x, p.y, r / viewScale, 0, Math.PI * 2);
			ctx.fillStyle = color;
			ctx.fill();

			if (isActive) {
				ctx.strokeStyle = dark ? '#fff' : '#333';
				ctx.lineWidth = 2 / viewScale;
				ctx.stroke();
			}
		}

		ctx.restore();

		// Draw labels in screen space (fixed size regardless of zoom)
		const fontSize = 12;
		ctx.font = `${fontSize}px system-ui, -apple-system, sans-serif`;
		ctx.textAlign = 'center';

		for (let i = 0; i < nodePos.length; i++) {
			const p = nodePos[i];
			const isActive = p.node.id === activeNodeId;
			const isHovered = i === hoveredIdx;
			if (!isActive && !isHovered) continue;

			// Convert world position to screen position
			const sx = p.x * viewScale + width / 2 + viewX;
			const sy = p.y * viewScale + height / 2 + viewY;

			const label = p.node.name.replace(/\.md$/, '');
			const textWidth = ctx.measureText(label).width;
			const px = 5;
			const py = 3;
			const labelY = sy + p.r + 14;

			ctx.fillStyle = dark ? 'rgba(0,0,0,0.75)' : 'rgba(255,255,255,0.9)';
			ctx.beginPath();
			ctx.roundRect(
				sx - textWidth / 2 - px,
				labelY - fontSize,
				textWidth + px * 2,
				fontSize + py * 2,
				3
			);
			ctx.fill();

			ctx.fillStyle = dark ? '#e5e5e5' : '#333';
			ctx.fillText(label, sx, labelY);
		}

		// Status text
		ctx.fillStyle = dark ? 'rgba(255,255,255,0.4)' : 'rgba(0,0,0,0.3)';
		ctx.font = '11px system-ui';
		ctx.textAlign = 'right';
		const statusText = simRunning
			? `${nodePos.length} nodes · ${linkIdx.length} links · simulating...`
			: `${nodePos.length} nodes · ${linkIdx.length} links`;
		ctx.fillText(statusText, width - 12, height - 12);
	}

	// ─── Mouse interaction ───

	function screenToWorld(sx: number, sy: number): { wx: number; wy: number } {
		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		return {
			wx: (sx - width / 2 - viewX) / viewScale,
			wy: (sy - height / 2 - viewY) / viewScale,
		};
	}

	function findNodeAt(wx: number, wy: number): number {
		const hitRadius = 12 / viewScale;
		for (let i = nodePos.length - 1; i >= 0; i--) {
			const dx = wx - nodePos[i].x;
			const dy = wy - nodePos[i].y;
			if (dx * dx + dy * dy < hitRadius * hitRadius) return i;
		}
		return -1;
	}

	function handleMouseMove(e: MouseEvent) {
		if (dragging) {
			viewX = dragViewX + (e.clientX - dragStartX);
			viewY = dragViewY + (e.clientY - dragStartY);
			if (!simRunning) draw();
			return;
		}

		const rect = canvasEl.getBoundingClientRect();
		const { wx, wy } = screenToWorld(e.clientX - rect.left, e.clientY - rect.top);
		const idx = findNodeAt(wx, wy);

		if (idx !== hoveredIdx) {
			hoveredIdx = idx;
			canvasEl.style.cursor = idx >= 0 ? 'pointer' : 'grab';
			if (!simRunning) draw();
		}
	}

	function handleMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		if (hoveredIdx >= 0) return; // Will handle via click
		dragging = true;
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		dragViewX = viewX;
		dragViewY = viewY;
		canvasEl.style.cursor = 'grabbing';
	}

	function handleMouseUp(e: MouseEvent) {
		dragging = false;
		canvasEl.style.cursor = hoveredIdx >= 0 ? 'pointer' : 'grab';
	}

	function handleClick(e: MouseEvent) {
		if (hoveredIdx >= 0) {
			const n = nodePos[hoveredIdx].node;
			onNodeClick(n.path, n.vaultName);
		}
	}

	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
		const rect = canvasEl.getBoundingClientRect();
		const mx = e.clientX - rect.left;
		const my = e.clientY - rect.top;

		// Zoom toward mouse position
		const width = containerEl.clientWidth;
		const height = containerEl.clientHeight;
		const wx = mx - width / 2 - viewX;
		const wy = my - height / 2 - viewY;

		viewScale *= zoomFactor;
		viewScale = Math.max(0.1, Math.min(10, viewScale));
		viewX -= wx * (zoomFactor - 1);
		viewY -= wy * (zoomFactor - 1);

		if (!simRunning) draw();
	}

	onMount(() => {
		setupCanvas();

		resizeObserver = new ResizeObserver(() => {
			setupCanvas();
			if (!simRunning) draw();
		});
		if (containerEl) resizeObserver.observe(containerEl);

		themeObserver = new MutationObserver(() => {
			if (!simRunning && nodePos.length > 0) draw();
		});
		themeObserver.observe(document.body, { attributes: true, attributeFilter: ['class'] });

		// Defer init to next frame to avoid blocking mount
		requestAnimationFrame(() => {
			initLayout();
		});
	});

	onDestroy(() => {
		simRunning = false;
		cancelAnimationFrame(animFrame);
		resizeObserver?.disconnect();
		themeObserver?.disconnect();
	});

	// Re-layout when nodes change significantly
	let prevLen = 0;
	$effect(() => {
		const len = nodes.length;
		if (len !== prevLen && len > 0) {
			prevLen = len;
			requestAnimationFrame(() => initLayout());
		}
	});
</script>

<div class="graph-container" bind:this={containerEl}>
	<canvas
		bind:this={canvasEl}
		onmousemove={handleMouseMove}
		onmousedown={handleMouseDown}
		onmouseup={handleMouseUp}
		onclick={handleClick}
		onwheel={handleWheel}
		onmouseleave={() => { dragging = false; hoveredIdx = -1; if (!simRunning) draw(); }}
	></canvas>
</div>

<style>
	.graph-container {
		width: 100%;
		height: 100%;
		position: relative;
		overflow: hidden;
		background: var(--background-secondary);
	}
	canvas {
		display: block;
		width: 100%;
		height: 100%;
		cursor: grab;
	}
</style>

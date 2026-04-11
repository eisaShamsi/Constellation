<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	interface SkyNode {
		id: string;
		name: string;
		path: string;
		libraryName: string;
		linkCount: number;
	}

	interface SkyLink {
		source: string;
		target: string;
	}

	let {
		nodes,
		links,
		activeNodeId = '',
		onNodeClick,
	}: {
		nodes: SkyNode[];
		links: SkyLink[];
		activeNodeId?: string;
		onNodeClick: (id: string) => void;
	} = $props();

	let containerEl: HTMLDivElement;
	let canvasEl: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let nodePositions: { node: SkyNode; x: number; y: number; r: number }[] = [];
	let resizeObserver: ResizeObserver | null = null;
	let hoveredNode: SkyNode | null = null;

	const LIBRARY_COLORS = ['#8b5cf6', '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#ec4899', '#06b6d4', '#84cc16'];
	let libraryColorMap = new Map<string, string>();

	function isDark(): boolean {
		return document.body.classList.contains('theme-dark');
	}

	function layout() {
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
		if (!ctx) return;

		// Assign library colors
		const libraryNames = [...new Set(nodes.map(n => n.libraryName))];
		libraryColorMap = new Map();
		libraryNames.forEach((v, i) => libraryColorMap.set(v, LIBRARY_COLORS[i % LIBRARY_COLORS.length]));

		// Position nodes: active at center, others in circle
		const cx = width / 2;
		const cy = height / 2;
		const radius = Math.min(width, height) * 0.32;
		const active = nodes.find(n => n.id === activeNodeId);
		const others = nodes.filter(n => n.id !== activeNodeId);

		nodePositions = [];
		if (active) {
			nodePositions.push({ node: active, x: cx, y: cy, r: 8 });
		}
		others.forEach((n, i) => {
			const angle = (2 * Math.PI * i) / Math.max(others.length, 1) - Math.PI / 2;
			nodePositions.push({
				node: n,
				x: cx + Math.cos(angle) * radius,
				y: cy + Math.sin(angle) * radius,
				r: 5,
			});
		});

		draw(dpr);
	}

	function draw(dpr?: number) {
		if (!ctx || !canvasEl) return;
		dpr = dpr || window.devicePixelRatio || 1;
		const dark = isDark();

		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);

		const posMap = new Map(nodePositions.map(p => [p.node.id, p]));

		// Draw links
		ctx.strokeStyle = dark ? 'rgba(255,255,255,0.15)' : 'rgba(0,0,0,0.12)';
		ctx.lineWidth = 1;
		for (const link of links) {
			const s = posMap.get(link.source);
			const t = posMap.get(link.target);
			if (s && t) {
				ctx.beginPath();
				ctx.moveTo(s.x, s.y);
				ctx.lineTo(t.x, t.y);
				ctx.stroke();
			}
		}

		// Draw nodes
		for (const pos of nodePositions) {
			const isActive = pos.node.id === activeNodeId;
			const isHovered = hoveredNode === pos.node;
			const color = libraryColorMap.get(pos.node.libraryName) || '#6b7280';
			const r = isActive ? 8 : isHovered ? 7 : 5;

			ctx.beginPath();
			ctx.arc(pos.x, pos.y, r, 0, Math.PI * 2);
			ctx.fillStyle = isActive ? color : (dark ? adjustAlpha(color, 0.8) : color);
			ctx.fill();

			if (isActive) {
				ctx.strokeStyle = dark ? '#fff' : '#333';
				ctx.lineWidth = 2;
				ctx.stroke();
			}
		}

		// Draw labels for all nodes
		const fontSize = 10;
		ctx.font = `${fontSize}px system-ui, -apple-system, sans-serif`;
		ctx.textAlign = 'center';
		for (const pos of nodePositions) {
			const isActive = pos.node.id === activeNodeId;
			const isHovered = hoveredNode === pos.node;

			const label = pos.node.name.replace(/\.md$/, '');
			const textWidth = ctx.measureText(label).width;

			ctx.fillStyle = (isActive || isHovered)
				? (dark ? '#fff' : '#000')
				: (dark ? '#bbb' : '#555');
			ctx.fillText(label, pos.x, pos.y + (isActive ? 12 : 9) + fontSize - 1);
		}
	}

	function adjustAlpha(hex: string, alpha: number): string {
		const r = parseInt(hex.slice(1, 3), 16);
		const g = parseInt(hex.slice(3, 5), 16);
		const b = parseInt(hex.slice(5, 7), 16);
		return `rgba(${r},${g},${b},${alpha})`;
	}

	function handleMouseMove(e: MouseEvent) {
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const y = e.clientY - rect.top;

		let found: SkyNode | null = null;
		for (const pos of nodePositions) {
			const dx = x - pos.x;
			const dy = y - pos.y;
			if (dx * dx + dy * dy < 100) { // 10px radius
				found = pos.node;
				break;
			}
		}

		if (found !== hoveredNode) {
			hoveredNode = found;
			canvasEl.style.cursor = found ? 'pointer' : 'default';
			draw();
		}
	}

	function handleClick(e: MouseEvent) {
		if (!canvasEl || !hoveredNode) return;
		onNodeClick(hoveredNode.id);
	}

	onMount(() => {
		resizeObserver = new ResizeObserver(() => {
			requestAnimationFrame(layout);
		});
		if (containerEl) resizeObserver.observe(containerEl);
		// Initial layout deferred
		requestAnimationFrame(layout);
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// Re-layout when nodes change
	let prevLen = 0;
	$effect(() => {
		const len = nodes.length;
		if (len !== prevLen) {
			prevLen = len;
			requestAnimationFrame(layout);
		}
	});
</script>

<div class="local-star" bind:this={containerEl}>
	<canvas
		bind:this={canvasEl}
		onmousemove={handleMouseMove}
		onclick={handleClick}
	></canvas>
</div>

<style>
	.local-star {
		width: 100%;
		height: 100%;
		min-height: 200px;
		position: relative;
		background: var(--background-secondary);
	}
	canvas {
		display: block;
		width: 100%;
		height: 100%;
	}
</style>

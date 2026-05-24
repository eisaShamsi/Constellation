<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	// MIG-044 Phase 2 (correction #2) — NSC summary headline in the hover
	// tooltip. The two earlier corrections wired SkyView.svelte and
	// FullSkyView.svelte; both turned out to be dead code (no static
	// importer in the source tree — confirmed via `grep -r "import.*SkyView" src/`).
	// The component actually mounted in the right-side panel AND the second
	// screen is THIS file, LocalSkyView. Wiring lands here.
	import { getSummaryFor } from '$lib/nsc/summaryStore';

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

	// MIG-044 Phase 2 (correction #2) — hover tooltip state.
	// Two-line shape: name on top, NSC summary headline below in muted
	// italic. Tooltip positioned in canvas-local coords (anchor the parent
	// `.local-star` is already position: relative). Headline fetched
	// lazily via the shared store with a monotonic stale-promise guard.
	let tooltipText = $state('');
	let tooltipX = $state(0);
	let tooltipY = $state(0);
	let tooltipVisible = $state(false);
	let tooltipHeadline = $state('');
	let tooltipHeadlinePath = '';
	let tooltipHeadlineToken = 0;

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

		// MIG-044 Phase 2 (correction #2) — drive HTML tooltip alongside
		// the canvas hover detection. Position in canvas-local coords so
		// it tracks the cursor without depending on page scroll. Headline
		// fetched lazily; the monotonic token guards stale promises.
		if (found) {
			tooltipText = found.name.replace(/\.md$/, '');
			// Edge-aware positioning: default right of cursor; flip left if
			// the tooltip would overflow the canvas; same shape vertically.
			// Width/height are upper bounds from CSS (max-width 240, ~80h).
			const panelW = canvasEl.clientWidth;
			const panelH = canvasEl.clientHeight;
			const W = 240, H = 80, PAD = 8;
			tooltipX = (x + 14 + W + PAD > panelW) ? Math.max(PAD, x - W - 14) : x + 14;
			tooltipY = (y - 10 + H + PAD > panelH) ? Math.max(PAD, y - H - 14) : y - 10;
			tooltipVisible = true;
			if (found.path !== tooltipHeadlinePath) {
				tooltipHeadlinePath = found.path;
				tooltipHeadline = '';
				const myToken = ++tooltipHeadlineToken;
				const targetPath = found.path;
				getSummaryFor(targetPath).then((entry) => {
					if (myToken !== tooltipHeadlineToken) return;
					tooltipHeadline = entry?.headline ?? '';
				}).catch(() => { /* ignore */ });
			}
		} else {
			tooltipVisible = false;
		}
	}

	function handleMouseLeave() {
		// MIG-044 Phase 2 (correction #2) — clear hover + tooltip together.
		if (hoveredNode !== null) {
			hoveredNode = null;
			if (canvasEl) canvasEl.style.cursor = 'default';
			draw();
		}
		tooltipVisible = false;
		tooltipHeadlinePath = '';
		tooltipHeadline = '';
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
		onmouseleave={handleMouseLeave}
		onclick={handleClick}
	></canvas>
	<!-- MIG-044 Phase 2 (correction #2) — hover tooltip (name + headline) -->
	{#if tooltipVisible}
		<div class="local-star-tooltip" style="left: {tooltipX}px; top: {tooltipY}px;" dir="auto">
			<div class="local-star-tooltip-name">{tooltipText}</div>
			{#if tooltipHeadline}
				<div class="local-star-tooltip-headline" title={tooltipHeadline}>{tooltipHeadline}</div>
			{/if}
		</div>
	{/if}
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
	/* MIG-044 Phase 2 (correction #2) — hover tooltip. Two-line layout,
	   pointer-events:none so it never blocks canvas mouse handling.
	   Class name is intentionally namespaced (.local-star-tooltip*) to
	   keep Svelte's scoped CSS pruner from clipping it. */
	.local-star-tooltip {
		position: absolute;
		pointer-events: none;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		padding: 4px 10px;
		font-size: 0.8rem;
		color: var(--text-normal);
		box-shadow: var(--shadow-s);
		/* Narrower than before (was 320) — keeps the tooltip inside the
		   typical SV panel width even when the bubble is near the right
		   edge. Combined with the JS-side flip in handleMouseMove. */
		max-width: 240px;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.local-star-tooltip-name {
		font-weight: 500;
		/* Name stays single-line + ellipsis: note names are usually short
		   and a wrapped name reads awkwardly above a wrapped headline. */
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.local-star-tooltip-headline {
		font-size: 0.7rem;
		font-style: italic;
		color: var(--text-faint);
		line-height: 1.35;
		/* Allow the headline to wrap to multiple lines so users see the
		   full first sentence instead of "…military" truncation. Cap to
		   3 lines so the tooltip stays compact for long summaries. */
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
		word-wrap: break-word;
		overflow-wrap: anywhere;
	}
</style>

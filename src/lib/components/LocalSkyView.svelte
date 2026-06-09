<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	// MIG-044 Phase 2 (correction #2) — NSC summary headline in the hover
	// tooltip. The two earlier corrections wired SkyView.svelte and
	// FullSkyView.svelte; both turned out to be dead code (no static
	// importer in the source tree — confirmed via `grep -r "import.*SkyView" src/`).
	// The component actually mounted in the right-side panel AND the second
	// screen is THIS file, LocalSkyView. Wiring lands here.
	import { getSummaryFor } from '$lib/nsc/summaryStore';
	// MIG-072 §5 — second-screen parity: LocalSkyView shares the full Sky View palette.
	import { resolveSkyPalette } from '$lib/graph/skyPalette';
	import { appSettings, liveStyleDraft } from '$lib/libraries/store';

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
		libraryColorMap = {},
	}: {
		nodes: SkyNode[];
		links: SkyLink[];
		activeNodeId?: string;
		onNodeClick: (id: string) => void;
		// MIG-072 §5 — the SAME canonical { libraryName → colour } map the full Sky View uses
		// (built once via buildLibraryColorMap($libraries) at the mount). Replaces LSV's old
		// divergent LIBRARY_COLORS so a library is the same colour in both renderers.
		libraryColorMap?: Record<string, string>;
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

	// MIG-072 §5 — palette resolved in the Svelte layer exactly as GraphMindView does, so the
	// companion graph honours the Style Setter and matches the full Sky View. draw() never calls
	// getComputedStyle (Perf Rule 3). LSV draws no typed edges, so typedLinks is empty ({}).
	let isDark = $state(typeof document !== 'undefined' && document.body.classList.contains('theme-dark'));
	let skyThemeObserver: MutationObserver | null = null;
	const skyPalette = $derived(
		resolveSkyPalette({ ...($appSettings.styleOverride ?? {}), ...$liveStyleDraft }, isDark, {})
	);
	function skyHex(n: number): string { return '#' + n.toString(16).padStart(6, '0'); }

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
		const dark = isDark;

		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);

		const posMap = new Map(nodePositions.map(p => [p.node.id, p]));

		// Draw links — MIG-072 §5: untyped edge colour + opacity from the shared palette.
		ctx.strokeStyle = adjustAlpha(skyHex(skyPalette.edgeNormal), skyPalette.edgeNormalAlpha);
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
			// MIG-072 §5: canonical library colour (prop) → palette node-default fallback.
			const color = libraryColorMap[pos.node.libraryName] || skyHex(skyPalette.nodeDefault);
			const r = isActive ? 8 : isHovered ? 7 : 5;

			ctx.beginPath();
			ctx.arc(pos.x, pos.y, r, 0, Math.PI * 2);
			ctx.fillStyle = isActive ? color : (dark ? adjustAlpha(color, 0.8) : color);
			ctx.fill();

			if (isActive) {
				// MIG-072 §5: the open-note ring colour from the shared palette.
				ctx.strokeStyle = skyHex(skyPalette.ringActive);
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

			// MIG-072 §5: label colour from the shared palette; active/hovered at full strength,
			// the rest faded — preserves the companion's "active note stands out" affordance.
			const labelHex = skyHex(skyPalette.label);
			ctx.fillStyle = (isActive || isHovered) ? labelHex : adjustAlpha(labelHex, 0.6);
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
		// MIG-072 §5 — track the body theme class so the palette's dark/light defaults resolve
		// correctly (mirrors GraphMindView); a change flips isDark → redraw via the palette $effect.
		skyThemeObserver = new MutationObserver(() => {
			const d = document.body.classList.contains('theme-dark');
			if (d !== isDark) isDark = d;
		});
		skyThemeObserver.observe(document.body, { attributes: true, attributeFilter: ['class'] });
		// Initial layout deferred
		requestAnimationFrame(layout);
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		resizeObserver = null;
		skyThemeObserver?.disconnect(); // MIG-072 §5 — Rule 4: no leaked observers
		skyThemeObserver = null;
	});

	// MIG-072 §5 — repaint when the Style Setter palette or the theme changes (positions are
	// unchanged, so a draw() is enough). This is the live-preview path with the Setter open.
	// Track ONLY palette + theme; untrack draw() so we add no new redraw triggers for the
	// props it reads (links/activeNodeId keep their existing redraw paths untouched).
	$effect(() => {
		void skyPalette; void isDark;
		untrack(() => { if (ctx) draw(); });
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
		/* Shares the Style Setter → Sky View → Canvas background (--skyview-bg) with
		   the full Sky View, so both look identical. The 2D canvas clears with
		   clearRect (transparent), so this CSS colour shows through. */
		background: var(--skyview-bg, var(--background-secondary));
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

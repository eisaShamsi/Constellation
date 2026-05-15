<!--
  MIG-025 §B.1 — Sight v6 mini-dome Svelte wrapper.

  Thin Svelte component that wraps the pure-function renderer in
  miniDome.ts. One canvas per mini-dome; resize via ResizeObserver;
  re-paint on stars or highlightedPath change.

  §B.1 ships the structural skeleton; channel-specific rendering
  lands incrementally in §B.2 (confidence) → §B.3 (stage) →
  §B.4 (acts) → §B.5 (provenance). Linked brushing (§B.6) and
  cross-filter (§B.7) wire later.
-->
<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import type { StarDerived, MiniDomeChannel } from './types';
	import { renderMiniDome, miniDomeHitTest } from './miniDome';
	import type { DomeLayout } from './anchor';

	let {
		channel,
		stars,
		anchorLayout,
		highlightedPath = null,
		onHover = () => {},
	}: {
		channel: MiniDomeChannel;
		stars: StarDerived[];
		anchorLayout: DomeLayout;
		highlightedPath?: string | null;
		/** §B.6 — fired when the cursor moves over a star (or null
		 *  when the cursor moves off all stars / leaves the canvas).
		 *  The parent (SightV6) owns the canonical hoveredPath; the
		 *  mini only PROPOSES via onHover() and receives the resolved
		 *  value back through the highlightedPath prop. */
		onHover?: (path: string | null) => void;
	} = $props();

	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let hostEl = $state<HTMLDivElement | null>(null);
	let canvasWidth = $state(0);
	let canvasHeight = $state(0);
	let dpr = $state(1);
	let resizeObserver: ResizeObserver | null = null;

	function syncCanvasSize(): void {
		if (!canvasEl || !hostEl) return;
		const rect = hostEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;
		dpr = Math.max(1, window.devicePixelRatio || 1);
		canvasEl.width = Math.max(1, Math.floor(canvasWidth * dpr));
		canvasEl.height = Math.max(1, Math.floor(canvasHeight * dpr));
		paint();
	}

	function paint(): void {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		renderMiniDome(ctx, stars, channel, canvasWidth, canvasHeight, anchorLayout, {
			highlightedPath,
		});
	}

	// §B.6 — pointer hit-test → dispatch hover upward to SightV6, which
	// updates its canonical hoveredPath. The new value flows back to ALL
	// 4 minis (and the anchor) via the highlightedPath prop, completing
	// the bidirectional linked-brushing loop with a single source of truth.
	function handlePointerMove(ev: PointerEvent): void {
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const x = ev.clientX - rect.left;
		const y = ev.clientY - rect.top;
		const hit = miniDomeHitTest(stars, x, y, channel, anchorLayout, canvasWidth, canvasHeight);
		if (hit !== highlightedPath) {
			onHover(hit);
		}
	}

	function handlePointerLeave(): void {
		if (highlightedPath !== null) {
			onHover(null);
		}
	}

	onMount(() => {
		syncCanvasSize();
		if (hostEl) {
			resizeObserver = new ResizeObserver(() => syncCanvasSize());
			resizeObserver.observe(hostEl);
		}
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// Repaint on star set, anchor layout, or highlight change.
	$effect(() => {
		void stars;
		void highlightedPath;
		void anchorLayout;
		untrack(() => paint());
	});
</script>

<div bind:this={hostEl} class="mini-dome-host">
	<canvas
		bind:this={canvasEl}
		class="mini-dome-canvas"
		class:has-hover={highlightedPath !== null}
		onpointermove={handlePointerMove}
		onpointerleave={handlePointerLeave}
	></canvas>
</div>

<style>
	.mini-dome-host {
		position: relative;
		width: 100%;
		height: 100%;
		min-width: 0;
		min-height: 0;
	}
	.mini-dome-canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		display: block;
		cursor: default;
	}
	.mini-dome-canvas.has-hover {
		cursor: pointer;
	}
</style>

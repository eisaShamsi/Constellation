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
	import { renderMiniDome } from './miniDome';

	let {
		channel,
		stars,
		highlightedPath = null,
	}: {
		channel: MiniDomeChannel;
		stars: StarDerived[];
		highlightedPath?: string | null;
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
		renderMiniDome(ctx, stars, channel, canvasWidth, canvasHeight, {
			highlightedPath,
		});
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

	// Repaint on star set or highlight change.
	$effect(() => {
		void stars;
		void highlightedPath;
		untrack(() => paint());
	});
</script>

<div bind:this={hostEl} class="mini-dome-host">
	<canvas bind:this={canvasEl} class="mini-dome-canvas"></canvas>
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
	}
</style>

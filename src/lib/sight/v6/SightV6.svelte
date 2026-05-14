<!--
  MIG-025 §A.6 — Sight v6 main component (placeholder).

  This is the §A.6 skeleton mount: header strip + canvas div + a
  visible "Sight v6 (under construction)" placeholder so the §A.7
  +layout.svelte mount block has a concrete component to import.

  §A.8 lands the anchor dome chrome (5 strata, calendar rim, labels).
  §A.9 lands the anchor stars + lines (channel encoding).
  §A.10 lands the facet sidebar.
  §A.11 lands the first-boot tour overlay.
  §B (Phase 2) lands the four mini-domes + cross-filter.
  §C (Phase 3) lands the register chip + 4 production registers.
  §D (Phase 4) lands the 3 v1-preview registers + tests + v5 deletion.

  Visual contract: docs/sight-redesign-v0.3-full-layout.svg
  Concept Paper:    docs/Constellation-Sight-Concept-Paper-v4.0.md
-->
<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { backfillProgress } from './backfillProgress.svelte';
	import { renderAnchorDome } from './anchor';

	// Signature mirrors SightV5 so +layout.svelte can pass the same
	// (path, libraryName) → openNoteTab callback to either engine
	// without per-engine adapter logic.
	let { onOpenNote = (_path: string, _libraryName: string) => {} }: {
		onOpenNote?: (path: string, libraryName: string) => void;
	} = $props();

	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let canvasHostEl = $state<HTMLDivElement | null>(null);
	let canvasWidth = $state(0);
	let canvasHeight = $state(0);
	let dpr = $state(1);

	let resizeObserver: ResizeObserver | null = null;

	function paint(): void {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		// CSS-pixel layout dimensions (logical units) — render math runs
		// in CSS pixels; the DPR scaling happens in the canvas backing
		// store via setTransform below.
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		renderAnchorDome(
			ctx,
			[],          // §A.9 will pass real stars
			[],          // §A.9 will pass real link edges
			canvasWidth,
			canvasHeight,
			{ locale: navigator.language ?? 'en' },
		);
	}

	function syncCanvasSize(): void {
		if (!canvasEl || !canvasHostEl) return;
		const rect = canvasHostEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;
		dpr = Math.max(1, window.devicePixelRatio || 1);
		// Set the canvas backing store to DPR-scaled physical pixels;
		// CSS keeps it at logical CSS-pixel size via the .sight-v6-canvas
		// {position:absolute, inset:0} block below.
		canvasEl.width = Math.max(1, Math.floor(canvasWidth * dpr));
		canvasEl.height = Math.max(1, Math.floor(canvasHeight * dpr));
		paint();
	}

	onMount(async () => {
		// §A.4 progressive backfill listener — start as soon as we mount.
		// renderReady flips when tier 1 completes; the anchor dome (§A.8/9)
		// will gate its first STAR paint on this signal. The CHROME
		// (strata rings, calendar rim, labels) renders immediately so the
		// user sees the dome instantly — no blank-screen-while-warming.
		await backfillProgress.start();
		syncCanvasSize();
		if (canvasHostEl) {
			resizeObserver = new ResizeObserver(() => syncCanvasSize());
			resizeObserver.observe(canvasHostEl);
		}
	});

	onDestroy(() => {
		backfillProgress.stop();
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// Re-paint when geometry changes. The chrome is render-ready
	// without backfill; star+line paints (§A.9) will gate on
	// backfillProgress.renderReady.
	$effect(() => {
		void canvasWidth;
		void canvasHeight;
		void dpr;
		untrack(() => paint());
	});

	// silence unused-prop lint; §A.9 wires onOpenNote to star clicks.
	$effect(() => {
		void onOpenNote;
	});
</script>

<div class="sight-v6-root">
	<div class="sight-v6-header">
		<span class="sight-v6-title">Constellation Sight</span>
		<span class="sight-v6-subtitle">v6.0 — under construction</span>
	</div>

	<div bind:this={canvasHostEl} class="sight-v6-canvas-host">
		<canvas bind:this={canvasEl} class="sight-v6-canvas"></canvas>
		{#if !backfillProgress.renderReady}
			<div class="sight-v6-loading">
				{#if backfillProgress.progress}
					Sight v6 cache: tier {backfillProgress.progress.tier}/5
					({backfillProgress.progress.doneRows}/{backfillProgress.progress.totalRows})
				{:else}
					Preparing Sight v6 cache…
				{/if}
			</div>
		{:else if !backfillProgress.done}
			<div class="sight-v6-loading sight-v6-loading-bg">
				Tier {backfillProgress.progress?.tier}/5 streaming…
			</div>
		{/if}
	</div>
</div>

<style>
	.sight-v6-root {
		position: relative;
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background: #080c16;
		color: #e8ebf2;
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);
	}

	.sight-v6-header {
		flex: 0 0 auto;
		display: flex;
		align-items: baseline;
		gap: 12px;
		padding: 14px 24px;
		border-bottom: 1px solid #1a1f2e;
	}

	.sight-v6-title {
		font-size: 18px;
		font-weight: 500;
		letter-spacing: 0.5px;
	}

	.sight-v6-subtitle {
		font-size: 11px;
		color: #5a6275;
	}

	.sight-v6-canvas-host {
		flex: 1 1 auto;
		position: relative;
		overflow: hidden;
	}

	.sight-v6-canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		display: block;
	}

	.sight-v6-loading {
		position: absolute;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		font-size: 12px;
		color: #7a8295;
		padding: 8px 16px;
		background: rgba(13, 19, 34, 0.85);
		border: 1px solid #2a3245;
		border-radius: 6px;
		pointer-events: none;
	}

	.sight-v6-loading-bg {
		left: auto;
		right: 16px;
		top: auto;
		bottom: 16px;
		transform: none;
		font-size: 10px;
		color: #5a6275;
		opacity: 0.7;
	}
</style>

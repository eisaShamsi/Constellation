<!--
  MIG-025 §A.6/§A.7/§A.8/§A.9/§A.10 — Sight v6 main component.

  §A.6  — placeholder mount
  §A.7  — wired into +layout.svelte (B2 dual-mount)
  §A.8  — chrome paints (5 strata + calendar rim + labels)
  §A.9  — IPC integration: warm_cache → render_ready event → load
          layout + links → compute positions → paint stars + lines.
          Pointer events → hit-test → openNote callback.
  §A.10 — facet sidebar (Hearst Flamenco, 6 facets, Folder TOP).
          Filter state held in this component; facets.ts does the
          filter logic + Hearst preview-count computation.

  §A.11 lands the first-boot tour.
  §A.12 lands the v5→v6 settings migration.
  §A.13 lands the CI perf harness.

  Visual contract: docs/sight-redesign-v0.3-full-layout.svg
  Concept Paper:    docs/Constellation-Sight-Concept-Paper-v4.0.md
-->
<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { backfillProgress } from './backfillProgress.svelte';
	import {
		renderAnchorDome,
		computeStarPositions,
		computeDomeLayout,
		starHitTest,
	} from './anchor';
	import {
		emptyFilters,
		applyFilters,
		computeFacetCounts,
		toggleFilter,
		type FacetFilters,
	} from './facets';
	import FacetSidebar from './facetSidebar.svelte';
	import type { LayoutCacheRow, LinkEdge, StarDerived, FacetId } from './types';

	let { onOpenNote = (_path: string, _libraryName: string) => {} }: {
		onOpenNote?: (path: string, libraryName: string) => void;
	} = $props();

	// ── Canvas state ───────────────────────────────────────────────
	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let canvasHostEl = $state<HTMLDivElement | null>(null);
	let canvasWidth = $state(0);
	let canvasHeight = $state(0);
	let dpr = $state(1);

	// ── Data state ─────────────────────────────────────────────────
	let rows = $state<LayoutCacheRow[]>([]);
	let links = $state<LinkEdge[]>([]);
	let stars = $state<StarDerived[]>([]);
	let hoveredPath = $state<string | null>(null);

	// ── §A.10 facet state ──────────────────────────────────────────
	let filters = $state<FacetFilters>(emptyFilters());
	let sidebarExpanded = $state(false);

	// Filtered row set + recomputed facet counts (Hearst preview).
	const filteredRows = $derived(applyFilters(rows, filters));
	const facets = $derived(computeFacetCounts(rows, filters));

	let resizeObserver: ResizeObserver | null = null;

	// ── Render ────────────────────────────────────────────────────

	function paint(): void {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		// Filter the visible link set to edges where BOTH endpoints
		// survive the facet filter — keeps the dome readable when the
		// user narrows the universe.
		const visiblePaths = new Set(filteredRows.map((r) => r.notePath));
		const visibleLinks = links.filter(
			(l) => visiblePaths.has(l.sourcePath) && visiblePaths.has(l.targetPath),
		);
		renderAnchorDome(ctx, stars, visibleLinks, canvasWidth, canvasHeight, {
			locale: navigator.language ?? 'en',
			highlightedPath: hoveredPath,
		});
	}

	function syncCanvasSize(): void {
		if (!canvasEl || !canvasHostEl) return;
		const rect = canvasHostEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;
		dpr = Math.max(1, window.devicePixelRatio || 1);
		canvasEl.width = Math.max(1, Math.floor(canvasWidth * dpr));
		canvasEl.height = Math.max(1, Math.floor(canvasHeight * dpr));
		recomputeStars();
		paint();
	}

	function recomputeStars(): void {
		if (filteredRows.length === 0 || canvasWidth === 0 || canvasHeight === 0) {
			stars = [];
			return;
		}
		const layout = computeDomeLayout(canvasWidth, canvasHeight);
		stars = computeStarPositions(filteredRows, layout.centerX, layout.centerY, layout.radius);
	}

	// ── Data load ─────────────────────────────────────────────────

	async function loadLayoutAndLinks(): Promise<void> {
		try {
			rows = await invoke<LayoutCacheRow[]>('sight_v6_get_layout');
			recomputeStars();
			if (rows.length > 0) {
				const paths = rows.map((r) => r.notePath);
				links = await invoke<LinkEdge[]>('sight_v6_get_link_set_for_notes', { paths });
			}
			paint();
		} catch (err) {
			console.error('[Sight v6] loadLayoutAndLinks failed:', err);
		}
	}

	async function startWarmCache(): Promise<void> {
		try {
			await invoke<number>('sight_v6_warm_cache');
		} catch (err) {
			console.error('[Sight v6] warm_cache failed:', err);
		}
	}

	// ── Pointer events ────────────────────────────────────────────

	function pointerToCanvas(ev: { clientX: number; clientY: number }): { x: number; y: number } | null {
		if (!canvasEl) return null;
		const rect = canvasEl.getBoundingClientRect();
		return { x: ev.clientX - rect.left, y: ev.clientY - rect.top };
	}

	function handlePointerMove(ev: PointerEvent): void {
		const pt = pointerToCanvas(ev);
		if (!pt) return;
		const hit = starHitTest(stars, pt.x, pt.y);
		if (hit !== hoveredPath) {
			hoveredPath = hit;
			paint();
		}
	}

	function handlePointerLeave(): void {
		if (hoveredPath !== null) {
			hoveredPath = null;
			paint();
		}
	}

	function handleClick(ev: MouseEvent): void {
		const pt = pointerToCanvas(ev);
		if (!pt) return;
		const hit = starHitTest(stars, pt.x, pt.y);
		if (!hit) return;
		const row = rows.find((r) => r.notePath === hit);
		if (row && row.libraryName) {
			onOpenNote(row.notePath, row.libraryName);
		}
	}

	// ── §A.10 facet handlers ──────────────────────────────────────

	function handleFacetToggle(facet: FacetId, categoryId: string): void {
		filters = toggleFilter(filters, facet, categoryId);
	}

	function handleSidebarExpandToggle(): void {
		sidebarExpanded = !sidebarExpanded;
	}

	// ── Lifecycle ─────────────────────────────────────────────────

	onMount(async () => {
		await backfillProgress.start();
		syncCanvasSize();
		if (canvasHostEl) {
			resizeObserver = new ResizeObserver(() => syncCanvasSize());
			resizeObserver.observe(canvasHostEl);
		}
		startWarmCache();
	});

	onDestroy(() => {
		backfillProgress.stop();
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// React to backfill render-ready: load layout once tier 1 done.
	$effect(() => {
		if (backfillProgress.renderReady) {
			untrack(() => loadLayoutAndLinks());
		}
	});

	// Repaint on geometry change.
	$effect(() => {
		void canvasWidth;
		void canvasHeight;
		void dpr;
		untrack(() => paint());
	});

	// §A.10 — repaint AND recompute star positions when the filtered
	// row set changes (filter toggled or sidebar expand/collapse
	// changes layout). Stars are positioned over filteredRows so
	// the dome re-sizes the visible point cloud.
	$effect(() => {
		void filteredRows;
		untrack(() => {
			recomputeStars();
			paint();
		});
	});

	// Repaint on sidebar expand/collapse (changes canvas-host width).
	// Use a microtask so the layout stabilizes before syncCanvasSize.
	$effect(() => {
		void sidebarExpanded;
		queueMicrotask(() => {
			untrack(() => syncCanvasSize());
		});
	});
</script>

<div class="sight-v6-root">
	<div class="sight-v6-header">
		<span class="sight-v6-title">Constellation Sight</span>
		<span class="sight-v6-subtitle">v6.0 — anchor dome + facets (Phase 1)</span>
	</div>

	<div class="sight-v6-body">
		<FacetSidebar
			{facets}
			{filters}
			expanded={sidebarExpanded}
			onToggle={handleFacetToggle}
			onExpandToggle={handleSidebarExpandToggle}
		/>

		<div bind:this={canvasHostEl} class="sight-v6-canvas-host">
			<canvas
				bind:this={canvasEl}
				class="sight-v6-canvas"
				class:has-hover={hoveredPath !== null}
				onpointermove={handlePointerMove}
				onpointerleave={handlePointerLeave}
				onclick={handleClick}
			></canvas>
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
			{#if hoveredPath}
				<div class="sight-v6-hover-info">
					{hoveredPath}
				</div>
			{/if}
		</div>
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

	.sight-v6-body {
		flex: 1 1 auto;
		display: flex;
		flex-direction: row;
		min-height: 0;
		overflow: hidden;
	}

	.sight-v6-canvas-host {
		flex: 1 1 auto;
		position: relative;
		overflow: hidden;
		min-width: 0;
	}

	.sight-v6-canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		display: block;
		cursor: default;
	}

	.sight-v6-canvas.has-hover {
		cursor: pointer;
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

	.sight-v6-hover-info {
		position: absolute;
		left: 16px;
		bottom: 16px;
		font-size: 11px;
		color: #a0aabe;
		padding: 4px 10px;
		background: rgba(13, 19, 34, 0.92);
		border: 1px solid #2a3245;
		border-radius: 4px;
		pointer-events: none;
		max-width: 60%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>

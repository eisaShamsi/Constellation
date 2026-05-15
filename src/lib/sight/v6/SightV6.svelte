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
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import { backfillProgress } from './backfillProgress.svelte';
	import {
		renderAnchorDome,
		computeStarPositions,
		computeDomeLayout,
		starHitTest,
		type DomeLayout,
	} from './anchor';
	import {
		emptyFilters,
		applyFilters,
		computeFacetCounts,
		toggleFilter,
		type FacetFilters,
	} from './facets';
	import FacetSidebar from './facetSidebar.svelte';
	import Tour from './tour.svelte';
	import MiniDome from './MiniDome.svelte';
	import type { LayoutCacheRow, LinkEdge, StarDerived, FacetId, MiniDomeChannel } from './types';

	let { onOpenNote = (_path: string, _libraryName: string) => {} }: {
		onOpenNote?: (path: string, libraryName: string) => void;
	} = $props();

	// ── Canvas state ───────────────────────────────────────────────
	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let canvasHostEl = $state<HTMLDivElement | null>(null);
	let canvasWidth = $state(0);
	let canvasHeight = $state(0);
	let dpr = $state(1);

	// 2026-05-14 §A.14 fix-9 (Boss-test cycle 2 feature ask): zoom + pan.
	// Eisa: "If we enabled zoom-in/out, it would be more helpful to get
	// closer to the stars." Wheel = zoom-toward-cursor; left-drag = pan.
	// At dense centroids the user can zoom in to inspect individual
	// stars even when the unzoomed view shows brightened texture.
	let zoomScale = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	const ZOOM_MIN = 0.5;
	// 2026-05-14 §A.14 fix-16: ZOOM_MAX bumped 8 → 24 per Eisa's request
	// "I want to zoom in further, closer to the nodes, 3 times as much."
	// At 24× zoom, baseline node renders at 7.5 px screen radius (15 px
	// diameter — well past the spec'd 5 px). Aliasing artifacts from
	// sub-pixel rendering disappear; nodes render as crisp anti-aliased
	// circles. ZOOM_MAX_FOR_SIZING in anchor.ts stays at 8 so the world-
	// coord radius math (= 2.5 / 8 = 0.3125) is unchanged — bump only
	// affects how far the user can wheel in.
	const ZOOM_MAX = 24;
	let dragState: { startSx: number; startSy: number; startPanX: number; startPanY: number; moved: boolean } | null = null;
	const DRAG_THRESHOLD = 4; // px before pointermove counts as drag, not click

	// ── Data state ─────────────────────────────────────────────────
	let rows = $state<LayoutCacheRow[]>([]);
	let links = $state<LinkEdge[]>([]);
	let stars = $state<StarDerived[]>([]);
	let hoveredPath = $state<string | null>(null);

	// ── §A.10 facet state ──────────────────────────────────────────
	let filters = $state<FacetFilters>(emptyFilters());
	let sidebarExpanded = $state(false);

	// ── §A.11 first-boot tour state ────────────────────────────────
	// Snapshot at mount: if tourSeen is false/undefined, show the
	// 4-step orientation overlay. Updates persist via saveSettings.
	let tourVisible = $state(false);

	// ── §B.1 mini-domes diagnostics visibility ─────────────────────
	// Default-simple per Concept Paper §6: mini-domes hidden on
	// every Sight open. Cmd-D / Ctrl-D toggles visibility within
	// the session. Pro mode (§B.10) will read appSettings.sight.proMode
	// and override the default-hidden initial state.
	let diagnosticsVisible = $state(false);
	const MINI_DOME_CHANNELS: MiniDomeChannel[] = ['confidence', 'stage', 'acts', 'provenance'];

	// Anchor layout snapshot for mini-domes — they need it to scale
	// star positions from anchor world coords to mini canvas coords.
	const anchorLayout: DomeLayout = $derived(
		computeDomeLayout(canvasWidth, canvasHeight),
	);

	function dismissTour(): void {
		tourVisible = false;
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, tourSeen: true },
		}));
		saveSettings();
	}

	// Filtered row set + recomputed facet counts (Hearst preview).
	const filteredRows = $derived(applyFilters(rows, filters));
	const facets = $derived(computeFacetCounts(rows, filters));

	let resizeObserver: ResizeObserver | null = null;

	// ── Render ────────────────────────────────────────────────────

	function paint(): void {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		// §A.14 fix-9: combined transform = DPR × zoomScale + pan offset.
		// Render math runs in CSS pixels for the unzoomed canvas; the
		// zoom+pan + DPR factors compose into setTransform.
		const sx = dpr * zoomScale;
		const tx = dpr * panX;
		const ty = dpr * panY;
		ctx.setTransform(sx, 0, 0, sx, tx, ty);
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
			zoomScale: zoomScale,
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
		// Screen → canvas-CSS-pixel.
		const sx = ev.clientX - rect.left;
		const sy = ev.clientY - rect.top;
		// §A.14 fix-9: convert screen → world (unzoomed) coordinates.
		// The renderer draws in world space transformed by zoomScale+pan;
		// hit-test must invert the transform to compare against world-
		// coord star positions.
		return { x: (sx - panX) / zoomScale, y: (sy - panY) / zoomScale };
	}

	// 2026-05-14 §A.14 fix-4 (Boss-test #4 feedback): hover tooltip
	// showed raw notePath ("Research/Botany/Apple Tree Fruit.md").
	// Extract just the filename-without-.md as the human-readable
	// title; show the full path as a secondary line for disambiguation
	// when there are folders in the path.
	function noteTitle(path: string): string {
		const last = path.split('/').pop() || path;
		return last.replace(/\.md$/i, '');
	}

	function handlePointerMove(ev: PointerEvent): void {
		// §A.14 fix-9: drag-pan support. When mouse button held + moved
		// past DRAG_THRESHOLD, treat as pan rather than hover.
		if (dragState && (ev.buttons & 1) === 1) {
			const dx = ev.clientX - dragState.startSx;
			const dy = ev.clientY - dragState.startSy;
			if (!dragState.moved && Math.hypot(dx, dy) > DRAG_THRESHOLD) {
				dragState.moved = true;
			}
			if (dragState.moved) {
				panX = dragState.startPanX + dx;
				panY = dragState.startPanY + dy;
				paint();
				return;
			}
		}
		const pt = pointerToCanvas(ev);
		if (!pt) return;
		// Hit-test tolerance is in screen px; divide by zoom for world px.
		const hit = starHitTest(stars, pt.x, pt.y, 9 / zoomScale);
		if (hit !== hoveredPath) {
			hoveredPath = hit;
			paint();
		}
	}

	function handlePointerDown(ev: PointerEvent): void {
		// §A.14 fix-9: start of potential drag-pan.
		dragState = {
			startSx: ev.clientX,
			startSy: ev.clientY,
			startPanX: panX,
			startPanY: panY,
			moved: false,
		};
	}

	function handlePointerUp(): void {
		dragState = null;
	}

	function handlePointerLeave(): void {
		dragState = null;
		if (hoveredPath !== null) {
			hoveredPath = null;
			paint();
		}
	}

	function handleClick(ev: MouseEvent): void {
		// §A.14 fix-9: ignore clicks that were actually drags.
		if (dragState?.moved) {
			dragState = null;
			return;
		}
		const pt = pointerToCanvas(ev);
		if (!pt) return;
		const hit = starHitTest(stars, pt.x, pt.y, 9 / zoomScale);
		if (!hit) return;
		const row = rows.find((r) => r.notePath === hit);
		if (row && row.libraryName) {
			onOpenNote(row.notePath, row.libraryName);
		}
	}

	// §A.14 fix-9: mouse-wheel zoom-toward-cursor.
	function handleWheel(ev: WheelEvent): void {
		ev.preventDefault();
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const sx = ev.clientX - rect.left;
		const sy = ev.clientY - rect.top;
		// World point under cursor BEFORE zoom.
		const wx = (sx - panX) / zoomScale;
		const wy = (sy - panY) / zoomScale;
		// Zoom: positive deltaY (scroll down) zooms OUT; smooth ratio.
		const factor = ev.deltaY < 0 ? 1.15 : 1 / 1.15;
		const nextScale = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, zoomScale * factor));
		if (nextScale === zoomScale) return;
		// Adjust pan so the world point under cursor stays under cursor.
		panX = sx - wx * nextScale;
		panY = sy - wy * nextScale;
		zoomScale = nextScale;
		paint();
	}

	// §A.14 fix-9: keyboard escape resets zoom + pan as a last-resort
	// "I lost the dome" recovery. Esc still also closes Sight v6 via
	// +layout.svelte's escape handler — that handler runs FIRST when
	// nothing-zoomed; this resets when zoomed.
	// §B.1: also handles Cmd-D / Ctrl-D for mini-domes diagnostics
	// toggle. Listed in Concept Paper §5 gesture grammar.
	function handleKey(ev: KeyboardEvent): void {
		if (ev.key === '0' && (ev.ctrlKey || ev.metaKey)) {
			ev.preventDefault();
			zoomScale = 1;
			panX = 0;
			panY = 0;
			paint();
		} else if ((ev.key === 'd' || ev.key === 'D') && (ev.ctrlKey || ev.metaKey) && !ev.shiftKey) {
			// Cmd-D / Ctrl-D — toggle mini-domes diagnostics visibility.
			// Excludes Shift to avoid colliding with §B.10's Cmd-Shift-D
			// for Pro mode persistent toggle.
			ev.preventDefault();
			diagnosticsVisible = !diagnosticsVisible;
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
		// 2026-05-14 §A.14 fix-11 (Boss-test cycle 3.1 zoom dead): the
		// Svelte template `onwheel={handleWheel}` did not fire in the
		// release build — likely Tauri WebView2 + Svelte 5 wheel-event
		// quirk, possibly passive-by-default. Switch to imperative
		// addEventListener with explicit { passive: false } so
		// preventDefault() works and the handler is guaranteed to run.
		// The template binding is removed to avoid potential double-
		// firing on some platforms.
		if (canvasEl) {
			canvasEl.addEventListener('wheel', handleWheel, { passive: false });
		}
		startWarmCache();
		// §A.11 — fire the tour if user hasn't seen it yet. Snapshot
		// (no $store subscription needed for the show-once gate).
		const settingsSnapshot = get(appSettings);
		if (!settingsSnapshot.sight?.tourSeen) {
			tourVisible = true;
		}
	});

	onDestroy(() => {
		backfillProgress.stop();
		resizeObserver?.disconnect();
		resizeObserver = null;
		// §A.14 fix-11 — match the addEventListener in onMount.
		if (canvasEl) {
			canvasEl.removeEventListener('wheel', handleWheel);
		}
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

	// §B.6 — repaint anchor when hoveredPath changes from any source.
	// Forward direction (anchor pointermove → hoveredPath) already calls
	// paint() explicitly inside handlePointerMove. Reverse direction
	// (mini-dome onHover → hoveredPath) needs this effect to redraw the
	// anchor's gold highlight ring. paint() is idempotent so the
	// occasional double-paint on forward-direction hover is harmless.
	$effect(() => {
		void hoveredPath;
		untrack(() => paint());
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

		<div bind:this={canvasHostEl} class="sight-v6-canvas-host" class:has-minis={diagnosticsVisible}>
			<canvas
				bind:this={canvasEl}
				class="sight-v6-canvas"
				class:has-hover={hoveredPath !== null}
				class:is-dragging={dragState?.moved}
				onpointermove={handlePointerMove}
				onpointerdown={handlePointerDown}
				onpointerup={handlePointerUp}
				onpointerleave={handlePointerLeave}
				onclick={handleClick}
				onkeydown={handleKey}
				tabindex="0"
			></canvas>
			<!-- §A.14 fix-11: zoom indicator. Renders ONLY when zoom != 1.0
			     so it doesn't clutter the default view. If wheel fires and
			     state updates, this badge appears + reflects zoom level even
			     if the render pipeline is broken — clean diagnostic for
			     "did wheel fire" vs "did render apply". Cmd-0 hides it
			     by resetting zoom to 1. -->
			{#if zoomScale !== 1 || panX !== 0 || panY !== 0}
				<div class="sight-v6-zoom-badge">
					zoom: {zoomScale.toFixed(2)}× · pan: {Math.round(panX)},{Math.round(panY)} · Ctrl-0 reset
				</div>
			{/if}
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
					<span class="sight-v6-hover-title">{noteTitle(hoveredPath)}</span>
					{#if hoveredPath !== noteTitle(hoveredPath) + '.md' && hoveredPath !== noteTitle(hoveredPath)}
						<span class="sight-v6-hover-path">{hoveredPath}</span>
					{/if}
				</div>
			{/if}
			{#if tourVisible}
				<Tour onComplete={dismissTour} />
			{/if}
		</div>

		{#if diagnosticsVisible}
			<!-- §B.1: 2×2 mini-domes grid. Skeleton renders chrome
			     (background + stratum bands + channel title) only;
			     channel-specific star renderers fill in §B.2–§B.5. -->
			<div class="sight-v6-minis-grid">
				{#each MINI_DOME_CHANNELS as channel (channel)}
					<div class="sight-v6-mini-cell">
						<MiniDome
							{channel}
							stars={filteredRows.length > 0 ? stars : []}
							{anchorLayout}
							highlightedPath={hoveredPath}
							onHover={(path) => { hoveredPath = path; }}
						/>
					</div>
				{/each}
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
	/* §B.1: when mini-domes visible, anchor compresses to ~60%
	   of remaining horizontal space; minis grid takes ~40%. */
	.sight-v6-canvas-host.has-minis {
		flex: 0 1 60%;
	}

	.sight-v6-minis-grid {
		flex: 0 1 40%;
		display: grid;
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
		gap: 8px;
		padding: 8px;
		min-width: 0;
		min-height: 0;
	}
	.sight-v6-mini-cell {
		position: relative;
		min-width: 0;
		min-height: 0;
		border: 1px solid #1a1f2e;
		border-radius: 4px;
		overflow: hidden;
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
	.sight-v6-canvas.is-dragging {
		cursor: grabbing;
	}
	.sight-v6-canvas:focus {
		outline: none;
	}

	.sight-v6-zoom-badge {
		position: absolute;
		right: 16px;
		top: 16px;
		font-size: 11px;
		color: #7dd3fc;
		padding: 4px 10px;
		background: rgba(13, 19, 34, 0.92);
		border: 1px solid #3b5998;
		border-radius: 4px;
		pointer-events: none;
		font-variant-numeric: tabular-nums;
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
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 6px 12px;
		background: rgba(13, 19, 34, 0.94);
		border: 1px solid #2a3245;
		border-radius: 4px;
		pointer-events: none;
		max-width: 60%;
	}
	.sight-v6-hover-title {
		font-size: 12px;
		font-weight: 500;
		color: #e8ebf2;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.sight-v6-hover-path {
		font-size: 10px;
		color: #5a6275;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>

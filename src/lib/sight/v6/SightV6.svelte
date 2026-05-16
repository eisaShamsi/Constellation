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
		filtersEmpty,
		confidenceLevelOf,
		provenanceSectorOf,
		type FacetFilters,
	} from './facets';
	import { bandForRawStratum } from './dome';
	import FacetSidebar from './facetSidebar.svelte';
	import Tour from './tour.svelte';
	import MiniDome from './MiniDome.svelte';
	import RegisterChip from './registerChip.svelte';
	import type { LayoutCacheRow, LinkEdge, StarDerived, FacetId, MiniDomeChannel, SlotChannel } from './types';

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
	// the session. Extended view (§B.10) reads appSettings.sight.extended
	// and overrides the default-hidden initial state.
	let diagnosticsVisible = $state(false);
	const MINI_DOME_CHANNELS: MiniDomeChannel[] = ['confidence', 'stage', 'acts', 'provenance'];

	// §B.6-fix-3 — dome-swap state. `primaryChannel` is whichever of
	// the 5 surfaces currently occupies the large primary canvas slot.
	// Default 'anchor' = the universe-baseline view (renderAnchorDome
	// with full chrome). Click any mini → promote it to primary;
	// click the demoted anchor (now in a mini slot) → swap back.
	// Per Eisa cycle-2: "click on every mini-dome to enlarge it to
	// the same size as the anchor dome ... the user could switch
	// between all the domes (including the main one) to check their
	// details."
	const ALL_SLOTS: SlotChannel[] = ['anchor', 'confidence', 'stage', 'acts', 'provenance'];
	let primaryChannel = $state<SlotChannel>('anchor');

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

	// §B.7-fix-2 (Eisa cycle-2 ghost mode): set of notePaths that pass
	// the current filter. null when no filter active (skip the ghost
	// pathway entirely → all stars render at full encoding). Renderers
	// (anchor + mini channel renderers) check this set per-star and
	// fade non-members to GHOST_ALPHA (0.15).
	const matchedPaths = $derived(
		filtersEmpty(filters) ? null : new Set(filteredRows.map((r) => r.notePath)),
	);

	// §B.9 (2026-05-16) — density-aggregation mode. Active when the
	// matched (or all-when-no-filter) star count exceeds
	// `appSettings.sight.hexBinThreshold` (default 5000 per Concept
	// Paper §3.4). Above the threshold, channel renderers switch
	// per-star alpha from full encoding to a low value (~0.3) so
	// overlapping stars additive-blend into a perceptual density
	// gradient — the dense regions read as "more stars here" without
	// every dot needing to be individually visible.
	//
	// This is a lightweight stand-in for the full d3-hexbin
	// aggregation the Concept Paper specifies for v6.1. Real
	// hex-binning (compute bins → render hexagons with dominant-value
	// fill + count badge) lands as v6.2 polish (PJ-NNN allocated
	// post-ship). For Eisa's 7,645-note universe, the lightweight
	// density mode is sufficient — discrete dots remain visible
	// enough to hit-test, but dense clusters no longer look like
	// solid blobs of cream.
	const densityMode = $derived.by(() => {
		const threshold = $appSettings.sight?.hexBinThreshold ?? 5000;
		const count = matchedPaths === null ? rows.length : matchedPaths.size;
		return count > threshold;
	});

	// §B.7-fix-3 (Eisa cycle-3 ask): when a star is hovered anywhere in
	// Sight, derive its per-facet category values so the FacetSidebar
	// can highlight the matching chips. Reverse direction of the
	// existing forward-link (click chip → filter dome). Six facets:
	// folder / library / stratum / confidence / stage / provenance.
	// null when no star hovered → sidebar reverts to no-highlight state.
	const hoveredFacetValues = $derived.by(() => {
		if (!hoveredPath) return null;
		const row = rows.find((r) => r.notePath === hoveredPath);
		if (!row) return null;
		return {
			folder: row.folderPath ?? undefined,
			library: row.libraryName ?? undefined,
			stratum: bandForRawStratum(row.stratum),
			confidence: confidenceLevelOf(row),
			stage: row.stage ?? undefined,
			provenance: provenanceSectorOf(row.sourcesPrimary),
		} as Partial<Record<FacetId, string>>;
	});

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
			matchedPaths,
			densityMode,
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
		// §B.7-fix-2 (Eisa cycle-2 ghost mode): compute star positions
		// from the FULL universe (rows), not the filtered subset. The
		// filter only controls which stars render at full opacity vs
		// faded ghost (via matchedPaths). This way the user can hover
		// AND Shift+click on a non-matched ghost star to add its category
		// to the filter — multi-select within a facet works directly
		// from the dome instead of requiring sidebar chip interaction.
		if (rows.length === 0 || canvasWidth === 0 || canvasHeight === 0) {
			stars = [];
			return;
		}
		const layout = computeDomeLayout(canvasWidth, canvasHeight);
		stars = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
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
		// §B.7-fix-1 (Eisa cycle-1 Stage 4): Shift+click on the anchor
		// dome is a no-op. Anchor has no channel-specific category
		// (it's the universe-baseline view), so cross-filter doesn't
		// apply here. Without this guard, Shift+click fell through to
		// onOpenNote and opened the note instead of being silent — the
		// guard mirrors MiniDome's handling for 'anchor' / 'acts' channels.
		if (ev.shiftKey) return;
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
			// Session-only — does NOT touch persistent sight.extended.
			// Excludes Shift to avoid colliding with Cmd-Shift-D below.
			ev.preventDefault();
			diagnosticsVisible = !diagnosticsVisible;
		} else if ((ev.key === 'd' || ev.key === 'D') && (ev.ctrlKey || ev.metaKey) && ev.shiftKey) {
			// §B.10 — Cmd-Shift-D / Ctrl-Shift-D — toggle the extended-view
			// setting PERSISTENTLY. Extended view = minis default-visible
			// on every Sight open (instead of default-hidden). State
			// stored in appSettings.sight.extended; survives across
			// sessions via saveSettings. Also flips diagnosticsVisible
			// to match the new value, so the toggle has immediate effect
			// in the current session.
			// §B.10-fix-1 (Eisa cycle-1, 2026-05-16): field name renamed
			// `proMode` → `extended` per Eisa: "Pro" overpromised, the
			// feature only controls default view extent. Migration in
			// applyParsedSettings carries existing users' values forward.
			ev.preventDefault();
			const newExtended = !($appSettings.sight?.extended ?? false);
			appSettings.update((s) => ({
				...s,
				sight: { ...s.sight, extended: newExtended },
			}));
			saveSettings();
			diagnosticsVisible = newExtended;
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
		// §A.11 — fire the tour if user hasn't seen it yet. Snapshot
		// (no $store subscription needed for the show-once gate).
		// §B.10 — also read sight.extended here: if extended view was
		// enabled in a previous session (via Cmd-Shift-D), default
		// diagnosticsVisible=true so the minis are shown on this open.
		// Per-session Cmd-D toggle (handleKey case below) still works
		// to temporarily hide the minis without touching the persistent
		// extended setting.
		const settingsSnapshot = get(appSettings);
		if (!settingsSnapshot.sight?.tourSeen) {
			tourVisible = true;
		}
		if (settingsSnapshot.sight?.extended) {
			diagnosticsVisible = true;
		}
	});

	onDestroy(() => {
		backfillProgress.stop();
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// §B.6-fix-3 — wheel listener moved out of onMount and into a
	// $effect keyed on canvasEl, because the anchor canvas now mounts/
	// unmounts based on primaryChannel (dome-swap). When primaryChannel
	// flips between 'anchor' and a mini channel, canvasEl rebinds; the
	// effect re-attaches the listener to the new element and the cleanup
	// detaches from the old. Replaces the §A.14 fix-11 onMount block.
	// Imperative addEventListener kept (NOT Svelte template binding) per
	// the original fix-11 lesson — Tauri WebView2 + Svelte 5 onwheel
	// silent-fails in release builds.
	$effect(() => {
		const el = canvasEl;
		if (!el) return;
		el.addEventListener('wheel', handleWheel, { passive: false });
		return () => {
			el.removeEventListener('wheel', handleWheel);
		};
	});

	// §B.6-fix-3 — re-sync canvas dimensions after the primary slot
	// swaps. Mounting a fresh canvas via {#if} doesn't fire the
	// ResizeObserver (host size unchanged), so syncCanvasSize must be
	// called explicitly when canvasEl rebinds.
	$effect(() => {
		void canvasEl;
		void primaryChannel;
		untrack(() => {
			if (canvasEl) syncCanvasSize();
		});
	});

	// §B.6-fix-3 — promoted-mini click → open note. Looks up the row
	// to find libraryName, then dispatches the parent's onOpenNote.
	function handlePromotedOpenNote(notePath: string): void {
		const row = rows.find((r) => r.notePath === notePath);
		if (row && row.libraryName) {
			onOpenNote(notePath, row.libraryName);
		}
	}

	// §B.6-fix-3 — swap a different channel into the primary slot.
	// Resets zoom + pan (the previous slot's transform doesn't apply
	// to the new primary). hoveredPath stays — linked brushing is
	// continuous across swaps.
	function handlePromote(slot: SlotChannel): void {
		if (slot === primaryChannel) return;
		primaryChannel = slot;
		zoomScale = 1;
		panX = 0;
		panY = 0;
	}

	// §B.6-fix-4c (Eisa cycle-3 Stage 7): explicit "Reset View" returns
	// to the default layout — anchor in primary slot + zoom 1 + no pan.
	// Header button exposes this when there's something to reset.
	function handleResetView(): void {
		primaryChannel = 'anchor';
		zoomScale = 1;
		panX = 0;
		panY = 0;
	}

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

	// §A.10 — repaint AND recompute star positions when the row set
	// changes (data load or refresh).
	// §B.7-fix-2 — recompute is now driven by `rows` (full universe),
	// not `filteredRows`. Filter changes only need a repaint, not a
	// star-position recompute, because stars are positioned over the
	// full universe and ghost-rendering happens per-star at paint time.
	$effect(() => {
		void rows;
		untrack(() => {
			recomputeStars();
			paint();
		});
	});

	// Repaint when filter set changes (matchedPaths updates, ghost
	// rendering needs to refresh).
	$effect(() => {
		void filteredRows;
		untrack(() => paint());
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
		<span class="sight-v6-subtitle">v6.2 — Registers (Phase 3)</span>
		<!-- §C.1 — Register chip. Sits between the subtitle and the
		     EXTENDED badge per Concept Paper §2.5. Default state shows
		     only the active register (collapsed); click to expand the
		     full 7-chip row. Click any chip → switches activeRegister
		     (writes to appSettings.sight.activeRegister via the canonical
		     update+saveSettings pattern; partial-ships §C.8 persistence).
		     Hover any chip → English secondary label tooltip per §11
		     invariant. v1-preview registers (Dignāga / Suhrawardi
		     Ishrāqī / Mohist sān biǎo) carry a "preview" badge per §4.2. -->
		<RegisterChip />
		<!-- §B.10 — small "EXTENDED" indicator when the persistent
		     extended-view setting is on (Cmd-Shift-D toggles). Per
		     Concept Paper §11 invariant 9 (no persistent toggle bars),
		     this is informational only — clicking it doesn't toggle.
		     Cmd-Shift-D is the toggle.
		     §B.10-fix-1 (Eisa cycle-1, 2026-05-16): full rename —
		     user-facing label "Pro" → "Extended" AND internal field
		     name `proMode` → `extended` per Eisa: "even the internal
		     appSettings.sight.proMode field name has to change."
		     Settings migration in applyParsedSettings carries forward
		     existing users' values. -->
		{#if $appSettings.sight?.extended}
			<span class="sight-v6-pro-badge" title="Extended view is ON — minis default-visible on every Sight open. Cmd-Shift-D to toggle.">
				EXTENDED
			</span>
		{/if}
		<!-- §B.7-fix-1 (Eisa cycle-1 Stage 1 ask: "we need to add a
		     count of affected notes when shift-clicking"): when any
		     facet filter is active, show the filtered/total count in
		     the header so the user sees the immediate impact of their
		     Shift+click without needing to inspect the sidebar. -->
		{#if !filtersEmpty(filters)}
			<span class="sight-v6-filter-count" title="Filtered notes / total notes">
				{filteredRows.length.toLocaleString()} / {rows.length.toLocaleString()} notes
			</span>
		{/if}
		<!-- §B.6-fix-4c — Reset View button. Visible when the layout
		     has been changed away from the default (anchor primary at
		     zoom 1.0). Clicking returns to default in one tap, no need
		     to manually swap back through the demoted-anchor mini. -->
		{#if primaryChannel !== 'anchor' || zoomScale !== 1 || panX !== 0 || panY !== 0}
			<button class="sight-v6-reset-btn" onclick={handleResetView} title="Return to anchor dome at default zoom (Reset View)">
				Reset View
			</button>
		{/if}
	</div>

	<div class="sight-v6-body">
		<FacetSidebar
			{facets}
			{filters}
			expanded={sidebarExpanded}
			onToggle={handleFacetToggle}
			onExpandToggle={handleSidebarExpandToggle}
			{hoveredFacetValues}
		/>

		<div bind:this={canvasHostEl} class="sight-v6-canvas-host" class:has-minis={diagnosticsVisible}>
			{#if primaryChannel === 'anchor'}
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
			{:else}
				<!-- §B.6-fix-3 — promoted mini in primary slot. Fills the
				     canvas-host space. Uses MiniDome with compact=false so
				     the channel renders with bigger dots (radius 3 vs 0.75)
				     and a click on a star opens it in the editor (callback
				     wired below to handlePromotedOpenNote). The original
				     anchor canvas is unmounted; its zoom/pan state resets
				     when the user swaps anchor back into primary. -->
				<div class="sight-v6-promoted-host">
					<MiniDome
						channel={primaryChannel}
						stars={stars}
						{anchorLayout}
						highlightedPath={hoveredPath}
						onHover={(path) => { hoveredPath = path; }}
						compact={false}
						onOpenNote={handlePromotedOpenNote}
						onFacetFilter={handleFacetToggle}
						{matchedPaths}
					/>
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
			<!-- §B.1: 2×2 mini-domes grid. Channel-specific renderers per §B.2-§B.5.
			     §B.6-fix-3: iterates ALL_SLOTS (anchor + 4 channels) skipping the
			     primary, so the demoted anchor takes the slot vacated by whichever
			     channel was promoted. Click any mini → handlePromote swaps it into
			     the primary slot. -->
			<div class="sight-v6-minis-grid">
				{#each ALL_SLOTS as slot (slot)}
					{#if slot !== primaryChannel}
						<div class="sight-v6-mini-cell">
							<!-- §B.6-fix-6: onOpenNote also passed to compact
							     mini-grid instances per Eisa cycle-5: a star
							     click in any dome (mini or primary) opens
							     the note. Click on empty space promotes (per
							     fix-3); the mini's handleClick hit-tests
							     first, so star vs empty is unambiguous. -->
							<!-- §B.7 — onFacetFilter dispatched on Shift+click on a
							     star; reuses the existing handleFacetToggle handler
							     (same pipeline the facet sidebar uses), so the cross-
							     filter applies uniformly across all 5 surfaces via
							     filteredRows → recomputeStars → repaint. -->
							<MiniDome
								channel={slot}
								stars={stars}
								{anchorLayout}
								highlightedPath={hoveredPath}
								onHover={(path) => { hoveredPath = path; }}
								compact={true}
								onPromote={handlePromote}
								onOpenNote={handlePromotedOpenNote}
								onFacetFilter={handleFacetToggle}
								{matchedPaths}
							{densityMode}
							/>
						</div>
					{/if}
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

	/* §B.10 — Extended-view indicator (user-facing label "EXTENDED";
	   internal field also renamed to `extended` per §B.10-fix-1).
	   Tiny gold-on-dark tag immediately right of the subtitle. Not a
	   button — Cmd-Shift-D is the toggle. CSS class name `.sight-v6-
	   pro-badge` kept (DOM-only identifier, no architectural meaning). */
	.sight-v6-pro-badge {
		display: inline-flex;
		align-items: center;
		padding: 2px 7px;
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.6px;
		color: #fbbf24;
		background: rgba(251, 191, 36, 0.10);
		border: 1px solid rgba(251, 191, 36, 0.45);
		border-radius: 3px;
		cursor: help;
		font-variant: small-caps;
	}

	/* §B.7-fix-1 — filter affected-count badge. Shows X/Y notes when
	   any facet filter is active. Subtle but readable. Sits before the
	   Reset View button via margin-left:auto on the count (which pushes
	   it right-aligned with the button to its right when both visible). */
	.sight-v6-filter-count {
		margin-left: auto;
		padding: 4px 10px;
		font-size: 11px;
		color: #fbbf24;
		font-variant-numeric: tabular-nums;
		background: rgba(58, 67, 90, 0.35);
		border: 1px solid rgba(251, 191, 36, 0.4);
		border-radius: 4px;
	}

	/* §B.6-fix-4c — Reset View button. Lives at the right edge of the
	   header strip via margin-left:auto. Subtle by default; visible
	   on hover. Designed not to compete with the title.
	   §B.7-fix-1: when filter-count badge is also visible, the badge
	   takes the margin-left:auto and the button sits next to it via
	   a smaller fixed margin. */
	.sight-v6-reset-btn {
		margin-left: auto;
		padding: 4px 12px;
		font-size: 11px;
		font-family: inherit;
		color: #e8ebf2;
		background: rgba(58, 67, 90, 0.55);
		border: 1px solid #3b5998;
		border-radius: 4px;
		cursor: pointer;
		transition: background 0.12s ease;
	}
	.sight-v6-filter-count + .sight-v6-reset-btn {
		margin-left: 8px;
	}
	.sight-v6-reset-btn:hover {
		background: rgba(74, 90, 130, 0.85);
	}
	.sight-v6-reset-btn:active {
		background: rgba(58, 67, 90, 0.85);
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

	/* §B.6-fix-3 — promoted mini wrapper. Fills the canvas-host the
	   same way the anchor canvas does (absolute inset 0). The MiniDome
	   inside has its own host div + ResizeObserver, so size changes
	   propagate naturally. */
	.sight-v6-promoted-host {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
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

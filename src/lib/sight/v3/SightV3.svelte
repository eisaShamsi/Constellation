<!--
    SightV3.svelte — the v3 star-chart Sight component.

    §1C: empty Pixi canvas + placeholder text.
    §1D: stars at MDS-embedded positions, sized by centrality;
        Lambert ↔ stereographic projection toggle.
    §1E (this commit): territories + faint connector lines at rest +
        hover/click/double-click + tooltip + side panel + Suwaidi
        warm-cream + gold palette. The Boss-test gate.

    §1E architecture — three Pixi containers on a single Stage:
      territoryContainer  — community polygons (cycled-pastel fill)
      edgeContainer       — faint connector lines (Suwaidi cream, low α)
      starContainer       — stars (Suwaidi cream, sized by centrality)
      focusOverlay        — brightened edges + outlined focus star,
                            redrawn ONLY on hover/click state changes.
    The first three layers redraw together on layout/projection changes.
    The focus overlay redraws on hover/click only — base stays static.

    Companion to: docs/Constellation-Sight-v3-Concept-Paper-v1.1.md.
    Plan ref:    lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-PLAN.md.
-->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Application, Container, Graphics, Text, TextStyle, Sprite, Texture } from 'pixi.js';
    import { libraryStats, appSettings, type SkyNode, type SkyLink } from '$lib/libraries/store';
    import { get } from 'svelte/store';
    import { fetchLayout, type LayoutPoint } from '$lib/sight/layout-cache';
    import { fetchDensityField, type DensityField } from '$lib/sight/density-cache';
    import { embedToScreen, type ProjectionMode } from '$lib/sight/projection';
    import { detectClusters, type ClusterInfo } from '$lib/graph/clusterEngine';
    import { communityTerritories, type Point2D } from '$lib/sight/community-territory';
    import {
        computeHealthReport,
        emptyHealthReport,
        type HealthReport,
    } from '$lib/sight/universe-health';
    import {
        monthArcSegments,
        pickMonth,
        gregorianMonthFromSegment,
        type CalendarSystem,
        type MonthSegment,
    } from '$lib/sight/calendar-rim';
    import { locale as i18nLocale } from '$lib/i18n';
    import {
        SUWAIDI_PALETTE,
        communityColorInt,
        SKY_BACKGROUND,
        CONNECTOR_LINE_COLOR,
    } from '$lib/sight/palette';
    import SightV3SidePanel from './SightV3SidePanel.svelte';
    import { t } from '$lib/i18n';

    // MIG-019 §2G — Polar layout helpers.  See docs/SIGHT-V3-VISUAL-SPEC.md §2.
    import {
        polarToCartesian,
        radiusFromCentrality,
        magnitudeSize,
        magnitudeAlpha,
        PALETTE as V3_PALETTE,
        DOME_RATIOS,
    } from './polar';
    import {
        type SightMode,
        type ModeContext,
        type ModeStats,
        type ModePosition,
        DEFAULT_MODE,
        resolveMode,
        positionForMode,
        buildModeStats,
        emptyModeStats,
    } from './modes';
    import {
        buildRegionLayout,
        azimuthInWedge,
        type RegionLayout,
        type RegionWedge,
    } from './regions';
    import { buildLibraryColorMap } from './library-colors';
    import { detectDir } from '$lib/utils';

    interface Props {
        nodes: SkyNode[];
        links: SkyLink[];
        /** MIG-019 §2E: search-hub match set. Lowercased note names.
         *  Null when no search is active; non-empty Set when query has matches.
         *  Matched stars flare (brighter + bigger); non-matched dim heavily;
         *  territories containing matches get a halo glow. */
        searchMatchIds?: Set<string> | null;
        /** §2G.3c: active Universe name shown above the dome, between
         *  the Universe Health metrics and the chart. */
        universeName?: string;
        onClose: () => void;
        onOpenNote: (path: string, libraryName: string) => void;
    }
    let { nodes, links, searchMatchIds = null, universeName = '', onClose, onOpenNote }: Props = $props();

    // ─── DOM + Pixi handles ──────────────────────────────────────────
    let canvasContainer: HTMLDivElement;
    let app: Application | null = null;
    /** MIG-019 §2B: Milky Way density wash. Sits between sky background
     *  and territoryContainer so stars + edges still render on top. */
    let milkyWayContainer: Container | null = null;
    /** MIG-019 §2C: Calendar rim. Outermost layer (above stars/focus) so
     *  rim labels + arc lines never get hidden by the chart contents. */
    let calendarRimContainer: Container | null = null;
    let territoryContainer: Container | null = null;
    let edgeContainer: Container | null = null;
    let starContainer: Container | null = null;
    let focusOverlay: Container | null = null;
    let placeholderText: Text | null = null;
    let resizeObserver: ResizeObserver | null = null;
    // §2G.3l: closeBtn $state ref + bind:this + $effect addEventListener
    // pattern ALL removed. Per Svelte 5 docs (and the closed issue
    // #10435), the canonical pattern is `<button onclick={fn}>` directly.
    // Six rounds of "defensive" wiring fought a non-bug.
    /** §2G.3g: parent of every chart layer that should scale + pan
     *  with the zoom controls. Placeholder text and HTML overlays
     *  are NOT inside this — they stay anchored to the window. */
    let chartContainer: Container | null = null;

    // ─── Data ────────────────────────────────────────────────────────
    let layoutPoints: LayoutPoint[] = $state([]);
    /** MIG-019 §2A+§2B redesign: 2D density field from TF-IDF
     *  (PJ-035 → Milky Way). 256×256 f32 grid; ~256 KB, input-size
     *  invariant. Rendered as a single Pixi Sprite. */
    let densityField = $state<DensityField | null>(null);
    /** Map note_path → its layout point (for fast lookup during edge draw). */
    let pathToPoint = new Map<string, LayoutPoint>();
    /** MIG-019 §2C: per-note creation date (epoch ms) for month filter. */
    let pathToCreatedAt = new Map<string, number>();
    /** MIG-019 §2C: cached month segments (one per ring × 12 months). */
    let monthSegments: MonthSegment[] = [];
    /** MIG-019 §2D: computed universe-health report (passed to side panel). */
    let healthReport = $state<HealthReport>(emptyHealthReport());
    /** Map note_path → community id. */
    let pathToCommunity = new Map<string, number>();
    /** Map note_path → its on-screen position (recomputed every redraw). */
    let pathToScreen = new Map<string, { x: number; y: number; r: number; baseAlpha: number }>();
    /** Community metadata from clusterEngine. */
    let clusters: ClusterInfo[] = [];
    /** Map community-id → cluster info for fast tooltip lookup. */
    let communityById = new Map<number, ClusterInfo>();
    /** Centrality rank map for tooltip display. */
    let centralityRank = new Map<string, number>();
    /** Map name (lowercased) → note_path, for joining skyNodes ↔ layoutPoints. */
    let nameToPath = new Map<string, string>();
    /** Map note_path → display title. */
    let pathToTitle = new Map<string, string>();
    /** Map note_path → library name (used by onOpenNote). */
    let pathToLibrary = new Map<string, string>();
    /** All graph edges in (path-A, path-B) form, deduped. The focus
     *  overlay walks this on hover/select to find incident edges. */
    let resolvedEdges: Array<{ a: string; b: string }> = [];

    // ─── MIG-019 §2G: Polar layout state ─────────────────────────────
    /** The active rim-axis mode. Default = 'regions' (spec §1.3).
     *  Persistence + toggle UI land in §2G.4 / §2G.5. */
    let currentMode = $state<SightMode>(resolveMode(
        ($appSettings as any)?.sight?.lastMode ?? null,
    ));
    /** Region (library) wedge layout — built once per fetch.
     *  Updated when libraryPaths or layoutPoints change. */
    let regionLayout = $state<RegionLayout | null>(null);
    /** The library path/name tuples passed into fetchLayout — kept on
     *  hand so we can rebuild the region layout if mode/data changes. */
    let libraryPathsCached: Array<[string, string]> = [];
    /** §2G.3e: in-wedge azimuth jitter [0, 2π) per note.
     *
     *  Originally populated from `atan2(embed_x, embed_y)` of the MDS
     *  layout. Problem: notes from the same MDS cluster (i.e., same
     *  community) get nearly-identical embed angles. After `azimuthInWedge`
     *  maps the angle into the wedge, all those stars stack on a single
     *  in-wedge azimuth → radial spoke per cluster (Eisa screenshot
     *  2026-05-07 evening).
     *
     *  Fix: derive a uniform per-note hash so notes spread evenly across
     *  the wedge regardless of MDS clustering. Deterministic per
     *  `note_path` so positions are stable across renders. */
    let pathToEmbedAngle = new Map<string, number>();
    /** §2G.3c: centrality rank (0 = most central → center; 1 = least
     *  central → rim). Used for radial position. Real centrality data
     *  is heavily skewed near 0, so ranking via centrality_norm directly
     *  packs >90 % of stars at the rim. Ranking by percentile produces
     *  a uniform radial distribution. */
    let pathToCentralityRank = new Map<string, number>();
    /** §2G.3d: per-note SkyNode lookup for ModeContext (linkCount,
     *  outgoingCount, createdAt, modifiedAt). */
    let pathToSkyNode = new Map<string, SkyNode>();
    /** §2G.3d: universe-wide stats (T mode wedges, time spans, etc.). */
    let modeStats: ModeStats = emptyModeStats();
    /** §2G.3f: per-library deterministic color map (libraryPath →
     *  { hex, css, index }). Built once per fetch from the region
     *  wedge order (sorted by note count desc). Stars, rim numbers,
     *  and the legend panel all read from here so colors stay in
     *  lockstep across the three surfaces. */
    let libraryColors = $state<Map<string, { hex: number; css: string; index: number }>>(new Map());

    // ─── §2G.3g: chart-level zoom + pan ──────────────────────────────
    /** Scale factor for the chart layers (stars, rim, milky way, etc.).
     *  HTML overlays (Universe Health, Universe-name header, legend,
     *  close button) stay screen-anchored. */
    let chartZoom = $state(1.0);
    let chartPanX = $state(0);
    let chartPanY = $state(0);
    const MIN_ZOOM = 0.4;
    const MAX_ZOOM = 5.0;
    const DRAG_THRESHOLD_PX = 4;
    /** Drag state for the pan gesture. Null when no button is down. */
    let panDragState: {
        startClientX: number;
        startClientY: number;
        startPanX: number;
        startPanY: number;
    } | null = null;
    /** True if the current pointer-down → pointer-up sequence has
     *  exceeded the drag threshold. Used to suppress the click that
     *  fires after a drag. Reset on the next pointer-down. */
    let panDragMoved = false;

    /** §2G.3b: rim label geometry, published by `drawRegionRim`.
     *  Rendered as HTML elements (with `dir="auto"`) in the Svelte
     *  template so Unicode bidi shaping works natively for Arabic /
     *  Hebrew / mixed-script library names. */
    interface RimLabelGeo {
        key: string;
        name: string;
        /** §2G.3f: 1-indexed library number rendered ON the rim.
         *  Library names move to the legend panel — see §2G.3f. */
        index: number;
        /** §2G.3f: per-library hex color (CSS string). */
        colorCss: string;
        count: number;
        /** Number label position (mid-arc). */
        lx: number;
        ly: number;
        /** Rotation in degrees, already flipped for the lower half. */
        rotDeg: number;
    }
    let rimLabelGeometry = $state<RimLabelGeo[]>([]);

    // ─── Interaction state ───────────────────────────────────────────
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);
    let hoveredPath = $state<string | null>(null);
    let selectedPath = $state<string | null>(null);
    let tooltipText = $state<string>('');
    let tooltipX = $state<number>(0);
    let tooltipY = $state<number>(0);
    let tooltipVisible = $state<boolean>(false);
    /** MIG-019 §2C: rim-driven month filter. -1 = no filter; 0..11 =
     *  Gregorian month index. Hijri / others land in PJ-014 follow-up. */
    let monthFilterMonth = $state<number>(-1);
    /** Whether the filter is from a click (persistent) or hover (preview). */
    let monthFilterPersistent = $state<boolean>(false);
    let hoveredMonth = $state<number>(-1);
    /** MIG-019 §2E: derived set of matched note_paths from searchMatchIds.
     *  Empty Set = no filter (or empty match list). */
    let matchedPaths = $state<Set<string>>(new Set());

    // ─── Helpers ─────────────────────────────────────────────────────
    function currentProjection(): ProjectionMode {
        const s = get(appSettings) as any;
        return s?.sight?.projection === 'stereographic' ? 'stereographic' : 'lambert';
    }

    /** MIG-019 §2E.4: Pixi v8 `removeChildren()` only DETACHES — it
     *  doesn't free GPU buffers. With 6+ $effects firing on mount and
     *  fullRedraw running once, calling `container.removeChildren()`
     *  before each redraw leaks WebGL state across cycles. The GPU
     *  process eventually saturates → page-OOM.
     *
     *  This helper detaches AND destroys every child so per-cycle
     *  state actually returns to zero. */
    function safeClearContainer(container: Container) {
        const old = container.removeChildren();
        for (const child of old) {
            try { child.destroy(); } catch { /* defensive — already destroyed */ }
        }
    }

    /** §2G.3e: deterministic [0, 1) hash from a string. djb2 variant —
     *  fast, collision-tolerant for our use (uniform spread of stars
     *  within their wedge, not security). */
    function hash01(s: string): number {
        let h = 5381;
        for (let i = 0; i < s.length; i++) {
            h = ((h << 5) + h) ^ s.charCodeAt(i);  // h * 33 ^ c
        }
        return ((h >>> 0) % 1_000_003) / 1_000_003;
    }

    function starRadius(centrality_norm: number): number {
        return 1.5 + Math.sqrt(Math.max(0, centrality_norm)) * 4.5;
    }

    /** §2G.3m: SINGLE source of truth for the actual drawn star radius.
     *  Both drawStars and drawFocusOverlay call this so the selection
     *  ring exactly matches the node it surrounds (was: drawStars used
     *  one formula, drawFocusOverlay read screen.r which used a
     *  different scale → ring up to 2.35× bigger than the node). */
    const STAR_MIN_RADIUS = 1.2;
    const STAR_MAX_RADIUS = 4.0;
    function actualNodeRadius(screenR: number): number {
        const sizeNorm = Math.max(0, Math.min(1, screenR / 8.4));
        return STAR_MIN_RADIUS + sizeNorm * (STAR_MAX_RADIUS - STAR_MIN_RADIUS);
    }

    /** §2G.3b — dome geometry with breathing room.
     *
     *  Eisa's directive 2026-05-07: "the dome should be at least 100 px
     *  away from the top and bottom window borders." The Universe Health
     *  card occupies ~155 px at top, so we reserve ~270 px there
     *  (Universe Health + 100 px clear margin). 100 px reserved at
     *  bottom. Sides reserve 100 px so the rim labels (which extend
     *  ~70 px outside the dome) don't kiss the window edge.
     *
     *  The dome center shifts down to the middle of the AVAILABLE
     *  vertical space, not the middle of the canvas.  */
    /** §2G.3l (evidence-backed redesign): Pixi-native zoom for the
     *  canvas, CSS-transform on a separate wrapper for HTML overlays.
     *  Both driven by the same chartZoom + chartPanX/Y values,
     *  applied in lockstep so they look like one lens.
     *
     *  Why split? CSS-scaling the <canvas> element blurs it (canvas
     *  is a fixed bitmap; MDN "Optimizing canvas"). The canonical
     *  Pixi pattern is `container.scale.set(zoom)` with `pivot` and
     *  `position` for translation (Steve Ruiz "Creating a Zoom UI";
     *  pixi-viewport library). Hit-testing automatically accounts
     *  for container transforms.
     *
     *  HTML overlays (legend, Universe Health, etc.) are pure DOM,
     *  so they CSS-scale via a wrapper transform. They stay sharp
     *  because they're vector text/CSS, not bitmap.
     *
     *  Eisa directive 2026-05-08: "everything locked in place; mouse
     *  wheel as a lens." Two transform sources, one chartZoom value,
     *  same visual result. */
    let overlaysTransform = $state('none');
    let overlaysTransformOrigin = $state('50% 50%');

    /** Pixi-side: scale + pan the chartContainer (which holds stars,
     *  rim, Milky Way, edges, focus overlay). */
    function updateChartTransform() {
        if (!chartContainer) return;
        const vp = getViewport();
        if (!vp) return;
        chartContainer.pivot.set(vp.cx, vp.cy);
        chartContainer.position.set(vp.cx + chartPanX, vp.cy + chartPanY);
        chartContainer.scale.set(chartZoom);
    }

    /** HTML side: CSS transform on the overlays wrapper. Same pivot
     *  and translate as Pixi so they move in lockstep.
     *  §2G.3m: switched to translate3d/scale3d for GPU-precise
     *  sub-pixel rendering matching what Pixi does on its canvas;
     *  the 2D versions can produce slight rounding differences
     *  that show up as "library numbers offset on zoom." */
    function syncOverlaysTransform() {
        const vp = getViewport();
        if (!vp) {
            overlaysTransform = 'none';
            overlaysTransformOrigin = '50% 50%';
            return;
        }
        overlaysTransformOrigin = `${vp.cx}px ${vp.cy}px`;
        if (chartZoom === 1 && chartPanX === 0 && chartPanY === 0) {
            overlaysTransform = 'none';
        } else {
            overlaysTransform = `translate3d(${chartPanX}px, ${chartPanY}px, 0) scale3d(${chartZoom}, ${chartZoom}, 1)`;
        }
    }

    /** One call applies both transforms in lockstep. */
    function syncZoomTransform() {
        updateChartTransform();
        syncOverlaysTransform();
    }
    /** Back-compat alias. */
    function syncRimTransform() { syncOverlaysTransform(); }

    /** §2G.3g: wheel-zoom handler. preventDefault stops the page from
     *  scrolling and stops Ctrl+wheel from triggering browser zoom.
     *  §2G.3k: rAF-throttled — high-DPI mice fire 100+ wheel events
     *  per second; without throttling, each event was scheduling a
     *  Svelte reactive update and flooding the main thread to a
     *  freeze. Now: zoom state updates immediately, but the
     *  syncZoomTransform DOM write happens at most once per frame. */
    let zoomFrame: number | null = null;
    function handleWheel(ev: WheelEvent) {
        ev.preventDefault();
        const dz = -ev.deltaY * 0.0015;
        const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, chartZoom * Math.exp(dz)));
        if (newZoom === chartZoom) return;
        chartZoom = newZoom;
        if (zoomFrame !== null) return;  // already a sync scheduled this frame
        zoomFrame = requestAnimationFrame(() => {
            zoomFrame = null;
            syncZoomTransform();
        });
    }

    /** §2G.3g: pointer-down starts a potential pan gesture. We don't
     *  commit to dragging until the pointer moves more than DRAG_THRESHOLD;
     *  that lets short clicks still hit stars. */
    function handlePointerDown(ev: PointerEvent) {
        if (ev.button !== 0) return;  // primary mouse button only
        panDragState = {
            startClientX: ev.clientX,
            startClientY: ev.clientY,
            startPanX: chartPanX,
            startPanY: chartPanY,
        };
        panDragMoved = false;
    }

    /** §2G.3g: pointer-up cleans up the drag state and the cursor. */
    function handlePointerUp() {
        if (panDragState) {
            panDragState = null;
            if (canvasContainer) canvasContainer.style.cursor = 'default';
        }
    }

    /** §2G.3g: reset zoom + pan to defaults. Bound to Esc when no
     *  selection is active (existing handleEscape path) and to a
     *  "Reset view" button when zoom != 1 or pan != 0. */
    function resetView() {
        if (chartZoom === 1 && chartPanX === 0 && chartPanY === 0) return;
        chartZoom = 1.0;
        chartPanX = 0;
        chartPanY = 0;
        updateChartTransform();
        syncRimTransform();
    }

    function getViewport() {
        if (!app) return null;
        const w = app.screen.width;
        const h = app.screen.height;
        if (w === 0 || h === 0) return null;

        // §2G.3c: bumped from 270 → 320 so the Universe-name header
        // (which sits above the dome and below the Universe Health
        // metrics) has 50 px of dedicated space. 100 px Eisa-mandated
        // dome breathing room is preserved.
        const TOP_RESERVE = 320;
        const BOTTOM_RESERVE = 100;  // Eisa-mandated minimum
        const SIDE_RESERVE = 100;    // room for rim labels

        const availableW = Math.max(100, w - 2 * SIDE_RESERVE);
        const availableH = Math.max(100, h - TOP_RESERVE - BOTTOM_RESERVE);
        // Account for the rim band ~80 px outside the dome by keeping
        // domeRadius + 80 within the available bounding box.
        const RIM_OUT_PX = 80;
        const radius = Math.max(
            50,
            Math.min(availableW / 2, availableH / 2) - RIM_OUT_PX,
        );
        const cx = w / 2;
        const cy = TOP_RESERVE + availableH / 2;
        return { cx, cy, radius };
    }

    function buildIndices() {
        // Path-keyed lookups
        pathToPoint.clear();
        for (const pt of layoutPoints) {
            pathToPoint.set(pt.note_path, pt);
        }

        // Centrality rank: highest centrality_norm → rank 1
        centralityRank.clear();
        const ranked = [...layoutPoints].sort((a, b) => b.centrality_norm - a.centrality_norm);
        ranked.forEach((p, i) => centralityRank.set(p.note_path, i + 1));

        // Name + library + createdAt lookups from skyNodes
        nameToPath.clear();
        pathToTitle.clear();
        pathToLibrary.clear();
        pathToCreatedAt.clear();
        // §2G.3d: full per-note SkyNode lookup so positionForMode can
        // access linkCount, outgoingCount, createdAt, etc. in O(1).
        pathToSkyNode.clear();
        for (const n of nodes) {
            nameToPath.set(n.id, n.path);
            pathToTitle.set(n.path, n.name);
            pathToLibrary.set(n.path, n.libraryName);
            if (typeof n.createdAt === 'number') {
                pathToCreatedAt.set(n.path, n.createdAt);
            }
            pathToSkyNode.set(n.path, n);
        }
        // §2G.3d: build universe-wide stats once per fetch (T mode
        // year wedges, createdAt/modifiedAt min/max, etc.).
        const statsInput = new Map<string, { createdAt?: number; modifiedAt?: number }>();
        for (const n of nodes) {
            statsInput.set(n.path, {
                createdAt: typeof n.createdAt === 'number' ? n.createdAt : undefined,
                // SkyNode lacks modifiedAt today — reuse createdAt as a
                // safe stand-in until note_meta.modified_at is piped
                // through. Recency degrades gracefully (rim).
                modifiedAt: typeof n.createdAt === 'number' ? n.createdAt : undefined,
            });
        }
        modeStats = buildModeStats(statsInput);

        // Resolved edges in (path-A, path-B) form, deduped via
        // index-packed bigints. Replaces the prior `Set<string>` of
        // `${aPath}|${bPath}` keys (which held ~656k × ~150 bytes ≈
        // 100 MB on Boss-scale universes — a primary OOM contributor).
        // Now: `Set<bigint>` of `(a_idx << 32 | b_idx)` keys, ~16 bytes
        // each. ~10× memory reduction.
        const pathToIdxLocal = new Map<string, number>();
        let nextIdx = 0;
        const indexOf = (p: string): number => {
            let i = pathToIdxLocal.get(p);
            if (i === undefined) { i = nextIdx++; pathToIdxLocal.set(p, i); }
            return i;
        };
        const edgeSet = new Set<bigint>();
        resolvedEdges = [];
        for (const link of links) {
            const aPath = nameToPath.get(link.source);
            const bPath = nameToPath.get(link.target);
            if (!aPath || !bPath || aPath === bPath) continue;
            if (!pathToPoint.has(aPath) || !pathToPoint.has(bPath)) continue;
            const aIdx = indexOf(aPath);
            const bIdx = indexOf(bPath);
            const lo = aIdx < bIdx ? aIdx : bIdx;
            const hi = aIdx < bIdx ? bIdx : aIdx;
            const key = (BigInt(lo) << 32n) | BigInt(hi);
            if (edgeSet.has(key)) continue;
            edgeSet.add(key);
            resolvedEdges.push({ a: aPath, b: bPath });
        }

        // Communities — Louvain on (id, name) nodes + skyLinks
        const communityNodeSubset = nodes.map((n) => ({ id: n.id, name: n.name }));
        const louvain = detectClusters(
            communityNodeSubset,
            links.map((l) => ({ source: l.source, target: l.target })),
        );
        clusters = louvain.clusters;
        communityById.clear();
        for (const c of clusters) communityById.set(c.id, c);
        // Map note_path → community id (via lowercased name).
        pathToCommunity.clear();
        for (const [nodeId, communityId] of louvain.assignments) {
            const path = nameToPath.get(nodeId);
            if (path) pathToCommunity.set(path, communityId);
        }

        // MIG-019 §2D: compute universe-health report from Louvain output.
        // Passes through to clusterEngine.ts::computeUniverseHealth +
        // computeStructuralGaps via the universe-health.ts wrapper.
        const linkPairs = links.map((l) => ({ source: l.source, target: l.target }));
        if (clusters.length > 0 && layoutPoints.length > 0) {
            healthReport = computeHealthReport(
                louvain.modularity,
                clusters,
                linkPairs,
                louvain.assignments,
                layoutPoints.length,
            );
        } else {
            healthReport = emptyHealthReport();
        }
    }

    /** Project all layout points to screen using the active rim-axis mode.
     *
     *  §2G.3d: per-mode (X, Y, Z) dispatch. Each mode declares its own
     *  azimuth / radius / magnitude rules (see modes.ts::positionForMode).
     *  Color stays invariant (community Louvain). The chart becomes a
     *  multi-instrument cognitive lens: same Universe, different scan.
     *
     *  R · Regions  X=library wedge        Y=centrality rank   Z=degree
     *  L · Link Types X=dominant link type Y=type diversity*   Z=outgoing*
     *  T · Time     X=creation date wedge  Y=recency           Z=age
     *  C · Confidence — falls back to Regions until P2 ships
     *  S · Stages — falls back to Regions until P3 ships
     *  A · Acts — falls back to Regions until P4 ships
     *
     *  *Items marked * use SkyNode.outgoingCount today; full L mode
     *   needs note_links.link_type piped in (§2G.4 follow-up).
     */
    function recomputeScreenPositions() {
        const viewport = getViewport();
        if (!viewport) return;

        // Build per-fetch caches once (region wedges, embed angles,
        // centrality rank). Cleared whenever fetchLayout runs.
        if (regionLayout == null && layoutPoints.length > 0 && libraryPathsCached.length > 0) {
            regionLayout = buildRegionLayout(layoutPoints, libraryPathsCached);
            // §2G.3f: per-library color map. Built once from the wedge
            // order (sorted by count desc). Stars + rim numbers + legend
            // panel all read from this single source of truth.
            libraryColors = buildLibraryColorMap(regionLayout.wedges);
            // §2G.3e: hash-based in-wedge azimuth jitter so MDS-clustered
            // notes don't stack on the same radial spoke. Uniform [0, 2π).
            pathToEmbedAngle.clear();
            for (const pt of layoutPoints) {
                pathToEmbedAngle.set(pt.note_path, hash01(pt.note_path) * Math.PI * 2);
            }
            pathToCentralityRank.clear();
            const sorted = [...layoutPoints].sort(
                (a, b) => b.centrality_norm - a.centrality_norm,
            );
            const denom = Math.max(1, sorted.length - 1);
            sorted.forEach((pt, i) => {
                // §2G.3f: SQRT mapping for radial position. Real centrality
                // distribution is fine; the issue is angular AREA scales
                // with r, so a uniform rank percentile packs more stars
                // per unit area near center. sqrt(rank) corrects this →
                // uniform AREA density, no inner crowding. Plus the
                // small ±1.5 % jitter (different salt) breaks rings.
                const baseRank = Math.sqrt(i / denom);
                const jitter = (hash01(pt.note_path + '|rj') - 0.5) * 0.03;
                pathToCentralityRank.set(
                    pt.note_path,
                    Math.max(0, Math.min(1, baseRank + jitter)),
                );
            });
        }

        pathToScreen.clear();
        const domeR = viewport.radius;
        const cx = viewport.cx;
        const cy = viewport.cy;

        // §2G.3c radial breathing room.
        const INNER_INSET = 0.04 * domeR;
        const OUTER_CAP = 0.96 * domeR;

        // Reusable ModeContext — mutated per-iteration to avoid
        // allocations on the hot path.
        const ctx: ModeContext = {
            notePath: '',
            centralityRank: 0,
            linkCount: 0,
            outgoingCount: 0,
            createdAt: null,
            modifiedAt: null,
            regionLayout,
            embedAngleRad: 0,
            domeR,
            innerInset: INNER_INSET,
            outerCap: OUTER_CAP,
            stats: modeStats,
        };

        for (const pt of layoutPoints) {
            const sky = pathToSkyNode.get(pt.note_path);
            ctx.notePath = pt.note_path;
            ctx.centralityRank = pathToCentralityRank.get(pt.note_path) ?? 0.5;
            ctx.linkCount = sky?.linkCount ?? 0;
            ctx.outgoingCount = sky?.outgoingCount ?? 0;
            ctx.createdAt = (sky?.createdAt ?? null) as number | null;
            ctx.modifiedAt = (sky?.createdAt ?? null) as number | null;  // see buildIndices note
            ctx.embedAngleRad = pathToEmbedAngle.get(pt.note_path) ?? 0;
            ctx.regionLayout = regionLayout;
            ctx.stats = modeStats;

            const pos: ModePosition = positionForMode(currentMode, ctx);
            const { x, y } = polarToCartesian(pos.radius, pos.azimuth, cx, cy);
            pathToScreen.set(pt.note_path, {
                x, y,
                r: pos.magnitude * 1.4,
                baseAlpha: pos.alpha,
            });
        }

        // §2G.3e: repulsion pass — Eisa's directive ("each node shouldn't
        // touch or overlap its neighbors. There should be a node repulsion
        // effect"). Within each wedge, push apart any pair of stars
        // closer than MIN_DIST. Bounded to wedge bounds so a Biology
        // star can't drift into Physics. Spatial grid keeps it O(N) per
        // iteration; 6 iterations converges fast.
        if (regionLayout && currentMode === 'regions') {
            applyWedgeRepulsion(cx, cy, domeR);
        }
    }

    /** §2G.3e: wedge-bounded repulsion. Stars within the same wedge
     *  push each other apart until none overlap. Each star is then
     *  re-clamped to its wedge's angular bounds + the dome's radial
     *  bounds so repulsion can't push a star outside its wedge. */
    function applyWedgeRepulsion(cx: number, cy: number, domeR: number) {
        if (!regionLayout) return;
        // §2G.3f: tighter spacing rule per Eisa's directive ("nodes
        // shouldn't touch each other"). 9 px between centers comfortably
        // exceeds 2 × MAX_NODE_RADIUS = 8 px so adjacent stars at max
        // size still have a 1 px breathing gap. 12 iterations to ensure
        // convergence at high density.
        const MIN_DIST = 9;
        const MAX_ITER = 12;
        const ITER_FACTOR = 0.45; // per-pass push amount as fraction of deficit

        // Group stars by wedge — repulsion only acts within a wedge.
        type StarRef = { path: string; pos: { x: number; y: number; r: number; baseAlpha: number }; wedge: RegionWedge };
        const wedgeStars = new Map<RegionWedge, StarRef[]>();
        pathToScreen.forEach((screen, path) => {
            const wedge = regionLayout!.pathToWedge.get(path);
            if (!wedge) return;
            const arr = wedgeStars.get(wedge);
            const ref = { path, pos: screen, wedge };
            if (arr) arr.push(ref);
            else wedgeStars.set(wedge, [ref]);
        });

        const cellSize = MIN_DIST * 2;
        const innerR = 0.04 * domeR;
        const outerR = 0.96 * domeR;
        const TAU = Math.PI * 2;

        for (let iter = 0; iter < MAX_ITER; iter++) {
            let moved = false;
            wedgeStars.forEach((stars) => {
                if (stars.length < 2) return;
                // Build spatial grid (per iteration since positions move).
                const grid = new Map<string, StarRef[]>();
                for (const s of stars) {
                    const gx = Math.floor(s.pos.x / cellSize);
                    const gy = Math.floor(s.pos.y / cellSize);
                    const k = gx + ',' + gy;
                    const cell = grid.get(k);
                    if (cell) cell.push(s);
                    else grid.set(k, [s]);
                }
                // Repel each star against neighbors in 9 nearby cells.
                for (const s of stars) {
                    const gx = Math.floor(s.pos.x / cellSize);
                    const gy = Math.floor(s.pos.y / cellSize);
                    for (let dx = -1; dx <= 1; dx++) {
                        for (let dy = -1; dy <= 1; dy++) {
                            const cell = grid.get((gx + dx) + ',' + (gy + dy));
                            if (!cell) continue;
                            for (const o of cell) {
                                if (o.path === s.path) continue;
                                const ddx = s.pos.x - o.pos.x;
                                const ddy = s.pos.y - o.pos.y;
                                const d2 = ddx * ddx + ddy * ddy;
                                if (d2 >= MIN_DIST * MIN_DIST || d2 <= 0) continue;
                                const d = Math.sqrt(d2);
                                const push = (MIN_DIST - d) * ITER_FACTOR;
                                s.pos.x += (ddx / d) * push;
                                s.pos.y += (ddy / d) * push;
                                moved = true;
                            }
                        }
                    }
                }
                // After pushing, clamp every star back into its wedge.
                for (const s of stars) {
                    const ddx = s.pos.x - cx;
                    const ddy = s.pos.y - cy;
                    let r = Math.sqrt(ddx * ddx + ddy * ddy);
                    if (r < 1) r = 1;  // avoid div-by-zero on pole
                    let theta = Math.atan2(ddx, -ddy);  // 0 at top, CW+
                    theta = ((theta % TAU) + TAU) % TAU;
                    // Clamp angle to wedge bounds with 2 % inset.
                    const span = s.wedge.arcEndRad - s.wedge.arcStartRad;
                    const inset = Math.min(0.02 * span, 0.04);
                    const lo = s.wedge.arcStartRad + inset;
                    const hi = s.wedge.arcEndRad - inset;
                    if (theta < lo) theta = lo;
                    else if (theta > hi) theta = hi;
                    // Clamp radius to dome bounds.
                    if (r < innerR) r = innerR;
                    else if (r > outerR) r = outerR;
                    // Recompose.
                    s.pos.x = cx + r * Math.sin(theta);
                    s.pos.y = cy - r * Math.cos(theta);
                }
            });
            if (!moved) break;
        }
    }

    /** MIG-019 §2C — Calendar rim (concentric rings, one per enabled
     *  calendar system). Renders arc dividers + month labels around the
     *  perimeter just outside the dome. Eisa's design call (§11 Q3,
     *  2026-05-07): Gregorian default; users add others via Settings. */
    function drawCalendarRim() {
        if (!calendarRimContainer || !app) return;
        safeClearContainer(calendarRimContainer);

        const enabled = ($appSettings.sight?.calendarSystems ?? ['gregorian']) as CalendarSystem[];
        if (enabled.length === 0) return;

        // §2G.3b: share viewport with the dome so the rim is concentric.
        const vp = getViewport();
        if (!vp) return;
        const domeRadius = vp.radius;
        const rimViewport = {
            cx: vp.cx,
            cy: vp.cy,
            innerRadius: domeRadius + 4, // small gap between dome edge and rim
            outerRadius: domeRadius + 4 + 22 * enabled.length,
        };

        const locale = get(i18nLocale) ?? 'en';
        monthSegments = monthArcSegments(rimViewport, enabled, locale);

        // Current Gregorian month (highlighted by a brighter ring color).
        const currentGregorianMonth = new Date().getMonth();

        // Render arc dividers
        const arcs = new Graphics();
        for (const seg of monthSegments) {
            // Each segment: draw the radial divider line at the start angle
            // (ring boundary at startAngle).
            const x1 = rimViewport.cx + seg.rIn * Math.cos(seg.startAngle);
            const y1 = rimViewport.cy + seg.rIn * Math.sin(seg.startAngle);
            const x2 = rimViewport.cx + seg.rOut * Math.cos(seg.startAngle);
            const y2 = rimViewport.cy + seg.rOut * Math.sin(seg.startAngle);
            arcs.moveTo(x1, y1);
            arcs.lineTo(x2, y2);
        }
        // Outer + inner ring boundaries
        for (let r = 0; r <= enabled.length; r++) {
            const radius = rimViewport.innerRadius + r * 22;
            arcs.circle(rimViewport.cx, rimViewport.cy, radius);
        }
        arcs.stroke({ color: 0xd4af37, alpha: 0.35, width: 1 });
        calendarRimContainer.addChild(arcs);

        // Render month labels
        for (const seg of monthSegments) {
            const isCurrent = seg.calendar === 'gregorian' && seg.monthIndex === currentGregorianMonth;
            const isHovered = seg.calendar === 'gregorian' && seg.monthIndex === hoveredMonth;
            const isFiltered = seg.calendar === 'gregorian'
                && monthFilterMonth === seg.monthIndex
                && monthFilterPersistent;
            const alpha = isFiltered ? 1.0 : isHovered ? 0.95 : isCurrent ? 0.9 : 0.65;
            const fontSize = isCurrent || isFiltered ? 11 : 10;
            const label = new Text({
                text: seg.label,
                style: new TextStyle({
                    fill: isCurrent || isFiltered ? 0xd4af37 : 0xf5e6c8,
                    fontSize,
                    fontFamily: 'system-ui, -apple-system, sans-serif',
                    align: 'center',
                }),
                anchor: 0.5,
            });
            label.x = seg.labelX;
            label.y = seg.labelY;
            label.alpha = alpha;
            calendarRimContainer.addChild(label);
        }
    }

    /** Month-filter check: returns true if this note matches the current
     *  filter (or there is no filter). Used by drawStars to dim non-matching. */
    function passesMonthFilter(notePath: string): boolean {
        if (monthFilterMonth < 0) return true;
        const created = pathToCreatedAt.get(notePath);
        if (created === undefined) return false;
        return new Date(created).getMonth() === monthFilterMonth;
    }

    /** MIG-019 §2E: search-active check. */
    function isSearchActive(): boolean {
        return matchedPaths.size > 0;
    }

    function passesSearch(notePath: string): boolean {
        if (!isSearchActive()) return true;
        return matchedPaths.has(notePath);
    }

    /** Resolve a community's "has match" boolean — used for territory halo. */
    function communityHasMatch(communityId: number): boolean {
        if (!isSearchActive()) return false;
        for (const path of matchedPaths) {
            if (pathToCommunity.get(path) === communityId) return true;
        }
        return false;
    }

    /** MIG-019 §2A+§2B redesign — Milky Way density wash (PJ-035).
     *  Renders the Rust-side density grid as a single Pixi Sprite.
     *
     *  The grid (256×256 f32) was accumulated by rasterizing every
     *  high-similarity pair into grid space and then Gaussian-blurring
     *  to smooth into a continuous band texture. We build an RGBA8
     *  ImageData from the grid (Suwaidi cream tinted; alpha proportional
     *  to normalized cell value) and convert to a Pixi Texture via a
     *  one-time canvas. Single draw call — universe size irrelevant. */
    function drawMilkyWay() {
        if (!milkyWayContainer || !app) return;
        safeClearContainer(milkyWayContainer);
        // Hide if Settings → Sight → "Milky Way density wash" is OFF.
        if ($appSettings.sight?.showMilkyWay === false) {
            milkyWayContainer.visible = false;
            return;
        }
        milkyWayContainer.visible = true;
        if (!densityField || densityField.max_value <= 0) return;

        // Build RGBA8 ImageData from the float grid.
        const w = densityField.width;
        const h = densityField.height;
        const maxV = densityField.max_value || 1.0;
        const canvas = document.createElement('canvas');
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;
        const imgData = ctx.createImageData(w, h);
        const rgba = imgData.data;
        for (let i = 0; i < w * h; i++) {
            // Normalize, then apply soft gamma (sqrt) so faint regions
            // are still visible while the brightest spots don't oversaturate.
            const v = Math.sqrt(Math.max(0, densityField.values[i] / maxV));
            // Suwaidi cream (#f5e6c8) — same as the connector lines.
            rgba[i * 4 + 0] = 0xf5;
            rgba[i * 4 + 1] = 0xe6;
            rgba[i * 4 + 2] = 0xc8;
            // Cap alpha at ~110/255 (~43%) so the band stays subtle —
            // it's a backdrop, not a foreground feature.
            rgba[i * 4 + 3] = Math.round(v * 110);
        }
        ctx.putImageData(imgData, 0, 0);

        // Convert to Pixi Texture and stretch to fill the dome.
        // §2G.3b: align with shared viewport so the Milky Way sits
        // inside the actual dome (not centered on the raw canvas).
        const texture = Texture.from(canvas);
        const sprite = new Sprite(texture);
        const vp = getViewport();
        if (!vp) return;
        const domeRadius = vp.radius;
        sprite.width = domeRadius * 2;
        sprite.height = domeRadius * 2;
        sprite.x = vp.cx - domeRadius;
        sprite.y = vp.cy - domeRadius;
        milkyWayContainer.addChild(sprite);
    }

    /** Draw community territory polygons on the base layer.
     *  - MIG-019 §2E search-active path: territories with matches get
     *    halo (border thickens + brightens); territories without dim.
     *  - MIG-019 §2E always-on labels: render community label at the
     *    centroid when $appSettings.sight.alwaysOnLabels is true. */
    function drawTerritories() {
        if (!territoryContainer) return;
        safeClearContainer(territoryContainer);

        // MIG-019 §2G (Eisa post-§2G.3 directive 2026-05-07): community
        // territories were a v2 carryover that worked when MDS clustered
        // each community geographically. In the polar layout azimuth is
        // library-based, so each community is now scattered across many
        // wedges — the convex hulls span the whole dome and 20 of them
        // overlap into a solid blob that hides the stars.
        //
        // Disabled in §2G.3. We may revive a per-(library × community)
        // sub-territory layer in a later phase, but only after we have
        // a layout grammar where it'd actually communicate something.
        // (Function intentionally returns above. The territory layer
        //  stays empty under the polar layout — see comment.)
    }

    /** Draw faint connector lines on the base layer.
     *  Cap rendered edges at MAX_FAINT_EDGES_AT_REST to bound the Pixi
     *  buffer + draw cost on large universes (Boss-scale: 656k links).
     *  When over-cap, the visible sample is the FIRST N edges from
     *  `resolvedEdges` — deterministic since iteration order is stable.
     *  The full edge set is still in `resolvedEdges` for the focus
     *  overlay (hover/click brightens incident edges regardless of cap). */
    const MAX_FAINT_EDGES_AT_REST = 30000;
    /** MIG-019 §2G (spec §3): edges are HIDDEN in the resting state.
     *  No constellation lines are drawn unless a star is hovered or
     *  selected. The focus overlay (`drawFocusOverlay`) handles the
     *  active state by drawing only the selected node's incident edges
     *  in gold. Per Concept Paper §4.1 — "we will show it as faint lines
     *  until the user hovers over or the connected nodes linking them."
     *
     *  This function intentionally clears the edge container and stops
     *  there: it removes any prior constellation chains so they don't
     *  ghost across mode switches. The edge layer stays empty until
     *  `drawFocusOverlay` renders into it on hover/click.  */
    function drawEdges() {
        if (!edgeContainer) return;
        safeClearContainer(edgeContainer);
        // Resting state — no edges. The focus overlay handles active state.
    }

    /** Draw stars on the base layer.
     *
     *  MIG-019 §2E.4 SOLVE (2026-05-07): consolidated 7,334 separate
     *  `new Graphics()` instances into a SINGLE Graphics with 7,334
     *  circle subpaths. v2 ConstellationSight2 used Canvas 2D (single
     *  canvas, direct draw calls — no per-shape GPU allocations); my
     *  Pixi v8 implementation was allocating GPU buffers per-instance,
     *  which the WebView2 GPU process can't sustain at Boss scale
     *  (7,334 stars × 217k links). Single-Graphics + per-circle
     *  fill() batches everything into one GL draw call.
     *
     *  - MIG-019 §2C: dim stars failing the month filter.
     *  - MIG-019 §2E: search-active path — matched stars flare (size +
     *    brightness boost), non-matched dim heavily. */
    function drawStars() {
        if (!starContainer) return;
        safeClearContainer(starContainer);
        const searchOn = isSearchActive();
        const stars = new Graphics();
        // §2G.3m: actualNodeRadius is the SINGLE source of truth for
        // the drawn star radius. drawFocusOverlay calls the same
        // function so selection rings exactly match the node size.
        for (const pt of layoutPoints) {
            const screen = pathToScreen.get(pt.note_path);
            if (!screen) continue;
            const passesMonth = passesMonthFilter(pt.note_path);
            const passesSrc = passesSearch(pt.note_path);
            const baseAlpha = screen.baseAlpha;

            let radius = actualNodeRadius(screen.r);

            let alpha: number;
            if (searchOn) {
                if (passesSrc && passesMonth) {
                    radius = Math.min(radius * 1.5, STAR_MAX_RADIUS + 1.0);
                    alpha = 1.0;
                } else {
                    alpha = 0.10;
                }
            } else if (!passesMonth) {
                alpha = 0.15;
            } else {
                alpha = baseAlpha;
            }

            // §2G.3f: per-library color (was uniform near-black). Falls
            // back to ink when no wedge is found.
            const wedge = regionLayout?.pathToWedge.get(pt.note_path);
            const libColor = wedge ? libraryColors.get(wedge.libraryPath) : undefined;
            const fillColor = libColor?.hex ?? 0x1a1a1a;

            // §2G.3f: each star draws as TWO subpaths in the same
            // Graphics — fill + stroke — so we still hit the single
            // GL draw call from §2E.4 OOM solve.
            stars.circle(screen.x, screen.y, radius);
            stars.fill({ color: fillColor, alpha });
            // Thin contrast stroke around every star (Eisa directive).
            stars.circle(screen.x, screen.y, radius);
            stars.stroke({ color: 0x1a1a1a, alpha: alpha * 0.85, width: 0.6 });
        }
        starContainer.addChild(stars);
    }

    /** Redraw the focus overlay based on hoveredPath / selectedPath /
     *  search-active state.
     *  - No state → empty overlay (or search-match edges if search active).
     *  - hoveredPath (no selection) → brighten edges incident to that star.
     *  - selectedPath → brighten ALL edges within that star's community.
     *  - searchActive (no specific focus) → brighten edges between matched stars.
     *  Draws AFTER drawX so it sits on top. */
    function drawFocusOverlay() {
        if (!focusOverlay) return;
        safeClearContainer(focusOverlay);

        const focusPath = selectedPath ?? hoveredPath;
        const searchOn = isSearchActive();

        // MIG-019 §2E: if no focus star but search is active, brighten
        // edges between matched stars so the user sees the search-result
        // sub-graph clearly.
        if (!focusPath && searchOn) {
            const lines = new Graphics();
            let edgeCount = 0;
            for (const e of resolvedEdges) {
                if (matchedPaths.has(e.a) && matchedPaths.has(e.b)) {
                    const sa = pathToScreen.get(e.a);
                    const sb = pathToScreen.get(e.b);
                    if (!sa || !sb) continue;
                    lines.moveTo(sa.x, sa.y);
                    lines.lineTo(sb.x, sb.y);
                    edgeCount++;
                }
            }
            if (edgeCount > 0) {
                lines.stroke({ color: 0xc9a227, alpha: 0.7, width: 1.0 });
                focusOverlay.addChild(lines);
            }
            return;
        }

        if (!focusPath) return;

        const focusScreen = pathToScreen.get(focusPath);
        if (!focusScreen) return;

        // §2G.3h: edges incident to the FOCUSED NODE only (was: whole
        // community on selection). Cap at MAX_FOCUS_EDGES so a hub
        // node doesn't smother the chart with hundreds of fan-out
        // lines (Eisa screenshot 2026-05-08). Lighter stroke so 1-hop
        // neighbours remain readable through the gold rays.
        const MAX_FOCUS_EDGES = 50;
        const lines = new Graphics();
        let edgeCount = 0;
        const neighbours = new Set<string>();
        for (const e of resolvedEdges) {
            if (edgeCount >= MAX_FOCUS_EDGES) break;
            if (e.a !== focusPath && e.b !== focusPath) continue;
            const sa = pathToScreen.get(e.a);
            const sb = pathToScreen.get(e.b);
            if (!sa || !sb) continue;
            lines.moveTo(sa.x, sa.y);
            lines.lineTo(sb.x, sb.y);
            edgeCount++;
            neighbours.add(e.a === focusPath ? e.b : e.a);
        }
        if (edgeCount > 0) {
            // §2G.3m: switched to INK (#1a1a1a) at alpha 0.7 for
            // strong contrast on the cream BG. The dark-amber
            // (#6b4f0d) was still too low-contrast per Eisa's
            // §2G.3l feedback.
            lines.stroke({ color: 0x1a1a1a, alpha: 0.7, width: 1.0 });
            focusOverlay.addChild(lines);
        }
        // §2G.3m: neighbour rings sized to the ACTUAL node radius
        // (was: ns.r which is the position pseudo-radius, not the
        // drawn radius — rings ended up 1.5-2× the visible node).
        if (neighbours.size > 0) {
            const neighbourRings = new Graphics();
            for (const npath of neighbours) {
                const ns = pathToScreen.get(npath);
                if (!ns) continue;
                const r = actualNodeRadius(ns.r) + 0.8;
                neighbourRings.circle(ns.x, ns.y, r);
                neighbourRings.stroke({ color: 0xc9a227, alpha: 0.85, width: 0.8 });
            }
            focusOverlay.addChild(neighbourRings);
        }

        // §2G.3m: ring size matches the actual drawn node radius. Was
        // using `focusScreen.r + 1` where focusScreen.r is the position
        // pseudo-radius (0.7-8.4 px), not the rendered radius (1.2-4.0).
        // Now via the shared `actualNodeRadius()` helper so the ring
        // sits exactly 1 px outside the node's visible edge.
        const ring = new Graphics();
        const focusR = actualNodeRadius(focusScreen.r);
        ring.circle(focusScreen.x, focusScreen.y, focusR + 1);
        ring.stroke({ color: 0xc9a227, alpha: 1.0, width: 1.5 });
        focusOverlay.addChild(ring);
    }

    /** Find the nearest star within `r=10` of (px, py). Returns the
     *  note_path or null. O(n) iteration; for 30k stars this is ~1ms. */
    function pickStar(px: number, py: number): string | null {
        // §2G.3l: canvas is NOT CSS-scaled (Pixi-native chartContainer
        // scale handles the zoom). So (px, py) — already canvas-internal
        // CSS coords from getBoundingClientRect — need the inverse of
        // the chartContainer's transform:
        //   chartContainer = pivot(cx, cy) + position(cx+panX, cy+panY) + scale(zoom)
        //   visual_x = (canonical_x - cx) * zoom + cx + panX
        //   canonical_x = (visual_x - cx - panX) / zoom + cx
        const vp = getViewport();
        if (!vp) return null;
        const cpx = (px - vp.cx - chartPanX) / chartZoom + vp.cx;
        const cpy = (py - vp.cy - chartPanY) / chartZoom + vp.cy;

        const HOVER_RADIUS = 10;
        let best: string | null = null;
        let bestDist = HOVER_RADIUS * HOVER_RADIUS;
        for (const [path, s] of pathToScreen) {
            const dx = s.x - cpx;
            const dy = s.y - cpy;
            const d2 = dx * dx + dy * dy;
            // tolerance: hit even a bit outside the star sprite
            const effectiveR = s.r + 4;
            const hitR2 = Math.max(d2, 0) <= effectiveR * effectiveR ? d2 : Infinity;
            if (hitR2 < bestDist) {
                bestDist = hitR2;
                best = path;
            }
        }
        return best;
    }

    /** MIG-019 §2G: rim renderer dispatched by active mode.
     *
     *  For Regions mode (§2G.3), draws library wedge labels around the
     *  outer rim — proportional widths, blue ink, tangential rotation,
     *  with note counts just outside.
     *
     *  Time mode (§2G.4) will route through `drawCalendarRim()`.
     *  Other modes (Link Types, Confidence, Stages, Acts) will draw
     *  their fixed wedge labels via small per-mode renderers added
     *  in subsequent commits. */
    function drawRimForMode() {
        if (!calendarRimContainer || !app) return;
        if (currentMode === 'regions') {
            drawRegionRim();
        } else if (currentMode === 'time') {
            drawCalendarRim();
            rimLabelGeometry = [];  // hide region rim labels
        } else {
            // Other modes not yet wired — empty rim is fine for now.
            safeClearContainer(calendarRimContainer);
            rimLabelGeometry = [];
        }
    }

    /** MIG-019 §2G.3: Regions-mode rim. Library wedges sized by note
     *  count (largest first), with tangential blue-ink labels.
     *  §2G.3b: shares the same viewport as star positioning so the rim
     *  is concentric with the dome (not centered on the raw canvas). */
    function drawRegionRim() {
        if (!calendarRimContainer || !app) return;
        safeClearContainer(calendarRimContainer);
        if (!regionLayout || regionLayout.wedges.length === 0) return;

        const vp = getViewport();
        if (!vp) return;
        const cx = vp.cx;
        const cy = vp.cy;
        const domeR = vp.radius;
        const rimInner = domeR + 18;
        const rimOuter = domeR + 70;

        // Outer + inner rim circles
        const rim = new Graphics();
        rim.circle(cx, cy, rimOuter);
        rim.stroke({ color: 0x1a1a1a, alpha: 0.55, width: 0.8 });
        rim.circle(cx, cy, rimInner);
        rim.stroke({ color: 0x1a1a1a, alpha: 0.35, width: 0.5 });
        // Dome edge
        rim.circle(cx, cy, domeR);
        rim.stroke({ color: 0x1a1a1a, alpha: 0.4, width: 0.7 });

        // Wedge dividers + labels
        const TAU = Math.PI * 2;
        for (const wedge of regionLayout.wedges) {
            // Divider line at start of wedge — rim band only.
            const tStart = wedge.arcStartRad;
            // polar.ts convention: theta=0 at TOP, CW+. Pixi uses standard math:
            // angle 0 = right (+x), CCW positive. Convert by subtracting π/2 and
            // negating: pixi_angle = -(theta - π/2) = π/2 - theta. We use sin/cos
            // directly on polar coords, matching `polarToCartesian`.
            const dx0 = (rimInner - 2) * Math.sin(tStart);
            const dy0 = -(rimInner - 2) * Math.cos(tStart);
            const dx1 = (rimOuter + 4) * Math.sin(tStart);
            const dy1 = -(rimOuter + 4) * Math.cos(tStart);
            rim.moveTo(cx + dx0, cy + dy0);
            rim.lineTo(cx + dx1, cy + dy1);
        }
        rim.stroke({ color: 0x1a1a1a, alpha: 0.55, width: 0.7 });
        calendarRimContainer.addChild(rim);

        // §2G.3n: rim NUMBERS are now Pixi Text, drawn into the same
        // calendarRimContainer that holds the rim circles. Both are
        // children of chartContainer and share the chartContainer
        // transform — so when chartZoom changes, the numbers scale
        // EXACTLY in lockstep with the circles. No more CSS-vs-Pixi
        // divergence drift (the §2G.3l-m bug).
        //
        // (The library legend panel still uses HTML — it doesn't have
        // to align with the rim, and the legend names need `dir="auto"`
        // for Arabic/Hebrew/Persian library names. Single digits don't.)
        const labelR = (rimInner + rimOuter) / 2 + 2;
        for (const wedge of regionLayout.wedges) {
            const lc = libraryColors.get(wedge.libraryPath);
            if (!lc) continue;
            const t = wedge.arcMidRad;
            const lx = cx + labelR * Math.sin(t);
            const ly = cy - labelR * Math.cos(t);
            const numText = new Text({
                text: String(lc.index),
                style: new TextStyle({
                    fontFamily: 'serif',
                    fontSize: 16,
                    fontWeight: '700',
                    fill: lc.hex,
                    stroke: { color: 0xfaf6e8, width: 3 },  // cream halo for legibility
                }),
                anchor: 0.5,
            });
            numText.x = lx;
            numText.y = ly;
            calendarRimContainer.addChild(numText);
        }

        // Legend still needs the wedge metadata (colors + names + counts).
        rimLabelGeometry = regionLayout.wedges.map((wedge) => {
            const lc = libraryColors.get(wedge.libraryPath);
            return {
                key: wedge.libraryPath,
                name: wedge.libraryName,
                index: lc?.index ?? 0,
                colorCss: lc?.css ?? '#2a4a8c',
                count: wedge.noteCount,
                lx: 0, ly: 0, rotDeg: 0,  // unused now; kept for type compat
            };
        });
    }

    function fullRedraw() {
        recomputeScreenPositions();
        drawMilkyWay();
        drawTerritories();
        drawEdges();
        drawStars();
        drawFocusOverlay();
        drawRimForMode();
        // §2G.3g: keep the Pixi container transform + HTML rim wrapper
        // transform in sync with the (possibly resized) viewport.
        updateChartTransform();
        syncRimTransform();
    }

    function showPlaceholder(message: string, isError: boolean = false) {
        if (!app) return;
        if (!placeholderText) {
            placeholderText = new Text({
                text: message,
                style: new TextStyle({
                    fill: isError ? 0xa83232 : 0x1a1a1a,
                    fontSize: 18,
                    fontFamily: 'system-ui, -apple-system, sans-serif',
                    align: 'center',
                }),
                anchor: 0.5,
            });
            app.stage.addChild(placeholderText);
        } else {
            placeholderText.text = message;
            (placeholderText.style as TextStyle).fill = isError ? 0xa83232 : 0x1a1a1a;
            placeholderText.visible = true;
        }
        placeholderText.x = app.screen.width / 2;
        placeholderText.y = app.screen.height / 2;
    }

    function hidePlaceholder() {
        if (placeholderText) placeholderText.visible = false;
    }

    // ─── Pointer handlers ────────────────────────────────────────────
    /** Get rim viewport for hit-testing. §2G.3b: mirrors the shared
     *  dome viewport so hit-tests land where the rim is actually drawn. */
    function getRimViewport() {
        if (!app) return null;
        const vp = getViewport();
        if (!vp) return null;
        const enabled = ($appSettings.sight?.calendarSystems ?? ['gregorian']) as CalendarSystem[];
        const domeRadius = vp.radius;
        return {
            cx: vp.cx,
            cy: vp.cy,
            innerRadius: domeRadius + 4,
            outerRadius: domeRadius + 4 + 22 * enabled.length,
            enabled,
        };
    }

    function handlePointerMove(ev: PointerEvent) {
        const rect = canvasContainer.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

        // §2G.3g: drag-to-pan gesture. While the pointer is down, we
        // accumulate movement until DRAG_THRESHOLD is exceeded, then
        // commit to a pan and suppress hover/click for the duration.
        // §2G.3k: rAF-throttled (same reason as wheel — high-DPI
        // pointermove fires faster than the main thread can absorb).
        if (panDragState) {
            const dx = ev.clientX - panDragState.startClientX;
            const dy = ev.clientY - panDragState.startClientY;
            if (!panDragMoved && Math.hypot(dx, dy) > DRAG_THRESHOLD_PX) {
                panDragMoved = true;
                if (canvasContainer) canvasContainer.style.cursor = 'grabbing';
                tooltipVisible = false;
            }
            if (panDragMoved) {
                chartPanX = panDragState.startPanX + dx;
                chartPanY = panDragState.startPanY + dy;
                if (zoomFrame === null) {
                    zoomFrame = requestAnimationFrame(() => {
                        zoomFrame = null;
                        syncZoomTransform();
                    });
                }
                return;  // suppress hover during drag
            }
        }

        // MIG-019 §2C / §2G.3b: rim hit-test only in Time mode. In other
        // modes, the calendar-rim hit-test was wiping the region rim
        // because it shares the same Pixi container — Eisa caught this
        // 2026-05-07. Mode is user-driven only, never auto-switched.
        if (currentMode === 'time') {
            const rimVp = getRimViewport();
            if (rimVp) {
                const monthHit = pickMonth(rimVp, px, py, rimVp.enabled);
                if (monthHit) {
                    const greg = gregorianMonthFromSegment(monthHit);
                    if (hoveredMonth !== (greg ?? -1)) {
                        hoveredMonth = greg ?? -1;
                        if (!monthFilterPersistent) {
                            monthFilterMonth = greg ?? -1;
                            drawStars();
                        }
                        drawCalendarRim();
                    }
                    tooltipVisible = false;
                    return;
                } else if (hoveredMonth !== -1) {
                    hoveredMonth = -1;
                    if (!monthFilterPersistent) {
                        monthFilterMonth = -1;
                        drawStars();
                    }
                    drawCalendarRim();
                }
            }
        }

        const nearest = pickStar(px, py);
        if (nearest !== hoveredPath) {
            hoveredPath = nearest;
            drawFocusOverlay();
        }
        if (nearest) {
            const title = pathToTitle.get(nearest) ?? '(untitled)';
            const cid = pathToCommunity.get(nearest);
            const community = cid !== undefined ? (communityById.get(cid)?.suggestedName ?? `#${cid}`) : '?';
            const rank = centralityRank.get(nearest) ?? 0;
            tooltipText = `${title}\n${$t('sightV3.tooltip.community') || 'Community'}: ${community}\n${$t('sightV3.tooltip.centralityRank') || 'Centrality rank'}: #${rank}`;
            tooltipX = ev.clientX + 12;
            tooltipY = ev.clientY + 12;
            tooltipVisible = true;
        } else {
            tooltipVisible = false;
        }
    }

    function handlePointerLeave() {
        if (hoveredPath !== null) {
            hoveredPath = null;
            drawFocusOverlay();
        }
        // §2G.3b: month-rim cleanup only in Time mode.
        if (currentMode === 'time' && hoveredMonth !== -1) {
            hoveredMonth = -1;
            if (!monthFilterPersistent) {
                monthFilterMonth = -1;
                drawStars();
            }
            drawCalendarRim();
        }
        tooltipVisible = false;
    }

    function handleClick(ev: MouseEvent) {
        // §2G.3g: a click that follows a pan gesture isn't a real click.
        // panDragMoved was set during pointer-move; consume it here so
        // the next click works normally.
        if (panDragMoved) {
            panDragMoved = false;
            return;
        }
        const rect = canvasContainer.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

        // §2C / §2G.3b: rim click toggles persistent month filter.
        // Time mode only — Region rim clicks fall through to star pick.
        if (currentMode === 'time') {
            const rimVp = getRimViewport();
            if (rimVp) {
                const monthHit = pickMonth(rimVp, px, py, rimVp.enabled);
                if (monthHit) {
                    const greg = gregorianMonthFromSegment(monthHit);
                    if (greg !== null) {
                        if (monthFilterPersistent && monthFilterMonth === greg) {
                            // Click same month again → clear persistent filter
                            monthFilterPersistent = false;
                            monthFilterMonth = -1;
                        } else {
                            monthFilterPersistent = true;
                            monthFilterMonth = greg;
                        }
                        drawStars();
                        drawCalendarRim();
                        return;
                    }
                }
            }
        }

        const nearest = pickStar(px, py);
        if (nearest) {
            selectedPath = nearest;
            drawFocusOverlay();
        } else {
            // Click on background clears selection AND any persistent month filter
            if (selectedPath !== null) {
                selectedPath = null;
                drawFocusOverlay();
            }
            if (currentMode === 'time' && monthFilterPersistent) {
                monthFilterPersistent = false;
                monthFilterMonth = -1;
                drawStars();
                drawCalendarRim();
            }
        }
    }

    function handleDoubleClick(ev: MouseEvent) {
        const rect = canvasContainer.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;
        const nearest = pickStar(px, py);
        if (nearest) {
            const lib = pathToLibrary.get(nearest) ?? '';
            onOpenNote(nearest, lib);
            onClose();
        }
    }

    // ─── Side-panel handlers ─────────────────────────────────────────
    function sidePanelOpenNote() {
        if (selectedPath) {
            const lib = pathToLibrary.get(selectedPath) ?? '';
            onOpenNote(selectedPath, lib);
            onClose();
        }
    }
    function sidePanelClose() {
        selectedPath = null;
        drawFocusOverlay();
    }

    // ─── Side-panel computed data ────────────────────────────────────
    const sidePanelTitle = $derived(selectedPath ? (pathToTitle.get(selectedPath) ?? '(untitled)') : '');
    const sidePanelCommunity = $derived.by(() => {
        if (!selectedPath) return '';
        const cid = pathToCommunity.get(selectedPath);
        if (cid === undefined) return '';
        return communityById.get(cid)?.suggestedName ?? `#${cid}`;
    });
    const sidePanelRank = $derived(selectedPath ? (centralityRank.get(selectedPath) ?? 0) : 0);
    const sidePanelTotalNotes = $derived(layoutPoints.length);
    const sidePanelIncoming = $derived.by(() => {
        if (!selectedPath) return 0;
        let count = 0;
        const sel = selectedPath;
        for (const e of resolvedEdges) if (e.a === sel || e.b === sel) count++;
        return count;
    });
    const sidePanelOutgoing = $derived(0); // §1E: incoming + outgoing combined; MIG-019 splits by direction

    /** §2G.3n: list of 1-hop neighbours for the selected note (title +
     *  library color), so the side panel can show "Connected notes:"
     *  to satisfy Eisa's request for connected-node titles. Capped at
     *  50 like the focus-overlay edges (hub nodes can have hundreds
     *  of connections — limit to keep the panel scannable). */
    const sidePanelConnectedNotes = $derived.by(() => {
        if (!selectedPath) return [] as Array<{ path: string; title: string; libraryName: string; colorCss: string }>;
        const sel = selectedPath;
        const seen = new Set<string>();
        const out: Array<{ path: string; title: string; libraryName: string; colorCss: string }> = [];
        for (const e of resolvedEdges) {
            if (out.length >= 50) break;
            const otherPath = e.a === sel ? e.b : e.b === sel ? e.a : null;
            if (!otherPath || seen.has(otherPath)) continue;
            seen.add(otherPath);
            const libraryName = pathToLibrary.get(otherPath) ?? '';
            const wedge = regionLayout?.pathToWedge.get(otherPath);
            const colorCss = wedge ? (libraryColors.get(wedge.libraryPath)?.css ?? '#1a1a1a') : '#1a1a1a';
            out.push({
                path: otherPath,
                title: pathToTitle.get(otherPath) ?? '(untitled)',
                libraryName,
                colorCss,
            });
        }
        return out;
    });

    // ─── Lifecycle ───────────────────────────────────────────────────
    onMount(async () => {
        app = new Application();
        await app.init({
            // MIG-019 §2G: Suwaidi cream parchment (#faf6e8) — was navy.
            background: 0xfaf6e8,
            resizeTo: canvasContainer,
            antialias: true,
            // §2G.3g: honor device pixel ratio so browser zoom (Ctrl+/-)
            // re-renders the dome at the new resolution. autoDensity
            // keeps the CSS size matched to the container while the
            // backing buffer scales by resolution. Without these two,
            // the canvas backing buffer stayed at the old size after
            // a zoom, so the dome appeared misaligned with the HTML
            // overlay (rim numbers, legend, etc.).
            autoDensity: true,
            resolution: Math.max(1, window.devicePixelRatio || 1),
        });
        canvasContainer.appendChild(app.canvas);

        // MIG-019 §2A+§2B redesign: Milky Way is a single Pixi Sprite
        // built from the Rust-side pre-blurred density grid. Sits
        // beneath territories so stars + edges + territory borders all
        // render visibly over the band texture. No BlurFilter needed
        // (blur is done in Rust at compute time, not per-frame).
        milkyWayContainer = new Container();
        territoryContainer = new Container();
        edgeContainer = new Container();
        starContainer = new Container();
        focusOverlay = new Container();
        // MIG-019 §2C: calendar rim sits ABOVE everything else so labels
        // never hide behind chart contents.
        calendarRimContainer = new Container();
        // Order matters: Milky Way at the back, then territories, edges,
        // stars, focus overlay, calendar rim on top.
        // §2G.3g: chartContainer wraps every layer that should zoom +
        // pan together. Stars, rim, edges, milky way, focus overlay all
        // live inside it. Placeholder text stays on the stage so it
        // doesn't scale with chart zoom.
        chartContainer = new Container();
        app.stage.addChild(chartContainer);
        chartContainer.addChild(milkyWayContainer);
        chartContainer.addChild(territoryContainer);
        chartContainer.addChild(edgeContainer);
        chartContainer.addChild(starContainer);
        chartContainer.addChild(focusOverlay);
        chartContainer.addChild(calendarRimContainer);

        // MIG-019 §2E.2 SOLVE: progress markers visible to the user
        // (production builds disable DevTools, so console.log isn't
        // §2G.3f: silent loading per Eisa. The staged "Stage 1/4..."
        // text was diagnostic noise during a normal load; remove it.
        // Errors and the empty-universe case still surface a placeholder.
        // Detailed timing still goes to console.log for diagnostics.

        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            libraryPathsCached = libraryPaths;
            regionLayout = null;

            const layoutT0 = performance.now();
            layoutPoints = await fetchLayout(libraryPaths, 50);
            const layoutMs = Math.round(performance.now() - layoutT0);
            console.log(`[SightV3] layout: ${layoutPoints.length} points in ${layoutMs}ms`);

            if (layoutPoints.length === 0) {
                showPlaceholder('No notes in this universe yet.');
            } else {
                const idxT0 = performance.now();
                buildIndices();
                console.log(`[SightV3] buildIndices: ${resolvedEdges.length} unique edges in ${Math.round(performance.now() - idxT0)}ms`);

                const drawT0 = performance.now();
                fullRedraw();
                console.log(`[SightV3] fullRedraw: ${Math.round(performance.now() - drawT0)}ms`);

                hidePlaceholder();
            }

            // MIG-019 §2A+§2B redesign: background density-grid fetch.
            // Output-bounded (~256 KB) — universe size irrelevant.
            // Failures fall through to no Milky Way (graceful degradation).
            (async () => {
                const densT0 = performance.now();
                try {
                    const field = await fetchDensityField(libraryPaths, 50, 0.3);
                    const elapsed = Math.round(performance.now() - densT0);
                    console.log(`[SightV3 §2A+§2B] density grid fetched: ${field.width}×${field.height} cells, max=${field.max_value.toFixed(3)} in ${elapsed}ms`);
                    if (!app) return; // component unmounted while we waited
                    densityField = field;
                    drawMilkyWay();
                } catch (densErr) {
                    console.error('[SightV3 §2A+§2B] density grid fetch failed (Milky Way empty; chart still functional):', densErr);
                    densityField = null;
                }
            })();
        } catch (e) {
            errorMessage = String(e);
            console.error('[SightV3 §1E] layout fetch failed:', e);
            showPlaceholder(`Layout fetch failed: ${errorMessage}`, true);
        }
        isLoading = false;

        // §2G.3h: wheel listener wired via addEventListener so we can
        // pass `passive: false` and have preventDefault() actually
        // stop the page from scrolling / Ctrl+wheel from triggering
        // browser zoom. Svelte's `onwheel` attribute defaults to
        // passive in modern browsers, which silently swallows
        // preventDefault.
        canvasContainer.addEventListener('wheel', handleWheel, { passive: false });

        // §2G.3l: close-button wiring is back in the markup as
        // `onclick={fn}` per Svelte 5 docs. No more $effect / bind:this
        // / addEventListener dance. The previous patterns silently
        // dropped listeners.

        // Resize observer — picks up CSS-size changes (window resize,
        // sidebar collapse, etc.) and triggers a full redraw.
        resizeObserver = new ResizeObserver(() => {
            if (!isLoading && layoutPoints.length > 0) {
                fullRedraw();
            } else if (placeholderText && placeholderText.visible && app) {
                placeholderText.x = app.screen.width / 2;
                placeholderText.y = app.screen.height / 2;
            }
        });
        resizeObserver.observe(canvasContainer);

        // §2G.3g: visualViewport listener — browser zoom (Ctrl+/-)
        // changes devicePixelRatio without always firing a CSS-size
        // resize on the container. When that happens we still need
        // to re-render so HTML overlay positions (rim numbers, etc.)
        // stay aligned with the Pixi-drawn dome.
        if (typeof window !== 'undefined' && window.visualViewport) {
            const onZoom = () => {
                if (!app) return;
                // Re-set the resolution so Pixi renders at the new DPR.
                const dpr = Math.max(1, window.devicePixelRatio || 1);
                if (app.renderer.resolution !== dpr) {
                    app.renderer.resolution = dpr;
                    app.renderer.resize(app.screen.width, app.screen.height);
                }
                if (!isLoading && layoutPoints.length > 0) {
                    fullRedraw();
                }
            };
            window.visualViewport.addEventListener('resize', onZoom);
            // Save for cleanup in onDestroy.
            (visualViewportCleanup as { fn: (() => void) | null }).fn = () => {
                window.visualViewport?.removeEventListener('resize', onZoom);
            };
        }
    });

    /** §2G.3g: closure-captured cleanup ref for the visualViewport
     *  listener (set in onMount, called from onDestroy). */
    const visualViewportCleanup: { fn: (() => void) | null } = { fn: null };

    // Re-draw on projection toggle
    $effect(() => {
        const _projection = $appSettings.sight?.projection;
        if (!isLoading && layoutPoints.length > 0) {
            fullRedraw();
        }
    });

    // MIG-019 §2B: redraw Milky Way on visibility toggle.
    // Reading $appSettings.sight?.showMilkyWay subscribes the effect.
    $effect(() => {
        const _show = $appSettings.sight?.showMilkyWay;
        if (!isLoading && layoutPoints.length > 0) {
            drawMilkyWay();
        }
    });

    // MIG-019 §2C: redraw rim when the user enables/disables calendar
    // systems in Settings → Sight → Calendar systems. §2G dispatches
    // through `drawRimForMode()` so the right rim renders for the
    // active mode (calendar only matters in Time mode).
    $effect(() => {
        const _calendars = $appSettings.sight?.calendarSystems;
        if (!isLoading && layoutPoints.length > 0) {
            drawRimForMode();
        }
    });

    // MIG-019 §2E: search match propagation.
    // searchMatchIds is a Set<string> of lowercased note names from
    // SearchHub. Convert to Set<note_path> via the nameToPath map and
    // redraw the affected layers.
    $effect(() => {
        const ids = searchMatchIds;
        const newMatched = new Set<string>();
        if (ids && ids.size > 0) {
            for (const name of ids) {
                const path = nameToPath.get(name);
                if (path) newMatched.add(path);
            }
        }
        matchedPaths = newMatched;
        if (!isLoading && layoutPoints.length > 0) {
            drawTerritories();
            drawStars();
            drawFocusOverlay();
        }
    });

    // MIG-019 §2E: always-on labels toggle.
    // Reading $appSettings.sight?.alwaysOnLabels triggers re-render of
    // territories (since labels are rendered there).
    $effect(() => {
        const _alwaysLabels = $appSettings.sight?.alwaysOnLabels;
        if (!isLoading && layoutPoints.length > 0) {
            drawTerritories();
        }
    });

    // §2G.3k: REMOVED the redundant $effect that watched
    // chartZoom/Pan. It was firing on EVERY wheel notch (high-DPI
    // mice fire 100+ wheel events per second), each invocation
    // triggering Svelte reactivity scheduling, and flooding the main
    // thread to the point of an apparent freeze (Eisa report on
    // §2G.3j: "the app freezes. It is non-responsive"). Every wheel
    // / drag / reset / Esc handler now calls syncZoomTransform()
    // synchronously and that's enough — no belt-and-suspenders
    // needed at the cost of overhead.

    // §2G.3l: close-button $effect REMOVED. Was creating a new
    // handler reference each cycle and re-registering listeners on
    // every reactive update. Replaced with `onclick={fn}` in the
    // markup — the canonical Svelte 5 pattern.

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
            resizeObserver = null;
        }
        // §2G.3g: tear down the visualViewport zoom listener.
        if (visualViewportCleanup.fn) {
            visualViewportCleanup.fn();
            visualViewportCleanup.fn = null;
        }
        // §2G.3h: tear down the wheel listener.
        if (canvasContainer) {
            canvasContainer.removeEventListener('wheel', handleWheel);
        }
        if (app) {
            app.destroy(true, { children: true, texture: true });
            app = null;
        }
        milkyWayContainer = null;
        territoryContainer = null;
        edgeContainer = null;
        starContainer = null;
        focusOverlay = null;
        calendarRimContainer = null;
        placeholderText = null;
    });

    function handleEscape(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            e.preventDefault();
            // §2G.3m: simplified — Esc always closes Sight. User has
            // the Reset View button for view reset and can click empty
            // space to clear selection. Single-press close gives a
            // guaranteed escape hatch since the (×) button has had
            // intermittent failures.
            onClose();
        }
    }
</script>

<svelte:window onkeydown={handleEscape} />

<div class="sight-v3-root">
    <!-- §2G.3o: STRUCTURAL FIX — root is now `display: flex;
         flex-direction: column`, matching v2's `.sight2-root` exactly.
         Header is a REAL flex row (Row 1, fixed height); body fills
         the remaining height (Row 2). They do NOT overlap.
         Eight prior iterations failed because the close button was in
         a `position: absolute` strip overlaying the canvas — the
         button's `pointer-events: auto` + parent's `pointer-events:
         none` should have worked on paper, but click events were
         silently lost in some unknown event-routing path between
         the absolute button and the absolute canvas sibling.
         The v2 `ConstellationSight2.svelte` and `SkyView.svelte` have
         shipped working close buttons for months — both put the close
         button inside a NORMAL flex row that is a real layout
         participant, NOT an overlay. This commit copies that exact
         structure for v3. -->
    <div class="sight-v3-header">
        <span class="sight-v3-header-spacer"></span>
        <button class="sight-v3-close" onclick={() => onClose?.()}>×</button>
    </div>

    <!-- §2G.3o: body row — takes the remaining height. Acts as the
         positioning ancestor for the absolute canvas + reset-view +
         overlays-wrapper + tooltip. -->
    <div class="sight-v3-body">

    <!-- §2G.3i: Reset View button (chrome). Always visible per Eisa's
         directive. Faded when at default state, prominent when zoom
         or pan changes. -->
    <button
        type="button"
        class="sight-v3-reset-view"
        class:reset-active={chartZoom !== 1 || chartPanX !== 0 || chartPanY !== 0}
        onclick={resetView}
        aria-label={$t('sightV3.resetView') || 'Reset view'}
    >Reset view</button>

    <!-- §2G.3l: Pixi canvas — direct child of body, NOT inside a CSS-
         transformed wrapper. Pixi's `chartContainer.scale.set(zoom)`
         handles the zoom natively, keeping the canvas crisp at any
         zoom level (per Steve Ruiz's "Creating a Zoom UI" article and
         the pixi-viewport library: zoom by scaling a Container, not
         by CSS-scaling the <canvas> element).
         MDN: "Canvas is rendering to a bitmap of one size then scaling
         the bitmap to fit the CSS dimensions" — CSS-scaling the
         canvas would blur it. -->
    <div
        class="sight-v3-canvas"
        bind:this={canvasContainer}
        onpointerdown={handlePointerDown}
        onpointermove={handlePointerMove}
        onpointerleave={handlePointerLeave}
        onpointerup={handlePointerUp}
        onclick={handleClick}
        ondblclick={handleDoubleClick}
        role="application"
        aria-label="Constellation Sight v3"
    ></div>

    <!-- §2G.3l: HTML overlays wrapper. CSS-transform driven by the same
         chartZoom/Pan as the Pixi chartContainer, so HTML overlays
         (rim numbers, Universe Health, Universe-name, legend) zoom
         and pan in lockstep with the Pixi-rendered chart. The wrapper
         is `pointer-events: none` so clicks fall through to the
         canvas's pointer handlers below. -->
    <!-- §2G.3n: rim numbers REMOVED from this wrapper — they're now
         Pixi Text inside calendarRimContainer (a child of chartContainer)
         so they share the Pixi-side transform with the rim circles
         and no longer drift on zoom. The wrapper still scales the
         remaining HTML overlays (Universe Health, Universe-name,
         legend) which are screen-anchored UI rather than chart elements
         that need pixel-perfect rim alignment. -->
    <div
        class="sight-v3-overlays-wrapper"
        style="transform-origin: {overlaysTransformOrigin}; transform: {overlaysTransform};"
    >

    <!-- §2G.3f: Library legend panel. Anchored to the LEFT for LTR
         interfaces, RIGHT for RTL. Lists Universe root + numbered
         libraries with color swatches and note counts. -->
    {#if libraryColors.size > 0 && rimLabelGeometry.length > 0}
        <div class="sight-v3-legend" class:legend-rtl={detectDir(universeName) === 'rtl'}>
            <div class="sight-v3-legend-header" dir="auto">
                <div class="sight-v3-legend-universe-label">UNIVERSE</div>
                <div class="sight-v3-legend-universe-name">{universeName || 'Universe'}</div>
            </div>
            <div class="sight-v3-legend-divider"></div>
            <div class="sight-v3-legend-libs">
                {#each rimLabelGeometry as geo (geo.key)}
                    <div class="sight-v3-legend-row" dir="auto">
                        <span class="sight-v3-legend-num" style="background: {geo.colorCss};">{geo.index}</span>
                        <span class="sight-v3-legend-name" title={geo.name}>{geo.name}</span>
                        <span class="sight-v3-legend-count">{geo.count.toLocaleString()}</span>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    <!-- MIG-019 §2G (spec §4): Universe Health anchor — top-center,
         above the dome. Roundel at center, two metrics flanking on
         each side. Renders only when layout has loaded.
         (Metric labels are cartographic — kept in English uppercase
         like astronomy chart labels; i18n keys land in §2G.6.) -->
    {#if !isLoading && layoutPoints.length > 0}
        <div class="sight-v3-health-anchor" aria-hidden="false">
            <div class="sight-v3-health-caption">UNIVERSE HEALTH</div>
            <div class="sight-v3-health-row">
                <div class="sight-v3-metric">
                    <div class="sight-v3-metric-label">MODULARITY</div>
                    <div class="sight-v3-metric-value">{healthReport.modularity.display}</div>
                    <div class="sight-v3-metric-pill pill-{healthReport.modularity.status}">
                        {healthReport.modularity.status.toUpperCase()}
                    </div>
                </div>
                <div class="sight-v3-metric">
                    <div class="sight-v3-metric-label">DOMINANCE</div>
                    <div class="sight-v3-metric-value">{healthReport.dominance.display}</div>
                    <div class="sight-v3-metric-pill pill-{healthReport.dominance.status}">
                        {healthReport.dominance.status.toUpperCase()}
                    </div>
                </div>
                <div class="sight-v3-roundel" aria-label="Universe Health Score">
                    <div class="sight-v3-score">{Math.round(healthReport.score)}</div>
                    <div class="sight-v3-score-denom">/ 100</div>
                </div>
                <div class="sight-v3-metric">
                    <div class="sight-v3-metric-label">ENTROPY</div>
                    <div class="sight-v3-metric-value">{healthReport.entropy.display}</div>
                    <div class="sight-v3-metric-pill pill-{healthReport.entropy.status}">
                        {healthReport.entropy.status.toUpperCase()}
                    </div>
                </div>
                <div class="sight-v3-metric">
                    <div class="sight-v3-metric-label">CONNECTIVITY</div>
                    <div class="sight-v3-metric-value">{healthReport.connectivity.display}</div>
                    <div class="sight-v3-metric-pill pill-{healthReport.connectivity.status}">
                        {healthReport.connectivity.status.toUpperCase()}
                    </div>
                </div>
            </div>
        </div>

        <!-- §2G.3c: Universe name — above the dome, below the Universe
             Health card. `dir="auto"` so Arabic / Hebrew names render
             in their natural script direction. -->
        {#if universeName}
            <div class="sight-v3-universe-name" dir="auto">{universeName}</div>
        {/if}
    {/if}

    </div><!-- /.sight-v3-overlays-wrapper —§2G.3l -->

    <!-- §2G.3i: tooltip OUTSIDE the zoom wrapper. Inside, the wrapper's
         CSS transform breaks `position: fixed` (which becomes relative
         to the transformed ancestor instead of the viewport). Outside,
         it stays anchored to the cursor regardless of zoom/pan, with
         very high z-index so the title is always readable. -->
    {#if tooltipVisible && tooltipText}
        <div
            class="sight-v3-tooltip"
            style="left: {tooltipX}px; top: {tooltipY}px;"
            dir="auto"
        >{tooltipText}</div>
    {/if}

    </div><!-- /.sight-v3-body —§2G.3o -->

    <SightV3SidePanel
        notePath={selectedPath}
        noteTitle={sidePanelTitle}
        communityName={sidePanelCommunity}
        centralityRank={sidePanelRank}
        totalNotes={sidePanelTotalNotes}
        incomingCount={sidePanelIncoming}
        outgoingCount={sidePanelOutgoing}
        connectedNotes={sidePanelConnectedNotes}
        onOpenNote={sidePanelOpenNote}
        onConnectedClick={(path) => {
            // §2G.3n: clicking a connected note in the side panel
            // selects it (cascades the focus overlay to that node).
            const sky = pathToSkyNode.get(path);
            if (sky) onOpenNote(path, sky.libraryName);
        }}
        onClose={sidePanelClose}
    />
</div>

<style>
    /* MIG-019 §2G: Suwaidi cream parchment palette (was navy theme).
       §2G.3o: RESTORED `display: flex; flex-direction: column` on the
       root, matching v2's `.sight2-root` exactly. The header is the
       first flex row (fixed height); the body is the second (flex: 1).
       The canvas + overlays-wrapper sit INSIDE the body and are
       `position: absolute; inset: 0` relative to the body — they fill
       only the space below the header, never the header itself. The
       close button is a real flex-row child, not floating in an
       absolute strip over the canvas. This is the structural pattern
       that has shipped working in v2 + SkyView for months. */
    .sight-v3-root {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: #faf6e8;  /* cream parchment */
        z-index: 1000;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* §2G.3o: body row — takes remaining height after header. Acts as
       the positioning ancestor for the absolute canvas + reset button
       + overlays wrapper. */
    .sight-v3-body {
        flex: 1;
        position: relative;
        overflow: hidden;
        min-height: 0;  /* allow flex child to shrink below content size */
    }

    .sight-v3-canvas {
        position: absolute;
        inset: 0;
        cursor: default;
    }

    .sight-v3-canvas :global(canvas) {
        display: block;
        width: 100% !important;
        height: 100% !important;
    }

    /* §2G.3o: header bar — REAL FLEX ROW, no longer an absolute
       overlay. Lives as Row 1 of the root flex column; the body
       below it (canvas + overlays) lives as Row 2. They do not
       overlap. This matches v2's `.sight2-header` pattern verbatim
       (ConstellationSight2.svelte:1277-1285).

       Why this is the structural fix that finally works (per Svelte
       #15343 + Pixi #10911 audit findings, 2026-05-08):
       Svelte 5 delegates `onclick={fn}` to `<body>` and relies on
       the click event bubbling all the way up. When the button is
       in an absolute overlay sibling of a canvas wrapper that ALSO
       has onclick + pointerdown + pointermove + pointerup +
       dblclick handlers (all delegated to body), the canvas's
       handlers and Pixi v8's document-level capture-phase pointer
       listener can interfere with the click delivery to the button.
       Hover (CSS `:hover`) still fires because hover is NOT
       delegated. That exactly matches the symptom — gold-on-hover
       but no click — across eight prior iterations.

       The fix: put the close button in a clean DOM branch
       (.sight-v3-header → .sight-v3-close) that does NOT cross the
       canvas branch (.sight-v3-body → .sight-v3-canvas) on its
       bubble path to body. Structural separation > pointer-events
       gymnastics. */
    .sight-v3-header {
        flex-shrink: 0;          /* keeps header at natural height */
        height: 44px;
        display: flex;
        align-items: center;
        padding: 0 16px;
        background: rgba(250, 246, 232, 0.85);
        border-bottom: 1px solid rgba(26, 26, 26, 0.08);
    }
    .sight-v3-header-spacer {
        flex: 1;
    }
    .sight-v3-close {
        /* Matches v2 exactly: inline flex item, simple style. No
           position, no z-index, no pointer-events tricks. */
        border: none;
        background: transparent;
        cursor: pointer;
        font-size: 22px;
        line-height: 28px;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        color: rgba(26, 26, 26, 0.7);
        padding: 0;
        font-family: serif;
        font-weight: 600;
    }
    .sight-v3-close:hover {
        color: #1a1a1a;
        background: rgba(201, 162, 39, 0.35);
    }

    /* §2G.3l: HTML overlays wrapper. CSS-transform scales the rim
       numbers / Universe Health / Universe-name / legend in lockstep
       with the Pixi `chartContainer.scale` (Pixi-side handles the
       canvas, CSS handles HTML — both driven by the same chartZoom).
       `pointer-events: none` so clicks pass through to the canvas's
       handlers; individual overlays can re-enable if they need to
       capture clicks. */
    .sight-v3-overlays-wrapper {
        position: absolute;
        inset: 0;
        pointer-events: none;
        will-change: transform;
        z-index: 6;
    }

    /* §2G.3i: Reset View button — chrome (NOT inside zoom wrapper).
       Always visible per Eisa's directive ("I cannot see the reset
       button"); muted at default state, prominent when zoom or pan
       differs from canonical. */
    .sight-v3-reset-view {
        position: absolute;
        bottom: 28px;
        left: 24px;
        z-index: 999;
        background: rgba(250, 246, 232, 0.85);
        border: 1px solid rgba(26, 26, 26, 0.25);
        border-radius: 18px;
        padding: 8px 18px;
        font-family: serif;
        font-size: 13px;
        font-weight: 500;
        color: rgba(26, 26, 26, 0.5);
        cursor: pointer;
        opacity: 0.6;
        transition: opacity 200ms ease, background 200ms ease, color 200ms ease;
    }
    .sight-v3-reset-view.reset-active {
        opacity: 1;
        background: rgba(250, 246, 232, 0.95);
        border: 1.5px solid rgba(201, 162, 39, 0.85);
        color: #1a1a1a;
        font-weight: 600;
        box-shadow: 0 1px 3px rgba(26, 26, 26, 0.1);
    }
    .sight-v3-reset-view:hover {
        opacity: 1;
        background: rgba(201, 162, 39, 0.30);
        border-color: #c9a227;
        color: #1a1a1a;
    }
    .sight-v3-reset-view:active {
        transform: scale(0.96);
    }

    /* §2G.3c: Universe name header — above the dome, below the
       Universe Health metrics. Sits in the 50 px slot between
       y≈230 (metrics bottom) and y≈320 (dome top). */
    .sight-v3-universe-name {
        position: absolute;
        top: 248px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 8;
        font-family: serif;
        font-size: 22px;
        font-weight: 600;
        font-style: italic;
        letter-spacing: 0.06em;
        color: #2a4a8c;
        opacity: 0.95;
        max-width: 70vw;
        text-align: center;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        pointer-events: none;
        /* §2G.3h: cream backdrop so zoomed stars don't bleed through. */
        background: rgba(250, 246, 232, 0.93);
        padding: 4px 18px;
        border-radius: 10px;
    }

    /* §2G.3f: Library legend panel — anchored to the left edge for
       LTR interfaces, right edge for RTL. Lists Universe root +
       numbered libraries with color swatches. The number swatches
       on the rim point users to this panel for the actual names. */
    .sight-v3-legend {
        position: absolute;
        top: 270px;
        left: 24px;
        width: 248px;
        max-height: calc(100vh - 380px);
        z-index: 7;
        font-family: serif;
        background: rgba(250, 246, 232, 0.85);
        border: 1px solid rgba(26, 26, 26, 0.18);
        border-radius: 8px;
        padding: 14px 16px;
        backdrop-filter: blur(2px);
        overflow-y: auto;
        box-shadow: 0 1px 3px rgba(26, 26, 26, 0.05);
        /* §2G.3l: parent wrapper is pointer-events:none for click
           pass-through, but the legend itself shows native title
           tooltips on hover — so re-enable pointer events here. */
        pointer-events: auto;
    }
    .sight-v3-legend.legend-rtl {
        left: auto;
        right: 24px;
    }
    .sight-v3-legend-header {
        margin-bottom: 8px;
    }
    .sight-v3-legend-universe-label {
        font-size: 9px;
        letter-spacing: 2.5px;
        color: rgba(26, 26, 26, 0.55);
        margin-bottom: 4px;
    }
    .sight-v3-legend-universe-name {
        font-size: 14px;
        font-style: italic;
        font-weight: 600;
        color: #2a4a8c;
        line-height: 1.3;
        word-break: break-word;
    }
    .sight-v3-legend-divider {
        height: 1px;
        background: rgba(26, 26, 26, 0.15);
        margin: 10px 0;
    }
    .sight-v3-legend-libs {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .sight-v3-legend-row {
        display: grid;
        grid-template-columns: 24px 1fr auto;
        gap: 8px;
        align-items: center;
        font-size: 12px;
    }
    .sight-v3-legend-num {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        border-radius: 50%;
        color: #faf6e8;
        font-weight: 700;
        font-size: 11px;
        font-family: serif;
    }
    .sight-v3-legend-name {
        color: #1a1a1a;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .sight-v3-legend-count {
        color: rgba(26, 26, 26, 0.55);
        font-size: 10px;
        font-style: italic;
        white-space: nowrap;
    }

    /* MIG-019 §2G (spec §4): Universe Health anchor.
       Top-center, roundel + flanking metrics, blue-ink + gold accents.
       §2G.3h: cream backdrop + soft shadow so zoomed-in stars don't
       bleed through the metrics text (Eisa screenshot 2026-05-08). */
    .sight-v3-health-anchor {
        position: absolute;
        top: 16px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 8;
        display: flex;
        flex-direction: column;
        align-items: center;
        pointer-events: none;
        font-family: serif;
        background: rgba(250, 246, 232, 0.93);
        border: 1px solid rgba(26, 26, 26, 0.12);
        border-radius: 14px;
        padding: 10px 22px 14px;
        box-shadow: 0 1px 3px rgba(26, 26, 26, 0.05);
    }
    .sight-v3-health-caption {
        font-size: 11px;
        letter-spacing: 3px;
        color: rgba(26, 26, 26, 0.6);
        margin-bottom: 6px;
    }
    .sight-v3-health-row {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 36px;
    }
    .sight-v3-roundel {
        width: 100px;
        height: 100px;
        border: 2.5px solid #c9a227;
        border-radius: 50%;
        background: #faf6e8;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        line-height: 1;
    }
    .sight-v3-score {
        font-size: 38px;
        font-weight: 600;
        color: #c9a227;
        line-height: 1;
    }
    .sight-v3-score-denom {
        font-size: 10px;
        color: rgba(26, 26, 26, 0.6);
        margin-top: 4px;
    }
    .sight-v3-metric {
        display: flex;
        flex-direction: column;
        align-items: center;
        min-width: 96px;
    }
    .sight-v3-metric-label {
        font-size: 10px;
        letter-spacing: 2px;
        color: rgba(26, 26, 26, 0.55);
        margin-bottom: 4px;
        white-space: nowrap;
    }
    .sight-v3-metric-value {
        font-size: 22px;
        font-weight: 500;
        color: rgba(26, 26, 26, 0.92);
        line-height: 1.1;
    }
    .sight-v3-metric-pill {
        margin-top: 6px;
        padding: 3px 10px;
        border-radius: 8px;
        font-size: 9px;
        font-weight: 600;
        letter-spacing: 1.6px;
        border: 0.7px solid;
    }
    .pill-healthy {
        color: #3a8a4a;
        border-color: rgba(58, 138, 74, 0.6);
        background: rgba(58, 138, 74, 0.10);
    }
    .pill-caution {
        color: #c9831f;
        border-color: rgba(201, 131, 31, 0.6);
        background: rgba(201, 131, 31, 0.10);
    }
    .pill-imbalanced {
        color: #a83232;
        border-color: rgba(168, 50, 50, 0.6);
        background: rgba(168, 50, 50, 0.10);
    }

    .sight-v3-tooltip {
        position: fixed;
        background: rgba(250, 246, 232, 0.98);
        color: #1a1a1a;
        border: 1px solid rgba(201, 162, 39, 0.7);
        padding: 9px 12px;
        font-size: 13px;
        font-family: serif;
        border-radius: 6px;
        pointer-events: none;
        white-space: pre-line;
        /* §2G.3i: bumped 50 → 1500 so the tooltip sits above the side
           panel (50), the close button (1000), and everything else. */
        z-index: 1500;
        max-width: 320px;
        line-height: 1.4;
        box-shadow: 0 2px 6px rgba(26, 26, 26, 0.15);
    }
    /* First line of the tooltip is the note title — make it bold so
       it reads as a heading. The community/centrality lines that
       follow are plain text. */
    .sight-v3-tooltip::first-line {
        font-weight: 700;
        font-size: 14px;
    }
</style>

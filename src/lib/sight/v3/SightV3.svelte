<!--
    SightV3.svelte — the v3 star-chart Sight component.

    §2G.3q: Canvas 2D + D3-zoom renderer (replaces Pixi.js v8 which had
    document-level capture-phase pointer listeners that stole global
    click events, breaking the close button across 11 iterations).

    Architecture — immediate-mode Canvas 2D draw() pipeline:
      1. Clear + cream fill
      2. Apply d3-zoom transform (translate + scale)
      3. Draw layers in order: Milky Way → territories → edges →
         stars → focus overlay → rim
      4. Sync HTML overlay CSS transform for lockstep zoom/pan.

    Companion to: docs/Constellation-Sight-v3-Concept-Paper-v1.1.md.
    Plan ref:    lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-PLAN.md.
-->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import * as d3 from 'd3';
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

    // ─── DOM + Canvas 2D handles ────────────────────────────────────
    // §2G.3q: Pixi.js v8 → D3 + Canvas 2D migration. Pixi's EventSystem
    // registered document-level capture-phase pointer listeners that stole
    // global click events, breaking the close button across 11 iterations.
    // Canvas 2D has zero global event side effects. D3 provides zoom/pan.
    let canvasContainer: HTMLDivElement;
    let canvasEl: HTMLCanvasElement | null = null;
    let ctx: CanvasRenderingContext2D | null = null;
    let canvasDpr = 1;    // devicePixelRatio
    let canvasW = 0;      // CSS pixel width
    let canvasH = 0;      // CSS pixel height
    /** Offscreen canvas for the Milky Way density wash (built once per
     *  density-field fetch, drawn via ctx.drawImage each frame). */
    let milkyWayOffscreen: HTMLCanvasElement | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let placeholderMessage: string | null = null;
    let placeholderIsError = false;

    // ─── Data ────────────────────────────────────────────────────────
    let layoutPoints: LayoutPoint[] = $state([]);
    /** MIG-019 §2A+§2B redesign: 2D density field from TF-IDF
     *  (PJ-035 → Milky Way). 256×256 f32 grid; ~256 KB, input-size
     *  invariant. Rendered as a Canvas 2D drawImage from offscreen canvas. */
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

    // ─── §2G.3q: chart-level zoom + pan (d3-zoom) ─────────────────
    /** d3-zoom transform — replaces manual chartZoom/chartPanX/chartPanY.
     *  `t.k` = scale, `t.x` = translateX, `t.y` = translateY. */
    let currentTransform = $state(d3.zoomIdentity);
    const MIN_ZOOM = 0.4;
    const MAX_ZOOM = 5.0;
    let zoomBehavior: d3.ZoomBehavior<HTMLCanvasElement, unknown> | null = null;
    /** True if d3-zoom is in an active drag gesture. Used to suppress
     *  hover tooltips during pan and suppress the click after drag. */
    let isDragging = false;
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

    /** §2G.3q: HTML overlays CSS transform — driven by d3-zoom's
     *  currentTransform so HTML legends / health / universe-name scale
     *  in lockstep with the Canvas 2D chart. */
    let overlaysTransform = $state('none');
    let overlaysTransformOrigin = $state('0 0');

    function syncOverlaysTransform() {
        const t = currentTransform;
        if (t.k === 1 && t.x === 0 && t.y === 0) {
            overlaysTransform = 'none';
        } else {
            overlaysTransformOrigin = '0 0';
            overlaysTransform = `translate3d(${t.x}px, ${t.y}px, 0) scale3d(${t.k}, ${t.k}, 1)`;
        }
    }

    function resetView() {
        if (!canvasEl || !zoomBehavior) return;
        d3.select(canvasEl).transition().duration(300)
            .call(zoomBehavior.transform, d3.zoomIdentity);
    }

    /** Convert a hex integer (0xd4af37) to a CSS string ('#d4af37'). */
    function hexInt(n: number): string {
        return '#' + n.toString(16).padStart(6, '0');
    }

    function getViewport() {
        const w = canvasW;
        const h = canvasH;
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
        const mctx: ModeContext = {
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
            mctx.notePath = pt.note_path;
            mctx.centralityRank = pathToCentralityRank.get(pt.note_path) ?? 0.5;
            mctx.linkCount = sky?.linkCount ?? 0;
            mctx.outgoingCount = sky?.outgoingCount ?? 0;
            mctx.createdAt = (sky?.createdAt ?? null) as number | null;
            mctx.modifiedAt = (sky?.createdAt ?? null) as number | null;  // see buildIndices note
            mctx.embedAngleRad = pathToEmbedAngle.get(pt.note_path) ?? 0;
            mctx.regionLayout = regionLayout;
            mctx.stats = modeStats;

            const pos: ModePosition = positionForMode(currentMode, mctx);
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
        if (!ctx) return;

        const enabled = ($appSettings.sight?.calendarSystems ?? ['gregorian']) as CalendarSystem[];
        if (enabled.length === 0) return;

        const vp = getViewport();
        if (!vp) return;
        const domeRadius = vp.radius;
        const rimViewport = {
            cx: vp.cx,
            cy: vp.cy,
            innerRadius: domeRadius + 4,
            outerRadius: domeRadius + 4 + 22 * enabled.length,
        };

        const locale = get(i18nLocale) ?? 'en';
        monthSegments = monthArcSegments(rimViewport, enabled, locale);

        const currentGregorianMonth = new Date().getMonth();

        // Arc dividers
        ctx.beginPath();
        for (const seg of monthSegments) {
            const x1 = rimViewport.cx + seg.rIn * Math.cos(seg.startAngle);
            const y1 = rimViewport.cy + seg.rIn * Math.sin(seg.startAngle);
            const x2 = rimViewport.cx + seg.rOut * Math.cos(seg.startAngle);
            const y2 = rimViewport.cy + seg.rOut * Math.sin(seg.startAngle);
            ctx.moveTo(x1, y1);
            ctx.lineTo(x2, y2);
        }
        // Outer + inner ring boundaries
        for (let r = 0; r <= enabled.length; r++) {
            const radius = rimViewport.innerRadius + r * 22;
            ctx.moveTo(rimViewport.cx + radius, rimViewport.cy);
            ctx.arc(rimViewport.cx, rimViewport.cy, radius, 0, Math.PI * 2);
        }
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.35)';
        ctx.lineWidth = 1;
        ctx.stroke();

        // Month labels
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        for (const seg of monthSegments) {
            const isCurrent = seg.calendar === 'gregorian' && seg.monthIndex === currentGregorianMonth;
            const isHovered = seg.calendar === 'gregorian' && seg.monthIndex === hoveredMonth;
            const isFiltered = seg.calendar === 'gregorian'
                && monthFilterMonth === seg.monthIndex
                && monthFilterPersistent;
            const alpha = isFiltered ? 1.0 : isHovered ? 0.95 : isCurrent ? 0.9 : 0.65;
            const fontSize = isCurrent || isFiltered ? 11 : 10;
            ctx.font = `${fontSize}px system-ui, -apple-system, sans-serif`;
            ctx.fillStyle = isCurrent || isFiltered ? `rgba(212, 175, 55, ${alpha})` : `rgba(245, 230, 200, ${alpha})`;
            ctx.fillText(seg.label, seg.labelX, seg.labelY);
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
     *  Renders the Rust-side density grid as a Canvas 2D drawImage.
     *
     *  The grid (256×256 f32) was accumulated by rasterizing every
     *  high-similarity pair into grid space and then Gaussian-blurring
     *  to smooth into a continuous band texture. We build an RGBA8
     *  ImageData from the grid (Suwaidi cream tinted; alpha proportional
     *  to normalized cell value) onto an offscreen canvas and blit it
     *  each frame. Single draw call — universe size irrelevant. */
    /** Build the offscreen Milky Way canvas from the density field.
     *  Called once per density-field fetch; drawn via drawMilkyWay(). */
    function buildMilkyWayImage() {
        if ($appSettings.sight?.showMilkyWay === false) {
            milkyWayOffscreen = null;
            return;
        }
        if (!densityField || densityField.max_value <= 0) {
            milkyWayOffscreen = null;
            return;
        }
        const w = densityField.width;
        const h = densityField.height;
        const maxV = densityField.max_value || 1.0;
        const offCanvas = document.createElement('canvas');
        offCanvas.width = w;
        offCanvas.height = h;
        const offCtx = offCanvas.getContext('2d');
        if (!offCtx) return;
        const imgData = offCtx.createImageData(w, h);
        const rgba = imgData.data;
        for (let i = 0; i < w * h; i++) {
            const v = Math.sqrt(Math.max(0, densityField.values[i] / maxV));
            rgba[i * 4 + 0] = 0xf5;
            rgba[i * 4 + 1] = 0xe6;
            rgba[i * 4 + 2] = 0xc8;
            rgba[i * 4 + 3] = Math.round(v * 110);
        }
        offCtx.putImageData(imgData, 0, 0);
        milkyWayOffscreen = offCanvas;
    }

    function drawMilkyWay() {
        if (!ctx || !milkyWayOffscreen) return;
        const vp = getViewport();
        if (!vp) return;
        const domeR = vp.radius;
        ctx.drawImage(milkyWayOffscreen, vp.cx - domeR, vp.cy - domeR, domeR * 2, domeR * 2);
    }

    /** Draw community territory polygons on the base layer.
     *  - MIG-019 §2E search-active path: territories with matches get
     *    halo (border thickens + brightens); territories without dim.
     *  - MIG-019 §2E always-on labels: render community label at the
     *    centroid when $appSettings.sight.alwaysOnLabels is true. */
    function drawTerritories() {

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
     *  Cap rendered edges at MAX_FAINT_EDGES_AT_REST to bound draw cost
     *  on large universes (Boss-scale: 656k links).
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
        // Resting state — no edges. The focus overlay handles active state.
    }

    /** Draw stars on the base layer.
     *
     *  §2G.3q Canvas 2D: draws each star as a filled circle with a thin
     *  contrast stroke. Immediate-mode rendering — no GPU buffer
     *  allocations (the root cause of the 3-minute Pixi v8 render).
     *
     *  - MIG-019 §2C: dim stars failing the month filter.
     *  - MIG-019 §2E: search-active path — matched stars flare (size +
     *    brightness boost), non-matched dim heavily. */
    function drawStars() {
        if (!ctx) return;
        const searchOn = isSearchActive();
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

            const wedge = regionLayout?.pathToWedge.get(pt.note_path);
            const libColor = wedge ? libraryColors.get(wedge.libraryPath) : undefined;
            const fillCss = libColor?.css ?? '#1a1a1a';

            // Fill
            ctx.beginPath();
            ctx.arc(screen.x, screen.y, radius, 0, Math.PI * 2);
            ctx.fillStyle = fillCss;
            ctx.globalAlpha = alpha;
            ctx.fill();
            // Thin contrast stroke
            ctx.strokeStyle = '#1a1a1a';
            ctx.lineWidth = 0.6;
            ctx.globalAlpha = alpha * 0.85;
            ctx.stroke();
        }
        ctx.globalAlpha = 1.0;
    }

    /** Redraw the focus overlay based on hoveredPath / selectedPath /
     *  search-active state.
     *  - No state → empty overlay (or search-match edges if search active).
     *  - hoveredPath (no selection) → brighten edges incident to that star.
     *  - selectedPath → brighten ALL edges within that star's community.
     *  - searchActive (no specific focus) → brighten edges between matched stars.
     *  Draws AFTER drawX so it sits on top. */
    /** §2G.3q Canvas 2D: draw focus overlay (edges + rings for
     *  hovered/selected star). Called AFTER drawStars in the draw()
     *  pipeline so it renders on top of the base layer. */
    function drawFocusOverlay() {
        if (!ctx) return;

        const focusPath = selectedPath ?? hoveredPath;
        const searchOn = isSearchActive();

        // MIG-019 §2E: if no focus star but search is active, brighten
        // edges between matched stars so the user sees the search-result
        // sub-graph clearly.
        if (!focusPath && searchOn) {
            ctx.beginPath();
            let edgeCount = 0;
            for (const e of resolvedEdges) {
                if (matchedPaths.has(e.a) && matchedPaths.has(e.b)) {
                    const sa = pathToScreen.get(e.a);
                    const sb = pathToScreen.get(e.b);
                    if (!sa || !sb) continue;
                    ctx.moveTo(sa.x, sa.y);
                    ctx.lineTo(sb.x, sb.y);
                    edgeCount++;
                }
            }
            if (edgeCount > 0) {
                ctx.strokeStyle = '#c9a227';
                ctx.globalAlpha = 0.7;
                ctx.lineWidth = 1.0;
                ctx.stroke();
                ctx.globalAlpha = 1.0;
            }
            return;
        }

        if (!focusPath) return;

        const focusScreen = pathToScreen.get(focusPath);
        if (!focusScreen) return;

        // §2G.3h: edges incident to the FOCUSED NODE only.
        const MAX_FOCUS_EDGES = 50;
        ctx.beginPath();
        let edgeCount = 0;
        const neighbours = new Set<string>();
        for (const e of resolvedEdges) {
            if (edgeCount >= MAX_FOCUS_EDGES) break;
            if (e.a !== focusPath && e.b !== focusPath) continue;
            const sa = pathToScreen.get(e.a);
            const sb = pathToScreen.get(e.b);
            if (!sa || !sb) continue;
            ctx.moveTo(sa.x, sa.y);
            ctx.lineTo(sb.x, sb.y);
            edgeCount++;
            neighbours.add(e.a === focusPath ? e.b : e.a);
        }
        if (edgeCount > 0) {
            ctx.strokeStyle = '#1a1a1a';
            ctx.globalAlpha = 0.7;
            ctx.lineWidth = 1.0;
            ctx.stroke();
        }

        // Neighbour rings
        if (neighbours.size > 0) {
            for (const npath of neighbours) {
                const ns = pathToScreen.get(npath);
                if (!ns) continue;
                const r = actualNodeRadius(ns.r) + 0.8;
                ctx.beginPath();
                ctx.arc(ns.x, ns.y, r, 0, Math.PI * 2);
                ctx.strokeStyle = '#c9a227';
                ctx.globalAlpha = 0.85;
                ctx.lineWidth = 0.8;
                ctx.stroke();
            }
        }

        // Focus ring on the selected/hovered star
        const focusR = actualNodeRadius(focusScreen.r);
        ctx.beginPath();
        ctx.arc(focusScreen.x, focusScreen.y, focusR + 1, 0, Math.PI * 2);
        ctx.strokeStyle = '#c9a227';
        ctx.globalAlpha = 1.0;
        ctx.lineWidth = 1.5;
        ctx.stroke();
        ctx.globalAlpha = 1.0;
    }

    /** Find the nearest star within `r=10` of (px, py). Returns the
     *  note_path or null. O(n) iteration; for 30k stars this is ~1ms. */
    /** §2G.3q Canvas 2D: find the nearest star within hit radius of
     *  (px, py) in CSS coordinates. Uses d3-zoom's `currentTransform
     *  .invert()` to map screen → data space. O(n) scan — ~1ms for 7k. */
    function pickStar(px: number, py: number): string | null {
        // Invert the d3-zoom transform to get canonical (data-space) coords.
        const [cpx, cpy] = currentTransform.invert([px, py]);

        const HOVER_RADIUS = 10;
        let best: string | null = null;
        let bestDist = HOVER_RADIUS * HOVER_RADIUS;
        for (const [path, s] of pathToScreen) {
            const dx = s.x - cpx;
            const dy = s.y - cpy;
            const d2 = dx * dx + dy * dy;
            const effectiveR = s.r + 4;
            const hitR2 = d2 <= effectiveR * effectiveR ? d2 : Infinity;
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
    /** §2G.3q Canvas 2D: dispatch rim rendering by active mode. */
    function drawRimForMode() {
        if (!ctx) return;
        if (currentMode === 'regions') {
            drawRegionRim();
        } else if (currentMode === 'time') {
            drawCalendarRim();
            rimLabelGeometry = [];
        } else {
            rimLabelGeometry = [];
        }
    }

    /** MIG-019 §2G.3: Regions-mode rim. Library wedges sized by note
     *  count (largest first), with tangential blue-ink labels.
     *  §2G.3b: shares the same viewport as star positioning so the rim
     *  is concentric with the dome (not centered on the raw canvas). */
    /** §2G.3q Canvas 2D: Regions-mode rim. Library wedges sized by note
     *  count (largest first), with tangential labels drawn on canvas. */
    function drawRegionRim() {
        if (!ctx || !regionLayout || regionLayout.wedges.length === 0) return;

        const vp = getViewport();
        if (!vp) return;
        const cx = vp.cx;
        const cy = vp.cy;
        const domeR = vp.radius;
        const rimInner = domeR + 18;
        const rimOuter = domeR + 70;

        // Outer rim circle
        ctx.beginPath();
        ctx.arc(cx, cy, rimOuter, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(26, 26, 26, 0.55)';
        ctx.lineWidth = 0.8;
        ctx.stroke();
        // Inner rim circle
        ctx.beginPath();
        ctx.arc(cx, cy, rimInner, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(26, 26, 26, 0.35)';
        ctx.lineWidth = 0.5;
        ctx.stroke();
        // Dome edge
        ctx.beginPath();
        ctx.arc(cx, cy, domeR, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(26, 26, 26, 0.4)';
        ctx.lineWidth = 0.7;
        ctx.stroke();

        // Wedge dividers
        ctx.beginPath();
        for (const wedge of regionLayout.wedges) {
            const tStart = wedge.arcStartRad;
            const dx0 = (rimInner - 2) * Math.sin(tStart);
            const dy0 = -(rimInner - 2) * Math.cos(tStart);
            const dx1 = (rimOuter + 4) * Math.sin(tStart);
            const dy1 = -(rimOuter + 4) * Math.cos(tStart);
            ctx.moveTo(cx + dx0, cy + dy0);
            ctx.lineTo(cx + dx1, cy + dy1);
        }
        ctx.strokeStyle = 'rgba(26, 26, 26, 0.55)';
        ctx.lineWidth = 0.7;
        ctx.stroke();

        // Rim numbers — drawn directly on the canvas so they scale
        // with the d3-zoom transform (no CSS-vs-Canvas drift).
        const labelR = (rimInner + rimOuter) / 2 + 2;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.font = '700 16px serif';
        for (const wedge of regionLayout.wedges) {
            const lc = libraryColors.get(wedge.libraryPath);
            if (!lc) continue;
            const t = wedge.arcMidRad;
            const lx = cx + labelR * Math.sin(t);
            const ly = cy - labelR * Math.cos(t);
            // Cream halo for legibility
            ctx.strokeStyle = '#faf6e8';
            ctx.lineWidth = 3;
            ctx.globalAlpha = 1.0;
            ctx.strokeText(String(lc.index), lx, ly);
            // Colored number
            ctx.fillStyle = lc.css;
            ctx.globalAlpha = 1.0;
            ctx.fillText(String(lc.index), lx, ly);
        }

        // Legend metadata (colors + names + counts for the HTML panel).
        rimLabelGeometry = regionLayout.wedges.map((wedge) => {
            const lc = libraryColors.get(wedge.libraryPath);
            return {
                key: wedge.libraryPath,
                name: wedge.libraryName,
                index: lc?.index ?? 0,
                colorCss: lc?.css ?? '#2a4a8c',
                count: wedge.noteCount,
                lx: 0, ly: 0, rotDeg: 0,
            };
        });
    }

    /** §2G.3q Canvas 2D: unified draw pipeline. Clears the canvas,
     *  applies the d3-zoom transform, then draws each layer in order.
     *  Called on every state change (zoom, pan, hover, select, resize). */
    function draw() {
        if (!ctx || !canvasEl) return;
        const w = canvasW;
        const h = canvasH;
        if (w === 0 || h === 0) return;

        recomputeScreenPositions();

        // Clear entire canvas (in device-pixel space).
        ctx.save();
        ctx.setTransform(canvasDpr, 0, 0, canvasDpr, 0, 0);
        ctx.clearRect(0, 0, w, h);
        // Fill cream background.
        ctx.fillStyle = '#faf6e8';
        ctx.fillRect(0, 0, w, h);
        ctx.restore();

        // Apply d3-zoom transform so all drawing is in data space.
        const t = currentTransform;
        ctx.save();
        ctx.setTransform(
            canvasDpr * t.k, 0,
            0, canvasDpr * t.k,
            canvasDpr * t.x, canvasDpr * t.y,
        );

        // Layer order: Milky Way → territories → edges → stars →
        // focus overlay → rim. Same order as the old Pixi containers.
        drawMilkyWay();
        drawTerritories();
        drawEdges();
        drawStars();
        drawFocusOverlay();
        drawRimForMode();

        // Placeholder text (if no stars).
        if (placeholderMessage) {
            ctx.font = '18px system-ui, -apple-system, sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = placeholderIsError ? '#a83232' : '#1a1a1a';
            ctx.globalAlpha = 1.0;
            ctx.fillText(placeholderMessage, w / 2, h / 2);
        }

        ctx.restore();

        // Keep HTML overlays in sync with the zoom transform.
        syncOverlaysTransform();
    }

    /** Legacy alias — some $effects still call fullRedraw(). */
    function fullRedraw() { draw(); }

    /** §2G.3q: placeholder is now a simple string rendered by draw(). */
    function showPlaceholder(message: string, isError: boolean = false) {
        placeholderMessage = message;
        placeholderIsError = isError;
        draw();
    }

    function hidePlaceholder() {
        placeholderMessage = null;
        draw();
    }

    // ─── Pointer handlers ────────────────────────────────────────────
    // §2G.3q: Pan and zoom are handled by d3-zoom (wheel, drag, touch).
    // These handlers deal with hover, click, double-click, and rim
    // hit-testing only. d3-zoom's drag detection sets `isDragging` so
    // we can suppress click-after-pan.

    /** Get rim viewport for hit-testing. */
    function getRimViewport() {
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
        // Suppress hover during d3-zoom drag.
        if (isDragging) {
            tooltipVisible = false;
            return;
        }
        const rect = (canvasEl ?? canvasContainer).getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

        // Rim hit-test only in Time mode.
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
                        }
                        draw();
                    }
                    tooltipVisible = false;
                    return;
                } else if (hoveredMonth !== -1) {
                    hoveredMonth = -1;
                    if (!monthFilterPersistent) {
                        monthFilterMonth = -1;
                    }
                    draw();
                }
            }
        }

        const nearest = pickStar(px, py);
        if (nearest !== hoveredPath) {
            hoveredPath = nearest;
            draw();
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
            draw();
        }
        if (currentMode === 'time' && hoveredMonth !== -1) {
            hoveredMonth = -1;
            if (!monthFilterPersistent) {
                monthFilterMonth = -1;
            }
            draw();
        }
        tooltipVisible = false;
    }

    function handleClick(ev: MouseEvent) {
        // Suppress click after d3-zoom pan gesture.
        if (panDragMoved) {
            panDragMoved = false;
            return;
        }
        const rect = (canvasEl ?? canvasContainer).getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

        // Rim click — Time mode only.
        if (currentMode === 'time') {
            const rimVp = getRimViewport();
            if (rimVp) {
                const monthHit = pickMonth(rimVp, px, py, rimVp.enabled);
                if (monthHit) {
                    const greg = gregorianMonthFromSegment(monthHit);
                    if (greg !== null) {
                        if (monthFilterPersistent && monthFilterMonth === greg) {
                            monthFilterPersistent = false;
                            monthFilterMonth = -1;
                        } else {
                            monthFilterPersistent = true;
                            monthFilterMonth = greg;
                        }
                        draw();
                        return;
                    }
                }
            }
        }

        const nearest = pickStar(px, py);
        if (nearest) {
            selectedPath = nearest;
            draw();
        } else {
            if (selectedPath !== null) {
                selectedPath = null;
            }
            if (currentMode === 'time' && monthFilterPersistent) {
                monthFilterPersistent = false;
                monthFilterMonth = -1;
            }
            draw();
        }
    }

    function handleDoubleClick(ev: MouseEvent) {
        const rect = (canvasEl ?? canvasContainer).getBoundingClientRect();
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
        // §2G.3q: Canvas 2D initialization — replaces Pixi.js Application.
        // Zero global event side effects (the root cause of the 11-iteration
        // close-button failure). D3 provides zoom/pan via d3-zoom.
        const canvas = document.createElement('canvas');
        canvasContainer.appendChild(canvas);
        canvasEl = canvas;
        ctx = canvas.getContext('2d');
        if (!ctx) {
            console.error('[SightV3] Failed to get Canvas 2D context');
            return;
        }

        // Size the canvas to fill the container at device pixel ratio.
        function sizeCanvas() {
            if (!canvasEl || !ctx) return;
            const rect = canvasContainer.getBoundingClientRect();
            canvasW = rect.width;
            canvasH = rect.height;
            canvasDpr = Math.max(1, window.devicePixelRatio || 1);
            canvasEl.width = Math.round(canvasW * canvasDpr);
            canvasEl.height = Math.round(canvasH * canvasDpr);
            canvasEl.style.width = canvasW + 'px';
            canvasEl.style.height = canvasH + 'px';
        }
        sizeCanvas();

        // d3-zoom: handles wheel zoom, drag pan, touch pinch. Replaces
        // the manual chartZoom/chartPanX/chartPanY + handleWheel +
        // handlePointerDown/Up drag logic. `passive: false` on the
        // wheel listener so preventDefault blocks browser zoom.
        zoomBehavior = d3.zoom<HTMLCanvasElement, unknown>()
            .scaleExtent([MIN_ZOOM, MAX_ZOOM])
            .on('start', (event) => {
                if (event.sourceEvent?.type === 'mousedown') {
                    isDragging = false;
                    panDragMoved = false;
                }
            })
            .on('zoom', (event) => {
                currentTransform = event.transform;
                // Detect drag (pan) to suppress click after.
                if (event.sourceEvent?.type === 'mousemove') {
                    isDragging = true;
                    panDragMoved = true;
                    tooltipVisible = false;
                }
                draw();
            })
            .on('end', () => {
                isDragging = false;
            });

        const sel = d3.select(canvasEl);
        sel.call(zoomBehavior);
        // Filter: allow double-click for star open (not zoom reset).
        sel.on('dblclick.zoom', null);

        // Data fetch + initial draw.
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
                draw();
                console.log(`[SightV3] draw: ${Math.round(performance.now() - drawT0)}ms`);

                hidePlaceholder();
            }

            // Background density-grid fetch for Milky Way.
            (async () => {
                const densT0 = performance.now();
                try {
                    const field = await fetchDensityField(libraryPaths, 50, 0.3);
                    const elapsed = Math.round(performance.now() - densT0);
                    console.log(`[SightV3 §2A+§2B] density grid fetched: ${field.width}×${field.height} cells, max=${field.max_value.toFixed(3)} in ${elapsed}ms`);
                    if (!canvasEl) return; // component unmounted while we waited
                    densityField = field;
                    buildMilkyWayImage();
                    draw();
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

        // Resize observer — picks up CSS-size changes and resizes canvas.
        resizeObserver = new ResizeObserver(() => {
            sizeCanvas();
            if (!isLoading && layoutPoints.length > 0) {
                draw();
            } else if (placeholderMessage) {
                draw();
            }
        });
        resizeObserver.observe(canvasContainer);

        // visualViewport listener — browser zoom changes DPR.
        if (typeof window !== 'undefined' && window.visualViewport) {
            const onBrowserZoom = () => {
                sizeCanvas();
                if (!isLoading && layoutPoints.length > 0) {
                    draw();
                }
            };
            window.visualViewport.addEventListener('resize', onBrowserZoom);
            (visualViewportCleanup as { fn: (() => void) | null }).fn = () => {
                window.visualViewport?.removeEventListener('resize', onBrowserZoom);
            };
        }
    });

    /** §2G.3g: closure-captured cleanup ref for the visualViewport
     *  listener (set in onMount, called from onDestroy). */
    const visualViewportCleanup: { fn: (() => void) | null } = { fn: null };

    // Re-draw on projection toggle.
    $effect(() => {
        const _projection = $appSettings.sight?.projection;
        if (!isLoading && layoutPoints.length > 0) {
            draw();
        }
    });

    // §2B: redraw on Milky Way visibility toggle.
    $effect(() => {
        const _show = $appSettings.sight?.showMilkyWay;
        if (!isLoading && layoutPoints.length > 0) {
            buildMilkyWayImage();
            draw();
        }
    });

    // §2C: redraw rim on calendar systems change.
    $effect(() => {
        const _calendars = $appSettings.sight?.calendarSystems;
        if (!isLoading && layoutPoints.length > 0) {
            draw();
        }
    });

    // §2E: search match propagation.
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
            draw();
        }
    });

    // §2E: always-on labels toggle.
    $effect(() => {
        const _alwaysLabels = $appSettings.sight?.alwaysOnLabels;
        if (!isLoading && layoutPoints.length > 0) {
            draw();
        }
    });

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
            resizeObserver = null;
        }
        if (visualViewportCleanup.fn) {
            visualViewportCleanup.fn();
            visualViewportCleanup.fn = null;
        }
        // §2G.3q: tear down d3-zoom listeners on the canvas.
        if (canvasEl && zoomBehavior) {
            d3.select(canvasEl).on('.zoom', null);
        }
        // Remove canvas from DOM.
        if (canvasEl && canvasEl.parentElement) {
            canvasEl.parentElement.removeChild(canvasEl);
        }
        canvasEl = null;
        ctx = null;
        milkyWayOffscreen = null;
        zoomBehavior = null;
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
    <!-- §2G.3p: Close button REMOVED from SightV3. It now lives in
         +layout.svelte as an external fixed-position button that directly
         sets `sightV3Active = false` — replicating SkyView's star-close
         pattern (line 5108). Ten prior iterations failed trying to make
         a close button work INSIDE this component (Svelte 5 delegation +
         Pixi v8 capture interference). The Esc handler below still works
         as a keyboard escape hatch. -->

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
        class:reset-active={currentTransform.k !== 1 || currentTransform.x !== 0 || currentTransform.y !== 0}
        onclick={resetView}
        aria-label={$t('sightV3.resetView') || 'Reset view'}
    >Reset view</button>

    <!-- §2G.3q: Canvas 2D — d3-zoom handles zoom/pan transforms
         directly on the canvas context. The <canvas> element is
         created by onMount inside this container. -->
    <!-- §2G.3q: Canvas 2D container. d3-zoom handles wheel + drag
         on the <canvas> element created in onMount. Pointer-move,
         click, double-click are wired here for star interaction.
         No onpointerdown/onpointerup — d3-zoom owns the drag. -->
    <div
        class="sight-v3-canvas"
        bind:this={canvasContainer}
        onpointermove={handlePointerMove}
        onpointerleave={handlePointerLeave}
        onclick={handleClick}
        ondblclick={handleDoubleClick}
        role="application"
        aria-label="Constellation Sight v3"
    ></div>

    <!-- §2G.3q: HTML overlays wrapper. CSS-transform driven by d3-zoom's
         currentTransform so HTML overlays (Universe Health, Universe-name,
         legend) zoom and pan in lockstep with the Canvas 2D chart.
         `pointer-events: none` so clicks fall through to the canvas. -->
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

    /* §2G.3q: Canvas 2D child — sized by JS (sizeCanvas) to match
       container at device pixel ratio. No Pixi :global(canvas) needed. */
    .sight-v3-canvas {
        position: absolute;
        inset: 0;
        cursor: default;
    }

    .sight-v3-canvas :global(canvas) {
        display: block;
    }

    /* §2G.3p: header + close button REMOVED from SightV3 — they now
       live in +layout.svelte as an external fixed-position button
       (class .sight-v3-ext-close) that directly sets sightV3Active.
       Replicates SkyView's star-close pattern. */

    /* §2G.3q: HTML overlays wrapper. CSS-transform scales the
       Universe Health / Universe-name / legend in lockstep with the
       Canvas 2D d3-zoom transform. `pointer-events: none` so clicks
       pass through to the canvas's handlers; individual overlays
       can re-enable if they need to capture clicks. */
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

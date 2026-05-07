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

    /** §2G.3b: rim label geometry, published by `drawRegionRim`.
     *  Rendered as HTML elements (with `dir="auto"`) in the Svelte
     *  template so Unicode bidi shaping works natively for Arabic /
     *  Hebrew / mixed-script library names. */
    interface RimLabelGeo {
        key: string;
        name: string;
        count: number;
        /** Library name label position. */
        lx: number;
        ly: number;
        /** Note-count caption position. */
        cxLabel: number;
        cyLabel: number;
        /** Rotation in degrees, already flipped for the lower half. */
        rotDeg: number;
        /** §2G.3c: Tangential chord length at the label radius (px),
         *  used as max-width so labels can't bleed into adjacent
         *  wedges. Long names ellipsize; short names breathe. */
        maxWidthPx: number;
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
                // §2G.3e: small per-note rank jitter (±1.5 %) so notes
                // with the same centrality rank don't form perfect
                // concentric rings. Hash on a different salt than azimuth.
                const baseRank = i / denom;
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
        const MIN_DIST = 6;       // px between star centers
        const MAX_ITER = 6;
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
        for (const pt of layoutPoints) {
            const screen = pathToScreen.get(pt.note_path);
            if (!screen) continue;
            const passesMonth = passesMonthFilter(pt.note_path);
            const passesSrc = passesSearch(pt.note_path);
            // §2G.3d: base alpha is now mode-aware (computed in
            // positionForMode and stored in pathToScreen). Search /
            // month filter overrides still apply.
            const baseAlpha = screen.baseAlpha;

            let alpha: number;
            let radius = screen.r;
            if (searchOn) {
                if (passesSrc && passesMonth) {
                    radius = screen.r * 1.5;
                    alpha = 1.0;
                } else {
                    alpha = 0.10;
                }
            } else if (!passesMonth) {
                alpha = 0.15;
            } else {
                alpha = baseAlpha;
            }

            // MIG-019 §2G: stars are near-black (#1a1a1a) on cream
            // parchment, per the Suwaidi star-chart palette.  Magnitude
            // alpha already encodes brightness; modulating it here keeps
            // the search-active dim/flare semantics intact.
            // Each circle gets its own fill() call so per-star alpha
            // is preserved. Pixi v8 batches all subpaths into one GL
            // draw call regardless.
            stars.circle(screen.x, screen.y, radius);
            stars.fill({ color: 0x1a1a1a, alpha });
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

        // Decide which edges to brighten:
        //  - Selected: all edges where both ends are in the same community.
        //  - Hovered (no selection): edges incident to the hover.
        const focusCommunity = pathToCommunity.get(focusPath);
        const isSelected = selectedPath !== null;

        const lines = new Graphics();
        let edgeCount = 0;
        for (const e of resolvedEdges) {
            let highlight = false;
            if (isSelected && focusCommunity !== undefined) {
                const ca = pathToCommunity.get(e.a);
                const cb = pathToCommunity.get(e.b);
                if (ca === focusCommunity && cb === focusCommunity) highlight = true;
            } else {
                if (e.a === focusPath || e.b === focusPath) highlight = true;
            }
            if (!highlight) continue;
            const sa = pathToScreen.get(e.a);
            const sb = pathToScreen.get(e.b);
            if (!sa || !sb) continue;
            lines.moveTo(sa.x, sa.y);
            lines.lineTo(sb.x, sb.y);
            edgeCount++;
        }
        if (edgeCount > 0) {
            lines.stroke({ color: 0xf5e6c8, alpha: 0.85, width: 1.5 });
            focusOverlay.addChild(lines);
        }

        // Outlined focus star (a ring around the focused star)
        const ring = new Graphics();
        ring.circle(focusScreen.x, focusScreen.y, focusScreen.r + 3);
        ring.stroke({ color: 0xd4af37, alpha: 0.95, width: 2 });
        focusOverlay.addChild(ring);
    }

    /** Find the nearest star within `r=10` of (px, py). Returns the
     *  note_path or null. O(n) iteration; for 30k stars this is ~1ms. */
    function pickStar(px: number, py: number): string | null {
        const HOVER_RADIUS = 10;
        let best: string | null = null;
        let bestDist = HOVER_RADIUS * HOVER_RADIUS;
        for (const [path, s] of pathToScreen) {
            const dx = s.x - px;
            const dy = s.y - py;
            const d2 = dx * dx + dy * dy;
            const effectiveR = s.r + 4; // tolerance: hit even a bit outside the star sprite
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

        // §2G.3b: Wedge labels were Pixi Text — Pixi v8's text path
        // doesn't reliably handle Unicode bidi shaping (Arabic/Hebrew
        // came out backwards on Eisa's universe). Labels now render as
        // HTML elements with `dir="auto"` so the browser handles bidi
        // natively. We just publish the wedge geometry below; the
        // Svelte template renders the labels.
        rimLabelGeometry = regionLayout.wedges.map((wedge) => {
            const t = wedge.arcMidRad;
            const labelR = (rimInner + rimOuter) / 2 + 2;
            const countR = rimOuter + 22;
            const lx = cx + labelR * Math.sin(t);
            const ly = cy - labelR * Math.cos(t);
            const cxLabel = cx + countR * Math.sin(t);
            const cyLabel = cy - countR * Math.cos(t);
            // Tangent rotation in degrees, flipped on the lower half so
            // text reads right-side up.
            let rotDeg = (t * 180) / Math.PI;
            if (t > Math.PI / 2 && t < (3 * Math.PI) / 2) rotDeg += 180;
            // §2G.3c: tangential chord length = arc length at the label
            // radius. Use that as the label's max-width so long library
            // names ellipsize instead of bleeding across adjacent wedges.
            const arcSize = wedge.arcEndRad - wedge.arcStartRad;
            const maxWidthPx = Math.max(40, arcSize * labelR - 8);
            return {
                key: wedge.libraryPath,
                name: wedge.libraryName,
                count: wedge.noteCount,
                lx, ly, cxLabel, cyLabel, rotDeg, maxWidthPx,
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

    // ─── Lifecycle ───────────────────────────────────────────────────
    onMount(async () => {
        app = new Application();
        await app.init({
            // MIG-019 §2G: Suwaidi cream parchment (#faf6e8) — was navy.
            background: 0xfaf6e8,
            resizeTo: canvasContainer,
            antialias: true,
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
        app.stage.addChild(milkyWayContainer);
        app.stage.addChild(territoryContainer);
        app.stage.addChild(edgeContainer);
        app.stage.addChild(starContainer);
        app.stage.addChild(focusOverlay);
        app.stage.addChild(calendarRimContainer);

        // MIG-019 §2E.2 SOLVE: progress markers visible to the user
        // (production builds disable DevTools, so console.log isn't
        // observable). The placeholder text updates at each stage so
        // the OOM page (or status of a hung mount) shows where in the
        // pipeline we got — diagnostic without needing a console.
        showPlaceholder(`Sight v3 — opening
${nodes?.length ?? 0} notes · ${links?.length ?? 0} links`);

        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            // MIG-019 §2G: cache for the polar layout's Region wedge build.
            libraryPathsCached = libraryPaths;
            // Fresh fetch — invalidate any previous regionLayout build so
            // recomputeScreenPositions() rebuilds against the new layoutPoints.
            regionLayout = null;

            showPlaceholder(`Stage 1/4: fetching layout
${nodes?.length ?? 0} notes · ${links?.length ?? 0} links`);
            const layoutT0 = performance.now();
            layoutPoints = await fetchLayout(libraryPaths, 50);
            const layoutMs = Math.round(performance.now() - layoutT0);
            console.log(`[SightV3] layout: ${layoutPoints.length} points in ${layoutMs}ms`);

            // MIG-019 §2A.1 hot-fix (Boss-test OOM 2026-05-07):
            // similarity fetch was BLOCKING the chart render — on 7,600-
            // note universes the IPC payload was blowing JS heap before
            // hidePlaceholder() could fire. Now we render the layout
            // first (stars + territories + connector lines + rim) and
            // fetch similarity in the BACKGROUND. Milky Way appears when
            // the IPC returns; if it OOMs or fails, the rest of v3 still
            // works — graceful degradation.
            if (layoutPoints.length === 0) {
                showPlaceholder('No notes in this universe yet.');
            } else {
                showPlaceholder(`Stage 2/4: building indices (Louvain communities, ${links?.length ?? 0} edges)`);
                const idxT0 = performance.now();
                buildIndices();
                console.log(`[SightV3] buildIndices: ${resolvedEdges.length} unique edges in ${Math.round(performance.now() - idxT0)}ms`);

                showPlaceholder(`Stage 3/4: rendering (${layoutPoints.length} stars, ${Math.min(resolvedEdges.length, MAX_FAINT_EDGES_AT_REST)} edges at-rest)`);
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

        // Resize observer
        resizeObserver = new ResizeObserver(() => {
            if (!isLoading && layoutPoints.length > 0) {
                fullRedraw();
            } else if (placeholderText && placeholderText.visible && app) {
                placeholderText.x = app.screen.width / 2;
                placeholderText.y = app.screen.height / 2;
            }
        });
        resizeObserver.observe(canvasContainer);
    });

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

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
            resizeObserver = null;
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
            if (selectedPath !== null) {
                selectedPath = null;
                drawFocusOverlay();
            } else {
                onClose();
            }
        }
    }
</script>

<svelte:window onkeydown={handleEscape} />

<div class="sight-v3-root">
    <button class="sight-v3-close" onclick={onClose} aria-label={$t('sightV3.close') || 'Close Sight'}>×</button>
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

    {#if tooltipVisible && tooltipText}
        <div
            class="sight-v3-tooltip"
            style="left: {tooltipX}px; top: {tooltipY}px;"
            dir="auto"
        >{tooltipText}</div>
    {/if}

    <!-- §2G.3b: Region-rim labels (HTML overlay). Library names go
         through `dir="auto"` so the browser handles bidi natively —
         Arabic / Hebrew / Persian library names render right-to-left
         within their tangent rotation. -->
    {#each rimLabelGeometry as geo (geo.key)}
        <div
            class="sight-v3-rim-label"
            style="left: {geo.lx}px; top: {geo.ly}px; transform: translate(-50%, -50%) rotate({geo.rotDeg}deg); max-width: {geo.maxWidthPx}px;"
            dir="auto"
            title={geo.name}
        >{geo.name.toUpperCase()}</div>
        <div
            class="sight-v3-rim-count"
            style="left: {geo.cxLabel}px; top: {geo.cyLabel}px; transform: translate(-50%, -50%) rotate({geo.rotDeg}deg); max-width: {geo.maxWidthPx}px;"
        >{geo.count.toLocaleString()} notes</div>
    {/each}

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

    <SightV3SidePanel
        notePath={selectedPath}
        noteTitle={sidePanelTitle}
        communityName={sidePanelCommunity}
        centralityRank={sidePanelRank}
        totalNotes={sidePanelTotalNotes}
        incomingCount={sidePanelIncoming}
        outgoingCount={sidePanelOutgoing}
        onOpenNote={sidePanelOpenNote}
        onClose={sidePanelClose}
    />
</div>

<style>
    /* MIG-019 §2G: Suwaidi cream parchment palette (was navy theme). */
    .sight-v3-root {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: #faf6e8;  /* cream parchment */
        z-index: 1000;
        display: flex;
        align-items: stretch;
    }

    .sight-v3-canvas {
        flex: 1;
        position: relative;
        cursor: default;
    }

    .sight-v3-canvas :global(canvas) {
        display: block;
        width: 100% !important;
        height: 100% !important;
    }

    .sight-v3-close {
        position: absolute;
        top: 12px;
        right: 16px;
        width: 32px;
        height: 32px;
        background: rgba(26, 26, 26, 0.04);
        border: 1px solid rgba(26, 26, 26, 0.25);
        border-radius: 50%;
        color: #1a1a1a;
        font-size: 20px;
        line-height: 28px;
        cursor: pointer;
        z-index: 10;
        padding: 0;
    }

    .sight-v3-close:hover {
        background: rgba(201, 162, 39, 0.20);
        border-color: rgba(201, 162, 39, 0.7);
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
    }

    /* §2G.3b: Region-rim labels (HTML overlay).
       Native bidi via `dir="auto"` so Arabic library names render
       right-to-left within their tangent rotation. */
    .sight-v3-rim-label,
    .sight-v3-rim-count {
        position: absolute;
        pointer-events: none;
        white-space: nowrap;
        text-align: center;
        font-family: serif;
        z-index: 6;
        user-select: none;
        /* §2G.3c: long library names ellipsize within their wedge's
           tangential chord so they can't bleed into adjacent wedges. */
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .sight-v3-rim-label {
        color: #2a4a8c;
        /* §2G.3e: 15 → 12 px to fit longer library names without
           triggering ellipsis as aggressively. */
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 0.16em;
        opacity: 0.92;
    }
    .sight-v3-rim-count {
        color: rgba(26, 26, 26, 0.6);
        font-size: 9px;
        letter-spacing: 0.08em;
        font-style: italic;
    }

    /* MIG-019 §2G (spec §4): Universe Health anchor.
       Top-center, roundel + flanking metrics, blue-ink + gold accents. */
    .sight-v3-health-anchor {
        position: absolute;
        top: 24px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 8;
        display: flex;
        flex-direction: column;
        align-items: center;
        pointer-events: none;
        font-family: serif;
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
        background: rgba(250, 246, 232, 0.96);
        color: #1a1a1a;
        border: 1px solid rgba(201, 162, 39, 0.55);
        padding: 8px 10px;
        font-size: 12px;
        border-radius: 4px;
        pointer-events: none;
        white-space: pre-line;
        z-index: 50;
        max-width: 260px;
    }
</style>

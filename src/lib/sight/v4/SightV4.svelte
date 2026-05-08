<!--
    SightV4.svelte — the v4 star-chart Sight component.

    Architecture pivot from v3: this component is a PLAIN FLEX CHILD,
    NOT a position:fixed overlay. The parent (+layout.svelte) mounts it
    inside `.content-area` using the same `.star-fullscreen` pattern
    that SkyView uses. The close button lives in the PARENT's header
    row and directly sets `sightV4Active = false` — zero callback
    crossing component boundaries.

    This eliminates the root cause of 13 failed close-button iterations
    in v3: a position:fixed overlay with d3-zoom on a viewport-filling
    canvas consumed all pointer events before any button could receive
    them.

    Rendering: Canvas 2D + D3-zoom (proven stack from SkyView + v3's
    §2G.3q migration). Immediate-mode draw() pipeline:
      1. Clear + cream fill
      2. Apply d3-zoom transform (translate + scale)
      3. Draw layers: Milky Way → edges → stars → focus overlay → rim
      4. Sync HTML overlay CSS transform for lockstep zoom/pan.

    Companion to: docs/Constellation-Sight-v3-Concept-Paper-v1.1.md.
-->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import * as d3 from 'd3';
    import { libraryStats, appSettings, type SkyNode, type SkyLink } from '$lib/libraries/store';
    import { get } from 'svelte/store';
    import { fetchLayout, type LayoutPoint } from '$lib/sight/layout-cache';
    import { fetchDensityField, type DensityField } from '$lib/sight/density-cache';
    import { type ProjectionMode } from '$lib/sight/projection';
    import { detectClusters, type ClusterInfo } from '$lib/graph/clusterEngine';
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
    import SightV4SidePanel from './SightV4SidePanel.svelte';
    import { t } from '$lib/i18n';

    // MIG-019 §2G — Polar layout helpers.
    import {
        polarToCartesian,
        radiusFromCentrality,
        magnitudeSize,
        magnitudeAlpha,
        PALETTE as V3_PALETTE,
        DOME_RATIOS,
    } from '../v3/polar';
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
    } from '../v3/modes';
    import {
        buildRegionLayout,
        azimuthInWedge,
        type RegionLayout,
        type RegionWedge,
    } from '../v3/regions';
    import { buildLibraryColorMap } from '../v3/library-colors';
    import { detectDir } from '$lib/utils';

    interface Props {
        nodes: SkyNode[];
        links: SkyLink[];
        searchMatchIds?: Set<string> | null;
        universeName?: string;
        onOpenNote: (path: string, libraryName: string) => void;
    }
    let { nodes, links, searchMatchIds = null, universeName = '', onOpenNote }: Props = $props();

    // ─── DOM + Canvas 2D handles ────────────────────────────────────
    let canvasContainer: HTMLDivElement;
    let canvasEl: HTMLCanvasElement | null = null;
    let ctx: CanvasRenderingContext2D | null = null;
    let canvasDpr = 1;
    let canvasW = 0;
    let canvasH = 0;
    let milkyWayOffscreen: HTMLCanvasElement | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let placeholderMessage: string | null = null;
    let placeholderIsError = false;

    // ─── Data ────────────────────────────────────────────────────────
    let layoutPoints: LayoutPoint[] = $state([]);
    let densityField = $state<DensityField | null>(null);
    let pathToPoint = new Map<string, LayoutPoint>();
    let pathToCreatedAt = new Map<string, number>();
    let monthSegments: MonthSegment[] = [];
    let healthReport = $state<HealthReport>(emptyHealthReport());
    let pathToCommunity = new Map<string, number>();
    let pathToScreen = new Map<string, { x: number; y: number; r: number; baseAlpha: number }>();
    let clusters: ClusterInfo[] = [];
    let communityById = new Map<number, ClusterInfo>();
    let centralityRank = new Map<string, number>();
    let nameToPath = new Map<string, string>();
    let pathToTitle = new Map<string, string>();
    let pathToLibrary = new Map<string, string>();
    let resolvedEdges: Array<{ a: string; b: string }> = [];

    // ─── Polar layout state ──────────────────────────────────────────
    let currentMode = $state<SightMode>(resolveMode(
        ($appSettings as any)?.sight?.lastMode ?? null,
    ));
    let regionLayout = $state<RegionLayout | null>(null);
    let libraryPathsCached: Array<[string, string]> = [];
    let pathToEmbedAngle = new Map<string, number>();
    let pathToCentralityRank = new Map<string, number>();
    let pathToSkyNode = new Map<string, SkyNode>();
    let modeStats: ModeStats = emptyModeStats();
    let libraryColors = $state<Map<string, { hex: number; css: string; index: number }>>(new Map());

    // ─── Chart-level zoom + pan (d3-zoom) ────────────────────────────
    let currentTransform = $state(d3.zoomIdentity);
    const MIN_ZOOM = 0.4;
    const MAX_ZOOM = 5.0;
    let zoomBehavior: d3.ZoomBehavior<HTMLCanvasElement, unknown> | null = null;
    let isDragging = false;
    let panDragMoved = false;

    interface RimLabelGeo {
        key: string;
        name: string;
        index: number;
        colorCss: string;
        count: number;
        lx: number;
        ly: number;
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
    let monthFilterMonth = $state<number>(-1);
    let monthFilterPersistent = $state<boolean>(false);
    let hoveredMonth = $state<number>(-1);
    let matchedPaths = $state<Set<string>>(new Set());

    // ─── Helpers ─────────────────────────────────────────────────────
    function hash01(s: string): number {
        let h = 5381;
        for (let i = 0; i < s.length; i++) {
            h = ((h << 5) + h) ^ s.charCodeAt(i);
        }
        return ((h >>> 0) % 1_000_003) / 1_000_003;
    }

    const STAR_MIN_RADIUS = 1.2;
    const STAR_MAX_RADIUS = 4.0;
    function actualNodeRadius(screenR: number): number {
        const sizeNorm = Math.max(0, Math.min(1, screenR / 8.4));
        return STAR_MIN_RADIUS + sizeNorm * (STAR_MAX_RADIUS - STAR_MIN_RADIUS);
    }

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

    function getViewport() {
        const w = canvasW;
        const h = canvasH;
        if (w === 0 || h === 0) return null;
        const TOP_RESERVE = 320;
        const BOTTOM_RESERVE = 100;
        const SIDE_RESERVE = 100;
        const availableW = Math.max(100, w - 2 * SIDE_RESERVE);
        const availableH = Math.max(100, h - TOP_RESERVE - BOTTOM_RESERVE);
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
        pathToPoint.clear();
        for (const pt of layoutPoints) {
            pathToPoint.set(pt.note_path, pt);
        }

        centralityRank.clear();
        const ranked = [...layoutPoints].sort((a, b) => b.centrality_norm - a.centrality_norm);
        ranked.forEach((p, i) => centralityRank.set(p.note_path, i + 1));

        nameToPath.clear();
        pathToTitle.clear();
        pathToLibrary.clear();
        pathToCreatedAt.clear();
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

        const statsInput = new Map<string, { createdAt?: number; modifiedAt?: number }>();
        for (const n of nodes) {
            statsInput.set(n.path, {
                createdAt: typeof n.createdAt === 'number' ? n.createdAt : undefined,
                modifiedAt: typeof n.createdAt === 'number' ? n.createdAt : undefined,
            });
        }
        modeStats = buildModeStats(statsInput);

        // Resolved edges — deduped via bigint keys (10× less memory than string keys)
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

        // Communities — Louvain
        const communityNodeSubset = nodes.map((n) => ({ id: n.id, name: n.name }));
        const louvain = detectClusters(
            communityNodeSubset,
            links.map((l) => ({ source: l.source, target: l.target })),
        );
        clusters = louvain.clusters;
        communityById.clear();
        for (const c of clusters) communityById.set(c.id, c);
        pathToCommunity.clear();
        for (const [nodeId, communityId] of louvain.assignments) {
            const path = nameToPath.get(nodeId);
            if (path) pathToCommunity.set(path, communityId);
        }

        // Universe health report
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

    // ─── Screen position projection ──────────────────────────────────
    function recomputeScreenPositions() {
        const viewport = getViewport();
        if (!viewport) return;

        if (regionLayout == null && layoutPoints.length > 0 && libraryPathsCached.length > 0) {
            regionLayout = buildRegionLayout(layoutPoints, libraryPathsCached);
            libraryColors = buildLibraryColorMap(regionLayout.wedges);
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
        const INNER_INSET = 0.04 * domeR;
        const OUTER_CAP = 0.96 * domeR;

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
            mctx.modifiedAt = (sky?.createdAt ?? null) as number | null;
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

        if (regionLayout && currentMode === 'regions') {
            applyWedgeRepulsion(cx, cy, domeR);
        }
    }

    // ─── Wedge-bounded repulsion ─────────────────────────────────────
    function applyWedgeRepulsion(cx: number, cy: number, domeR: number) {
        if (!regionLayout) return;
        const MIN_DIST = 9;
        const MAX_ITER = 12;
        const ITER_FACTOR = 0.45;
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
                const grid = new Map<string, StarRef[]>();
                for (const s of stars) {
                    const gx = Math.floor(s.pos.x / cellSize);
                    const gy = Math.floor(s.pos.y / cellSize);
                    const k = gx + ',' + gy;
                    const cell = grid.get(k);
                    if (cell) cell.push(s);
                    else grid.set(k, [s]);
                }
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
                for (const s of stars) {
                    const ddx = s.pos.x - cx;
                    const ddy = s.pos.y - cy;
                    let r = Math.sqrt(ddx * ddx + ddy * ddy);
                    if (r < 1) r = 1;
                    let theta = Math.atan2(ddx, -ddy);
                    theta = ((theta % TAU) + TAU) % TAU;
                    const span = s.wedge.arcEndRad - s.wedge.arcStartRad;
                    const inset = Math.min(0.02 * span, 0.04);
                    const lo = s.wedge.arcStartRad + inset;
                    const hi = s.wedge.arcEndRad - inset;
                    if (theta < lo) theta = lo;
                    else if (theta > hi) theta = hi;
                    if (r < innerR) r = innerR;
                    else if (r > outerR) r = outerR;
                    s.pos.x = cx + r * Math.sin(theta);
                    s.pos.y = cy - r * Math.cos(theta);
                }
            });
            if (!moved) break;
        }
    }

    // ─── Calendar rim ────────────────────────────────────────────────
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

        ctx.beginPath();
        for (const seg of monthSegments) {
            const x1 = rimViewport.cx + seg.rIn * Math.cos(seg.startAngle);
            const y1 = rimViewport.cy + seg.rIn * Math.sin(seg.startAngle);
            const x2 = rimViewport.cx + seg.rOut * Math.cos(seg.startAngle);
            const y2 = rimViewport.cy + seg.rOut * Math.sin(seg.startAngle);
            ctx.moveTo(x1, y1);
            ctx.lineTo(x2, y2);
        }
        for (let r = 0; r <= enabled.length; r++) {
            const radius = rimViewport.innerRadius + r * 22;
            ctx.moveTo(rimViewport.cx + radius, rimViewport.cy);
            ctx.arc(rimViewport.cx, rimViewport.cy, radius, 0, Math.PI * 2);
        }
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.35)';
        ctx.lineWidth = 1;
        ctx.stroke();

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

    // ─── Filter helpers ──────────────────────────────────────────────
    function passesMonthFilter(notePath: string): boolean {
        if (monthFilterMonth < 0) return true;
        const created = pathToCreatedAt.get(notePath);
        if (created === undefined) return false;
        return new Date(created).getMonth() === monthFilterMonth;
    }

    function isSearchActive(): boolean {
        return matchedPaths.size > 0;
    }

    function passesSearch(notePath: string): boolean {
        if (!isSearchActive()) return true;
        return matchedPaths.has(notePath);
    }

    // ─── Milky Way density wash ──────────────────────────────────────
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

    // ─── Stars ───────────────────────────────────────────────────────
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

            ctx.beginPath();
            ctx.arc(screen.x, screen.y, radius, 0, Math.PI * 2);
            ctx.fillStyle = fillCss;
            ctx.globalAlpha = alpha;
            ctx.fill();
            ctx.strokeStyle = '#1a1a1a';
            ctx.lineWidth = 0.6;
            ctx.globalAlpha = alpha * 0.85;
            ctx.stroke();
        }
        ctx.globalAlpha = 1.0;
    }

    // ─── Focus overlay (hover/select edges + rings) ──────────────────
    function drawFocusOverlay() {
        if (!ctx) return;
        const focusPath = selectedPath ?? hoveredPath;
        const searchOn = isSearchActive();

        // Search-active with no specific focus: brighten search-result sub-graph
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

        const focusR = actualNodeRadius(focusScreen.r);
        ctx.beginPath();
        ctx.arc(focusScreen.x, focusScreen.y, focusR + 1, 0, Math.PI * 2);
        ctx.strokeStyle = '#c9a227';
        ctx.globalAlpha = 1.0;
        ctx.lineWidth = 1.5;
        ctx.stroke();
        ctx.globalAlpha = 1.0;
    }

    // ─── Rim rendering ───────────────────────────────────────────────
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

        // Rim numbers
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
            ctx.strokeStyle = '#faf6e8';
            ctx.lineWidth = 3;
            ctx.globalAlpha = 1.0;
            ctx.strokeText(String(lc.index), lx, ly);
            ctx.fillStyle = lc.css;
            ctx.globalAlpha = 1.0;
            ctx.fillText(String(lc.index), lx, ly);
        }

        // Legend metadata
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

    // ─── Hit testing ─────────────────────────────────────────────────
    function pickStar(px: number, py: number): string | null {
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

    // ─── Unified draw pipeline ───────────────────────────────────────
    function draw() {
        if (!ctx || !canvasEl) return;
        const w = canvasW;
        const h = canvasH;
        if (w === 0 || h === 0) return;

        recomputeScreenPositions();

        ctx.save();
        ctx.setTransform(canvasDpr, 0, 0, canvasDpr, 0, 0);
        ctx.clearRect(0, 0, w, h);
        ctx.fillStyle = '#faf6e8';
        ctx.fillRect(0, 0, w, h);
        ctx.restore();

        const t = currentTransform;
        ctx.save();
        ctx.setTransform(
            canvasDpr * t.k, 0,
            0, canvasDpr * t.k,
            canvasDpr * t.x, canvasDpr * t.y,
        );

        drawMilkyWay();
        drawStars();
        drawFocusOverlay();
        drawRimForMode();

        if (placeholderMessage) {
            ctx.font = '18px system-ui, -apple-system, sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = placeholderIsError ? '#a83232' : '#1a1a1a';
            ctx.globalAlpha = 1.0;
            ctx.fillText(placeholderMessage, w / 2, h / 2);
        }

        ctx.restore();
        syncOverlaysTransform();
    }

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
        if (isDragging) {
            tooltipVisible = false;
            return;
        }
        const rect = (canvasEl ?? canvasContainer).getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

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
        if (panDragMoved) {
            panDragMoved = false;
            return;
        }
        const rect = (canvasEl ?? canvasContainer).getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

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
        }
    }

    // ─── Side-panel handlers ─────────────────────────────────────────
    function sidePanelOpenNote() {
        if (selectedPath) {
            const lib = pathToLibrary.get(selectedPath) ?? '';
            onOpenNote(selectedPath, lib);
        }
    }
    function sidePanelClose() {
        selectedPath = null;
        draw();
    }

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
    const sidePanelOutgoing = $derived(0);

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
        // Canvas 2D init — zero global event side effects.
        const canvas = document.createElement('canvas');
        canvasContainer.appendChild(canvas);
        canvasEl = canvas;
        ctx = canvas.getContext('2d');
        if (!ctx) {
            console.error('[SightV4] Failed to get Canvas 2D context');
            return;
        }

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

        // d3-zoom: wheel zoom, drag pan, touch pinch.
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
        sel.on('dblclick.zoom', null);

        // Data fetch + initial draw
        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            libraryPathsCached = libraryPaths;
            regionLayout = null;

            const layoutT0 = performance.now();
            layoutPoints = await fetchLayout(libraryPaths, 50);
            const layoutMs = Math.round(performance.now() - layoutT0);
            console.log(`[SightV4] layout: ${layoutPoints.length} points in ${layoutMs}ms`);

            if (layoutPoints.length === 0) {
                showPlaceholder('No notes in this universe yet.');
            } else {
                const idxT0 = performance.now();
                buildIndices();
                console.log(`[SightV4] buildIndices: ${resolvedEdges.length} unique edges in ${Math.round(performance.now() - idxT0)}ms`);
                const drawT0 = performance.now();
                draw();
                console.log(`[SightV4] draw: ${Math.round(performance.now() - drawT0)}ms`);
                hidePlaceholder();
            }

            // Background density-grid fetch for Milky Way
            (async () => {
                const densT0 = performance.now();
                try {
                    const field = await fetchDensityField(libraryPaths, 50, 0.3);
                    const elapsed = Math.round(performance.now() - densT0);
                    console.log(`[SightV4] density grid fetched: ${field.width}×${field.height} cells, max=${field.max_value.toFixed(3)} in ${elapsed}ms`);
                    if (!canvasEl) return;
                    densityField = field;
                    buildMilkyWayImage();
                    draw();
                } catch (densErr) {
                    console.error('[SightV4] density grid fetch failed (Milky Way empty; chart still functional):', densErr);
                    densityField = null;
                }
            })();
        } catch (e) {
            errorMessage = String(e);
            console.error('[SightV4] layout fetch failed:', e);
            showPlaceholder(`Layout fetch failed: ${errorMessage}`, true);
        }
        isLoading = false;

        // Resize observer
        resizeObserver = new ResizeObserver(() => {
            sizeCanvas();
            if (!isLoading && layoutPoints.length > 0) {
                draw();
            } else if (placeholderMessage) {
                draw();
            }
        });
        resizeObserver.observe(canvasContainer);

        // visualViewport listener
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

    const visualViewportCleanup: { fn: (() => void) | null } = { fn: null };

    // Reactive effects for settings changes
    $effect(() => {
        const _projection = $appSettings.sight?.projection;
        if (!isLoading && layoutPoints.length > 0) {
            draw();
        }
    });

    $effect(() => {
        const _show = $appSettings.sight?.showMilkyWay;
        if (!isLoading && layoutPoints.length > 0) {
            buildMilkyWayImage();
            draw();
        }
    });

    $effect(() => {
        const _calendars = $appSettings.sight?.calendarSystems;
        if (!isLoading && layoutPoints.length > 0) {
            draw();
        }
    });

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
        if (canvasEl && zoomBehavior) {
            d3.select(canvasEl).on('.zoom', null);
        }
        if (canvasEl && canvasEl.parentElement) {
            canvasEl.parentElement.removeChild(canvasEl);
        }
        canvasEl = null;
        ctx = null;
        milkyWayOffscreen = null;
        zoomBehavior = null;
    });
</script>

<!-- v4: NO position:fixed root. This is a plain flex child inside
     .content-area's .star-fullscreen in +layout.svelte. The parent
     owns the close button. -->
<div class="sight-v4-root">
    <div class="sight-v4-body">

    <button
        type="button"
        class="sight-v4-reset-view"
        class:reset-active={currentTransform.k !== 1 || currentTransform.x !== 0 || currentTransform.y !== 0}
        onclick={resetView}
        aria-label={$t('sightV3.resetView') || 'Reset view'}
    >Reset view</button>

    <div
        class="sight-v4-canvas"
        bind:this={canvasContainer}
        onpointermove={handlePointerMove}
        onpointerleave={handlePointerLeave}
        onclick={handleClick}
        ondblclick={handleDoubleClick}
        role="application"
        aria-label="Constellation Sight"
    ></div>

    <!-- HTML overlays wrapper — zoom-synced -->
    <div
        class="sight-v4-overlays-wrapper"
        style="transform-origin: {overlaysTransformOrigin}; transform: {overlaysTransform};"
    >

    {#if libraryColors.size > 0 && rimLabelGeometry.length > 0}
        <div class="sight-v4-legend" class:legend-rtl={detectDir(universeName) === 'rtl'}>
            <div class="sight-v4-legend-header" dir="auto">
                <div class="sight-v4-legend-universe-label">UNIVERSE</div>
                <div class="sight-v4-legend-universe-name">{universeName || 'Universe'}</div>
            </div>
            <div class="sight-v4-legend-divider"></div>
            <div class="sight-v4-legend-libs">
                {#each rimLabelGeometry as geo (geo.key)}
                    <div class="sight-v4-legend-row" dir="auto">
                        <span class="sight-v4-legend-num" style="background: {geo.colorCss};">{geo.index}</span>
                        <span class="sight-v4-legend-name" title={geo.name}>{geo.name}</span>
                        <span class="sight-v4-legend-count">{geo.count.toLocaleString()}</span>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    {#if !isLoading && layoutPoints.length > 0}
        <div class="sight-v4-health-anchor" aria-hidden="false">
            <div class="sight-v4-health-caption">UNIVERSE HEALTH</div>
            <div class="sight-v4-health-row">
                <div class="sight-v4-metric">
                    <div class="sight-v4-metric-label">MODULARITY</div>
                    <div class="sight-v4-metric-value">{healthReport.modularity.display}</div>
                    <div class="sight-v4-metric-pill pill-{healthReport.modularity.status}">
                        {healthReport.modularity.status.toUpperCase()}
                    </div>
                </div>
                <div class="sight-v4-metric">
                    <div class="sight-v4-metric-label">DOMINANCE</div>
                    <div class="sight-v4-metric-value">{healthReport.dominance.display}</div>
                    <div class="sight-v4-metric-pill pill-{healthReport.dominance.status}">
                        {healthReport.dominance.status.toUpperCase()}
                    </div>
                </div>
                <div class="sight-v4-roundel" aria-label="Universe Health Score">
                    <div class="sight-v4-score">{Math.round(healthReport.score)}</div>
                    <div class="sight-v4-score-denom">/ 100</div>
                </div>
                <div class="sight-v4-metric">
                    <div class="sight-v4-metric-label">ENTROPY</div>
                    <div class="sight-v4-metric-value">{healthReport.entropy.display}</div>
                    <div class="sight-v4-metric-pill pill-{healthReport.entropy.status}">
                        {healthReport.entropy.status.toUpperCase()}
                    </div>
                </div>
                <div class="sight-v4-metric">
                    <div class="sight-v4-metric-label">CONNECTIVITY</div>
                    <div class="sight-v4-metric-value">{healthReport.connectivity.display}</div>
                    <div class="sight-v4-metric-pill pill-{healthReport.connectivity.status}">
                        {healthReport.connectivity.status.toUpperCase()}
                    </div>
                </div>
            </div>
        </div>

        {#if universeName}
            <div class="sight-v4-universe-name" dir="auto">{universeName}</div>
        {/if}
    {/if}

    </div><!-- /.sight-v4-overlays-wrapper -->

    {#if tooltipVisible && tooltipText}
        <div
            class="sight-v4-tooltip"
            style="left: {tooltipX}px; top: {tooltipY}px;"
            dir="auto"
        >{tooltipText}</div>
    {/if}

    </div><!-- /.sight-v4-body -->

    <SightV4SidePanel
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
            const sky = pathToSkyNode.get(path);
            if (sky) onOpenNote(path, sky.libraryName);
        }}
        onClose={sidePanelClose}
    />
</div>

<style>
    /* v4: NO position:fixed. This component fills its parent flex slot.
       The parent (+layout.svelte .star-fullscreen) provides the full-screen
       layout. This is the architectural fix that v3 never had. */
    .sight-v4-root {
        width: 100%;
        height: 100%;
        background: #faf6e8;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        position: relative;
    }

    .sight-v4-body {
        flex: 1;
        position: relative;
        overflow: hidden;
        min-height: 0;
    }

    .sight-v4-canvas {
        position: absolute;
        inset: 0;
        cursor: default;
    }

    .sight-v4-canvas :global(canvas) {
        display: block;
    }

    .sight-v4-overlays-wrapper {
        position: absolute;
        inset: 0;
        pointer-events: none;
        will-change: transform;
        z-index: 6;
    }

    .sight-v4-reset-view {
        position: absolute;
        bottom: 28px;
        left: 24px;
        z-index: 10;
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
    .sight-v4-reset-view.reset-active {
        opacity: 1;
        background: rgba(250, 246, 232, 0.95);
        border: 1.5px solid rgba(201, 162, 39, 0.85);
        color: #1a1a1a;
        font-weight: 600;
        box-shadow: 0 1px 3px rgba(26, 26, 26, 0.1);
    }
    .sight-v4-reset-view:hover {
        opacity: 1;
        background: rgba(201, 162, 39, 0.30);
        border-color: #c9a227;
        color: #1a1a1a;
    }
    .sight-v4-reset-view:active {
        transform: scale(0.96);
    }

    .sight-v4-universe-name {
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
        background: rgba(250, 246, 232, 0.93);
        padding: 4px 18px;
        border-radius: 10px;
    }

    .sight-v4-legend {
        position: absolute;
        top: 270px;
        left: 24px;
        width: 248px;
        max-height: calc(100% - 380px);
        z-index: 7;
        font-family: serif;
        background: rgba(250, 246, 232, 0.85);
        border: 1px solid rgba(26, 26, 26, 0.18);
        border-radius: 8px;
        padding: 14px 16px;
        backdrop-filter: blur(2px);
        overflow-y: auto;
        box-shadow: 0 1px 3px rgba(26, 26, 26, 0.05);
        pointer-events: auto;
    }
    .sight-v4-legend.legend-rtl {
        left: auto;
        right: 24px;
    }
    .sight-v4-legend-header {
        margin-bottom: 8px;
    }
    .sight-v4-legend-universe-label {
        font-size: 9px;
        letter-spacing: 2.5px;
        color: rgba(26, 26, 26, 0.55);
        margin-bottom: 4px;
    }
    .sight-v4-legend-universe-name {
        font-size: 14px;
        font-style: italic;
        font-weight: 600;
        color: #2a4a8c;
        line-height: 1.3;
        word-break: break-word;
    }
    .sight-v4-legend-divider {
        height: 1px;
        background: rgba(26, 26, 26, 0.15);
        margin: 10px 0;
    }
    .sight-v4-legend-libs {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .sight-v4-legend-row {
        display: grid;
        grid-template-columns: 24px 1fr auto;
        gap: 8px;
        align-items: center;
        font-size: 12px;
    }
    .sight-v4-legend-num {
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
    .sight-v4-legend-name {
        color: #1a1a1a;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .sight-v4-legend-count {
        color: rgba(26, 26, 26, 0.55);
        font-size: 10px;
        font-style: italic;
        white-space: nowrap;
    }

    .sight-v4-health-anchor {
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
    .sight-v4-health-caption {
        font-size: 11px;
        letter-spacing: 3px;
        color: rgba(26, 26, 26, 0.6);
        margin-bottom: 6px;
    }
    .sight-v4-health-row {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 36px;
    }
    .sight-v4-roundel {
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
    .sight-v4-score {
        font-size: 38px;
        font-weight: 600;
        color: #c9a227;
        line-height: 1;
    }
    .sight-v4-score-denom {
        font-size: 10px;
        color: rgba(26, 26, 26, 0.6);
        margin-top: 4px;
    }
    .sight-v4-metric {
        display: flex;
        flex-direction: column;
        align-items: center;
        min-width: 96px;
    }
    .sight-v4-metric-label {
        font-size: 10px;
        letter-spacing: 2px;
        color: rgba(26, 26, 26, 0.55);
        margin-bottom: 4px;
        white-space: nowrap;
    }
    .sight-v4-metric-value {
        font-size: 22px;
        font-weight: 500;
        color: rgba(26, 26, 26, 0.92);
        line-height: 1.1;
    }
    .sight-v4-metric-pill {
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

    .sight-v4-tooltip {
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
        z-index: 1500;
        max-width: 320px;
        line-height: 1.4;
        box-shadow: 0 2px 6px rgba(26, 26, 26, 0.15);
    }
    .sight-v4-tooltip::first-line {
        font-weight: 700;
        font-size: 14px;
    }
</style>

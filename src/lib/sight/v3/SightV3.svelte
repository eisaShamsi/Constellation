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
    import { Application, Container, Graphics, Text, TextStyle, BlurFilter } from 'pixi.js';
    import { libraryStats, appSettings, type SkyNode, type SkyLink } from '$lib/libraries/store';
    import { get } from 'svelte/store';
    import { fetchLayout, type LayoutPoint } from '$lib/sight/layout-cache';
    import { fetchSimilarity, type SimilarityEdge } from '$lib/sight/similarity-cache';
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

    interface Props {
        nodes: SkyNode[];
        links: SkyLink[];
        /** MIG-019 §2E: search-hub match set. Lowercased note names.
         *  Null when no search is active; non-empty Set when query has matches.
         *  Matched stars flare (brighter + bigger); non-matched dim heavily;
         *  territories containing matches get a halo glow. */
        searchMatchIds?: Set<string> | null;
        onClose: () => void;
        onOpenNote: (path: string, libraryName: string) => void;
    }
    let { nodes, links, searchMatchIds = null, onClose, onOpenNote }: Props = $props();

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
    /** MIG-019 §2B: high-similarity edges from TF-IDF (PJ-035 → Milky Way). */
    let similarityEdges: SimilarityEdge[] = $state([]);
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
    let pathToScreen = new Map<string, { x: number; y: number; r: number }>();
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
    /** Adjacency: note_path → set of neighbor note_paths. Used for hover highlight. */
    let pathAdjacency = new Map<string, Set<string>>();
    /** All graph edges in (path-A, path-B) form, deduped. */
    let resolvedEdges: Array<{ a: string; b: string }> = [];

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

    function starRadius(centrality_norm: number): number {
        return 1.5 + Math.sqrt(Math.max(0, centrality_norm)) * 4.5;
    }

    function getViewport() {
        if (!app) return null;
        const w = app.screen.width;
        const h = app.screen.height;
        if (w === 0 || h === 0) return null;
        const radius = (Math.min(w, h) / 2) * 0.92;
        return { cx: w / 2, cy: h / 2, radius };
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
        for (const n of nodes) {
            nameToPath.set(n.id, n.path);
            pathToTitle.set(n.path, n.name);
            pathToLibrary.set(n.path, n.libraryName);
            if (typeof n.createdAt === 'number') {
                pathToCreatedAt.set(n.path, n.createdAt);
            }
        }

        // Resolved edges in (path-A, path-B) form, deduped
        const edgeSet = new Set<string>();
        resolvedEdges = [];
        pathAdjacency.clear();
        for (const link of links) {
            const aPath = nameToPath.get(link.source);
            const bPath = nameToPath.get(link.target);
            if (!aPath || !bPath || aPath === bPath) continue;
            if (!pathToPoint.has(aPath) || !pathToPoint.has(bPath)) continue;
            const key = aPath < bPath ? `${aPath}|${bPath}` : `${bPath}|${aPath}`;
            if (edgeSet.has(key)) continue;
            edgeSet.add(key);
            resolvedEdges.push({ a: aPath, b: bPath });
            // Adjacency
            let setA = pathAdjacency.get(aPath);
            if (!setA) { setA = new Set(); pathAdjacency.set(aPath, setA); }
            setA.add(bPath);
            let setB = pathAdjacency.get(bPath);
            if (!setB) { setB = new Set(); pathAdjacency.set(bPath, setB); }
            setB.add(aPath);
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

    /** Project all layout points to screen and cache the result. */
    function recomputeScreenPositions() {
        const viewport = getViewport();
        if (!viewport) return;
        const projection = currentProjection();
        pathToScreen.clear();
        for (const pt of layoutPoints) {
            const { x, y } = embedToScreen(pt.embed_x, pt.embed_y, projection, viewport);
            pathToScreen.set(pt.note_path, { x, y, r: starRadius(pt.centrality_norm) });
        }
    }

    /** MIG-019 §2C — Calendar rim (concentric rings, one per enabled
     *  calendar system). Renders arc dividers + month labels around the
     *  perimeter just outside the dome. Eisa's design call (§11 Q3,
     *  2026-05-07): Gregorian default; users add others via Settings. */
    function drawCalendarRim() {
        if (!calendarRimContainer || !app) return;
        calendarRimContainer.removeChildren();

        const enabled = ($appSettings.sight?.calendarSystems ?? ['gregorian']) as CalendarSystem[];
        if (enabled.length === 0) return;

        const w = app.screen.width;
        const h = app.screen.height;
        const domeRadius = (Math.min(w, h) / 2) * 0.92;
        const rimViewport = {
            cx: w / 2,
            cy: h / 2,
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

    /** MIG-019 §2B — Milky Way density wash (PJ-035).
     *  Renders the top-2000 highest-similarity TF-IDF edges as soft
     *  alpha-blended cream-colored lines. The container has a BlurFilter
     *  applied so the lines smear into a continuous band of texture
     *  rather than reading as discrete connections (which would compete
     *  with the explicit-wikilink connector lines).
     *
     *  Capped at 2000 edges per Architect §8 — Pixi v8 BlurFilter on
     *  more than that starts to chug; keeping the strongest signals is
     *  what matters for the at-a-glance density read. */
    const MILKY_WAY_TOP_N = 2000;
    function drawMilkyWay() {
        if (!milkyWayContainer) return;
        milkyWayContainer.removeChildren();
        // Hide if Settings → Sight → "Milky Way density wash" is OFF.
        if ($appSettings.sight?.showMilkyWay === false) {
            milkyWayContainer.visible = false;
            return;
        }
        milkyWayContainer.visible = true;

        const lines = new Graphics();
        let drawnCount = 0;
        // similarityEdges is sorted descending by similarity in §2A,
        // so the slice picks the strongest signals.
        for (const edge of similarityEdges) {
            if (drawnCount >= MILKY_WAY_TOP_N) break;
            const sa = pathToScreen.get(edge.note_path_a);
            const sb = pathToScreen.get(edge.note_path_b);
            if (!sa || !sb) continue;
            lines.moveTo(sa.x, sa.y);
            lines.lineTo(sb.x, sb.y);
            drawnCount++;
        }
        // Suwaidi cream, very low alpha so the BlurFilter merges
        // overlapping lines into a band texture.
        lines.stroke({ color: CONNECTOR_LINE_COLOR, alpha: 0.06, width: 1 });
        milkyWayContainer.addChild(lines);
    }

    /** Draw community territory polygons on the base layer.
     *  - MIG-019 §2E search-active path: territories with matches get
     *    halo (border thickens + brightens); territories without dim.
     *  - MIG-019 §2E always-on labels: render community label at the
     *    centroid when $appSettings.sight.alwaysOnLabels is true. */
    function drawTerritories() {
        if (!territoryContainer) return;
        territoryContainer.removeChildren();

        const labeled: Array<Point2D & { communityId: number }> = [];
        for (const [path, screen] of pathToScreen) {
            const communityId = pathToCommunity.get(path);
            if (communityId === undefined) continue;
            labeled.push({ x: screen.x, y: screen.y, communityId });
        }
        const territories = communityTerritories(labeled);
        const searchOn = isSearchActive();
        const alwaysOnLabels = $appSettings.sight?.alwaysOnLabels === true;

        for (const [communityId, hull] of territories) {
            if (hull.length < 3) continue; // skip degenerate communities
            const color = communityColorInt(communityId);
            const hasMatch = searchOn && communityHasMatch(communityId);

            // Search-active styling:
            //  - territories with matches: thicker border + higher alpha (halo)
            //  - territories without matches: lower fill alpha (dim)
            const fillAlpha = searchOn ? (hasMatch ? 0.16 : 0.04) : 0.10;
            const strokeAlpha = searchOn ? (hasMatch ? 0.85 : 0.10) : 0.30;
            const strokeWidth = searchOn && hasMatch ? 2.5 : 1;

            const poly = new Graphics();
            poly.poly(hull.flatMap((p) => [p.x, p.y]));
            poly.fill({ color, alpha: fillAlpha });
            poly.stroke({ color, alpha: strokeAlpha, width: strokeWidth });
            territoryContainer.addChild(poly);

            // MIG-019 §2E: always-on label at territory centroid.
            if (alwaysOnLabels) {
                const cluster = communityById.get(communityId);
                if (cluster && cluster.suggestedName) {
                    // Centroid as polygon vertex average (approximate).
                    let cx = 0, cy = 0;
                    for (const p of hull) { cx += p.x; cy += p.y; }
                    cx /= hull.length;
                    cy /= hull.length;
                    const label = new Text({
                        text: cluster.suggestedName,
                        style: new TextStyle({
                            fill: 0xd4af37, // Suwaidi gold
                            fontSize: 11,
                            fontFamily: 'system-ui, -apple-system, sans-serif',
                            align: 'center',
                            fontStyle: 'italic',
                        }),
                        anchor: 0.5,
                    });
                    label.x = cx;
                    label.y = cy;
                    label.alpha = 0.7;
                    territoryContainer.addChild(label);
                }
            }
        }
    }

    /** Draw faint connector lines on the base layer. */
    function drawEdges() {
        if (!edgeContainer) return;
        edgeContainer.removeChildren();

        const lines = new Graphics();
        for (const e of resolvedEdges) {
            const sa = pathToScreen.get(e.a);
            const sb = pathToScreen.get(e.b);
            if (!sa || !sb) continue;
            lines.moveTo(sa.x, sa.y);
            lines.lineTo(sb.x, sb.y);
        }
        lines.stroke({ color: CONNECTOR_LINE_COLOR, alpha: 0.12, width: 1 });
        edgeContainer.addChild(lines);
    }

    /** Draw stars on the base layer.
     *  - MIG-019 §2C: dim stars failing the month filter.
     *  - MIG-019 §2E: search-active path — matched stars flare (size +
     *    brightness boost), non-matched dim heavily. */
    function drawStars() {
        if (!starContainer) return;
        starContainer.removeChildren();
        const searchOn = isSearchActive();
        for (const pt of layoutPoints) {
            const screen = pathToScreen.get(pt.note_path);
            if (!screen) continue;
            const passesMonth = passesMonthFilter(pt.note_path);
            const passesSrc = passesSearch(pt.note_path);
            const star = new Graphics();
            const baseAlpha = 0.65 + pt.centrality_norm * 0.35;

            // Decide alpha + size based on filter state.
            let alpha: number;
            let radius = screen.r;
            if (searchOn) {
                if (passesSrc && passesMonth) {
                    // Match: 1.5x size + full alpha (flare effect)
                    radius = screen.r * 1.5;
                    alpha = 1.0;
                } else {
                    // Non-match: heavy dim
                    alpha = 0.10;
                }
            } else if (!passesMonth) {
                // Month-filter dim
                alpha = 0.15;
            } else {
                alpha = baseAlpha;
            }

            star.circle(0, 0, radius);
            star.fill({ color: 0xf5e6c8, alpha });
            star.x = screen.x;
            star.y = screen.y;
            starContainer.addChild(star);
        }
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
        focusOverlay.removeChildren();

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
                lines.stroke({ color: 0xf5e6c8, alpha: 0.85, width: 1.5 });
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

    function fullRedraw() {
        recomputeScreenPositions();
        drawMilkyWay();
        drawTerritories();
        drawEdges();
        drawStars();
        drawFocusOverlay();
        drawCalendarRim();
    }

    function showPlaceholder(message: string, isError: boolean = false) {
        if (!app) return;
        if (!placeholderText) {
            placeholderText = new Text({
                text: message,
                style: new TextStyle({
                    fill: isError ? 0xff6b6b : 0xf5e6c8,
                    fontSize: 18,
                    fontFamily: 'system-ui, -apple-system, sans-serif',
                    align: 'center',
                }),
                anchor: 0.5,
            });
            app.stage.addChild(placeholderText);
        } else {
            placeholderText.text = message;
            (placeholderText.style as TextStyle).fill = isError ? 0xff6b6b : 0xf5e6c8;
            placeholderText.visible = true;
        }
        placeholderText.x = app.screen.width / 2;
        placeholderText.y = app.screen.height / 2;
    }

    function hidePlaceholder() {
        if (placeholderText) placeholderText.visible = false;
    }

    // ─── Pointer handlers ────────────────────────────────────────────
    /** Get rim viewport for hit-testing. Mirrors drawCalendarRim's geometry. */
    function getRimViewport() {
        if (!app) return null;
        const w = app.screen.width;
        const h = app.screen.height;
        const enabled = ($appSettings.sight?.calendarSystems ?? ['gregorian']) as CalendarSystem[];
        const domeRadius = (Math.min(w, h) / 2) * 0.92;
        return {
            cx: w / 2,
            cy: h / 2,
            innerRadius: domeRadius + 4,
            outerRadius: domeRadius + 4 + 22 * enabled.length,
            enabled,
        };
    }

    function handlePointerMove(ev: PointerEvent) {
        const rect = canvasContainer.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;

        // MIG-019 §2C: rim hit-test first. If hovering a calendar month,
        // preview-filter the chart by that month (but only if the filter
        // isn't already pinned by a click).
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
        if (hoveredMonth !== -1) {
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

        // MIG-019 §2C: rim click toggles persistent month filter.
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
            if (monthFilterPersistent) {
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
            background: SKY_BACKGROUND,
            resizeTo: canvasContainer,
            antialias: true,
        });
        canvasContainer.appendChild(app.canvas);

        // MIG-019 §2B: Milky Way density layer. Sits beneath territories
        // (deepest above the sky background) so stars + edges + territory
        // borders all render visibly over the band texture. BlurFilter
        // (8px blur) merges discrete soft lines into a continuous wash —
        // the visual idiom from the Suwaidi-chart reference image.
        milkyWayContainer = new Container();
        const blurFilter = new BlurFilter({ strength: 8, quality: 4 });
        milkyWayContainer.filters = [blurFilter];
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

        showPlaceholder($t('sightV3.placeholder') || 'Sight v3 — projection foundation (MIG-018 §1E)');

        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            const layoutT0 = performance.now();
            layoutPoints = await fetchLayout(libraryPaths, 50);
            console.log(
                `[SightV3] fetched ${layoutPoints.length} layout points across ${libraryPaths.length} libraries in ${Math.round(performance.now() - layoutT0)}ms`,
            );

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
                buildIndices();
                hidePlaceholder();
                fullRedraw();
            }

            // Background similarity fetch (non-blocking)
            (async () => {
                const simT0 = performance.now();
                try {
                    const edges = await fetchSimilarity(libraryPaths, 50, 0.3);
                    console.log(`[SightV3 §2B] fetched ${edges.length} similarity edges in ${Math.round(performance.now() - simT0)}ms`);
                    if (!app) return; // component unmounted while we waited
                    similarityEdges = edges;
                    drawMilkyWay();
                } catch (simErr) {
                    console.error('[SightV3 §2B] similarity fetch failed (Milky Way empty; chart still functional):', simErr);
                    similarityEdges = [];
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

    // MIG-019 §2C: redraw calendar rim when the user enables/disables
    // calendar systems in Settings → Sight → Calendar systems.
    $effect(() => {
        const _calendars = $appSettings.sight?.calendarSystems;
        if (!isLoading && layoutPoints.length > 0) {
            drawCalendarRim();
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

    <SightV3SidePanel
        notePath={selectedPath}
        noteTitle={sidePanelTitle}
        communityName={sidePanelCommunity}
        centralityRank={sidePanelRank}
        totalNotes={sidePanelTotalNotes}
        incomingCount={sidePanelIncoming}
        outgoingCount={sidePanelOutgoing}
        health={healthReport}
        onOpenNote={sidePanelOpenNote}
        onClose={sidePanelClose}
    />
</div>

<style>
    .sight-v3-root {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: #0f1729;
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
        background: rgba(245, 230, 200, 0.1);
        border: 1px solid rgba(245, 230, 200, 0.3);
        border-radius: 50%;
        color: #f5e6c8;
        font-size: 20px;
        line-height: 28px;
        cursor: pointer;
        z-index: 10;
        padding: 0;
    }

    .sight-v3-close:hover {
        background: rgba(212, 175, 55, 0.25);
        border-color: rgba(212, 175, 55, 0.6);
    }

    .sight-v3-tooltip {
        position: fixed;
        background: rgba(15, 23, 41, 0.94);
        color: #f5e6c8;
        border: 1px solid rgba(212, 175, 55, 0.45);
        padding: 8px 10px;
        font-size: 12px;
        border-radius: 4px;
        pointer-events: none;
        white-space: pre-line;
        z-index: 50;
        max-width: 260px;
    }
</style>

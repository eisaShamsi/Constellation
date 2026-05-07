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
    import { Application, Container, Graphics, Text, TextStyle } from 'pixi.js';
    import { libraryStats, appSettings, type SkyNode, type SkyLink } from '$lib/libraries/store';
    import { get } from 'svelte/store';
    import { fetchLayout, type LayoutPoint } from '$lib/sight/layout-cache';
    import { embedToScreen, type ProjectionMode } from '$lib/sight/projection';
    import { detectClusters, type ClusterInfo } from '$lib/graph/clusterEngine';
    import { communityTerritories, type Point2D } from '$lib/sight/community-territory';
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
        onClose: () => void;
        onOpenNote: (path: string, libraryName: string) => void;
    }
    let { nodes, links, onClose, onOpenNote }: Props = $props();

    // ─── DOM + Pixi handles ──────────────────────────────────────────
    let canvasContainer: HTMLDivElement;
    let app: Application | null = null;
    let territoryContainer: Container | null = null;
    let edgeContainer: Container | null = null;
    let starContainer: Container | null = null;
    let focusOverlay: Container | null = null;
    let placeholderText: Text | null = null;
    let resizeObserver: ResizeObserver | null = null;

    // ─── Data ────────────────────────────────────────────────────────
    let layoutPoints: LayoutPoint[] = $state([]);
    /** Map note_path → its layout point (for fast lookup during edge draw). */
    let pathToPoint = new Map<string, LayoutPoint>();
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

        // Name + library lookups from skyNodes
        nameToPath.clear();
        pathToTitle.clear();
        pathToLibrary.clear();
        for (const n of nodes) {
            nameToPath.set(n.id, n.path);
            pathToTitle.set(n.path, n.name);
            pathToLibrary.set(n.path, n.libraryName);
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

    /** Draw community territory polygons on the base layer. */
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

        for (const [communityId, hull] of territories) {
            if (hull.length < 3) continue; // skip degenerate communities
            const color = communityColorInt(communityId);
            const poly = new Graphics();
            poly.poly(hull.flatMap((p) => [p.x, p.y]));
            poly.fill({ color, alpha: 0.10 });
            poly.stroke({ color, alpha: 0.30, width: 1 });
            territoryContainer.addChild(poly);
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

    /** Draw stars on the base layer. */
    function drawStars() {
        if (!starContainer) return;
        starContainer.removeChildren();
        for (const pt of layoutPoints) {
            const screen = pathToScreen.get(pt.note_path);
            if (!screen) continue;
            const star = new Graphics();
            star.circle(0, 0, screen.r);
            star.fill({
                color: 0xf5e6c8,
                alpha: 0.65 + pt.centrality_norm * 0.35,
            });
            star.x = screen.x;
            star.y = screen.y;
            starContainer.addChild(star);
        }
    }

    /** Redraw the focus overlay based on hoveredPath / selectedPath.
     *  - No state → empty overlay.
     *  - hoveredPath (no selection) → brighten edges incident to that star.
     *  - selectedPath → brighten ALL edges within that star's community.
     *  Draws AFTER drawX so it sits on top. */
    function drawFocusOverlay() {
        if (!focusOverlay) return;
        focusOverlay.removeChildren();

        const focusPath = selectedPath ?? hoveredPath;
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
        drawTerritories();
        drawEdges();
        drawStars();
        drawFocusOverlay();
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
    function handlePointerMove(ev: PointerEvent) {
        const rect = canvasContainer.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;
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
        tooltipVisible = false;
    }

    function handleClick(ev: MouseEvent) {
        const rect = canvasContainer.getBoundingClientRect();
        const px = ev.clientX - rect.left;
        const py = ev.clientY - rect.top;
        const nearest = pickStar(px, py);
        if (nearest) {
            selectedPath = nearest;
            drawFocusOverlay();
        } else {
            // Click on background clears selection
            if (selectedPath !== null) {
                selectedPath = null;
                drawFocusOverlay();
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

        territoryContainer = new Container();
        edgeContainer = new Container();
        starContainer = new Container();
        focusOverlay = new Container();
        // Order matters: territories at the back, then edges, then stars, then focus on top.
        app.stage.addChild(territoryContainer);
        app.stage.addChild(edgeContainer);
        app.stage.addChild(starContainer);
        app.stage.addChild(focusOverlay);

        showPlaceholder($t('sightV3.placeholder') || 'Sight v3 — projection foundation (MIG-018 §1E)');

        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            layoutPoints = await fetchLayout(libraryPaths, 50);
            console.log(
                `[SightV3 §1E] fetched ${layoutPoints.length} layout points across ${libraryPaths.length} libraries`,
            );

            if (layoutPoints.length === 0) {
                showPlaceholder('No notes in this universe yet.');
            } else {
                buildIndices();
                hidePlaceholder();
                fullRedraw();
            }
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

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
            resizeObserver = null;
        }
        if (app) {
            app.destroy(true, { children: true, texture: true });
            app = null;
        }
        territoryContainer = null;
        edgeContainer = null;
        starContainer = null;
        focusOverlay = null;
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

<!--
    SightV3.svelte — the v3 star-chart Sight component.

    §1C: empty Pixi canvas + placeholder text.
    §1D (this commit): renders stars at MDS-embedded positions,
        sized by centrality; reads `$appSettings.sight.projection`
        and re-projects on toggle (Lambert ↔ stereographic, free
        operation since the embedding is unchanged).
    §1E adds territories + faint connector lines + hover/click +
        Suwaidi warm-cream + gold palette + side panel.

    Companion to: docs/Constellation-Sight-v3-Concept-Paper-v1.1.md.
    Plan ref:    lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-PLAN.md.
-->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Application, Container, Graphics, Text, TextStyle } from 'pixi.js';
    import { libraryStats, appSettings } from '$lib/libraries/store';
    import { get } from 'svelte/store';
    import { fetchLayout, type LayoutPoint } from '$lib/sight/layout-cache';
    import { embedToScreen, type ProjectionMode } from '$lib/sight/projection';
    import { t } from '$lib/i18n';

    interface Props {
        onClose: () => void;
    }
    let { onClose }: Props = $props();

    let canvasContainer: HTMLDivElement;
    let app: Application | null = null;
    let starContainer: Container | null = null;
    let placeholderText: Text | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let layoutPoints: LayoutPoint[] = $state([]);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);

    /** Re-derive the projection mode from settings on each draw. */
    function currentProjection(): ProjectionMode {
        const s = get(appSettings) as any;
        return (s?.sight?.projection === 'stereographic' ? 'stereographic' : 'lambert');
    }

    /** Map centrality_norm [0,1] → star radius in pixels.
     *  Logarithmic-ish scale: faintest stars ~1.5 px, brightest ~6 px.
     *  Square-root keeps the difference visible without dominating dense
     *  charts where most stars cluster near low centrality. */
    function starRadius(centrality_norm: number): number {
        return 1.5 + Math.sqrt(Math.max(0, centrality_norm)) * 4.5;
    }

    /** Star fill color — Suwaidi warm-cream baseline. §1E will modulate
     *  by lifecycle stage (recently traversed = brighter). */
    const STAR_COLOR = 0xf5e6c8;

    /** Render the star field. Called on initial layout fetch, on
     *  viewport resize, and on projection toggle. */
    function drawStars() {
        if (!app || !starContainer) return;
        starContainer.removeChildren();
        if (placeholderText) {
            placeholderText.visible = false;
        }

        const w = app.screen.width;
        const h = app.screen.height;
        if (w === 0 || h === 0) return;

        const radius = (Math.min(w, h) / 2) * 0.92;  // small margin for rim (MIG-019)
        const viewport = { cx: w / 2, cy: h / 2, radius };
        const projection = currentProjection();

        for (const pt of layoutPoints) {
            const { x, y } = embedToScreen(pt.embed_x, pt.embed_y, projection, viewport);
            const r = starRadius(pt.centrality_norm);
            const star = new Graphics();
            star.circle(0, 0, r);
            star.fill({ color: STAR_COLOR, alpha: 0.65 + pt.centrality_norm * 0.35 });
            star.x = x;
            star.y = y;
            starContainer.addChild(star);
        }
    }

    /** Show placeholder text (empty universe / loading / error). */
    function showPlaceholder(message: string, isError: boolean = false) {
        if (!app) return;
        if (!placeholderText) {
            placeholderText = new Text({
                text: message,
                style: new TextStyle({
                    fill: isError ? 0xff6b6b : STAR_COLOR,
                    fontSize: 18,
                    fontFamily: 'system-ui, -apple-system, sans-serif',
                    align: 'center',
                }),
                anchor: 0.5,
            });
            app.stage.addChild(placeholderText);
        } else {
            placeholderText.text = message;
            placeholderText.style.fill = isError ? 0xff6b6b : STAR_COLOR;
            placeholderText.visible = true;
        }
        placeholderText.x = app.screen.width / 2;
        placeholderText.y = app.screen.height / 2;
    }

    onMount(async () => {
        // Initialize Pixi Application.
        app = new Application();
        await app.init({
            background: 0x0f1729, // Suwaidi-chart deep midnight blue
            resizeTo: canvasContainer,
            antialias: true,
        });
        canvasContainer.appendChild(app.canvas);

        // Star container — separate layer so we can clear + redraw without
        // touching the placeholder or future territory/edge layers.
        starContainer = new Container();
        app.stage.addChild(starContainer);

        showPlaceholder($t('sightV3.placeholder') || 'Sight v3 — projection foundation (MIG-018 §1D)');

        // Fetch layout via IPC.
        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            layoutPoints = await fetchLayout(libraryPaths, 50);
            console.log(
                `[SightV3 §1D] fetched ${layoutPoints.length} layout points across ${libraryPaths.length} libraries`,
            );

            if (layoutPoints.length === 0) {
                showPlaceholder('No notes in this universe yet.');
            } else {
                drawStars();
            }
        } catch (e) {
            errorMessage = String(e);
            console.error('[SightV3 §1D] layout fetch failed:', e);
            showPlaceholder(`Layout fetch failed: ${errorMessage}`, true);
        }

        isLoading = false;

        // Resize observer: Pixi auto-resizes the canvas; we manually
        // redraw stars so positions update. Per Performance Rule 4:
        // disconnect on destroy.
        resizeObserver = new ResizeObserver(() => {
            drawStars();
            if (placeholderText && placeholderText.visible && app) {
                placeholderText.x = app.screen.width / 2;
                placeholderText.y = app.screen.height / 2;
            }
        });
        resizeObserver.observe(canvasContainer);
    });

    // Re-draw on projection toggle. Reading `$appSettings.sight?.projection`
    // is what subscribes the effect — Svelte 5 runes track the read.
    $effect(() => {
        const _projection = $appSettings.sight?.projection;
        if (!isLoading && layoutPoints.length > 0) {
            drawStars();
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
        starContainer = null;
        placeholderText = null;
    });

    function handleEscape(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            e.preventDefault();
            onClose();
        }
    }
</script>

<svelte:window onkeydown={handleEscape} />

<div class="sight-v3-root">
    <button class="sight-v3-close" onclick={onClose} aria-label={$t('sightV3.close') || 'Close Sight'}>×</button>
    <div class="sight-v3-canvas" bind:this={canvasContainer}></div>
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
</style>

<!--
    SightV3.svelte — the v3 star-chart Sight component.

    §1C (this commit): skeleton. Mounts an empty Pixi Application,
    fetches the layout via the IPC, logs the count, renders a
    placeholder text. No stars yet, no territories, no interactions.

    §1D adds star rendering + projection toggle.
    §1E adds territories + faint connector lines + hover/click +
    Suwaidi warm-cream + gold palette + side panel.

    Companion to: docs/Constellation-Sight-v3-Concept-Paper-v1.1.md.
    Plan ref:    lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-PLAN.md.
-->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Application, Text, TextStyle } from 'pixi.js';
    import { libraryStats } from '$lib/libraries/store';
    import { get } from 'svelte/store';
    import { fetchLayout, type LayoutPoint } from '$lib/sight/layout-cache';
    import { t } from '$lib/i18n';

    interface Props {
        onClose: () => void;
    }
    let { onClose }: Props = $props();

    let canvasContainer: HTMLDivElement;
    let app: Application | null = null;
    let layoutPoints: LayoutPoint[] = $state([]);
    let isLoading = $state(true);
    let errorMessage = $state<string | null>(null);

    onMount(async () => {
        // Initialize Pixi Application.
        app = new Application();
        await app.init({
            background: '#0f1729',  // Suwaidi-chart deep midnight blue (placeholder for §1E full palette)
            resizeTo: canvasContainer,
            antialias: true,
        });
        canvasContainer.appendChild(app.canvas);

        // Render placeholder text while §1C development continues.
        const placeholder = new Text({
            text: $t('sightV3.placeholder') || 'Sight v3 — projection foundation (MIG-018 §1C)',
            style: new TextStyle({
                fill: '#f5e6c8',  // Suwaidi warm-cream
                fontSize: 18,
                fontFamily: 'system-ui, -apple-system, sans-serif',
                align: 'center',
            }),
            anchor: 0.5,
        });
        placeholder.x = app.screen.width / 2;
        placeholder.y = app.screen.height / 2;
        app.stage.addChild(placeholder);

        // Fetch layout via IPC. Logs count for §1C verification; §1D
        // will use the points to render stars.
        try {
            const stats = get(libraryStats);
            const libraryPaths: Array<[string, string]> = stats.map((s) => [s.path, s.name]);
            layoutPoints = await fetchLayout(libraryPaths, 50);
            console.log(
                `[SightV3 §1C] fetched ${layoutPoints.length} layout points across ${libraryPaths.length} libraries`,
            );

            // Update placeholder to confirm fetch succeeded.
            placeholder.text =
                ($t('sightV3.placeholder') || 'Sight v3 — projection foundation (MIG-018 §1C)') +
                `\n${layoutPoints.length} notes loaded`;
        } catch (e) {
            errorMessage = String(e);
            console.error('[SightV3 §1C] layout fetch failed:', e);
            placeholder.text = `Layout fetch failed: ${errorMessage}`;
            placeholder.style.fill = '#ff6b6b';
        }

        isLoading = false;
    });

    onDestroy(() => {
        // Per CLAUDE.md Performance Rule 4 (no memory leaks): destroy
        // the Pixi Application on unmount, releasing all GPU textures
        // and event listeners.
        if (app) {
            app.destroy(true, { children: true, texture: true });
            app = null;
        }
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
    <button class="sight-v3-close" onclick={onClose} aria-label="Close Sight">×</button>
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

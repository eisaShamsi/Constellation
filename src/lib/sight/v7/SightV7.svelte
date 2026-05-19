<!--
  MIG-036 P3 (2026-05-19) — Sight v7 universe-view mount.

  P1 shipped this as a placeholder; P3 (this commit) wires it to
  the real dispatcher (anchor-v7.ts) with masādir-v7 as the first
  working sample tradition. Future phases extend:

    P4–P5 — port the remaining 23 traditions to TraditionModuleV7
    P6    — Time Dome (separate module, opts into the calendar rim)
    P7    — drill-in interaction (click cell → stack view)
    P8    — mini-dome adaptation under v7 hybrid model
    P9    — dropdown reorganization (Time group + 24 traditions)

  Still gated by SIGHT_V7_ENABLED (false in dev). Production users
  see v6 until the ship-gate (P11) flips the flag.

  Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t, locale } from '$lib/i18n';
	import { get } from 'svelte/store';
	import type { LayoutCacheRow } from '$lib/sight/v6/types';
	import { readChromePalette } from '$lib/sight/v6/dome';
	import { masadirV7 } from './traditions/masadir';
	import {
		renderAnchorDomeV7,
		cellAtPoint,
		type CellHitTestV7,
	} from './anchor-v7';

	let {
		onOpenNote: _onOpenNote = () => {},
	}: {
		onOpenNote?: (path: string, libraryName: string) => void;
	} = $props();

	// P3 hardcodes masādir as the active tradition — the full dropdown
	// (24 traditions + Time Dome group) lands in P9. This is enough for
	// the P3 verification clause: "masādir under v7 renders 4 wedges
	// with density blobs sized by population count."
	const activeTradition = masadirV7;

	let canvasEl: HTMLCanvasElement | null = $state(null);
	let hostEl: HTMLDivElement | null = $state(null);

	let rows: LayoutCacheRow[] = $state([]);
	let loading = $state(true);
	let errorMessage: string | null = $state(null);

	let hoveredCellId: string | null = $state(null);
	let lastHitTests: CellHitTestV7[] = [];

	let resizeObserver: ResizeObserver | null = null;
	let unsubscribeLocale: (() => void) | null = null;

	// ────────────────────────────────────────────────────────────────
	// Canvas paint loop
	// ────────────────────────────────────────────────────────────────

	function paint() {
		if (!canvasEl || !hostEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		// DPR-aware sizing so the dome stays crisp on hi-DPI displays.
		const dpr = window.devicePixelRatio || 1;
		const cssWidth = hostEl.clientWidth;
		const cssHeight = hostEl.clientHeight;
		if (cssWidth <= 0 || cssHeight <= 0) return;

		const backingWidth = Math.floor(cssWidth * dpr);
		const backingHeight = Math.floor(cssHeight * dpr);
		if (canvasEl.width !== backingWidth || canvasEl.height !== backingHeight) {
			canvasEl.width = backingWidth;
			canvasEl.height = backingHeight;
			canvasEl.style.width = `${cssWidth}px`;
			canvasEl.style.height = `${cssHeight}px`;
		}
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

		const palette = readChromePalette(hostEl);
		const currentLocale = get(locale) || 'en';
		const labelize = (key: string) => get(t)(key) as string;

		lastHitTests = renderAnchorDomeV7(ctx, rows, cssWidth, cssHeight, {
			tradition: activeTradition,
			locale: currentLocale,
			chromePalette: palette,
			labelize,
			hoveredCellId,
			selectedCellId: null, // P7 wires drill-in selection
		});
	}

	// ────────────────────────────────────────────────────────────────
	// Mouse interactions (hover + click)
	// ────────────────────────────────────────────────────────────────

	function handleMouseMove(e: MouseEvent) {
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const px = e.clientX - rect.left;
		const py = e.clientY - rect.top;
		const hit = cellAtPoint(lastHitTests, px, py);
		const newHover = hit?.cellId ?? null;
		if (newHover !== hoveredCellId) {
			hoveredCellId = newHover;
			paint();
		}
	}

	function handleMouseLeave() {
		if (hoveredCellId !== null) {
			hoveredCellId = null;
			paint();
		}
	}

	function handleClick(e: MouseEvent) {
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const px = e.clientX - rect.left;
		const py = e.clientY - rect.top;
		const hit = cellAtPoint(lastHitTests, px, py);
		if (!hit) return;
		// P7 will wire this into the cell-view drill-in. For P3, log so
		// the Boss test can verify the cell IDs are resolving correctly
		// from the click → cellAtPoint pipeline.
		console.log(
			`[Sight v7] cell click: ${hit.cellId} — ${hit.cellLabel} (${hit.population} notes)`,
		);
	}

	// ────────────────────────────────────────────────────────────────
	// Mount / data fetch
	// ────────────────────────────────────────────────────────────────

	onMount(() => {
		(async () => {
			try {
				rows = await invoke<LayoutCacheRow[]>('sight_v6_get_layout');
				loading = false;
				// Wait a frame so the canvas has its layout-driven size.
				requestAnimationFrame(paint);
			} catch (err) {
				errorMessage = String(err);
				loading = false;
			}
		})();

		// Resize observer: repaint when the dome container changes size.
		if (hostEl) {
			resizeObserver = new ResizeObserver(() => paint());
			resizeObserver.observe(hostEl);
		}

		// Locale change → repaint so stratum/cell labels update.
		unsubscribeLocale = locale.subscribe(() => {
			if (!loading) requestAnimationFrame(paint);
		});
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		resizeObserver = null;
		unsubscribeLocale?.();
		unsubscribeLocale = null;
	});
</script>

<div class="sight-v7-root">
	<div class="sight-v7-header">
		<span class="sight-v7-title">{$t('sight.v6.title')}</span>
		<span class="sight-v7-subtitle">v7.0 — Form-Aligns-To-Purpose</span>
		<span class="sight-v7-tradition">
			{$t(activeTradition.name) === activeTradition.name
				? activeTradition.name
				: $t(activeTradition.name)}
		</span>
	</div>
	<div class="sight-v7-canvas-host" bind:this={hostEl}>
		{#if loading}
			<div class="sight-v7-status">Loading…</div>
		{:else if errorMessage}
			<div class="sight-v7-status sight-v7-status-error">
				Sight v7 failed to load: {errorMessage}
			</div>
		{:else}
			<canvas
				bind:this={canvasEl}
				onmousemove={handleMouseMove}
				onmouseleave={handleMouseLeave}
				onclick={handleClick}
			></canvas>
			{#if rows.length === 0}
				<div class="sight-v7-status">
					No notes in the active library — open a library with content to
					see the dome populate.
				</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	.sight-v7-root {
		flex: 1 1 auto;
		display: flex;
		flex-direction: column;
		min-height: 0;
		background: var(--background-primary, #0c1322);
		color: var(--text-normal, #cdd5e0);
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);
	}
	.sight-v7-header {
		flex: 0 0 auto;
		display: flex;
		align-items: baseline;
		gap: 14px;
		padding: 12px 18px;
		border-bottom: 1px solid var(--background-modifier-border, #1a1f2e);
	}
	.sight-v7-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-normal, #cdd5e0);
	}
	.sight-v7-subtitle {
		font-size: 11px;
		font-style: italic;
		color: var(--text-faint, #5a6275);
	}
	.sight-v7-tradition {
		font-size: 12px;
		color: var(--text-muted, #a0aabe);
		margin-inline-start: auto;
	}
	.sight-v7-canvas-host {
		flex: 1 1 auto;
		position: relative;
		min-height: 0;
		overflow: hidden;
	}
	.sight-v7-canvas-host canvas {
		display: block;
		width: 100%;
		height: 100%;
	}
	.sight-v7-status {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 13px;
		color: var(--text-muted, #a0aabe);
		pointer-events: none;
	}
	.sight-v7-status-error {
		color: #f08a8a;
	}
</style>

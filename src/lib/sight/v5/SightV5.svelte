<script lang="ts">
	/**
	 * Sight v5 — Layer 1 visual foundation, MIG-024.
	 *
	 * §1 shipped the skeleton + mount path; §3 lands the dome chrome:
	 * 8 strata bands + Milky Way wash + calendar rim + month labels.
	 * Mode toggle UI lands in §4; stars + interactivity in §5.
	 *
	 * Render strategy (D-V1 = Canvas 2D + D3-zoom): two-layer Canvas —
	 * base layer for the dome chrome (drawn once per cache-warm
	 * cycle), focus overlay for hover/select state (redrawn on
	 * interaction, lands in §5). Month labels are HTML overlay, NOT
	 * canvas-drawn — `dir="auto"` handles RTL automatically per v3
	 * invariant 12.
	 *
	 * Concept Paper v3.1 §5 + Mock B1 visual contract.
	 */
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import type { SightV5Mode, SightV5Scope } from './types';
	import { renderBaseLayer } from './render';
	import { calendarRimMonths, type MonthLabel } from './dome';

	// ─── persisted mode + scope ────────────────────────────────────
	const VALID_MODES: ReadonlySet<SightV5Mode> = new Set(['R', 'L', 'T', 'C', 'S', 'A', 'P']);
	const VALID_SCOPES: ReadonlySet<SightV5Scope> = new Set(['universe', 'library', 'folder']);

	let activeMode: SightV5Mode = $derived.by(() => {
		const saved = $appSettings.sight?.lastMode;
		return saved && VALID_MODES.has(saved) ? saved : 'R';
	});
	let activeScope: SightV5Scope = $derived.by(() => {
		const saved = $appSettings.sight?.lastScope;
		return saved && VALID_SCOPES.has(saved) ? saved : 'universe';
	});

	// ─── canvas + container refs ───────────────────────────────────
	let canvasEl: HTMLCanvasElement | undefined = $state(undefined);
	let containerEl: HTMLDivElement | undefined = $state(undefined);
	let canvasWidth = $state(800);
	let canvasHeight = $state(600);

	// Dome radius scales to the smaller container dimension, leaving
	// ~50 px of padding for the calendar rim's month label overlay.
	let domeRadius = $derived(Math.max(120, Math.min(canvasWidth, canvasHeight) / 2 - 50));

	// Month labels — computed from active locale + domeRadius.
	let monthLabels: MonthLabel[] = $derived(calendarRimMonths(domeRadius, $locale));

	// Current month index (0..11) — used by render.ts to gold-tint
	// the current month's wedge.
	const currentMonthIndex = new Date().getMonth();

	// ─── render orchestration ──────────────────────────────────────
	function draw() {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		// Set Canvas resolution to device-pixel-ratio-aware logical size.
		const dpr = window.devicePixelRatio || 1;
		canvasEl.width = canvasWidth * dpr;
		canvasEl.height = canvasHeight * dpr;
		canvasEl.style.width = `${canvasWidth}px`;
		canvasEl.style.height = `${canvasHeight}px`;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		renderBaseLayer(ctx, canvasWidth, canvasHeight, domeRadius, currentMonthIndex);
	}

	// Redraw whenever the dome size or locale changes.
	$effect(() => {
		// Touch reactive deps so this effect re-runs when they change.
		void canvasWidth; void canvasHeight; void domeRadius; void $locale;
		draw();
	});

	onMount(() => {
		if (!containerEl) return;
		// Initial measure.
		const rect = containerEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;

		// Observe resize.
		const ro = new ResizeObserver(entries => {
			for (const entry of entries) {
				canvasWidth = entry.contentRect.width;
				canvasHeight = entry.contentRect.height;
			}
		});
		ro.observe(containerEl);

		return () => ro.disconnect();
	});
</script>

<div class="sight-v5-root" bind:this={containerEl}>
	<!-- Canvas base layer — dome chrome (§3) + stars (§5, future). -->
	<canvas class="sight-v5-canvas" bind:this={canvasEl}></canvas>

	<!-- HTML overlay layer for month labels (NOT canvas-drawn text per
	     v3 invariant 12; dir="auto" handles RTL). Positioned absolutely
	     relative to the canvas; transform centers each label at its
	     calendar-rim wedge position. -->
	<div class="sight-v5-rim-labels" aria-hidden="false">
		{#each monthLabels as label (label.monthIndex)}
			<span
				class="sight-v5-rim-label"
				dir="auto"
				style="left: {canvasWidth / 2 + label.x}px; top: {canvasHeight / 2 + label.y}px;"
			>{label.label}</span>
		{/each}
	</div>

	<!-- Status strip — §1 placeholder, will be replaced by the mode
	     toggle bar in §4 and the side panel in §5. -->
	<div class="sight-v5-status-strip">
		<span class="sight-v5-status-label">{$t('sight.v5.mode') || 'Mode'}:</span>
		<strong>{activeMode}</strong>
		<span class="sight-v5-status-sep">·</span>
		<span class="sight-v5-status-label">{$t('sight.v5.scope') || 'Scope'}:</span>
		<strong>{activeScope}</strong>
	</div>
</div>

<style>
	.sight-v5-root {
		position: relative;
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		background: #faf6e8;
		color: #1a1a1a;
		font-family: Georgia, 'Times New Roman', serif;
		overflow: hidden;
	}
	.sight-v5-canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		display: block;
	}
	.sight-v5-rim-labels {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.sight-v5-rim-label {
		position: absolute;
		transform: translate(-50%, -50%);
		font-size: 10px;
		color: #2a4a8c;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		white-space: nowrap;
	}
	.sight-v5-status-strip {
		position: absolute;
		top: 16px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.85rem;
		color: #3a3a3a;
		background: rgba(250, 246, 232, 0.85);
		padding: 0.25rem 0.75rem;
		border-radius: 4px;
		pointer-events: none;
	}
	.sight-v5-status-label {
		color: #2a4a8c;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		font-size: 0.7rem;
	}
	.sight-v5-status-sep {
		color: #b8a98a;
	}
</style>

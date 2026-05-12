<script lang="ts">
	/**
	 * Sight v5 — Layer 1 visual foundation, MIG-024.
	 *
	 * §1 — module skeleton + appSettings extension.
	 * §3 — dome chrome (8 strata bands + Milky Way wash + calendar rim).
	 * §4 — mode toggle bar (7 modes) + scope toggle bar (3 scopes per D-V3)
	 *      + per-mode wedge boundary spokes overlay.
	 * §5 — stars + connectors + hover/select + side panel (future).
	 *
	 * Render strategy: Canvas 2D + D3-zoom (D-V1 lock). Two-layer:
	 * base layer (dome chrome + stars; redrawn on size/mode/scope
	 * change) + focus overlay (hover/select; §5).
	 *
	 * Mount pattern (inherited from v4): flex child inside
	 * `.content-area`, close button in `+layout.svelte` header row.
	 *
	 * Concept Paper v3.1 §5–§7 + Mock B1 visual contract.
	 */
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import type { SightV5Mode, SightV5Scope } from './types';
	import { renderBaseLayer } from './render';
	import { calendarRimMonths, type MonthLabel } from './dome';
	import { buildModeContext } from './modes';

	// ─── canonical mode + scope key sets ───────────────────────────
	const MODES: ReadonlyArray<SightV5Mode> = ['R', 'L', 'T', 'C', 'S', 'A', 'P'];
	const VALID_MODES: ReadonlySet<SightV5Mode> = new Set(MODES);
	const VALID_SCOPES: ReadonlySet<SightV5Scope> = new Set(['universe', 'library', 'folder']);
	const SCOPE_LETTERS: Record<SightV5Scope, string> = {
		universe: 'U',
		library: 'L',
		folder: 'F',
	};

	// ─── persisted mode + scope ────────────────────────────────────
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

	let domeRadius = $derived(Math.max(120, Math.min(canvasWidth, canvasHeight) / 2 - 50));
	let monthLabels: MonthLabel[] = $derived(calendarRimMonths(domeRadius, $locale));
	const currentMonthIndex = new Date().getMonth();

	// §4: wedge-boundary angles for the active mode. Computed from an
	// empty rows array — wedge GEOMETRY (uniform spacing for fixed-
	// order modes; library-count-proportional for R) doesn't need
	// counts; §5 will supply real rows + use the same context for
	// per-star azimuth.
	let modeWedgeAngles: number[] = $derived.by(() => {
		const ctx = buildModeContext(activeMode, [], $locale);
		// Use each bucket's start angle; the last bucket's end angle
		// equals the first bucket's start angle (full circle).
		return ctx.wedges.map(w => w.azimuthStart);
	});

	// ─── render orchestration ──────────────────────────────────────
	function draw() {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		const dpr = window.devicePixelRatio || 1;
		canvasEl.width = canvasWidth * dpr;
		canvasEl.height = canvasHeight * dpr;
		canvasEl.style.width = `${canvasWidth}px`;
		canvasEl.style.height = `${canvasHeight}px`;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		renderBaseLayer(ctx, canvasWidth, canvasHeight, domeRadius, currentMonthIndex, modeWedgeAngles);
	}

	$effect(() => {
		void canvasWidth; void canvasHeight; void domeRadius; void $locale; void modeWedgeAngles;
		draw();
	});

	onMount(() => {
		if (!containerEl) return;
		const rect = containerEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;
		const ro = new ResizeObserver(entries => {
			for (const entry of entries) {
				canvasWidth = entry.contentRect.width;
				canvasHeight = entry.contentRect.height;
			}
		});
		ro.observe(containerEl);
		return () => ro.disconnect();
	});

	// ─── mode + scope toggle handlers ──────────────────────────────
	function setMode(m: SightV5Mode) {
		if (m === activeMode) return;
		$appSettings.sight = { ...($appSettings.sight ?? {}), lastMode: m };
		saveSettings();
	}
	function setScope(s: SightV5Scope) {
		if (s === activeScope) return;
		$appSettings.sight = { ...($appSettings.sight ?? {}), lastScope: s };
		saveSettings();
	}

	// ─── mode + scope state helpers ────────────────────────────────
	// Per Concept Paper §5.5 + D-V6, mode P is dimmed when sparse
	// data is detected; for §4 we treat all modes as "ready" since
	// the data check happens in §5 (when actual rows are read from
	// the layout cache). For now, every mode reflects two states:
	// active (gold) or ready (cream, inactive).
	function modeState(m: SightV5Mode): 'active' | 'ready' {
		return m === activeMode ? 'active' : 'ready';
	}
	function scopeState(s: SightV5Scope): 'active' | 'ready' {
		return s === activeScope ? 'active' : 'ready';
	}
</script>

<div class="sight-v5-root" bind:this={containerEl}>
	<!-- Canvas base layer — dome chrome (§3) + per-mode wedge spokes
	     (§4) + stars (§5, future). -->
	<canvas class="sight-v5-canvas" bind:this={canvasEl}></canvas>

	<!-- HTML overlay for month labels (NOT canvas-drawn text per v3
	     invariant 12; dir="auto" handles RTL). -->
	<div class="sight-v5-rim-labels" aria-hidden="false">
		{#each monthLabels as label (label.monthIndex)}
			<span
				class="sight-v5-rim-label"
				dir="auto"
				style="left: {canvasWidth / 2 + label.x}px; top: {canvasHeight / 2 + label.y}px;"
			>{label.label}</span>
		{/each}
	</div>

	<!-- §4 mode toggle bar — 7 buttons, top-center. -->
	<div class="sight-v5-mode-bar" role="tablist" aria-label="Sight modes">
		{#each MODES as m (m)}
			<button
				type="button"
				role="tab"
				aria-selected={m === activeMode}
				class="sight-v5-mode-btn"
				class:active={modeState(m) === 'active'}
				class:ready={modeState(m) === 'ready'}
				title={$t(`sight.v5.mode.${m}.title`) || m}
				onclick={() => setMode(m)}
			>{m}</button>
		{/each}
	</div>

	<!-- §4 scope toggle bar — 3 buttons (D-V3), below mode bar. -->
	<div class="sight-v5-scope-bar" role="tablist" aria-label="Sight scope">
		{#each (['universe', 'library', 'folder'] as SightV5Scope[]) as s (s)}
			<button
				type="button"
				role="tab"
				aria-selected={s === activeScope}
				class="sight-v5-scope-btn"
				class:active={scopeState(s) === 'active'}
				title={$t(`sight.v5.scope.${s}.title`) || s}
				onclick={() => setScope(s)}
			>{SCOPE_LETTERS[s]}</button>
		{/each}
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
	/* Mode toggle bar — Mock B1 geometry: 7 buttons, ~50w × 44h, 10 gap, centered above dome. */
	.sight-v5-mode-bar {
		position: absolute;
		top: 24px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		gap: 10px;
		z-index: 2;
	}
	.sight-v5-mode-btn {
		width: 50px;
		height: 44px;
		border-radius: 6px;
		font-family: Georgia, 'Times New Roman', serif;
		font-size: 20px;
		font-weight: 700;
		cursor: pointer;
		transition: background 200ms ease, color 200ms ease, border-color 200ms ease;
	}
	.sight-v5-mode-btn.active {
		background: #c9a227;
		color: #faf6e8;
		border: 1px solid #c9a227;
	}
	.sight-v5-mode-btn.ready {
		background: #fbf8ec;
		color: #1a1a1a;
		border: 1px solid #1a1a1a;
		opacity: 0.95;
	}
	.sight-v5-mode-btn:hover {
		filter: brightness(0.97);
	}
	/* Scope toggle bar — 3 buttons, below mode bar. */
	.sight-v5-scope-bar {
		position: absolute;
		top: 80px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		gap: 6px;
		z-index: 2;
	}
	.sight-v5-scope-btn {
		width: 34px;
		height: 28px;
		border-radius: 4px;
		font-family: Georgia, 'Times New Roman', serif;
		font-size: 13px;
		font-weight: 700;
		cursor: pointer;
		background: #fbf8ec;
		color: #2a4a8c;
		border: 1px solid #2a4a8c;
		transition: background 200ms ease, color 200ms ease;
	}
	.sight-v5-scope-btn.active {
		background: #2a4a8c;
		color: #faf6e8;
	}
</style>

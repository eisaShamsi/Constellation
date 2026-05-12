<script lang="ts">
	/**
	 * Sight v5 — Layer 1 visual foundation, MIG-024.
	 *
	 * §1 — module skeleton + appSettings extension.
	 * §3 — dome chrome (8 strata bands + Milky Way wash + calendar rim).
	 * §4 — mode toggle bar (7 modes) + scope toggle bar (3 scopes per D-V3).
	 * §5 — layout-cache load + stars + connectors + hover/select + side panel.
	 *
	 * Mount pattern (inherited from v4): flex child inside
	 * `.content-area`, close button in `+layout.svelte` header row.
	 *
	 * Concept Paper v3.1 §5–§7 + Mock B1 visual contract.
	 */
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t, locale } from '$lib/i18n';
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import type { SightV5Mode, SightV5Scope, LayoutCacheRow, LinkEdge } from './types';
	import {
		renderBaseLayer,
		renderFocusOverlay,
		computeStarPositions,
		starHitDistanceSq,
		type StarPosition,
	} from './render';
	import { calendarRimMonths, type MonthLabel, radiusForStratum } from './dome';
	import { buildModeContext, azimuthForMode } from './modes';
	import { filterNotesByScope } from './scope';
	import SightV5SidePanel from './SightV5SidePanel.svelte';

	interface Props {
		onOpenNote?: (path: string, libraryName: string) => void;
	}
	let { onOpenNote }: Props = $props();

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

	// ─── layout cache + scope-filtered rows ────────────────────────
	let allRows: LayoutCacheRow[] = $state([]);
	let links: LinkEdge[] = $state([]);
	let isLoading = $state(true);

	// Scope-filtered rows (D-V3): universe = all; library / folder pick
	// the active sidebar context. For §5 we use whatever appSettings
	// stores; the wiring to a real "active library / folder" comes when
	// the layout passes that context as a prop (future).
	let visibleRows = $derived.by(() => filterNotesByScope(allRows, activeScope, null));

	// ─── mode context + star positions ─────────────────────────────
	let modeContext = $derived.by(() => buildModeContext(activeMode, visibleRows, $locale));
	let modeWedgeAngles: number[] = $derived(modeContext.wedges.map(w => w.azimuthStart));

	let stars: StarPosition[] = $derived.by(() => {
		const bandFor = (s: number) => radiusForStratum(s, domeRadius);
		const azFor = (r: LayoutCacheRow) => azimuthForMode(r, modeContext);
		return computeStarPositions(visibleRows, bandFor, azFor);
	});
	let starsByPath: Map<string, StarPosition> = $derived.by(() => {
		const m = new Map<string, StarPosition>();
		for (const s of stars) m.set(s.row.notePath, s);
		return m;
	});

	// ─── hover + select state ──────────────────────────────────────
	let hoveredStar: StarPosition | null = $state(null);
	let selectedStar: StarPosition | null = $state(null);
	let tooltipX = $state(0);
	let tooltipY = $state(0);

	// Incident edges for the focused star (selected wins; hover falls back).
	let focusedStar: StarPosition | null = $derived(selectedStar ?? hoveredStar);
	let incidentEdges: LinkEdge[] = $derived.by(() => {
		const f = focusedStar as StarPosition | null;
		if (f == null) return [];
		const path: string = f.row.notePath;
		return links.filter((e: LinkEdge) => e.sourcePath === path || e.targetPath === path);
	});

	// Mode P empty-state — when most of the visible set is unsourced.
	let modePEmptyState = $derived.by(() => {
		if (activeMode !== 'P') return false;
		if (visibleRows.length === 0) return false;
		const unsourced = visibleRows.filter(r => r.sourcesPrimary == null).length;
		return unsourced / visibleRows.length > 0.7;
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
		renderBaseLayer(
			ctx,
			canvasWidth,
			canvasHeight,
			domeRadius,
			currentMonthIndex,
			modeWedgeAngles,
			stars,
			links,
		);
		// Focus overlay (brightened edges + selection ring) draws over
		// the base. For §5 we re-draw both on focus change; future
		// optimization could split into two physical Canvas layers
		// per the Concept Paper §11.1 perf strategy.
		if (focusedStar) {
			renderFocusOverlay(ctx, focusedStar, incidentEdges, starsByPath);
		}
	}

	$effect(() => {
		// Touch reactive deps so this effect re-runs when they change.
		void canvasWidth; void canvasHeight; void domeRadius; void $locale;
		void modeWedgeAngles; void stars; void links; void focusedStar;
		draw();
	});

	// ─── data load ─────────────────────────────────────────────────
	// 2026-05-12 boot-path hot-fix: warm_cache runs the §2 backfill
	// LAZILY on first SightV5 mount (init_db no longer does it; the
	// bulk INSERT...SELECT was O(N²) without target_path index, hung
	// app boot on 7,636-note universes). Loading state stays visible
	// until both warm_cache + get_layout return.
	async function loadLayout() {
		try {
			isLoading = true;
			// Step 1: warm the cache (runs backfill if sentinel missing;
			// fast no-op if already done).
			await invoke<number>('sight_v5_warm_cache');
			// Step 2: read the populated cache.
			const rows = await invoke<LayoutCacheRow[]>('sight_v5_get_layout', {
				scopeKind: 'universe',  // scope filter applies frontend-side per D-V3
				scopeId: null,
			});
			allRows = rows;
			// Pull link edges for the loaded notes — capped to 2000 paths
			// to keep the IPC payload bounded on large universes; future
			// polish: chunk if needed.
			const paths = rows.slice(0, 2000).map(r => r.notePath);
			if (paths.length > 0) {
				const edges = await invoke<LinkEdge[]>('sight_v5_get_link_set_for_notes', { paths });
				links = edges;
			}
		} catch (err) {
			console.error('[sight v5] loadLayout failed', err);
		} finally {
			isLoading = false;
		}
	}

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
		void loadLayout();
		// Esc clears selection (also handled at +layout.svelte for app-wide Esc).
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape' && selectedStar) {
				selectedStar = null;
				e.stopPropagation();
			}
		};
		window.addEventListener('keydown', onKey, true);
		return () => {
			ro.disconnect();
			window.removeEventListener('keydown', onKey, true);
		};
	});

	// ─── mode + scope toggle handlers ──────────────────────────────
	function setMode(m: SightV5Mode) {
		if (m === activeMode) return;
		$appSettings.sight = { ...($appSettings.sight ?? {}), lastMode: m };
		saveSettings();
		// Clear selection on mode change — the star's wedge position
		// just changed; old selection coords are stale.
		selectedStar = null;
		hoveredStar = null;
	}
	function setScope(s: SightV5Scope) {
		if (s === activeScope) return;
		$appSettings.sight = { ...($appSettings.sight ?? {}), lastScope: s };
		saveSettings();
	}

	function modeState(m: SightV5Mode): 'active' | 'ready' {
		return m === activeMode ? 'active' : 'ready';
	}
	function scopeState(s: SightV5Scope): 'active' | 'ready' {
		return s === activeScope ? 'active' : 'ready';
	}

	// ─── pointer interactions ──────────────────────────────────────
	const HIT_RADIUS_PX = 8;       // max distance from cursor to count as a hover

	function onCanvasMouseMove(e: MouseEvent) {
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		// Mouse coordinates relative to dome center.
		const mx = e.clientX - rect.left - canvasWidth / 2;
		const my = e.clientY - rect.top - canvasHeight / 2;
		let best: StarPosition | null = null;
		let bestDist = HIT_RADIUS_PX * HIT_RADIUS_PX;
		for (const s of stars) {
			const d = starHitDistanceSq(s, mx, my);
			if (d < bestDist) {
				bestDist = d;
				best = s;
			}
		}
		hoveredStar = best;
		if (best) {
			tooltipX = e.clientX - rect.left + 12;
			tooltipY = e.clientY - rect.top + 12;
		}
	}

	function onCanvasMouseLeave() {
		hoveredStar = null;
	}

	function onCanvasClick(e: MouseEvent) {
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const mx = e.clientX - rect.left - canvasWidth / 2;
		const my = e.clientY - rect.top - canvasHeight / 2;
		let best: StarPosition | null = null;
		let bestDist = HIT_RADIUS_PX * HIT_RADIUS_PX;
		for (const s of stars) {
			const d = starHitDistanceSq(s, mx, my);
			if (d < bestDist) {
				bestDist = d;
				best = s;
			}
		}
		// Click on a star → select; click on background → clear.
		selectedStar = best;
	}

	function noteTitleForTooltip(path: string): string {
		const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
		const name = slash >= 0 ? path.slice(slash + 1) : path;
		return name.replace(/\.md$/i, '');
	}

	// Open in editor handler — forwards to parent prop (mirror v4 pattern).
	function handleOpenInEditor(path: string) {
		const star = selectedStar;
		if (!star) return;
		onOpenNote?.(path, star.row.libraryName ?? '');
	}

	// Open Source Review for empty-state CTA.
	function openSourceReview() {
		window.dispatchEvent(new CustomEvent('constellation:open-source-review'));
	}
</script>

<div class="sight-v5-root" bind:this={containerEl}>
	<!-- Canvas — dome chrome + stars + connectors. -->
	<canvas
		class="sight-v5-canvas"
		bind:this={canvasEl}
		onmousemove={onCanvasMouseMove}
		onmouseleave={onCanvasMouseLeave}
		onclick={onCanvasClick}
	></canvas>

	<!-- Month labels HTML overlay. -->
	<div class="sight-v5-rim-labels" aria-hidden="false">
		{#each monthLabels as label (label.monthIndex)}
			<span
				class="sight-v5-rim-label"
				dir="auto"
				style="left: {canvasWidth / 2 + label.x}px; top: {canvasHeight / 2 + label.y}px;"
			>{label.label}</span>
		{/each}
	</div>

	<!-- Hover tooltip (star title; near cursor). -->
	{#if hoveredStar && !selectedStar}
		<div
			class="sight-v5-tooltip"
			dir="auto"
			style="left: {tooltipX}px; top: {tooltipY}px;"
		>{noteTitleForTooltip(hoveredStar.row.notePath)}</div>
	{/if}

	<!-- §4 mode toggle bar. -->
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

	<!-- §4 scope toggle bar. -->
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

	<!-- Mode P empty-state CTA (D-V6). -->
	{#if modePEmptyState}
		<div class="sight-v5-empty-state">
			<p>{$t('sight.v5.empty.provenance') || 'Most of your universe is unsourced.'}</p>
			<button class="sight-v5-empty-cta" onclick={openSourceReview}>
				{$t('sight.v5.empty.classify') || 'Classify via Source Review →'}
			</button>
		</div>
	{/if}

	<!-- Loading state. -->
	{#if isLoading}
		<div class="sight-v5-loading">{$t('sight.v5.loading') || 'Loading…'}</div>
	{/if}

	<!-- §5 side panel for selected star. -->
	{#if selectedStar}
		<SightV5SidePanel
			note={selectedStar.row}
			linkCount={incidentEdges.length}
			onClose={() => selectedStar = null}
			onOpenInEditor={handleOpenInEditor}
		/>
	{/if}
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
		cursor: default;
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
	.sight-v5-tooltip {
		position: absolute;
		background: rgba(26, 26, 26, 0.9);
		color: #faf6e8;
		padding: 0.25rem 0.5rem;
		border-radius: 3px;
		font-size: 0.8rem;
		pointer-events: none;
		z-index: 4;
		max-width: 280px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
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
	.sight-v5-empty-state {
		position: absolute;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		text-align: center;
		max-width: 26rem;
		background: rgba(250, 246, 232, 0.95);
		padding: 1.5rem 2rem;
		border-radius: 6px;
		border: 1px dashed #b8a98a;
		z-index: 3;
		font-style: italic;
		color: #3a3a3a;
	}
	.sight-v5-empty-state p {
		margin: 0 0 1rem 0;
	}
	.sight-v5-empty-cta {
		background: #2a4a8c;
		color: #faf6e8;
		border: none;
		padding: 0.5rem 1rem;
		border-radius: 4px;
		font-family: inherit;
		font-size: 0.9rem;
		cursor: pointer;
	}
	.sight-v5-loading {
		position: absolute;
		left: 50%;
		bottom: 24px;
		transform: translateX(-50%);
		font-size: 0.85rem;
		color: #3a3a3a;
		font-style: italic;
		opacity: 0.8;
	}
</style>

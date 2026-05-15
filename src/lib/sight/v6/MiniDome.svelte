<!--
  MIG-025 §B.1 — Sight v6 mini-dome Svelte wrapper.

  Thin Svelte component that wraps the pure-function renderer in
  miniDome.ts. One canvas per mini-dome; resize via ResizeObserver;
  re-paint on stars or highlightedPath change.

  §B.1 ships the structural skeleton; channel-specific rendering
  lands incrementally in §B.2 (confidence) → §B.3 (stage) →
  §B.4 (acts) → §B.5 (provenance). Linked brushing (§B.6) and
  cross-filter (§B.7) wire later.
-->
<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import type { StarDerived, SlotChannel } from './types';
	import { renderMiniDome, miniDomeHitTest } from './miniDome';
	import type { DomeLayout } from './anchor';

	let {
		channel,
		stars,
		anchorLayout,
		highlightedPath = null,
		onHover = () => {},
		compact = true,
		onPromote = () => {},
		onOpenNote = () => {},
	}: {
		channel: SlotChannel;
		stars: StarDerived[];
		anchorLayout: DomeLayout;
		highlightedPath?: string | null;
		/** §B.6 — fired when the cursor moves over a star (or null
		 *  when the cursor moves off all stars / leaves the canvas).
		 *  The parent (SightV6) owns the canonical hoveredPath; the
		 *  mini only PROPOSES via onHover() and receives the resolved
		 *  value back through the highlightedPath prop. */
		onHover?: (path: string | null) => void;
		/** §B.6-fix-3 — compact=true is the small mini-slot rendering
		 *  (current default). compact=false is the promoted primary-
		 *  slot rendering: bigger dots (dotRadius 3 instead of 0.75),
		 *  and clicks open notes instead of dispatching promote. */
		compact?: boolean;
		/** §B.6-fix-3 — fires when the canvas is clicked while compact
		 *  (mini slot). Parent uses this to swap primary slots. */
		onPromote?: (channel: SlotChannel) => void;
		/** §B.6-fix-3 — fires when the cursor clicks a star while NOT
		 *  compact (this mini occupies the primary slot). Parent uses
		 *  this to open the note in the editor. */
		onOpenNote?: (notePath: string) => void;
	} = $props();

	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let hostEl = $state<HTMLDivElement | null>(null);
	let canvasWidth = $state(0);
	let canvasHeight = $state(0);
	let dpr = $state(1);
	let resizeObserver: ResizeObserver | null = null;

	// §B.6-fix-4d (Eisa cycle-3 General Finding: "we need to make sure
	// that the zoom function is enabled in any dome that gets enlarged"):
	// zoom + pan state, ACTIVE only when compact=false (promoted slot).
	// Mini-slot (compact=true) renderings keep an identity transform and
	// these vars are inert. Mirrors the anchor canvas pattern in
	// SightV6.svelte (zoomScale / panX / panY / dragState / DRAG_THRESHOLD).
	// State is local — each promotion creates a fresh MiniDome instance
	// (because primary-slot and mini-grid live at different DOM positions),
	// so zoom resets to 1 on every fresh promotion automatically.
	let zoomScale = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let dragState = $state<{ startSx: number; startSy: number; startPanX: number; startPanY: number; moved: boolean } | null>(null);
	const ZOOM_MIN = 0.5;
	const ZOOM_MAX = 24;
	const DRAG_THRESHOLD = 4;

	function syncCanvasSize(): void {
		if (!canvasEl || !hostEl) return;
		const rect = hostEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;
		dpr = Math.max(1, window.devicePixelRatio || 1);
		canvasEl.width = Math.max(1, Math.floor(canvasWidth * dpr));
		canvasEl.height = Math.max(1, Math.floor(canvasHeight * dpr));
		paint();
	}

	function paint(): void {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		// §B.6-fix-3: when promoted to the primary slot (compact=false),
		// scale dots up so individual stars are visible at the larger
		// canvas size. §B.6-fix-4 (Eisa cycle-3 Stage 2/3): 5-px ⌀ per
		// Eisa's spec → radius 2.5 (was 3 in fix-3). Top-decile in
		// Acts preserves its 4× ratio relative to base (2.5 × 4 = 10 px
		// when promoted).
		// §B.6-fix-4d: also apply zoom/pan transform when promoted so
		// the user can zoom in to inspect detail (parallels the anchor
		// canvas zoom behavior). transform = identity when compact.
		if (!compact) {
			const sx = dpr * zoomScale;
			const tx = dpr * panX;
			const ty = dpr * panY;
			ctx.setTransform(sx, 0, 0, sx, tx, ty);
		}
		renderMiniDome(ctx, stars, channel, canvasWidth, canvasHeight, anchorLayout, {
			highlightedPath,
			dotRadius: compact ? 0.75 : 2.5,
		});
	}

	// §B.6 — pointer hit-test → dispatch hover upward to SightV6, which
	// updates its canonical hoveredPath. The new value flows back to ALL
	// 4 minis (and the anchor) via the highlightedPath prop, completing
	// the bidirectional linked-brushing loop with a single source of truth.
	// §B.6-fix-4d: when promoted (!compact), ALSO handles drag-pan and
	// inverts the zoom transform for hit-test.
	function handlePointerMove(ev: PointerEvent): void {
		if (!canvasEl) return;
		// Drag-pan path (only meaningful when promoted with active drag).
		if (!compact && dragState && (ev.buttons & 1) === 1) {
			const dx = ev.clientX - dragState.startSx;
			const dy = ev.clientY - dragState.startSy;
			if (!dragState.moved && Math.hypot(dx, dy) > DRAG_THRESHOLD) {
				dragState.moved = true;
			}
			if (dragState.moved) {
				panX = dragState.startPanX + dx;
				panY = dragState.startPanY + dy;
				return;
			}
		}
		const rect = canvasEl.getBoundingClientRect();
		let x = ev.clientX - rect.left;
		let y = ev.clientY - rect.top;
		// Invert zoom transform for hit-test when promoted.
		if (!compact) {
			x = (x - panX) / zoomScale;
			y = (y - panY) / zoomScale;
		}
		// World tolerance scales inversely with zoom so the screen
		// hit zone stays constant in screen pixels.
		const tol = compact ? 12 : 12 / zoomScale;
		const hit = miniDomeHitTest(stars, x, y, channel, anchorLayout, canvasWidth, canvasHeight, tol);
		if (hit !== highlightedPath) {
			onHover(hit);
		}
	}

	function handlePointerLeave(): void {
		dragState = null;
		if (highlightedPath !== null) {
			onHover(null);
		}
	}

	// §B.6-fix-4d — drag-pan start + end (only meaningful in promoted).
	function handlePointerDown(ev: PointerEvent): void {
		if (compact) return;
		dragState = {
			startSx: ev.clientX,
			startSy: ev.clientY,
			startPanX: panX,
			startPanY: panY,
			moved: false,
		};
	}

	function handlePointerUp(): void {
		if (compact) return;
		dragState = null;
	}

	// §B.6-fix-4d — wheel = zoom-toward-cursor. Mirrors SightV6's
	// handleWheel for the anchor canvas. Active only when promoted.
	function handleWheel(ev: WheelEvent): void {
		if (compact) return;
		ev.preventDefault();
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const sx = ev.clientX - rect.left;
		const sy = ev.clientY - rect.top;
		const wx = (sx - panX) / zoomScale;
		const wy = (sy - panY) / zoomScale;
		const factor = ev.deltaY < 0 ? 1.15 : 1 / 1.15;
		const nextScale = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, zoomScale * factor));
		if (nextScale === zoomScale) return;
		panX = sx - wx * nextScale;
		panY = sy - wy * nextScale;
		zoomScale = nextScale;
	}

	// §B.6-fix-4d — Cmd-0 / Ctrl-0 reset zoom + pan (parity with anchor).
	function handleKey(ev: KeyboardEvent): void {
		if (compact) return;
		if (ev.key === '0' && (ev.ctrlKey || ev.metaKey)) {
			ev.preventDefault();
			zoomScale = 1;
			panX = 0;
			panY = 0;
		}
	}

	// §B.6-fix-6 (Eisa cycle-5 Stages 2+3): click semantics unified
	// across compact + non-compact. Star click → open the note in EITHER
	// slot ("I want to check any notes by clicking on any star, either
	// through the main dome or the mini-domes"). Empty-space click →
	// channel-specific:
	//   • compact (mini slot): promote into primary slot ("if I want to
	//     promote that mini-dome to be a main one it will be through
	//     the dome space itself, not the stars").
	//   • non-compact (primary slot): no-op — use the demoted-anchor
	//     mini-slot click to swap back, or the header Reset View button.
	// §B.6-fix-4d: ignore clicks that were actually drag-pans.
	function handleClick(ev: MouseEvent): void {
		if (!canvasEl) return;
		if (dragState?.moved) {
			dragState = null;
			return;
		}
		const rect = canvasEl.getBoundingClientRect();
		let x = ev.clientX - rect.left;
		let y = ev.clientY - rect.top;
		// Invert zoom transform for hit-test in promoted slot only;
		// compact mini canvases have identity transform.
		if (!compact) {
			x = (x - panX) / zoomScale;
			y = (y - panY) / zoomScale;
		}
		const tol = compact ? 12 : 12 / zoomScale;
		const hit = miniDomeHitTest(stars, x, y, channel, anchorLayout, canvasWidth, canvasHeight, tol);
		if (hit) {
			// Star click: same in both slots → open the note.
			onOpenNote(hit);
			return;
		}
		// Empty-space click: only meaningful in compact (promote).
		if (compact) {
			onPromote(channel);
		}
	}

	onMount(() => {
		syncCanvasSize();
		if (hostEl) {
			resizeObserver = new ResizeObserver(() => syncCanvasSize());
			resizeObserver.observe(hostEl);
		}
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// §B.6-fix-4d — wheel listener via $effect so it re-attaches on
	// every canvas mount. Mirrors SightV6's pattern (which exists for
	// the same reason: the anchor canvas mounts/unmounts on dome-swap).
	// Imperative addEventListener is required (Tauri WebView2 + Svelte 5
	// onwheel silent-fails in release builds — §A.14 fix-11 lesson).
	$effect(() => {
		const el = canvasEl;
		if (!el) return;
		el.addEventListener('wheel', handleWheel, { passive: false });
		return () => {
			el.removeEventListener('wheel', handleWheel);
		};
	});

	// §B.6-fix-4d — repaint when zoom or pan changes (only meaningful
	// when promoted, but the effect fires regardless and paint() is
	// idempotent).
	$effect(() => {
		void zoomScale;
		void panX;
		void panY;
		untrack(() => paint());
	});

	// Repaint on star set, anchor layout, or highlight change.
	$effect(() => {
		void stars;
		void highlightedPath;
		void anchorLayout;
		untrack(() => paint());
	});
</script>

<div bind:this={hostEl} class="mini-dome-host">
	<canvas
		bind:this={canvasEl}
		class="mini-dome-canvas"
		class:has-hover={highlightedPath !== null}
		class:is-promoted={!compact}
		class:is-dragging={dragState?.moved}
		onpointermove={handlePointerMove}
		onpointerdown={handlePointerDown}
		onpointerup={handlePointerUp}
		onpointerleave={handlePointerLeave}
		onclick={handleClick}
		onkeydown={handleKey}
		tabindex={compact ? -1 : 0}
	></canvas>
</div>

<style>
	.mini-dome-host {
		position: relative;
		width: 100%;
		height: 100%;
		min-width: 0;
		min-height: 0;
	}
	.mini-dome-canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		display: block;
		/* §B.6-fix-4 (Eisa cycle-3 Stage 1.3): cursor: pointer always
		   on a mini in a mini slot — clicking anywhere promotes the
		   channel into the primary slot, so the whole canvas is a
		   click target. The pointer cursor signals this affordance. */
		cursor: pointer;
	}
	/* When promoted to the primary slot (compact=false), the canvas
	   becomes a regular inspection surface: click on a star opens
	   the note; click on empty area is a no-op. Cursor reflects: */
	.mini-dome-canvas.is-promoted {
		cursor: default;
	}
	.mini-dome-canvas.is-promoted.has-hover {
		cursor: pointer;
	}
	.mini-dome-canvas.is-promoted.is-dragging {
		cursor: grabbing;
	}
</style>

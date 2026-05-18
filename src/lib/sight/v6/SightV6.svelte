<!--
  MIG-025 §A.6/§A.7/§A.8/§A.9/§A.10 — Sight v6 main component.

  §A.6  — placeholder mount
  §A.7  — wired into +layout.svelte (B2 dual-mount)
  §A.8  — chrome paints (5 strata + calendar rim + labels)
  §A.9  — IPC integration: warm_cache → render_ready event → load
          layout + links → compute positions → paint stars + lines.
          Pointer events → hit-test → openNote callback.
  §A.10 — facet sidebar (Hearst Flamenco, 6 facets, Folder TOP).
          Filter state held in this component; facets.ts does the
          filter logic + Hearst preview-count computation.

  §A.11 lands the first-boot tour.
  §A.12 lands the v5→v6 settings migration.
  §A.13 lands the CI perf harness.

  Visual contract: docs/sight-redesign-v0.3-full-layout.svg
  Concept Paper:    docs/Constellation-Sight-Concept-Paper-v4.0.md
-->
<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';
	import { appSettings, saveSettings } from '$lib/libraries/store';
	import { backfillProgress } from './backfillProgress.svelte';
	import {
		renderAnchorDome,
		computeStarPositions,
		computeDomeLayout,
		starHitTest,
		type DomeLayout,
	} from './anchor';
	import {
		emptyFilters,
		applyFilters,
		computeFacetCounts,
		toggleFilter,
		filtersEmpty,
		confidenceLevelOf,
		provenanceSectorOf,
		type FacetFilters,
	} from './facets';
	import { bandForRawStratum, readChromePalette } from './dome';
	import FacetSidebar from './facetSidebar.svelte';
	import Tour from './tour.svelte';
	import MiniDome from './MiniDome.svelte';
	import TraditionChip from './traditionChip.svelte';
	import { getTraditionById, registerUserTraditions } from './traditions';
	import { loadUserTraditions, type UserTraditionModule } from './traditions/userDefinedLoader';
	import type { LayoutCacheRow, LinkEdge, StarDerived, FacetId, MiniDomeChannel, SlotChannel, TraditionId } from './types';
	import { marked } from 'marked';

	let { onOpenNote = (_path: string, _libraryName: string) => {} }: {
		onOpenNote?: (path: string, libraryName: string) => void;
	} = $props();

	// ── Canvas state ───────────────────────────────────────────────
	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let canvasHostEl = $state<HTMLDivElement | null>(null);
	let canvasWidth = $state(0);
	let canvasHeight = $state(0);
	let dpr = $state(1);

	// 2026-05-14 §A.14 fix-9 (Boss-test cycle 2 feature ask): zoom + pan.
	// Eisa: "If we enabled zoom-in/out, it would be more helpful to get
	// closer to the stars." Wheel = zoom-toward-cursor; left-drag = pan.
	// At dense centroids the user can zoom in to inspect individual
	// stars even when the unzoomed view shows brightened texture.
	let zoomScale = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	const ZOOM_MIN = 0.5;
	// 2026-05-14 §A.14 fix-16: ZOOM_MAX bumped 8 → 24 per Eisa's request
	// "I want to zoom in further, closer to the nodes, 3 times as much."
	// At 24× zoom, baseline node renders at 7.5 px screen radius (15 px
	// diameter — well past the spec'd 5 px). Aliasing artifacts from
	// sub-pixel rendering disappear; nodes render as crisp anti-aliased
	// circles. ZOOM_MAX_FOR_SIZING in anchor.ts stays at 8 so the world-
	// coord radius math (= 2.5 / 8 = 0.3125) is unchanged — bump only
	// affects how far the user can wheel in.
	const ZOOM_MAX = 24;
	let dragState: { startSx: number; startSy: number; startPanX: number; startPanY: number; moved: boolean } | null = null;
	const DRAG_THRESHOLD = 4; // px before pointermove counts as drag, not click

	// ── Data state ─────────────────────────────────────────────────
	let rows = $state<LayoutCacheRow[]>([]);
	let links = $state<LinkEdge[]>([]);
	// §C.3-fix-1 (Eisa cycle-1 Stage 3 FAIL: "The Provenance is the same
	// as before, yet the other three are similar to the anchor dome"):
	// tradition isolation per Concept Paper §11 invariant 6 requires that
	// mini-domes NEVER inherit the anchor's tradition-remapped positions.
	// Provenance was fine because it self-positions into 5 angle sectors;
	// Confidence / Stage / Acts read the anchor `stars` array directly
	// (they only recolor / resize / fade in place), so they were
	// inheriting pramāṇa's NE-quadrant pile-up.
	//   - `stars`        — tradition-REMAPPED positions, used by anchor.
	//                       For Aristotelian == default Aristotelian.
	//                       For pramāṇa / masādir / Polanyi / etc.
	//                       == tradition.remapStarPosition(...).
	//   - `starsDefault` — ALWAYS default Aristotelian positions,
	//                       used by mini-domes. Tradition-agnostic so the
	//                       4 minis stay culturally neutral regardless
	//                       of which tradition the chip is on.
	// Linked brushing (hover anchor → highlight mini, hover mini →
	// highlight anchor) works via notePath lookup which both arrays
	// share, so brushing crosses cleanly between the two coordinate
	// systems.
	let stars = $state<StarDerived[]>([]);
	let starsDefault = $state<StarDerived[]>([]);
	let hoveredPath = $state<string | null>(null);

	// ── §A.10 facet state ──────────────────────────────────────────
	let filters = $state<FacetFilters>(emptyFilters());
	let sidebarExpanded = $state(false);

	// ── §A.11 first-boot tour state ────────────────────────────────
	// Snapshot at mount: if tourSeen is false/undefined, show the
	// 4-step orientation overlay. Updates persist via saveSettings.
	let tourVisible = $state(false);

	// ── MIG-026 Phase κ.1 — user-defined traditions ────────────────
	// Populated at mount by loadUserTraditions() (which reads
	// <Universe>/.constellation/traditions/*.json via the
	// sight_v6_read_user_traditions IPC). Passed to TraditionChip so
	// the dropdown can render them alongside curated families. Also
	// pushed into the index.ts USER_REGISTRY so the anchor renderer's
	// getTraditionById lookup resolves user ids correctly.
	let userTraditions = $state<UserTraditionModule[]>([]);

	// ── §B.1 mini-domes diagnostics visibility ─────────────────────
	// Default-simple per Concept Paper §6: mini-domes hidden on
	// every Sight open. Cmd-D / Ctrl-D toggles visibility within
	// the session. Extended view (§B.10) reads appSettings.sight.extended
	// and overrides the default-hidden initial state.
	let diagnosticsVisible = $state(false);
	const MINI_DOME_CHANNELS: MiniDomeChannel[] = ['confidence', 'stage', 'acts', 'provenance'];

	// §B.6-fix-3 — dome-swap state. `primaryChannel` is whichever of
	// the 5 surfaces currently occupies the large primary canvas slot.
	// Default 'anchor' = the universe-baseline view (renderAnchorDome
	// with full chrome). Click any mini → promote it to primary;
	// click the demoted anchor (now in a mini slot) → swap back.
	// Per Eisa cycle-2: "click on every mini-dome to enlarge it to
	// the same size as the anchor dome ... the user could switch
	// between all the domes (including the main one) to check their
	// details."
	const ALL_SLOTS: SlotChannel[] = ['anchor', 'confidence', 'stage', 'acts', 'provenance'];
	let primaryChannel = $state<SlotChannel>('anchor');

	// Anchor layout snapshot for mini-domes — they need it to scale
	// star positions from anchor world coords to mini canvas coords.
	const anchorLayout: DomeLayout = $derived(
		computeDomeLayout(canvasWidth, canvasHeight),
	);

	function dismissTour(): void {
		tourVisible = false;
		appSettings.update((s) => ({
			...s,
			sight: { ...s.sight, tourSeen: true },
		}));
		saveSettings();
	}

	// ── MIG-026 Phase ι.2 — manifest disclosure modal ──────────────
	// State + handlers for the in-Sight tradition-manifest viewer. The
	// ⓘ button in the chip dropdown calls handleOpenManifest(id); the
	// modal renders the bundled markdown via marked + lazy import of
	// _manifests.generated.ts (defers ~87KB of doc strings until the
	// user first asks to see one).
	//
	// Lifecycle is tied to the chip dropdown: closeManifestModal fires
	// (a) when the X button is clicked, (b) when the click-outside-card
	// overlay is clicked, (c) automatically when the dropdown closes
	// (via the chip's onDropdownClose callback). The cascade-close
	// prevents the modal from floating over a closed dropdown after
	// Esc / click-outside / tradition switch.
	let manifestModalId = $state<string | null>(null);
	let manifestContent = $state<string | null>(null);
	const manifestModalOpen = $derived(manifestModalId !== null);

	/** Synthesize a manifest markdown body from a UserTraditionModule.
	 *  User-defined traditions don't have a pre-bundled .md file (they
	 *  ship as JSON specs); the modal renders this synthesized view
	 *  with the user's name + scope + citation inline. The structure
	 *  mirrors the curated manifests so the modal styling carries
	 *  over without changes. */
	function synthesizeUserManifest(t: UserTraditionModule): string {
		const parts: string[] = [
			`# ${t.name}`,
			'',
			`**Family**: ${t.family || 'user-defined'} · **Shape**: ${t.shape} · **id**: \`${t.id}\``,
			'',
			'## Scope',
			'',
			t.scope || '_(no scope provided in the JSON spec)_',
		];
		if (t.tooltip && t.tooltip !== t.name) {
			parts.push('', '## Tooltip', '', t.tooltip);
		}
		if (t.citation) {
			parts.push('', '## Citation', '', t.citation);
		}
		parts.push(
			'',
			'---',
			'',
			'_User-defined tradition (MIG-026 Phase κ.1 declarative loader). The full v1 schema reference lives at `docs/traditions/schema/tradition.v1.schema.json`._',
		);
		return parts.join('\n');
	}

	async function handleOpenManifest(id: string): Promise<void> {
		manifestModalId = id;
		manifestContent = null; // show loading state while the import resolves
		// User-defined traditions (matched by the user- prefix) carry
		// their manifest content inline in the loaded UserTraditionModule.
		// No bundle import needed — synthesize directly.
		if (id.startsWith('user-')) {
			const t = userTraditions.find((u) => u.id === id);
			if (manifestModalId === id) {
				manifestContent = t
					? synthesizeUserManifest(t)
					: `# Couldn't load manifest\n\nThe user-defined tradition **${id}** is no longer registered. Try restarting Constellation.`;
			}
			return;
		}
		try {
			const mod = await import('./traditions/_manifests.generated');
			// Defensive: ignore if the user closed or switched while the
			// import was in flight.
			if (manifestModalId === id) {
				manifestContent = mod.getManifest(id as TraditionId);
			}
		} catch (err) {
			// Should not happen in practice — the file is bundled at
			// build time via the prebuild script. Surface the error in
			// the modal body so a user can report it rather than seeing
			// a blank card.
			console.error('[sight-v6] failed to load manifest', err);
			if (manifestModalId === id) {
				manifestContent = `# Couldn't load manifest\n\nThe scholarly manifest for **${id}** could not be loaded. This usually means the bundle is stale — try restarting Constellation. If it persists, the build script \`scripts/build-tradition-manifests.mjs\` may need to be re-run.`;
			}
		}
	}

	function closeManifestModal(): void {
		manifestModalId = null;
		manifestContent = null;
	}

	// Strip the YAML frontmatter block (between the first pair of `---`
	// fences) so the rendered HTML doesn't show the raw metadata as a
	// table or horizontal rule. Leaves the body markdown intact.
	function stripManifestFrontmatter(md: string): string {
		if (!md.startsWith('---\n')) return md;
		const end = md.indexOf('\n---\n', 4);
		if (end === -1) return md;
		return md.slice(end + 5);
	}

	function renderManifestMarkdown(md: string): string {
		const body = stripManifestFrontmatter(md);
		return marked.parse(body, { async: false }) as string;
	}

	// Filtered row set + recomputed facet counts (Hearst preview).
	const filteredRows = $derived(applyFilters(rows, filters));
	const facets = $derived(computeFacetCounts(rows, filters));

	// §B.7-fix-2 (Eisa cycle-2 ghost mode): set of notePaths that pass
	// the current filter. null when no filter active (skip the ghost
	// pathway entirely → all stars render at full encoding). Renderers
	// (anchor + mini channel renderers) check this set per-star and
	// fade non-members to GHOST_ALPHA (0.15).
	const matchedPaths = $derived(
		filtersEmpty(filters) ? null : new Set(filteredRows.map((r) => r.notePath)),
	);

	// §B.9 (2026-05-16) — density-aggregation mode. Active when the
	// matched (or all-when-no-filter) star count exceeds
	// `appSettings.sight.hexBinThreshold` (default 5000 per Concept
	// Paper §3.4). Above the threshold, channel renderers switch
	// per-star alpha from full encoding to a low value (~0.3) so
	// overlapping stars additive-blend into a perceptual density
	// gradient — the dense regions read as "more stars here" without
	// every dot needing to be individually visible.
	//
	// This is a lightweight stand-in for the full d3-hexbin
	// aggregation the Concept Paper specifies for v6.1. Real
	// hex-binning (compute bins → render hexagons with dominant-value
	// fill + count badge) lands as v6.2 polish (PJ-NNN allocated
	// post-ship). For Eisa's 7,645-note universe, the lightweight
	// density mode is sufficient — discrete dots remain visible
	// enough to hit-test, but dense clusters no longer look like
	// solid blobs of cream.
	const densityMode = $derived.by(() => {
		const threshold = $appSettings.sight?.hexBinThreshold ?? 5000;
		const count = matchedPaths === null ? rows.length : matchedPaths.size;
		return count > threshold;
	});

	// MIG-026 §γ-fix-1 — anchor-only override of density mode for
	// spread-shape traditions. Reserved for when the anchor renderer
	// actually consumes densityMode (currently anchor.ts treats it as
	// API-symmetric-but-unused — see anchor.ts step §B.9 comment).
	// Mini-domes keep `densityMode` (above) unchanged.
	const anchorDensityMode = $derived.by(() => {
		if (!densityMode) return false;
		const tradition = getTraditionById(
			$appSettings.sight?.activeTradition as TraditionId | undefined,
		);
		if (tradition?.shape === 'horizontal-bands') return false;
		return true;
	});

	// MIG-026 §γ-fix-2 (Eisa Boss test 2026-05-17) — anchor star radius
	// boost in SCREEN pixels for spread-shape traditions. The default
	// BASE_STAR_RADIUS (5 px ⌀ @ 8× zoom = ~0.6 px ⌀ @ 1× zoom) was
	// tuned for Aristotelian's concentrated clusters where overlapping
	// sub-pixel dots additive-blend into a milky-way texture. In spread
	// layouts (Mohist horizontal-bands; future grid/rings/relational)
	// stars don't overlap and individual sub-pixel dots dissolve into
	// the bg. Boss test 2026-05-17: "I think it is not completely
	// visible because of the star's size. Let's pump up the size by
	// 2px, just for this type." +2 px to the body radius (~4 px to the
	// visible diameter) reads clearly in spread layouts without making
	// cluster-style layouts look gloopy.
	const anchorStarRadiusBoostScreenPx = $derived.by(() => {
		const tradition = getTraditionById(
			$appSettings.sight?.activeTradition as TraditionId | undefined,
		);
		// §γ-fix-2 + §δ.2-fix-1 + §ε.2-fix-1: per-shape boost values
		// tuned by Boss across iterations. Two tiers:
		//   +2 px — VERY spread shapes (stars scattered across many
		//           large cells; need bigger dots to read individually):
		//             horizontal-bands (Mohist, 3 full-width stripes)
		//             grid             (Shāṭibī, 15 cells)
		//   +1.5 px — MODERATELY spread shapes (narrower spread regions
		//             — ring bands or concentric zones):
		//             cyclic-flow (Dewey, 75% ring band)
		//             rings       (Husserl, Ibn Rushd — 4 nested zones)
		//   0 px — CLUSTER shapes (default-tight Aristotelian-style
		//          concentration; additive blending serves):
		//             sectoral, gradient
		// Future spread-shapes (relational) join as their phases ship.
		switch (tradition?.shape) {
			case 'horizontal-bands':
			case 'grid':
			case 'binary-flow':
			case 'relational':
			case 'gradient':
			case 'sectoral':
				// §θ-fix-2 (Eisa Boss test 2026-05-18): 'sectoral' added
				// to the +2 tier per "bump the stars by 2px, raise the
				// opacity" for ALL sectoral traditions (Aristotelian +
				// pramāṇa + masādir + Peirce + Habermas + Longino +
				// Mencian sprouts + Korean Sŏngnihak + Akan Wiredu).
				// The opacity boost happens via BODY_OPACITY_MULT raised
				// 0.7 → 1.0 in anchor.ts (affects all traditions); the
				// size boost is per-shape here.
				// §θ-fix-1: 'gradient' (Polanyi) — fog overlay reduces
				// visibility, +2 px compensates.
				return 2;
			case 'cyclic-flow':
			case 'rings':
			case 'ladder':
				return 1.5;
			default:
				return 0;
		}
	});

	// §C.4 — extension chips from the active tradition. masādir is the
	// only tradition that ships extensionChips in Phase 3 (4 supplementary
	// sources per Concept Paper §4.1.3: istiḥsān, istiṣḥāb, maṣlaḥa
	// mursalah, ʿurf). The chip row renders as a positioned-absolute
	// HTML overlay below the anchor canvas (see template). Returns null
	// when no tradition is active or the active tradition has no extension
	// chips → conditional render is suppressed entirely.
	const traditionExtensionChips = $derived.by((): string[] | null => {
		const reg = getTraditionById(
			$appSettings.sight?.activeTradition as TraditionId | undefined,
		);
		if (!reg?.extensionChips) return null;
		const chips = reg.extensionChips();
		return chips.length === 0 ? null : chips;
	});

	// §B.7-fix-3 (Eisa cycle-3 ask): when a star is hovered anywhere in
	// Sight, derive its per-facet category values so the FacetSidebar
	// can highlight the matching chips. Reverse direction of the
	// existing forward-link (click chip → filter dome). Six facets:
	// folder / library / stratum / confidence / stage / provenance.
	// null when no star hovered → sidebar reverts to no-highlight state.
	const hoveredFacetValues = $derived.by(() => {
		if (!hoveredPath) return null;
		const row = rows.find((r) => r.notePath === hoveredPath);
		if (!row) return null;
		return {
			folder: row.folderPath ?? undefined,
			library: row.libraryName ?? undefined,
			stratum: bandForRawStratum(row.stratum),
			confidence: confidenceLevelOf(row),
			stage: row.stage ?? undefined,
			provenance: provenanceSectorOf(row.sourcesPrimary),
		} as Partial<Record<FacetId, string>>;
	});

	let resizeObserver: ResizeObserver | null = null;

	// ── Render ────────────────────────────────────────────────────

	function paint(): void {
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		// §A.14 fix-9: combined transform = DPR × zoomScale + pan offset.
		// Render math runs in CSS pixels for the unzoomed canvas; the
		// zoom+pan + DPR factors compose into setTransform.
		const sx = dpr * zoomScale;
		const tx = dpr * panX;
		const ty = dpr * panY;
		ctx.setTransform(sx, 0, 0, sx, tx, ty);
		// Filter the visible link set to edges where BOTH endpoints
		// survive the facet filter — keeps the dome readable when the
		// user narrows the universe.
		const visiblePaths = new Set(filteredRows.map((r) => r.notePath));
		const visibleLinks = links.filter(
			(l) => visiblePaths.has(l.sourcePath) && visiblePaths.has(l.targetPath),
		);
		// §C.3 — pass the active tradition so renderAnchorDome can draw
		// the tradition's sector dividers + labels (e.g., pramāṇa's 4
		// quadrant lines + wedge labels). Star positions themselves are
		// already remapped at recomputeStars time (§C.2). For Aristotelian
		// and Polanyi this is a no-op — sectorDividers is undefined on
		// those modules.
		const activeTradition = getTraditionById(
			$appSettings.sight?.activeTradition as TraditionId | undefined,
		);
		// MIG-027 — read chrome colors from CSS variables on the canvas
		// host element. The document body's CSS vars are set by
		// +layout.svelte's theme $effect (which calls deriveThemeVariables
		// on the active theme's 5 colors); so they cascade to canvasHostEl
		// and any descendant. Computed at every paint so theme changes
		// take effect on the next render (the theme-change $effect
		// further down in this file forces a repaint on activeThemeId
		// transitions).
		const chromePalette = readChromePalette(canvasHostEl);
		renderAnchorDome(ctx, stars, visibleLinks, canvasWidth, canvasHeight, {
			locale: navigator.language ?? 'en',
			highlightedPath: hoveredPath,
			zoomScale: zoomScale,
			matchedPaths,
			// MIG-026 §γ-fix-1: anchor-only density-mode override for
			// spread-shape traditions (reserved scaffolding; anchor
			// renderer currently treats densityMode as API-symmetric-
			// but-unused — see anchor.ts §B.9 comment).
			densityMode: anchorDensityMode,
			// MIG-026 §γ-fix-2: anchor-only star radius boost for
			// spread-shape traditions. 2 px for Mohist horizontal-bands
			// (and future grid/rings/relational); 0 for cluster-style
			// shapes (Aristotelian / pramāṇa / masādir / Polanyi).
			starRadiusBoostScreenPx: anchorStarRadiusBoostScreenPx,
			tradition: activeTradition,
			chromePalette,
		});
	}

	function syncCanvasSize(): void {
		if (!canvasEl || !canvasHostEl) return;
		const rect = canvasHostEl.getBoundingClientRect();
		canvasWidth = rect.width;
		canvasHeight = rect.height;
		dpr = Math.max(1, window.devicePixelRatio || 1);
		canvasEl.width = Math.max(1, Math.floor(canvasWidth * dpr));
		canvasEl.height = Math.max(1, Math.floor(canvasHeight * dpr));
		recomputeStars();
		paint();
	}

	function recomputeStars(): void {
		// §B.7-fix-2 (Eisa cycle-2 ghost mode): compute star positions
		// from the FULL universe (rows), not the filtered subset. The
		// filter only controls which stars render at full opacity vs
		// faded ghost (via matchedPaths). This way the user can hover
		// AND Shift+click on a non-matched ghost star to add its category
		// to the filter — multi-select within a facet works directly
		// from the dome instead of requiring sidebar chip interaction.
		if (rows.length === 0 || canvasWidth === 0 || canvasHeight === 0) {
			stars = [];
			starsDefault = [];
			return;
		}
		const layout = computeDomeLayout(canvasWidth, canvasHeight);
		// §C.3-fix-1 — compute TWO star arrays so mini-domes stay
		// tradition-agnostic per Concept Paper §11 invariant 6.
		//
		// `starsDefault` (no tradition arg → null → default Aristotelian
		// positions) is consumed by the 4 mini-dome instances. Result:
		// Confidence / Stage / Acts / Provenance always show the full-
		// circle stratum × time arrangement regardless of which tradition
		// the chip is on.
		//
		// `stars` (with the active tradition) is consumed by the anchor.
		// For Aristotelian → identity → same as starsDefault. For
		// pramāṇa / masādir / Polanyi / Ishrāqī / Mohist sān biǎo (as
		// each module ships) → tradition.remapStarPosition rearranges
		// stars into the tradition's geometric vocabulary.
		//
		// Linked brushing crosses cleanly: hoveredPath is the notePath
		// string which both arrays share; the gold-ring highlight in
		// each surface lands at that surface's own coordinate for the
		// matching row.
		const activeTradition = getTraditionById(
			$appSettings.sight?.activeTradition as TraditionId | undefined,
		);
		starsDefault = computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius);
		stars = activeTradition
			? computeStarPositions(rows, layout.centerX, layout.centerY, layout.radius, activeTradition)
			: starsDefault;
	}

	// ── Data load ─────────────────────────────────────────────────

	async function loadLayoutAndLinks(): Promise<void> {
		try {
			rows = await invoke<LayoutCacheRow[]>('sight_v6_get_layout');
			recomputeStars();
			if (rows.length > 0) {
				const paths = rows.map((r) => r.notePath);
				links = await invoke<LinkEdge[]>('sight_v6_get_link_set_for_notes', { paths });
			}
			paint();
		} catch (err) {
			console.error('[Sight v6] loadLayoutAndLinks failed:', err);
		}
	}

	async function startWarmCache(): Promise<void> {
		try {
			await invoke<number>('sight_v6_warm_cache');
		} catch (err) {
			console.error('[Sight v6] warm_cache failed:', err);
		}
	}

	// ── Pointer events ────────────────────────────────────────────

	function pointerToCanvas(ev: { clientX: number; clientY: number }): { x: number; y: number } | null {
		if (!canvasEl) return null;
		const rect = canvasEl.getBoundingClientRect();
		// Screen → canvas-CSS-pixel.
		const sx = ev.clientX - rect.left;
		const sy = ev.clientY - rect.top;
		// §A.14 fix-9: convert screen → world (unzoomed) coordinates.
		// The renderer draws in world space transformed by zoomScale+pan;
		// hit-test must invert the transform to compare against world-
		// coord star positions.
		return { x: (sx - panX) / zoomScale, y: (sy - panY) / zoomScale };
	}

	// 2026-05-14 §A.14 fix-4 (Boss-test #4 feedback): hover tooltip
	// showed raw notePath ("Research/Botany/Apple Tree Fruit.md").
	// Extract just the filename-without-.md as the human-readable
	// title; show the full path as a secondary line for disambiguation
	// when there are folders in the path.
	function noteTitle(path: string): string {
		const last = path.split('/').pop() || path;
		return last.replace(/\.md$/i, '');
	}

	function handlePointerMove(ev: PointerEvent): void {
		// §A.14 fix-9: drag-pan support. When mouse button held + moved
		// past DRAG_THRESHOLD, treat as pan rather than hover.
		if (dragState && (ev.buttons & 1) === 1) {
			const dx = ev.clientX - dragState.startSx;
			const dy = ev.clientY - dragState.startSy;
			if (!dragState.moved && Math.hypot(dx, dy) > DRAG_THRESHOLD) {
				dragState.moved = true;
			}
			if (dragState.moved) {
				panX = dragState.startPanX + dx;
				panY = dragState.startPanY + dy;
				paint();
				return;
			}
		}
		const pt = pointerToCanvas(ev);
		if (!pt) return;
		// Hit-test tolerance is in screen px; divide by zoom for world px.
		const hit = starHitTest(stars, pt.x, pt.y, 9 / zoomScale);
		if (hit !== hoveredPath) {
			hoveredPath = hit;
			paint();
		}
	}

	function handlePointerDown(ev: PointerEvent): void {
		// §A.14 fix-9: start of potential drag-pan.
		dragState = {
			startSx: ev.clientX,
			startSy: ev.clientY,
			startPanX: panX,
			startPanY: panY,
			moved: false,
		};
	}

	function handlePointerUp(): void {
		dragState = null;
	}

	function handlePointerLeave(): void {
		dragState = null;
		if (hoveredPath !== null) {
			hoveredPath = null;
			paint();
		}
	}

	function handleClick(ev: MouseEvent): void {
		// §A.14 fix-9: ignore clicks that were actually drags.
		if (dragState?.moved) {
			dragState = null;
			return;
		}
		// §B.7-fix-1 (Eisa cycle-1 Stage 4): Shift+click on the anchor
		// dome is a no-op. Anchor has no channel-specific category
		// (it's the universe-baseline view), so cross-filter doesn't
		// apply here. Without this guard, Shift+click fell through to
		// onOpenNote and opened the note instead of being silent — the
		// guard mirrors MiniDome's handling for 'anchor' / 'acts' channels.
		if (ev.shiftKey) return;
		const pt = pointerToCanvas(ev);
		if (!pt) return;
		const hit = starHitTest(stars, pt.x, pt.y, 9 / zoomScale);
		if (!hit) return;
		const row = rows.find((r) => r.notePath === hit);
		if (row && row.libraryName) {
			onOpenNote(row.notePath, row.libraryName);
		}
	}

	// §A.14 fix-9: mouse-wheel zoom-toward-cursor.
	function handleWheel(ev: WheelEvent): void {
		ev.preventDefault();
		if (!canvasEl) return;
		const rect = canvasEl.getBoundingClientRect();
		const sx = ev.clientX - rect.left;
		const sy = ev.clientY - rect.top;
		// World point under cursor BEFORE zoom.
		const wx = (sx - panX) / zoomScale;
		const wy = (sy - panY) / zoomScale;
		// Zoom: positive deltaY (scroll down) zooms OUT; smooth ratio.
		const factor = ev.deltaY < 0 ? 1.15 : 1 / 1.15;
		const nextScale = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, zoomScale * factor));
		if (nextScale === zoomScale) return;
		// Adjust pan so the world point under cursor stays under cursor.
		panX = sx - wx * nextScale;
		panY = sy - wy * nextScale;
		zoomScale = nextScale;
		paint();
	}

	// §A.14 fix-9: keyboard escape resets zoom + pan as a last-resort
	// "I lost the dome" recovery. Esc still also closes Sight v6 via
	// +layout.svelte's escape handler — that handler runs FIRST when
	// nothing-zoomed; this resets when zoomed.
	// §B.1: also handles Cmd-D / Ctrl-D for mini-domes diagnostics
	// toggle. Listed in Concept Paper §5 gesture grammar.
	function handleKey(ev: KeyboardEvent): void {
		if (ev.key === '0' && (ev.ctrlKey || ev.metaKey)) {
			ev.preventDefault();
			zoomScale = 1;
			panX = 0;
			panY = 0;
			paint();
		} else if ((ev.key === 'd' || ev.key === 'D') && (ev.ctrlKey || ev.metaKey) && !ev.shiftKey) {
			// Cmd-D / Ctrl-D — toggle mini-domes diagnostics visibility.
			// Session-only — does NOT touch persistent sight.extended.
			// Excludes Shift to avoid colliding with Cmd-Shift-D below.
			ev.preventDefault();
			diagnosticsVisible = !diagnosticsVisible;
		} else if ((ev.key === 'd' || ev.key === 'D') && (ev.ctrlKey || ev.metaKey) && ev.shiftKey) {
			// §B.10 — Cmd-Shift-D / Ctrl-Shift-D — toggle the extended-view
			// setting PERSISTENTLY. Extended view = minis default-visible
			// on every Sight open (instead of default-hidden). State
			// stored in appSettings.sight.extended; survives across
			// sessions via saveSettings. Also flips diagnosticsVisible
			// to match the new value, so the toggle has immediate effect
			// in the current session.
			// §B.10-fix-1 (Eisa cycle-1, 2026-05-16): field name renamed
			// `proMode` → `extended` per Eisa: "Pro" overpromised, the
			// feature only controls default view extent. Migration in
			// applyParsedSettings carries existing users' values forward.
			ev.preventDefault();
			const newExtended = !($appSettings.sight?.extended ?? false);
			appSettings.update((s) => ({
				...s,
				sight: { ...s.sight, extended: newExtended },
			}));
			saveSettings();
			diagnosticsVisible = newExtended;
		}
	}

	// ── §A.10 facet handlers ──────────────────────────────────────

	function handleFacetToggle(facet: FacetId, categoryId: string): void {
		filters = toggleFilter(filters, facet, categoryId);
	}

	function handleSidebarExpandToggle(): void {
		sidebarExpanded = !sidebarExpanded;
	}

	// ── Lifecycle ─────────────────────────────────────────────────

	onMount(async () => {
		await backfillProgress.start();
		syncCanvasSize();
		if (canvasHostEl) {
			resizeObserver = new ResizeObserver(() => syncCanvasSize());
			resizeObserver.observe(canvasHostEl);
		}
		startWarmCache();
		// MIG-026 Phase κ.1 — load user-defined traditions from the
		// active Universe's .constellation/traditions/ folder. The
		// IPC + validator are designed to be best-effort: missing
		// directory → empty result, malformed files → console.warn
		// + skip. Anchor renderer + chip dropdown re-read on next
		// repaint cycle automatically (the $effect on activeTradition
		// already triggers a recompute when settings change).
		try {
			const loaded = await loadUserTraditions();
			registerUserTraditions(loaded);
			userTraditions = loaded;
			if (loaded.length > 0) {
				console.log(
					`[sight-v6] loaded ${loaded.length} user-defined tradition(s) from .constellation/traditions/`,
				);
				recomputeStars();
				paint();
			}
		} catch (err) {
			console.warn('[sight-v6] user-tradition load failed (continuing without):', err);
		}
		// §A.11 — fire the tour if user hasn't seen it yet. Snapshot
		// (no $store subscription needed for the show-once gate).
		// §B.10 — also read sight.extended here: if extended view was
		// enabled in a previous session (via Cmd-Shift-D), default
		// diagnosticsVisible=true so the minis are shown on this open.
		// Per-session Cmd-D toggle (handleKey case below) still works
		// to temporarily hide the minis without touching the persistent
		// extended setting.
		const settingsSnapshot = get(appSettings);
		if (!settingsSnapshot.sight?.tourSeen) {
			tourVisible = true;
		}
		if (settingsSnapshot.sight?.extended) {
			diagnosticsVisible = true;
		}
	});

	onDestroy(() => {
		backfillProgress.stop();
		resizeObserver?.disconnect();
		resizeObserver = null;
	});

	// §B.6-fix-3 — wheel listener moved out of onMount and into a
	// $effect keyed on canvasEl, because the anchor canvas now mounts/
	// unmounts based on primaryChannel (dome-swap). When primaryChannel
	// flips between 'anchor' and a mini channel, canvasEl rebinds; the
	// effect re-attaches the listener to the new element and the cleanup
	// detaches from the old. Replaces the §A.14 fix-11 onMount block.
	// Imperative addEventListener kept (NOT Svelte template binding) per
	// the original fix-11 lesson — Tauri WebView2 + Svelte 5 onwheel
	// silent-fails in release builds.
	$effect(() => {
		const el = canvasEl;
		if (!el) return;
		el.addEventListener('wheel', handleWheel, { passive: false });
		return () => {
			el.removeEventListener('wheel', handleWheel);
		};
	});

	// §B.6-fix-3 — re-sync canvas dimensions after the primary slot
	// swaps. Mounting a fresh canvas via {#if} doesn't fire the
	// ResizeObserver (host size unchanged), so syncCanvasSize must be
	// called explicitly when canvasEl rebinds.
	$effect(() => {
		void canvasEl;
		void primaryChannel;
		untrack(() => {
			if (canvasEl) syncCanvasSize();
		});
	});

	// §B.6-fix-3 — promoted-mini click → open note. Looks up the row
	// to find libraryName, then dispatches the parent's onOpenNote.
	function handlePromotedOpenNote(notePath: string): void {
		const row = rows.find((r) => r.notePath === notePath);
		if (row && row.libraryName) {
			onOpenNote(notePath, row.libraryName);
		}
	}

	// §B.6-fix-3 — swap a different channel into the primary slot.
	// Resets zoom + pan (the previous slot's transform doesn't apply
	// to the new primary). hoveredPath stays — linked brushing is
	// continuous across swaps.
	function handlePromote(slot: SlotChannel): void {
		if (slot === primaryChannel) return;
		primaryChannel = slot;
		zoomScale = 1;
		panX = 0;
		panY = 0;
	}

	// §B.6-fix-4c (Eisa cycle-3 Stage 7): explicit "Reset View" returns
	// to the default layout — anchor in primary slot + zoom 1 + no pan.
	// Header button exposes this when there's something to reset.
	function handleResetView(): void {
		primaryChannel = 'anchor';
		zoomScale = 1;
		panX = 0;
		panY = 0;
	}

	// React to backfill render-ready: load layout once tier 1 done.
	$effect(() => {
		if (backfillProgress.renderReady) {
			untrack(() => loadLayoutAndLinks());
		}
	});

	// Repaint on geometry change.
	$effect(() => {
		void canvasWidth;
		void canvasHeight;
		void dpr;
		untrack(() => paint());
	});

	// §A.10 — repaint AND recompute star positions when the row set
	// changes (data load or refresh).
	// §B.7-fix-2 — recompute is now driven by `rows` (full universe),
	// not `filteredRows`. Filter changes only need a repaint, not a
	// star-position recompute, because stars are positioned over the
	// full universe and ghost-rendering happens per-star at paint time.
	$effect(() => {
		void rows;
		untrack(() => {
			recomputeStars();
			paint();
		});
	});

	// Repaint when filter set changes (matchedPaths updates, ghost
	// rendering needs to refresh).
	$effect(() => {
		void filteredRows;
		untrack(() => paint());
	});

	// Repaint on sidebar expand/collapse (changes canvas-host width).
	// Use a microtask so the layout stabilizes before syncCanvasSize.
	$effect(() => {
		void sidebarExpanded;
		queueMicrotask(() => {
			untrack(() => syncCanvasSize());
		});
	});

	// §B.6 — repaint anchor when hoveredPath changes from any source.
	// Forward direction (anchor pointermove → hoveredPath) already calls
	// paint() explicitly inside handlePointerMove. Reverse direction
	// (mini-dome onHover → hoveredPath) needs this effect to redraw the
	// anchor's gold highlight ring. paint() is idempotent so the
	// occasional double-paint on forward-direction hover is harmless.
	$effect(() => {
		void hoveredPath;
		untrack(() => paint());
	});

	// MIG-027 — repaint anchor when the active interface theme changes.
	// Reads $appSettings.activeThemeId (set by SettingsModal theme
	// picker) and $appSettings.colorScheme (system/light/dark for
	// auto-pairing). +layout.svelte's theme $effect updates document.body's
	// CSS variables synchronously when these change; our paint() then
	// re-reads the new CSS-var values via readChromePalette. No recompute
	// needed — star positions are theme-independent.
	$effect(() => {
		void $appSettings.activeThemeId;
		void $appSettings.colorScheme;
		untrack(() => paint());
	});

	// §C.2 — recompute + repaint when the active epistemic tradition
	// changes. Reads $appSettings.sight.activeTradition so this effect
	// re-fires whenever traditionChip.svelte writes a new id. The
	// recompute is necessary because each star's (x, y) is now a
	// function of the tradition's remapStarPosition; with Aristotelian
	// active the values are identical to default so this is a no-op
	// repaint, but for pramāṇa / masādir / Polanyi / Ishrāqī / Mohist
	// (once their modules ship) the stars actually move into new
	// positions. Wrapped in untrack so the internal writes inside
	// recomputeStars + paint don't trigger infinite re-runs. Reading
	// `void` of the optional chain short-circuits cleanly when
	// $appSettings.sight is undefined (very-early-boot edge case).
	$effect(() => {
		void $appSettings.sight?.activeTradition;
		untrack(() => {
			recomputeStars();
			paint();
		});
	});
</script>

<div class="sight-v6-root">
	<div class="sight-v6-header">
		<span class="sight-v6-title">Constellation Sight</span>
		<span class="sight-v6-subtitle">v6.3 — Traditions (Phase 1)</span>
		<!-- §C.1 — Tradition chip. Sits between the subtitle and the
		     EXTENDED badge per Concept Paper §2.5. Default state shows
		     only the active tradition (collapsed); click to expand the
		     full 7-chip row. Click any chip → switches activeTradition
		     (writes to appSettings.sight.activeTradition via the canonical
		     update+saveSettings pattern; partial-ships §C.8 persistence).
		     Hover any chip → English secondary label tooltip per §11
		     invariant. v1-preview traditions (Dignāga / Suhrawardi
		     Ishrāqī / Mohist sān biǎo) carry a "preview" badge per §4.2. -->
		<TraditionChip
			openManifest={handleOpenManifest}
			onDropdownClose={closeManifestModal}
			{userTraditions}
		/>
		<!-- §B.10 — small "EXTENDED" indicator when the persistent
		     extended-view setting is on (Cmd-Shift-D toggles). Per
		     Concept Paper §11 invariant 9 (no persistent toggle bars),
		     this is informational only — clicking it doesn't toggle.
		     Cmd-Shift-D is the toggle.
		     §B.10-fix-1 (Eisa cycle-1, 2026-05-16): full rename —
		     user-facing label "Pro" → "Extended" AND internal field
		     name `proMode` → `extended` per Eisa: "even the internal
		     appSettings.sight.proMode field name has to change."
		     Settings migration in applyParsedSettings carries forward
		     existing users' values. -->
		{#if $appSettings.sight?.extended}
			<span class="sight-v6-pro-badge" title="Extended view is ON — minis default-visible on every Sight open. Cmd-Shift-D to toggle.">
				EXTENDED
			</span>
		{/if}
		<!-- §B.7-fix-1 (Eisa cycle-1 Stage 1 ask: "we need to add a
		     count of affected notes when shift-clicking"): when any
		     facet filter is active, show the filtered/total count in
		     the header so the user sees the immediate impact of their
		     Shift+click without needing to inspect the sidebar. -->
		{#if !filtersEmpty(filters)}
			<span class="sight-v6-filter-count" title="Filtered notes / total notes">
				{filteredRows.length.toLocaleString()} / {rows.length.toLocaleString()} notes
			</span>
		{/if}
		<!-- §B.6-fix-4c — Reset View button. Visible when the layout
		     has been changed away from the default (anchor primary at
		     zoom 1.0). Clicking returns to default in one tap, no need
		     to manually swap back through the demoted-anchor mini. -->
		{#if primaryChannel !== 'anchor' || zoomScale !== 1 || panX !== 0 || panY !== 0}
			<button class="sight-v6-reset-btn" onclick={handleResetView} title="Return to anchor dome at default zoom (Reset View)">
				Reset View
			</button>
		{/if}
	</div>

	<div class="sight-v6-body">
		<FacetSidebar
			{facets}
			{filters}
			expanded={sidebarExpanded}
			onToggle={handleFacetToggle}
			onExpandToggle={handleSidebarExpandToggle}
			{hoveredFacetValues}
		/>

		<div bind:this={canvasHostEl} class="sight-v6-canvas-host" class:has-minis={diagnosticsVisible}>
			{#if primaryChannel === 'anchor'}
				<canvas
					bind:this={canvasEl}
					class="sight-v6-canvas"
					class:has-hover={hoveredPath !== null}
					class:is-dragging={dragState?.moved}
					onpointermove={handlePointerMove}
					onpointerdown={handlePointerDown}
					onpointerup={handlePointerUp}
					onpointerleave={handlePointerLeave}
					onclick={handleClick}
					onkeydown={handleKey}
					tabindex="0"
				></canvas>
				<!-- §A.14 fix-11: zoom indicator. Renders ONLY when zoom != 1.0
				     so it doesn't clutter the default view. If wheel fires and
				     state updates, this badge appears + reflects zoom level even
				     if the render pipeline is broken — clean diagnostic for
				     "did wheel fire" vs "did render apply". Cmd-0 hides it
				     by resetting zoom to 1. -->
				{#if zoomScale !== 1 || panX !== 0 || panY !== 0}
					<div class="sight-v6-zoom-badge">
						zoom: {zoomScale.toFixed(2)}× · pan: {Math.round(panX)},{Math.round(panY)} · Ctrl-0 reset
					</div>
				{/if}
				<!-- §C.4 — extension chips overlay. Active for masādir (the
				     only Phase-3 tradition with supplementary sources per
				     §4.1.3: istiḥsān / istiṣḥāb / maṣlaḥa mursalah / ʿurf).
				     Position absolute at the bottom-center of the canvas
				     host so the chip row sits below the dome without
				     stealing canvas height. Pointer-events: none on the
				     wrapper so the chips don't intercept dome hover/click;
				     they are display-only in §C.4 (per-note opt-in via
				     `masadir_source` frontmatter ships in §C.4-fix-N). -->
				{#if traditionExtensionChips}
					<div class="sight-v6-extension-chips" role="list" aria-label="Additional masādir sources">
						{#each traditionExtensionChips as chip (chip)}
							<span class="extension-chip" role="listitem" title="Additional masādir source (display-only in v6.2; per-note opt-in via frontmatter ships in a follow-up)">
								{chip}
							</span>
						{/each}
					</div>
				{/if}
			{:else}
				<!-- §B.6-fix-3 — promoted mini in primary slot. Fills the
				     canvas-host space. Uses MiniDome with compact=false so
				     the channel renders with bigger dots (radius 3 vs 0.75)
				     and a click on a star opens it in the editor (callback
				     wired below to handlePromotedOpenNote). The original
				     anchor canvas is unmounted; its zoom/pan state resets
				     when the user swaps anchor back into primary. -->
				<div class="sight-v6-promoted-host">
					<MiniDome
						channel={primaryChannel}
						stars={starsDefault}
						{anchorLayout}
						highlightedPath={hoveredPath}
						onHover={(path) => { hoveredPath = path; }}
						compact={false}
						onOpenNote={handlePromotedOpenNote}
						onFacetFilter={handleFacetToggle}
						{matchedPaths}
					/>
				</div>
			{/if}
			{#if !backfillProgress.renderReady}
				<div class="sight-v6-loading">
					{#if backfillProgress.progress}
						Sight v6 cache: tier {backfillProgress.progress.tier}/5
						({backfillProgress.progress.doneRows}/{backfillProgress.progress.totalRows})
					{:else}
						Preparing Sight v6 cache…
					{/if}
				</div>
			{:else if !backfillProgress.done}
				<div class="sight-v6-loading sight-v6-loading-bg">
					Tier {backfillProgress.progress?.tier}/5 streaming…
				</div>
			{/if}
			{#if hoveredPath}
				<div class="sight-v6-hover-info">
					<span class="sight-v6-hover-title">{noteTitle(hoveredPath)}</span>
					{#if hoveredPath !== noteTitle(hoveredPath) + '.md' && hoveredPath !== noteTitle(hoveredPath)}
						<span class="sight-v6-hover-path">{hoveredPath}</span>
					{/if}
				</div>
			{/if}
			{#if tourVisible}
				<Tour onComplete={dismissTour} />
			{/if}
		</div>

		{#if diagnosticsVisible}
			<!-- §B.1: 2×2 mini-domes grid. Channel-specific renderers per §B.2-§B.5.
			     §B.6-fix-3: iterates ALL_SLOTS (anchor + 4 channels) skipping the
			     primary, so the demoted anchor takes the slot vacated by whichever
			     channel was promoted. Click any mini → handlePromote swaps it into
			     the primary slot. -->
			<div class="sight-v6-minis-grid">
				{#each ALL_SLOTS as slot (slot)}
					{#if slot !== primaryChannel}
						<div class="sight-v6-mini-cell">
							<!-- §B.6-fix-6: onOpenNote also passed to compact
							     mini-grid instances per Eisa cycle-5: a star
							     click in any dome (mini or primary) opens
							     the note. Click on empty space promotes (per
							     fix-3); the mini's handleClick hit-tests
							     first, so star vs empty is unambiguous. -->
							<!-- §B.7 — onFacetFilter dispatched on Shift+click on a
							     star; reuses the existing handleFacetToggle handler
							     (same pipeline the facet sidebar uses), so the cross-
							     filter applies uniformly across all 5 surfaces via
							     filteredRows → recomputeStars → repaint. -->
							<MiniDome
								channel={slot}
								stars={starsDefault}
								{anchorLayout}
								highlightedPath={hoveredPath}
								onHover={(path) => { hoveredPath = path; }}
								compact={true}
								onPromote={handlePromote}
								onOpenNote={handlePromotedOpenNote}
								onFacetFilter={handleFacetToggle}
								{matchedPaths}
							{densityMode}
							/>
						</div>
					{/if}
				{/each}
			</div>
		{/if}
	</div>

	<!-- MIG-026 Phase ι.2 — manifest disclosure modal. Mounted at
	     sight-v6-root level (sibling of header + body) so it overlays
	     the entire Sight surface — including the chip dropdown — when
	     the user clicks ⓘ on a tradition row. Click outside the card
	     or click the X to close; the chip's onDropdownClose cascade
	     also closes the modal when the dropdown closes for any other
	     reason (Esc, click-outside, tradition switch). -->
	{#if manifestModalOpen}
		<div
			class="sight-v6-manifest-overlay"
			role="dialog"
			aria-modal="true"
			aria-label="Tradition manifest"
			onclick={closeManifestModal}
			onkeydown={(e) => { if (e.key === 'Escape') closeManifestModal(); }}
			tabindex="-1"
		>
			<div
				class="sight-v6-manifest-card"
				onclick={(ev) => ev.stopPropagation()}
				onkeydown={(ev) => ev.stopPropagation()}
				role="document"
				tabindex="-1"
			>
				<button
					class="sight-v6-manifest-close"
					type="button"
					onclick={closeManifestModal}
					title="Close manifest (Esc also works)"
					aria-label="Close manifest"
				>×</button>
				{#if manifestContent}
					<article class="sight-v6-manifest-body">
						{@html renderManifestMarkdown(manifestContent)}
					</article>
				{:else}
					<div class="sight-v6-manifest-loading">Loading manifest…</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.sight-v6-root {
		position: relative;
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		/* MIG-027 — theme-aware. Background + text follow active
		   interface theme via CSS vars set on document.body by
		   +layout.svelte's theme $effect. */
		background: var(--background-primary, #080c16);
		color: var(--text-normal, #e8ebf2);
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);

		/* MIG-027 §-fix-2 — Sight highlight color (the "this is your
		   current focus / active filter / hover-linked context" indicator).
		   Originally #fbbf24 (bright amber) hardcoded — works on dark
		   themes, washes out on cream/white themes. Theme-conditional vars
		   below override for light themes via :global(body.theme-light)
		   so the same semantic meaning carries cleanly across both. The
		   :global() prefix is needed because body.theme-light lives outside
		   the component scope (set by +layout.svelte's theme $effect on
		   document.body). */
		--sight-highlight: #fbbf24;
		--sight-highlight-bg-soft: rgba(251, 191, 36, 0.10);
		--sight-highlight-bg-strong: rgba(251, 191, 36, 0.18);
		--sight-highlight-border-soft: rgba(251, 191, 36, 0.45);
	}
	:global(body.theme-light) .sight-v6-root {
		/* Light-theme highlight: amber-700 (#b45309). Deeper, more
		   saturated; passes WCAG AA contrast on cream / off-white
		   backgrounds. Bg / border alphas slightly nudged up for the
		   darker hue. */
		--sight-highlight: #b45309;
		--sight-highlight-bg-soft: rgba(180, 83, 9, 0.10);
		--sight-highlight-bg-strong: rgba(180, 83, 9, 0.16);
		--sight-highlight-border-soft: rgba(180, 83, 9, 0.55);
	}

	.sight-v6-header {
		flex: 0 0 auto;
		display: flex;
		align-items: baseline;
		gap: 12px;
		padding: 14px 24px;
		border-bottom: 1px solid var(--background-modifier-border, #1a1f2e);
	}

	.sight-v6-title {
		font-size: 18px;
		font-weight: 500;
		letter-spacing: 0.5px;
	}

	.sight-v6-subtitle {
		font-size: 11px;
		color: var(--text-muted, #5a6275);
	}

	/* §B.10 — Extended-view indicator (user-facing label "EXTENDED";
	   internal field also renamed to `extended` per §B.10-fix-1).
	   Tiny gold-on-dark tag immediately right of the subtitle. Not a
	   button — Cmd-Shift-D is the toggle. CSS class name `.sight-v6-
	   pro-badge` kept (DOM-only identifier, no architectural meaning). */
	.sight-v6-pro-badge {
		display: inline-flex;
		align-items: center;
		padding: 2px 7px;
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.6px;
		/* MIG-027 §-fix-2: theme-aware semantic highlight */
		color: var(--sight-highlight);
		background: var(--sight-highlight-bg-soft);
		border: 1px solid var(--sight-highlight-border-soft);
		border-radius: 3px;
		cursor: help;
		font-variant: small-caps;
	}

	/* §B.7-fix-1 — filter affected-count badge. Shows X/Y notes when
	   any facet filter is active. Subtle but readable. Sits before the
	   Reset View button via margin-left:auto on the count (which pushes
	   it right-aligned with the button to its right when both visible).
	   MIG-027 §-fix-2: text + border use --sight-highlight (theme-aware
	   semantic gold — bright amber on dark, deep amber on light). The
	   filter-active = gold semantic is preserved; only the luminosity
	   adapts so it reads on both themes. */
	.sight-v6-filter-count {
		margin-left: auto;
		padding: 4px 10px;
		font-size: 11px;
		color: var(--sight-highlight);
		font-variant-numeric: tabular-nums;
		background: var(--background-secondary, rgba(58, 67, 90, 0.35));
		border: 1px solid var(--sight-highlight-border-soft);
		border-radius: 4px;
	}

	/* §B.6-fix-4c — Reset View button. Lives at the right edge of the
	   header strip via margin-left:auto. Subtle by default; visible
	   on hover. Designed not to compete with the title.
	   §B.7-fix-1: when filter-count badge is also visible, the badge
	   takes the margin-left:auto and the button sits next to it via
	   a smaller fixed margin. */
	.sight-v6-reset-btn {
		margin-left: auto;
		padding: 4px 12px;
		font-size: 11px;
		font-family: inherit;
		color: var(--text-normal, #e8ebf2);
		background: var(--background-secondary, rgba(58, 67, 90, 0.55));
		border: 1px solid var(--text-accent, #3b5998);
		border-radius: 4px;
		cursor: pointer;
		transition: background 0.12s ease;
	}
	.sight-v6-filter-count + .sight-v6-reset-btn {
		margin-left: 8px;
	}
	.sight-v6-reset-btn:hover {
		background: var(--background-modifier-hover, rgba(74, 90, 130, 0.85));
	}
	.sight-v6-reset-btn:active {
		background: var(--interactive-hover, rgba(58, 67, 90, 0.85));
	}

	.sight-v6-body {
		flex: 1 1 auto;
		display: flex;
		flex-direction: row;
		min-height: 0;
		overflow: hidden;
	}

	.sight-v6-canvas-host {
		flex: 1 1 auto;
		position: relative;
		overflow: hidden;
		min-width: 0;
	}
	/* §B.1: when mini-domes visible, anchor compresses to ~60%
	   of remaining horizontal space; minis grid takes ~40%. */
	.sight-v6-canvas-host.has-minis {
		flex: 0 1 60%;
	}

	.sight-v6-minis-grid {
		flex: 0 1 40%;
		display: grid;
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
		gap: 8px;
		padding: 8px;
		min-width: 0;
		min-height: 0;
	}
	.sight-v6-mini-cell {
		position: relative;
		min-width: 0;
		min-height: 0;
		border: 1px solid var(--background-modifier-border, #1a1f2e);
		border-radius: 4px;
		overflow: hidden;
	}

	.sight-v6-canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		display: block;
		cursor: default;
	}

	/* §B.6-fix-3 — promoted mini wrapper. Fills the canvas-host the
	   same way the anchor canvas does (absolute inset 0). The MiniDome
	   inside has its own host div + ResizeObserver, so size changes
	   propagate naturally. */
	.sight-v6-promoted-host {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}

	.sight-v6-canvas.has-hover {
		cursor: pointer;
	}
	.sight-v6-canvas.is-dragging {
		cursor: grabbing;
	}
	.sight-v6-canvas:focus {
		outline: none;
	}

	.sight-v6-zoom-badge {
		position: absolute;
		right: 16px;
		top: 16px;
		font-size: 11px;
		color: var(--text-accent, #7dd3fc);
		padding: 4px 10px;
		background: var(--background-secondary, rgba(13, 19, 34, 0.92));
		border: 1px solid var(--text-accent, #3b5998);
		border-radius: 4px;
		pointer-events: none;
		font-variant-numeric: tabular-nums;
	}

	/* §C.4 — extension chips overlay for masādir tradition. A small
	   horizontal row of italic-styled chip badges at the bottom-center
	   of the canvas-host, ~16 px above the bottom edge. pointer-events
	   none on the wrapper so the chips don't interfere with anchor dome
	   gestures; individual chips re-enable pointer-events for the hover
	   tooltip (title attribute). Visual style is intentionally close to
	   the stratum labels — italic, muted color, sits as soft chrome
	   rather than as an active control. */
	.sight-v6-extension-chips {
		position: absolute;
		left: 50%;
		bottom: 16px;
		transform: translateX(-50%);
		display: flex;
		gap: 8px;
		pointer-events: none;
		z-index: 2;
	}
	.sight-v6-extension-chips .extension-chip {
		pointer-events: auto;
		display: inline-flex;
		align-items: center;
		padding: 3px 9px;
		font-family: inherit;
		font-style: italic;
		font-size: 10px;
		color: var(--text-muted, #a0a8ba);
		background: var(--background-secondary, rgba(58, 67, 90, 0.35));
		border: 1px solid var(--background-modifier-border, rgba(160, 168, 186, 0.30));
		border-radius: 3px;
		cursor: help;
		white-space: nowrap;
	}

	/* MIG-027 §-fix-1: bg + border + text bumped to theme vars. Originally
	   hardcoded to dark-theme values; leaked through during light-theme
	   Boss test. */
	.sight-v6-loading {
		position: absolute;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		font-size: 12px;
		color: var(--text-muted, #7a8295);
		padding: 8px 16px;
		background: var(--background-secondary, rgba(13, 19, 34, 0.85));
		border: 1px solid var(--background-modifier-border, #2a3245);
		border-radius: 6px;
		pointer-events: none;
	}

	.sight-v6-loading-bg {
		left: auto;
		right: 16px;
		top: auto;
		bottom: 16px;
		transform: none;
		font-size: 10px;
		color: var(--text-faint, #5a6275);
		opacity: 0.7;
	}

	/* MIG-027 §-fix-1: hover-info was the most visible leak in Boss
	   test — dark navy box on cream bg. All four properties now
	   theme-aware. */
	.sight-v6-hover-info {
		position: absolute;
		left: 16px;
		bottom: 16px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 6px 12px;
		background: var(--background-secondary, rgba(13, 19, 34, 0.94));
		border: 1px solid var(--background-modifier-border, #2a3245);
		border-radius: 4px;
		pointer-events: none;
		max-width: 60%;
	}
	.sight-v6-hover-title {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-normal, #e8ebf2);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.sight-v6-hover-path {
		font-size: 10px;
		color: var(--text-faint, #5a6275);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* MIG-026 Phase ι.2 — manifest disclosure modal styles. Theme-aware
	   via MIG-027 CSS vars; chrome inherits the active interface theme
	   automatically (Constellation Light / Dark / Nord / Solarized). */
	.sight-v6-manifest-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 32px;
		background: rgba(8, 12, 22, 0.65);
		backdrop-filter: blur(2px);
		z-index: 100;
	}
	:global(body.theme-light) .sight-v6-manifest-overlay {
		background: rgba(220, 220, 220, 0.55);
	}

	.sight-v6-manifest-card {
		position: relative;
		max-width: 720px;
		width: 100%;
		max-height: 100%;
		overflow-y: auto;
		padding: 32px 36px 28px;
		background: var(--background-primary, #0c1322);
		border: 1px solid var(--background-modifier-border, #2a3245);
		border-radius: 8px;
		box-shadow: var(--shadow-l, 0 12px 36px rgba(0, 0, 0, 0.5));
		color: var(--text-normal, #cdd5e0);
		font-family: var(--interface-font, 'Inter', system-ui, sans-serif);
		line-height: 1.55;
	}

	.sight-v6-manifest-close {
		position: absolute;
		top: 10px;
		right: 12px;
		width: 30px;
		height: 30px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		line-height: 1;
		color: var(--text-faint, #7a8295);
		background: transparent;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		transition: color 0.12s ease, background 0.12s ease;
	}
	.sight-v6-manifest-close:hover {
		color: var(--text-normal, #e8ebf2);
		background: var(--background-modifier-hover, rgba(255, 255, 255, 0.08));
	}

	.sight-v6-manifest-loading {
		padding: 40px 0;
		text-align: center;
		color: var(--text-muted, #7b8499);
		font-size: 13px;
	}

	.sight-v6-manifest-body :global(h1) {
		margin: 0 0 8px;
		font-size: 22px;
		font-weight: 600;
		color: var(--text-normal, #e8ebf2);
		letter-spacing: 0.2px;
	}
	.sight-v6-manifest-body :global(h2) {
		margin: 22px 0 6px;
		font-size: 14px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 1px;
		color: var(--text-muted, #9aa3b8);
	}
	.sight-v6-manifest-body :global(h3) {
		margin: 14px 0 4px;
		font-size: 13px;
		font-weight: 600;
		color: var(--text-normal, #e8ebf2);
	}
	.sight-v6-manifest-body :global(p) {
		margin: 0 0 10px;
		font-size: 13px;
		color: var(--text-normal, #c8cdd9);
	}
	.sight-v6-manifest-body :global(strong) {
		color: var(--text-normal, #e8ebf2);
		font-weight: 600;
	}
	.sight-v6-manifest-body :global(em) {
		font-style: italic;
		color: var(--text-muted, #9aa3b8);
	}
	.sight-v6-manifest-body :global(ul),
	.sight-v6-manifest-body :global(ol) {
		margin: 0 0 12px 22px;
		padding: 0;
	}
	.sight-v6-manifest-body :global(li) {
		font-size: 13px;
		margin: 0 0 4px;
		color: var(--text-normal, #c8cdd9);
	}
	.sight-v6-manifest-body :global(code) {
		padding: 1px 5px;
		font-family: var(--mono-font, 'Fira Code', monospace);
		font-size: 12px;
		background: var(--background-secondary, rgba(40, 50, 70, 0.5));
		border-radius: 3px;
		color: var(--text-accent, #7dd3fc);
	}
	.sight-v6-manifest-body :global(blockquote) {
		margin: 8px 0;
		padding: 4px 12px;
		border-left: 3px solid var(--background-modifier-border, #3b5998);
		color: var(--text-muted, #9aa3b8);
		font-style: italic;
	}
	.sight-v6-manifest-body :global(hr) {
		margin: 18px 0;
		border: none;
		border-top: 1px solid var(--background-modifier-border, #2a3245);
	}
</style>

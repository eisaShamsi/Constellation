# MIG-072 — Full Style Setter Coverage of the Sky View — PLAN (Phase 2)

**Status:** Plan complete, awaiting owner approval before Build. Architect: `MIG-072-skyview-style-coverage-ARCHITECT.md`.
**Mechanism chosen by owner:** **Option B — "one palette, both graphs"** ("follow the SV colouring mechanism" — the
engine is *told* its colours via `setData` today; we extend that with `setPalette`).

---

## Mechanism

A **pure resolver** `resolveSkyPalette(overrideVars, draftVars, linkTypes, isDark): SkyPalette` lives in a new
`src/lib/graph/skyPalette.ts`. It resolves EVERY Sky View colour (+ the in-scope "intensity" alphas) from:
- **(a)** `var(--skyview-X, <today's exact value>)` — a per-Universe CSS var, defaulting to the current
  hardcoded value, so **unset = pixel-identical to today**; and
- **(b)** `linkTypeColor()` (the user-editable registry) for the **8 typed links** — no duplicate.

Each consumer (`GraphMindView` for the full graph; the LocalSkyView mounts for the second screen) wraps the
resolver in a Svelte `$derived` keyed on `$styleOverride` / `$liveStyleDraft` / `$linkTypesStore` / theme mode,
and pushes the result into its renderer via a new **`setPalette()`** (mirrors `setData()`). **No
`getComputedStyle` / IPC in any `draw()`** — the palette is resolved in the Svelte layer and handed in;
recompute happens only when those stores change. `SkyPalette` + `DEFAULT_PALETTE` (the exact current constants)
are defined in `graphEngine.ts` (plain TS, no Svelte imports); the literal `var(--skyview-X, …)` strings live
once in `skyPalette.ts` — that file is the **wiring-audit consumer site**.

`--skyview-bg` (shipped in the prior commit) stays a **pure CSS background** behind the transparent canvas — the
one Sky View var that does NOT flow through the JS palette.

## Invariants carried from Phase 1 (must not break)
1. Perf Rule 3 — zero `getComputedStyle`/IPC in any `draw()` (7,600+ nodes).
2. BUG-015 — the engines never write `body.style`; new vars ride the single apply `$effect` (`+layout.svelte:1558`).
3. Typed-link single-source — read `linkTypeColor()`; delete `TYPED_LINK_COLORS`; **no duplicate control**.
4. `--skyview-bg` keeps working unchanged.
5. Dark/light still resolves (the resolver picks the mode-correct default when a var is unset).
6. Second-screen parity — LocalSkyView gets the SAME palette; kill its divergent `LIBRARY_COLORS`.
7. Wiring audit 143+N / 0 dead — each new `--skyview-X` appears literally as `var(--skyview-X` in `skyPalette.ts`.
8. Unset var = today's exact look (no regression). No disk/schema migration (`styleOverride` is free-form JSON).

---

## Phased steps (each = one commit, `§N`)

### §1 — Palette seam + typed-link single-source *(internal; the provable win)*
- New `src/lib/graph/skyPalette.ts` (resolver + `PALETTE_VARS` literal-var table); add `SkyPalette` / `DEFAULT_PALETTE`
  to `graphEngine.ts`.
- **Delete `TYPED_LINK_COLORS`** (graphEngine ~88-97); add a `palette` field + `setPalette(p)` (after ~469); route the
  typed-link read (~1943) and ALL inline draw constants (~1852-53, 1887, 1946, 1971, 2055/2075, 2145, 2151, 2163,
  2174/2176, 2181 + `MATURITY_COLORS`, 2190 + `MOC_RING_COLOR`, 2196, `BADGE_COLORS` ~577, gizmo ~303/2245-47/2295,
  labels ~2336/605) through `this.palette.*`. Hex→int conversion at the `setPalette` boundary.
- `GraphMindView.svelte`: add `skyPalette` `$derived` (resolver + `typedLinks` from the registry) → `$effect`
  `engine.setPalette(...)` (+ once after `init()` ~756); add a theme-mode signal so a pure dark/light flip recomputes.
- `LocalSkyView.svelte`: accept a `palette` prop (consumed in §5).
- **Verify (owner-testable PAUSE):** Sky View pixel-identical to today; recolouring "supports" in Setter → Links now
  recolours the graph edge (headline bug fixed); dark/light still correct. *(Inv 1,3,6,8)*

### §2 — Setter controls: nodes, rings & glows *(owner-testable)*
- `StyleSetter.svelte` `ELEMENTS`: `skyNodes` (`--skyview-node-default`, `--skyview-ring-active/-selection/-pinned/-orphan`),
  `skyMaturity` (5 vars), `skyGlow` (`--skyview-glow-received/-discovered`, `--skyview-moc-ring`). Add keys to the `sky`
  category. Add matching `var(--skyview-X, default)` rows to `skyPalette.ts`.
- **Verify (PAUSE):** changing Active/Pinned ring, maturity, origin glow restyles live; Discard reverts; fresh
  Universe pixel-identical. *(Inv 2,5,7,8,9)*
- **As-built deviations (2026-06-08):**
  - **Dropped `ringSelection`** as a control — the selection ring is dynamically the *selected library's* colour
    (`+layout.svelte:5809`), meaningful association, not a static style (Form-Aligns-To-Purpose).
  - **Dropped `Seed` maturity** control — seed notes draw no ring by design; `--skyview-maturity-seed` stays in
    the palette only as the unknown-state fallback.
  - **Removed `accent`/`link`** from the `sky` category — they never reached the graph (chrome vars).
  - **Live centre preview** added (`ss-skyprev`): the Sky surface now uses three-zone (`twoZone` excludes `sky`)
    so the centre stage shows a labelled bubble preview (Nodes / Maturity rings / Glows & MOC) that recolours
    live via the `.ss` `draftStyle` cascade — a focused preview beats hunting a ring-change in a 7,600-node graph.
  - **Node frame stroke — PER-RING (Eisa 2026-06-08, 2nd pass)** — each ring has its own width + solid/dotted,
    not one global setting. Palette holds `ringFrames: Record<id, {width, style}>` (ids: active · pinned · orphan
    · sapling · evergreen · canonical · wilting · moc) resolved from `--skyview-frame-<id>-width/-style`
    (literal names in `FRAME_VARS`, the audit's consumers). `strokeRing(…, frameId)` looks up the per-ring frame
    (width multiplier + dashed-circle arc segments; PIXI has no native line-dash). The Nodes/Maturity/Glows
    groups each carry per-ring width+style controls; preview rings use the `border` shorthand per-ring so width +
    dotted + colour all show live. (The earlier global `--skyview-ring-width/-style` were removed.)
  - **Node-scheme legend (Eisa)** — a collapsible "Node scheme" key in the Sky View legend (`GraphMindView`)
    showing each ring/glow in the user's chosen colours (`skyHex(skyPalette.*)`). i18n keys use `$t(...) || 'EN'`
    fallback; translations batched.
  - **Clean dotted rendering (Eisa)** — dotted rings draw as round dots built into ONE `fill()` per ring
    (was spiky per-segment arc strokes); cheaper + cleaner.
  - **Stacked rings — MOCK-UP APPROVED (Eisa 2026-06-08)** — a node carries 2-3 rings at once
    (maturity + MOC/orphan + interaction) drawn 0.5px apart → they merged. Root cause Eisa identified:
    superimposition. Fix: glows draw FIRST (diffuse halos, behind), then the applicable rings are COLLECTED
    inner→outer (maturity → MOC → orphan → pinned → open-note → selection) and drawn in EVENLY-SPACED
    concentric bands. **Approved constants (from the interactive mock-up `docs/MIG-072-stacked-rings-mockup.html`,
    "as-is"): first-ring gap 1.5, gap between rings 2.6, ring-width multiplier default 1.5.** orphan/MOC are
    mutually exclusive (0 links vs ≥5 outgoing) so the realistic stack is 1-3 rings. This changes the default
    node look (intended improvement, owner-approved) — the "unset = today's look" invariant still holds for
    COLOURS; ring spacing/width is the approved §2 visual design. Mock-up was designed → approved before any code.

### §3 — Setter controls: links & overlays *(owner-testable)*
- `ELEMENTS`: `skyLinks` (`--skyview-edge-normal`, `--skyview-edge-highlight`, `--skyview-arrow-out/-in`,
  `--skyview-semantic`, `--skyview-cluster`; + in-scope alphas), `skyOverlays` (`--skyview-trail`, 7 `--skyview-badge-*`).
  Add to `sky` category + `PALETTE_VARS`.
- **Verify (PAUSE):** hover → highlight edges / mid-arrows / semantic links honour new colours; search badges + trail
  recolour.

### §4 — Setter controls: labels (full font), gizmo & remaining *(owner-testable)*
- `ELEMENTS`: `skyLabels` — **full font control on the Sky View text (Eisa, 2026-06-08):** colour
  (`--skyview-label`), **font family** (`--skyview-label-font`, a `select` over the shared `FONTS`),
  **size** (`--skyview-label-size`), and **thickness/weight** (`--skyview-label-weight`). Also `skyGizmo`
  (`--skyview-gizmo-x/y/z/-centre`).
- Engine: the node-label `TextStyle` (graphEngine ~2333) currently takes only `fontSize` (from
  `EngineConfig.labelFontSize`) + the script-aware `getFontForText` family + default weight. Extend it to
  read `palette.labelFamily/labelSize/labelWeight/label` (family **overrides** the script default when set;
  unset = `getFontForText` as today). Recreate/restyle pooled labels on palette change. Apply the same
  font to edge-type labels (~2004) + the search-badge name label (~605) for consistency.
- **Predecessor reconciliation (label size):** `EngineConfig.labelFontSize` already drives label size from
  the Sky View **⚙ Graph Appearance** panel (a persisted `skyViewSettings` value). The Style Setter becomes
  the styling home; resolve drift by having `--skyview-label-size`, **when set**, win over
  `config.labelFontSize` (the ⚙ slider stays as a quick in-graph control and writes the same effective
  size; unset Setter var = the ⚙ value = today). No control is removed silently; logged per the Predecessor
  Lookup Rule. (If Eisa prefers, the ⚙ size row can instead be retired into the Setter — decide at §4.)
- `setPalette` must update the already-created gizmo `Text.style.fill`.
- **Verify (PAUSE):** Sky View text restyles live — pick a **font**, change **size**, change **thickness**,
  change **colour**; tilt to 3D and recolour the axes + centre dot. **Full vocabulary now covered.**

### §5 — LocalSkyView parity *(one source)*
- **Delete `LIBRARY_COLORS`** (LSV ~56); pass the resolved colour map + `palette` into LSV; swap `draw()` hardcodes
  (~118, 135, 144, 161-163) to `palette.*`. Keep `--skyview-bg` as the pure-CSS background (~296).
- Thread `palette` into the 4 mounts: `+layout.svelte:6505`, `SecondScreenPage.svelte:1320/1451/1604` (each wraps the
  shared resolver in its own `$derived`).
- **Verify (PAUSE):** second-screen / right-sidebar star colours match the full graph; a Setter change applies to BOTH
  renderers identically. *(Inv 6)*

### §6 — Audit, drift, migration-path & build *(Phase-4 hooks)*
- Wiring audit → `WIRED 143+N / DEAD 0`. Invariant greps: `getComputedStyle` in `src/lib/graph` still 0; sole
  `body.style` writer still `+layout.svelte:1558`; `TYPED_LINK_COLORS` gone. Drift grep: no surviving duplicate palette
  (`LIBRARY_COLORS`, engine `MATURITY_COLORS`/`BADGE_COLORS`, inline `0x…`).
- `npm run check` (ignore the 2 pre-existing errors); `npm run tauri build`.
- **Final owner tutorial:** new Universe pixel-identical; walk each Sky View element group, change a colour in each, Keep
  → persists per-Universe; another Universe keeps its own look; typed-link recolour follows; second screen matches;
  Discard reverts.

**Rollback:** each phase is one revertible commit; §1 is the keystone (revert restores constants via `DEFAULT_PALETTE`).
No disk/schema change; stale `--skyview-*` keys are inert (cleared by `_lastStyleSettingsKeys`, `+layout.svelte:1580`).

---

## SCOPE CHECKPOINT (owner decides before §2)

**1. Coverage scope:**
- **(A) colours + the 5 "intensity" alphas** (`edgeNormalAlpha` 0.25/0.15, `semanticAlphaMul` 0.6, `glowOriginAlpha`
  0.06, `stratumGlowAlphaUnit` 0.08, `dimAlpha` 0.12) — **recommended**;
- (B) colours only (alphas stay hardcoded);
- (C) colours + alphas + a few sizes/widths (creates a second source for `linkThickness` etc. — only if wanted).
- Out of scope either way: node-label font/size (already `EngineConfig.labelFontSize`) and the library legend colours
  (data-driven per legend mode).

**2. Dark/light var shape:**
- **One var per element** (a set value applies to both modes; unset → theme-correct default; matches `--skyview-bg`) —
  **recommended**; vs
- Two vars (`*-dark` / `*-light`) for independent per-mode control (doubles the control count).

## Critical files
- `src/lib/graph/graphEngine.ts` · `src/lib/graph/skyPalette.ts` (new) · `src/lib/components/GraphMindView.svelte` ·
  `src/lib/components/StyleSetter.svelte` · `src/lib/components/LocalSkyView.svelte` · mounts in `+layout.svelte` +
  `SecondScreenPage.svelte`.

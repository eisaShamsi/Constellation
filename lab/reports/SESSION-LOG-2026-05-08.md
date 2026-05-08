# Session log — 2026-05-08

**Cascade**: MIG-019 §2G.3 redesign cascade → §2G.3a → §2G.3b → §2G.3c → §2G.3d → §2G.3e → §2G.3f → §2G.3g → §2G.3h → §2G.3i. Eight test rounds with Eisa, eight commits + pushes, eight orientation v-bumps inline (v1.58 → v1.65). Lens-style zoom architecture finalized at the end of the day, awaiting §2G.3i Boss verdict.

The work this session is the second half of the §2G visual redesign that started yesterday evening (2026-05-07) — see `SESSION-LOG-2026-05-07.md` for §2G.1 → §2G.3 plus the (X, Y, Z) per-mode grammar approval.

---

## Commits today (chronological)

| Commit | Phase | Visible to user |
|--------|-------|-----------------|
| `7b1ecd3` | §2G.3e | de-spoke (hash jitter + 6-iter wedge repulsion) + smaller rim font 15→12 px + placeholder text "MDS embedding" → "fetching layout" |
| `6156f9a` | §2G.3f | numbered rim (1, 2, 3 …) colored per library + library legend panel (left for LTR, right for RTL) + library color palette (golden-angle hue stride) + node size cap (max r=4) + node stroke (0.6 px ink) + stronger repulsion (MIN_DIST 6→9, iter 6→12) + sqrt radial mapping (uniform area density) + silent loading |
| `4c8e2dc` | §2G.3g | Pixi `autoDensity: true` + `resolution: devicePixelRatio` (browser zoom no longer breaks layout) + `chartContainer` wrapping all chart layers + wheel zoom + drag pan + Reset View button + close button bumped to z:100 |
| `8596bed` | §2G.3h | `pickStar` inverse-transform (hover/click hit accurate when zoomed/panned) + selection ring r+1 stroke 1.5 (was r+3 stroke 2 — ring now matches node) + cream backdrops on Universe Health + Universe-name (no star bleed-through) + edges incident-only with cap 50 + 1-hop neighbour rings + side panel z-index 5→50 + Esc cascade (clear → reset → close) + close button z-index 100→1000 + wheel via `addEventListener({passive:false})` |
| `1592663` | §2G.3i | **Lens-style zoom architecture**: `.sight-v3-zoom-wrapper` wraps canvas + chart overlays; ONE CSS transform scales them together. Pixi `chartContainer.scale` REMOVED (CSS does it all). + close button via `bind:this` + raw `addEventListener('click', ...)` (bypasses Svelte `onclick` after four failed rounds) + Reset View always visible (.reset-active class for prominent state) + tooltip moved OUTSIDE the lens wrapper (CSS `position: fixed` was breaking inside transformed parent) + tooltip z-index 50→1500 + ::first-line bold for note title + edges darkened from light gold to dark burnt-amber for visibility on cream |

All commits pushed to `origin/main`. Origin tracks `1592663` at end of day.

---

## Eisa-flagged bugs and how each round addressed them

| Round | Eisa report | §2G.3x fix |
|-------|-------------|-----------|
| §2G.3 | "Why are nodes clustered toward the rim? Fix the bleeding fonts. Universe name above the dome." | §2G.3c (label truncation, Universe-name header), §2G.3e (rank-percentile radius), then sqrt mapping in §2G.3f |
| §2G.3e | "Better! I like how it renders. But why the nodes form spokes inside each wedge?" | The spoke fix actually shipped IN §2G.3e (hash-based azimuth jitter) — Eisa was looking at §2G.3d output |
| §2G.3f | "Replace rim labels with numbers. Library legend on the side. Nodes colored per library. Size cap. Stroke. Repulsion." | §2G.3f shipped all of these |
| §2G.3g (test) | "Mouse hovering displaced. Universe Health overlap dome. Hit-testing broken. Close button doesn't work. Reset button invisible. Edges overwhelming." | §2G.3h addressed all but the close button and lens-zoom mismatch |
| §2G.3h (test) | "Close button STILL doesn't work. Whole-page lens zoom needed (everything scales together, not just the chart). Note title still not visible." | §2G.3i — full lens architecture redo + close via raw addEventListener |
| §2G.3i | (pending Eisa test) | TBD |

---

## State of standing (SO #5 record)

### Verified shipped + protected
- §2G.1 visual spec doc (committed yesterday, `bfb8aba`)
- §2G.2 pure helpers polar.ts/modes.ts/regions.ts (`b1a2477`)
- §2G.3 → §2G.3c rolled commit (yesterday, `7d6fcf6`)
- §2G.3d (X, Y, Z) per-mode grammar (yesterday, `467b9f2`)
- §2G.3e → §2G.3i: today's five commits, all pushed

### At-risk / pending Boss test
- §2G.3i (the lens architecture) — built, installer ready at `~13:17`, awaiting test result. Two critical fixes: lens zoom (everything scales together) + close button via raw addEventListener.

### Known-broken (until §2G.3i Boss test passes)
- Close button (four rounds of failed Svelte `onclick` — now using raw DOM addEventListener as last-resort)
- Possibly: hit-testing under non-default zoom/pan (refactored in §2G.3i; algebra verified before commit)

### Pending but not started
- §2G.4: Mode toggle UI + 600 ms eased migration animation
- §2G.5: Mode persistence (`appSettings.sight.lastMode`)
- §2G.6: 3-agent audit + tag milestone + orientation v1.66 + i18n keys for mode names + close-out
- MIG-020 (layer peeling + v2 retire)

### Documentation drift (this PCS resolves)
- Help: master `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` was a v2-era doc with a "🚧 being rebuilt" banner. PCS updates it for v3 polar layout.
- User Manual §8 (Constellation Sight) was v2-era (gravity-well graph). PCS rewrites for v3.
- Translations (`docs/help.{lang}/Constellation Sight/`) deferred to §2G.6 (close-out batch).
- MoCh due (last `MoCh-2026-05-07-2200.md`; today is ~5 hours of direct chat).

### Orientation chain
- v1.58 → v1.65 (8 bumps today, all inline with the trigger commit per the inline-orientation rule).
- Each preserves prior versions in `docs/` per SO #6.

---

## What §2G.3 → §2G.3i taught us

1. **Architectural pivots come at higher altitudes than the implementation guesses.** Eisa's "lens" framing for zoom (everything scales together) was a single sentence that invalidated three rounds of "chart zooms while overlays stay anchored" implementation. The fix wasn't more CSS knobs — it was rethinking what the user expected. Always ask "what's your mental model?" before optimizing.

2. **Svelte 5 `onclick={fn}` can fail silently in some contexts and we don't know why.** Four rounds of escalating defensive fixes (lambda, stopPropagation, z-index 100 → 1000, type="button", explicit invocation) all kept hover-style firing while click events vanished. `bind:this` + raw `addEventListener('click', fn)` is the bulletproof escape hatch. When the framework binding fails inexplicably, drop to raw DOM.

3. **CSS `position: fixed` is broken inside `transform`-ed ancestors.** Spec quirk — fixed becomes relative to nearest transformed ancestor instead of the viewport. The tooltip stayed broken until I moved it OUTSIDE the lens wrapper.

4. **Per-step orientation v-bump caught its first violation today**. Earlier in the §2G.1 → §2G.3c rollout I batched the v1.58 → v1.59 update into a separate "SO catch-up" commit. Per `feedback_orientation_inline_with_commit.md` (top principal), that's a violation. After Eisa's "Have you SO's?" check-in last night, every commit since has carried its own orientation v-bump inline. v1.59 (catch-up) → v1.60 (§2G.3d inline) → v1.61 → v1.62 → v1.63 → v1.64 → v1.65 (§2G.3i inline) — clean discipline.

5. **The (X, Y, Z) grammar is the right architectural anchor.** Eisa's pivot from "switchable rim axis" to "per-mode (X, Y, Z) where each mode picks its own azimuth/radius/magnitude" elevated Sight from a chart with one toggle into a multi-instrument cognitive lens. The visual spec at v1.0 vs v1.1 captures this.

---

## §2G.3j → §2G.3n cascade (catch-up — five more rounds same day)

The §2G.3i Boss test surfaced regressions; what looked like a one-step verification fanned out into five more iterations. All shipped same-day (2026-05-08), all pushed to `origin/main`, each with its own inline orientation v-bump per the inline-orientation rule.

| Commit | Phase | Visible to user |
|--------|-------|-----------------|
| `3623887` | §2G.3j | Fix lens-architecture regressions: `.sight-v3-zoom-wrapper` got `display: flex; align-items: stretch` so canvas's `flex: 1` actually works (was sized to content default ~800×800) + close-button binding via `$effect` reactive on `closeBtn` $state ref (bind:this could resolve after onMount) + close-button z-index 1000 → 9999 |
| `9b1c5fb` | §2G.3k | Wheel-zoom freeze fix: removed redundant `$effect` watching `chartZoom` + `chartPanX` + `chartPanY` (was firing 200+ Svelte reactive updates per second on smooth-scroll wheels) + rAF-throttled wheel + drag handlers via single shared `zoomFrame` token (DOM writes coalesced to one per animation frame, math stays responsive) |
| `a78e0c3` | §2G.3l | **Evidence-backed redesign after a 5-agent audit** (3 codebase Explore + 2 web research). Findings cited from MDN, Steve Ruiz, pixi-viewport, W3C resize-observer, Svelte 5 docs. Reverted lens-CSS-on-canvas (CSS-scaling a `<canvas>` blurs the bitmap). Restored Pixi-native `chartContainer.scale.set(zoom)`. New `.sight-v3-overlays-wrapper` (HTML only, separate from canvas) scales via CSS for rim numbers + Universe Health + Universe-name + legend in lockstep. Canvas back to `position: absolute; inset: 0`. Hit-testing inverse-transform: `internal_x = (mouse_x - cx - panX) / chartZoom + cx`. Close button via direct `onclick={(e) => { e.stopPropagation(); onClose(); }}` (no more bind:this / addEventListener). |
| `b64d6b1` | §2G.3m | Defensive fix-up batch: close button gets BOTH `onclick` AND `onpointerup` (8th iteration; pointerup fires earlier in pointer-event sequence and is rarely suppressed) + Esc simplified to single-press always-close (cascade reset/clear/close was friction; Esc is the guaranteed escape hatch) + selection ring matches node size via shared `actualNodeRadius()` helper (was 2.35× too big for brightest stars) + edges → INK `#1a1a1a @ alpha 0.7` (dark burnt-amber still too low contrast on cream) + CSS overlays-wrapper switched to `translate3d/scale3d` (GPU-precise, eliminates sub-pixel divergence with Pixi rim arcs) |
| **(this commit)** | **§2G.3n** | **Adopt v2 working pattern wholesale** after Eisa: *"Check how we manage to do it right in the SV. It is already working there."* + *"Fix it, don't patch it."* — (1) Close button via thin `.sight-v3-header` flex strip with inline `onclick={() => onClose?.()}` button (matches `ConstellationSight2.svelte` line 1063 exactly) — no bind:this, no addEventListener, no $effect, no z-index 9999. (2) Library rim numbers moved from HTML overlay to **Pixi `Text`** inside `calendarRimContainer` — single transform pipeline, mathematically impossible to drift on zoom. CSS `.sight-v3-rim-number` rule deleted as dead code. (3) `sidePanelConnectedNotes` `$derived.by` walks 1-hop neighbours (cap 50) of selected note; new `connectedNotes` prop + `onConnectedClick` callback on `SightV3SidePanel`; side panel renders clickable list (color dot + title + library name) under "Connected notes (N)" header — clicking a row recentres the panel on that neighbour. |

### Eisa-flagged round-by-round (continuation)

| Round | Eisa report | Fix |
|-------|-------------|-----|
| §2G.3i | "What happened to my Sight? Even mouse wheel doesn't work. Nothing is working. Enough patching." | §2G.3j (canvas-sizing fix + reactive close-button binding) |
| §2G.3j | "When I try to zoom in using the mouse wheel, the app freezes." | §2G.3k (rAF throttle + remove redundant $effect) |
| §2G.3k | "Not working. Enough wasting my time. Bring in the audit agents and conduct the necessary research." | §2G.3l (5-agent audit → evidence-backed redesign with citations) |
| §2G.3l | "Library numbers offset on zoom. Selection ring doesn't match node size. Edges low contrast. Close button NOT WORKING." | §2G.3m (translate3d + actualNodeRadius helper + ink edges + onpointerup) |
| §2G.3m | "Close button still not working. Library handles still not fixed to rim. Need connected-note titles. Fix it, don't patch it." | §2G.3n (v2-pattern adoption: header bar + Pixi rim text + connected-notes list) |

### Orientation chain (continuation)

- v1.65 → v1.66 (§2G.3j inline) → v1.67 (§2G.3k) → v1.68 (§2G.3l) → v1.69 (§2G.3m) → **v1.70 (§2G.3n inline)**
- All preserved in `docs/` per SO #6. Five more bumps today, all inline with the trigger commit per the inline-orientation rule.

### What §2G.3l audit + §2G.3n v2-pattern adoption taught us (additions)

6. **When a bug resists 8 iterations of patches, look for a working version elsewhere in the codebase.** Eisa's "check how we manage to do it right in the SV" was the breakthrough. v2's `ConstellationSight2.svelte` had the exact same close-button widget shipping for months in production — a thin flex header with `onclick={() => onClose?.()}`. The v3 attempts kept inventing new defenses (z-index 9999, addEventListener, $effect, defensive pointerup) when the right answer was "match the working pattern verbatim". When stuck on a feature that's working elsewhere, **adopt, don't reinvent**.

7. **Two transform pipelines = drift.** Rim numbers as HTML overlay scaled via CSS `transform` while Pixi rim arcs scaled via `chartContainer.scale.set()` produced a tiny but visible offset that compounded with zoom. `translate3d` reduced it (GPU matrix precision) but didn't eliminate it. Fix: **single source of geometric truth**. Move the labels INTO the same Pixi container as the geometry they're labeling. Mathematically impossible to drift. Eisa's "fix it, don't patch it" was the right framing.

8. **Connected-notes visibility is a knowledge-formulation requirement, not a UI nicety.** A graph-view side panel that shows counts but not which notes feels like file-management; a panel that shows the names of the linked notes — clickable for further exploration — is the cognitive vocabulary of the Living Link Architecture. The 1-hop neighbour list is the smallest unit that makes Sight a thinking instrument rather than a counter. Cap 50 prevents hub-overflow but matches the focus-overlay edge cap so the visual and the panel agree on what counts as "connected".

---

## §2G.3o — Structural fix (close button, ninth-iteration root cause)

Pre-test, after Eisa pushed back: *"I am really wondering why you are not able to fix a simple task, like a 'Close' function!! Is it that hard? How many attempts so far?"* — and then: *"Go and do your homework. Dig for the simple, proven right solution."*

The §2G.3n commit had only copied v2's MARKUP (header div + close button), keeping v3's CSS where the header was `position: absolute` over the canvas. That's not "adopting v2's pattern" — that's a partial copy with the broken layout still in place. Eight iterations of pointer-events thread-the-needle and z-index escalation papered over the structural mistake without ever removing it.

### Homework — what the audit found

Spawned a parallel research agent. Found documented root cause:

- **Svelte 5 issue #15343 + #13213**: Svelte 5 delegates `onclick={fn}` handlers to `<body>` and relies on the click event bubbling all the way up. Any ancestor / sibling-with-handlers in the bubble path that calls `stopPropagation()` (or whose Svelte-delegated handler interferes with the bubble path) silently swallows the click. **Hover still works because hover is NOT delegated.** This exactly matches v3's symptom.
- **Pixi v8 EventSystem**: registers a `document.addEventListener('pointermove', …, true)` capture-phase listener for hit-testing. Combined with the canvas wrapper having `onclick`+`onpointerdown`+`onpointermove`+`onpointerup`+`ondblclick` (all body-delegated), this compounds the delegation problem.
- **Working examples in the same codebase** (`ConstellationSight2.svelte:1041-1064, 1269-1304` and `SkyView.svelte:961-965, 1162-1200`) BOTH use a flex-column root with the header as a real layout participant (no `position: absolute` on the header). That structural separation removes the bubble-path interference.

### What §2G.3o ships

| Commit | Phase | Visible to user |
|--------|-------|-----------------|
| **(this commit)** | **§2G.3o** | **Structural fix.** `.sight-v3-root` → `display: flex; flex-direction: column`. `.sight-v3-header` → real flex Row 1 (44 px, `flex-shrink: 0`, NO position:absolute, NO z-index, NO pointer-events). New `.sight-v3-body` Row 2 (`flex: 1; position: relative`) wraps canvas + Reset View + overlays + tooltip. Close button is now a plain inline flex item with plain `onclick={() => onClose?.()}` — no defenses, no patches. The bubble path from close button no longer crosses the canvas branch, so Svelte's body delegate routes the click correctly. Type-checked clean. |

### Why this works where eight prior iterations failed

| Iteration | Approach | Why it failed |
|-----------|----------|---------------|
| §2G.3g | Bumped close-button z-index to 100 | Z-index doesn't fix delegation; click never reached body |
| §2G.3h | Bumped to 1000 + Esc cascade | Same reason |
| §2G.3i | `bind:this` + `addEventListener('click')` | Bypassed delegation BUT `closeBtn` $state ref had timing issues; some attaches no-op'd |
| §2G.3j | Reactive `$effect` on `closeBtn` ref + z-index 9999 | $effect re-ran but stale closures dropped listeners on cleanup |
| §2G.3l | Pixi-native zoom + plain `onclick` | Restored the trigger condition (overlay sibling of canvas) |
| §2G.3m | + defensive `onpointerup` | pointerup also bubbles through canvas-event-handler delegates |
| §2G.3n | Copied v2's MARKUP but kept v3's broken CSS | Header was still `position: absolute` over canvas — the trigger condition stayed |
| **§2G.3o** | **Match v2's STRUCTURE: flex-column root, header as real Row 1, canvas in Row 2** | **Removes the trigger condition entirely** |

### Lesson learned (one-line, for LL-NNN bookkeeping)

When a Svelte 5 button click doesn't fire but hover does, the click event isn't reaching `<body>` for delegation. Don't add defenses (z-index, addEventListener, capture phase, pointerup) — find what's intercepting the bubble path and remove it structurally. The interceptor is usually a sibling element with its own delegated handlers + a layered absolute layout.

### Orientation chain

- v1.70 → **v1.71** (§2G.3o inline). Preserved in `docs/`.

---

## §2G.3p–§2G.3q — External close button + Pixi.js v8 → D3 + Canvas 2D migration

§2G.3o's structural flex-column fix (and the external close button in +layout.svelte) still didn't solve the close button because the ROOT CAUSE was deeper than Svelte 5 event delegation: **Pixi.js v8's EventSystem registers `document.addEventListener('pointermove/pointerup', ..., true)` capture-phase listeners that globally intercept pointer events.** These steal click events BEFORE any DOM element receives them — even buttons completely outside the Pixi component, in the parent layout, with higher z-index. CSS `:hover` still works (browser rendering engine, not JS events), which is why hover gold-tint always worked but onclick never fired across 11 iterations.

Eisa: *"Enough patching, find a root solution, even if it means restructuring the Sight all over again."* Then: *"Is Pixi the right tool? What about Canvas, since we are building a 2D tool?"*

### What §2G.3q ships

**Complete Pixi.js v8 → D3 + Canvas 2D migration** of `src/lib/sight/v3/SightV3.svelte`:

- **Renderer**: `new Application()` + Pixi Containers/Graphics/Text → raw `<canvas>` + `getContext('2d')` + immediate-mode draw pipeline
- **Zoom/Pan**: Manual `chartZoom`/`chartPanX`/`chartPanY` + `handleWheel` + `handlePointerDown`/`handlePointerUp` → `d3.zoom()` (wheel, drag, touch pinch, passive:false, rAF-throttled)
- **Stars**: `new Graphics().circle().fill().stroke()` → `ctx.beginPath(); ctx.arc(); ctx.fill(); ctx.stroke()`
- **Focus overlay**: Pixi Graphics lines + circles → Canvas 2D moveTo/lineTo/arc
- **Region rim**: Pixi Graphics + Pixi Text → Canvas 2D arc/stroke/strokeText/fillText
- **Milky Way**: Pixi Sprite+Texture → offscreen canvas + ctx.drawImage
- **Hit-testing**: Manual inverse transform → `currentTransform.invert([px, py])` (d3-zoom)
- **Placeholder**: Pixi Text on stage → simple string rendered in draw() via ctx.fillText
- **DPR handling**: Canvas attribute size = CSS size × devicePixelRatio; ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
- **Close button**: Works because Canvas 2D has ZERO global event side effects (no document-level listeners)

D3 v7 was already a dependency (used by SkyView). pixi.js stays in package.json for Constellation Map (graphEngine.ts).

### Build verification

- `svelte-check`: 0 new errors (1 pre-existing `fresh` LinkLifecycle — deferred MIG-014 follow-up)
- `npm run build`: ✓ built in 1m 54s, output clean
- ModeContext local variable renamed `ctx` → `mctx` to avoid shadowing the Canvas 2D context

### What this eliminates permanently

1. Pixi.js v8 EventSystem global capture-phase listeners (the 11-iteration close-button killer)
2. GPU buffer allocation per-star (the 3-minute render time on Boss-scale universes)
3. Pixi Container hierarchy management (6 containers + stage → single draw() pipeline)
4. Two-transform-pipeline drift (Pixi scale vs CSS transform — now one d3-zoom transform)

### Orientation chain

- v1.71 → **v1.72** (§2G.3q inline). Preserved in `docs/`.

---

## Session 2 — Sight v3 → v4 clean-slate pivot

### Background

Eisa tested the v3 §2G.3r build (raw DOM close button, 13th iteration). Close button still didn't work. Directive: **abandon v3, start from scratch as v4.**

Investigation revealed the root cause that 13 iterations couldn't fix: SightV3 mounted as `position: fixed; inset: 0; z-index: 1000` — a full-screen overlay OUTSIDE normal DOM flow. D3-zoom's event listeners on the viewport-filling canvas consumed all pointer events. Every z-index escalation, raw addEventListener, Svelte bypass, external button — all failed because the architecture was fundamentally wrong.

**SkyView's close button works because it renders inside `.content-area` as a normal flex child.** Header with close button in first row. Graph view in second row. No position:fixed. No z-index wars.

### What v4 ships

| File | Change |
|------|--------|
| `src/lib/sight/v4/SightV4.svelte` | **NEW** — clean-slate component. NO position:fixed root. Plain flex child. Same Canvas 2D + D3-zoom draw pipeline. Reuses all v3 helper modules (modes.ts, polar.ts, regions.ts, library-colors.ts). |
| `src/lib/sight/v4/SightV4SidePanel.svelte` | **NEW** — copy of v3 side panel (unchanged logic). |
| `src/lib/sight/engine.ts` | SIGHT_V3_ENABLED = false, SIGHT_V4_ENABLED = true |
| `src/routes/+layout.svelte` | v4 mounted INSIDE `.content-area` using `.star-fullscreen` pattern (same as SkyView). Close button: `<button class="star-close" onclick={() => sightV4Active = false}>×</button>` |
| `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` | §8, §9, §11 updated (Pixi → Canvas 2D, v4 pivot noted) |
| `docs/SIGHT-V3-VISUAL-SPEC.md` | §7 invariants updated (Pixi → Canvas 2D) |
| `docs/Constellation Orientation & Onboarding v1.73.md` | v4 pivot documented |

### Orientation chain

- v1.72 → **v1.73** (v3→v4 pivot). Preserved in `docs/`.

---

## Next steps

1. **Build binary** and have Eisa test:
   - Stage 1: close button (× in header row) — THE TEST that has failed 13 times
   - Stage 2: chart renders (stars, rim numbers, Universe Health, legend, Milky Way)
   - Stage 3: zoom/pan (mouse wheel + drag)
   - Stage 4: hover tooltip + click selection + connected-notes side panel + double-click to open note
2. After PASS → §2G.4 mode toggle UI + §2G.5 persistence + §2G.6 audit/close-out
3. MIG-020 (layer peeling + v2 retire)

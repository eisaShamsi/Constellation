# Constellation — Orientation & Onboarding

**Version 1.60 | 2026-05-07**

> **What changed in v1.60** (same day as v1.59; MIG-019 §2G.3c finish + §2G.3d (X, Y, Z) refactor):
>
> **§2G.3c (rim label ellipsis + Universe-name header) and §2G.3d ((X, Y, Z) per-mode dispatch) shipped together** in one commit, with this orientation bump and `docs/SIGHT-V3-VISUAL-SPEC.md` v1.0 → v1.1 inline (per the "orientation in same commit as SO #6 trigger" rule):
>
> - **Rim label ellipsis**: per-wedge tangential chord length emitted as `max-width` per label; long library names truncate with `…` instead of bleeding across adjacent wedges. CSS `text-overflow: ellipsis` on `.sight-v3-rim-label` and `.sight-v3-rim-count`.
>
> - **Universe-name header**: new `universeName` prop on `SightV3.svelte`, passed from `+layout.svelte` as `activeUniverseName`. Renders as a serif italic blue-ink header positioned in the 50 px slot between the Universe Health metrics and the dome top edge. `dir="auto"` so Arabic / Hebrew Universe names render correctly. `getViewport()` TOP_RESERVE bumped 270 → 320 px to accommodate.
>
> - **(X, Y, Z) per-mode grammar** (the architectural elevation Eisa approved earlier this evening — visual spec doc bumps to v1.1):
>   - `src/lib/sight/v3/modes.ts` extended with `ModeContext`, `ModePosition`, `ModeStats`, and a `positionForMode(mode, ctx)` dispatcher.
>   - `positionForRegions`: X = library wedge azimuth, Y = centrality rank percentile, Z = total degree (link count).
>   - `positionForLinkTypes`: Z = outgoingCount works today; X (dominant link type) and Y (type diversity) need `note_links.link_type` piped through, so it currently routes through Regions for X/Y until §2G.4 follow-up.
>   - `positionForTime`: X = creation date wedge (year wedges sized by note count, empty years compressed), Y = recency from `modifiedAt` (today: `createdAt` stand-in until `note_meta.modified_at` is piped), Z = age (oldest = brightest).
>   - C/S/A modes fall back to Regions until their data layers ship per Concept Paper §6.3 P2/P3/P4.
>   - `buildModeStats()` builds universe-wide stats (T mode wedges, time spans) once per fetch.
>   - `recomputeScreenPositions()` in `SightV3.svelte` refactored to use the dispatcher with a reusable `ModeContext` (no per-iteration allocation).
>   - `pathToScreen` map now carries `baseAlpha` so star magnitude is fully mode-aware.
>
> - **Visual spec doc v1.1**:
>   - §1 restructured from "switchable rim axis" to "per-mode (X, Y, Z) grammar" with the full 6-row table per mode (X / Y / Z / cognitive question / data status).
>   - §7 invariants list updated: only **color** is mode-invariant; X/Y/Z are mode-specific. Added invariants 11 (color preserved across mode switches), 12 (rim labels are HTML overlay for native bidi).
>
> **Type-checked clean** (only the pre-existing `LinkLifecycle.fresh` issue in store.ts that's already logged for post-CE follow-up).
>
> **Boss test pending** — installer build + walkthrough delivered next; then §2G.4 cascades (mode toggle UI + 600 ms eased migration + finally lighting up the toggle so the user can switch R ↔ L ↔ T live).
>
> **What's still pending** (post-§2G.3d):
> - §2G.4: mode toggle UI (top-right 6-button bar, R/L/T highlighted "READY", C/S/A dimmed "AVAILABLE LATER") + 600 ms eased migration animation + keyboard shortcuts (R/L/T/C/S/A direct-switch keys + Esc to clear).
> - §2G.5: persist `appSettings.sight.lastMode` per Universe; resolve to `Regions` if a stored mode's data layer isn't ready.
> - §2G.6: 3-agent audit (invariants / drift / migration path) + tag MIG-019 milestone + orientation v1.61 + i18n keys for mode names + close-out.

**Version 1.59 | 2026-05-07**

> **What changed in v1.59** (same day as v1.58; MIG-019 §2A → §2E shipped + §2G in flight):
>
> **MIG-019 §2A → §2E SHIPPED (10 commits today, pushed)** — full v3 §2 surface beyond the §1 foundation:
> - **§2A** TF-IDF compute + similarity IPC (PJ-035 foundation, schema v2)
> - **§2B** Milky Way density wash + Settings toggle
> - **§2A+§2B redesign** density grid replaces edge list (OOM-proof — input-size invariant payload, 256 KB output regardless of universe size). Eisa's directive: "Don't patch it. Solve it."
> - **§2C** calendar rim (Gregorian default + Hijri toggle; Solar Hijri / Hebrew via Settings) + month filter
> - **§2D** universe-health card (modularity / dominance / entropy / connectivity)
> - **§2E** full search integration + always-on labels + Boss-test gate
> - **§2E.1 → §2E.4** four OOM hot-fix commits on Boss-scale 7,636-note universe; root cause was Pixi v8 GPU buffer exhaustion from per-star `new Graphics()` instances. Solve: single Graphics with subpaths + `safeClearContainer` destroying children on remove.
> - origin/main was 36 hours stale; all 10 §2A → §2E commits + §2G work pushed tonight.
>
> **MIG-019 §2G IN FLIGHT (3 commits tonight, pushed)** — visual rewrite from MDS layout to polar layout per Eisa's design directive (`docs/Constellation-Sight-v3-mockup-A2-toggle.svg` + `docs/SIGHT-V3-VISUAL-SPEC.md`):
> - **§2G.1** Visual spec doc + 5 mockup options + Suwaidi cream palette (`bfb8aba`).
> - **§2G.2** Pure helpers: `polar.ts` / `modes.ts` / `regions.ts` (`b1a2477`).
> - **§2G.3 → §2G.3c** SightV3.svelte rewrite cascade (`7d6fcf6`):
>   - Theme: navy → cream parchment.
>   - Polar layout: radius from centrality rank percentile (was direct centrality_norm — distribution skewed packed >90 % at rim); azimuth from library wedge.
>   - Region rim: library wedges sorted by note count desc, empty wedges compressed, blue-ink labels.
>   - Edges: hidden in resting state. Hover/click reveals selected node's links in gold (Concept Paper §4.1).
>   - Universe Health: HTML overlay anchored top-center, four metrics flanking the gold roundel.
>   - Side panel: Universe Health section removed; opens only on star selection.
>   - Calendar-rim hover/click handlers gated on `currentMode === 'time'` (Eisa caught the rim auto-shifting from Regions to calendar months).
>   - Dome margins: 270 px top + 100 px bottom + 100 px sides + 80 px outside-dome reserved.
>   - Region rim labels migrated from Pixi Text to HTML overlay with `dir="auto"` so the browser handles bidi natively for Arabic / Hebrew / mixed-script library names.
>
> **Eisa-approved (X, Y, Z) per-mode grammar** (visual-spec doc bumps to v1.1 in §2G.3d):
> | Mode | X (azimuth) | Y (radius) | Z (magnitude) | Cognitive question |
> |---|---|---|---|---|
> | R · Regions | Library | Centrality rank | Degree | "Where in my cosmos does this idea live, and how central?" |
> | L · Link Types | Dominant outgoing link type | Type diversity | Total outgoing links | "What kind of reasoning, and how versatile?" |
> | T · Time | Creation date wedge | Recency (last edit) | Age | "When did it emerge, and is it still alive?" |
> | C · Confidence | Dominant confidence | Certainty homogeneity | Total link count | "How certain, and how consistent?" |
> | S · Stages | Dominant lifecycle stage | Avg link weight | Total traversal count | "How alive, and how worn the path?" |
> | A · Acts | Which Act produced the note | Synthesis depth | Total connections | "Where in the formulation arc?" |
>
> Color stays invariant across all modes (Louvain community membership). Sight becomes a true multi-instrument cognitive lens — the §5.3 stethoscope / MRI / ECG metaphor finally fully realised.
>
> **Boss tests passed**: §2G.3 visual (cream theme + polar layout + region rim + Universe Health + edges hidden), §2G.3a (territories disabled, side panel cleaned, metric labels fixed), §2G.3b (mode-lock + dome margins + Arabic RTL).
>
> **Standing Order catch-up tonight (~21:50)** — implementation cascade had pulled past doc discipline. Caught up: 3 commits + push tonight; session-log entries for §2G.1 → §2G.3c + state-of-standing record (SO #5) for the X/Y/Z pivot; this orientation v1.59; MoCh due.
>
> **Next**: §2G.3c finish (rim label ellipsis CSS + Universe-name header) → §2G.3d (X/Y/Z refactor) → §2G.4 (mode toggle + 600 ms migration) → §2G.5 (mode persistence) → §2G.6 (audit + close + orientation v1.60).

**Version 1.58 | 2026-05-07**

> **What changed in v1.58** (same day as v1.57; MIG-018 ships v3 projection foundation):
>
> **MIG-018 closes Done** — Sight v3 projection foundation live in production. Six-phase cascade (§1A → §1F) shipped today across 8 commits. The first of three v3-build MIGs per the Concept Paper v1.1 §9 trajectory.
>
> **What's user-visible in v3 §1E**:
> - Star-icon dock button next to where the v2 eye-icon used to be.
> - Dome of stars on Suwaidi-chart deep midnight blue, sized by betweenness centrality (logarithmic 6-magnitude scale).
> - Constellation territories drawn as soft Suwaidi pastel polygons (warm-cream + gold + amber + dusty rose + sandy tan + parchment + antique-white + dark goldenrod cycled by Louvain community id).
> - Faint connector lines visible at rest (Eisa's design call: "we will show it as faint lines until the user hovers over it" — the v3 reframe of v1.1 paper Principle 6).
> - Hover star → tooltip + incident edges brighten; click → constellation lights up + side panel slides in; double-click → opens note in editor.
> - Settings → Sight section: Lambert (default, equal-area) / Stereographic (equal-angle) projection toggle. Switching is free (frontend-only re-projection of the cached MDS embedding).
> - Esc clears selection then closes; deterministic per-snapshot layout means notes return to remembered positions on re-open (spatial-memory grammar working).
>
> **Boss test passed all 11 steps** with `SIGHT_V3_ENABLED = true` flipped locally. Const now committed `true` — production-ready in default config.
>
> **Three-agent audit CLEAN** (0 P0, 0 P1, 0 P2, 0 P3). Audit report at `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md`. Verified all 13 invariants from the Architect, the drift map, and the seven migration-path scenarios.
>
> **What's NOT in v3 yet** (deferred per Concept Paper v1.1 §9.2-§9.3):
> - Milky Way density wash (PJ-035) — MIG-019.
> - Calendar rim (Gregorian default + user-add via Settings) — MIG-019.
> - Universe-health card in side panel — MIG-019.
> - Search flares + halo (basic match-highlight wired in §1E; full version in MIG-019).
> - Magnitude slider / layer peeling (PJ-036) — MIG-020.
> - v2 retirement — MIG-020 (after Boss confirms v3 stable across multiple sessions).
> - PJ-037 (Map↔Sight integration) — REJECTED, not in any v3 MIG.
>
> **Pending Jobs v1.7** (`docs/Constellation Pending Jobs v1.7.md`):
> - PJ-038 status: Confirmed → **In-Progress** (1 of 3 MIGs done).
> - MIG-018 trajectory updated: phase 1/3 closed; MIG-019 next-up.
> - Done count after v1.7: 7 (unchanged). Cancelled: 1 (PJ-034). Rejected: 1 (PJ-037).
>
> **Eight commits today on the v3 trajectory**:
> | Commit | Phase / scope |
> |---|---|
> | `1164b08` | PJ-038 Concept Paper v1.0 drafted |
> | `44c37c9` | PJ-038 Concept Paper v1.1 ratified + PJ-037 rejected + PJ v1.6 + orientation v1.57 |
> | `51e270a` | MIG-018 Architect + Plan |
> | `fe85792` | MIG-018 §1A — schema + Rust skeleton |
> | `24aa6bd` | MIG-018 §1B — Landmark-MDS compute (5 unit tests passing) |
> | `dd6759e` | MIG-018 §1C — frontend skeleton + dock button + i18n 15 locales |
> | `4dc6878` | MIG-018 §1D — star rendering + Lambert/stereographic toggle |
> | `26ce36e` | MIG-018 §1E — territories + faint lines + hover/click + Suwaidi palette [Boss-test passed] |
> | (this) | MIG-018 §1F — audit + close-out + orientation v1.58 + Pending Jobs v1.7 + SIGHT_V3_ENABLED=true committed |

**Version 1.57 | 2026-05-07**

> **What changed in v1.57** (same day as v1.56; PJ-038 v3 Concept Paper ratified, PJ-037 rejected, MIG-018 unblocked):
>
> **Sight v3 Concept Paper v1.1 ratified by Eisa**. `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` is the design contract for the multi-MIG v3 build. v1.0 was the same-day "drafted, awaiting review" state with ten open §11 questions; v1.1 has all ten resolved + two structural revisions: (a) connector lines are now **faint at rest, brighten on hover/select** (replaces v2's "no lines until hover" — Eisa wants the structural pattern visible at rest, just unobtrusive); (b) Map ↔ Sight integration **rejected** — PJ-037 retired in Pending Jobs v1.6.
>
> **Ten §11 design calls (resolved 2026-05-07)**:
> 1. **Embedding**: graph-distance MDS (Landmark variant for memory).
> 2. **Projection**: both Lambert (default) and stereographic ship; user toggle in Settings.
> 3. **Calendar rim**: Gregorian default; users add others (Hijri, Solar Hijri, Hebrew, etc.) via Settings → "Calendar systems."
> 4. **Magnitude slider**: astronomy convention — drag right peels bright stars.
> 5. **Two-up panel**: N/A — Map↔Sight integration rejected (PJ-037).
> 6. **Constellation labels**: hover/select only by default; Settings toggle for always-on.
> 7. **Color scheme**: cycled pastels by Louvain id default; user-overridable via existing Style Settings.
> 8. **Search filter persistence**: Esc + click-background to clear; no persistence across Sight close/reopen.
> 9. **Render layer**: Pixi.js (consistent with Sky View / v2). Two Pixi layers (base + focus overlay) + DOM layer for UI chrome.
> 10. **Accessibility (high-contrast / keyboard nav)**: deferred to a separate post-v3 PJ.
>
> **PJ-037 Rejected**. Sight v3 stays single-view; Map and Sight remain independent surfaces. The "Map diagnoses, Sight prescribes" loop happens in the user's head, not in a shared cursor. Number retired per stable-reference-numbers rule.
>
> **Pending Jobs v1.6** (`docs/Constellation Pending Jobs v1.6.md`):
> - PJ-037 → Rejected.
> - PJ-038 §8 trajectory revised: MIG-020 phase reduced to PJ-036 + v2 retire only (no PJ-037 absorption).
> - Done count: 7 (unchanged). Cancelled: 1 (PJ-034). **Rejected: 1 (PJ-037 — new)**.
>
> **Three-MIG v3 build sequence** (per Concept Paper v1.1 §9):
> - **MIG-018** — Projection foundation. Rust `compute_layout_embedding` (Landmark MDS), `sight_v3_layout` SQLite cache, `src/lib/sight/projection.ts` + `SightV3.svelte`, dock button + Settings entry behind `SIGHT_V3_ENABLED`. Boss-test gate: stars render at correct positions, basic hover/click works.
> - **MIG-019** — Density + time + search. PJ-035 (Milky Way), calendar rim, search integration, universe-health card. Boss-test gate: full visual grammar live.
> - **MIG-020** — Layer peeling + v2 retire. PJ-036 magnitude slider, v2 fallback removal once Boss confirms v3 stable.
>
> **Next-up**: MIG-018 Architect.

**Version 1.56 | 2026-05-07**

> **What changed in v1.56** (same day as v1.55; MIG-017 closes — v2 Sight unreachable in production):
>
> **MIG-017 (PJ-039) shipped — single phase, single commit.** v2 Sight is now unreachable from the running app's user surface in default config. Mechanism: a single code constant `SIGHT_V2_ENABLED = false` in the new `src/lib/sight/engine.ts` module gates four UI surfaces — dock button, modal mount, "Return to Lens" button, Settings → Plugins entry. The v2 component (`ConstellationSight2.svelte`), the `lens*` `$state` fields in `+layout.svelte`, the `toggleLens()` async function, the Rust analytics modules (`lens.rs`, `lenses.rs`), and the `constellation_sight_*` IPCs are **all preserved on disk** as a known-good fallback. Re-enable = flip the const + rebuild.
>
> **Help-doc banner added** at the top of `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` — "Constellation Sight is being rebuilt; v3 is in design; here's the link to the v1.1 Concept Paper for context." Original v2 documentation paragraphs untouched beneath.
>
> **Pending Jobs v1.5** (`docs/Constellation Pending Jobs v1.5.md`):
> - PJ-039 → **Done**.
> - Top of queue rotates: **PJ-038 (Sight v3 + own Concept Paper)** → PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub).
> - Done count after v1.5: 7 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027, PJ-039). Cancelled count: 1 (PJ-034).
>
> **Audit: three-agent** (invariants / drift / migration-path) ran on the diff. Audit report at `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`. 0 P0, 0 P1.
>
> **Next-up — PJ-038**: Sight v3 build with **own dedicated Concept Paper**. Multi-MIG. Star-chart aesthetic per the v1.1 paper §13–§14 vision. v3 inherits the Rust analytics IPCs as-is from v2; rebuilds the visualization layer entirely. PJ-035 (content similarity) / PJ-036 (layer peeling) / PJ-037 (Map↔Sight integration) absorbed as v3 features rather than v2 add-ons.

**Version 1.55 | 2026-05-07**

> **What changed in v1.55** (Boss-directed 2026-05-07; closes the MIG-016 cycle, lands the Sight Concept Paper v1.1, and frames the v3 trajectory):
>
> **MIG-016 closes — Cancelled (partial-shipped).** §1A instrumentation + §1B edges-on-hover gate shipped (commits `a0babbb` → `7e76b17` → `62718f7`). §1C (Web Worker offload), §1D (post-paint prewarm), §1E (SQLite `sight_cache`) **abandoned mid-flight**. Audit close-out at `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`. PJ-034 retired. The "instant first-toggle" headline goal was not met for v2; designed-in for v3 from the start.
>
> **Decision: secure-don't-muddle.** v2 Sight (`ConstellationSight2.svelte` + the `lens_*` Rust modules + `constellation_sight_*` IPCs) is being **disabled as a known-good fallback** under MIG-017 (PJ-039), while v3 is built fresh under PJ-038. The Rust analytics IPCs and the v2 Svelte component **stay on disk** — they are the proven baseline if v3 fails. v3's visualization layer is rebuilt entirely from the **star-chart aesthetic** (Suwaidi northern-hemisphere chart reference; Sight Concept Paper v1.1 §13).
>
> **Sight Concept Paper v1.1 lands.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` is the markdown port of Eisa's April 2026 v1.0 PDF, refreshed with: (a) "What this paper IS" disclaimer, (b) §12 truth-status matrix (each mechanic mapped to *what's actually shipped*), (c) **Principle 6 — reveal-on-demand** (the edges-on-hover gate as a permanent design principle, not a perf hack), (d) three implementation gaps tracked as PJ-035 / PJ-036 / PJ-037, (e) §13 star-chart vision as the design north star, (f) §14 v3 redesign noted with **its own dedicated Concept Paper to follow**.
>
> **Honest delivery score for v2 Sight**: ~70-80% of the Concept Paper's analytical promise. Centrality / community detection / structural gaps / universe-health all real. **Three notable omissions** — content-similarity TF-IDF edges (PJ-035), layer peeling (PJ-036), Map↔Sight integration (PJ-037) — all inheritable into v3 by design.
>
> **Pending Jobs v1.4** (`docs/Constellation Pending Jobs v1.4.md`):
> - PJ-034 closes as **Cancelled (partial-shipped)** — new terminal status added to status vocabulary.
> - **PJ-035** allocated — Sight content-similarity TF-IDF edges.
> - **PJ-036** allocated — Sight layer peeling.
> - **PJ-037** allocated — Map ↔ Sight integration.
> - **PJ-038** allocated — Sight v3 build with own dedicated Concept Paper (multi-MIG, star-chart aesthetic).
> - **PJ-039** allocated — MIG-017 disable v2 Sight (mini-MIG, single session, **next-up**).
>
> **Top of queue**: PJ-039 (MIG-017 disable v2) → PJ-038 (Sight v3 + own Concept Paper) → PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub).
>
> **Done count after v1.4**: 6 (unchanged). Cancelled count: 1 (PJ-034 partial-shipped).
>
> **§17 update**: `Constellation_Lens_Concept_Paper_Eisa.pdf` no longer in "binary docs not read" — read in full this session via `pypdf`, content folded into `docs/Constellation-Sight-Concept-Paper-v1.1.md` markdown port.

**Version 1.54 | 2026-05-06**

> **What changed in v1.54** (same day as v1.53; MIG-016 §1B lands): edges-on-hover gate in Sight, mirroring Sky View's "nervous system" pattern.
>
> **§1A wrap**: data-collection gate generated three mount-trace paste-throughs (175 / 174-188 / 367 ms total) confirming mount is fast. The toggle trace never fired through Eisa's clipboard because the cache-fast path skips the marks once `lensHealth !== null`. Path 2 chosen (skip toggle-trace data; proceed to §1B based on the verifiable-fast mount + the unmeasured-but-likely first-paint render cost). `alert()` / `clipboard.writeText` calls removed from the §1A code (intrusive in-build); `performance.mark` instrumentation + `console.log` retained for future DevTools sessions.
>
> **§1B implementation** in `src/lib/components/ConstellationSight2.svelte`:
> - **`neighborMap: Map<string, Set<string>>`** populated once per `buildSimData()` call (every link contributes both directions). Mirrors `graphEngine.ts:410-429`.
> - **`needsEdgeDraw` gate** in `draw()`: skips the entire `drawLinks()` call unless one of four conditions holds — hovered node, selected node, active search, or hovered link annotation. On the idle-Sight common path, edge-draw cost drops to **zero**.
> - **Hover/select neighborhood filter** at the top of `drawLinks()`: when one node is hovered or selected (and search isn't active and no link annotation is hovered), iteration drops from `O(E)` to `O(degree)` via the `focusOnly` early-skip.
>
> **Boss-test gate next**: install build, toggle Sight (expect nodes-only paint, no edges), hover a node (its neighborhood lights up), search a term (matched nodes' edges show), Escape / move cursor away (edges hide).

**Version 1.53 | 2026-05-06**

> **What changed in v1.53** (same day as v1.52; MIG-016 §1A fix-up): production binary ships with DevTools disabled, so the §1A `console.table` dump alone wasn't usable for the data-collection gate (Eisa: "the developer console won't open with the binary"). Added a clipboard-write + `alert()` fallback alongside the existing `console.log`: after Sight toggle completes (and after the Sight2 mount), the trace is JSON-formatted as a paste-friendly text block, written to clipboard, and a confirmation alert prompts Eisa to paste it into chat. Both alert dialogs fire in sequence (toggle → mount). Console.log retained for any future session where DevTools is enabled.
>
> **Side-observation worth flagging**: production-build DevTools disabled by default is a Tauri default that may be worth re-evaluating since Eisa is the project's operator + tester (not just an end user). Logged as a candidate PJ for next Pending Jobs bump if Eisa wants persistent DevTools access.

**Version 1.52 | 2026-05-06**

> **What changed in v1.52** (same day as v1.51; MIG-016 §1A lands): instrumentation phase. `performance.mark`s wrapped around every step of `toggleLens()` in `src/routes/+layout.svelte:3332-3460` (rust-centrality / louvain / structural-gaps / universe-health / stratum-weighted / top-bridges / community-profiles / bridge-suggestions / total) AND every step of the cold-mount path in `src/lib/components/ConstellationSight2.svelte::onMount` (buildSimData / layout / fitToScreen / total). Both dumps via `console.table` after lensActive flips / mount completes. No behaviour change.
>
> **Boss data-collection gate next**: build, install, open DevTools console, toggle Sight, send the two console.table outputs to Claude. Trace calibrates §1B (edges-on-hover) / §1C (worker offload) / §1D (post-paint prewarm) / §1E (SQLite cache) per-phase budgets.

**Version 1.51 | 2026-05-06**

> **What changed in v1.51** (same day as v1.50; MIG-016 opens — PJ-034 Sight instant-toggle perf): Eisa-directed perf work on Constellation Sight after a three-pass cross-check (v1.50 latest body, then deeper agent reading §4.x bodies + recent session logs, then full-history scan of all 50 orientation versions + 29 session logs).
>
> **Three findings drove the design choices:**
>
> 1. **No prior B-4-style architecture proposal for Sight has been made.** Boot-prewarm + SQLite cache + dedicated worker + edges-on-hover for Sight is net-new ground. The 2026-04-22 §55 in-memory `lensDataStale` cache is the only prior Sight perf layer; MIG-016 supersedes it as the L1 of a three-tier cache.
> 2. **The 2026-04-13 Sight2 redesign decided "all links solid by default"** — Eisa-confirmed reversed today (2026-05-06). New default: edges hidden until hover or search match. Rationale: the 2026-04-13 decision predated the 2026-04-21 Sky View edges-on-hover work that proved how much render headroom that pattern unlocks.
> 3. **PJ-025 reframe** — PJ-025 was retired as OBSOLETE in Pending Jobs v1.2 because Sight is on-demand (not boot-rebuilt). PJ-034 covers a **different perf axis** — first-toggle latency, which the 2026-04-22 §55 in-memory cache doesn't address across session boundaries. PJ-025 stays retired; PJ-034 is the net-new MIG.
>
> **Architect doc**: `lab/reports/PJ-034-SIGHT-INSTANT-TOGGLE-ARCHITECT.md`. Six-phase plan (instrumentation → edges-on-hover → worker offload → post-paint prewarm → SQLite cache → audit). Three Boss-test gates (Phases 1B, 1D, 1E).
>
> **Awaiting Eisa's "Architect approved"** before writing the Plan.

**Version 1.50 | 2026-05-06**

> **What changed in v1.50** (same day as v1.49; Pending Jobs v1.3 closes the deeper cross-check): the deeper cross-check agent (this time reading orientation §4.x BODIES + session logs per the new SO #8) classified all 27 remaining PJ entries against the latest canonical state.
>
> **Outcome — only 1 entry needed flipping**:
> - **PJ-006 (Living Link Architecture P2–P5) → SHIPPED.** Orientation v1.49 §4.4 confirms `_link_traverse / _link_decay / _link_set_confidence / _link_archive` IPCs (P2/P3), `formulationAnalysis` wrapper (P4), and `KnowledgeHealthDashboard.svelte` mounted in `+layout.svelte:5975` (P5). All four phases live and user-validated.
>
> All 27 other entries verified unchanged from v1.2. No new stale entries surfaced. Scope-rewrites in v1.2 (PJ-010, PJ-014, PJ-021) confirmed correct.
>
> **Done count after v1.3**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027).
>
> **Top of queue rotates**: PJ-005 (MIG-007 Links Settings tab) → PJ-002 (cid_cn collision scrub) → PJ-008 (Outgoing Links typed-link dedupe).
>
> **The audit cycle that produced v1.2 → v1.3**: a real demonstration of why the iterative cross-check pattern works. v1.2 caught 5 stale entries (3 OBSOLETE + 2 SHIPPED) but missed PJ-006 because the agent only read preambles. The PJ-006 catch produced SO #8 ("read bodies, not just preambles") which the v1.3 cross-check obeyed and closed cleanly.

**Version 1.49 | 2026-05-06**

> **What changed in v1.49** (same day as v1.48; Standing Order #8 added + PJ-006 catch): Eisa-directed Standing Order: cross-check any PJ before tackling it.
>
> **The catch**: I had just closed Pending Jobs v1.2 (committed `3929bba`) and prepared to cascade into PJ-006 (Living Link Architecture P2–P5). Started by re-reading the PJ-006 entry → discovered orientation §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since v1.40 (2026-05-05). The v1.2 cross-check agent missed this because my own instructions told it to read only the "What changed in vX.Y" preambles, not orientation bodies. Body trumps preamble for canonical state.
>
> **Eisa's response**: "Don't start tackling any PJs unless you cross-check them with the orientation and session log files." Recorded as **Standing Order #8** in CLAUDE.md + memory feedback note `feedback_pj_crosscheck_before_tackle.md`.
>
> **What's next**: Path 1 — bump to Pending Jobs v1.3 with PJ-006 marked OBSOLETE/SHIPPED, AND re-run a deeper cross-check that reads orientation §4.x BODIES (not just preambles) plus session logs. Find any other stale entries the v1.2 audit missed. After v1.3 closes, work the new top-of-queue.

**Version 1.48 | 2026-05-06**

> **What changed in v1.48** (same day as v1.47; Pending Jobs v1.1 → v1.2 cross-check audit): Eisa-directed cross-check of every Pending Jobs entry against the full orientation timeline (v1.0 → v1.47). Outcome: **2 entries closed (PJ-001, PJ-007 already SHIPPED), 3 entries verified OBSOLETE (PJ-025 Sight, PJ-026 sidebar stars, PJ-027 Map — all already write-time-derived or cache-fast), 3 entries scope-rewritten (PJ-010 alias-bleed, PJ-014 doc-body backfill, PJ-021 narrowed to "verify-then-narrow"), 6 new entries allocated PJ-028 → PJ-033** (MIG-014 §2F audit P2/P3 follow-ups carried from memory).
>
> **Pending Jobs v1.2** (`docs/Constellation Pending Jobs v1.2.md`) is the new canonical backlog. v1.1 stays as iteration record per the doc-versioning convention. Stable reference numbers preserved — PJ-025/026/027 retired but their entries kept with OBSOLETE status; numbers never reused.
>
> **Top of queue** (per v1.2 Quick Reference): PJ-006 (Living Link Architecture P2–P5) → PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub).
>
> **Living Link Architecture P2–P5 (PJ-006)** is unblocked now that PJ-007 closed. It's the multi-MIG that completes the Living Link Architecture as a whole — needs its own Concept Paper before the Migration Rule cascade.

**Version 1.47 | 2026-05-06**

> **What changed in v1.47** (same day as v1.46; MIG-015 closes): §1D three-agent audit complete. Audit report at `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md`.
>
> **Audit verdict**: 11 of 12 invariants ✅. One P0 found and fixed in the close-out commit: the DB mutex was held across the whole chunked loop, contradicting the §1B design promise + §1C Boss-test claim ("you can edit notes, search, switch tabs while it runs"). Three agents converged on the same finding at three severities (invariant: P1, drift: P0, migration-path: P2). Fix: refactored the chunked helper to a single-chunk `sentinel_bigram_rows_chunk(conn, chunk_size)`, with the worker (`run_v2_sentinel_migration`) doing the lock dance per chunk + 10ms inter-chunk yield. Other DB callers now see ~10ms availability windows between chunks as originally promised.
>
> **Visual Boss test on §1C skipped per Eisa's call.** Boss's library is already at v2 from earlier MIG-013 testing; rolling back to manufacture migration work would touch closed-feature production data. Working Agreement #4 forbids "let's see what happens" on closed-feature data. Static audit verifies behaviour by code-reading; future users with pre-MIG-013 backups will exercise the visible path naturally.
>
> **MIG-015 STATE — CLOSED.** PJ-001 (chunked v2 sentinel migration with progress UI) shipped. The deferred P1-M1 from MIG-013 §1E is now closed.
>
> | Phase | Scope | Status | Commit |
> |---|---|---|---|
> | §1A | Rust helpers (count + chunked-helper) | Done | `0ca7e64` |
> | §1B | init_db defers; async task wired | Done | `df0bf87` |
> | §1C | Frontend strip + 15-locale i18n | Done; visual test skipped per Eisa | `62d3b4a` |
> | §1D | Three-agent audit + P0 fix | Done | (this commit) |
>
> **Next**: PJ-006 — Living Link Architecture P2–P5 (multi-MIG, the link-side lifecycle work that PJ-007 unblocked). Eisa queued for after MIG-015.

**Version 1.46 | 2026-05-06**

> **What changed in v1.46** (same day as v1.45; MIG-015 §1B + §1C land): §1B moves the v2 sentinel migration off the boot critical path (deferred to a worker thread spawned from `ensure_search_db_ready`, mirroring the `sky_backfill::maybe_schedule` pattern). §1C ships the frontend status-bar progress strip (`MigrationProgressStrip.svelte`) in a new `.sb-center` group + i18n keys in all 15 locales (`migrationProgress.termVocabV2.label` and `.done`).
>
> **MIG-015 phase status**:
>
> | Phase | Scope | Status | Commit |
> |---|---|---|---|
> | §1A | Rust helpers (count + chunked) | Done | `0ca7e64` |
> | §1B | init_db defers; async task wired | Done | `df0bf87` |
> | §1C | Frontend strip + 15-locale i18n | Done; **awaiting Boss test** | (this commit) |
> | §1D | Three-agent audit | Pending | — |
>
> The Boss test for §1C verifies: (a) installing the new MSI on a library with a manually-rolled-back schema version produces a fast first paint with the strip visible; (b) the completed counter climbs steadily; (c) the strip self-hides 4 seconds after `done`; (d) crash recovery resumes correctly via the WHERE-clause filter.
>
> **Architect doc**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`. **Plan**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`.

**Version 1.45 | 2026-05-06**

> **What changed in v1.45** (same day as v1.44; MIG-015 opens): MIG-015 (PJ-001) starts — chunked v2 sentinel migration with progress UI. §1A lands the Rust helpers (`count_pending_v2_sentinel_rows`, `sentinel_bigram_rows_chunked`) next to the existing `sentinel_bigram_rows`; no behaviour change yet.
>
> **Why MIG-015 exists.** The MIG-013 v2 sentinel migration converts every `term_vocab` bigram row's `bridge_concept_id` from NULL → `'-'`. On Boss-equivalent libraries (~5.7M rows) the bulk UPDATE blocks boot for 30–90 sec with no UI feedback. Boss is past it; new users with pre-MIG-013 backups would hit it once and see a frozen splash. PJ-001 closes this gap.
>
> **Design (Option C, approved):** defer the v2 step off the boot critical path; spawn a one-shot async task that runs the chunked migration with progress emit. 100k rows per chunk. Tauri event channel `migration:term_vocab_v2`. Frontend status-bar strip in a new `.sb-center` group. All 15 locales updated upfront (no PJ-014 deferral). Crash-recoverable by construction.
>
> **Phase rollout** (mini-MIG):
>
> | Phase | Scope | Status | Commit |
> |---|---|---|---|
> | §1A | Rust helpers (count + chunked) | Done | (this commit) |
> | §1B | init_db defers; async task wired | Pending | — |
> | §1C | Frontend strip + 15-locale i18n | Pending; Boss-test gate | — |
> | §1D | Three-agent audit | Pending | — |
>
> **Architect doc**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`. **Plan**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`.
>
> **Next after MIG-015**: PJ-006 (Living Link Architecture P2–P5 — multi-MIG, the link-side lifecycle work that PJ-007 unblocked). Eisa queued it for after PJ-001 closes.

**Version 1.44 | 2026-05-06**

> **What changed in v1.44** (same day as v1.43; MIG-014 closes): §2F three-agent audit complete. Audit report at `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`.
>
> **Audit findings:**
> - Invariant agent: PASS — all 10 invariants hold (LIVING_LINK_BASELINE.length === 6, single-arg lookupStageEmoji, Law 2.7 satisfied, M11 zero-diff intact).
> - Drift agent: 3 P0 + 2 P1, all `[pre-existing]` — fixed in close-out commits.
> - Migration-path agent: PASS for P0/P1; 2 P2 + 4 P3 logged as memory follow-ups (`project_mig014_audit_p2_p3_followups.md`).
>
> **Close-out fixes (P0 — write paths still emitting dropped Zettelkasten values):**
> - `src/lib/components/FocusPane.svelte` — Promote button **removed entirely** per Eisa's Option B. `onexit` simplified from `(promote?: string) => void` → `() => void`. Caller in `+layout.svelte` simplified. `focusPane.promote` i18n keys deleted (en + ar).
> - `src/lib/components/ExpressionForge.svelte` — composition note now writes `stage: maturity` (was `synthesis`).
> - `src/lib/components/SenseMakingCanvas.svelte` — canvas-promoted note now writes `stage: growth` (was `permanent`).
>
> **Close-out fixes (P1 — read paths missing `spark` + `archived` typo):**
> - `src/lib/components/KnowledgeHealthDashboard.svelte` — Lifecycle Cards now use all 6 baseline keys; `archived` → `archival`; `spark` added.
> - `src-tauri/src/search.rs` — `lifecycle` aggregation buckets aligned with `LIVING_LINK_BASELINE`. DB enum stays `'archived'` for back-compat; bucket key uses `archival`.
>
> **MIG-014 STATE — CLOSED.** PJ-007 (Note Stage Taxonomy) shipped via the per-note dash-encoded model. §2A → §2F complete. The §1A → §1D commits stay as the iteration record per Eisa's call.

**Version 1.43 | 2026-05-06**

> **What changed in v1.43** (same day as v1.42; MIG-014 §2E ships): the help + User Manual rewrite for the new Stages model lands. Eisa confirmed §2C+§2D Boss test PASSED after the Law 2.7 architectural fix.
>
> **Doc updates:**
>
> - `docs/User Manual.md` §18.6 — "Externalization Engine" rewritten as "Stages — the Living Link lifecycle". Six fixed lifecycle stages (Spark / Birth / Growth / Maturity / Dormancy / Archival) replace the old Zettelkasten 4 (Fleeting / Literature / Permanent / Synthesis). Per-note custom-term suffix model (`spark-concept`, `birth-concept`, …) documented with Mode A / Mode B dropdown explanation.
> - `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` Feature 6 — same rewrite.
> - `docs/help.ar/User Manual.md` §18.6 — Arabic equivalent (الشرارة / ولادة / نمو / نضج / سُبات / أرشفة).
> - Multi-Lens "By Stage" reference updated in both User Manuals to point at the Living Link lifecycle instead of the old four-stage Externalization Engine.
> - 13 other locales' User Manuals queued via PJ-014 backfill (de / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh).
>
> **Old Zettelkasten values still display** via `LEGACY_ZETTELKASTEN_EMOJI` for any pre-MIG-014 notes; they aren't promoteable in the new chain.
>
> **MIG-014 §2 status**: §2A → §2E shipped. §2F (three-agent audit) is the only remaining phase before MIG-014 closes.

**Version 1.42 | 2026-05-06**

> **What changed in v1.42** (same day as v1.41, after the §2C+§2D Boss-test fix sequence): adds **Law 2.7 — Single source of truth: properties have one parent** to `Constellation Development Laws v1.4.md` (NEW alongside v1.3).
>
> **Why it landed.** Three patches in a row failed to keep the breadcrumb / Properties / file tree surfaces in sync during the MIG-014 §2C+§2D Boss test. Eisa: "Enough patching." Root cause: three components each held a local `$state` copy of the stage value; each surface updated through a different path. Fixes re-aligned two surfaces while leaving the third drifting. The architectural fix made `currentStage` in NotePane a `$derived` of the prop instead of a local `$state` mirror, removed the local-mutation lines on the promote/demote click handlers, and stripped the `onstagechange` local-setter — every surface now derives from the on-disk content (proxied by `openTabs[id].content`). One source, one update path through `handlePromote → writeNote → openTabs.update → parsed re-derives → stage prop re-passes → derived chain refreshes`.
>
> **Generalisation.** The rule isn't stage-specific. Title, tags, links, body — every first-class property the user can edit through more than one surface — has one canonical owner; UI surfaces are subfunctions that derive. Local `$state` mirrors are forbidden. Edit buffers (input typing), UI-only state (dropdown open/closed), and caches with clear invalidation paths are the named exceptions.
>
> **MIG-014 §2 status update**: §2A → §2D shipped; the §2D fix sequence (commits `bb7a6ef → e3a97a1`) cleared the §2C+§2D Boss-test failures. Awaiting Eisa retest before declaring §2C+§2D passed and moving on to §2E (help + User Manual) and §2F (audit).

**Version 1.41 | 2026-05-06**

> **What changed in v1.41** (next day after v1.40; MIG-014 mid-cascade): the Pending Jobs document, the Constellation Development Laws bumped 1.0 → 1.3, the NotePane Specs distilled from 121 commits, the MoCh convention added as Standing Order #7, two new top-principal feedback memories, MIG-014 opened with the Note-Stage Taxonomy migration: Architect + Plan + §1A → §1D iteration (proven-wrong model) + Stages Concept Paper v1.0 → v1.2 + Plan v2 → v4 + §2A → §2D shipped (correct model). PJ-007 status: in-build, awaiting Boss test on the §2C+§2D combined gate.
>
> **Two key process events** that produced new durable rules:
>
> 1. **MoCh — Minutes of Chating** (Boss-directed 2026-05-06). Every ~3 hours of direct chat, write a fresh file at `docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md` recording Boss ↔ Claude interaction (questions, steers, decisions, outputs) — distinct from the session log (which captures *what shipped*). Recorded as **Standing Order #7** in CLAUDE.md and as feedback memory `feedback_minutes_of_chating.md`. First file: `docs/MoCh/MoCh-2026-05-06-0900.md`.
> 2. **SO #6 inline-with-commit reinforcement** (Boss-directed 2026-05-06). After v1.40 landed, ten hours of triggering changes (Laws v1.1→v1.3, Pending Jobs, NotePane Specs, MoCh convention, MIG-014 Architect/Plan/Build cascade, Stages Concept Paper, §1A→§1D, §2A→§2D) accumulated *without a single orientation bump*. Boss asked "Why do I have to remind you?" Recorded as feedback memory `feedback_orientation_inline_with_commit.md` (top-principal): orientation v-bump lands IN THE SAME COMMIT as any SO #6 trigger; no batching, no waiting for the next commit. v1.41 closes the drift.
>
> **Constellation Development Laws — v1.0 → v1.3.**
>
> - **v1.1** — Law 1.6 added (State the function in hand). [Already in v1.40 preamble.]
> - **v1.2** — Law 2.6 corrected: cUniverse moved out of the required hierarchy layer to a sibling-federation position; structure is `Universe → (Library | Folder | Note)` with cUniverse as an optional federated peer.
> - **v1.3** — Law 2.6 refined: Universe root is itself a default Library (`is_universe_notes` flag in `libraries.json`). The Universe is *not* a higher abstraction over Libraries; it is a Library with an additional federation role. Notes and folders may live directly under the Universe root.
>
> **Constellation Pending Jobs v1.1** (`docs/Constellation Pending Jobs v1.1.md`, NEW alongside v1.0). Durable project backlog. **Stable reference numbers** introduced as a top-of-document rule: each pending job carries a unique sequence number `PJ-NNN` that is never reissued, even after the job is shipped or abandoned. Numbers act as cross-document references (memory notes, session logs, commits, MoCh files all use `PJ-NNN`). Twelve jobs catalogued: PJ-001 through PJ-012. Status vocabulary: Pending / Confirmed / In-Build / Shipped / Abandoned.
>
> **NotePane Specs v1.0** (`docs/NotePane Specs v1.0.md`, NEW). Distilled from the 121-commit `NotePane.svelte` history. Twelve sections, fourteen hard invariants, twelve forbidden anti-patterns. Each statement sourced to a commit hash. **§3.5 corrected post-write** (commit `9973e65`): the breadcrumb is `[← demote] [emoji label badge] [promote →]` — promote/demote arrows + badge, NO dropdown. Earlier draft mistakenly captured the dropdown experiment from commit `6cbe87c` as final state; that experiment was undone at `90c1ea8` (§136, 2026-05-02 redesign). The current code at `NotePane.svelte:918-958` has no dropdown.
>
> **MIG-014 — Note-Stage Taxonomy** (closes PJ-007). The currently-active migration. Two model iterations:
>
> 1. **Iteration 1 (§1A → §1D, kept as iteration record)** — flat extensibility model. `CustomStage { name, emoji }` struct + `custom_stages: Vec<CustomStage>` field on UniverseMeta + 5 IPC commands. PropertyEditor combobox + custom inline dropdown + emoji chain across breadcrumb / file tree / Inspector360. Boss test surfaced multiple model failures: long promote chain doesn't scale; custom emoji adds visual noise; per-Universe scope wrong. **Stays in `main` as the iteration record per Eisa's call (don't rewrite history).**
> 2. **Iteration 2 (§2A → §2D, current)** — per-note dash-encoded model. UniverseMeta restored to pre-§1A shape. The custom term lives only as the dash suffix in each note's frontmatter `stage:` value (e.g. `stage: spark-concept`). The PropertyEditor combobox is a 6-entry mode-flip dropdown: Mode A (input empty / matches a fixed name) → 6 baselines; Mode B (custom word in input or dash suffix) → 6 paired stages. The breadcrumb chain walks the lifecycle and carries the suffix verbatim. No Universe-wide settings, no Settings panel.
>
> **Stages Concept Paper v1.0 → v1.2** captures the model evolution:
>
> | Version | Model                                                | Status     |
> | ------- | ---------------------------------------------------- | ---------- |
> | v1.0    | 2D matrix with multiple custom types                  | Superseded |
> | v1.2    | Per-note custom term, dash-encoded, 6-entry mode-flip dropdown | **Current** |
>
> Both versions kept in `docs/` as a thinking-trace per the orientation-versioning convention.
>
> **MIG-014 §2 Plan v4** (`lab/reports/MIG-014-NOTE-STAGE-PLAN-v4.md`) is the active plan. Six phases:
>
> | Phase | Scope                                                          | Status              | Commit    |
> | ----- | -------------------------------------------------------------- | ------------------- | --------- |
> | §2A   | Rust schema cleanup                                            | Done                | `2f58b8a` |
> | §2B   | Frontend store cleanup + new pure helpers                       | Done                | `59ed95c` |
> | §2C   | PropertyEditor mode-flip combobox                               | Done; Boss-test pending | `432076c` |
> | §2D   | NotePane breadcrumb chain walks within suffix                   | Done; Boss-test pending | `2c58bda` |
> | §2E   | Help + User Manual (en + ar)                                   | Pending             | —         |
> | §2F   | Three-agent audit                                              | Pending             | —         |
>
> §2C + §2D are paired Boss-test gates; combined MSI build in flight at v1.41 commit time.
>
> **Five top-principal feedback rules now in memory** (per `~/.claude/projects/.../memory/MEMORY.md`):
>
> 1. `feedback_dont_make_things_up.md` (BASIC RULE — top of all rules).
> 2. `feedback_secure_dont_muddle.md` (validate every change against full architecture).
> 3. `feedback_tutorial_tests_and_cascade.md` (every test as tutorial; plan = build approval).
> 4. `feedback_minutes_of_chating.md` (MoCh — NEW 2026-05-06).
> 5. `feedback_orientation_inline_with_commit.md` (orientation v-bump in the triggering commit — NEW 2026-05-06, post-Eisa-correction).
>
> **What's open at v1.41 commit time**:
>
> - MIG-014 §2C+§2D MSI build in flight; Boss-test tutorial pending on completion.
> - §2E help + User Manual rewrite (en + ar; PJ-014 backfill).
> - §2F three-agent audit.
> - PJ-014 — 13-locale i18n + User Manual backfill (carried).
> - Carried-over: LinkLifecycle dedupe (Option B approved, deferred until post-CE); pre-MIG-013 backups hit blocking v2 sentinel migration.

**Version 1.40 | 2026-05-05**

> **What changed in v1.40** (same day as v1.39; MIG-013 closes): **MIG-013 §1D-C help + User Manual updates shipped, §1E three-agent audit complete, two cleanup P1s applied, one P1 deferred with documentation. MIG-013 (CTSE Bridge Adapter) is closed.**
>
> **Plus a new Foundational law**: **Law 1.6 — State the function in hand** (Boss-directed 2026-05-05 after the §1D wrong-target post-mortem). At task start, every fresh session, every surface pivot, and every correction, name the function in hand in one line. Predecessor Lookup, Stop-on-Correction, Testing Instructions, and Migration Rule all read against this anchor. Without it they float. Recorded as a top-principal in `CLAUDE.md` and as Law 1.6 in `docs/Constellation Development Laws v1.1.md` (NEW alongside v1.0).
>
> **§1E audit findings** (`lab/reports/MIG-013-CTSE-AUDIT.md`):
>
> | audit | P0 | P1 | result |
> |---|---:|---:|---|
> | Invariant (Rules 1-4 + 8, M11 zero-diff, IPC, tokenizer symmetry, tests) | 0 | 0 | PASS |
> | Drift (16 retired surfaces, dead schema, dead i18n, stale comments) | 0 | 2 | PASS WITH P1 |
> | Migration-Path (4 scenarios: fresh / pre-MIG-013 / interrupt / rollback) | 0 | 1 | PASS WITH P1 |
>
> **Two P1s applied in close-out** (`b66101b → 5a0be3c → 11410dd → b66101b → b5ce03a → close-out commit`):
>
> 1. **P1-D1**: dead `settings.index.semanticSearch` i18n block removed from all 15 locale files. ~300 dead lines cut. The block was the abandoned MIG-012 toggle UI strings; the toggle UI itself was removed in §1D-B but the keys hung on. Cleaned via Python script; verified zero references in `src/`.
> 2. **P1-D2**: four stale Rust doc comments in `src-tauri/src/search.rs` (lines 88+, ~455+, ~483+) referencing the retired `ctse_run_backfill` / "the backfill" as live writers. Rewritten to reflect post-§1D Option B dead-schema status — the column survives as forward-compat; the v2 sentinel migration is now defensive-only.
>
> **One P1 deferred with documentation**:
>
> 3. **P1-M1**: the v2 sentinel migration's bulk UPDATE blocks boot for tens of seconds to minutes on pre-MIG-013 DBs (~5.7M bigram rows on Boss-equivalent libraries) with zero user feedback. Boss has already completed the migration on the §1D-D binary; new pre-MIG-013 backups would hit it once. Memory entry filed: `project_mig013_v2_migration_blocking_boot.md`. Fix is bounded (chunk + Tauri progress events) but ships as a focused mini-MIG before any v1.0 release — out of MIG-013 scope.
>
> **§1D-C help + User Manual updates** (en + ar):
>
> - `docs/help.uConstellation.World/Index/Index.md` — replaces the trailing "planned next step" caveat with a full **Cross-language Filter — `≈ similar`** section. Three-layer filter breakdown (literal substring / bridge `via {lemma}` / concept `≈ similar`), plain-language mechanism for layer 3, expected-misses note.
> - `docs/User Manual.md` — same shape (English).
> - `docs/help.ar/User Manual.md` — same shape (Arabic, with localized variable names: `≈ مشابه`).
> - 13 other locale User Manuals deferred to the existing `project_user_manual_13_locales_backfill.md` queue.
>
> **What MIG-013 shipped (final state):**
>
> - `bridge_vectors` — 30 MB baked asset (20K M11 concepts × 384 f32 vectors), built once at compile time via `cargo run --bin build_concept_vectors --release`. Loaded once at runtime via `OnceLock`.
> - `ctse::hooks` — write-time `term_vocab` ledger maintenance via `on_note_indexed` / `on_note_deleted`. Per-save tokenize → signed delta → upsert. Pure local bookkeeping, no ONNX in the write path.
> - `ctse::search::ctse_search_terms_by_concept` — query-time concept expansion for the IndexPanel filter `≈ similar` row. Embeds query, finds top-K M11 concepts, expands to multilingual lemmas via the `concept_lemmas()` map (built once at boot from `LexiconGraph`), tokenizes each lemma into FTS5-namespace stems, returns the subset that exists in `term_vocab`.
> - `term_vocab.bridge_concept_id` column + index + v1/v2 schema migrations — dead schema, forward-compat preserved.
> - M11 zero-diff invariant intact across the entire migration.
>
> **Architecture aligns with industry best practice** (Lucene `SynonymGraphFilter`, SQLite FTS5 Method 2, CLIR query-translation, Primo controlled-vocabulary expansion). No backfill, no first-fill, no per-library setup cost. Reacts automatically to new M11 releases.
>
> **Boss test passed** Stage 1 (function alive end-to-end on the rebuilt binary) and Stage 2 (coverage + steady-state timing). MIG-013 closes.
>
> **What's next (post-MIG-013)**: there's no immediate follow-up. Open queue items: P1-M1 mini-MIG (chunked bigram-sentinel migration), the long-deferred 13-locale User Manual backfill, and the always-on cross-language `≈ similar` setting may eventually want a kill-switch toggle if Boss wants noise control. None are urgent.

**Version 1.39 | 2026-05-05**

> **What changed in v1.39** (same day as v1.38; §1D wrong-target correction + new top-principals + Constellation Development Laws v1.0): **CTSE concept search lands in the IndexPanel filter — its correct home — replacing the SearchHub `concept` category that v1.38 wired up by mistake. Two new top-principals added to CLAUDE.md (Predecessor Lookup Rule, Stop-On-Correction Rule) so the same drift can't recur. New durable artifact: `docs/Constellation Development Laws v1.0.md` — distilled from CLAUDE.md, every prior orientation version, every session log, every LL entry, and the Boss-feedback record.**
>
> **The §1D wrong-target incident**. After the Option B pivot (v1.38), I shipped the cross-language search wired to **SearchHub** (the global Ctrl+Shift+F modal). MIG-012's predecessor `searchTermsSemantic` had lived in the **IndexPanel filter dropdown** with a `≈ similar` badge. Four explicit pointers (Settings flag named `index.semanticSearchEnabled`, IndexPanel was the actual call site, MIGs 010/011/012 all operated on IndexPanel, the Settings progress strip was under Settings → Index) all said "Index panel"; I read past every one and shipped the wrong target. Boss correction: "SearchHub? But we are working on the Index!" After a follow-up "How can you be confused?" — explicit acknowledgement that the orientation/SO scaffolding exists precisely to prevent this kind of drift, and that I bypassed every layer.
>
> **Two new top-principals in CLAUDE.md** (Boss-approved 2026-05-05):
>
> - **Predecessor Lookup Rule** — before removing/moving/replacing any user-facing feature, IPC, settings entry, or UI wiring, write a `Predecessor → Replacement` entry into the day's session log: *where it lives now* (file path, function name, settings path, predecessor MIG number) → *where its replacement goes* (default: same place; different place ONLY with explicit Boss approval) → *what gets cut and kept*. Verified against the orientation doc, not memory. The entry comes BEFORE any code edit. Now active for §1D-D and onward.
> - **Stop-On-Correction Rule** — when Boss says "wrong target", "you're confused", "no", "unacceptable", or any equivalent course-correction, STOP all in-flight code edits, list everything changed since the last explicit Boss approval, state the corrected understanding, wait for "proceed". No pivot-and-power-through. Overrides Plan-Approval-Equals-Build-Approval (a correction revokes the cascade approval).
>
> **Constellation Development Laws v1.0** (`docs/Constellation Development Laws v1.0.md`, NEW). A durable, higher-order companion to CLAUDE.md. Five Parts:
>
> | Part | Theme | Sample laws |
> |---|---|---|
> | I | Foundational | 1.1 Don't make things up · 1.2 User is Boss not lab assistant · 1.5 Cross-check against proven methods |
> | II | Engineering | 2.1 Fast software is the best software · 2.2 The Migration Rule · 2.3 Architectural-impact review · 2.4 Constraint as design |
> | III | Process | 3.1 Plan = Build approval · 3.2 Predecessor Lookup · 3.3 Stop on Correction · 3.4 Standing Order · 3.5 Tutorial tests · 3.7 Verify binary · 3.8 Walk through writes |
> | IV | Communication | 4.1 Plain language · 4.2 Don't muddle · 4.3 Reuse don't duplicate · 4.4 One-sentence end-of-turn |
> | V | Recovery | 5.1 No more than 3 patches · 5.2 Backup at milestones · 5.3 State-of-standing before pivot · 5.4 Avoid destructive shortcuts |
>
> Plus **Appendix A: dated timeline of canonical violations** — six entries through 2026-05-05. Each violation produced a law. The Laws doc is updated frequently (new top-principal added → new law; recurring failure pattern crystallizes → new law; Boss correction → new law). Each version is a NEW file; older versions stay as historical record. Same convention as the orientation doc.
>
> **§1D-D — IndexPanel restoration (this commit)**. The Predecessor Lookup entry was written into today's session log before any code edit, per the new Law 3.2:
>
> - **Predecessor**: MIG-012 `searchTermsSemantic` per-keystroke effect in `src/lib/components/IndexPanel.svelte`, gated on `$appSettings.index.semanticSearchEnabled`, populating `semanticMatches` and rendering `≈ similar` badge in the filter dropdown.
> - **Replacement**: SAME PLACE (IndexPanel filter dropdown). The badge UX is restored byte-for-byte. The per-keystroke effect now calls `ctseSearchTermsByConcept` over the new `ctse_search_terms_by_concept` Tauri command. Toggle gate omitted (always-on) per Law 2.4 — no per-library setup cost with CTSE, no reason to add a toggle to disable a now-free feature.
> - **Cut**: SearchHub `concept` category integration (reverted). `ctse_search_by_concept` (note-returning) → renamed and reshaped to `ctse_search_terms_by_concept` (term-returning, IndexPanel-shaped). `ctseSearchByConcept` / `CtseConceptHit` frontend types removed. The `searchHub.concept` and `searchBadges.concept` i18n keys stay (no longer used; harmless dead keys, will GC later).
> - **Kept**: §1C hook simplifications. `term_vocab.bridge_concept_id` dead schema (forward-compat). `concept_lemmas()` in-memory map. `bridge_vectors` matrix asset. The renamed `ctse::search` Tauri command.
>
> **The new read path** (Index panel filter, `≈ similar` row):
>
> 1. User types "knowledge" in the IndexPanel filter input.
> 2. Layer 1 (literal substring) — match the user's query against term names. Always on. Existing.
> 3. Layer 2 (cross-language bridge, MIG-010/MIG-011) — query M11 for cross-language equivalents, surface those terms with a `via {lemma}` badge. Existing.
> 4. **Layer 3 (CTSE concept expansion, NEW in §1D-D)** — embed the query, find top-K M11 concepts, expand each to multilingual lemmas, tokenize each lemma through `fts5_tokenizer::tokenize_to_vec` to get FTS5-namespace stems, look up which stems exist in `term_vocab`. Surface those terms with the `≈ similar` badge. Per-keystroke debounced (300 ms; CLAUDE.md Rule 3). Always on (no toggle).
>
> Per-query cost: ~50 ms e5 inference + ~5 ms cosine k-NN + sub-ms in-memory map lookup + sub-ms SQL `term IN (...)` lookup. Reacts automatically to new M11 releases.
>
> **What's pending after this commit**: Boss reinstalls the rebuilt binary and tests Stage 1 of the IndexPanel cross-language flow (open Index panel → type "knowledge" → expect Arabic terms like "معرف" to appear with the `≈ similar` badge in the dropdown). Then §1D-C (help files + User Manual updates) and §1E (three-agent audit).

**Version 1.38 | 2026-05-05**

> **What changed in v1.38** (same day as v1.37; mid-§1D Boss test, second pivot): **the entire CTSE backfill / first-fill pipeline is retired. After two mid-test fixes (bigram-explosion v1.37, then a half-million-stem slow-path projection at 2+ hours), Boss asked `cross-check this with proven methods used by similar coding communities`. Five parallel WebSearches against Lucene, Elasticsearch, SQLite FTS5, CLIR research, and library-platform documentation surfaced an unanimous industry pattern: query-time concept/synonym expansion, NOT index-time term tagging. Lucene retired index-time `SynonymFilter` for query-time `SynonymGraphFilter` in 2017; SQLite FTS5 docs explicitly list query-time expansion as Method 2; CLIR canonical technique is query-translation; Primo / Ex Libris controlled-vocabulary expansion is at search time. CTSE now follows the same pattern.**
>
> **What that means concretely**: the `ctse_search_by_concept` Tauri command is now self-contained. It embeds the user query, finds top-K M11 concepts via cosine k-NN against the baked 20K-concept matrix, expands each concept to its multilingual lemmas via an in-memory `concept_id → [lemmas]` map (built once at boot from `LexiconGraph`, ~5 MB, ~10 ms), unions and deduplicates the lemmas, and runs an FTS5 OR-clause MATCH against `notes_fts`. No `term_vocab.bridge_concept_id` reads, no per-term backfill, no first-fill, no boot-time wait. Boss's library doesn't need to wait through any concept-resolution job to test cross-language search — the rebuilt binary is immediately functional.
>
> **CLAUDE.md Working Agreement #5 added** (this commit): *"Cross-check every non-trivial fix or design against proven methods before applying it."* Before locking in any subsystem-crossing fix or feature, run parallel WebSearch queries against how mature systems and communities solve the same problem (Lucene, Elasticsearch, SQLite, vector DB practice, library science, IR/CLIR research, PKM tools), compare honestly, surface both options to Boss with the tradeoffs that matter, and pick the battle-tested pattern over the inventive one. Canonical violation: §1D-A backfill was an inventive solution to a problem the dominant industry pattern simply doesn't have.
>
> **Code surface deleted by Option B**:
> - `src-tauri/src/ctse/backfill.rs` (entire file: `ctse_run_backfill`, `ctse_cancel_backfill`, `ctse_backfill_status` Tauri commands).
> - `src-tauri/src/ctse/first_fill.rs` (entire file: `ctse_first_fill`, `ctse_first_fill_status`, `ctse_cancel_first_fill` Tauri commands).
> - `ctse::resolve_term_pure`, `ctse::resolve_term_to_concept`, `ctse::resolve_term_multilang`, `ctse::fast_path_concept_id` (the resolver helpers — orphaned).
> - `ctse::hooks::fast_path_resolve_new_terms` (the per-save concept resolution).
> - 7 frontend store wrappers (`ctseFirstFill[Status]`, `ctseRunBackfill`, `ctseBackfillStatus`, `ctseCancel*`) + `CtseFillProgress` / `CtseFillPhase` / `CtseFillStatus` types + `ctseFillStatus` writable.
> - The `+layout.svelte` boot-time auto-fire `$effect` + the bottom-of-viewport status-bar strip + 50 lines of associated CSS.
> - The `ctse.firstFillProgress / firstFillDone / backfillProgress / backfillDone / cancelled` i18n keys (en + ar).
>
> **Code surface kept**:
> - `ctse::hooks::on_note_indexed` and `on_note_deleted` (term_vocab count maintenance — the Index panel still consumes `term_vocab`).
> - `ctse::search::ctse_search_by_concept` Tauri command (the read path).
> - `bridge_vectors` module (the 20K-concept matrix asset and its loader).
> - `term_vocab.bridge_concept_id` schema column + index + the v1/v2 migrations (dead but idempotent — preserved for forward-compat in case a future "deep concept tagging" feature wants to populate it again).
> - SearchHub frontend wiring for the `concept` category, the `searchHub.concept` and `searchBadges.concept` i18n keys.
>
> **Net diff**: ~580 lines removed, ~80 added. Dramatically simpler architecture. Same query latency (~80 ms end-to-end). Same cross-language coverage (in-vocabulary terms, ~20K M11 concepts × 15 languages each). Reacts automatically to new M11 releases — no rebuild-the-concept-index step ever required.
>
> **§1C/§1D-A/§1D-B status under Option B**:
> - §1C (`5aac7fa`) — schema + write-time hook + retired init_term_embeddings: STILL VALID. The schema column stays as dead-but-harmless; the hook stays simplified.
> - §1D-A (`7b52f1d`) — first-fill + concept search backend: SEARCH PATH SUPERSEDED, first-fill module deleted. Tauri search command rewritten.
> - §1D-B (`0ac12eb`) — frontend wiring + Settings cleanup: SearchHub `concept` category + Settings cleanup BOTH STAY. The boot-time auto-fire + status-bar strip + frontend store wrappers are deleted.
> - §1D bigram-explosion fix (`9aba974`) — schema v2 bulk-sentinel: STAYS as dead-but-idempotent migration. The follow-up SQL filter in `ctse::backfill::next_batch` is moot since the file no longer exists.
>
> **What's pending after this commit**: Boss reinstalls the rebuilt binary and runs the combined Stage 1+2 test (now condensed because there's no backfill phase to wait through): open SearchHub, type a cross-language query, expect concept-category hits in the other script. Then §1D-C (help files + User Manual updates) and §1E (three-agent audit).

**Version 1.37 | 2026-05-05**

> **What changed in v1.37** (same day as v1.36; mid-§1D Boss test): **MIG-013 §1D-A and §1D-B shipped + bigram-explosion fix landed. The first §1D Boss-test launch hit a near-freeze: `term_vocab` had 5.73 million NULL `bridge_concept_id` rows on Boss's 7,639-note library — ~50K real stems plus ~5.68M bigrams (every adjacent stem-pair across all notes, joined by `BIGRAM_SEP` = U+001F). The backfill correctly skipped each bigram in microseconds but the sheer volume (11K+ batched UPDATE transactions) saturated the SearchState mutex, hung the WebView, and projected at ~2 hours wall-clock to finish.**
>
> **Working Agreement #4 lesson**: the architect doc (`MIG-013-CTSE-ARCHITECT-v2.md §3.3`) said "long-tail proper nouns, code identifiers" without quantifying the bigram contribution. The previous `init_term_embeddings` flow filtered terms with `total_count >= 20` before any work — implicitly excluding bigram noise. When §1C removed `init_term_embeddings`, that load-bearing filter was removed without realizing it. Should have run `SELECT COUNT(*) FROM term_vocab` against Boss's library before shipping §1C; did not. Logging this and adding pre-ship counter-measure to the migration checklist (see §17 / Lessons-Learned candidate LL-029).
>
> **§1D-A (`7b52f1d`) — first-fill + concept-search backend**:
> - `ctse_first_fill` Tauri command (`src-tauri/src/ctse/first_fill.rs`) — chunked-transaction walk over `note_meta.body_text`, re-fires `on_note_indexed(old=None)` per row inside 50-note transactions. Resumable via the shared `term_embed_cancel` atomic. Companion `ctse_first_fill_status` returns true iff `term_vocab` is empty AND `note_meta` has body content (the frontend gate). Cancellation via `ctse_cancel_first_fill`. Emits `ctse-firstfill-progress` events.
> - `ctse_search_by_concept` Tauri command (`src-tauri/src/ctse/search.rs`) — embeds the query, picks top-K M11 concepts above a tunable cosine threshold (`DEFAULT_MIN_SCORE = 0.55`, `CONCEPT_TOP_K = 10`), expands to every term_vocab row whose `bridge_concept_id` matches, builds an FTS5 OR-clause MATCH (200-term cap, phrase-quoted), returns notes with snippets. Cross-language for free. Bigram terms (containing U+001F) filtered out of the OR clause. Per-call cost: ~50 ms e5 inference + ~5 ms cosine sweep + sub-ms term lookup + FTS5 MATCH.
>
> **§1D-B (`0ac12eb`) — frontend wiring + Settings cleanup**:
> - `store.ts` — adds `ctseSearchByConcept`, `ctseFirstFill[Status]`, `ctseRunBackfill[Status]`, `ctseCancel[FirstFill|Backfill]` IPC wrappers + `CtseConceptHit` / `CtseFillProgress` / `CtseFillPhase` / `CtseFillStatus` types + the module-scoped `ctseFillStatus` writable. Removes `searchTermsSemantic`, `initTermEmbeddings`, `cancelTermEmbeddings`, `termEmbeddingStatus`, `termEmbedProgress`, `TermSimilarity`, `TermEmbedProgress`.
> - `+layout.svelte` — boot-time `$effect` after `graphReady` listens for both progress streams (push payload into `ctseFillStatus`), calls `ctseFirstFillStatus` → `ctseFirstFill` (if needed), then `ctseBackfillStatus` → `ctseRunBackfill` (if NULLs remain). Status-bar strip is a fixed bottom-of-viewport banner subscribed to `$ctseFillStatus`, hides 4s after `done`. Listeners cleaned via `cleanupFns`.
> - `SettingsModal.svelte` — removed the entire MIG-012 semantic-search block (toggle UI, progress strip, Rebuild Term Embeddings button, $effect / listen / UnlistenFn / untrack auto-trigger). The `index.semanticSearchEnabled` flag stays in the schema with no readers (future GC will drop it).
> - `IndexPanel.svelte` — removed the per-keystroke `searchTermsSemantic` effect + `semanticMatches` Map + `≈ similar` badge + `semanticSearchEnabled` prop. Pure literal substring + bridge expansion browsing now.
> - `SearchHub.svelte` — added `concept` category alongside the six existing ones. `triggerSearch` calls `universalSearch` and `ctseSearchByConcept` in parallel; concept hits are mapped to `ConstellationSearchResult` and stuffed into `response.concept` for the existing rendering loop. CTSE failures degrade gracefully (warn-and-continue). Cyan "≈" badge.
> - `i18n` — en/ar add `searchHub.concept`, `searchBadges.concept`, and `ctse.{firstFill,backfill}{Progress,Done}/cancelled` keys. 13 other locales fall back per the established backfill pattern.
>
> **Cargo.toml fix (`46b3675`)** — added `default-run = "constellation"` to disambiguate Cargo's main binary now that §1A introduced a second `[[bin]]` (`build_concept_vectors`). `cargo build --lib` (used during all §1A–§1D verifications) didn't catch it; full Tauri bundling did.
>
> **Bigram-explosion fix (this version)** — bumps `TERM_VOCAB_BRIDGE_SCHEMA_VERSION` 1 → 2. New `sentinel_bigram_rows()` helper in `search.rs::init_db` runs once on the v1→v2 migration: a single bulk `UPDATE term_vocab SET bridge_concept_id = '-' WHERE bridge_concept_id IS NULL AND term LIKE '%' || CHAR(31) || '%'` that turns 5.68M useless backfill candidates into pre-sentinelled tombstones in sub-second wall-clock. `ctse::backfill::next_batch` and `count_null_rows` also gain `AND term NOT LIKE '%' || CHAR(31) || '%'` filters as belt-and-suspenders against future writes. After this fix:
>
> | metric | before fix | after fix |
> |---|--:|--:|
> | term_vocab rows visible to backfill (Boss's library) | 5,729,974 | ~50,000 |
> | wall-clock to drain backfill | ~2 hours, UI hung | ~5–10 min, UI responsive |
> | bigram rows in `term_vocab` | unchanged (~5.68M, kept for FTS5) | unchanged (~5.68M, all sentinelled) |
> | mutation pressure on the SearchState mutex | 11K+ batched UPDATE tx | ~100 batched UPDATE tx |
>
> **What's pending (next session)**: (a) Boss re-runs Stage 1 of the §1D test on the rebuilt binary (the lock-up should be gone, the strip should show "~50,000" not "~5,700,000"). (b) Stage 2 of the test — actual cross-language SearchHub query. (c) Help files + User Manual updates (§1D-C). (d) Three-agent §1E audit (invariants, drift, migration-path).

**Version 1.36 | 2026-05-05**

> **What changed in v1.36** (same day as v1.35; cascade continues from §1B → §1C): **MIG-013 §1C shipped — write-time hooks + slow-path backfill scaffold + retirement of the legacy `init_term_embeddings` Tauri pipeline. Term vocabulary is now maintained on every note save via the same FTS5 tokenizer that backs `notes_fts`; new terms get a fast-path M11 concept lookup (microseconds, no ONNX) immediately, and the slow-path resolution for misses runs in a separate Tauri command (resumable, cancellable, batched). The whole `term_embeddings` table + bulk `populate_term_vocab` bootstrap is gone.**
>
> **What landed in §1C**:
> - **Schema migration** — `term_vocab.bridge_concept_id TEXT` column + supporting index, gated by `schema_versions.term_vocab_bridge = 1`. Idempotent ALTER TABLE; fresh DBs and existing DBs converge.
> - **Write-time hook** (`src-tauri/src/ctse/hooks.rs`, NEW) — `on_note_indexed(conn, path, old_body, new_body)` tokenizes both bodies via `fts5_tokenizer::tokenize_to_vec`, computes signed per-term `(total_delta, doc_delta)`, upserts `term_vocab`, and fast-path-resolves M11 concept ids for newly-introduced terms. `on_note_deleted(conn, path, body)` subtracts contributions; tombstones (zero counts) are kept so revival is free. **No ONNX in the write path** — slow-path is the backfill's job. 1 MiB body cap matches the prior `BODY_CAP_BYTES` precedent. Stopword set cached at module level via `OnceLock`.
> - **Wire-in** — `search.rs::reindex_single_note` and `reindex_delete_note` now read `note_meta.body_text` once before and once after `index_note`, then call the hook. Hook errors are logged but never fail the reindex (term_vocab is a derived view; file + note_meta are the sources of truth).
> - **Slow-path backfill** (`src-tauri/src/ctse/backfill.rs`, NEW) — three Tauri commands: `ctse_run_backfill`, `ctse_cancel_backfill`, `ctse_backfill_status`. Walks `WHERE bridge_concept_id IS NULL ORDER BY total_count ASC LIMIT 500` (TF-IDF descending — rarest first → search becomes useful early). Per-term resolution via new `ctse::resolve_term_multilang(app, term)` (15-language fast-path FST sweep, then e5 inference + cosine k-NN). Sentinel `'-'` for "tried and failed" so re-runs visit only new NULL rows. Resumable per batch transaction. Cancellation reuses `EmbeddingState.term_embed_cancel: AtomicBool` (orphaned by §1C-5). Emits `ctse-backfill-progress` events.
> - **Retired** — `init_term_embeddings` / `cancel_term_embeddings` / `search_terms_semantic` / `term_embedding_status` Tauri commands; `populate_term_vocab` (the Phase 1 rayon bootstrap); `blob_to_vec` (orphan); `TermEmbedProgress` and `TermSimilarity` payload structs; the `term_embeddings` CREATE TABLE. The `term_vocab` comment in `init_db` updated to point at the §1C write-time-derivation maintenance path.
> - **Shared helpers in `ctse/mod.rs`** — new `pub fn fast_path_concept_id(graph, term)` (multi-language lookup; bigrams skipped) and `pub fn resolve_term_multilang(app, term)` (fast-path-then-slow-path; used by backfill).
>
> **Verification — 9 ctse tests + 6 bridge_vectors tests green**. New ctse::hooks tests cover first-time index (insert + fast-path), idempotent resave (zero delta), edit (signed delta), delete (tombstone), and bigram tokens (stay NULL). M11 zero-diff invariant holds: `git diff src-tauri/src/lexicon/` empty.
>
> **Known gap (resolved in §1D)**: the SettingsModal frontend still calls the four removed Tauri commands. Toggling semantic search ON in Settings throws at runtime on the first IPC. **No Boss test in this gap** — §1D follows immediately and removes the call sites + the `termEmbedProgress` writable store + the `confirmDialog` for "Rebuild Term Embeddings".
>
> **Known gap on existing libraries**: `term_vocab` rows from the prior `populate_term_vocab` bootstrap have `doc_count = 0` (the bulk loader skipped that field). Cosmetic only — the backfill cursor is the NULL filter on `bridge_concept_id`, ordering is `total_count` (correct from before).
>
> **What's pending (§1D, next)**: auto-trigger first-fill on boot when `term_vocab` is empty (walks `note_meta.body_text` and re-fires `on_note_indexed`); auto-trigger `ctse_run_backfill` on boot when NULL rows exist; status-bar progress strip subscribed to `ctse-backfill-progress`; new `ctse_search_by_concept` Tauri command + frontend wiring; full Settings UI cleanup (kill the four old IPCs' call sites, `termEmbedProgress` writable, "Rebuild Term Embeddings" confirm dialog); update help files + User Manual (15 languages) for cross-language Constellation Sight. **First Boss-testable gate fires at §1D** (cross-language semantic search). **§1E**: three-agent audit per Migration Rule §4.

**Version 1.35 | 2026-05-05**

> **What changed in v1.35** (next day after v1.34): **MIG-013 §1A + §1B shipped — full architectural pivot of the term-vocabulary semantic-search pipeline. Per-library term-embedding is retired; the 20K M11 controlled-vocabulary concepts are embedded once at build time and shipped with the binary as a 30 MB asset. Library size becomes irrelevant for semantic search.**
>
> **Why the pivot**: MIG-012 fix-1 / fix-2 (v1.34) shipped the auto-trigger + status UI, but on Boss's 7,635-note multi-script library the underlying `init_term_embeddings` flow stalled at note 601/7635 (Phase 1 single-thread bootstrap), then later at total=0 for 7+ minutes (Phase 1.5 batch-ONNX with heartbeat-after-fetch). Three SME audits (parallel-systems, library/IR, application-architecture) independently concluded **the unit being embedded was wrong** — LCSH/MeSH/AAT (the canonical IR pattern since 1909) embed the controlled vocabulary, not the patron's corpus. M11 is exactly that controlled vocabulary. Boss directive 2026-05-05: "Go for A. But don't touch the M11's ~20K concepts."
>
> **Hard constraint** (Boss): `lexicon/` source files have a zero-line diff at every CTSE commit. Verified mechanically by `git diff src-tauri/src/lexicon/` returning empty. CTSE reads M11; never writes to it.
>
> **§1A shipped (`5e1c0f1`) — build-time concept-vector pipeline**:
> - New offline `[[bin]]` target `build_concept_vectors`: reads M11's seed TSV (read-only via `lexicon::parse`), picks one canonical surface form per concept (en > zh > es > fr > de > ... fallback chain), embeds with multilingual-e5-small in batches of 128 on `available_parallelism()` threads, validates per-vector L2-norms, writes asset.
> - New `src-tauri/src/bridge_vectors/` module (stub at this phase): `ASSET_MAGIC = b"CTSEBV01"`, `VECTOR_DIM = 384`. Layout: 8-byte magic + u32 count + u32 dim + concept-id table (u16 LE byte_len + UTF-8) + 4-byte-aligned f32 LE row-major matrix.
> - New `pub fn embeddings::embed_passages_standalone(model_path, tokenizer_path, texts, intra_threads, batch_size)` — builds its own ONNX session without an `AppHandle`; chunks through the existing `run_embedding_batch` pipeline. Runtime engine path is unchanged.
> - Visibility flips in `lib.rs`: `arabic`, `embeddings`, `lexicon` → `pub mod`. Purely additive; in-crate access paths unchanged. Required so the build helper can name `lexicon::ConceptRecord`, `arabic::Lang`, and `embeddings::embed_passages_standalone`.
> - **Numbers from the build run**: 20,000 concepts parsed, 100% English coverage (fallback chain never fired), 1,008.5 passages/sec on 24 threads, 19.8 sec total embed, 29.6 MB asset (committed to repo per Boss directive — changes only when `lexicon_v1.tsv` does).
>
> **§1B shipped (`909e381`) — runtime loader + Bridge Adapter**:
> - `bridge_vectors/asset.rs` — `parse()` over `include_bytes!("data/concept_vectors_v1.bin")`. Copies into owned `Box<[f32]>` to avoid the f32-alignment hazard of reinterpreting `include_bytes!` data as `&[f32]`.
> - `bridge_vectors/store.rs` — `ConceptVectorStore` with `nearest_concept` (top-1) and `nearest_concepts_k` (small-k via sorted Vec, beats BinaryHeap for k≤32). Cosine over flat row-major matrix.
> - `bridge_vectors/mod.rs` — `pub fn get() -> &'static ConceptVectorStore` singleton via `OnceLock`.
> - New `src-tauri/src/ctse/` module — Bridge Adapter:
>   - `resolve_term_pure(graph, store, embed_query, term, lang, threshold)` — pure dependency-injected core; closure invoked **only when M11 fast path misses**.
>   - `resolve_term_to_concept(app, term, lang)` — Tauri-context wrapper; pulls singletons + delegates query embed to `embeddings::constellation_embed_text`.
>   - `DEFAULT_THRESHOLD = 0.78` (initial guess from e5 model card; tunable in §1D).
> - **Fast path**: `LexiconGraph::find_nodes(lang, lemma)` (M11 already-public method) → direct `graph.nodes[idx as usize].concept_id` (M11 already-public field). Microseconds; no ONNX. Hits ~80% of expected terms.
> - **Slow path**: e5 embedding of the query term + cosine k-NN against the 20K matrix. Used only when fast path misses.
> - **10 tests, all green**: 5 store unit (synthetic basis vectors), 1 baked-asset round-trip (real 30 MB asset), 4 adapter (real M11). The fast-path test uses a panicking-on-call closure to verify the slow path is never invoked when M11 has the lemma.
>
> **What's pending (next session)**: §1C (Rust-side only — re-scoped from approved Plan): schema migration `term_vocab.bridge_concept_id`, write-time hook in `reindex_single_note` (fast-path-only resolution; no ONNX in write path), Tauri `ctse_run_backfill` command (NULL-row walker, batched, resumable, sentinel-marked failures), removal of `init_term_embeddings` + `term_embeddings` table + `populate_term_vocab`. **§1D**: auto-trigger backfill, status-bar progress UI, `ctse_search_by_concept` query path, full Settings UI cleanup. **First Boss-test gate fires at §1D** (cross-language semantic search). **§1E**: three-agent audit per Migration Rule §4. Resume checklist in `lab/reports/SESSION-LOG-2026-05-05.md` "State-of-standing" section.
>
> **What gets retired in §1C**: `init_term_embeddings` Tauri command + the entire per-library term-embedding loop (the source of every Phase 1.5 freeze). The `term_embeddings` table is dropped (note: existing DBs leave the table dangling — harmless but worth a future GC). The Settings modal "Rebuild Term Embeddings" button + `termEmbedProgress` writable store + the v1.34 status line `✓ N terms indexed` all go with it. Replaced by silent write-time derivation (Rule 8) and a single status-bar progress strip when the backfill is running.
>
> **Where to read the design**: `lab/reports/MIG-013-CTSE-ARCHITECT-v2.md` (architecture; supersedes v1 which is preserved as historical record), `lab/reports/MIG-013-CTSE-PLAN.md` (phase-by-phase commits with verification clauses), `lab/reports/SESSION-LOG-2026-05-05.md` (full session record + state-of-standing).
>
> **Standing migration checklist update (LL-027 candidate)**: when an audit returns three independent SME reports converging on "the unit of work is wrong, not the implementation", **stop iterating on the implementation** — ship a fresh Architect doc that pivots, not another fix-N. Five fix-3 → fix-9 attempts to scale `init_term_embeddings` would have continued indefinitely without the cross-discipline audit. The audit pattern (X-ray view + Library/IR view + App-architecture view) is the right shape for any "we keep band-aiding the same hot path" symptom. To be confirmed and added to LL after §1E.

**Version 1.34 | 2026-05-04**

> **What changed in v1.34** (same day as v1.33, post-MIG-012 polish): **MIG-012 Build.7-fix-1 + Build.7-fix-2 shipped — auto-trigger semantic-init when toggle flips ON, plus visible status line and manual Rebuild button.** Closes the "deferred follow-up" logged in `MIG-012-AUDIT.md`.
>
> **fix-1 (`91356b1`)**: SettingsModal `$effect` watches `$appSettings.index.semanticSearchEnabled`; on ON-flip calls `termEmbeddingStatus()` and, if count is 0, attaches a `term-embedding-progress` Tauri event listener and fires `init_term_embeddings(false)`. Progress UI (live counter + accent-fill bar + Cancel button) renders inline below the toggle, driven by a new module-scoped `termEmbedProgress` writable store so the job survives Settings modal mount/unmount. On OFF-flip while running: cancels via `cancel_term_embeddings`, waits for the cancelled-event flush, then clears UI after 4 sec. Resumable: re-firing skips already-embedded terms via the existence check.
>
> **fix-2 (`dd3b2e5`)**: when toggle is ON and no job is in progress, a status line shows the current state — **`✓ {N} terms indexed`** (when count > 0; ready to use) or **"Index not built yet"** (when count == 0; either freshly toggled or a real bug). Rebuild / Build now button on the same row gives users a manual escape valve, useful when models change in the future and especially as the only way to verify state in production builds (Tauri disables DevTools at release-build time, so the user can't `invoke('term_embedding_status')` from console). Lesson: **production-Tauri-builds-disable-DevTools means every state the user might want to verify needs visible UI affordance.** Logged for future MIGs introducing background jobs.
>
> Boss verified PASS — status line correctly shows **`✓ 18,200 terms indexed`** on a Universe where yesterday's MIG-012 G2 session embedded the table. Confirms fix-1 correctly skips re-init when populated AND fix-2 surfaces the truth visibly.
>
> **i18n**: 9 new keys × 15 locales (3 from fix-1 + 6 from fix-2). Full ar+en, English placeholders in 13 others per established backfill pattern.
>
> **Standing migration checklist update (LL-026)**: when introducing a long-running background job that affects user-visible feature state, the migration MUST also include (a) a UI status indicator visible without DevTools and (b) a manual trigger affordance (Rebuild / Force / Run-now button). Mandatory for ship.

**Version 1.33 | 2026-05-04**

> **What changed in v1.33** (same day as v1.32; Boss "Proceed all" cascade): **THREE more MIGs closed back-to-back — MIG-011, MIG-012, plus a pre-existing script-filter bug fix and the note-stage-taxonomy-decision queue.** The Index function went from "mentions-side cross-language" (v1.32) to a full vocabulary search engine across all three retrieval layers: literal substring (always-on), lexical-bridge (M11 corpus, 20K concepts × 15 langs), semantic (multilingual-e5-small ONNX embeddings).
>
> **Pre-existing script-filter bug fix** (`5dbb43f`): typing Arabic in the Index filter while script-tab "All" was active returned 0 results until the user bounced through "عربي" once. Two layers — substring-direction-mismatch (FTS5 stores stems shorter than typed surface forms; the bidirectional `query.includes(term)` check was gated on comma-mode-only) and stale-letter-filter persistence (clicking a Latin letter then typing Arabic dropped Arabic terms via the active letter filter). Both fixed; bidirectional substring is now always active and the letter filter auto-clears when filtered entries don't match it.
>
> **MIG-011 closed — cross-language Index *filter* expansion.** Mirror of MIG-010 applied to the search box: typing "knowledge" surfaces Arabic terms `معرف` / `علم` with `via knowledge` badges; typing `معرفة` surfaces English `knowledg` / `cognit` with `via معرفة` badges. New Tauri command `lexicon_expand_for_filter`; frontend per-keystroke debounce 300ms + cancel-token + per-session cache; same Settings toggle drives both surfaces (one mental model, two behaviors). 5 build commits + simplify + audit. Boss verified PASS at G2.
>
> **Side-discovery during MIG-011 G2 testing** (`c95a0e6`): two i18n keys (`indexPanel.returnToIndex` + 6 Living Link lifecycle stages under `notePane.stage.*`) were rendering as raw literals in the Arabic interface — and audit showed they were missing in **all 15 locales**. Backfilled with full ar+en + English placeholders in 13 others. The deeper question — should Notes use Living Link lifecycle stages (`spark/birth/growth/maturity/dormancy/archival`) or Zettelkasten stages (`fleeting/literature/permanent/synthesis`)? — queued as `project_note_stage_taxonomy_decision.md` for Boss design call.
>
> **MIG-012 closed — Index Search Engine: search history + semantic search.** Boss-approved Q1.A + Q2.C + Q3.B (term-level embeddings, lazy-on-first-semantic-query bootstrap, SQLite-per-Universe history). Two new tables (`term_embeddings`, `index_search_history`) with idempotent `CREATE TABLE IF NOT EXISTS` for transparent migration. 4 new Rust IPCs for embeddings (`init_term_embeddings` with progress events, `cancel_term_embeddings`, `search_terms_semantic`, `term_embedding_status`) + 3 for history (`read_index_history`, `write_index_history_entry`, `clear_index_history`). Frontend: 2 new Settings toggles + Clear button, per-keystroke debounced semantic search (mirrors MIG-011 pattern), filter loop now matches across direct → bridge → semantic with priority, `≈ similar` cyan badge for semantic matches, history dropdown on filter focus, full Arabic translation. 8 build commits + simplify + audit + confirm-dialog fix. Boss verified PASS at all three G2 stages.
>
> **§Build.8 simplify caught 3 Tier 1 issues** that would have shipped to users: (1) `init_term_embeddings` held `EmbeddingState.engine` and `SearchState.db` for the entire ~10–20 min embed-all loop, freezing every concurrent IPC during the job — fixed via lock-per-iteration. (2) f32 LE BLOB encode/decode duplicated between note + term + read paths — extracted `vec_to_blob` / `blob_to_vec` helpers; existing `constellation_embed_notes` migrated to use them too. (3) `TERM_EMBED_CANCEL` was a process-global static; moved to `EmbeddingState` for per-app-instance scope. The simplify methodology earned its keep on this MIG.
>
> **§Build.8-fix (`8d98a3a`)**: Boss G2 stage 1 step 6 surfaced that the browser-native `confirm()` dialog couldn't honor app i18n — both message text and OK/Cancel buttons stayed English even on the Arabic interface. Replaced with the existing `ConfirmDialog.svelte` component for the Clear-history button; Arabic users now see fully-localized "حذف نهائي... / مسح / إلغاء". Pattern for any future confirmation surface.
>
> **Boss-approved follow-on workstreams (logged 2026-05-04, NOT yet started)**:
> - Note-stage taxonomy decision (Living Link lifecycle vs Zettelkasten) — `project_note_stage_taxonomy_decision.md`. Quick i18n fix shipped today; deeper architecture decision deferred.
> - Auto-trigger semantic-init when toggle flips on — Plan-promised but currently the init must be invoked explicitly. Manual trigger via DevTools available (`init_term_embeddings`). Logged for Build.7-fix-1.
> - Search history toggle: track this with the rest of the deferred items in the existing backfill workstream.
>
> **Lessons logged this round (LL-025)**: simplify pass with parallel review agents earns its keep on cross-subsystem migrations. The lock-per-iteration find on MIG-012 §Build.8 would have shipped a real ~20-min freeze to Stage 2 testers without the simplify check — caught before binary release. Lesson: **for any migration that adds a new long-running background job, `/simplify` is mandatory before the Boss G test.** Adding to the standing migration checklist.

> **What changed in v1.32**: **MIG-010 closed — Lexical Bridge integration into the Index panel.** Boss directive: "finish and implement the Index function." Build cascade ran §A (Phase A bug fix — register `read_cooccurring_terms` in `tauri::generate_handler!`, the chip-strip cooccurrence panel was silently broken pre-MIG-010) → Architect doc → Plan doc → §Build.1 (`pub(crate)` bridge helpers + parameterize `find_match_via_marked` for STX/ETX vs `<mark>` delimiter regimes) → §Build.2 (`read_term_mentions` extended with `expand_cross_language: Option<bool>`; new `via_lemma: Option<String>` on IndexMention; `build_term_match_clause` helper with 4 unit tests) → §Build.3 (Settings: new "Index" section + `indexExpandCrossLanguage: bool` toggle in 15 locales) → §Build.4 (IndexPanel reads setting, renders `via_lemma` badge with `dir="auto"`) → §Build.4-fix (G2 cosmetics: off-state visual contrast + RTL toggle slider mirror; latent G3 fix attempted) → §Build.4-fix2 (defensive expansion fallback + frontend error catch — diagnostic infrastructure) → §Build.4-fix3 (the actual G3 root cause: `$effect` in IndexPanel read `mentionsCache.size` making the cache its own dependency → Rule 2 violation — wrapped cache reads in `untrack()`) → §Build.5 (`/simplify` three-agent pass — fixed Tier 1 prop-coupling via `cacheKey?: unknown` rename, Tier 2 `LexicalExpansion::into_parts()` accessor + `fts_quote_phrase` extraction + flatten `match` block + `prepare_cached` + gated `eprintln`, Tier 3 magic-pixel comment) + Phase 4 Audit doc.
>
> **Boss verified PASS at G2 + G3** — screenshot showed Arabic notes ("2007", "2010", "428 هـ") with **`via علم`** badges + Spanish-language reference ("Ada Lovelace") with **`via conocimiento`** badge. The 7,600-note mixed Arabic/English library is now searchable by *concept* across languages, not just by literal lemma. Audit at `lab/reports/MIG-010-AUDIT.md` confirms all 11 invariants hold.
>
> **Phase D (boot perf)**: deferred `readIndexEntries()` from `graphReady` to first Index-panel open. ~tens of ms saved on every boot for users who don't open the Index that session. Cost paid on demand.
>
> **Phase E (docs)**: dedicated Index help page at `docs/help.uConstellation.World/Index/Index.md`. User Manual §7 + Arabic User Manual §8 updated with cross-language toggle subsection. 13 other locale User Manuals queued in existing `project_user_manual_13_locales_backfill.md`.
>
> **Phase G (guidance)**: teaching doc `docs/help.uConstellation.World/Index/Index Guidance — How to Read Your Vocabulary.md` — three reads (frequency profile, language-pair balance, cognitive adjacency), five common patterns + readings, weekly-practice ritual. Boss-pattern teaching doc, modeled after the queued 360.3D Stratification Matrix guidance.
>
> **Lesson logged (LL-024)**: `$effect` body must declare its dependencies explicitly. The §Build.4-fix3 root cause (cache-invalidation effect tracked the cache it managed → infinite-clear loop) is a CLAUDE.md Rule 2 violation that I shipped without an end-to-end IPC trace. New rule: for any cross-subsystem `$effect` work, run a console-level trace BEFORE the Boss test cycle. Working Agreement #4 self-correction.
>
> **Boss-approved follow-on workstreams** (logged 2026-05-04, NOT yet started):
> - **MIG-011** — cross-language Index *filter* (mirror of mentions expansion, applied to the search box). Today the filter does substring matching only; bridge-aware filtering is the next step.
> - **MIG-012** (eventually) — Index search engine: search history + semantic search powered by existing `embeds.rs` ONNX pipeline. Memory: `project_index_search_engine_history_semantic.md`.
> - Pre-existing Index script-filter bug ("All" hides Arabic terms until "عربي" bounce). Memory: `project_index_script_filter_all_hides_arabic.md`.
> - "Rebuild Index" button — explicitly **deferred** per Rule 8 (no `rebuild_*` commands; FTS5 triggers maintain the index at write-time). Memory: `project_index_rebuild_button_decision.md`.
>
> **Phase C status**: Settings → Boot-perf scorecard turned out to be ALREADY shipped (5-criterion view in `SettingsModal.svelte`); STATUS.md was stale on this. Rebuild Index button deferred per above.

> **What changed in v1.31**: **MIG-008 closed.** Build cascade ran §145 (CreateItemDialog component + i18n en+ar) → §146 (wire New Folder) → §147 (wire New Note) → §148 (wire New Base, replace NewBaseDialog) → §149 (wire New Library + new `create_new_library_at` Rust IPC) → §150 (orphan sweep — five state vars + two functions + `NewBaseDialog.svelte` deleted) → §151 (Boss-flagged context-menu gaps: folder right-click missing "New Base" + library-row right-click falling through to browser-default menu — both fixed) → §152 (Build.7 /simplify checkpoint: i18n backfill 13 locales, `create_new_library_at` async, IME composition guard, KIND_LABELS lookup, `parseFrontmatter` instead of hand-rolled regex, dropped `defaultName` prop + `lastOpenState` $effect, `baseSelectedSet` for O(1) lookup, plus four Boss-approved adds — right-click "New note" now applies folder templates the same way the toolbar does, `/libraries` route migrated to the dialog, path-traversal hardening on Rust create IPCs via `sanitize_name`) + docs commit (User Manual + 2 help articles + Arabic User Manual) + audit doc. Boss verified PASS across all 8 create scenarios on the §151 binary plus the four §152-specific verifications (templates, route migration, path traversal, IME). Audit at `lab/reports/MIG-008-AUDIT.md` confirms all 11 invariants (I1–I11) hold. Project memory `project_create_dialog_standardize.md` marked SHIPPED.
>
> **Logged for follow-up**: 13 User Manual translations (`project_user_manual_13_locales_backfill.md`); reserved-Windows-name + trailing-dot/space hardening on Rust create IPCs (pre-existing gap, not MIG-008-introduced); collision popup (`project_rename_collision_popup_wanted.md`) — pre-existing, will compose with the dialog when shipped.

> **What changed in v1.30** (MIG-008 starts; §142–§144 closed MIG-006 §4):
>
> **MIG-008 — Create-Dialog Standardization (Phase 1 Architect committed at `22839d4`)**. Boss directive 2026-05-03: "Whenever I created a folder it is created in the respective location under the name 'New Folder'. It shouldn't work this way. What I want it to do is to follow the standard way of any file system. A popup dialog box should emerge to name the new folder and to choose the location. Same thing should happen when creating new note, base or library." Architect plan at `lab/reports/MIG-008-CREATE-DIALOG-ARCHITECT.md`. Inventory found four inconsistent create flows (Folder rejects collisions / Note auto-increments / Base has its own `NewBaseDialog` / Library has folder picker only); 11 invariants (I1–I11) defined; three options enumerated (A: shared modal, B: inline tree-row input, C: rich modal with templates); **Option A approved by Boss**. Phase 2 Build cascade kicks off in 8 steps (§Build.1–.8): build shared `<CreateItemDialog>` component → wire each of the four affordances → drop orphaned auto-create handlers → /simplify → audit. Each step pauses for Boss-testable verification clause.
>
> **MIG-006 §4 closed (§142 + §144)**. Original gap from §3-redo Stage 1 testing: Outgoing Links / Backlinks panels stayed stale after wikilink rename cascade because the SQLite index wasn't reindexed and the frontend's `allLibraryLinks` `$state` was loaded once at boot and never refreshed. **§142** plugged the Rust side (cascade walker calls `reindex_single_note` for each rewritten path; new `library_name` parameter on the `update_links_on_rename` IPC). **§143** attempted a frontend-side targeted update of `allLibraryLinks` but only matched entries whose `target` equaled the rename's `oldName` exactly — after several renames in a session (Hub v4 → v5 → … → v8) the in-memory state had drifted further than any single rename's `oldName`, so the targeted match never fired. **§144** superseded §143 with the simpler drift-resistant fix: re-fetch `cache_boot_snapshot_graph` post-cascade and replace `allLibraryLinks` + `notePathToAliases` wholesale. Catches not just the just-rewritten target but any drift accumulated in the session. Boss tested PASS — Outgoing Links panel updates immediately after rename, no app restart, no manual rebuild.
>
> **Side discoveries during §144 testing**: (1) Pre-§140 cid_cn collision found in Boss's SourceA test note (title: Hub v6, cid_cn matching Hub v8) — §140's check prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed via delete + recreate. Logged for future sessions: a one-time scrub utility for existing libraries is queued. (2) Unlinked Mentions panel matches frontmatter alias entries — the scanner reads full file content (frontmatter + body) so YAML alias entries (`- "Hub v6"` from rename history) surface as "unlinked mentions". Logged in project memory `project_unlinked_mentions_alias_bleed.md`; pair with the existing `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.
>
> **Boss agenda items added today** (queued, not in scope of any in-flight MIG):
> - Standard OS-style create dialog (greenlit → became MIG-008).
> - One-time scrub utility for pre-§140 cid_cn collisions in existing libraries.
> - Outgoing Links display case fix (`hub v8` → `Hub v8`; cosmetic).
> - Unlinked Mentions / frontmatter alias bleed (project memory above).
> - NSIS bundling lock investigation — recurring `os error 32` when Constellation is running during build; not a tooling bug per se but worth a workaround.

> **What changed in v1.29** (§135 + §136, same calendar day as v1.28):
>
> **§135 — `/simplify` checkpoint over §128-§134** (commit `fe9bf9e`). Three review agents (reuse / quality / efficiency) walked the MIG-006 §3 redo arc with Boss-supplied focus areas. Real-bug fixes shipped: refcounted `cascadingPaths` (Set → `Map<string, number>` so spam-renames in the same library don't pop each other's marks); killed the 1-second magic-timeout settle (orchestrator now `await`s `reloadTabsFromDisk(result.rewritten)` directly — real completion signal, no listener race, no wall-clock penalty on single-file renames); extracted `tabsInLibrary(libraryPath)` helper with separator-bounded prefix check (`/Foo/Bar` no longer falsely matches `/Foo/Bar2`). Efficiency wins: `reloadTabsFromDisk` batched + idempotent (parallel reads, single `openTabs.update`, skips bump when content matches); `watcher_suppress::was_recent` cheap-path lookup with opportunistic 256-threshold sweep (was O(N) `retain` on every watcher event); `CascadeResult.failed` capped at 100 entries with a `failed_truncated: usize` counter (defensive against pathological cascades bloating the IPC payload); consolidated `isCascading` WHY-comments at the three gate sites into one canonical docstring on `isCascading()` itself.
>
> **§142–§144 — MIG-006 §4 closed (write-time index propagation, both Rust + frontend halves)**. Boss surfaced the original gap in §3-redo Stage 1 testing: after rename, Outgoing Links panel kept showing the OLD target name (`foo`, lowercased) — the body cascaded but `note_meta.outgoing_links_json` and `note_links` weren't updated, so panels reading the index served stale data. **§142** plugged the Rust side: `update_links_on_rename` now calls `reindex_single_note` for each rewritten path after the cascade walk, with a new `library_name` parameter on the IPC. SQLite caught up. **§143** attempted a frontend-side targeted update of `allLibraryLinks` (the boot-snapshot `$state` the panels actually read from), but only matched entries where the in-memory `target` equaled the rename's `oldName` exactly — and after several renames in a session (Hub v4 → v5 → … → v8), the in-memory state had drifted further than any single rename's `oldName`, so the targeted match never fired. **§144** superseded §143 with the simpler drift-resistant fix: re-fetch `cache_boot_snapshot_graph` post-cascade and replace `allLibraryLinks` + `notePathToAliases` wholesale. Boss tested PASS on the §144 binary — Outgoing Links panel now updates immediately after rename. Closes the original Stage 1 observation. (§143's targeted update is left in the commit history as an "almost-fix" anchor — useful context for the next person who wonders why we don't do incremental updates.)
>
> **Tab/title corruption discovered + recovered during §144 testing**: a SourceA test file from earlier sessions had `title: Hub v6` AND a duplicate `cid_cn` matching Hub v8's identity — pre-§140 corruption that survived in the disk file. §140's `cid_cn` check prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed by delete + recreate. Post-§140 the bug shouldn't reproduce on fresh notes. Logged for future sessions: existing libraries may carry pre-§140 cid_cn collisions; those need manual recovery (delete + recreate) or a one-time scrub utility.
>
> **Side discovery during §144 testing — Unlinked Mentions panel matches frontmatter alias entries** (logged: `project_unlinked_mentions_alias_bleed.md`). The scanner reads the full file content (frontmatter + body) when looking for the active note's name as a plain-text occurrence, so frontmatter `aliases:` entries surface as "unlinked mentions" of unrelated notes. Should split on the closing `---` fence. Pair with `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.
>
> **§141 — `/simplify` checkpoint over §137-§140**. Three review agents (reuse / quality / efficiency) walked the §137-§140 diff. Real cleanups shipped: **(a)** new `normalizePathKey(p)` exported from `src/lib/utils.ts` — the `(p) => p.replace(/\\/g, '/').toLowerCase()` function was duplicated 7+ times across utils, store, and +layout. Single source of truth so a future filesystem-rule change (case-sensitive volumes, NFC normalisation) is one edit, not eleven. Every path-keyed Map operation now goes through this. **(b)** `WAB_LS_KEY = 'constellation-wab'` constant in store.ts — the localStorage key was hard-coded in five places. **(c)** Single `walkAuxStatePaths` walker shared by `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` — both used to walk the same three structures (in-memory wab, in-memory recentWrites, localStorage wab) with identical norm-and-prefix matching. The walker passes the ORIGINAL key to the decide callback so folder-rename suffix preservation works on case-mixed Windows paths. **(d)** `openNoteTab`'s wab/disk choice extracted to `resolveNoteContent(filePath)` helper — the §140 inline check was three levels deep with three duplicated `clearWriteAhead` calls. The helper returns `{content, cursorPos, scrollTop}`: when wab is stale (cid_cn mismatch), drops the wab cursor/scroll too — they were for the OLD note, a subtle correctness improvement the inline §140 code missed. **(e)** `handleStageChanged(path, stage)` hoisted in +layout.svelte — the 3-line callback was inlined twice (main editor + split/second-screen path). **(f)** `extractCidCn` regex bounded to the first `---…---` frontmatter block — prior code matched against the full content, so a 10MB note made the lazy regex walk the whole body. **(g)** Stripped `// §139:` / `// §140:` inline anchor comments where they narrated what the code obviously does; kept multi-line docstrings on function declarations.
>
> **§140 — Cross-note content corruption via stale `writeAheadBuffer` (Rule 8 + the BUG-015 corruption class)**. Boss reported a **serious data corruption bug**: "Sometimes, when switching between notes after renaming or creating notes, I discover that a note replicates its contents, title, and cid_cn into another note. The victim note keeps its title in the file tree, but when I click it, it shows the culprit note (title, content, and properties)." Investigation pinpointed `writeAheadBuffer` (in-memory `Map<filePath, V>` + `localStorage` backup that survives app restarts). When a note is flushed, the editor's content is stashed under its file path so a later `openNoteTab` can substitute it for a disk read. **`renameItem` / `moveItem` / `deleteItem` migrate `openTabs.path` correctly but never touched the buffer** — so when a path was reused after a rename or delete (trivial with human-named notes: rename Foo → Bar, create new Foo, the new Foo lands at the old `…/Foo.md` path), `openNoteTab` hit the stale buffer entry and loaded the OLD note's content (cid_cn / title / body) into the new tab. The file tree kept showing the new note's title (driven by `display_title` from disk frontmatter — disk was correct) while the tab held the old note's content (in-memory only, until the user typed and triggered a `handleSave` that committed the corruption to disk too). Same Rule 8 / write-time-derivation gap §137 closed for `stageMap` / `maturityMap` — except corruption-class severity. §140 closes it three ways: **(1)** new helpers `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` migrate / drop `writeAheadBuffer` + `recentWrites` entries (in-memory + localStorage backup), with folder-prefix support for folder rename / delete; **(2)** wired into `renameItem`, `moveItem`, `deleteItem`; **(3)** defense-in-depth in `openNoteTab` — when a wab entry hits, also read disk and compare the `cid_cn` signature; on mismatch, prefer disk and clear the stale buffer (handles historical localStorage entries from before §140). Self-healing via (3) for any user with stale localStorage from prior sessions.
>
> **§139 — Three production-binary bugs Boss caught (RTL arrows, recursive FileTree, SvelteMap reactivity)**. Boss installed the §138 production binary and reported three bugs from real-world testing: (1) Promote → / ← demote arrows inverted in RTL note context — the visual reading direction is right-to-left so `→` reads as "backward" in RTL; fix is to swap arrow characters when `dir === 'rtl'`. (2) Folder children in the file tree never receive `stageMap` / `maturityMap` — `<svelte:self>` recursion at `FileTree.svelte:102` was missing those two props from its prop list, so notes inside any folder rendered with default empty maps. (3) Promote/demote and "add Stage via property panel" updated the breadcrumb badge but **not** the file-tree emoji — the chain (handlePromote → onStageChanged → stageMap reassign) looked correct but the file tree didn't re-render. Root cause: the `$state(new Map())` + reassign-to-fresh-Map pattern has a Svelte 5 prop-propagation quirk visible specifically through this child-reads-via-prop path. Fix: switch `stageMap` and `maturityMap` to `SvelteMap` (Svelte 5's explicitly-reactive Map) — mutations are reactive at the operation level, no reassign-to-force-identity needed. Updated all six call sites (enrichNodesBackground, §138 toggleLibrary scans, §137 handleRenameComplete migrations, both onStageChanged callbacks) to use direct `.set()` / `.delete()`. New `migratePathKeyedMapInPlace<V>` helper in `src/lib/utils.ts` for SvelteMap targets in §137. `notePathToAliases` and `searchLinkCounts` stay on the original `$state(new Map())` pattern for now — narrow scope, only the user-visible drift surfaces converted.
>
> **§138 — Stage + maturity load on library expand (deeper Rule 8 fix)**. Boss tested §137 and reported: "the emoji is not visible, not before renaming or after it." The §137 path migration was correct but lit nothing because the upstream `stageMap` and `maturityMap` were both **empty on boot**. Audit found the cause: `enrichNodesBackground` (the only path populating these maps) was deliberately removed from the boot flow for boot-perf — comment at `+layout.svelte:2744-2757` explains "ZERO BOOT-TIME WALKS." Before §138, the only triggers were the Sky View legend's `onRequestEnrichment` button, the Settings → Rebuild Index path, and the first-ever-launch modal. None of those fire on a normal boot, so the file tree never showed stage emojis or maturity dots. §138 adds a third trigger: when the user expands a library in the sidebar (`toggleLibrary`, first-expand only), fire `scan_note_stages` + `compute_note_maturity` for that library and merge results into `stageMap` / `maturityMap`. Fire-and-forget so the expand isn't blocked; maps are reactive `$state` so the file tree re-renders when each scan returns. This respects the boot-perf discipline (no walks on boot) while restoring the Rule 8 expectation (every derived view present at the moment the user looks at it). Mutation guard: the merge only writes a fresh Map when at least one entry actually changed — Svelte doesn't fire spurious reactivity on no-op merges.
>
> **§137 — Rename propagates to path-keyed reactive state (Rule 8 reinforcement)**. Boss observation during Stage 5 testing: "we used to have the stage icon attached to the note title as a prefix — and we want Constellation to do it instantly when the user promotes, demotes, renames, or re-renames. That's why Constellation is unique and has its own prediction engine." Audit revealed: file-tree stage emoji + maturity dot + alias index + search-hub link counts are all `Map<path, V>` reactive `$state` in `+layout.svelte` (`stageMap`, `maturityMap`, `notePathToAliases`, `searchLinkCounts`). Promote/demote already kept them in sync via the `onStageChanged` callback chain; **rename did not**. After a rename, the renamed file's old path stayed in every map as an orphan, and the new path had no entry — so the file-tree showed the renamed note without its stage emoji until the next library scan. Direct violation of Rule 8 (Write-Time Derivation: "every computed view in Constellation is maintained at write time, not read time"). §137 adds `migratePathKeyedMap<V>(map, oldPath, newPath)` in `src/lib/utils.ts` (handles file rename, folder-prefix rename, and no-op canonical-file renames where the disk path stays the same; returns `null` to skip spurious reactivity when nothing migrated) and calls it from `handleRenameComplete` for all four affected maps. The renamed file's stage emoji, maturity dot, and aliases now follow the path the moment the rename lands.
>
> **§136 — Stage breadcrumb redesign + `handlePromote` cascade gate**. Boss observation: the breadcrumb Stage dropdown duplicated the property panel — same control, two surfaces. Homework on commit history showed why: the predecessor commit (`87d21d7`, CE Phase 6) added Stage to the breadcrumb as a one-click `Promote →` *verb* per `docs/CE-spec.md` Phase 6, then commit `6cbe87c` (40 minutes later) silently refactored the verb into a property-selector dropdown. Boss's "not LOGICAL" critique was reading the post-refactor state correctly. §136 restores the verb-distinct design: the breadcrumb now renders `[← demote] [stage badge] [Promote →]` with visual asymmetry — Promote prominent (accent border), demote subdued (faint arrow, no border, tooltip-only label). Demote is permitted (CE-spec one-way line was an oversimplification — knowledge revision is real research practice), but visually subdued to encode the frequency asymmetry. Removal of the stage property entirely stays in the property panel (verbs vs administration). Side fix: `NoteEditor.handlePromote` was the *other* drift surface the §134 audit missed — it bypassed the `isCascading` cascade gate the same way `PropertyEditor.saveTabContent` did. Added the gate at the top of `handlePromote`. Both stage-edit paths (breadcrumb verb + property panel) and both body-edit paths (`handleSave` + `handleFlush`) now share one consistent cascade gate. CE-spec Phase 6 updated to match (the "one-way" line is now historically annotated). i18n: added `notePane.demote` to all 15 locales; `notePane.promote` already existed from CE Phase 6.
>
> Stage 1-4 of the §3 redo Boss test cycle have all PASSED (basic cascade ✓, open-editor coherence ✓ — the headline win, pre-cascade-staleness ✓, multi-source watcher-loop ✓). Stage 5 (PropertyEditor / handlePromote cascade gate verification) and Stage 6 (spam-rename refcount) remain.

> **What changed in v1.28**: MIG-006 §3 redo lands clean (commits §128-§133). After the §115 attempt at §3-expanded ("open-editor coherence") burned BUG-015, MIG-006 §3 sat in `REVERTED` status for a week. Boss directed (via the 360.3D pattern) that a Concept Paper come first; that landed as §127 (`docs/Rename-Function-Concept-Paper-v1.0.md` + `lab/reports/MIG-006-3-REDO-ARCHITECT.md`). The redo itself shipped across §128-§133 as six landable steps + Phase 4 audit closure, all anchored to the eight P1-P8 invariants and Principle D6 (no `$effect` reads/writes value/editBody — that's BUG-015's class).
>
> **The redo (Concept Paper Option A — recreate via `{#key}` bump):**
>
> - **§128 (§3-redo.1)** — `flushAllTabsInLibrary(libraryPath)` helper in `store.ts`. Iterates open tabs in the affected library, writes any in-flight `writeAheadBuffer` content to disk via `writeNote`, marks each path as a recent write so the watcher's external-edit emit skips it. Closes F2-pre-cascade-staleness.
> - **§129 (§3-redo.2)** — new `src-tauri/src/watcher_suppress.rs` module: `mark(path)` / `was_recent(path)` with 2.5 s TTL. Cascade walker calls `mark` before each `fs::write`; the file watcher's emit path filters out recent writes. Closes F3-watcher-loop.
> - **§130 (§3-redo.3)** — `CascadeResult { rewritten, failed }` struct + `cascade:rewrote { paths }` Tauri event. Per Concept Paper D3, the cascade is per-file atomic but not transactional across files; failures collect into `result.failed` rather than rolling back successes.
> - **§131 (§3-redo.4)** — `OpenTab.reloadVersion?: number` field + `reloadTabFromDisk(path)` helper + `cascade:rewrote` listener in `+layout.svelte`. The listener re-reads each affected file from disk, updates `tab.content`, bumps `reloadVersion`. NoteEditor's `{#key}` includes `reloadVersion` so NotePane destroys + remounts with fresh content. Per Principle D6, this is the safe primitive — never an `$effect`-driven `view.dispatch`.
> - **§132 (§3-redo.5)** — `handleRenameComplete` orchestration: markCascading → flushAllTabsInLibrary → updateLinksOnRename → settle → clearCascading. NoteEditor's `handleSave` and `handleFlush` both gate on `isCascading(filePath)` and bail out for the duration. Closes F2-post-cascade-stomp.
> - **§133 (§3-redo.6)** — `/simplify` checkpoint cleanups: path normalisation in `cascadingPaths` Set + `flushAllTabsInLibrary` (Windows backslash vs forward-slash), parallelised `cascade:rewrote` listener (Promise.all), conditional 1 s settle (skip when `result.rewritten.length === 0`), opportunistic full-map GC in `watcher_suppress::was_recent`.
> - **§134 (§3-redo.7) — Phase 4 audit closure (this commit).** Three review agents found two HIGH/MEDIUM drift items shipped as fixes here:
>   - **PropertyEditor bypass (HIGH)** — `PropertyEditor.svelte` calls `saveTabContent` directly when the user edits a frontmatter property. Without an `isCascading` gate inside `saveTabContent`, a property edit during the cascade window would stomp the cascade's wikilink rewrite. Fixed by adding `if (isCascading(filePath)) return` at the top of `saveTabContent`. NoteEditor's gates on `handleSave`/`handleFlush` cover the body-save path; this gate covers the property-save path. Both routes now share the same protection.
>   - **Universe-switch leak (MEDIUM)** — `cascadingPaths` Set entries persisted across Universe switches. New `clearAllCascading()` helper called from `handleUniverseSwitch` so the new Universe starts with a clean slate.
>   - Concurrent renames + typing-during-cascade keystroke loss documented as known limitations; fixes deferred (concurrent renames need a `rename_id` serialization layer; keystroke loss is the input-block step that Concept Paper P4 explicitly accepts as out-of-scope for v1).
>
> **What MIG-006 §3 redo does NOT cover** (queued for §3-redo.8 onward, mapped to the original §4-§11 plan in `MIG-006-WIKILINK-CASCADE.md`):
> - Reindex via `index_note` (P7 — `note_links.target_name` reflecting disk).
> - Sync/async dispatch + progress events (P6 — hub-rename UX).
> - Atomic per-file writes via tempfile (P5 — kill-mid-cascade integrity).
> - Pre-MIG-006 backfill command for stale wikilinks.
> - Phase 4 audit (FULL — per-step audits ran inline; the cross-cutting audit happens at MIG-006 closure).
>
> **Migration table updated**: MIG-006 row now shows §1-§3 ✅ + §4-§11 ⏸.

> **What changed in v1.27**: Inline warning icons in matrix column headers (commit §125). Boss tested §124 on Abu Bakr and reported: "It is easy to identify the blind spot, but not the tensions. Is it in the Causes?" The §124 brown top border on Contradicts was being clipped by the matrix's `border-radius: 12px` + `overflow: hidden`. Boss's fix: "Maybe if we add the warning icons in their place, it will be easier."
>
> **§125 adds the same icon as the corresponding HUD chip directly above the column name** in the column header:
>
> - Blind spot column → ⚠ in red (alongside the existing full-red §122 treatment)
> - Fragile column (Derives From) → ⚠ in yellow
> - Tensions column (Contradicts) → ⚡ in brown (`#8b4513` light theme, `#c89875` dark theme)
>
> The icon is the primary signal; the §124 top border stays as a secondary cue (visible on middle columns even when the rounded corners clip the leftmost / rightmost). Visual continuity from HUD chip to column is direct: see ⚡ at the bottom, find ⚡ at the top of Contradicts.
>
> **No backend change in §125** — frontend template + CSS only.

> **What changed in v1.26**: Per-warning HUD chip colours + matching column-header overlays for fragile and tensions (commit §124). Boss confirmed §122 (red blind-spot column highlighting) on دمشق, then asked: "I want to have the same for the other warnings, like Orphan. But we have to choose a different color for each one."
>
> **Colour assignments**:
> - **Blind spots** (typed columns with 0 connections) — **red** (`var(--text-error)`). Existing §122 treatment; unchanged.
> - **Orphan** (no inbound links) — **orange** (`var(--color-orange)`). HUD chip only — no natural matrix counterpart, since "no one points at me" isn't a column-level signal.
> - **Fragile** (load-bearing on thin foundation) — **yellow** (`var(--color-yellow)`). HUD chip + 3 px yellow top border on the Derives From column header (the column whose under-population is what `single_point_of_failure` measures).
> - **Tensions** (active Contradicts links pointing at this note) — **brown** (Boss directive; brown isn't in the theme palette so hardcoded `#8b4513` for light theme and `#c89875` for dark theme). HUD chip + 3 px brown top border on the Contradicts column header.
>
> **Stacking precedence**: when a column is both a blind-spot and a fragile/tensions overlay candidate, blind-spot wins (red replaces everything). The `tensions-flag` and `fragile-flag` classes are only applied when `!isBlindSpot`. In practice tensions and blind-spot on Contradicts are mutually exclusive (tensions = inbound contradicts, which would make column count > 0); fragile + blind-spot on Derives From overlap only when the note has zero outbound derives-from while still being load-bearing-via-inbound — the red treatment is more important there.
>
> **No backend change in §124** — frontend CSS + classes only.

> **What changed in v1.25**: Stage 3.2 follow-up — blind-spot column highlighting (commit §122). Boss tested S3.2 on note دمشق, confirmed the column-totals row delivers the §4.2 Connection-Profile signal cleanly, then asked: "since the matrix identified the blind spots, it should highlight them within the matrix to help the user undertake the right measures."
>
> **Shipped in §122**: when a typed column's total is 0, the column header gets a warning treatment in addition to its normal type-coding:
>
> - Background gradient swaps from the soft type-colour tint (5%) to a `var(--text-error)`-mixed warning tint (14%).
> - Bottom border switches from the type colour to `var(--text-error)`.
> - The column name and the `0` count both render in `var(--text-error)`.
>
> Untyped is excluded from blind-spot detection — its 0 means "no plain wikilinks", not a typed-direction gap.
>
> Theme-aware via `var(--text-error)` (defined in `theme.css` as `--color-red`). With four-plus blind-spot columns, the visual is intentionally loud — the matrix is telling you which directions of reasoning haven't been declared for this note. The bottom HUD's `⚠ N blind spots` chip stays as a corroborating count.
>
> **No backend change in §122** — frontend CSS-only.

> **What changed in v1.24**: Three §120 retest follow-ups (commit §121). Boss flagged on the Arabic locale: (a) the `Untyped` column header still rendered in English because `typeLabels` derived skipped untyped via `if (lt === 'untyped') continue` — leftover from the §113 hardcode workaround; (b) the stage value `spark` (used in Boss's library) wasn't in the i18n stage map; (c) Arabic stratum-name terminology corrections — Boss's preferred terms: L3 رأي (vs قضية), L7 منظور (vs نموذج), L8 رؤية شاملة (vs رؤية كونية).
>
> **Three fixes shipped in §121**:
>
> 1. **Untyped column localized**. `typeLabels` in `Inspector360.svelte` no longer skips untyped — the loop now treats it uniformly, looking up `inspector360.untyped` (which §120 added to en + ar). With the §120 fallback chain, locales without that key fall through to en. Hardcoded English values stay as the final defensive fallback.
> 2. **`stage_spark` added** to en.json + ar.json. English: "spark"; Arabic: شرارة. Stage values are user-defined free-text (read directly from the YAML frontmatter `stage:` field by `extract_stage()` in `inspector360.rs`), so Boss's library uses lifecycle terminology beyond the four canonical Zettelkasten stages. Other lifecycle terms (birth/growth/maturity/dormancy/renewal) can be added on-demand if encountered.
> 3. **Arabic stratum corrections**: `stratum_name_3` قضية → رأي, `stratum_name_7` نموذج → منظور, `stratum_name_8` رؤية كونية → رؤية شاملة. Updated dependent help strings (`help_stratum_3/7/8`, `help_axis_stratum`, `help_dim_stratum`) to use the new terminology consistently.
>
> **No backend change in §121** — frontend + i18n only.

> **What changed in v1.23**: Three §119 follow-ups bundled (commit §120). Boss flagged on the §119 binary: (a) tooltip text for the dimension-strip `?` icons rendered ALL CAPS — inheriting `text-transform: uppercase` from the parent strip label; (b) tooltips near the right edge of the matrix were clipped because `transform: translate(-50%)` pushed half the tooltip off-screen; (c) "everything fully localized, like the Stratum, and the top row" — non-typed text in the matrix (stratum names, dim labels, maturity/origin/stage values, "Due", "Untyped") still rendered in English even on the Arabic locale, plus the new help text needed translations.
>
> **Three fixes shipped in §120**:
>
> 1. **HelpTip uppercase + edge-clip**. `.help-tooltip` now sets `text-transform: none` to override any uppercase ancestor; `font-weight: 400; letter-spacing: normal` for safety. `computeCoords()` clamps the tooltip's `x` coordinate to viewport bounds (190 px conservative half-width + 12 px margin), so triggers near the left or right edge no longer clip the tooltip.
> 2. **i18n fallback chain**. `t` derived in [`src/lib/i18n/index.ts:108`](src/lib/i18n/index.ts:108) now falls back to `en.json` when the active locale's lookup returns the literal key path (i.e. the key isn't in the active locale). Previously, missing keys in non-en locales returned the key string verbatim — a bug that forced the §104/§113 Untyped-label hardcode. With the fallback chain, missing keys display English instead, and partial translation stays graceful while translators backfill. Loaders cast each non-en locale through `unknown as typeof en` to bypass strict structural matching (the runtime fallback handles missing keys cleanly).
> 3. **Full Arabic + English localization of the matrix**. New i18n keys in `inspector360.*`:
>    - `untyped`, `stratum_name_1..8`, `dim_stratum/maturity/origin/stage/review/trails/lenses` (10)
>    - `maturity_seed/sapling/evergreen/canonical/wilting`, `origin_received/discovered/mixed/none`, `stage_fleeting/literature/permanent/synthesis/none`, `review_due/none` (16)
>    - `axis_stratum_label`, `axis_type_label` (2)
>    - `help_axis_stratum/type`, `help_stratum_1..8`, `help_type_*` (8), `help_dim_*` (7), `help_grand_total`, `help_hud_orphan/fragile/blind_spots/tensions` (4) — total 30 help strings
>    - All keys added to en.json (English source-of-truth) and ar.json (full Arabic translation, native-quality terminology). Other 13 locales fall back to English via the new chain — to be backfilled later.
>
> `Inspector360.svelte` updated: every previously-hardcoded label uses `tr($t(key), key, fallback)` where `tr()` is a small helper that returns the translation when present and the English fallback when `$t` returns the literal key. Static `STRATUM_NAMES`, `HELP_STRATUM`, `HELP_TYPE`, `HELP_DIM`, `HELP_GRAND`, `HELP_HUD`, `HELP_AXIS_*` constants removed; only `STRATUM_FALLBACK` retained as the in-component English fallback.
>
> **No backend change in §120** — frontend + i18n only.

> **What changed in v1.22**: Stage 3.1 follow-up — first-time-user `(?)` help affordances on the 360.3D matrix (commit §119). Boss S3.1 finding: "for the first-time user, we need to help them figure out what this matrix is all about. We need to explain each stratum, type, and/or every bit of detail within the 360.3D. By adding a (?) with each one of those elements."
>
> **Shipped in §119**:
>
> 1. **New reusable component** [`src/lib/components/HelpTip.svelte`](src/lib/components/HelpTip.svelte) — small `?` button that surfaces a styled tooltip on hover, and pins-on-click for accessibility / touch (outside-click dismisses). Tooltip uses `position: fixed` driven by `getBoundingClientRect()` so it escapes overflow boundaries. Theme-aware via `--background-secondary` / `--text-normal` / `--text-accent`.
> 2. **30 help markers wired** across the full-window matrix in [Inspector360.svelte](src/lib/components/Inspector360.svelte). Coverage:
>    - Corner cell: 2 (`▲ Stratum` axis legend, `Type →` axis legend)
>    - Column headers: 8 (one per typed direction + Untyped)
>    - Stratum row labels: 8 (L1 Datum → L8 Worldview)
>    - Dimension strip cells: 5 base + 2 conditional (Stratum, Maturity, Origin, Stage, Review, Trails, Lenses)
>    - Grand total Σ in the corner cell: 1
>    - HUD warnings: 4 (Orphan, Fragile, Blind spots, Tensions)
> 3. **Explanation text** authored as one-paragraph descriptions per element. Stratum text covers what kind of note lives at that altitude. Type text covers what the typed link asserts and shows the wikilink syntax. Dimension text covers the source-of-truth + how it's computed. HUD text covers when the warning fires and what it means cognitively. Axis-legend text in the corner cell explains how to read the matrix overall.
>
> **Compact scorecard untouched** — the sidebar widget is too narrow for `?` icons. First-time learning happens in the full-window matrix; once Boss is fluent, the scorecard reads at a glance.
>
> **No backend change in §119** — frontend-only.

> **What changed in v1.21**: Sky View inspect-mode lockout fix (commit §118). Bug Boss reported on 2026-05-01: in Sky View, click a node → app opens that note as a tab → close that tab via its own × (rather than via the "Return to Sky View" dismiss pill) → app locks; both sidebars refuse to open from their toggle buttons; only recovery is restarting the app.
>
> **Root cause**: clicking a Sky View node calls `handleSkyNodeClick` which (1) snapshots the current sidebar state to `sidebarSnapshots.get('skyInspect')`, (2) hides both sidebars, (3) sets `skyViewInspectMode = true`. The intended exit is a pill rendered next to the active tab — clicking its body returns to Sky View, clicking its `×` dismisses inspect mode and pops the snapshot. **But the pill only renders while `$activeTab?.path` is truthy** ([+layout.svelte:4439](src/routes/+layout.svelte:4439)), and the sidebar toggle handlers are guarded by `!skyViewInspectMode` ([+layout.svelte:1660-1661](src/routes/+layout.svelte:1660)). Closing the tab via its own × clears `$activeTabId` to `null` → pill disappears with the tab → flag stays `true` → toggles refuse to fire. Locked.
>
> **Fix shipped in §118**: a `$effect` in [+layout.svelte:586-590](src/routes/+layout.svelte:586) watches `skyViewInspectMode` and `$activeTabId`. When the tab goes null mid-inspect, it runs the same cleanup the dismiss × button runs — `popSidebars('skyInspect')` to restore the pre-SV sidebar layout, then sets `skyViewInspectMode = false`. Tab-close-via-X now exits inspect mode cleanly. Frontend-only fix; the dismiss pill itself is unchanged for users who use the intended path.

> **What changed in v1.20**: Verification B Check-2 follow-up (commit §117). Boss accepted §115's column-header text colour change but flagged the background tint as still too strong: "lower the tinted background more." §117 reduced the tint from 10 % type-colour mix to 5 %. Text colour and bottom-border colour kept the §115 values. One-liner CSS change.

> **What changed in v1.19**: Verification A retest fixes (commit §116). Boss tested the §115 list-of-titles and surfaced two issues:
>
> 1. **Cell expansion persisted across navigation.** Click a list item → matrix moves to new note → previously-expanded `(stratum, type)` cell stayed expanded on the new note. Boss: "It should collapse by default when we move to another node." Same on back-bar return: "When we are back, it should collapse automatically."
> 2. **Untyped should be expandable too.** Boss originally directed (S1.3.5 in §114) to exclude Untyped because dot-grid expansion at 800+ would balloon the matrix. §115 reworked expansion as a scrollable title list, which contains the size cleanly. Boss: "Let's have the 'untyped' expandable like the other type."
>
> **Fix shipped in §116** (frontend-only):
>
> 1. **Auto-reset on navigation**: a `$effect` watches `data?.note_path` and resets `expandedCells = new Set()` whenever it changes. Covers both forward (title-click → onNoteClick fires → parent updates `data` → effect runs → state clears) and backward (back-bar → onBack restores prior `data` → same path).
> 2. **Untyped exclusion removed** from `toggleCellExpand` and the template branch. The `+N` chip on Untyped is now a clickable button just like the seven typed columns. The list view caps at 240 px with internal scroll regardless of count, so Untyped's typically-large overflow is contained.
>
> **No backend change in §116** — frontend-only.

> **What changed in v1.18**: Stage 1 + Stage 2 retest follow-up bump (commit §115) — six refinements bundled into one rebuild after Boss walked all 6 + 6 sub-stages of the matrix tutorial.
>
> **Six fixes shipped in §115** (frontend-only):
>
> 1. **Expanded typed-cell renders as a list of note titles, not more dots.** S1.3.5 surfaced this: when the user clicked `+N` on a typed cell, §114's design just showed all the hidden dots — visually overwhelming for cells with 30+ connections, and the user still had to hover each dot to learn the name. New design: clicking `+N` switches the cell into a **vertical list of note titles**, each clickable to navigate. Dot bullet shows the type colour beside each name.
> 2. **Always-visible `×` collapse button** at the top-right of the expanded list. Replaces §114's `−` button which was at the *end* of the dots and easy to miss when the cell scrolled. Now positioned absolutely so it stays visible regardless of scroll.
> 3. **Max-height + internal scroll** (240 px) on the expanded list so very large typed cells (e.g. Abu Bakr's L7-Supports with 49 connections) don't balloon the row past the canvas. List scrolls inside the cell.
> 4. **Active-note name chip removed** from the row label. The note's name is already visible in the matrix header at the top; repeating it on the active stratum row was redundant. Active row is still highlighted in the theme accent (purple band + accented row number) — that signal is preserved.
> 5. **Column-header text contrast.** §113's gradient used 22 % type-colour tint with text in the same hue, which read as colour-on-same-colour. Reduced tint to 10 % and switched text colour to `color-mix(var(--col-color) 55 %, var(--text-normal))` so text stays type-coded but lifts off the background. Bottom border keeps the full-strength type colour for the visual signal.
> 6. **Grand total visible** in the top-right corner cell (the row-totals header). New layout stacks `Σ` symbol over the matrix-wide grand total of all (deduped per cell) connections. Confirms at a glance that column-totals sum equals row-totals sum equals this number.
>
> **No backend change in §115** — frontend-only. The §112 backend (`stratum: u8` on `LinkedNote` + `precompute_all_strata`) stays as-is.

> **What changed in v1.17**: a Stage 1.2 retest fix bump (commit §114). The §113 "2× sizes" directive overshot for the full-window matrix — Boss confirmed S1.1 (compact scorecard) but flagged S1.2 (full-window matrix) with two findings: "Minimize by 1" (sizes too big) and "L1 missing, L2 cut" (the bottom of the matrix was clipped by `overflow: hidden` because 8 rows × 110 px row-min exceeded the canvas height).
>
> **Fix shipped in §114** (frontend-only, full-window only — compact scorecard untouched):
>
> 1. **Full-window matrix scaled down ~25 %.** `360.3D` label 32 px → 24 px, brain icon 56 px → 40 px, header name 44 px → 32 px, strip label 22 px → 16 px, strip value 30 px → 22 px, column name 18 px → 14 px, column count 26 px → 20 px, row num 26 px → 20 px, row name 24 px → 18 px, active chip 20 px → 15 px, HUD font 28 px → 21 px, dot 16 px → 13 px. Padding tightened to match.
> 2. **Cell row min reduced from 110 px → 78 px** (and column min 120 px → 96 px, row-label column 280 px → 220 px, row-total column 100 px → 76 px). All 8 stratum rows now fit in a typical 1080p viewport without clipping.
> 3. **`min-height: 0`** on `.i360-matrix-wrap` so the matrix can shrink in tight viewports rather than getting clipped.
>
> **Compact scorecard unchanged**: Boss explicitly passed S1.1 at the §113 sizes (1.85rem name, 1.4rem pills, 14 px bar height), so those stayed.

> **What changed in v1.16**: a Stage-1-tutorial fix bump for the §112 Stratification Matrix (commit §113). Boss walked S1.1 → S1.6 in sequence and recorded seven refinements; all of them landed in one rebuild rather than commit-per-fix.
>
> **Fixes shipped in §113**:
>
> 1. **`Untyped` label hardcoded** in both the compact bar chart label and the matrix column header. The §104 fix had been preserved across the spherical line until §112 reverted it, and the i18n-key leak (`inspector360.unty…`) returned in Stage 1.1. The fix is the same as §104's: `$t('inspector360.untyped')` returns the literal key string when the translation is missing, which is truthy, so the OR fallback never fires; hardcode `'Untyped'` for that one type, keep `$t()` for the seven typed directions where the keys exist in en.json.
> 2. **Compact bars switched from max-normalised to percent-of-total.** Boss's "Abu Bakr" test note had Untyped=6,107 vs Supports=101 — max normalisation collapsed every typed bar to ~1.6% width and made them invisible. Each bar now fills its share of total connections and the right-hand number reads `X.X%` (or `—` for zero). The shape of the share, not the absolute count, carries the cognitive signal.
> 3. **Compact scorecard text and figures roughly doubled.** Card name 0.95rem → 1.85rem, pills 0.72rem → 1.4rem, bar height 8 px → 14 px, label column 90 px → 130 px, count column 28 px → 60 px to fit `100.0%`.
> 4. **Full-window background and chrome are now theme-aware.** Hardcoded `#060612` / `#0a0a1c` / `#060614` and `rgba(255,255,255,0.X)` greys replaced with `var(--background-primary)`, `var(--background-primary-alt)`, `var(--background-secondary)`, `var(--text-normal)`, `var(--text-muted)`, `var(--text-faint)`, `var(--text-accent)`, `var(--background-modifier-border)`. Active-row purple now derives from `--text-accent` via `color-mix`, so it follows the theme accent instead of locking to a single hex.
> 5. **Full-window `360.3D` header label doubled** (16 px → 32 px). Brain icon 28 px → 56 px. Active-note name 26 px → 44 px.
> 6. **Full-window matrix text and figures doubled.** Strip labels 11 px → 22 px, strip values 16 px → 30 px. Column headers 10 px → 18 px. Column counts 14 px → 26 px. Row labels 13 px → 24-26 px. Active chip 11 px → 20 px. HUD text 16 px → 28 px. Dot size 11 px → 16 px (subset; doubling fully would break 16-dot density per cell). Cell row height 72 px → 110 px. Row-label column 200 px → 280 px; row-total column 64 px → 100 px; column min 80 px → 120 px.
> 7. **Hover label moved from the fixed top-right of the matrix to a floating tooltip that sits directly above the hovered dot.** The previous placement (which I'd justified as "doesn't follow mouse, doesn't pop chrome on dense rows") forced the user to look away from the dot they were hovering. New placement uses `position: fixed` driven by the dot's `getBoundingClientRect()` so it escapes `overflow: hidden` on the matrix and works regardless of cell layout.
>
> **No backend change in §113** — frontend-only. The §112 backend (`stratum: u8` on `LinkedNote` + `precompute_all_strata`) stays as-is.
>
> **Process note**: I bundled S1.1 through S1.6 into one tutorial message and Boss flagged the staging violation early. The remaining sub-stages were sent one at a time (S1.2 alone, then S1.3, then S1.4, etc.). `feedback_staged_tests.md` interpreted strictly going forward — one focused test per turn, never a numbered list of tests in a single message.

> **What changed in v1.15**: the 360.3D Inspector redesign lands as code (commit §112). The concept paper (v1.0) was approved; the clean-slate redesign is the **Stratification Matrix**.
>
> **The matrix in one sentence**: an 8 × 8 grid where the **vertical axis is stratum** (L8 Worldview at the top → L1 Datum at the bottom) and the **horizontal axis is link direction** (the 7 typed directions + Untyped). Each connected note becomes a small dot in the cell at the intersection of its own stratum and the typed direction it shares with the active note. The active note's row is highlighted; **empty cells are visually present** (diagonal stripes) so absence reads as readily as presence — Concept Paper §4.3 "Absence is first-class."
>
> **Why this is the right shape (vs spheres / sectors)**: stratum is the dimension Constellation alone measures, and the matrix puts it on the dominant visual axis (vertical position = altitude in the knowledge hierarchy). Typed direction now has its own dedicated lane instead of competing with stratum on a polar layout. Counts read at a glance: column totals tell you which directions you over- or under-use; row totals tell you which strata your thinking spans. Gaps (empty rows = strata you haven't reached; empty cells = directions you don't use at this stratum) are part of the geometry, not afterthoughts.
>
> **Backend addition** ([`inspector360.rs`](src-tauri/src/inspector360.rs)): `LinkedNote` now carries `stratum: u8`. A new `precompute_all_strata()` helper computes every note's stratum once at the top of `get_360_view`, building an inbound-count + sources-of map up front so each `LinkedNote` can be stamped in O(1). Total cost stays O(N + total_links) — same big-O as before. The same rule set used for the active note (`compute_stratum_for_note`) is reused for connections.
>
> **Frontend rewrite** ([`Inspector360.svelte`](src/lib/components/Inspector360.svelte)): the spherical line — `SECTOR_MAP`, `polarToXY`, the three viz-mode toggle (Atmospheric / Neural / Cosmic), `ringsLayout`, `layoutMode`, `allNodes`, `vizMode` — is gone. Full-window mode is the matrix on an HTML/CSS grid (no SVG polar coordinates). Compact sidebar is now a **scorecard**: note name + stratum pill + maturity pill + ↑outbound/↓inbound/word counts + a per-type bar chart with explicit "—" markers for blind spots + a flags row. The matrix is too dense for a 280 px-wide sidebar; the scorecard is the right read at that scale.
>
> **Preserved from §107 / §109**: hover-only labels (no always-on names cluttering pattern reading), per-render `uniqueId` keying so empty-path collisions don't multi-highlight, multi-hop back-stack for click-to-navigate. Universe switch still resets the back-stack to `[]`.
>
> **Dropped permanently**: `vizMode` dropdown, polar / angular layout primitives, `SECTOR_THRESHOLD` hybrid logic, depth-based ring assignment, count-based ring assignment. The §110 binary (the previous "final iteration" of the spherical line) is no longer the latest runnable Inspector — the §112 binary is.

**Author of facts: Eisa ALSHAMSI (project owner, designer, IT Boss).**
**Maintainer: Claude (consultant / engineer / SME).**

---

## 0. How to use this document

**This is the first document any new Claude session reads.** It exists so a fresh AI can get to architectural fluency in one read instead of rediscovering the project from `git log` + screenshots over several frustrating turns.

**Maintenance is a Standing Order** (`CLAUDE.md` Standing Order #6). Whenever a fact below changes — a phase ships, a rule is added, a doc-drift item is fixed, a migration closes — update this file in the same commit that lands the change. Bump the version when the structure changes; date-stamp every section that updates. **The filename always carries its version suffix**: `Constellation Orientation & Onboarding v1.0.md`, `... v1.1.md`, `... v1.2.md`, etc. **Each new version is written as a NEW file alongside the existing ones — older versions are NEVER deleted.** They remain in `docs/` as a historical record the project owner uses to track how orientation evolved. A new session reads only the highest-version file, but the trail behind it is durable.

**This document is grounded.** Every claim cites the authoritative source (file:line, commit hash, or session log section). When two project documents disagree, I name both and don't pick a winner unless code-reading resolves it. When I don't know something, I say so explicitly in §17.

**Hard rule for every reader (human or AI) of this file**: if you find this document contradicts the actual codebase or a more recent session log, **trust the code and the session log first**, then update this file in the same session.

### v1.14 changelog (vs v1.13)

v1.14 was a clean-slate reset for the 360.3D Inspector (commit §111) on 2026-04-30. After five attempts (§104, §106, §107, §109, §110) at the spherical / orbital / compass-position layout — exceeding LL-014's three-attempts rule — Boss invoked the rule and directed a return to first principles.

Two artefacts shipped in §111 (no code change):

1. **Concept Paper v1.0** — `docs/360.3D-Concept-Paper-v1.0.md`. Defines what 360.3D is, why it exists, what cognitive dimensions it encodes, the three outputs the user should leave with (Position / Connection Profile / Absence), the eight design principles any 360.3D visualisation must satisfy, and what 360.3D is NOT (vs Sky View, Map, Sight, Index, OrgChart). Recommended starting axis: **stratum**.

2. **Orientation v1.14** — captured the reset and the pending clean-slate redesign.

The redesign itself shipped in §112 — see v1.15 above.

### v1.13 changelog (vs v1.12)

v1.13 was a sector-layout fix (commit §110) on 2026-04-30. The §109 depth-based rings didn't help "1902"-class data because `inspector360.rs::get_360_view` stamps every outbound and inbound link with `depth = 1`. §110 replaced depth-based with count-based ring assignment: typed groups sorted by count, distributed across the inner two rings (smallest typed → inner 160, largest typed → middle 270); untyped always on the outer ring 380. Three reliably distinct rings, no typed/untyped collision. **§110 is the final iteration of the spherical layout line — see v1.14 for the clean-slate reset.**

### v1.12 changelog (vs v1.11)

v1.12 was a sector-layout course-correction (commit §109) on 2026-04-30. **Restored depth-based sector rings** `[160, 270, 380]` (matching the compact widget). Each typed group's nodes cluster at their SECTOR_MAP compass angle with the widget's 8°-per-node spread; ring radius determined by note depth. **The §109 fix was insufficient for "1902"-class data** because the IPC always stamps typed links with depth=1, so every typed node piled onto the inner ring 160 and untyped depth-1 collided with them. §110 (v1.13) corrected this with count-based ring assignment.

### v1.11 changelog (vs v1.10)

v1.11 was a Stage 2B retest follow-up (commit §107) on 2026-04-30. Boss reported two findings on the v1.10 binary.

Two changes in §107:

1. **Single-ring sector layout** (interpreting "Distribute all nodes in one circle"): replaced §106's three depth-based rings with a single ring at `SECTOR_RADIUS = 290`. **This was an over-correction; §109 restored depth-based rings.**
2. **Hover label leak fix**: each rendered node now carries a `uniqueId`; hover state renamed `hoveredNode → hoveredId` keying on it instead of `node.path`. Fixes the empty-path collision (`inspector360.rs::get_360_view` returns `path: ""` for outbound links to notes outside the library). **This fix is preserved post-§109.**

### v1.10 changelog (vs v1.9)

v1.10 was a tuning bump for the Stage 2B sector layout (commit §106) on 2026-04-30. Boss reported during Stage 2B retest that the §104 sector mode rendered the test note "1902" too sparsely on the full-window canvas. Boss directive: "It has to be similar to the widget."

Two changes in §106:

1. **Sector spread formula switched** from §100's normalised cap to **the compact widget's exact formula** `(i - (n-1)/2) * 8`. Trade-off: large sectors bleed past their 50° semantic slot into adjacent compass directions. The widget shows this; Boss accepted.
2. **`SECTOR_THRESHOLD` raised** from 8 → **30**. Notes with up to 30 typed-link connections per group now use sector layout; Abu Bakr-class hubs still trigger ring-per-group.

### v1.9 changelog (vs v1.8)

v1.9 was a **CE Phase 12 hardening / refinement bump** (commits §96–§104, ten commits since v1.8 closed) on 2026-04-30. Phase 12 became user-testable on 2026-04-29; Boss tutorial-tested it across Stage 1 and Stage 2 over two days, and every iteration rolled into a fix-and-rebuild loop. Net result: the 360° Inspector surface that v1.8 announced as "enabled" is now the surface the Boss is actually using.

Highlights:

1. **Stage 1 hotfix (§96)** — clicking the new right-sidebar 360° tab routed the user back to Properties because a safety `$effect` (`+layout.svelte:1255`) was force-resetting `rightSidebarTab` to the first known visible tab. The `tabVisible` map and fallback `order` array missed `inspector360`. Fixed; tab now sticks.
2. **rs-tabs strip overflow fix (§97)** — adding the 11th tab pushed past the default 340 px sidebar width; the new tab clipped at the right edge. Pure CSS: replaced default `<button>` padding with explicit `padding: 0; flex: 1 1 28px; min-width: 24px; flex-wrap: wrap;`. Tabs now wrap to a second row instead of clipping.
3. **Compact-mode back-nav (§98 → §99)** — Boss requested a "back to source note" affordance inside the compact widget. Started as single-step (§98) then upgraded to a **multi-hop stack** (§99) per Boss directive: walks all the way back through any chain. State: `inspector360BackStack: $state<Array<{path, name}>>`. Universe switch resets the stack to `[]`.
4. **Stage 2 omnibus (§100)** — five Stage 2 findings: dock-button tooltip i18n leak (`ribbon.inspector360` key returned verbatim because `$t()` returns the key on miss); viz didn't fill canvas (removed `max-width: 1400px; max-height: 900px;` from `.i360-viz`); side panels + HUD doubled in size; tighter sector grouping `(i / (n-1) - 0.5) * 50`; full-window auto-close removed in favour of "Return to {previous}" header button.
5. **Sector → ring-per-group → hybrid (§101 → §102 → §104)** — three iterations on visualisation layout. §104 made the choice automatic: sector layout when max typed-group count ≤ `SECTOR_THRESHOLD = 8`, ring-per-group when above.
6. **Minimised nodes + hover-only labels (§103)** — node radii reduced 10/7/4 → 6/4/3. Always-on labels removed; hover-only with 13 px font + 3 px black SVG stroke. 6 px invisible hit-area expansion.
7. **Dedupe by path + Untyped label fix (§104)** — frontend dedup per-group in `ringsLayout` (the IPC returns the same note from outbound + inbound + second-order). Untyped label hardcoded `'Untyped'` to skip the broken i18n fallback.

**Boss's perf verdict on Phase 12**: first-fetch "almost instantly". **MIG-010 priority dropped to LOW** based on lived experience.

**Process violations recorded for the day**: (a) the over-long Stage 2 tutorial bundled 2.1–2.7 in one message — `feedback_staged_tests.md` rule. (b) Standing Order #6 violation: §96–§104 shipped without bumping the orientation in the same commit. **v1.9 was the catch-up bump.**

### v1.8 changelog (vs v1.7)

v1.8 captured three landings on 2026-04-29:

1. **MIG-003 integrated to main** via fast-forward of `claude/frosty-stonebraker-75c9bf` (the side branch that closed MIG-003 on 2026-04-28 but was never merged). `origin/main` moved from `6545b3e` (MIG-008/009 tip) to `8cb80ac` (MIG-003 handover). Three byte-identical "stranded" closure docs in main's working tree (the v1.7 file, SESSION-LOG-2026-04-28.md §85–§89, CANONICAL-FILENAME-ARCHITECTURE.md updates) became tracked. Source ↔ binary parity restored at main by copying the post-MIG-003 release artifacts from the frosty worktree.
2. **CE Phase 12 360° Inspector re-enabled** (§93 + §94 + §95). Backend `get_360_view` IPC was already shipped from earlier work; only the import + UI wiring was gated at `+layout.svelte:84`. Re-enable shipped both surfaces: a compact right-sidebar tab and a full-window overlay reachable from a new ribbon-dock button. IPC fetch debounced 200 ms with sequence-guard + last-fetched-key dedup; lazy-mount via `inspector360EverOpened`. The `get_360_view` IPC walks the full library on every call (acknowledged Rule-8 violation); MIG-010-scale work to cache `note_360_view` was queued, contingent on Boss's perf verdict.
3. **CE Phase 9 Multi-Lens approved for re-wire on Path B** (Rule-8 compliant) — queued after MIG-006 §3 redo. `lenses.rs::apply_lens` stays dead until that future MIG-010-scale migration.

### v1.7 changelog (vs v1.6)

v1.7 captured MIG-003 closure (Human-name Filenames) on the side branch `claude/frosty-stonebraker-75c9bf`. § 6 fully rewritten to reflect the inverted architecture: `cid_cn` is the immutable internal id (frontmatter only), filenames are human-readable. § 8 migration table updated to mark MIG-003 closed. The Canonical Filename Architecture design doc was given a Post-MIG-003 historical banner. Visible behavior change: every `.md` file on disk now has a human title as its filename; renames cascade through every dependent table (`note_meta`, `note_links`, `sky_nodes`, `note_aliases`, `note_embeddings`).

**Important context for any reader of v1.7**: at the time v1.7 was written, the seven MIG-003 commits + this v1.7 file itself + the closure session-log entries + the CANONICAL-FILENAME-ARCHITECTURE.md updates **only existed on the `claude/frosty-stonebraker-75c9bf` branch and as uncommitted/untracked files in `main`'s working tree**. They were not on `origin/main`. The stranded state was discovered and resolved at the start of the 2026-04-29 session via `git merge --ff-only` (see v1.8 note above). v1.7's "MIG-003 closed" claim was correct — but only on the side branch; the main-line integration arrived a day later.

### v1.6 changelog (vs v1.5)

v1.6 captures two cleanup migrations shipped on 2026-04-27 / 28:

**MIG-008 — Canonical Naming Cleanup** ✅ closed.

- Added shared helper `note_display_name(path, content_opt)` in [`libraries.rs`](src-tauri/src/libraries.rs) — smart enough to skip the file read for human-named files (file_stem IS the title) and only pay the I/O cost for canonical-named files.
- Patched ~14 sites across `map.rs`, `inspector360.rs`, `strata.rs`, `maturity.rs`, `provenance.rs`, `review.rs`, `lenses.rs`, `tasks.rs`, `tension.rs`, `libraries.rs::scan_index_words_recursive`, `trails.rs::find_note_recursive`, `universe.rs::collect_templates_recursive` — all switched from `path.file_stem()` to the helper so user-visible labels show frontmatter title instead of canonical filenames.
- Two of those changes are **correctness fixes**, not just label fixes: `inspector360.rs:88` (now matches incoming wikilinks for canonical notes) and `trails.rs::find_note_recursive` (canonical notes were unfindable by name lookup).
- User-verified across Stages 1, 3, 4a/4b, 5 (Constellation Map, Strata + Maturity + Provenance, Tasks, Review Pulse, Tension via Health). Stages 2 (Inspector 360) and 4c (Multi-Lens) skipped — surfaces are deliberately disabled or dead in current builds (see below).
- Phase 4 audit clean: invariant check / drift check / migration-path check all PASS.

**MIG-009 — Lens-to-Sight Naming Cleanup** ✅ closed.

- Renamed `src-tauri/src/lens.rs` → `src-tauri/src/sight.rs` to align the analytics module's filename with its UI surface (Constellation Sight, formerly Constellation Lens).
- Renamed Tauri commands: `constellation_lens_centrality` → `constellation_sight_centrality`, `constellation_lens_tag_edges` → `constellation_sight_tag_edges`. Frontend `+layout.svelte:3235` invoke updated atomically.
- Frontend JS variable names (`lensActive`, `toggleLens`, `lensCentrality`, `lensCommunities`, `lensCommunityAssignments`, `lensGaps`, `lensHealth`, `lensLoading`, `lensDataStale`, `availableLenses`, `activeLensId` — ~60 occurrences) intentionally **not** renamed; deferred as bookkeeping with no architectural payoff.
- `src-tauri/src/lenses.rs` (plural — CE Phase 9 Multi-Lens) **NOT renamed** — separate concern, deferred to whenever CE Phase 9 is resumed (see "dead-code finding" below).
- User-verified: Constellation Sight still renders centrality + community + gaps after rebuild.

**Dead-code finding** (catalogued, not fixed in this bump):

- `lenses.rs::apply_lens` has **zero frontend callers**. Verified by exhaustive grep on 2026-04-27. The Settings UI can still create + save lens definitions via `list_lenses` / `save_lenses`, but those definitions are never applied to anything. The Multi-Lens (CE Phase 9) IPC pipeline is dead-on-arrival.
- Decision deferred: either delete `lenses.rs` + the Settings lens-definition UI, or re-wire `apply_lens` into a real surface (Sight or a separate panel). Tracked in `project_lenses_apply_lens_dead_code.md` memory.
- MIG-008's patches to `lenses.rs::scan_property_recursive` and `scan_tags_lens_recursive` ship harmlessly but don't run today. Don't revert; the code is correct should the wiring be restored.

**UI / surface notes locked into memory this session:**

- Constellation Lens / Multi-Lens UI surface was renamed to **Constellation Sight** earlier (`feedback_lens_renamed_to_sight.md`). Internal Rust file was just renamed to match (MIG-009).
- 360° Inspector frontend component is deliberately disabled at [`+layout.svelte:84`](src/routes/+layout.svelte:84) — Rust backend (`inspector360.rs`) ships ready, but no UI surface mounts it today.

**New backlog items**:

- Decide fate of CE Phase 9 Multi-Lens (delete vs re-wire). Tracked.
- Decide fate of CE Phase 12 360° Inspector (re-enable vs withdraw).
- `docs/IPC-CONTRACT.md` is now even staler — missing the `constellation_sight_*` rename. Doc-drift item.

### v1.5 changelog (vs v1.4)

v1.5 is a focused-fix bump for the Unlinked Mentions panel (item 6 from the option-(e) backlog). User-verified 2026-04-27 ~18:00.

**§90 — Unlinked Mentions panel: scanner fix + frontmatter-title label**

Two bugs in `scan_unlinked_mentions` ([`libraries.rs:1665-1759`](src-tauri/src/libraries.rs:1665)) closed in one commit:

1. **Scanner false-positive on typed/aliased wikilinks.** The previous "skip source if `[[NoteName]]` substring is present" check was too narrow — every typed-link form `[[NoteName|supports]]`, every alias form `[[OldTitle]]`, and every embed `![[NoteName]]` slipped past it. The active note's title would then be matched as plain text *inside the wikilink markup* and counted as an unlinked mention. Fix: strip ALL wikilinks (regular + embed forms) from content before plain-text scanning. The regex `!?\[\[[^\]]*\]\]` removes them all in one pass.
2. **Source-row label was canonical filename, not human title.** Filename for canonical notes (`20260426T140940Z_NOTE_11B4`) is unreadable; users couldn't tell which note was being shown. Fix: prefer `extract_frontmatter_title()` (already used by the rename path), fall back to `path.file_stem()` only when title is missing.

**Side benefit.** Both fixes are upstream in Rust, so any future caller of `scan_unlinked_mentions` automatically gets correct behavior. No frontend changes needed; the existing `BacklinksPanel.svelte` Unlinked-Mentions section renders the corrected data unmodified.

**What this closes from §12 / §13 / backlog**:
- Item 6 (Unlinked Mentions double-count + canonical filename label) — both bugs fixed.
- The "(e) didn't fully cover item 6" gap I owned in v1.4 — now closed.

**Open items still in the backlog** (unchanged from v1.4 plus the snapshot-path mystery and second-screen alias):
- MIG-007 — Links Settings tab consolidation.
- Constellation Map: tooltip canonical-filename + search highlight + suspected memory leak (the canonical-filename label fix in §90 does NOT propagate to the Map — Map uses a different code path; that's still pending in `project_constellation_map_backlog.md`).
- SecondScreenPage.svelte buildSkyData calls still alias-blind.
- Architectural mystery: why is `cache_boot_snapshot_sky` bypassed at boot in builds that contain MIG-001/MIG-004 §8.

### v1.4 changelog (vs v1.3)

v1.4 captures the 2026-04-27 work session: MIG-005 Tutorial #1 testing, the Sky View edge regression fix (§88), the panel-dedupe fix (§89), and a basket of new backlog items the testing surfaced.

**Architecture / fixes shipped:**

- **§88** — `buildSkyData` fallback now alias-aware. The legacy graph-population path that runs when `cache_boot_snapshot_sky` is bypassed had no alias resolution; renamed-target wikilinks were silently dropped, leaving renamed notes as bubble-without-edges in Sky View. Fix at [`store.ts`](src/lib/libraries/store.ts) buildSkyData now accepts an optional `notePathToAliases` map and applies the same 3-tier resolution as `cache.rs::read_sky_links_raw`. User-verified.
- **§89** — Backlinks / Outgoing Links panel dedupe. A source note with both `[[Note]]` (regular) and `[[Note|supports]]` (typed) targeting the same active note used to render twice — once with no badge, once with the type badge. Now grouped by source path (Backlinks) / target name (Outgoing) into ONE row carrying a `linkTypes[]` array of all distinct typed-link badges. Helper `dedupeBySource` in `store.ts`. Same change includes annotation-redundancy suppression: when a typed-link annotation IS the typed-link keyword (e.g. `[[Note|supports]]` stores "supports" in both slots), the redundant italic prose underneath the badge is now suppressed.
- **Badge taxonomy update**: **M = Mutual link** confirmed by project owner 2026-04-27. Moved out of Unresolved into the link-relationship table in `Badge-Taxonomy.md`. **No more pending badge letters.** §13.1 here updated to match.

**New backlog items surfaced this session:**

- **Auto-update Links toggle is misplaced** under "Sky View & Links". Decision 2026-04-27: a new "Links" Settings tab will consolidate every link-related control. Will be **MIG-007** when greenlit. *(Reverses the v1.2 §12 entry that wrongly "corrected" v1.0's right call.)*
- **Constellation Map UX bugs**: tooltips show canonical filename instead of human title; search doesn't highlight matched arc; suspected memory leak / slowness. All filed in `project_constellation_map_backlog.md`.
- **Unlinked Mentions panel** double-counts wikilink occurrences as unlinked mentions (the scanner doesn't strip wikilink syntax before matching) AND shows source label as canonical filename instead of human title.
- **SecondScreenPage.svelte buildSkyData calls** still use the 2-arg form (alias-blind). Same rename-drops-edges symptom there until threaded.
- **Architectural mystery**: even with MIG-005/MIG-004 §8 in the binary, the alias-aware sky snapshot path (`cache_boot_snapshot_sky`) appears to be bypassed at boot — the legacy `buildSkyData` runs instead. The §88 defensive fix neutralizes user-visible impact, but the underlying "why" is unresolved. Filed for follow-up forensics.

**New top-principal rules / Standing Orders saved this session:**

- **Standing Order — staged tests**: split test tutorials into stages. Send Stage 1, wait for findings, then Stage 2. Never dump 6 tests at once. (Memory: `feedback_staged_tests.md`.)
- **Stage 0 — verify the running binary's mtime** before any test tutorial. The user runs an installed `.exe`, not the source on disk — confirm the binary contains the feature being tested. (Memory: `feedback_verify_binary_before_testing.md`. Earned by the 2026-04-27 incident where I burned hours testing against a binary that pre-dated the feature.)
- **Sky View vs Constellation Map vocabulary** — Sky View has bubbles (PIXI nodes); Constellation Map has sunburst arcs (D3). NOT interchangeable. Same correction had to be made twice. (Memory: `feedback_skyview_vs_map_vocabulary.md`.)

**§17 unknowns reduced:**

- **M = Mutual link** — resolved (see above). Removed from §17.
- Sidebar active-item highlight ~10 s lag — still unresolved.
- 2026-04-16 untracked-backup vs tracked log diff — still unresolved.

### v1.3 changelog (vs v1.2)

v1.3 is a focused correction round driven by [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md), the canonical badge reference dated 2026-04-15 (predates v1.0). I missed it on every prior orientation pass. Corrections folded in:

- **§13.1** badge table rewritten:
  - **W** = Wikilink (`[[target]]`), grey `#94a3b8` — was "unresolved" in v1.2.
  - **LT** = Link **Target** (this note links *to* the queried note), green — was "Link Type" in v1.2 (wrong).
  - **G** = deprecated, superseded by **#** — added to the table for posterity.
  - The badge set ships in **two** components per the source-of-truth invariant: [`ConstellationMap.svelte:80-84`](src/lib/components/ConstellationMap.svelte:80) **and** [`ConstellationSight2.svelte:79-83`](src/lib/components/ConstellationSight2.svelte:79). Both must agree letter→color.
  - Semantic clarification: badges indicate **where in the note the search query matched** (or what link relationship), not arbitrary note categories.
- **§14** "Where to read what" — new row pointing to `docs/Badge-Taxonomy.md`.
- **§17** unknowns — **W removed** (now resolved). M still pending owner clarification.

### v1.2 changelog (vs v1.1)

v1.2 closes the §17 unread list. Significant additions:

- **§3.2** corrected: `+layout.svelte` reactive declarations are now **155 $state, 29 $effect, 1 $derived** (was 77/17/19 in LL-002 / 2026-03-27 — file has roughly doubled).
- **§3.3** corrected: 32 Rust modules; ~120 commands.
- **§3.5** (NEW): full Rust module sizes — `search.rs` 4790, `libraries.rs` 3978, `universe.rs` 1472, `canonical.rs` 1401, `cache.rs` 824.
- **§4.2** enriched per-phase with the Rust file path, the actual aggregator details for Phase 12, and corrected Phase 9 lenses status.
- **§5** Arabic Engine: confirmed mmap is wired through ([`fst_bake.rs:323`](src-tauri/src/arabic/fst_bake.rs:323)), via `Arc<Mmap>` shared by both stripped + folded FSTs.
- **§5.5** (NEW): ai/, embeds/, embeddings/, tasks/, lens.rs (Brandes betweenness), inspector360.rs, sky_backfill.rs (BATCH_SIZE=1000, INTER_BATCH_SLEEP_MS=50), boot_bundle.rs.
- **§7.1** editor stack now described per-plugin from full reads. Added the LL-014 RULE A / RULE B in `calloutPlugin.ts`.
- **§7.4** (NEW): `store.ts` write-ahead buffer (memory + localStorage), navigation supersede tokens, `recentWrites` 2 s gate, save coalescing.
- **§7.5** (NEW): `secondScreen.ts` event API (12 main→screen, 4 screen→main, 1 bidirectional).
- **§9.3** (NEW): boot-bundle (10 IPCs → 1 round-trip) for early-boot data.
- **§11** LL list now grounded in verbatim text.
- **§12** drift list refreshed: `autoUpdateLinks` toggle is **correctly under "Sky View & Links"** (v1.0 misclaimed it as misplaced); `IPC-CONTRACT.md` still 4 weeks stale.
- **§13** badge taxonomy resolved: **T/C/P/S confirmed**; **#, ∅, W, M and LT/LF/⇄/LB/LA also defined** in `ConstellationMap.svelte:80-84`. **W and M letter meanings remain unresolved** (no doc found; honest).
- **§13** auto-update-links toggle confirmed at Settings → **Sky View & Links** (not "Files" as v1.0 wrongly suggested).
- **§14** corrected `lib.rs:233-432` line range.
- **§15.3** (NEW): collision tiebreak — name wins over alias; identical-alias multi-target is **first-write-wins, undefined order**.
- **§17** dramatically reduced — every Rust module read; every CM6 plugin read; every major Svelte component surveyed; `store.ts`, `secondScreen.ts`, `universe/store.ts` read; user manual + 24 help topics + BASES_MVP_SPEC + Concept Paper + Editor-Spec + eNotePane-development-record indexed; 14 translated User Manuals confirmed (ar = 1328 lines, others = 1120, parity confirmed); 20 session logs digested chronologically.
- **§17 remaining unknowns**: badge letters W and M (defined in code but undocumented); sidebar active-item highlight ~10 s lag origin (no reactive source isolated).

---

## 1. What Constellation IS

**Constellation is a Personal Knowledge Formulation desktop application.**

The distinction is fundamental — it is **not** PKM (Personal Knowledge Management):

> Knowledge Management asks: "Where did I put that?"
> Knowledge Formulation asks: "What can I BUILD from what I know?"
> *(`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md:13-17`)*

It is built on **standard Markdown files** (`.md` + YAML frontmatter) on the user's local filesystem, with a portable Universe-config layer above. Local-first, no telemetry, no cloud, no account.

- **Author**: Eisa ALSHAMSI
- **License**: MIT
- **Repository**: `github.com/eisaShamsi/Constellation`
- **Stack**: Tauri v2 (Rust backend) + SvelteKit + Svelte 5 + SQLite (rusqlite, bundled) + ONNX Runtime (`ort`) + CodeMirror 6 + PIXI v8 + D3 v7
- **Languages supported at launch**: 15 — `ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh`
- **RTL languages first-class**: 4 — Arabic, Hebrew, Persian, Urdu
- **Platforms**: Windows, macOS, Linux desktop. CI ships Windows builds today.
- **Mobile**: iOS/Android excluded via `cfg(not(any(target_os="ios", target_os="android")))` for `memmap2`. Not shipping mobile apps.

---

## 2. Universe / Library / Note hierarchy

Constellation has a **five-level knowledge hierarchy**:

```
Universe (root, named by user, contains universe.json)
  └── cUniverse (child universe — federation of libraries)
       └── Library (self-contained knowledge base, like Obsidian vault)
            └── Folder (subdirectory inside a Library)
                 └── Note (single .md file with optional YAML frontmatter)
```

- **Universe** = portable directory. Contains `.constellation/` subfolder with `universe.json`, `libraries.json`, `settings.json`, `bookmarks.json`, `workspaces.json`, `property-types.json`, `bases/`, `templates/`. Move it to another machine and the entire workspace follows.
- **Library** = first-class citizen with its own color/appearance/tags/links/index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. Constellation reads them in place — never copies.
- **Folder ≠ Library**. Folders are organizational only.
- **Terminology**: use "Library" everywhere, **never** "vault" (except for Obsidian import compatibility).

### 2.1 Universe migration (legacy → current)

[`universe.rs::migrate_legacy_data`](src-tauri/src/universe.rs:1306) moves a v1 layout to v2:

- **From**: flat `universe.json` / `vaults.json` / `settings.json` at universe root; registry stored at `app_data_dir/vaults.json`; nested `name/name/` notes layout.
- **To**: `.constellation/` subdirectory; `vaults.json` renamed to `libraries.json`; registry moved to `app_data_dir/universes.json` (UniverseRegistry with `entries` and `active_id`); flat notes layout (Universe root IS the library, Obsidian-style).

`migrate_to_constellation` (line 133), `ensure_universe_notes_folder` (line 195), `set_active_universe` (line 545 — also consolidates same-name nesting `C:\Name\Name\` → `C:\Name\`).

### 2.2 Child-universe federation

[`universe.rs:425`](src-tauri/src/universe.rs:425) `resolve_child_universe_roots(parent)` reads `universe.json::children[]`, canonicalizes, filters directories. `resolve_libraries_recursive` (line 353) collects own + all child libraries, prevents circular refs, deduplicates by path. Frontend command: `resolve_universe_libraries`.

---

## 3. Architecture (one-page view)

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (SvelteKit / Svelte 5)                                │
│  src/routes/                                                    │
│    +layout.svelte (6872 lines — orchestrator, see §3.2)         │
│    +page.svelte (1 line — note viewing handled by layout)       │
│    libraries/+page.svelte (704 lines — library management)      │
│    skills/+page.svelte (219 lines — skills/onboarding)          │
│  Second window: static/screen.html (separate Tauri webview)     │
│  Editors: NotePane.svelte (388) / FocusPane.svelte (213)        │
│  Panels: Sky View (PIXI), Constellation Map (D3 sunburst),      │
│    Inspector 360, Tension, Sight, Lens, Bases, Tasks, Calendar, │
│    Backlinks, OutgoingLinks, IndexPanel, OrgChart, SearchHub    │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC (~120 commands, 32 Rust modules)                     │
│  - perf_trace (LL-021): every dispatch stamped at the boundary  │
│    via Box-typed closure wrapping generate_handler!             │
│  - 3 plugins: opener / process / updater                        │
│  - panic hook in run() writes constellation-crash.log           │
│    (NO panic-handler plugin — just std::panic::set_hook)        │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust, src-tauri/src/, 32 modules — see §3.5)          │
│  - libraries.rs (3978) — file I/O, link extraction, cascade     │
│  - search.rs (4790) — SQLite, FTS5, Living Link triggers,       │
│    sky_nodes/sky_links triggers (Rule 8)                        │
│  - cache.rs (824) — boot snapshot, alias resolution             │
│  - canonical.rs (1401) — YYYYMMDDTHHMMSSZ_KIND_XXXX             │
│  - universe.rs (1472) — universe/cUniverse + legacy migration   │
│  - arabic/ (15 files) — 5-layer morphological engine, mmap'd    │
│  - lexicon/ (6 modules) — Lexical Bridge polylingual lemma graph│
│  - CE Layer 1: strata.rs / maturity.rs / tension.rs /           │
│    provenance.rs / inspector360.rs / lens.rs / lenses.rs /      │
│    review.rs / trails.rs / canvas.rs                            │
│  - bases.rs — .base file CRUD (read-time)                       │
│  - dataview.rs — DQL queries (read-time)                        │
│  - importers.rs — 7 source formats (one-off, async)             │
│  - watcher.rs — notify-rs file watch (must be async)            │
│  - boot_bundle.rs — 10 IPCs collapsed into 1                    │
│  - sky_backfill.rs — resumable populator, BATCH_SIZE=1000       │
│  - embeddings.rs — ONNX multilingual-e5-small (write-time)      │
│  - embeds.rs / fts5_tokenizer.rs                                │
│  - perf_trace.rs — IPC arrival tracer                           │
│  - ai/mod.rs — OpenAI/Anthropic/Gemini/Ollama                   │
├─────────────────────────────────────────────────────────────────┤
│  Storage                                                         │
│  - .md files on disk (source of truth)                          │
│  - SQLite DB at <universe>/.constellation/search.db              │
│    Tables: schema_versions, note_meta, note_embeddings,         │
│    note_links, note_aliases, sky_nodes, sky_links, notes_fts,   │
│    notes_vocab (fts5vocab), sky_backfill_cursor,                │
│    term_vocab [+ bridge_concept_id col post §1C, MIG-013]       │
│    (term_embeddings table retired in MIG-013 §1C)               │
│  - boot-perf.latest.json — per-boot scorecard                   │
│  - .meta.json sidecars for non-markdown files (canonical)       │
│  - .constellation/review-pulse.json — Phase 7 schedule state    │
│  - .constellation/arabic-overrides.json — L5 user overrides     │
│  - kind_registry.json — auto-generated KIND codes (file_kinds)  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Key dependencies (versions)

| Layer | Package | Version | Purpose |
|---|---|---|---|
| Rust | `tauri` | 2.x with `protocol-asset` feature | App runtime |
| Rust | `rusqlite` | bundled | SQLite |
| Rust | `ort` | ONNX Runtime | Semantic embeddings |
| Rust | `tokenizers` | HuggingFace (with `onig`) | Tokenizers |
| Rust | `fst` | BurntSushi | Arabic generative index |
| Rust | `memmap2` | 0.9 (desktop only) | mmap baked Arabic FST — **wired through** [`fst_bake.rs:323`](src-tauri/src/arabic/fst_bake.rs:323) |
| Rust | `notify` | File watcher | |
| JS | `svelte` | ^5.0 | UI framework (runes mode) |
| JS | `@sveltejs/kit` | ^2.9 | Routing |
| JS | `@codemirror/*` | 6.x (full set) | Editor |
| JS | `pixi.js` | ^8.17 | Sky View force graph (LL-019: `pixi.js/unsafe-eval` first) |
| JS | `d3` | ^7.9 | Constellation Map sunburst |
| JS | `@xenova/transformers` | ^2.17 | Frontend ONNX |
| JS | `katex` / `mermaid` / `marked` / `dompurify` | latest | Math / diagrams / markdown / XSS |

Plugins: `tauri-plugin-opener`, `tauri-plugin-process`, `tauri-plugin-updater`. **No panic-handler plugin** — the crash log path uses `std::panic::set_hook` in [`lib.rs:212-222`](src-tauri/src/lib.rs:212).

### 3.2 The `+layout.svelte` reactivity load (corrected counts)

`+layout.svelte` is the orchestrator. **6872 lines as of 2026-04-26.** Reactive declaration counts (verified by Grep this round):

| Kind | Count | LL-002 baseline (2026-03-27) | Change |
|---|---|---|---|
| `$state` | **155** | 77 | +78 |
| `$effect` | **29** | 17 | +12 |
| `$derived` | **1** (`allTagsList`) | 19 | −18 |

Growth drivers: multi-phase graph boot, second-screen sync effects, Tier 1 panel-placement state, child-universe sidebar expansion, lazy-mount flags. The drop in `$derived` count reflects intentional consolidation — derivations now live inside `$state`-bearing handlers or were promoted to module-level helpers.

`+page.svelte` is **a single-line comment** — the entire note-viewing UI is composed inside `+layout.svelte`. The `libraries/` (704 lines) and `skills/` (219 lines) routes are real pages.

**Lazy-mount flags** ([`+layout.svelte:569-572`](src/routes/+layout.svelte:569)): `mapEverOpened`, `orgChartEverOpened`. Both are sticky $state(false), set true via $effect on `showConstellationMap` / `showOrgChart`, **reset in `handleUniverseSwitch` at lines 1935-1936**. Used to gate `{#if mapEverOpened}` ... `{#if showConstellationMap}` two-tier rendering (LL-022 compliance).

**$effect violation candidates flagged** (audit-pending): line 498 (`lastSavedContent` async-race risk per LL-023), lines 781 / 837 / 1235 / 1353 / 1449 / 3480 (always-mounted IPC fan-out — index/sky scans run regardless of visibility).

### 3.3 Tauri command surface

[`lib.rs:233-432`](src-tauri/src/lib.rs:233) registers ~120 commands across 32 modules. The `invoke_handler` is wrapped in a Box-typed closure that records each dispatch via `perf_trace::record(invoke.message.command())` — the LL-021 IPC arrival tracer.

Two Tauri v2 type-system subtleties (from LL-021):

1. `generate_handler!` must be bound via `Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync + 'static>` to pin the macro's `R: Runtime` generic at the binding site.
2. `invoke.message.command()` returns `&str`; call `perf_trace::record` *before* forwarding to `inner(invoke)`.

**[`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) is significantly stale** (last updated 2026-03-31; lists ~80 commands of ~120). Until refreshed, [`lib.rs:233-432`](src-tauri/src/lib.rs:233) is the authoritative command registry.

### 3.4 Build / Release / CSP / Windows / Capabilities

**Versions** (in sync at 0.3.4):
- [`package.json`](package.json) — `"version": "0.3.4"`
- [`src-tauri/tauri.conf.json:4`](src-tauri/tauri.conf.json:4) — `"version": "0.3.4"`
- `src-tauri/Cargo.toml` — bumped per release workflow

**`tauri.conf.json` highlights**:
- `productName: "Constellation"`, `identifier: "world.uconstellation.app"`
- Two windows: `main` (1200×800) and `second-screen` (1200×800, `url: "screen.html"`, `visible: false` at startup).
- CSP: `default-src 'self'`; `script-src 'self' 'unsafe-inline'`; **no `unsafe-eval`** → LL-019 still applies (PIXI must use `pixi.js/unsafe-eval` side-effect import).
- Asset protocol enabled, `allow: ["**/*"]`, `requireLiteralLeadingDot: false`.
- Updater enabled, endpoint = public Gist (`gist.githubusercontent.com/.../latest.json`); minisign pubkey embedded.

**Capabilities** ([`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json)) — applies to both `main` and `second-screen` windows. Permissions: `core:default`, window controls, `core:webview:allow-create-webview-window`, `core:webview:allow-set-webview-zoom`, `opener:default`, `updater:default`, `process:allow-restart`.

**Second-window file**: [`static/screen.html`](static/screen.html) (built copy at `build/screen.html`).

**CI / release** ([`.github/workflows/release.yml`](.github/workflows/release.yml)) — `windows-latest` runner. Tag push `v*` or manual `workflow_dispatch` (bump `patch|minor|major` or `custom_version`). Bumps `package.json` + `tauri.conf.json` + `Cargo.toml` in lock-step, commits, tags, runs `tauri-action`. Post-release, downloads `latest.json` from release assets and `gh gist edit` updates the public Gist that the in-app updater polls.

**No frontend test harness** (no vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`). Rust unit tests only.

### 3.5 Rust module sizes (full census)

| File | LOC | Role |
|---|---|---|
| `search.rs` | 4790 | SQLite schema + FTS5 + Living Link triggers + search commands |
| `libraries.rs` | 3978 | File I/O + cascade walker + link extraction + 11 cascade tests |
| `universe.rs` | 1472 | Universe registry + child federation + legacy migration |
| `canonical.rs` | 1401 | Canonical filename generation + cid_cn migration + repair |
| `cache.rs` | 824 | Boot snapshots (core/graph/sky) + perf instrumentation |
| `embeds.rs` | 708 | Living embed resolver (`![[target]]`) — 7 resolution tiers |
| `inspector360.rs` | 517 | Aggregates 9 phase data per note (read-only); §112 added per-note `stratum` + `precompute_all_strata` |
| `lens.rs` | 419 | Brandes' betweenness centrality + tag-shared edges |
| `sky_backfill.rs` | 470 | Resumable populator (BATCH=1000, sleep=50ms) |
| `tasks.rs` | 495 | Task scanning (Tasks plugin emoji syntax) |
| `boot_bundle.rs` | 138 | 10 IPCs collapsed into 1 round-trip |
| `tension.rs` | — | CE Phase 4 |
| `provenance.rs` | — | CE Phase 5 (isnad-inspired) |
| `review.rs` | — | CE Phase 7 |
| `trails.rs` | — | CE Phase 8 |
| `canvas.rs` | — | CE Phase 10/11 (Cynefin) |
| `lenses.rs` | — | CE Phase 9 (Multi-Lens) — Rule 8 hybrid violation |
| `bases.rs` | — | .base file CRUD — Rule 8 read-time violation |
| `dataview.rs` | — | DQL queries — Rule 8 read-time violation |
| `importers.rs` | — | 7 source formats (Obsidian / Bear / Notion / Evernote / Markdown / HTML / Constellation backup) |
| `embeddings.rs` | — | ONNX e5-small (384-dim, 100 langs) |
| `watcher.rs` | — | Must be `async` (else Boot Criterion 2 dies) |
| `file_kinds.rs` | — | 3-layer kind classification |
| `fts5_tokenizer.rs` | 479 | Custom 'constellation' tokenizer (stemming + bigrams) |
| `perf_trace.rs` | 71 | TRACE_LOG mutex; record/get/clear |
| `strata.rs` | — | CE Phase 2 (8-level hierarchy) |
| `maturity.rs` | — | CE Phase 3 (5 states) |
| `map.rs` | — | Constellation Map (D3 sunburst data) — Rule 8 read-time |
| `arabic/mod.rs` + 14 files | — | 5-layer morphological engine |
| `lexicon/` | 6 files | Polylingual lemma graph |
| `ai/mod.rs` | 406 | 4-provider AI abstraction |

---

## 4. The Cognitive Engine (CE)

`docs/CE-spec.md` + `docs/cognitive-engine-roadmap.md` are the canonical specs. Two-layer architecture.

### 4.1 Seven epistemological foundations (`CE-spec.md:22-29`)

1. Knowledge is not information — value is in connections, not storage.
2. Knowledge has a vertical dimension — 8-level hierarchy (Datum → Worldview).
3. Knowledge has a certainty dimension — `ilm al-yaqin → haqq al-yaqin`.
4. Knowledge is organized by immutable principles — non-contradiction, causality, hierarchy.
5. Knowledge has diverse sources — sensory, rational, transmitted, experimental, intuitive.
6. Knowledge exists on a spectrum — received vs discovered.
7. The essence of knowledge is understanding-generative apprehension.

### 4.2 Layer 1 — Structural Cognition (zero AI). All shipped.

| # | Name | File | Rule 8 |
|---|---|---|---|
| 1 | Typed Links | `libraries.rs` + `search.rs` (note_links + triggers) | ✅ Write-time |
| 2 | Knowledge Strata (8-level) | [`strata.rs`](src-tauri/src/strata.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1137`](src-tauri/src/search.rs:1137)) |
| 3 | Maturity Lifecycle | [`maturity.rs`](src-tauri/src/maturity.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1215`](src-tauri/src/search.rs:1215)) |
| 4 | Tension Detector | `tension.rs` | ⚠️ Partial — contradictions cached, structural gaps on read |
| 5 | Provenance Chain (isnad-inspired) | `provenance.rs` | ⚠️ Partial — frontmatter sources cached, traversals on read |
| 6 | Externalization | within `strata.rs` (word_count signal) | ✅ Write-time |
| 7 | Review Pulse | `review.rs` | Hybrid — `.constellation/review-pulse.json` |
| 8 | Trails | `trails.rs` | ✅ Write-time |
| 9 | **Multi-Lens Views** | `lenses.rs` | ❌ **Hybrid violation** — definitions write-time (`lenses.json`), results recomputed on read (`apply_lens` walks the tree) |
| 10/11 | Expression Forge / Sense-Making Canvas | `canvas.rs` | ✅ Write-time (JSON persisted) |
| 12 | 360° Inspector ✅ enabled v1.8 §93, hardened v1.9 §96–§104, **redesigned v1.15 §112 (Stratification Matrix)** | `inspector360.rs` (517 lines) | ⚠️ **Read-time aggregation, but actual perf is fine** — the per-fetch cost was theorised as 1–3 s but Boss's lived experience is "almost instantly". MIG-010 (cache `note_360_view` write-time) priority dropped to LOW. Frontend mitigations still in place: debounce 200 ms, sequence guard, last-fetched-key dedup, lazy mount, dedupe-by-path in the matrix. |

**Inspector 360° aggregator** ([`inspector360.rs:1`](src-tauri/src/inspector360.rs:1)): aggregates `Note360View` from typed/untyped links (7 types) + active-note stratum + maturity + contradictions + orphan/SPOF flags + provenance + stage + review + trail membership + lens groups + missing-link-types gap analysis. **Post-§112**: every `LinkedNote` (outbound, inbound, second-order) also carries `stratum: u8`, populated by `precompute_all_strata()` — a single pass that builds an inbound-count + sources-of map for the library, then runs the existing `compute_stratum_for_note` rule set against each note. O(N + total_links). Same big-O as before; constants higher but sub-second on the 7,600-note Universe per Boss's lived experience.

**Frontend Inspector 360 surface** (post-§112 — **Stratification Matrix**):

- Two display modes via the `compact` prop. Compact = right-sidebar tab (scorecard glance widget). Full-window = ribbon-dock button (deliberate-study matrix, replaces editor area).
- **Full-window = the matrix.** HTML/CSS Grid (no SVG polar coordinates). 8 rows (stratum L8 → L1, top-down) × 8 columns (`supports`, `contradicts`, `causes`, `derives-from`, `generalizes`, `exemplifies`, `part-of`, `untyped`) + a 200 px row-label column on the left + a 64 px row-totals column on the right. Each `(stratum, type)` cell holds the connected notes whose stratum matches the row, drawn as 11 px coloured dots (max 16 per cell, then `+N` overflow chip). Active note's row is highlighted (purple background gradient + bold `L{n}` chip showing the note's truncated name). Empty cells render diagonal stripes — gaps as first-class signal.
- **Compact = a scorecard.** Stratum pill (`L4 Concept`), maturity pill, ↑outbound/↓inbound/word-count line, per-type bar chart (label + filled track + count, with explicit `—` for blind spots and 50 % opacity to mark zero rows), and a flags row (orphan, fragile, gap count, due for review). No matrix — 280 px is too narrow.
- **Multi-hop back stack** shared between compact and full-window. State: `inspector360BackStack: $state<Array<{path, name}>>` in `+layout.svelte`. Forward node-click pushes current; back click pops one entry; bar shows `← {previous}` until empty. Universe switch resets to `[]`.
- **Hover-only labels** (preserved from §107). Hovering a dot reveals the connected note's name in a fixed top-right tooltip on the matrix canvas — does not follow the mouse, doesn't pop chrome on dense rows. The dot itself enlarges (`scale(1.6)`) and gains a colored glow (`box-shadow: 0 0 10px var(--dot-color)`) on hover.
- **Per-cell dedup** on path so the same note returned from outbound + inbound + second-order sources renders once per `(type, stratum)` cell.
- **Dimension strip** below the header surfaces the non-spatial dimensions: Stratum (with name), Maturity (color dot), Origin + trust depth (color dot), Stage (icon + name), Review (date or "Due"), Trails / Lenses (count) — only shown if non-empty.
- **Bottom HUD** keeps the existing `total_outbound` / `total_inbound` / `word_count` summary plus warning chips for orphan / fragile / blind-spots / tensions.
- **Dropped permanently**: `vizMode` dropdown (Atmospheric / Neural / Cosmic), `SECTOR_MAP`, `polarToXY`, `ringsLayout`, `layoutMode`, `allNodes`, `SECTOR_THRESHOLD`. Polar geometry is gone from the file; the design space the matrix occupies is grid + axis semantics.

### 4.3 Layer 2 — AI Discovery (5 phases, 🔲 all not started)

12. Hidden Pattern Discovery (ghost links via semantic engine).
13. Blind Spot Detection.
14. Cross-Domain Insight Generation.
15. Socratic Challenger.
16. Worldview Synthesis.

Local-LLM-first; cloud opt-in only. Existing infrastructure: `ai_send_message` Tauri command across 4 providers (OpenAI / Anthropic / Google Gemini / Ollama — [`ai/mod.rs:1-406`](src-tauri/src/ai/mod.rs:1)); embeddings via ONNX multilingual-e5-small (384-dim, 100 languages — `embeddings.rs`).

### 4.4 The Living Link Architecture (P0–P5 all shipped + user-validated)

`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` is the philosophy doc.

**8 link properties**: Type · Direction · Annotation · Weight · Confidence · Created · Last Traversed · Traversal Count.

**7 typed link types** (default `relates`/`associative`):
`supports` (blue) · `contradicts` (red) · `causes` (orange) · `exemplifies` (green) · `generalizes` (purple) · `derives-from` (gold) · `part-of` (gray).

**Syntax**: `[[Target|type]]` (pipe-after-target). The 3-part form `[[Target|alias|type]]` is parsed via `lastIndexOf('|')` ([`livePreview.ts:926-965`](src/lib/editor/livePreview.ts:926)).

**4 confidence levels**: `hypothesis` → `evidence` → `established` → `contested`. Auto-promote at traversal_count ≥3 → evidence, ≥10 → established. Manual override via right-click.

**Decay formula** (display-only — `weight` raw column never modified):
```
effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)
```
Default half-life: 60 days.

**Storage**: dual-layer design (LINK files on disk + SQLite). **The on-disk LINK files layer was deliberately deferred** — implementation lives only in `note_links` SQLite table.

**Archive = soft-delete.** Reversible via Link Dashboard's Archived tab.

**Lifecycle commands** ([`search.rs:2330-2938`](src-tauri/src/search.rs:2330)): `_link_stats`, `_link_traverse` (updates weight via `1.0 + ln(1 + traversal_count)`), `_link_dormant`, `_link_decay`, `_link_set_confidence`, `_link_backfill_confidence`, `_link_archive` / `_unarchive` / `_archived`.

---

## 5. The Arabic Engine + Lexical Bridge

A native 5-layer morphological engine. Built from scratch, license-clean. **Not a port.**

### 5.1 Engine architecture (verbatim from [`arabic/mod.rs:16-37`](src-tauri/src/arabic/mod.rs:16))

```
[L1 normalizer]        — tashkeel / tatweel removal, hamza variants,
                          language detection; preserves surface form
   ↓
[L2 protected list]    — ~20K proper nouns + loanwords (hash lookup)
   ↓
[L3 generative FST]    — rolling-hash + FST over all (root × pattern)
                          combinations
   ↓
[L4 disambiguator]     — ranks multiple analyses by corpus frequency
   ↓
[L5 user overrides]    — per-Universe learning layer
```

**5 logical layers, 15 physical Rust files** in `src-tauri/src/arabic/`:

- `normalizer.rs` (484 lines) — L1: tashkeel/tatweel strip, aggressive folding (alif/ya/ta-marbuta), script detection (Arabic/PersianFamily/Hebrew/Latin/Other). Core test: `وائل` survives stripping (Light10 bug fix).
- `protected.rs` (551 lines) — L2: TSV-backed `HashMap<stripped, ProtectedEntry>` (~1196 entries). Categories: ProperNoun / Place / Loanword / Function. First-write-wins on dupes. M1e flagship: `وائل`, `محمد`, `إنترنت` return verbatim with confidence=1.0.
- `fst_index.rs` (598 lines) — L3: `GenerativeFst` wraps **two `fst::Map<FstBytes>`** (stripped + folded). Packing: FST value = `(offset u32 << 32 | count u32)`. ~300K distinct keys, ~1.1M forms at 7K-root scale, single-digit MB via prefix sharing.
- `fst_bake.rs` (991 lines) — M3-baker on-disk cache. **mmap wired through line 323**: `Mmap::map(&file)?` → `Arc<Mmap>` shared by both stripped + folded FSTs (single syscall + VMA). Cache filename: `arabic-fst-v{djb2(SEED_TSV) XOR CACHE_FORMAT_VERSION:016x}.bin`. Mobile fallback: heap `Vec<u8>`.
- `generator.rs` — Template substitution `(Root, Pattern) → surface`. Placeholders ف/ع/ل. Phonology passes: gemination fusion, hamza carrier picking, weak-radical rewrites (M2.c).
- `patterns.rs` — ~158 morphological patterns (verbal 50, verbal nouns 20, participles 22, broken plurals 27, etc.). All patterns carry full tashkeel.
- `roots.rs` — Root inventory (595 seed → 7K corpus). Classification: Hamzated / Geminated / Assimilated / Hollow / Defective / Sound (triliterals); Sound / Weak (quadriliterals).
- `affixes.rs` — Affix-peeling cascade (e.g., ال + كاتب).
- `disambiguate.rs` — L4 deterministic ranking (confidence → origin priority → POS → fewer affixes → alphabetic).
- `overrides.rs` — L5 per-Universe JSON store at `<universe>/.constellation/arabic-overrides.json`. Tauri commands: `read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`, `reindex_arabic_overrides`.
- `types.rs` — `Analysis`, `Root`, `Pattern`, `PartOfSpeech`, etc.
- `regression.rs`, `bench.rs`, `rss.rs` — test/bench harness (cfg-gated).

**Entry points** ([`arabic/mod.rs:129-564`](src-tauri/src/arabic/mod.rs:129)): `analyze`, `analyze_with_overrides`, `analyze_best`, `analyze_with_overrides_best`.

### 5.2 M-numbered milestones (NOT module boundaries)

The "M3-M14" series in session logs are **project milestones**. Engine is 5 layers (above). All M-milestones shipped:

- M3 FST-backed generative index + M3-baker cache.
- M5 502-case regression corpus, 100% pass.
- M6 FTS5 routes Arabic stemming through `analyze_best`. Closes flagship `وائل → "ائل"` mangle.
- M7 deterministic disambiguator.
- M8 + M8b + M8c — L5 user overrides + ACTIVE_STORE registry + Settings UI.
- M9 bench — ~130k words/sec, ~7.6 MiB cache.
- M10 Lexical Bridge architecture (15-concept seed).
- M11-infra Lexical Bridge baker.
- M11-data v1 (49-concept seed).
- **M11-data v2 Producer ✅ complete** — **20,000 concepts** across **499 thematic shards** in `lab/m11-data/concepts/` (verified by `wc -l lexicon_v1.tsv` = 20,015 lines incl. header).
- M12 query expansion plumbing (`escape_fts_term`, `build_match_expr`, `expand_to_match_expr`).
- M12-detect language detection (15-language classifier).
- M12-bench (mean 5.2 µs, p99 15.8 µs — 60–600× under 1 ms budget).
- M13 multilingual result badge (`match_via`).
- M14 lexical_search end-to-end bench gate.

### 5.3 Lexical Bridge (`src-tauri/src/lexicon/`, 6 modules)

**Polylingual lemma graph**, not a morphological tool: every lemma in any of the 15 languages can be looked up and yields its equivalents in any other.

- **graph.rs** — Node identity: `(lang, lemma, sense_id)`. Edge types: Equivalent / Synonym / Hypernym / Hyponym / UserLink. Storage: FST `{lang_code}:{normalized_lemma} → (first_node_idx u32 << 32 | sense_count u32)`. Core tier: ~20K concepts × 10 langs ≈ 200K nodes, ~800K edges.
- **expansion.rs** — Query expansion. `SynonymLevel`: None / Synonym / SynonymAndHypernyms (±1 hop). Pipeline: lemmatize → fetch equivalents → add synonyms/hypernyms → build FTS5 MATCH across selected languages. Cap 8 per language by default.
- **bake.rs** — TSV ingestion + binary cache (content-addressed, version-hash gated).
- **detect.rs** — Language detection (15-language Unicode classifier).
- **fts.rs** — FTS5 integration (escape, match expression assembly).
- **parse.rs** — TSV format parsing.

Source: `src-tauri/src/lexicon/data/lexicon_v1.tsv`. Built deterministically by [`lab/m11-data/build.py`](lab/m11-data/build.py) (Python 3) from 499 JSON shards.

**Coverage policy**: `en` + `ar` required per concept; target ≥8 of 15 languages. **No third-party sources** — all content original (WordNet / Wiktionary explicitly rejected per project policy in `lab/m11-data/README.md`).

### 5.4 Custom FTS5 tokenizer ('constellation')

[`src-tauri/src/fts5_tokenizer.rs`](src-tauri/src/fts5_tokenizer.rs) (479 lines). Wraps the Rust stemming pipeline: Arabic Light10 + Hebrew prefix stripping + Persian / Cyrillic / Devanagari / German / Spanish / Portuguese / French / Turkish / English stemmers + bigrams. Symmetric across `FTS5_TOKENIZE_DOCUMENT` (write) and `FTS5_TOKENIZE_QUERY` (read).

**Token emission**:
1. Primary token: stemmed form.
2. Bigram (colocated): `prev_stem \x1f cur_stem` (separator `0x1f` unmatchable in user text).
3. Stopwords/length-filtered: emit nothing, break bigram chain.
4. Bigrams form **only between tokens in the same script** (prevents Arabic↔English bigram noise).

All Arabic-side morphology delegates to `crate::libraries::process_word_for_fts` → `analyze_best()`.

### 5.5 Other Rust modules (read this round)

- **inspector360.rs** (517, post-§112) — see §4.2 row 12.
- **lens.rs** (419) — Brandes' betweenness O(VE), weighted by link_type (supports=1.0, causes=0.9, contradicts=0.8). **At >500 nodes**: approximate sampling (200 sources). Tag-shared edges command: weight 0.6 × shared_tag_count, top 500.
- **boot_bundle.rs** (138) — `BootBundle`: libraries + settings + bookmarks + workspaces + property_types + workspace_bases + child_universes + child_universe_lib_paths + per-step `timings_ms`. Replaces ~10 serialized IPCs.
- **sky_backfill.rs** (470) — MIG-001 §5 resumable populator. `sky_backfill_cursor` table stores `last_path`. `BATCH_SIZE=1000`, `INTER_BATCH_SLEEP_MS=50`. Per-batch phases: A (insert sky_nodes/links under lock) → B (read note files, compute word_count + created_at + aliases, no lock) → C (UPDATE note_meta) → D (UPDATE sky_nodes stratum/maturity). Idempotent via `INSERT OR IGNORE`. Final stamp: `schema_versions.sky = SKY_SCHEMA_VERSION`.
- **tasks.rs** (495) — `[- | * | +] [ ] | [x] | [X] text` pattern. Extracts: due_date (📅 YYYY-MM-DD or `[due:: …]`), priority (⏫🔼🔽), tags (#tag), created_date (➕), done_date (✅). Commands: `scan_library_tasks`, `scan_note_tasks`, `toggle_task`, `scan_library_note_dates`.
- **embeds.rs** (708) — Living embed resolver. 7-tier search order: relative-to-note → absolute-in-vault → explicit-attachment-folder (`.obsidian/app.json`) → fallback (attachments/ images/ assets/) → vault-wide index → vault root. `EmbedKind`: image / audio / video / pdf / canvas / excalidraw / note / generic / missing. URLs: data: if ≤4 MB, else `asset://localhost/{encoded_path}`. Digit normalization: Arabic-Indic (٠–٩) + Extended (۰–۹) → ASCII.
- **embeddings.rs** — ONNX runtime + multilingual-e5-small (384-dim, 100 langs), 100% offline. `constellation_init_embeddings`, `_embed_text`, `_embed_notes`, `_embedding_status`. Vectors persisted to SQLite. **MIG-013 §1A**: added `pub fn embed_passages_standalone(model_path, tokenizer_path, texts, intra_threads, batch_size)` for the offline `build_concept_vectors` `[[bin]]` (builds its own ONNX session without an `AppHandle`). **MIG-013 §1C (pending)** removes the per-library term-embedding loop (`init_term_embeddings`, `populate_term_vocab`, `term_embeddings` table) — superseded by the M11 Bridge Adapter below.
- **bridge_vectors/** (MIG-013 §1B) — CTSE Bridge Vector Store. `asset.rs` parses the baked `concept_vectors_v1.bin` (30 MB, 20K × 384 f32, magic `CTSEBV01`, L2-normalized) via `include_bytes!` into an owned `Box<[f32]>`. `store.rs` does cosine k-NN over the flat row-major matrix (`nearest_concept`, `nearest_concepts_k`). `mod.rs::get()` is the `OnceLock` singleton. **Constant-time semantic search regardless of library size** — the asset is fixed at compile time.
- **ctse/** (MIG-013 §1B) — Constellation Terms Scanning Engine, Bridge Adapter. `resolve_term_pure(graph, store, embed_query, term, lang, threshold)` — pure DI core for tests. `resolve_term_to_concept(app, term, lang)` — Tauri-context wrapper. **Fast path**: `LexiconGraph::find_nodes` + `graph.nodes[idx].concept_id` (microseconds, no ONNX, ~80% hit rate on M11-covered terms). **Slow path**: e5 query embed + cosine k-NN, gated by `DEFAULT_THRESHOLD = 0.78`. M11 zero-touch invariant: every CTSE commit verifies `git diff src-tauri/src/lexicon/` returns empty. §1C wires `resolve_term_to_concept` into `reindex_single_note`; §1D wires it into the search query path.
- **build_assets/build_concept_vectors.rs** (MIG-013 §1A) — offline `[[bin]]` target run once per release: `cd src-tauri && cargo run --bin build_concept_vectors --release`. Reads M11 TSV (read-only via `lexicon::parse`), picks one canonical surface form per concept (`en > zh > es > fr > de > ja > ru > pt > ar > ko > hi > tr > fa > he > ur` priority), embeds with multilingual-e5-small in batches of 128, validates per-vector L2-norms, writes `bridge_vectors/data/concept_vectors_v1.bin`. Boss-approved policy: the `.bin` is committed to the repo (changes only when `lexicon_v1.tsv` does).
- **importers.rs** — 7 formats async. `import_pick_source`, `_preview`, `_execute`, `_with_canonical`.
- **watcher.rs** — `notify` crate. **MUST be `#[tauri::command(async)]`** (recursive watch is blocking I/O; sync command runs on WebView2 UI thread → Boot Criterion 2 fails). Inline note at lines 19-38 explains the constraint.
- **dataview.rs** — DQL TABLE / LIST / TASK / CALENDAR + FROM + WHERE + SORT + LIMIT. Reuses bases.rs scan primitives. Read-time recompute on every `execute_dataview_query`.
- **bases.rs** — `.base` YAML CRUD. Live scans on `query_base`. 5 commands.
- **perf_trace.rs** (71) — `static TRACE_LOG: Mutex<Vec<(String, u64)>>`. `record(cmd)` / `get_perf_trace_log` / `clear_perf_trace_log`.
- **file_kinds.rs** (454) — 3-layer kind classifier. Layer 1: extension map. Layer 2 (markdown): explicit frontmatter `kind:` / `type:`, then heuristics (LINK = from+to fields; TMPL = `<%…%>` / `{{…}}` ≥3 occurrences or `template: true`; MARK = `url:` + body <500 chars; CLIP = `source:` + blockquotes; BASE = `schema:` / `dataview` blocks; default = NOTE). Layer 3: unknown extension → `auto_generate(ext)` → persist in `kind_registry.json`. 4 unit tests.

---

## 6. Filename + Identity Architecture (post-MIG-003, 2026-04-28)

> **Architecture inverted by MIG-003 (commits §85–§89). The legacy "canonical filename = primary key" design is preserved as historical record in `docs/CANONICAL-FILENAME-ARCHITECTURE.md` § 0 banner; the rest of that doc describes the pre-MIG-003 design.**

### 6.1 Two ids, two purposes

| | What it is | Where it lives | Mutability |
|---|---|---|---|
| **`cid_cn`** | Immutable internal id, namespace-safe ("Constellation Node id") | Frontmatter `cid_cn:` field + `note_meta.cid_cn` column + every dependent-table `_cid_cn` column | **Never changes** for the life of the note |
| **Filename** | Human-readable representation of the title | The on-disk `.md` filename + `note_meta.path` column | Changes when the user renames the note |

`cid_cn` format is still the canonical pattern (`YYYYMMDDTHHMMSSZ_KIND_XXXX`), but it is no longer used as a filename — only as an internal correlation key.

### 6.2 Frontmatter contract

```yaml
---
title: Agriculture System
cid_cn: 20260410T153045Z_NOTE_7F3A
kind: note
created: 2026-04-10T15:30:45Z
aliases:
  - Old Title (preserved on rename)
---
```

`title` is user-mutable and equals the filename stem in the steady state. `aliases:` accumulates old titles automatically on rename (so wikilinks targeting the old name still resolve). `cid_cn:` is the load-bearing internal id and is never edited by the user.

### 6.3 12 file kinds — unchanged

`NOTE` · `BASE` · `TMPL` · `LINK` · `MARK` · `CLIP` · `IMG` · `AUD` · `VID` · `ATT` · `CANVAS` · `DRAW` ([`file_kinds.rs:25-45`](src-tauri/src/file_kinds.rs:25)). Auto-generated for unknown extensions (e.g. `.blend` → `BLEND`). The kind is recorded in `cid_cn` itself (the `_KIND_` segment) and in frontmatter; classification logic is unchanged.

### 6.4 `cid_cn` generator

[`canonical.rs:49-93`](src-tauri/src/canonical.rs:49) — timestamp source priority: frontmatter `created:` → filesystem creation → modification → `Utc::now()`; XXXX is 4-char uppercase hex; collision avoidance tries 10 hex suffixes, fallback +1 second. Output is the cid_cn string written to frontmatter at note creation.

### 6.5 Rename flow (post-MIG-003 §89)

`rename_item` ([`libraries.rs:rename_item`](src-tauri/src/libraries.rs)) — unified single path for `.md` files:
1. Read current frontmatter title (for alias preservation).
2. Update frontmatter title + append old title to `aliases:`.
3. `fs::rename` old_path → new_path.
4. Cascade DB: `UPDATE note_meta.path` (fires `note_meta_sky_au` → propagates to sky_nodes/sky_links) + explicit UPDATE on `note_links.source_path/.target_path`, `note_aliases.path`, `note_embeddings.path`.
5. Stamp 'rename' alias row keyed to the new path (durable safety net independent of frontmatter edits).
6. Reindex the note at new path.
7. Frontend cascades `[[OldTitle]]` → `[[NewTitle]]` body rewrite via existing `update_links_on_rename`.

The legacy "canonical-detection special case" that updated frontmatter without renaming the file is **removed**. Folder rename keeps the legacy fs::rename-only flow (folder DB cascade is its own concern, deferred).

### 6.6 New-note creation flow (post-MIG-003 §89)

`create_note` ([`libraries.rs:create_note`](src-tauri/src/libraries.rs)) — single unified path:
1. Sanitize the user-supplied title via `note_display_filename()` (strips reserved chars, falls back to "Untitled" if empty).
2. Resolve filename collision via `resolve_filename_collision()` — auto-suffixes "Untitled" → "Untitled 1.md" → "Untitled 2.md".
3. Generate fresh cid_cn via `canonical::generate_canonical()`.
4. Write frontmatter with `title`, `cid_cn`, `kind`, `created`.

The previous `native` / `compatible` mode branching is removed. Every library creates human-named files; cid_cn lives only in frontmatter.

### 6.7 Wikilink resolution — unchanged shape, alias-aware

Wikilinks target **titles**, never cid_cn. Resolution order: `title exact → aliases → original_filename → broken (red)`. The alias table (`note_aliases`) is populated from frontmatter `aliases:` lists by the indexer plus explicit 'rename' rows stamped by `rename_item`.

### 6.8 The MIG-003 commit trail

| § | What landed |
|---|---|
| §85 (Step 1) | `cid_cn` column on `note_meta` + UNIQUE index `idx_note_meta_cid_cn` + backfill from frontmatter (7,610 rows; 38 + 4 collisions auto-resolved). Schema-versions module `note_meta` stamped to 1. |
| §86 (Step 2) | `cid_cn` columns on `note_links` (source + target) / `sky_nodes` / `note_aliases` / `note_embeddings` + per-table backfill via JOIN on existing path columns. Schema-versions module `dependent_tables_mig003` stamped to 1. |
| §87 (Step 3) | All 7 INSERT writers stamp cid_cn at write time. `note_meta_sky_ai` trigger updated to copy cid_cn. Boot-time soft re-backfill (cheap, 0 rows in steady state). The `target_cid_cn` bulk re-backfill was caught + omitted (would have hung the app at boot — Working Agreement #4 violation). |
| §88 (Step 4) | New module `mig003_step4.rs`. Walked 17 libraries, found 19 canonical-named .md files (only the user's "inbox" Universe Notes folder used canonical mode; the 16 declared libraries already had human filenames). Per-library transaction; audit log to `.constellation/mig003-step4-renames.tsv`. Schema-versions module `mig003_step4` stamped to 1. |
| §89 (Step 5) | Unified `create_note` + `rename_item` flows. Canonical-detection special case removed (dead code post-Step-4). |

### 6.9 What was deliberately skipped

- **Step 6** (promote `cid_cn` to formal PRIMARY KEY of `note_meta`, drop redundant path columns from dependent tables) — the dual-keyed schema is not a defect; path columns are still load-bearing for fs operations; the rebuild risk was judged not worth the cleanliness gain.
- **§89 alias-append** (preserve old canonical stem in frontmatter aliases of the 19 renamed files) — those files are all dev/test notes from this week's work, no external references existed; saved as wanted-feature memory if future external integration ever needs it.
- **User Manual + 14 i18n translations update** — the user-visible behavior change is small (filenames are now intuitive); separate doc-only commit when convenient, not a blocker.

### 6.10 Legacy commands still in the tree

- `canonicalize_preview` / `canonicalize_execute` / `auto_canonicalize_all` / `inject_cid_library` / `de_canonicalize_library` / `repair_external_libraries_on_startup` — these were the original architecture's tooling. Post-MIG-003 they are mostly dead code. `inject_cid_library` is harmless (just stamps cid_cn into frontmatter); `de_canonicalize_library` is a no-op in the new world (filenames are already human). Deletion candidates for a future cleanup migration; not urgent.

---

## 7. Editor (NotePane / FocusPane)

**Two editors**:

- **[`FocusPane.svelte`](src/lib/components/FocusPane.svelte)** (213 lines) — quick capture, plain text. Imports **only** `bidiPlugin` + base CM6. No markdown parser, no syntax highlighting, no decorations. Comment at line 201 codifies: "Tab switches destroy/recreate FocusPane with new value prop" — no $effect for value sync.
- **[`NotePane.svelte`](src/lib/components/NotePane.svelte)** (388 lines) — full WYSIWYG-like CodeMirror 6. Live preview decorations, callouts, code blocks, images, wikilinks, tables.

### 7.1 The shared editor stack — full per-plugin

`src/lib/editor/` — 11 plugins per the **Editor Parity Rule**.

- **activeEditor.ts** (24) — Singleton `lastView` registry; queried by emoji/icon picker.
- **bidiPlugin.ts** (209) — Per-line script detection (Arabic, Hebrew, Devanagari, CJK split into Hiragana/Katakana → Japanese, Hangul → Korean, else Chinese, Cyrillic, Latin). Theme rule `unicodeBidi:isolate` on `[dir]` lines. Empty-line RTL inheritance from preceding non-empty line. Viewport-only scan; debounced 300 ms.
- **calloutPlugin.ts** (420) — **LL-014 freeze-proof architecture** (lines 5-23 doc):
  - **RULE A**: `Decoration.replace` only when cursor on **different line**. Provably safe — cursor on line N cannot be inside replace covering line M (M ≠ N).
  - **RULE B**: Collapsed body lines use zero-length `Decoration.line({class})` at `line.from === line.from`. CSS `display:none` on `.cm-callout-body-collapsed` does the hiding; Decoration.replace never spans the collapsed region. Cursor never gets "inside" a replace → no CM6 nudge loop.
  - Fold state: `StateField<Set<number>>`. Line numbers remapped via `tr.changes.mapPos()` on docChanged so fold persists across edits.
- **completions.ts** (156) — Wikilink (20-item cap), tag (Unicode `\p{L}` regex, RTL-aware), typed-link (matches `[[note|type]]` and `[[note|alias|type]]` via `lastIndexOf('|')`), slash (14 commands incl. `/table 3x4`).
- **iconSets.ts** (173) — 4 libraries: Lucide (~1500), Phosphor (~1500), Heroicons (~300), Feather (~290). Lazy-load via single shared promise; cached afterwards. `wrapForInsertion` namespaces icon ids.
- **lineDecoPlugin.ts** (131) — Blockquote + fenced-code line-level borders/background. Syntax tree resolved once at viewport start (replaces O(N) forward scan). Callout detection: upward scan max 50 lines.
- **livePreview.ts** (1271) — Core inline-render plugin.
  - **Pre-cached Decoration objects** at lines 138-181: `headingDecos[0..5]`, `boldDeco`, `italicDeco`, `strikeDeco`, `codeDeco`, `linkDeco`, `replaceDeco`, 8 typed-link decos, 2 checkbox states (CR Rule 1).
  - **ViewPlugin update guard** (LL-002, lines 1046-1098): `contextChanged` branch detects path/attachment-folder/traversal-map state effects; `selectionSet` guard rebuilds **only when cursor crosses line boundary** (CR Rule 1); `docChanged` fast path maps decorations + debounces full rebuild 300 ms.
  - Image/embed resolution: 7-tier search; cached (`_imageCache`, `_embedCache`); circular-transclusion guard (`_transcludeStack`).
  - Widgets: ImageWidget, UniversalEmbedWidget (image/audio/video/pdf/canvas/excalidraw/note-transclusion/generic/missing), IconShortcodeWidget, CheckboxWidget, InlineHtmlWidget, AlignmentWidget, CodeBlockLabelWidget, DataviewLabelWidget. All implement `eq()` for memoization.
  - Living Link traversal chip (P4.2, lines 967-988): keyed on `sourcePathLower|targetNameLower`; emits `×N` widget on high-count links.
- **markdownHighlight.ts** (49) — Lezer extension for `==highlight==`. Adds `Highlight` and `HighlightMark` syntax-tree nodes.
- **shortcodeAutocomplete.ts** (167) — Loads 23 emojibase locale datasets in parallel. Combined emoji + icon ranking; per-set boosts (lucide 0, feather −1, heroicons −2, phosphor −3). Lazy-load on first `:` keystroke.
- **tableFormulas.ts** (163) — `=SUM/AVG/COUNT/MIN/MAX(A1:A5)`. A1 syntax with column-letter → 0-based index. Numeric-aware, fallback to `localeCompare` (Arabic-aware).
- **tableUtils.ts** (363) — `parseTable`, `formatTable`, `generateTable`, `detectTabularText` (TSV-first then CSV, ≥50% row consistency required), add/delete/move row/col, `setAlignment`, `sortByColumn` (numeric-aware).

### 7.2 Key NotePane spec rules (top-principal)

- **§2.1 — The Editor Owns Its Content.** After mount, CM6 owns the document. One-way: Editor → onchange(text) → Parent stores → Debounced save. Never Parent → Editor.
- **§2.6 — No `$effect` for Editor State.** No `$effect` reads or writes `value` / `editBody`. Only allowed: dir change (guarded by `prevDir`), font change (guarded by `prevFontKey`). **Violating §2.6 caused BUG-015** (see §8.1).
- **PaperOnDesk (PoD) layout**: gray desk `#e8e8ec`, white paper `max-width: 1200px`, `padding: 48px`.
- **Auto-title format**: code generates canonical `YYYYMMDDTHHMMSSZ_NOTE_XXXX` filename + `title:` field.

### 7.3 Audit-agent count (clarification)

Three sets exist; the umbrella is "14 audit agents":

- **[`lab/audit-agents.md`](lab/audit-agents.md) — 7**: PA / AA / MA / SCA / RA / UXA / CQA.
- **NotePane spec — 8**: above + **EA** (Environment Auditor), added 2026-03-27.
- **[`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) — 14**: 8 above + LA / SIA / SA / DIA / CFS / OGA.

Migrations use a different cohort: Phase 4 of `/migration` runs three parallel agents (Invariant Check / Drift Check / Migration Path).

### 7.4 `src/lib/libraries/store.ts` (write-ahead buffer, navigation)

**Stores**: `libraries`, `editingTabIds`, `openTabs`, `activeTabId`, `splitActive`, `focusedTabId`, `bookmarks`. Derived: `activeTab`, `universeNotesLibrary`, `selectedNote`, `focusedTab`, `libraryCount`, `totalStars`.

**Save discipline**:
- `saveLocks` map prevents concurrent writes per tab.
- `recentWrites` map (2 s TTL) gates the file watcher to ignore notes the app just wrote (prevents echo loops).
- **Write-Ahead Buffer**: in-memory + localStorage. `getWriteAhead()` checks memory first, falls back to localStorage (crash-safe). Cleared on tab close.
- `saveTabContent()`:
  - Auto-stamps "updated" / "حُدث" date if the property type === `date`.
  - Emits `screen:note-saved` for the second-screen window.
  - Async reindex via `constellation_search_reindex`.
  - Async semantic embed via `constellation_embed_notes`.
  - Tracks recent-edited in localStorage (20-deep) for second-screen dashboard.
  - **Does not dispatch to openTabs during autosave** — editor owns content, store re-syncs on tab switch.

**Navigation**: per-tab `_navTokens` prevent races on rapid Alt+Left/Alt+Right (newer click supersedes in-flight handler). 200-entry `_navTrace` ring-buffer exposed as `window.__navTrace`. Cross-library nav handled in `loadTabHistoryEntry`.

**Frontmatter parser**: multi-line YAML lists + inline `[a, b, c]`; type detection (list / link / checkbox / datetime / date / number / text); **Arabic property keys** recognized (`الوسم`, `وسوم`, `المجموعة`, ...); date normalization DD/MM/YYYY → YYYY-MM-DD.

### 7.5 `src/lib/secondScreen.ts` (12 events main→screen, 4 screen→main, 1 bidi)

**Window mgmt**: `openSecondScreen`, `openSecondScreenSmart` (auto-positions on secondary monitor at 80% size), `closeSecondScreen`, `isSecondScreenOpen`, `listMonitors`.

**Events**:
- **Main → Screen**: `screen:open-note`, `:universe-switched`, `:settings-changed`, `:context-changed` (editor/skyview), `:skyview-hover`, `:skyview-click`, `:sidebar-mode-changed`, `:split-mode-changed`, `:dashboard-open-note`, `:dashboard-tag-selected`, `:index-search`, plus workspace state restore.
- **Screen → Main**: `screen:open-in-main` (reverse-open), `:closed`, `:state-request` (workspace save), `:state-response` (restore).
- **Bidirectional**: `screen:note-saved` (both windows listen).

**Workspace State**: `ScreenState { mode: 'grid'|'star'|'detail'|'skyview'; linkedBrowsing; tabs; activeTabPath }`.

`src/lib/universe/store.ts` — 18 async invocation wrappers. **No local Svelte stores.** Pure IPC pass-through; Rust holds state.

---

## 8. Migrations (active state, 2026-05-07)

`/migration` — four-phase workflow: **Architect → Plan → Build → Audit**.

| ID | Plan | Status |
|---|---|---|
| **MIG-001** Sky View Write-Time Derivation | `lab/reports/MIG-001-SKYVIEW-WTD.md` | ✅ Closed. |
| **MIG-002** Enrichment Persistence | `lab/reports/MIG-002-ENRICHMENT-PERSISTENCE.md` | ⏳ §1–§6 shipped + tested. §7–§10 pending. |
| **MIG-003** Human-name Filenames | `lab/reports/MIG-003-HUMAN-NAME-FILENAMES.md` | ✅ Closed (2026-04-28). Steps 1–5 shipped (§85–§89); Step 6 (PK promotion) skipped by Boss decision; Steps 7–9 (docs + audit + PCS) shipped 2026-04-28. See § 6 of this orientation. |
| **MIG-004** Alias-Aware Resolution | `lab/reports/MIG-004-ALIAS-AWARE-RESOLUTION.md` | ✅ Closed. 9/12 invariants verified. |
| **MIG-005** Alias-aware in-memory inbound | `lab/reports/MIG-005-ALIAS-AWARE-INMEMORY.md` | ⏳ Steps 1–3 shipped (§121/§122/§123 — `map.rs` / `strata.rs` / `maturity.rs`). Tutorial paused after fabrication caught. Steps 4–8 pending. |
| **MIG-006** Wikilink Rename Cascade | `lab/reports/MIG-006-WIKILINK-CASCADE.md` | ⏳ §1 ✅. §2 ✅ + 11 cascade tests. §3 expanded shipped at `3c4732d`, **REVERTED at `5afe0c2`** (BUG-015). §3 redo + §4–§11 pending. |
| **MIG-014** Note-stage taxonomy (per-note dash-encoded) | `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md` | ✅ Closed (2026-05-06). PJ-007 done. |
| **MIG-015** Chunked v2 sentinel migration + status-bar UI | `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md` | ✅ Closed (2026-05-06). PJ-001 done. |
| **MIG-016** Sight instant-toggle perf | `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` | 🟡 **Closed — Cancelled (partial-shipped)** (2026-05-07). §1A + §1B shipped (commits `a0babbb` → `7e76b17` → `62718f7`). §1C / §1D **cancelled**, §1E **deferred to PJ-038 v3 inheritance**. PJ-034 retired. |
| **MIG-017** Disable v2 Sight (precondition for v3) | `lab/reports/MIG-017-DISABLE-V2-SIGHT-{ARCHITECT,PLAN,AUDIT}.md` | ✅ **Closed** (2026-05-07). PJ-039 done. v2 unreachable in default config. Mechanism: `SIGHT_V2_ENABLED = false` const in `src/lib/sight/engine.ts` gates dock button + modal mount + Return-to-Lens button + Settings plugin entry. v2 component + Rust analytics + IPCs preserved on disk as known-good fallback. |
| **MIG-018** Sight v3 projection foundation | `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-{ARCHITECT,PLAN,AUDIT}.md` | ✅ **Closed** (2026-05-07). PJ-038 phase 1 of 3 done. v3 reachable in production. Star-chart visualization with graph-distance Landmark-MDS embedding (Rust), Lambert / stereographic projections (user-toggle), constellation territories (Suwaidi warm-cream + gold palette), faint-at-rest connector lines, hover/click/double-click interactivity, side panel. Three-agent audit CLEAN. `SIGHT_V3_ENABLED = true` committed. Eight commits — see v1.58 preamble. |
| **MIG-019** Sight v3 — density (PJ-035 Milky Way) + calendar rim + search integration + universe-health card | (planned, single MIG) | 🟢 **Next-up** (2026-05-07). Phase 2 of 3 in v3 trajectory. Implements the InfraNodus-defining mechanic (TF-IDF content similarity → Milky Way band) + Gregorian-default calendar rim + full search-flare integration + universe-health metric card in side panel. |

### 8.1 The MIG-006 §3 / BUG-015 incident

- **§115** (`3c4732d`, 2026-04-25) shipped MIG-006 §3 expanded "open-editor coherence" — included a **value-prop → CM6 doc sync `$effect`** in NotePane that dispatched a doc-replace transaction on parent body-prop change.
- The `$effect` raced with `{#key tab.id+'|'+tab.path}` `onDestroy` on tab navigation. Click source → click target → reactivity propagated `tab.content` to target's body → OLD source NotePane's `value` prop changed → `$effect` replaced its own CM6 doc with target's body BEFORE `{#key}` ran destroy → destroy's `doFlush()` read the swapped doc → `handleFlush` wrote that swapped content to the OLD pane's `mountedFilePath`. Result: target file body overwritten with source body.
- **NotePane spec §2.6 explicitly forbade this pattern.** Spec wasn't read before commit.
- §116 (`5afe0c2`) reverted §115. §117 + §118 cleaned docs + recovered disk. BUG-014 closed as collateral.
- **Lesson**: per BASIC RULE + Working Agreement #4, every change touching write paths / lifecycle / reactivity / IPC contract MUST validate against the architecture before shipping. The MIG-006 §3 plan even documented a **fictional** "existing prop-change handler" that didn't exist — the plan misled itself.

---

## 9. Boot performance — 5 ship-gate criteria

`lab/boot-perf/BOOT-BUDGET.md`. Test corpus: **trial Universe (7,600 notes, 16 libraries, 656k typed links, 4k images on Windows 11 NTFS)**.

| # | Criterion | Status |
|---|---|---|
| 1 | UI visible ≤ 2.5 s | ✅ ~870 ms production (verified 2026-04-19) |
| 2 | Fully responsive (`hydrated_ms`) ≤ 6 s | ✅ closed at **811 ms** after Round 7 (LL-021) |
| 3 | Idle RSS ≤ 350 MB | 🔲 Not measured |
| 4 | Stat-sweep 50 externally-modified files ≤ 3 s, non-blocking | 🔲 Not implemented |
| 5 | Kill-mid-index recovery (no duplicate notes, no WAL corruption) | 🔲 Not implemented |

**Permanent diagnostic instrumentation** (kept after Criterion 2):
- **Five-stamp IPC diagnostic** (LL-021): `invoke_start_unix_ms` → `server_start_unix_ms` → per-phase `Instant::now()` → `server_return_unix_ms` → `client_recv_unix_ms`.
- **`perf_trace::TRACE_LOG`** at [`src-tauri/src/perf_trace.rs`](src-tauri/src/perf_trace.rs) — wraps `generate_handler!` to stamp every IPC dispatch arrival.
- **JS heartbeat** (max-gap from `boot:paint` to `boot:hydrated`).

### 9.1 What closed Criterion 2

`perf_trace` arrival tracer (Round 6) showed `constellation_map_universe` dispatched twice (~17.2 s gap), blocking `cache_boot_snapshot_core`. Round-7 fix: single attribute change `#[tauri::command]` → `#[tauri::command(async)]` on `constellation_map_universe`. `core_queue_ms` ~19.9 s → 4 ms; `hydrated_ms` 811 ms. **5,100× reduction.**

### 9.2 Other boot-perf primitives

- **Covering index** `idx_note_boot_snapshot ON note_meta(name, path, library_name)` — 100–1000× speedup (LL-020 corollary).
- **Paint-first UI** (LL-018): `appReady = true` synchronously; data hydrates after.
- **`LIBRARIES_CACHE`** (LL-016): in-memory cache for `load_all_libraries` invalidated by `save_libraries` + `set_active_universe`.
- **Always-mounted lazy-mount** (LL-022): `*EverOpened` flags for Map / OrgChart.
- **Watcher async** ([`watcher.rs:19-38`](src-tauri/src/watcher.rs:19) inline note): recursive watch is blocking I/O; sync command runs on WebView2 UI thread → Boot Criterion 2 fails.

### 9.3 Boot bundle — 10 IPCs into 1

[`boot_bundle.rs`](src-tauri/src/boot_bundle.rs) returns a single `BootBundle { libraries, settings, bookmarks, workspaces, property_types, workspace_bases, child_universes, child_universe_lib_paths, timings_ms[per step] }`. Replaces ~10 serialized invokes during `initializeApp`.

---

## 10. Standing rules (top-principal hierarchy)

### 10.1 BASIC RULE — Don't Make Things Up *(top of all rules)*

If I don't have a clue or information, I say **"I don't know."** No invented file paths, line numbers, function names, badge taxonomies, prior-art summaries, or any factual claim. **Fabrication is the worst class of error** — bugs are recoverable; trust isn't.

When tempted to add a "side note" — every claim in it must be sourced. If any claim isn't, the entire side note is cut.

Canonical violation prevented: 2026-04-26 tutorial fabricated T/C/P badge meanings as "Theory/Concept/Proposition." Actual: T = Title, C = Content, P = Property, with S = Semantic.

### 10.2 Working Agreement #1–#4

1. **Do the work yourself.** SQL, log greps, file inspection, build verification — Claude's job.
2. **One location: `E:\مشاريع كلاود\Constellation` on `main`.**
3. **The user is a non-technical IT Boss.** Plain language; tutorials per §10.4.
4. **Validate every change against the entire architecture before shipping.** Spawn parallel agents for any change touching write paths / lifecycle / reactivity / IPC. (BUG-015 is the canonical violation this rule prevents.)

### 10.3 Standing Orders

1. Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` after every phase / step / significant commit.
2. Update help files + User Manual + 14 translations on user-facing changes.
3. Session log = safety net for context loss.
4. `/simplify` (code review) after each phase.
5. **State-of-standing record before any pivot or major triage** — `§STATE-OF-STANDING` in the day's session log.
6. **Maintain `docs/Constellation Orientation & Onboarding vX.Y.md`** — filename always carries version suffix; rename in same commit on bump.

### 10.4 Tutorial Rule (top principal)

Every test instruction is a tutorial. Define the feature first (what / why / why it matters). Click-by-click walkthrough. Pre-state, action, post-state per step. Failure modes spelled out. Plain language only.

### 10.5 Plan Approval = Build Approval (top principal)

Once user approves a plan, Claude cascades through build steps autonomously. Stops only at: user-testable verification clauses, genuine architectural surprise, plan completion.

### 10.6 Migration Rule

Subsystem-crossing changes go through `/migration` four-phase workflow before any code is written. Single-file refactors → `/simplify`.

### 10.7 Performance Rules (8)

1. Every keystroke instant. Line-change guard for `selectionSet`. Pre-cache module-level Decorations.
2. No `$effect` loops. `$derived` for computed values.
3. No heavy work on the main thread. Vault indexing / search / file I/O → Rust. Debounce saves ≥1500 ms. **Zero `invoke()` on the keystroke hot path.**
4. No memory leaks. Every `setTimeout` / `setInterval` / `addEventListener` / `EditorView` / `listen()` / `requestAnimationFrame` → cleanup in `onDestroy`.
5. Minimal DOM. `display: none` not removal. No `:global()` cross-tree CSS.
6. No unnecessary imports. No `@codemirror/language-data` in FocusPane (500 KB+).
7. Test before commit. 10-char rapid type in NotePane + FocusPane after every change.
8. **Write-Time Derivation.** Every computed view maintained at write time. Persist + trigger on source-of-truth write path. Reads = cheap lookups. **No new feature may regress boot / typing / IPC** on the 7,600-note Universe.

### 10.8 Architecture principles

- **File Over App.** `.md` on disk = source of truth.
- **Local-First.** No telemetry, no cloud dependency.
- **Knowledge Formulation, not Management.**
- **The Living Link Architecture.**
- **Constraint as Design.** FocusPane has no toolbar — that IS the design.
- **Language-First by Design.** Bidi is architectural.
- **Constellation Knowledge Hierarchy** (5 levels).

### 10.9 Don't (hard "no" list)

- Don't use preview/screenshot tools unless essential.
- Don't add unnecessary abstractions.
- Don't use "vault" terminology in new code.
- Don't add a feature that makes the app slower.
- Don't commit `$effect` loops.
- Don't import heavy libraries in FocusPane.
- Don't use `position: absolute` for layout.
- Don't write CSS magic numbers without comment.
- **Don't patch the same bug more than three times** (LL-014).
- Don't create `Decoration.mark/replace/widget` inside builders — pre-cache.
- Don't call `invoke()` from a CM6 ViewPlugin or input event handler.
- **Don't duplicate working code by copy-paste-and-adapt** — extract.
- **Additional screens are displays, not domains.**

### 10.10 PCS Protocol

Push + Commit + Standing Order. Every milestone: verify build → commit → push → milestone tag → ZIP → session log → help files → 14 translations → SO.

### 10.11 Backup routine

`git tag milestone/<name> <commit>` + `git push origin --tags`. ZIP: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`.

### 10.12 Versioned filename for this orientation doc — preserve every version

This file's name **always** carries its version suffix: `Constellation Orientation & Onboarding vX.Y.md`.

**Rule (corrected 2026-04-26):** when bumping the version, **write the new version as a NEW file**. Do NOT delete or overwrite the previous version. Older versions stay in `docs/` as a historical record — the project owner uses the trail to track how the project's architectural understanding evolved.

A new session reads only the highest-version file. But the trail behind it is durable.

---

## 11. Lessons Learned (LL-001 → LL-023, summary)

[`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) is canonical.

- **LL-001** Tauri IPC = #1 perf killer. Zero IPC during typing.
- **LL-002** `+layout.svelte` reactivity cascade. Direct mutation bypasses Svelte; never store-mutate from `onDestroy` or hot path. *(2026-03-27, file 3873 lines / 77/17/19. Today: 6872 / 155/29/1.)*
- **LL-003** Build passing ≠ working app.
- **LL-004** CM6 widget event handling — capture-phase `addEventListener` on editor DOM.
- **LL-005** `tauri dev` rewrites Cargo.toml. Use forwarding feature pattern.
- **LL-006** Phase-by-phase with user GO/NO-GO.
- **LL-007** Shared plugins in `src/lib/editor/` pay off.
- **LL-008** Session log = lifeline.
- **LL-009** Derive state, don't duplicate.
- **LL-010** Merge iteration loops over visible ranges.
- **LL-011** Tauri v2 asset protocol — 4 things: protocol-asset Cargo feature; assetProtocol enable+scope in tauri.conf.json; `http://asset.localhost` in CSP `img-src` AND `connect-src`; `https:` in `img-src`.
- **LL-012** `posAtDOM` unreliable for replacement widgets. Use `posAtCoords({x, y})`.
- **LL-013** `getCursorColumn` pipe-counting bug.
- **LL-014** **Three Strikes** — fix from root after 3 failed patches.
- **LL-015** Always test production before chasing dev-mode performance (~37 s/IPC dev overhead in Tauri v2 + Vite + DevTools).
- **LL-016** Cache at the call site when callers are unknown.
- **LL-017** When patching fails, spawn adversarial expert agents.
- **LL-018** **Paint-First UI** — never gate first paint on IPC.
- **LL-019** PIXI v8 + Tauri CSP — `import 'pixi.js/unsafe-eval'` as side-effect before any PIXI class. Never relax app-wide CSP.
- **LL-020** Wall-vs-server-time diagnostics. Plus covering-index corollary.
- **LL-021** Five-stamp IPC diagnostic + `perf_trace` arrival tracer. Methodology: Stage 1 stamps → Stage 2 plausible patches (stop after 2 fail) → Stage 3 cheap falsifiers → Stage 4 dispatcher tracer → Stage 5 named-culprit conversion.
- **LL-022** Always-mounted UI = always-running IPC. `*EverOpened` lazy-mount. Reset flags on context switch.
- **LL-023** Don't regress working features. 4-step verification: render → event → state → data path.

---

## 12. Documentation drift log

| Doc | Drift |
|---|---|
| [`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) | Last 2026-03-31. Lists ~80 commands; actual ~120. |
| [`docs/CE-spec.md`](docs/CE-spec.md) | Body progress table at line 862-878 stale (says Phases 4 + 7 + 12-16 not started; roadmap and code show 1–11 done). |
| [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) | Says `cid`; code uses `cid_cn` namespace — see §6.1. |
| [`docs/Constellation-Editor-Spec.md`](docs/Constellation-Editor-Spec.md) | Describes a custom-built editor never built. CodeMirror 6 was used. **Aspirational.** |
| `lab/reports/MIG-006-WIKILINK-CASCADE.md:165-167` | The §3 plan claimed an existing prop-change handler that didn't exist. |
| Audit-agent count | `lab/audit-agents.md` = 7; NotePane spec = 8 (adds EA); `docs/AUDIT-SYSTEM.md` = 14. `lab/audit-agents.md` not updated to umbrella. |
| **CE Rule 8 audit-pending** | `bases.rs` (read-time `query_base`); `dataview.rs` (read-time); `lenses.rs` (hybrid violation: definitions write-time, results read-time on `apply_lens`); **Constellation Map** (`map.rs::constellation_map_universe` walks filesystem on every open). Sky View now write-time post-MIG-001. |
| **No frontend test harness** | No vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`. Rust unit tests only: 11 in `cascade_walker_tests`, 6 in `canonical.rs`, 4 in `file_kinds.rs`. |
| **No help topic for Constellation Map** | Sky View has [`docs/help.uConstellation.World/Sky View/Sky View.md`](docs/help.uConstellation.World/Sky%20View/Sky%20View.md). |
| Versioning | All three (`package.json`, `tauri.conf.json`, `Cargo.toml`) at 0.3.4 today. |
| Orientation v1.0 — auto-update toggle placement | v1.0 bug §13 said the toggle was wrongly placed under "Sky View & Links" and should be elsewhere. **The actual UI section is "Sky View & Links" and that's correct** (it's a links-cascade behavior, not a files-management one). v1.2 corrects: toggle is **correctly placed**. |

---

## 13. Outstanding bugs / cosmetic issues

| ID | Status |
|---|---|
| **BUG-013** open-editor cascade race | Open. Documented limitation: switch tabs before renaming a target whose source is visible. |
| **BUG-014** orphan `cid_cn` (collateral from BUG-012) | Closed §118 (2026-04-25). |
| **BUG-015** target-body corruption from §115 value-sync `$effect` | Vector removed at §116 (`5afe0c2`). Forensics in `lab/forensics/`. |
| Title-heading rename gap | **CONFIRMED**: [`NoteEditor.svelte:179-204`](src/lib/components/NoteEditor.svelte:179) handler calls `renameItem(filePath, newPath)` only — does **NOT** call `updateLinksOnRename`. The cascade is gated only by file-tree rename ([+layout.svelte:3807-3808](src/routes/+layout.svelte:3807) — conditional on `$appSettings.autoUpdateLinks && !isDir`). |
| Sidebar active-item highlight ~10 s lag | **Origin unresolved.** No reactive source / debounce / async refresh found that accounts for the 10 s; further forensics needed when it next reproduces. |

### 13.1 Badge taxonomy

Canonical reference: [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md). Render sites (must stay in sync per the source-of-truth invariant):
- [`ConstellationMap.svelte:80-84`](src/lib/components/ConstellationMap.svelte:80) — `CAT_COLORS` map; rendered at line 660 (current result) and line 711 (result list).
- [`ConstellationSight2.svelte:79-83`](src/lib/components/ConstellationSight2.svelte:79) — `CAT_COLORS` map.

**What badges mean.** A badge tells the user **where in the note the search query matched** (or what kind of link relationship the result represents). One result can carry multiple badges.

**Content / structural matches** (where in the note the match occurred):

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **T** | Title | Blue | `#3b82f6` |
| **C** | Content (body text) | Green | `#16a34a` |
| **P** | Property (frontmatter key/value) | Amber | `#f59e0b` |
| **S** | Semantic (embedding similarity) | Purple | `#7c3aed` |
| **W** | Wikilink (`[[target]]`) | Grey | `#94a3b8` |
| **#** | Tag / Hashtag (`#tag` or YAML `tags:`) | Pink | `#f472b6` |
| **∅** | Empty / Null result | Slate | `#64748b` |

**Link-relationship badges** (matched by virtue of how the result links to/from the queried note):

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **LT** | Link **Target** (this note links *to* the queried note) | Green | `#16a34a` |
| **LF** | Link From (this note is linked *from* the queried note) | Red | `#ef4444` |
| **⇄** | Bidirectional (mutual link in both directions) | Violet | `#8b5cf6` |
| **LB** | Link Back (backlink hit) | Light blue | `#0ea5e9` |
| **LA** | Link Alias (matched via the link's display alias rather than its target) | Pink | `#d946ef` |
| **M** | Mutual link (the queried note links *to* the source AND the source links *back*) | Cyan | `#06b6d4` |

**Deprecated**:

| Badge | Status |
|---|---|
| **G** | Earlier identifier for Tag/Hashtag. Superseded by **#**. Not present in current code. |

**Unresolved**: none. M was the last pending letter; resolved 2026-04-27 as Mutual link.

**Adding a new badge**: see `docs/Badge-Taxonomy.md` § "Adding a new badge" — must update both `CAT_COLORS` maps in lock-step + this section + Badge-Taxonomy.md.

### 13.2 Filter chips on Constellation Map ([`ConstellationMap.svelte:114-125`](src/lib/components/ConstellationMap.svelte:114))

These are **search-syntax helpers**, not letter badges:
`linksTo` (`links to [[`) · `linksFrom` (`links from [[`) · `orphans` · `tag` (`#`) · `supports` (`supports [[`) · `contradicts` (`contradicts [[`).

### 13.3 Auto-update-links toggle path

**[`SettingsModal.svelte:1395-1428`](src/lib/components/SettingsModal.svelte:1395)** — under section `activeSection === 'skyview'` (display label "Sky View & Links"). Toggle binds to `$appSettings.autoUpdateLinks`. Cascade trigger ([`+layout.svelte:3807`](src/routes/+layout.svelte:3807)):

```
if ($appSettings.autoUpdateLinks && !isDir) {
  await updateLinksOnRename(lib.path, oldName, newName);
}
```

---

## 14. Where to read what (index)

| Topic | Source |
|---|---|
| Why Constellation exists / vision | [`docs/Constellation — Concept Paper.md`](docs/Constellation%20—%20Concept%20Paper.md) |
| **Sight — what it's for + analytical foundation + truth-status + v3 north star** | [`docs/Constellation-Sight-Concept-Paper-v1.1.md`](docs/Constellation-Sight-Concept-Paper-v1.1.md) (v1.1 markdown port + v3 forward-look) · v1.0 source: `docs/Constellation_Lens_Concept_Paper_Eisa.pdf` |
| **Sight v3 — visual + interaction specification (ratified 2026-05-07)** | [`docs/Constellation-Sight-v3-Concept-Paper-v1.1.md`](docs/Constellation-Sight-v3-Concept-Paper-v1.1.md) (v1.0 = draft; v1.1 = post-Eisa-design-review) |
| Map (radial sunburst) | [`docs/Constellation_Map_Concept_Paper_Eisa.pdf`](docs/Constellation_Map_Concept_Paper_Eisa.pdf) |
| Living Link philosophy + 8 properties + 7 types + 6 lifecycle stages | [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) |
| Cognitive Engine 16-phase spec | [`docs/CE-spec.md`](docs/CE-spec.md) + [`docs/cognitive-engine-roadmap.md`](docs/cognitive-engine-roadmap.md) |
| Canonical filename + 12 kinds + import pipeline | [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) |
| NotePane editor rules | [`docs/NotePane-spec.md`](docs/NotePane-spec.md) |
| Audit system (7 / 8 / 14) | [`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) + [`lab/audit-agents.md`](lab/audit-agents.md) |
| Migration four-phase workflow | [`.claude/skills/migration.md`](.claude/skills/migration.md) |
| PCS protocol | [`docs/PCS-PROTOCOL.md`](docs/PCS-PROTOCOL.md) |
| Working protocols / Tutorial Rule | [`docs/WORK-BEHAVIOR.md`](docs/WORK-BEHAVIOR.md) |
| Hard-won rules from real bugs | [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) (LL-001 → LL-023) |
| Migration plans | `lab/reports/MIG-NNN-*.md` |
| Active boot-perf budget | [`lab/boot-perf/BOOT-BUDGET.md`](lab/boot-perf/BOOT-BUDGET.md) |
| What's in flight today | `lab/reports/SESSION-LOG-{latest-date}.md` |
| Subsystem status snapshot | [`lab/reports/STATUS.md`](lab/reports/STATUS.md) |
| User-facing feature docs | `docs/help.uConstellation.World/<topic>/<topic>.md` (24 topics) |
| Master User Manual (English, 25 chapters) | [`docs/User Manual.md`](docs/User%20Manual.md) |
| 14 translated User Manuals | `docs/help.{ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}/User Manual.md` (ar = 1328 lines, others = 1120) |
| **Tauri command registry (authoritative)** | [`src-tauri/src/lib.rs:233-432`](src-tauri/src/lib.rs:233) |
| Tauri config / windows / CSP | [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) |
| Window permissions | [`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json) |
| Release workflow (CI) | [`.github/workflows/release.yml`](.github/workflows/release.yml) |
| Bases MVP | [`docs/BASES_MVP_SPEC.md`](docs/BASES_MVP_SPEC.md) |
| Badge taxonomy (canonical reference) | [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md) |
| eNotePane build history | `docs/eNotePane-development-record.md` + `lab/experiments/phase-N-*.md` |
| Forensic snapshots | `lab/forensics/` |

---

## 15. Session-start protocol

1. **`git pull origin main`** to sync.
2. **`git log --oneline -10`** for recent work.
3. **Read `lab/reports/SESSION-LOG-{latest-date}.md`**. Look for `§STATE-OF-STANDING`.
4. **Read THIS document** (`docs/Constellation Orientation & Onboarding vX.Y.md`).
5. **Read [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md)** — every rule was earned by a real bug.
6. **Read [`CLAUDE.md`](CLAUDE.md)** — top-principal rules + Working Agreement + Standing Orders.
7. **Read [`lab/reports/STATUS.md`](lab/reports/STATUS.md)** — one-page subsystem status index.
8. **Read memory files** at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\MEMORY.md` and linked entries.

If any contradict each other, ground in the code (`grep`) and update the stale doc in the same session.

### 15.1 Tools you'll need

- `gh` — GitHub CLI (release pipeline, PR ops).
- `git`, `npm`, `cargo`, `tauri` (`npm run tauri`).
- `sqlite3` for direct DB inspection (Rust side ships `rusqlite` bundled — no external sqlite3 required at runtime, but useful at dev time).

### 15.2 Boot pipeline summary

1. `paint:start` → `paint` (target ≤ 870 ms — Criterion 1) → app shell visible.
2. `cache_boot_snapshot_core` (note list, libraries, settings) — awaited.
3. `cache_boot_snapshot_graph` (links, tags, aliases) — deferred via `requestIdleCallback`.
4. `cache_boot_snapshot_sky` (pre-shaped sky_nodes + sky_links from triggers) — parallel with graph.
5. `boot:hydrated` (target ≤ 6 s — Criterion 2; achieved 811 ms).

### 15.3 Wikilink resolution + collision tiebreak

Three-tier resolution ([`cache.rs:553-588`](src-tauri/src/cache.rs:553)):
1. **`name_to_idx` hit** → use canonical id.
2. **`alias_to_path` hit** → resolve to canonical path → bump on canonical row.
3. **Unresolved** → fall back to lowercase comparison; orphan edge skipped.

**Tiebreak under collisions**:
- Two notes with identical title (case-insensitive): **Unresolved** — both match, no deterministic winner.
- Title equals another note's alias: **Name wins** (tier 1 precedes tier 2).
- Two notes share an alias: **First-write-wins** — `alias_to_path` is single-valued; insertion order undefined. Practical advice: avoid shared aliases.

---

## 16. Standing Order #6 (this document's maintenance contract)

Update this document in the same commit when:

- A migration starts, ships a step, or closes.
- A top-principal rule is added or reworded.
- A BUG-NNN opens or closes.
- A drift item from §12 is fixed (remove the row).
- A new LL-NNN is added.
- A boot-perf criterion changes or closes.
- A version bumps (`Cargo.toml`, `package.json`, `tauri.conf.json`).
- A subsystem ships a major feature.
- A help topic ships or restructures.

**Bump version (1.x → 1.y)** on structural changes. **Write the new version as a NEW file** in the same commit (filename always carries version suffix per §10.12). **Do NOT delete the previous version** — every version stays as a historical record. Date-stamp every section that updates.

The document **must remain readable in one pass.** If it grows past ~1500 lines, split into linked sub-documents in `docs/orientation/`.

---

## 17. What I (Claude) have NOT read in detail (v1.2 — significantly reduced)

This list is mandated by the BASIC RULE. If you need certainty on a claim that touches an "unread" file, **read it before acting**.

**Source code I have NOT read in full**:
- Some sections of `search.rs` (4790 lines), `libraries.rs` (3978) — read at section level, not line-by-line. Function signatures, schema, triggers, command surface confirmed.
- `+layout.svelte` (6872 lines) — structural map only (region table + $effect inventory + IPC list + component mount list). Not line-by-line.
- `libraries/+page.svelte` (704), `skills/+page.svelte` (219) — listed and counted, not read.

**Docs I have NOT read in full**:
- 14 translated User Manuals (parity confirmed: ar = 1328 lines, others = 1120; same chapter structure).
- `docs/User Manual.md` chapters beyond TOC + opening paragraphs.
- Binary docs (`docs/GraphMind*.docx`, `docs/constellation_cognitive_engine_v2.1.pdf`) — text tools cannot extract reliably.

**Resolved this session (2026-05-07)**:
- `docs/Constellation_Lens_Concept_Paper_Eisa.pdf` — read in full via `pypdf` extraction; content folded into the markdown port at `docs/Constellation-Sight-Concept-Paper-v1.1.md`. Removed from the "binary docs not read" list above.

**Session logs partially read**:
- 2026-04-18 (1.46 MB): structural digest + sampled headlines (Arabic Engine M3-M14 milestone day).
- 2026-04-19 (99 KB): structural digest.
- All 20 logs digested chronologically (see §11 / §15 / §16 references throughout this doc).

**Specifics I do NOT know**:
- **Sidebar active-item highlight ~10 s lag origin** — no reactive source / debounce / async refresh isolates the lag. Reproduce-and-instrument needed.
- **Why the alias-aware sky snapshot path (`cache_boot_snapshot_sky`) is bypassed at boot** in builds that contain MIG-001 / MIG-004 §8 / MIG-005. The §88 defensive fix neutralizes user impact, but the underlying "why" is unresolved.
- **Whether `2026-04-16.UNTRACKED-BACKUP.md` (3.8 KB) and the tracked `2026-04-16.md` (13 KB) diverge in content** — sizes differ; backup may be checkpoint or partial draft. No content-level diff performed.
- **Whether the SECTOR_THRESHOLD = 8 cut-off feels right at the boundary** (v1.9 §104). The hybrid layout flips from sector to ring-per-group when the largest typed-link group exceeds 8 notes. Below 8 the sector layout looks balanced; above, the rings layout. The threshold itself is arbitrary; if Boss reports flips happening at the wrong moment for their data, the constant is one edit. Right now no data point either way.
- **Visualisation-mode distinctness (Stage 2E, deferred)** — at v1.9 commit time Boss had not yet flagged the three modes (Atmospheric / Neural / Cosmic) as too similar after the §103/§104 changes. The mode-specific decorations were redesigned to differentiate (rotating ellipses vs faint dashed rings vs solid coloured rings + sector lines + rim labels), but it's not Boss-confirmed. Triage only if flagged in 2E retest.

**Resolved during v1.9** (folded into §4.2 row 12 above, removed from §17):
- *Actual `get_360_view` latency on 7,600-note Universe.* Boss reports "almost instantly". MIG-010 priority dropped to LOW.
- *Inspector 360 first-fetch empty-state UX.* Confirmed not jarring in practice — the IPC is fast enough that the empty state barely shows.

**Resolved this session (2026-04-27):**
- **M = Mutual link** (was unresolved badge letter through v1.3). Confirmed by project owner; folded into §13.1 + Badge-Taxonomy.md.
- **W = Wikilink** (was unresolved through v1.1). Resolved earlier via Badge-Taxonomy.md.

**Future maintainers**: when you read one of the above and confirm a fact, update §17 to remove it AND fold the verified fact into the relevant section above. Keep §17 honest.

---

*End of v1.58 (preserves v1.14 footer cadence: each version is a NEW file alongside its predecessors). Maintained per Standing Order #6.*

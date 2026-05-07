# Session log — 2026-05-07

**Cascade**: MIG-016 close-out → Sight Concept Paper v1.1 → Pending Jobs v1.4 → Orientation v1.55, all in one commit. Then MIG-017 (PJ-039) is next-up and PJ-038 (Sight v3 with own Concept Paper) follows.

---

## What landed today

| Artifact | Path | Purpose |
|---|---|---|
| MIG-016 §1F audit close-out (scope-narrowed) | `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` | Closes PJ-034 as Cancelled (partial-shipped). 0 P0, 0 P1, 1 P3 (mousemove handler iterates simLinks; moot once v2 disabled). |
| Sight Concept Paper v1.1 (markdown port + truth-status refresh + v3 north star) | `docs/Constellation-Sight-Concept-Paper-v1.1.md` | Ports Eisa's April 2026 v1.0 PDF to markdown. Adds: §0 "what this paper IS" disclaimer, §4 Principle 6 (reveal-on-demand), §12 truth-status matrix, §13 star-chart vision (Suwaidi reference), §14 v3 redesign with own dedicated Concept Paper. |
| Pending Jobs v1.4 | `docs/Constellation Pending Jobs v1.4.md` | PJ-034 closes Cancelled (partial-shipped). New: PJ-035 (content similarity TF-IDF), PJ-036 (layer peeling), PJ-037 (Map↔Sight integration), PJ-038 (Sight v3 build with own Concept Paper), PJ-039 (MIG-017 disable v2 — next-up). New status `Cancelled (partial-shipped)` added to vocabulary. |
| Orientation v1.55 | `docs/Constellation Orientation & Onboarding v1.55.md` | Bumps from v1.54. v1.55 preamble documents the close-out cascade. §8 Migrations table adds MIG-014, MIG-015, MIG-016, MIG-017 rows. §14 indexes the new Sight Concept Paper. §17 removes Lens PDF from "binary docs not read" (read in full this session via pypdf). |

## Why MIG-016 closed early

Eisa's directive 2026-05-07: **secure-don't-muddle.** v2 Sight (`ConstellationSight2.svelte` + the `lens_*` Rust modules + `constellation_sight_*` IPCs) is being **disabled** as a known-good fallback while v3 is built fresh on the **star-chart aesthetic** (Suwaidi northern-hemisphere chart reference). Continuing perf work on a view that's about to be shelved is wasted effort — except where it's inheritable into v3.

| Phase | Original scope | Disposition |
|---|---|---|
| §1A | `performance.mark` instrumentation | ✅ Shipped (`a0babbb`) — marks free-running, alerts removed in §1B (`62718f7`) |
| §1B | Edges-on-hover gate + neighborMap + hover/select filter | ✅ Shipped (`62718f7`) — Boss-test PASSED |
| §1C | sightWorker.ts (Louvain + gaps + profiles + bridges off main thread) | ❌ **Cancelled** — wasted on a disabled view |
| §1D | Post-paint prewarm | ❌ **Cancelled** — same reason |
| §1E | SQLite `sight_cache` | ⏸ **Deferred to PJ-038** — v3 will compute the same outputs and benefit from cross-session persistence |
| §1F | Three-agent audit | ✅ Scope-narrowed (this commit's audit doc) — inline-light vs. parallel-agents because surface area is two commits + one Boss-test |

## Why an honest delivery score for v2 Sight is ~70-80%

Confronted with Eisa's question "Do you think Constellation is what the paper claims?", I auditted §3.3 (three edge types) + §2.2 (six core mechanics) against shipping code:

**Shipped**: Brandes betweenness centrality, Louvain community detection, structural gap detection, universe-health metric (Modularity + Dominance + Entropy + Connectivity), wikilink edges, shared-tag edges, non-linear navigation (click-to-open), Knowledge Insights sidebar, reveal-on-demand (MIG-016 §1B).

**Not shipped** (and tracked as PJs in v1.4):
- **PJ-035** — content-similarity TF-IDF edges. *The* InfraNodus-defining mechanic (the "edges of latent meaning"). v2 cannot detect a gap between two clusters of *unlinked-but-related* notes.
- **PJ-036** — layer peeling. The "remove top-N centrality nodes and recompute" mechanic. Without this, MOC/index notes dominate centrality readings.
- **PJ-037** — Map ↔ Sight integration. The "Map diagnoses, Sight prescribes" loop is one-way at best today (each surface independent).

These three drove the v3 conversation. They're not patchable into v2 cleanly because v2's force-directed visual grammar fights each one. They drop into v3 with cleaner visual homes (Milky Way band for similarity, "hide brightest stars" toggle for peeling, two-up panel for Map↔Sight).

## Why a v3 redesign + own Concept Paper

Eisa's pivot 2026-05-07: looked at a 19th-century-style northern-hemisphere star chart (Suwaidi reference), proposed Constellation Sight should "interpret its core function as something similar to this image." The visual analogy is exact:
- Star magnitude → centrality
- Constellation territories → Louvain communities
- Constellation lines → wikilinks/shared-tag edges (rendered only when focused — Principle 6 made visual)
- Milky Way band → content-similarity density (PJ-035 absorbed cleanly)
- Calendar rim → time dimension (creation date, last-traversed, lifecycle stage band)
- Empty patches between constellations → structural gaps (Burt's structural holes idiom is *literally* the same)
- Dome of the sky as a whole → universe health visible at a glance

Force-directed layouts (v2's choice) re-run the simulation each session — the user can't build a spatial mental map. Star charts are stable: once the projection is computed, the same notes always sit in the same regions of the dome. **Spatial memory becomes a feature.**

Decision: build v3 fresh; preserve v2 as known-good fallback. v3 inherits the Rust analytics IPCs as-is; rebuilds the visualization layer entirely.

**v3 gets its own dedicated Concept Paper** (Eisa directive). The v1.1 paper is the *analytical foundation* both versions share; v3's paper is the *visual + interaction specification*. Read side-by-side when v3 work begins.

## Cascade order from here

1. **PJ-039 — MIG-017 (next-up)**: disable v2 Sight (mini-MIG, single session). Feature flag `sight.engine: 'v2-disabled' | 'v2' | 'v3'`, default `'v2-disabled'`. Hide dock button + modal + Settings entry. v2 component + IPCs stay on disk.
2. **PJ-038 — Sight v3 build** (multi-MIG, with own Concept Paper). Star-chart aesthetic. Inherits Rust analytics. PJ-035 / PJ-036 / PJ-037 absorbed as v3 features.

## Commits earlier in the cascade (carried)

| Commit | Phase / scope |
|---|---|
| `cb6c675` | PJ-034 / MIG-016 — Architect: Sight instant-toggle perf |
| `cd82976` | MIG-016 (PJ-034) Plan v1 — six-phase rollout |
| `a0babbb` | MIG-016 §1A — instrument toggleLens() + Sight2 mount with perf marks |
| `7e76b17` | MIG-016 §1A fix — clipboard + alert fallback (no DevTools needed) |
| `62718f7` | MIG-016 §1B — edges-on-hover gate in Sight + drop §1A alerts |
| (this) | MIG-016 §1F audit + Sight Concept Paper v1.1 + Pending Jobs v1.4 + orientation v1.55 |

## Decisions made

1. **MIG-016 closes — Cancelled (partial-shipped).** New status added to Pending Jobs vocabulary.
2. **v2 Sight to be disabled, not removed.** Component + IPCs stay on disk as known-good fallback. PJ-039 / MIG-017 is the disable mechanism.
3. **v3 Sight rebuilt fresh on star-chart aesthetic.** Force-directed force layout dropped in favour of stable 2D polar projection.
4. **v3 gets its own dedicated Concept Paper.** v1.1 paper covers analytical foundation; v3 paper will cover visual + interaction.
5. **Three v2 implementation gaps allocated** as PJ-035 / PJ-036 / PJ-037 — inheritable into v3 by design.
6. **Honest delivery score (~70-80%)** documented in v1.1 Concept Paper §12 truth-status matrix. Future Claude sessions read this before claiming Sight is feature-complete.

## At-risk / open

- **MIG-017 (PJ-039) implementation** — feature-flag mechanism not yet written. Need to verify: where does the dock button live? where does the modal mount? where does the Settings entry render? Plan + Build + Audit are next.
- **Annotation write-path gap** (carried) — link `annotation` field has no UI today; data model supports it. Track separately from v3 work.

## Known-broken (carried)

- `LinkLifecycle` dedupe in `store.ts:2298` — Option B approved 2026-05-01, deferred until post-CE.
- Pre-MIG-013 backups hit blocking v2 sentinel migration on libraries that haven't migrated yet. (Boss-equivalent libraries already migrated.)
- 6 MIG-014 §2F P2/P3 follow-ups (PJ-028 → PJ-033) — non-blocking; each acceptable as graceful degradation.

## Doc drift fixed today

- Lens → Sight rename completed in user-facing Concept Paper (markdown port v1.1). v1.0 PDF stays in `docs/` as historical record.
- Orientation §17 — `Constellation_Lens_Concept_Paper_Eisa.pdf` removed from "binary docs not read" (read this session via pypdf).
- Pending Jobs v1.4 status vocabulary — added `Cancelled (partial-shipped)` to handle PJ-034's clean exit shape.

## Next decision point

After this commit lands, run **MIG-017 Architect** for disabling v2 Sight. Single mini-MIG, single session. Then **PJ-038 Architect + own Concept Paper** for v3.

---

## End-of-day update — MIG-017 (PJ-039) CLOSED

After Eisa's "Proceed", cascaded through Architect → Plan → Build → Audit in one session. v2 Sight is now unreachable from the running app's user surface in default config; v2 component + IPCs preserved on disk as known-good fallback for v3 (PJ-038).

### Mechanism (Architect → Plan)

Single code constant `SIGHT_V2_ENABLED = false` in new module `src/lib/sight/engine.ts`. Gates four UI surfaces with `&& SIGHT_V2_ENABLED` (or conditional spread for the SettingsModal entry):

- Dock button — `src/routes/+layout.svelte:4361`.
- Modal mount + overlay class binding — `src/routes/+layout.svelte:4993-4994`.
- "Return to Lens" button — `src/routes/+layout.svelte:4741`.
- Settings → Plugins → Visualization plugin entry — `src/lib/components/SettingsModal.svelte:270`.

Plus a 🚧 banner prepended to `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md`; original v2 documentation untouched beneath.

### Why a code constant, not a Settings flag

The fallback is a *codebase* fallback, not a user-facing toggle. Settings flag would have required a one-time migration to flip existing users' saved `enabledFeatures.constellationSight: true` → `false`. Const-based gate wins regardless of saved state — zero churn, single source of truth, one-edit rollback.

### Audit (three agents in parallel)

All three agents (invariants / drift / migration-path) returned **CLEAN**. Audit report: `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`. 0 P0, 0 P1, 0 P2, 0 P3.

| Agent | Coverage | Verdict |
|---|---|---|
| **Invariants** | All 12 invariants from Architect §6 + extra entry-point scan | ✅ CLEAN |
| **Drift** | Implicit consumers, CSS overlay collapse, cross-surface touchpoints, naming forward-compat | ✅ CLEAN |
| **Migration-path** | All 5 scenarios from Architect §8 + 2 extra paths (`lensReturnPending` entry points, v3 coexistence) | ✅ CLEAN |

### Commits today

| Commit | Phase / scope |
|---|---|
| `94c4331` | MIG-016 closes — Cancelled (partial-shipped) + Sight Concept Paper v1.1 + Pending Jobs v1.4 + orientation v1.55 |
| (this) | MIG-017 closes — disable v2 Sight + Pending Jobs v1.5 + orientation v1.56 |

### Decisions made

1. **Code constant, not Settings flag**, for the v2-disable mechanism. Rationale documented in Architect §4.1.
2. **Belt-and-suspenders gating**: dock button + modal mount + "Return to Lens" button + Settings entry all gate independently. Defense-in-depth against any stray code path.
3. **Help-doc banner is non-destructive** — original v2 documentation preserved beneath, since it remains accurate for the v2 component on disk.
4. **No Boss test gate this MIG** — UI-hide MIG, not feature MIG. Audit replaces Boss test for verification.
5. **v3 forward-compat baked in**: const naming (`SIGHT_V2_ENABLED`) explicitly accommodates a future `SIGHT_V3_ENABLED` const in the same file. No `enabledFeatures` field collision when v3 ships.

### State at end of day

- **MIG-016 (PJ-034)** — Cancelled (partial-shipped). §1A + §1B in production.
- **MIG-017 (PJ-039)** — CLOSED. v2 Sight disabled cleanly.
- **PJ-038 (Sight v3 build with own Concept Paper)** — UNBLOCKED. Top of queue.
- **Done count**: 7 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027, PJ-039). Cancelled: 1 (PJ-034 partial-shipped).
- **Documentation aligned**: orientation v1.55 → v1.56; Pending Jobs v1.4 → v1.5; v2 help doc has the 🚧 banner.

### Tomorrow's first move

Whatever Eisa picks. PJ-038 Architect + dedicated Concept Paper for v3 Sight is the next logical step — multi-MIG with star-chart aesthetic per the v1.1 paper §13–§14 vision. PJ-035 / PJ-036 / PJ-037 absorb into v3 design.

---

## End-of-day update — PJ-038 Concept Paper v1.0 drafted

After "go," drafted the dedicated v3 Concept Paper. Single artifact: `docs/Constellation-Sight-v3-Concept-Paper-v1.0.md` (~600 lines, 14 sections + glossary).

### What's in the paper

- **§1 vision**: the 19th-century printed-star-chart reference; the at-a-glance read.
- **§2 visual grammar**: nine star-chart elements mapped row-by-row to their Sight semantics (star → note, magnitude → centrality, territory → community, connector lines → wikilinks, Milky Way → similarity, label → community-profile, calendar rim → time, empty patches → structural gaps, dome → universe health).
- **§3 projection method**: two-stage pipeline (spectral embedding → Lambert azimuthal equal-area). Determinism-per-snapshot caching in SQLite (mirrors deferred MIG-016 §1E design). Edge cases (disconnected components, tiny universes, dominant-community case).
- **§4 interactivity**: nine interaction modes — resting / hover star / click star / double-click / hover territory / click territory / search / calendar rim / right-click.
- **§5 absorbing the deferred PJs**: PJ-035 → Milky Way, PJ-036 → magnitude slider, PJ-037 → two-up panel.
- **§6 universe-health at a glance**: how the four metrics (M / D / E / C) become visually obvious from dome shape.
- **§7 internationalization**: RTL, Arabic constellation labels, Hijri rim.
- **§8 performance**: latency budgets + three-layer rendering (SVG + Canvas + DOM) + SQLite cache + idle-prewarm.
- **§9 phased rollout**: three MIGs (MIG-018 projection foundation; MIG-019 density + time + search; MIG-020 layer peeling + Map↔Sight + v2 retire). Each its own four-phase migration cycle. Each Boss-test gated.
- **§10 out of scope**.
- **§11 ten open questions for Eisa**: design calls needed before code begins (spectral vs. MDS, Lambert vs. stereographic, Hijri rim, magnitude slider direction, two-up default, label visibility, color scheme, search persistence, render layer, accessibility timing).
- **§12 acceptance criteria**.
- **§13 glossary**, **§14 cross-references**.

### Why a paper before any code

Eisa's directive 2026-05-07: *"v3 shall get its own dedicated Concept Paper."* The v1.1 paper covers the *analytical foundation* both versions share; v3's paper covers the *visual + interaction grammar* that's new. Read side-by-side, they form the design contract for the multi-MIG build.

### What's next

**Awaiting Eisa's design review on §11.** Ten design questions need calls before MIG-018 opens. None blocks the others; Eisa can answer in any order. Once §11 resolves, the paper bumps to v1.1 with the design choices baked in, and MIG-018 Architect opens.

If Eisa wants to refactor §1-§10 of the paper itself (e.g., reject the star-chart aesthetic for a different visual grammar), that's the moment — before any code. Cheap to redirect now; expensive after MIG-018 ships.

---

## End-of-day update — v3 Concept Paper v1.1 ratified

Eisa's design review came back with comprehensive answers. All ten §11 questions resolved + two structural revisions made beyond §11 scope.

### §11 design calls

| # | Decision |
|---|---|
| 1 | Embedding: graph-distance MDS (Landmark variant) |
| 2 | Projection: BOTH Lambert + stereographic ship; user toggle |
| 3 | Calendar rim: Gregorian default; users add others via Settings |
| 4 | Magnitude slider: astronomy convention (drag right = peel) |
| 5 | Two-up panel: N/A (PJ-037 rejected) |
| 6 | Constellation labels: hover/select only by default; Settings toggle for always-on |
| 7 | Color scheme: cycled pastels by Louvain id; user-overridable via Style Settings |
| 8 | Search filter persistence: Esc + click-background |
| 9 | Render layer: Pixi.js |
| 10 | Accessibility: defer to post-v3 PJ |

### Structural revisions beyond §11

1. **§4.1 resting state**: connector lines now **faint at rest, brighten on hover/select** (replaces v1.0's "no lines until hover"). Eisa's directive: *"with v3, we will show it as faint lines until the user hovers over it or the connected nodes linking them."* This reframes v1.1 paper Principle 6 — *reveal* now means *brighten*, not *render-from-zero*. Triggers a §8.2 rendering split: two Pixi layers (base = always-on faint structure, focus overlay = brightening on hover) so per-frame draw cost stays near zero at rest while the structural pattern is always visible.

2. **§5.3 Map↔Sight integration: REJECTED**. Eisa: *"There won't be Map-Sight integration."* PJ-037 marked Rejected in Pending Jobs v1.6; number retired. v3 stays single-view; Map and Sight remain independent surfaces. The "Map diagnoses, Sight prescribes" loop happens in the user's head.

### Artifacts in this commit

- `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` — ratified Concept Paper. v1.0 stays in `docs/` as historical record.
- `docs/Constellation Pending Jobs v1.6.md` — PJ-037 Rejected; PJ-038 §8 trajectory revised; new "Rejected" count tracked.
- `docs/Constellation Orientation & Onboarding v1.57.md` — bump from v1.56 with v3 ratification preamble + new index entry for the v3 Concept Paper.

### Next move

**MIG-018 Architect**. The first of three v3-build MIGs. Scope: Rust `compute_layout_embedding` (Landmark-MDS variant), `sight_v3_layout` SQLite cache + write-time triggers, `src/lib/sight/projection.ts` + `SightV3.svelte`, dock button + Settings entry behind `SIGHT_V3_ENABLED`, both Lambert + stereographic projections. Boss-test gate at end: stars render at correct positions, basic hover/click/double-click works, projection toggle works.

Per Migration Rule, after MIG-018 Architect lands I'll write the Plan and **stop for Eisa's explicit Plan approval before Build**.

---

## End-of-day update — MIG-018 CLOSES (PJ-038 phase 1 of 3)

After Eisa's Plan approval ("§1A Approved" + 2 refinements) at ~mid-day, cascaded through six phases (§1A → §1F) in one session. v3 projection foundation is **live in production**.

### Eight commits today on the v3 trajectory

| Commit | Phase / scope |
|---|---|
| `1164b08` | PJ-038 Concept Paper v1.0 drafted (awaiting design review) |
| `44c37c9` | PJ-038 Concept Paper v1.1 ratified + PJ-037 rejected + Pending Jobs v1.6 + orientation v1.57 |
| `51e270a` | MIG-018 Architect + Plan |
| `fe85792` | MIG-018 §1A — schema + sight_layout.rs skeleton + IPC registered |
| `24aa6bd` | MIG-018 §1B — Landmark-MDS compute + persistence + invalidation IPC (5 unit tests passing) |
| `dd6759e` | MIG-018 §1C — frontend skeleton + dock button + Settings entry + i18n 15 locales |
| `4dc6878` | MIG-018 §1D — star rendering + Lambert/stereographic projection toggle + Settings → Sight section |
| `26ce36e` | MIG-018 §1E — territories + faint connector lines + hover/click + side panel + Suwaidi palette [Boss-test gate] |
| (this) | MIG-018 §1F — audit + Pending Jobs v1.7 + orientation v1.58 + SIGHT_V3_ENABLED=true committed |

### What v3 looks like now

Click the new star-icon dock button. The screen fills with a deep midnight-blue dome of stars at deterministic Landmark-MDS positions. Constellation territories are soft Suwaidi pastel regions (warm-cream / gold / amber / dusty rose / sandy tan / parchment / antique-white / dark goldenrod cycled by Louvain community id). Faint connector lines weave between stars — visible structure that doesn't shout. Hover a star: tooltip + incident edges brighten + gold ring around the focus. Click: that constellation lights up + side panel slides in with title, community, centrality rank, connection counts, "Open in editor" button. Double-click: opens the note. Settings → Sight → Projection: switch Lambert ↔ Stereographic; the dome reshapes; same notes return to remembered positions on the next toggle.

### Boss test (§1E gate)

Eisa flipped `SIGHT_V3_ENABLED = true` locally, ran the install, walked through all 11 steps. **Report: "All pass"**.

### Three-agent audit (§1F)

All three agents (invariants / drift / migration-path) returned **CLEAN**. 0 P0, 0 P1, 0 P2, 0 P3. Audit report: `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md`. Verified all 13 invariants from Architect §3, the drift map (zero implicit consumers), the seven migration-path scenarios, and four extra paths.

### Decisions made (recap)

1. **Graph-distance MDS** chosen over spectral embedding (§11 Q1).
2. **Both Lambert + stereographic** projections ship; user-toggle (§11 Q2).
3. **Calendar rim**: Gregorian default; users add others via Settings (§11 Q3) — UI lands in MIG-019.
4. **Magnitude slider direction**: astronomy convention (right = peel) (§11 Q4) — implementation in MIG-020.
5. **Two-up panel**: N/A — PJ-037 rejected (§11 Q5).
6. **Constellation labels**: hover/select only by default; Settings toggle for always-on (§11 Q6).
7. **Color scheme**: cycled Suwaidi pastels by Louvain id (§11 Q7) — implemented this MIG.
8. **Search filter persistence**: Esc + click-background (§11 Q8).
9. **Render layer**: Pixi.js (§11 Q9) — implemented this MIG.
10. **Accessibility**: deferred (§11 Q10).
11. **Faint lines at rest** (Eisa-directed beyond §11): the v3 reframe of v1.1 paper Principle 6 — *reveal* now means *brighten*, not *render-from-zero*.
12. **`enabledFeatures.constellationSightV3`**: fresh field name (NOT reuse v2's `constellationSight`) — Eisa's call.

### State at end of day

- **PJ-038 (Sight v3 build)**: **In-Progress** — 1 of 3 MIGs done.
- **MIG-018**: closed Done.
- **MIG-019 (next-up)**: density (PJ-035 Milky Way) + calendar rim + universe-health card + full search integration.
- **MIG-020 (after MIG-019)**: layer peeling (PJ-036) + v2 retire.
- **Done count**: 7 (unchanged). **Cancelled**: 1 (PJ-034). **Rejected**: 1 (PJ-037).
- **Documentation aligned**: orientation v1.55 → v1.58 (4 bumps today); Pending Jobs v1.4 → v1.7 (3 bumps today); v3 Concept Paper v1.0 + v1.1 + audit report all in `docs/` and `lab/reports/`.

### Tomorrow's first move

Whatever Eisa picks. **MIG-019 Architect** is the natural next step — density + time + search + universe-health card on the v3 base. MIG-019 is single MIG, multi-phase build; expect ~6 phases similar to MIG-018's shape.

---

## MIG-019 §2G — Polar layout redesign (in flight, evening of 2026-05-07)

After §2A–§2F shipped (density-grid IPC replacing the OOM-prone edge list, OOM solved at 7,636-note scale), Eisa critiqued the visual: doesn't resemble celestial hemisphere, calendar rim doesn't scale years, Universe Health card location is a distraction. Pivot from code to design: four mockup options generated (`docs/Constellation-Sight-v3-mockup-{A,B,C,D}-*.svg`) on Suwaidi cream palette. Eisa picked Option A (polar grid).

Then, three iterations of refinement:
1. **Rim axis is switchable** (Eisa's idea): the chart becomes a multi-lens diagnostic. Rim toggles between Regions / Link Types / Time / Confidence / Stages / Acts (R/L/T/C/S/A). Radius (centrality) and color (community) stay invariant; only azimuth changes.
2. **Time becomes its own mode** (T) — no orthogonal year ladder needed.
3. **Universe Health → top-center, metrics flanking the roundel** (not below). The dome edge is clean.
4. **Edges hidden in resting state**, revealed on hover/click of a node (Concept Paper §4.1).

Final mockup: `docs/Constellation-Sight-v3-mockup-A2-toggle.svg`. Approved.

### §2G.1 — Visual spec doc (in this commit)

`docs/SIGHT-V3-VISUAL-SPEC.md` codifies the design contract. 10 sections covering: 6 modes & their wedge labels, polar grammar, edge resting/active states, Universe Health anchor, toggle UI (production = compact letter-button bar, top-right), Suwaidi palette tokens, 10 layout invariants, deferred items, phase mapping, acceptance criteria.

The three approved defaults:
1. Rim labels = uniform Suwaidi blue ink (`#2a4a8c`); star colors per-community.
2. Wedge order = by note count, largest first.
3. Production toggle = compact letter-button bar (R/L/T active; C/S/A "available later"); full preview strip from mockup is exposition only.

### §2G plan (cascade per Plan Approval = Build Approval)

| Phase | Work | User-test stop? |
|-------|------|-----------------|
| §2G.1 | Visual spec doc | No |
| §2G.2 | Pure helpers (polar.ts, modes.ts, regions.ts) | No |
| §2G.3 | SightV3.svelte rewrite | **YES** — Boss tests visual |
| §2G.4 | Mode toggle + 600 ms migration animation | **YES** — Boss tests animation |
| §2G.5 | Mode persistence (`appSettings.sight.lastMode`) | **YES** — Boss tests persistence |
| §2G.6 | Three-agent audit + orientation v-bump + tag | No |

Architectural-impact scan complete (Explore agent): SightV3.svelte is isolated (1180 lines, only consumed by `+layout.svelte`); v2 is independent; settings persistence pattern is established; library color is NOT centrally stored (sidestepped via uniform blue ink labels).

### §2G.1 SHIPPED (`bfb8aba`)

`docs/SIGHT-V3-VISUAL-SPEC.md` written — design contract codifying the approved mockup. Five mockup SVGs + the original mockup checked in. `lab/sight-v3-mockup-generator.py` shipped. Three approved defaults locked: rim labels = uniform Suwaidi blue ink; wedge order = note-count desc; production toggle = compact letter-button bar (full preview strip is design exposition only).

### §2G.2 SHIPPED (`b1a2477`)

Three pure helper modules in `src/lib/sight/v3/`:
- `polar.ts` — polar math, magnitude buckets, palette tokens, dome ratios, animation easing
- `modes.ts` — 6 rim-axis modes (R · L · T ready; C · S · A "available later" per Concept Paper §6.3)
- `regions.ts` — `buildRegionLayout()` for library wedge sizing (sorted by note count, empty wedges compressed, Windows/Unix path-prefix tolerant)

Type-checked clean. Stateless. No allocations on the hot path.

### §2G.3 → §2G.3c SHIPPED (`7d6fcf6`)

Four phases of in-flight rewrites rolled into one commit so the working tree stops carrying ~5 hours of uncommitted code.

**§2G.3** — Core polar layout rewrite:
- Theme: navy → cream parchment; near-black stars; gold/cyan reference rings.
- MDS `embedToScreen()` replaced by polar positioning (radius=centrality, azimuth=mode wedge).
- `drawRegionRim()` ships library wedges with blue-ink tangent labels.
- `drawEdges()` empty by default — focus overlay handles hover/click reveal in gold.
- Universe Health → HTML overlay anchored top-center.

**§2G.3a** — After Eisa's first Boss test (PASS):
- Disabled `drawTerritories()` — community hulls span the whole dome in polar layout, made a tan blob hiding stars.
- `SightV3SidePanel` Universe Health section removed; panel opens only on star selection.
- Metric labels switched from `$t()` to literal English uppercase (i18n keys land in §2G.6).

**§2G.3b** — After Eisa's second Boss test:
- Calendar-rim hover/click handlers gated on `currentMode === 'time'`. Mode is user-driven only — Eisa caught the rim auto-shifting from Regions to calendar months on hover.
- `getViewport()` reserves 270px top + 100px bottom + 100px sides + 80px outside-dome. Dome center shifts down to middle of AVAILABLE space.
- Region rim labels migrated from Pixi Text → HTML overlay with `dir="auto"`. Browser handles bidi natively for Arabic / Hebrew / mixed-script library names.

**§2G.3c** (partial; finish + ship in next commit):
- Centrality RANK PERCENTILE for radius (was direct centrality_norm; real graphs are skewed → most stars packed at rim).
- Per-wedge `maxWidthPx` emitted in rim label geometry (ellipsis CSS pending).
- Universe-name header (above dome, below Health metrics) pending.

### §2G state-of-standing as of 2026-05-07 ~21:50 (SO #5 record before §2G.3d)

**Major architectural pivot during this phase:** Eisa proposed and approved a per-mode (X, Y, Z) grammar where each rim mode declares its own azimuth / radius / magnitude variables — turning Sight from a single chart into a multi-instrument cognitive lens (the §5.3 stethoscope/MRI/ECG metaphor from the Concept Paper). Color stays invariant (community Louvain).

**(X, Y, Z) approved per mode:**
| Mode | X (azimuth) | Y (radius) | Z (magnitude) |
|------|-------------|------------|---------------|
| R · Regions | Library | Centrality rank | Degree |
| L · Link Types | Dominant outgoing link type | Type diversity | Total outgoing links |
| T · Time | Creation date wedge | Recency (last edit) | Age |
| C · Confidence | Dominant confidence | Certainty homogeneity | Total link count |
| S · Stages | Dominant lifecycle stage | Avg link weight | Total traversal count |
| A · Acts | Which Act | Synthesis depth | Total connections |

**Verified shipped + protected:**
- §2G.1: visual spec doc + mockups (`bfb8aba`, pushed)
- §2G.2: pure helpers (`b1a2477`, pushed)
- §2G.3 → §2G.3c partial: SightV3 rewrite (`7d6fcf6`, pushed)
- 10 prior MIG-019 §2A → §2E commits also pushed (origin/main was 36h stale)

**At-risk / in-flight (uncommitted):**
- §2G.3c finish: rim label ellipsis CSS, Universe-name header markup
- §2G.3d: (X, Y, Z) per-mode grammar refactor + spec doc v1.0 → v1.1
- §2G.4: mode toggle UI + 600ms migration animation
- §2G.5: mode persistence
- §2G.6: 3-agent audit + orientation v-bump + tag

**Known-broken:** none currently observed in the §2G.3b build. Two Boss tests passed.

**Pending but not started:** MIG-020 (layer peeling + v2 retire) after MIG-019 closes.

**Documentation drift:**
- Orientation still at v1.58 — hasn't bumped for §2G in flight. Will bump to v1.59 in this catch-up commit.
- MoCh: last `MoCh-2026-05-06-0900.md`; today's been ~5+ hours of direct chat — fresh MoCh due.

### Next session resumes at

§2G.3c finish (rim label ellipsis CSS + Universe-name header markup) → cascade into §2G.3d (X/Y/Z refactor) → build → user tests → §2G.4 (toggle + animation) → user tests → §2G.5 (persistence) → user tests → §2G.6 (audit + close).

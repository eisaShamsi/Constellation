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

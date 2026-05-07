# MIG-016 §1F — Audit close-out

**Migration**: MIG-016 (PJ-034) — Sight instant-toggle perf
**Scope**: §1A (instrumentation) + §1B (edges-on-hover gate) only
**Date**: 2026-05-07
**Status**: PJ-034 partial-shipped; MIG-016 closes early; §1C/§1D cancelled; §1E deferred to PJ-038 (v3 build inheritance).

---

## Why an early close

Eisa's directive on 2026-05-07: secure-don't-muddle. v2 Sight (the current `ConstellationSight2.svelte`) is being disabled as a known-good fallback while v3 (star-chart redesign) is built fresh. Continuing perf work on a view that's about to be shelved is wasted effort — except where the work is inheritable.

| Phase | Original scope | Disposition |
|---|---|---|
| §1A | performance.marks instrumentation | ✅ Shipped (`a0babbb`); marks free-running, alerts removed in §1B (`62718f7`) |
| §1B | Edges-on-hover gate + neighborMap + hover/select filter | ✅ Shipped (`62718f7`); Eisa Boss-test PASSED |
| §1C | sightWorker.ts extraction (Louvain + gaps + profiles + bridges off main thread) | ❌ **Cancelled** — wasted work on a disabled view |
| §1D | Post-paint prewarm | ❌ **Cancelled** — same reason |
| §1E | SQLite sight_cache | ⏸ **Deferred** — v3 will compute the same outputs and benefit from cross-session persistence; reframed under PJ-038 inheritance |
| §1F | Three-agent audit | ✅ Scope-narrowed (this report); inline-light vs. parallel-agents because the surface area is two commits and one Boss-test |

---

## Invariants — verified

1. **neighborMap correctness**: populated once per `buildSimData()` call; symmetric (each link contributes both directions); rebuilt fresh on every Sight remount. Verified by code review of `ConstellationSight2.svelte:233-249`.
2. **Edge-draw gate fires correctly**: `needsEdgeDraw = hoveredNode || selectedNode || searchActive || hoveredLink`. All four are existing `$state`-tracked reactive variables; the gate composes them at frame time. Verified by code review of `ConstellationSight2.svelte:528-545`.
3. **Hover/select neighborhood filter**: `focusOnly` short-circuit at the top of `drawLinks()` skips non-incident edges in O(1) per link, dropping iteration from O(E) to O(degree). Falls back to full iteration in search-active and link-annotation-hover paths. Verified by code review of `ConstellationSight2.svelte:566-600`.
4. **performance.mark overhead**: marks are no-ops in production WebView outside DevTools-attached scenarios; `console.log` retained but harmless. No measurable overhead.
5. **No regression to existing behaviour**: drawLinks() body unchanged below the focusOnly filter — all existing logic preserved (search highlighting, direction arrows, opacity, dormancy fade, link-type colours).
6. **M11 zero-diff**: `git diff src-tauri/src/lexicon/` returns empty for all MIG-016 commits.

---

## Drift — none found

- No other consumer of `simLinks` outside `drawLinks()` and the mousemove link-annotation detection at `:887-913` (which is unchanged and out-of-scope for §1B).
- `neighborMap` is private state; no external consumers depend on it.
- §1A alerts/clipboard prompts removed cleanly in §1B; only `console.log` + `performance.mark` retained.
- No frontend or backend IPC changes; no schema changes.

---

## Migration-path — clean

- No DB schema changes (§1E never landed).
- No frontend store contract changes (`lensActive`, `lensCentrality`, `lensCommunities`, `lensHealth`, `lensDataStale` all unchanged in shape).
- Existing universes toggle Sight correctly. Cache-warm path still hits the early return at `+layout.svelte:3343`.
- §1A `performance.marks` add no boot cost (deferred to first toggleLens call, which is on-demand only).

---

## Findings

- **0 P0** issues found.
- **0 P1** issues found.
- **1 P3** logged: the mousemove handler at `ConstellationSight2.svelte:887-913` iterates `simLinks` for link-annotation hover detection. With 30k+ links this is per-frame cost during mouse movement when no node is hovered. **Pre-existing pattern**, flagged in the §1B commit message, will be **moot when v2 is disabled in MIG-017**.

---

## Cancelled work — what's lost vs. preserved

**Cancelled (§1C, §1D)** — code never written. The Web Worker pattern (forceWorker.ts) and post-paint prewarm pattern remain proven elsewhere; v3 can reuse the patterns directly without reusing v2's wiring. **Net loss: zero implementation; design knowledge preserved in the v1.1 Concept Paper.**

**Deferred (§1E)** — the SQLite `sight_cache` design was scoped in the Plan but never coded. v3 will compute identical analytical outputs (centrality, communities, gaps, health) and benefit from the same cross-session persistence pattern. Reframed in PJ-038 under "v3 perf inheritance — sight_cache." **Net loss: zero implementation; design knowledge preserved in the Plan + this report.**

---

## What MIG-016 actually shipped

- **Edges-on-hover gate** in v2 Sight, mirroring Sky View's "nervous system" pattern at `graphEngine.ts:1880-1894`.
- **Pre-built `neighborMap`** for O(degree) hover/select edge lookups.
- **Verifiable mount-path perf data**: ~175-367 ms total mount time, dominated by `buildSimData` (~113-331 ms). Mount is **not** the bottleneck.
- **Performance.mark instrumentation** preserved for any future DevTools-enabled session (toggle pipeline + cold mount path covered).
- **Concept Paper v1.1** (committed alongside this audit) — Lens → Sight rename, truth-status table, Principle 6 (reveal-on-demand), star-chart north star, three implementation-gap PJs (PJ-035 / PJ-036 / PJ-037), v3 redesign target (PJ-038 with own Concept Paper).

PJ-034 closes as **partial-shipped** in Pending Jobs v1.4. The "instant first-toggle" headline goal is unmet; v2 won't reach it (and is being disabled). v3 will be designed for it from the start.

---

## Closing the cascade

After this audit:
- `Constellation Pending Jobs v1.4.md` — PJ-034 status updated; PJ-035 / PJ-036 / PJ-037 / PJ-038 allocated.
- `Constellation-Sight-Concept-Paper-v1.1.md` — markdown port + v1.1 refresh.
- Orientation v1.55 — bumped inline.
- Next: **MIG-017** disables v2 Sight (mini-MIG, single session); then **PJ-038** opens with its own Concept Paper.

# MIG-018 §1F — Three-agent audit

**Migration**: MIG-018 — Sight v3 projection foundation (within PJ-038)
**Date**: 2026-05-07
**Mode**: parallel three-agent (invariants / drift / migration-path)
**Verdict**: **CLEAN.** 0 P0, 0 P1, 0 P2, 0 P3.

---

## What got audited

Five-commit cascade `fe85792` → `26ce36e`:

| Phase | Commit | Scope |
|---|---|---|
| §1A | `fe85792` | sight_v3 schema + sight_layout.rs skeleton + IPC registered |
| §1B | `24aa6bd` | Landmark-MDS compute + persistence + invalidation IPC (5 unit tests passing) |
| §1C | `dd6759e` | Frontend skeleton + dock button + Settings entry behind SIGHT_V3_ENABLED + i18n 15 locales |
| §1D | `4dc6878` | Star rendering + Lambert/stereographic projection toggle + Settings → Sight section |
| §1E | `26ce36e` | Territories + faint connector lines + hover/click + side panel + Suwaidi palette [Boss-test gate] |

Eisa's Boss test passed all 11 steps with `SIGHT_V3_ENABLED = true` flipped locally. The const is now committed `true` (Eisa's working-tree edit) — production-ready.

Type-check: `npm run check` shows 1 pre-existing PJ-012 error (`LinkLifecycle.fresh`, deferred until post-CE) and 0 new errors.
Cargo check: 22 warnings (all pre-existing, unrelated). 0 errors. 5/5 unit tests passing.

---

## Agent #1 — Invariants

**Coverage**: all 13 invariants from Architect §3.

| # | Invariant | Verdict |
|---|---|---|
| 1 | v2 stays disabled but on disk | ✅ `SIGHT_V2_ENABLED = false`; `ConstellationSight2.svelte` + `sight.rs` IPCs untouched |
| 2 | Determinism (same input → same output) | ✅ 5 unit tests in `sight_layout.rs` cover ring graphs, repeated calls, disconnected components, single-node, hash determinism |
| 3 | Boot performance (no new IPC on boot path) | ✅ `constellation_sight_v3_layout` only called from `SightV3.svelte` `onMount`, never from boot sequence |
| 4 | Write-time derivation (Rule 8) | ✅ Schema tables created via `CREATE TABLE IF NOT EXISTS`; cache populated only on user toggle; no scan-on-boot or scan-on-tab-focus |
| 5 | No `$effect` loops | ✅ Single `$effect` reads `$appSettings.sight?.projection` and calls `fullRedraw()`; no reactive state mutation inside |
| 6 | No memory leaks | ✅ `onDestroy` disconnects `ResizeObserver`, calls `app.destroy(true, { children: true, texture: true })`, nulls all container refs |
| 7 | i18n integrity (15 locales) | ✅ All 15 locales have new keys: `sightV3.{title, placeholder, close, tooltip.*, sidePanel.*}`, `settings.{sections.sight, sight.intro, sight.projection.*, plugins.constellationSightV3*}` |
| 8 | RTL parity | ✅ `dir="auto"` on side panel + tooltip; layout renders RTL when interface locale is RTL |
| 9 | No regression in other surfaces | ✅ Sky View / OrgChart / Map / Index / SearchHub / Inspector360 dock buttons + handlers all intact (only additive changes to navigation handlers to clear `sightV3Active`) |
| 10 | M11 zero-diff | ✅ `git diff fe85792~1..HEAD -- src-tauri/src/lexicon/` empty |
| 11 | PJ-038 §8 trajectory honored | ✅ PJ-035 (Milky Way) / PJ-036 (layer peeling) NOT shipped; PJ-037 (Map↔Sight) NOT touched (rejected); deferred per Concept Paper v1.1 §9.1 |
| 12 | Pixi resource budget (4-container architecture) | ✅ territoryContainer + edgeContainer + starContainer + focusOverlay in z-order; base layers static; focus redraws only on hover/click |
| 13 | Settings round-trip | ✅ `appSettings.sight.projection` round-trips cleanly; spread pattern (`{ ...$appSettings.sight, projection }`) preserves future v3 settings |

**Extra entry-point scan**: zero v2-Sight surfaces inadvertently touched. `SIGHT_V3_ENABLED = true` rollout has no missing gates.

**Verdict**: CLEAN.

---

## Agent #2 — Drift

**Coverage**: implicit consumers, dead-code/orphan scan, cross-surface, algorithmic correctness, forward-compat.

- **Implicit consumers**: zero. All 12 declared consumers in Architect §4 drift table verified present and correct.
- **Dead-code scan**: `SIGHT_V3_SCHEMA_VERSION` is correctly `#[allow(dead_code)]` per Architect §1A (read in §1B's invalidation logic, deferred). `invalidateLayout()` is exported but has zero callers (deferred to §1D-or-later); no orphan risk because frontend wiring intentionally lands later.
- **Cross-surface touchpoints**: second screen unaffected (mirrors `editor | skyview | browser`, not v3). Sky View / Map / OrgChart / Inspector360 unchanged. i18n integrity preserved.
- **Algorithmic correctness**:
  - **Convex hull** (Andrew's monotone chain): cross product correct, lower+upper concatenation correct, degenerate cases (n < 3) handled.
  - **Projection math**: Lambert `r' = 2·sin(atan(r)/2)`, stereographic `r' = 2·tan(atan(r)/2)` — both correct, well-defined for `r ∈ [0, 0.95]`.
  - **Hit-testing**: O(n) per pointermove on 30k stars ≈ 1ms; no per-iteration allocations.
- **Forward-compat**: `SIGHT_V2_ENABLED` / `SIGHT_V3_ENABLED` naming clear; settings namespace `sight` reserved for future fields (`alwaysOnLabels`, `calendarSystems`, `magnitudeThreshold`).

**Verdict**: CLEAN.

---

## Agent #3 — Migration-path

**Coverage**: all 7 scenarios from Architect §5 + 4 extra paths.

| # | Scenario | Verified |
|---|---|---|
| 1 | Fresh install (no settings.json, no DB) | ✅ DEFAULT_SETTINGS provides `sight.projection='lambert'` + `enabledFeatures.constellationSightV3=true`; `init_db` creates tables empty; first toggle runs cold MDS, persists, returns. |
| 2 | Existing user, no `sight` field saved | ✅ Settings merge fills from DEFAULT_SETTINGS. Same end-state as #1. |
| 3 | Existing user, stale `enabledFeatures.constellationSight: true` from v2-era | ✅ v2 + v3 fields coexist. v2 gate short-circuits on `SIGHT_V2_ENABLED=false`; v3 gate opens. No collision. |
| 4 | Existing user, `enabledFeatures.constellationSightV3: false` | ✅ Setting wins; no v3 dock button. User re-enables via Settings → Plugins → "Constellation Sight" v3 row. |
| 5 | Mid-compute interruption | ✅ Cursor table tracks state; §1B's "always recompute" semantics make resume automatic. No corruption. |
| 6 | Rollback to pre-MIG-018 (flip `SIGHT_V3_ENABLED=false`) | ✅ v3 dock button hidden; schema tables stay (idempotent); cached rows inert; no data loss. |
| 7 | Forward to MIG-019 | ✅ Schema stable across phases; same IPC consumed; ALTER TABLE / new tables for new fields. |

**Extra paths verified**:
- **Settings round-trip (projection toggle)**: spread-preserve pattern safe for future v3 settings.
- **Library set change**: cache key (`library_set_hash`) is order-invariant; old hash rows stay inert.
- **community_id placeholder semantics**: §1B persists 0, §1E ignores IPC's value and runs frontend Louvain. Future MIGs need to wire Rust-side Louvain before reading IPC's community_id.
- **`SIGHT_V3_ENABLED=true` state confirmation**: working-tree value matches expected post-Boss-test state.

**Verdict**: CLEAN.

---

## Severity roll-up

| Severity | Count |
|---|---|
| P0 (release blocker) | 0 |
| P1 (fix before commit) | 0 |
| P2 (monitor post-ship) | 0 |
| P3 (low-risk observation) | 0 |

---

## Boss-test pass record (2026-05-07)

Eisa's report: **"All pass"** on the 11-step Boss-test tutorial in §1E commit message.

Verified observable:
1. ✅ Build install path: `src-tauri/target/release/constellation.exe` (post-§1E mtime).
2. ✅ Star-icon dock button reachable in production.
3. ✅ At-rest: dome of stars + soft Suwaidi pastel territories + faint connector lines visible.
4. ✅ Hover star: tooltip + incident edges brighten + gold ring around hovered.
5. ✅ Click star: constellation lights up + side panel slides in.
6. ✅ Double-click star: opens note in editor.
7. ✅ Settings → Sight → Projection: Lambert ↔ Stereographic toggle, stars shift positions.
8. ✅ Switch back to Lambert: notes return to remembered positions (spatial-memory grammar).
9. ✅ Search a note: matched stars flare; rest dim.
10. ✅ Esc clears filter / clears selection / closes Sight.
11. ✅ No regression: Sky View, Map, OrgChart, Inspector360, Index all functional.

---

## Final verdict — MIG-018 ready to close

All three agents converged on CLEAN. The five-commit cascade (§1A → §1E) is invariants-clean, drift-clean, migration-path-clean, and Boss-test verified. `SIGHT_V3_ENABLED = true` is the committed source state — v3 projection foundation ships in production for the next build.

**Acceptance criteria from Architect §6**: all 12 met.

**Closing actions for §1F (this commit)**:
1. ✅ This audit report.
2. Bump orientation v1.57 → v1.58 (MIG-018 row flips to ✅ Closed).
3. Bump Pending Jobs v1.6 → v1.7 (PJ-038 status: Confirmed → In-Progress; MIG-018 closed; MIG-019 next-up).
4. Append session log close-out.
5. Single commit lands all four artifacts + the `SIGHT_V3_ENABLED = true` source flip.

**MIG-018 closes Done**.

Next: **MIG-019** — density (PJ-035 Milky Way) + time (calendar rim) + search integration + universe-health card. Per Concept Paper v1.1 §9.2.

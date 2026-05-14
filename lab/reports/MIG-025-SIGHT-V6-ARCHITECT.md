# MIG-025 — Sight v6 Build — Architect

**Phase:** 1 of 4 (/migration discipline)
**Date:** 2026-05-13
**Status:** Awaiting Boss decisions on 2 flagged items before Plan opens.
**Reference contract:** `docs/Constellation-Sight-Concept-Paper-v4.0.md` (ratified 2026-05-13)
**Visual references:** `docs/sight-redesign-v0.3-full-layout.svg`, `docs/sight-redesign-v0.3-register-chip-detail.svg`
**Prior pattern:** `lab/reports/MIG-024-SIGHT-V5-LAYER-1-VISUAL-FOUNDATION-ARCHITECT.md`

> Concept Paper v4.0 §9.2 settles the four-phase architecture. This doc is NOT a re-architecture; it specifies HOW to execute the rollout safely.

---

## §1 — Territory map (the v5 surface that v6 replaces)

### §1.1 v5 files (frontend, `src/lib/sight/v5/`)

| File | LOC class | v6 disposition |
|---|---|---|
| `SightV5.svelte` (~780 lines) | Main component: 7-mode bar, scope bar, dome canvas, wedge filter chip, tooltip, side panel mount | **Delete in Phase 4**; replaced by `v6/SightV6.svelte` (anchor + facet sidebar + register chip + mini-domes per §9.1) |
| `render.ts` (~553 lines) | Canvas 2D `renderBaseLayer` + `renderFocusOverlay` + `computeStarPositions` + `wedgeKeyAtPoint` + hollow-cascade frame coloring | **Rebuild** as `v6/anchor.ts` — channel encoding changes (shape from library, opacity from confidence, pip from stage, size binary for acts, line color from typed-link kind). Hit-test logic reusable. |
| `modes.ts` (~306 lines) | `buildModeContext` for R/L/T/C/S/A/P; wedge bucket structure | **Delete** — v6 has no modes. The bucket pattern is conceptually replaced by facet sidebar categories + mini-dome channel isolation. |
| `dome.ts` (~157 lines) | `stratumBandBoundaries`, `radiusForStratum`, `calendarRimMonths`, `calendarRimSpokes`, `milkyWayEllipses`, `PALETTE` | **Reuse mostly intact**: strata-band math, calendar rim spokes, hash-based jitter carry forward; v5 has 8 strata bands — v6 spec is 5 (Foundation → Edge of Knowing), so `stratumBandBoundaries` needs to be reparameterized. Suwaidi `PALETTE` token names survive; some hex values change (v6 §3.4 specifies new stage + link palettes with CIE Delta-E ≥30 verification). |
| `types.ts` (~105 lines) | `SightV5Mode`, `SightV5Scope`, `LayoutCacheRow`, `LinkEdge`, `Star`, `Wedge`, `ModeContext` | **Rebuild** as `v6/types.ts` — mode/scope union types disappear; new contracts for facet, register, mini-dome channel, gesture event. `LayoutCacheRow`/`LinkEdge` shapes likely extend (new fields per channel orthogonality: stage hue, acts top-decile flag, provenance sector, library shape index). |
| `scope.ts` (~40 lines) | `filterNotesByScope` (universe / library / folder) | **Delete** — v6 uses facet sidebar's Folder + Library facets to do scope filtering; the universe stays canvas-implicit per Concept Paper §2.4 ("Universe is implicit"). |
| `SightV5SidePanel.svelte` | Per-note detail pane on star click | **Reuse partially** — selected-star detail pattern carries forward to v6's hover popover + click-to-open behavior per §5. Probably becomes `v6/StarPopover.svelte`. |

### §1.2 Rust files (`src-tauri/src/`)

| File | What | v6 disposition |
|---|---|---|
| `sight_v5.rs` (~875 lines) | `LayoutCacheRow` struct; `LinkEdge` struct; `ensure_sight_v5_layout_table`; `ensure_sight_v5_invalidation_trigger`; `compute_universe_snapshot_hash`; `backfill_sight_v5_layout` (sentinel v3 with auto-clear); 4 Tauri IPCs; ~20 unit tests | **Rebuild as `sight_v6.rs`**. Cache schema needs new columns: `link_in_count`, `link_out_count`, `frontmatter_key_count`, `body_chars` (currently noted in render.ts as "added to support fix-8 but not yet populated"), plus v6-specific fields (`top_decile_acts` flag, `provenance_sector_index`, `library_shape_index`). Sentinel chain restarts (`sight_v6_layout_backfill_v1`). Snapshot hash logic reusable. Trigger pattern reusable. |
| `lib.rs` (lines 352–355) | Registers the 4 IPCs in the Tauri builder | **Modify**: swap `sight_v5::*` for `sight_v6::*`; keep both during Phase 1–3 if running v5/v6 side-by-side (see §3 option B). |
| `search.rs` (lines 1537–1545) | `init_db` calls `ensure_sight_v5_layout_table` + `ensure_sight_v5_invalidation_trigger`. Lazy-backfill comment. | **Modify**: add v6 table+trigger creation; keep v5 calls if dual-mounted during transition. |

### §1.3 SQLite surface

- `sight_v5_layout` table — 13 columns (note_path PK + 12 derived) + 2 covering indexes (library, folder).
- Two invalidation triggers: `sight_v5_layout_invalidate_au` (AFTER UPDATE on note_meta) + `sight_v5_layout_invalidate_ad` (AFTER DELETE on note_meta).
- Sentinel in `schema_versions` table: `'mig024_sight_v5_layout_backfill_v3'` (with auto-clear of v1/v2). Pattern is the reference for v6.
- **v6 needs**: `sight_v6_layout` table with ≥4 new columns; same trigger family; new sentinel chain.

### §1.4 Tauri IPCs (4 surfaces)

All exposed via `lib.rs` `tauri::generate_handler!`; all called from `SightV5.svelte::loadLayout` except `sight_v5_get_universe_snapshot_hash` which is **registered but currently unused** (built for future cache-invalidation polling that never landed).

| IPC | Frontend caller(s) | v6 successor |
|---|---|---|
| `sight_v5_warm_cache` | `SightV5.svelte:263` | `sight_v6_warm_cache` |
| `sight_v5_get_layout(scope_kind, scope_id)` | `SightV5.svelte:265` | `sight_v6_get_layout` (scope_kind parameter likely drops; facet filtering is frontend) |
| `sight_v5_get_link_set_for_notes(paths)` | `SightV5.svelte:275` | `sight_v6_get_link_set_for_notes` (cap at 800 lines per §2.2 fade rule, not 2000) |
| `sight_v5_get_universe_snapshot_hash` | **unused** | Drop or reuse for live-update polling |

### §1.5 Feature flag (`src/lib/sight/engine.ts`)

Currently 4 flags, all but V5 false:
- `SIGHT_V2_ENABLED = false`
- `SIGHT_V3_ENABLED = false`
- `SIGHT_V4_ENABLED = false`
- `SIGHT_V5_ENABLED = true`

v6 adds `SIGHT_V6_ENABLED`. Toggle strategy depends on §3 Option B choice below.

### §1.6 Consumers of v5 to verify (Plan must enumerate)

These are the exact places Phase 2 Plan must touch:

| Location | What it does |
|---|---|
| `src/routes/+layout.svelte:67` | `import SightV5 from '$lib/sight/v5/SightV5.svelte';` |
| `src/routes/+layout.svelte:68` | `import { SIGHT_V2_ENABLED, SIGHT_V3_ENABLED, SIGHT_V4_ENABLED, SIGHT_V5_ENABLED } from '$lib/sight/engine';` |
| `src/routes/+layout.svelte:752` | `let sightV5Active = $state(false);` — the mount-state local |
| `src/routes/+layout.svelte:1047` | `sightV5Active` participates in `fullPageActive` derivation |
| `src/routes/+layout.svelte:1706, 2940, 3619, 4358, 4454, 4852` | 6 other call sites that set `sightV5Active = false` as part of view-switching mutual exclusivity |
| `src/routes/+layout.svelte:4465–4475` | Dock button: `{#if SIGHT_V5_ENABLED && $appSettings.enabledFeatures?.constellationSightV3 !== false}` (note: still gated on the *v3* feature flag in user settings — a quirk; v6 should either keep this or introduce `constellationSightV6`) |
| `src/routes/+layout.svelte:5343–5362` | The mount block: `{:else if sightV5Active && SIGHT_V5_ENABLED}` … `<SightV5 onOpenNote={…}>` |
| `src-tauri/src/lib.rs:352–355` | IPC registrations |
| `src-tauri/src/search.rs:1537–1545` | `init_db` schema setup |
| `src/lib/libraries/store.ts:3416–3446, 3596–3605, 3712–3715` | `appSettings.sight` namespace declaration + DEFAULT_SETTINGS + parse merge |
| `src/lib/components/SettingsModal.svelte:1626–1702` | Sight settings UI (projection, milky-way, calendar systems, alwaysOnLabels) — note: these are v3-era keys still used by SightV3/V4 but **not consumed by v5**. v6 may want a fresh Settings panel. |

### §1.7 Settings keys (`appSettings.sight.*`)

Verified by reading `store.ts`. Actual keys:

- `projection` — `'lambert' | 'stereographic'` (v3/v4 only; v5/v6 ignore)
- `showMilkyWay` — `boolean` (v3/v4 only; v5/v6 ignore)
- `calendarSystems` — `Array<'gregorian' | 'hijri' | 'solar-hijri' | 'hebrew'>` (v3/v4 only; v6 spec at §2.2 specifies Gregorian rim only by default)
- `alwaysOnLabels` — `boolean` (v3/v4 only)
- `lastMode` — `'R' | 'L' | 'T' | 'C' | 'S' | 'A' | 'P'` (v5 only; **dropped in v6** per §9.3)
- `lastScope` — `'universe' | 'library' | 'folder'` (v5 only; **dropped in v6** — facet sidebar handles scope)

v6 will likely add: `proMode` (boolean, §6.4), `tourSeen` (boolean, §5 first-boot tour), `activeRegister` (one of 7 register IDs, §4), `hexBinThreshold` (number, default 5000, §2.3), and `linkFadeThreshold` (number, default 800, §2.2).

### §1.8 Mount surface

Sight is mounted **only** in `src/routes/+layout.svelte` via the dock button. No router hash, no deep-link, no embedded inline use. The full-page mount pattern is well-trodden (Sight v4/v5 used it; SkyView is the originator).

---

## §2 — The 10 invariants from v4.0 §11 (verbatim, must not break)

> *Quoted verbatim from `docs/Constellation-Sight-Concept-Paper-v4.0.md` §11:*

1. **Channel orthogonality**: no two channels share a Bertin variable.
2. **Default Suwaidi-fidelity**: anchor dome ≥80% of visible canvas in default state.
3. **Cross-filter performance**: ≤16 ms on 7,636 notes × 5 views.
4. **CIE Delta-E ≥30**: between any two co-rendered hues at build time.
5. **Pip foveation threshold**: anchor pip ≥1.8 px at default zoom, suppressed below 1.5 px.
6. **Register isolation**: register chip remaps anchor dome only; mini-domes stay culturally neutral.
7. **Register manifest**: each register's geometry is documented + citation-tracked in version control.
8. **Folder visibility**: Folder is a first-class facet in the sidebar.
9. **Gesture chrome**: no persistent toggle bars. All interaction via gestures + sidebar/chip/mini-dome clicks.
10. **First-boot tour**: 4 steps, skippable, always re-available in Help.

Plus the foundational v5-era invariants that survive: I-1 file-over-app, I-2 zero keystroke lag, I-3 write-time derivation, I-7 i18n parity, I-8 ≤6 s boot, I-13 Sight = whole universe, 360.3D = single note.

---

## §3 — Build-strategy options (the genuine forks)

For each: **speed** = time-to-Phase-1-ship, **effort** = total eng-weeks across all 4 phases, **risk** = low / medium / high.

### Option A — MIG structure: single MIG-025 with sub-phases vs. four separate MIGs

| Choice | Speed | Effort | Risk | Trade-off |
|---|---|---|---|---|
| **A1** Single MIG-025 with §A/§B/§C/§D phases | Phase 1 in ~6 wk | ~18 wk total | Medium | One Architect+Plan, one Audit at end. Cleaner rollback (one revert), one risk register. But Audit phase fires only after all 4 sub-phases — a regression in Phase 2 can hide until Phase 4 audit. |
| **A2** Four separate MIGs (one per phase) | Phase 1 in ~6 wk | ~19 wk (overhead) | Low | Each phase gets its own /migration four-phase pass including its own Audit. Tighter quality gate per phase. More ceremony; each MIG re-explores territory. v4.0 §9.2 expects ONE migration. |

**Recommendation: A1** — Concept Paper §9.2 is explicit about phased shipping within one architectural contract; treat the 4 phases as sub-MIGs internally but bundle them under MIG-025 for traceability. Each phase still gets its own verification clauses per §13 of the Concept Paper. Defensible alternative: A2 if Eisa wants the post-phase Audit to fire continuously. **Equally defensible.**

### Option B — v5 deletion timing + dual-mount strategy

| Choice | Speed | Effort | Risk | Trade-off |
|---|---|---|---|---|
| **B1** Phase 4 deletes v5 (per v4.0 §9.3) | Fastest to clean tree | Baseline | Medium | The 12 weeks between Phase 1 ship and Phase 4 ship, v5 sits as dead code. If v6.0 has a regression, no fallback inside the same build. Requires git revert to get back to v5. |
| **B2** Add `SIGHT_V6_ENABLED` flag; v5 stays reachable Phase 1–3, deleted Phase 4 | +0.5 wk Phase 1 | +0.5 wk overhead | **Low** | Both engines mount; dock button switches. Mutual exclusivity follows v4→v5 ship pattern (engine.ts flags). User can toggle in Settings if v6 misbehaves. Phase 4 cleans up. |
| **B3** Keep both side-by-side via Settings toggle for two release cycles | +1 wk overall | +1.5 wk | Medium | Settings exposes "Use Sight v6 (Beta)" toggle. v5 stays in production until user explicitly switches. Tour fires only on v6 toggle. Risk: forking maintenance — two SQLite caches, two trigger chains, drift. |

**Recommendation: B2** — Mirrors the proven v4→v5 cutover pattern. Both engines coexist for ~12 wk; Phase 4 deletes v5 (delete the v5/ directory + sight_v5.rs + drop sight_v5_layout table). Defensible alternative: B3 if Eisa wants user-facing beta gating. **Eisa-decision required**: how exposed should the toggle be? B2 is dev-flag-only (no user UI); B3 is user-visible setting.

### Option C — SQLite cache migration

| Choice | Speed | Effort | Risk | Trade-off |
|---|---|---|---|---|
| **C1** Eager: drop `sight_v5_layout`, build `sight_v6_layout` synchronously at v6.0 first boot | Fast warm reads after migration | Baseline | Medium | One-shot transactional rebuild. Blocks first Sight v6 open on 7,636-note universe for ~3–10 s. Resumable if interrupted (sentinel pattern). Per v4.0 §9.3 this is the prescribed approach. |
| **C2** Lazy: keep v5 cache; rebuild v6 cache on first Sight open with loading state | Slowest first open | Low | Low | Familiar pattern (v5 already does lazy backfill via `warm_cache`). No double-cache cost. But the first open IS slow regardless; user perceives "v6 is slower" until cache warmed. |
| **C3** Progressive backfill: Tauri-event–driven incremental fill with status-bar progress per Standing Order resumability | Best UX (works while user uses app) | +1 wk | Medium | Per Standing Order (CLAUDE.md resumability rule). Sight v6 renders progressively as cache fills. Complex: requires partial-cache rendering, event-throttling, status-bar widget. Best fit for the Concept Paper's "no mid-edit interruption" principle (§9.3). |

**Recommendation: C3** — Concept Paper §9.3 explicitly says "Backfill runs on first boot in background with progress in status bar (resumable per Standing Order)." C3 is the Concept-Paper-aligned default. Defensible alternative: C1 for simplicity if Eisa accepts the 3–10 s first-open block. **Risk note**: C3's progressive rendering means visible stars change as cache fills, which can be disorienting; mitigate by gating Sight v6 render-readiness on "first stratum tier complete" not "all rows complete."

### Option D — Worktree strategy

Per Working Agreement #2 (single primary repo, worktrees for parallel work):

| Choice | Speed | Effort | Risk | Trade-off |
|---|---|---|---|---|
| **D1** Single worktree for all 4 phases | Baseline | Baseline | Low | One branch, sequential commits. Aligns with Working Agreement #2 default. Each phase merges to main on Phase ship gate; next phase opens against fresh main. |
| **D2** Per-phase worktrees that merge sequentially | +0.5 wk overhead | +0.5 wk | Medium | Each phase in its own worktree (allows /migration Audits to fire in parallel with next phase's Architect). More git overhead; merge-conflict risk between phases that touch the same files (`+layout.svelte` likely). |

**Recommendation: D1** — The /migration workflow expects sequential phase progression. D2's parallelism doesn't actually buy speed because Phase N+1 depends on Phase N's user-visible behavior being known-good. **Equally defensible** if Eisa wants to overlap audit-of-N with architect-of-(N+1).

### Option E — First-boot tour persistence

| Choice | Trade-off |
|---|---|
| **E1** Show on first-ever Sight v6 open only | Per v4.0 §5; users see it once, dismiss via `tourSeen` flag. Returns via Help → "Sight tour." |
| **E2** Show on every Sight version bump (v6.0 → v6.1 → v6.2 → v6.3) | Re-introduces user to each new phase's surface. Risk: 4 tours in 18 weeks feels nagging. |
| **E3** Opt-in via Help menu only | Skipped on first launch entirely. Conflicts with Concept Paper §5 invariant 10. |

**Recommendation: E1** — Direct alignment with invariant 10. Defensible alternative: E2 with the tour content updated to "what's new in v6.1" — but that crosses into release-notes territory; better handled by a release-notes affordance than the tour. **E1 is the contract.**

### Option F — Performance gate strategy

| Choice | Speed | Effort | Risk | Trade-off |
|---|---|---|---|---|
| **F1** Verify §8.3 budgets at end of each phase | Baseline | Low | Low | Catches regressions early; aligns with each phase's verification clauses (§13). |
| **F2** Only at v6.0 final ship (Phase 4) | Fastest dev | Low | **High** | A Phase-2 perf regression hides until Phase 4 audit. Catastrophic if final perf fails and root cause is upstream. |
| **F3** Continuously via CI on every build | Highest confidence | +1 wk Phase 1 (CI plumbing) | Low | Phase 1 adds vitest+playwright perf harness; every PR validates. Concept Paper §8.3 implies this ("Verified via render-budget test in CI"). |

**Recommendation: F3** — Concept Paper §8.3 is explicit: render-budget test in CI. Add the harness in Phase 1 so phases 2–4 inherit it. Phase 4 adds the channel-orthogonality + CIE Delta-E tests per §13.4.

### Option G — Settings migration

| Choice | Trade-off |
|---|---|
| **G1** Read `sight_v5_*` once and map forward (per v4.0 §9.3) | `lastMode` dropped; `lastScope` carries forward. Quiet automatic upgrade. Concept Paper default. |
| **G2** Drop and start fresh | Cleanest schema. Users with custom scope settings lose them. Surprising. |
| **G3** Offer the user a one-time choice during the upgrade dialog | Most respectful. Adds friction; users may not understand the question. |

**Recommendation: G1** — Concept Paper §9.3 is explicit. `lastScope` → carry forward; `lastMode` → drop silently. **No Eisa-decision required.**

### Option H — Register manifest creation timing

| Choice | Trade-off |
|---|---|
| **H1** Create all 7 `docs/registers/*.md` files in Phase 3 alongside the registers | Per v4.0 §9.2 — Phase 3 is when registers land. Manifests written when the geometry is implemented. |
| **H2** Pre-create stubs in Phase 1 as part of foundation work | Architect+Plan can reference the manifests during phases 1–2. Risk: stubs drift from final implementation. |

**Recommendation: H1** — Phase 3 is when the geometry exists. **Equally defensible** if Eisa wants the Cross-Civ SMEs to pre-review citation drafts.

### Option I — Mock-to-implementation reference

| Choice | Trade-off |
|---|---|
| **I1** Use the v0.3 mock SVGs as the visual contract | Lower upfront cost; engineers reference the SVGs directly. Concept Paper §12 preserves them as "visual reference for Sight v6 implementation." |
| **I2** Convert them to a polished design-system spec in Phase 1 | Adds ~1 wk Phase 1; produces a Storybook-style spec with all states. Best for long-term maintenance. |

**Recommendation: I1** — Treat the SVGs as the contract; v6 ships pixel-aligned within Suwaidi palette tolerance. **Equally defensible.**

---

## §4 — Migration / back-fill / rollback concerns

### §4.1 First-boot-after-upgrade

User launches Constellation post-MIG-025 ship (Phase 1 = v6.0):

1. `init_db` runs `ensure_sight_v6_layout_table` + `ensure_sight_v6_invalidation_trigger`. Idempotent.
2. `sight_v5_layout` table stays untouched (Option B2 strategy). Triggers stay live.
3. User clicks Sight in dock → mounts `SightV6.svelte`.
4. `SightV6.svelte` calls `sight_v6_warm_cache` → backfill runs (Option C3 progressive). Stars appear progressively over ~3–10 s on 7,636-note universe.
5. First-boot tour overlay fires (`tourSeen` not yet set). 4 skippable steps per §5.
6. Settings migration: `appSettings.sight.lastMode` dropped silently; `appSettings.sight.lastScope` mapped forward.

### §4.2 Mid-Phase-2-shipped-mid-Phase-3-still-in-flight

User is on Sight v6.1 (Phase 2 shipped; mini-domes work; Phase 3 register chip not yet shipped):

- Anchor dome works.
- "Show diagnostics" gesture reveals mini-domes per §2.3.
- Register chip behavior: **Eisa-decision required** — hidden entirely until Phase 3, OR shown as non-interactive placeholder?
- Settings menu does NOT show register switch.

### §4.3 Rollback v6.x → v5 (regression scenario)

If Phase 1 ships v6.0 and a regression surfaces requiring rollback:

**Option B2 path (recommended):**
1. Flip `SIGHT_V6_ENABLED = false` and `SIGHT_V5_ENABLED = true` in `engine.ts`. Single-line edit. Ship hotfix.
2. v5 dock button reappears. v5 component mounts. v5's `sight_v5_warm_cache` still works.
3. v6 `sight_v6_layout` rows stay in DB — they don't conflict.
4. Tour state stays set; if v6 re-enables, no second tour.
5. Settings: `lastScope` was mapped forward but v5 still reads it correctly; no data loss.

**Highest-risk migration item**: dual-trigger window during Phases 1–3 (both `sight_v5_layout_invalidate_*` and `sight_v6_layout_invalidate_*` fire on every `note_meta` write). No corruption risk, but a write-throughput regression under heavy editing is plausible. **Mitigation**: Phase 4 explicitly drops v5 triggers in the same migration step that drops the v5 cache table. Phases 1–3 accept the dual-trigger cost (cheap; just two DELETEs on indexed columns).

### §4.4 Schema-version sentinel race

Both v5 and v6 sentinels live in the same `schema_versions` table. v5's chain (`mig024_sight_v5_layout_backfill_v3`) and v6's chain (`mig025_sight_v6_layout_backfill_v1`) coexist without conflict. Low risk.

---

## §5 — Defensible defaults (recommended option pack)

| Choice | Default | Eisa-decision required? |
|---|---|---|
| A — MIG structure | **A1** Single MIG-025 with sub-phases | No (defensible) |
| B — v5 deletion timing | **B2** `SIGHT_V6_ENABLED` flag, v5 reachable Phase 1–3, deleted Phase 4 | **Yes** — confirm dev-flag-only vs. user-visible toggle |
| C — SQLite cache migration | **C3** Progressive backfill with status-bar progress | No (Concept Paper-aligned) |
| D — Worktree strategy | **D1** Single worktree all 4 phases | No (defensible) |
| E — Tour persistence | **E1** First-ever Sight v6 open only | No (invariant 10) |
| F — Performance gate | **F3** Continuous via CI from Phase 1 | No (Concept Paper §8.3) |
| G — Settings migration | **G1** Read `sight_v5_*` once, map forward | No (Concept Paper §9.3) |
| H — Register manifests | **H1** Create in Phase 3 with the geometry | No (defensible) |
| I — Visual reference | **I1** v0.3 SVGs as the contract | No (Concept Paper §12) |
| Plus: Phase-2 register chip presence | **Hidden until Phase 3** (cleaner) | **Yes** — Eisa picks |

**Total recommended pack effort**: ~18 wk (Concept Paper baseline) + ~1 wk B2 overhead + ~1 wk C3 overhead + ~1 wk F3 CI plumbing = **~21 wk**.

---

## §6 — What's next

After Eisa locks B and the Phase-2-chip question: **Phase 2 Plan opens** as `lab/reports/MIG-025-SIGHT-V6-PLAN.md`. The Plan ordered-step-lists each phase against the file map in §1 + the invariant list in §2 + the test surface implied by F3.

After the Plan: **Phase 3 Build** cascades through phases 1→4 sub-phases, each gated by its own §13 verification clauses.

After Phase 4 Build: **Phase 4 Audit** — three agents in parallel per /migration. Agent 4A walks the 10 invariants. Agent 4B walks the trigger/IPC drift. Agent 4C walks the first-boot / mid-phase / rollback sequences from §4.

---

**Critical files for Phase 2 Plan:**
- `E:\مشاريع كلاود\Constellation\docs\Constellation-Sight-Concept-Paper-v4.0.md`
- `E:\مشاريع كلاود\Constellation\src\routes\+layout.svelte`
- `E:\مشاريع كلاود\Constellation\src-tauri\src\sight_v5.rs`
- `E:\مشاريع كلاود\Constellation\src\lib\sight\engine.ts`
- `E:\مشاريع كلاود\Constellation\src\lib\libraries\store.ts`

**End of MIG-025 Architect.** Awaiting Eisa's lock on Option B (dev-flag vs. user-toggle) and the Phase-2 register chip question.

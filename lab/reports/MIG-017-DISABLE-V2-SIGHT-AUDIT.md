# MIG-017 §1.10 — Three-agent audit

**Migration**: MIG-017 (PJ-039) — Disable v2 Sight
**Date**: 2026-05-07
**Mode**: parallel three-agent (invariants / drift / migration-path).
**Verdict**: **CLEAN.** 0 P0, 0 P1, 0 P2, 0 P3.

---

## What got audited

Single uncommitted change set:

- **New file**: `src/lib/sight/engine.ts` — exports `SIGHT_V2_ENABLED = false` (15 lines incl. doc comment).
- **Edits**:
  - `src/routes/+layout.svelte` — added `import { SIGHT_V2_ENABLED }` at line 65; gated dock button at line 4361 (`SIGHT_V2_ENABLED && ...`); gated overlay+modal mount at lines 4993–4994 (both `class:` binding and `{#if}`); gated "Return to Lens" button at line 4741.
  - `src/lib/components/SettingsModal.svelte` — added `import { SIGHT_V2_ENABLED }` at line 20; converted plugin entry at line 270 from unconditional object to conditional spread `...(SIGHT_V2_ENABLED ? [{...}] : [])`.
  - `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` — banner block prepended after frontmatter; original v2 documentation untouched beneath.

Pre-commit verification: `npm run check` ran with **1 pre-existing PJ-012 error** (`store.ts:2324 LinkLifecycle.fresh`) and **0 new errors**. 299 unused-CSS-selector warnings, all unrelated to MIG-017.

---

## Agent #1 — Invariants

**Coverage**: all 12 invariants from Architect §6.

| # | Invariant | Verdict |
|---|---|---|
| 1 | v2 component compiles | ✅ `ConstellationSight2.svelte` intact; import at `+layout.svelte:64` untouched. |
| 2 | All `lensActive = false` reset paths still fire | ✅ 10 occurrences verified across `+layout.svelte`. |
| 3 | No regression to other surfaces | ✅ Sky View, OrgChart, Map, Index, SearchHub, Inspector360 each retain own `enabledFeatures` gate. |
| 4 | No new IPC | ✅ Zero `invoke(` patterns in diff. |
| 5 | No `$effect` loop introduced | ✅ Const read inside `{#if}` only. |
| 6 | No boot-perf regression | ✅ One-line const export; tree-shaking removes unreachable branch. |
| 7 | i18n integrity | ✅ All 15 locales preserve `lens.*` and `settings.plugins.constellationSight*` keys. |
| 8 | Settings round-trip | ✅ `DEFAULT_SETTINGS.constellationSight: true` stays vestigial; load/save preserve user value; gate now requires `SIGHT_V2_ENABLED`. |
| 9 | `ConstellationMap` filename collision | ✅ Map's `lens.fitToScreen` i18n untouched. |
| 10 | CE Phase 9 multi-lens isolation | ✅ `availableLenses`, `activeLensId`, `lensGroups`, `lensEntries`, `create-lens` palette entry, `deleteLens` all untouched. |
| 11 | Help-doc banner non-destructive | ✅ Banner prepended; original v2 paragraphs intact. (Only English; Arabic help dir has no Sight file — banner prep is moot there until structure expands.) |
| 12 | M11 zero-diff | ✅ `git diff src-tauri/src/lexicon/` empty. |

**Extra entry-point scan**: agent grepped `lensActive\s*=\s*true`, `toggleLens\s*\(`, `ConstellationSight2`, `enabledFeatures.constellationSight`. All v2 entry points are gated — dock button, modal mount, "Return to Lens" button, Settings plugin entry. `toggleLens()` invocation is inside the gated dock button. `lensActive = true` assignments are inside the (untouched) `toggleLens()` body or the now-gated "Return to Lens" handler. **No unprotected v2 entry points found.**

**Verdict**: CLEAN.

---

## Agent #2 — Drift

**Coverage**: implicit consumers, CSS rule collapse, cross-surface touchpoints, naming forward-compat.

- **Implicit consumers**: zero. Three explicit consumers (4 sites) accounted for; agent grepped for any other reference to `lensActive`, `lensReturnPending`, `enabledFeatures.constellationSight`, `toggleLens` and confirmed none are unprotected.
- **CSS overlay collapse**: `.lens-overlay { display: none; }` base; `.lens-overlay.lens-visible { display: flex; }` active. With `class:lens-visible={lensActive && SIGHT_V2_ENABLED}` evaluating `false`, overlay div takes zero space. No z-index conflict, no visual debris.
- **Cross-surface touchpoints**: Sky View / Map / OrgChart / Inspector360 each have independent gates — unaffected by `SIGHT_V2_ENABLED`. Second screen (`src/lib/secondScreen.ts`) mirrors `editor | skyview | browser` ContextMode; no Sight propagation needed. i18n untouched in all 15 locales. Help-doc cross-reference to `Constellation-Sight-Concept-Paper-v1.1.md` valid.
- **Naming forward-compat**: `SIGHT_V2_ENABLED` explicitly names the version. The doc comment in `engine.ts` already announces v3 will add `SIGHT_V3_ENABLED` as a coexisting flag. No risk of misinterpretation.

**Verdict**: CLEAN.

---

## Agent #3 — Migration-path

**Coverage**: five scenarios from Architect §8 + two extra paths.

| # | Scenario | Predicted | Verified |
|---|---|---|---|
| 1 | Fresh install (no settings.json) | hidden | ✅ DEFAULT's `constellationSight: true` is vestigial; `SIGHT_V2_ENABLED && true` → `false` → no button. |
| 2 | Existing user, `constellationSight: true` saved | hidden | ✅ Merge preserves `true`; gate short-circuits same as #1. |
| 3 | Existing user, `constellationSight: false` saved | hidden | ✅ Right-side gate fails first; short-circuit. |
| 4 | Mid-state — `lensActive = true` via DevTools | overlay collapsed | ✅ Both `class:lens-visible` and `{#if}` short-circuit; no mount, no null-pointer surface. |
| 5 | Rollback to v2 (`SIGHT_V2_ENABLED = true`) | identical to pre-MIG | ✅ Each gate reduces to its pre-MIG-017 form. Dock button reappears, modal opens, `lensReturnPending` gate becomes active again. |

**Extra path 1** (`lensReturnPending` entry points): grep finds exactly one setter at `+layout.svelte:5018` inside the v2 modal's `onNoteClick` handler. Unreachable when `SIGHT_V2_ENABLED = false` because the component never mounts. Defensive gate at line 4741 is robust.

**Extra path 2** (v3 coexistence): `engine.ts` is one line of code + extensible. No `enabledFeatures` field collision (v3 will register under a new key). `SettingsModal` plugin list uses spread operator so v3 entry can be added on the next line. Mutual exclusion enforced by conjunctive gates, not by code structure — both engines can coexist in the diff with only one rendering at a time.

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

## Final verdict — ready to commit

All three agents converged on **CLEAN**. The four UI gates (`SIGHT_V2_ENABLED && ...`) successfully isolate v2 Sight from the user surface in default config. v2 component code, `lens*` `$state` fields, `toggleLens()` function, Rust analytics modules, and `constellation_sight_*` IPCs are all preserved on disk. Re-enable is one edit (`SIGHT_V2_ENABLED = true` + rebuild).

**Acceptance criteria from Architect §11**: all 9 met or scheduled (Boss-test items 1–6 will be confirmed when the Boss runs the next install; criteria 7 [audit clean] is this report; criteria 8–9 [orientation v1.56 + Pending Jobs v1.5] land in the same commit).

**Commit can proceed.**

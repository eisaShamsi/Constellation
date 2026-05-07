# MIG-017 — Disable v2 Sight (Architect)

**Migration**: MIG-017 (PJ-039)
**Boss directive**: 2026-05-07. *"Secure what's achieved, never muddle. Disable v2 Sight as a known-good fallback while v3 is built fresh under PJ-038."*
**Effort class**: Mini-MIG (single session, single commit if Plan stays clean).
**Precondition for**: PJ-038 — Sight v3 build with own dedicated Concept Paper.

---

## 1 · Goal

Hide v2 Sight from the running app's user-visible surface — dock button, modal mount, Settings plugin entry, "Return to Lens" button — while preserving the v2 component, the `lens*` `$state` fields, the `toggleLens()` function, the Rust analytics modules (`lens.rs` / `lenses.rs`), and the `constellation_sight_*` IPCs **on disk** as a known-good fallback.

End-state semantics: a fresh v0.3.5 (or successor) install presents zero v2 Sight surfaces to the user. A developer can revive v2 by flipping a single code constant and rebuilding — no settings migration, no schema change, no IPC re-registration.

This is **not** a delete. v2 stays in the repo as the proven baseline. If v3 fails to ship, v2 is the rollback target.

---

## 2 · Surfaces v2 is reachable through (today)

Mapped from grep over `src/`:

| # | Surface | File:line | Current gate | Reachability |
|---|---|---|---|---|
| 1 | **Dock button** | `+layout.svelte:4358-4368` | `{#if $appSettings.enabledFeatures?.constellationSight !== false}` | Default `true` in `DEFAULT_SETTINGS` (`store.ts:3362`) → button visible for fresh installs *and* existing users with saved settings. |
| 2 | **Modal mount** | `+layout.svelte:4988-5018` (`<div class="lens-overlay">` + `{#if lensActive}`) | `lensActive` boolean only | Reachable iff `lensActive = true` at any time. Set by surface 1, surface 4, or `toggleLens()` direct call. |
| 3 | **Settings → Plugins entry** | `SettingsModal.svelte:267` | None — listed unconditionally | Renders the toggle row in Settings → Plugins → Visualization. |
| 4 | **"Return to Lens" button** | `+layout.svelte:4738-4742` | `{#if lensReturnPending}` | Belt-and-suspenders. Set when user navigates Index/SearchHub/SkyView from inside Sight. Unreachable cold (v2 must have been opened first). |
| 5 | **Help doc** | `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` | None | Static markdown page documenting v2's gravity-well visualization. |

**Surfaces NOT reachable**:
- No keyboard shortcut binds `toggleLens()`. Verified via grep over `src/`.
- No Command Palette entry opens Sight directly. (`create-lens` at `+layout.svelte:1730` opens Settings, not Sight — that's CE Phase 9 multi-lens machinery, separate from v2 Sight.)
- No deep-link / URL fragment opens Sight (no router-based entry).

This means surfaces 1–4 are the complete UI gate set; close those four and v2 is unreachable.

---

## 3 · v2 Sight code that MUST stay on disk (known-good fallback)

Inventory of what `MIG-017` does not touch:

### 3.1 Frontend (Svelte)
- `src/lib/components/ConstellationSight2.svelte` — full v2 component. Untouched.
- `+layout.svelte` `lens*` `$state` fields (`lensActive`, `lensLoading`, `lensCentrality`, `lensCommunities`, `lensCommunityAssignments`, `lensGaps`, `lensHealth`, `lensBridges`, `lensShowTagEdges`, `lensPeelCount`, `lensTagEdges`, `lensCommunityProfiles`, `lensContradictions`, `lensDataStale`, `lensReturnPending`). Untouched.
- `+layout.svelte:3332-3471` `toggleLens()` async function + the `MIG-016 §1A` performance.mark instrumentation block. Untouched.
- `+layout.svelte:1255-1265` WTD `$effect` invalidating `lensDataStale` on graph version bumps. Untouched.
- All `lensActive = false` reset lines (Escape key, search-hub return, sky-view return, index entry, etc.) — untouched. They're harmless if `lensActive` can never become `true`.

### 3.2 Backend (Rust)
- `src-tauri/src/lens.rs` (centrality + community detection + structural gaps + universe health) — untouched.
- `src-tauri/src/lenses.rs` (CE Phase 9 multi-lens machinery; orthogonal to v2 Sight; tracked separately under PJ-013 dead-code decision) — untouched.
- All `constellation_sight_*` IPCs — registered, callable, untouched.

### 3.3 i18n
- 15 locale `settings.plugins.constellationSight` + `constellationSightDesc` strings — untouched.
- 15 locale `lens.title`, `lens.returnToLens`, `commands.createLens` strings — untouched.

### 3.4 What this means for re-enable
A developer flips `SIGHT_V2_ENABLED` from `false` to `true` in one file, rebuilds, ships. v2 surfaces all reappear in their existing form. Zero data migration, zero schema work, zero IPC re-registration. The fallback is genuinely one-edit.

---

## 4 · Mechanism — single code constant

### 4.1 Why a code constant, not a Settings flag

The v1.1 Concept Paper §14 sketched `sight.engine: 'v2-disabled' | 'v2' | 'v3'` as a Settings field. After investigation of the actual surfaces (§2), a **code constant** is cleaner because:

1. **The fallback is a *codebase* fallback, not a user toggle.** If v3 fails, we flip the constant and rebuild — that's a developer action, not a user-facing setting. Exposing the flag in Settings would create a confusing "v2 (disabled) / v2 (enabled) / v3" tri-state for end users with no useful semantics.
2. **No settings migration**. A Settings flag would need a migration to flip `constellationSight: true` → `false` for existing users with saved settings. A code constant just wins, regardless of saved state. Zero churn.
3. **Single source of truth**. One file, one line, three consumers (dock button, modal mount, Settings entry). Findable by grep, editable by one keystroke.
4. **Forward-compat with v3**. When v3 ships, the same file extends to:
   ```ts
   export const SIGHT_V2_ENABLED = false;
   export const SIGHT_V3_ENABLED = true;
   ```
   Or a single `SIGHT_ENGINE = 'v3'` literal type. The string-union form from the Concept Paper §14 is preserved as a future-shape; we land it as two booleans now because that's all we need today.

### 4.2 Where the constant lives

New file: `src/lib/sight/engine.ts`

```ts
/**
 * Sight engine flags — MIG-017 / PJ-039.
 *
 * v2 Sight (`ConstellationSight2.svelte` + `lens.rs` + `constellation_sight_*` IPCs)
 * is preserved on disk as a known-good fallback. To revive v2 for diagnostics or
 * because v3 failed to ship, flip `SIGHT_V2_ENABLED` to `true`, rebuild, ship.
 *
 * Future v3 (PJ-038) will add `SIGHT_V3_ENABLED` here. The two are mutually
 * exclusive in production (only one engine renders at a time); the dual flags
 * exist so a developer can A/B them in a custom build.
 */
export const SIGHT_V2_ENABLED = false;
```

This is the entire content of the new file. Eight statements of comment + one line of code.

### 4.3 Why not co-locate with `+layout.svelte`?

`+layout.svelte` already imports from `src/lib/...`; adding one more import keeps the gating uniform with how every other feature flag is wired (e.g., `import { appSettings } from '$lib/libraries/store'`). And v3 will populate `src/lib/sight/` further (`projection.ts`, `constellation-territories.ts`, `time-rim.ts`, etc.) — bootstrapping the directory now is cheap. SettingsModal.svelte and any future consumer import from the same place.

### 4.4 Why not use the existing `enabledFeatures.constellationSight` field?

It's a per-user setting, layered on top of `DEFAULT_SETTINGS`. To hide v2 from existing users (who have `constellationSight: true` saved), we'd need a one-time migration on settings load. That's three steps:
- Bump `appSettings` schema version.
- Add migration: "if loaded settings has `constellationSight === true` and current version is < N, flip to `false`."
- Add v3-aware logic later: when v3 ships, the same field needs to mean something different.

A code constant skips all three. The existing `enabledFeatures.constellationSight` field stays on disk in user settings — vestigial, harmless. When v3 ships under PJ-038, we revisit whether to repurpose it or migrate to a new `enabledFeatures.constellationSightV3` key.

---

## 5 · Help-doc treatment

`docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` is what users see when they click the help icon (or browse the help library) for Sight.

**Treatment**: prepend a banner at the top of the file:

```markdown
> **🚧 Constellation Sight is being rebuilt.**
>
> The current "gravity-well" visualization (v2) has been disabled. A new Sight (v3)
> is in design — based on a star-chart aesthetic that lets you see your entire
> knowledge universe at a glance. The reference page below describes what v2 *did*
> and is preserved while v3 ships.
>
> Read [`Constellation-Sight-Concept-Paper-v1.1.md`](../../Constellation-Sight-Concept-Paper-v1.1.md)
> for what Sight is for and §13–§14 for the v3 vision.
```

The original v2 documentation stays beneath, untouched. When v3 lands, this file gets a full rewrite under PJ-038's own help-doc pass.

**i18n parity**: en + ar updated this MIG. The 13 other locale help-docs (if they exist) carry the same banner translation in PJ-014's User Manual backfill batch. Per check, only `docs/help.uConstellation.World/` and `docs/help.ar/` exist for help docs; other locales use the User Manual single-file translations. So: en + ar prepended this MIG; nothing else needed for help.

---

## 6 · Invariants — what must not break

| # | Invariant | Why it matters |
|---|---|---|
| 1 | **v2 component compiles** | `ConstellationSight2.svelte` must still type-check + render if `SIGHT_V2_ENABLED` is flipped to `true`. We don't delete or stub anything inside it. |
| 2 | **All `lensActive = false` reset paths still fire** | They are no-ops when `lensActive` is permanently `false`, but flipping `SIGHT_V2_ENABLED` to `true` must not require re-wiring those paths. |
| 3 | **No regression to other surfaces** | Index, SearchHub, SkyView, Map, Inspector360 — none of them touch v2 Sight wiring. Their `lensActive = false` lines stay in place as harmless cleanup. |
| 4 | **No new IPC** | This MIG is a frontend-only gate. Zero Rust changes. |
| 5 | **No `$effect` loop introduced** | Performance Rule 2. The new gate is a static const read inside an `{#if}` — not a reactive expression that could loop. |
| 6 | **No boot-perf regression** | Importing one tiny module from `src/lib/sight/engine.ts` adds zero cost. The module exports a single boolean literal; tree-shaking + dead-code elimination removes any unreachable branch. |
| 7 | **i18n integrity** | All 15 locales' `settings.plugins.constellationSight*` and `lens.*` strings stay valid. None are deleted. None are referenced from a removed surface. |
| 8 | **Settings round-trip** | Existing users with saved `enabledFeatures.constellationSight: true` should have that value preserved on read+write of settings.json. The constant gates rendering, not state. |
| 9 | **`ConstellationMap` filename collision** | Map's "Sight" mention in tooltips/labels (if any) — must not be removed; Map is not part of MIG-017. |
| 10 | **CE Phase 9 multi-lens isolation** | `lenses.rs::apply_lens` (PJ-013 dead code) is unrelated to v2 Sight. The "Create Lens" Command Palette entry stays. The `availableLenses` / `activeLensId` / `lensGroups` / `lensEntries` Svelte state stays. |
| 11 | **Help-doc banner is non-destructive** | Original v2 documentation paragraphs untouched beneath the banner. |
| 12 | **M11 zero-diff** | `git diff src-tauri/src/lexicon/` returns empty for this MIG. |

---

## 7 · Drift map — what new state the system gains

A "drift" check (per LL-023): does this MIG add any new state, file, schema field, or contract that the rest of the system doesn't yet know about?

| New surface | Consumers | Drift risk |
|---|---|---|
| `src/lib/sight/engine.ts` (new file, 1 export) | `+layout.svelte` (2 import sites: dock button gate + modal mount gate), `SettingsModal.svelte` (1 import site) | Zero. Three explicit consumers, all introduced in this same commit. No future code expected to import this without knowing. |
| Banner block in help doc | None — read-only | Zero. |
| (No new schema field, no new IPC, no new persisted state.) | — | — |

LL-023 risk class: **low**. Pattern matches "single new module imported by N explicit consumers, all wired in the same diff."

---

## 8 · Migration-path map — first-boot, mid-state, rollback

| Scenario | Behavior |
|---|---|
| **Fresh install (no settings.json yet)** | DEFAULT_SETTINGS used. `SIGHT_V2_ENABLED = false` → no dock button, no Settings plugin entry rendered. User never sees v2. |
| **Existing user, `enabledFeatures.constellationSight: true` saved** | Settings load: `enabledFeatures.constellationSight = true`. Dock button gate is now `SIGHT_V2_ENABLED && (enabledFeatures.constellationSight !== false)`. With `SIGHT_V2_ENABLED = false`, the && short-circuits. No dock button. The saved `true` is preserved on next save (round-trip clean). |
| **Existing user, `enabledFeatures.constellationSight: false` saved** | Same end-state — no dock button. The setting was `false` before; the new gate keeps it `false`. |
| **Mid-state: user mid-toggle** | If `lensActive` is somehow `true` at the moment a build with `SIGHT_V2_ENABLED = false` boots — `{#if lensActive && SIGHT_V2_ENABLED}` short-circuits → modal does not render. No null-pointer surface. The state is harmless. |
| **Rollback to v2** | Developer flips `SIGHT_V2_ENABLED = true`, rebuilds. Dock button reappears with the same i18n title. Settings plugin entry reappears. v2 modal opens on click. Behaviour identical to pre-MIG-017. |
| **v3 ships (PJ-038)** | New flag `SIGHT_V3_ENABLED = true` added to `engine.ts`. v3 mounts behind that gate. v2 stays at `SIGHT_V2_ENABLED = false`. Both can coexist in the codebase; only one ever renders (mutual exclusion enforced by the gate, not by code structure). |

No settings migration needed for any path. No schema migration needed. No data migration needed.

---

## 9 · Three-agent audit anticipations

The audit phase will spawn three agents (invariants / drift / migration-path). Anticipated findings:

- **Invariants agent**: should verify all 12 rows of §6 hold. Likely-clean; risk: missed entry point (a 5th surface that grep didn't catch). Mitigation: agent runs a full grep over `src/` for `lensActive\s*=\s*true` + `toggleLens\s*\(` + `ConstellationSight2`.
- **Drift agent**: §7 says drift risk is low. Agent verifies no consumer of the new const is forgotten (e.g., a stray `{#if $appSettings.enabledFeatures?.constellationSight !== false}` that doesn't get the `SIGHT_V2_ENABLED &&` prefix).
- **Migration-path agent**: §8 covers the five scenarios. Agent verifies no settings.json read path silently re-enables v2 (e.g., a "if saved value is missing, default to true" path that inverts the gate intent).

---

## 10 · Out of scope

- **PJ-013 (`apply_lens` / `lenses.rs` dead-code decision)**. Tempting to bundle: removing CE Phase 9 multi-lens machinery while we're "in the lens area." Rejected: separate concern, separate audit surface, separate Boss decision, separate PJ. Bundle later if the cost of two MIGs is prohibitive.
- **PJ-035 / PJ-036 / PJ-037** (Sight content-similarity / layer peeling / Map↔Sight integration). All inheritable into v3 under PJ-038, not v2.
- **v3 Concept Paper**. Lives under PJ-038; this MIG just clears the runway.
- **Help-doc full rewrite**. v3 will own its own help-doc pass; MIG-017 only adds a banner.

---

## 11 · Acceptance criteria (will be re-enumerated in the Plan)

1. `npm run tauri dev` (or production build) shows no Sight dock button.
2. Settings → Plugins → Visualization shows no "Constellation Sight" row.
3. The "Return to Lens" button is unreachable (because v2 was never opened, `lensReturnPending` cannot be set).
4. Help-doc banner displays at the top of the existing v2 help page.
5. Flipping `SIGHT_V2_ENABLED = true` and rebuilding restores v2 behaviour identically.
6. No regression in: Sky View, OrgChart, Map, Index, SearchHub, Inspector360, Knowledge Health Dashboard, Settings, dock buttons for those views, Command Palette, keyboard shortcuts.
7. Three-agent audit clean (invariants / drift / migration-path).
8. Orientation v1.55 → v1.56 bumped inline (SO #6).
9. Pending Jobs v1.4 → v1.5 with PJ-039 closed as Done; PJ-038 unblocked.

---

**End of Architect.** The next document is the Plan, which decomposes the work into a single committable phase (no Boss test gate — this is a UI-hide MIG, not a feature MIG; the audit verifies the hide).

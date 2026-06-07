# MIG-071 — Themes + Saved Styles unification — PLAN (Phase 2)

**Function in hand:** the styling-persistence layer — **Themes** (the 6 built-ins + the user's `customThemes`, the "base coat") and **Saved Styles** (the app-global `style-presets.json` overlays) — collapsed into ONE store with ONE active-look concept, the **Obsidian Community Themes** online import removed. All-local. Working dir `E:\مشاريع كلاود\Constellation`, branch `main`. Date 2026-06-07.

**Option chosen by Eisa: C — "True single store."** Migrate the 6 built-ins + `customThemes` into the style-presets store, retire `activeThemeId` in favour of an active-style-id, rewrite the +layout apply path to compose from the unified store, remove the Obsidian importer. This is the highest-blast-radius option; the plan below makes it SAFE by introducing the replacement first, re-pointing every consumer while the old fields still exist and work, running a **non-destructive** migration, proving parity, and only then deleting the dead old paths — **one at a time, last.**

> **Phase-1 source:** `docs/MIG-071-themes-styles-unification-ARCHITECT.md`. Phase-4 Audit (3 agents) follows the build per the Migration Rule.

> ### REVISION 2 — 2026-06-07 (FINAL, supersedes Option C's "merge into one gallery") — Eisa: REMOVE the theme subsystem entirely; the Style Setter is the sole styling home
> The §C Boss test exposed a sequencing bug, and through it Eisa redirected the whole migration. The end-state is **not** a merged gallery — it is **deleting the Appearance theme layer** and keeping only the Style Setter:
> 1. **Settings → Appearance "Themes" area removed entirely** — gallery, "New Theme", the theme editor, and the **Obsidian import** all go. Appearance keeps its non-theme controls (title alignment, Living-Link lifecycle, …).
> 2. **All theme DATA wiped, NO backup** — the 6 built-ins removed; the user's `customThemes` cleared; `activeThemeId`/`activeStyleId` reset to `''` (plain default base). One-shot, sentinel-guarded migration. (Eisa: "don't keep anything… only the ones in the Appearance.")
> 3. **The Style Setter is UNTOUCHED and becomes the only place styling happens** — its saved Styles (`style-presets.json`), the per-Universe `styleOverride` (current look), and `styleSwatches` (palette) all stay. (Eisa: "the Style Setter is doing better… don't touch it.")
> 4. **Apply path simplifies to "plain default + Setter look"** — no theme resolution; the `§A–§C` `activeStyleId`/`resolveActiveBase` machinery becomes dead and is removed in the `/simplify` pass. Base look = `theme.css` `:root` defaults; `styleOverride` + `liveStyleDraft` apply on top (BUG-015 single writer unchanged).
>
> **Re-sequenced steps (FINAL):** **§D** = empty `BUILTIN_THEMES` + one-shot wipe migration (clear `customThemes`, reset active ids). **§G** = remove the Appearance theme UI (gallery + editor fns + Obsidian card/mount) and **delete** `ObsidianThemeBrowser.svelte` + `obsidianImporter.ts` (§H folds in here). **Build + Boss test the functional end-state.** Then **§K** = `/simplify` the dead theme machinery (apply-path theme branch, `BUILTIN_THEMES` symbol, `deriveThemeVariables`, `themeToStyle`/`unifiedStyleList`/`resolveActiveBase`/`isBaseStyle`, `activeStyleId`, `StylePresetsPanel`) + Sight repaint-trigger reads. **Docs** = the 15-language "Appearance and Themes" help + User Manual rewritten for "no themes; styling lives in the Setter" + orientation bump — after Boss validation. Then Phase-4 Audit.
> **Note:** §A–§C (`activeStyleId` field, resolver, apply-path resolution) are now superseded; they stay in history but their code is removed by §K. LL-032 (Setter render path untouched) + BUG-015 (one writer) hold throughout.

---

## 0. Verified territory (file:line — read, not remembered)

### The single apply path (BUG-015 single writer)
`src/routes/+layout.svelte` L1556-1655 — the ONE `$effect` that writes body CSS vars. Compose order today:
1. `let themeId = s.activeThemeId` (L1559); auto light/dark pairing rewrites `themeId` to `current.pairedThemeId` for the resolved scheme (L1562-1572, reads `[...BUILTIN_THEMES, ...customThemes]` at L1566).
2. find theme = `s.customThemes?.find(id) || BUILTIN_THEMES.find(id)` (L1583-1585 — **customs win on id collision**, the 2026-04-14 fix).
3. `deriveThemeVariables(theme.colors, theme.type)` → set untracked (L1589-1591); `theme-light`/`theme-dark` body class (L1593-1594); `theme.customCSS` `<style>` el (L1596-1606); Style-Settings blocks/values → `trackedVars` (L1607-1617).
4. else-branch: no theme → bare accent vars (L1618-1624).
5. `Object.assign(trackedVars, s.styleOverride ?? {})` (L1629) — per-Universe overlay, tracked.
6. `Object.assign(trackedVars, $liveStyleDraft)` (L1635) — Setter live layer, tracked, wins.
7. `_lastStyleSettingsKeys` diff clears removed tracked vars (L1638-1642); class sweep for `css-settings-*` (L1647-1654).

### Every `activeThemeId` consumer (non-doc, non-archived-orientation)
- `src/routes/+layout.svelte` L1559 (read), L1566 (`customThemes` join for pairing), L1584 (`customThemes` find).
- `src/lib/components/SettingsModal.svelte` L66 `selectTheme` → `updateSettings({activeThemeId})`; L91 `saveTheme`; L100 `deleteTheme` (clears active if deleted); L117 `importTheme`; L2059 card `class:active`; L2089-2090 "Reset to default" → `updateSettings({activeThemeId:''})`. `allThemes` derived at L63 = `[...BUILTIN_THEMES, ...customThemes]`.
- `src/lib/libraries/stylePresets.ts` L77 (`colorsTheme` section captures `activeThemeId`+`customThemes`); L269 (`themeToStyle` emits `activeThemeId`); L325 (`stylePreview` reads it). **Already part of the §A scaffold** — the bridge already speaks `activeThemeId`.
- `src/lib/sight/v6/SightV6.svelte` L1003 `void $appSettings.activeThemeId` — **repaint trigger only** (value unused). `src/lib/sight/v6/MiniDome.svelte` L169 — same. **Both gated behind `SIGHT_V6_ENABLED=false`** (`src/lib/sight/engine.ts`; Sight disabled per MIG-038) — not reachable at runtime today, but must still compile and must re-fire on look-change after the rename.
- `src/lib/libraries/store.ts` L3260 (type field) + L3658 (default `''`).
- `tests/mig-069/stylePresets.test.ts` L12 (fixture).

### Every `customThemes` consumer (the same set, plus)
- `src/routes/+layout.svelte` L1518/L1545 — `_coreBlockCleanupDone` migration that strips dead Style-Settings blocks and writes `customThemes` back.
- `src/lib/components/ObsidianThemeBrowser.svelte` L73/L83 — **to delete** (§5).
- `src/lib/libraries/stylePresets.ts` `applyPreset` merge (L222-227), `themeToStyle` (L271), `unifiedStyleList` (L283-285), `stylePreview` (L326).

### The two live styling UIs (and the dead one)
- **Settings → Appearance gallery** — `SettingsModal.svelte` L2057-2086: static-swatch `<div>`s (`background:{theme.colors.x}`, L2062-2065), **never** `deriveThemeVariables` → LL-032-safe by construction. This is the unified gallery's home.
- **Style Setter** — `StyleSetter.svelte`, mounted full-page at `+layout.svelte` L6866. Loads **raw** `loadStylePresets()` → `savedStyles` (L647), renders them as **name rows only** (L708-730). Explicitly does NOT import `unifiedStyleList`/`themeToStyle`/`stylePreview` (L29-33, L1016) — **this is the LL-032 boundary; the Setter render path stays untouched.**
- **`StylePresetsPanel.svelte`** — renders `stylePreview` self-portrait cards over saved presets. **Not mounted anywhere in `src/`** (no import in `SettingsModal` or `+layout`; only referenced in a `StyleSetter` comment). Orphaned/dead today.

### Write rails (second-screen sync is automatic)
`updateSettings` (store.ts L4097), `setStyleOverride`/`mergeStyleOverride`/`clearAllStyleOverride`/`setPerScriptFont`/`addStyleSwatch`… all do `appSettings.update` → `saveSettings()` (300ms debounced `save_universe_settings`) → `emit('screen:settings-changed', get(appSettings))`. `applyPreset` (stylePresets.ts L198) routes through `updateSettings` + `saveLinkTypes`. The second window re-runs its own copy of the L1556 apply `$effect`. **No new emit needed anywhere if every write keeps routing through these helpers.**

### Storage
- Saved styles: `{app_data_dir}/style-presets.json` via `load_style_presets`/`save_style_presets` (`src-tauri/src/style_presets.rs` L29/L48) — a dumb JSON array, shape owned by the frontend.
- Themes/active id: inside per-Universe settings (`save_universe_settings`), loaded by `applyParsedSettings` (store.ts L3890).
- **Two stores, two files.** Option C unifies the *logical* model and the *read/apply path*; the plan keeps writing themes to settings (so rollback is clean) until the final cleanup steps.

---

## 1. The unified model (how a single entry encodes base vs overlay)

Re-use the **already-coded MIG-070 §A model** (`StylePreset` with `source: 'builtin' | 'theme' | 'style'`, `stylePresets.ts` L36-52). One list, two roles:

- **Base-coat look** = a preset whose `colorsTheme` section carries a full theme (`activeThemeId` + the theme object in `customThemes`, exactly what `themeToStyle` already emits, L255-275). Applying it sets the active theme.
- **Overlay look** = a saved preset carrying partial `styleOverride`/other sections (the MIG-069 Styles). Applying it merges overrides.
- The card shows a **"Theme" vs "Style" tag** from `source` (L44). Both apply via `applyPreset` → `updateSettings`, so the L1556 single writer composes them in its existing order (base → values → override → live). **No new writer, no new compose rule** — this is the BUG-015 guarantee preserved.

**The one new concept Option C adds: an "active style id."** Today the active *base* is `activeThemeId`. We add `activeStyleId: string` to `AppSettings` as the canonical pointer to the currently-applied **base-coat** preset in the unified list. It is the *successor field* to `activeThemeId`, living in the same place (`AppSettings`, written by `updateSettings`). For the entire migration `activeStyleId` is **derived from / kept in lockstep with** `activeThemeId` so the old field stays a valid fallback; only the final cleanup step makes `activeStyleId` the sole source of truth.

> **Why an id and not "just keep activeThemeId"?** Eisa picked C precisely to retire `activeThemeId` as the public concept. A base-coat preset has id `theme:<themeId>` (the §A convention, L260). `activeStyleId = 'theme:' + activeThemeId` for a theme base; the apply path resolves the embedded `activeThemeId`/`customThemes` from that preset. This keeps the +layout writer working off real theme colours while the *pointer* is unified.

---

## 2. Sequencing rule (the core of safety)

1. Introduce the unified active-style concept + read-time union **first**, keeping `customThemes` + `activeThemeId` intact and authoritative (§A, §B).
2. Re-point the apply path + every `activeThemeId` consumer to resolve **through** the unified resolver while old fields still exist (§C, §D, §E).
3. Run the **non-destructive** data migration — build the unified store from existing `customThemes` + built-ins + `style-presets.json`; keep old fields **inert, not deleted** (§F).
4. Move the gallery UI to the unified list (§G).
5. Remove Obsidian (§H).
6. **Delete dead old paths/fields LAST, one at a time, only after parity is proven** (§I, §J).

**Never delete a store/field in the same step that introduces its replacement.** Each step is one commit `§N — <name>` with a Boss-testable verification clause where user-facing.

---

## STEP-BY-STEP PLAN

### §A — Add `activeStyleId` to the settings shape (inert, additive)
**Files:** `src/lib/libraries/store.ts` — `AppSettings` interface (add `activeStyleId: string` next to `activeThemeId` L3260); `DEFAULT_SETTINGS` (add `activeStyleId: ''` next to L3658). No reader yet.
**Migration touch:** `applyParsedSettings` (L3890) — after the `appSettings.set` merge, add a **back-fill**: if `activeStyleId` is empty/absent but `activeThemeId` is set, set `activeStyleId = 'theme:' + activeThemeId` (idempotent — silent once populated). This is pure derivation from an existing field; nothing destructive.
**No UI, no apply-path change yet.**
**Verification:** App boots; themes still apply exactly as before; nothing visibly changes. (Internal: a freshly-loaded settings object now carries `activeStyleId` mirroring the active theme — confirmed by the build compiling and the app launching with no theme regression.)
**Rollback at this stage:** reverting sees original `customThemes`/`activeThemeId`; the unread `activeStyleId` field is harmless.

### §B — Add the unified *resolver* (read-time), keep `themeToStyle`/`unifiedStyleList`
**Files:** `src/lib/libraries/stylePresets.ts`. Add a pure helper `resolveActiveBase(savedStyles: StylePreset[]): StylePreset | undefined` that, given the current `appSettings`, returns the base-coat preset matching `activeStyleId` (falling back to `'theme:' + activeThemeId`) from `unifiedStyleList(savedStyles)`. Add `isBaseStyle(p)` (`p.source === 'builtin' || p.source === 'theme'`) and reuse `isUserStyle` (L295). No store writes; `unifiedStyleList` already assembles built-ins + custom-theme wrappers + saved styles at read time (L280-292) — non-destructive.
**Verification:** Pure-function step, no user-facing change. App boots and themes apply unchanged. (Unit-testable: `resolveActiveBase` returns the Constellation-Dark base preset when `activeThemeId='constellation-dark'`.)
**Rollback:** nothing stored; revert is clean.

### §C — Re-point the +layout apply path to resolve through the unified base, OLD fields still primary
**File:** `src/routes/+layout.svelte`, the L1556 `$effect` ONLY (no second effect — BUG-015).
**New compose order (same single writer, same 4 layers):**
1. **base-from-active-style** — resolve the active base preset via the §B resolver. Extract its theme (`activeThemeId` + embedded `customThemes` from the preset's `colorsTheme` section). For built-ins this is `BUILTIN_THEMES.find`; for theme-wrappers it's the embedded theme object. **Crucially, fall back to the existing `s.activeThemeId` + `s.customThemes` find (current L1583-1585 logic) when no unified base resolves** — so behaviour is identical for every existing user on first boot. Keep the **customs-win-on-id-collision** rule (the 2026-04-14 fix) inside the resolver.
2. derive vars + `styleSettingsValues` + `customCSS` + Style-Settings blocks — unchanged (L1589-1617), now fed from the resolved theme.
3. auto light/dark pairing — unchanged logic (L1562-1572), but applied to the **resolved base theme** (still reads `pairedThemeId`; see §D for where pairing re-homes).
4. `styleOverride` (L1629) → `liveStyleDraft` (L1635) — **untouched**, same order, same `_lastStyleSettingsKeys` clear.
**No second writer introduced.** The effect still reads `$appSettings` + `$liveStyleDraft` and writes `document.body.style` once.
**Verification (Boss-testable):**
- *Define:* "Themes are the colour-and-look base of Constellation. This change moves how the app *finds* the active theme to the new unified path, while keeping the same six built-in themes and any themes you made."
- *Walk-through:* Open Settings → Appearance. Click each of the six built-in theme cards in turn (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark). **Expected:** the whole window recolours instantly to that theme, exactly as before. Click a custom theme you made earlier (if any) — it applies and your colours show. Set the colour-scheme to "System" and toggle your OS dark/light — the paired theme switches automatically. Open the Style Setter, drag a colour slider — the live preview still restyles instantly (no lag). Close it with Keep — your override sticks on top of the theme.
- *Failure mode:* If a theme card click does nothing, or the wrong colours appear, or there's typing/slider lag, the resolver fallback is wrong — stop and fix before the next step.
**Rollback:** revert this one commit → the apply path reads `activeThemeId` directly again; all stored data untouched.

### §D — Re-point the remaining `activeThemeId` *readers* (pairing helper, Sight repaint triggers)
**Files:**
- `src/routes/+layout.svelte` — the auto-pairing block (L1562-1572): factor the "resolve scheme → swap to paired theme" into a small helper that takes the resolved base theme, so pairing lives in one place and reads the unified base. **Light/dark pairing re-homes here** — `pairedThemeId` stays a property of the theme object embedded in the base preset (no schema change; the §A `themeToStyle` already carries the full theme incl. `pairedThemeId`).
- `src/lib/sight/v6/SightV6.svelte` L1003 and `src/lib/sight/v6/MiniDome.svelte` L169 — these are **repaint triggers** (`void $appSettings.activeThemeId`). Change to `void $appSettings.activeStyleId; void $appSettings.activeThemeId;` (read **both** during the migration so the effect re-fires whether the look changes via the new id or the old field). Both files are behind `SIGHT_V6_ENABLED=false` — must still compile; no runtime effect today.
**Verification:** Boots clean; pairing still works (covered by §C test). Sight is disabled so no visible change; the build must succeed with both reads.
**Rollback:** revert → readers read `activeThemeId` only; clean.

### §E — Write `activeStyleId` alongside `activeThemeId` on every base-selection write
**File:** `src/lib/libraries/stylePresets.ts` `applyPreset` (L198-241). When a **base-coat** preset (source `builtin`/`theme`) is applied, also set `activeStyleId = preset.id` in the same `partial` passed to `updateSettings` — so the new pointer and the old field move together (`activeThemeId` from the section, `activeStyleId` = the preset id). Overlay presets don't touch either.
**Why here, not in the gallery:** `applyPreset` is the one apply rail; writing both ids here means the gallery (§G) and any future caller stay correct automatically.
**Verification:** Apply a saved Style that carries a theme — both the theme colours and the active-card highlight update; re-open Settings and the same base shows selected. No second-screen desync (one `updateSettings` → one emit).
**Rollback:** revert → only `activeThemeId` is written; `activeStyleId` falls back via §A's derive on next load.

### §F — Non-destructive data migration: build the unified store, keep old fields inert
**This is the data-migration step. It does NOT delete anything.**
**File:** `src/lib/libraries/store.ts` `applyParsedSettings` (L3890) — add a one-shot, idempotent back-fill guarded by a sentinel (e.g. `mig071Done` in settings, mirroring the existing `v6MigrationDone`/proMode→extended pattern at L3943+):
1. Load `style-presets.json` (via `loadStylePresets`).
2. For each `customThemes` entry not already represented as a `theme:<id>` preset, **append** a derived base preset (`themeToStyle(theme)`) to the saved-styles array. (Built-ins are *not* persisted — they're assembled at read time from `BUILTIN_THEMES`; persisting them would duplicate on every version bump. The unified list shows them already.)
3. Save the augmented `style-presets.json` once.
4. Set the sentinel; set `activeStyleId` from `activeThemeId` (idempotent with §A).
**Idempotent + resumable:** the per-theme "already represented?" check means a re-run (or an interrupt mid-write) adds only what's missing; the sentinel skips the whole block once complete. **Size:** 6 built-ins (not persisted) + a handful of custom themes (tiny) → runs **instantly** inline at boot; **no background task / status strip needed** (Performance Rule 8's heavy-path clause does not apply — confirmed by the data being a few KB of theme objects, not a Universe walk). If a future user has dozens of custom themes the same loop still completes in milliseconds.
**Non-destructive guarantee:** `customThemes` and `activeThemeId` in settings are **untouched** — the migration only *appends* to `style-presets.json` and *adds* `activeStyleId`. Nothing is rewritten or removed.
**Verification (Boss-testable):**
- *Define:* "Your saved Styles and your themes are merging into one list. This step copies your custom themes into that one list without deleting the originals — so nothing can be lost."
- *Walk-through:* (Best tested by Eisa on a Universe that has at least one hand-made custom theme.) Launch the updated app. Open Settings → Appearance. **Expected:** every theme you had before is still there and still applies; if you had saved Styles, they're unaffected. Close and relaunch — no duplicate theme entries appear (idempotency). Your active theme is still active.
- *Failure mode:* If a custom theme vanished from the gallery, or duplicates pile up on each relaunch, the migration is wrong — STOP; this is the one step that could touch user data, and the non-destructive contract was violated.
**Rollback:** revert the build → the app reads `customThemes`/`activeThemeId` from settings (untouched) exactly as before. The extra `theme:<id>` entries appended to `style-presets.json` are harmless to the old build (it ignores unknown presets; they deserialise as plain presets). **A downgrade sees its original themes intact.**

> **DATA-SAFETY FLAG:** §F is the only step that writes user data. The guards: (a) append-only to `style-presets.json`, never rewrite `customThemes`; (b) idempotent per-theme check; (c) sentinel; (d) the existing `applyPreset` merge already proves the non-destructive merge pattern (L222-239). Phase-4 Audit's migration-path agent must replay first-boot, double-boot, and mid-write-interrupt against a Universe with custom themes.

### §G — Move the gallery UI to the unified list (Settings → Appearance)
**File:** `src/lib/components/SettingsModal.svelte`.
- Replace `allThemes` (L63) with the unified list: `unifiedStyleList(loadedSavedStyles)` (load saved styles on modal open, like the Setter does at L647). Render the **same static-swatch card** (L2057-2073) but driven by `stylePreview(p)` colours (still plain `<div>` swatches — **no `deriveThemeVariables`, no live engine per card**; LL-032-safe exactly as the existing grid). Add the **"Theme" vs "Style" tag** from `p.source`.
- `selectTheme` (L65) → `applyPreset(p)` (one rail; sets both ids via §E). `class:active` (L2059) → `p.id === activeStyleId` (fallback `'theme:'+activeThemeId`).
- Theme CRUD buttons (New/Edit/Delete, L2068-2077) stay for **theme-wrapper** entries (`isUserStyle`-false but `source==='theme'`); saved-Style entries get rename/duplicate/delete via the existing stylePresets calls. Keep the editor (`startNewTheme`/`startEditTheme`/`saveTheme`/`deleteTheme`, L69-103) writing to `customThemes` for now (still the source for theme objects; cleanup is §I).
- **LL-032 statement:** this gallery lives in Settings → Appearance, renders static swatches, never calls `deriveThemeVariables` per card, and is NOT the Style Setter. The Setter render path is not touched in this step.
**Verification (Boss-testable):**
- *Define:* "Themes and Saved Styles now appear together in one gallery in Settings → Appearance — each tagged 'Theme' (a full look) or 'Style' (a saved tweak). Clicking any one applies it."
- *Walk-through:* Open Settings → Appearance. **Expected:** one grid showing the six built-in themes (tagged Theme), your custom themes (tagged Theme), and your saved Styles (tagged Style). Click a built-in — the app recolours. Click a saved Style — your saved look applies. Click "New Theme", pick colours, Save — it appears in the grid and applies. Edit then Delete a custom theme — it's removed and, if it was active, the app reverts to default. **No lag, no freeze** when the gallery opens (this is the LL-032 watch point — static swatches must render instantly even with many entries).
- *Failure mode:* If opening the gallery freezes or stutters, a live theme engine slipped into a card — STOP (LL-032). If clicking a Style doesn't apply, the `applyPreset` wiring is wrong.

### §H — Remove the Obsidian Community Themes import
**Files:**
- **Delete** `src/lib/components/ObsidianThemeBrowser.svelte` and `src/lib/theme/obsidianImporter.ts` (confirmed: `obsidianImporter.ts` is imported ONLY by `ObsidianThemeBrowser.svelte`; that component is mounted ONLY by `SettingsModal.svelte`).
- `src/lib/components/SettingsModal.svelte` — remove the **4 refs**: import L9, `showObsidianBrowser` state L61, the Obsidian card L2082-2085, the mount L2472-2476.
- **Keep `source: 'obsidian'` inert** in `ConstellationTheme` (store.ts L3060) so previously-imported themes (now plain `customThemes`, surviving as theme-wrapper presets) **deserialise cleanly**.
- **DO NOT touch** the unrelated note-importer `'obsidian'` refs (`ImporterModal`/`UniverseSetup` — that's vault import, a different feature).
**Verification (Boss-testable):**
- *Define:* "Constellation no longer downloads themes from the Obsidian community gallery (an online dependency we're dropping for a fully-local app). Any Obsidian theme you already imported stays — it's now just one of your custom themes."
- *Walk-through:* Open Settings → Appearance. **Expected:** the "Obsidian Themes" card is gone; New / Import (from file) remain. If you had previously imported an Obsidian theme, it's still in the gallery as a normal custom theme and still applies.
- *Failure mode:* If a previously-imported Obsidian theme disappeared, the `source:'obsidian'` enum was wrongly removed — restore it.
**Rollback:** revert restores both files + the 4 refs; no data was touched.

### §I — Cleanup pass 1: retire the in-Settings theme editor's direct `customThemes` writes → route through the unified store
**Only after §C-§H parity is proven.** **File:** `src/lib/components/SettingsModal.svelte`. Make `saveTheme`/`deleteTheme`/`importTheme`/`startNewTheme` operate on the unified store (theme-wrapper presets) rather than writing `customThemes` directly — so there's **one** write rail for looks. The `customThemes` settings array becomes a **read-through compatibility shim** (still populated by the migration, still read by the apply-path fallback) but no longer the primary write target.
**Verification:** All gallery CRUD (create/edit/delete/import-from-file a theme) still works and persists across relaunch; second screen reflects changes. **Stop here if anything regresses** — this is the first step that changes a *write* target.
**Rollback:** revert → editor writes `customThemes` again; data intact.

### §J — Cleanup pass 2 (FINAL): make `activeStyleId` the sole base pointer; mark `activeThemeId`/`customThemes` legacy
**Only after a full green test cycle on §A-§I, ideally one Boss-confirmed session of daily use.** **Files:** `src/routes/+layout.svelte` (drop the `activeThemeId`/`customThemes` *fallback* in the resolver — `activeStyleId` + the unified store are now authoritative); `src/lib/sight/v6/*` (drop the legacy `void activeThemeId` read, keep `activeStyleId`); `stylePresets.ts` (stop writing `activeThemeId` in `applyPreset`); `tests/mig-069/stylePresets.test.ts` (update fixture).
**Keep the fields in `AppSettings`** (`activeThemeId`, `customThemes`) marked `@deprecated` and still written by the migration shim for **one release** so a downgrade still finds them. Do NOT physically delete the fields in this MIG — schedule that for a later cleanup MIG once no shipped build reads them (the "delete dead paths last" discipline, one field at a time).
**Verification (Boss-testable):**
- *Define:* "The app now uses the single unified store as the only source for which look is active. Themes still work identically; this just removes the old internal pointer."
- *Walk-through:* Full re-run of the §C + §G tests (apply each built-in, custom theme, saved Style; pairing; Setter live edit + Keep). Everything behaves identically.
- *Failure mode:* Any theme that won't apply means a consumer still depended on the dropped fallback — STOP and re-point it.
**Rollback:** revert this commit → the §C fallback returns; because the fields were kept inert (not deleted), the previous stage is fully restored.

### §K — `/simplify` pass on the final diff
Run `/simplify` over the full MIG-071 diff (reuse / dead-code / altitude — quality only, not bug-hunting; `/code-review` is the Phase-4 Audit's job). Remove any now-dead helper left behind (e.g. the orphaned `StylePresetsPanel.svelte` if it's still unreferenced after §G — confirm with a repo-wide search before deleting, and only if truly unused).
**Verification:** build clean; no behaviour change; diff is minimal.

---

## 3. Invariant-by-invariant risk mitigation (every Phase-1 invariant addressed)

| Invariant | How the plan protects it |
|---|---|
| **LL-032** (no themed gallery / live engine in the Setter render path; no `<select>`/cards over `BUILTIN_THEMES`/`customThemes` there) | The unified gallery lives in **Settings → Appearance** (§G) with the **existing static-swatch cards** — plain `<div>` swatches from `stylePreview` colours, **never** `deriveThemeVariables` per card. **The Style Setter (`StyleSetter.svelte`) is not modified** — it keeps its lightweight saved-style **name rows** (L708-730) and keeps NOT importing `unifiedStyleList`/`themeToStyle`/`stylePreview` (L29-33). §D's only Setter-adjacent touch is the Sight repaint trigger, not the Setter. Stated per step in §G and §J. **Watch point:** §G's gallery-open must be lag-free with many entries. |
| **BUG-015** (exactly ONE writer to body CSS vars) | §C rewrites the **single existing `$effect`** (L1556) in place — **no second effect added.** New compose order is **base-from-active-style → derived vars/values → `styleOverride` → `liveStyleDraft`** (the same 4 layers, same `_lastStyleSettingsKeys` clear). All look-writes route through `updateSettings`/`applyPreset`; the gallery never writes body vars directly. Confirmed no second writer in any step. |
| **Base + overlay composition** | Preserved by the unified-entry model (§1): a base-coat preset feeds layer 1 (theme colours + Style-Settings values); an overlay preset feeds `styleOverride`; the +layout order (L1589→1629→1635) composes them. `applyPreset` (the one apply rail) keeps the existing non-destructive `customThemes`/link-palette merge (L222-239). |
| **Light/dark pairing** | Re-homed in §D into a single pairing helper in +layout that reads `pairedThemeId` off the **theme object embedded in the base preset**. `themeToStyle` already carries the full theme incl. `pairedThemeId` (no schema change). Behaviour: System scheme + OS toggle still swaps to the paired theme (verified in §C/§J tests). |
| **Every `activeThemeId` consumer re-pointed** | Enumerated in §0 and handled: apply path + pairing + find (§C, §D); SettingsModal gallery + CRUD + reset (§G, §I); `stylePresets.ts` capture/emit/preview (already §A scaffold; §E writes both ids); SightV6 + MiniDome repaint triggers (§D); store.ts type/default (§A adds successor, §J marks legacy); test fixture (§J). |
| **Second-screen sync** | Untouched and automatic: every look-write goes through `updateSettings`/`mergeStyleOverride`/`applyPreset`, each of which calls `emit('screen:settings-changed', get(appSettings))` (store.ts L4101 etc.). The second window re-runs its own L1556 apply `$effect`. No new emit, no new listener needed (`secondScreen.ts` `notifySettingsChanged`/`onSettingsChanged` unchanged). |
| **Data survival** | §F is append-only to `style-presets.json` + additive `activeStyleId`; `customThemes`/`activeThemeId` are never rewritten until they're inert legacy shims (§J, kept one release). Idempotent + resumable + sentinel-guarded. |

---

## 4. Data migration / rollback summary (concrete, per stage)

- **First boot after update:** §A derives `activeStyleId` from `activeThemeId`; §F (sentinel-guarded, idempotent) appends `theme:<id>` wrappers for each `customThemes` entry to `style-presets.json` and saves once. Built-ins are assembled at read time, never persisted. **Runs inline, instantly** (a few KB of theme objects — not a Universe walk; Rule-8 background-task clause does not apply). No status strip.
- **Idempotent + resumable:** per-theme "already represented?" check + boot sentinel → re-run or mid-write interrupt only adds what's missing; completed migration is a no-op on subsequent boots.
- **Non-destructive at every stage:**
  - After §A-§H: `customThemes` + `activeThemeId` in settings are **byte-for-byte the originals**; only `activeStyleId` was added and `style-presets.json` was appended to. **A rollback (revert any/all of §A-§H) sees its original two stores intact** and the old apply path reads them directly.
  - After §I: theme CRUD writes route through the unified store, but `customThemes` is still populated by the shim and read by the fallback → a rollback still finds themes.
  - After §J: `activeStyleId` is authoritative; `activeThemeId`/`customThemes` are deprecated-but-present shims kept one release → a downgrade still finds them. **No field is physically deleted in MIG-071.**
- **The one step that could corrupt data:** §F (flagged inline). Guards: append-only, idempotent, sentinel, proven merge pattern. Phase-4 Audit migration agent must replay first-boot / double-boot / interrupt on a custom-theme Universe.
- **No step silently rewrites or deletes a user's custom theme or saved Style.** The riskiest *write-target* change (§I) is gated behind proven §C-§H parity and is itself reversible.

---

## 5. What the "Themes" section becomes + Obsidian removal

- **Settings → Appearance "Themes" heading (L2056)** becomes a **single merged gallery** of looks — built-in themes, custom themes, and saved Styles together, each with a **"Theme" / "Style" tag**. Rename the heading via i18n to "Appearance & Themes" / its unified equivalent (Eisa's term `منسق المظهر` / Style Setter naming is separate — this is the *gallery* heading). New / Import-from-file remain; the Obsidian card is removed.
- **Obsidian removal (§H):** delete `ObsidianThemeBrowser.svelte` + `obsidianImporter.ts`; remove the 4 `SettingsModal` refs (import L9, state L61, card L2082-2085, mount L2472-2476); **keep `source:'obsidian'` enum inert** (store.ts L3060) for deserialisation; **do not touch** the note-importer `'obsidian'` refs.

---

## 6. Docs (SO #6 / #2) — which step carries them

- **§G carries the help + manual rewrite** (the gallery is the user-visible change): rewrite `docs/help.uConstellation.World/Appearance and Themes/Appearance and Themes.md` (the unified gallery, Theme-vs-Style tags) **+ all 14 translations `docs/help.{lang}/`** + `docs/User Manual.md` (+ translations) — the 15-language "Appearance and Themes" rewrite rides **in the §G commit**.
- **§H carries the Obsidian-removal doc edit** (remove the "Import Obsidian community themes" paragraph from the same help topic, 15 languages) — in the §H commit.
- **Orientation bump (SO #6):** a MIG ships → bump `docs/Constellation Orientation & Onboarding vX.Y.md` (new file, never overwrite) describing the unified store + retired `activeThemeId` + removed Obsidian importer. The bump rides the **final user-facing commit (§J)**, or §G if Eisa wants it documented at gallery-ship. Update §0/§3 body (apply-path order, consumer list) and the "what Claude has NOT read" list.
- **Session log (SO #1):** each `§N` commit logged in `lab/reports/SESSION-LOG-2026-06-07.md` as it lands.

---

## 7. After the build

- **§K `/simplify`** on the full diff (quality pass).
- **Phase 4 — Audit (3 agents in parallel):** invariants (LL-032 / BUG-015 / base+overlay / pairing / second-screen), drift (new guards the system doesn't know about), migration path (first-boot / double-boot / schema-mismatch / mid-`§F`-interrupt / rollback at each stage). Per the Migration Rule.

---

## 8. Concise step list (for Eisa's approval)

1. **§A — Add `activeStyleId` (inert, additive).** *After this, the app boots and themes apply exactly as before; a new internal pointer mirrors the active theme.*
2. **§B — Add the read-time unified resolver.** *Pure functions; no user-facing change; themes still apply unchanged.*
3. **§C — Re-point the +layout apply path through the unified base (old fields still fallback).** *Every built-in + custom theme still applies instantly; pairing works; Setter live edit + Keep still lag-free.*
4. **§D — Re-point remaining `activeThemeId` readers (pairing helper, Sight repaint triggers).** *Boots clean; pairing works; Sight (disabled) still compiles.*
5. **§E — Write `activeStyleId` alongside `activeThemeId` on base selection.** *Applying a theme-bearing Style updates colours + the selected-card highlight; survives relaunch; second screen stays in sync.*
6. **§F — Non-destructive data migration (append theme wrappers to the styles store; old fields inert).** *Every existing theme still present and applies; no duplicates on relaunch; nothing lost. (The one data-write step — guarded idempotent + sentinel.)*
7. **§G — Move the gallery to the unified list in Settings → Appearance (+ 15-language help/manual).** *One grid of Themes + Styles, tagged; clicking any applies it; New/Edit/Delete work; gallery opens with no freeze.*
8. **§H — Remove the Obsidian importer (+ help edit).** *The "Obsidian Themes" card is gone; previously-imported themes survive as custom themes.*
9. **§I — Cleanup pass 1: route the Settings theme editor through the unified store.** *Theme create/edit/delete/import-from-file still work and persist; second screen reflects changes.*
10. **§J — Cleanup pass 2 (final): make `activeStyleId` authoritative; mark `activeThemeId`/`customThemes` legacy shims (kept one release) (+ orientation bump).** *Full re-run of theme/Style/pairing/Setter tests — identical behaviour.*
11. **§K — `/simplify` the diff; then Phase-4 Audit (3 agents).** *Build clean, minimal diff, no behaviour change.*

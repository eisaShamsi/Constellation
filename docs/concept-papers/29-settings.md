# 29 — Settings (Concept Paper)

> Follows the template in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3 and serves [00-Constellation](00-Constellation-Core-Concept-Paper.md). Settings is the **control surface** for the whole app: it doesn't create knowledge, it tunes how the gate (the Editor) and every satellite behave. It is the place where `enabledFeatures` toggles live — so it is also, literally, the switchboard for bring-up.

## 1. Function in hand
The **Settings** modal — `src/lib/components/SettingsModal.svelte`, opened from the dock gear button / command palette (`showSettings` in `+layout.svelte`). A left-dock-nav, multi-section preferences panel over the per-Universe `appSettings` store, with sub-panels mounted from it (`IconOverrideSettings`, `ArabicOverridesPanel`, `PerLibraryCalibrationView`, the Style Setter overlay).

## 2. Purpose
The ONE job: **let the user shape how Constellation behaves** — editor options, language, fonts/scripts, link-lifecycle tuning, intelligence (AI/CECE), security/PIN, panels layout, hotkeys, and the **plug-in (`enabledFeatures`) on/off switches**. It serves no single Act directly; it is **meta** — it configures the instruments the Five Acts are performed *with*. Justified: *File Over App* + *Local-First* mean preferences are the user's, stored per-Universe on disk; and *Constraint as Design* means the plug-in toggles here are how a user turns satellites off to keep the app fast and focused. Without it, every behavior would be hard-coded.

## 3. What it is NOT
- **Not** a knowledge surface — it edits no `.md` note, creates no link, computes no derived view.
- **Not** the styling engine — MIG-070/071 moved all theme/CSS work to the **Style Setter** (the `stylesetter` nav item opens that full-page overlay; there is no inline theme editor here anymore).
- **Not** a per-note or per-tab control — it is **per-Universe** global state (`appSettings`), persisted once per Universe.
- **Not** the second-screen's own settings — it is the single authority; the second screen is *notified*, it does not re-edit.

## 4. Wiring
- **Inputs (stores read):** `appSettings`, `aiSettings`, `libraries`, `libraryStats`, `systemFonts`, `commands` (prop, for the hotkeys list), `locale`/`SUPPORTED_LOCALES`, `SIGHT_V2_ENABLED`/`SIGHT_V3_ENABLED` flags.
- **Inputs (IPC):** `classifier_scan_status`, `pick_folder`, `list_lenses`, `read_universe_settings` (via `loadSettings` at boot, in the store — not the modal), plus the write-journal/version reads (`readWriteJournalStats`, `getVersion`, updater `check`).
- **Outputs (IPC):** `save_universe_settings` (debounced 300 ms via `saveSettings()`), `classifier_scan_start`, `save_lenses`, updater `relaunch`. Security writes via `updateSecuritySettings`.
- **Outputs (events):** `screen:settings-changed` emitted on every `updateSettings`/`updateSecuritySettings`; `notifySettingsChanged({...})` to propagate locale/font changes to the second screen.
- **Consumers:** the Editor (reads `appSettings.editor.*`, fonts, scripts), every panel that honors `enabledFeatures`, the second screen (via the emit/`notifySettingsChanged`), the Style Setter, CECE/AI engine, the security PIN gate.
- **Connection to the Editor (the gate):** **indirect, read-only.** Settings writes to `appSettings`; the Editor and panels *react* to that store. Settings never touches note content, the save path, or the Editor's in-memory model — it only changes the configuration the Editor consults. This is the safe relationship: the control surface tunes the gate without ever writing through it.

## 5. Right-click / context menu
- **None.** Grep for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` in `SettingsModal.svelte` returns **no matches**. Every action is a left-click on a labeled control (toggle, select, button, nav item).
- **Gap assessment: acceptable, not a debt.** A preferences panel is an explicit-control surface; there is no per-row "target kind" that would benefit from a right-click action menu the way the file tree, tabs, or note panels do. No action here is reachable only by right-click, and none *should* be. If a future section grows a list of user objects (e.g. custom font sets, lenses) with per-item operations beyond the inline edit/delete buttons it already has, that list — and only that list — should adopt the shared `<ContextMenu>` / `buildContextMenu` (MIG-077), never a hand-rolled menu. Flag at bring-up only if such a list appears.

## 6. Multilingual
- **Mostly localized.** All section labels, plug-in names/descriptions, and control labels flow through `$t('settings.…')`. The locale switcher iterates `SUPPORTED_LOCALES` and calls `setLocale()`.
- **Hardcoded-English risk — flagged.** Many `$t()` calls carry an inline English fallback (`$t('settings.sections.links') || 'Links'`, `… || 'Sight'`, dozens more). If a key is **missing from a locale file**, the user sees English — the fallback masks i18n gaps rather than failing loud. Bring-up MUST verify every `settings.*` key exists in **all 15 locales** (ar de en es fa fr he hi ja ko pt ru tr ur zh) so no fallback ever fires. Treat each `|| 'English'` as a key-coverage TODO, not a finished string.
- **RTL.** Handled at the **document level**: switching to an RTL locale flips the page `dir`, and the modal carries `:global([dir="rtl"])` rules for the toggle sliders (`.toggle-slider::after` left/right swap). There is **no per-field `detectDir()`** — correct here, since labels are UI chrome in the active locale, not mixed-script user content. Verify the whole modal mirrors cleanly in Arabic/Hebrew at bring-up.
- **Native equivalents:** per the standing order, script/section names must use the right native term in each locale (e.g. مصادر), not a transliteration — verify in the locale files.

## 7. Boot behavior
- **Runs at boot?** The **modal does not** — it mounts lazily under `{#if showSettings}` only when the user opens it. What runs at boot is the **store's `loadSettings()`** (one `read_universe_settings` IPC) that hydrates `appSettings` from the persisted per-Universe JSON.
- **Rule 8 status: ✅ reads-persisted.** Settings are **stored**, not recomputed. `loadSettings` reads `read_universe_settings`; `saveSettings` writes `save_universe_settings` (debounced). The modal renders directly from the `appSettings` store value — it derives nothing universe-wide and re-walks nothing. No `scan_*`/`rebuild_*` anywhere. Compliant.
- **Cost:** boot cost is the single `read_universe_settings` read (small JSON; **estimated ~1–5 ms**, not measured). Opening the modal mounts a ~3,200-line component once; **estimated** a few ms, off the hot path. Per-change cost: one debounced `save_universe_settings` + one event emit.

## 8. Flag / gate & bring-up position
- **Gate today:** **none on the modal itself** — Settings is core, always available (it is *where* the other gates live). Internally it gates its own sections: the Sight nav items/plug-in cards are wrapped in `SIGHT_V2_ENABLED` / `SIGHT_V3_ENABLED`, and the plug-in cards write `enabledFeatures.{id}`.
- **Bring-up phase:** **1 (Core spine)** — it must be on early because it is the switchboard that flips every *other* function's `enabledFeatures` gate. Depends on: the `appSettings` store + `read/save_universe_settings` IPC, and the i18n bundle. Its individual sections come alive as their subsystems are brought up (a Sight section is meaningless until Sight is re-enabled).

## 9. Budget
- **Boot budget:** the `read_universe_settings` hydrate must stay within the shell's pre-paint envelope (sub-10 ms target); the modal contributes **zero** until opened.
- **Interaction budget:** opening the modal and switching sections must be instant (no perceptible lag); every toggle/select writes via the **debounced** `saveSettings` (300 ms) — never one IPC per keystroke (Rule 3). The Style-Setter live preview path is in-memory (zero IPC on slider drag).
- **Regression guard:** open Settings, switch through every section (no stutter); toggle a plug-in and confirm exactly one debounced `save_universe_settings`; switch locale and confirm the modal + second screen update; confirm no settings change ever fires a note write or reindex.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** every section's controls read and write the right `appSettings` field; changes persist across restart.
- [ ] **Serves Constellation's core purpose:** the `enabledFeatures` toggles correctly enable/disable each satellite (the switchboard works); *Constraint as Design* honored.
- [ ] **Wires correctly to the Editor (the gate):** changing an editor/font/script setting updates the live Editor with no note write, no reindex, no model touch.
- [ ] **Right-click present + correct:** N/A by design (explicit-control surface); confirmed no action is right-click-only. Any future per-item list uses shared `<ContextMenu>`, never hand-rolled.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** every `settings.*` key exists in all 15 locales (no `|| 'English'` fallback ever fires); the modal mirrors cleanly in RTL; native terms verified.
- [ ] **Within budget:** boot hydrate within envelope; section-switch instant; writes debounced; Style-Setter preview does zero disk IPC.
- [ ] **Obeys Rule 8:** reads persisted settings; recomputes no universe-wide derived view.
- [ ] **Holds its invariants:** per-Universe scope (no per-note leakage); single authority (second screen is notified, never re-edits); `screen:settings-changed`/`notifySettingsChanged` fire on every change.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** (boot hydrate cost estimated, not measured) · Notes: Right-click correctly absent (explicit-control surface). **Central bring-up watch item:** the pervasive `$t('…') || 'English'` fallbacks must be resolved to real keys in all 15 locales so no English ever leaks. Rule 8 clean. Styling lives in the Style Setter now (MIG-070/071), not here. Wiring to the Editor is read-only via `appSettings` — Settings never writes through the gate. Sub-panels (`IconOverrideSettings`, `ArabicOverridesPanel`, `PerLibraryCalibrationView`) are folded here; the Style Setter has/needs its own paper. Several IPC names and the exact RTL mirror behavior should be re-verified live in bring-up.

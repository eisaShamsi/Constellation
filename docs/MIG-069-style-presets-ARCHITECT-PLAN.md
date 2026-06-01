# MIG-069 — Style Presets ("Portable Styles")

**Status:** Plan approved (Eisa, 2026-06-01). Cascading build.
**One-line:** Named, app-global, switchable bundles of *style* configuration — section-choosable per preset, exportable/importable as a `.json` file, reusable across every universe.

---

## Why
A user runs many universes; each carries their personal visual style (theme, fonts, link-type colours, pill shape, typed-link display). Today, re-styling a new universe means redoing every step by hand — exactly Obsidian's per-vault friction. We give users **named "Styles"** they can save, switch with a click, export to a file, and share — the VS Code *Profiles* model, expressed the Local-First / File-Over-App way (a portable file you own, no cloud).

## Research (proven patterns — cross-checked before designing)
- **VS Code Profiles** — named customization bundles, app-global, switch from a dropdown, export/import a file, share by handing over the file. The closest match; Eisa chose its *section-picking* flavour. <https://code.visualstudio.com/docs/configure/profiles>
- **Obsidian** — appearance + CSS snippets live per-vault in `.obsidian/`; **no built-in switcher** — the friction we beat by being app-global. <https://obsidian.md/help/snippets>
- **Power BI themes / WordPress theme.json** — confirm a **named JSON file** as the portable unit. <https://learn.microsoft.com/en-us/power-bi/create-reports/desktop-report-themes>

## The key architectural insight
Constellation already keeps the **universe registry** app-global at `{app_data_dir}/universes.json` (`universe.rs::registry_path`/`load_registry`/`save_registry`). Style presets ride the identical rails at **`{app_data_dir}/style-presets.json`** — global, reusable across universes, with **zero invented infrastructure**. Per-universe config stays where it is (`<universe>/.constellation/settings.json` and `link-types.json`); a preset is *applied to* the current universe.

## Data model
```
StylePreset = {
  id: string, name: string, icon?: string,
  schema: "constellation-style/1",
  createdAt?, updatedAt?,
  sections: { <sectionKey>: <captured values> }   // only the ticked sections are present
}
```
Stored app-global as an array. **Export** = one preset written as a standalone `.json`.

### Section catalogue (section → real config)
| Section | Source |
|---|---|
| `colorsTheme` | appSettings: colorScheme, accentColor, activeThemeId, customThemes, iconOverrides |
| `fonts` | appSettings: interface/text/mono fonts + sizes, scriptFonts, font sets, font mode/theme, scripts, numeralStyle |
| `linkColors` | the **registry** (`link-types.json` deltas) — the link palette |
| `pillShape` | appSettings.linkPills.shape (radius/height/fontWeight) |
| `typedLinkDisplay` | appSettings: colourTypedLinks, showTypedLinkLabels |
| `skyView` | appSettings.skyView.* |
| `layout` | appSettings: panelPlacements, pane widths, titleAlignment, focus |
| `behaviour` *(optional, default off)* | editor toggles, link format, etc. — **never** security/githubToken |

## Apply path (live, via existing rails)
- appSettings sections → `updateSettings(partial)` (auto-saves to the universe + emits `screen:settings-changed` → second screen).
- `linkColors` → `save_universe_link_types(deltas)` → re-seeds the registry → editor + panels update **live** (the MIG-067 follow-up live-rebuild).
- Sections **absent** from a preset are left untouched (partial apply).

## Invariants (must not break)
1. **Partial apply** — an absent section never clobbers that aspect of the current universe.
2. **Privacy** — `security`/`githubToken` never travel in a preset (sharing leaks nothing).
3. **Graceful import** — malformed/foreign/old files fail safely (schema-versioned + validated); never corrupt.
4. **App-global** — presets survive universe switches; never written into a universe.
5. **No boot/perf regression** — presets read only when the Styles UI opens, never on boot (Rule 8).
6. **Cross-window** — applying a preset propagates to the second screen via the existing event.

## Options considered & rejected
- *appSettings-only presets* — loses the link palette (which Eisa explicitly wants portable). ✗
- *single "full clone", no sections* — Eisa chose section-picking for shareable-theme vs full-clone flexibility. ✗

## Plan (each phase = one landable commit)
- **A — Foundation:** Rust `style_presets.rs` (`load_style_presets`/`save_style_presets`, mirrors the universe registry; registered in `lib.rs`); frontend `stylePresets.ts` (types + section catalogue + load/save).
- **B — Engine:** `captureCurrentStyle(sections)` + `applyPreset(preset)` (+ Vitest).
- **C — Styles UI:** a "Styles" section in Settings — list, Apply, Save-current-as-new (tick sections + name), Rename, Duplicate, Delete. **[Boss test]**
- **D — Export / Import:** export a preset → `.json` (rfd save dialog); import → validate → add (rfd open dialog). The share story. **[Boss test]**
- **E — Starters + polish:** 1 built-in starter (the default look); i18n (15 languages); help docs; orientation v-bump.
- **F — Audit + PCS:** 3-agent invariants/drift/migration-path; push + milestone tag + ZIP.

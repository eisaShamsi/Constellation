# MIG-070 §C — Style/Theme Settings Audit & Merge Plan

**Goal (Eisa):** make the **Style Setter** the *single* styling surface in Constellation. Cross-check it against **Appearance** + **Style Settings** (and the rest of Settings) so nothing is lost, then merge.

**Method:** 3 parallel read-only audit agents swept the Settings UI (2026-06-03). Findings below are verified against the actual source.

---

## 1. The core problem: THREE styling surfaces, THREE storage models

| Surface | Where | What it edits | Storage | Persisted? |
|---|---|---|---|---|
| **Style Settings tab** | Settings → Style Settings (`StyleSettingsPanel.svelte` + catalog `constellationStyleSettings.ts`) | 75 CSS-variable controls (5 blocks) | `theme.styleSettingsValues` (per active theme) → `generateStyleSettingsCSS` → `document.body.style` | **Yes** (per theme, in `customThemes`) |
| **Style Setter** (MIG-070) | Appearance → "✦ Open Style Setter" (`StyleSetter.svelte`) | live click-to-edit elements; ~25 vars **not** in the catalog | `draft` → on Apply, written **directly** to `document.body.style` | **No** (session-only) |
| **Style Presets** (MIG-069) | Appearance → `StylePresetsPanel.svelte` | named bundles of **appSettings** fields (8 sections), NOT CSS vars | `{app_data}/style-presets.json` | Yes (app-global) |

**The merge's central decision:** the Style Setter currently **does not persist**. To become the only styling surface it must persist, absorb the catalog, and reconcile these three storage models. **Recommended: adopt the catalog's proven path** — the Style Setter writes to the active theme's `styleSettingsValues` and applies via `generateStyleSettingsCSS`; add the ~25 Setter-only vars to the catalog. One storage, one apply path, already battle-tested.

---

## 2. CHECKLIST — everything that must survive the merge

### A. Theme management (Appearance tab) — *keep, move into Style Setter*
- [ ] Theme gallery (built-in + custom cards) · select active theme (`activeThemeId`)
- [ ] New theme · Import theme · **Obsidian Themes browser** · Reset to default · Edit · Delete · Export
- [ ] Theme editor: 5 colour pickers (background/surface/text/accent/border) + name + light/dark type
- [ ] (the 4 quick-themes already in the Style Setter are a subset of this)

### B. The CSS-variable catalog (Style Settings tab) — 75 settings, *the spine of the merge*
- **Colors (19):** background-primary/-alt/-secondary/-secondary-alt, modifier-hover/-border/-form-field; text-normal/-muted/-faint/-on-accent/-error/-warning/-success; interactive-accent/-hover, text-accent
- **Typography (13):** font-interface-size, font-text-size, font-monospace-size; h1–h6 size; heading-weight; line-height-normal, line-height-tight, paragraph-spacing
- **Layout & Shape (8):** radius-s/m/l; border-width; shadow-s/-l; file-line-width; file-margins
- **Components (≈40):** sidebar (width/bg); **dock** (width/btn-size/icon-size/radius/icon-color/bg); **sidebar toolbar** (height/bg/btn-size/icon-size/radius/icon-color); **layout bar** (bg/height/btn-size/icon-size/radius/icon-color/active-color); **top bar + tabs** (topbar height/bg; tab font-size/height/bg/color/active-bg/active-color/border); **file explorer** (ft-master font-size/weight/color/row-padding); **right sidebar** (bg, tabs-bg, tab-height, icon-size, tab-color, active-color); tab-radius; **buttons** (radius/pad-x/pad-y); **tags & callouts** (tag radius/bg/color, callout-radius)
- **Editor (8):** link-color/-hover/-decoration; code-background/code-normal/code-block-radius; blockquote-border-color/-width; caret-color; text-selection
- **+ theme-specific blocks** from imported Obsidian themes (`/* @settings */` YAML) rendered after the core blocks
- **+ Style-Settings JSON** import / export / copy / paste

### C. Fonts — *live in the LANGUAGE tab, not Appearance!*
- [ ] Font families: interface / text / mono (via **Font Sets** + **Custom Font Sets** editor)
- [ ] Per-script font assignment (`languageFontSets`, primary + secondary script)
- [ ] **Font Theme** (Default / Typewriter)
- [ ] Numeral style (Arabic / Hindi)
- *(The Style Setter exposes interface/text font-family pickers; it has NO per-script fonts, NO font-theme, NO numeral style.)*

### D. Link styling (Appearance tab)
- [ ] **8 typed-link colours** + add/delete custom types + parent nesting + reset (`LinkTypesEditor`)
- [ ] Typed-link display toggles: show labels, colour-by-type
- [ ] Living-Link **Pill shape**: radius / height / text-weight / reset

### E. Style Presets (Appearance tab) — named styles
- [ ] Preset gallery: Apply / Rename / Duplicate / Delete / Export / Import / "Save current style…" with per-section include ticks (8 sections)
- [ ] (Style Setter already has a lightweight "save as named style" — overlaps; unify)

### F. Visual settings in OTHER tabs (decide: fold in, or leave as plugin-functional)
- **Editor tab:** readable line length (the only reading-width control), properties-in-document, line numbers, indentation guides, floating toolbar
- **Sky View tab:** node size, label visibility, **label font size**, link thickness, physics (repel/force/distance), show orphans
- **Sight tab** (flag-off): projection, milky way, calendar rings, labels
- **Panels tab:** panel placement (layout, 11 panels)
- **App Icons tab:** per-slot icon glyph overrides (no colour/size)

### G. Has data + apply path but NO editor UI today (gaps to fill in the merge)
- [ ] **`accentColor`** — applied everywhere, but no picker in Settings (only the theme editor's accent)
- [ ] **`theme.customCSS`** — injected, but no CSS editor (only set via Obsidian import)
- [ ] **Per-library appearance** (`libraryAppearances`: accent + per-library interface/text/mono fonts + per-library CSS theme) — read & applied, but **not editable anywhere**
- [ ] **`colorScheme` dark/light/system** toggle — only a dock command, not in Settings

### H. Behavioural (NOT styling — likely keep in their own tabs)
- Living-Link **Lifecycle**: decay toggle, half-life, confidence back-fill
- Editor behaviour: tab size, autopair, spellcheck, link format, etc.

---

## 3. Cross-check: Style Setter vs the catalog

### Already covered by the Style Setter (20 catalog settings)
interactive-accent · background-secondary · background-primary · text-normal · font-text-size · font-monospace-size · link-color · link-decoration · h1–h6 size · heading-weight · ft-master color/font-size/weight/row-padding · code-background · code-normal · statusbar bg/color/font-size/height

### Style-Setter-only vars (≈25) — NOT in the catalog (must be ADDED to it on merge)
`--editor-text-color` · `--bold-color` / `--bold-weight` · `--italic-color` · `--strikethrough-color` / `--strikethrough-thickness` · `--blockquote-text-color` · `--ft-master-font-family` · `--ft-row-radius` · `--ft-border-width/-style/-color` · `--ft-{library,folder,cuniverse}-color/-font-family/-font-size/-weight` · `--universe-bar-color/-bg/-font-family/-font-size` · `--font-interface-theme` / `--font-text-theme` / `--font-monospace-theme` (font families) · derived `--accent-h/s/l` / `--text-accent` / `--interactive-accent-hover`

### Catalog settings NOT yet in the Style Setter (the gap to build)
- **Colors:** background-primary-alt, -secondary-alt, modifier-hover/-border/-form-field; text-muted/-faint/-on-accent/-error/-warning/-success; interactive-accent-hover, text-accent
- **Typography:** font-interface-size, line-height-normal, line-height-tight, paragraph-spacing
- **Layout & Shape (all):** radius-s/m/l, border-width, shadow-s/-l, file-line-width, file-margins
- **Components (most):** sidebar width/bg; **dock** (#2); **sidebar toolbar** (#3); **layout bar** + **top bar/tabs** (#4); **right sidebar** (#new); tab-radius; buttons; tags & callouts
- **Editor:** link-color-hover, code-block-radius, blockquote-border-color/-width (Setter has only blockquote *text* colour), caret-color, text-selection

---

## 4. Proposed merge architecture (for a /migration)

This is **/migration-worthy** (storage reconciliation + frozen MIG-069 presets path + cross-surface invariants). Proposed shape:

1. **One storage + apply path:** the Style Setter persists to `theme.styleSettingsValues` and applies via `generateStyleSettingsCSS`. Add the ~25 Setter-only vars to `constellationStyleSettings.ts`. Wire the editor's `livePreviewTheme` reads to those catalog vars (already mostly done in §3A/§3B).
2. **Element coverage:** every catalog setting becomes a clickable element/control in the Style Setter, organised under the **categories (Surfaces)** just built (Interface / Editor / + new groups: Colors, Typography, Shape, Components-chrome). Add the gap controls (Layout & Shape, dock, toolbars, tabs, right sidebar, buttons, tags).
3. **Themes + Presets + Styles → one "Styles" gallery** (MIG-070 §A `unifiedStyleList` already drafts this): built-in themes + custom themes + saved styles in one list; selecting applies; "save current" persists. Fold theme management (gallery/CRUD/Obsidian-import) into the Setter.
4. **Fonts:** bring font-family pickers (interface/text/mono), per-script fonts, font-theme, and numeral style into a "Typography/Fonts" category (sourced from the Language tab's model).
5. **Link styling:** the 8 typed-link colours + display toggles + pill shape become a "Links" category.
6. **Fill the no-UI gaps:** accent picker, dark/light/system toggle, custom-CSS editor, **per-library appearance editor**.
7. **Retire** the old Appearance styling controls + the Style Settings tab once parity is reached (Eisa: "the only Style Settings"). Behavioural toggles stay in Editor/Appearance.

---

## 5. Phase-1 Architect corrections + chosen approach (2026-06-03)

**Eisa's decisions:** persistence = theme base **+ per-Universe override**; scope = **all in one migration**; execution = **formal /migration**.

**Architect corrected two §3/§4 claims (verified against source):**
- `SecondScreenPage.svelte` applies only the theme light/dark **class + font vars** on `documentElement` — it does **NOT** call `deriveThemeVariables`/`generateStyleSettingsCSS`. So full colour/style-var mirroring to the second screen is a **gap to build**, not an existing behaviour.
- `libraryAppearances` (`LibraryAppearance`, store.ts ≈L2256) is **loaded but has no DOM apply path** in the frontend. Both the editor UI *and* the apply path are missing.
- `deriveThemeVariables` (≈L3100) already emits ~8 of the "25 Setter-only" vars (`--editor-text-color`, `--text-accent`, `--interactive-accent-hover`, `--accent-h/s/l`, `--background-*-alt`, `--text-muted/-faint`). So only **~17 vars are truly new** to add to the catalog.

**Chosen sub-options (architect-recommended, all low-risk):**
- **A1** — per-Universe override = new `appSettings.styleOverride: Record<string,string>`, merged **on top** of the theme's `styleSettingsValues` in the same `+layout` apply `$effect` (after `deriveThemeVariables`), registering its keys in `_lastStyleSettingsKeys` so it survives theme switches and clears cleanly.
- **B1** — add only the ~17 truly-new vars to `constellationStyleSettings.ts`; reuse the derived ones.
- **C1** — fold Themes + Presets into one "Styles" gallery via the existing `unifiedStyleList` (read-time merge; keep both stores; frozen MIG-069 untouched).
- **D1** — extend the standalone `StyleSetter.svelte` overlay; retire the old Appearance/Style-Settings styling tabs only at parity.

**Top invariants:** existing themes load; `styleSettingsValues` apply unchanged; **frozen MIG-069 presets keep working until retired**; a persisted look **survives theme switch** (override re-applies after derivation); no boot/typing/IPC regression (7,600-note Universe); RTL/i18n; FocusPane plain-text exception. **Mid-migration: keep BOTH old tabs and the new surface writing the same values until parity — never delete the old apply path first (BUG-015-class race).** Rollback is non-destructive: older builds ignore the unknown `styleOverride` key (preserved in JSON round-trip), themes/presets still load.

### Open decisions for Eisa
- **D1 — Persistence:** per-theme (`styleSettingsValues`, recommended) · app-global · per-Universe?
- **D2 — Scope:** does the single surface absorb ALL of {catalog, themes, presets, fonts, link colours, no-UI gaps}, or a phased subset first?
- **D3 — Heavy-surface visuals** (Sky View graph appearance, etc.): fold in, or leave in their plugin tabs as functional settings?
- **D4 — Per-library appearance:** build the missing editor inside the Style Setter (per-Universe / per-library scope)?
- **D5 — Execution:** run this as a formal `/migration` (architect → plan → build → audit), given the storage reconciliation + frozen presets path.

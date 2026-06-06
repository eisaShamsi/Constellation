# MIG-070 §C — Settings ↔ Style Setter duplication audit (Phase 9 parity prep)

**Written:** 2026-06-06 · **For:** Eisa's request — *"cross-check the whole Settings to locate any possible duplication with the Style Setter."* · **Method:** an Explore agent enumerated every styling control in `SettingsModal` + `StyleSettingsPanel` + `StylePresetsPanel` + `LinkTypesEditor` + the catalog; cross-referenced here against the Style Setter's `ELEMENTS`/categories. · **Purpose:** the inputs to **Phase 9** (retire the old controls at a 1:1 parity gate).

## Headline

The Style Setter now **duplicates almost the entire styling surface of Settings.** The biggest overlap is the **Style Settings tab** — ~95% of its CSS-variable controls are now also in the Setter (it writes the same vars via `styleOverride`). Parts of **Appearance** and the **Language** font controls are duplicated too. A short list of Style-Settings-only controls remains (the Setter would need to cover them, or they stay, before the tab can retire).

---

## A. DUPLICATED — adjustable in BOTH Settings and the Style Setter

| Settings control(s) | Style Setter equivalent | Same var(s)/store |
|---|---|---|
| **Style Settings → Backgrounds & Surfaces** (7) | Global → Backgrounds | `--background-primary/-alt`, `-secondary/-alt`, `-modifier-hover/-border/-form-field` |
| **Style Settings → Text colours** (7) | Global → Text shades + status | `--text-normal/-muted/-faint/-on-accent/-error/-warning/-success` |
| **Style Settings → Accent** (3) | Editor → Accent + Global → Accent shades | `--interactive-accent/-hover`, `--text-accent` |
| **Style Settings → Font sizes** (3) | Global → Type (interface) · Editor → Body text (text) · Editor → Inline code (mono) | `--font-interface-size`, `--font-text-size`, `--font-monospace-size` |
| **Style Settings → Headings** (H1–H6 + weight) | Editor → Heading 1…6 | `--h1-size…--h6-size`, `--heading-weight` |
| **Style Settings → Rhythm** (line-heights, spacing) | Global → Type & rhythm | `--line-height-normal/-tight`, `--paragraph-spacing` |
| **Style Settings → Corners / Border / Editor width** | Global → Shape & corners | `--radius-s/m/l`, `--border-width`, `--file-line-width`, `--file-margins` |
| **Style Settings → Sidebar / Dock / Toolbar / Layout bar / Top bar+Tabs / Right sidebar / Status bar / File explorer** | Components (cSidebar, cDock, cToolbar, cLayoutBar, cTabs, cRightSidebar) · Interface (fileTree, statusbar) | all `--sidebar-*`, `--dock-*`, `--sidebar-toolbar/-btn-*`, `--layout-*`, `--topbar-*`/`--tab-*`, `--rs-*`/`--right-sidebar-*`, `--statusbar-*`, `--ft-master-*` |
| **Style Settings → Tab shape / Buttons / Tags & callouts** | Components → Top bar & tabs · Buttons · Tags & callouts | `--tab-radius`, `--button-*`, `--tag-*`/`--callout-radius` |
| **Style Settings → Links** (colour, decoration) | Editor → Link | `--link-color`, `--link-decoration` |
| **Style Settings → Code** (bg, text) | Editor → Inline code | `--code-background`, `--code-normal` |
| **Appearance → Living Link Pills** (radius, height, weight) | Links → pill shape | `linkPills.shape.{radius,height,fontWeight}` |
| **Appearance → Typed Link Display** (colour, labels) | Links → toggles | `colourTypedLinks`, `showTypedLinkLabels` |
| **Appearance → Link Types editor** | Links → Typed-link colours (**same component embedded**) | the link-type registry — *one source, not a real duplicate* |
| **Appearance → Styles** (save/apply/rename/delete/export) | Setter → Saved styles (**same MIG-069 engine**) | `style-presets.json` — *same engine, two front-ends* |
| **Appearance → Interface / Note font size** (quick sliders) | Global → Type · Editor → Body text | `--font-interface-size`, `--font-text-size` |
| **Language → per-script font sets** | Global → Per-script fonts | `appSettings.perScriptFonts` (Setter) vs `languageFontSets` (Settings) — *related, see note* |

## B. STYLE-SETTINGS-ONLY — the gaps the Setter does NOT yet cover

These ~5 control-groups are in the Style Settings tab but have **no Setter element** yet — they must be added to the Setter (or consciously dropped) **before** the Style Settings tab can retire at parity:

| Control | Var(s) | Note |
|---|---|---|
| **Cursor & selection** | `--caret-color`, `--text-selection` | no Setter element |
| **Link hover colour** | `--link-color-hover` | Setter has `--link-color` + decoration, not hover |
| **Code-block radius** | `--code-block-radius` | Setter's Inline code has bg/text/font/size, not block radius |
| **Blockquote border** | `--blockquote-border-color`, `--blockquote-border-width` | Setter's Blockquote has *text* colour only, not the bar |
| **Shadows** | `--shadow-s`, `--shadow-l` | Setter's Shape has radii/border, not shadows |

## C. SETTINGS-ONLY — NOT duplication (stays in Settings, by design)

- **Themes** (select / create / edit / import Obsidian) — deliberately Settings-only; the Setter holds *styles*, not themes.
- **Title alignment** — a layout preference, not a CSS-var look.
- **Living Link Lifecycle** (decay on/off, half-life) — behavioural (sort weighting), not styling.
- **Language**: font theme (default/typewriter), numeral style, date formats, script toolbar, secondary-language enable — locale/behaviour, not Setter styling.
- **Style Settings import / export / copy / reset-all** — bulk ops on the catalog.

## D. SETTER-ONLY — net-new (no Settings equivalent)

- Per-element note chrome the catalog never had: **breadcrumb** (colour+size), **note summary** (colour/font/size/weight/italic), the **Universe panel**.
- The **live-preview-on-the-real-app**, **Keep/Discard**, **inspect-to-style**, **saved swatches**, and the `styleOverride`-carrying named Styles — a UX layer, not duplicated controls.

---

## Recommendation (Phase 9 path)

The duplication is real and large — exactly the redundancy MIG-070 §C set out to remove. The clean retire order:

1. **Close the §B gaps in the Setter** (~5 small additions: cursor/selection, link-hover, code-block-radius, blockquote border, shadows) so the Setter is a **superset** of the Style Settings tab.
2. **Phase 9.2 — retire the Style Settings tab** (the ~85-control catalog UI). Keep `StyleSettingsPanel`'s *apply path* + the catalog itself (still the engine `styleOverride` rides on); remove only the **tab UI**.
3. **Phase 9.3 — trim the Appearance tab's styling duplicates** (pill shape, typed-link display, the quick font-size sliders) now that the Setter owns them. **Keep** the **Styles** cards + **Link Types** editor + **Themes** (the first two are shared engines; themes are Settings-only by design).
4. **Language**: leave it — its font controls are locale-bound; the Setter's per-script fonts are the *style* layer on top.

**Invariant (BUG-015 discipline):** both surfaces write the same stores today, so they stay in lockstep until the retire commit — delete the duplicate UI **last**, only after the §B parity gate, one tab at a time, each revertible.

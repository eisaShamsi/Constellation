---
aliases:
  - Themes
  - Style Settings
  - Custom theme
  - Import Obsidian theme
  - Delete theme
  - Export style settings
description: Personalize every visible part of Constellation — themes, colors, typography, and component styling via Appearance and the native Style Settings tab.
---

# Appearance and Themes

Constellation's appearance is controlled from two places in **Settings**:

1. **Appearance** — pick or create a theme, import themes from Obsidian's community registry, and adjust global font and layout preferences.
2. **Style Settings** — a dedicated tab that exposes every visible piece of Constellation's interface as a live-tweakable control (sliders, color pickers, dropdowns). Changes apply instantly and are saved to the active theme.

Together they let you reshape the app to match your workflow, screen size, and personal taste — without editing a single line of CSS.

## Themes

A **theme** is a named bundle of colors, settings, and CSS that defines how Constellation looks. Constellation ships with six built-in themes (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark), all paired across light and dark system modes.

### Pick a theme

1. Open **Settings → Appearance**.
2. Click any card in the **Themes** grid. The theme applies immediately.
3. The active card is highlighted with an accent border.

### Create a custom theme

1. In the themes grid click the dashed **+ New Theme** card.
2. Give it a name, choose light or dark, and pick five colors (background, surface, text, accent, border).
3. Click **Save**. Your theme now appears in the grid.

All other variables (hover states, shadows, muted text) are derived automatically from your five colors using HSL math, so you only control what matters.

### Edit or delete a custom theme

Hover over any custom theme card:
- **✏️ (pencil)** — opens the editor to change its name, type, or five core colors.
- **✕ (red X)** — deletes the theme after confirmation. Built-in themes cannot be deleted. If you delete the active theme, Constellation reverts to the default.

### Import an Obsidian community theme

Click **🟣 Obsidian Themes** to browse over 200 community themes:
1. Search by name or author.
2. Click **Preview** for a mockup of the layout and five-color swatch.
3. Click **Import** — the theme's CSS is downloaded, adapted for Constellation (selector shim + variable extraction + CodeMirror syntax colors), and added to your custom themes.
4. If the theme supports **Style Settings**, the count is shown on its card; those options appear in the Style Settings tab after import.

## Style Settings

The **Style Settings** tab is Constellation's native, theme-agnostic control panel. It covers every visible piece of chrome plus the editor, and works with any theme (built-in, custom, or imported).

### How it's organized

Sections are collapsed by default. Click a chevron to expand:

- **Constellation — Colors** — background & surfaces, text, accent
- **Constellation — Typography** — interface/note/code font sizes, H1–H6 sizes, heading weight, line heights, paragraph spacing
- **Constellation — Layout & Shape** — corners (small / medium / large radii), border widths, shadows, editor line length, side margins
- **Constellation — Components** — ribbon dock, sidebar action toolbar, layout bar (pane toggles), top bar / tab strip, status bar, file explorer, right sidebar, buttons, tags, callouts
- **Constellation — Editor** — links, code & blocks, blockquote, cursor & selection

### Changing a value

- **Color pickers** — click the swatch, pick a color. The hex shows next to it.
- **Sliders** — drag to adjust. The numeric value shows in the unit (px, %, etc.).
- **Switches** — click to toggle classes on/off (mostly for imported themes).
- **Dropdowns** — pick an option (link decoration style, etc.).
- **Reset arrow (↺)** — appears on hover at the end of each row. Clicking it clears your override and restores the theme's default.

### How saving works

- Changes are saved automatically to the active theme's **styleSettingsValues**.
- If you change a Style Setting while a built-in theme is active, Constellation **auto-clones** the built-in into your custom themes (as `{Name} (custom)`), then saves your changes there. The built-in stays untouched.
- The **Saved to:** label at the bottom of the tab shows which theme currently holds your overrides.
- Click **Reset all to defaults** to wipe every override on the active theme.

### Import / Export Style Settings

A toolbar at the top of the Style Settings tab:

- **📋 Paste from clipboard** — one-click: reads the clipboard and merges valid JSON into the active theme.
- **⬆️ Import / Paste** — opens a textarea; paste JSON by hand. Choose **Merge** (adds/overrides) or **Replace all** (wipes, uses only pasted).
- **📄 From file** — open a `.json` file exported from Obsidian's Style Settings plugin or another Constellation install.
- **📋 Copy** — copies the current values to your clipboard as pretty-printed JSON.
- **⬇️ Export** — saves the values as `{theme-name}-style-settings.json`.

The JSON format matches Obsidian's Style Settings plugin exactly — a flat object mapping setting IDs to string values:

```json
{
  "h1-size": "36",
  "interactive-accent": "#7c3aed",
  "my-themed-color@@light": "#ffffff",
  "my-themed-color@@dark": "#1e1e2e"
}
```

This means you can copy your Style Settings from Obsidian and paste them straight into Constellation, or vice versa.

## What you can control

Every setting lives under one of the five blocks above. Highlights:

### Typography

- **Interface font size** — sidebar, toolbars, menus
- **Note font size** — body text in the editor
- **Code font size** — inline code and fenced code blocks
- **H1 – H6 sizes** — each heading level individually
- **Heading weight** — lightness or boldness of all headings
- **Line heights** — normal (body) and tight (headings and dense UI)
- **Paragraph spacing** — gap between paragraphs

### Shell components

- **Ribbon dock (left icons)** — width, button size, icon size, radius, colors
- **Sidebar action toolbar** — new note / table / folder icons — size, color, height, background
- **Layout bar (pane toggles)** — left / split / right sidebar toggles — button size, icon size, colors, active-state color
- **Top bar / Tab strip** — only visible when notes are open in tabs; controls strip height, background, tab height/font/radius, active and inactive tab colors
- **Status bar** — height, font size, background, text color
- **Right sidebar (inspector)** — background, tab row height, tab icon size, tab icon colors
- **File explorer (left sidebar)** — Universe notes row, child universe (cUniverse) rows, library names, folders, notes — each with independent size, weight, and color; plus vertical row spacing

### Editor

- **Heading sizes** (H1–H6) and weight
- **Line height** in the note body
- **Inline code** background, text color, radius, font size
- **Link color** (default + hover) and decoration style (none / underline / dotted)
- **Callout bar width** and **callout radius**
- **Cursor color** and **selection background**

### Colors (every color in the app)

- Background (primary / alt), surfaces, hover background, borders, input background
- Text (normal / muted / faint / on-accent), error / warning / success states
- Accent (interactive accent + hover), accent text

## Frequently asked questions

### Can I style the Windows title bar ("Constellation v0.3.4 — …")?

No — that bar is drawn by the operating system (Windows / macOS / Linux). Constellation has no CSS access to it. Everything below it is fully stylable.

### Why doesn't the sidebar width slider work?

Sidebar width is controlled by the drag handle on the sidebar's edge (drag to resize). We deliberately do not duplicate that control in Style Settings to avoid conflicting sources of truth.

### Where do my Style Settings live?

Inside `Universe/settings.json` under `customThemes[i].styleSettingsValues`, scoped to each theme. They travel with your Universe — if you sync your Universe directory across devices, your styling comes with it.

### Can I share a theme with someone?

Yes:
- **Full theme** — in the theme editor, click **Export**. Share the `.json` file. The recipient clicks **↓ Import** on the themes grid and selects it.
- **Just Style Settings values** — in the Style Settings tab, click **Export** to export only the slider/color values (not the theme structure). Useful for applying your personal tweaks on top of someone else's theme.

### An imported Obsidian theme looks broken. What now?

Obsidian themes can be complex. Known cases:
- Themes that use **HSL-split colors** (like Minimal) — supported in Constellation from this release onward.
- Themes that depend on Obsidian's specific DOM structure may render partially. Constellation includes a class shim that maps the most common selectors, but very structural themes may require tweaking the five core colors or adjusting Style Settings values by hand to compensate.

## Related

- [[Universe]] — where themes and Style Settings values are stored
- [[Libraries]] — per-library color accents (set in the library settings, independent of themes)
- [[Importer]] — for importing notes, not themes (theme import is under Appearance)

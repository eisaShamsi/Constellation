---
aliases:
  - Themes
  - Style Settings
  - Styles
  - Style Presets
  - Link Types editor
  - Typed link colours
  - Custom theme
  - Import Obsidian theme
  - Delete theme
  - Export style settings
description: Personalize every visible part of Constellation — themes, colors, typography, component styling, named switchable Styles, and typed-link colours — via Appearance, the native Style Settings tab, and the Link Types editor.
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

## Style Settings → now in the Style Setter

> **Note:** The standalone **Style Settings** tab has been **retired**. Every control it had now lives in the **Style Setter** (its own tab in the Settings sidebar) — which covers all of them and more (breadcrumb, note summary, the Universe panel, per-script fonts). The reference below describes that styling surface, now reached through the Style Setter.

The styling surface is Constellation's native, theme-agnostic control panel. It covers every visible piece of chrome plus the editor, and works with any theme (built-in, custom, or imported).

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

## Styles (named, switchable looks)

A **Style** is a complete, named "look" for Constellation that you save, switch to with one click, and reuse across every Universe — like a profile. Where a *theme* is just colors and CSS, a Style can capture your whole configuration: theme, fonts, the eight link-type colors, pill shape, typed-link display, Sky View look, layout, and behaviour preferences. Save the look you like, name it, and jump between looks anytime.

Styles live in the **Style Setter** as your **Saved styles** list — apply one with a click; hover a row to Update, Export, Rename, or Delete it.

### Save your current look as a Style

1. Open the **Style Setter** (its tab in the Settings sidebar). Your **Saved styles** are at the bottom-left.
2. Click **+ Save current style…**.
3. Type a name (for example, `Midnight`).
4. Tick which **sections** to include — Theme, Fonts, Link colors, Pill shape, Typed-link display, Sky View, Layout, Behaviour. (Leave them all on to capture everything.)
5. Click **Save**. A card appears with a live preview of the look you just saved.

### Apply, rename, duplicate, delete

Each Style card has:
- **Apply** — switches the app to that look immediately.
- **✎ Rename**, **⧉ Duplicate**, **✕ Delete** — manage your Styles.
- **⤓ Export** — see below.

### Share a Style (export / import)

- **Export:** click **⤓** on a card and choose where to save. You get a `{name}.constellation-style.json` file you can send to anyone.
- **Import:** click **Import…** at the bottom of the Styles section and pick a `.constellation-style.json` file. It is added as a new Style, ready to apply.

Styles are **app-global** — stored once for your whole Constellation install (not inside any one Universe), so every Universe you open can use them. Applying a Style **merges** its link-type colors into the current Universe: your Universe keeps any custom link types it already has — a Style never deletes them.

> **Privacy:** a Style never carries secrets, tokens, or folder paths — only visual preferences. It is safe to share.

### Styles vs. Themes vs. Style Settings

- A **Theme** is colors + CSS — one ingredient of the look.
- **Style Settings** are fine-grained per-theme tweaks (sliders, color pickers).
- A **Style** is the whole bundle — theme *and* fonts *and* link colors *and* shape *and* behaviour — saved under one name, switchable in a click, and reusable across Universes.

## Typed-link colours and labels

Constellation's links can carry a **type** — *supports*, *contradicts*, *causes*, *exemplifies*, *generalizes*, *derives-from*, *part-of*, *supersedes* — and the app colours each type, and can label it, so you can read a note's reasoning at a glance.

### The colours

Each link type has its own colour, shown wherever the type appears: the typed link in the editor, and the coloured **pills** in the Backlinks and Outgoing Links panels (and the Knowledge-Health view). The colour comes from **one source** — the Link Types editor — so changing a colour updates every surface at once.

### The two display switches

In **Settings**, two toggles control how typed links look (both on by default):
- **Colour typed links** — draw each typed link in its type's colour (off = the plain wikilink colour).
- **Show the label above** — show the type name (for example, `supports`) just above the link in the editor.

### The label's language follows the note

The type label and the panel pills appear in the **note's own main language**, not the interface language. An Arabic note shows `يدعم`; an English note shows `supports` — even if you have switched Constellation's menus to a different language. The look stays true to what you are reading.

### Recolour a type (the Link Types editor)

1. Open the **Style Setter → Links** category.
2. Each type shows its name and a colour swatch. Click a swatch to pick a new colour.
3. The change reflects **live** everywhere — the editor links and the Backlinks / Outgoing pills update as you pick.
4. **Reset colours to default** restores the original eight colours.

## The Style Setter

The **Style Setter** is a full-page design studio for your whole interface. Instead of adjusting settings one at a time and imagining the result, you change a control and watch your **real app** restyle as you do.

**Open it:** go to **Settings → Appearance** and click **"✦ Open Style Setter."** You can **resize the panel** — drag the small grip at its bottom-right corner; it remembers the size next time.

**Pick what to style — the left list.** Down the left are the *Surfaces* you can style:

- **Interface** — the file tree, status bar, and universe bar.
- **Components** — the ribbon dock, toolbars, top bar & tabs, buttons, tags & callouts.
- **Editor** — the note itself: the **breadcrumb** path line, headings, bold, italic, links, inline code, blockquotes, and the **note summary** (the italic line under the title — its own colour, font, size, thickness, and italic on/off).
- **Global** — background and text shades, accent shades, type & spacing, corners & borders, and per-script fonts.
- **Links** — the typed-link colours and how they display.
- **Sky View / OrgChart / Index / Cataloger / Shell** — the plugin surfaces. The **Sky View** surface includes a **Canvas** element whose **Background** colour sets the backdrop behind the graph bubbles — independent of the panel colour (see *Sky View canvas*, below).

Below them are your **saved styles** — click one to apply that whole look at once (see *Save a look as a named Style*, below). *(Built-in themes are picked from Settings → Appearance, not here.)*

**Two ways you see your changes:**

- **The Editor category** shows a **note preview in the centre.** Click a heading, bold, a link, or the page and its controls appear on the right; the preview updates instantly.
- **Every other category** docks the panel to one side and goes see-through, and your edits show on the **real app, live.** Change the status-bar colour or the dock width and the actual sidebar, dock, tabs, and status bar restyle **as you drag.** A green **● live** tag in the top bar reminds you that you are editing the real thing.

**The Links category** keeps the typed-link colours and shape in one place. Each of the eight types (supports, contradicts, …) is shown as its real coloured **pill** — **click a pill to recolour it,** and the change reflects live everywhere (the editor links and the Backlinks / Outgoing pills). Above the list are switches — **Colour typed links** and **Show type labels** — and the **pill shape** (corner radius, height, label weight). A **Saved colours** palette remembers every colour you pick so you can reuse it on any element: click a saved colour to apply it to the highlighted type. (To name or remove saved colours, use **Manage** — see *Saved colours* below.)

**Fonts — your installed typefaces.** Every font picker in the Setter — **Interface**, **Note**, **Code**, and the file-tree / chrome fonts — lists the fonts **actually installed on your computer**, alphabetically, with **System / Serif / Mono** kept on top. **Each name is shown in its own typeface,** so you can preview a font before choosing it. (If your system blocks font detection, the Setter falls back to a curated cross-platform list — you still get a real, usable set.)

**Saved colours — a named, reusable palette.** Whenever you pick a colour, the Setter remembers it under **Saved colours** (shown for any element that has a colour control). Click a saved colour to apply it to the control you last touched — and the palette is shared everywhere, including the typed-link colours in the **Links** category. Click **Manage** next to the heading to **name** a colour (e.g. *Brand teal*), rename it, or remove it. **Removing is deliberate** — there is no accidental right-click: in **Manage**, click the **✕** on a colour, then confirm **Remove** (or **Cancel**). Your names are saved per-Universe and survive a restart.

**Sky View canvas — its own background colour.** The Sky View graph draws its bubbles on a transparent canvas, so the colour you see behind them is just a backdrop. Open **Style Setter → Sky View → Canvas** and set **Background** to give the graph its own backdrop — a deep colour to make the bubbles pop, or any shade you like. It is **independent of the panel/sidebar colour**, so recolouring the graph never moves the rest of your interface. Left unset, the canvas follows the panel surface (the default look). The same colour applies to the small Sky View on the second screen, so both match. The preview card in the Setter shows your chosen colour live as you pick it.

**Inspect — click the app to find its controls.** Click **⌖ Inspect** in the top bar, then hover your **real app**: the part under your cursor is highlighted and named, and clicking it jumps the Setter straight to that part's controls (then exits inspect). It reaches the dock, toolbars, tabs, sidebars, status bar, file tree and folders, note text, tags, the Universe panel, the sidebar's **library** and **child-universe** rows, and generic **buttons**. Press **Esc** to leave inspect.

**Keep, Discard, Reset.** When you like what you see, click **Keep** (top right) to save the look **for this Universe** — it survives a restart. **Discard** (or simply closing with **✕** or **Esc**) throws away your unsaved edits and the real app snaps back to the saved look. **Reset** clears everything back to the plain theme. Nothing is written to disk until you Keep.

**Save a look as a named Style.** To reuse a look, save it under a name: type a name in the top **"draft:"** field and click **"+ Save current as a style"** (bottom-left). It joins your **Saved styles** list — app-global (reusable across every Universe), and it captures the look you designed in the Setter, not just a theme. **Click a saved style to apply it.** Hover a saved-style row for its actions: **↻ Update** (overwrite that style with your *current* look — keeps its name), **⤓ Export** (share it as a `.constellation-style.json`), **✎ Rename**, and **✕ Delete**.

> Built-in themes (Midnight, Daylight, …) live in **Settings → Appearance**, not the Setter — the Setter holds your **saved styles** and the live per-Universe look. The full Styles manager (with duplicate / import) is also in **Settings → Appearance → Styles**.

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

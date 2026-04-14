# Session Log — 2026-04-14
# Native Style Settings — Full Constellation Shell Customization

## Summary
Transformed Style Settings from a passthrough for imported Obsidian theme options into a **native, theme-agnostic control panel covering every visible piece of Constellation's chrome**. Every component — dock, sidebars, layout bar, tab strip, status bar, file explorer, inspector — is now wired to CSS variables exposed as sliders and color pickers. Import/export supports Obsidian's Style Settings JSON format.

## What Shipped

### 1. Style Settings as a standalone tab
- New sidebar entry between **Appearance** and **Hotkeys** (`sliders` icon)
- Replaces the previous "nested under Appearance" design
- Shows the active theme's controls; auto-clones built-in themes into custom themes on first change so values persist
- `SettingsModal.svelte:127, 1770+` — section + block

### 2. Delete theme
- Red `✕` button appears on hover next to the edit pencil on custom theme cards
- Built-in themes protected
- Confirms via native `confirm()` dialog; falls back to default if active theme is deleted
- `SettingsModal.svelte:1627, 2240+`

### 3. Native Constellation core blocks
New file `src/lib/theme/constellationStyleSettings.ts` with 5 blocks × dozens of settings:
- **Colors** — background, surface, text, accent, borders, state colors
- **Typography** — interface/note/code sizes, H1–H6 sizes, heading weight, line heights, paragraph spacing
- **Layout & Shape** — small/medium/large radii, border width, shadows, readable line length, side margins
- **Components** — ribbon dock, sidebar action toolbar, layout bar (pane toggles), top bar / tab strip, status bar, file explorer, right sidebar, buttons, tags, callouts
- **Editor** — link color/hover/decoration, code colors, blockquote, cursor & selection

Each id maps to a CSS variable (no `--` prefix) consumed by shell CSS with `var(--id, fallback)`.

### 4. Shell CSS wired to CSS variables
Every visible component stylesheet routed through the new variables:

**Editor (`+layout.svelte`, `livePreview.ts`, `calloutPlugin.ts`, `NotePane.svelte`)**
- H1–H6 sizes + weight + tight line height
- Body font size, normal line-height, caret color, selection background
- Inline code background/text/radius/font size
- Link color, hover color, decoration
- Callout bar width + callout radius

**Shell (`+layout.svelte`)**
- Dock: width, background, button size, icon size, button radius, icon color
- Sidebar action toolbar: height, background, button size, icon size, radius, color
- Layout bar: background, height, button size, icon size, radius, icon color (default + active)
- Top bar / tab strip: min height, background, tab height, tab radius, tab font size, tab bg + text (active/inactive), tab border
- Status bar: height, font size, background, text color (plus grid row updates so height actually expands)
- Right sidebar: background, tab row background/height, tab icon size, tab icon color (default + active)

**File explorer (`FileTree.svelte`, `+layout.svelte`)**
- Universe notes row: size, weight, color
- Child universe (cUniverse) row: size, weight, color
- Library name: size, weight, color
- Folders & notes: base size, folder weight/color, note weight/color
- Row vertical spacing

### 5. Import / Export
Toolbar at top of Style Settings tab:
- **Paste from clipboard** — one-click apply, auto-reads clipboard, merges; falls back to paste box on parse error or permission denial
- **Import / Paste** — textarea modal; Merge or Replace all
- **From file** — `.json` file picker
- **Copy** — copies current values as JSON
- **Export** — downloads `.json` named after active theme

Format is Obsidian-compatible: flat object of `setting-id → value`, including `id@@light` / `id@@dark` suffixes for themed colors.

### 6. Collapse behaviour
- All heading sub-sections collapsed by default on first open
- `computeVisible()` helper in `StyleSettingsPanel.svelte` honors level-based nesting (H2 closes under H3, etc.)
- Chevron rotates: ▼ expanded, ▶ collapsed

### 7. Localization
Added 19 Style Settings keys across all 15 locale files (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh). Native scripts used. Product names (JSON, Obsidian, Constellation) preserved.

### 8. Bugs fixed this session
- **Runaway panel growth** — earlier `onChange` persisted `[...CORE, ...existing]` back to `theme.styleSettingsBlocks`, doubling on every change. Now values-only; blocks merged at apply-time. One-time cleanup effect scrubs `styleSettingsBlocks` of any core-block copies from previously corrupted saves.
- **No-op collapse** — chevrons rotated but content never hid. `computeVisible()` now filters items following a collapsed heading.
- **`{@const}` placement** — moved logic into a helper function after Svelte's placement rule rejected it inside `<div>`.
- **i18n raw keys** — 19 new keys added to each locale; Arabic sidebar now shows localized labels.
- **Status bar height clipping** — grid-template-rows had hard-coded `24px`; now `var(--statusbar-height, 24px)` so height expansion actually works.
- **Typo fix** — `SettingsModal.svelte:1705` stray `t` before `<!-- Style Settings -->` comment.

## Files touched
- `src/lib/theme/constellationStyleSettings.ts` — NEW (core blocks definition)
- `src/lib/components/StyleSettingsPanel.svelte` — collapse helper, default-collapsed init
- `src/lib/components/SettingsModal.svelte` — Style Settings tab, toolbar, delete theme, import/export, clipboard paste
- `src/lib/components/FileTree.svelte` — folder & note rows wired to CSS vars
- `src/lib/components/NotePane.svelte` — editor body font size, line height, caret, selection
- `src/lib/editor/livePreview.ts` — heading sizes, code, link styling
- `src/lib/editor/calloutPlugin.ts` — callout bar width + radius
- `src/routes/+layout.svelte` — dock, sidebar toolbar, layout bar, tab bar, tabs, status bar, right sidebar, library/universe/cuniverse rows + one-time cleanup effect + merged core blocks in theme application
- `src/lib/i18n/*.json` — 19 new keys in all 15 locales

## Open items
- **Shell sweep — not yet wired**: `--radius-s/m/l`, `--border-width`, `--button-radius/padding`, `--tag-radius/bg/color`, `--file-line-width`, `--file-margins`, `--paragraph-spacing`, `--font-interface-size` consumers.
- **Obsidian importer HSL-split bug**: ✅ FIXED in `ea37036` — `resolveColor()` now assembles HSL-split colors and follows `var()` chains.
- **Help docs**: ✅ DONE in `3a9927a` — `Appearance and Themes` page added across all 15 locales + User Manual updated.
- **`/simplify` review**: ✅ DONE in `3bc2d3a` — three review agents found 9 issues; consolidated fixes shipped.

---

## Phase 2 — Trial Universe build (commits `f2e46ba`, `ea37036`, `3bc2d3a`, `3a9927a`, `c8b79d0`, `521d9d2`, `bf58hcway run aborted`, `… current run`)

### Scaffolding shipped
- `lab/trial-universe/` — Node generator that fetches from Wikipedia (multi-language) + Wikimedia Commons.
- Topology: 1 Universe → 4 cUniverses (Science, Humanities, Arts & Culture, العالم العربي) → 16 libraries → ~80 folders.
- Curated contradiction pairs seeded per library so Tension Detector finds real opposing positions.

### Generator features
- HTML→Markdown with infobox/navbox stripping, GFM tables.
- Hero images downloaded from Commons with CC license preservation.
- Frontmatter: title, aliases, tags, maturity (canonical/evergreen/sapling/seed), stage (spark/birth/growth/maturity), source, source_url, license, attribution, library, cUniverse, folder, plus 30+ infobox-derived properties (born/died/era/school/field/notable_works/influenced_by, etc.).
- Callouts: `> [!abstract]` (TL;DR), `> [!example]` (notable works), `> [!info]` (key facts), `> [!warning]` (disputed perspective for contradiction pairs). Bilingual labels (EN + AR).
- Typed link assignment: 7 cognitive types (supports/contradicts/causes/exemplifies/generalizes/derives-from/part-of) chosen via curated contradictions, infobox hints, in-body causal/derivation phrases, and folder structure.
- Seed expansion: after explicit seeds, walks 1-hop outbound links of built notes (filtered against other libraries' seeds) to fill quota.
- Cross-cUniverse round-robin so the build reaches every cUniverse.

### Pilots run
- PoC (20 notes, 1 library): typed link distribution 60% supports / 26% part-of / 10% exemplifies.
- Pilot v3 (100 notes, 4 cUniverses): improved to 51% supports / 21% exemplifies / 14% part-of after infobox hints + reverse contradicts.
- Pilot v4 + Arab cUniverse (100 notes, 4 cUniverses incl. العالم العربي): Arabic notes correct, callouts firing, properties extracted.

### Critical bug fixed mid-pilot
- Cache path used `\w` regex (ASCII-only) → all Arabic titles collapsed to the same cache file → catastrophic content cross-contamination. Fixed with `\p{L}\p{N}` Unicode property escapes.

### Full build in progress
Commit `521d9d2`. Target: 6,000+ notes (Arab cUniverse ≥1000). librariesMax=16, notesPerLibrary=550. Per-library targets bumped:
- Science: Physics 550, Biology 550, CS 500, Earth 450
- Humanities: Philosophy 550, History 550, Linguistics 450, Religion 500
- Arts: Literature 550, Music 500, Architecture 450, Film 400
- Arab: History 450, Geography 400, Sciences 400, Literature 350

ETA 6–8 hours. Background task `bf58hcway` running.

---

## Phase 3 — Living Link Architecture P2–P5

### P2 Traversal Tracking — ✅ COMPLETE (`2c285fb`)
- Discovered: `constellation_link_traverse` Rust command and `openNoteTab(_fromNotePath)` plumbing already existed; wikilink, Backlinks, Outgoing Links surfaces already record traversal.
- Added: Sky View node click + Trail prev/next now pass source path so the traversal counts.
- DB updates: `traversal_count++`, `last_traversed = now`, weight recomputed `1 + ln(1+count)`, dormant→active.

### P3 Weight + Lifecycle — 🔲 NEXT
Needs: 6 lifecycle stages (spark/birth/growth/maturity/dormancy/renewal/archival) + decay job (5%/month) + visual indicators (weight bars, stage chips).

### P4 Formulation Queries — 🔲 PENDING
Needs: Rust commands for strongest chains, contested clusters, untraversed canonical, dormant canonical; new "Formulation" tab in SearchHub.

### P5 Knowledge Health Dashboard — 🔲 PENDING
Needs: aggregate stats (link health, orphan pressure, stagnant convictions, emerging tensions), single-page view, dock entry.

---

## Phase 4 — Style Settings refinements (commits `a0ded32`, `9e0758a`, `d57b549`, `920107f`, `ee0514d`)

### Master File Explorer (single source of truth)
After multiple attempts at master-with-per-tier-override cascade (which kept breaking due to stored legacy values + custom-vs-builtin id collision), per user direction collapsed to **single master per property**:
- Font size, Weight, Text color, Row spacing — each applied uniformly to every row in the file explorer.
- Universe notes, child universe entries, library headers, folders, notes all read `--ft-master-*` directly.
- Per-tier overrides removed.

### Critical bug found and fixed in this round
`+layout.svelte` theme-apply effect was looking up the active theme as `BUILTIN_THEMES.find(...) || customThemes.find(...)`. When Style Settings auto-clones a built-in into customThemes (keeping the same id) so user values can persist, the find returned the **built-in first** — silently ignoring the user's `styleSettingsValues`. Reversed: customs now take precedence over built-ins on id collision.

### Stale CSS variable cleanup
`root.setProperty` never removes properties. When a user reset a Style Settings row, the previously-set CSS var would persist forever on `document.body`. Added a `_lastStyleSettingsKeys` registry that diffs keys between applies and removes any no-longer-present property.

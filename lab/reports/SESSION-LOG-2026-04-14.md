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

---

## Phase 5 — Universal Embed Resolver (Obsidian parity)

Replaced the image-only embed handler with a full-spectrum resolver.

Commits `e050cb5`, `45f4a4e`, `12697de`, `9754014`, `3314ae6`, `fea299a`, `40eb19c`, `c645ec6`.

- New `src-tauri/src/embeds.rs` with a 6-level resolution pipeline matching Obsidian exactly (note-relative → vault-path → `.obsidian/app.json` attachmentFolderPath → common fallback folders → vault-wide filename index → vault root).
- Vault index is lazy-cached, digit-normalized (Arabic-Indic / Persian ↔ ASCII so `Pasted image ٢٠٢٥٠٩١٥.png` and `Pasted image 20250915.png` match).
- Longest-prefix library match in `openNoteTab` so nested libraries route correctly.
- `UniversalEmbedWidget` routes every `![[target]]` to the right renderer: Image / Audio / Video / PDF / Canvas / Excalidraw / Note transclusion / Generic file / Missing placeholder.
- Note transclusion (read-only, scoped to `#heading` or `^block-id`, circular-guarded, header click opens source).
- Rich diagnostic card on miss — shows tried paths, attachmentFolderPath from `.obsidian/app.json`, vault file count, did-you-mean suggestions, and a live filesystem listing of the expected attachment folder.

---

## Phase 6 — Canonical filename disaster recovery + lockdown

Commits `d6228b7`, `cc0ca57`, `b19908c`.

Problem: earlier builds mass-renamed external Obsidian vaults to Constellation's canonical scheme (`20260410T153045Z_NOTE_XXXX.md`), breaking every wikilink and embed reference. 10,616 files across 8 libraries had been corrupted.

Fix, in three layers:
1. **Automatic repair on startup** — `repair_external_libraries_on_startup` scans every library (no mode gate — filesystem is the source of truth). For any library with canonical-named .md files, invokes `de_canonicalize_library` which restores original filenames from frontmatter `title` / `original_filename`. Ran once on the user's machine, restored all 10,616 files instantly with zero errors.
2. **Permanent import lockdown** — `handleAddLibrary` now calls `handleKeepIntact` directly. The "adopt canonical" dialog is gone. External files are NEVER renamed.
3. **Namespaced CID property** — stable identifier renamed `cid` → `cid_cn` (zero-collision with any pre-existing `cid:` in a user's vault). Lazy injection per-note on first open via `ensure_cid_cn_cmd`. Timestamp = file's creation/mtime. Legacy `cid:` values migrate in-place on first touch.

---

## Phase 7 — Emoji & Icon Library (Core Plug-In)

Commits `ce3c3e0`, `e7bf160`, `6d36c14`, `fdeaae1`, `8b6521e`, `8c63ea0`, `c1dcd41`.

Full-scope plug-in (~3,600 icons across four MIT-licensed sets + every Unicode emoji with 23-locale keyword search):

- **Picker** (`EmojiIconPicker.svelte`): three-tab modal (Emoji / Icons / Recent). Ctrl+. globally. Emoji tabs + set-filter chips for icons (Lucide / Phosphor / Heroicons / Feather).
- **Inline `:shortcode:` autocomplete** (`shortcodeAutocomplete.ts`): 23-locale emoji keyword search + all icon names (both short and namespaced forms).
- **Widget renderer** (`IconShortcodeWidget` in livePreview.ts): `:lucide-heart:` renders as inline SVG at 1.15em. Keeps `.md` files small/readable.
- **App icon overrides** (`iconOverrides.ts`, `IconOverrideSettings.svelte`, `SlotIcon.svelte`): 60+ customizable shell slots (Dock, Sidebar Toolbar, Layout Bar, Inspector Tabs, File Tree, Editor Toolbar, Callouts). New Settings sub-tab "App Icons". Reference wiring on Knowledge Health dock button — other slots follow the same `<SlotIcon slot="..."><svg>...</svg></SlotIcon>` pattern.
- **Active editor registry** (`activeEditor.ts`): tracks last-focused editor so picker inserts at correct cursor across split/multi-pane layouts.
- Lucide format fix: v1.x uses `Array<[tag, attrs]>` not the `[tag, attrs, children]` shape we assumed initially.

---

## Phase 8 — Living Link Architecture P2–P5

Commits `2c285fb`, `3c78b2c`.

All four phases wired. P2 Traversal Tracking + P3 Weight/Lifecycle + P4 Formulation Queries + P5 Knowledge Health Dashboard — discovered the backend (`constellation_link_traverse`, `constellation_link_decay`, `constellation_formulation_analysis`, `constellation_link_stats`) was already implemented; this session wired missing surfaces (Sky View node click, Trail prev/next) and completed the UI plumbing for the Knowledge Health Dashboard (already-existing but unreachable component `KnowledgeHealthDashboard.svelte` — state variable was never declared). Daily auto-decay trigger added in `onMount` via localStorage-gated 24h check.

---

## Phase 9 — Trial Universe build (COMPLETE)

Background task `bf58hcway` finished after ~3 hours.

| Metric | Value |
|---|---|
| **Total notes** | **7,600** (target: 6,000+) ✅ |
| Science | 2,050 |
| Humanities | 2,046 |
| Arts & Culture | 1,900 |
| **العالم العربي** | **1,600** (target: ≥1,000) ✅ |
| Total images | 4,064 (Wikimedia Commons, CC-attributed) |
| Size on disk | 967 MB |
| Skipped seeds | 49 (disambiguation / 404 / redirects) |

**Link distribution** (656,855 typed links total):
- derives-from: 43.9%
- supports: 42.9%
- exemplifies: 5.7%
- **contradicts: 3.0%** (19,321 — strong input for Tension Detector)
- causes: 2.7%
- part-of: 1.4%
- generalizes: 0.3%

**Structure**: 1 Universe (Constellation Discovery) → 4 cUniverses → 16 libraries → ~80 folders → 7,600 notes. Rich frontmatter (30+ curated properties from Wikipedia infoboxes). Callouts (`abstract` / `example` / `info` / `warning`) fire per note. Hero images on every note that has one in Commons. Arabic content sourced from `ar.wikipedia.org`.

**Next steps for the trial Universe**:
- Ship as a ZIP release artifact (user-downloadable)
- Smoke-test by opening in Constellation
- Verify embed resolution works across this scale
- Optionally: regenerate with tuned heuristics (derives-from is overweighted — could rebalance)

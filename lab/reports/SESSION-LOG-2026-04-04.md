# Session Log — 2026-04-04

## Phase: Post-NotePane Rebuild Audit & Fix

### Context
After NotePane was rebuilt from scratch with a new individual-props interface, a full app audit revealed 4 out of 5 NotePane instances were silently broken (using the legacy `tab={}` interface). Additional issues found in workspace restore, settings propagation, and dead code.

### Commit: `3d35146`
**Tag:** `milestone/post-notepane-audit`

### Changes Made

#### CRITICAL: 4 Broken NotePane Instances Fixed
All were passing legacy `tab={tabObj}` + invalid props (`isFocused`, `color`, `splitView`, `libraryTrees`). Fixed with full individual props + callbacks:

1. **Split View** (`+layout.svelte`) — Full editing with save/flush/properties/stage in all panes
2. **Index Preview** (`+layout.svelte`) — Full editor when clicking notes from Index
3. **Second Screen Detail** (`SecondScreenPage.svelte`) — Full editor with save support
4. **Second Screen Peek** (`SecondScreenPage.svelte`) — Full editor in sky view peek

#### Split View Resizable Dividers
- Added per-divider drag-to-resize for split view panes
- Each divider tracks its adjacent pair of panes independently
- Works with 2+ panes, vertical and horizontal, RTL-aware
- 20%-80% clamped range, 100px minimum per pane
- Reduced desk padding in split view (24px → 8px) for tighter layout

#### Workspace Restore Bug
- Added missing tabs to `validTabs`: 'health', 'provenance', 'review'
- Workspaces saved with CE sidebar tabs now restore correctly

#### Settings Propagation to Second Screen
- Added reactive `$effect` in SettingsModal that watches visual settings
- `notifySettingsChanged()` now fires for: colorScheme, fontSize, interfaceFontSize, fontTheme, primaryScript, accentColor, readableLineLength, showLineNumbers, showFloatingToolbar

#### Cleanup
- Removed unused `LinkDashboard` import from +layout.svelte
- Disabled Inspector360 (removed from sidebar, command palette, main window — code stays for future revisit)
- Expanded Inspector360 i18n keys across all 15 locales

### Test Results
- Split view: Both panes have full toolbar, breadcrumb, properties, stage dropdown ✓
- Split resize: Draggable dividers work for 2+ panes ✓
- Index: Notes opened from Index are fully editable ✓
- Build: `npx vite build` clean ✓

### Files Modified
- `src/routes/+layout.svelte` — Split view NotePane, Index NotePane, workspace restore, dead import, split resize CSS
- `src/lib/components/SecondScreenPage.svelte` — Detail + Peek NotePane, added store imports
- `src/lib/components/SettingsModal.svelte` — Settings propagation effect
- `src/lib/components/Inspector360.svelte` — Rewritten with 3 viz modes (disabled)
- `src/lib/i18n/*.json` (15 files) — Inspector360 expanded keys
- `src/lib/graph/graphEngine.ts` — Minor updates

---

## Phase: Dashboard Home Screen + Split View Panels Companion

### Commit: `a122598`
**Tag:** `milestone/dashboard-split-companion`

### Changes Made

#### Dashboard as Optional Home Screen
- Extracted `DashboardView.svelte` component from SecondScreenPage (~360 lines → shared component)
- Added `showDashboard` setting to `appSettings` (default: off)
- Main window shows Dashboard when no tabs open + toggle enabled
- "Show Dashboard" button on home screen, × button to hide
- SecondScreenPage now uses the shared DashboardView component

#### Split View Panels Companion (Second Screen)
- Added `emitSplitModeChanged` / `onSplitModeChanged` events to secondScreen.ts
- Main window emits split state + focused tab data to second screen via $effect
- Second screen switches to "Panels Companion" mode when split view is active
- Shows Properties, Backlinks, Tags, Star, Tasks tabs for the focused note
- Auto-reverts to previous mode when split view is turned off

#### Documentation
- User Manual: Added Split View (§6) and Index (§7) sections, renumbered §8-§20
- Second Screen help file: Added Note Editing + expanded Settings Sync
- All 14 translations updated

### Files Created
- `src/lib/components/DashboardView.svelte` — extracted dashboard component

### Files Modified
- `src/routes/+layout.svelte` — Dashboard home, split state emission
- `src/lib/components/SecondScreenPage.svelte` — DashboardView usage, split companion mode
- `src/lib/libraries/store.ts` — showDashboard setting
- `src/lib/secondScreen.ts` — SplitCompanionData type + events
- `src/lib/i18n/*.json` (15 files) — dashboard + splitCompanion keys
- `docs/User Manual.md` + 14 translations
- `docs/help.uConstellation.World/Second Screen/Second Screen.md`

---

## Phase: NoteEditor Extraction + Shared Utilities + SS Architecture Fix

### Commits: `40950e8` → `48405ca`
**Tag:** `milestone/dashboard-ss-interaction`

### Architectural Principles Established

1. **"Secure the winning"** — If a feature works, extract into a shared component. Never copy-paste and adapt. (CLAUDE.md rule + memory)
2. **"Screens are displays, not domains"** — Second screen mounts core components, never re-implements save/load/edit. (CLAUDE.md rule + memory)

### NoteEditor Wrapper
- Created `NoteEditor.svelte` — accepts a tab-like object, handles all save/flush/rename/stage internally
- Replaced all 7 NotePane call sites (main editor, split view, index preview, SS detail, SS peek, dashboard note, dashboard tag note)
- Net: **291 added, 490 removed** — eliminated ~200 lines of duplicated callback code

### Shared Utilities Extracted
- `colors.ts` — `buildLibraryColorMap()` replaces duplicate color arrays
- `recentNotes.ts` — `getRecentLists()`, `addRecentOpened()`, `addRecentEdited()`
- `tagUtils.ts` — `scanAllLibraryTags()` replaces duplicate merge loops

### Second Screen Architecture Fix
- Removed `onNoteSaved` content re-read (NoteEditor handles its own state)
- `loadAllData()` only shows spinner on first startup (`initialLoadDone` guard)
- Main window now listens for `screen:note-saved` to sync SS edits back
- Dashboard tag click: sends to SS when open, no local split panel duplication

### Callout RTL Fix
- Detect RTL script from title/body text content explicitly
- Set `dir="rtl"` or `dir="ltr"` on callout line decorations — no more `dir="auto"` fooled by emoji icons

### Breadcrumb Alignment
- Matched `.e-breadcrumb` padding to `.e-paper` padding (16px → 48px)

### Dashboard → SS Interaction (Tested & Confirmed)
- Click recently edited/opened note → opens as full editor on SS ✓
- Click tag → SS shows split view (note list + editor) ✓
- Tag click with SS open → no duplicate panel on Dashboard ✓
- Edits on SS persist and sync back to main window ✓
- No "Loading..." flash during editing ✓
- No editor slowness ✓

### Files Created
- `src/lib/components/NoteEditor.svelte`
- `src/lib/libraries/colors.ts`
- `src/lib/libraries/recentNotes.ts`
- `src/lib/libraries/tagUtils.ts`

### Files Modified
- `src/routes/+layout.svelte` — NoteEditor usage, onNoteSaved listener, breadcrumb padding
- `src/lib/components/SecondScreenPage.svelte` — NoteEditor usage, architecture fix
- `src/lib/components/DashboardView.svelte` — shared utilities, tag-to-SS logic
- `src/lib/components/NotePane.svelte` — breadcrumb padding
- `src/lib/editor/calloutPlugin.ts` — RTL direction detection
- `CLAUDE.md` — two new principle rules

---

## Phase: Index — Full NLP Pipeline + SS Interaction + UX Polish

### Commits: `d754659` → `7756f01`
**Tag:** `milestone/index-nlp-complete`

### Full NLP Pipeline (Rust Backend)
- **Arabic**: Lucene Light10 algorithm (normalize → prefix → suffix)
  - Normalization: tashkeel removal, hamza unification, ة→ه, ى→ي
  - Prefix: وال/بال/كال/فال (3-char), ال/لل (2-char), و only (1-char)
  - Suffix: ها/ان/ات/ون/ين/يه/يت/ته (2-char), ه/ي (1-char)
  - "بن"/"ابن" as stopwords, skip words >15 chars
- **English**: Porter-like (plurals, -ing, -ed, -tion, -ness, -ment, -ly)
- **French/Spanish/Portuguese**: suffix removal (char-safe UTF-8)
- **German**: umlaut normalization + suffix removal
- **Russian**: case/gender/number suffixes
- **Turkish**: agglutinative suffix removal
- **Hindi/Persian**: suffix removal with normalization
- **Hebrew**: prefix removal (ב/ל/מ/ה/ו/כ/ש)
- **Japanese/Korean/Chinese**: stopword filtering
- **All 15 languages**: comprehensive stop word lists
- **UTF-8 safety**: all stemmers use char-based operations (not byte slicing)

### Index → Second Screen Interaction
- **Term click** → SS shows note list + editor (same pattern as Dashboard tags)
- **Ctrl+Click multi-term** → SS shows compare mode (columns per term)
- Events: emitIndexTermSelected, emitIndexCompare

### Index UX Polish
- **Letter filtering**: click letter → shows only that letter's terms + count
- **Term count updates** with language and letter filters
- **One term expanded at a time** (clicking new term collapses previous)
- **Comma-separated search**: substring match (finds stemmed forms)
- **Ctrl+Click opens as tab**: closes Index, shows "Return to Index" button
- **Return to Index**: preserves exact state (scroll, filter, letter, expanded term) via display:none instead of unmount
- **Term highlight**: wholeWord search for non-Arabic, regex with word boundaries for Arabic
- **Callout RTL**: explicit dir detection from text content

### Files Modified
- `src-tauri/src/libraries.rs` — Full NLP pipeline rewrite
- `src/lib/components/IndexPanel.svelte` — Letter filtering, term expand, chevron/name split
- `src/lib/components/NotePane.svelte` — Term highlight with Arabic normalization reversal
- `src/lib/components/SecondScreenPage.svelte` — Index term + compare modes
- `src/lib/secondScreen.ts` — IndexTermData, IndexCompareData events
- `src/routes/+layout.svelte` — Index overlay (display:none), Return button, Ctrl+Click

---

## Phase: Index Polish — SS Integration, Anchor Bar, Arabic Display

### Commits: `695042e` → `99a8eb6`
**Tag:** `milestone/index-arabic-display`

### Changes
- **Selected terms anchor bar**: persistent chip bar at top of Index showing all Ctrl+Click selected terms — click × to deselect, "Clear all" to reset, always visible while scrolling
- **Arabic display fix**: split normalization into two levels — display preserves original chars (ة أ إ آ ى), index key unifies for grouping. "تربة" now displays correctly (not "تربه")
- **Term highlight**: Arabic regex with word boundaries, scroll to first match centered

### Files Modified
- `src-tauri/src/libraries.rs` — Two-level Arabic normalization (display vs key)
- `src/lib/components/IndexPanel.svelte` — Anchor bar UI + CSS

---

## Phase: Wikilink Navigation

### Commits: `fbe1e2c` → `1e98682`
**Tag:** `milestone/wikilink-navigation`

### Changes
- **Pointer cursor**: wikilinks show pointing finger on hover (`.cm-md-link { cursor: pointer }`)
- **Single click**: opens linked note in same tab
- **Ctrl+Click**: opens in new tab
- **Non-existent note**: creates it with default frontmatter (`created:` date + user defaults)
- **Root cause fix**: uses `mousedown` instead of `click` — CM6 strips livePreview decorations on click (moves cursor → triggers decoration rebuild), so the handler must fire BEFORE CM6 processes the event
- **Secured in NoteEditor**: `onlinkclick` callback resolves wikilinks via `resolveWikilinkCrossLibrary`, works in every editor instance automatically

### Files Modified
- `src/lib/components/NotePane.svelte` — mousedown handler, `onlinkclick` prop
- `src/lib/components/NoteEditor.svelte` — `handleLinkClick` with resolve + create
- `src/lib/editor/livePreview.ts` — `cursor: pointer` on `.cm-md-link`

### Open Items
- CE Layer 2: Pending
- Font theme application: still duplicated between +layout.svelte and SecondScreenPage
- SS dead code cleanup: loadDashboardData, tag/recent state variables
- Index: virtual scrolling for 60k+ terms

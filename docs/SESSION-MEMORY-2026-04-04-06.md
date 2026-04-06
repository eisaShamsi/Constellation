---
name: Session 2026-04-04/06 — Massive build session summary
description: Complete record of all achievements, principles, architectural decisions, and lessons from the April 4-6 2026 marathon session
type: project
---

# Session Summary: April 4–6, 2026

## What Was Built (Chronological)

### 1. Post-NotePane Rebuild Audit & Fix
- Full app audit discovered 4 out of 5 NotePane instances were silently broken (using legacy `tab={}` interface)
- Split View, Index Preview, Second Screen Detail, Second Screen Peek — all fixed with correct individual props
- Split View: added draggable dividers between panes (per-divider tracking, RTL-aware)
- Workspace restore: added missing 'health', 'provenance', 'review' tabs to validTabs
- Settings propagation: `$effect` in SettingsModal watches visual settings and calls `notifySettingsChanged()`
- Removed dead `LinkDashboard` import

### 2. NoteEditor Wrapper (Shared Component)
- Created `NoteEditor.svelte` — one component wrapping NotePane with all props/callbacks
- Replaced all 7 NotePane call sites (main editor, split view, index, SS detail, SS peek, dashboard note, dashboard tag)
- Net: 291 added, 490 removed — eliminated ~200 lines of duplicated code

### 3. Shared Utilities Extracted
- `colors.ts` — `buildLibraryColorMap()` (was duplicated in 2 files)
- `recentNotes.ts` — `getRecentLists()`, `addRecentOpened()`, `addRecentEdited()` (was in 3 files)
- `tagUtils.ts` — `scanAllLibraryTags()` (was in 2 files)

### 4. Dashboard Home Screen
- Extracted `DashboardView.svelte` from SecondScreenPage (~360 lines → shared component)
- Main window: optional dashboard when no tabs open (toggle, default off)
- SS: receives recently edited/opened note clicks + tag split view
- Tag click with SS open: no duplicate panel on main window

### 5. Second Screen Architecture Fix
- Removed `onNoteSaved` content re-read (NoteEditor handles its own state)
- `loadAllData()` only shows spinner on first startup (`initialLoadDone` guard)
- Main window listens for `screen:note-saved` to sync SS edits back

### 6. Callout RTL Fix
- Detect RTL script from actual title/body text content
- Set `dir="rtl"` or `dir="ltr"` explicitly on callout line decorations
- Breadcrumb padding aligned with paper padding (16px → 48px)

### 7. Index — Full NLP Pipeline (Rust)
- Arabic: Lucene Light10 (normalize → prefix → suffix, two-level display vs key normalization)
- English: Porter-like stemmer
- French/Spanish/Portuguese: char-safe UTF-8 suffix removal
- German: umlaut normalization + suffixes
- Russian: case/gender/number suffixes
- Turkish: agglutinative suffix removal
- Hindi/Persian: suffix removal with normalization
- Hebrew: prefix removal (ב/ל/מ/ה/ו/כ/ש)
- Japanese/Korean/Chinese: stopword filtering
- All 15 languages: comprehensive stop word lists
- Arabic display: preserves original characters (ة أ إ آ ى) — only index key normalizes

### 8. Index UX Polish
- Letter filtering: click letter → shows only that letter's terms + count
- One term expanded at a time
- Comma-separated search: substring match
- Ctrl+Click opens note as regular tab + "Return to Index" button (state preserved via display:none)
- Selected terms anchor bar (Ctrl+Click multi-select, persistent chips)
- Term highlight: wholeWord for non-Arabic, regex with word boundaries for Arabic
- Term count reflects active language/letter filter

### 9. Index → Second Screen
- Term click → SS shows note list + editor (same pattern as Dashboard tags)
- Ctrl+Click multi-term → SS compare mode (columns per term)

### 10. Wikilink Navigation
- Pointer cursor on hover (`.cm-md-link { cursor: pointer }`)
- Single click: opens linked note (same tab)
- Ctrl+Click: opens in new tab
- Non-existent note: creates with default frontmatter
- Uses `mousedown` not `click` (fires before CM6 strips livePreview decorations)
- Wired in NoteEditor — works everywhere automatically

### 11. CE Layer 2: Constellation Map
- Rust: `map.rs` with `constellation_map_data` + `constellation_map_universe`
- Svelte: `ConstellationMap.svelte` with D3.js `d3.partition()` sunburst
- Full hierarchy: Universe → Child Universes → Libraries → Folders → Notes
- 3 color modes: Maturity, Stratum, Library
- Drill-down with breadcrumb, Escape to go up
- Center text stacked (title/notes/words), clipped to ring boundary
- "Return to Map" button (state preserved via display:none overlay)
- Always-rendered overlay pattern (skip render when container < 10px)

### 12. Map → Second Screen Companion
- Universe level: grid of library mini-sunburst cards
- Library/folder drill-down: child mini-maps in grid
- Note click: NoteEditor + context mini-map
- Color mode syncs across all maps
- Compact mode for mini-maps (no header/breadcrumb/legend)
- Only emits when Map is actually visible

## Architectural Principles Established

### 1. Secure the Winning
If a feature works, extract into a shared component. Never copy-paste and adapt. One source of truth, tested once, used many times.
**How to apply:** Before building, check if the working version exists. Extract and reuse. NoteEditor is the reference example.

### 2. Screens Are Displays, Not Domains
Second screen mounts core components — never re-implements save/load/edit. The core editor handles all operations regardless of which window it's in.
**How to apply:** No `onNoteSaved` re-reads, no `loading = true` on file changes, no competing tab management in SS.

### 3. Testing Instructions Rule
Every test must include a tutorial: define the feature (what, why, why it matters) + step-by-step walkthrough (every click, every field, every expected result).
**How to apply:** This builds documentation as we build software. Never assume the user knows internal component names.

### 4. Don't Patch More Than Three Times
If three attempts fail, stop and find the root cause.
**How to apply:** The wikilink click required root cause analysis — CM6 strips decorations on click, so we use mousedown.

### 5. Constellation's Essence
An extension of the mind. Focus on writing. Simple by default, powerful on demand. With a second monitor, distractions move to SS — main window becomes clean writing space.

## CE Layer Architecture
- Layer 1: Structural Cognition (11 phases) — COMPLETE
- Layer 2: Constellation Map — radial knowledge visualization — PHASE 1 COMPLETE
- Layer 3: Constellation Lens — network analysis engine (future)
- Layer 4: AI Discovery — embeddings, semantic links, AI insight (future)

## Key Milestones (Git Tags)
- `milestone/post-notepane-audit`
- `milestone/dashboard-split-companion`
- `milestone/dashboard-ss-interaction`
- `milestone/index-nlp-complete`
- `milestone/index-arabic-display`
- `milestone/wikilink-navigation`
- `milestone/constellation-map-phase1`
- `milestone/map-ss-companion`

## Documentation Created
- `docs/WORK-BEHAVIOR.md` — session protocols, testing rules, principles
- `docs/PCS-PROTOCOL.md` — Push + Commit + Standing Order guide

## Next Session Focus
SS architecture redesign:
1. Monitor detection (2+ monitors required)
2. No auto-restore on startup
3. Main window = clean writing space (no right sidebar when SS active)
4. Panels migrate to SS (Properties, Backlinks, Tags, Star, Tasks, Health, Provenance, Review)
5. Context-aware SS (adapts to editing → panels, Map → companion, Index → exploration)

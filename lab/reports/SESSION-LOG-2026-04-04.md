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

### Open Items
- CE Layer 2: Pending
- Font theme application: still duplicated between +layout.svelte and SecondScreenPage (~65 lines)
- SS dead code cleanup: loadDashboardData, tag/recent state variables still present

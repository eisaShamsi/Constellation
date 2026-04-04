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

### Open Items
- CE Layer 2: Pending
- Dashboard in second screen: test tag interaction and note clicking
- Split companion: backlinks/tasks panels need library scan integration

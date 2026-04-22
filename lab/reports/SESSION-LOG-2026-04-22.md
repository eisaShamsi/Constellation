# Session Log — 2026-04-22

## § 49. Tier 1 Settings UI: Panel Placement Picker

Settings → Panels section added so users can control where each of the
11 movable panels lives (left-of-note flanking, right-of-note flanking,
right sidebar, or hidden).

### Changes

- `SettingsModal.svelte`:
  - New `'panels'` section added to the sections array with a `layout`
    icon (grid-based SVG path).
  - `PanelId`, `PanelSlot` types imported from store.
  - Section renders an `{#each}` over all 11 panels — each row shows
    panel name, description, and a 4-option slot `<select>`. On change:
    spreads the existing `panelPlacements` record, overrides the single
    changed key, calls `updateSettings()`.
  - "Reset to defaults" button at the bottom restores
    `DEFAULT_SETTINGS.panelPlacements`.
- `src/lib/i18n/*.json` — all 15 locales updated via
  `lab/scripts/i18n_panels.py` (new script). Adds:
  - `settings.sections.panels` — section label.
  - `settings.panels.{intro, slotsHeading, slotLeftOfNote,
    slotRightOfNote, slotRightSidebar, slotHidden, panel*, resetDefault,
    resetDefaultDesc}`.
- svelte-check: 55 pre-existing errors, 55 after — zero regression.

### Commit

`b8f018d` — §49 Tier 1 Settings UI: panel placement picker in Settings → Panels

---

## § 50. WTD: Eliminate Tag Browser Filesystem Scan

Write-Time Derivation fix (CLAUDE.md Rule 8). The tag browser was calling
`scan_library_tags` (7,600 file reads) on every NotebookNavigator mount.
The boot snapshot already reads tag counts from SQLite's `note_meta` table
(`tags_json` column), maintained at write time by the indexer.

### Root cause

`NotebookNavigator.svelte:88` called `invoke('scan_library_tags', ...)` in
`onMount()` for every library on every open — O(N file reads).
`allLibraryTags` in `+layout.svelte` was already populated from
`cache_boot_snapshot_graph` (O(1) SQLite query), but was never passed down.

### Fix

- `NotebookNavigator.svelte`: Added `initialTags?: Record<string,number>`
  prop. When non-empty, `onMount` skips `scan_library_tags` entirely;
  `tagMap = initialTags`. Falls back to filesystem scan only when
  `initialTags` is empty (second screen, or mount before graph ready).
- `+layout.svelte`: Passes `{allLibraryTags}` as `initialTags` to
  NotebookNavigator.

### Result

Opening the Tags browser: O(7,600 file reads) → O(1) on any session
where the boot graph has loaded.

### Commit

`7f8a980` — §50 WTD: eliminate scan_library_tags filesystem scan in NotebookNavigator

---

## § 51. Tier 1b: Draggable Resize Handles on Flanking Columns

Users can now drag the border between a flanking panel and the note editor
to resize it. Width is persisted to `appSettings`.

### Changes

- `store.ts`:
  - `AppSettings` gains `leftOfNoteWidth: number` (default 280) and
    `rightOfNoteWidth: number` (default 280).
  - `DEFAULT_SETTINGS` updated with both new fields.
- `+layout.svelte`:
  - `leftFlankWidth` / `rightFlankWidth` `$state` vars, initialized from
    `appSettings` via a once-only `$effect` (gate: `flankWidthsLoaded`).
  - `startFlankResize(side, e)`: pointer-tracked drag handler. Clamps
    [180, 500] px. RTL-aware (delta flipped). Calls `updateSettings()`
    with final widths on `mouseup` — one IPC call per drag.
  - `.flank-handle` divs (4px visual, 12px hit target via `::before`)
    between flank-start↔center and center↔flank-end.
  - `flank-resizing` class on wrapper during drag.
  - `.flank` CSS: `flex-basis` moved to inline `style:flex-basis` binding.

### Commit

`a1c581e` — §51 Tier 1b: draggable resize handles on flanking columns with width persistence

---

## Push

All §48 (backlog commit) + §49 + §50 + §51 pushed to `origin/main`.
`git push origin main` → `33f1aa1..a1c581e`.

## Still on the queue

- Tier 2: drag-and-drop panel rearrangement (slot highlighting + drop zones)
- Tier 3: detachable floating panels (needs Tauri multi-window)
- Write-Time Derivation: Sky View skyNodes/skyLinks (already cached from
  SQLite via cache_boot_snapshot_graph — `buildSkyData` JS transform is
  the remaining O(N+E) cost; acceptable for now)
- Write-Time Derivation: Sight dashboard, sidebar star counts — audit pending
- navTrace instrumentation dev-gate
- Settings → Debug Boot Performance scorecard UI
- RTL alignment pass on Arabic docx
- Fix `picker` / `knowledgeHealth` i18n keys missing from non-EN locales
  (55 pre-existing svelte-check errors — separate cleanup task)

---

## § 52. Clear All svelte-check Type Errors (55 → 0)

**Commit**: da8dc1e  
**Status**: ✅ Complete

### Problem
55 pre-existing `svelte-check` type errors accumulated across multiple sessions.
TypeScript reported cascading errors in `src/lib/i18n/index.ts` plus scattered
component and library errors.

### Solution: i18n Gaps (39 keys × 14 locales)

Two scripts written to add missing translation keys:
- `lab/scripts/i18n_picker_khealth.py` — `picker` + `knowledgeHealth` top-level sections
- `lab/scripts/i18n_complete_fix.py` — comprehensive fix for all remaining gaps:
  `settings.debug` (23 keys), `settings.templates.*` (16 keys), `settings.sections.*` (2 keys),
  `settings.plugins.emojiIconPicker` (2 keys), `commands.newLibrary`, `sidebar.newLibrary`,
  `ribbon.knowledgeHealth`, `constellationMap.*` (3 keys)
  Applied to all 14 non-English locales (+37–39 keys each).

### Solution: TypeScript Fixes (14 remaining after i18n)

| File | Fix |
|------|-----|
| `store.ts` | Added `emojiIconPicker` to `enabledFeatures` type + `DEFAULT_SETTINGS` |
| `store.ts` | Added `status?: 'active' \| 'archived'` to `NoteLink` interface |
| `store.ts` | Added `ai?: { contextLines?, libraryAccess?, ... }` to `AppSettings` |
| `NoteEditor.svelte` | `resolved.libraryName` → `resolved.library_name`; compute color via `buildLibraryColorMap(get(libraries))` |
| `OutgoingLinksPanel.svelte` | Same `library_name` fix |
| `ConstellationSight.svelte` | Added missing `searchMatchIds` prop destructuring |
| `ConstellationMap.svelte` | Use `any` for `MapNode` in `$props()` type annotation (Svelte 5 scope quirk) |
| `EditorContextMenu.svelte` | Added `'list' \| 'blockquote'` to `CursorContext` type |
| `livePreview/lineDecoPlugin/bidiPlugin.ts` | `view.destroyed` → `(view as any).destroyed` (CM6 private field) |
| `markdownHighlight.ts` | Cast string `style` values to `any` for Lezer compatibility |
| `graphEngine.ts` | `@ts-ignore` on PixiJS `direction` property |
| `semanticEngine.ts` | `embedder: any` for `FeatureExtractionPipeline` (Xenova types mismatch) |
| `SearchHub.svelte` | Wrap `onClose` in arrow fn for `MouseEventHandler` compatibility |
| `SettingsModal.svelte` | Optional chaining `ai?.contextLines`, `editingTheme?.id` |
| `PropertyEditor.svelte` | Fix `getDateDir()` return type to `'ltr' \| 'rtl' \| 'auto'` |
| `OrgChart/+layout.svelte` | Non-null assertions inside `{#if}` guards |
| `vite.config.js` | Remove stale `@ts-expect-error` |

### Result
- Before: 55 errors  
- After: **0 errors**, 284 warnings (warnings are CSS/minor, not blocking)

### Still on the queue
- Tier 2: drag-and-drop panel rearrangement (slot highlighting + drop zones)
- Tier 3: detachable floating panels (needs Tauri multi-window)
- Write-Time Derivation: Sight dashboard, sidebar star counts — audit pending
- navTrace instrumentation dev-gate
- Settings → Debug Boot Performance scorecard UI

---

## § 53. Flanking Panel Collapse Buttons + Sidebar Tab Safety Reset

**Commit**: 4834697  
**Status**: ✅ Complete — pushed to origin/main

### Changes

**Flanking panel collapse buttons**
- Added `leftFlankCollapsed` / `rightFlankCollapsed` `$state` booleans in `+layout.svelte`
- Left drag handle becomes a flex column (`flank-handle-wrap`) with a collapse toggle button
  above the resize strip — chevron points right when open, left when collapsed (RTL mirrors)
- Right side mirrors: chevron points left when open, right when collapsed
- Collapsed flank: `flex-basis: 0`, `min-width: 0 !important`, `overflow: hidden`,
  `padding: 0` — content hidden with 120ms ease transition (CSS only, no JS layout)
- Resize is disabled when flank is collapsed (`!leftFlankCollapsed && startFlankResize(...)`)

**Sidebar tab safety reset**
- Added `$effect` that watches `$appSettings.panelPlacements`; if `rightSidebarTab === 'backlinks'`
  but neither backlinks nor outgoing is placed in `right-sidebar`, resets tab to `'properties'`
  — prevents blank right sidebar when user moves panels away from the sidebar

### Still on the queue
- Tier 2: drag-and-drop panel rearrangement (slot highlighting + drop zones)
- Tier 3: detachable floating panels (needs Tauri multi-window)
- Right sidebar tab bar: "hidden" placement panels still show their tabs
  (only backlinks/outgoing are gated; properties, tags, sky, etc. always show)
- Write-Time Derivation: Sight dashboard (Louvain + health computed on each toggleLens() call)
- Write-Time Derivation: sidebar star counts audit
- navTrace instrumentation dev-gate

---

## § 54. Gate All Right-Sidebar Tabs on Panel Placement

**Commit**: 75bfd4d  
**Status**: ✅ Complete — pushed to origin/main

### Problem
Only the `backlinks` tab was gated on its `panelPlacements` setting. The other 9
tabs (properties, tags, sky, tasks, calendar, health, provenance, review, links)
always showed in the right sidebar tab bar regardless of whether those panels had
been moved to a flank column or hidden via Settings → Panels.

### Solution
- Wrapped each `rs-tab` button in `{#if ($appSettings.panelPlacements?.ID ?? fallback) === 'right-sidebar'}`
  — fallback defaults match DEFAULT_SETTINGS (left-of-note for backlinks,
  right-of-note for outgoing, right-sidebar for everything else)
- Added `type PanelId` to layout.svelte type imports
- Expanded the safety `$effect` from single-panel check (backlinks only) to a full
  10-panel map; if `rightSidebarTab` references a panel no longer in right-sidebar,
  automatically switches to the first still-visible tab in priority order

---

## § 55. Write-Time Derivation Cache for Constellation Lens Analytics

**Commit**: a19bc05  
**Status**: ✅ Complete — pushed to origin/main

### Problem
Every time the user toggled the Constellation Lens on, 9 expensive steps ran:
Rust centrality IPC, Louvain community detection (JS), structural gap analysis,
universe health scoring, stratum-weighted centrality, bridge list, community
profiles, bridge suggestions, and contradiction detection. Turning the lens off
and back on (even without any notes changing) triggered the full pipeline again.

### Solution: WTD in-memory cache
- Added `lensDataStale = $state(true)` flag
- Added `$effect` watching `skyVersion`: when the sky graph rebuilds after a
  successful computation (`lensHealth !== null`), marks `lensDataStale = true`
- Modified `toggleLens()`:
  - **Toggle OFF**: keeps all computed data in memory (just hides overlay)
  - **Toggle ON, data fresh**: instant activation — O(1), no IPC or computation
  - **Toggle ON, data stale**: runs full 9-step pipeline, then clears stale flag
    (with race guard: only clears if `skyVersion` hasn't changed during async work)
- Result: Louvain + centrality runs at most once per graph change — every
  subsequent open within the same graph state is instant

### Still on the queue
- Write-Time Derivation: Backlinks/Outgoing panels (recomputed on tab focus)
- Write-Time Derivation: Tag browser (scanned on open)
- Write-Time Derivation: Sight dashboard Sight2 component
- Tier 2: drag-and-drop panel rearrangement (slot highlighting + drop zones)
- navTrace instrumentation dev-gate

---

## § 56. Living Link Panel Tooltips + Roadmap Update

**Commit**: 89fa974  
**Status**: ✅ Complete — pushed to origin/main

### Changes

**lastTraversed in Backlinks/Outgoing tooltips**
- `getBacklinks()` and `getOutgoingLinks()` in `store.ts` now include
  `lastTraversed: l.last_traversed ?? ''` in each returned link object
- `BacklinksPanel.svelte`: added `fmtTraversed()` relative-time helper;
  traversal chip tooltip expanded to: "Traversed N times · tier · Last: 2d ago"
- `OutgoingLinksPanel.svelte`: same helper and tooltip expansion
- Shows "today", "yesterday", "Nd ago", "Nw ago", "Nmo ago", "Ny ago"

**Cognitive roadmap update**
- `docs/cognitive-engine-roadmap.md`: updated P2–P5 to ✅ — all Living
  Link Architecture phases are shipped (traversal tracking, confidence levels,
  weight+lifecycle, formulation analysis, Knowledge Health dashboard)

---

## § 57. Panel Placement Help Documentation

**Commit**: (next)  
**Status**: 🔄 In progress

### Changes
- `docs/help.uConstellation.World/Panels/Panels.md` — NEW help file
  covering the panel placement system (slots, flanking columns, collapse
  buttons, right sidebar tabs, workspaces)
- `docs/User Manual.md` — new §15b Panels section with table, description
  of flanking column resize/collapse behavior, sidebar tab gating, workspaces
- ToC entry added for §15b

## Session Summary — 2026-04-22

### Total changes this session

| § | Commit | Change |
|---|--------|--------|
| §52 | da8dc1e | Fixed all 55 pre-existing svelte-check type errors (0 errors remaining) |
| §53 | 4834697 | Flanking panel collapse buttons + sidebar tab safety reset |
| §54 | 75bfd4d | Gate all right-sidebar tabs on panel placement setting |
| §55 | a19bc05 | WTD cache for Constellation Lens analytics (instant re-open) |
| §56 | 89fa974 | Living Link: lastTraversed in panel tooltips + roadmap update |
| SO57 | 48638fa | Help page + User Manual for Panel Placement System |

### Key improvements
1. **Zero type errors**: All 55 pre-existing TypeScript errors cleared
2. **Collapse buttons**: Flanking panels now have triangle toggle buttons with 120ms animation
3. **Tab placement gating**: Right sidebar tabs correctly hidden when panel is moved/hidden
4. **WTD Lens cache**: Constellation Lens analytics cached in memory — instant toggle if graph unchanged
5. **lastTraversed tooltip**: Traversal chip shows "Last: 2d ago" relative date on hover
6. **Documentation**: New Panels help page + User Manual §15b added

### Still pending
- Tier 2: drag-and-drop panel rearrangement with slot highlighting + drop zones
- Tier 3: detachable floating panels (needs Tauri multi-window)
- Write-Time Derivation: Sight dashboard (ConstellationSight2 computes graph on every open)
- Write-Time Derivation: Backlinks in SQLite (currently pure JS filter — acceptable for now)
- navTrace instrumentation dev-gate
- Functional test for Panel Placement with user (definitions + step-by-step)

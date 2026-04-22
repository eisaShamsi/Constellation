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

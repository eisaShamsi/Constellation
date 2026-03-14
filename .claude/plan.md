# Plan: Second Screen (Dual Monitor) Feature

## Overview

Add a Lightroom-style **Second Screen** feature. A dedicated companion window with **switchable view modes** that the user can drag to a second monitor. Both windows share the same universe, vaults, and file watchers.

**Monitor 1 (Main):** Full app — sidebar, graph, search, tabs, editor
**Monitor 2 (Second Screen):** Companion window with view mode selector

---

## View Modes (like Lightroom's Secondary Window)

The second screen has a **mode switcher** in its toolbar (bottom or top bar with icons):

| Mode | Icon | Description |
|------|------|-------------|
| **Grid** | ▦ | Card grid of all notes across vaults — shows note title, vault color dot, preview snippet, modified date. Click to open in main window or in Detail mode. Searchable/filterable. |
| **Graph** | ◉ | Full graph view (reuses GraphView component). Click a node → opens in main window. Navigation-focused. |
| **Detail** | ▢ | Single note reader/editor (NotePane). When you click a note in main window's tree/search, it shows here. |

A small **mode indicator bar** at the bottom shows which mode is active (like Lightroom's G / E / D shortcuts).

### Interaction Between Windows

- **Main → Second Screen:** Clicking a note in the sidebar/search/graph with `Ctrl+click` or a dedicated button sends it to the second screen's Detail mode.
- **Second Screen → Main:** Clicking a note card in Grid mode or a node in Graph mode opens it in the main window's editor.
- **Linked browsing (optional toggle):** When enabled, navigating in the main window automatically updates the second screen's Detail view (like Lightroom's "lock" behavior).

---

## Architecture

### Tauri Multi-Window

- Main window → `/` (existing layout)
- Second screen → `/screen` (new SvelteKit route)
- Created at runtime via `WebviewWindow` (not at startup)

### State Sync via Tauri Events

| Event | Direction | Purpose |
|-------|-----------|---------|
| `screen:open-note` | Main → Screen | Send note to detail view |
| `screen:open-in-main` | Screen → Main | Send note to main editor |
| `note-saved` | Both ↔ Both | Refresh when note saved in either window |
| `vault-changed` | Rust → Both | File watcher (already exists) |
| `universe-switched` | Main → Screen | Reload everything |
| `screen:mode-changed` | Screen → Main | Inform main of current mode |
| `screen:closed` | Screen → Main | Cleanup |

### Second Screen State (local, per-window)

- `currentMode`: 'grid' | 'graph' | 'detail'
- `detailNote`: currently displayed note in detail mode
- `gridFilter`: search/filter text for grid mode
- `gridSort`: sort order (name, date, vault)
- `linkedBrowsing`: boolean — auto-follow main window navigation

---

## Implementation Steps

### Phase 1: Tauri Backend

1. **`src-tauri/capabilities/default.json`** — Add `"second-screen"` to windows array, add permissions: `core:window:allow-create`, `core:window:allow-close`, `core:window:allow-set-focus`, `core:window:allow-set-title`
2. **`src-tauri/src/lib.rs`** — Add `open_second_screen` command: creates `WebviewWindow` labeled `"second-screen"` pointing to `/screen`, sized to fill the second monitor if available

### Phase 2: Second Screen Route

3. **`src/routes/screen/+layout.svelte`** — Minimal layout: theme + i18n setup, Tauri event listeners, no sidebar
4. **`src/routes/screen/+page.svelte`** — Second screen UI:
   - Mode switcher bar (Grid / Graph / Detail icons)
   - Conditional rendering based on `currentMode`
   - Grid mode: card layout with note cards, search bar, vault filter, sort
   - Graph mode: reuse `GraphView.svelte` component
   - Detail mode: reuse `NotePane.svelte` component
   - Linked browsing toggle in toolbar
   - Event listeners for cross-window sync

### Phase 3: Grid Mode Component

5. **`src/lib/components/NoteGrid.svelte`** — New component:
   - Loads all notes via existing `collect_vault_notes` IPC
   - Renders as responsive card grid
   - Each card: note title, vault color indicator, 2-line preview, modified date
   - Search input filters by name/content
   - Sort: by name, date modified, vault
   - Click card → emits event to open in main window
   - Double-click → switches to Detail mode with that note

### Phase 4: Cross-Window Communication

6. **`src/lib/secondScreen.ts`** — Helper module:
   - `openSecondScreen(mode?)` — invoke Rust command, optionally set initial mode
   - `sendNoteToScreen(note)` — emit `screen:open-note`
   - `sendNoteToMain(note)` — emit `screen:open-in-main`
   - `isSecondScreenOpen()` — check if window exists
   - `closeSecondScreen()` — close the window

### Phase 5: Main Window Integration

7. **Toolbar button** — Monitor icon (like Lightroom's `2` button) in top toolbar, toggles second screen
8. **Sidebar/tree context** — `Ctrl+click` on any note sends it to second screen
9. **Graph node context** — Right-click option "Open in Second Screen"
10. **Tab context menu** — "Send to Second Screen" option

### Phase 6: Polish

11. **Window title** — "Constellation - Screen 2 - [Mode] - [Universe]"
12. **Keyboard shortcuts** — `G` for Grid, `E` for Graph, `D` for Detail (when second screen focused)
13. **i18n** — Add keys across all 15 locales
14. **Help docs** — Create `docs/.../Second Screen/Second Screen.md`
15. **Workspace save/restore** — Include second screen mode + state in workspace data

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/routes/screen/+page.svelte` | **Create** — Second screen with mode switcher |
| `src/routes/screen/+layout.svelte` | **Create** — Minimal layout (theme + i18n) |
| `src/lib/components/NoteGrid.svelte` | **Create** — Card grid view for Grid mode |
| `src/lib/secondScreen.ts` | **Create** — Cross-window communication |
| `src-tauri/capabilities/default.json` | **Modify** — Window permissions |
| `src-tauri/src/lib.rs` | **Modify** — `open_second_screen` command |
| `src/routes/+layout.svelte` | **Modify** — Toolbar button, Ctrl+click handling |
| `src/lib/vaults/store.ts` | **Modify** — Note save broadcasting |
| 15 x `src/lib/i18n/*.json` | **Modify** — Second screen translations |
| `docs/.../Second Screen/Second Screen.md` | **Create** — Help documentation |

---

## Risks & Mitigations

- **Store isolation:** Each window has own Svelte stores — sync via Tauri events, not shared memory. This is desired (independent state per window).
- **Same-note editing:** If both windows edit same note, last-save-wins. V1 shows a warning badge when a note is open in both.
- **Performance:** Grid mode loading all notes at once could be slow with 1000+ notes. Use virtual scrolling for the card grid.
- **Graph reuse:** GraphView component currently lives inside layout. Need to make it standalone/importable for the second screen.

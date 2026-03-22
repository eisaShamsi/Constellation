---
aliases:
  - Notes Management
  - Sidebar
  - File Explorer
  - Notes Navigator
  - Sky View
  - OrgChart
  - Organization Chart
  - File tree
  - Notebook Navigator
  - PiP overlay
  - Picture-in-Picture
description: The Notes Management sidebar unifies File Explorer, Notes Navigator, and Sky View (OrgChart) into a single tabbed panel for browsing your knowledge base.
---

# Notes Management

The Notes Management sidebar is the primary way to browse and organize your notes in Constellation. It unifies three previously separate views — File Explorer, Notes Navigator, and Organization Chart (Sky View) — into a single sidebar panel with mode tabs.

## Elements toolbar

The top row of the sidebar always shows the **Elements toolbar** with quick-action buttons:

| Button | Action |
|--------|--------|
| **New Note** | Create a new note in the selected folder |
| **New Base** | Create a new base (structured data note) |
| **New Folder** | Create a new folder in the selected library |

These buttons are always visible regardless of which mode tab is active.

---

## Mode tabs

The second row contains three mode tabs. Click a tab to switch how your notes are displayed:

| Tab | Icon | Description |
|-----|------|-------------|
| **Tree** | Folder tree icon | Classic File Explorer — browse your libraries as a folder hierarchy |
| **List** | List icon | Notes Navigator — dual-pane file browser with folder, tag, and property browsing |
| **OrgChart** | Tree diagram icon | Sky View — interactive tree-list hierarchy visualization |

Your selection and scroll position are preserved when switching between tabs.

> [!note]
> The Star View ribbon icon has been removed from the left dock. Sky View (OrgChart) is now accessible via the OrgChart mode tab in this sidebar. Star View (the full 3D graph) is still available via `Ctrl+G` or Mission Control.

---

## Adaptive sidebar width

The sidebar automatically adjusts its width to fit the longest library or child universe name visible in the current view. This ensures all names are fully readable without manual resizing.

---

## Child universe grouping

Across all three modes, content is organized with consistent grouping:

1. **Child universes first** — each child universe appears as a collapsible group with its libraries nested inside
2. **Own libraries below** — the parent universe's own libraries appear below a visual separator

This grouping is consistent across Tree, List, and OrgChart modes, so switching tabs does not change the structural hierarchy.

---

## Cross-mode selection sync

Clicking a child universe, library, folder, or note in any sidebar mode highlights the corresponding nodes in the Star View graph (if Star View is open). This bidirectional sync helps you maintain spatial awareness as you browse your knowledge base in different modes.

---

## Picture-in-Picture (PiP) overlay

When Star View is open and you click a child universe, library, or folder in the sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay on top of the main Star View graph.

### What the PiP shows

| Feature | Description |
|---------|-------------|
| **Filtered sub-graph** | Only nodes belonging to the selected scope (universe, library, or folder) |
| **Filtered legend** | Its own color legend showing only the relevant entries |
| **Resizable** | Drag the edges or corners to resize the PiP window |
| **Repositionable** | Drag the title bar to move the PiP anywhere on screen |

The PiP provides a focused view of a subset of your graph without losing the full Star View context behind it.

---

## Tree mode (File Explorer)

The classic file tree for browsing notes and folders within your libraries:

- Expand and collapse folders by clicking or using arrow keys
- Right-click for context menu (New Note, New Folder, Rename, Delete)
- Drag and drop to move notes between folders
- Folders and notes are grouped by child universe membership

---

## List mode (Notes Navigator)

A dual-pane browser for advanced note browsing:

### Browse sources

| Source | Description |
|--------|-------------|
| **Folders** | Navigate your library folder tree. Click a folder to see its notes. |
| **Tags** | Hierarchical tag browser. Click a tag to see all notes with that tag. |
| **Properties** | Search notes by frontmatter property key/value pairs. |

### File list

The right pane shows matching notes with:

- **Title** and **preview snippet** (first 100 characters)
- **Tag badges** (colored pills)
- **Relative date** (e.g., "2d ago", "3mo")
- **Library color dot**

### Sorting

Click the sort buttons to order by:

- **A** — Name (alphabetical)
- **Clock icon** — Last modified
- **##** — File size

### Batch operations

Select multiple files using checkboxes, then use the batch bar:

- **Tag** — Add a tag to all selected notes
- **Move** — Move selected notes to a different folder
- **Delete** — Delete selected notes (with confirmation)

---

## OrgChart mode (Sky View)

An interactive tree-list visualization of your entire knowledge base hierarchy.

### Hierarchy sources

Switch the hierarchy source using the dropdown:

| Source | What it shows |
|--------|---------------|
| **Folders** | Library > Folder > Subfolder > Note (default) |
| **Tags** | Tag taxonomy tree (nested tags like `#science/physics` become branches) |
| **MOC Links** | Hub notes with 5+ outgoing links and their targets |
| **Parent Property** | Notes with `parent: [[X]]` in frontmatter form a chain |

### Interactions

| Action | Effect |
|--------|--------|
| **Click a folder** | Expand or collapse its children |
| **Click a note** | Opens it in the editor |
| **Double-click a folder** | Drill down — re-roots the chart at that folder |
| **Breadcrumb trail** | Navigate back up after drilling down |
| **Ctrl+F** | Search — highlights the path from root to matching nodes |

---

## Keyboard navigation

| Key | Action |
|-----|--------|
| **Arrow Up/Down** | Navigate the file list or tree |
| **Arrow Left/Right** | Collapse/expand folders in Tree and OrgChart modes |
| **Enter** | Open the focused note |
| **Space** | Toggle checkbox on the focused note (List mode) |

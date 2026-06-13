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
| **New Note** | Open the Create dialog to make a new note |
| **New Base** | Open the Create dialog to make a new base (structured data note) |
| **New Folder** | Open the Create dialog to make a new folder |
| **New Library** | Open the Create dialog to make a new library |

These buttons are always visible regardless of which mode tab is active.

---

## The Create dialog

Every "create" affordance in Constellation — the toolbar buttons above, the right-click menu on a folder or library, and the "+ New …" command-palette entries — opens the same modal dialog. This is intentional: you name the new item **before** it lands on disk, and validation happens upfront, so the typical operating-system create flow is preserved across the whole app.

### What you see

When you click any "create" affordance, a dialog appears with these fields:

| Field | Behavior |
|-------|----------|
| **Title** | Identifies what's being created — *New Note*, *New Folder*, *New Base*, or *New Library*. |
| **Location** | The parent folder. **Read-only** when you invoked the create from a context that already knows the location (right-clicking a folder, or the toolbar button which uses the active library's root). **Pickable** for *New Library* — a *Pick…* button next to the field opens an OS folder picker so you can choose where the library lives. **Hidden** for *New Base* in the workspace (workspace bases always live in the workspace folder; there is no location to pick). |
| **Name** | Pre-filled with a sensible default (e.g. *New Folder*, *Untitled*, *Untitled Base*, *My Library*) and pre-selected so a single keystroke replaces it. Type the name you actually want. |
| **Libraries to query** | *(New Base only, workspace flow)* A multi-select list of libraries the base will read from. *All libraries* is the default. |
| **Create** / **Cancel** | Buttons at the bottom. *Create* commits; *Cancel* closes without side effect. |

### Keyboard

| Key | Behavior |
|-----|----------|
| **Enter** | Same as clicking *Create*. |
| **Escape** | Same as clicking *Cancel*. Closes the dialog without creating anything. |
| **Click outside the dialog** | Same as Escape. |
| **Tab** | Moves focus between Location, Name, Libraries, Cancel, Create. |

The Name field has focus when the dialog opens, with the default name pre-selected — so you can start typing immediately.

### Validation

The dialog won't let you click *Create* with a name that won't work on disk:

- **Empty name** — *Create* is disabled. The dialog shows "Name cannot be empty."
- **Illegal characters** — `\`, `/`, `:`, `*`, `?`, `"`, `<`, `>`, `|` are blocked at the input. The dialog shows the full character list inline. *Create* is disabled until you remove them.
- **Folder/file already exists at the location** — when you click *Create*, the system reports the conflict back into the dialog's error region. The dialog stays open so you can rename and try again.

> [!info]
> The dialog also blocks names containing `..` (parent-directory escapes) — this is enforced both in the dialog and in the create operation itself, so a slip of the keyboard cannot accidentally place a folder outside the location you picked.

### Right-click affordances in the sidebar

Right-clicking surfaces the same dialog from these surfaces:

| Right-click on | Menu shows | What happens |
|----------------|------------|--------------|
| **A folder in the file tree** | New Note · New Folder · New Base · Rename · Delete | The first three open the Create dialog with that folder pre-filled as Location. |
| **A library row** (the universe-notes header, an own library, or a child-universe library) | New Note · New Folder · New Base | Opens the Create dialog with that library's root pre-filled as Location. Library rows do not offer Rename or Delete here — those operations live in the Library Manager. |
| **A note in the file tree** | Rename · Delete | Notes don't host children, so no "create new" options are offered. |

### Renaming a note updates every link to it

When you rename a note — either from the **file tree** (right-click → Rename) or by **editing its title** at the top of the page — Constellation rewrites every `[[wikilink]]` that points to it, across the whole library, to the new name. You don't fix links by hand, and links never silently break.

While those links are being updated you'll see a brief read-only **"Updating links…"** overlay on the affected note(s); the editor accepts no typing for that moment, so nothing is lost while the note reloads. On a small library this is near-instant; on a very large library it may take a second or two. The note that previously linked to the old name still resolves afterward, because the old title is kept as an **alias** on the renamed note.

### Templates apply uniformly

When you create a new note, Constellation looks up any folder template configured for the parent folder and applies it. **This now happens regardless of how you invoked the create** — the toolbar button, the right-click menu, and the command palette all run the same path. Earlier versions skipped templates on the right-click path; that inconsistency is fixed.

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
> The Sky View ribbon icon has been removed from the left dock. Sky View (OrgChart) is now accessible via the OrgChart mode tab in this sidebar. Sky View (the full 3D graph) is still available via `Ctrl+G` or Mission Control.

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

Clicking a child universe, library, folder, or note in any sidebar mode highlights the corresponding nodes in the Sky View graph (if Sky View is open). This bidirectional sync helps you maintain spatial awareness as you browse your knowledge base in different modes.

---

## Picture-in-Picture (PiP) overlay

When Sky View is open and you click a child universe, library, or folder in the sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay on top of the main Sky View graph.

### What the PiP shows

| Feature | Description |
|---------|-------------|
| **Filtered sub-graph** | Only nodes belonging to the selected scope (universe, library, or folder) |
| **Filtered legend** | Its own color legend showing only the relevant entries |
| **Resizable** | Drag the edges or corners to resize the PiP window |
| **Repositionable** | Drag the title bar to move the PiP anywhere on screen |

The PiP provides a focused view of a subset of your graph without losing the full Sky View context behind it.

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

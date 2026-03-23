---
aliases:
  - Dual monitor
  - Second monitor
  - Second screen
  - Multi-window
  - Companion window
  - Dashboard
description: Use Constellation's Second Screen as a mode-based companion — showing a Universe Dashboard in File Explorer mode, Navigator, Sky View, or Star View companion depending on the active sidebar.
---

# Second Screen

The Second Screen is a mode-based companion window that adapts to your current sidebar mode. Instead of duplicating the main CodeMirror 6 editor, it provides contextual views and information relevant to what you're doing.

## Opening the Second Screen

| Method | Action |
|--------|--------|
| **Ribbon** | Click the monitor icon in the left ribbon |
| **Mission Control** | Press `Ctrl+P`, type "Second Screen" |
| **Keyboard shortcut** | `Ctrl+Shift+2` |

The second screen opens as an independent window. Drag it to your second monitor for a dual-screen workflow.

When you close the main window, the second screen closes automatically.

---

## Mode-Based Companion

The second screen automatically adapts its content based on the active sidebar mode in the main window:

| Main Sidebar Mode | Second Screen Shows |
|---|---|
| **File Explorer** (tree) | Universe Dashboard |
| **Navigator** (list) | Full Navigator view |
| **Sky View** (skyview) | Sky View tree |
| **Star View** (graph) | Star View companion with backlinks, forward links, tags, and local graph |

Switching sidebar modes in the main window instantly updates the second screen.

---

## Universe Dashboard (File Explorer Mode)

When the main window is in File Explorer mode, the second screen displays a comprehensive Universe Dashboard.

### Stat Cards

At the top, large stat cards show:

- **Universe** name
- **Child Universes** count
- **Libraries** count
- **Folders** count
- **Notes** count

### Child Universes

Each child universe is listed with:

- The child universe icon and name
- Stat boxes showing the number of libraries, folders, and notes (in the universe accent color)
- Expandable list of linked libraries underneath, each with their own folder/note counts

### Libraries

Libraries not belonging to a child universe are listed separately. Each library shows:

- Library name with its color dot
- Folder and note counts in color-coded stat boxes matching the library's theme color

### Recently Edited / Recently Opened

Two side-by-side columns track your session activity:

| Column | Rule |
|--------|------|
| **Recently Edited** | Notes you opened and modified (content was saved) during the current session |
| **Recently Opened** | Notes you opened but did not edit during the current session |

Each entry shows the note name, library color dot, and timestamp. Click any entry to open the note in the main window.

> [!note]
> These lists are session-based. They reset when you restart the application.

### Tags

All tags across all libraries, sorted by usage count. Each tag shows its name and count.

**Click a tag** to expand a split view:
- Left side: the tag list (with the selected tag highlighted)
- Right side: all notes using that tag, grouped by library

Click the close button (×) to collapse the tag notes panel.

---

## Star View Companion

When the main window is in Star View mode, the second screen becomes a Star View companion that shows detailed information about the node you hover or click.

### Hover Preview

Hover over a node in the main window's Star View to see its details in the second screen:

- Note name, library, and markdown preview
- Backlinks and forward links
- Tags
- Local graph showing direct connections

### Pinned Selection

Click a node to pin it. The companion stays focused on the pinned note even when you move the mouse away. Navigation history (back/forward) lets you browse through pinned nodes.

### Peek Preview

Click any backlink or forward link in the companion to open a full editable preview in the left panel.

---

## Navigator Companion

When the main window is in Navigator mode, the second screen shows a full Navigator view. Click notes to open them in the main window. Double-click to send them to the main window.

---

## Sky View Companion

When the main window is in Sky View mode, the second screen shows the Sky View tree with the full directory structure.

---

## Settings Sync

All settings changes propagate instantly to the second screen:

| Setting | Sync |
|---------|------|
| **Language** | Instant — UI text updates without restart |
| **Theme** (light/dark/system) | Instant |
| **Fonts** (interface, text, mono, script) | Instant |
| **Font size** | Instant |

---

## Workspace Integration

Workspaces save and restore the second screen state, including whether it was open and its current mode.

---

## RTL Support

The Second Screen fully supports right-to-left (RTL) languages including Arabic, Hebrew, Persian, and Urdu. The dashboard, tags, and all companion modes render correctly in RTL.

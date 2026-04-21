---
aliases:
  - Dual monitor
  - Second monitor
  - Second screen
  - Multi-window
  - Companion window
  - Dashboard
description: Use Constellation's Second Screen as a mode-based companion — showing a Universe Dashboard in File Explorer mode, Navigator, Sky View, or Sky View companion depending on the active sidebar. Requires two or more connected monitors.
---

# Second Screen

The Second Screen is a mode-based companion window that adapts to your current sidebar mode. Instead of duplicating the main CodeMirror 6 editor, it provides contextual views and information relevant to what you're doing.

> [!important] Requires Two Monitors
> The Second Screen is only available when **two or more monitors** are connected to your computer. If only one monitor is detected, the Second Screen button, keyboard shortcut, and command palette entries are hidden. This is by design — the Second Screen is meant to live on a dedicated display so your main window stays a clean writing space.

## Opening the Second Screen

| Method | Action |
|--------|--------|
| **Dock** | Click the monitor icon in the bottom dock bar |
| **Mission Control** | Press `Ctrl+P`, type "Second Screen" |
| **Keyboard shortcut** | `Ctrl+Shift+2` |

The second screen automatically positions itself centered on your secondary display, filling approximately 80% of the screen.

When the Second Screen opens, the main window's right sidebar automatically hides — its panels (Properties, Backlinks, Tags, Sky View, Tasks) migrate to the Second Screen. When you close the Second Screen, the right sidebar returns.

The Second Screen always starts closed — it is never auto-restored from workspaces. You open it deliberately when you need it.

When you close the main window, the second screen closes automatically.

---

## Editor Panels Companion

When you're editing a note with the Second Screen open, it shows the panels that would normally appear in the right sidebar:

| Panel | What It Shows |
|-------|---------------|
| **Properties** | Frontmatter key-value pairs from the active note |
| **Backlinks** | Notes that link TO the current note, with library color dots |
| **Forward Links** | Notes that the current note links TO |
| **Tags** | Tags from frontmatter, displayed as badges |
| **Sky View** | Local graph showing the note and its direct connections |
| **Tasks** | Tasks found in the note |

The panels update automatically when you switch between tabs in the main window. Click any backlink or forward link to open that note in the main window.

---

## Mode-Based Companion

The second screen automatically adapts its content based on the active sidebar mode in the main window:

| Main Sidebar Mode | Second Screen Shows |
|---|---|
| **File Explorer** (tree) | Universe Dashboard |
| **Navigator** (list) | Full Navigator view |
| **Sky View** (skyview) | Sky View tree |
| **Sky View** (graph) | Sky View companion with backlinks, forward links, tags, and local graph |

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

## Sky View Companion

When the main window is in Sky View mode, the second screen becomes a Sky View companion that shows detailed information about the node you hover or click.

### Hover Preview

Hover over a node in the main window's Sky View to see its details in the second screen:

- Note name, library, and markdown preview
- Backlinks and forward links
- Tags
- Local graph showing direct connections

### Pinned Selection

Click a node to pin it. The companion stays focused on the pinned note even when you move the mouse away. Navigation history (back/forward) lets you browse through pinned nodes.

### Peek Preview

Click any backlink or forward link in the companion to open a full editable preview in the left panel. The peek editor is fully functional — you can type, save, change properties, and promote stage directly from the preview.

---

## Note Editing in Second Screen

The second screen supports full note editing in all modes:

- **Editor mode**: The detail view is a complete editor with toolbar, properties, stage dropdown, and save support
- **Sky View peek**: Click a link in the companion to open a full editor in the peek panel
- **Save**: Press Ctrl+S or the editor auto-saves — changes sync back to the main window automatically
- **Rename**: Edit the title to rename the file
- **Properties**: Expand the Properties panel to edit frontmatter

---

## Dashboard Interaction

When the Dashboard is active on the main window (no notes open), clicking items on the Dashboard sends them to the second screen:

### Recently Edited/Opened Notes

Click any note in the Recently Edited or Recently Opened section. The second screen opens that note as a full editor — you can read, edit, save, and change properties without leaving the Dashboard.

### Tag Browsing

Click any tag on the Dashboard. The second screen switches to a split view:

- **Left column**: All notes containing that tag, with library color dots
- **Right column**: Click any note in the list to open it as a full editor

The Dashboard stays clean — the tag split panel only appears on the second screen, not on the main window.

All edits made on the second screen are saved to disk and synced back to the main window automatically.

---

## Navigator Companion

When the main window is in Navigator mode, the second screen shows a full Navigator view. Click notes to open them in the main window. Double-click to send them to the main window.

---

## Sky View Companion

When the main window is in Sky View mode, the second screen shows the Sky View tree with the full directory structure.

---

## Settings Sync

All visual settings changes propagate instantly to the second screen:

| Setting | Sync |
|---------|------|
| **Language** | Instant — UI text updates without restart |
| **Theme** (light/dark/system) | Instant |
| **Fonts** (interface, text, mono, script) | Instant |
| **Font size** (interface and editor) | Instant |
| **Accent color** | Instant |
| **Editor settings** (readable line length, line numbers, floating toolbar) | Instant |
| **Primary script** | Instant |

---

## Split View Comparison

When Split View is active in the main window (multiple notes open side by side), the Second Screen switches to **comparison mode**:

- A **panel tab bar** at the top lets you choose which panel to compare (Properties, Backlinks, Tags, Sky View, Tasks)
- Below, **one column per note** displays that panel's data side by side
- Each column header shows the note name with a library color dot
- Click any backlink or forward link in any column to open it in the main window

This is designed for comparing metadata across notes — see which notes share tags, compare backlinks, or view tasks side by side.

---

## Index Companion

When the Index is open in the main window, clicking a term sends its data to the Second Screen:

- **Single term**: A split view with the note list on the left and a full editor on the right
- **Multi-term compare** (Ctrl+Click): One column per term, each showing its note list. Click a note to open it in the editor pane.

---

## Constellation Map Companion

When the Constellation Map is open in the main window, the Second Screen shows a companion view:

- **Drill-down view**: A grid of mini-maps for child folders/libraries
- **Note click**: Opens the note in a full editor alongside a context mini-map
- **Color mode dropdown**: Switch between Maturity, Stratum, and Library coloring
- **Legend**: Shows the color meaning for the selected mode

The color mode and legend sync across all mini-maps on the Second Screen.

---

## Workspace Integration

Workspaces save and restore the second screen state, including whether it was open and its current mode.

---

## RTL Support

The Second Screen fully supports right-to-left (RTL) languages including Arabic, Hebrew, Persian, and Urdu. The dashboard, tags, and all companion modes render correctly in RTL.

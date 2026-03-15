---
aliases:
  - Star View
  - Sky view
  - Link star view
  - Network view
  - Note connections
description: Visualize and explore the connections between your notes using Constellation's interactive Star View.
---

# Star View

Star View displays your notes as an interactive network of nodes and links. Each node is a note, and each line represents a `[[wikilink]]` between notes. The more connections a note has, the larger its node appears.

## Opening Star View

| Method | Action |
|--------|--------|
| **Mission Control** | Press `Ctrl+P`, type "star view" |
| **Ribbon** | Click the star view icon in the top toolbar |

Press `Escape` to close the star view.

---

## Interacting with the star view

- **Pan**: Click and drag on empty space.
- **Zoom**: Scroll wheel or pinch gesture.
- **Drag nodes**: Click and drag any node to reposition it.
- **Hover**: Shows the note name as a tooltip and highlights connected nodes.
- **Click a node**: Opens that note in a new tab.
- **Legend**: Click a library name in the legend to toggle its visibility.

---

## Controls panel

Click the gear icon (top-right of the star view) to open the **Settings panel**. It has four collapsible sections:

### Filters

Control which notes appear on the star view.

| Control | Description |
|---------|-------------|
| **Search** | Filter nodes by name or path. Supports `path:` prefix for folder filtering (e.g., `path:Projects`). |
| **Existing files only** | Hides "ghost" nodes — links to notes that don't exist yet. |
| **Orphans** | Toggle notes that have no links. When off, only connected notes appear. |

> [!tip]
> Combine search with library legend toggles for precise filtering. For example, hide all libraries except one, then search for a specific folder path.

### Groups

Create color-coded groups to visually categorize nodes.

1. Click **New group**.
2. Enter a search query (same syntax as the filter: plain text or `path:folder`).
3. Choose a color using the color picker.
4. All matching nodes will be painted with that color.

You can create multiple groups. The first matching group takes priority.

> [!tip]
> Use groups to color-code by project or topic. For example:
> - `path:Projects` with blue
> - `path:Resources` with green
> - `path:Archive` with gray

### Display

Control the visual appearance of the star view.

| Control | Description |
|---------|-------------|
| **Arrows** | Show directional arrowheads on links. |
| **Text fade threshold** | Controls the zoom level at which note labels become visible. Lower values show labels earlier. |
| **Node size** | Scale all nodes larger or smaller. Nodes still scale relative to their connection count. |
| **Link thickness** | Scale the width of link lines. |
| **Animate** | Toggle the force simulation animation. When off, the layout is computed instantly and frozen. |

### Forces

Control the physics simulation that arranges nodes.

| Force | Description | Effect of increasing |
|-------|-------------|---------------------|
| **Center force** | Pulls everything toward the center. | Sky view becomes more compact. |
| **Repel force** | Pushes nodes apart. | Nodes spread out, clusters separate. |
| **Link force** | Pulls linked nodes closer together. | Tighter clusters around connected notes. |
| **Link distance** | Minimum distance between linked nodes. | More spacing between connected nodes. |

> [!tip]
> A good starting point for readable star views: high link force, moderate repel force, and moderate center force. This naturally forms visible clusters around your hub notes.

Click the **reset** button (circular arrow icon) to restore all controls to their default values.

---

## Library clusters and child universes

When you have multiple libraries, the star view automatically:

- **Colors nodes** by library (each library gets a unique color).
- **Draws convex hulls** — semi-transparent colored regions around each library's notes.
- **Shows a legend** in the top-left corner listing each library with its note count.
- **Dashes cross-library links** — links between notes in different libraries appear as dashed lines.

Click any library name in the legend to hide or show that library's nodes.

### Child universe libraries in the star view

If your universe has [[Universe#Child universes (Universe of Universes)|child universes]], their libraries appear in the star view automatically alongside your own libraries. Each child universe library gets its own color, legend entry, and convex hull — just like your own libraries. Cross-library links between parent and child universe notes are shown as dashed lines.

> [!tip]
> Use the legend to toggle child universe libraries on or off. This makes it easy to focus on just your own libraries or explore how your notes connect to notes in child universes.

---

## Link types

If your notes use typed links (e.g., `[[note|type:related-to]]`), they appear with distinct colors:

| Type | Color |
|------|-------|
| related-to | Blue |
| prerequisite | Red |
| see-also | Green |
| contradicts | Amber |
| supports | Purple |
| extends | Pink |

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Escape` | Close Star View |
| `Ctrl+P` → "star view" | Open Star View |

---

## RTL support

Star View works correctly with Arabic, Hebrew, and other RTL note names. Labels render in the correct direction and tooltips display RTL text properly.

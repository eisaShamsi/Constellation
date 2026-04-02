---
aliases:
  - Star View
  - Sky View
  - GraphMind
  - Sky View
  - Link star view
  - Network view
  - Note connections
  - 3D graph
description: Visualize and explore the connections between your notes using Constellation's interactive Star View powered by the GraphMind engine.
---

# Star View

Star View displays your notes as an interactive network of nodes and links, powered by the **GraphMind** engine (Pixi.js WebGL). Each node is a note, and each line represents a `[[wikilink]]` between notes. The more connections a note has, the larger its node appears.

## Opening Star View

| Method | Action |
|--------|--------|
| **Mission Control** | Press `Ctrl+P`, type "star view" |
| **Keyboard** | `Ctrl+G` |

Press `Escape` to close the Star View.

> [!note]
> The Star View ribbon icon has been removed from the left dock. Star View is now accessible via keyboard shortcut or Mission Control. The Sky View (OrgChart) mode is available as a tab in the Notes Management sidebar.

---

## Interacting with the graph

### Basic interactions

| Input | Behavior |
|-------|----------|
| **Pan** | Click and drag on empty space |
| **Zoom** | Scroll wheel (2D) or `Ctrl+Scroll` (3D) |
| **Drag nodes** | Click and drag any node to reposition it |
| **Hover** | Shows the note name in the status bar and highlights connected nodes and edges |
| **Click a node** | Opens that note in the editor |
| **Double-click a node** | Zooms in and centers on that node |
| **Right-click a node** | Opens the context menu |

### Context menu

Right-click any node to access:

| Action | Description |
|--------|-------------|
| **Open** | Opens the note in the editor |
| **Focus** | Enters focus mode centered on this node |
| **Pin** | Locks the node at its current position. Click again to unpin. |
| **Hide** | Hides the node from the graph. Use "Show all" in the toolbar to reveal hidden nodes. |

---

## 3D navigation

Star View supports full 3D navigation — fly through your notes like navigating through stars.

### Entering 3D mode

**Middle-click and drag** (or **Alt+click and drag**) to rotate the graph in 3D space. Once rotated, 3D navigation controls become active.

### 3D controls

| Input | Action |
|-------|--------|
| **Middle-click drag** | Rotate around X and Y axes |
| **Shift+Middle-click drag** | Rotate around Z axis |
| **W / Arrow Up** | Fly forward (into the screen) |
| **S / Arrow Down** | Fly backward |
| **A / Arrow Left** | Strafe left |
| **D / Arrow Right** | Strafe right |
| **Q** | Move down |
| **E** | Move up |
| **Ctrl+Scroll** | Zoom (change field of view) |
| **Regular Scroll** | Fly forward/backward along camera direction |
| **0** | Reset rotation back to flat 2D view |
| **Reset button** (↺ icon) | Same as pressing `0` |

### XYZ axis gizmo

When in 3D mode, a color-coded axis guide appears in the bottom-left corner:

| Axis | Color | Direction |
|------|-------|-----------|
| **X** | Red | Left–Right |
| **Y** | Green | Up–Down |
| **Z** | Blue | Forward–Back (depth) |

The gizmo rotates with the camera so you always know your orientation.

### Hover and click in 3D

You can hover over and click nodes while navigating in 3D. The note name appears in the status bar, and clicking opens the note — just like in 2D mode.

---

## Layout modes

Star View offers three layout algorithms. Switch between them by pressing `Ctrl+L` or using the layout button in the toolbar.

| Mode | Description | Best for |
|------|-------------|----------|
| **Organic** | Force-directed layout. Clusters emerge naturally from link density. | General exploration — the default mode. |
| **Hierarchical** | Top-down directed acyclic graph (DAG). | Structured libraries with parent–child relationships. |
| **Temporal** | Nodes arranged along a horizontal time axis by creation date. | Seeing when notes were created and how the library grew. |

Switching modes triggers a smooth animated transition that preserves your spatial orientation.

> [!tip]
> Hierarchical mode is especially useful for notes that follow a tree-like structure (e.g., MOCs linking to subtopics). Temporal mode reveals your intellectual timeline — when clusters of related notes were created.

---

## Focus mode

Focus mode shows only a specific note and its neighborhood. It is a dynamic, interactive local graph.

### Entering focus mode

- **Right-click a node** → **Focus**
- **Press Space** to toggle focus mode on the currently active note

### Focus controls

When in focus mode, a control bar appears at the top:

| Control | Description |
|---------|-------------|
| **Depth slider** (1–5) | How many hops of connections to show. 1 = direct links only, 5 = five levels deep. |
| **Direction filter** (↔ / ← / →) | Show all links, incoming only, or outgoing only. |
| **Exit button** (×) | Return to the full Sky View |

### Navigation breadcrumb

As you click through nodes in focus mode, a breadcrumb trail appears at the top showing your navigation path. Click any breadcrumb to jump back to that note's local graph.

> [!tip]
> Combine focus mode with the depth slider to progressively explore a note's neighborhood. Start at depth 1 to see direct connections, then increase to discover second and third-degree relationships.

---

## Search-to-highlight

Press `Ctrl+F` to open the search bar. Type a query to highlight matching notes.

Unlike a filter, search-to-highlight **dims** non-matching nodes without removing them. You retain the full graph structure and spatial context while the matching nodes are highlighted.

> [!tip]
> Search works in both the full graph and focus mode. You can search while in 3D mode as well.

---

## Settings panel

Click the gear icon (⚙) in the toolbar to open the settings panel. It has three tabs:

### Graph Appearance

| Control | Description | Default |
|---------|-------------|---------|
| **Node size** | Scale all nodes larger or smaller | 1.5 |
| **Label visibility** | When labels appear: On hover, Always, or None | On hover |
| **Label font size** | Size of note name labels | 12 |
| **Link thickness** | Width of edge lines | 1 |
| **Show orphan notes** | Include notes with no links | On |

### Physics

| Control | Description | Default |
|---------|-------------|---------|
| **Repulsion** | How strongly nodes push apart | 50 |
| **Link force** | How strongly linked nodes attract | 0.05 |
| **Link distance** | Target distance between linked nodes | 30 |
| **Reheat simulation** | Restart the force layout from the current state | — |

### AI

Settings for semantic AI links (Phase 2 — requires local embedding model).

| Control | Description |
|---------|-------------|
| **Show semantic links** | Toggle AI-detected dashed edges |
| **Confidence threshold** | Slider to filter semantic links by similarity score |

---

## Legend

The legend appears in the bottom-right corner and shows color assignments for your libraries.

### Color mode toggle

Click **Library** or **Folder** buttons at the top of the legend to switch how nodes are colored:

| Mode | Coloring |
|------|----------|
| **Library** | Each library gets a unique color |
| **Folder** | Each top-level folder gets a unique color |

### Visibility checkboxes

Each legend entry has a checkbox. Uncheck a library or folder to hide its nodes from the graph. This lets you focus on specific subsets of your knowledge base.

> [!tip]
> When in Folder mode, the folder count is shown in parentheses. Long folder lists are scrollable.

---

## Status bar

The bottom-left status bar shows:

- **Node count** — total visible nodes
- **Edge count** — total visible edges
- **MOC count** — number of Maps of Content (high-connectivity hub notes)
- **Hovered note name** — appears when you hover over a node

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+G` | Open Star View |
| `Escape` | Close Star View |
| `Ctrl+F` | Toggle search-to-highlight |
| `Ctrl+L` | Cycle layout mode (Organic → Hierarchical → Temporal) |
| `Space` | Toggle focus mode on active note |
| `0` | Reset 3D rotation to flat 2D |
| `W/A/S/D` | Fly through 3D space (when rotated) |
| `Q/E` | Move down/up in 3D space |

---

## RTL support

Star View provides first-class support for Arabic, Hebrew, and other RTL scripts:

- **Node labels** auto-detect script direction — Arabic titles render right-to-left
- **Legend items** flip dot/text order based on content language
- **Tooltips and panels** respect RTL layout
- **Arabic font fallback** — labels use system Arabic fonts (Noto Naskh Arabic, Segoe UI) when the primary font lacks Arabic glyph coverage

---

## Picture-in-Picture (PiP) overlay

When Star View is open and you click a child universe, library, or folder in the Notes Management sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay on top of the main graph.

### What the PiP shows

The PiP displays a filtered sub-graph containing only the nodes that belong to the selected scope. For example, clicking a library shows only that library's notes and their interconnections.

### PiP features

| Feature | Description |
|---------|-------------|
| **Filtered graph** | Only nodes from the selected scope appear |
| **Filtered legend** | The PiP has its own legend showing only the relevant entries |
| **Resizable** | Drag the edges or corners to resize the PiP window |
| **Repositionable** | Drag the title bar to move the PiP anywhere on screen |

### Cross-mode selection sync

Clicking a child universe, library, folder, or note in any sidebar mode (Tree, List, or OrgChart) highlights the corresponding nodes in the Star View graph. This bidirectional sync helps you maintain spatial awareness while browsing in the sidebar.

---

## Knowledge Strata

Star View automatically sizes nodes based on their knowledge level (1-8):

- Small dots: simple notes (Datum, Information)
- Medium nodes: connected notes (Proposition, Concept)
- Large glowing hubs: synthesis notes (Theory, Paradigm, Worldview)

Higher-level nodes have a complementary-colored glow halo for visual contrast. This activates when a library has 20+ notes.

---

## Note Maturity

Nodes display a colored ring indicating maturity:

- No ring: Seed (new note)
- Light green ring: Sapling (growing)
- Rich green ring: Evergreen (well-established)
- Gold ring: Canonical (authoritative reference)

Maturity is also shown in the file tree (left border) and tab bar (colored dot).

---

## Technical notes

Star View is powered by the **GraphMind** engine, a Pixi.js WebGL renderer with a d3-force simulation running in a dedicated Web Worker. This architecture ensures:

- **60fps rendering** even with thousands of nodes
- **Non-blocking layout** — force simulation never freezes the UI
- **Hover is visual-only** — hovering never triggers physics recalculation
- **The simulation stops after settling** — once nodes find their positions, the physics engine fully stops. Only dragging a node or changing settings restarts it.

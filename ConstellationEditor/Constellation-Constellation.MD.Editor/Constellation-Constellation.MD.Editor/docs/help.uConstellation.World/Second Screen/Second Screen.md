---
aliases:
  - Dual monitor
  - Second monitor
  - Second screen
  - Multi-window
  - Companion window
description: Use Constellation's Second Screen feature to expand your workspace across two monitors — browse and navigate on one screen, read and edit on the other.
---

# Second Screen

Second Screen opens a dedicated companion window that you can drag to a second monitor. Inspired by Adobe Lightroom's secondary display, it gives you three switchable view modes for maximum flexibility.

## Opening the Second Screen

| Method | Action |
|--------|--------|
| **Ribbon** | Click the monitor icon in the left ribbon |
| **Mission Control** | Press `Ctrl+P`, type "Second Screen" |
| **Keyboard shortcut** | `Ctrl+Shift+2` |

The second screen opens as an independent window. Drag it to your second monitor for a dual-screen workflow.

---

## View Modes

The second screen has three modes, switchable from the toolbar:

### Grid (G)

A searchable card grid of all notes across all libraries.

- Each card shows the note name, folder path, and library (with color indicator).
- **Search** to filter notes by name or path.
- **Library filter** dropdown to show notes from a specific library.
- **Sort** by name or library.
- **Click** a card to open the note in the main window.
- **Double-click** a card to open it in the second screen's Detail mode.

### Sky View (E)

The full interactive [[Sky View]] rendered in the second screen. Use it as a navigation tool — click any node to open the note in the main window. All sky view controls (filters, groups, display, forces) are available.

### Detail (D)

A focused note reader/editor with its own tab bar. Notes sent from the main window appear here. You get the full editing experience — properties, CodeMirror editor, live preview, markdown rendering — in a distraction-free window without sidebars.

---

## Linked Browsing

Toggle the **link icon** in the toolbar to enable or disable linked browsing.

| State | Behavior |
|-------|----------|
| **On** (default) | When the main window sends a note, the second screen automatically switches to Detail mode and shows it. |
| **Off** | Notes sent from the main window are opened silently in the second screen's tabs, but the current mode doesn't change. |

---

## Sending Notes Between Windows

| Action | Result |
|--------|--------|
| **Main → Second Screen** | Use Mission Control "Send to Second Screen" to send the active note. |
| **Second Screen → Main** | Click a card in Grid mode or a node in Sky View mode — the note opens in the main window. |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+2` | Toggle second screen window |
| `G` | Switch to Grid mode (when second screen is focused) |
| `E` | Switch to Sky View mode (when second screen is focused) |
| `D` | Switch to Detail mode (when second screen is focused) |

---

## Workflow Examples

### Research workflow
- **Main window:** Editor with your current draft open.
- **Second screen (Grid):** Browse all reference notes. Click to open them in the main editor.

### Sky View exploration
- **Main window:** Read/edit notes as you go.
- **Second screen (Sky View):** Navigate the knowledge sky view. Click nodes to open them in the main editor.

### Side-by-side reading
- **Main window:** Edit one note.
- **Second screen (Detail):** Read a reference note for comparison.

---

## Single-Monitor Users

Don't have a second monitor? The same NoteGrid and SkyView panels are available directly in the **right sidebar** of the main window.

| Sidebar Tab | Description |
|-------------|-------------|
| **Grid** (grid icon) | Searchable card grid of all notes — same as second screen's Grid mode |
| **Sky View** (network icon) | Interactive link sky view — same as second screen's Sky View mode |

Open the right sidebar with the sidebar toggle button, then click the Grid or Sky View tab icon.

> [!tip]
> These sidebar panels work alongside the existing Properties, Backlinks, Tags, and Link Dashboard tabs. You can switch between all six tabs freely.

---

## Workspace Integration

Workspaces now save and restore the **complete UI state**, including the second screen.

### What gets saved

| State | Details |
|-------|---------|
| **Editor tabs** | Open notes, active tab, split view |
| **Left sidebar** | Open/closed, width |
| **Right sidebar** | Open/closed, active tab, width |
| **Second screen** | Open/closed, current mode (Grid/Sky View/Detail), linked browsing state, open tabs |

### How it works

1. Open the Workspace Manager from Mission Control or sidebar.
2. **Save** a workspace — the full UI layout including second screen state is captured.
3. **Restore** a workspace — sidebars, tabs, and the second screen are all restored to their saved state.

> [!tip]
> If you restore a workspace that had the second screen open, it will automatically open and switch to the saved mode with the saved tabs.

> [!warning]
> Workspaces saved before this update will still restore correctly — only tabs and split state are restored, and the new sidebar/screen fields are ignored if absent.

---

## RTL Support

The Second Screen fully supports right-to-left (RTL) languages including Arabic, Hebrew, Persian, and Urdu. All modes — Grid, Sky View, and Detail — render correctly in RTL.

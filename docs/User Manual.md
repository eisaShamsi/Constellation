# Constellation User Manual

**Version 0.3.4 | March 2026**

Constellation is a Personal Knowledge Management (PKM) desktop application for managing Markdown note libraries. Built with Tauri v2, SvelteKit, and Rust, it runs natively on Windows, macOS, and Linux with full Arabic and RTL support.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Universe and Libraries](#universe-and-libraries)
3. [Creating and Editing Notes](#creating-and-editing-notes)
4. [Star View (GraphMind)](#star-view-graphmind)
5. [Second Screen](#second-screen)
6. [Properties and Frontmatter](#properties-and-frontmatter)
7. [Templates](#templates)
8. [Tables](#tables)
9. [Tasks](#tasks)
10. [Importer](#importer)
11. [Calendar](#calendar)
12. [Lens](#lens)
13. [Settings](#settings)
14. [Keyboard Shortcuts](#keyboard-shortcuts)
15. [RTL and Arabic Support](#rtl-and-arabic-support)
16. [Security and Privacy](#security-and-privacy)

---

## 1. Getting Started

### Installation

Download the latest installer from the [Constellation releases page](https://github.com/eisaShamsi/Constellation/releases):

- **Windows**: `.exe` (NSIS) or `.msi` installer
- **macOS**: `.dmg` disk image
- **Linux**: `.AppImage` or `.deb` package

### First Launch

When you first open Constellation, the **Universe Setup Wizard** guides you through:

1. **Choose your language** — 15 languages supported
2. **Create or import a library** — point to an existing folder of Markdown files, or start fresh
3. **Name your universe** — the universe is the container for all your libraries

### Interface Overview

| Element | Description |
|---------|-------------|
| **Sidebar (Ribbon)** | Navigation buttons: File tree, Search, Star View, Calendar, Templates, Settings |
| **File Tree** | Browse notes and folders within your libraries |
| **Editor** | Read and edit your Markdown notes |
| **Tab Bar** | Open multiple notes in tabs |
| **Status Bar** | Word count, character count, reading time |

---

## 2. Universe and Libraries

### What is a Universe?

A **Universe** is the top-level container that holds all your libraries. Think of it as your workspace or vault collection.

### What is a Library?

A **Library** is a folder on your computer containing Markdown (`.md`) files. You can have multiple libraries in one universe — for example, one for work notes and one for personal notes.

### Managing Libraries

- **Add a library**: Settings > Libraries > Add Library, or drag a folder into the app
- **Remove a library**: Settings > Libraries > click the remove button next to the library name
- **Library settings**: Each library can have its own appearance settings (fonts, colors)

### Child Universes

You can nest universes inside universes. A **Child Universe** is another universe folder referenced by your parent universe. Notes from child universes appear in Star View alongside your own notes, with cross-library links shown as dashed lines.

---

## 3. Creating and Editing Notes

### Creating a Note

| Method | Action |
|--------|--------|
| **Keyboard** | `Ctrl+N` |
| **File Tree** | Right-click a folder > New Note |
| **Mission Control** | `Ctrl+P` > "New note" |

### Editor Modes

Constellation offers two editor modes, selectable in **Settings > Editor > Editor type**:

#### Markdown Editor (CodeMirror)

The default editor for power users. Write Markdown directly with:

- **Live Preview** — renders formatting inline while you type
- **Source Mode** — shows raw Markdown syntax
- **Formatting toolbar** — appears on text selection
- **Slash commands** — type `/` for quick insertions
- **Wikilink autocomplete** — type `[[` to link notes
- **Multiple cursors** — `Alt+Click` or `Ctrl+D`

#### Document Editor (TipTap)

A WYSIWYG word-processor experience with a visual toolbar:

- Bold, Italic, Underline, Strikethrough, Highlight
- Headings (H1–H3), Text alignment
- Bullet lists, Numbered lists, Task lists
- Blockquotes, Code blocks, Horizontal rules
- Tables (insert, add/remove rows and columns)
- Links and Images

Both editors save as standard Markdown files. You can switch between them at any time without data loss.

### Text Formatting Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+Shift+S` | Strikethrough |
| `Ctrl+Shift+H` | Highlight |
| `Ctrl+K` | Insert wikilink |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |

### Linking Notes

Type `[[` to open the note autocomplete. Start typing a note name and select from suggestions. Links appear as clickable wikilinks: `[[Note Name]]`.

You can also link to specific headings: `[[Note Name#Heading]]`.

---

## 4. Star View (GraphMind)

Star View visualizes your notes as an interactive 3D graph powered by the **GraphMind** engine (Pixi.js WebGL).

### Opening Star View

- Click the graph icon in the sidebar
- Press `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Star View"

### Navigation

| Input | Action |
|-------|--------|
| **Click + drag** | Pan the graph |
| **Scroll** | Zoom in/out |
| **Click a node** | Open the note |
| **Right-click a node** | Context menu (Open, Focus, Pin, Hide) |
| **Middle-click drag** | Rotate in 3D |
| **W/A/S/D** | Fly through 3D space |
| **0** | Reset rotation to 2D |
| **Ctrl+F** | Search and highlight |
| **Space** | Toggle focus mode |

### Layout Modes

Press `Ctrl+L` to cycle between:

- **Organic** — force-directed layout where clusters emerge naturally
- **Hierarchical** — top-down tree layout
- **Temporal** — notes arranged by creation date on a timeline

### Focus Mode

Right-click a node > **Focus** to see only its neighborhood. Adjust:

- **Depth** (1–5 hops) — how many levels of connections to show
- **Direction** (↔/←/→) — all links, incoming only, or outgoing only

### 3D Navigation

Middle-click and drag to rotate. Use W/A/S/D/Q/E to fly through the star field. An XYZ axis gizmo in the corner shows your orientation. Press `0` to reset.

### Settings

Click the gear icon for:

- **Appearance**: Node size, label visibility, font size, link thickness, show orphans
- **Physics**: Repulsion force, link force, link distance
- **AI**: Semantic link threshold (Phase 2)

### Legend

Bottom-right legend shows library/folder colors with checkboxes to toggle visibility.

---

## 5. Second Screen

Open a separate window for side-by-side note viewing.

- **Open**: Click the second screen icon in the sidebar, or `Ctrl+Shift+N`
- **Sync**: Notes open in the second screen independently. Font and theme settings apply to both windows.
- **Note width**: Adjustable via the width slider in the toolbar

---

## 6. Properties and Frontmatter

Notes can have YAML frontmatter at the top:

```yaml
---
tags: [project, active]
date: 2026-03-19
status: in-progress
---
```

Constellation detects property types automatically:

| Type | Example |
|------|---------|
| **Text** | `author: John` |
| **Number** | `priority: 5` |
| **Date** | `date: 2026-03-19` |
| **List** | `tags: [a, b, c]` |
| **Checkbox** | `done: true` |
| **Link** | `related: [[Other Note]]` |

Toggle property display in **Settings > Editor > Properties in document** (Visible / Hidden / Source).

---

## 7. Templates

Create reusable note templates:

1. Create a folder for templates in your library
2. Set the template folder path in **Settings > Templates**
3. When creating a new note, choose a template from the template picker

Templates support variables:

| Variable | Replaced with |
|----------|---------------|
| `{{date}}` | Current date |
| `{{time}}` | Current time |
| `{{title}}` | Note title |
| `{{clipboard}}` | Clipboard contents |

---

## 8. Tables

### Markdown Tables

Type a Markdown table manually or use the `/table` slash command:

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

### Table Toolbar

When your cursor is inside a table, a floating toolbar appears with:

- Add/remove rows and columns
- Align columns (left, center, right)
- Navigate cells with `Tab` / `Shift+Tab`

### Tables in Document Editor

The Document editor (TipTap) provides a visual table experience:

- Click the table button to insert
- Use the dropdown for row/column management
- Resize columns by dragging borders

---

## 9. Tasks

Constellation supports task checkboxes in notes:

```markdown
- [ ] Incomplete task
- [x] Completed task
```

In Live Preview mode, checkboxes are clickable. Tasks can be searched and filtered across your libraries.

---

## 10. Importer

Import notes from other PKM tools:

- **Obsidian** — imports vaults with full wikilink compatibility
- **Markdown folders** — import any folder of `.md` files
- **Other formats** — HTML, text files

Go to **Settings > Importer** to start an import.

---

## 11. Calendar

The Calendar view shows notes organized by date:

- Notes with a `date` property appear on their respective days
- Daily notes can be created for any date
- Navigate months with arrow buttons

Open the Calendar from the sidebar.

---

## 12. Lens

Lens provides filtered views of your notes:

- Filter by tags, folders, properties
- Sort by name, date, or custom properties
- Save lens configurations for quick access

---

## 13. Settings

Access Settings from the sidebar gear icon or `Ctrl+,`.

### General

- Language (15 languages)
- Theme (Light / Dark)
- Interface font, Text font, Mono font, Font size

### Editor

- Editor type (Markdown / Document)
- Default view (Reading / Editing)
- Live Preview mode
- Line numbers, Indentation guides, Spellcheck
- Auto-pair brackets, Smart lists

### Libraries

- Add/remove libraries
- Per-library appearance settings
- Attachment folder location

### Updates

- Check for updates
- GitHub token for private repo updates

---

## 14. Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New note |
| `Ctrl+O` | Star Jump (quick open) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Open Star View |
| `Ctrl+,` | Settings |
| `Ctrl+Shift+F` | Search library |
| `Ctrl+Shift+N` | Second screen |

### Editor

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+K` | Insert wikilink |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+D` | Select next occurrence |
| `Ctrl+/` | Toggle comment |
| `Tab` | Indent / next table cell |

### Star View

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Search-to-highlight |
| `Ctrl+L` | Cycle layout mode |
| `Space` | Toggle focus mode |
| `0` | Reset 3D rotation |
| `W/A/S/D/Q/E` | Fly through 3D |
| `Escape` | Close Star View |

---

## 15. RTL and Arabic Support

Constellation provides first-class support for Arabic, Hebrew, Persian, Urdu, and other RTL scripts:

- **Auto-detection**: Note direction is detected automatically from content
- **Interface**: Full RTL interface when Arabic/Hebrew language is selected
- **Editor**: RTL text editing with correct cursor movement and selection
- **Star View**: Arabic labels render right-to-left with proper font fallback
- **Legend**: Items flip dot/text order based on content language
- **Script fonts**: Configure Arabic, Hebrew, and CJK fonts independently in Settings

### Setting Up for Arabic

1. Go to **Settings > General > Language** and select Arabic
2. Optionally set a dedicated Arabic font in **Settings > General > Script fonts**
3. Notes with Arabic content will automatically render RTL

---

## 16. Security and Privacy

- **All data stays local** — no cloud sync, no telemetry, no tracking
- **Markdown files** — your notes are plain text files you own completely
- **No account required** — Constellation works entirely offline
- **Optional updates** — check for updates manually via Settings
- **Open source** — inspect the code at [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

*Constellation User Manual — Version 0.3.4 — March 2026*
*uconstellation.world*

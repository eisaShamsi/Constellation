# Constellation User Manual

**Version 0.3.4 | March 2026**

Constellation is a Personal Knowledge Management (PKM) desktop application for managing Markdown note libraries. Built with Tauri v2, SvelteKit, and Rust, it runs natively on Windows, macOS, and Linux with full Arabic and RTL support.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Universe and Libraries](#universe-and-libraries)
3. [Creating and Editing Notes](#creating-and-editing-notes)
4. [Notes Management Sidebar](#notes-management-sidebar)
5. [Star View (GraphMind)](#star-view-graphmind)
6. [Second Screen](#second-screen)
7. [Properties and Frontmatter](#properties-and-frontmatter)
8. [Templates](#templates)
9. [Tables](#tables)
10. [Tasks](#tasks)
11. [Importer](#importer)
12. [Calendar](#calendar)
13. [Lens](#lens)
14. [Settings](#settings)
15. [Keyboard Shortcuts](#keyboard-shortcuts)
16. [RTL and Arabic Support](#rtl-and-arabic-support)
17. [Security and Privacy](#security-and-privacy)
18. [Cognitive Engine](#cognitive-engine)

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
| **Sidebar (Ribbon)** | Navigation buttons: Notes Management, Search, Calendar, Templates, Settings |
| **Notes Management** | Unified sidebar with mode tabs: Tree (File Explorer), List (Notes Navigator), OrgChart (Sky View) |
| **Editor** | Read and edit your Markdown notes |
| **Tab Bar** | Open multiple notes in tabs |
| **Status Bar** | Word count, character count, reading time |

---

## 2. Universe and Libraries

### What is a Universe?

A **Universe** is the top-level container that holds all your libraries. Think of it as your workspace or library collection.

### What is a Library?

A **Library** is a folder on your computer containing Markdown (`.md`) files. You can have multiple libraries in one universe — for example, one for work notes and one for personal notes.

### Managing Libraries

- **Add a library**: Settings > Libraries > Add Library, or drag a folder into the app
- **Remove a library**: Settings > Libraries > click the remove button next to the library name
- **Library settings**: Each library can have its own appearance settings (fonts, colors)

### Universe Notes Folder

Every universe automatically gets a **Universe Notes** folder at its root, named after the universe. This folder holds cross-library notes — MOCs (Maps of Content), dashboards, indexes, and any notes that don't belong to a single library.

- Appears at the top of the File Explorer, above child universes and libraries
- Included in search, Star View, and all features
- When creating a new note (`Ctrl+N`), the Universe Notes folder appears as the first option

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

### Editor

Constellation uses a high-performance CodeMirror 6 editor with two modes, switchable via the toggle in the breadcrumb bar:

#### Live Preview (default)

Renders Markdown formatting inline while you type — bold text appears bold, headings render as headings, links become clickable. Click on formatted text to reveal the Markdown syntax for editing.

#### Source Mode

Shows raw Markdown syntax for full control. Ideal for power users who prefer to see and type Markdown directly.

Both modes share these features:

- **Persistent toolbar** — formatting buttons always visible in Google Docs-style order (Bold, Italic, Underline, Headings, Lists, Tables, etc.)
- **Floating toolbar** — appears on text selection (can be disabled in Settings > Editor)
- **Right-click context menu** — full formatting options in a contextual menu
- **Find & Replace** — `Ctrl+F` to find, `Ctrl+H` to find and replace
- **Slash commands** — type `/` for quick insertions
- **Wikilink autocomplete** — type `[[` to link notes
- **RTL support** — toolbar icons flip for RTL content, layout adapts to text direction
- **Font sets** — per-language font customization via Settings > Language
- **Script toolbars** — language-specific symbol and punctuation toolbars (Arabic symbols, Hebrew, CJK punctuation, etc.)
- **Tag autocomplete** — type `#` to search and insert tags
- **Tashkeel highlighting** — optional Arabic diacritics highlighting toggle in the toolbar

### Callouts

Create styled callout blocks for notes, warnings, tips, and other admonitions:

```markdown
> [!note] Important information
> The content of the callout goes here.

> [!warning] Be careful
> This action cannot be undone.

> [!tip]- Click to expand
> Collapsible callout content.
```

Supported types: `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Each type has a distinct color and icon. Add `-` after the type to make it collapsible (starts collapsed), or `+` (starts expanded).

### Highlight Syntax

Wrap text in double equals to highlight it:

```markdown
This is ==highlighted text== in your note.
```

In Live Preview, the `==` marks are hidden and the text appears with a yellow background.

### Code Blocks

Fenced code blocks display with a background color and language label:

````markdown
```javascript
const greeting = "Hello, world!";
```
````

The language name appears as a badge above the code block.

### Image Embeds

Embed images inline in your notes:

```markdown
![Alt text](https://example.com/image.png)   — external URL
![[photo.jpg]]                                 — local file from library
```

In Live Preview, images render inline. Local images must be in your library folder. External images require an internet connection.

### Table Toolbar

When your cursor is inside a markdown table, a floating toolbar appears with:

- **+ Row / + Col** — add rows or columns
- **- Row / - Col** — remove rows or columns
- **Alignment** — left, center, right alignment per column
- **Sort** — sort rows ascending or descending by the current column
- **Tab / Shift+Tab** — navigate between table cells

### Toolbar Toggle

The toolbar has a toggle button (≡) as its first item. Click to show/hide all toolbar buttons. When hidden, only the toggle remains visible.

### Text Alignment

Three alignment buttons in the toolbar: Align Start, Align Center, Align End.

- In LTR: Start = left, End = right
- In RTL: Start = right, End = left (buttons adapt automatically)
- Alignment wraps the line in `<div style="text-align: ...">` — visible in Live Preview as aligned text

### Additional Formatting

- **Underline** — wraps text in `<u>...</u>`, renders underlined in Live Preview
- **Subscript** — wraps in `<sub>...</sub>`, renders as subscript
- **Superscript** — wraps in `<sup>...</sup>`, renders as superscript
- **Clear Formatting** — strips all markdown and HTML formatting marks from selected text
- **Find & Replace** — opens the CodeMirror search panel (`Ctrl+F`)

### Text Formatting Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+Shift+S` | Strikethrough |
| `Ctrl+Shift+H` | Highlight |
| `Ctrl+K` | Insert wikilink |
| `Ctrl+F` | Find |
| `Ctrl+H` | Find & Replace |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |

### Linking Notes

Type `[[` to open the note autocomplete. Start typing a note name and select from suggestions. Links appear as clickable wikilinks: `[[Note Name]]`.

You can also link to specific headings: `[[Note Name#Heading]]`.

---

## 4. Notes Management Sidebar

The Notes Management sidebar unifies three browsing modes into a single panel, replacing the separate File Explorer, Notebook Navigator, and Organization Chart (Sky View) with a tabbed interface.

### Elements Toolbar

The top row of the sidebar always shows the **Elements toolbar** with quick-action buttons:

| Button | Action |
|--------|--------|
| **New Note** | Create a new note in the selected folder |
| **New Base** | Create a new base (structured data note) |
| **New Folder** | Create a new folder in the selected library |

### Mode Tabs

The second row contains three mode tabs to switch how your notes are displayed:

| Tab | Icon | Description |
|-----|------|-------------|
| **Tree** | Folder tree icon | Classic File Explorer — browse your libraries as a folder hierarchy |
| **List** | List icon | Notes Navigator — dual-pane file browser with folder, tag, and property browsing |
| **OrgChart** | Tree diagram icon | Sky View — interactive hierarchy tree visualization |

Click a tab to switch modes. Your selection and scroll position are preserved when switching back.

### Adaptive Sidebar Width

The sidebar automatically adjusts its width to fit the longest library or child universe name visible in the current view. This ensures all names are readable without manual resizing.

### Child Universe Grouping

Across all three modes, content is organized with consistent grouping:

1. **Child universes first** — each child universe appears as a collapsible group with its libraries nested inside
2. **Own libraries below** — the parent universe's own libraries appear below a visual separator

This grouping is consistent across Tree, List, and OrgChart modes.

### Cross-Mode Selection Sync

Clicking a child universe, library, folder, or note in any sidebar mode highlights the corresponding nodes in the Star View graph. This bidirectional sync helps you maintain spatial awareness as you browse your knowledge base.

### Picture-in-Picture (PiP) Overlay

When Star View is open and you click a child universe, library, or folder in the sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay. The PiP shows a filtered sub-graph containing only the nodes belonging to the selected scope, with its own legend showing only the relevant entries. You can resize and reposition the PiP window freely.

### Tree Mode (File Explorer)

The classic file tree for browsing notes and folders:

- Expand/collapse folders with click or arrow keys
- Right-click for context menu (New Note, New Folder, Rename, Delete)
- Drag and drop to move notes between folders

### List Mode (Notes Navigator)

A dual-pane browser for advanced note browsing:

| Pane | Content |
|------|---------|
| **Left pane** | Folder tree, tag browser, or property browser (switchable) |
| **Right pane** | Matching notes with title, preview snippet, tags, and date |

Sorting options: Name (alphabetical), Last modified, File size.

Batch operations: Select multiple files with checkboxes, then tag, move, or delete.

### OrgChart Mode (Sky View)

An interactive tree-list visualization of your entire knowledge base hierarchy:

- Click to expand/collapse branches
- Click a note to open it in the editor
- Supports folder, tag, MOC link, and parent-property hierarchy sources

---

## 5. Star View (GraphMind)

Star View visualizes your notes as an interactive 3D graph powered by the **GraphMind** engine (Pixi.js WebGL).

### Opening Star View

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

### Knowledge Strata (Cognitive Engine)

Constellation automatically classifies every note into an 8-level knowledge hierarchy based on structural signals — no manual tagging required.

| Level | Name | How it's determined |
|-------|------|-------------------|
| 1 | Datum | Short note (≤50 words), no links |
| 2 | Information | 50–200 words, few links |
| 3 | Proposition | 200+ words or 2+ links |
| 4 | Concept | 3+ outgoing links, has `generalizes` links |
| 5 | Principle | Has `causes` or `supports` typed links |
| 6 | Theory | 8+ outgoing links (Map of Content) |
| 7 | Paradigm | Referenced by 3+ high-level notes |
| 8 | Worldview | Highest centrality in the graph |

**Visual**: Higher-level notes appear as larger nodes with a complementary-colored glow halo. Notes below level 4 are small dots. This activates automatically when a library has 20+ notes.

### Note Maturity Lifecycle (Cognitive Engine)

Notes grow through 4 maturity states, computed from inbound links and file age:

| State | Visual | Conditions |
|-------|--------|-----------|
| 🌱 Seed | No indicator | New note, no inbound links |
| 🌿 Sapling | Light green border | 1–3 inbound links or 2+ days old |
| 🌳 Evergreen | Rich green border | 4+ inbound links, 7+ days old |
| ⭐ Canonical | Gold border | 10+ inbound links, untouched 30+ days |

**Decay**: An Evergreen note untouched for 90+ days enters a "wilting" state (dimmed border).

Maturity appears in three places:
- **File tree**: colored left border on note names
- **Star View**: colored ring around nodes
- **Tab bar**: small colored dot (●) before the note title

---

## 6. Second Screen

The Second Screen is a mode-based companion window that adapts to your current sidebar mode.

- **Open**: Click the second screen icon in the sidebar, or `Ctrl+Shift+2`
- **Auto-closes**: When you close the main window, the second screen closes automatically

### Mode-Based Companion

The second screen changes its content based on the active sidebar mode in the main window:

| Main Sidebar Mode | Second Screen Shows |
|---|---|
| **File Explorer** | Universe Dashboard — stats, library breakdown, child universes, tags, recently edited/opened notes |
| **Navigator** | Full Navigator view for browsing notes |
| **Sky View** | Sky View tree with directory structure |
| **Star View** | Star View companion with backlinks, forward links, tags, and local graph |

### Universe Dashboard (File Explorer Mode)

When the main window is in File Explorer mode, the second screen displays a dashboard with:

- **Stat cards** — Universe name, child universe count, total libraries, folders, and notes
- **Child Universes** — Each child universe with its linked libraries and folder/note counts
- **Libraries** — Each library with folder/note counts in color-coded stat boxes
- **Recently Edited** — Notes you modified in the current session (tracked when you save changes)
- **Recently Opened** — Notes you opened but did not edit in the current session
- **Tags** — All tags across libraries sorted by count; click a tag to see all notes using it

### Settings Sync

Theme, font, and **language** changes in Settings instantly propagate to the second screen — no restart needed.

---

## 7. Properties and Frontmatter

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

## 8. Templates

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

## 9. Tables

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

---

## 10. Tasks

Constellation supports task checkboxes in notes:

```markdown
- [ ] Incomplete task
- [x] Completed task
```

In Live Preview mode, checkboxes are clickable. Tasks can be searched and filtered across your libraries.

---

## 11. Importer

Import notes from other PKM tools:

- **Obsidian** — imports libraries with full wikilink compatibility
- **Markdown folders** — import any folder of `.md` files
- **Other formats** — HTML, text files

Go to **Settings > Importer** to start an import.

---

## 12. Calendar

The Calendar view shows notes organized by date:

- Notes with a `date` property appear on their respective days
- Daily notes can be created for any date
- Navigate months with arrow buttons

Open the Calendar from the sidebar.

---

## 13. Lens

Lens provides filtered views of your notes:

- Filter by tags, folders, properties
- Sort by name, date, or custom properties
- Save lens configurations for quick access

---

## 14. Settings

Access Settings from the sidebar gear icon or `Ctrl+,`.

### Dashboard

- Universe overview and statistics

### Appearance

- Color scheme (Light / Dark / System)
- Accent color
- Interface font size (11–18px)
- Note font size

### Language

A dedicated tab consolidating all language-related settings:

- **Interface language** — select from 15 supported languages
- **Writing languages** — set a Primary language and an optional Secondary language, each with its own font set
- **Font Mode** — Universal (one font for all) or Per-Language (separate fonts per writing language)
- **Custom Font Sets** — system font dropdown for interface, text, and mono fonts per language
- **Date & Numbers** — numeral style (Arabic 0-9 or Hindi numerals), per-language date format with a Contextual checkbox for direction-aware rendering
- **Script Tools** — language-specific symbol and punctuation toolbars (Arabic, Hebrew, CJK, etc.)
- **Font Theme** — choose between Default and Typewriter font themes. The Typewriter theme applies authentic pre-PC-era fonts for each script (Courier Prime for Latin, Noto Naskh Arabic for Arabic, Miriam Libre for Hebrew, PT Mono for Cyrillic, Tiro Devanagari Hindi for Hindi, and system CJK fonts for Chinese/Japanese/Korean)

### Editor

- Floating toolbar toggle (show/hide toolbar on text selection)
- Tashkeel highlight (Arabic diacritics highlighting)
- Line numbers, Indentation guides, Spellcheck
- Auto-pair brackets, Smart lists
- Properties in document (Visible / Hidden / Source)

### Libraries

- Add/remove libraries
- Per-library appearance settings
- Attachment folder location

### Updates

- Check for updates
- GitHub token for private repo updates

---

## 15. Keyboard Shortcuts

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
| `Ctrl+F` | Find |
| `Ctrl+H` | Find & Replace |
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

## 16. RTL and Arabic Support

Constellation provides first-class support for Arabic, Hebrew, Persian, Urdu, and other RTL scripts:

- **Auto-detection**: Note direction is detected automatically from content
- **Interface**: Full RTL interface when Arabic/Hebrew language is selected
- **Editor**: RTL text editing with correct cursor movement and selection
- **Star View**: Arabic labels render right-to-left with proper font fallback
- **Legend**: Items flip dot/text order based on content language
- **Script fonts**: Configure Arabic, Hebrew, and CJK fonts independently in Settings > Language
- **Script toolbars**: Language-specific symbol and punctuation buttons (Arabic symbols, Hebrew, CJK punctuation)
- **Tashkeel highlighting**: Toggle Arabic diacritics highlighting from the editor toolbar

### Setting Up for Arabic

1. Go to **Settings > Language** and select Arabic as your interface or writing language
2. Optionally set a dedicated Arabic font set in **Settings > Language > Custom Font Sets**
3. Enable Script Tools for Arabic symbol toolbar access
4. Notes with Arabic content will automatically render RTL

---

## 17. Security and Privacy

- **All data stays local** — no cloud sync, no telemetry, no tracking
- **Markdown files** — your notes are plain text files you own completely
- **No account required** — Constellation works entirely offline
- **Optional updates** — check for updates manually via Settings
- **Open source** — inspect the code at [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 18. Cognitive Engine

The Cognitive Engine is a two-layer architecture that transforms Constellation from a note-taking app into a knowledge cognition instrument.

**Layer 1 — Structural Cognition** (zero AI dependency): Tools that analyze your notes' structure, connections, and metadata to surface insights. Works fully offline.

**Layer 2 — AI Discovery** (coming soon): AI reads Layer 1's structures to find patterns you cannot see from inside your own knowledge.

### Currently Available (Layer 1)

| Feature | What it does |
|---------|-------------|
| **Typed Links** | Add semantic meaning to links: `[[note\|supports]]`, `[[note\|contradicts]]`, etc. 7 link types with distinct colors in Star View |
| **Knowledge Strata** | Auto-classifies notes into 8 levels (Datum → Worldview) based on word count, link count, and link types |
| **Maturity Lifecycle** | Tracks note growth: Seed → Sapling → Evergreen → Canonical. Shown in file tree, Star View, and tab bar |

These features require no configuration — they activate automatically as your library grows.

---

*Constellation User Manual — Version 0.3.4 — March 2026*
*uconstellation.world*

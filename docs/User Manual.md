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

### Auto-Reopen

Constellation remembers your last active universe and reopens it automatically on launch. If the universe was moved or its path changed, Constellation detects and heals the path automatically.

### Portable Universes

Constellation universes are fully portable. You can move a universe folder to any location — a different drive, USB stick, or another computer — and Constellation will automatically detect and fix all internal paths when you reopen it.

To move a universe:
1. Close Constellation
2. Move or copy the universe folder to the new location
3. Open Constellation → it shows the Welcome screen (old path no longer valid)
4. Choose **Open Existing Universe** and point to the new location
5. All notes and libraries appear immediately — paths are auto-fixed

The universe folder structure follows the Obsidian model: notes go directly in the root folder, configuration lives in `.constellation/`.

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

> "The quantity of your data and information doesn't matter. It is NOT about how many references or sources you keep or store; it is about how you formulate your KNOWLEDGE from them, and how to link all of it into one meaningful awareness."

The Cognitive Engine is a two-layer architecture that transforms Constellation from a note-taking app into a knowledge cognition instrument. Most note apps help you store and retrieve information. The Cognitive Engine goes further: it helps you understand what your knowledge actually means, where it comes from, how mature it is, and where the gaps lie.

**Layer 1 — Structural Cognition** (zero AI dependency): Nine tools that analyze your notes' structure, connections, and metadata to surface insights. Everything runs locally on your machine, fully offline, with no AI dependency. The engine reads the shape of your library — word counts, link counts, link types, and graph topology — to tell you things about your knowledge that you cannot easily see yourself.

**Layer 2 — AI Discovery** (coming soon): AI will read Layer 1's structures to find patterns you cannot see from inside your own knowledge.

All nine Cognitive Engine features require no configuration. They activate automatically as your library grows. You do not need to enable them or install anything extra.

---

### 18.1 Typed Links

**What it is**

Typed Links let you add semantic meaning to the connections between your notes. Instead of a plain link like `[[Climate Change]]` that only says "these two notes are related somehow," a Typed Link says exactly how they are related: `[[Climate Change|type:supports]]` means "this note provides evidence for the claims in Climate Change." Constellation supports seven link types, each with a distinct color in Star View.

**Why it matters for knowledge**

A library full of plain links is like a map with roads but no signs. You can see that places are connected, but you cannot tell whether a road goes uphill or downhill, whether it carries agreement or disagreement. Typed Links turn your library from a web of vague associations into a network of explicit reasoning. When you mark one note as supporting another, or contradicting it, you are doing the real work of knowledge: deciding what relates to what, and how.

**How to use it**

1. Open a note in the editor (NotePane or FocusPane).
2. Type `[[` to begin a wiki-link, then type the name of the target note.
3. After the note name, type `|type:` — an autocomplete menu will appear showing all seven link types.
4. Select the type that describes the relationship. The full syntax is:
   - `[[Target Note|type:supports]]` — this note provides evidence for Target Note
   - `[[Target Note|type:contradicts]]` — this note disagrees with or challenges Target Note
   - `[[Target Note|type:causes]]` — this note describes a cause of what Target Note describes
   - `[[Target Note|type:exemplifies]]` — this note is a concrete instance of Target Note's concept
   - `[[Target Note|type:generalizes]]` — this note abstracts or broadens Target Note's idea
   - `[[Target Note|type:derives-from]]` — this note's knowledge originates from Target Note
   - `[[Target Note|type:part-of]]` — this note is a component or subsection of Target Note
5. Press Enter to confirm.

**Where you see it**

- **In the editor**: Typed links render as colored text with a tooltip showing the link type. Hover over any typed link to see its semantic label.
- **In Star View (GraphMind)**: Each link type has a distinct color, so you can visually trace chains of support, contradiction, or derivation across your entire library.
- **In the autocomplete menu**: When you type `|type:` inside a wiki-link, all seven types appear with short descriptions.

**Tips**

- You do not need to type every link. Start by typing the links that carry the strongest meaning — the ones where you know "this supports that" or "this contradicts that." Even a handful of typed links will light up your Star View.
- The `contradicts` type is especially powerful. It forces you to acknowledge tensions in your thinking, which is where real learning happens.
- You can always add a type to an existing link later. Just open the note, find the plain `[[link]]`, and add `|type:supports` (or whichever type applies).
- Typed Links feed into three other Cognitive Engine features: Knowledge Strata uses them to determine note depth, the Tension Detector watches for contradictions, and the Provenance Chain follows `derives-from` links.

---

### 18.2 Knowledge Strata

**What it is**

Knowledge Strata automatically classifies every note in your library into one of eight levels, from raw data to worldview. The classification is computed from structural signals — word count, link count, and which typed links a note uses — with no AI and no manual tagging. The eight levels are:

| Level | Name | What it means |
|-------|------|---------------|
| 1 | **Datum** | A brief note (50 words or fewer), no links. A raw fact or observation. |
| 2 | **Information** | A note of 50-200 words with 0-1 links. A single topic, lightly developed. |
| 3 | **Proposition** | A note with 200+ words or 2+ links. An argument or explanation taking shape. |
| 4 | **Concept** | Links to 3+ notes and uses `generalizes` links. An abstraction that unifies ideas. |
| 5 | **Principle** | Links to 3+ concepts and uses `causes` or `supports` links. A rule or pattern you have identified. |
| 6 | **Theory** | A Map of Content (8+ outgoing links) with many `part-of` inbound links. A structured framework. |
| 7 | **Paradigm** | Referenced by 3+ high-stratum notes with high centrality. A lens through which you see a domain. |
| 8 | **Worldview** | Highest centrality in your library, deepest `derives-from` chain root. Your most foundational belief. |

**Why it matters for knowledge**

Most people cannot tell you the difference between a fact they jotted down and a principle they have tested over years. Knowledge Strata makes that difference visible. When you see that 80% of your notes are at level 1-2 (raw data), you know you are collecting but not synthesizing. When you see a note climb from Proposition to Principle, you know your understanding of that topic is deepening. The strata are not a judgment — they are a mirror.

**How to use it**

1. Simply write notes and create links as you normally would. Knowledge Strata computes automatically.
2. To see a note's stratum, check the **right sidebar** — the stratum level appears in the note's properties section.
3. In **Star View**, nodes are sized and layered by stratum. Higher-stratum notes appear larger and more prominent.
4. To raise a note's stratum naturally:
   - Write more (expand from a short fact into a developed explanation).
   - Link it to other notes (connect it to the broader web of your knowledge).
   - Use Typed Links (adding `supports`, `generalizes`, or `causes` links signals deeper structural relationships).

**Where you see it**

- **Right sidebar**: The note's stratum level is shown in the properties area.
- **Star View (GraphMind)**: Node size reflects stratum. Datum notes appear as small dots; Worldview notes appear as large, prominent nodes.
- **Strata are recalculated** each time you open a library, so they always reflect the current state of your notes.

**Tips**

- Do not chase high strata for their own sake. A library of all level-8 notes would be meaningless. The value is in seeing the distribution — a healthy library has notes at every level.
- If you notice an important topic stuck at Datum or Information, that is a signal to develop it further: write more, link it to related ideas, explain why it matters.
- Strata reward genuine intellectual work. You cannot game the system by adding meaningless links — the engine checks for specific link types (`generalizes`, `supports`, `causes`) that indicate real conceptual relationships.

---

### 18.3 Maturity Lifecycle

**What it is**

The Maturity Lifecycle tracks how developed each note is over time. Every note begins as a **Seed** (a fresh idea, just planted) and can grow through four stages as you revisit, expand, and refine it:

| Stage | Meaning |
|-------|---------|
| **Seed** | A new note, recently created, still brief and unconnected. |
| **Sapling** | The note is growing — it has some content and a few links, but is still developing. |
| **Evergreen** | A well-developed note with substantial content, multiple links, and clear structure. Reliable reference material. |
| **Canonical** | Your most authoritative notes — deeply linked, frequently referenced by other notes, and representing your settled understanding of a topic. |

There is also a **Wilting** state for notes that were once active but have become disconnected or outdated.

**Why it matters for knowledge**

Ideas do not arrive fully formed. They start as fragments, grow through revision, and eventually become pillars of your understanding. The Maturity Lifecycle makes this growth visible. When you see a Seed, you know it needs attention. When you see an Evergreen, you know you can trust it as a reference. This is the difference between a pile of notes and a living, growing body of knowledge.

**How to use it**

1. Maturity is computed automatically based on word count, link count, creation date, and how frequently other notes reference it. You do not need to set it manually.
2. To grow a note's maturity:
   - **Seed to Sapling**: Add content (expand beyond a brief jotting) and create at least one link to another note.
   - **Sapling to Evergreen**: Develop the note into a substantial piece — add structure, link it to multiple related notes, and revisit it over time.
   - **Evergreen to Canonical**: This happens naturally when other notes in your library frequently link to it. A Canonical note is one that your own knowledge graph treats as a hub.

**Where you see it**

- **File tree**: Each note shows a small maturity indicator (icon or color) so you can scan your library and see which notes are Seeds, Saplings, Evergreens, or Canonical.
- **Star View (GraphMind)**: Maturity affects the visual appearance of nodes, helping you see at a glance which parts of your knowledge are well-developed and which are still germinating.
- **Tab bar**: The active note's maturity stage is visible in the tab, so you always know the state of the note you are editing.

**Tips**

- Treat Seeds as a to-do list for your thinking. Periodically review your Seeds and decide: develop this further, or let it go.
- Do not worry about Wilting notes. They may represent ideas you explored and moved past — that is a normal part of intellectual growth.
- The most valuable moment is when a Sapling becomes an Evergreen. That transition means you have taken a half-formed idea and turned it into a reliable piece of your knowledge base.

---

### 18.4 Tension Detector

**What it is**

The Tension Detector is a knowledge health monitor that scans your library for structural problems: contradictions between notes, orphan notes with no connections, structural gaps where clusters of notes lack bridges between them, and single points of failure where removing one note would disconnect an entire branch of your knowledge. It activates automatically once your library has at least 50 linked notes.

The Tension Detector identifies four types of issues:

| Issue | What it means |
|-------|---------------|
| **Contradictions** | Two notes linked with `contradicts` typed links. These are not errors — they are valuable tensions in your thinking that deserve attention. |
| **Orphan Notes** | Notes with zero incoming or outgoing links. They exist in isolation, disconnected from everything else you know. |
| **Structural Gaps** | Groups of notes that share a tag or topic but have no links between them. They should be connected but are not. |
| **Single Points of Failure** | A note that is the only bridge between two clusters. If you removed it, an entire section of your knowledge would become disconnected. |

**Why it matters for knowledge**

A library of notes can develop blind spots just like a person's thinking can. You might have two notes that directly contradict each other without realizing it. You might have an important idea sitting in isolation, never connected to the rest of your work. The Tension Detector surfaces these structural weaknesses so you can address them deliberately. Contradictions are especially valuable — acknowledging where your ideas conflict is often where the deepest learning happens.

**How to use it**

1. Open the **Tension** tab in the right sidebar (it appears as a panel alongside your note properties).
2. If your library has fewer than 50 linked notes, you will see a progress indicator showing how close you are to activation.
3. Once active, the panel shows four collapsible sections: Contradictions, Orphan Notes, Structural Gaps, and Single Points of Failure.
4. Each issue has a severity indicator (red for high, amber for medium, gray for low).
5. Click any item to open the relevant note directly.
6. To resolve issues:
   - **Contradictions**: Read both notes. Decide if the contradiction is genuine (keep both and explore the tension) or accidental (update one to be consistent).
   - **Orphans**: Link the orphan note to at least one related note, or decide it does not belong in your library.
   - **Structural Gaps**: Create links between notes that share a topic but are not yet connected.
   - **Single Points of Failure**: Add alternative connections so that no single note is the only bridge.

**Where you see it**

- **Right sidebar — Tension tab**: The main interface for the Tension Detector, showing all four issue types with counts and clickable items.
- **Star View**: Contradiction links appear in a distinct color, making tension lines visible in your knowledge graph.

**Tips**

- Do not try to eliminate all tensions. A library with zero contradictions might mean you are not thinking critically enough. The goal is awareness, not perfection.
- Check the Tension panel once a week as part of a review routine. It is like a health checkup for your knowledge.
- Orphan notes are the easiest issue to fix and often the most rewarding. A single link can integrate a forgotten idea back into your thinking.
- Single Points of Failure are the most dangerous structural issue. If a key bridging note is deleted or corrupted, you could lose the connection between two important areas of your knowledge.

---

### 18.5 Provenance Chain

**What it is**

The Provenance Chain traces where your knowledge comes from by following `derives-from` typed links. When you write a note that is based on a book, a lecture, a conversation, or another note, you can mark that relationship with `[[Source Note|type:derives-from]]`. The Provenance Chain follows these links backward to build a complete ancestry tree for any note, and classifies your knowledge as either **Received** (originating from external sources) or **Discovered** (your own original thinking).

**Why it matters for knowledge**

Knowing what you think is only half the picture. Knowing where your ideas came from — and whether they are truly yours or inherited — is what turns information into genuine understanding. The Provenance Chain makes this lineage visible. When you see that a note's ancestry traces back through three levels to an external source, you understand that it is received knowledge that you have processed and transformed. When a note has no external sources in its chain, you know it represents your own original synthesis. This distinction matters because received knowledge and discovered knowledge serve different roles in your thinking.

**How to use it**

1. When you create a note based on an external source (a book, article, lecture, or someone else's idea), add a `derives-from` link:
   - Example: `[[Thinking Fast and Slow|type:derives-from]]`
   - This tells Constellation that your current note's knowledge originates from that source.
2. You can chain derivations: Note C derives from Note B, which derives from Note A. The Provenance Chain will trace the entire lineage.
3. To mark a source as external (not part of your library), simply note it in the source note's content — the Provenance Chain will recognize sources that have no further `derives-from` links as root sources.
4. Open the **Provenance** tab in the right sidebar to see:
   - The ancestry tree of the current note, showing every source in the chain.
   - The **depth** of the chain (how many levels back the derivation goes).
   - The **origin classification**: Received, Discovered, or Mixed.

**Where you see it**

- **Right sidebar — Provenance tab**: Shows the full ancestry tree for the currently open note, with depth count and origin classification (Received / Discovered / Mixed).
- **Star View (GraphMind)**: `derives-from` links appear as distinct edges, so you can visually trace provenance chains across your library.
- **Note properties**: The origin type (Received / Discovered / Mixed) appears in the note's metadata in the right sidebar.

**Tips**

- You do not need to add `derives-from` links to every note. Focus on the notes where source attribution matters — where you want to remember "this idea came from that book" or "this argument builds on that conversation."
- A note classified as "Discovered" is not necessarily better than "Received." The most powerful knowledge often comes from deeply processing received ideas until they become your own. The classification helps you see the balance.
- If the Provenance tab shows "No derives-from chain found," it means the current note has no provenance links yet. The panel will display a hint reminding you of the syntax.
- Provenance Chains are especially valuable for academic work, research projects, or any context where you need to trace an idea back to its original source.

### 18.6 Externalization Engine

**What it is**

A progressive formalization pipeline that tracks how your notes mature from raw captures to crystallized insights. Every note can be assigned one of four stages:

| Stage | Icon | Meaning |
|-------|------|---------|
| Fleeting | 🌱 | Quick capture, passing thought |
| Literature | 📖 | Rewritten from a source in your own words |
| Permanent | 🔗 | Atomic idea, one concept, connected to your graph |
| Synthesis | ✨ | Original insight combining multiple permanent notes |

**Why it matters**

Most apps treat all notes equally. The Externalization Engine makes the distinction visible — you can see at a glance how much of your library is raw capture versus genuine understanding.

**How to use it**

1. In the breadcrumb bar (above the editor), use the stage dropdown to select a stage for the current note.
2. Or expand Properties and use the stage dropdown there. Both sync instantly with the file tree.
3. To promote a note, change the dropdown from one stage to the next. In Focus mode, click "Promote to Permanent" at the bottom.
4. To remove a stage, select "— Stage —" from the dropdown.

**Where you see it**

- **Breadcrumb bar**: A dropdown with emoji + stage name appears above the editor.
- **Properties panel**: A stage dropdown appears when the `stage` property exists on the note.
- **File tree**: An emoji icon appears next to the note name matching its stage.
- **Focus mode footer**: A "Promote to Permanent" button for quick stage advancement.

**Tips**

- Stages are completely optional — notes without a stage work normally.
- Start by marking your most important notes as Permanent or Synthesis.
- Use Fleeting for quick captures in Focus mode.
- The four stages follow the Zettelkasten progression: fleeting thoughts become literature notes, which become permanent atomic ideas, which combine into original synthesis.

---

### 18.7 Review Pulse

**What it is**

Review Pulse is a spaced resurfacing system that brings notes back to your attention at expanding intervals: 1 day, then 3, then 7, then 14, then 30 days after your last review. It also monitors notes tagged with `#assumption` or `#model` as mental model checkpoints and maintains a "Never Reviewed" queue for notes you captured but never revisited.

**Why it matters**

Knowledge decays without revisitation. You write a note today, and in three weeks you have forgotten it exists. Spaced repetition is the most well-established technique in cognitive science for fighting this decay. Review Pulse applies this principle to your actual notes — not flashcards, but the knowledge artifacts you created yourself. The mental model checkpoint feature ensures your foundational assumptions get regular inspection.

**How to use it**

1. Click the **Review Pulse** tab in the left sidebar. You will see three sections: Due for Review, Mental Model Checkpoints (`#assumption` / `#model` tagged notes), and Never Reviewed.
2. Click any note in the list to open it and read through it.
3. Choose one of three actions:
   - **Reviewed** (checkmark) — confirms you have re-read the note. The next review is scheduled at the next interval (1 → 3 → 7 → 14 → 30 days).
   - **Snooze 7d** (eye icon) — pushes the note back by 7 days without advancing the interval.
   - **Dismiss** (archive icon) — removes the note from the review queue entirely.
4. Open the Command Palette and type "Review due notes" to jump directly to due notes.

**Where you see it**

- **Left sidebar**: The Review Pulse tab with a badge count showing how many notes are due.
- **Command Palette**: "Review due notes" command for quick access.

**Tips**

- Make reviewing a daily habit. The intervals are designed so this never takes long.
- Tag your core beliefs and working assumptions with `#assumption` or `#model` so they appear in Mental Model Checkpoints.
- The Never Reviewed section surfaces notes you captured but never integrated into your thinking.

---

### 18.8 Trails

**What it is**

Trails are named, ordered sequences of notes — like chapters in a book or stops on a guided tour through your knowledge. A trail is defined by adding `trail: true` to a note's frontmatter, then listing wikilinks in order in the note body. Each note in a trail knows its position and provides navigation to the previous and next note.

**Why it matters**

Knowledge is not always a web. Sometimes it is a path — a learning sequence, an argument progression, a narrative. Trails let you capture that order explicitly, adding a linear dimension to your non-linear library.

**How to use it**

1. Create a new note with `trail: true` in the frontmatter:
   ```yaml
   ---
   trail: true
   ---
   ```
2. In the note body, list wikilinks in the order you want them followed:
   ```markdown
   1. [[First Note]]
   2. [[Second Note]]
   3. [[Third Note]]
   ```
3. When you open any note that belongs to a trail, the breadcrumb bar shows a trail indicator with the trail name and position (e.g., "My Trail 2/5"). Arrow buttons navigate to the previous and next note.
4. Open the Command Palette and type "Open Trail" to see all trails in your library.

**Where you see it**

- **Breadcrumb bar**: Trail indicator with name, position, and prev/next navigation arrows.
- **Command Palette**: "Open Trail" command lists all trails.

**Tips**

- Use trails for onboarding: create a "Start Here" trail that walks newcomers through your most important notes.
- Use trails for argument construction: lay out reasoning from premise to conclusion.
- A note can belong to multiple trails. The breadcrumb shows whichever trail you navigated from.

### 18.9 Multi-Lens Views

**What it is**

Multi-Lens Views let you view your library through different classification schemes without changing your folder structure or duplicating notes. A "lens" is a virtual grouping that reorganizes notes based on a property or tag. Built-in lenses: "By Stage" (groups by Fleeting/Literature/Permanent/Synthesis) and "By Topic" (groups by tags). You can create custom lenses in Settings.

**Why it matters**

Folder structures impose a single hierarchy, but knowledge does not fit one tree. Multi-Lens Views let you switch between perspectives without moving files. The same notes, viewed through different organizational lenses.

**How to use it**

1. In the sidebar, find the **lens dropdown** at the top of the file tree (defaults to "Folders").
2. Select a lens: "By Stage," "By Topic," or a custom lens. The sidebar reorganizes instantly.
3. Select "Folders" to return to the default file tree.
4. To create a custom lens: open **Settings > Knowledge Management**, click **Create Lens**, name it, and choose which frontmatter property to group by.
5. Or use the Command Palette: type "Create Lens" to create a lens directly.

**Where you see it**

- **Sidebar dropdown**: Lens selector at the top of the file tree.
- **Settings > Knowledge Management**: Create, edit, and delete custom lenses.
- **Command Palette**: "Create Lens" command.

**Tips**

- "Folders" is always available as the default file tree. Lenses are additive, not replacements.
- "By Stage" pairs with the Externalization Engine to show your formalization progress.
- "By Topic" is useful for large libraries where related notes are scattered across folders.
- Custom lenses can group by any frontmatter property: `project`, `status`, `priority`, etc.
- No notes are duplicated or moved. Lenses are purely virtual views.

---

*Constellation User Manual — Version 0.3.4 — March 2026*
*uconstellation.world*

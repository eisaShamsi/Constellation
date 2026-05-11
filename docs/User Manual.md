# Constellation User Manual

**Version 0.1.0 | March 2026**

Constellation is a Personal Knowledge Formulation desktop application. Not a file manager — a thinking instrument. Built with Tauri v2, SvelteKit, and Rust, it runs natively on Windows, macOS, and Linux with full multilingual support (15 languages, RTL-native).

---

## Table of Contents

1. [Knowledge Formulation](#knowledge-formulation)
2. [Getting Started](#getting-started)
3. [Universe and Libraries](#universe-and-libraries)
4. [Creating and Editing Notes](#creating-and-editing-notes)
4. [Notes Management Sidebar](#notes-management-sidebar)
5. [Search](#search)
6. [Sky View (GraphMind)](#star-view-graphmind)
6. [Split View](#split-view)
7. [Index](#index)
8. [Constellation Sight](#constellation-lens)
9. [Second Screen](#second-screen)
10. [Properties and Frontmatter](#properties-and-frontmatter)
10b. [Source Review (CECE)](#10b-source-review-constellation-epistemic-content-engine--cece)
11. [Templates](#templates)
12. [Tables](#tables)
13. [Tasks](#tasks)
14. [Importer](#importer)
15. [Calendar](#calendar)
16. [Lens (DQL Queries)](#lens)
16b. [Panels](#panels)
17. [Settings](#settings)
17. [Keyboard Shortcuts](#keyboard-shortcuts)
18. [RTL and Arabic Support](#rtl-and-arabic-support)
19. [Security and Privacy](#security-and-privacy)
20. [Constellation Map](#constellation-map)
21. [Cognitive Engine](#cognitive-engine)

---

## Knowledge Formulation

Constellation is built on a simple belief: **knowledge is not about storage — it's about formulation**. You don't become wiser by organizing files. You become wiser by connecting ideas, challenging assumptions, tracing origins, and synthesizing understanding.

### The Living Link

In Constellation, a link between two notes is not a dead pointer. It is a **living vessel** that carries meaning:

- **Type** — the kind of relationship (supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of).
- **Annotation** — the *why*. Your reasoning at the moment of linking, authored inline via `[[type::Target|your reasoning]]` and displayed in italic purple text under the link in Backlinks / Outgoing panels.
- **Weight** — how significant the connection is. Starts at 1.0, grows logarithmically with each traversal, and decays exponentially when neglected.
- **Confidence** — how certain you are. Four tiers (Hypothesis → Evidence → Established → Contested). Auto-promotes as you traverse; right-click any link to override.
- **Tier (visual)** — derived from traversal count: *emerging* (×1–2), *established* (×3–9), *load-bearing* (×10+), *stale* (≥90d untouched).
- **Archive** — every operation is reversible. Archived links are soft-deleted (hidden everywhere, preserved in history) and can be restored from the Link Dashboard.

Detailed step-by-step tutorials for every Living Link function — authoring, contesting, archiving, decay settings, the back-fill one-shot, and the Link Dashboard's seven tabs — live in the dedicated help file: [Knowledge Formulation](help.uConstellation.World/Knowledge%20Formulation/Knowledge%20Formulation.md).

### The Five Acts of Knowledge Creation

1. **Observation** — You capture something new (a note is born)
2. **Connection** — You link it to existing knowledge (the first heartbeat)
3. **Tension** — You discover a contradiction (critical thinking begins)
4. **Synthesis** — You resolve the tension with a new understanding (knowledge is created)
5. **Conviction** — Evidence accumulates over time (the idea becomes bedrock)

### Searching Your Thinking

Constellation's search engine is a diagnostic instrument for your intellectual life:

- `supports [[Democracy]]` — What evidence supports this idea?
- `contradicts [[My Thesis]]` — What challenges my thinking?
- `causes [[Event]]` — What led to this outcome?
- `orphans` — Which ideas are isolated and unconnected?

All operators work in your language — type in Arabic, French, Japanese, or any of 15 supported languages.

For the full specification, see `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`.

### Living Link Tutorials

Every Living Link function, step by step.

#### Tutorial 1 — Your first typed link
1. Open any note. Place cursor where you want the link.
2. Type: `[[supports::Mughal Empire]]`
3. Save. The link renders with a small **supports** badge in the Backlinks and Outgoing Links panels.

Recognized types: `supports`, `contradicts`, `causes`, `exemplifies`, `generalizes`, `derives-from`, `part-of`. Anything else is parsed as an untyped `relates` link.

#### Tutorial 2 — Adding an annotation (the "why")
1. Write a typed link as in Tutorial 1.
2. Add a single pipe `|` after the target, followed by your reasoning: `[[supports::Mughal Empire|Babur launched his 1526 invasion from Kabul]]`
3. Save.
4. Open the target note and look at the right sidebar → Backlinks tab. Under the context excerpt you'll see your annotation in italic purple quotes.

Rule: only one pipe per link. Write concise, future-proof reasoning ("timeline fits" beats "see above").

#### Tutorial 3 — Tier growth through traversal
Click a wikilink: it gets a `×1` chip. Click it 3 total times: the chip colors shift to *established*. Click 10+ times total: the chip becomes solid purple — **load-bearing**. 90+ days without traversal: the chip turns **amber (stale)**.

| Traversals | Tier | Visual |
|---|---|---|
| 1–2 | emerging | faint tint |
| 3–9 | established | stronger tint |
| 10+ | load-bearing | solid fill, white text |
| 90+ days idle | stale | amber |

#### Tutorial 4 — Tier vs Confidence
- **Tier** = earned passively from traversal count.
- **Confidence** = your epistemic stance (Hypothesis / Evidence / Established / Contested).

Auto-promotion: ×1–2 → Hypothesis, ×3–9 → Evidence, ×10+ → Established. User-set `Contested` is never overridden by the auto-rule.

#### Tutorial 5 — Contesting / force-promoting confidence
1. Right sidebar → **Backlinks** or **Outgoing Links**.
2. **Right-click** the link row (left-click navigates).
3. In the popover, pick one: Hypothesis / Evidence / Established / Contested. The current level is highlighted.
4. Change is saved immediately. Right-click again to verify.

#### Tutorial 6 — Archiving a link (soft delete)
1. Right-click the link row.
2. Below the four confidence options, click **Archive link**.
3. Row disappears from Backlinks / Outgoing / Most-Traveled / Stale. Traversal count and confidence are preserved.

#### Tutorial 7 — Restoring an archived link
1. Right sidebar → **Link Dashboard** (share-2 icon, last tab).
2. Click the **Archived** tab (rightmost).
3. Click the circular-arrow button at the end of the row.
4. Link returns to active status with weight reset to 1.0.

#### Tutorial 8 — One-shot confidence back-fill
Use this when you imported notes from elsewhere, or if links existed before auto-promotion shipped: they may have high traversal counts but stale confidence.

1. Settings → **Appearance** → **Living Link Lifecycle**.
2. Find **Back-fill link confidence** row.
3. Click **Run back-fill**.
4. Result appears in accent color: *Promoted N link(s) (→evidence: X, →established: Y).*

Safe to re-run. Never downgrades. Preserves `Contested`.

#### Tutorial 9 — Tuning decay (half-life)
Settings → **Appearance** → **Living Link Lifecycle**:

- **Apply weight decay to link sorts** — toggle off for raw counts only.
- **Decay half-life** slider — 7–365 days (default 60).

Guideline: 30 days (aggressive — "what's alive now?") · 60 days (balanced default) · 120 days (gentle, slow research) · 365 days (nearly off).

#### Tutorial 10 — The Link Dashboard (seven tabs)

| Tab | Question it answers |
|---|---|
| Most Connected | Which notes have the most links? |
| Most Traveled | Which links have you walked most often? |
| Stale | 90+ days untouched — revisit or retire. |
| Cross-Library | Links crossing library boundaries. |
| Broken | Links pointing to notes that don't exist. |
| Orphans | Notes with zero links — isolated cells. |
| Archived | Soft-deleted links with one-click restore. |

#### Tutorial 11 — Searching your knowledge
All operators work in 15 languages:

| Query | Returns |
|---|---|
| `supports [[Democracy]]` | Notes that `supports`-link to Democracy. |
| `contradicts [[My Thesis]]` | Counter-evidence. |
| `causes [[Event]]` | Causal precedents. |
| `derives-from [[Source]]` | Intellectual lineage. |
| `orphans` | Isolated notes. |

#### Keyboard shortcuts
- **Right-click a link row** → confidence/archive popover.
- **Ctrl/Cmd-click a wikilink** → opens target in a new tab.
- **Middle-click a wikilink** → same as Ctrl-click.

---

## 1. Getting Started

### System Requirements

#### Minimum — to run Constellation

- **Operating system**: Windows 10 or 11; macOS 11 (Big Sur) or later; or 64-bit Linux from the last 3 years (Ubuntu 22.04, Fedora 38, Debian 12, or equivalent)
- **Processor**: Any 64-bit computer made in 2013 or later — any Intel or AMD desktop or laptop, or any Apple Silicon Mac
- **Memory**: 4 GB free RAM
- **Disk space**: 200 MB for Constellation, plus space for your notes (your notes are plain Markdown files — typically 1–10 MB per 1,000 notes)
- **Internet**: **Not required.** Constellation runs fully on your machine. You only need internet if you choose to download an optional add-on.

#### Recommended — for comfortable everyday use, large libraries (5,000+ notes), and the Second Screen feature

- **Processor**: 8-core modern processor (Intel or AMD from 2018 onward, or any Apple Silicon Mac)
- **Memory**: 8 GB free RAM
- **Disk space**: 1 GB for Constellation and its caches
- **Display**: Full HD (1920×1080) or higher; a second monitor unlocks the Second Screen feature

#### For Constellation Sight v5 — the source classifier

Sight v5 ships with a small built-in classifier that suggests source-types for your notes. **It runs on the same hardware as Constellation core — no extra requirements.**

For users who want **higher classification accuracy** (especially for Arabic, Hebrew, Persian, and other non-Latin scripts), an optional larger classifier is available:

- **Processor**: 4-core or better
- **Memory**: 4 GB free RAM during a classification run, on top of Constellation's normal usage
- **Disk space**: 1.5 GB additional for the model file
- **Internet**: required for the one-time download (~1.1 GB) from Settings → AI. After download, the classifier runs entirely on your machine — no internet ever.

**Optional GPU acceleration** (NVIDIA, Apple Metal, or Vulkan-compatible) speeds the classifier 5–20× but is **not required** — everything works on the CPU alone.

#### Older or lower-spec machines

Constellation is designed to be fast on a 10-year-old laptop. Everything bundled in the installer works on the minimum spec above. The optional larger Sight classifier is the only feature that benefits from more recent hardware.

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

- **Create a new library**: click **+ Library** in the sidebar toolbar, the welcome screen, or the Library Manager. The Create dialog asks for a location (click *Pick…* to choose any folder on disk) and a name (pre-filled with *My Library* and pre-selected — just type the name you want). Click *Create* and the new library appears in the sidebar.
- **Link an existing library**: open the Library Manager and choose *Link existing library*, or use Mission Control (`Ctrl+P`) → *Add library*. A folder picker opens; pick a folder that's already on disk (e.g. an Obsidian vault) and Constellation registers it without copying or moving any files.
- **Remove a library**: open the Library Manager and click the trash icon next to the library. Your files are not deleted — Constellation only forgets about the library.
- **Library settings**: Each library can have its own appearance settings (fonts, colors).

### Universe Notes Folder

Every universe automatically gets a **Universe Notes** folder at its root, named after the universe. This folder holds cross-library notes — MOCs (Maps of Content), dashboards, indexes, and any notes that don't belong to a single library.

- Appears at the top of the File Explorer, above child universes and libraries
- Included in search, Sky View, and all features
- When creating a new note (`Ctrl+N`), the Universe Notes folder appears as the first option

### Child Universes

You can nest universes inside universes. A **Child Universe** is another universe folder referenced by your parent universe. Notes from child universes appear in Sky View alongside your own notes, with cross-library links shown as dashed lines.

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

Every "create" affordance in Constellation — for notes, folders, bases, and libraries — opens the same modal **Create dialog**, so the experience is consistent across the app.

| Method | Action |
|--------|--------|
| **Keyboard** | `Ctrl+N` (or `Ctrl+P` → *New note*) |
| **Sidebar toolbar** | Click **+ Note** |
| **File Tree right-click** | Right-click a folder → *New note*. The folder you clicked is pre-filled as the location. |
| **Library row right-click** | Right-click a library row in the sidebar → *New note*. The library's root is pre-filled as the location. |

In the dialog, the name field is pre-filled with the default (*Untitled*) and pre-selected — start typing to replace it. Press **Enter** to create, **Escape** to cancel. The new note opens in a tab and switches to edit mode automatically.

If a folder template is configured for the parent folder, it is applied to the new note no matter how you invoked the create. Earlier versions skipped templates on the right-click path; that inconsistency is fixed.

The same dialog handles **New Folder**, **New Base**, and **New Library** — invoke them via the corresponding `+` toolbar buttons, the right-click menus, or Mission Control. New Library additionally shows a *Pick…* button so you can choose where the library folder lives on disk; New Base in workspace mode hides the location and shows a multi-select for which libraries the base will query.

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

Clicking a child universe, library, folder, or note in any sidebar mode highlights the corresponding nodes in the Sky View graph. This bidirectional sync helps you maintain spatial awareness as you browse your knowledge base.

### Picture-in-Picture (PiP) Overlay

When Sky View is open and you click a child universe, library, or folder in the sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay. The PiP shows a filtered sub-graph containing only the nodes belonging to the selected scope, with its own legend showing only the relevant entries. You can resize and reposition the PiP window freely.

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

## 5. Search

Constellation includes a hybrid multilingual search engine powered by SQLite FTS5 with BM25 ranking, structured query filters, and Arabic-optimized normalization. Search is accessible from the sidebar toolbar.

### How to Search

Click the search icon in the sidebar toolbar or use `Ctrl+Shift+F` to activate search mode. Type your query and results appear after a brief debounce (300ms). Press `Escape` or click the `×` button to clear the search and return to the file tree.

### Search Syntax

| Syntax | Example | What it finds |
|--------|---------|---------------|
| Free text | `project management` | Notes containing those words in title or body |
| Tag filter | `#research` | Notes tagged with `#research` |
| Property filter | `status=active` | Notes with frontmatter property `status` equal to `active` |
| Wikilink filter | `links to [[Climate]]` | Notes that link to `[[Climate]]` |
| Library scope | `in:MyLibrary` | Restricts results to a specific library |
| Combined | `#research status=active economy` | All filters applied together |

### Match-Type Badges

Each search result displays a colored badge indicating how the match was found. The badge shows a localized letter from your language for accessibility (color-blind safe):

| Badge | Color | Meaning |
|-------|-------|---------|
| **T** (en) / **ع** (ar) | Blue | Title match — the search term appears in the note's name |
| **C** (en) / **م** (ar) | Green | Content match — the search term appears in the note's body |
| **S** (en) / **د** (ar) | Purple | Semantic match — conceptually related (requires embedding model) |
| **P** (en) / **خ** (ar) | Amber | Property match — matched via frontmatter property filter |
| **#** | Pink | Tag match — matched via tag filter |
| **W** (en) / **ر** (ar) | Light blue | Wikilink match — matched via wikilink filter |

Badge letters are localized for all 15 supported languages.

### Pinned Results (Navigate Through Results)

Search results stay visible after you click one. The opened note is highlighted in the result list so you can see which result you are viewing. Click another result to navigate to it without re-searching. This lets you browse through multiple results from a single search.

To clear the search, press `Escape` or click the `×` button.

### Keyboard Navigation

| Key | Action |
|-----|--------|
| `Arrow Down` | Select next result |
| `Arrow Up` | Select previous result |
| `Enter` | Open the selected result |
| `Escape` | Clear search and return to file tree |

### Search Term Highlighting

When you open a note from search results, all occurrences of your search term are highlighted in the editor. This works with Arabic-aware diacritic-insensitive matching — searching for "ادارة" will highlight "إدارة" and all diacritical variants.

### Search History

Click on the search field when it is empty to see your recent searches (last 20 queries). Each entry shows the query text and how long ago it was performed. Click any history entry to re-run that search instantly. Use the "Clear history" link at the bottom to erase all history.

Search history is stored locally on your device and persists across app restarts.

### Search Hub

The Search Hub is a full-screen search experience. Click the magnifying glass icon in the dock bar to open it. Both sidebars collapse to give maximum space. Type any term and Constellation searches everywhere simultaneously, grouping results into 5 categories: Titles, Contents, Tags, Properties, and Wikilinks. Each category has a collapsible section with a count badge. Click any result to open it in the editor with all occurrences highlighted. A "Return to Search Hub" button appears so you can go back without re-searching.

### Link Operators

Constellation supports 6 link-topology search operators:

| Syntax | What it finds |
|--------|---------------|
| `links to [[X]]` | Notes that link to X (backlinks) |
| `links from [[X]]` | Notes that X links to (outgoing links) |
| `mutual [[X]]` | Notes linked to X AND X links back (bidirectional) |
| `mentions [[X]]` | Notes containing X's name without a [[wikilink]] |
| `orphans` | Notes with no incoming or outgoing links |
| `links between [[X]] and [[Y]]` | Notes that link to both X and Y |

When typing any link operator, the `[[` autocomplete shows all notes in the universe. After selecting a note, type `#` for heading completion or `|type:` for link type completion.

---

## 6. Sky View (GraphMind)

Sky View visualizes your notes as an interactive 3D graph powered by the **GraphMind** engine (Pixi.js WebGL).

### Opening Sky View

- Press `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Sky View"

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
- **Sky View**: colored ring around nodes
- **Tab bar**: small colored dot (●) before the note title

---

## 6. Split View

Split View lets you edit multiple notes side by side in the main window.

### Opening Split View

- **Command Palette**: `Ctrl+P` then type "Split View"
- **Keyboard shortcut**: Use the assigned shortcut to cycle through modes
- **Cycle**: Off → Vertical (side by side) → Horizontal (top and bottom) → Off

### Editing in Split View

Each pane is a fully independent editor with:
- Full toolbar (bold, italic, headings, alignment, etc.)
- Breadcrumb navigation (library / note name)
- Properties panel and stage dropdown
- Save support (Ctrl+S saves the focused pane)
- Title editing and file rename

### Resizing Panes

Drag the divider between panes to resize them. Each divider is independent — with 3+ notes open, you can resize any adjacent pair without affecting the others. Works in both vertical and horizontal modes.

### Focus

Click any pane to focus it. The focused pane receives keyboard shortcuts and is tracked by the right sidebar panels (Properties, Backlinks, etc.).

---

## 7. Index

The Index is a comprehensive term glossary across all your libraries — every meaningful word, sorted alphabetically with occurrence counts.

### Opening the Index

- **Dock button**: Click the Index icon (book) in the left dock
- **Command Palette**: `Ctrl+P` then type "Index"

### Multilingual NLP Pipeline

The Index processes text through a language-aware pipeline before indexing:

- **Arabic**: Lucene Light10 algorithm — removes tashkeel, unifies hamza, strips definite article (الـ), removes grammatical suffixes
- **Hebrew**: Prefix removal (ב/ל/מ/ה/ו/כ/ש)
- **English**: Porter-like stemming (plurals, verb forms, suffixes)
- **French/Spanish/Portuguese/German**: Language-specific suffix removal
- **Russian/Turkish/Hindi/Persian**: Morphological suffix removal
- **All 15 languages**: Stop word filtering (articles, prepositions, conjunctions)

### Browsing

- **Language tabs**: Switch between All, Arabic, Hebrew, English, or # (special characters)
- **Alphabet bar**: Click a letter to filter to terms starting with that letter — the term count updates to show how many terms match
- **Click the same letter again** to clear the filter and show all terms
- **Sort modes**: Alphabetical (default) or by frequency (most common first)

### Editing from the Index

Click any note in a term's references to open it in a split preview pane alongside the Index. The preview pane is a full editor — you can edit, save, change properties, and promote stage. The search term is highlighted in the note and scrolled to automatically.

Press `Ctrl+Click` to open the note as a regular tab. A "Return to Index" button appears in the tab bar — click it to return to exactly where you left off in the Index.

### Second Screen Integration

When the Second Screen is open:
- **Click a term** → Second Screen shows all notes containing that term in a split view (note list + editor)
- **Ctrl+Click multiple terms** → Second Screen shows compare mode with each term in its own column

### Cross-language Mentions (M11 Lexical Bridge)

Constellation's Lexical Bridge knows that "knowledge" in English, "معرفة" / "علم" in Arabic, "connaissance" in French, and "知识" in Chinese all refer to the same concept — across 20,000 concepts × 15 languages, baked into the app.

By default, clicking a term in the Index shows only notes that contain that **literal word**. If you want the Index to also surface notes about the **same concept in other languages**, turn on the toggle:

- Open **Settings → Index → Expand mentions cross-language**
- Click any term in the Index — the mentions list now includes notes in other languages too
- Each cross-language match carries a small **"via {lemma}"** badge after the note name (e.g. "via شجرة" on an English-titled note when you clicked "tree", or "via knowledge" on an Arabic-titled note when you clicked "معرفة")
- Direct same-language matches still appear with no badge

The toggle is **off by default** to preserve the literal-only Index behaviour for users who want it strictly per-language. When on, it composes with everything else — frequency sort, letter filter, script tabs, second screen.

### Cross-language Filter — `≈ similar` (always on)

The filter box at the top of the Index now does **three layers** of matching as you type. Each layer adds a different kind of result:

1. **Literal substring** (always on). Typing `know` surfaces every term in your vocabulary containing those letters: `knowledge`, `known`, `knowing`, etc. The fastest layer.
2. **Cross-language bridge** — when **Settings → Index → "Expand mentions cross-language"** is on, typing `knowledge` ALSO surfaces Arabic terms whose dictionary translation is "knowledge" (`معرفة`, `علم`, …). Each marked with the **"via knowledge"** badge.
3. **Cross-language concept (`≈ similar`)** — always on, no setup. Typing `knowledge` ALSO surfaces terms whose **M11 concept** is the same as yours, even when there's no direct dictionary translation in your library. These rows carry the **`≈ similar`** badge.

How layer 3 works in plain terms: when you type `knowledge`, Constellation embeds that word once into a 384-dimension semantic space (~50 ms), looks up the ten nearest concepts in the M11 dictionary that ships with the app, expands each concept into all the languages it covers, and shows you which of *your* vocabulary terms map to those concepts. So if your library has Arabic notes that use `معرفة`, the stem `معرف` will appear in the dropdown with the `≈ similar` badge — even if you never turned the cross-language bridge toggle on.

The first time you type any query in a fresh session, expect a 2–5 second wait while the embedding model loads. Every query after that runs in ~80 ms; the panel stays responsive while you type. The IPC is debounced at 300 ms so only the *settled* query fires the embedding call, not every keystroke.

Misses are normal. The M11 dictionary covers 20,000 common-vocabulary concepts. Specialized jargon, proper nouns, and rare regional variants will often miss `≈ similar` — they still appear if they match the literal substring (layer 1) or the bridge (layer 2). Misses are not bugs.

There is **no setup**: no embedding-build phase, no per-library training, no "Rebuild" button anywhere. The 20K concept matrix ships with the app as a 30 MB asset; the per-query lookup is fully local.

---

## 8. Constellation Sight

The Constellation Sight visualizes your entire knowledge universe as a celestial-hemisphere star chart. It answers: **"What does my knowledge look like, and how healthy is it?"**

### Opening the Sight

Click the **Sight button** (star icon) in the left ribbon. The dome of stars renders on a cream parchment background — Suwaidi northern-hemisphere chart aesthetic. Click the **(×)** button in the header bar to close, or press **Esc**.

### The Dome

Notes appear as small colored dots — stars. They're arranged in a polar layout:

| Visual | Meaning (Regions mode — the default) |
|--------|--------------------------------------|
| **Position from center to rim (radius)** | How central the note is in your link graph. Center = most-connected hub; rim = peripheral leaves. |
| **Position around the rim (azimuth)** | Which **library** the note lives in. Each library gets its own wedge, sized proportional to note count. |
| **Star color** | Library membership. Same library = same color. Each library gets a unique color from a deterministic palette. |
| **Star size** | Total link count (in + out). Brightest stars are your most-connected notes. Capped so no star dwarfs the others. |
| **Black outline** | Every star has a thin contrast frame so it stays visible against the cream background. |

### The Library Legend

A panel on the **left side** of the screen (or **right side** if your Universe name reads right-to-left) lists every library with its color swatch, name, and note count. Each library is **numbered** (1, 2, 3, …) and the same number appears around the rim of the dome in the matching color — so you can read the chart by glancing between the legend and the rim.

The legend's header also shows your Universe name and a "UNIVERSE" caption. Long Universe / library names truncate gracefully; hover any name for the full title.

### The Universe Health Card

A roundel above the dome shows the overall **Universe Health score** (0-100), with four metrics flanking it: **Modularity**, **Dominance**, **Entropy**, **Connectivity**. Each metric has a colored status pill (HEALTHY / CAUTION / IMBALANCED) computed from your graph topology.

### The (X, Y, Z) Grammar — Multiple Modes

Sight is a **multi-instrument cognitive lens**. The same Universe can be read through six different "modes" — each with its own meaning for X (azimuth), Y (radius), and Z (size). Color (library) stays the same across all modes.

| Mode | X (rim wedge) | Y (radius) | Z (size) | Cognitive question |
|------|---------------|------------|----------|--------------------|
| **R · Regions** *(default)* | Library | Centrality rank | Total degree | "Where in my cosmos does this idea live, and how central?" |
| **L · Link Types** | Dominant outgoing link type (supports, contradicts…) | Type diversity | Outgoing links | "What kind of reasoning, and how versatile?" |
| **T · Time** | Creation date wedge (year + month) | Recency (last edit) | Age | "When did it emerge, and is it still alive?" |
| **C · Confidence** *(coming soon)* | Dominant confidence | Certainty homogeneity | Link count | "How certain, and how consistent?" |
| **S · Stages** *(coming soon)* | Dominant lifecycle stage | Avg link weight | Traversal count | "How alive, and how worn?" |
| **A · Acts** *(coming soon)* | Which Act produced the note | Synthesis depth | Connections | "Where in the formulation arc?" |

**Today's build:** Regions mode is active by default. The toggle bar to switch to other modes is shipping in the next phase. Stars will *migrate* between (X, Y, Z) positions when you switch modes — same star, different scan.

### Interaction

| Gesture | Effect |
|---------|--------|
| **Hover a star** | Tooltip shows the note's title (bold), community, and centrality rank. |
| **Click a star** | The star gets a gold ring; its links radiate out in ink-dark lines; connected (1-hop neighbour) stars get thin gold rings. The right-side panel slides in with note details (title, community, centrality rank, incoming/outgoing link counts, the **Connected notes** clickable list, and an "Open in editor" button). Click any row in the Connected notes list to recentre the side panel on that neighbour without leaving Sight. |
| **Click empty space** | Clears the selection. |
| **Double-click a star** | Opens the note in the editor. |
| **Mouse wheel** | Zoom in / out around the dome center. The whole "page" — chart, library legend, Universe Health card, Universe-name header — scales together as a lens. Range: 0.4× to 5×. |
| **Click + drag empty space** | Pan the chart. Drag threshold is 4 px so short clicks still hit stars. Cursor changes to `grabbing` while dragging. |
| **Reset View button** *(bottom-left)* | Snaps zoom + pan back to canonical. Always visible — muted at default state, prominent when zoomed/panned. |
| **Esc** | Cascading: first press clears any selected star; second press resets zoom + pan; third press closes Sight. |

### Search in Sight

Click the magnifying glass (or focus search via the global shortcut). Matched stars flare brighter; non-matched stars dim.

### Closing

Click the **(×)** at the top-right, or press **Esc** until the chart closes.

---

## 9. Second Screen

The Second Screen is a mode-based companion window that adapts to your current sidebar mode.

> **Requires Two Monitors** — The Second Screen is only available when two or more monitors are connected. With a single monitor, the button, keyboard shortcut, and command palette entries are hidden. The right sidebar provides the same panels inline.

- **Open**: Click the monitor icon in the bottom dock bar, or `Ctrl+Shift+2`
- **Auto-positions**: Centered on your secondary monitor at ~80% of the display
- **Auto-closes**: When you close the main window, the second screen closes automatically
- **Right sidebar auto-hides**: When the Second Screen opens, the right sidebar hides; when it closes, the sidebar returns

### Mode-Based Companion

The second screen changes its content based on the active sidebar mode in the main window:

| Main Sidebar Mode | Second Screen Shows |
|---|---|
| **File Explorer** | Universe Dashboard — stats, library breakdown, child universes, tags, recently edited/opened notes |
| **Navigator** | Full Navigator view for browsing notes |
| **Sky View** | Sky View tree with directory structure |
| **Sky View** (graph) | Sky View companion with backlinks, forward links, tags, and local graph |
| **Split View** | Comparison panels — all split notes side by side with shared panel selector |
| **Constellation Map** | Map companion with mini-maps, color dropdown, and legend |
| **Index** | Term exploration — note list + editor for clicked terms |

### Universe Dashboard (File Explorer Mode)

When the main window is in File Explorer mode, the second screen displays a dashboard with:

- **Stat cards** — Universe name, child universe count, total libraries, folders, and notes
- **Child Universes** — Each child universe with its linked libraries and folder/note counts
- **Libraries** — Each library with folder/note counts in color-coded stat boxes
- **Recently Edited** — Notes you modified in the current session (tracked when you save changes)
- **Recently Opened** — Notes you opened but did not edit in the current session
- **Tags** — All tags across libraries sorted by count; click a tag to see all notes using it

### Dashboard Interaction

When the Dashboard is active on the main window, clicking items sends them to the second screen:

- **Recently Edited/Opened**: Click a note to open it as a full editor on the second screen
- **Tags**: Click a tag to show all notes using it in a split view — note list on the left, full editor on the right

All edits on the second screen sync back to the main window automatically.

### Note Editing in Second Screen

The second screen supports full note editing — type, save, rename, and change properties just like the main window. Changes sync back to the main window automatically.

### Settings Sync

All visual settings instantly propagate to the second screen — no restart needed:

- **Language**: Interface language changes apply immediately
- **Theme**: Light/dark/system mode switches instantly
- **Fonts**: Interface font, text font, mono font, and script-specific fonts
- **Font size**: Both interface and editor font sizes
- **Editor**: Readable line length, line numbers, floating toolbar
- **Accent color**: Theme accent color changes

---

## 10. Properties and Frontmatter

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

## 10b. Source Review (Constellation Epistemic Content Engine — CECE)

Two of the most important frontmatter properties — `sources:` and `content_type:` — describe *how you came to know* something and *what kind of knowledge* it is. Constellation's **Epistemic Content Engine** (CECE) classifies every note along these two axes automatically using a 6-cataloger ensemble. The **Source Review** panel is where you review and correct those classifications.

### What the engine does

When you classify a note (right-click → "Suggest sources & content type", or via Settings > Run scan, or automatically via the background scan toggle), CECE runs six independent catalogers against the note. Each one reads the note through a different lens and votes on two questions:

- **Source** (horizontal axis) — where did this knowledge *come from*? Eleven possible values: perception, inference, testimony, mass-transmission, comparison, postulation, non-apprehension, memory, innate-disposition, inspiration, revelation. Plus *unclassifiable*.
- **Content Type** (vertical axis) — what *kind* of knowledge is this? Five top-level branches: sensory inputs, symbolic entities, semantic contents, epistemic states, higher-order constructs.

The two axes are independent. A note "I doubt the moon landing" is testimony (someone reported it) on source + epistemic-states/doubt (your stance) on content-type.

The engine runs **on your device** — no notes ever leave Constellation.

### The six catalogers

Each cataloger is one lens. The Source Review card shows them as six small colored dots in the top right corner of each card:

- **Your frontmatter** (blue) — adopts what you've already set, with absolute authority
- **Citations & structure** (rose) — citations, blockquotes, theorem markers, definition phrases
- **Wordstems & lexicon** (amber) — Arabic root analysis + cross-lingual term equivalence
- **Linked notes** (teal) — typed Living Links to other classified notes
- **Similar notes** (violet) — embedding-similarity to your already-classified notes
- **AI judgment** (green) — a local LLM (Qwen3-4B; *not yet active*, deferred to a future release)

A filled dot means that cataloger voiced and agrees with the synthesis. A ringed dot means it voiced but dissented. A dashed-outline dot means it stayed silent (no signal in this lens).

### Three confidence regimes

After the catalogers vote, each axis lands in one of three regimes:

- **Unanimous** — every voicing cataloger agreed
- **Strong majority (one dissent)** — most agreed; one dissenter named
- **Split** — no clear majority; the engine refuses to guess and asks you to pick

Each axis gets its own regime independently — a card can be Unanimous on horizontal + Split on vertical, etc.

### Sibling Disambiguation

When an axis is Split, the engine surfaces the candidate values as **chips** under a prompt: *"Pick which one fits the note best."* Click a chip → the engine writes that pick to the note's frontmatter and removes the card from the queue. If the OTHER axis was settled (Unanimous or Strong majority), the engine *also* writes that axis's value at the same time — one click finishes both axes when only one was Split.

### The reasoning trail

Every card has a *"▸ Why this classification?"* toggle. Expanding it shows one row per voicing cataloger with the reasoning, self-reported confidence, and friendly rule chips ("Surface keyword match", "Arabic root match (CAE)", "Definition marker", etc.) — these are the specific rules each cataloger triggered.

During your **first 50 reviews** the trail auto-expands on every card (a *trust-calibration period*) so you can build intuition for when to trust the engine. After that, trails collapse to on-demand on Unanimous cards. Override anytime in **Settings > Intelligence > CECE > Reasoning trail visibility**.

### The queue composition filter

Above the count strip, five chips slice the queue by what kind of decision each card needs:

- **All** — the full queue
- **Both axes need your call** — both axes Split
- **Source needs your call** — horizontal Split + vertical settled
- **Content type needs your call** — vertical Split + horizontal settled
- **Catalogers agreed** — neither axis Split (rubber-stamp candidates)

Each chip shows its bucket count. The filter is a render-layer slicer — Approve All math always operates on the full queue regardless of which filter is active.

### Per-card actions

- **Accept** — write the engine's synthesis primary on both axes; remove the card. Updates per-cataloger reliability.
- **Edit** — open a tree picker for both axes; choose manually. Same reliability update.
- **Reject** — clear the card without writing.
- **Sibling Disambiguation chip** — on Split cards only.

### Per-Library calibration

**Settings > Intelligence > CECE > Per-Library calibration** opens a read-only table showing each cataloger's accuracy per axis on the active Library. Different Libraries have different per-cataloger accuracies — Linguistic excels on Arabic-heavy Libraries, Graph excels on densely-linked ones. The synthesis layer uses this calibration data to weight votes.

A cataloger needs **20 corrections** before its accuracy ratio is shown. Below that threshold, the label reads *"(uniform)"* — the cataloger contributes uniformly weighted votes until enough data accumulates.

### Background classification

By default, CECE classifies notes only when you ask it to (right-click or Settings scan button). You can opt into automatic classification in **Settings > Intelligence > CECE > Background classification**:

- **On note save** — classify each note ~1.5 seconds after you stop typing (rides the existing debounced save; never fires per-keystroke; typing stays instant)
- **On app start** — scan unclassified notes once per launch

For deeper detail (every dot status, every rule chip, click-by-click walkthroughs of common scenarios), see the **Source Review** topic in the help system.

---

## 11. Templates

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

## 11. Tables

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

## 12. Tasks

Constellation supports task checkboxes in notes:

```markdown
- [ ] Incomplete task
- [x] Completed task
```

In Live Preview mode, checkboxes are clickable. Tasks can be searched and filtered across your libraries.

---

## 13. Importer

Import notes from other PKM tools:

- **Obsidian** — imports libraries with full wikilink compatibility
- **Markdown folders** — import any folder of `.md` files
- **Other formats** — HTML, text files

Go to **Settings > Importer** to start an import.

---

## 14. Calendar

The Calendar view shows notes organized by date:

- Notes with a `date` property appear on their respective days
- Daily notes can be created for any date
- Navigate months with arrow buttons

Open the Calendar from the sidebar.

---

## 15. Lens

Lens provides filtered views of your notes:

- Filter by tags, folders, properties
- Sort by name, date, or custom properties
- Save lens configurations for quick access

---

## 15b. Panels

Constellation's panels — Backlinks, Outgoing Links, Properties, Tags, Sky View, Tasks, Calendar, Knowledge Health, Provenance, Review Pulse, and Link Dashboard — can each be placed in one of four positions via **Settings → Panels**.

### Panel slots

| Slot | Description |
|------|-------------|
| **Left of note** | A column to the left of the editor. Best for Backlinks. |
| **Right of note** | A column to the right of the editor. Best for Outgoing Links. |
| **Right sidebar** | The right-side tab strip (default for most panels). |
| **Hidden** | Removes the panel from the interface entirely. |

### Default layout

By default, Backlinks appear to the **left** of your note and Outgoing Links appear to the **right** — placing the note at the center of its link network. All other panels live in the **right sidebar**.

### Flanking columns

The left-of-note and right-of-note positions create **flanking columns** — thin panels that bracket the editor:

- **Resize**: Drag the strip between the flank and the editor.
- **Collapse/expand**: Click the ▶/◀ arrow at the top of the strip. The column slides away with a 120 ms animation; your width setting is remembered.

### Right sidebar tabs

The tab bar shows only the panels currently placed in `right-sidebar`. Tabs for panels moved to flanks or hidden are automatically removed. If you move the active tab's panel away, Constellation switches to the next available tab.

### Workspaces

Panel placements are saved and restored with workspaces. Older workspaces (saved before this feature existed) leave the current layout unchanged when loaded.

---

## 16. Settings

Access Settings from the sidebar gear icon or `Ctrl+,`.

### Dashboard

- Universe overview and statistics

### Appearance

- Color scheme (Light / Dark / System)
- Accent color
- Interface font size (11–18px)
- Note font size
- **Themes** — pick from six built-in themes, create custom themes (five-color editor), import themes from the Obsidian Community registry (200+ themes), or import a `.json` theme file. Delete any custom theme with the ✕ button on hover.

### Style Settings

A dedicated tab for fine-grained customization of every visible interface element, applied live to the active theme.

- **Colors** — background, surfaces, text (normal/muted/faint), accent, borders, state colors
- **Typography** — interface / note / code font sizes, H1–H6 sizes, heading weight, line heights, paragraph spacing
- **Layout & Shape** — small/medium/large corner radii, border widths, shadows, editor readable line length, side margins
- **Components** — ribbon dock, sidebar action toolbar, layout bar (pane toggles), top bar / tab strip, status bar, right sidebar (inspector), file explorer (Universe notes, child universes, libraries, folders, notes), buttons, tags, callouts — each with independent size, radius, color, and where applicable, active-state styling
- **Editor** — link color/hover/decoration, inline code color/background/radius, blockquote bar width/color, cursor color, selection background

**Import / Export** — toolbar at top of the tab:
- Paste from clipboard (one-click)
- Import / Paste (textarea with Merge or Replace)
- From file (.json)
- Copy (current values to clipboard)
- Export (.json)

The format matches Obsidian's Style Settings plugin exactly, so you can share settings between Obsidian and Constellation.

Changes auto-save to the active theme; if you edit a built-in theme, it is auto-cloned into your custom themes so changes persist without modifying the original.

### Language

A dedicated tab consolidating all language-related settings:

- **Interface language** — select from 15 supported languages
- **Writing languages** — set a Primary language and an optional Secondary language, each with its own font set
- **Font Mode** — Universal (one font for all) or Per-Language (separate fonts per writing language)
- **Custom Font Sets** — system font dropdown for interface, text, and mono fonts per language
- **Date & Numbers** — numeral style (Arabic 0-9 or Hindi numerals), per-language date format with a Contextual checkbox for direction-aware rendering
- **Script Tools** — language-specific symbol and punctuation toolbars (Arabic, Hebrew, CJK, etc.)
- **Font Theme** — choose between Default and Typewriter font themes. The Typewriter theme applies authentic pre-PC-era fonts for each script (Courier Prime for Latin, Noto Naskh Arabic for Arabic, Miriam Libre for Hebrew, PT Mono for Cyrillic, Tiro Devanagari Hindi for Hindi, and system CJK fonts for Chinese/Japanese/Korean)

### Arabic Overrides

A per-Universe panel where you pin how the Arabic engine analyses specific surfaces — your own coinages, local names, field-specific loanwords, or cases where you disagree with the engine's automatic reading. Each override wins over the generative FST, the cascade, and the heuristic fallback. Adding or removing an override triggers a targeted reindex of only the notes that contain the affected surface — no full rebuild. See §18 ("RTL and Arabic Support") for the step-by-step walkthrough.

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

### Debug

Read-only diagnostic view for power users and developers. The **Boot Performance** scorecard reads `<universe>/.constellation/boot-perf.latest.json` — written on every launch — and evaluates it against the five ship-gate criteria defined in `lab/boot-perf/BOOT-BUDGET.md`:

1. **UI visible** (≤ 2.5s) — sidebar painted, last-open note skeleton on screen.
2. **Fully responsive** (≤ 6s) — typing instant, toolbar clicks open panes, search returns results.
3. **Idle RSS memory** (≤ 350 MB) — 30 seconds after Criterion 2, no input, no notes open.
4. **Post-boot stat sweep** — 50 externally-modified files detected in ≤ 3s, non-blocking.
5. **Kill mid-index recovery** — force-killing during initial index rebuild resumes cleanly.

Each row shows the target, the measured value, and a PASS/FAIL/Not-measured pill. Two collapsible panels below the scorecard show the full per-phase breakdown (graph-ready, core snapshot wall/queue/body/transport, graph snapshot, and the fire-and-forget fan-out) plus the raw JSON.

The scorecard is read-only; to refresh it, close Constellation and relaunch, then re-open the panel (or press the **Refresh** button).

---

## 17. Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New note |
| `Ctrl+O` | Star Jump (quick open) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Open Sky View |
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

### Sky View

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Search-to-highlight |
| `Ctrl+L` | Cycle layout mode |
| `Space` | Toggle focus mode |
| `0` | Reset 3D rotation |
| `W/A/S/D/Q/E` | Fly through 3D |
| `Escape` | Close Sky View |

---

## 18. RTL and Arabic Support

Constellation provides first-class support for Arabic, Hebrew, Persian, Urdu, and other RTL scripts:

- **Auto-detection**: Note direction is detected automatically from content
- **Interface**: Full RTL interface when Arabic/Hebrew language is selected
- **Editor**: RTL text editing with correct cursor movement and selection
- **Sky View**: Arabic labels render right-to-left with proper font fallback
- **Legend**: Items flip dot/text order based on content language
- **Script fonts**: Configure Arabic, Hebrew, and CJK fonts independently in Settings > Language
- **Script toolbars**: Language-specific symbol and punctuation buttons (Arabic symbols, Hebrew, CJK punctuation)
- **Tashkeel highlighting**: Toggle Arabic diacritics highlighting from the editor toolbar

### Setting Up for Arabic

1. Go to **Settings > Language** and select Arabic as your interface or writing language
2. Optionally set a dedicated Arabic font set in **Settings > Language > Custom Font Sets**
3. Enable Script Tools for Arabic symbol toolbar access
4. Notes with Arabic content will automatically render RTL

### Arabic Engine Overrides

Constellation's Arabic engine is a five-layer morphological analyser that runs beneath every search, link, and index entry. It understands roots, patterns, proper nouns, loanwords, and phonological repairs — so a query for كاتب finds كتبنا and كتاب, but وائل stays intact as a name instead of being mangled into ائل.

The **Arabic Overrides** panel in Settings is where you teach the engine your own terminology. Each override is the sovereign answer — it wins over the generative FST, the cascade, and the heuristic fallback.

**When to use overrides:**
- Personal names, local place names, or field-specific terms the engine does not know
- Coinages or acronyms unique to your Universe
- Loanwords where you want a specific spelling preserved
- Any case where the engine's automatic analysis disagrees with how you read the word

**Step-by-step:**

1. Open **Settings** (gear icon or `Ctrl + ,` / `Cmd + ,`) and select **Arabic Overrides** in the sidebar.
2. Click **Add override**.
3. Fill in:
   - **Surface** — the Arabic word as you type it
   - **Lemma** — the canonical form the engine should return
   - **Root** (optional) — 3 or 4 consonants if the word has a classical root
   - **Pattern** (optional) — e.g. `فاعل`
   - **Part of speech** — Proper noun / Noun / Adjective / Adverb / Verb / Particle / Foreign / Unknown
   - **Note** (optional) — a line of context for yourself
4. Click **Save**. The panel shows **Reindexing…** while every note containing the surface is re-tokenised, then **Reindexed N note(s)** when done.
5. To remove an override, click the **×** on its row — the same reindex sweep runs in reverse.

Overrides are stored per Universe at `<universe>/.constellation/arabic-overrides.json` — plain text, alphabetically sorted, atomically written. You can version-control the file or share it across devices.

---

## 19. Security and Privacy

- **All data stays local** — no cloud sync, no telemetry, no tracking
- **Markdown files** — your notes are plain text files you own completely
- **No account required** — Constellation works entirely offline
- **Optional updates** — check for updates manually via Settings
- **Open source** — inspect the code at [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 20. Constellation Map

The Constellation Map is a radial sunburst visualization that shows the structure, density, and maturity of your entire knowledge universe.

### Opening the Map

- **Dock button**: Click the Constellation Map icon in the left dock
- **Command Palette**: `Ctrl+P` then type "Constellation Map"

### What You See

- **Center**: Your Universe name with total note and word counts
- **First ring**: Libraries (each colored with its library color). If your universe has child universes, they appear here too.
- **Deeper rings**: Folders and subfolders within each library
- **Outermost segments**: Individual notes

### Color Modes

Switch between three modes via the dropdown:
- **Maturity**: seed (gray) → sapling (light green) → evergreen (green) → canonical (gold) → wilting
- **Stratum**: L1 (blue) → L8 (red) — showing knowledge complexity
- **Library**: all segments inherit their parent library's color

### Drill-Down Navigation

Click any folder segment to zoom in. A breadcrumb trail shows your path. Click any breadcrumb item to zoom back, or press Escape. Click a note segment to open it in the editor.

### Return to Map

After opening a note from the Map, a "Return to Map" button appears in the tab bar. Click it to return to exactly where you were — same drill-down level preserved.

---

## 21. Cognitive Engine

> "The quantity of your data and information doesn't matter. It is NOT about how many references or sources you keep or store; it is about how you formulate your KNOWLEDGE from them, and how to link all of it into one meaningful awareness."

The Cognitive Engine is a two-layer architecture that transforms Constellation from a note-taking app into a knowledge cognition instrument. Most note apps help you store and retrieve information. The Cognitive Engine goes further: it helps you understand what your knowledge actually means, where it comes from, how mature it is, and where the gaps lie.

**Layer 1 — Structural Cognition** (zero AI dependency): Ten tools that analyze your notes' structure, connections, and metadata to surface insights. Everything runs locally on your machine, fully offline, with no AI dependency. The engine reads the shape of your library — word counts, link counts, link types, and graph topology — to tell you things about your knowledge that you cannot easily see yourself.

**Layer 2 — AI Discovery** (coming soon): AI will read Layer 1's structures to find patterns you cannot see from inside your own knowledge.

All ten Cognitive Engine features require no configuration. They activate automatically as your library grows. You do not need to enable them or install anything extra.

---

### 18.1 Typed Links

**What it is**

Typed Links let you add semantic meaning to the connections between your notes. Instead of a plain link like `[[Climate Change]]` that only says "these two notes are related somehow," a Typed Link says exactly how they are related: `[[Climate Change|type:supports]]` means "this note provides evidence for the claims in Climate Change." Constellation supports seven link types, each with a distinct color in Sky View.

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
- **In Sky View (GraphMind)**: Each link type has a distinct color, so you can visually trace chains of support, contradiction, or derivation across your entire library.
- **In the autocomplete menu**: When you type `|type:` inside a wiki-link, all seven types appear with short descriptions.

**Tips**

- You do not need to type every link. Start by typing the links that carry the strongest meaning — the ones where you know "this supports that" or "this contradicts that." Even a handful of typed links will light up your Sky View.
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
3. In **Sky View**, nodes are sized and layered by stratum. Higher-stratum notes appear larger and more prominent.
4. To raise a note's stratum naturally:
   - Write more (expand from a short fact into a developed explanation).
   - Link it to other notes (connect it to the broader web of your knowledge).
   - Use Typed Links (adding `supports`, `generalizes`, or `causes` links signals deeper structural relationships).

**Where you see it**

- **Right sidebar**: The note's stratum level is shown in the properties area.
- **Sky View (GraphMind)**: Node size reflects stratum. Datum notes appear as small dots; Worldview notes appear as large, prominent nodes.
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
- **Sky View (GraphMind)**: Maturity affects the visual appearance of nodes, helping you see at a glance which parts of your knowledge are well-developed and which are still germinating.
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
- **Sky View**: Contradiction links appear in a distinct color, making tension lines visible in your knowledge graph.

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
- **Sky View (GraphMind)**: `derives-from` links appear as distinct edges, so you can visually trace provenance chains across your library.
- **Note properties**: The origin type (Received / Discovered / Mixed) appears in the note's metadata in the right sidebar.

**Tips**

- You do not need to add `derives-from` links to every note. Focus on the notes where source attribution matters — where you want to remember "this idea came from that book" or "this argument builds on that conversation."
- A note classified as "Discovered" is not necessarily better than "Received." The most powerful knowledge often comes from deeply processing received ideas until they become your own. The classification helps you see the balance.
- If the Provenance tab shows "No derives-from chain found," it means the current note has no provenance links yet. The panel will display a hint reminding you of the syntax.
- Provenance Chains are especially valuable for academic work, research projects, or any context where you need to trace an idea back to its original source.

### 18.6 Stages — the Living Link lifecycle

**What it is**

A note's lifecycle position. Knowledge isn't born finished — an idea begins as a flicker, takes shape, accumulates evidence, settles, fades, or is retired. Stages mark **where the thinking sits now**, not what the note is about.

Constellation uses **six fixed lifecycle stages** that any note can carry, in this order:

| # | Stage | Icon | Meaning |
|---|-------|------|---------|
| 1 | Spark | ✨ | First ignition — a question, hypothesis, or hunch captured before substance. |
| 2 | Birth | 🌱 | First concrete formulation; a defensible claim. |
| 3 | Growth | 🌿 | Active development — evidence, structure, links accumulating. |
| 4 | Maturity | 🌳 | Settled; depended-upon; cite-stable. |
| 5 | Dormancy | 😴 | Quiet but preserved. A pause, not a retirement. |
| 6 | Archival | 📦 | Retired or superseded; preserved for reference. |

You can also add a **per-note custom term** that pairs with each lifecycle stage — e.g. `Spark-Concept`, `Birth-Concept`, `Growth-Concept`. Custom terms are typed per note and live only on that note (nothing is set Universe-wide).

**Why it matters**

A note about gardening and a note about epistemology both move through the same lifecycle. Tracking that lifecycle makes the distinction between raw capture, working substance, and settled knowledge visible at a glance.

The custom-term layer lets you mark *what kind* of note a particular stage represents — say, a `Birth-Concept` versus a `Birth-Hypothesis` versus a `Birth-Argument` — without forcing a Universe-wide vocabulary.

**How to use it**

*Setting a stage from the Properties panel:*
1. Open any note and expand the Properties panel.
2. Click the stage value. A dropdown opens with **6 entries**.
3. **Mode A** (default): the 6 fixed lifecycle stages. Pick one to commit.
4. **Mode B** (custom): type a word like `concept` in the input. The dropdown swaps to show 6 paired stages — `Spark-Concept`, `Birth-Concept`, etc. Pick one. The fixed entries are hidden in Mode B.
5. Press **Enter** without picking to commit:
   - Mode A: typed name commits as-is (e.g. `birth` → `birth`).
   - Mode B: typed term commits as `spark-<term>` (the first paired entry).
6. **Switching back to fixed**: clear the input or type a fixed name. The dropdown returns to Mode A.

*Promoting / demoting from the breadcrumb:*
1. The breadcrumb above the editor shows the current stage as a badge: `🌿 Growth-Concept`.
2. The **Promote →** arrow advances to the next lifecycle phase, **carrying the suffix verbatim** (`Growth-Concept` → `Maturity-Concept`).
3. The **← Demote** arrow goes back one phase.
4. At Spark, the demote arrow is hidden. At Archival, the promote arrow is hidden.
5. To change the custom term itself (or remove it), edit the value in the Properties panel.

**Where you see it**

- **Breadcrumb bar**: badge with `<emoji> <Lifecycle>` or `<emoji> <Lifecycle>-<Term>`, plus the promote/demote arrows.
- **Properties panel**: the single stage combobox with mode-flip behaviour.
- **File tree**: a lifecycle emoji appears next to each note's name. The custom-term suffix is *not* shown in the tree (lifecycle only).
- **360.3D / Inspector**: the full label with suffix, e.g. `✨ Spark-Concept`.

**Tips**

- Stages are optional. Notes without a `stage:` row work normally.
- The dash separator (`Spark-Concept`) is the canonical encoding. On disk the value is lowercase: `stage: spark-concept`.
- Each note's custom term is independent. Two notes with `stage: birth-concept` aren't linked to each other — each typed it on its own.
- Promote / demote walks the lifecycle, not the custom term. To switch tracks, edit Properties.
- **Old Zettelkasten values still work**: notes saved before MIG-014 with `stage: fleeting / literature / permanent / synthesis` keep their on-disk values and display with their old emoji. They aren't promoteable in the new chain — to advance them, edit the stage value to a Living Link baseline.

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

Multi-Lens Views let you view your library through different classification schemes without changing your folder structure or duplicating notes. A "lens" is a virtual grouping that reorganizes notes based on a property or tag. Built-in lenses: "By Stage" (groups by lifecycle position — Spark / Birth / Growth / Maturity / Dormancy / Archival) and "By Topic" (groups by tags). You can create custom lenses in Settings.

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
- "By Stage" pairs with the Living Link lifecycle (§18.6) to show your formalization progress — Spark / Birth / Growth / Maturity / Dormancy / Archival, plus any per-note custom-term variants.
- "By Topic" is useful for large libraries where related notes are scattered across folders.
- Custom lenses can group by any frontmatter property: `project`, `status`, `priority`, etc.
- No notes are duplicated or moved. Lenses are purely virtual views.

---

### 18.10 360.3D Inspector — Stratification Matrix

**What it is**

The 360.3D Inspector is the synthesis surface of the Cognitive Engine. Every other CE feature (Strata, Maturity, Tension, Provenance, Stage, Review, Trails, Lenses) shows you one slice of a single note. The Inspector shows you all of them at once, for one note, in one visual frame. It answers exactly one question, deeply:

> **Where does this note stand in my Cognitive Knowledge?**

The Inspector renders that standing as a **Stratification Matrix** — an 8 × 8 grid where the **vertical axis is stratum** (the 8 levels of intellectual altitude: Worldview at the top → Datum at the bottom) and the **horizontal axis is link direction** (the 7 typed link types — supports, contradicts, causes, derives-from, generalizes, exemplifies, part-of — plus an Untyped column).

Each connected note appears as a coloured dot in the cell at the intersection of its own stratum and the typed direction it shares with the active note. Your active note's row is highlighted in purple; its name appears as a chip on the right edge of that row. Empty cells render as faint diagonal stripes — gaps are shown as deliberately as connections.

**Why it matters**

A note's value is not just its content — it is its place in your knowledge. The matrix makes that place visible at a glance:

- **Vertical position** tells you the note's intellectual altitude. A note at L4 with all its connections at L1–L2 is sitting on raw data; the same note at L4 with connections at L5–L7 is reaching upward toward principle and theory. You can see the difference without reading a single name.
- **Horizontal spread** tells you whether your thinking around the note is balanced. A row that is full under `supports` and empty under `contradicts` is one-sided thinking, visible at a glance. The Inspector does not answer for you, but it makes the question hard to miss.
- **Empty cells** are first-class. Five blind spots in a row are five questions you have not asked yet.

Below the matrix, a dimensions strip surfaces the note's non-spatial facts (maturity, origin type with trust depth, stage, review status, trail memberships, lens groupings). A bottom HUD summarises structural facts: outbound count, inbound count, word count, orphan flag, fragility flag, blind-spots count, and tensions count.

**How to use it**

There are two ways to open the Inspector for whichever note you have active.

1. **Compact scorecard in the right sidebar (always-on glance)**: in the right sidebar tab strip, click the **360.3D Inspector tab** (icon: a small reticle — circle with centre dot and four spokes). A scorecard appears: the note's name, a stratum pill (e.g. `L4 Concept`), a maturity pill, your outbound / inbound / word counts, and a per-direction bar chart with explicit em-dash markers for blind spots. The scorecard updates as you switch notes in the editor.

2. **Full-window matrix (deliberate study)**: in the left ribbon (dock), click the **360.3D Inspector dock button** (same reticle icon, larger). The editor area is replaced by the full matrix.

**Reading the matrix**

- **Hover any dot** to reveal the connected note's name in a tooltip at the top-right of the matrix. The tooltip stays in place — it does not chase the mouse — so you can read it while looking at other rows.
- **Click any dot** to navigate the Inspector to that note. The matrix re-fetches and redraws around the new note as the active centre. A back-button bar appears at the top of the Inspector.
- **Click the back-button** ( ← {previous note name} ) to step back one note in your trail. It walks all the way back through any chain of clicks until you reach the note you started from.
- **Close the full-window** with the **×** in the top-right corner.

**Where you see it**

- **Right sidebar tab**: the compact scorecard widget, alongside Backlinks / Outgoing / Tasks / Calendar / Health / Provenance / Review / Links.
- **Ribbon dock button**: opens the full-window matrix.

**Tips**

- The compact scorecard is for ambient awareness as you write. The full-window matrix is for deliberate study of a single note.
- The matrix's strength is at-a-glance pattern reading. Don't try to memorise individual dots — read the shape of each row, each column, and the empty regions.
- A row that is fully empty (no connections at that stratum) is a stratum your thinking has not reached. A column that is fully empty (no connections of that type) is a typed direction you have never used for this note.
- The "Open a note to see its 360.3D view" empty state means the Inspector is open but no note is in focus. Open any note in the editor (or click in your library) and the matrix will fill in.
- On large libraries (thousands of notes), the first open of the Inspector for a given note takes a moment while it computes. Subsequent opens are faster.

---

*Constellation User Manual — Version 0.1.0 — March 2026*
*uconstellation.world*

# Constellation Development Plan
## Goal: Complete Knowledge Management Platform

> Feature gap analysis based on comprehensive review of existing knowledge management tools
> vs. Constellation's current codebase (44 components, 9 AI skills, 4 providers, file watcher, split view)

---

## Current State Summary

### Already Implemented
- Multi-library management (add, remove, browse)
- File tree with expand/collapse
- Note reading with rendered Markdown (headings, bold, italic, lists, code, links, tables, blockquotes, images, checkboxes, HR)
- WikiLink rendering (`[[target]]` and `[[target|display]]`) + click-to-open
- CodeMirror 6 editor with auto-save (800ms debounce)
- Smart bracket/pair wrapping with undo/redo support
- YAML frontmatter property editor (text, number, date, list, link types)
- Split view (vertical/horizontal) with focused pane tracking
- Tab system with library color indicators
- Right sidebar (properties panel + outline/headings)
- Left sidebar (library tree + search)
- Context menu (new note, new folder, rename, delete)
- Confirmation dialog for destructive actions
- File watcher (per-library, .md + directories)
- Library appearance loading from existing theme settings
- Per-library CSS theming (accent color, fonts, font size)
- Full-text search (filename + content, 50 results)
- RTL/LTR auto-detection + 15 languages
- AI integration (OpenAI, Anthropic, Gemini, Ollama)
- 9 built-in AI skills (summarize, Q&A, writing assistant, auto-linker, translate, meeting notes, chart generator, research)
- Status bar (word count, char count, properties count, library count)
- Settings modal (language, AI provider config)
- Inline rename in file tree

### Not Implemented (Gap Analysis)
The gaps are organized into **10 phases** below, ordered by impact and dependency.

---

## Phase 1: Markdown Rendering Completeness
**Priority: Critical | Effort: Medium | Impact: High**

Constellation renders basic Markdown but is missing several extended formatting features.

### 1.1 Highlights (`==text==`)
- Add custom `marked` extension (like the wikilink one)
- Tokenizer: match `==(.+?)==`
- Renderer: `<mark>text</mark>`

### 1.2 Strikethrough (`~~text~~`)
- Verify GFM strikethrough is enabled in `marked` config
- Should render as `<del>text</del>`

### 1.3 Callouts (`> [!type] Title`)
- Add custom `marked` extension for blockquote parsing
- Detect `[!type]` pattern at start of blockquote
- 13 callout types with distinct colors/icons: note, abstract, info, todo, tip, success, question, warning, failure, danger, bug, example, quote
- Support aliases (e.g., `summary` → `abstract`)
- Foldable callouts: `[!type]+` (open) and `[!type]-` (closed)
- Nested callouts support
- CSS styling for each type

### 1.4 Footnotes (`[^ref]`)
- Add footnote extension to `marked`
- Inline reference: `[^1]` renders as superscript link
- Definition: `[^1]: text` renders at bottom of note
- Click footnote reference → scroll to definition

### 1.5 Comments (`%% hidden %%`)
- Add tokenizer for `%%(.+?)%%` (inline and multiline)
- Strip from rendered output (hidden in reading view)
- Show as-is in editing view

### 1.6 Math/LaTeX (`$inline$` and `$$block$$`)
- Integrate KaTeX or MathJax library
- Inline math: `$formula$` → rendered inline
- Block math: `$$\nformula\n$$` → centered block
- Add as custom `marked` extension

### 1.7 Mermaid Diagrams
- Integrate Mermaid.js library
- Detect ` ```mermaid ` code blocks
- Render as SVG diagrams in reading view
- Support: flowcharts, sequence diagrams, gantt, pie, etc.

### 1.8 Syntax Highlighting in Code Blocks
- Integrate highlight.js or Prism.js
- Auto-detect language from ` ```lang ` identifier
- Apply syntax colors in rendered code blocks

### 1.9 Embedded Notes (Transclusion) (`![[note]]`)
- Add tokenizer for `![[target]]` syntax
- Resolve target note path
- Fetch and render content inline
- Support `![[note#heading]]` (embed section)
- Support `![[note#^block-id]]` (embed block)
- Support `![[image.png]]` with size `![[image.png|640x480]]`
- Prevent circular embedding (detect loops)

### Files to modify:
- `src/lib/utils.ts` — Add marked extensions (callouts, highlights, comments, footnotes, embeds)
- `src/lib/components/NotePane.svelte` — Handle embed resolution, math/mermaid rendering
- `package.json` — Add katex/mermaid/highlight.js dependencies
- New CSS for callout types, footnotes, math blocks

---

## Phase 2: Editor Enhancements
**Priority: Critical | Effort: Large | Impact: High**

The editor needs several features to match the best-in-class editing experience.

### 2.1 Link Autocomplete
- When user types `[[`, show a dropdown of all notes in library
- Fuzzy search as user types
- Show file path for disambiguation
- Include aliases in suggestions
- Press Enter/Tab to insert selected note name
- Close on `]]` or Escape

### 2.2 Tag Autocomplete
- When user types `#`, show dropdown of existing tags
- Fuzzy search
- Support nested tags (`#parent/child`)

### 2.3 Slash Commands
- When user types `/` at start of line, show command menu
- Commands: heading levels, list types, callout, code block, table, divider, etc.
- Insert the appropriate Markdown syntax

### 2.4 Smart Lists
- Auto-continue lists when pressing Enter
- `- item` + Enter → `- ` on next line
- `1. item` + Enter → `2. ` on next line
- `- [ ] task` + Enter → `- [ ] ` on next line
- Empty list item + Enter → remove the list prefix (exit list)
- Tab/Shift+Tab for indent/outdent within lists

### 2.5 Tab Key Handling
- Tab inserts indentation (configurable: tabs vs spaces)
- Shift+Tab removes one level of indentation
- With selection: indent/outdent all selected lines

### 2.6 Line Operations
- Ctrl+Shift+K: Delete current line
- Alt+Up/Down: Move current line up/down
- Ctrl+Shift+Up/Down: Duplicate line up/down
- Ctrl+D: Select current word / next occurrence

### 2.7 Multiple Cursors (Stretch goal)
- Alt+Click to add cursors
- Ctrl+D for multi-select same word
- This is complex in a textarea — may need to upgrade to a proper editor component later

### 2.8 Find and Replace
- Ctrl+F: Find bar within the note
- Ctrl+H: Find and replace
- Support regex
- Match case toggle
- Replace one / replace all

### Files to modify:
- `src/lib/components/NotePane.svelte` — Editor keydown handlers, autocomplete logic
- New `src/lib/components/AutocompleteDropdown.svelte` — Reusable autocomplete component
- New `src/lib/components/FindReplace.svelte` — Find/replace bar
- `src/lib/libraries/store.ts` — Functions to get all note names, all tags for autocomplete

---

## Phase 3: Advanced Linking & Navigation
**Priority: High | Effort: Medium | Impact: High**

### 3.1 Backlinks Panel
- New sidebar panel showing all notes that link TO the current note
- Scan all library notes for `[[current-note-name]]` references
- Show context around each backlink (surrounding text)
- Linked mentions (explicit `[[links]]`)
- Unlinked mentions (text matching note name without `[[]]`)
- Click to open the linking note
- "Link" button to convert unlinked mention to wikilink

### 3.2 Outgoing Links Panel
- New sidebar panel showing all notes the current note links TO
- Parse current note for all `[[target]]` references
- Show which targets exist vs. don't exist (dead links)
- Unlinked mentions: notes whose names appear in text without links

### 3.3 Link to Heading (`[[note#heading]]`)
- Extend wikilink resolution to support `#heading` suffix
- When resolving, find the note then scroll to the heading
- In autocomplete: after selecting a note, show heading sub-menu

### 3.4 Block References (`[[note#^block-id]]`)
- Support `^block-id` syntax at end of any paragraph/list item
- Link to specific blocks within notes
- Auto-generate block IDs when linking

### 3.5 Aliases
- Read `aliases` property from frontmatter
- Include aliases in link autocomplete suggestions
- Show alias indicator in suggestions (arrow icon)
- Resolve links by alias name

### 3.6 Auto-update Links on Rename
- When renaming a note, scan all library notes for references to the old name
- Update all `[[old-name]]` → `[[new-name]]`
- Handle both wikilinks and markdown links
- Configurable: ask before updating / auto-update / don't update

### Files to modify:
- New `src/lib/components/BacklinksPanel.svelte`
- New `src/lib/components/OutgoingLinksPanel.svelte`
- `src/routes/+layout.svelte` — Add new panels to right sidebar
- `src/lib/libraries/store.ts` — Backlink scanning, alias resolution, link update on rename
- `src-tauri/src/libraries.rs` — Batch scan notes for links, batch update links

---

## Phase 4: Built-in Features
**Priority: High | Effort: Large | Impact: High**

### 4.1 Sky View
- Interactive force-directed graph visualization
- Each note = node, each `[[link]]` = edge
- Node size proportional to number of connections
- **Interaction**: Hover to highlight connections, click to open note
- **Controls**: Zoom, pan, search filter
- **Groups**: Color nodes by folder, tag, or custom query
- **Local Graph**: Show connections for just the active note
- Use D3.js force simulation or vis.js/cytoscape.js
- Toggle between global and local Sky Views

### 4.2 Command Palette
- `Ctrl+P` to open command palette overlay
- Search and execute any command
- Show associated keyboard shortcuts
- Pinned/recent commands at top
- Commands from: file operations, editing, view toggles, navigation

### 4.3 Quick Switcher
- `Ctrl+O` to open quick switcher overlay
- Fuzzy search all notes by name
- Show file path for disambiguation
- Create new note if no match found
- Recent files at top when empty

### 4.4 Daily Notes
- One-click create/open today's note
- Configurable date format for filename (default: `YYYY-MM-DD`)
- Configurable folder for daily notes
- Template support (insert template content on creation)
- Ribbon icon + command palette entry
- Optional: open on startup

### 4.5 Templates
- Designate a template folder in settings
- Insert template content into current note
- Template variables:
  - `{{title}}` — current note filename
  - `{{date}}` — current date (format configurable)
  - `{{time}}` — current time
- Command palette: "Insert template" → select from template list

### 4.6 Bookmarks
- Star/bookmark notes, folders, headings, searches
- Bookmark sidebar panel with drag-to-reorder
- Bookmark groups/folders for organization
- Keyboard shortcut to toggle bookmark on current note
- Persist bookmarks to universe directory

### 4.7 Tags View
- Sidebar panel listing all tags across the library
- Show count of notes per tag
- Click tag to filter/search
- Nested tag hierarchy display (`#parent/child` as tree)
- Distinguish inline tags (`#tag`) from frontmatter tags

### 4.8 Page Preview (Hover Preview)
- Ctrl+hover (or just hover) over internal links
- Show floating popup with rendered preview of linked note
- Configurable: hover delay, Ctrl+hover vs hover
- Preview disappears on mouse leave

### 4.9 Outline Panel Enhancement
- Already have basic outline in right sidebar
- Add: fold/collapse heading sections
- Add: drag headings to reorder (rearrange note structure)
- Add: highlight currently visible heading (scroll sync)

### 4.10 Note Composer
- **Merge notes**: Select target note, merge current note into it, update all links
- **Extract selection**: Select text, extract to new note, leave `[[link]]` behind
- Available from command palette and context menu

### Files to modify:
- New `src/lib/components/GraphView.svelte` — Force graph (D3.js)
- New `src/lib/components/CommandPalette.svelte` — Overlay command search
- New `src/lib/components/QuickSwitcher.svelte` — Note search overlay
- New `src/lib/components/BookmarksPanel.svelte` — Bookmarks sidebar
- New `src/lib/components/TagsPanel.svelte` — Tags sidebar
- New `src/lib/components/PagePreview.svelte` — Hover preview popup
- New `src/lib/components/DailyNotes.svelte` — Daily notes config/button
- `src/routes/+layout.svelte` — Register global hotkeys, add panels
- `src/lib/libraries/store.ts` — Graph data computation, bookmark store, tag index
- `src-tauri/src/libraries.rs` — Scan all notes for links (graph data), tag indexing
- `package.json` — Add d3.js dependency

---

## Phase 5: File Management Enhancements
**Priority: Medium | Effort: Medium | Impact: Medium**

### 5.1 Drag and Drop in File Tree
- Drag files between folders to move them
- Drag folders to reorganize
- Visual indicators (drop target highlighting)
- Update all internal links after moving

### 5.2 File Sorting
- Sort by name (A-Z, Z-A)
- Sort by modification date (newest first, oldest first)
- Configurable per-library or global
- Sort toggle in file explorer header

### 5.3 Reveal Active File in Explorer
- Button/command to scroll file tree to and highlight the currently open note
- Auto-expand parent folders

### 5.4 Deleted Files Handling
- Configurable: system trash, `.trash/` folder, permanent delete
- Confirm before delete (already implemented)
- Show deleted count in confirmation dialog

### 5.5 Attachment Management
- Configurable default attachment folder
- When pasting/dropping images, save to configured folder
- Support embedding images, audio, video, PDF in notes
- Image paste from clipboard → save as attachment → insert `![[image]]`

### 5.6 File Recovery / Version History
- Periodic snapshots of note content
- Store snapshots in universe directory
- UI to browse and restore previous versions
- Configurable snapshot interval and retention

### Files to modify:
- `src/lib/components/FileTree.svelte` — Drag/drop, sort controls
- `src/lib/libraries/store.ts` — File move with link updates, attachment handling
- `src-tauri/src/libraries.rs` — Move file command, trash handling, snapshot system
- `src/routes/+layout.svelte` — Attachment paste handler, reveal in explorer
- New `src/lib/components/FileHistory.svelte` — Version history browser

---

## Phase 6: Appearance & Theming
**Priority: Medium | Effort: Medium | Impact: Medium**

### 6.1 Dark Mode
- Full dark mode support
- Toggle: Light / Dark / System
- Dark mode CSS variables for all components
- Respect existing appearance settings from library themes

### 6.2 CSS Snippets Integration
- Read CSS snippets from library configuration
- Apply enabled snippets to the note rendering
- Toggle snippets in settings
- Preview snippet effects

### 6.3 Theme Integration
- Support custom themes
- Apply theme CSS variables to note rendering
- Show theme name in settings

### 6.4 Font Management
- Font picker UI in settings
- Apply custom fonts to:
  - Interface (UI elements)
  - Text (note content)
  - Monospace (code blocks, editor)
- Ctrl+Scroll to adjust font size

### 6.5 Accent Color Picker
- Color picker in settings
- Apply accent color across all UI elements
- Override per-library from appearance settings

### 6.6 Readable Line Length
- Toggle max-width constraint on note content
- Default: ~700px centered
- When disabled: content fills full width

### 6.7 Show Line Numbers (Editor)
- Toggle line numbers in editor
- Gutter with line numbers alongside the editor

### Files to modify:
- `src/app.css` or new `src/lib/theme.css` — Dark mode variables, base theme
- `src/routes/+layout.svelte` — Theme toggle, system theme detection
- `src/lib/libraries/store.ts` — Theme/snippet loading
- `src/lib/components/NotePane.svelte` — Line numbers, readable width
- Settings page — Theme section

---

## Phase 7: Keyboard Shortcuts & Commands
**Priority: Medium | Effort: Medium | Impact: High**

### 7.1 Global Hotkey System
- Centralized hotkey registry
- Default hotkeys:
  - `Ctrl+N` — New note
  - `Ctrl+O` — Quick switcher
  - `Ctrl+P` — Command palette
  - `Ctrl+F` — Find in note
  - `Ctrl+H` — Find and replace
  - `Ctrl+Shift+F` — Search in library
  - `Ctrl+E` — Toggle edit/reading mode
  - `Ctrl+Enter` — Toggle checkbox
  - `Ctrl+B` — Bold
  - `Ctrl+I` — Italic
  - `Ctrl+K` — Insert link
  - `Ctrl+;` — Add property
  - `Ctrl+Tab` / `Ctrl+Shift+Tab` — Switch tabs
  - `Ctrl+W` — Close tab
  - `Ctrl+\` — Toggle left sidebar
  - `Ctrl+Shift+\` — Toggle right sidebar
  - `Alt+←/→` — Navigate back/forward
  - `F2` — Rename file
- Customizable hotkey bindings
- Conflict detection
- Persist to settings

### 7.2 Hotkey Settings Panel
- List all commands with current hotkey
- Search/filter commands
- Click to rebind
- Reset to defaults
- Show conflicts

### 7.3 Navigation History
- Back/Forward navigation (like browser)
- Track visited notes in a history stack
- Alt+← / Alt+→ to navigate
- Breadcrumb or back button in UI

### Files to modify:
- New `src/lib/hotkeys.ts` — Hotkey registry, default bindings, conflict detection
- `src/routes/+layout.svelte` — Global keydown listener
- Settings page — Hotkeys section
- `src/lib/libraries/store.ts` — Navigation history stack

---

## Phase 8: Advanced Features
**Priority: Low | Effort: Large | Impact: Medium**

### 8.1 Canvas (Whiteboard)
- Infinite zoomable canvas workspace
- Card types: text cards, note cards (linked), media cards
- Connections between cards (arrows/lines with labels)
- Card colors and groups
- Pan, zoom, zoom-to-fit
- Save as `.canvas` JSON files (JSON Canvas spec)
- This is a MAJOR feature — consider using a canvas library (Fabric.js, Konva, or custom SVG)

### 8.2 Workspaces
- Save current layout (open tabs, split configuration, sidebar state) as named workspace
- Switch between saved workspaces
- Persist to universe directory

### 8.3 Pop-out Windows
- Open a note in a separate OS window
- Tauri supports multiple windows
- Independent tab management per window

### 8.4 Vim Mode
- Optional Vim keybindings in the editor
- All standard Vim motions, modes, commands

### 8.5 Import System
- Import wizard for migrating from other note apps
- Support Notion, Evernote, Bear, Roam Research, Apple Notes
- Convert proprietary formats to Markdown
- Map metadata to frontmatter properties

### 8.6 Bases (Database Views)
- Create `.base` files with database-like views of notes
- Table, List, Cards, Map layouts
- Filter by properties, sort, group
- Formulas and computed fields
- CSV export
- This is a MAJOR feature — consider building incrementally

### Files to modify:
- New `src/lib/components/Canvas.svelte` — Canvas workspace (major)
- New `src/lib/components/ImportWizard.svelte` — Import UI
- New `src/lib/components/BasesView.svelte` — Database views (major)
- `src-tauri/src/lib.rs` — Multi-window support
- Various store and backend changes

---

## Phase 9: Settings & Configuration
**Priority: Medium | Effort: Medium | Impact: Medium**

### 9.1 Comprehensive Settings Page
Expand the settings page to include all configurable options:

#### Editor Settings
- Default view for new tabs (reading/editing)
- Default editing mode (source/live preview)
- Readable line length toggle
- Strict line breaks
- Show line numbers
- Show indentation guides
- Auto-pair brackets (already implemented, make configurable)
- Auto-pair Markdown syntax
- Smart lists (auto-continue)
- Tab size (2/4 spaces)
- Spellcheck toggle + language

#### Files & Links Settings
- Default location for new notes (root / current folder / specific folder)
- Default attachment folder
- Link format (shortest path / relative / absolute)
- Auto-update links on rename
- Wikilinks vs Markdown links
- Confirm file deletion (already implemented)
- Trash destination (system trash / .trash / permanent)
- Excluded files/folders pattern

#### Appearance Settings
- Color scheme (light / dark / system)
- Accent color
- Interface font
- Text font
- Monospace font
- Font size
- Readable line length
- CSS snippets management

#### Hotkeys Settings
- Full hotkey configuration panel (see Phase 7)

#### Feature Settings
- Enable/disable built-in features
- AI provider configuration (already implemented)

### 9.2 Settings Persistence
- Save settings to a JSON file (per-library or global)
- Load settings on app startup
- Settings change triggers reactive updates

### Files to modify:
- `src/routes/settings/+page.svelte` — Major expansion
- New `src/lib/settings/store.ts` — Settings store with persistence
- `src-tauri/src/libraries.rs` — Settings read/write commands

---

## Phase 10: Polish & Quality of Life
**Priority: Low | Effort: Small-Medium | Impact: Medium**

### 10.1 Random Note
- Command/button to open a random note from the library
- Ribbon icon

### 10.2 Word Count Enhancement
- Show reading time estimate
- Show word count for selection

### 10.3 Note Info
- Show file path, creation date, modification date
- File size
- Link count, backlink count

### 10.4 Ribbon (Icon Bar)
- Vertical icon bar on far left
- Quick access icons: new note, open library, search, graph, daily note, command palette
- Configurable: show/hide icons, reorder

### 10.5 Table Editing
- Right-click context menu in tables
- Add/remove rows and columns
- Move rows/columns
- Format table (auto-align pipes)
- Tab key navigation between cells

### 10.6 URI Protocol
- Register `constellation://` URI scheme via Tauri
- Support: open note, new note, search, daily note
- Deep linking into specific notes

### 10.7 Drag & Drop External Files
- Drag files from system file manager into note
- Images → save as attachment + insert `![[image]]`
- Markdown files → import into library
- Other files → save as attachment

### 10.8 Clipboard Image Paste
- Ctrl+V with image in clipboard
- Save image to attachment folder
- Insert `![[image-timestamp.png]]` at cursor

### 10.9 Export Options
- Export note as PDF
- Export note as HTML
- Export library as ZIP

---

## Implementation Priority Matrix

| Phase | Priority | Effort | Dependencies | Status |
|-------|----------|--------|-------------- |--------|
| 1. Markdown Rendering | Critical | Medium | None | Not Started |
| 2. Editor Enhancements | Critical | Large | None | Not Started |
| 3. Advanced Linking | High | Medium | Phase 1 | Not Started |
| 4. Built-in Features | High | Large | Phase 2, 3 | Not Started |
| 5. File Management | Medium | Medium | None | Not Started |
| 6. Appearance & Theming | Medium | Medium | None | Not Started |
| 7. Keyboard Shortcuts | Medium | Medium | Phase 4 | Not Started |
| 8. Advanced Features | Low | Large | Phase 1-7 | Not Started |
| 9. Settings & Config | Medium | Medium | Phase 6, 7 | Not Started |
| 10. Polish & QoL | Low | Small-Med | Various | Not Started |

---

## Recommended Implementation Order

### Sprint 1 (Foundation) — Phases 1 + 6.1
- Complete Markdown rendering (callouts, highlights, math, mermaid, embeds)
- Add dark mode support
- **Why first**: Everything else builds on proper rendering

### Sprint 2 (Editor) — Phase 2
- Link autocomplete, smart lists, tab handling, find/replace
- **Why second**: Core editing experience is critical for usability

### Sprint 3 (Navigation) — Phases 3 + 4.8
- Backlinks, outgoing links, aliases, link auto-update
- Page preview on hover
- **Why third**: Knowledge graph features are a core differentiator

### Sprint 4 (Discovery) — Phases 4.1-4.5
- Sky View, command palette, quick switcher, daily notes, templates
- **Why fourth**: These are the most-used knowledge management features

### Sprint 5 (Organization) — Phases 4.6-4.10 + 5
- Bookmarks, tags view, note composer, file drag/drop, sorting
- **Why fifth**: Organization features build on discovery features

### Sprint 6 (Customization) — Phases 6 + 7 + 9
- Theming, CSS snippets, hotkey system, comprehensive settings
- **Why sixth**: Customization is important but not blocking

### Sprint 7 (Advanced) — Phases 8 + 10
- Canvas, workspaces, vim mode, import system, bases, polish
- **Why last**: These are large features that can be added incrementally

---

## Technical Decisions Needed

1. **Editor upgrade**: The current `<textarea>` will hit limits with line numbers, syntax highlighting, multiple cursors, and Vim mode. Consider migrating to **CodeMirror 6** for the editor component. This would unlock:
   - Syntax highlighting in edit mode
   - Line numbers
   - Multiple cursors
   - Vim mode
   - Code folding
   - Custom decorations (inline rendering)
   - Better performance for large files

2. **Graph library**: For the Sky View, evaluate:
   - **D3.js force-layout** — Most flexible, but complex
   - **Cytoscape.js** — Purpose-built for graphs, good performance
   - **vis.js Network** — Easy to use, good interactivity

3. **Math rendering**: Choose between:
   - **KaTeX** — Faster, smaller bundle, stricter LaTeX
   - **MathJax** — More complete LaTeX support, larger bundle

4. **Canvas implementation**: For the Canvas feature, evaluate:
   - **Fabric.js** — Full-featured canvas library
   - **Custom SVG** — Lighter weight, good for connections
   - **Konva.js** — Good performance with layers

5. **Database views (Bases)**: This is essentially building a query engine. Consider:
   - Building a simple property-based filter/sort system first
   - Using SQL.js (SQLite in WASM) for complex queries
   - Starting with Table view only, adding other layouts later

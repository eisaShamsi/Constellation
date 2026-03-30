# Competitive Feature Reference

> Reference catalog of knowledge management features across the industry.
> Used as a benchmark for Constellation's feature development.
> Sourced from competitor documentation (March 2026)

---

## TABLE OF CONTENTS

1. [Core Architecture](#1-core-architecture)
2. [Editing and Formatting](#2-editing-and-formatting)
3. [Linking System](#3-linking-system)
4. [File and Library Management](#4-file-and-library-management)
5. [User Interface](#5-user-interface)
6. [Core Plugins](#6-core-plugins)
7. [Bases (Database Views)](#7-bases-database-views)
8. [Extensibility System](#8-extensibility-system)
9. [Sync Service](#9-sync-service)
10. [Publish Service](#10-publish-service)
11. [Web Clipper](#11-web-clipper)
12. [CLI](#12-cli)
13. [Settings System](#13-settings-system)

---

## 1. CORE ARCHITECTURE

### 1.1 Library System
- A library is a folder on the local filesystem containing notes, attachments, and a configuration folder
- Libraries are fully local-first; all data stored as plain files
- Multiple libraries supported, switched via Library Switcher
- Create new library or open existing folder as library
- Rename, move, or remove libraries from the library list
- Transfer settings between libraries
- Configuration stored in `.obsidian/` folder within each library

### 1.2 How Obsidian Stores Data
- Notes stored as plain Markdown (`.md`) files
- Canvas files stored as JSON (`.canvas`) using the JSON Canvas spec
- Bases stored as `.base` files
- Configuration folder (`.obsidian/`) contains:
  - `app.json` - Editor and app settings
  - `appearance.json` - Theme and appearance settings
  - `community-plugins.json` - Installed community plugins list
  - `core-plugins.json` - Enabled core plugins
  - `core-plugins-migration.json` - Migration state
  - `hotkeys.json` - Custom hotkey mappings
  - `workspace.json` - Current workspace layout
  - Plugins folder with per-plugin settings
  - Themes folder
  - Snippets folder for CSS snippets

### 1.3 Accepted File Formats
- **Markdown**: `.md`
- **Bases**: `.base`
- **JSON Canvas**: `.canvas`
- **Images**: `.avif`, `.bmp`, `.gif`, `.jpeg`, `.jpg`, `.png`, `.svg`, `.webp`
- **Audio**: `.flac`, `.m4a`, `.mp3`, `.ogg`, `.wav`, `.webm`, `.3gp`
- **Video**: `.mkv`, `.mov`, `.mp4`, `.ogv`, `.webm`
- **PDF**: `.pdf`
- Extensible via community plugins

### 1.4 Symbolic Links and Junctions
- Support for symlinks and junctions to reference external folders/files inside a library

---

## 2. EDITING AND FORMATTING

### 2.1 Basic Formatting Syntax (Markdown)
- **Paragraphs**: Blank line separates paragraphs
- **Line breaks**: Trailing double space or `<br>` for line breaks within a paragraph
- **Headings**: 6 levels (`# H1` through `###### H6`)
- **Bold**: `**text**` or `__text__`
- **Italic**: `*text*` or `_text_`
- **Bold + Italic**: `***text***` or `___text___`
- **Highlights**: `==highlighted text==`
- **Strikethrough**: `~~text~~`
- **Internal links**: `[[note]]` wiki-link syntax
- **External links**: `[text](url)` standard Markdown links
  - Escape blank spaces in links with `%20`
- **External images**: `![alt](url)`
- **Blockquotes**: `> quote text` (nestable)
- **Ordered lists**: `1. item`
- **Unordered lists**: `- item` or `* item`
- **Task lists**: `- [ ] unchecked` / `- [x] checked`
- **Nested lists**: Indent with spaces/tabs
- **Horizontal rule**: `---`, `***`, or `___`
- **Inline code**: backtick syntax
- **Code blocks**: Triple backtick with optional language identifier for syntax highlighting
- **Nested code blocks**: Use more backticks (4+) for nesting
- **Footnotes**: `[^ref]` inline, `[^ref]: text` for definition
- **Comments**: `%% comment %%` (Obsidian-specific, hidden in preview)
- **Escaping Markdown**: Backslash `\` before special characters

### 2.2 Advanced Formatting Syntax
- **Tables**: Pipe `|` and hyphen `-` syntax
  - Right-click context menu in Live Preview for adding/removing columns/rows
  - Sort and move columns/rows via context menu
  - Insert Table command from Command Palette
  - Format content within tables (links, code, bold, etc.)
- **Diagrams**: Mermaid.js integration via code blocks (` ```mermaid `)
  - Support linking files within diagrams
- **Math / LaTeX**: MathJax rendering
  - Inline math: `$formula$`
  - Block math: `$$formula$$`

### 2.3 Obsidian Flavored Markdown
- Based on CommonMark + GitHub Flavored Markdown + LaTeX
- Additions beyond standard Markdown:
  - `[[Wikilinks]]`
  - `![[Embed files]]`
  - `==Highlights==`
  - `%%Comments%%`
  - `> [!callout]` callout blocks
  - Block references `^block-id`
  - Tags `#tag`
  - Properties (YAML frontmatter)

### 2.4 Callouts
- Syntax: `> [!type] Title` with content below
- Customizable title
- **Foldable callouts**: `> [!type]+` (default open) or `> [!type]-` (default closed)
- **Nested callouts** supported
- **Custom callouts** via CSS snippets
- Insert callout command in Command Palette
- Wrap existing selected content in a callout
- Right-click to change callout type in Live Preview
- **Supported types** (each with unique icon and color):
  - `note`
  - `abstract` (aliases: `summary`, `tldr`)
  - `info`
  - `todo`
  - `tip` (aliases: `hint`, `important`)
  - `success` (aliases: `check`, `done`)
  - `question` (aliases: `help`, `faq`)
  - `warning` (aliases: `caution`, `attention`)
  - `failure` (aliases: `fail`, `missing`)
  - `danger` (alias: `error`)
  - `bug`
  - `example`
  - `quote` (alias: `cite`)

### 2.5 Tags
- Inline tags: `#tagname` anywhere in note body
- YAML tags: `tags:` property in frontmatter (list format)
- **Nested tags**: `#parent/child/grandchild` hierarchy
- Tag format rules: Letters, numbers, underscore, hyphen, forward slash; must contain a non-numeric character
- Find notes via Search plugin with `tag:` operator
- Tags view plugin shows all tags with counts
- Click tag to search, Ctrl/Cmd+click to toggle in search

### 2.6 Properties (Frontmatter)
- YAML frontmatter block at top of note between `---` delimiters
- Add via Command Palette, hotkey (`Ctrl/Cmd+;`), or three-dot menu
- **Property Types**:
  - **Text**: Plain string values
  - **List**: Array of values
  - **Number**: Numeric values
  - **Checkbox**: Boolean true/false
  - **Date**: Date values (YYYY-MM-DD)
  - **Date & Time**: DateTime values
  - **Tags**: Tag values in properties
- **Display Modes**: Source view or rendered properties view
- Search properties with `[property:value]` syntax
- Use in templates
- Rename properties globally across library
- CSS snippets can style properties
- **Default properties**: `tags`, `aliases`, `cssclasses`
- **Publish properties**: `publish`, `permalink`, `description`, `image`, `cover`
- JSON property support for complex data

### 2.7 Folding
- Fold/collapse headings to hide content beneath them
- Fold indented content
- Fold settings: "Fold heading" and "Fold indent" in Settings > Editor

### 2.8 Multiple Cursors
- Hold `Alt` (or `Option` on macOS) and click to add additional cursors
- Edit text at multiple positions simultaneously
- **Rectangular selection**: Hold `Alt+Shift` and drag

### 2.9 Views and Editing Mode
- **Reading View**: Rendered Markdown display (non-editable)
- **Editing View** with two modes:
  - **Live Preview**: WYSIWYG-like editing; renders Markdown in real-time while editing
  - **Source Mode**: Plain text Markdown editing, shows all syntax
- Toggle between views via Command Palette, hotkey, or status bar icon
- Default view and editing mode configurable in Settings

### 2.10 Editing Shortcuts
- Platform-specific shortcuts (Windows/Linux vs macOS)
- **Common actions**: Bold, italic, link, internal link, undo, redo
- **Text editing**: Cut line, copy line, delete line, delete word
- **Text navigation**: Move by word, move to line start/end, move to document start/end
- **Text selection**: Select word, select line, select all, extend selection
- **Text formatting**: Toggle bold/italic/highlight/strikethrough/code/comment

### 2.11 Embed Web Pages
- `<iframe>` HTML element support for embedding web pages
- Embed YouTube videos
- Embed tweets

### 2.12 HTML Content
- Sanitized HTML supported in notes
- `<script>` elements stripped for security
- **Limitations**: No Markdown rendering inside HTML elements
- Common usage: Comments (`<!-- -->`), underline (`<u>`), span/div with classes
- Strikethrough via `<s>` tag

### 2.13 Attachments
- Import supported file formats into library
- Configurable default attachment location:
  - Library root folder
  - Subfolder within library
  - Same folder as current note
  - Subfolder under current note's folder

---

## 3. LINKING SYSTEM

### 3.1 Internal Links
- **Wikilink format**: `[[Note Name]]` or `[[Note Name.md]]`
- **Markdown format**: `[Display Text](Note%20Name)` or `[Display Text](Note%20Name.md)`
- **Link to heading**: `[[Note#Heading]]`
- **Link to block**: `[[Note#^block-id]]`
  - Block IDs: Alphanumeric characters auto-generated or manually created
  - Any paragraph, list item, table row, etc. can be a block
- **Change display text**: `[[Note|Custom Display Text]]` (wikilink) or `[Custom Text](Note)` (markdown)
- **Auto-update**: Obsidian automatically updates internal links when renaming files (configurable)
- **Link suggestions**: Autocomplete dropdown when typing `[[`
- **Preview linked file**: Hover over link to see preview (with Page Preview plugin)

### 3.2 Aliases
- Define alternative names for notes in frontmatter: `aliases: [Name1, Name2]`
- Link using alias: Appears in link autocomplete suggestions
- Alias shown in link suggestion with arrow indicator
- Find unlinked mentions for aliases

### 3.3 Embedding Files
- Syntax: `![[filename]]` (exclamation mark before internal link)
- **Embed notes**: `![[Note]]` -- renders full note content inline
- **Embed headings**: `![[Note#Heading]]`
- **Embed blocks**: `![[Note#^block-id]]`
- **Embed images**: `![[image.png]]` with optional size `![[image.png|640x480]]` or `![[image.png|640]]`
- **Embed audio**: `![[audio.mp3]]` -- renders audio player
- **Embed video**: Renders video player
- **Embed PDF**: `![[document.pdf]]` with optional page `![[document.pdf#page=3]]`
- **Embed lists**: Embed specific list items from other notes
- **Embed search results**: `\`\`\`query` code block syntax
- **Drag and drop**: Drag supported files into note to embed automatically (desktop)

---

## 4. FILE AND LIBRARY MANAGEMENT

### 4.1 Note Management
- **Create note**: `Ctrl+N` / `Cmd+N`, or via File Explorer, or Command Palette
- **Rename note**: Click title, or F2 in File Explorer, or via Command Palette
- **Delete note**: Via File Explorer context menu, Command Palette, or three-dot menu
- Deleted files go to system trash, Obsidian trash (`.trash/`), or permanent delete (configurable)
- Confirm deletion option

### 4.2 Library Management
- **Create new library**: Empty library with default settings
- **Create library from existing folder**: Open any folder as a library
- **Rename library**: Only renames the folder on disk
- **Move library**: Move to a different folder location
- **Remove library**: Remove from library list (does not delete files)
- **Transfer settings**: Copy `.obsidian/` configuration between libraries
- **Library Switcher**: UI for managing all libraries

### 4.3 File Explorer (Core Plugin)
- Tree view of all files and folders in sidebar
- Create, rename, delete files and folders
- Drag and drop to move files
- Sort by file name, modification date
- Context menu operations
- Reveal file in system file manager

### 4.4 File Recovery (Core Plugin)
- Periodic snapshots of notes
- Recover previous versions of notes
- Configurable snapshot interval and retention period

---

## 5. USER INTERFACE

### 5.1 Workspace
- Main container for all UI components
- Consists of: Ribbon, Sidebars (left/right), Tab groups, Status bar
- Layout persists between sessions
- Different layout on desktop vs. mobile

### 5.2 Tabs
- Open unlimited tabs
- **Arrange tabs**: Drag to reorder within or between tab groups
- **Split tab groups**: Horizontal or vertical split
- **Resize tab groups**: Drag divider between groups
- **Move tab to new window**: Pop out into separate window
- **Move tab between windows**
- **Pin tabs**: Pinned tabs stay when opening new links
- **Switch tabs**: `Ctrl+Tab` / `Ctrl+Shift+Tab` to cycle
- **Stack tab groups**: Stacked tabs show as a vertical list
- **Linked views**: Link a tab to another so it follows along (e.g., outline linked to editor)
- **Save layouts**: Save and restore window/tab arrangements via Workspaces plugin

### 5.3 Sidebar
- Left sidebar and right sidebar
- Toggle visibility of each sidebar independently
- **Tabs**: Each sidebar contains tabs from plugins (File Explorer, Search, Bookmarks, etc.)
- **Open/reopen tabs** in sidebar
- **Close tabs** in sidebar
- **Rearrange tabs**: Drag to reorder
- **Pin tabs** in sidebar
- **Tab groups in sidebar**: Create multiple tab groups within a sidebar
- Mobile-specific sidebar behavior

### 5.4 Ribbon
- Vertical icon bar on the far left
- Quick access to common actions
- Configurable: show/hide ribbon, reorder icons
- Right-click to configure which icons appear
- Plugins can add ribbon icons

### 5.5 Status Bar
- Horizontal bar at the bottom of the app
- Shows information: word count, character count, editing mode
- Plugins can add status bar items

### 5.6 Hotkeys (Keyboard Shortcuts)
- Fully customizable keyboard shortcuts for all commands
- Assign multiple hotkey combinations to a single command
- Remove existing hotkey assignments
- View hotkeys in Command Palette or Settings > Hotkeys
- Search/filter commands in hotkey settings
- Conflict detection for duplicate assignments
- Reset hotkeys to defaults
- Default hotkeys provided for common operations

### 5.7 Appearance
- **Color scheme**: Light, Dark, or Adapt to system
- **Accent color**: Customizable accent color
- **Custom themes**: Community theme marketplace
- **Custom app icon**: Change the Obsidian icon
- **Translucent window**: Semi-transparent window (desktop)
- **Font customization**:
  - Interface font
  - Text (editor) font
  - Monospace font
  - Font size
  - Quick font size adjustment (Ctrl+scroll)
- **Interface options**:
  - Show/hide inline title
  - Show/hide tab title bar
  - Show/hide ribbon
  - Ribbon menu configuration
- **Advanced**:
  - Zoom level
  - Native menus vs custom menus
  - Window frame style (native vs. hidden)
  - Hardware acceleration toggle
- **CSS snippets**: Apply custom CSS on top of theme

### 5.8 Drag and Drop
- **Drag tabs**: Rearrange, split, or move to new windows
- **Drag sources**: Files, folders, content, links
- **Drop destinations**: Editor (creates link or embed), File Explorer (moves files), sidebar
- **Drag from outside Obsidian**: Import files by dragging from system file manager
- **Drop files outside Obsidian**: Export/move files out

### 5.9 Pop-out Windows
- Open notes in separate floating windows
- Move tabs between main window and pop-out windows
- Multiple pop-out windows supported

### 5.10 Settings
- Dedicated settings panel with search
- Organized into sections: General, Editor, Files and links, Appearance, Hotkeys, Core plugins, Community plugins

### 5.11 Language Settings
- Multiple language support for the interface
- Spellcheck with configurable languages

---

## 6. CORE PLUGINS

Each core plugin can be individually enabled or disabled.

### 6.1 Search
- Full-text search across entire library
- Opens in left sidebar by default (`Ctrl+Shift+F`)
- **Search terms**: Words, phrases (quoted), logical operators (OR)
- **Search operators**:
  - `file:` -- match filename
  - `path:` -- match file path
  - `content:` -- match only file content (not filename)
  - `tag:` -- match tags
  - `line:` -- match multiple terms on same line
  - `block:` -- match multiple terms in same block
  - `section:` -- match multiple terms in same section
  - `task:` -- match tasks; `task-todo:` (unchecked), `task-done:` (checked)
- **Search properties**: `[property:value]` syntax
- **Case sensitivity**: Toggle case-sensitive search
- **Sort order**: Change result sorting (relevance, filename, modified date, created date)
- **Copy search results**: Copy results as Markdown links
- **Regular expressions**: Toggle regex search mode
- **Configure search settings**: Match case, explain search term, collapse results, extra context, sort order
- **Embed search results**: Use ` ```query ` code block to embed live search results in notes

### 6.2 Sky View
- Visualize note relationships as an interactive node graph
- Nodes = notes, Edges = internal links
- Node size proportional to number of connections
- **Interaction**: Hover to highlight connections, click to open note, right-click for context menu
- **Navigation**: Scroll to zoom, drag to pan, arrow keys, +/- keys
- **Settings**:
  - **Filters**: Show/hide tags, attachments, existing notes only, orphans; filter by search query
  - **Groups**: Color-code nodes by search query (e.g., all notes with tag X in one color)
  - **Display**: Toggle arrows, text labels; adjust node size, link thickness, font size
  - **Forces**: Adjust center force, repel force, link force, link distance
- **Time-lapse animation**: Animate graph growth over time
- **Local Graph**: View connections for a single note (reduced scope)

### 6.3 Backlinks
- Shows all notes that link TO the current note
- Displays in sidebar panel
- Shows context around each backlink
- **Linked mentions**: Explicit links to current note
- **Unlinked mentions**: Text that matches the note name but is not yet linked
- Convert unlinked mention to a link with one click
- Filter results
- Collapse/expand results
- Show more context

### 6.4 Outgoing Links
- Shows all notes that the current note links TO
- Displays in sidebar panel
- **Linked mentions**: Notes explicitly linked from current note
- **Unlinked mentions**: Notes whose names appear in text but are not linked
- Convert unlinked mentions to links

### 6.5 Templates
- Insert pre-defined content snippets into notes
- Template folder configurable in settings
- **Template variables**:
  - `{{title}}` -- current note title
  - `{{date}}` -- current date (format configurable)
  - `{{time}}` -- current time (format configurable)
- Insert template via Command Palette or hotkey
- Date and time format customizable (Moment.js format strings)

### 6.6 Command Palette
- Quick-access command launcher (`Ctrl+P` / `Cmd+P`)
- Search and execute any command
- Shows associated hotkeys
- Pinned commands appear at top
- Access to all core and plugin commands

### 6.7 Quick Switcher
- Rapidly open any note by typing its name (`Ctrl+O` / `Cmd+O`)
- Fuzzy search matching
- Create new note if no match found
- Shows file path for disambiguation

### 6.8 Bookmarks
- Bookmark items for quick access
- **Bookmarkable items**: Files, folders, graphs, searches, headings, blocks, links
- **Bookmark groups**: Organize bookmarks into groups/folders
- Drag to reorder bookmarks
- Bookmark multiple files from File Explorer
- Open bookmarks from sidebar panel

### 6.9 Canvas
- Infinite, freeform visual workspace
- **Card types**:
  - Text cards (standalone content)
  - Note cards (linked to existing notes)
  - Media cards (images, video, audio)
  - Web page cards (embed URLs)
  - Folder cards (show folder contents)
- **Card operations**: Edit, delete, swap content, resize, arrange
- **Select cards**: Click, Ctrl+click for multi-select, drag to select area
- **Arrange**: Align, distribute, stack cards
- **Connections**:
  - Draw arrows/lines between cards
  - Labels on connections
  - Change connection colors
  - Navigate connections
  - Disconnect cards
- **Groups**: Group cards together visually with a background region
- **Canvas navigation**: Pan (drag/arrow keys), zoom (scroll/+/-), zoom to fit, zoom to selection, reset zoom
- **Colors**: Assign colors to cards and connections
- **Advanced**: Keyboard shortcuts, drag to add content, narrow-to-block editing

### 6.10 Daily Notes
- Automatically create or open a note for today's date
- Configurable date format for filenames
- Configurable default folder for daily notes
- Template support for daily notes
- Open on startup option

### 6.11 Audio Recorder
- Record audio directly within Obsidian
- Save audio recording as attachment in current note

### 6.12 File Explorer
- Sidebar panel showing library file/folder tree
- Create, rename, delete, move files and folders
- Drag and drop support
- Context menu with full operations
- Sort files by name or date
- Reveal active file in explorer
- Create new note or folder
- Show/hide file extensions

### 6.13 File Recovery
- Periodic snapshots of note content
- Browse and restore previous versions
- Configurable snapshot interval
- Configurable retention period

### 6.14 Format Converter
- Convert Markdown from other apps to Obsidian format
- Supported source formats: Roam Research, Bear, Zettelkasten
- Convert properties to current format
- Library-wide batch conversion

### 6.15 Note Composer
- **Merge notes**: Combine two notes into one, update all links
- **Extract note**: Select text and extract to a new note, leaving a link behind
- **Template file**: Use template when extracting notes

### 6.16 Outline
- Sidebar panel showing heading structure of active note
- Click heading to jump to that section
- Hierarchical display reflecting heading levels

### 6.17 Page Preview
- Hover over internal link to see a popup preview of the linked note
- Preview shows rendered content
- Works in editor and in reading view
- `Ctrl+hover` (configurable) to trigger preview

### 6.18 Properties View
- Dedicated sidebar panel for viewing/editing note properties
- Shows all properties across the library
- Browse properties by name
- View all values for a given property

### 6.19 Random Note
- Open a random note from the library
- Accessible via Command Palette or ribbon icon

### 6.20 Slash Commands
- Type `/` in editor to trigger command suggestions
- Quick inline access to commands without opening Command Palette

### 6.21 Slides
- Present notes as slideshows
- Use `---` (horizontal rule) to separate slides
- Powered by reveal.js
- Presentation mode with navigation controls

### 6.22 Tags View
- Sidebar panel listing all tags in library
- Shows tag counts (number of notes per tag)
- Click tag to search
- Ctrl/Cmd+click to toggle in search
- Nested tag hierarchy display

### 6.23 Unique Note Creator
- Create notes with timestamp-based names (Zettelkasten-style)
- Format: `YYYYMMDDHHmm` by default
- Template support for unique notes
- Configurable naming format and folder

### 6.24 Web Viewer
- Open external links within Obsidian (desktop only)
- **Reader view**: Simplified reading mode for web pages
- **Save to library**: Save web content as Markdown note
- **Ad blocking** support
- **Security**: Sandboxed browsing

### 6.25 Word Count
- Display word count and character count in status bar
- Updates in real-time while editing

### 6.26 Workspaces
- Save and restore complete window/tab layouts
- Multiple named workspace configurations
- Switch between workspaces quickly
- Saves: open tabs, tab arrangement, split panes, sidebar state

---

## 7. BASES (DATABASE VIEWS)

### 7.1 Overview
- Database-like views of notes based on their properties
- Each base can have one or more views
- File format: `.base`

### 7.2 Creating Bases
- Create via Command Palette, File Explorer, or right-click menu
- **Embed a base**: Embed as file link (`![[mybase.base]]`) or as code block in notes

### 7.3 Views
- **Toolbar**: Controls for view management, filters, sorts
- **Add and switch views**: Multiple views per base
- **View settings**: Configure each view independently
- **Layout types**:
  - **Table view**: Spreadsheet-like grid of properties
  - **List view**: Simple list format
  - **Cards view**: Card/kanban-style layout
  - **Map view**: Geographic map based on location properties
- **Filters**:
  - Filter by property values
  - Filter components: field, operator, value
  - Conjunctions: AND, OR
  - Filter groups for complex logic
  - Advanced filter editor
- **Sort and group**:
  - Sort by any property (ascending/descending)
  - Multiple sort levels
  - Group results by property
- **Limit results**: Cap the number of displayed results
- **Copy to clipboard**: Copy view data
- **Export CSV**: Export base data as CSV
- **Embed a view**: Embed specific views in notes

### 7.4 Bases Syntax
- Query syntax for selecting and filtering notes
- Column definitions and property references

### 7.5 Functions
- Built-in functions for computed fields
- Data manipulation and aggregation

### 7.6 Formulas
- Formula system for calculated properties
- Reference other properties in formulas

---

## 8. EXTENSIBILITY SYSTEM

### 8.1 Community Plugins
- Third-party plugin marketplace
- Browse, install, enable, update, uninstall plugins
- **Restricted mode**: Disable all community plugins for security
- Manage installed plugins list
- Each plugin has its own settings panel
- Plugin security review process

### 8.2 Themes
- Community theme marketplace
- Browse, install, update, uninstall themes
- One active theme at a time
- Themes override CSS variables

### 8.3 CSS Snippets
- Small CSS files for custom styling
- Stored in `.obsidian/snippets/` folder
- Toggle individual snippets on/off
- Stack on top of active theme
- Can target any part of the UI

### 8.4 Obsidian URI Protocol
- Custom `obsidian://` URL scheme for deep linking
- **Actions**:
  - `obsidian://open` -- Open a note (by library, file, path)
  - `obsidian://new` -- Create a new note with optional content
  - `obsidian://daily` -- Open or create today's daily note
  - `obsidian://search` -- Open search with a query
  - `obsidian://library` -- Open the library manager
- **Parameters**: library, file, path, content, append, overwrite, heading, block
- x-callback-url support
- Shorthand formats for convenience
- Integration with Hook and other apps

### 8.5 Obsidian CLI
- Command-line interface for terminal-based library interaction
- **General commands**: help, version, reload, restart
- **File operations**: create, read, append, prepend, move, rename, delete, open
- **File listing**: file, files, folder, folders
- **Links**: backlinks, links, unresolved, orphans, deadends
- **Search**: search, search:context, search:open
- **Daily notes**: daily, daily:path, daily:read, daily:append, daily:prepend
- **Properties**: aliases, properties, property:set, property:remove, property:read
- **Tags**: tags, tag
- **Tasks**: tasks, task
- **Templates**: templates, template:read, template:insert
- **Bookmarks**: bookmarks, bookmark
- **Outline**: outline
- **Plugins**: plugins, plugin:enable, plugin:disable, plugin:install, plugin:uninstall, plugin:reload
- **Bases**: bases, base:views, base:create, base:query
- **Themes/Snippets**: themes, theme:set, theme:install, theme:uninstall, snippets, snippet:enable, snippet:disable
- **File history**: diff, history, history:list, history:read, history:restore, history:open
- **Sync commands**: sync, sync:status, sync:history, sync:read, sync:restore, sync:open, sync:deleted
- **Publish commands**: publish:site, publish:list, publish:status, publish:add, publish:remove, publish:open
- **Random**: random, random:read
- **Command palette**: commands, command, hotkeys, hotkey
- **Library management**: library:list, library:path, library:open, library:config
- **Developer**: devtools, console, screenshot
- TUI (terminal user interface) mode
- Target specific libraries and files
- Copy output support
- Parameters and flags system

### 8.6 Obsidian Headless
- Run Obsidian without a GUI (headless mode)
- For automation and scripting

---

## 9. SYNC SERVICE (Obsidian Sync)

### 9.1 Overview
- End-to-end encrypted cloud sync
- Sync notes across any device and OS
- Not dependent on third-party cloud storage

### 9.2 Features
- Local and remote library pairing
- Selective syncing (choose what to sync)
- **Sync settings**: Sync app settings, hotkeys, themes, CSS snippets, plugins
- **Version history**: Browse and restore previous versions of notes
- **Status icon and messages**: Visual sync status in status bar
- **Collaborate**: Share libraries with team members
- **Plans and storage limits**: Different tiers
- **Security and privacy**: End-to-end encryption, zero-knowledge
- **Sync regions**: Choose data center region
- **Upgrade encryption**: Migrate to newer encryption
- **Headless sync**: Sync without GUI

---

## 10. PUBLISH SERVICE (Obsidian Publish)

### 10.1 Overview
- Publish notes as a website/wiki/knowledge base
- Hosted by Obsidian

### 10.2 Features
- **Set up**: Configure site from within Obsidian
- **Manage sites**: Multiple publish sites
- **Customize site**: Theme, navigation, components
- **Publish content**: Select which notes to publish
- **Collaborate**: Multiple contributors to a Publish site
- **Social media link previews**: Open Graph metadata
- **Media files**: Images, audio, video hosting
- **Analytics**: Basic traffic analytics
- **Custom domains**: Use your own domain name
- **Permalinks**: Custom URL paths
- **SEO**: Search engine optimization features
- **Security and privacy**: Password protection, access control
- **Publish limitations**: Documented constraints

---

## 11. WEB CLIPPER

### 11.1 Overview
- Browser extension for saving web content to Obsidian
- Available for major browsers

### 11.2 Features
- **Clip web pages**: Save full page or selection as Markdown
- **Highlight web pages**: Highlight and annotate before saving
- **Interpret web pages**: AI-assisted content extraction
- **Templates**: Customizable clip templates
- **Variables**: Dynamic variables in templates
- **Filters**: Process clipped content with filters
- **Logic**: Conditional logic in clip templates

---

## 12. IMPORT SYSTEM

### 12.1 Importer Plugin
- Dedicated import tool for migrating from other apps

### 12.2 Supported Sources
- Apple Notes
- Bear
- Craft
- Evernote
- Google Keep
- Microsoft OneNote
- Notion
- Roam Research
- Apple Journal
- CSV files
- HTML files
- Markdown files
- Textbundle files
- Zettelkasten notes

---

## 13. SETTINGS SYSTEM

### 13.1 General Settings
- Version and updates
- Language selection
- Help links
- Account management
- Advanced: Startup notification

### 13.2 Editor Settings
- **Tab behavior**: Always focus new tabs, default view for new tabs, default editing mode
- **Display**:
  - Readable line length (max width)
  - Strict line breaks
  - Properties in document display mode
  - Fold heading
  - Fold indent
  - Show line numbers
  - Show indentation guides
  - Right-to-left (RTL) text support
  - Auto-pair brackets
  - Auto-pair Markdown syntax
  - Smart lists (auto-continue lists)
  - Indent using tabs vs spaces
  - Convert pasted HTML to Markdown
- **Behavior**:
  - Spellcheck toggle
  - Spellcheck languages
  - Indent visual width
- **Advanced**:
  - Vim key bindings (full Vim mode)

### 13.3 Files and Links Settings
- Default location for new notes (root, current folder, specific folder)
- Default location for new attachments
- **Links**:
  - New link format (shortest path, relative path, absolute path)
  - Automatically update internal links on rename
  - Use Wikilinks vs Markdown links
  - Show all file types in link suggestions
- **Trash**:
  - Confirm file deletion
  - Deleted files destination (system trash, Obsidian trash, permanent)
- **Advanced**:
  - Excluded files (pattern-based exclusion)
  - Override config folder location
  - Allow URI callbacks
  - Rebuild library cache

### 13.4 Appearance Settings
- Base color scheme (light/dark/system)
- Accent color picker
- Theme selection and management
- **Font**:
  - Interface font
  - Text font
  - Monospace font
  - Font size
  - Quick font size adjustment
- **Interface**:
  - Show inline title
  - Show tab title bar
  - Show ribbon
  - Ribbon menu configuration
- **Advanced**:
  - Zoom level
  - Native menus
  - Window frame style
  - Custom app icon
  - Translucent window
  - Hardware acceleration
- CSS snippets management

### 13.5 Hotkeys Settings
- List all commands with assigned hotkeys
- Search/filter commands
- Add, modify, remove hotkey assignments
- Conflict detection

### 13.6 Core Plugins Settings
- Toggle each core plugin on/off
- Per-plugin settings pages

### 13.7 Community Plugins Settings
- Restricted mode toggle
- Browse community plugins
- Manage installed plugins
- Per-plugin settings pages

---

## PLATFORM SUPPORT

- **Desktop**: Windows, macOS, Linux
- **Mobile**: iOS, iPadOS, Android
- **Tablet**: Optimized layouts
- Mobile app has adapted UI (sidebar behavior, tab management)
- Cross-platform sync via Obsidian Sync or third-party services

---

## FEATURE SUMMARY (Quick Reference)

| Category | Count | Key Features |
|----------|-------|--------------|
| Basic Formatting | 18+ | Headings, bold, italic, highlights, lists, tasks, code, footnotes, comments |
| Advanced Formatting | 3 | Tables, Mermaid diagrams, LaTeX math |
| Callouts | 13 types | Foldable, nestable, customizable callout blocks |
| Linking | 6+ | Wikilinks, Markdown links, heading links, block links, aliases, auto-update |
| Embedding | 7+ | Notes, images, audio, video, PDF, lists, search results |
| Properties | 8 types | Text, list, number, checkbox, date, datetime, tags, JSON |
| Core Plugins | 26 | Search, Graph, Canvas, Bases, Daily Notes, Templates, etc. |
| UI Components | 11+ | Tabs, sidebars, ribbon, status bar, pop-out windows, etc. |
| Extensibility | 6 | Community plugins, themes, CSS snippets, URI protocol, CLI, headless |
| Services | 3 | Sync, Publish, Web Clipper |
| Import Sources | 14 | Apple Notes, Notion, Evernote, OneNote, Roam, etc. |
| Settings | 50+ | Editor, appearance, files, hotkeys, plugins configuration |

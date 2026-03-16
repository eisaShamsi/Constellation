# Constellation

**A Map of Maps**

Constellation is a standalone desktop knowledge management platform that unifies multiple libraries of Markdown files into one coherent interface — without ever merging, moving, or copying your files.

If you maintain separate collections of notes — work, personal, research, projects — you know the pain: switching between windows, losing track of where things are, and never seeing the big picture. Constellation solves this by giving you a single dashboard that reads and writes files in place across all your libraries.

---

## The Problem

Markdown-based note-taking tools are brilliant for individual collections. But when you have many, each becomes an isolated island. There is no way to:

- Search across all libraries at once
- See how ideas connect across library boundaries
- Manage everything from one place

Constellation bridges this gap.

## Core Philosophy

- **You own everything.** Your files stay exactly where they are. Constellation never copies, moves, or uploads anything.
- **All local.** No cloud. No accounts. No telemetry. No tracking. Everything runs on your machine.
- **Read and write in place.** Constellation works directly with your library files. No duplication.
- **Non-destructive.** If you delete Constellation, you lose nothing. Your libraries remain untouched, exactly as they were.

## The Metaphor

Constellation uses a universe-and-stars metaphor to make multi-library navigation intuitive:

| Constellation Term | What It Means |
|---|---|
| **Universe** | A portable data container — holds your libraries, bases, settings, bookmarks, and workspaces in a single directory you own |
| **Library** | A registered folder of Markdown files |
| **Star** | A note (markdown file) |
| **Constellation** | A custom saved view — your curated grouping of stars |
| **Star Line** | A cross-library reference connecting stars across libraries |
| **Sky View** | The unified graph showing all libraries and their connections |

## Features

### Universes
- **First-launch wizard** — Step-by-step setup: welcome → name & locate → add libraries/child universes
- **Create or Open** — Create a new universe or open an existing one (e.g., from another device)
- **Universe Manager** — Switch between, create, or remove universes from the sidebar footer
- **Child Universes** — Reference other universes as children to inherit their libraries automatically
- **Window title** — Shows "Constellation - UniverseName" so you always know which universe is active
- **Fast switching** — Parallel loading with instant UI display; caches rebuild in the background
- **Portable** — Each universe is a self-contained directory you can move, copy, or share

### Tab Navigation
- **Single-tab navigation** — Clicking a note replaces the active tab content, with back/forward history per tab
- **Ctrl+click** opens a new tab explicitly
- **Navigation arrows** appear in the breadcrumb area after navigating to a second note
- **Tab pinning** — Pin tabs to keep them always open
- **Empty tab view** with quick actions: Create new note, Go to file, Close

### Sidebar Toolbar
Four action buttons at the top of the left sidebar:
- **New note** — Creates a new note with library picker (if multiple libraries)
- **New folder** — Creates a new folder with library picker (if multiple libraries)
- **Sort order** — Cycles through: Name A→Z, Name Z→A, Modified newest, Modified oldest
- **Collapse/Expand All** — Toggles all library folder trees open or closed

### Library Management
- Add, remove, and switch between multiple libraries
- Libraries are registered per-universe — each universe has its own library set
- Color-coded libraries for visual distinction
- Library switcher accessible from the footer

### File Operations
- Create, rename, and delete notes and folders
- Move items between folders
- Move items to trash (configurable: system trash, .trash folder, or permanent delete)
- Bookmarks for quick access to frequently used notes

### Cross-Library Linking

Constellation's core differentiator — your `[[wikilinks]]` work across every library you add.

#### Basic Cross-Library Links
- **`[[note]]`** resolves across ALL libraries (current library searched first, then others)
- **`[[library:note]]`** targets a specific library explicitly
- Cross-library links show a dotted underline + arrow indicator
- Autocomplete suggests notes from all libraries, with library name shown
- Hover preview works across library boundaries

#### Alias Resolution
- Notes with `aliases:` in their YAML frontmatter are discoverable by any alias
- Example: a note named "JavaScript" with `aliases: [JS, ECMAScript]` can be linked as `[[JS]]` from any library
- Filename match takes priority; alias lookup runs only when no filename matches

#### Heading & Block References
- **`[[note#heading]]`** — links to a specific heading; clicking scrolls to that section
- **`[[note#^block-id]]`** — links to a specific block by its ID
- Works across libraries: `[[library:note#heading]]`
- Autocomplete: type `[[note#` to see a list of headings in the target note

#### Cross-Library Embeds
- **`![[note]]`** — embeds the full content of another note inline
- **`![[note#heading]]`** — embeds only the section under that heading
- **`![[library:note]]`** — embeds from a specific library
- Embeds nest up to 3 levels deep

#### Typed Links
- **`[[note|type:related-to]]`** — adds a semantic type to the link
- Built-in types: `related-to`, `prerequisite`, `see-also`, `contradicts`, `supports`, `extends`
- Typed links appear with distinct colors in the graph view
- Autocomplete: type `[[note|type:` to see available link types

#### Smart Auto-Linker (Unlinked Mentions)
- The Backlinks panel detects plain-text mentions of the current note name across all libraries
- Expand the "Unlinked Mentions" section to see them
- Click "Link it" to automatically wrap the mention in `[[wikilinks]]`

#### Link Dashboard
- Open from the right sidebar (chain icon tab)
- **Most Connected** — top 10 notes by link count
- **Cross-Library** — all links that cross library boundaries
- **Broken** — links pointing to notes that don't exist
- **Orphans** — notes with no incoming or outgoing links

#### Enhanced Graph View
- Library clusters: notes from the same library are gently grouped together with colored hulls
- Cross-library edges appear as dashed lines with a gradient between library colors
- Typed links are color-coded by type in the graph
- Click a library name in the legend to show/hide its nodes

### Markdown Rendering
- Full markdown rendering with syntax highlighting
- WikiLinks with preview on hover
- Callouts, footnotes, math (KaTeX), Mermaid diagrams
- Highlight syntax (`==text==`)
- Image embeds from library attachments

### Search & Navigation
- Cross-library full-text search
- Search operators (type, tag, property filters)
- Quick switcher (Ctrl+O)
- Command palette (Ctrl+P)
- Keyboard shortcuts for common actions

### Panels
- Properties editor (YAML frontmatter)
- Backlinks panel (with unlinked mentions + auto-link)
- Outgoing links panel
- Tags panel
- Graph view (with library clustering + typed link colors)
- Link Dashboard (cross-library links, broken links, orphans, most connected)
- Calendar panel
- Tasks panel

### Built-in Features
- **Dataview queries** — TABLE, LIST, TASK, CALENDAR queries within notes
- **Tasks** — Library-wide task scanning, toggle, due dates, priority, tags
- **Calendar** — Month view with note/task dots, daily note creation
- **Bases (Databases)** — Table, list, card views with filtering, sorting, formulas
- **Templates** — Template variables (date, time, title, folder, library, cursor)
- **Split panes** — Vertical and horizontal splits
- **Second screen** — Pop-out window for dual-monitor workflows

### Workspaces
- Save and restore window layouts

## AI-Powered

Constellation is AI-powered — but on your terms. Bring your own AI provider and API key. Nothing is sent anywhere without your explicit action.

### Supported Providers

| Provider | Type |
|---|---|
| **OpenAI** (GPT-4o, etc.) | Cloud |
| **Claude** (Anthropic) | Cloud |
| **Google Gemini** | Cloud |
| **Ollama** | Local (runs on your machine) |

### AI Skills

Skills are AI-powered workflows that enhance your notes. Constellation ships with 8 built-in skills, and you can create or install custom ones:

| Skill | What It Does |
|---|---|
| **Summarize Note** | Condense any note into key points |
| **Smart Q&A** | Ask questions about your notes, get AI answers |
| **Writing Assistant** | Expand, rewrite, or improve text |
| **Auto-Linker** | Discover connections between notes across libraries |
| **Translate Note** | Translate between languages |
| **Meeting Notes** | Structure raw meeting notes |
| **Chart Generator** | Create charts from note data |
| **Research Assistant** | Analyze and synthesize across multiple notes |

Skills are extensible — create your own by defining a prompt template, inputs, and output format.

## Importers

Constellation includes built-in importers to bring your notes from other tools:

| Source | What Gets Imported |
|---|---|
| **Markdown folders** | Direct library registration — no conversion needed |
| **Notion exports** | Cleans hex IDs, converts internal links to wikilinks |
| **Bear notes** | Converts Bear's format to standard Markdown |
| **Evernote (.enex)** | Converts ENML to Markdown, preserves tags and dates as frontmatter |
| **HTML files** | Converts to clean Markdown |
| **CSV files** | Each row becomes a note with frontmatter properties |
| **Plain text files** | Direct import with Markdown extension |

## Multi-Language Interface

Constellation supports 15 languages with full RTL support:

| Language | Direction |
|---|---|
| Arabic, Persian, Hebrew, Urdu | RTL |
| English, Spanish, French, German, Portuguese, Russian, Turkish, Hindi, Chinese, Japanese, Korean | LTR |

The entire interface — including all skills — adapts direction, layout, and text based on the selected language.

## Security

Constellation takes security seriously:

- **HTML sanitization** — All rendered markdown is sanitized through DOMPurify before DOM injection, preventing XSS attacks from malicious `.md` files
- **Path containment** — All file operations validate that paths are within registered libraries, preventing path traversal attacks. Read, write, scan, and link-resolution commands all enforce library membership
- **Content Security Policy** — Restrictive CSP prevents unauthorized script execution
- **Mermaid sandboxing** — Diagram rendering uses strict security mode with DOMPurify on SVG output
- **KaTeX sanitization** — Math formula output is sanitized before DOM injection
- **Name sanitization** — File and folder names are validated to reject traversal characters (`..`, `\`, `:`)
- **No shell execution** — The Rust backend uses direct filesystem APIs, never shell commands
- **Minimal Tauri capabilities** — Only `core:default` and `opener:default` permissions; no broad filesystem or HTTP access
- **Daily automated audits** — Scheduled code audits check for memory leaks, security vulnerabilities, and code quality

## Architecture

Constellation is built with modern, performant technologies:

- **[Tauri v2](https://tauri.app)** — Rust-powered desktop framework. Small bundle size (~10 MB), low memory usage, native performance.
- **[Svelte 5](https://svelte.dev)** — Reactive frontend framework with minimal overhead.
- **TypeScript** — Type-safe frontend development.
- **Rust Backend** — File system operations, library indexing, and cross-library reference management handled by Rust for speed and reliability.

### Why Tauri?

| | Tauri | Electron |
|---|---|---|
| Bundle size | ~10 MB | ~150 MB |
| Memory usage | ~30 MB | ~150 MB+ |
| Backend | Rust (native) | Node.js |
| Renderer | System WebView | Bundled Chromium |

## Platform Support

### Desktop (Phase 1 — Current)
- Windows
- macOS
- Linux

### Mobile (Phase 2 — Future)
A companion mobile app is planned for read-only library browsing and search. Full mobile editing will follow in Phase 3.

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org) (v18+)
- npm

### Getting Started

```bash
# Clone the repository
git clone https://github.com/eisaShamsi/Constellation.git
cd Constellation

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Roadmap

1. **Project scaffold** — Tauri + Svelte + TypeScript setup
2. **Library registration** — Add/remove library paths
3. **File browser** — Navigate library contents
4. **Search** — Cross-library full-text search
5. **Graph view** — Unified Sky View with library clustering
6. **Cross-library references** — Star Lines
7. **Plugin system** — TypeScript extension API
8. **Mobile companion** — Read-only library access

## License

MIT License. See [LICENSE](LICENSE) for details.

## Links

- **Website**: [uConstellation.World](https://uConstellation.World)
- **Repository**: [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

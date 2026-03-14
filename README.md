# Constellation

**A Map of Maps**

Constellation is a standalone desktop application that brings all your [Obsidian](https://obsidian.md) vaults together in one unified interface — without ever merging, moving, or copying your files.

If you use multiple Obsidian vaults, you know the pain: switching between windows, losing track of where things are, and never seeing the big picture. Constellation solves this by giving you a single dashboard that sits on top of your vaults, reading and writing files in place.

---

## The Problem

Obsidian is brilliant for individual vaults. But when you have many — work, personal, research, projects — each vault becomes an isolated island. There is no way to:

- Search across all vaults at once
- See how ideas connect across vault boundaries
- Manage everything from one place

Constellation bridges this gap.

## Core Philosophy

- **You own everything.** Your files stay exactly where they are. Constellation never copies, moves, or uploads anything.
- **All local.** No cloud. No accounts. No telemetry. No tracking. Everything runs on your machine.
- **Read and write in place.** Constellation works directly with your vault files. No duplication.
- **Non-destructive.** If you delete Constellation, you lose nothing. Your vaults remain untouched, exactly as they were.

## The Metaphor

Constellation uses a universe-and-stars metaphor to make multi-vault navigation intuitive:

| Constellation Term | What It Means |
|---|---|
| **Universe** | A portable data container — holds your vaults, bases, settings, bookmarks, and workspaces in a single directory you own |
| **Star** | A note (markdown file) |
| **Constellation** | A custom saved view — your curated grouping of stars |
| **Star Line** | A cross-vault reference connecting stars across vaults |
| **Sky View** | The unified graph showing all vaults and their connections |

## Features

### Universes
- **First-launch wizard** — Step-by-step setup: welcome → name & locate → add vaults/child universes
- **Create or Open** — Create a new universe or open an existing one (e.g., from another device)
- **Universe Manager** — Switch between, create, or remove universes from the sidebar footer
- **Child Universes** — Reference other universes as children to inherit their vaults automatically
- **Window title** — Shows "Constellation - UniverseName" so you always know which universe is active
- **Fast switching** — Parallel loading with instant UI display; caches rebuild in the background
- **Portable** — Each universe is a self-contained directory you can move, copy, or share

### Tab Navigation (Obsidian-style)
- **Single-tab navigation** — Clicking a note replaces the active tab content, with back/forward history per tab
- **Ctrl+click** opens a new tab explicitly
- **Navigation arrows** appear in the breadcrumb area after navigating to a second note
- **Empty tab view** with quick actions: Create new note, Go to file, Close

### Sidebar Toolbar
Four action buttons at the top of the left sidebar:
- **New note** — Creates a new note with vault picker (if multiple vaults)
- **New folder** — Creates a new folder with vault picker (if multiple vaults)
- **Sort order** — Cycles through: Name A→Z, Name Z→A, Modified newest, Modified oldest
- **Collapse/Expand All** — Toggles all vault folder trees open or closed

### Vault Management
- Add, remove, and switch between multiple vaults
- Vaults are registered per-universe — each universe has its own vault set
- Color-coded vaults for visual distinction
- Vault switcher accessible from the footer

### File Operations
- Create, rename, and delete notes and folders
- Move items to trash (.trash folder within vault)
- Bookmarks for quick access to frequently used notes

### Cross-Vault Linking

Constellation's core differentiator — your `[[wikilinks]]` work across every vault you add.

#### Basic Cross-Vault Links
- **`[[note]]`** resolves across ALL vaults (current vault searched first, then others)
- **`[[vault:note]]`** targets a specific vault explicitly
- Cross-vault links show a dotted underline + arrow indicator
- Autocomplete suggests notes from all vaults, with vault name shown
- Hover preview works across vault boundaries

#### Alias Resolution
- Notes with `aliases:` in their YAML frontmatter are discoverable by any alias
- Example: a note named "JavaScript" with `aliases: [JS, ECMAScript]` can be linked as `[[JS]]` from any vault
- Filename match takes priority; alias lookup runs only when no filename matches

#### Heading & Block References
- **`[[note#heading]]`** — links to a specific heading; clicking scrolls to that section
- **`[[note#^block-id]]`** — links to a specific block by its ID
- Works across vaults: `[[vault:note#heading]]`
- Autocomplete: type `[[note#` to see a list of headings in the target note

#### Cross-Vault Embeds
- **`![[note]]`** — embeds the full content of another note inline
- **`![[note#heading]]`** — embeds only the section under that heading
- **`![[vault:note]]`** — embeds from a specific vault
- Embeds nest up to 3 levels deep

#### Typed Links
- **`[[note|type:related-to]]`** — adds a semantic type to the link
- Built-in types: `related-to`, `prerequisite`, `see-also`, `contradicts`, `supports`, `extends`
- Typed links appear with distinct colors in the graph view
- Autocomplete: type `[[note|type:` to see available link types

#### Smart Auto-Linker (Unlinked Mentions)
- The Backlinks panel detects plain-text mentions of the current note name across all vaults
- Expand the "Unlinked Mentions" section to see them
- Click "Link it" to automatically wrap the mention in `[[wikilinks]]`

#### Link Dashboard
- Open from the right sidebar (chain icon tab)
- **Most Connected** — top 10 notes by link count
- **Cross-Vault** — all links that cross vault boundaries
- **Broken** — links pointing to notes that don't exist
- **Orphans** — notes with no incoming or outgoing links

#### Enhanced Graph View
- Vault clusters: notes from the same vault are gently grouped together with colored hulls
- Cross-vault edges appear as dashed lines with a gradient between vault colors
- Typed links are color-coded by type in the graph
- Click a vault name in the legend to show/hide its nodes

### Markdown Rendering
- Full markdown rendering with syntax highlighting
- WikiLinks with preview on hover
- Callouts, footnotes, math (KaTeX), Mermaid diagrams
- Highlight syntax (`==text==`)
- Image embeds from vault attachments

### Search & Navigation
- Cross-vault full-text search
- Quick switcher (Ctrl+O)
- Command palette (Ctrl+P)
- Keyboard shortcuts for common actions

### Panels
- Properties editor (YAML frontmatter)
- Backlinks panel (with unlinked mentions + auto-link)
- Outgoing links panel
- Tags panel
- Graph view (with vault clustering + typed link colors)
- Link Dashboard (cross-vault links, broken links, orphans, most connected)

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
| **Auto-Linker** | Discover connections between notes across vaults |
| **Translate Note** | Translate between languages |
| **Meeting Notes** | Structure raw meeting notes |
| **Chart Generator** | Create charts from note data |
| **Research Assistant** | Analyze and synthesize across multiple notes |

Skills are extensible — create your own by defining a prompt template, inputs, and output format.

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
- **Path containment** — All file operations validate that paths are within registered vaults, preventing path traversal attacks. Read, write, scan, and link-resolution commands all enforce vault membership
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
- **Rust Backend** — File system operations, vault indexing, and cross-vault reference management handled by Rust for speed and reliability.

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
A companion mobile app is planned for read-only vault browsing and search. Full mobile editing will follow in Phase 3.

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
2. **Vault registration** — Add/remove vault paths
3. **File browser** — Navigate vault contents
4. **Search** — Cross-vault full-text search
5. **Graph view** — Unified Sky View with vault clustering
6. **Cross-vault references** — Star Lines
7. **Plugin system** — TypeScript extension API
8. **Mobile companion** — Read-only vault access

## License

MIT License. See [LICENSE](LICENSE) for details.

## Links

- **Website**: [uConstellation.World](https://uConstellation.World)
- **Repository**: [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

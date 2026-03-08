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
| **Universe** | An Obsidian vault |
| **Star** | A note (markdown file) |
| **Constellation** | A custom saved view — your curated grouping of stars |
| **Star Line** | A cross-vault reference connecting stars in different universes |
| **Sky View** | The unified graph showing all universes and their connections |

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

Constellation supports **Arabic (RTL)** and **English (LTR)** out of the box. The entire interface — including all skills — adapts direction, layout, and text based on the selected language.

## Planned Features

- **Unified Dashboard** — One interface for all your vaults. Browse, search, and manage files across every vault.
- **Full CRUD Operations** — Create, read, update, and organize files and folders, scoped to the correct vault.
- **Cross-Vault References** — Link notes across vault boundaries using Star Lines. These references live in Constellation's own metadata, never modifying your vault files.
- **Unified Graph View (Sky View)** — See all your vaults visualized as clusters of stars, with cross-vault connections drawn between them.
- **Cross-Vault Search** — Query across every vault simultaneously. Find anything, anywhere.
- **Plugin System** — Extend Constellation with TypeScript plugins. Community-built, sandboxed for safety.
- **Infographics & Slide Decks** — Generate visual outputs from your notes using AI skills.

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

- **Website**: [notesconstellation.com](https://notesconstellation.com)
- **Repository**: [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

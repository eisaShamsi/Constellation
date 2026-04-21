# Constellation — Concept Paper

**Version 2.0 — March 2026**

---

## 1. What Is Constellation?

Constellation is a desktop knowledge management platform built for people who think in connected notes. It stores everything as standard Markdown files on your local file system — no cloud accounts, no vendor lock-in, no subscription required.

Constellation introduces the **Universe** — a portable, self-contained workspace that unifies multiple libraries of Markdown files, structured databases, AI assistance, task management, and calendar views into a single coherent experience. Where other tools give you a single notebook, Constellation gives you an interconnected system.

**Technical foundation:** Tauri v2 (Rust backend) + SvelteKit + Svelte 5. Native performance, small binary size, full offline operation, no Electron overhead.

---

## 2. The Problem Constellation Solves

Knowledge management today is fragmented. Users face a common set of problems regardless of which tool they use:

| Problem | What Users Do Today | The Cost |
|---------|---------------------|----------|
| Notes scattered across tools | Manual copy-paste between apps | Lost connections, duplicated effort |
| One notebook at a time | Close one project to open another | Context-switching, no cross-project search or linking |
| Missing task management | Separate task app (Todoist, Things, etc.) | Tasks disconnected from the notes that created them |
| No database views | Export to spreadsheets or use separate tools | Data lives outside the knowledge system |
| Rigid table editing | Edit tables in a spreadsheet, paste back | Workflow interruption, no formulas in notes |
| Calendar disconnected | Separate calendar app | Daily notes and tasks not visible in one place |
| Template systems | Manual copy-paste or tool-specific syntax | Inconsistency, wasted setup time |
| AI as an afterthought | Separate AI tool, copy context manually | No integration with your actual notes |
| Importing from other tools | Manual conversion or format-specific scripts | Friction prevents migration, data stays trapped |

Some tools solve a few of these. None solve all of them. Users end up assembling a patchwork of apps, plugins, and workarounds — becoming systems integrators instead of knowledge workers.

**Constellation eliminates the integration tax.** Every capability listed above is built in, tested together, and ships as a unified experience.

### Non-Destructive by Design

Constellation is built on a foundational principle: **your files are never modified without your explicit action.** It reads your existing Markdown folders exactly as they are — it does not inject metadata, rewrite frontmatter, alter folder structures, or create hidden configuration files inside your libraries. Your Markdown files remain pure, portable, and fully compatible with any text editor or tool that reads standard Markdown.

This means adopting Constellation carries **zero risk**. Point it at your existing folders of notes, explore every feature, and if you ever decide to use a different tool — nothing has changed. There is no migration, no conversion, and no cleanup required. Constellation is a window into your knowledge, not a lock on it.

---

## 3. Core Architecture: The Universe Model

Constellation's defining architectural concept is the **Universe** — a portable directory that owns all user configuration and workspace state, separate from your notes.

```
MyUniverse/
  universe.json          # Identity and metadata
  libraries.json         # Registered library paths
  settings.json          # All preferences
  bookmarks.json         # Saved bookmarks
  workspaces.json        # Tab layouts
  property-types.json    # Custom property mappings
  bases/                 # Workspace-level databases
```

### Why This Matters

- **Portability.** Copy the universe directory to another machine and everything follows — settings, bookmarks, workspaces, database definitions. The libraries themselves are just folders of Markdown files that live wherever you want.
- **Multi-library by design.** A universe can register any number of libraries. Search, Sky View, task scanning, backlinks, and databases all operate across library boundaries natively.
- **Hierarchy.** Universes can reference child universes, inheriting their libraries. A team lead's universe can include a shared team universe plus a personal universe — with circular reference prevention built in.
- **No lock-in.** The universe is JSON files in a folder. The libraries are Markdown files in folders. Walk away at any time — your notes are standard files that any tool can read.

---

## 4. What Constellation Offers

### 4.1 Capabilities That Set Constellation Apart

| Capability | Details |
|-----------|---------|
| **True multi-library workspace** | Open, search, link, and graph across multiple libraries simultaneously in one window. |
| **Universe portability** | All configuration travels in a single portable directory. Move machines and your entire workspace follows. |
| **Child universes** | Compose workspaces hierarchically — a team library feeds into your personal universe automatically. |
| **Cross-library backlinks** | See which notes in *any* library link to the current note — not limited to a single library. |
| **Cross-library graph** | One knowledge graph showing connections across all your libraries. |
| **Unified task scanning** | Global Tasks view aggregates tasks from every library with filtering by library, priority, due date, and text search. |
| **Built-in Bases (databases)** | Non-destructive database views with table/card/list modes, filtering, sorting, inline editing — no external tools needed. |
| **Table formulas** | `=SUM()`, `=AVG()`, `=COUNT()`, `=MIN()`, `=MAX()` with cell references and ranges, evaluated in-place inside your Markdown tables. |
| **Multi-provider AI** | OpenAI, Anthropic, Google Gemini, and Ollama (local) from one interface, with 8 pre-built skills — directly integrated with your notes. |
| **Second screen** | A fully independent secondary window that extends your workspace across two screens — edit, browse, view graphs, or manage tasks side by side with no limitations. Not just a reference pane; a complete second workspace. |
| **15 languages at launch** | English, Arabic, German, Spanish, French, Hebrew, Hindi, Japanese, Korean, Portuguese, Russian, Turkish, Urdu, Chinese, Farsi — all with full RTL support. |
| **Security layer** | Library encryption at rest, idle lock with PIN, API key storage in OS keyring. |
| **Non-destructive library access** | Never modifies library files without explicit user action. Zero-risk adoption — try Constellation and switch tools freely with no trace left behind. |

### 4.2 Everything Built In

Features that other tools require plugins, extensions, or external apps to achieve ship built into Constellation:

| Feature | How Others Handle It | Constellation (Built-In) |
|---------|---------------------|--------------------------|
| Structured queries (Lens) | Plugin-based or external scripts | Native query parser (TABLE, LIST, TASK, CALENDAR queries) |
| Task management | Separate task apps or plugins | Library-wide scanning, toggle, due dates, priority, tags |
| Calendar sidebar | Separate calendar plugins | Month view with note/task dots, daily note creation |
| Advanced tables | Basic Markdown tables or spreadsheets | Row/column operations, sorting, move, formulas |
| Templates | Manual copy-paste or plugin syntax | Template variables (date, time, title, folder, library, cursor) |
| Note importing | Manual conversion scripts | 7 formats: Markdown folders, Notion, Bear, Evernote, HTML, CSV, Plain Text |
| Backlinks panel | Basic or plugin-dependent | Enhanced with cross-library support and unlinked mentions |
| Sky View | Single-source only in most tools | Cross-library nodes, force controls, grouping |
| Tag browser | Basic implementations | Tag frequency analysis, library-wide aggregation |

### 4.3 Import From Anywhere

Constellation's built-in importer supports migration from:

| Source | What Gets Imported |
|--------|-------------------|
| **Markdown folders** | Direct library registration — no conversion needed |
| **Notion exports** | Cleans hex IDs, converts internal links to wikilinks |
| **Bear notes** | Converts Bear's format to standard Markdown |
| **Evernote (.enex)** | Converts ENML to Markdown, preserves tags and dates as frontmatter |
| **HTML files** | Converts to clean Markdown |
| **CSV files** | Each row becomes a note with frontmatter properties |
| **Plain text files** | Direct import with Markdown extension |

Your existing notes from any tool become first-class citizens in Constellation without losing structure or metadata.

### 4.4 What Constellation Does Not Do (Yet)

Transparency matters. These are capabilities not currently in Constellation:

| Feature | Status |
|---------|--------|
| Mobile apps (iOS/Android) | Not yet — desktop only (Windows, macOS, Linux) |
| Cloud sync | Not built-in — use Git, Syncthing, or any file sync solution |
| Infinite canvas / whiteboard | Not yet |
| PDF annotation | Not yet |
| Audio recording | Not yet |
| Third-party plugin API | Not yet |

---

## 5. Who Is Constellation For?

### 5.1 The Multi-Project Professional

**Profile:** Consultant, researcher, or knowledge worker who maintains separate note collections for different clients, projects, or life domains.

**Pain today:** Must close one project to open another. Cannot search across collections. Cannot see connections between a client's project notes and research notes in a separate folder.

**Constellation answer:** Register all libraries in one universe. Search, graph, task scan, and link across all of them simultaneously.

### 5.2 The Tool-Fatigued Power User

**Profile:** Power user running multiple apps and extensions who spends significant time managing updates, resolving conflicts, and debugging breakage.

**Pain today:** Every tool update is a risk. Task management, databases, templates, calendar, and AI are all separate systems maintained by different teams on different schedules.

**Constellation answer:** All of these are built-in, tested together, and updated as one unit. Zero extension management.

### 5.3 The Arabic/RTL Knowledge Worker

**Profile:** User who works primarily in Arabic, Hebrew, Farsi, or Urdu and needs a note-taking system that treats RTL as a first-class concern.

**Pain today:** RTL support is inconsistent in most tools. Editors assume LTR. Date keys and list keys don't recognize Arabic equivalents. UI elements break in mirrored layouts.

**Constellation answer:** 15 languages including 4 RTL languages. Arabic property key detection (date, list, checkbox keys recognized in Arabic). Full UI mirroring. RTL-aware tables, forms, editors, and calendar.

### 5.4 The Team Lead or Organization Builder

**Profile:** Manager or team lead who wants to share a knowledge base with team members while maintaining personal notes separately.

**Pain today:** No concept of workspace composition in most tools. Shared note collections require manual setup per person.

**Constellation answer:** Create a team universe with shared libraries. Each team member adds the team universe as a child of their personal universe. Team libraries appear automatically alongside personal libraries.

### 5.5 The AI-Augmented Researcher

**Profile:** Researcher or student who wants AI assistance integrated directly into their note-taking workflow — summarization, Q&A, writing assistance, translation.

**Pain today:** Must use a separate AI tool, manually copy context, and paste results back. Or install competing AI extensions with inconsistent interfaces and separate API key management.

**Constellation answer:** One AI settings panel. Four provider options (including local Ollama for privacy). Eight pre-built skills. API keys stored in the OS keyring, not in plaintext config files.

### 5.6 The Security-Conscious User

**Profile:** Professional handling sensitive notes (legal, medical, financial, personal) who needs encryption and access control.

**Pain today:** Most note apps offer no built-in encryption, no idle lock, and store API keys in plaintext configuration files.

**Constellation answer:** Library encryption at rest, idle lock with PIN, API key storage in OS keyring.

### 5.7 The Migrating User

**Profile:** Someone moving away from Notion, Evernote, Bear, or another tool who wants to own their data locally without losing years of accumulated notes.

**Pain today:** Migration is painful. Export formats are inconsistent. Internal links break. Metadata gets lost. Many users stay locked in because switching costs are too high.

**Constellation answer:** Built-in importer handles 7 formats. Notion hex IDs are cleaned, links are converted to wikilinks, Evernote ENML becomes Markdown with frontmatter. One-click migration, zero data loss.

---

## 6. Technical Advantages

### 6.1 Performance

Constellation's Rust backend performs file operations, link scanning, task extraction, and database queries at native speed. Heavy operations — library-wide task scanning, structured queries, link graph building — execute in the Rust process and return structured results to the frontend. The editor never competes with background processing for resources.

### 6.2 Binary Size and Resource Usage

Tauri v2 uses the system's native webview rather than bundling Chromium. The result is a significantly smaller binary and lower memory footprint compared to Electron-based alternatives.

### 6.3 Security Model

Tauri's Rust backend provides a natural security boundary. File system access is controlled through explicit Tauri commands — the frontend cannot access arbitrary files. Path traversal prevention is enforced at the Rust layer (canonicalization checks on all file operations).

### 6.4 Data Sovereignty

All data lives on the user's file system in standard formats:
- Notes: Markdown files with YAML frontmatter
- Databases: JSON `.base` files
- Configuration: JSON files in the universe directory
- Attachments: Standard image/PDF files in library folders

No telemetry. No cloud dependency. No account required.

---

## 7. Development Validation Criteria

This section defines how we measure whether Constellation fulfills its purpose. Each criterion maps to a testable capability.

### 7.1 Core Promise: "Your Notes, Your Way"

| Test | Expected Result |
|------|----------------|
| Open any folder of Markdown files | All notes visible, frontmatter parsed, links resolved |
| Edit a note and save | File on disk updates, readable by any Markdown tool |
| Create a note with frontmatter | Valid YAML frontmatter, standard format |
| Resolve `[[wikilinks]]` | Correct resolution across files and folders |
| Render callouts, highlights, math, mermaid | Rich rendering of extended Markdown syntax |

### 7.2 Multi-Library Promise: "A Universe of Knowledge"

| Test | Expected Result |
|------|----------------|
| Register 3+ libraries | All appear in file explorer with distinct colors |
| Search across libraries | Results from all libraries, labeled by source |
| Graph across libraries | Nodes from all libraries, cross-library edges visible |
| Backlinks across libraries | Note in Library A shows backlinks from Library B |
| Tasks across libraries | Global Tasks view aggregates all libraries |

### 7.3 All-In-One Promise: "Everything Built In"

| Test | Expected Result |
|------|----------------|
| Structured query in note | `TABLE`, `LIST`, `TASK` queries render results |
| Task checkbox toggle | Toggle in sidebar updates file on disk |
| Calendar dot indicators | Days with notes/tasks show visual indicators |
| Table formula evaluation | `=SUM(A1:A5)` calculates correctly |
| Template insertion | Variables replaced with current date, time, title |
| Import from Notion export | Hex IDs removed, links converted to wikilinks |

### 7.4 User Experience Promise: "Works for Everyone"

| Test | Expected Result |
|------|----------------|
| Switch to Arabic | Full UI in Arabic, RTL layout, mirrored sidebar |
| Create note with Arabic properties | Date/list/checkbox keys detected correctly |
| Open app on new machine with universe copy | All settings, bookmarks, workspaces restored |
| Lock app, enter PIN | Notes inaccessible until correct PIN entered |

---

## 8. Competitive Landscape

Constellation occupies a unique position in the knowledge management space: **local-first, multi-library, all-in-one, and multilingual.**

| Dimension | Constellation | Obsidian | Notion | Logseq | Roam | Bear |
|-----------|--------------|----------|--------|--------|------|------|
| Data ownership | Local files | Local files | Cloud-hosted | Local files | Cloud-hosted | iCloud |
| Offline capability | Full | Full | Limited | Full | None | Full |
| File format | Standard Markdown | Standard Markdown | Proprietary | Markdown/EDN | Proprietary | Proprietary |
| Multi-library | Native (Universe) | One library per window | N/A (workspaces) | Single graph | Single graph | N/A |
| Cross-library search | Yes | No | N/A | No | No | No |
| Cross-library graph | Yes | No | N/A | No | No | No |
| Database views | Built-in (Bases) | Plugin required | Native | Queries (limited) | Queries | No |
| Task management | Built-in | Plugin required | Basic | Plugin required | Basic | No |
| AI integration | Built-in (4 providers) | Plugin required | Built-in (1 provider) | Plugin required | Plugin required | No |
| Table formulas | Built-in | Plugin required | Limited | No | No | No |
| Import sources | 7 formats built-in | Plugin required | Built-in (limited) | Limited | Limited | Limited |
| RTL / Arabic | 15 languages, 4 RTL | Community effort | Limited | Limited | Limited | No |
| Pricing | Free / Open Source | Freemium | Freemium + subscription | Free | Subscription | Subscription |
| Architecture | Tauri (Rust + native webview) | Electron | Web app | Electron | Web app | Native (Apple) |

### Constellation's Position

Constellation does not compete by being "a better version of X." It competes by being **a complete knowledge management platform** that eliminates the need to assemble a stack of tools. Users coming from any tool — or from no tool at all — can start with Constellation and have everything they need from day one.

For users of existing Markdown-based tools, the transition is seamless: point Constellation at your existing folders and everything works. For users of proprietary tools, the built-in importer handles the conversion.

---

## 9. Roadmap

Based on this concept paper, the following development priorities align with Constellation's positioning:

### High Priority (Reinforces Core Differentiators)

1. **Polish multi-library experience** — cross-library move/copy, library-scoped settings
2. **Bases performance at scale** — handle 10,000+ note databases efficiently
3. **AI skill expansion** — custom skill builder, context-aware library Q&A
4. **Mobile companion** — read-only library browser for iOS/Android

### Medium Priority (Broadens Platform)

5. **Canvas / whiteboard** — infinite canvas with embedded notes
6. **PDF annotation** — highlight and annotate PDFs within libraries
7. **Publish / static site export** — generate websites from library content
8. **Constellation URI protocol** — deep linking into specific notes and views

### Lower Priority (Future Vision)

9. **Audio recording and transcription**
10. **Plugin API** — allow third-party extensions (carefully scoped)
11. **Collaborative editing** — real-time multi-user editing via CRDT

---

## 10. Conclusion

Constellation exists because knowledge management should not require systems integration. A note-taking platform should ship with the tools its users need — databases, tasks, calendars, templates, importers, AI, and multi-library support — tested together, updated together, and usable out of the box.

For the knowledge worker who has built a workflow across multiple tools and feels the friction of managing that stack, Constellation offers a unified alternative that works with standard Markdown files, requires zero configuration, and provides capabilities no single existing tool offers — true multi-library workspaces, cross-library everything, and portable universe-based configuration.

The files are yours. The format is Markdown. The door is always open.

---

*Constellation is open source under the MIT license.*
*Developed by Eisa ALSHAMSI*
*Repository: github.com/eisaAlshamsi/Constellation*

---

## Legal Notice

### Trademark Acknowledgments

All product names, logos, and brands mentioned in this document are the property of their respective owners. "Obsidian" is a trademark of Dynalist Inc. "Notion" is a trademark of Notion Labs, Inc. "Bear" is a trademark of Shiny Frog Ltd. "Evernote" is a trademark of Bending Spoons S.p.A. "Logseq" is a trademark of Logseq, Inc. "Roam" is a trademark of Roam Research, Inc.

Constellation is an independent project and is not affiliated with, endorsed by, or sponsored by any of the companies mentioned above. All references to third-party products in this document are for purposes of factual comparison and interoperability description only, under nominative fair use.

### Intellectual Property Statement

Constellation is original software developed independently. It does not contain, incorporate, or derive from any third-party application source code. Constellation reads and writes standard Markdown files with YAML frontmatter — open, non-proprietary formats. Wikilink syntax (`[[link]]`) originates from wiki software and is not proprietary to any vendor. File-level interoperability with various Markdown-based tools is achieved through standard file system operations on open formats, not through reverse engineering or use of proprietary APIs.

### Open Source Compliance

Constellation is licensed under the MIT License. All third-party dependencies are used in compliance with their respective open source licenses. A full dependency audit is maintained in the project repository.

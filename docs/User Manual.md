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
8. [Constellation Sight](#8-constellation-sight)
8b. [Constellation Nervous System (CNS)](#8b-constellation-nervous-system-cns)
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

- **Type** — the kind of relationship (supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of, supersedes).
- **Annotation** — the *why*. Your reasoning at the moment of linking, authored inline via `[[type::Target|your reasoning]]` and displayed in italic purple text under the link in Backlinks / Outgoing panels.
- **Weight** — how significant the connection is. Starts at 1.0, grows logarithmically with each traversal, and decays exponentially when neglected.
- **Confidence** — how certain you are. Four tiers (Hypothesis → Evidence → Established → Contested). Auto-promotes as you traverse; right-click any link to override.
- **Tier (visual)** — derived from traversal count: *emerging* (×1–2), *established* (×3–9), *load-bearing* (×10+), *stale* (≥90d untouched).
- **Archive** — every operation is reversible. Archived links are soft-deleted (hidden everywhere, preserved in history) and can be restored from the Circulatory System's **Retired Reasoning** register.

Detailed step-by-step tutorials for every Living Link function — authoring, contesting, archiving, decay settings, the back-fill one-shot, and the Circulatory System's seven registers — live in the dedicated help file: [Knowledge Formulation](help.uConstellation.World/Knowledge%20Formulation/Knowledge%20Formulation.md).

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
3. Save. The link renders with a small **supports** badge in the Backlinks and Outgoing Links panels. The badge follows the **note's** language, not the interface's — an English note shows *supports* even in an Arabic interface; an Arabic note shows **يدعم**. The badge is part of the note's content, so switching the app language never re-labels the links inside your notes.

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
3. Row disappears from Backlinks / Outgoing and from the Circulatory System's registers. Traversal count and confidence are preserved.

#### Tutorial 7 — Restoring an archived link
1. Open the **Circulatory System** (the pulse-line icon in the left dock, below CNS).
2. Find the **Retired Reasoning** register.
3. Click **Restore** on the row you want back (use **Show all** if you have more than twenty archived links).
4. Link returns to active status — type, annotation, confidence, and traversal history intact; weight is recomputed from the traversal history, so a well-traveled link returns as strong as it left.

#### Backlinks / Outgoing performance + summary toggles
On notes with thousands of backlinks (a hub like *ISBN*), the Backlinks and Outgoing Links panels render only the rows currently on screen — the list gets its own scrollbar past ~50 items, so opening and scrolling a hub stays smooth no matter how many thousands of links it has.

Two optional one-line **note summaries** are available, both **off by default** (for a leaner, faster view):
- **Backlink/outgoing row summaries** — Settings → **Panels** → **Summaries** → *Note summaries*. When on, a one-line AI summary appears under each linked note in the Backlinks/Outgoing panels (for the first ~120 rows on very large lists).
- **Note-title summary** — Settings → **Editor** → *Note title summary*. When on, the open note's own one-line summary shows directly under its title in the editor.

Toggle either on or off at any time; turning a toggle off removes its summaries immediately.

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

#### Tutorial 10 — The Circulatory System (seven registers)

Open from the left dock (the pulse-line icon below CNS), the command palette, or Settings → Links.

| Register | Question it answers |
|---|---|
| Living Connections | What am I actively thinking through? |
| Load-Bearing Reasoning | What does my understanding rest on? |
| Cooling Inquiries | What have I stopped returning to? (90+ days) |
| Conviction & Doubt | How settled is my thinking? |
| The Life of a Connection | Where are my links in their lifecycle? |
| Retired Reasoning | What did I set aside — and can revive? |
| The Acts of Inquiry | What kinds of thinking am I doing? |

(The former Link Dashboard tab retired into these registers; most-connected stays in Knowledge Health, orphans in the tension panel.)

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
| **Notes Management** | Unified sidebar with mode tabs: Tree (File Explorer — with filter, sort, multi-select, batch), Digest (Universe Digest), OrgChart (Sky View) |
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

### Syncing and External Changes

Constellation follows **File Over App** — your notes are plain `.md` files on disk, and the app watches them for changes. If a note arrives or changes from *outside* Constellation while the app is open — an Obsidian sync from another device, a `git pull`, a cloud-sync tool (iCloud / Syncthing / OneDrive), or a file you drop into a library folder — Constellation picks it up **automatically**, within about a second, with **no restart**:

- The note appears in the **file tree**.
- It becomes findable in **Quick Switcher** (`Ctrl+O`), **Search**, the **Index**, **backlinks**, and the library **note count** — all update on their own.
- If you rename a folder from outside the app, its notes stay findable at the new location and the old entries are cleaned up.
- A large batch (a `git pull` of many notes, or a first sync) is indexed in the background — typing stays instant while search catches up.

You don't need to do anything: Constellation keeps its search index in step with your files as they change on disk. *(One detail: renaming a folder from **outside** the app resets those notes' review-schedule and link-weight history — the note text itself is untouched. Renaming folders **inside** Constellation preserves everything.)*

**If the changed note is currently OPEN in a tab**, Constellation brings it up to date safely — your work is never silently overwritten:

- If you have **no unsaved changes** in that note, the open note quietly refreshes to show the outside edit, so your next keystroke builds on the new version. *(Previously, an open note kept showing the old text and your next keystroke could silently save over the outside edit — that can no longer happen.)*
- If you **do have unsaved edits** in that note at the same moment an outside change arrives — a genuine conflict — Constellation never touches your unsaved work. It keeps **your** version in the editor, writes the incoming outside version to a **side-copy** next to the note (named `<note>.conflict-<timestamp>.md.txt`, so nothing is ever lost), and shows a banner: *"An external edit to {note} was kept as a separate copy — your version is unchanged."* Click **Show copy** to open the folder to that side-copy. The side-copy is an inert `.txt` file — it never appears in your sidebar or search and never triggers another sync.

**Merging the two versions.** The conflict banner also has a **Merge…** button. It opens a full-screen, two-column view — **Your version** on the left (editable) beside the **Outside copy** on the right (read-only) — with the differences highlighted and the identical parts folded away. Next to each difference is a **Copy to mine** button that pulls that outside change into your version; you can also edit the left column freely to combine the two by hand. When you're done, **Save merged** writes your reconciled note and moves the side-copy to the library trash (recoverable, never deleted); **Cancel** changes nothing — both versions stay exactly as they were. Constellation never merges automatically — the reconciliation is always your choice.

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

### Your tabs come back on relaunch

Until now, closing Constellation forgot which notes you had open — every launch started blank. Now the app remembers your open tabs, which one was active, and whether the window was split, and puts them back automatically the next time you launch. The desk looks the way you left it.

- The memory is **per-Universe** and updates quietly about a second after you open, close, or rearrange tabs. A crash or force-kill loses at most the last second of *arrangement* — never note content (content safety is a separate, older mechanism).
- A note that was moved or deleted while the app was closed is simply skipped; the rest of your tabs still return.
- To turn it off: **Settings → Editor → Restore tabs on relaunch**. Turning it off also deletes the remembered session — off means *stop remembering*.
- Named **Workspaces** are unaffected: they stay your deliberate, hand-saved snapshots. This feature is just the rolling "last state".
- Known limit: with a split view, the split itself returns but which tabs sat in which half is not remembered yet.

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

#### Customising callouts — colours, icons, and your own types

The callout colours and icons are **yours to change**, and you can invent your own callout types. Open the **Style Setter** (the 🎨 button in the dock), choose the **Editor** category, then click **Callouts**. The centre opens a single **Callouts manager** where every callout is one row showing its colour, its icon, and its name.

- **Recolour a built-in callout.** Click the colour swatch on its row. A small palette opens with your saved colours (click one to apply) plus a **Custom…** picker for any new colour — a colour you pick is also added to your palette for next time. *Colour changes for the built-in types are saved when you press **Keep/Apply** in the Style Setter.*
- **Change a built-in callout's icon.** Click the icon on its row. The Emoji & Icon Library opens — pick any emoji or vector icon. It changes everywhere immediately, in the colour of that callout. A small **↺** appears so you can revert just that icon.
- **Reset the built-ins.** The **↺ Reset this element** button at the top of the manager reverts all built-in callout colours and icons to their defaults. (Your custom callouts are left alone — remove those individually.)
- **Create your own callout type.** Below the divider is the **Add** row. Type a **Name** (e.g. `Decision`, or `فكرة`), a **Trigger** word (the `[!word]` you'll type — any language works, including Arabic), pick a **colour** and an **icon**, and click **Add**. Now typing `> [!decision]` (or `> [!فكرة]`) in any note renders your callout. If you don't type a title after the trigger, the callout header shows your callout's name in bold.
- **Edit or remove a custom callout.** Use the **✎** (edit the name/trigger) and **✕** (remove) on its row. Removing a type leaves the `[!…]` text in your notes untouched — it simply reverts to the plain note look until you re-create the type.

Your custom callouts, colours, and icons are saved **with this Universe**, so they travel with your library.

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

### Saving and Recovery

Constellation **saves automatically** as you type — there is no Save button. Your edits are written to the `.md` file a moment after you pause, and **immediately whenever you leave the note** — switching notes, following a `[[wikilink]]`, pressing Back/Forward, or closing the tab — **even if you were still typing**. A note is marked "saved" only once it is genuinely written to disk, so moving away never costs you an edit. (If the file happens to be locked at that moment, Constellation keeps you on the note and shows the recovery banner below instead of moving on.)

**Closing the app is a save point too.** When you close Constellation, every note with unsaved typing is written to its file **before** the window closes — including words typed in the very last second before you clicked ✕. A normal close (nothing unsaved) is instant, exactly as before; when there is something to write, the window may stay open for a brief moment (capped at five seconds) while your notes land safely on disk.

**One note, one tab.** Opening a note that is already open simply jumps to its existing tab — a note is never open in two tabs at once, so two copies can never overwrite each other.

If a save ever **fails** — for example a sync tool (iCloud / OneDrive / Syncthing) or antivirus briefly locks the file — Constellation does **not** lose your work:

- A banner appears at the top: *"Couldn't save {note} — your edit is safe and will retry."* Your typing stays on screen and is held safely in memory (and in a recovery buffer that survives a restart).
- Constellation **auto-retries every few seconds**, so once the file frees up your edit is written on its own — even if you have walked away.
- You can also click **Retry now** on the banner to save immediately. The banner disappears once the note saves.

You never have to worry about a locked or briefly-unavailable file costing you an edit.

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

The second row contains mode tabs to switch how your notes are displayed:

| Tab | Icon | Description |
|-----|------|-------------|
| **Tree** | Folder tree icon | File Explorer — browse, filter, sort, multi-select, and batch-organize your notes and folders |
| **Digest** | Lines icon | Universe Digest — an at-a-glance summary of your active universe |
| **OrgChart** | Tree diagram icon | Sky View — interactive hierarchy tree visualization |

Click a tab to switch modes.

> The former dual-pane "Notes Navigator" (List mode) has been **retired** — its file-management strengths (filter, richer sort, multi-select, and batch operations) now live directly in the **File Explorer** below, where they belong to the file system itself. Its facet-browsing overlapped surfaces that already own those jobs (the Tags panel, Search Hub), so it is no longer a separate mode.

### Adaptive Sidebar Width

The sidebar automatically adjusts its width to fit the longest library or child universe name visible in the current view. This ensures all names are readable without manual resizing.

### Child Universe Grouping

Across all three modes, content is organized with consistent grouping:

1. **Child universes first** — each child universe appears as a collapsible group with its libraries nested inside
2. **Own libraries below** — the parent universe's own libraries appear below a visual separator

This grouping is consistent across Tree, Digest, and OrgChart modes.

### Cross-Mode Selection Sync

Clicking a child universe, library, folder, or note in any sidebar mode highlights the corresponding nodes in the Sky View graph. This bidirectional sync helps you maintain spatial awareness as you browse your knowledge base.

### Picture-in-Picture (PiP) Overlay

When Sky View is open and you click a child universe, library, or folder in the sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay. The PiP shows a filtered sub-graph containing only the nodes belonging to the selected scope, with its own legend showing only the relevant entries. You can resize and reposition the PiP window freely.

### Tree Mode (File Explorer)

The file tree for browsing **and organizing** your notes and folders. Beyond the classic tree, it now carries the file-management muscle you need for a large library:

**Filter by name.** A filter box sits at the top of the tree. Type any fragment of a note or folder name (in any language) and the tree narrows to matches, opening the folders that contain them so nothing is buried. The filter searches **every** library — collapsed ones are loaded and revealed automatically, then restored to exactly how you had them when you clear the filter. It matches **names only**, never note contents (searching *inside* notes is Search Hub's job).

**Sort eight ways.** The sort button cycles through **Name** (A→Z / Z→A), **Modified** (newest / oldest), **Created** (newest / oldest), and **Size** (largest / smallest); folders always stay on top. Hover the button to see the current sort.

**Multi-select.** **Ctrl-click** (⌘-click on Mac) to add or remove a note or folder from the selection; **Shift-click** to select a whole range. Plain-clicking a note still just opens it — the selection stays put until you press **Escape** or clear it. Selected rows are highlighted with an accent bar.

**Batch operations.** With items selected, a bar appears at the bottom of the sidebar showing the count, with **Move**, **Add tag**, and **Delete**. Each applies to the whole selection through the same safe, gated operations a single note uses — so batch-tagging never corrupts a note, and delete is trash-backed. Notes from linked child-universes (read-only) are skipped automatically.

**The basics remain:**
- Expand/collapse folders with click or arrow keys
- Right-click for context menu — **notes:** Open, Open in new tab, Rename, Move, Add tag, Copy path, Copy name, Reveal in tree, Suggest sources, Delete; **folders:** New note, New folder, New base, Rename, Move, Delete; **library roots:** New note, New folder, New base
- Drag and drop to move notes between folders
- **Move** opens a universe-wide folder picker (all libraries) — search or scroll, double-click to move instantly

**Renaming updates links automatically.** When you rename a note — from the file tree (right-click → Rename) or by editing its title at the top of the page — Constellation rewrites every `[[link]]` pointing to it across the library to the new name, so links never silently break. A brief "Updating links…" overlay appears on the affected note(s) while this runs (the editor pauses typing for that moment); the old title is kept as an alias so existing links still resolve.

**Name collisions are caught universe-wide.** Every note title stays unique across your whole universe — all libraries and any linked child universes — so `[[wikilinks]]` always resolve to exactly one note. When you create a note with a typed name, or rename one, onto a title that already exists *anywhere*, a dialog appears: **Change name** (pre-filled with a free suggestion like *Foo 1*), **Overwrite** (the displaced note is moved to its library's `.trash` first — recoverable, and given a numeric suffix if a same-named note is already trashed, so trash is never clobbered), or **Cancel**. The dialog names which library the existing note already lives in. Quick Capture's auto-named notes are not interrupted; folders are unaffected.

**Deleting notes is recoverable.** When you delete a note or folder — right-click → **Delete** in the file tree, or multi-select **Delete** in the File Explorer's batch bar — where it goes is set by **Settings → Universe & Libraries → "Deleted files"**: the **Windows Recycle Bin** (the default), or a **`.trash` folder** kept either inside the note's own library or at the universe root (your choice, in the same setting). Either way the note is recoverable, and it disappears from your tree and search immediately. There is deliberately **no "permanently delete" option** — routine deletes are always reversible.

### OrgChart Mode (Sky View)

An interactive tree-list visualization of your entire knowledge base hierarchy:

- Click to expand/collapse branches
- Click a note to open it in the editor
- Right-click for a contextual menu — **notes:** Open, Open in new tab, Rename, Move, Add tag, Copy path, Copy name, Reveal in tree, Suggest sources, Delete; **folders:** New note, New folder, New base, Expand/Collapse, Rename, Move, Reveal in tree, Delete; **library roots:** New note, New folder, New base, Expand/Collapse
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

### Quick Switcher (Ctrl+O)

The Quick Switcher is the **jump-to-note-by-name** command — press `Ctrl+O`, type part of a title, press Enter. It searches **titles and aliases only** (never note contents — that's the Search Hub's job below), entirely from memory, so results appear instantly as you type, in any language.

Results are **ranked by how well the title matches**: an exact title always comes first, then titles that *start with* your words, then titles containing them at a word boundary, then looser matches — so typing `islam` puts the note titled *Islam* above *"Abraham in Islam"*. Arabic matching ignores diacritics and hamza variants (typing `اسلام` finds `إسلام`). Notes whose **alias** matches appear as *alias → real title* rows. Titles from **linked cUniverses** are included — the switcher spans your whole universe.

- **Empty Ctrl+O** shows your recently-opened notes — Enter re-opens the last one.
- **Create note "…"** — when nothing matches your text exactly, a bottom row lets you create a note with that name on the spot (it lands in your universe root and opens immediately).
- **Search "…" in Search Hub** — the last row hands your query to the full content search.

### Search Hub

The Search Hub is a full-screen search experience. Click the magnifying glass icon in the dock bar to open it. Both sidebars collapse to give maximum space. Type any term and Constellation searches everywhere simultaneously, grouping results into 5 categories: Titles, Contents, Tags, Properties, and Wikilinks. Each category has a collapsible section with a count badge. Click any result to open it in the editor with all occurrences highlighted. A "Return to Search Hub" button appears so you can go back without re-searching.

Under the search box are two tabs: **Results** and **Collections**.

### Collections

A **Collection** is a named, saved basket of notes you hand-pick — the working set for a task. Search normally throws its results away when you close it; a Collection is your decision about *which of those results matter*, kept across restarts. It's different from its neighbours on purpose: **Search** finds notes (temporary), **Bases** build a live table from a rule (auto-filled), the **File Explorer** shows where notes live (one home each). A Collection is none of those — it's your judgement, saved: any note can sit in as many collections as you like, hand-placed.

**Making one, and filling it.** Open the Search Hub → **Collections** tab → **+ New**, type a name, press Enter. Then run a search on the **Results** tab, **right-click** any result → **Add to collection ▸** and pick your collection (or **New collection…**). You can also right-click a note in the file tree the same way.

**Working with a collection.** The dropdown at the top switches between your collections. Each note shows its live details (title, library) and opens when you click it; the **✕** on the right takes a note out of the collection (it never deletes the note itself). You can **rename** (✎) or **delete** (🗑) any collection you made. Four filter chips — **Due**, **Unlinked**, **Contested**, **Forming** — *narrow* the view to notes in that state (they never add notes; combine them for AND). A collection stays current automatically: edit a note elsewhere and its row updates within a moment.

**Starred (your bookmarks).** Bookmarks are now a special, permanent collection called **Starred** — it can't be deleted, and it's also shown in the sidebar's "Bookmarks" section for quick access. The ⭐ star button works exactly as before; anything you star lands in Starred. Everything you had bookmarked before is migrated in automatically the first time you open this version (your old bookmarks file is kept untouched as a backup). In the sidebar, each bookmark now shows *where it lives* — cUniverse / library / folder — on the far side of its row, and a right-click there gives you Open, Reveal in tree, Remove bookmark, Add to collection, and Copy.

### Cross-universe federation

When your active universe has one or more cUniverses linked, search results span the federated set — you'll see notes from BOTH your active universe AND each linked cUniverse in the same result list. The status-bar note count reflects the federated total (e.g. "8751 notes" instead of just the active universe's count). Sidebar library badges show per-library counts including cUniverse libraries.

If a cUniverse is unavailable at boot (its `search.db` file is missing, locked by another process, or schema-drifted), a **triangle warning badge** appears in the status bar with a count of skipped cUniverses. Click it to see which cUniverses were skipped and the reason. The rest of the app continues to work — search still spans the cUniverses that DID attach successfully (skip-unavailable model).

To open a cUniverse as the active universe (e.g. to build its missing `search.db`), use the Universe Manager from the sidebar.

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

> **Note — the graph now loads when you open Sky View, not at startup.** To keep Constellation's startup fast on large libraries, the knowledge graph is no longer read at launch. It loads the first time you open Sky View (or the Lens / Sight views) and is quietly pre-loaded in the background a moment after the app is ready — so it's usually there instantly. If you open Sky View within a second or two of launching, you may briefly see a small "Loading graph…" indicator while it finishes loading.

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

Constellation Sight is the **diagnostic instrument** for your knowledge universe. A central anchor dome shows every note positioned by **stratum** (how foundational the thinking is) × **time** (when written), with four mini-domes alongside that re-encode the same universe through different channels — **Confidence** (opacity), **Stage** (color), **Acts** (size), **Provenance** (5 sectors).

It answers: **"How is my Epistemic Content shaped and organized?"**

### Opening the Sight

Click the **eye icon** in the dock at the left edge of Constellation. The anchor dome renders within 2–5 seconds. Close with the **(×)** in the top-right or press **Esc**.

### Key concepts

- **Coordinated Views.** Hover any star in any of the 5 surfaces (anchor + 4 minis) → gold ring on the same note in all 5, plus matching sidebar chips light up.
- **Dome-swap.** Click empty area of any mini-dome → it promotes into the primary slot at full size; the previous primary demotes into the vacated mini slot. Click another mini's empty area to shuffle.
- **Cross-filter from the dome.** Shift+click a star in the Stage / Confidence / Provenance mini → filter to that star's category. Click again to remove.
- **Ghost mode.** When a filter is active, non-matching stars stay visible at low opacity. Shift+click a faded ghost to ADD its category to the filter (multi-select within a facet, directly from the dome).
- **Extended view.** Ctrl+Shift+D (Cmd+Shift+D on Mac) toggles persistent "minis default-visible on every open". "EXTENDED" badge appears in header. Ctrl+D keeps its per-session toggle behavior.

### Mini-dome channels

| Mini | Channel | Encoding |
|---|---|---|
| **Confidence** | confidenceAlpha | Opacity — confident notes brighter, tentative ones fade |
| **Stage** | lifecycle stage | Color — cyan spark, orange birth, violet growth, green maturity, yellow renewal, gray dormancy/archival |
| **Acts** | top-decile by link count | Binary size — top-10% biggest dots, rest small |
| **Provenance** | sources_primary | 5 angular sectors — Self / Read / Heard / Reasoned / Tradition |

### Common gestures

| Gesture | Effect |
|---|---|
| **Hover a star** | Gold ring on the same note in all 5 surfaces + matching sidebar chips tint gold |
| **Plain click a star** | Opens the note in the editor. "Return to Sight" button appears in the note's tab bar |
| **Shift+click a star** in Stage / Confidence / Provenance mini | Toggle the filter on that star's category |
| **Click empty area of a mini** | Promote that mini into the primary slot |
| **Wheel-zoom (primary slot)** | Zoom toward cursor (0.5× to 24×) |
| **Click+drag empty space** (primary slot) | Pan the view |
| **Ctrl+0 / Cmd+0** | Reset zoom + pan |
| **Ctrl+D / Cmd+D** | Toggle minis (session only) |
| **Ctrl+Shift+D / Cmd+Shift+D** | Toggle Extended view (persistent) |
| **Reset View button** | Return to anchor primary at zoom 1.0 |
| **Esc** | Close Sight |

For the full reference — every visual element, every interaction nuance, density mode, the facet sidebar's 6 facet groups — see the in-app help topic **Constellation Sight** under Help → Knowledge Formulation.

### 8a. Per-note tradition fields (MIG-029)

The tradition chip in the top-left of Sight lets you re-frame the dome through 24 scholarly traditions across 10 epistemic families. For nine of those traditions (the sectoral / concentric / ladder shapes), each note can be **explicitly classified** via a frontmatter field. Notes without the field land in a sensible per-tradition default bucket; notes WITH the field land in the bucket you've named.

Add the field to a note's YAML frontmatter:

```yaml
---
masadir_source: sunnah
---
```

Switch to that tradition's chip → your note lands in its named sector instead of the default.

**Allowed fields and values:**

| Tradition | Frontmatter field | Allowed values | Default if absent |
|---|---|---|---|
| **masādir** (Sunni uṣūl al-fiqh) | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` |
| **pramāṇa** (Indian Nyāya) | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` |
| **Ibn Rushd burhān** | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `shir` (outermost ring) |
| **PaRDeS** (Jewish hermeneutics) | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` |
| **Peirce** (3 phaneroscopic categories) | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` |
| **Habermas** (3 knowledge interests) | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` |
| **Mencian sprouts** (4 moral sprouts) | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` |
| **Mohist sān biǎo** (3 standards) | `mohist_zone` | `ben` / `yuan` / `yong` | hash-bucketed across 3 zones |
| **Korean Sŏngnihak** (Four-Seven debate) | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` |

**Behavior:**
- If you write a value the tradition doesn't recognize (typo or invented), the note lands in the default bucket. No crash, no rendering glitch.
- Frontmatter changes propagate automatically — save the note → the dome's next render reflects the change.
- The same field is read only by its named tradition. Setting `masadir_source: sunnah` on a note has no effect when you switch to PaRDeS or Peirce — each tradition reads its own field, independently.
- This is the most explicit way to control the dome's spatial grammar. Without these fields, the geometry is correct but every note defaults to the same bucket; with them, the chip is analytically meaningful.

**Traditions without per-note fields** (currently bucket all stars by other means — folder / library / hash):

- Aristotelian (the default, no remap)
- Polanyi (gradient fog; no sectoring)
- Husserl, Longino, Shāṭibī maqāṣid, Maimonidean prophecy, Talmudic 13 middot, Wang Yangming, Mignolo pluriversal, Dussel transmodernity, Maldonado-Torres, Akan Wiredu, Ibn Khaldūn ʿumrān, Ibuanyidanda

(Future MIGs may add per-note frontmatter fields for these as user demand surfaces.)

---

## 8b. Constellation Nervous System (CNS)

The **wiring diagram of your universe** — the topology instrument of the Connection question, and the structural sibling of the Circulatory System (CCS): CNS reads the wiring (who connects to whom); CCS reads the flow (how the connections live). In Arabic the surface is **الجهاز العصبي للمعرفة** — the Nervous System of Knowledge.

It answers: **"What is the SHAPE of my thinking — its regions, its bridges, its silences?"**

### Opening CNS

Click the **CNS button** (the branching nerve-cell icon) in the left dock — beside the Circulatory System's pulse icon. You can also cross between the two organs: CCS's header carries a CNS button, and CNS's panel carries a "Circulation → Circulatory System" row.

### The gravity well — the layout IS the reading

- **Distance from the center = structural centrality** (the notes your wiring routes through sit deep).
- **Angular sector = your own libraries** — position always follows *your* order; analysis never moves a note.
- **Node color = library**, until the header's **Regions lens** (three-circles button) recolors nodes by their *found* region of thought. Color is the lens; position never changes.
- Links stay hidden until you hover, select, or search (the resting state is calm).
- The header caption — *resolved connections · this universe* — marks that CNS counts the connections that resolve on the graph, while CCS and Knowledge Health count every recorded link record.

### The panel registers

- **Structural Cohesion** — the 0–100 score of how well-formed the wiring is, with Modularity, Dominance, Diversity, and Links/Note readable.
- **Regions** — the found neighborhoods, with size and dominant maturity. **Hover** a row → everything outside the region mutes to black-and-white. **Click** a row → the mute **pins** while you work inside the region (re-click to unpin); a selected note's connections keep their color even across regions.
- **Top Bridges** — the notes that hold the structure together (highest centrality).
- **Hubs** — the most-connected notes (this is their one home).
- **Blind Spots** — region pairs that should touch and don't (*A ↮ B*), each with up to three clickable **suggested bridge notes**. An empty list is good news.

### Interaction

| Gesture | Effect |
|---|---|
| **Hover a node** | Title tooltip; its links light up. |
| **Single click a node** | Selects it + lights its neighborhood. Nothing opens. |
| **Double click a node** | Opens the note in the editor. |
| **Wheel / drag / corners button** | Zoom / pan / fit to screen at any window size. |
| **Hover / click a Regions row** | Black-and-white mute / pin the mute. |
| **Click empty space** | Clears the selection. |
| **Esc** | Closes CNS. |

For the full reference, see the in-app help topic **Constellation Nervous System (CNS)** under Help → Knowledge Formulation.

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
| **Editor** (a note open) | **The Knowledge Cockpit** — a read-only view of the open note: its links as a graph — The Butterfly, The Ledger, or The Orrery plus a deck of gauges showing the note's own statistics |
| **File Explorer** | Universe Dashboard — stats, library breakdown, child universes, tags, recently edited/opened notes |
| **Navigator** | Full Navigator view for browsing notes |
| **Sky View** | Sky View tree with directory structure |
| **Sky View** (graph) | Sky View companion with backlinks, forward links, tags, and local graph |
| **Split View** | Comparison panels — all split notes side by side with shared panel selector |
| **Constellation Map** | Map companion with mini-maps, color dropdown, and legend |
| **Index** | Term exploration — note list + editor for clicked terms |

### Editor Mode — the Knowledge Cockpit

When a note is open in the main window, the Second Screen becomes the **Knowledge Cockpit**: a read-only view of everything *around* that note. It never edits and never saves — it complements the note you are writing, it does not duplicate it. A **read-only** badge in the corner says so.

**The coupling dial** (top-left) decides which note the cockpit is looking at:

| Setting | What it does |
|---|---|
| **Follow** (default) | The cockpit always shows the note you are editing. Switch notes in the main window and the cockpit follows. |
| **Pin** | The cockpit locks onto the note it is showing. Move around the main window freely — the cockpit stays put. A **pinned** badge appears. |

**The facet tabs** below the dial choose what you want to know about the note. **Links** is the one that is live today; the rest are being wired in.

#### The note graph — two lenses

The **Links** facet draws the note's living links. Your note sits in the middle; everything that **points at it** is on the **left**, everything it **points to** is on the **right**. Every link is drawn — nothing is hidden or sampled. Colour tells you the *kind* of relationship (supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of, supersedes).

Pick the lens with the toggle at the right-hand end of the tab row. Your choice is remembered.

- **The Butterfly** (default) — two facing wings. Each wedge is one kind of relationship; the biggest one points straight out along the wing, the smaller ones fan above and below it, so the wings sit level and mirror each other. Inside a wedge, **every individual link is its own stem** running out to the wedge's rim, ending in a small bead. A stronger, more-travelled link gets a **bigger bead** and a slightly darker stem. Each wedge names itself down the edge of the screen — `part-of · 394` — so nothing overlaps.
- **The Ledger** — the same links as a balance sheet. A rail runs down the middle, one row per kind of relationship, identical on both sides. Backlink bars grow left, outgoing bars grow right, measured against a shared scale, so you can see at a glance whether a note is held up by what points at it or by what it points to. **Click any bar** to open the list of its individual links.
- **The Orrery** — the note as a solar system, showing TIME. Your note is the sun; its links orbit it on six rings from **today** (close, warm) out to **never walked** (the cold rim), so a link's distance shows how recently you last followed it. Which direction it sits shows its relationship type, and each wing's width shows how many links of that type there are. Solid dots point out, hollow rings point in; a bigger dot is a more load-bearing link. The one thing to watch for: a **big dot pulsing amber out on a cold ring** — a link your thinking rests on that you've stopped walking. **Hover a wing** and it zooms open to show every one of its links, larger; move away and it settles back.

In both lenses: **hover** a link to see its name, and **click** it to open that note **in the main window**. The cockpit itself never changes what you're editing.

#### The gauge deck

Along the bottom of both lenses runs a strip of gauges — the note's own statistics, grouped by the four questions the Cognitive Engine asks of any piece of knowledge:

| Gauge | Shows |
|---|---|
| **Development** | Stage and maturity as filled dot-ladders, plus a coloured review pill (up to date · due · stale · never reviewed) |
| **Content** | The word count, the note's stratum, and its tags |
| **Origin** | Where the note came from — provenance, source, and the date it was created |
| **Connection** | A relationship-mix bar, a supports-versus-contradicts balance meter, a confidence bar, and the count of load-bearing links |

A gauge only appears if the note actually carries that information.

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

### The Cataloger — the full-page home

The same cards also live in a full-page view called **the Cataloger**, opened from the **stacked-cards icon in the left dock**. It is the same engine and the same queue, given the whole window instead of a narrow sidebar tab — and it adds two controls the sidebar tab never had:

- **Classify a note…** — a search box that lets you classify *any* note by name, without opening it first. Type a few letters, pick the note, and a fresh card appears in the queue.
- **Build all summaries** — pre-computes the note summary (see below) for every note that lacks one, in the background, with progress in the status bar.

A **Start scan** button (the same universe-wide scan as Settings) and a live progress strip round out the header. Close the Cataloger with the **(×)** or **Esc**. (When the *Classify a note…* search box is open, the first **Esc** closes just that box.)

A naming note: **the Cataloger** is the *room* (the full-page view); **the catalogers** are the *six lenses* inside the engine that vote on each card. Don't confuse the two.

### Note summaries

Under each card's title sits a short **Summary** — a few sentences telling you what the note is about, so you can classify it without opening it. Constellation always prefers a summary *you* wrote and only generates one when you haven't:

1. A `summary:` / `description:` / `abstract:` / `excerpt:` **frontmatter field**, used verbatim.
2. A `> [!summary]` / `[!abstract]` / `[!tldr]` **callout** in the body, used verbatim.
3. Otherwise, a **generated** summary — the note's three most-central sentences, extracted (never invented) and shown in original order.

Generated summaries are **read-only** — Constellation never writes one back into your note (File-Over-App), and everything is computed **on your device**. If you want a summary to live in the file, write one yourself and Constellation will show yours instead.

For deeper detail (every dot status, every rule chip, click-by-click walkthroughs), see the **Source Review**, **The Cataloger**, and **Note Summaries** topics in the help system.

---

## 10c. Epistemic Metadata

A small set of optional frontmatter fields for recording richer information about how a note's knowledge was acquired, who holds the position, what discipline it belongs to, and when you last revised your view. Added in MIG-022 §A in response to the gap analysis (`docs/epistemic-content-gap-analysis.md`).

These fields are **all optional**. Notes without them work unchanged.

### Quick reference

| Field | Type | Purpose |
|---|---|---|
| `held_by` | text | Whose stance is this? (defaults to `user`; can be `"al-Shāfiʿī"`, `"Ḥanafī"`, etc.) |
| `domain` | list | Disciplinary tags for retrieval (`[fiqh, ʿibādāt]`) |
| `function` | text | What this note is for (`reference` / `seed` / `actionable` / `shipped`) |
| `provenance_civilization` | text | Tradition vocabulary (`sunni-usuli` / `analytic-western` / `nyaya` / etc.) |
| `updated_at` | date | When you last deliberately revised your view (distinct from the file-system mtime) |
| `ikhtilāf` | list of objects | Structured scholarly disagreement (`[{school, position}, ...]`) |
| `warrant` | text | Grade label (parsed but inert until the Warrant Research workstream ships) |
| `warrant_notes` | text | Free text supporting the warrant grade (also inert) |

### How they appear in the Properties panel

Each field renders with the type-appropriate editor:
- Text fields → text input
- `domain` → tag list (Enter to add, × to remove)
- `updated_at` → date picker
- **A property holding a block of its own fields** (for example `source:` with `title`,
  `author` and `year` underneath) → **read-only summary**: the row lists the field names
  as chips followed by a faint *read-only* label. The whole row is inert — the value, the
  field name, and the remove button — because a block Constellation cannot fully edit is
  a block it will not damage. Everything else on the note edits normally and leaves it
  untouched. Editing nested fields in the panel is planned; until then, edit them in any
  text editor and Constellation will preserve your changes exactly.
- **`ikhtilāf` → custom widget** with two side-by-side inputs per row (school + position) plus a remove button per row, and an "Add school" button at the bottom. The widget reads from + writes to the structured YAML, so round-trips preserve every field.

### What about `supersedes`?

`supersedes` is a *relationship between notes* (this note replaces an earlier one), not a property of a single note. Constellation handles it as a **typed link**, not a YAML scalar:

```markdown
This replaces my earlier analysis: [[old-note-id|supersedes]]
```

The `|supersedes` suffix on the wikilink makes it a typed-link of the `supersedes` kind — distinct slate blue-gray pill, shows up in Backlinks + Outgoing Links panels, participates in Living Link Architecture.

### What this is NOT

The new fields are **schema** — a recognized vocabulary you can fill in. CECE doesn't currently consume them for classification. Future MIGs (Warrant Research workstream, MIG-023 temporal axis) will ship features that read `warrant`, `updated_at`, and friends.

For deeper detail + a worked example, see the **Epistemic Metadata** topic in the help system.

---

## 10d. Constellation Sight v5 (SUPERSEDED)

> **This section is preserved as historical reference only.** Sight v5 was the previous generation of the dome view; it has been superseded by Sight v6.1 (Coordinated Views) as documented in Section 8 above and in the in-app help topic **Constellation Sight**. The v5 chrome and mode-toggle interaction described below is no longer the live UI.



A full-screen visualization of the shape and organization of your epistemic content as a stable star chart.

### Open Sight v5

Click the star icon in the left dock. Sight v5 takes the full content area; press **Esc** or click the **×** in the header to close.

### What you see

- Eight concentric strata bands — L1 Datum at the rim, L8 Worldview at the pole.
- A 12-month calendar rim wrapping the outside of the dome.
- Stars (your notes) positioned at their stratum band.
- Faint typed-link lines between notes.

### Encodings (never change with mode)

- **Position** (radial) = strata (where in the L1→L8 hierarchy the note sits).
- **Size** = maturity (seed → sapling → evergreen → canonical → wilting).
- **Brightness** = confidence (hypothesis → evidence → established).
- **Color** = ink black for normal; red for contested.

### The 7 modes

R Regions · L Link Types · T Time · C Confidence · S Stages · A Acts · P Provenance. The toggle bar at the top of the dome re-cuts the rim wedges per mode. **Strata stays the radius**; only the angular position changes.

### The 3 scopes

U Universe · L Library · F Folder. The scope toggle below the mode bar narrows the visible note set.

### Interactions

- Hover a star → tooltip + incident links brighten.
- Click a star → side panel with note detail + "Open in editor →".
- Click background or press Esc → clear selection.

### What Sight v5 is for

It answers one question: **"Is my universe healthy? If not, where does it need to be handled?"** Layer 1 (the visual foundation) shows you the shape; Layers 2–4 (coming in future MIGs) add diagnostic + recommendation + local-AI coaching.

For the full feature set + canonical design contract, see `Constellation-Sight-Concept-Paper-v3.1.md` and the help topic `Sight v5`.

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

The **Calendar** is a full-page month view, opened from the **left dock** (the calendar icon). Days with notes or due tasks are marked with coloured **dots**. The header shows the month in your chosen calendar; if you've set a **secondary calendar**, a subtitle below shows that calendar's equivalent range (e.g. a Gregorian month shows its Hijri span, "Dhul-Hijjah 1447 – Muharram 1448 AH").

**Clicking a day.** Each day cell is interactive:

- **Click the empty space (or the day number)** → opens (or creates) that day's **daily note**. Clicking a date that already has a daily note simply **opens** it — it never makes a duplicate.
- **Click a dot** → opens that specific item. A **gold** dot is the daily note; a **purple** dot is another note edited that day; a **red** dot is a task due that day. (Colours are themable in the Style Setter → Calendar.) If a day has several notes or tasks, clicking the dot shows a small **list** to pick from.
- **Click a task dot** → opens the note **scrolled to that task's line**, ready to edit. In the task list you can also **tick a task's checkbox to complete it** right from the calendar — completed tasks drop off immediately. Only tasks that carry their own `📅 YYYY-MM-DD` appear on the calendar (the date is what places them on a day).

**Cultural calendars (eight).** In **Settings → Calendar** you can set the **calendar system** — **Gregorian, Hijri (Islamic), Solar Hijri (Persian), Hebrew, Indian (Saka), Buddhist, Chinese, or Korean** — and the whole month grid switches to it, showing both the chosen-calendar date (large) and the Gregorian date (small) in each cell, plus the moon phase. Each month header shows the month **name, its number in parentheses, and the year** — the number helps with calendars whose month order is unfamiliar. The **Chinese and Korean** calendars are *lunisolar*: they sometimes insert a **leap month** (闰六月 / 윤6월), which the calendar shows as its own page so navigation never skips or doubles it. The Hijri calendar uses an accurate astronomical engine; sacred months are highlighted and Islamic events are marked. You can also choose the **week start** (Sunday/Monday) and toggle the **week-number column**.

**Hijri calendar options.** Under **Settings → Calendar → "Hijri calendar (Islamic)"** there are two extra controls:

- **Calculation method** — **Astronomical (Lunar Conjunction)**, which follows the true new-moon (most accurate, the default), or **Tabular (al-Tawfīqāt al-Ilhāmiyyah)**, the classical arithmetic cycle.
- **Month correction** — nudge a Hijri month's start by ±1 or ±2 days to match a **local moon sighting**. Pick the Hijri year and month, choose an offset, and click **Set**; the correction applies to that month and every month after it. Your corrections are listed (each removable), with a **Clear all** button.

Both settings (and your corrections) are saved **with your universe**, so they travel across your devices.

**Chinese & Korean display options.** Korea uses the Chinese lunar calendar, so the two share identical dates — what distinguishes them is the script and the year. When either is your main or secondary calendar, **Settings → Calendar** shows two extra controls: a **year display** (Chinese: the sexagenary cycle 丙午年, the plain year, or both; Korean: the **Dangi** era 단기 4359, the year, or the sexagenary 병오년) and **month names** — *native script* (五月 / 5월) or *phonetic*, the month's pronunciation written in your own language (English "Wǔyuè / Owol"; Arabic "وُو-يوي / أوه-وُل").

**Styling the calendar.** Open the **Style Setter** (left dock, or **Settings → Style Setter**) and pick the **Calendar** surface to restyle every part — each element has its own **colour and text size** (day numbers, the cross-reference date, the month pill, weekday headers, week numbers, the moon glyph, the Today highlight, grid lines, and the note/task/event dots), plus the calendar **font**. A live, full-size preview updates as you edit; click **Keep** to apply.

> **Daily-note filenames always stay Gregorian** (`YYYY-MM-DD`) regardless of the displayed calendar — so your files stay portable and sort correctly. The cultural date is shown in the calendar (and can be recorded in the note's frontmatter).

The Calendar fully serves daily notes: click any day to open it, or run the **"Daily Note"** command (command palette) to jump to today.

**Recording a cultural date in a note.** Two opt-in tools write the cultural date into a note's **properties** (the filename always stays Gregorian `YYYY-MM-DD`):

- **Daily-note Hijri stamp** — *Settings → Calendar → "Stamp the Hijri date in daily notes."* When on (available only while the Hijri calendar is your **main or secondary**), every **new** daily note gets a `hijri:` line, e.g. `hijri: 1448-01-06`. Notes you already have are never touched.
- **"+ Hijri" in a note's Properties** — open any note's **Properties**, hover the date, and a small **"+ Hijri"** button appears (plus "+ Jalali", "+ Hebrew", and so on — **one button per non-Gregorian calendar you've selected**). Click it and Constellation reads the note's Gregorian date and adds the equivalent, e.g. `jalali: 1405-03-30`. The Korean button writes the **Dangi** year; a Chinese/Korean **leap month** is marked with an `L` (e.g. `chinese: 2025-06L-17`). If the note has no date property, it uses the file's creation date.

---

## 15. Constellation Base & Lenses

A **Constellation Base** turns a set of notes into a live table — one row per note, one column per property — that you can sort, edit, and reshape **without moving any file**. The same query engine powers it whether you open it as a full tab or embed it inside a note.

> The Base is **non-destructive**: it reads your notes in place. A `.base` file holds only the query (which notes, which columns, what order); your Markdown is never copied or changed by the table itself. Governing principle: **"Strong yet Simple, by default"** — the table opens familiar and uncluttered, with the deeper cognitive columns one click away.

### The full-tab Base

Open a `.base` file and it fills the tab as an interactive table:

- **Name column first** — click a note's name to open it. Every matching note is a row, with **no row limit** (the table is virtualized, so thousands of notes scroll smoothly).
- **+ Add column** — pick from **Your fields** (frontmatter properties found in your notes) or **Constellation** (built-in: Name, Path, Created, Summary).
- **Sort** — click a header to cycle ascending → descending → off; use the **Sort** panel to sort by several columns at once.
- **Search this base** — the search box in the header filters the rows as you type, matching a note's name *and* the text of every visible column. The count badge shows `matching / total` while you filter (e.g. `4/7684`). It searches every script — type Arabic to find Arabic titles. Filtering is instant even on thousands of rows.
- **Letter rail** — on a base with 50+ rows, a slim strip of letters appears at the table's edge, built from the first letters of your actual note titles (so it shows A–Z for English, أ ب ت… for Arabic, and the right letters for any other script). Click a letter to jump straight to the first note starting with it — if the table isn't already sorted by Name, it sorts by Name first, then jumps.
- **Right-click a row** — opens the standard note menu: Open, Open in new tab, Bookmark, Copy path / name, Reveal in file tree, Open in default app, Show in system explorer, Style… (Renaming, moving and deleting are deliberately not offered here — do those from the file tree.)
- **Edit in place** — double-click one of your frontmatter cells to change it (list fields like `maturity` show a dropdown of valid values in their natural order); the change is written to the note's YAML on disk. Name and Created are read-only.
- **Reorder** — drag a column header sideways to move it.
- **Convert older bases** — a `.base` from Obsidian or an earlier Constellation is detected and left untouched, with a one-click **Convert to Constellation Base** offer.

**New Base** writes a small YAML file for you:

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

(See the in-app help topic **Bases** for the full walkthrough.)

### Embedded Base blocks (inside a note)

You can drop a Base into the body of any note using a ` ```base ` fenced code block. The minimal form is just the view:

````markdown
```base
view: table
```
````

When you view the note, the block becomes the same interactive table. In Live Preview, click the block to expand and edit it. The built-in dimensions you can show as columns:

| Dimension | What it shows |
|-----------|---------------|
| `note.name` | The note's filename (without `.md`) |
| `note.path` | The note's full path |
| `note.created_at` | The note's creation timestamp |
| `note.headline` | The note's auto-generated summary |

Plus any of your own frontmatter properties (added from the **+ Add column** picker).

**Federation:** by default a Base reads across the active Universe **and** every linked cUniverse. To limit results to the active Universe only, set `federation: active` under `scope` in the YAML. Notes from a linked Universe are read-only — you can view and sort them, but editing is reserved for notes you own.

### Five Acts — built-in lenses for the Five Acts of Knowledge Creation

The sidebar's **Five Acts** section (above Workspace Bases) lists Constellation-curated host notes — markdown files at `{universe}/Five Acts/*.md` that come pre-loaded with a `base` block. v1 ships with one:

- **Observation — Recent Captures** — a federated list of the 20 most recently captured notes across your active universe + cUniverses. Click it to see what you've been working on lately.

You can edit these host notes freely. Constellation will not overwrite your edits — if you change the YAML, your version stays. If you delete the file, Constellation will re-create it on next launch (transfer-on-edit policy).

### Legacy Lens panel

The older Lens panel (filter by tags, folders, properties; save configurations) is still available under **Settings → Panels → Lens**. It is non-destructive — your existing saved lenses keep working.

---

## 15b. Panels

Constellation's panels — Backlinks, Outgoing Links, Properties, Tags, Sky View, Tasks, Calendar, Knowledge Health, Provenance, and Review Pulse — can each be placed in one of four positions via **Settings → Panels**. (The former Link Dashboard panel retired into the Circulatory System, a full-page view opened from the left dock.)

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

The right sidebar is the **open note's context** — every tab answers "tell me about *this* note." The **Tags** tab shows only the open note's tags; the whole-universe tag list lives on the **Dashboard**. The **360.3D Inspector** can now be placed/hidden like any other panel (Settings → Panels).

### Right sidebar text size

Each right-sidebar panel has its **own** text size, set in the **Style Setter** → **Components** → **Right Sidebar Text** (a slider per panel: Properties, Backlinks, Tags, Sky View, Tasks, Health, Provenance, Review, 360.3D, Source Review; 70–140 %, default 100). Scaling is **text-only** — spacing is unchanged — and applies only to the right-sidebar copy of each panel. Drag a panel's slider while that panel's tab is open to see it resize live.

### Workspaces

Panel placements are saved and restored with workspaces. Older workspaces (saved before this feature existed) leave the current layout unchanged when loaded.

### Structure (structural links)

The **Structure** panel shows where the open note sits inside a larger *work* — a book, a screenplay, a course, a Map of Content. It answers a different question from the Backlinks and Outgoing Links panels. Those answer *"how does this idea relate to another idea?"* (the thinking links — supports, contradicts, causes…). Structure answers *"where does this note sit in the whole work I'm composing?"* — Book → Part → Chapter → Scene.

This is the **compositional spine** of a work: the table of contents, the ordered outline. It is deliberately kept **out of** every thinking, maturity, and connection measure — placing a note "under a Book" never changes that note's maturity, its connection counts, or its presence in Sky View. A table of contents is authorship, not a claim to be judged.

**The two kinds of structural link** (you only ever type one side — Constellation works out the reverse for you):

- **`parent`** — *this note's* place under one parent (e.g. a chapter declares the part it belongs to).
- **`contains`** — *this note's* ordered list of children (e.g. a book lists its parts in reading order).

**Authoring a structural link** — open the note's **Properties** (the Properties tab in the right sidebar, or the properties block at the top of the note):

1. Click **+ Add property** and type the key `parent` or `contains`.
2. In the value, type the **target note's name** — just the name, e.g. `Part I - The Cartographer`. Constellation wraps it into a `[[link]]` for you; you do **not** type the brackets. (If you paste a name that already has brackets, it still stores cleanly as a single `[[name]]` — never a double-wrap.)
3. For `contains`, add each child as its own chip, in the order you want them to read — that order becomes the outline order.

Structural links **rename safely**: rename a chapter and its place in the structure follows automatically, because the link points at the note, not at a frozen piece of text.

**Reading the Structure panel** — open the **Structure** tab in the right sidebar (just after Backlinks):

- The panel shows the **whole work** as an indented outline (teal bullets), headed **OUTLINE** with a count of the descendants — not just the open note's own children.
- The note you're currently viewing is **highlighted** ("you are here") within that outline.
- A **breadcrumb** along the top shows the path up the spine (e.g. *The Atlas of Lost Places › Part I › Chapter 1*). Click any crumb — or any outline row — to jump to that note.
- A **Whole work ⇄ This note** toggle (top-right of the panel) switches between the entire work and just the open note's own subtree. It appears only when the note actually has a parent, so the two views differ.
- If the structure accidentally loops back on itself (note A's parent is B, and B's parent is A), the outline draws the chain and then stops cleanly, marking the cut point with a small **↻**. It never hangs.

**Resolving a conflict (Contested).** If two notes both claim the same child — one through the child's own `parent`, the other through a `contains` list — the panel flags that row as **Contested** (an amber ⚠ badge naming the other claimant) rather than silently dropping it. Two one-click buttons resolve it:

- **Keep** — keep the child's own declared parent (this note releases its claim on the child).
- **Move here** — accept this note as the parent (the child's `parent` switches to this note).

Either button updates the note files directly and refreshes the outline. Nothing is ever changed without your click.

---

## 16. Settings

Access Settings from the sidebar gear icon or `Ctrl+,`.

### Dashboard

- Universe overview and statistics

### Appearance

- Color scheme (Light / Dark / System)
- Accent color
- **Themes** — pick from six built-in themes, create custom themes (five-color editor), import themes from the Obsidian Community registry (200+ themes), or import a `.json` theme file. Delete any custom theme with the ✕ button on hover.
- **Title alignment** (note title start / centre) and **Living Link Lifecycle** (link-weight decay on/off + half-life).

*Everything else visual — fonts, sizes, the whole interface chrome, the editor look, link-type colours, and saved Styles — now lives in the **Style Setter** (its own tab in the Settings sidebar, below).*

### Style Settings → retired into the Style Setter

The standalone **Style Settings** tab has been **retired** — every control it had now lives in the **Style Setter** (below), which covers all of them and adds surfaces it never had (breadcrumb, note summary, the Universe panel, per-script fonts). For reference, that styling surface includes:

- **Colors** — background, surfaces, text (normal/muted/faint), accent, borders, state colors
- **Typography** — interface / note / code font sizes, H1–H6 sizes, heading weight, line heights, paragraph spacing
- **Layout & Shape** — small/medium/large corner radii, border widths, shadows, editor readable line length, side margins
- **Shadows** — the drop-shadow depth for **modal dialogs**, **pop-up menus/pickers**, and **tooltips**, each a preset (None / Soft / Medium / Strong / Dramatic); set one and every surface of that class matches
- **Components** — ribbon dock, sidebar action toolbar, layout bar (pane toggles), top bar / tab strip **(plus tab-bar extras: the new-tab “+” button, its bulb icon, and the tab-scroll arrows)**, status bar, right sidebar (inspector), file explorer (Universe notes, child universes, libraries, folders, notes), buttons, tags, callouts — each with independent size, radius, color, and where applicable, active-state styling
- **Editor** — link color/hover/decoration, inline code color/background/radius, blockquote bar width/color, cursor color, selection background, **highlight (background · text · radius, shared across `==`, `<mark>`, and the toolbar H)**, **URL color**, **markup-mark color** (the in-editor `#`/`**`/`==` syntax marks), and the **link-traversal (×N) chip** (background · text · radius)
- **Panels** — knowledge-health card, provenance tag, task badge, stale-review marker, 360 markers, the **traversal chips** (the `×N` badge's colour per lifecycle tier), and the **Link tooltip** — the explanation box Constellation draws when you rest the pointer on a link row, its `×N` badge, an annotation, or a note summary. Background · text · border · radius · **line height** · max width · padding. *Line height is the one to reach for if marks above the letters — Arabic, Persian, Urdu or Hebrew — look cramped against the top of the box.* Its shadow is not set here: it follows the shared **Shadows → tooltips** preset, so every tooltip in the app keeps the same depth.

**Import / Export** — toolbar at top of the tab:
- Paste from clipboard (one-click)
- Import / Paste (textarea with Merge or Replace)
- From file (.json)
- Copy (current values to clipboard)
- Export (.json)

The format matches Obsidian's Style Settings plugin exactly, so you can share settings between Obsidian and Constellation.

Changes auto-save to the active theme; if you edit a built-in theme, it is auto-cloned into your custom themes so changes persist without modifying the original.

### Styles

A **Style** is a complete, named look — theme, fonts, link colors, pill shape, typed-link display, Sky View, layout, and behaviour — saved under one name and switchable with a click. Styles are **app-global** (shared across every Universe) and live in the **Style Setter** as your **Saved styles** — apply one with a click; hover a row for Update / Export / Rename / Delete.

- **Save** — *+ Save current style…* → name it, tick which sections to include, Save.
- **Apply / Rename / Duplicate / Delete** — on each card.
- **Export / Import** — share a Style as a `{name}.constellation-style.json` file (⤓ on a card to export; *Import…* at the bottom to add one).

Applying a Style **merges** its link colors into the current Universe (your custom link types are never deleted). A Style carries only visual preferences — never secrets, tokens, or paths — so it is safe to share. (Distinct from a *Theme*, which is colors + CSS, and *Style Settings*, which are per-theme tweaks: a Style bundles all of those, plus behaviour, under one switchable name.)

### The Style Setter

The **Style Setter** is the **single home for all styling** — a full-page design studio you open from its own **Style Setter** tab in the Settings sidebar (the ✦ entry), or the **crosshair icon above the dock's gear** to jump straight into inspect-mode (hover any part of the app and click to style it). The panel is **resizable** (drag the bottom-right grip; it remembers the size). Down the left you pick a *Surface* to style — **Interface** (file tree, status bar), **Components** (dock, toolbars, tabs, buttons, tags), **Editor** (the note — breadcrumb, headings, bold, links, code, blockquote, and the **note summary** with its own colour/font/size/weight/italic), **Global** (shades, accent, corners, per-script fonts — with a live sample card), **Links** (typed-link colours + display), and the plugin surfaces (Sky View, CNS, OrgChart, Index, Cataloger, Shell) — with your **saved styles** listed at the bottom to apply with a click. (Built-in themes live in **Settings → Appearance**.)

There are two ways you see your edits. The **Editor**, **Sky View**, and **CNS** categories show a **preview in the centre** (a sample note, a labelled bubble board, or a miniature gravity well, filling the zone) — click a part of it and its controls appear on the right, updating instantly. The CNS category carries the well's **Background**, **Hover label background / text**, and **Text size** (the label pill scales with the text; the background applies on Keep, the label settings on the next CNS open). **Every other category** docks the panel to one side and goes see-through, and your edits appear on the **real app, live** — change the status-bar colour or the dock width and the actual chrome restyles as you drag (a green **● live** tag marks this). The **Links** category shows each of the eight types as its real coloured **pill** — click a pill to recolour it (live everywhere) — plus the **Colour typed links** / **Show type labels** switches, the **pill shape**, and a reusable **Saved colours** palette.

**Right-sidebar panels and dialog backdrops.** Two more surfaces round out the Setter. The **Panels** category restyles the small pieces inside the right-sidebar panels — the Knowledge-Health summary cards, the Provenance panel's *External* source tag, the Tasks panel's due badges (Overdue / Today) and tag pill, the Review panel's *Stale* notice, the 360.3D matrix's Tension / Fragile / Blind-spot markers, and the Backlinks / Outgoing **×N** usage chips (each wear-tier — emerging → established → load-bearing → stale — its own colour) — each with a live preview in the centre. And under **Global**, **Overlays → Dimmed opacity** sets how dark the app dims *behind* any dialog box: one slider governs every modal's backdrop at once (0 % = no dim, 100 % = near-black). Context menus and small pop-ups stay undimmed by design — only true dialogs dim.

**Fonts, named colours, and inspect.** Every font picker — Interface, Note, Code, and the file-tree fonts — lists the fonts **installed on your computer**, each shown in its own typeface (with a curated fallback if your system blocks detection). The **Saved colours** palette can be **named**: click **Manage** beside it to label a colour, rename it, or remove it — removing is a deliberate **✕ → Remove / Cancel**, never an accidental right-click. And **⌖ Inspect** now reaches the sidebar's **library** and **child-universe** rows and generic **buttons**, on top of the chrome it already covered.

**Sky View canvas background.** The Sky View surface has a **Canvas** element — set its **Background** colour (Style Setter → Sky View → Canvas) to give the graph its own backdrop, **independent of the panel/sidebar colour**. A deep canvas makes the bubbles pop; left unset, it follows the panel surface (the default look). The same colour applies to the small Sky View on the second screen, and the Setter's preview card shows it live as you pick.

Click **Keep** to save the look **for this Universe** (it survives a restart); **Discard** (or **✕** / **Esc**) throws away unsaved edits and the app snaps back; **Reset** returns to the plain theme. Nothing is saved to disk until you Keep.

To reuse a look, save it as a named **Style.** Type a name in the top "draft:" field and click **+ Save current as a style** — it appears in the **Saved styles** list (bottom-left), is app-global (reusable across every Universe), and captures the look you designed in the Setter, not just a theme. Click a saved style to apply it; hover its row for **↻ Update** (overwrite it with your current look), **⤓ Export**, **✎ Rename**, and **✕ Delete**. *(Built-in themes — Midnight, Daylight… — stay in **Settings → Appearance**; the Setter holds your saved styles and the live look.)*

### Link Types

The **Style Setter → Links** category is where each link type's **colour** is set. Click a type's swatch to recolour it; the change reflects **live** everywhere — the typed links in the editor and the coloured pills in the Backlinks / Outgoing panels. To restore the originals, use the **↺ Reset this element** button at the top-right of the Links controls — it resets the whole Links element at once (the eight built-in colours back to their defaults, both display switches back on, and the pill shape back to standard); your own custom link types keep their colours (they have no "default"). Two display toggles (both on by default) control how typed links appear: **Colour typed links** (draw each in its type's colour) and **Show the label above** (the type name above the link in the editor). The label and pills are shown in the **note's main language**, not the interface language — an Arabic note shows `يدعم`, an English note shows `supports`.

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
- **Editor**: RTL text editing with **Word-style caret behavior** (see below)

### How the caret and arrows behave in Arabic and bilingual text

Constellation's editor follows the same logic Microsoft Word uses on Windows, so your muscle memory carries over:

- **Arrow keys move by one character of the *text*, in reading order** — not by one position on the screen. In pure Arabic or pure English this looks exactly like the arrow you pressed. At a seam between Arabic and English (e.g. an Arabic sentence containing an English word), the caret steps cleanly through each character in writing order and "hops" across the seam — that hop is correct, and it's what stops the caret from feeling stuck at the boundary.
- **Home** goes to the reading *start* of the line — the **right** edge on an Arabic line; **End** goes to the reading *end* — the **left** edge. Pressing **Enter** on an Arabic line places the new-line caret on the **right**.
- **Triple-click** selects the paragraph's **text** (not the empty space to the side of it). **Double-click** selects a word.
- A **Latin word at the end of an Arabic line** keeps a clear, stable caret position instead of losing its direction.

### Selecting and navigating by unit

Every unit of text has a fast selector, identical in Arabic, English, and mixed notes:

- **Word** — double-click. **Sentence** — **Ctrl+click** anywhere in it, or **Ctrl+Shift+S** at the caret. Sentence detection understands Arabic punctuation: **؟ ۔ !** and the full stop end a sentence, while the Arabic semicolon **؛** is a pause *inside* one — and decimals like 3.14 never split. (Ctrl+click replaces the old add-a-cursor gesture.)
- **Line** — **Ctrl+L**. **Paragraph** (the block between empty lines) — **Ctrl+Shift+L**, or triple-click. Highlights hug the text — on an Arabic line the selection stops at the words instead of stretching across the empty left side.
- **Screenful** — **Shift+Page Down/Up**. **Everything** — **Ctrl+A**.
- **Move by paragraph** — **Ctrl+↓** jumps to the next paragraph's start, **Ctrl+↑** to the current one's (again for the previous). Add **Shift** to select paragraph-by-paragraph.

### Forcing a paragraph's direction

Sometimes the automatic detection isn't what you want — an Arabic paragraph opening with an English brand name, or an English paragraph you want read right-to-left:

- **Press and release Right Ctrl+Shift** → the paragraph the cursor is in becomes **100% right-to-left**. **Left Ctrl+Shift** → **100% left-to-right**. (The Microsoft Word convention.)
- It fires **on release**, with no other key in between — so Ctrl+Shift+S and friends keep working untouched.
- The override is a **hard** one (it beats auto-detection), applies to the whole paragraph or every paragraph a selection spans, and is stored **inside the text** as an invisible direction character — it survives restarts and sync, and travels with the text into Word or Obsidian.
- One **Ctrl+Z** undoes it. Markdown stays safe: lists, headings, and quotes keep their markers; code blocks, tables, and lines that *begin* with a #tag are deliberately left untouched.

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

### Write integrity

Settings → **Security & Privacy** shows a **Write integrity** readout — a window into Constellation's *write journal*, the behind-the-scenes log that records every note-write so any data anomaly is traceable to a single file (the safety system built after a corruption incident).

- **Writes journaled** — how many note-writes the journal has recorded.
- **Anomalies** — writes that looked wrong (an incoming note's identity not matching the file on disk). A healthy system shows **✓ 0**; if any were ever recorded, the count turns red and shows the **most-recent-anomaly date**, so a long-since-fixed incident reads as stale, not current.
- **Monitoring (shadow mode)** — the journal currently watches and logs but doesn't block writes; full enforcement is a later step.
- **Open journal folder** — opens the folder holding `write-journal.jsonl` for inspection.

The readout re-reads every time you open the section. You don't need to act on it — it's a transparency window confirming the write-safety machinery is healthy.

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

1. Open a note, then open the **Tension** tab in the right sidebar (it appears as a panel alongside your note properties). The analysis starts the moment the tab opens — you will briefly see *"Analyzing library…"* while Constellation scans the library the open note belongs to. The result is reused instantly until you move to a note from a different library.
2. If your library has fewer than 50 linked notes, you will see a progress indicator showing how close you are to activation.
3. Once active, the panel shows four collapsible sections: Contradictions, Orphan Notes, Structural Gaps, and Single Points of Failure. If the same pair of notes is linked as contradicting more than once, it appears as a single row with a ×N count instead of repeated entries, and long lists scroll.
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
- On large libraries (thousands of notes), the first open of the Inspector for a given note takes a moment while it computes. During that moment you'll see a brief **"Loading 360° view…"** spinner, and — importantly — **the rest of the app stays responsive** (you can keep clicking, scrolling and typing); the computation runs in the background rather than freezing the window. When you switch notes you'll see the spinner again for the new note rather than the previous note's data lingering. Subsequent opens of a note you've already viewed are instant.

---

*Constellation User Manual — Version 0.1.0 — March 2026*
*uconstellation.world*


## The Note-Context Right Sidebar

The right sidebar's panels are **note-context**: each answers a question about the note you have open right now.

- **Tags** — the open note's tags. (The universe-wide all-tags browser moved to the Dashboard.)
- **Knowledge Health** — the open note's intellectual *tensions*: contradicted by another note, orphaned (nothing links to it), a single point of failure, or in a structural gap. The whole-library Knowledge Health dashboard is a separate full-page view (Command Palette → Knowledge Health).
- **Tasks** — the open note's tasks. The whole-universe task agenda is a **Tasks** button in the left dock (full-page, with filters, grouping, and search; it follows the universe theme and has a *Style Setter → Global Tasks* tab with colours + a Text-size slider). Natural-language due dates: type `@today`, `@next week`, a weekday, or `@in 3 days` and accept the suggestion to pin a real date — toggle in Settings → Editor.
- **Source Review** — the open note's pending source suggestion. The universe-wide review queue, with the bulk Approve-all / Reject-all tools, lives in the full-page **Cataloger** (left dock). The Cataloger has a *Style Setter → Cataloger → Text size* control.

A note-context panel shows **"No note selected"** when no note is open.

---

## 22. Review Pulse (the Reviewer)

**Review Pulse** is Constellation's *call-back list*: the one surface that comes to you with the notes that need attention, ranked by urgency, each with a plain-language reason and a prescribed remedy. Open it from the **🕐 clock** icon in the left dock.

It is a two-column **master-detail** surface. The left column is a queue grouped into **six lenses** (all always shown; empty ones greyed with a 0; each collapsible):

- 🥀 **Stale** — a note this one leans on changed after you last reviewed it.
- 🔄 **Due for Review** — the review interval elapsed.
- 🧠 **Mental-Model Checkpoints** — an assumption/model to re-examine.
- 🔗 **Orphan — connect me** — real content that nothing links to yet (an *alarm* to connect it, not clutter).
- ⚠ **Fragile — shore me up** — many depend on it, little holds it up.
- 📝 **Never reviewed** — never given a first read-through.

Click a note and the right pane **diagnoses and prescribes**: a summary (always shown), the "why now," the one healthy remedy, and a **Priority** (0–100) rendered as a readable recipe bar — its segments (Time pressure, Depended-on, Maturity, …) add up to the number, so you can see *why* it ranks where it does. **Drag the slider** to override the priority (badged "manual," with "Reset to computed"). Act with **✓ Reviewed**, **🔗 Connect** (orphans), **👁 Snooze 7d**, or **🗄️ Dismiss**; hand off to **Open in editor**, **Full context (360°)**, or **Classify**. Opening a note from the Reviewer leaves a **‹ Reviewer** button in the top tab strip to return.

Each note's own status also appears in the right-sidebar **🕐 Review** tab when that note is open — the same priority + actions, scoped to that note.

**Settings → Review** sets the *staleness grace period* (days, minimum 1): a dependency change only flags a note as stale once that many days have passed since your last review. Keep it higher if you make many small edits.

---

## 23. Suggested Connections

Constellation is for *formulating* knowledge, and knowledge is connection. **Suggested Connections** finds the notes already in your Library that are most related to the one you're looking at — the relatives it should link to but doesn't yet — and turns any of them into a **typed link** in a single click. It is "more like this," but for thinking.

**Every suggestion is typed.** When you accept one, Constellation asks *how* the two notes relate — supports, contradicts, exemplifies, derives-from, and so on, or simply **associative**. A typed link is a piece of reasoning you can later read, search, and challenge; the feature never adds links in bulk and never adds an untyped link silently. (See **Knowledge Formulation** and **Properties**.)

**How it finds them.** Candidates come **only from your own Library**, ranked against Constellation's live search index by the most *distinctive* shared vocabulary — the rare, telling words, not the common ones. Each suggestion shows the **shared terms** that explain why it surfaced, so you never accept a black-box guess.

**Five places, one list.** The same suggestion list appears in the **Reviewer** (🕐, for notes it flags as *orphan* or *fragile*), the **Backlinks tab** (right sidebar), the **360° Inspector**, the **Health tab**, and **Sky View** (🌌 — right-click any star → **Suggest connections…**).

**Inbound vs outbound — and why you don't choose.** Diagnostic surfaces (the **360° Inspector** and **Health tab**) suggest **inbound** connections — *which notes should point **here***. General surfaces (the **Backlinks tab** and **Sky View**) suggest **outbound** connections — *what this note should point **to***. The surface picks the direction that fits its job; you pick the note and the type. (A future update will let you switch the direction yourself.)

**Using it.** Under the **Suggested connections** heading you'll see related notes ranked closest-first, each with its shared terms. Click a candidate's **Link** button → in the little **"How do they relate?"** menu pick the relationship type → the typed link is created **instantly** and the suggestion drops off the list. It then lives in the note's **properties** and appears in its backlinks/outgoing links and across the graph. If nothing truly fits, leave them — or, in the Reviewer, mark the note a deliberate **standalone**. Suggested Connections proposes; you decide.

**Local, private, non-blocking.** Suggestions are computed on demand from your Library only — nothing leaves your device — and gathering them never blocks your typing (you'll see a brief "Finding related notes…" while it works). The suggestions, the shared-term hints, and the relationship types all appear in your chosen language and mirror correctly for right-to-left scripts.


### Properties styling (Style Setter)

Open the **Style Setter** (Settings → Appearance → ✦ Open Style Setter, or its own tab) and pick the **Properties** category to restyle the small tags inside a note's frontmatter. Two elements: **Property tags** (the ordinary `tags`-style chips — Tag background, Tag text, Tag radius 0–20 px, Height 14–32 px) and **Taxonomy pills** (Background, Text, Radius 0–20 px). A live preview in the centre updates as you edit; every value starts at exactly today's look, so nothing changes until you touch a control. Click **Keep** to save for this Universe.

### Cognitive colours (Style Setter)

The **Cognitive colours** category gives you **one shared colour per cognitive state**, so every surface that shows that state agrees. Five sets:

- **Maturity** — Seed, Sapling, Evergreen, Canonical, Wilting.
- **Confidence** — Hypothesis, Evidence, Established, Contested.
- **Origin** — Received, Discovered, Mixed, None.
- **Stage** — Spark, Birth, Growth, Maturity, Dormancy, Archival.
- **Match category** (why a search result matched) — Title, Content, Tag, Wikilink, Property, Semantic, Structured.

The behaviour is **unify on demand**: nothing changes until you pick a colour. Each surface keeps its current colour as a fallback, and the moment you set a state's colour here, **every** surface that shows that state — file tree, tabs, the note inspector, the in-editor search highlight, the match badge, and the search-result highlight — snaps to your colour at once. Leave a state untouched and it looks exactly as before. Click **Keep** to save.

### Right-click menus

Constellation gives you a context menu in three places, each offering only the actions that fit where you clicked:

- **Right-click the note body** — Add link / Add external link; **Format ▸** (Bold, Italic, Underline, Strikethrough, Highlight, Inline code, Math, Toggle comment, Superscript, Subscript, Clear formatting); **Paragraph ▸** (Bullet/Numbered/Task list, H1–H6, Body, Blockquote); **Insert ▸** (Footnote, Table, Callout, Horizontal rule, Code block, Math block, Image); Cut / Copy / Paste / Paste as plain text / Select all; and **Style…** (opens the Style Setter on the **Editor** category).
- **Right-click a frontmatter property row** — Copy value, Copy name, Remove property, Add property; then the same editing menu as the body; and **Style…** opening the Style Setter on the **Properties** category.
- **Right-click a search result** — a **safe** subset: Open, Open in new tab, Reveal in tree, Copy link, Copy path, Bookmark, Show in explorer, Open in default app, and **Style…** (the **Cognitive colours** category). By design there is **no Rename, Move, or Delete** here — the search panel does not keep an up-to-the-second copy of the file tree, so destructive actions stay in the file tree where the view is always current.

Each **Style…** entry lands on the category for the thing you right-clicked, so you never have to hunt for the right controls. Every menu item, category name, and state label appears in your chosen interface language and mirrors for right-to-left layouts.


## Template Studio — recognising the kinds of note you keep writing

Open the ruled-page icon in the left dock. Constellation shows the kinds of note
already recurring in your Universe — each with a count, the fields those notes
share, and five real examples. Where your own material contains a name for a
kind (a library, a tag, words in titles), it is proposed with its evidence shown
as plain counts; otherwise the name box is empty and the naming is yours.

Press **Keep** to turn a kind into a template. The exact file, fields and
sections are stated before anything is written; optional fields are chips you
tick in. Keep never overwrites an existing template — you choose *Rename it* or
*Add these fields to it* — and **Undo** moves the new file to the trash (it
refuses if you have edited the template since). Kept kinds stay marked with the
name you gave them across restarts: the record is a `from_kind:` line in the
template file itself, so it travels with your files.

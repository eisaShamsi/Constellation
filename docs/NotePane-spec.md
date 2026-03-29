# eNotePane Specification
## The Ultimate Note — Fast, Smooth, Better Than Obsidian

---

## 0. The Philosophy of a Note

### 0.0 Why Notes Exist

A note is the most **democratic form of knowledge**. It doesn't require education to start. A child draws a line — that's a note. A professor writes a theorem — that's also a note. The medium is the same.

Throughout history, notes have been:
- **Clay tablets** in Mesopotamia — accounting, laws, stories
- **Papyrus scrolls** in Egypt — knowledge preservation
- **Margins of books** — dialogue with the author across centuries
- **Da Vinci's notebooks** — ideas that became inventions 500 years later
- **Post-it notes** — the fleeting thought that changes a meeting
- **Your phone at 3am** — the idea that won't let you sleep

The common thread: **a note is a human being saying "this matters enough to write down."**

If a note is that fundamental — that human — then the tool must **disappear**. The user should feel like they're writing on paper, not operating software.

The philosophy:

1. **The note is the product, not the app** — Constellation is invisible; the note is what remains
2. **Speed is respect** — every millisecond of delay says "the software matters more than your thought"
3. **Simplicity is trust** — a clean page says "I trust you to fill this however you want"
4. **Durability is responsibility** — a `.md` file on disk says "your words will outlive this app"
5. **No opinion** — the note doesn't judge. It accepts a single character or a 100,000-word manuscript equally

This is different from a "document editor" (Word), a "knowledge base" (Notion), or a "writing tool" (Scrivener). Those have opinions about what you should create. **A note has no opinion. It just waits.**

### 0.0.1 The Universal Note

What makes Constellation a truly effective extension tool for knowledge capturing, making, and creating is that it can accept and work with any type of notes that have been produced since the dawn of civilization.

| Era | Medium | What Constellation Accepts |
|---|---|---|
| Ancient | Clay tablet marks, tally counts | Plain text, numbers, symbols |
| Classical | Scrolls, manuscripts, marginalia | Long-form text, annotations, footnotes |
| Medieval | Illuminated manuscripts, calligraphy | Rich text, Arabic/Hebrew/CJK scripts, RTL |
| Renaissance | Da Vinci's mirror-written notebooks | Mixed-direction text, sketches (future) |
| Industrial | Ledgers, forms, tables | Tables, structured data, properties |
| Modern | Index cards, post-its, bullet journals | Short notes, lists, tags, quick capture |
| Digital | Hypertext, wikis, databases | Wikilinks, embeds, backlinks, metadata |
| Future | AI-assisted, voice-to-text, multimodal | Extensible architecture, open format |

All of these are still just **marks on a surface**. Text, numbers, symbols, connections. The format changes but the act is the same — someone recording something that matters.

**What makes Constellation universal:**

1. **Plain Markdown** — the most portable, durable digital format. Readable in 1960s terminals and 2160s devices
2. **Any script** — Arabic, Hebrew, Chinese, Devanagari, Latin, Cyrillic — first-class, not afterthought
3. **Any direction** — RTL, LTR, mixed — the note adapts to the writer, not the other way around
4. **Any length** — one character to one million words
5. **Any structure** — unstructured thought, bulleted list, formal document, data table — all valid
6. **Any connection** — `[[wikilinks]]` connect notes the way the human mind connects ideas
7. **Any metadata** — YAML frontmatter carries whatever properties the user needs
8. **Open file** — `.md` on disk. No lock-in. Take your notes anywhere, forever

**The Promise:**

> Constellation doesn't own your notes. It serves them. Whatever you've written — in any language, any format, any era of your life — Constellation can hold it, find it, connect it, and give it back to you instantly.

### 0.0.2 eNotePane's Purpose

**FocusPane** = capture ideas fast. Plain text. No distractions. Write now, organize later. The **notebook you carry** — scribble fast.

**eNotePane** = the complete note. Full markdown, full features, full tools. Where the captured idea becomes a structured, formatted, linked, searchable document. The **desk where you work** — organize, format, connect, polish.

### 0.1 Essence
A note is **an act of remembering**. It is an extension of human memory — the bridge between a thought in the mind and a permanent record outside the mind.

A note can be:
- A full thought with a title and paragraphs
- A single word or a phone number scribbled in a hurry
- Three characters: `!!!`
- A sketch, a drawing, a doodle
- A title with nothing below it
- Mixed scripts, symbols, numbers — anything
- A memory

**The only universal property of a note: someone chose to record something.**

### 0.2 Core Components

| Component | Role | Required |
|---|---|---|
| **Title** | Identity, search, classification, filename | Core — auto-generated if not provided |
| **Body** | The content itself — the reason the note exists | Core — can be empty |
| **Properties** | Metadata (date, tags, category) | Optional — enhances organization |
| **Formatting** | Structure (headings, bold, lists) | Optional — enhances readability |
| **Links** | Connections to other notes | Optional — enhances knowledge |

### 0.3 Title
The title is a **core component** because it:
- **Identifies** — distinguishes this note from thousands of others
- **Classifies** — helps organize notes into categories
- **Enables search** — finding a note among 10,000 requires a name
- **Is the file name** — title = `filename.md`

The title is **not required to start writing**. The user can type content first and add a title later. If no title is given, the system auto-generates:

```
CoNoteDDMMYYYY.HH:MM
```

Example: `CoNote26032026.14:35`

- `CoNote` — identifies it as a Constellation note
- `DDMMYYYY` — date of creation
- `HH:MM` — time of creation (ensures uniqueness)

This means: every untitled note has a **unique, chronologically sortable, informative name**.

### 0.3.1 Note Metadata (YAML Frontmatter)

The filename stays simple and human-readable. Rich metadata lives inside the file as YAML frontmatter:

```yaml
---
created: 2026-03-26T14:35:00
modified: 2026-03-26T15:22:00
title: CoNote26032026.14:35
type: fleeting
os: Windows 11 (22H2)
device: DESKTOP-A1B2C3
deviceName: Eisa's PC
universe: Two Universe UNIVERSE
library: كون عيسى
folder: /Research/Notes
tags: []
source: ""
author: ""
---
```

**Auto-populated on note creation (user never needs to type these):**

| Field | Source | Description |
|---|---|---|
| `created` | `new Date().toISOString()` | Full timestamp with seconds |
| `modified` | Updated on every save | Last modification time |
| `title` | Filename without `.md` | Note identity |
| `type` | `fleeting` (FocusPane) / `permanent` (eNotePane) | Note type for Zettelkasten workflow |
| `os` | Tauri `os.type()` + `os.version()` | Which OS created this note |
| `device` | Tauri `os.hostname()` | Unique machine identifier |
| `deviceName` | From system or settings | Human-readable device name |
| `universe` | Current universe context | Which universe this belongs to |
| `library` | Current library context | Which library within the universe |
| `folder` | File path relative to library root | Location within library |

**User-populated (optional, added via Properties panel):**

| Field | Description |
|---|---|
| `tags` | Classification tags |
| `source` | For literature notes — where the idea came from |
| `author` | Who wrote this note |
| `prev` / `next` | Zettelkasten sequence links |
| Custom keys | Any user-defined properties |

**Principles:**
- Filename stays clean — `CoNote26032026.14:35.md` or user's own title
- Metadata is searchable — Rust-side indexer reads YAML frontmatter
- Metadata is portable — standard YAML, readable by any tool
- Auto-populated silently — user never sees the machinery
- User-editable — Properties panel in eNotePane lets users add/edit
- Invisible by default — frontmatter hidden in editor, shown in Properties panel

### 0.4 Why People Take Notes

| Reason | Example | Speed Need |
|---|---|---|
| **1. Capture a fleeting thought** | Idea in the shower, a sudden insight | Instant — thought disappears in seconds |
| **2. Record information** | Meeting notes, lecture, phone number | Fast — real-time with the source |
| **3. Organize thinking** | Outline a project, plan a trip | Medium — deliberate, structured |
| **4. Communicate** | Letter, email draft, message | Medium — clarity matters |
| **5. Create knowledge** | Research, connect ideas, synthesize | Slow — deep thinking |
| **6. Produce work** | Article, report, book chapter | Slow — craft matters |
| **7. Preserve** | Journal, archive, institutional record | Varies — durability matters |

**Critical insight:** Reasons 1-2 demand speed above all else. If the app is slow, the thought is gone. A beautiful note that's slow to open is a **failed note**.

### 0.5 Design Implications

From the nature of a note, the eNotePane must:

1. **Not require a title to start** — content first, title later
2. **Not require any structure** — an empty file is a valid note
3. **Accept any input instantly** — text, numbers, symbols, mixed scripts
4. **Never block the user** — no loading, no processing, no "wait"
5. **Preserve exactly what was typed** — no auto-correction, no silent modification
6. **Auto-generate a meaningful title** — `CoNoteDDMMYYYY.HH:MM` if none provided

---

## 1. Philosophy

### 1.1 Speed Is Non-Negotiable
> "Speed and reliability are often intuited hand-in-hand. Speed is a proxy for general engineering quality." — [Craig Mod](https://craigmod.com/essays/fast_software/)

> "Editor application is the single thing that matters most to reducing overall typing latency." — [Pavel Fatin](https://pavelfatin.com/typing-with-pleasure/)

> "If you want to create digital artifacts that last, they must be files you can control." — [Steph Ango (kepano)](https://stephango.com/file-over-app)

**Rule:** Performance > Features > Appearance. Always.

### 1.2 The 5ms Rule
From Pavel Fatin's research on typing latency:
- **Best editors** (GVim): ~1ms processing latency
- **Good editors** (Sublime Text): ~8ms processing latency
- **Unacceptable** (Atom): ~50ms processing latency
- **Human perception**: Even 1-20ms delays affect accuracy and speed
- **Target for eNotePane**: < 5ms processing latency per keystroke

### 1.3 File Over App
From Steph Ango's philosophy:
- The `.md` file on disk is the source of truth
- The app is just a window into the file
- The note must outlive the app
- Never lock content in proprietary formats

---

## 2. Architecture Rules

### 2.1 The Editor Owns Its Content
**Problem we solved:** `$effect` loops caused cursor jumping and lag because Svelte's reactivity fought with CM6's internal state.

**Rule:** After mount, CM6 owns the document. The parent component NEVER writes back to the editor during typing. Communication is ONE-WAY:

```
Editor → onchange(text) → Parent stores text → Debounced save to disk
```

**Never:**
```
Editor → onchange(text) → Parent updates $state → $effect syncs back → Editor replaces document → CURSOR JUMPS
```

**Source:** Hard-won experience from our FocusPane and CodeMirrorEditor bugs.

### 2.2 No Store Updates During Typing
**Problem we solved:** `saveTabContent()` updated the Svelte store on every autosave, triggering full component re-renders (49 template references to `tab.*`).

**Rule:** Autosave writes to disk ONLY. The store is updated only on:
- Tab close (onDestroy flush)
- Tab switch (via `{#key tab.id}`)
- Note reload from disk

**Source:** Our `updateTabContent()` analysis showing full reactivity cascade on every save.

### 2.3 Zero Custom Plugins at Start
**Problem we solved:** 10+ ViewPlugins firing on every keystroke, each iterating visible lines with regex.

**Rule:** Start with ZERO custom CM6 extensions. Add each one individually, measuring performance before and after. Each plugin must justify its existence with:
1. A clear user need
2. No measurable typing latency increase (< 1ms per plugin)
3. Proper update guards (`if (update.docChanged) { ... }`)

### 2.4 Use MatchDecorator for Incremental Updates
**Problem we solved:** Our ViewPlugins rebuilt ALL decorations from scratch on every keystroke.

**Rule:** When decorations are needed, use CM6's `MatchDecorator` with `updateDeco()` for incremental updates — NOT full rebuilds. From [CodeMirror docs](https://codemirror.net/examples/decoration/):

```typescript
// GOOD: Incremental update
update(update: ViewUpdate) {
    this.decorations = this.decorator.updateDeco(update, this.decorations);
}

// BAD: Full rebuild on every change
update(update: ViewUpdate) {
    if (update.docChanged) {
        this.decorations = buildAllDecorations(update.view); // SLOW
    }
}
```

### 2.5 Viewport-Only Processing
**Rule:** NEVER iterate the entire document. Only process `view.visibleRanges`. CM6's viewport-based rendering is one of its key performance features — don't bypass it.

**Source:** [CodeMirror System Guide](https://codemirror.net/docs/guide/), Obsidian's support for million-line documents.

### 2.6 No $effect for Editor State
**Rule:** No `$effect` block shall read or write editor content (`value`, `editBody`, etc.). The only allowed `$effect` blocks are:
- Direction change (rare, guarded by `prevDir`)
- Font change (rare, guarded by `prevFontKey`)

**Source:** Our cursor-jumping bugs in both FocusPane and CodeMirrorEditor.

---

## 3. Note Characteristics

### 3.1 Visual Layout: PaperOnDesk (PoD)
- Gray desk surface: `#e8e8ec`
- White paper: `max-width: 1200px`, centered
- Paper padding: `48px` all sides
- No visible borders on editor area
- No active line highlight
- No gutter borders
- Clean, distraction-free writing surface

### 3.2 Components (top to bottom)
1. **Tab bar** — locked to paper edge, horizontal scroll, (+) new tab
2. **Breadcrumb** — library / note name, more options (⋮)
3. **Title** — editable, centered or start-aligned (setting)
4. **Properties** — collapsible, YAML frontmatter editor
5. **Toolbar** — formatting buttons, contextual script symbols
6. **Editor** — CM6 with minimal extensions
7. **Status bar** — word count, character count (at window bottom)

### 3.3 Editor Extensions (Ordered by Priority)

**Phase 1: Bare Minimum (must be instant)**
- `history()` — undo/redo
- `drawSelection()` — cursor rendering
- `markdown()` — syntax parsing (NO `codeLanguages` — saves 500KB+)
- `keymap.of([defaultKeymap, historyKeymap])` — standard keys
- `EditorView.lineWrapping` — wrap long lines
- `EditorView.editorAttributes.of({ dir })` — RTL/LTR
- `EditorView.updateListener` — immediate onchange, no debounce

**Phase 2: Syntax Highlighting (add after Phase 1 is confirmed fast)**
- `syntaxHighlighting(defaultHighlightStyle)` — colors for markdown syntax

**Phase 3: Live Preview (add after Phase 2 is confirmed fast)**
- Headings: hide `#` marks, apply font size — using `MatchDecorator`
- Bold/Italic: hide `**`/`_` marks, apply style — using `MatchDecorator`
- Links: style `[text](url)` — using `MatchDecorator`
- Each feature added ONE AT A TIME with performance testing

**Phase 4: Advanced Features (add after Phase 3 is confirmed fast)**
- Wikilinks: `[[note]]` bracket hiding + link styling
- Callouts: `> [!type]` with collapse/expand
- Code blocks: background coloring
- Tags: `#tag` styling
- Images: inline preview

**Phase 5: Productivity (add after Phase 4 is confirmed fast)**
- Autocomplete (wikilinks, tags)
- Search & replace
- Fold headings
- Line numbers (optional)

### 3.4 RTL/Bidi Support
**Rule:** Use CM6's built-in bidi support + `dir` attribute on the editor. NO custom bidiPlugin iterating lines with Unicode regex.

The browser's text layout engine handles bidirectional text natively through the Unicode Bidi Algorithm (UAX #9). CM6's `EditorView.bidiSpans` provides line-level bidi information when needed.

### 3.5 Font Handling
- Fonts configured via CSS custom properties (`--font-text-theme`, `--font-monospace-theme`)
- Per-library font overrides via `EditorView.theme()`
- NO per-line font detection (the bidiPlugin's `detectLineScript` was a performance killer)

### 3.6 UI Design Principles

The UI emerges from the philosophy: **the tool disappears, the note remains.**

1. **Invisible Chrome** — the interface should feel like paper, not software. Borders, toolbars, and controls recede until needed. The writing surface dominates.

2. **Typography First** — the most important visual element is the text itself. Title, headings, body — their hierarchy must be clear through size, weight, and spacing alone, not through decorative elements.

3. **Consistent Rhythm** — spacing between components follows a predictable pattern. The eye should flow naturally from title → properties → content without jarring gaps or cramped areas.

4. **Minimal Color** — the desk is gray, the paper is white, the text is dark. Accent color appears only for interactive elements (links, buttons, selections). No gratuitous color.

5. **No Decorative Animation** — animations serve navigation context only (collapsing/expanding, tab switching). Never decorative. Never blocking. If an animation takes more than 150ms, it's too slow.

6. **Contextual Controls** — toolbar, properties, script symbols appear when relevant and can be hidden when not. The empty page has minimal chrome. A note with tables shows table tools. A note in Arabic shows Arabic symbols.

7. **Respect the Paper Metaphor** — the PaperOnDesk layout is not just aesthetic. It frames the writing experience. The gray desk provides visual breathing room. The white paper focuses attention. The shadow gives depth. These are functional, not decorative.

---

## 4. Behavior Rules

### 4.1 Typing Must Be Instant
- Zero perceptible lag between keystroke and character appearance
- Test: type 10 characters rapidly in Arabic. If there's any delay, something is wrong.
- Measure: use `performance.now()` in the updateListener to track processing time

### 4.2 Save Is Background-Only
- Debounce: 1500ms after last keystroke
- Save writes to disk via Rust IPC (`writeNote`)
- NO store update during autosave
- Store updated only on tab close/switch

### 4.3 Tab Switch = Component Recreation
- Use `{#key tab.id}` to destroy and recreate the editor on tab switch
- This is cleaner than trying to sync editor content via props/$effect
- Cursor position and scroll position restored from tab state on mount

### 4.4 No Feature Shall Slow Typing
- Every new feature must be tested with rapid typing in Arabic AND English
- If a feature adds measurable latency (> 1ms), it must be:
  - Debounced (only run after typing pause)
  - Moved to `requestIdleCallback`
  - Or removed entirely

### 4.5 Progressive Enhancement
Features are added in phases. Each phase must pass the typing test before proceeding to the next. If Phase N fails, we fix it before adding Phase N+1.

---

## 5. Knowledge Organization

### 5.1 Philosophy: Enable, Don't Enforce

Zettelkasten, PARA, MOC, Johnny Decimal, GTD — these are knowledge organization systems created by users over decades. Constellation does not pick one. It provides the **infrastructure** that makes ALL of them possible.

> Zettelkasten is not a feature to build. It's an **emergent property** of a note system that has: atomic notes + links + backlinks + properties + search.

### 5.2 Zettelkasten Mapping

Niklas Luhmann's slip-box had two principles: one idea per note, and notes connected by links rather than categorized by folders. Constellation's architecture naturally supports this:

| Zettelkasten Concept | Constellation Implementation |
|---|---|
| **Fleeting note** (quick capture) | FocusPane — write fast, refine later |
| **Literature note** (source summary) | eNotePane — with `source` property, citations |
| **Permanent note** (refined idea) | eNotePane — with wikilinks, tags, properties |
| **Index note** (topic entry point) | MOC — a regular note filled with organized `[[links]]` |
| **Structure note** (thought sequence) | Outline note — numbered links in order |
| **Unique ID** | `CoNoteDDMMYYYY.HH:MM` — auto-generated, chronologically sortable |
| **Cross-references** | `[[wikilinks]]` — bidirectional connections |
| **Reverse lookup** | Backlinks panel — "what links to this note?" |

### 5.3 Infrastructure for Any System

The following capabilities enable Zettelkasten AND any other knowledge system:

1. **Links are first-class** — `[[wikilinks]]` are fast to create, visually distinct, clickable, and searchable
2. **Backlinks are always available** — every note shows what links to it
3. **Properties support note typing** — YAML frontmatter can carry:
   ```yaml
   type: fleeting | literature | permanent | index | log
   source: "Book Title, Author"
   prev: "[[previous thought]]"
   next: "[[continuation]]"
   ```
4. **FocusPane → eNotePane = fleeting → permanent** — the transition from quick capture to refined note is seamless. Same `.md` file, different editing experience
5. **Search across all notes is instant** — Rust-side indexing enables finding any note among thousands in milliseconds
6. **Graph view shows the connection web** — visual map of how ideas relate
7. **Unlinked mentions** — surface notes that reference this note's title without a formal `[[link]]`, helping discover hidden connections
8. **MOC needs no special feature** — a Map of Content is just a note with organized links. The system already supports it
9. **Tags for lightweight categorization** — `#topic` groups notes without rigid hierarchy
10. **Libraries as separate knowledge domains** — like Luhmann's multiple slip-boxes for different projects

### 5.4 The Promote Workflow

The natural knowledge lifecycle in Constellation:

```
Thought → FocusPane (fleeting) → eNotePane (refine) → Link → Connect → Knowledge
```

1. **Capture** — FocusPane: type the raw idea, fast, no friction
2. **Refine** — switch to eNotePane: add title, structure, formatting
3. **Type** — set `type: permanent` in properties (optional)
4. **Link** — add `[[wikilinks]]` to related notes
5. **Connect** — backlinks and graph reveal the web of ideas
6. **Discover** — unlinked mentions surface forgotten connections
7. **Build** — MOC/index notes emerge as topics accumulate

No step is required. A note can stay fleeting forever. The system enables growth, not forces it.

---

## 6. Anti-Patterns (Never Do These)


| Anti-Pattern | Why It's Bad | What To Do Instead |
|---|---|---|
| `$effect` reading editor value | Causes echo loops, cursor jumping | Editor owns content, one-way communication |
| `updateTabContent()` during autosave | Triggers full Svelte re-render cascade | Write to disk only, update store on close |
| Full decoration rebuild per keystroke | O(n) per keystroke, kills performance | Use `MatchDecorator.updateDeco()` |
| `@codemirror/language-data` import | Pulls 500KB+ of parsers | Import only `markdown()` without `codeLanguages` |
| Unicode regex on every keystroke | Arabic text regex is expensive | Use CM6 built-in bidi, or debounce to idle |
| Multiple ViewPlugins all firing | Each iterates visible ranges | Combine into one plugin or debounce non-critical |
| `position: absolute/fixed` for layout | Breaks flow, causes reflow | Use flexbox/grid |
| Inline styles overriding CSS | Hard to maintain, causes specificity wars | Use CSS classes |

---

## 7. Experiment Lab

### 7.1 Purpose
Every feature, extension, or change is tested in isolation BEFORE entering production. Nothing goes into `src/` without passing the lab.

### 7.2 Location
```
lab/
  experiments/    — one file per feature experiment
  benchmarks/     — typing latency measurement tools
  reports/        — audit results and experiment logs
```

### 7.3 Workflow
```
1. PROPOSE  → describe what and why
2. EXPERIMENT → build in lab/experiments/ (isolated)
3. BENCHMARK → measure with lab/benchmarks/typing-latency.ts
4. AUDIT    → run all 8 audit agents
5. APPROVE  → all agents must PASS
6. IMPLEMENT → merge to src/
```

### 7.4 Rule
**Nothing enters production without passing the lab.** If an experiment fails benchmarks or audit, it goes back to the drawing board.

---

## 8. Audit System

### 8.1 Purpose
Independent, unbiased validation of every change against this spec. The auditors don't accept work that fails their criteria. They are adversarial by design.

### 8.2 Audit Agents (8 total)

| # | Agent | Mission |
|---|---|---|
| 1 | **Performance Auditor (PA)** | Every keystroke < 5ms. No full rebuilds. No regex on every keystroke. |
| 2 | **Architecture Auditor (AA)** | One-way editor→parent flow. No $effect echo loops. Clean reactivity. |
| 3 | **Memory Auditor (MA)** | Zero leaks. Every timer/listener/view cleaned up in onDestroy. |
| 4 | **Spec Compliance Auditor (SCA)** | Implementation matches this spec exactly. Pixel values, behaviors, rules. |
| 5 | **RTL/Bidi Auditor (RA)** | Perfect bidirectional text. CM6 built-in bidi. No custom plugin overhead. |
| 6 | **UX Auditor (UXA)** | Instant feel. No friction. Tab switch instant. Save reliable. |
| 7 | **Code Quality Auditor (CQA)** | Clean code. No dead code. No TS errors. < 500 lines per component. |
| 8 | **Environment Auditor (EA)** | The app environment must be healthy before phase work begins. No pre-existing lag, no unresponsive UI, no memory leaks from other components. If the environment is degraded, STOP — identify, document, and fix the issue before continuing. |

### 8.3 Audit Protocol
```
Developer submits code
    ↓
All 8 agents run in PARALLEL
    ↓
Each returns PASS or FAIL with evidence
    ↓
ALL 8 must PASS to merge
    ↓
Any FAIL → fix → re-audit from scratch
```

### 8.4 Fix One, Search All
When a bug is found and fixed in one file, **immediately search the ENTIRE codebase** for the same pattern. If FocusPane had the bug, check eNotePane, CodeMirrorEditor, SecondScreenPage, and every other editor. No exception.

### 8.5 Blocking Issue Rule
**If at any point during development or testing, an issue is discovered that degrades app responsiveness, causes lag, freezes, or otherwise violates the performance contract — ALL phase work STOPS immediately.** The issue must be:
1. **Identified** — root cause documented with evidence (line numbers, reactive chains, profiling data)
2. **Documented** — written up in `lab/reports/` with severity, impact, and proposed fix
3. **Fixed** — the fix must pass the Environment Auditor (EA) before phase work resumes
4. **Verified** — the user confirms the app is responsive again

No phase work continues until the environment is healthy. A fast editor inside a slow app is still a slow app.

---

## 9. Testing Protocol

Before committing any editor change:

1. **Rapid Typing Test**: Type 20 Arabic characters as fast as possible. Zero lag = pass.
2. **Long Document Test**: Open a 5000-word note. Scroll smoothly. Type at the bottom. No stutter.
3. **Tab Switch Test**: Switch between 5 open tabs rapidly. Content loads instantly. No flash of empty content.
4. **Save Test**: Type, wait 2 seconds, close tab, reopen. Content is preserved.
5. **RTL Test**: Type Arabic, then English on the next line. Both render correctly. No layout jump.
6. **Benchmark Test**: Run `typing-latency.ts` tracker. Average must be < 5ms, P95 < 10ms.

---

## 10. Build Plan — Phases

### Overview

Each phase builds on the previous. No phase starts until the previous passes all 8 audit agents + the testing protocol. This is not a sprint — it's a craft.

```
Phase 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8
  ↑                                        |
  └── If ANY phase fails, stop, fix, re-audit
```

### Phase 0: The Skeleton
**Goal:** An empty eNotePane component that renders the PaperOnDesk layout with NO editor.
**What we build:**
- `eNotePane.svelte` — the component file
- Gray desk (`#e8e8ec`) with centered white paper (`max-width: 1200px`, `padding: 48px`)
- Title input (editable, dir="auto", centered/start per setting)
- Auto-title: `CoNoteDDMMYYYY.HH:MM` if empty on blur
- No editor, no toolbar, no properties — just desk + paper + title
**Pass criteria:** Renders correctly, title editable, auto-title works, RTL/LTR correct
**Audit focus:** SCA, RA, CQA

### Phase 1: The Bare Editor
**Goal:** Type text. That's it. Must be instant.
**What we build:**
- CM6 EditorView inside the paper
- Extensions: `history()`, `drawSelection()`, `markdown()` (no codeLanguages), `keymap`, `lineWrapping`, `dir`, `updateListener`
- One-way communication: editor → `onchange(text)` → parent stores in non-reactive variable
- No `$effect` for value sync
- `{#key tab.id}` for tab switch
**Pass criteria:** Type 20 Arabic characters rapidly with ZERO lag. < 5ms average latency.
**Audit focus:** PA, AA, MA, UXA

### Phase 2: Save & Restore
**Goal:** Notes persist across sessions.
**What we build:**
- Debounced save (1500ms) → writes to disk via Rust IPC
- NO store update during autosave
- onDestroy: flush save + update store
- Tab switch: destroy + recreate editor with new content
- Cursor position + scroll position preserved per tab
**Pass criteria:** Type → close tab → reopen → content is there. Switch tabs 10 times → no content loss.
**Audit focus:** AA, UXA, MA

### Phase 3: Breadcrumb & Properties
**Goal:** Note navigation and metadata editing.
**What we build:**
- Breadcrumb bar: library / note name, back/forward navigation, more options (⋮)
- Properties: collapsible YAML frontmatter editor
- Note type property support: `type: fleeting | literature | permanent | index | log`
- Source property for literature notes
**Pass criteria:** Properties save correctly, breadcrumb navigation works, collapse/expand smooth
**Audit focus:** SCA, UXA, CQA

### Phase 4: Toolbar
**Goal:** Formatting controls for the editor.
**What we build:**
- Toolbar with: H1/H2/H3, Bold, Italic, Underline, Strikethrough, Highlight
- Lists: ordered, unordered, task
- Insert: link, image, table, code, blockquote, horizontal rule
- Undo/Redo
- Contextual script symbols (Arabic عربي, English Aa, etc.)
- Toolbar dispatches CM6 commands — does NOT modify editor state directly
**Pass criteria:** Each button applies correct markdown syntax. Typing speed unaffected (< 5ms).
**Audit focus:** PA, UXA, SCA

### Phase 5: Syntax Highlighting
**Goal:** Markdown syntax gets visual treatment.
**What we build:**
- `syntaxHighlighting(defaultHighlightStyle)` extension
- Headings, bold, italic, code, links — colored in source mode
- Benchmark: confirm < 5ms after adding
**Pass criteria:** Syntax colored. Typing still instant.
**Audit focus:** PA, SCA

### Phase 6: Live Preview (Incremental)
**Goal:** WYSIWYG-like experience using `MatchDecorator` (incremental updates, NOT full rebuilds).
**What we build — one at a time, benchmarked individually:**
1. Headings: hide `#` marks, apply font size
2. Bold/Italic: hide `**`/`_` marks, apply style
3. Strikethrough: hide `~~` marks, apply line-through
4. Highlights: hide `==` marks, apply background
5. Inline code: hide backticks, apply monospace style
6. Links: style `[text](url)`
7. Wikilinks: hide `[[` `]]`, show display text, style as link
8. Checkboxes: replace `[ ]`/`[x]` with interactive checkbox
9. Horizontal rules: style `---`
10. Tags: style `#tag` with accent background

**Each sub-feature:**
- Built in `lab/experiments/`
- Benchmarked with `typing-latency.ts`
- Must add < 1ms to average latency
- Uses `MatchDecorator.updateDeco()` — NEVER full rebuild

**Pass criteria:** Each decoration works. Typing still < 5ms total.
**Audit focus:** PA (critical), SCA, UXA

### Phase 7: Advanced Features (Incremental)
**Goal:** Rich document features, each benchmarked individually.
**What we build — one at a time:**
1. Callouts: `> [!type]` with colored borders, icons, collapse/expand
2. Code blocks: background coloring, language label
3. Images: inline preview for `![](url)` and `![[file.png]]`
4. Blockquote line decorations: left border + background
5. Tables: table toolbar (add/remove row/column, sort)
6. Embeds: `![[note]]` transclusion preview

**Each sub-feature must pass PA audit (< 1ms added latency).**

**Pass criteria:** All features work. Typing still < 5ms total.
**Audit focus:** PA, SCA, UXA, MA

### Phase 8: Knowledge Infrastructure
**Goal:** Enable Zettelkasten and all knowledge systems.
**What we build:**
1. Wikilink autocomplete: type `[[` → instant search across all notes
2. Tag autocomplete: type `#` → instant search across all tags
3. Backlinks panel: show notes that link to this note
4. Unlinked mentions: show notes that mention this note's title
5. Graph integration: visual map of connections
6. Search: Rust-side full-text search with instant results
7. Note type workflow: FocusPane (fleeting) → eNotePane (permanent) transition

**Pass criteria:** Autocomplete responds in < 50ms. Backlinks load in < 100ms. Graph renders smoothly.
**Audit focus:** PA, AA, UXA, SCA

### Phase Summary

| Phase | What | Key Metric |
|---|---|---|
| 0 | Skeleton (desk + paper + title) | Renders correctly |
| 1 | Bare editor (type text) | < 5ms latency |
| 2 | Save & restore | Zero data loss |
| 3 | Breadcrumb & properties | Correct metadata |
| 4 | Toolbar | Formatting works, < 5ms |
| 5 | Syntax highlighting | Colors correct, < 5ms |
| 6 | Live preview (10 features) | Each < 1ms added, total < 5ms |
| 7 | Advanced features (6 features) | Each < 1ms added, total < 5ms |
| 8 | Knowledge infrastructure | Autocomplete < 50ms, backlinks < 100ms |

### Timeline Estimate

- Phase 0-2: **Foundation** — get this RIGHT, no shortcuts
- Phase 3-5: **Core experience** — the note becomes useful
- Phase 6-7: **Polish** — the note becomes beautiful
- Phase 8: **Knowledge** — the note becomes powerful

---

## 11. Sources & References

- [Craig Mod — Fast Software, the Best Software](https://craigmod.com/essays/fast_software/)
- [Pavel Fatin — Typing with Pleasure (Latency Research)](https://pavelfatin.com/typing-with-pleasure/)
- [Steph Ango — File Over App](https://stephango.com/file-over-app)
- [Marijn Haverbeke — CodeMirror 6.0](https://marijnhaverbeke.nl/blog/codemirror-6.html)
- [CodeMirror System Guide](https://codemirror.net/docs/guide/)
- [CodeMirror Decoration Example](https://codemirror.net/examples/decoration/)
- [Obsidian Editor Extensions Docs](https://marcusolsson.github.io/obsidian-plugin-docs/editor/extensions)
- [ACM Research: Effects of Text Input Latency](https://dl.acm.org/doi/fullHtml/10.1145/3626705.3627784)

---

*Document created: 2026-03-26*
*For: Constellation eNotePane — The Ultimate Note*

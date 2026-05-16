# Constellation — Universal Orientation

*A self-contained briefing for an outside AI assistant (e.g. Claude Chat) who has been asked to help with research, design, or feature ideas for Constellation. No prior context required. Last updated 2026-05-16.*

---

## 1. What Constellation Is (in one paragraph)

**Constellation is a Personal Knowledge Formulation system.** It is a local-first desktop application — Tauri v2, Rust backend, SvelteKit + Svelte 5 frontend, SQLite (FTS5) index — built for a single human doing the long, slow work of thinking. Notes are plain Markdown files with YAML frontmatter, stored on disk in directories the user controls. The application is a window into those files; the files are the source of truth. Constellation supports 15 languages simultaneously from the ground up, with per-line bidirectional text as a core architectural feature. It works fully offline, always, and ships no telemetry of any kind.

The product is built around a single thesis: **knowledge is not about storing information; it is about connecting, challenging, synthesizing, and building understanding over time.** Every feature is judged against that thesis. Features that make storage easier without helping formulation are cut. Features that hide what the user is doing — auto-summarization, AI-as-author, recommendation engines — are refused.

---

## 2. What Constellation Is NOT

These are sharp lines, not preferences. They define the product by negation.

- **NOT a notes app** in the Notion / Roam / Obsidian "capture everything" mold. Capture is welcome, but capture is the beginning, not the point.
- **NOT cloud-based.** No accounts, no servers, no sync product. The user owns sync (Git, Syncthing, iCloud, Dropbox — their choice).
- **NOT a chatbot.** Constellation does not converse with the user about their notes. There is no chat sidebar that asks "what would you like to know?"
- **NOT an AI-writes-for-you tool.** Constellation does not generate notes, does not draft paragraphs, does not autocomplete sentences. The human writes. Always.
- **NOT a file manager.** It is not designed to be the place where the user manages their filesystem.
- **NOT a knowledge management system** in the corporate-IT sense. There is no "knowledge base team", no permissions model, no audit log. It is one human's instrument.
- **NOT a graph database masquerading as a notes app.** Links are first-class, but the graph is downstream of the writing, not the point.
- **NOT a wiki.** Pages are not the unit; notes are. Notes do not need titles to exist.
- **NOT a productivity tool.** It does not measure word counts, streaks, time-on-task, or "knowledge created today." It does not gamify thinking.
- **NOT a publishing platform.** It is not designed to push notes outward to the web.
- **NOT a vector-database-with-a-UI.** Embeddings may eventually power *some* features (cross-language link discovery, source classification), but the source of truth is always the Markdown file and the typed link graph.

This negation list exists because the gravitational pull of every category above is enormous. Naming what Constellation refuses keeps the product from being eaten by adjacent ideas.

---

## 3. The Founding Distinction: Formulation vs Management

The single most important distinction in the product:

- **Knowledge management** = storing, retrieving, organizing knowledge that already exists.
- **Knowledge formulation** = creating, connecting, challenging, synthesizing new knowledge.

Most "PKM" tools (Personal Knowledge Management) are managers. They are good filing cabinets with backlinks. Constellation is a **Personal Knowledge Formulation** system. The verb is different. The features follow from the verb.

A management system makes it easier to find what you already wrote. A formulation system makes it easier to think the next thought.

This distinction is the reason Constellation has:
- Links that carry **type**, **annotation**, **weight**, **confidence**, and **lifecycle stage**, not just "this points to that"
- A **diagnostic surface** (Constellation Sight) that shows the shape of your epistemic content, not just a list of notes
- A **connection-traversal surface** (Constellation Nervous System / CNS) that surfaces synthesis points and structural gaps, not just a backlinks panel
- A **Living Link Architecture** in which links are first-class knowledge objects, stored as files on disk
- A **lifecycle vocabulary** (Spark → Birth → Growth → Maturity → Dormancy → Renewal → Archival) that treats every idea as something that grows or fades

And the reason Constellation does not have:
- A "smart" assistant that proposes notes
- An "AI summarize this folder" button
- A "trending topics in your vault" widget
- A recommendation engine

---

## 4. The Five Acts of Knowledge Creation

Constellation's cognitive frame. Every interface should support at least one of these five movements:

1. **Observation** — noticing something worth recording
2. **Connection** — relating one observation to another
3. **Tension** — surfacing where observations disagree
4. **Synthesis** — producing a new claim from the tension
5. **Conviction** — establishing how much weight the claim now bears

The five Acts map directly onto the link-confidence ladder (`hypothesis → evidence → established → contested`) and onto the lifecycle stages. They are the reason Constellation distinguishes between a *spark* (an Act 1 observation) and an *established* claim (an Act 5 conviction): the same idea moves through these stages, and the system should make the movement legible.

---

## 5. The Knowledge Hierarchy

Constellation organizes stored knowledge into four structural levels, plus an optional federation layer. No other PKM system has this depth.

```
Universe (root directory)
│  Auto-registers a default "universe_notes" Library at the Universe root
│  (the Obsidian-style flat layout). Notes/folders at the root are content
│  of this default library.
│
├── Folder, Note  (directly at the Universe root — content of "universe_notes")
│
├── Library (additional registered libraries, with their own paths)
│    └── Folder
│         └── Note
│
└── cUniverse (zero or more — optional federation links)
     └── Library  (libraries from the linked Universe — recursive)
          └── Folder
               └── Note
```

- **Universe** — the top-level container. One Universe is "active" per Constellation instance. The Universe directory carries `universe.json` (federation + meta manifest) and `.constellation/libraries.json` (libraries manifest).
- **Library** — a complete, self-contained knowledge base (equivalent to an Obsidian vault) and a direct child of a Universe. Has its own color, appearance, tags, links, index.
- **cUniverse (Child Universe)** — *optional*. A linked Universe whose libraries get federated into the parent at runtime. A Universe with zero cUniverses is a complete, valid setup; federation is opt-in.
- **Folder** — a subdirectory within a Library. Organizational only.
- **Note** — a single `.md` file with optional YAML frontmatter. The atomic unit.

Library ≠ Folder. A Library is a first-class citizen with its own identity. A Folder is just file organization inside a Library.

---

## 6. The Living Link Architecture

In Constellation, **a link is a first-class knowledge object**, not a pointer between two notes.

Each link is stored as a `.md` file on disk — `YYYYMMDDTHHMMSSZ_LINK_XXXX.md` — and indexed in a `note_links` SQLite table for fast lookup. The dual-layer storage (file on disk = source of truth; DB = index) lets the link survive any database rebuild.

A link carries **eight properties**:

1. **Type** — one of seven cognitive verbs: `supports`, `contradicts`, `causes`, `exemplifies`, `generalizes`, `derives-from`, `part-of`. These are the cognitive vocabulary of the system.
2. **Direction** — from-note → to-note (semantically meaningful for most types).
3. **Annotation** — a free-text note about *why* this link exists.
4. **Weight** — earned through use. Logarithmic growth on traversal, 5% monthly decay without use. A link the user keeps walking gets heavier; a link the user has forgotten fades.
5. **Confidence** — one of four levels: `hypothesis`, `evidence`, `established`, `contested`. Reflects the user's epistemic stance on the link itself.
6. **Created** — timestamp.
7. **Last Traversed** — timestamp.
8. **Traversal Count** — integer.

Links move through a **lifecycle**: Spark → Birth → Growth → Maturity → Dormancy → Renewal → Archival. The system never deletes; it archives. Every link operation is reversible.

The seven link types are not arbitrary. They are the canonical cognitive vocabulary the product is built on. `supports` and `contradicts` are the two halves of an argument; `causes` is the asymmetric explanatory link; `exemplifies` and `generalizes` are the inductive ladder; `derives-from` is the genealogical link; `part-of` is the mereological link. Together they cover the kinds of relationship a thinker actually traces between ideas.

---

## 7. The Surfaces (What the User Sees)

Constellation is built around a small number of carefully-considered surfaces, each answering a different question.

### NotePane — the standard editor
Full Markdown editing built on CodeMirror 6. Live preview, callouts, syntax highlighting, per-line bidirectional text, the full linked-thinking experience. This is where most writing happens.

### FocusPane — capture mode
**Plain text only.** No markdown parser, no syntax highlighting, no decorations, no toolbar. The same `.md` file under the hood, but rendered without any visual furniture. The constraint is the design: when the user wants to capture an idea fast, every UI element is a distraction.

### Constellation Sight — the diagnostic instrument
*Visualizes your entire knowledge universe as a stratified anchor dome with four coordinated mini-domes that re-encode the same notes through different channels.*

- **Anchor dome** — each note placed by **stratum** (depth of thought, 5 concentric rings) × **time** (12-month calendar edge)
- **Four mini-domes** rendering the same universe through one channel each:
  - **Confidence** → opacity (more confident notes are brighter)
  - **Stage** → color (cyan/orange/purple/green/yellow/grey for the seven lifecycle stages)
  - **Acts** → size (top decile of link-count = larger dots)
  - **Provenance** → 5 sectors (Self / Read / Heard / Reasoned / Tradition)
- **Facet sidebar** — six filter facets: Folder / Library / Stratum / Confidence / Stage / Provenance
- **Bidirectional linked brushing** — hover any star in any of the 5 surfaces, the same note lights up everywhere
- **Dome swap** — click any mini's empty area to promote it to the primary slot; the previous primary demotes into the vacated mini slot
- **Shift+click cross-filter** — Shift+click a star on Stage / Confidence / Provenance and the universe filters to that category across all 5 views
- **Ghost mode** — when a filter is active, non-matching stars stay visible at low opacity (15%) instead of vanishing; Shift+click a ghost to add its category to the filter (multi-select within a facet from the dome)
- **Extended view** (Cmd-Shift-D) — persistent mini-dome visibility across sessions

Sight answers: **"How is my epistemic content shaped and organized?"**

### Constellation Nervous System (CNS) — the connection-traversal view
*Sister surface to Sight. If Sight is the sensory form of your universe, CNS is the neural wiring.*

- **Universe Health card** — composite score + four metrics: Modularity (how cleanly notes cluster into communities), Dominance (whether one community dominates), Entropy (variety of community sizes), Connectivity (average links per note). Each metric carries a HEALTHY / CAUTION / IMBALANCED pill.
- **Gravity well** — force-directed graph; notes are nodes, typed links are edges, communities self-organize into clusters
- **Top Bridges** — notes that connect the most different communities ("synthesis points")
- **Communities** — detected note clusters
- **Blind Spots** — pairs of notes the graph algorithm thinks *should* be linked but aren't — an explorable hypothesis about structural gaps in your thinking
- **Single-click-preview / double-click-open** — single click shows a side panel with title, community, centrality rank, incoming/outgoing links *without opening the note*; double click opens it

CNS answers: **"How are the ideas in my universe connected, and where are the gaps?"**

### Constellation Map — sunburst visualization
A radial sunburst (D3) showing the Universe → Library → Folder → Note hierarchy as nested arcs. Useful for understanding the *shape* of one's organizational structure at a glance.

### Sky View — bubble graph of links
A force-directed bubble graph (PIXI) showing notes as nodes and links as edges. (Distinct from CNS: Sky View is a navigational view; CNS is an analytic view.)

### Index panel — term browser
A sidebar panel that reads directly from the SQLite FTS5 vocabulary dictionary. Click any term to see every note that mentions it. Built on write-time derivation — the FTS5 index is kept in sync with note content via triggers, so the term browser is always current without any rebuild step.

### Faceted search / SearchHub
Multi-faceted query interface combining text search, link-type filters, confidence filters, stage filters, and folder/library scoping.

---

## 8. Multilingual by Design

Constellation supports **15 languages** simultaneously from the ground up by design: Arabic (ar), German (de), English (en), Spanish (es), Persian (fa), French (fr), Hebrew (he), Hindi (hi), Japanese (ja), Korean (ko), Portuguese (pt), Russian (ru), Turkish (tr), Urdu (ur), Chinese (zh).

Multilingual support is not an afterthought; it is a core architectural feature:

- **Per-line bidirectional text** as a CM6 plugin (`bidiPlugin`). Mixed Arabic/Hebrew/English/Urdu in a single line renders correctly with no special markup required.
- **RTL-aware UI** throughout — chevrons flip, panels mirror, layout respects `dir` attributes.
- **Per-library and per-app font controls** including dedicated script fonts (so Arabic, Hebrew, CJK, Devanagari each get their own preferred typeface).
- **Help docs and User Manual** maintained in all 15 locales. Recent translations carry a `translation_status: AI-generated YYYY-MM-DD — native-speaker review recommended` frontmatter so downstream reviewers know the provenance.
- **Brand names kept English** (Constellation, Sight, CNS, Confidence, Stage) across all locales to maintain a single global product identity.

This is not lip service. The product owner is an Arabic-native speaker; Arabic is a first-class language, not a translated afterthought.

---

## 9. Local-First and File-Over-App

Two architectural commitments that govern everything:

**File over app.** Notes are `.md` files on disk. The user owns them. They open in any text editor. They survive Constellation being uninstalled. They sync via whatever the user chooses. The application is just a window. Constellation never modifies file content silently — every change is the result of an explicit user action.

**Local first.** All data stays on the user's device. No telemetry. No tracking. No cloud dependency. No account required. The app works fully offline, instantly, always. Sync is the user's choice (Git, Syncthing, iCloud, Dropbox), not Constellation's product.

These two commitments together explain a lot of architectural choices:

- The SQLite index is **ephemeral** — rebuilt from files when needed, updated incrementally at runtime. Files survive index loss.
- LINK files are stored on disk with deterministic filenames so they survive DB rebuilds.
- All settings are in `.constellation/` directories the user can inspect and edit.
- The "vault" terminology of Obsidian compatibility lives only at the import boundary; the canonical term throughout is **Library**.

---

## 10. Performance Philosophy

> *"Speed and reliability are often intuited hand-in-hand. Speed is a proxy for general engineering quality."* — Craig Mod
>
> *"If you want to create digital artifacts that last, they must be files you can control."* — Steph Ango (kepano)

The product takes performance seriously enough to make it a hard constraint:

- **Every keystroke must be instant.** Zero perceptible lag between typing and screen update. If the user notices delay, it's a bug.
- **No `$effect` loops** in Svelte 5. No effect that reads and writes the same reactive variable. Use `$derived` for computed values, `$effect` only for side effects.
- **Pre-cached CM6 decorations** at module load — never allocate `Decoration.mark()` inside a builder function.
- **Zero `invoke()` calls on the keystroke hot path.** Search queries debounced ≥300ms with previous-call cancellation. Saves debounced ≥1500ms.
- **Virtualize every list** that can exceed 50 items: file tree, search results, backlinks, tag browser, command palette.
- **Heavy work on the Rust side** via Tauri commands. Never parse 1000+ notes in JS.
- **Write-time derivation.** Every computed view (FTS5 index, term vocabulary, link counts) is maintained at write-time via triggers, not rebuilt at read-time. The app does not recompute on boot; it reads what's already stored.

The discipline pays off on large Universes (7,600+ notes verified in real use). The boot path, the typing path, and the IPC path are protected by rule, not by aspiration.

---

## 11. Current State (as of 2026-05-16)

### Shipped and stable

- Universe → Library → Folder → Note hierarchy (four levels)
- cUniverse federation layer (optional, opt-in)
- NotePane (full markdown editor, CM6, per-line bidi, live preview, callouts, syntax highlighting)
- FocusPane (plain-text capture, zero decorations)
- Constellation Sight v6.1 — Coordinated Views (anchor dome + 4 mini-domes + 6-facet sidebar + ghost mode + density mode + Extended view + dome-swap + Shift+click cross-filter + bidirectional linked brushing)
- Constellation Nervous System (CNS) — Universe Health + gravity well + Top Bridges + Communities + Blind Spots + single-click-preview / double-click-open
- Constellation Map (sunburst)
- Sky View (bubble graph)
- Index panel (FTS5 vocabulary-backed term browser)
- 15 i18n locales for the UI
- Help docs and User Manual in 15 locales (Sight + CNS topics translated 2026-05-16; older topics vintage)
- Living Link Architecture P0-P1 (Type, Direction, Annotation, Weight, Confidence stored both on disk as LINK files and in the SQLite index)
- Universal Epistemic Content Taxonomy as the scholarly spine for the Provenance dimension (5 branches × 11 sources, bilingual EN/AR, spans five civilizations)

### In active development (or queued next)

- **Phase 3 §C — Register chips.** Four production-polish registers each providing a different epistemic frame:
  - **Aristotelian** (default) — the four causes / syllogistic frame
  - **pramāṇa** — the Indian epistemology frame (perception, inference, testimony, etc.)
  - **masādir** — the classical Islamic sources-of-knowledge frame
  - **Polanyi** — tacit knowledge / personal knowledge frame
  Plus v1-preview register labeled but unfinished: Mohist sān-biǎo. *(Dignāga + Suhrawardi Ishrāqī were originally on this list; both permanently excluded 2026-05-16 — Dignāga by direct product decision, Ishrāqī under the religious-lineage rule that ships in orientation v2.09.)*
- **Living Link Architecture P2-P5** — lifecycle stage transitions in real use, weight decay/growth in real use, traversal-count display in panels, link search by all 8 properties.
- **Sight Settings UI section** — dedicated Settings tab for Sight preferences.
- **Confidence-population workflow** — UI for assigning confidence to existing links.
- **Sight v6.2 d3-hexbin polish** — true hex-binning for density mode at high note-counts.
- **Local LLM classifier for Six Sources** — the strategy is decided (local LLM, not rules, not cloud); the specific model + inference engine + bundling approach is open.
- **CECE Source Review queue** — 4,475 pending source-classification suggestions await user review; once approved, they populate the Provenance mini-dome with real data.
- **User Manual translation refresh** — Sections 8 / 8b / 10d in the 14 non-English Manuals still describe Sight v5. Separate cascade from the in-app help topic.

### Refused

- Cloud sync as a Constellation product
- AI as note author
- Recommendation engine
- Gamification
- Telemetry
- Vendor lock-in formats
- A "smart assistant" sidebar

---

## 12. Constraints That Cannot Be Violated

If a proposed feature breaks any of these, it does not ship — regardless of how attractive the feature is.

1. **File over app.** Notes are `.md` files on disk. The user owns them.
2. **Local first.** All data on the user's device. No telemetry. Works fully offline.
3. **Every keystroke instant.** No feature may regress typing latency on a 7,600-note Universe.
4. **Multilingual by design.** No layout, font, cursor, or input assumption may break for any of the 15 languages.
5. **Constraint as design.** FocusPane has no toolbar. Sight v6 has no register chip. These are not gaps; they ARE the design.
6. **Reversibility.** Archival, not deletion. Every operation can be undone.
7. **No proprietary formats.** Everything is standard Markdown + YAML frontmatter + SQLite.
8. **No silent file modification.** Every change to a `.md` file comes from an explicit user action.
9. **Write-time derivation.** No new feature may add a "scan on boot" or "rebuild on panel open" path. Derived views must be maintained via triggers on the source-of-truth write path.

---

## 13. How to Help Constellation Grow

If you (an outside AI) have been asked to do research for Constellation, here is the shape of question that helps.

### Good research questions

- *"What does the academic literature on Cross-Language Information Retrieval (CLIR) suggest about discovering links between notes written in different languages?"*
- *"How does Lucene's SynonymGraphFilter compare to SQLite FTS5's Method 2 for query-time concept expansion at 100K-document scale?"*
- *"What are the trade-offs between Llama-3.1-8B-Instruct, Phi-3-medium, and Qwen-2.5-7B for embedded local classification of source-type (the 11 Provenance categories) on consumer hardware?"*
- *"How do mature controlled-vocabulary systems (LCSH, MeSH, AAT) handle term-expansion at query time, and what does their experience suggest about the Constellation Index panel's expansion behavior?"*
- *"What does the IR literature say about community detection algorithms (Louvain vs Leiden vs SBM) for a heterogeneous typed-link graph at the 10K-50K-node scale?"*
- *"What is the prior art for lifecycle metaphors (spark → maturity → archival) in PKM or in adjacent fields like idea-management or creativity research?"*
- *"How do mature epistemology frameworks (Aristotelian causes, pramāṇa, masādir al-ma'rifa, Polanyi's tacit knowledge) compare on the question of how a single piece of knowledge moves from observation to conviction?"*
- *"What are the practical limits of CodeMirror 6 ViewPlugin decoration sets, and at what document size do common patterns degrade?"*

These questions share a shape: they ask about *prior art*, *trade-offs at scale*, or *what mature systems do*. They respect that Constellation is built on careful tradition-checking, not invention from scratch.

### Bad research questions

- *"Should Constellation add an AI assistant that writes notes for the user?"* — No. This violates the founding principle (formulation, not generation).
- *"Should Constellation store notes in a cloud database for sync?"* — No. Local-first is non-negotiable.
- *"Should Constellation add a recommendation engine for 'notes you might like'?"* — No. Constellation does not curate the user's attention.
- *"Should Constellation gamify writing with streaks and word-counts?"* — No. Constellation does not measure or motivate; it just records and connects.
- *"Should Constellation add usage telemetry to improve the product?"* — No. Telemetry violates the local-first commitment.
- *"Should Constellation auto-tag notes with ML?"* — Only if the tag is the user's; auto-tagging that the user did not approve is silent file modification.
- *"How can Constellation compete with Notion's database features?"* — Wrong frame. Constellation is not in Notion's category.

The bad questions are not bad because the underlying ideas are bad in the abstract. They are bad because they would convert Constellation into a different product. Research that helps Constellation grow is research that deepens its current commitments, not research that adds adjacent commitments.

---

## 14. Vocabulary Cheat-Sheet

For grounding terminology in any research output:

| Term | Meaning |
|---|---|
| **Universe** | top-level root directory; one is active per Constellation instance |
| **Library** | self-contained knowledge base, child of a Universe |
| **cUniverse** | optional federated child Universe |
| **Folder** | filesystem subdirectory inside a Library |
| **Note** | one `.md` file with optional YAML frontmatter |
| **Link types (7)** | supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of |
| **Confidence levels (4)** | hypothesis → evidence → established → contested |
| **Lifecycle stages (7)** | Spark → Birth → Growth → Maturity → Dormancy → Renewal → Archival |
| **Five Acts** | Observation → Connection → Tension → Synthesis → Conviction |
| **Stratum (6)** | Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing — depth-of-thought layers in Sight |
| **Provenance (5)** | Self / Read / Heard / Reasoned / Tradition — source-of-knowledge categories |
| **Sight** | the diagnostic visualization (anchor dome + 4 mini-domes) |
| **CNS** | Constellation Nervous System — connection-traversal view |
| **Map** | sunburst visualization of the hierarchy |
| **Sky View** | bubble graph of the link network |
| **NotePane / FocusPane** | the two editor modes |
| **Living Link** | a link as a first-class knowledge object stored as its own `.md` file |
| **Write-time derivation** | the architectural rule that derived views are maintained via triggers, not rebuilt at read-time |
| **Six Sources / CECE** | the Universal Epistemic Content Taxonomy classification pipeline for Provenance |

---

## 15. One-Paragraph Summary You Can Paste Anywhere

> Constellation is a local-first desktop Personal Knowledge Formulation system (Tauri + Rust + SvelteKit, SQLite FTS5 index, Markdown files on disk as source of truth) built for a single human doing the long, slow work of thinking. It supports 15 languages with per-line bidirectional text as a core feature. Notes live in a Universe → Library → Folder → Note hierarchy with optional federation across Universes. Links are first-class knowledge objects with type (7 cognitive verbs), confidence (4 levels), lifecycle stages (Spark → Birth → Growth → Maturity → Dormancy → Renewal → Archival), and earned weight. Two diagnostic surfaces — Sight (anchor-dome + 4 coordinated mini-domes) and CNS (connection-traversal graph + Universe Health) — let the user audit the shape of their own thinking. The product refuses cloud sync, AI-as-author, telemetry, gamification, and recommendation engines as a matter of principle. Speed is treated as engineering quality: every keystroke must be instant. The thesis is that knowledge is about connecting and synthesizing, not storing — and every feature is judged against that thesis.

---

*End of universal orientation. For deeper architectural detail (internal version history, MIG numbers, session logs), see the versioned `docs/Constellation Orientation & Onboarding vX.Y.md` series. For implementation rules and conventions, see `CLAUDE.md`. For Constellation's philosophical foundations, see `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`.*

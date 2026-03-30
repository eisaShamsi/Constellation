# Constellation — Cognitive Engine Roadmap
**"Constellation does not manage knowledge. It helps the user know."**

Architecturally derived from: `docs/constellation_cognitive_engine_v2.1.pdf`

---

## Vision

Two-layer architecture that transforms Constellation from a knowledge management tool into a knowledge cognition instrument:

- **Layer 1 — Structural Cognition**: 12 tools that work through data structure, graph topology, metadata, and visual cues. Zero AI dependency.
- **Layer 2 — AI Discovery**: 5 AI-powered capabilities that read Layer 1's structures to discover hidden patterns, blind spots, and cross-domain connections.

**Governing principle**: Complexity absorbed by the system, simplicity experienced by the user. Every tool must feel like thinking itself, not like operating software.

---

## Foundation Inventory (Already Built)

| Component | Status | Notes |
|---|---|---|
| GraphMind (Pixi.js force graph) | ✅ | 3 layouts, semantic links, cluster detection |
| Wikilinks + Backlinks | ✅ | Bidirectional, cross-library, rename-update |
| Frontmatter / YAML property system | ✅ | 7 property types, auto-type detection |
| Tags + TagsPanel | ✅ | Full scan, browse, filter |
| AI integration | ✅ | `ai_send_message`, embeddings, semantic engine |
| FocusPane (= Fleeting capture) | ✅ | Maps to Externalization Engine Stage 1 |
| Dataview / Bases query system | ✅ | `execute_dataview_query` |
| Search (full-text + property) | ✅ | Rust-side, fast |
| Tasks + Calendar | ✅ | `scan_library_tasks`, CalendarPanel |

---

## Build Sequence

### LAYER 1 — Structural Cognition

---

#### Phase 1: Typed Links  ⬅ CURRENT
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-01-typed-links.md`

The keystone feature. Every downstream tool depends on it.

**What it does**: Extends `[[wikilink]]` syntax to carry semantic meaning via a pipe character:
```
[[note|supports]]      — evidential relationship
[[note|contradicts]]   — tension (triggers Tension Detector)
[[note|causes]]        — causal (directional arrow in GraphMind)
[[note|exemplifies]]   — instance-of relationship
[[note|generalizes]]   — abstraction relationship
[[note|derives-from]]  — provenance (feeds Provenance Chain)
[[note|part-of]]       — compositional hierarchy
```
Untyped links default to `associative`. Power users type links; beginners never need to.

**Depends on**: Existing wikilink parser
**Unlocks**: Knowledge Strata, Tension Detector, Provenance Chain, GraphMind semantic rendering

---

#### Phase 2: Knowledge Strata
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-02-knowledge-strata.md`

Auto-classifies notes by abstraction level. No manual tagging.

**8-level hierarchy** (Datum → Worldview):
- Level 1–2: Datum / Information — raw facts, short notes, no links
- Level 3: Proposition — single claim, ≤1 source
- Level 4: Concept — links 3+ propositions, abstracts shared pattern
- Level 5: Principle — links 3+ concepts, states a general rule
- Level 6: Theory — Map of Content unifying principles
- Level 7: Paradigm — meta-framework spanning multiple theories
- Level 8: Worldview — foundational organizing structure

**Signals used** (pure Rust computation):
- Note word count
- Outgoing link count
- Incoming link count (from backlinks)
- Link types present (|causes, |generalizes raise stratum faster)
- Graph position (betweenness centrality proxy)

**Visual**: GraphMind nodes glow with increasing size + intensity as stratum rises.

**Depends on**: Phase 1 (Typed Links for richer signals)
**Unlocks**: Tension Detector (strata-aware), Review Pulse (priority by stratum)

---

#### Phase 3: Maturity Lifecycle
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-03-maturity-lifecycle.md`

Tracks note growth through 4 states. No manual tagging.

**States**:
- 🌱 **Seed** (بذرة): Newly captured, unlinked. Visual: faint dotted border.
- 🌿 **Sapling** (شتلة): Edited ≥1 time, 1–3 links. Visual: thin solid border, light green.
- 🌳 **Evergreen** (دائمة الخضرة): Multiple edits, 4+ links, referenced by others. Visual: full border, rich green.
- ⭐ **Canonical** (مرجعية): Referenced by 10+ notes, stable 30+ days. Visual: golden border, star.

**Decay**: Evergreen note untouched 90+ days while its tag-domain has active new notes → "wilting" state (subtle dimming).

**Signals used**: file modified-time, inbound link count, outbound link count (all from existing Rust commands)

**Depends on**: Wikilinks (existing), file metadata (existing)
**Unlocks**: Review Pulse (staleness detection), Tension Detector (orphan detection)

---

#### Phase 4: Tension Detector
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-04-tension-detector.md`

Surfaces contradictions and knowledge gaps. Zero AI. Presented as a gentle "knowledge health" panel.

**Detects**:
1. **Contradictions**: Notes linked with `|contradicts`
2. **Orphan knowledge**: Notes with zero inbound links
3. **Structural gaps**: Tag clusters with no cross-links between them
4. **Single points of failure**: Concepts referenced by many but supported by only one source

**Depends on**: Phase 1 (Typed Links), Phase 3 (Maturity — for orphan severity)
**Unlocks**: Layer 2 Tension analysis

---

#### Phase 5: Provenance Chain
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-05-provenance-chain.md`

Source lineage and trust depth. Inspired by Islamic isnad tradition.

**Mechanics**:
- Built from `|derives-from` typed links
- Any note can display its full ancestry back to primary sources
- **Received knowledge** (متلقّاة): chain traces to external source → cool color temperature in graph
- **Discovered knowledge** (مُكتشَفة): chain originates with user → warm glow in graph
- **Trust depth**: counts chain length (computational isnad — no content judgment)

**Depends on**: Phase 1 (|derives-from link type)
**Unlocks**: Layer 2 Blind Spot Detection (weak provenance signals)

---

#### Phase 6: Externalization Engine
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-06-externalization-engine.md`

Progressive formalization pipeline: fleeting → literature → permanent → synthesis.

**Stages** (stored as `stage:` frontmatter property):
1. **Fleeting** (عابرة): FocusPane = this stage already. Quick capture, no structure.
2. **Literature** (مرجعية): Processed from a source. `source:` property required.
3. **Permanent** (دائمة): Atomic idea, linked to graph. One idea per note.
4. **Synthesis** (تركيبية): Combines multiple permanent notes into new insight.

**UX**: One-click promotion between stages. Stage shown as subtle indicator in tab/file tree.
Not mandatory — just a scaffold for users who want it.

**Depends on**: Frontmatter system (existing), FocusPane (existing), Phase 3 (Maturity alignment)

---

#### Phase 7: Review Pulse
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-07-review-pulse.md`

Spaced resurfacing and staleness monitoring.

**Three modes**:
1. **Spaced Resurfacing**: Notes never revisited, queued at expanding intervals. Not flashcards — presents note and asks: "Still relevant? Link it? Archive it?"
2. **Staleness Scan**: Evergreen/canonical notes untouched while their domain grows. (Feeds from Phase 3 decay detection.)
3. **Mental Model Checkpoints**: Notes tagged `#assumption` or `#model` periodically surface with: "Do you still hold this view?"

**Depends on**: Phase 2 (Strata for priority), Phase 3 (Maturity/decay)

---

#### Phase 8: Trails
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-08-trails.md`

Named, ordered sequences of notes. First-class objects in the knowledge graph.

**Mechanics**:
- Trail = `.trail.md` file with ordered list of note paths
- Appears as path overlay in GraphMind
- Sequential navigation: previous / next within trail
- Playback mode: note-by-note presentation with branch-and-return
- Trails can feed Expression Forge as article backbone

**Depends on**: GraphMind (existing), Wikilinks (existing)

---

#### Phase 9: Multi-Lens Views
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-09-multi-lens-views.md`

Same content viewed through multiple independent classification schemes.

**Mechanics**:
- Each lens = named tag-hierarchy or metadata-query (extends existing Bases/Dataview)
- Switch lenses from sidebar toggle — no note duplication
- Multilingual lens: RTL + LTR concept pairs side by side

**Depends on**: Tags (existing), Dataview/Bases (existing)

---

#### Phase 10: Expression Forge
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-10-expression-forge.md`

Synthesis workspace for creating output from knowledge.

**Mechanics**:
- Assembles notes from graph proximity + user selection (pure topology, no AI)
- Side-by-side reading + writing
- Draft feeds Socratic Challenger (Layer 2) when AI is enabled
- Completion of the cycle: capture → cognition → expression

**Depends on**: Phase 2 (Strata for suggestions), Phase 8 (Trails as backbone), Phase 6 (Synthesis stage notes)

---

#### Phase 11: Sense-Making Canvas
**Status**: 🔲 Not started
**Spec**: `docs/phases/CE-phase-11-sensemaking-canvas.md`

Pre-structural space for ambiguous, half-formed, contradictory ideas.

**Mechanics**:
- Infinite spatial canvas (drag snippets, images, links, text fragments)
- Four optional Cynefin quadrants: Clear, Complicated, Complex, Chaotic
- One-click promotion: canvas item → proper note (carries canvas context as metadata)

**Note**: This is the most engineering-intensive feature (Excalidraw-level complexity). Scoped for late implementation.

**Depends on**: Frontmatter (existing), NotePane (existing)

---

### LAYER 2 — AI Discovery
*(Activates after Layer 1 establishes rich structural foundation)*

---

#### Phase 12: Hidden Pattern Discovery
Semantic content analysis beyond topology. Surfaces as "ghost links" (dashed translucent lines) in GraphMind.
**Depends on**: GraphMind semantic engine (partially exists), Phase 1–4

#### Phase 13: Blind Spot Detection
AI examines knowledge graph against external knowledge to identify domain gaps.
**Depends on**: Phase 2 (Strata), Phase 5 (Provenance), AI integration (existing)

#### Phase 14: Cross-Domain Insight Generation
AI reads Community Lenses and proposes cross-domain analogies.
**Depends on**: Phase 9 (Multi-Lens Views), AI integration (existing)

#### Phase 15: Socratic Challenger
When writing synthesis notes or in Expression Forge, AI asks challenging questions. Never provides answers.
**Depends on**: Phase 10 (Expression Forge), AI integration (existing)

#### Phase 16: Worldview Synthesis
AI reads full graph → generates "Worldview Map" of user's intellectual architecture.
**Depends on**: All Layer 1 phases complete, AI integration (existing)
**Policy decision needed**: Local LLM vs. selective context window vs. optional cloud

---

## UX Principles (apply to every phase)

1. **Zero-configuration start**: New user sees clean editor. No setup.
2. **Earned complexity**: Tools reveal as library grows. Strata after 20+ notes. Tension Detector after 50+ linked notes.
3. **Ambient indicators**: Maturity = border colors. Strata = node sizes. Staleness = dimming. Never pop-ups.
4. **Optional depth**: Typed links optional. Canvas quadrants optional. User chooses depth.
5. **One-action transitions**: Promote canvas item: one click. Type a link: one pipe character.
6. **RTL as native citizen**: Every tool, view, and label works multidirectionally.

---

## Build Rules (inherited from CLAUDE.md + Lessons Learned)

- **Each phase has a spec file** in `docs/phases/` before code is written
- **Phase passes user GO/NO-GO test** before next phase begins
- **No re-testing passed phases** in subsequent phases
- **Commit + tag after each passed phase**
- **Session log updated** after every phase (SO)
- **No feature that makes the app slower** — every keystroke must remain instant
- **No AI dependency in Layer 1** — all 12 tools work offline, always

---

## Progress Log

| Phase | Name | Status | Commit | Date |
|---|---|---|---|---|
| 1 | Typed Links | 🔲 Not started | — | — |
| 2 | Knowledge Strata | 🔲 Not started | — | — |
| 3 | Maturity Lifecycle | 🔲 Not started | — | — |
| 4 | Tension Detector | 🔲 Not started | — | — |
| 5 | Provenance Chain | 🔲 Not started | — | — |
| 6 | Externalization Engine | 🔲 Not started | — | — |
| 7 | Review Pulse | 🔲 Not started | — | — |
| 8 | Trails | 🔲 Not started | — | — |
| 9 | Multi-Lens Views | 🔲 Not started | — | — |
| 10 | Expression Forge | 🔲 Not started | — | — |
| 11 | Sense-Making Canvas | 🔲 Not started | — | — |
| 12 | Hidden Pattern Discovery | 🔲 Not started | — | — |
| 13 | Blind Spot Detection | 🔲 Not started | — | — |
| 14 | Cross-Domain Insights | 🔲 Not started | — | — |
| 15 | Socratic Challenger | 🔲 Not started | — | — |
| 16 | Worldview Synthesis | 🔲 Not started | — | — |

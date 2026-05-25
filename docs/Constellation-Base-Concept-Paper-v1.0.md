---
title: Constellation Base — Concept Paper
version: 1.0
date: 2026-05-25
status: Concept articulation. Guiding light for the design phase. No architecture commitments yet.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
predecessor: docs/BASES_MVP_SPEC.md (the MVP shipped 2026-03-12, commit c5b05f5c)
adjacent:
  - docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md (founding mission)
  - docs/Constellation-NSC-Concept-Paper-v2.0.md (the NSC service Bases consumes)
explicitly_out_of_scope:
  - Sight (disabled in core MIG-038, 2026-05-19; moved to External Plug-in / Constellation Wings)
  - Constellation Map (same status as Sight)
  - Constellation Mind / local-LLM stack (reverted MIG-046/047/048, 2026-05-25)
---

# Constellation Base — Concept Paper v1.0

## 1. Premise

A Constellation Base is a **living lens onto your epistemic content**, parameterized by question and shaped by the dimensions Constellation tracks that no other PKM tool tracks. It is not a database query, not a spreadsheet replacement, not a Notion-clone for markdown files. It is the surface through which a user asks their own collection — *"Show me this slice of my thinking, in this shape, right now"* — and gets an answer that is instant, formed of plain files, and richer than any other PKM can deliver.

Every PKM tool ships some version of this feature class because the market demands it (the nine user-stickiness effects documented in §3). The question is not whether Constellation has Bases. The question is **what makes a Constellation Base specifically Constellation's** — built *of* the architecture rather than added *to* it.

This paper answers that question by stating the principles that will govern the feature, the dimensions Constellation can leverage that competitors cannot, and the boundary between what a Base is and what it must never become.

---

## 2. The Question a Constellation Base Answers

> **"What is my collection telling me when I ask it this question?"**

The user brings a question — a frame, a slice, a curiosity. The Base brings the collection arranged by that question. The answer is:

- **Instant.** Whether you have 50 notes or 50,000.
- **Rich.** Drawing on Living Links, summaries, embeddings, classifications, and federation — not just YAML scalars.
- **Plain.** The view is a `.base` YAML file alongside your notes. The data lives in each note's frontmatter. Walk away and lose nothing.
- **Shaped.** The view is rendered in the form that answers the question — table, card, list, possibly a typed-link-graph subset, possibly federated across universes.

Constellation's question is not *"what does my data look like as a table?"* That is Obsidian's. Constellation's question is **cognitive** — what does my own collection look like when I frame it this way?

---

## 3. The Nine Effects (Eight Honored, One Refused)

From the field research, nine user-stickiness effects emerge from Bases-class features across the PKM/PKF industry. A Constellation Base will deliver eight; one we explicitly refuse.

### Honored

1. **The dashboard effect** — *"I see what's alive in my work right now."*
2. **The lens effect** — *"The same notes look different depending on what I'm asking."*
3. **The aggregation effect** — *"How many, how much, what's the distribution."*
4. **The edit-in-place effect** — *"I update the data without opening the note."*
5. **The externalized-self effect** — *"This is the shape of my own thinking."*
6. **The anxiety-reduction effect** — *"The system won't lose anything."*
7. **The project-page effect** — *"All the parts of this project live in one assemblage."*
8. **The assemblage effect** — *"One note holds many views of my collection, each answering a different question."*

These are non-negotiable. A Constellation Base that fails to deliver any of them has failed.

### Refused

9. **The structure-invitation effect** — *"I now have a beautiful schema that proves I'm organized."*

We refuse this explicitly. Bases must **make existing structure visible**, not invite the user to invent structure they have not earned through writing. The Zettelkasten tradition (Tietze's "Collector's Fallacy", Matuschak's "collecting feels more useful than it usually is", Doto's tensions essays) names this trap precisely: filing things into well-tagged buckets *feels* like knowledge work but is not. A Base view that exists primarily to give the user the satisfaction of looking at well-organized rows is productivity theater. Constellation will not ship template schemas designed to be aspirational. Every shipped Base view must answer a question the user actually has, drawn from properties they have actually populated.

This refusal is not abstract. It governs every decision about defaults, templates, and onboarding. When in doubt: **fewer Bases, deeper Bases**.

---

## 4. What a Constellation Base IS / IS NOT

### IS
- A **saved query + view** over the user's notes, persisted as a plain `.base` YAML file.
- A **knowledge lens** parameterized by question — table for inventory, card for browsing, list for sequence, plus future shapes the questions earn.
- A **first-class citizen** of the user's universe — embeddable in any host note, addressable in the sidebar, sortable by every dimension Constellation tracks.
- **Write-time derived** — the underlying index is maintained as notes are written, not rebuilt when the view opens.
- **Multilingual native** — Arabic column titles + English values, English titles + Persian values, all 15 languages, bidirectional, by default.
- **Federated** — when the user's universe has cUniverse children, a Base can span them.

### IS NOT
- A **spreadsheet replacement.** Bases is not for users who want Excel. Use Excel.
- A **task manager.** Bases can render task-like notes, but Constellation does not ship a task feature pretending to be Bases.
- A **CRM, recipe manager, habit tracker, or any other vertical application.** Constellation is not Notion-with-templates. We ship the surface; the user brings the question.
- A **structure machine.** Bases does not invite users to design schemas. It reveals the structure already present in their notes.
- A **Sight successor or visualization layer.** Sight is an External Plug-in (Constellation Wings) per MIG-038, 2026-05-19. Bases is not its replacement and does not absorb its responsibilities. If Sight returns, it returns as Wings; Bases does not contain it.

---

## 5. Founding Principles

Five principles, in priority order. When they conflict, higher-numbered yield to lower-numbered.

### 5.1 Form-Aligns-To-Purpose
Every column, filter, view shape, and rendering must carry cognitive meaning. If a view's geometry has degrees of freedom the question does not fill, change the primitive — don't fill the freedom with noise. A Base for "books I'm reading" needs progress + recency. A Base for "contradictions in my thinking" needs typed-link `contradicts` + confidence. Same notes, different shapes, because different questions.

This is the top principal of Constellation, restated here because it is the principle most violated by Bases-class features in the broader PKM market. Notion table views with twelve mostly-empty columns are the canonical violation.

### 5.2 Knowledge Formulation, Not Management
Bases must serve the Five Acts of Knowledge Creation: **Observation → Connection → Tension → Synthesis → Conviction**. A view that surfaces "notes that contradict notes I'm confident about" generates synthesis pressure. A view that lists "all my notes with `status: done`" is management. The first is the brand; the second is filler.

This is the difference between a Base view that produces thought and a Base view that organizes the products of thought.

### 5.3 Living Links as Queryable Dimensions
No other PKM treats links as typed entities with confidence, weight, traversal count, and lifecycle stage. Constellation does. **The Living Link Architecture is Bases' single biggest leverage point.** A Bases query that filters by `link.confidence > established` or sorts by `total_inbound_link_weight` is structurally impossible in Obsidian Bases, Notion databases, Tana search nodes, or Anytype sets. It is native here.

Bases is the surface that makes the Living Link Architecture operable for everyday queries.

### 5.4 Write-Time Derivation (CE Rule 8)
Every Bases query reads from derived state maintained at write time. No live filesystem scan, no on-demand frontmatter parse on a 10,000-note universe. The dashboard effect (Effect §1) dies if the query is slow; therefore the query must never be slow. This principle is architectural, not optional.

The current MVP violates this principle (`query_base` does a live scan). Reconciling Bases with Rule 8 is the largest architectural work this paper anticipates — but the principle precedes the architecture.

### 5.5 Language-First
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. Arabic frontmatter property names with English values, English property names with Persian values, mixed Hebrew + English in the same column header — all must render correctly without per-feature engineering. This is not an enhancement; it is the day-one design constraint.

---

## 6. The Constellation-Specific Leverage Points

Here is what is possible in a Constellation Base that is structurally impossible in any other PKM tool. These are the leverage points the architecture earns us.

### 6.1 Living Link Columns and Filters
- **Filter:** *"Notes with at least one outgoing `supports` link to a hypothesis-confidence note."*
- **Sort:** *"By total inbound link traversal weight, descending"* — notes ranked by the gravity of attention.
- **Filter:** *"Notes whose typed links are all marked `established` or higher"* — mature work.
- **Filter:** *"Notes whose links have entered Dormancy"* — neglected work.
- **Filter:** *"Notes that have been `supersedes`d by another note"* — and the link back to what superseded them.
- **Column:** *Link confidence distribution* (e.g., "3 hypothesis · 5 evidence · 12 established · 1 contested") — the epistemic state of the note in one cell.

These are queries no other PKM can pose, because no other PKM tracks these dimensions.

### 6.2 NSC Summary Headlines as a Default Column
The Note Summaries Cataloger (NSC, MIG-040 → MIG-045) already produces a one-sentence headline for every note in the user's universe, maintained at write time. **Every Constellation Base view shows headlines by default**, as a faint italic line under each row.

The user reads "what this note is about" before clicking. This is the single highest-leverage addition Bases can adopt, because the service is already live — Bases simply needs to consume it. No other PKM has machine-generated note headlines available to its database views.

### 6.3 Semantic Similarity as a Column Type
Embeddings exist (`embeddings.rs`, ONNX `multilingual-e5-small`, 384-dim, 100 languages). A Base column can render "similarity to this seed note" as a sortable number. A filter can express *"notes within 0.2 cosine distance of this seed note."* Semantic neighborhood becomes a first-class query criterion alongside string and numeric properties.

### 6.4 The Cataloger (CECE) Classifications as a Dimension
CECE — *The Cataloger* — classifies each note on two axes (content-type × source) via a 5-cataloger heuristic ensemble. These classifications can flow into Bases as filter and group-by axes — *"all my Observation-type notes from User-Authority sources"*, *"all my Argumentation-type notes from Semantic-cataloged origins"*. The Cataloger's output is Bases' epistemic columns.

### 6.5 Index Term Columns
The Index panel extracts terms from every note (`notes_vocab` FTS5 dictionary). A Bases column can display "top 3 terms in this note" inline. A filter can express *"notes containing term X via lemma Y"* — using the existing via-bridge machinery (MIG-010 → MIG-012).

### 6.6 Federation Across cUniverses
When a user federates their personal universe with a team universe (or any other cUniverse child), Bases queries can span the federation. *"All my book notes across my personal universe and the shared research universe."* `resolve_libraries_recursive` already flattens the federation tree; Bases rides it. **No mainstream PKM ships federated query across independent universes** — this is unexplored territory in the field.

### 6.7 Search-Hybrid Filtering
Constellation has FTS5, structured, semantic, and hybrid search modes already wired through SearchHub. A Base filter can be expressed as a search query — *"notes matching this query both textually and semantically"* — only possible in a tool with both engines.

### 6.8 Cloud AI (later phase) for NL → Query
The cloud AI bridge (`ai/mod.rs`, Anthropic / OpenAI / OpenRouter) is the only LLM surface in Constellation as of v2.34 (post-Mind-revert). A later phase can expose *"describe the view you want"* → the model produces the `.base` YAML. This is not v1, but the architecture admits it cleanly.

### 6.9 The Living Link Cell-Edit (aspirational)
The most ambitious leverage point: a Base cell can edit a *typed link* rather than just a string property. Mark a row as `supports` another row from a cell click. The Base becomes a relationship editor, not just a property editor. This is genuinely novel — no other PKM has it because no other PKM has typed-link semantics. **This is a later phase, not v1** — but the architecture admits it and the surface is worth claiming early.

---

## 7. The Host-Note / Assemblage Mode

The assemblage effect (Effect §8) is the strongest practical pattern in modern PKM. A single note becomes a workspace containing many views:

```
# Project: Aristotle's Ethics

## Active reading
```base
filter: tag=aristotle AND link.confidence in [hypothesis, evidence]
view: table
columns: [name, headline, confidence, updated]
```

## Productive contradictions
```base
filter: tag=aristotle AND link.type=contradicts
view: cards
```

## Open hypotheses
```base
filter: tag=aristotle AND link.confidence=hypothesis
view: list
```

## Notes
...the user's prose continues here...
```

Three Bases views inline in one host note, each answering a different question about the same collection. The host note is still a plain `.md` file — open it in any editor, you see prose plus three code blocks.

### What makes this Constellation's, not Notion's

- **The host note is sacred.** It remains a plain `.md` file. The Bases are inline `.base` code blocks. No proprietary container, no lock-in.
- **Each view leverages Constellation dimensions.** The Aristotle host note above embeds (a) a confidence-aware table from NSC + Living Links, (b) a contradictions card view from typed links, (c) an unresolved-hypotheses list. Each view is asking a *Constellation-specific cognitive question*.
- **Live, write-time-maintained.** Open the project page — all three views are instant. No spinners. The host note loads at the speed of any other note because the views read derived state.

### The "Knowledge Construction Page" pattern

The host-note assemblage is not just embedding views; it is a **named UX pattern** Constellation can teach: the *Knowledge Construction Page*. A user creates a host note for any project, area, or inquiry, and embeds the views that reveal what they need to see. The note holds the prose; the views hold the slices. The user constructs knowledge by composing prose-with-views, not by tagging-and-hoping.

This is Effect §8 (the assemblage effect) rendered through Constellation's vocabulary.

---

## 8. The Five Acts as Operational Templates

The Five Acts of Knowledge Creation are Constellation's cognitive model. Bases makes them operable. Each Act ships as a named template — not a schema for the user to fill, but a query Constellation runs over what the user has already written.

| Act | Bases Template | What it Surfaces |
|---|---|---|
| **Observation** | "Recent Captures" | Last 14 days, sorted by creation date, NSC headlines visible, no link metadata. The intake queue. |
| **Connection** | "Single-Direction Conduits" | Notes with high outbound link counts but low inbound — work that points outward but has not yet been pointed *to*. Awaiting reciprocation. |
| **Tension** | "Productive Frictions" | Notes connected by `contradicts`-typed links. Pairs surfaced together. The forge of synthesis. |
| **Synthesis** | "Convergence Points" | Notes with high inbound link weight from multiple sources of `established` confidence. Where threads have joined. |
| **Conviction** | "Load-Bearing Work" | Notes with `confidence: established` AND traversal count > N (configurable). The pieces of the universe that, by the system's lights, are doing the most work. |

These are the **only template Bases Constellation ships by default**. They are not schemas users build; they are queries Constellation runs. Each one teaches the user what a Constellation Base is for by demonstrating it against their own notes.

This is the antidote to the structure-invitation effect (§3): users learn Bases not by being given empty schemas to populate, but by seeing the system reveal patterns in work they have already done.

---

## 9. What Sets a Constellation Base Apart

Three lines of differentiation, in honest priority order:

1. **Living Links as query dimensions.** Filter by confidence, sort by weight, group by typed link, surface contradictions automatically. Structurally impossible in any other PKM.
2. **Summary headlines visible by default.** Every row shows what the note *is about* before the user clicks. NSC is the differentiator that makes the dashboard effect (§1) work — you scan the headlines, not just the names.
3. **Federation across universes.** Query spans cUniverses. The team and the individual share one view. The long-tail PKM dream nobody has shipped.

A Constellation Base that does not deliver these three is not a Constellation Base — it is Obsidian Bases running on a different runtime. The point of building Bases *of* Constellation rather than *into* Constellation is that the architecture *gives* us these three. Refusing to use them is the violation.

---

## 10. Architectural Mandates

Five mandates, derived from the principles. The architecture phase will refine; these set the boundary.

### 10.1 Write-Time Derivation
A `bases_cache` SQLite table (or equivalent) is maintained by triggers on `note_meta` writes (and `note_links` writes for link-aware queries). `query_base` becomes a cheap SQL lookup. No live filesystem scan in the keystroke-to-screen path.

### 10.2 File-Over-App
`.base` files remain plain YAML on disk. The cache is internal optimization; the source of truth is the file. Delete the cache → it rebuilds. Delete the `.base` file → the view is gone, every note's frontmatter is intact.

### 10.3 Instant on 10k
Every Base query, including federated and Living-Link-filtered, must return in under 50ms on a 7,600-note universe. This is the gate for the dashboard effect.

### 10.4 Multilingual Native
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. No "Phase 2 RTL" deferral. Mixed-script cells render correctly without per-feature engineering.

### 10.5 Embedded Bases Are First-Class
An inline `.base` block in a host note has the same capability surface as a workspace-level `.base`. The host-note assemblage mode is not a stripped-down sibling.

---

## 11. Out of Scope (v1)

These are excluded by design from the first delivery, not because they're bad but because they would dilute the principles.

- **Pre-built vertical templates** (CRM, recipe manager, habit tracker, gym log, book tracker, contact manager, etc.). The Five Acts templates ship; no others. (Refused per §3.)
- **Aggregation formulas** (sum, average, count) beyond what's needed for the Five Acts. Later phase.
- **Calendar / timeline / board / gallery views.** v1 ships table + card + list — the three the question shapes already earn. Other shapes ship when a Constellation-specific question earns them.
- **Cloud AI NL → query.** The architecture admits it; v1 doesn't ship it.
- **Generative lens suggestions** (the system proposing Bases based on usage). Research mode, not v1.
- **Real-time multi-user collaboration in a Base.** Constellation is local-first; collaboration is the user's choice via Git, Syncthing, iCloud.
- **Bases-from-external-data** (Notion's "Connections" / Coda's "Packs" pattern). Not aligned with file-over-app.

---

## 12. Roadmap (Provisional)

Sequencing, not commitments. Each phase is itself a separate `/migration` workflow (Architect → Plan → Build → Audit → PCS).

- **Phase 0 — Concept** (this paper) ✓
- **Phase 1 — Rule 8 Migration.** `bases_cache` table, triggers, `query_base` cheap lookup. The architectural foundation. No new user-visible features beyond instant performance.
- **Phase 2 — Living Link Columns.** Extend `.base` schema to express link-confidence, link-type, link-weight as columns/filters/sorts. The leverage points (§6.1) become user-facing.
- **Phase 3 — NSC Headlines as Default Column.** Headlines visible in every Bases view by default. Hover for full summary.
- **Phase 4 — Federation.** `.base` queries can span cUniverses.
- **Phase 5 — Five Acts Templates.** The named templates from §8 ship as built-in Bases.
- **Phase 6 — Host-Note Assemblage Mode.** Inline `.base` blocks in any host note, first-class capability surface.
- **Phase 7+** — Semantic columns, Cataloger classifications, Index terms, search-hybrid filters, cell-edit on typed links, NL → query.

Each phase is independently shippable and reversible. None of them precludes a future Bases-as-Wings-aware integration if that becomes the right shape.

---

## 13. Open Questions

These need direction before architecture phase begins.

1. **Headline-as-default-column.** Opt-in, opt-out, or unconditional? *Recommendation: unconditional.* Eisa decides.
2. **`.base` schema extension for Living Links.** YAML key names — `filter.link.type`, `filter.link.confidence`, etc. Bikeshed worth doing once.
3. **Five Acts templates.** Ship as user-editable copies, ship as read-only system Bases, or ship as both? *Recommendation: read-only by default; "Customize" duplicates to a user-editable copy.* Eisa decides.
4. **Host-note assemblage mode acceleration.** Ship in Phase 6 as planned, or accelerate to Phase 1.5 because it's the strongest user-facing pattern? Eisa decides.
5. **Federation default behavior.** Does a Bases view automatically federate to cUniverses if the user has them, or is federation an explicit opt-in per Base? *Recommendation: explicit opt-in (`selected_vaults` already has the right shape).* Eisa decides.
6. **360.3D.** With Sight disabled in core per MIG-038, is the per-note 360.3D still active? If yes, is there any relationship between Bases and 360.3D worth designing in? If no, this paper need not mention it again. Honest gap in my knowledge — needs Eisa's confirmation.
7. **Constellation Wings integration.** When the External Plug-in subsystem ships, should Bases expose data to External Plug-ins via a stable IPC contract? Likely yes, but the contract is Wings' to define, not Bases'. Mention only.
8. **Cell-edit on typed links (§6.9).** v1, Phase 2, or aspirational? *Recommendation: Phase 7 — a workflow innovation worth design care, not the first ship.* Eisa decides.

---

## 14. Predecessor and Adjacent Documents

- **Predecessor:** `docs/BASES_MVP_SPEC.md` — the shipped MVP design (commit `c5b05f5c`, 2026-03-12). Source of the existing `.base` YAML format, the Tauri command surface, the table/card/list views, the cell-edit-in-place flow. **This Concept Paper does not invalidate the MVP; it articulates the destination.**
- **Successor:** future Architect docs per `/migration` discipline, one per Phase above (MIG-NNN allocation at architecture time).
- **Adjacent — founding mission:** `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` (Living Link Architecture and the Five Acts).
- **Adjacent — service consumed:** `docs/Constellation-NSC-Concept-Paper-v2.0.md` (NSC as Core Plug-in; Bases is a downstream consumer).
- **Adjacent — current state record:** `docs/Constellation Orientation & Onboarding v2.34.md` §4582 (current Bases subsystem record).
- **Explicitly NOT in scope:** the disabled Sight subsystem (MIG-038, 2026-05-19, moved to Constellation Wings as an External Plug-in) and the reverted Constellation Mind subsystem (MIG-046/047/048, reverted 2026-05-25). Bases is not their replacement and does not absorb their responsibilities.

---

## 15. Closing — The Guiding Light

When in doubt during the design phase, return to one question:

> **Does this make the user re-encounter their own thinking — or invite them to re-organize it?**

The first is knowledge formulation. The second is productivity theater. Constellation ships the first.

This paper exists to keep that distinction visible at every step from here to release.

---

*End of Concept Paper v1.0. To be updated only on substantive change of vision; tactical updates belong in Architect docs of subsequent phases.*

---
title: Constellation Base — Concept Paper
version: 1.1
date: 2026-05-25 (same day as v1.0, post-decisions)
status: 7 of 8 open questions from v1.0 §13 locked. One specific clarification pending — the Bases ↔ 360.3D relationship.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
supersedes: v1.0 (preserved at docs/Constellation-Base-Concept-Paper-v1.0.md as historical record)
predecessor_design: docs/BASES_MVP_SPEC.md (the MVP shipped 2026-03-12, commit c5b05f5c)
adjacent:
  - docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md (founding mission)
  - docs/Constellation-NSC-Concept-Paper-v2.0.md (the NSC service Bases consumes)
explicitly_out_of_scope:
  - Sight (disabled in core MIG-038, 2026-05-19; moved to External Plug-in / Constellation Wings)
  - Constellation Map (same status as Sight)
  - Constellation Mind / local-LLM stack (reverted MIG-046/047/048, 2026-05-25)
---

# Constellation Base — Concept Paper v1.1

> **What changed in v1.1** — Eisa locked 7 of the 8 open questions from v1.0 §13 in the same session (2026-05-25). The paper folds those decisions in:
>
> - **Headlines unconditional, render per view-context** (§6.2 expanded).
> - **All Living Link dimensions are queryable** — 8 properties + lifecycle stages + supersedes chains (§6.1 expanded).
> - **Five Acts templates ship as both read-only system Bases AND editable user copies on duplicate** (§8 expanded).
> - **Host-Note Assemblage Mode accelerated to Phase 1.5** — ships right after the Rule 8 migration, before Living Link columns (§7 substantially expanded, §12 roadmap renumbered).
> - **Federation defaults to auto** — `selected_vaults` becomes opt-OUT rather than opt-in (§6.6 expanded, §10.6 added).
> - **Constellation Wings integration: bidirectional contract** — Bases exposes data to Wings AND Bases can consume from Wings registrations (§10.7 added).
> - **Cell-edit on typed links: locked to Phase 7** — last on the roadmap, by design (§6.9 noted).
> - **360.3D relationship: alive, exists** — one specific clarification still pending before architecture phase begins (§13).

---

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
- **Federated by default** — when the user's universe has cUniverse children, a Base spans them automatically (auto-federation locked in v1.1; see §6.6, §10.6).

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

### 6.1 Living Link Columns and Filters (full surface, locked v1.1)

Per Eisa's lock on §13 #2 (v1.1, 2026-05-25), **all** Living Link dimensions are exposed as queryable columns, filters, and sorts. The full surface:

**The eight link properties** (from `note_links` and the LINK file kind):
- `link.type` — the typed vocabulary: `supports`, `contradicts`, `causes`, `exemplifies`, `generalizes`, `derives_from`, `part_of`, `supersedes`, plus the default `associative`. Filter by type, group by type, count by type.
- `link.direction` — incoming, outgoing, or symmetric. The directionality dimension.
- `link.annotation` — the user's note on the link itself (a string). Searchable.
- `link.weight` — numeric, earned through use (logarithmic growth on traversal, 5% monthly decay without use). Sortable and rangeable.
- `link.confidence` — the four-level enum: `hypothesis` → `evidence` → `established` → `contested`. Filter, sort, group.
- `link.created` — timestamp.
- `link.last_traversed` — timestamp. Powers the dormancy detector.
- `link.traversal_count` — integer. The literal count of how often the user has walked this link.

**Plus the lifecycle stage** — derived from the above per the Living Link spec:
- `link.lifecycle` — `Spark` | `Birth` | `Growth` | `Maturity` | `Dormancy` | `Renewal` | `Archival`. The single most expressive query axis.

**Plus aggregated dimensions** at the note level:
- `note.inbound_link_count_by_type` — *e.g., 3 supports, 1 contradicts, 5 associative*.
- `note.outbound_link_count_by_type` — same shape, outbound.
- `note.total_inbound_link_weight` — sum across all inbound links.
- `note.total_outbound_link_weight` — sum across all outbound links.
- `note.inbound_confidence_distribution` — *e.g., "3 hypothesis · 5 evidence · 12 established · 1 contested"*.
- `note.dormant_link_count` — links that have entered Dormancy.

**Plus relational queries:**
- `note.is_supersedes_of(other)` — *this note supersedes another*.
- `note.is_superseded_by(other)` — *this note has been superseded by another*.
- `note.contradicts_any_established` — *this note contradicts at least one note of `established` confidence*. A native Tension-Act filter.

**Example queries the full surface enables:**
- *"Notes whose outbound links are mostly `supports` to `established`-confidence targets"* — load-bearing scholarly work.
- *"Notes whose inbound links have entered Dormancy"* — neglected work.
- *"Notes superseded by another note that is itself superseded"* — chains of revision, deep epistemic history.
- *"Notes with > 5 `contradicts` links from `established`-confidence sources"* — claims under sustained tension.
- *"Notes with the highest total traversal weight in the last 30 days"* — what the user is actually returning to.

These are queries no other PKM can pose because no other PKM tracks these dimensions. **Bases is the surface that makes the Living Link Architecture operable for everyday queries.**

### 6.2 NSC Summary Headlines — Unconditional Default (locked v1.1)

Per Eisa's lock on §13 #1 (v1.1, 2026-05-25), **every Constellation Base view shows the NSC headline unconditionally** as a default column. The headline is the one-sentence "what this note is about" already produced by the Note Summaries Cataloger (NSC, MIG-040 → MIG-045) and maintained at write time.

**Context-aware rendering** — the rendering style depends on the view shape:
- **Table view:** the headline renders as a faint italic line directly under the note name, in a slightly smaller font.
- **Card view:** the headline is the main visible body of the card, prominent.
- **List view:** the headline is inline after the note name, separated by an em-dash.
- **Future view shapes:** each must define its headline rendering as part of the view spec.

The headline content itself is the same across contexts (NSC produces one headline per note); only the rendering varies. The user always knows "what this note is about" before clicking.

This is the single highest-leverage addition Bases can adopt because the service is already live — Bases simply needs to consume it. No other PKM has machine-generated note headlines available to its database views.

### 6.3 Semantic Similarity as a Column Type

Embeddings exist (`embeddings.rs`, ONNX `multilingual-e5-small`, 384-dim, 100 languages). A Base column can render "similarity to this seed note" as a sortable number. A filter can express *"notes within 0.2 cosine distance of this seed note."* Semantic neighborhood becomes a first-class query criterion alongside string and numeric properties.

### 6.4 The Cataloger (CECE) Classifications as a Dimension

CECE — *The Cataloger* — classifies each note on two axes (content-type × source) via a 5-cataloger heuristic ensemble. These classifications flow into Bases as filter and group-by axes — *"all my Observation-type notes from User-Authority sources"*, *"all my Argumentation-type notes from Semantic-cataloged origins"*. The Cataloger's output is Bases' epistemic columns.

### 6.5 Index Term Columns

The Index panel extracts terms from every note (`notes_vocab` FTS5 dictionary). A Bases column can display "top 3 terms in this note" inline. A filter can express *"notes containing term X via lemma Y"* — using the existing via-bridge machinery (MIG-010 → MIG-012).

### 6.6 Federation Across cUniverses — Auto by Default (locked v1.1)

Per Eisa's lock on §13 #5 (v1.1, 2026-05-25), **federation is automatic by default**. When the user's universe has cUniverse children, every Base query spans them unless explicitly scoped.

- **The default behavior:** a Bases view returns rows from the current universe AND all federated cUniverses, transparently. The user does not need to opt in.
- **`selected_vaults` becomes the opt-OUT mechanism.** The existing `BaseSource.selected_vaults` filter (already in the v1 schema, see `bases.rs` and `types.ts`) is the channel users use to constrain a Base to a specific library or subset of universes when they want to.
- **UI signaling:** federated rows are visually marked (small icon or color tint) so the user can see at a glance which universe a row came from. The exact UI affordance is an architecture-phase detail; the principle is *visible federation*.
- **`resolve_libraries_recursive` already flattens the federation tree** in `universe.rs`. Bases rides it for free.

**No mainstream PKM ships federated query across independent universes by default** — this is unexplored territory in the field, and Constellation makes it the *default* rather than an advanced feature.

### 6.7 Search-Hybrid Filtering

Constellation has FTS5, structured, semantic, and hybrid search modes already wired through SearchHub. A Base filter can be expressed as a search query — *"notes matching this query both textually and semantically"* — only possible in a tool with both engines.

### 6.8 Cloud AI (later phase) for NL → Query

The cloud AI bridge (`ai/mod.rs`, Anthropic / OpenAI / OpenRouter) is the only LLM surface in Constellation as of v2.34 (post-Mind-revert). A later phase can expose *"describe the view you want"* → the model produces the `.base` YAML. This is not v1, but the architecture admits it cleanly.

### 6.9 The Living Link Cell-Edit (Phase 7, locked v1.1)

Per Eisa's lock on §13 #8 (v1.1, 2026-05-25), this leverage point is **deferred to Phase 7** — the last phase on the roadmap. A Base cell could edit a *typed link* rather than just a string property: mark a row as `supports` another row from a cell click, turning the Base into a relationship editor. This is genuinely novel — no other PKM has it because no other PKM has typed-link semantics. It is worth claiming as a destination in this paper, but it does not ship early and does not appear in v1 / Phase 1.5 / Phase 2 / Phase 3 / etc.

---

## 7. The Host-Note / Assemblage Mode (Phase 1.5, locked v1.1)

Per Eisa's lock on §13 #4 (v1.1, 2026-05-25), the **host-note assemblage mode is accelerated to Phase 1.5** — it ships immediately after the Rule 8 migration (Phase 1) and before Living Link columns (Phase 2). This elevates assemblage from "the eventual sixth feature" to "the second user-facing feature." It is the strongest practical pattern in modern PKM and Constellation reaches for it early.

### What Phase 1.5 ships

A single note becomes a workspace containing many views. Inline ` ```base ` code blocks in any host note render as full Bases views with the **same capability surface as workspace-level `.base` files**. The host note remains a plain `.md` file — open it in any editor, you see prose plus YAML code blocks.

```markdown
# Project: Aristotle's Ethics

## Active reading
\`\`\`base
filter: tag=aristotle AND status=in-progress
view: table
columns: [name, headline, status, updated]
\`\`\`

## Recent captures
\`\`\`base
filter: tag=aristotle AND created>30d-ago
view: cards
\`\`\`

## Notes
...the user's prose continues here...
```

Two Bases views inline in one host note, each answering a different question about the same collection. Each view is instant because the data they read is the same write-time-derived state Phase 1 establishes.

**At Phase 1.5 the inline filter surface is the v1 set** — `is`, `is_not`, `contains`, `gt`, `lt`, `is_empty`, `is_not_empty`, plus the unconditional NSC headline column. Living Link filters arrive in Phase 2, which is when host-note assemblage really comes alive. The Phase 1.5 example above intentionally uses only v1-era filters.

The fully-realized end-state, by way of preview, is what the paper aspires to reach by Phase 3:

```markdown
## Productive contradictions
\`\`\`base
filter: tag=aristotle AND link.type=contradicts
view: cards
\`\`\`

## Mature, load-bearing claims
\`\`\`base
filter: tag=aristotle AND link.confidence=established AND link.traversal_count > 5
view: list
\`\`\`
```

Each view above is asking a *Constellation-specific cognitive question*. The host note holds the prose; the views hold the slices.

### What makes this Constellation's, not Notion's

- **The host note is sacred.** It remains a plain `.md` file. The Bases are inline `.base` code blocks. No proprietary container, no lock-in. The user can take the file to any other markdown editor and lose only the rendered views — the prose, the frontmatter, and the embedded YAML are intact.
- **Each view leverages Constellation dimensions** (once Phase 2+ ships the leverage points). The Aristotle host note above embeds (a) a confidence-aware table from NSC + Living Links, (b) a contradictions card view from typed links, (c) a load-bearing-claims list. Each view is asking a question only Constellation can answer.
- **Live, write-time-maintained.** Open the project page — all views are instant. No spinners. The host note loads at the speed of any other note because the views read derived state.

### The "Knowledge Construction Page" pattern

The host-note assemblage is not just embedding views; it is a **named UX pattern** Constellation can teach: the *Knowledge Construction Page*. A user creates a host note for any project, area, or inquiry, and embeds the views that reveal what they need to see. The note holds the prose; the views hold the slices. The user constructs knowledge by composing prose-with-views, not by tagging-and-hoping.

This is Effect §8 (the assemblage effect) rendered through Constellation's vocabulary. With Phase 1.5 acceleration, this is the second pattern users will encounter — right after they notice that their existing Bases now respond instantly.

### Why accelerate to Phase 1.5

Three reasons that argued for the acceleration:
1. **It's the dominant user-facing pattern across the entire PKM market.** Notion is built on it; Obsidian Bases supports it via inline `.base` blocks; Tana, Capacities, Anytype all converge on it. Shipping it late is shipping the table stakes late.
2. **It composes with the Rule 8 migration.** Phase 1 establishes the cheap-lookup index; Phase 1.5 simply uses that index from inside any note. No new indexing work — only the renderer and the YAML-block extractor.
3. **It teaches the system's affordances by example.** A user who opens a host note with three embedded views immediately understands what a Bases view is for. A user who only sees the sidebar Bases list has a worse onboarding.

---

## 8. The Five Acts as Operational Templates (locked v1.1)

The Five Acts of Knowledge Creation are Constellation's cognitive model. Bases makes them operable. Each Act ships as a named template — not a schema for the user to fill, but a query Constellation runs over what the user has already written.

| Act | Bases Template | What it Surfaces |
|---|---|---|
| **Observation** | "Recent Captures" | Last 14 days, sorted by creation date, NSC headlines visible, no link metadata. The intake queue. |
| **Connection** | "Single-Direction Conduits" | Notes with high outbound link counts but low inbound — work that points outward but has not yet been pointed *to*. Awaiting reciprocation. |
| **Tension** | "Productive Frictions" | Notes connected by `contradicts`-typed links. Pairs surfaced together. The forge of synthesis. |
| **Synthesis** | "Convergence Points" | Notes with high inbound link weight from multiple sources of `established` confidence. Where threads have joined. |
| **Conviction** | "Load-Bearing Work" | Notes with `confidence: established` AND traversal count > N (configurable). The pieces of the universe that, by the system's lights, are doing the most work. |

### Distribution model — both shapes (locked v1.1)

Per Eisa's lock on §13 #3 (v1.1, 2026-05-25), the Five Acts templates ship in **both shapes**:

1. **Read-only system Bases.** The five templates are installed in every universe as system-owned Bases at a fixed path (e.g., `{universe}/.constellation/bases/system/five-acts/`). They are not user-editable in place. They update automatically when Constellation ships a refinement to a template.
2. **Editable user copies on duplicate.** A "Customize" gesture (button on the view, or `Save As…`) duplicates the system Base into the user's workspace-bases area as a fully independent, fully editable `.base` file. The duplicate retains a `derivedFrom: system/five-acts/{act-name}` marker so the lineage is visible, but the copy is otherwise free to diverge.

This protects the canonical Five Acts representations as Constellation's *teaching artifacts* — users encountering them are encountering the system's articulation of what the Acts mean — while preserving the user's right to customize, fork, and re-shape.

These are the **only template Bases Constellation ships by default**. They are not schemas users build; they are queries Constellation runs. Each one teaches the user what a Constellation Base is for by demonstrating it against their own notes.

This is the antidote to the structure-invitation effect (§3): users learn Bases not by being given empty schemas to populate, but by seeing the system reveal patterns in work they have already done.

---

## 9. What Sets a Constellation Base Apart

Three lines of differentiation, in honest priority order:

1. **Living Links as query dimensions.** Filter by confidence, sort by weight, group by typed link, surface contradictions automatically. Structurally impossible in any other PKM.
2. **Summary headlines visible by default, context-aware rendering.** Every row shows what the note *is about* before the user clicks. NSC is the differentiator that makes the dashboard effect (§1) work — you scan the headlines, not just the names.
3. **Federation across universes — auto by default.** Query spans cUniverses without the user opting in. The team and the individual share one view. The long-tail PKM dream nobody has shipped, made the *default*.

A Constellation Base that does not deliver these three is not a Constellation Base — it is Obsidian Bases running on a different runtime. The point of building Bases *of* Constellation rather than *into* Constellation is that the architecture *gives* us these three. Refusing to use them is the violation.

---

## 10. Architectural Mandates

Seven mandates, derived from the principles. The architecture phase will refine; these set the boundary.

### 10.1 Write-Time Derivation
A `bases_cache` SQLite table (or equivalent) is maintained by triggers on `note_meta` writes (and `note_links` writes for link-aware queries). `query_base` becomes a cheap SQL lookup. No live filesystem scan in the keystroke-to-screen path.

### 10.2 File-Over-App
`.base` files remain plain YAML on disk. The cache is internal optimization; the source of truth is the file. Delete the cache → it rebuilds. Delete the `.base` file → the view is gone, every note's frontmatter is intact.

### 10.3 Instant on 10k
Every Base query, including federated and Living-Link-filtered, must return in under 50ms on a 7,600-note universe. This is the gate for the dashboard effect.

### 10.4 Multilingual Native
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. No "Phase 2 RTL" deferral. Mixed-script cells render correctly without per-feature engineering.

### 10.5 Embedded Bases Are First-Class
An inline ` ```base ` block in a host note has the same capability surface as a workspace-level `.base`. The host-note assemblage mode (Phase 1.5) is not a stripped-down sibling.

### 10.6 Federation Is Default-On (added v1.1)
Bases queries auto-span cUniverses. The user does not opt in. Visible UI affordance distinguishes federated rows from local rows so the federation is never invisible — only frictionless. `selected_vaults` is the explicit opt-out channel.

### 10.7 Constellation Wings Integration Is Bidirectional (added v1.1, per §13 #7 lock)
When the External Plug-in subsystem (Wings) ships, the Bases ↔ Wings contract is **bidirectional**:
- **Bases exposes data to Wings.** External Plug-ins can query Bases programmatically via a stable IPC contract: list available Bases, parse a `.base` definition, execute a query, observe data-changed events. This lets Wings plugins build their own surfaces on top of Bases (alternative renderers, AI assistants, sync agents, etc.).
- **Bases consumes from Wings.** External Plug-ins can register their own data sources or column types into Bases via the same IPC. A weather plugin could register a `weather.temperature` column; a calendar plugin could register a `calendar.next_event` column. Bases treats these registered sources as first-class extensions of its column vocabulary.

The exact IPC shape is Wings' to specify (when Wings ships) — this paper records that the contract MUST be bidirectional, not that Wings has been built yet.

---

## 11. Out of Scope (v1)

These are excluded by design from the first delivery, not because they're bad but because they would dilute the principles.

- **Pre-built vertical templates** (CRM, recipe manager, habit tracker, gym log, book tracker, contact manager, etc.). The Five Acts templates ship; no others. (Refused per §3.)
- **Aggregation formulas** (sum, average, count) beyond what's needed for the Five Acts. Later phase.
- **Calendar / timeline / board / gallery views.** v1 ships table + card + list — the three the question shapes already earn. Other shapes ship when a Constellation-specific question earns them.
- **Cloud AI NL → query.** The architecture admits it; v1 doesn't ship it.
- **Generative lens suggestions** (the system proposing Bases based on usage). Research mode, not v1.
- **Real-time multi-user collaboration in a Base.** Constellation is local-first; collaboration is the user's choice via Git, Syncthing, iCloud.
- **Bases-from-external-data** (Notion's "Connections" / Coda's "Packs" pattern). Not aligned with file-over-app *until* Wings ships, at which point external-data integrations are Wings' responsibility per §10.7.

---

## 12. Roadmap (Provisional — Updated v1.1)

Sequencing, not commitments. Each phase is itself a separate `/migration` workflow (Architect → Plan → Build → Audit → PCS).

- **Phase 0 — Concept** (this paper) ✓
- **Phase 1 — Rule 8 Migration.** `bases_cache` table, triggers, `query_base` cheap lookup. The architectural foundation. No new user-visible features beyond instant performance.
- **Phase 1.5 — Host-Note Assemblage Mode.** (Accelerated per Eisa lock §13 #4.) Inline ` ```base ` code blocks render as full Bases views in any host note. Same capability surface as workspace-level `.base` files. Uses Phase 1's cheap-lookup index; uses Phase 1 filter set.
- **Phase 2 — Living Link Columns.** Extend `.base` schema to express the full Living Link surface (§6.1) as columns / filters / sorts. The architecture's biggest leverage point becomes user-facing.
- **Phase 3 — NSC Headlines as Default Column.** Headlines visible in every Bases view by default, context-aware rendering. Hover for full summary.
- **Phase 4 — Federation Auto-On.** `.base` queries automatically span cUniverses. `selected_vaults` becomes the opt-out channel.
- **Phase 5 — Five Acts Templates.** The named templates from §8 ship as built-in Bases — read-only system Bases + editable user copies on duplicate.
- **Phase 6 — Semantic + Cataloger + Index Columns.** §6.3 / §6.4 / §6.5 / §6.7 leverage points wired in.
- **Phase 7 — Cell-Edit on Typed Links.** §6.9. The relationship-editor mode. (Locked here per §13 #8.)
- **Phase 8+** — NL → query, generative lens suggestions, alternative renderers (likely via Wings).

Each phase is independently shippable and reversible. None of them precludes a future Bases-as-Wings-aware integration if that becomes the right shape.

---

## 13. Decisions Locked 2026-05-25

Replacing the original v1.0 §13 "Open Questions" — these are the resolutions Eisa locked in the same session.

| # | Question (v1.0) | Resolution (v1.1) | Where folded |
|---|---|---|---|
| 1 | Headlines unconditional or opt-in? | **Unconditional default. Rendering style is context-aware** — table view = sub-line, card view = main body, list view = inline, future view shapes specify their own rendering. | §6.2 expanded |
| 2 | `.base` schema extension for Living Links — which dimensions? | **All.** Full eight properties + lifecycle stage + aggregated note-level dimensions + relational queries (supersedes chains, contradicts-any-established). | §6.1 expanded |
| 3 | Five Acts templates — read-only / editable / both? | **Both.** Read-only system Bases at `{universe}/.constellation/bases/system/five-acts/`; "Customize" gesture duplicates to user-editable copy with `derivedFrom` lineage marker. | §8 expanded |
| 4 | Host-Note Assemblage Mode acceleration? | **Accelerate to Phase 1.5.** Ships right after the Rule 8 migration, before Living Link columns. | §7 substantially expanded; §12 roadmap |
| 5 | Federation default behavior? | **Auto.** `selected_vaults` becomes opt-OUT. Visible UI marker on federated rows. | §6.6 expanded; §10.6 added |
| 7 | Constellation Wings integration? | **Both directions.** Bases exposes data to Wings (IPC for external plugins to query) AND consumes from Wings (external plugins can register column types). Contract shape is Wings' to specify when Wings ships. | §10.7 added |
| 8 | Cell-edit on typed links phase? | **Phase 7** (last on the current roadmap). | §6.9, §12 |

### Pending one clarification before architecture phase begins

**#6 — Bases ↔ 360.3D relationship.** Eisa confirmed (v1.1, 2026-05-25): *"360.3D: still active. Yes, there is a relationship between Bases and 360.3D."*

The relationship exists. Its specific shape is the one item this paper cannot yet characterize because I do not have enough information to design it. The possible shapes — none preferred by me, listed for clarity:

- (a) **360.3D consumes Bases** — a Bases query feeds the set of notes that 360.3D stratifies (filter your universe down to a slice, then view that slice in 360.3D).
- (b) **360.3D is a view-shape of Bases** — table / card / list / **360.3D** as fourth view type, addressable per-Base.
- (c) **360.3D's stratification produces a Bases column** — the per-note stratum index becomes a sortable / filterable column inside any Bases view.
- (d) **Two-way** — some combination of the above.
- (e) **Something else entirely** that I cannot name without further information.

This is the one item where the BASIC RULE binds me: I will not invent the relationship. It needs Eisa's direction before Phase 1 architecture can begin (the cache schema may need to admit a 360.3D-aware column, depending on the answer). Until then, this paper records the existence of the relationship and the shape of the uncertainty.

---

## 14. Predecessor and Adjacent Documents

- **Predecessor (this Concept Paper line):** v1.0 at `docs/Constellation-Base-Concept-Paper-v1.0.md` — the pre-decisions draft, retained as historical record per the Mind v1.0/v1.1 pattern.
- **Predecessor (MVP):** `docs/BASES_MVP_SPEC.md` — the shipped MVP design (commit `c5b05f5c`, 2026-03-12). Source of the existing `.base` YAML format, the Tauri command surface, the table/card/list views, the cell-edit-in-place flow. **This Concept Paper does not invalidate the MVP; it articulates the destination.**
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

*End of Concept Paper v1.1. To be updated only on substantive change of vision — most notably, when the Bases ↔ 360.3D relationship (§13 #6) is characterized.*

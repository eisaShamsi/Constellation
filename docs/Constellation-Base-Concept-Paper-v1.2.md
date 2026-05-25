---
title: Constellation Base — Concept Paper
version: 1.2
date: 2026-05-25 (same day as v1.0 and v1.1; post-360.3D relationship lock)
status: All 8 open questions from v1.0 §13 locked. Concept paper complete and ready to serve as the guiding light through the design phase. Phase 1 (Rule 8 migration) architecture can begin when Eisa schedules.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
supersedes: v1.1 (preserved at docs/Constellation-Base-Concept-Paper-v1.1.md as historical record)
predecessor_versions:
  - v1.0 — pre-decisions draft (docs/Constellation-Base-Concept-Paper-v1.0.md)
  - v1.1 — 7 of 8 decisions locked (docs/Constellation-Base-Concept-Paper-v1.1.md)
predecessor_design: docs/BASES_MVP_SPEC.md (the MVP shipped 2026-03-12, commit c5b05f5c)
adjacent:
  - docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md (founding mission — Living Link Architecture, Five Acts)
  - docs/Constellation-NSC-Concept-Paper-v2.0.md (NSC as Core Plug-in; Bases is a downstream consumer)
  - docs/360.3D-Concept-Paper-v1.0.md (the cognitive companion surface — per-note standing)
  - docs/360.3D-Matrix-Reading-Guide-v1.0.md (practical reading of the matrix)
explicitly_out_of_scope:
  - Sight (disabled in core MIG-038, 2026-05-19; moved to External Plug-in / Constellation Wings)
  - Constellation Map (same status as Sight)
  - Constellation Mind / local-LLM stack (reverted MIG-046/047/048, 2026-05-25)
---

# Constellation Base — Concept Paper v1.2

> **What changed in v1.2** — Eisa confirmed and locked the Bases ↔ 360.3D relationship in the same session (2026-05-25). The eighth and final open question from v1.0 §13 is now resolved.
>
> - **§6.10 added** — *"Cognitive Engine Dimensions as Bases Columns (the 360.3D bridge)."* All 10 cognitive dimensions that 360.3D displays for one note become first-class queryable columns in Bases for many notes. Stratum, Maturity, Stage, Provenance, per-type connection counts, connection stratification, review pulse, trail membership, structural flags (orphan / fragile / blind-spots / tensions), word count. The architectural fact that makes this cheap: 360.3D does not compute new metrics — it displays measurements already taken by the Cognitive Engine. The data layer already carries everything; Bases just needs to expose it.
> - **§7 expanded** — Phase 1.5 also ships an *"Open in 360.3D"* row gesture. One click from any Bases row opens the full Inspector for that note, threading the comparison surface (Bases) to the standing surface (360.3D).
> - **§9 fourth differentiator added** — Cognitive Engine measurements queryable across the collection. **The first PKM in the world to make "intellectual altitude" and "developmental shape" queryable.** No other tool has these dimensions at all, let alone exposes them to a query layer.
> - **§12 roadmap updated** — Phase 2.5 inserted for the cognitive dimensions (between Living Link columns and NSC headlines, per Eisa's phase pick). The Open-in-360.3D gesture lands in Phase 1.5 as a lightweight navigation addition.
> - **§13 #6 resolved.** All eight original open questions now closed. The concept paper enters service as the durable guiding light for the design phase.
> - **§14 references** — added the 360.3D Concept Paper and Matrix Reading Guide as adjacent documents.

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
- **Rich.** Drawing on Living Links, summaries, embeddings, classifications, Cognitive Engine measurements (Stratum / Maturity / Stage / Provenance / connection geometry / structural flags), and federation — not just YAML scalars.
- **Plain.** The view is a `.base` YAML file alongside your notes. The data lives in each note's frontmatter (or in the CE's derived state). Walk away and lose nothing.
- **Shaped.** The view is rendered in the form that answers the question — table, card, list, possibly a typed-link-graph subset, possibly federated across universes, with a one-click bridge to 360.3D for any single-note deep dive.

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
- **Federated by default** — when the user's universe has cUniverse children, a Base spans them automatically (§6.6, §10.6).
- **Threaded to 360.3D** — every row carries an "Open in 360.3D" gesture; the Cognitive Engine dimensions 360.3D displays for one note are queryable as Bases columns for many notes (§6.10, §7, §10.8).

### IS NOT
- A **spreadsheet replacement.** Bases is not for users who want Excel. Use Excel.
- A **task manager.** Bases can render task-like notes, but Constellation does not ship a task feature pretending to be Bases.
- A **CRM, recipe manager, habit tracker, or any other vertical application.** Constellation is not Notion-with-templates. We ship the surface; the user brings the question.
- A **structure machine.** Bases does not invite users to design schemas. It reveals the structure already present in their notes.
- A **Sight successor or visualization layer.** Sight is an External Plug-in (Constellation Wings) per MIG-038, 2026-05-19. Bases is not its replacement and does not absorb its responsibilities. If Sight returns, it returns as Wings; Bases does not contain it.
- A **360.3D replacement.** 360.3D is the standing-with-one-note surface; Bases is the surveying-across-many surface. They are architecturally complementary (§6.10). The 360.3D Concept Paper §6 says *"Not a comparison view"* — Bases IS the comparison view. They thread; they do not subsume each other.

---

## 5. Founding Principles

Five principles, in priority order. When they conflict, higher-numbered yield to lower-numbered.

### 5.1 Form-Aligns-To-Purpose
Every column, filter, view shape, and rendering must carry cognitive meaning. If a view's geometry has degrees of freedom the question does not fill, change the primitive — don't fill the freedom with noise. A Base for "books I'm reading" needs progress + recency. A Base for "contradictions in my thinking" needs typed-link `contradicts` + confidence. Same notes, different shapes, because different questions.

This is the top principal of Constellation, restated here because it is the principle most violated by Bases-class features in the broader PKM market. Notion table views with twelve mostly-empty columns are the canonical violation.

### 5.2 Knowledge Formulation, Not Management
Bases must serve the Five Acts of Knowledge Creation: **Observation → Connection → Tension → Synthesis → Conviction**. A view that surfaces "notes that contradict notes I'm confident about" generates synthesis pressure. A view that lists "all my notes with `status: done`" is management. The first is the brand; the second is filler.

This is the difference between a Base view that produces thought and a Base view that organizes the products of thought.

### 5.3 Living Links + Cognitive Engine Measurements as Queryable Dimensions
No other PKM treats links as typed entities with confidence, weight, traversal count, and lifecycle stage; no other PKM measures a note's intellectual altitude, developmental maturity, formalization stage, or structural flags. **Constellation does both.** The Living Link Architecture (§6.1) and the Cognitive Engine measurement set (§6.10) together form Bases' single biggest leverage advantage. Queries that filter by `link.confidence > established` or sort by `note.stratum DESC` or surface `note.is_fragile AND note.review.is_due` are structurally impossible in Obsidian Bases, Notion databases, Tana search nodes, or Anytype sets. They are native here.

Bases is the surface that makes the Living Link Architecture and the Cognitive Engine measurements operable for everyday queries.

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

### 6.10 Cognitive Engine Dimensions as Bases Columns — the 360.3D Bridge (locked v1.2)

Per Eisa's lock on §13 #6 (v1.2, 2026-05-25), **every cognitive measurement displayed by the 360.3D Inspector becomes a first-class queryable column in Bases.** The 360.3D Concept Paper §6 states explicitly: *"Not a comparison view. The inspector is about one note's standing in itself."* Bases IS the comparison view; this section wires the two surfaces together.

**The architectural fact that makes this cheap:** 360.3D does not compute new metrics. It displays measurements already taken by the Cognitive Engine — `strata.rs`, `review.rs`, `trails.rs`, `note_links`, etc. The data layer already carries everything; Bases just needs to expose it.

**The full surface (mirroring 360.3D §3):**

| 360.3D dimension | Bases column | Source in CE |
|---|---|---|
| **Stratum** (L1 Datum → L8 Worldview) | `note.stratum` | `strata.rs` / `compute_stratum_for_note` |
| **Maturity** (Seed / Sapling / Evergreen / Canonical / Wilting) | `note.maturity` | CE maturity calculator |
| **Stage** (Fleeting / Literature / Permanent / Synthesis) | `note.stage` | CE stage signal |
| **Provenance — origin** (Received / Discovered / Mixed) | `note.provenance.origin` | derives-from chain analysis |
| **Provenance — trust depth** | `note.provenance.trust_depth` | chain length to root |
| **Per-type connection counts** | `note.connections.{supports, contradicts, causes, derives_from, generalizes, exemplifies, part_of, untyped, total}` | `note_links` aggregate |
| **Connection stratification** | `note.connection_strata` — distribution of neighbors by row | derived from neighbor strata |
| **Review pulse** | `note.review.last_reviewed`, `note.review.is_due` | `review.rs` / review-pulse.json |
| **Trail / lens membership** | `note.trails_count` | `trails.rs` |
| **Structural flags** | `note.is_orphan`, `note.is_fragile`, `note.blind_spots_count`, `note.tensions_count` | computed from `note_links` aggregates |
| **Word count** | `note.word_count` | `note_meta` |

**Queries this enables — direct lifts from 360.3D's mental shapes catalogue (`docs/360.3D-Matrix-Reading-Guide-v1.0.md` §8), applied to the collection:**

- *"All my L7 Paradigm notes with zero `contradicts` links"* — the Echo-at-high-stratum pattern across the universe, in one view.
- *"Notes flagged Fragile AND Due for review"* — the cleanup queue.
- *"Notes with the Hollow-Middle shape — L1 high, L2 and L3 nearly empty, L4+ asserted"* — a saved query the user returns to weekly to audit methodological soundness. (This is the Al-Idrisi pattern from the Reading Guide §10, now hunted across all paradigm notes at once.)
- *"L4 Concept notes with strong inbound from L8 Worldview"* — well-grounded concepts.
- *"Orphan notes at L1 with no outbound links"* — pure leaves needing connection.
- *"All notes promoted from Sapling to Evergreen in the last 60 days"* — the growth dashboard.
- *"Notes with `Untyped` count more than 10× the total typed count"* — the Uncommitted-Hub pattern (Reading Guide §8.2), surfaced across the collection so the user can systematically promote untyped wikilinks to typed forms.

These are 360.3D's reads, now queryable across the collection. A user can save them as Bases — including embedded in host-note assemblages (§7) — and check the universe's epistemic health at a glance.

**The threading principle:** Bases provides the WHICH (which notes share this cognitive shape across my universe?); 360.3D provides the WHY (for THIS one note, what's its full standing?). The user surveys in Bases, then opens 360.3D for the individual rows that interest them. The "Open in 360.3D" gesture (§7) is the bridge that makes this thread one-click.

---

## 7. The Host-Note / Assemblage Mode — and the Open-in-360.3D Gesture (Phase 1.5, locked v1.1 + v1.2)

Per Eisa's lock on §13 #4 (v1.1, 2026-05-25), the **host-note assemblage mode is accelerated to Phase 1.5** — it ships immediately after the Rule 8 migration (Phase 1) and before Living Link columns (Phase 2). Per Eisa's lock on §13 #6 (v1.2, 2026-05-25), the **"Open in 360.3D" row gesture also lands in Phase 1.5** as a lightweight navigation addition. These two together elevate Phase 1.5 from a single feature ship to a small constellation of capabilities that compose.

### 7.1 The host-note assemblage capability (locked v1.1)

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

**At Phase 1.5 the inline filter surface is the v1 set** — `is`, `is_not`, `contains`, `gt`, `lt`, `is_empty`, `is_not_empty`, plus the unconditional NSC headline column. Living Link filters arrive in Phase 2 and Cognitive Engine dimensions arrive in Phase 2.5 — which is when host-note assemblage really comes alive. The Phase 1.5 example above intentionally uses only v1-era filters.

The fully-realized end-state, by way of preview, is what the paper aspires to reach by Phase 3:

```markdown
## Productive contradictions
\`\`\`base
filter: tag=aristotle AND link.type=contradicts
view: cards
\`\`\`

## Mature, load-bearing claims (Phase 2 surface)
\`\`\`base
filter: tag=aristotle AND link.confidence=established AND link.traversal_count > 5
view: list
\`\`\`

## Paradigm-level claims without challenge (Phase 2.5 surface)
\`\`\`base
filter: tag=aristotle AND note.stratum>=L7 AND note.connections.contradicts=0
view: table
columns: [name, headline, stratum, connections.contradicts]
\`\`\`

## Notes due for review with fragile dependencies (Phase 2.5 surface)
\`\`\`base
filter: tag=aristotle AND note.review.is_due=true AND note.is_fragile=true
view: list
\`\`\`
```

Each view above is asking a *Constellation-specific cognitive question*. The host note holds the prose; the views hold the slices.

### 7.2 The "Open in 360.3D" row gesture (locked v1.2)

Every Bases row carries an affordance that opens the **full 360.3D Inspector for that note**. The exact UI is an architecture-phase detail (icon on hover, context-menu action, double-click name, or some combination — to be locked in the Phase 1.5 Architect doc). The principle is the same shape regardless: **one click from any Bases row drops the user into the full Stratification Matrix for that note**.

This is the navigation glue between *surveying* and *standing*:

1. The user surveys a collection in Bases — say, all L7 Paradigm notes with zero `contradicts` links.
2. They spot a row whose shape interests them or surprises them — perhaps a note they thought had been challenged.
3. One click → 360.3D opens for that note. The full matrix is visible; the user reads the standing in depth (Position / Connection Profile / Absence per the Reading Guide).
4. From the 360.3D matrix's existing click-to-navigate behavior, they can walk into a neighbor (the Inspector pushes a back-stack entry).
5. Returning to the Bases view, the user's query state is preserved — they can continue surveying from where they left off.

The two surfaces compose without overlap: Bases is the comparison surface 360.3D explicitly points to (per 360.3D Concept Paper §6); 360.3D is the deep-read surface Bases routes its rows into.

### 7.3 What makes this Constellation's, not Notion's

- **The host note is sacred.** It remains a plain `.md` file. The Bases are inline `.base` code blocks. No proprietary container, no lock-in. The user can take the file to any other markdown editor and lose only the rendered views — the prose, the frontmatter, and the embedded YAML are intact.
- **Each view leverages Constellation dimensions** (once Phase 2 / 2.5 ship the leverage points). The Aristotle host note above embeds (a) a confidence-aware table from NSC + Living Links, (b) a contradictions card view from typed links, (c) a paradigm-without-challenge audit using Cognitive Engine dimensions, (d) a fragile-and-due cleanup queue. Each view is asking a question only Constellation can answer.
- **Live, write-time-maintained.** Open the project page — all views are instant. No spinners. The host note loads at the speed of any other note because the views read derived state.
- **One-click to deep read.** Any row in any embedded view threads to its 360.3D standing without leaving the construction page.

### 7.4 The "Knowledge Construction Page" pattern

The host-note assemblage is not just embedding views; it is a **named UX pattern** Constellation can teach: the *Knowledge Construction Page*. A user creates a host note for any project, area, or inquiry, and embeds the views that reveal what they need to see. The note holds the prose; the views hold the slices; the row gesture threads to 360.3D for one-note depth. The user constructs knowledge by composing prose-with-views-with-deep-reads, not by tagging-and-hoping.

This is Effect §8 (the assemblage effect) rendered through Constellation's vocabulary. With Phase 1.5 acceleration, this is the second pattern users will encounter — right after they notice that their existing Bases now respond instantly.

### 7.5 Why accelerate to Phase 1.5

Three reasons that argued for the acceleration:
1. **It's the dominant user-facing pattern across the entire PKM market.** Notion is built on it; Obsidian Bases supports it via inline `.base` blocks; Tana, Capacities, Anytype all converge on it. Shipping it late is shipping the table stakes late.
2. **It composes with the Rule 8 migration.** Phase 1 establishes the cheap-lookup index; Phase 1.5 simply uses that index from inside any note. No new indexing work — only the renderer and the YAML-block extractor.
3. **It teaches the system's affordances by example.** A user who opens a host note with three embedded views immediately understands what a Bases view is for. A user who only sees the sidebar Bases list has a worse onboarding. With the 360.3D gesture also shipping in 1.5, the user immediately learns that Bases and 360.3D thread together — not as separate features but as a single workflow.

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

Four lines of differentiation, in honest priority order:

1. **Living Links as query dimensions.** Filter by confidence, sort by weight, group by typed link, surface contradictions automatically. Structurally impossible in any other PKM.
2. **Summary headlines visible by default, context-aware rendering.** Every row shows what the note *is about* before the user clicks. NSC is the differentiator that makes the dashboard effect (§1) work — you scan the headlines, not just the names.
3. **Federation across universes — auto by default.** Query spans cUniverses without the user opting in. The team and the individual share one view. The long-tail PKM dream nobody has shipped, made the *default*.
4. **Cognitive Engine measurements queryable across the collection** (added v1.2). Stratum, Maturity, Stage, Provenance, structural flags, review pulse — every dimension the 360.3D Inspector displays for one note becomes a Bases column for many notes. **The first PKM in the world to make "intellectual altitude" and "developmental shape" queryable.** No other tool has these dimensions at all, let alone exposes them to a query layer.

A Constellation Base that does not deliver these four is not a Constellation Base — it is Obsidian Bases running on a different runtime. The point of building Bases *of* Constellation rather than *into* Constellation is that the architecture *gives* us these four. Refusing to use them is the violation.

---

## 10. Architectural Mandates

Eight mandates, derived from the principles. The architecture phase will refine; these set the boundary.

### 10.1 Write-Time Derivation
A `bases_cache` SQLite table (or equivalent) is maintained by triggers on `note_meta` writes (and `note_links` writes for link-aware queries, and Cognitive Engine measurement writes for §6.10 columns). `query_base` becomes a cheap SQL lookup. No live filesystem scan in the keystroke-to-screen path.

### 10.2 File-Over-App
`.base` files remain plain YAML on disk. The cache is internal optimization; the source of truth is the file. Delete the cache → it rebuilds. Delete the `.base` file → the view is gone, every note's frontmatter is intact.

### 10.3 Instant on 10k
Every Base query, including federated and Living-Link-filtered and Cognitive-Engine-filtered, must return in under 50ms on a 7,600-note universe. This is the gate for the dashboard effect.

### 10.4 Multilingual Native
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. No "Phase 2 RTL" deferral. Mixed-script cells render correctly without per-feature engineering.

### 10.5 Embedded Bases Are First-Class
An inline ` ```base ` block in a host note has the same capability surface as a workspace-level `.base`. The host-note assemblage mode (Phase 1.5) is not a stripped-down sibling.

### 10.6 Federation Is Default-On (added v1.1)
Bases queries auto-span cUniverses. The user does not opt in. Visible UI affordance distinguishes federated rows from local rows so the federation is never invisible — only frictionless. `selected_vaults` is the explicit opt-out channel.

### 10.7 Constellation Wings Integration Is Bidirectional (added v1.1)
When the External Plug-in subsystem (Wings) ships, the Bases ↔ Wings contract is **bidirectional**:
- **Bases exposes data to Wings.** External Plug-ins can query Bases programmatically via a stable IPC contract: list available Bases, parse a `.base` definition, execute a query, observe data-changed events.
- **Bases consumes from Wings.** External Plug-ins can register their own data sources or column types into Bases via the same IPC. A weather plugin could register a `weather.temperature` column; a calendar plugin could register a `calendar.next_event` column.

The exact IPC shape is Wings' to specify (when Wings ships).

### 10.8 360.3D Bridge Is Bidirectional (added v1.2)
The Bases ↔ 360.3D contract is **bidirectional in data, light in UI**:
- **Bases consumes from the Cognitive Engine.** All 10 dimensions in §6.10 are read from the same CE measurements 360.3D reads from. No new computation — only new exposure. The `bases_cache` schema admits the CE-measurement columns.
- **Bases threads to 360.3D.** The Open-in-360.3D row gesture (§7.2) is the navigation glue. Bases sets the comparative context; 360.3D delivers the deep read for any chosen row.
- **360.3D need not know about Bases.** The Inspector continues to work as a standalone surface from any note open in the editor. The bridge is one-way at the navigation layer — Bases routes into 360.3D, not the reverse.

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
- **Bases-driven 360.3D filtering.** The reverse direction — a Bases query feeding into 360.3D's connection display — is not v1; possibly later. The v1 Bases ↔ 360.3D bridge is one-way at the UI layer (Bases routes into 360.3D; 360.3D operates independently).

---

## 12. Roadmap (Provisional — Updated v1.2)

Sequencing, not commitments. Each phase is itself a separate `/migration` workflow (Architect → Plan → Build → Audit → PCS).

- **Phase 0 — Concept** (this paper) ✓
- **Phase 1 — Rule 8 Migration.** `bases_cache` table, triggers, `query_base` cheap lookup. The architectural foundation. No new user-visible features beyond instant performance.
- **Phase 1.5 — Host-Note Assemblage Mode + Open-in-360.3D Navigation Gesture.** (Accelerated per Eisa lock §13 #4; gesture added per §13 #6 lock.) Inline ` ```base ` code blocks render as full Bases views in any host note. Same capability surface as workspace-level `.base` files. Every Bases row carries an "Open in 360.3D" affordance. Uses Phase 1's cheap-lookup index; uses Phase 1 filter set.
- **Phase 2 — Living Link Columns.** Extend `.base` schema to express the full Living Link surface (§6.1) as columns / filters / sorts. The architecture's biggest leverage point becomes user-facing.
- **Phase 2.5 — Cognitive Engine Dimensions as Bases Columns (the 360.3D Bridge).** (Locked per Eisa §13 #6, v1.2.) Extend `.base` schema to express the full Cognitive Engine measurement surface (§6.10) — Stratum, Maturity, Stage, Provenance, per-type connection counts, connection stratification, review pulse, trail membership, structural flags, word count — as columns / filters / sorts. The Stratification Matrix's reads become queryable across the universe.
- **Phase 3 — NSC Headlines as Default Column.** Headlines visible in every Bases view by default, context-aware rendering. Hover for full summary.
- **Phase 4 — Federation Auto-On.** `.base` queries automatically span cUniverses. `selected_vaults` becomes the opt-out channel.
- **Phase 5 — Five Acts Templates.** The named templates from §8 ship as built-in Bases — read-only system Bases + editable user copies on duplicate.
- **Phase 6 — Semantic + Cataloger + Index Columns.** §6.3 / §6.4 / §6.5 / §6.7 leverage points wired in.
- **Phase 7 — Cell-Edit on Typed Links.** §6.9. The relationship-editor mode. (Locked here per §13 #8.)
- **Phase 8+** — NL → query, generative lens suggestions, alternative renderers (likely via Wings), Bases-driven 360.3D filtering (the reverse-direction bridge).

Each phase is independently shippable and reversible. None of them precludes a future Bases-as-Wings-aware integration if that becomes the right shape.

---

## 13. Decisions Locked 2026-05-25

All eight original open questions are now resolved.

| # | Question (v1.0) | Resolution | Where folded |
|---|---|---|---|
| 1 | Headlines unconditional or opt-in? | **Unconditional default. Rendering style is context-aware** — table view = sub-line, card view = main body, list view = inline, future view shapes specify their own rendering. | §6.2 |
| 2 | `.base` schema extension for Living Links — which dimensions? | **All.** Full eight properties + lifecycle stage + aggregated note-level dimensions + relational queries. | §6.1 |
| 3 | Five Acts templates — read-only / editable / both? | **Both.** Read-only system Bases at `{universe}/.constellation/bases/system/five-acts/`; "Customize" gesture duplicates to user-editable copy with `derivedFrom` lineage marker. | §8 |
| 4 | Host-Note Assemblage Mode acceleration? | **Accelerate to Phase 1.5.** | §7, §12 |
| 5 | Federation default behavior? | **Auto.** `selected_vaults` becomes opt-OUT. Visible UI marker on federated rows. | §6.6, §10.6 |
| 6 | Bases ↔ 360.3D relationship? | **Adopted (c) + (d).** All 10 Cognitive Engine dimensions become Bases columns (Phase 2.5). Each Bases row carries an "Open in 360.3D" affordance (Phase 1.5). 360.3D and Bases are architecturally complementary — 360.3D = standing of one note, Bases = surveying many notes; both read from the same Cognitive Engine measurements. | §6.10, §7.2, §9.4, §10.8, §12 |
| 7 | Constellation Wings integration? | **Both directions.** Bases exposes data to Wings AND consumes from Wings. Contract shape is Wings' to specify when Wings ships. | §10.7 |
| 8 | Cell-edit on typed links phase? | **Phase 7** (last on the current roadmap). | §6.9, §12 |

With all eight closed, this concept paper enters service as the durable guiding light for the design phase. **Phase 1 (Rule 8 migration) architecture can begin when Eisa schedules.**

---

## 14. Predecessor and Adjacent Documents

- **Predecessor (this Concept Paper line):**
  - v1.0 at `docs/Constellation-Base-Concept-Paper-v1.0.md` — the pre-decisions draft.
  - v1.1 at `docs/Constellation-Base-Concept-Paper-v1.1.md` — 7 of 8 decisions locked.
  - Both retained as historical record per the Mind v1.0/v1.1 pattern.
- **Predecessor (MVP):** `docs/BASES_MVP_SPEC.md` — the shipped MVP design (commit `c5b05f5c`, 2026-03-12). Source of the existing `.base` YAML format, the Tauri command surface, the table/card/list views, the cell-edit-in-place flow. **This Concept Paper does not invalidate the MVP; it articulates the destination.**
- **Successor:** future Architect docs per `/migration` discipline, one per Phase above (MIG-NNN allocation at architecture time).
- **Adjacent — founding mission:** `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` (Living Link Architecture and the Five Acts).
- **Adjacent — service consumed:** `docs/Constellation-NSC-Concept-Paper-v2.0.md` (NSC as Core Plug-in; Bases is a downstream consumer).
- **Adjacent — companion surface (added v1.2):** `docs/360.3D-Concept-Paper-v1.0.md` (the per-note standing surface; the §6 *"Not a comparison view"* line is the architectural fact that anchors the Bases ↔ 360.3D split). `docs/360.3D-Matrix-Reading-Guide-v1.0.md` (the practical reading of the matrix; relevant for Phase 5 Five Acts templates and any system Bases that lift the Reading Guide's mental shapes into queries).
- **Adjacent — current state record:** `docs/Constellation Orientation & Onboarding v2.34.md` §4582 (current Bases subsystem record).
- **Explicitly NOT in scope:** the disabled Sight subsystem (MIG-038, 2026-05-19, moved to Constellation Wings as an External Plug-in) and the reverted Constellation Mind subsystem (MIG-046/047/048, reverted 2026-05-25). Bases is not their replacement and does not absorb their responsibilities.

---

## 15. Closing — The Guiding Light

When in doubt during the design phase, return to one question:

> **Does this make the user re-encounter their own thinking — or invite them to re-organize it?**

The first is knowledge formulation. The second is productivity theater. Constellation ships the first.

This paper exists to keep that distinction visible at every step from here to release.

---

*End of Concept Paper v1.2. With all eight original open questions resolved, this version enters service as the durable guiding light. To be updated only on substantive change of vision.*

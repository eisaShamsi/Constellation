---
title: Constellation Base — Concept Paper
version: 1.3
date: 2026-05-25 (same session as v1.0/v1.1/v1.2; post-CNS familiarization pass)
status: All 8 original open questions + 1 CNS-related lock complete. Concept paper enters service as the durable guiding light through the design phase.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
supersedes: v1.2 (preserved at docs/Constellation-Base-Concept-Paper-v1.2.md as historical record)
predecessor_versions:
  - v1.0 — pre-decisions draft (docs/Constellation-Base-Concept-Paper-v1.0.md)
  - v1.1 — 7 of 8 decisions locked (docs/Constellation-Base-Concept-Paper-v1.1.md)
  - v1.2 — all 8 closed + 360.3D bridge folded in (docs/Constellation-Base-Concept-Paper-v1.2.md)
predecessor_design: docs/BASES_MVP_SPEC.md (the MVP shipped 2026-03-12, commit c5b05f5c)
adjacent:
  - docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md (founding mission — Living Link Architecture, Five Acts)
  - docs/Constellation-NSC-Concept-Paper-v2.0.md (NSC as Core Plug-in; Bases is a downstream consumer)
  - docs/360.3D-Concept-Paper-v1.0.md (per-note cognitive-standing surface)
  - docs/360.3D-Matrix-Reading-Guide-v1.0.md (practical reading of the matrix)
  - docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md (CNS — formerly Sight v2 — help doc; primary user-facing reference)
explicitly_out_of_scope:
  - Sight (disabled in core MIG-038, 2026-05-19; moved to External Plug-in / Constellation Wings)
  - Constellation Map (same status as Sight)
  - Constellation Mind / local-LLM stack (reverted MIG-046/047/048, 2026-05-25)
---

# Constellation Base — Concept Paper v1.3

> **What changed in v1.3** — A familiarization pass on CNS (Constellation Nervous System, formerly Sight v2) raised one additional design question, locked in the same session (2026-05-25):
>
> - **§6.11 added** — *"CNS Measurements as Bases Columns (the Network Bridge)."* Community membership, centrality rank, top-bridge identification, load-bearing flag, Blind Spot participation count. CNS measures the graph-topology layer that the Cognitive Engine and 360.3D do not reach.
> - **§7.3 added** — the "Open in CNS" row gesture, alongside Open-in-360.3D in Phase 1.5. The Bases row now carries **two** threading gestures: per-note cognitive depth (360.3D) AND network neighborhood (CNS).
> - **§7.4 added** — "Two threading gestures, one surveying surface." Names the three-surface workflow Constellation uniquely enables: Bases (comparison of many) → 360.3D (cognitive depth of one) → CNS (network depth of one).
> - **§9 — fifth differentiator added.** Network topology queryable across the collection. **The first PKM to make "synthesis points" and "structural gaps" filterable across the note collection** rather than only visualizable in a single graph view.
> - **§10.9 added** — new architectural mandate. The CNS Bridge is bidirectional in data, light in UI (mirrors §10.8 for 360.3D). Includes the wrinkle that CNS metrics are graph-global, not per-note-cheap; freshness strategy (α / β / γ) deferred to Phase 2.6 Architect doc.
> - **§11 — out-of-scope item added.** Bases-driven CNS filtering (the reverse direction — a Bases query feeding into CNS's display) joins Bases-driven 360.3D filtering as Phase 8+ territory.
> - **§12 roadmap** — Phase 2.6 inserted after Phase 2.5 (the 360.3D dimensions) and before Phase 3 (NSC headlines).
> - **§13 row 9 added** — the CNS-bridge question locked: all measurements as columns (Phase 2.6), Open-in-CNS gesture (Phase 1.5), freshness deferred to Phase 2.6 Architect doc.
> - **Architectural note on CNS-stays-in-core** — per orientation v2.18 / MIG-038, CNS survived the Sight + Map disabling and remained a Core Plug-in. The current "Sight" subsystem in Constellation (post-rename) is *not* Sight v2 / CNS; it is the **sensory** view that was then itself disabled per MIG-038. CNS — the **connection-traversal** view — is in core, alive, and lives at `src-tauri/src/sight.rs` (the file name preserves the v2 lineage by the internal-name-stays convention).

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
- **Rich.** Drawing on Living Links, summaries, embeddings, classifications, Cognitive Engine measurements (Stratum / Maturity / Stage / Provenance / connection geometry / structural flags), CNS network measurements (community / centrality / bridges / load-bearing / blind-spots), and federation — not just YAML scalars.
- **Plain.** The view is a `.base` YAML file alongside your notes. The data lives in each note's frontmatter (or in the CE's / CNS's derived state). Walk away and lose nothing.
- **Shaped.** The view is rendered in the form that answers the question — table, card, list, possibly a typed-link-graph subset, possibly federated across universes, with one-click bridges to **both** 360.3D (per-note cognitive depth) **and** CNS (per-note network depth).

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
- **Threaded to 360.3D AND CNS** — every row carries two navigation gestures. *Open in 360.3D* drops the user into the Stratification Matrix for cognitive standing. *Open in CNS* drops the user into the gravity well for network neighborhood. The cognitive measurements 360.3D displays AND the network measurements CNS displays are also queryable as Bases columns (§6.10 + §6.11, §7.2 + §7.3, §10.8 + §10.9).

### IS NOT
- A **spreadsheet replacement.** Bases is not for users who want Excel. Use Excel.
- A **task manager.** Bases can render task-like notes, but Constellation does not ship a task feature pretending to be Bases.
- A **CRM, recipe manager, habit tracker, or any other vertical application.** Constellation is not Notion-with-templates. We ship the surface; the user brings the question.
- A **structure machine.** Bases does not invite users to design schemas. It reveals the structure already present in their notes.
- A **Sight successor or visualization layer.** Sight (the post-rename sensory view) is an External Plug-in (Constellation Wings) per MIG-038, 2026-05-19. Bases is not its replacement and does not absorb its responsibilities. If Sight returns, it returns as Wings; Bases does not contain it.
- A **360.3D replacement.** 360.3D is the standing-with-one-note surface; Bases is the surveying-across-many surface. They are architecturally complementary (§6.10). The 360.3D Concept Paper §6 says *"Not a comparison view"* — Bases IS the comparison view. They thread; they do not subsume each other.
- A **CNS replacement.** CNS is the network-traversal surface — the gravity well, the Universe Health card, the Top Bridges and Blind Spots panels. Bases is not a graph view and does not visualize the typed-link network. CNS and Bases share the same network measurements (§6.11) and thread by gesture (§7.3), but the gravity well lives in CNS only.

---

## 5. Founding Principles

Five principles, in priority order. When they conflict, higher-numbered yield to lower-numbered.

### 5.1 Form-Aligns-To-Purpose
Every column, filter, view shape, and rendering must carry cognitive meaning. If a view's geometry has degrees of freedom the question does not fill, change the primitive — don't fill the freedom with noise. A Base for "books I'm reading" needs progress + recency. A Base for "contradictions in my thinking" needs typed-link `contradicts` + confidence. Same notes, different shapes, because different questions.

This is the top principal of Constellation, restated here because it is the principle most violated by Bases-class features in the broader PKM market. Notion table views with twelve mostly-empty columns are the canonical violation.

### 5.2 Knowledge Formulation, Not Management
Bases must serve the Five Acts of Knowledge Creation: **Observation → Connection → Tension → Synthesis → Conviction**. A view that surfaces "notes that contradict notes I'm confident about" generates synthesis pressure. A view that lists "all my notes with `status: done`" is management. The first is the brand; the second is filler.

This is the difference between a Base view that produces thought and a Base view that organizes the products of thought.

### 5.3 Living Links + Cognitive Engine + CNS Measurements as Queryable Dimensions
No other PKM treats links as typed entities with confidence, weight, traversal count, and lifecycle stage. No other PKM measures a note's intellectual altitude, developmental maturity, formalization stage, or structural flags. No other PKM identifies graph-detected communities, top bridges, or structural Blind Spots. **Constellation does all three.** The Living Link Architecture (§6.1), the Cognitive Engine measurement set (§6.10), and the CNS network analysis (§6.11) together form Bases' decisive leverage advantage. Queries that filter by `link.confidence > established` or sort by `note.stratum DESC` or surface `note.cns.is_top_bridge AND note.review.is_due` are structurally impossible in Obsidian Bases, Notion databases, Tana search nodes, or Anytype sets. They are native here.

Bases is the surface that makes these three measurement layers operable for everyday queries.

### 5.4 Write-Time Derivation (CE Rule 8) — with one Acknowledged Wrinkle
Every Bases query reads from derived state maintained at write time — for per-note dimensions. No live filesystem scan, no on-demand frontmatter parse on a 10,000-note universe. The dashboard effect (Effect §1) dies if the query is slow; therefore the query must never be slow. This principle is architectural, not optional.

**The acknowledged wrinkle:** CNS measurements (community detection, modularity, centrality, top-bridge identification) are graph-global, not per-note-cheap. They require analyzing the whole link graph and cannot be maintained on every link write the way per-note dimensions can. Three candidate freshness strategies are documented at §6.11 and §13 row 9; the choice is locked at the Phase 2.6 Architect doc.

The current MVP violates the per-note write-time principle (`query_base` does a live scan). Reconciling Bases with Rule 8 is the largest architectural work this paper anticipates — but the principle precedes the architecture.

### 5.5 Language-First
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. Arabic frontmatter property names with English values, English property names with Persian values, mixed Hebrew + English in the same column header — all must render correctly without per-feature engineering. This is not an enhancement; it is the day-one design constraint.

---

## 6. The Constellation-Specific Leverage Points

Here is what is possible in a Constellation Base that is structurally impossible in any other PKM tool. These are the leverage points the architecture earns us.

### 6.1 Living Link Columns and Filters (full surface, locked v1.1)

Per Eisa's lock on §13 #2 (v1.1, 2026-05-25), **all** Living Link dimensions are exposed as queryable columns, filters, and sorts. The full surface:

**The eight link properties** (from `note_links` and the LINK file kind):
- `link.type` — the typed vocabulary: `supports`, `contradicts`, `causes`, `exemplifies`, `generalizes`, `derives_from`, `part_of`, `supersedes`, plus the default `associative`. Filter by type, group by type, count by type.
- `link.direction` — incoming, outgoing, or symmetric.
- `link.annotation` — the user's note on the link itself (a string). Searchable.
- `link.weight` — numeric, earned through use. Sortable and rangeable.
- `link.confidence` — the four-level enum: `hypothesis` → `evidence` → `established` → `contested`.
- `link.created`, `link.last_traversed`, `link.traversal_count`.

**Plus the lifecycle stage** — `link.lifecycle` — `Spark` | `Birth` | `Growth` | `Maturity` | `Dormancy` | `Renewal` | `Archival`.

**Plus aggregated dimensions** at the note level:
- `note.inbound_link_count_by_type`, `note.outbound_link_count_by_type`
- `note.total_inbound_link_weight`, `note.total_outbound_link_weight`
- `note.inbound_confidence_distribution`
- `note.dormant_link_count`

**Plus relational queries:**
- `note.is_supersedes_of(other)` / `note.is_superseded_by(other)`
- `note.contradicts_any_established` — a native Tension-Act filter.

These are queries no other PKM can pose because no other PKM tracks these dimensions.

### 6.2 NSC Summary Headlines — Unconditional Default (locked v1.1)

Per Eisa's lock on §13 #1 (v1.1, 2026-05-25), **every Constellation Base view shows the NSC headline unconditionally** as a default column. Context-aware rendering: table view = sub-line, card view = main body, list view = inline. The headline content is the same across contexts (NSC produces one headline per note); only the rendering varies.

### 6.3 Semantic Similarity as a Column Type

Embeddings exist (`embeddings.rs`, ONNX `multilingual-e5-small`, 384-dim, 100 languages). A Base column can render "similarity to this seed note" as a sortable number. A filter can express *"notes within 0.2 cosine distance of this seed note."*

### 6.4 The Cataloger (CECE) Classifications as a Dimension

CECE's 2-axis classifications (content-type × source) flow into Bases as filter and group-by axes.

### 6.5 Index Term Columns

The Index panel's term extraction (`notes_vocab` FTS5 dictionary) feeds Bases columns: top-N terms per note, filtering by term-with-bridge-to-lemma.

### 6.6 Federation Across cUniverses — Auto by Default (locked v1.1)

Per Eisa's lock on §13 #5 (v1.1, 2026-05-25), federation is automatic. Every Base spans cUniverses; `selected_vaults` is the opt-out channel; federated rows carry a visible marker. No mainstream PKM ships federated query across independent universes by default.

### 6.7 Search-Hybrid Filtering

FTS5 + structured + semantic + hybrid search modes are already wired through SearchHub. A Base filter can be expressed as a search query.

### 6.8 Cloud AI (later phase) for NL → Query

The cloud AI bridge (`ai/mod.rs`) can later expose *"describe the view you want"* → the model produces the `.base` YAML.

### 6.9 The Living Link Cell-Edit (Phase 7, locked v1.1)

The most ambitious leverage point: a Base cell editing a *typed link* rather than just a string property. Locked to Phase 7.

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
| **Connection stratification** | `note.connection_strata` (distribution of neighbors by row) | derived from neighbor strata |
| **Review pulse** | `note.review.last_reviewed`, `note.review.is_due` | `review.rs` / review-pulse.json |
| **Trail / lens membership** | `note.trails_count` | `trails.rs` |
| **Structural flags** | `note.is_orphan`, `note.is_fragile`, `note.blind_spots_count`, `note.tensions_count` | computed from `note_links` aggregates |
| **Word count** | `note.word_count` | `note_meta` |

**Example queries** — direct lifts from 360.3D's mental shapes catalogue applied across the collection:
- *"All my L7 Paradigm notes with zero `contradicts` links"* — the Echo-at-high-stratum pattern, universe-wide.
- *"Notes flagged Fragile AND Due for review"* — the cleanup queue.
- *"Notes with the Hollow-Middle shape"* — methodological-soundness audit (the Al-Idrisi pattern from the Reading Guide §10, hunted across all paradigm notes).
- *"L4 Concept notes with strong inbound from L8 Worldview"* — well-grounded concepts.
- *"Orphan notes at L1 with no outbound links"* — pure leaves needing connection.

**The threading principle:** Bases provides the WHICH (which notes share this cognitive shape?); 360.3D provides the WHY (for THIS one note, what's its full standing?). The "Open in 360.3D" gesture (§7.2) is the bridge that makes this one-click.

### 6.11 CNS Measurements as Bases Columns — the Network Bridge (locked v1.3)

Per Eisa's lock during the CNS familiarization pass (v1.3, 2026-05-25), **every measurement CNS surfaces about a note becomes a first-class queryable column in Bases.** CNS (Constellation Nervous System, formerly Sight v2) is Constellation's connection-traversal view — alive in core per orientation v2.18 / MIG-038 despite the Sight + Map disabling.

The §6.10 360.3D Bridge exposes per-note **cognitive** standing (intellectual altitude, developmental maturity, formalization stage). The §6.11 CNS Bridge exposes per-note **network** position (community membership, centrality, bridge role, structural-gap participation). They are categorically different measurement axes, both queryable in Bases for the first time in any PKM.

**The full surface:**

| CNS measurement | Bases column | What it surfaces |
|---|---|---|
| **Community membership** | `note.cns.community_id`, `note.cns.community_name` | Which graph-detected cluster this note belongs to. Filter / group by. |
| **Centrality rank** | `note.cns.centrality_rank` | Graph centrality position. Sort, range filter. |
| **Top bridge flag** | `note.cns.is_top_bridge` | Boolean — does this note link otherwise-separate communities? |
| **Bridge breadth** | `note.cns.bridge_count` | How many distinct communities this node bridges. The synthesis-power metric. |
| **Load-bearing flag** | `note.cns.is_load_bearing` | Boolean — high inbound, many dependents. |
| **Blind Spot participation** | `note.cns.blind_spot_count` | Number of suggested missing connections involving this note. |

**Queries this enables:**
- *"All my Top Bridge notes ordered by traversal weight"* — the synthesis-point inventory, sorted by how much they're actually being used.
- *"Notes in Community {X} ordered by stratum"* — explore an emergent cluster, vertically. (Combines §6.10 stratum + §6.11 community in one query.)
- *"Notes flagged as Blind Spot endpoints, with at least 3 Blind Spot suggestions, sorted by centrality"* — the connection-building queue ranked by potential leverage.
- *"Load-bearing notes that are also Fragile"* — the universe's structural risk register. The same note flagged twice from two different measurement axes (CNS's load-bearing AND CE's fragile).
- *"Top Bridges that haven't been traversed in 60 days"* — synthesis points going dormant. (Combines §6.1 `link.lifecycle` + §6.11 `is_top_bridge` — three measurement sources composing into one query.)
- *"Notes in a community with weak internal modularity, ordered by their centrality in that community"* — cluster repair candidates.

**The architectural wrinkle (acknowledged here, resolved at Phase 2.6):** CNS measurements are graph-global. Community detection, modularity, centrality — these depend on the *whole* link graph and are not cheap to recompute on every link write. This makes §6.11 different from §6.10, where per-note dimensions are already maintained at write time. The freshness strategy is locked at the Phase 2.6 Architect doc; three candidates are on the table:

- **(α) Debounced graph-write recompute.** Every link write schedules a debounced CNS-recompute (~30s after last write); results materialize into `cns_cache`. Eventually consistent; bursty under heavy editing.
- **(β) CNS-open caching.** CNS's existing read-time computation runs when the user opens CNS; results persist to `cns_cache`. Bases reads the most recent cache. Stale if CNS hasn't been opened recently — surfaced with a "last analyzed N days ago" indicator.
- **(γ) Scheduled recompute.** A background tokio task recomputes once per hour (or per universe-open). Predictable; never blocks.

The right answer depends on (a) CNS's actual compute cost on a 7,600-note universe and (b) the freshness tolerance Eisa wants for CNS-flavored Bases. Decision lives in Phase 2.6.

**The threading principle (mirrors §6.10):** Bases provides the WHICH (which notes share this network property?); CNS provides the WHY (for THIS one note, what's its network neighborhood?). The "Open in CNS" gesture (§7.3) is the bridge that makes this one-click.

---

## 7. The Host-Note / Assemblage Mode — and the Two Threading Gestures (Phase 1.5)

Per Eisa's locks on §13 #4 (v1.1) and §13 #6 (v1.2) and §13 row 9 (v1.3), **Phase 1.5 ships three things together**: the host-note assemblage mode + the Open-in-360.3D row gesture + the Open-in-CNS row gesture. Together they elevate Phase 1.5 from a single feature ship to a small constellation of capabilities that compose.

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
...prose continues here...
```

Two Bases views inline in one host note, each answering a different question about the same collection. Each view is instant because the data they read is the same write-time-derived state Phase 1 establishes.

**At Phase 1.5 the inline filter surface is the v1 set** — `is`, `is_not`, `contains`, `gt`, `lt`, `is_empty`, `is_not_empty`, plus the unconditional NSC headline column. Living Link filters arrive in Phase 2; Cognitive Engine dimensions arrive in Phase 2.5; CNS network measurements arrive in Phase 2.6 — which is when host-note assemblage really comes alive.

The fully-realized end-state, by way of preview:

```markdown
## Paradigm-level claims without challenge (Phase 2.5 surface)
\`\`\`base
filter: tag=aristotle AND note.stratum>=L7 AND note.connections.contradicts=0
view: table
columns: [name, headline, stratum, connections.contradicts]
\`\`\`

## Top Bridges into this project (Phase 2.6 surface)
\`\`\`base
filter: tag=aristotle AND note.cns.is_top_bridge=true
view: cards
columns: [name, headline, cns.community_name, cns.bridge_count]
\`\`\`
```

### 7.2 The "Open in 360.3D" row gesture (locked v1.2)

Every Bases row carries an affordance that opens the **full 360.3D Inspector for that note**. One click from any row drops the user into the full Stratification Matrix for that note. The user reads the standing in depth (Position / Connection Profile / Absence per the Reading Guide), can walk neighbors via 360.3D's click-to-navigate, and returns to Bases with their query preserved.

### 7.3 The "Open in CNS" row gesture (locked v1.3)

Alongside Open-in-360.3D, every Bases row carries a second navigation affordance: **Open in CNS**. The exact UI is an architecture-phase detail (a network icon, a context-menu action, a keyboard shortcut, or some combination — to be locked in the Phase 1.5 Architect doc). The principle is the same shape regardless: **one click from any Bases row drops the user into CNS centered on that note**, with the note's community highlighted and the gravity well laid out around it.

This is the second navigation glue — between *surveying* and *network neighborhood reading*:

1. The user surveys a collection in Bases — say, all Top Bridge notes ordered by traversal weight.
2. They spot a row whose role they want to inspect — perhaps a bridge that doesn't match their mental model of a synthesis point.
3. One click → CNS opens with that node selected, its community surfaced, its connections rendered.
4. The CNS preview panel shows community + centrality + link lists — without committing to opening the note in the editor.
5. Returning to the Bases view, the user's query state is preserved.

### 7.4 Two threading gestures, one surveying surface (added v1.3)

After Phase 1.5 ships, every Bases row carries **two threading affordances**, both lightweight, both leading to deep-read surfaces:

| Gesture | Routes to | What it shows |
|---|---|---|
| Open in **360.3D** | The per-note cognitive standing surface | Stratification Matrix — Position / Connection Profile / Absence |
| Open in **CNS** | The per-note network neighborhood surface | Community, centrality, top-bridge role, blind-spot suggestions |

The user surveys in Bases and chooses *which depth* to drop into for any given row. Both deep-read surfaces remain standalone — they work fine from any open note. Bases just makes the routing one-click from a comparative context.

This is the **three-surface workflow** that distinguishes Constellation:
- **Bases** = comparison of many notes (the surveying)
- **360.3D** = cognitive standing of one note (the cognitive depth)
- **CNS** = network position of one note (the network depth)

No other PKM has all three. No other PKM threads them together with single-click row gestures.

### 7.5 What makes this Constellation's, not Notion's

- **The host note is sacred.** It remains a plain `.md` file. The Bases are inline `.base` code blocks. No proprietary container, no lock-in.
- **Each view leverages Constellation dimensions** (once Phase 2 / 2.5 / 2.6 ship the leverage points). Living Links, Cognitive Engine measurements, CNS network position — all queryable.
- **Live, write-time-maintained** for per-note dimensions; **freshness-strategy-determined** for CNS network measurements (§6.11). Open the project page — all views are instant for the per-note columns; CNS columns show their freshness indicator if the strategy chosen at Phase 2.6 has any latency.
- **Two clicks to depth.** Any row in any embedded view threads to its 360.3D standing OR its CNS neighborhood without leaving the construction page.

### 7.6 The "Knowledge Construction Page" pattern

The host-note assemblage is not just embedding views; it is a **named UX pattern** Constellation can teach: the *Knowledge Construction Page*. A user creates a host note for any project, area, or inquiry, and embeds the views that reveal what they need to see. The note holds the prose; the views hold the slices; the two row gestures thread to 360.3D (cognitive) or CNS (network) for one-note depth in either direction. The user constructs knowledge by composing prose-with-views-with-deep-reads, not by tagging-and-hoping.

This is Effect §8 (the assemblage effect) rendered through Constellation's vocabulary.

### 7.7 Why accelerate to Phase 1.5

Three reasons that argued for the acceleration:
1. **It's the dominant user-facing pattern across the entire PKM market.** Notion is built on it; Obsidian Bases supports it via inline `.base` blocks; Tana, Capacities, Anytype all converge on it. Shipping it late is shipping the table stakes late.
2. **It composes with the Rule 8 migration.** Phase 1 establishes the cheap-lookup index; Phase 1.5 simply uses that index from inside any note.
3. **It teaches the system's affordances by example.** A user who opens a host note with three embedded views immediately understands what a Bases view is for. With the two threading gestures (360.3D and CNS) also shipping in 1.5, the user immediately learns that Bases is the *survey-and-thread* surface that ties Constellation's three measurement surfaces together as a workflow.

---

## 8. The Five Acts as Operational Templates (locked v1.1)

The Five Acts of Knowledge Creation are Constellation's cognitive model. Bases makes them operable. Each Act ships as a named template — not a schema for the user to fill, but a query Constellation runs over what the user has already written.

| Act | Bases Template | What it Surfaces |
|---|---|---|
| **Observation** | "Recent Captures" | Last 14 days, sorted by creation date, NSC headlines visible. The intake queue. |
| **Connection** | "Single-Direction Conduits" | Notes with high outbound link counts but low inbound. Awaiting reciprocation. |
| **Tension** | "Productive Frictions" | Notes connected by `contradicts`-typed links. The forge of synthesis. |
| **Synthesis** | "Convergence Points" | Notes with high inbound link weight from multiple `established`-confidence sources. Where threads have joined. |
| **Conviction** | "Load-Bearing Work" | Notes with `confidence: established` AND high traversal count. The pieces of the universe doing the most work. |

### Distribution model — both shapes (locked v1.1)

Per Eisa's lock on §13 #3 (v1.1, 2026-05-25): **both shapes**:
1. **Read-only system Bases** at `{universe}/.constellation/bases/system/five-acts/`.
2. **Editable user copies on duplicate** via "Customize" gesture, retaining a `derivedFrom: system/five-acts/{act-name}` lineage marker.

This is the antidote to the structure-invitation effect (§3): users learn Bases not by being given empty schemas to populate, but by seeing the system reveal patterns in work they have already done.

---

## 9. What Sets a Constellation Base Apart

Five lines of differentiation, in honest priority order:

1. **Living Links as query dimensions.** Filter by confidence, sort by weight, group by typed link, surface contradictions automatically. Structurally impossible in any other PKM.
2. **Summary headlines visible by default, context-aware rendering.** Every row shows what the note *is about* before the user clicks. NSC is the differentiator that makes the dashboard effect (§1) work.
3. **Federation across universes — auto by default.** Query spans cUniverses without the user opting in. The long-tail PKM dream nobody has shipped, made the *default*.
4. **Cognitive Engine measurements queryable across the collection** (added v1.2). Stratum, Maturity, Stage, Provenance, structural flags, review pulse — every dimension the 360.3D Inspector displays for one note becomes a Bases column for many notes. **The first PKM in the world to make "intellectual altitude" and "developmental shape" queryable.**
5. **Network topology queryable across the collection** (added v1.3). Community membership, centrality rank, top-bridge identification, load-bearing flags, Blind Spot participation — the graph-global measurements CNS surfaces for the universe become Bases columns and filters. **The first PKM to make "synthesis points" and "structural gaps" filterable across the note collection** rather than only visualizable in a single graph view.

A Constellation Base that does not deliver these five is not a Constellation Base — it is Obsidian Bases running on a different runtime. The point of building Bases *of* Constellation rather than *into* Constellation is that the architecture *gives* us these five. Refusing to use them is the violation.

---

## 10. Architectural Mandates

Nine mandates, derived from the principles. The architecture phase will refine; these set the boundary.

### 10.1 Write-Time Derivation (per-note dimensions)
A `bases_cache` SQLite table (or equivalent) is maintained by triggers on `note_meta` writes (and `note_links` writes for link-aware queries, and Cognitive Engine measurement writes for §6.10 columns). `query_base` becomes a cheap SQL lookup for per-note dimensions. No live filesystem scan in the keystroke-to-screen path.

### 10.2 File-Over-App
`.base` files remain plain YAML on disk. The cache is internal optimization; the source of truth is the file. Delete the cache → it rebuilds. Delete the `.base` file → the view is gone, every note's frontmatter is intact.

### 10.3 Instant on 10k
Every Base query, including federated and Living-Link-filtered and Cognitive-Engine-filtered, must return in under 50ms on a 7,600-note universe. **For CNS-filtered queries, "instant" is bounded by the freshness strategy chosen at Phase 2.6** — the read is instant; the underlying CNS analysis may be debounced, cached, or scheduled.

### 10.4 Multilingual Native
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. No "Phase 2 RTL" deferral. Mixed-script cells render correctly without per-feature engineering.

### 10.5 Embedded Bases Are First-Class
An inline ` ```base ` block in a host note has the same capability surface as a workspace-level `.base`. The host-note assemblage mode (Phase 1.5) is not a stripped-down sibling.

### 10.6 Federation Is Default-On (added v1.1)
Bases queries auto-span cUniverses. The user does not opt in. Visible UI affordance distinguishes federated rows from local rows so the federation is never invisible — only frictionless. `selected_vaults` is the explicit opt-out channel.

### 10.7 Constellation Wings Integration Is Bidirectional (added v1.1)
When the External Plug-in subsystem (Wings) ships, the Bases ↔ Wings contract is **bidirectional**: Bases exposes data to Wings (IPC for external plugins to query) AND consumes from Wings (external plugins can register column types).

### 10.8 360.3D Bridge — Bidirectional in Data, Light in UI (added v1.2)
- **Bases consumes from the Cognitive Engine.** All 10 dimensions in §6.10 are read from the same CE measurements 360.3D reads from. No new computation — only new exposure.
- **Bases threads to 360.3D.** The Open-in-360.3D row gesture (§7.2) is the navigation glue.
- **360.3D need not know about Bases.** The Inspector continues to work as a standalone surface from any note open in the editor. The bridge is one-way at the navigation layer.

### 10.9 CNS Bridge — Bidirectional in Data, Light in UI (added v1.3)
- **Bases consumes from CNS.** All measurements in §6.11 are read from the same CNS analysis that powers the Universe Health card, the gravity well, Top Bridges, Communities, and Blind Spots. No new computation in Bases; only new exposure.
- **Bases threads to CNS.** The Open-in-CNS row gesture (§7.3) is the navigation glue. Bases sets the comparative context; CNS delivers the network neighborhood for any chosen row.
- **CNS need not know about Bases.** The Network view continues to work as a standalone surface from the dock. The bridge is one-way at the navigation layer.
- **Freshness strategy is deferred.** Because CNS measurements are graph-global, they are NOT cheap to maintain at write time the way per-note measurements are. The three candidate strategies (α / β / γ from §6.11) are locked at the Phase 2.6 Architect doc, not in this Concept Paper.

---

## 11. Out of Scope (v1)

These are excluded by design from the first delivery, not because they're bad but because they would dilute the principles.

- **Pre-built vertical templates** (CRM, recipe manager, habit tracker, etc.). The Five Acts templates ship; no others.
- **Aggregation formulas** beyond what's needed for the Five Acts. Later phase.
- **Calendar / timeline / board / gallery views.** v1 ships table + card + list. Other shapes ship when a Constellation-specific question earns them.
- **Cloud AI NL → query.** The architecture admits it; v1 doesn't ship it.
- **Generative lens suggestions.** Research mode, not v1.
- **Real-time multi-user collaboration in a Base.** Constellation is local-first.
- **Bases-from-external-data** (Notion's "Connections" / Coda's "Packs"). Wings' responsibility per §10.7 when Wings ships.
- **Bases-driven 360.3D filtering.** The reverse direction — a Bases query feeding into 360.3D's connection display — is not v1; possibly Phase 8+.
- **Bases-driven CNS filtering** (added v1.3). The reverse direction — a Bases query feeding into CNS's gravity-well rendering (show only these notes and their links) — is not v1; possibly Phase 8+.

---

## 12. Roadmap (Provisional — Updated v1.3)

Sequencing, not commitments. Each phase is itself a separate `/migration` workflow (Architect → Plan → Build → Audit → PCS).

- **Phase 0 — Concept** (this paper) ✓
- **Phase 1 — Rule 8 Migration.** `bases_cache` table, triggers, `query_base` cheap lookup. The architectural foundation.
- **Phase 1.5 — Host-Note Assemblage + Open-in-360.3D + Open-in-CNS gestures.** (Accelerated per §13 #4; gestures added per §13 #6 and #9.) Inline ` ```base ` code blocks render as full Bases views in any host note. Every row carries two threading affordances.
- **Phase 2 — Living Link Columns.** §6.1 surface becomes user-facing.
- **Phase 2.5 — Cognitive Engine Dimensions (the 360.3D Bridge).** §6.10 surface becomes user-facing.
- **Phase 2.6 — CNS Network Measurements (the CNS Bridge).** §6.11 surface becomes user-facing. **Includes the freshness-strategy decision** (α / β / γ).
- **Phase 3 — NSC Headlines as Default Column.** Headlines visible in every Bases view by default, context-aware rendering.
- **Phase 4 — Federation Auto-On.** `.base` queries automatically span cUniverses.
- **Phase 5 — Five Acts Templates.** §8 named templates ship as both read-only system Bases and editable user copies.
- **Phase 6 — Semantic + Cataloger + Index Columns.** §6.3 / §6.4 / §6.5 / §6.7 leverage points wired in.
- **Phase 7 — Cell-Edit on Typed Links.** §6.9. The relationship-editor mode.
- **Phase 8+** — NL → query, generative lens suggestions, alternative renderers (likely via Wings), Bases-driven 360.3D filtering, Bases-driven CNS filtering.

Each phase is independently shippable and reversible. None of them precludes a future Bases-as-Wings-aware integration if that becomes the right shape.

---

## 13. Decisions Locked 2026-05-25

All eight original open questions from v1.0 §13 were closed in v1.2. The CNS familiarization pass that produced v1.3 raised one additional design question, locked in the same session.

| # | Question | Resolution | Where folded |
|---|---|---|---|
| 1 | Headlines unconditional or opt-in? | **Unconditional default.** Context-aware rendering. | §6.2 |
| 2 | `.base` schema extension for Living Links — which dimensions? | **All.** | §6.1 |
| 3 | Five Acts templates — read-only / editable / both? | **Both.** | §8 |
| 4 | Host-Note Assemblage Mode acceleration? | **Accelerate to Phase 1.5.** | §7, §12 |
| 5 | Federation default behavior? | **Auto.** `selected_vaults` becomes opt-OUT. | §6.6, §10.6 |
| 6 | Bases ↔ 360.3D relationship? | **All 10 CE dimensions as Bases columns (Phase 2.5). Open-in-360.3D gesture (Phase 1.5).** | §6.10, §7.2, §9.4, §10.8, §12 |
| 7 | Constellation Wings integration? | **Both directions.** | §10.7 |
| 8 | Cell-edit on typed links phase? | **Phase 7.** | §6.9, §12 |
| 9 | **Bases ↔ CNS relationship?** (raised v1.3) | **All 6 CNS measurements as Bases columns (Phase 2.6). Open-in-CNS gesture (Phase 1.5). Freshness strategy (α/β/γ) deferred to Phase 2.6 Architect doc.** | §6.11, §7.3, §9.5, §10.9, §11, §12 |

With all nine closed, this concept paper enters service as the durable guiding light for the design phase. **Phase 1 (Rule 8 migration) architecture can begin when Eisa schedules.**

---

## 14. Predecessor and Adjacent Documents

- **Predecessor (this Concept Paper line):**
  - v1.0 at `docs/Constellation-Base-Concept-Paper-v1.0.md` — pre-decisions draft.
  - v1.1 at `docs/Constellation-Base-Concept-Paper-v1.1.md` — 7 of 8 locked.
  - v1.2 at `docs/Constellation-Base-Concept-Paper-v1.2.md` — all 8 closed + 360.3D bridge.
  - All retained as historical record per the Mind v1.0/v1.1 pattern.
- **Predecessor (MVP):** `docs/BASES_MVP_SPEC.md` — the shipped MVP design (commit `c5b05f5c`, 2026-03-12). **This Concept Paper does not invalidate the MVP; it articulates the destination.**
- **Successor:** future Architect docs per `/migration` discipline, one per Phase above (MIG-NNN allocation at architecture time).
- **Adjacent — founding mission:** `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` (Living Link Architecture and the Five Acts).
- **Adjacent — service consumed:** `docs/Constellation-NSC-Concept-Paper-v2.0.md` (NSC as Core Plug-in; Bases is a downstream consumer).
- **Adjacent — companion cognitive surface** (added v1.2): `docs/360.3D-Concept-Paper-v1.0.md` and `docs/360.3D-Matrix-Reading-Guide-v1.0.md`.
- **Adjacent — companion network surface** (added v1.3): `docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md` — the CNS help doc. CNS does not yet have a standalone Concept Paper; the help doc is the primary user-facing reference. Implementation: `src-tauri/src/sight.rs` (the file name preserves the v2 lineage by the internal-name-stays convention; CNS is *not* the Sight subsystem disabled per MIG-038).
- **Adjacent — current state record:** `docs/Constellation Orientation & Onboarding v2.34.md` §4582 (current Bases subsystem record).
- **Explicitly NOT in scope:** the disabled Sight subsystem (MIG-038, 2026-05-19, moved to Constellation Wings as an External Plug-in) and the reverted Constellation Mind subsystem (MIG-046/047/048, reverted 2026-05-25).

---

## 15. Closing — The Guiding Light

When in doubt during the design phase, return to one question:

> **Does this make the user re-encounter their own thinking — or invite them to re-organize it?**

The first is knowledge formulation. The second is productivity theater. Constellation ships the first.

This paper exists to keep that distinction visible at every step from here to release.

---

*End of Concept Paper v1.3. With all nine decisions resolved, this version enters service as the durable guiding light. To be updated only on substantive change of vision.*

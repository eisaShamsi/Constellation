---
title: Constellation Base — Concept Paper
version: 1.4
date: 2026-05-25 (same session as v1.0/v1.1/v1.2/v1.3; post-Cataloger familiarization pass)
status: All 10 design decisions locked. Concept paper enters service as the durable guiding light through the design phase.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
supersedes: v1.3 (preserved at docs/Constellation-Base-Concept-Paper-v1.3.md as historical record)
predecessor_versions:
  - v1.0 — pre-decisions draft
  - v1.1 — 7 of 8 closed
  - v1.2 — all 8 + 360.3D bridge folded in
  - v1.3 — CNS bridge added
predecessor_design: docs/BASES_MVP_SPEC.md (the MVP shipped 2026-03-12, commit c5b05f5c)
adjacent:
  - docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md (founding mission)
  - docs/Constellation-NSC-Concept-Paper-v2.0.md (NSC service Bases consumes)
  - docs/360.3D-Concept-Paper-v1.0.md (per-note cognitive standing surface)
  - docs/360.3D-Matrix-Reading-Guide-v1.0.md (practical matrix reading)
  - docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md (CNS — formerly Sight v2 — help doc)
  - docs/Constellation-CECE-Concept-Paper-v1.0.md (CECE = The Cataloger — added v1.4)
  - docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md (unified CE / 360.3D / Cataloger user docs — added v1.4)
explicitly_out_of_scope:
  - Sight (disabled in core MIG-038, 2026-05-19; moved to External Plug-in / Constellation Wings)
  - Constellation Map (same status as Sight)
  - Constellation Mind / local-LLM stack (reverted MIG-046/047/048, 2026-05-25)
  - CECE Reasoning cataloger (depended on the reverted local-LLM stack; the 5-cataloger heuristic ensemble is what ships and what Bases queries)
---

# Constellation Base — Concept Paper v1.4

> **What changed in v1.4** — Eisa caught a gap: **The Cataloger (CECE) was treated as a Phase 6 afterthought** in v1.3 §6.4 (lumped with Semantic + Index columns) when it is a **Core Plug-in** with the same architectural status as CNS. v1.4 promotes The Cataloger to the same Bridge treatment as the §6.10 360.3D Bridge and §6.11 CNS Bridge:
>
> - **§6.12 added** — *"CECE Measurements as Bases Columns — the Epistemic Bridge."* Source × Content Type classifications, regime state (unanimous / strong_majority / split), disambiguation flags, per-cataloger reasoning provenance become Bases columns at Phase 2.7. CECE's two axes — **Source** (where the knowledge came from: 11 parents → 41 leaves drawn from five civilizations' epistemologies) and **Content-type** (what kind of knowledge it is: 5 branches → ~218 sub-nodes) — are unique to Constellation. No other PKM classifies notes on these epistemological axes.
> - **§6.4 retired.** The placeholder text from v1.3 §6.4 has been folded into the new §6.12; §6.4 is now a redirect note.
> - **§7.4 added** — the "Open in The Cataloger" row gesture, alongside Open-in-360.3D and Open-in-CNS, all in Phase 1.5. Every Bases row now carries **three threading gestures**: per-note cognitive depth (360.3D), network neighborhood (CNS), and epistemic provenance (The Cataloger).
> - **§7.5 reframed** — "Two threading gestures, one surveying surface" becomes "**Three threading gestures**, one surveying surface." The three-surface workflow is now a **four-surface workflow**: Bases (surveying) → 360.3D (cognitive depth) → CNS (network depth) → The Cataloger (epistemic depth). **No other PKM has all four; no other PKM threads them.**
> - **§9 sixth differentiator added** — Epistemic classification queryable across the collection. **The first PKM to make "where this knowledge came from" and "what kind of knowledge this is" filterable across the note collection.**
> - **§10.10 added** — new architectural mandate. The CECE Bridge is bidirectional in data, light in UI (mirrors §10.8 + §10.9). **No freshness wrinkle** unlike CNS — CECE's classifications are persisted in `sources_suggestions` (engine proposals) and `note_meta.properties_json` mirroring frontmatter (user-approved canonical state). Bases reads both cheaply via SQL JOIN.
> - **§11** — Bases-driven CECE filtering added as Phase 8+ out-of-scope.
> - **§12 roadmap** — Phase 2.7 inserted after Phase 2.6 (CNS Bridge) and before Phase 3 (NSC Headlines).
> - **§13 row 10 added** — the Cataloger-bridge question locked: all CECE measurements as columns (Phase 2.7); Open-in-Cataloger gesture (Phase 1.5); Reasoning cataloger explicitly out of scope (depends on the reverted Mind stack).
> - **Architectural note throughout** — CECE was promoted from subsystem to Core Plug-in on 2026-05-19 (same day as the Sight + Map disabling per MIG-038). User-facing name: **"The Cataloger"** (en) / **المُصنِّف** (ar). Internal engine name: **CECE**. Both appear in this paper per the CECE v1.0 §10 naming decision.

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
- **Rich.** Drawing on Living Links, summaries, embeddings, **CECE epistemic classifications**, **Cognitive Engine measurements** (Stratum / Maturity / Stage / Provenance / connection geometry / structural flags), **CNS network measurements** (community / centrality / bridges / load-bearing / blind-spots), and federation — not just YAML scalars.
- **Plain.** The view is a `.base` YAML file alongside your notes. The data lives in each note's frontmatter (or in the CE's / CNS's / CECE's derived state). Walk away and lose nothing.
- **Shaped.** The view is rendered in the form that answers the question — table, card, list, possibly federated across universes — with **one-click bridges to three deep-read surfaces**: 360.3D (cognitive depth), CNS (network depth), The Cataloger (epistemic depth).

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
- **Threaded to 360.3D, CNS, AND The Cataloger** — every row carries three navigation gestures. Open-in-360.3D drops into the Stratification Matrix (cognitive standing). Open-in-CNS drops into the gravity well (network neighborhood). Open-in-Cataloger drops into the Source Review panel (epistemic classification). The measurements that each of these surfaces displays are also queryable as Bases columns (§6.10 + §6.11 + §6.12, §7.2 + §7.3 + §7.4, §10.8 + §10.9 + §10.10).

### IS NOT
- A **spreadsheet replacement.** Bases is not for users who want Excel. Use Excel.
- A **task manager.** Bases can render task-like notes, but Constellation does not ship a task feature pretending to be Bases.
- A **CRM, recipe manager, habit tracker, or any other vertical application.** Constellation is not Notion-with-templates. We ship the surface; the user brings the question.
- A **structure machine.** Bases does not invite users to design schemas. It reveals the structure already present in their notes.
- A **Sight successor or visualization layer.** Sight (the post-rename sensory view) is an External Plug-in (Constellation Wings) per MIG-038, 2026-05-19. Bases is not its replacement.
- A **360.3D / CNS / Cataloger replacement.** Each of these surfaces is a per-note deep-read tool with its own purpose: 360.3D = cognitive standing; CNS = network position; The Cataloger = epistemic classification. Bases is the comparison surface; they are deep-read surfaces. **They thread; they do not subsume each other.**

---

## 5. Founding Principles

Five principles, in priority order. When they conflict, higher-numbered yield to lower-numbered.

### 5.1 Form-Aligns-To-Purpose
Every column, filter, view shape, and rendering must carry cognitive meaning. If a view's geometry has degrees of freedom the question does not fill, change the primitive — don't fill the freedom with noise. This is the top principal of Constellation, restated here because it is the principle most violated by Bases-class features in the broader PKM market.

### 5.2 Knowledge Formulation, Not Management
Bases must serve the Five Acts of Knowledge Creation: **Observation → Connection → Tension → Synthesis → Conviction**. A view that surfaces "notes that contradict notes I'm confident about" generates synthesis pressure. A view that lists "all my notes with `status: done`" is management. The first is the brand; the second is filler.

### 5.3 Living Links + Cognitive Engine + CNS + CECE Measurements as Queryable Dimensions
No other PKM treats links as typed entities with confidence, weight, traversal count, and lifecycle stage. No other PKM measures a note's intellectual altitude, developmental maturity, formalization stage, or structural flags. No other PKM identifies graph-detected communities, top bridges, or structural Blind Spots. **No other PKM classifies notes on the Source × Content-type epistemic axes drawn from five civilizations' epistemologies.** Constellation does all four. The Living Link Architecture (§6.1), Cognitive Engine measurement set (§6.10), CNS network analysis (§6.11), and CECE epistemic classification (§6.12) together form Bases' decisive leverage advantage. Queries that filter by `link.confidence > established`, sort by `note.stratum DESC`, surface `note.cns.is_top_bridge AND note.review.is_due`, or filter by `note.cece.content_type = hypothesis AND note.cece.source = inference` are structurally impossible in Obsidian Bases, Notion databases, Tana search nodes, or Anytype sets. They are native here.

Bases is the surface that makes these four measurement layers operable for everyday queries.

### 5.4 Write-Time Derivation (CE Rule 8) — with one Acknowledged Wrinkle
Every Bases query reads from derived state maintained at write time — for per-note dimensions. No live filesystem scan, no on-demand frontmatter parse on a 10,000-note universe.

**The acknowledged wrinkle:** CNS measurements (community detection, modularity, centrality, top-bridge identification) are graph-global, not per-note-cheap. Three candidate freshness strategies are documented at §6.11 and §13 row 9; the choice is locked at the Phase 2.6 Architect doc.

**No wrinkle for CECE:** Cataloger classifications are persisted in `sources_suggestions` (engine proposals) and frontmatter / `note_meta.properties_json` (user-approved canonical state). User-triggered scans materialize them; Bases reads them at any time without any freshness anxiety.

### 5.5 Language-First
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one. CECE's `المُصنِّف` Arabic surface, CNS's `الجهاز العصبي`, the Five Acts cognitive vocabulary — all render correctly without per-feature engineering. This is not an enhancement; it is the day-one design constraint.

---

## 6. The Constellation-Specific Leverage Points

Here is what is possible in a Constellation Base that is structurally impossible in any other PKM tool.

### 6.1 Living Link Columns and Filters (full surface, locked v1.1)

Per Eisa's lock on §13 #2 (v1.1, 2026-05-25), **all** Living Link dimensions are queryable as columns / filters / sorts:

- **Eight link properties:** `link.type` / `link.direction` / `link.annotation` / `link.weight` / `link.confidence` / `link.created` / `link.last_traversed` / `link.traversal_count`.
- **Lifecycle stage:** `link.lifecycle` — Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival.
- **Aggregated note-level:** `note.inbound_link_count_by_type`, `note.outbound_link_count_by_type`, `note.total_inbound_link_weight`, `note.inbound_confidence_distribution`, `note.dormant_link_count`.
- **Relational:** `note.is_supersedes_of`, `note.is_superseded_by`, `note.contradicts_any_established`.

These queries are impossible in any other PKM because no other PKM tracks these dimensions.

### 6.2 NSC Summary Headlines — Unconditional Default (locked v1.1)

Per Eisa's lock on §13 #1 (v1.1, 2026-05-25), every Bases view shows the NSC headline unconditionally as a default column. Context-aware rendering: table = sub-line, card = main body, list = inline. The user always knows "what this note is about" before clicking. No other PKM has machine-generated note headlines available to its database views.

### 6.3 Semantic Similarity as a Column Type

Embeddings exist (`embeddings.rs`, ONNX `multilingual-e5-small`, 384-dim, 100 languages). A Base column can render "similarity to this seed note" as a sortable number.

### 6.4 The Cataloger (CECE) Classifications — see §6.12

*Placeholder section retained for backward reference. The full CECE Bridge treatment is at §6.12 below. CECE classifications are queryable as a first-class measurement axis parallel to §6.10 (360.3D / CE dimensions) and §6.11 (CNS network measurements).*

### 6.5 Index Term Columns

The Index panel's term extraction (`notes_vocab` FTS5 dictionary) feeds Bases columns: top-N terms per note, filtering by term-with-bridge-to-lemma.

### 6.6 Federation Across cUniverses — Auto by Default (locked v1.1)

Per Eisa's lock on §13 #5 (v1.1, 2026-05-25), federation is automatic. Every Base spans cUniverses; `selectedLibraries` is the opt-out channel (renamed from the legacy `selectedVaults` per the MIG-054 alignment); federated rows carry a visible UI marker.

### 6.7 Search-Hybrid Filtering

FTS5 + structured + semantic + hybrid search modes are already wired through SearchHub. A Base filter can be expressed as a search query.

### 6.8 Cloud AI (later phase) for NL → Query

The cloud AI bridge (`ai/mod.rs`) can later expose *"describe the view you want"* → the model produces the `.base` YAML.

### 6.9 The Living Link Cell-Edit (Phase 7, locked v1.1)

A Base cell editing a *typed link* rather than just a string property. Locked to Phase 7.

### 6.10 Cognitive Engine Dimensions as Bases Columns — the 360.3D Bridge (locked v1.2)

Per Eisa's lock on §13 #6 (v1.2, 2026-05-25), **every cognitive measurement displayed by the 360.3D Inspector becomes a first-class queryable column in Bases.**

**The architectural fact that makes this cheap:** 360.3D does not compute new metrics. It displays measurements already taken by the Cognitive Engine — `strata.rs`, `review.rs`, `trails.rs`, `note_links`, etc. The data layer already carries everything; Bases just needs to expose it.

| 360.3D dimension | Bases column |
|---|---|
| **Stratum** (L1 Datum → L8 Worldview) | `note.stratum` |
| **Maturity** (Seed / Sapling / Evergreen / Canonical / Wilting) | `note.maturity` |
| **Stage** (Spark / Birth / Growth / Maturity / Dormancy / Archival — the Living Link lifecycle vocabulary) | `note.stage` |
| **Provenance** (Received / Discovered / Mixed + trust depth) | `note.provenance.origin`, `note.provenance.trust_depth` |
| **Per-type connection counts** | `note.connections.{supports, contradicts, causes, derives_from, generalizes, exemplifies, part_of, untyped, total}` |
| **Connection stratification** | `note.connection_strata` |
| **Review pulse** | `note.review.last_reviewed`, `note.review.is_due` |
| **Trail / lens membership** | `note.trails_count` |
| **Structural flags** | `note.is_orphan`, `note.is_fragile`, `note.blind_spots_count`, `note.tensions_count` |
| **Word count** | `note.word_count` |

Sample queries: *"All my L7 Paradigm notes with zero `contradicts` links"* (Echo-at-high-stratum, universe-wide); *"Notes with the Hollow-Middle shape"* (methodological-soundness audit); *"L4 Concept notes with strong inbound from L8 Worldview"* (well-grounded concepts).

### 6.11 CNS Measurements as Bases Columns — the Network Bridge (locked v1.3)

Per Eisa's lock during the CNS familiarization pass (v1.3, 2026-05-25), every measurement CNS surfaces about a note becomes a first-class queryable column in Bases.

| CNS measurement | Bases column |
|---|---|
| **Community membership** | `note.cns.community_id`, `note.cns.community_name` |
| **Centrality rank** | `note.cns.centrality_rank` |
| **Top bridge flag** | `note.cns.is_top_bridge` |
| **Bridge breadth** | `note.cns.bridge_count` |
| **Load-bearing flag** | `note.cns.is_load_bearing` |
| **Blind Spot participation** | `note.cns.blind_spot_count` |

Sample queries: *"All my Top Bridge notes ordered by traversal weight"*; *"Notes in Community {X} ordered by stratum"* (combines §6.10 + §6.11); *"Top Bridges that haven't been traversed in 60 days"* (combines §6.1 lifecycle + §6.11).

**The architectural wrinkle (acknowledged here, resolved at Phase 2.6):** CNS measurements are graph-global. Community detection, modularity, centrality — these depend on the *whole* link graph. The freshness strategy is locked at the Phase 2.6 Architect doc; three candidates (α debounced / β CNS-open-cached / γ scheduled) are on the table.

### 6.12 CECE Measurements as Bases Columns — the Epistemic Bridge (locked v1.4)

Per Eisa's lock during the Cataloger familiarization pass (v1.4, 2026-05-25), **every measurement CECE surfaces about a note becomes a first-class queryable column in Bases.** CECE (Constellation Epistemic Content Engine) ships as **"The Cataloger"** in the user-facing vocabulary (Arabic: **المُصنِّف**) — a Core Plug-in promoted from subsystem to dock-mounted feature on 2026-05-19, the same day Sight + Map were disabled.

The §6.10 360.3D Bridge exposes per-note **cognitive standing**. The §6.11 CNS Bridge exposes per-note **network position**. The §6.12 CECE Bridge exposes per-note **epistemic classification** — answering two questions CECE alone asks: *what kind of knowledge is this?* (Content-type axis) and *where did it come from?* (Source axis).

**The architectural foundation (from CECE Concept Paper §3):** CECE's two axes are abstracted from a comparative survey of five civilizations' epistemologies — Greek/European, Arabic-Islamic, Indian *pramāṇa*, Chinese Mohist/Confucian, and Persian Illuminationist thought. The Source axis names the convergent loci of *how* knowledge arrives (perception / inference / testimony / mass-transmission / comparison / postulation / non-apprehension / memory / innate-disposition / inspiration / revelation — 11 parents, 41 leaves). The Content-type axis names *what kind* of cognitive object the note is (sensory-inputs / symbolic-entities / semantic-contents / epistemic-states / higher-order-constructs — 5 branches, ~218 sub-nodes, max depth 4).

**The architectural fact that makes this cheap:** CECE does not compute classifications at read time. The 5-cataloger heuristic ensemble (User-Authority / Structural / Linguistic / Graph / Semantic — the 6th "Reasoning" cataloger is **out of scope post-MIG-046/047/048 revert**) runs during user-triggered scans, and results are **persisted twice**: as engine proposals in the `sources_suggestions` SQL table, and (on user approval) as canonical state in note frontmatter (`sources:` / `content_type:`) mirrored to `note_meta.properties_json`. Bases reads both via cheap SQL JOIN. **No freshness wrinkle** unlike the CNS Bridge — classifications are stored, never recomputed on Bases read.

**The full surface:**

| CECE measurement | Bases column | Source |
|---|---|---|
| **Approved Source (primary)** | `note.cece.source.primary` | `note_meta.properties_json` mirror of frontmatter `sources:` |
| **Approved Source (secondary)** | `note.cece.source.secondary` | same |
| **Approved Content-type (primary)** | `note.cece.content_type.primary` | mirror of frontmatter `content_type:` |
| **Approved Content-type (secondary)** | `note.cece.content_type.secondary` | same |
| **Suggestion regime — Source axis** | `note.cece.source.regime` (unanimous / strong_majority / split) | `sources_suggestions` table |
| **Suggestion regime — Content-type axis** | `note.cece.content_type.regime` | same |
| **Needs disambiguation — Source axis** | `note.cece.source.needs_disambiguation` (boolean) | derived from regime = split |
| **Needs disambiguation — Content-type axis** | `note.cece.content_type.needs_disambiguation` | same |
| **Disambiguation candidates** | `note.cece.{axis}.candidates` (list when in Split regime) | `sources_suggestions.needs_user_disambiguation_between` |
| **Has pending suggestion** | `note.cece.has_pending_suggestion` (boolean) | row exists in `sources_suggestions` without approval |
| **Last classified** | `note.cece.last_scanned` (timestamp) | `sources_suggestions.updated_at` |
| **Classified by** | `note.cece.classified_by` (list of cataloger names) | `sources_suggestions` per-cataloger trail |

**Sample queries the full surface enables:**

- *"All my Argument-type notes from Testimony sources"* — find rhetorical arguments based on what was said. Combines content_type filter + source filter.
- *"Notes with Split regime on the Source axis, ordered by inbound link weight"* — the source-disambiguation queue, triaged by importance.
- *"Hypothesis-type notes from Inference sources, sorted by stratum"* — pure analytic hypotheses, ordered by intellectual altitude. (Combines §6.12 + §6.10.)
- *"Notes classified by User-Authority only"* — your hand-declared knowledge (highest trust).
- *"L7 Paradigm notes from Tradition sources that are Top Bridges"* — combining §6.10 stratum + §6.12 source + §6.11 bridge role. **Three measurement axes in one query — only possible in Constellation.**
- *"Notes with pending Source-Review suggestions involving the active note's community"* — combines §6.11 community + §6.12 pending state. The classification queue, scoped to the cluster the user is currently focused on.
- *"Sensory-input notes from Perception sources"* — first-person observation notes.
- *"Higher-order-construct notes (theories, laws, wisdom) from Inspiration / Revelation sources"* — visionary or revelatory claims, surfaced for scrutiny.

**The Reasoning Cataloger is OUT OF SCOPE:** v1.4 explicitly excludes the designed-but-not-wired Reasoning cataloger. It depended on the local-LLM stack reverted in MIG-046/047/048. The 5-cataloger ensemble is what ships; Bases queries the live 5-cataloger output. If Mind ever returns in a different form (GPU-accelerated, cloud-routed, or different role), Phase 2.7 may be revised to add Reasoning-cataloger columns; until then, those columns don't exist.

**The threading principle (mirrors §6.10 + §6.11):** Bases provides the WHICH (which notes share this epistemic classification?); The Cataloger provides the WHY (for THIS one note, what's its full per-cataloger reasoning trail and disambiguation chips?). The user surveys in Bases, then opens The Cataloger for the rows that interest them — particularly the disambiguation queue. The "Open in The Cataloger" gesture (§7.4) is the bridge.

**The Cataloger's epistemic humility carries through:** CECE refuses to assign on Split. Bases preserves this — Notes in Split regime show their classification as "(disambiguating)" in the relevant cell, not as a guessed value. The refusal is data; Bases displays the refusal honestly.

---

## 7. The Host-Note / Assemblage Mode — and the Three Threading Gestures (Phase 1.5)

Per Eisa's locks on §13 #4 (v1.1), §13 #6 (v1.2), §13 row 9 (v1.3), and §13 row 10 (v1.4), **Phase 1.5 ships four things together**: the host-note assemblage mode + the Open-in-360.3D gesture + the Open-in-CNS gesture + the Open-in-Cataloger gesture. Together they elevate Phase 1.5 from a single feature ship to a small constellation of capabilities that compose.

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

**At Phase 1.5 the inline filter surface is the v1 set** — `is`, `is_not`, `contains`, `gt`, `lt`, `is_empty`, `is_not_empty`, plus the unconditional NSC headline column. Living Link filters arrive in Phase 2; Cognitive Engine dimensions arrive in Phase 2.5; CNS network measurements arrive in Phase 2.6; CECE epistemic classifications arrive in Phase 2.7 — which is when host-note assemblage really comes alive.

The fully-realized end-state, by way of preview, is what the paper aspires to reach by Phase 2.7:

```markdown
## Productive contradictions (Phase 2 surface)
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

## Synthesis points in the project (Phase 2.6 surface)
\`\`\`base
filter: tag=aristotle AND note.cns.is_top_bridge=true
view: cards
columns: [name, headline, cns.community_name, cns.bridge_count]
\`\`\`

## Argument-type claims from Tradition sources (Phase 2.7 surface)
\`\`\`base
filter: tag=aristotle AND note.cece.content_type=argument AND note.cece.source=tradition
view: list
columns: [name, headline, cece.content_type.primary, cece.source.primary]
\`\`\`

## Notes pending disambiguation in this project (Phase 2.7 surface)
\`\`\`base
filter: tag=aristotle AND note.cece.has_pending_suggestion=true
view: table
columns: [name, headline, cece.source.regime, cece.content_type.regime]
\`\`\`
```

Each view above is asking a *Constellation-specific cognitive question*. The host note holds the prose; the views hold the slices.

### 7.2 The "Open in 360.3D" row gesture (locked v1.2)

Every Bases row carries an affordance that opens the **full 360.3D Inspector for that note**. One click → the user reads the standing in depth (Position / Connection Profile / Absence per the Reading Guide).

### 7.3 The "Open in CNS" row gesture (locked v1.3)

Every Bases row also carries an affordance that opens **CNS centered on that note** — gravity well laid out, community highlighted, network neighborhood visible.

### 7.4 The "Open in The Cataloger" row gesture (locked v1.4)

Every Bases row also carries an affordance that opens **The Cataloger's Source Review panel scoped to that note**. The exact UI is an architecture-phase detail (icon, context-menu action, or keyboard shortcut — to be locked in the Phase 1.5 Architect doc). The principle: **one click from any Bases row opens the per-note classification card** showing:

- The note's Source × Content-type classification (primary + optional secondary on both axes).
- The per-cataloger reasoning trail (which cataloger said what, with confidence).
- Disambiguation chips if the note is in Split regime on either axis.
- Accept / Reject / Edit actions.
- The trust-calibration banner (for the first 50 reviews per Library).

This is the third navigation glue — between *surveying* and *epistemic-classification reading*:

1. The user surveys a collection in Bases — say, all Hypothesis-type notes from Inference sources.
2. They spot a row whose classification seems wrong, or that's flagged with `has_pending_suggestion = true`.
3. One click → The Cataloger opens with that note's card centered, showing the engine's reasoning.
4. The user can accept, reject, edit, or leave it pending; The Cataloger's preview-vs-open semantics are preserved.
5. Returning to the Bases view, the user's query state is preserved.

### 7.5 Three threading gestures, one surveying surface — the four-surface workflow (updated v1.4)

After Phase 1.5 ships, every Bases row carries **three threading affordances**, all lightweight, all leading to deep-read surfaces:

| Gesture | Routes to | What it shows |
|---|---|---|
| Open in **360.3D** | Per-note cognitive standing surface | Stratification Matrix — Position / Connection Profile / Absence |
| Open in **CNS** | Per-note network neighborhood surface | Community, centrality, top-bridge role, blind-spot suggestions |
| Open in **The Cataloger** | Per-note epistemic classification surface | Source × Content-type card, per-cataloger reasoning trail, disambiguation chips |

The user surveys in Bases and chooses *which depth* to drop into for any given row. All three deep-read surfaces remain standalone — they work fine from any open note. Bases just makes the routing one-click from a comparative context.

This is the **four-surface workflow** that distinguishes Constellation:
- **Bases** = comparison of many notes (the surveying)
- **360.3D** = cognitive standing of one note (the cognitive depth)
- **CNS** = network position of one note (the network depth)
- **The Cataloger** = epistemic classification of one note (the epistemic depth)

**No other PKM has all four. No other PKM threads them together with single-click row gestures.**

### 7.6 What makes this Constellation's, not Notion's

- **The host note is sacred.** Plain `.md` file. The Bases are inline `.base` code blocks. No proprietary container.
- **Each view leverages Constellation dimensions** (once Phase 2 / 2.5 / 2.6 / 2.7 ship the leverage points).
- **Live, write-time-maintained** for per-note dimensions and CECE classifications; **freshness-strategy-determined** for CNS network measurements (§6.11).
- **Three clicks to depth.** Any row threads to 360.3D OR CNS OR The Cataloger without leaving the construction page.

### 7.7 The "Knowledge Construction Page" pattern

The host-note assemblage is not just embedding views; it is a **named UX pattern** Constellation can teach: the *Knowledge Construction Page*. A user creates a host note for any project, area, or inquiry, and embeds the views that reveal what they need to see. The note holds the prose; the views hold the slices; the three row gestures thread to 360.3D (cognitive), CNS (network), or The Cataloger (epistemic) for one-note depth in any of three directions.

### 7.8 Why accelerate to Phase 1.5

Three reasons argued for the acceleration:
1. **It's the dominant user-facing pattern across the entire PKM market.**
2. **It composes with the Rule 8 migration.**
3. **It teaches the system's affordances by example.** A user who opens a host note with three embedded views immediately understands what a Bases view is for. With the three threading gestures shipping in 1.5, the user immediately learns that Bases is the *survey-and-thread* surface that ties Constellation's four measurement surfaces together as one workflow.

---

## 8. The Five Acts as Operational Templates (locked v1.1)

The Five Acts of Knowledge Creation are Constellation's cognitive model. Bases makes them operable.

| Act | Bases Template | What it Surfaces |
|---|---|---|
| **Observation** | "Recent Captures" | Last 14 days, sorted by creation date, NSC headlines visible. The intake queue. |
| **Connection** | "Single-Direction Conduits" | Notes with high outbound link counts but low inbound. Awaiting reciprocation. |
| **Tension** | "Productive Frictions" | Notes connected by `contradicts`-typed links. The forge of synthesis. |
| **Synthesis** | "Convergence Points" | Notes with high inbound link weight from multiple `established`-confidence sources. |
| **Conviction** | "Load-Bearing Work" | Notes with `confidence: established` AND high traversal count. |

**Distribution model — both shapes** (locked v1.1, §13 #3): read-only system Bases at `{universe}/.constellation/bases/system/five-acts/` + editable user copies on duplicate via "Customize" gesture with `derivedFrom` lineage marker.

---

## 9. What Sets a Constellation Base Apart

Six lines of differentiation, in honest priority order:

1. **Living Links as query dimensions.** Filter by confidence, sort by weight, group by typed link. Structurally impossible in any other PKM.
2. **Summary headlines visible by default, context-aware rendering.** The NSC differentiator that makes the dashboard effect (§1) work.
3. **Federation across universes — auto by default.** The long-tail PKM dream nobody has shipped, made the *default*.
4. **Cognitive Engine measurements queryable across the collection** (the 360.3D Bridge, added v1.2). Stratum, Maturity, Stage, Provenance, structural flags, review pulse. **The first PKM to make "intellectual altitude" and "developmental shape" queryable.**
5. **Network topology queryable across the collection** (the CNS Bridge, added v1.3). Communities, top-bridges, blind-spots, load-bearing flags. **The first PKM to make "synthesis points" and "structural gaps" filterable across the note collection.**
6. **Epistemic classification queryable across the collection** (the CECE Bridge, added v1.4). Source × Content-type measurements drawn from five civilizations' epistemologies. **The first PKM to make "where this knowledge came from" and "what kind of knowledge this is" filterable across the note collection.** Notes carry their epistemic texture as first-class queryable data — alongside the engine's classification regime (unanimous / strong_majority / split) and the per-cataloger reasoning trail.

A Constellation Base that does not deliver these six is not a Constellation Base — it is Obsidian Bases running on a different runtime. The point of building Bases *of* Constellation rather than *into* Constellation is that the architecture *gives* us these six. Refusing to use them is the violation.

---

## 10. Architectural Mandates

Ten mandates, derived from the principles. The architecture phase will refine; these set the boundary.

### 10.1 Write-Time Derivation (per-note dimensions)
A `bases_cache` SQLite table (or equivalent — `note_meta.properties_json` already serves this role; see MIG-054 Architect doc §4.B) is maintained by triggers on the upstream write path. `query_base` becomes a cheap SQL lookup for per-note dimensions.

### 10.2 File-Over-App
`.base` files remain plain YAML on disk. The cache is internal optimization; the source of truth is the file.

### 10.3 Instant on 10k
Every Base query, including federated and Living-Link-filtered and Cognitive-Engine-filtered and CECE-filtered, returns in under 50ms on a 7,600-note Universe. **For CNS-filtered queries, "instant" is bounded by the freshness strategy chosen at Phase 2.6** — the read is instant; the underlying CNS analysis may be debounced, cached, or scheduled.

### 10.4 Multilingual Native
Every operator name, every column header rendering, every error message — all 15 locales, bidirectional, day one.

### 10.5 Embedded Bases Are First-Class
An inline ` ```base ` block in a host note has the same capability surface as a workspace-level `.base`.

### 10.6 Federation Is Default-On (added v1.1)
Bases queries auto-span cUniverses. The user does not opt in. `selectedLibraries` is the explicit opt-out channel.

### 10.7 Constellation Wings Integration Is Bidirectional (added v1.1)
When Wings ships, the Bases ↔ Wings contract is bidirectional: Bases exposes data to Wings AND consumes from Wings (external plugins can register column types).

### 10.8 360.3D Bridge — Bidirectional in Data, Light in UI (added v1.2)
- Bases consumes from the Cognitive Engine.
- Bases threads to 360.3D (Open-in-360.3D row gesture).
- 360.3D need not know about Bases — it operates standalone.

### 10.9 CNS Bridge — Bidirectional in Data, Light in UI (added v1.3)
- Bases consumes from CNS.
- Bases threads to CNS (Open-in-CNS row gesture).
- CNS need not know about Bases.
- **Freshness strategy deferred to Phase 2.6 Architect doc** (α / β / γ from §6.11).

### 10.10 CECE Bridge — Bidirectional in Data, Light in UI (added v1.4)
- **Bases consumes from CECE.** All measurements in §6.12 are read from `sources_suggestions` (engine proposals) and `note_meta.properties_json` (user-approved canonical state via frontmatter mirror). Both are already write-time-derived; no new computation in Bases.
- **Bases threads to The Cataloger.** The Open-in-Cataloger row gesture (§7.4) is the navigation glue. Bases sets the comparative context; The Cataloger delivers the per-cataloger reasoning trail for any chosen row.
- **The Cataloger need not know about Bases.** The Source Review panel continues to work as a standalone surface from the right sidebar tab and the left-dock CatalogerView. The bridge is one-way at the navigation layer — Bases routes into The Cataloger, not the reverse.
- **No freshness wrinkle** — unlike CNS, CECE classifications are persisted on disk (frontmatter) and in SQL (`sources_suggestions`). Bases reads at any time without staleness anxiety. Scans are manual; the user controls when re-classification happens.
- **CECE's epistemic humility is preserved.** Notes in Split regime show "(disambiguating)" in the relevant Bases cell, not a guessed value. The refusal is data; Bases displays the refusal honestly.

---

## 11. Out of Scope (v1)

- **Pre-built vertical templates** (CRM / recipe manager / habit tracker / etc.). Five Acts templates ship; no others.
- **Aggregation formulas** beyond what's needed for the Five Acts. Later phase.
- **Calendar / timeline / board / gallery views.** v1 ships table + card + list.
- **Cloud AI NL → query.** Architecture admits it; v1 doesn't ship it.
- **Generative lens suggestions.** Research mode, not v1.
- **Real-time multi-user collaboration in a Base.** Local-first; collaboration is via Git / Syncthing / iCloud.
- **Bases-from-external-data** (Notion's "Connections" / Coda's "Packs"). Wings' responsibility when Wings ships.
- **Bases-driven 360.3D filtering.** Reverse direction; possibly Phase 8+.
- **Bases-driven CNS filtering** (added v1.3). Reverse direction; possibly Phase 8+.
- **Bases-driven Cataloger filtering** (added v1.4). Reverse direction — a Bases query feeding into a scoped Source Review queue. Not v1; possibly Phase 8+.
- **CECE Reasoning Cataloger columns** (added v1.4). The 6th cataloger is designed but un-wired; depends on the reverted local-LLM stack (MIG-046/047/048). v1 ships with the 5-cataloger heuristic ensemble. If Mind ever returns in a different form, Phase 2.7 may be revisited to add Reasoning-cataloger columns.

---

## 12. Roadmap (Provisional — Updated v1.4)

Sequencing, not commitments. Each phase is its own `/migration` workflow.

- **Phase 0 — Concept** (this paper) ✓
- **Phase 1 — Rule 8 Migration (MIG-054).** Cheap-lookup `query_base` via SQL against `note_meta.properties_json`. The architectural foundation.
- **Phase 1.5 — Host-Note Assemblage + Three Threading Gestures** (Open-in-360.3D + Open-in-CNS + Open-in-Cataloger). Inline ` ```base ` blocks render in any host note; every row carries three threading affordances.
- **Phase 2 — Living Link Columns.** §6.1 surface user-facing.
- **Phase 2.5 — Cognitive Engine Dimensions (the 360.3D Bridge).** §6.10 surface user-facing.
- **Phase 2.6 — CNS Network Measurements (the CNS Bridge).** §6.11 surface user-facing. **Includes the freshness-strategy decision** (α / β / γ).
- **Phase 2.7 — CECE Epistemic Classifications (the Cataloger Bridge).** §6.12 surface user-facing. The 5-cataloger output queryable; no freshness wrinkle.
- **Phase 3 — NSC Headlines as Default Column.** Headlines visible in every Bases view by default; context-aware rendering.
- **Phase 4 — Federation Auto-On.** `.base` queries automatically span cUniverses.
- **Phase 5 — Five Acts Templates.** §8 named templates ship as both read-only system Bases and editable user copies.
- **Phase 6 — Semantic + Index Columns.** §6.3 + §6.5 + §6.7 leverage points wired in.
- **Phase 7 — Cell-Edit on Typed Links.** §6.9. The relationship-editor mode.
- **Phase 8+** — NL → query; generative lens suggestions; alternative renderers (likely via Wings); Bases-driven 360.3D / CNS / Cataloger filtering.

---

## 13. Decisions Locked 2026-05-25

All ten design questions are resolved as of this session.

| # | Question | Resolution | Where folded |
|---|---|---|---|
| 1 | Headlines unconditional or opt-in? | **Unconditional default.** Context-aware rendering. | §6.2 |
| 2 | `.base` schema extension for Living Links — which dimensions? | **All.** | §6.1 |
| 3 | Five Acts templates — read-only / editable / both? | **Both.** | §8 |
| 4 | Host-Note Assemblage Mode acceleration? | **Accelerate to Phase 1.5.** | §7, §12 |
| 5 | Federation default behavior? | **Auto.** `selectedLibraries` is opt-OUT. | §6.6, §10.6 |
| 6 | Bases ↔ 360.3D relationship? | **All 10 CE dimensions as columns + Open-in-360.3D row gesture.** | §6.10, §7.2, §9.4, §10.8, §12 |
| 7 | Constellation Wings integration? | **Both directions.** | §10.7 |
| 8 | Cell-edit on typed links phase? | **Phase 7.** | §6.9, §12 |
| 9 | Bases ↔ CNS relationship? | **All 6 CNS measurements as columns + Open-in-CNS row gesture; freshness deferred to Phase 2.6 Architect doc.** | §6.11, §7.3, §9.5, §10.9, §11, §12 |
| **10** | **Bases ↔ The Cataloger (CECE) relationship?** (raised v1.4) | **All CECE measurements (approved Source × Content-type from frontmatter mirror + suggestion regime + disambiguation state from `sources_suggestions`) as Bases columns at Phase 2.7. Open-in-Cataloger row gesture at Phase 1.5. Reasoning cataloger OUT OF SCOPE (depends on reverted Mind stack). No freshness wrinkle — classifications are persisted.** | §6.12, §7.4, §9.6, §10.10, §11, §12 |

With all ten closed, this concept paper enters service as the durable guiding light for the design phase. **Phase 1 (MIG-054 Rule 8 migration) architecture is ready; the Architect doc is at `docs/MIG-054-bases-rule8-migration-ARCHITECT.md`.**

---

## 14. Predecessor and Adjacent Documents

- **Predecessor (this Concept Paper line):**
  - v1.0 — pre-decisions draft.
  - v1.1 — 7 of 8 closed.
  - v1.2 — all 8 + 360.3D bridge.
  - v1.3 — CNS bridge added.
  - All retained as historical record.
- **Predecessor (MVP):** `docs/BASES_MVP_SPEC.md` (commit `c5b05f5c`, 2026-03-12). **This Concept Paper does not invalidate the MVP; it articulates the destination.**
- **Successor (Phase 1):** `docs/MIG-054-bases-rule8-migration-ARCHITECT.md` (in this same commit set).
- **Adjacent — founding mission:** `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`.
- **Adjacent — service consumed:** `docs/Constellation-NSC-Concept-Paper-v2.0.md` (Bases is a downstream consumer of NSC).
- **Adjacent — companion cognitive surface:** `docs/360.3D-Concept-Paper-v1.0.md`, `docs/360.3D-Matrix-Reading-Guide-v1.0.md`.
- **Adjacent — companion network surface:** `docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md`. CNS lives at `src-tauri/src/sight.rs` (the file name preserves the v2 lineage; CNS is *not* the Sight subsystem disabled per MIG-038).
- **Adjacent — companion epistemic surface (added v1.4):** `docs/Constellation-CECE-Concept-Paper-v1.0.md` (the canonical CECE / Cataloger design doc — explains the cross-civilizational foundation, the 6-cataloger architecture, the Source Review workflow, the §10 naming decision). `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` (the unified user-facing CE help doc — includes the Cataloger / Source Review walkthrough). CECE lives at `src-tauri/src/cece/` (one module per cataloger plus `synthesis.rs` orchestrator + `wiring.rs` Tauri IPC bindings).
- **Adjacent — current state record:** `docs/Constellation Orientation & Onboarding v2.35.md` §4.5 (current Cataloger subsystem record).
- **Explicitly NOT in scope:** the disabled Sight subsystem (MIG-038, 2026-05-19); the reverted Constellation Mind subsystem (MIG-046/047/048); the unwired CECE Reasoning cataloger (depends on the reverted Mind stack).

---

## 15. Closing — The Guiding Light

When in doubt during the design phase, return to one question:

> **Does this make the user re-encounter their own thinking — or invite them to re-organize it?**

The first is knowledge formulation. The second is productivity theater. Constellation ships the first.

This paper exists to keep that distinction visible at every step from here to release.

---

*End of Concept Paper v1.4. With all ten decisions resolved, this version enters service as the durable guiding light. To be updated only on substantive change of vision.*

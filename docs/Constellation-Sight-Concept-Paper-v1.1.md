# Constellation Sight — Concept Paper

**Version 1.1 (markdown port + truth-status refresh) | 2026-05-07**
**Original**: *Constellation_Lens_Concept_Paper_Eisa.pdf* (Eisa, April 2026, v1.0 internal working document)
**Author**: Eisa Alshamsi · eisa@uconstellation.world
**Audience**: every future Claude session, the Boss reviewing where Sight stands, any future contributor.

> **What changed in v1.1** (Boss-directed 2026-05-07):
> 1. **Lens → Sight rename** end-to-end. The user-facing surface is now called *Constellation Sight*; the v1.0 paper used "Lens" before the rename. Internal Rust names (`lens.rs`, `lenses.rs`, `apply_lens`, IPC names like `constellation_sight_centrality`) are not renamed in this port — they remain as on disk.
> 2. **Truth-status discipline**. A new §0 disclaimer ("What this paper IS") and §12 truth-status matrix ("what's shipped vs. what the paper claims"). v1.0 was a *forward-looking design proposal*; this v1.1 honours the design while being explicit about what made it through to running code and what didn't.
> 3. **Principle 6 added: reveal-on-demand**. Edges hide until the user hovers a node (or runs a search, or selects). The pattern that lets Sight feel instant on a 30,000-edge universe. Shipped in MIG-016 §1B.
> 4. **Three implementation gaps tracked**. The mechanics most central to InfraNodus — *content similarity*, *layer peeling*, *Map↔Sight integration* — are not yet in v2 Sight. Each is allocated a stable Pending Job number: **PJ-035** (content similarity TF-IDF edges), **PJ-036** (layer peeling), **PJ-037** (Map↔Sight integration).
> 5. **Star-chart vision documented as design north star** (§13). The reference image is a 19th-century-style printed chart of the northern-hemisphere stars (Suwaidi reference; sample owned by the Boss). The visual metaphor — magnitude → centrality, constellation territories → communities, Milky Way → density, rim calendar → time — is the target shape of v3 Sight.
> 6. **v3 redesign noted (§14)**. **PJ-038 — Sight v3 build** is allocated. v2 Sight will be disabled (whole package: dock button, modal, Settings entry) as a known-good fallback under MIG-017; v3 will be built fresh with its **own dedicated Concept Paper** that supersedes this one for v3 specifics.

---

## §0 · What this paper IS (and is NOT)

This paper is a **design document and intent record**. It articulates what Sight *should be* — the analytical paradigm, the user value proposition, the principles, the edge types, the metrics, the use cases. It is the cornerstone for every future Sight conversation.

This paper is **not a status report**. The v1.0 PDF was written in April 2026 as a forward-looking proposal, before any Sight code shipped. Several mechanics described here (content similarity edges, layer peeling, Map↔Sight integration) are **not in the running app**. §12 is the honest scoreboard; the rest of the paper describes the target.

This paper is **not a v3 specification**. v2 Sight (`ConstellationSight2.svelte` + the `lens_*` Rust modules) is being shelved as a known-good fallback while v3 is built from scratch on the star-chart aesthetic (§13–§14). v3 will have its own dedicated Concept Paper. **Read that paper for v3 specifics; read this one for the analytical foundations both share.**

---

## §1 · Executive Summary

Constellation Sight is a text-network-analysis engine that applies network science algorithms to a user's knowledge base, revealing topical clusters, conceptual bridge notes, structural gaps, and cognitive diversity metrics. It draws direct inspiration from **InfraNodus** — a peer-reviewed text-network-analysis tool by Dmitry Paranyushkin / Nodus Labs (WWW'19) that uses graph theory to generate insight from any discourse.

Where InfraNodus analyzes individual texts or external sources (Google results, tweets, PDFs) through a cloud-based web app, Sight reimagines the same analytical principles for a local-first PKM context: operating across an entire universe of interconnected notes, computed in Rust on the user's machine, integrated with Constellation's existing graph infrastructure.

### Core proposition

Sight transforms Constellation's note graph from a passive relationship visualization into an **active analytical engine** that:

- Identifies topical communities,
- Ranks notes by bridging importance,
- Detects structural blind spots,
- Measures the cognitive diversity of a knowledge base,

— answering the question: ***"What patterns and gaps exist in my thinking?"***

This paper is a companion to the **Constellation Map Concept Paper** (April 2026), which proposed a radial knowledge visualization inspired by Goalscape. The two are complementary: the **Map** provides structural overview at the organizational level; **Sight** provides analytical intelligence at the conceptual level. Together they position Constellation as a knowledge intelligence platform unlike anything in the current PKM landscape.

---

## §2 · Source Analysis: InfraNodus

### §2.1 What InfraNodus is

InfraNodus is a text-network-analysis tool developed by Dmitry Paranyushkin and Nodus Labs (France, registered as InfraNodus SAS). First prototyped in 2011, launched commercially 2018, peer-reviewed at WWW'19 (The Web Conference). Used by researchers, marketers, writers, and organizations including Greenpeace and Procter & Gamble.

InfraNodus represents text as a network graph where words/concepts are nodes and their co-occurrences within a sliding window create weighted edges. It then applies network science algorithms — **betweenness centrality**, **community detection (Louvain)**, **Force-Atlas layout** — to extract structural insight that goes beyond standard NLP, word clouds, or LDA topic modeling.

Its stack: NestJS (Node.js), Prisma, PostgreSQL, Sigma.js (graph rendering), GPT-5 integration for AI-powered insight generation. It offers an Obsidian plugin, browser extension, MCP server, API, and n8n integration.

### §2.2 Six core analytical mechanics

| # | Mechanic | Algorithm / method | What it reveals |
|---|---|---|---|
| 1 | **Betweenness centrality** | For each node, count how many shortest paths between all other node pairs pass through it (Brandes' algorithm, O(VE)). | Concepts that serve as **bridges** between topic areas. Not necessarily the most frequent — the most structurally important connectors. |
| 2 | **Community detection** | Louvain: iteratively optimize modularity by grouping densely-connected nodes. | **Topical clusters** that emerge from actual co-occurrence patterns, not from user-defined tags or folders. |
| 3 | **Structural gap detection** | Identify pairs of communities with high internal density but low inter-community connectivity. Highlight the shortest missing bridges. | **Blind spots**: areas where two relevant clusters exist but lack connection. Where new insight is most likely to emerge. |
| 4 | **Discourse bias scoring** | Combine modularity (M > 0.4 = pronounced structure), cluster dominance (% of nodes in largest cluster), Shannon entropy (E) of top-4 node assignments. | A quantitative measure: **cognitively diverse vs. biased**. High entropy = diverse; low entropy = echo chamber. |
| 5 | **Layer peeling** | Remove top-N nodes by centrality; recalculate all metrics on the residual graph. | The **structure beneath the obvious**. Iterative peeling reveals progressive depth layers. |
| 6 | **Non-linear navigation** | Click any node → retrieve all source-text statements containing that concept and its neighbors. | The graph becomes an **interactive reading and exploration device**, not just a visualization. |

### §2.3 The structural gap insight

InfraNodus's most distinctive contribution is the **structural gap concept**. Most text-analysis tools focus on what's *present* in a text — frequent words, dominant topics, sentiment. InfraNodus inverts this by focusing on what's *absent*: the connections that don't yet exist between clusters that could meaningfully relate.

The intellectual foundation draws on Ronald Burt's theory of **structural holes** (1992): innovation arises at the boundaries between groups, not within them. InfraNodus applies this to knowledge: if a user's notes have a dense cluster on "Islamic jurisprudence" and a separate dense cluster on "systems engineering" but no notes connecting them, that's a structural gap — and a prompt to explore whether a connection exists (e.g., how systematic legal reasoning parallels requirements engineering).

---

## §3 · The Sight Concept

### §3.1 Definition

Constellation Sight is an analytical engine that applies network science algorithms to the user's universe graph, transforming the existing graph view from a passive map into an active knowledge-discovery tool. It computes betweenness centrality, detects topical communities, identifies structural gaps, and measures cognitive diversity — all locally, in Rust, with results cached and reused while fresh.

### §3.2 Semantic mapping: text analysis → universe analysis

InfraNodus operates on a **word-level co-occurrence graph** built from a single text or corpus. Sight operates on a **note-level relationship graph** built from an entire universe.

| InfraNodus dimension | InfraNodus context | Constellation Sight context |
|---|---|---|
| Nodes | Words/concepts (lemmatized) | **Notes** (each note = one node) |
| Edges | Co-occurrence in a 4-gram sliding window | **Wikilinks**, shared tags, content similarity |
| Edge weight | Proximity in sentence (closer = heavier) | Link type weight: explicit wikilink > shared tag > content similarity |
| Betweenness centrality | Bridging words | **Bridging notes** — most structurally important notes in the universe |
| Community detection | Topical word clusters | **Knowledge domains** from link/tag patterns — emergent, not folder-defined |
| Structural gaps | Discourse blind spots | **Knowledge blind spots**: areas that should connect but don't |
| Bias scoring | Cognitive diversity of a single text | **Cognitive diversity of the entire universe** — "universe health" metric |
| Layer peeling | Remove top concepts → reveal depth | Hide dominant notes (MOCs, index notes) → reveal underlying structure |
| Non-linear navigation | Click node → source excerpt | Click node → **open note in editor** |

### §3.3 Three edge types for a richer graph

InfraNodus builds its graph from a single edge type (word co-occurrence). Sight has access to a **richer signal set**:

- **Explicit wikilinks (weight 1.0)**. Manually authored by the user — deliberate connections, highest weight. The Universe Index tracks these.
- **Shared tags (weight 0.6)**. Two notes sharing a frontmatter or inline tag are implicitly related. Tag co-occurrence creates edges the user may not have considered.
- **Content similarity (weight 0.3)**. Notes with high textual similarity (TF-IDF cosine, optionally embeddings) but **no explicit link**. The most interesting edges — they reveal connections the user hasn't yet made explicit. Computed lazily and cached.

The multi-edge approach means Sight can detect structural gaps **even when the user hasn't created wikilinks** — surfacing "you wrote about similar topics in these two notes but never linked them" as an actionable suggestion.

### §3.4 Universe health: the cognitive diversity metric

Adapting InfraNodus's discourse-bias scoring, Sight computes a composite **universe health** metric:

| Component | Metric | What it measures | Healthy range |
|---|---|---|---|
| **Modularity (M)** | Louvain modularity score | How distinct the topical clusters are. M > 0.4 = well-defined communities. | 0.3 – 0.6 |
| **Dominance (D)** | % of notes in the largest community | Whether one topic overwhelms the universe. D > 50% = imbalance. | < 35% |
| **Entropy (E)** | Shannon entropy of community distribution | How evenly knowledge is distributed across topics. Higher = more diverse. | > 2.0 bits |
| **Connectivity (C)** | Average path length between communities | How well-bridged the knowledge areas are. Low = well-connected. | < 4 hops |

These combine into a single **diversity score (0–100)** displayed as a compact indicator. A universe with high modularity, low dominance, high entropy, and low inter-community path length is a cognitively diverse, well-bridged knowledge base.

---

## §4 · Design Principles

### Principle 1 — Reveal, don't prescribe

Sight surfaces patterns and gaps; it does not tell the user what to do. A structural gap is **information**, not an instruction. The user decides whether to bridge it, ignore it, or explore it later.

### Principle 2 — Compute locally, present unobtrusively

All graph analytics run in the Rust backend on the user's machine. **No data leaves the device.** Results appear as overlays and a sidebar panel within the existing graph view, not as a separate tool or modal. The user can toggle Sight on/off without leaving their workflow.

### Principle 3 — Multiple edge types, single coherent graph

Wikilinks, shared tags, and content similarity are combined into one weighted graph. The user doesn't need to understand graph theory — they see one visualization where edge thickness/opacity reflects connection strength and colors reflect auto-detected topic clusters.

### Principle 4 — Emergent structure over imposed structure

Community detection produces clusters from **actual content patterns**, not from the user's folder hierarchy. This respects the bottom-up ethos of Zettelkasten and Digital Gardening while providing the top-down overview that users crave.

### Principle 5 — Language-agnostic by design

The graph analytics layer operates on **structural relationships** (links, tags, similarity scores), not on language-specific NLP. Sight works identically for Arabic, English, or bilingual universes — a critical requirement for Constellation's RTL-first architecture. Language-specific processing (lemmatization, stopword removal) is only needed for the optional content-similarity edge, and is modularized per language.

### Principle 6 — Reveal-on-demand (added in v1.1)

> *"When Sight opens, it shall display only the nodes; if we hover over one of these nodes, it will display its links."* — Eisa, 2026-05-07.

Edges hide by default. They render only when the user signals interest: hovering a node, selecting a node, running a search, or hovering a link annotation. The hidden state is the **resting state**. This:

- Lets a 30,000-edge universe feel instant on first open (the per-frame O(E) edge-iteration cost goes to zero in the common case).
- Mirrors the way the human eye reads a star chart: you see the field of stars first, and the lines between stars only appear when you focus on a constellation.
- Is the same "nervous system" pattern Sky View uses (`graphEngine.ts:1880-1894`) — proven on real Constellation data.
- Composes cleanly with Principle 1: the user, by hovering, *invokes* the connection. Sight doesn't impose it.

This principle is not in InfraNodus (their cloud GPU can render every edge always). It is **Constellation-native** — a direct consequence of running on the user's machine and refusing to make them wait. Shipped in MIG-016 §1B (commit `62718f7`).

---

## §5 · Use Cases

### §5.1 Knowledge blind-spot discovery

A researcher studying Islamic intellectual heritage and systems engineering has dense clusters in each domain. Sight detects a structural gap and surfaces: *"These two areas are well-developed but disconnected. Consider whether maqasid al-shari'a (objectives-based reasoning) relates to requirements traceability in SE."* The researcher creates a bridging note — and a new line of inquiry emerges.

### §5.2 Bridge-note identification

A user has a note titled "Epistemology" that links to both philosophy and AI/ML notes. Betweenness centrality ranks it as the **#3 most structurally important note** in the universe, even though it has fewer links than heavily-connected index notes. The user realizes this is a critical conceptual junction and invests in developing it further.

### §5.3 Universe-health audit

A student checks their universe health: Modularity 0.7, Dominance 62%, Entropy 1.4. Sight flags this as a low-diversity universe dominated by one topic. The student uses this as motivation to explore underdeveloped areas, using the structural gaps as starting points.

### §5.4 Research-literature mapping

A PhD candidate imports 50 research-paper notes. Sight clusters them into topical communities and reveals which sub-topics are well-covered and which represent gaps in the literature review. Content-similarity edges highlight papers that discuss related concepts but aren't yet cross-referenced.

### §5.5 Layer peeling for deep exploration

A user's universe is dominated by a few heavily-linked MOC notes. They activate layer peeling, which hides the top-10 centrality nodes and recalculates. Suddenly, a **secondary structure emerges**: smaller conceptual clusters and bridge notes invisible under the MOC layer. This is analogous to *removing the obvious to reveal the subtle* — a principle that resonates with the *tawīl* (تأويل) tradition of reading beneath surface meaning.

---

## §6 · Integration Architecture (v2 — current)

### §6.1 Position within Constellation

Sight is not a separate view — it is an **analytical layer within the existing graph view**. When the user activates Sight, the graph gains overlay features: community coloring, centrality-based node sizing, gap highlighting, and a sidebar analytics panel. Sight enhances the graph; it does not replace it.

| Component | Engine | Without Sight | With Sight active |
|---|---|---|---|
| Node sizing | Pixi.js (WebGL) | Sized by degree (link count) | Sized by **betweenness centrality** (bridging importance) |
| Node coloring | Pixi.js | User-defined or uniform | Auto-colored by **detected community** |
| Edge rendering | Pixi.js | Wikilinks only | Wikilinks + shared tags + content similarity (toggleable; **content similarity not yet shipped — see §12**) |
| Sidebar | Svelte 5 | Basic note metadata | Analytics panel: top communities, top bridges, structural gaps, universe health |
| Interaction | Pixi.js events | Click → open note | Click → open + select to explore neighborhood + right-click for gap suggestions |

### §6.2 Computation pipeline

1. **Universe Index (Rust)** — maintains the note graph with wikilinks and metadata. Extended to compute shared-tag edges and (future) content-similarity edges.
2. **Graph analytics engine (Rust)** — Brandes' betweenness centrality, Louvain community detection, modularity scoring, Shannon entropy, structural gap identification. Runs asynchronously; results cached and invalidated on universe change.
3. **IPC bridge (Tauri v2)** — `constellation_sight_centrality`, `constellation_sight_communities`, `constellation_sight_structural_gaps`, `constellation_sight_universe_health`, etc. Each returns a JSON payload consumed by the frontend.
4. **Sight overlay (Svelte 5 + Pixi.js)** — applies community colors, centrality sizing, gap highlighting (dashed lines between unconnected community pairs), sidebar analytics panel.

### §6.3 Performance considerations

- **Brandes' algorithm**: O(VE). For 5,000 notes / 20,000 edges ≈ 100M operations — sub-second in Rust.
- **Louvain**: O(n log n). Trivially fast.
- **Content similarity**: the most expensive operation. Should be **incremental** — vectors only for changed/new notes, all vectors cached, similarities recomputed lazily.
- **Approximate centrality**: for universes > 50,000 notes, sampling-based centrality without significant accuracy loss (network science literature).
- **Edge-render cost**: addressed by Principle 6 (edges hidden until hover/search/select).

---

## §7 · Synergy with the Constellation Map

Sight and Map are complementary:

| Dimension | Constellation Map | Constellation Sight |
|---|---|---|
| Source inspiration | Goalscape | InfraNodus |
| Chart type | Radial sunburst (concentric rings) | Force-directed network graph (v2) / star-chart (v3 target — §13) |
| Primary question | *What is the shape and maturity of my knowledge?* | *What patterns and blind spots exist in my thinking?* |
| Analytical level | Organizational (areas, topics, folders) | Conceptual (semantic relationships, co-occurrence) |
| Node meaning | Knowledge branch (subtree of notes) | Individual note |
| Key metric: size | Knowledge weight (density, volume) | Betweenness centrality (bridging importance) |
| Key metric: color | Categorical grouping (user-defined) | Community membership (auto-detected) |
| Key metric: fill | Maturity (seed → evergreen) | N/A (not hierarchical) |
| Unique capability | Zero-sum priority allocation | **Structural gap detection** |
| Computation backend | Rust (Universe Index subtree stats) | Rust (graph analytics engine) |
| Rendering | SVG via D3.js | Canvas via Pixi.js (v2) |

The data flow is complementary: both read from the same Universe Index but compute different analytics. A user might open the **Map** to see that their "Systems Engineering" branch is underdeveloped (thin segment, low maturity), then switch to **Sight** to discover which specific notes in that area would benefit from connections to other domains (structural gaps). **The Map diagnoses; Sight prescribes.**

> **Note on Map↔Sight integration**: as of 2026-05-07, the two surfaces are independent — clicking a Map segment does *not* filter Sight to that branch, and Sight selections are not visualized on the Map. Wiring this is allocated as **PJ-037** (see §12).

---

## §8 · Competitive Differentiation

| Application | Graph type | Centrality ranking | Community detection | Structural gap detection | Universe health metric |
|---|---|---|---|---|---|
| Obsidian | Force-directed | No (degree only) | No | No | No |
| Logseq | Force-directed | No | No | No | No |
| Roam Research | Force-directed | No | No | No | No |
| Notion | None | N/A | No | No | No |
| InfraNodus | Force-directed | Yes (betweenness) | Yes (Louvain) | Yes | Yes (discourse bias) |
| InfraNodus + Obsidian plugin | Force-directed | Yes | Yes | Yes | Partial |
| **Constellation Sight (v2 shipped)** | Force-directed | Yes (betweenness) | Yes (Louvain) | Yes | Yes (universe health) |

### Why Constellation's position is distinct

InfraNodus's Obsidian plugin provides some of these capabilities but operates as an **external SaaS service**: notes are uploaded to InfraNodus's EU servers for processing. **Constellation's advantage is local-first computation** — all analytics run on the user's machine in Rust, with no data leaving the device. This is a fundamental architectural difference for privacy-conscious users.

InfraNodus also analyzes text at the **word level** (co-occurrence of terms). Sight analyzes at the **note level** (relationships between documents) — the natural unit for PKM. Sight can leverage structural signals (wikilinks, tags, library, folder hierarchy) that InfraNodus's word-level approach cannot access.

---

## §9 · Risks and Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Information overload from too many suggested gaps | Medium | Limit displayed gaps to top-5 by default. Rank gaps by relevance score combining community importance and gap width. Let users dismiss suggestions. |
| Betweenness centrality biased toward MOC/index notes | Medium | Layer peeling toggle (PJ-036): exclude high-degree notes from centrality. Alternatively, normalize centrality by degree. |
| Content similarity is computationally expensive | Medium | Make it opt-in. Compute incrementally on changed notes only. Cache TF-IDF vectors. Use approximate methods for large universes. |
| Users may not understand graph-analytics terminology | Low | Plain-language labels: "Bridge notes" not "Betweenness centrality." "Knowledge clusters" not "Communities." "Blind spots" not "Structural gaps." |
| Arabic text requires specific lemmatization for content similarity | Low | Modular NLP pipeline with language-specific stemmers. Arabic stemming (ISRIStemmer or similar) as a configurable option. Tag/link-based edges work without any NLP. |
| Feature overlap with the base graph view | Low | Sight is an **overlay**, not a replacement. The graph without Sight remains a valid, simpler view. Sight adds optional depth. |
| First-toggle latency on large universes | **High** *(added in v1.1)* | **Principle 6 (reveal-on-demand)**: edges hidden until hover/search/select. Drops per-frame edge-iteration to zero in the resting case. Shipped MIG-016 §1B. |

---

## §10 · Suggested Development Roadmap (v1.0 vintage — see §12 for actual status)

### Phase 1: Graph analytics engine (foundation) — *✅ Shipped*
- Brandes' betweenness centrality in Rust ✅
- Louvain community detection in Rust ✅
- Sight IPCs (`constellation_sight_centrality`, etc.) ✅
- Community coloring and centrality sizing as overlays in the graph view ✅
- **Deliverable**: graph nodes auto-colored by topic, sized by importance ✅

### Phase 2: Structural gaps and sidebar (intelligence) — *✅ Shipped*
- Structural gap detection (inter-community distance analysis) ✅
- Analytics sidebar panel in Svelte 5: top communities, top bridges, top gaps ✅
- Gap highlighting in the graph (dashed lines between unconnected community pairs) ✅
- Universe health metric (modularity + dominance + entropy + connectivity) ✅
- **Deliverable**: an active knowledge-discovery tool with actionable suggestions ✅

### Phase 3: Multi-edge and layer peeling (depth) — *⚠ Partial*
- Shared-tag edges as a second edge type — *✅ wired into shipping graph build*
- Content similarity edges (TF-IDF, incremental, opt-in) — **❌ not shipped (PJ-035)**
- Layer peeling: hide top-N centrality nodes and recalculate — **❌ not shipped (PJ-036)**
- Map↔Sight integration: click Map segment → open Sight filtered to that branch — **❌ not shipped (PJ-037)**

---

## §11 · Conclusion

InfraNodus demonstrates that applying network science to text — specifically betweenness centrality, community detection, and structural gap analysis — produces insights no amount of keyword counting, sentiment analysis, or LLM summarization can replicate. Its peer-reviewed methodology has been validated by researchers, organizations, and thousands of users worldwide.

Constellation Sight transplants this analytical paradigm from cloud-based text analysis to local-first personal knowledge management, operating at the **note level** rather than the word level, leveraging Constellation's existing Rust backend and graph infrastructure. The result is a knowledge-discovery engine that transforms a passive graph visualization into an active partner in thinking.

Combined with the Constellation Map, Sight completes Constellation's vision as a **knowledge intelligence platform**: the Map shows the *shape* of knowledge; Sight reveals its *hidden patterns*. No competing PKM tool offers either capability, let alone both.

---

## §12 · Truth-status matrix (added in v1.1)

A row-by-row scoreboard: each mechanic from this paper, mapped to *what's actually shipped* in v2 Sight as of 2026-05-07.

| Mechanic / claim | Status | Where it lives | Notes |
|---|---|---|---|
| **Betweenness centrality (Brandes)** | ✅ Shipped | `src-tauri/src/lens.rs::compute_centrality`; IPC `constellation_sight_centrality` | Powers node-size mapping. Uses Brandes' exact algorithm; sub-second on Boss-scale (~7,600 notes). |
| **Community detection (Louvain)** | ✅ Shipped | `src-tauri/src/lens.rs::compute_communities`; IPC `constellation_sight_communities` | Powers node-color mapping. Modularity reported alongside. |
| **Structural gap detection** | ✅ Shipped | IPC `constellation_sight_structural_gaps` | Top-N gaps surfaced in the Knowledge Insights panel inside Sight. |
| **Discourse bias / universe health (M + D + E + C)** | ✅ Shipped | IPC `constellation_sight_universe_health`; rendered as the universe-health card in Sight's right panel | Modularity, dominance, entropy, connectivity all computed and displayed. |
| **Layer peeling** | ❌ Not shipped | n/a | **PJ-036**. The "remove top-N centrality nodes and recompute" mechanic is unimplemented. v3 design north star (§13) gives this one a natural visual slot. |
| **Non-linear navigation (click node → open note)** | ✅ Shipped | `ConstellationSight2.svelte` mousedown / dblclick handlers | Single click selects + reveals neighborhood; double click opens the note. |
| **Edge type 1 — explicit wikilinks (weight 1.0)** | ✅ Shipped | `lens.rs` graph-build path | Source-of-truth edge type. |
| **Edge type 2 — shared tags (weight 0.6)** | ✅ Shipped | `lens.rs` graph-build path | Wired in alongside wikilinks. |
| **Edge type 3 — content similarity (weight 0.3, TF-IDF)** | ❌ Not shipped | n/a | **PJ-035**. The mechanic that most distinguishes InfraNodus from naïve graph tools. v2 Sight cannot detect *latent* connections (notes that talk about the same thing without a wikilink). |
| **Reveal-on-demand (Principle 6)** | ✅ Shipped | `ConstellationSight2.svelte` `needsEdgeDraw` gate + `focusOnly` filter | MIG-016 §1B (commit `62718f7`). Edges render only on hover / search / select / link-annotation hover. |
| **Sidebar analytics panel** | ✅ Shipped | "Knowledge Insights" sidebar inside Sight | Top communities, top bridges, structural gaps, universe-health card. |
| **Gap highlighting (dashed lines between unconnected community pairs)** | ⚠ Partial | n/a (panel-only) | Gaps are listed in the sidebar; they are not yet drawn as overlay arcs/lines on the graph itself. Slated for v3 (the star-chart aesthetic surfaces them naturally as proposed corridors). |
| **Map ↔ Sight integration** | ❌ Not shipped | n/a | **PJ-037**. Click a Map segment to open Sight filtered to that branch; Sight community selection visualized on the Map. The "Map diagnoses, Sight prescribes" loop (§7) is not yet round-trippable. |
| **Local-first / privacy** | ✅ Shipped | All Sight IPCs run in-process Rust; zero network calls | Matches Constellation's local-first principle (CLAUDE.md "Architecture Principles"). |
| **Language-agnostic (Principle 5)** | ✅ Shipped (for shipped edges) | The structural pipeline is language-blind | When PJ-035 lands, the language-specific NLP for content similarity needs the modular per-language stemmer pipeline §3.4 promises. |
| **Annotation-write UI for typed-link annotations** | ❌ Missing | n/a | Pre-existing gap (not part of this paper's claims, surfaced during MIG-016 review): there is no UI to set a link's `annotation` field today. The data model supports it; nothing writes to it. Track separately. |

### Summary of v2 Sight's truth-status

**~70-80% of the analytical promise is delivered.** The structural-analysis core (centrality, communities, gaps, health) is real and runs on the Boss's universe. The three notable omissions are:

1. **PJ-035** — content-similarity edges. The mechanic that lets Sight surface *latent* knowledge connections, not just the explicit ones the user already authored.
2. **PJ-036** — layer peeling. The mechanic that lets the user temporarily hide MOC/index nodes and see what's underneath.
3. **PJ-037** — Map↔Sight integration. The mechanic that lets the two analytical surfaces inform each other.

These are not architectural blockers — each is an additive feature that the existing pipeline can absorb. They are tracked in the Pending Jobs doc and inheritable into v3 by design.

---

## §13 · Star-chart vision — design north star (added in v1.1)

> *"To deliver the Sight promise, the UI should be 2D to begin with. The user should identify what Sight claims to deliver with one look."* — Eisa, 2026-05-07.

The Boss's reference image for v3 Sight is a 19th-century-style **printed star chart of the northern hemisphere**: a circular field of stars on a deep navy ground, constellations outlined in faint connector lines, the Milky Way drawn as a softer band of density across the chart, mythological figures sketched over the major constellations, and a calendar rim around the perimeter showing months and right-ascension hours.

The visual analogy is exact:

| Star-chart element | Sight semantic | Why it works |
|---|---|---|
| **Star magnitude** (brightness / size) | **Betweenness centrality** of a note | A bridge note *is* a bright star: structurally important even if it doesn't have the most edges. |
| **Constellation territories** (the bordered regions) | **Louvain communities** | A constellation is *a region of stars that hangs together as a meaning*. That's exactly what a community is. |
| **Constellation lines** (the connectors between stars within a constellation) | **Wikilinks + shared-tag edges within a community** | The lines that tell you *which* stars belong together. Shown only when you focus on a constellation — same as Principle 6. |
| **Milky Way band** (the soft density wash) | **Content-similarity edges (PJ-035)** | A diffuse texture of *related-but-not-explicitly-connected* mass. The "this is a region of related thinking" wash. |
| **Mythological figures / labels** | **Community labels** (top-3 representative terms per cluster) | Tells you *what this region of the sky is about*. |
| **Calendar rim** (month / RA-hour markers around the perimeter) | **Time dimension** — note creation date, last-traversed, lifecycle stage band | Makes time a navigable axis. The user can rotate the rim to see "what was I writing about this month last year." |
| **Empty patches between constellations** | **Structural gaps** | The dark sky *between* constellations is where the next discovery lives. Same intuition as Burt's structural holes. |
| **The dome of the sky as a whole** | **Universe health** at a glance | A balanced sky with constellations distributed evenly across the dome reads as healthy; one constellation taking 60% of the dome reads as imbalanced — *immediately, visually, before any number*. |

### Why star-chart over force-directed (v2's choice)

Force-directed layouts (the v2 approach) carry several costs:

- They are *layout-driven*: the same universe re-runs the simulation differently every time, so the user can't build a *spatial mental map* of where things live.
- They show edges-first: the graph reads as a tangle, and the InfraNodus answer to that — colour and centrality overlays — only goes so far.
- They don't have a natural visual home for *time* or for *gaps*.

Star charts solve all three:

- The layout is **stable** — once the projection is computed, the same notes always sit in the same regions of the dome. The user *learns the sky*. Spatial memory becomes a feature.
- The layout is **node-first** — stars are points; the connectors are an annotation laid over them. This is Principle 6 made visual: stars are always visible, connector lines render only when the user focuses a constellation.
- The layout has **natural slots for time** (the calendar rim) and **for gaps** (the dark sky between constellations).

### The InfraNodus comparison

InfraNodus is a *force-directed graph with topology overlays*. v3 Sight, by contrast, is a *spatial map with topology meaning baked into position*. The technical analytics (Brandes, Louvain, structural gaps, universe health) survive unchanged from v2 — they continue to be the math. What changes is the *visualization grammar*: from "physics simulation that re-runs each session" to "fixed sky-map that the user navigates by spatial memory." This is closer to what the Boss said the user should be able to identify "with one look."

---

## §14 · v3 redesign — PJ-038 (added in v1.1)

**Decision** (Eisa, 2026-05-07): **secure what's achieved, never muddle**.

- **v2 Sight** (`ConstellationSight2.svelte` + the `lens_*` Rust modules + `constellation_sight_*` IPCs) is **disabled** as a known-good fallback under **MIG-017** (single mini-MIG session). The whole user-visible package is hidden: dock button, modal, Settings entry. The compute IPCs and the v2 Svelte component are kept on disk — they are the proven baseline if v3 fails.
- **v3 Sight** is built **fresh** from the star-chart aesthetic (§13). Allocated as **PJ-038**.
- **PJ-038 gets its own dedicated Concept Paper.** This v1.1 paper is the *analytical foundation* both v2 and v3 share; the v3 paper is the *visual and interaction specification* for the new build. Read both side-by-side when v3 work begins.

### What v3 inherits from v2

The Rust analytical pipeline is inheritable as-is. v3 reads the same JSON payloads from the same Sight IPCs:

- `constellation_sight_centrality` → star magnitude.
- `constellation_sight_communities` → constellation territories.
- `constellation_sight_structural_gaps` → dark-sky gaps between constellations.
- `constellation_sight_universe_health` → the dome-balance read.

And the three deferred PJs (PJ-035 / PJ-036 / PJ-037) drop into v3 with cleaner visual homes than they would have had in v2:

- **PJ-035 content-similarity edges** → become the **Milky Way band** (a diffuse density wash, not extra edge lines competing with the constellation connectors).
- **PJ-036 layer peeling** → becomes a **"hide brightest stars" toggle** — visually obvious instead of buried in a menu.
- **PJ-037 Map↔Sight integration** → becomes a **two-up panel** where the Map (sunburst) and Sight (sky chart) share a selection cursor.

### What v3 cannot inherit

The visual layer is rebuilt entirely:

- **Force-directed Pixi.js simulation** → 2D polar projection (or equivalent fixed-position computation).
- **D3-style force layout** → astronomy-style projection math (likely Lambert azimuthal equal-area or similar — the v3 paper will pick).
- **Edge-render hot path** → rebuilt around the constellation-line idiom (lines render only inside the focused constellation territory, never globally).

### Inheritability of v2's perf wins

The MIG-016 §1A perf instrumentation (the `performance.mark` ladder around `toggleLens()`) and §1B reveal-on-demand pattern transfer cleanly into v3 — they encode universal lessons (mount is fast, edge iteration is the cost) rather than v2-specific wiring.

The MIG-016 §1E SQLite `sight_cache` design (deferred from v2) becomes more valuable in v3, where the projection math is deterministic per-universe-snapshot — caching the projected positions is a clean win.

### Why the rebuild is the right call

> *"To secure what has been achieved so far with the current Sight, we will disable it for now (the whole package), and will create the new Sight (v3) based on the current wins. If it proves its worth, we will use it. If not, we already have the current one."* — Eisa, 2026-05-07.

The reasoning is sound:

1. **v2's analytics work** — the math is right, the IPCs are stable, the caching is correct. That's preserved.
2. **v2's visualization is force-directed** — which is what every PKM tool ships, and which the Boss has tried and decided is not enough.
3. **The risk of "improving v2 incrementally" is muddling**: each tweak fights the force-directed grammar. A clean v3 build, on a shelved v2 fallback, lets the new aesthetic be tried without losing the proven baseline.

---

## §15 · Cross-references

This paper is read alongside:

- **`docs/Constellation_Lens_Concept_Paper_Eisa.pdf`** — the original v1.0 source. This v1.1 supersedes it for active reference but the PDF stays as historical record.
- **`docs/Constellation Pending Jobs v1.4.md`** — PJ-034 (Sight perf), PJ-035 (content-similarity edges), PJ-036 (layer peeling), PJ-037 (Map↔Sight integration), PJ-038 (Sight v3 build with own Concept Paper).
- **`docs/Constellation_Map_Concept_Paper_Eisa.pdf`** — the companion paper. Map and Sight are designed to be read together.
- **`docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md`** — the user-facing help doc. Reflects what *currently* ships in v2.
- **`lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`** — the perf-and-edge-on-hover audit; documents what §1A / §1B shipped and why §1C / §1D were cancelled and §1E deferred to v3.
- **`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`** — the broader knowledge-formulation philosophy that frames *why* Sight matters (the Five Acts: Observation → Connection → Tension → Synthesis → Conviction).
- **`src/lib/components/ConstellationSight2.svelte`** — v2 implementation (the version being shelved under MIG-017).

---

**End of v1.1.** The next document in the chain is the **PJ-038 v3 Concept Paper**, which will specify the star-chart projection, the constellation-territory rendering, the calendar-rim time axis, and the precise visual grammar that this v1.1 only sketches in §13. Until that paper exists, this v1.1 is the canonical reference for *what Sight is for*; the v3 paper, when it lands, will be the canonical reference for *what v3 looks like and how it is built.*

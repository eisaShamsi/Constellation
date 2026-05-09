# Constellation Sight — Concept Paper v2.0

**Version 2.0 | 2026-05-09**
**Author**: Eisa Alshamsi · eisa@uconstellation.world · drafted with Claude.
**Audience**: every future Claude session, the Boss reviewing where Sight stands, any future contributor.
**Supersedes**: `Constellation-Sight-Concept-Paper-v1.1.md` (InfraNodus-spined), `Constellation-Sight-v3-Concept-Paper-v1.1.md` (per-mode-X/Y/Z grammar), `SIGHT-V3-VISUAL-SPEC.md` (centrality-on-radius). All three preserved on disk as historical record per SO #6.

> **What's new in v2.0** (Boss-ratified 2026-05-09):
>
> 1. **InfraNodus heritage dropped.** The analytical-network-science framing (Brandes betweenness, Louvain, structural gaps, universe-health metrics) is **out**. The four `constellation_sight_*` IPCs are orphaned and queued for deletion alongside `lenses.rs::apply_lens`.
> 2. **New canonical question:** "How is my Epistemic Content shaped and/or organized?" (Eisa: *"forget about the InfraNodus."*)
> 3. **New scholarly foundation:** the Universal Epistemic Content Taxonomy — 5 branches × 11 sources, drawn from five civilizational epistemological traditions (Greek + Western analytic; Sunni Islamic *kalām* / *uṣūl al-fiqh* / *falsafa*; Indian *pramāṇa-vāda*; Mohist / Neo-Confucian Chinese; Persian-Islamic *Ishrāqī*) plus Jewish, Tibetan Buddhist, African, and Mesoamerican supplementary input.
> 4. **Strata is the constant radius across all modes.** Only azimuth changes per mode. Star size = maturity, brightness = confidence, red dot = contested — all constant. This REVOKES the v1.x per-mode (X, Y, Z) grammar.
> 5. **Seven modes (R / L / T / C / S / A / P).** P = Provenance is new — the Sources axis lifted from the taxonomy's horizontal dimension.
> 6. **Sources tracked Day 1.** New per-note frontmatter field `sources:` (multi-source, ranked) + `note_meta.sources` SQLite mirror. Auto-classifier (local LLM) proposes; user approves via the new Source Review sidebar panel; nothing is silently inferred or defaulted.
> 7. **Visual aesthetic preserved.** Suwaidi cream-parchment night sky, faint connector lines at rest brightening on hover, calendar rim, soft Milky Way wash. The aesthetic that v3 invested in survives — only what those visuals *mean* changed.
> 8. **First-sight understanding within ~5 seconds is a hard constraint.** A first-time Constellation user with no training must understand what they're looking at. This codifies Eisa's 2026-04-13 directive (*"simplicity should come from understanding what you see at first sight, NOT to raise more questions"*) into a measurable design gate.
> 9. **Sight = whole universe; 360.3D = single note.** Mutually exclusive scopes. The "what Sight is NOT" line the v1.x papers never wrote, finally written.

---

## §0 · What this paper IS (and is NOT)

This paper is the **canonical specification of Constellation Sight v5** — the design contract every implementation phase reconciles against. It defines the question Sight answers, the visual grammar, the seven modes, the Sources subsystem, the Epistemic Classifier, and the boundary lines that distinguish Sight from every other Constellation surface.

This paper is **not** a status report. As of writing, **no Sight v5 code has been written.** The v4 build on `main` is paused; v5 is gated on this paper's approval, the MIG-021 Architect doc, and Eisa's confirmation per the /migration discipline.

This paper is **not** the Universal Epistemic Content Taxonomy itself. The taxonomy is the scholarly foundation — see `docs/epistemic-content-EN.md`, `docs/epistemic-content-AR.md`, `docs/epistemic-content-taxonomy.md`, `docs/epistemic-content-taxonomy-chart.html`. This paper specifies how Sight v5 *uses* the taxonomy as backend vocabulary while keeping the UI plain.

---

## §1 · Executive Summary

**Constellation Sight v5** is a single full-screen surface that visualizes a user's entire knowledge universe as a stable star chart on cream parchment. Each note is a star. The dome is divided into eight concentric strata bands (L1 Datum at the rim, L8 Worldview at the pole), and seven mode toggles re-cut the rim wedges to answer different cognitive questions:

- **R** — *Regions*: which library each note lives in
- **L** — *Link Types*: what kind of reasoning the note does
- **T** — *Time*: when the note was created
- **C** — *Confidence*: how certain the note's claims are
- **S** — *Stages*: where the note is in its lifecycle
- **A** — *Acts*: where the note is in the formulation arc
- **P** — *Provenance*: what source produced the note's content (perception, inference, testimony, mass-transmission, comparison, postulation, non-apprehension, memory, innate disposition, inspiration, revelation)

The same notes appear in every mode at the same strata band; only the wedge slicing changes. Star size encodes maturity. Star brightness encodes confidence. Red dots encode contested. Faint colored lines encode typed links (visible at rest, brighten on hover/select).

Sight v5 answers exactly one question: **"How is my Epistemic Content shaped and/or organized?"** It does not analyze, score, or recommend. It shows the user the shape of their own thinking through whichever cognitive lens they pick.

---

## §2 · The canonical question

> *How is my Epistemic Content shaped and/or organized?*

**Shape** — radial weight per stratum. *Where does my thinking concentrate?* A user whose stars cluster mid-dome (L4-L5 Hypothesis-Theory) is in active conjecture. A user whose stars cluster near the rim (L1-L2 Datum-Fact) is collecting raw material. A user with bright stars at the pole (L7-L8 Perspective-Worldview) has synthesized.

**Organization** — wedge distribution per active mode. *How is my thinking grouped?* Under Regions: which library carries which strata. Under Time: in which months I produced what. Under Provenance: how much of my universe was perception versus inference versus testimony versus revelation.

The two readings are visible simultaneously in every mode — *shape* lives on the radial axis, *organization* lives on the angular axis. The user toggles modes to ask different organization-questions while the shape (strata distribution) stays put.

---

## §3 · The scholarly foundation

### §3.1 The Universal Epistemic Content Taxonomy

Sight v5 inherits its vocabulary from a cross-civilizational taxonomy of cognitive objects (`docs/epistemic-content-taxonomy.md`). The taxonomy was distilled from five major epistemological traditions that independently identified the same structural loci:

| Tradition | Source documents | Key contribution |
|---|---|---|
| Greek & Western analytic | Plato, Aristotle, Stoics, Kant, Frege, Russell, Polanyi | The JTB analysis of knowledge; the *lekton* / propositional content; the Data-Information-Knowledge-Wisdom (DIKW) hierarchy |
| Sunni Islamic — *kalām*, *uṣūl al-fiqh*, *falsafa* | Al-Jurjānī, Al-Āmidī, Al-Ghazālī, Ibn Sīnā, Ibn Rushd | The *taṣawwur* / *taṣdīq* binary; the graded epistemic-states scale (*shakk* → *ẓann* → *ʿilm* → *yaqīn*); the *masādir al-maʿrifah* (sources of knowledge) including the distinctive *al-tawātur* |
| Indian *pramāṇa-vāda* | Nyāya, Mīmāṃsā, Vedānta, Buddhist (Dignāga, Dharmakīrti), Jaina | The six *pramāṇa* (perception, inference, comparison, testimony, postulation, non-apprehension); the *prama* / *aprama* distinction; *svalakṣaṇa* / *sāmānyalakṣaṇa* |
| Classical Chinese | Mohist Canon, Confucian, Daoist, Neo-Confucian | The *míng-shí* (name-reality) correspondence; the three sources (*wén zhī*, *shuō zhī*, *qīn zhī*); *zhī xíng hé yī* (unity of knowing and acting) |
| Persian-Islamic *Ishrāqī* | Suhrawardī | *Al-ʿilm al-ḥuṣūlī* (representational knowledge) vs *al-ʿilm al-ḥuḍūrī* (presential knowledge) |

Plus supplementary input from Jewish (Maimonides), Tibetan Buddhist (Sa-paṇ), African (Wiredu, Oruka), and Mesoamerican (León-Portilla, Maffie) traditions.

### §3.2 The two-axis structure

The taxonomy has **two orthogonal axes**:

- **Vertical** — five primary branches of epistemic content: (1) Sensory inputs · (2) Symbolic entities · (3) Semantic contents · (4) Epistemic states · (5) Higher-order constructs.
- **Horizontal** — eleven sources/means of knowledge: perception · inference · testimony · mass-transmission · comparison · postulation · non-apprehension · memory · innate disposition · inspiration · revelation.

A single epistemic item is located by both axes: *what kind of content is it* (vertical) × *what source produced it* (horizontal).

### §3.3 Strata IS the Constellation projection of the vertical axis

The existing 8-level strata field — populated by the user across all 7,636 trial-universe notes — maps cleanly onto the 5-branch vertical axis condensed by epistemic elevation:

| Stratum | Universal Taxonomy mapping |
|---|---|
| L1 Datum | Branch 1 (sensory inputs) + Branch 2.3 (data) |
| L2 Fact | Branch 3.5 (fact / *wāqiʿah*) |
| L3 Opinion | Branch 4.5 (epistemic state — *ẓann*) |
| L4 Hypothesis | Branch 5.1 (construct — *faraḍiyyah*) |
| L5 Theory | Branch 5.2 (*naẓariyyah*) |
| L6 Framework | Branches 5.3 – 5.5 (model / law / doctrine) |
| L7 Perspective | Branches 5.6 – 5.7 (insight / wisdom — *baṣīrah / ḥikmah*) |
| L8 Worldview | Branch 5.8 (*ruʾyah kawniyyah*) |

This means **Sight v5's strata-as-radius design is doubly justified** — by Constellation's native taxonomy (the user already labels every note with a stratum) AND by the cross-civilizational scholarly tradition the new taxonomy synthesizes. The user's existing labels become the spatial backbone of Sight v5 with no new annotation required.

### §3.4 Sources are the new dimension Sight v5 introduces

The horizontal axis (the 11 sources / *masādir al-maʿrifah* / *pramāṇa*) is **not yet tracked in Constellation today**. Sight v5 introduces it as a new per-note frontmatter field (`sources:`) and as the seventh Sight mode (P — Provenance). See §7 for the Sources subsystem and §8 for the Epistemic Classifier that helps users populate it.

---

## §4 · The visual grammar

### §4.1 The dome

A circular field on Suwaidi cream parchment (`#faf6e8` background). Eight concentric strata bands divide the dome from the central pole (L8 Worldview) to the rim (L1 Datum). Faint sand grid lines (`#b8a98a`) mark each band boundary.

A 12-month calendar rim wraps the outside of the dome (always present; serves the Time mode and provides a stable temporal reference in every other mode).

A soft Milky Way wash drifts across the chart in two diffuse ellipses — content-similarity density, the visual texture of *related themes that aren't explicitly linked*.

### §4.2 Stars

Each note is a single circular dot — a star.

- **Position**: radial = the note's stratum (L8 center → L1 rim, never changes); angular = determined by the active mode's wedge slicing (see §5).
- **Size**: maturity (seed = 1.5 px, sapling = 2.5 px, evergreen = 3.5 px, canonical = 5 px, wilting = 2 px greyed). Constant across modes.
- **Brightness**: confidence (hypothesis = 0.45 alpha, evidence = 0.7, established = 1.0). Constant across modes.
- **Color**: ink (`#1a1a1a`) by default. Red (`#a83232`) for notes whose primary link confidence is *contested*. Constant across modes.

### §4.3 Connector lines

Faint at rest (~0.10–0.15 alpha), color-coded by typed-link kind:

- Green (`#3a8a4a`) — *supports* / *derives-from*
- Red (`#a83232`) — *contradicts*
- Gold (`#c9a227`) — *exemplifies* / *generalizes*
- Blue ink (`#2a4a8c`) — *causes* / *part-of*

On hover or select, the focused star's incident edges brighten to ~0.85 alpha; other edges stay faint. **Principle 6 from v1.1 reframed**: *reveal* now means *brighten*, not *render-from-zero*. The structural pattern of the universe is always visible at rest; focus simply highlights what the user is looking at.

### §4.4 Toggle bar

A six-button row at the top of the dome (later seven when P ships): **R · L · T · C · S · A · P**. Active mode gold-filled with parchment letter. Available-but-inactive modes have cream background + ink letter + thin outline. Modes whose data isn't yet populated (e.g., before user has assigned stages or sources) are dimmed with a dashed outline and hover-tooltip explaining what unlocks them.

A small caption under the bar names the active mode in the user's interface locale.

### §4.5 What is NEVER shown in the chrome

- Civilizational labels ("Branch 2 Symbolic Entities", "*pramāṇa*", "*kalām*"). The taxonomy is the scholarly foundation; the UI uses Constellation-native vocabulary only.
- Network-science terminology ("betweenness centrality", "Louvain modularity", "modularity score"). These are the InfraNodus heritage Sight v5 is leaving behind.
- Numerical scores out of 100. Sight v5 shows distributions visually; if the user wants a number, they read the count badge on hover.

---

## §5 · The seven modes

Each mode declares its own **azimuth** (rim wedge slicing). Strata stays the radius. Star size, brightness, color, and link colors stay constant. The migration animation between modes (~600 ms ease) interpolates only the angular position of each star — stars slide tangentially around their stratum ring as the wedges re-cut.

| ID | Mode | Wedge basis | The cognitive question | Data source |
|---|---|---|---|---|
| **R** | Regions | Library (sized by note count, biggest first) | "Where in my cosmos does this idea live?" | Library membership (existing) |
| **L** | Link Types | Dominant outgoing link type (7 types + Untyped) | "What kind of reasoning, and how versatile?" | `note_links.link_type` (existing) |
| **T** | Time | Creation month (12 wedges; current month subtly highlighted) | "When did it emerge, and is it still alive?" | `note_meta.created` (existing) |
| **C** | Confidence | Dominant per-note link confidence (4 wedges: hypothesis · evidence · established · contested) | "How certain, and how consistent?" | `note_links.confidence` (existing) |
| **S** | Stages | Dominant lifecycle stage (6 wedges: Spark → Birth → Growth → Maturity → Dormancy → Archival) | "How alive, and how worn the path?" | `note_meta.stage` (shipped MIG-014) |
| **A** | Acts | Which Act produced the note (5 wedges: Observation → Connection → Tension → Synthesis → Conviction) | "Where in the formulation arc?" | per-note act tag (CE Layer 2 — partial) |
| **P** | Provenance | Primary source of the note's content (11 wedges from the taxonomy) | "What kind of knowing produced this?" | `note_meta.sources` (NEW — see §7) |

When a mode's data is partially populated, Sight renders what's available and shows an "Unsourced" / "Unstaged" / "Unacted" wedge for missing data — the visible wedge becomes a to-do list.

### §5.1 Mode persistence

Last-used mode persists per Universe via `appSettings.sight.lastMode`. Default for first-time use: **R** (Regions). If the saved mode is unavailable (P before any sources are assigned), fall back to R.

### §5.2 Why no per-mode (X, Y, Z) grammar

The v3 visual spec proposed each mode declaring its own (X = azimuth, Y = radius, Z = magnitude). v5 revokes this. Reasons:

1. **Spatial memory survives mode switches.** The user learns the dome once; mode toggles re-aim the lens through the same sky.
2. **The cognitive question maps cleanly.** *Shape* lives on the radius (constant strata); *organization* lives on the angular axis (mode-dependent wedges). One image, two readings.
3. **Cross-surface coherence with 360.3D.** 360.3D's Stratification Matrix anchors per-note view to strata. Sight anchoring per-universe view to strata makes the two surfaces echo — same axis, different scope.
4. **The InfraNodus-derived (X = library, Y = centrality rank) Regions-mode anchor doesn't apply** with InfraNodus dropped. The natural successor is constant-strata-radius.

The cost: stars don't fly across the sky between modes (the v3 "diagnostic migration trajectory" is gone). A star at L7 in May stays at L7 in Research. That cost is acceptable — diagnostic patterns appear in the *wedge weights* (which library has the heaviest L7 cluster?), not in star migration.

---

## §6 · The four constants

These are the load-bearing invariants. Every mode honors them. Breaking any is a P0 regression.

| Constant | Encoded property | Source data |
|---|---|---|
| **Radius** | Strata (L8 pole → L1 rim) | `note_meta.stratum` |
| **Size** | Maturity (5 sizes from seed to canonical) | `note_meta.maturity` |
| **Brightness** | Confidence (3 alpha levels from hypothesis to established) | derived from `note_links.confidence` (per-note primary) |
| **Red dot** | Contested (any inbound `contradicts` link with non-archived status) | `note_links.link_type` + `confidence` |

Color stays ink for everything else. Library color is **not** used for stars — that's the Map's vocabulary; Sight uses the strata-band rings as its grouping signal, not per-star color.

---

## §7 · The Sources subsystem

The Sources axis is the new dimension Sight v5 introduces. It is the horizontal axis of the Universal Epistemic Content Taxonomy lifted into Constellation as a per-note property.

### §7.1 The 11 sources

| # | English | Arabic | Sanskrit | Definition |
|---|---|---|---|---|
| 1 | Perception | الحِسّ | *pratyakṣa* | Direct sensory contact with an object |
| 2 | Inference | العَقل | *anumāna* | Derivation of conclusion from premises |
| 3 | Testimony | الخَبَر | *śabda* | Reliable verbal report from another knower |
| 4 | Mass-transmission | التَّواتُر | — | Convergent reports too numerous to collude on falsehood |
| 5 | Comparison | القياس | *upamāna* | Knowledge of an object via similarity to a known object |
| 6 | Postulation / IBE | الاستنباط الافتراضي | *arthāpatti* | Inference of an unobserved fact required to explain an observed one |
| 7 | Non-apprehension | عَدَم الإدراك | *anupalabdhi* | Knowledge of absence via the absence of perception |
| 8 | Memory | الذاكرة | *smṛti* | Recall of previously cognized content |
| 9 | Innate disposition | الفِطرة | — | Pre-experiential cognitive endowment |
| 10 | Inspiration | الإلهام | — | Non-discursive apprehension in mystical traditions |
| 11 | Revelation | الوحي | — | Communication from a divine source |

All 11 ship Day 1. Per the taxonomy's own pluralism on contested categories, Constellation does not editorialize: a journalist logging a quoted source picks *testimony*; a *Hadith* scholar picks *mass-transmission*; a poet logging a vision picks *inspiration*; a philosopher logging a deduction picks *inference*. Constellation provides the vocabulary and lets the user choose.

### §7.2 Per-note storage

**Frontmatter** (canonical, source of truth):

```yaml
sources:
  - testimony
  - mass-transmission
```

The list is **ordered**: first = primary source, subsequent = secondary. A note may have any number of sources from 0 to 11. An empty/missing field means *unsourced* (the user hasn't classified yet).

**SQLite mirror** (`note_meta.sources`, JSON-encoded list, fast-read index). Updated by the existing write-time `scan_note_*` pipeline that already maintains `stratum`, `maturity`, `stage`. Mirror policy matches MIG-014: frontmatter wins on disagreement; SQLite is the read cache.

### §7.3 Setting sources — three paths

1. **PropertyEditor combobox** — multi-select dropdown listing all 11 sources in the user's interface locale. Matches the existing Strata / Maturity / Stage controls exactly (Law 2.7 single-source-of-truth).
2. **Source Review sidebar panel** — a queue surface where the Epistemic Classifier (§8) presents its suggestions one note at a time. User can Accept (writes to `sources:`), Edit (modify the suggestion before accepting), or Reject (skip and clear the suggestion). Mirrors the existing Review Pulse panel pattern.
3. **Right-click → "Suggest sources for this note"** — context menu on any note (in the file tree, in Sky View, in Sight, anywhere a note is selectable). Triggers an on-demand single-note classification; result appears in the Source Review panel.

### §7.4 The "Unsourced" wedge in Sight mode P

Notes whose `sources:` field is empty or missing render in a dedicated **Unsourced** wedge in mode P. This wedge is the visible to-do list — its size shrinks as the user (with the classifier's help) classifies more of the universe. A universe whose Unsourced wedge dominates is a universe whose epistemic provenance has not been examined yet; the visual itself is the prompt.

---

## §8 · The Epistemic Classifier

A new Constellation subsystem that reads notes and proposes source assignments for the user to approve.

### §8.1 Architecture overview

Two-tier classifier strategy (per the 2026-05-09 LLM research, `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`):

**Tier 1 — Bundled starter classifier (default, Day 1, no extra requirements)**

Reuses Constellation's existing `multilingual-e5-small` ONNX embedding model (113 MB, already shipping for semantic search). Classification by embedding-similarity:

1. At app build time: embed the canonical English + Arabic definition of each of the 11 sources from the taxonomy. Cache the 11 vectors.
2. At classification time: embed the note's content (title + body). Compute cosine similarity to each of the 11 source vectors. Take top-N (default N=3) as suggestions, ordered by similarity descending.

Strengths: zero additional bundle cost, works in all 15 locales out of the box, no LLM dependencies. Limitations: embedding-similarity struggles to distinguish conceptually adjacent sources (testimony vs mass-transmission; perception vs memory; revelation vs inspiration). Realistic accuracy: ~65–75% top-1 on Eisa's universe.

**Tier 2 — Optional larger classifier (downloadable, higher accuracy)**

User downloads **Qwen3-1.7B Q4_K_M GGUF** (~1.1 GB) from Settings → AI when they want higher accuracy, particularly for Arabic / Hebrew / Persian content. Inference via **llama.cpp** (`llama-cpp-2` Rust crate). Classification by structured generation:

1. Few-shot prompt with the 11 source definitions + ~5 hand-written examples of correct classifications.
2. **GBNF grammar constraint** forces the LLM to output valid JSON matching the schema `{ sources: [{name: SourceID, confidence: 0..1, evidence: string}] }`.
3. Top-N (default N=3) returned, sorted by confidence descending.

Strengths: ~85–90% top-1 accuracy across all 15 locales, particularly strong on Arabic. Limitations: requires the optional download; first-run latency until the model is loaded into RAM.

### §8.2 Both tiers feed the same review queue

Tier 1 and Tier 2 produce identical-shaped suggestion records. They write to a separate frontmatter field `sources_suggested:` (NOT `sources:` directly — the user is the only path to canonical assignment). Once the user approves via the Source Review panel, the suggestion is consumed: `sources:` is written, `sources_suggested:` is cleared.

### §8.3 Classification triggers

- **Background scan** — opt-in per Universe in Settings → AI → Source Classifier. Runs on idle (mirrors `sky_backfill` pattern), processes the universe in chunks with progress in the status bar. Resumable across sessions. Cancel-able.
- **On note save** — when a note is created or edited substantially (configurable threshold), classify in the background within ~30 seconds.
- **On demand** — right-click a note → "Suggest sources for this note" runs the classifier synchronously on that single note.

### §8.4 Reversibility

The user can clear `sources:` for any note from the PropertyEditor. The classifier will re-suggest on the next scan if background classification is enabled. Nothing the classifier writes is destructive — `sources_suggested:` is the buffer, `sources:` is the user-controlled canonical field, and clearing either is a one-click revert.

### §8.5 Boot perf invariant

Per CLAUDE.md Performance Rule 8 (Write-Time Derivation) + Rule 3 (no heavy work on the main thread):

- **Tier 1 classifier**: e5-small loads lazy on first classification (existing pattern from semantic search). Zero impact on boot.
- **Tier 2 classifier**: Qwen3-1.7B loads lazy on first use. Optionally unloaded after configurable idle timeout. Zero impact on boot.
- **Background scan**: starts only after `boot:hydrated`. The boot critical path stays ≤ 6 seconds (currently ≤ 1 s on Eisa's machine).

---

## §9 · What Sight v5 IS NOT

The 360.3D Concept Paper (`docs/360.3D-Concept-Paper-v1.0.md`) explicitly enumerates "what 360.3D is NOT vs Sky View, Map, Sight, Index, OrgChart." That section is load-bearing — it's the boundary that prevents accidental duplication. The v1.x Sight Concept Papers never wrote the reciprocal section. v2.0 writes it.

| Surface | What it answers | Why Sight v5 is not it |
|---|---|---|
| **Sky View** | "What does my note network *feel* like, alive?" | Sky View is a force-directed PIXI bubble graph showing the live nervous-system topology. Sight v5 is a stable star chart anchored by strata. Different visual grammar, different question. Sky View edges-on-hover; Sight stars-by-strata. |
| **Constellation Map** | "What is the shape and density of my libraries?" | The Map is a D3 sunburst tracking the file-tree hierarchy (Universe → cUniverses → Libraries → Folders → Notes). Sight v5 is a sky chart tracking epistemic content distribution. Different organizing principle (hierarchy vs strata). |
| **OrgChart** | "What is my Universe's organizational hierarchy?" | OrgChart is the connector-box tree of structural containment. Sight v5 doesn't show containment; it shows distribution. |
| **Search Hub** | "Where in my system does this term/link/concept exist?" | Search is point-query against a corpus. Sight is whole-universe distribution. They share zero use cases. |
| **Index Panel** | "Which terms appear in my notes and where?" | Index is term-level vocabulary browsing (built on FTS5). Sight is note-level epistemic distribution. Adjacent surfaces, orthogonal jobs. |
| **360.3D / Inspector 360** | "Where does THIS note stand? (Position / Profile / Absence)" | **Eisa's load-bearing line, 2026-05-09**: 360.3D = single note; Sight = whole universe. Mutually exclusive scopes. Selecting a note in Sight should hand off to 360.3D / the editor — not deepen Sight's own per-note view. |
| **Knowledge Health Dashboard** | "What's the health of my link-graph (lifecycle, decay, hubs, weak foundations, bias alerts)?" | KHD is link-graph diagnostics. Sight is epistemic content distribution. KHD's universe-level metrics (modularity, dominance, entropy, connectivity from the InfraNodus heritage) are NOT replicated in Sight v5 — they were already shipped in KHD when the InfraNodus framing was alive. |
| **Multi-Lens** (`lenses.rs`, CE Phase 9) | (was: "group notes by tag / property") | WITHDRAWN. `lenses.rs::apply_lens` queued for deletion. The "group-by" job is reframed as Sight's multi-mode wedges. |

The five core functions invariant from 2026-04-13 still holds: **Search Hub · OrgChart · Sky View · Map · Sight** are the five non-overlapping cognitive surfaces. 360.3D, KHD, Index, Backlinks, Outgoing, Tags, Tasks, Calendar, Tension, Provenance, Review Pulse, Sense-Making Canvas, Expression Forge, etc. are adjacent surfaces with their own non-overlapping jobs.

---

## §10 · Performance budgets

Per CLAUDE.md Performance Rules + the 2026-04-15 boot-perf discipline:

| Metric | Budget | Notes |
|---|---|---|
| First-toggle latency on 7,636-note universe (cold) | ≤ 500 ms | Layout cache miss; rebuild + draw |
| First-toggle latency (warm SQLite cache) | ≤ 50 ms | Layout cached; draw only |
| Mode-switch animation (R ↔ L ↔ T ↔ ...) | 600 ms ease | Pure JS re-projection, no IPC |
| Hover-star highlight | ≤ 16 ms (single frame) | Decoration on focus overlay only |
| Idle-Sight per-frame cost (no hover, no select) | ≤ 1 ms | Static base layer; no redraw |
| Memory footprint (Sight open, 7,636 notes) | ≤ 40 MB above app baseline | Two Canvas 2D layers + DOM overlays |
| Boot impact | **Zero** | Sight is lazy-mounted on dock-button click; layout cache warmed via `requestIdleCallback` after `boot:hydrated` |

The classifier has its own budget per §8.5.

---

## §11 · Phased rollout

Sight v5 ships across three sequenced MIGs, each one landable independently and gated by an Eisa test:

### §11.1 MIG-021 — Sources subsystem + Epistemic Classifier (foundation)

Lands the Sources field, the e5-small bundled classifier, the Source Review panel, the PropertyEditor combobox, the right-click context action, the background scan job, and the 15-locale i18n. Sight itself is not yet built; this MIG produces the *data* Sight v5 will visualize.

Eisa test gate: classifier suggests sources for a sample of trial-universe notes; user approves a few via the panel; verify `sources:` lands in frontmatter and `note_meta.sources` mirror; verify multi-locale labels render correctly.

Dedicated Architect doc: `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`.

### §11.2 MIG-022 — Sight v5 visual foundation

Lands the dome, the eight strata bands, the calendar rim, the toggle bar with R + T modes (the two whose data is fully populated), the four constant encodings (radius / size / brightness / red), the connector-line layer, the hover/select interactions, and the side panel.

Eisa test gate: open Sight v5 from the dock, verify the dome renders correctly on the trial Universe in both R and T modes, verify mode toggle and migration animation, verify hover / select / Esc behaviors, verify the close button works (the v3/v4 lesson: mount inside `.content-area` per SkyView pattern, NOT `position: fixed` overlay).

### §11.3 MIG-023 — Modes L / C / S / A / P + Tier-2 classifier + close-out

Lands the remaining five modes (each gated on its data being available — A and P specifically rely on MIG-021's sources field), the optional Qwen3-1.7B download path, the Tier-2 inference engine integration via `llama-cpp-2`, the Settings → AI panel for downloading and managing the larger classifier, and the comprehensive 15-locale help docs + User Manual rewrite of the Sight section.

Eisa test gate: full mode rotation (R → L → T → C → S → A → P) on a Universe with sources populated, optional download path tested end-to-end, accuracy comparison between Tier 1 and Tier 2 on a hand-labeled subset.

### §11.4 Cleanup MIG (number TBD)

Deletes `lenses.rs::apply_lens` (CE Phase 9 withdrawn), the orphaned `constellation_sight_*` IPCs in the old `sight.rs`, the v2/v3/v4 Sight Svelte components (after Eisa confirms v5 stable across multiple sessions), and the obsolete v1.x Sight Concept Papers (move to `docs/historical/` per SO #6).

---

## §12 · Acceptance criteria (high level)

For Sight v5 to close as Done:

1. The five core-function invariant holds: Sight v5 does not duplicate Search Hub, OrgChart, Sky View, Map, or any adjacent surface (per §9).
2. A first-time user with no Constellation training opens Sight v5 and can articulate within ~5 seconds what they're looking at (the hard constraint from Eisa's 2026-05-09 directive).
3. All seven modes render correctly on the 7,636-note trial Universe within performance budgets (§10).
4. Mode toggle is stable spatial-memory: the same star sits at the same strata band in every mode.
5. The four constants (radius / size / brightness / red) hold across every mode without exception.
6. Sources field populated for ≥80% of the trial Universe via the classifier-and-approval workflow.
7. Both Tier 1 (bundled) and Tier 2 (optional download) classifier paths work end-to-end.
8. Help docs + User Manual (EN + AR canonical, 13 other locales queued as PJ) describe Sight v5 as it ships.
9. Three-agent audit clean across all three MIGs (MIG-021 / 022 / 023).
10. Eisa confirms across multiple sessions that Sight v5 delivers the at-a-glance promise.

---

## §13 · Glossary

| Term | Definition |
|---|---|
| **Sight v5** | The fifth Sight implementation generation. Star-chart aesthetic, taxonomy-spined, seven modes, strata-as-radius. |
| **Star** | A note rendered as a circular dot on the dome. |
| **Strata band** | A concentric ring on the dome corresponding to one of the eight stratum levels (L1 Datum at rim, L8 Worldview at pole). |
| **Mode** | A wedge-slicing scheme that reorganizes the rim. Seven modes: R / L / T / C / S / A / P. |
| **Wedge** | A radial sector of the dome corresponding to one bucket of the active mode (a month, a library, a stage, a source...). |
| **Universal Epistemic Content Taxonomy** | The cross-civilizational scholarly framework Sight v5 uses as its backend vocabulary. Five branches × eleven sources. See `docs/epistemic-content-taxonomy.md`. |
| **Source** | One of the 11 *masādir al-maʿrifah* / *pramāṇa* drawn from the taxonomy's horizontal axis. Per-note frontmatter field. |
| **Provenance (mode P)** | The Sight mode whose wedges are the 11 sources. |
| **Epistemic Classifier** | The Constellation subsystem that proposes source assignments for user approval. Two tiers: bundled (e5-small) and optional (Qwen3-1.7B). |
| **Source Review panel** | The new sidebar surface where the user approves classifier suggestions. |
| **Tier 1 / Tier 2 classifier** | Bundled vs optional-download classifier paths. See §8. |

---

## §14 · Cross-references

- `docs/epistemic-content-EN.md` — comparative civilizational essay, English (the intellectual case)
- `docs/epistemic-content-AR.md` — Arabic version
- `docs/epistemic-content-taxonomy.md` — formal two-axis taxonomy, bilingual
- `docs/epistemic-content-taxonomy-chart.html` — interactive 5-level chart
- `docs/Sight-vNext-MockA-Dashboard.svg` — Option A (rejected, decision record)
- `docs/Sight-vNext-MockB-Metaphor.svg` — Option B base
- `docs/Sight-vNext-MockB1-Toggle.svg` — **APPROVED visual reference**
- `docs/Sight-vNext-MockB2-Compare.svg` — help-doc teaching diagram
- `docs/360.3D-Concept-Paper-v1.0.md` — companion paper for the per-note diagnostic surface
- `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md` — the LLM research that informed §8
- `lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md` — the implementation Architect doc (companion to this paper)
- `docs/Constellation Orientation & Onboarding v1.76.md` — current orientation; Sight v5 decisions captured in v1.75 + v1.76 preambles
- **Obsoleted (preserved as historical record)**: `Constellation-Sight-Concept-Paper-v1.1.md`, `Constellation-Sight-v3-Concept-Paper-v1.1.md`, `SIGHT-V3-VISUAL-SPEC.md`

---

**End of v2.0.** This paper is the design contract for Sight v5, ratified by Eisa 2026-05-09. The next document in the chain is the **MIG-021 Architect** — the first of three migration cycles that build Sight v5 in phases.

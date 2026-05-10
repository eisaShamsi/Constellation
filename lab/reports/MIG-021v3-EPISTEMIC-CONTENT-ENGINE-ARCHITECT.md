# MIG-021v3 — Constellation Epistemic Content Engine (CECE)
## Cataloger Ensemble Architecture

**Status**: Architect draft, awaiting Boss review.
**Date**: 2026-05-10.
**Supersedes**: `MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md`
**Anchored against**:
- Eisa, *Automatic Classification of Personal Knowledge Notes by Epistemic Source and Content Type* (`epistemic-classifier-paper-EN.md` v1.0, 2026-05-09)
- Four research-agent reports filed in this session (hierarchical text classification SOTA; library-science prior art; local-LLM benchmarks; active-learning + provenance design patterns)
- `docs/sources-of-knowledge-diagram.html` (Eisa-canonical horizontal taxonomy)
- `docs/epistemic-content-taxonomy-chart.html` (Eisa-canonical vertical taxonomy)
- `docs/Constellation-Sight-Concept-Paper-v2.0.md` §7-§9
- All MIG-021v2 commits §1A' through §1F'.b

---

## §0  What changed from v2 — and why

MIG-021v2 framed the work as a three-tier classifier (rules → embeddings → LLM) with confidence routing. That framing inherits the **generic classifier ceiling** documented across NLP and library-science literature: **45–60% leaf-level accuracy** for a 218-leaf, 5-deep, multi-label, bilingual taxonomy on a small local LLM, even with the best engineering. Two structural facts make this ceiling hold:

1. Many sibling distinctions (e.g. `mass-transmission/verbal` vs `mass-transmission/meaning`) are **Bayes-irreducible from text alone** — no model at any size separates them without side-channel data. (Scott et al., EJS 2016, formal proof.)
2. The literature measures *generic* classifiers on text alone. Constellation has primitives (CAE, Lexical Bridge, Living Links graph, frontmatter, capture-time metadata) that **the literature couldn't measure because no public benchmark has them**.

Two cumulative reframings, both Boss-driven, change the architecture:

**Reframing 1 — generic classifier → cataloger algorithm (Boss directive 2026-05-10)**
A classifier asks *"what label is most likely?"* and outputs probabilities. A **cataloger** asks *"how would a trained cataloger navigate the taxonomy and arrive at an assignment for this note?"* — and outputs an **assignment + reasoning trail**. The DDC analogy is exact: a librarian reads structurally (frontmatter → title → body → links), navigates the schedule top-down, applies declarative rules in ambiguous cases, builds the call number compositionally, documents the reasoning. Different optimization target, different output, different failure mode.

**Reframing 2 — single cataloger → cataloger ensemble (Boss directive 2026-05-10)**
Multiple methodologically distinct catalogers reading through different lenses produce uncorrelated errors. Disagreement *itself* is a signal — it identifies the Bayes-irreducible cases and surfaces them to the user with all reasoning trails visible. Library-science precedent: LC, NLM (PubMed/MeSH), OCLC all use second-cataloger review as quality control. ML precedent: Snorkel (Stanford → Apple/Google/Intel production), Mixture of Experts, BioASQ winners stack methods.

The combination is the **Cataloger Ensemble Architecture (the implementation pattern)** of the **Epistemic Content Engine (the system as a whole)**.

This document defines the architecture. The Plan is a separate document and comes after Boss approval here.

---

## §1  Vocabulary

| Term | Definition |
|---|---|
| **Constellation Epistemic Content Engine (CECE)** | The system as a whole that classifies notes along Source + Content Type axes. Boss-named 2026-05-10. |
| **Cataloger** | A single classification process with a methodologically distinct lens (Linguistic, Structural, Graph, Semantic, Reasoning, User-Authority). Each cataloger reads the note through a specific Constellation primitive. |
| **Cataloger Ensemble** | The synthesis layer that combines multiple cataloger outputs into a final assignment + composite reasoning trail. |
| **Reasoning Trail** | The structured output a cataloger produces: assignment + rules fired + alternatives considered + per-alternative rejection reason + confidence band. |
| **Schedule Navigation** | Top-down traversal of the taxonomy: pick the parent class first, then descend to the leaf only when the parent is settled. (DDC analogy.) |
| **Cataloger Rules** | Declarative principles that govern ambiguous cases (Rule of Authority, Rule of Application, Rule of Three, Rule of Side-channel Preference, Rule of Authority Control). |
| **Authority** | A higher-priority signal that overrides lower-priority ones (frontmatter > body text; user-supplied > AI-inferred; explicit citation > implicit reference). |
| **Synthesis Layer** | The meta-component that combines per-cataloger reasoning trails into a single ensemble assignment. Two design options: weighted-vote vs Snorkel-style learned generative model. |
| **Confidence Regime** | One of three states the synthesis layer reports: **Unanimous** (all catalogers agree → silent accept), **Strong Majority** (4 of 5 agree → accept with dissent surfaced as "see also"), **Split** (3-2 or worse → engine refuses to assign and asks the user). |
| **Sibling Disambiguation** | The user-asks pattern triggered when catalogers split between two siblings. The UI shows all reasoning trails side by side; the user picks; the correction logs which catalogers were right and which misfired. |
| **Bayes-irreducible Pair** | A sibling pair (e.g. `mass-transmission/verbal` vs `mass-transmission/meaning`) where text alone cannot disambiguate. The ensemble detects these by inter-cataloger split and surfaces them rather than guessing. |
| **CAE** | Constellation Arabic Engine — root-pattern morphology, ḥamza unification, broken-plural recognition. (Existing primitive.) |
| **Lexical Bridge** | Constellation's cross-lingual term equivalence map. (Existing primitive.) |
| **Living Links** | Constellation's typed semantic graph (supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of). (Existing primitive, P0–P1 shipped.) |
| **Per-Library Calibration** | Each cataloger's reliability score is tracked per Library; the synthesis layer weights catalogers based on their per-Library track record. |

---

## §2  The Catalogers

The CECE runs six catalogers per note. Each one is a self-contained classification process with a defined lens, inputs, output schema, and reliability profile. Each cataloger leverages exactly one Constellation primitive (or the absence of it, in the User-Authority case).

Common output schema for every cataloger:

```json
{
  "cataloger": "linguistic",
  "voiced_opinion": true,
  "horizontal": [{"id": "testimony/scriptural", "primary": true, "weight": 0.85}],
  "vertical": [{"id": "semantic-contents/proposition", "primary": true, "weight": 0.78}],
  "reasoning": "CAE-normalized chain marker حدثنا (root ح-د-ث) detected; ...",
  "rules_fired": ["citation_marker", "root_pattern_match"],
  "alternatives_considered": [
    {"axis": "horizontal", "id": "mass-transmission/verbal", "rejected_because": "no تواتر marker"}
  ],
  "self_reported_confidence": "high" | "medium" | "low" | "abstain"
}
```

`voiced_opinion: false` means this cataloger had no signal from its lens (e.g. Graph Cataloger on an orphan note, User-Authority Cataloger on a note with no frontmatter). Catalogers that abstain do not contribute to the synthesis.

### §2.1  Linguistic Cataloger

**Lens**: CAE morphology + Lexical Bridge.
**Inputs**: note text, CAE-normalized roots/patterns, Lexical Bridge equivalents for any matched terms.
**Outputs assignment when**: a root-pattern match against the lexicon gives high confidence (e.g. `قياس` → root ق-ي-س, pattern fiʿāl → `comparison/ratio-legis`); cross-civilizational equivalent matched via Bridge.
**Strong on**: technical Arabic / Sanskrit / Greek vocabulary; root-aware (avoids false positives from string-similar non-epistemic terms — `قياس` measurement vs `قياس` analogy disambiguated by surrounding root context).
**Weak on**: pure free-form prose without keyword anchors.
**Latency**: microseconds.
**Existing code that becomes substrate**: `src-tauri/src/sources/horizontal_taxonomy.rs` (data), `src-tauri/data/sources_lexicon.json` (lexicon — written in §1G2'), CAE crate (existing).

### §2.2  Structural Cataloger

**Lens**: note structure — frontmatter, citation patterns, headings, blockquotes, code blocks, link types, equation markers.
**Inputs**: parsed note structure (frontmatter dict, citation regex matches, structural counts).
**Outputs assignment when**: structural patterns provide strong signal (ISBN/DOI → `testimony/scriptural`; equations + units → `semantic-contents/fact`; stance markers like "I doubt" → `epistemic-states/doubt`).
**Strong on**: notes with rich metadata, citations, formal structure.
**Weak on**: free-form notes with no structural markers.
**Latency**: microseconds.
**Existing code that becomes substrate**: `regex_horizontal` block in `sources_lexicon.json`, `extract_sources` / `extract_content_type` from `sources/mod.rs`.

### §2.3  Graph Cataloger

**Lens**: Living Links typed neighborhood — what classifications similar-via-link neighbors got.
**Inputs**: list of typed neighbors (via Living Links IPC), each neighbor's current Source + Content Type assignments.
**Outputs assignment when**: a meaningful majority of typed neighbors (especially `derives-from`, `part-of`, `exemplifies`) share a classification.
**Strong on**: densely-linked notes; authority-control via consensus.
**Weak on**: orphan or new notes (`voiced_opinion: false` when degree < 2).
**Latency**: milliseconds (one DB query for typed neighbors + their current assignments).
**Net-new code**: a new IPC that returns `{neighbor_path, link_type, sources, content_type}` for a given note path; the cataloger logic that synthesizes a vote from this list.

### §2.4  Semantic Cataloger

**Lens**: Tier 2 multilingual embedding similarity to neighbors via kNN-blend over the user's already-classified vault.
**Inputs**: note text → embedding; cosine top-k against the per-Library exemplar memory (existing `note_embeddings` table can host this); current assignments of those k neighbors.
**Outputs assignment when**: top-k neighbors agree.
**Strong on**: notes similar to many already-classified ones.
**Weak on**: novel territory; cold-start at zero corrections.
**Latency**: ~30 ms (existing tier1_embedding path).
**Existing code that becomes substrate**: `src-tauri/src/classifier/tier1_embedding.rs` — repurposed from "classify against 274 cached candidate vectors" to "find k nearest classified notes in the vault." Schema for storing exemplar memory already exists in `note_embeddings`.

### §2.5  Reasoning Cataloger

**Lens**: Tier 3 LLM with the full schedule + cataloger rules in the prompt.
**Inputs**: note text, frontmatter, optional list of unresolved candidates from cheaper catalogers.
**Outputs assignment when**: invoked. Always voices an opinion (LLM doesn't abstain).
**Two-track Tier 3** (Boss decision, see §8.2):
- **Local Qwen3-4B-Instruct-2507 Q5_K_M** — privacy default, ~1.5s per note, no API cost. The +10–15pp leaf-accuracy gain over 1.7B is sourced (Few-shot Dilemma + distil-labs benchmark).
- **Cloud frontier (Claude / GPT-4)** — opt-in per Library or per note. Higher accuracy ceiling; user gives explicit consent because notes leave the device.
**Schedule navigation in the prompt**: the prompt presents the parent classes first; if the LLM's parent confidence is high, a second prompt with only that parent's children is sent. Top-down decomposition consistently outperforms flat at depth ≥3 (KG-HTC arXiv 2505.05583).
**GBNF grammar constraint**: output schema is grammar-constrained to valid taxonomy IDs only. Small accuracy win (+1 to +3pp per ACL 2025 industry paper); large operational reliability win (no parse failures, no hallucinated labels).
**Strong on**: novel cases, cross-civilizational reasoning, the irreducible residual.
**Weak on**: fast/easy cases (overkill — runs only when cheaper catalogers disagree or are silent).
**Latency**: 1.5 s (local 4B) or 1.5 s (cloud frontier).
**Net-new code**: llama.cpp integration (Tauri side), Qwen3-4B GGUF download manager, GBNF grammar generator from the taxonomy IDs, prompt builder with schedule + rules + few-shot exemplars, optional cloud-LLM IPC adapter.

### §2.6  User-Authority Cataloger

**Lens**: frontmatter + capture-time fields ONLY.
**Inputs**: parsed YAML frontmatter (`sources:`, `content_type:`, `acquisition_method:`, `tradition:`, `source_citation:`, `confidence:`).
**Outputs assignment when**: frontmatter has explicit values.
**Strong on**: notes with user-supplied provenance (the CIP precedent — author > cataloger at depth).
**Weak on**: notes without frontmatter (`voiced_opinion: false` rather than guess).
**Latency**: microseconds.
**Special role**: when this cataloger voices an opinion, the synthesis layer treats it as **absolute authority** — no other cataloger overrides it. The other catalogers still run (their reasoning trails are logged for future correction-loop signal), but their output is informational, not authoritative.
**Existing code that becomes substrate**: `extract_sources` / `extract_content_type`, frontmatter precedence already in §1G2' code.

---

## §3  The Synthesis Layer

The synthesis layer takes 1–6 reasoning trails and produces a single ensemble assignment + composite reasoning trail.

### §3.1  Three confidence regimes

The regime is determined by inter-cataloger agreement on the **primary** assignment per axis:

1. **Unanimous** — all voicing catalogers agree on the primary. → High confidence. Accept silently. Composite reasoning trail = union of cataloger reasonings, deduplicated.

2. **Strong Majority** — supermajority (≥4 of 5 voicing catalogers, OR User-Authority voiced) agrees on the primary. → Moderate confidence. Accept with the dissenting cataloger's choice surfaced as "see also." Composite reasoning trail records the dissent.

3. **Split** — 3-2 or worse, OR User-Authority absent and others split close. → **Engine refuses to assign and triggers Sibling Disambiguation**. The UI shows all reasoning trails side by side; the user picks. The correction logs which catalogers were right per axis per leaf — that's gold for active learning.

Thresholds for "voicing" / "agreement on primary" are tunable per Library and start with conservative defaults to be calibrated empirically.

### §3.2  Synthesis design — two options for Boss decision

**Option A: Weighted Vote.**
Each cataloger has a fixed reliability weight per axis. Votes are summed; primary is the leaf with highest weighted sum. Simple, transparent, no learning required. Works on Day 1 with hand-tuned weights.

**Option B: Snorkel-style Learned Generative Model.**
A small generative model learns each cataloger's reliability per axis per Library by observing agreement patterns over time. More accurate eventually, requires correction-log data to train, opaque in early days.

**Recommendation**: ship Option A on Day 1 (hand-tuned weights based on agent-reported strengths); add Option B in MIG-022 once we have ≥3 months of correction-log data per Library. This matches Snorkel's own deployment pattern (Apple, Google, Intel all started with hand-weighted before learned synthesis).

### §3.3  Per-Library Reliability Tracking

Each cataloger's reliability is tracked per axis per Library:

```
correction_log entry → updates per-(cataloger, axis, Library) accuracy histogram
                    → feeds the synthesis weights
```

Why per-Library: the Graph Cataloger is unusually accurate on a Library with a dense Living Links graph (Eisa's hadith Library) and unusually weak on a Library that's mostly orphan notes (a fresh Library). The Linguistic Cataloger is unusually accurate on Arabic-heavy Libraries and weaker on English-only ones. Per-Library calibration captures this.

Storage: per-Library JSON file at `<library>/.constellation/cataloger_reliability.json`. Append-only updates; never deletes; survives backups. Schema:
```json
{
  "linguistic": {"horizontal": {"correct": 412, "wrong": 38}, "vertical": {...}},
  "structural": {...},
  "graph": {...},
  ...
}
```

### §3.4  Composite Reasoning Trail Schema

The synthesis layer's output:
```json
{
  "horizontal": {
    "primary": "testimony/scriptural",
    "secondary": ["testimony/authoritative"],
    "regime": "unanimous" | "strong_majority" | "split",
    "see_also": [],
    "needs_user_disambiguation_between": null
  },
  "vertical": {
    "primary": "semantic-contents/proposition",
    "regime": "strong_majority",
    "see_also": ["semantic-contents/concept"],
    "dissenter": "graph"
  },
  "catalogers_voiced": ["linguistic", "structural", "semantic", "reasoning"],
  "catalogers_silent": ["graph", "user_authority"],
  "synthesis_method": "weighted_vote",
  "composite_reasoning": "...",
  "per_cataloger_trails": [
    {"cataloger": "linguistic", "trail": "..."},
    ...
  ]
}
```

This is what gets persisted, surfaced to the user, and logged for active learning.

---

## §4  Cataloger Rules — the codified discipline

Real DDC catalogers operate by documented rules (Rule of Three, Rule of Application, Rule of Zero, etc.). Our catalogers operate by analogous rules, encoded once and consulted by every cataloger that needs them.

### §4.1  The five rules

**Rule of Authority** — Frontmatter > body text > AI-inferred. The User-Authority Cataloger encodes this absolutely. Other catalogers respect it implicitly (they never voice an opinion that contradicts an explicit frontmatter value; if they would, they instead voice agreement with the frontmatter and append "via [other-evidence]" to their reasoning).

**Rule of Application** — Classify by what the note USES the concept for, not what it merely mentions. A note that says *"al-Bukhari is the source of many authentic hadiths"* is about al-Bukhari (who); a note that says *"حدثنا الإمام البخاري في صحيحه أن النبي ﷺ قال..."* is testimony/scriptural transmitting al-Bukhari's content (the use). The Linguistic and Structural Catalogers must distinguish use from mention; the Reasoning Cataloger should be primed for this.

**Rule of Three** — When 3+ candidates have similar weight at a depth, ascend one level and assign the parent rather than guess at the leaf. Encoded in each cataloger's "self_reported_confidence: low" path (returns parent + low confidence rather than a guessed leaf).

**Rule of Side-channel Preference** — When prose evidence and side-channel evidence (Living Links typed neighbors, CAE morphology, Lexical Bridge equivalents) disagree, prefer the side-channel. Side channels are harder to fake; prose can be misleading.

**Rule of Authority Control** — When neighboring notes (via Living Links) have a consensus classification and the current note is in their semantic neighborhood, prefer to align with them unless explicit reason otherwise. Mirrors LCSH authority files. Encoded in the Graph Cataloger.

### §4.2  Rule encoding

Rules are NOT hardcoded into each cataloger. They live as a single declarative JSON file (`src-tauri/data/cataloger_rules.json`) that catalogers consult. New rules can be added without touching cataloger code; existing rules can be refined without breaking compilation.

This matches the way DDC's *Manual* documents rules separately from the schedules.

---

## §5  Schedule Navigation — top-down per-axis

Every cataloger that produces a leaf assignment first navigates the parent class.

### §5.1  Two-step decomposition

1. **Step 1**: classify into the top-level branch (5 vertical branches; 11 horizontal parents). High confidence is achievable here per all the research (top-of-axis: 80–94%).
2. **Step 2**: only if step 1 is settled at high confidence, descend into the parent's children. The classifier's candidate set in step 2 is restricted to the chosen parent's children — making the classification problem N-way over ~5 candidates instead of N-way over 270.

### §5.2  Depth budget per axis

Each cataloger respects a depth budget:
- **L1 (top-of-axis)**: confident assignment expected in ≥85% of notes
- **L2 (sub-categories)**: confident assignment in ≥70% of notes; rest at L1 with `descend_uncertain: true` flag
- **L3+ (deep leaves)**: confident assignment in ≥50% of notes that have already passed L2; rest at L2 with `descend_uncertain: true`

The User-Authority Cataloger ignores the depth budget — if the user wrote `epistemic-states/certainty/religious/ʿilm-al-yaqīn` in frontmatter, that's the assignment, full stop.

### §5.3  The "abstain at depth" pattern

When a cataloger's depth-budget check fails, it returns the parent class + `descend_uncertain: true`. The synthesis layer treats this as a vote for the parent. If 3+ catalogers vote parent + descend_uncertain, the assignment is the parent and the engine surfaces the leaf candidates as "see also."

This is the **honest abstention** pattern from your paper §3.4 ("calibrated confidence and the right to abstain") — applied per-depth-level.

---

## §6  Repositioning what we shipped (v2 → v3)

Nothing from §1A'–§1F'.b is thrown away. Each component is repositioned:

| Shipped (v2) | Status | New role (v3) |
|---|---|---|
| §1A' schema (`note_meta.sources`, `note_meta.content_type`, `sources_suggestions`) | Preserved | CECE substrate; `sources_suggestions` becomes the synthesis-layer output store. |
| §1A' horizontal + vertical taxonomy data | Preserved | Schedule reference for all catalogers. |
| §1B' `tier1_embedding.rs` (e5-small ONNX, 274 cached candidates) | Repurposed | Becomes the **Semantic Cataloger**: reused for embedding, but cosine-similarity target shifts from "274 cached candidates" to "k nearest already-classified notes in the per-Library exemplar memory." |
| §1B' tier-aware fallback at 0.55 | Repurposed | Becomes one rule in the Linguistic Cataloger. |
| §1C' `SourceReviewPanel.svelte` (dual-axis cards) | Upgraded | Becomes the ensemble's review UI. Cards now render the composite reasoning trail; on Split regime, surfaces all cataloger trails side by side. |
| §1C' `TaxonomyTreePicker.svelte` | Preserved | Used by PropertyEditor + Source Review. No change. |
| §1D' PropertyEditor inline pickers | Preserved | Hardens the **User-Authority Cataloger**'s input path. |
| §1E' right-click context action | Preserved | Triggers ensemble run on a single note. |
| §1F' background scan + status-bar strip + Settings toggle | Preserved | Runs the ensemble across the vault. Strip behavior unchanged. |
| §1F'.b Approve All / Reject All bulk actions | Preserved | Approve All now writes the synthesized ensemble assignment (not just top-3 raw suggestions). |
| §1G' i18n full pass | Preserved | EN+AR coverage stays current. |
| §1G2' code (written, NOT committed) — `tier1_rules.rs`, `sources_lexicon.json`, `correction_log.rs` | Repositioned | Splits across **Linguistic Cataloger** (CAE+lexicon path), **Structural Cataloger** (regex+frontmatter path), and **correction log** (preserved as-is — substrate for active learning). |
| Frontend tier badge (`T1` / `T2`) | Replaced | Becomes a per-cataloger badge cluster: `LING ✓ STRUCT ✓ GRAPH – SEM ✓ REASON ✓ AUTH –` (one badge per cataloger; ✓ = voiced + agreed with primary, – = silent, ✗ = dissented). |

Net: zero rollbacks, zero schema changes, no commits to undo. The §1G2' code on disk gets reorganized rather than thrown away.

---

## §7  Net-new components

| Component | Purpose | Approximate size |
|---|---|---|
| **Graph Cataloger** | Reads Living Links typed neighborhood + their classifications | New IPC + new Rust module |
| **User-Authority Cataloger** | Formal abstain-or-echo behavior on frontmatter alone | Small Rust module (mostly already exists in §1G2' frontmatter precedence code) |
| **Reasoning Cataloger** | Tier 3 LLM with schedule + rules in prompt; two-track local/cloud | Substantial: llama.cpp wrapper, GBNF generator, prompt builder, optional cloud adapter |
| **Synthesis Layer** | Weighted-vote synthesis (Day 1); learned synthesis (MIG-022) | Medium Rust module + per-Library reliability JSON |
| **Cataloger Rules JSON** | Declarative encoding of the five rules + future additions | Data file + loader |
| **Composite Reasoning Trail Schema** | Persisted form of ensemble output | Schema + serde derives + DB column |
| **Sibling Disambiguation UI** | User-facing UX when ensemble hits Split regime | Svelte component (extends Source Review) |
| **Per-Library Reliability Tracking** | Updates cataloger weights from correction log | Module + JSON file per Library |
| **Ensemble Orchestration** | Decides which catalogers run in what order with what timeouts | Coordinator module |
| **Reasoning Trail Renderer** | UI component that pretty-prints reasoning trails (always-visible vs on-disagreement-only is a Boss decision) | Svelte component |

---

## §8  Design tradeoffs — Boss decisions

These are the open architectural questions. Each has a recommended default; Boss override is welcome.

### §8.1  Number of catalogers — five or six?

**Six** (as designed): Linguistic, Structural, Graph, Semantic, Reasoning, User-Authority.
**Five** (alternative): drop User-Authority; merge its behavior into the synthesis-layer precedence rule.

**Recommendation: SIX.** User-Authority as a distinct cataloger with a reasoning trail of its own (`"Set in frontmatter on 2026-05-10"`) is auditable in a way that "synthesis-layer precedence" is not. Cost is negligible (microseconds, no model).

### §8.2  Tier 3 strategy — local-only, cloud-only, or both?

Three options:

**(a) Local-only** — Qwen3-4B Q5_K_M, no cloud option. Maximum privacy. Accuracy ceiling: per agent 3, leaf-level 65–80% in the user's active subtrees. Bundle download ~2.5 GB.

**(b) Cloud-only** — Claude / GPT-4 / Gemini via API. Higher accuracy ceiling. Notes leave the device. Cost per scan; user must have API key. Conflicts with CLAUDE.md's "Local-First" principle as the default.

**(c) Both — local default + cloud opt-in per Library or per note.** Privacy floor = local. Accuracy ceiling = cloud, when user explicitly opts in. Matches CLAUDE.md's "Local-First by Default; Cloud Opt-in" pattern.

**Recommendation: (c).** This honors the architectural principle without sacrificing the accuracy ceiling for users who want it.

### §8.3  Synthesis design — weighted vote vs Snorkel-style learned

Already covered in §3.2. **Recommendation: weighted vote on Day 1; Snorkel-style learned synthesis in MIG-022.**

### §8.4  Per-Library calibration — Day 1 or deferred?

**Day 1**: ship the per-Library reliability JSON + tracking from the start. Empty file means uniform weights; populates as corrections arrive.
**Deferred**: ship uniform weights only; add per-Library calibration in MIG-022.

**Recommendation: Day 1**, but as an empty file with uniform default weights. Tracking starts immediately; weights only change once enough corrections accumulate.

### §8.5  Reasoning trail rendering — always-visible vs on-disagreement-only

**Always-visible**: every Source Review card shows the composite reasoning trail by default; cataloger badges always visible.
**On-disagreement-only**: Source Review card shows composite reasoning trail only when regime is Strong Majority or Split (i.e. when there's something interesting); Unanimous regime shows clean compact card.

**Recommendation: on-disagreement-only by default + Settings toggle to enable always-visible** for users who want auditability. Most cards will be Unanimous; constant reasoning-trail noise hurts review velocity.

### §8.6  Sibling Disambiguation UX — modal, inline, or sidebar?

**Modal**: blocks user until they pick. Bad — "Agent Approval Fatigue" pattern; contradicts the Don't-Modal-Interrupt rule.
**Inline in Source Review**: split cards expand inline showing all reasoning trails; user picks via radio buttons.
**Sidebar queue**: Split-regime cards collect in a separate "Needs your call" section; user works through them at their pace.

**Recommendation: inline in Source Review + sidebar count badge.** Don't move them into a separate UI; just visually distinguish them in the existing queue and surface a count.

### §8.7  Boss-test gate granularity

Three options:

**Per-cataloger gate**: each cataloger ships independently with its own Boss-test gate. Six gates. Maximum verification, slowest cascade.
**Per-axis gate**: catalogers ship in axis bundles (horizontal-axis pass: Linguistic + Structural + Graph + Semantic + Reasoning + User-Authority running on horizontal only; then same for vertical). Two gates.
**Per-synthesis gate**: one Boss-test gate after the full ensemble + synthesis layer is wired. Single gate.

**Recommendation: per-axis gate (two)** + a final integration gate. Each per-axis gate verifies catalogers + synthesis on one axis; final gate verifies dual-axis behavior + Sibling Disambiguation UI.

### §8.8  MIG-022 scope

What gets pushed off into MIG-022 (post §1K' close-out):
- Snorkel-style learned synthesis model
- Vault-specific fine-tuning of cataloger reliability weights
- Active-learning queue diversity constraints (avoid the user being asked about 30 outlier notes in a row)
- kNN-blend exemplar memory growth + pruning
- Cloud-Tier-3 API adapter (if Boss approves §8.2 (c) but wants to ship local-first first)
- Per-cataloger calibration audits surfaced in Settings ("the Linguistic Cataloger has been right 84% of the time on this Library")

---

## §9  Performance budget

Per-note budget for the ensemble running on a single classification request (right-click action OR background scan):

| Cataloger | Latency | Cost | Runs always? |
|---|---|---|---|
| Linguistic | <5 ms | free | yes |
| Structural | <5 ms | free | yes |
| User-Authority | <5 ms | free | yes (abstains if no frontmatter) |
| Graph | ~10–30 ms | free | only if note has ≥2 typed neighbors |
| Semantic | ~30 ms | free | only if cheap catalogers don't reach Unanimous |
| Reasoning (local 4B) | ~1500 ms | free | only if cheaper catalogers split or hit Strong Majority dissent |
| Reasoning (cloud frontier) | ~1500 ms | $0.001–0.005 | only if user opted in AND cheaper catalogers split |

**Cascade pattern**: cheap catalogers (Linguistic, Structural, User-Authority, Graph) run first. If they reach Unanimous, return immediately (~50 ms total). Otherwise Semantic runs. If still not Unanimous, Reasoning runs. The expensive cataloger runs ~10–25% of the time in steady state.

**Vault-scan budget** (7,000 notes):
- Local-only ensemble: ~30 min (Apple Silicon) to ~4 hr (budget x86 CPU)
- Cloud opt-in: similar wall-clock (Reasoning Cataloger latency similar; cloud API can run in parallel)

**Per Performance Rule 1**: ensemble runs on a background thread; never blocks the UI. Typing remains instant during scan. Already enforced by §1F' scan_job pattern.

---

## §10  Invariants — the things that MUST hold

These are the architectural commitments. Every component must respect them; the audit phase verifies them.

1. **User-Authority is absolute** — when User-Authority Cataloger voices an opinion, no other cataloger overrides it. Frontmatter is the single source of truth for that note.
2. **Reasoning trails are never silently discarded** — every cataloger's reasoning is logged, even when the synthesis layer doesn't surface it. The user can audit historical decisions.
3. **Disagreement surfaces** — when catalogers Split, the engine refuses to assign and asks the user. Silent guessing under disagreement is forbidden.
4. **Local-first by default** — Tier 3 cloud is opt-in per Library. Default install never sends notes off-device.
5. **Cataloger errors do NOT propagate** — one cataloger crashing or timing out is isolated; the synthesis layer treats it as `voiced_opinion: false` and proceeds.
6. **Performance Rule 1 preserved** — ensemble never blocks the UI thread. Background scan + per-note classification both run on worker threads.
7. **CAE / Lexical Bridge / Living Links integrity** — these are substrates, not internal-only dependencies. The CECE depends on them but doesn't modify them. If Living Links P2–P5 ships changes, the Graph Cataloger adapts; it doesn't regress the Living Links contract.
8. **Correction log is append-only** — never deletes; survives backups; per-Library; full audit trail forever.
9. **Per-Library calibration is per-Library** — no cross-Library data leakage. Each Library's `cataloger_reliability.json` is private to that Library.
10. **Confidence regime is honest** — "Unanimous" doesn't mean "100% accurate"; it means "the catalogers, reading through different lenses, all agreed." The user manual says this plainly.
11. **Schedule navigation is mandatory at depth ≥3** — top-down decomposition on every cataloger that targets L3+. Flat 270-way classification is forbidden.
12. **Ensemble timeouts are bounded** — every cataloger has a max latency; if it exceeds, the synthesis layer treats it as `voiced_opinion: false`. The full ensemble per note is bounded at ~3 seconds (local Tier 3) or ~5 seconds (cloud Tier 3 with retry).

---

## §11  Migration path from v2

No rollback required. Migration is purely additive:

1. **§1A'–§1F'.b stay shipped exactly as-is.** Schema unchanged. UI components reused.
2. **§1G2' code (uncommitted) is reorganized** into Linguistic + Structural Cataloger seeds + correction log. The `tier1_rules.rs` file becomes two cataloger modules; the lexicon JSON is unchanged.
3. **New cataloger modules + synthesis layer + IPCs** are added incrementally per the Plan (which follows this Architect doc).
4. **Frontend tier badge changes** from `T1`/`T2` to per-cataloger badge cluster — backward-compatible because old `classifier_tier` field stays in the schema, mapped from synthesis regime.
5. **No DB migration needed** — existing `sources_suggestions` table accommodates the new composite reasoning trail by extending `suggestions_json` to optionally include cataloger trails.

First-boot behavior on a Universe with v2-era classifications: existing `sources_suggestions` rows are interpreted as legacy (no per-cataloger trails); new classifications get full ensemble output. The Source Review panel handles both formats.

---

## §12  Open questions for Boss approval

Each maps to a §8 design tradeoff. Defaults shown; Boss can override any.

1. **Number of catalogers**: SIX (default) vs FIVE.
2. **Tier 3 strategy**: BOTH local + cloud opt-in (default) vs LOCAL-only vs CLOUD-only.
3. **Synthesis design**: WEIGHTED VOTE on Day 1 + Snorkel in MIG-022 (default) vs Snorkel from Day 1 vs weighted-vote permanent.
4. **Per-Library calibration**: DAY 1 with empty file (default) vs DEFERRED to MIG-022.
5. **Reasoning trail rendering**: ON-DISAGREEMENT-ONLY by default + Settings toggle (default) vs ALWAYS-VISIBLE vs ON-DISAGREEMENT-ONLY no toggle.
6. **Sibling Disambiguation UX**: INLINE in Source Review + sidebar count badge (default) vs MODAL vs SEPARATE PANEL.
7. **Boss-test gate granularity**: PER-AXIS (two gates) + final integration (default) vs PER-CATALOGER (six gates) vs PER-SYNTHESIS (one gate).
8. **MIG-022 scope**: as listed in §8.8 (default) — Boss can move items in/out.

Plus three meta-questions:

9. **Do we proceed to Plan-drafting** after Boss reviews this Architect, or are there clarifications first?
10. **Is the cataloger ensemble naming acceptable** ("Linguistic / Structural / Graph / Semantic / Reasoning / User-Authority Cataloger"), or do you want different names that read better to non-technical users?
11. **Top-level system name** — Boss-approved 2026-05-10: **Constellation Epistemic Content Engine (CECE)**.

---

## §13  What this Architect does NOT decide

Out of scope for this document — left for the Plan or for later MIGs:

- Specific phase ordering and Boss-test gate placement (Plan)
- Implementation details for each cataloger (Plan / Build)
- LLM prompt design specifics (Build, with examples in Plan)
- GBNF grammar generation specifics (Build)
- UI mockups for Sibling Disambiguation (Plan, after Boss approves §8.6)
- Help-doc + User Manual rewrites (§1I' equivalent in the new Plan)
- Concept Paper amendments (close-out phase)
- Orientation v-bump cadence (close-out phase)

---

## §14  Appendix: where each research finding lands

For traceability — every substantive claim in this document is anchored against one of the four agent reports or your paper. Listed once for the audit:

| Architectural decision | Sourced from |
|---|---|
| Top-down schedule navigation outperforms flat at depth ≥3 | Agent 3 — KG-HTC arXiv 2505.05583; Agent 1 — LSHTC literature |
| Capture-time provenance lifts F1 by 7–15% | Agent 4 — MATCH paper, WWW 2021, arXiv 2102.07349 |
| Local 4B (not 1.7B) for Tier 3 | Agent 3 — Few-shot Dilemma paper + distil-labs benchmark; Qwen3 quantization study |
| GBNF grammar constraint helps closed-set classification | Agent 3 — ACL 2025 industry paper |
| Inter-cataloger agreement at depth is structurally low even for humans | Agent 2 — Funk & Reid 1983 (33.8%); Tonta (14%); PubMed MTI subheading 48%/30% |
| Snorkel-style synthesis lifts F1 by 3–15pp | Agent 4 — Snorkel literature, PMC6436830 |
| Hierarchical correction propagation formalization | Agent 4 — Springer 2014 chapter |
| Bayes-irreducible cases require side-channel data | Agent 1 — Scott et al., EJS 2016, formal proof |
| Multi-label is favored over single-label for retrieval F1 | Agent 2 — MDPI 2023 review |
| Author-supplied (CIP) outperforms post-hoc cataloging at depth | Agent 2 — Mysore vs LC, Malaysian NLM CIP studies |
| User-Authority Cataloger as absolute precedence | Eisa paper §3.5 + §4.1; Agent 2 CIP precedent |
| Per-Library calibration as the right unit | Agent 4 — single-user systems; Eisa paper §6.3 #3 |
| Inline UX, never modal, for confirmation | Agent 4 — Gmail Smart Compose model; Agent Approval Fatigue literature |

Engineering inferences NOT sourced from research (flagged as such):
- The exact six-cataloger split (Linguistic / Structural / Graph / Semantic / Reasoning / User-Authority) — Architect engineering judgment, anchored against the paper §4 three-tier framing extended.
- Specific latency budgets per cataloger — extrapolated from existing §1B' / §1F' timings, not measured for the proposed catalogers.
- The 10–25% Reasoning-Cataloger invocation rate — depends on cheaper-cataloger agreement rates; not measured. Will be empirical.

End of Architect.

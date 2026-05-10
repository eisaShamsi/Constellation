# MIG-021v3 — Constellation Epistemic Content Engine (CECE)
## Build Plan — Cataloger Ensemble Architecture

**Status**: drafted, awaiting Boss approval to start the cascade.
**Date**: 2026-05-10.
**Architect**: `lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md` (Boss-approved 2026-05-10 with all eight defaults).
**Closes**: MIG-021v2 (§1A'–§1G2' shipped or substrate; commit `55f5877`).

---

## §0  Locked decisions (per Architect §8 + §12, Boss-approved)

These decisions govern the entire cascade. Plan steps reference them but do NOT re-decide them.

1. **Six catalogers**: Linguistic, Structural, Graph, Semantic, Reasoning, User-Authority.
2. **Tier 3 strategy**: LOCAL-ONLY. Qwen3-4B-Instruct-2507 Q5_K_M via llama.cpp. No cloud track, no opt-in. Boss directive 2026-05-10: notes never leave the device. (Supersedes original Architect §8.2 "BOTH" default.)
3. **Synthesis design**: weighted vote on Day 1; Snorkel-style learned synthesis deferred to MIG-022.
4. **Per-Library calibration**: Day 1 with empty `cataloger_reliability.json` per Library; uniform weights until corrections accumulate.
5. **Reasoning trail rendering**: on-disagreement-only by default; Settings toggle to enable always-visible.
6. **Sibling Disambiguation UX**: inline in Source Review + sidebar count badge.
7. **Boss-test gate granularity**: per-axis gates (TWO) + final integration gate (ONE) — three total.
8. **MIG-022 scope**: Snorkel learned synthesis + vault fine-tuning + active-learning queue diversity + cloud-Tier-3 production-hardening + per-cataloger calibration audits surfaced in Settings.
9. **System name**: **Constellation Epistemic Content Engine (CECE)**.
10. **Cataloger naming**: Linguistic / Structural / Graph / Semantic / Reasoning / User-Authority (kept; user-facing UI may render shorter labels).

---

## §1  Phase overview

Eleven phases, three Boss-test gates. v2 ship code is preserved as substrate throughout.

| # | Phase | Type | Boss-test gate? |
|---|---|---|---|
| V3-§1 | Synthesis layer + cataloger trait + ensemble orchestrator + rules JSON + reliability skeleton | Foundation | — |
| V3-§2 | User-Authority Cataloger | Cataloger | — |
| V3-§3 | Structural Cataloger (reuses §1G2' regex code) | Cataloger | — |
| V3-§4 | Linguistic Cataloger (reuses §1G2' lexicon + adds CAE + Lexical Bridge) | Cataloger | — |
| V3-§5 | Semantic Cataloger (repurposes §1B' tier1_embedding for kNN-blend on per-Library exemplar memory) | Cataloger | — |
| V3-§6 | Graph Cataloger (new IPC for typed-neighbor lookup) | Cataloger | — |
| V3-§7 | Reasoning Cataloger (local Qwen3-4B Q5_K_M + GBNF + prompt builder + schedule navigation) | Cataloger | — |
| V3-§8 | Source Review UI: composite reasoning trail rendering + per-cataloger badge cluster + Sibling Disambiguation inline + Settings toggle | UI | ✅ **Gate 1: Horizontal axis** |
| V3-§9 | Vertical-axis activation across all catalogers + dual-axis ensemble run | Activation | ✅ **Gate 2: Vertical axis** |
| V3-§10 | Settings → CECE section (model download + cloud opt-in + reasoning trail toggle + per-Library calibration view) + i18n EN+AR + Help docs + User Manual chapter | UI / docs | — |
| V3-§11 | /simplify + 3-agent audit + Concept Paper v2.1 + orientation v1.84 close-out | Audit / close-out | ✅ **Gate 3: Final integration** |

13 other locales for i18n queued as PJ per existing convention.

---

## §2  V3-§1 — Foundation: Synthesis layer + cataloger trait + orchestrator + rules JSON + reliability skeleton

**Goal**: scaffolding everything else hangs off. No catalogers yet; just the contract every cataloger will implement, the synthesis layer that combines them, the ensemble orchestrator that runs them in cascade, and the per-Library reliability storage. cargo-check clean; no user-visible changes.

**Files touched**:

- **NEW** `src-tauri/src/cece/mod.rs` — top-level module `cece` (Constellation Epistemic Content Engine). Re-exports submodules.
- **NEW** `src-tauri/src/cece/cataloger.rs` — defines `trait Cataloger`:
  ```rust
  pub trait Cataloger: Send + Sync {
      fn name(&self) -> &'static str;
      fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail>;
      fn supported_axes(&self) -> &[Axis];
  }
  pub struct CatalogerContext {
      pub note_path: String,
      pub content: String,
      pub frontmatter_sources: Vec<String>,
      pub frontmatter_content_type: Vec<String>,
      // Lazy-loaded as needed by individual catalogers:
      pub typed_neighbors: OnceCell<Vec<TypedNeighbor>>,
      pub cae_normalized: OnceCell<CaeNormalizedText>,
      // ...
  }
  pub struct ReasoningTrail {
      pub cataloger: String,
      pub voiced_opinion: bool,
      pub horizontal: Vec<AxisAssignment>,
      pub vertical: Vec<AxisAssignment>,
      pub reasoning: String,
      pub rules_fired: Vec<String>,
      pub alternatives_considered: Vec<RejectedAlternative>,
      pub self_reported_confidence: Confidence,
  }
  pub enum Axis { Horizontal, Vertical }
  pub enum Confidence { High, Medium, Low, Abstain }
  ```
- **NEW** `src-tauri/src/cece/orchestrator.rs` — runs the six catalogers in cost order (cheap first). Skips Reasoning Cataloger if cheaper catalogers reach Unanimous. Bounded per-cataloger timeouts. Returns `Vec<ReasoningTrail>` for the synthesis layer to combine.
- **NEW** `src-tauri/src/cece/synthesis.rs` — weighted-vote synthesis:
  ```rust
  pub fn synthesize(
      trails: &[ReasoningTrail],
      reliability: &ReliabilityProfile,
  ) -> CompositeAssignment;
  pub struct CompositeAssignment {
      pub horizontal: AxisDecision,
      pub vertical: AxisDecision,
      pub composite_reasoning: String,
      pub catalogers_voiced: Vec<String>,
      pub catalogers_silent: Vec<String>,
      pub per_cataloger_trails: Vec<ReasoningTrail>,
  }
  pub struct AxisDecision {
      pub primary: Option<String>,
      pub secondary: Vec<String>,
      pub regime: ConfidenceRegime,
      pub see_also: Vec<String>,
      pub needs_user_disambiguation_between: Option<Vec<String>>,
      pub dissenter: Option<String>,
  }
  pub enum ConfidenceRegime { Unanimous, StrongMajority, Split }
  ```
- **NEW** `src-tauri/src/cece/reliability.rs` — per-Library reliability JSON read/write:
  ```rust
  pub struct ReliabilityProfile {
      // (cataloger, axis) → (correct, wrong, total)
      pub stats: HashMap<(String, Axis), AccuracyHistogram>,
  }
  pub fn load_or_default(library_path: &str) -> ReliabilityProfile;
  pub fn record_correction(library_path: &str, cataloger: &str, axis: Axis, was_correct: bool);
  pub fn weight_for(profile: &ReliabilityProfile, cataloger: &str, axis: Axis) -> f32;
  ```
  Storage: `<library>/.constellation/cataloger_reliability.json`. Empty file at first; uniform weights returned. Schema versioned for future Snorkel migration.
- **NEW** `src-tauri/src/cece/rules.rs` — loads `data/cataloger_rules.json` once via `OnceLock`. Defines:
  ```rust
  pub enum RuleId { Authority, Application, Three, SideChannel, AuthorityControl }
  pub fn rule(id: RuleId) -> &'static RuleSpec;
  ```
- **NEW** `src-tauri/data/cataloger_rules.json` — declarative encoding of the five Architect §4 rules. Each rule has `id`, `name`, `description`, `applies_to_catalogers`, `signal_priority`. Editable without touching Rust code.
- **EDIT** `src-tauri/src/lib.rs` — `pub mod cece;` + `.manage(cece::synthesis::SynthesisState::new())` if any state needed.

**Key implementation notes**:
- The orchestrator runs cheap catalogers concurrently (rayon), then evaluates whether to invoke Reasoning Cataloger. Bounded by `tokio::time::timeout` (e.g. 100 ms cheap, 3 s total without LLM, 5 s total with LLM).
- One cataloger panicking does not kill the ensemble — caught at the orchestrator and treated as `voiced_opinion: false`.
- Per Performance Rule 5: zero allocations in the hot path beyond the trail itself.

**Self-verification**:
- `cargo check` clean on the workspace.
- `cargo test cece::tests` covers: orchestrator timeout, panic isolation, weighted-vote synthesis with toy trails, reliability load/save round-trip, rules JSON parse.

---

## §3  V3-§2 — User-Authority Cataloger

**Goal**: shipping cataloger #1. Frontmatter-only; absolute precedence per Architect §10 invariant 1.

**Files touched**:
- **NEW** `src-tauri/src/cece/catalogers/user_authority.rs` — implements `Cataloger`:
  - `classify`: if frontmatter sources/content_type non-empty + valid IDs → return high-confidence ReasoningTrail with rule_fired = [`Rule of Authority`]; else `voiced_opinion: false`.
  - Reasoning string: `"Set in frontmatter on {date} (manual)"`.
- **EDIT** `src-tauri/src/cece/orchestrator.rs` — register the cataloger, weight it absolute (0.99) so synthesis treats it as override.
- **EDIT** `src-tauri/src/cece/synthesis.rs` — special-case branch: if User-Authority voiced, skip the vote and pass through.

**Self-verification**:
- `cargo test cece::catalogers::user_authority::tests` — covers: empty frontmatter abstains; populated frontmatter wins; invalid IDs in frontmatter dropped (defense in depth).

---

## §4  V3-§3 — Structural Cataloger

**Goal**: cataloger #2. Reuses the regex + structural patterns we wrote in §1G2' (`tier1_rules.rs::regex_horizontal` block) — repackaged as a Cataloger module.

**Files touched**:
- **NEW** `src-tauri/src/cece/catalogers/structural.rs` — implements `Cataloger`:
  - Reads `regex_horizontal` patterns from `data/sources_lexicon.json` (no schema change to that file).
  - Adds new structural detectors: equation markers (`$...$`, `$$...$$`), code blocks (`\`\`\``), heading depth ratio, named-entity density (proper-noun ratio), markdown table presence.
  - Returns top-N per axis with rules_fired = [`citation_marker`, `equation_marker`, `quote_block`, …].
- **EDIT** `src-tauri/src/cece/orchestrator.rs` — register; weight tuned for "Structural is reliable on notes with citations / equations / blockquotes; abstain otherwise."

**Self-verification**:
- `cargo test cece::catalogers::structural::tests` — covers: ISBN → testimony/scriptural; DOI → testimony/scriptural; equation → semantic-contents/fact (where vertical taxonomy fits); blockquote → testimony/direct-witness; empty body abstains.

---

## §5  V3-§4 — Linguistic Cataloger

**Goal**: cataloger #3. Reuses `data/sources_lexicon.json` token-match logic from §1G2'. Adds CAE morphology integration (root-pattern matching) + Lexical Bridge cross-civilizational equivalents.

**Files touched**:
- **NEW** `src-tauri/src/cece/catalogers/linguistic.rs` — implements `Cataloger`:
  - Calls CAE on the note text (or reuses cached CAE-normalized form from `CatalogerContext`).
  - Uses CAE's root output to disambiguate string-similar terms (`قياس` measurement vs `قياس` analogy via root context).
  - Calls Lexical Bridge to map Arabic/Sanskrit terms to their canonical taxonomy ID.
  - Falls back to surface-token lexicon match when CAE/Bridge are silent.
  - Returns top-N per axis with rules_fired = [`root_pattern_match`, `bridge_equivalent`, `surface_token_match`].
- **EDIT** `src-tauri/src/cece/cataloger.rs` — `CatalogerContext.cae_normalized` lazy-load helper.
- **EDIT** `src-tauri/src/cece/orchestrator.rs` — register.

**Key implementation notes**:
- CAE integration requires: identifying which CAE function returns root + pattern for an Arabic term (audit existing `arabic_engine` crate; no new IPC unless Rust-side direct call doesn't exist).
- Lexical Bridge integration requires: identifying the Bridge query function (read-side; no writes to the Bridge from CECE).
- Lexicon JSON `tokens` array stays unchanged; the matching logic gets smarter.

**Self-verification**:
- `cargo test cece::catalogers::linguistic::tests` — covers: `قياس` in epistemological context → comparison/ratio-legis; `قياس` in measurement context → abstain; `pratyakṣa` → perception/external; absent terms → abstain.

---

## §6  V3-§5 — Semantic Cataloger

**Goal**: cataloger #4. Repurposes existing `tier1_embedding.rs` from "classify against 274 cached candidate vectors" to "find k nearest already-classified notes in the per-Library exemplar memory." kNN-blend pattern from Agent 4 research.

**Files touched**:
- **NEW** `src-tauri/src/cece/catalogers/semantic.rs` — implements `Cataloger`:
  - Embed the note text (existing e5-small ONNX path).
  - Cosine-top-k against per-Library exemplar memory (k=5 default).
  - Vote: each neighbor contributes its current Source + Content Type assignments weighted by cosine similarity.
  - Returns top-N per axis with rules_fired = [`semantic_neighbor_consensus`] + reasoning includes the neighbor paths.
- **NEW** `src-tauri/src/cece/exemplar_memory.rs` — manages per-Library exemplar memory:
  - Reuses existing `note_embeddings` table (no schema change).
  - Adds `note_classifications` view: notes with non-empty `note_meta.sources` AND `note_meta.content_type`.
  - On every classification accept: the note becomes an exemplar.
  - Supports 7,000+ notes per Library efficiently (pre-built ANN index optional; brute-force cosine fine for first ship).
- **EDIT** `src-tauri/src/cece/cataloger.rs` — `CatalogerContext.embedding` lazy-load helper.
- **EDIT** `src-tauri/src/cece/orchestrator.rs` — register; weight tuned for "Semantic is reliable when ≥3 of 5 neighbors agree; abstain on cold-start (<10 classified notes in Library)."

**Self-verification**:
- `cargo test cece::catalogers::semantic::tests` — covers: cold-start abstains; clear neighbor consensus → high confidence; mixed neighbors → low confidence; large vault efficient (synthetic 5000-note benchmark stays under 100 ms).

---

## §7  V3-§6 — Graph Cataloger

**Goal**: cataloger #5. Uses Living Links typed neighborhood as an authority-control reference. **Net-new IPC**: typed-neighbor lookup with their classifications.

**Files touched**:
- **NEW** `src-tauri/src/cece/catalogers/graph.rs` — implements `Cataloger`:
  - Calls new `links_typed_neighbors_with_classifications` IPC (or direct internal function) to fetch `Vec<{neighbor_path, link_type, sources, content_type}>` for the current note.
  - Vote: each typed neighbor contributes weighted by link type (`derives-from` and `part-of` strongest; `contradicts` inverted; `supports` neutral; `causes` and `exemplifies` moderate).
  - Returns top-N per axis with rules_fired = [`typed_neighbor_consensus`] + reasoning includes the typed link paths.
- **EDIT** `src-tauri/src/links/...` (existing Living Links module) — add helper that joins `note_links` with `note_meta.sources` + `note_meta.content_type` for a given note path. Read-only; no schema changes.
- **EDIT** `src-tauri/src/cece/orchestrator.rs` — register; weight tuned for "Graph is reliable when degree ≥ 3 typed neighbors; abstain on orphans."

**Key implementation notes**:
- Living Links P0–P1 already shipped. Schema for `note_links` exists; this cataloger is read-only.
- Per Architect §10 invariant 7: CECE depends on Living Links but doesn't modify it. If Living Links P2–P5 ships changes, the Graph Cataloger adapts.

**Self-verification**:
- `cargo test cece::catalogers::graph::tests` — covers: orphan note abstains; consensus among `derives-from` neighbors → high confidence; mixed typed-neighbor signals → low confidence; `contradicts` link properly inverted.

---

## §8  V3-§7 — Reasoning Cataloger

**Goal**: cataloger #6. The largest phase. **Local-only** Qwen3-4B-Instruct-2507 Q5_K_M via llama.cpp + GBNF grammar constraint + schedule-navigation prompt builder. Notes never leave the device.

**Files touched**:

- **EDIT** `src-tauri/Cargo.toml` — add `llama-cpp-2 = "0.x"` dependency.
- **NEW** `src-tauri/src/cece/catalogers/reasoning.rs` — llama.cpp wrapper (no `_local` suffix needed since there's no other track):
  - Lazy-load Qwen3-4B-Instruct-2507 Q5_K_M GGUF on first use (resident in memory until unload).
  - Two-step prompt: (1) classify into parent class with full top-level taxonomy; (2) if parent confidence high, classify into that parent's children only.
  - GBNF grammar generator: produces grammar that constrains output to valid JSON matching schema `{horizontal: [valid_id], vertical: [valid_id], reasoning: string, alternatives_considered: [{id: valid_id, reason: string}]}`.
  - Returns ReasoningTrail with rules_fired = [`schedule_navigation_top_down`, `gbnf_constrained`].
- **NEW** `src-tauri/src/cece/catalogers/reasoning_download.rs` — resumable HTTP download from `https://github.com/eisaShamsi/Constellation/releases/download/cece-reasoning/qwen3-4b-instruct-2507-q5_k_m.gguf` (~2.5 GB). Progress events on `cece:download` channel. Resume on partial download.
- **NEW** `src-tauri/src/cece/catalogers/reasoning_prompt.rs` — prompt builder + GBNF grammar generator + few-shot exemplar pool (12–18 examples spanning the taxonomy).

**IPCs**:
- `cece_reasoning_download` — kicks off Qwen3-4B download.
- `cece_reasoning_status` — current state (downloaded, downloading%, ready, etc.).
- `cece_reasoning_unload` — frees memory.

**Settings UI for download** lands in V3-§10 (batched with the rest of CECE settings).

**Self-verification**:
- `cargo test cece::catalogers::reasoning::tests` (Tauri-side):
  - GBNF generator produces valid grammar that constrains to taxonomy IDs.
  - Prompt builder produces deterministic output for given inputs.
  - Two-step decomposition works (parent classification → child classification).
- Manual cargo-only verification: Reasoning Cataloger can be invoked, returns valid trail, doesn't crash on malformed model output.

**Note**: Boss-test for the Reasoning Cataloger user flow lands in **V3-§8 (Gate 1)** since the UI to expose it is part of the same gate.

**What's removed from the original Plan draft** (Boss directive 2026-05-10): no `reasoning_cloud.rs` adapter, no per-Library cloud opt-in, no `cece_reasoning_set_cloud_opt_in` IPC, no OpenClaw API integration. Privacy guarantee is absolute: notes never leave the device.

---

## §9  V3-§8 — Source Review UI rewire — ✅ Boss-test gate 1: Horizontal axis

**Goal**: rewire `SourceReviewPanel.svelte` to render composite reasoning trails + per-cataloger badge cluster + Sibling Disambiguation inline UI. **Activate the full ensemble on the horizontal axis**. Cards now show what they used to show plus the new metadata; on disagreement, a structured user-prompt UI appears.

**Files touched**:
- **EDIT** `src-tauri/src/sources/mod.rs` — `sources_suggestions.suggestions_json` extended (backward-compatible) to optionally carry the composite reasoning trail. Existing v2-era rows still readable.
- **EDIT** `src-tauri/src/classifier/mod.rs::classifier_suggest_for_note` — replaces the v2 three-tier path with a CECE orchestrator call. Persists composite reasoning trail to `sources_suggestions`. Returns it to the frontend.
- **EDIT** `src/lib/components/SourceReviewPanel.svelte`:
  - Card header: per-cataloger badge cluster (✓ / – / ✗ per cataloger), replacing the single `T1 / T2` badge.
  - Card body: same dual-axis suggestion display (kept). Adds **on-disagreement-only reasoning trail** section (collapsed by default; expand button if regime ≠ Unanimous; always-visible if Settings toggle on).
  - **NEW** `Sibling Disambiguation` inline UI: when `regime: split` is detected, the card morphs into a "needs your call" form — shows the candidate siblings side by side with each cataloger's reasoning + a primary-pick radio. User pick triggers `cece_resolve_disambiguation` IPC.
  - Sidebar tab title: count badge shows `N pending • {S split}` where S is the count of Split-regime cards needing user attention.
- **NEW** IPC `cece_resolve_disambiguation` — accepts user's pick, writes to frontmatter, clears suggestion, logs correction with full per-cataloger trail (so reliability tracking can update).
- **EDIT** `src/lib/i18n/en.json` + `ar.json` — `cece.cataloger.{linguistic,structural,graph,semantic,reasoning,user_authority}` short labels + `cece.regime.{unanimous,strong_majority,split}` + `cece.disambiguation.{title,prompt,pick,reason_label}` (~20 strings).

**Self-verification**:
- `cargo test`: sources_suggestions schema migration is backward-compatible.
- `npx svelte-check`: clean.
- Manual: open a note, trigger right-click classify, verify card renders + trail shows on Strong-Majority/Split regimes only.

### ✅ Boss-test gate 1 — HORIZONTAL axis

The Boss tests the full ensemble end-to-end on the horizontal axis. (Vertical-axis catalogers exist but are not yet wired through the synthesis layer to surface in the UI.)

**Stages**:

**Stage 0 — verify the binary**.

**Stage 1 — single-cataloger sanity** (verify each cataloger fires correctly):
1. Set `sources: [testimony]` in a note's frontmatter manually. Right-click → Suggest. Card shows `User-Authority ✓` with reasoning *"Set in frontmatter (manual)"*; other catalogers show as `–` (silent because they defer to authority).
2. Create a note with body `"حدثنا الإمام البخاري في صحيحه..."`. Right-click → Suggest. Card shows `Linguistic ✓` (CAE root match on `حدثنا`) and `Structural ✓` (chain marker regex). Suggestion: `mass-transmission/verbal`.
3. Create a note with body `"See ISBN 978-0-12-345678-9 for more details."`. Right-click → Suggest. Card shows `Structural ✓` (ISBN regex). Suggestion: `testimony/scriptural`.
4. Create a note linked via `derives-from` to two existing classified hadith-collection notes. Right-click → Suggest. Card shows `Graph ✓` (typed-neighbor consensus). Suggestion follows the neighbors.
5. Create a note semantically similar to ≥5 already-classified notes. Right-click → Suggest. Card shows `Semantic ✓` (kNN-blend agreement).
6. Create a note with novel content (no frontmatter, no citations, no neighbors, no semantic match). Right-click → Suggest. Card shows `Reasoning ✓` (local Qwen3-4B fires); cheaper catalogers show `–`.

**Stage 2 — confidence regimes**:
1. Note where all voicing catalogers agree → card silently shows top suggestion + Unanimous badge cluster.
2. Note where most catalogers agree but one dissents → card shows Strong-Majority indicator + dissent surfaced in "see also."
3. Note where catalogers split (3-2 between two siblings, e.g. `mass-transmission/verbal` vs `testimony/scriptural`) → Sibling Disambiguation UI appears inline. User picks one; correction is logged.

**Stage 3 — Reasoning trail UX**:
1. Default Settings: Source Review cards show reasoning trails ONLY on Strong-Majority/Split regimes. Unanimous cards are clean.
2. Toggle Settings → "Always show reasoning trails." Now every card shows the trail. Toggle off — back to default.

**Stage 4 — Cloud opt-in**:
1. Settings → CECE → Cloud frontier LLM (per Library) → opt in. Re-classify a note that previously hit local Qwen3-4B. Reasoning Cataloger now uses cloud; reasoning trail shows it.

**Pass criteria**:
- All six catalogers fire correctly when their lens has signal.
- All three confidence regimes surface as designed.
- Reasoning trails appear per Settings.
- Sibling Disambiguation flow works end-to-end with correction logging.
- Reliability JSON updates after corrections (verify file mtime).

---

## §10  V3-§9 — Vertical-axis activation — ✅ Boss-test gate 2: Vertical axis

**Goal**: wire the vertical axis through every cataloger + the synthesis layer. Most cataloger code is already axis-agnostic (it returns both `horizontal` and `vertical` arrays in the trail); this phase verifies vertical-axis behavior end-to-end and adds vertical-specific lexicon entries / regex patterns / typed-neighbor weighting.

**Files touched**:
- **EDIT** `src-tauri/data/sources_lexicon.json` — vertical-axis lexicon expansion. Currently has 6 vertical entries; expand to ~20 covering top-level branches: epistemic-states/{doubt,belief,certainty,knowledge,illusion}, semantic-contents/{concept,proposition,fact,theory,model,information,idea}, sensory-inputs (signal types), symbolic-entities (sign, code), higher-order-constructs (theory, doctrine, worldview).
- **EDIT** `src-tauri/src/cece/catalogers/structural.rs` — vertical-axis structural detectors: theorem/lemma/proof markers → `semantic-contents/proposition`; "I doubt / I believe / متأكد" → `epistemic-states/*`; numerical data + units → `semantic-contents/fact`.
- **EDIT** `src-tauri/src/cece/catalogers/linguistic.rs` — uses existing vertical-axis lexicon entries; also flags Arabic uṣūlī terms that are vertical-axis specific (e.g. `يقين` → certainty).
- **EDIT** `src-tauri/src/cece/synthesis.rs` — verify dual-axis composite assignment correctly handles independent regimes per axis (one axis Unanimous + other axis Split is fine).
- **EDIT** `src/lib/components/SourceReviewPanel.svelte` — verify both axes render side by side with their own regimes + reasoning + Sibling Disambiguation triggers per axis.

**Self-verification**:
- `cargo test cece::tests::vertical_axis` — covers per-cataloger vertical-axis behavior.

### ✅ Boss-test gate 2 — VERTICAL axis

**Stages mirror Gate 1's** but on the vertical axis (epistemic states, semantic contents, sensory inputs, etc.):
- **Stage 1**: each cataloger fires correctly on vertical-axis content (frontmatter `content_type:`, structural markers like "I doubt", semantic neighbors, typed-neighbor consensus, novel content via Reasoning).
- **Stage 2**: confidence regimes work per axis independently — one axis Unanimous + other axis Split.
- **Stage 3**: Sibling Disambiguation triggers per axis (user can resolve horizontal split without touching vertical and vice versa).

**Pass criteria**: vertical axis behaves equivalently to horizontal, with no cross-axis bleed.

---

## §11  V3-§10 — Settings + i18n + Help docs + User Manual

**Goal**: user-facing surfaces around CECE: Settings UI, i18n full pass for all CECE strings, Help docs chapter, User Manual chapter. Honest accuracy framing per Architect §10 invariant 10.

**Files touched**:

**Settings UI**:
- **EDIT** `src/lib/components/SettingsModal.svelte` — new "Constellation Epistemic Content Engine" section under Intelligence:
  - **Reasoning Cataloger model** (local-only — notes never leave the device):
    - Status: "Qwen3-4B Q5_K_M — downloaded / not downloaded / downloading X%."
    - Button: "Download" / "Re-download" / "Unload from memory."
    - Note text: "All inference is on your device. No data is sent over the network."
  - **Reasoning trail visibility**: toggle "Always show reasoning trails (default: on disagreement only)."
  - **Per-Library calibration view** (read-only): expandable section showing each cataloger's accuracy per axis on this Library, with note count it's been evaluated against.
  - **Background scan toggle**: "Auto-classify with CECE on note save / on app start" (default: app start only).

**i18n**:
- **EDIT** `src/lib/i18n/en.json` + `ar.json` — full CECE chrome (~50 strings):
  - Cataloger short labels + descriptions
  - Confidence regime labels
  - Sibling Disambiguation prompts
  - Settings labels
  - Reasoning Cataloger download UI
  - Help text per setting
- 13 other locales queued as PJ.

**Help docs**:
- **NEW** `docs/help.uConstellation.World/CECE/CECE.md` — full help topic:
  - What CECE is (one paragraph plain language)
  - The six catalogers explained for a non-expert user (one paragraph each, no jargon)
  - Three confidence regimes (with examples)
  - Sibling Disambiguation: when and why the system asks
  - How to interpret reasoning trails
  - When to enable cloud frontier
  - Honest accuracy framing per invariant 10
- **NEW** `docs/help.ar/CECE/CECE.md` — Arabic translation of the above.

**User Manual**:
- **EDIT** `docs/User Manual.md` — new chapter "The Epistemic Content Engine" under §3 (Creating and Editing Notes):
  - What gets classified (Source + Content Type axes)
  - What the cards mean
  - When to confirm vs trust
  - The capture-time fields (`tradition`, `acquisition_method`) and why they help
  - Honest accuracy claim: "Top-of-axis (5 content types, 11 source families): accurate. Mid-depth: helpful but imperfect. Deep leaves: suggestion-grade — the system asks when sibling pairs are close. Improves over time as you correct it."
- **EDIT** `docs/help.ar/User Manual.md` — Arabic mirror.

**Self-verification**:
- Switch UI to Arabic; click through every CECE surface; confirm RTL renders correctly + no key-string leaks.
- Switch to a non-EN-non-AR locale (Spanish); confirm graceful English fallback.

---

## §12  V3-§11 — /simplify + 3-agent audit + close-out — ✅ Boss-test gate 3: Final integration

**Goal**: standard /migration close-out + three-agent audit + Concept Paper amendments + orientation v-bump.

**Steps**:

1. **`/simplify` over the full v3 diff** (V3-§1 through V3-§10). Tier-1 findings fixed before audit.

2. **Three audit agents in parallel**:
   - **Invariant agent**: verify the twelve invariants from Architect §10 hold (User-Authority absolute; reasoning trails preserved; disagreement surfaces; local-first by default; cataloger errors don't propagate; Performance Rule 1 preserved; CAE/Bridge/Living Links untouched; correction log append-only; per-Library calibration is per-Library; confidence regime honest; schedule navigation mandatory at depth ≥3; ensemble timeouts bounded).
   - **Drift agent**: any new guards introduced that the system doesn't know about? (per LL-023 lesson)
   - **Migration-path agent**: first-boot, mid-scan restart, downgrade from v3 → v2 (data loss?), per-Library reliability JSON corruption recovery, missing Qwen3-4B GGUF, cloud-API failure during scan, both-axes empty/partial state, v2-era `sources_suggestions` rows displayed correctly in v3 UI.
   - Findings written to `lab/reports/MIG-021v3-CECE-AUDIT.md`.

3. **Fix P0/P1 findings** in close-out commit.

4. **Concept Paper v2.1 amendments**: update `docs/Constellation-Sight-Concept-Paper-v2.0.md` § 7-9 to reflect CECE shipping. New sections covering the cataloger ensemble, the synthesis layer, the honest accuracy framing.

5. **Orientation v1.84 close-out** — full body update of §3 / §4.x / §13 / §17 (deferred since v1.74). Each subsystem section updated with current state. The orientation v-bump cadence has been preamble-only since v1.75; this is the body update we've owed since the cascade started.

### ✅ Boss-test gate 3 — FINAL INTEGRATION

**Stages**:

**Stage 1 — fresh universe end-to-end**:
1. Create a fresh test universe with 100 notes spanning all five vertical branches and all eleven horizontal sources (Eisa-curated mini-vault).
2. Settings → CECE → Run classification scan.
3. Verify: scan completes with progress strip; ~95% of notes get Unanimous or Strong-Majority assignments; ~5% surface as Sibling Disambiguation; reasoning trails available on all.
4. Resolve all Sibling Disambiguation prompts.
5. Verify: per-Library reliability JSON updates; cataloger weights start to differentiate.

**Stage 2 — Eisa's main universe**:
1. Run scan on the Eisa main vault (7,000+ notes).
2. Verify: scan completes within performance budget (~30 min Apple Silicon / ~4 hr x86).
3. Verify: no typing lag during scan (Performance Rule 1).
4. Verify: queue surfaces sensible Sibling Disambiguation candidates (not 6,000 of them).

**Stage 3 — cloud frontier opt-in**:
1. Opt in to cloud LLM per Library.
2. Re-classify the Sibling Disambiguation queue with cloud.
3. Verify: cloud assignments match expectations; reasoning trails clearer; user-perceived accuracy improves on the hard residual.

**Stage 4 — Arabic vault**:
1. On a Library that's mostly Arabic content, run the scan.
2. Verify: Linguistic Cataloger fires high (CAE doing its work); Reasoning Cataloger handles uṣūlī terminology better than v2 baseline.
3. Verify: reasoning trails render correctly in RTL.

**Stage 5 — long-term learning**:
1. Over the course of one week, accept ~50 corrections.
2. Verify: per-Library reliability JSON shows differentiation between catalogers.
3. Verify: subsequent scans show fewer Sibling Disambiguation prompts in the user's active subtrees.

**Pass criteria**: all five stages pass + audit findings are P0/P1-clean + the User Manual chapter reads as truthful to actual system behavior.

---

## §13  Risks + mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Qwen3-4B GGUF download (~2.5 GB) is too large for some users | Medium | Medium | Allow opt-out: Reasoning Cataloger can run cloud-only if user prefers. Local download is recommended, not required. |
| llama-cpp-2 Windows build issues (CMake / MSVC toolchain) | Medium | High | Test build locally on Windows before V3-§7 lands. If build fails, fall back to llama.cpp via subprocess (we ship the binary). |
| Per-Library reliability JSON corruption | Low | Medium | Atomic write via temp-file rename. On parse failure, fall back to empty profile + log warning. |
| Sibling Disambiguation queue overwhelms user on first vault scan | Medium | Medium | Tune Split-regime threshold conservatively (require ≥3 catalogers split, not 2-1). Cap visible Disambiguation prompts per session at 50. |
| Cataloger ordering bug — Reasoning runs even when cheaper catalogers reach Unanimous | Low | Low (perf only) | Orchestrator unit tests explicitly verify Reasoning is skipped on Unanimous from cheaper catalogers. |
| User-Authority precedence accidentally overridden by ensemble synthesis | Low | High (data integrity) | Synthesis layer has a hard early-return for User-Authority `voiced_opinion: true`. Unit test explicitly verifies. Audit agent §12 Invariant 1 verifies. |
| Living Links P2–P5 schema changes break Graph Cataloger | Low | Medium | Graph Cataloger uses read-only IPCs from Living Links module; adapt if/when those IPCs change. Architect §10 invariant 7 codifies this. |
| v2-era `sources_suggestions` rows don't render in v3 UI | Low | Medium | Backward-compatible JSON parsing in SourceReviewPanel — handles both v2 (raw suggestions only) and v3 (composite trail) shapes. Tested in Audit migration-path agent. |
| Reasoning Cataloger latency exceeds 2 s on cold-start | Medium | Low | First call warms the GGUF; subsequent calls are fast. Status strip shows "Loading model…" on first call. |
| Per-cataloger weight tuning is wrong and synthesis surfaces nonsense | Medium | Medium | Day 1 weights are documented + reviewable (in `data/cataloger_weights_default.json`). Eisa can tune via Settings → CECE → Advanced (deferred to MIG-022 if not needed Day 1). |

---

## §14  Sequencing diagram

```
                                                   ┌──────────────────────────────────┐
                                                   │ V3-§11 Audit + Concept Paper +  │
                                                   │ orientation v1.84 close-out     │
                                                   │ ✅ Gate 3: Final Integration    │
                                                   └────────────────▲─────────────────┘
                                                                    │
                                                   ┌────────────────┴─────────────────┐
                                                   │ V3-§10 Settings + i18n + Docs    │
                                                   └────────────────▲─────────────────┘
                                                                    │
                                                   ┌────────────────┴─────────────────┐
                                                   │ V3-§9 Vertical-axis activation   │
                                                   │ ✅ Gate 2: Vertical axis         │
                                                   └────────────────▲─────────────────┘
                                                                    │
                                                   ┌────────────────┴─────────────────┐
                                                   │ V3-§8 SourceReview UI rewire     │
                                                   │ ✅ Gate 1: Horizontal axis       │
                                                   └────────────────▲─────────────────┘
                                                                    │
                ┌──────────────┬─────────────┬───────┴──┬──────────┬─────────────┐
                │              │             │          │          │             │
        ┌───────┴────┐  ┌──────┴──────┐ ┌────┴───┐ ┌────┴───┐ ┌────┴────┐ ┌──────┴───────┐
        │ V3-§2      │  │ V3-§3       │ │ V3-§4  │ │ V3-§5  │ │ V3-§6   │ │ V3-§7        │
        │ User-      │  │ Structural  │ │ Lingu- │ │ Seman- │ │ Graph   │ │ Reasoning    │
        │ Authority  │  │ Cataloger   │ │ istic  │ │ tic    │ │ Cata-   │ │ Cataloger    │
        │ Cataloger  │  │             │ │ Cata-  │ │ Cata-  │ │ loger   │ │ (local 4B +  │
        │            │  │             │ │ loger  │ │ loger  │ │         │ │  cloud opt-in│
        └───────┬────┘  └──────┬──────┘ └────┬───┘ └────┬───┘ └────┬────┘ └──────┬───────┘
                │              │             │          │          │             │
                └──────────────┴─────────────┴──────────┴──────────┴─────────────┘
                                                    │
                                  ┌─────────────────┴─────────────────┐
                                  │ V3-§1 Foundation                  │
                                  │  - cataloger trait                │
                                  │  - synthesis layer (weighted vote)│
                                  │  - orchestrator (cost-ordered)    │
                                  │  - rules JSON + reliability JSON  │
                                  └───────────────────────────────────┘
```

3 user-testable Boss-test gates: V3-§8 (horizontal), V3-§9 (vertical), V3-§11 (final integration). Other phases self-verify (cargo check, cargo test) and cascade autonomously per Plan-Approval-Equals-Build-Approval.

---

## §15  References

- [`MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md`](MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md) — the architectural spec this Plan implements.
- [`MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md`](MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md) — superseded; substrate inventory.
- [`MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-PLAN.md`](MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-PLAN.md) — superseded; v2 phase log.
- Eisa, *Automatic Classification of Personal Knowledge Notes by Epistemic Source and Content Type* (`epistemic-classifier-paper-EN.md` v1.0) — the foundational paper this Plan operationalizes.
- Four research-agent reports (filed in this session): hierarchical text classification SOTA; library-science prior art (LCSH/MeSH/Dewey); local-LLM benchmarks (Qwen3/Phi/Llama); active-learning + provenance design patterns.
- `docs/sources-of-knowledge-diagram.html` — Eisa-canonical horizontal taxonomy.
- `docs/epistemic-content-taxonomy-chart.html` — Eisa-canonical vertical taxonomy.
- `docs/help.uConstellation.World/Arabic Engine/Arabic Engine.md` — CAE reference.
- Snorkel literature (Stanford → production at Apple/Google/Intel) — synthesis pattern reference for MIG-022.

End of Plan.

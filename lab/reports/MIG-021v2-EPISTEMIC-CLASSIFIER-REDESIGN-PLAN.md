# MIG-021v2 — Epistemic Classifier Redesign — Plan

**Date**: 2026-05-09
**Architect**: [`MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md`](MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md) — approved by Eisa 2026-05-09 (Sight mode P = Option α; Tier 3 don't hide; all 8 §12 open questions = yes).
**Anchored against**: [`docs/sources-of-knowledge-diagram.html`](../../docs/sources-of-knowledge-diagram.html) (horizontal axis, Eisa-canonical) + [`docs/epistemic-content-taxonomy-chart.html`](../../docs/epistemic-content-taxonomy-chart.html) (vertical axis, Eisa-canonical).
**Hard constraint**: zero impact on boot-perf budget (≤ 6 s hydrated; ≤ 1 s on Eisa's machine). Verified at the end of every commit.

---

## §0 · Locked design decisions (per Architect §10 + §12 + Eisa's ratification)

| Decision | Locked value | Reversible? |
|---|---|---|
| Field name for vertical axis | `content_type` | Yes (single search-replace) |
| Slug scheme | kebab-case + parent-prefix on collision | Yes (rename via migration) |
| Multi-select within an axis | YES (parent + child both checked allowed) | Yes (UI behavior) |
| Tree picker layout | Side-by-side ≥ 1200px; tabbed below | Yes (CSS) |
| Default classifier output level | Suggest at deepest level where confidence ≥ 0.55; fall back to parent | Yes (single threshold constant) |
| Tier 3 visibility | ALL tiers always shown — NO Settings opt-out built | Reversible by adding the toggle later if requested |
| Tier-aware classifier confidence fallback | ENABLED — when top-1 is Tier 3 and confidence < 0.55, suggest Tier 1/2 alternative | Yes (single flag) |
| Sight v5 mode P design | Option α — 11 horizontal parents as wedges with drill-down on click + tier coloring | Decision deferred to MIG-022 implementation; locked for now as the design intent |
| Concept Paper amendments | Land in v2.1 on §1K' close-out | Yes (single commit) |

These are committed. Build cascade does not pause to re-ask. Architectural surprise rule still applies for genuinely unmapped invariants.

---

## §1 · Phased build (11 commits, 3 user-test gates)

Each phase = one landable commit + verification. **Eisa Boss-tests at the gates marked ✅.** Other phases self-verify (cargo check, type-check, /simplify pass).

### Phase §1A' — Extract diagrams + schema migration + frontmatter `content_type:` + 3 IPCs

**Goal**: ship the data substrate. Both taxonomies land as structured data; the new `content_type` column joins `sources`; mirror IPCs for the new field.

**Files touched**:
- **NEW** `src-tauri/src/sources/horizontal_taxonomy.rs` — extracted from `docs/sources-of-knowledge-diagram.html`. Const data: 11 parents with `(id, en, ar, tr, tier)` + 41 children with `(parent_id, en, ar, tr)`. Plus stable slug map.
- **NEW** `src-tauri/src/sources/vertical_taxonomy.rs` — extracted from `docs/epistemic-content-taxonomy-chart.html`. ~218 nodes as nested struct with `(id, en, ar, parent_id)`. Tree-walking helpers.
- **NEW** `src/lib/sources/horizontalTaxonomy.ts` — TypeScript mirror, same data, same slugs. Single source of truth in code = the Rust files; TS files are JSON-equivalent re-exports for the picker.
- **NEW** `src/lib/sources/verticalTaxonomy.ts` — same.
- **EDIT** `src-tauri/src/sources.rs`:
  - Replace flat `SOURCE_IDS: &[&str; 12]` with helpers reading from `horizontal_taxonomy::all_ids()` (53 IDs: 1 root + 11 parents + 41 leaves) + the `unclassifiable` opt-out.
  - Add `extract_content_type(content) -> Vec<String>` (mirror of `extract_sources`).
  - Add `read_content_type_for_note`, `write_content_type_to_db`, `rewrite_frontmatter_content_type`.
  - Add `ensure_note_meta_content_type_column(conn)` — idempotent ALTER.
- **EDIT** `src-tauri/src/search.rs::init_db` — wire the new column ensure.
- **EDIT** `src-tauri/src/search.rs::index_note` — extract `content_type:` on every save, stamp into the new column (extend INSERT 13 → 14 columns).
- **EDIT** `src-tauri/src/lib.rs` — register 3 new IPCs:
  - `content_type_get_for_note(path) -> Vec<String>`
  - `content_type_set_manual(path, content_type) -> ()`
  - `content_type_clear(path) -> ()`

**Self-verification**:
- `cargo check` passes
- Schema migration runs idempotently (re-run is no-op)
- Manual: write a sample frontmatter with both `sources:` and `content_type:` to one trial-universe note; restart Constellation; confirm both `note_meta.sources` AND `note_meta.content_type` mirror updates
- Unit tests: extract_content_type handles all three YAML shapes (scalar / inline / block list); silently drops unknown values

---

### Phase §1B' — Expand classifier to 271 candidates + tier-aware fallback + axis tag

**Goal**: classifier suggests for both axes; tier-aware fallback when borderline.

**Files touched**:
- **EDIT** `src-tauri/src/classifier/source_definitions.rs` — replace 11-entry constant with:
  - `HORIZONTAL_DEFINITIONS: &[(parent_id, leaf_id_or_self, en_definition); 53]` — 11 parent self-definitions + 41 leaf definitions. Each ~120-150 words, drawn from the diagram's data plus a leaf-specific paragraph (parent definition reused as base + leaf-specific contrast).
  - `VERTICAL_DEFINITIONS: &[(node_id, en_definition); ~218]` — extracted from the vertical chart. Where the chart only has labels (no descriptions), generate concise definitions from the parent context.
- **EDIT** `src-tauri/src/classifier/tier1_embedding.rs`:
  - Two cached vector pools: `HORIZONTAL_VECTORS` (53) + `VERTICAL_VECTORS` (218); each a `OnceLock<Vec<(String, Vec<f32>)>>`.
  - At first classifier call: embed all 271 definitions (one-time ~10 sec on Eisa's machine).
  - `classify(text)` returns two parallel suggestion sets: top-3 horizontal + top-3 vertical, each tagged with `axis: "horizontal" | "vertical"`.
  - **Tier-aware fallback** (§10.4): when top-1 horizontal is a Tier 3 parent (or descendant) AND confidence < 0.55, replace it with the highest-scoring Tier 1/2 candidate. Tier metadata read from `horizontal_taxonomy::tier_for(parent_id)`.
  - **Leaf-vs-parent strategy** (Q5): for each axis, if the deepest-level top-1 has confidence ≥ 0.55, use the leaf; else fall back to its parent.
- **EDIT** `src-tauri/src/sources.rs` — `Suggestion` struct adds `axis: String` field (defaults `"horizontal"` for backward compat with §1A/§1B suggestions still in DB).
- **EDIT** `src-tauri/src/classifier/mod.rs::classifier_suggest_for_note`:
  - Returns `SuggestionRecord` with combined suggestions list (both axes interleaved by axis).
  - `write_suggestions` continues to persist as one JSON list; the `axis` tag distinguishes.

**Self-verification**:
- `cargo check` passes
- Manual: invoke `classifier_suggest_for_note` on 10 hand-picked trial-universe notes; inspect output; verify both axes return suggestions; verify Tier 3 fallback fires on at least one note where the classifier weakly picks revelation/inspiration on a non-religious note
- Acceptable: classifier produces structurally correct output (tagged by axis, ordered by confidence). Accuracy is not the gate at this phase — Eisa tests in §1C' Boss-test
- Unit tests: l2_normalize, dot, clamp01, axis-tag presence, tier-fallback logic

---

### Phase §1C' — Tree-picker component + replace flat grid + show both axes ✅ Eisa Boss-test gate

**Goal**: the new tree picker ships; Source Review panel shows both axes; Edit mode uses tree pickers × 2.

**Files touched**:
- **NEW** `src/lib/sources/TaxonomyTreePicker.svelte` (~400 LOC):
  - Props: `taxonomy: TaxonomyNode[]`, `axis: "horizontal" | "vertical"`, `selected: Set<string>`, `onChange: (Set<string>) => void`, `tierColors: boolean` (true for horizontal axis)
  - Renders nested tree with expand/collapse chevrons, multi-select checkboxes per node, tri-script labels (EN + AR + transliteration where present)
  - Tier-based color coding when `tierColors=true`: teal/purple/amber left border on horizontal parents (matches `sources-of-knowledge-diagram.html`)
  - Search/filter input at top; auto-expands ancestors of matches
  - Keyboard nav (arrows / Space / Enter)
  - RTL-aware (`dir="auto"`; `border-inline-start` for tier color)
- **EDIT** `src/lib/components/SourceReviewPanel.svelte`:
  - Card body: render TWO suggestion sublists per record, grouped by `s.axis`. Each labeled "Sources" / "Content type" with appropriate tier badge for horizontal items.
  - Edit mode: replace the flat 12-pill grid with TWO `TaxonomyTreePicker` instances (horizontal on top, vertical below; OR side-by-side on wide viewports per Q4).
  - Accept commits BOTH axes (calls `sources_set_manual` + new `content_type_set_manual`).
- **EDIT** `src/lib/i18n/en.json` + `ar.json`:
  - Add `sources.review.axis.horizontal` / `axis.vertical` labels (e.g., "Sources" / "Content type")
  - Add `taxonomyTreePicker.*` chrome strings (~10 strings: search placeholder, expand all, collapse all, tier-1/2/3 legend tooltips)
- New IPC needed in this phase (mirrors sources):
  - `content_type_get_suggestions(path)` (already returned by classifier in suggestions; this is for the panel to re-fetch)
  - `content_type_list_pending_suggestions()` (LIKELY merge into existing `sources_list_pending_suggestions` since both axes share the queue)

**Eisa Boss-test gate**:
1. Build & launch.
2. Open Source Review panel; click "Classify open note" on 3-5 notes.
3. Each card now shows TWO suggestion sublists (Sources + Content type). Tier badges visible on horizontal items (teal Perception/Inference; purple Testimony/Comparison/Memory/Innate-disposition; amber Mass-transmission/Postulation/Non-apprehension/Inspiration/Revelation).
4. Click Edit on one card; confirm two tree pickers render side-by-side (or stacked if narrow). Pre-checked nodes match the suggestions. Hover any node — definition tooltip in current locale.
5. Modify selection across both axes (uncheck a horizontal leaf, check a vertical leaf at depth 3). Click Save. Confirm BOTH `sources:` and `content_type:` frontmatter fields update on disk.
6. Click Accept on another card — both axes' top suggestions written to frontmatter.
7. Click Reject on another card — both axes' suggestions cleared, no frontmatter writes.
8. Switch UI to Arabic — confirm tree picker renders RTL with tier-color border on the leading edge (right side); tri-script labels render correctly.

If any step fails, surface immediately and pause cascade.

---

### Phase §1D' — PropertyEditor integration

**Goal**: per-note manual setting via two tree pickers in the existing PropertyEditor.

**Files touched**:
- **EDIT** `src/lib/components/PropertyEditor.svelte` — add two new rows between Maturity and Stage:
  - Horizontal sources (TaxonomyTreePicker, axis="horizontal", tierColors=true)
  - Vertical content_type (TaxonomyTreePicker, axis="vertical", tierColors=false)
- **EDIT** `src/lib/i18n/en.json` + `ar.json` — add `propertyEditor.sourcesField` / `contentTypeField` labels

**Self-verification**:
- Open PropertyEditor on any note; both fields render
- Multi-select on each; save; confirm frontmatter and SQLite mirror update for both fields

---

### Phase §1E' — Right-click context action

**Goal**: on-demand classification surfaces from any note context.

**Files touched**:
- **EDIT** existing context-menu wiring in FileTree / Sky View / wherever notes are right-clickable — add "Suggest sources & content type for this note" item
- Action invokes `classifier_suggest_for_note(path)` then opens Source Review panel scrolled to the new entry
- **EDIT** `src/lib/i18n/en.json` + `ar.json` — `sources.contextMenu.suggest` key (replaces the §1E wording)

**Self-verification**:
- Right-click any note in any context → "Suggest sources & content type" appears → click → Source Review panel opens within ~3 seconds with new entry showing both axes

---

### Phase §1F' — Background scan extends to both axes ✅ Eisa Boss-test gate

**Goal**: the universe-scale resumable scan classifies BOTH axes.

**Files touched**:
- **EDIT** `src-tauri/src/classifier/scan_job.rs` — per-note: invoke classifier (already returns both axes' suggestions); write to queue (already supports both via §1B' axis tag)
- **EDIT** status-bar `MigrationProgressStrip` — same chrome, no change needed (already chunked + cancelable from §1F of original Plan; `§1F` was deferred and never built — will be NEW in this phase)
- **NEW** Settings → AI section gains a toggle: "Auto-classify sources and content type for new and changed notes" (single toggle covers both axes)

**Eisa Boss-test gate**:
1. Settings → AI → toggle on auto-classify
2. Confirm status-bar shows "Classifying… 0 / 7,636" and count starts climbing
3. Type 10 chars in any note while scan runs — confirm zero typing lag (Performance Rule 1)
4. Close + reopen Constellation mid-scan → resumes from where it left off
5. Cancel from status-bar group → scan stops cleanly
6. Run to completion → Source Review panel has hundreds of records each with both axis suggestions

---

### Phase §1G' — i18n full pass (~270 NEW + ~436 lifted strings; EN + AR canonical)

**Goal**: every new UI string + the 41 horizontal sub-leaves + ~218 vertical nodes translated EN + AR.

**Files touched**:
- **EDIT** `src/lib/i18n/en.json`:
  - `sources.label.{41_subleaves}` — short labels lifted from the diagram
  - `sources.description.{41_subleaves}` — one-sentence definitions
  - `sources.transliteration.{nodes_with_transliteration}` — Sanskrit/Pali transliterations as separate i18n entries (so they render even in non-EN locales)
  - `sources.evidence.{any_new_horizontal_leaves}` — evidence one-liners for classifier output
  - `contentType.label.{~218_nodes}` — lifted from `epistemic-content-taxonomy-chart.html` (Eisa pre-authored EN+AR pairs)
  - `contentType.description.{~218_nodes}` — short descriptions where present in chart; auto-derive from parent context where absent
  - `taxonomyTreePicker.{search|expandAll|collapseAll|tier1Legend|tier2Legend|tier3Legend|noResults|...}` — chrome (~10)
- **EDIT** `src/lib/i18n/ar.json` — full Arabic mirror (Arabic data lifted from both diagrams; Eisa is native verifier on these)

**Self-verification**:
- Switch UI to Arabic; click through every new surface (tree picker, Source Review, PropertyEditor, Settings → AI); confirm RTL renders correctly; confirm no key-strings leak (literal `sources.foo.bar` showing instead of translated text)
- Switch to a non-EN-non-AR locale (Spanish); confirm graceful English fallback

---

### Phase §1G2' — Tier 1 deterministic rules engine + bilingual lexicon + correction log ✅ Eisa Boss-test gate

**Amendment date**: 2026-05-09. Inserted into the cascade after Eisa accepted the consultant analysis surfaced post-§1D' showing that (a) the current Tier-2-only architecture caps Source accuracy at ~75-85% at leaf level, (b) the absent Tier 1 layer means we run the embedding model on notes a regex would handle deterministically in microseconds, and (c) without an active-learning correction log every user override is information thrown away.

**Goal**: ship a free, deterministic Tier 1 that resolves ~30-40% of notes before Tier 2 is consulted, plus the correction log that turns every user override into future training data.

**Files touched**:
- **NEW** `src-tauri/src/classifier/tier1_rules.rs`:
  - Frontmatter precedence: if `sources:` or `content_type:` is already in the YAML (set manually via §1D'), Tier 1 returns those values with confidence 1.0 and the lower tiers are skipped.
  - Regex patterns for citation forms (Sahih al-Bukhari N, ISBN, DOI, URL → testimony; SI units + numerals → fact; equations → fact; quoted blocks → testimony).
  - Verbal-marker matching: "I doubt / I believe / I'm certain / I lean toward" + Arabic equivalents → epistemic state (vertical axis).
  - Bilingual lexicon lookup against the new `src-tauri/data/sources_lexicon.json` (next bullet).
- **NEW** `src-tauri/data/sources_lexicon.json` — paired EN/AR/Sanskrit terms drawn from the Eisa-canonical horizontal taxonomy. Each entry: `{ tokens: [...], axis: "horizontal", target: "<taxonomy_id>", weight: 0.85, scope: "uṣūl|nyāya|generic" }`. Examples: `متواتر / mutawātir → mass-transmission/lafẓī`; `إجماع / ijmāʿ → testimony/consensus`; `قياس → comparison/analogy`; `pratyakṣa → perception/external`.
- **NEW** `src-tauri/src/classifier/correction_log.rs` — append-only NDJSON at `<library>/.constellation/classifier_corrections.jsonl`. Each user override (Source Review reject/accept-with-edit, PropertyEditor manual change away from classifier suggestion) writes a tuple: `{ ts, note_path, axis, predicted: [...], corrected: [...], tier_used }`. No deletion API — the file is the ground-truth audit trail.
- **EDIT** `src-tauri/src/classifier/mod.rs` — `classify_note(path)` now: Tier 1 → if confidence ≥ 0.85 return; else Tier 2 → if confidence ≥ 0.55 return; else (in §1H') Tier 3.
- **EDIT** `src-tauri/src/sources/mod.rs` — `sources_set_manual` and `content_type_set_manual` log to the correction log when the new value differs from the most recent classifier suggestion for that note.
- **EDIT** `src/lib/components/SourceReviewPanel.svelte` — Tier badge on each suggestion ("T1 rule" / "T2 embedding" / "T3 LLM" — pending §1H') so the user sees which tier produced the suggestion.

**Eisa Boss-test gate**:
1. Create a note with frontmatter `sources: [testimony]` set manually. Right-click → Suggest. Result: instant; suggestion echoes the manual value with badge "T1 rule" + confidence 1.0. Tier 2 is NOT consulted (verify with logging).
2. Create a note with the body "حدثنا الإمام مسلم في صحيحه..." Right-click → Suggest. Tier 1 lexicon matches `حدثنا` + `صحيحه` → `testimony` with badge "T1 rule".
3. Create a note with body "I doubt that the moon landing happened in 1969." Right-click → Suggest. Tier 1 verbal-marker hit → vertical axis = `epistemic-states/doubt`.
4. Create a note with no frontmatter, no citations, no verbal markers ("Constellation is a personal knowledge formulation system."). Right-click → Suggest. Tier 1 returns nothing high-confidence; Tier 2 takes over and badge shows "T2 embedding".
5. In Source Review, override a Tier-2 suggestion with a manual selection. Open `<library>/.constellation/classifier_corrections.jsonl` — confirm one new line was appended with the predicted vs corrected tuple.

---

### Phase §1G3' — Provenance metadata schema + capture-time UX ✅ Eisa Boss-test gate

**Amendment date**: 2026-05-09. Companion to §1G2'. The consultant analysis ranked this as #2 highest-leverage of six recommendations: "Five seconds of user input at capture is worth more accuracy than any architectural improvement."

**Goal**: standardize provenance fields in the YAML frontmatter and surface them at note-creation time as a single optional drop-down — so authoritative ground truth flows in by capture, not by retroactive classification.

**Files touched**:
- **EDIT** `src-tauri/src/sources/mod.rs` — extend the YAML schema with three new optional fields:
  - `source_citation` (string) — free-form citation if known (e.g. `Sahih al-Bukhari 2371`, `ISBN 978-0-...`, `https://...`)
  - `acquisition_method` (enum) — `firsthand | reading | hearing | derivation | unknown` — orthogonal to taxonomic source ID; cheaper for the user to pick than full source classification
  - `confidence` (number 0–1) — user's own self-assessed confidence in the proposition (NOT the classifier's; distinct field)
- **EDIT** `src/lib/components/PropertyEditor.svelte` — add three new `KEY_SUGGESTIONS` entries (`source_citation`, `acquisition_method`, `confidence`); the `acquisition_method` key gets a special render branch with a 5-option dropdown.
- **NEW** `src/lib/sources/QuickCaptureSourceWidget.svelte` — small inline widget that appears in NewNoteDialog/CreateItemDialog under the title field: a single drop-down "How did you come across this?" with the 5 acquisition_method options + an "I'll fill this in later" escape. Selection writes the field into the new note's frontmatter on creation.
- **EDIT** `src-tauri/src/classifier/tier1_rules.rs` — `acquisition_method` is now a Tier 1 strong signal: `firsthand → perception`, `reading → testimony`, `hearing → testimony/oral`, `derivation → inference`, `unknown` → no prior.
- **EDIT** `src/lib/i18n/en.json` + `ar.json` — `quickCapture.acquisitionMethod.{firsthand|reading|hearing|derivation|unknown|skip}` (~6 strings) + `propertyEditor.acquisitionMethod.label`.

**Eisa Boss-test gate**:
1. Cmd/Ctrl+N to create a new note. The new dropdown appears under the title field with the prompt "How did you come across this?" and 5 + skip options.
2. Pick "Reading"; type a title; create. The new note's frontmatter contains `acquisition_method: reading`.
3. Right-click → Suggest sources. Tier 1 fires: `acquisition_method: reading` is a strong prior for `testimony` → suggestion comes back instantly with high confidence and badge "T1 rule".
4. From PropertyEditor, edit `acquisition_method` to `firsthand`. Re-suggest. Tier 1 now flips the prior to `perception`.
5. Verify the dropdown is RTL when UI is Arabic.

---

### Phase §1H' — Tier-3 LLM (Qwen3-1.7B + llama.cpp) integration ✅ Eisa Boss-test gate

**Renamed 2026-05-09**: was "Tier-2 download + llama.cpp" — renamed to **Tier-3** to reflect the corrected three-tier architecture (Tier 1 deterministic rules → Tier 2 embeddings → Tier 3 LLM). The LLM is now consulted only on the residual that Tier 1 + Tier 2 cannot confidently resolve. Substantively unchanged otherwise.



**Goal**: optional Tier-3 LLM classifier ships; both axes get LLM-quality classification on the hard residual that Tier 1 + Tier 2 left low-confidence.

**Files touched**:
- `src-tauri/Cargo.toml` — add `llama-cpp-2` dependency
- **NEW** `src-tauri/src/classifier/tier2_llm.rs` — llama.cpp wrapper. Lazy-load Qwen3-1.7B GGUF on first use. Few-shot prompt with both taxonomy hierarchies. **GBNF grammar constrains output to one-of-271 across both axes** — guaranteed valid JSON matching schema `{ horizontal: [...], vertical: [...] }`.
- **NEW** `src-tauri/src/classifier/prompt.rs` — few-shot prompt template + GBNF grammar
- **NEW** `src-tauri/src/classifier/tier2_download.rs` — resumable HTTP download from `https://github.com/eisaShamsi/Constellation/releases/download/sight-v5-classifier/qwen3-1.7b-q4_k_m.gguf` (Q1 default per original Plan §0)
- 4 new IPCs: `tier2_download_model`, `tier2_status`, `tier2_unload`, `classifier_reclassify_all_with_tier2`
- **NEW** `src/lib/sources/SettingsAIPanel.svelte` — Tier-2 download/manage UI
- **EDIT** `src/lib/i18n/en.json` + `ar.json` — `sources.settings.tier2.*` (~15 strings)

**Eisa Boss-test gate**:
1. Settings → AI → "Download larger classifier" → progress bar to completion (~2-10 min)
2. Status flips to "Downloaded — ready"
3. Right-click an Arabic-heavy note → "Suggest sources & content type" → confirm Tier-2 suggestions appear (visibly different + more confident than Tier-1)
4. Compare suggestions on 5 hand-picked notes between Tier-1 and Tier-2; Tier-2 should produce richer per-classification evidence (quoted text from the note rather than generic per-source signature)
5. Settings → AI → "Re-classify all with larger model" → queue refills with Tier-2 suggestions
6. Settings → AI → "Unload model from memory" → confirm RAM frees

---

### Phase §1I' — Help docs + User Manual EN + AR

**Goal**: user can read about the two-axis system end-to-end before opting in.

**Files touched**:
- **EDIT** `docs/User Manual.md` — new section under §3 Creating and Editing Notes: "Source Tags & Content Type"
  - Explains both axes with the choice-of-depth principle
  - Tier system: brief explanation; not gatekept; user picks at any tier
  - How to set manually (PropertyEditor) vs via classifier
  - Optional larger classifier path (Settings → AI download)
- **EDIT** `docs/help.ar/User Manual.md` — Arabic translation
- **NEW** `docs/help.uConstellation.World/Sources/Sources.md` — full help topic with both diagrams as links + worked examples + FAQ
- **NEW** `docs/help.ar/Sources/Sources.md` — Arabic translation
- 13 other locales → queued as PJ (existing convention)

**Self-verification**:
- Read EN + AR sections from a first-time-user perspective
- Confirm the diagrams link works from User Manual
- Confirm tier system is explained without civilizational jargon dominating

---

### Phase §1J' — `/simplify` checkpoint + 3-agent audit

**Goal**: standard /migration audit before close.

**Files touched**:
- `/simplify` over the full diff §1A' – §1I'. Tier-1 findings fixed before audit.
- 3 audit agents in parallel:
  - **Invariant**: do the 12 P1-P12 from original Architect (still applicable) + the new tier-aware behaviors hold?
  - **Drift**: any new guards introduced that the system doesn't know about?
  - **Migration-path**: first-boot, mid-backfill restart, downgrade, Tier-2 corruption, both-axes empty/partial state
- Findings written to `lab/reports/MIG-021v2-EPISTEMIC-CLASSIFIER-AUDIT.md`
- P0/P1 findings fixed in close-out commit

**Self-verification**:
- /simplify produces zero unaddressed Tier-1 findings
- All three audit agents PASS or PASS-WITH-FIXED-P0/P1

---

### Phase §1K' — Close-out + Concept Paper v2.1 + orientation v1.80+

**Goal**: MIG-021v2 marked Done; Concept Paper amended; orientation bumped.

**Files touched**:
- **NEW** `docs/Constellation-Sight-Concept-Paper-v2.1.md` — amends v2.0 per Architect §11:
  - §7.1: 11 parents + sub-leaves + tier metadata
  - §7.1b: vertical axis section
  - §7.2: frontmatter contract adds `content_type:`
  - §7.3: tree picker replaces dropdown
  - §7.4: Sight mode P clarified as Option α
  - §7.5 NEW: tier system + tier-aware confidence fallback
  - §5: modes table updated
- **EDIT** `docs/Constellation Orientation & Onboarding v1.80.md` (or whichever version is next at close-out) — preamble bump per SO #6 (subsystem ships major feature trigger). Body §3 / §4.x finally updated to reflect the new Sources + Content Type subsystem.
- **EDIT** `lab/reports/SESSION-LOG-2026-XX-XX.md` — append phase log
- **EDIT** `docs/Constellation Pending Jobs vN.md` — mark MIG-021v2 / PJ-NNN Done. Allocate PJ for 13-locale Sources translation + 13-locale ContentType translation.

**Self-verification**:
- `git push origin main` succeeds
- Boss confirmation across multiple sessions: Sight v5 Sources foundation + Content Type ship is stable

---

## §2 · Sequencing diagram

```
§1A' Schema + extract  ─────►
§1B' Classifier ext.  ───────►
                            ✅ §1C' TreePicker + dual axes  ──────►
                                                              §1D' PropertyEditor  ──►
                                                                                §1E' Right-click  ──►
                                                                                              ✅ §1F' Background scan  ──►
                                                                                                                   §1G' i18n full pass  ──►
                                                                                                                                   ✅ §1H' Tier-2 + llama.cpp  ──►
                                                                                                                                                          §1I' Help docs  ──►
                                                                                                                                                                       §1J' /simplify + audit  ──►
                                                                                                                                                                                       §1K' Close-out + PCS
```

3 user-testable Boss-test gates: §1C', §1F', §1H'. Other phases self-verify and cascade autonomously per Plan-Approval-Equals-Build-Approval.

---

## §3 · Risk register (delta from Architect §13)

| Risk | Severity | Mitigation |
|---|---|---|
| Vertical taxonomy extraction (~218 nodes) introduces parsing bugs | Medium | Single source of truth in Rust file; TS file is mechanical re-export; spot-check 10 random nodes against the chart |
| Definitions for vertical leaves are not pre-authored in the chart (only labels exist) — generating ~218 short descriptions risks fabrication | Medium | Generate from PARENT CONTEXT mechanically (e.g., "Bioelectric: a biological signal of the bioelectric type"); flag any that need richer text for a future PJ; do NOT invent rich definitions where the chart only has a label |
| Tier-aware fallback logic introduces subtle suggestion-ordering bugs | Medium | Unit tests: explicit cases for "top-1 Tier 3 below 0.55 → fallback fires", "top-1 Tier 3 above 0.55 → fallback does NOT fire", "no Tier 1/2 candidates available → no fallback possible" |
| Tree picker performance with 271 horizontal + 218 vertical nodes might lag on low-end machines | Low | Render only visible (collapsed-by-default; lazy-render expanded subtrees); virtualize if >100 visible items |
| Two parallel fields per note doubles the cognitive cost on every note save | Medium | Both fields are OPTIONAL — user can leave either or both empty; classifier suggests both but user decides what to accept |
| llama-cpp-2 Windows build issues (CMake / MSVC toolchain) — same as original | Medium | Test build locally on Windows before §1H' lands |
| Concept Paper v2.0 mid-revision; future sessions might miss v2.1 amendments | Low | This Plan + the Architect document the deltas; v2.1 lands at §1K' close-out as one focused doc |

---

## §4 · Out of scope (this MIG)

- **Sight v5 visual** (the dome rendering, the modes, the tree-picker integration into Sight itself). That's MIG-022.
- **13-locale translation** of the new strings — queued as PJ (per existing convention).
- **Concept Paper v2.1 amendments** — drafted at §1K' close-out, not earlier.
- **Sight v5 mode P implementation** — design Option α locked here; build is in MIG-022.
- **Cross-platform validation** beyond Windows — release-checklist concern; not in MIG-021v2.

---

## §5 · Cross-references

- [`docs/sources-of-knowledge-diagram.html`](../../docs/sources-of-knowledge-diagram.html) — horizontal taxonomy, Eisa-canonical
- [`docs/epistemic-content-taxonomy-chart.html`](../../docs/epistemic-content-taxonomy-chart.html) — vertical taxonomy, Eisa-canonical
- [`MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md`](MIG-021v2-EPISTEMIC-CLASSIFIER-REDESIGN-ARCHITECT.md) — the Architect this Plan implements
- [`MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md`](MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md) — original Plan, superseded
- [`MIG-021-LOCAL-LLM-RESEARCH.md`](MIG-021-LOCAL-LLM-RESEARCH.md) — Tier-2 model + engine research; informs §1H'
- CLAUDE.md — Performance Rules 3 + 8; Working Agreements 4 + 5; Plan-Approval-Equals-Build-Approval

---

**End of MIG-021v2 Plan.**

On Boss approval: Build cascade begins with §1A' (schema + frontmatter `content_type:` + 3 IPCs — non-user-visible foundation, self-verifying). Cascades autonomously through §1A' → §1B' → ✅§1C' (first Boss-test gate).

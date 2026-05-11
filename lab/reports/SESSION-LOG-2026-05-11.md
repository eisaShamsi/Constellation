# Session log — 2026-05-11

## V3-§9 vertical-axis activation cascade — A → E

Continuation from 2026-05-10's V3-§8 close-out + Boss-test follow-up cascade (r5 → r8). Boss elected to fix Gate 1 follow-up issues #1 and #2 plus the queue filter (r7 + r8) before V3-§9. Then for V3-§9 itself: Architect doc → Plan → Boss approves Option C (Full) → cascade A→E.

### V3-§9 Architect (Phase 1)

`lab/reports/MIG-021v3-V3-§9-VERTICAL-ACTIVATION-ARCHITECT.md` written first. Audit table showed most of V3-§9's original-Plan scope was already shipped through V3-§8. Surfaced 4 actual gaps: lexicon thinness, structural detector asymmetry, dormant per-axis reliability, Reasoning Cataloger interface lock-in for V3-§7.b. Three scope options (A: lexicon only; B: + structural + Gate 2; C: + per-axis reliability + Reasoning prompt). Recommended B; Boss picked **C**.

### V3-§9 Plan (Phase 2)

`lab/reports/MIG-021v3-V3-§9-VERTICAL-ACTIVATION-PLAN.md` — 5 phases A→E with verification clauses, files-touched lists, risk register inheritance, Gate 2 6-stage Boss-test spelled out per the Testing Instructions Rule. Boss approved verbatim; cascade per Plan-Approval-Equals-Build-Approval.

### V3-§9.A — Vertical lexicon expansion (`4e0981a`)

Added 12 new entries to `sources_lexicon.json::vertical` covering all 5 branches (was 7 entries all in `epistemic-states`).

Two ID typos caught + corrected during Plan §1's pre-commit grep validation:
- `semantic-contents/theory` doesn't exist → `higher-order-constructs/theory`
- `epistemic-states/knowledge/by-acquaintance` → `epistemic-states/knowledge/by-content/acquaintance`

ال-prefix coverage gap caught when Arabic worldview test failed: surface-token match is exact substring (not lemma-aware), so all Arabic nouns frequently taking ال definite article need both forms in the tokens list. Updated all 12 new entries.

4 new spot tests in linguistic.rs:
- `v3_p9a_definition_phrasing_fires_concept`
- `v3_p9a_arabic_worldview_fires_higher_order` (caught the ال-prefix gap)
- `v3_p9a_propositional_knowledge_phrasing_fires_correct_target`
- `v3_p9a_doctrine_phrasing_fires_higher_order`

71 cece tests pass total (was 67 in r8).

### V3-§9.B — Structural vertical detectors (`d9dfa60`)

Added 5 new vertical regex rules + 1 line-pass density rule to `structural.rs::vertical_rules()`:
- English definition marker → `semantic-contents/concept` (0.80)
- Arabic definition marker → `semantic-contents/concept` (0.80)
- English worldview marker → `higher-order-constructs/worldview` (0.75)
- Arabic worldview marker → `higher-order-constructs/worldview` (0.75)
- Figure/diagram reference → `sensory-inputs/signal/physical/electromagnetic` (0.70)
- Code-block-density → `symbolic-entities/sign` (0.65–0.80 density-driven)

Code-block-density is a separate `count_code_block_fences()` line-pass + inline check in `classify()`, not a regex tuple — threshold gating + density-driven weight don't fit the `(regex, target, weight)` shape used by `vertical_rules()`.

7 new V3-§9.B regression tests:
- `v3_p9b_english_definition_fires_concept`
- `v3_p9b_arabic_definition_fires_concept`
- `v3_p9b_worldview_marker_fires_higher_order`
- `v3_p9b_figure_reference_fires_visual_signal`
- `v3_p9b_code_block_density_fires_symbolic_entity`
- `v3_p9b_single_code_block_does_not_fire_density` (over-match guard)
- `v3_p9b_count_code_block_fences_helper` (line-pass unit test)

78 cece tests pass.

### V3-§9.C — Wire reliability updates into corrections (`ec5527e`)

Architect doc framed Phase C as "schema migration v1→v2." Re-auditing `reliability.rs` BEFORE implementing revealed schema was already per-axis. The real gap: `record_correction()` was defined but called from nowhere. Reliability tracking was built end-to-end but **dormant** — accuracy ratios stayed at uniform 1.0 forever.

Re-scoped Phase C to wire dormant machinery:
- New `update_reliability_from_correction(library_root, composite_json, axis, user_pick)` helper iterates per-cataloger trails, marks each voicing cataloger correct if its primary matched user's pick, wrong otherwise. Silent catalogers don't get bumped (silence is neither right nor wrong).
- Wired into `sources_set_manual` + `content_type_set_manual`, snapshotting `composite_json` BEFORE `clear_suggestions` deletes the row.
- Best-effort: malformed JSON / missing fields / unknown axis → no-op (correction_log still fires).

7 new V3-§9.C wiring tests covering: correct cataloger bump, silent cataloger no-bump, axis-specific no cross-pollution, vertical correction bumps vertical only, malformed JSON no-op, unknown axis no-op, accumulating bumps across multiple corrections.

85 cece tests pass.

**Lesson logged in commit message + orientation v1.90**: when an Architect doc's Phase scope rests on memory of how something is implemented, validate against actual source BEFORE drafting the implementation. The "schema migration" Phase C would have been a no-op had I not re-checked `reliability.rs` first. The real Phase C work (wiring) is more valuable AND smaller scope.

### V3-§9.D — Reasoning Cataloger axis-aware GBNF (`b18a3ee`)

Same audit-first pattern: existing SYSTEM_PROMPT already explicitly distinguishes the two axes (HORIZONTAL = SOURCE, VERTICAL = CONTENT TYPE), and combined GBNF already enforces axis separation at the leaf level. Phase D's prompt-rewrite scope was already done.

Real gap: missing axis-specific grammar functions for V3-§7.b two-pass classification. Added:
- `build_gbnf_horizontal_only()` — h-only grammar
- `build_gbnf_vertical_only()` — v-only grammar
- `build_gbnf_combined()` — backward-compat alias for the existing combined grammar
- `GRAMMAR_CACHE_HORIZONTAL` + `GRAMMAR_CACHE_VERTICAL` `OnceLock`s
- Shared `build_gbnf_axis_only(axis)` helper

No runtime change today (Reasoning still abstains because llama.cpp not wired). When V3-§7.b ships, wiring layer chooses single-pass or two-pass per benchmark.

5 new V3-§9.D tests:
- `v3_p9d_horizontal_grammar_only_contains_horizontal_ids`
- `v3_p9d_vertical_grammar_only_contains_vertical_ids`
- `v3_p9d_combined_grammar_unchanged` (identity guard)
- `v3_p9d_system_prompt_explicitly_distinguishes_axes` (anti-drift guard)
- `v3_p9d_axis_aware_exemplars_balance_horizontal_and_vertical`

90 cece tests pass total.

### V3-§9.E — NSIS rebuild + orientation v1.90 + Gate 2 Boss-test ready (this commit)

NSIS `Constellation_0.3.4_x64-setup.exe` rebuilt; mtime captured at commit time.

Orientation v1.89 → v1.90 documents:
- All 4 phases A→D shipped commits
- The two re-scopings (Phase C from "schema migration" to "wire dormant machinery"; Phase D scope already partly there)
- Cumulative test count 67 → 90 (+23)
- Gate 2 6-stage Boss-test plan ready

Gate 2 awaits Boss-test session.

### Re-scoping pattern observation (worth surfacing)

Two of four V3-§9 build phases (C and D) had their Architect-doc scope reduced after auditing actual source. The pattern: **the V3-§8 cascade did more vertical-axis work than the original V3-§9 plan accounted for**. So two of the four "missing pieces" the Architect doc identified turned out to already exist; the actual gap was elsewhere.

The Plan's pre-commit validation step (grep IDs through the taxonomy + audit current code before drafting implementation) caught the divergence early enough to avoid wasted work. Without it, Phase C would have been a no-op schema migration and Phase D would have rewritten an already-correct prompt.

Net wall-clock: ~2hrs of agent time vs the Plan's 4-6hr estimate, because two phases shrank significantly during implementation. All ID typos and ال-prefix gaps caught pre-commit by following the Plan's "validate first" guidance.

---

## V3-§9.C.2 — Dual-axis reliability gap (this commit)

Eisa caught this during Gate 2 Stage 3. The freshly-created `cataloger_reliability.json` had only ONE entry (`semantic.horizontal.correct = 1`) instead of the four expected entries. The Accept flow's two back-to-back IPCs (`sources_set_manual` then `content_type_set_manual`) had a silent gap: the second IPC found the suggestion row already cleared by the first IPC's `clear_suggestions` call, so its `prior_composite` was None and the reliability update was skipped.

### Root cause

V3-§9.C wired `update_reliability_from_correction` INTO each per-axis IPC. That works when only one axis is being written (PropertyEditor manual edit, single-axis correction). It silently fails on the dual-axis flow because the second call can't read what the first one deleted.

Same pattern affected `cece_resolve_disambiguation`'s auto-write path: when a Split-on-one-axis card had a settled value on the other axis, the second axis IPC also lost the composite_json snapshot.

### Fix

Refactored reliability updates OUT of the per-axis IPCs. New IPC `cece_record_correction_for_card(note_path, composite_json, horizontal_pick, vertical_pick)` takes the snapshot explicitly + updates both axes from one source.

Wired into:
- **Frontend `acceptSuggestion`**: snapshots `record.composite_json` BEFORE the two writes, calls new IPC after.
- **`cece_resolve_disambiguation`**: snapshots `composite_json` at the same time it reads `extract_other_axis_settled`, calls new IPC at the end.

Single-axis callers (PropertyEditor) don't get reliability updates — that's correct behavior since manual property edits aren't keyed to a specific suggestion row's per-cataloger trail.

### Tests

2 new V3-§9.C.2 reliability tests (the IPC's logic is mirrored in a `dual_axis_record` test helper to avoid needing a Tauri AppHandle):
- `v3_p9c2_dual_axis_accept_updates_both_axes`
- `v3_p9c2_horizontal_only_pick_updates_horizontal_only`

92 cece tests pass total (was 90 in V3-§9.E; +2).

### Files touched in V3-§9.C.2

- `src-tauri/src/sources/mod.rs` — removed `update_reliability_from_correction` calls from `sources_set_manual` + `content_type_set_manual`; kept the `prior_composite` snapshot in scope with a comment explaining the move
- `src-tauri/src/classifier/mod.rs` — added `cece_record_correction_for_card` IPC; refactored `cece_resolve_disambiguation` to snapshot composite once then call the new IPC at the end
- `src-tauri/src/lib.rs` — registered the new IPC
- `src-tauri/src/cece/reliability.rs` — added 2 V3-§9.C.2 dual-axis tests
- `src/lib/components/SourceReviewPanel.svelte` — `acceptSuggestion` snapshots `record.composite_json` and calls `cece_record_correction_for_card` after the two writes
- `docs/Constellation Orientation & Onboarding v1.91.md` — new orientation file
- `lab/reports/SESSION-LOG-2026-05-11.md` — this entry

### Lesson

When two IPCs operate on the same row in sequence, **whichever one needs to read state from that row must snapshot it before the first IPC's side-effects fire**. Lift the read into the orchestration layer (the caller making both IPC calls) and pass the snapshot explicitly to a dedicated handler. Don't rely on each per-axis IPC to re-derive state from a row that may have been mutated by its sibling.

Same spirit as V3-§8.r7 Issue #1 (where two filters using "the same logic" disagreed on the same data, fixed by routing them through one helper). The recurring pattern: when two paths share a dependency on transient state, centralize the dependency.

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

---

## Gate 2 PASS close-out + PJ-040 filed (orientation v1.92, Pending Jobs v1.8)

After V3-§9.C.2, Eisa re-ran Gate 2 Stage 3 + completed Stages 4, 5, 6.

### Stage 3 (re-test)
JSON file shows entries for FOUR catalogers (linguistic, user_authority, semantic, structural) with BOTH `horizontal` AND `vertical` sub-objects appearing for catalogers that voiced on those axes — exactly the dual-axis signature V3-§9.C.2 was designed to produce. Pre-fix, only one axis would have updated per Accept. PASS.

### Stage 4
Reasoning Cataloger silent (rightmost green dot dashed) on every card. llama.cpp not wired; abstain behavior unchanged. PASS.

### Stage 5
5 distinct vertical primaries across 5 test notes:
- Stage 1 — concept → `semantic-contents/concept`
- Stage 2.1 — worldview (EN) → `higher-order-constructs/worldview`
- Stage 2.2 — رؤية كونية (AR) → `higher-order-constructs/worldview`
- Stage 2.3 — figures → `sensory-inputs/signal/physical/electromagnetic`
- Stage 2.4 — code v2 → `symbolic-entities/sign`

The vertical axis is no longer collapsing all notes to one or two values. PASS.

### Stage 6
Re-classified `الخط العربي`. Card rendered with UA dot filled (blue), SOURCES collapsed to single `Authoritative testimony 100%` (UA short-circuit signature). Trail toggle hidden (trust-cal counter past 50-review threshold; r5.3 working as designed for Unanimous cards). No horizontal regression vs Gate 1 PASS state.

Eisa substituted with `الحضارة الإسلامية` for the trail visibility check — confirmed horizontal axis classification works end-to-end on real Arabic content (synthesis chose `testimony/authoritative` StrongMajority via 2-of-3 cataloger agreement). PASS.

### Architectural observation → PJ-040

Stage 6 surfaced an architectural observation: when UA short-circuits on PARTIAL frontmatter (e.g. only `sources:` set, no `content_type:`), `user_authority_short_circuit` produces vertical with `primary: None` → no vertical suggestion entry → CONTENT TYPE section vanishes from the card. The other catalogers' high-confidence vertical votes are discarded.

Filed as **PJ-040** (Pending Jobs v1.8): refactor `user_authority_short_circuit` to short-circuit ONLY the axes UA voiced on; for unfilled axes, fall through to normal `vote_on_axis` weighted-vote path. Not blocking V3-§10; could be a focused mini-MIG between V3-§10 phases or after Gate 3.

Behavior has been the same since V3-§1 — only became visible after V3-§9.A populated meaningful vertical lexicon coverage that would now have something to discard.

### Gate 2 PASS — V3-§9 vertical-axis activation complete

V3-§9 cumulative scoreboard:
- 6 commits shipped (`4e0981a`, `d9dfa60`, `ec5527e`, `b18a3ee`, `bf07ae1`, `75807a3`)
- +25 cece tests (67 → 92)
- 2 Boss-test catches mid-cascade (V3-§9.A's ال-prefix gap, V3-§9.C.2's dual-axis silent gap), both fixed inline
- ~3hrs of agent time + Eisa's two Boss-test sessions
- Orientation v1.89 → v1.92 (3 versions documenting the cascade + close-out)
- 1 PJ filed (PJ-040)

Both cataloger-ensemble axes are now production-ready. Next: V3-§10 (Settings + i18n + Help docs + User Manual) — user-facing surfaces around the engine.

---

## V3-§10 cascade — User-facing surfaces (Option C, full cascade A→G)

After Gate 2 close-out, Eisa picked Option C for V3-§10 (Settings UI + en+ar i18n + EN docs + 13-locale i18n backfill + 14-locale help topic + 14-locale User Manual chapter). Architect doc + Plan written and approved; cascade landed in 7 commits.

### V3-§10.A — Settings UI + IPC + appSettings.cece (`d44b115`)

New "Constellation Epistemic Content Engine" Settings section under Intelligence with 4 setting rows: Reasoning Cataloger model status (read-only — "Not downloaded, deferred to V3-§7.b" + disabled "Coming soon" button), Reasoning trail visibility dropdown (Always / On disagreement (default) / Never), Background classification dropdown (Off (default) / On note save / On app start), Per-Library calibration collapsible.

Backend additions: `cece_get_reliability_for_active_library(note_path)` + `cece_get_active_library_root(note_path)` IPCs in `reliability.rs`. Library resolution: note_path → containing Library; fallback to first Library when no note open. Returns empty default when no reliability JSON exists yet.

Frontend additions:
- New `<PerLibraryCalibrationView>` Svelte component (read-only table with empty-state, "(uniform)" labels for catalogers below MIN_SAMPLES_FOR_WEIGHTING=20 threshold).
- `appSettings.cece` sub-object with `reasoningTrailVisibility` + `backgroundScan` flags. Defaults preserve pre-V3-§10 behavior.
- `SourceReviewPanel.svelte::isTrailOpen()` now respects the visibility setting.
- `NoteEditor.svelte::handleSave()` fires `classifier_suggest_for_note` after disk write when `backgroundScan === 'on_save'`. Rides existing 1500ms debounced save — never fires per-keystroke.
- `+layout.svelte::onMount()` fires `classifier_scan_start` 5s after boot when `backgroundScan === 'on_startup'`.

### V3-§10.B — en + ar i18n for cece.settings.* (`0054981`)

28 new keys added to en.json + ar.json::cece.settings.

### V3-§10.C — EN help topic + EN User Manual chapter (`34a96a9`)

New `docs/help.uConstellation.World/Source Review/Source Review.md` topic — ~3500 words, 13 sections covering: what CECE does, two axes plain-language, six catalogers (with lens-color guide), three confidence regimes, Sibling Disambiguation walkthrough, reasoning trail, queue composition filter, per-card actions, trust-calibration period, per-Library calibration, background classification, common workflows.

New `## 10b. Source Review (CECE)` chapter in `docs/User Manual.md` — ~800 words mirroring the help topic at User Manual depth.

Cross-reference added to `Cognitive Engine` topic.

### V3-§10.D — 13-locale i18n backfill (`259c333`)

All 13 non-en/non-ar locales got the full `cece` block (~90 keys each) translated. Done via 5 parallel agents per language family:
- Romance (de, es, fr) — agent adce9fb08dc6afde2
- Iberian + Slavic (pt, ru) — agent a1c61aa1249170478
- Arabic-script + Hebrew (fa, ur, he) — agent ad82ded015191e0c3
- CJK (ja, ko, zh) — agent add3540ff7d2025d7
- Turkish + Hindi (tr, hi) — agent a21385f53ed8bc681

Each block has a `_translation_note` disclaimer in target language. JSON parse-validity verified for all 13. Insertion point near `panels`/`migrationProgress` in most files.

### V3-§10.E — 14-locale help topic translations

New `docs/help.{locale}/Source Review/Source Review.md` for all 14 non-English locales. Each starts with translated disclaimer header. Done via 5 parallel agents:
- Romance + Iberian (de, es, fr, pt) — agent a8ee26fec10b8a85a
- Slavic + Turkic (ru, tr) — agent a3824cb128000dbae
- Arabic-script + Hebrew (ar, fa, ur, he) — agent a572c37f35a27d71f
- CJK (ja, ko, zh) — agent ac0be44d26f89925e
- Hindi (hi) — agent a521c08fd71d7e9d4

File sizes verify content presence (range 14KB zh — denser CJK — to 39KB hi — Devanagari with parenthetical English glosses); word counts within ±25% of 2300-word English source.

### V3-§10.F — 14-locale User Manual chapter translations (`50a67b0`)

Each translated User Manual got the new chapter inserted at appropriate position (10b for Latin/CJK, 10ب for fa/ur, 10ב for he, 11ب for ar where ch10 is Second Screen). TOC entries added. Done via 2 parallel agents:
- Romance + Slavic + Turkic (de, es, fr, pt, ru, tr) — agent a55075ffe6b7ec2fa
- Arabic-script + Hebrew + CJK + Hindi (ar, fa, ur, he, ja, ko, zh, hi) — agent af4bbb0923096de53

### Translation honesty

Per Option C's risk register: every translated file carries an AI-translation disclaimer in the target language. The `_translation_note` field in i18n JSON + the inline disclaimer at the top of help/User Manual files are the honest signals.

Agents flagged specific terms worth native-speaker review:
- The 11 Source axis values translate cleanly into ar (matches `ar.json::sources.label` exactly) but renderings for fa/ur/he/ja/ko/zh/hi are plain-language paraphrases of Sunni/Hindu nyāya tradition terms. Highest-priority follow-up.
- "Sibling Disambiguation" kept in Latin form alongside locale translation (UI feature name).
- "Living Links" preserved Latin across all locales.

### V3-§10.G — NSIS + orientation v1.93 + Gate 3 ready (this commit)

NSIS rebuilt. Orientation v1.92 → v1.93. Gate 3 Boss-test ready per Plan §7 (7 stages: build installed, Settings section renders, trail visibility setting works in 3 modes, background scan doesn't fire on keystroke, per-Library calibration view shows real data, i18n in 13 other locales, help topic discoverability, User Manual chapter present).

### Cascade scoreboard

7 commits + 1 close-out = 8 phases total. Estimated wall-clock per the Plan was 12-15hrs of agent time; actual ~3-4hrs (parallel agent translation accelerated D/E/F substantially). All translation work shipped with disclaimer headers per Option C's risk register.

If Gate 3 PASSes: V3-§11 final integration audit + MIG-021v3 entire close-out.

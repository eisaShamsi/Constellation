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

---

## Gap analysis acknowledgment (mid-Gate-3 pause)

Eisa surfaced an analytical addendum mid-Gate-3 — `docs/epistemic-content-gap-analysis.md` (Eisa 2026, "Gap Analysis of the Universal Epistemic Content Model"). The piece names three structural gaps in the two-axis (Source × Content Type) framework that CECE classifies against:

1. **Temporal / dynamic axis** — beliefs change over time; CECE today is static
2. **Justification / warrant axis** — source ≠ warrant (mutawātir vs anonymous forum post both qualify as "testimony"); Sunni uṣūl has rich grading vocabulary (mutawātir / mashhūr / āḥād / ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ) that CECE collapses into source
3. **Contestation / agent axis** — the model assumes the user's stance; doesn't represent ikhtilāf or scholar/school-attributed stances

Plus five minor extensions (domain, function, confidence-as-probability, linguistic provenance, logical relations between notes).

Eisa asked for my opinion before proceeding with Gate 3. I offered three paths:
- (A) Close Gate 3 on V3-§10 as scoped; file gap analysis as input for separate MIG-022 workstream
- (B) Pause Gate 3, expand V3-§10 scope to include §6.1 YAML metadata extensions
- (C) Run Gate 3 in smaller form (Stages 1-3); start gap analysis Architect doc in parallel

Recommended (A) — the gap analysis is a serious scholarly piece deserving its own architectural response, not a mid-cascade rushed extension. The §6.2 justification classifier alone is bigger than V3-§10 + V3-§11 combined (multi-month research project requiring labeled hadith corpus, domain experts, different model architecture).

Pushback I offered on the analysis itself:
- §6.1's `warrant: "mutawātir"` flat string under-specifies what's actually a multi-dimensional grading (chain × text × derived conditions)
- §6.2 framing as "another classifier head" understates the work
- §6.1's `supersedes` / `contradicts` overlap with the existing Living Links architecture — should extend Living Link types, not add parallel YAML fields
- §5.2 dhawq deserves more weight (recognized Sufi epistemic category that doesn't fit the 11 sources cleanly)

Eisa picked (A) and agreed with the pushback. Action items landed:

1. Gap analysis doc copied into repo at `docs/epistemic-content-gap-analysis.md` (alongside the existing companion papers)
2. Orientation v1.93 "what V3-§10 does NOT do" section now explicitly acknowledges the three structural gaps + names MIG-022 as the response workstream
3. Gate 3 Boss-test resumes (Stages 0-3 instructions already sent before the pause)
4. After Gate 3 PASS + V3-§11 close-out, write MIG-022 Architect doc mapping §6.1/§6.2/§6.3 to MIG candidates with effort + risk estimates

---

## Gate 3 PASS close-out + V3-§10.A.1 + V3-§10.D.2 + 3 new PJs (this commit)

After landing the gap analysis acknowledgment, Eisa resumed Gate 3. All 7 stages PASSed with two inline fixes.

### Stage 3 (re-test on V3-§10.A.1)
The `4ede8ef` fix worked — typing-perf preserved, queue updates after debounced save. Root cause was that V3-§10.A's direct-IPC call from NoteEditor.handleSave never notified the Source Review panel. Fix: dispatch the same `constellation:classify-and-show` window event the right-click context menu uses; the panel's existing `handleClassifyAndShow` handles the IPC + queue prepend + flash highlight in one path. Same lesson as V3-§9.C.2 (centralize shared-flow dependencies).

### Stage 5 (i18n catches)
Three observations from Eisa's screenshots in German + Chinese + Arabic:

1. **"Sources & content type classifier" section header in English** — V3-§10.D backfilled `cece.*` keys but missed `settings.classifier.*` (5 keys added by MIG-021v2 §1F' before V3, never translated beyond en + ar). Fixed inline as **V3-§10.D.2** (`54276c3`) via a Python batch script — translations inline-supplied, JSON-validated, all 13 locales now have `settings.classifier` populated.

2. **Per-cataloger reasoning prose in English** — visible on the Arabic-UI screenshot. The `reasoning: String` field is generated by Rust cataloger code at classification time via `format!()` with hardcoded English templates and stored verbatim in `composite_json`. Frontend renders raw via `{t.reasoning}` — never goes through `$t()`. Filed as **PJ-041**.

3. **Per-card SOURCES/CONTENT TYPE values + Sibling Disambig chips in English/Arabic only** — visible on the German screenshot. The vertical (225 nodes) + horizontal (~30 nodes) taxonomy data has `en` + `ar` fields only. Filed as **PJ-043**.

Also caught (related to PJ-041): the `[high]`/`[medium]`/`[low]` confidence enum bypasses i18n entirely. Filed as **PJ-042** (smallest of the three; ~30 min fix).

### Stages 6 + 7
Help topic discoverable in en + de. User Manual chapter present in en + de. Both PASS.

### Decision: Path (A) — file three new PJs for MIG-022 cascade

The three i18n gaps (PJ-041, PJ-042, PJ-043) are structural — the strings don't go through `$t()` to begin with, so V3-§10.D's backfill couldn't have addressed them. They share a theme with the gap analysis (the engine output structure has assumptions that don't scale to the user's full model) and belong in the MIG-022 cascade.

V3-§10's i18n scope was honest from the start: "strings the frontend sends through `$t()`". Orientation v1.94 documents what V3-§10 does NOT translate so future sessions don't get confused.

### Files in scope this commit

- `src/lib/i18n/{de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — added `settings.classifier` block via Python batch (V3-§10.D.2 — 13 files, 91 insertions)
- `src/lib/components/NoteEditor.svelte` — V3-§10.A.1 dispatch fix (already shipped in `4ede8ef` earlier)
- `docs/Constellation Pending Jobs v1.9.md` — new file with PJ-041 / PJ-042 / PJ-043
- `docs/Constellation Orientation & Onboarding v1.94.md` — new file with Gate 3 PASS scoreboard + "what V3-§10 does NOT translate" section
- `lab/reports/SESSION-LOG-2026-05-11.md` — this entry

### V3-§10 cumulative scoreboard

9 commits across A → G + A.1 + D.2 + this close-out (`d44b115`, `0054981`, `34a96a9`, `259c333`, `7d6e1a0`, `50a67b0`, `a4438ac`, `4ede8ef`, `54276c3`, plus this commit). Two Boss-test catches landed inline. Three deeper i18n gaps filed for MIG-022.

### What's next

- V3-§11 (final integration audit + MIG-021v3 entire close-out)
- MIG-022 (gap analysis response + PJ-041/042/043 + the three structural axis gaps)

---

## V3-§11 — Final integration audit + MIG-021v3 close-out (this commit)

Three parallel agents (invariants / drift / migration-path) ran the integration audit over the entire MIG-021v3 cascade — V3-§1 → V3-§10, every .r/.X follow-up, every Gate. Verdict: **MIG-021v3 ships**, conditional on one AT RISK fix that landed in this same commit.

### Audit agent results

**Agent 1 — Invariants (9 HOLDS · 1 AT RISK · 0 VIOLATED):**
- HOLDS: 6-cataloger ensemble dispatch · tier timeouts (cheap=500ms / medium=2s / expensive=5s) · three confidence regimes · Sibling Disambiguation contract · per-Library reliability per-axis-aware · queue composition filter regime semantics · Reasoning Cataloger GBNF interface lock-in · all 90 cece.* + cece.settings.* keys present in 15 locales · Editor Parity Rule unaffected (CECE runs out-of-band of CM6 hot path)
- AT RISK: V3-§10 Settings flag persistence — `cece` block declared in `AppSettings` interface but missing from `DEFAULT_SETTINGS` and `loadSettings` deep-merge. All 5 current consumers use defensive `?.` + `??` so latent-correct today, but a future consumer reading `$appSettings.cece.someFlag` without optional-chain would throw, and the deep-merge omission means a future cece sub-key would silently overwrite saved settings on load.
- VIOLATED: 0

**Agent 2 — Drift (1 P1 + 3 P3s, nothing blocking):**
- F1 (P1) — Legacy classifier dead code (`tier1_embedding.rs`, `tier1_rules.rs`, large parts of `source_definitions.rs`); orchestrator + synthesis + 6 catalogers replaced them in V3-§8 but the modules still compile. Candidate cleanup MIG (~1-2hrs).
- F2 (P3) — Unused import in `wiring.rs` (residual from V3-§9.C.2 refactor)
- F3 (P3) — Magic number `0.85` in `reasoning.rs` should be a named const
- F4 (P3) — Dead `write_suggestions` from earlier draft of synthesis.rs
- F5 — False alarm (Pending Jobs v1.8/v1.9 exist; agent caught a stale state)

**Agent 3 — Migration path (all 7 scenarios PASS · LOW risk):**
- First-boot · v2-era upgrade (composite_json column ALTER) · reliability JSON format variations · mid-classification interrupt · Settings flag back-compat (PASS today only because consumers are defensive — strengthened to contractually PASS by the AT RISK fix this commit) · i18n locale switching · rollback to V3-§7

### The AT RISK fix (this commit)

Added 4 lines to `src/lib/libraries/store.ts`:
- `cece: { reasoningTrailVisibility: 'on_disagreement', backgroundScan: 'off' }` in `DEFAULT_SETTINGS`
- `cece: { ...DEFAULT_SETTINGS.cece, ...((parsed.cece as Record<string, unknown>) || {}) }` in `loadSettings` spread

No behavior change for current users; contract-strengthening only. Verified all 5 consumer call sites (`+layout.svelte:2088`, `NoteEditor.svelte:182`, `SourceReviewPanel.svelte:256`, `SettingsModal.svelte:1867,1882`) still compile and still use defensive defaults — so this is purely defense-in-depth for future consumers.

### Files in scope this commit

- `src/lib/libraries/store.ts` — AT RISK fix (+11 lines: 5 in DEFAULT_SETTINGS, 6 in loadSettings)
- `lab/reports/MIG-021v3-V3-§11-FINAL-INTEGRATION-AUDIT.md` — new file consolidating the 3 agent reports
- `docs/Constellation Orientation & Onboarding v1.95.md` — new file marking "MIG-021v3 ships"
- `lab/reports/SESSION-LOG-2026-05-11.md` — this entry

### MIG-021v3 cumulative scoreboard

| Phase | Commits | What | Outcome |
|---|---|---|---|
| V3-§1 → V3-§7 | many | engine spine (6 catalogers, GBNF, synthesis, IPC, reasoning prompt) | engine internals shipped |
| V3-§8 | `daeba00` `3f486b4` `191fb8c` `663e31f` `23e50c0` `72629fc` `84cde6f` `c355be1` `b366d97` `cf03670` `787b64d` | orchestrator wired + 6 audit fixes + UX polish + queue filter | Gate 1 PASS |
| V3-§9 | `4e0981a` `d9dfa60` `ec5527e` `b18a3ee` `bf07ae1` `75807a3` `d5fc070` | vertical-axis activation + dual-axis reliability fix | Gate 2 PASS |
| V3-§10 | `d44b115` `0054981` `34a96a9` `259c333` `7d6e1a0` `50a67b0` `a4438ac` `4ede8ef` `54276c3` `237871f` | Settings + 15-locale i18n + EN+14 help + EN+14 User Manual | Gate 3 PASS |
| V3-§11 | this commit | AT RISK fix + audit report + orientation v1.95 | **MIG-021v3 ships** |

**Tests:** 92 cece tests, all PASS. **Gates:** 1 PASS · 2 PASS · 3 PASS · 4 (V3-§11) PASS.

**PJs filed during cascade:** PJ-040 · PJ-041 · PJ-042 · PJ-043.

### What's next

- **MIG-022** — Architect doc responding to:
  - Gap analysis (`docs/epistemic-content-gap-analysis.md`) — temporal axis, justification/warrant axis, contestation/agent axis, plus the five minor extensions
  - PJ-040 (UA partial-frontmatter)
  - PJ-041 / PJ-042 / PJ-043 (engine-output i18n gaps)
  - F1 from this audit (legacy classifier dead-code cleanup, §0 housekeeping)
  - Boss decides which pieces to pursue when; the order is decided in the Architect cycle, not pre-committed here.

---

## MIG-022 Architect filed (this commit)

Phase 1 of `/migration` for the MIG-022 cascade. `lab/reports/MIG-022-ARCHITECT.md` maps the territory across six work clusters and surfaces eight decisions for Eisa to lock before Phase 2 (Plan) opens.

### Source material reviewed

- `docs/epistemic-content-gap-analysis.md` (339 lines, full read) — three structural gaps + five minor extensions + §6.1/§6.2/§6.3 recommendations
- `docs/Constellation Pending Jobs v1.9.md` — PJ-040, PJ-041, PJ-042, PJ-043 entries
- `lab/reports/MIG-021v3-V3-§11-FINAL-INTEGRATION-AUDIT.md` — F1 dead-code finding context
- `src-tauri/src/cece/cataloger.rs` — confirmed `Confidence` enum + serialization
- `src-tauri/src/cece/synthesis.rs` lines 92-167, 423-433 — confirmed UA short-circuit shape
- `src-tauri/src/cece/catalogers/{structural,linguistic}.rs` — confirmed `build_reasoning` exists at structural.rs:366 and linguistic.rs:463
- `src-tauri/src/sources/vertical_taxonomy.rs` — confirmed `VerticalNode { id, en, ar, parent_id, branch }` shape
- `src-tauri/src/classifier/mod.rs` head — confirmed legacy module declarations still exist; reachability of `classifier_suggest_for_note` flagged as Plan-phase verification

### Six work clusters identified

| Cluster | What | Effort | Risk |
|---|---|---|---|
| A | §6.1 YAML metadata extensions | 3-5 days | Low-medium |
| B | §6.3 Temporal axis via history layer | 2-4 weeks | Medium-high |
| C | §6.2 Warrant classifier (Sunni uṣūl) | Multi-month research | High |
| D | PJ-040 fix (partial UA short-circuit) | ½ day | Low |
| E | PJ-041/042/043 i18n bundle | 1-2 weeks | Low-medium |
| F | Legacy classifier cleanup | ½ day | Low |

### Recommended MIG-022 scope

Architect §5 recommends MIG-022 = **§0 Cleanup (F) + §A YAML metadata + §D UA fix + §E i18n** = ~3 weeks of agent time, two Boss-test gates. Cluster B deferred to MIG-023; Cluster C deferred to a Constellation Warrant Research workstream.

### Eight decisions surfaced for Eisa

1. D-A1 — `supersedes`/`contradicts` as YAML scalars vs. typed-link names (rec: typed-link names)
2. D-A2 — Warrant display in Cluster A or all warrant UI deferred (rec: display now)
3. D-A3 — `taxonomy_version` now or stay implicit (rec: implicit)
4. D-B3 — Cluster B in MIG-022 or split to MIG-023 (rec: split)
5. D-C1 — Cluster C in MIG-022 or workstream-deferred (rec: workstream-deferred)
6. D-E1 — PJ-043 storage: struct fields / per-locale JSON / frontend i18n keys (rec: frontend keys)
7. D-E2 — Cluster E one bundled MIG or three separate (rec: one MIG with three sub-phases)
8. D-F1 — Cluster F as MIG-022 §0 or its own mini-MIG (rec: §0 housekeeping)

### Cross-cutting invariants identified (§3)

Ten invariants any MIG-022 work must NOT break: File-over-app, Performance Rule 1 (keystroke), Performance Rule 8 (write-time derivation), Editor Parity, Living Link Architecture, CECE 6-cataloger contract, i18n parity, Reliability data continuity, Migration from CECE v1 universes, No regression on boot/typing latency.

### Plan-phase verification queue (§8)

Five Architect-phase claims that the Plan must verify in code before proceeding:

1. F1 reachability — is `classifier::*` actually unreachable? (head of `mod.rs` shows `#[tauri::command]` declarations still)
2. `build_reasoning` confirmed in 2/6 catalogers; the other 4 stated by PJ-041 but not yet directly verified
3. Frontmatter parser path — needs to be located, not just guessed
4. Properties panel path — needs to be located
5. Schema migration testing on Eisa's primary 7,600-note universe — perf bar

### Files in scope this commit

- `lab/reports/MIG-022-ARCHITECT.md` — new, ~720 lines
- `docs/Constellation Orientation & Onboarding v1.96.md` — new, "MIG-022 cascade opens" preamble
- `lab/reports/SESSION-LOG-2026-05-11.md` — this entry

### What's next

**Eisa locks the eight §6 decisions.** Then Phase 2 (Plan) lays out commits + verification clauses + Boss-test gates. Phase 3 (Build) cascades. Phase 4 (Audit) runs the three-agent integration check.

---

## MIG-022 Architect decisions locked + Phase 2 Plan filed (this commit)

Eisa locked all eight §6 decisions in one message. Two diverged from my recommendations:

- **D-A2: β** (defer all warrant UI to Warrant Research) — I had recommended α (warrant display in Cluster A). Eisa's call: warrant fields parse but stay inert in MIG-022; the entire warrant surface (display + classifier) ships as one cohesive workstream.
- **D-B3: IN MIG-022** (Cluster B included, not split to MIG-023) — I had recommended split. Eisa's call: temporal axis is in MIG-022 scope; ~5-7 weeks total wall-clock; one cohesive cascade.

Plus the explicit **D-C1 commitment**: Warrant Research opens as a workstream **immediately after MIG-022 closes** (not "someday"). Concept Paper to follow.

### Cross-check applied (WA #5)

Spawned a general-purpose agent to cross-check D-B1 (SQLite + triggers for `note_state_history`) against industry patterns: Datomic / XTDB bitemporal, SQL:2011 SYSTEM_TIME, event sourcing, per-file Git versioning. Verdict: SQLite + triggers is the right pattern for local-first SQLite (Datasette uses it via `sqlite-history-json`). Three refinements adopted:

1. **JSON-diff column shape** — single `changes_json TEXT` instead of `(axis_changed, old_value, new_value)` triples. Adding new epistemic fields later doesn't require rewriting the trigger.
2. **`WHEN OLD.field IS NOT NEW.field` trigger guard** — prevents trigger fires on no-op writes.
3. **`DROP TRIGGER` + bulk-insert during backfill** — the §B.3 backfill on a 7,600-note universe could otherwise blow up wall-clock by firing 7,600 sequential trigger evaluations.

Soft-delete vs CASCADE deferred to §B.1 micro-decision.

### Phase 2 Plan landed: 14-16 phases, 5 Boss-test gates

```
§0 — Cleanup (Cluster F)              ──► Verification: cargo build + test pass
§D — UA partial-frontmatter (PJ-040)  ──► Boss-Test Gate 1
§E.1 — Confidence enum i18n (PJ-042) ──┘
§E.2 — Cataloger reasoning i18n (PJ-041) ──► Boss-Test Gate 2
§E.3 — Taxonomy labels i18n (PJ-043) ──┘
§A.1 — Frontmatter schema additions ──┐
§A.2 — Typed-link extension (+supersedes) ──► Boss-Test Gate 3
§A.3 — Properties panel UI ───────────┤
§A.4 — i18n + help docs ───────────────┘
§B.1 — note_state_history schema ─────┐
§B.2 — Trigger + WHEN guards          │
§B.3 — Backfill (existing notes)      ├──► Boss-Test Gate 4
§B.4 — Query API IPC                  │
§B.5 — UI surface (per D-B4)          │
§B.6 — i18n + help + User Manual ─────┘
§N — Final integration audit ─────────► Boss-Test Gate 5 + MIG-022 ships
```

### Two new Plan-level decisions for Eisa

Surfaced in MIG-022-PLAN.md §6:

1. **D-A4** — `ikhtilāf` Properties panel UX: (α) full widget / (β) raw YAML edit / (γ) read-only display? *Rec: (β) — niche field; raw YAML in MIG-022; polished widget as future PJ.*
2. **D-B4** — Temporal-axis UI surface: (α) side-panel / (β) Sight v3 overlay / (γ) search filter? *Rec: (α) — matches Backlinks panel pattern; doesn't couple MIG-022 to Sight v3 stability.*

### Files in scope this commit

- `lab/reports/MIG-022-PLAN.md` — new, ~750 lines
- `docs/Constellation Orientation & Onboarding v1.97.md` — new
- `lab/reports/SESSION-LOG-2026-05-11.md` — this entry

### What's next

Eisa answers D-A4 + D-B4 + approves the Plan. **Plan-Approval-Equals-Build-Approval** kicks in: I cascade through §0 → §D → §E.1 → §E.2 → §E.3 → §A.1-§A.4 → §B.1-§B.6 → §N autonomously, stopping only at the 5 Boss-test gates and on genuine architectural surprise.

After MIG-022 closes (Phase 4 audit + close-out commit): the **Constellation Warrant Research** workstream opens with its own Concept Paper.

---

## MIG-022 §0 — Legacy classifier cleanup ships (this commit)

Phase 3 Build cascade opens. §0.1 reachability analysis filed at `lab/reports/MIG-022-§0-REACHABILITY.md`; §0.2 deletion + comment fixes land in this commit.

### Decisions locked just before this commit

- **D-A4: α** (full custom widget for `ikhtilāf` Properties panel) — diverges from my recommendation (β raw YAML); means §A.3 grows ~2 days for the polished widget.
- **D-B4: β** (Sight v3 overlay for temporal-axis UI surface) — diverges from my recommendation (α side-panel); aligns with Constellation's universe-as-cosmos metaphor (gap-analysis §6.3 queries are universe-wide). Couples §B.5 to Sight v3 stability — flagged in the Plan's risk register.

### §0 deletion summary

**Deleted (3 files, 988 LoC of dead code):**
- `src-tauri/src/classifier/source_definitions.rs` (323 LoC) — only consumer was `tier1_embedding.rs`
- `src-tauri/src/classifier/tier1_embedding.rs` (345 LoC) — zero production callers; superseded by V3-§8 CECE 6-cataloger ensemble
- `src-tauri/src/classifier/tier1_rules.rs` (320 LoC) — only callers were its own `#[test]` functions

**Kept (still reachable):**
- `classifier/mod.rs` — 3 active `#[tauri::command]`s (`classifier_suggest_for_note`, `cece_record_correction_for_card`, `cece_resolve_disambiguation`)
- `classifier/scan_job.rs` — 3 active `#[tauri::command]`s + `ScanState` Tauri-managed
- `classifier/correction_log.rs` — `library_root_for_note` has 4 call sites across `sources/mod.rs`, `cece/reliability.rs`, and `classifier/mod.rs`

**Comment cleanup:**
- `mod.rs:16-21` — module declarations replaced with audit-F1 historical note
- `mod.rs:28-31` — docstring updated ("Tier 1 classification" → "CECE 6-cataloger ensemble")
- `mod.rs:60-67` — misleading "tier1_* consumed by catalogers via wiring" comment replaced with accurate note that CECE replaced the v2 classifier wholesale
- `mod.rs:554-556` — dead reference to `super::source_definitions` and `super::tier1_embedding` removed
- `sources/vertical_taxonomy.rs:18` — reference to deleted `classifier::source_definitions` updated to historical note

### The misleading audit clue traced

The V3-§11 audit drift agent's F1 finding said classifier modules were dead. The misleading evidence was a comment in `mod.rs:64` claiming the v2 `tier1_*` modules were "consumed by Linguistic, Structural, Semantic catalogers via the wiring layer." Verified false by grep — `cece/wiring.rs` and all six cataloger files have zero `tier1_*` references. The catalogers each implement their logic from scratch. The aspirational comment from V3-§8 planning never got updated when implementation took a different path.

### Verification

- **`cargo build --lib`**: clean (1m 59s; 41 pre-existing warnings, zero from this change)
- **`cargo test --lib`**: 627 passed, 2 failed, 3 ignored
  - Both failures are in `search.rs` (`tests_m12::unknown_word_falls_back_to_none`, `tests_m8c::reindex_updates_all_matching_rows_in_one_pass`)
  - Verified pre-existing by stashing my changes and re-running the same two tests on the prior tree — both still failed
  - Verified independent by `Grep "classifier::|tier1_|source_definitions" src/search.rs` returning zero matches
- **Test count delta** matches expectation: deleted modules carried 5 internal tests (`tier1_rules.rs`) + 3 internal tests (`source_definitions.rs`) = 8 tests gone

### Per-cataloger test count

The 92 cece tests from V3-§11 are unaffected (they live in `src-tauri/src/cece/`, not in the deleted classifier files). No `cece` test broken by the change.

### Files in scope this commit

- `src-tauri/src/classifier/mod.rs` — module decls + comment cleanup
- `src-tauri/src/classifier/source_definitions.rs` — deleted
- `src-tauri/src/classifier/tier1_embedding.rs` — deleted
- `src-tauri/src/classifier/tier1_rules.rs` — deleted
- `src-tauri/src/sources/vertical_taxonomy.rs` — comment cleanup
- `lab/reports/MIG-022-§0-REACHABILITY.md` — new reachability report

### What's next: §D — PJ-040 partial UA short-circuit

§0 is internal cleanup with no user-visible behavior change; §D is the next phase in the cascade. §D refactors `synthesis.rs::user_authority_short_circuit` to short-circuit per-axis (instead of both-axes-or-nothing), so frontmatter with only `sources:` set surfaces a synthesized vertical primary too. ½ day, 3 new tests, internal change with user-visible effect (Boss-Test Gate 1 fires after §D + §E.1 both land).

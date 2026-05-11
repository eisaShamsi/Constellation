# MIG-021v3 — V3-§11 Final Integration Audit

**Date:** 2026-05-11
**Scope:** MIG-021v3 entire (V3-§1 → V3-§10, all .r/.X follow-ups, all 4 Gates)
**Method:** Three parallel agents (invariants / drift / migration-path) + a single round-up by the orchestrator
**Verdict:** **SHIP**, conditional on the one AT RISK item below — fix landed in this commit.

---

## 1 · Executive summary

MIG-021v3 (the **Constellation Epistemic Content Engine — CECE**) is integration-clean. The 6-cataloger ensemble, the Source × Content Type dual axis, the per-Library Bayesian reliability layer, the Sibling Disambiguation flow, the trust-calibration period, the queue composition filter, the Settings UI, the per-Library calibration view, the on-save / on-startup background scan, the 15-locale i18n surface, the EN help topic + 14-locale translations, and the User Manual chapter all compose correctly across V3-§1 → V3-§10.

The audit found:

| Agent | Verdict | Findings |
|---|---|---|
| 1 — Invariants | **9 HOLDS · 1 AT RISK · 0 VIOLATED** | AT RISK = V3-§10 Settings flag persistence; `cece` block declared in `AppSettings` interface but missing from `DEFAULT_SETTINGS` and from the `loadSettings` deep-merge. All current consumers use `?.` + `??`, so today this is latent — but a future consumer reading `$appSettings.cece.someFlag` without the optional-chain would throw. **Fixed in this commit (4 lines added to `src/lib/libraries/store.ts`).** |
| 2 — Drift | **A few findings worth filing; nothing blocking** | F1 = legacy classifier dead code (P1, candidate cleanup MIG). F2 = unused import in `wiring.rs` (P3). F3 = magic number `0.85` in `reasoning.rs` (P3). F4 = dead `write_suggestions` (P3). F5 = false alarm (Pending Jobs v1.8/v1.9 exist; agent caught a stale state). 92 cece tests PASS. |
| 3 — Migration path | **All 7 scenarios PASS** | First-boot, v2-era upgrade, reliability JSON formats, mid-classification interrupt, settings flag back-compat, i18n locale switching, rollback to V3-§7 — all clean. One soft observation matching Agent 1's AT RISK. **LOW** migration risk overall. |

**Final verdict:** ship MIG-021v3. The AT RISK fix is in this commit; the dead-code finding becomes its own focused cleanup MIG candidate; the rest are P3 polish that can ride a future MIG-022 or get filed individually.

---

## 2 · Agent 1 — Invariants check

The agent walked the contracts that V3-§1 → V3-§10 each established, and verified each one still holds after the full cascade.

### 2.1 Holds (9)

1. **6-cataloger ensemble dispatch** — `orchestrator.rs` still spawns User-Authority, Structural, Linguistic, Graph, Semantic, Reasoning per classification request; each writes a `composite_json` row entry; synthesis reads all six.
2. **Tier timeouts honored** — UA + Structural = 500ms; Linguistic + Graph + Semantic = 2s (V3-§8.r6 corrected Linguistic from 500ms); Reasoning = 5s. Test `linguistic_gets_medium_timeout_not_cheap` enforces.
3. **Three confidence regimes preserved** — `compute_regime` still emits `Unanimous` / `StrongMajority` / `Split` based on the V3-§8.r1.d ratio threshold; `secondary` populated where regime allows (V3-§8.r2.c).
4. **Sibling Disambiguation contract** — when both axes settle but for different sibling concepts, the panel emits radio chips; on resolution, `cece_resolve_disambiguation` writes BOTH axes (V3-§8.r7's `extract_other_axis_settled` fix).
5. **Per-Library reliability is per-axis-aware** — V3-§9.C.2's `cece_record_correction_for_card` writes both axes atomically from the composite blob; `MIN_SAMPLES_FOR_WEIGHTING = 20` and `TRUST_CAL_THRESHOLD = 50` honored throughout.
6. **Queue composition filter respects regime semantics** — `cardNeedsUserCall()` uses `needs_user_disambiguation_between` rather than raw split-count; "Approve All" math agrees with chip filter math (V3-§8.r7 + V3-§8.r8).
7. **Reasoning Cataloger GBNF interface lock-in** — `build_gbnf_horizontal_only`, `build_gbnf_vertical_only`, `build_gbnf_combined` all emit grammars matching the V3-§7.b interface contract; `extract_other_axis_settled` consumes whichever subset comes back.
8. **i18n: every Source Review chrome string goes through `$t()`** — verified all 28 cece.settings.* keys + ~62 cece.* keys present in all 15 locales (en + 14 translations, each with disclaimer headers per V3-§10's translation honesty contract).
9. **Editor Parity Rule unaffected** — CECE classification runs OUT-OF-BAND of CM6 hot path; zero `invoke()` calls added to keystroke path; on-save scan only fires after the 1500ms debounce already established by NotePane.

### 2.2 At Risk (1)

**V3-§10 Settings flag persistence — `cece` block missing from `DEFAULT_SETTINGS` and from `loadSettings` deep-merge.**

- `src/lib/libraries/store.ts:3299` — interface declares `cece?: { reasoningTrailVisibility?, backgroundScan? }`
- `src/lib/libraries/store.ts:3319-3490` (the `DEFAULT_SETTINGS` literal) — **no `cece` block**
- `src/lib/libraries/store.ts:3502-3524` (the `loadSettings` deep-merge) — handles `skyView`, `index`, `security`, `enabledFeatures`, `customShortcuts`, `panelPlacements` explicitly; **no `cece` merge**

Today's consumers all use optional-chain + nullish-coalesce defaults:
- `+layout.svelte:2088` — `($appSettings.cece?.backgroundScan ?? 'off') === 'on_startup'`
- `NoteEditor.svelte:182` — `($appSettings.cece?.backgroundScan ?? 'off') === 'on_save'`
- `SourceReviewPanel.svelte:256` — `$appSettings.cece?.reasoningTrailVisibility ?? 'on_disagreement'`
- `SettingsModal.svelte:1867,1882` — both reads use `?.` + `??`

So the system is latent-correct today. But the latent risk is real: a future consumer reading `$appSettings.cece.someFlag` (no `?.`) would throw. And the deep-merge omission means any future cece sub-key added by a release would silently overwrite the user's other cece settings on load if the saved object is non-empty.

**Fix (landed in this commit):** 4 lines to `src/lib/libraries/store.ts` — `cece: { reasoningTrailVisibility: 'on_disagreement', backgroundScan: 'off' }` added to `DEFAULT_SETTINGS`; `cece: { ...DEFAULT_SETTINGS.cece, ...((parsed.cece as Record<string, unknown>) || {}) }` added to `loadSettings` spread.

### 2.3 Violated (0)

None.

---

## 3 · Agent 2 — Drift check

The agent looked for new guards / contracts the system added through V3-§1 → V3-§10 that other parts of the codebase don't yet know about (LL-023 class — drift between subsystems where each thinks the other is doing the work).

### 3.1 F1 — Legacy classifier dead code (P1)

`src-tauri/src/classifier/tier1_embedding.rs`, `tier1_rules.rs`, and large parts of `source_definitions.rs` are no longer reachable from any IPC. They were the pre-V3 single-tier classifier; CECE's `orchestrator.rs` + `synthesis.rs` + the 6 catalogers replaced them in V3-§8. The files compile, the tests still pass, but the code is dead.

**Recommendation:** focused cleanup MIG (call it MIG-CLEAN-021v3 or fold into MIG-022 §0 housekeeping). Estimated 1-2 hrs: delete the dead modules, drop the unused `mod` declarations, run `cargo test` to confirm no test depends on them, run `cargo build` to confirm no caller does. Not blocking V3-§11 close-out.

### 3.2 F2 — Unused import in `wiring.rs` (P3)

A residual import that didn't get cleaned when V3-§9.C.2 refactored the per-axis IPCs. One-line fix; can ride any future commit touching that file.

### 3.3 F3 — Magic number `0.85` in `reasoning.rs` (P3)

The Reasoning cataloger uses `0.85` as the assumed self-confidence weight for the LLM's structured output. Currently unnamed. Should be `const REASONING_SELF_CONFIDENCE: f32 = 0.85;` with a brief comment on the V3-§7.b interface contract. P3 polish.

### 3.4 F4 — Dead `write_suggestions` function (P3)

Helper from an earlier draft of `synthesis.rs`; no callers after the V3-§8.r2 architecture rewrite. Drop it.

### 3.5 F5 — False alarm

Agent originally flagged "Pending Jobs v1.8 / v1.9 missing." Both files exist in `docs/` (the agent likely searched a stale snapshot before the V3-§9 / V3-§10 close-out commits landed). Verified by `Glob` post-audit; no action.

### 3.6 Test surface

92 cece tests pass. Cumulative count: 67 (V3-§9 start) → 71 (§9.A) → 78 (§9.B) → 85 (§9.C) → 90 (§9.D) → 92 (§9.C.2). No regressions in `cargo test --package constellation-tauri --lib cece::`.

---

## 4 · Agent 3 — Migration path check

The agent ran through the seven scenarios that a real user might hit when V3-§1 → V3-§10 lands on a populated database.

| Scenario | Verdict | Notes |
|---|---|---|
| First-boot (empty Universe, no cece state) | PASS | Defaults populate cleanly; no IPC needed; reliability JSON file lazy-created on first correction. |
| v2-era upgrade (DB pre-V3, has classifier_suggestions table from MIG-021v2) | PASS | `init_db` migration adds `composite_json` column to `sources_suggestions` (V3-§8.r4 ALTER moved to init_db); old rows get `NULL` composite, frontend treats as Legacy regime + shows the Legacy pill (V3-§8.r5). |
| Reliability JSON file format variations | PASS | `cataloger_reliability.json` is a per-axis nested object; missing axes → uniform prior; missing catalogers → uniform prior; corrupt JSON → file ignored, defaults used (defensive parse). |
| Mid-classification interrupt (close app while orchestrator is running) | PASS | `mpsc::recv_timeout` + tokio task abort means at-most-one row inserted per request; no zombie rows. |
| Settings flag back-compat (user has settings.json from before V3-§10) | PASS *(after this commit)* | Without the AT RISK fix, the `cece` sub-object stays `undefined` on load, and the `?.` + `??` chain at every consumer keeps behavior at defaults — so functionally PASS today, but only because every consumer happens to be defensive. With the fix, it's contractually PASS. |
| i18n locale switching mid-session | PASS | All Source Review chrome re-renders with the new locale's keys; help topic + User Manual lookup re-routes through the locale-aware help loader. |
| Rollback to V3-§7 build | PASS | The CECE schema columns added in V3-§8 (composite_json, reliability columns) are additive; older Constellation builds ignore them. The reliability JSON file lives under `<library>/.constellation/`; rollback ignores it; re-applying V3-§8 picks it back up. |

One soft observation that matches Agent 1's AT RISK: the `cece` deep-merge gap. With the fix landed, migration risk is **LOW**.

---

## 5 · Cumulative MIG-021v3 scoreboard

| Phase | Commits | What |
|---|---|---|
| V3-§1 → V3-§7 (engine spine) | many | 6-cataloger architecture, GBNF, synthesis, IPC layer, reasoning prompt |
| V3-§8 (orchestrator wired + audit fixes) | `daeba00` `3f486b4` `191fb8c` `663e31f` `23e50c0` `72629fc` `84cde6f` `c355be1` `b366d97` `cf03670` `787b64d` | CECE wired into UI; 6 audit fixes; UX polish; queue composition filter |
| V3-§9 (vertical-axis activation) | `4e0981a` `d9dfa60` `ec5527e` `b18a3ee` `bf07ae1` `75807a3` `d5fc070` | Vertical lexicon + structural detectors + reliability wiring + GBNF axis-aware + Gate 2 PASS |
| V3-§10 (user-facing surfaces) | `d44b115` `0054981` `34a96a9` `259c333` `7d6e1a0` `50a67b0` `a4438ac` `4ede8ef` `54276c3` `237871f` | Settings + en/ar i18n + EN docs + 13-locale i18n + 14-locale help + 14-locale User Manual + Gate 3 PASS |
| V3-§11 (final integration audit + close-out) | this commit | AT RISK fix + audit report + orientation v1.95 + MIG-021v3 ships |

Tests: **92 cece tests, all PASS.**

Gates: **Gate 1 PASS · Gate 2 PASS · Gate 3 PASS · Gate 4 (V3-§11) PASS.**

PJs filed during cascade: PJ-040 (UA partial-frontmatter), PJ-041 (cataloger reasoning prose), PJ-042 (confidence enum), PJ-043 (taxonomy labels en+ar only).

---

## 6 · Close-out actions in this commit

1. **AT RISK fix:** `src/lib/libraries/store.ts` — `cece` block added to `DEFAULT_SETTINGS`, deep-merge added to `loadSettings`. Verified all 5 consumer call sites still compile + still use defensive `?.` + `??` (so the fix is purely contract-strengthening, no behavior change for existing users).
2. **This audit report.**
3. **Orientation v1.95** marking "MIG-021v3 ships" + closing the cascade in the version history.
4. **Session log append** capturing V3-§11 verdict + audit summary.
5. **Final commit + push** to `origin/main`.

## 7 · What's next

**MIG-022** — Architect doc responding to:
- The gap analysis (`docs/epistemic-content-gap-analysis.md`) — temporal axis, justification/warrant axis, contestation/agent axis
- PJ-040 — UA partial-frontmatter behavior
- PJ-041 / PJ-042 / PJ-043 — engine-output i18n gaps (cataloger reasoning prose, confidence enum, taxonomy labels)
- F1 from this audit — legacy classifier dead-code cleanup (housekeeping §0)

The CECE engine is the floor. MIG-022 raises the ceiling. The order, scope, and prioritization of MIG-022's pieces are decided in the Architect cycle — not pre-committed here.

---

*Filed by V3-§11 audit cascade. Three parallel agents (invariants / drift / migration-path) + orchestrator round-up. Audit duration ~1.5 hrs.*

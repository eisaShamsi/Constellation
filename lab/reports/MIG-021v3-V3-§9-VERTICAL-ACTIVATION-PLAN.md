# V3-§9 — Vertical-axis Activation — Plan Document (Option C — Full)

**Date:** 2026-05-11  
**Author:** Claude (Plan phase per /migration workflow)  
**Status:** awaiting Boss approval to begin Build phase  
**Predecessor:** V3-§9 Architect doc (`MIG-021v3-V3-§9-VERTICAL-ACTIVATION-ARCHITECT.md`), Boss picked **Option C — Full**

Plan-Approval-Equals-Build-Approval applies once Eisa approves: I cascade through the phases autonomously, stopping only at user-testable verification clauses (Gate 2 Boss-test), genuine architectural surprises, or plan completion.

---

## §0 — Scope summary

Five build phases (A → E), each landable as one commit. Total estimated effort ~4-6 hrs of agent time + Eisa's Gate 2 Boss-test session at the end.

| Phase | What | Files | Commit | Verification |
|---|---|---|---|---|
| **A** | Vertical lexicon expansion 7→~20 entries | `sources_lexicon.json` | `V3-§9.A` | `cargo test cece::catalogers::linguistic::tests` + spot tests |
| **B** | Structural cataloger vertical detectors | `structural.rs` | `V3-§9.B` | `cargo test cece::catalogers::structural::tests` |
| **C** | Per-axis reliability schema migration | `reliability.rs` + `synthesis.rs` | `V3-§9.C` | `cargo test cece::reliability::tests` + migration round-trip |
| **D** | Reasoning Cataloger axis-aware prompt + GBNF | `reasoning_prompt.rs` + `reasoning.rs` | `V3-§9.D` | `cargo test cece::catalogers::reasoning::tests` |
| **E** | NSIS rebuild + orientation v1.90 + Gate 2 Boss-test | docs + build artifacts | `V3-§9.E` | **✅ Boss-test Gate 2** |

Each phase commit includes the relevant unit-test additions, a session-log entry, and (for E) the orientation bump.

---

## §1 — Phase A: Vertical lexicon expansion

**Goal:** bring vertical-axis lexicon coverage to parity with horizontal (~20 entries across all 5 branches).

**Files:**
- `src-tauri/data/sources_lexicon.json` — add 12 new entries to the `vertical` array

**Specific entries to add** (per Architect §9 appendix — exact targets verified against `vertical_taxonomy.rs`):

```jsonc
// epistemic-states (3 new — current 7 stays as-is)
{ "tokens": ["I know that", "we know", "نعلم أن", "معلوم", "من المعلوم"],
  "target": "epistemic-states/knowledge/by-content/propositional",
  "weight": 0.85, "evidence": "Propositional knowledge marker" },
{ "tokens": ["I have met", "I have seen", "I encountered", "تعرفت على", "شاهدت", "قابلت"],
  "target": "epistemic-states/knowledge/by-acquaintance",
  "weight": 0.80, "evidence": "Knowledge-by-acquaintance marker" },
{ "tokens": ["illusion", "mirage", "وهم", "سراب", "خيال"],
  "target": "epistemic-states/illusion",
  "weight": 0.85, "evidence": "Illusion marker" },

// semantic-contents (6 new — branch was empty)
{ "tokens": ["the proposition", "the claim that", "the statement that", "القضية", "أن X يعني"],
  "target": "semantic-contents/proposition",
  "weight": 0.80, "evidence": "Proposition marker" },
{ "tokens": ["the concept of", "is defined as", "we define", "مفهوم", "تعريف"],
  "target": "semantic-contents/concept",
  "weight": 0.80, "evidence": "Concept marker" },
{ "tokens": ["the fact that", "established fact", "حقيقة", "ثابت أن"],
  "target": "semantic-contents/fact",
  "weight": 0.80, "evidence": "Fact marker" },
{ "tokens": ["the theory of", "according to the theory", "نظرية"],
  "target": "semantic-contents/theory",
  "weight": 0.80, "evidence": "Theory marker" },
{ "tokens": ["the idea of", "I had the idea", "فكرة", "خطر ببالي"],
  "target": "semantic-contents/idea/constructed",
  "weight": 0.75, "evidence": "Constructed-idea marker" },
{ "tokens": ["the information", "the data", "معلومات", "بيانات"],
  "target": "semantic-contents/information",
  "weight": 0.75, "evidence": "Information marker" },

// sensory-inputs (1 new — branch was empty)
{ "tokens": ["the signal", "the input", "إشارة", "مدخل حسي"],
  "target": "sensory-inputs/signal",
  "weight": 0.70, "evidence": "Sensory signal marker" },

// symbolic-entities (1 new — branch was empty)
{ "tokens": ["the symbol", "the sign", "إشارة رمزية", "رمز"],
  "target": "symbolic-entities/sign",
  "weight": 0.70, "evidence": "Symbol/sign marker" },

// higher-order-constructs (2 new — branch was empty)
{ "tokens": ["worldview", "paradigm", "framework", "رؤية كونية", "إطار"],
  "target": "higher-order-constructs/worldview",
  "weight": 0.80, "evidence": "Worldview/paradigm marker" },
{ "tokens": ["doctrine", "school of thought", "مذهب", "عقيدة", "مدرسة فكرية"],
  "target": "higher-order-constructs/doctrine",
  "weight": 0.80, "evidence": "Doctrine/school-of-thought marker" }
```

**Validation step before commit:** verify each `target` string exists in `vertical_taxonomy.rs::VERTICAL_NODES` — if any ID is wrong (e.g. typo, missing intermediate node), the lexicon entry will be silently dropped at cataloger run time, which is graceful but defeats the purpose. Grep each target ID through the taxonomy; fix any mismatches before commit.

**Self-verification:**
- `cargo test cece::catalogers::linguistic::tests` — all existing tests must still pass
- `cargo test cece::` — full cece suite must still pass (67 tests baseline from r8)
- Add 1-2 new spot tests confirming a couple of the new entries fire on synthetic input (e.g. test note containing "the proposition that" should fire `semantic-contents/proposition`)

**Pass criteria:** all tests green; lexicon JSON parses; vertical entry count goes from 7 to ~19.

**Commit:** `MIG-021v3 V3-§9.A — Vertical lexicon expansion (7 → ~19 entries; covers all 5 branches)`

---

## §2 — Phase B: Structural cataloger vertical-axis detectors

**Goal:** add 5-6 new vertical regex rules to `structural.rs::vertical_rules()` covering structural patterns the lexicon can't catch.

**Files:**
- `src-tauri/src/cece/catalogers/structural.rs` — extend `vertical_rules()` + add 5-6 regression tests

**New rules to add:**

```rust
// Definition markers — concept signal
(
    r"(?i)\b(is defined as|defined as|we define|let .{1,30} be|means by definition)\b",
    "semantic-contents/concept",
    0.80,
    "Definition marker (English)",
),
(
    r"(تُعرَّف|نُعرِّف|التعريف|يُعرَّف|نقصد بـ)",
    "semantic-contents/concept",
    0.80,
    "Definition marker (Arabic)",
),
// Worldview / paradigm markers — higher-order construct
(
    r"(?i)\b(worldview|paradigm|framework|conceptual scheme)\b",
    "higher-order-constructs/worldview",
    0.75,
    "Worldview/paradigm marker (English)",
),
(
    r"(رؤية كونية|إطار|نموذج معرفي|تصور)",
    "higher-order-constructs/worldview",
    0.75,
    "Worldview/paradigm marker (Arabic)",
),
// Image/figure references — visual sensory signal
(
    r"(?i)\b(figure \d+|fig\.\s*\d+|see figure|see fig\.|في الشكل|انظر الشكل)\b",
    "sensory-inputs/signal/physical/electromagnetic",
    0.70,
    "Visual reference marker (figure/diagram)",
),
// Code-block density — symbolic-entity signal
// Triggers when the note has 3+ markdown code blocks (```...```).
// Implemented as a synthetic post-pass after regex matching since
// counting code blocks requires either a line-pass or a multiline regex.
```

The code-block-density rule needs a slightly different shape — count `^```` occurrences in the body, fire if count >= 6 (3 opening + 3 closing fences). Implement as a separate inline function called from `classify()` rather than as a regex rule; document why.

**Regression tests to add (in `structural.rs::tests`):**

- `english_definition_marker_fires_concept`
- `arabic_definition_marker_fires_concept`
- `worldview_marker_fires_higher_order`
- `arabic_worldview_marker_fires_higher_order`
- `figure_reference_fires_visual_signal`
- `code_block_density_fires_symbolic_entity`

**Self-verification:**
- `cargo test cece::catalogers::structural::tests` — new tests pass + existing 11 tests still pass
- `cargo test cece::` — full suite green
- No horizontal-axis regression: existing horizontal rules in `structural.rs` are untouched

**Pass criteria:** all tests green; structural cataloger now has 12 vertical rules (was 6); 6 new regression tests added.

**Commit:** `MIG-021v3 V3-§9.B — Structural vertical detectors (definitions + worldview + visual + code-density)`

---

## §3 — Phase C: Per-axis reliability schema migration

**Goal:** track per-cataloger reliability separately for horizontal and vertical axes. Currently a single Bayesian credible interval per cataloger per Library; migration splits it into two.

**Why this matters:** Linguistic might be 95% accurate on horizontal (lexicon coverage strong) but 50% on vertical (lexicon coverage thin). A single shared reliability profile can't represent that — synthesis weighting under-trusts Linguistic on horizontal or over-trusts on vertical.

**Files:**
- `src-tauri/src/cece/reliability.rs` — schema bump + back-compat reader
- `src-tauri/src/cece/synthesis.rs` — pass axis to reliability lookup; update `vote_on_axis` to use per-axis profile

**Schema change:**

v1 (current):
```jsonc
{
  "linguistic": { "mean": 0.7, "count": 100, "successes": 70 },
  "structural": { "mean": 0.65, "count": 80, "successes": 52 },
  ...
}
```

v2 (new):
```jsonc
{
  "schema_version": 2,
  "linguistic": {
    "horizontal": { "mean": 0.7, "count": 100, "successes": 70 },
    "vertical":   { "mean": 0.7, "count": 100, "successes": 70 }
  },
  "structural": {
    "horizontal": { "mean": 0.65, "count": 80, "successes": 52 },
    "vertical":   { "mean": 0.65, "count": 80, "successes": 52 }
  },
  ...
}
```

**Back-fill rule:** when reader detects v1 shape (no `schema_version` field, profile values are flat objects with `mean` directly), copy the single profile into BOTH `horizontal` and `vertical` slots — preserves the Bayesian prior. From then on, accuracy updates land in the axis-specific slot.

**Synthesis change:** `vote_on_axis(voiced, axis, reliability)` already takes `axis` as parameter (verified — `synthesis.rs:181`). The reliability lookup inside currently does `reliability.weight_for(cataloger)`; change to `reliability.weight_for(cataloger, axis)`. The new method returns the axis-specific profile's mean.

**Tests to add:**
- `v1_profile_back_fills_to_both_axes` — read a v1 JSON blob, assert both axes get the same prior
- `v2_profile_round_trips` — write v2, read v2, assert structural equality
- `axis_specific_update_doesnt_cross_pollute` — bump linguistic.horizontal, assert linguistic.vertical unchanged
- `synthesis_uses_per_axis_profile` — construct a profile where linguistic.horizontal=0.95 and linguistic.vertical=0.50, assert weighted vote on horizontal favors linguistic and on vertical doesn't
- `default_profile_is_per_axis` — `ReliabilityProfile::default()` has both axes for every cataloger

**Risk:** schema migration affects every Library that has run a CECE classification. The back-fill must be idempotent (re-reading a v2 profile must not re-back-fill).

**Self-verification:**
- `cargo test cece::reliability::tests` — new tests pass + existing tests still pass
- `cargo test cece::synthesis::tests` — synthesis still passes (the existing tests use `ReliabilityProfile::default()`; verify the default still produces a usable per-axis profile)
- `cargo test cece::` — full suite green

**Pass criteria:** all tests green; v1 → v2 migration is automatic on first read; synthesis weighting routes through axis-specific profiles.

**Commit:** `MIG-021v3 V3-§9.C — Per-axis reliability tracking (schema v2 + back-compat reader)`

---

## §4 — Phase D: Reasoning Cataloger axis-aware prompt + GBNF

**Goal:** lock in the right interface for V3-§7.b llama.cpp wiring. Currently the Reasoning Cataloger has a single GBNF grammar that includes both axes' valid IDs in one alternation; the prompt doesn't explicitly distinguish source-of-knowledge from kind-of-knowledge.

**Note:** Reasoning Cataloger currently abstains at runtime (llama.cpp not wired per V3-§7.b deferred). Phase D's changes are no-op at runtime but lock in the right interface for when V3-§7.b ships. **Do not add LLM-quality verification stages here** — they belong in V3-§7.b.

**Files:**
- `src-tauri/src/cece/catalogers/reasoning_prompt.rs` — update SYSTEM_PROMPT with explicit per-axis reasoning instructions; reorganize few-shot exemplars so each demonstrates per-axis reasoning explicitly
- `src-tauri/src/cece/catalogers/reasoning_prompt.rs` — split the GBNF generator into two functions: `gbnf_for_horizontal()` and `gbnf_for_vertical()`. Each grammar covers only its axis's valid IDs. Keep the single combined grammar function as `gbnf_combined()` for backward compatibility (callers can opt into single-pass or two-pass)
- `src-tauri/src/cece/catalogers/reasoning.rs` — note the new shape but no behavioral change yet (still abstains)

**Prompt changes (SYSTEM_PROMPT):**

Add a new section explicitly distinguishing the two axes:

```
You classify a note along TWO orthogonal axes:

AXIS 1 — SOURCE (horizontal): Where does this knowledge COME FROM?
  Examples: testimony (someone told me), perception (I saw it), inference
  (I derived it), comparison (analogy to known case), revelation (sacred
  text), inspiration (sudden insight).

AXIS 2 — CONTENT TYPE (vertical): What KIND of knowledge is this?
  Examples: sensory input (raw signal), symbolic entity (sign/code),
  semantic content (concept/proposition/fact), epistemic state (doubt/
  certainty/belief), higher-order construct (worldview/doctrine).

A single note has values on BOTH axes. They are independent — a note
about "I doubt the moon landing" is testimony (someone reported it)
+ epistemic-states/doubt (the user's stance toward it).

Output BOTH axes in your JSON response.
```

Reorganize 4-6 of the existing few-shot exemplars to demonstrate this explicitly. Add 2-3 new exemplars where the two axes are clearly different (testimony + doubt; perception + concept; etc.).

**GBNF changes:**

Two new functions alongside the existing single-grammar function:

```rust
/// V3-§9.D — axis-specific GBNF for two-pass classification.
/// Grammar for the horizontal axis only — closed set of source IDs.
pub fn gbnf_for_horizontal() -> &'static str { ... }

/// Grammar for the vertical axis only — closed set of content_type IDs.
pub fn gbnf_for_vertical() -> &'static str { ... }

/// Combined grammar (current behavior) — both axes in one response.
/// Kept for back-compat; new V3-§7.b code can pick single-pass or two.
pub fn gbnf_combined() -> &'static str { ... }
```

The grammars use the same `OnceLock` cache pattern as the current `gbnf()` (per r4.6).

**Tests to add:**
- `horizontal_grammar_only_contains_horizontal_ids` — parse the horizontal grammar, assert no vertical IDs leak in
- `vertical_grammar_only_contains_vertical_ids` — same for vertical
- `combined_grammar_unchanged` — round-trip identity test on the existing grammar (no regression)
- `system_prompt_mentions_both_axes` — string search for "AXIS 1" and "AXIS 2" in the prompt (so future edits don't accidentally drop the per-axis guidance)
- `axis_aware_exemplars_balance_horizontal_and_vertical` — count how many few-shot examples have non-trivial vertical reasoning; assert ≥3

**Self-verification:**
- `cargo test cece::catalogers::reasoning::tests` — new tests pass + existing tests still pass
- `cargo test cece::` — full suite green
- Reasoning Cataloger still abstains at runtime (llama.cpp not wired) — verify the abstain behavior is unchanged

**Pass criteria:** all tests green; system prompt explicitly distinguishes the two axes; two new axis-specific GBNF functions exist alongside the combined one; no runtime behavioral change today.

**Commit:** `MIG-021v3 V3-§9.D — Reasoning Cataloger axis-aware prompt + per-axis GBNF (interface lock-in for V3-§7.b)`

---

## §5 — Phase E: NSIS rebuild + orientation v1.90 + Gate 2 Boss-test

**Goal:** ship a build Eisa can install + run the Boss-test verification clause for V3-§9.

**Files:**
- `docs/Constellation Orientation & Onboarding v1.90.md` (NEW — bump from v1.89, document V3-§9 close-out)
- `lab/reports/SESSION-LOG-2026-05-11.md` — entry summarizing all 5 phases
- NSIS build artifact: `Constellation_0.3.4_x64-setup.exe`

### ✅ Boss-test Gate 2 — VERTICAL axis

This is the user-testable verification clause. Per the Testing Instructions Rule, every stage articulates the feature first, then walks through interaction by interaction. Stages mirror Gate 1's structure but exercise vertical-axis content.

**Stage 0 — Verify the new build is installed**

1. Close Constellation if running.
2. Run installer: `E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.3.4_x64-setup.exe` (mtime captured at commit time of V3-§9.E).
3. Launch + open the same Library you've been Boss-testing.

**Stage 1 — Verify lexicon expansion fires on vertical content**

*Feature:* Phase A added 12 lexicon entries covering all 5 branches of the vertical taxonomy. Notes containing words/phrases like "I know that", "the proposition", "the concept of", "the worldview", "مفهوم", etc. should now fire Linguistic on those vertical-axis IDs.

1. Create a new note titled "Test V3-§9 Stage 1" with body:
   > The concept of constructive proof is defined as a proof that exhibits the object whose existence is asserted. The fact that classical mathematics accepts non-constructive proofs reflects a deep philosophical doctrine.

2. Right-click the note → "Suggest sources & content type."
3. Open the Source Review panel.

**Expected:** the new card's Linguistic trail should mention `semantic-contents/concept` (from "the concept of" or "is defined as") AND/OR `semantic-contents/fact` (from "the fact that") AND/OR `higher-order-constructs/doctrine` (from "doctrine"). Multiple vertical hits is fine; the synthesis layer picks the highest-weight primary.

**Failure mode:** if the trail's vertical primary is something unrelated like `epistemic-states/doubt` or empty, Phase A's lexicon entries didn't fire. Tell me which entry should have triggered.

**Stage 2 — Verify structural detectors fire on definition / worldview / image / code content**

*Feature:* Phase B added 5-6 structural regex rules: definition markers, worldview markers, figure references, code-block density.

1. Create a new note "Test V3-§9 Stage 2 — definitions" with body:
   > A monad is defined as a monoid in the category of endofunctors. We define functor composition as ...

2. Suggest classification.

**Expected:** Structural cataloger trail mentions `semantic-contents/concept` (definition marker fired). Synthesis primary on vertical should be `semantic-contents/concept`.

3. Create a new note "Test V3-§9 Stage 2 — worldview" with body:
   > The materialist worldview reduces all phenomena to physical interactions. Within this paradigm, consciousness is an emergent property of neural activity.

**Expected:** vertical primary should be `higher-order-constructs/worldview`.

4. Create a note "Test V3-§9 Stage 2 — code" with body containing 3+ ` ``` ` code blocks (any language).

**Expected:** vertical primary should be `symbolic-entities/sign` or `symbolic-entities/code` (depending on which target the rule used).

**Failure mode:** any of the three notes whose vertical primary is wrong → tell me which note + what was rendered + what was expected.

**Stage 3 — Verify per-axis reliability profile splits correctly**

*Feature:* Phase C migrated reliability tracking from single-profile-per-cataloger to per-cataloger-per-axis.

1. Open `<universe>/.constellation/cece-reliability/<library_root>.json` in any text editor.

**Expected:** the file structure now has `"schema_version": 2` at the top, and each cataloger entry has BOTH `"horizontal"` and `"vertical"` sub-objects with their own `mean`, `count`, `successes`. If your profile was previously v1, both sub-objects should have the SAME values (back-fill copied the single prior into both axes).

2. Reject a few cards in the Source Review queue (any cards). Each rejection updates the relevant cataloger's reliability profile.

3. Re-open the JSON.

**Expected:** the cataloger profiles for the catalogers that voiced on rejected cards should have updated counts. If you rejected a card where the synthesis primary was on the horizontal axis only, only the `horizontal` sub-object should have a bumped count for the catalogers that voiced on horizontal — `vertical` should be unchanged.

**Failure mode:** if both axes update on every reject, the per-axis routing isn't happening. If neither axis updates, the reliability writer regressed.

**Stage 4 — Verify Reasoning Cataloger interface (no runtime change today)**

*Feature:* Phase D updated the Reasoning Cataloger's prompt + grammar to be axis-aware. The Reasoning Cataloger still abstains because llama.cpp isn't wired (V3-§7.b deferred), so this stage is a sanity check, not a quality test.

1. Look at any card's per-cataloger badge cluster.

**Expected:** the green Reasoning dot should still be dashed-outline silent (Reasoning Cataloger abstains; this is unchanged).

**Failure mode:** if Reasoning starts producing voiced trails, llama.cpp got accidentally wired — that's premature; report it.

**Stage 5 — End-to-end Boss-test on diverse vertical content**

*Feature:* the cumulative effect of A+B+C is that vertical-axis classification on a Library with diverse content (not just doubt/certainty Arabic notes) should produce richer, more accurate vertical primaries.

1. Right-click 5-10 fresh notes spanning different content types: a math proof note, a philosophical worldview note, a sensory observation note ("I saw a red bird this morning"), a code-heavy note, a fact-based note ("Baghdad was founded in 762 CE").

2. For each card in the queue, hover the dot cluster + read the trail.

**Expected:** vertical primaries diverge across the cards based on content type. The math note → `semantic-contents/proposition` or `concept`. The worldview note → `higher-order-constructs/worldview`. The sensory note → `epistemic-states/knowledge/by-acquaintance` or `sensory-inputs/signal`. The code note → `symbolic-entities/sign`. The fact note → `semantic-contents/fact`.

**Failure mode:** if all vertical primaries land on the same value (e.g. all `epistemic-states/doubt`), vertical-axis lexicon coverage is still asymmetric. Tell me which notes landed on which primaries.

**Stage 6 — Confirm no horizontal regression**

1. Re-classify `الخط العربي` (which already has `sources: testimony/authoritative` from Stage 2.1 of the Gate 1 cycle). UA short-circuits horizontal.

**Expected:** the Source Review card renders identically to how it did in the r8 verification: dot cluster shows UA filled (blue, voiced + agrees), other catalogers' status unchanged from r8 build, horizontal Unanimous, vertical synthesis whatever the catalogers vote (now with the new vertical lexicon entries possibly firing). Approve All math should still correctly count this card as agreed (cardNeedsUserCall returns false).

**Failure mode:** any horizontal-axis behavior change from the r8 build is a regression — tell me what changed.

### Gate 2 PASS criteria

- Stage 1 + Stage 2 confirm new lexicon and structural detectors fire on the right content
- Stage 3 confirms per-axis reliability schema works
- Stage 4 confirms Reasoning Cataloger interface unchanged at runtime
- Stage 5 confirms end-to-end vertical diversity on real content
- Stage 6 confirms no horizontal regression

If all 6 stages pass, **Gate 2 closes** and we move to V3-§10 (Settings + i18n + Help docs + User Manual).

**Commit:** `MIG-021v3 V3-§9.E — NSIS build + orientation v1.90 + Gate 2 Boss-test ready`

---

## §6 — What V3-§9 does NOT do (re-confirming non-scope from Architect §7)

- ❌ Backlog (PJ-NNN): vertical-axis Settings UI for cataloger weights — **V3-§10 territory**
- ❌ Backlog (PJ-NNN): help docs / User Manual chapter for vertical axis — **V3-§10 territory**
- ❌ Out of scope: vertical taxonomy data extension — Eisa-canonical, separate decision
- ❌ Out of scope: Reasoning Cataloger LLM quality validation — **V3-§7.b territory**
- ❌ Out of scope: separate "Suggest content type" context menu — current combined action is correct UX
- ❌ Out of scope: standalone vertical-only review mode — r8's filter already provides this

---

## §7 — Risk register (mitigated)

Inherited from Architect §5; tracking mitigations applied per phase.

| Risk | Phase | Mitigation in this Plan |
|---|---|---|
| Lexicon entry references invalid taxonomy ID | A | Pre-commit grep validation (§1) |
| Structural regex over-matches | B | Same defensive weight tier (0.65–0.85) as existing rules; r5.7 blockquote pattern for follow-up if needed |
| Schema migration affects existing reliability JSON files | C | Back-compat reader v1→v2; idempotent on v2; round-trip test (§3) |
| Reasoning prompt change carries risk for V3-§7.b | D | No runtime behavior today; Reasoning still abstains; tests assert prompt structure not LLM quality |
| Lexicon size growth slows boot | A | OnceLock lazy init; +12 entries is microseconds at first call |
| New structural detectors regress horizontal regression tests | B | Vertical rules isolated in `vertical_rules()`; horizontal in `horizontal_rules()`; full cece test suite after every commit |
| Cross-axis bleed in reliability updates | C | Test `axis_specific_update_doesnt_cross_pollute` |

---

## §8 — Approval request

**Boss, please approve this Plan.**

Once approved, per Plan-Approval-Equals-Build-Approval I'll cascade through phases A → E autonomously, stopping only at:
- The Stage 0–6 user-testable verification clauses in Phase E (Gate 2 Boss-test)
- Genuine architectural surprises (will surface and pause)
- Plan completion (summarize close-out + propose next step toward V3-§10)

The Standing Order session-log discipline applies between steps; I'll log each `V3-§9.X` commit as it lands. Orientation bump to v1.90 lands inside the Phase E commit per SO #6.

Estimated wall-clock time: 4-6 hrs of agent build/test/commit + Eisa's Gate 2 Boss-test session at the end.

---

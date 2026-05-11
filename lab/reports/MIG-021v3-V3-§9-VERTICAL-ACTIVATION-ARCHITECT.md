# V3-§9 — Vertical-axis Activation — Architect Document

**Date:** 2026-05-10  
**Author:** Claude (audit + design)  
**Status:** Architect Phase — awaiting Boss approval to proceed to Plan phase  
**Predecessor:** V3-§8.r1→r8 (horizontal axis production-ready, Gate 1 PASS)

---

## §1 — Where we actually are (vertical-axis state audit)

The original V3-§9 plan from `MIG-021v3-EPISTEMIC-CONTENT-ENGINE-PLAN.md` §10 framed this phase as "wire the vertical axis through every cataloger + the synthesis layer." That framing was written before V3-§8 cascade — and a lot of "vertical activation" work happened implicitly while we were fixing horizontal-axis bugs.

**What's already shipped (verified during V3-§8.r1→r8 cascade):**

| Layer | Vertical-axis status | Evidence |
|---|---|---|
| **Taxonomy data** | 225 nodes in `vertical_taxonomy.rs`, full 5-branch tree (Sensory · Symbolic · Semantic · States · Higher-order) | Source: `src-tauri/src/sources/vertical_taxonomy.rs` |
| **IPCs** | `content_type_set_manual`, `content_type_get_for_note`, `extract_content_type`, `rewrite_frontmatter_content_type`, `write_content_type_to_db` all exist | `src-tauri/src/sources/mod.rs` |
| **Catalogers** | All 6 catalogers produce vertical output in their trails (`vertical → epistemic-states/doubt` etc.) | Verified in `synthesis::tests::ua_short_circuit_serializes_both_regimes_as_unanimous` JSON dump |
| **Synthesis** | Per-axis regime computation works independently | r5-r8 Boss-test screenshots show H=Unanimous + V=StrongMajority on the same card |
| **UI render** | CONTENT TYPE section, disambig chips per axis, dot cluster reflects per-axis dissent, filter chips per axis | r5-r8 Boss-test screenshots |
| **Disambig** | `cece_resolve_disambiguation` accepts `axis: "vertical"` + auto-writes settled horizontal on vertical pick | r7 + r8 verification (Baghdad note) |
| **Filter** | "Content type needs your call" bucket isolates vertical-Split cards | r8 verification screenshot |
| **PropertyEditor** | TaxonomyTreePicker renders vertical in edit mode | `SourceReviewPanel.svelte` Edit mode + `PropertyEditor.svelte` |
| **Context menu** | "Suggest sources & content type" right-click classifies both axes in one shot | `+layout.svelte` |

**What's NOT yet shipped (the actual V3-§9 scope):**

1. **Lexicon thinness** — `sources_lexicon.json::vertical` has **7 entries** vs **17 horizontal entries**. All 7 are concentrated in the `epistemic-states` branch (doubt, certainty, belief, opinion, three yaqīn varieties). The other four branches (Sensory · Symbolic · Semantic · Higher-order) have **zero** lexicon coverage.
2. **Structural-cataloger vertical detectors** — `structural.rs::vertical_rules()` has 6 inline rules (doubt/certainty markers EN+AR, theorem markers, numerical units → fact). Coverage parallel to horizontal would require: definition markers, classification markers, sensory-input markers (image/audio refs), symbolic-entity markers (formal-language code blocks).
3. **Per-axis reliability tracking** — `reliability.rs::ReliabilityProfile` tracks per-cataloger accuracy at the Library level but **does not split per-axis**. A cataloger might be accurate on horizontal but unreliable on vertical; the synthesis weighting can't distinguish.
4. **Reasoning Cataloger axis-awareness** — the GBNF grammar includes both axes' valid IDs in one alternation. The prompt doesn't explicitly tell the LLM to reason about source-of-knowledge separately from kind-of-knowledge. Cross-axis confusion is theoretically possible.
5. **Gate 2 Boss-test scenarios** — verify vertical-axis classification on diverse vertical content (not just doubt/certainty markers in Arabic notes — also propositional/perceptual/idea/theory in mixed-language notes).

**Cross-cutting observations:**
- Most of the original V3-§9 plan's listed work ("verify dual-axis composite assignment correctly handles independent regimes per axis") is already implicitly verified through the V3-§8.r5→r8 cascade and the recent Boss-test cycle.
- The remaining work is **enrichment + audit**, not new architecture.

---

## §2 — Invariants that must not break

(Inherited from V3-§8 + extended for vertical)

1. **User-Authority absolute precedence** — when `content_type:` is set in frontmatter, vertical synthesis short-circuits regardless of other catalogers' votes (already enforced in `user_authority_short_circuit`).
2. **Per-axis independence** — one axis Split + other axis Unanimous is a valid + handled state.
3. **Backward compatibility** — every change MUST preserve readability of pre-V3-§9 `composite_json` blobs. New lexicon entries can't change the meaning of existing entries.
4. **Sources of truth respected** — vertical taxonomy data lives in `vertical_taxonomy.rs` (Rust mirror of `docs/epistemic-content-taxonomy-chart.html`). New lexicon entries reference taxonomy IDs; if a referenced ID doesn't exist, the entry is dropped at validation (defense-in-depth, like horizontal).
5. **No regression on horizontal** — vertical-axis enrichment must not change horizontal-axis behavior. Verified by re-running existing horizontal regression tests after every commit.
6. **Boot performance unchanged** — lexicon size growth (7 → ~20 vertical entries) must not regress boot or classification latency. Catalogers already iterate the full lexicon; adding 13 entries is a microsecond.
7. **i18n complete on user-facing strings** — any new Settings UI / help text for vertical-axis features goes through `$t()` from day one (en + ar at minimum).

---

## §3 — Three scope options for Boss decision

Each option is fully landable; the deeper options include the shallower ones.

### Option A — Minimal (lexicon only, ~30 min, 1 commit)

**Just expand the vertical lexicon from 7 → ~20 entries** covering all 5 branches.

- New lexicon entries:
  - `semantic-contents/proposition`: "claim that", "the proposition", "the statement that", "أن X يعني", "القضية"
  - `semantic-contents/concept`: "the concept of", "is defined as", "we define", "مفهوم", "تعريف"
  - `semantic-contents/fact`: "the fact that", "established fact", "حقيقة", "ثابت"
  - `semantic-contents/theory`: "the theory of", "according to the theory", "نظرية"
  - `semantic-contents/idea/constructed`: "the idea of", "I had the idea", "فكرة", "خطر ببالي"
  - `semantic-contents/information`: "the information", "the data", "معلومات", "بيانات"
  - `higher-order-constructs/worldview`: "worldview", "paradigm", "framework", "رؤية كونية", "إطار"
  - `higher-order-constructs/doctrine`: "doctrine", "school of thought", "مذهب", "عقيدة"
  - `epistemic-states/knowledge/by-content/propositional`: "I know that", "we know", "نعلم أن", "معلوم"
  - `epistemic-states/knowledge/by-acquaintance`: "I have met", "I have seen", "تعرفت على", "شاهدت"
  - `sensory-inputs/signal`: "the signal", "the input", "إشارة", "مدخل حسي"
  - `symbolic-entities/sign`: "the sign", "the symbol", "إشارة رمزية", "رمز"
  - `epistemic-states/illusion`: "illusion", "mirage", "وهم", "سراب"

**What this yields:** Linguistic cataloger fires on a much broader range of vertical-axis content. Surface keyword and CAE root paths both benefit. Covers all 5 branches at top level + a few key sub-branches.

**What this leaves on the table:** No per-axis reliability tracking; no Reasoning prompt refinement; structural detectors unchanged.

### Option B — Standard (Option A + structural detectors + Gate 2 Boss-test, ~2-3 hrs, 3 commits)

Includes Option A plus:

- **Vertical structural detectors** in `structural.rs::vertical_rules()`:
  - Definition markers: `(?i)\b(is defined as|defined as|we define|let X be)\b` → `semantic-contents/concept`
  - Arabic equivalents: `(تُعرَّف|نُعرِّف|التعريف|يُعرَّف)` → `semantic-contents/concept`
  - Worldview/paradigm markers: `(?i)\b(worldview|paradigm|framework)\b` + `(رؤية كونية|إطار|نموذج)` → `higher-order-constructs/worldview`
  - Image/figure references: `(?i)\b(figure \d+|see fig\.|في الشكل)\b` → `sensory-inputs/visual` (vertical signal of visual sensory content)
  - Code-block density: when the note has ≥3 code blocks, → `symbolic-entities/code` weight 0.65

- **Gate 2 Boss-test plan** spelled out per the Testing Instructions Rule (define feature → walk through). Stages mirror Gate 1's structure but on vertical-axis content.

**What this yields:** Vertical axis behaves equivalently to horizontal; Gate 2 verifies it.

**What this leaves on the table:** Per-axis reliability still uses one shared profile per cataloger. Reasoning Cataloger prompt unchanged.

### Option C — Full (Option B + per-axis reliability + Reasoning axis-awareness, ~4-6 hrs, 5 commits)

Includes Option B plus:

- **Per-axis reliability schema migration** — `ReliabilityProfile` extended to track per-cataloger-per-axis credible intervals separately. JSON schema bump (with backward-compat reader for v1 profiles). Synthesis weighting picks the right per-axis profile.
- **Reasoning Cataloger axis-aware prompt + GBNF** — the system prompt explicitly distinguishes "source of knowledge" (horizontal) from "kind of knowledge" (vertical). The few-shot exemplars are reorganized to demonstrate per-axis reasoning. Optionally split the GBNF into two axis-specific grammars + run two LLM passes (one per axis) — only if the single-grammar approach shows cross-axis confusion in Gate 2.

**What this yields:** A more rigorous vertical axis with proper per-axis reliability + LLM axis-awareness. Maximum quality.

**Cost:** Schema migration is a real change (per-Library JSON files need a back-fill on first read; needs a migration test). Reasoning Cataloger changes carry the risk of regressing horizontal accuracy if not carefully tested.

---

## §4 — Recommended option

**Option B.** Rationale:

- The lexicon expansion (Option A) is the highest-impact / lowest-risk change. It directly addresses the audit P2 finding "lexicon thinness for cold-start" on the vertical side.
- Structural detectors expansion (Option B's increment) is a small, safe addition — `structural.rs` is well-tested and the new rules follow the existing pattern.
- Gate 2 Boss-test is required regardless — it's the verification clause for V3-§9.
- Per-axis reliability (Option C's increment) is a real improvement but it's a non-trivial schema migration that doesn't have to land in the same cascade as the lexicon work. Better as a focused follow-up MIG once we have real per-axis accuracy data to motivate the design.
- Reasoning Cataloger axis-awareness (Option C's other increment) is also worth doing but should wait until llama.cpp is actually wired (V3-§7.b) — otherwise we're tuning a prompt the LLM isn't running.

Option C items get filed as PJ-NNN backlog with clear scope.

---

## §5 — Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| New lexicon entry references invalid taxonomy ID | Medium | Low (entry silently dropped) | Existing `is_valid_content_type_id` check at cataloger run time |
| New structural regex over-matches and pollutes vertical primary | Medium | Medium (false positives in Boss-test) | Same defensive weight tier (0.65–0.85) as existing rules; r5.7 blockquote-weight pattern showed how to detect over-matching | 
| Vertical lexicon Arabic terms collide with horizontal lexicon (same Arabic word means both source-of-knowledge AND kind-of-knowledge) | Low | Low (catalogers vote on both axes independently; no cross-axis bleed) | None needed — by-design |
| Gate 2 Boss-test reveals cross-axis confusion in Reasoning Cataloger output | Low (LLM not currently wired) | High if LLM was live | Defer Reasoning Cataloger axis-awareness to V3-§7.b co-landing |
| Lexicon size growth slows boot | Low (single-digit ms) | Negligible | Already includes lazy-init via `OnceLock`; adding 13 entries is a microsecond at first call |
| New structural detectors regress existing horizontal regression tests | Low (vertical rules are isolated in `vertical_rules()`) | Medium | Run full `cargo test cece::` after every commit |

---

## §6 — Files in scope

For Option B (recommended):

- **EDIT** `src-tauri/data/sources_lexicon.json` — vertical entries 7 → ~20
- **EDIT** `src-tauri/src/cece/catalogers/structural.rs` — `vertical_rules()` expansion + 4-6 new regression tests
- **EDIT** `src-tauri/src/cece/catalogers/linguistic.rs` — no code change, but new lexicon entries flow through automatically; add 2-3 regression tests confirming new vertical primaries fire
- **NEW** `lab/reports/MIG-021v3-V3-§9-VERTICAL-ACTIVATION-PLAN.md` — Phase 2 plan document
- **EDIT** `docs/Constellation Orientation & Onboarding v1.89.md` → bump to v1.90 documenting V3-§9 close-out
- **EDIT** `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — per-step log entries

Non-files: 1 NSIS rebuild, 1 commit per phase, 1 Gate 2 Boss-test cycle.

---

## §7 — What V3-§9 does NOT do

Explicit non-scope (worth listing so it doesn't drift in):

- ❌ Per-axis reliability tracking (Option C item; backlog as PJ-NNN)
- ❌ Reasoning Cataloger axis-aware prompt (Option C item; defer to V3-§7.b)
- ❌ Settings UI for vertical-axis cataloger weights (V3-§10 territory)
- ❌ Help docs for vertical-axis features (V3-§10 territory)
- ❌ Vertical-axis context menu separate from horizontal (current "Suggest sources & content type" already covers both axes; splitting into two menu items is a UX regression — adds clicks for the common case)
- ❌ Standalone "vertical-only review" mode (r8's filter already provides this via "Content type needs your call" bucket)
- ❌ Any change to vertical taxonomy data (225 nodes is comprehensive; if Eisa wants to extend it, that's a separate decision tied to the Universal Epistemic Content Taxonomy doc)

---

## §8 — Decision request

**Boss, please pick:**

- **(A)** Option A only — lexicon expansion, ship in 1 commit
- **(B)** Option B (recommended) — lexicon + structural + Gate 2
- **(C)** Option C — full per-axis reliability + Reasoning prompt
- **(D)** Skip V3-§9; current vertical-axis behavior is sufficient; jump to V3-§10 (Settings + i18n + Help docs + User Manual)

Once you pick, I'll write the Plan document (Phase 2 of the migration workflow) with phase-by-phase commits and verification clauses, and you approve the plan before any code lands.

---

## §9 — Appendix: vertical-axis lexicon coverage gap (current vs proposed)

Current (7 entries, all in `epistemic-states`):
- doubt, certainty, belief/occurrent, opinion/probable, three yaqīn variants

Proposed for Option A/B (~20 entries, covering all 5 branches):

| Branch | Current count | Proposed count | New entries |
|---|---|---|---|
| epistemic-states | 7 | 9 | + knowledge/propositional, knowledge/by-acquaintance, illusion |
| semantic-contents | 0 | 6 | + concept, proposition, fact, theory, idea/constructed, information |
| sensory-inputs | 0 | 1 | + signal (top-level) |
| symbolic-entities | 0 | 1 | + sign (top-level) |
| higher-order-constructs | 0 | 2 | + worldview, doctrine |
| **Total** | **7** | **~19** | +12 net |

This brings vertical lexicon coverage to roughly parity with horizontal (17 entries) — both axes have one entry per major sub-branch instead of one branch dominating.

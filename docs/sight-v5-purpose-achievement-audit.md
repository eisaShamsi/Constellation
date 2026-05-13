# Constellation Sight v5 — Purpose-Achievement Audit

**Version 0.1 | 2026-05-13.**
**Author**: Eisa Alshamsi · drafted with Claude.
**Companion to**: `Constellation-Sight-Concept-Paper-v3.1.md` (the design contract) + `sight-v5-mode-concepts.md` v0.1 (the mode-concepts deep-dive).
**Trigger**: Eisa, 2026-05-13: *"making sure Sight will achieve its core purpose, given its mode concept."*

---

## §0 · The audit criterion

Sight v5 is a **four-layer analytical instrument** (Concept Paper v3.1 §1):

> *"It does not just visualize — it analyzes, scores, recommends, and coaches. After identifying the shape of the user's Cognitive Knowledge and Epistemic Content, it helps the user enhance their Cognitive and Epistemic Knowledge. Like having your own local AI."*

Its canonical question (Concept Paper v3.1 §2):

> *"Is my universe healthy? If not, where does it need to be handled?"*

This audit tests, for each of the 7 modes × each of the 4 layers, **"can the architecture today achieve the mode's concept at this layer?"** Findings classify into:

- **🟢 Ships (full purpose-achievement)** — concept is delivered as designed
- **🟡 Degraded (partial purpose-achievement)** — concept partly delivered; gaps reduce diagnostic value but don't break it
- **🔴 Broken (purpose NOT achieved)** — concept fails on real data; the mode lies or shows nothing useful
- **⏳ Future-layer** — concept reserved for MIG-025/026/027; not in scope for MIG-024 ship

The audit's output: a clear list of what MUST ship in MIG-024 §N close-out for Sight v5 to achieve its purpose at Layer 1, vs what's polish that can wait.

---

## §1 · The four-layer purpose-test

Each mode has a Layer 1 deliverable today (visible in the dome) + Layer 2/3/4 deliverables in future MIGs. The purpose-test for each layer:

| Layer | The purpose-question |
|---|---|
| **Layer 1 — Visual foundation** | Does the dome show the TRUTH of the mode's wedge basis × strata distribution? |
| **Layer 2 — Diagnostic** | Does the Findings card correctly identify health-vs-unhealth signals from the mode's data? |
| **Layer 3 — Recommendation** | Do the LLM-generated recommendations name real notes + actionable next steps? |
| **Layer 4 — Coaching** | Does conversational mode walk the user through executing recommendations correctly? |

A mode whose Layer 1 LIES is broken — Layers 2–4 inherit the lie. **Layer 1 truth is the precondition for everything else.** This audit prioritizes Layer 1 truth + identifies what blocks it per mode.

---

## §2 · Per-mode purpose-achievement status

### §2.1 Mode R — Regions

**Concept** (mode-concepts §4): user-defined libraries × strata distribution; the bridge between top-down organization and bottom-up scholarly scaffold; surfaces "where in MY organizational scheme is each cognitive layer?"

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🟢 **Ships** | Library wedges sized correctly; library names on rim (fix-3); stars at correct strata; mixed Arabic+Latin renders. Verified on Eisa's 7,636-note universe. |
| **L2 Diagnostic** | ⏳ Future (MIG-025) | "*Library X has L1-L2 only — pretending to be developed*" alert needs computing aggregations the cache supports |
| **L3 Recommendation** | ⏳ Future (MIG-026) | |
| **L4 Coaching** | ⏳ Future (MIG-027) | |

**Mode R achieves its Layer 1 purpose today.** The wedge-hover tooltip + wedge-click→filter polish would enhance but doesn't gate purpose-achievement.

---

### §2.2 Mode L — Link Types

**Concept** (mode-concepts §5): the Living Link cognitive vocabulary as a navigable axis; reveals the user's cognitive-move signature.

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🟡 **Degraded** | All 10 wedges render in canonical order; stars positioned by `dominantLinkType`; connector colors per typed kind correct. **BUT**: the `untyped` wedge SHOULD render hollow with gold frame to distinguish "no typed links" from "typed-link-untyped" — currently solid like every other wedge. The "**discipline gauge**" reading (§5.3 — "*the untyped wedge size is a direct measure of how much of your linking is undisciplined*") is muted because untyped notes don't visually distinguish from typed ones. |
| **L2 Diagnostic** | ⏳ Future (MIG-025) | Tie-resolution surface (ambiguity-surfacing pattern) needs Layer 2 |
| **L3** | ⏳ Future | |
| **L4** | ⏳ Future | |

**Mode L's Layer 1 purpose is degraded by the missing hollow rendering.** The "*your L7-L8 stars hollow with gold frame in `derives-from`*" diagnostic Eisa-confirmed as the killer single-look read (§5.3) is **not visible today**. Hollow rendering is on the §11.1 fix-N list (fix-8). **Fix-8 is purpose-achievement-blocking for Mode L.**

---

### §2.3 Mode T — Time

**Concept** (mode-concepts §6): temporal axis of cognitive development; when did each cognitive depth emerge?

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🟢 **Ships** | 12 month wedges; locale-aware month names; today-tint correctly gated to T (fix-4); stars correctly positioned by `createdMonth`. Verified on Eisa's universe — the MAR-APR-MAY clustering matches recent activity. |
| **L2 Diagnostic** | ⏳ Future (MIG-025) | Year-disambiguation; trajectory analysis via `note_state_history`; cognitive-cooling alert |
| **L3 / L4** | ⏳ Future | |

**Mode T achieves its Layer 1 purpose for typical (≤3 year) universes.** Year-collapse on long universes is a degradation, not a Layer 1 break. Year-disambiguation is rightly Layer 2 (MIG-025).

---

### §2.4 Mode C — Confidence

**Concept** (mode-concepts §7): the certainty axis; reveals the stratum-confidence mismatch (intellectual overreach diagnostic).

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🔴 **BROKEN** | The chrome ships (4 wedges, canonical order, alpha encoding wired). **BUT**: §2 backfill writes `NULL AS confidence_alpha` → every star defaults to alpha 0.45 → every star (except contested) lands in the `hypothesis` wedge → **the hypothesis wedge contains 99% of the universe artificially; the evidence/established wedges are empty regardless of actual data.** Mode C is **functionally non-operational**. The triple-tautology read (§7.6) — angular + alpha + color all encoding confidence — is broken because the angular dimension is uniformly hypothesis. Eisa-test on the current build would show all stars at hypothesis-alpha throughout the wedges. |
| **L2 Diagnostic** | ⏳ Future (MIG-025) | The stratum-confidence mismatch alert (the killer single-look read at §7.3 — "*high-stratum notes at hypothesis = intellectual overreach*") is the highest-value Mode C diagnostic and CANNOT compute on broken Layer 1 data |
| **L3 / L4** | ⏳ Future | |

**Mode C purpose is NOT achieved today.** Fix-7 (§11.1) replaces the NULL placeholder with the SQL aggregation defined in §7.7 of the deep-dive. **Fix-7 is purpose-achievement-blocking for Mode C — without it, Sight v5 ships with a mode that visibly lies.** Highest-priority §N item.

---

### §2.5 Mode S — Stages

**Concept** (mode-concepts §8): the developmental arc; reveals the Spark-to-Maturity ratio + Dormancy-at-high-strata diagnostic.

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🟡 **Degraded** | 6 wedges in canonical lifecycle order; stars positioned by `note_meta.stage` via json_extract; PropertyEditor shipped. **Three gaps**: (a) **Muted wedge background for Dormancy + Archival** (Eisa-confirmed §8.6 + §11.1 fix-9) — the "active vs inert lifecycle phases" visual encoding is missing, so the inert portion of the universe doesn't pop visually; (b) **Hollow rendering for unstaged notes** (gold frame) — same as Mode L's untyped issue, scaled to Mode S; (c) `stage.*` i18n keys may not be populated in 13 non-en/ar locales (§11.2 verification). |
| **L2 Diagnostic** | ⏳ Future (MIG-025) | Spark-to-Maturity ratio; Dormancy at high-strata; Stage × Time / × Acts / × Maturity-size compositions |
| **L3 / L4** | ⏳ Future | |

**Mode S's Layer 1 purpose is partially achieved.** The lifecycle wedges work; the active-vs-inert visual distinction is missing. **Fix-9 (muted backgrounds) closes the visual purpose gap.** Hollow rendering (fix-8) closes the missing-stage gap. Both are §N polish, neither is a hard block.

---

### §2.6 Mode A — Acts

**Concept** (mode-concepts §9): the methodological vocabulary; reveals the bottleneck-at-Tension diagnostic + the cognitive-arc signature.

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🟡 **Degraded — content gap, not code gap** | 6 wedges in canonical Five-Acts order; stars positioned by `actsPrimary` via `json_extract(properties_json, '$.act')`. **BUT**: per Concept Paper §6 ("CE Layer 2 — partial"), most notes don't have `act` set on real universes. **Eisa's universe will show a near-empty Tension wedge AND a heavy Unacted wedge** — not because Eisa avoids Tension, but because the data isn't tagged. The mode's diagnostic value is degraded until users tag (slow + manual) OR CECE adds an Acts classifier (Layer 2 work in MIG-025). PLUS: fix-10 (§11.1) — verify the frontmatter field name is `act` vs `acts` vs `action`. If mismatch, ALL notes appear Unacted regardless of user input. |
| **L2 Diagnostic** | ⏳ Future (MIG-025) — **CECE Acts classifier moves here** | The bottleneck-at-Tension alert + Acts × Time/Stage/Confidence compositions are Layer 2 |
| **L3 / L4** | ⏳ Future | |

**Mode A's Layer 1 purpose is achievable in principle, degraded in practice by data sparsity.** The chrome will ship correctly; the visual-honesty problem is "user sees mostly Unacted because that's the truth." This is actually GOOD purpose-achievement at Layer 1 — it shows the user the truth that they haven't been tagging Acts. Surfacing the data sparsity IS the diagnostic. **Fix-10 (verify field name) is purpose-achievement-blocking** if the mismatch exists; otherwise it's a no-op confirm.

---

### §2.7 Mode P — Provenance

**Concept** (mode-concepts §10): the architectural-keystone mode; cross-civilizational epistemic-origin signature.

| Layer | Status | What's working / what's blocking |
|---|---|---|
| **L1 Visual** | 🟢 **Ships** (with caveat) | 12 wedges in canonical Universal Taxonomy order; family extraction from `testimony/authoritative` → `testimony` works; `sources.label.*` i18n complete in 15 locales; empty-state CTA shipped (fix-3). **The caveat**: like Mode A, real universes start mostly Unsourced — Eisa's universe will show a heavy Unsourced wedge until CECE classifies. **Unlike Mode A, this is FINE purpose-wise** — the empty-state CTA is part of the design (D-V6.α); the visual-as-prompt loop into Source Review is the intended Layer 1 user flow. |
| **L2 Diagnostic** | ⏳ Future (MIG-025) | Cross-civilizational signature analysis (uniquely Constellation); methodology gap alert; source × stratum compositions |
| **L3 / L4** | ⏳ Future | Civilizational-tradition-matching coaching (MIG-027) — profoundly novel |
| **Hollow for unsourced** | ❌ Pending (fix-8) | The hollow + gold-frame signal would visually distinguish "no source" from "source = X" within the Unsourced wedge edge |

**Mode P's Layer 1 purpose is achieved.** The empty-state CTA already handles the data-sparsity case correctly (visual-as-prompt). Hollow rendering (fix-8) enhances; doesn't gate.

---

## §3 · Cross-mode infrastructure purpose-achievement

### §3.1 Hollow rendering with 4 frame colors (fix-8)

**Touches every mode.** Without it:
- Mode L can't surface the "*your L7-L8 in `derives-from` are hollow*" worldview-lineage diagnostic
- Mode C's missing-confidence stars are invisible
- Mode S's unstaged stars are invisible
- Mode A's Unacted-wedge stars look identical to tagged stars
- Mode P's Unsourced-wedge stars look identical to sourced stars
- The 4-frame-color cascade (mode-data → no-links → no-props → no-content) is the universal incompleteness vocabulary; without it, the "what's missing in your universe" diagnostic is invisible across the board

**Fix-8 is the single highest-leverage §N item — it lights up incompleteness diagnostics across all 7 modes simultaneously.**

### §3.2 Wedge-hover tooltips + wedge-click→filter

Every mode would benefit. Not purpose-achievement-blocking — the user can still SEE the dome correctly; tooltips and filtering enhance navigation. **§N polish, not §N must-have.**

### §3.3 The ambiguity-surfacing pattern (Layer 2)

Logged from Mode L's tie-breaking discussion (§5.7); reapplied to Mode C's confidence ties (§7.8). This is Layer 2 infrastructure (MIG-025) — not §N.

### §3.4 The mode-switch 600 ms ease animation

Currently snaps. Concept Paper v3.1 §6 specs the 600 ms ease. Not purpose-achievement-blocking — visual polish only. Defer to MIG-025 or beyond.

---

## §4 · MIG-024 §N close-out — must-haves vs polish

### §4.1 PURPOSE-ACHIEVEMENT-BLOCKING (must ship in §N)

| ID | What | Blocks which mode | Effort |
|---|---|---|---|
| **fix-7** | `confidence_alpha` SQL aggregation in §2 backfill | **Mode C — currently broken/lies** | ½ day (already drafted) |
| **fix-8** | Hollow rendering + 4 frame-color cascade per §2 | **All 7 modes — incompleteness diagnostics invisible** | ~1 day (single render.ts change) |
| **fix-10** | Verify Mode A frontmatter field name `act` consistency | **Mode A — if mismatch, ALL notes appear Unacted regardless of user input** | ½ day verification + ½ day fix if needed |

**These three are non-negotiable for Sight v5 to achieve its Layer 1 purpose.** Without fix-7, Mode C lies. Without fix-8, every mode's "what's missing" diagnostic is invisible. Without fix-10 (if mismatch exists), Mode A is broken.

### §4.2 PURPOSE-ENHANCING (ship in §N if time permits; defer otherwise)

| ID | What | Affects which mode | Effort |
|---|---|---|---|
| **fix-9** | Muted wedge background for Mode S Dormancy + Archival (`#f0e9cb`) | Mode S — active-vs-inert visual distinction | ½ day |
| **§N polish: wedge-hover tooltips** | All modes | Navigation polish; not purpose-blocking | 1 day |
| **§N polish: wedge-click → auto-filter** | All modes | Navigation polish | 1 day |
| **§N polish: locale-key verification** (`stage.*`, `acts.*`, `linkTypes.*`) | Modes S, A, L (Modes T + P + C already verified) | Quality polish | ½ day |
| **§N polish: today-marker beyond month tint** | Mode T | Polish; not purpose-blocking | ½ day |
| **§N polish: Mode L `supersedes` wedge highlight on link add** | Mode L | UX polish | ½ day |
| **§N polish: Mode P CECE scan progress integration** | Mode P empty state | UX polish | ½ day |
| **§N polish: Mode S archival workflow integration** | Mode S | UX polish | ½ day |

### §4.3 NOT in §N (Layer 2/3/4 territory — MIG-025 onward)

Everything in §11.3, §11.4, §11.5 of the mode-concepts doc. Confirmed correct allocation: those items genuinely require new computation/inference layers beyond what Layer 1's data substrate offers.

---

## §5 · MIG-025 readiness gates

The deep-dive surfaced ~25 distinct Layer 2 diagnostics across 7 modes. Before MIG-025 Architect opens, the following preconditions need to be true:

1. **Layer 1 truth across all 7 modes** — purpose-achievement-blocking items (fix-7, fix-8, fix-10) shipped + Eisa-tested. Layer 2 reads against Layer 1's cache; if cache lies, diagnostics inherit the lie.
2. **CECE Acts classifier scope decision** — does the Acts classifier ship in MIG-025 §N (alongside the diagnostic engine) or as a separate sub-cluster? Mode A is the most data-starved mode; this matters.
3. **The ambiguity-surfacing pattern UI surface** — design + implementation locked early in MIG-025 since multiple modes depend on it (Mode L ties, Mode C ties, future modes).
4. **The 5 highest-value Layer 2 findings** (per §11.3 of mode-concepts) — concretely scoped before Architect opens:
   - Mode C stratum-confidence mismatch
   - Mode S Dormancy-at-high-strata
   - Mode A Tension-avoidance
   - Mode P cross-civilizational signature
   - Mode T trajectory analysis (note_state_history consumer)

---

## §6 · Concept Paper v3.2 amendments needed

Compiled from the deep-dive divergences + the audit findings above. To fold into Concept Paper v3.2 at MIG-024 §N close-out:

| # | Amendment | Source |
|---|---|---|
| 1 | **Calendar rim labels are PER-MODE**, not always months (revokes v3.1 §5.2 "always present temporal reference") | fix-3 already shipped; doc not yet updated |
| 2 | **Milky Way wash temporarily removed** pending PJ-035 real-data shipment | fix-3; doc not yet updated |
| 3 | **Hollow rendering with 4 frame-color cascade** — promote to v3.2 §5 visual encoding (was just "hollow" with no semantic in v3.1) | mode-concepts §2 |
| 4 | **The four-constants → seven-encodings expansion** (radial / size / brightness / color + hollow-fill + frame-color + wedge-background) — restructure v3.2 §7 to acknowledge the expanded encoding vocabulary | mode-concepts §1.1–§1.2 |
| 5 | **Wedge background as data-encoding** (Mode S muting precedent + civilizational-pluralism principle blocks it for Mode P) — new §5.x in v3.2 governing when wedge backgrounds may carry data | mode-concepts §8.6 + §10.6 |
| 6 | **Per-mode wedge label specification** — replace v3.1 §6's terse table with the 7 mode-specific articulations from mode-concepts §4–§10 | mode-concepts §4–§10 |
| 7 | **Ambiguity-surfacing pattern** as a Layer 2 cross-cutting subsystem | mode-concepts §11.3 |
| 8 | **The five highest-value Layer 2 findings** — make them explicit in v3.2 §11 (the Layer 2 ship target) | mode-concepts §11.3 |
| 9 | **Civilizational pluralism principle** as a top-tier design constraint (was implicit in v3.1; explicit in mode-concepts §10.6) | mode-concepts §10.6 |
| 10 | **Maturity-as-size vs Maturity-as-stage** clarification — explicit in v3.2 to prevent future confusion | mode-concepts §8.6.1 |

---

## §7 · Decisions surfaced for Eisa

These are the validation-pass decisions that need Eisa input before §N close-out:

| # | Decision | Default if Eisa silent |
|---|---|---|
| **D-V8** | Should fix-9 (Mode S muted backgrounds) ship in §N alongside fix-7/8/10, or wait? Already Eisa-confirmed in design; just confirm shipping cadence. | Ship in §N alongside the others (small effort; confirmed design) |
| **D-V9** | CECE Acts classifier — ship as part of MIG-024 §N (so Mode A becomes data-rich on Eisa-test), or wait for MIG-025? | Wait for MIG-025 (the V3-§7.b llama.cpp work + Acts-classifier prompt is non-trivial) |
| **D-V10** | The wedge-hover tooltip + wedge-click→filter §N polish items — ship in §N, or defer to MIG-025? | Ship in §N (~1 day each; high navigation-value; well-bounded) |
| **D-V11** | The Concept Paper v3.2 fold-in — happens in the §N close-out commit alongside the fix-N's, or as a separate commit before? | Same commit (one §N close-out commit folds everything) |
| **D-V12** | Click-order canonicalization — should it stay R → L → T → C → S → A → P (current; calibrated to user investment), or move P earlier given its architectural-keystone status? | Keep current order — calibrated to user experience, not architectural weight |

---

## §8 · Audit verdict

**Sight v5 will achieve its core purpose given its mode concept** — *conditional on shipping the 3 purpose-achievement-blocking fixes in §N close-out*:

- **fix-7** (Mode C confidence_alpha) — without it, Mode C lies
- **fix-8** (hollow rendering + 4 frame colors) — without it, every mode's "what's missing" diagnostic is invisible
- **fix-10** (Mode A field name verification) — without it, if mismatch exists, Mode A is broken

The other §N polish items enhance Layer 1 but don't gate purpose-achievement. The 4-MIG roadmap (MIG-024 → 025 → 026 → 027) is correctly scoped given the deep-dive insights. No re-architecture needed — the architecture is sound; the implementation has 3 specific gaps (and ~7 polish items) before Layer 1 ships at full purpose.

**Recommendation**: complete the 3 purpose-blocking fixes + Eisa-test, then close MIG-024 §N. Fold mode-concepts deep-dive into Concept Paper v3.2 in the same close-out commit.

---

**End of v0.1.** This audit has the green light to drive MIG-024 §N close-out scope decisions.
# MIG-021v2 — Epistemic Classifier Redesign — Architect

**Status**: Phase 1 (Architect) — awaiting Boss Phase 2 sign-off.
**Date**: 2026-05-09
**Supersedes**: [`MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`](MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md) — original 11-flat-source design.
**Trigger**: Boss directive 2026-05-09 — *"It would be easier for regular users to select the right source if we include the whole taxonomy. … We will give the user the choice to choose the right level."*
**Scope ratification**: Boss 2026-05-09 — Option B (two parallel fields) + at least 2 levels of depth on the horizontal axis.
**Horizontal taxonomy**: Eisa-authored at `docs/sources-of-knowledge-diagram.html` (11 parents × 3-5 leaves = 52 horizontal nodes incl. root).
**Vertical taxonomy**: Eisa-authored at `docs/epistemic-content-taxonomy-chart.html` (5 branches × ~218 nodes deep).
**Anchored against**: [`docs/Constellation-Sight-Concept-Paper-v2.0.md`](../../docs/Constellation-Sight-Concept-Paper-v2.0.md) §7 + §8 (note: §7 needs a substantive amendment — see §11 below).

---

## §1 · Why redesign

Original MIG-021 scoped the Sources field to 11 flat horizontal items. Boss tested §1A → §1C end-to-end (gates 1/2/3 PASS) and surfaced: leaves are easier to recognize than abstract parents. Per the Boss-ratified Option B, ship **two parallel fields** per note:

- **`sources:` (horizontal axis)** — *"How did this knowing get into your universe?"* — now with 2 levels of depth, **using Eisa's `docs/sources-of-knowledge-diagram.html` as canonical**.
- **`content_type:` (vertical axis)** — *"What KIND of cognitive content is this?"* — the full ~218-node tree from `docs/epistemic-content-taxonomy-chart.html`, lifted intact.

Both fields are multi-select, pickable at any depth. The user can pick a parent ("Testimony" / "Semantic contents") OR a deep leaf ("Authoritative testimony" / "Constructed idea") OR both.

---

## §2 · The horizontal-axis taxonomy (Eisa-ratified)

Source of truth: `docs/sources-of-knowledge-diagram.html`. Below is a flat enumeration for the build phases. The diagram itself is the authoritative visual reference — this table is the data view.

### §2.1 · Eleven parents with tier metadata

| ID | English | Arabic | Transliteration | Tier |
|---|---|---|---|---|
| **S1** | Perception / Sensation | الحِسّ | *pratyakṣa* | **1** universally accepted |
| **S2** | Inference / Reason | العَقل | *anumāna* | **1** universally accepted |
| **S3** | Testimony | الخَبَر | *śabda* | **2** broadly accepted |
| **S4** | Mass-transmission | التَّواتُر | — | **3** school-specific |
| **S5** | Comparison / Analogy | القياس | *upamāna* | **2** broadly accepted |
| **S6** | Postulation / IBE | الاستنباط الافتراضي | *arthāpatti* | **3** school-specific |
| **S7** | Non-apprehension | عَدَم الإدراك | *anupalabdhi* | **3** school-specific |
| **S8** | Memory | الذاكرة | *smṛti* | **2** broadly accepted |
| **S9** | Innate disposition / Intuition | الفِطرة / الحَدْس | — | **2** broadly accepted |
| **S10** | Inspiration / Mystical apprehension | الإلهام / الكَشْف | — | **3** school-specific |
| **S11** | Revelation | الوحي | — | **3** school-specific |

Tier coloring (per the diagram): teal `#0f6e56` for Tier 1, purple `#534ab7` for Tier 2, amber `#854f0b` for Tier 3.

### §2.2 · Forty-one sub-leaves (Eisa-authored, traditional vocabulary)

Each parent's children, in the order Eisa specified:

| Parent | Sub-leaves (English / Arabic / transliteration) |
|---|---|
| **S1 Perception** | External perception (الإدراك الخارجي / *bāhya pratyakṣa*); Internal perception (الإدراك الباطني / *mānasa pratyakṣa*); Self-perception (الشُّعور بالذات / *svasaṃvedana*); Extraordinary perception (الإدراك الفائق / *yogaja* / *mushāhadah*) |
| **S2 Inference** | Deductive inference (الاستنباط البُرهاني); Inductive inference (الاستقراء); Abductive inference (الاستدلال الافتراضي); Necessary reason (العَقل الضَّروري); Speculative reason (العَقل النَّظَري) |
| **S3 Testimony** | Direct witness testimony (الشَّهادة المباشرة); Reported testimony (الخَبَر المنقول); Authoritative testimony (خَبَر الثِّقة / *āpta-vacana*); Scriptural testimony (النَّقل الشَّرعي) |
| **S4 Mass-transmission** | Verbal mass-transmission (تَواتُر لَفظي); Meaning mass-transmission (تَواتُر مَعنوي); Practical mass-transmission (تَواتُر عَمَلي) |
| **S5 Comparison** | Analogy by ratio legis (قياس العِلَّة); Analogy by indication (قياس الدِّلالة); Analogy by resemblance (قياس الشَّبَه); A fortiori analogy (قياس الأَوْلى) |
| **S6 Postulation** | From perceived fact (من مُعطى مُشاهَد / *dṛṣṭārthāpatti*); From heard fact (من خَبَر مَسموع / *śrutārthāpatti*); Inference to best explanation (الاستدلال على أفضل تفسير) |
| **S7 Non-apprehension** | Prior absence (العَدَم السابِق / *prāgabhāva*); Posterior absence (العَدَم اللَّاحِق / *pradhvaṃsābhāva*); Mutual absence (العَدَم التَّبادُلي / *anyonyābhāva*); Absolute absence (العَدَم المُطلَق / *atyantābhāva*) |
| **S8 Memory** | Recollection (التَّذَكُّر); Recognition (التَّعَرُّف / *pratyabhijñā*); Episodic memory (ذاكرة الأحداث); Semantic memory (ذاكرة المعاني) |
| **S9 Innate disposition** | Primordial disposition (الفِطرة, Sunni); Intuition of first principles (بَدَهيات العَقل); Innate moral knowledge (المعرفة الأخلاقية الفطرية / *liángzhī*); Self-evident axioms (العُلوم الضَّرورية) |
| **S10 Inspiration** | Ilhām (الإلهام); Kashf (الكَشْف); True dream-vision (الرُّؤيا الصَّادِقة) |
| **S11 Revelation** | Recited revelation — Quran (الوحي المتلوّ); Non-recited revelation — Sunnah (الوحي غير المتلوّ); Modes of receiving revelation (أوجُه نزول الوحي) |

### §2.3 · Plus the 12th opt-out token (unchanged from §1A)

`unclassifiable` — leaf decision; no sub-categories; suppresses future classifier suggestions on a note.

### §2.4 · Totals

- 11 horizontal parents (Eisa-canonical, with tier metadata)
- 41 sub-leaves (Eisa-canonical, traditional terms)
- 1 opt-out token
- = **53 horizontal node IDs** total

---

## §3 · The vertical-axis taxonomy (already Eisa-ratified)

Source of truth: `docs/epistemic-content-taxonomy-chart.html`. ~218 nodes across 5 top-level branches (Sensory inputs · Symbolic entities · Semantic contents · Epistemic states · Higher-order constructs), depth varies, max depth 5.

Field name in frontmatter: `content_type:` (Q1 in §12 — open to rename).

Stable IDs: kebab-case slugs derived from English labels, parent-prefix on collision. Final scheme decided at extraction time per Q2.

---

## §4 · What's preserved from MIG-021 §1A-§1C

| What shipped | Status under v2 |
|---|---|
| `note_meta.sources` SQLite column (text JSON list) | KEEP — stores deeper IDs |
| `sources_suggestions` queue table | KEEP — same shape, now spans both axes |
| Frontmatter `sources:` field + parser (`extract_sources`) | KEEP — same parser pattern, validates against expanded SOURCE_IDS |
| `sources::write_sources_to_db` + `rewrite_frontmatter_sources` | KEEP — same |
| `Suggestion` + `SuggestionRecord` structs | KEEP — adds optional `axis` field ("horizontal" / "vertical") |
| `classifier::classifier_suggest_for_note` IPC | KEEP — same signature, larger candidate vocabulary, returns suggestions for both axes |
| `classifier::tier1_embedding` cosine pipeline | KEEP — same math, ~53 horizontal + ~218 vertical = ~271 cached vectors |
| Source Review panel queue + Accept/Edit/Reject flow | KEEP — flow stays; Edit picker swaps from flat checkbox grid to tree picker (×2, one per axis) |
| 7 Tauri IPCs from §1A/§1B/§1C | KEEP — same names, same signatures |
| EN+AR i18n for 11 source labels + descriptions + evidence + chrome | KEEP — labels unchanged for 11 parents; new labels needed for 41 sub-leaves and 218 vertical nodes |

What gets ADDED:
- New `note_meta.content_type` column (vertical axis JSON list)
- New frontmatter `content_type:` field + parser (mirrors `extract_sources`)
- 3 new IPCs for the content_type field (mirror sources IPCs)
- New `TaxonomyTreePicker.svelte` component (replaces flat grid in Edit mode; reused by PropertyEditor in §1D')
- Expanded classifier source-definitions covering ~271 nodes total
- Tier metadata on horizontal parents (used in tree-picker badges + classifier confidence-fallback per §6)
- ~270 new bilingual i18n entries
- Tri-script display support (EN + AR + transliteration where present per §2.2)

What gets THROWN AWAY: nothing. The original §1C flat-checkbox grid was replaced as scope evolved; old code can be removed in §1J' /simplify pass.

---

## §5 · Tree-picker UI

New component: `src/lib/sources/TaxonomyTreePicker.svelte`. Mirrors the visual language of Eisa's two HTML diagrams (the user already understands these patterns).

Design principles:
- **Hierarchical tree** with expand/collapse chevrons per node, mirroring `sources-of-knowledge-diagram.html` for horizontal and `epistemic-content-taxonomy-chart.html` for vertical
- **Multi-select via checkbox per node** (parent + child can both be checked)
- **Tier-based color coding** on horizontal parents (teal / purple / amber per §2.1) — preserves the diagram's visual grammar
- **Tri-script display** — EN primary, AR secondary, Sanskrit/Pali transliteration tertiary in italic muted color (where present)
- **Search/filter input** at the top — auto-expands ancestors of matching nodes
- **Two trees side-by-side** in Edit mode (horizontal sources on left in LTR, right in RTL; vertical content_type on the other side); tabbed below 1200px viewport
- **Keyboard navigation** — arrow keys move, Space toggles, Enter commits
- **RTL-aware** — chevrons flip; tree indent flows right-to-left in Arabic
- **`dir="auto"`** at root for native bidi rendering

Used by: `SourceReviewPanel.svelte` (Edit mode) AND `PropertyEditorSourcesField.svelte` (PropertyEditor combobox in §1D').

---

## §6 · Classifier extension

Tier-1 (e5-small embedding-similarity) generalizes:

- Embed each of the ~53 horizontal definitions + ~218 vertical definitions = ~271 cached vectors at first call (~5 sec one-time, OnceLock-cached)
- Per note: embed text once, compute cosine similarity to all 271, return TWO sets:
  - Top-3 horizontal (with axis="horizontal", confidence, evidence)
  - Top-3 vertical (with axis="vertical", confidence, evidence)
- Both sets feed the same queue. Source Review panel renders them grouped by axis in the same card.

**Tier-aware confidence fallback** (new in v2): when the classifier's top-1 horizontal pick is a Tier 3 source (Mass-transmission / Postulation / Non-apprehension / Inspiration / Revelation) and confidence is below a threshold (e.g., 0.55), suggest the parent of the next-highest Tier 1 or Tier 2 candidate as an alternative. Reasoning: contested epistemic categories shouldn't be silently auto-suggested for non-religious / non-philosophical notes.

**Leaf-vs-parent suggestion strategy** (Q5 in §12):
- If leaf-level confidence ≥ 0.55: suggest the leaf
- Otherwise: suggest the parent (less specific, but more reliable)

Tier-2 (Qwen3-1.7B via llama.cpp, deferred to §1H'): handles deep hierarchy via structured prompt with explicit GBNF grammar enforcing one-of-271 output; produces dynamic per-note evidence (a quote from the actual text rather than the generic per-source signature).

---

## §7 · Phase re-sequence

Each phase is one landable commit with verification.

**Already shipped (preserved):**
- ~~§1A — Schema + frontmatter parser + 3 IPCs~~ — ✅ shipped (`4d6ef37`); adapts via §1A'
- ~~§1B — Tier-1 classifier (11 sources)~~ — ✅ shipped (`dcbd40e`); adapts via §1B'
- ~~§1C — Source Review panel (flat picker)~~ — ✅ shipped (`4e70393` + fixes `4769fbe` `c3f3e96` `ec288fe`); Edit-mode picker replaced via §1C'

**New cascade:**

| Phase | What | User-test gate? |
|---|---|---|
| **§0v2** | Eisa approves this Architect | Boss approval |
| **§1A'** | Extract `sources-of-knowledge-diagram.html` AND `epistemic-content-taxonomy-chart.html` to structured Rust + TS data files (`taxonomy_data.rs` + `taxonomy.ts`). Add `note_meta.content_type` column. Add `content_type:` frontmatter parser + writer. Add 3 new IPCs (mirrors sources IPCs) | self-verify |
| **§1B'** | Expand classifier source-definitions to ~271 candidates (11 + 41 horizontal + 218 vertical + 1 opt-out). Refit cosine pipeline. Update `Suggestion` to include `axis` tag. Add tier-aware confidence fallback (§6) | self-verify |
| **§1C'** | New `TaxonomyTreePicker.svelte` component. Replace SourceReviewPanel Edit-mode flat grid with tree picker × 2. Update Source Review card to show both-axes suggestion sets grouped | ✅ Boss-test |
| **§1D'** | PropertyEditor integration: two tree pickers between Maturity and Stage rows | self-verify |
| **§1E'** | Right-click context action: "Suggest sources & content type for this note" | self-verify |
| **§1F'** | Background scan job (extends existing — same scan, now classifies both axes) | ✅ Boss-test |
| **§1G'** | i18n full pass — ~270 new bilingual labels (EN+AR canonical; 13 other locales follow chain) | self-verify |
| **§1H'** | Tier-2 download + llama.cpp integration — Qwen3-1.7B suggests both axes via structured prompt with GBNF | ✅ Boss-test |
| **§1I'** | Help docs + User Manual (EN + AR) — explains both axes, tree picker, tier system, choice-of-depth principle | self-verify |
| **§1J'** | `/simplify` checkpoint + 3-agent audit | self-verify |
| **§1K'** | Close-out — orientation v1.79+ bump, Concept Paper v2.0 → v2.1 amendments per §11 | PCS |

3 user-test gates (same count as original Plan): §1C', §1F', §1H'.

---

## §8 · i18n scope

| Surface | New strings (per locale) | This MIG | Other 13 locales |
|---|---|---|---|
| Horizontal parents (11) | 0 (already shipped) | — | — |
| Horizontal sub-leaves (41) | 41 labels + 41 short descriptions = 82 | EN+AR (this MIG) | EN fallback → PJ |
| Vertical nodes (~218) | 218 labels + ~218 descriptions = ~436 (Eisa pre-authored EN+AR pairs in chart) | EN+AR (lifted from chart, no new authoring) | EN fallback → PJ |
| Tree picker chrome (search box, expand all, collapse, axis tabs, tier badges) | ~12 | EN+AR (this MIG) | EN fallback → PJ |
| **Total NEW per locale** | **~530** (~94 new authoring + ~436 lifted) | EN + AR | 13 others queued |

13-locale follow-up PJ allocated (same convention as MIG-014).

---

## §9 · Sight v5 mode P (Provenance) impact

The original Plan had mode P show 11 wedges. With redesign:

**Option α** — Mode P shows the **11 horizontal parents** as wedges; clicking a wedge zooms into a sub-view of that parent's leaves. Familiar shape; drill-down on demand. Tier coloring inherited (teal/purple/amber wedges).

**Option β** — Mode P shows wedges at WHICHEVER LEVEL THE USER PICKED. If they classified at parent, wedge is parent; if at leaf, wedge is leaf. Granular but unstable.

**Option γ** — Two separate Sight modes: P (Provenance / horizontal) and N (oNtology / vertical). User toggles axis.

Recommendation: **Option α** for first ship. Decision deferred to MIG-022 (Sight v5 visual ship); not blocking this MIG.

---

## §10 · Tier system UX (NEW design dimension introduced by Eisa's diagram)

Eisa's diagram introduces 3 tiers of acceptance. The redesign should reflect this in UX:

### §10.1 · Tree picker
- Horizontal parents render with their tier color in the tree picker (matching the diagram's left-border colors)
- A small legend shows what each tier means

### §10.2 · Source Review panel
- Each suggestion's source badge shows the tier color of its parent (e.g., a "Revelation" suggestion gets an amber-tinted badge; "Perception" gets teal)
- Sub-leaves inherit their parent's tier color

### §10.3 · Settings → Sources
A new optional toggle: **"Hide Tier 3 (school-specific) sources from suggestions"** — for users who don't want Mass-transmission, Postulation, Non-apprehension, Inspiration, or Revelation suggested. User can still pick them manually via PropertyEditor; they just won't auto-surface in the queue.

Default: OFF (all tiers suggested).

### §10.4 · Classifier confidence fallback (per §6)
Tier-aware: when top-1 horizontal pick is Tier 3 and confidence is borderline, suggest a Tier 1 or Tier 2 alternative as the second pick. Avoids inappropriately surfacing contested categories on secular notes.

---

## §11 · Concept Paper v2.0 amendments

The following sections need substantive amendment when this MIG closes:

- **§7.1** — "The 11 sources" table → "The 11 horizontal sources + sub-leaves + tier metadata" (linking to `taxonomy_data.rs`); add §7.1b for the vertical axis
- **§7.2** — frontmatter contract: add `content_type:` field alongside `sources:`
- **§7.3** — three setting paths: tree picker now used (replaces "multi-select dropdown")
- **§7.4** — "Unsourced wedge" still applies; Sight mode P wedge slicing (§9 above) needs ratification
- **§5** — modes table: clarify mode P's wedges are horizontal parents (Option α) with drill-down + tier coloring
- **NEW §7.5** — tier system, tier-aware confidence fallback, Settings → Sources tier toggle

These amendments land in Concept Paper v2.1 in §1K' close-out commit.

---

## §12 · Open questions (locked with sensible defaults; surface if Eisa overrides)

| # | Question | Locked default | Reversible? |
|---|---|---|---|
| Q1 | Field name for the vertical axis | `content_type` | Yes |
| Q2 | Slug scheme for nodes | kebab-case + parent-prefix on collision | Yes |
| Q3 | Multi-select within an axis | YES — allow parent + child both checked | Yes (UI behavior) |
| Q4 | Tree picker layout | Side-by-side ≥1200px; tabbed below | Yes (CSS) |
| Q5 | Default classifier output (parent or leaf) | Suggest at deepest level where confidence ≥ 0.55; fall back to parent | Yes (single threshold constant) |
| Q6 | Tier 3 default visibility | All tiers shown; user can hide Tier 3 in Settings → Sources | Yes (toggle) |
| Q7 | Tier-aware confidence fallback (§6) | ENABLED — when top-1 is Tier 3 and confidence < 0.55, suggest Tier 1/2 alternative | Yes (single flag) |
| Q8 | Concept Paper amendments | Land in v2.1 on §1K' close-out | Yes (single commit) |

---

## §13 · Risk register (delta from original Architect)

| Risk | Severity | Mitigation |
|---|---|---|
| Classifier accuracy drops at leaf level (more candidates) | High | Suggest at parent when leaf confidence < 0.55; user drills down manually if desired |
| Tree picker UX gets unwieldy with 271 nodes | Medium | Search/filter at top + collapse-by-default + sensible expand-on-relevance; mirror diagram UX |
| Two parallel fields per note doubles cognitive cost on every save | Medium | Both fields are OPTIONAL — user can leave either or both empty |
| i18n scope (~530 strings × 15 locales) becomes translation backlog | Medium | EN+AR shipped this MIG; 13 others as PJ; English fallback works correctly today |
| Tier system feels gatekeepy if mishandled in UI | Low | Tier coloring is informational only; nothing is blocked or hidden by default; opt-out in Settings only |
| 13-locale PJ never gets done; non-EN/AR users see English deeper labels | Low | Standard fallback chain; users know convention |
| Concept Paper v2.0 mid-revision; future sessions might miss v2.1 amendments | Low | This Architect documents deltas; v2.1 lands on §1K' close-out |
| Eisa's leaves are scholarly (e.g., *prāgabhāva* / *anyonyābhāva*) — non-expert users may not recognize them | Medium | Hover tooltips on every leaf show plain-language definition + transliteration + Arabic equivalent; no jargon mandatory |
| Some Tier 3 leaves duplicate concepts (e.g., S6 "Inference to best explanation" vs S2 "Abductive inference") | Low | Both classify to legitimate sources; user can pick whichever feels more accurate; not a bug |

---

## §14 · Cross-references

- [`docs/sources-of-knowledge-diagram.html`](../../docs/sources-of-knowledge-diagram.html) — **canonical horizontal-axis taxonomy** (Eisa-authored, this Architect's source of truth for §2)
- [`docs/epistemic-content-taxonomy-chart.html`](../../docs/epistemic-content-taxonomy-chart.html) — **canonical vertical-axis taxonomy** (Eisa-authored, this Architect's source of truth for §3)
- [`docs/epistemic-content-taxonomy.md`](../../docs/epistemic-content-taxonomy.md) — bilingual prose taxonomy (the markdown reference both diagrams synthesize)
- [`MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md`](MIG-021-EPISTEMIC-CLASSIFIER-ARCHITECT.md) — superseded by this v2
- [`MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md`](MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md) — superseded; new Plan lands after this Architect approval
- [`docs/Constellation-Sight-Concept-Paper-v2.0.md`](../../docs/Constellation-Sight-Concept-Paper-v2.0.md) §7 + §8 — needs v2.1 amendments per §11 (deferred to §1K' close-out)
- Memory: `project_sight_classifier_local_llm.md`; `project_sight_taxonomy_foundation.md`; `project_sight_canonical_answer.md`; `project_sight_360_scope_orthogonal.md`

---

**End of MIG-021v2 Architect.**

**Boss review checklist:**
1. **§2 horizontal taxonomy** — confirmed canonical from your diagram ✅ no ratification needed (already your work)
2. **§5 tree picker UI** — approve principles or revise
3. **§6 classifier extension** — approve tier-aware fallback at confidence < 0.55, or override threshold
4. **§7 phase re-sequence** — approve §1A' → §1K' or revise
5. **§9 Sight mode P design** — Option α / β / γ (or defer to MIG-022)
6. **§10 tier UX** — approve tree-picker tier coloring + Settings opt-out for Tier 3 + classifier fallback
7. **§12 open questions** — agree to all eight defaults, or override per question

On approval: a new Plan doc sequences §1A' → §1K' with per-step verification clauses, then Build cascade resumes per Plan-Approval-Equals-Build-Approval discipline.

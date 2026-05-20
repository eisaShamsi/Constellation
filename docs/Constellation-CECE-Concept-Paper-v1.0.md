# Constellation CECE — Subsystem Concept Paper

**Version 1.0 | 2026-05-19**

> **Purpose**: Define CECE (Constellation Epistemic Content Engine) as a subsystem of Constellation — its core concept, why it exists, how it works, what it depends on, and where it sits among the other subsystems. This paper also exists to settle CECE's **user-facing name** (§10): per Eisa's direction, the name should follow from the core concept, so the concept is articulated first.
>
> **Accuracy note**: this paper distinguishes *shipped reality* from *designed-but-not-yet-wired* throughout (see §5). The implementation was mapped from code (`src-tauri/src/cece/`, `src-tauri/src/classifier/`, `src-tauri/src/sources/`) + `lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md`; the intellectual foundation from `docs/epistemic-content-EN.md` + `docs/epistemic-content-taxonomy.md`.

---

## §1 — The core concept (the one sentence)

**CECE reads each note and answers two questions: *what kind of knowledge is this?* and *where did it come from?* — placing every note on a two-axis taxonomy of epistemic content drawn from the convergent structure of the world's major epistemological traditions.**

Everything else in this paper elaborates that sentence. The two questions are the two axes:

- **Content-type axis** (*what kind*): is this a raw sensory input, a symbolic datum, a semantic content (concept / proposition / fact), an epistemic state (doubt / opinion / belief / certainty), or a higher-order construct (hypothesis / theory / law / wisdom)?
- **Source axis** (*where from*): did this knowledge arrive by perception, inference, testimony, mass-transmission, comparison, postulation, non-apprehension, memory, innate disposition, inspiration, or revelation?

CECE is the subsystem that makes the **epistemic texture** of a user's universe visible — not just *what* they know (that's search), and not just *how their notes connect* (that's the link system), but *what kind of knowing each note is* and *what authority it rests on*.

## §2 — Why CECE exists

Constellation's founding mission (`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`): cultivate **wisdom** through the living link system — *"Understanding that survives contradiction is wisdom."*

But not all knowledge carries equal epistemic weight. A reasoned inference, a transmitted testimony, a raw sensory datum, and a synthesized theory are different *kinds* of cognitive object resting on different *sources* of authority. A knowledge-formulation system that treats them all as undifferentiated "notes" cannot help the user reason about the **quality and provenance** of their understanding.

CECE supplies that missing dimension. With it, the user can see things no file manager can show:

- *"80% of my notes are testimony — I've read widely but reasoned little."*
- *"My 'theories' rest on notes still in a doubt state — my convictions are built on sand."*
- *"This whole cluster came from a single source — my view here is narrow."*

These are **formulation diagnostics at the epistemic level**. They serve the Five Acts (Observation → Connection → Tension → Synthesis → Conviction) by revealing where a user's knowledge actually sits in the ascent from raw input to integrated wisdom.

## §3 — The intellectual foundation

CECE's two axes are not invented; they are abstracted from a comparative survey of five civilizations' epistemologies (`docs/epistemic-content-EN.md`). Across Greek/European, Arabic-Islamic, Indian *pramāṇa*, Chinese Mohist/Confucian, and Persian Illuminationist thought, **three structural loci recur**:

1. **Sources** — sensation, inference, testimony, revelation, intuition, innate disposition. (Indian *pramāṇa* canonized up to six; Sunni *uṣūl* recognized five with a distinctive elevation of *tawātur*; the Mohists named three — *wén/shuō/qīn zhī*.)
2. **Contents** — concepts, propositions, facts (Sunni *taṣawwur* / *qaḍiyyah*; Stoic *lekton*; Mohist *shí*).
3. **States** — doubt, opinion, belief, knowledge, certainty (Sunni *shakk* / *ẓann* / *ʿilm* / *yaqīn*; Greek *doxa* / *epistēmē*; Indian *prama* / *aprama*).

The synthesis taxonomy (`docs/epistemic-content-taxonomy.md`) folds loci 2 + 3 into the **content-type axis** (5 branches: sensory inputs · symbolic entities · semantic contents · epistemic states · higher-order constructs) and keeps locus 1 as the **source axis** (11 sources). This is the cross-civilizational spine CECE classifies against — *"five longitude lines on the same globe."*

## §4 — The two axes, precisely

| Axis | Question | Structure (as implemented) |
|---|---|---|
| **Content-type** (vertical) | *What kind of cognitive object?* | 1 root (`epistemic-content`) → 5 branches (`sensory-inputs`, `symbolic-entities`, `semantic-contents`, `epistemic-states`, `higher-order-constructs`) → ~218 sub-nodes, max depth 4. |
| **Source** (horizontal) | *Where did it come from?* | 11 parents (`perception`, `inference`, `testimony`, `mass-transmission`, `comparison`, `postulation`, `non-apprehension`, `memory`, `innate-disposition`, `inspiration`, `revelation`) + 41 leaves + 1 `unclassifiable` opt-out = 53 IDs. |

The content-type axis relates to Constellation's existing **8-level strata** field (Datum → Worldview) — strata is a condensed *elevation* projection of the same taxonomy. CECE's vertical axis is the full version; strata is the simplified user-facing slice.

Both axes support **multi-assignment** (a note can carry primary + secondary sources/types), and both are stored canonically in note frontmatter (`sources:` / `content_type:`) mirrored to `note_meta` for fast reads — the same File-Over-App pattern the rest of Constellation uses.

## §5 — How CECE classifies (shipped reality vs. designed)

CECE classifies each note through an **ensemble of six "catalogers,"** each reading the note through a different Constellation primitive, whose verdicts a synthesis layer combines.

| Cataloger | Lens | Status |
|---|---|---|
| **User-Authority** | Frontmatter (`sources:` / `content_type:` the user has voiced) | **Live.** Absolute precedence — if the user has voiced an assignment, synthesis short-circuits and accepts it. |
| **Structural** | Note structure (citations, headings, blockquotes, code, equations, stance markers) via regex/pattern | **Live.** <5 ms. |
| **Linguistic** | Arabic CAE morphology + Lexical Bridge cross-lingual equivalence | **Live.** Root match → high confidence; surface token → medium; embedding fallback → low. |
| **Graph** | Living-Links typed neighborhood (derives-from / part-of / supports / …) | **Live.** Weighted vote of typed neighbors; needs ≥2 typed neighbors to vote; abstains on orphans. |
| **Semantic** | e5-small ONNX embeddings + kNN to per-Library exemplars | **Live.** Votes the classifications of the 5 nearest already-classified neighbors (cosine ≥0.55); needs ≥3 exemplars; cold-start abstains. |
| **Reasoning** | Local LLM (designed: Qwen3-4B Q5_K_M via llama.cpp, two-step parent→child, GBNF-constrained) | **Designed, NOT wired.** The trait + prompt builder + grammar exist, but the inference function is `None`; the cataloger abstains on every note in the current release. |

**This is the single most important accuracy point in this paper: CECE ships today as a 5-cataloger heuristic ensemble. The local-LLM "Reasoning" cataloger is architected but not yet operational.** Any user-facing copy must not claim "AI/LLM classification" as a shipped capability — it is a designed extension (the llama.cpp wiring is deferred).

**The orchestrator** runs the cheap catalogers first (Pass 1), synthesizes, and only escalates to the expensive Reasoning cataloger (Pass 2) if the cheap ones don't already agree — a cost cascade. The synthesis layer assigns one of three **confidence regimes**:

- **Unanimous** — all voicing catalogers agree → accept silently, high confidence.
- **Strong Majority** — ≥4 of 5 agree (or User-Authority voiced) → accept, surface dissent as "see also."
- **Split** — 3-2 or worse → **CECE refuses to assign** and surfaces the conflicting candidates to the user (Sibling Disambiguation).

The refusal-on-Split behavior is a deliberate epistemic humility: the engine does not guess when its own lenses disagree. Per-Library cataloger reliability is tracked (`cataloger_reliability.json`) and a correction log (`correction_log.ndjson`) records every user override for future calibration.

## §6 — The Source Review workflow

CECE never silently rewrites a note. Its output is a **suggestion queue** the user reviews:

1. A scan (manual today — `classifier_scan_start`; or per-note `classifier_suggest_for_note`) runs the orchestrator and writes to the `sources_suggestions` table — it does **not** touch frontmatter.
2. The **Source Review panel** (right sidebar today) renders each suggestion as a card with the full per-cataloger reasoning trail (on non-Unanimous regimes).
3. The user approves (→ `sources_set_manual` writes the chosen IDs to frontmatter + mirror + correction log) or rejects (→ clears the suggestion, no write).
4. Bulk approve/reject handles the whole queue.

This is the canonical File-Over-App read-then-write-on-consent pattern: the engine proposes; the user disposes; the `.md` file is the source of truth.

*(Note: `backgroundScan` settings — off / on_save / on_startup — are defined but the auto-trigger is not yet wired; scans are manual-only in the current release.)*

## §7 — What CECE reads and writes

**Reads**: note content + frontmatter; the Arabic CAE morphology engine; the Lexical Bridge; the Living-Links typed graph (`note_links`); the e5-small embedding store; per-Library exemplar memory + reliability calibration.

**Writes**: `sources_suggestions` (the queue); on user approval, frontmatter `sources:` / `content_type:` + `note_meta` mirror + `correction_log.ndjson`. Nothing else. CECE is otherwise read-only with respect to user content.

**IPC surface**: `classifier_suggest_for_note`, `classifier_scan_start` / `_cancel` / `_status`; `sources_get_for_note`, `sources_set_manual`, `sources_reject_suggestion`, `sources_clear`, `sources_get_suggestions`, `sources_list_pending_suggestions`, `sources_get_horizontal_taxonomy`, `sources_get_vertical_taxonomy`, plus bulk ops.

## §8 — Place among subsystems

CECE is an **infrastructure subsystem** that other surfaces consume:

- **Sight** reads CECE's source assignments to populate its Provenance mini-dome (and, in the dropped MIG-029, would have read content-type for per-note tradition placement).
- **Search / Index** can filter by source/content-type once assignments exist.
- The **living link system** is *upstream* of CECE (the Graph cataloger reads typed links) and could become *downstream* (a future tension-detector could weight contradictions by epistemic source).

CECE depends on the Arabic engine, the Lexical Bridge, the embedding store, and the link graph. Nothing depends on CECE being *mounted* — it runs (or is dormant) independently of any visualization.

## §9 — Architectural invariants

1. **User authority is absolute** — a voiced frontmatter assignment overrides every cataloger.
2. **Refuse rather than guess** — on cataloger disagreement (Split), surface the conflict; don't assign.
3. **Propose, never impose** — classification produces suggestions; only user approval writes frontmatter.
4. **Local-only** — all inference (embeddings today; LLM when wired) runs on-device; zero cloud path.
5. **File-Over-App** — frontmatter is canonical; `note_meta` is a rebuildable mirror.
6. **Per-Library calibration** — reliability + exemplars are scoped per Library, not global.

## §10 — The naming question (the decision this paper enables)

**The problem**: the engine is internally "CECE" (Constellation Epistemic Content Engine); its user-facing panel is "**Source Review**." But "Source Review" names only *half* the concept — the source axis — and ignores the content-type axis, which is the other half of CECE's core job. As CECE moves to a first-class **left-dock feature** (a Core Plug-in, per Eisa 2026-05-19), it needs a user-facing name that honors the *whole* concept.

The core concept (§1) is: *classify each note's epistemic content by kind and by origin.* A good name should evoke **classifying / placing / understanding the nature of one's knowledge** — not just "reviewing sources."

Candidate names, each measured against the core concept:

| Candidate | Evokes | Fit | Risk |
|---|---|---|---|
| **Source Review** (current) | Reviewing where notes came from | Names only the source axis | Undersells the content-type half |
| **The Cataloger** | Library-science classification (matches the internal "cataloger" architecture) | Captures *both* axes as "classifying knowledge"; warm, human | "Cataloging" may read as mere filing, not epistemic insight |
| **Epistemic Lens** / **Lens** | Seeing the *kind* and *origin* of knowledge | Captures the "see your epistemic texture" purpose | "Lens" was used by old Sight ("Constellation Lens"); collision risk |
| **Provenance** | Origin + authority of knowledge | Strong on the source axis | Again under-weights content-type |
| **Epistemic Content** / **Content Engine** | The literal taxonomy | Accurate, complete | Abstract; not inviting to a non-academic user |
| **Knowing** / **Ways of Knowing** | The cross-civilizational framing (sources + kinds of knowing) | Honors the foundation; evocative | Vague as a dock label |

**My recommendation for your decision**: **"The Cataloger"** (or simply **"Cataloger"**) as the user-facing dock name, with **CECE** retained as the internal engine name (exactly as Sky View / Sight keep internal v-numbers). Rationale: it's the only candidate that naturally spans *both* axes (a cataloger classifies an item by multiple facets — kind and origin), it matches the engine's own architecture (six catalogers), it's a warm human word rather than an abstract one, and it carries the library-science lineage that fits a knowledge-formulation tool. The dock tooltip can carry the fuller phrase ("Cataloger — classify each note's kind and source of knowledge").

### DECISION (Eisa, 2026-05-19): **The Cataloger**

The user-facing name is **The Cataloger**. The internal engine name stays **CECE** (Constellation Epistemic Content Engine) — same pattern as Sky View / Sight keeping their internal names. So:

- **User-facing** (dock button, dock view title, tooltip, Settings toggle, help text): "The Cataloger" / "Cataloger".
- **Internal** (Rust modules `cece/`, IPC names `classifier_*` / `sources_*`, this Concept Paper's engine label, code comments): "CECE".
- The existing right-sidebar **Source Review** panel keeps its name for now (it's the per-note review surface); the left-dock view is "The Cataloger" (the universe-wide home).

**Localization decision (Eisa, 2026-05-19): Arabic = المُصنِّف** (*al-muṣannif*, "the classifier"). Eisa chose the *classifier* sense over the cataloger/indexer sense (مُفهرِس). Guidance for the remaining 13 locales at the build's i18n step: follow the **classifier sense** (the concept Eisa anchored on), not the literal library-"cataloger" word — e.g., 分类器 (zh), 分類器 (ja), 분류기 (ko), Klassifikator (de), Clasificador (es), Classificateur (fr), Sınıflandırıcı (tr), Классификатор (ru), वर्गीकारक (hi), دسته‌بند (fa), המסווג (he), درجہ بند (ur), Classificador (pt). The English user-facing brand stays **"The Cataloger"** (the chosen English name); non-English locales render the classifier-sense equivalent. Final 13-locale values to be confirmed during the build.

## §11 — Future workstreams

- **Wire the Reasoning cataloger** — add the llama.cpp / Qwen3 local-LLM path (deferred from MIG-021v3 §7.b). Turns the 5-cataloger heuristic ensemble into the designed 6-cataloger one.
- **Auto-scan triggers** — wire `backgroundScan` (on_save / on_startup) so classification isn't manual-only.
- **Learned synthesis** (MIG-022) — replace the hand-tuned weighted vote with correction-log-calibrated weights.
- **Temporal axis + warrant classifier + YAML metadata extensions** (MIG-022 reserved).
- **Left-dock Core Plug-in promotion** (today's task, post-naming) — dock button + full-page view reusing `SourceReviewPanel` + `ClassifierScanProgressStrip`.
- **Process the standing queue** — ~4,475 pending suggestions await review in the trial universe; clearing them makes the Provenance data real.

---

*Concept Paper v1.0, cut 2026-05-19. Companion to the implementation contract in `lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md` and the intellectual foundation in `docs/epistemic-content-EN.md`. The paper keeps the internal "CECE" engine label by design (§10 decision); the user-facing name is "The Cataloger." The filename stays `Constellation-CECE-Concept-Paper` since CECE is the durable internal name.*

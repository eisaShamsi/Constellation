# Source Review

*(Constellation Epistemic Content Engine — CECE)*

The Source Review panel is where Constellation asks you to review classifications produced by the **Constellation Epistemic Content Engine** (CECE). Each card in the queue shows a note + the engine's read of how that note fits into your knowledge taxonomy. You Accept, Edit, Reject, or pick a Sibling Disambiguation chip — and over time the engine learns your Library's shape.

This topic explains every part of a Source Review card, what the colored dots mean, when to trust the engine, and how to navigate hundreds of cards without scrolling forever.

> **Two places, one panel.** The cards described here appear both in the **Source Review** tab in the right sidebar and in the full-page **Cataloger** view (the stacked-cards icon in the left dock). They are the same panel and the same engine. The Cataloger gives the queue the whole window plus a note-picker and a "Build all summaries" button — see the **The Cataloger** topic. Everything below applies to the cards in either place.

---

## What CECE actually does

When you classify a note (right-click → "Suggest sources & content type", or via the Settings → Run scan button), CECE runs **six independent catalogers** against the note. Each cataloger reads the note through its own lens — frontmatter, citations, wordstems, linked notes, similar notes, AI judgment — and votes on two questions:

- **Source (horizontal axis)**: where did this knowledge *come from*? Examples: testimony (someone told me), perception (I saw it), inference (I derived it), revelation (sacred text), and eight more.
- **Content Type (vertical axis)**: what *kind* of knowledge is this? Examples: epistemic state (doubt / certainty / belief), semantic content (concept / proposition / fact / theory), sensory input, symbolic entity, higher-order construct (worldview / doctrine).

The two axes are **independent**. A note about "I doubt the moon landing" is testimony (someone reported it) on the source axis + epistemic-states/doubt (the user's stance toward it) on the content-type axis.

After the catalogers vote, a **synthesis layer** combines their votes into a single classification per axis, with one of three confidence regimes:

- **Unanimous** — every voicing cataloger agreed
- **Strong majority** — most agreed, one dissented (note shows the dissenter's name)
- **Split** — no clear majority; the engine "refused to assign" and asks *you* to pick

Everything runs **on your device**. No notes ever leave Constellation.

---

## The two axes in plain language

### Source — *where did this knowledge come from?*

Eleven possible values plus *unclassifiable*:

- **Perception** — first-hand sensory observation
- **Inference** — reasoning from premises (deduction, induction, analogy)
- **Testimony** — someone else's report (a quote, a citation, a referenced source)
- **Mass-transmission** — convergent reports by many independent witnesses (Sunni *al-tawatur*)
- **Comparison** — knowledge by analogy to a known case (legal *qiyās*, scientific analogies)
- **Postulation** — inference to the best explanation (*arthapatti*)
- **Non-apprehension** — knowledge of absence
- **Memory** — recall of past experience
- **Innate disposition** — pre-experiential knowing (*fitrah*)
- **Inspiration** — mystical or creative apprehension (*kashf*)
- **Revelation** — sacred-text or prophetic transmission (*al-wahy*)
- **Unclassifiable** — opt out of this classification

### Content Type — *what kind of knowledge is this?*

Five top-level branches with sub-branches:

- **Sensory inputs** — raw signals (visual, acoustic, chemical, …)
- **Symbolic entities** — signs, symbols, codes
- **Semantic contents** — concepts, propositions, facts, ideas, information
- **Epistemic states** — doubt, belief, opinion, certainty, knowledge, illusion
- **Higher-order constructs** — theories, doctrines, worldviews, paradigms

Both axes have several layers of refinement under each top-level value (e.g. *epistemic-states/knowledge/by-content/propositional* is a leaf).

---

## The six catalogers

Each cataloger is one *lens* through which CECE reads a note. The Source Review card shows them as **six small colored dots** in the top right corner. Hover any dot to see its name + status.

| Dot | Cataloger | What it reads |
|---|---|---|
| 🔵 blue | **Your frontmatter** (User Authority) | The `sources:` and `content_type:` fields you've already set. If you've classified the note yourself, this lens has *absolute authority* — the synthesis adopts your choice and skips the others. |
| 🌹 rose | **Citations & structure** (Structural) | Citations, blockquotes, code blocks, theorem markers, definition phrases ("the concept of X is defined as…"), figure references. Reads the note's structural shape. |
| 🟡 amber | **Wordstems & lexicon** (Linguistic) | Arabic root analysis (CAE), surface keyword matching, cross-lingual term equivalence (Bridge). Catches Arabic-aware classification that pure embeddings miss. |
| 🟢 teal | **Linked notes** (Graph) | Typed Living Links (`[[Note\|supports]]`, `[[Note\|contradicts]]`, etc.) to other classified notes. Inherits classification from neighbors when they cluster. |
| 🟣 violet | **Similar notes** (Semantic) | Embedding-similarity to your already-classified notes (k-Nearest-Neighbor). Pulls in the consensus when this note's content vector clusters with classified notes. |
| 🟢 green | **AI judgment** (Reasoning) | A local LLM (Qwen3-4B Q5_K_M) running grammar-constrained inference. *Not yet active* — model wiring deferred to a later release. The dot stays silent on every card today. |

### Dot status

- **Filled** — voiced + agrees with the synthesis
- **Ringed** — voiced + dissents from the synthesis (this lens picked something different)
- **Dashed outline** — silent (no signal in this lens for this note)

The dot cluster is the at-a-glance ensemble health indicator. A card with all six dots filled is the engine's strongest possible classification (rare). A card with one or two ringed dots is showing its reasoning honestly — the lenses disagreed.

---

## The three confidence regimes

After the catalogers vote, CECE labels each axis with one of three regimes:

- **Unanimous** — every voicing cataloger picked the same primary value. The card has no special pill.
- **Strong majority (one dissent)** — most agreed; one dissenter is shown by name. The card has a purple "Strong majority" pill in the header.
- **Split** — no clear majority. The card has a gold "Catalogers split — needs your call" pill, **a gold left border**, and a Sibling Disambiguation form with chips for you to pick from.

Each axis gets its own regime independently. A card can be Unanimous on horizontal + Split on vertical (or vice versa). The header pill summarizes the worst regime across both axes.

---

## Sibling Disambiguation

When an axis is Split, CECE refuses to guess and instead surfaces the candidate values as **radio chips** under a prompt:

> *"The catalogers split between these candidates. Pick which one fits the note best:"*

You click a chip → the engine writes that pick to the note's frontmatter, removes the card from the queue, and updates per-Library reliability data.

If the OTHER axis was settled (Unanimous or Strong majority), CECE *also* writes that axis's value at the same time — so a single chip click finishes both axes, not just the one you picked. The same card never asks you twice.

If both axes are Split, you pick one chip per axis (two clicks).

---

## The reasoning trail

Each card has a **"▸ Why this classification?"** toggle (or "▾ Hide reasoning" if open). Expanding it shows one row per voicing cataloger:

- **Lens-color dot** matching the dot cluster
- **Cataloger label** (e.g. "Wordstems & lexicon")
- **Self-reported confidence** in brackets: `[high]` `[medium]` `[low]`
- **One-line reasoning** explaining what fired (e.g. *"Linguistic match: vertical → semantic-contents/concept (weight 0.80)"*)
- **Friendly rule chips** below the reasoning, like `Surface keyword match`, `Side-channel preference rule`, `Arabic root match (CAE)` — these are the specific rules each cataloger triggered

During your **first 50 reviews** the trail auto-expands on every card so you can build intuition for when to trust the engine. After that, the trail collapses to on-demand on Unanimous cards and stays auto-expanded on Strong majority + Split cards (where the disagreement is informative).

You can override this default at any time in Settings → Intelligence → CECE → Reasoning trail visibility:

- **Always show** — open on every card
- **On disagreement only (default)** — open on Split + Strong majority cards, plus the first 50 reviews
- **Always hide** — manual click required to expand

---

## The queue composition filter

Above the count strip there are **five chips** that slice your queue by the kind of decision each card needs from you:

| Chip | Shows |
|---|---|
| **All** *(default)* | the full queue |
| **Both axes need your call** | cards where BOTH horizontal AND vertical are Split |
| **Source needs your call** | cards where horizontal is Split + vertical is settled |
| **Content type needs your call** | cards where vertical is Split + horizontal is settled |
| **Catalogers agreed** | cards where neither axis is Split — quick rubber-stamp candidates |

Each chip shows its bucket count (e.g. *"Source needs your call (43)"*). Empty buckets are dimmed and disabled. Clicking a chip re-renders the visible cards; the count strip and Approve All math always operate on the **full** queue regardless of the active filter, so you can always see the true totals.

The filter solves the needle-in-haystack problem when your queue has hundreds of cards. Want to clear all rubber-stamp candidates first? Click **Catalogers agreed** then click **Approve all**. Want to focus on the hardest cases? Click **Both axes need your call**.

---

## The note summary under each card

Under each card's title sits a short **Summary** line — a few sentences that tell you what the note is about, so you can decide how to classify it without opening it. Constellation always shows a summary *you* wrote (a `summary:` / `description:` / `abstract:` / `excerpt:` frontmatter field, or a `> [!summary]` / `[!abstract]` / `[!tldr]` callout in the body) and only generates one when you haven't. Generated summaries are extractive — the note's own most-central sentences — and are never written back into your file. Full detail is in the **Note Summaries** topic.

---

## Per-card actions

Every card has four actions at the bottom (or three on Split cards where Disambig replaces Accept/Edit):

- **Accept** — write the engine's synthesis primary on both axes to the note's frontmatter, remove the card from the queue. Updates per-cataloger reliability.
- **Edit** — open a tree picker for both axes; you choose values manually. Same reliability update.
- **Reject** — clear the card without writing anything. The engine will re-suggest if you re-classify later. (Rejection does NOT update reliability — the user "doesn't want any of these" is ambiguous as a feedback signal.)
- **Sibling Disambiguation chip** — on Split cards, click one of the candidate chips. Writes the picked value (and auto-writes the other axis if it was settled).

---

## The trust-calibration period

Your first **50 reviews** of CECE-classified cards are a *trust-calibration period*. During this time the reasoning trail auto-expands on every card (regardless of regime), and a quiet banner at the panel top reminds you: *"Showing reasoning trails until you review N more cards — helps you learn when to trust the catalogers."*

After 50 reviews the banner disappears and trails collapse to the default on-demand behavior. You can override via Settings if you want to keep them always-open or always-closed.

The point of the calibration period: CECE is a probabilistic system that gets better as you correct it (per-Library reliability). Seeing *why* each cataloger voted the way it did during the first 50 reviews lets you build your own intuition for when its conclusions are trustworthy on this specific Library's content.

---

## Per-Library calibration

Settings → Intelligence → CECE → **Per-Library calibration** opens a read-only table showing each cataloger's per-axis accuracy on the active Library:

```
Cataloger          Horizontal      Vertical
─────────          ──────────      ────────
Your frontmatter   12/12 (100%)    4/4 (100%)
Citations          18/22 (82%)     6/8 (75%)
Wordstems          24/28 (86%)     20/26 (77%)
Linked notes       3/4 (uniform)   2/3 (uniform)
Similar notes      14/19 (74%)     12/19 (63%)
AI judgment        — (not running) — (not running)
```

The numbers are correct/total counts. The percentage is shown after a cataloger has 20+ corrections on that Library × axis (the threshold for stable accuracy data). Below the threshold, the label shows **(uniform)** — the cataloger contributes uniformly weighted votes until enough data accumulates.

Different Libraries can have wildly different per-cataloger accuracies. The Linguistic cataloger excels on Arabic-heavy Libraries; the Graph cataloger excels on densely-linked Libraries. The synthesis layer uses the per-Library calibration data to weight votes — so a cataloger that's been wrong 70% of the time in *this* Library has its votes downweighted in the next round of synthesis.

---

## Background classification

The Source Review queue can grow two ways:

1. **Manual** (default) — you right-click a note → "Suggest sources & content type", or you trigger Settings → Run classification scan.
2. **Background** — Settings → Intelligence → CECE → Background classification. Two modes:
   - **On note save** — auto-classify each note ~1.5 seconds after you stop typing (rides the existing debounced save; never fires per-keystroke).
   - **On app start** — scan unclassified notes once per launch.

Background classification is **off by default**. Both background modes run on a background thread + emit progress events; typing stays instant; you can cancel from the Source Review panel header.

---

## Common workflows

**"I just installed CECE — where do I start?"**
Open the Source Review panel. Right-click 5-10 notes from your file tree → "Suggest sources & content type" to seed the queue. Click through the cards one at a time. The reasoning trail auto-expands during your first 50 reviews — read it. After 5-10 cards you'll start to see which catalogers are reliable on your content.

**"My queue has 1,200 cards — where do I focus?"**
Use the filter chips. Start with **Catalogers agreed** (rubber-stamp candidates) → click Approve all to clear them. Then **Source needs your call** + **Content type needs your call** for Split cases that need one decision each. **Both axes need your call** is the hardest set; save it for last.

**"How do I know when to Accept vs Reject vs Edit vs Disambig?"**
- **Accept** when the synthesis primary matches your read of the note.
- **Reject** when none of the suggestions fit (e.g. the engine missed something you know about the note).
- **Edit** when you want a value not in any of the suggestions.
- **Sibling Disambiguation chip** when the card is Split and one of the candidates is correct.

**"How do I see which catalogers I trust most?"**
Open Settings → Intelligence → CECE → Per-Library calibration. The table shows per-cataloger accuracy across the corrections you've made on this Library.

---

## Related topics

- **The Cataloger** — the full-page home for these cards, with a note-picker ("Classify a note…") and a "Build all summaries" button.
- **Note Summaries** — how the Summary line under each card is produced, and the author-first precedence that always prefers your own words.
- **Cognitive Engine** — the broader knowledge formulation philosophy CECE fits into.
- **Properties** — the `sources:` and `content_type:` frontmatter fields CECE writes to.
- **Knowledge Hierarchy** — how Source × Content Type fits into the Universe / Library / Folder / Note structure.


---

## Right sidebar vs the Cataloger — two distinct places

The right-sidebar **Source Review** tab and the full-page **Cataloger** are now **distinct surfaces**:

- **Right sidebar → Source Review** shows the pending suggestion for the **note you have open** — its own card and its per-card *Accept / Edit / Reject*.
- **The Cataloger** (stacked-cards icon, left dock) shows the **whole-universe** review queue — every note awaiting a decision — together with the bulk **Approve all / Reject all** tools and the filter chips. Those bulk tools live *only* here, never beside a single note.

You can resize the Cataloger's text in **Style Setter → Cataloger → Text size**.

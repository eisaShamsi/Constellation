# Constellation NSC — Subsystem Concept Paper

**Version 1.0 | 2026-05-20**

> **Purpose**: Define the **Note Summary Creator (NSC)** — a Constellation subsystem that gives every note a short, faithful summary so the user can grasp what a note is about without opening it. Born from Eisa's 2026-05-20 direction during the MIG-039 (Cataloger) work: *"design and build a Note Summary Creator that first checks whether the note has a summary and uses it; if not, it summarizes the whole note. Of course, it should be able to handle any language."* Method constraint (Eisa): **use a proven standard method**, not an invented one.

---

## §1 — The core concept (one sentence)

**NSC produces, for any note, a 2–3 sentence summary — using the note's own summary field if the author wrote one, otherwise generating one with a proven, language-agnostic extractive algorithm — and surfaces it in the Cataloger / Source Review card.**

## §2 — Why NSC exists

The Cataloger (CECE's left-dock home, MIG-039) shows a queue of notes to review. To act on a card, the user needs to know *what the note is about*. Opening each note breaks the review flow. NSC closes that gap: a glance-level summary under the note title. More broadly, a faithful per-note summary is a reusable knowledge primitive (search previews, hover cards, future surfaces) — Eisa frames it as a Constellation differentiator.

## §3 — The method (proven standard, decided)

**Extractive, embedding-based TextRank.** (Eisa, 2026-05-20: extractive now; abstractive AI-rewrite is a future upgrade once the local LLM is wired.)

1. **Frontmatter precedence** — if the note's YAML frontmatter has a `summary` / `description` / `abstract` / `excerpt` field, NSC returns it verbatim (`source = "frontmatter"`). The author's words win (File-Over-App).
2. **Sentence segmentation** — split the body with the **Unicode Text Segmentation standard, UAX #29** (`unicode-segmentation` crate's sentence-boundary iterator). This is the cross-language standard; it handles CJK, Arabic, and Indic scripts. **Fallback** for scripts without sentence punctuation (Thai, Lao): if segmentation yields too few units for the text length, split on paragraph/line breaks; if still one block, return the opening portion (`source = "opening"`). NSC never fails on any language.
3. **Ranking** — embed each sentence with the existing **multilingual e5-small** ONNX model (`embeddings::run_embedding_batch`, "query: " prefix), build a cosine-similarity graph (vectors are L2-normalized → cosine = dot product), and run **weighted PageRank** (the TextRank formulation — Mihalcea & Tarau 2004) to score sentence centrality. Return the top-k (k≈3) sentences **in original document order** (`source = "extractive"`).

**Why this is the standard, not an invention.** TextRank (single-document) + PageRank is the canonical unsupervised extractive summarizer; LexRank is its multi-document sibling. The modern, documented variant replaces classic word-overlap similarity with sentence-embedding cosine similarity — which is what reusing e5-small gives us. The approach has published results across English, Arabic (TAC MultiLing), Hindi, Urdu, Gujarati, Bengali, and CJK. UAX #29 is Unicode's own standard for sentence boundaries.

## §4 — Storage & performance (write-time derivation, Rule 8)

- **Cache table** `note_summaries (path PK, summary TEXT, source TEXT, content_hash TEXT, updated_at INTEGER)`. `content_hash` (over `note_meta.body_text`) drives invalidation: stale when the note's body changes.
- **Off the hot path.** Summarization (sentence embedding) is NOT done in `index_note`'s save transaction. A **deferred background worker** (mirroring the existing `constellation_embed_notes` embedding pipeline) computes summaries for notes whose cache is missing/stale, in batches, off the keystroke/save path. Boot back-fill is deferred (after paint), resumable, with status-bar progress.
- **Delivery without lag.** `sources_list_pending_suggestions` LEFT JOINs `note_summaries`, so each card's summary arrives *with* the queue — zero per-card IPC on render (the lesson from the MIG-039 leak). An on-demand `nsc_get_summary(note_path)` IPC covers refresh / cache-miss.

## §5 — Architectural invariants

1. **File-Over-App** — NSC is **read-only** on notes. It never writes a generated summary into the note file. The cache is a rebuildable derived view.
2. **Author authority** — a frontmatter summary always overrides the generated one.
3. **No hot-path heavy work** — summarization runs in a deferred worker; reads are cheap cache lookups (Rule 3 + Rule 8).
4. **Local-only** — all inference on-device (e5-small ONNX); zero cloud path.
5. **Language-agnostic** — UAX #29 segmentation + multilingual embeddings + graceful fallback; works for every language, degrading to opening-text only for punctuation-less scripts.
6. **Faithful** — extractive output is composed of the note's own sentences; it never invents text.

## §6 — IPC surface

- `nsc_get_summary(note_path) -> { summary, source }` — get-or-compute (cache-first).
- `nsc_backfill_start` / `_status` / `_cancel` — the deferred worker controls (mirrors `classifier_scan_*`).
- `SuggestionRecord` gains `summary` + `summary_source` (optional, serde-default — old rows deserialize).

## §7 — Place among subsystems

NSC depends on the **embedding engine** (`embeddings.rs`, shared with CECE's Semantic cataloger + semantic search) and the **frontmatter parser** (`search::parse_frontmatter`) + `note_meta.body_text`. The **Cataloger / Source Review** card consumes it. Nothing depends on NSC being mounted; it runs (or is dormant) independently.

## §8 — Future workstreams

- **Abstractive upgrade** — when the local LLM (Qwen via llama.cpp, designed in CECE §11) is wired, add an abstractive mode (fluent rewrite) behind a setting; the card UI is unchanged. Decided by Eisa as a roadmap item, not v1.
- **Reuse note embeddings** — the per-sentence embeddings could feed back into semantic search / kNN exemplars; out of v1 scope.
- **Summary length / count setting** — a Settings control for k (sentences) if users want longer/shorter summaries.

---

*Concept Paper v1.0, cut 2026-05-20. Companion to the MIG-039 Cataloger work (`Constellation-CECE-Concept-Paper-v1.0.md`) and the session log `lab/reports/SESSION-LOG-2026-05-20.md`. Method grounded in TextRank/LexRank (graph-based extractive summarization) + Unicode UAX #29 (sentence segmentation), per Eisa's "use a proven standard method" directive.*

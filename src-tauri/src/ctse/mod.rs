//! Constellation Terms Scanning Engine — Bridge Adapter (MIG-013).
//!
//! Two surfaces:
//!
//! - [`hooks`] — write-time `term_vocab` ledger maintenance. Every
//!   note save calls `hooks::on_note_indexed` to apply a signed
//!   per-term delta against the shadow vocabulary table. Delete-path
//!   uses `hooks::on_note_deleted`. Pure local bookkeeping; no ONNX,
//!   no concept resolution.
//!
//! - [`search`] — query-time concept expansion. The `ctse_search_by_concept`
//!   Tauri command embeds the user query, finds the top-K nearest
//!   M11 concepts via cosine k-NN against the baked
//!   `bridge_vectors` matrix, expands each concept to its
//!   multilingual lemmas via an in-memory `concept_id → [lemmas]`
//!   map (built once at boot from `LexiconGraph`), and runs an
//!   FTS5 OR-clause MATCH against `notes_fts`. Cross-language
//!   search "for free" — typing "knowledge" surfaces "معرفة"
//!   notes because both lemmas live under the same M11 concept.
//!
//! ## Why query-time, not document-side concept tagging
//!
//! Earlier MIG-013 §1D drafts pre-computed
//! `term_vocab.bridge_concept_id` for every user term (eager
//! tagging) plus a slow-path ONNX backfill for terms M11 didn't
//! cover. Boss-test surfaced that an encyclopedic 7,639-note
//! library produced 5.7M `term_vocab` rows (50K stems + 5.68M
//! bigrams) and that the slow-path projected at multiple hours of
//! ONNX inference. The dominant industry pattern — Lucene
//! `SynonymGraphFilter`, SQLite FTS5 Method 2, CLIR query
//! translation, Primo controlled-vocabulary expansion — runs
//! synonym/concept expansion at query time, not at index time.
//! Constellation now does the same. (The `bridge_concept_id`
//! column the eager-tagging draft added was inert dead schema,
//! never read; MIG-042 dropped it.)
//!
//! ## M11 zero-touch
//!
//! Both surfaces consume `LexiconGraph` and `bridge_vectors`
//! read-only. `lexicon/` source files have a zero-line diff at
//! every CTSE commit (verified mechanically by
//! `git diff src-tauri/src/lexicon/` returning empty).

pub mod hooks;
pub mod search;

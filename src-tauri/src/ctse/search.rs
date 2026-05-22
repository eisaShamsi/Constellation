//! CTSE concept-based term lookup for the Index panel filter
//! (MIG-013 §1D, query-time expansion architecture).
//!
//! ## What it does
//!
//! `ctse_search_terms_by_concept` is the read path for the Index
//! panel filter's `≈ similar` row in the dropdown. It replaces the
//! retired MIG-012 `search_terms_semantic` (which ran cosine over a
//! per-library `term_embeddings` table that took hours to build).
//!
//! Flow:
//!
//! 1. Embed the user's filter query once via multilingual-e5-small.
//! 2. Cosine k-NN against the baked 20K M11 concept-vector matrix
//!    yields the top-K nearest concept IDs (each `c:…`) with scores.
//! 3. For each surviving concept ID we expand it back to its lemmas
//!    in every language M11 covers, using a process-wide
//!    `concept_id → [lemma]` index built once at boot from
//!    `LexiconGraph`. So `c:knowledge` expands to `["knowledge",
//!    "معرفة", "wissen", "savoir", …]`.
//! 4. Each lemma is tokenized through `fts5_tokenizer::tokenize_to_vec`
//!    so the resulting stems live in the same namespace as
//!    `term_vocab.term`. (E.g., the lemma "knowledge" → stem
//!    "knowledg"; "معرفة" → stem "معرف".)
//! 5. We look up which of those stems actually exist in the user's
//!    `term_vocab` (a single SQL `WHERE term IN (...)`). Only stems
//!    the user's library has are returned.
//! 6. Each returned term carries the cosine score of the highest
//!    concept that produced it, so the IndexPanel filter dropdown
//!    can sort by relevance and render the badge.
//!
//! ## Why query-time expansion (not pre-computed `bridge_concept_id`)
//!
//! Mature search systems uniformly do **query-time concept/synonym
//! expansion**, not document-side concept tagging:
//!
//! - **Lucene/Elasticsearch**: `SynonymGraphFilter` (2017+) is
//!   recommended at query time over the deprecated index-time
//!   `SynonymFilter`.
//! - **SQLite FTS5**: documented "Method 2 — query-time synonym
//!   expansion" is one of the three core synonym strategies.
//! - **CLIR (Cross-Language Information Retrieval)**: query
//!   translation is the canonical technique. The query is short and
//!   dynamic; the corpus is large and static.
//! - **Library platforms (Primo, Ex Libris)**: controlled-vocabulary
//!   expansion runs at query time so the index doesn't need to be
//!   re-tagged when the vocabulary changes.
//!
//! Constellation's CTSE follows the same pattern. (MIG-042 dropped the
//! `term_vocab.bridge_concept_id` column that an earlier eager-tagging
//! draft used — it was inert dead schema, never read.) Adding a new M11
//! release picks up automatically on the next query.
//!
//! ## Cost profile
//!
//! - Boot: build `concept_id → lemmas` map once via a single pass
//!   over `LexiconGraph::nodes`. ~10 ms on M11's 200K-node graph,
//!   ~5 MB resident. Cached forever via `OnceLock`.
//! - Per query: ~50 ms e5 inference + ~5 ms cosine k-NN + sub-ms
//!   in-memory expansion + sub-ms SQL `term IN (...)` lookup.
//! - Frontend MUST debounce ≥300 ms (CLAUDE.md Rule 3).

use rusqlite::params_from_iter;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use tauri::Manager;

use crate::bridge_vectors;
use crate::embeddings;
use crate::lexicon::LexiconGraph;
use crate::search::SearchState;

/// Top-K nearest concept count. Each concept expands to ~10–15
/// lemmas (all the languages M11 covers), so the candidate stem
/// pool is roughly K × 12 entries before filtering by the user's
/// own vocabulary. K=10 is comfortable for the IndexPanel filter
/// dropdown without flooding it with noise.
const CONCEPT_TOP_K: usize = 10;

/// Minimum cosine score for a concept to be included. Below this
/// the concept is "too far" from the query and would dilute the
/// `≈ similar` annotations with semantic noise.
const DEFAULT_MIN_SCORE: f32 = 0.55;

/// One result row of [`ctse_search_terms_by_concept`]. The shape
/// matches the retired MIG-012 `TermSimilarity` so the IndexPanel
/// filter UX is a drop-in (renderer, sort order, dedupe by `term`).
#[derive(Debug, Clone, Serialize)]
pub struct CtseTermSimilarity {
    /// The stem as stored in `term_vocab.term` — same namespace the
    /// FTS5 tokenizer emits, so it matches IndexPanel `entry.term`
    /// exactly without further normalization.
    pub term: String,
    /// Cosine score of the highest M11 concept that brought this
    /// term into the result set. In `[min_score, 1.0]`.
    pub score: f32,
}

/// Process-wide `concept_id → [lemma]` reverse index. Built lazily
/// on first call by walking `LexiconGraph::nodes` once. Each lemma
/// appears once per concept it belongs to (multilingual coverage —
/// "knowledge" and "معرفة" both land under `c:knowledge`).
fn concept_lemmas() -> &'static HashMap<String, Vec<String>> {
    static CACHE: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let graph = LexiconGraph::get();
        let mut map: HashMap<String, Vec<String>> = HashMap::with_capacity(20_000);
        for node in &graph.nodes {
            map.entry(node.concept_id.clone())
                .or_default()
                .push(node.lemma.clone());
        }
        for lemmas in map.values_mut() {
            lemmas.sort();
            lemmas.dedup();
        }
        map
    })
}

/// Process-wide stopword set used by the FTS5 tokenizer. Cached
/// once because a typical query expansion runs the tokenizer over
/// 100+ lemmas per call.
fn stopwords_cached() -> &'static HashSet<String> {
    static SW: OnceLock<HashSet<String>> = OnceLock::new();
    SW.get_or_init(crate::libraries::build_stopwords)
}

/// Embed the query, find top-K M11 concepts, expand to multilingual
/// lemmas, tokenize each lemma into the FTS5 stem namespace, and
/// return the subset that exists in the user's `term_vocab` —
/// i.e., terms the user's library actually contains.
///
/// Returns `Ok(Vec::new())` on empty/whitespace queries, on no
/// concept above the score threshold, on no lemmas (vanishingly
/// rare), and on no stems matching `term_vocab` (the user's
/// library has no vocabulary in any of the surviving concepts).
/// The caller (IndexPanel) renders all four the same: no
/// `≈ similar` annotations.
///
/// Per-keystroke callers MUST debounce (≥300 ms — CLAUDE.md Rule 3).
#[tauri::command]
pub fn ctse_search_terms_by_concept(
    app: tauri::AppHandle,
    query: String,
    top_k: Option<u32>,
    min_score: Option<f32>,
) -> Result<Vec<CtseTermSimilarity>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let top_k = top_k.unwrap_or(CONCEPT_TOP_K as u32).min(50) as usize;
    let min_score = min_score.unwrap_or(DEFAULT_MIN_SCORE);

    // Step 1 — embed the query (auto-inits engine on first call).
    let query_vec = embeddings::constellation_embed_text(app.clone(), q.to_string())?;

    // Step 2 — top-K concepts above threshold, with their scores.
    let store = bridge_vectors::get();
    let top_concepts: Vec<(String, f32)> = store
        .nearest_concepts_k(&query_vec, top_k)
        .into_iter()
        .filter(|(_, score)| *score >= min_score)
        .filter_map(|(row, score)| {
            store.concept_id(row).map(|id| (id.to_string(), score))
        })
        .collect();
    if top_concepts.is_empty() {
        return Ok(Vec::new());
    }

    // Step 3 — expand each concept ID to its multilingual lemmas
    // and track the highest concept score per lemma.
    let map = concept_lemmas();
    let mut lemma_scores: HashMap<String, f32> = HashMap::new();
    for (cid, score) in &top_concepts {
        if let Some(lemmas) = map.get(cid) {
            for lemma in lemmas {
                let entry = lemma_scores.entry(lemma.clone()).or_insert(0.0);
                if *score > *entry {
                    *entry = *score;
                }
            }
        }
    }
    if lemma_scores.is_empty() {
        return Ok(Vec::new());
    }

    // Step 4 — tokenize each lemma through the FTS5 tokenizer to
    // get the stem(s) `term_vocab` actually stores. Skip bigrams
    // (joined by `BIGRAM_SEP`) — they aren't searchable as standalone
    // Index terms and the user wouldn't expect them in the dropdown.
    let stopwords = stopwords_cached();
    let mut stem_scores: HashMap<String, f32> = HashMap::new();
    for (lemma, score) in &lemma_scores {
        let tokens = crate::fts5_tokenizer::tokenize_to_vec(lemma, stopwords);
        for token in tokens {
            if token.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP) {
                continue;
            }
            let entry = stem_scores.entry(token).or_insert(0.0);
            if *score > *entry {
                *entry = *score;
            }
        }
    }
    if stem_scores.is_empty() {
        return Ok(Vec::new());
    }

    // Step 5 — filter to stems the user's `term_vocab` actually has.
    // Single SQL with parameterized IN (...). Caps the cardinality
    // at 250 to keep the parameter count well below SQLite's 999
    // default.
    const STEM_CAP: usize = 250;
    let stems: Vec<String> = stem_scores.keys().take(STEM_CAP).cloned().collect();
    let placeholders = (1..=stems.len())
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT term FROM term_vocab WHERE term IN ({})", placeholders);

    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(stems.iter()), |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let existing: HashSet<String> = rows.filter_map(|r| r.ok()).collect();

    // Step 6 — join scores back to existing stems, sort descending.
    let mut results: Vec<CtseTermSimilarity> = stem_scores
        .into_iter()
        .filter(|(stem, _)| existing.contains(stem))
        .map(|(term, score)| CtseTermSimilarity { term, score })
        .collect();
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity: the lazy `concept_id → lemmas` map covers a known
    /// English lemma and a non-Latin equivalent under the same
    /// concept ID. This is the cross-language coverage the read
    /// path depends on.
    #[test]
    fn concept_lemmas_includes_book_in_multiple_languages() {
        let map = concept_lemmas();
        assert!(!map.is_empty(), "concept_lemmas map must not be empty");
        let book_concept = map
            .iter()
            .find(|(_, lemmas)| lemmas.iter().any(|l| l == "book"));
        assert!(
            book_concept.is_some(),
            "M11 should have a concept whose lemmas include 'book'"
        );
        let (_, lemmas) = book_concept.unwrap();
        let has_non_latin = lemmas
            .iter()
            .any(|l| l.chars().any(|c| !c.is_ascii() && c.is_alphabetic()));
        assert!(
            has_non_latin,
            "expected at least one non-ASCII lemma alongside 'book'; got {lemmas:?}",
        );
    }
}

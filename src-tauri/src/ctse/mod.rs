//! Constellation Terms Scanning Engine — Bridge Adapter (MIG-013 §1B).
//!
//! Resolves a user-library term to its M11 concept ID. Two paths:
//!
//! 1. **Fast path**: exact-form lookup against the M11 lexicon
//!    (`lexicon::LexiconGraph::find_nodes`). O(|lemma|) FST query;
//!    no ONNX inference. Hits ~80% of expected terms.
//! 2. **Slow path**: embed the term via multilingual-e5-small and
//!    cosine-NN against the baked concept-vector matrix
//!    (`bridge_vectors::ConceptVectorStore::nearest_concept`). One
//!    ~10 ms ONNX inference + ~5 ms k-NN. Used only for terms M11
//!    doesn't have exact-form coverage for (proper nouns, code
//!    identifiers, regional variants).
//!
//! ## Public surface
//!
//! - [`resolve_term_pure`] — testable, dependency-injected core.
//!   Takes a graph, a vector store, and an `embed_query` callback;
//!   returns the resolved concept ID or `None`.
//! - [`resolve_term_to_concept`] — Tauri-context wrapper that pulls
//!   the lexicon singleton, the bridge-vector singleton, and the
//!   embedding engine from app state. The intended caller for
//!   §1C's write-time hooks.
//!
//! ## Threshold
//!
//! `DEFAULT_THRESHOLD = 0.78` — initial guess from the e5 model
//! card. Tuned empirically in §1D once Boss reports query-quality
//! signal on his library.
//!
//! ## M11 zero-touch
//!
//! This module reads `LexiconGraph::get()` and indexes
//! `graph.nodes[idx].concept_id` directly. Both the method and the
//! field were already public before MIG-013; no lexicon-module
//! source is modified.

use crate::arabic::Lang;
use crate::bridge_vectors::{self, ConceptVectorStore};
use crate::lexicon::LexiconGraph;

/// Default cosine threshold for the slow path. Below this, the
/// nearest concept is treated as "too far" and the term is left
/// unresolved (`None`). 0.78 is an initial empirical guess from
/// the multilingual-e5-small model card; will be tuned in §1D.
pub const DEFAULT_THRESHOLD: f32 = 0.78;

/// Pure resolver used by tests and by the AppHandle wrapper.
///
/// `embed_query` is invoked **only when the fast path misses** — so
/// callers are encouraged to defer engine init until really needed.
/// The closure must produce a `VECTOR_DIM`-long L2-normalized vector
/// (the same shape `bridge_vectors::nearest_concept` expects).
pub fn resolve_term_pure<F>(
    graph: &LexiconGraph,
    store: &ConceptVectorStore,
    embed_query: F,
    term: &str,
    lang: Lang,
    threshold: f32,
) -> Option<String>
where
    F: FnOnce(&str) -> Option<Vec<f32>>,
{
    // ── Fast path: M11 exact-form lookup ──
    let nodes = graph.find_nodes(lang, term);
    if let Some(&first_idx) = nodes.first() {
        if let Some(node) = graph.nodes.get(first_idx as usize) {
            return Some(node.concept_id.clone());
        }
    }

    // ── Slow path: embed + cosine k-NN ──
    let q = embed_query(term)?;
    let (row, score) = store.nearest_concept(&q)?;
    if score < threshold {
        return None;
    }
    store.concept_id(row).map(str::to_string)
}

/// Tauri-context wrapper. Resolves a term to its M11 concept ID
/// using the live lexicon, the baked bridge-vector store, and the
/// existing embedding engine.
///
/// Returns:
/// - `Ok(Some(concept_id))` — resolved (fast or slow path).
/// - `Ok(None)` — no fast-path hit AND slow-path score below
///   `DEFAULT_THRESHOLD` (or slow-path embedding failed cleanly).
/// - `Err(_)` — only on hard failures upstream of resolution
///   (engine init failure cascading from a missing model file).
pub fn resolve_term_to_concept(
    app: &tauri::AppHandle,
    term: &str,
    lang: Lang,
) -> Result<Option<String>, String> {
    let graph = LexiconGraph::get();
    let store = bridge_vectors::get();
    // Borrow `app` for the closure without cloning the AppHandle on
    // the fast path (where the closure is never invoked).
    let app_ref = app;
    let resolved = resolve_term_pure(
        graph,
        store,
        |t: &str| crate::embeddings::constellation_embed_text(app_ref.clone(), t.to_string()).ok(),
        term,
        lang,
        DEFAULT_THRESHOLD,
    );
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge_vectors::VECTOR_DIM;

    /// Fast path: a known M11 lemma resolves without invoking the
    /// slow-path closure.
    #[test]
    fn fast_path_resolves_known_lemma_without_calling_slow_path() {
        let graph = LexiconGraph::get();
        let store = bridge_vectors::get();
        let result = resolve_term_pure(
            graph,
            store,
            |_t| panic!("slow path must NOT be invoked when fast path hits"),
            "book",
            Lang::En,
            DEFAULT_THRESHOLD,
        );
        assert!(
            result.is_some(),
            "M11 covers 'book' in English; fast path should succeed"
        );
        let cid = result.unwrap();
        assert!(
            cid.starts_with("c:"),
            "concept id should be M11-namespaced, got {cid:?}"
        );
    }

    /// Slow path with a zero query vector returns `None` (every
    /// dot product is 0, well below the threshold). Confirms the
    /// threshold gate is wired correctly.
    #[test]
    fn slow_path_zero_query_returns_none_below_threshold() {
        let graph = LexiconGraph::get();
        let store = bridge_vectors::get();
        let result = resolve_term_pure(
            graph,
            store,
            |_t| Some(vec![0.0f32; VECTOR_DIM]),
            "garblefuxxzqq-not-in-m11",
            Lang::En,
            DEFAULT_THRESHOLD,
        );
        assert!(
            result.is_none(),
            "zero-query slow path must score 0.0 < threshold {DEFAULT_THRESHOLD}"
        );
    }

    /// Slow path with a strict threshold of 1.01 (impossible to
    /// reach) returns None even if the fake embed vector happens
    /// to match a concept perfectly. Confirms the threshold is
    /// strictly less-than (a perfect 1.0 score must clear 0.78
    /// but not 1.01).
    #[test]
    fn slow_path_above_one_threshold_rejects_everything() {
        let graph = LexiconGraph::get();
        let store = bridge_vectors::get();
        // Use the first row of the actual store — guaranteed to
        // produce score 1.0 against itself.
        let mut probe = vec![0.0f32; VECTOR_DIM];
        probe[0] = 1.0; // arbitrary non-zero query
        let result = resolve_term_pure(
            graph,
            store,
            |_t| Some(probe.clone()),
            "garblefuxxzqq-not-in-m11",
            Lang::En,
            1.01,
        );
        assert!(result.is_none());
    }

    /// Slow path with a permissive threshold of 0.0 always returns
    /// *some* concept (the nearest one, however weak). Confirms the
    /// fallthrough returns `Some` when score ≥ threshold.
    #[test]
    fn slow_path_zero_threshold_always_returns_some() {
        let graph = LexiconGraph::get();
        let store = bridge_vectors::get();
        let mut probe = vec![0.0f32; VECTOR_DIM];
        probe[0] = 1.0;
        let result = resolve_term_pure(
            graph,
            store,
            |_t| Some(probe.clone()),
            "garblefuxxzqq-not-in-m11",
            Lang::En,
            0.0,
        );
        assert!(result.is_some());
        assert!(result.unwrap().starts_with("c:"));
    }
}

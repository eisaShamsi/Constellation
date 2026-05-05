//! In-memory concept-vector store with cosine k-NN.
//!
//! Holds the parsed asset (concept IDs + flat row-major f32 matrix)
//! and answers nearest-neighbor queries. Inputs and stored vectors
//! are L2-normalized, so cosine similarity = dot product.
//!
//! Performance: the inner loop is a tight dot product over `dim`
//! contiguous f32s, repeated `count` times. For the M11 corpus
//! (count = 20K, dim = 384) this is ~7.7M multiply-adds per query
//! — ~5 ms on a modern CPU, easily within the keystroke budget when
//! search is debounced (≥300 ms).

use crate::bridge_vectors::VECTOR_DIM;

/// Owned in-memory store of concept vectors.
pub struct ConceptVectorStore {
    /// Concept IDs in matrix-row order (`ids.len() == count`).
    ids: Vec<String>,
    /// Row-major flat matrix: `count * VECTOR_DIM` floats. Stored as
    /// `Box<[f32]>` so it sits on the heap with correct f32 alignment
    /// regardless of how the underlying byte source was aligned.
    matrix: Box<[f32]>,
    count: usize,
}

impl ConceptVectorStore {
    /// Build a store from owned id/matrix data. The caller is responsible
    /// for ensuring `matrix.len() == ids.len() * VECTOR_DIM` and that
    /// every row is L2-normalized — both invariants are written by
    /// `build_concept_vectors` and validated on parse in `asset.rs`.
    pub fn new(ids: Vec<String>, matrix: Box<[f32]>) -> Result<Self, String> {
        let count = ids.len();
        if matrix.len() != count * VECTOR_DIM {
            return Err(format!(
                "matrix length {} != count {} * dim {}",
                matrix.len(), count, VECTOR_DIM
            ));
        }
        Ok(Self { ids, matrix, count })
    }

    /// Number of concepts in the store.
    pub fn count(&self) -> usize { self.count }

    /// Concept ID for a given row (None if out of bounds).
    pub fn concept_id(&self, row: usize) -> Option<&str> {
        self.ids.get(row).map(String::as_str)
    }

    /// Nearest concept by cosine similarity. `query` must be
    /// `VECTOR_DIM`-long and L2-normalized; otherwise the score is
    /// uninterpretable. Returns `(row, score)` or `None` when the
    /// store is empty / `query` is the wrong dim.
    pub fn nearest_concept(&self, query: &[f32]) -> Option<(usize, f32)> {
        if query.len() != VECTOR_DIM || self.count == 0 {
            return None;
        }
        let mut best_row = 0usize;
        let mut best_score = f32::NEG_INFINITY;
        for row in 0..self.count {
            let off = row * VECTOR_DIM;
            let mut dot: f32 = 0.0;
            // Indexed loop; bounds-checks elide because both bounds are
            // const w.r.t. the loop. LLVM auto-vectorizes this on x86/arm.
            for j in 0..VECTOR_DIM {
                dot += query[j] * self.matrix[off + j];
            }
            if dot > best_score {
                best_score = dot;
                best_row = row;
            }
        }
        Some((best_row, best_score))
    }

    /// Top-`k` neighbors by cosine similarity, descending score.
    /// Stable: ties keep their first-seen row. Returns up to `k`
    /// entries (fewer if the store has fewer rows).
    pub fn nearest_concepts_k(&self, query: &[f32], k: usize) -> Vec<(usize, f32)> {
        if query.len() != VECTOR_DIM || self.count == 0 || k == 0 {
            return Vec::new();
        }
        // For typical k (≤32), maintaining a small sorted vec beats a
        // BinaryHeap — the linear search is faster than heap ops for
        // small k, and avoids std::collections allocations.
        let cap = k.min(self.count);
        let mut top: Vec<(usize, f32)> = Vec::with_capacity(cap);
        for row in 0..self.count {
            let off = row * VECTOR_DIM;
            let mut dot: f32 = 0.0;
            for j in 0..VECTOR_DIM {
                dot += query[j] * self.matrix[off + j];
            }
            if top.len() < cap {
                top.push((row, dot));
                // Sort descending after each insert until full.
                top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            } else if dot > top[cap - 1].1 {
                top[cap - 1] = (row, dot);
                top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
        }
        top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a tiny synthetic store: identity-ish vectors padded to
    /// `VECTOR_DIM`. Each concept's vector has a 1.0 in slot `i` and
    /// zeros elsewhere. After L2-normalize that's the canonical basis,
    /// so query [1, 0, 0, ...] returns concept 0 with score 1.0.
    fn synthetic_store(n: usize) -> ConceptVectorStore {
        let mut ids = Vec::with_capacity(n);
        let mut matrix = vec![0.0f32; n * VECTOR_DIM];
        for i in 0..n {
            ids.push(format!("c:test-{i}"));
            matrix[i * VECTOR_DIM + i % VECTOR_DIM] = 1.0;
        }
        ConceptVectorStore::new(ids, matrix.into_boxed_slice()).unwrap()
    }

    #[test]
    fn nearest_concept_returns_exact_match_with_score_one() {
        let store = synthetic_store(8);
        let mut query = vec![0.0f32; VECTOR_DIM];
        query[3] = 1.0;
        let (row, score) = store.nearest_concept(&query).expect("non-empty store");
        assert_eq!(row, 3, "query in basis-3 should match concept 3");
        assert!((score - 1.0).abs() < 1e-6, "score {} should be ~1.0", score);
    }

    #[test]
    fn nearest_concept_rejects_wrong_dim() {
        let store = synthetic_store(8);
        let bad = vec![0.0f32; VECTOR_DIM - 1];
        assert!(store.nearest_concept(&bad).is_none());
    }

    #[test]
    fn nearest_concept_zero_query_returns_zero_score() {
        let store = synthetic_store(8);
        let zero = vec![0.0f32; VECTOR_DIM];
        let (_, score) = store.nearest_concept(&zero).expect("non-empty store");
        assert!(score.abs() < 1e-6, "zero query → zero dot product");
    }

    #[test]
    fn top_k_returns_descending_scores() {
        let store = synthetic_store(8);
        // Query that overlaps slot 3 most, then slot 5, then slot 1.
        let mut query = vec![0.0f32; VECTOR_DIM];
        query[3] = 0.9;
        query[5] = 0.5;
        query[1] = 0.2;
        // Normalize for fair cosine.
        let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        for v in query.iter_mut() { *v /= norm; }

        let top = store.nearest_concepts_k(&query, 3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].0, 3);
        assert_eq!(top[1].0, 5);
        assert_eq!(top[2].0, 1);
        assert!(top[0].1 > top[1].1);
        assert!(top[1].1 > top[2].1);
    }

    #[test]
    fn top_k_clamps_to_count() {
        let store = synthetic_store(3);
        let query = vec![0.0f32; VECTOR_DIM];
        let top = store.nearest_concepts_k(&query, 10);
        assert_eq!(top.len(), 3);
    }
}

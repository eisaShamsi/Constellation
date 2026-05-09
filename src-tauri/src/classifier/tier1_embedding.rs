//! MIG-021v2 §1B' — Tier 1 classifier (e5-small embedding-similarity), v2.
//!
//! Pipeline:
//!   1. At first call: build the unified ~275-candidate set
//!      (52 horizontal + 222 vertical), embed each definition via the
//!      existing `multilingual-e5-small` ONNX engine, cache TWO
//!      L2-normalized vector pools (HORIZONTAL_VECTORS + VERTICAL_VECTORS)
//!      keyed by ID.
//!   2. Per classification: embed the note's text once, compute cosine
//!      similarity against BOTH pools, return parallel suggestion sets:
//!      top-3 horizontal + top-3 vertical, each tagged with `axis`.
//!   3. Tier-aware confidence fallback: if top-1 horizontal pick is a
//!      Tier 3 source (or its leaf) with confidence < 0.55, swap with
//!      the highest-scoring Tier 1/2 candidate (Plan §0 Q7 ENABLED).
//!   4. Leaf-vs-parent strategy: if a leaf's confidence < 0.55, suggest
//!      its parent instead (more reliable; user can drill down manually).
//!
//! Bundled tier — same hardware as Constellation core. e5-small (113 MB
//! ONNX) already shipped for semantic search; reused at zero bundle cost.
//! Accuracy expectation: ~75-85% top-1 at PARENT level, ~50-65% top-1 at
//! LEAF level. Tier 2 (Qwen3-1.7B via llama.cpp, §1H') for higher accuracy.

use crate::sources::{horizontal_taxonomy, Suggestion};
use std::sync::OnceLock;
use tauri::Manager;

use super::source_definitions::{build_classifier_candidates, ClassifierAxis};

/// L2-normalized embeddings of the horizontal axis (52 entries: 11
/// parents + 41 leaves; opt-out token excluded).
static HORIZONTAL_VECTORS: OnceLock<Vec<(String, Vec<f32>)>> = OnceLock::new();

/// L2-normalized embeddings of the vertical axis (~218 entries: 5
/// branches + nested sub-nodes; root excluded).
static VERTICAL_VECTORS: OnceLock<Vec<(String, Vec<f32>)>> = OnceLock::new();

/// Top-N suggestions per axis. Per Plan §1B' verification: top-3.
const TOP_N: usize = 3;

/// Tier-aware fallback threshold (Plan §0 Q5 + Q7). Below this confidence,
/// the classifier prefers more-universally-accepted alternatives over
/// Tier 3 picks AND prefers parent-level over leaf-level.
const CONFIDENCE_FALLBACK_THRESHOLD: f32 = 0.55;

/// Maximum text length for embedding (char-boundary safe for UTF-8).
const MAX_TEXT_LEN: usize = 2000;

/// Classify a note's text against BOTH axes. Returns up to TOP_N
/// suggestions per axis (so up to 2*TOP_N total), each tagged with
/// `axis = "horizontal" | "vertical"`. Suggestions sorted within
/// each axis by confidence descending.
pub fn classify(app: &tauri::AppHandle, note_text: &str) -> Result<Vec<Suggestion>, String> {
    let (horizontal_vectors, vertical_vectors) = ensure_vector_pools(app)?;

    let truncated = truncate_to_char_boundary(note_text, MAX_TEXT_LEN);
    let prefixed = format!("query: {}", truncated);
    let note_vec = embed_with_cached_engine(app, &prefixed)?;
    let note_normalized = l2_normalize(&note_vec);

    let horizontal_top = top_n_horizontal(&note_normalized, horizontal_vectors);
    let vertical_top = top_n_for_pool(
        &note_normalized,
        vertical_vectors,
        ClassifierAxis::Vertical,
    );

    let mut combined: Vec<Suggestion> = Vec::with_capacity(2 * TOP_N);
    combined.extend(horizontal_top);
    combined.extend(vertical_top);
    Ok(combined)
}

/// Horizontal axis cosine ranking + tier-aware fallback + leaf-vs-parent
/// adjustment. Returns the top TOP_N as Suggestion records tagged
/// `axis = "horizontal"`.
fn top_n_horizontal(
    note_vec: &[f32],
    pool: &[(String, Vec<f32>)],
) -> Vec<Suggestion> {
    let mut scored: Vec<(String, f32)> = pool
        .iter()
        .map(|(id, vec)| (id.clone(), dot(note_vec, vec)))
        .collect();
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply tier-aware fallback (Plan §0 Q7) if top-1 is Tier 3 below threshold.
    if let Some((top_id, top_score)) = scored.first().cloned() {
        if *top_score_is_tier_three(&top_id) && top_score < CONFIDENCE_FALLBACK_THRESHOLD {
            // Promote the highest-scoring Tier 1 or Tier 2 candidate to top.
            if let Some(idx) = scored.iter().position(|(id, _)| {
                let t = horizontal_taxonomy::effective_tier(id);
                matches!(t, Some(1) | Some(2))
            }) {
                if idx > 0 {
                    let promoted = scored.remove(idx);
                    scored.insert(0, promoted);
                }
            }
        }
    }

    // Apply leaf-vs-parent strategy (Plan §0 Q5): if leaf's confidence
    // < threshold, replace with parent if parent isn't already in top-N.
    let adjusted: Vec<(String, f32)> = scored
        .into_iter()
        .map(|(id, score)| {
            if score < CONFIDENCE_FALLBACK_THRESHOLD {
                if let Some(parent) = horizontal_taxonomy::parent_of(&id) {
                    return (parent.to_string(), score);
                }
            }
            (id, score)
        })
        .collect();

    // Dedupe (the leaf-fallback may have collapsed multiple leaves to one parent)
    let mut seen = std::collections::HashSet::new();
    let mut deduped: Vec<(String, f32)> = Vec::new();
    for (id, score) in adjusted {
        if seen.insert(id.clone()) {
            deduped.push((id, score));
        }
    }

    deduped
        .into_iter()
        .take(TOP_N)
        .map(|(source, score)| {
            let evidence = brief_signature_for(&source).to_string();
            Suggestion {
                source,
                confidence: clamp01(score),
                evidence,
                axis: ClassifierAxis::Horizontal.as_str().to_string(),
            }
        })
        .collect()
}

/// Generic top-N for a vector pool, no axis-specific adjustments. Used
/// for the vertical axis (no tier system; no leaf-vs-parent fallback at
/// this phase — vertical accuracy is BRANCH-level reliable per Plan §6).
fn top_n_for_pool(
    note_vec: &[f32],
    pool: &[(String, Vec<f32>)],
    axis: ClassifierAxis,
) -> Vec<Suggestion> {
    let mut scored: Vec<(String, f32)> = pool
        .iter()
        .map(|(id, vec)| (id.clone(), dot(note_vec, vec)))
        .collect();
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    scored
        .into_iter()
        .take(TOP_N)
        .map(|(id, score)| Suggestion {
            source: id,
            confidence: clamp01(score),
            evidence: String::new(), // vertical axis: no per-source signature yet
            axis: axis.as_str().to_string(),
        })
        .collect()
}

fn top_score_is_tier_three(id: &str) -> &'static bool {
    if matches!(horizontal_taxonomy::effective_tier(id), Some(3)) {
        &true
    } else {
        &false
    }
}

/// Build both vector pools on first call; cache for process lifetime.
fn ensure_vector_pools(
    app: &tauri::AppHandle,
) -> Result<(&'static Vec<(String, Vec<f32>)>, &'static Vec<(String, Vec<f32>)>), String> {
    if HORIZONTAL_VECTORS.get().is_some() && VERTICAL_VECTORS.get().is_some() {
        return Ok((
            HORIZONTAL_VECTORS.get().unwrap(),
            VERTICAL_VECTORS.get().unwrap(),
        ));
    }

    let candidates = build_classifier_candidates();

    let mut horizontal: Vec<(String, Vec<f32>)> = Vec::new();
    let mut vertical: Vec<(String, Vec<f32>)> = Vec::new();

    for c in candidates {
        // e5 convention: definitions are passages, prefix with "passage: ".
        let prefixed = format!("passage: {}", c.embedding_text);
        let vec = embed_with_cached_engine(app, &prefixed)?;
        let normalized = l2_normalize(&vec);
        match c.axis {
            ClassifierAxis::Horizontal => horizontal.push((c.id, normalized)),
            ClassifierAxis::Vertical => vertical.push((c.id, normalized)),
        }
    }

    let _ = HORIZONTAL_VECTORS.set(horizontal);
    let _ = VERTICAL_VECTORS.set(vertical);
    Ok((
        HORIZONTAL_VECTORS.get().ok_or("HORIZONTAL_VECTORS init failed")?,
        VERTICAL_VECTORS.get().ok_or("VERTICAL_VECTORS init failed")?,
    ))
}

/// Embed text via the cached e5-small engine.
fn embed_with_cached_engine(
    app: &tauri::AppHandle,
    text: &str,
) -> Result<Vec<f32>, String> {
    crate::embeddings::ensure_engine(app)?;
    let state = app.state::<crate::embeddings::EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard
        .as_ref()
        .ok_or("Embedding engine not initialized after ensure_engine")?;
    crate::embeddings::run_embedding(engine, text)
}

// ─── Math helpers (unchanged from §1B) ──────────────────────────────

fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn clamp01(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

fn truncate_to_char_boundary(text: &str, max_len: usize) -> &str {
    if text.len() <= max_len {
        return text;
    }
    let mut end = max_len;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Brief signature describing a horizontal source — used as the evidence
/// string in suggestions. Tier-2 will produce richer per-classification
/// evidence by quoting the actual textual cue; Tier-1 uses this generic
/// per-parent fallback. Sub-leaves inherit their parent's signature.
fn brief_signature_for(source: &str) -> &'static str {
    let parent = horizontal_taxonomy::parent_of(source).unwrap_or(source);
    match parent {
        "perception" => "First-hand sensory observation",
        "inference" => "Reasoning from premises to conclusion",
        "testimony" => "Reported by another knower",
        "mass-transmission" => "Convergent multi-witness consensus (al-tawatur)",
        "comparison" => "Knowledge by analogy / qiyas / upamana",
        "postulation" => "Inference to the best explanation (arthapatti)",
        "non-apprehension" => "Knowledge of absence (anupalabdhi)",
        "memory" => "Recall of past experience (smrti)",
        "innate-disposition" => "Pre-experiential intuition (fitrah / nous)",
        "inspiration" => "Mystical or creative apprehension (al-ilham)",
        "revelation" => "Sacred-text or prophetic transmission (al-wahy)",
        _ => "",
    }
}

/// Test-only: expose vector counts for pool-population assertions.
#[doc(hidden)]
pub fn pool_counts_for_test() -> Option<(usize, usize)> {
    Some((
        HORIZONTAL_VECTORS.get()?.len(),
        VERTICAL_VECTORS.get()?.len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l2_normalize_produces_unit_vector() {
        let v = vec![3.0, 4.0];
        let n = l2_normalize(&v);
        let mag: f32 = n.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dot_product_correct() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((dot(&a, &b) - 1.0).abs() < 1e-6);
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(dot(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn clamp01_bounds() {
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(-0.1), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(clamp01(f32::NAN), 0.0);
    }

    #[test]
    fn brief_signature_walks_to_parent() {
        // Leaves inherit parent's signature
        assert_eq!(
            brief_signature_for("perception/external"),
            brief_signature_for("perception")
        );
        assert_eq!(
            brief_signature_for("revelation/recited"),
            brief_signature_for("revelation")
        );
    }

    #[test]
    fn brief_signature_covers_all_classifiable() {
        for id in crate::sources::classifiable_sources() {
            let sig = brief_signature_for(id);
            assert!(!sig.is_empty(), "missing signature for {}", id);
        }
    }
}

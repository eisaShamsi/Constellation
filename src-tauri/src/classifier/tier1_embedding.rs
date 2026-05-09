//! MIG-021 §1B — Tier 1 classifier: e5-small embedding-similarity.
//!
//! Pipeline:
//!   1. At first call: embed each of the 11 source definitions via
//!      the existing `multilingual-e5-small` ONNX engine, cache the
//!      11 × 384-dim L2-normalized vectors in a `OnceLock`.
//!   2. Per classification: embed the note's text, compute cosine
//!      similarity against each cached source vector, return top-N
//!      sorted by similarity descending.
//!
//! Bundled tier — works on the same hardware as Constellation core
//! (e5-small is already shipped at 113 MB for semantic search; reusing
//! it here adds zero bundle cost). Accuracy ~65–75% top-1 on most
//! universes per the §1B verification gate; Tier 2 (Qwen3-1.7B via
//! llama.cpp, §1H) provides ~85–90% for users who download it.

use crate::sources::Suggestion;
use std::sync::OnceLock;
use tauri::Manager;

use super::source_definitions::SOURCE_DEFINITIONS;

/// L2-normalized embeddings of the 11 source definitions, computed
/// once at first classification call and cached for the lifetime of
/// the process. `Vec<(source_id, normalized_vector)>`.
static SOURCE_VECTORS: OnceLock<Vec<(String, Vec<f32>)>> = OnceLock::new();

/// Top-N suggestions to return per classification. Per Plan §1B
/// verification: Tier 1 returns top-3, ordered by cosine similarity.
const TOP_N: usize = 3;

/// Maximum text length passed to the embedding (char-boundary safe
/// for UTF-8). Per Plan §0 Q4 — Tier 1 first 2000 chars.
const MAX_TEXT_LEN: usize = 2000;

/// Classify a note's text against the 11 source definitions.
///
/// Returns top-3 suggestions ordered by cosine similarity descending,
/// each with confidence (the cosine similarity itself, [0..1] for
/// L2-normalized vectors) and an evidence string (currently the
/// matched source's brief signature; richer evidence extraction may
/// arrive in a future revision).
pub fn classify(app: &tauri::AppHandle, note_text: &str) -> Result<Vec<Suggestion>, String> {
    // Ensure source vectors are populated (one-time cost on first call).
    let source_vectors = ensure_source_vectors(app)?;

    // Embed the note text.
    let truncated = truncate_to_char_boundary(note_text, MAX_TEXT_LEN);
    let prefixed = format!("query: {}", truncated);
    let note_vec = embed_with_cached_engine(app, &prefixed)?;
    let note_normalized = l2_normalize(&note_vec);

    // Cosine similarity against each source vector.
    let mut scores: Vec<(String, f32)> = source_vectors
        .iter()
        .map(|(id, vec)| (id.clone(), dot(&note_normalized, vec)))
        .collect();

    // Sort descending by score.
    scores.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top-N, build Suggestion records.
    let suggestions: Vec<Suggestion> = scores
        .into_iter()
        .take(TOP_N)
        .map(|(source, score)| {
            let evidence = brief_signature_for(&source);
            Suggestion {
                source,
                confidence: clamp01(score),
                evidence: evidence.to_string(),
            }
        })
        .collect();

    Ok(suggestions)
}

/// Ensure the 11 source vectors are computed and cached. First call
/// embeds all 11 definitions and L2-normalizes them; subsequent calls
/// are O(1) read from the OnceLock.
fn ensure_source_vectors(app: &tauri::AppHandle) -> Result<&'static Vec<(String, Vec<f32>)>, String> {
    if let Some(v) = SOURCE_VECTORS.get() {
        return Ok(v);
    }

    let mut vectors: Vec<(String, Vec<f32>)> = Vec::with_capacity(SOURCE_DEFINITIONS.len());
    for (id, definition) in SOURCE_DEFINITIONS {
        // e5 convention: definitions are passages, prefix with "passage: ".
        let prefixed = format!("passage: {}", definition);
        let vec = embed_with_cached_engine(app, &prefixed)?;
        vectors.push((id.to_string(), l2_normalize(&vec)));
    }

    // Initialize OnceLock; if another thread beat us to it, use theirs
    // (semantically identical — both threads produce the same vectors).
    let _ = SOURCE_VECTORS.set(vectors);
    SOURCE_VECTORS
        .get()
        .ok_or_else(|| "SOURCE_VECTORS init failed".to_string())
}

/// Embed text via the cached e5-small engine in `EmbeddingState`.
/// Lazy-initializes the engine on first call (existing pattern from
/// `embeddings::ensure_engine`).
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

// ─── Math helpers ──────────────────────────────────────────────────

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

/// Brief signature describing what the source represents — used as the
/// `evidence` string in suggestion records. The frontend may use this
/// in tooltips. Tier 2 (LLM) can produce richer per-classification
/// evidence by quoting the actual textual cue; Tier 1 doesn't have
/// that extraction step, so we fall back to a generic signature.
fn brief_signature_for(source: &str) -> &'static str {
    match source {
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
        _ => "Unknown source",
    }
}

/// Test-only: expose the cached source vectors. Used by integration
/// tests (when they exist) to verify the embedding cache is populated.
#[doc(hidden)]
pub fn get_source_vectors_for_test() -> Option<&'static Vec<(String, Vec<f32>)>> {
    SOURCE_VECTORS.get()
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
    fn l2_normalize_handles_zero_vector() {
        let v = vec![0.0, 0.0, 0.0];
        let n = l2_normalize(&v);
        assert_eq!(n, v);
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
    fn truncate_respects_char_boundary() {
        let s = "héllo";
        let t = truncate_to_char_boundary(s, 3);
        assert!(s.is_char_boundary(t.len()));
    }

    #[test]
    fn brief_signature_covers_all_classifiable() {
        for id in crate::sources::classifiable_sources() {
            let sig = brief_signature_for(id);
            assert_ne!(sig, "Unknown source", "missing signature for {}", id);
        }
    }
}

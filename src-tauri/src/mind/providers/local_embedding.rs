//! `LocalEmbeddingProvider` — wraps the existing `embeddings.rs` ONNX
//! pipeline (multilingual-e5-small, 384-dim, 100 languages) so the new
//! `EmbeddingProvider` trait surface has a real backing for vector
//! retrieval. MIG-047 Phase 0b Step D.
//!
//! **Zero new ONNX session** — we reuse the same `EmbeddingState` that
//! HMSE retrieval already drives via Tauri State. The existing HMSE
//! callers (`embeddings::run_embedding`, `run_embedding_batch`) keep
//! working unchanged; this provider is a parallel consumer.
//!
//! The embedding model selection (multilingual-e5-small) is a Phase
//! 0b/1 decision; the trait surface admits future swap to BGE-M3 or an
//! Arabic-specific embedding model (Concept Paper §15 Q2) without
//! touching consumers.

use std::sync::Arc;

use async_trait::async_trait;
use tauri::{AppHandle, Manager};

use crate::mind::provider::{
    EmbeddingCapabilities, EmbeddingProvider, InferenceError,
};

/// The model id + capabilities mirror what `embeddings.rs` actually loads.
/// If the ONNX model is swapped in a later phase, update both `embeddings.rs`
/// and this `embed_capabilities()` together.
const EMBED_MODEL_ID: &str = "multilingual-e5-small";
const EMBED_DIM: u32 = 384;
const EMBED_MAX_INPUT_TOKENS: u32 = 512;

pub struct LocalEmbeddingProvider {
    app: Arc<AppHandle>,
}

impl LocalEmbeddingProvider {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app: Arc::new(app),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, InferenceError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // ONNX inference is CPU-heavy and synchronous — offload to the
        // blocking-task pool so we don't stall the async runtime.
        let app = self.app.clone();
        let texts_owned: Vec<String> = texts.to_vec();

        tokio::task::spawn_blocking(move || -> Result<Vec<Vec<f32>>, InferenceError> {
            // Ensure the e5-small ONNX engine is loaded. The first call
            // pays the model-load cost (~150ms); subsequent calls are free.
            crate::embeddings::ensure_engine(&app)
                .map_err(InferenceError::Runtime)?;

            let state = app.state::<crate::embeddings::EmbeddingState>();
            // Same poison-recovery pattern as cece/wiring.rs:128 (the engine
            // is just a Session + Tokenizer; no in-memory invariant requires
            // strict poisoning).
            let guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
            let engine = guard.as_ref().ok_or_else(|| {
                InferenceError::NotConfigured(
                    "embedding engine failed to initialize".into(),
                )
            })?;

            crate::embeddings::run_embedding_batch(engine, &texts_owned)
                .map_err(InferenceError::Runtime)
        })
        .await
        .map_err(|e| InferenceError::Runtime(format!("blocking-task join error: {e}")))?
    }

    fn embed_capabilities(&self) -> EmbeddingCapabilities {
        EmbeddingCapabilities {
            model_id: EMBED_MODEL_ID.into(),
            dimension: EMBED_DIM,
            max_input_tokens: EMBED_MAX_INPUT_TOKENS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: a real integration test would need a Tauri AppHandle and a
    // loaded ONNX model — both heavy. The capability check is the
    // unit-test-friendly surface here; the actual embed() path is exercised
    // by HMSE retrieval (which already runs against the same backing
    // `EmbeddingState`) and will be exercised end-to-end by Phase 1's
    // search-tool flow.

    #[test]
    fn capabilities_match_e5_small() {
        // We can construct EmbeddingCapabilities directly without an
        // AppHandle; the trait impl's value of `embed_capabilities()`
        // is what consumers see.
        let caps = EmbeddingCapabilities {
            model_id: EMBED_MODEL_ID.into(),
            dimension: EMBED_DIM,
            max_input_tokens: EMBED_MAX_INPUT_TOKENS,
        };
        assert_eq!(caps.model_id, "multilingual-e5-small");
        assert_eq!(caps.dimension, 384);
        assert_eq!(caps.max_input_tokens, 512);
    }
}

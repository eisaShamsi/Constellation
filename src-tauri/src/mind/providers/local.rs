//! `LocalProvider` — real local inference for Constellation Mind.
//!
//! MIG-047 Phase 0b Step C-v2 (2026-05-24). Wraps `llama-cpp-2` (which
//! wraps upstream llama.cpp), the runtime our model-pipeline workflow
//! uses to quantize Fanar. Replaces the §C-v1 mistral.rs implementation
//! that panicked on Fanar's `gemma2` GGUF architecture.
//!
//! Why llama-cpp-2: the same llama.cpp release (`b6285`) that quantized
//! Fanar to Q4_K_M GGUF in our workflow also reads it back at inference
//! time. Full Gemma-2 GGUF support is upstream. The CECE V3-§7 deferred
//! risk (Windows MSVC cmake/C++ chain) is now eaten in Phase 0b instead
//! of Phase 2.5 (Eisa's Path A choice after the §C-v1 panic).
//!
//! Lazy load: the `LlamaModel` is created on the first `generate()`
//! call via `tokio::sync::OnceCell`. Subsequent calls reuse the loaded
//! model (mmap-backed; ~5 GiB resident for Fanar-1-9B Q4_K_M). The
//! global `LlamaBackend` is process-wide (one per process per llama.cpp
//! upstream contract).

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use tokio::sync::{mpsc, OnceCell};

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};

use crate::mind::events::StreamEvent;
use crate::mind::provider::{
    ChatMessage, ChatRole, FinishReason, GenParams, InferenceError, InferenceProvider,
    ProviderCapabilities, TokenUsage,
};

/// Process-wide llama.cpp backend. Constructed once on first model load
/// and never dropped (llama.cpp expects this).
static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

fn get_backend() -> Result<&'static LlamaBackend, InferenceError> {
    if let Some(b) = BACKEND.get() {
        return Ok(b);
    }
    let b = LlamaBackend::init()
        .map_err(|e| InferenceError::Runtime(format!("LlamaBackend::init: {e}")))?;
    Ok(BACKEND.get_or_init(|| b))
}

pub struct LocalProvider {
    model_path: PathBuf,
    model_id: String,
    /// Lazily-loaded model. Wrapped in `Arc` for cheap cloning into
    /// spawned tasks (the inference task needs to own the model so the
    /// derived `LlamaContext`'s lifetime is bound to a stable owner).
    model: OnceCell<Arc<LlamaModel>>,
}

impl LocalProvider {
    pub fn new(model_path: PathBuf, model_id: impl Into<String>) -> Self {
        Self {
            model_path,
            model_id: model_id.into(),
            model: OnceCell::new(),
        }
    }

    async fn get_model(&self) -> Result<Arc<LlamaModel>, InferenceError> {
        let arc_ref = self
            .model
            .get_or_try_init(|| async {
                let backend = get_backend()?;
                let path = self.model_path.clone();
                // Loading + mmap is sync + heavy — offload to blocking pool.
                let model = tokio::task::spawn_blocking(
                    move || -> Result<LlamaModel, InferenceError> {
                        let params = LlamaModelParams::default();
                        LlamaModel::load_from_file(backend, &path, &params).map_err(|e| {
                            InferenceError::Runtime(format!("load_from_file: {e}"))
                        })
                    },
                )
                .await
                .map_err(|e| InferenceError::Runtime(format!("model-load join: {e}")))??;
                Ok::<_, InferenceError>(Arc::new(model))
            })
            .await?;
        Ok(Arc::clone(arc_ref))
    }
}

#[async_trait]
impl InferenceProvider for LocalProvider {
    async fn generate(
        &self,
        messages: &[ChatMessage],
        params: &GenParams,
    ) -> Result<mpsc::Receiver<StreamEvent>, InferenceError> {
        let model = self.get_model().await?;
        let backend = get_backend()?;

        let prompt = build_prompt(messages);
        let max_tokens = params.max_tokens as i32;
        let temperature = params.temperature;
        let top_p = params.top_p;

        let (tx, rx) = mpsc::channel(64);

        // Drive inference on the blocking pool (decode/sample are sync + heavy).
        tokio::task::spawn_blocking(move || {
            let result = run_inference(
                &model, backend, &prompt, max_tokens, temperature, top_p, &tx,
            );
            if let Err(e) = result {
                let _ = tx.blocking_send(StreamEvent::Error {
                    message: e.to_string(),
                });
            }
        });

        Ok(rx)
    }

    async fn classify(
        &self,
        _text: &str,
        _labels: &[String],
    ) -> Result<Vec<(String, f32)>, InferenceError> {
        // llama-cpp-2 exposes per-token logprobs which could power a
        // softmax-over-labels classifier; Phase 3 (MIG-051) wires that
        // path via constrained generation (GBNF grammar over a label
        // regex). Deferred until then; orchestrator + Phase 1 tools
        // don't call classify() in 0b.
        Err(InferenceError::NotConfigured(
            "classify() is not implemented in Phase 0b; coming in Phase 3 (MIG-051) via \
             constrained generation"
                .into(),
        ))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            model_id: self.model_id.clone(),
            runtime: "llama-cpp-2".into(),
            // Fanar default context. Phase 1 reads from loaded model's
            // metadata once capabilities() is plumbed through an async path.
            max_context_tokens: 8192,
            supports_tool_calls: infer_tool_support(&self.model_id),
            supports_citation: true,
        }
    }
}

/// One inference run: tokenize → decode prompt → sample tokens until
/// EOS / `max_tokens` / frontend closes. Emits `StreamEvent::Token`s
/// per generated token + a terminal `Done` event.
fn run_inference(
    model: &LlamaModel,
    backend: &LlamaBackend,
    prompt: &str,
    max_tokens: i32,
    temperature: f32,
    top_p: f32,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), InferenceError> {
    let ctx_params = LlamaContextParams::default();
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| InferenceError::Runtime(format!("new_context: {e}")))?;

    let tokens_list = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| InferenceError::Runtime(format!("str_to_token: {e}")))?;

    let n_prompt = tokens_list.len() as i32;
    if n_prompt == 0 {
        return Err(InferenceError::Runtime(
            "empty prompt after tokenization".into(),
        ));
    }

    // Feed prompt tokens in a single batch.
    let mut batch = LlamaBatch::new(512, 1);
    for (i, token) in tokens_list.iter().enumerate() {
        let is_last = i as i32 == n_prompt - 1;
        batch
            .add(*token, i as i32, &[0], is_last)
            .map_err(|e| InferenceError::Runtime(format!("batch.add prompt[{i}]: {e}")))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| InferenceError::Runtime(format!("decode prompt: {e}")))?;

    // Sampler chain: temperature → top-p → dist (random with seed).
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(temperature),
        LlamaSampler::top_p(top_p, 1),
        LlamaSampler::dist(42),
    ]);

    let mut n_cur = n_prompt;
    let n_len = n_prompt + max_tokens;
    let mut tokens_emitted: u32 = 0;
    let mut frontend_closed = false;

    while n_cur < n_len {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }

        let piece = match model.token_to_str(token, Special::Tokenize) {
            Ok(s) => s,
            // Multi-byte UTF-8 split across tokens may produce an Err
            // mid-character; emit empty and let the next token complete it.
            Err(_) => String::new(),
        };

        tokens_emitted += 1;
        if !frontend_closed
            && tx
                .blocking_send(StreamEvent::Token { text: piece })
                .is_err()
        {
            // Frontend dropped — keep generating to drain llama's KV state
            // cleanly, but stop forwarding events.
            frontend_closed = true;
        }

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| InferenceError::Runtime(format!("batch.add iter[{n_cur}]: {e}")))?;
        n_cur += 1;
        ctx.decode(&mut batch)
            .map_err(|e| InferenceError::Runtime(format!("decode iter[{n_cur}]: {e}")))?;
    }

    let finish_reason = if n_cur >= n_len {
        FinishReason::Length
    } else {
        FinishReason::Stop
    };

    let _ = tx.blocking_send(StreamEvent::Done {
        finish_reason,
        usage: TokenUsage {
            input_tokens: n_prompt as u32,
            output_tokens: tokens_emitted,
        },
    });

    Ok(())
}

/// Build a Gemma-2 chat-formatted prompt from a message history.
/// Fanar inherits Gemma-2's `<start_of_turn>` / `<end_of_turn>` template.
fn build_prompt(messages: &[ChatMessage]) -> String {
    let mut prompt = String::new();
    for m in messages {
        match m.role {
            ChatRole::System | ChatRole::User => {
                prompt.push_str("<start_of_turn>user\n");
                prompt.push_str(&m.content);
                prompt.push_str("<end_of_turn>\n");
            }
            ChatRole::Assistant => {
                prompt.push_str("<start_of_turn>model\n");
                prompt.push_str(&m.content);
                prompt.push_str("<end_of_turn>\n");
            }
            ChatRole::Tool => {
                // Tools aren't first-class in Gemma 2; embed the result
                // as a user-role observation. Phase 1 (MIG-048) refines
                // the chat-template handling for tool messages.
                prompt.push_str("<start_of_turn>user\n[tool_result]\n");
                prompt.push_str(&m.content);
                prompt.push_str("\n<end_of_turn>\n");
            }
        }
    }
    // Open the model turn so generation produces only the assistant reply.
    prompt.push_str("<start_of_turn>model\n");
    prompt
}

fn infer_tool_support(model_id: &str) -> bool {
    let lower = model_id.to_lowercase();
    lower.contains("qwen")
        || lower.contains("llama-3")
        || lower.contains("mistral-nemo")
        || lower.contains("gemma")
        || lower.contains("fanar")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Like the §C-v1 version, runtime-touching paths aren't unit-testable
    // (no GGUF in git; load is multi-GiB). Boss-test Stage 0 + bench_runtime
    // is the integration verification. Unit tests cover pure helpers.

    #[test]
    fn build_prompt_wraps_user_message() {
        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: "Hello".into(),
            tool_call_id: None,
            tool_name: None,
        }];
        let prompt = build_prompt(&messages);
        assert!(prompt.starts_with("<start_of_turn>user\nHello<end_of_turn>"));
        assert!(prompt.ends_with("<start_of_turn>model\n"));
    }

    #[test]
    fn build_prompt_handles_arabic_user_message() {
        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: "مرحبا، كيف حالك؟".into(),
            tool_call_id: None,
            tool_name: None,
        }];
        let prompt = build_prompt(&messages);
        assert!(prompt.contains("مرحبا، كيف حالك؟"));
        assert!(prompt.ends_with("<start_of_turn>model\n"));
    }

    #[test]
    fn build_prompt_handles_tool_role() {
        let messages = vec![
            ChatMessage {
                role: ChatRole::User,
                content: "search".into(),
                tool_call_id: None,
                tool_name: None,
            },
            ChatMessage {
                role: ChatRole::Tool,
                content: "{\"results\": []}".into(),
                tool_call_id: Some("c1".into()),
                tool_name: Some("search_notes".into()),
            },
        ];
        let prompt = build_prompt(&messages);
        assert!(prompt.contains("[tool_result]"));
        assert!(prompt.contains("\"results\": []"));
    }

    #[test]
    fn infer_tool_support_recognizes_known_families() {
        assert!(infer_tool_support("fanar-1-9b-q4km"));
        assert!(infer_tool_support("Fanar-1-9B-Instruct"));
        assert!(infer_tool_support("gemma-2-9b"));
        assert!(infer_tool_support("qwen3-8b"));
        assert!(!infer_tool_support("custom-toy-model"));
    }

    #[test]
    fn capabilities_report_llama_cpp_runtime() {
        let p = LocalProvider::new(PathBuf::from("/dev/null/model.gguf"), "test-model");
        let caps = p.capabilities();
        assert_eq!(caps.model_id, "test-model");
        assert_eq!(caps.runtime, "llama-cpp-2");
        assert!(caps.supports_citation);
    }
}

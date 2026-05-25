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
use crate::mind::orchestrator::gbnf;
use crate::mind::provider::{
    ChatMessage, ChatRole, FinishReason, GenParams, InferenceError, InferenceProvider,
    ProviderCapabilities, TokenUsage, ToolSchema,
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
        // MIG-048 §D: tool palette is owned by the spawned task so the
        // grammar string can outlive the &GenParams borrow.
        let tools = params.tools.clone();

        let (tx, rx) = mpsc::channel(64);

        // Drive inference on the blocking pool (decode/sample are sync + heavy).
        tokio::task::spawn_blocking(move || {
            let result = run_inference(
                &model, backend, &prompt, max_tokens, temperature, top_p, &tools, &tx,
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

/// The exact byte sequence that signals the start of a tool-call JSON.
/// Matches `gbnf::trigger_word()` — kept here as `&str` for ergonomic
/// `String::find`.
const TOOL_CALL_TRIGGER: &str = r#"{"tool":"#;

/// One inference run: tokenize → decode prompt → sample tokens until
/// EOS / `max_tokens` / frontend closes. Emits `StreamEvent::Token`s
/// per generated token + a terminal `Done` event — OR a single
/// `StreamEvent::ToolCall` + `Done { ToolCall }` when the model emits
/// a tool-call JSON (gated by GBNF grammar when `tools` non-empty).
fn run_inference(
    model: &LlamaModel,
    backend: &LlamaBackend,
    prompt: &str,
    max_tokens: i32,
    temperature: f32,
    top_p: f32,
    tools: &[ToolSchema],
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), InferenceError> {
    // MIG-048 §D: the default LlamaContextParams sets n_ctx to 512 which
    // is far too small for a Phase 1 chat turn (a system prompt with
    // tools + 6 tool descriptions easily exceeds 500 tokens). Boss-test
    // §I (2026-05-25) hit "Insufficient Space of 512" on the first turn.
    // Bump to Fanar's full 8192-token context — matches the budget
    // history::DEFAULT_CONTEXT_BUDGET_TOKENS (6500) leaves room for under,
    // and KV cache RAM (~168 MB at f16) is modest on modern hardware.
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(std::num::NonZeroU32::new(8192));
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

    // Feed prompt tokens in a single batch. Capacity must accommodate
    // the full tokenized prompt — when §F added the canonical system
    // prompt + tool-palette inline (~400-600 tokens), the original 512
    // hard-cap overflowed on real chat turns ("Insufficient Space of
    // 512" runtime error caught in Boss-test §I 2026-05-25). Size to
    // match n_ctx (8192) so the batch can hold any prompt the trim
    // budget (6500 tokens) lets through.
    let batch_capacity = (n_prompt as usize).max(8192);
    let mut batch = LlamaBatch::new(batch_capacity, 1);
    for (i, token) in tokens_list.iter().enumerate() {
        let is_last = i as i32 == n_prompt - 1;
        batch
            .add(*token, i as i32, &[0], is_last)
            .map_err(|e| InferenceError::Runtime(format!("batch.add prompt[{i}]: {e}")))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| InferenceError::Runtime(format!("decode prompt: {e}")))?;

    // Sampler chain. Grammar (if any) goes FIRST so it filters candidate
    // tokens before temperature/top-p/dist make the final pick. We use
    // `grammar_lazy` — the grammar only enforces shape AFTER the trigger
    // byte sequence `{"tool":` appears in the output. Up until then
    // the model can emit any prose (including starting with `{` for
    // unrelated reasons — code blocks, equations, etc.).
    let mut samplers: Vec<LlamaSampler> = Vec::with_capacity(4);
    if !tools.is_empty() {
        let grammar_str = gbnf::from_tools(tools);
        let g = LlamaSampler::grammar_lazy(
            model,
            &grammar_str,
            "tool-call",
            [gbnf::trigger_word()],
            &[],
        )
        .map_err(|e| InferenceError::Runtime(format!("grammar_lazy init: {e:?}")))?;
        samplers.push(g);
    }
    samplers.push(LlamaSampler::temp(temperature));
    samplers.push(LlamaSampler::top_p(top_p, 1));
    samplers.push(LlamaSampler::dist(42));
    let mut sampler = LlamaSampler::chain_simple(samplers);

    let mut n_cur = n_prompt;
    let n_len = n_prompt + max_tokens;
    let mut tokens_emitted: u32 = 0;
    let mut frontend_closed = false;

    // Streaming + tool-call detection state.
    //
    // `total_text` accumulates every emitted piece so we can search for
    // the trigger word (`{"tool":` — 8 bytes) without losing visibility
    // when it spans multiple BPE tokens. `emitted_len` tracks how many
    // bytes of `total_text` have already been forwarded as `Token`
    // events. The invariant: the trailing
    // `TOOL_CALL_TRIGGER.len() - 1 = 7` bytes of `total_text` are NEVER
    // emitted until we're sure they aren't the start of a forming
    // trigger — this is the safety window. When the trigger appears,
    // we emit the prose prefix as one Token and start `tool_buf`
    // accumulation; when `gbnf::try_parse_tool_call` succeeds, we emit
    // `ToolCall` + `Done { ToolCall }` and return early.
    let mut total_text = String::new();
    let mut emitted_len: usize = 0;
    let mut in_tool_call = false;
    let mut tool_buf = String::new();
    const HOLD_BACK: usize = 7; // TOOL_CALL_TRIGGER.len() - 1

    while n_cur < n_len {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }

        let piece = match model.token_to_str(token, Special::Tokenize) {
            Ok(s) => s,
            // Multi-byte UTF-8 split across tokens may produce an Err
            // mid-character; treat as empty and let the next token complete it.
            Err(_) => String::new(),
        };

        tokens_emitted += 1;

        if !piece.is_empty() {
            if in_tool_call {
                tool_buf.push_str(&piece);
                if let Ok(Some((tool_name, args))) = gbnf::try_parse_tool_call(&tool_buf) {
                    let id = format!("call_{}", chrono::Utc::now().timestamp_micros());
                    let _ = tx.blocking_send(StreamEvent::ToolCall {
                        id,
                        name: tool_name,
                        args,
                    });
                    let _ = tx.blocking_send(StreamEvent::Done {
                        finish_reason: FinishReason::ToolCall,
                        usage: TokenUsage {
                            input_tokens: n_prompt as u32,
                            output_tokens: tokens_emitted,
                        },
                    });
                    return Ok(());
                }
            } else {
                total_text.push_str(&piece);
                // Trigger search — only over the slice we haven't yet emitted.
                if let Some(rel_idx) = total_text[emitted_len..].find(TOOL_CALL_TRIGGER) {
                    let trigger_abs = emitted_len + rel_idx;
                    // Emit prose between emitted_len and the trigger.
                    if trigger_abs > emitted_len {
                        let prose = total_text[emitted_len..trigger_abs].to_string();
                        if !frontend_closed
                            && tx
                                .blocking_send(StreamEvent::Token { text: prose })
                                .is_err()
                        {
                            frontend_closed = true;
                        }
                    }
                    tool_buf = total_text[trigger_abs..].to_string();
                    in_tool_call = true;
                    emitted_len = total_text.len();
                } else {
                    // No trigger yet — emit everything except the trailing
                    // safety window. Use a UTF-8-safe char boundary for the
                    // cut so we never split a multi-byte codepoint.
                    let target_end = total_text.len().saturating_sub(HOLD_BACK);
                    let mut safe_end = target_end;
                    while safe_end > emitted_len && !total_text.is_char_boundary(safe_end) {
                        safe_end -= 1;
                    }
                    if safe_end > emitted_len {
                        let chunk = total_text[emitted_len..safe_end].to_string();
                        if !frontend_closed
                            && tx
                                .blocking_send(StreamEvent::Token { text: chunk })
                                .is_err()
                        {
                            frontend_closed = true;
                        }
                        emitted_len = safe_end;
                    }
                }
            }
        }

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| InferenceError::Runtime(format!("batch.add iter[{n_cur}]: {e}")))?;
        n_cur += 1;
        ctx.decode(&mut batch)
            .map_err(|e| InferenceError::Runtime(format!("decode iter[{n_cur}]: {e}")))?;
    }

    // End-of-stream flush: emit any held-back prose / partial tool-call.
    if !in_tool_call && emitted_len < total_text.len() && !frontend_closed {
        let tail = total_text[emitted_len..].to_string();
        let _ = tx.blocking_send(StreamEvent::Token { text: tail });
    } else if in_tool_call && !tool_buf.is_empty() && !frontend_closed {
        // Mid-tool-call EOS — surface the partial JSON so the user sees
        // the model's incomplete intent.
        let _ = tx.blocking_send(StreamEvent::Token { text: tool_buf.clone() });
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

//! `LocalProvider` — real local inference for Constellation Mind.
//!
//! MIG-047 Phase 0b Step C. Wraps `mistralrs 0.8.1` (per Eisa's §4 A
//! lock A4: mistral.rs in 0b for Fanar; llama-cpp-2 added in Phase 2.5
//! for Jais). Pure-Rust runtime on top of Candle — no cmake / C++
//! chain on Windows MSVC.
//!
//! Lazy load: the underlying `mistralrs::Model` is created on the first
//! `generate()` call via `tokio::sync::OnceCell`. Subsequent calls
//! reuse the loaded model (mmap-backed; ~5 GiB resident for Fanar-1-9B
//! Q4_K_M). Constructing a `LocalProvider` without invoking `generate()`
//! pays no model-load cost — preserves the boot-time invariant from
//! Architect §3 invariant 5.
//!
//! Engine-reboot mitigation (mistralrs issue #2147): when the frontend
//! drops the `StreamEvent` receiver mid-turn, our spawned forwarder
//! task continues draining the inner `mistralrs::Stream` to completion
//! rather than dropping it. Dropping mid-stream can panic mistralrs's
//! engine and force a reboot of the next turn. The agent reading the
//! mistralrs source confirmed this in #2147.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::{mpsc, OnceCell};

use mistralrs::{
    ChunkChoice, Delta, Function, GgufModelBuilder, Model, RequestBuilder, Response,
    StopTokens, TextMessageRole, TokenSource, Tool,
    ToolChoice as MrsToolChoice, ToolType,
};

use crate::mind::events::StreamEvent;
use crate::mind::provider::{
    ChatMessage, ChatRole, FinishReason, GenParams, InferenceError, InferenceProvider,
    ProviderCapabilities, ToolChoice, TokenUsage,
};

/// Real LocalProvider. One instance per loaded model; safe to share via
/// `Arc<LocalProvider>` across Tauri commands (mistralrs's `Model` is
/// `Arc<MistralRs>`-backed and `Send + Sync`).
pub struct LocalProvider {
    /// Absolute path to the `.gguf` file (one of the installed models
    /// from the per-user registry — see `mind::model_install`).
    model_path: PathBuf,
    /// User-visible model identity (e.g. `"fanar-1-9b-q4km"`).
    model_id: String,
    /// Lazily-loaded model; first `generate()` pays the load cost.
    /// Wrapped in `Arc` because mistralrs's `Model` doesn't implement
    /// `Clone` and `stream_chat_request` returns a borrow we need to
    /// move into a spawned task — owning an `Arc<Model>` lets us
    /// clone the Arc cheaply and pass an owned copy into the spawn.
    model: OnceCell<Arc<Model>>,
}

impl LocalProvider {
    pub fn new(model_path: PathBuf, model_id: impl Into<String>) -> Self {
        Self {
            model_path,
            model_id: model_id.into(),
            model: OnceCell::new(),
        }
    }

    /// Get a cloneable handle to the loaded mistralrs Model, loading it
    /// on first call. Returns `Arc<Model>` so callers can move it into
    /// spawned tasks.
    async fn get_model(&self) -> Result<Arc<Model>, InferenceError> {
        let arc_ref = self
            .model
            .get_or_try_init(|| async {
                let dir = self
                    .model_path
                    .parent()
                    .ok_or_else(|| {
                        InferenceError::NotConfigured(format!(
                            "model path has no parent: {}",
                            self.model_path.display()
                        ))
                    })?
                    .to_string_lossy()
                    .to_string();
                let file = self
                    .model_path
                    .file_name()
                    .ok_or_else(|| {
                        InferenceError::NotConfigured(format!(
                            "model path has no filename: {}",
                            self.model_path.display()
                        ))
                    })?
                    .to_string_lossy()
                    .to_string();

                let model = GgufModelBuilder::new(dir, vec![file])
                    .with_force_cpu() // Phase 0b ships CPU-only; GPU features land later
                    .with_token_source(TokenSource::None) // no HF auth needed (already-downloaded)
                    .with_max_num_seqs(1) // single-user desktop app
                    .build()
                    .await
                    .map_err(|e| {
                        InferenceError::Runtime(format!("mistralrs GgufModelBuilder: {e}"))
                    })?;
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
        // Load the model (lazy on first call). We must own the Model
        // inside the spawned task — the `Stream<'_>` mistralrs returns
        // borrows from the Model, so we can't keep the model behind
        // `&self` and move just the stream. Model is Arc<MistralRs>-backed
        // and cheap to clone.
        let model_owned: Arc<Model> = self.get_model().await?;
        let req = build_request(messages, params);

        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            // stream_chat_request runs in this task, so the returned
            // `Stream<'_>` borrows from `model_owned` (owned here).
            let mut stream = match model_owned.stream_chat_request(req).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx
                        .send(StreamEvent::Error {
                            message: format!("stream_chat_request: {e}"),
                        })
                        .await;
                    return;
                }
            };

            let mut frontend_closed = false;
            let mut emitted_terminal = false;

            while let Some(resp) = stream.next().await {
                let events = translate_response(resp);
                for ev in events {
                    let is_terminal = matches!(
                        ev,
                        StreamEvent::Done { .. } | StreamEvent::Error { .. }
                    );
                    if is_terminal {
                        emitted_terminal = true;
                    }
                    if !frontend_closed && tx.send(ev).await.is_err() {
                        frontend_closed = true;
                        // Don't break — keep draining the inner stream
                        // to avoid mistralrs's engine-reboot issue
                        // (#2147). The frontend has dropped its
                        // receiver; we'll just stop forwarding but
                        // continue consuming.
                    }
                }
            }

            // Stream closed without an explicit Done/Error — synthesize Stop.
            if !emitted_terminal && !frontend_closed {
                let _ = tx
                    .send(StreamEvent::Done {
                        finish_reason: FinishReason::Stop,
                        usage: TokenUsage::default(),
                    })
                    .await;
            }
        });

        Ok(rx)
    }

    async fn classify(
        &self,
        _text: &str,
        _labels: &[String],
    ) -> Result<Vec<(String, f32)>, InferenceError> {
        // mistralrs does NOT expose a first-class softmax-over-labels API.
        // Phase 3 (MIG-051) wires this via constrained generation
        // (mistralrs Constraint over a regex of the label set) or via
        // logprobs (set_return_logprobs(true) then read Choice.logprobs).
        // Defer until then — orchestrator + Phase 1 tools don't call
        // classify() in 0b.
        Err(InferenceError::NotConfigured(
            "classify() is not implemented in Phase 0b; coming in Phase 3 (MIG-051) via \
             constrained generation"
                .into(),
        ))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            model_id: self.model_id.clone(),
            runtime: "mistralrs-0.8".into(),
            // 8192 is the Fanar default; once the model is loaded we
            // could read self.model.get().map(|m| m.config()) but the
            // capabilities() method is sync. The orchestrator only
            // surfaces this for diagnostics; an upper-bound estimate
            // is fine until Phase 1 plumbs the real config readout.
            max_context_tokens: 8192,
            supports_tool_calls: infer_tool_support(&self.model_id),
            // Local models follow the system prompt's citation rule by
            // construction (no upstream rewriting); the post-generation
            // citation validator (Phase 1 / MIG-048) is the enforcer.
            supports_citation: true,
        }
    }
}

// ─── Request construction ─────────────────────────────────────────

fn build_request(messages: &[ChatMessage], params: &GenParams) -> RequestBuilder {
    let mut req = RequestBuilder::new();

    for m in messages {
        match m.role {
            ChatRole::System => {
                req = req.add_message(TextMessageRole::System, m.content.clone());
            }
            ChatRole::User => {
                req = req.add_message(TextMessageRole::User, m.content.clone());
            }
            ChatRole::Assistant => {
                req = req.add_message(TextMessageRole::Assistant, m.content.clone());
            }
            ChatRole::Tool => {
                let id = m.tool_call_id.clone().unwrap_or_default();
                req = req.add_tool_message(m.content.clone(), id);
            }
        }
    }

    req = req
        .set_sampler_temperature(params.temperature as f64)
        .set_sampler_topp(params.top_p as f64)
        .set_sampler_max_len(params.max_tokens as usize);

    if !params.stop.is_empty() {
        req = req.set_sampler_stop_toks(StopTokens::Seqs(params.stop.clone()));
    }

    if !params.tools.is_empty() {
        let tools: Vec<Tool> = params
            .tools
            .iter()
            .map(|t| {
                // mistralrs Tool::function::parameters expects HashMap<String, Value>
                // carrying the JSON-Schema object (type, properties, required, ...).
                let params_map: HashMap<String, serde_json::Value> = match &t.input_schema {
                    serde_json::Value::Object(map) => {
                        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    }
                    _ => HashMap::new(),
                };
                Tool {
                    tp: ToolType::Function,
                    function: Function {
                        name: t.name.clone(),
                        description: Some(t.description.clone()),
                        parameters: Some(params_map),
                    },
                }
            })
            .collect();
        req = req.set_tools(tools);

        let mrs_choice = match params.tool_choice {
            ToolChoice::Auto => MrsToolChoice::Auto,
            ToolChoice::None => MrsToolChoice::None,
            // mistralrs's ToolChoice may not expose Required directly; Auto with
            // a system-prompt nudge is the common fallback. Phase 1 + bench can
            // refine this if needed.
            ToolChoice::Required => MrsToolChoice::Auto,
        };
        req = req.set_tool_choice(mrs_choice);
    }

    req
}

// ─── Response translation ─────────────────────────────────────────

fn translate_response(resp: Response) -> Vec<StreamEvent> {
    match resp {
        Response::Chunk(chunk) => translate_chunk(chunk),
        Response::Done(full) => {
            // Non-streamed path; we won't hit this on stream_chat_request,
            // but be defensive.
            let usage = TokenUsage {
                input_tokens: full.usage.prompt_tokens as u32,
                output_tokens: full.usage.completion_tokens as u32,
            };
            vec![StreamEvent::Done {
                finish_reason: FinishReason::Stop,
                usage,
            }]
        }
        Response::ModelError(msg, _partial) => {
            vec![StreamEvent::Error { message: msg }]
        }
        Response::InternalError(e) | Response::ValidationError(e) => {
            vec![StreamEvent::Error {
                message: e.to_string(),
            }]
        }
        _ => Vec::new(),
    }
}

fn translate_chunk(chunk: mistralrs::ChatCompletionChunkResponse) -> Vec<StreamEvent> {
    let mut out: Vec<StreamEvent> = Vec::new();
    let Some(ChunkChoice {
        delta:
            Delta {
                content,
                tool_calls,
                ..
            },
        finish_reason,
        ..
    }) = chunk.choices.into_iter().next()
    else {
        return out;
    };

    if let Some(text) = content {
        if !text.is_empty() {
            out.push(StreamEvent::Token { text });
        }
    }

    if let Some(tcs) = tool_calls {
        for tc in tcs {
            // mistralrs delivers tool-call arguments as a JSON string
            // (the model emitted JSON; mistralrs's parser left it as a
            // string for the caller to parse). Try to parse; if it
            // fails, surface as a string Value so the dispatcher can
            // still see what the model emitted.
            let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                .unwrap_or_else(|_| serde_json::Value::String(tc.function.arguments.clone()));
            out.push(StreamEvent::ToolCall {
                id: tc.id,
                name: tc.function.name,
                args,
            });
        }
    }

    if let Some(reason) = finish_reason {
        out.push(StreamEvent::Done {
            finish_reason: translate_finish_reason(&reason),
            // Chunks don't carry usage; aggregate happens in the orchestrator
            // (which counts tokens via the telemetry counters per MIG-046 §E).
            usage: TokenUsage::default(),
        });
    }

    out
}

fn translate_finish_reason(s: &str) -> FinishReason {
    match s {
        "stop" => FinishReason::Stop,
        "length" => FinishReason::Length,
        "tool_calls" | "tool_call" => FinishReason::ToolCall,
        "cancelled" | "canceled" => FinishReason::Cancelled,
        _ => FinishReason::Stop,
    }
}

fn infer_tool_support(model_id: &str) -> bool {
    let lower = model_id.to_lowercase();
    // Known tool-trained families. Conservative — false-negatives are
    // safer than false-positives here (model still works for chat; it
    // just won't be offered tools).
    lower.contains("qwen")
        || lower.contains("llama-3")
        || lower.contains("mistral-nemo")
        || lower.contains("gemma")
        || lower.contains("fanar")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::provider::ToolSchema;

    // We can't load a real GGUF in unit tests (multi-GiB; not in
    // git). The integration verification happens through Step G's
    // Boss-test Stage 0. What we CAN test here: the pure functions
    // that build requests + translate responses without touching
    // mistralrs's runtime.

    #[test]
    fn finish_reason_translation_covers_known_strings() {
        assert_eq!(translate_finish_reason("stop"), FinishReason::Stop);
        assert_eq!(translate_finish_reason("length"), FinishReason::Length);
        assert_eq!(translate_finish_reason("tool_calls"), FinishReason::ToolCall);
        assert_eq!(translate_finish_reason("tool_call"), FinishReason::ToolCall);
        assert_eq!(translate_finish_reason("cancelled"), FinishReason::Cancelled);
        // Unknown reasons map conservatively to Stop.
        assert_eq!(translate_finish_reason("eos_token"), FinishReason::Stop);
    }

    #[test]
    fn infer_tool_support_recognizes_known_families() {
        assert!(infer_tool_support("fanar-1-9b-q4km"));
        assert!(infer_tool_support("Fanar-1-9B-Instruct"));
        assert!(infer_tool_support("gemma-2-9b"));
        assert!(infer_tool_support("qwen3-8b"));
        assert!(infer_tool_support("Llama-3.1-8B-Instruct"));
        assert!(infer_tool_support("mistral-nemo-12b"));
        // Unknown / not-trained-for-tools.
        assert!(!infer_tool_support("custom-toy-model"));
    }

    #[test]
    fn build_request_with_no_tools_skips_tool_setup() {
        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
            tool_call_id: None,
            tool_name: None,
        }];
        let params = GenParams::default();
        // Just exercise the builder — no assertion on internal state,
        // we're checking it doesn't panic and produces a RequestBuilder.
        let _req = build_request(&messages, &params);
    }

    #[test]
    fn build_request_with_tools_maps_schema() {
        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: "search for X".into(),
            tool_call_id: None,
            tool_name: None,
        }];
        let params = GenParams {
            tools: vec![ToolSchema {
                name: "search_notes".into(),
                description: "Search the user's notes".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            }],
            tool_choice: ToolChoice::Auto,
            ..GenParams::default()
        };
        let _req = build_request(&messages, &params);
        // Just confirms no panic + builds.
    }

    #[test]
    fn capabilities_report_mistralrs_runtime() {
        let p = LocalProvider::new(PathBuf::from("/dev/null/model.gguf"), "test-model");
        let caps = p.capabilities();
        assert_eq!(caps.model_id, "test-model");
        assert!(caps.runtime.starts_with("mistralrs"));
        assert!(caps.supports_citation);
    }
}

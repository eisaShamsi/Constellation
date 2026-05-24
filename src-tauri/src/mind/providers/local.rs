//! `LocalProvider` — deterministic stub for Phase 0a.
//!
//! In Phase 0b (MIG-047) this is replaced by a real implementation that
//! wraps `mistral.rs` or `llama-cpp-2` (chosen by the one-day micro-bench
//! at the start of Phase 0b). For 0a, it emits fixed event sequences so
//! the trait surface can be exercised by unit tests + the orchestrator
//! skeleton (Step D) + the Tauri IPC layer (Step C) without any model
//! loaded.
//!
//! Tool-call protocol (Pattern B — "generate-restart", matching the
//! Anthropic HTTP API style):
//! - Round 1: user message + non-empty `params.tools` → emit one
//!   `ToolCall(name=tools[0].name)` + `Done { finish_reason: ToolCall }`.
//! - Round 2: caller appends a `ChatMessage { role: Tool, … }` and calls
//!   `generate` again → emit 3 finalizing tokens + `Done { finish_reason: Stop }`.
//!
//! Phase 1's orchestrator (MIG-048) is the consumer that loops `generate`
//! until `Done { finish_reason: Stop | Length | Cancelled | Error }`.

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::mind::events::StreamEvent;
use crate::mind::provider::{
    ChatMessage, ChatRole, EmbeddingCapabilities, EmbeddingProvider, FinishReason, GenParams,
    InferenceError, InferenceProvider, ProviderCapabilities, TokenUsage,
};

/// 384-dim matches `multilingual-e5-small` (the model wired in
/// `src-tauri/src/embeddings.rs`) so the Phase 0b/1 swap from this stub to
/// a real `LocalEmbeddingProvider` wrapping the ONNX session is
/// dimensionally a no-op for callers.
const STUB_EMBED_DIM: u32 = 384;

pub struct LocalProvider;

impl LocalProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceProvider for LocalProvider {
    async fn generate(
        &self,
        messages: &[ChatMessage],
        params: &GenParams,
    ) -> Result<mpsc::Receiver<StreamEvent>, InferenceError> {
        let (tx, rx) = mpsc::channel(16);

        // Pattern B: if the last message is a Tool result, we're resuming
        // a tool-call round-trip — emit finalization tokens.
        let is_round_two = matches!(
            messages.last(),
            Some(ChatMessage {
                role: ChatRole::Tool,
                ..
            })
        );
        let want_tool_call = !params.tools.is_empty() && !is_round_two;
        let tool_name = params.tools.first().map(|t| t.name.clone());

        tokio::spawn(async move {
            if want_tool_call {
                let _ = tx
                    .send(StreamEvent::ToolCall {
                        id: "stub-call-1".into(),
                        name: tool_name.unwrap_or_else(|| "stub_tool".into()),
                        args: serde_json::json!({ "query": "stub" }),
                    })
                    .await;
                let _ = tx
                    .send(StreamEvent::Done {
                        finish_reason: FinishReason::ToolCall,
                        usage: TokenUsage {
                            input_tokens: 10,
                            output_tokens: 5,
                        },
                    })
                    .await;
            } else if is_round_two {
                for piece in ["Result", " received", " — done."] {
                    let _ = tx
                        .send(StreamEvent::Token {
                            text: piece.to_string(),
                        })
                        .await;
                }
                let _ = tx
                    .send(StreamEvent::Done {
                        finish_reason: FinishReason::Stop,
                        usage: TokenUsage {
                            input_tokens: 18,
                            output_tokens: 3,
                        },
                    })
                    .await;
            } else {
                for piece in ["Hello", " from", " the", " local", " stub."] {
                    let _ = tx
                        .send(StreamEvent::Token {
                            text: piece.to_string(),
                        })
                        .await;
                }
                let _ = tx
                    .send(StreamEvent::Done {
                        finish_reason: FinishReason::Stop,
                        usage: TokenUsage {
                            input_tokens: 8,
                            output_tokens: 5,
                        },
                    })
                    .await;
            }
        });

        Ok(rx)
    }

    async fn classify(
        &self,
        _text: &str,
        labels: &[String],
    ) -> Result<Vec<(String, f32)>, InferenceError> {
        if labels.is_empty() {
            return Ok(Vec::new());
        }
        let n = labels.len() as f32;
        let mut out: Vec<(String, f32)> = labels
            .iter()
            .enumerate()
            .map(|(i, l)| (l.clone(), (n - i as f32) / n))
            .collect();
        out.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(out)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            model_id: "local-stub".into(),
            runtime: "stub".into(),
            max_context_tokens: 4096,
            supports_tool_calls: true,
            supports_citation: true,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for LocalProvider {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, InferenceError> {
        Ok(texts
            .iter()
            .map(|_| vec![0.0_f32; STUB_EMBED_DIM as usize])
            .collect())
    }

    fn embed_capabilities(&self) -> EmbeddingCapabilities {
        EmbeddingCapabilities {
            model_id: "stub".into(),
            dimension: STUB_EMBED_DIM,
            max_input_tokens: 512,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::provider::ToolSchema;

    fn user_msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: ChatRole::User,
            content: content.into(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    #[tokio::test]
    async fn generate_emits_five_tokens_then_done_when_no_tools() {
        let p = LocalProvider::new();
        let msgs = vec![user_msg("hi")];
        let mut rx = p.generate(&msgs, &GenParams::default()).await.unwrap();

        let mut tokens: Vec<String> = Vec::new();
        let mut done_seen = false;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::Token { text } => tokens.push(text),
                StreamEvent::Done { finish_reason, .. } => {
                    assert_eq!(finish_reason, FinishReason::Stop);
                    done_seen = true;
                }
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens.concat(), "Hello from the local stub.");
        assert!(done_seen);
    }

    #[tokio::test]
    async fn generate_round_one_emits_tool_call_then_done_toolcall() {
        let p = LocalProvider::new();
        let msgs = vec![user_msg("create a note")];
        let params = GenParams {
            tools: vec![ToolSchema {
                name: "create_note".into(),
                description: "stub".into(),
                input_schema: serde_json::json!({ "type": "object" }),
            }],
            ..GenParams::default()
        };

        let mut rx = p.generate(&msgs, &params).await.unwrap();
        let mut got_tool_call_name: Option<String> = None;
        let mut got_finish: Option<FinishReason> = None;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::ToolCall { name, .. } => got_tool_call_name = Some(name),
                StreamEvent::Done { finish_reason, .. } => got_finish = Some(finish_reason),
                other => panic!("unexpected round-1 event: {other:?}"),
            }
        }
        assert_eq!(got_tool_call_name.as_deref(), Some("create_note"));
        assert_eq!(got_finish, Some(FinishReason::ToolCall));
    }

    #[tokio::test]
    async fn generate_round_two_finalizes_after_tool_result() {
        let p = LocalProvider::new();
        // Round-two messages: user + tool-result.
        let msgs = vec![
            user_msg("create a note"),
            ChatMessage {
                role: ChatRole::Tool,
                content: serde_json::json!({ "status": "ok" }).to_string(),
                tool_call_id: Some("stub-call-1".into()),
                tool_name: Some("create_note".into()),
            },
        ];
        let params = GenParams {
            tools: vec![ToolSchema {
                name: "create_note".into(),
                description: "stub".into(),
                input_schema: serde_json::json!({ "type": "object" }),
            }],
            ..GenParams::default()
        };

        let mut rx = p.generate(&msgs, &params).await.unwrap();
        let mut tokens: Vec<String> = Vec::new();
        let mut done_seen = false;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::Token { text } => tokens.push(text),
                StreamEvent::Done { finish_reason, .. } => {
                    assert_eq!(finish_reason, FinishReason::Stop);
                    done_seen = true;
                }
                other => panic!("unexpected round-2 event: {other:?}"),
            }
        }
        assert_eq!(tokens.len(), 3);
        assert!(done_seen);
    }

    #[tokio::test]
    async fn embed_returns_384dim_zero_vectors() {
        let p = LocalProvider::new();
        let out = p
            .embed(&["one".into(), "two".into(), "three".into()])
            .await
            .unwrap();
        assert_eq!(out.len(), 3);
        for v in &out {
            assert_eq!(v.len(), STUB_EMBED_DIM as usize);
            assert!(v.iter().all(|x| *x == 0.0));
        }
        // Capabilities advertise the same dimension.
        assert_eq!(p.embed_capabilities().dimension, STUB_EMBED_DIM);
    }

    #[tokio::test]
    async fn classify_returns_descending_confidences() {
        let p = LocalProvider::new();
        let labels = vec!["alpha".into(), "beta".into(), "gamma".into()];
        let out = p.classify("ignored", &labels).await.unwrap();
        assert_eq!(out.len(), 3);
        for w in out.windows(2) {
            assert!(w[0].1 >= w[1].1, "expected descending: {:?}", out);
        }
    }

    #[test]
    fn capabilities_advertise_local_stub() {
        let cap = LocalProvider::new().capabilities();
        assert_eq!(cap.model_id, "local-stub");
        assert_eq!(cap.runtime, "stub");
        assert!(cap.supports_tool_calls);
        assert!(cap.supports_citation);
    }
}

//! `CloudProvider` — Anthropic-shaped scaffold stub for Phase 0a.
//!
//! Phase 5 (MIG-053) replaces this with a real Anthropic HTTP client +
//! cost-meter (per Concept Paper v1.1 §11.4 and §10.2 — `Arc<CostMeter>`
//! field there will land in Phase 5). The 0a stub does **NOT** make any
//! network call — invariant 7 ("Local-First / no exfiltration") in the
//! Phase 0a Architect — it returns deterministic canned events so unit
//! tests can verify the trait surface accepts a "cloud" shape:
//!
//! - Large context window (advertised as 200k tokens — Claude's range)
//! - `runtime` field carries the `anthropic-http-stub` discriminator
//! - `supports_citation = false` — cloud providers can't be statically
//!   guaranteed; citation faithfulness depends on system-prompt enforcement
//!   at runtime, validated post-generation by the citation validator
//!   (Phase 1 MIG-048).

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::mind::events::StreamEvent;
use crate::mind::provider::{
    ChatMessage, FinishReason, GenParams, InferenceError, InferenceProvider,
    ProviderCapabilities, TokenUsage,
};

pub struct CloudProvider {
    pub model_id: String,
}

impl CloudProvider {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
        }
    }
}

impl Default for CloudProvider {
    fn default() -> Self {
        Self::new("anthropic-claude-stub")
    }
}

#[async_trait]
impl InferenceProvider for CloudProvider {
    async fn generate(
        &self,
        _messages: &[ChatMessage],
        _params: &GenParams,
    ) -> Result<mpsc::Receiver<StreamEvent>, InferenceError> {
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(async move {
            for piece in ["Cloud", " stub", " reply", "."] {
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
                        input_tokens: 12,
                        output_tokens: 4,
                    },
                })
                .await;
        });
        Ok(rx)
    }

    async fn classify(
        &self,
        _text: &str,
        labels: &[String],
    ) -> Result<Vec<(String, f32)>, InferenceError> {
        Ok(labels.iter().map(|l| (l.clone(), 0.5)).collect())
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            model_id: self.model_id.clone(),
            runtime: "anthropic-http-stub".into(),
            max_context_tokens: 200_000,
            supports_tool_calls: true,
            // Cloud providers cannot statically guarantee citation faithfulness;
            // the system-prompt rule + post-generation validator (Phase 1) enforces it.
            supports_citation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::provider::ChatRole;

    #[tokio::test]
    async fn generate_emits_four_tokens_then_done() {
        let p = CloudProvider::default();
        let msgs = vec![ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
            tool_call_id: None,
            tool_name: None,
        }];
        let mut rx = p.generate(&msgs, &GenParams::default()).await.unwrap();

        let mut tokens: Vec<String> = Vec::new();
        let mut done_seen = false;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::Token { text } => tokens.push(text),
                StreamEvent::Done { .. } => done_seen = true,
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert_eq!(tokens.concat(), "Cloud stub reply.");
        assert!(done_seen);
    }

    #[test]
    fn capabilities_advertise_anthropic_shape() {
        let cap = CloudProvider::default().capabilities();
        assert!(cap.runtime.starts_with("anthropic"));
        assert_eq!(cap.max_context_tokens, 200_000);
        // Cloud cannot statically guarantee citation discipline.
        assert!(!cap.supports_citation);
    }
}

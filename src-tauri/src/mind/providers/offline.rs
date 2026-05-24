//! `OfflineProvider` — what runs when no provider is configured.
//!
//! Returns one synthesized assistant message explaining the user needs to
//! install a model. Never makes any network call. Always available; the
//! safe fallback when LocalProvider isn't loaded AND CloudProvider has no
//! key, AND when a Cloud monthly cost cap auto-disables (Phase 5
//! deliverable per Concept Paper v1.1 §11.4).

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::mind::events::StreamEvent;
use crate::mind::provider::{
    ChatMessage, FinishReason, GenParams, InferenceError, InferenceProvider,
    ProviderCapabilities, TokenUsage,
};

pub struct OfflineProvider;

impl OfflineProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OfflineProvider {
    fn default() -> Self {
        Self::new()
    }
}

const OFFLINE_MESSAGE: &str =
    "Constellation Mind has no model configured. Open Settings → Mind to install one.";

#[async_trait]
impl InferenceProvider for OfflineProvider {
    async fn generate(
        &self,
        _messages: &[ChatMessage],
        _params: &GenParams,
    ) -> Result<mpsc::Receiver<StreamEvent>, InferenceError> {
        let (tx, rx) = mpsc::channel(2);
        tokio::spawn(async move {
            let _ = tx
                .send(StreamEvent::Token {
                    text: OFFLINE_MESSAGE.to_string(),
                })
                .await;
            let _ = tx
                .send(StreamEvent::Done {
                    finish_reason: FinishReason::Stop,
                    usage: TokenUsage::default(),
                })
                .await;
        });
        Ok(rx)
    }

    async fn classify(
        &self,
        _text: &str,
        _labels: &[String],
    ) -> Result<Vec<(String, f32)>, InferenceError> {
        Err(InferenceError::NotConfigured(
            "OfflineProvider cannot classify — install a model first".into(),
        ))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            model_id: "offline".into(),
            runtime: "none".into(),
            max_context_tokens: 0,
            supports_tool_calls: false,
            supports_citation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::provider::ChatRole;

    #[tokio::test]
    async fn generate_returns_one_message_then_done() {
        let p = OfflineProvider::new();
        let msgs = vec![ChatMessage {
            role: ChatRole::User,
            content: "anything".into(),
            tool_call_id: None,
            tool_name: None,
        }];
        let mut rx = p.generate(&msgs, &GenParams::default()).await.unwrap();

        let mut text_seen: Option<String> = None;
        let mut done_seen = false;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::Token { text } => text_seen = Some(text),
                StreamEvent::Done { .. } => done_seen = true,
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert_eq!(text_seen.as_deref(), Some(OFFLINE_MESSAGE));
        assert!(done_seen);
    }

    #[tokio::test]
    async fn classify_errors_not_configured() {
        let p = OfflineProvider::new();
        let err = p
            .classify("ignored", &["alpha".into()])
            .await
            .unwrap_err();
        match err {
            InferenceError::NotConfigured(_) => {}
            other => panic!("expected NotConfigured, got {other:?}"),
        }
    }
}

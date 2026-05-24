//! Inference + embedding trait surface — the strategic moat.
//!
//! Two traits, split intentionally (Concept Paper v1.1 §10.1, MA-1) so a
//! model family that is strong at generation but indifferent at embedding
//! (or vice versa) can be wired in without forcing one provider to do
//! both. `RoutedProvider` (Phase 2.5 / MIG-050) wraps multiple
//! `InferenceProvider`s and is *itself* an `InferenceProvider`.
//!
//! Future implementations (none in Phase 0a — see `providers/` in Step B
//! for the stubs):
//! - `LocalProvider` — wraps `mistral.rs` or `llama-cpp-2` (Phase 0b)
//! - `RoutedProvider` — composes 1..N inner providers (Phase 2.5)
//! - `CloudProvider` — Anthropic / OpenAI / OpenRouter (Phase 5)
//! - `OfflineProvider` — synthesized "no model configured" responses
//! - `LocalEmbeddingProvider` — wraps `src-tauri/src/embeddings.rs` (Phase 0b/1)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::mind::events::StreamEvent;

// ─── Chat message + role ───────────────────────────────────────────

/// One message in the chat history sent to the model.
///
/// Roles match the OpenAI/Anthropic conventions; the prompt assembler in
/// Phase 1 (MIG-048) will translate to the wire format of the chosen
/// provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    /// For the `Tool` role: the `tool_call_id` this message is the result
    /// of (matches the `id` of an earlier `StreamEvent::ToolCall`).
    /// `None` for other roles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// For the `Tool` role: the tool name this message is the result of.
    /// `None` for other roles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

// ─── Generation parameters ─────────────────────────────────────────

/// Parameters for one `InferenceProvider::generate` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    #[serde(default)]
    pub stop: Vec<String>,
    /// Tool schemas the model may call. Empty = no tools available.
    #[serde(default)]
    pub tools: Vec<ToolSchema>,
    /// Whether the model must call a tool, may choose, or must not.
    #[serde(default)]
    pub tool_choice: ToolChoice,
}

impl Default for GenParams {
    fn default() -> Self {
        Self {
            max_tokens: 1024,
            temperature: 0.7,
            top_p: 0.95,
            stop: Vec::new(),
            tools: Vec::new(),
            tool_choice: ToolChoice::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    /// Model may call any of the available tools, or none.
    #[default]
    Auto,
    /// Model must not call any tool.
    None,
    /// Model must call a tool (any of the available tools).
    Required,
}

// ─── Tool schema (the contract the model sees) ─────────────────────

/// One tool the model may invoke. The shape mirrors the
/// Anthropic / OpenAI function-calling schema (see Concept Paper v1.1
/// Appendix B for the `create_note` example).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    /// JSON-schema-shaped input parameter description. Validated by the
    /// dispatcher before any approval modal is shown (Phase 1 / MIG-048).
    pub input_schema: serde_json::Value,
}

// ─── Termination + usage ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Generation finished naturally (EOS or matched a `GenParams.stop` token).
    Stop,
    /// Hit `GenParams.max_tokens`.
    Length,
    /// Model emitted a tool call; the orchestrator handed control back to
    /// the dispatcher. The next turn iteration resumes generation after
    /// the tool result is pushed into history.
    ToolCall,
    /// Generation was cancelled by the frontend (channel dropped, user
    /// dismissed the chat, etc.).
    Cancelled,
    /// Provider returned an error mid-stream.
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// ─── Provider self-description ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// e.g. `"local-stub"`, `"fanar-1-9b-q4km"`, `"anthropic-claude-sonnet-4-6"`.
    pub model_id: String,
    /// e.g. `"stub"`, `"mistral.rs"`, `"llama-cpp-2"`, `"anthropic-http"`.
    pub runtime: String,
    /// Maximum context window in tokens.
    pub max_context_tokens: u32,
    /// Whether this provider supports the streaming tool-call protocol.
    pub supports_tool_calls: bool,
    /// Whether this provider's output can be reliably citation-bound
    /// (i.e. doesn't rewrite retrieved content). Local providers
    /// typically true; cloud providers depend on system-prompt
    /// enforcement.
    pub supports_citation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCapabilities {
    /// e.g. `"multilingual-e5-small"`, `"bge-m3"`, `"stub"`.
    pub model_id: String,
    /// Output vector dimension.
    pub dimension: u32,
    /// Maximum input length in tokens before truncation.
    pub max_input_tokens: u32,
}

// ─── Errors ────────────────────────────────────────────────────────

/// Error type returned by both traits.
///
/// `#[serde(tag = "kind")]` produces a discriminated union the frontend can
/// match on directly. `Display` is implemented manually (no `thiserror`
/// dep in Phase 0a per Architect invariant 3 / "no new heavy deps").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum InferenceError {
    /// The provider isn't configured (no model loaded, no API key, etc.).
    /// Typically `OfflineProvider` returns this from `generate`.
    NotConfigured(String),

    /// Underlying runtime failed (`mistral.rs`, `llama-cpp-2`, HTTP client, …).
    Runtime(String),

    /// The model produced output that violated its schema (e.g. a tool
    /// call with non-JSON args).
    InvalidOutput(String),

    /// The call was cancelled by the caller (channel dropped, user
    /// dismissed the chat, etc.).
    Cancelled,
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured(s) => write!(f, "provider not configured: {s}"),
            Self::Runtime(s) => write!(f, "runtime error: {s}"),
            Self::InvalidOutput(s) => write!(f, "invalid model output: {s}"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for InferenceError {}

// ─── The two traits ────────────────────────────────────────────────

/// Generation, classification, capability self-description.
///
/// Every implementation — `LocalProvider`, `CloudProvider`,
/// `RoutedProvider`, `OfflineProvider` — implements this trait. The
/// `ChatOrchestrator` and the tool dispatcher hold
/// `Arc<dyn InferenceProvider>` and never see the concrete type. This is
/// the strategic moat (Concept Paper §5.5 / §14.2).
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Stream generation with tool-call support.
    ///
    /// Returns a receiver of `StreamEvent`s. Implementations MUST emit
    /// exactly one terminal event (`Done` or `Error`) and then drop the
    /// sender so the channel closes.
    async fn generate(
        &self,
        messages: &[ChatMessage],
        params: &GenParams,
    ) -> Result<mpsc::Receiver<StreamEvent>, InferenceError>;

    /// Lightweight classification over a fixed label set.
    ///
    /// Returns `(label, confidence)` pairs sorted descending by
    /// confidence. Confidences SHOULD sum to ~1.0 but are not strictly
    /// required to (different model families normalize differently).
    /// Phase 3 (MIG-051) uses this for note-type taxonomy classification.
    async fn classify(
        &self,
        text: &str,
        labels: &[String],
    ) -> Result<Vec<(String, f32)>, InferenceError>;

    /// Provider self-description for diagnostics and model swapping.
    fn capabilities(&self) -> ProviderCapabilities;
}

/// Embedding generation. Composed independently of `InferenceProvider`
/// (MA-1) so the embedding model can evolve without touching the
/// generation surface — and vice versa.
///
/// Phase 0b/1 wraps `src-tauri/src/embeddings.rs` (the existing
/// `multilingual-e5-small` ONNX pipeline) as a `LocalEmbeddingProvider`.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding vectors for a batch of texts. The returned
    /// `Vec<Vec<f32>>` has one inner vector per input text, in the same
    /// order. Each inner vector has length `embed_capabilities().dimension`.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, InferenceError>;

    /// Embedding-provider self-description.
    fn embed_capabilities(&self) -> EmbeddingCapabilities;
}

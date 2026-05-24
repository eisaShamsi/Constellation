//! MIG-046 Phase 0a — Constellation Mind: Inference Abstraction Skeleton.
//!
//! Phase 0a lays the trait surface only. Phase 0b (MIG-047) wires real local
//! inference; Phase 1 (MIG-048) ships the conversational RAG; Phase 2.5
//! (MIG-050) adds the `RoutedProvider` that composes multiple `LocalProvider`s.
//!
//! See:
//! - `docs/Constellation-Mind-Concept-Paper-v1.1.md` for the full architecture
//! - `docs/MIG-046-constellation-mind-phase0a-inference-abstraction-ARCHITECT.md`
//!   for the phase scope, invariants, and step plan
//!
//! ## Module layout
//!
//! - `provider` — the two traits (`InferenceProvider` + `EmbeddingProvider`)
//!   that gate the entire LLM surface, plus the supporting types they exchange.
//! - `events` — the `StreamEvent` enum streamed from a turn to the frontend.
//!
//! ## What does NOT live here yet
//!
//! Later steps of MIG-046 add:
//! - `providers/` — three deterministic stub implementations (Step B)
//! - `commands.rs` — Tauri IPC commands (Step C)
//! - `orchestrator.rs` — the `ChatOrchestrator` (Step D)
//! - `telemetry.rs` — in-process counters (Step E)
//!
//! ## Coexistence with existing intelligence surfaces
//!
//! Phase 0a leaves all of these untouched:
//! - `src-tauri/src/ai/mod.rs` — cloud bridge (Phase 5 / MIG-053 refactors as `CloudProvider`)
//! - `src-tauri/src/cece/catalogers/reasoning.rs` — CECE's local-LLM stub (Phase 3 / MIG-051 rewires through `RoutedProvider`)
//! - `src-tauri/src/embeddings.rs` — ONNX embedding pipeline (Phase 0b/1 wraps as `LocalEmbeddingProvider`)
//! - `src-tauri/src/nsc/` — the `summarize` tool delegates here in Phase 1 / MIG-048

pub mod commands;
pub mod events;
pub mod orchestrator;
pub mod provider;
pub mod providers;
pub mod telemetry;

pub use events::StreamEvent;
pub use orchestrator::{
    framing, CannedDispatcher, ChatConfig, ChatError, ChatOrchestrator, ToolDispatcher,
    TurnOutcome, UiEvent,
};
pub use provider::{
    ChatMessage, ChatRole, EmbeddingCapabilities, EmbeddingProvider, FinishReason, GenParams,
    InferenceError, InferenceProvider, ProviderCapabilities, TokenUsage, ToolChoice, ToolSchema,
};
pub use providers::{CloudProvider, LocalEmbeddingProvider, LocalProvider, OfflineProvider};
pub use telemetry::TelemetrySnapshot;

//! Three deterministic stub providers for Phase 0a (MIG-046 Step B).
//!
//! Each implements `InferenceProvider` with a known event sequence so
//! unit tests, the IPC layer (Step C), and the orchestrator (Step D) can
//! exercise the trait surface without any model loaded.
//!
//! Replaced by real implementations in later phases:
//! - `local` — Phase 0b (MIG-047) swaps the stub for a real `mistral.rs`
//!   or `llama-cpp-2` wrapper (chosen by the Phase 0b micro-bench).
//! - `cloud` — Phase 5 (MIG-053) swaps the stub for a real Anthropic
//!   HTTP client (Eisa's OpenClaw experience applies).
//! - `offline` — kept as the safe fallback throughout. The "no model
//!   configured" path the user sees on a fresh install or when their
//!   chosen provider fails to load.

pub mod cloud;
pub mod local;
pub mod local_embedding;
pub mod local_stub;
pub mod offline;

pub use cloud::CloudProvider;
pub use local::LocalProvider;
pub use local_embedding::LocalEmbeddingProvider;
pub use local_stub::LocalStubProvider;
pub use offline::OfflineProvider;

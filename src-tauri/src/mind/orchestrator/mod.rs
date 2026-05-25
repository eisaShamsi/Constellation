//! `mind/orchestrator/` — the conversation engine.
//!
//! Phase 1 §A promoted this from a single `orchestrator.rs` file to a
//! directory module so the dispatcher implementations and the 4 (then 6)
//! tool wrappers can each live in their own files instead of accreting
//! into one large file.
//!
//! ## Module layout
//!
//! - [`core`] — `ChatOrchestrator`, `ChatConfig`, `UiEvent`,
//!   `ChatError`, `TurnOutcome`, and the [`core::framing`] helper.
//! - [`dispatcher`] — the [`dispatcher::ToolDispatcher`] trait,
//!   [`dispatcher::CannedDispatcher`] (test-only), and
//!   [`dispatcher::RealToolDispatcher`] (production).
//! - [`tools`] — one module per tool the model can call
//!   (`search_notes`, `read_note`, `find_similar`, `summarize`).
//!   §C extends with `list_recent` + `graph_neighbors`.
//!
//! ## Phase 1 follow-ups in sibling modules
//!
//! - `prompt.rs` (§F) — system prompt + envelope assembly
//! - `citation_validator.rs` (§G) — `[note:UUID]` post-stream validator
//! - `history.rs` (§K) — sliding-window E2 trim
//! - `gbnf.rs` (§D) — JSON-Schema → GBNF grammar for the local sampler

pub mod core;
pub mod dispatcher;
pub mod tools;

pub use core::{
    framing, ChatConfig, ChatError, ChatOrchestrator, TurnOutcome, UiEvent,
};
pub use dispatcher::{CannedDispatcher, RealToolDispatcher, ToolDispatcher};

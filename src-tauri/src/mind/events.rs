//! Streaming events from the Reasoning Layer.
//!
//! One `mind_start_turn` IPC call (Step C of MIG-046) produces a stream of
//! `StreamEvent`s on a Tauri `Channel<StreamEvent>`. The frontend consumes
//! them in order; the channel closes after a single terminal event
//! (`Done` or `Error`).

use serde::{Deserialize, Serialize};

use crate::mind::provider::{FinishReason, TokenUsage};

/// One streaming event from a single turn.
///
/// The variant order is the order events typically arrive in:
/// zero-or-more `Token`s interleaved with zero-or-more `ToolCall`s,
/// terminated by exactly one `Done` (or one `Error` if the provider
/// failed mid-stream).
///
/// `#[serde(tag = "type")]` produces a discriminated union the frontend
/// can match on directly:
/// `{"type":"token","text":"…"}`, `{"type":"tool_call","id":"…","name":"…","args":{…}}`,
/// `{"type":"done","finish_reason":"stop","usage":{…}}`,
/// `{"type":"error","message":"…"}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// A piece of assistant text. Concatenating all `Token.text` in order
    /// reconstructs the assistant's message for the turn.
    Token { text: String },

    /// The model requested a tool. The dispatcher (Step D / Phase 1)
    /// executes it and feeds the result back into the next iteration via
    /// `ChatMessage { role: Tool, tool_call_id, tool_name, content }`.
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
    },

    /// The turn completed normally. `finish_reason` distinguishes
    /// natural stop, length cap, tool-call yield, etc.
    Done {
        finish_reason: FinishReason,
        usage: TokenUsage,
    },

    /// The provider hit an unrecoverable error. The turn ends.
    Error { message: String },
}

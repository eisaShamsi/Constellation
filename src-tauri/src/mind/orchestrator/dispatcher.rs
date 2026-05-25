//! Tool dispatcher — routes a model's tool-call request to the right
//! implementation and converts the result into the JSON tool-result the
//! model sees next.
//!
//! ## Two implementations
//!
//! - [`CannedDispatcher`] — returns `{"status": "ok", "tool": <name>}` for
//!   every call. Kept since MIG-046 for unit tests where the orchestrator
//!   loop shape is what's under test, not the tool side effect.
//!
//! - [`RealToolDispatcher`] — Phase 1 (§A) addition. Holds an
//!   `AppHandle` (cheap to clone — internally `Arc`-backed) so each tool
//!   can reach its subsystem via `app.state::<X>()`. Catches tool
//!   errors and turns them into `{"status": "error", "error": "..."}`
//!   JSON the model can read on its next pass.
//!
//! ## Why `AppHandle` (not `Arc<AppHandle>`)
//!
//! The Architect §4 A wrote "`Arc<tauri::AppHandle>`" for clarity, but
//! the codebase's convention (`mind/model_install/commands.rs`,
//! `ai/mod.rs`, `cece/wiring.rs`) holds `AppHandle` directly because
//! `tauri::AppHandle: Clone` is already `Arc`-backed under the hood —
//! wrapping it in a second `Arc<>` adds a pointless indirection. The
//! field type used here matches the codebase convention.

use async_trait::async_trait;
use serde_json::{json, Value};
use tauri::AppHandle;

use super::tools;

// ─── Trait ────────────────────────────────────────────────────────

/// Tool dispatcher abstraction. Phase 0a used a `CannedDispatcher`;
/// Phase 1 adds [`RealToolDispatcher`] with the 4 ready read tools.
/// Phase 1 §C wires the remaining 2; Phase 2 (MIG-049) adds the write
/// tools with approval-gate semantics.
#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    async fn dispatch(&self, tool_name: &str, args: Value) -> Value;
}

// ─── CannedDispatcher (test-only / loop verification) ──────────────

pub struct CannedDispatcher;

#[async_trait]
impl ToolDispatcher for CannedDispatcher {
    async fn dispatch(&self, tool_name: &str, _args: Value) -> Value {
        json!({ "status": "ok", "tool": tool_name })
    }
}

// ─── RealToolDispatcher (Phase 1 §A) ───────────────────────────────

/// Production dispatcher used by `mind_start_turn` from Step §E onward.
/// Routes by tool name into the [`super::tools`] module family; any
/// unknown name returns `{"status": "error", "error": "unknown tool"}`
/// so the model sees the failure and can recover (e.g., re-attempt with
/// a known name).
pub struct RealToolDispatcher {
    app: AppHandle,
}

impl RealToolDispatcher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[async_trait]
impl ToolDispatcher for RealToolDispatcher {
    async fn dispatch(&self, tool_name: &str, args: Value) -> Value {
        let app = self.app.clone();
        let result: Result<Value, String> = match tool_name {
            "search_notes" => tools::search_notes::run(app, args).await,
            "read_note" => tools::read_note::run(app, args).await,
            "find_similar" => tools::find_similar::run(app, args).await,
            "summarize" => tools::summarize::run(app, args).await,
            // §C will add: "list_recent", "graph_neighbors"
            other => Err(format!("unknown tool: {other}")),
        };

        match result {
            Ok(v) => v,
            Err(e) => json!({
                "status": "error",
                "tool": tool_name,
                "error": e,
            }),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn canned_dispatcher_echoes_tool_name() {
        let d = CannedDispatcher;
        let result = d.dispatch("anything", json!({})).await;
        assert_eq!(result.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(result.get("tool").and_then(|v| v.as_str()), Some("anything"));
    }

    // Note: RealToolDispatcher integration tests require a Tauri AppHandle
    // which we don't construct in unit tests. The routing logic is exercised
    // end-to-end in Step §E's manual verification + Step §M's audit. Pure
    // schema-level tests live in `tools/mod.rs::tests::ready_palette_*`.
}

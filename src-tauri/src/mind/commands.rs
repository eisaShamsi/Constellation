//! Tauri IPC commands for the Mind subsystem.
//!
//! Two commands ship in Phase 0a:
//! - `mind_start_turn` — opens a `tauri::ipc::Channel<StreamEvent>`,
//!   spawns a task that drives a provider through one turn, pushes
//!   events to the channel until a terminal event. Returns immediately
//!   so the frontend can begin awaiting events.
//! - `mind_telemetry_snapshot` — returns the current in-process counters
//!   (`telemetry::snapshot()`). Phase 0a returns all zeros; Step E wires
//!   real atomics into the orchestrator.
//!
//! Wired into `lib.rs:invoke_handler` alongside the existing `ai::*`
//! entries. The `ai::*` commands are left strictly untouched (Architect
//! §3 invariant 1).
//!
//! Streaming primitive: `tauri::ipc::Channel<T>` — Tauri v2's first-class
//! typed-event channel (Architect §4 Option C1). One channel per
//! `mind_start_turn` invocation; closed naturally when the task drops the
//! sender (the channel's drop signals the frontend's `onmessage` close).

use std::path::PathBuf;

use serde::Deserialize;
use tauri::ipc::Channel;
use tauri::AppHandle;

use crate::mind::events::StreamEvent;
use crate::mind::provider::{ChatMessage, ChatRole, GenParams, InferenceProvider};
use crate::mind::providers::LocalProvider;
use crate::mind::telemetry::{self, TelemetrySnapshot};

/// Request shape for `mind_start_turn`.
///
/// Phase 0a only needs `user_message`; later phases add `conversation_id`
/// (so the orchestrator can resume history), provider selection, and
/// per-turn override flags. Keeping the type explicit now means future
/// fields land additively.
#[derive(Debug, Deserialize)]
pub struct StartTurnRequest {
    pub user_message: String,
    /// Reserved for Phase 1+ (orchestrator history is keyed on this).
    /// Present in 0a so the frontend contract is stable.
    #[serde(default)]
    pub conversation_id: String,
}

/// Start one conversational turn and stream events to the frontend.
///
/// Phase 0b behaviour (MIG-047 §G): resolves the user's currently-active
/// model via `mind_active_model`, instantiates a real `LocalProvider`
/// against that model's GGUF path, and drives one turn. The command
/// returns immediately after spawning the streaming task — the task
/// drains the provider's receiver and forwards each event to the
/// frontend `Channel`. A terminal `Done` or `Error` event closes the
/// channel.
///
/// If no model is installed/active, emits a single `StreamEvent::Error`
/// pointing the user to Settings → Mind and returns `Err(...)`. The
/// chat UI (Phase 1 / MIG-048) renders this inline as a "install a
/// model first" notice.
#[tauri::command]
pub async fn mind_start_turn(
    app: AppHandle,
    request: StartTurnRequest,
    on_event: Channel<StreamEvent>,
) -> Result<(), String> {
    // Resolve the active model from the install registry.
    let active = match crate::mind::model_install::commands::mind_active_model(app.clone()).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            let msg = "No active Constellation Mind model. \
                       Open Settings → Mind to install one.";
            let _ = on_event.send(StreamEvent::Error {
                message: msg.to_string(),
            });
            return Err(msg.to_string());
        }
        Err(e) => {
            let msg = format!("Failed to read installed-model registry: {e}");
            let _ = on_event.send(StreamEvent::Error {
                message: msg.clone(),
            });
            return Err(msg);
        }
    };

    let provider = LocalProvider::new(PathBuf::from(&active.file_path), active.id.clone());

    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: request.user_message,
        tool_call_id: None,
        tool_name: None,
    }];
    let params = GenParams::default();

    tauri::async_runtime::spawn(async move {
        match provider.generate(&messages, &params).await {
            Ok(mut rx) => {
                while let Some(ev) = rx.recv().await {
                    // `send` returns Err when the frontend has dropped
                    // the channel (user dismissed the chat, navigated
                    // away). Stop draining quietly in that case.
                    if on_event.send(ev).is_err() {
                        break;
                    }
                }
            }
            Err(e) => {
                let _ = on_event.send(StreamEvent::Error {
                    message: e.to_string(),
                });
            }
        }
    });

    Ok(())
}

/// Read out the current in-process telemetry counters.
///
/// Phase 0a returns all zeros (`telemetry::snapshot()` is a stub). Step E
/// wires real counters through the orchestrator.
#[tauri::command]
pub async fn mind_telemetry_snapshot() -> Result<TelemetrySnapshot, String> {
    Ok(telemetry::snapshot())
}

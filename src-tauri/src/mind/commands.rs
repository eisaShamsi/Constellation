//! Tauri IPC commands for the Mind subsystem.
//!
//! Two commands ship in Phase 0a, with §E (MIG-048) refactoring
//! `mind_start_turn` to drive through `ChatOrchestrator`:
//!
//! - `mind_start_turn` — opens a `tauri::ipc::Channel<StreamEvent>`,
//!   spawns a task that drives one turn through the orchestrator,
//!   bridging `UiEvent` to `StreamEvent` for the frontend.
//! - `mind_telemetry_snapshot` — returns the current in-process counters
//!   (`telemetry::snapshot()`). The orchestrator increments those
//!   counters internally during each turn.
//!
//! Wired into `lib.rs:invoke_handler` alongside the existing `ai::*`
//! entries. The `ai::*` commands are left strictly untouched (Architect
//! §3 invariant 1).
//!
//! Streaming primitive: `tauri::ipc::Channel<T>` — Tauri v2's first-class
//! typed-event channel (Architect §4 Option C1). One channel per
//! `mind_start_turn` invocation; closed naturally when the spawned task
//! drops the sender (the channel's drop signals the frontend's
//! `onmessage` close).

use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use tauri::ipc::Channel;
use tauri::AppHandle;
use tokio::sync::mpsc;

use crate::mind::events::StreamEvent;
use crate::mind::orchestrator::{ChatOrchestrator, RealToolDispatcher, ToolDispatcher, UiEvent};
use crate::mind::provider::{GenParams, InferenceProvider};
use crate::mind::providers::LocalProvider;
use crate::mind::telemetry::{self, TelemetrySnapshot};

/// Request shape for `mind_start_turn`.
///
/// `conversation_id` is reserved for Phase 1.x — when the orchestrator
/// gains persistent multi-turn history keyed on this. Each call today
/// is single-turn (history starts empty).
#[derive(Debug, Deserialize)]
pub struct StartTurnRequest {
    pub user_message: String,
    #[serde(default)]
    pub conversation_id: String,
}

/// Start one conversational turn and stream events to the frontend.
///
/// §E (MIG-048) wiring:
/// 1. Resolve active model via `mind_active_model`.
/// 2. Construct `LocalProvider` against the model's GGUF path.
/// 3. Construct `RealToolDispatcher::new(app)` — the orchestrator
///    routes tool calls through this.
/// 4. Spawn the orchestrator turn on the Tauri async runtime. The
///    orchestrator emits `UiEvent`s on an internal mpsc channel; a
///    second spawned task translates each `UiEvent` to the
///    corresponding `StreamEvent` and forwards it on the Tauri
///    `Channel<StreamEvent>`.
/// 5. Returns immediately — the frontend's `onmessage` handler drains
///    events until a terminal `Done` / `Error`.
///
/// If no model is installed/active, emits a single `StreamEvent::Error`
/// and returns `Err(...)`. The chat UI (§H) renders this inline as an
/// "install a model first" notice.
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

    let provider: Arc<dyn InferenceProvider> = Arc::new(LocalProvider::new(
        PathBuf::from(&active.file_path),
        active.id.clone(),
    ));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(RealToolDispatcher::new(app.clone()));

    let user_message = request.user_message;

    // The orchestrator emits UiEvent; a bridge task translates each
    // UiEvent into the appropriate StreamEvent for the frontend channel.
    let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(64);

    // Spawn the bridge task. Lives until ui_rx closes (orchestrator
    // task drops its sender on turn completion).
    let on_event_bridge = on_event.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(ev) = ui_rx.recv().await {
            let stream_ev = match ev {
                UiEvent::AssistantToken(text) => Some(StreamEvent::Token { text }),
                UiEvent::ToolCallProposed { id, name, args } => {
                    Some(StreamEvent::ToolCall { id, name, args })
                }
                UiEvent::TurnDone {
                    finish_reason,
                    usage,
                } => Some(StreamEvent::Done {
                    finish_reason,
                    usage,
                }),
                UiEvent::Error { message } => Some(StreamEvent::Error { message }),
                // Internal accounting only — no frontend signal.
                UiEvent::ToolCallResolved { .. } | UiEvent::ToolBudgetReached => None,
            };
            if let Some(se) = stream_ev {
                if on_event_bridge.send(se).is_err() {
                    // Frontend dropped — stop translating.
                    break;
                }
            }
        }
    });

    // Spawn the orchestrator turn. Drops `ui_tx` on completion, which
    // closes the bridge's `ui_rx`, which lets the bridge task exit.
    //
    // §G: build the citation-validator closure here so the AppHandle
    // clone is owned by the closure (not by the orchestrator struct).
    // This keeps the orchestrator's surface free of tauri::AppHandle
    // and lets unit tests construct an orchestrator without a runtime.
    let app_for_validator = app.clone();
    let citation_hook: crate::mind::orchestrator::CitationValidatorHook =
        Box::new(move |text: &str| {
            let (_valid, invalid) =
                crate::mind::orchestrator::citation_validator::scan_and_verify(
                    &app_for_validator,
                    text,
                );
            invalid
        });

    tauri::async_runtime::spawn(async move {
        let mut orch = ChatOrchestrator::new(provider, dispatcher)
            .with_citation_validator(citation_hook);

        // GenParams with the full Phase 1 read-tool palette so the model
        // knows which tools it may call. The LocalProvider's run_inference
        // installs GBNF grammar gate on this list (§D).
        let params = GenParams {
            max_tokens: 1024,
            tools: crate::mind::orchestrator::tools::ready_palette(),
            ..GenParams::default()
        };

        if let Err(e) = orch.turn(user_message, params, ui_tx.clone()).await {
            // The orchestrator already emitted UiEvent::Error before
            // returning Err in §E. The extra send here covers the
            // ChatError::UiChannelClosed case where the bridge already
            // exited — best-effort, ignored on failure.
            let _ = ui_tx
                .send(UiEvent::Error {
                    message: e.to_string(),
                })
                .await;
        }
    });

    Ok(())
}

/// Read out the current in-process telemetry counters.
///
/// The orchestrator increments these counters during each turn (Step E
/// of MIG-046, exercised end-to-end by §E of MIG-048).
#[tauri::command]
pub async fn mind_telemetry_snapshot() -> Result<TelemetrySnapshot, String> {
    Ok(telemetry::snapshot())
}


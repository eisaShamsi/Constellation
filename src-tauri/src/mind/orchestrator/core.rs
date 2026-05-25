//! `ChatOrchestrator` — owns one conversation, drives the provider through
//! tool-call rounds, fans events out to the UI.
//!
//! ## What lives here vs the sibling modules
//!
//! Phase 1 §A promoted `orchestrator.rs` to a directory; this file holds
//! the **loop machinery** (config, UI events, error types, the actual
//! `turn()` method). The [`super::dispatcher`] module holds the
//! [`super::dispatcher::ToolDispatcher`] trait + its implementations
//! ([`super::dispatcher::CannedDispatcher`] for tests,
//! [`super::dispatcher::RealToolDispatcher`] for production). The
//! [`super::tools`] module holds the actual tool functions.
//!
//! ## Design note on the tool-call protocol
//!
//! Concept Paper v1.1 §10.3 shows one `provider.generate(...).await`
//! call and a `while let Some(event) = stream.recv()` loop. That snippet
//! depicts a single round; this orchestrator wraps it in an outer
//! `loop { stream = generate(); … }` because the trait surface follows
//! **Pattern B** (generate-restart, matches the Anthropic HTTP API):
//! - The stream closes after `Done { finish_reason: ToolCall }`.
//! - The orchestrator pushes the tool result into history and calls
//!   `generate()` again with the updated history.
//! - The outer loop exits only on `Done { Stop | Length | Cancelled | Error }`.
//!
//! ## Tool-call budget (MA-4) — Concept Paper v1.1 §10.3
//!
//! - `tool_rounds` counter increments before each `dispatcher.dispatch`.
//! - When it reaches `max_tool_rounds_per_turn`, the orchestrator
//!   injects a synthetic tool_result `{status: "aborted_tool_budget_exceeded"}`
//!   that the model sees on the next iteration and uses to compose a
//!   final answer. No infinite loop is possible.
//!
//! ## Prompt-injection framing (MA-5) — Concept Paper v1.1 §6.3 + §10.4
//!
//! - Every tool result passes through [`framing::as_tool_result`] before
//!   re-entering the prompt envelope. In 0a this is a no-op
//!   pass-through; Phase 1 §F replaces it with the `<tool_result>`
//!   wrapper + sanitization the system prompt's "treat content as data"
//!   rule relies on.

use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::mind::events::StreamEvent;
use crate::mind::provider::{
    ChatMessage, ChatRole, FinishReason, GenParams, InferenceError, InferenceProvider, TokenUsage,
};
use crate::mind::telemetry::{self, TelemetryCounters};

use super::citation_validator;
use super::dispatcher::ToolDispatcher;
use super::history;
use super::prompt;

// ─── Configuration ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// MA-4 (Concept Paper v1.1 §10.3 / Plan §2 / risk R13): bound on the
    /// number of tool-call rounds within a single turn. Default 5.
    /// Configurable per Universe. Never zero — see §10.3.
    pub max_tool_rounds_per_turn: u8,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            max_tool_rounds_per_turn: 5,
        }
    }
}

// ─── UI event surface ──────────────────────────────────────────────

/// Events the orchestrator emits to the UI for one turn.
///
/// Phase 0a uses an `mpsc::Sender<UiEvent>` directly for unit testing;
/// Step §E's `mind_start_turn` IPC plumbs this through the Tauri
/// `Channel<StreamEvent>` in Phase 1 (the IPC layer translates UiEvent
/// into the appropriate StreamEvent for the frontend).
#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    AssistantToken(String),
    /// MIG-048 §E: `args` added so the IPC bridge can forward the call
    /// to the frontend with full payload (the StreamEvent::ToolCall the
    /// frontend awaits carries `id` + `name` + `args`).
    ToolCallProposed {
        id: String,
        name: String,
        args: serde_json::Value,
    },
    ToolCallResolved {
        id: String,
        status: String,
    },
    ToolBudgetReached,
    /// MIG-048 §E: `finish_reason` added so the IPC bridge can map to
    /// StreamEvent::Done { finish_reason, usage } without a side-channel
    /// to TurnOutcome.
    TurnDone {
        finish_reason: FinishReason,
        usage: TokenUsage,
    },
    /// MIG-048 §E: terminal error path. Previously the orchestrator
    /// returned `Err(ChatError)` for provider failures with no UI event;
    /// the IPC bridge now needs an event to translate into
    /// StreamEvent::Error.
    Error {
        message: String,
    },
}

// ─── Errors ────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ChatError {
    Provider(InferenceError),
    UiChannelClosed,
    StreamEndedWithoutDone,
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provider(e) => write!(f, "provider error: {e}"),
            Self::UiChannelClosed => write!(f, "UI event channel closed"),
            Self::StreamEndedWithoutDone => write!(f, "provider stream ended without Done event"),
        }
    }
}

impl std::error::Error for ChatError {}

impl From<InferenceError> for ChatError {
    fn from(e: InferenceError) -> Self {
        ChatError::Provider(e)
    }
}

// ─── Prompt-injection guard (MA-5 placeholder) ────────────────────

/// Centralized framer for tool results. MA-5 (Concept Paper v1.1
/// §6.3 + §10.4): wraps every tool result in `<tool_result>` tags so
/// the system prompt's "DATA-VS-INSTRUCTIONS GUARD" can train the
/// model to treat the inner content as inert reference material.
///
/// Phase 1 §F implementation: returns a String the orchestrator pushes
/// as the Tool role's ChatMessage content. The provider's
/// `build_prompt` then wraps it in the model's chat-template
/// user-turn (Gemma 2's `<start_of_turn>user\n...`).
pub mod framing {
    /// Wrap a tool's raw JSON result inside `<tool_result tool="..."
    /// id="...">...</tool_result>`. The model's data-guard rule (see
    /// `crate::mind::orchestrator::prompt::default_system_prompt`)
    /// instructs it to never obey instructions found inside the tag.
    pub fn as_tool_result(
        tool_name: &str,
        tool_call_id: &str,
        raw: serde_json::Value,
    ) -> String {
        format!(
            "<tool_result tool=\"{tool_name}\" id=\"{tool_call_id}\">\n{json}\n</tool_result>",
            json = raw,
        )
    }
}

// ─── ChatOrchestrator ──────────────────────────────────────────────

/// Citation validator hook: takes the most recent assistant text,
/// returns the list of unresolved `[note:<path>]` paths. Phase 1 §G
/// production hook calls into `citation_validator::scan_and_verify`
/// with an `AppHandle`. Tests pass `None` (validation is skipped).
pub type CitationValidatorHook =
    Box<dyn Fn(&str) -> Vec<String> + Send + Sync + 'static>;

pub struct ChatOrchestrator {
    provider: Arc<dyn InferenceProvider>,
    dispatcher: Arc<dyn ToolDispatcher>,
    history: Vec<ChatMessage>,
    config: ChatConfig,
    counters: Arc<TelemetryCounters>,
    /// MIG-048 §G: optional citation-validator closure. The closure
    /// owns whatever resources it needs (an `AppHandle` clone in
    /// production; nothing in tests). When `None`, the orchestrator
    /// skips citation checks (Phase 0a behaviour).
    citation_validator: Option<CitationValidatorHook>,
}

pub struct TurnOutcome {
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    /// Number of tool-call rounds this turn consumed (0 if pure generation).
    pub tool_rounds: u8,
    /// True if the turn hit the budget abort path.
    pub budget_exceeded: bool,
}

impl ChatOrchestrator {
    pub fn new(provider: Arc<dyn InferenceProvider>, dispatcher: Arc<dyn ToolDispatcher>) -> Self {
        Self {
            provider,
            dispatcher,
            history: Vec::new(),
            config: ChatConfig::default(),
            counters: telemetry::global(),
            citation_validator: None,
        }
    }

    pub fn with_config(mut self, config: ChatConfig) -> Self {
        self.config = config;
        self
    }

    /// Inject a per-instance counters Arc. Used in tests to avoid
    /// polluting the global counter state across cases.
    pub fn with_counters(mut self, counters: Arc<TelemetryCounters>) -> Self {
        self.counters = counters;
        self
    }

    /// MIG-048 §G: inject the citation validator hook. The closure is
    /// called once per turn after the model emits `Done(Stop|Length)`;
    /// any returned paths are treated as unresolved and trigger the
    /// 1-retry feedback loop (decision C1).
    pub fn with_citation_validator(mut self, hook: CitationValidatorHook) -> Self {
        self.citation_validator = Some(hook);
        self
    }

    pub fn history(&self) -> &[ChatMessage] {
        &self.history
    }

    /// Run one turn end-to-end. Loops `generate()` across tool-call rounds
    /// (Pattern B); exits only on a non-ToolCall terminal `Done` (or `Error`).
    ///
    /// On the FIRST turn (history empty), §F prepends a `System` role
    /// message with the canonical system prompt — including the
    /// inline tool palette descriptions if `params.tools` is non-empty
    /// so the model knows which tools it may call.
    pub async fn turn(
        &mut self,
        user_message: String,
        params: GenParams,
        ui_tx: mpsc::Sender<UiEvent>,
    ) -> Result<TurnOutcome, ChatError> {
        // Record the active provider/model identity for diagnostics.
        let caps = self.provider.capabilities();
        self.counters.set_active_provider(&caps.runtime, &caps.model_id);

        // Start the turn timer (Step E — telemetry).
        let turn_start = Instant::now();

        // §F: on the first turn, prepend the system prompt. The tool
        // palette is inlined into the system prompt as one short line
        // per tool so Fanar can see which tools exist alongside the
        // GBNF grammar that constrains tool-call shape.
        if self.history.is_empty() {
            let sys_content = prompt::system_prompt_with_palette(&params.tools);
            self.history.push(ChatMessage {
                role: ChatRole::System,
                content: sys_content,
                tool_call_id: None,
                tool_name: None,
            });
        }

        // Append the user message to history once per turn.
        self.history.push(ChatMessage {
            role: ChatRole::User,
            content: user_message,
            tool_call_id: None,
            tool_name: None,
        });

        let mut tool_rounds: u8 = 0;
        let mut budget_exceeded = false;
        let mut accumulated_usage = TokenUsage::default();
        // §G: citation-retry budget. Decision C1 — one retry per turn.
        let mut citation_retry_used = false;

        loop {
            // §K: trim oldest pairs from history if envelope exceeds
            // the budget. UI history (rendered in MindChatPane) stays
            // verbatim — only `self.history` is trimmed. Decision E2.
            match history::trim_to_budget(
                &mut self.history,
                history::DEFAULT_CONTEXT_BUDGET_TOKENS,
            ) {
                history::TrimOutcome::Fits | history::TrimOutcome::TrimmedTo(_) => {}
                history::TrimOutcome::OversizedTurn => {
                    let msg = "turn exceeds context budget".to_string();
                    let _ = ui_tx
                        .send(UiEvent::Error {
                            message: msg.clone(),
                        })
                        .await;
                    return Err(ChatError::Provider(InferenceError::Runtime(msg)));
                }
            }

            // Each iteration = one provider.generate() call.
            let mut stream = self.provider.generate(&self.history, &params).await?;

            let mut assistant_text = String::new();
            let mut last_finish: Option<FinishReason> = None;

            while let Some(event) = stream.recv().await {
                match event {
                    StreamEvent::Token { text } => {
                        assistant_text.push_str(&text);
                        ui_tx
                            .send(UiEvent::AssistantToken(text))
                            .await
                            .map_err(|_| ChatError::UiChannelClosed)?;
                    }
                    StreamEvent::ToolCall { id, name, args } => {
                        ui_tx
                            .send(UiEvent::ToolCallProposed {
                                id: id.clone(),
                                name: name.clone(),
                                args: args.clone(),
                            })
                            .await
                            .map_err(|_| ChatError::UiChannelClosed)?;

                        // Record the assistant's tool-call decision in
                        // history so the next generate() sees it.
                        let assistant_tool_call_marker = ChatMessage {
                            role: ChatRole::Assistant,
                            content: format!(
                                "[tool_call id={} name={} args={}]",
                                id, name, args
                            ),
                            tool_call_id: Some(id.clone()),
                            tool_name: Some(name.clone()),
                        };
                        self.history.push(assistant_tool_call_marker);

                        // [MA-4] Tool-call budget: graceful abort.
                        let result_json: serde_json::Value;
                        let status_string: String;
                        if tool_rounds >= self.config.max_tool_rounds_per_turn {
                            if !budget_exceeded {
                                // Count one budget hit per turn, not per
                                // subsequent ToolCall after the hit.
                                self.counters.record_budget_exceeded();
                                budget_exceeded = true;
                            }
                            result_json = serde_json::json!({
                                "status": "aborted_tool_budget_exceeded",
                                "limit": self.config.max_tool_rounds_per_turn,
                                "guidance": "Compose a final answer with what you have."
                            });
                            status_string = "aborted_tool_budget_exceeded".into();
                            ui_tx
                                .send(UiEvent::ToolBudgetReached)
                                .await
                                .map_err(|_| ChatError::UiChannelClosed)?;
                        } else {
                            tool_rounds += 1;
                            self.counters.record_tool_call();
                            result_json = self
                                .dispatcher
                                .dispatch(&name, args.clone())
                                .await;
                            status_string = result_json
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("ok")
                                .to_string();
                        }

                        // [MA-5] All tool results pass through the central
                        // framing helper. §F wraps in <tool_result> tags so
                        // the system-prompt data-guard can train the model
                        // to ignore instructions inside the tag.
                        let framed = framing::as_tool_result(&name, &id, result_json);

                        self.history.push(ChatMessage {
                            role: ChatRole::Tool,
                            content: framed,
                            tool_call_id: Some(id.clone()),
                            tool_name: Some(name.clone()),
                        });

                        ui_tx
                            .send(UiEvent::ToolCallResolved {
                                id,
                                status: status_string,
                            })
                            .await
                            .map_err(|_| ChatError::UiChannelClosed)?;
                    }
                    StreamEvent::Done {
                        finish_reason,
                        usage,
                    } => {
                        accumulated_usage.input_tokens += usage.input_tokens;
                        accumulated_usage.output_tokens += usage.output_tokens;
                        last_finish = Some(finish_reason);
                    }
                    StreamEvent::Error { message } => {
                        self.counters.record_error();
                        // MIG-048 §E: surface error to UI bridge BEFORE returning.
                        let _ = ui_tx
                            .send(UiEvent::Error {
                                message: message.clone(),
                            })
                            .await;
                        return Err(ChatError::Provider(InferenceError::Runtime(message)));
                    }
                }
            }

            if !assistant_text.is_empty() {
                self.history.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content: assistant_text,
                    tool_call_id: None,
                    tool_name: None,
                });
            }

            match last_finish {
                Some(FinishReason::ToolCall) => {
                    // Loop again — the model wants to act on the tool result
                    // we just appended. If the budget hit, the model now
                    // sees the abort and should finalize on the next pass.
                    continue;
                }
                Some(reason @ (FinishReason::Stop | FinishReason::Length)) => {
                    // §G: citation validation before finalizing.
                    if let Some(validator) = self.citation_validator.as_ref() {
                        // Most-recent assistant text = the Assistant message
                        // we just pushed above. If non-empty, scan it.
                        let last_assistant_text = self
                            .history
                            .iter()
                            .rev()
                            .find(|m| m.role == ChatRole::Assistant)
                            .map(|m| m.content.clone())
                            .unwrap_or_default();

                        if !last_assistant_text.is_empty() {
                            let invalid = validator(&last_assistant_text);

                            if !invalid.is_empty() {
                                if !citation_retry_used {
                                    // First failure → re-prompt with feedback.
                                    citation_retry_used = true;
                                    let feedback =
                                        citation_validator::feedback_message(&invalid);
                                    self.history.push(ChatMessage {
                                        role: ChatRole::System,
                                        content: feedback,
                                        tool_call_id: None,
                                        tool_name: None,
                                    });
                                    continue;
                                }
                                // Second failure → warn the user inline.
                                let warning =
                                    citation_validator::warning_prefix(&invalid);
                                if ui_tx
                                    .send(UiEvent::AssistantToken(warning))
                                    .await
                                    .is_err()
                                {
                                    // Frontend closed — surface the channel
                                    // error like every other UI send path.
                                    return Err(ChatError::UiChannelClosed);
                                }
                            }
                        }
                    }

                    let latency_ms = turn_start.elapsed().as_millis() as u64;
                    self.counters.record_turn(
                        latency_ms,
                        accumulated_usage.input_tokens as u64,
                        accumulated_usage.output_tokens as u64,
                    );
                    ui_tx
                        .send(UiEvent::TurnDone {
                            finish_reason: reason,
                            usage: accumulated_usage,
                        })
                        .await
                        .map_err(|_| ChatError::UiChannelClosed)?;
                    return Ok(TurnOutcome {
                        finish_reason: reason,
                        usage: accumulated_usage,
                        tool_rounds,
                        budget_exceeded,
                    });
                }
                Some(FinishReason::Cancelled) => {
                    let latency_ms = turn_start.elapsed().as_millis() as u64;
                    self.counters.record_turn(
                        latency_ms,
                        accumulated_usage.input_tokens as u64,
                        accumulated_usage.output_tokens as u64,
                    );
                    // MIG-048 §E: emit TurnDone for the IPC bridge.
                    ui_tx
                        .send(UiEvent::TurnDone {
                            finish_reason: FinishReason::Cancelled,
                            usage: accumulated_usage,
                        })
                        .await
                        .map_err(|_| ChatError::UiChannelClosed)?;
                    return Ok(TurnOutcome {
                        finish_reason: FinishReason::Cancelled,
                        usage: accumulated_usage,
                        tool_rounds,
                        budget_exceeded,
                    });
                }
                Some(FinishReason::Error) => {
                    self.counters.record_error();
                    let msg = "provider returned Error finish reason".to_string();
                    // MIG-048 §E: surface error to UI bridge BEFORE returning.
                    let _ = ui_tx
                        .send(UiEvent::Error {
                            message: msg.clone(),
                        })
                        .await;
                    return Err(ChatError::Provider(InferenceError::Runtime(msg)));
                }
                None => {
                    self.counters.record_error();
                    // MIG-048 §E: surface error to UI bridge BEFORE returning.
                    let _ = ui_tx
                        .send(UiEvent::Error {
                            message: "provider stream ended without Done event".into(),
                        })
                        .await;
                    return Err(ChatError::StreamEndedWithoutDone);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::orchestrator::dispatcher::CannedDispatcher;
    use crate::mind::provider::{ChatRole, ProviderCapabilities, ToolSchema};
    use crate::mind::providers::LocalStubProvider;

    #[tokio::test]
    async fn turn_completes_with_local_stub_no_tools() {
        let provider: Arc<dyn InferenceProvider> = Arc::new(LocalStubProvider::new());
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(CannedDispatcher);
        let mut orch = ChatOrchestrator::new(provider, dispatcher);

        let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(32);

        let outcome = orch
            .turn("hello".into(), GenParams::default(), ui_tx)
            .await
            .expect("turn ok");

        assert_eq!(outcome.finish_reason, FinishReason::Stop);
        assert_eq!(outcome.tool_rounds, 0);
        assert!(!outcome.budget_exceeded);

        // Drain UI events: 5 AssistantToken + 1 TurnDone.
        let mut tokens = 0;
        let mut turn_done = false;
        while let Some(ev) = ui_rx.recv().await {
            match ev {
                UiEvent::AssistantToken(_) => tokens += 1,
                UiEvent::TurnDone { .. } => {
                    turn_done = true;
                    break;
                }
                other => panic!("unexpected UI event: {other:?}"),
            }
        }
        assert_eq!(tokens, 5);
        assert!(turn_done);
    }

    #[tokio::test]
    async fn turn_completes_one_tool_round_with_local_stub() {
        let provider: Arc<dyn InferenceProvider> = Arc::new(LocalStubProvider::new());
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(CannedDispatcher);
        let mut orch = ChatOrchestrator::new(provider, dispatcher);

        let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(64);
        let params = GenParams {
            tools: vec![ToolSchema {
                name: "search_notes".into(),
                description: "stub".into(),
                input_schema: serde_json::json!({"type": "object"}),
            }],
            ..GenParams::default()
        };

        let outcome = orch
            .turn("search for X".into(), params, ui_tx)
            .await
            .expect("turn ok");

        assert_eq!(outcome.finish_reason, FinishReason::Stop);
        assert_eq!(outcome.tool_rounds, 1);
        assert!(!outcome.budget_exceeded);

        // Sequence: ToolCallProposed → ToolCallResolved → 3 AssistantTokens → TurnDone.
        let mut saw_proposed = false;
        let mut saw_resolved = false;
        let mut tokens = 0;
        let mut turn_done = false;
        while let Some(ev) = ui_rx.recv().await {
            match ev {
                UiEvent::ToolCallProposed { name, .. } => {
                    assert_eq!(name, "search_notes");
                    saw_proposed = true;
                }
                UiEvent::ToolCallResolved { status, .. } => {
                    assert_eq!(status, "ok");
                    saw_resolved = true;
                }
                UiEvent::AssistantToken(_) => tokens += 1,
                UiEvent::TurnDone { .. } => {
                    turn_done = true;
                    break;
                }
                other => panic!("unexpected UI event: {other:?}"),
            }
        }
        assert!(saw_proposed);
        assert!(saw_resolved);
        assert_eq!(tokens, 3);
        assert!(turn_done);
    }

    /// Stub provider that loops ToolCall forever (until budget aborts).
    struct LoopingToolCallProvider;

    #[async_trait::async_trait]
    impl InferenceProvider for LoopingToolCallProvider {
        async fn generate(
            &self,
            messages: &[ChatMessage],
            _params: &GenParams,
        ) -> Result<mpsc::Receiver<StreamEvent>, InferenceError> {
            let (tx, rx) = mpsc::channel(8);
            // Look at the LAST tool result for the "aborted" sentinel —
            // if present, the orchestrator has told us the budget hit, so
            // we should compose a finalizing answer (Stop) instead of
            // looping more ToolCalls.
            let aborted = messages.iter().rev().any(|m| {
                m.role == ChatRole::Tool
                    && m.content.contains("aborted_tool_budget_exceeded")
            });
            tokio::spawn(async move {
                if aborted {
                    let _ = tx
                        .send(StreamEvent::Token {
                            text: "Final answer.".into(),
                        })
                        .await;
                    let _ = tx
                        .send(StreamEvent::Done {
                            finish_reason: FinishReason::Stop,
                            usage: TokenUsage {
                                input_tokens: 1,
                                output_tokens: 1,
                            },
                        })
                        .await;
                } else {
                    let _ = tx
                        .send(StreamEvent::ToolCall {
                            id: "loop".into(),
                            name: "search_notes".into(),
                            args: serde_json::json!({}),
                        })
                        .await;
                    let _ = tx
                        .send(StreamEvent::Done {
                            finish_reason: FinishReason::ToolCall,
                            usage: TokenUsage {
                                input_tokens: 1,
                                output_tokens: 1,
                            },
                        })
                        .await;
                }
            });
            Ok(rx)
        }

        async fn classify(
            &self,
            _text: &str,
            _labels: &[String],
        ) -> Result<Vec<(String, f32)>, InferenceError> {
            Ok(Vec::new())
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                model_id: "looping-stub".into(),
                runtime: "stub".into(),
                max_context_tokens: 4096,
                supports_tool_calls: true,
                supports_citation: true,
            }
        }
    }

    #[tokio::test]
    async fn turn_aborts_on_tool_call_budget_then_finalizes() {
        let provider: Arc<dyn InferenceProvider> = Arc::new(LoopingToolCallProvider);
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(CannedDispatcher);
        let mut orch = ChatOrchestrator::new(provider, dispatcher)
            .with_config(ChatConfig {
                max_tool_rounds_per_turn: 3,
            });

        let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(128);
        let outcome = orch
            .turn("loop forever".into(), GenParams::default(), ui_tx)
            .await
            .expect("turn ok");

        // The budget should fire and the looping stub then finalizes Stop.
        assert!(outcome.budget_exceeded);
        assert_eq!(outcome.tool_rounds, 3);
        assert_eq!(outcome.finish_reason, FinishReason::Stop);

        // Confirm the orchestrator emitted a ToolBudgetReached event.
        let mut saw_budget = false;
        while let Some(ev) = ui_rx.recv().await {
            if matches!(ev, UiEvent::ToolBudgetReached) {
                saw_budget = true;
            }
            if matches!(ev, UiEvent::TurnDone { .. }) {
                break;
            }
        }
        assert!(saw_budget, "expected ToolBudgetReached UI event");
    }

    #[test]
    fn framing_as_tool_result_wraps_in_tool_result_tag_in_phase_1() {
        let raw = serde_json::json!({"status": "ok", "echo": 42});
        let framed = framing::as_tool_result("search_notes", "call_42", raw.clone());
        assert!(framed.starts_with("<tool_result tool=\"search_notes\" id=\"call_42\">"));
        assert!(framed.contains("\"status\":\"ok\""));
        assert!(framed.trim_end().ends_with("</tool_result>"));
    }

    #[tokio::test]
    async fn turn_prepends_system_prompt_when_history_empty() {
        let provider: Arc<dyn InferenceProvider> = Arc::new(LocalStubProvider::new());
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(CannedDispatcher);
        let mut orch = ChatOrchestrator::new(provider, dispatcher);
        let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(32);

        let _ = orch
            .turn("hello".into(), GenParams::default(), ui_tx)
            .await
            .expect("turn ok");
        while ui_rx.recv().await.is_some() {}

        assert!(matches!(
            orch.history().first().map(|m| m.role),
            Some(ChatRole::System)
        ));
        assert!(orch.history()[0].content.contains("CITATION RULE"));
    }

    #[tokio::test]
    async fn turn_increments_per_instance_telemetry_counters() {
        let provider: Arc<dyn InferenceProvider> = Arc::new(LocalStubProvider::new());
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(CannedDispatcher);
        let counters = Arc::new(TelemetryCounters::new());
        let mut orch = ChatOrchestrator::new(provider, dispatcher)
            .with_counters(counters.clone());

        let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(64);
        let params = GenParams {
            tools: vec![ToolSchema {
                name: "search_notes".into(),
                description: "stub".into(),
                input_schema: serde_json::json!({"type": "object"}),
            }],
            ..GenParams::default()
        };
        let outcome = orch
            .turn("first turn".into(), params, ui_tx)
            .await
            .expect("turn ok");
        // Drain remaining UI events so the channel can close cleanly.
        while ui_rx.recv().await.is_some() {}

        let s = counters.snapshot();
        assert_eq!(s.turn_count, 1, "snapshot: {:?}", s);
        assert_eq!(s.tool_calls_count, 1);
        assert_eq!(s.tool_call_rounds_exceeded_count, 0);
        assert_eq!(s.errors_count, 0);
        // LocalProvider stub's round-1 usage is in=10/out=5, round-2 is
        // in=18/out=3 → accumulated 28 / 8.
        assert_eq!(s.tokens_in, 28);
        assert_eq!(s.tokens_out, 8);
        assert_eq!(s.provider_id, "stub");
        assert_eq!(s.model_id, "local-stub");
        assert_eq!(outcome.tool_rounds, 1);
    }

    #[tokio::test]
    async fn budget_abort_records_one_budget_hit_in_telemetry() {
        let provider: Arc<dyn InferenceProvider> = Arc::new(LoopingToolCallProvider);
        let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(CannedDispatcher);
        let counters = Arc::new(TelemetryCounters::new());
        let mut orch = ChatOrchestrator::new(provider, dispatcher)
            .with_config(ChatConfig {
                max_tool_rounds_per_turn: 2,
            })
            .with_counters(counters.clone());

        let (ui_tx, mut ui_rx) = mpsc::channel::<UiEvent>(128);
        let outcome = orch
            .turn("loop".into(), GenParams::default(), ui_tx)
            .await
            .expect("turn ok");
        while ui_rx.recv().await.is_some() {}

        let s = counters.snapshot();
        assert!(outcome.budget_exceeded);
        assert_eq!(s.tool_call_rounds_exceeded_count, 1,
            "exactly one budget hit per turn, even if multiple ToolCalls after");
        assert_eq!(s.tool_calls_count, 2,
            "the two pre-budget ToolCalls were dispatched");
        assert_eq!(s.turn_count, 1);
    }
}

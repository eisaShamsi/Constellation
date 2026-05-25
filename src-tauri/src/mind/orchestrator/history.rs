//! Sliding-window history trim — Eisa-locked decision E2.
//!
//! When the orchestrator's prompt envelope (system + history + tools)
//! would exceed the model's context budget, the OLDEST user/assistant
//! turn pairs are dropped from the prompt-envelope view. The
//! conversation history shown in the UI is preserved verbatim — only
//! what Fanar sees is compacted.
//!
//! ## Why E2 (sliding window) instead of E1 (Fanar-summarized)
//!
//! Eisa overrode the original E1 recommendation. Rationale:
//! - E1 adds latency on overflow (an extra Fanar generate() call to
//!   produce the summary).
//! - E1 risks the summarizer losing information the user expects
//!   Fanar to remember (lossy summarization is invisible).
//! - E2 is sub-millisecond + deterministic. The dropped turns are
//!   still visible in the UI — the user can scroll back and see the
//!   full record. "Visible-but-dropped" beats "invisible lossy."
//!
//! ## Token budget
//!
//! Phase 1 uses a `chars / 4` heuristic for token counting (Fanar's
//! Gemma-2 tokenizer averages ~3.5-4.2 chars/token across English +
//! Arabic). A real tokenizer pass through llama-cpp-2's tokenize API
//! would be more accurate; Phase 1.x will land that if the heuristic
//! proves unreliable in Boss-test Stage 1.
//!
//! ## Edge case: single oversized turn
//!
//! If even after dropping all but the last user/assistant pair the
//! envelope STILL exceeds budget (one turn alone > budget — rare),
//! the trim returns `OversizedTurn` so the orchestrator can surface
//! `StreamEvent::Error("turn exceeds context budget")` rather than
//! silently truncating the user's message.

use crate::mind::provider::{ChatMessage, ChatRole};

/// Default token budget for the prompt envelope. Leaves room for the
/// model's response within Fanar's 8192-token context window:
/// 6500 envelope + 1500 response + ~200 system prompt overhead.
pub const DEFAULT_CONTEXT_BUDGET_TOKENS: usize = 6500;

/// `chars / 4` heuristic — Gemma-2 tokenizer averages ~3.5-4.2
/// chars/token across English + Arabic. Phase 1.x may upgrade to a
/// real tokenizer pass.
fn estimate_tokens(s: &str) -> usize {
    (s.chars().count() / 4).max(1)
}

/// Sum of estimated tokens across every message's content.
pub fn estimate_envelope_tokens(messages: &[ChatMessage]) -> usize {
    messages.iter().map(|m| estimate_tokens(&m.content)).sum()
}

/// Outcome of a trim attempt.
#[derive(Debug, PartialEq)]
pub enum TrimOutcome {
    /// Envelope already fits — nothing was dropped.
    Fits,
    /// Some old turn pairs were dropped. The argument is the new
    /// envelope length (in messages).
    TrimmedTo(usize),
    /// Even after dropping everything but the last user-message + the
    /// system prompt, the envelope still exceeds budget. The
    /// orchestrator should surface an error rather than silently
    /// truncate the user's message.
    OversizedTurn,
}

/// Trim `messages` in place so the estimated token sum is at most
/// `budget`. Drops the oldest user/assistant pairs from the
/// envelope; the System message (if at index 0) is always preserved
/// because the orchestrator relies on the system prompt staying in
/// scope across the conversation.
///
/// Returns the [`TrimOutcome`] so the caller can react to
/// [`TrimOutcome::OversizedTurn`].
pub fn trim_to_budget(messages: &mut Vec<ChatMessage>, budget: usize) -> TrimOutcome {
    if estimate_envelope_tokens(messages) <= budget {
        return TrimOutcome::Fits;
    }

    // Preserve the leading System message — Fanar relies on the system
    // prompt for citation discipline + data-vs-instructions guard.
    let has_system =
        messages.first().map(|m| m.role == ChatRole::System).unwrap_or(false);
    let preserve_idx = if has_system { 1 } else { 0 };

    // Drop turns in user/assistant PAIRS from the front (after the
    // preserved system prompt) until we fit. Tool messages and lone
    // unmatched messages are dropped greedily.
    let mut dropped_any = false;
    while estimate_envelope_tokens(messages) > budget {
        if messages.len() <= preserve_idx + 1 {
            // Only the system prompt + a single message remain.
            // If even that's over budget, the last turn is oversized.
            return TrimOutcome::OversizedTurn;
        }
        // Drop one message from index `preserve_idx`. If it's a User,
        // also drop the next message (likely the Assistant reply for
        // that turn). This keeps user/assistant pairing intact.
        let removed = messages.remove(preserve_idx);
        dropped_any = true;
        if removed.role == ChatRole::User
            && messages.len() > preserve_idx
            && messages[preserve_idx].role == ChatRole::Assistant
        {
            messages.remove(preserve_idx);
        }
    }

    if dropped_any {
        TrimOutcome::TrimmedTo(messages.len())
    } else {
        TrimOutcome::Fits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::provider::ChatMessage;

    fn msg(role: ChatRole, content: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: content.into(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    #[test]
    fn fits_returns_immediately_when_under_budget() {
        let mut history = vec![
            msg(ChatRole::System, "short system prompt"),
            msg(ChatRole::User, "hello"),
            msg(ChatRole::Assistant, "hi"),
        ];
        let outcome = trim_to_budget(&mut history, 1000);
        assert_eq!(outcome, TrimOutcome::Fits);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn trims_oldest_user_assistant_pair_first() {
        let big = "x".repeat(4000); // ≈1000 tokens
        let mut history = vec![
            msg(ChatRole::System, "sys"),
            msg(ChatRole::User, &big),
            msg(ChatRole::Assistant, &big),
            msg(ChatRole::User, &big),
            msg(ChatRole::Assistant, &big),
        ];
        // Budget allows roughly one pair + system. Two pairs = 4000 tokens.
        let outcome = trim_to_budget(&mut history, 1500);
        assert!(matches!(outcome, TrimOutcome::TrimmedTo(_)));
        // System should still be at index 0.
        assert_eq!(history[0].role, ChatRole::System);
        // At least one pair was dropped.
        assert!(history.len() < 5);
    }

    #[test]
    fn preserves_system_prompt_when_trimming() {
        let big = "x".repeat(8000);
        let mut history = vec![
            msg(ChatRole::System, "I am Constellation Mind."),
            msg(ChatRole::User, &big),
            msg(ChatRole::Assistant, &big),
            msg(ChatRole::User, "latest"),
        ];
        let _ = trim_to_budget(&mut history, 100);
        // System must still be present, and at index 0.
        assert!(!history.is_empty());
        assert_eq!(history[0].role, ChatRole::System);
        assert!(history[0].content.contains("Constellation Mind"));
    }

    #[test]
    fn surfaces_oversized_turn_when_single_message_exceeds_budget() {
        let huge = "x".repeat(40_000); // ≈10000 tokens
        let mut history = vec![
            msg(ChatRole::System, "sys"),
            msg(ChatRole::User, &huge),
        ];
        let outcome = trim_to_budget(&mut history, 1000);
        assert_eq!(outcome, TrimOutcome::OversizedTurn);
    }

    #[test]
    fn handles_history_without_system_prompt() {
        let big = "x".repeat(4000);
        let mut history = vec![
            msg(ChatRole::User, &big),
            msg(ChatRole::Assistant, &big),
            msg(ChatRole::User, &big),
            msg(ChatRole::Assistant, &big),
        ];
        let outcome = trim_to_budget(&mut history, 1500);
        assert!(matches!(outcome, TrimOutcome::TrimmedTo(_)));
        assert!(history.len() < 4);
    }

    #[test]
    fn estimate_envelope_tokens_sums_across_messages() {
        let h = vec![
            msg(ChatRole::User, &"x".repeat(40)), // 10 tokens
            msg(ChatRole::Assistant, &"y".repeat(80)), // 20 tokens
        ];
        assert_eq!(estimate_envelope_tokens(&h), 30);
    }
}

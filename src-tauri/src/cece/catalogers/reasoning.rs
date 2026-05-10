//! MIG-021v3 V3-§7 — Reasoning Cataloger.
//!
//! The "expensive but smart" cataloger. Calls a local LLM (planned:
//! Qwen3-4B-Instruct-2507 Q5_K_M GGUF via llama.cpp) with the prompt
//! built by `reasoning_prompt::build_full_prompt()` and the GBNF
//! grammar from `reasoning_prompt::build_gbnf_grammar()` to constrain
//! output to valid taxonomy IDs only.
//!
//! Per Architect §2.5 + §10 invariant 4 (LOCAL-ONLY):
//!   * Local Qwen3-4B Q5_K_M only — notes never leave the device
//!   * No cloud track, no opt-in
//!   * Lazy-load GGUF on first use
//!   * Two-step decomposition (parent first, then child)
//!   * GBNF grammar guarantees parseable output
//!
//! Honest scope of THIS commit (V3-§7):
//!   * Cataloger interface + injectable inference function
//!   * Prompt builder + GBNF grammar (in reasoning_prompt.rs)
//!   * JSON response parser
//!   * Abstain when no inference fn is wired
//!
//! Wired in V3-§7.b or V3-§8 orchestrator wiring (deferred from this
//! commit because adding `llama-cpp-2` to Cargo.toml is a Plan §13
//! medium-likelihood / high-impact risk on Windows toolchain
//! compatibility — surfacing it as its own focused commit lets us
//! catch the build break early without blocking the cataloger
//! interface from landing).

use crate::cece::cataloger::{
    Axis, AxisAssignment, Cataloger, CatalogerContext, Confidence, ReasoningTrail,
    RejectedAlternative,
};
use crate::cece::catalogers::reasoning_prompt::{build_full_prompt, build_gbnf_grammar};
use crate::sources::{is_valid_content_type_id, is_valid_source_id};
use serde::Deserialize;

/// Inference function signature. Takes the full prompt + the GBNF
/// grammar; returns the LLM's grammar-constrained JSON output (a
/// string that matches the schema in build_gbnf_grammar).
///
/// The grammar guarantees parseable JSON when honored. The default
/// (no fn wired) path returns abstain.
pub type InferenceFn = Box<
    dyn Fn(&str, &str) -> Result<String, String> + Send + Sync + 'static,
>;

pub struct ReasoningCataloger {
    inference_fn: Option<InferenceFn>,
}

impl ReasoningCataloger {
    pub fn new() -> Self {
        Self { inference_fn: None }
    }

    pub fn with_inference(inference: InferenceFn) -> Self {
        Self {
            inference_fn: Some(inference),
        }
    }
}

impl Default for ReasoningCataloger {
    fn default() -> Self {
        Self::new()
    }
}

impl Cataloger for ReasoningCataloger {
    fn name(&self) -> &'static str {
        "reasoning"
    }

    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
        let Some(infer) = &self.inference_fn else {
            // Engine not wired (no GGUF downloaded yet, OR llama-cpp-2
            // dep not yet linked, OR test-only path). Abstain
            // gracefully — the orchestrator will fall back to whatever
            // the cheaper catalogers produced.
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Reasoning engine not loaded (model not downloaded or engine not initialized).",
            ));
        };

        let prompt = build_full_prompt(&ctx.note_path, &ctx.content);
        let grammar = build_gbnf_grammar();

        let raw = match infer(&prompt, &grammar) {
            Ok(s) => s,
            Err(e) => {
                return Some(ReasoningTrail::abstain(
                    self.name(),
                    &format!("Reasoning inference failed: {}", e),
                ));
            }
        };

        // Parse the grammar-constrained JSON.
        let parsed: ReasoningResponse = match serde_json::from_str(&raw) {
            Ok(p) => p,
            Err(e) => {
                return Some(ReasoningTrail::abstain(
                    self.name(),
                    &format!(
                        "Reasoning output failed JSON parse (grammar misalignment?): {}",
                        e
                    ),
                ));
            }
        };

        // Defense-in-depth: even though the grammar should constrain
        // outputs to valid IDs, validate again before trusting.
        let horizontal: Vec<AxisAssignment> = parsed
            .horizontal
            .iter()
            .filter(|id| is_valid_source_id(id))
            .enumerate()
            .map(|(i, id)| AxisAssignment {
                id: id.clone(),
                primary: i == 0,
                weight: 0.85,
                descend_uncertain: false,
            })
            .collect();

        let vertical: Vec<AxisAssignment> = parsed
            .vertical
            .iter()
            .filter(|id| is_valid_content_type_id(id))
            .enumerate()
            .map(|(i, id)| AxisAssignment {
                id: id.clone(),
                primary: i == 0,
                weight: 0.85,
                descend_uncertain: false,
            })
            .collect();

        if horizontal.is_empty() && vertical.is_empty() {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Reasoning output had no valid taxonomy IDs after validation.",
            ));
        }

        let alternatives: Vec<RejectedAlternative> = parsed
            .alternatives_considered
            .into_iter()
            .filter_map(|a| {
                let axis = if is_valid_source_id(&a.id) {
                    Axis::Horizontal
                } else if is_valid_content_type_id(&a.id) {
                    Axis::Vertical
                } else {
                    return None;
                };
                Some(RejectedAlternative {
                    axis,
                    id: a.id,
                    rejected_because: a.reason,
                })
            })
            .collect();

        Some(ReasoningTrail {
            cataloger: self.name().to_string(),
            voiced_opinion: true,
            horizontal,
            vertical,
            reasoning: parsed.reasoning,
            rules_fired: vec![
                "schedule_navigation_top_down".to_string(),
                "gbnf_constrained".to_string(),
                "rule_of_application".to_string(),
            ],
            alternatives_considered: alternatives,
            self_reported_confidence: Confidence::High,
        })
    }

    fn supported_axes(&self) -> &[Axis] {
        &[Axis::Horizontal, Axis::Vertical]
    }
}

// JSON response shape (matches the GBNF grammar in reasoning_prompt.rs).
#[derive(Debug, Deserialize)]
struct ReasoningResponse {
    horizontal: Vec<String>,
    vertical: Vec<String>,
    reasoning: String,
    alternatives_considered: Vec<ReasoningAlternative>,
}

#[derive(Debug, Deserialize)]
struct ReasoningAlternative {
    id: String,
    reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> CatalogerContext {
        CatalogerContext::new(
            "test.md".to_string(),
            "some note content".to_string(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn no_inference_wired_abstains() {
        let c = ReasoningCataloger::new();
        let trail = c.classify(&ctx()).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("not loaded"));
    }

    #[test]
    fn inference_returning_valid_json_voices() {
        let infer = Box::new(|_p: &str, _g: &str| {
            Ok(r#"{
                "horizontal": ["testimony/scriptural"],
                "vertical": ["semantic-contents/proposition"],
                "reasoning": "Hadith citation present.",
                "alternatives_considered": [
                    {"id": "mass-transmission/verbal", "reason": "single chain, not mutawātir"}
                ]
            }"#
            .to_string())
        });
        let c = ReasoningCataloger::with_inference(infer);
        let trail = c.classify(&ctx()).unwrap();
        assert!(trail.voiced_opinion);
        assert_eq!(trail.horizontal[0].id, "testimony/scriptural");
        assert_eq!(trail.vertical[0].id, "semantic-contents/proposition");
        assert_eq!(trail.alternatives_considered.len(), 1);
        assert_eq!(trail.self_reported_confidence, Confidence::High);
    }

    #[test]
    fn invalid_taxonomy_ids_filtered() {
        let infer = Box::new(|_p: &str, _g: &str| {
            Ok(r#"{
                "horizontal": ["bogus_id_1", "testimony/scriptural", "bogus_id_2"],
                "vertical": ["semantic-contents/proposition"],
                "reasoning": "Mostly nonsense, one valid.",
                "alternatives_considered": []
            }"#
            .to_string())
        });
        let c = ReasoningCataloger::with_inference(infer);
        let trail = c.classify(&ctx()).unwrap();
        assert_eq!(trail.horizontal.len(), 1);
        assert_eq!(trail.horizontal[0].id, "testimony/scriptural");
    }

    #[test]
    fn malformed_json_abstains() {
        let infer = Box::new(|_p: &str, _g: &str| Ok("not json at all".to_string()));
        let c = ReasoningCataloger::with_inference(infer);
        let trail = c.classify(&ctx()).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("JSON parse"));
    }

    #[test]
    fn inference_failure_abstains() {
        let infer = Box::new(|_p: &str, _g: &str| Err("model crashed".to_string()));
        let c = ReasoningCataloger::with_inference(infer);
        let trail = c.classify(&ctx()).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("model crashed"));
    }

    #[test]
    fn all_invalid_ids_abstains() {
        let infer = Box::new(|_p: &str, _g: &str| {
            Ok(r#"{
                "horizontal": ["bogus_1", "bogus_2"],
                "vertical": ["bogus_3"],
                "reasoning": "All garbage.",
                "alternatives_considered": []
            }"#
            .to_string())
        });
        let c = ReasoningCataloger::with_inference(infer);
        let trail = c.classify(&ctx()).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("no valid taxonomy IDs"));
    }
}

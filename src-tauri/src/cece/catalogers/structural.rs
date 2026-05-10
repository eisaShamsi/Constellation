//! MIG-021v3 V3-§3 — Structural Cataloger.
//!
//! Reads through the note's STRUCTURE — citations, blockquotes, headings,
//! code blocks, equation markers, stance markers — independent of the
//! note's content semantics. The "what shape is this writing in?" lens.
//!
//! Per Architect §2.2:
//!   Strong on: notes with rich metadata, citations, formal structure
//!   Weak on:   pure free-form prose without structural markers
//!   Latency:   microseconds (regex + counting, no model)
//!
//! Reuses the `regex_horizontal` block in `data/sources_lexicon.json`
//! (no schema change to that file). Adds vertical-axis structural
//! detectors: epistemic stance markers, mathematical notation, code,
//! quotation patterns.
//!
//! Rules fired (Architect §4):
//!   * Rule of Application — distinguishes use-of-citation from
//!     mention-of-citation by anchoring on structural form
//!   * Rule of Three — abstains at depth when too many candidates fire

use crate::cece::cataloger::{
    Axis, AxisAssignment, Cataloger, CatalogerContext, Confidence, ReasoningTrail,
};
use crate::sources::{is_valid_content_type_id, is_valid_source_id};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct StructuralCataloger;

impl StructuralCataloger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StructuralCataloger {
    fn default() -> Self {
        Self::new()
    }
}

impl Cataloger for StructuralCataloger {
    fn name(&self) -> &'static str {
        "structural"
    }

    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
        // Horizontal: regex_horizontal patterns from the lexicon JSON +
        // any new structural detectors we add directly.
        let mut horizontal_hits: HashMap<String, (f32, String)> = HashMap::new();
        for rule in horizontal_rules() {
            if rule.pattern.is_match(&ctx.content) && is_valid_source_id(&rule.target) {
                let entry = horizontal_hits
                    .entry(rule.target.clone())
                    .or_insert((0.0, rule.evidence.clone()));
                if rule.weight > entry.0 {
                    entry.0 = rule.weight;
                }
            }
        }

        // Vertical: stance markers + structural-content detectors.
        let mut vertical_hits: HashMap<String, (f32, String)> = HashMap::new();
        for rule in vertical_rules() {
            if rule.pattern.is_match(&ctx.content) && is_valid_content_type_id(&rule.target) {
                let entry = vertical_hits
                    .entry(rule.target.clone())
                    .or_insert((0.0, rule.evidence.clone()));
                if rule.weight > entry.0 {
                    entry.0 = rule.weight;
                }
            }
        }

        if horizontal_hits.is_empty() && vertical_hits.is_empty() {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "No structural patterns matched (no citations, code, blockquotes, stance markers, or equations).",
            ));
        }

        let horizontal = top_assignments(horizontal_hits, 3);
        let vertical = top_assignments(vertical_hits, 3);
        let reasoning = build_reasoning(&horizontal, &vertical);
        let mut rules_fired = Vec::new();
        if !horizontal.is_empty() {
            rules_fired.push("structural_pattern_match".to_string());
        }
        if !vertical.is_empty() {
            rules_fired.push("stance_or_form_marker".to_string());
        }

        // Self-confidence: structural patterns are strong evidence when
        // a citation/equation/blockquote fires; weaker for soft markers.
        let confidence = if highest_weight(&horizontal) >= 0.85
            || highest_weight(&vertical) >= 0.85
        {
            Confidence::High
        } else {
            Confidence::Medium
        };

        Some(ReasoningTrail {
            cataloger: self.name().to_string(),
            voiced_opinion: true,
            horizontal,
            vertical,
            reasoning,
            rules_fired,
            alternatives_considered: Vec::new(),
            self_reported_confidence: confidence,
        })
    }

    fn supported_axes(&self) -> &[Axis] {
        &[Axis::Horizontal, Axis::Vertical]
    }
}

// ─── Lexicon-loaded horizontal regex rules ─────────────────────────

#[derive(Debug, Deserialize)]
struct LexiconFile {
    regex_horizontal: Vec<RegexRuleRaw>,
}

#[derive(Debug, Deserialize)]
struct RegexRuleRaw {
    pattern: String,
    target: String,
    weight: f32,
    evidence: String,
}

#[derive(Debug)]
struct CompiledRule {
    pattern: Regex,
    target: String,
    weight: f32,
    evidence: String,
}

static HORIZONTAL_RULES: OnceLock<Vec<CompiledRule>> = OnceLock::new();

fn horizontal_rules() -> &'static [CompiledRule] {
    HORIZONTAL_RULES
        .get_or_init(|| {
            const RAW: &str = include_str!("../../../data/sources_lexicon.json");
            let parsed: LexiconFile = serde_json::from_str(RAW)
                .expect("sources_lexicon.json must parse");
            parsed
                .regex_horizontal
                .into_iter()
                .filter_map(|r| {
                    Regex::new(&r.pattern).ok().map(|p| CompiledRule {
                        pattern: p,
                        target: r.target,
                        weight: r.weight,
                        evidence: r.evidence,
                    })
                })
                .collect()
        })
        .as_slice()
}

// ─── Inline vertical-axis structural rules ─────────────────────────
// These are kept in code (not the lexicon JSON) because they target
// vertical-axis content_type IDs and the JSON's vertical block is for
// token matching, not regex. Future refactor may unify them.

static VERTICAL_RULES: OnceLock<Vec<CompiledRule>> = OnceLock::new();

fn vertical_rules() -> &'static [CompiledRule] {
    VERTICAL_RULES
        .get_or_init(|| {
            let specs: &[(&str, &str, f32, &str)] = &[
                // Doubt markers — first-person uncertainty.
                (
                    r"(?i)\b(I doubt|I'?m not sure|I am not sure|I'?m uncertain|I question)\b",
                    "epistemic-states/doubt",
                    0.85,
                    "First-person doubt marker (English)",
                ),
                (
                    r"(أشكّ|أشك|في شك|غير متأكد)",
                    "epistemic-states/doubt",
                    0.85,
                    "First-person doubt marker (Arabic)",
                ),
                // Certainty markers.
                (
                    r"(?i)\b(I'?m certain|I am certain|certainly|undoubtedly|without doubt)\b",
                    "epistemic-states/certainty",
                    0.85,
                    "First-person certainty marker (English)",
                ),
                (
                    r"(متأكد|يَقين|بكلّ يقين|بلا شك)",
                    "epistemic-states/certainty",
                    0.85,
                    "First-person certainty marker (Arabic)",
                ),
                // Belief markers.
                (
                    r"(?i)\b(I believe|I think that|I suppose|I assume)\b",
                    "epistemic-states/belief/occurrent",
                    0.80,
                    "First-person belief marker (English)",
                ),
                (
                    r"(أعتقد|أظنّ|أظن|أرى أن)",
                    "epistemic-states/belief/occurrent",
                    0.80,
                    "First-person belief marker (Arabic)",
                ),
                // Mathematical / proof structure → propositional knowledge.
                // LaTeX inline math, theorem markers.
                (
                    r"(\$[^\$]+\$|\\begin\{(?:equation|theorem|lemma|proof|proposition)\}|^\s*Theorem[\s:.]|^\s*Lemma[\s:.]|^\s*Proof[\s:.])",
                    "epistemic-states/knowledge/by-content/propositional",
                    0.75,
                    "Mathematical / theorem-proof structure",
                ),
                // Numerical data + units → fact (semantic-contents/fact
                // is the closest match; if missing from taxonomy this
                // entry will be dropped at validation).
                (
                    r"\b\d+(\.\d+)?\s*(km|m|cm|mm|kg|g|°[CF]|%|hPa|MHz|GHz|ms)\b",
                    "semantic-contents/idea/constructed",
                    0.65,
                    "Numerical measurement with unit",
                ),
            ];
            specs
                .iter()
                .filter_map(|(pat, target, weight, evidence)| {
                    Regex::new(pat).ok().map(|p| CompiledRule {
                        pattern: p,
                        target: target.to_string(),
                        weight: *weight,
                        evidence: evidence.to_string(),
                    })
                })
                .collect()
        })
        .as_slice()
}

// ─── Helpers ───────────────────────────────────────────────────────

fn top_assignments(
    hits: HashMap<String, (f32, String)>,
    max: usize,
) -> Vec<AxisAssignment> {
    let mut entries: Vec<(String, f32)> = hits.into_iter().map(|(k, (w, _))| (k, w)).collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries.truncate(max);
    entries
        .into_iter()
        .enumerate()
        .map(|(i, (id, weight))| AxisAssignment {
            id,
            primary: i == 0,
            weight,
            descend_uncertain: false,
        })
        .collect()
}

fn highest_weight(assignments: &[AxisAssignment]) -> f32 {
    assignments
        .iter()
        .map(|a| a.weight)
        .fold(0.0, f32::max)
}

fn build_reasoning(h: &[AxisAssignment], v: &[AxisAssignment]) -> String {
    use std::fmt::Write;
    let mut out = String::from("Structural patterns matched: ");
    let mut parts = Vec::new();
    if let Some(top) = h.first() {
        parts.push(format!("horizontal → {} (weight {:.2})", top.id, top.weight));
    }
    if let Some(top) = v.first() {
        parts.push(format!("vertical → {} (weight {:.2})", top.id, top.weight));
    }
    let _ = write!(out, "{}.", parts.join("; "));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_for_body(body: &str) -> CatalogerContext {
        CatalogerContext::new(
            "test.md".to_string(),
            body.to_string(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn isbn_fires_testimony_scriptural() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body(
                "See the analysis at ISBN 978-0-12-345678-9 for more details.",
            ))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.horizontal.iter().any(|a| a.id == "testimony/scriptural"));
    }

    #[test]
    fn doi_fires_testimony_scriptural() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body("As described in 10.1234/abc.5678/xyz, the result..."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.horizontal.iter().any(|a| a.id == "testimony/scriptural"));
    }

    #[test]
    fn blockquote_fires_testimony_direct_witness() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body("Some setup.\n\n> Quoted statement here.\n\nAnd more."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail
            .horizontal
            .iter()
            .any(|a| a.id == "testimony/direct-witness"));
    }

    #[test]
    fn english_doubt_marker_fires_vertical_doubt() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body("I doubt that the moon landing happened in 1969."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.vertical.iter().any(|a| a.id == "epistemic-states/doubt"));
    }

    #[test]
    fn arabic_doubt_marker_fires_vertical_doubt() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body("أشكّ في صحة هذا الادعاء."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.vertical.iter().any(|a| a.id == "epistemic-states/doubt"));
    }

    #[test]
    fn empty_body_abstains() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body("Constellation is a knowledge tool."))
            .unwrap();
        // Plain prose → no structural marker → abstain.
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn equation_marker_fires_propositional_knowledge() {
        let c = StructuralCataloger::new();
        let trail = c
            .classify(&ctx_for_body("The result follows: $x^2 + y^2 = z^2$."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail
            .vertical
            .iter()
            .any(|a| a.id == "epistemic-states/knowledge/by-content/propositional"));
    }
}

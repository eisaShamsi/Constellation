//! MIG-021v3 V3-§2 — User-Authority Cataloger.
//!
//! The simplest cataloger by far. Reads frontmatter only; voices an
//! opinion when the YAML carries explicit `sources:` or `content_type:`
//! values; abstains otherwise.
//!
//! Per Architect §2.6 + invariant 1: this cataloger has absolute
//! precedence — when it voices, the synthesis layer short-circuits
//! and accepts its assignment as-is. Other catalogers still run (their
//! reasoning trails are preserved for audit + future active learning),
//! but they don't change the outcome.
//!
//! Architect mapping:
//!   - Cataloger trait → impl below
//!   - Rule of Authority → fires on every voiced classification
//!   - CIP precedent (Agent 2 library-science research): author-supplied
//!     metadata outperforms post-hoc cataloging at depth. The user is
//!     the author of their own notes.

use crate::cece::cataloger::{
    AxisAssignment, Axis, Cataloger, CatalogerContext, Confidence, ReasoningTemplate,
    ReasoningTrail,
};
use crate::sources::{is_valid_content_type_id, is_valid_source_id};
use serde_json::json;

pub struct UserAuthorityCataloger;

impl UserAuthorityCataloger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UserAuthorityCataloger {
    fn default() -> Self {
        Self::new()
    }
}

impl Cataloger for UserAuthorityCataloger {
    fn name(&self) -> &'static str {
        "user_authority"
    }

    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
        let h_empty = ctx.frontmatter_sources.is_empty();
        let v_empty = ctx.frontmatter_content_type.is_empty();

        // Both axes empty → abstain. Synthesis ignores; other catalogers
        // get the floor.
        if h_empty && v_empty {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Note has no `sources:` or `content_type:` in frontmatter.",
            ));
        }

        // Validate IDs defensively. An invalid ID slipped past §1A'
        // validation is treated as if absent — we never echo a bogus
        // value as authoritative.
        let horizontal: Vec<AxisAssignment> = ctx
            .frontmatter_sources
            .iter()
            .filter(|id| is_valid_source_id(id))
            .enumerate()
            .map(|(i, id)| AxisAssignment {
                id: id.clone(),
                primary: i == 0,
                weight: 1.0,
                descend_uncertain: false,
            })
            .collect();

        let vertical: Vec<AxisAssignment> = ctx
            .frontmatter_content_type
            .iter()
            .filter(|id| is_valid_content_type_id(id))
            .enumerate()
            .map(|(i, id)| AxisAssignment {
                id: id.clone(),
                primary: i == 0,
                weight: 1.0,
                descend_uncertain: false,
            })
            .collect();

        // After validation, both empty → still abstain (the YAML had
        // values but none were valid taxonomy IDs).
        if horizontal.is_empty() && vertical.is_empty() {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Frontmatter values present but none match a known taxonomy ID.",
            ));
        }

        let (reasoning, reasoning_template) = build_reasoning(&horizontal, &vertical);

        Some(ReasoningTrail {
            cataloger: self.name().to_string(),
            voiced_opinion: true,
            horizontal,
            vertical,
            reasoning,
            reasoning_template: Some(reasoning_template),
            rules_fired: vec!["rule_of_authority".to_string()],
            alternatives_considered: Vec::new(),
            self_reported_confidence: Confidence::High,
        })
    }

    fn supported_axes(&self) -> &[Axis] {
        &[Axis::Horizontal, Axis::Vertical]
    }
}

/// MIG-022 §E.2 (PJ-041) — emits the English fallback string AND the
/// structured i18n template. Three template variants based on which
/// axes UA voiced:
///   - both axes  → cece.reasoning.user_authority.both
///   - h only     → cece.reasoning.user_authority.horizontal_only
///   - v only     → cece.reasoning.user_authority.vertical_only
fn build_reasoning(
    h: &[AxisAssignment],
    v: &[AxisAssignment],
) -> (String, ReasoningTemplate) {
    use std::fmt::Write;
    let h_joined = h.iter().map(|a| a.id.as_str()).collect::<Vec<_>>().join(", ");
    let v_joined = v.iter().map(|a| a.id.as_str()).collect::<Vec<_>>().join(", ");

    // English fallback string — preserves pre-MIG-022 behavior.
    let mut english = String::from("Set in note frontmatter (manual). ");
    if !h.is_empty() {
        let _ = write!(english, "Sources: {}. ", h_joined);
    }
    if !v.is_empty() {
        let _ = write!(english, "Content type: {}. ", v_joined);
    }

    // Template variant + params per axis combination.
    let template = match (!h.is_empty(), !v.is_empty()) {
        (true, true) => ReasoningTemplate {
            key: "user_authority.both".to_string(),
            params: json!({ "sources": h_joined, "content_type": v_joined }),
        },
        (true, false) => ReasoningTemplate {
            key: "user_authority.horizontal_only".to_string(),
            params: json!({ "sources": h_joined }),
        },
        (false, true) => ReasoningTemplate {
            key: "user_authority.vertical_only".to_string(),
            params: json!({ "content_type": v_joined }),
        },
        // Unreachable: caller already checks both empty → abstain.
        // Guard with a no-axes template just in case future refactors
        // change call shape.
        (false, false) => ReasoningTemplate {
            key: "user_authority.both".to_string(),
            params: json!({ "sources": "", "content_type": "" }),
        },
    };

    (english, template)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(h: Vec<&str>, v: Vec<&str>) -> CatalogerContext {
        CatalogerContext::new(
            "test.md".to_string(),
            String::new(),
            h.into_iter().map(String::from).collect(),
            v.into_iter().map(String::from).collect(),
        )
    }

    #[test]
    fn empty_frontmatter_abstains() {
        let c = UserAuthorityCataloger::new();
        let trail = c.classify(&ctx(Vec::new(), Vec::new())).unwrap();
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn populated_horizontal_voices_high_confidence() {
        let c = UserAuthorityCataloger::new();
        let trail = c.classify(&ctx(vec!["testimony"], Vec::new())).unwrap();
        assert!(trail.voiced_opinion);
        assert_eq!(trail.horizontal.len(), 1);
        assert_eq!(trail.horizontal[0].id, "testimony");
        assert!(trail.horizontal[0].primary);
        assert_eq!(trail.self_reported_confidence, Confidence::High);
        assert_eq!(trail.rules_fired, vec!["rule_of_authority"]);
    }

    #[test]
    fn invalid_ids_dropped() {
        let c = UserAuthorityCataloger::new();
        let trail = c
            .classify(&ctx(vec!["testimony", "not_a_real_id"], Vec::new()))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert_eq!(trail.horizontal.len(), 1);
        assert_eq!(trail.horizontal[0].id, "testimony");
    }

    #[test]
    fn all_invalid_ids_abstains() {
        let c = UserAuthorityCataloger::new();
        let trail = c
            .classify(&ctx(vec!["bogus_1", "bogus_2"], vec!["also_bogus"]))
            .unwrap();
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn first_id_marked_primary() {
        let c = UserAuthorityCataloger::new();
        let trail = c
            .classify(&ctx(vec!["testimony", "perception"], Vec::new()))
            .unwrap();
        assert!(trail.horizontal[0].primary);
        assert!(!trail.horizontal[1].primary);
    }
}

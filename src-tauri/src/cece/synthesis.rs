//! MIG-021v3 V3-§1 — Synthesis layer (weighted vote on Day 1).
//!
//! Takes 1–6 reasoning trails from the orchestrator and produces a
//! single composite assignment + composite reasoning trail per
//! Architect §3.
//!
//! Three confidence regimes per axis:
//!   * Unanimous — all voicing catalogers agree on the primary
//!   * Strong Majority — supermajority agree; dissenter surfaced as "see also"
//!   * Split — refuses to assign; triggers Sibling Disambiguation UI
//!
//! Per Architect §3.2: weighted vote on Day 1; Snorkel-style learned
//! synthesis deferred to MIG-022.
//!
//! Per Architect §10 invariant 1: when User-Authority Cataloger voices
//! an opinion, it overrides everything. The synthesis layer has a hard
//! early-return for this case.

use crate::cece::cataloger::{AxisAssignment, ReasoningTrail, Confidence, Axis};
use crate::cece::reliability::{weight_for, ReliabilityProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One of three confidence regimes per axis. Drives UI behavior:
/// Unanimous accepts silently; StrongMajority surfaces a dissent;
/// Split refuses and triggers Sibling Disambiguation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceRegime {
    Unanimous,
    StrongMajority,
    Split,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisDecision {
    pub primary: Option<String>,
    pub secondary: Vec<String>,
    pub regime: ConfidenceRegime,
    /// Candidates the synthesis surfaces below the primary — neighboring
    /// leaves the user might want to know about.
    pub see_also: Vec<String>,
    /// When `regime == Split`, the candidates the user must pick from.
    /// `None` for Unanimous and StrongMajority.
    pub needs_user_disambiguation_between: Option<Vec<String>>,
    /// When `regime == StrongMajority`, the cataloger that dissented.
    pub dissenter: Option<String>,
    /// MIG-021v3 V3-§8 fix-A — actual ensemble vote weight for the
    /// primary, normalized to [0, 1]. Replaces the hardcoded 0.85
    /// constant the IPC was emitting before. Reflects how strongly
    /// the catalogers' weighted vote favored this leaf.
    #[serde(default)]
    pub primary_weight: f32,
    /// Vote weights for `see_also` entries, in the same order as
    /// `see_also`. Empty when see_also is empty.
    #[serde(default)]
    pub see_also_weights: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeAssignment {
    pub horizontal: AxisDecision,
    pub vertical: AxisDecision,
    /// One paragraph synthesizing the per-cataloger trails into a
    /// user-facing explanation.
    pub composite_reasoning: String,
    pub catalogers_voiced: Vec<String>,
    pub catalogers_silent: Vec<String>,
    /// "weighted_vote" today; "snorkel" in MIG-022.
    pub synthesis_method: String,
    /// Full per-cataloger trails, preserved for audit + reasoning-trail UI.
    pub per_cataloger_trails: Vec<ReasoningTrail>,
}

/// Synthesize one composite assignment from a set of cataloger trails.
/// Per Architect §3.1 + invariant 1.
pub fn synthesize(
    trails: Vec<ReasoningTrail>,
    reliability: &ReliabilityProfile,
) -> CompositeAssignment {
    // Split into voiced / silent up front for the composite output.
    let (voiced, silent): (Vec<_>, Vec<_>) = trails
        .iter()
        .partition(|t| t.voiced_opinion);

    let catalogers_voiced: Vec<String> = voiced.iter().map(|t| t.cataloger.clone()).collect();
    let catalogers_silent: Vec<String> = silent.iter().map(|t| t.cataloger.clone()).collect();

    // ── Invariant 1: User-Authority short-circuit ──
    // When User-Authority voiced, its assignment is the ensemble assignment.
    // Other catalogers' trails are still preserved (for audit + future
    // active-learning signal) but don't influence the decision.
    if let Some(ua) = voiced.iter().find(|t| t.cataloger == "user_authority") {
        return user_authority_short_circuit(
            ua,
            trails.clone(),
            catalogers_voiced,
            catalogers_silent,
        );
    }

    // ── Per-axis weighted vote ──
    let horizontal = vote_on_axis(&voiced, Axis::Horizontal, reliability);
    let vertical = vote_on_axis(&voiced, Axis::Vertical, reliability);

    let composite_reasoning =
        compose_reasoning(&voiced, &horizontal, &vertical);

    CompositeAssignment {
        horizontal,
        vertical,
        composite_reasoning,
        catalogers_voiced,
        catalogers_silent,
        synthesis_method: "weighted_vote".to_string(),
        per_cataloger_trails: trails,
    }
}

fn user_authority_short_circuit(
    ua: &ReasoningTrail,
    all_trails: Vec<ReasoningTrail>,
    catalogers_voiced: Vec<String>,
    catalogers_silent: Vec<String>,
) -> CompositeAssignment {
    let h = AxisDecision {
        primary: ua.horizontal.iter().find(|a| a.primary).map(|a| a.id.clone()),
        secondary: ua
            .horizontal
            .iter()
            .filter(|a| !a.primary)
            .map(|a| a.id.clone())
            .collect(),
        regime: ConfidenceRegime::Unanimous,
        see_also: Vec::new(),
        needs_user_disambiguation_between: None,
        dissenter: None,
        // User-supplied frontmatter is the authoritative answer — full
        // confidence (1.0) per Architect §2.6 + invariant 1.
        primary_weight: 1.0,
        see_also_weights: Vec::new(),
    };
    let v = AxisDecision {
        primary: ua.vertical.iter().find(|a| a.primary).map(|a| a.id.clone()),
        secondary: ua
            .vertical
            .iter()
            .filter(|a| !a.primary)
            .map(|a| a.id.clone())
            .collect(),
        regime: ConfidenceRegime::Unanimous,
        see_also: Vec::new(),
        needs_user_disambiguation_between: None,
        dissenter: None,
        primary_weight: 1.0,
        see_also_weights: Vec::new(),
    };
    CompositeAssignment {
        horizontal: h,
        vertical: v,
        composite_reasoning: format!("Set by user in frontmatter ({}).", ua.reasoning),
        catalogers_voiced,
        catalogers_silent,
        synthesis_method: "user_authority_short_circuit".to_string(),
        per_cataloger_trails: all_trails,
    }
}

/// Per-axis weighted vote across voicing catalogers' assignments.
///
/// Algorithm:
///   1. For each cataloger that voiced, take its primary assignment on
///      this axis (skip if no primary on this axis — cataloger had nothing
///      to say about this axis specifically).
///   2. Multiply its weight by reliability_weight(cataloger, axis) *
///      assignment.weight * confidence_multiplier(self_reported_confidence).
///   3. Sum weights per candidate ID.
///   4. The candidate with the highest weighted sum is the primary.
///   5. Compute regime by counting catalogers that voted for the primary
///      vs catalogers that dissented (voted for something else).
fn vote_on_axis(
    voiced: &[&ReasoningTrail],
    axis: Axis,
    reliability: &ReliabilityProfile,
) -> AxisDecision {
    let mut weighted_votes: HashMap<String, f32> = HashMap::new();
    let mut who_voted_for: HashMap<String, Vec<String>> = HashMap::new();

    for trail in voiced {
        let assignments = match axis {
            Axis::Horizontal => &trail.horizontal,
            Axis::Vertical => &trail.vertical,
        };
        // Use the primary assignment on this axis; skip if cataloger
        // had nothing to say about this axis.
        let Some(primary_assignment) = assignments.iter().find(|a| a.primary) else {
            continue;
        };
        let w = weight_for(reliability, &trail.cataloger, axis)
            * primary_assignment.weight
            * confidence_multiplier(trail.self_reported_confidence);
        *weighted_votes.entry(primary_assignment.id.clone()).or_insert(0.0) += w;
        who_voted_for
            .entry(primary_assignment.id.clone())
            .or_default()
            .push(trail.cataloger.clone());
    }

    // No voicing on this axis at all.
    if weighted_votes.is_empty() {
        return AxisDecision {
            primary: None,
            secondary: Vec::new(),
            regime: ConfidenceRegime::Unanimous, // vacuously
            see_also: Vec::new(),
            needs_user_disambiguation_between: None,
            dissenter: None,
            primary_weight: 0.0,
            see_also_weights: Vec::new(),
        };
    }

    // Sort candidates by weighted vote descending.
    let mut sorted: Vec<(String, f32)> = weighted_votes.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Normalize weights: divide everything by the highest weighted vote
    // so that the primary always reads ≤ 1.0 and see_also reads as a
    // fraction of how strongly it competed with the primary. This makes
    // the displayed percentages mean "how much of the total ensemble
    // signal favored this leaf relative to the winner."
    let normalizer = sorted[0].1.max(0.0001);

    let (primary_id, primary_raw_weight) = sorted[0].clone();
    let primary_weight = (primary_raw_weight / normalizer).clamp(0.0, 1.0);
    let primary_voters = who_voted_for.get(&primary_id).cloned().unwrap_or_default();

    // Count catalogers that voted vs catalogers that voted differently.
    let total_voters: usize = who_voted_for.values().map(|v| v.len()).sum();
    let primary_voter_count = primary_voters.len();

    let regime = compute_regime(primary_voter_count, total_voters);

    let dissenter = if regime == ConfidenceRegime::StrongMajority {
        // Find a cataloger that voted for something else.
        who_voted_for
            .iter()
            .filter(|(id, _)| **id != primary_id)
            .flat_map(|(_, voters)| voters.iter().cloned())
            .next()
    } else {
        None
    };

    let see_also: Vec<String> = sorted
        .iter()
        .skip(1)
        .take(3)
        .map(|(id, _)| id.clone())
        .collect();
    let see_also_weights: Vec<f32> = sorted
        .iter()
        .skip(1)
        .take(3)
        .map(|(_, w)| (w / normalizer).clamp(0.0, 1.0))
        .collect();

    let needs_user_disambiguation_between = if regime == ConfidenceRegime::Split {
        // Surface the top 2-3 candidates for the user to pick.
        Some(sorted.iter().take(3).map(|(id, _)| id.clone()).collect())
    } else {
        None
    };

    AxisDecision {
        primary: Some(primary_id),
        secondary: Vec::new(),
        regime,
        see_also,
        needs_user_disambiguation_between,
        dissenter,
        primary_weight,
        see_also_weights,
    }
}

/// Confidence regime decision per Architect §3.1:
///   * Unanimous: all voters voted for primary.
///   * Strong Majority: ≥ 2/3 of voters AND dissenters ≤ 1.
///   * Split: otherwise.
fn compute_regime(primary_voters: usize, total_voters: usize) -> ConfidenceRegime {
    if primary_voters == total_voters {
        ConfidenceRegime::Unanimous
    } else if total_voters >= 3
        && (primary_voters * 3) >= (total_voters * 2)
        && (total_voters - primary_voters) <= 1
    {
        ConfidenceRegime::StrongMajority
    } else {
        ConfidenceRegime::Split
    }
}

fn confidence_multiplier(c: Confidence) -> f32 {
    match c {
        Confidence::High => 1.0,
        Confidence::Medium => 0.7,
        Confidence::Low => 0.4,
        Confidence::Abstain => 0.0,
    }
}

fn compose_reasoning(
    voiced: &[&ReasoningTrail],
    horizontal: &AxisDecision,
    vertical: &AxisDecision,
) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = write!(
        out,
        "Horizontal: {} ({:?}); Vertical: {} ({:?}). ",
        horizontal.primary.as_deref().unwrap_or("(none)"),
        horizontal.regime,
        vertical.primary.as_deref().unwrap_or("(none)"),
        vertical.regime,
    );
    let voiced_names: Vec<&str> = voiced.iter().map(|t| t.cataloger.as_str()).collect();
    let _ = write!(
        out,
        "Catalogers voiced: {}.",
        voiced_names.join(", "),
    );
    out
}

// Stub for the unused import warning suppression; AxisAssignment is part
// of the public schema but not directly referenced in this file's logic.
#[allow(dead_code)]
fn _suppress_unused(_a: AxisAssignment) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn trail(
        name: &str,
        h_id: Option<&str>,
        v_id: Option<&str>,
        confidence: Confidence,
    ) -> ReasoningTrail {
        ReasoningTrail {
            cataloger: name.to_string(),
            voiced_opinion: confidence != Confidence::Abstain,
            horizontal: h_id
                .map(|id| {
                    vec![AxisAssignment {
                        id: id.to_string(),
                        primary: true,
                        weight: 1.0,
                        descend_uncertain: false,
                    }]
                })
                .unwrap_or_default(),
            vertical: v_id
                .map(|id| {
                    vec![AxisAssignment {
                        id: id.to_string(),
                        primary: true,
                        weight: 1.0,
                        descend_uncertain: false,
                    }]
                })
                .unwrap_or_default(),
            reasoning: String::new(),
            rules_fired: Vec::new(),
            alternatives_considered: Vec::new(),
            self_reported_confidence: confidence,
        }
    }

    #[test]
    fn user_authority_short_circuits() {
        let trails = vec![
            trail("user_authority", Some("testimony"), Some("epistemic-states"), Confidence::High),
            trail("linguistic", Some("inference"), Some("semantic-contents"), Confidence::High),
        ];
        let r = ReliabilityProfile::default();
        let result = synthesize(trails, &r);
        assert_eq!(result.horizontal.primary.as_deref(), Some("testimony"));
        assert_eq!(result.vertical.primary.as_deref(), Some("epistemic-states"));
        assert_eq!(result.synthesis_method, "user_authority_short_circuit");
    }

    #[test]
    fn unanimous_regime_when_all_agree() {
        let trails = vec![
            trail("linguistic", Some("testimony"), None, Confidence::High),
            trail("structural", Some("testimony"), None, Confidence::High),
            trail("graph", Some("testimony"), None, Confidence::High),
        ];
        let r = ReliabilityProfile::default();
        let result = synthesize(trails, &r);
        assert_eq!(result.horizontal.primary.as_deref(), Some("testimony"));
        assert_eq!(result.horizontal.regime, ConfidenceRegime::Unanimous);
    }

    #[test]
    fn strong_majority_when_one_dissents() {
        let trails = vec![
            trail("linguistic", Some("testimony"), None, Confidence::High),
            trail("structural", Some("testimony"), None, Confidence::High),
            trail("graph", Some("testimony"), None, Confidence::High),
            trail("semantic", Some("inference"), None, Confidence::High),
        ];
        let r = ReliabilityProfile::default();
        let result = synthesize(trails, &r);
        assert_eq!(result.horizontal.primary.as_deref(), Some("testimony"));
        assert_eq!(result.horizontal.regime, ConfidenceRegime::StrongMajority);
        assert_eq!(result.horizontal.dissenter.as_deref(), Some("semantic"));
    }

    #[test]
    fn split_when_close_disagreement() {
        let trails = vec![
            trail("linguistic", Some("testimony"), None, Confidence::High),
            trail("structural", Some("testimony"), None, Confidence::High),
            trail("graph", Some("inference"), None, Confidence::High),
            trail("semantic", Some("inference"), None, Confidence::High),
        ];
        let r = ReliabilityProfile::default();
        let result = synthesize(trails, &r);
        assert_eq!(result.horizontal.regime, ConfidenceRegime::Split);
        assert!(result.horizontal.needs_user_disambiguation_between.is_some());
    }
}

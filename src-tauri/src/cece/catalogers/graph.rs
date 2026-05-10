//! MIG-021v3 V3-§6 — Graph Cataloger.
//!
//! Reads through the LIVING LINKS TYPED NEIGHBORHOOD — what notes are
//! linked to this one (via the `note_links` table), what TYPE each link
//! has (supports / contradicts / causes / exemplifies / generalizes /
//! derives-from / part-of), and what the typed neighbors were
//! classified as. The "what does my neighborhood think this is?" lens.
//!
//! Per Architect §2.3 + §4 Rule of Authority Control:
//!   Strong on: densely-linked notes; authority-control via consensus
//!   Weak on:   orphan notes (degree < 2); abstain
//!   Latency:   ~10–30 ms (one DB query for typed neighbors + their
//!              classifications)
//!
//! Vote weighting per link type (Architect §2.3):
//!   * derives-from / part-of  → strongest signal (1.0×) — these say
//!     the current note IS LIKE the linked note in some structural way
//!   * contradicts             → INVERTED signal (-0.7×) — neighbor's
//!     classification is what this note ISN'T
//!   * supports / generalizes  → moderate signal (0.7×) — related but
//!     not identical
//!   * causes / exemplifies    → light signal (0.5×)
//!
//! Implementation pattern matches Semantic Cataloger: an injectable
//! `lookup_fn: NeighborLookupFn` returns the typed-neighbor list when
//! wired by the orchestrator (V3-§8); None in unit tests, in which
//! case the cataloger abstains gracefully.

use crate::cece::cataloger::{
    Axis, AxisAssignment, Cataloger, CatalogerContext, Confidence, ReasoningTrail, TypedNeighbor,
};
use crate::sources::{is_valid_content_type_id, is_valid_source_id};
use std::collections::HashMap;

/// Minimum typed-neighbor count for the cataloger to vote. Below this,
/// signal is too sparse — abstain rather than guess from one neighbor.
pub const MIN_NEIGHBORS_FOR_VOTING: usize = 2;

pub type NeighborLookupFn = Box<
    dyn Fn(&str) -> Result<Vec<TypedNeighbor>, String> + Send + Sync + 'static,
>;

pub struct GraphCataloger {
    lookup_fn: Option<NeighborLookupFn>,
}

impl GraphCataloger {
    pub fn new() -> Self {
        Self { lookup_fn: None }
    }

    pub fn with_lookup(lookup: NeighborLookupFn) -> Self {
        Self {
            lookup_fn: Some(lookup),
        }
    }
}

impl Default for GraphCataloger {
    fn default() -> Self {
        Self::new()
    }
}

impl Cataloger for GraphCataloger {
    fn name(&self) -> &'static str {
        "graph"
    }

    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
        let Some(lookup) = &self.lookup_fn else {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Typed-neighbor lookup not wired (unit-test or boot-time path).",
            ));
        };

        let neighbors = match lookup(&ctx.note_path) {
            Ok(n) => n,
            Err(e) => {
                return Some(ReasoningTrail::abstain(
                    self.name(),
                    &format!("Neighbor lookup failed: {}", e),
                ));
            }
        };

        // Filter to neighbors that have at least one classification on
        // at least one axis. An unclassified neighbor contributes no
        // information to the vote.
        let classified: Vec<&TypedNeighbor> = neighbors
            .iter()
            .filter(|n| !n.neighbor_sources.is_empty() || !n.neighbor_content_type.is_empty())
            .collect();

        if classified.len() < MIN_NEIGHBORS_FOR_VOTING {
            return Some(ReasoningTrail::abstain(
                self.name(),
                &format!(
                    "Sparse neighborhood: {} classified typed neighbor(s); need ≥ {}.",
                    classified.len(),
                    MIN_NEIGHBORS_FOR_VOTING
                ),
            ));
        }

        // Weighted vote per axis using link-type weights. A "negative
        // vote" (from a `contradicts` neighbor) is recorded as
        // negative weight on the neighbor's classification, which the
        // top-N selection then de-prioritizes.
        let mut h_votes: HashMap<String, f32> = HashMap::new();
        let mut v_votes: HashMap<String, f32> = HashMap::new();
        let mut neighbors_used: Vec<(String, String, f32)> = Vec::new(); // (path, link_type, weight)
        let mut total_signal = 0.0_f32;

        for n in &classified {
            let weight = link_type_weight(&n.link_type);
            if weight.abs() < 0.01 {
                continue; // unknown / unsupported link type
            }
            neighbors_used.push((n.neighbor_path.clone(), n.link_type.clone(), weight));
            total_signal += weight.abs();
            for src in &n.neighbor_sources {
                if is_valid_source_id(src) {
                    *h_votes.entry(src.clone()).or_insert(0.0) += weight;
                }
            }
            for ct in &n.neighbor_content_type {
                if is_valid_content_type_id(ct) {
                    *v_votes.entry(ct.clone()).or_insert(0.0) += weight;
                }
            }
        }

        if neighbors_used.is_empty() {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Typed neighbors found, but none had recognized link types.",
            ));
        }

        let horizontal = top_assignments(h_votes, 3, total_signal);
        let vertical = top_assignments(v_votes, 3, total_signal);

        // Confidence by max link weight present in the vote.
        let max_weight = neighbors_used
            .iter()
            .map(|(_, _, w)| w.abs())
            .fold(0.0_f32, f32::max);
        let confidence = if max_weight >= 0.95 {
            Confidence::High
        } else if max_weight >= 0.65 {
            Confidence::Medium
        } else {
            Confidence::Low
        };

        let reasoning = build_reasoning(&horizontal, &vertical, &neighbors_used);

        Some(ReasoningTrail {
            cataloger: self.name().to_string(),
            voiced_opinion: !horizontal.is_empty() || !vertical.is_empty(),
            horizontal,
            vertical,
            reasoning,
            rules_fired: vec![
                "typed_neighbor_consensus".to_string(),
                "rule_of_authority_control".to_string(),
            ],
            alternatives_considered: Vec::new(),
            self_reported_confidence: confidence,
        })
    }

    fn supported_axes(&self) -> &[Axis] {
        &[Axis::Horizontal, Axis::Vertical]
    }
}

/// Map a Living Links link type to a vote weight. Positive values
/// pull the neighbor's classification toward this note; negative
/// values push it away. Unknown types return 0 (no vote).
///
/// Per Architect §2.3 — the seven Living Link types per the design.
fn link_type_weight(link_type: &str) -> f32 {
    match link_type {
        "derives-from" | "part-of" => 1.0,
        "contradicts" => -0.7,
        "supports" | "generalizes" => 0.7,
        "causes" | "exemplifies" => 0.5,
        _ => 0.0,
    }
}

fn top_assignments(
    votes: HashMap<String, f32>,
    max: usize,
    normalizer: f32,
) -> Vec<AxisAssignment> {
    if votes.is_empty() {
        return Vec::new();
    }
    // Filter out negative final scores — those are "anti-votes" from
    // contradicts links and shouldn't surface as positive assignments.
    let mut entries: Vec<(String, f32)> = votes
        .into_iter()
        .filter(|(_, w)| *w > 0.0)
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries.truncate(max);
    let denom = normalizer.max(0.0001);
    entries
        .into_iter()
        .enumerate()
        .map(|(i, (id, vote))| AxisAssignment {
            id,
            primary: i == 0,
            weight: (vote / denom).clamp(0.0, 1.0),
            descend_uncertain: false,
        })
        .collect()
}

fn build_reasoning(
    h: &[AxisAssignment],
    v: &[AxisAssignment],
    neighbors: &[(String, String, f32)],
) -> String {
    use std::fmt::Write;
    let mut out = String::from("Typed-neighbor consensus: ");
    let mut parts = Vec::new();
    if let Some(top) = h.first() {
        parts.push(format!("horizontal → {} (weight {:.2})", top.id, top.weight));
    }
    if let Some(top) = v.first() {
        parts.push(format!("vertical → {} (weight {:.2})", top.id, top.weight));
    }
    let _ = write!(out, "{}.", parts.join("; "));
    let preview: Vec<String> = neighbors
        .iter()
        .take(3)
        .map(|(p, lt, w)| format!("{} ({}, {:+.2})", short_name(p), lt, w))
        .collect();
    let _ = write!(out, " Neighbors: {}.", preview.join(", "));
    out
}

fn short_name(path: &str) -> String {
    path.rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".md")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> CatalogerContext {
        CatalogerContext::new(
            "current.md".to_string(),
            String::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    fn n(path: &str, link_type: &str, sources: Vec<&str>, content_type: Vec<&str>) -> TypedNeighbor {
        TypedNeighbor {
            neighbor_path: path.to_string(),
            link_type: link_type.to_string(),
            neighbor_sources: sources.into_iter().map(String::from).collect(),
            neighbor_content_type: content_type.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn no_lookup_wired_abstains() {
        let c = GraphCataloger::new();
        let trail = c.classify(&ctx()).unwrap();
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn orphan_abstains() {
        let lookup = Box::new(|_p: &str| Ok(Vec::new()));
        let c = GraphCataloger::with_lookup(lookup);
        let trail = c.classify(&ctx()).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("Sparse neighborhood"));
    }

    #[test]
    fn one_classified_neighbor_abstains() {
        let lookup = Box::new(|_p: &str| {
            Ok(vec![n(
                "lonely.md",
                "derives-from",
                vec!["testimony"],
                vec![],
            )])
        });
        let c = GraphCataloger::with_lookup(lookup);
        let trail = c.classify(&ctx()).unwrap();
        // Only 1 classified neighbor → below MIN_NEIGHBORS_FOR_VOTING.
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn derives_from_consensus_fires_high() {
        let lookup = Box::new(|_p: &str| {
            Ok(vec![
                n("hadith1.md", "derives-from", vec!["testimony"], vec!["semantic-contents"]),
                n("hadith2.md", "derives-from", vec!["testimony"], vec!["semantic-contents"]),
                n("hadith3.md", "part-of", vec!["testimony"], vec!["semantic-contents"]),
            ])
        });
        let c = GraphCataloger::with_lookup(lookup);
        let trail = c.classify(&ctx()).unwrap();
        assert!(trail.voiced_opinion);
        assert_eq!(trail.horizontal[0].id, "testimony");
        assert!(trail.horizontal[0].primary);
        assert_eq!(trail.self_reported_confidence, Confidence::High);
    }

    #[test]
    fn contradicts_inverts_vote() {
        // Two derives-from neighbors say "perception"; one contradicts
        // says "testimony" → inverted, so testimony gets a -0.7 vote
        // and perception wins. Final perception score: 2.0 / 2.7 ≈ 0.74.
        let lookup = Box::new(|_p: &str| {
            Ok(vec![
                n("a.md", "derives-from", vec!["perception"], vec![]),
                n("b.md", "derives-from", vec!["perception"], vec![]),
                n("c.md", "contradicts", vec!["testimony"], vec![]),
            ])
        });
        let c = GraphCataloger::with_lookup(lookup);
        let trail = c.classify(&ctx()).unwrap();
        assert!(trail.voiced_opinion);
        assert_eq!(trail.horizontal[0].id, "perception");
        // testimony was anti-voted → must NOT appear as a positive
        // assignment.
        assert!(!trail.horizontal.iter().any(|a| a.id == "testimony"));
    }

    #[test]
    fn unknown_link_types_ignored() {
        let lookup = Box::new(|_p: &str| {
            Ok(vec![
                n("a.md", "wibble", vec!["testimony"], vec![]),
                n("b.md", "wobble", vec!["testimony"], vec![]),
                n("c.md", "wubble", vec!["testimony"], vec![]),
            ])
        });
        let c = GraphCataloger::with_lookup(lookup);
        let trail = c.classify(&ctx()).unwrap();
        // All link types unknown → no recognized weight → abstain.
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn link_type_weights_correct() {
        assert_eq!(link_type_weight("derives-from"), 1.0);
        assert_eq!(link_type_weight("part-of"), 1.0);
        assert_eq!(link_type_weight("contradicts"), -0.7);
        assert_eq!(link_type_weight("supports"), 0.7);
        assert_eq!(link_type_weight("generalizes"), 0.7);
        assert_eq!(link_type_weight("causes"), 0.5);
        assert_eq!(link_type_weight("exemplifies"), 0.5);
        assert_eq!(link_type_weight("nonsense"), 0.0);
    }
}

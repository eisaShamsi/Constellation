//! MIG-021v3 V3-§5 — Semantic Cataloger.
//!
//! Reads through SEMANTIC SIMILARITY — embeds the note via the existing
//! e5-small ONNX path, finds the k nearest already-classified notes in
//! the per-Library exemplar memory, and votes their classifications
//! weighted by cosine similarity.
//!
//! Per Architect §2.4:
//!   Strong on: notes similar to many already-classified ones
//!   Weak on:   novel territory; cold-start at zero corrections
//!   Latency:   ~30 ms (e5-small embed) + few ms for kNN vote
//!
//! Cold-start handling: if the per-Library exemplar memory has fewer
//! than `MIN_EXEMPLARS_FOR_VOTING` classified notes, this cataloger
//! abstains. The active subtree of any new Library has no signal yet;
//! Linguistic and Structural carry the load until corrections accumulate.
//!
//! Two injectable functions (per the LinguisticCataloger pattern):
//!   * `embed_fn` — text → embedding vector
//!   * `lookup_fn` — embedding → list of (path, sources, content_type, cosine)
//!
//! Both wired by the orchestrator in V3-§8 when AppHandle is available;
//! both None in unit tests, in which case the cataloger abstains.

use crate::cece::cataloger::{
    Axis, AxisAssignment, Cataloger, CatalogerContext, Confidence, ReasoningTrail,
};
use crate::sources::{is_valid_content_type_id, is_valid_source_id};
use std::collections::HashMap;

/// Minimum number of classified neighbors before voting can fire.
/// Below this, the cataloger abstains to avoid noisy cold-start signal.
pub const MIN_EXEMPLARS_FOR_VOTING: usize = 3;

/// Top-K neighbors consulted per classification.
pub const TOP_K_NEIGHBORS: usize = 5;

/// Minimum cosine similarity for a neighbor to count toward the vote.
/// Below this, the neighbor is too dissimilar to inform anything.
pub const MIN_COSINE_FOR_VOTE: f32 = 0.55;

pub type EmbedFn = Box<dyn Fn(&str) -> Result<Vec<f32>, String> + Send + Sync + 'static>;

/// Returns up to `k` nearest already-classified neighbors of the query
/// embedding, scoped to a single Library. Each tuple:
/// (note_path, sources, content_type, cosine_score). Score in [0.0, 1.0].
pub type NeighborLookupFn = Box<
    dyn Fn(&[f32], usize) -> Result<Vec<NeighborRecord>, String> + Send + Sync + 'static,
>;

#[derive(Debug, Clone)]
pub struct NeighborRecord {
    pub note_path: String,
    pub sources: Vec<String>,
    pub content_type: Vec<String>,
    pub cosine: f32,
}

pub struct SemanticCataloger {
    embed_fn: Option<EmbedFn>,
    lookup_fn: Option<NeighborLookupFn>,
}

impl SemanticCataloger {
    pub fn new() -> Self {
        Self {
            embed_fn: None,
            lookup_fn: None,
        }
    }

    pub fn with_io(embed: EmbedFn, lookup: NeighborLookupFn) -> Self {
        Self {
            embed_fn: Some(embed),
            lookup_fn: Some(lookup),
        }
    }
}

impl Default for SemanticCataloger {
    fn default() -> Self {
        Self::new()
    }
}

impl Cataloger for SemanticCataloger {
    fn name(&self) -> &'static str {
        "semantic"
    }

    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
        let (Some(embed), Some(lookup)) = (&self.embed_fn, &self.lookup_fn) else {
            // No embedder / no DB lookup wired — typical in unit tests.
            return Some(ReasoningTrail::abstain(
                self.name(),
                "Embedder or neighbor-lookup not wired (unit-test or boot-time path).",
            ));
        };

        // 1. Embed the note text.
        let embedding = match embed(&ctx.content) {
            Ok(v) => v,
            Err(e) => {
                return Some(ReasoningTrail::abstain(
                    self.name(),
                    &format!("Embedding failed: {}", e),
                ));
            }
        };

        // 2. Find k nearest already-classified neighbors in this Library.
        let neighbors = match lookup(&embedding, TOP_K_NEIGHBORS) {
            Ok(n) => n,
            Err(e) => {
                return Some(ReasoningTrail::abstain(
                    self.name(),
                    &format!("Neighbor lookup failed: {}", e),
                ));
            }
        };

        if neighbors.len() < MIN_EXEMPLARS_FOR_VOTING {
            return Some(ReasoningTrail::abstain(
                self.name(),
                &format!(
                    "Cold-start: only {} classified neighbors found (need ≥ {}).",
                    neighbors.len(),
                    MIN_EXEMPLARS_FOR_VOTING
                ),
            ));
        }

        // 3. Weighted vote: each neighbor contributes its assignments
        //    weighted by cosine. Skip neighbors below MIN_COSINE_FOR_VOTE.
        let mut h_votes: HashMap<String, f32> = HashMap::new();
        let mut v_votes: HashMap<String, f32> = HashMap::new();
        let mut neighbor_paths: Vec<(String, f32)> = Vec::new();
        let mut total_signal = 0.0_f32;
        for n in &neighbors {
            if n.cosine < MIN_COSINE_FOR_VOTE {
                continue;
            }
            neighbor_paths.push((n.note_path.clone(), n.cosine));
            total_signal += n.cosine;
            for src in &n.sources {
                if is_valid_source_id(src) {
                    *h_votes.entry(src.clone()).or_insert(0.0) += n.cosine;
                }
            }
            for ct in &n.content_type {
                if is_valid_content_type_id(ct) {
                    *v_votes.entry(ct.clone()).or_insert(0.0) += n.cosine;
                }
            }
        }

        if neighbor_paths.is_empty() {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "No neighbors above the minimum cosine threshold.",
            ));
        }

        // Normalize votes to [0, 1] using total_signal so weights are
        // comparable across catalogers downstream.
        let horizontal = top_assignments(h_votes, 3, total_signal);
        let vertical = top_assignments(v_votes, 3, total_signal);

        let confidence = if neighbor_paths.iter().any(|(_, c)| *c >= 0.85) {
            Confidence::High
        } else if neighbor_paths.iter().any(|(_, c)| *c >= 0.70) {
            Confidence::Medium
        } else {
            Confidence::Low
        };

        let reasoning = build_reasoning(&horizontal, &vertical, &neighbor_paths);

        Some(ReasoningTrail {
            cataloger: self.name().to_string(),
            voiced_opinion: !horizontal.is_empty() || !vertical.is_empty(),
            horizontal,
            vertical,
            reasoning,
            rules_fired: vec![
                "semantic_neighbor_consensus".to_string(),
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

fn top_assignments(
    votes: HashMap<String, f32>,
    max: usize,
    normalizer: f32,
) -> Vec<AxisAssignment> {
    if votes.is_empty() {
        return Vec::new();
    }
    let mut entries: Vec<(String, f32)> = votes.into_iter().collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries.truncate(max);
    let denom = normalizer.max(0.0001);
    entries
        .into_iter()
        .enumerate()
        .map(|(i, (id, vote))| AxisAssignment {
            id,
            primary: i == 0,
            // Normalize to roughly [0, 1] but cap to keep within band.
            weight: (vote / denom).clamp(0.0, 1.0),
            descend_uncertain: false,
        })
        .collect()
}

fn build_reasoning(
    h: &[AxisAssignment],
    v: &[AxisAssignment],
    neighbors: &[(String, f32)],
) -> String {
    use std::fmt::Write;
    let mut out = String::from("Semantic neighbor consensus: ");
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
        .map(|(p, c)| format!("{} ({:.2})", short_name(p), c))
        .collect();
    let _ = write!(out, " Top neighbors: {}.", preview.join(", "));
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

    fn ctx_for_body(body: &str) -> CatalogerContext {
        CatalogerContext::new(
            "test.md".to_string(),
            body.to_string(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn no_io_wired_abstains() {
        // The default-constructed cataloger has no embed_fn / lookup_fn,
        // which is the unit-test path. It must abstain rather than panic.
        let c = SemanticCataloger::new();
        let trail = c.classify(&ctx_for_body("any text")).unwrap();
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn cold_start_abstains_below_min_exemplars() {
        let embed = Box::new(|_text: &str| Ok(vec![0.1_f32; 384]));
        let lookup = Box::new(|_q: &[f32], _k: usize| {
            Ok(vec![NeighborRecord {
                note_path: "only_one.md".to_string(),
                sources: vec!["testimony".to_string()],
                content_type: vec![],
                cosine: 0.95,
            }])
        });
        let c = SemanticCataloger::with_io(embed, lookup);
        let trail = c.classify(&ctx_for_body("query")).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("Cold-start"));
    }

    #[test]
    fn neighbor_consensus_fires() {
        let embed = Box::new(|_text: &str| Ok(vec![0.2_f32; 384]));
        let lookup = Box::new(|_q: &[f32], _k: usize| {
            Ok(vec![
                NeighborRecord {
                    note_path: "a.md".to_string(),
                    sources: vec!["testimony".to_string()],
                    content_type: vec!["semantic-contents".to_string()],
                    cosine: 0.92,
                },
                NeighborRecord {
                    note_path: "b.md".to_string(),
                    sources: vec!["testimony".to_string()],
                    content_type: vec!["semantic-contents".to_string()],
                    cosine: 0.88,
                },
                NeighborRecord {
                    note_path: "c.md".to_string(),
                    sources: vec!["testimony".to_string()],
                    content_type: vec!["semantic-contents".to_string()],
                    cosine: 0.83,
                },
            ])
        });
        let c = SemanticCataloger::with_io(embed, lookup);
        let trail = c.classify(&ctx_for_body("query")).unwrap();
        assert!(trail.voiced_opinion);
        assert_eq!(trail.horizontal[0].id, "testimony");
        assert!(trail.horizontal[0].primary);
        assert_eq!(trail.self_reported_confidence, Confidence::High);
    }

    #[test]
    fn neighbors_below_min_cosine_dropped() {
        let embed = Box::new(|_text: &str| Ok(vec![0.0_f32; 384]));
        let lookup = Box::new(|_q: &[f32], _k: usize| {
            Ok(vec![
                NeighborRecord {
                    note_path: "weak1.md".to_string(),
                    sources: vec!["testimony".to_string()],
                    content_type: vec![],
                    cosine: 0.30,
                },
                NeighborRecord {
                    note_path: "weak2.md".to_string(),
                    sources: vec!["testimony".to_string()],
                    content_type: vec![],
                    cosine: 0.20,
                },
                NeighborRecord {
                    note_path: "weak3.md".to_string(),
                    sources: vec!["testimony".to_string()],
                    content_type: vec![],
                    cosine: 0.10,
                },
            ])
        });
        let c = SemanticCataloger::with_io(embed, lookup);
        let trail = c.classify(&ctx_for_body("query")).unwrap();
        // All neighbors below MIN_COSINE_FOR_VOTE → no signal → abstain.
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn embed_failure_abstains() {
        let embed = Box::new(|_text: &str| Err("ONNX session failed".to_string()));
        let lookup = Box::new(|_q: &[f32], _k: usize| Ok(Vec::new()));
        let c = SemanticCataloger::with_io(embed, lookup);
        let trail = c.classify(&ctx_for_body("any")).unwrap();
        assert!(!trail.voiced_opinion);
        assert!(trail.reasoning.contains("Embedding failed"));
    }
}

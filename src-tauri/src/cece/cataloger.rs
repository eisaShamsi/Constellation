//! MIG-021v3 V3-§1 — Cataloger trait + shared types.
//!
//! Every cataloger (Linguistic, Structural, Graph, Semantic, Reasoning,
//! User-Authority) implements this trait. The orchestrator runs them in
//! cost order; the synthesis layer combines their reasoning trails.
//!
//! Per Architect §2: each cataloger reads through a methodologically
//! distinct lens and either voices an opinion (with a reasoning trail)
//! or abstains (`voiced_opinion: false`). Abstention is a positive
//! signal — it tells the synthesis layer this lens had no evidence.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// The two classification axes. A cataloger may produce assignments for
/// both, one, or neither (when it abstains).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn as_str(self) -> &'static str {
        match self {
            Axis::Horizontal => "horizontal",
            Axis::Vertical => "vertical",
        }
    }
}

/// A single cataloger's confidence in its own output. Returned per-trail,
/// not per-axis (a cataloger that's confident on horizontal but unsure
/// on vertical should produce two separate trails or use abstain).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Strong evidence; weight ~1.0 in synthesis.
    High,
    /// Moderate evidence; weight ~0.7 in synthesis.
    Medium,
    /// Weak evidence; weight ~0.4 in synthesis. Often paired with
    /// `descend_uncertain: true` on the assignment.
    Low,
    /// No evidence; cataloger abstains. Synthesis ignores this trail.
    Abstain,
}

/// Single assignment within an axis: the taxonomy ID, primary/secondary
/// flag, optional weight (cataloger's own per-assignment confidence), and
/// the depth-budget escape hatch from Architect §5.3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisAssignment {
    pub id: String,
    pub primary: bool,
    pub weight: f32,
    /// True when the cataloger believes the parent class is correct but
    /// is not confident at the leaf. Synthesis layer treats this as a
    /// vote for the parent + "see also" for the leaf candidates.
    #[serde(default)]
    pub descend_uncertain: bool,
}

/// An alternative the cataloger considered but rejected. Used for the
/// reasoning trail and for the active-learning correction log (so when
/// the user overrides, we know which alternative was the right one).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedAlternative {
    pub axis: Axis,
    pub id: String,
    pub rejected_because: String,
}

/// The output of a single cataloger's `classify` call. Persisted, audited,
/// surfaced to the user, and consumed by the synthesis layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrail {
    /// Lowercase short name (e.g. "linguistic", "structural", "graph").
    pub cataloger: String,
    /// False when the cataloger had no signal from its lens. Other fields
    /// are still present (empty) but ignored by synthesis.
    pub voiced_opinion: bool,
    pub horizontal: Vec<AxisAssignment>,
    pub vertical: Vec<AxisAssignment>,
    /// One paragraph in plain language explaining how this cataloger
    /// arrived at its assignment. Surfaced to the user verbatim.
    pub reasoning: String,
    /// Names of cataloger rules (Architect §4) that fired during this
    /// classification. e.g. ["root_pattern_match", "rule_of_authority"].
    pub rules_fired: Vec<String>,
    /// Sibling/near-miss leaves the cataloger considered + rejected.
    pub alternatives_considered: Vec<RejectedAlternative>,
    pub self_reported_confidence: Confidence,
}

impl ReasoningTrail {
    /// Convenience constructor for the "abstain" case — every other field
    /// empty. The synthesis layer skips trails where `voiced_opinion: false`.
    pub fn abstain(cataloger: &str, reason: &str) -> Self {
        Self {
            cataloger: cataloger.to_string(),
            voiced_opinion: false,
            horizontal: Vec::new(),
            vertical: Vec::new(),
            reasoning: reason.to_string(),
            rules_fired: Vec::new(),
            alternatives_considered: Vec::new(),
            self_reported_confidence: Confidence::Abstain,
        }
    }
}

/// The shared input every cataloger receives. Lazy-loaded helpers
/// (typed_neighbors, cae_normalized, embedding) populate on first
/// access so catalogers that don't need them pay no cost.
///
/// All `OnceLock` fields are populated by the orchestrator on demand
/// (the orchestrator knows which catalogers will be invoked + which
/// inputs they need).
pub struct CatalogerContext {
    pub note_path: String,
    pub content: String,
    /// Frontmatter `sources:` already extracted. Empty Vec when absent.
    pub frontmatter_sources: Vec<String>,
    /// Frontmatter `content_type:` already extracted. Empty Vec when absent.
    pub frontmatter_content_type: Vec<String>,
    /// Lazily populated: typed-neighbor list for the Graph Cataloger.
    /// `None` means the orchestrator hasn't loaded it yet (cataloger
    /// calls `typed_neighbors_or_load()`).
    pub typed_neighbors: OnceLock<Vec<TypedNeighbor>>,
    /// Lazily populated: CAE morphology output for the Linguistic
    /// Cataloger.
    pub cae_normalized: OnceLock<CaeNormalizedText>,
    /// Lazily populated: e5-small embedding for the Semantic Cataloger.
    pub embedding: OnceLock<Vec<f32>>,
}

impl CatalogerContext {
    pub fn new(
        note_path: String,
        content: String,
        frontmatter_sources: Vec<String>,
        frontmatter_content_type: Vec<String>,
    ) -> Self {
        Self {
            note_path,
            content,
            frontmatter_sources,
            frontmatter_content_type,
            typed_neighbors: OnceLock::new(),
            cae_normalized: OnceLock::new(),
            embedding: OnceLock::new(),
        }
    }
}

/// A typed neighbor of the current note via Living Links. Used by the
/// Graph Cataloger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedNeighbor {
    pub neighbor_path: String,
    /// One of: supports / contradicts / causes / exemplifies /
    /// generalizes / derives-from / part-of (per Living Links §0).
    pub link_type: String,
    pub neighbor_sources: Vec<String>,
    pub neighbor_content_type: Vec<String>,
}

/// CAE-normalized text for the Linguistic Cataloger. Each entry is one
/// detected Arabic root + its surface form + position + pattern.
#[derive(Debug, Clone, Default)]
pub struct CaeNormalizedText {
    pub roots: Vec<DetectedRoot>,
}

#[derive(Debug, Clone)]
pub struct DetectedRoot {
    pub root: String,
    pub surface_form: String,
    pub byte_offset: usize,
    /// Optional pattern/wazn classification (e.g. "fiʿāl" for قياس).
    pub pattern: Option<String>,
}

/// The trait every cataloger implements. Stateless across calls — any
/// per-Library state (e.g. exemplar memory for Semantic) lives in the
/// cataloger's own struct, not in the trait.
pub trait Cataloger: Send + Sync {
    /// Lowercase short name. Used in reasoning trails, reliability JSON,
    /// UI badges. Must be stable across versions.
    fn name(&self) -> &'static str;

    /// Run classification. Returns `None` if the cataloger panics or
    /// times out (caught by the orchestrator); returns `Some(trail)`
    /// with `voiced_opinion: false` if the cataloger ran but had no
    /// signal.
    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail>;

    /// Which axes this cataloger can produce assignments for. Most
    /// catalogers support both; some (e.g. a future Audio-only one)
    /// might support only one.
    fn supported_axes(&self) -> &[Axis];
}

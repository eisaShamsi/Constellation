//! Lexicon graph data structures.
//!
//! The graph is loaded once at boot from embedded binary data (core tier)
//! plus any installed expansion packs and the current Universe's
//! user-overrides file. After loading it is immutable; updates from the
//! learning loop (layer 5) append to the overrides file and trigger a
//! focused in-memory patch without a full rebuild.
//!
//! # Node identity
//!
//! A node is `(lang, lemma, sense_id)`. The tuple is required because:
//!   - The same string can be a different word in different languages:
//!     English "or" and French "or" are unrelated nodes.
//!   - The same string can be multiple senses in one language:
//!     English "bank" has a riverside sense and a financial sense, each
//!     with different translations.
//!
//! # Edge semantics
//!
//! Edges are undirected and typed:
//!   - `Equivalent` — translation equivalence (the common case).
//!   - `Synonym` — in-language near-equivalent (used for layer 2 expansion).
//!   - `Hypernym` — target is a broader concept ("knowledge" → "cognition").
//!   - `Hyponym` — target is a narrower concept.
//!   - `UserLink` — added by the Universe-level override layer.
//!
//! # Storage footprint
//!
//! Core tier ≈ 20,000 concepts × ~10 langs with translations each ≈
//! 200K nodes and ~800K edges. Stored as a compact adjacency CSR format
//! (two parallel vecs: offsets + targets) plus a FST for name → node-id
//! lookup. Total ≈ 14 MB uncompressed, ≈ 4 MB gzipped in the binary.

use crate::arabic::Lang;
use serde::{Deserialize, Serialize};

/// Opaque sense identifier. For WordNet-derived entries this is the
/// synset offset; for Wiktionary entries it is a hash of the sense
/// line; for user overrides it is `0` (single default sense).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SenseId(pub u32);

impl SenseId {
    /// The default / unspecified sense. Used when the data source does
    /// not distinguish senses (loanwords, proper nouns, user overrides).
    pub const DEFAULT: SenseId = SenseId(0);
}

/// A lemma node in the lexicon graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LemmaNode {
    pub lang: Lang,
    /// The lemma string (no morphological affixes, stored in the language's
    /// canonical form — e.g. Arabic uses the bare dictionary form with
    /// no definite article).
    pub lemma: String,
    /// Sense within this lemma. Multiple `LemmaNode` records can share
    /// `(lang, lemma)` but differ in `sense_id`.
    pub sense_id: SenseId,
}

/// Kind of edge between two lemma nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Cross-lingual translation equivalence (primary use).
    Equivalent,
    /// In-language synonym (layer-2 expansion, always included).
    Synonym,
    /// Broader concept (behind the hypernym toggle, layer 3).
    Hypernym,
    /// Narrower concept (behind the hypernym toggle, layer 3).
    Hyponym,
    /// User-added link in the current Universe.
    UserLink,
}

/// Weighted edge between two lemma nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub target: u32, // node index in `LexiconGraph::nodes`
    pub kind: EdgeKind,
    /// 0.0–1.0. Higher = more authoritative (WordNet > Wiktionary > User).
    /// Used to rank the first-returned equivalent when multiple candidates
    /// exist in the same target language.
    pub weight: f32,
}

/// The full lexicon graph, loaded once at boot.
///
/// Uses CSR (compressed sparse row) adjacency for cache-friendly traversal:
///   - `nodes[i]` holds the lemma node metadata.
///   - `edge_offsets[i..i+2]` gives the slice of `edges` belonging to node i.
///
/// Lookups from `(Lang, lemma)` → node index go through `name_index`,
/// which is a FST (finite-state transducer) for O(|lemma|) lookup without
/// hashing the full string. Same structure used for the Arabic engine's
/// FST — we reuse the compiler.
#[derive(Debug, Default)]
pub struct LexiconGraph {
    pub nodes: Vec<LemmaNode>,
    pub edge_offsets: Vec<u32>,
    pub edges: Vec<Edge>,
    /// Serialized FST bytes. Keys are `"{lang_code}:{lemma}"`, values are
    /// byte-offsets into the nodes array (packed when there are multiple
    /// senses for the same surface).
    pub name_index: Vec<u8>,
}

impl LexiconGraph {
    /// Total node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total edge count.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Load the core-tier graph from embedded binary data.
    ///
    /// During early development this returns an empty graph. M11 replaces
    /// the body with actual `include_bytes!` decoding of the compiled
    /// lexicon blob.
    pub fn load_core() -> Self {
        Self::default()
    }
}

//! Lexicon graph data structures — M10 architecture fill-out.
//!
//! The graph is loaded once at boot from embedded binary data (core tier)
//! plus any installed expansion packs (M13) and the current Universe's
//! user-overrides file (M14). After loading it is immutable; updates from
//! the learning loop append to the overrides file and trigger a focused
//! in-memory patch without a full rebuild.
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
//!   - `Synonym` — in-language near-equivalent (used for layer-2 expansion).
//!   - `Hypernym` — target is a broader concept ("knowledge" → "cognition").
//!   - `Hyponym` — target is a narrower concept.
//!   - `UserLink` — added by the Universe-level override layer (M14).
//!
//! # Storage footprint
//!
//! Core tier target: 20,000 concepts × ~10 langs with translations each ≈
//! 200K nodes and ~800K edges. Stored as a compact adjacency CSR format
//! (two parallel vecs: `edge_offsets` + `edges`) plus an `fst::Map` for
//! `(lang, lemma) → node_id` lookup. Total ≈ 14 MB uncompressed, ≈ 4 MB
//! gzipped in the binary. M10 ships a ~15-concept seed that exercises
//! every code path end-to-end; the real 20K seed arrives in M11.
//!
//! # FST key encoding
//!
//! Keys are `"{lang_code}:{normalized_lemma}"`, where `normalized_lemma`
//! is produced by [`normalize_for_lookup`]. Packing the language into
//! the key means a single FST covers all 15 languages with no index
//! explosion; the colon separator is not a valid first character in any
//! supported language code or lemma, so there is no ambiguity.
//!
//! # FST value encoding
//!
//! Values are `u64`: low 32 bits = first node index, high 32 bits = count
//! of *consecutive* node indices in the `nodes` vector that share this
//! normalized key (so a single surface with N senses becomes N contiguous
//! nodes + one FST entry). M10 only has single-sense concepts, so every
//! count is 1; the packing is prepared for M11's WordNet multi-sense data.

use super::bake;
use super::parse::{parse_with_diagnostics, ConceptRecord};
use crate::arabic::normalizer::normalize_stripped;
use crate::arabic::{Lang, PartOfSpeech};
use fst::{Map, MapBuilder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The embedded core-tier seed TSV, exposed as a `&'static str` so
/// [`bake::version_hash`] can hash its bytes deterministically for the
/// cache filename. Parallel to `arabic::roots::seed_tsv()`.
pub fn seed_tsv() -> &'static str {
    include_str!("data/seed_v1.tsv")
}

/// Opaque sense identifier. For WordNet-derived entries this is the
/// synset offset; for Wiktionary entries it is a hash of the sense
/// line; for user overrides it is `0` (single default sense). M10
/// only produces `DEFAULT` sense ids — M11 populates the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SenseId(pub u32);

impl SenseId {
    /// The default / unspecified sense. Used when the data source does
    /// not distinguish senses (loanwords, proper nouns, M10 seed rows).
    pub const DEFAULT: SenseId = SenseId(0);
}

/// A lemma node in the lexicon graph.
//
// `Hash` is intentionally not derived: `LemmaNode` carries an
// `Option<PartOfSpeech>` that the `arabic::types::PartOfSpeech` enum
// does not currently implement (and widening that derive would be a
// cross-module change). No caller hashes `LemmaNode` directly — nodes
// live in a `Vec<LemmaNode>` and are addressed by `u32` index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LemmaNode {
    pub lang: Lang,
    /// The lemma string (no morphological affixes, stored in the language's
    /// canonical form — e.g. Arabic uses the bare dictionary form with
    /// no definite article). Not normalized — the display form.
    pub lemma: String,
    /// Sense within this lemma. Multiple `LemmaNode` records can share
    /// `(lang, lemma)` but differ in `sense_id`.
    pub sense_id: SenseId,
    /// Part of speech, copied down from the `ConceptRecord` when the
    /// seed specified one. Lets M12's query expansion filter by POS
    /// (e.g. a Verb query should not expand into Noun neighbours).
    pub pos: Option<PartOfSpeech>,
    /// Concept ID this node belongs to — enables O(1) concept-to-labels
    /// lookups for the `expand()` walk.
    pub concept_id: String,
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
    /// exist in the same target language. M10's hand-picked seed uses a
    /// flat 1.0 — real provenance weighting arrives with M11's data.
    pub weight: f32,
}

/// The full lexicon graph, loaded once at boot.
///
/// Uses CSR (compressed sparse row) adjacency for cache-friendly traversal:
///   - `nodes[i]` holds the lemma node metadata.
///   - `edge_offsets[i..i+2]` gives the slice of `edges` belonging to node i.
///
/// Lookups from `(Lang, lemma)` → node indices go through `name_index`,
/// an `fst::Map` for O(|lemma|) lookup without hashing the full string.
/// Same structure used for the Arabic engine's generative FST — we reuse
/// the compiler.
pub struct LexiconGraph {
    pub nodes: Vec<LemmaNode>,
    /// One entry per node + one sentinel at the end. `edges[edge_offsets[i]..edge_offsets[i+1]]`
    /// slices to node i's adjacency list. Length is `nodes.len() + 1`.
    pub edge_offsets: Vec<u32>,
    pub edges: Vec<Edge>,
    /// FST mapping `"{lang_code}:{normalized_lemma}"` → packed `(offset, count)`
    /// of consecutive node indices sharing that key. See the module
    /// docs for the value encoding.
    pub name_index: Map<Vec<u8>>,
}

/// Serialisable form of [`LexiconGraph`] — everything the graph needs
/// to reconstruct itself, with the FST held as raw bytes so the bake
/// layer can persist it as an opaque blob.
///
/// The bundle is what `build_bundle` emits, what `bake::write_bundle`
/// persists to disk, and what `from_bundle` consumes when reconstructing
/// the live graph from either a fresh build or a cache hit. Cold and
/// warm init paths therefore go through **the exact same** reconstruction
/// step — no divergence, no "oh we forgot to apply X on the warm path"
/// class of bug. See `arabic::fst_bake` for the precedent.
#[derive(Debug)]
pub struct LexiconBundle {
    pub nodes: Vec<LemmaNode>,
    pub edge_offsets: Vec<u32>,
    pub edges: Vec<Edge>,
    pub name_index_bytes: Vec<u8>,
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

    /// Adjacency slice for node `i`. Empty slice if `i` is out of bounds.
    pub fn edges_of(&self, i: u32) -> &[Edge] {
        let i = i as usize;
        if i + 1 >= self.edge_offsets.len() {
            return &[];
        }
        let start = self.edge_offsets[i] as usize;
        let end = self.edge_offsets[i + 1] as usize;
        &self.edges[start..end]
    }

    /// Look up the nodes that match `(lang, lemma)`. Normalizes the query
    /// the same way the build step normalized the seed — Arabic-script
    /// diacritics are stripped, Latin-script input is lowercased. Returns
    /// node indices into `self.nodes`.
    ///
    /// Multiple hits are returned when a surface has multiple senses
    /// (WordNet polysemy, M11+). In M10 every hit is length 1.
    pub fn find_nodes(&self, lang: Lang, lemma: &str) -> Vec<u32> {
        let key = build_key(lang, &normalize_for_lookup(lang, lemma));
        let Some(packed) = self.name_index.get(&key) else {
            return Vec::new();
        };
        let offset = (packed & 0xFFFF_FFFF) as u32;
        let count = (packed >> 32) as u32;
        (offset..offset + count).collect()
    }

    /// Access the lazily-initialised singleton.
    ///
    /// Three-stage init, in preference order (mirrors `arabic::fst_index::GenerativeFst::get`):
    ///
    ///   1. **Cache hit** — [`bake::try_load_cached`] reads a bundle from
    ///      the user's cache directory (content-addressed by a hash of
    ///      [`seed_tsv()`] + [`bake::CACHE_FORMAT_VERSION`]). If the file
    ///      is present, well-formed, and parses back into a live `Map`,
    ///      we use it and skip parsing + FST compilation entirely. At the
    ///      M11 20K-concept scale this is where the cold-start savings
    ///      come from.
    ///   2. **Cache miss / corrupt cache** — fall through to an in-memory
    ///      parse + build via [`build_bundle`], then persist the result
    ///      best-effort so the *next* launch hits the cache. A failed
    ///      persist is silent: a read-only cache dir must never stop the
    ///      lexicon from loading.
    ///   3. **Final reconstruction** — the freshly-built bundle always
    ///      parses (we wrote those FST bytes seconds ago), so the
    ///      `expect` is unreachable in practice; we keep it as a
    ///      loud-failure tripwire in case a future refactor breaks the
    ///      build/parse invariant.
    ///
    /// Subsequent calls after the first are free — the [`OnceLock`]
    /// short-circuits.
    pub fn get() -> &'static LexiconGraph {
        static SINGLETON: OnceLock<LexiconGraph> = OnceLock::new();
        SINGLETON.get_or_init(|| {
            // Stage 1: try the on-disk cache. Any failure returns None and
            // we fall through — the cache layer never panics.
            if let Some(bundle) = bake::try_load_cached() {
                if let Ok(g) = Self::from_bundle(bundle) {
                    return g;
                }
                // FST-byte parse failure from a successfully-decoded
                // bundle would indicate a mismatch between what `bake`
                // wrote and what `fst::Map` can read — treat as corrupt
                // cache and rebuild. No panic.
            }

            // Stage 2: cold start. Build the bundle in memory, persist it
            // for next launch (best-effort), then hand the same bundle to
            // `from_bundle` so stages 1 and 2 go through the exact same
            // reconstruction path.
            let tsv = seed_tsv();
            let (records, errors) = parse_with_diagnostics(tsv);
            for (line, err) in &errors {
                eprintln!("lexicon seed row {line} skipped: {err:?}");
            }
            let bundle = build_bundle(records)
                .expect("lexicon seed failed to compile — this is a shipping bug");
            bake::persist_best_effort(&bundle);

            Self::from_bundle(bundle).expect(
                "freshly-built lexicon bundle must parse back into a live Map — \
                 if this trips, the build/parse invariant in build_bundle has \
                 been broken",
            )
        })
    }

    /// Load the core-tier graph from the embedded seed TSV. Public
    /// mainly so tests can reconstruct without touching the singleton
    /// or the disk cache.
    ///
    /// On any build-side error the function panics with a descriptive
    /// message rather than silently returning an empty graph — a broken
    /// lexicon at boot is a shipping bug that must surface loudly.
    pub fn load_core() -> Self {
        let tsv = seed_tsv();
        let (records, errors) = parse_with_diagnostics(tsv);
        for (line, err) in &errors {
            eprintln!("lexicon seed row {line} skipped: {err:?}");
        }
        let bundle = build_bundle(records)
            .expect("lexicon seed failed to compile — this is a shipping bug");
        Self::from_bundle(bundle)
            .expect("freshly-built lexicon bundle must parse — invariant broken")
    }

    /// Construct a graph for tests / diagnostics without touching the
    /// singleton or the embedded seed.
    pub fn from_records(records: Vec<ConceptRecord>) -> Result<Self, BuildError> {
        let bundle = build_bundle(records)?;
        Self::from_bundle(bundle)
    }

    /// Reconstruct a live [`LexiconGraph`] from a [`LexiconBundle`].
    /// Single entry point for both the cache-hit and cold-build paths so
    /// any bug in FST-byte handling surfaces identically in either.
    pub fn from_bundle(bundle: LexiconBundle) -> Result<Self, BuildError> {
        let name_index = Map::new(bundle.name_index_bytes)?;
        Ok(LexiconGraph {
            nodes: bundle.nodes,
            edge_offsets: bundle.edge_offsets,
            edges: bundle.edges,
            name_index,
        })
    }

    /// Snapshot this graph into a [`LexiconBundle`] for persistence. The
    /// returned bundle carries the FST's raw bytes (a copy of the
    /// underlying `Vec<u8>`) plus clones of the node / edge tables — so
    /// the caller owns a completely independent snapshot they can ship
    /// across a thread or to disk without touching the live graph.
    ///
    /// Used by tests that persist + reload via `bake::write_bundle` /
    /// `bake::load_bundle` against an arbitrary temp path. Production
    /// never calls this — `build_bundle` is the cold-path factory.
    pub fn to_bundle(&self) -> LexiconBundle {
        LexiconBundle {
            nodes: self.nodes.clone(),
            edge_offsets: self.edge_offsets.clone(),
            edges: self.edges.clone(),
            name_index_bytes: self.name_index.as_fst().as_bytes().to_vec(),
        }
    }

    /// Empty-graph constructor. Used by `Default` and by tests that
    /// exercise the zero-node edge cases of `find_nodes` / `edges_of`.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edge_offsets: vec![0], // sentinel — one more than nodes.len()
            edges: Vec::new(),
            name_index: Map::default(),
        }
    }
}

impl Default for LexiconGraph {
    fn default() -> Self {
        Self::empty()
    }
}

/// Errors produced by [`LexiconGraph::from_records`] / [`LexiconGraph::load_core`].
#[derive(Debug)]
pub enum BuildError {
    /// `fst::MapBuilder::insert` rejected a key (out of order or duplicate).
    /// Should not happen given our build pipeline sorts and dedups; an
    /// occurrence is a bug in [`build`].
    Fst(fst::Error),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Fst(e) => write!(f, "lexicon FST build failed: {e}"),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<fst::Error> for BuildError {
    fn from(e: fst::Error) -> Self {
        BuildError::Fst(e)
    }
}

/// Normalize a lemma for indexing and lookup. See the module docs.
pub fn normalize_for_lookup(lang: Lang, lemma: &str) -> String {
    match lang {
        // Arabic-script languages: strip diacritics, tatweel, and trim.
        // Case-folding is a no-op for these scripts so we skip the
        // `to_lowercase` pass.
        Lang::Ar | Lang::Fa | Lang::Ur => normalize_stripped(lemma.trim()),
        // Everything else: trim + case-fold. For CJK and Hebrew the fold
        // is a no-op; for Latin/Cyrillic/Turkish it unifies surface
        // variants. Turkish dotted-vs-dotless I edge cases are handled
        // by Rust's `to_lowercase` via Unicode case mapping.
        _ => lemma.trim().to_lowercase(),
    }
}

/// FST key builder — lang code prefix + colon + normalized lemma.
fn build_key(lang: Lang, normalized: &str) -> String {
    let mut s = String::with_capacity(3 + normalized.len());
    s.push_str(lang.code());
    s.push(':');
    s.push_str(normalized);
    s
}

/// Core builder: records → [`LexiconBundle`].
///
/// Returns a serialisable bundle rather than a live graph so the cache
/// layer (`bake::persist_best_effort`) and the reconstruction step
/// (`from_bundle`) can share the same intermediate representation —
/// cold- and warm-path init go through the same `from_bundle` call so
/// any FST-byte handling bug surfaces identically in either.
///
/// Algorithm (all O(n) over total lemma count):
///
/// 1. Flatten records into a `(lang_code, normalized_lemma, display_lemma,
///    pos, concept_id)` tuple list. Multi-lemma columns expand into
///    multiple tuples sharing `concept_id`.
/// 2. Sort by `(lang_code, normalized_lemma)` so consecutive entries with
///    the same normalized key become contiguous — that is the packing
///    the FST expects.
/// 3. Walk the sorted list:
///    - emit a `LemmaNode` per tuple in order → this is `nodes`.
///    - on each transition to a new `(lang_code, normalized)` key,
///      emit one FST entry pointing to the first node of the group.
/// 4. Walk the records again to emit `Edge`s:
///    - `Equivalent` between every pair of nodes from *different*
///      languages within the same concept.
///    - `Synonym` between every pair of nodes from the *same* language
///      within the same concept (multi-lemma columns).
/// 5. Sort each node's outgoing edges by (`kind`, `target`) then compact
///    into CSR — `edge_offsets` of length `nodes.len() + 1`.
pub fn build_bundle(records: Vec<ConceptRecord>) -> Result<LexiconBundle, BuildError> {
    #[derive(Clone)]
    struct Entry {
        lang: Lang,
        normalized: String,
        display: String,
        pos: Option<PartOfSpeech>,
        concept_id: String,
    }

    // Step 1: flatten.
    let mut entries: Vec<Entry> = Vec::new();
    for rec in &records {
        for (&lang, lemmas) in &rec.labels {
            for lemma in lemmas {
                let normalized = normalize_for_lookup(lang, lemma);
                if normalized.is_empty() {
                    continue;
                }
                entries.push(Entry {
                    lang,
                    normalized,
                    display: lemma.clone(),
                    pos: rec.pos,
                    concept_id: rec.id.clone(),
                });
            }
        }
    }

    // Step 2: sort by `(lang_code, normalized)`. Using `code()` keeps the
    // sort key identical to the FST key prefix we'll emit in step 3 —
    // anything else risks the MapBuilder rejecting entries out of order.
    entries.sort_by(|a, b| {
        a.lang
            .code()
            .cmp(b.lang.code())
            .then_with(|| a.normalized.cmp(&b.normalized))
    });

    // Step 3: build nodes + FST name_index in one pass.
    let mut nodes: Vec<LemmaNode> = Vec::with_capacity(entries.len());
    // Build the FST into memory. `MapBuilder::memory()` returns a builder
    // that collects bytes in a Vec<u8>; we pull them out with `into_inner()`
    // and wrap in a `Map<Vec<u8>>`.
    let mut fst_builder = MapBuilder::memory();
    let mut i = 0;
    while i < entries.len() {
        let group_start = i;
        let key_lang = entries[i].lang;
        let key_norm = entries[i].normalized.clone();
        // Group contiguous entries with the same (lang, normalized).
        while i < entries.len()
            && entries[i].lang == key_lang
            && entries[i].normalized == key_norm
        {
            nodes.push(LemmaNode {
                lang: entries[i].lang,
                lemma: entries[i].display.clone(),
                sense_id: SenseId::DEFAULT,
                pos: entries[i].pos,
                concept_id: entries[i].concept_id.clone(),
            });
            i += 1;
        }
        let offset = group_start as u64;
        let count = (i - group_start) as u64;
        let packed = (count << 32) | (offset & 0xFFFF_FFFF);
        let key = build_key(key_lang, &key_norm);
        fst_builder.insert(key, packed)?;
    }
    let name_index_bytes = fst_builder.into_inner()?;

    // Step 4: emit edges.
    //
    // For each concept, look up every node that belongs to it (via a
    // pre-built index from concept_id → Vec<node_idx>), then produce
    // O(n²) pair edges within that concept — Equivalent if cross-lang,
    // Synonym if same-lang. This is intentionally quadratic per-concept:
    // each concept has ≤30 nodes, so worst case ~900 pair ops per
    // concept × 20K concepts = ~18M ops at full scale, still well under
    // a millisecond of build time.
    let mut concept_to_nodes: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        concept_to_nodes
            .entry(node.concept_id.as_str())
            .or_default()
            .push(idx as u32);
    }

    // `edges_by_source[i]` accumulates node `i`'s outgoing edges.
    let mut edges_by_source: Vec<Vec<Edge>> = vec![Vec::new(); nodes.len()];
    for concept_node_ids in concept_to_nodes.values() {
        for &a in concept_node_ids {
            for &b in concept_node_ids {
                if a == b {
                    continue;
                }
                let kind = if nodes[a as usize].lang == nodes[b as usize].lang {
                    EdgeKind::Synonym
                } else {
                    EdgeKind::Equivalent
                };
                edges_by_source[a as usize].push(Edge {
                    target: b,
                    kind,
                    weight: 1.0,
                });
            }
        }
    }

    // Step 5: CSR-compact the adjacency lists. Sort each node's edges
    // by (kind, target) so iteration order is deterministic — tests
    // that assert on edge ordering depend on this.
    let mut edge_offsets: Vec<u32> = Vec::with_capacity(nodes.len() + 1);
    let mut edges: Vec<Edge> = Vec::new();
    edge_offsets.push(0);
    for src_edges in &mut edges_by_source {
        src_edges.sort_by(|a, b| {
            (a.kind as u8, a.target).cmp(&(b.kind as u8, b.target))
        });
        edges.extend_from_slice(src_edges);
        edge_offsets.push(edges.len() as u32);
    }

    Ok(LexiconBundle {
        nodes,
        edge_offsets,
        edges,
        name_index_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::parse::parse;

    fn tiny_graph() -> LexiconGraph {
        let tsv = "\
c:book\tNoun\ten:book,books\tar:كتاب\tfr:livre
c:read\tVerb\ten:read\tar:قرأ
";
        let recs = parse(tsv);
        LexiconGraph::from_records(recs).unwrap()
    }

    #[test]
    fn empty_graph_has_sentinel_offset() {
        let g = LexiconGraph::empty();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
        assert_eq!(g.edge_offsets, vec![0]);
    }

    #[test]
    fn find_returns_nothing_on_empty_graph() {
        let g = LexiconGraph::empty();
        assert!(g.find_nodes(Lang::En, "knowledge").is_empty());
    }

    #[test]
    fn builds_graph_from_tsv() {
        let g = tiny_graph();
        // c:book contributes 4 nodes (en:book + en:books + ar:كتاب + fr:livre)
        // c:read contributes 2 nodes (en:read + ar:قرأ)
        assert_eq!(g.node_count(), 6);
    }

    #[test]
    fn find_nodes_round_trips() {
        let g = tiny_graph();
        let ids = g.find_nodes(Lang::En, "book");
        assert_eq!(ids.len(), 1);
        assert_eq!(g.nodes[ids[0] as usize].lemma, "book");
        assert_eq!(g.nodes[ids[0] as usize].lang, Lang::En);
    }

    #[test]
    fn find_nodes_normalizes_case() {
        let g = tiny_graph();
        assert_eq!(g.find_nodes(Lang::En, "BOOK"), g.find_nodes(Lang::En, "book"));
        assert_eq!(g.find_nodes(Lang::En, "  book  "), g.find_nodes(Lang::En, "book"));
    }

    #[test]
    fn find_nodes_strips_arabic_diacritics() {
        let g = tiny_graph();
        // كِتَاب (with diacritics) should normalize to كتاب.
        let ids_vocalized = g.find_nodes(Lang::Ar, "كِتَاب");
        let ids_stripped = g.find_nodes(Lang::Ar, "كتاب");
        assert!(!ids_stripped.is_empty());
        assert_eq!(ids_vocalized, ids_stripped);
    }

    #[test]
    fn find_nodes_misses_cross_language() {
        let g = tiny_graph();
        // "book" is English — looking it up under Arabic must find nothing.
        assert!(g.find_nodes(Lang::Ar, "book").is_empty());
    }

    #[test]
    fn edges_of_book_reaches_all_concept_siblings() {
        let g = tiny_graph();
        let ids = g.find_nodes(Lang::En, "book");
        let edges = g.edges_of(ids[0]);
        // Sibling nodes inside c:book: en:books (Synonym), ar:كتاب + fr:livre
        // (Equivalent). So 3 edges total out of en:book.
        assert_eq!(edges.len(), 3);
        let kinds: Vec<_> = edges.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds.iter().filter(|&&k| k == EdgeKind::Synonym).count(),
            1,
            "expected one Synonym edge to `books`"
        );
        assert_eq!(
            kinds.iter().filter(|&&k| k == EdgeKind::Equivalent).count(),
            2,
            "expected two Equivalent edges (ar, fr)"
        );
    }

    #[test]
    fn edges_are_sorted_deterministically() {
        let g = tiny_graph();
        let ids = g.find_nodes(Lang::En, "book");
        let edges = g.edges_of(ids[0]);
        // After the (kind, target) sort, Equivalents come before Synonyms
        // since `EdgeKind::Equivalent as u8 == 0` and `Synonym as u8 == 1`.
        for w in edges.windows(2) {
            let a = (w[0].kind as u8, w[0].target);
            let b = (w[1].kind as u8, w[1].target);
            assert!(a <= b, "edges not sorted: {:?} then {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn edges_of_out_of_bounds_is_empty() {
        let g = tiny_graph();
        assert!(g.edges_of(99).is_empty());
    }

    #[test]
    fn seed_builds_via_singleton() {
        let g = LexiconGraph::get();
        assert!(
            g.node_count() > 50,
            "seed should produce substantial node count, got {}",
            g.node_count()
        );
        // Sanity: English "book" resolves; Arabic كتاب resolves.
        assert!(!g.find_nodes(Lang::En, "book").is_empty());
        assert!(!g.find_nodes(Lang::Ar, "كتاب").is_empty());
    }

    #[test]
    fn node_pos_is_populated_from_seed() {
        let g = tiny_graph();
        let ids = g.find_nodes(Lang::En, "book");
        assert_eq!(g.nodes[ids[0] as usize].pos, Some(PartOfSpeech::Noun));
        let ids = g.find_nodes(Lang::En, "read");
        assert_eq!(g.nodes[ids[0] as usize].pos, Some(PartOfSpeech::Verb));
    }

    #[test]
    fn concept_id_is_stamped_on_every_node() {
        let g = tiny_graph();
        for id in g.find_nodes(Lang::En, "book") {
            assert_eq!(g.nodes[id as usize].concept_id, "c:book");
        }
    }
}

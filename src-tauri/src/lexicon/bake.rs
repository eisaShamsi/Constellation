//! M11-infra — on-disk cache for the compiled Lexical Bridge graph.
//!
//! The M10 [`crate::lexicon::graph::LexiconGraph`] is a pure function of
//! the embedded seed TSV (`data/seed_v1.tsv`). Once M11 ships the 20K-
//! concept core bundle, that same function produces an ~14 MB in-memory
//! graph whose cold build takes ~200–400 ms (parse + FST compile +
//! O(concept²) edge emission). Every launch paying that cost is wasted
//! work — the bundle is identical on every boot until the seed edits.
//!
//! This module applies the same Write-Time Derivation pattern that
//! [`crate::arabic::fst_bake`] uses for the Arabic analyzer FST:
//!
//!   * compute once from the seed → [`crate::lexicon::graph::build_bundle`];
//!   * persist the resulting [`LexiconBundle`] to the user's cache dir;
//!   * on next launch, [`try_load_cached`] rehydrates the bundle and
//!     [`crate::lexicon::graph::LexiconGraph::from_bundle`] wraps the
//!     FST bytes in a live `Map`.
//!
//! Cold- and warm-start paths both end in `from_bundle`, so any FST-byte
//! handling bug would surface identically in either — there is no
//! "works on rebuild but not on cache hit" failure mode.
//!
//! ## File layout
//!
//! The cache is a single self-describing binary blob. All multi-byte
//! integers are little-endian. String lengths are `u32`; node / edge
//! counts and the FST byte length are `u64`.
//!
//! ```text
//!   magic:         [u8; 8]  = b"CAELEX01"
//!   version_hash:  u64      (djb2(seed_tsv) XOR CACHE_FORMAT_VERSION)
//!   node_count:    u64
//!   nodes:         [LemmaNode_encoded × node_count]
//!   offset_count:  u64      (= node_count + 1, sentinel-terminated)
//!   edge_offsets:  [u32 × offset_count]
//!   edge_count:    u64
//!   edges:         [Edge_encoded × edge_count]
//!   fst_byte_len:  u64
//!   fst_bytes:     [u8]
//!
//!   LemmaNode_encoded:
//!     lang_tag:      u8         (see encode_lang / decode_lang)
//!     lemma_len:     u32
//!     lemma_bytes:   [u8]       UTF-8
//!     sense_id:      u32
//!     pos_tag:       u8         (0 = None, 1..=8 = Some(<variant>))
//!     concept_len:   u32
//!     concept_bytes: [u8]       UTF-8
//!
//!   Edge_encoded:
//!     target:        u32
//!     kind_tag:      u8         (see encode_edge_kind / decode_edge_kind)
//!     weight:        f32        (little-endian bits)
//! ```
//!
//! ## Invalidation
//!
//! The cache filename includes the `version_hash`, so any edit to the
//! seed TSV flips the filename and the old cache is orphaned. Editing
//! the encoder / decoder (e.g. reordering `PartOfSpeech` variants)
//! requires bumping [`CACHE_FORMAT_VERSION`] — the old hash no longer
//! matches the compiled-in const, a rebuild triggers, and everyone is
//! consistent again.
//!
//! ## Failure policy
//!
//! Every disk operation is best-effort: a corrupt file, a read-only
//! cache dir, a concurrent writer — **none** of these stop the lexicon
//! from loading. On any persistence error we silently fall back to
//! building in-memory, so the worst case is the M10 behaviour (parse +
//! FST compile on every launch).

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::arabic::{Lang, PartOfSpeech};

use super::graph::{seed_tsv, Edge, EdgeKind, LemmaNode, LexiconBundle, SenseId};

/// Bump whenever the encoder / decoder layout, tag assignments, or any
/// shared structural invariant changes. Folded into the version hash so
/// the next launch detects the mismatch and transparently rebuilds.
/// Seed-TSV edits do NOT need a bump — the TSV content is already part
/// of the hash.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// Magic bytes at the start of the cache file. Guards against loading
/// an unrelated `lexicon-*.bin` from a different application or a
/// partial write from a previous process. Distinct from the arabic
/// FST cache's `CAEFST01` so the two caches never get confused.
const MAGIC: [u8; 8] = *b"CAELEX01";

// ──────────────────────────────────────────────────────────────────────
// Cache path resolution
// ──────────────────────────────────────────────────────────────────────

/// Resolve the preferred cache path for the compiled lexicon bundle,
/// or `None` if the platform's cache directory can't be determined.
///
/// Path shape: `<cache_dir>/constellation/lexicon-v{hash:016x}.bin`.
/// The hash is baked into the filename so stale caches from previous
/// seed versions coexist peacefully.
pub fn cache_file_path() -> Option<PathBuf> {
    let base = dirs::cache_dir()?;
    let hash = version_hash();
    Some(
        base.join("constellation")
            .join(format!("lexicon-v{hash:016x}.bin")),
    )
}

/// Content-addressed version identifier. Stable across processes with
/// the same binary, changes on any seed edit (automatic — seed content
/// is hashed) or encoder-layout change (manual — bump
/// [`CACHE_FORMAT_VERSION`]).
///
/// Uses djb2 rather than `std::hash::DefaultHasher` because
/// `DefaultHasher` is explicitly documented as unstable across Rust
/// releases — we need stability here or a compiler upgrade would
/// orphan every user's cache.
pub fn version_hash() -> u64 {
    static HASH: OnceLock<u64> = OnceLock::new();
    *HASH.get_or_init(|| djb2(seed_tsv().as_bytes()) ^ (CACHE_FORMAT_VERSION as u64))
}

/// djb2 — chosen for determinism and simplicity. Not cryptographic,
/// not meant to be: we only need collision resistance strong enough
/// that two *intentional* version changes produce two different
/// filenames.
fn djb2(bytes: &[u8]) -> u64 {
    let mut h: u64 = 5381;
    for &b in bytes {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

// ──────────────────────────────────────────────────────────────────────
// Load / persist — public API
// ──────────────────────────────────────────────────────────────────────

/// Try to load a previously-baked bundle from the preferred cache path.
/// Returns `None` on any error (missing file, wrong magic, version-hash
/// mismatch, truncation, unreadable enum tag, I/O failure). Never
/// panics — falling back to an in-memory rebuild is always safe.
pub fn try_load_cached() -> Option<LexiconBundle> {
    let path = cache_file_path()?;
    load_bundle(&path).ok()
}

/// Persist a bundle to the preferred cache path, best-effort. Any
/// error is swallowed — a read-only or full cache dir must never gate
/// the lexicon from loading.
pub fn persist_best_effort(bundle: &LexiconBundle) {
    if let Some(path) = cache_file_path() {
        let _ = write_bundle(&path, bundle);
    }
}

/// Read a bundle from an arbitrary path. Exposed for tests so we don't
/// race the user's real cache directory.
pub fn load_bundle(path: &std::path::Path) -> io::Result<LexiconBundle> {
    let mut file = fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    decode_bundle(&buf)
}

/// Write a bundle to an arbitrary path. Creates parent directories if
/// needed. Atomic: stages to `<path>.tmp`, renames on success.
pub fn write_bundle(path: &std::path::Path, bundle: &LexiconBundle) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let encoded = encode_bundle(bundle);
    // Atomic-ish write: stage to `<path>.tmp`, rename on success. Avoids
    // a partial file if we die mid-write.
    let tmp = path.with_extension("bin.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&encoded)?;
        f.sync_all().ok(); // best-effort fsync
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────
// Encode / decode
// ──────────────────────────────────────────────────────────────────────

fn encode_bundle(bundle: &LexiconBundle) -> Vec<u8> {
    // Pre-size to roughly the final length. Each node carries ~30 bytes
    // of fixed-layout fields plus two variable-length strings; each
    // edge is 9 bytes fixed. Overshoot is harmless, it just saves a
    // couple of reallocations on a big corpus.
    let rough = 8 + 8
        + 8 + bundle.nodes.len() * 40
        + 8 + bundle.edge_offsets.len() * 4
        + 8 + bundle.edges.len() * 9
        + 8 + bundle.name_index_bytes.len();
    let mut out = Vec::with_capacity(rough);

    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&version_hash().to_le_bytes());

    // Nodes.
    out.extend_from_slice(&(bundle.nodes.len() as u64).to_le_bytes());
    for n in &bundle.nodes {
        encode_node(&mut out, n);
    }

    // Edge offsets (u32 each).
    out.extend_from_slice(&(bundle.edge_offsets.len() as u64).to_le_bytes());
    for &off in &bundle.edge_offsets {
        out.extend_from_slice(&off.to_le_bytes());
    }

    // Edges.
    out.extend_from_slice(&(bundle.edges.len() as u64).to_le_bytes());
    for e in &bundle.edges {
        encode_edge(&mut out, e);
    }

    // FST bytes.
    out.extend_from_slice(&(bundle.name_index_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&bundle.name_index_bytes);

    out
}

fn encode_node(out: &mut Vec<u8>, node: &LemmaNode) {
    out.push(encode_lang(node.lang));
    encode_str(out, &node.lemma);
    out.extend_from_slice(&node.sense_id.0.to_le_bytes());
    out.push(encode_pos(node.pos));
    encode_str(out, &node.concept_id);
}

fn encode_edge(out: &mut Vec<u8>, edge: &Edge) {
    out.extend_from_slice(&edge.target.to_le_bytes());
    out.push(encode_edge_kind(edge.kind));
    out.extend_from_slice(&edge.weight.to_le_bytes());
}

fn encode_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

fn decode_bundle(buf: &[u8]) -> io::Result<LexiconBundle> {
    let mut cur = Cursor { buf, pos: 0 };

    let magic = cur.read_bytes(8)?;
    if magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "lexicon cache: magic mismatch (not our file, or corruption)",
        ));
    }

    let got_hash = cur.read_u64()?;
    if got_hash != version_hash() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "lexicon cache: version hash mismatch (seed or format changed)",
        ));
    }

    // Nodes.
    let node_count = cur.read_u64()? as usize;
    // Sanity cap — an encoded bundle claiming 10B nodes is either hostile
    // input or a length-field swap. Reject before allocating a giant Vec.
    const MAX_NODES: usize = 10_000_000;
    if node_count > MAX_NODES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lexicon cache: implausible node_count {node_count}"),
        ));
    }
    let mut nodes = Vec::with_capacity(node_count);
    for _ in 0..node_count {
        nodes.push(decode_node(&mut cur)?);
    }

    // Edge offsets.
    let offset_count = cur.read_u64()? as usize;
    // Must be node_count + 1 (sentinel). Reject any other shape rather
    // than silently accepting a malformed file.
    if offset_count != node_count + 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "lexicon cache: edge_offsets length {offset_count} disagrees \
                 with node_count+1 ({})",
                node_count + 1
            ),
        ));
    }
    let mut edge_offsets = Vec::with_capacity(offset_count);
    for _ in 0..offset_count {
        edge_offsets.push(cur.read_u32()?);
    }

    // Edges.
    let edge_count = cur.read_u64()? as usize;
    const MAX_EDGES: usize = 100_000_000;
    if edge_count > MAX_EDGES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lexicon cache: implausible edge_count {edge_count}"),
        ));
    }
    let mut edges = Vec::with_capacity(edge_count);
    for _ in 0..edge_count {
        edges.push(decode_edge(&mut cur)?);
    }

    // FST bytes.
    let fst_len = cur.read_u64()? as usize;
    let name_index_bytes = cur.read_bytes(fst_len)?.to_vec();

    // Trailing garbage is a protocol violation — reject rather than
    // silently accept.
    if cur.pos != cur.buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "lexicon cache: {} trailing bytes after FST block",
                cur.buf.len() - cur.pos
            ),
        ));
    }

    Ok(LexiconBundle {
        nodes,
        edge_offsets,
        edges,
        name_index_bytes,
    })
}

fn decode_node(cur: &mut Cursor) -> io::Result<LemmaNode> {
    let lang_tag = cur.read_u8()?;
    let lang = decode_lang(lang_tag).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lexicon cache: unknown Lang tag {lang_tag}"),
        )
    })?;
    let lemma = decode_str(cur)?;
    let sense_id = SenseId(cur.read_u32()?);
    let pos_tag = cur.read_u8()?;
    let pos = decode_pos(pos_tag).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lexicon cache: unknown PartOfSpeech tag {pos_tag}"),
        )
    })?;
    let concept_id = decode_str(cur)?;
    Ok(LemmaNode {
        lang,
        lemma,
        sense_id,
        pos,
        concept_id,
    })
}

fn decode_edge(cur: &mut Cursor) -> io::Result<Edge> {
    let target = cur.read_u32()?;
    let kind_tag = cur.read_u8()?;
    let kind = decode_edge_kind(kind_tag).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lexicon cache: unknown EdgeKind tag {kind_tag}"),
        )
    })?;
    let weight = f32::from_le_bytes(cur.read_array_4()?);
    Ok(Edge {
        target,
        kind,
        weight,
    })
}

fn decode_str(cur: &mut Cursor) -> io::Result<String> {
    let len = cur.read_u32()? as usize;
    let bytes = cur.read_bytes(len)?.to_vec();
    String::from_utf8(bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lexicon cache: invalid UTF-8 in string field: {e}"),
        )
    })
}

// ──────────────────────────────────────────────────────────────────────
// Lang <-> u8 tag
//
// Hand-coded rather than `#[repr(u8)] as u8` so a developer reordering
// the enum for readability cannot silently reshuffle tags and orphan
// existing caches. Tags are append-only: NEVER renumber an existing
// one. When adding a new Lang, give it the next unused tag here AND
// bump `CACHE_FORMAT_VERSION` so old caches invalidate.
// ──────────────────────────────────────────────────────────────────────

fn encode_lang(lang: Lang) -> u8 {
    match lang {
        Lang::Ar => 0,
        Lang::De => 1,
        Lang::En => 2,
        Lang::Es => 3,
        Lang::Fa => 4,
        Lang::Fr => 5,
        Lang::He => 6,
        Lang::Hi => 7,
        Lang::Ja => 8,
        Lang::Ko => 9,
        Lang::Pt => 10,
        Lang::Ru => 11,
        Lang::Tr => 12,
        Lang::Ur => 13,
        Lang::Zh => 14,
    }
}

fn decode_lang(tag: u8) -> Option<Lang> {
    Some(match tag {
        0 => Lang::Ar,
        1 => Lang::De,
        2 => Lang::En,
        3 => Lang::Es,
        4 => Lang::Fa,
        5 => Lang::Fr,
        6 => Lang::He,
        7 => Lang::Hi,
        8 => Lang::Ja,
        9 => Lang::Ko,
        10 => Lang::Pt,
        11 => Lang::Ru,
        12 => Lang::Tr,
        13 => Lang::Ur,
        14 => Lang::Zh,
        _ => return None,
    })
}

// ──────────────────────────────────────────────────────────────────────
// PartOfSpeech <-> u8 tag (0 = None, 1..=8 = Some(variant))
// ──────────────────────────────────────────────────────────────────────

fn encode_pos(pos: Option<PartOfSpeech>) -> u8 {
    match pos {
        None => 0,
        Some(PartOfSpeech::Noun) => 1,
        Some(PartOfSpeech::Verb) => 2,
        Some(PartOfSpeech::Adjective) => 3,
        Some(PartOfSpeech::Adverb) => 4,
        Some(PartOfSpeech::ProperNoun) => 5,
        Some(PartOfSpeech::Particle) => 6,
        Some(PartOfSpeech::Foreign) => 7,
        Some(PartOfSpeech::Unknown) => 8,
    }
}

fn decode_pos(tag: u8) -> Option<Option<PartOfSpeech>> {
    Some(match tag {
        0 => None,
        1 => Some(PartOfSpeech::Noun),
        2 => Some(PartOfSpeech::Verb),
        3 => Some(PartOfSpeech::Adjective),
        4 => Some(PartOfSpeech::Adverb),
        5 => Some(PartOfSpeech::ProperNoun),
        6 => Some(PartOfSpeech::Particle),
        7 => Some(PartOfSpeech::Foreign),
        8 => Some(PartOfSpeech::Unknown),
        _ => return None,
    })
}

// ──────────────────────────────────────────────────────────────────────
// EdgeKind <-> u8 tag
// ──────────────────────────────────────────────────────────────────────

fn encode_edge_kind(kind: EdgeKind) -> u8 {
    match kind {
        EdgeKind::Equivalent => 0,
        EdgeKind::Synonym => 1,
        EdgeKind::Hypernym => 2,
        EdgeKind::Hyponym => 3,
        EdgeKind::UserLink => 4,
    }
}

fn decode_edge_kind(tag: u8) -> Option<EdgeKind> {
    Some(match tag {
        0 => EdgeKind::Equivalent,
        1 => EdgeKind::Synonym,
        2 => EdgeKind::Hypernym,
        3 => EdgeKind::Hyponym,
        4 => EdgeKind::UserLink,
        _ => return None,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Byte cursor — bounded reads that Err cleanly on short buffers
// instead of panicking on an out-of-bounds slice access.
// ──────────────────────────────────────────────────────────────────────

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn read_bytes(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "length overflow"))?;
        if end > self.buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "lexicon cache: wanted {n} bytes at {}, only {} left",
                    self.pos,
                    self.buf.len() - self.pos
                ),
            ));
        }
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_array_4(&mut self) -> io::Result<[u8; 4]> {
        let b = self.read_bytes(4)?;
        Ok([b[0], b[1], b[2], b[3]])
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::graph::{build_bundle, legacy_seed_tsv, LexiconGraph};
    use crate::lexicon::parse::parse;

    fn sample_bundle() -> LexiconBundle {
        // Minimal hand-built bundle: one lemma node, no edges, empty FST.
        // We don't need a real FST for encode/decode roundtrip tests —
        // the cache treats FST bytes as opaque.
        LexiconBundle {
            nodes: vec![LemmaNode {
                lang: Lang::En,
                lemma: "book".to_string(),
                sense_id: SenseId::DEFAULT,
                pos: Some(PartOfSpeech::Noun),
                concept_id: "c:book".to_string(),
            }],
            edge_offsets: vec![0, 0], // node_count + 1, zero-length slice
            edges: Vec::new(),
            name_index_bytes: fst::MapBuilder::memory().into_inner().unwrap(),
        }
    }

    fn tmp_path(label: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        p.push(format!("constellation-lexicon-bake-{label}-{pid}-{nanos}.bin"));
        p
    }

    #[test]
    fn encode_lang_is_injective_and_total() {
        use Lang::*;
        let all = [Ar, De, En, Es, Fa, Fr, He, Hi, Ja, Ko, Pt, Ru, Tr, Ur, Zh];
        assert_eq!(all.len(), 15, "Lang::all should yield all 15 variants");
        for l in all {
            let tag = encode_lang(l);
            assert_eq!(decode_lang(tag), Some(l), "{l:?} round-trip");
        }
        let mut tags: Vec<u8> = all.iter().map(|&l| encode_lang(l)).collect();
        tags.sort_unstable();
        let n = tags.len();
        tags.dedup();
        assert_eq!(tags.len(), n, "duplicate Lang tags — orphans caches");
    }

    #[test]
    fn decode_lang_rejects_unknown_tag() {
        assert!(decode_lang(99).is_none());
        assert!(decode_lang(15).is_none(), "15 is one past the last valid tag");
    }

    #[test]
    fn encode_pos_is_injective_and_total_incl_none() {
        use PartOfSpeech::*;
        // 0 = None, then 1..=8 for the 8 variants.
        let states: &[Option<PartOfSpeech>] = &[
            None,
            Some(Noun),
            Some(Verb),
            Some(Adjective),
            Some(Adverb),
            Some(ProperNoun),
            Some(Particle),
            Some(Foreign),
            Some(Unknown),
        ];
        for &s in states {
            let tag = encode_pos(s);
            assert_eq!(decode_pos(tag), Some(s), "{s:?} round-trip");
        }
        let mut tags: Vec<u8> = states.iter().map(|&p| encode_pos(p)).collect();
        tags.sort_unstable();
        let n = tags.len();
        tags.dedup();
        assert_eq!(tags.len(), n, "duplicate POS tags — orphans caches");
    }

    #[test]
    fn decode_pos_rejects_unknown_tag() {
        assert!(decode_pos(99).is_none());
        assert!(decode_pos(9).is_none(), "9 is one past the last valid tag");
    }

    #[test]
    fn encode_edge_kind_is_injective_and_total() {
        use EdgeKind::*;
        let all = [Equivalent, Synonym, Hypernym, Hyponym, UserLink];
        for k in all {
            let tag = encode_edge_kind(k);
            assert_eq!(decode_edge_kind(tag), Some(k), "{k:?} round-trip");
        }
        let mut tags: Vec<u8> = all.iter().map(|&k| encode_edge_kind(k)).collect();
        tags.sort_unstable();
        let n = tags.len();
        tags.dedup();
        assert_eq!(tags.len(), n, "duplicate EdgeKind tags — orphans caches");
    }

    #[test]
    fn decode_edge_kind_rejects_unknown_tag() {
        assert!(decode_edge_kind(99).is_none());
    }

    #[test]
    fn encode_decode_node_roundtrip() {
        let node = LemmaNode {
            lang: Lang::Ar,
            lemma: "كتاب".to_string(),
            sense_id: SenseId(42),
            pos: Some(PartOfSpeech::Noun),
            concept_id: "c:book".to_string(),
        };
        let mut buf = Vec::new();
        encode_node(&mut buf, &node);
        let mut cur = Cursor { buf: &buf, pos: 0 };
        let back = decode_node(&mut cur).expect("decode");
        assert_eq!(back.lang, node.lang);
        assert_eq!(back.lemma, node.lemma);
        assert_eq!(back.sense_id, node.sense_id);
        assert_eq!(back.pos, node.pos);
        assert_eq!(back.concept_id, node.concept_id);
        assert_eq!(cur.pos, buf.len(), "decoded bytes must match encoded length");
    }

    #[test]
    fn encode_decode_edge_roundtrip() {
        let edge = Edge {
            target: 7,
            kind: EdgeKind::Equivalent,
            weight: 0.75,
        };
        let mut buf = Vec::new();
        encode_edge(&mut buf, &edge);
        let mut cur = Cursor { buf: &buf, pos: 0 };
        let back = decode_edge(&mut cur).expect("decode");
        assert_eq!(back.target, edge.target);
        assert_eq!(back.kind, edge.kind);
        assert!((back.weight - edge.weight).abs() < f32::EPSILON);
        assert_eq!(cur.pos, buf.len());
    }

    #[test]
    fn bundle_write_read_roundtrip() {
        let path = tmp_path("roundtrip");
        let original = sample_bundle();
        write_bundle(&path, &original).expect("write");
        let loaded = load_bundle(&path).expect("load");

        assert_eq!(loaded.nodes.len(), original.nodes.len());
        assert_eq!(loaded.nodes[0].lemma, original.nodes[0].lemma);
        assert_eq!(loaded.nodes[0].lang, original.nodes[0].lang);
        assert_eq!(loaded.edge_offsets, original.edge_offsets);
        assert_eq!(loaded.edges.len(), original.edges.len());
        assert_eq!(loaded.name_index_bytes, original.name_index_bytes);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_missing_file() {
        let path = tmp_path("missing");
        assert!(load_bundle(&path).is_err(), "missing file must Err");
    }

    #[test]
    fn load_rejects_wrong_magic() {
        let path = tmp_path("wrongmagic");
        fs::write(&path, b"NOTMAGIC....garbage....").unwrap();
        let err = load_bundle(&path).expect_err("wrong magic must Err");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_truncated_file() {
        let path = tmp_path("truncated");
        let mut encoded = encode_bundle(&sample_bundle());
        // Lop off the last half — the FST-len or edges sections truncate.
        encoded.truncate(encoded.len() / 2);
        fs::write(&path, &encoded).unwrap();
        assert!(load_bundle(&path).is_err(), "truncated file must Err");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_wrong_version_hash() {
        let path = tmp_path("wronghash");
        let mut encoded = encode_bundle(&sample_bundle());
        // Version hash lives at bytes [8..16] (right after magic). Flipping
        // any of its bytes simulates what a seed-TSV edit would look like
        // to the reader.
        encoded[8] ^= 0xFF;
        fs::write(&path, &encoded).unwrap();
        let err = load_bundle(&path).expect_err("wrong hash must Err");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_trailing_garbage() {
        let path = tmp_path("trailing");
        let mut encoded = encode_bundle(&sample_bundle());
        encoded.extend_from_slice(b"EXTRA BYTES");
        fs::write(&path, &encoded).unwrap();
        let err = load_bundle(&path).expect_err("trailing bytes must Err");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cache_file_path_includes_version_hash() {
        // Sanity: the filename carries the hash, which is how seed-TSV
        // changes invalidate caches even without rewriting the file
        // atomically.
        let Some(p) = cache_file_path() else {
            // Some minimal CI images have no cache dir — skip.
            return;
        };
        let name = p.file_name().and_then(|n| n.to_str()).unwrap();
        assert!(name.starts_with("lexicon-v"), "unexpected filename: {name}");
        assert!(name.ends_with(".bin"), "unexpected filename: {name}");
        let hash_hex = &name["lexicon-v".len()..name.len() - ".bin".len()];
        assert_eq!(
            hash_hex.len(),
            16,
            "hash must render as 16 lowercase hex chars (u64), got {hash_hex:?}"
        );
        assert!(
            hash_hex.chars().all(|c| c.is_ascii_hexdigit()),
            "hash filename segment must be pure hex, got {hash_hex:?}"
        );
    }

    #[test]
    fn version_hash_is_stable_across_calls() {
        // OnceLock caching means successive calls must never disagree.
        // If this ever fails, seed hashing has become nondeterministic
        // and every launch would rebuild the cache.
        let a = version_hash();
        let b = version_hash();
        assert_eq!(a, b);
    }

    #[test]
    fn djb2_matches_known_values() {
        // Hand-verified seeds — guards against a refactor accidentally
        // changing the hash output and silently orphaning every cache.
        // (If this test ever breaks intentionally, bump CACHE_FORMAT_VERSION
        // in the same commit.) djb2("") = 5381 per Knuth's "Notes on
        // Hashing".
        assert_eq!(djb2(b""), 5381);
        // djb2("a") = 5381 * 33 + 97 = 177670.
        assert_eq!(djb2(b"a"), 177670);
        // Determinism over an arbitrary blob.
        let x = djb2(b"hello world");
        let y = djb2(b"hello world");
        assert_eq!(x, y);
        assert_ne!(djb2(b"hello world"), djb2(b"hello worlD"));
    }

    #[test]
    fn real_seed_bundle_writes_reads_reconstructs() {
        // End-to-end: build the legacy M10 15-concept seed bundle,
        // write it to a temp path, read it back, reconstruct a graph,
        // verify lookups still resolve. This is the historical canary
        // — seed_v1.tsv is preserved on disk as the M10 regression
        // fixture, and this test guarantees it still parses cleanly
        // through every later encoder / decoder change.
        let tsv = legacy_seed_tsv();
        let recs = parse(tsv);
        let original = build_bundle(recs).expect("build_bundle legacy seed");

        let path = tmp_path("realseed");
        write_bundle(&path, &original).expect("write");
        let loaded = load_bundle(&path).expect("load");
        assert_eq!(loaded.nodes.len(), original.nodes.len());
        assert_eq!(loaded.edges.len(), original.edges.len());
        assert_eq!(loaded.edge_offsets, original.edge_offsets);
        assert_eq!(loaded.name_index_bytes, original.name_index_bytes);

        let g = LexiconGraph::from_bundle(loaded).expect("reconstruct");
        // Sanity: at least one known lookup still works.
        assert!(
            !g.find_nodes(Lang::En, "book").is_empty(),
            "en:book must resolve in the reconstructed graph"
        );
        assert!(
            !g.find_nodes(Lang::Ar, "كتاب").is_empty(),
            "ar:كتاب must resolve in the reconstructed graph"
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn real_lexicon_bundle_writes_reads_reconstructs() {
        // End-to-end canary for the production corpus (lexicon_v1.tsv,
        // the M11-data deliverable). Mirrors the legacy seed canary but
        // asserts a lookup (`en:tree`) that is only present in the
        // production corpus, guaranteeing the round-trip actually
        // consumed the larger file and not the legacy seed by accident.
        let tsv = seed_tsv();
        let recs = parse(tsv);
        // Production corpus is strictly larger than the legacy 15-row
        // seed; this assertion doubles as a "seed swap actually
        // happened" tripwire.
        assert!(
            recs.len() > 20,
            "lexicon_v1 must carry more concepts than the legacy seed \
             (got {} records — did seed_tsv() silently revert?)",
            recs.len()
        );
        let original = build_bundle(recs).expect("build_bundle real lexicon");

        let path = tmp_path("reallexicon");
        write_bundle(&path, &original).expect("write");
        let loaded = load_bundle(&path).expect("load");
        assert_eq!(loaded.nodes.len(), original.nodes.len());
        assert_eq!(loaded.edges.len(), original.edges.len());
        assert_eq!(loaded.edge_offsets, original.edge_offsets);
        assert_eq!(loaded.name_index_bytes, original.name_index_bytes);

        let g = LexiconGraph::from_bundle(loaded).expect("reconstruct");
        // Spot-check a lookup that only exists in the production
        // corpus (not in the legacy 15-concept seed).
        assert!(
            !g.find_nodes(Lang::En, "tree").is_empty(),
            "en:tree must resolve — it is in lexicon_v1 but not seed_v1"
        );
        // And the mandatory Arabic round-trip, per project rule: every
        // row must carry Arabic. Pick a concept that's Arabic-only in
        // the production corpus.
        assert!(
            !g.find_nodes(Lang::Ar, "شجرة").is_empty(),
            "ar:شجرة must resolve in the reconstructed production graph"
        );

        let _ = fs::remove_file(&path);
    }
}

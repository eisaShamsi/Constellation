//! CTSE Bridge Vector Store — MIG-013 Phase 1.
//!
//! Stores one 384-dim L2-normalized vector per M11 concept (~20K),
//! produced **at build time** by the offline `build_concept_vectors`
//! [[bin]] target and shipped as a binary asset:
//! `bridge_vectors/data/concept_vectors_v1.bin`.
//!
//! ## Layout (v1)
//!
//! ```text
//! [ 0..  8] magic         = b"CTSEBV01"
//! [ 8.. 12] count: u32 LE        ← number of concepts (e.g. 20_000)
//! [12.. 16] dim: u32 LE          ← 384 (multilingual-e5-small)
//! [16..  N] concept_id_table     ← count rows, each: u16 LE byte_len + UTF-8 bytes
//! [ N..end] vector_matrix        ← count × dim × f32 LE, row-major, L2-normalized
//! ```
//!
//! `N` is computed by reading the id table sequentially. After the
//! id table the file is 4-byte-aligned (the writer emits zero-pad
//! up to the next 4-byte boundary so the vector matrix is word-aligned
//! on disk; the runtime parser still copies into a fresh
//! `Box<[f32]>` so we don't depend on the in-memory placement of
//! `include_bytes!` data).
//!
//! ## Why the asset is build-time, not runtime
//!
//! Embedding the M11 corpus once at build time gives **constant-time
//! semantic search regardless of library size** (Architect v2 §2).
//! Per-library bootstrap freezes are impossible because there is no
//! per-library bootstrap.
//!
//! ## Public API
//!
//! - [`get`] — process-wide singleton; lazily parses on first call.
//! - [`ConceptVectorStore::nearest_concept`] — cosine top-1 over the
//!   flat matrix.
//! - [`ConceptVectorStore::nearest_concepts_k`] — top-`k` neighbors,
//!   descending score.

mod asset;
mod store;

pub use asset::{parse, ParsedAsset};
pub use store::ConceptVectorStore;

use std::sync::OnceLock;

/// Layout magic. Bumped if the on-disk format changes.
pub const ASSET_MAGIC: &[u8; 8] = b"CTSEBV01";

/// Embedding dimension produced by multilingual-e5-small.
pub const VECTOR_DIM: usize = 384;

/// Process-wide singleton. First access parses the baked asset
/// (~30 ms for the 30 MB asset on a modern machine — single
/// allocation + copy + per-row UTF-8 decode of concept ids).
/// Subsequent accesses are free.
///
/// Panics if the asset is corrupt — that's a build-time invariant
/// failure (the `build_concept_vectors` [[bin]] validates magic /
/// counts / dims / L2-norms before writing) and the only way to
/// recover is to rebuild the asset.
pub fn get() -> &'static ConceptVectorStore {
    static SINGLETON: OnceLock<ConceptVectorStore> = OnceLock::new();
    SINGLETON.get_or_init(|| {
        let parsed = parse().expect(
            "CTSE concept-vector asset failed to parse. \
             Rebuild via: cd src-tauri && cargo run --bin build_concept_vectors --release"
        );
        ConceptVectorStore::new(parsed.concept_ids, parsed.matrix)
            .expect("CTSE asset header consistent but matrix length wrong — bug")
    })
}

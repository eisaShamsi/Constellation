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
//! up to the next 4-byte boundary so the vector matrix is mmap-friendly).
//!
//! ## Why the asset is build-time, not runtime
//!
//! Embedding the M11 corpus once at build time gives **constant-time
//! semantic search regardless of library size** (Architect v2 §2).
//! Per-library bootstrap freezes are impossible because there is no
//! per-library bootstrap.
//!
//! ## Phase 1A — this file is a stub
//!
//! Phase 1A produces the asset only. Phase 1B fills in the runtime
//! loader (`load`, `nearest_concept`, `nearest_concepts_k`) and the
//! cosine k-NN. Phase 1C wires the adapter to the write path. See
//! `lab/reports/MIG-013-CTSE-PLAN.md`.

/// Layout magic. Bumped if the on-disk format changes.
pub const ASSET_MAGIC: &[u8; 8] = b"CTSEBV01";

/// Embedding dimension produced by multilingual-e5-small.
pub const VECTOR_DIM: usize = 384;

//! MIG-055 — Constellation Base (lens module entry).
//!
//! The lens module implements the new Constellation Base per the
//! Concept Paper v1.4 + the MIG-055 Architect v1.1. It is a clean
//! rebuild — no carryover from the old MVP (Bases pre-MIG-054).
//!
//! ## Layered architecture
//!
//! - **dimensions.rs** — the registry of cognitive dimensions a lens
//!   can reference (`note.name`, `note.created_at`, etc.). Future
//!   phases extend the registry with Living Link / CE / CNS / CECE
//!   dimensions.
//!
//! - **definition.rs** — the `LensDefinition` data shape (filled in §B).
//! - **parser.rs** — YAML → `LensDefinition` (filled in §B).
//! - **validator.rs** — schema-validation against the registry (§B).
//! - **sql_builder.rs** — `LensDefinition` → parameterized SQL (§C).
//! - **query.rs** — the `execute_lens` Tauri command (§C).
//! - **system_notes.rs** — system-shipped Five Acts host notes (§E).
//!
//! ## Naming convention (Architect §11 #1 lock)
//!
//! - `note.X` — per-note properties (name, path, created_at, stratum, …).
//! - `link.X` — Living Link properties (confidence, weight, lifecycle).
//! - `note.cns.X` — CNS measurements (community, centrality, top-bridge).
//! - `note.cece.X` — CECE classifications (source.primary, content_type.primary).
//!
//! Each prefix points future readers at the SOURCE surface for the
//! dimension. The naming is locked across all future phases.

pub mod dimensions;

pub use dimensions::{
    DimensionDef, DimensionKind, all_dimensions, dimension_names, lookup_dimension,
};

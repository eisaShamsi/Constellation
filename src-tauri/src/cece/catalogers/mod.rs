//! MIG-021v3 — Cataloger implementations.
//!
//! Each cataloger lives in its own file. Phases V3-§2 through V3-§7
//! ship them in cost order: cheap first (User-Authority, Structural,
//! Linguistic), then medium (Graph, Semantic), then expensive (Reasoning).

pub mod user_authority;
pub mod structural;

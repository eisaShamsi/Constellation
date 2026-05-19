//! MIG-036 P1 (2026-05-19) — Sight v7 Rust scaffolding.
//!
//! Parallel module to `sight_v6.rs` for the Form-Aligns-To-Purpose
//! redesign. P1 lands the module shell + flag-gating only; no IPCs
//! yet. Subsequent phases fill in:
//!
//! - P3 — universe-view IPC (per-cell density)
//! - P7 — cell drill-in IPC (per-cell stack of notes)
//! - P6 — Time Dome IPC variant (stratum × time, identical to v6's
//!   get_layout but renamed to make the time-anchor explicit)
//!
//! The cache schema is REUSED from `sight_v6_layout` (per Architect
//! §8). v7's IPCs read from the same table that v6's backfill +
//! invalidation triggers maintain. No new schema migration needed
//! for v7's data path.
//!
//! Architect doc: lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md

// Re-export the v6 cache shape since v7 reuses it. This is the
// only place v7 depends on v6's surface. After v6 retirement
// (MIG-037), this `pub use` becomes a local definition.
pub use crate::sight_v6::LayoutCacheRow;

/// MIG-036 P1 — placeholder marker confirming v7 module loads.
/// Removed in P3 when the first real IPC ships.
#[allow(dead_code)]
pub fn sight_v7_scaffolding_marker() -> &'static str {
    "MIG-036 P1 — Sight v7 scaffolding"
}

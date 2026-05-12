//! MIG-021v3 — Constellation Epistemic Content Engine (CECE).
//!
//! The CECE classifies notes along two orthogonal axes (Source +
//! Content Type) using a Cataloger Ensemble. Six methodologically
//! distinct catalogers each read the note through a different
//! Constellation primitive (CAE morphology, structural regex, Living
//! Links graph, embedding similarity, local LLM, frontmatter
//! authority); a synthesis layer combines their reasoning trails into
//! one of three confidence regimes (Unanimous / Strong-Majority /
//! Split). On Split, the engine refuses to assign and asks the user
//! via Sibling Disambiguation.
//!
//! See `lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md`
//! for the full architecture spec.
//!
//! V3-§1 Foundation: trait, types, synthesis, orchestrator, rules,
//! reliability tracking. No catalogers yet — they ship in V3-§2 through
//! V3-§7 in cost order.

pub mod cataloger;
pub mod orchestrator;
pub mod reliability;
pub mod rules;
pub mod synthesis;
pub mod wiring;

// Future submodule for the six catalogers (V3-§2 through V3-§7 will
// add files inside `cece/catalogers/`).
pub mod catalogers;

// MIG-022 §B — Note state history (temporal axis). Persists changes to
// epistemic frontmatter fields per the gap-analysis §6.3 recommendation.
// §B.1 ships the table + index; §B.2 the trigger; §B.3 the backfill;
// §B.4 the query API; §B.5 the Sight v3 overlay UI.
pub mod history;

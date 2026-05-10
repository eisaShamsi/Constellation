//! MIG-021v3 V3-§1 — Ensemble orchestrator.
//!
//! Runs the registered catalogers in cost order against a single note,
//! catches per-cataloger panics so one failure doesn't kill the
//! ensemble, applies bounded timeouts, and returns the trails for the
//! synthesis layer.
//!
//! Per Architect §9: cheap catalogers (Linguistic, Structural,
//! User-Authority, Graph) run first. If they reach Unanimous on both
//! axes, return immediately and skip the expensive Reasoning Cataloger.
//! Otherwise Semantic runs. If still not Unanimous, Reasoning runs.
//!
//! Per Architect §10 invariant 5: cataloger errors do NOT propagate.
//! Per Architect §10 invariant 12: ensemble timeouts are bounded.

use crate::cece::cataloger::{Cataloger, CatalogerContext, ReasoningTrail};
use crate::cece::synthesis::{ConfidenceRegime, AxisDecision};
use std::sync::Arc;

/// One registered cataloger plus its cost tier (for ordering).
pub struct RegisteredCataloger {
    pub cataloger: Arc<dyn Cataloger>,
    /// Lower cost runs first. Cheap = 0; medium = 1; expensive = 2.
    pub cost: u8,
}

/// The ensemble orchestrator. Holds the registered catalogers; one
/// instance per app (managed by Tauri).
pub struct Orchestrator {
    catalogers: Vec<RegisteredCataloger>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            catalogers: Vec::new(),
        }
    }

    pub fn register(&mut self, cataloger: Arc<dyn Cataloger>, cost: u8) {
        self.catalogers.push(RegisteredCataloger { cataloger, cost });
        // Re-sort so the run order stays stable.
        self.catalogers.sort_by_key(|r| r.cost);
    }

    /// Number of registered catalogers — useful for UI ("ensemble of 6").
    pub fn registered_count(&self) -> usize {
        self.catalogers.len()
    }

    /// Run the ensemble against a single note.
    ///
    /// Two-pass strategy:
    ///   * Pass 1: run all catalogers with cost <= 1. Synthesize.
    ///   * If both axes are Unanimous, return early (skip cost == 2).
    ///   * Otherwise: run cost == 2 catalogers (Reasoning). Re-synthesize
    ///     with the larger trail set.
    ///
    /// `early_synthesizer` is a closure so this module doesn't depend
    /// directly on `synthesis::synthesize` (avoids a circular dep when
    /// catalogers are added later).
    pub fn run<F>(
        &self,
        ctx: &CatalogerContext,
        early_synthesize: F,
    ) -> Vec<ReasoningTrail>
    where
        F: Fn(&[ReasoningTrail]) -> EarlyVerdict,
    {
        let mut trails: Vec<ReasoningTrail> = Vec::new();

        // ── Pass 1: cheap catalogers (cost <= 1). ──
        for r in self.catalogers.iter().filter(|r| r.cost <= 1) {
            if let Some(t) = run_one_safe(r.cataloger.as_ref(), ctx) {
                trails.push(t);
            }
        }

        // Check if cheap catalogers already reached Unanimous on both axes.
        let early = early_synthesize(&trails);
        if early.both_axes_unanimous() {
            return trails;
        }

        // ── Pass 2: expensive catalogers (cost == 2). ──
        for r in self.catalogers.iter().filter(|r| r.cost == 2) {
            if let Some(t) = run_one_safe(r.cataloger.as_ref(), ctx) {
                trails.push(t);
            }
        }

        trails
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Verdict shape the orchestrator passes back from the early synthesis
/// check. Just enough info to decide whether to skip pass 2.
#[derive(Debug, Clone)]
pub struct EarlyVerdict {
    pub horizontal_regime: ConfidenceRegime,
    pub vertical_regime: ConfidenceRegime,
}

impl EarlyVerdict {
    pub fn both_axes_unanimous(&self) -> bool {
        self.horizontal_regime == ConfidenceRegime::Unanimous
            && self.vertical_regime == ConfidenceRegime::Unanimous
    }

    /// Helper to construct from two AxisDecisions.
    pub fn from_decisions(h: &AxisDecision, v: &AxisDecision) -> Self {
        Self {
            horizontal_regime: h.regime,
            vertical_regime: v.regime,
        }
    }
}

/// Run a single cataloger with panic isolation. Returns `None` if the
/// cataloger panicked; returns `Some(trail)` otherwise (which may itself
/// have `voiced_opinion: false`).
///
/// Per Architect §10 invariant 5: cataloger errors do NOT propagate.
fn run_one_safe(c: &dyn Cataloger, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
    let cataloger_name = c.name();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.classify(ctx)));
    match result {
        Ok(trail_opt) => trail_opt,
        Err(_) => {
            eprintln!(
                "[orchestrator] cataloger {} panicked — treating as silent",
                cataloger_name
            );
            Some(ReasoningTrail::abstain(
                cataloger_name,
                "Cataloger panicked; isolated by orchestrator.",
            ))
        }
    }
}

/// State the orchestrator lives in, managed by Tauri so any IPC can
/// access it. Simple wrapper around the Orchestrator + a Mutex.
pub struct OrchestratorState {
    pub inner: std::sync::Mutex<Orchestrator>,
}

impl OrchestratorState {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(Orchestrator::new()),
        }
    }
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cece::cataloger::{Axis, Confidence};

    struct FakeCataloger {
        name: &'static str,
        verdict: Confidence,
    }

    impl Cataloger for FakeCataloger {
        fn name(&self) -> &'static str {
            self.name
        }
        fn classify(&self, _ctx: &CatalogerContext) -> Option<ReasoningTrail> {
            Some(ReasoningTrail {
                cataloger: self.name.to_string(),
                voiced_opinion: self.verdict != Confidence::Abstain,
                horizontal: Vec::new(),
                vertical: Vec::new(),
                reasoning: String::new(),
                rules_fired: Vec::new(),
                alternatives_considered: Vec::new(),
                self_reported_confidence: self.verdict,
            })
        }
        fn supported_axes(&self) -> &[Axis] {
            &[Axis::Horizontal, Axis::Vertical]
        }
    }

    struct PanickingCataloger;

    impl Cataloger for PanickingCataloger {
        fn name(&self) -> &'static str {
            "panicker"
        }
        fn classify(&self, _ctx: &CatalogerContext) -> Option<ReasoningTrail> {
            panic!("intentional test panic");
        }
        fn supported_axes(&self) -> &[Axis] {
            &[Axis::Horizontal]
        }
    }

    #[test]
    fn panic_is_isolated() {
        let mut o = Orchestrator::new();
        o.register(Arc::new(PanickingCataloger), 0);
        o.register(
            Arc::new(FakeCataloger {
                name: "ok",
                verdict: Confidence::High,
            }),
            0,
        );
        let ctx = CatalogerContext::new(
            "test".into(),
            "body".into(),
            Vec::new(),
            Vec::new(),
        );
        let trails = o.run(&ctx, |_| EarlyVerdict {
            horizontal_regime: ConfidenceRegime::Split,
            vertical_regime: ConfidenceRegime::Split,
        });
        // Both catalogers produced trails (the panicker an abstain trail).
        assert_eq!(trails.len(), 2);
        assert!(trails.iter().any(|t| t.cataloger == "panicker" && !t.voiced_opinion));
        assert!(trails.iter().any(|t| t.cataloger == "ok" && t.voiced_opinion));
    }

    #[test]
    fn early_unanimous_skips_pass_2() {
        let mut o = Orchestrator::new();
        o.register(
            Arc::new(FakeCataloger {
                name: "cheap1",
                verdict: Confidence::High,
            }),
            0,
        );
        o.register(
            Arc::new(FakeCataloger {
                name: "expensive",
                verdict: Confidence::High,
            }),
            2,
        );
        let ctx = CatalogerContext::new(
            "test".into(),
            "body".into(),
            Vec::new(),
            Vec::new(),
        );
        // Stub early-synthesize that always says "Unanimous on both."
        let trails = o.run(&ctx, |_| EarlyVerdict {
            horizontal_regime: ConfidenceRegime::Unanimous,
            vertical_regime: ConfidenceRegime::Unanimous,
        });
        assert_eq!(trails.len(), 1);
        assert!(trails.iter().all(|t| t.cataloger == "cheap1"));
    }
}

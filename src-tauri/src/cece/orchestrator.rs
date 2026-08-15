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
        // PJ-282 — ONE clone of the context per classification, shared by every cataloger's
        // detached worker (not one clone per cataloger). See `run_one_safe` for why the workers
        // need owned data at all.
        let shared = Arc::new(ctx.clone());

        // ── Pass 1: cheap catalogers (cost <= 1). ──
        for r in self.catalogers.iter().filter(|r| r.cost <= 1) {
            if let Some(t) = run_one_safe(Arc::clone(&r.cataloger), Arc::clone(&shared)) {
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
            if let Some(t) = run_one_safe(Arc::clone(&r.cataloger), Arc::clone(&shared)) {
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

/// Run a single cataloger with panic isolation AND timeout enforcement.
///
/// Per Architect §10 invariant 5: cataloger errors do NOT propagate.
/// Per Architect §10 invariant 12: ensemble timeouts are bounded.
///
/// V3-§8.r4.4 (audit P1.4): the original `run_one_safe` only caught
/// panics, not hangs. A pathological regex in Structural or a hung
/// embedding call in Semantic would block the IPC indefinitely.
/// Now: cataloger runs on a dedicated thread; channel `recv_timeout`
/// enforces the per-cataloger budget. Choosing thread-channel over
/// `tokio::time::timeout` to keep the orchestrator synchronous and
/// avoid an async refactor that would touch every cataloger.
///
/// Cost: one thread spawn per cataloger per IPC. Threads are cheap on
/// modern OS (~1ms / ~50KB stack); 6 catalogers per note × 7K notes
/// in a Library scan = 42K thread spawns over the scan = negligible
/// vs the actual classification work.
///
/// ## PJ-282 (tenth sweep) — the thread must be DETACHED, or the timeout is theatre
///
/// The fix above used `std::thread::scope`, and **`scope` joins every thread it spawned before
/// it returns**. So `recv_timeout` only chose which value came back: after it fired, the function
/// still blocked for `classify()`'s full true duration. The bound this function exists to provide
/// did not exist, `Architect §10 invariant 12` ("ensemble timeouts are bounded") was silently
/// false, and — worse than merely absent — the abstain trail told the user the cataloger had been
/// *"isolated by orchestrator"* when nothing had been isolated. The codebase's own measurements
/// show that mattering: `run_embedding` is ~32 s on a large note against Semantic's 2 s budget,
/// and `constellation_embed_notes` holds the engine mutex across an entire batch, so a
/// concurrent classification parks inside `classify()` for minutes.
///
/// Scoped threads were chosen because `&dyn Cataloger` cannot cross a thread boundary. That is no
/// longer a constraint: the registry already holds `Arc<dyn Cataloger>` (`Send + Sync`), and
/// `CatalogerContext` is owned data, so both can simply be MOVED into a detached thread.
///
/// The trade is deliberate and stated: an over-budget worker is orphaned rather than killed —
/// Rust cannot kill a thread — so it runs to completion and dies unobserved, its `send` failing
/// harmlessly into a dropped receiver. That is exactly what the old comment promised, and it costs
/// one orphaned thread in a case that is already pathological. The caller gets its budget back.
fn run_one_safe(c: Arc<dyn Cataloger>, ctx: Arc<CatalogerContext>) -> Option<ReasoningTrail> {
    let cataloger_name = c.name(); // &'static str — survives the move below
    let timeout = cataloger_timeout(cataloger_name);

    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();

    {
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.classify(&ctx)));
            // Send result; ignore if receiver already dropped (timeout fired).
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(trail_opt)) => trail_opt,
            Ok(Err(_)) => {
                eprintln!(
                    "[orchestrator] cataloger {} panicked — treating as silent",
                    cataloger_name
                );
                Some(ReasoningTrail::abstain(
                    cataloger_name,
                    "Cataloger panicked; isolated by orchestrator.",
                ))
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                eprintln!(
                    "[orchestrator] cataloger {} exceeded {:?} timeout — treating as silent",
                    cataloger_name, timeout
                );
                Some(ReasoningTrail::abstain(
                    cataloger_name,
                    &format!("Cataloger exceeded {:?} timeout; isolated by orchestrator.", timeout),
                ))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Worker thread vanished without sending — should not
                // happen under normal conditions; treat as abstention.
                eprintln!(
                    "[orchestrator] cataloger {} worker channel disconnected",
                    cataloger_name
                );
                Some(ReasoningTrail::abstain(
                    cataloger_name,
                    "Cataloger worker disconnected; isolated by orchestrator.",
                ))
            }
        }
    }
}

/// Per-cataloger timeout budget. Cheap catalogers get tight budgets;
/// the Reasoning Cataloger (LLM) gets the slowest tier.
fn cataloger_timeout(name: &str) -> std::time::Duration {
    use std::time::Duration;
    match name {
        // Cheap microsecond catalogers — anything over 100ms is a hang.
        "user_authority" | "structural" => Duration::from_millis(500),
        // Linguistic is normally cheap (regex + lexicon) but its CAE root
        // extraction and Bridge slow-path embedding lookup scale with note
        // length. On a 30K-character Arabic note the slow-path can
        // legitimately take 600–1500ms. V3-§8.r6 (post-Gate-1 Boss-test
        // 2026-05-10): moved from 500ms to 2s tier alongside Graph +
        // Semantic. The original 500ms grouping was a silent-abstain
        // regression — long Arabic notes had Linguistic dropped from the
        // ensemble entirely. Caught on the الخط العربي Boss-test card.
        "linguistic" => Duration::from_secs(2),
        // Medium DB-query catalogers — typical 30ms; allow 2s for cold cache.
        "graph" | "semantic" => Duration::from_secs(2),
        // Reasoning is LLM-bound — typical 1.5s; allow 5s for cold start.
        "reasoning" => Duration::from_secs(5),
        // Unknown / future catalogers — generous default.
        _ => Duration::from_secs(2),
    }
}

// V3-§8.r2.d note (audit Software Architecture #6): the original V3-§1
// scaffolded an `OrchestratorState` Tauri-managed wrapper here, but it
// was never wired — `classifier_suggest_for_note` constructs a fresh
// `Orchestrator` per IPC call. The audit flagged this as a half-
// finished migration. Rather than shipping a stale struct that
// implied a sharing pattern that doesn't exist, the dead state was
// deleted in r2.
//
// Why per-IPC construction stays: the Orchestrator's closures capture
// per-call state (the `MemoizedEmbed` cache that dedupes Linguistic +
// Semantic embed calls within a single note classification). Boot-
// time construction would need either a different caching strategy
// (per-app LRU with eviction) or a per-call context that catalogers
// read from. The latter is what the deleted `CatalogerContext`
// OnceLock fields tried to express; we deleted them in r2.a because
// no cataloger read them. Future work could re-introduce a per-call
// services struct + boot-time orchestrator; not a Day 1 ship blocker.
//
// Construction cost per IPC: ~6 `Arc::new` + ~6 closure boxes +
// 6-element `Vec::sort_by_key`. Microseconds. The real per-note cost
// is in the catalogers' classify() calls, not this construction.

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
                reasoning_template: None,
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
    fn linguistic_gets_medium_timeout_not_cheap() {
        // V3-§8.r6 regression — Linguistic was originally grouped with the
        // 500ms cheap-tier catalogers, but its CAE root extraction +
        // Bridge slow-path embedding lookup scale with note length. On a
        // long Arabic note the slow-path can take 600–1500ms, which
        // silently exceeded the 500ms budget and dropped Linguistic from
        // the trail set entirely. Caught on the الخط العربي Boss-test
        // card. Don't let it drift back.
        use std::time::Duration;
        assert_eq!(
            cataloger_timeout("linguistic"),
            Duration::from_secs(2),
            "Linguistic must be in the 2s tier — see r6 fix"
        );
        assert_eq!(
            cataloger_timeout("structural"),
            Duration::from_millis(500),
            "Structural stays in the 500ms cheap tier"
        );
        assert_eq!(
            cataloger_timeout("user_authority"),
            Duration::from_millis(500),
            "User-Authority stays in the 500ms cheap tier"
        );
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

/// PJ-282 (tenth sweep) — the per-cataloger timeout must be a REAL wall-clock bound.
#[cfg(test)]
mod tests_pj282_timeout_is_a_real_bound {
    use super::*;
    use crate::cece::cataloger::Axis;

    /// Named "structural" so `cataloger_timeout` gives it the 500 ms tier, then blocks for far
    /// longer than that. It never returns a trail: reaching the end would mean the timeout lost.
    struct SleepyCataloger;

    impl Cataloger for SleepyCataloger {
        fn name(&self) -> &'static str {
            "structural"
        }
        fn classify(&self, _ctx: &CatalogerContext) -> Option<ReasoningTrail> {
            std::thread::sleep(std::time::Duration::from_secs(8));
            None
        }
        fn supported_axes(&self) -> &[Axis] {
            &[Axis::Horizontal, Axis::Vertical]
        }
    }

    /// **RED before the fix.** `std::thread::scope` joins its threads before returning, so the
    /// old `run_one_safe` blocked the full 8 s here while reporting — in the trail it handed
    /// back — that the cataloger had been "isolated by orchestrator". A guard that reports
    /// success at isolating something it did not isolate is worse than no guard: the P1.4 hang
    /// class stayed open while the code and the Architect invariant both said it was closed.
    #[test]
    fn an_over_budget_cataloger_returns_control_at_its_budget() {
        let ctx = Arc::new(CatalogerContext::new(
            "/probe.md".to_string(),
            "body".to_string(),
            Vec::new(),
            Vec::new(),
        ));
        let t0 = std::time::Instant::now();
        let trail = run_one_safe(Arc::new(SleepyCataloger), ctx);
        let waited = t0.elapsed();

        assert!(
            waited < std::time::Duration::from_secs(4),
            "run_one_safe blocked {waited:?} on a cataloger with a 500 ms budget — the timeout \
             does not bound wall-clock time"
        );
        let trail = trail.expect("a timed-out cataloger abstains rather than vanishing");
        assert!(!trail.voiced_opinion, "an abstention must not count as an opinion");
        assert!(
            trail.reasoning.contains("timeout"),
            "and the trail must say WHY it abstained: {}",
            trail.reasoning
        );
    }
}

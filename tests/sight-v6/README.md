# Sight v6 — Test Harness (MIG-025 §A.13)

This directory contains the **test source files** for Sight v6's continuous
performance and layout-fidelity gates per Concept Paper v4.0 §8.3 + §11
invariants 2 and 3.

## Status

The test files are **harness-ready** — written against the public APIs
in `src/lib/sight/v6/` so they exercise the production code paths
exactly. The actual CI runners (vitest for `perf.test.ts`, playwright
for `layout-fidelity.test.ts`) are **deferred to a focused follow-up**
because adding either as a new devDependency is a project-level decision
beyond MIG-025's scope (the project has no JS test framework today;
choosing one — vitest vs jest vs vest, playwright vs cypress — needs
its own discussion).

The Rust backend tests already exercise the cache schema, the
synchronous and progressive backfills, and the trigger invalidation
paths via `cargo test --lib sight_v6` (25 tests passing as of §A.4).

## Files

| File | What it asserts | Runner |
|---|---|---|
| `perf.test.ts` | computeStarPositions on a 7,636-note synthetic fixture completes in ≤16 ms (Concept Paper §11 invariant 3 cross-filter budget for §A.10). Render-budget tests for §A.8/§A.9 paths. | vitest (deferred) |
| `layout-fidelity.test.ts` | The anchor dome occupies ≥80% of the canvas-host width × height in default state (Concept Paper §11 invariant 2). | playwright (deferred) |

## When the runners land

The §D.4 ship gate (Phase 4 — channel orthogonality + CIE Delta-E
automated tests) is the natural moment to add vitest and playwright as
project-level devDependencies. Until then:

- The test files compile under svelte-check (zero new errors).
- The pure-function paths (`computeStarPositions`, `computeDomeLayout`,
  filter logic) can be benchmarked with a lightweight Node ESM script
  if needed mid-phase.
- The Rust tests cover backend correctness.

## Verification gate (Phase 1 ship — §A.14)

Eisa's Boss-test of Sight v6.0 verifies the runtime behaviors
manually:
- "When I open Sight v6 with `SIGHT_V6_ENABLED=true`, the dome
  fills the canvas without obvious chrome dominance" (§11 inv 2).
- "Cross-filter via the sidebar feels instant" (§11 inv 3 by user
  perception; quantitative gate lands in §D.4).
- "Tour fires once on first open, doesn't re-fire on second open"
  (§11 inv 10).

The §D.4 CI gates make these guarantees automated rather than
relying on Boss-test attention.

/**
 * Sight engine flags — MIG-017 (v2 disable), MIG-018 (v3 build), MIG-019 (v4 pivot), MIG-024 (v5 build).
 *
 * v2 Sight (`ConstellationSight2.svelte` + `sight.rs` + `constellation_sight_*` IPCs)
 * is preserved on disk as a known-good fallback. Flip `SIGHT_V2_ENABLED` to
 * `true` and rebuild to bring v2 back for diagnostics.
 *
 * v3 Sight (`SightV3.svelte`) — RETIRED. 13 close-button iterations failed
 * because v3 mounted as a `position: fixed` overlay outside normal DOM flow.
 * D3-zoom's event listeners on the viewport-filling canvas consumed pointer
 * events before any button could receive them. Kept on disk for reference.
 *
 * v4 Sight (`SightV4.svelte`) — clean-slate rebuild using SkyView's proven
 * mount pattern: flex child inside `.content-area`, close button in
 * +layout.svelte's header row. Same Canvas 2D + D3-zoom render pipeline,
 * all v3 helper modules reused (modes.ts, polar.ts, regions.ts, etc.).
 *
 * v5 Sight (`SightV5.svelte`) — the canonical Sight target per Concept Paper
 * v3.1 (2026-05-12). Four-layer analytical instrument: Layer 1 visual
 * foundation (MIG-024) → Layer 2 diagnostic (MIG-025) → Layer 3
 * recommendation (MIG-026) → Layer 4 coaching (MIG-027). Eight strata
 * bands (constant radius), 7 mode toggles (R/L/T/C/S/A/P), 3-button scope
 * toggle (U/L/F per D-V3), four constants (radial/size/brightness/color)
 * that never change with mode. All inference local via CECE's e5-small +
 * Qwen3-1.7B + llama.cpp stack. Visual contract: docs/Sight-vNext-MockB1-Toggle.svg.
 *
 * The flags are mutually exclusive in production (only one engine renders
 * at a time); the dual flags exist so a developer can A/B them in a custom
 * build, and so the cutover is a single-edit operation. Per D-V5 (Eisa,
 * 2026-05-12), V4 flips to `false` in the same edit V5 flips to `true`
 * (the v5 ship moment, MIG-024 §6).
 */
// MIG-024 §6 ship moment (2026-05-12): V4 hidden, V5 active per D-V5.
// v4 component stays on disk for cleanup MIG (after Eisa confirms v5
// stable across multiple sessions). Flipping V4 back to true + V5 to
// false brings v4 back as the rollback target.
//
// MIG-025 §A.1 (2026-05-14): SIGHT_V6_ENABLED added per Concept Paper v4.0
// (ratified 2026-05-13). v6 specifies the next implementation, replacing
// v5's seven-mode toggle architecture with Coordinated Views (anchor dome
// + facet sidebar at Phase 1; 4 mini-domes Phase 2; 7-register chip Phase
// 3). Phased build per MIG-025 Plan §A→§D over ~21 wk; v5 stays mounted
// via dual-flag (B2) until §D.6 deletes it.
//
// MIG-025 §A.14 SHIP MOMENT (2026-05-14): Sight v6.0 ships after a
// 16-fix cycle through 7 NSIS Boss-test builds. Eisa accepted cycle-3.7
// ("Ship"). Phase 1 deliverable:
//   • Anchor dome with stratum × time × all-circle nodes; default-zoom
//     density-gradient view; wheel-zoom up to 24× for "crystal-clear"
//     individual node inspection.
//   • Hearst Flamenco facet sidebar (6 facets, Folder TOP, live counts).
//   • First-boot 4-step orientation tour.
//   • B2 dual-mount: v5 still reachable via its dock button.
//   • v5→v6 settings migration (lastMode dropped, lastScope preserved
//     as dead key, v6MigrationDone sentinel stamped).
//
// 16 fixes across the cycle:
//   1-4: chrome contrast, jitter widening, smaller stars, hover-title
//   5  : applyParsedSettings shared helper (fixed boot-bundle drift —
//        loadSettings() had zero callers, the migration was dead code)
//   6-9: brighter chrome, additive density blending, two-pass render,
//        wheel-zoom + drag-pan + Cmd-0 reset
//   10 : zoom regression — clear+bg in identity transform
//   11 : addEventListener wheel binding (Svelte template binding silent-
//        failed in cycle-3 multi-edit batch — Working Agreement #4 lesson)
//   12 : phyllotaxis spiral packing (later reverted via A/B test)
//   13 : node sizing 5 px ⌀ at max zoom — converts default view to true
//        density chart; zoom reveals individuals
//   14 : revert phyllotaxis (jitter wins A/B per Eisa)
//   15 : all-circle nodes (drop library-shape encoding at small sizes)
//   16 : hover-ring screen-padded + ZOOM_MAX 8→24× for crystal-clear
//
// Phase 2 (§B) opens next: 4 mini-domes + cross-filter brushing + Pro
// mode toggle. v4.1 polish targets allocated separately (hex-bin
// aggregation for 50k+-note universes, library-tint recognition aid,
// etc.).
//
// References:
//   docs/Constellation-Sight-Concept-Paper-v4.0.md
//   docs/Constellation Orientation & Onboarding v2.02.md (this commit)
//   lab/reports/MIG-025-SIGHT-V6-ARCHITECT.md
//   lab/reports/MIG-025-SIGHT-V6-PLAN.md
//   lab/reports/SESSION-LOG-2026-05-14.md
export const SIGHT_V2_ENABLED = false;
export const SIGHT_V3_ENABLED = false;
export const SIGHT_V4_ENABLED = false;
export const SIGHT_V5_ENABLED = true;
export const SIGHT_V6_ENABLED = true;

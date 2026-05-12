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
export const SIGHT_V2_ENABLED = false;
export const SIGHT_V3_ENABLED = false;
export const SIGHT_V4_ENABLED = false;
export const SIGHT_V5_ENABLED = true;

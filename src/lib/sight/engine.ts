/**
 * Sight engine flags — MIG-017 / PJ-039.
 *
 * v2 Sight (`ConstellationSight2.svelte` + `lens.rs` + `constellation_sight_*` IPCs)
 * is preserved on disk as a known-good fallback. To revive v2 for diagnostics or
 * because v3 failed to ship, flip `SIGHT_V2_ENABLED` to `true`, rebuild, ship.
 *
 * Future v3 (PJ-038) will add `SIGHT_V3_ENABLED` here. The two are mutually
 * exclusive in production (only one engine renders at a time); the dual flags
 * exist so a developer can A/B them in a custom build.
 *
 * See: lab/reports/MIG-017-DISABLE-V2-SIGHT-ARCHITECT.md §4.
 */
export const SIGHT_V2_ENABLED = false;

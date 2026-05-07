/**
 * Sight engine flags — MIG-017 (v2 disable) and MIG-018 (v3 build).
 *
 * v2 Sight (`ConstellationSight2.svelte` + `sight.rs` + `constellation_sight_*` IPCs)
 * is preserved on disk as a known-good fallback. Flip `SIGHT_V2_ENABLED` to
 * `true` and rebuild to bring v2 back for diagnostics.
 *
 * v3 Sight (`SightV3.svelte` + `sight_layout.rs` + `constellation_sight_v3_*` IPCs)
 * is the new star-chart engine being built under PJ-038. Stays at `false` while
 * MIG-018 §1A–§1E land; flips to `true` in §1F after Boss-test passes.
 *
 * The two flags are mutually exclusive in production (only one engine renders
 * at a time); the dual flags exist so a developer can A/B them in a custom
 * build, and so the cutover from v2 to v3 is a single-edit operation.
 *
 * See: lab/reports/MIG-017-DISABLE-V2-SIGHT-ARCHITECT.md §4 (v2 disable
 * mechanism), lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-ARCHITECT.md §4
 * (v3 ship-behind-flag pattern).
 */
export const SIGHT_V2_ENABLED = false;
export const SIGHT_V3_ENABLED = true;

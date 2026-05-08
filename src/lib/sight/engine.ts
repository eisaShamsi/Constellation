/**
 * Sight engine flags — MIG-017 (v2 disable), MIG-018 (v3 build), MIG-019 (v4 pivot).
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
 * The flags are mutually exclusive in production (only one engine renders
 * at a time); the dual flags exist so a developer can A/B them in a custom
 * build, and so the cutover is a single-edit operation.
 */
export const SIGHT_V2_ENABLED = false;
export const SIGHT_V3_ENABLED = false;
export const SIGHT_V4_ENABLED = true;

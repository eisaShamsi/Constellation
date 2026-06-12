/**
 * MIG-076 §C — the single-ownership integration toggle.
 *
 * When `true`, the editor surfaces SEED from and SAVE through `noteModel`
 * (single content ownership). When `false`, the legacy `tab.content` path runs
 * unchanged. The model is MAINTAINED either way (cheap, harmless) — so flipping
 * this is an instant, clean rollback with NO half-state: set to `false` and
 * rebuild to return the app to the proven §C-1 behavior, without git surgery.
 *
 * Default `true` for the Boss test. The toggle exists precisely because the
 * integration wiring is the one thing the headless harness cannot cover
 * (CLAUDE.md Editor-Surface Gate) — so it must be reversible in one keystroke.
 */
export const SINGLE_OWNERSHIP = true;

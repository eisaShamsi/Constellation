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

/**
 * MIG-107 — single ownership for PROPERTIES, the second half of the same idea.
 *
 * `SINGLE_OWNERSHIP` above made the model the authority for a note's BODY and for what gets
 * written to disk. Properties were left behind: both Properties panels still seed from
 * `tab.content` — a projection of the file as it looked when the note was opened — and each save
 * replaces the model's whole array from that drifted copy. Whoever saves second silently deletes
 * the other's frontmatter (PJ-174 AK-2/AK-3, reproduced in `tests/pj-174/propsOwnership.test.ts`).
 *
 * When `true`, the panels READ from the model (Slice 3) and WRITE field-level intents (Slice 4).
 * When `false`, the legacy projection path runs unchanged — the model is maintained either way, so
 * flipping this back is an instant rollback with no half-state, exactly like the flag above.
 *
 * Slice 3 turns on the READ half only. The write half is still a whole-array replace, so AK-2/AK-3
 * are NOT yet fixed by this flag alone — that is deliberate, so the read half can be validated on
 * its own and the swap can be reverted without losing it.
 */
export const PROPS_SINGLE_OWNERSHIP = true;

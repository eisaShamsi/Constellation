/**
 * PJ-106 §A1 (2026-07-14) — the RTL-motion rollback lever.
 *
 * When true, the editor enables CodeMirror's `EditorView.perLineTextDirection` so the
 * caret/selection MOTION engine reads each line's own direction (the bidiPlugin already
 * renders per-line `dir` DOM attributes; this connects those to Home/End/arrows/word-hops),
 * and the editable surfaces set a DETERMINISTIC base direction instead of the viewport-
 * first-strong `dir='auto'` that competed with it.
 *
 * One const → instant rollback. EXACT SCOPE (audit-pinned, 2026-07-16): false strips the
 * MOTION facet (perLineTextDirection), the logical-arrow keymap, the B1/B2/B3 selection
 * keymaps + Ctrl+click sentence gesture, and the §B4 direction-switch gesture. It does NOT
 * revert: the deterministic base direction (dirCompartment — not flag-gated), §B0 triple-click
 * text-only, the bidiPlugin's rendering behaviors (statically registered, per SI4-03), or any
 * §B4 RLM/LRM marks already persisted in notes — those are plain-text bytes and stay
 * render-honored under flag-off (only the gesture to add/change them goes away). Same
 * rollback-lever pattern as `ownershipFlag`/`cockpitFlag`. Retired at the migration close.
 */
export const RTL_MOTION_ENABLED = true;

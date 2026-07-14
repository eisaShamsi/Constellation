/**
 * PJ-106 §A1 (2026-07-14) — the RTL-motion rollback lever.
 *
 * When true, the editor enables CodeMirror's `EditorView.perLineTextDirection` so the
 * caret/selection MOTION engine reads each line's own direction (the bidiPlugin already
 * renders per-line `dir` DOM attributes; this connects those to Home/End/arrows/word-hops),
 * and the editable surfaces set a DETERMINISTIC base direction instead of the viewport-
 * first-strong `dir='auto'` that competed with it.
 *
 * One const → instant rollback: false reverts to the pre-A1 behavior (facet off, the
 * bidiPlugin's rendering untouched — it stays statically registered, per amendment SI4-03,
 * so this flag governs ONLY motion, never the DOM-attribute rendering). Same rollback-lever
 * pattern as `ownershipFlag`/`cockpitFlag`. Retired at the migration close.
 */
export const RTL_MOTION_ENABLED = true;

/**
 * PJ-114 §1.1 (2026-07-17) — the FM+ (Focus Mode Plus) build-time rollback lever.
 *
 * FM+ is the opt-in layer that lets Focus mode surface a thin right-click menu + follow
 * `[[links]]` in-place, WITHOUT ever leaving the parser-free plain-text capture surface.
 * It delegates every action to existing engines (rename cascade, link resolver, note-open
 * path, shared context menu) — zero new domain logic; Focus stays byte-for-byte itself
 * whenever FM+ is not active.
 *
 * TWO layers gate the feature:
 *   - THIS build-time const (`FM_PLUS_ENABLED`) — the hard kill-switch. `false` tree-shakes /
 *     inerts ALL FM+ code (the footer toggle, the contextmenu handler, the Mod-click/Mod-Enter
 *     follow gesture, the menu, the in-Focus navigation), so Focus reverts to exactly today's
 *     behavior regardless of the user setting.
 *   - the user setting `appSettings.focusModePlus` (persisted, default false) — the per-user
 *     opt-in.
 * Effective = `FM_PLUS_ENABLED && $appSettings.focusModePlus` (the `fmPlusEffective` derived
 * in +layout). Same rollback-lever pattern as `ownershipFlag`/`rtlFlag`/`cockpitFlag`.
 * Retired (folded to always-on) at the migration close once the whole feature is Boss-validated.
 */
export const FM_PLUS_ENABLED = true;

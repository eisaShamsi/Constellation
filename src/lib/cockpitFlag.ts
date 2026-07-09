/**
 * PJ-068 v2 — Knowledge Cockpit rollout flag.
 *
 * When true, the second screen renders the three-zone cockpit (Estimation Map /
 * Control Dashboard / Operation Map) for a focused note instead of the legacy
 * tabbed editor-panels companion. The old path stays in code behind `!COCKPIT_ENABLED`
 * for one-line rollback until the migration closes (P5). Frontend-only, no schema.
 */
export const COCKPIT_ENABLED = true;

export type DialMode = 'normal' | 'live' | 'locked';

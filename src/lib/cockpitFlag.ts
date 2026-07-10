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

/** The note-graph "lens" the second screen renders: switchable visual styles over the same
 *  live link data, chosen in Settings, coloured via the Style Setter. The Aster (relationship
 *  rose) was retired 2026-07-10 after art-direction review — its shared polar origin could not
 *  break the circle or part the wings. Replaced by two lenses on one chassis (twin origins ·
 *  vertical gutter · length-encoding · gauge deck): The Butterfly (facing blooms) + The Ledger
 *  (balance sheet). Heartwood + Orrery remain planned (baseline radial fallback until built). */
export type NoteGraphStyle = 'butterfly' | 'ledger' | 'heartwood' | 'orrery';
export const NOTE_GRAPH_STYLES: { id: NoteGraphStyle; labelKey: string; label: string; built: boolean }[] = [
	{ id: 'butterfly', labelKey: 'cockpit.lens.butterfly', label: 'The Butterfly (facing blooms)', built: true },
	{ id: 'ledger', labelKey: 'cockpit.lens.ledger', label: 'The Ledger (balance sheet)', built: true },
	{ id: 'heartwood', labelKey: 'cockpit.lens.heartwood', label: 'Heartwood (living tree)', built: false },
	{ id: 'orrery', labelKey: 'cockpit.lens.orrery', label: 'The Orrery (orbital sky)', built: false },
];

/** Normalize a stored/loaded lens value: retired ('aster') or unknown values fall back to the
 *  default so the second screen never renders a dead lens. Read-time migration — no schema. */
export function normalizeGraphStyle(v: unknown): NoteGraphStyle {
	return NOTE_GRAPH_STYLES.some((s) => s.id === v) ? (v as NoteGraphStyle) : 'butterfly';
}

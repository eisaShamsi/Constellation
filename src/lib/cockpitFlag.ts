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

/** The note-graph "lens" the second screen renders (Boss ruling 2026-07-09): three
 *  switchable visual styles over the same live link data, chosen in Settings, coloured
 *  via the Style Setter. Aster (the flagship rose) ships first; Heartwood + Orrery follow. */
export type NoteGraphStyle = 'aster' | 'heartwood' | 'orrery';
export const NOTE_GRAPH_STYLES: { id: NoteGraphStyle; label: string; built: boolean }[] = [
	{ id: 'aster', label: 'Aster (relationship rose)', built: true },
	{ id: 'heartwood', label: 'Heartwood (living tree)', built: false },
	{ id: 'orrery', label: 'Orrery (orbital sky)', built: false },
];

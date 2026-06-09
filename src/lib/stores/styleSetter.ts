import { writable } from 'svelte/store';

/**
 * MIG-070 — the standalone Constellation Style Setter (CSS).
 *
 * A single boolean store driving whether the full-page Style Setter is open. Kept
 * deliberately tiny + independent of all existing style code (the MIG-069 Style Presets
 * engine is NOT touched): the Setter is its own world. Any entry point (a Settings button,
 * a command, a dock icon) just flips this; the Setter mounts once at the top level and
 * shows itself when true.
 */
export const styleSetterOpen = writable(false);

/** Open / close helpers (so call sites don't import `writable` semantics). */
export function openStyleSetter() { styleSetterOpen.set(true); }
export function closeStyleSetter() { styleSetterOpen.set(false); }

/** MIG-070 §C item D — request the Setter to OPEN straight into inspect mode (the dock shortcut:
 *  inspect & restyle without going through Settings). The Setter watches this, calls its
 *  startInspect on open, then resets the flag. */
export const styleSetterInspectRequest = writable(false);
export function openStyleSetterInspect() { styleSetterInspectRequest.set(true); styleSetterOpen.set(true); }

/** MIG-007 — open the Setter straight to a given category (e.g. 'links' for the Link-Type editor),
 *  used by the Links Settings tab hub. The Setter watches this on open, navigates there, then clears it. */
export const styleSetterCategoryRequest = writable<string | null>(null);
export function openStyleSetterToCategory(cat: string) { styleSetterCategoryRequest.set(cat); styleSetterOpen.set(true); }

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

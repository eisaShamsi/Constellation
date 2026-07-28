/**
 * MIG-107 Slice 2 — the READ BRIDGE for note properties.
 *
 * `noteModel` is **deliberately non-reactive** — a plain module `Map`. Its header states why, and
 * the reason is a scar: *"a store update inside a `{#key}` teardown re-enters the render the store
 * drives"* (the §C-2 lesson). So panels cannot subscribe to the model, and the model must not be
 * made to announce its own changes.
 *
 * This module is the seam. It holds one counter that ticks when some note's properties change, so a
 * panel can render **from the model** — the single authority — while still being told when to look
 * again. The model stays inert; the decision about which call sites announce stays here and in the
 * `noteSession` wrappers, where it is explicit and reviewable, rather than buried in the authority.
 *
 * ## Rules, each load-bearing
 *
 * 1. **Never key a `{#key}` block on this.** It exists to invalidate a `$derived`, nothing more.
 *    Keying a block on it would recreate the §C-2 hazard exactly: a props write during a teardown
 *    would re-enter the render that teardown belongs to. `reloadVersion` remains the ONLY remount
 *    signal.
 * 2. **It is not on the keystroke path.** Body typing does not touch properties, so this stays
 *    silent while the user writes. Property edits are debounced and rare by comparison.
 * 3. **One global counter, not a store per note.** A tick makes every mounted panel re-read *its own*
 *    model; a panel whose note did not change reads back the identical `props` array **reference**
 *    (the model only re-assigns `props` when something actually changed), so Svelte's `$derived`
 *    sees no change and nothing downstream re-renders. Cheaper than a map of stores, and it cannot
 *    leak an entry when a note closes.
 * 4. **Carries no payload.** Deliberately: a payload would be a second copy of the truth, which is
 *    the entire defect this migration exists to remove. Subscribers are told *"look again"* and must
 *    then read the model.
 */
import { writable } from 'svelte/store';

/**
 * Ticks whenever any open note's properties change. Subscribe to invalidate a `$derived` that reads
 * `getModel(id)?.props`; never read a meaning into the number itself.
 */
export const propsVersion = writable(0);

/** Announce that some note's properties changed. Called by the `noteSession` intent wrappers. */
export function bumpProps(): void {
	propsVersion.update((n) => n + 1);
}

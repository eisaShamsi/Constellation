import { writable } from 'svelte/store';

/**
 * PJ-088 — the conflict-resolution side-by-side MERGE view.
 *
 * A single store holding the merge TARGET (which note + which `.conflict` side-copy) that drives
 * the full-center-zone overlay. Deliberately tiny + independent (cloned from styleSetter.ts): the
 * overlay mounts once at the top level and shows itself when the target is non-null. Any entry
 * point (today the conflict banner's "Merge…" button; later a file-tree menu) just sets the target;
 * the heavy @codemirror/merge view is lazy-imported inside the overlay so it never touches the main
 * bundle / boot / keystroke hot path (Rules 3 + 6).
 */
export interface MergeTarget {
	/** The real note being reconciled (its live model is the left/editable pane + the save target). */
	notePath: string;
	/** The `.conflict-<UTCz>.md.txt` side-copy holding the incoming outside version (right/read-only pane). */
	sidecarPath: string;
	/** Display name for the overlay title. */
	noteName: string;
}

export const mergeViewTarget = writable<MergeTarget | null>(null);

export function openMergeView(target: MergeTarget) { mergeViewTarget.set(target); }
export function closeMergeView() { mergeViewTarget.set(null); }

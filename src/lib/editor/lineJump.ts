/**
 * MIG-082 §A.2 — one-shot "jump to this 1-indexed line on the tab's next mount".
 *
 * Keyed by tab id. `openNoteTab(..., targetLine)` sets it; NotePane's mount block
 * TAKES it (read-once + delete) and dispatches a CM6 selection + scrollIntoView.
 * One-shot so a later remount (tab switch, rename-cascade reloadVersion) never
 * re-jumps. Deliberately CM6-free so the core store can import it without pulling
 * the editor bundle — the actual dispatch lives in NotePane / activeEditor.
 */
const pending = new Map<string, number>();

/** Schedule a 1-indexed line jump for `tabId`'s next mount (no-op for line <= 0). */
export function setPendingLineJump(tabId: string, line: number): void {
	if (tabId && line > 0) pending.set(tabId, line);
}

/** Read-and-clear the pending jump for `tabId` (undefined if none). */
export function takePendingLineJump(tabId: string): number | undefined {
	const v = pending.get(tabId);
	if (v !== undefined) pending.delete(tabId);
	return v;
}

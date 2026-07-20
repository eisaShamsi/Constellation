/**
 * Active editor registry.
 *
 * Any CM6 EditorView that wants to receive global commands (e.g. the
 * emoji/icon picker's insert-at-cursor) registers itself here on focus.
 * The picker doesn't need to guess which pane the user meant — it reads
 * the last registered view directly.
 */
import { EditorView } from '@codemirror/view';

let lastView: EditorView | null = null;
let lastPath: string | null = null; // §A.2 — the note path the registered view belongs to

export function registerActiveEditor(view: EditorView, path?: string) {
	if (path === undefined && view !== lastView) {
		// A DIFFERENT view registering with an unknown path must not inherit the previous
		// note's path — the path-guarded getter would then hand this view out for a note it
		// does not belong to (latent aliasing; no caller does this today, but the guard must
		// hold by construction). A same-view re-register without a path (focusin after mount)
		// keeps its known path, as before.
		lastPath = null;
	}
	lastView = view;
	if (path !== undefined) lastPath = path;
}

export function unregisterActiveEditor(view: EditorView) {
	if (lastView === view) { lastView = null; lastPath = null; }
}

export function getActiveEditor(): EditorView | null {
	return lastView;
}

/**
 * The registered view ONLY if it belongs to `path` — the same guard `goToLineIfActive`
 * uses, exposed for callers that need the view itself (template insert-at-cursor).
 *
 * The guard is not optional for any caller with an `await` between choosing the target
 * note and dispatching into it: template processing can open a `{{prompt:…}}` dialog,
 * and if the user switches tabs while it is up, an unguarded dispatch lands the insert
 * in the WRONG note — the cross-note content class (BUG-023/§C) this registry's
 * path-awareness exists to prevent.
 */
export function getActiveEditorForPath(path: string): EditorView | null {
	return lastView && lastPath === path ? lastView : null;
}

/**
 * MIG-082 §A.2 — jump the registered editor's cursor to a 1-indexed `line` and scroll it
 * into view, but ONLY if the registered view belongs to `path` (so we never dispatch into the
 * wrong note). Selection-only: NO document change, NO save (Editor-Surface Gate #2). Used for
 * the "the task's note is already the active tab" case, where no remount occurs. Returns true
 * if it jumped.
 */
export function goToLineIfActive(path: string, line: number): boolean {
	if (!lastView || lastPath !== path || line <= 0) return false;
	const doc = lastView.state.doc;
	const n = Math.min(Math.max(1, Math.floor(line)), doc.lines);
	const pos = doc.line(n).from;
	lastView.dispatch({ selection: { anchor: pos }, effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
	lastView.focus();
	return true;
}

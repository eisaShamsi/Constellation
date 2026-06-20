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

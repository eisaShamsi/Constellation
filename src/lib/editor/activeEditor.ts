/**
 * Active editor registry.
 *
 * Any CM6 EditorView that wants to receive global commands (e.g. the
 * emoji/icon picker's insert-at-cursor) registers itself here on focus.
 * The picker doesn't need to guess which pane the user meant — it reads
 * the last registered view directly.
 */
import type { EditorView } from '@codemirror/view';

let lastView: EditorView | null = null;

export function registerActiveEditor(view: EditorView) {
	lastView = view;
}

export function unregisterActiveEditor(view: EditorView) {
	if (lastView === view) lastView = null;
}

export function getActiveEditor(): EditorView | null {
	return lastView;
}

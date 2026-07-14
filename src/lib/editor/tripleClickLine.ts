/**
 * PJ-106 §B0 — triple-click selects the line's TEXT, not the trailing newline.
 *
 * CodeMirror's default triple-click (rangeForClick, @codemirror/view) selects
 * `[line.from, line.to + 1]` — it INCLUDES the trailing newline. drawSelection then
 * paints the selection across the empty remainder of the visual line: on an RTL line
 * whose text is right-aligned, that empty run is to the LEFT of the text, so the
 * highlight stretches past the words (the Boss's 2026-07-14 remark). This overrides
 * ONLY the triple-click gesture to select `[line.from, line.to]` — the text, nothing
 * more. Every other gesture (single/double click, drag) falls through to the default.
 *
 * Shared across every editable note surface (Editor Parity): NotePane, FocusPane, and
 * the conflict-merge panes.
 */
import { EditorView } from '@codemirror/view';
import { EditorSelection } from '@codemirror/state';

export const tripleClickTextOnly = EditorView.mouseSelectionStyle.of((view, event) => {
	if (event.detail !== 3) return null; // only triple-click; all other gestures = CM6 default

	const startPos = view.posAtCoords({ x: event.clientX, y: event.clientY });
	if (startPos == null) return null;
	const startLine = view.state.doc.lineAt(startPos);

	// Whole-line-TEXT range from the start line to the line under `headPos` (supports
	// triple-click-drag across lines, in either direction). Never includes the newline.
	const rangeForLines = (headPos: number) => {
		const headLine = view.state.doc.lineAt(headPos);
		const forward = headLine.from >= startLine.from;
		const anchor = forward ? startLine.from : startLine.to;
		const head = forward ? headLine.to : headLine.from;
		return EditorSelection.range(anchor, head);
	};

	return {
		get: (curEvent, extend, multiple) => {
			const curPos = view.posAtCoords({ x: curEvent.clientX, y: curEvent.clientY }) ?? startPos;
			const range = rangeForLines(curPos);
			const sel = view.state.selection;
			if (extend) return sel.replaceRange(sel.main.extend(range.anchor, range.head));
			if (multiple) return sel.addRange(range);
			return EditorSelection.create([range], 0);
		},
		update: () => false, // positions are recomputed from live coords in get(); nothing to map
	};
});

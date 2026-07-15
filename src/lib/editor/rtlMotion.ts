/**
 * PJ-106 §A5 — LOGICAL (Word/Windows) arrow motion for bidirectional text.
 *
 * THE CONCEPT (the horse): an arrow key should move the caret by ONE CHARACTER OF THE TEXT,
 * in the order the text is actually written — never by one position on the screen. In mixed
 * Arabic/Latin writing those two orders diverge, and CodeMirror's default follows the SCREEN
 * (visual motion): at an Arabic↔Latin boundary the caret appears to stall, double back, or
 * "lose its direction" (the Boss's symptom ③). Microsoft Word on Windows — the Boss's daily
 * muscle memory — follows the TEXT (logical motion): every press advances exactly one
 * character of the document, and at a boundary the caret visibly hops to where that next
 * character actually lives. Boss ruling 2026-07-14: "Logical — like Microsoft Word."
 *
 * WHICH ARROW MEANS "FORWARD" is resolved from the base direction at the caret (the same
 * rule CM6's own cursorCharLeft/Right use, via textDirectionAt — which, thanks to §A1's
 * perLineTextDirection, is now the PER-LINE base):
 *   - LTR line: → = forward (next char), ← = backward.
 *   - RTL line: ← = forward (next char), → = backward.
 * So in PURE Arabic or PURE English the caret still moves the way the arrow points — nothing
 * changes. The ONLY behavioral difference from the old default is AT A BIDI BOUNDARY, where
 * the caret now steps through the text in writing order instead of hopping around the screen.
 *
 * ATOMIC WIDGET SKIP (design-inspection H3): logical motion steps RAW string offsets, and —
 * unlike CM6's visual motion — does not skip replaced/collapsed widgets. NotePane collapses
 * a ```lens``` block into a card (an always-on replace decoration), so plain logical arrows
 * would walk the caret INTO the widget's hidden source, where it is invisible and takes one
 * press per hidden character to escape — the exact "stuck caret" this migration exists to
 * kill, and it would regress ENGLISH notes too. The fix is deliberately SCOPED: the skip
 * lives in these commands, NOT in the global `EditorView.atomicRanges` facet, because that
 * facet also feeds the DELETION path (`skipAtomic` in @codemirror/commands) and would
 * silently change how Backspace treats a lens block — a behavior change outside this
 * migration's scope and untested by its gate.
 *
 * The skip source is INJECTED rather than imported, so FocusPane can use logical arrows
 * without pulling `livePreview` (and the markdown parser with it) into its bundle — Rule 6,
 * "no heavy imports in FocusPane". FocusPane has no widgets, so it passes nothing.
 */
import { EditorView, Direction, keymap, type Command } from '@codemirror/view';
import {
	EditorSelection,
	Prec,
	findClusterBreak,
	type EditorState,
	type Extension,
	type RangeSet,
	type RangeValue,
} from '@codemirror/state';

/** One character forward/backward in LOGICAL (raw string index) order — the Word semantics.
 *  Mirrors @codemirror/commands' internal `byCharLogical`: grapheme-cluster aware inside a
 *  line, and steps across the line break at a line edge. Exported for headless tests (this
 *  is pure offset arithmetic — the one part of RTL motion that CAN be tested without layout). */
export function logicalCharPos(state: EditorState, pos: number, forward: boolean): number {
	const line = state.doc.lineAt(pos);
	if (pos === (forward ? line.to : line.from)) {
		// At a line edge — step over the newline into the neighbouring line.
		return forward ? Math.min(state.doc.length, line.to + 1) : Math.max(0, line.from - 1);
	}
	return line.from + findClusterBreak(line.text, pos - line.from, forward);
}

/** If `pos` landed INSIDE a collapsed widget's hidden source, jump clear of it (see the
 *  ATOMIC WIDGET SKIP note above). Touching an edge is fine — only strictly-inside traps. */
function skipCollapsed(
	state: EditorState,
	pos: number,
	forward: boolean,
	skipRanges?: (s: EditorState) => RangeSet<RangeValue> | null | undefined,
): number {
	const set = skipRanges?.(state);
	if (!set) return pos;
	let out = pos;
	set.between(pos, pos, (from, to) => {
		if (pos > from && pos < to) {
			out = forward ? to : from;
			return false; // stop iterating — a position can only be inside one replaced range
		}
		return undefined;
	});
	return out;
}

/** Move (or extend the selection by) one logical character. */
function moveLogical(
	view: EditorView,
	forward: boolean,
	select: boolean,
	skipRanges?: (s: EditorState) => RangeSet<RangeValue> | null | undefined,
): boolean {
	const state = view.state;
	const sel = state.selection;
	const ranges = sel.ranges.map((range) => {
		// A plain arrow on a NON-empty selection collapses it to its logical edge — the same
		// thing CM6's own char commands do (`rangeEnd`), so this stays familiar.
		if (!select && !range.empty) {
			return EditorSelection.cursor(forward ? range.to : range.from);
		}
		const moved = skipCollapsed(state, logicalCharPos(state, range.head, forward), forward, skipRanges);
		return select
			? EditorSelection.range(range.anchor, moved)
			: EditorSelection.cursor(moved, forward ? -1 : 1);
	});
	view.dispatch(
		state.update({
			selection: EditorSelection.create(ranges, sel.mainIndex),
			scrollIntoView: true,
			userEvent: 'select',
		}),
	);
	return true;
}

/** Is the base direction AT THE CARET left-to-right? (Per-line, courtesy of §A1's
 *  perLineTextDirection — the same signal CM6's cursorCharLeft/Right read.) */
function baseIsLtr(view: EditorView): boolean {
	return view.textDirectionAt(view.state.selection.main.head) === Direction.LTR;
}

/**
 * The logical-arrow keymap. `Prec.high` so it wins over `defaultKeymap`'s visual
 * ArrowLeft/ArrowRight (which are the ONLY other binding for these keys — conflict-checked).
 *
 * @param skipRanges Optional source of collapsed/replaced ranges the caret must not enter
 *                   (NotePane/merge panes pass the lens-block decorations; FocusPane omits it).
 */
export function logicalArrowKeymap(
	skipRanges?: (s: EditorState) => RangeSet<RangeValue> | null | undefined,
): Extension {
	const left: Command = (v) => moveLogical(v, !baseIsLtr(v), false, skipRanges);
	const right: Command = (v) => moveLogical(v, baseIsLtr(v), false, skipRanges);
	const selectLeft: Command = (v) => moveLogical(v, !baseIsLtr(v), true, skipRanges);
	const selectRight: Command = (v) => moveLogical(v, baseIsLtr(v), true, skipRanges);
	return Prec.high(
		keymap.of([
			{ key: 'ArrowLeft', run: left, shift: selectLeft, preventDefault: true },
			{ key: 'ArrowRight', run: right, shift: selectRight, preventDefault: true },
		]),
	);
}

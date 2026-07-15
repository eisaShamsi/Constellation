/**
 * PJ-106 §B1 — paragraph navigation (Word/Windows Ctrl+↑ / Ctrl+↓).
 *
 * THE CONCEPT (the horse): jump the caret between PARAGRAPHS, the way Ctrl+↑/↓ does in
 * Microsoft Word — the Boss's daily muscle memory, and his Round-1 symptom ("cannot navigate
 * through a line or a paragraph"). A "paragraph" in a Constellation note is a run of
 * consecutive non-blank document lines delimited by blank lines (the Markdown paragraph; lines
 * are soft-wrapped, so ONE document line = one paragraph even when it wraps across several
 * visual rows). Ctrl+↓ moves to the START of the next paragraph; Ctrl+↑ moves to the start of
 * the CURRENT paragraph, or — if the caret is already there — the start of the PREVIOUS one.
 *
 * WHY THIS IS DIRECTION-BLIND (unlike the arrows of §A5): "next paragraph" is always further
 * DOWN the document and "previous" always further UP — a vertical, logical relationship that
 * has nothing to do with left/right. So paragraph motion is byte-identical in LTR, RTL, and
 * bilingual notes; bidi never enters into it. The commands work on raw document offsets, so
 * they are unit-testable WITHOUT layout (the half of RTL navigation that genuinely can be), and
 * they carry no markdown knowledge — Rule-6-safe for FocusPane (no parser is pulled in).
 *
 * BINDING: Mod-ArrowUp / Mod-ArrowDown (= Ctrl+↑/↓ on Windows, Word's convention). Verified
 * UNBOUND in CM6's `defaultKeymap` AND in Constellation's command registry (only Alt+←/→ =
 * nav-back/forward and Alt+↑/↓ = move-line are taken), so `Prec.high` overrides nothing — it is
 * there for determinism, matching §A5's logical-arrow keymap. Hold Shift to extend the
 * selection paragraph-by-paragraph (Ctrl+Shift+↑/↓ — also the Word convention).
 */
import { EditorView, keymap, type Command } from '@codemirror/view';
import { EditorSelection, Prec, type EditorState, type Extension } from '@codemirror/state';

/** Is document line number `lineNo` blank (empty or whitespace-only)? Out-of-range → false. */
function isBlankLine(state: EditorState, lineNo: number): boolean {
	return lineNo >= 1 && lineNo <= state.doc.lines && state.doc.line(lineNo).text.trim() === '';
}

/** A line that BEGINS a paragraph: non-blank, and either the first line or preceded by a blank. */
function isParagraphStart(state: EditorState, lineNo: number): boolean {
	return !isBlankLine(state, lineNo) && (lineNo === 1 || isBlankLine(state, lineNo - 1));
}

/** Offset of the NEXT paragraph's start after `pos` — document end if there is none. */
export function paragraphForwardPos(state: EditorState, pos: number): number {
	const doc = state.doc;
	const from = doc.lineAt(pos).number;
	for (let n = from + 1; n <= doc.lines; n++) {
		if (isParagraphStart(state, n)) return doc.line(n).from;
	}
	return doc.length;
}

/** Offset of the CURRENT paragraph's start, or the PREVIOUS paragraph's start if the caret is
 *  already at the current start (document start if there is none) — the Word Ctrl+↑ rule. */
export function paragraphBackwardPos(state: EditorState, pos: number): number {
	const doc = state.doc;
	const cur = doc.lineAt(pos);
	let block = cur.number; // the line whose block we will search ABOVE
	if (!isBlankLine(state, cur.number)) {
		// Climb to the start of the paragraph the caret sits in.
		let s = cur.number;
		while (s > 1 && !isBlankLine(state, s - 1)) s--;
		const startFrom = doc.line(s).from;
		if (pos > startFrom) return startFrom; // → start of the CURRENT paragraph
		block = s; // already at the start → fall through to the PREVIOUS paragraph
	}
	// Step above the current block, skip blank lines, then climb to that paragraph's start.
	let n = block - 1;
	while (n >= 1 && isBlankLine(state, n)) n--;
	if (n < 1) return 0; // nothing above → document start
	while (n > 1 && !isBlankLine(state, n - 1)) n--;
	return doc.line(n).from;
}

/** Move (or extend the selection of) every cursor to the next/previous paragraph start. */
function moveParagraph(view: EditorView, forward: boolean, select: boolean): boolean {
	const state = view.state;
	const sel = state.selection;
	const ranges = sel.ranges.map((range) => {
		const target = forward
			? paragraphForwardPos(state, range.head)
			: paragraphBackwardPos(state, range.head);
		return select ? EditorSelection.range(range.anchor, target) : EditorSelection.cursor(target);
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

/** Ctrl+↑ / Ctrl+↓ paragraph navigation, + Shift to extend the selection (the Word convention). */
export function paragraphNavKeymap(): Extension {
	const up: Command = (v) => moveParagraph(v, false, false);
	const down: Command = (v) => moveParagraph(v, true, false);
	const selectUp: Command = (v) => moveParagraph(v, false, true);
	const selectDown: Command = (v) => moveParagraph(v, true, true);
	return Prec.high(
		keymap.of([
			{ key: 'Mod-ArrowUp', run: up, shift: selectUp, preventDefault: true },
			{ key: 'Mod-ArrowDown', run: down, shift: selectDown, preventDefault: true },
		]),
	);
}

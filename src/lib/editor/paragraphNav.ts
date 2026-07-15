/**
 * PJ-106 §B1 — paragraph navigation (Word/Windows Ctrl+↑ / Ctrl+↓)
 * PJ-106 §B2 — select by unit: line (Ctrl+L) and paragraph block (Ctrl+Shift+L).
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

// ── PJ-106 §B2 — select by unit (line / paragraph block) ──────────────────────
//
// THE CONCEPT (the horse): the Boss's Round-1 list — "cannot select a word, a sentence, a
// line, a whole paragraph, or a whole page." Word (double-click), page (Shift+PageUp/Down) and
// all (Ctrl+A) are already covered by CM6; sentence is §B3. This adds the two genuine gaps:
//   • LINE  (Ctrl+L)       — the current document line's TEXT.
//   • PARAGRAPH (Ctrl+Shift+L) — the whole blank-line-delimited block (may span several lines).
// Both select the TEXT only, never the trailing newline — §B0's rule: on an RTL line the
// newline's empty run sits to the LEFT of the right-aligned words, so including it paints the
// highlight past the text. (CM6's own Alt-l `selectLine` includes `to+1`; we override it here
// so Alt-l and Ctrl+L behave identically and correctly on RTL.) Direction-blind, offset-pure,
// parser-free — the range math is exported for headless tests.

/** The current document line(s)'s TEXT range for a selection span: [firstLine.from, lastLine.to]
 *  — never the trailing newline (§B0). */
export function lineTextRange(state: EditorState, from: number, to: number): { from: number; to: number } {
	const doc = state.doc;
	return { from: doc.lineAt(from).from, to: doc.lineAt(to).to };
}

/** The blank-line-delimited paragraph BLOCK spanning [from,to], text-only. A block is the run of
 *  consecutive non-blank lines around the span; a caret on a blank line selects just that line. */
export function paragraphBlockRange(state: EditorState, from: number, to: number): { from: number; to: number } {
	const doc = state.doc;
	let s = doc.lineAt(from).number;
	if (!isBlankLine(state, s)) while (s > 1 && !isBlankLine(state, s - 1)) s--;
	let e = doc.lineAt(to).number;
	if (!isBlankLine(state, e)) while (e < doc.lines && !isBlankLine(state, e + 1)) e++;
	return { from: doc.line(s).from, to: doc.line(e).to };
}

/** Apply a per-range text-range computer to every cursor and select the results. */
function selectByUnit(
	view: EditorView,
	unit: (state: EditorState, from: number, to: number) => { from: number; to: number },
): boolean {
	const state = view.state;
	const ranges = state.selection.ranges.map((range) => {
		const r = unit(state, range.from, range.to);
		return EditorSelection.range(r.from, r.to);
	});
	view.dispatch(
		state.update({
			selection: EditorSelection.create(ranges, state.selection.mainIndex),
			userEvent: 'select',
		}),
	);
	return true;
}

/** Ctrl+L → select the current line's text; Ctrl+Shift+L → select the whole paragraph block.
 *  Alt+L is overridden to the same text-only line select, fixing CM6's newline-including default
 *  on RTL lines (§B0). */
export function selectUnitKeymap(): Extension {
	const line: Command = (v) => selectByUnit(v, lineTextRange);
	const para: Command = (v) => selectByUnit(v, paragraphBlockRange);
	return Prec.high(
		keymap.of([
			{ key: 'Mod-l', run: line, preventDefault: true },
			{ key: 'Alt-l', run: line, preventDefault: true }, // override CM6 selectLine's trailing newline
			{ key: 'Shift-Mod-l', run: para, preventDefault: true },
		]),
	);
}

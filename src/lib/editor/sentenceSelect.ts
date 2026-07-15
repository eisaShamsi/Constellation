/**
 * PJ-106 §B3 — select the SENTENCE (Ctrl+click, + Ctrl+Shift+S at the caret).
 *
 * THE CONCEPT (the horse): the Boss's Round-1 "cannot select a sentence," resolved by the
 * Round-4 ruling — Ctrl+click selects the sentence (Microsoft Word's Windows convention; Word
 * on Mac uses Cmd+click), plus a keyboard command for the same. A "sentence" is found with the
 * platform's own Unicode sentence segmenter (`Intl.Segmenter`, granularity 'sentence' = the
 * UAX #29 algorithm), so Arabic terminators are handled natively and CORRECTLY: it breaks on
 * ؟ (U+061F), ۔ (U+06D4), ! and . but NOT on ؛ (U+061B, the Arabic SEMICOLON — Sentence_Break
 * SContinue, i.e. intra-sentence), and it does not false-break decimals ("3.14") or common
 * abbreviations. Verified live on the same V8 engine family WebView2 ships (design-inspection
 * H4): "…صحيح؟ نعم…! …الثالثة؛ ما زالت…" → 3 breaks, ؛ kept; "Pi is 3.14 exactly." → 1 segment.
 *
 * NO regex fallback: the naive `[؟!؛.]` splitter over-segments (breaks at ؛ mid-sentence, and
 * "3.14" into two). `Intl.Segmenter` is present on every evergreen WebView2; if it is somehow
 * absent we degrade to selecting the whole LINE — always correct, never a false split.
 *
 * Parser-free (`Intl.Segmenter` is a browser built-in) — Rule-6-safe for FocusPane.
 */
import { EditorView, keymap, type Command } from '@codemirror/view';
import { EditorSelection, Prec, type EditorState, type Extension } from '@codemirror/state';

// Minimal local shape so this compiles regardless of the tsconfig `lib` (the project has never
// referenced Intl.Segmenter before; we do not want to depend on its ambient type being present).
type SentenceSegmenter = { segment(input: string): Iterable<{ index: number; segment: string }> };

let segmenter: SentenceSegmenter | null | undefined;
function getSegmenter(): SentenceSegmenter | null {
	if (segmenter === undefined) {
		const I = Intl as unknown as {
			Segmenter?: new (locale?: string, opts?: { granularity: string }) => SentenceSegmenter;
		};
		try {
			segmenter = I.Segmenter ? new I.Segmenter(undefined, { granularity: 'sentence' }) : null;
		} catch {
			segmenter = null;
		}
	}
	return segmenter;
}

/** The sentence range containing `offset` within one line of text (offsets relative to the line).
 *  Trailing whitespace is trimmed so the highlight hugs the sentence but keeps its terminator.
 *  Returns null only for an empty line. Exported for headless tests. */
export function sentenceRangeInLine(lineText: string, offset: number): { from: number; to: number } | null {
	if (!lineText) return null;
	const seg = getSegmenter();
	// No segmenter → select the whole line (safe degradation, never a false split).
	if (!seg) return { from: 0, to: lineText.length };

	const off = Math.max(0, Math.min(offset, lineText.length));
	let match: { from: number; to: number } | null = null;
	let last: { from: number; to: number } | null = null;
	for (const s of seg.segment(lineText)) {
		const from = s.index;
		const to = s.index + s.segment.length;
		last = { from, to };
		if (off >= from && off < to) {
			match = { from, to };
			break;
		}
	}
	const chosen = match ?? last; // caret at the very end of the line → the last sentence
	if (!chosen) return null;
	// Intl sentence segments include the trailing space(s) up to the next sentence — trim them.
	let to = chosen.to;
	while (to > chosen.from && /\s/.test(lineText[to - 1])) to--;
	return { from: chosen.from, to };
}

/** Sentence range in DOCUMENT coordinates for a document position. A sentence never crosses a
 *  line break (UAX #29 breaks at LF), so segmenting the caret's line is exact and bounded. */
function sentenceRangeAtPos(state: EditorState, pos: number): { from: number; to: number } | null {
	const line = state.doc.lineAt(pos);
	const r = sentenceRangeInLine(line.text, pos - line.from);
	return r ? { from: line.from + r.from, to: line.from + r.to } : null;
}

/** Ctrl+click (Cmd+click on Mac) → select the sentence. Deliberately overrides CM6's Mod+click
 *  multi-cursor (Boss ruling, Round 4). Consulted BEFORE CM6's basicMouseSelection
 *  (view/index.js:4865), so it wins the click. Other gestures return null → CM6 default. */
export const ctrlClickSentence = EditorView.mouseSelectionStyle.of((view, event) => {
	const mod = event.ctrlKey || event.metaKey;
	if (!mod || event.altKey || event.shiftKey || event.button !== 0 || event.detail !== 1) return null;
	const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
	if (pos == null) return null;
	const range = sentenceRangeAtPos(view.state, pos);
	if (!range) return null;
	const sel = EditorSelection.create([EditorSelection.range(range.from, range.to)], 0);
	return {
		get: () => sel, // Ctrl+click selects the sentence; no drag-extend
		update: () => false,
	};
});

/** Keyboard "select sentence at caret" for each cursor. */
const selectSentence: Command = (view) => {
	const state = view.state;
	const ranges = state.selection.ranges.map((r) => {
		const range = sentenceRangeAtPos(state, r.head);
		return range ? EditorSelection.range(range.from, range.to) : r;
	});
	view.dispatch(
		state.update({
			selection: EditorSelection.create(ranges, state.selection.mainIndex),
			userEvent: 'select',
		}),
	);
	return true;
};

/** Ctrl+Shift+S → select the sentence at the caret. (Verified free in the app registry + CM6.) */
export function sentenceSelectKeymap(): Extension {
	return Prec.high(keymap.of([{ key: 'Mod-Shift-s', run: selectSentence, preventDefault: true }]));
}

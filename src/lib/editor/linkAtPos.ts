// PJ-114 — the ONE parser-free wikilink line-scan. A pure regex over a SINGLE line's text —
// NEVER the CM6 markdown parser. This is what lets a plain-text surface detect a [[link]]
// under the cursor while staying parser-free (Rule 6 + the Editor-Parity Focus exception).
//
// Consumers: FocusPane today; NotePane's editor menu joins in Phase-1 §7, replacing its own
// first-match-on-the-line regexes (which today target the WRONG link when a line has several).
//
// History: §0.2 extracted this from CodeMirrorEditor.svelte — a component that turned out to be
// unmounted dead code and was deleted in Phase-1 §1. The helper itself is sound (11 unit tests,
// tests/pj-114/linkAtPos.test.ts) and is genuinely used by FocusPane; only the "proven against
// the main editor" claim in §0.2 was wrong — NotePane builds its own editor and its own menu.

export interface WikilinkHit {
	/** The full inner text between `[[` `]]`, e.g. `"Note|alias"` or `"Note#heading"`. */
	raw: string;
	/** The resolved target name: alias (`|…`) and heading (`#…`) stripped, e.g. `"Note"`. */
	target: string;
	/** Column of the opening `[[` within the line (0-based). */
	from: number;
	/** Column just past the closing `]]` within the line. */
	to: number;
}

/**
 * Find a `[[wikilink]]` under `offset` within a single line's `lineText`.
 *
 * @param lineText the text of ONE editor line (never the whole document).
 * @param offset   the caret/click column within that line (0-based).
 * @returns the wikilink hit under `offset`, or `null` if `offset` is not inside one.
 *
 * The hit predicate (`offset` within `[from, to]`, inclusive of both ends) is identical to
 * the one the main editor's Ctrl/⌘-click has always used, so extracting this changes no
 * behavior. When multiple wikilinks share a line, the one containing `offset` is returned.
 */
export function findWikilinkAtLineOffset(lineText: string, offset: number): WikilinkHit | null {
	const wikiRe = /\[\[([^\]]+)\]\]/g;
	let match: RegExpExecArray | null;
	while ((match = wikiRe.exec(lineText)) !== null) {
		const from = match.index;
		const to = match.index + match[0].length;
		if (offset >= from && offset <= to) {
			const raw = match[1];
			const target = raw.split('|')[0].split('#')[0];
			return { raw, target, from, to };
		}
	}
	return null;
}

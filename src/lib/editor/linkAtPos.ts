// PJ-114 §0.2 — the ONE parser-free wikilink line-scan, shared by the main editor
// (CodeMirrorEditor Ctrl/⌘-click) and FocusPane's FM+ affordances (right-click menu,
// Mod-click/Mod-Enter follow). A pure regex over a SINGLE line's text — NEVER the CM6
// markdown parser. This is what lets FocusPane detect a [[link]] under the cursor while
// staying strictly plain-text/parser-free (Rule 6 + the Editor-Parity Focus exception).
//
// Extracted verbatim (same regex, same hit predicate, same alias/#heading stripping) from
// CodeMirrorEditor.svelte's Ctrl-click handler so both surfaces share one copy rather than
// duplicating the scan (the "no copy-paste-and-adapt — one source of truth" rule).

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

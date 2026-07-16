/**
 * PJ-106 §B4 — the per-paragraph direction override (Word's Right/Left-Ctrl+Shift).
 *
 * THE CONCEPT (the horse): the Boss's Round-3 directive — "Right Ctrl+Shift → the current
 * paragraph becomes 100% RTL; Left Ctrl+Shift → 100% LTR." A HARD, explicit direction that
 * WINS over the automatic first-strong detection: an Arabic paragraph can be forced LTR, a
 * Latin one forced RTL, and the choice survives close/reopen/sync because it is persisted
 * IN the text — as an invisible Unicode direction mark (RLM U+200F / LRM U+200E) at the
 * start of each paragraph line's CONTENT. Plain-text, portable (Word/Obsidian/browsers all
 * honor first-strong, and a leading mark IS the first strong character), byte-honest
 * (File-Over-App: the note carries its own formatting, no sidecar, no schema).
 *
 * SCOPE = the blank-line-delimited paragraph block the caret is in (B1/B2's definition),
 * or every block a selection touches. Every CONTENT line of the block gets the mark —
 * direction renders per-line in the editor, so marking each line is what "100%" means.
 *
 * MARKDOWN SAFETY (each rule earned by the §B4 adversarial review): the mark is inserted at
 * the line's CONTENT START — after whitespace and block markers (`- ` `1. ` `> ` `#` `- [ ] `)
 * — never before them. NEVER touched: blank lines (a mark would JOIN two markdown paragraphs),
 * a document-leading YAML frontmatter block (the merge view edits the FULL file — a marked
 * `title:` key would silently corrupt metadata), code fences and their CONTENT (an invisible
 * char in code corrupts it when copied out; the parity scan is CommonMark-aware — a closing
 * fence must match the opener's character and length, and fences nest under quote/list
 * markers), indented-code-shaped lines (4+ spaces / tab), table rows, rules and setext
 * underlines, link-reference/footnote definitions, and lines whose content BEGINS with a
 * #tag (a mark before `#` kills the tag in the index, tasks, and Obsidian alike — such a
 * line keeps automatic direction; the rest of its block still flips).
 *
 * LEFT-vs-RIGHT CTRL: a CM6 keymap keys on the logical key and cannot tell the sides apart,
 * so this is a raw DOM listener pair reading `KeyboardEvent.code` — registered as
 * `domEventObservers`, NOT `domEventHandlers`: observers always run, even for keys a keymap
 * consumed. (With handlers, Ctrl+Shift+S — consumed by §B3's keymap — would never reach the
 * disarm branch and the release would spuriously flip the paragraph: the review's app-killer.)
 * The gesture fires on the Windows modifier-hotkey convention — Ctrl+Shift pressed and
 * RELEASED with no other key or click in between; the CTRL key's side decides. AltGr layouts
 * are belted out (`getModifierState('AltGraph')`).
 *
 * SAVE PATH: the change is dispatched as ONE normal edit (userEvent 'input') — it flows
 * through the exact keystroke pipeline (onDocChange → model setBody → debounced durable
 * save). `isolateHistory` keeps it a self-contained undo step (typing right after must not
 * join it — Ctrl+Z would silently strip the whole override). The selection is mapped with
 * assoc 1 so the caret lands AFTER the inserted mark — typing immediately continues INSIDE
 * the override instead of in front of it. Read-only surfaces are belted out.
 */
import { EditorView } from '@codemirror/view';
import { EditorSelection, type ChangeSpec, type EditorState, type Extension } from '@codemirror/state';
import { isolateHistory } from '@codemirror/commands';
import { paragraphBlockRange } from './paragraphNav';
import { BLOCK_PREFIX_RE } from './bidiPlugin';

export const RLM = '‏'; // RIGHT-TO-LEFT MARK — invisible strong-RTL
export const LRM = '‎'; // LEFT-TO-RIGHT MARK — invisible strong-LTR

const FENCE_RE = /^(`{3,}|~{3,})/;
const INDENTED_CODE_RE = /^(?: {4,}|\t)/;

/** Lines that must NEVER receive a mark: no real content, or position-sensitive structure. */
function lineEligible(text: string): boolean {
	if (!text.trim()) return false; // blank
	if (/^\s*\|/.test(text)) return false; // table row — a leading mark breaks GFM tables
	if (INDENTED_CODE_RE.test(text)) return false; // indented-code shape — never mark code
	if (/^\s{0,3}(?:[-*_]\s*){3,}\s*$/.test(text)) return false; // horizontal rule
	if (/^\s{0,3}(?:=+|-+)\s*$/.test(text)) return false; // setext underline
	if (/^\s{0,3}\[[^\]]+\]:\s/.test(text)) return false; // link-ref / footnote definition
	// Syntax-only (pure markers, no letters and no digits — digit-only lines like `123 456`
	// ARE content and stay eligible so a forced block never renders half-flipped).
	if (/^[#*>\-\s`\[\]()!|~=+.‎‏]*$/.test(text)) return false;
	return true;
}

/** Offset (within the line) where the mark belongs — after whitespace + block markers. */
export function contentStartOffset(lineText: string): number {
	return BLOCK_PREFIX_RE.exec(lineText)![0].length;
}

/**
 * The pure core: compute the ChangeSpecs that force every content line of every paragraph
 * block touched by the selection to `dir`. Replaces any existing leading mark-run at the
 * content start (switching direction = one replace), no-ops when already set (idempotent —
 * repeat presses never dirty the note). One linear pass from the doc start tracks the two
 * skip regions — a document-leading frontmatter block and code fences (opener character +
 * length matched per CommonMark, block markers stripped first so quoted/listed fences count).
 * This is an occasional gesture, not a keystroke path; O(doc) here is §A2's Enter budget.
 * Exported for headless tests.
 */
export function computeParagraphDirChanges(state: EditorState, dir: 'rtl' | 'ltr'): ChangeSpec[] {
	const desired = dir === 'rtl' ? RLM : LRM;
	const doc = state.doc;

	// 1. A document-leading YAML frontmatter block is untouchable (the conflict-merge view
	//    edits the FULL file; NotePane/FocusPane docs are body-only, so this only ever
	//    suppresses that case — a body genuinely starting with `---` merely no-ops).
	let fmEndLine = 0;
	if (doc.lines > 1 && doc.line(1).text.trim() === '---') {
		for (let n = 2; n <= doc.lines; n++) {
			if (doc.line(n).text.trim() === '---') { fmEndLine = n; break; }
		}
	}
	const fmEnd = fmEndLine > 0 ? doc.line(fmEndLine).to : -1;

	// 2. The target lines: every line of every paragraph block the selection touches.
	//    A caret/selection WHOLLY inside the frontmatter targets nothing — the user is not
	//    in a prose paragraph, and a gesture there must not reach across the fence into the
	//    body (the least-surprise contract; a mixed selection still covers its body lines).
	const targets = new Set<number>();
	let lastTarget = 0;
	for (const range of state.selection.ranges) {
		if (fmEndLine > 0 && range.from <= fmEnd && range.to <= fmEnd) continue;
		const block = paragraphBlockRange(state, range.from, range.to);
		const first = doc.lineAt(block.from).number;
		const last = doc.lineAt(block.to).number;
		for (let n = first; n <= last; n++) targets.add(n);
		if (last > lastTarget) lastTarget = last;
	}

	// 3. One pass to the last target: fence parity (opener-matched), then per-line changes.
	const changes: ChangeSpec[] = [];
	let fence: { char: string; len: number } | null = null;
	for (let n = 1; n <= lastTarget; n++) {
		const line = doc.line(n);
		if (n <= fmEndLine) continue; // frontmatter — structural, never marked
		const raw = line.text;
		// Fence detection: block markers stripped first (a fence nested under `> ` still
		// fences); an indented-code-shaped line is content, never a fence delimiter.
		if (!INDENTED_CODE_RE.test(raw)) {
			const m = FENCE_RE.exec(raw.replace(BLOCK_PREFIX_RE, ''));
			if (m) {
				const char = m[1][0];
				const len = m[1].length;
				if (!fence) { fence = { char, len }; continue; }
				// Only a MATCHING closer ends the fence (CommonMark): same char, ≥ length.
				if (char === fence.char && len >= fence.len) { fence = null; continue; }
				// A different fence-looking line INSIDE a fence is code content — falls through.
			}
		}
		if (fence) continue; // inside a code fence — content is code, never marked
		if (!targets.has(n)) continue;
		if (!lineEligible(raw)) continue;
		const cs = contentStartOffset(raw);
		// A content-leading #tag would be severed from the index/tasks/Obsidian by ANY mark
		// before the `#` — that line keeps automatic direction (the rest of the block flips).
		if (/^#[^\s#]/.test(raw.slice(cs))) continue;
		// The existing mark-run at the content start (ours or imported) — replaced wholesale.
		let runEnd = cs;
		while (runEnd < raw.length && (raw[runEnd] === RLM || raw[runEnd] === LRM)) runEnd++;
		if (raw.slice(cs, runEnd) === desired) continue; // idempotent — no change, no dirty
		changes.push({ from: line.from + cs, to: line.from + runEnd, insert: desired });
	}
	return changes;
}

/** Apply the override to the live view as ONE normal, undoable, self-contained edit. */
export function setParagraphDirection(view: EditorView, dir: 'rtl' | 'ltr'): boolean {
	if (view.state.readOnly) return false; // Display-not-Domain — read-only surfaces never write
	const specs = computeParagraphDirChanges(view.state, dir);
	if (specs.length === 0) return true; // already set — handled, nothing to do
	const changes = view.state.changes(specs);
	// assoc 1: a caret sitting exactly at an insertion point lands AFTER the new mark, so
	// immediate typing continues inside the override instead of re-flipping the line.
	const sel = view.state.selection;
	const selection = EditorSelection.create(
		sel.ranges.map((r) => EditorSelection.range(changes.mapPos(r.anchor, 1), changes.mapPos(r.head, 1))),
		sel.mainIndex,
	);
	view.dispatch({
		changes,
		selection,
		userEvent: 'input',
		annotations: isolateHistory.of('full'), // one self-contained undo step, nothing joins it
	});
	return true;
}

/**
 * The Right/Left-Ctrl+Shift gesture. Arm on both-modifiers-down (the CTRL side decides),
 * DISARM on any other keydown or mousedown, fire once on the first modifier keyup — the
 * Windows modifier-hotkey convention. Registered as domEventObservers so the disarm sees
 * EVERY key, including chords a keymap consumed (Ctrl+Shift+S/L/F…) — with plain handlers
 * those never arrive and their release would spuriously flip the paragraph.
 */
export function paragraphDirKeys(): Extension {
	let ctrlSide: 'left' | 'right' = 'left';
	let armed: 'left' | 'right' | null = null;
	return EditorView.domEventObservers({
		keydown(e, view) {
			if (e.getModifierState?.('AltGraph')) { armed = null; return; } // AltGr chord ≠ gesture
			if (e.key === 'Control') {
				ctrlSide = e.code === 'ControlRight' ? 'right' : 'left';
				if (e.shiftKey) armed = ctrlSide; // Shift was already down — both held now
			} else if (e.key === 'Shift') {
				if (e.ctrlKey) armed = ctrlSide; // Ctrl was already down — both held now
			} else {
				armed = null; // any real key (Ctrl+Shift+S, …) cancels the gesture
			}
		},
		keyup(e, view) {
			if (armed && (e.key === 'Control' || e.key === 'Shift')) {
				const side = armed;
				armed = null; // fire once
				if (!view.state.readOnly) {
					setParagraphDirection(view, side === 'right' ? 'rtl' : 'ltr');
				}
			}
		},
		mousedown() {
			armed = null; // a click moves the caret — firing after it would hit the WRONG paragraph
		},
		blur() {
			armed = null; // focus left mid-gesture — never fire on a stale arm
		},
	});
}

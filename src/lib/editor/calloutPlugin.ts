/**
 * Callout Plugin — Obsidian-compatible callout rendering for CodeMirror 6.
 *
 * ┌─────────────────────────────────────────────────────────────────────────────┐
 * │  FREEZE-PROOF ARCHITECTURE — Two inviolable rules                           │
 * │                                                                             │
 * │  ROOT CAUSE of CM6 freeze:                                                  │
 * │  Decoration.replace([from, to]) creates a cursor-exclusion range.           │
 * │  If the cursor is inside that range, CM6 nudges it out → selectionSet       │
 * │  fires → plugin rebuilds → range is restored → CM6 nudges again → loop.    │
 * │  The editor freezes permanently.                                            │
 * │                                                                             │
 * │  RULE A — Cursor-safe replace (title widget + ">" prefix removal):          │
 * │    Decoration.replace is ONLY added when the cursor is on a DIFFERENT line. │
 * │    At line granularity this is provably safe: a cursor on line N cannot     │
 * │    be inside a replace range that covers exclusively line M (M ≠ N).       │
 * │                                                                             │
 * │  RULE B — Zero-length line decoration (collapsed body hiding):              │
 * │    Decoration.line({ class }) is added at (line.from, line.from).           │
 * │    from === to → no range → cursor can never be "inside" it → CM6          │
 * │    never nudges → freeze loop is architecturally impossible.                │
 * │    CSS display:none on .cm-callout-body-collapsed does the actual hiding.   │
 * └─────────────────────────────────────────────────────────────────────────────┘
 *
 * Other design decisions:
 *   - Per-type color via CSS --callout-color (no color values in JS)
 *   - Fold state: StateField<Set<number>> survives doc changes via pos mapping
 *   - Chevron click: handled by NotePane's capture-phase mousedown — NOT here
 *     (avoids double-dispatch when both a plugin handler and a native handler exist)
 *   - Viewport-only scan: O(visible_lines), never O(document)
 */
import {
	ViewPlugin,
	type ViewUpdate,
	Decoration,
	type DecorationSet,
	EditorView,
	WidgetType,
} from '@codemirror/view';
import { RangeSetBuilder, StateField, StateEffect } from '@codemirror/state';

// ─── Icon map ─────────────────────────────────────────────────────────────────
const CALLOUT_ICONS: Record<string, string> = {
	note: 'ℹ️',      info: 'ℹ️',
	tip: '💡',       hint: '💡',       important: '💡',
	success: '✅',   check: '✅',      done: '✅',
	question: '❓',  help: '❓',       faq: '❓',
	warning: '⚠️',   caution: '⚠️',    attention: '⚠️',
	failure: '❌',   fail: '❌',       missing: '❌',
	danger: '⛔',    error: '⛔',
	bug: '🐛',
	example: '📝',
	quote: '💬',     cite: '💬',
	abstract: '📋',  summary: '📋',    tldr: '📋',
};

// ─── Fold state ───────────────────────────────────────────────────────────────
// Dispatch toggleCallout.of(startLineNumber) to toggle a callout open/closed.
// The field stores a Set of *toggled* line numbers — flipped from their default state.
// (A "-" marker means defaultCollapsed=true. Adding its line to the Set means open.)
export const toggleCallout = StateEffect.define<number>();

export const calloutCollapseField = StateField.define<Set<number>>({
	create: () => new Set(),
	update(collapsed, tr) {
		let next = collapsed;

		// Remap stored line numbers through document edits so fold state survives typing
		if (tr.docChanged) {
			const mapped = new Set<number>();
			for (const ln of collapsed) {
				try {
					const oldPos = tr.startState.doc.line(ln).from;
					mapped.add(tr.state.doc.lineAt(tr.changes.mapPos(oldPos)).number);
				} catch { /* line was deleted — drop it */ }
			}
			next = mapped;
		}

		// Apply toggle effects from this transaction
		for (const e of tr.effects) {
			if (e.is(toggleCallout)) {
				next = new Set(next);
				next.has(e.value) ? next.delete(e.value) : next.add(e.value);
			}
		}

		return next;
	},
});

// ─── Title widget ─────────────────────────────────────────────────────────────
// Replaces the raw "> [!type]- My Title" line with a styled widget.
// Only added when the cursor is on a DIFFERENT line (RULE A).
class CalloutTitleWidget extends WidgetType {
	constructor(
		readonly type: string,
		readonly icon: string,
		readonly title: string,
		readonly foldable: boolean,
		readonly collapsed: boolean,
		readonly lineNum: number,
	) { super(); }

	toDOM() {
		// Detect direction from title text (RTL scripts → rtl, else ltr)
		const isRtl = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/.test(this.title);
		const titleDir = isRtl ? 'rtl' : 'ltr';

		const wrap = document.createElement('span');
		wrap.className = 'cm-callout-title-widget';
		wrap.setAttribute('dir', titleDir);
		wrap.setAttribute('data-callout', this.type);

		// Icon
		const iconEl = document.createElement('span');
		iconEl.className = 'cm-callout-icon';
		iconEl.textContent = this.icon;
		wrap.appendChild(iconEl);

		// Fold chevron (only rendered if the callout has a +/- marker)
		if (this.foldable) {
			const chevron = document.createElement('span');
			chevron.className = 'cm-callout-chevron';
			chevron.textContent = this.collapsed ? ' ▶' : ' ▼';
			chevron.dataset.calloutLine = String(this.lineNum);
			wrap.appendChild(chevron);
		}

		// Title text
		const titleEl = document.createElement('span');
		titleEl.className = 'cm-callout-title-text';
		titleEl.textContent = this.title ? ' ' + this.title : '';
		wrap.appendChild(titleEl);

		return wrap;
	}

	// CM6 reuses existing DOM if eq() returns true — no needless rebuilds
	eq(other: CalloutTitleWidget): boolean {
		return (
			this.type === other.type &&
			this.title === other.title &&
			this.foldable === other.foldable &&
			this.collapsed === other.collapsed &&
			this.lineNum === other.lineNum
		);
	}

	// true → CM6 does not swallow events from this widget → they bubble to NotePane
	ignoreEvent() { return true; }
}

// ─── Callout detection ────────────────────────────────────────────────────────
interface CalloutBlock {
	type: string;
	foldMarker: string; // '+' | '-' | ''
	startLine: number;
	endLine: number;
}

/**
 * Finds every callout block whose *start* line falls within [fromLine, toLine].
 * A callout block is a run of contiguous "> " lines starting with "> [!type]".
 */
function findCalloutsInRange(
	doc: EditorView['state']['doc'],
	fromLine: number,
	toLine: number,
): CalloutBlock[] {
	const result: CalloutBlock[] = [];
	let ln = fromLine;

	while (ln <= toLine) {
		const text = doc.line(ln).text;
		const m = text.match(/^>\s*\[!(\w+)\]([+-])?\s*/);
		if (m) {
			const type = m[1].toLowerCase();
			const foldMarker = m[2] ?? '';

			// Walk forward to find the last body line
			const cap = Math.min(doc.lines, ln + 200);
			let endLine = ln;
			for (let l = ln + 1; l <= cap; l++) {
				const t = doc.line(l).text;
				if (!/^>\s?/.test(t) || /^>\s*\[!\w+\]/.test(t)) break;
				endLine = l;
			}

			result.push({ type, foldMarker, startLine: ln, endLine });
			ln = endLine + 1; // skip to after this callout
		} else {
			ln++;
		}
	}

	return result;
}

// ─── Decoration builder ───────────────────────────────────────────────────────
function buildCalloutDecorations(view: EditorView): DecorationSet {
	const { doc } = view.state;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const collapsed = view.state.field(calloutCollapseField);

	// Collect then sort — RangeSetBuilder requires strictly ascending positions
	const all: { from: number; to: number; deco: Decoration }[] = [];

	for (const { from, to } of view.visibleRanges) {
		// Scan a few lines above viewport so callouts starting just off-screen render correctly
		const startLine = Math.max(1, doc.lineAt(from).number - 5);
		const endLine = doc.lineAt(to).number;

		for (const callout of findCalloutsInRange(doc, startLine, endLine)) {
			const icon = CALLOUT_ICONS[callout.type] ?? 'ℹ️';
			const foldable = callout.foldMarker === '-' || callout.foldMarker === '+';
			const defaultCollapsed = callout.foldMarker === '-';

			// XOR toggle: if the line is in the Set, its visible state is flipped
			const isCollapsed = collapsed.has(callout.startLine)
				? !defaultCollapsed
				: defaultCollapsed;

			const titleLine = doc.line(callout.startLine);

			// ── 1. Title line border + tint (always, even when cursor is on it) ──
			all.push({
				from: titleLine.from, to: titleLine.from,
				deco: Decoration.line({
					class: 'cm-callout-line cm-callout-title-line',
					attributes: { 'data-callout': callout.type, dir: 'auto' },
				}),
			});

			// ── 2. Title widget (RULE A: only when cursor is elsewhere) ──
			if (cursorLine !== callout.startLine) {
				const rawTitle = titleLine.text
					.replace(/^>\s*\[!\w+\][+-]?\s*/, '')
					.trim();
				all.push({
					from: titleLine.from,
					to: titleLine.to,
					deco: Decoration.replace({
						widget: new CalloutTitleWidget(
							callout.type, icon, rawTitle,
							foldable, isCollapsed, callout.startLine,
						),
					}),
				});
			}

			if (!isCollapsed) {
				// ── 3. Body lines: border + tint + ">" prefix removal ──
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					const line = doc.line(l);

					// Border + tint (zero-length — always safe)
					all.push({
						from: line.from, to: line.from,
						deco: Decoration.line({
							class: 'cm-callout-line cm-callout-body-line',
							attributes: { 'data-callout': callout.type, dir: 'auto' },
						}),
					});

					// ">" prefix removal (RULE A: skip cursor line)
					if (cursorLine !== l) {
						const prefix = line.text.match(/^>\s?/);
						if (prefix) {
							all.push({
								from: line.from,
								to: line.from + prefix[0].length,
								deco: Decoration.replace({}),
							});
						}
					}
				}
			} else {
				// ── 4. Collapsed body: hide with CSS (RULE B: zero-length line deco) ──
				// The cursor's own line is always kept visible so the cursor never disappears.
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					if (cursorLine === l) continue; // never hide cursor line
					const line = doc.line(l);
					all.push({
						from: line.from, to: line.from,  // from === to = zero-length
						deco: Decoration.line({ class: 'cm-callout-body-collapsed' }),
					});
				}
			}
		}
	}

	// Sort ascending by position; line decos (to===from) before inline decos at same pos
	all.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const { from, to, deco } of all) builder.add(from, to, deco);
	return builder.finish();
}

// ─── ViewPlugin ───────────────────────────────────────────────────────────────
class CalloutDecoPlugin {
	decorations: DecorationSet;
	private lastCursorLine = -1;

	constructor(view: EditorView) {
		this.decorations = buildCalloutDecorations(view);
		this.lastCursorLine = view.state.doc.lineAt(view.state.selection.main.head).number;
	}

	update(update: ViewUpdate) {
		const hasToggle = update.transactions.some(
			t => t.effects.some(e => e.is(toggleCallout))
		);

		// Full rebuild on: fold toggle, doc edit, or viewport scroll
		if (hasToggle || update.docChanged || update.viewportChanged) {
			this.decorations = buildCalloutDecorations(update.view);
			this.lastCursorLine = update.view.state.doc
				.lineAt(update.view.state.selection.main.head).number;
			return;
		}

		// Cursor move: only rebuild when cursor crosses a line boundary
		// (title widget visibility depends on which line the cursor is on)
		if (update.selectionSet) {
			const newLine = update.view.state.doc
				.lineAt(update.view.state.selection.main.head).number;
			if (newLine !== this.lastCursorLine) {
				this.lastCursorLine = newLine;
				this.decorations = buildCalloutDecorations(update.view);
			}
		}
	}
}

const calloutDecoPlugin = ViewPlugin.fromClass(CalloutDecoPlugin, {
	decorations: v => v.decorations,
});

// ─── Exports ──────────────────────────────────────────────────────────────────
// Chevron click is handled by NotePane's chevronHandler (capture-phase mousedown).
// We do NOT register a domEventHandlers here — two handlers on the same element
// would cause double-dispatch in edge cases.
export const calloutPlugin = [calloutDecoPlugin];
export const calloutClickHandler = EditorView.domEventHandlers({}); // kept for import compat

export const calloutTheme = EditorView.theme({
	// Left border on every callout line
	'.cm-callout-line': {
		borderInlineStart: '3px solid var(--callout-color, #448aff)',
		paddingInlineStart: '12px !important',
	},
	// Title line: stronger background + bold
	'.cm-callout-title-line': {
		background: 'color-mix(in srgb, var(--callout-color, #448aff) 8%, transparent)',
		fontWeight: '500',
	},
	// Body line: lighter background
	'.cm-callout-body-line': {
		background: 'color-mix(in srgb, var(--callout-color, #448aff) 4%, transparent)',
	},
	// RULE B — collapsed body lines: hidden by CSS, not by Decoration.replace.
	// CM6 sees these lines as normal (zero-length deco) so the cursor can move
	// through them freely and the cursor's own line is always kept visible above.
	'.cm-callout-body-collapsed': {
		display: 'none',
	},
	// Widget layout
	'.cm-callout-title-widget': { display: 'inline', direction: 'inherit' },
	'.cm-callout-icon': {
		fontWeight: '600',
		fontSize: '0.95em',
		verticalAlign: 'middle',
		color: 'var(--callout-color, #448aff)',
	},
	'.cm-callout-chevron': {
		cursor: 'pointer',
		fontSize: '0.8em',
		opacity: '0.7',
		userSelect: 'none',
		verticalAlign: 'middle',
	},
	'.cm-callout-title-text': { fontWeight: '600' },

	// Per-type color variables — add any new type here, never in JS
	'[data-callout="note"]':      { '--callout-color': '#448aff' },
	'[data-callout="info"]':      { '--callout-color': '#448aff' },
	'[data-callout="abstract"]':  { '--callout-color': '#00b0ff' },
	'[data-callout="summary"]':   { '--callout-color': '#00b0ff' },
	'[data-callout="tldr"]':      { '--callout-color': '#00b0ff' },
	'[data-callout="tip"]':       { '--callout-color': '#00bfa5' },
	'[data-callout="hint"]':      { '--callout-color': '#00bfa5' },
	'[data-callout="important"]': { '--callout-color': '#00bfa5' },
	'[data-callout="success"]':   { '--callout-color': '#00c853' },
	'[data-callout="check"]':     { '--callout-color': '#00c853' },
	'[data-callout="done"]':      { '--callout-color': '#00c853' },
	'[data-callout="question"]':  { '--callout-color': '#ff9100' },
	'[data-callout="help"]':      { '--callout-color': '#ff9100' },
	'[data-callout="faq"]':       { '--callout-color': '#ff9100' },
	'[data-callout="warning"]':   { '--callout-color': '#ff9100' },
	'[data-callout="caution"]':   { '--callout-color': '#ff9100' },
	'[data-callout="attention"]': { '--callout-color': '#ff9100' },
	'[data-callout="failure"]':   { '--callout-color': '#ff5252' },
	'[data-callout="fail"]':      { '--callout-color': '#ff5252' },
	'[data-callout="missing"]':   { '--callout-color': '#ff5252' },
	'[data-callout="danger"]':    { '--callout-color': '#ff1744' },
	'[data-callout="error"]':     { '--callout-color': '#ff1744' },
	'[data-callout="bug"]':       { '--callout-color': '#ff1744' },
	'[data-callout="example"]':   { '--callout-color': '#7c4dff' },
	'[data-callout="quote"]':     { '--callout-color': '#9e9e9e' },
	'[data-callout="cite"]':      { '--callout-color': '#9e9e9e' },
});

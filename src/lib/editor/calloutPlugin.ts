/**
 * Callout Plugin — Obsidian-compatible callout rendering for CodeMirror 6.
 *
 * Architecture (matches Obsidian's proven approach):
 *
 * 1. FULL LINE replacement: when cursor is NOT on the title line, the entire
 *    line content is replaced by a CalloutTitleWidget. This makes the cursor
 *    provably safe — it is on a different line → can never be inside the
 *    replace range → no "cursor in replaced range" freeze loop.
 *    When cursor IS on the title line → no replace → raw markdown for editing.
 *
 * 2. SEPARATE domEventHandlers for fold click: registered as a standalone
 *    EditorView extension, not embedded in the ViewPlugin. Avoids the
 *    stopImmediatePropagation / priority issues that silently break
 *    ViewPlugin.eventHandlers for widget clicks.
 *
 * 3. CSS-driven colors via data-callout="type" + --callout-color variable.
 *    No color values in JS — any new type is one CSS rule.
 *
 * 4. eq() on widget prevents DOM rebuilds when nothing changed.
 * 5. Viewport-only scan — O(visible_lines), never O(document).
 * 6. Synchronous rebuild — regex-only, no syntaxTree, <1ms per update.
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

// ─── Icon map ───
const calloutIcons: Record<string, string> = {
	note: 'ℹ️',    info: 'ℹ️',
	tip: '💡',     hint: '💡',     important: '💡',
	success: '✅',  check: '✅',   done: '✅',
	question: '❓', help: '❓',    faq: '❓',
	warning: '⚠️',  caution: '⚠️', attention: '⚠️',
	failure: '❌',  fail: '❌',    missing: '❌',
	danger: '⛔',   error: '⛔',
	bug: '🐛',
	example: '📝',
	quote: '💬',    cite: '💬',
	abstract: '📋', summary: '📋', tldr: '📋',
};

// ─── Fold state ───
export const toggleCallout = StateEffect.define<number>();

export const calloutCollapseField = StateField.define<Set<number>>({
	create: () => new Set(),
	update(collapsed, tr) {
		let next = collapsed;
		if (tr.docChanged) {
			const newSet = new Set<number>();
			for (const oldLine of collapsed) {
				try {
					const oldPos = tr.startState.doc.line(oldLine).from;
					newSet.add(tr.state.doc.lineAt(tr.changes.mapPos(oldPos)).number);
				} catch { /* line gone */ }
			}
			next = newSet;
		}
		for (const e of tr.effects) {
			if (e.is(toggleCallout)) {
				next = new Set(next);
				if (next.has(e.value)) next.delete(e.value);
				else next.add(e.value);
			}
		}
		return next;
	},
});

// ─── Widget: renders the entire title line content ───
// Replaces the full "> [!type]- My Title" line when cursor is elsewhere.
// Safe: cursor is on a DIFFERENT line → never inside this replace range.
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
		const wrap = document.createElement('span');
		wrap.className = 'cm-callout-title-widget';
		wrap.setAttribute('dir', 'auto');
		wrap.setAttribute('data-callout', this.type);

		const iconEl = document.createElement('span');
		iconEl.className = 'cm-callout-icon';
		iconEl.textContent = this.icon;
		wrap.appendChild(iconEl);

		if (this.foldable) {
			const chevron = document.createElement('span');
			chevron.className = 'cm-callout-chevron';
			chevron.textContent = this.collapsed ? ' ▶' : ' ▼';
			chevron.dataset.calloutLine = String(this.lineNum);
			wrap.appendChild(chevron);
		}

		const titleEl = document.createElement('span');
		titleEl.className = 'cm-callout-title-text';
		titleEl.textContent = this.title ? ' ' + this.title : '';
		wrap.appendChild(titleEl);

		return wrap;
	}

	// eq() prevents CM6 from destroying/recreating the DOM node when unchanged
	eq(other: CalloutTitleWidget) {
		return (
			this.type === other.type &&
			this.title === other.title &&
			this.foldable === other.foldable &&
			this.collapsed === other.collapsed &&
			this.lineNum === other.lineNum
		);
	}

	// true = CM6 does not consume this widget's events → they bubble normally
	ignoreEvent() { return true; }
}

// ─── Callout block ───
interface CalloutBlock {
	type: string;
	foldMarker: string;
	startLine: number;
	endLine: number;
}

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
			const foldMarker = m[2] || '';
			const cap = Math.min(doc.lines, ln + 200);
			let last = ln;
			for (let l = ln + 1; l <= cap; l++) {
				const t = doc.line(l).text;
				if (!/^>\s?/.test(t) || /^>\s*\[!\w+\]/.test(t)) break;
				last = l;
			}
			result.push({ type, foldMarker, startLine: ln, endLine: last });
			ln = last + 1;
		} else {
			ln++;
		}
	}
	return result;
}

function buildCalloutDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const collapsed = view.state.field(calloutCollapseField);
	const all: { from: number; to: number; deco: Decoration }[] = [];

	for (const { from, to } of view.visibleRanges) {
		const startLine = Math.max(1, doc.lineAt(from).number - 5);
		const endLine = doc.lineAt(to).number;

		for (const callout of findCalloutsInRange(doc, startLine, endLine)) {
			const icon = calloutIcons[callout.type] || 'ℹ️';
			const foldable = callout.foldMarker === '-' || callout.foldMarker === '+';
			const defaultCollapsed = callout.foldMarker === '-';
			const isCollapsed = collapsed.has(callout.startLine)
				? !defaultCollapsed
				: defaultCollapsed;

			const titleLine = doc.line(callout.startLine);
			const cursorOnTitle = cursorLine === callout.startLine;
			const cursorInBody = cursorLine > callout.startLine
				&& cursorLine <= callout.endLine;

			// ── Title line: border + tinted background (always) ──
			all.push({
				from: titleLine.from, to: titleLine.from,
				deco: Decoration.line({
					class: 'cm-callout-line cm-callout-title-line',
					attributes: { 'data-callout': callout.type },
				}),
			});

			// ── Full-line widget: shown when cursor is on a different line ──
			// SAFE: cursor is on another line → provably outside this replace range.
			// When cursor IS on title line → no replace → raw markdown for editing.
			if (!cursorOnTitle) {
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
				// ── Body lines: border + lighter tint ──
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					const line = doc.line(l);
					all.push({
						from: line.from, to: line.from,
						deco: Decoration.line({
							class: 'cm-callout-line cm-callout-body-line',
							attributes: { 'data-callout': callout.type },
						}),
					});
					// Remove "> " prefix — skip if cursor is on this line
					if (cursorLine !== l) {
						const qm = line.text.match(/^>\s?/);
						if (qm) {
							all.push({
								from: line.from,
								to: line.from + qm[0].length,
								deco: Decoration.replace({}),
							});
						}
					}
				}
			} else if (!cursorOnTitle && !cursorInBody && callout.endLine > callout.startLine) {
				// ── Collapse body ──
				// Safe guards: cursor NOT on title (so cursor < titleLine.to, safely before
				// the range start) AND cursor NOT in body (so cursor not inside range).
				// If cursor is on the title line, titleLine.to is the range boundary — CM6
				// may nudge cursor there → rebuild → nudge loop → freeze. Skip collapse.
				all.push({
					from: titleLine.to,
					to: doc.line(callout.endLine).to,
					deco: Decoration.replace({}),
				});
			}
		}
	}

	// RangeSetBuilder requires ascending order; line decos (from===to) before inline
	all.sort((a, b) => a.from - b.from || a.to - b.to);
	const builder = new RangeSetBuilder<Decoration>();
	for (const r of all) builder.add(r.from, r.to, r.deco);
	return builder.finish();
}

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
		if (hasToggle || update.docChanged || update.viewportChanged) {
			this.decorations = buildCalloutDecorations(update.view);
			this.lastCursorLine = update.view.state.doc
				.lineAt(update.view.state.selection.main.head).number;
			return;
		}
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

// ─── Exports ───
// Fold click is handled by NotePane's native chevronHandler (capture phase).
// We don't register a duplicate domEventHandlers here — two handlers on the same
// event would cause double dispatches in some edge cases.
export const calloutPlugin = [calloutDecoPlugin];

// Backward compat (NotePane / CodeMirrorEditor import these individually)
export const calloutClickHandler = EditorView.domEventHandlers({});

export const calloutTheme = EditorView.theme({
	'.cm-callout-line': {
		borderInlineStart: '3px solid var(--callout-color, #448aff)',
		paddingInlineStart: '12px !important',
	},
	'.cm-callout-title-line': {
		background: 'color-mix(in srgb, var(--callout-color, #448aff) 8%, transparent)',
		fontWeight: '500',
	},
	'.cm-callout-body-line': {
		background: 'color-mix(in srgb, var(--callout-color, #448aff) 4%, transparent)',
	},
	'.cm-callout-title-widget': {
		display: 'inline',
	},
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
	'.cm-callout-title-text': {
		fontWeight: '600',
	},
	// Per-type color variables
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

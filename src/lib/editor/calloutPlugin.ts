/**
 * Callout Plugin — CodeMirror ViewPlugin that renders Obsidian-style callouts.
 *
 * Architecture (per reference doc):
 * - Colors driven entirely by CSS via data-callout="type" attribute + --callout-color variable.
 *   No color values in JS. Any callout type works automatically via CSS cascade.
 * - Icons remain as emoji text nodes in the widget (CSS ::before is messier for emoji).
 * - dir="auto" on the widget container for correct RTL icon/chevron placement.
 * - Fold state in CM6 StateField — survives virtualization / offscreen destruction.
 * - eq() on WidgetType prevents unnecessary DOM rebuilds when decoration content is unchanged.
 * - eventHandlers in ViewPlugin spec for chevron clicks — CM6 manages listener cleanup.
 * - Viewport-only processing — O(visible_lines), never O(document).
 * - Synchronous rebuild — regex-only, no syntaxTree, so <1ms per rebuild.
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

// ─── Icon map (type → emoji) ───
// Colors intentionally absent — handled by CSS via --callout-color custom property
const calloutIcons: Record<string, string> = {
	note: 'ℹ️',   info: 'ℹ️',
	tip: '💡',    hint: '💡',    important: '💡',
	success: '✅', check: '✅',  done: '✅',
	question: '❓', help: '❓',  faq: '❓',
	warning: '⚠️', caution: '⚠️', attention: '⚠️',
	failure: '❌', fail: '❌',   missing: '❌',
	danger: '⛔',  error: '⛔',
	bug: '🐛',
	example: '📝',
	quote: '💬',   cite: '💬',
	abstract: '📋', summary: '📋', tldr: '📋',
};

// ─── Collapse/Expand state ───
export const toggleCallout = StateEffect.define<number>(); // line number to toggle

export const calloutCollapseField = StateField.define<Set<number>>({
	create: () => new Set(),
	update(collapsed, tr) {
		let next = collapsed;
		if (tr.docChanged) {
			const newSet = new Set<number>();
			const doc = tr.state.doc;
			for (const oldLine of collapsed) {
				try {
					const oldPos = tr.startState.doc.line(oldLine).from;
					const newPos = tr.changes.mapPos(oldPos);
					newSet.add(doc.lineAt(newPos).number);
				} catch {
					// line no longer exists — discard
				}
			}
			next = newSet;
		}
		for (const effect of tr.effects) {
			if (effect.is(toggleCallout)) {
				next = new Set(next);
				if (next.has(effect.value)) next.delete(effect.value);
				else next.add(effect.value);
			}
		}
		return next;
	},
});

// ─── Widget: replaces "> [!type]+/- " prefix with icon + optional chevron ───
class CalloutIconWidget extends WidgetType {
	constructor(
		readonly type: string,
		readonly icon: string,
		readonly foldable: boolean,
		readonly collapsed: boolean,
		readonly lineNum: number,
	) { super(); }

	toDOM() {
		// dir="auto" — correct icon/chevron placement in both LTR and RTL notes
		const span = document.createElement('span');
		span.className = 'cm-callout-icon';
		span.setAttribute('dir', 'auto');
		span.setAttribute('data-callout', this.type); // CSS hook for --callout-color

		span.appendChild(document.createTextNode(`${this.icon} `));

		if (this.foldable) {
			const chevron = document.createElement('span');
			chevron.className = 'cm-callout-chevron';
			chevron.textContent = this.collapsed ? '▶' : '▼';
			chevron.dataset.calloutLine = String(this.lineNum);
			span.appendChild(chevron);
		}

		return span;
	}

	// eq() prevents CM6 from destroying and recreating the DOM node when nothing changed
	eq(other: CalloutIconWidget) {
		return (
			this.type === other.type &&
			this.icon === other.icon &&
			this.foldable === other.foldable &&
			this.collapsed === other.collapsed &&
			this.lineNum === other.lineNum
		);
	}

	ignoreEvent() { return false; }
}

// ─── Callout block data ───
interface CalloutBlock {
	type: string;
	foldMarker: string; // '+' | '-' | ''
	startLine: number;
	endLine: number;
}

/** Scan lines [fromLine, toLine] for callout headers. Cap body scan at startLine+200. */
function findCalloutsInRange(
	doc: EditorView['state']['doc'],
	fromLine: number,
	toLine: number,
): CalloutBlock[] {
	const callouts: CalloutBlock[] = [];
	let lineNum = fromLine;

	while (lineNum <= toLine) {
		const text = doc.line(lineNum).text;
		const match = text.match(/^>\s*\[!(\w+)\]([+-])?\s*/);

		if (match) {
			const type = match[1].toLowerCase();
			const foldMarker = match[2] || '';

			const scanLimit = Math.min(doc.lines, lineNum + 200);
			let lastLine = lineNum;
			for (let l = lineNum + 1; l <= scanLimit; l++) {
				const next = doc.line(l).text;
				if (!/^>\s?/.test(next) || /^>\s*\[!\w+\]/.test(next)) break;
				lastLine = l;
			}

			callouts.push({ type, foldMarker, startLine: lineNum, endLine: lastLine });
			lineNum = lastLine + 1;
		} else {
			lineNum++;
		}
	}

	return callouts;
}

function buildCalloutDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const collapsed = view.state.field(calloutCollapseField);
	const all: { from: number; to: number; deco: Decoration }[] = [];

	for (const { from, to } of view.visibleRanges) {
		// Look slightly above the viewport to catch callouts that start just off-screen
		const startLine = Math.max(1, doc.lineAt(from).number - 5);
		const endLine = doc.lineAt(to).number;

		for (const callout of findCalloutsInRange(doc, startLine, endLine)) {
			const icon = calloutIcons[callout.type] || 'ℹ️';
			const cursorInCallout = cursorLine >= callout.startLine && cursorLine <= callout.endLine;

			// Obsidian standard: +/- markers control foldability and initial state
			const foldable = callout.foldMarker === '-' || callout.foldMarker === '+';
			const defaultCollapsed = callout.foldMarker === '-';
			// Toggle flips the default
			const isCollapsed = collapsed.has(callout.startLine) ? !defaultCollapsed : defaultCollapsed;

			const titleLine = doc.line(callout.startLine);

			// ── Title line: border + tinted background ──
			// data-callout sets --callout-color via CSS; no color values in JS
			all.push({
				from: titleLine.from, to: titleLine.from,
				deco: Decoration.line({
					class: 'cm-callout-line cm-callout-title-line',
					attributes: { 'data-callout': callout.type },
				}),
			});

			// Widget: hide the raw "> [!type]+/- " prefix, show icon + chevron
			// Show when cursor is outside the callout, or when collapsed (so user can re-expand)
			const showWidget = !cursorInCallout || isCollapsed;

			if (!isCollapsed) {
				// Body lines: lighter tint + border
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					const line = doc.line(l);
					all.push({
						from: line.from, to: line.from,
						deco: Decoration.line({
							class: 'cm-callout-line cm-callout-body-line',
							attributes: { 'data-callout': callout.type },
						}),
					});
				}
			}

			if (showWidget) {
				const prefixMatch = titleLine.text.match(/^>\s*\[!\w+\][+-]?\s*/);
				if (prefixMatch) {
					all.push({
						from: titleLine.from,
						to: titleLine.from + prefixMatch[0].length,
						deco: Decoration.replace({
							widget: new CalloutIconWidget(callout.type, icon, foldable, isCollapsed, callout.startLine),
						}),
					});
				}

				if (isCollapsed && callout.endLine > callout.startLine) {
					// Collapse: hide all body lines
					all.push({
						from: titleLine.to,
						to: doc.line(callout.endLine).to,
						deco: Decoration.replace({}),
					});
				} else {
					// Expanded: hide "> " prefix on each body line
					for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
						const line = doc.line(l);
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
			}
		}
	}

	// RangeSetBuilder requires ascending order; line decos (from===to) before inline at same pos
	all.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const r of all) builder.add(r.from, r.to, r.deco);
	return builder.finish();
}

class CalloutPluginClass {
	decorations: DecorationSet;
	private lastCursorLine = -1;

	constructor(view: EditorView) {
		this.decorations = buildCalloutDecorations(view);
		this.lastCursorLine = view.state.doc.lineAt(view.state.selection.main.head).number;
	}

	update(update: ViewUpdate) {
		const hasToggle = update.transactions.some(t => t.effects.some(e => e.is(toggleCallout)));

		if (hasToggle || update.docChanged || update.viewportChanged) {
			this.decorations = buildCalloutDecorations(update.view);
			this.lastCursorLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			return;
		}

		if (update.selectionSet) {
			const newLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			if (newLine !== this.lastCursorLine) {
				this.lastCursorLine = newLine;
				this.decorations = buildCalloutDecorations(update.view);
			}
		}
	}
}

export const calloutPlugin = ViewPlugin.fromClass(CalloutPluginClass, {
	decorations: (v) => v.decorations,
	// eventHandlers in spec — CM6 manages listener cleanup automatically (no memory leaks)
	eventHandlers: {
		mousedown(event, view) {
			const target = event.target as HTMLElement;
			const chevron = target.closest?.('.cm-callout-chevron') as HTMLElement | null;
			if (chevron?.dataset.calloutLine) {
				event.preventDefault();
				event.stopImmediatePropagation();
				const lineNum = parseInt(chevron.dataset.calloutLine, 10);
				if (!isNaN(lineNum)) view.dispatch({ effects: toggleCallout.of(lineNum) });
				return true;
			}
			return false;
		},
	},
});

// Backward compat export (CodeMirrorEditor.svelte imports this)
export const calloutClickHandler = EditorView.domEventHandlers({});

export const calloutTheme = EditorView.theme({
	// ── Base callout line (title + body) ──
	// --callout-color is set per-type below; CSS cascade handles all visual treatment.
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

	// ── Widget styles ──
	'.cm-callout-icon': {
		fontWeight: '600',
		fontSize: '0.95em',
		verticalAlign: 'middle',
		color: 'var(--callout-color, #448aff)',
	},
	'.cm-callout-chevron': {
		cursor: 'pointer',
		fontSize: '0.7em',
		opacity: '0.6',
		display: 'inline-block',
		marginInlineEnd: '4px',
		userSelect: 'none',
		verticalAlign: 'middle',
	},

	// ── Per-type color variables ──
	// Any unknown type falls back to --callout-color: #448aff (note/info blue)
	'[data-callout="note"]':     { '--callout-color': '#448aff' },
	'[data-callout="info"]':     { '--callout-color': '#448aff' },
	'[data-callout="abstract"]': { '--callout-color': '#00b0ff' },
	'[data-callout="summary"]':  { '--callout-color': '#00b0ff' },
	'[data-callout="tldr"]':     { '--callout-color': '#00b0ff' },
	'[data-callout="tip"]':      { '--callout-color': '#00bfa5' },
	'[data-callout="hint"]':     { '--callout-color': '#00bfa5' },
	'[data-callout="important"]':{ '--callout-color': '#00bfa5' },
	'[data-callout="success"]':  { '--callout-color': '#00c853' },
	'[data-callout="check"]':    { '--callout-color': '#00c853' },
	'[data-callout="done"]':     { '--callout-color': '#00c853' },
	'[data-callout="question"]': { '--callout-color': '#ff9100' },
	'[data-callout="help"]':     { '--callout-color': '#ff9100' },
	'[data-callout="faq"]':      { '--callout-color': '#ff9100' },
	'[data-callout="warning"]':  { '--callout-color': '#ff9100' },
	'[data-callout="caution"]':  { '--callout-color': '#ff9100' },
	'[data-callout="attention"]':{ '--callout-color': '#ff9100' },
	'[data-callout="failure"]':  { '--callout-color': '#ff5252' },
	'[data-callout="fail"]':     { '--callout-color': '#ff5252' },
	'[data-callout="missing"]':  { '--callout-color': '#ff5252' },
	'[data-callout="danger"]':   { '--callout-color': '#ff1744' },
	'[data-callout="error"]':    { '--callout-color': '#ff1744' },
	'[data-callout="bug"]':      { '--callout-color': '#ff1744' },
	'[data-callout="example"]':  { '--callout-color': '#7c4dff' },
	'[data-callout="quote"]':    { '--callout-color': '#9e9e9e' },
	'[data-callout="cite"]':     { '--callout-color': '#9e9e9e' },
});

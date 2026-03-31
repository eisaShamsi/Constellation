/**
 * Callout Plugin — CodeMirror ViewPlugin that renders Obsidian-style callouts
 * (> [!type] title) with colored borders, backgrounds, styled titles,
 * and collapse/expand support (> [!type]- for collapsed, > [!type]+ for expanded).
 *
 * Foldable: only explicit +/- markers (Obsidian standard).
 * Performance: visible ranges only, synchronous rebuild, no timers, no rAF, no dispatch.
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

// ─── Callout type → color mapping ───
const calloutColors: Record<string, string> = {
	note: '#448aff', info: '#448aff',
	tip: '#00bfa5', hint: '#00bfa5', important: '#00bfa5',
	success: '#00c853', check: '#00c853', done: '#00c853',
	question: '#ff9100', help: '#ff9100', faq: '#ff9100',
	warning: '#ff9100', caution: '#ff9100', attention: '#ff9100',
	failure: '#ff5252', fail: '#ff5252', missing: '#ff5252',
	danger: '#ff1744', error: '#ff1744',
	bug: '#ff1744',
	example: '#7c4dff',
	quote: '#9e9e9e', cite: '#9e9e9e',
	abstract: '#00b0ff', summary: '#00b0ff', tldr: '#00b0ff',
};

const calloutIcons: Record<string, string> = {
	note: 'ℹ️', info: 'ℹ️',
	tip: '💡', hint: '💡', important: '💡',
	success: '✅', check: '✅', done: '✅',
	question: '❓', help: '❓', faq: '❓',
	warning: '⚠️', caution: '⚠️', attention: '⚠️',
	failure: '❌', fail: '❌', missing: '❌',
	danger: '⛔', error: '⛔',
	bug: '🐛',
	example: '📝',
	quote: '💬', cite: '💬',
	abstract: '📋', summary: '📋', tldr: '📋',
};

// ─── Collapse/Expand state ───
export const toggleCallout = StateEffect.define<number>(); // line number to toggle

// Tracks which callout start lines are collapsed (by line number)
const calloutCollapseField = StateField.define<Set<number>>({
	create: () => new Set(),
	update(collapsed, tr) {
		let next = collapsed;
		if (tr.docChanged) {
			const newSet = new Set<number>();
			const doc = tr.state.doc;
			for (const oldLine of collapsed) {
				try {
					const oldDoc = tr.startState.doc;
					if (oldLine <= oldDoc.lines) {
						const oldPos = oldDoc.line(oldLine).from;
						const newPos = tr.changes.mapPos(oldPos);
						const newLine = doc.lineAt(newPos).number;
						newSet.add(newLine);
					}
				} catch {
					// Line no longer exists — drop it
				}
			}
			next = newSet;
		}
		for (const effect of tr.effects) {
			if (effect.is(toggleCallout)) {
				next = new Set(next);
				if (next.has(effect.value)) {
					next.delete(effect.value);
				} else {
					next.add(effect.value);
				}
			}
		}
		return next;
	},
});

/** Widget that replaces the "> [!type]+/- " prefix with an icon + optional chevron */
class CalloutIconWidget extends WidgetType {
	constructor(
		readonly type: string,
		readonly color: string,
		readonly icon: string,
		readonly foldable: boolean,
		readonly collapsed: boolean,
		readonly lineNum: number,
	) { super(); }

	toDOM() {
		const span = document.createElement('span');
		span.className = 'cm-callout-icon';
		span.style.color = this.color;
		span.appendChild(document.createTextNode(`${this.icon} `));

		if (this.foldable) {
			const chevron = document.createElement('span');
			chevron.className = 'cm-callout-chevron';
			chevron.textContent = this.collapsed ? '▶' : '▼';
			chevron.style.cursor = 'pointer';
			chevron.style.fontSize = '0.7em';
			chevron.style.opacity = '0.6';
			chevron.style.display = 'inline-block';
			chevron.style.marginInlineEnd = '4px';
			chevron.dataset.calloutLine = String(this.lineNum);
			span.appendChild(chevron);
		}

		return span;
	}

	eq(other: CalloutIconWidget) {
		return (
			this.type === other.type &&
			this.foldable === other.foldable &&
			this.collapsed === other.collapsed &&
			this.lineNum === other.lineNum
		);
	}

	ignoreEvent() { return false; }
}

interface CalloutBlock {
	type: string;
	title: string;
	foldMarker: string; // '+', '-', or ''
	startLine: number;
	endLine: number;
}

/** Find callout blocks within a line range. Cap body scan at startLine+200. */
function findCalloutsInRange(
	doc: EditorView['state']['doc'],
	fromLine: number,
	toLine: number,
): CalloutBlock[] {
	const callouts: CalloutBlock[] = [];
	let lineNum = fromLine;

	while (lineNum <= toLine) {
		const line = doc.line(lineNum);
		const match = line.text.match(/^>\s*\[!(\w+)\]([+-])?\s*(.*)?$/);

		if (match) {
			const type = match[1].toLowerCase();
			const foldMarker = match[2] || '';
			const title = match[3]?.trim() || '';

			// Scan forward for body lines — cap at startLine+200 to avoid O(N) on huge docs
			const scanLimit = Math.min(doc.lines, lineNum + 200);
			let lastLine = lineNum;
			for (let l = lineNum + 1; l <= scanLimit; l++) {
				const nextText = doc.line(l).text;
				if (!/^>\s?/.test(nextText) || /^>\s*\[!\w+\]/.test(nextText)) break;
				lastLine = l;
			}

			callouts.push({ type, title, foldMarker, startLine: lineNum, endLine: lastLine });
			lineNum = lastLine + 1;
			continue;
		}
		lineNum++;
	}

	return callouts;
}

function buildCalloutDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const collapsed = view.state.field(calloutCollapseField);

	const all: { from: number; to: number; deco: Decoration }[] = [];

	for (const { from, to } of view.visibleRanges) {
		// Expand the search range slightly to catch callouts that start just above viewport
		const startLine = Math.max(1, doc.lineAt(from).number - 5);
		const endLine = doc.lineAt(to).number;

		const callouts = findCalloutsInRange(doc, startLine, endLine);

		for (const callout of callouts) {
			const color = calloutColors[callout.type] || '#448aff';
			const icon = calloutIcons[callout.type] || 'ℹ️';
			const cursorInCallout = cursorLine >= callout.startLine && cursorLine <= callout.endLine;

			// Obsidian standard: only explicit +/- markers make a callout foldable
			const foldable = callout.foldMarker === '-' || callout.foldMarker === '+';
			const defaultCollapsed = callout.foldMarker === '-';
			// Toggle flips the default: if default=collapsed, toggle=expanded, and vice versa
			const isCollapsed = collapsed.has(callout.startLine) ? !defaultCollapsed : defaultCollapsed;

			const titleLine = doc.line(callout.startLine);

			// Title line — always gets border + background
			all.push({
				from: titleLine.from, to: titleLine.from,
				deco: Decoration.line({
					class: 'cm-callout-line cm-callout-title-line',
					attributes: {
						style: `border-inline-start: 3px solid ${color}; padding-inline-start: 12px; background: color-mix(in srgb, ${color} 8%, transparent);`,
					},
				}),
			});

			// Widget: show icon + chevron when cursor is outside the callout, OR when collapsed
			// (collapsed always shows widget so user can re-expand)
			const showWidget = !cursorInCallout || isCollapsed;

			if (!isCollapsed) {
				// Content lines — border + lighter background
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					const line = doc.line(l);
					all.push({
						from: line.from, to: line.from,
						deco: Decoration.line({
							class: 'cm-callout-line',
							attributes: {
								style: `border-inline-start: 3px solid ${color}; padding-inline-start: 12px; background: color-mix(in srgb, ${color} 4%, transparent);`,
							},
						}),
					});
				}
			}

			if (showWidget) {
				// Replace "> [!type]+/- " prefix with icon + optional chevron
				const prefixMatch = titleLine.text.match(/^>\s*\[!\w+\][+-]?\s*/);
				if (prefixMatch) {
					const prefixEnd = titleLine.from + prefixMatch[0].length;
					all.push({
						from: titleLine.from, to: prefixEnd,
						deco: Decoration.replace({
							widget: new CalloutIconWidget(callout.type, color, icon, foldable, isCollapsed, callout.startLine),
						}),
					});
				}

				if (isCollapsed && callout.endLine > callout.startLine) {
					// Hide all content lines when collapsed
					const lastContentLine = doc.line(callout.endLine);
					all.push({
						from: titleLine.to, to: lastContentLine.to,
						deco: Decoration.replace({}),
					});
				} else {
					// Show content but hide > markers on each content line
					for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
						const line = doc.line(l);
						const qm = line.text.match(/^>\s?/);
						if (qm) {
							all.push({
								from: line.from, to: line.from + qm[0].length,
								deco: Decoration.replace({}),
							});
						}
					}
				}
			}
		}
	}

	// Sort: by position, line decos (from===to) before inline at same position
	all.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const r of all) {
		builder.add(r.from, r.to, r.deco);
	}
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

		// calloutPlugin is regex-only (no syntaxTree) so sync rebuild is fast (<1ms).
		// No debounce needed — callout visuals must appear immediately when user types them.
		if (hasToggle || update.docChanged || update.viewportChanged) {
			this.decorations = buildCalloutDecorations(update.view);
			this.lastCursorLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			return;
		}

		if (update.selectionSet) {
			const newCursorLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			// Rebuild only when cursor crosses a line boundary — avoids redundant rebuilds
			// on same-line cursor movements (word navigation, char-by-char within a line)
			if (newCursorLine !== this.lastCursorLine) {
				this.lastCursorLine = newCursorLine;
				this.decorations = buildCalloutDecorations(update.view);
			}
		}
	}
}

export const calloutPlugin = ViewPlugin.fromClass(CalloutPluginClass, {
	decorations: (v) => v.decorations,
	eventHandlers: {
		mousedown(event, view) {
			const target = event.target as HTMLElement;
			const chevron = target.closest?.('.cm-callout-chevron') as HTMLElement | null;
			if (chevron && chevron.dataset.calloutLine) {
				event.preventDefault();
				event.stopImmediatePropagation();
				const lineNum = parseInt(chevron.dataset.calloutLine, 10);
				if (!isNaN(lineNum)) {
					view.dispatch({ effects: toggleCallout.of(lineNum) });
				}
				return true;
			}
			return false;
		},
	},
});

export { calloutCollapseField };

// Keep export for backward compat
export const calloutClickHandler = EditorView.domEventHandlers({});

export const calloutTheme = EditorView.theme({
	'.cm-callout-line': {
		borderRadius: '0',
	},
	'.cm-callout-title-line': {
		fontWeight: '500',
	},
	'.cm-callout-icon': {
		fontWeight: '600',
		fontSize: '0.95em',
		display: 'inline',
		verticalAlign: 'middle',
	},
	'.cm-callout-chevron': {
		userSelect: 'none',
	},
});

/**
 * Callout Plugin — CodeMirror ViewPlugin that renders Obsidian-style callouts
 * (> [!type] title) with colored borders, backgrounds, styled titles,
 * and collapse/expand support (> [!type]- for collapsed, > [!type]+ for expanded).
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
const toggleCallout = StateEffect.define<number>(); // line number to toggle

// Tracks which callout start lines are collapsed (by line number)
const calloutCollapseField = StateField.define<Set<number>>({
	create: () => new Set(),
	update(collapsed, tr) {
		let next = collapsed;
		// If doc changed, remap line numbers (simplified: clear on doc change)
		if (tr.docChanged) {
			// Remap positions through changes
			const newSet = new Set<number>();
			const doc = tr.state.doc;
			for (const oldLine of collapsed) {
				try {
					// Map the start of the old line through changes
					const oldDoc = tr.startState.doc;
					if (oldLine <= oldDoc.lines) {
						const oldPos = oldDoc.line(oldLine).from;
						const newPos = tr.changes.mapPos(oldPos);
						const newLine = doc.lineAt(newPos).number;
						newSet.add(newLine);
					}
				} catch {
					// Line no longer exists, drop it
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

/** Widget that replaces the > [!type] title line with styled title + chevron */
class CalloutTitleWidget extends WidgetType {
	type: string;
	title: string;
	color: string;
	icon: string;
	foldable: boolean;
	collapsed: boolean;
	lineNum: number;

	constructor(type: string, title: string, foldable: boolean, collapsed: boolean, lineNum: number) {
		super();
		this.type = type;
		this.title = title;
		this.color = calloutColors[type] || '#448aff';
		this.icon = calloutIcons[type] || 'ℹ️';
		this.foldable = foldable;
		this.collapsed = collapsed;
		this.lineNum = lineNum;
	}

	toDOM(view: EditorView) {
		const span = document.createElement('span');
		span.className = 'cm-callout-title';
		span.style.color = this.color;

		const titleText = this.title || this.type.charAt(0).toUpperCase() + this.type.slice(1);

		if (this.foldable) {
			const chevron = document.createElement('span');
			chevron.className = 'cm-callout-chevron';
			chevron.textContent = this.collapsed ? '▶' : '▼';
			chevron.style.cursor = 'pointer';
			chevron.style.fontSize = '0.7em';
			chevron.style.opacity = '0.6';
			chevron.style.transition = 'transform 0.15s ease';
			chevron.style.display = 'inline-block';
			chevron.style.marginInlineEnd = '4px';

			chevron.dataset.calloutLine = String(this.lineNum);

			span.appendChild(document.createTextNode(`${this.icon}  `));
			span.appendChild(document.createTextNode(titleText));
			span.appendChild(document.createTextNode('  '));
			span.appendChild(chevron);
		} else {
			span.textContent = `${this.icon}  ${titleText}`;
		}

		return span;
	}

	eq(other: CalloutTitleWidget) {
		return this.type === other.type && this.title === other.title
			&& this.foldable === other.foldable && this.collapsed === other.collapsed;
	}
}

/** Detect callout blocks */
interface CalloutBlock {
	type: string;
	title: string;
	foldMarker: string; // '+', '-', or ''
	startLine: number;
	endLine: number;
}

function findCallouts(view: EditorView): CalloutBlock[] {
	const doc = view.state.doc;
	const callouts: CalloutBlock[] = [];

	for (const { from, to } of view.visibleRanges) {
		let lineNum = doc.lineAt(from).number;
		const endLineNum = doc.lineAt(to).number;

		while (lineNum <= endLineNum) {
			const line = doc.line(lineNum);
			const match = line.text.match(/^>\s*\[!(\w+)\]([+-])?\s*(.*)?$/);
			if (match) {
				const type = match[1].toLowerCase();
				const foldMarker = match[2] || '';
				const title = match[3]?.trim() || '';

				let lastLine = lineNum;
				for (let l = lineNum + 1; l <= doc.lines; l++) {
					const nextText = doc.line(l).text;
					// Stop if: not a > line, or it's a new callout start
					if (!/^>\s?/.test(nextText) || /^>\s*\[!\w+\]/.test(nextText)) {
						break;
					}
					lastLine = l;
				}

				callouts.push({ type, title, foldMarker, startLine: lineNum, endLine: lastLine });
				lineNum = lastLine + 1;
				continue;
			}
			lineNum++;
		}
	}

	return callouts;
}

function buildCalloutDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const callouts = findCallouts(view);
	const collapsed = view.state.field(calloutCollapseField);

	const all: { from: number; to: number; deco: Decoration }[] = [];

	for (const callout of callouts) {
		const color = calloutColors[callout.type] || '#448aff';
		const cursorInCallout = cursorLine >= callout.startLine && cursorLine <= callout.endLine;

		const foldable = callout.foldMarker === '-' || callout.foldMarker === '+';
		// Default: '-' starts collapsed, '+' starts expanded. User toggle flips it.
		let isCollapsed: boolean;
		if (collapsed.has(callout.startLine)) {
			isCollapsed = callout.foldMarker !== '-'; // toggled: flip the default
		} else {
			isCollapsed = callout.foldMarker === '-'; // default from marker
		}

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

		// When collapsed: ALWAYS show widget (even when cursor is on title line)
		// so the user can click the chevron to expand
		const showWidget = isCollapsed || !cursorInCallout;

		// Content lines — border + lighter background (only if expanded)
		if (!isCollapsed) {
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
			// Replace title line with widget
			all.push({
				from: titleLine.from, to: titleLine.to,
				deco: Decoration.replace({
					widget: new CalloutTitleWidget(callout.type, callout.title, foldable, isCollapsed, callout.startLine),
				}),
			});

			if (isCollapsed && callout.endLine > callout.startLine) {
				// Hide all content lines when collapsed
				const lastContentLine = doc.line(callout.endLine);
				all.push({
					from: titleLine.to, to: lastContentLine.to,
					deco: Decoration.replace({}),
				});
			} else if (!cursorInCallout) {
				// Show content but hide > markers
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

	// Sort by position — line decos (from===to) come before inline at same position
	all.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const r of all) {
		builder.add(r.from, r.to, r.deco);
	}
	return builder.finish();
}

class CalloutPluginClass {
	decorations: DecorationSet;

	rebuildTimer: ReturnType<typeof setTimeout> | null = null;

	constructor(view: EditorView) {
		this.decorations = buildCalloutDecorations(view);
	}

	update(update: ViewUpdate) {
		// Toggle effect or viewport change — rebuild immediately
		if (update.transactions.some(t => t.effects.some(e => e.is(toggleCallout)))
			|| update.viewportChanged
			|| (update.selectionSet && !update.docChanged)) {
			this.decorations = buildCalloutDecorations(update.view);
			return;
		}
		if (update.docChanged) {
			this.decorations = this.decorations.map(update.changes);
			if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
			const view = update.view;
			this.rebuildTimer = setTimeout(() => {
				this.rebuildTimer = null;
				requestAnimationFrame(() => {
					if (!view.destroyed) {
						this.decorations = buildCalloutDecorations(view);
					}
				});
			}, 350);
		}
	}

	destroy() {
		if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
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
					setTimeout(() => {
						if (!view.destroyed) {
							view.dispatch({ effects: toggleCallout.of(lineNum) });
						}
					}, 0);
				}
				return true;
			}
			return false;
		},
	},
});

export { calloutCollapseField };

// Keep export for backward compat but it's now handled inside the plugin
export const calloutClickHandler = EditorView.domEventHandlers({});

export const calloutTheme = EditorView.theme({
	'.cm-callout-line': {
		borderRadius: '0',
	},
	'.cm-callout-title-line': {
		fontWeight: '500',
	},
	'.cm-callout-title': {
		fontWeight: '600',
		fontSize: '0.95em',
		display: 'inline-flex',
		alignItems: 'center',
		gap: '4px',
	},
	'.cm-callout-chevron': {
		userSelect: 'none',
	},
});

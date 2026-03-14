/**
 * Live Preview — CodeMirror ViewPlugin that renders markdown inline.
 * Hides syntax characters when cursor is not on the same line,
 * and styles content (headings, bold, italic, etc.) directly in the editor.
 */
import {
	ViewPlugin,
	type ViewUpdate,
	Decoration,
	type DecorationSet,
	EditorView,
	WidgetType,
} from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';
import { RangeSetBuilder } from '@codemirror/state';

// CSS classes for live preview decorations
const headingClasses = [
	'cm-md-heading1',
	'cm-md-heading2',
	'cm-md-heading3',
	'cm-md-heading4',
	'cm-md-heading5',
	'cm-md-heading6',
];

const boldDeco = Decoration.mark({ class: 'cm-md-bold' });
const italicDeco = Decoration.mark({ class: 'cm-md-italic' });
const strikeDeco = Decoration.mark({ class: 'cm-md-strikethrough' });
const codeDeco = Decoration.mark({ class: 'cm-md-code' });
const linkDeco = Decoration.mark({ class: 'cm-md-link' });
const highlightDeco = Decoration.mark({ class: 'cm-md-highlight' });
const hrDeco = Decoration.mark({ class: 'cm-md-hr' });
const blockquoteDeco = Decoration.mark({ class: 'cm-md-blockquote' });

class CheckboxWidget extends WidgetType {
	checked: boolean;
	constructor(checked: boolean) {
		super();
		this.checked = checked;
	}
	toDOM() {
		const cb = document.createElement('input');
		cb.type = 'checkbox';
		cb.checked = this.checked;
		cb.className = 'cm-md-checkbox';
		cb.setAttribute('aria-label', this.checked ? 'Completed' : 'Todo');
		return cb;
	}
	eq(other: CheckboxWidget) { return this.checked === other.checked; }
}

/** Widget shown for dataview code blocks when cursor is outside */
class DataviewLabelWidget extends WidgetType {
	query: string;
	constructor(query: string) {
		super();
		this.query = query;
	}
	toDOM() {
		const wrap = document.createElement('div');
		wrap.className = 'cm-dv-label-widget';
		const badge = document.createElement('span');
		badge.className = 'cm-dv-badge';
		badge.textContent = 'Dataview';
		wrap.appendChild(badge);
		const preview = document.createElement('code');
		preview.className = 'cm-dv-query-preview';
		preview.textContent = this.query.length > 80 ? this.query.slice(0, 80) + '…' : this.query;
		wrap.appendChild(preview);
		return wrap;
	}
	eq(other: DataviewLabelWidget) { return this.query === other.query; }
}

function buildDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const ranges: { from: number; to: number; deco: Decoration }[] = [];

	// Process only visible ranges for performance
	for (const { from, to } of view.visibleRanges) {
		syntaxTree(view.state).iterate({
			from, to,
			enter(node) {
				const nodeLine = doc.lineAt(node.from).number;
				const nodeEndLine = doc.lineAt(node.to).number;
				const onCursorLine = nodeLine === cursorLine;
				const cursorInBlock = cursorLine >= nodeLine && cursorLine <= nodeEndLine;

				// ATX Headings (# through ######)
				if (node.name.startsWith('ATXHeading') && node.name.length === 11) {
					const level = parseInt(node.name[10]) - 1;
					if (level >= 0 && level < 6) {
						ranges.push({ from: node.from, to: node.to, deco: Decoration.mark({ class: headingClasses[level] }) });
					}
				}

				// Hide heading markers (# characters) when cursor is not on that line
				if (node.name === 'HeaderMark' && !onCursorLine) {
					const end = Math.min(node.to + 1, doc.lineAt(node.from).to);
					ranges.push({ from: node.from, to: end, deco: Decoration.replace({}) });
				}

				// Strong emphasis (bold)
				if (node.name === 'StrongEmphasis') {
					ranges.push({ from: node.from, to: node.to, deco: boldDeco });
				}

				// Emphasis (italic)
				if (node.name === 'Emphasis') {
					ranges.push({ from: node.from, to: node.to, deco: italicDeco });
				}

				// Hide emphasis markers when not on cursor line
				if (node.name === 'EmphasisMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({}) });
				}

				// Strikethrough
				if (node.name === 'Strikethrough') {
					ranges.push({ from: node.from, to: node.to, deco: strikeDeco });
				}
				if (node.name === 'StrikethroughMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({}) });
				}

				// Inline code
				if (node.name === 'InlineCode') {
					ranges.push({ from: node.from, to: node.to, deco: codeDeco });
				}
				if (node.name === 'CodeMark' && !onCursorLine) {
					const text = doc.sliceString(node.from, node.to);
					if (text === '`') {
						ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({}) });
					}
				}

				// Links
				if (node.name === 'Link' || node.name === 'URL') {
					ranges.push({ from: node.from, to: node.to, deco: linkDeco });
				}

				// Blockquote markers
				if (node.name === 'QuoteMark') {
					ranges.push({ from: node.from, to: node.to, deco: blockquoteDeco });
				}

				// Horizontal rule
				if (node.name === 'HorizontalRule') {
					ranges.push({ from: node.from, to: node.to, deco: hrDeco });
				}

				// Task list checkboxes
				if (node.name === 'TaskMarker') {
					const text = doc.sliceString(node.from, node.to);
					const checked = text.includes('x') || text.includes('X');
					if (!onCursorLine) {
						ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
							widget: new CheckboxWidget(checked),
						}) });
					}
				}

				// Dataview fenced code blocks — show label widget when cursor is outside
				if (node.name === 'FencedCode' && !cursorInBlock) {
					const firstLine = doc.lineAt(node.from);
					const info = firstLine.text.trim();
					if (/^```+\s*dataview\s*$/i.test(info)) {
						// Extract the query text (between opening and closing fence)
						const innerFrom = firstLine.to + 1;
						const lastLine = doc.lineAt(node.to);
						const innerTo = lastLine.text.trim().startsWith('```') ? lastLine.from : node.to;
						const queryText = innerTo > innerFrom ? doc.sliceString(innerFrom, innerTo).trim() : '';
						ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
							widget: new DataviewLabelWidget(queryText),
						}) });
					}
				}
			}
		});
	}

	// Sort by from position, then by length (shorter ranges first for proper nesting)
	ranges.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const r of ranges) {
		builder.add(r.from, r.to, r.deco);
	}
	return builder.finish();
}

// The ViewPlugin class
class LivePreviewPlugin {
	decorations: DecorationSet;

	constructor(view: EditorView) {
		this.decorations = buildDecorations(view);
	}

	update(update: ViewUpdate) {
		if (update.docChanged || update.selectionSet || update.viewportChanged) {
			this.decorations = buildDecorations(update.view);
		}
	}
}

/** The live preview extension — add to CM extensions to enable */
export const livePreviewPlugin = ViewPlugin.fromClass(LivePreviewPlugin, {
	decorations: (v) => v.decorations,
});

/** Theme for live preview decorations */
export const livePreviewTheme = EditorView.theme({
	'.cm-md-heading1': {
		fontSize: '1.8em',
		fontWeight: '700',
		lineHeight: '1.3',
	},
	'.cm-md-heading2': {
		fontSize: '1.5em',
		fontWeight: '700',
		lineHeight: '1.3',
	},
	'.cm-md-heading3': {
		fontSize: '1.25em',
		fontWeight: '700',
		lineHeight: '1.3',
	},
	'.cm-md-heading4': {
		fontSize: '1.1em',
		fontWeight: '700',
		lineHeight: '1.3',
	},
	'.cm-md-heading5': {
		fontSize: '1.0em',
		fontWeight: '700',
		lineHeight: '1.3',
	},
	'.cm-md-heading6': {
		fontSize: '0.95em',
		fontWeight: '700',
		color: 'var(--text-muted)',
		lineHeight: '1.3',
	},
	'.cm-md-bold': {
		fontWeight: '700',
	},
	'.cm-md-italic': {
		fontStyle: 'italic',
	},
	'.cm-md-strikethrough': {
		textDecoration: 'line-through',
		opacity: '0.6',
	},
	'.cm-md-code': {
		fontFamily: 'var(--font-monospace-theme)',
		backgroundColor: 'var(--background-primary-alt)',
		borderRadius: '3px',
		padding: '1px 4px',
	},
	'.cm-md-link': {
		color: 'var(--vault-accent, var(--interactive-accent))',
		textDecoration: 'underline',
		textDecorationColor: 'color-mix(in srgb, var(--vault-accent, var(--interactive-accent)) 40%, transparent)',
	},
	'.cm-md-highlight': {
		backgroundColor: 'color-mix(in srgb, var(--color-yellow) 35%, transparent)',
		borderRadius: '2px',
		padding: '1px 0',
	},
	'.cm-md-hr': {
		display: 'block',
		textAlign: 'center',
		color: 'var(--background-modifier-border)',
	},
	'.cm-md-blockquote': {
		color: 'var(--text-muted)',
	},
	'.cm-md-checkbox': {
		verticalAlign: 'middle',
		marginRight: '4px',
		cursor: 'pointer',
		accentColor: 'var(--vault-accent, var(--interactive-accent))',
	},
	'.cm-dv-label-widget': {
		display: 'flex',
		alignItems: 'center',
		gap: '8px',
		padding: '6px 10px',
		margin: '4px 0',
		border: '1px solid var(--background-modifier-border)',
		borderRadius: '6px',
		background: 'var(--background-secondary)',
		cursor: 'pointer',
		userSelect: 'none',
	},
	'.cm-dv-badge': {
		fontSize: '11px',
		fontWeight: '600',
		color: 'var(--interactive-accent)',
		textTransform: 'uppercase',
		letterSpacing: '0.5px',
		flexShrink: '0',
	},
	'.cm-dv-query-preview': {
		fontSize: '11px',
		color: 'var(--text-muted)',
		overflow: 'hidden',
		textOverflow: 'ellipsis',
		whiteSpace: 'nowrap',
		background: 'none',
		padding: '0',
		fontFamily: 'var(--font-monospace-theme)',
	},
});

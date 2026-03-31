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
import { RangeSetBuilder, StateField, StateEffect } from '@codemirror/state';
import { convertFileSrc } from '@tauri-apps/api/core';

// ─── Library path state field (for resolving image embeds) ───
export const setLibraryPath = StateEffect.define<string>();

export const libraryPathField = StateField.define<string>({
	create: () => '',
	update(value, tr) {
		for (const effect of tr.effects) {
			if (effect.is(setLibraryPath)) return effect.value;
		}
		return value;
	},
});

function resolveEmbedImage(view: EditorView, filename: string): string | null {
	const libPath = view.state.field(libraryPathField, false);
	if (!libPath) return null;
	const sep = libPath.includes('\\') ? '\\' : '/';
	const fullPath = libPath + sep + filename;
	try {
		return convertFileSrc(fullPath);
	} catch {
		return null;
	}
}

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
const tagDeco = Decoration.mark({ class: 'cm-md-tag' });
const replaceDeco = Decoration.replace({}); /* cached — avoids allocation per decoration */

// Typed link decorations — one per semantic link type (CE Phase 1)
const TYPED_LINK_TYPES = new Set([
	'supports', 'contradicts', 'causes', 'exemplifies',
	'generalizes', 'derives-from', 'part-of', 'associative',
]);
const typedLinkDecos: Record<string, ReturnType<typeof Decoration.mark>> = {
	supports:       Decoration.mark({ class: 'cm-md-link cm-link-supports' }),
	contradicts:    Decoration.mark({ class: 'cm-md-link cm-link-contradicts' }),
	causes:         Decoration.mark({ class: 'cm-md-link cm-link-causes' }),
	exemplifies:    Decoration.mark({ class: 'cm-md-link cm-link-exemplifies' }),
	generalizes:    Decoration.mark({ class: 'cm-md-link cm-link-generalizes' }),
	'derives-from': Decoration.mark({ class: 'cm-md-link cm-link-derives-from' }),
	'part-of':      Decoration.mark({ class: 'cm-md-link cm-link-part-of' }),
	associative:    linkDeco,
};

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

/** Widget for inline images when cursor is off the line */
class ImageWidget extends WidgetType {
	src: string;
	alt: string;
	constructor(src: string, alt: string) {
		super();
		this.src = src;
		this.alt = alt;
	}
	toDOM() {
		const wrap = document.createElement('div');
		wrap.className = 'cm-md-image-widget';
		const img = document.createElement('img');
		img.src = this.src;
		img.alt = this.alt || '';
		img.loading = 'lazy';
		img.onerror = () => {
			wrap.innerHTML = '';
			const fallback = document.createElement('span');
			fallback.className = 'cm-md-image-fallback';
			fallback.textContent = `📷 ${this.alt || this.src}`;
			wrap.appendChild(fallback);
		};
		wrap.appendChild(img);
		return wrap;
	}
	eq(other: ImageWidget) { return this.src === other.src; }
}

/** Widget for code block language label */
class CodeBlockLabelWidget extends WidgetType {
	lang: string;
	constructor(lang: string) {
		super();
		this.lang = lang;
	}
	toDOM() {
		const badge = document.createElement('span');
		badge.className = 'cm-md-codeblock-lang';
		badge.textContent = this.lang;
		return badge;
	}
	eq(other: CodeBlockLabelWidget) { return this.lang === other.lang; }
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
					ranges.push({ from: node.from, to: end, deco: replaceDeco });
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
					ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
				}

				// Strikethrough
				if (node.name === 'Strikethrough') {
					ranges.push({ from: node.from, to: node.to, deco: strikeDeco });
				}
				if (node.name === 'StrikethroughMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
				}

				// Inline code
				if (node.name === 'InlineCode') {
					ranges.push({ from: node.from, to: node.to, deco: codeDeco });
				}
				if (node.name === 'CodeMark' && !onCursorLine) {
					const text = doc.sliceString(node.from, node.to);
					if (text === '`') {
						ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
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

				// Highlight (==text==)
				if (node.name === 'Highlight') {
					ranges.push({ from: node.from, to: node.to, deco: highlightDeco });
				}
				if (node.name === 'HighlightMark' && !onCursorLine) {
					ranges.push({ from: node.from, to: node.to, deco: replaceDeco });
				}

				// Fenced code blocks
				if (node.name === 'FencedCode') {
					const firstLine = doc.lineAt(node.from);
					const info = firstLine.text.trim();

					if (!cursorInBlock) {
						// Dataview — show label widget
						if (/^```+\s*dataview\s*$/i.test(info)) {
							const innerFrom = firstLine.to + 1;
							const lastLine = doc.lineAt(node.to);
							const innerTo = lastLine.text.trim().startsWith('```') ? lastLine.from : node.to;
							const queryText = innerTo > innerFrom ? doc.sliceString(innerFrom, innerTo).trim() : '';
							ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
								widget: new DataviewLabelWidget(queryText),
							}) });
						}
					}

					// Language label for non-dataview code blocks
					if (!cursorInBlock) {
						const langMatch = info.match(/^```+\s*(\S+)/);
						if (langMatch && !/^dataview$/i.test(langMatch[1])) {
							ranges.push({ from: firstLine.to, to: firstLine.to, deco: Decoration.widget({
								widget: new CodeBlockLabelWidget(langMatch[1]),
								side: 1,
							}) });
						}
					}
				}

				// Inline images: ![[file.png]] or ![alt](url)
				if (node.name === 'Image' && !onCursorLine) {
					const text = doc.sliceString(node.from, node.to);
					// Standard markdown: ![alt](url)
					const mdMatch = text.match(/^!\[([^\]]*)\]\(([^)]+)\)/);
					if (mdMatch) {
						ranges.push({ from: node.from, to: node.to, deco: Decoration.replace({
							widget: new ImageWidget(mdMatch[2], mdMatch[1]),
						}) });
					}
				}
			}
		});
	}

	// Single-pass line scan for wikilink embeds, wikilinks, and tags
	// (Merged from 3 separate loops to reduce iteration overhead)
	const IMG_EXTS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico', 'avif']);
	for (const { from: vFrom, to: vTo } of view.visibleRanges) {
		for (let pos = vFrom; pos < vTo;) {
			const line = doc.lineAt(pos);
			if (line.number !== cursorLine) {
				const lineText = line.text;

				// Image embeds: ![[file.png]]
				const embedRe = /!\[\[([^\]]+)\]\]/g;
				let m;
				while ((m = embedRe.exec(lineText)) !== null) {
					const target = m[1];
					const ext = target.split('.').pop()?.toLowerCase() || '';
					if (IMG_EXTS.has(ext)) {
						const imgSrc = resolveEmbedImage(view, target);
						if (imgSrc) {
							const absFrom = line.from + m.index;
							ranges.push({ from: absFrom, to: absFrom + m[0].length, deco: Decoration.replace({
								widget: new ImageWidget(imgSrc, target),
							}) });
						}
					}
				}

				// Wikilinks: [[note]], [[note|display]], or [[note|link-type]] (CE typed links)
				const wikiRe = /(?<!!)\[\[([^\]]+)\]\]/g;
				while ((m = wikiRe.exec(lineText)) !== null) {
					const absFrom = line.from + m.index;
					const absTo = absFrom + m[0].length;
					const innerFrom = absFrom + 2;
					const innerTo = absTo - 2;
					const pipeIndex = m[1].indexOf('|');
					if (pipeIndex >= 0) {
						const afterPipe = m[1].slice(pipeIndex + 1).trim().toLowerCase();
						if (TYPED_LINK_TYPES.has(afterPipe)) {
							// Typed link: show note name in type color, hide [[, |type, ]]
							const noteEnd = innerFrom + pipeIndex;
							ranges.push({ from: absFrom, to: innerFrom, deco: replaceDeco }); // hide [[
							ranges.push({ from: innerFrom, to: noteEnd, deco: typedLinkDecos[afterPipe] ?? linkDeco }); // note name
							ranges.push({ from: noteEnd, to: absTo, deco: replaceDeco }); // hide |type]]
						} else {
							// Display alias: [[note|alias]] — show alias as link text
							const displayFrom = innerFrom + pipeIndex + 1;
							ranges.push({ from: absFrom, to: displayFrom, deco: replaceDeco });
							ranges.push({ from: displayFrom, to: innerTo, deco: linkDeco });
							ranges.push({ from: innerTo, to: absTo, deco: replaceDeco });
						}
					} else {
						ranges.push({ from: absFrom, to: innerFrom, deco: replaceDeco });
						ranges.push({ from: innerFrom, to: innerTo, deco: linkDeco });
						ranges.push({ from: innerTo, to: absTo, deco: replaceDeco });
					}
				}

				// Tags: #tag-name
				const tagRe = /(?:^|\s)(#[a-zA-Z\u0600-\u06FF][\w\u0600-\u06FF/-]*)/g;
				while ((m = tagRe.exec(lineText)) !== null) {
					const tagStart = line.from + m.index + (m[0].length - m[1].length);
					ranges.push({ from: tagStart, to: tagStart + m[1].length, deco: tagDeco });
				}
			}
			pos = line.to + 1;
		}
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
	private rebuildTimer: ReturnType<typeof setTimeout> | null = null;
	private lastCursorLine = -1;

	constructor(view: EditorView) {
		this.decorations = buildDecorations(view);
		this.lastCursorLine = view.state.doc.lineAt(view.state.selection.main.head).number;
	}

	update(update: ViewUpdate) {
		if (update.viewportChanged) {
			// Scroll — always rebuild, clear any pending debounce
			if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
			this.decorations = buildDecorations(update.view);
			this.lastCursorLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			return;
		}

		if (update.selectionSet && !update.docChanged) {
			// Cursor moved — live preview markers are LINE-based (hide when cursor leaves the line),
			// so rebuilding on same-line cursor moves (word nav, char nav) is redundant and causes
			// perceptible lag. Only rebuild when the cursor crosses a line boundary.
			const newLine = update.view.state.doc.lineAt(update.view.state.selection.main.head).number;
			if (newLine !== this.lastCursorLine) {
				this.lastCursorLine = newLine;
				if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
				this.decorations = buildDecorations(update.view);
			}
			return;
		}

		if (update.docChanged) {
			// ⚡ Fast path: map existing decorations — O(changes), ~0ms, keeps typing instant
			this.decorations = this.decorations.map(update.changes);
			// Debounced full rebuild — no view.dispatch (fresh decos apply on next natural update)
			if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
			const view = update.view;
			this.rebuildTimer = setTimeout(() => {
				this.rebuildTimer = null;
				if (!view.destroyed) {
					this.decorations = buildDecorations(view);
				}
			}, 300);
		}
	}

	destroy() {
		if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
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
		color: 'var(--library-accent, var(--interactive-accent))',
		textDecoration: 'underline',
		textDecorationColor: 'color-mix(in srgb, var(--library-accent, var(--interactive-accent)) 40%, transparent)',
	},
	// CE Phase 1 — Typed Link colors (underline tint matches GraphMind + BacklinksPanel badges)
	'.cm-link-supports':     { color: '#4A9EFF', textDecorationColor: '#4A9EFF66' },
	'.cm-link-contradicts':  { color: '#FF4A4A', textDecorationColor: '#FF4A4A66' },
	'.cm-link-causes':       { color: '#FF8C42', textDecorationColor: '#FF8C4266' },
	'.cm-link-exemplifies':  { color: '#4AFF88', textDecorationColor: '#4AFF8866' },
	'.cm-link-generalizes':  { color: '#A44AFF', textDecorationColor: '#A44AFF66' },
	'.cm-link-derives-from': { color: '#FFD700', textDecorationColor: '#FFD70066' },
	'.cm-link-part-of':      { color: '#AAAAAA', textDecorationColor: '#AAAAAA66' },
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
	'.cm-md-tag': {
		color: 'var(--library-accent, var(--interactive-accent))',
		backgroundColor: 'color-mix(in srgb, var(--library-accent, var(--interactive-accent)) 10%, transparent)',
		borderRadius: '3px',
		padding: '1px 4px',
		fontSize: '0.9em',
	},
	'.cm-md-checkbox': {
		verticalAlign: 'middle',
		marginInlineEnd: '4px',
		cursor: 'pointer',
		accentColor: 'var(--library-accent, var(--interactive-accent))',
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
	'.cm-md-image-widget': {
		display: 'block',
		margin: '8px 0',
	},
	'.cm-md-image-widget img': {
		maxWidth: '100%',
		borderRadius: '6px',
		border: '1px solid var(--background-modifier-border)',
	},
	'.cm-md-image-fallback': {
		display: 'inline-block',
		padding: '4px 8px',
		fontSize: '12px',
		color: 'var(--text-muted)',
		background: 'var(--background-secondary)',
		borderRadius: '4px',
	},
	'.cm-md-codeblock-lang': {
		display: 'inline-block',
		fontSize: '10px',
		fontWeight: '600',
		color: 'var(--text-muted)',
		textTransform: 'uppercase',
		letterSpacing: '0.5px',
		padding: '1px 6px',
		marginLeft: '8px',
		background: 'var(--background-secondary)',
		borderRadius: '3px',
		verticalAlign: 'middle',
	},
});

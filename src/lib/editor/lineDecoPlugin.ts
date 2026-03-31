/**
 * Line Decoration Plugin — Adds line-level decorations for:
 * - Blockquote lines (non-callout): left border + subtle background
 * - Fenced code blocks: background shading
 * Separate plugin to avoid conflicts with mark/replace decorations in livePreview.
 *
 * Performance: Only processes visible ranges. Callout detection uses
 * a single upward scan with early termination.
 */
import {
	ViewPlugin,
	type ViewUpdate,
	Decoration,
	type DecorationSet,
	EditorView,
} from '@codemirror/view';
import { RangeSetBuilder } from '@codemirror/state';
import { syntaxTree } from '@codemirror/language';

const codeBlockLineDeco = Decoration.line({ class: 'cm-codeblock-line' });
const blockquoteLineDeco = Decoration.line({ class: 'cm-blockquote-line' });

/** Check if a blockquote line belongs to a callout block by scanning upward (max 50 lines) */
function isInsideCallout(doc: { line(n: number): { text: string }}, lineNum: number): boolean {
	for (let j = lineNum - 1; j >= Math.max(1, lineNum - 50); j--) {
		const prevText = doc.line(j).text;
		if (/^>\s*\[!\w+\]/.test(prevText)) return true;
		if (!/^>\s?/.test(prevText)) return false;
	}
	return false;
}

function buildLineDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const builder = new RangeSetBuilder<Decoration>();
	// Use syntax tree once for the whole build — O(log n) node lookups vs O(N) line scan
	const tree = syntaxTree(view.state);

	for (const { from, to } of view.visibleRanges) {
		const startLine = doc.lineAt(from).number;
		const endLine = doc.lineAt(to).number;

		// ⚡ Check if viewport start sits inside a FencedCode node using the syntax tree.
		// This replaces the old O(N) forward scan from line 1 which became slow when
		// the user had scrolled far into a large document.
		let node = tree.resolve(from, 1);
		let inCodeBlock = false;
		while (node) {
			if (node.name === 'FencedCode') { inCodeBlock = true; break; }
			if (!node.parent) break;
			node = node.parent;
		}

		for (let i = startLine; i <= endLine; i++) {
			const line = doc.line(i);
			const text = line.text;

			// Toggle fenced code blocks
			if (/^```/.test(text)) {
				builder.add(line.from, line.from, codeBlockLineDeco);
				inCodeBlock = !inCodeBlock;
				continue;
			}

			if (inCodeBlock) {
				builder.add(line.from, line.from, codeBlockLineDeco);
				continue;
			}

			// Blockquote lines (skip callout content — handled by calloutPlugin)
			if (/^>\s?/.test(text) && !/^>\s*\[!\w+\]/.test(text)) {
				if (!isInsideCallout(doc, i)) {
					builder.add(line.from, line.from, blockquoteLineDeco);
				}
			}
		}
	}

	return builder.finish();
}

class LineDecoPluginClass {
	decorations: DecorationSet;
	private rebuildTimer: ReturnType<typeof setTimeout> | null = null;

	constructor(view: EditorView) {
		this.decorations = buildLineDecorations(view);
	}

	update(update: ViewUpdate) {
		if (update.viewportChanged) {
			// Scroll — sync rebuild
			if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
			this.decorations = buildLineDecorations(update.view);
			return;
		}
		if (update.docChanged) {
			// ⚡ Fast path: map through changes, debounce full rebuild
			this.decorations = this.decorations.map(update.changes);
			if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
			const view = update.view;
			this.rebuildTimer = setTimeout(() => {
				this.rebuildTimer = null;
				if (!view.destroyed) {
					this.decorations = buildLineDecorations(view);
				}
			}, 300);
		}
	}

	destroy() {
		if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
	}
}

export const lineDecoPlugin = ViewPlugin.fromClass(LineDecoPluginClass, {
	decorations: (v) => v.decorations,
});

export const lineDecoTheme = EditorView.theme({
	'.cm-codeblock-line': {
		backgroundColor: 'var(--background-primary-alt, #f5f5f5)',
		borderRadius: '0',
	},
	'.cm-blockquote-line': {
		borderInlineStart: '3px solid var(--text-faint, #ccc)',
		paddingInlineStart: '12px',
		backgroundColor: 'color-mix(in srgb, var(--text-faint, #ccc) 5%, transparent)',
	},
});

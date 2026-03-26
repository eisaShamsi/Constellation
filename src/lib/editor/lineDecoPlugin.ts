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

	// Pre-scan: determine if visible ranges start inside a code block
	// by counting ``` fences from document start to first visible line
	for (const { from, to } of view.visibleRanges) {
		const startLine = doc.lineAt(from).number;
		const endLine = doc.lineAt(to).number;

		// Determine code block state at startLine by scanning from doc start
		// (code blocks can span viewport boundaries)
		let inCodeBlock = false;
		for (let i = 1; i < startLine; i++) {
			if (/^```/.test(doc.line(i).text)) {
				inCodeBlock = !inCodeBlock;
			}
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
	rebuildTimer: ReturnType<typeof setTimeout> | null = null;

	constructor(view: EditorView) {
		this.decorations = buildLineDecorations(view);
	}

	update(update: ViewUpdate) {
		if (update.viewportChanged || (update.selectionSet && !update.docChanged)) {
			this.decorations = buildLineDecorations(update.view);
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
						this.decorations = buildLineDecorations(view);
					}
				});
			}, 400);
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

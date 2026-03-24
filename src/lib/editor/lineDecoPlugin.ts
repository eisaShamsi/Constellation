/**
 * Line Decoration Plugin — Adds line-level decorations for:
 * - Blockquote lines (non-callout): left border + subtle background
 * - Fenced code blocks: background shading
 * Separate plugin to avoid conflicts with mark/replace decorations in livePreview.
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

function buildLineDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const builder = new RangeSetBuilder<Decoration>();

	// Track code block regions and blockquote lines
	let inCodeBlock = false;

	for (let i = 1; i <= doc.lines; i++) {
		const line = doc.line(i);
		const text = line.text;

		// Toggle fenced code blocks
		if (/^```/.test(text)) {
			// Add decoration to the fence line itself
			builder.add(line.from, line.from, codeBlockLineDeco);
			inCodeBlock = !inCodeBlock;
			continue;
		}

		if (inCodeBlock) {
			builder.add(line.from, line.from, codeBlockLineDeco);
			continue;
		}

		// Blockquote lines (only non-callout: skip lines that are part of > [!type] blocks)
		if (/^>\s?/.test(text) && !/^>\s*\[!\w+\]/.test(text)) {
			// Check if this is a continuation of a callout (previous line was > [!type] or > content)
			let isCalloutContent = false;
			for (let j = i - 1; j >= 1; j--) {
				const prevText = doc.line(j).text;
				if (/^>\s*\[!\w+\]/.test(prevText)) {
					isCalloutContent = true;
					break;
				}
				if (/^>\s?/.test(prevText)) {
					continue; // keep checking upward
				}
				break; // non-blockquote line, stop
			}
			if (!isCalloutContent) {
				builder.add(line.from, line.from, blockquoteLineDeco);
			}
		}
	}

	return builder.finish();
}

class LineDecoPluginClass {
	decorations: DecorationSet;

	constructor(view: EditorView) {
		this.decorations = buildLineDecorations(view);
	}

	update(update: ViewUpdate) {
		if (update.docChanged || update.viewportChanged) {
			this.decorations = buildLineDecorations(update.view);
		}
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

/**
 * Bidi Plugin — Per-line direction detection for CodeMirror 6.
 * Detects the dominant script per line and applies dir="rtl" or dir="ltr"
 * as a line decoration. Also applies per-line font family when script fonts
 * are configured.
 */
import {
	ViewPlugin,
	type ViewUpdate,
	Decoration,
	type DecorationSet,
	EditorView,
} from '@codemirror/view';
import { RangeSetBuilder, StateEffect, StateField } from '@codemirror/state';

// RTL script ranges (Arabic, Hebrew, Farsi, Urdu, etc.)
const RTL_RE = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF\uFB1D-\uFB4F]/;
const LTR_RE = /[A-Za-z\u00C0-\u024F\u1E00-\u1EFF\u0400-\u04FF]/;

// Arabic-specific range for font detection
const ARABIC_RE = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/;

/** Detect direction of a single line by first strong directional character */
function detectLineDir(text: string): 'rtl' | 'ltr' | null {
	// Strip markdown syntax to find actual text content
	const clean = text.replace(/^[#*>\-\s`\[\]()!|~=+\d.]+/, '');
	if (!clean) return null; // empty or syntax-only line

	for (const ch of clean) {
		if (RTL_RE.test(ch)) return 'rtl';
		if (LTR_RE.test(ch)) return 'ltr';
	}
	return null; // no strong directional characters (numbers, symbols only)
}

/** Detect dominant script for font selection */
function detectLineScript(text: string): string | null {
	if (ARABIC_RE.test(text)) return 'arabic';
	return null; // default font
}

// ─── Script font configuration (passed from app settings) ───
export const setScriptFonts = StateEffect.define<Record<string, string>>();

export const scriptFontsField = StateField.define<Record<string, string>>({
	create: () => ({}),
	update(value, tr) {
		for (const effect of tr.effects) {
			if (effect.is(setScriptFonts)) return effect.value;
		}
		return value;
	},
});

function buildBidiDecorations(view: EditorView): DecorationSet {
	const doc = view.state.doc;
	const scriptFonts = view.state.field(scriptFontsField, false) || {};
	const builder = new RangeSetBuilder<Decoration>();

	// Get the editor's base direction
	const editorDir = view.dom.getAttribute('dir') || 'ltr';

	// Track direction of the previous non-empty line for empty-line inheritance
	let prevDir: 'rtl' | 'ltr' | null = null;

	for (const { from, to } of view.visibleRanges) {
		let lineNum = doc.lineAt(from).number;
		const endLineNum = doc.lineAt(to).number;

		// Seed prevDir by scanning the line just above the visible range
		if (lineNum > 1) {
			for (let j = lineNum - 1; j >= Math.max(1, lineNum - 10); j--) {
				const d = detectLineDir(doc.line(j).text);
				if (d) { prevDir = d; break; }
			}
		}

		while (lineNum <= endLineNum) {
			const line = doc.line(lineNum);
			let lineDir = detectLineDir(line.text);

			// Empty line: inherit direction from the nearest previous non-empty line.
			// This ensures the cursor lands on the correct side after pressing Enter in RTL text.
			if (lineDir === null && prevDir !== null) {
				lineDir = prevDir;
			}

			if (lineDir) {
				prevDir = lineDir; // update for next empty-line

				const attrs: Record<string, string> = {};
				const style: string[] = [];

				// Apply dir attribute only when it differs from the editor base
				if (lineDir !== editorDir) {
					attrs.dir = lineDir;
					style.push(`text-align: ${lineDir === 'rtl' ? 'right' : 'left'}`);
				}

				// Apply script-specific font
				const script = detectLineScript(line.text);
				if (script && scriptFonts[script]) {
					style.push(`font-family: ${scriptFonts[script]}`);
				}

				if (style.length) attrs.style = style.join('; ');

				// Only add decoration if there's something to apply
				if (Object.keys(attrs).length > 0) {
					builder.add(line.from, line.from, Decoration.line({ attributes: attrs }));
				}
			}

			lineNum++;
		}
	}

	return builder.finish();
}

class BidiPluginClass {
	decorations: DecorationSet;

	constructor(view: EditorView) {
		this.decorations = buildBidiDecorations(view);
	}

	update(update: ViewUpdate) {
		if (update.docChanged || update.viewportChanged
			|| update.transactions.some(t => t.effects.some(e => e.is(setScriptFonts)))) {
			this.decorations = buildBidiDecorations(update.view);
		}
	}
}

export const bidiPlugin = ViewPlugin.fromClass(BidiPluginClass, {
	decorations: (v) => v.decorations,
});

export const bidiTheme = EditorView.theme({
	// Ensure RTL lines have proper cursor positioning
	'.cm-line[dir="rtl"]': {
		unicodeBidi: 'isolate',
	},
	'.cm-line[dir="ltr"]': {
		unicodeBidi: 'isolate',
	},
});

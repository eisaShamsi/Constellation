/**
 * Bidi Plugin — Per-line direction detection for CodeMirror 6.
 * Detects the dominant script per line and applies dir="rtl" as a line decoration.
 * LTR is the default — only RTL lines and empty lines inheriting RTL get decorated.
 * Also applies per-line font family when script fonts are configured.
 *
 * Performance: map+debounce on docChanged (syntaxTree-free, but still O(visible_lines)).
 * Theme: .cm-line[dir] gets unicode-bidi:isolate to override the plaintext CSS in NotePane,
 * ensuring the dir='rtl' attribute actually controls cursor placement on empty lines.
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

// Script-specific ranges for per-line font detection
const ARABIC_RE    = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/;
const HEBREW_RE    = /[\u0590-\u05FF\uFB1D-\uFB4F]/;
const DEVANAGARI_RE = /[\u0900-\u097F]/;
const CYRILLIC_RE  = /[\u0400-\u04FF]/;
const CJK_RE       = /[\u4E00-\u9FFF\u3400-\u4DBF\u3000-\u303F\u30A0-\u30FF\u3040-\u309F\uAC00-\uD7AF]/;

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

/** Detect dominant script for font selection — returns key matching TYPEWRITER_FONTS.scriptFonts */
function detectLineScript(text: string): string | null {
	if (ARABIC_RE.test(text))    return 'arabic';
	if (HEBREW_RE.test(text))    return 'hebrew';
	if (DEVANAGARI_RE.test(text)) return 'devanagari';
	// CJK: distinguish Japanese (has kana) vs Korean (has Hangul) vs Chinese
	if (CJK_RE.test(text)) {
		if (/[\u3040-\u30FF]/.test(text)) return 'japanese';  // Hiragana/Katakana → Japanese
		if (/[\uAC00-\uD7AF]/.test(text)) return 'korean';    // Hangul → Korean
		return 'chinese';                                        // CJK only → Chinese
	}
	if (CYRILLIC_RE.test(text))  return 'cyrillic';
	return null; // default font (Latin)
}

/** Resolve the editor's effective base direction, treating 'auto' as 'ltr' unless content says otherwise */
function resolveEditorDir(view: EditorView): 'rtl' | 'ltr' {
	const domDir = view.dom.getAttribute('dir') || 'ltr';
	if (domDir === 'rtl') return 'rtl';
	if (domDir === 'ltr') return 'ltr';
	// 'auto' — scan first few lines to determine actual direction
	const doc = view.state.doc;
	for (let i = 1; i <= Math.min(doc.lines, 10); i++) {
		const d = detectLineDir(doc.line(i).text);
		if (d) return d;
	}
	return 'ltr'; // fallback: LTR
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
	const editorDir = resolveEditorDir(view);

	// Track direction of the most recent non-empty line for empty-line inheritance.
	// Empty lines after RTL lines should get dir='rtl' so the cursor lands on the right
	// side (correct RTL beginning) when the user presses Enter after RTL text.
	let prevDir: 'rtl' | 'ltr' | null = null;

	for (const { from, to } of view.visibleRanges) {
		const startLine = doc.lineAt(from).number;
		const endLine = doc.lineAt(to).number;

		// Seed prevDir by scanning lines just above the visible range (max 10 up)
		if (startLine > 1) {
			for (let j = startLine - 1; j >= Math.max(1, startLine - 10); j--) {
				const d = detectLineDir(doc.line(j).text);
				if (d) { prevDir = d; break; }
			}
		}

		for (let lineNum = startLine; lineNum <= endLine; lineNum++) {
			const line = doc.line(lineNum);
			let lineDir = detectLineDir(line.text);

			// Empty line: inherit direction from nearest preceding non-empty line.
			// Only inherit RTL — LTR is the default and needs no explicit decoration.
			if (lineDir === null && prevDir === 'rtl') {
				lineDir = 'rtl';
			}

			if (lineDir !== null) {
				prevDir = lineDir;
			}

			// Only decorate when the line direction differs from the editor base direction.
			// This prevents adding dir='ltr' to LTR lines in LTR documents (a no-op that
			// caused DOM thrashing when editorDir was 'auto' and all lines got decorated).
			if (lineDir !== null && lineDir !== editorDir) {
				const attrs: Record<string, string> = { dir: lineDir };
				const style: string[] = [
					`text-align: ${lineDir === 'rtl' ? 'right' : 'left'}`,
				];

				const script = detectLineScript(line.text);
				if (script && scriptFonts[script]) {
					style.push(`font-family: ${scriptFonts[script]}`);
				}

				attrs.style = style.join('; ');
				builder.add(line.from, line.from, Decoration.line({ attributes: attrs }));
			} else if (lineDir !== null && lineDir === editorDir) {
				// Same direction as editor base — only add if there's a font override
				const script = detectLineScript(line.text);
				if (script && scriptFonts[script]) {
					builder.add(line.from, line.from, Decoration.line({
						attributes: { style: `font-family: ${scriptFonts[script]}` },
					}));
				}
			}
		}
	}

	return builder.finish();
}

class BidiPluginClass {
	decorations: DecorationSet;
	private rebuildTimer: ReturnType<typeof setTimeout> | null = null;

	constructor(view: EditorView) {
		this.decorations = buildBidiDecorations(view);
	}

	update(update: ViewUpdate) {
		const hasScriptFonts = update.transactions.some(t => t.effects.some(e => e.is(setScriptFonts)));

		if (update.viewportChanged || hasScriptFonts) {
			// Scroll or font change — sync rebuild
			if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
			this.decorations = buildBidiDecorations(update.view);
			return;
		}

		if (update.docChanged) {
			// map fast path, then debounced rebuild (no syntaxTree but still O(visible_lines))
			this.decorations = this.decorations.map(update.changes);
			if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
			const view = update.view;
			this.rebuildTimer = setTimeout(() => {
				this.rebuildTimer = null;
				if (!view.destroyed) {
					this.decorations = buildBidiDecorations(view);
				}
			}, 300);
		}
	}

	destroy() {
		if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
	}
}

export const bidiPlugin = ViewPlugin.fromClass(BidiPluginClass, {
	decorations: (v) => v.decorations,
});

export const bidiTheme = EditorView.theme({
	// Lines with an explicit dir attribute use isolate so the dir attribute
	// actually governs cursor placement — overrides NotePane's unicode-bidi:plaintext.
	// Without this, dir='rtl' on an empty line is ignored and the cursor defaults to LTR.
	'.cm-line[dir]': {
		unicodeBidi: 'isolate',
	},
	'.cm-line[dir="rtl"]': {
		unicodeBidi: 'isolate',
	},
	'.cm-line[dir="ltr"]': {
		unicodeBidi: 'isolate',
	},
});

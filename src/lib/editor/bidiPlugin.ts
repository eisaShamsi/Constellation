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

/** Leading whitespace + block-level markdown markers, STRUCTURED (nesting allowed:
 *  `> - [x] `). Shared by detectLineDir (strip before the first-strong scan \u2014 a checked
 *  task's `x` must never read as the line's first strong char) and by paragraphDir.ts \u00A7B4
 *  (the direction mark is inserted right after this prefix). Lives here so the import is
 *  one-directional (paragraphDir already imports from this module). */
export const BLOCK_PREFIX_RE = /^(?:\s*(?:>\s?|(?:[-*+]|\d{1,9}[.)])\s+(?:\[[ xX]\]\s+)?|#{1,6}\s+))*\s*/;

// Script-specific ranges for per-line font detection
const ARABIC_RE    = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/;
const HEBREW_RE    = /[\u0590-\u05FF\uFB1D-\uFB4F]/;
const DEVANAGARI_RE = /[\u0900-\u097F]/;
const CYRILLIC_RE  = /[\u0400-\u04FF]/;
const CJK_RE       = /[\u4E00-\u9FFF\u3400-\u4DBF\u3000-\u303F\u30A0-\u30FF\u3040-\u309F\uAC00-\uD7AF]/;

/** Detect direction of a single line by first strong directional character.
 *  PJ-106 §B4: the invisible direction marks (RLM U+200F / LRM U+200E) ARE strong
 *  characters per UAX #9 — the browser's `unicode-bidi: plaintext` first-strong already
 *  honors them, so this scan must agree or the plugin's dir stamp would FIGHT the user's
 *  explicit per-paragraph override (paragraphDir.ts) and flip a forced line back.
 *  Exported for the §B4 headless tests. */
export function detectLineDir(text: string): 'rtl' | 'ltr' | null {
	// Structured strip FIRST (quotes/bullets/ordered/task markers — `- [x] ` would otherwise
	// leave its `x` as the first strong char and misread a checked Arabic task as LTR), then
	// the legacy flat strip for the leftovers. The §B4 marks are in neither class — a leading
	// mark survives into `clean` and wins the first-strong scan below.
	const clean = text.replace(BLOCK_PREFIX_RE, '').replace(/^[#*>\-\s`\[\]()!|~=+\d.]+/, '');
	if (!clean) return null; // empty or syntax-only line

	for (const ch of clean) {
		if (ch === '‏') return 'rtl'; // RLM — the §B4 override mark
		if (ch === '‎') return 'ltr'; // LRM
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

/** Resolve the editor's base direction from the DOM `dir` attribute the host sets
 *  (NotePane/FocusPane/merge panes now set a DETERMINISTIC 'rtl'|'ltr', never 'auto').
 *  PJ-106 §A1 (H9): the buildBidiDecorations gate must use the SAME base the content
 *  attribute uses, so a per-line decoration is added exactly when the line differs from
 *  that base. The legacy 'auto' viewport-scan branch is removed — it could disagree with
 *  the content attribute and desync motion from render; a lingering 'auto'/unset dir
 *  falls back to 'ltr'. */
function resolveEditorDir(view: EditorView): 'rtl' | 'ltr' {
	return view.dom.getAttribute('dir') === 'rtl' ? 'rtl' : 'ltr';
}

/** PJ-106 §B4 — did this change set touch a direction mark (RLM/LRM), in either the
 *  inserted or the DELETED text? A mark edit changes text WITHOUT changing the line count,
 *  so the §A2 structural branch doesn't fire and the map-fast-path would leave the dir
 *  stamp (and with it the MOTION engine's textDirectionAt) up to 300 ms behind the render —
 *  the SI2-3 desync class. Content-based (not effect-based) so undo/redo, paste, and
 *  cross-window adoption of marked text all get the same-frame rebuild. O(changed spans). */
const MARKS_RE = /[‎‏]/;
function changesTouchDirMarks(update: ViewUpdate): boolean {
	if (!update.docChanged) return false;
	let touched = false;
	update.changes.iterChanges((fromA, toA, _fromB, _toB, inserted) => {
		if (touched) return;
		if (MARKS_RE.test(inserted.toString())) touched = true;
		else if (fromA < toA && MARKS_RE.test(update.startState.doc.sliceString(fromA, toA))) touched = true;
	});
	return touched;
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
			const detected = detectLineDir(line.text);
			// A NEUTRAL line has no strong directional character (empty / whitespace / syntax /
			// numbers only). Its effective direction is its own strong dir, else the nearest
			// preceding non-empty line, else the editor's base.
			const isNeutral = detected === null;
			const lineDir: 'rtl' | 'ltr' = detected ?? prevDir ?? editorDir;
			if (detected !== null) prevDir = detected;

			// PJ-106 §A3 — a NEUTRAL line at RTL needs an EXPLICIT dir attribute to override
			// `.cm-line { unicode-bidi: plaintext }`, which otherwise resolves an empty line to
			// LTR (no strong char → the Unicode default), dropping the caret on the LEFT even in
			// an RTL note (the Boss's Enter-on-RTL bug). A NON-neutral line only needs a dir when
			// it differs from the base — its own first-strong char already renders it correctly
			// under plaintext, so stamping the base-matching majority avoids DOM thrash.
			const needsDir = lineDir !== editorDir || (isNeutral && lineDir === 'rtl');
			if (needsDir) {
				const attrs: Record<string, string> = { dir: lineDir };
				const style: string[] = [`text-align: ${lineDir === 'rtl' ? 'right' : 'left'}`];
				const script = detectLineScript(line.text);
				if (script && scriptFonts[script]) style.push(`font-family: ${scriptFonts[script]}`);
				attrs.style = style.join('; ');
				builder.add(line.from, line.from, Decoration.line({ attributes: attrs }));
			} else {
				// Base-matching line — only a font override, if configured.
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
		const hasScriptFonts = update.transactions.some(t => t.effects.some(e => e.is(setScriptFonts)))
			|| changesTouchDirMarks(update); // §B4 — mark edits rebuild same-frame (incl. undo/redo)

		if (update.viewportChanged || hasScriptFonts) {
			// Scroll or font change — sync rebuild
			if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
			this.decorations = buildBidiDecorations(update.view);
			return;
		}

		if (update.docChanged) {
			// map fast path first (keeps existing dir decorations aligned to the new offsets)
			this.decorations = this.decorations.map(update.changes);
			// PJ-106 §A2 — a STRUCTURAL change (line inserted/removed, e.g. Enter) rebuilds
			// SYNCHRONOUSLY, so a brand-new empty line gets its inherited dir='rtl' in the SAME
			// frame. Otherwise the 300 ms debounce leaves the new line under unicode-bidi:plaintext
			// (LTR default) and the caret flashes on the wrong side after every Enter. Pressing
			// Enter is an occasional gesture, not a per-keystroke burst, so a sync O(visible_lines)
			// rebuild here does not violate Rule 1 (character typing still takes the debounce below).
			if (update.startState.doc.lines !== update.state.doc.lines) {
				if (this.rebuildTimer) { clearTimeout(this.rebuildTimer); this.rebuildTimer = null; }
				this.decorations = buildBidiDecorations(update.view);
				return;
			}
			if (this.rebuildTimer) clearTimeout(this.rebuildTimer);
			const view = update.view;
			this.rebuildTimer = setTimeout(() => {
				this.rebuildTimer = null;
				if (!(view as any).destroyed) {
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

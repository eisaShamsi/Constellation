/**
 * Callout Plugin — Obsidian-compatible callout rendering for CodeMirror 6.
 *
 * ┌─────────────────────────────────────────────────────────────────────────────┐
 * │  FREEZE-PROOF ARCHITECTURE — Two inviolable rules                           │
 * │                                                                             │
 * │  ROOT CAUSE of CM6 freeze:                                                  │
 * │  Decoration.replace([from, to]) creates a cursor-exclusion range.           │
 * │  If the cursor is inside that range, CM6 nudges it out → selectionSet       │
 * │  fires → plugin rebuilds → range is restored → CM6 nudges again → loop.    │
 * │  The editor freezes permanently.                                            │
 * │                                                                             │
 * │  RULE A — Cursor-safe replace (title widget + ">" prefix removal):          │
 * │    Decoration.replace is ONLY added when the cursor is on a DIFFERENT line. │
 * │    At line granularity this is provably safe: a cursor on line N cannot     │
 * │    be inside a replace range that covers exclusively line M (M ≠ N).       │
 * │                                                                             │
 * │  RULE B — Zero-length line decoration (collapsed body hiding):              │
 * │    Decoration.line({ class }) is added at (line.from, line.from).           │
 * │    from === to → no range → cursor can never be "inside" it → CM6          │
 * │    never nudges → freeze loop is architecturally impossible.                │
 * │    CSS display:none on .cm-callout-body-collapsed does the actual hiding.   │
 * └─────────────────────────────────────────────────────────────────────────────┘
 *
 * Other design decisions:
 *   - Per-type color via CSS --callout-color (no color values in JS)
 *   - Fold state: StateField<Set<number>> survives doc changes via pos mapping
 *   - Chevron click: handled by NotePane's capture-phase mousedown — NOT here
 *     (avoids double-dispatch when both a plugin handler and a native handler exist)
 *   - Viewport-only scan: O(visible_lines), never O(document)
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
import { resolveOverrideSync, resolveRefSync } from '$lib/theme/iconOverrides';
import { CALLOUT_ICONS, CALLOUT_FAMILY } from '$lib/editor/calloutFamilies';
import { peekCustomCallout } from '$lib/theme/customCallouts';

// The family data lives in calloutFamilies.ts (dependency-free; shared with the
// customCallouts store + the Setter UI). Re-export for existing importers.
export { CALLOUT_FAMILIES, calloutDefaultIcon } from '$lib/editor/calloutFamilies';

// MIG-089 Phase A — dispatch this to force a callout decoration rebuild (so a
// per-type ICON override or a new custom type repaints an OPEN editor without the
// user typing/scrolling). Colours ride CSS and need no rebuild; icons are baked
// into the widget DOM at build time, so a settings change must re-trigger a build.
export const refreshCallouts = StateEffect.define<null>();

// ─── Fold state ───────────────────────────────────────────────────────────────
// Dispatch toggleCallout.of(startLineNumber) to toggle a callout open/closed.
// The field stores a Set of *toggled* line numbers — flipped from their default state.
// (A "-" marker means defaultCollapsed=true. Adding its line to the Set means open.)
export const toggleCallout = StateEffect.define<number>();

export const calloutCollapseField = StateField.define<Set<number>>({
	create: () => new Set(),
	update(collapsed, tr) {
		let next = collapsed;

		// Remap stored line numbers through document edits so fold state survives typing
		if (tr.docChanged) {
			const mapped = new Set<number>();
			for (const ln of collapsed) {
				try {
					const oldPos = tr.startState.doc.line(ln).from;
					mapped.add(tr.state.doc.lineAt(tr.changes.mapPos(oldPos)).number);
				} catch { /* line was deleted — drop it */ }
			}
			next = mapped;
		}

		// Apply toggle effects from this transaction
		for (const e of tr.effects) {
			if (e.is(toggleCallout)) {
				next = new Set(next);
				next.has(e.value) ? next.delete(e.value) : next.add(e.value);
			}
		}

		return next;
	},
});

// ─── Title widget ─────────────────────────────────────────────────────────────
// Replaces the raw "> [!type]- My Title" line with a styled widget.
// Only added when the cursor is on a DIFFERENT line (RULE A).
class CalloutTitleWidget extends WidgetType {
	constructor(
		readonly type: string,
		readonly icon: string,
		readonly title: string,
		readonly displayName: string,   // MIG-089 — shown when there's no explicit title (the custom type's name)
		readonly foldable: boolean,
		readonly collapsed: boolean,
		readonly lineNum: number,
	) { super(); }

	toDOM() {
		// Detect direction from title text (RTL scripts → rtl, else ltr)
		const isRtl = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/.test(this.title || this.displayName);
		const titleDir = isRtl ? 'rtl' : 'ltr';

		const wrap = document.createElement('span');
		wrap.className = 'cm-callout-title-widget';
		wrap.setAttribute('dir', titleDir);
		wrap.setAttribute('data-callout', this.type);

		// Icon — an emoji glyph (textContent) or a resolved <svg> override (innerHTML).
		const iconEl = document.createElement('span');
		iconEl.className = 'cm-callout-icon';
		if (this.icon.startsWith('<svg')) iconEl.innerHTML = this.icon;
		else iconEl.textContent = this.icon;
		wrap.appendChild(iconEl);

		// Fold chevron (only rendered if the callout has a +/- marker)
		if (this.foldable) {
			const chevron = document.createElement('span');
			chevron.className = 'cm-callout-chevron';
			chevron.textContent = this.collapsed ? ' ▶' : ' ▼';
			chevron.dataset.calloutLine = String(this.lineNum);
			wrap.appendChild(chevron);
		}

		// Title text
		const titleEl = document.createElement('span');
		titleEl.className = 'cm-callout-title-text';
		const headerText = this.title || this.displayName;
		titleEl.textContent = headerText ? ' ' + headerText : '';
		wrap.appendChild(titleEl);

		return wrap;
	}

	// CM6 reuses existing DOM if eq() returns true — no needless rebuilds
	eq(other: CalloutTitleWidget): boolean {
		return (
			this.type === other.type &&
			this.icon === other.icon &&   // MIG-089 — an icon override must re-render the widget
			this.title === other.title &&
			this.displayName === other.displayName &&
			this.foldable === other.foldable &&
			this.collapsed === other.collapsed &&
			this.lineNum === other.lineNum
		);
	}

	// true → CM6 does not swallow events from this widget → they bubble to NotePane
	ignoreEvent() { return true; }
}

// ─── Callout detection ────────────────────────────────────────────────────────
interface CalloutBlock {
	type: string;
	foldMarker: string; // '+' | '-' | ''
	startLine: number;
	endLine: number;
}

/**
 * Finds every callout block whose *start* line falls within [fromLine, toLine].
 * A callout block is a run of contiguous "> " lines starting with "> [!type]".
 */
function findCalloutsInRange(
	doc: EditorView['state']['doc'],
	fromLine: number,
	toLine: number,
): CalloutBlock[] {
	const result: CalloutBlock[] = [];
	let ln = fromLine;

	while (ln <= toLine) {
		const text = doc.line(ln).text;
		// MIG-089 — the type is any non-`]` run (Language-First: Arabic/CJK/… триггеры,
		// not just Latin `\w`). Trimmed + lowercased to match the stored slug.
		const m = text.match(/^>\s*\[!([^\]\n]+)\]([+-])?\s*/);
		if (m) {
			const type = m[1].trim().toLowerCase();
			const foldMarker = m[2] ?? '';

			// Walk forward to find the last body line
			const cap = Math.min(doc.lines, ln + 200);
			let endLine = ln;
			for (let l = ln + 1; l <= cap; l++) {
				const t = doc.line(l).text;
				if (!/^>\s?/.test(t) || /^>\s*\[![^\]\n]+\]/.test(t)) break;
				endLine = l;
			}

			result.push({ type, foldMarker, startLine: ln, endLine });
			ln = endLine + 1; // skip to after this callout
		} else {
			ln++;
		}
	}

	return result;
}

// ─── Decoration builder ───────────────────────────────────────────────────────
function buildCalloutDecorations(view: EditorView): DecorationSet {
	const { doc } = view.state;
	const cursorLine = doc.lineAt(view.state.selection.main.head).number;
	const collapsed = view.state.field(calloutCollapseField);

	// Collect then sort — RangeSetBuilder requires strictly ascending positions
	const all: { from: number; to: number; deco: Decoration }[] = [];

	for (const { from, to } of view.visibleRanges) {
		// Scan a few lines above viewport so callouts starting just off-screen render correctly
		const startLine = Math.max(1, doc.lineAt(from).number - 5);
		const endLine = doc.lineAt(to).number;

		for (const callout of findCalloutsInRange(doc, startLine, endLine)) {
			// MIG-089 — per-type icon override (emoji or SVG) keyed on the family;
			// falls back to the built-in emoji (and to ℹ️ for an unknown custom type).
			const isBuiltin = callout.type in CALLOUT_FAMILY;
				const family = isBuiltin ? CALLOUT_FAMILY[callout.type] : callout.type;
				// A CUSTOM type (not a built-in family) takes its icon + colour from the
				// per-Universe customCallouts registry; its colour is injected inline below
				// (built-in types get their colour from the CSS theme var instead).
				const custom = isBuiltin ? null : peekCustomCallout(callout.type);
				const customColor = custom && /^#[0-9a-fA-F]{3,8}$|^rgba?\(/.test(custom.color) ? custom.color : null;
				const colorStyle = customColor ? { style: '--callout-color:' + customColor } : null;
			const icon = (custom ? resolveRefSync(custom.icon) : resolveOverrideSync('callout.' + family)) ?? CALLOUT_ICONS[callout.type] ?? 'ℹ️';
			const foldable = callout.foldMarker === '-' || callout.foldMarker === '+';
			const defaultCollapsed = callout.foldMarker === '-';

			// XOR toggle: if the line is in the Set, its visible state is flipped
			const isCollapsed = collapsed.has(callout.startLine)
				? !defaultCollapsed
				: defaultCollapsed;

			const titleLine = doc.line(callout.startLine);
			const rawTitle = titleLine.text
				.replace(/^>\s*\[![^\]\n]+\][+-]?\s*/, '')
				.trim();
			// MIG-089 \u2014 the header label shown when there is NO explicit title: a custom
			// type shows its NAME (e.g. \u00AB\u0641\u0643\u0631\u0629\u00BB), so `> [!\u0641\u0643\u0631\u0629]` headers read \u00AB\u0641\u0643\u0631\u0629\u00BB (bold).
			// Built-ins keep today's look (no default title). The widget shows rawTitle || displayName.
			const displayName = custom ? custom.name : '';
			const headerText = rawTitle || displayName;
			// Detect direction from whatever the header will actually show.
			const titleIsRtl = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/.test(headerText);
			const titleDir = titleIsRtl ? 'rtl' : 'ltr';

			// ── 1. Title line border + tint (always, even when cursor is on it) ──
			all.push({
				from: titleLine.from, to: titleLine.from,
				deco: Decoration.line({
					class: 'cm-callout-line cm-callout-title-line',
					attributes: { 'data-callout': callout.type, dir: titleDir, ...colorStyle },
				}),
			});

			// ── 2. Title widget (RULE A: only when cursor is elsewhere) ──
			if (cursorLine !== callout.startLine) {
				all.push({
					from: titleLine.from,
					to: titleLine.to,
					deco: Decoration.replace({
						widget: new CalloutTitleWidget(
							callout.type, icon, rawTitle, displayName,
							foldable, isCollapsed, callout.startLine,
						),
					}),
				});
			}

			if (!isCollapsed) {
				// ── 3. Body lines: border + tint + ">" prefix removal ──
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					const line = doc.line(l);

					// Border + tint (zero-length — always safe)
					const bodyText = line.text.replace(/^>\s?/, '');
					const bodyIsRtl = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF]/.test(bodyText);
					all.push({
						from: line.from, to: line.from,
						deco: Decoration.line({
							class: 'cm-callout-line cm-callout-body-line',
							attributes: { 'data-callout': callout.type, dir: bodyIsRtl ? 'rtl' : 'ltr', ...colorStyle },
						}),
					});

					// ">" prefix removal (RULE A: skip cursor line)
					if (cursorLine !== l) {
						const prefix = line.text.match(/^>\s?/);
						if (prefix) {
							all.push({
								from: line.from,
								to: line.from + prefix[0].length,
								deco: Decoration.replace({}),
							});
						}
					}
				}
			} else {
				// ── 4. Collapsed body: hide with CSS (RULE B: zero-length line deco) ──
				// The cursor's own line is always kept visible so the cursor never disappears.
				for (let l = callout.startLine + 1; l <= callout.endLine; l++) {
					if (cursorLine === l) continue; // never hide cursor line
					const line = doc.line(l);
					all.push({
						from: line.from, to: line.from,  // from === to = zero-length
						deco: Decoration.line({ class: 'cm-callout-body-collapsed' }),
					});
				}
			}
		}
	}

	// Sort ascending by position; line decos (to===from) before inline decos at same pos
	all.sort((a, b) => a.from - b.from || a.to - b.to);

	const builder = new RangeSetBuilder<Decoration>();
	for (const { from, to, deco } of all) builder.add(from, to, deco);
	return builder.finish();
}

// ─── ViewPlugin ───────────────────────────────────────────────────────────────
class CalloutDecoPlugin {
	decorations: DecorationSet;
	private lastCursorLine = -1;

	constructor(view: EditorView) {
		this.decorations = buildCalloutDecorations(view);
		this.lastCursorLine = view.state.doc.lineAt(view.state.selection.main.head).number;
	}

	update(update: ViewUpdate) {
		const hasToggle = update.transactions.some(
			t => t.effects.some(e => e.is(toggleCallout) || e.is(refreshCallouts))
		);

		// Full rebuild on: fold toggle, callout-settings refresh, doc edit, or viewport scroll
		if (hasToggle || update.docChanged || update.viewportChanged) {
			this.decorations = buildCalloutDecorations(update.view);
			this.lastCursorLine = update.view.state.doc
				.lineAt(update.view.state.selection.main.head).number;
			return;
		}

		// Cursor move: only rebuild when cursor crosses a line boundary
		// (title widget visibility depends on which line the cursor is on)
		if (update.selectionSet) {
			const newLine = update.view.state.doc
				.lineAt(update.view.state.selection.main.head).number;
			if (newLine !== this.lastCursorLine) {
				this.lastCursorLine = newLine;
				this.decorations = buildCalloutDecorations(update.view);
			}
		}
	}
}

const calloutDecoPlugin = ViewPlugin.fromClass(CalloutDecoPlugin, {
	decorations: v => v.decorations,
});

// ─── Exports ──────────────────────────────────────────────────────────────────
// Chevron click is handled by NotePane's chevronHandler (capture-phase mousedown).
// We do NOT register a domEventHandlers here — two handlers on the same element
// would cause double-dispatch in edge cases.
export const calloutPlugin = [calloutDecoPlugin];
export const calloutClickHandler = EditorView.domEventHandlers({}); // kept for import compat

export const calloutTheme = EditorView.theme({
	// Left border on every callout line
	'.cm-callout-line': {
		borderInlineStart: 'var(--blockquote-border-width, 3px) solid var(--callout-color, #448aff)',
		paddingInlineStart: '12px !important',
		borderRadius: 'var(--callout-radius, 0)',
	},
	// Title line: stronger background + bold
	'.cm-callout-title-line': {
		background: 'color-mix(in srgb, var(--callout-color, #448aff) 8%, transparent)',
		fontWeight: '500',
	},
	// Body line: lighter background
	'.cm-callout-body-line': {
		background: 'color-mix(in srgb, var(--callout-color, #448aff) 4%, transparent)',
	},
	// RULE B — collapsed body lines: hidden by CSS, not by Decoration.replace.
	// CM6 sees these lines as normal (zero-length deco) so the cursor can move
	// through them freely and the cursor's own line is always kept visible above.
	'.cm-callout-body-collapsed': {
		display: 'none',
	},
	// Widget layout
	'.cm-callout-title-widget': { display: 'inline', direction: 'inherit' },
	'.cm-callout-icon': {
		fontWeight: '600',
		fontSize: '0.95em',
		verticalAlign: 'middle',
		color: 'var(--callout-color, #448aff)',
	},
	// MIG-089 — an SVG icon override (Lucide/Phosphor/…) sizes to ~1em and inherits
	// the callout colour via currentColor (the icon sets use stroke="currentColor").
	'.cm-callout-icon svg': {
		width: '1em',
		height: '1em',
		verticalAlign: 'middle',
		display: 'inline-block',
	},
	'.cm-callout-chevron': {
		cursor: 'pointer',
		fontSize: '0.8em',
		opacity: '0.7',
		userSelect: 'none',
		verticalAlign: 'middle',
	},
	'.cm-callout-title-text': { fontWeight: '700' },   // MIG-089 — the callout header label is bold

	// Per-type color variables — add any new type here, never in JS.
	// MIG-088 Phase 3 §3a — each family reads a per-Universe Style Setter var
	// (Editor → Callouts) with TODAY'S hex as the fallback, so the look is
	// byte-identical until the user edits a colour. Aliases share their family
	// var so they stay in lockstep. `question` and `warning` keep SEPARATE vars
	// (distinct callout families) though both default to #ff9100.
	'[data-callout="note"]':      { '--callout-color': 'var(--callout-note-color, #448aff)' },
	'[data-callout="info"]':      { '--callout-color': 'var(--callout-note-color, #448aff)' },
	'[data-callout="abstract"]':  { '--callout-color': 'var(--callout-abstract-color, #00b0ff)' },
	'[data-callout="summary"]':   { '--callout-color': 'var(--callout-abstract-color, #00b0ff)' },
	'[data-callout="tldr"]':      { '--callout-color': 'var(--callout-abstract-color, #00b0ff)' },
	'[data-callout="tip"]':       { '--callout-color': 'var(--callout-tip-color, #00bfa5)' },
	'[data-callout="hint"]':      { '--callout-color': 'var(--callout-tip-color, #00bfa5)' },
	'[data-callout="important"]': { '--callout-color': 'var(--callout-tip-color, #00bfa5)' },
	'[data-callout="success"]':   { '--callout-color': 'var(--callout-success-color, #00c853)' },
	'[data-callout="check"]':     { '--callout-color': 'var(--callout-success-color, #00c853)' },
	'[data-callout="done"]':      { '--callout-color': 'var(--callout-success-color, #00c853)' },
	'[data-callout="question"]':  { '--callout-color': 'var(--callout-question-color, #ff9100)' },
	'[data-callout="help"]':      { '--callout-color': 'var(--callout-question-color, #ff9100)' },
	'[data-callout="faq"]':       { '--callout-color': 'var(--callout-question-color, #ff9100)' },
	'[data-callout="warning"]':   { '--callout-color': 'var(--callout-warning-color, #ff9100)' },
	'[data-callout="caution"]':   { '--callout-color': 'var(--callout-warning-color, #ff9100)' },
	'[data-callout="attention"]': { '--callout-color': 'var(--callout-warning-color, #ff9100)' },
	'[data-callout="failure"]':   { '--callout-color': 'var(--callout-failure-color, #ff5252)' },
	'[data-callout="fail"]':      { '--callout-color': 'var(--callout-failure-color, #ff5252)' },
	'[data-callout="missing"]':   { '--callout-color': 'var(--callout-failure-color, #ff5252)' },
	'[data-callout="danger"]':    { '--callout-color': 'var(--callout-danger-color, #ff1744)' },
	'[data-callout="error"]':     { '--callout-color': 'var(--callout-danger-color, #ff1744)' },
	'[data-callout="bug"]':       { '--callout-color': 'var(--callout-danger-color, #ff1744)' },
	'[data-callout="example"]':   { '--callout-color': 'var(--callout-example-color, #7c4dff)' },
	'[data-callout="quote"]':     { '--callout-color': 'var(--callout-quote-color, #9e9e9e)' },
	'[data-callout="cite"]':      { '--callout-color': 'var(--callout-quote-color, #9e9e9e)' },
});

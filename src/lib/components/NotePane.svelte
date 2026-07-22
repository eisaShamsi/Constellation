<script lang="ts">
	/**
	 * NotePane — The core note editor.
	 * Gray desk + breadcrumb + white paper + title + properties + CM6 editor + persistence.
	 * Live preview decorations via shared livePreview plugin. Typing must be instant.
	 * Originally developed as eNotePane (experimental), promoted to production 2026-03-29.
	 */
	import { onMount, onDestroy, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { t, dir as uiDir } from '$lib/i18n'; // uiDir: UI-language direction for the menu's TEXT (NotePane already has a note `dir` prop)
	import { appSettings, getEffectiveScriptFonts } from '$lib/libraries/store';
	import { lookupStageEmoji, stageLabel, nextStage, prevStage } from '$lib/libraries/store';
	import type { FrontmatterProperty } from '$lib/libraries/store';
	import { stripLinkTypePrefix } from '$lib/libraries/linkTypeRegistry';
	import PropertyEditor from './PropertyEditor.svelte';
	import ContextMenu from './ContextMenu.svelte'; // MIG-077 §F-Editor — note RC uses the SHARED menu (consistent with the file tree)
	import type { MenuItem } from './contextMenuBuilder';
	import { openStyleSetterToCategory } from '$lib/stores/styleSetter'; // MIG-077 §F — RC "Style…"
	import { EditorView, keymap, drawSelection, ViewPlugin, WidgetType, Decoration, type DecorationSet, type ViewUpdate } from '@codemirror/view';
	import { EditorState, Compartment, Prec, StateField, StateEffect, RangeSetBuilder, Text } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
	import { tags } from '@lezer/highlight';
	import { defaultKeymap, history, historyKeymap, undo, redo, indentWithTab } from '@codemirror/commands';
	import { autocompletion, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
	import { search, openSearchPanel, searchKeymap, SearchQuery, setSearchQuery, findNext } from '@codemirror/search';
	import { livePreviewPlugin, livePreviewTheme, libraryPathField, setLibraryPath, notePathField, setNotePath, attachmentFolderField, setAttachmentFolder, linkTraversalMapField, setLinkTraversalMap, baseLensField } from '$lib/editor/livePreview';
	import { calloutPlugin, calloutTheme, calloutCollapseField, toggleCallout, refreshCallouts, CALLOUT_FAMILIES } from '$lib/editor/calloutPlugin';
	import { prewarmIcons } from '$lib/theme/iconOverrides';
	import { lineDecoPlugin, lineDecoTheme } from '$lib/editor/lineDecoPlugin';
	import { bidiPlugin, bidiTheme, scriptFontsField, setScriptFonts } from '$lib/editor/bidiPlugin';
	import { RTL_MOTION_ENABLED } from '$lib/editor/rtlFlag'; // PJ-106 §A1
	import { tripleClickTextOnly } from '$lib/editor/tripleClickLine'; // PJ-106 §B0
	import { logicalArrowKeymap } from '$lib/editor/rtlMotion'; // PJ-106 §A5
	import { paragraphNavKeymap, selectUnitKeymap } from '$lib/editor/paragraphNav'; // PJ-106 §B1/§B2
	import { ctrlClickSentence, sentenceSelectKeymap } from '$lib/editor/sentenceSelect'; // PJ-106 §B3
	import { paragraphDirKeys } from '$lib/editor/paragraphDir'; // PJ-106 §B4
	import { registerActiveEditor, unregisterActiveEditor } from '$lib/editor/activeEditor';
	import { takePendingLineJump } from '$lib/editor/lineJump';
	import { Highlight as HighlightExt } from '$lib/editor/markdownHighlight';
	import { createWikilinkCompletion, createTagCompletion, createSlashCompletion, createTypedLinkCompletion, createTaskDateCompletion } from '$lib/editor/completions';
	import { shortcodeCompletion } from '$lib/editor/shortcodeAutocomplete';
	import TableToolbar from './TableToolbar.svelte';
	import { parseTable, formatTable, addRow, addColumn, deleteRow, deleteColumn, setAlignment, moveRow, moveColumn, sortByColumn, type ParsedTable } from '$lib/editor/tableUtils';
	import { evaluateTableFormulas, indexToCol } from '$lib/editor/tableFormulas';

	/* Markdown syntax colors. MIG-070 §3 — per-element colour + heading weight read CSS
	   variables (written by the Style Setter onto <body>), falling back to these original
	   defaults so an unset Setter looks exactly as before. This HighlightStyle is applied to
	   the syntax token and wins over livePreviewTheme, so it MUST own the per-element colour
	   (a theme rule reading the same var was silently overridden — the §3A "only code changed"
	   bug). Sizes still live in livePreviewTheme (the highlight sets no font-size). */
	const markdownHighlightStyle = HighlightStyle.define([
		{ tag: tags.heading1, color: 'var(--h1-color, #d73a49)', fontWeight: 'var(--heading-weight, 700)' },
		{ tag: tags.heading2, color: 'var(--h2-color, #d73a49)', fontWeight: 'var(--heading-weight, 700)' },
		{ tag: tags.heading3, color: 'var(--h3-color, #d73a49)', fontWeight: 'var(--heading-weight, 600)' },
		{ tag: tags.heading4, color: 'var(--h4-color, #d73a49)', fontWeight: 'var(--heading-weight, 600)' },
		{ tag: tags.heading5, color: 'var(--h5-color, #d73a49)', fontWeight: 'var(--heading-weight, 600)' },
		{ tag: tags.heading6, color: 'var(--h6-color, #d73a49)', fontWeight: 'var(--heading-weight, 600)' },
		{ tag: tags.strong, color: 'var(--bold-color, #e36209)' },
		{ tag: tags.emphasis, color: 'var(--italic-color, #7c3aed)' },
		{ tag: tags.strikethrough, textDecoration: 'line-through', textDecorationColor: 'var(--strikethrough-color, currentColor)', textDecorationThickness: 'var(--strikethrough-thickness, auto)' },
		{ tag: tags.monospace, color: 'var(--code-normal, #16a34a)' },
		{ tag: tags.link, color: 'var(--link-color, #2563eb)' },
		/* MIG-088 §3c — Syntax tokens: the URL colour + the frontmatter-fence / meta grey become
		   Style-Setter vars (fence + meta share one var — same colour, same structural-punctuation
		   family). Byte-identical fallbacks; this HighlightStyle wins over the theme so it owns them. */
		{ tag: tags.url, color: 'var(--url-color, #0891b2)' },
		{ tag: tags.processingInstruction, color: 'var(--syntax-meta-color, #888)' }, /* frontmatter fences */
		{ tag: tags.meta, color: 'var(--syntax-meta-color, #888)' },
	]);

	const IDLE_SAVE_INTERVAL = 30_000; /* ms — periodic background save when idle */

	// Multi-color highlight decorations
	const setColorHighlights = StateEffect.define<{ ranges: { from: number; to: number; cssClass: string }[] }>();
	const colorHighlightField = StateField.define<DecorationSet>({
		create() { return Decoration.none; },
		update(decos, tr) {
			for (const e of tr.effects) {
				if (e.is(setColorHighlights)) {
					const builder = new RangeSetBuilder<Decoration>();
					const sorted = [...e.value.ranges].sort((a, b) => a.from - b.from);
					for (const r of sorted) {
						builder.add(r.from, r.to, Decoration.mark({ class: r.cssClass }));
					}
					return builder.finish();
				}
			}
			return decos.map(tr.changes);
		},
		provide: f => EditorView.decorations.from(f),
	});

	const HIGHLIGHT_TYPE_CLASSES: Record<string, string> = {
		title: 'cm-hl-title', content: 'cm-hl-content', tag: 'cm-hl-tag',
		property: 'cm-hl-property', wikilink: 'cm-hl-wikilink', semantic: 'cm-hl-semantic',
	};

	let {
		value = '',
		title = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		initialCursorPos = 0,
		initialScrollTop = 0,
		/* Phase 3: breadcrumb + properties */
		libraryName = '',
		tabId = '',
		filePath = '',
		libraryPath = '',
		properties = [] as FrontmatterProperty[],
		rawYaml = '',
		canGoBack = false,
		canGoForward = false,
		saving = false,
		/* G3 — read-only display surface (the second screen's default). When true the
		   CM6 body is non-editable (EditorState.readOnly + EditorView.editable.of(false)),
		   the title input is read-only, and PropertyEditor's disk writes are gated. The
		   view still renders live (livePreview, links, decorations). Reconfigured live
		   (compartment) so the Settings toggle takes effect without a remount. */
		readOnly = false,
		/* Phase 8: autocomplete data */
		noteNames = [] as { name: string; path: string; libraryName?: string }[],
		allTags = [] as string[],
		/* Callbacks */
		onchange,
		onDocChange,
		onsave,
		onflush,
		ontitlechange,
		onnavigateback,
		onnavigateforward,
		onmoreaction,
		onpropschange,
		onLiveProps,
		stage = '',
		onpromote,
		trail = '',
		trailIndex = 0,
		trailTotal = 0,
		onTrailPrev,
		onTrailNext,
		highlightTerm = '',
		summaryHeadline = '',
		onlinkclick,
		linkTraversalMap,
	}: {
		value?: string;
		title?: string;
		dir?: 'ltr' | 'rtl';
		initialCursorPos?: number;
		initialScrollTop?: number;
		libraryName?: string;
		tabId?: string;
		filePath?: string;
		libraryPath?: string;
		properties?: FrontmatterProperty[];
		rawYaml?: string;
		canGoBack?: boolean;
		canGoForward?: boolean;
		saving?: boolean;
		readOnly?: boolean;
		noteNames?: { name: string; path: string; libraryName?: string }[];
		allTags?: string[];
		onchange?: (value: string) => void;
		/* MIG-076 §C — O(1) live push of the CM6 doc to the note model on every
		   change. Passes the immutable Text rope (no toString()), so the model
		   stays current per-keystroke without paying the hot-path cost `onchange`
		   deliberately avoids. */
		onDocChange?: (doc: Text) => void;
		onsave?: (value: string, filePath: string) => void;
		onflush?: (text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number, filePath: string) => void;
		ontitlechange?: (newTitle: string, filePath: string) => void;
		onnavigateback?: () => void;
		onnavigateforward?: () => void;
		onmoreaction?: (action: string) => void;
		onpropschange?: () => void;
		/* MIG-087 §E (item 2) — forwarded to the embedded PropertyEditor; one-way
		   live props-count observer for the host's status-bar count. */
		onLiveProps?: (tabId: string, count: number) => void;
		stage?: string;
		onpromote?: (nextStage: string) => void;
		trail?: string;
		trailIndex?: number;
		trailTotal?: number;
		onTrailPrev?: () => void;
		onTrailNext?: () => void;
		highlightTerm?: string;
		summaryHeadline?: string;
		onlinkclick?: (link: string, newTab?: boolean) => void;
		linkTraversalMap?: Map<string, number>;
	} = $props();

	let titleValue = $state(title);
	// Re-sync when the title prop changes from outside (rename from parent,
	// frontmatter title edited elsewhere, second-screen update). Skip while
	// the user is actively editing the input — we don't want to stomp
	// their typing. Without this, titleValue was captured once at mount
	// and subsequent renames displayed a stale title in the big heading
	// even though Properties showed the correct current title.
	$effect(() => {
		if (titleEl && document.activeElement === titleEl) return;
		if (titleValue !== title) titleValue = title;
	});
	// MIG-014 §2D fix — stage is the PARENT; every UI surface that touches it
	// is a subfunction that DERIVES, not a holder of its own copy. Per Eisa's
	// architectural directive 2026-05-06 ("Enough patching"). The prior
	// `$state(stage?.toLowerCase() ?? '')` held a local copy that drifted from
	// Properties + file tree across handlePromote / handlePromote-from-disk /
	// onstagechange paths — three patches couldn't keep all three surfaces
	// in sync because each was its own source of truth. Now currentStage
	// derives from the `stage` prop, which itself derives from the on-disk
	// `stage:` frontmatter via NoteEditor.parsed. One source, one update path.
	let currentStage = $derived(stage?.toLowerCase() ?? '');
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	let latestText = value;
	let dirty = false;
	let idleSaveTimer: ReturnType<typeof setInterval> | null = null;
	let debouncedSaveTimer: ReturnType<typeof setTimeout> | null = null;
	let rafHandle: number | null = null;
	let checkboxHandler: EventListener | null = null;
	let chevronHandler: EventListener | null = null;
	let linkClickHandler: EventListener | null = null;
	const dirCompartment = new Compartment();
	const rtlMotionCompartment = new Compartment(); // PJ-106 §A1 — perLineTextDirection rollback lever
	const livePreviewCompartment = new Compartment();
	const typedLinkModeCompartment = new Compartment();
	// G3 — read-only compartment so the Settings toggle flips editability live
	// (reconfigure, no remount → cursor/scroll preserved).
	const readOnlyCompartment = new Compartment();
	let livePreviewEnabled = $state(true);

	/** §E.2 — the content-DOM classes that drive typed-link display in the editor:
	 *  `cm-lt-labels` shows each type's name above its link; `cm-lt-plain` reverts
	 *  to the standard wikilink colour (colour-by-type off). Driven by appSettings. */
	function typedLinkModeClass(colour: boolean, labels: boolean): string {
		const c: string[] = [];
		if (labels) c.push('cm-lt-labels');
		if (!colour) c.push('cm-lt-plain');
		return c.join(' ');
	}

	/* ─── Table toolbar state ─── */
	let tableToolbarX = $state(0);
	let tableToolbarY = $state(0);
	let currentTable = $state<ParsedTable | null>(null);
	let tableToolbarVisible = $derived(currentTable !== null);

	/* ─── Phase 3 state ─── */
	let propsCollapsed = $state(true);
	let showMoreMenu = $state(false);
	let moreMenuEl: HTMLDivElement | undefined;
	// RTL truncation fix (2026-07-18) — the ⋯ dropdown is FIXED-positioned (coords below) so it
	// escapes .e-desk's overflow-x:hidden clip that truncated its items in RTL. Mirrors the shared
	// ContextMenu's fixed + measure-then-clamp approach.
	let bcMenuEl = $state<HTMLDivElement | undefined>();
	let bcMenuTop = $state(0);
	let bcMenuLeft = $state(0);
	const hasHistory = $derived(canGoBack || canGoForward);
	const propsMode = $derived($appSettings.propertiesInDocument ?? 'visible');

	/* ─── More menu ─── */
	function toggleMoreMenu(e: MouseEvent) {
		showMoreMenu = !showMoreMenu;
		if (showMoreMenu) {
			// Anchor the fixed dropdown under the ⋯ button, then measure + clamp to the viewport so
			// it never overflows a screen edge or gets clipped by an ancestor's overflow (the RTL bug).
			const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
			bcMenuTop = rect.bottom + 4;
			bcMenuLeft = Math.max(8, rect.right - 220); // provisional (right-align) until measured
			tick().then(() => {
				if (!bcMenuEl) return;
				const w = bcMenuEl.offsetWidth;
				const h = bcMenuEl.offsetHeight;
				// Anchor by the NOTE's direction — that is what decides which side the ⋯ button sits
				// on (an RTL note mirrors the breadcrumb, putting ⋯ on the left). Always open INTO the
				// note: left-align when the button is at the note's left edge, right-align when it's at
				// the right edge. Viewport-overflow anchoring was WRONG — with the sidebar open the
				// button is far from the screen edge, so "right-align" stopped flipping and the menu
				// spilled over the file tree (Boss-reported: Latin UI + Arabic note). The clamp stays
				// as a pure safety net for extreme window sizes.
				let left = dir === 'rtl' ? rect.left : rect.right - w;
				left = Math.max(8, Math.min(left, window.innerWidth - w - 8));
				let top = rect.bottom + 4;
				if (top + h > window.innerHeight - 8) top = Math.max(8, rect.top - h - 4);
				bcMenuLeft = left;
				bcMenuTop = top;
			});
			setTimeout(() => window.addEventListener('click', closeMoreMenu, { once: true }), 0);
		}
	}
	function closeMoreMenu() { showMoreMenu = false; }
	function handleMoreAction(action: string) {
		showMoreMenu = false;
		// Caret hand-off — hand our cursor to Focus so it opens where the writer actually is,
		// instead of jumping to the top (Smooth Transitions). Set BEFORE emitting, so the layout's
		// switchToFocus handler can consume it. Same body coordinates on both surfaces.
		if (action === 'switchToFocus') {
			// Caret continuity — flush NOW so our cursor lands in `tab.cursorPos` (the app's EXISTING
			// per-tab cursor memory: doFlush → onflush → NoteEditor writes ct.cursorPos) BEFORE Focus
			// mounts and reads it. Synchronous, so it never races teardown. Same path teardown uses —
			// not a new mechanism.
			doFlush();
		}
		onmoreaction?.(action);
	}

	/* ─── Callout-aware Enter: exit empty blockquote lines ─── */
	// When cursor is on a line that is only "> " or ">", pressing Enter should
	// exit the callout rather than continue adding empty ">" lines.
	function calloutExitOnEnter(view: EditorView): boolean {
		const { state } = view;
		const range = state.selection.main;
		if (!range.empty) return false;
		const line = state.doc.lineAt(range.head);
		if (!/^>\s*$/.test(line.text)) return false;
		// Replace the empty blockquote line with an empty line (exit callout)
		view.dispatch({
			changes: { from: line.from, to: line.to, insert: '' },
			selection: { anchor: line.from },
		});
		return true;
	}


	/* ─── Background save ─── */
	// Snapshot of the file this NotePane instance is editing, captured at
	// mount. The `filePath` prop updates reactively when the parent swaps
	// to a different tab — during the {#key}-triggered destroy there's a
	// brief window where `filePath` already points to the NEW tab but this
	// editor's doc still holds the OLD note's body. Using the live prop at
	// that point corrupts the target file's write-ahead buffer and (if
	// dirty) its on-disk content. Route through `mountedFilePath` so the
	// save always lands on the note this editor was actually editing.
	let mountedFilePath = filePath ?? '';
	function doSave() {
		if (!dirty) return;
		dirty = false;
		// Snapshot text here — the one place we pay the O(N) toString() cost,
		// at most once per autosave cycle (1.5s), never on individual keystrokes.
		if (view) latestText = view.state.doc.toString();
		onsave?.(latestText, mountedFilePath);
	}
	function doFlush() {
		if (view) latestText = view.state.doc.toString();
		const cursorPos = view ? view.state.selection.main.head : 0;
		const scrollTop = view ? view.scrollDOM.scrollTop : 0;
		onflush?.(latestText, dirty, cursorPos, scrollTop, mountedFilePath);
	}
	function handleVisibilityChange() { if (document.hidden && dirty) doSave(); }
	function handleBeforeUnload() { doFlush(); }

	/* ─── Table toolbar ─── */
	function updateTableToolbar(editorView: EditorView) {
		const pos = editorView.state.selection.main.head;
		const table = parseTable(editorView.state, pos);
		if (table) {
			currentTable = table;
			const firstLine = editorView.state.doc.line(table.startLine);
			const lastLine = editorView.state.doc.line(table.endLine);
			const startCoords = editorView.coordsAtPos(firstLine.from);
			const endCoords = editorView.coordsAtPos(lastLine.to);
			if (startCoords && endCoords) {
				tableToolbarX = (startCoords.left + endCoords.right) / 2;
				tableToolbarY = startCoords.top - 44;
			}
		} else {
			currentTable = null;
		}
	}

	function applyTableChange(newTable: ParsedTable | null) {
		if (!view || !newTable || !currentTable) return;
		const startLine = view.state.doc.line(currentTable.startLine);
		const endLine = view.state.doc.line(currentTable.endLine);
		view.dispatch({ changes: { from: startLine.from, to: endLine.to, insert: formatTable(newTable.rows, newTable.alignments) } });
		currentTable = newTable;
		view.focus();
	}

	function insertFormulaAtCursor(editorView: EditorView, table: ParsedTable) {
		const col = table.cursorCol;
		const formula = `=SUM(${indexToCol(col)}1:${indexToCol(col)}${table.rows.length - 1})`;
		const newRows = table.rows.map(r => [...r]);
		newRows[table.cursorRow][col] = formula;
		applyTableChange({ ...table, rows: newRows });
	}

	/** Map a rows[] index to a document line number, skipping the separator line */
	function tableRowToLineNum(table: ParsedTable, row: number): number {
		/* row 0 = header = startLine. row 1+ = data lines after separator. */
		return row === 0 ? table.startLine : table.separatorLineNum + row;
	}

	/** Move cursor to the given cell (row, col) */
	function moveCursorToCell(editorView: EditorView, table: ParsedTable, row: number, col: number) {
		const lineNum = tableRowToLineNum(table, row);
		if (lineNum < 1 || lineNum > editorView.state.doc.lines) return;
		const line = editorView.state.doc.line(lineNum);
		/* Find the col-th cell by counting pipes */
		const startsWithPipe = line.text.trimStart().startsWith('|');
		const targetPipe = col + (startsWithPipe ? 1 : 0); /* pipe index before target cell */
		let pipeCount = 0;
		let offset = startsWithPipe ? 2 : 0; /* default: after first pipe + space */
		for (let i = 0; i < line.text.length; i++) {
			if (line.text[i] === '|') {
				pipeCount++;
				if (pipeCount > targetPipe) break;
				offset = i + 2; /* after pipe + space */
			}
		}
		editorView.dispatch({ selection: { anchor: line.from + Math.min(offset, line.text.length) } });
	}

	function tableTab(editorView: EditorView): boolean {
		const table = parseTable(editorView.state, editorView.state.selection.main.head);
		if (!table) return false;
		let nextRow = table.cursorRow;
		let nextCol = table.cursorCol + 1;
		if (nextCol >= table.columnCount) {
			nextCol = 0;
			nextRow++;
			if (nextRow >= table.rows.length) {
				applyTableChange(addRow(table, table.rows.length - 1));
				return true;
			}
		}
		moveCursorToCell(editorView, table, nextRow, nextCol);
		return true;
	}

	function tableShiftTab(editorView: EditorView): boolean {
		const table = parseTable(editorView.state, editorView.state.selection.main.head);
		if (!table) return false;
		let prevRow = table.cursorRow;
		let prevCol = table.cursorCol - 1;
		if (prevCol < 0) {
			prevCol = table.columnCount - 1;
			prevRow--;
			if (prevRow < 0) return true; /* already at first cell */
		}
		moveCursorToCell(editorView, table, prevRow, prevCol);
		return true;
	}

	/* ─── Autocomplete (shared factories from $lib/editor/completions) ─── */
	const wikilinkCompletion = createWikilinkCompletion(() => noteNames);
	const tagCompletion = createTagCompletion(() => allTags);
	const slashCompletion = createSlashCompletion();
	const typedLinkCompletion = createTypedLinkCompletion(); // CE Phase 1: [[note|type]]
	// MIG-080 §C.2 — natural-language task due-date autosuggest (@today / bare keyword → 📅 date).
	const taskDateCompletion = createTaskDateCompletion(() => $appSettings.naturalLanguageTaskDates ?? true);

	/**
	 * MIG-103 D2 — the "Start from a template…" door shown inside an empty note.
	 *
	 * Built as a DOM element (not a Svelte component) because CM6's `placeholder`
	 * owns its lifecycle: it mounts this while the doc is empty and discards it on
	 * the first character. The container ignores pointer events so clicking the
	 * blank area still just places the cursor; only the button is clickable.
	 * Read-only hosts get nothing — a display never starts a note.
	 */
	function templateDoorElement(): HTMLElement {
		const wrap = document.createElement('span');
		wrap.className = 'e-tpl-door';
		const btn = document.createElement('button');
		btn.type = 'button';
		btn.className = 'e-tpl-door-btn';
		btn.textContent = get(t)('templates.startFromTemplate');
		btn.addEventListener('mousedown', (ev) => {
			// mousedown, not click: CM6 would otherwise take focus and place the
			// cursor first, dismissing the door before the click lands.
			ev.preventDefault();
			ev.stopPropagation();
			document.dispatchEvent(new CustomEvent('constellation:apply-template-here', {
				detail: { path: filePath },
			}));
		});
		wrap.appendChild(btn);
		return wrap;
	}

	/**
	 * Is the body BLANK — empty, or nothing but whitespace?
	 *
	 * CM6's own `placeholder` extension was the first attempt and it never fired:
	 * its condition is literally `doc.length ? none : placeholder`, and a freshly
	 * created note's body is `"\n"` — the blank line after the closing `---` — so
	 * length is 1 and the placeholder is never shown (Boss-reported 2026-07-21;
	 * diagnosed by reading the note's bytes and the library's source, not guessed).
	 *
	 * Deliberately NOT fixed by trimming what `create_note` writes: content
	 * handling stays byte-exact (MIG-101 §A0) and a UI affordance is never a reason
	 * to change what lands on disk.
	 *
	 * O(1) on any real note: the string is only materialised when the document is
	 * already tiny, so this never costs anything on the keystroke path (Rule 1).
	 */
	function isBlankBody(state: EditorState): boolean {
		const len = state.doc.length;
		if (len === 0) return true;
		if (len > 8) return false; // a blank body is a handful of whitespace chars at most
		return state.doc.toString().trim() === '';
	}

	/** The door, as a CM6 widget. One instance per editor; `toDOM` builds fresh. */
	class TemplateDoorWidget extends WidgetType {
		toDOM(): HTMLElement { return templateDoorElement(); }
		// Let the button's own listener handle clicks; the editor stays out of it.
		ignoreEvent(): boolean { return true; }
	}

	/**
	 * Shows the template door while the body is blank, and removes it the moment
	 * anything is typed. Rebuilds ONLY when blankness actually flips — not on every
	 * keystroke — so a burst of typing costs one comparison per change and nothing
	 * else. Read-only hosts never get the door: a display never starts a note.
	 */
	function templateDoorExtension() {
		const doorDeco = Decoration.set([
			Decoration.widget({ widget: new TemplateDoorWidget(), side: 1 }).range(0),
		]);
		const visible = (state: EditorState) => !readOnly && isBlankBody(state);
		return ViewPlugin.fromClass(
			class {
				decorations: DecorationSet;
				private wasVisible: boolean;
				constructor(view: EditorView) {
					this.wasVisible = visible(view.state);
					this.decorations = this.wasVisible ? doorDeco : Decoration.none;
				}
				update(u: ViewUpdate) {
					if (!u.docChanged) return;
					const now = visible(u.state);
					if (now !== this.wasVisible) {
						this.wasVisible = now;
						this.decorations = now ? doorDeco : Decoration.none;
					}
				}
			},
			{ decorations: (v) => v.decorations },
		);
	}

	/* ─── Mount ─── */
	onMount(() => {
		const state = EditorState.create({
			doc: value,
			extensions: [
				history(),
				drawSelection(),
				// MIG-103 D2 (Boss-ruled 2026-07-21) — the template door, INSIDE the note.
				// New Note stays blank and instant (D1: that gesture is the CAPTURE
				// gesture), but a bare empty surface with no pathway is itself a
				// defect — so an empty body offers a quiet, ignorable way in.
				// CM6's own `placeholder` is exactly right: it renders only while the
				// document is empty and removes itself the instant a character is
				// typed, natively — no reactivity of ours on the keystroke path
				// (Rule 1). Only the button takes pointer events, so clicking
				// anywhere else in the empty note still just places the cursor.
				// NotePane only, deliberately: FocusPane is capture at its purest and
				// its blankness is the design (Editor Parity Rule exception).
				templateDoorExtension(),
				markdown({ base: markdownLanguage, extensions: [HighlightExt] }),
				syntaxHighlighting(markdownHighlightStyle),
				calloutCollapseField,
				livePreviewCompartment.of(livePreviewEnabled ? [livePreviewPlugin, livePreviewTheme, calloutPlugin, calloutTheme, baseLensField] : []),
				lineDecoPlugin, lineDecoTheme,
				scriptFontsField, bidiPlugin, bidiTheme, /* per-line RTL/LTR direction + cursor positioning */
				libraryPathField, notePathField, attachmentFolderField, /* image path resolution */
				linkTraversalMapField, /* P4.2: per-wikilink `×N` chip */
				...($appSettings.autoPairBrackets ? [closeBrackets()] : []),
				search({ top: true }),
				colorHighlightField,
				autocompletion({
					override: (
						$appSettings.enabledFeatures?.emojiIconPicker !== false
							? [taskDateCompletion, typedLinkCompletion, wikilinkCompletion, tagCompletion, slashCompletion, shortcodeCompletion]
							: [taskDateCompletion, typedLinkCompletion, wikilinkCompletion, tagCompletion, slashCompletion]
					),
					activateOnTyping: true,
					// MIG-067 — the dropdown re-builds + bidi-lays-out this many rows
					// every keystroke; on a mixed-script library that DOM work (not the
					// note search, measured at 0.12 ms) is the autocomplete lag. Fewer
					// rendered rows = lighter redraw; scroll still reaches the rest.
					maxRenderedOptions: 8,
				}),
				// Prec.highest: runs before @codemirror/lang-markdown's built-in
				// blockquote-continue keymap (which auto-adds "> " on every Enter).
				Prec.highest(keymap.of([{ key: 'Enter', run: calloutExitOnEnter }])),
				keymap.of([
					{ key: 'Tab', run: tableTab },
					{ key: 'Shift-Tab', run: tableShiftTab },
					indentWithTab,
					...defaultKeymap, ...historyKeymap, ...($appSettings.autoPairBrackets ? closeBracketsKeymap : []), ...searchKeymap,
				]),
				readOnlyCompartment.of(readOnly ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : []),
				/* PJ-106 §A1 (SI2-1) — ONE authority for the base direction: editor + content
				   attributes both carry the note's RESOLVED base (detectDir → 'rtl'|'ltr', never
				   the viewport-first-strong 'auto' that competed with the motion engine). */
				dirCompartment.of([
					EditorView.editorAttributes.of({ dir: dir === 'rtl' ? 'rtl' : 'ltr' }),
					EditorView.contentAttributes.of({ dir: dir === 'rtl' ? 'rtl' : 'ltr' }),
				]),
				/* PJ-106 — the RTL MOTION bundle, in ONE compartment (a Compartment may appear
				   only once in a configuration) so RTL_MOTION_ENABLED=false strips the whole
				   feature and restores the pre-PJ-106 motion byte-for-byte:
				     §A1 — tell the CARET/SELECTION engine to read per-line direction (the
				           bidiPlugin already RENDERS it; this connects it to MOTION).
				     §A5 — Word-style LOGICAL arrows. The skip source keeps the caret OUT of a
				           collapsed lens block's hidden source (design-inspection H3) WITHOUT
				           touching the global atomicRanges facet, which also feeds Backspace. */
				rtlMotionCompartment.of(
					RTL_MOTION_ENABLED
						? [
								EditorView.perLineTextDirection.of(true),
								logicalArrowKeymap((s) => s.field(baseLensField, false) ?? null),
								paragraphNavKeymap(), // PJ-106 §B1 — Ctrl+↑/↓ paragraph navigation
								selectUnitKeymap(), // PJ-106 §B2 — Ctrl+L line / Ctrl+Shift+L paragraph
								ctrlClickSentence, // PJ-106 §B3 — Ctrl+click selects the sentence
								sentenceSelectKeymap(), // PJ-106 §B3 — Ctrl+Shift+S select sentence at caret
								paragraphDirKeys(), // PJ-106 §B4 — Right/Left-Ctrl+Shift paragraph direction
							]
						: [],
				),
				tripleClickTextOnly, // PJ-106 §B0 — triple-click selects text, not the trailing newline
				typedLinkModeCompartment.of(EditorView.contentAttributes.of({
					class: typedLinkModeClass($appSettings.colourTypedLinks, $appSettings.showTypedLinkLabels),
				})),
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						// ⚡ PERF: do NOT call doc.toString() here — it's O(N) on every keystroke
						// and causes progressive lag as the document grows.
						// latestText is refreshed in doSave/doFlush at the moment of writing.
						dirty = true;
						onchange?.('');
						// MIG-076 §C — O(1) live push of the doc rope to the note model
						// (no toString()). Keeps single-ownership current per change so a
						// view that mounts mid-edit always seeds from fresh content.
						onDocChange?.(update.state.doc);
						// Debounced save: 1500ms after last keystroke.
						// Ensures content is saved even if the idle timer (30s) hasn't fired.
						if (debouncedSaveTimer) clearTimeout(debouncedSaveTimer);
						debouncedSaveTimer = setTimeout(() => { debouncedSaveTimer = null; doSave(); }, 1500);
					}
					// Table toolbar: only on cursor move, NOT on every keystroke (parseTable is expensive)
					if (update.selectionSet && !update.docChanged) {
						updateTableToolbar(update.view);
					}
					// Toolbar direction: lightweight check on selection or doc change
					if (update.selectionSet || update.docChanged) {
						const curLine = update.state.doc.lineAt(update.state.selection.main.head);
						let detectedDir: 'ltr' | 'rtl' | null = null;
						if (curLine.text.trim()) {
							detectedDir = RTL_DETECT.test(curLine.text) ? 'rtl' : 'ltr';
						} else {
							for (let n = curLine.number - 1; n >= 1; n--) {
								const prev = update.state.doc.line(n).text;
								if (prev.trim()) { detectedDir = RTL_DETECT.test(prev) ? 'rtl' : 'ltr'; break; }
							}
						}
						if (detectedDir) toolbarDir = detectedDir;
					}
				}),
				EditorView.theme({
					'&': { background: 'transparent', border: 'none', outline: 'none' },
					'&.cm-focused': { outline: 'none' },
					'.cm-scroller': { overflow: 'auto', fontFamily: 'inherit', fontSize: 'var(--font-text-size, 16px)', lineHeight: 'var(--line-height-normal, 1.75)' },
					// MIG-070 §3 — the NOTE body reads its own --editor-text-color (Style Setter "Body
				// text → Text colour"), falling back to --text-normal so unset = unchanged. This is
				// what keeps note styling from bleeding into the chrome (file tree etc., which use
				// --text-normal): the note has its own knob, the Interface keeps --text-normal.
				'.cm-content': { padding: '0', color: 'var(--editor-text-color, var(--text-normal, #1a1a1a))', caretColor: 'var(--caret-color, var(--text-normal, #1a1a1a))' },
					'.cm-cursor': { borderLeftColor: 'var(--caret-color, var(--text-normal, #1a1a1a))', borderLeftWidth: '1.5px' },
					'.cm-line': { padding: '0' },
					'.cm-activeLine': { background: 'transparent' },
					'.cm-activeLineGutter': { display: 'none' },
					'.cm-gutters': { display: 'none' },
					'.cm-selectionBackground': { background: 'var(--text-selection, color-mix(in srgb, var(--interactive-accent, #7c3aed) 20%, transparent))' },
					'.cm-searchMatch': {
						backgroundColor: 'color-mix(in srgb, var(--interactive-accent, #7c3aed) 25%, transparent)',
						outline: '1px solid color-mix(in srgb, var(--interactive-accent, #7c3aed) 50%, transparent)',
						borderRadius: '2px',
					},
					'.cm-searchMatch-selected': {
						backgroundColor: 'color-mix(in srgb, var(--interactive-accent, #7c3aed) 40%, transparent)',
						outline: '2px solid var(--interactive-accent, #7c3aed)',
						borderRadius: '2px',
					},
					/* Hide search panel — we only want the highlights, not the UI */
					'.cm-search.cm-panel': { display: 'none' },
					/* Multi-color search match highlights per type */
					/* MIG-088 §2e — shared Match-category colours (Style Setter → Cognitive colours); fallback = today's value. */
					'.cm-hl-title': { backgroundColor: 'color-mix(in srgb, var(--match-category-title, #3b82f6) 25%, transparent)', outline: '1px solid var(--match-category-title, #3b82f6)', borderRadius: '2px' },
					'.cm-hl-content': { backgroundColor: 'color-mix(in srgb, var(--match-category-content, #16a34a) 25%, transparent)', outline: '1px solid var(--match-category-content, #16a34a)', borderRadius: '2px' },
					'.cm-hl-tag': { backgroundColor: 'color-mix(in srgb, var(--match-category-tag, #f472b6) 25%, transparent)', outline: '1px solid var(--match-category-tag, #f472b6)', borderRadius: '2px' },
					'.cm-hl-property': { backgroundColor: 'color-mix(in srgb, var(--match-category-property, #f59e0b) 25%, transparent)', outline: '1px solid var(--match-category-property, #f59e0b)', borderRadius: '2px' },
					'.cm-hl-wikilink': { backgroundColor: 'color-mix(in srgb, var(--match-category-wikilink, #60a5fa) 25%, transparent)', outline: '1px solid var(--match-category-wikilink, #60a5fa)', borderRadius: '2px' },
					'.cm-hl-semantic': { backgroundColor: 'color-mix(in srgb, var(--match-category-semantic, #7c3aed) 25%, transparent)', outline: '1px solid var(--match-category-semantic, #7c3aed)', borderRadius: '2px' },
				}),
			],
		});

		// Pre-populate the image-path state fields BEFORE the view is created so
		// the ViewPlugin constructor's initial `buildDecorations` sees the
		// correct paths. If we dispatch these effects after view creation, the
		// first render's ImageWidgets are built with empty paths and either
		// 404-flood against the dev origin (if their filenames are relative)
		// or silently fall back to placeholders that never re-resolve — state
		// field changes alone don't trigger a decoration rebuild.
		const imgEffects: any[] = [];
		if (libraryPath) imgEffects.push(setLibraryPath.of(libraryPath));
		if (filePath) imgEffects.push(setNotePath.of(filePath));
		imgEffects.push(setAttachmentFolder.of($appSettings.defaultAttachmentFolder || ''));
		if (linkTraversalMap) imgEffects.push(setLinkTraversalMap.of(linkTraversalMap));
		const initialState = imgEffects.length
			? state.update({ effects: imgEffects }).state
			: state;

		try {
			view = new EditorView({ state: initialState, parent: editorEl! });
		} catch (e) {
			// Fallback: create editor without livePreview if decorations fail
			// (e.g., RangeError on content with line-spanning replace decorations)
			console.warn('[NotePane] Editor init failed, retrying without livePreview:', e);
			const fallbackState = EditorState.create({
				doc: value,
				extensions: [
					history(),
					keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab, ...closeBracketsKeymap, ...searchKeymap]),
					drawSelection(),
					markdown({ base: markdownLanguage }),
					syntaxHighlighting(markdownHighlightStyle),
					EditorView.lineWrapping,
					search(),
					colorHighlightField,
				],
			});
			view = new EditorView({ state: fallbackState, parent: editorEl! });
		}

		/* Caret hand-off — if we're coming BACK from Focus mode on this same note, restore the
		   cursor where Focus left it, so the round-trip is seamless. Single-use: on an ordinary
		   note open the slot is empty and this is a no-op. Clamped, since the body may have grown
		   or shrunk while in Focus. */
		// Caret continuity — deliberately NO cursor restore here. The app's own per-tab cursor memory
		// (`initialCursorPos`, applied further below in the saved-cursor branch) is the SINGLE
		// mechanism. A second restore here was tried and it FOUGHT the real one and lost.

		/* Highlight term(s) — supports multi-term comma-separated (,،、) */
		if (highlightTerm && view) {
			// Split by comma variants
			const terms = highlightTerm.split(/[,،、]/).map(s => s.trim()).filter(s => s.length > 0);
			const d = '[\u064B-\u065F\u0670]*'; // optional Arabic diacritics class
			const termPatterns = terms.map(term => {
				const isArabic = /[\u0600-\u06FF]/.test(term);
				if (isArabic) {
					const expanded = term.split('').map(ch => {
						if (ch === 'ه') return `[هة]${d}`;
						if (ch === 'ا') return `[اأإآٱ]${d}`;
						if (ch === 'ي') return `[يى]${d}`;
						return ch + d;
					}).join('');
					return `(?:ال)?${expanded}`;
				}
				return term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
			});
			const pattern = termPatterns.join('|');
			const q = new SearchQuery({ search: pattern, caseSensitive: false, regexp: true });
			view.dispatch({ effects: setSearchQuery.of(q) });
			// Open search panel to activate highlights (panel is hidden via CSS)
			openSearchPanel(view);
			// Scroll to first occurrence — needs delay for editor to fully render
			setTimeout(() => {
				if (!view) return;
				findNext(view);
				// Ensure the match is scrolled into view
				setTimeout(() => {
					if (!view) return;
					const sel = view.state.selection.main;
					if (sel.from !== sel.to) {
						view.dispatch({ effects: EditorView.scrollIntoView(sel.from, { y: 'center' }) });
					}
				}, 50);
			}, 300);

				// Multi-color decorations: scan doc and classify each match
				setTimeout(() => {
					if (!view) return;
					const doc = view.state.doc.toString();
					const re = new RegExp(pattern, 'gi');
					const ranges: { from: number; to: number; cssClass: string }[] = [];
					let m;
					// Find YAML frontmatter boundary
					let fmEnd = 0;
					if (doc.startsWith('---')) {
						const idx = doc.indexOf('---', 3);
						if (idx > 0) fmEnd = idx + 3;
					}
					// First heading end (title area)
					const firstHeading = doc.match(/^#{1,6}\s+.+$/m);
					const titleEnd = firstHeading ? (doc.indexOf(firstHeading[0]) + firstHeading[0].length) : 0;

					while ((m = re.exec(doc)) !== null) {
						const pos = m.index;
						const line = doc.substring(Math.max(0, doc.lastIndexOf('\n', pos)), doc.indexOf('\n', pos + 1) || doc.length);
						let cssClass = 'cm-hl-content'; // default: green
						if (pos <= titleEnd && titleEnd > 0) {
							cssClass = 'cm-hl-title'; // blue: in title
						} else if (pos < fmEnd) {
							if (line.includes('#')) cssClass = 'cm-hl-tag'; // pink: tag in frontmatter
							else cssClass = 'cm-hl-property'; // amber: property in frontmatter
						} else if (/\[\[.*\]\]/.test(line)) {
							cssClass = 'cm-hl-wikilink'; // light blue: in wikilink line
						} else if (/#\S/.test(line)) {
							cssClass = 'cm-hl-tag'; // pink: inline tag
						}
						ranges.push({ from: pos, to: pos + m[0].length, cssClass });
					}
					if (ranges.length > 0) {
						view.dispatch({ effects: setColorHighlights.of({ ranges }) });
					}
				}, 500);
			}

		/* Checkbox toggle — capture phase, O(1) via posAtCoords */
		checkboxHandler = ((event: MouseEvent) => {
			const target = event.target as HTMLElement;
			if (!(target.tagName === 'INPUT' && target.classList.contains('cm-md-checkbox'))) return;
			event.preventDefault();
			event.stopPropagation();
			if (!view) return;
			const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
			if (pos == null) return;
			const line = view.state.doc.lineAt(pos);
			const m = line.text.match(/^(\s*[-*+]\s)\[( |x|X)\]/);
			if (!m) return;
			const checkStart = line.from + m[1].length + 1;
			const newChar = (m[2] === ' ') ? 'x' : ' ';
			view.dispatch({ changes: { from: checkStart, to: checkStart + 1, insert: newChar } });
		}) as EventListener;
		editorEl!.addEventListener('mousedown', checkboxHandler, true);

		/* Callout chevron toggle — capture phase */
		chevronHandler = ((event: MouseEvent) => {
			const target = event.target as HTMLElement;
			const chevron = target.closest?.('.cm-callout-chevron') as HTMLElement | null;
			if (!chevron || !chevron.dataset.calloutLine || !view) return;
			event.preventDefault();
			event.stopPropagation();
			const lineNum = parseInt(chevron.dataset.calloutLine, 10);
			if (isNaN(lineNum)) return;

			const state = view.state;
			// Determine if this click will collapse (new isCollapsed = true)
			const collapseSet = state.field(calloutCollapseField, false);
			const currentlyToggled = collapseSet?.has(lineNum) ?? false;
			const titleText = state.doc.line(lineNum).text;
			const foldMatch = titleText.match(/^>\s*\[!\w+\]([+-])/);
			const defaultCollapsed = foldMatch?.[1] === '-';
			const willCollapse = currentlyToggled ? !defaultCollapsed : defaultCollapsed;

			// Find the callout end line
			let endLine = lineNum;
			for (let l = lineNum + 1; l <= Math.min(state.doc.lines, lineNum + 200); l++) {
				const t = state.doc.line(l).text;
				if (!/^>\s?/.test(t) || /^>\s*\[!\w+\]/.test(t)) break;
				endLine = l;
			}

			// If collapsing with cursor inside body: move cursor to after callout
			// in the SAME transaction so buildCalloutDecorations sees cursor outside.
			const cursorLine = state.doc.lineAt(state.selection.main.head).number;
			const cursorInBody = cursorLine > lineNum && cursorLine <= endLine;
			const moveSelection = (willCollapse && cursorInBody && endLine < state.doc.lines)
				? { anchor: state.doc.line(endLine + 1).from }
				: undefined;

			view.dispatch({ effects: toggleCallout.of(lineNum), selection: moveSelection });
		}) as EventListener;
		editorEl!.addEventListener('mousedown', chevronHandler, true);

		/* Wikilink / Markdown link click — uses mousedown (before CM6 moves cursor and strips decorations) */
		linkClickHandler = ((event: MouseEvent) => {
			if (!view) return;
			// Only left-click
			if (event.button !== 0) return;

			const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
			if (pos === null) return;
			const line = view.state.doc.lineAt(pos);
			const offset = pos - line.from;

			// Check for wikilink [[target|alias]] or [[target]]
			const wikiRe = /\[\[([^\]]+)\]\]/g;
			let match;
			while ((match = wikiRe.exec(line.text)) !== null) {
				// Click must be inside the inner text (between [[ and ]])
				const innerStart = match.index + 2;
				const innerEnd = match.index + match[0].length - 2;
				if (offset >= innerStart && offset <= innerEnd) {
					event.preventDefault();
					event.stopPropagation();
					// MIG-067 — predicate-first [[type::target]]: strip the known type:: prefix
					// so the click opens the TARGET, not a note named "type::target".
					const link = stripLinkTypePrefix(match[1]).split('|')[0].split('#')[0].trim();
					const newTab = event.ctrlKey || event.metaKey;
					if (onlinkclick) {
						onlinkclick(link, newTab);
					}
					return;
				}
			}

			// Check for markdown link [text](url) — only on Ctrl+Click
			if (event.ctrlKey || event.metaKey) {
				const mdRe = /\[([^\]]+)\]\(([^)]+)\)/g;
				while ((match = mdRe.exec(line.text)) !== null) {
					if (offset >= match.index && offset <= match.index + match[0].length) {
						event.preventDefault();
						const url = match[2];
						if (url.startsWith('http://') || url.startsWith('https://')) {
							window.open(url, '_blank');
						} else if (onlinkclick) {
							onlinkclick(url, true);
						}
						return;
					}
				}
			}
		}) as EventListener;
		// Use mousedown instead of click — fires BEFORE CM6 processes the event
		// and strips livePreview decorations from the clicked line
		editorEl!.addEventListener('mousedown', linkClickHandler, true);

		// §A.2 — a one-shot line jump (from a calendar task dot / Tasks panel) takes priority over
		// the saved cursor/scroll. Selection-only dispatch → no doc change, no save (Gate #2).
		const pendingLine = takePendingLineJump(tabId);
		if (pendingLine && pendingLine > 0) {
			const n = Math.min(Math.max(1, Math.floor(pendingLine)), view.state.doc.lines);
			const pos = view.state.doc.line(n).from;
			view.dispatch({ selection: { anchor: pos }, effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
			view.focus();
		} else if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
			view.focus();
		} else {
			titleEl?.focus();
		}
		if (!pendingLine && initialScrollTop > 0) {
			// Single RAF — double-RAF leaked the outer handle on cleanup.
			// setTimeout(0) gives CM6 one frame to layout, then we scroll.
			rafHandle = requestAnimationFrame(() => {
				rafHandle = null;
				view?.scrollDOM.scrollTo({ top: initialScrollTop });
			});
		}

		idleSaveTimer = setInterval(() => { requestIdleCallback(() => doSave()); }, IDLE_SAVE_INTERVAL);
		document.addEventListener('visibilitychange', handleVisibilityChange);
		window.addEventListener('beforeunload', handleBeforeUnload);
		// Register this editor with the global active-editor registry so the
		// emoji/icon picker (and any future global command) can insert into it.
		view?.dom.addEventListener('focusin', onEditorFocusIn);
		if (view) registerActiveEditor(view, filePath);
	});

	function onEditorFocusIn() {
		if (view) registerActiveEditor(view, filePath);
	}


	/* ─── Destroy ─── */
	onDestroy(() => {
		if (idleSaveTimer) clearInterval(idleSaveTimer);
		if (debouncedSaveTimer) { clearTimeout(debouncedSaveTimer); debouncedSaveTimer = null; }
		document.removeEventListener('visibilitychange', handleVisibilityChange);
		window.removeEventListener('beforeunload', handleBeforeUnload);
		if (view) {
			view.dom.removeEventListener('focusin', onEditorFocusIn);
			unregisterActiveEditor(view);
		}
		if (checkboxHandler && editorEl) editorEl.removeEventListener('mousedown', checkboxHandler, true);
		if (chevronHandler && editorEl) editorEl.removeEventListener('mousedown', chevronHandler, true);
		if (linkClickHandler && editorEl) editorEl.removeEventListener('mousedown', linkClickHandler, true);
		if (rafHandle !== null) { cancelAnimationFrame(rafHandle); rafHandle = null; }
		doFlush();
		view?.destroy();
		view = null;
	});

	/* ─── Dir sync ─── */
	let prevDir = dir;
	$effect(() => {
		if (view && dir !== prevDir) {
			prevDir = dir;
			view.dispatch({ effects: dirCompartment.reconfigure([
				EditorView.editorAttributes.of({ dir: dir === 'rtl' ? 'rtl' : 'ltr' }),
				EditorView.contentAttributes.of({ dir: dir === 'rtl' ? 'rtl' : 'ltr' }),
			]) });
		}
	});

	/* ─── Traversal map sync (P4.2) ───
	 * The initial state already carries the map from mount time; this effect
	 * refreshes it when the boot graph lands (or is replaced on a Universe
	 * switch) while the note is already open. Skipping the decoration
	 * rebuild is handled inside ViewPlugin.update — a state-field-only
	 * transaction doesn't set viewportChanged / selectionSet / docChanged,
	 * so we also explicitly trigger a viewport refresh via the same
	 * transaction to get the `×N` chips re-rendered. */
	let prevTraversalMap = linkTraversalMap;
	$effect(() => {
		if (view && linkTraversalMap && linkTraversalMap !== prevTraversalMap) {
			prevTraversalMap = linkTraversalMap;
			view.dispatch({ effects: setLinkTraversalMap.of(linkTraversalMap) });
		}
	});

	/* ─── Script fonts sync (for per-line RTL font override in bidiPlugin) ─── */
	// Guard: appSettings is a Svelte store — $effect would re-run on ANY settings change.
	// Only dispatch when scriptFonts actually changes to avoid spurious DOM mutations.
	let _prevScriptFontsKey = '{}';
	$effect(() => {
		const fonts = getEffectiveScriptFonts($appSettings);
		const key = JSON.stringify(fonts);
		if (view && key !== _prevScriptFontsKey) {
			_prevScriptFontsKey = key;
			view.dispatch({ effects: setScriptFonts.of(fonts) });
		}
	});

	/* ─── Live Preview toggle ─── */
	let prevLivePreview = livePreviewEnabled;
	$effect(() => {
		if (view && livePreviewEnabled !== prevLivePreview) {
			prevLivePreview = livePreviewEnabled;
			view.dispatch({
				effects: livePreviewCompartment.reconfigure(
					livePreviewEnabled ? [livePreviewPlugin, livePreviewTheme, calloutPlugin, calloutTheme, baseLensField] : []
				)
			});
		}
	});

	/* ─── Callout customisation (MIG-089) — live repaint on a settings change ─── */
	// Callout COLOURS for BUILT-INS ride CSS vars (instant, no editor involvement).
	// ICONS (built-in + custom) are baked into the widget DOM at build time, and a
	// CUSTOM type's colour is injected inline at build time — so an icon override or a
	// custom-callout change must force a decoration rebuild. Guard on a signature of just
	// the callout settings (appSettings is a store — a bare $effect fires on every setting).
	// Warm the icon cache first so an SVG (Lucide/…) override resolves on the rebuild.
	let _prevCalloutSig: string | null = null;
	$effect(() => {
		const ov = $appSettings.iconOverrides ?? {};
		const iconSig = CALLOUT_FAMILIES.map((f) => ov['callout.' + f] ?? '').join('|');
		const custom = $appSettings.customCallouts ?? [];
		const customSig = custom.map((c) => `${c.slug}${c.color}${c.icon}`).join('');
		const sig = iconSig + '' + customSig;
		if (sig === _prevCalloutSig) return;
		const first = _prevCalloutSig === null;
		const isEmpty = !iconSig.replace(/\|/g, '') && customSig === '';
		_prevCalloutSig = sig;
		const v = view;
		if (!v) return;
		// First run with nothing custom → the default render is already correct (no work).
		if (first && isEmpty) return;
		prewarmIcons().then(() => { try { v.dispatch({ effects: refreshCallouts.of(null) }); } catch { /* view torn down */ } });
	});

	/* ─── G3 read-only toggle — reconfigure editability live ─── */
	// Guard on the boolean so the compartment only reconfigures when the Settings
	// toggle flips (appSettings is a store — a bare $effect would fire on any change).
	let _prevReadOnly = readOnly;
	$effect(() => {
		if (view && readOnly !== _prevReadOnly) {
			_prevReadOnly = readOnly;
			view.dispatch({ effects: readOnlyCompartment.reconfigure(
				readOnly ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : []
			) });
		}
	});

	/* ─── Typed-link display mode (§E.2: colour-by-type / label-above) ─── */
	// Guard on a key of just the two booleans — appSettings is a store, so a bare
	// $effect would re-run on ANY setting change; only reconfigure when these flip.
	let _prevTypedLinkModeKey = `${$appSettings.colourTypedLinks}|${$appSettings.showTypedLinkLabels}`;
	$effect(() => {
		const key = `${$appSettings.colourTypedLinks}|${$appSettings.showTypedLinkLabels}`;
		if (view && key !== _prevTypedLinkModeKey) {
			_prevTypedLinkModeKey = key;
			view.dispatch({ effects: typedLinkModeCompartment.reconfigure(EditorView.contentAttributes.of({
				class: typedLinkModeClass($appSettings.colourTypedLinks, $appSettings.showTypedLinkLabels),
			})) });
		}
	});

	/* ─── Title ─── */
	function handleTitleBlur() {
		const trimmed = titleValue.trim();
		if (!trimmed) {
			// Restore the original title — never generate a new one.
			// The title comes from the filename (compatible mode) or frontmatter (canonical mode).
			titleValue = title || filePath.split(/[\\/]/).pop()?.replace(/\.(md|base)$/, '') || $t('actions.untitled');
		}
		// Pass `mountedFilePath` (snapshotted at mount) so the parent can detect
		// a swapped tab. When a wikilink click reuses the active tab, the OLD
		// NotePane is destroyed; a final blur on its title input could otherwise
		// fire ontitlechange with a stale source-tab title while `tab.path`
		// already points to the target — corrupting the target's frontmatter
		// title via rename_item (canonical mode). The same staleness guard
		// pattern protects body saves via mountedFilePath in doSave/doFlush.
		if (titleValue !== title) ontitlechange?.(titleValue, mountedFilePath);
	}
	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') { e.preventDefault(); view?.focus(); }
	}

	const titleAlignment = $derived($appSettings.titleAlignment ?? 'center');

	/* ─── Phase 4: Toolbar helpers ─── */
	function wrapSelection(before: string, after: string) {
		if (!view) return;
		const { from, to } = view.state.selection.main;
		if (from === to) {
			view.dispatch({ changes: { from, to, insert: before + after }, selection: { anchor: from + before.length } });
		} else {
			const sel = view.state.sliceDoc(from, to);
			if (sel.startsWith(before) && sel.endsWith(after)) {
				const inner = sel.slice(before.length, -after.length);
				view.dispatch({ changes: { from, to, insert: inner }, selection: { anchor: from, head: from + inner.length } });
			} else {
				view.dispatch({ changes: { from, to, insert: before + sel + after }, selection: { anchor: from + before.length, head: from + before.length + sel.length } });
			}
		}
		view.focus();
	}
	function insertLinePrefix(prefix: string) {
		if (!view) return;
		const { from } = view.state.selection.main;
		const line = view.state.doc.lineAt(from);
		if (line.text.startsWith(prefix)) {
			view.dispatch({ changes: { from: line.from, to: line.from + prefix.length, insert: '' } });
		} else {
			view.dispatch({ changes: { from: line.from, insert: prefix } });
		}
		view.focus();
	}
	function insertAtCursor(text: string) {
		if (!view) return;
		const { from } = view.state.selection.main;
		view.dispatch({ changes: { from, insert: text }, selection: { anchor: from + text.length } });
		view.focus();
	}
	function tbUndo() { if (view) { undo(view); view.focus(); } }
	function tbRedo() { if (view) { redo(view); view.focus(); } }
	function tbFind() { if (view) openSearchPanel(view); }

	/** Strip all markdown/HTML formatting marks from selection */
	function clearFormatting() {
		if (!view) return;
		const { from, to } = view.state.selection.main;
		if (from === to) return;
		let text = view.state.sliceDoc(from, to);
		text = text
			.replace(/\*\*(.+?)\*\*/g, '$1')
			.replace(/__(.+?)__/g, '$1')
			.replace(/~~(.+?)~~/g, '$1')
			.replace(/==(.+?)==/g, '$1')
			.replace(/`(.+?)`/g, '$1')
			.replace(/_(.+?)_/g, '$1')
			.replace(/\*(.+?)\*/g, '$1')
			.replace(/<u>(.*?)<\/u>/gi, '$1')
			.replace(/<sub>(.*?)<\/sub>/gi, '$1')
			.replace(/<sup>(.*?)<\/sup>/gi, '$1')
			.replace(/<span[^>]*>(.*?)<\/span>/gi, '$1')
			.replace(/<div[^>]*>(.*?)<\/div>/gi, '$1');
		view.dispatch({ changes: { from, to, insert: text }, selection: { anchor: from, head: from + text.length } });
		view.focus();
	}

	/** Wrap current line in an alignment div, or toggle alignment */
	function setLineAlignment(align: 'left' | 'center' | 'right') {
		if (!view) return;
		const { from } = view.state.selection.main;
		const line = view.state.doc.lineAt(from);
		const text = line.text;
		// Remove existing alignment wrapper if present
		const alignRe = /^<div style="text-align:\s*(left|center|right)">(.*)<\/div>$/;
		const match = text.match(alignRe);
		if (match) {
			if (match[1] === align) {
				// Same alignment: remove wrapper (toggle off)
				view.dispatch({ changes: { from: line.from, to: line.to, insert: match[2] } });
			} else {
				// Different alignment: replace
				view.dispatch({ changes: { from: line.from, to: line.to, insert: `<div style="text-align: ${align}">${match[2]}</div>` } });
			}
		} else {
			// No wrapper: add alignment
			view.dispatch({ changes: { from: line.from, to: line.to, insert: `<div style="text-align: ${align}">${text}</div>` } });
		}
		view.focus();
	}

	// ─── RTL-aware toolbar: detect current line direction ───
	const RTL_DETECT = /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF\u0590-\u05FF\uFB1D-\uFB4F]/;
	let toolbarDir = $state<'ltr' | 'rtl'>('ltr');

	let toolbarVisible = $state($appSettings.showFloatingToolbar !== false);
	let showHeadingMenu = $state(false);
	let showListMenu = $state(false);
	let showInsertMenu = $state(false);
	function closeMenus() { showHeadingMenu = false; showListMenu = false; showInsertMenu = false; }
	function toggleMenu(menu: 'heading' | 'list' | 'insert') {
		const was = menu === 'heading' ? showHeadingMenu : menu === 'list' ? showListMenu : showInsertMenu;
		closeMenus();
		if (!was) {
			if (menu === 'heading') showHeadingMenu = true;
			else if (menu === 'list') showListMenu = true;
			else showInsertMenu = true;
			setTimeout(() => window.addEventListener('click', closeMenus, { once: true }), 0);
		}
	}

	/* ─── MIG-077 §F-Editor: the note's right-click menu (NotePane is the live editor) ─── */
	let editorCtxMenu = $state<{ x: number; y: number; onLink: boolean } | null>(null);

	// MIG-077 §F-Editor — frontmatter (Properties panel) right-click. Boss: "all the
	// above" → property actions + the editor menu + Style. NotePane owns the menu (it
	// has the editor commands + view); PropertyEditor emits the row + exposes add/remove.
	let propEditorRef = $state<any>(null);
	let fmCtxMenu = $state<{ x: number; y: number; idx: number; key: string; value: string; readOnly?: boolean } | null>(null);
	function handlePropContextMenu(prop: FrontmatterProperty, idx: number, x: number, y: number) {
		// PJ-136 — a nested-map row has no scalar value (its content lives in the CST),
		// so copy what the row actually SHOWS: its child field names. `readOnly` drops
		// the Remove item below, because the write path refuses to splice this block and
		// an action that cannot happen must not be offered.
		const isNestedMap = prop.type === 'nested-map';
		fmCtxMenu = {
			x, y, idx,
			key: prop.key ?? '',
			value: isNestedMap ? (prop.nestedKeys ?? []).join(', ') : String(prop.value ?? ''),
			readOnly: isNestedMap,
		};
	}
	function getFrontmatterMenuItems(): MenuItem[] {
		const fm = fmCtxMenu;
		const items: MenuItem[] = [];
		if (fm) {
			items.push(
				{ label: $t('contextMenu.copyValue'), icon: '📋', action: () => navigator.clipboard.writeText(fm.value).catch(() => {}) },
				{ label: $t('contextMenu.copyName'), icon: '🏷', action: () => navigator.clipboard.writeText(fm.key).catch(() => {}) },
			);
			if (!fm.readOnly) {
				items.push({ label: $t('contextMenu.removeProperty'), icon: '🗑️', danger: true, action: () => propEditorRef?.removeProperty?.(fm.idx) });
			}
			items.push(
				{ label: $t('contextMenu.addProperty'), icon: '➕', action: () => propEditorRef?.addProperty?.() },
				{ separator: true },
			);
		}
		items.push(...getEditorMenuItems(false, 'frontmatter'));
		return items;
	}

	function handleEditorContextMenu(e: MouseEvent) {
		if (!view) return;
		e.preventDefault();
		const pos = view.posAtCoords({ x: e.clientX, y: e.clientY });
		const sel = view.state.selection.main;
		const hasSelection = sel.from !== sel.to;
		// Land the caret where the user right-clicked (unless they right-clicked a selection).
		if (pos !== null && !hasSelection) view.dispatch({ selection: { anchor: pos } });
		const line = view.state.doc.lineAt(view.state.selection.main.head);
		const onLink = /\[\[[^\]]+\]\]/.test(line.text) || /\[[^\]]+\]\([^)]+\)/.test(line.text);
		editorCtxMenu = { x: e.clientX, y: e.clientY, onLink };
	}

	// MIG-077 §F-Editor — the note's right-click menu, built for the SHARED <ContextMenu>
	// (same chrome + icons + fly-out submenus as the file tree). Items reuse NotePane's
	// own commands via the ecm* helpers below.
	function getEditorMenuItems(onLink: boolean, styleCat: string = 'editor'): MenuItem[] {
		const items: MenuItem[] = [];
		if (onLink) {
			items.push(
				{ label: $t('contextMenu.openLink'), icon: '📂', action: () => ecmLinkAction('open') },
				{ label: $t('contextMenu.copyTarget'), icon: '📋', action: () => ecmLinkAction('copyTarget') },
				{ label: $t('contextMenu.editLink'), icon: '✏️', action: () => ecmLinkAction('edit') },
				{ label: $t('contextMenu.removeLink'), icon: '✖️', action: () => ecmLinkAction('remove') },
				{ separator: true },
			);
		}
		items.push(
			{ label: $t('contextMenu.link'), icon: '🔗', action: () => ecmInsert('link') },
			{ label: $t('contextMenu.externalLink'), icon: '🌐', action: () => ecmInsert('externalLink') },
			{ separator: true },
			{ label: $t('contextMenu.format'), icon: '🅰️', submenu: [
				{ label: $t('contextMenu.bold'), action: () => ecmFormat('bold') },
				{ label: $t('contextMenu.italic'), action: () => ecmFormat('italic') },
				{ label: $t('contextMenu.underline'), action: () => ecmFormat('underline') },
				{ label: $t('contextMenu.strikethrough'), action: () => ecmFormat('strikethrough') },
				{ label: $t('contextMenu.highlight'), action: () => ecmFormat('highlight') },
				{ separator: true },
				{ label: $t('contextMenu.inlineCode'), action: () => ecmFormat('code') },
				{ label: $t('contextMenu.math'), action: () => ecmFormat('math') },
				{ label: $t('contextMenu.toggleComment'), action: () => ecmFormat('toggleComment') },
				{ separator: true },
				{ label: $t('contextMenu.superscript'), action: () => ecmFormat('superscript') },
				{ label: $t('contextMenu.subscript'), action: () => ecmFormat('subscript') },
				{ label: $t('contextMenu.clearFormatting'), action: () => ecmFormat('clear') },
			] },
			{ label: $t('contextMenu.paragraph'), icon: '¶', submenu: [
				{ label: $t('contextMenu.bulletList'), action: () => ecmList('bullet') },
				{ label: $t('contextMenu.numberedList'), action: () => ecmList('numbered') },
				{ label: $t('contextMenu.taskList'), action: () => ecmList('task') },
				{ separator: true },
				{ label: 'H1', action: () => ecmHeading(1) },
				{ label: 'H2', action: () => ecmHeading(2) },
				{ label: 'H3', action: () => ecmHeading(3) },
				{ label: 'H4', action: () => ecmHeading(4) },
				{ label: 'H5', action: () => ecmHeading(5) },
				{ label: 'H6', action: () => ecmHeading(6) },
				{ label: $t('contextMenu.body'), action: () => ecmHeading(0) },
				{ separator: true },
				{ label: $t('contextMenu.blockquote'), action: () => ecmInsert('blockquote') },
			] },
			{ label: $t('contextMenu.insert'), icon: '➕', submenu: [
				{ label: $t('contextMenu.footnote'), action: () => ecmInsert('footnote') },
				{ label: $t('contextMenu.table'), action: () => ecmInsert('table') },
				{ label: $t('contextMenu.callout'), action: () => ecmInsert('callout') },
				{ label: $t('contextMenu.horizontalRule'), action: () => ecmInsert('horizontalRule') },
				{ separator: true },
				{ label: $t('contextMenu.codeBlock'), action: () => ecmInsert('codeBlock') },
				{ label: $t('contextMenu.mathBlock'), action: () => ecmInsert('mathBlock') },
				{ label: $t('contextMenu.image'), action: () => ecmInsert('image') },
			] },
			{ separator: true },
			{ label: $t('contextMenu.cut'), icon: '✂️', action: () => ecmClipboard('cut') },
			{ label: $t('contextMenu.copy'), icon: '📋', action: () => ecmClipboard('copy') },
			{ label: $t('contextMenu.paste'), icon: '📥', action: () => ecmClipboard('paste') },
			{ label: $t('contextMenu.pasteAsPlainText'), icon: '📄', action: () => ecmClipboard('pastePlain') },
			{ label: $t('contextMenu.selectAll'), icon: '🔲', action: () => ecmClipboard('selectAll') },
			{ separator: true },
			{ label: $t('contextMenu.style'), icon: '🎨', action: () => openStyleSetterToCategory(styleCat) },
		);
		return items;
	}

	// Map the editor context-menu actions onto NotePane's own commands (reuse, no reinvention).
	function ecmFormat(type: string) {
		switch (type) {
			case 'bold': wrapSelection('**', '**'); break;
			case 'italic': wrapSelection('_', '_'); break;
			case 'underline': wrapSelection('<u>', '</u>'); break;
			case 'strikethrough': wrapSelection('~~', '~~'); break;
			case 'highlight': wrapSelection('==', '=='); break;
			case 'code': wrapSelection('`', '`'); break;
			case 'math': wrapSelection('$', '$'); break;
			case 'superscript': wrapSelection('<sup>', '</sup>'); break;
			case 'subscript': wrapSelection('<sub>', '</sub>'); break;
			case 'toggleComment': wrapSelection('%%', '%%'); break;
			case 'clear': clearFormatting(); break;
		}
	}
	function ecmInsert(type: string) {
		switch (type) {
			case 'link': wrapSelection('[[', ']]'); break;
			case 'externalLink': wrapSelection('[', '](url)'); break;
			case 'blockquote': insertAtCursor('\n> '); break;
			case 'codeBlock': insertAtCursor('\n```\n\n```\n'); break;
			case 'horizontalRule': insertAtCursor('\n---\n'); break;
			case 'table': insertAtCursor('\n| Col 1 | Col 2 |\n| --- | --- |\n| | |\n'); break;
			case 'callout': insertAtCursor('\n> [!note]\n> '); break;
			case 'mathBlock': insertAtCursor('\n$$\n\n$$\n'); break;
			case 'image': insertAtCursor('![](url)'); break;
			case 'footnote': {
				if (!view) return;
				const docLen = view.state.doc.length;
				const at = view.state.selection.main.head;
				const def = (docLen > 0 ? '\n\n' : '') + '[^1]: ';
				view.dispatch({ changes: [{ from: at, insert: '[^1]' }, { from: docLen, insert: def }], selection: { anchor: docLen + 4 + def.length } });
				view.focus();
				break;
			}
		}
	}
	function ecmHeading(level: number) {
		if (!view) return;
		const line = view.state.doc.lineAt(view.state.selection.main.from);
		const m = line.text.match(/^#{1,6}\s/);
		const prefix = level === 0 ? '' : '#'.repeat(level) + ' ';
		if (m) view.dispatch({ changes: { from: line.from, to: line.from + m[0].length, insert: prefix } });
		else if (prefix) view.dispatch({ changes: { from: line.from, insert: prefix } });
		view.focus();
	}
	function ecmList(type: string) {
		if (type === 'bullet') insertLinePrefix('- ');
		else if (type === 'numbered') insertLinePrefix('1. ');
		else if (type === 'task') insertLinePrefix('- [ ] ');
	}
	function ecmClipboard(action: string) {
		if (!view) return;
		if (action === 'cut') document.execCommand('cut');
		else if (action === 'copy') document.execCommand('copy');
		else if (action === 'paste' || action === 'pastePlain') {
			navigator.clipboard.readText().then((txt) => {
				if (txt && view) { const { from, to } = view.state.selection.main; view.dispatch({ changes: { from, to, insert: txt }, selection: { anchor: from + txt.length } }); }
			}).catch(() => {});
		} else if (action === 'selectAll') {
			view.dispatch({ selection: { anchor: 0, head: view.state.doc.length } });
		}
		view.focus();
	}
	function ecmLinkAction(action: string) {
		if (!view) return;
		const line = view.state.doc.lineAt(view.state.selection.main.head);
		const text = line.text;
		if (action === 'open') {
			const m = text.match(/\[\[([^\]]+)\]\]/);
			if (m) document.dispatchEvent(new CustomEvent('constellation:navigate-link', { detail: { link: m[1].split('|')[0].split('#')[0] } }));
		} else if (action === 'copyTarget') {
			const m = text.match(/\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/);
			if (m) navigator.clipboard.writeText(m[1]).catch(() => {});
		} else if (action === 'edit') {
			const s = text.indexOf('[['); const en = text.indexOf(']]');
			if (s >= 0 && en >= 0) view.dispatch({ selection: { anchor: line.from + s, head: line.from + en + 2 } });
		} else if (action === 'remove') {
			const nt = text.replace(/\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g, '$1').replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
			view.dispatch({ changes: { from: line.from, to: line.to, insert: nt } });
		}
		view.focus();
	}
</script>

<div class="e-desk" dir={dir} data-style-target="text">
	<!-- ─── Breadcrumb ─── -->
	<div class="e-breadcrumb">
		{#if hasHistory}
			<button class="e-bc-nav" onclick={() => onnavigateback?.()} disabled={!canGoBack} title={$t('notePane.back')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
			</button>
			<button class="e-bc-nav" onclick={() => onnavigateforward?.()} disabled={!canGoForward} title={$t('notePane.forward')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
			</button>
		{/if}
		<span class="e-bc-lib">{libraryName}</span>
		<span class="e-bc-sep">/</span>
		<span class="e-bc-note">{title}</span>
		{#if currentStage}
			<!-- MIG-014 §2D — promote / demote walk the lifecycle baseline.
			     Custom-term suffix (e.g. `-concept`) carries verbatim across
			     the chain via nextStage / prevStage. Chain length is always 6.
			     Old Zettelkasten values (fleeting/literature/permanent/synthesis)
			     yield null from next/prev, so neither arrow renders — the
			     badge still shows correctly via lookupStageEmoji's legacy
			     fallback. -->
			{@const np = nextStage(currentStage)}
			{@const pp = prevStage(currentStage)}
			{@const stageEmoji = lookupStageEmoji(currentStage)}
			{@const stageDisplayLabel = stageLabel(currentStage, $t)}
			{@const isRTL = dir === 'rtl'}
			<div class="e-bc-stage-wrap">
				{#if pp}
					<button class="e-bc-demote"
						title={$t('notePane.demote')}
						aria-label={$t('notePane.demote')}
						onmousedown={(e) => e.preventDefault()}
						onclick={() => {
							onpromote?.(pp);
							view?.focus();
						}}>{isRTL ? '→' : '←'}</button>
				{/if}
				<span class="e-bc-stage-badge" title={stageDisplayLabel}>
					{stageEmoji} {stageDisplayLabel}
				</span>
				{#if np}
					<button class="e-bc-promote"
						title={$t('notePane.promote')}
						onmousedown={(e) => e.preventDefault()}
						onclick={() => {
							onpromote?.(np);
							view?.focus();
						}}>{$t('notePane.promote')} {isRTL ? '←' : '→'}</button>
				{/if}
			</div>
		{/if}
		{#if trail}
			<div class="e-bc-trail">
				<span class="e-bc-trail-label">🛤️ {trail}</span>
				<span class="e-bc-trail-pos">— {trailIndex + 1} / {trailTotal}</span>
				<button class="e-bc-trail-btn" disabled={trailIndex === 0} onmousedown={(e) => e.preventDefault()} onclick={() => onTrailPrev?.()}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
				</button>
				<button class="e-bc-trail-btn" disabled={trailIndex >= trailTotal - 1} onmousedown={(e) => e.preventDefault()} onclick={() => onTrailNext?.()}>
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
				</button>
			</div>
		{/if}
		<div class="e-bc-actions">
			{#if saving}<span class="e-bc-saving">{$t('notePane.saving')}</span>{/if}
			<div class="e-bc-more-wrap" bind:this={moreMenuEl}>
				<button class="e-bc-dots" onclick={toggleMoreMenu} title={$t('notePane.moreOptions')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="12" cy="19" r="2"/></svg>
				</button>
				{#if showMoreMenu}
					<div class="e-bc-menu" bind:this={bcMenuEl} dir={$uiDir} style="top: {bcMenuTop}px; left: {bcMenuLeft}px;">
						<button class="e-bc-menu-item" onclick={() => { livePreviewEnabled = !livePreviewEnabled; showMoreMenu = false; }}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">{#if livePreviewEnabled}<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>{:else}<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>{/if}</svg>
							{livePreviewEnabled ? ($t('notePane.sourceMode') || 'Source mode') : ($t('notePane.livePreview') || 'Live preview')}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('switchToFocus')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
							{$t('notePane.focusMode') || 'Focus mode'}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('addProperty')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/></svg>
							{$t('contextMenu.addProperty')}
						</button>
						<!-- MIG-101 Phase A — SHAPE. A note's shape governs how it is presented,
						     never what it contains, so every one of these is reversible and none
						     of them touches the body. `revertShape` undoes the last shape change
						     exactly, including back to unshaped.
					     Hidden when read-only: a display (second screen, Index preview) must
					     never be OFFERED a write it cannot legitimately perform. NoteEditor's
					     applyShape refuses independently — this is the second layer, so the
					     UI never shows a dead action. -->
					{#if !readOnly}
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('shapeScrap')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10l6-6V5a2 2 0 0 0-2-2z"/><path d="M15 21v-6h6"/></svg>
							{$t('shape.setScrap')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('shapePage')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="3" width="16" height="18" rx="2"/><path d="M8 8h8M8 12h8M8 16h5"/></svg>
							{$t('shape.setPage')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('shapeClear')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M15 9l-6 6M9 9l6 6"/></svg>
							{$t('shape.clear')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('shapeRevert')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7v6h6"/><path d="M3.5 13a9 9 0 1 0 2.1-6.4L3 9"/></svg>
							{$t('shape.revert')}
						</button>
					{/if}
					<!-- MIG-103 §1 — Save as Template: the THREE kinds (Boss taxonomy, 2026-07-21).
					     Whole note (properties + body) · Frontmatter (properties only) · Snippet
					     (a body fragment). Each stamps template_kind so the "use" side knows the
					     action. Hidden when read-only: a display never creates universe files. -->
					{#if !readOnly}
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('saveTplWhole')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="3" width="16" height="18" rx="2"/><path d="M8 8h8M8 12h8M8 16h5"/></svg>
							{$t('templates.saveAsWhole')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('saveTplFrontmatter')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="3" width="16" height="18" rx="2"/><path d="M8 7h8M8 10h8M8 13h6"/><path d="M4 15.5h16"/></svg>
							{$t('templates.saveAsFrontmatter')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('saveTplSnippet')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 6l-4 6 4 6M16 6l4 6-4 6"/></svg>
							{$t('templates.saveAsSnippet')}
						</button>
					{/if}
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('rename')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
							{$t('contextMenu.rename')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('revealInTree')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9z"/><path d="M9 22V12h6v10"/></svg>
							{$t('contextMenu.revealInTree')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('showInExplorer')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2v11z"/></svg>
							{$t('contextMenu.showInExplorer')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('openDefaultApp')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><path d="M15 3h6v6"/><path d="M10 14L21 3"/></svg>
							{$t('contextMenu.openDefaultApp')}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('copyPath')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
							{$t('contextMenu.copyPath')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('copyName')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1"/></svg>
							{$t('contextMenu.copyName')}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item e-bc-menu-danger" onclick={() => handleMoreAction('delete')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
							{$t('contextMenu.deleteFile')}
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- ─── Paper ─── -->
	<div class="e-paper">
		<input
			class="e-title"
			class:e-title-center={titleAlignment === 'center'}
			bind:this={titleEl}
			bind:value={titleValue}
			dir="auto"
			placeholder={$t('notePane.titlePlaceholder')}
			spellcheck="false"
			readonly={readOnly}
			onblur={handleTitleBlur}
			onkeydown={handleTitleKeydown}
		/>

		<!-- ─── Summary (NSC headline) — under the title, inside the page ─── -->
		{#if summaryHeadline}
			<div class="e-summary" dir="auto" title={summaryHeadline}>{summaryHeadline}</div>
		{/if}

		<!-- ─── Properties ─── -->
		{#if propsMode !== 'hidden' && (properties.length > 0 || rawYaml)}
			{#if propsMode === 'source'}
				<button class="e-props-toggle" onclick={() => propsCollapsed = !propsCollapsed}>
					<svg class="e-props-chevron" class:collapsed={propsCollapsed} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
					<span>{$t('notePane.properties')}</span>
				</button>
				{#if !propsCollapsed}
					<pre class="e-props-source">{rawYaml}</pre>
				{/if}
			{:else}
				<PropertyEditor
					bind:this={propEditorRef}
					{properties}
					body={value}
					{tabId}
					{filePath}
					{libraryName}
					{readOnly}
					noteDir={dir}
					collapsed={propsCollapsed}
					onToggle={() => propsCollapsed = !propsCollapsed}
					onstagechange={(s) => { onpromote?.(s); }}
					{onLiveProps}
					onPropContextMenu={handlePropContextMenu}
				/>
			{/if}
			<hr class="e-props-divider" />
		{/if}

		<!-- ─── Toolbar ─── -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="e-toolbar" dir={toolbarDir} onmousedown={(e) => e.preventDefault()}>
			<button class="e-tb e-toolbar-toggle" class:e-tb-off={!toolbarVisible} title={$t('toolbar.toggle') || 'Toolbar On/Off'} onclick={() => { toolbarVisible = !toolbarVisible; view?.focus(); }}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h16M4 18h16"/></svg>
			</button>
		{#if toolbarVisible}
			<div class="e-tb-sep"></div>
			<button class="e-tb" title={$t('toolbar.bold')} onclick={() => wrapSelection('**', '**')}><strong>B</strong></button>
			<button class="e-tb" title={$t('toolbar.italic')} onclick={() => wrapSelection('_', '_')}><em>I</em></button>
			<button class="e-tb" title={$t('toolbar.strikethrough')} onclick={() => wrapSelection('~~', '~~')}><s>S</s></button>
			<button class="e-tb" title={$t('toolbar.underline') || 'Underline'} onclick={() => wrapSelection('<u>', '</u>')}><span style="text-decoration:underline">U</span></button>
			<button class="e-tb" title={$t('toolbar.highlight')} onclick={() => wrapSelection('==', '==')}><span class="e-tb-hl">H</span></button>
			<button class="e-tb mono" title={$t('toolbar.code')} onclick={() => wrapSelection('`', '`')}>&lt;/&gt;</button>
			<button class="e-tb" title={$t('toolbar.subscript') || 'Subscript'} onclick={() => wrapSelection('<sub>', '</sub>')}><span style="font-size:11px">X<sub style="font-size:9px">2</sub></span></button>
			<button class="e-tb" title={$t('toolbar.superscript') || 'Superscript'} onclick={() => wrapSelection('<sup>', '</sup>')}><span style="font-size:11px">X<sup style="font-size:9px">2</sup></span></button>
			<div class="e-tb-sep"></div>
			<div class="e-tb-drop"><button class="e-tb" onclick={() => toggleMenu('heading')}>H<span class="e-tb-caret">▾</span></button>
				{#if showHeadingMenu}<div class="e-tb-menu">{#each [1,2,3,4,5,6] as lv}<button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('#'.repeat(lv) + ' '); }}>H{lv}</button>{/each}</div>{/if}</div>
			<div class="e-tb-drop"><button class="e-tb" onclick={() => toggleMenu('list')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg><span class="e-tb-caret">▾</span></button>
				{#if showListMenu}<div class="e-tb-menu"><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('- '); }}>• Bullet</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('1. '); }}>1. Numbered</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('- [ ] '); }}>☐ Task</button></div>{/if}</div>
			<div class="e-tb-sep"></div>
			<button class="e-tb e-tb-flip" title={$t('toolbar.alignStart') || 'Align start'} onclick={() => setLineAlignment(toolbarDir === 'rtl' ? 'right' : 'left')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 10H3M21 6H3M21 14H3M17 18H3"/></svg></button>
			<button class="e-tb" title={$t('toolbar.alignCenter') || 'Align center'} onclick={() => setLineAlignment('center')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 10H6M21 6H3M21 14H3M18 18H6"/></svg></button>
			<button class="e-tb e-tb-flip" title={$t('toolbar.alignEnd') || 'Align end'} onclick={() => setLineAlignment(toolbarDir === 'rtl' ? 'left' : 'right')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 10H7M21 6H3M21 14H3M21 18H7"/></svg></button>
			<div class="e-tb-sep"></div>
			<button class="e-tb" title={$t('toolbar.link')} onclick={() => wrapSelection('[[', ']]')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg></button>
			<div class="e-tb-drop"><button class="e-tb" onclick={() => toggleMenu('insert')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/></svg><span class="e-tb-caret">▾</span></button>
				{#if showInsertMenu}<div class="e-tb-menu"><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n> '); }}>❝ Blockquote</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n```\n\n```\n'); }}>⌨ Code block</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n---\n'); }}>― Rule</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n| Col 1 | Col 2 |\n| --- | --- |\n| | |\n'); }}>{$t('toolbar.table')}</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('![](url)'); }}>🖼 Image</button></div>{/if}</div>
			<div class="e-tb-sep"></div>
			<button class="e-tb" title={$t('toolbar.clearFormatting') || 'Clear formatting'} onclick={clearFormatting}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M5 19L19 5" stroke-opacity="0.4"/></svg></button>
			<button class="e-tb" title={$t('toolbar.find') || 'Find & Replace'} onclick={tbFind}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/></svg></button>
			<div class="e-tb-sep"></div>
			<button class="e-tb e-tb-flip" title="Undo" onclick={tbUndo}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6.69 3L3 13"/></svg></button>
			<button class="e-tb e-tb-flip" title="Redo" onclick={tbRedo}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6.69 3L21 13"/></svg></button>
		{/if}
		</div>

		{#if tableToolbarVisible && view && currentTable}
			<div class="e-table-toolbar-float" style="position: fixed; left: {tableToolbarX}px; top: {tableToolbarY}px; transform: translateX(-50%); z-index: 200;">
			<TableToolbar
				x={0}
				y={0}
				dir={toolbarDir}
				onAddRow={() => applyTableChange(addRow(currentTable!, currentTable!.cursorRow))}
				onAddColumn={() => applyTableChange(addColumn(currentTable!, currentTable!.cursorCol))}
				canDeleteRow={currentTable!.cursorRow > 0 && currentTable!.rows.length > 1}
				canDeleteColumn={currentTable!.columnCount > 1}
				onDeleteRow={() => applyTableChange(deleteRow(currentTable!, currentTable!.cursorRow))}
				onDeleteColumn={() => applyTableChange(deleteColumn(currentTable!, currentTable!.cursorCol))}
				onAlignLeft={() => applyTableChange(setAlignment(currentTable!, currentTable!.cursorCol, 'left'))}
				onAlignCenter={() => applyTableChange(setAlignment(currentTable!, currentTable!.cursorCol, 'center'))}
				onAlignRight={() => applyTableChange(setAlignment(currentTable!, currentTable!.cursorCol, 'right'))}
				onMoveRowUp={() => applyTableChange(moveRow(currentTable!, currentTable!.cursorRow, 'up'))}
				onMoveRowDown={() => applyTableChange(moveRow(currentTable!, currentTable!.cursorRow, 'down'))}
				onMoveColLeft={() => applyTableChange(moveColumn(currentTable!, currentTable!.cursorCol, 'left'))}
				onMoveColRight={() => applyTableChange(moveColumn(currentTable!, currentTable!.cursorCol, 'right'))}
				onSortAsc={() => applyTableChange(sortByColumn(currentTable!, currentTable!.cursorCol, 'asc'))}
				onSortDesc={() => applyTableChange(sortByColumn(currentTable!, currentTable!.cursorCol, 'desc'))}
				onInsertFormula={() => { if (view) insertFormulaAtCursor(view, currentTable!); }}
				onEvaluateFormulas={() => applyTableChange({ ...currentTable!, rows: evaluateTableFormulas(currentTable!.rows) })}
			/>
			</div>
		{/if}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="e-editor" bind:this={editorEl} oncontextmenu={handleEditorContextMenu}></div>
		{#if editorCtxMenu}
			<ContextMenu
				x={editorCtxMenu.x}
				y={editorCtxMenu.y}
				items={getEditorMenuItems(editorCtxMenu.onLink)}
				onClose={() => editorCtxMenu = null}
			/>
		{/if}
		{#if fmCtxMenu}
			<ContextMenu
				x={fmCtxMenu.x}
				y={fmCtxMenu.y}
				items={getFrontmatterMenuItems()}
				onClose={() => fmCtxMenu = null}
			/>
		{/if}
	</div>
</div>

<style>
	/* MIG-103 D2 — the "Start from a template…" door inside an empty note.
	   The wrapper takes NO pointer events so clicking the blank body still just
	   places the cursor; only the button is clickable. :global() because CM6
	   owns this DOM (it lives inside .cm-placeholder), so Svelte scoping would
	   never reach it. Quiet by construction — it must read as an offer, not a
	   prompt, and it disappears on the first keystroke. */
	:global(.cm-placeholder) { pointer-events: none; }
	:global(.e-tpl-door) { pointer-events: none; }
	:global(.e-tpl-door-btn) {
		pointer-events: auto;
		font-family: inherit; font-size: 0.85em;
		color: var(--text-faint); background: none;
		border: 1px dashed var(--background-modifier-border, #ccc);
		border-radius: 6px; padding: 2px 10px; cursor: pointer;
		opacity: 0.75; transition: opacity 120ms ease, color 120ms ease;
	}
	:global(.e-tpl-door-btn:hover) {
		opacity: 1; color: var(--text-normal);
		border-color: var(--interactive-accent);
	}
	/* ─── The Desk (spec 3.1) ─── */
	.e-desk {
		flex: 1; display: flex; flex-direction: column; align-items: center;
		background: var(--background-secondary, #e8e8ec); padding-inline: 24px;
		overflow-y: auto; overflow-x: hidden; min-width: 0; min-height: 0;
	}

	/* ─── Breadcrumb (above paper) ─── */
	.e-breadcrumb {
		/* MIG-070 §C — own size (default 0.78rem), so the Setter's "Breadcrumb" element can resize it. */
		padding: 4px var(--file-margins, 48px); font-size: var(--breadcrumb-size, 0.78rem); color: var(--text-faint);
		display: flex; align-items: center; min-height: 28px; flex-shrink: 0;
		width: 100%; max-width: var(--file-line-width, 1200px); background: var(--background-primary, #ffffff);
		border-bottom: 1px solid var(--background-modifier-border, #e0e0e0);
	}
	/* MIG-070 §3B/§C — breadcrumb (library + "/" separator + note name) follows the interface text
	   colour (--text-normal) BY DEFAULT (Eisa: breadcrumb is chrome), but a Setter "Breadcrumb"
	   override (--breadcrumb-color) wins when set, so it can be styled independently if wanted. */
	.e-bc-lib { color: var(--breadcrumb-color, var(--text-normal)); }
	.e-bc-sep { margin: 0 4px; color: var(--breadcrumb-color, var(--text-normal)); }
	.e-bc-note { color: var(--breadcrumb-color, var(--text-normal)); }
	/* §136 — bidirectional stage controls (CE-spec Phase 6, post-revision).
	 * Promote → is the canonical forward verb (prominent, accent border).
	 * ← Demote is the legitimate-but-occasional revision verb (subdued, no
	 * border). Visual asymmetry encodes the frequency asymmetry: forward
	 * is the expected direction, backward is permitted when new evidence
	 * requires revisiting an earlier stage. */
	.e-bc-stage-wrap {
		display: flex; align-items: center; gap: 4px;
		margin-inline-start: 6px;
	}
	.e-bc-stage-badge {
		font-size: 0.72rem; color: var(--text-muted);
		padding: 1px 6px; border-radius: 4px;
		background: var(--background-secondary);
		font-family: inherit; white-space: nowrap;
	}
	.e-bc-promote {
		font-size: 0.68rem; color: var(--interactive-accent); background: none;
		border: 1px solid var(--interactive-accent); border-radius: 4px;
		padding: 1px 6px; cursor: pointer; font-family: inherit;
	}
	.e-bc-promote:hover { background: var(--interactive-accent); color: var(--text-on-accent, #fff); }
	.e-bc-demote {
		font-size: 0.85rem; line-height: 1; color: var(--text-faint);
		background: none; border: none; cursor: pointer; font-family: inherit;
		padding: 0 4px;
	}
	.e-bc-demote:hover { color: var(--text-muted); }
	.e-bc-trail {
		display: flex; align-items: center; gap: 4px; margin-inline-start: 8px;
		font-size: 0.72rem; color: var(--text-muted);
		background: var(--background-secondary); border-radius: 4px; padding: 2px 6px;
	}
	.e-bc-trail-label { font-weight: 600; color: var(--interactive-accent); }
	.e-bc-trail-pos { color: var(--text-faint); }
	.e-bc-trail-btn {
		width: 20px; height: 20px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; cursor: pointer;
		color: var(--text-muted);
	}
	.e-bc-trail-btn:hover:not(:disabled) { background: var(--background-modifier-hover); color: var(--text-normal); }
	.e-bc-trail-btn:disabled { opacity: 0.3; cursor: default; }
	.e-bc-actions { margin-inline-start: auto; display: flex; align-items: center; gap: 4px; position: relative; }
	.e-bc-saving { font-size: 0.7rem; color: var(--interactive-accent); }
	.e-bc-nav {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer; flex-shrink: 0;
	}
	.e-bc-nav:hover:not(:disabled) { background: var(--background-modifier-hover); color: var(--text-normal); }
	.e-bc-nav:disabled { opacity: 0.3; cursor: default; }
	:global([dir="rtl"]) .e-bc-nav svg { transform: scaleX(-1); }
	.e-bc-dots {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer;
	}
	.e-bc-dots:hover { background: var(--background-modifier-border); color: var(--text-normal); }
	.e-bc-more-wrap { position: relative; }
	.e-bc-menu {
		/* RTL truncation fix (2026-07-18) — FIXED (not absolute) so the dropdown escapes
		   .e-desk's overflow-x:hidden clip; top/left set inline by toggleMoreMenu (measured +
		   viewport-clamped, LTR + RTL). Direction follows the UI via the dir attribute. */
		position: fixed; z-index: 300;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px 0; min-width: 220px; max-width: min(360px, calc(100vw - 16px));
		max-height: 80vh; overflow-y: auto; box-shadow: 0 4px 16px rgba(0,0,0,0.15);
	}
	.e-bc-menu-item {
		display: flex; align-items: center; gap: 10px; width: 100%; padding: 7px 14px;
		border: none; background: none; cursor: pointer; font-size: 13px;
		color: var(--text-normal); text-align: start; font-family: var(--font-interface-theme);
	}
	.e-bc-menu-item:hover { background: var(--background-modifier-hover); }
	.e-bc-menu-item svg { flex-shrink: 0; opacity: 0.6; }
	.e-bc-menu-danger { color: var(--text-error, #e53935); }
	.e-bc-menu-danger:hover { background: color-mix(in srgb, var(--text-error, #e53935) 10%, transparent); }
	.e-bc-menu-sep { height: 1px; margin: 4px 10px; background: var(--background-modifier-border); }

	/* ─── The Paper (spec 3.1) ─── */
	.e-paper {
		/* §C Phase 9 wiring-audit — wire the Setter's "Reading width" + "Note margins" (px, defaults = today's look). */
		width: 100%; max-width: var(--file-line-width, 1200px); flex: 1;
		display: flex; flex-direction: column; background: var(--background-primary, #ffffff);
		font-family: var(--font-text-theme, inherit);
		padding: var(--file-margins, 48px); min-width: 0; overflow-y: auto; overflow-x: hidden;
	}

	/* ─── Title (spec 0.3) ─── */
	.e-title {
		display: block; width: 100%; border: none; outline: none; background: transparent;
		font-size: 28px; font-weight: 700; font-family: inherit;
		color: var(--editor-text-color, var(--text-normal, #1a1a1a)); padding: 0;
		margin-block: 0 24px; margin-inline: 0; text-align: start;
	}
	.e-title.e-title-center { text-align: center; }
	/* NSC summary — under the title, inside the page (MIG-070 §iter2-#1, Eisa) */
	.e-summary {
		/* MIG-070 §C — fully restyleable by the Setter's "Note summary" element (colour / font / size /
		   weight / italic), each defaulting to today's look when unset. */
		font-style: var(--summary-style, italic);
		color: var(--summary-color, var(--text-muted));
		font-size: var(--summary-size, 0.95rem);
		font-family: var(--summary-font, inherit);
		font-weight: var(--summary-weight, 400);
		line-height: 1.55;
		margin-block: -12px 22px;
		margin-inline: 0;
		text-align: start;
	}
	.e-title::placeholder { color: var(--text-faint, #ccc); font-weight: 400; }

	/* ─── Properties toggle + source view ─── */
	.e-props-toggle {
		display: flex; align-items: center; gap: 6px;
		border: none; background: none; cursor: pointer; padding: 4px 0;
		font-size: 0.8rem; color: var(--text-muted); font-family: inherit;
	}
	.e-props-toggle:hover { color: var(--text-normal); }
	.e-props-chevron { transition: transform 0.2s; flex-shrink: 0; }
	.e-props-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .e-props-chevron.collapsed { transform: rotate(90deg); }
	.e-props-source {
		font-size: 0.8rem; color: var(--text-muted); background: var(--background-secondary, #f5f5f5);
		padding: 8px 12px; border-radius: 4px; margin: 4px 0 0; overflow-x: auto;
		white-space: pre-wrap; font-family: var(--font-monospace-theme, monospace);
	}
	.e-props-divider {
		border: none; border-top: 1px solid var(--background-modifier-border, #e8e8e8);
		margin: 12px 0;
	}

	/* ─── Toolbar (Phase 4) ─── */
	.e-toolbar { display: flex; align-items: center; gap: 2px; padding: 4px 0; margin-bottom: 8px; border-bottom: 1px solid var(--background-modifier-border, #e8e8e8); flex-wrap: wrap; }
	.e-tb { display: flex; align-items: center; justify-content: center; gap: 2px; width: 28px; height: 28px; border: none; background: none; border-radius: 4px; color: var(--text-muted); cursor: pointer; font-size: 13px; font-family: inherit; }
	.e-tb:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.e-tb.mono { font-family: var(--font-monospace-theme, monospace); font-size: 11px; }
	/* MIG-088 §3b — shares the editor highlight bg + radius. Text colour MIRRORS the body highlight
	   (.cm-md-highlight/.cm-html-mark set no colour → inherit --editor-text-color), so the chip stays
	   readable on any user-chosen --highlight-bg (incl. a dark one) instead of a fixed dark literal. */
	.e-tb-hl { background: var(--highlight-bg, #fef08a); padding: 0 3px; border-radius: var(--highlight-radius, 2px); color: var(--highlight-text, var(--editor-text-color, var(--text-normal, #1a1a1a))); font-size: 12px; }
	.e-tb-sep { width: 1px; height: 18px; background: var(--background-modifier-border); margin: 0 4px; }
	.e-tb.e-tb-off { opacity: 0.35; }
	.e-toolbar[dir="rtl"] .e-tb-flip svg { transform: scaleX(-1); }
	.e-tb-caret { font-size: 8px; opacity: 0.5; margin-inline-start: -2px; }
	.e-tb-drop { position: relative; }
	.e-tb-menu { position: absolute; top: 100%; left: 0; z-index: 100; background: var(--background-primary); border: 1px solid var(--background-modifier-border); border-radius: 6px; padding: 4px 0; min-width: 140px; box-shadow: 0 4px 12px rgba(0,0,0,0.12); }
	:global([dir="rtl"]) .e-tb-menu { left: auto; right: 0; }
	.e-tb-menu-item { display: block; width: 100%; padding: 5px 12px; border: none; background: none; cursor: pointer; font-size: 13px; color: var(--text-normal); text-align: start; font-family: inherit; }
	.e-tb-menu-item:hover { background: var(--background-modifier-hover); }

	/* ─── Editor ─── */
	.e-editor { flex: 1; min-height: 0; }
	.e-editor :global(.cm-editor) { height: 100%; }
	.e-editor :global(.cm-line) { unicode-bidi: plaintext; }
	/* Lines with an explicit dir attribute (set by bidiPlugin) use isolate so dir='rtl'
	   actually governs cursor placement — higher specificity overrides the plaintext above */
	.e-editor :global(.cm-line[dir]) { unicode-bidi: isolate; }
	.e-editor :global(.cm-editor),
	.e-editor :global(.cm-editor.cm-focused) { outline: none !important; border: none !important; }
	/* Force cursor visibility — prevents invisible cursor on click */
	.e-editor :global(.cm-cursor) {
		/* §C Phase 9 wiring-audit — honour the Setter's "Cursor & selection → Cursor colour"
		   (--caret-color); defaults to --text-normal (today's look). */
		border-left: 1.5px solid var(--caret-color, var(--text-normal, #1a1a1a)) !important;
		visibility: visible !important;
	}

	/* ─── Inline icon shortcode widget (:lucide-heart:, :phosphor-book:, ...) ─── */
	.e-editor :global(.cm-icon-inline) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		vertical-align: middle;
		height: 1.15em;
		width: 1.15em;
		line-height: 1;
	}
	.e-editor :global(.cm-icon-inline svg) {
		width: 1.15em;
		height: 1.15em;
	}

	/* ─── Universal Embed Widget (Living Embed Resolver) ─── */
	.e-editor :global(.cm-md-embed) {
		display: block;
		margin: 10px 0;
	}
	.e-editor :global(.cm-md-embed img),
	.e-editor :global(.cm-md-embed video) {
		max-width: 100%;
		border-radius: 6px;
	}
	.e-editor :global(.cm-md-embed audio) {
		width: 100%;
	}
	.e-editor :global(.cm-embed-loading) {
		display: inline-block;
		padding: 6px 10px;
		border-radius: 6px;
		background: var(--background-modifier-border);
		color: var(--text-muted);
		font-size: 0.85em;
	}
	.e-editor :global(.cm-embed-caption) {
		font-size: 0.75em;
		color: var(--text-muted);
		margin-top: 4px;
		text-align: center;
	}
	.e-editor :global(.cm-embed-pdf) {
		width: 100%;
		height: 600px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: var(--background-secondary);
	}
	.e-editor :global(.cm-embed-card) {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		background: var(--background-secondary);
	}
	.e-editor :global(.cm-embed-card-icon) {
		font-size: 1.6em;
		flex-shrink: 0;
	}
	.e-editor :global(.cm-embed-card-body) {
		min-width: 0;
		flex: 1;
	}
	.e-editor :global(.cm-embed-card-title) {
		font-weight: 600;
		color: var(--text-normal);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.e-editor :global(.cm-embed-card-sub) {
		font-size: 0.8em;
		color: var(--text-muted);
	}
	.e-editor :global(.cm-embed-missing) {
		border-color: color-mix(in srgb, var(--text-warning, #f59e0b) 40%, transparent);
		background: color-mix(in srgb, var(--text-warning, #f59e0b) 8%, var(--background-secondary));
	}
	.e-editor :global(.cm-embed-missing-details) {
		margin-top: 4px;
		font-size: 0.78em;
		color: var(--text-muted);
	}
	.e-editor :global(.cm-embed-missing-details summary) {
		cursor: pointer;
		padding: 4px 0;
	}
	.e-editor :global(.cm-embed-missing-info) {
		white-space: pre-wrap;
		word-break: break-all;
		font-family: var(--font-monospace-theme, monospace);
		font-size: 0.88em;
		padding: 6px 8px;
		border-radius: 4px;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		margin-top: 4px;
		max-height: 180px;
		overflow-y: auto;
	}
	/* Note transclusion */
	.e-editor :global(.cm-embed-transclusion) {
		border-inline-start: 3px solid var(--interactive-accent);
		background: var(--background-secondary);
		border-radius: 0 6px 6px 0;
		padding: 8px 12px;
	}
	.e-editor :global(.cm-embed-transclusion-header) {
		font-weight: 600;
		color: var(--interactive-accent);
		font-size: 0.9em;
		padding-bottom: 4px;
		border-bottom: 1px solid var(--background-modifier-border);
		margin-bottom: 6px;
	}
	.e-editor :global(.cm-embed-transclusion-header:hover) {
		text-decoration: underline;
	}
	.e-editor :global(.cm-embed-transclusion-body) {
		white-space: pre-wrap;
		color: var(--text-muted);
		font-size: 0.92em;
		max-height: 360px;
		overflow-y: auto;
	}

</style>

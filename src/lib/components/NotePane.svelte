<script lang="ts">
	/**
	 * NotePane — The core note editor.
	 * Gray desk + breadcrumb + white paper + title + properties + CM6 editor + persistence.
	 * Live preview decorations via shared livePreview plugin. Typing must be instant.
	 * Originally developed as eNotePane (experimental), promoted to production 2026-03-29.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings, getEffectiveScriptFonts } from '$lib/libraries/store';
	import { LIVING_LINK_BASELINE, lookupStageEmoji } from '$lib/libraries/store';
	import type { FrontmatterProperty } from '$lib/libraries/store';
	import PropertyEditor from './PropertyEditor.svelte';
	import { EditorView, keymap, drawSelection, Decoration, type DecorationSet } from '@codemirror/view';
	import { EditorState, Compartment, Prec, StateField, StateEffect, RangeSetBuilder } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
	import { tags } from '@lezer/highlight';
	import { defaultKeymap, history, historyKeymap, undo, redo, indentWithTab } from '@codemirror/commands';
	import { autocompletion, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
	import { search, openSearchPanel, searchKeymap, SearchQuery, setSearchQuery, findNext } from '@codemirror/search';
	import { livePreviewPlugin, livePreviewTheme, libraryPathField, setLibraryPath, notePathField, setNotePath, attachmentFolderField, setAttachmentFolder, linkTraversalMapField, setLinkTraversalMap } from '$lib/editor/livePreview';
	import { calloutPlugin, calloutTheme, calloutCollapseField, toggleCallout } from '$lib/editor/calloutPlugin';
	import { lineDecoPlugin, lineDecoTheme } from '$lib/editor/lineDecoPlugin';
	import { bidiPlugin, bidiTheme, scriptFontsField, setScriptFonts } from '$lib/editor/bidiPlugin';
	import { registerActiveEditor, unregisterActiveEditor } from '$lib/editor/activeEditor';
	import { Highlight as HighlightExt } from '$lib/editor/markdownHighlight';
	import { createWikilinkCompletion, createTagCompletion, createSlashCompletion, createTypedLinkCompletion } from '$lib/editor/completions';
	import { shortcodeCompletion } from '$lib/editor/shortcodeAutocomplete';
	import TableToolbar from './TableToolbar.svelte';
	import { parseTable, formatTable, addRow, addColumn, deleteRow, deleteColumn, setAlignment, moveRow, moveColumn, sortByColumn, type ParsedTable } from '$lib/editor/tableUtils';
	import { evaluateTableFormulas, indexToCol } from '$lib/editor/tableFormulas';

	/* Markdown syntax colors */
	const markdownHighlightStyle = HighlightStyle.define([
		{ tag: tags.heading1, color: '#d73a49', fontWeight: '700' },
		{ tag: tags.heading2, color: '#d73a49', fontWeight: '700' },
		{ tag: tags.heading3, color: '#d73a49', fontWeight: '600' },
		{ tag: tags.heading4, color: '#d73a49', fontWeight: '600' },
		{ tag: tags.heading5, color: '#d73a49', fontWeight: '600' },
		{ tag: tags.heading6, color: '#d73a49', fontWeight: '600' },
		{ tag: tags.strong, color: '#e36209' },
		{ tag: tags.emphasis, color: '#7c3aed' },
		{ tag: tags.strikethrough, textDecoration: 'line-through' },
		{ tag: tags.monospace, color: '#16a34a' },
		{ tag: tags.link, color: '#2563eb' },
		{ tag: tags.url, color: '#0891b2' },
		{ tag: tags.processingInstruction, color: '#888' }, /* frontmatter fences */
		{ tag: tags.meta, color: '#888' },
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
		/* Phase 8: autocomplete data */
		noteNames = [] as { name: string; path: string; libraryName?: string }[],
		allTags = [] as string[],
		/* Callbacks */
		onchange,
		onsave,
		onflush,
		ontitlechange,
		onnavigateback,
		onnavigateforward,
		onmoreaction,
		onpropschange,
		stage = '',
		onpromote,
		trail = '',
		trailIndex = 0,
		trailTotal = 0,
		onTrailPrev,
		onTrailNext,
		highlightTerm = '',
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
		noteNames?: { name: string; path: string; libraryName?: string }[];
		allTags?: string[];
		onchange?: (value: string) => void;
		onsave?: (value: string, filePath: string) => void;
		onflush?: (text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number, filePath: string) => void;
		ontitlechange?: (newTitle: string, filePath: string) => void;
		onnavigateback?: () => void;
		onnavigateforward?: () => void;
		onmoreaction?: (action: string) => void;
		onpropschange?: () => void;
		stage?: string;
		onpromote?: (nextStage: string) => void;
		trail?: string;
		trailIndex?: number;
		trailTotal?: number;
		onTrailPrev?: () => void;
		onTrailNext?: () => void;
		highlightTerm?: string;
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
	let currentStage = $state(stage?.toLowerCase() ?? '');
	// Stage sync: breadcrumb ← Properties panel via onstagechange callback (no $effect needed)
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
	const livePreviewCompartment = new Compartment();
	let livePreviewEnabled = $state(true);

	/* ─── Table toolbar state ─── */
	let tableToolbarX = $state(0);
	let tableToolbarY = $state(0);
	let currentTable = $state<ParsedTable | null>(null);
	let tableToolbarVisible = $derived(currentTable !== null);

	/* ─── Phase 3 state ─── */
	let propsCollapsed = $state(true);
	let showMoreMenu = $state(false);
	let moreMenuEl: HTMLDivElement | undefined;
	const hasHistory = $derived(canGoBack || canGoForward);
	const propsMode = $derived($appSettings.propertiesInDocument ?? 'visible');

	/* ─── More menu ─── */
	function toggleMoreMenu() {
		showMoreMenu = !showMoreMenu;
		if (showMoreMenu) {
			setTimeout(() => window.addEventListener('click', closeMoreMenu, { once: true }), 0);
		}
	}
	function closeMoreMenu() { showMoreMenu = false; }
	function handleMoreAction(action: string) {
		showMoreMenu = false;
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

	/* ─── Mount ─── */
	onMount(() => {
		const state = EditorState.create({
			doc: value,
			extensions: [
				history(),
				drawSelection(),
				markdown({ base: markdownLanguage, extensions: [HighlightExt] }),
				syntaxHighlighting(markdownHighlightStyle),
				calloutCollapseField,
				livePreviewCompartment.of(livePreviewEnabled ? [livePreviewPlugin, livePreviewTheme, calloutPlugin, calloutTheme] : []),
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
							? [typedLinkCompletion, wikilinkCompletion, tagCompletion, slashCompletion, shortcodeCompletion]
							: [typedLinkCompletion, wikilinkCompletion, tagCompletion, slashCompletion]
					),
					activateOnTyping: true,
					maxRenderedOptions: 20,
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
				dirCompartment.of(EditorView.editorAttributes.of({ dir: dir || 'auto' })),
				EditorView.contentAttributes.of({ dir: 'auto' }),
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						// ⚡ PERF: do NOT call doc.toString() here — it's O(N) on every keystroke
						// and causes progressive lag as the document grows.
						// latestText is refreshed in doSave/doFlush at the moment of writing.
						dirty = true;
						onchange?.('');
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
					'.cm-content': { padding: '0', caretColor: 'var(--caret-color, var(--text-normal, #1a1a1a))' },
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
					'.cm-hl-title': { backgroundColor: 'color-mix(in srgb, #3b82f6 25%, transparent)', outline: '1px solid #3b82f6', borderRadius: '2px' },
					'.cm-hl-content': { backgroundColor: 'color-mix(in srgb, #16a34a 25%, transparent)', outline: '1px solid #16a34a', borderRadius: '2px' },
					'.cm-hl-tag': { backgroundColor: 'color-mix(in srgb, #f472b6 25%, transparent)', outline: '1px solid #f472b6', borderRadius: '2px' },
					'.cm-hl-property': { backgroundColor: 'color-mix(in srgb, #f59e0b 25%, transparent)', outline: '1px solid #f59e0b', borderRadius: '2px' },
					'.cm-hl-wikilink': { backgroundColor: 'color-mix(in srgb, #60a5fa 25%, transparent)', outline: '1px solid #60a5fa', borderRadius: '2px' },
					'.cm-hl-semantic': { backgroundColor: 'color-mix(in srgb, #7c3aed 25%, transparent)', outline: '1px solid #7c3aed', borderRadius: '2px' },
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
					const link = match[1].split('|')[0].split('#')[0].trim();
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

		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
			view.focus();
		} else {
			titleEl?.focus();
		}
		if (initialScrollTop > 0) {
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
		if (view) registerActiveEditor(view);
	});

	function onEditorFocusIn() {
		if (view) registerActiveEditor(view);
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
			view.dispatch({ effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir })) });
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
					livePreviewEnabled ? [livePreviewPlugin, livePreviewTheme, calloutPlugin, calloutTheme] : []
				)
			});
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
</script>

<div class="e-desk" dir={dir}>
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
			<!-- MIG-014 §1D — stage list = Living Link 6-stage baseline + per-Universe customs.
			     Demote/promote arrows iterate through it in order. Custom stages append
			     chronologically. Old Zettelkasten values (fleeting/literature/permanent/
			     synthesis) yield idx = -1 so neither arrow renders, but the badge still
			     shows via lookupStageEmoji's legacy fallback. -->
			{@const stageOrder = LIVING_LINK_BASELINE.map(s => s.name)}
			{@const idx = stageOrder.indexOf(currentStage)}
			{@const stageEmoji = lookupStageEmoji(currentStage)}
			{@const stageDisplayLabel = LIVING_LINK_BASELINE.some(b => b.name === currentStage) ? $t(`notePane.stage.${currentStage}`) : currentStage.charAt(0).toUpperCase() + currentStage.slice(1)}
			{@const isRTL = dir === 'rtl'}
			<div class="e-bc-stage-wrap">
				{#if idx > 0}
					<button class="e-bc-demote"
						title={$t('notePane.demote')}
						aria-label={$t('notePane.demote')}
						onmousedown={(e) => e.preventDefault()}
						onclick={() => {
							const prev = stageOrder[idx - 1];
							currentStage = prev;
							onpromote?.(prev);
							view?.focus();
						}}>{isRTL ? '→' : '←'}</button>
				{/if}
				<span class="e-bc-stage-badge" title={stageDisplayLabel}>
					{stageEmoji} {stageDisplayLabel}
				</span>
				{#if idx >= 0 && idx < stageOrder.length - 1}
					<button class="e-bc-promote"
						title={$t('notePane.promote')}
						onmousedown={(e) => e.preventDefault()}
						onclick={() => {
							const next = stageOrder[idx + 1];
							currentStage = next;
							onpromote?.(next);
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
					<div class="e-bc-menu">
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
			onblur={handleTitleBlur}
			onkeydown={handleTitleKeydown}
		/>

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
					{properties}
					body={value}
					{tabId}
					{filePath}
					{libraryName}
					noteDir={dir}
					collapsed={propsCollapsed}
					onToggle={() => propsCollapsed = !propsCollapsed}
					onstagechange={(s) => { currentStage = s; onpromote?.(s); }}
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
		<div class="e-editor" bind:this={editorEl}></div>
	</div>
</div>

<style>
	/* ─── The Desk (spec 3.1) ─── */
	.e-desk {
		flex: 1; display: flex; flex-direction: column; align-items: center;
		background: #e8e8ec; padding-inline: 24px;
		overflow-y: auto; overflow-x: hidden; min-width: 0; min-height: 0;
	}

	/* ─── Breadcrumb (above paper) ─── */
	.e-breadcrumb {
		padding: 4px 48px; font-size: 0.78rem; color: var(--text-faint);
		display: flex; align-items: center; min-height: 28px; flex-shrink: 0;
		width: 100%; max-width: 1200px; background: #ffffff;
		border-bottom: 1px solid var(--background-modifier-border, #e0e0e0);
	}
	.e-bc-lib { color: var(--text-muted); }
	.e-bc-sep { margin: 0 4px; color: var(--background-modifier-border-focus); }
	.e-bc-note { color: var(--text-normal); }
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
		position: absolute; top: 100%; right: 0; z-index: 100;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px 0; min-width: 220px; max-height: 80vh; overflow-y: auto;
		box-shadow: 0 4px 16px rgba(0,0,0,0.15); direction: ltr;
	}
	:global([dir="rtl"]) .e-bc-menu { right: auto; left: 0; direction: rtl; }
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
		width: 100%; max-width: 1200px; flex: 1;
		display: flex; flex-direction: column; background: #ffffff;
		padding: 48px; min-width: 0; overflow-y: auto; overflow-x: hidden;
	}

	/* ─── Title (spec 0.3) ─── */
	.e-title {
		display: block; width: 100%; border: none; outline: none; background: transparent;
		font-size: 28px; font-weight: 700; font-family: inherit;
		color: var(--text-normal, #1a1a1a); padding: 0;
		margin-block: 0 24px; margin-inline: 0; text-align: start;
	}
	.e-title.e-title-center { text-align: center; }
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
	.e-tb-hl { background: #fef08a; padding: 0 3px; border-radius: 2px; color: #1a1a1a; font-size: 12px; }
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
		border-left: 1.5px solid var(--text-normal, #1a1a1a) !important;
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

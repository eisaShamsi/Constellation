<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { EditorView, ViewPlugin, Decoration, keymap, placeholder as cmPlaceholder, drawSelection, dropCursor, highlightActiveLine, highlightActiveLineGutter, lineNumbers, highlightSpecialChars, rectangularSelection, type ViewUpdate, type DecorationSet } from '@codemirror/view';
	import { EditorState, Compartment, type Range } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { languages } from '@codemirror/language-data';
	import { defaultKeymap, history, historyKeymap, indentWithTab, undo, redo } from '@codemirror/commands';
	import { searchKeymap, highlightSelectionMatches, openSearchPanel, selectNextOccurrence } from '@codemirror/search';
	import { closeBrackets, closeBracketsKeymap, autocompletion, type CompletionContext, type Completion } from '@codemirror/autocomplete';
	import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput, foldGutter, foldKeymap, foldService, foldAll, unfoldAll, indentUnit } from '@codemirror/language';
	import type { FileEntry } from '$lib/libraries/store';
	import { saveClipboardImage, resolveWikilinkCrossLibrary, getNoteHeadings, appSettings, getFontSetById, SCRIPT_UNICODE_RANGES } from '$lib/libraries/store';
	import { t } from '$lib/i18n';
	import FormattingToolbar from './FormattingToolbar.svelte';
	import TableToolbar from './TableToolbar.svelte';
	import { parseTable, formatTable, addRow, addColumn, deleteRow, deleteColumn, setAlignment, moveRow, moveColumn, sortByColumn, generateTable, detectTabularText, tabularTextToTable, type ParsedTable } from '$lib/editor/tableUtils';
	import { evaluateTableFormulas, indexToCol } from '$lib/editor/tableFormulas';
	import { livePreviewPlugin, livePreviewTheme, libraryPathField, setLibraryPath } from '$lib/editor/livePreview';
	import { calloutPlugin, calloutTheme, calloutCollapseField, calloutClickHandler } from '$lib/editor/calloutPlugin';
	import { lineDecoPlugin, lineDecoTheme } from '$lib/editor/lineDecoPlugin';
	import { bidiPlugin, bidiTheme, scriptFontsField, setScriptFonts } from '$lib/editor/bidiPlugin';
	import { Highlight as HighlightExt } from '$lib/editor/markdownHighlight';
	import { createTagCompletion, createSlashCompletion } from '$lib/editor/completions';
	import TableGridPicker from './TableGridPicker.svelte';
	import EditorContextMenu from './EditorContextMenu.svelte';
	import { syntaxTree } from '@codemirror/language';

	let {
		value = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		placeholder = '',
		onchange,
		noteNames = [] as { name: string; path: string; libraryName?: string }[],
		allTags = [] as string[],
		libraryPath = '',
		livePreview = false,
		showLineNumbers = true,
		foldHeading = true,
		foldIndent = true,
		indentationGuides = false,
		indentWithTabs = true,
		tabSize = 4,
		autoPairMarkdown = true,
		initialCursorPos = 0,
		initialScrollTop = 0,
		onCursorChange,
		onScrollChange,
	}: {
		value: string;
		dir?: 'ltr' | 'rtl';
		placeholder?: string;
		onchange: (value: string) => void;
		noteNames?: { name: string; path: string; libraryName?: string }[];
		allTags?: string[];
		libraryPath?: string;
		livePreview?: boolean;
		showLineNumbers?: boolean;
		foldHeading?: boolean;
		foldIndent?: boolean;
		indentationGuides?: boolean;
		indentWithTabs?: boolean;
		tabSize?: number;
		autoPairMarkdown?: boolean;
		initialCursorPos?: number;
		initialScrollTop?: number;
		onCursorChange?: (pos: number) => void;
		onScrollChange?: (top: number) => void;
	} = $props();

	let containerEl: HTMLDivElement;
	let view = $state<EditorView | undefined>(undefined);
	let onchangeTimer: ReturnType<typeof setTimeout> | null = null;
	let toolbarRafPending = false;
	let dirCompartment = new Compartment();
	let livePreviewCompartment = new Compartment();
	let lineNumbersCompartment = new Compartment();
	let foldGutterCompartment = new Compartment();
	let indentGuidesCompartment = new Compartment();
	let tabConfigCompartment = new Compartment();
	let fontCompartment = new Compartment();
	let updating = false;
	let lastInternalValue = value;

	function buildFontTheme(): ReturnType<typeof EditorView.theme> {
		const s = get(appSettings);
		const defaultUI = '-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif';
		const defaultMono = '"Cascadia Code", "Fira Code", Consolas, monospace';
		let fontFamily = defaultUI;
		let monoFont = defaultMono;
		let fontSize = (s.fontSize || 17) + 'px';

		if (s.fontMode === 'universal') {
			const set = getFontSetById(s.activeFontSetId || 'system', s.customFontSets || []);
			fontFamily = set?.textFont || set?.name || s.textFont || defaultUI;
			monoFont = set?.monoFont || s.monoFont || defaultMono;
		} else {
			const langSets = s.languageFontSets || {};
			const customSets = s.customFontSets || [];
			const latinSet = getFontSetById(langSets.latin || 'system', customSets);
			fontFamily = latinSet?.textFont || latinSet?.name || s.textFont || defaultUI;
			monoFont = latinSet?.monoFont || s.monoFont || defaultMono;

			// Generate @font-face for per-language scripts
			let css = '';
			let hasPerScript = false;
			for (const [script, range] of Object.entries(SCRIPT_UNICODE_RANGES)) {
				if (script === 'latin') continue;
				const setId = langSets[script];
				if (!setId || setId === 'system') continue;
				const set = getFontSetById(setId, customSets);
				if (!set) continue;
				hasPerScript = true;
				const txtName = (set.textFont || set.name || '').split(',')[0].trim().replace(/"/g, '');
				if (txtName) {
					css += `@font-face { font-family: "ConstellationEditor"; src: local("${txtName}"); unicode-range: ${range}; }\n`;
				}
			}
			let editorStyleEl = document.getElementById('constellation-editor-fonts');
			if (!editorStyleEl) {
				editorStyleEl = document.createElement('style');
				editorStyleEl.id = 'constellation-editor-fonts';
				document.head.appendChild(editorStyleEl);
			}
			editorStyleEl.textContent = css;
			if (hasPerScript) {
				fontFamily = `"ConstellationEditor", ${fontFamily}`;
			}
		}

		return EditorView.theme({
			'.cm-content': { fontFamily, fontSize, lineHeight: '1.7' },
			'.cm-content .cm-line': { fontFamily },
			'&': { fontSize },
		});
	}

	// Formatting toolbar state
	let toolbarVisible = $state(false);
	let toolbarX = $state(0);
	let toolbarY = $state(0);
	let toolbarTimeout: ReturnType<typeof setTimeout> | null = null;

	function updateToolbar(editorView: EditorView) {
		// Clear any pending show
		if (toolbarTimeout) { clearTimeout(toolbarTimeout); toolbarTimeout = null; }

		const { from, to } = editorView.state.selection.main;
		if (from === to || !containerEl) {
			toolbarVisible = false;
			return;
		}

		// Debounce: only show toolbar after selection is stable for 400ms
		toolbarTimeout = setTimeout(() => {
			if (!containerEl || !view) return;
			const sel = view.state.selection.main;
			if (sel.from === sel.to) { toolbarVisible = false; return; }
			const fromCoords = view.coordsAtPos(sel.from);
			const toCoords = view.coordsAtPos(sel.to);
			if (!fromCoords || !toCoords) { toolbarVisible = false; return; }
			const containerRect = containerEl.getBoundingClientRect();
			const midX = (fromCoords.left + toCoords.right) / 2 - containerRect.left;
			const topY = Math.min(fromCoords.top, toCoords.top) - containerRect.top - 44;
			toolbarX = Math.max(120, Math.min(midX, containerRect.width - 120));
			toolbarY = Math.max(4, topY);
			toolbarVisible = true;
		}, 400);
	}

	function applyHeading(level: number) {
		if (!view) return;
		const { state } = view;
		const line = state.doc.lineAt(state.selection.main.from);
		const existingMatch = line.text.match(/^(#{1,6})\s/);
		const prefix = '#'.repeat(level) + ' ';
		if (existingMatch) {
			view.dispatch({ changes: { from: line.from, to: line.from + existingMatch[0].length, insert: prefix } });
		} else {
			view.dispatch({ changes: { from: line.from, to: line.from, insert: prefix } });
		}
		view.focus();
	}

	// Table toolbar state
	let tableToolbarVisible = $state(false);
	let tableToolbarX = $state(0);
	let tableToolbarY = $state(0);
	let currentTable = $state<ParsedTable | null>(null);

	function updateTableToolbar(editorView: EditorView) {
		const pos = editorView.state.selection.main.head;
		const table = parseTable(editorView.state, pos);
		if (table && containerEl) {
			currentTable = table;
			const line = editorView.state.doc.line(table.startLine);
			const coords = editorView.coordsAtPos(line.from);
			if (coords) {
				const containerRect = containerEl.getBoundingClientRect();
				tableToolbarX = Math.max(160, containerRect.width / 2);
				tableToolbarY = coords.top - containerRect.top - 40;
				if (tableToolbarY < 4) tableToolbarY = coords.bottom - containerRect.top + 4; // Below if no space above
			}
			tableToolbarVisible = true;
		} else {
			tableToolbarVisible = false;
			currentTable = null;
		}
	}

	// Grid picker state (for visual table insertion)
	let gridPickerVisible = $state(false);
	let gridPickerX = $state(0);
	let gridPickerY = $state(0);

	// ─── Context menu state ───
	let contextMenuVisible = $state(false);
	let contextMenuX = $state(0);
	let contextMenuY = $state(0);
	let contextMenuHasSelection = $state(false);
	let contextMenuCursorContext = $state<'normal' | 'heading' | 'table' | 'checkbox' | 'link' | 'codeblock' | 'list' | 'blockquote'>('normal');
	// Toolbar context — tracks what cursor context the toolbar should show
	let toolbarContext = $state<'normal' | 'heading' | 'table' | 'codeblock' | 'list' | 'blockquote'>('normal');
	let toolbarHeadingLevel = $state<number | null>(null);
	let contextMenuHeadingLevel = $state<number | null>(null);

	function detectCursorContext(editorView: EditorView): { context: typeof contextMenuCursorContext; headingLevel: number | null } {
		const pos = editorView.state.selection.main.head;
		const tree = syntaxTree(editorView.state);
		let node = tree.resolve(pos, -1);
		let context: typeof contextMenuCursorContext = 'normal';
		let headingLevel: number | null = null;

		while (node) {
			const name = node.name;
			if (name === 'Table') { context = 'table'; break; }
			if (name === 'FencedCode' || name === 'CodeBlock') { context = 'codeblock'; break; }
			if (name.startsWith('ATXHeading')) {
				context = 'heading';
				const match = name.match(/(\d)/);
				headingLevel = match ? parseInt(match[1]) : null;
				if (!headingLevel) {
					const line = editorView.state.doc.lineAt(pos);
					const hm = line.text.match(/^(#{1,6})\s/);
					headingLevel = hm ? hm[1].length : null;
				}
				break;
			}
			if (name === 'Link' || name === 'WikiLink') { context = 'link'; break; }
			if (name === 'TaskMarker') { context = 'checkbox'; break; }
			if (name === 'BulletList' || name === 'OrderedList' || name === 'ListItem') { context = 'list' as any; break; }
			if (name === 'Blockquote') { context = 'blockquote' as any; break; }
			if (node.parent) { node = node.parent; } else { break; }
		}

		// Also check for task markers via line text
		if (context === 'normal') {
			const line = editorView.state.doc.lineAt(pos);
			if (/^\s*- \[[ x]\]/i.test(line.text)) context = 'checkbox';
			else if (/^\s*[-*+]\s/.test(line.text) || /^\s*\d+\.\s/.test(line.text)) context = 'list' as any;
			else if (/^>\s?/.test(line.text)) context = 'blockquote' as any;
		}

		return { context, headingLevel };
	}

	function editorEventHandlers() {
		return EditorView.domEventHandlers({
			// Ctrl+Click to follow links
			click: (event: MouseEvent, editorView: EditorView) => {
				if (!(event.ctrlKey || event.metaKey)) return false;
				const pos = editorView.posAtCoords({ x: event.clientX, y: event.clientY });
				if (pos === null) return false;
				const line = editorView.state.doc.lineAt(pos);
				const offset = pos - line.from;
				// Check for wikilink at click position
				const wikiRe = /\[\[([^\]]+)\]\]/g;
				let match;
				while ((match = wikiRe.exec(line.text)) !== null) {
					if (offset >= match.index && offset <= match.index + match[0].length) {
						event.preventDefault();
						const link = match[1].split('|')[0].split('#')[0];
						document.dispatchEvent(new CustomEvent('constellation:navigate-link', { detail: { link } }));
						return true;
					}
				}
				// Check for markdown link at click position
				const mdRe = /\[([^\]]+)\]\(([^)]+)\)/g;
				while ((match = mdRe.exec(line.text)) !== null) {
					if (offset >= match.index && offset <= match.index + match[0].length) {
						event.preventDefault();
						const url = match[2];
						if (url.startsWith('http://') || url.startsWith('https://')) {
							window.open(url, '_blank');
						}
						return true;
					}
				}
				return false;
			},
			contextmenu: (event: MouseEvent, editorView: EditorView) => {
				event.preventDefault();
				// Hide formatting toolbar
				toolbarVisible = false;
				if (toolbarTimeout) { clearTimeout(toolbarTimeout); toolbarTimeout = null; }

				const { context, headingLevel } = detectCursorContext(editorView);
				const sel = editorView.state.selection.main;
				contextMenuHasSelection = sel.from !== sel.to;
				contextMenuCursorContext = context;
				contextMenuHeadingLevel = headingLevel;
				contextMenuX = event.clientX;
				contextMenuY = event.clientY;
				contextMenuVisible = true;
			}
		});
	}

	function handleContextFormat(type: string) {
		if (!view) return;
		switch (type) {
			case 'bold': wrapSelection(view, '**', '**'); break;
			case 'italic': wrapSelection(view, '_', '_'); break;
			case 'strikethrough': wrapSelection(view, '~~', '~~'); break;
			case 'highlight': wrapSelection(view, '==', '=='); break;
			case 'code': wrapSelection(view, '`', '`'); break;
			case 'toggleComment': toggleComment(view); break;
			case 'toggleCheckbox': toggleCheckbox(view); break;
			case 'underline': applyUnderline(view); break;
		case 'subscript': applySubscript(view); break;
		case 'superscript': applySuperscript(view); break;
		case 'clear': clearFormatting(view); break;
		}
		view.focus();
	}

	function handleContextInsert(type: string) {
		if (!view) return;
		const pos = view.state.selection.main.head;
		const line = view.state.doc.lineAt(pos);
		const atLineStart = line.text.trim() === '';
		const nl = atLineStart ? '' : '\n';

		switch (type) {
			case 'link': wrapSelection(view, '[[', ']]'); break;
			case 'image': {
				const insert = nl + '![[]]' + '\n';
				view.dispatch({ changes: { from: atLineStart ? line.from : pos, insert }, selection: { anchor: (atLineStart ? line.from : pos) + nl.length + 3 } });
				break;
			}
			case 'table': showTableGridPicker(); return; // Don't focus yet
			case 'horizontalRule': {
				const insert = nl + '---\n';
				view.dispatch({ changes: { from: atLineStart ? line.from : pos, insert } });
				break;
			}
			case 'codeBlock': {
				const insert = nl + '```\n\n```\n';
				view.dispatch({ changes: { from: atLineStart ? line.from : pos, insert }, selection: { anchor: (atLineStart ? line.from : pos) + nl.length + 4 } });
				break;
			}
			case 'callout': {
				const insert = nl + '> [!note]\n> ';
				view.dispatch({ changes: { from: atLineStart ? line.from : pos, insert }, selection: { anchor: (atLineStart ? line.from : pos) + nl.length + insert.length - nl.length } });
				break;
			}
			case 'mathBlock': {
				const insert = nl + '$$\n\n$$\n';
				view.dispatch({ changes: { from: atLineStart ? line.from : pos, insert }, selection: { anchor: (atLineStart ? line.from : pos) + nl.length + 3 } });
				break;
			}
			case 'blockquote': {
				const text = line.text;
				if (text.startsWith('> ')) {
					view.dispatch({ changes: { from: line.from, to: line.from + 2, insert: '' } });
				} else {
					view.dispatch({ changes: { from: line.from, insert: '> ' } });
				}
				break;
			}
		}
		view.focus();
	}

	function handleContextHeading(level: number) {
		if (level === 0) {
			// Remove heading
			if (!view) return;
			const line = view.state.doc.lineAt(view.state.selection.main.from);
			const match = line.text.match(/^#{1,6}\s/);
			if (match) {
				view.dispatch({ changes: { from: line.from, to: line.from + match[0].length, insert: '' } });
			}
			view.focus();
		} else {
			applyHeading(level);
		}
	}

	function handleContextList(type: string) {
		if (!view) return;
		const line = view.state.doc.lineAt(view.state.selection.main.from);
		const text = line.text;
		// Remove existing list prefix
		const cleaned = text.replace(/^\s*([-*+]|\d+\.)\s(\[[ x]\]\s)?/, '');
		let prefix = '';
		switch (type) {
			case 'bullet': prefix = '- '; break;
			case 'numbered': prefix = '1. '; break;
			case 'task': prefix = '- [ ] '; break;
		}
		view.dispatch({ changes: { from: line.from, to: line.to, insert: prefix + cleaned } });
		view.focus();
	}

	function handleContextClipboard(action: string) {
		if (!view) return;
		switch (action) {
			case 'cut':
				document.execCommand('cut');
				break;
			case 'copy':
				document.execCommand('copy');
				break;
			case 'paste':
				navigator.clipboard.readText().then(text => {
					if (text) {
						const { from, to } = view!.state.selection.main;
						view!.dispatch({ changes: { from, to, insert: text }, selection: { anchor: from + text.length } });
					}
				}).catch(() => {});
				break;
			case 'pastePlain':
				pasteAsPlainText(view);
				break;
		}
		view.focus();
	}

	function handleContextTableAction(action: string) {
		if (!currentTable) return;
		switch (action) {
			case 'addRow': applyTableChange(addRow(currentTable, currentTable.cursorRow)); break;
			case 'addColumn': applyTableChange(addColumn(currentTable, currentTable.cursorCol)); break;
			case 'deleteRow': applyTableChange(deleteRow(currentTable, currentTable.cursorRow)); break;
			case 'deleteColumn': applyTableChange(deleteColumn(currentTable, currentTable.cursorCol)); break;
			case 'sortAsc': applyTableChange(sortByColumn(currentTable, currentTable.cursorCol, 'asc')); break;
			case 'sortDesc': applyTableChange(sortByColumn(currentTable, currentTable.cursorCol, 'desc')); break;
		}
	}

	function handleContextLinkAction(action: string) {
		if (!view) return;
		const pos = view.state.selection.main.head;
		const line = view.state.doc.lineAt(pos);
		const text = line.text;

		if (action === 'open') {
			// Find wikilink under cursor
			const wikiMatch = text.match(/\[\[([^\]]+)\]\]/);
			if (wikiMatch) {
				// Dispatch a custom event — NotePane handles wikilink navigation
				const event = new CustomEvent('constellation:navigate-link', { detail: { link: wikiMatch[1] } });
				document.dispatchEvent(event);
			}
		} else if (action === 'remove') {
			// Remove link syntax, keep text
			const newText = text.replace(/\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g, '$1')
				.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
			view.dispatch({ changes: { from: line.from, to: line.to, insert: newText } });
		} else if (action === 'edit') {
			// Select the link for editing — find the link bounds
			const wikiStart = text.indexOf('[[');
			const wikiEnd = text.indexOf(']]');
			if (wikiStart >= 0 && wikiEnd >= 0) {
				view.dispatch({ selection: { anchor: line.from + wikiStart, head: line.from + wikiEnd + 2 } });
			}
		}
		view.focus();
	}

	function clearFormatting(v: EditorView) {
		const { from, to } = v.state.selection.main;
		if (from === to) return;
		let text = v.state.sliceDoc(from, to);
		// Strip markdown formatting markers
		text = text.replace(/\*\*(.+?)\*\*/g, '$1')
			.replace(/__(.+?)__/g, '$1')
			.replace(/~~(.+?)~~/g, '$1')
			.replace(/==(.+?)==/g, '$1')
			.replace(/`(.+?)`/g, '$1')
			.replace(/_(.+?)_/g, '$1')
			.replace(/\*(.+?)\*/g, '$1');
		v.dispatch({ changes: { from, to, insert: text }, selection: { anchor: from, head: from + text.length } });
	}

	function showTableGridPicker() {
		if (!view || !containerEl) return;
		const pos = view.state.selection.main.head;
		const coords = view.coordsAtPos(pos);
		if (!coords) return;
		const rect = containerEl.getBoundingClientRect();
		gridPickerX = Math.max(100, Math.min(coords.left - rect.left, rect.width - 100));
		gridPickerY = coords.bottom - rect.top + 4;
		gridPickerVisible = true;
	}

	function insertTableFromGrid(rows: number, cols: number) {
		gridPickerVisible = false;
		if (!view) return;
		const tableStr = generateTable(rows, cols);
		const pos = view.state.selection.main.head;
		const line = view.state.doc.lineAt(pos);
		// Insert on a new line if cursor is not at start of an empty line
		const prefix = line.text.trim() === '' ? '' : '\n';
		view.dispatch({
			changes: { from: line.text.trim() === '' ? line.from : pos, to: line.text.trim() === '' ? line.to : pos, insert: prefix + tableStr + '\n' },
			selection: { anchor: (line.text.trim() === '' ? line.from : pos) + prefix.length + '| '.length }
		});
		view.focus();
	}

	// Expose for command palette / external triggers
	export function openTablePicker() {
		showTableGridPicker();
	}

	function applyTableChange(newTable: ParsedTable | null) {
		if (!view || !newTable || !currentTable) return;
		const startLine = view.state.doc.line(currentTable.startLine);
		const endLine = view.state.doc.line(currentTable.endLine);
		const formatted = formatTable(newTable.rows, newTable.alignments);
		view.dispatch({
			changes: { from: startLine.from, to: endLine.to, insert: formatted }
		});
		currentTable = newTable;
		view.focus();
	}

	function insertFormulaAtCursor(editorView: EditorView, table: ParsedTable) {
		// Insert a SUM formula placeholder at the current cell
		const col = table.cursorCol;
		const colLetter = indexToCol(col);
		const lastDataRow = table.rows.length - 1;
		const formula = `=SUM(${colLetter}1:${colLetter}${lastDataRow})`;
		const row = table.cursorRow;
		const newRows = table.rows.map(r => [...r]);
		newRows[row][col] = formula;
		applyTableChange({ ...table, rows: newRows });
	}

	// Tab navigation in tables
	function tableTab(editorView: EditorView): boolean {
		const table = parseTable(editorView.state, editorView.state.selection.main.head);
		if (!table) return false;
		let nextRow = table.cursorRow;
		let nextCol = table.cursorCol + 1;
		if (nextCol >= table.columnCount) {
			nextCol = 0;
			nextRow++;
			if (nextRow >= table.rows.length) {
				// Add a new row
				const newTable = addRow(table, table.rows.length - 1);
				applyTableChange(newTable);
				return true;
			}
		}
		// Move cursor to next cell — find the position
		const startLine = editorView.state.doc.line(table.startLine);
		const targetLineNum = nextRow === 0 ? table.startLine : (nextRow < table.separatorLineNum - table.startLine ? table.startLine + nextRow : table.startLine + nextRow + 1);
		if (targetLineNum > editorView.state.doc.lines) return true;
		const targetLine = editorView.state.doc.line(targetLineNum);
		const cells = targetLine.text.split('|');
		let offset = 0;
		// Skip leading pipe
		const startsWithPipe = targetLine.text.trimStart().startsWith('|');
		let pipeCount = 0;
		for (let i = 0; i < targetLine.text.length; i++) {
			if (targetLine.text[i] === '|') {
				pipeCount++;
				if (pipeCount === nextCol + (startsWithPipe ? 1 : 0) + 1) {
					// We found the start of the next cell — back up to cell content
					break;
				}
				offset = i + 2; // After pipe + space
			}
		}
		const pos = targetLine.from + Math.min(offset, targetLine.text.length);
		editorView.dispatch({ selection: { anchor: pos } });
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
			if (prevRow < 0) return true; // Already at first cell
		}
		const targetLineNum = prevRow === 0 ? table.startLine : (prevRow < table.separatorLineNum - table.startLine ? table.startLine + prevRow : table.startLine + prevRow + 1);
		if (targetLineNum < 1 || targetLineNum > editorView.state.doc.lines) return true;
		const targetLine = editorView.state.doc.line(targetLineNum);
		const startsWithPipe = targetLine.text.trimStart().startsWith('|');
		let pipeCount = 0;
		let offset = 0;
		for (let i = 0; i < targetLine.text.length; i++) {
			if (targetLine.text[i] === '|') {
				pipeCount++;
				if (pipeCount === prevCol + (startsWithPipe ? 1 : 0) + 1) break;
				offset = i + 2;
			}
		}
		const pos = targetLine.from + Math.min(offset, targetLine.text.length);
		editorView.dispatch({ selection: { anchor: pos } });
		return true;
	}

	// Smart pair wrapping — split into bracket pairs and markdown pairs
	const BRACKET_PAIRS: Record<string, string> = {
		'(': ')', '[': ']', '{': '}', '"': '"', "'": "'",
	};
	const MARKDOWN_PAIRS: Record<string, string> = {
		'`': '`', '_': '_', '*': '*', '~': '~', '=': '=',
	};

	function smartPairKeymap() {
		const pairs = { ...BRACKET_PAIRS, ...(autoPairMarkdown ? MARKDOWN_PAIRS : {}) };
		return Object.entries(pairs).map(([open, close]) => ({
			key: open === "'" ? "'" : open,
			run: (view: EditorView) => {
				const { state } = view;
				const { from, to } = state.selection.main;
				if (from === to) {
					// No selection: auto-close
					view.dispatch({
						changes: { from, to, insert: open + close },
						selection: { anchor: from + 1 }
					});
					return true;
				}
				// Selection: wrap
				const selected = state.sliceDoc(from, to);
				// Special: upgrade [text] → [[text]]
				if (open === '[' && selected.startsWith('[') && selected.endsWith(']')) {
					const inner = selected.slice(1, -1);
					view.dispatch({
						changes: { from, to, insert: '[[' + inner + ']]' },
						selection: { anchor: from, head: from + inner.length + 4 }
					});
					return true;
				}
				view.dispatch({
					changes: { from, to, insert: open + selected + close },
					selection: { anchor: from + 1, head: to + 1 }
				});
				return true;
			}
		}));
	}

	// Smart list continuation
	function smartEnter(view: EditorView): boolean {
		const { state } = view;
		const { from } = state.selection.main;
		const line = state.doc.lineAt(from);
		const lineText = line.text;

		// Match list patterns
		const orderedMatch = lineText.match(/^(\s*)(\d+)\.\s(.*)/);
		const unorderedMatch = lineText.match(/^(\s*)([-*+])\s(.*)/);
		const taskMatch = lineText.match(/^(\s*)([-*+])\s\[[ x]\]\s(.*)/);

		if (taskMatch) {
			if (!taskMatch[3].trim()) {
				// Empty task item: remove prefix
				view.dispatch({ changes: { from: line.from, to: line.to, insert: '' } });
				return true;
			}
			const indent = taskMatch[1];
			const bullet = taskMatch[2];
			view.dispatch({
				changes: { from, to: from, insert: `\n${indent}${bullet} [ ] ` },
				selection: { anchor: from + indent.length + bullet.length + 6 }
			});
			return true;
		}
		if (unorderedMatch) {
			if (!unorderedMatch[3].trim()) {
				view.dispatch({ changes: { from: line.from, to: line.to, insert: '' } });
				return true;
			}
			const indent = unorderedMatch[1];
			const bullet = unorderedMatch[2];
			view.dispatch({
				changes: { from, to: from, insert: `\n${indent}${bullet} ` },
				selection: { anchor: from + indent.length + bullet.length + 3 }
			});
			return true;
		}
		if (orderedMatch) {
			if (!orderedMatch[3].trim()) {
				view.dispatch({ changes: { from: line.from, to: line.to, insert: '' } });
				return true;
			}
			const indent = orderedMatch[1];
			const nextNum = parseInt(orderedMatch[2]) + 1;
			view.dispatch({
				changes: { from, to: from, insert: `\n${indent}${nextNum}. ` },
				selection: { anchor: from + indent.length + String(nextNum).length + 4 }
			});
			return true;
		}
		return false;
	}

	// Toggle checkbox
	function toggleCheckbox(view: EditorView): boolean {
		const { state } = view;
		const { from } = state.selection.main;
		const line = state.doc.lineAt(from);
		const match = line.text.match(/^(\s*[-*+]\s)\[( |x)\]/);
		if (match) {
			const checkStart = line.from + match[1].length + 1;
			const newChar = match[2] === ' ' ? 'x' : ' ';
			view.dispatch({ changes: { from: checkStart, to: checkStart + 1, insert: newChar } });
			return true;
		}
		return false;
	}

	// Wikilink autocomplete
	function wikilinkCompletion(context: CompletionContext): any {
		const before = context.matchBefore(/\[\[[^\]]*$/);
		if (!before) return null;
		const inner = before.text.slice(2);

		// MIG-067 §D — legacy `[[note|type:X]]` completion (obsolete taxonomy)
		// retired; canonical typed-link autocomplete lives in completions.ts.

		// Heading autocomplete: [[note#query
		const hashIdx = inner.indexOf('#');
		if (hashIdx >= 0) {
			const noteName = inner.slice(0, hashIdx);
			const headingQuery = inner.slice(hashIdx + 1).toLowerCase();
			return headingCompletion(context, before, noteName, headingQuery);
		}

		const query = inner.toLowerCase();
		const options: Completion[] = noteNames
			.filter(n => n.name.toLowerCase().includes(query))
			.slice(0, 20)
			.map(n => ({
				label: n.name,
				detail: n.libraryName ? ` — ${n.libraryName}` : undefined,
				type: 'text',
				apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
					/* Consume any trailing ]] left by closeBrackets */
					let end = to;
					const after = view.state.doc.sliceString(to, Math.min(to + 2, view.state.doc.length));
					if (after === ']]') end = to + 2;
					else if (after[0] === ']') end = to + 1;
					const insert = `[[${n.name}]]`;
					view.dispatch({
						changes: { from: before.from, to: end, insert },
						selection: { anchor: before.from + insert.length }
					});
				}
			}));
		return { from: before.from, options, filter: false };
	}

	// Heading autocomplete after [[note#
	async function headingCompletion(context: CompletionContext, before: any, noteName: string, headingQuery: string) {
		// Resolve the note to get its path
		const resolved = await resolveWikilinkCrossLibrary(libraryPath, noteName);
		if (!resolved) return null;

		const headings = await getNoteHeadings(resolved.path);
		const options: Completion[] = headings
			.filter(h => h.toLowerCase().includes(headingQuery))
			.slice(0, 20)
			.map(h => ({
				label: h,
				type: 'text',
				apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
					const insert = `[[${noteName}#${h}]]`;
					view.dispatch({
						changes: { from: before.from, to, insert },
						selection: { anchor: before.from + insert.length }
					});
				}
			}));
		return { from: before.from, options, filter: false };
	}

	// Tag + slash autocomplete (shared factories from $lib/editor/completions)
	const tagCompletion = createTagCompletion(() => allTags);
	const slashCompletion = createSlashCompletion();

	// Line operations
	function deleteLine(view: EditorView): boolean {
		const { state } = view;
		const { from } = state.selection.main;
		const line = state.doc.lineAt(from);
		const delTo = line.to < state.doc.length ? line.to + 1 : line.from > 0 ? line.from - 1 : line.to;
		const delFrom = line.to < state.doc.length ? line.from : (line.from > 0 ? line.from - 1 : line.from);
		view.dispatch({ changes: { from: delFrom, to: delTo } });
		return true;
	}

	function moveLine(view: EditorView, direction: 'up' | 'down'): boolean {
		const { state } = view;
		const { from, to } = state.selection.main;
		const startLine = state.doc.lineAt(from);
		const endLine = state.doc.lineAt(to);

		if (direction === 'up' && startLine.number === 1) return true;
		if (direction === 'down' && endLine.number === state.doc.lines) return true;

		if (direction === 'up') {
			const prevLine = state.doc.line(startLine.number - 1);
			const blockText = state.sliceDoc(startLine.from, endLine.to);
			view.dispatch({
				changes: [
					{ from: prevLine.from, to: endLine.to, insert: blockText + '\n' + prevLine.text }
				],
				selection: { anchor: from - prevLine.text.length - 1, head: to - prevLine.text.length - 1 }
			});
		} else {
			const nextLine = state.doc.line(endLine.number + 1);
			const blockText = state.sliceDoc(startLine.from, endLine.to);
			view.dispatch({
				changes: [
					{ from: startLine.from, to: nextLine.to, insert: nextLine.text + '\n' + blockText }
				],
				selection: { anchor: from + nextLine.text.length + 1, head: to + nextLine.text.length + 1 }
			});
		}
		return true;
	}

	// Formatting commands
	function wrapSelection(view: EditorView, before: string, after: string): boolean {
		const { state } = view;
		const { from, to } = state.selection.main;
		if (from === to) {
			// No selection: insert markers, cursor between
			view.dispatch({
				changes: { from, to, insert: before + after },
				selection: { anchor: from + before.length }
			});
		} else {
			const selected = state.sliceDoc(from, to);
			// Check if already wrapped — unwrap
			if (selected.startsWith(before) && selected.endsWith(after)) {
				const inner = selected.slice(before.length, -after.length);
				view.dispatch({
					changes: { from, to, insert: inner },
					selection: { anchor: from, head: from + inner.length }
				});
			} else {
				view.dispatch({
					changes: { from, to, insert: before + selected + after },
					selection: { anchor: from + before.length, head: to + before.length }
				});
			}
		}
		return true;
	}

	// HTML tag wrapping (for font, color, sub, sup, underline)
	function wrapWithHtmlTag(view: EditorView, tagOpen: string, tagClose: string): boolean {
		const { state } = view;
		const { from, to } = state.selection.main;
		if (from === to) {
			view.dispatch({
				changes: { from, to, insert: tagOpen + tagClose },
				selection: { anchor: from + tagOpen.length }
			});
		} else {
			const selected = state.sliceDoc(from, to);
			// Check if already wrapped with this tag — unwrap
			if (selected.startsWith(tagOpen) && selected.endsWith(tagClose)) {
				const inner = selected.slice(tagOpen.length, -tagClose.length);
				view.dispatch({
					changes: { from, to, insert: inner },
					selection: { anchor: from, head: from + inner.length }
				});
			} else {
				view.dispatch({
					changes: { from, to, insert: tagOpen + selected + tagClose },
					selection: { anchor: from + tagOpen.length, head: to + tagOpen.length }
				});
			}
		}
		return true;
	}

	function applyFontFamily(view: EditorView, fontValue: string): boolean {
		if (!fontValue) {
			// Remove font-family span if selection is inside one
			const { from, to } = view.state.selection.main;
			const selected = view.state.sliceDoc(from, to);
			const spanMatch = selected.match(/^<span style="font-family:[^"]*">([\s\S]*)<\/span>$/);
			if (spanMatch) {
				view.dispatch({
					changes: { from, to, insert: spanMatch[1] },
					selection: { anchor: from, head: from + spanMatch[1].length }
				});
				return true;
			}
			return false;
		}
		const safeFont = fontValue.replace(/"/g, "'");
		return wrapWithHtmlTag(view, `<span style="font-family: ${safeFont}">`, '</span>');
	}

	function applyTextColor(view: EditorView, color: string): boolean {
		return wrapWithHtmlTag(view, `<span style="color: ${color}">`, '</span>');
	}

	function removeTextColor(view: EditorView): boolean {
		const { from, to } = view.state.selection.main;
		const selected = view.state.sliceDoc(from, to);
		const spanMatch = selected.match(/^<span style="color:[^"]*">([\s\S]*)<\/span>$/);
		if (spanMatch) {
			view.dispatch({
				changes: { from, to, insert: spanMatch[1] },
				selection: { anchor: from, head: from + spanMatch[1].length }
			});
			return true;
		}
		return false;
	}

	function applySubscript(view: EditorView): boolean {
		return wrapWithHtmlTag(view, '<sub>', '</sub>');
	}

	function applySuperscript(view: EditorView): boolean {
		return wrapWithHtmlTag(view, '<sup>', '</sup>');
	}

	function applyUnderline(view: EditorView): boolean {
		return wrapWithHtmlTag(view, '<u>', '</u>');
	}

	// Font picker state
	let showFontPicker = $state(false);
	let showColorPicker = $state(false);

	// ─── Tashkeel (Arabic diacritics) highlighting ───
	let tashkeelHighlight = $state(false);
	const tashkeelCompartment = new Compartment();

	// Regex matching Arabic diacritical marks (harakat)
	const TASHKEEL_REGEX = /[\u064B-\u0652\u0670\u06D6-\u06DC\u06DF-\u06E4\u06E7\u06E8\u06EA-\u06ED]/g;

	const tashkeelDeco = Decoration.mark({ class: 'cm-tashkeel' });

	const tashkeelPlugin = ViewPlugin.fromClass(class {
		decorations: DecorationSet;
		constructor(view: EditorView) {
			this.decorations = this.buildDecos(view);
		}
		update(update: ViewUpdate) {
			if (update.docChanged || update.viewportChanged) {
				this.decorations = this.buildDecos(update.view);
			}
		}
		buildDecos(view: EditorView): DecorationSet {
			const builder: Range<Decoration>[] = [];
			const { from, to } = view.viewport;
			const text = view.state.doc.sliceString(from, to);
			let match: RegExpExecArray | null;
			TASHKEEL_REGEX.lastIndex = 0;
			while ((match = TASHKEEL_REGEX.exec(text)) !== null) {
				const pos = from + match.index;
				builder.push(tashkeelDeco.range(pos, pos + match[0].length));
			}
			return Decoration.set(builder, true);
		}
	}, { decorations: v => v.decorations });

	// ─── Script-specific symbol toolbars ───
	const SCRIPT_TOOLBARS: Record<string, { symbols: string[]; punctuation: string[]; numerals?: string[]; quran?: string[] }> = {
		arabic: {
			symbols: ['﷽', 'ﷺ', 'ﷻ', '۞', '﴿', '﴾', 'ۖ', 'ۗ', 'ۘ', 'ۙ', 'ۚ', 'ۛ', 'ۜ'],
			punctuation: ['،', '؛', '؟', '٪', '«', '»', 'ـ', '﴾﴿'],
			numerals: ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'],
			quran: ['﴿', '﴾', '۞', 'ۖ', 'ۗ', 'ۘ', 'ۙ', 'ۚ', 'ۛ', 'ۜ', '﷽', 'ﷺ', 'ﷻ'],
		},
		hebrew: {
			symbols: ['׳', '״', '־', '׀', '׆'],
			punctuation: ['׃', '׳', '״', '־', '–', '—'],
			numerals: [],
		},
		cjk: {
			symbols: ['〇', '々', '〆', '〒', '〓', '〔', '〕', '〖', '〗'],
			punctuation: ['。', '、', '！', '？', '「', '」', '『', '』', '【', '】', '（', '）', '《', '》', '〈', '〉', '・', '：', '；'],
			numerals: [],
		},
		cyrillic: {
			symbols: ['№', '§', '©', '®', '™', '°'],
			punctuation: ['\u00AB', '\u00BB', '\u2014', '\u2013', '\u2026', '\u201E', '\u201C'],
			numerals: [],
		},
		devanagari: {
			symbols: ['ॐ', '॰', '₹', '꣸', '꣹', '꣺'],
			punctuation: ['।', '॥', '॰'],
			numerals: ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'],
		},
		latin: {
			symbols: ['©', '®', '™', '†', '‡', '§', '¶', '•', '°', '′', '″', '∞', '√', '∑', '∏', '∫', 'Δ', 'π', 'Ω'],
			punctuation: ['\u2013', '\u2014', '\u2026', '\u00AB', '\u00BB', '\u201C', '\u201D', '\u2018', '\u2019', '\u2039', '\u203A'],
			numerals: [],
		},
	};

	let showScriptToolbar = $state(false);
	let scriptToolbarScript = $state('');

	function insertSymbol(sym: string) {
		if (!view) return;
		const { from, to } = view.state.selection.main;
		view.dispatch({ changes: { from, to, insert: sym } });
		view.focus();
	}

	// Determine which script toolbars to show — contextual based on note direction
	const rtlScripts = ['arabic', 'hebrew'];
	let activeScriptToolbars = $derived.by(() => {
		const s = $appSettings;
		if (!s.enableScriptToolbar) return [];
		const scripts = s.scriptToolbarScripts || [];
		if (dir === 'rtl') {
			// RTL notes: show Arabic/Hebrew if in user's scripts
			return scripts.filter(sc => rtlScripts.includes(sc));
		} else {
			// LTR notes: always show latin, plus any non-RTL scripts the user enabled
			const ltr = scripts.filter(sc => !rtlScripts.includes(sc));
			if (!ltr.includes('latin')) ltr.unshift('latin');
			return ltr;
		}
	});

	const fontFamilies = [
		{ label: 'Default', value: '' },
		{ label: 'Sans Serif', value: "Inter, -apple-system, sans-serif" },
		{ label: 'Serif', value: "Georgia, 'Times New Roman', serif" },
		{ label: 'Monospace', value: "'Cascadia Code', 'Fira Code', Consolas, monospace" },
		{ label: 'Amiri', value: "Amiri, serif" },
		{ label: 'Noto Sans Arabic', value: "'Noto Sans Arabic', sans-serif" },
		{ label: 'Noto Naskh Arabic', value: "'Noto Naskh Arabic', serif" },
		{ label: 'Cairo', value: "Cairo, sans-serif" },
		{ label: 'Tajawal', value: "Tajawal, sans-serif" },
		{ label: 'Lora', value: "Lora, serif" },
		{ label: 'Merriweather', value: "Merriweather, serif" },
		{ label: 'Roboto', value: "Roboto, sans-serif" },
		{ label: 'Open Sans', value: "'Open Sans', sans-serif" },
	];

	const colorPalette = [
		'#000000', '#434343', '#666666', '#999999', '#cccccc',
		'#ef4444', '#f97316', '#eab308', '#22c55e', '#14b8a6',
		'#3b82f6', '#6366f1', '#a855f7', '#ec4899', '#f43f5e',
		'#7c2d12', '#854d0e', '#166534', '#1e40af', '#581c87',
	];

	// Duplicate current line
	function duplicateLine(view: EditorView): boolean {
		const { state } = view;
		const { from } = state.selection.main;
		const line = state.doc.lineAt(from);
		view.dispatch({
			changes: { from: line.to, to: line.to, insert: '\n' + line.text },
			selection: { anchor: from + line.text.length + 1 }
		});
		return true;
	}

	// Toggle block comment %%...%%
	function toggleComment(view: EditorView): boolean {
		const { state } = view;
		const { from, to } = state.selection.main;
		if (from !== to) {
			// Selection: wrap/unwrap in %%
			const selected = state.sliceDoc(from, to);
			if (selected.startsWith('%%') && selected.endsWith('%%')) {
				const inner = selected.slice(2, -2);
				view.dispatch({ changes: { from, to, insert: inner }, selection: { anchor: from, head: from + inner.length } });
			} else {
				view.dispatch({ changes: { from, to, insert: '%%' + selected + '%%' }, selection: { anchor: from + 2, head: to + 2 } });
			}
		} else {
			// No selection: wrap/unwrap current line
			const line = state.doc.lineAt(from);
			const text = line.text;
			if (text.trimStart().startsWith('%%') && text.trimEnd().endsWith('%%')) {
				const inner = text.replace(/^\s*%%/, '').replace(/%%\s*$/, '');
				view.dispatch({ changes: { from: line.from, to: line.to, insert: inner } });
			} else {
				view.dispatch({ changes: { from: line.from, to: line.to, insert: '%%' + text + '%%' } });
			}
		}
		return true;
	}

	// Paste as plain text
	function pasteAsPlainText(view: EditorView): boolean {
		navigator.clipboard.readText().then(text => {
			if (text) {
				const { from, to } = view.state.selection.main;
				view.dispatch({
					changes: { from, to, insert: text },
					selection: { anchor: from + text.length }
				});
			}
		}).catch(() => {});
		return true;
	}

	// Markdown heading/list fold service
	function markdownFoldService(state: EditorState, lineStart: number, lineEnd: number): { from: number; to: number } | null {
		const line = state.doc.lineAt(lineStart);
		const text = line.text;

		// Heading folding: # Heading
		const headingMatch = text.match(/^(#{1,6})\s/);
		if (headingMatch) {
			const level = headingMatch[1].length;
			let endLine = line;
			for (let i = line.number + 1; i <= state.doc.lines; i++) {
				const nextLine = state.doc.line(i);
				const nextHeading = nextLine.text.match(/^(#{1,6})\s/);
				if (nextHeading && nextHeading[1].length <= level) break;
				endLine = nextLine;
			}
			if (endLine.number > line.number) {
				return { from: line.to, to: endLine.to };
			}
		}

		// List item folding: fold deeper-indented children
		const listMatch = text.match(/^(\s*)([-*+]|\d+\.)\s/);
		if (listMatch) {
			const indent = listMatch[1].length;
			let endLine = line;
			for (let i = line.number + 1; i <= state.doc.lines; i++) {
				const nextLine = state.doc.line(i);
				if (nextLine.text.trim() === '') { endLine = nextLine; continue; }
				const nextListMatch = nextLine.text.match(/^(\s*)/);
				const nextIndent = nextListMatch ? nextListMatch[1].length : 0;
				if (nextIndent <= indent) break;
				endLine = nextLine;
			}
			if (endLine.number > line.number) {
				return { from: line.to, to: endLine.to };
			}
		}

		return null;
	}

	const editorKeymap = keymap.of([
		{ key: 'Enter', run: smartEnter },
		{ key: 'Ctrl-Enter', run: toggleCheckbox },
		{ key: 'Ctrl-b', run: (v) => wrapSelection(v, '**', '**') },
		{ key: 'Ctrl-i', run: (v) => wrapSelection(v, '_', '_') },
		{ key: 'Ctrl-u', run: (v) => applyUnderline(v) },
		{ key: 'Ctrl-Shift-s', run: (v) => wrapSelection(v, '~~', '~~') },
		{ key: 'Ctrl-Shift-h', run: (v) => wrapSelection(v, '==', '==') },
		{ key: 'Ctrl-`', run: (v) => wrapSelection(v, '`', '`') },
		{ key: 'Ctrl-Shift-k', run: deleteLine },
		{ key: 'Alt-ArrowUp', run: (v) => moveLine(v, 'up') },
		{ key: 'Alt-ArrowDown', run: (v) => moveLine(v, 'down') },
		{ key: 'Ctrl-f', run: (v) => { openSearchPanel(v); return true; } },
		{ key: 'Ctrl-k', run: (v) => wrapSelection(v, '[[', ']]') },
		{ key: 'Ctrl-Shift-v', run: pasteAsPlainText },
		{ key: 'Ctrl-d', run: selectNextOccurrence },
		{ key: 'Ctrl-Shift-d', run: duplicateLine },
		{ key: 'Ctrl-/', run: toggleComment },
		...smartPairKeymap(),
	]);

	// Editor theme
	const editorTheme = EditorView.theme({
		'&': {
			fontSize: 'var(--font-text-size, 17px)',
		},
		'.cm-content': {
			fontFamily: 'var(--library-text-font, var(--font-text-theme, inherit))',
			fontSize: 'var(--library-font-size, var(--font-text-size, 17px))',
			lineHeight: '1.7',
			padding: '0',
			caretColor: 'var(--interactive-accent)',
		},
		'.cm-cursor': {
			borderLeftColor: 'var(--interactive-accent)',
		},
		'.cm-gutters': {
			backgroundColor: 'transparent',
			border: 'none',
			color: 'var(--color-base-40)',
			fontSize: '0.75rem',
		},
		'.cm-activeLineGutter': {
			backgroundColor: 'transparent',
			color: 'var(--text-faint)',
		},
		'.cm-activeLine': {
			backgroundColor: 'transparent',
		},
		'&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
			backgroundColor: 'rgba(255, 170, 0, 0.35) !important',
		},
		'.cm-line': {
			padding: '0 4px',
		},
		'.cm-tooltip-autocomplete': {
			border: '1px solid var(--background-modifier-border)',
			borderRadius: '6px',
			boxShadow: 'var(--shadow-l)',
			backgroundColor: 'var(--background-primary)',
			overflow: 'hidden',
		},
		'.cm-tooltip-autocomplete > ul > li': {
			padding: '4px 8px',
			fontSize: '0.82rem',
		},
		'.cm-tooltip-autocomplete > ul > li[aria-selected]': {
			backgroundColor: 'var(--interactive-accent)',
			color: 'var(--text-on-accent)',
		},
		'.cm-panels': {
			backgroundColor: 'var(--background-secondary)',
			borderBottom: '1px solid var(--background-modifier-border)',
		},
		'.cm-panel.cm-search': {
			padding: '4px 8px',
		},
		'.cm-panel.cm-search input': {
			border: '1px solid var(--background-modifier-border-focus)',
			borderRadius: '4px',
			padding: '2px 6px',
			fontSize: '0.82rem',
			backgroundColor: 'var(--background-primary)',
			color: 'var(--text-normal)',
		},
		'.cm-panel.cm-search button': {
			border: '1px solid var(--background-modifier-border-focus)',
			borderRadius: '4px',
			padding: '2px 8px',
			backgroundColor: 'var(--background-primary)',
			color: 'var(--text-normal)',
			cursor: 'pointer',
			fontSize: '0.8rem',
		},
		'.cm-searchMatch': {
			backgroundColor: 'color-mix(in srgb, var(--color-yellow) 35%, transparent)',
			outline: '1px solid var(--color-yellow)',
		},
		'.cm-searchMatch-selected': {
			backgroundColor: 'color-mix(in srgb, var(--color-green) 25%, transparent)',
			outline: '1px solid var(--color-green)',
		},
		'.cm-foldGutter span': {
			fontSize: '0.7rem',
			color: 'var(--color-base-40)',
			cursor: 'pointer',
		},
	});

	// Clipboard image paste handler
	function clipboardImagePaste() {
		return EditorView.domEventHandlers({
			paste: (event: ClipboardEvent, editorView: EditorView) => {
				const items = event.clipboardData?.items;
				if (!items) return false;

				// Check for images first
				if (libraryPath) {
					for (const item of items) {
						if (item.type.startsWith('image/')) {
							event.preventDefault();
							const blob = item.getAsFile();
							if (!blob) return true;

							const reader = new FileReader();
							reader.onload = async () => {
								const base64 = reader.result as string;
								try {
									const filename = await saveClipboardImage(libraryPath, base64);
									const embed = `![[${filename}]]`;
									const pos = editorView.state.selection.main.from;
									editorView.dispatch({
										changes: { from: pos, to: pos, insert: embed },
										selection: { anchor: pos + embed.length }
									});
								} catch (err) {
									console.error('Failed to paste image:', err);
								}
							};
							reader.readAsDataURL(blob);
							return true;
						}
					}
				}

				// Check for tabular text (TSV/CSV from spreadsheets)
				const text = event.clipboardData?.getData('text/plain');
				if (text) {
					const format = detectTabularText(text);
					if (format) {
						const lines = text.trim().split('\n');
						// Only auto-convert if it looks clearly tabular (2+ rows, 2+ columns)
						const colCount = lines[0].split(format === 'tsv' ? '\t' : ',').length;
						if (lines.length >= 2 && colCount >= 2) {
							event.preventDefault();
							const tableStr = tabularTextToTable(text, format);
							const { from, to } = editorView.state.selection.main;
							const line = editorView.state.doc.lineAt(from);
							const prefix = line.text.trim() === '' && from === to ? '' : '\n';
							const insertFrom = line.text.trim() === '' && from === to ? line.from : from;
							const insertTo = line.text.trim() === '' && from === to ? line.to : to;
							editorView.dispatch({
								changes: { from: insertFrom, to: insertTo, insert: prefix + tableStr + '\n' }
							});
							return true;
						}
					}
				}

				return false;
			}
		});
	}

	// Indentation guides ViewPlugin — vertical lines for nested list indentation
	const indentGuidesPlugin = ViewPlugin.fromClass(class {
		decorations: DecorationSet;
		constructor(view: EditorView) {
			this.decorations = this.buildGuides(view);
		}
		update(update: ViewUpdate) {
			if (update.docChanged || update.viewportChanged) {
				this.decorations = this.buildGuides(update.view);
			}
		}
		buildGuides(view: EditorView): DecorationSet {
			const widgets: Range<Decoration>[] = [];
			for (const { from, to } of view.visibleRanges) {
				for (let pos = from; pos <= to;) {
					const line = view.state.doc.lineAt(pos);
					const text = line.text;
					// Count leading whitespace
					let indent = 0;
					for (let i = 0; i < text.length; i++) {
						if (text[i] === '\t') indent += view.state.tabSize;
						else if (text[i] === ' ') indent += 1;
						else break;
					}
					// Add guide marks for each indentation level
					const tabW = view.state.tabSize;
					const levels = Math.floor(indent / tabW);
					if (levels > 0) {
						widgets.push(Decoration.line({ class: `cm-indent-guide cm-indent-${Math.min(levels, 10)}` }).range(line.from));
					}
					pos = line.to + 1;
				}
			}
			return Decoration.set(widgets, true);
		}
	}, { decorations: v => v.decorations });

	onMount(() => {
		const startState = EditorState.create({
			doc: value,
			extensions: [
				// ─── CORE ONLY: bare minimum for fast typing ───
				history(),
				drawSelection(),
				syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
				markdown({ base: markdownLanguage, extensions: [HighlightExt] }),
				keymap.of([
					indentWithTab,
					...defaultKeymap,
					...historyKeymap,
				]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir })),
				cmPlaceholder(placeholder),
				editorTheme,
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !updating) {
						const text = update.state.doc.toString();
						lastInternalValue = text;
						onchange(text);
					}
				}),
			]
		});
		view = new EditorView({ state: startState, parent: containerEl });

		// Restore cursor position and scroll
		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
		}
		if (initialScrollTop > 0) {
			requestAnimationFrame(() => {
				view?.scrollDOM.scrollTo({ top: initialScrollTop });
			});
		}

		// Auto-focus editor so cursor is visible
		requestAnimationFrame(() => view?.focus());

		// Event listeners disabled — plugins stripped for performance baseline
	});

	// ─── Prop→Editor sync: NO $effect for value ───
	// Editor owns its content after mount. Value prop only used in onMount.
	// Tab switches destroy/recreate the component with new value.

	// Only sync dir changes (rare — user switches note direction)
	let prevDir = dir;
	$effect(() => {
		if (!view || dir === prevDir) return;
		view.dispatch({ effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir })) });
		prevDir = dir;
	});

	// Font sync: separate because it reads $appSettings store (different reactivity)
	let prevFontKey = '';
	$effect(() => {
		const s = $appSettings;
		if (!view) return;
		// Build a key from font-relevant settings to detect actual changes
		const fontKey = `${s.textFont}|${s.monoFont}|${s.fontSize}|${s.fontMode}|${s.activeFontSetId}|${JSON.stringify(s.scriptFonts)}`;
		if (fontKey !== prevFontKey) {
			prevFontKey = fontKey;
			view.dispatch({ effects: fontCompartment.reconfigure(buildFontTheme()) });
		}
	});

	// Listen for fold-all / unfold-all events from command palette
	function handleFoldAll() { if (view) foldAll(view); }
	function handleUnfoldAll() { if (view) unfoldAll(view); }
	function handleInsertTable(e: Event) {
		const detail = (e as CustomEvent).detail;
		if (detail?.rows && detail?.cols) {
			insertTableFromGrid(detail.rows, detail.cols);
		} else {
			showTableGridPicker();
		}
	}

	// Check if the current selection looks like tabular data (for Convert to Table button)
	let selectionIsTabular = $state(false);

	function checkSelectionTabular(editorView: EditorView) {
		const { from, to } = editorView.state.selection.main;
		if (from === to) { selectionIsTabular = false; return; }
		const text = editorView.state.sliceDoc(from, to);
		selectionIsTabular = detectTabularText(text) !== null;
	}

	function convertSelectionToTable() {
		if (!view) return;
		const { from, to } = view.state.selection.main;
		if (from === to) return;
		const text = view.state.sliceDoc(from, to);
		const format = detectTabularText(text);
		if (!format) return;
		const tableStr = tabularTextToTable(text, format);
		view.dispatch({ changes: { from, to, insert: tableStr } });
		view.focus();
	}

	onDestroy(() => {
		view?.destroy();
	});

	export function focus() {
		view?.focus();
	}

	export function getView(): EditorView | undefined {
		return view;
	}
</script>

<div class="cm-wrapper">
	<!-- Persistent toolbar -->
	{#if view}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="cm-toolbar" dir={dir} style="direction: {dir}" onmousedown={(e) => e.preventDefault()}>
			<!-- Undo/Redo — arrows flip in RTL -->
			<button class="cm-tb" title="Undo (Ctrl+Z)" onclick={() => { undo(view!) }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="cm-icon-dir"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6.69 3L3 13"/></svg>
			</button>
			<button class="cm-tb" title="Redo (Ctrl+Y)" onclick={() => { redo(view!) }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="cm-icon-dir"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6.69 3L21 13"/></svg>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Heading -->
			<button class="cm-tb" class:cm-tb-active={toolbarContext === 'heading' && toolbarHeadingLevel === 1} title="Heading" onclick={() => applyHeading(1)}>H1</button>
			<button class="cm-tb" class:cm-tb-active={toolbarContext === 'heading' && toolbarHeadingLevel === 2} title="Heading 2" onclick={() => applyHeading(2)}>H2</button>
			<button class="cm-tb" class:cm-tb-active={toolbarContext === 'heading' && toolbarHeadingLevel === 3} title="Heading 3" onclick={() => applyHeading(3)}>H3</button>
			<span class="cm-tb-sep"></span>
			<!-- Text formatting -->
			<button class="cm-tb" title="Bold (Ctrl+B)" onclick={() => { wrapSelection(view!, '**', '**') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/></svg>
			</button>
			<button class="cm-tb" title="Italic (Ctrl+I)" onclick={() => { wrapSelection(view!, '_', '_') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg>
			</button>
			<button class="cm-tb" title="Underline (Ctrl+U)" onclick={() => { applyUnderline(view!) }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 3v7a6 6 0 0 0 6 6 6 6 0 0 0 6-6V3"/><line x1="4" y1="21" x2="20" y2="21"/></svg>
			</button>
			<button class="cm-tb" title="Strikethrough" onclick={() => { wrapSelection(view!, '~~', '~~') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4H9a3 3 0 0 0-2.83 4"/><path d="M14 12a4 4 0 0 1 0 8H6"/><line x1="4" y1="12" x2="20" y2="12"/></svg>
			</button>
			<button class="cm-tb" title="Highlight" onclick={() => { wrapSelection(view!, '==', '==') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 11-6 6v3h9l3-3"/><path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"/></svg>
			</button>
			<button class="cm-tb" title="Text color" onclick={() => showColorPicker = !showColorPicker}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16"/><path d="M9.5 4L4.5 16h2l1-3h7l1 3h2L12.5 4z"/></svg>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Alignment — icons mirror in RTL -->
			<button class="cm-tb" title="Align left" onclick={() => { /* future */ }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="cm-icon-dir"><line x1="17" y1="10" x2="3" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="17" y1="18" x2="3" y2="18"/></svg>
			</button>
			<button class="cm-tb" title="Align center" onclick={() => { /* future */ }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="10" x2="6" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="18" y1="18" x2="6" y2="18"/></svg>
			</button>
			<button class="cm-tb" title="Align right" onclick={() => { /* future */ }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="cm-icon-dir"><line x1="21" y1="10" x2="7" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="21" y1="18" x2="7" y2="18"/></svg>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Lists — markers flip in RTL -->
			<button class="cm-tb" title="Bullet list" onclick={() => { handleContextList('bullet') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="cm-icon-dir"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><circle cx="3" cy="6" r="1" fill="currentColor"/><circle cx="3" cy="12" r="1" fill="currentColor"/><circle cx="3" cy="18" r="1" fill="currentColor"/></svg>
			</button>
			<button class="cm-tb" title="Numbered list" onclick={() => { handleContextList('numbered') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="cm-icon-dir"><line x1="10" y1="6" x2="21" y2="6"/><line x1="10" y1="12" x2="21" y2="12"/><line x1="10" y1="18" x2="21" y2="18"/><path d="M4 6h1v4"/><path d="M4 10h2"/><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/></svg>
			</button>
			<button class="cm-tb" title="Task list" onclick={() => { handleContextList('task') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Blocks & inserts -->
			<button class="cm-tb" title="Blockquote" onclick={() => { handleContextInsert('blockquote') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"/><path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3c0 1 0 1 1 1z"/></svg>
			</button>
			<button class="cm-tb" title="Code block" onclick={() => { handleContextInsert('codeBlock') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
			</button>
			<button class="cm-tb" title="Horizontal rule" onclick={() => { handleContextInsert('horizontalRule') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="2" y1="12" x2="22" y2="12"/></svg>
			</button>
			<button class="cm-tb" title="Table" onclick={() => showTableGridPicker()}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/></svg>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Link & Image -->
			<button class="cm-tb" title="Link (Ctrl+K)" onclick={() => { wrapSelection(view!, '[[', ']]') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
			</button>
			<button class="cm-tb" title="Image" onclick={() => { handleContextInsert('image') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Inline code, sub, sup -->
			<button class="cm-tb" title="Inline code" onclick={() => { wrapSelection(view!, '`', '`') }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 18l6-6-6-6"/><path d="M8 6l-6 6 6 6"/></svg>
			</button>
			<button class="cm-tb" title="Subscript" onclick={() => { applySubscript(view!) }}>
				<span style="font-size:11px;font-weight:600;">X<sub style="font-size:8px">₂</sub></span>
			</button>
			<button class="cm-tb" title="Superscript" onclick={() => { applySuperscript(view!) }}>
				<span style="font-size:11px;font-weight:600;">X<sup style="font-size:8px">²</sup></span>
			</button>
			<span class="cm-tb-sep"></span>
			<!-- Font picker -->
			<button class="cm-tb cm-tb-text" title="Font" onclick={() => showFontPicker = !showFontPicker}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>
			</button>
			<!-- Find -->
			<button class="cm-tb" title="Find (Ctrl+F)" onclick={() => { if (view) openSearchPanel(view); }}>
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
			</button>
			<!-- Script-specific symbols integrated into toolbar -->
			{#if activeScriptToolbars.length > 0}
				<span class="cm-tb-sep"></span>
				{#each activeScriptToolbars as script}
					{@const data = SCRIPT_TOOLBARS[script]}
					{#if data}
						<div class="cm-script-group">
							<button class="cm-script-label" onclick={() => { scriptToolbarScript = scriptToolbarScript === script ? '' : script; }}>
								{script === 'arabic' ? 'عربي' : script === 'hebrew' ? 'עברית' : script === 'cjk' ? '中日韩' : script === 'cyrillic' ? 'Кир' : script === 'devanagari' ? 'देव' : 'Aa'}
								<svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class:rotated={scriptToolbarScript === script}><path d="M6 9l6 6 6-6"/></svg>
							</button>
							{#if scriptToolbarScript === script}
								<div class="cm-script-panel">
									{#if data.quran && script === 'arabic'}
										<div class="cm-script-section">
											<span class="cm-script-section-label">{$t('scriptToolbar.quranSymbols')}</span>
											<div class="cm-script-chars">
												{#each data.quran as sym}
													<button class="cm-script-char" onclick={() => insertSymbol(sym)} title={sym}>{sym}</button>
												{/each}
											</div>
										</div>
									{/if}
									<div class="cm-script-section">
										<span class="cm-script-section-label">{$t('scriptToolbar.symbols')}</span>
										<div class="cm-script-chars">
											{#each data.symbols as sym}
												<button class="cm-script-char" onclick={() => insertSymbol(sym)} title={sym}>{sym}</button>
											{/each}
										</div>
									</div>
									<div class="cm-script-section">
										<span class="cm-script-section-label">{$t('scriptToolbar.punctuation')}</span>
										<div class="cm-script-chars">
											{#each data.punctuation as sym}
												<button class="cm-script-char" onclick={() => insertSymbol(sym)} title={sym}>{sym}</button>
											{/each}
										</div>
									</div>
									{#if data.numerals && data.numerals.length > 0}
										<div class="cm-script-section">
											<span class="cm-script-section-label">{$t('scriptToolbar.numerals')}</span>
											<div class="cm-script-chars">
												{#each data.numerals as sym}
													<button class="cm-script-char" onclick={() => insertSymbol(sym)} title={sym}>{sym}</button>
												{/each}
											</div>
										</div>
									{/if}
								</div>
							{/if}
						</div>
					{/if}
				{/each}
			{/if}
		</div>
	{/if}

	<div class="cm-editor-area" bind:this={containerEl}></div>

	{#if toolbarVisible && view && $appSettings.showFloatingToolbar}
		<FormattingToolbar
			x={toolbarX}
			y={toolbarY}
			onBold={() => { wrapSelection(view!, '**', '**') }}
			onItalic={() => { wrapSelection(view!, '_', '_') }}
			onStrikethrough={() => { wrapSelection(view!, '~~', '~~') }}
			onHighlight={() => { wrapSelection(view!, '==', '==') }}
			onCode={() => { wrapSelection(view!, '`', '`') }}
			onLink={() => { wrapSelection(view!, '[[', ']]') }}
			onHeading={applyHeading}
			showConvertToTable={selectionIsTabular}
			onConvertToTable={convertSelectionToTable}
		/>
	{/if}
	{#if tableToolbarVisible && view && currentTable}
		<TableToolbar
			x={tableToolbarX}
			y={tableToolbarY}
			onAddRow={() => { if (currentTable) applyTableChange(addRow(currentTable, currentTable.cursorRow)); }}
			onAddColumn={() => { if (currentTable) applyTableChange(addColumn(currentTable, currentTable.cursorCol)); }}
			onDeleteRow={() => { if (currentTable) applyTableChange(deleteRow(currentTable, currentTable.cursorRow)); }}
			onDeleteColumn={() => { if (currentTable) applyTableChange(deleteColumn(currentTable, currentTable.cursorCol)); }}
			onAlignLeft={() => { if (currentTable) applyTableChange(setAlignment(currentTable, currentTable.cursorCol, 'left')); }}
			onAlignCenter={() => { if (currentTable) applyTableChange(setAlignment(currentTable, currentTable.cursorCol, 'center')); }}
			onAlignRight={() => { if (currentTable) applyTableChange(setAlignment(currentTable, currentTable.cursorCol, 'right')); }}
			onMoveRowUp={() => { if (currentTable) applyTableChange(moveRow(currentTable, currentTable.cursorRow, 'up')); }}
			onMoveRowDown={() => { if (currentTable) applyTableChange(moveRow(currentTable, currentTable.cursorRow, 'down')); }}
			onMoveColLeft={() => { if (currentTable) applyTableChange(moveColumn(currentTable, currentTable.cursorCol, 'left')); }}
			onMoveColRight={() => { if (currentTable) applyTableChange(moveColumn(currentTable, currentTable.cursorCol, 'right')); }}
			onSortAsc={() => { if (currentTable) applyTableChange(sortByColumn(currentTable, currentTable.cursorCol, 'asc')); }}
			onSortDesc={() => { if (currentTable) applyTableChange(sortByColumn(currentTable, currentTable.cursorCol, 'desc')); }}
			onInsertFormula={() => { if (currentTable && view) insertFormulaAtCursor(view, currentTable); }}
			onEvaluateFormulas={() => { if (currentTable) applyTableChange({ ...currentTable, rows: evaluateTableFormulas(currentTable.rows) }); }}
		/>
	{/if}
	{#if gridPickerVisible}
		<TableGridPicker
			x={gridPickerX}
			y={gridPickerY}
			onInsert={insertTableFromGrid}
			onClose={() => gridPickerVisible = false}
		/>
	{/if}
	{#if contextMenuVisible && view}
		<EditorContextMenu
			x={contextMenuX}
			y={contextMenuY}
			hasSelection={contextMenuHasSelection}
			cursorContext={contextMenuCursorContext}
			currentHeadingLevel={contextMenuHeadingLevel}
			onFormat={handleContextFormat}
			onInsert={handleContextInsert}
			onHeading={handleContextHeading}
			onList={handleContextList}
			onClipboard={handleContextClipboard}
			onTableAction={handleContextTableAction}
			onLinkAction={handleContextLinkAction}
			onClose={() => contextMenuVisible = false}
		/>
	{/if}
	<!-- Font picker dropdown -->
	{#if showFontPicker && view}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="cm-dropdown-overlay" onclick={() => showFontPicker = false}>
			<div class="cm-dropdown-menu cm-font-menu" onclick={(e) => e.stopPropagation()}>
				{#each fontFamilies as font}
					<button
						class="cm-dropdown-item"
						style={font.value ? `font-family: ${font.value}` : ''}
						onclick={() => { showFontPicker = false; if (view) applyFontFamily(view, font.value); view?.focus(); }}
					>
						{font.label}
					</button>
				{/each}
			</div>
		</div>
	{/if}
	<!-- Color picker dropdown -->
	{#if showColorPicker && view}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="cm-dropdown-overlay" onclick={() => showColorPicker = false}>
			<div class="cm-dropdown-menu cm-color-grid" onclick={(e) => e.stopPropagation()}>
				{#each colorPalette as c}
					<button class="cm-color-swatch" style="background:{c}" onclick={() => { showColorPicker = false; if (view) applyTextColor(view, c); view?.focus(); }} title={c}></button>
				{/each}
				<button class="cm-dropdown-item" onclick={() => { showColorPicker = false; if (view) removeTextColor(view); view?.focus(); }}>Remove color</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.cm-wrapper {
		flex: 1;
		min-height: 0;
		overflow: hidden;
		position: relative;
		display: flex;
		flex-direction: column;
	}
	.cm-editor-area {
		flex: 1;
		min-height: 0;
		overflow: hidden;
		display: flex;
		margin-top: 40px;
		flex-direction: column;
	}
	/* Persistent toolbar */
	.cm-toolbar {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 4px 8px;
		border-bottom: none;
		background: var(--background-primary);
		flex-wrap: wrap;
		flex-shrink: 0;
		font-family: var(--font-interface-theme);
	}
	.cm-tb {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		background: none;
		border-radius: 4px;
		cursor: pointer;
		color: var(--text-muted);
		font-size: 12px;
		font-weight: 600;
		font-family: var(--font-interface-theme);
		padding: 0;
	}
	.cm-tb-active {
		background: var(--interactive-accent) !important;
		color: white !important;
		border-radius: 4px;
	}
	.cm-tb:hover {
		background: var(--background-modifier-hover);
		color: var(--text-normal);
	}
	.cm-tb-text {
		width: auto;
		padding: 0 6px;
		gap: 4px;
	}
	/* Flip directional icons in RTL toolbar */
	.cm-toolbar[dir="rtl"] .cm-icon-dir { transform: scaleX(-1); }

	.cm-tb-sep {
		width: 1px;
		height: 18px;
		background: var(--background-modifier-border);
		margin: 0 3px;
		flex-shrink: 0;
	}
	.cm-wrapper :global(.cm-editor) {
		flex: 1;
		min-height: 0;
		outline: none;
	}
	.cm-wrapper :global(.cm-scroller) {
		overflow: auto;
	}
	/* Force selection highlight visibility */
	.cm-wrapper :global(.cm-editor.cm-focused .cm-selectionBackground) {
		background-color: rgba(255, 170, 0, 0.4) !important;
	}
	.cm-wrapper :global(.cm-selectionBackground) {
		background-color: rgba(255, 170, 0, 0.3) !important;
	}
	.cm-wrapper :global(.cm-editor .cm-line ::selection) {
		background-color: rgba(255, 170, 0, 0.4) !important;
	}
	.cm-wrapper :global(.cm-editor .cm-line::selection) {
		background-color: rgba(255, 170, 0, 0.4) !important;
	}
	.cm-wrapper :global(.cm-content ::selection) {
		background-color: rgba(255, 170, 0, 0.4) !important;
	}
	/* Indentation guides — vertical lines for nested content */
	.cm-wrapper :global(.cm-indent-guide) {
		position: relative;
	}
	.cm-wrapper :global(.cm-indent-guide::before) {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		border-left: 1px solid var(--background-modifier-border, #e0e0e0);
		opacity: 0.6;
	}
	.cm-wrapper :global(.cm-indent-1::before) { left: calc(1 * 2em); }
	.cm-wrapper :global(.cm-indent-2::before) { left: calc(2 * 2em); }
	.cm-wrapper :global(.cm-indent-3::before) { left: calc(3 * 2em); }
	.cm-wrapper :global(.cm-indent-4::before) { left: calc(4 * 2em); }
	.cm-wrapper :global(.cm-indent-5::before) { left: calc(5 * 2em); }
	.cm-wrapper :global(.cm-indent-6::before) { left: calc(6 * 2em); }
	.cm-wrapper :global(.cm-indent-7::before) { left: calc(7 * 2em); }
	.cm-wrapper :global(.cm-indent-8::before) { left: calc(8 * 2em); }
	.cm-wrapper :global(.cm-indent-9::before) { left: calc(9 * 2em); }
	.cm-wrapper :global(.cm-indent-10::before) { left: calc(10 * 2em); }
	/* Font/Color picker dropdowns */
	.cm-dropdown-overlay {
		position: fixed; inset: 0; z-index: 1000;
	}
	.cm-dropdown-menu {
		position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 8px; box-shadow: var(--dropdown-shadow, 0 8px 24px rgba(0,0,0,0.15));
		padding: 4px; min-width: 180px; max-height: 320px; overflow-y: auto;
		font-family: var(--font-interface-theme);
	}
	.cm-font-menu { min-width: 220px; }
	.cm-dropdown-item {
		display: block; width: 100%; padding: 6px 12px; border: none; background: none;
		color: var(--text-normal); cursor: pointer; text-align: start; border-radius: 4px;
		font-size: 13px; font-family: var(--font-interface-theme);
	}
	.cm-dropdown-item:hover { background: var(--background-modifier-hover); }
	.cm-color-grid {
		display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px;
		padding: 8px; min-width: 180px;
	}
	.cm-color-swatch {
		width: 28px; height: 28px; border-radius: 4px; border: 2px solid transparent;
		cursor: pointer; transition: border-color 0.15s;
	}
	.cm-color-swatch:hover { border-color: var(--text-normal); }

	/* Script toolbar */
	.cm-script-toolbar {
		display: flex; align-items: center; gap: 4px; padding: 2px 8px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		flex-wrap: wrap;
	}
	.cm-script-group { position: relative; }
	.cm-script-label {
		display: flex; align-items: center; gap: 3px;
		padding: 3px 8px; border-radius: 4px; border: none;
		background: var(--background-modifier-hover); color: var(--text-normal);
		font-size: 12px; font-weight: 600; cursor: pointer;
		font-family: var(--font-interface-theme);
	}
	.cm-script-label:hover { background: var(--interactive-accent); color: white; }
	.cm-script-label svg { transition: transform 0.15s; }
	.cm-script-label svg.rotated { transform: rotate(180deg); }
	.cm-script-panel {
		position: absolute; top: 100%; left: 0; z-index: 100;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 8px; box-shadow: var(--dropdown-shadow, 0 4px 16px rgba(0,0,0,0.15));
		padding: 8px; min-width: 280px; max-width: 400px;
	}
	.cm-script-section { margin-bottom: 6px; }
	.cm-script-section:last-child { margin-bottom: 0; }
	.cm-script-section-label {
		display: block; font-size: 10px; font-weight: 600; text-transform: uppercase;
		color: var(--text-muted); letter-spacing: 0.5px; margin-bottom: 4px;
		font-family: var(--font-interface-theme);
	}
	.cm-script-chars { display: flex; flex-wrap: wrap; gap: 2px; }
	.cm-script-char {
		display: flex; align-items: center; justify-content: center;
		width: 32px; height: 32px; border-radius: 4px; border: 1px solid var(--background-modifier-border);
		background: var(--background-primary); color: var(--text-normal);
		font-size: 16px; cursor: pointer; transition: all 0.1s;
	}
	.cm-script-char:hover {
		background: var(--interactive-accent); color: white;
		border-color: var(--interactive-accent);
	}
</style>

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorView, ViewPlugin, Decoration, keymap, placeholder as cmPlaceholder, drawSelection, dropCursor, highlightActiveLine, highlightActiveLineGutter, lineNumbers, highlightSpecialChars, rectangularSelection, type ViewUpdate, type DecorationSet } from '@codemirror/view';
	import { EditorState, Compartment, type Range } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { languages } from '@codemirror/language-data';
	import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
	import { searchKeymap, highlightSelectionMatches, openSearchPanel, selectNextOccurrence } from '@codemirror/search';
	import { closeBrackets, closeBracketsKeymap, autocompletion, type CompletionContext, type Completion } from '@codemirror/autocomplete';
	import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput, foldGutter, foldKeymap, foldService, foldAll, unfoldAll, indentUnit } from '@codemirror/language';
	import type { FileEntry } from '$lib/vaults/store';
	import { saveClipboardImage, resolveWikilinkCrossVault, getNoteHeadings } from '$lib/vaults/store';
	import FormattingToolbar from './FormattingToolbar.svelte';
	import TableToolbar from './TableToolbar.svelte';
	import { parseTable, formatTable, addRow, addColumn, deleteRow, deleteColumn, setAlignment, type ParsedTable } from '$lib/editor/tableUtils';
	import { livePreviewPlugin, livePreviewTheme } from '$lib/editor/livePreview';

	let {
		value = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		placeholder = '',
		onchange,
		noteNames = [] as { name: string; path: string; vaultName?: string }[],
		allTags = [] as string[],
		vaultPath = '',
		livePreview = false,
		showLineNumbers = true,
		foldHeading = true,
		foldIndent = true,
		indentationGuides = false,
		indentWithTabs = true,
		tabSize = 4,
		autoPairMarkdown = true,
	}: {
		value: string;
		dir?: 'ltr' | 'rtl';
		placeholder?: string;
		onchange: (value: string) => void;
		noteNames?: { name: string; path: string; vaultName?: string }[];
		allTags?: string[];
		vaultPath?: string;
		livePreview?: boolean;
		showLineNumbers?: boolean;
		foldHeading?: boolean;
		foldIndent?: boolean;
		indentationGuides?: boolean;
		indentWithTabs?: boolean;
		tabSize?: number;
		autoPairMarkdown?: boolean;
	} = $props();

	let containerEl: HTMLDivElement;
	let view: EditorView | undefined;
	let dirCompartment = new Compartment();
	let livePreviewCompartment = new Compartment();
	let lineNumbersCompartment = new Compartment();
	let foldGutterCompartment = new Compartment();
	let indentGuidesCompartment = new Compartment();
	let tabConfigCompartment = new Compartment();
	let updating = false;

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
	const LINK_TYPES = ['related-to', 'prerequisite', 'see-also', 'contradicts', 'supports', 'extends'];

	function wikilinkCompletion(context: CompletionContext): any {
		const before = context.matchBefore(/\[\[[^\]]*$/);
		if (!before) return null;
		const inner = before.text.slice(2);

		// Link type autocomplete: [[note|type:query
		const pipeIdx = inner.indexOf('|');
		if (pipeIdx >= 0) {
			const afterPipe = inner.slice(pipeIdx + 1);
			if (afterPipe.toLowerCase().startsWith('type:')) {
				const noteName = inner.slice(0, pipeIdx);
				const typeQuery = afterPipe.slice(5).toLowerCase();
				const options: Completion[] = LINK_TYPES
					.filter(t => t.includes(typeQuery))
					.map(t => ({
						label: t,
						type: 'keyword',
						apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
							const insert = `[[${noteName}|type:${t}]]`;
							view.dispatch({
								changes: { from: before.from, to, insert },
								selection: { anchor: before.from + insert.length }
							});
						}
					}));
				return { from: before.from, options, filter: false };
			}
		}

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
				detail: n.vaultName ? ` — ${n.vaultName}` : undefined,
				type: 'text',
				apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
					view.dispatch({
						changes: { from: before.from, to, insert: `[[${n.name}]]` },
						selection: { anchor: before.from + n.name.length + 4 }
					});
				}
			}));
		return { from: before.from, options, filter: false };
	}

	// Heading autocomplete after [[note#
	async function headingCompletion(context: CompletionContext, before: any, noteName: string, headingQuery: string) {
		// Resolve the note to get its path
		const resolved = await resolveWikilinkCrossVault(vaultPath, noteName);
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

	// Tag autocomplete
	function tagCompletion(context: CompletionContext) {
		const before = context.matchBefore(/#[\w\u0600-\u06FF/\-]*$/);
		if (!before) return null;
		const query = before.text.slice(1).toLowerCase();
		const options: Completion[] = allTags
			.filter(t => t.toLowerCase().includes(query))
			.slice(0, 20)
			.map(t => ({
				label: '#' + t,
				type: 'keyword',
				apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
					view.dispatch({
						changes: { from: before.from, to, insert: '#' + t },
						selection: { anchor: before.from + t.length + 1 }
					});
				}
			}));
		return { from: before.from, options, filter: false };
	}

	// Slash commands
	function slashCompletion(context: CompletionContext) {
		const line = context.state.doc.lineAt(context.pos);
		const lineStart = line.text.trimStart();
		if (!lineStart.startsWith('/')) return null;
		const before = context.matchBefore(/\/\w*$/);
		if (!before) return null;
		const commands: Completion[] = [
			{ label: '/heading1', detail: 'H1', apply: '# ' },
			{ label: '/heading2', detail: 'H2', apply: '## ' },
			{ label: '/heading3', detail: 'H3', apply: '### ' },
			{ label: '/bullet', detail: 'Bullet list', apply: '- ' },
			{ label: '/numbered', detail: 'Numbered list', apply: '1. ' },
			{ label: '/task', detail: 'Task', apply: '- [ ] ' },
			{ label: '/code', detail: 'Code block', apply: '```\n\n```' },
			{ label: '/quote', detail: 'Blockquote', apply: '> ' },
			{ label: '/divider', detail: 'Horizontal rule', apply: '---\n' },
			{ label: '/table', detail: 'Table', apply: '| Column 1 | Column 2 |\n| --- | --- |\n| | |\n' },
			{ label: '/callout', detail: 'Callout', apply: '> [!note] Title\n> Content\n' },
			{ label: '/math', detail: 'Math block', apply: '$$\n\n$$' },
			{ label: '/mermaid', detail: 'Mermaid diagram', apply: '```mermaid\ngraph TD\n  A --> B\n```\n' },
			{ label: '/template', detail: 'Insert template', apply: '' },
		];
		return {
			from: before.from,
			options: commands.map(c => ({
				...c,
				apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
					if (c.label === '/template') {
						// Clear the slash command text and trigger template picker
						view.dispatch({ changes: { from: line.from, to } });
						window.dispatchEvent(new CustomEvent('constellation:open-template-picker'));
						return;
					}
					// Replace the slash command (and any leading whitespace on the line) with the content
					view.dispatch({
						changes: { from: line.from, to, insert: c.apply as string }
					});
				}
			})),
			filter: true
		};
	}

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

	// Toggle Obsidian-style comment %%...%%
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

	// Obsidian-like theme
	const obsidianTheme = EditorView.theme({
		'&': {
			fontSize: '0.92rem',
		},
		'.cm-content': {
			fontFamily: 'var(--vault-mono-font, var(--font-monospace-theme))',
			lineHeight: '1.7',
			padding: '0',
			caretColor: 'var(--vault-accent, var(--interactive-accent))',
		},
		'.cm-cursor': {
			borderLeftColor: 'var(--vault-accent, var(--interactive-accent))',
		},
		'.cm-gutters': {
			backgroundColor: 'transparent',
			borderRight: '1px solid var(--background-modifier-border-focus)',
			color: 'var(--color-base-40)',
			fontSize: '0.75rem',
		},
		'.cm-activeLineGutter': {
			backgroundColor: 'transparent',
			color: 'var(--text-faint)',
		},
		'.cm-activeLine': {
			backgroundColor: 'var(--background-primary-alt)',
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
				if (!items || !vaultPath) return false;

				for (const item of items) {
					if (item.type.startsWith('image/')) {
						event.preventDefault();
						const blob = item.getAsFile();
						if (!blob) return true;

						const reader = new FileReader();
						reader.onload = async () => {
							const base64 = reader.result as string;
							try {
								const filename = await saveClipboardImage(vaultPath, base64);
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
				lineNumbersCompartment.of(showLineNumbers ? [lineNumbers(), highlightActiveLineGutter()] : []),
				highlightSpecialChars(),
				history(),
				foldGutterCompartment.of((foldHeading || foldIndent) ? [foldGutter(), foldService.of(markdownFoldService)] : []),
				drawSelection(),
				rectangularSelection(),
				dropCursor(),
				indentOnInput(),
				bracketMatching(),
				highlightActiveLine(),
				syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
				markdown({ base: markdownLanguage, codeLanguages: languages }),
				autocompletion({
					override: [wikilinkCompletion, tagCompletion, slashCompletion],
					activateOnTyping: true,
					maxRenderedOptions: 20,
				}),
				editorKeymap,
				keymap.of([
					{ key: 'Tab', run: tableTab },
					{ key: 'Shift-Tab', run: tableShiftTab },
					indentWithTab,
					...defaultKeymap,
					...historyKeymap,
					...searchKeymap,
					...foldKeymap,
					...closeBracketsKeymap,
				]),
				tabConfigCompartment.of([
					EditorState.tabSize.of(tabSize),
					indentUnit.of(indentWithTabs ? '\t' : ' '.repeat(tabSize)),
				]),
				indentGuidesCompartment.of(indentationGuides ? [indentGuidesPlugin] : []),
				dirCompartment.of(EditorView.editorAttributes.of({ dir })),
				livePreviewCompartment.of(livePreview ? [livePreviewPlugin, livePreviewTheme] : []),
				cmPlaceholder(placeholder),
				obsidianTheme,
				clipboardImagePaste(),
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !updating) {
						onchange(update.state.doc.toString());
					}
					if (update.selectionSet || update.docChanged) {
						updateToolbar(update.view);
						updateTableToolbar(update.view);
					}
				}),
			]
		});
		view = new EditorView({ state: startState, parent: containerEl });

		document.addEventListener('constellation:fold-all', handleFoldAll);
		document.addEventListener('constellation:unfold-all', handleUnfoldAll);
	});

	// Sync value prop → editor
	$effect(() => {
		if (view && value !== view.state.doc.toString()) {
			updating = true;
			view.dispatch({
				changes: { from: 0, to: view.state.doc.length, insert: value }
			});
			updating = false;
		}
	});

	// Sync dir prop → editor
	$effect(() => {
		if (view) {
			view.dispatch({
				effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir }))
			});
		}
	});

	// Sync livePreview prop → editor
	$effect(() => {
		if (view) {
			view.dispatch({
				effects: livePreviewCompartment.reconfigure(livePreview ? [livePreviewPlugin, livePreviewTheme] : [])
			});
		}
	});

	// Sync showLineNumbers prop → editor
	$effect(() => {
		if (view) {
			view.dispatch({
				effects: lineNumbersCompartment.reconfigure(showLineNumbers ? [lineNumbers(), highlightActiveLineGutter()] : [])
			});
		}
	});

	// Sync fold settings → editor
	$effect(() => {
		if (view) {
			view.dispatch({
				effects: foldGutterCompartment.reconfigure((foldHeading || foldIndent) ? [foldGutter(), foldService.of(markdownFoldService)] : [])
			});
		}
	});

	// Sync indentation guides → editor
	$effect(() => {
		if (view) {
			view.dispatch({
				effects: indentGuidesCompartment.reconfigure(indentationGuides ? [indentGuidesPlugin] : [])
			});
		}
	});

	// Sync tab config → editor
	$effect(() => {
		if (view) {
			view.dispatch({
				effects: tabConfigCompartment.reconfigure([
					EditorState.tabSize.of(tabSize),
					indentUnit.of(indentWithTabs ? '\t' : ' '.repeat(tabSize)),
				])
			});
		}
	});

	// Listen for fold-all / unfold-all events from command palette
	function handleFoldAll() { if (view) foldAll(view); }
	function handleUnfoldAll() { if (view) unfoldAll(view); }

	onDestroy(() => {
		if (toolbarTimeout) clearTimeout(toolbarTimeout);
		document.removeEventListener('constellation:fold-all', handleFoldAll);
		document.removeEventListener('constellation:unfold-all', handleUnfoldAll);
		view?.destroy();
	});

	export function focus() {
		view?.focus();
	}

	export function getView(): EditorView | undefined {
		return view;
	}
</script>

<div class="cm-wrapper" bind:this={containerEl}>
	{#if toolbarVisible && view}
		<FormattingToolbar
			x={toolbarX}
			y={toolbarY}
			onBold={() => { wrapSelection(view!, '**', '**'); view!.focus(); }}
			onItalic={() => { wrapSelection(view!, '_', '_'); view!.focus(); }}
			onStrikethrough={() => { wrapSelection(view!, '~~', '~~'); view!.focus(); }}
			onHighlight={() => { wrapSelection(view!, '==', '=='); view!.focus(); }}
			onCode={() => { wrapSelection(view!, '`', '`'); view!.focus(); }}
			onLink={() => { wrapSelection(view!, '[[', ']]'); view!.focus(); }}
			onHeading={applyHeading}
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
		/>
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
</style>

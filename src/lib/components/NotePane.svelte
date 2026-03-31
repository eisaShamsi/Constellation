<script lang="ts">
	/**
	 * NotePane — The core note editor.
	 * Gray desk + breadcrumb + white paper + title + properties + CM6 editor + persistence.
	 * Live preview decorations via shared livePreview plugin. Typing must be instant.
	 * Originally developed as eNotePane (experimental), promoted to production 2026-03-29.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import type { FrontmatterProperty } from '$lib/libraries/store';
	import PropertyEditor from './PropertyEditor.svelte';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
	import { tags } from '@lezer/highlight';
	import { defaultKeymap, history, historyKeymap, undo, redo, indentWithTab } from '@codemirror/commands';
	import { autocompletion, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
	import { livePreviewPlugin, livePreviewTheme, libraryPathField, setLibraryPath } from '$lib/editor/livePreview';
	import { calloutPlugin, calloutTheme, calloutCollapseField, toggleCallout } from '$lib/editor/calloutPlugin';
	import { lineDecoPlugin, lineDecoTheme } from '$lib/editor/lineDecoPlugin';
	import { Highlight as HighlightExt } from '$lib/editor/markdownHighlight';
	import { createWikilinkCompletion, createTagCompletion, createSlashCompletion, createTypedLinkCompletion } from '$lib/editor/completions';
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
		onsave?: (value: string) => void;
		onflush?: (text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number) => void;
		ontitlechange?: (newTitle: string) => void;
		onnavigateback?: () => void;
		onnavigateforward?: () => void;
		onmoreaction?: (action: string) => void;
		onpropschange?: () => void;
	} = $props();

	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	let latestText = value;
	let dirty = false;
	let idleSaveTimer: ReturnType<typeof setInterval> | null = null;
	let rafHandle: number | null = null;
	let checkboxHandler: EventListener | null = null;
	let chevronHandler: EventListener | null = null;
	const dirCompartment = new Compartment();
	const livePreviewCompartment = new Compartment();
	let livePreviewEnabled = $state(true);

	/* ─── Table toolbar state ─── */
	let tableToolbarX = $state(0);
	let tableToolbarY = $state(0);
	let currentTable = $state<ParsedTable | null>(null);
	let tableToolbarVisible = $derived(currentTable !== null);

	/* ─── Phase 3 state ─── */
	let propsCollapsed = $state(false);
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

	/* ─── Background save ─── */
	function doSave() {
		if (!dirty) return;
		dirty = false;
		// Snapshot text here — the one place we pay the O(N) toString() cost,
		// at most once per autosave cycle (1.5s), never on individual keystrokes.
		if (view) latestText = view.state.doc.toString();
		onsave?.(latestText);
	}
	function doFlush() {
		if (view) latestText = view.state.doc.toString();
		const cursorPos = view ? view.state.selection.main.head : 0;
		const scrollTop = view ? view.scrollDOM.scrollTop : 0;
		onflush?.(latestText, dirty, cursorPos, scrollTop);
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
				libraryPathField, /* image path resolution */
				closeBrackets(),
				autocompletion({ override: [typedLinkCompletion, wikilinkCompletion, tagCompletion, slashCompletion], activateOnTyping: true, maxRenderedOptions: 20 }),
				keymap.of([
					{ key: 'Tab', run: tableTab },
					{ key: 'Shift-Tab', run: tableShiftTab },
					indentWithTab,
					...defaultKeymap, ...historyKeymap, ...closeBracketsKeymap,
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
					}
					if (update.selectionSet && !update.docChanged) {
						updateTableToolbar(update.view);
					}
				}),
				EditorView.theme({
					'&': { background: 'transparent', border: 'none', outline: 'none' },
					'&.cm-focused': { outline: 'none' },
					'.cm-scroller': { overflow: 'auto', fontFamily: 'inherit', fontSize: '16px', lineHeight: '1.75' },
					'.cm-content': { padding: '0', caretColor: 'var(--text-normal, #1a1a1a)' },
					'.cm-cursor': { borderLeftColor: 'var(--text-normal, #1a1a1a)', borderLeftWidth: '1.5px' },
					'.cm-line': { padding: '0' },
					'.cm-activeLine': { background: 'transparent' },
					'.cm-activeLineGutter': { display: 'none' },
					'.cm-gutters': { display: 'none' },
					'.cm-selectionBackground': { background: 'color-mix(in srgb, var(--interactive-accent, #7c3aed) 20%, transparent)' },
				}),
			],
		});

		view = new EditorView({ state, parent: editorEl! });

		/* Set library path for image resolution */
		if (libraryPath) {
			view.dispatch({ effects: setLibraryPath.of(libraryPath) });
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
			if (!isNaN(lineNum)) {
				view.dispatch({ effects: toggleCallout.of(lineNum) });
			}
		}) as EventListener;
		editorEl!.addEventListener('mousedown', chevronHandler, true);

		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
			view.focus();
		} else {
			titleEl?.focus();
		}
		if (initialScrollTop > 0) {
			rafHandle = requestAnimationFrame(() => {
				rafHandle = requestAnimationFrame(() => {
					rafHandle = null;
					view?.scrollDOM.scrollTo({ top: initialScrollTop });
				});
			});
		}

		idleSaveTimer = setInterval(() => { requestIdleCallback(() => doSave()); }, IDLE_SAVE_INTERVAL);
		document.addEventListener('visibilitychange', handleVisibilityChange);
		window.addEventListener('beforeunload', handleBeforeUnload);
	});

	/* ─── Destroy ─── */
	onDestroy(() => {
		if (idleSaveTimer) clearInterval(idleSaveTimer);
		document.removeEventListener('visibilitychange', handleVisibilityChange);
		window.removeEventListener('beforeunload', handleBeforeUnload);
		if (checkboxHandler && editorEl) editorEl.removeEventListener('mousedown', checkboxHandler, true);
		if (chevronHandler && editorEl) editorEl.removeEventListener('mousedown', chevronHandler, true);
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
	function generateAutoTitle(): string {
		const now = new Date();
		const dd = String(now.getDate()).padStart(2, '0');
		const mm = String(now.getMonth() + 1).padStart(2, '0');
		const yyyy = now.getFullYear();
		const hh = String(now.getHours()).padStart(2, '0');
		const min = String(now.getMinutes()).padStart(2, '0');
		return `CoNote${dd}${mm}${yyyy}.${hh}:${min}`;
	}
	function handleTitleBlur() {
		const trimmed = titleValue.trim();
		if (!trimmed) titleValue = generateAutoTitle();
		if (titleValue !== title) ontitlechange?.(titleValue);
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
				/>
			{/if}
			<hr class="e-props-divider" />
		{/if}

		<!-- ─── Toolbar (Phase 4) — dispatches CM6 commands, never modifies state directly ─── -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="e-toolbar" onmousedown={(e) => e.preventDefault()}>
			<button class="e-tb" title={$t('toolbar.bold')} onclick={() => wrapSelection('**', '**')}><strong>B</strong></button>
			<button class="e-tb" title={$t('toolbar.italic')} onclick={() => wrapSelection('_', '_')}><em>I</em></button>
			<button class="e-tb" title={$t('toolbar.strikethrough')} onclick={() => wrapSelection('~~', '~~')}><s>S</s></button>
			<button class="e-tb" title={$t('toolbar.highlight')} onclick={() => wrapSelection('==', '==')}><span class="e-tb-hl">H</span></button>
			<button class="e-tb mono" title={$t('toolbar.code')} onclick={() => wrapSelection('`', '`')}>&lt;/&gt;</button>
			<div class="e-tb-sep"></div>
			<div class="e-tb-drop"><button class="e-tb" onclick={() => toggleMenu('heading')}>H<span class="e-tb-caret">▾</span></button>
				{#if showHeadingMenu}<div class="e-tb-menu">{#each [1,2,3,4,5,6] as lv}<button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('#'.repeat(lv) + ' '); }}>H{lv}</button>{/each}</div>{/if}</div>
			<div class="e-tb-drop"><button class="e-tb" onclick={() => toggleMenu('list')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg><span class="e-tb-caret">▾</span></button>
				{#if showListMenu}<div class="e-tb-menu"><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('- '); }}>• Bullet</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('1. '); }}>1. Numbered</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertLinePrefix('- [ ] '); }}>☐ Task</button></div>{/if}</div>
			<div class="e-tb-sep"></div>
			<button class="e-tb" title={$t('toolbar.link')} onclick={() => wrapSelection('[[', ']]')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg></button>
			<div class="e-tb-drop"><button class="e-tb" onclick={() => toggleMenu('insert')}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/></svg><span class="e-tb-caret">▾</span></button>
				{#if showInsertMenu}<div class="e-tb-menu"><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n> '); }}>❝ Blockquote</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n```\n\n```\n'); }}>⌨ Code block</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n---\n'); }}>― Rule</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('\n| Col 1 | Col 2 |\n| --- | --- |\n| | |\n'); }}>{$t('toolbar.table')}</button><button class="e-tb-menu-item" onclick={() => { closeMenus(); insertAtCursor('![](url)'); }}>🖼 Image</button></div>{/if}</div>
			<div class="e-tb-sep"></div>
			<button class="e-tb" title="Undo" onclick={tbUndo}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6.69 3L3 13"/></svg></button>
			<button class="e-tb" title="Redo" onclick={tbRedo}><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6.69 3L21 13"/></svg></button>
		</div>

		{#if tableToolbarVisible && view && currentTable}
			<div class="e-table-toolbar-float" style="position: fixed; left: {tableToolbarX}px; top: {tableToolbarY}px; transform: translateX(-50%); z-index: 200;">
			<TableToolbar
				x={0}
				y={0}
				onAddRow={() => applyTableChange(addRow(currentTable!, currentTable!.cursorRow))}
				onAddColumn={() => applyTableChange(addColumn(currentTable!, currentTable!.cursorCol))}
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
		padding: 4px 16px; font-size: 0.78rem; color: var(--text-faint);
		display: flex; align-items: center; min-height: 28px; flex-shrink: 0;
		width: 100%; max-width: 1200px; background: #ffffff;
		border-bottom: 1px solid var(--background-modifier-border, #e0e0e0);
	}
	.e-bc-lib { color: var(--text-muted); }
	.e-bc-sep { margin: 0 4px; color: var(--background-modifier-border-focus); }
	.e-bc-note { color: var(--text-normal); }
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
	.e-editor :global(.cm-editor),
	.e-editor :global(.cm-editor.cm-focused) { outline: none !important; border: none !important; }

</style>

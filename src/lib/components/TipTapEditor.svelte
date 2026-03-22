<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import { Table } from '@tiptap/extension-table';
	import { TableRow } from '@tiptap/extension-table-row';
	import { TableCell } from '@tiptap/extension-table-cell';
	import { TableHeader } from '@tiptap/extension-table-header';
	import Image from '@tiptap/extension-image';
	import Link from '@tiptap/extension-link';
	import TextAlign from '@tiptap/extension-text-align';
	import Underline from '@tiptap/extension-underline';
	import Placeholder from '@tiptap/extension-placeholder';
	import TaskList from '@tiptap/extension-task-list';
	import TaskItem from '@tiptap/extension-task-item';
	import Highlight from '@tiptap/extension-highlight';
	import { TextStyle } from '@tiptap/extension-text-style';
	import Color from '@tiptap/extension-color';
	import Subscript from '@tiptap/extension-subscript';
	import Superscript from '@tiptap/extension-superscript';
	import CharacterCount from '@tiptap/extension-character-count';
	import FontFamily from '@tiptap/extension-font-family';
	import { marked } from 'marked';
	import TurndownService from 'turndown';
	import { t } from '$lib/i18n';
	import { appSettings, getAllFontSets } from '$lib/libraries/store';

	let {
		value = '',
		dir = 'ltr' as 'ltr' | 'rtl' | 'auto',
		placeholder = '',
		onchange,
	}: {
		value?: string;
		dir?: 'ltr' | 'rtl' | 'auto';
		placeholder?: string;
		onchange?: (markdown: string) => void;
	} = $props();

	let editorEl: HTMLDivElement;
	let editor: Editor | null = null;
	let turndown: TurndownService;
	let isUpdating = false;
	let lastExternalValue = value;
	let lastInternalMarkdown = value; // Track MD we produced ourselves
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	// Configure turndown for clean Markdown output
	function initTurndown(): TurndownService {
		const td = new TurndownService({
			headingStyle: 'atx',
			hr: '---',
			bulletListMarker: '-',
			codeBlockStyle: 'fenced',
			emDelimiter: '*',
		});

		// Table support — pass through thead/tbody transparently
		td.addRule('tableSection', {
			filter: ['thead', 'tbody', 'tfoot'],
			replacement(content) {
				return content;
			}
		});
		td.addRule('tableCell', {
			filter: ['th', 'td'],
			replacement(content) {
				return ` ${content.trim()} |`;
			}
		});
		td.addRule('tableRow', {
			filter: 'tr',
			replacement(content) {
				return `|${content}\n`;
			}
		});
		td.addRule('table', {
			filter: 'table',
			replacement(content) {
				const rows = content.trim().split('\n').filter(Boolean);
				if (rows.length === 0) return '';
				const firstRow = rows[0];
				const cols = (firstRow.match(/\|/g) || []).length - 1;
				const separator = '|' + ' --- |'.repeat(cols);
				return '\n' + rows[0] + '\n' + separator + '\n' + rows.slice(1).join('\n') + '\n';
			}
		});

		// Preserve styled spans (font-family, color) as HTML in Markdown
		td.addRule('styledSpan', {
			filter(node) {
				if (node.nodeName !== 'SPAN') return false;
				const style = (node as HTMLElement).getAttribute('style') || '';
				return style.includes('font-family') || style.includes('color');
			},
			replacement(content, node) {
				const el = node as HTMLElement;
				// Build clean style string, replacing inner double quotes to avoid HTML attribute conflicts
				const style = el.getAttribute('style')?.replace(/"/g, "'") || '';
				return `<span style="${style}">${content}</span>`;
			}
		});

		// Preserve sub/sup/mark/u as HTML in Markdown output
		td.addRule('subscript', {
			filter: 'sub',
			replacement(content) {
				return `<sub>${content}</sub>`;
			}
		});
		td.addRule('superscript', {
			filter: 'sup',
			replacement(content) {
				return `<sup>${content}</sup>`;
			}
		});
		td.addRule('highlight', {
			filter(node) {
				return node.nodeName === 'MARK';
			},
			replacement(content, node) {
				const el = node as HTMLElement;
				const dataColor = el.getAttribute('data-color');
				if (dataColor) {
					return `<mark data-color="${dataColor}">${content}</mark>`;
				}
				return `<mark>${content}</mark>`;
			}
		});
		td.addRule('underline', {
			filter: 'u',
			replacement(content) {
				return `<u>${content}</u>`;
			}
		});

		// Task list support
		td.addRule('taskListItem', {
			filter(node) {
				return node.nodeName === 'LI' && node.getAttribute('data-type') === 'taskItem';
			},
			replacement(content, node) {
				const checked = (node as HTMLElement).getAttribute('data-checked') === 'true';
				return `- [${checked ? 'x' : ' '}] ${content.trim()}\n`;
			}
		});

		return td;
	}

	function markdownToHtml(md: string): string {
		if (!md) return '<p></p>';
		// Auto-repair broken font-family spans from older saves where nested double quotes
		// broke the HTML. Handles all variations:
		//   <span style="font-family: "Name", serif">
		//   <span style="font-family:"Name", serif;">
		//   <span style="font-family: "Name", serif;">
		let cleaned = md;
		// Fix broken double-quote font-family in span style attributes
		cleaned = cleaned.replace(/<span\s+style="font-family:\s*"([^"]+)"([^"]*?);?"\s*>/g,
			(_, name, rest) => `<span style="font-family: '${name.trim()}'${rest.trim() ? rest.trim() : ''}">`
		);
		// Also handle already-escaped entities from previous broken saves
		cleaned = cleaned.replace(/&lt;span\s+style=&quot;font-family:\s*&quot;([^&]+)&quot;([^&]*?)&quot;\s*;?\s*&gt;/g,
			(_, name, rest) => `<span style="font-family: '${name.trim()}'${rest.trim() ? rest.trim() : ''}">`
		);
		cleaned = cleaned.replace(/&lt;\/span&gt;/g, '</span>');
		// Fix unclosed font-family spans — add </span> at end of line if missing
		cleaned = cleaned.replace(/(<span\s+style="font-family:[^"]*">[^<]*?)$/gm, '$1</span>');
		return marked.parse(cleaned, { async: false, gfm: true }) as string;
	}

	function htmlToMarkdown(html: string): string {
		if (!turndown) turndown = initTurndown();
		return turndown.turndown(html);
	}

	// Listen for table insertion from NotePane breadcrumb bar
	function handleInsertTableEvent(e: Event) {
		const detail = (e as CustomEvent).detail;
		if (editor && detail?.rows && detail?.cols) {
			editor.chain().focus().insertTable({
				rows: detail.rows,
				cols: detail.cols,
				withHeaderRow: true,
			}).run();
		}
	}

	onMount(() => {
		turndown = initTurndown();
		document.addEventListener('constellation:insert-table', handleInsertTableEvent);

		editor = new Editor({
			element: editorEl,
			extensions: [
				StarterKit.configure({
					heading: { levels: [1, 2, 3, 4, 5, 6] },
				}),
				Table.configure({ resizable: true }),
				TableRow,
				TableCell,
				TableHeader,
				Image,
				Link.configure({ openOnClick: false }),
				TextAlign.configure({ types: ['heading', 'paragraph'] }),
				Underline,
				Placeholder.configure({ placeholder }),
				TaskList,
				TaskItem.configure({ nested: true }),
				Highlight,
				TextStyle,
				Color,
				Subscript,
				Superscript,
				CharacterCount,
				FontFamily,
			],
			content: markdownToHtml(value),
			editorProps: {
				attributes: {
					class: 'tiptap-content',
					dir,
				},
			},
			onUpdate: ({ editor: ed }) => {
				if (isUpdating) return;
				updateContext();
				// Debounce the expensive HTML→Markdown conversion
				if (debounceTimer) clearTimeout(debounceTimer);
				debounceTimer = setTimeout(() => {
					if (!ed || ed.isDestroyed) return;
					const html = ed.getHTML();
					const md = htmlToMarkdown(html);
					lastInternalMarkdown = md; // Track so $effect skips setContent
					lastExternalValue = md;
					onchange?.(md);
				}, 300);
			},
			onSelectionUpdate: () => {
				updateContext();
			},
		});
	});

	// Update content ONLY when value changes externally (e.g. switching notes)
	// Skip if the change came from the editor's own onUpdate (tracked via lastInternalMarkdown)
	$effect(() => {
		if (editor && value !== lastExternalValue && value !== lastInternalMarkdown) {
			lastExternalValue = value;
			lastInternalMarkdown = value;
			isUpdating = true;
			const html = markdownToHtml(value);
			editor.commands.setContent(html, { emitUpdate: false });
			isUpdating = false;
		} else if (value !== lastExternalValue) {
			// Value changed but matches our internal markdown — just update tracking
			lastExternalValue = value;
		}
	});

	// Update direction
	$effect(() => {
		if (editor) {
			editor.setOptions({
				editorProps: {
					attributes: { class: 'tiptap-content', dir },
				},
			});
		}
	});

	onDestroy(() => {
		if (debounceTimer) clearTimeout(debounceTimer);
		document.removeEventListener('constellation:insert-table', handleInsertTableEvent);
		// Flush any pending markdown conversion before destroying
		if (editor && !editor.isDestroyed) {
			const html = editor.getHTML();
			const md = htmlToMarkdown(html);
			if (md !== lastInternalMarkdown) {
				onchange?.(md);
			}
		}
		editor?.destroy();
	});

	// Toolbar actions
	function isActive(name: string, attrs?: Record<string, any>): boolean {
		return editor?.isActive(name, attrs) ?? false;
	}

	function toggleBold() { editor?.chain().focus().toggleBold().run(); }
	function toggleItalic() { editor?.chain().focus().toggleItalic().run(); }
	function toggleUnderline() { editor?.chain().focus().toggleUnderline().run(); }
	function toggleStrike() { editor?.chain().focus().toggleStrike().run(); }
	function toggleHighlight() { editor?.chain().focus().toggleHighlight().run(); }
	function toggleCode() { editor?.chain().focus().toggleCode().run(); }
	function toggleCodeBlock() { editor?.chain().focus().toggleCodeBlock().run(); }
	function toggleBlockquote() { editor?.chain().focus().toggleBlockquote().run(); }
	function toggleBulletList() { editor?.chain().focus().toggleBulletList().run(); }
	function toggleOrderedList() { editor?.chain().focus().toggleOrderedList().run(); }
	function toggleTaskList() { editor?.chain().focus().toggleTaskList().run(); }
	function setHeading(level: 1|2|3|4|5|6) { editor?.chain().focus().toggleHeading({ level }).run(); }
	function setParagraph() { editor?.chain().focus().setParagraph().run(); }
	function addHorizontalRule() { editor?.chain().focus().setHorizontalRule().run(); }
	function undo() { editor?.chain().focus().undo().run(); }
	function redo() { editor?.chain().focus().redo().run(); }

	function setTextAlign(align: string) { editor?.chain().focus().setTextAlign(align).run(); }

	function insertTable() {
		editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
	}
	function addColumnBefore() { editor?.chain().focus().addColumnBefore().run(); }
	function addColumnAfter() { editor?.chain().focus().addColumnAfter().run(); }
	function deleteColumn() { editor?.chain().focus().deleteColumn().run(); }
	function addRowBefore() { editor?.chain().focus().addRowBefore().run(); }
	function addRowAfter() { editor?.chain().focus().addRowAfter().run(); }
	function deleteRow() { editor?.chain().focus().deleteRow().run(); }
	function deleteTable() { editor?.chain().focus().deleteTable().run(); }

	function toggleSubscript() { editor?.chain().focus().toggleSubscript().run(); }
	function toggleSuperscript() { editor?.chain().focus().toggleSuperscript().run(); }
	function clearFormatting() { editor?.chain().focus().clearNodes().unsetAllMarks().run(); }
	function indent() { editor?.chain().focus().sinkListItem('listItem').run(); }
	function outdent() { editor?.chain().focus().liftListItem('listItem').run(); }

	function insertImage() {
		const url = prompt('Image URL:');
		if (url) editor?.chain().focus().setImage({ src: url }).run();
	}

	function insertLink() {
		const url = prompt('Link URL:');
		if (url) editor?.chain().focus().setLink({ href: url }).run();
	}
	function removeLink() { editor?.chain().focus().unsetLink().run(); }

	// ─── Contextual toolbar state ───
	let contextType = $state<'table' | 'link' | 'image' | 'codeBlock' | 'callout' | null>(null);
	let contextLinkUrl = $state('');
	function updateContext() {
		if (!editor) { contextType = null; return; }
		if (editor.isActive('table')) {
			contextType = 'table';
		} else if (editor.isActive('link')) {
			contextType = 'link';
			contextLinkUrl = editor.getAttributes('link')?.href || '';
		} else if (editor.isActive('image')) {
			contextType = 'image';
		} else if (editor.isActive('codeBlock')) {
			contextType = 'codeBlock';
		} else if (editor.isActive('blockquote')) {
			// Only show callout context bar if the blockquote starts with [! (a callout marker)
			const { from } = editor.state.selection;
			const resolved = editor.state.doc.resolve(from);
			let isCallout = false;
			for (let depth = resolved.depth; depth > 0; depth--) {
				const node = resolved.node(depth);
				if (node.type.name === 'blockquote') {
					const text = node.textContent;
					if (text.match(/^\[!\w+\]/)) {
						isCallout = true;
					}
					break;
				}
			}
			contextType = isCallout ? 'callout' : null;
		} else {
			contextType = null;
		}
	}

	function editLinkUrl() {
		const url = prompt('Edit link URL:', contextLinkUrl);
		if (url !== null) {
			editor?.chain().focus().extendMarkRange('link').setLink({ href: url }).run();
			contextLinkUrl = url;
		}
	}

	function openLinkUrl() {
		if (contextLinkUrl) window.open(contextLinkUrl, '_blank');
	}

	function toggleHeaderRow() { editor?.chain().focus().toggleHeaderRow().run(); }
	function toggleHeaderColumn() { editor?.chain().focus().toggleHeaderColumn().run(); }
	function mergeCells() { editor?.chain().focus().mergeCells().run(); }
	function splitCell() { editor?.chain().focus().splitCell().run(); }

	function setCodeBlockLanguage(lang: string) {
		editor?.chain().focus().updateAttributes('codeBlock', { language: lang }).run();
	}
	const codeLanguages = ['', 'javascript', 'typescript', 'python', 'rust', 'html', 'css', 'json', 'yaml', 'markdown', 'bash', 'sql', 'java', 'c', 'cpp', 'go', 'ruby', 'php', 'swift', 'kotlin'];

	function changeCalloutType(type: string) {
		if (!editor) return;
		const { state } = editor;
		const { from } = state.selection;
		const resolved = state.doc.resolve(from);

		// Walk up to find the blockquote node
		for (let depth = resolved.depth; depth > 0; depth--) {
			const node = resolved.node(depth);
			if (node.type.name === 'blockquote') {
				// Find the [!marker] text in the first text node of the blockquote
				const blockStart = resolved.start(depth);
				let found = false;
				node.descendants((child, childPos) => {
					if (found || !child.isText) return;
					const text = child.text || '';
					const match = text.match(/^\[!\w+\]/);
					if (match) {
						const markerFrom = blockStart + childPos;
						const markerTo = markerFrom + match[0].length;
						const newMarker = `[!${type}]`;
						const { tr } = state;
						tr.replaceWith(markerFrom, markerTo, state.schema.text(newMarker));
						editor!.view.dispatch(tr);
						found = true;
					}
				});
				break;
			}
		}
	}

	// Callout insertion
	let showCalloutMenu = $state(false);
	const calloutTypes = ['note', 'tip', 'warning', 'danger', 'info', 'success', 'question', 'quote', 'example', 'bug'];
	function insertCallout(type: string) {
		showCalloutMenu = false;
		editor?.chain().focus().toggleBlockquote().run();
		// Insert the callout marker text
		setTimeout(() => {
			editor?.commands.insertContent(`[!${type}] `);
		}, 10);
	}

	// ─── Context menu ───
	let showContextMenu = $state(false);
	let contextMenuX = $state(0);
	let contextMenuY = $state(0);
	let contextInTable = $state(false);
	let contextOnLink = $state(false);
	let contextHasSelection = $state(false);

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault();
		if (!editor) return;

		contextMenuX = e.clientX;
		contextMenuY = e.clientY;
		contextInTable = editor.isActive('table');
		contextOnLink = editor.isActive('link');
		contextHasSelection = !editor.state.selection.empty;
		showContextMenu = true;

		const close = () => { showContextMenu = false; window.removeEventListener('click', close); };
		setTimeout(() => window.addEventListener('click', close), 0);
	}

	function ctxBold() { editor?.chain().focus().toggleBold().run(); showContextMenu = false; }
	function ctxItalic() { editor?.chain().focus().toggleItalic().run(); showContextMenu = false; }
	function ctxStrikethrough() { editor?.chain().focus().toggleStrike().run(); showContextMenu = false; }
	function ctxUnderline() { editor?.chain().focus().toggleUnderline().run(); showContextMenu = false; }
	function ctxHighlight() { editor?.chain().focus().toggleHighlight().run(); showContextMenu = false; }
	function ctxInlineCode() { editor?.chain().focus().toggleCode().run(); showContextMenu = false; }
	function ctxCodeBlock() { editor?.chain().focus().toggleCodeBlock().run(); showContextMenu = false; }
	function ctxBlockquote() { editor?.chain().focus().toggleBlockquote().run(); showContextMenu = false; }
	function ctxBulletList() { editor?.chain().focus().toggleBulletList().run(); showContextMenu = false; }
	function ctxOrderedList() { editor?.chain().focus().toggleOrderedList().run(); showContextMenu = false; }
	function ctxTaskList() { editor?.chain().focus().toggleTaskList().run(); showContextMenu = false; }
	function ctxHorizontalRule() { editor?.chain().focus().setHorizontalRule().run(); showContextMenu = false; }
	function ctxClearFormatting() { editor?.chain().focus().clearNodes().unsetAllMarks().run(); showContextMenu = false; }
	function ctxCut() { document.execCommand('cut'); showContextMenu = false; }
	function ctxCopy() { document.execCommand('copy'); showContextMenu = false; }
	function ctxPaste() { navigator.clipboard.readText().then(text => { editor?.chain().focus().insertContent(text).run(); }); showContextMenu = false; }
	function ctxSelectAll() { editor?.chain().focus().selectAll().run(); showContextMenu = false; }

	function ctxInsertLink() {
		showContextMenu = false;
		const url = prompt('URL:');
		if (url) editor?.chain().focus().setLink({ href: url }).run();
	}
	function ctxEditLink() {
		showContextMenu = false;
		const current = editor?.getAttributes('link')?.href || '';
		const url = prompt('URL:', current);
		if (url) editor?.chain().focus().extendMarkRange('link').setLink({ href: url }).run();
	}
	function ctxRemoveLink() { editor?.chain().focus().unsetLink().run(); showContextMenu = false; }
	function ctxOpenLink() {
		const href = editor?.getAttributes('link')?.href;
		if (href) window.open(href, '_blank');
		showContextMenu = false;
	}

	function ctxAddRowBefore() { editor?.chain().focus().addRowBefore().run(); showContextMenu = false; }
	function ctxAddRowAfter() { editor?.chain().focus().addRowAfter().run(); showContextMenu = false; }
	function ctxAddColBefore() { editor?.chain().focus().addColumnBefore().run(); showContextMenu = false; }
	function ctxAddColAfter() { editor?.chain().focus().addColumnAfter().run(); showContextMenu = false; }
	function ctxDeleteRow() { editor?.chain().focus().deleteRow().run(); showContextMenu = false; }
	function ctxDeleteCol() { editor?.chain().focus().deleteColumn().run(); showContextMenu = false; }
	function ctxDeleteTable() { editor?.chain().focus().deleteTable().run(); showContextMenu = false; }

	function ctxSetHeading(level: number) { editor?.chain().focus().setHeading({ level: level as 1|2|3|4|5|6 }).run(); showContextMenu = false; }
	function ctxSetParagraph() { editor?.chain().focus().setParagraph().run(); showContextMenu = false; }

	// Font family picker — dynamic from font sets
	let showFontPicker = $state(false);
	let fontFamilies = $derived.by(() => {
		const fonts: { label: string; value: string; group?: string }[] = [
			{ label: 'Default', value: '' },
		];
		const seen = new Set<string>();
		const allSets = getAllFontSets($appSettings.customFontSets || []);
		for (const set of allSets) {
			const entries = [
				{ label: `${set.name} — Text`, value: set.textFont },
				{ label: `${set.name} — Interface`, value: set.interfaceFont },
				{ label: `${set.name} — Mono`, value: set.monoFont },
			];
			for (const entry of entries) {
				// Replace double quotes with single quotes to avoid HTML style attribute conflicts
				const safeValue = entry.value?.replace(/"/g, "'") || '';
				if (safeValue && !seen.has(safeValue)) {
					seen.add(safeValue);
					fonts.push({ label: entry.label, value: safeValue, group: set.name });
				}
			}
		}
		return fonts;
	});

	function setFontFamily(font: string) {
		showFontPicker = false;
		if (!font) {
			editor?.chain().focus().unsetFontFamily().run();
		} else {
			// Replace double quotes with single quotes to avoid HTML attribute conflicts
			const safeFont = font.replace(/"/g, "'");
			editor?.chain().focus().setFontFamily(safeFont).run();
		}
	}

	function getCurrentFont(): string {
		return editor?.getAttributes('textStyle')?.fontFamily || '';
	}

	// Text color
	let showColorPicker = $state(false);
	const colorPalette = [
		'#000000', '#434343', '#666666', '#999999',
		'#e53e3e', '#dd6b20', '#d69e2e', '#38a169',
		'#3182ce', '#805ad5', '#d53f8c', '#2d3748',
	];
	function setColor(color: string) {
		showColorPicker = false;
		editor?.chain().focus().setColor(color).run();
	}
	function removeColor() {
		showColorPicker = false;
		editor?.chain().focus().unsetColor().run();
	}

	// Find & Replace
	let showFindReplace = $state(false);
	let findText = $state('');
	let replaceText = $state('');

	function findNext() {
		if (!findText || !editor) return;
		const { state } = editor;
		const { doc } = state;
		const searchLower = findText.toLowerCase();
		const cursorPos = state.selection.to;

		// Collect all text segments with their ProseMirror positions
		const segments: { text: string; pos: number }[] = [];
		doc.descendants((node, pos) => {
			if (node.isText) {
				segments.push({ text: node.text || '', pos });
			}
		});

		// Find matches across all text nodes
		type Match = { from: number; to: number };
		const matches: Match[] = [];
		for (const seg of segments) {
			let idx = 0;
			while (true) {
				const found = seg.text.toLowerCase().indexOf(searchLower, idx);
				if (found < 0) break;
				matches.push({
					from: seg.pos + found,
					to: seg.pos + found + findText.length,
				});
				idx = found + findText.length;
			}
		}

		if (matches.length === 0) return;

		// Find first match after cursor, or wrap to first match
		const nextMatch = matches.find(m => m.from >= cursorPos) || matches[0];
		editor.chain().focus().setTextSelection({ from: nextMatch.from, to: nextMatch.to }).run();
	}

	function replaceOne() {
		if (!editor || !findText) return;
		const { state } = editor;
		const selectedText = state.doc.textBetween(state.selection.from, state.selection.to);
		if (selectedText.toLowerCase() === findText.toLowerCase()) {
			editor.chain().focus().deleteSelection().insertContent(replaceText).run();
			findNext();
		} else {
			findNext();
		}
	}

	function replaceAll() {
		if (!editor || !findText) return;
		const { doc, tr } = editor.state;
		const searchLower = findText.toLowerCase();
		// Collect all text node matches with their ProseMirror positions
		const matches: { from: number; to: number }[] = [];
		doc.descendants((node, pos) => {
			if (!node.isText) return;
			const text = node.text || '';
			let idx = 0;
			while (true) {
				const found = text.toLowerCase().indexOf(searchLower, idx);
				if (found < 0) break;
				matches.push({
					from: pos + found,
					to: pos + found + findText.length,
				});
				idx = found + findText.length;
			}
		});
		if (matches.length === 0) return;
		// Apply replacements in reverse order to preserve positions
		for (let i = matches.length - 1; i >= 0; i--) {
			const { from, to } = matches[i];
			tr.replaceWith(from, to, editor.state.schema.text(replaceText));
		}
		editor.view.dispatch(tr);
	}

	// Word/character count
	let wordCount = $derived.by(() => {
		if (!editor || tick < 0) return 0;
		const text = editor.state?.doc?.textContent || '';
		return text.trim() ? text.trim().split(/\s+/).length : 0;
	});
	let charCount = $derived.by(() => {
		if (!editor || tick < 0) return 0;
		return editor.storage?.characterCount?.characters() ?? 0;
	});

	// Reactive state for toolbar button highlighting
	let tick = $state(0);
	$effect(() => {
		if (!editor) return;
		const handler = () => { tick++; };
		editor.on('selectionUpdate', handler);
		editor.on('transaction', handler);
		return () => {
			editor?.off('selectionUpdate', handler);
			editor?.off('transaction', handler);
		};
	});

	// Force reactivity
	const _ = $derived(tick);

	let showTableMenu = $state(false);
	let showHeadingMenu = $state(false);
</script>

<div class="tiptap-wrapper" dir={dir}>
	<!-- Toolbar -->
	<div class="tiptap-toolbar">
		<!-- Undo/Redo -->
		<button class="tt-btn" title={$t('editor.undo')} onclick={undo}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6.69 3L3 13"/></svg>
		</button>
		<button class="tt-btn" title={$t('editor.redo')} onclick={redo}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6.69 3L21 13"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Heading dropdown -->
		<div class="tt-dropdown">
			<button class="tt-btn tt-dropdown-trigger" onclick={() => showHeadingMenu = !showHeadingMenu}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 12h8"/><path d="M4 18V6"/><path d="M12 18V6"/><path d="M17 12l3-2v8"/></svg>
				<svg class="tt-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
			</button>
			{#if showHeadingMenu}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div class="tt-dropdown-menu" onclick={() => showHeadingMenu = false}>
					<button class="tt-menu-item" class:active={isActive('paragraph')} onclick={setParagraph}>Paragraph</button>
					<button class="tt-menu-item" class:active={isActive('heading', {level:1})} onclick={() => setHeading(1)}>Heading 1</button>
					<button class="tt-menu-item" class:active={isActive('heading', {level:2})} onclick={() => setHeading(2)}>Heading 2</button>
					<button class="tt-menu-item" class:active={isActive('heading', {level:3})} onclick={() => setHeading(3)}>Heading 3</button>
					<button class="tt-menu-item" class:active={isActive('heading', {level:4})} onclick={() => setHeading(4)}>Heading 4</button>
				</div>
			{/if}
		</div>

		<!-- Font family dropdown -->
		<div class="tt-dropdown">
			<button class="tt-btn tt-dropdown-trigger tt-font-trigger" title="Font" onclick={() => showFontPicker = !showFontPicker}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>
				<span class="tt-font-label">{fontFamilies.find(f => f.value === getCurrentFont())?.label || 'Font'}</span>
				<svg class="tt-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
			</button>
			{#if showFontPicker}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div class="tt-dropdown-menu tt-font-menu" onclick={() => showFontPicker = false}>
					{#each fontFamilies as font}
						<button
							class="tt-menu-item"
							class:active={getCurrentFont() === font.value}
							style={font.value ? `font-family: ${font.value}` : ''}
							onclick={() => setFontFamily(font.value)}
						>
							{font.label}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<span class="tt-sep"></span>

		<!-- Text formatting -->
		<button class="tt-btn" class:active={isActive('bold')} title="Bold" onclick={toggleBold}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('italic')} title="Italic" onclick={toggleItalic}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('underline')} title="Underline" onclick={toggleUnderline}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 3v7a6 6 0 0 0 6 6 6 6 0 0 0 6-6V3"/><line x1="4" y1="21" x2="20" y2="21"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('strike')} title="Strikethrough" onclick={toggleStrike}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4H9a3 3 0 0 0-2.83 4"/><path d="M14 12a4 4 0 0 1 0 8H6"/><line x1="4" y1="12" x2="20" y2="12"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('highlight')} title="Highlight" onclick={toggleHighlight}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 11-6 6v3h9l3-3"/><path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Text alignment -->
		<button class="tt-btn" class:active={isActive('paragraph', { textAlign: 'left' })} title="Align left" onclick={() => setTextAlign('left')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="17" y1="10" x2="3" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="17" y1="18" x2="3" y2="18"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('paragraph', { textAlign: 'center' })} title="Align center" onclick={() => setTextAlign('center')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="10" x2="6" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="18" y1="18" x2="6" y2="18"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('paragraph', { textAlign: 'right' })} title="Align right" onclick={() => setTextAlign('right')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="21" y1="10" x2="7" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="21" y1="18" x2="7" y2="18"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Lists -->
		<button class="tt-btn" class:active={isActive('bulletList')} title="Bullet list" onclick={toggleBulletList}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><circle cx="3" cy="6" r="1" fill="currentColor"/><circle cx="3" cy="12" r="1" fill="currentColor"/><circle cx="3" cy="18" r="1" fill="currentColor"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('orderedList')} title="Numbered list" onclick={toggleOrderedList}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="10" y1="6" x2="21" y2="6"/><line x1="10" y1="12" x2="21" y2="12"/><line x1="10" y1="18" x2="21" y2="18"/><path d="M4 6h1v4"/><path d="M4 10h2"/><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('taskList')} title="Task list" onclick={toggleTaskList}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Blocks -->
		<button class="tt-btn" class:active={isActive('blockquote')} title="Blockquote" onclick={toggleBlockquote}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"/><path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3c0 1 0 1 1 1z"/></svg>
		</button>
		<button class="tt-btn" class:active={isActive('codeBlock')} title="Code block" onclick={toggleCodeBlock}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
		</button>
		<button class="tt-btn" title="Horizontal rule" onclick={addHorizontalRule}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="2" y1="12" x2="22" y2="12"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Table -->
		<div class="tt-dropdown">
			<button class="tt-btn" title="Table" onclick={() => showTableMenu = !showTableMenu}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/></svg>
				<svg class="tt-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
			</button>
			{#if showTableMenu}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div class="tt-dropdown-menu" onclick={() => showTableMenu = false}>
					<button class="tt-menu-item" onclick={insertTable}>Insert table</button>
					<button class="tt-menu-item" onclick={addRowBefore}>Add row above</button>
					<button class="tt-menu-item" onclick={addRowAfter}>Add row below</button>
					<button class="tt-menu-item" onclick={addColumnBefore}>Add column before</button>
					<button class="tt-menu-item" onclick={addColumnAfter}>Add column after</button>
					<button class="tt-menu-item" onclick={deleteRow}>Delete row</button>
					<button class="tt-menu-item" onclick={deleteColumn}>Delete column</button>
					<button class="tt-menu-item tt-danger" onclick={deleteTable}>Delete table</button>
				</div>
			{/if}
		</div>

		<!-- Link & Image -->
		<button class="tt-btn" class:active={isActive('link')} title="Link" onclick={isActive('link') ? removeLink : insertLink}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
		</button>
		<button class="tt-btn" title="Image" onclick={insertImage}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Inline code -->
		<button class="tt-btn" class:active={isActive('code')} title="Inline code" onclick={toggleCode}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 18l6-6-6-6"/><path d="M8 6l-6 6 6 6"/></svg>
		</button>

		<!-- Subscript / Superscript -->
		<button class="tt-btn" class:active={isActive('subscript')} title="Subscript" onclick={toggleSubscript}>
			<span style="font-size:12px;font-weight:600;">X<sub style="font-size:8px;">₂</sub></span>
		</button>
		<button class="tt-btn" class:active={isActive('superscript')} title="Superscript" onclick={toggleSuperscript}>
			<span style="font-size:12px;font-weight:600;">X<sup style="font-size:8px;">²</sup></span>
		</button>

		<span class="tt-sep"></span>

		<!-- Callout -->
		<div class="tt-dropdown">
			<button class="tt-btn" title="Callout" onclick={() => showCalloutMenu = !showCalloutMenu}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
				<svg class="tt-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
			</button>
			{#if showCalloutMenu}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div class="tt-dropdown-menu" onclick={() => showCalloutMenu = false}>
					{#each calloutTypes as cType}
						<button class="tt-menu-item" onclick={() => insertCallout(cType)}>
							<span class="callout-icon">{cType === 'note' ? 'ℹ️' : cType === 'tip' ? '💡' : cType === 'warning' ? '⚠️' : cType === 'danger' ? '🔴' : cType === 'info' ? 'ℹ️' : cType === 'success' ? '✅' : cType === 'question' ? '❓' : cType === 'quote' ? '💬' : cType === 'example' ? '📝' : '🐛'}</span>
							{cType.charAt(0).toUpperCase() + cType.slice(1)}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Text color -->
		<div class="tt-dropdown">
			<button class="tt-btn" title="Text color" onclick={() => showColorPicker = !showColorPicker}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16"/><path d="M9.5 4L4.5 16h2l1-3h7l1 3h2L12.5 4z"/></svg>
			</button>
			{#if showColorPicker}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div class="tt-dropdown-menu tt-color-grid" onclick={(e) => e.stopPropagation()}>
					{#each colorPalette as c}
						<button class="tt-color-swatch" style="background:{c}" onclick={() => setColor(c)} title={c}></button>
					{/each}
					<button class="tt-menu-item" onclick={removeColor}>Remove color</button>
				</div>
			{/if}
		</div>

		<span class="tt-sep"></span>

		<!-- Indent / Outdent -->
		<button class="tt-btn" title="Outdent" onclick={outdent}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 6h10"/><path d="M11 12h10"/><path d="M11 18h10"/><path d="M3 8l4 4-4 4"/></svg>
		</button>
		<button class="tt-btn" title="Indent" onclick={indent}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 6h10"/><path d="M11 12h10"/><path d="M11 18h10"/><path d="M7 8L3 12l4 4"/></svg>
		</button>

		<!-- Clear formatting -->
		<button class="tt-btn" title="Clear formatting" onclick={clearFormatting}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><line x1="3" y1="21" x2="21" y2="3" stroke="currentColor" stroke-width="1.5" opacity="0.5"/></svg>
		</button>

		<!-- Find & Replace -->
		<button class="tt-btn" class:active={showFindReplace} title="Find & Replace" onclick={() => showFindReplace = !showFindReplace}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
		</button>

		<span class="tt-sep"></span>

		<!-- Word count -->
		<span class="tt-word-count" title="{charCount} characters">{wordCount} words</span>
	</div>

	<!-- Contextual toolbar -->
	{#if contextType}
		<div class="tt-context-bar">
			{#if contextType === 'table'}
				<span class="tt-context-label">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
					Table
				</span>
				<span class="tt-context-sep"></span>
				<button class="tt-ctx-btn" title={$t('contextMenu.addRow')} onclick={addRowBefore}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14"/><path d="M5 12h14"/></svg>
					Row ↑
				</button>
				<button class="tt-ctx-btn" title={$t('contextMenu.addRow')} onclick={addRowAfter}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14"/><path d="M5 12h14"/></svg>
					Row ↓
				</button>
				<button class="tt-ctx-btn" title={$t('contextMenu.addColumn')} onclick={addColumnBefore}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14"/><path d="M5 12h14"/></svg>
					Col ←
				</button>
				<button class="tt-ctx-btn" title={$t('contextMenu.addColumn')} onclick={addColumnAfter}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14"/><path d="M5 12h14"/></svg>
					Col →
				</button>
				<span class="tt-context-sep"></span>
				<button class="tt-ctx-btn" onclick={toggleHeaderRow}>Header row</button>
				<button class="tt-ctx-btn" onclick={toggleHeaderColumn}>Header col</button>
				<button class="tt-ctx-btn" onclick={mergeCells}>Merge</button>
				<button class="tt-ctx-btn" onclick={splitCell}>Split</button>
				<span class="tt-context-sep"></span>
				<button class="tt-ctx-btn tt-ctx-danger" title={$t('contextMenu.deleteRow')} onclick={deleteRow}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/></svg>
					Row
				</button>
				<button class="tt-ctx-btn tt-ctx-danger" title={$t('contextMenu.deleteColumn')} onclick={deleteColumn}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/></svg>
					Col
				</button>
				<button class="tt-ctx-btn tt-ctx-danger" onclick={deleteTable}>Delete table</button>

			{:else if contextType === 'link'}
				<span class="tt-context-label">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
					Link
				</span>
				<span class="tt-context-sep"></span>
				<span class="tt-ctx-url" title={contextLinkUrl}>{contextLinkUrl.length > 40 ? contextLinkUrl.slice(0, 40) + '…' : contextLinkUrl}</span>
				<button class="tt-ctx-btn" onclick={editLinkUrl}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
					{$t('contextMenu.editLink')}
				</button>
				<button class="tt-ctx-btn" onclick={openLinkUrl}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
					{$t('contextMenu.openLink')}
				</button>
				<button class="tt-ctx-btn tt-ctx-danger" onclick={removeLink}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18"/><path d="M6 6l12 12"/></svg>
					{$t('contextMenu.removeLink')}
				</button>

			{:else if contextType === 'image'}
				<span class="tt-context-label">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
					Image
				</span>
				<span class="tt-context-sep"></span>
				<button class="tt-ctx-btn" onclick={() => {
					const alt = prompt('Alt text:', editor?.getAttributes('image')?.alt || '');
					if (alt !== null) editor?.chain().focus().updateAttributes('image', { alt }).run();
				}}>Alt text</button>
				<button class="tt-ctx-btn" onclick={() => {
					const w = prompt('Width (e.g., 300, 50%):', editor?.getAttributes('image')?.width || '');
					if (w !== null) editor?.chain().focus().updateAttributes('image', { width: w }).run();
				}}>Resize</button>

			{:else if contextType === 'codeBlock'}
				<span class="tt-context-label">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
					Code
				</span>
				<span class="tt-context-sep"></span>
				<select class="tt-ctx-select" onchange={(e) => setCodeBlockLanguage((e.target as HTMLSelectElement).value)}
					value={editor?.getAttributes('codeBlock')?.language || ''}>
					{#each codeLanguages as lang}
						<option value={lang}>{lang || 'Plain text'}</option>
					{/each}
				</select>

			{:else if contextType === 'callout'}
				<span class="tt-context-label">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
					Callout
				</span>
				<span class="tt-context-sep"></span>
				{#each calloutTypes as cType}
					<button class="tt-ctx-btn" onclick={() => changeCalloutType(cType)}>
						{cType === 'note' ? 'ℹ️' : cType === 'tip' ? '💡' : cType === 'warning' ? '⚠️' : cType === 'danger' ? '🔴' : cType === 'info' ? 'ℹ️' : cType === 'success' ? '✅' : cType === 'question' ? '❓' : cType === 'quote' ? '💬' : cType === 'example' ? '📝' : '🐛'}
						{cType}
					</button>
				{/each}
			{/if}
		</div>
	{/if}

	<!-- Find & Replace bar -->
	{#if showFindReplace}
		<div class="tt-find-bar">
			<input class="tt-find-input" type="text" placeholder="Find..." bind:value={findText} onkeydown={(e) => { if (e.key === 'Enter') findNext(); }} />
			<input class="tt-find-input" type="text" placeholder="Replace..." bind:value={replaceText} />
			<button class="tt-find-btn" onclick={findNext}>Next</button>
			<button class="tt-find-btn" onclick={replaceOne}>Replace</button>
			<button class="tt-find-btn" onclick={replaceAll}>All</button>
			<button class="tt-find-close" onclick={() => showFindReplace = false}>×</button>
		</div>
	{/if}

	<!-- Editor -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="tiptap-editor" bind:this={editorEl} oncontextmenu={handleContextMenu}></div>

	<!-- Context Menu -->
	{#if showContextMenu}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="ctx-menu" style="left:{contextMenuX}px;top:{contextMenuY}px" onclick={() => showContextMenu = false}>
			<!-- Clipboard -->
			<button class="ctx-item" onclick={ctxCut} disabled={!contextHasSelection}>
				<span class="ctx-label">{$t('contextMenu.cut')}</span><span class="ctx-shortcut">Ctrl+X</span>
			</button>
			<button class="ctx-item" onclick={ctxCopy} disabled={!contextHasSelection}>
				<span class="ctx-label">{$t('contextMenu.copy')}</span><span class="ctx-shortcut">Ctrl+C</span>
			</button>
			<button class="ctx-item" onclick={ctxPaste}>
				<span class="ctx-label">{$t('contextMenu.paste')}</span><span class="ctx-shortcut">Ctrl+V</span>
			</button>
			<button class="ctx-item" onclick={ctxSelectAll}>
				<span class="ctx-label">Select All</span><span class="ctx-shortcut">Ctrl+A</span>
			</button>

			<div class="ctx-sep"></div>

			<!-- Formatting (shown when text is selected) -->
			{#if contextHasSelection}
				<button class="ctx-item" onclick={ctxBold}>
					<span class="ctx-label">{$t('contextMenu.bold')}</span><span class="ctx-shortcut">Ctrl+B</span>
				</button>
				<button class="ctx-item" onclick={ctxItalic}>
					<span class="ctx-label">{$t('contextMenu.italic')}</span><span class="ctx-shortcut">Ctrl+I</span>
				</button>
				<button class="ctx-item" onclick={ctxUnderline}>
					<span class="ctx-label">{$t('contextMenu.underline')}</span><span class="ctx-shortcut">Ctrl+U</span>
				</button>
				<button class="ctx-item" onclick={ctxStrikethrough}>
					<span class="ctx-label">{$t('contextMenu.strikethrough')}</span>
				</button>
				<button class="ctx-item" onclick={ctxHighlight}>
					<span class="ctx-label">{$t('contextMenu.highlight')}</span>
				</button>
				<button class="ctx-item" onclick={ctxInlineCode}>
					<span class="ctx-label">{$t('contextMenu.inlineCode')}</span>
				</button>
				<button class="ctx-item" onclick={ctxClearFormatting}>
					<span class="ctx-label">{$t('contextMenu.clearFormatting')}</span>
				</button>

				<div class="ctx-sep"></div>
			{/if}

			<!-- Heading submenu -->
			<button class="ctx-item" onclick={() => ctxSetParagraph()}>
				<span class="ctx-label">{$t('contextMenu.paragraph')}</span>
			</button>
			<button class="ctx-item" onclick={() => ctxSetHeading(1)}>
				<span class="ctx-label">{$t('contextMenu.heading')} 1</span>
			</button>
			<button class="ctx-item" onclick={() => ctxSetHeading(2)}>
				<span class="ctx-label">{$t('contextMenu.heading')} 2</span>
			</button>
			<button class="ctx-item" onclick={() => ctxSetHeading(3)}>
				<span class="ctx-label">{$t('contextMenu.heading')} 3</span>
			</button>

			<div class="ctx-sep"></div>

			<!-- Block elements -->
			<button class="ctx-item" onclick={ctxBulletList}>
				<span class="ctx-label">{$t('contextMenu.bulletList')}</span>
			</button>
			<button class="ctx-item" onclick={ctxOrderedList}>
				<span class="ctx-label">{$t('contextMenu.numberedList')}</span>
			</button>
			<button class="ctx-item" onclick={ctxTaskList}>
				<span class="ctx-label">{$t('contextMenu.taskList')}</span>
			</button>
			<button class="ctx-item" onclick={ctxBlockquote}>
				<span class="ctx-label">{$t('contextMenu.blockquote')}</span>
			</button>
			<button class="ctx-item" onclick={ctxCodeBlock}>
				<span class="ctx-label">{$t('contextMenu.codeBlock')}</span>
			</button>
			<button class="ctx-item" onclick={ctxHorizontalRule}>
				<span class="ctx-label">{$t('contextMenu.horizontalRule')}</span>
			</button>

			<!-- Link actions -->
			<div class="ctx-sep"></div>
			{#if contextOnLink}
				<button class="ctx-item" onclick={ctxEditLink}>
					<span class="ctx-label">{$t('contextMenu.editLink')}</span>
				</button>
				<button class="ctx-item" onclick={ctxOpenLink}>
					<span class="ctx-label">{$t('contextMenu.openLink')}</span>
				</button>
				<button class="ctx-item" onclick={ctxRemoveLink}>
					<span class="ctx-label">{$t('contextMenu.removeLink')}</span>
				</button>
			{:else}
				<button class="ctx-item" onclick={ctxInsertLink}>
					<span class="ctx-label">{$t('contextMenu.link')}</span>
				</button>
			{/if}

			<!-- Table actions (only when inside a table) -->
			{#if contextInTable}
				<div class="ctx-sep"></div>
				<div class="ctx-group-label">{$t('contextMenu.table')}</div>
				<button class="ctx-item" onclick={ctxAddRowBefore}>
					<span class="ctx-label">{$t('contextMenu.addRow')} ↑</span>
				</button>
				<button class="ctx-item" onclick={ctxAddRowAfter}>
					<span class="ctx-label">{$t('contextMenu.addRow')} ↓</span>
				</button>
				<button class="ctx-item" onclick={ctxAddColBefore}>
					<span class="ctx-label">{$t('contextMenu.addColumn')} ←</span>
				</button>
				<button class="ctx-item" onclick={ctxAddColAfter}>
					<span class="ctx-label">{$t('contextMenu.addColumn')} →</span>
				</button>
				<button class="ctx-item ctx-danger" onclick={ctxDeleteRow}>
					<span class="ctx-label">{$t('contextMenu.deleteRow')}</span>
				</button>
				<button class="ctx-item ctx-danger" onclick={ctxDeleteCol}>
					<span class="ctx-label">{$t('contextMenu.deleteColumn')}</span>
				</button>
				<button class="ctx-item ctx-danger" onclick={ctxDeleteTable}>
					<span class="ctx-label">{$t('contextMenu.table')} ✕</span>
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.tiptap-wrapper {
		display: flex;
		flex-direction: column;
		flex: 1;
		overflow: hidden;
	}

	.tiptap-toolbar {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 4px 8px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		flex-wrap: wrap;
		min-height: 36px;
	}

	.tt-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 2px;
		width: 28px;
		height: 28px;
		border: none;
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		color: var(--text-muted);
		transition: all 0.15s;
	}
	.tt-btn:hover { background: var(--background-modifier-hover); color: var(--text-normal); }
	.tt-btn.active { background: var(--interactive-accent); color: white; }
	.tt-btn.active:hover { background: var(--interactive-accent-hover); }

	.tt-dropdown-trigger { width: auto; padding: 0 6px; }

	.tt-sep {
		width: 1px;
		height: 20px;
		background: var(--background-modifier-border);
		margin: 0 3px;
	}

	.tt-chevron { opacity: 0.5; }

	.tt-dropdown { position: relative; }
	.tt-dropdown-menu {
		position: absolute;
		top: 100%;
		left: 0;
		z-index: 100;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.15);
		padding: 4px;
		min-width: 160px;
	}
	.tt-menu-item {
		display: block;
		width: 100%;
		text-align: start;
		padding: 6px 10px;
		border: none;
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		font-size: 13px;
		color: var(--text-normal);
	}
	.tt-menu-item:hover { background: var(--background-modifier-hover); }
	.tt-menu-item.active { color: var(--interactive-accent); font-weight: 600; }
	.tt-menu-item.tt-danger { color: var(--text-error, #e53e3e); }
	.tt-menu-item.tt-danger:hover { background: rgba(229, 62, 62, 0.1); }

	/* Font picker */
	.tt-font-trigger { gap: 4px; }
	.tt-font-label {
		font-size: 11px;
		max-width: 80px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tt-font-menu {
		max-height: 300px;
		overflow-y: auto;
		min-width: 180px;
	}

	/* Word count */
	.tt-word-count {
		font-size: 11px;
		color: var(--text-faint);
		padding: 0 6px;
		white-space: nowrap;
		user-select: none;
	}

	/* Color picker grid */
	.tt-color-grid {
		display: grid !important;
		grid-template-columns: repeat(4, 1fr);
		gap: 4px;
		padding: 8px;
		min-width: 140px;
	}
	.tt-color-grid .tt-menu-item {
		grid-column: 1 / -1;
		text-align: center;
		margin-top: 4px;
	}
	.tt-color-swatch {
		width: 24px;
		height: 24px;
		border-radius: 4px;
		border: 2px solid var(--background-modifier-border);
		cursor: pointer;
		transition: transform 0.1s;
	}
	.tt-color-swatch:hover { transform: scale(1.2); border-color: var(--text-normal); }

	/* Callout icon */
	.callout-icon { margin-inline-end: 6px; }

	/* Contextual toolbar */
	.tt-context-bar {
		display: flex;
		align-items: center;
		gap: 3px;
		padding: 3px 8px;
		background: var(--background-primary);
		border-bottom: 1px solid var(--background-modifier-border);
		flex-wrap: wrap;
		min-height: 30px;
		animation: tt-ctx-slide 0.15s ease-out;
	}
	@keyframes tt-ctx-slide {
		from { opacity: 0; transform: translateY(-4px); }
		to { opacity: 1; transform: translateY(0); }
	}
	.tt-context-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-accent);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		padding: 2px 6px;
		user-select: none;
	}
	.tt-context-sep {
		width: 1px;
		height: 16px;
		background: var(--background-modifier-border);
		margin: 0 3px;
	}
	.tt-ctx-btn {
		display: flex;
		align-items: center;
		gap: 3px;
		padding: 3px 7px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--text-normal);
		cursor: pointer;
		font-size: 11px;
		font-family: var(--font-interface-theme);
		white-space: nowrap;
	}
	.tt-ctx-btn:hover { background: var(--background-modifier-hover); }
	.tt-ctx-danger { color: var(--text-error, #e53e3e); }
	.tt-ctx-danger:hover { background: color-mix(in srgb, var(--text-error, #e53e3e) 10%, transparent); }
	.tt-ctx-url {
		font-size: 11px;
		color: var(--text-muted);
		max-width: 250px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding: 2px 6px;
		background: var(--background-secondary);
		border-radius: 4px;
		font-family: var(--font-monospace-theme);
	}
	.tt-ctx-select {
		font-size: 11px;
		padding: 2px 6px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-family: var(--font-interface-theme);
	}

	/* Find & Replace bar */
	.tt-find-bar {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
	}
	.tt-find-input {
		flex: 1;
		max-width: 200px;
		height: 26px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: var(--background-primary);
		color: var(--text-normal);
		padding: 0 8px;
		font-size: 12px;
		font-family: var(--font-interface-theme);
	}
	.tt-find-input:focus { border-color: var(--interactive-accent); outline: none; }
	.tt-find-btn {
		height: 26px;
		padding: 0 10px;
		border: 1px solid var(--background-modifier-border);
		border-radius: 4px;
		background: var(--background-primary);
		color: var(--text-normal);
		font-size: 12px;
		cursor: pointer;
		font-family: var(--font-interface-theme);
	}
	.tt-find-btn:hover { background: var(--background-modifier-hover); }
	.tt-find-close {
		width: 26px;
		height: 26px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 16px;
		border-radius: 4px;
	}
	.tt-find-close:hover { background: var(--background-modifier-hover); color: var(--text-normal); }

	.tiptap-editor {
		flex: 1;
		overflow-y: auto;
		padding: 1rem 2rem;
	}

	/* TipTap ProseMirror content styles */
	.tiptap-editor :global(.tiptap-content) {
		outline: none;
		min-height: 100%;
		font-family: var(--font-text-theme, inherit);
		font-size: var(--font-text-size, 15px);
		line-height: 1.7;
		color: var(--text-normal);
	}

	.tiptap-editor :global(.tiptap-content p) { margin: 0 0 0.75rem; }
	.tiptap-editor :global(.tiptap-content h1) { font-size: 2rem; font-weight: 700; margin: 1.5rem 0 0.75rem; }
	.tiptap-editor :global(.tiptap-content h2) { font-size: 1.6rem; font-weight: 600; margin: 1.25rem 0 0.5rem; }
	.tiptap-editor :global(.tiptap-content h3) { font-size: 1.3rem; font-weight: 600; margin: 1rem 0 0.5rem; }
	.tiptap-editor :global(.tiptap-content h4) { font-size: 1.1rem; font-weight: 600; margin: 0.75rem 0 0.4rem; }
	.tiptap-editor :global(.tiptap-content h5) { font-size: 1rem; font-weight: 600; margin: 0.6rem 0 0.3rem; }
	.tiptap-editor :global(.tiptap-content h6) { font-size: 0.9rem; font-weight: 600; margin: 0.5rem 0 0.25rem; color: var(--text-muted); }

	.tiptap-editor :global(.tiptap-content blockquote) {
		border-inline-start: 3px solid var(--interactive-accent);
		padding-inline-start: 1rem;
		margin: 0.75rem 0;
		color: var(--text-muted);
	}

	.tiptap-editor :global(.tiptap-content code) {
		background: var(--background-secondary);
		padding: 2px 5px;
		border-radius: 3px;
		font-family: var(--font-monospace-theme, monospace);
		font-size: 0.9em;
	}

	.tiptap-editor :global(.tiptap-content pre) {
		background: var(--background-secondary);
		padding: 1rem;
		border-radius: 6px;
		overflow-x: auto;
		margin: 0.75rem 0;
	}
	.tiptap-editor :global(.tiptap-content pre code) {
		background: none;
		padding: 0;
	}

	.tiptap-editor :global(.tiptap-content hr) {
		border: none;
		border-top: 2px solid var(--background-modifier-border);
		margin: 1.5rem 0;
	}

	.tiptap-editor :global(.tiptap-content img) {
		max-width: 100%;
		border-radius: 6px;
		margin: 0.5rem 0;
	}

	.tiptap-editor :global(.tiptap-content a) {
		color: var(--interactive-accent);
		text-decoration: underline;
	}

	.tiptap-editor :global(.tiptap-content mark) {
		background: rgba(255, 212, 0, 0.4);
		border-radius: 2px;
		padding: 1px 2px;
	}

	/* Table styles */
	.tiptap-editor :global(.tiptap-content table) {
		border-collapse: collapse;
		width: 100%;
		margin: 1rem 0;
	}
	.tiptap-editor :global(.tiptap-content th),
	.tiptap-editor :global(.tiptap-content td) {
		border: 1px solid var(--background-modifier-border);
		padding: 8px 12px;
		text-align: start;
		min-width: 80px;
	}
	.tiptap-editor :global(.tiptap-content th) {
		background: var(--background-secondary);
		font-weight: 600;
	}
	.tiptap-editor :global(.tiptap-content .selectedCell) {
		background: rgba(139, 92, 246, 0.12);
	}
	.tiptap-editor :global(.tiptap-content .column-resize-handle) {
		position: absolute;
		right: -2px;
		top: 0;
		bottom: 0;
		width: 4px;
		background: var(--interactive-accent);
		cursor: col-resize;
	}

	/* Task list */
	.tiptap-editor :global(.tiptap-content ul[data-type="taskList"]) {
		list-style: none;
		padding: 0;
	}
	.tiptap-editor :global(.tiptap-content ul[data-type="taskList"] li) {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		margin: 4px 0;
	}
	.tiptap-editor :global(.tiptap-content ul[data-type="taskList"] li label) {
		margin-top: 3px;
	}

	/* Placeholder */
	.tiptap-editor :global(.tiptap-content p.is-editor-empty:first-child::before) {
		content: attr(data-placeholder);
		float: left;
		color: var(--text-faint);
		pointer-events: none;
		height: 0;
	}

	/* Lists */
	.tiptap-editor :global(.tiptap-content ul),
	.tiptap-editor :global(.tiptap-content ol) {
		padding-inline-start: 1.5rem;
		margin: 0.5rem 0;
	}
	.tiptap-editor :global(.tiptap-content li) {
		margin: 2px 0;
	}

	/* Subscript / Superscript */
	.tiptap-editor :global(.tiptap-content sub) { font-size: 0.75em; }
	.tiptap-editor :global(.tiptap-content sup) { font-size: 0.75em; }

	/* ─── Context Menu ─── */
	.ctx-menu {
		position: fixed;
		z-index: 9999;
		min-width: 220px;
		max-height: 70vh;
		overflow-y: auto;
		background: var(--background-primary);
		border: 1px solid var(--background-modifier-border);
		border-radius: 8px;
		box-shadow: 0 8px 24px rgba(0,0,0,0.15);
		padding: 4px 0;
		font-family: var(--font-interface-theme);
		font-size: 13px;
		direction: ltr;
		text-align: left;
	}
	.ctx-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 6px 14px;
		border: none;
		background: transparent;
		color: var(--text-normal);
		cursor: pointer;
		font-family: inherit;
		font-size: inherit;
		text-align: start;
	}
	.ctx-item:hover { background: var(--background-modifier-hover); }
	.ctx-item:disabled { opacity: 0.4; cursor: default; }
	.ctx-item:disabled:hover { background: transparent; }
	.ctx-item.ctx-danger { color: #ef4444; }
	.ctx-item.ctx-danger:hover { background: rgba(239,68,68,0.08); }
	.ctx-label { flex: 1; }
	.ctx-shortcut {
		margin-inline-start: 24px;
		font-size: 11px;
		color: var(--text-faint);
		white-space: nowrap;
	}
	.ctx-sep {
		height: 1px;
		margin: 4px 0;
		background: var(--background-modifier-border);
	}
	.ctx-group-label {
		padding: 4px 14px 2px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
</style>

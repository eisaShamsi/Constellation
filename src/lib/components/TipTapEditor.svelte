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
	import { marked } from 'marked';
	import TurndownService from 'turndown';
	import { t } from '$lib/i18n';

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

		// Table support
		td.addRule('tableCell', {
			filter: ['th', 'td'],
			replacement(content, node) {
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
			replacement(content, node) {
				const rows = content.trim().split('\n').filter(Boolean);
				if (rows.length === 0) return '';
				const firstRow = rows[0];
				const cols = (firstRow.match(/\|/g) || []).length - 1;
				const separator = '|' + ' --- |'.repeat(cols);
				return '\n' + rows[0] + separator + '\n' + rows.slice(1).join('\n') + '\n';
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
		return marked.parse(md, { async: false }) as string;
	}

	function htmlToMarkdown(html: string): string {
		if (!turndown) turndown = initTurndown();
		return turndown.turndown(html);
	}

	onMount(() => {
		turndown = initTurndown();

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

	function insertImage() {
		const url = prompt('Image URL:');
		if (url) editor?.chain().focus().setImage({ src: url }).run();
	}

	function insertLink() {
		const url = prompt('Link URL:');
		if (url) editor?.chain().focus().setLink({ href: url }).run();
	}
	function removeLink() { editor?.chain().focus().unsetLink().run(); }

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
	</div>

	<!-- Editor -->
	<div class="tiptap-editor" bind:this={editorEl}></div>
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
</style>

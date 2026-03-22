<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditEngine } from './core/EditEngine';
	import { InputHandler } from './core/InputHandler';
	import { EditorSelection } from './core/Selection';
	import { MarkdownParser } from './parser/MarkdownParser';
	import { DecorationEngine } from './render/DecorationEngine';
	import { ViewportRenderer } from './render/ViewportRenderer';
	import { t } from '$lib/i18n';
	import Toolbar from './ui/Toolbar.svelte';
	import ContextMenu from './ui/ContextMenu.svelte';
	import './theme/editor.css';
	import './theme/decorations.css';

	let {
		value = '',
		dir = 'ltr' as 'ltr' | 'rtl' | 'auto',
		placeholder = '',
		onchange,
		onsave,
		onlinkclick,
		readonly = false,
		noteNames = [] as { name: string; path: string; libraryName?: string }[],
		allTags = [] as string[],
		initialCursorPos = 0,
		initialScrollTop = 0,
		onCursorChange,
		onScrollChange,
		showLineNumbers = false,
	}: {
		value?: string;
		dir?: 'ltr' | 'rtl' | 'auto';
		placeholder?: string;
		onchange?: (markdown: string) => void;
		onsave?: () => void;
		onlinkclick?: (target: string) => void;
		readonly?: boolean;
		noteNames?: { name: string; path: string; libraryName?: string }[];
		allTags?: string[];
		initialCursorPos?: number;
		initialScrollTop?: number;
		onCursorChange?: (pos: number) => void;
		onScrollChange?: (top: number) => void;
		showLineNumbers?: boolean;
	} = $props();

	let containerEl: HTMLDivElement;
	let editorEl: HTMLDivElement;
	let engine: EditEngine;
	let inputHandler: InputHandler;
	let parser: MarkdownParser;
	let decorationEngine: DecorationEngine;
	let renderer: ViewportRenderer;
	let rafId: number | null = null;
	let changeDebounce: ReturnType<typeof setTimeout> | null = null;
	let lastExternalValue = value;
	let wordCount = $state(0);

	// Context menu state
	let contextMenuVisible = $state(false);
	let contextMenuX = $state(0);
	let contextMenuY = $state(0);

	// Track toolbar active state via direct DOM (not reactivity)
	let toolbarEl: HTMLElement | null = null;

	function initEditor() {
		engine = new EditEngine(value);
		parser = new MarkdownParser(engine.buffer);
		decorationEngine = new DecorationEngine();
		renderer = new ViewportRenderer(editorEl, engine.buffer);

		// Initial parse and render
		const blocks = parser.parse();
		const decos = decorationEngine.buildFromBlocks(blocks, engine.selection.head);
		renderer.setBlocks(blocks);
		renderer.setDecorations(decos);

		// Set up click handler
		renderer.setLineClickHandler((offset) => {
			engine.moveCursor(offset);
			updateAfterCursorMove();
			editorEl.focus();
		});

		// Set up checkbox toggle
		renderer.setCheckboxToggleHandler((line) => {
			const lineStart = engine.buffer.getLineStart(line);
			const lineText = engine.buffer.getLine(line);
			const match = lineText.match(/^(\s*)- \[([ x])\] /);
			if (match) {
				const checkOffset = lineStart + match[1].length + 3;
				const newChar = match[2] === ' ' ? 'x' : ' ';
				engine.buffer.replace(checkOffset, 1, newChar);
				scheduleRender();
				emitChange();
			}
		});

		// Set up input handler
		inputHandler = new InputHandler(engine, editorEl);
		inputHandler.attach();

		// Add Ctrl+S binding
		inputHandler.addKeyBinding({
			key: 's',
			ctrl: true,
			action: () => { if (onsave) onsave(); },
		});

		// Add Ctrl+K for links
		inputHandler.addKeyBinding({
			key: 'k',
			ctrl: true,
			action: (e) => e.insertLink('https://', ''),
		});

		// Listen to engine changes
		engine.onChange((changes, selection) => {
			scheduleRender();
			emitChange();
		});

		// Initial render
		renderer.render();
		renderer.renderCursor(engine.selection);

		// Restore cursor position
		if (initialCursorPos > 0) {
			engine.moveCursor(Math.min(initialCursorPos, engine.length));
		}
		if (initialScrollTop > 0) {
			renderer.scrollTop = initialScrollTop;
		}

		updateWordCount();
	}

	function scheduleRender() {
		if (rafId !== null) return;
		rafId = requestAnimationFrame(() => {
			rafId = null;
			reparse();
			renderer.render();
			renderer.renderCursor(engine.selection);
			updateToolbarState();
		});
	}

	function reparse() {
		const blocks = parser.parse(); // Full reparse for now; incremental later
		const decos = decorationEngine.buildFromBlocks(blocks, engine.selection.head);
		renderer.setBlocks(blocks);
		renderer.setDecorations(decos);
	}

	function updateAfterCursorMove() {
		// Rebuild decorations with new cursor position (for reveal behavior)
		const blocks = parser.getBlocks();
		const decos = decorationEngine.buildFromBlocks(blocks, engine.selection.head);
		renderer.setDecorations(decos);
		renderer.render();
		renderer.renderCursor(engine.selection);
		renderer.scrollToOffset(engine.selection.head);
		updateToolbarState();

		if (onCursorChange) onCursorChange(engine.selection.head);
	}

	function emitChange() {
		if (changeDebounce) clearTimeout(changeDebounce);
		changeDebounce = setTimeout(() => {
			const md = engine.getText();
			if (onchange) onchange(md);
			updateWordCount();
		}, 100);
	}

	function updateWordCount() {
		const text = engine.getText();
		const words = text.trim().split(/\s+/).filter(w => w.length > 0);
		wordCount = words.length;
	}

	function updateToolbarState() {
		if (!toolbarEl) return;

		requestAnimationFrame(() => {
			if (!toolbarEl || !engine) return;
			const head = engine.selection.head;
			const line = engine.buffer.getLineFromOffset(head);
			const lineText = engine.buffer.getLine(line);

			// Check inline marks at cursor
			const { from, to } = engine.selection;
			const textBefore = engine.buffer.getText(Math.max(0, from - 50), Math.min(50, from));
			const textAfter = engine.buffer.getText(to, Math.min(50, engine.length - to));

			// Bold
			toggleToolbarBtn(toolbarEl, 'bold', isWrappedBy(textBefore, textAfter, '**'));
			// Italic
			toggleToolbarBtn(toolbarEl, 'italic', isWrappedBy(textBefore, textAfter, '*') && !isWrappedBy(textBefore, textAfter, '**'));
			// Strikethrough
			toggleToolbarBtn(toolbarEl, 'strikethrough', isWrappedBy(textBefore, textAfter, '~~'));
			// Highlight
			toggleToolbarBtn(toolbarEl, 'highlight', isWrappedBy(textBefore, textAfter, '=='));
			// Heading
			const headingMatch = lineText.match(/^(#{1,6})\s/);
			const headingLevel = headingMatch ? headingMatch[1].length : 0;
			for (let i = 1; i <= 6; i++) {
				toggleToolbarBtn(toolbarEl, `h${i}`, headingLevel === i);
			}
			// Lists
			toggleToolbarBtn(toolbarEl, 'bullet-list', /^(\s*)[-*+] /.test(lineText));
			toggleToolbarBtn(toolbarEl, 'ordered-list', /^(\s*)\d+\. /.test(lineText));
			toggleToolbarBtn(toolbarEl, 'task-list', /^(\s*)- \[([ x])\] /.test(lineText));
			// Blockquote
			toggleToolbarBtn(toolbarEl, 'blockquote', lineText.startsWith('> '));
		});
	}

	function isWrappedBy(before: string, after: string, syntax: string): boolean {
		return before.endsWith(syntax) && after.startsWith(syntax);
	}

	function toggleToolbarBtn(toolbar: HTMLElement, name: string, active: boolean) {
		const btn = toolbar.querySelector(`[data-action="${name}"]`);
		if (btn) btn.classList.toggle('active', active);
	}

	// Handle toolbar actions
	function handleToolbarAction(event: CustomEvent) {
		const { action, data } = event.detail;
		editorEl.focus();

		switch (action) {
			case 'bold': engine.toggleMark('**'); break;
			case 'italic': engine.toggleMark('*'); break;
			case 'underline': engine.toggleMark('<u>'); break;
			case 'strikethrough': engine.toggleMark('~~'); break;
			case 'highlight': engine.toggleMark('=='); break;
			case 'code': engine.toggleMark('`'); break;
			case 'h1': engine.setHeading(1); break;
			case 'h2': engine.setHeading(2); break;
			case 'h3': engine.setHeading(3); break;
			case 'h4': engine.setHeading(4); break;
			case 'h5': engine.setHeading(5); break;
			case 'h6': engine.setHeading(6); break;
			case 'paragraph': engine.setHeading(0); break;
			case 'bullet-list': engine.toggleList('bullet'); break;
			case 'ordered-list': engine.toggleList('ordered'); break;
			case 'task-list': engine.toggleList('task'); break;
			case 'blockquote': engine.toggleBlockquote(); break;
			case 'code-block': engine.insertCodeBlock(data?.language); break;
			case 'link': engine.insertLink(data?.url ?? 'https://', data?.text); break;
			case 'image': engine.insertImage(data?.src ?? '', data?.alt); break;
			case 'table': engine.insertTable(data?.rows ?? 3, data?.cols ?? 3); break;
			case 'callout': engine.insertCallout(data?.type ?? 'info'); break;
			case 'hr': engine.insertHorizontalRule(); break;
			case 'subscript': engine.toggleMark('<sub>'); break;
			case 'superscript': engine.toggleMark('<sup>'); break;
			case 'indent': engine.indent(false); break;
			case 'outdent': engine.indent(true); break;
			case 'undo': engine.undo(); break;
			case 'redo': engine.redo(); break;
			case 'clear':
				// Clear formatting — remove inline marks around selection
				break;
			case 'select-all': engine.selectAll(); break;
			case 'font-family':
				if (data?.font) engine.wrapWithSpan(`font-family: ${data.font}`);
				break;
			case 'font-size':
				if (data?.size) engine.wrapWithSpan(`font-size: ${data.size}`);
				break;
			case 'color':
				if (data?.color) engine.wrapWithSpan(`color: ${data.color}`);
				break;
		}

		scheduleRender();
	}

	function handleContextMenu(event: MouseEvent) {
		event.preventDefault();
		contextMenuX = event.clientX;
		contextMenuY = event.clientY;
		contextMenuVisible = true;
	}

	function handleContextMenuAction(event: CustomEvent) {
		contextMenuVisible = false;
		// Reuse toolbar action handler
		handleToolbarAction(event);
	}

	// Watch for external value changes
	$effect(() => {
		if (value !== lastExternalValue && engine) {
			lastExternalValue = value;
			const currentText = engine.getText();
			if (value !== currentText) {
				engine.setContent(value);
				reparse();
				renderer.render();
				renderer.renderCursor(engine.selection);
			}
		}
	});

	// Scroll change tracking
	$effect(() => {
		if (renderer && onScrollChange) {
			const scroller = renderer.getScroller();
			const handler = () => {
				if (onScrollChange) onScrollChange(scroller.scrollTop);
			};
			scroller.addEventListener('scroll', handler, { passive: true });
			return () => scroller.removeEventListener('scroll', handler);
		}
	});

	onMount(() => {
		initEditor();
	});

	onDestroy(() => {
		if (inputHandler) inputHandler.detach();
		if (renderer) renderer.destroy();
		if (rafId !== null) cancelAnimationFrame(rafId);
		if (changeDebounce) clearTimeout(changeDebounce);
	});
</script>

<div
	class="ce-container"
	class:ce-readonly={readonly}
	bind:this={containerEl}
	dir={dir}
>
	{#if !readonly}
		<Toolbar
			bind:el={toolbarEl}
			onaction={handleToolbarAction}
			{wordCount}
			{dir}
		/>
	{/if}

	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="ce-editor"
		bind:this={editorEl}
		contenteditable={!readonly}
		role="textbox"
		aria-multiline="true"
		aria-label={$t('editor.ariaLabel')}
		tabindex="0"
		spellcheck="true"
		oncontextmenu={handleContextMenu}
		data-placeholder={placeholder}
	></div>

	{#if contextMenuVisible}
		<ContextMenu
			x={contextMenuX}
			y={contextMenuY}
			onaction={handleContextMenuAction}
			onclose={() => contextMenuVisible = false}
			{dir}
		/>
	{/if}
</div>

<style>
	.ce-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		width: 100%;
		position: relative;
		overflow: hidden;
	}

	.ce-editor {
		flex: 1;
		overflow: hidden;
		position: relative;
		outline: none;
		cursor: text;
		font-family: var(--font-text, -apple-system, BlinkMacSystemFont, 'Segoe UI', Inter, sans-serif);
		font-size: var(--font-size, 16px);
		line-height: 1.6;
		padding: 16px 24px;
		color: var(--text-primary, #1f2328);
	}

	.ce-editor:empty::before {
		content: attr(data-placeholder);
		color: var(--text-muted, #8b8b8b);
		pointer-events: none;
		position: absolute;
	}

	.ce-readonly {
		opacity: 0.8;
	}

	.ce-readonly .ce-editor {
		cursor: default;
	}
</style>

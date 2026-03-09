<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorView, keymap, placeholder as cmPlaceholder, drawSelection, dropCursor, highlightActiveLine, highlightActiveLineGutter, lineNumbers, highlightSpecialChars } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { languages } from '@codemirror/language-data';
	import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
	import { searchKeymap, highlightSelectionMatches, openSearchPanel } from '@codemirror/search';
	import { closeBrackets, closeBracketsKeymap, autocompletion, type CompletionContext, type Completion } from '@codemirror/autocomplete';
	import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput, foldGutter, foldKeymap } from '@codemirror/language';
	import type { FileEntry } from '$lib/vaults/store';

	import { saveClipboardImage } from '$lib/vaults/store';

	let {
		value = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		placeholder = '',
		onchange,
		noteNames = [] as { name: string; path: string }[],
		allTags = [] as string[],
		ar = false,
		vaultPath = '',
	}: {
		value: string;
		dir?: 'ltr' | 'rtl';
		placeholder?: string;
		onchange: (value: string) => void;
		noteNames?: { name: string; path: string }[];
		allTags?: string[];
		ar?: boolean;
		vaultPath?: string;
	} = $props();

	let containerEl: HTMLDivElement;
	let view: EditorView | undefined;
	let dirCompartment = new Compartment();
	let updating = false;

	// Smart pair wrapping
	const WRAP_PAIRS: Record<string, string> = {
		'(': ')', '[': ']', '{': '}', '"': '"', "'": "'", '`': '`', '_': '_', '*': '*',
	};

	function smartPairKeymap() {
		return Object.entries(WRAP_PAIRS).map(([open, close]) => ({
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
	function wikilinkCompletion(context: CompletionContext) {
		const before = context.matchBefore(/\[\[[^\]]*$/);
		if (!before) return null;
		const query = before.text.slice(2).toLowerCase();
		const options: Completion[] = noteNames
			.filter(n => n.name.toLowerCase().includes(query))
			.slice(0, 20)
			.map(n => ({
				label: n.name,
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
		];
		return {
			from: before.from,
			options: commands.map(c => ({
				...c,
				apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
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
		...smartPairKeymap(),
	]);

	// Obsidian-like theme
	const obsidianTheme = EditorView.theme({
		'&': {
			fontSize: '0.92rem',
			height: '100%',
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
			backgroundColor: 'hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.25) !important',
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

	onMount(() => {
		const startState = EditorState.create({
			doc: value,
			extensions: [
				lineNumbers(),
				highlightActiveLineGutter(),
				highlightSpecialChars(),
				history(),
				foldGutter(),
				drawSelection(),
				dropCursor(),
				indentOnInput(),
				bracketMatching(),
				highlightActiveLine(),
				highlightSelectionMatches(),
				syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
				markdown({ base: markdownLanguage, codeLanguages: languages }),
				autocompletion({
					override: [wikilinkCompletion, tagCompletion, slashCompletion],
					activateOnTyping: true,
					maxRenderedOptions: 20,
				}),
				editorKeymap,
				keymap.of([
					indentWithTab,
					...defaultKeymap,
					...historyKeymap,
					...searchKeymap,
					...foldKeymap,
					...closeBracketsKeymap,
				]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir })),
				cmPlaceholder(placeholder),
				obsidianTheme,
				clipboardImagePaste(),
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !updating) {
						onchange(update.state.doc.toString());
					}
				}),
			]
		});
		view = new EditorView({ state: startState, parent: containerEl });
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

<div class="cm-wrapper" bind:this={containerEl}></div>

<style>
	.cm-wrapper {
		flex: 1;
		overflow: hidden;
	}
	.cm-wrapper :global(.cm-editor) {
		height: 100%;
		outline: none;
	}
	.cm-wrapper :global(.cm-scroller) {
		overflow: auto;
	}
</style>

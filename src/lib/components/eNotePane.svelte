<script lang="ts">
	/**
	 * eNotePane — Phase 2: Save & Restore
	 * Gray desk + white paper + title + CM6 editor + persistence.
	 * Zero custom plugins. Typing must be instant.
	 * Spec: docs/eNotePane-spec.md, Section 4
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';

	const SAVE_DEBOUNCE = 1500; /* ms — spec Section 4.2 */

	let {
		value = '',
		title = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		initialCursorPos = 0,
		initialScrollTop = 0,
		onchange,
		onsave,
		onflush,
		ontitlechange,
		oncursorchange,
		onscrollchange,
	}: {
		value?: string;
		title?: string;
		dir?: 'ltr' | 'rtl';
		initialCursorPos?: number;
		initialScrollTop?: number;
		onchange?: (value: string) => void;
		onsave?: (value: string) => void;
		onflush?: (value: string) => void;
		ontitlechange?: (newTitle: string) => void;
		oncursorchange?: (pos: number) => void;
		onscrollchange?: (top: number) => void;
	} = $props();

	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	let latestText = value; /* non-reactive: tracks current content for saving */
	let dirty = false; /* true when text changed since last save */
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let rafHandle: number | null = null;
	const dirCompartment = new Compartment();

	/* ─── Mount: create editor, restore cursor/scroll, focus title ─── */
	onMount(() => {
		const state = EditorState.create({
			doc: value,
			extensions: [
				history(),
				drawSelection(),
				markdown({ base: markdownLanguage }), /* no codeLanguages — saves 500KB+ */
				keymap.of([...defaultKeymap, ...historyKeymap]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir: dir || 'auto' })),
				EditorView.contentAttributes.of({ dir: 'auto' }), /* browser auto-detects per paragraph */
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						const text = update.state.doc.toString();
						latestText = text;
						dirty = true;
						/* Immediate notify — parent tracks latest text */
						onchange?.(text);
						/* Debounced save — parent writes to disk after pause */
						if (saveTimer) clearTimeout(saveTimer);
						saveTimer = setTimeout(() => {
							saveTimer = null;
							dirty = false;
							onsave?.(latestText);
						}, SAVE_DEBOUNCE);
					}
				}),
				/* Minimal theme */
				EditorView.theme({
					'&': { background: 'transparent', border: 'none', outline: 'none' },
					'&.cm-focused': { outline: 'none' },
					'.cm-scroller': { overflow: 'auto', fontFamily: 'inherit', fontSize: '16px' /* base body text */, lineHeight: '1.75' /* comfortable reading rhythm */ },
					'.cm-content': { padding: '0', caretColor: 'var(--text-normal, #1a1a1a)' },
					'.cm-cursor': { borderLeftColor: 'var(--text-normal, #1a1a1a)', borderLeftWidth: '1.5px' /* visible cursor without being heavy */ },
					'.cm-line': { padding: '0' },
					'.cm-activeLine': { background: 'transparent' },
					'.cm-activeLineGutter': { display: 'none' },
					'.cm-gutters': { display: 'none' },
					'.cm-selectionBackground': { background: 'color-mix(in srgb, var(--interactive-accent, #7c3aed) 20%, transparent)' },
				}),
			],
		});

		view = new EditorView({ state, parent: editorEl! });

		/* Restore cursor position (spec 4.3) */
		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
		}
		/* Restore scroll position (spec 4.3) */
		if (initialScrollTop > 0) {
			rafHandle = requestAnimationFrame(() => {
				rafHandle = null;
				view?.scrollDOM.scrollTo({ top: initialScrollTop });
			});
		}

		titleEl?.focus();
	});

	/* ─── Destroy: flush save, save cursor/scroll, clean up ─── */
	onDestroy(() => {
		/* Cancel pending debounce and rAF (spec Rule 4) */
		if (saveTimer) clearTimeout(saveTimer);
		if (rafHandle !== null) { cancelAnimationFrame(rafHandle); rafHandle = null; }

		/* Flush unsaved content to parent only if dirty (spec 4.2) */
		if (dirty) {
			onflush?.(latestText);
		}

		/* Save cursor + scroll on destroy only (NOT during typing — spec 2.2) */
		if (view) {
			oncursorchange?.(view.state.selection.main.head);
			onscrollchange?.(view.scrollDOM.scrollTop);
		}

		view?.destroy();
		view = null;
	});

	/* ─── Dir sync: only fires when dir actually changes ─── */
	let prevDir = dir;
	$effect(() => {
		if (view && dir !== prevDir) {
			prevDir = dir;
			view.dispatch({ effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir })) });
		}
	});

	/* No $effect for value→editor sync. Editor owns content after mount. (spec 2.1) */
	/* Tab switches use {#key tab.id} to destroy/recreate with new value. */

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
		if (!trimmed) {
			titleValue = generateAutoTitle();
		}
		if (titleValue !== title) {
			ontitlechange?.(titleValue);
		}
	}

	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			view?.focus();
		}
	}

	const titleAlignment = $derived($appSettings.titleAlignment ?? 'center');

	/* ─── Exported methods for parent ─── */
	export function focus() { view?.focus(); }
	export function getText(): string { return view?.state.doc.toString() ?? ''; }
</script>

<div class="e-desk" dir={dir}>
	<div class="e-paper">
		<input
			class="e-title"
			class:e-title-center={titleAlignment === 'center'}
			bind:this={titleEl}
			bind:value={titleValue}
			dir="auto"
			placeholder={$t('eNotePane.titlePlaceholder')}
			spellcheck="false"
			onblur={handleTitleBlur}
			onkeydown={handleTitleKeydown}
		/>
		<div class="e-editor" bind:this={editorEl}></div>
	</div>
</div>

<style>
	/* ─── The Desk: gray surface behind the paper (spec 3.1) ─── */
	.e-desk {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		background: #e8e8ec; /* desk surface color */
		padding-inline: 24px; /* minimum 24px gray desk visible on each side */
		overflow-y: auto;
		overflow-x: hidden;
		min-width: 0;
		min-height: 0;
	}

	/* ─── The Paper: white writing surface (spec 3.1) ─── */
	.e-paper {
		width: 100%;
		max-width: 1200px; /* paper width from spec */
		flex: 1;
		display: flex;
		flex-direction: column;
		background: #ffffff; /* paper color */
		padding: 48px; /* paper padding from spec */
		min-width: 0;
		overflow-y: auto;
		overflow-x: hidden;
	}

	/* ─── Title: note identity (spec 0.3) ─── */
	.e-title {
		display: block;
		width: 100%;
		border: none;
		outline: none;
		background: transparent;
		font-size: 28px; /* title prominence — larger than body (16px) */
		font-weight: 700;
		font-family: inherit;
		color: var(--text-normal, #1a1a1a);
		padding: 0;
		margin-block: 0 24px; /* breathing room between title and editor */
		margin-inline: 0;
		text-align: start;
	}
	.e-title.e-title-center {
		text-align: center;
	}
	.e-title::placeholder {
		color: var(--text-faint, #ccc);
		font-weight: 400;
	}

	/* ─── Editor: fills remaining paper space ─── */
	.e-editor {
		flex: 1;
		min-height: 0;
	}
	.e-editor :global(.cm-editor) {
		height: 100%;
	}
	/* Per-line bidi: each line auto-detects its direction — zero JS cost */
	.e-editor :global(.cm-line) {
		unicode-bidi: plaintext;
	}
	.e-editor :global(.cm-editor),
	.e-editor :global(.cm-editor.cm-focused) {
		outline: none !important;
		border: none !important;
	}
</style>

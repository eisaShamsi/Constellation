<script lang="ts">
	/**
	 * eNotePane — Phase 2: Save & Restore
	 * Gray desk + white paper + title + CM6 editor + persistence.
	 * Zero custom plugins. Typing must be instant.
	 * Spec: docs/eNotePane-spec.md, Section 10 (Phase 2)
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';

	const IDLE_SAVE_INTERVAL = 30_000; /* ms — periodic background save when idle */

	let {
		value = '',
		title = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		initialCursorPos = 0,
		initialScrollTop = 0,
		onchange,
		onsave,
		onflush, /* (text, needsDiskSave, cursorPos, scrollTop) → parent updates store + WAB + disk */
		ontitlechange,
	}: {
		value?: string;
		title?: string;
		dir?: 'ltr' | 'rtl';
		initialCursorPos?: number;
		initialScrollTop?: number;
		onchange?: (value: string) => void;
		onsave?: (value: string) => void;
		onflush?: (text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number) => void;
		ontitlechange?: (newTitle: string) => void;
	} = $props();

	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	let latestText = value; /* non-reactive: tracks current content for saving */
	let dirty = false; /* true when text changed since last save */
	let idleSaveTimer: ReturnType<typeof setInterval> | null = null;
	let rafHandle: number | null = null;
	const dirCompartment = new Compartment();

	/* ─── Background save: periodic idle save + visibility change + beforeunload ─── */
	function doSave() {
		if (!dirty) return;
		dirty = false;
		onsave?.(latestText);
	}

	function doFlush() {
		const cursorPos = view ? view.state.selection.main.head : 0;
		const scrollTop = view ? view.scrollDOM.scrollTop : 0;
		onflush?.(latestText, dirty, cursorPos, scrollTop);
	}

	function handleVisibilityChange() {
		if (document.hidden && dirty) doSave();
	}

	function handleBeforeUnload() {
		/* Safety net: flush to WAB (localStorage) before app exit.
		   onDestroy may not fire reliably when Tauri window closes. */
		doFlush();
	}

	/* ─── Mount: create editor, restore cursor/scroll ─── */
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
						onchange?.(text);
						/* NO debounce timer — saves happen on:
						   1. Tab switch/close (onflush in onDestroy)
						   2. App losing focus (visibilitychange)
						   3. Periodic idle save (every 30s)
						   This keeps the typing path 100% free of IPC overhead. */
					}
				}),
				/* Minimal theme — spec 3.1 */
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

		/* Restore cursor + scroll OR focus title (spec 4.3) */
		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			/* Returning to a note — restore cursor in editor */
			view.dispatch({ selection: { anchor: initialCursorPos } });
			view.focus();
		} else {
			/* New note or no saved position — focus title */
			titleEl?.focus();
		}
		/* Scroll restore is independent of cursor (a note can be scrolled with cursor at 0).
		   Double-rAF: first frame lets CM6 measure + render, second frame scrolls safely. */
		if (initialScrollTop > 0) {
			rafHandle = requestAnimationFrame(() => {
				rafHandle = requestAnimationFrame(() => {
					rafHandle = null;
					view?.scrollDOM.scrollTo({ top: initialScrollTop });
				});
			});
		}

		/* Start periodic idle save + visibility listener + beforeunload safety net */
		idleSaveTimer = setInterval(() => {
			requestIdleCallback(() => doSave());
		}, IDLE_SAVE_INTERVAL);
		document.addEventListener('visibilitychange', handleVisibilityChange);
		window.addEventListener('beforeunload', handleBeforeUnload);
	});

	/* ─── Destroy: flush content + cursor/scroll, clean up ─── */
	onDestroy(() => {
		/* Cancel timers, remove listeners, cancel rAF (spec Rule 4) */
		if (idleSaveTimer) clearInterval(idleSaveTimer);
		document.removeEventListener('visibilitychange', handleVisibilityChange);
		window.removeEventListener('beforeunload', handleBeforeUnload);
		if (rafHandle !== null) { cancelAnimationFrame(rafHandle); rafHandle = null; }

		/* Single flush call with content + cursor + scroll.
		   Parent handles: store mutation + WAB (localStorage) + disk write. */
		doFlush();

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
		background: #e8e8ec;
		padding-inline: 24px;
		overflow-y: auto;
		overflow-x: hidden;
		min-width: 0;
		min-height: 0;
	}

	/* ─── The Paper: white writing surface (spec 3.1) ─── */
	.e-paper {
		width: 100%;
		max-width: 1200px;
		flex: 1;
		display: flex;
		flex-direction: column;
		background: #ffffff;
		padding: 48px;
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
		font-size: 28px;
		font-weight: 700;
		font-family: inherit;
		color: var(--text-normal, #1a1a1a);
		padding: 0;
		margin-block: 0 24px;
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

<script lang="ts">
	/**
	 * eNotePane — Phase 1: The Bare Editor
	 * Desk + paper + title + CM6 editor. Must be < 5ms latency.
	 * Spec: docs/eNotePane-spec.md, Section 3.3 Phase 1
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';

	let {
		value = '',
		title = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		onchange,
		ontitlechange,
	}: {
		value?: string;
		title?: string;
		dir?: 'ltr' | 'rtl';
		onchange?: (value: string) => void;
		ontitlechange?: (newTitle: string) => void;
	} = $props();

	/* ─── State ─── */
	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	const dirCompartment = new Compartment();

	/* ─── Mount: create editor + focus title ─── */
	onMount(() => {
		/* Focus title first — user types title, then Enter to body */
		titleEl?.focus();

		/* Create bare CM6 — spec Phase 1 extensions only */
		const state = EditorState.create({
			doc: value,
			extensions: [
				history(),
				drawSelection(),
				markdown({ base: markdownLanguage }), /* no codeLanguages — saves 500KB+ */
				keymap.of([
					...defaultKeymap,
					...historyKeymap,
				]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir })),
				EditorView.lineWrapping,
				EditorView.contentAttributes.of({ dir }),
				/* One-way: editor → parent. No debounce. Parent handles save. */
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						onchange?.(update.state.doc.toString());
					}
				}),
				/* Clean editor theme — no borders, no gutters, no highlights */
				EditorView.theme({
					'&': {
						background: 'transparent',
						fontSize: '16px', /* base body text size */
						fontFamily: 'inherit',
					},
					'&.cm-focused': {
						outline: 'none',
					},
					'.cm-scroller': {
						fontFamily: 'inherit',
					},
					'.cm-content': {
						caretColor: 'var(--text-normal, #1a1a1a)',
						padding: '0',
					},
					'.cm-cursor': {
						borderLeftColor: 'var(--text-normal, #1a1a1a)',
						borderLeftWidth: '1.5px', /* thin line cursor */
					},
					'.cm-line': {
						padding: '0',
					},
					'.cm-activeLine': {
						background: 'transparent', /* no active line highlight */
					},
					'.cm-gutters': {
						display: 'none', /* no gutters in eNotePane */
					},
					'.cm-selectionBackground': {
						background: 'color-mix(in srgb, var(--interactive-accent, #7c3aed) 20%, transparent)',
					},
				}),
			],
		});

		view = new EditorView({ state, parent: editorEl! });
	});

	/* ─── Destroy: clean up editor (MA requirement) ─── */
	onDestroy(() => {
		view?.destroy();
		view = null;
	});

	/* ─── Dir sync: only when dir prop changes (rare) ─── */
	let prevDir = dir;
	$effect(() => {
		if (view && dir !== prevDir) {
			prevDir = dir;
			view.dispatch({
				effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir })),
			});
		}
	});

	/* No $effect for value→editor sync. Editor owns content after mount. (spec 2.1, 2.6) */

	/* ─── Title ─── */

	/** Auto-generate title: CoNoteDDMMYYYY.HH:MM (spec Section 0.3) */
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
			/* Move focus to editor body */
			view?.focus();
		}
	}

	const titleAlignment = $derived($appSettings.titleAlignment ?? 'center');

	/** Expose focus method for parent to call */
	export function focus() {
		view?.focus();
	}
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

		<!-- CM6 Editor -->
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
		overflow: hidden;
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
		font-size: 28px; /* title prominence */
		font-weight: 700;
		font-family: inherit;
		color: var(--text-normal, #1a1a1a);
		padding: 0;
		margin-block: 0 24px; /* space below title before content */
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

	/* ─── Editor: CM6 container, fills remaining paper space ─── */
	.e-editor {
		flex: 1;
		min-height: 0;
	}
	/* Ensure CM6 fills the container */
	.e-editor :global(.cm-editor) {
		height: 100%;
	}
	.e-editor :global(.cm-scroller) {
		overflow: auto;
	}
</style>

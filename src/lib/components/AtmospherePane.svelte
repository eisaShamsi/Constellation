<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { languages } from '@codemirror/language-data';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
	import { livePreviewPlugin, livePreviewTheme } from '$lib/editor/livePreview';
	import { t } from '$lib/i18n';

	let {
		value = '',
		title = '',
		mode = 'blank-page' as 'blank-page' | 'typewriter' | 'manuscript' | 'flow',
		dir = 'ltr' as 'ltr' | 'rtl',
		onchange,
		ontitlechange,
		onaddproperty,
		onexit,
	}: {
		value: string;
		title?: string;
		mode?: 'blank-page' | 'typewriter' | 'manuscript' | 'flow';
		dir?: 'ltr' | 'rtl';
		onchange?: (value: string) => void;
		ontitlechange?: (title: string) => void;
		onaddproperty?: () => void;
		onexit?: () => void;
	} = $props();

	let editorEl: HTMLDivElement;
	let titleEl: HTMLInputElement;
	let view: EditorView | null = null;
	let updating = false;
	let wordCount = $state(0);
	let saveTimer: ReturnType<typeof setTimeout> | null = null;

	// Progressive disclosure states
	let hasContent = $state(value.trim().length > 0);
	let titleValue = $state(title);
	let titleFocused = $state(false);
	let hasTitleContent = $derived(titleValue.trim().length > 0);

	const dirCompartment = new Compartment();

	function getTheme(m: string) {
		const maxWidth = m === 'blank-page' ? '720px' : m === 'typewriter' ? '680px' : m === 'manuscript' ? '520px' : '100%';
		const lineHeight = m === 'blank-page' ? '2' : m === 'typewriter' ? '1.8' : m === 'manuscript' ? '2.2' : '1.9';
		const letterSpacing = m === 'manuscript' ? '0.015em' : 'normal';

		return EditorView.theme({
			'&': {
				background: 'transparent !important',
				border: 'none !important',
				fontSize: 'var(--font-text-size, 17px)',
			},
			'.cm-scroller': {
				overflow: 'auto',
				fontFamily: 'var(--font-text-theme, inherit)',
				paddingBottom: '40vh',
			},
			'.cm-content': {
				lineHeight,
				letterSpacing,
				caretColor: 'var(--interactive-accent, #7c3aed)',
				padding: '0',
				maxWidth,
				margin: '0 auto',
			},
			'.cm-cursor': {
				borderLeftColor: 'var(--interactive-accent, #7c3aed)',
				borderLeftWidth: '2px',
			},
			'.cm-line': {
				padding: '0 4px',
			},
			'.cm-activeLine': {
				background: m === 'typewriter' ? 'rgba(124, 58, 237, 0.03)' : 'transparent',
			},
			'.cm-gutters': {
				display: 'none !important',
			},
			'.cm-selectionBackground': {
				background: 'rgba(124, 58, 237, 0.15) !important',
			},
			'&.cm-focused .cm-selectionBackground': {
				background: 'rgba(124, 58, 237, 0.2) !important',
			},
		});
	}

	onMount(() => {
		const state = EditorState.create({
			doc: value,
			extensions: [
				getTheme(mode),
				history(),
				drawSelection(),
				syntaxHighlighting(defaultHighlightStyle),
				markdown({ base: markdownLanguage, codeLanguages: languages }),
				livePreviewPlugin,
				livePreviewTheme,
				keymap.of([
					...defaultKeymap,
					...historyKeymap,
					{ key: 'Escape', run: () => { onexit?.(); return true; } },
				]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir })),
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !updating) {
						const text = update.state.doc.toString();
						hasContent = text.trim().length > 0;
						const words = text.trim().split(/\s+/).filter(w => w.length > 0);
						wordCount = text.trim() ? words.length : 0;
						if (saveTimer) clearTimeout(saveTimer);
						saveTimer = setTimeout(() => onchange?.(text), 1500);
					}
				}),
			],
		});

		view = new EditorView({ state, parent: editorEl });

		// Initial state
		hasContent = value.trim().length > 0;
		const words = value.trim().split(/\s+/).filter(w => w.length > 0);
		wordCount = value.trim() ? words.length : 0;

		// Auto-focus the editor
		view.focus();
	});

	$effect(() => {
		if (view) {
			view.dispatch({ effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir })) });
		}
	});

	$effect(() => {
		if (view && value !== undefined) {
			const current = view.state.doc.toString();
			if (value !== current) {
				updating = true;
				view.dispatch({ changes: { from: 0, to: current.length, insert: value } });
				updating = false;
				hasContent = value.trim().length > 0;
			}
		}
	});

	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			view?.focus();
		}
		if (e.key === 'Escape') {
			onexit?.();
		}
	}

	function handleTitleBlur() {
		titleFocused = false;
		if (titleValue !== title) {
			ontitlechange?.(titleValue);
		}
	}

	onDestroy(() => {
		if (saveTimer) clearTimeout(saveTimer);
		if (view) {
			const text = view.state.doc.toString();
			onchange?.(text);
			view.destroy();
		}
	});
</script>

<div class="atm" class:rtl={dir === 'rtl'}>
	<!-- Title area — appears faintly when user starts writing -->
	<div class="atm-header" style="max-width: {mode === 'blank-page' ? '720px' : mode === 'typewriter' ? '680px' : mode === 'manuscript' ? '520px' : '100%'}">
		{#if hasContent || titleFocused || hasTitleContent}
			<input
				class="atm-title"
				class:ghost={!hasTitleContent && !titleFocused}
				class:visible={hasTitleContent || titleFocused}
				bind:this={titleEl}
				bind:value={titleValue}
				dir="auto"
				placeholder={$t('notePane.untitled') || 'Untitled'}
				spellcheck="false"
				onfocus={() => titleFocused = true}
				onblur={handleTitleBlur}
				onkeydown={handleTitleKeydown}
			/>
			<!-- (+) button for properties — appears when title has content -->
			{#if hasTitleContent}
				<button class="atm-add-props" onclick={onaddproperty} title={$t('contextMenu.addProperty') || 'Add property'}>
					+
				</button>
			{/if}
		{/if}
	</div>

	<!-- Editor — the blank page -->
	<div class="atm-editor" bind:this={editorEl}></div>

	<!-- Subtle status -->
	<div class="atm-status">
		{#if wordCount > 0}
			<span class="atm-words">{wordCount} {$t('atmosphere.wordCount')}</span>
		{/if}
		<span class="atm-hint">{$t('atmosphere.exitHint')}</span>
	</div>
</div>

<style>
	.atm {
		width: 100%;
		height: 100vh;
		display: flex;
		flex-direction: column;
		background: var(--background-primary, #ffffff);
		overflow: hidden;
	}
	.atm.rtl { direction: rtl; }

	/* ─── Title ─── */
	.atm-header {
		margin: 0 auto;
		width: 100%;
		padding: 60px 4px 0;
		flex-shrink: 0;
	}

	.atm-title {
		display: block;
		width: 100%;
		border: none;
		outline: none;
		background: transparent;
		font-family: var(--font-text-theme, inherit);
		font-size: calc(var(--font-text-size, 17px) * 1.6);
		font-weight: 700;
		color: var(--text-normal, #333);
		padding: 0 4px;
		transition: opacity 0.5s ease;
	}
	.atm-title::placeholder {
		color: var(--text-faint, #ddd);
		font-weight: 400;
	}
	.atm-title.ghost {
		opacity: 0.15;
	}
	.atm-title.visible {
		opacity: 1;
	}

	/* (+) Add properties button */
	.atm-add-props {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: 1px dashed var(--text-faint, #ddd);
		border-radius: 4px;
		background: transparent;
		color: var(--text-faint, #ccc);
		font-size: 16px;
		cursor: pointer;
		margin: 8px 4px;
		opacity: 0.4;
		transition: opacity 0.3s;
	}
	.atm-add-props:hover {
		opacity: 1;
		border-color: var(--interactive-accent, #7c3aed);
		color: var(--interactive-accent, #7c3aed);
	}

	/* ─── Editor ─── */
	.atm-editor {
		flex: 1;
		overflow: hidden;
		padding-top: 16px;
	}

	/* ─── Status bar ─── */
	.atm-status {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		display: flex;
		justify-content: center;
		align-items: center;
		gap: 24px;
		padding: 10px;
		pointer-events: none;
		opacity: 0.3;
		transition: opacity 0.3s;
	}
	.atm-status:hover {
		opacity: 0.6;
	}
	.atm-words, .atm-hint {
		font-size: 12px;
		color: var(--text-muted, #999);
		font-family: var(--font-interface-theme, sans-serif);
	}
</style>

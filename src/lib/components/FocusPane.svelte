<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState } from '@codemirror/state';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { appSettings, getEffectiveScriptFonts } from '$lib/libraries/store';
	import { bidiPlugin, bidiTheme, scriptFontsField, setScriptFonts } from '$lib/editor/bidiPlugin';
	import { t, tn } from '$lib/i18n';

	let {
		value = '',
		title = '',
		mode = 'blank-page' as 'blank-page' | 'typewriter' | 'manuscript' | 'flow',
		dir = 'ltr' as 'ltr' | 'rtl',
		onchange,
		ontitlechange,
		onexit,
		onflush,
	}: {
		value: string;
		title?: string;
		mode?: 'blank-page' | 'typewriter' | 'manuscript' | 'flow';
		dir?: 'ltr' | 'rtl';
		onchange?: (value: string) => void;
		ontitlechange?: (title: string) => void;
		onexit?: () => void;
		// Safety Audit G1 — fired on destroy/exit for an IMMEDIATE (non-debounced)
		// persist of the FINAL buffer, so a fast exit/tab-switch never loses the last
		// edit that the debounced onchange had not yet written.
		onflush?: (value: string) => void;
	} = $props();

	let editorEl: HTMLDivElement;
	let view: EditorView | null = null;
	let lastInternalValue = value;
	let wordCount = $state(0);
	let saveTimer: ReturnType<typeof setTimeout> | null = null;

	// ─── Progressive disclosure state ───
	let isTyping = $state(false);
	let pauseTimer: ReturnType<typeof setTimeout> | null = null;
	let showTitle = $state(false);
	let titleEditing = $state(false);
	let titleValue = $state(title);
	let hasTitleContent = $derived(titleValue.trim().length > 0);

	const PAUSE_DELAY = 3000; // 3 seconds of no typing → show title

	function onUserTyping() {
		isTyping = true;
		// Once title has content, it stays visible forever
		if (hasTitleContent) {
			showTitle = true;
		}
		if (pauseTimer) clearTimeout(pauseTimer);
		pauseTimer = setTimeout(() => {
			isTyping = false;
			if (wordCount > 0) {
				showTitle = true;
			}
		}, PAUSE_DELAY);
	}

	function handleTitleFocus() {
		titleEditing = true;
		showTitle = true;
	}

	function handleTitleBlur() {
		titleEditing = false;
		if (titleValue !== title) {
			ontitlechange?.(titleValue);
		}
		// If title is empty and user isn't typing, keep showing
		// If title has content, keep showing
	}

	function handleTitleInput() {
		// Title content changed
	}

	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			view?.focus();
		}
		if (e.key === 'Escape') {
			onexit?.();
		}
	}


	function getTheme(m: string) {
		const lineHeight = m === 'blank-page' ? '2' : m === 'typewriter' ? '1.8' : m === 'manuscript' ? '2.2' : '1.9';
		const letterSpacing = m === 'manuscript' ? '0.015em' : 'normal';

		return EditorView.theme({
			'&': {
				background: 'transparent !important',
				border: 'none !important',
				outline: 'none !important',
				fontSize: 'var(--font-text-size, 17px)',
			},
			'&.cm-focused': {
				outline: 'none !important',
				border: 'none !important',
			},
			'.cm-scroller': {
				overflow: 'auto',
				fontFamily: 'var(--font-text-theme, inherit)',
				paddingInline: '48px',
			},
			'.cm-content': {
				lineHeight,
				letterSpacing,
				caretColor: 'var(--interactive-accent, #7c3aed)',
				padding: '0',
				border: 'none !important',
				outline: 'none !important',
			},
			'.cm-cursor': {
				borderLeftColor: 'var(--text-normal, #333)',
				borderLeftWidth: '1.5px',
			},
			'.cm-line': {
				padding: '0 4px',
				border: 'none !important',
			},
			'.cm-activeLine': {
				background: m === 'typewriter' ? 'rgba(124, 58, 237, 0.03)' : 'transparent !important',
				borderLeft: 'none !important',
				borderRight: 'none !important',
			},
			'.cm-activeLineGutter': {
				display: 'none !important',
			},
			'.cm-gutters': {
				display: 'none !important',
			},
			'.cm-foldGutter': {
				display: 'none !important',
			},
			'.cm-selectionLayer': {
				border: 'none !important',
			},
			'.cm-content *': {
				borderTop: 'none !important',
				borderBottom: 'none !important',
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
				keymap.of([
					...defaultKeymap,
					...historyKeymap,
					{ key: 'Escape', run: () => { onexit?.(); return true; } },
				]),
				EditorView.editorAttributes.of({ dir: 'auto' }),
				scriptFontsField,
				bidiPlugin,
				bidiTheme,
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						const text = update.state.doc.toString();
						lastInternalValue = text;
						const words = text.trim().split(/\s+/).filter(w => w.length > 0);
						wordCount = text.trim() ? words.length : 0;
						onUserTyping();
						// Fire onchange immediately — parent should NOT feed value back
						onchange?.(text);
					}
				}),
			],
		});

		view = new EditorView({ state, parent: editorEl });

		// Initial state
		const words = value.trim().split(/\s+/).filter(w => w.length > 0);
		wordCount = value.trim() ? words.length : 0;

		// If opening an existing note with content, show title
		if (wordCount > 0 || hasTitleContent) {
			showTitle = true;
		}

		view.focus();
	});

	// Script fonts — dispatch to bidiPlugin whenever appSettings changes (typewriter preset aware)
	$effect(() => {
		const sf = getEffectiveScriptFonts($appSettings);
		if (view) {
			view.dispatch({ effects: setScriptFonts.of(sf) });
		}
	});

	// No $effect for value→editor sync. Editor owns its content after mount.
	// Tab switches destroy/recreate FocusPane with new value prop.

	onDestroy(() => {
		if (saveTimer) clearTimeout(saveTimer);
		if (pauseTimer) clearTimeout(pauseTimer);
		if (view) {
			const text = view.state.doc.toString();
			// Safety Audit G1 — flush the final buffer IMMEDIATELY on teardown (tab/mode
			// switch, close). onchange only debounces; onflush persists now so the last
			// keystrokes before a fast exit are never lost.
			onflush?.(text);
			view.destroy();
		}
	});
</script>

<div class="focus-pane" class:rtl={dir === 'rtl'}>
	<div class="focus-paper">
		<!-- Title — appears faintly when user pauses writing -->
		<div class="focus-title-area">
			{#if showTitle || titleEditing}
				<input
					class="focus-title"
					class:ghost={!titleEditing && !hasTitleContent}
					class:editing={titleEditing}
					bind:value={titleValue}
					dir="auto"
					style="text-align: {$appSettings.titleAlignment === 'center' ? 'center' : 'start'}"
					placeholder={dir === 'rtl' ? 'العنوان' : 'Title'}
					spellcheck="false"
					onfocus={handleTitleFocus}
					onblur={handleTitleBlur}
					oninput={handleTitleInput}
					onkeydown={handleTitleKeydown}
				/>
				{/if}
		</div>

		<!-- The blank page — editor -->
		<div class="focus-editor" bind:this={editorEl}></div>
	</div>

	<!-- Word count -->
	<div class="focus-footer">
		{#if wordCount > 0}
			<span>{$tn('plurals.words', wordCount)}</span>
		{/if}
	</div>
</div>

<style>
	/* The table — gray surface filling the entire window */
	.focus-pane {
		position: fixed;
		inset: 0;
		z-index: 100;
		display: flex;
		justify-content: center;
		background: #e8e8ec;
		overflow: hidden;
		padding: 24px 32px 0 32px;
	}
	.focus-pane.rtl { direction: rtl; }

	/* The paper — white sheet centered on the table */
	.focus-paper {
		width: 100%;
		max-width: 1200px;
		height: 100%;
		display: flex;
		flex-direction: column;
		background: #ffffff;
		border-radius: 6px 6px 0 0;
		box-shadow:
			-2px 0 8px rgba(0,0,0,0.04),
			2px 0 8px rgba(0,0,0,0.04),
			0 -2px 8px rgba(0,0,0,0.03);
		overflow-y: auto;
		overflow-x: hidden;
	}

	/* ─── Title area ─── */
	.focus-title-area {
		width: 100%;
		padding-top: 48px;
		padding-inline: 48px;
		flex-shrink: 0;
		min-height: 0;
	}

	.focus-title {
		display: block;
		width: 100%;
		border: none;
		outline: none;
		background: transparent;
		font-family: var(--font-text-theme, inherit);
		font-size: calc(var(--font-text-size, 17px) * 1.5);
		font-weight: 700;
		color: var(--text-normal, #333);
		padding: 0 4px;
		transition: opacity 0.8s ease;
	}
	.focus-title::placeholder {
		color: var(--text-faint, #ddd);
		font-weight: 300;
	}
	/* Ghost: barely visible, like fog */
	.focus-title.ghost {
		opacity: 0.08;
		transition: opacity 1.2s ease;
	}
	.focus-title.ghost:hover {
		opacity: 0.25;
		transition: opacity 0.3s ease;
	}
	.focus-title.editing {
		opacity: 1;
		transition: opacity 0.3s ease;
	}

	/* (+) Properties button */
	/* ─── Editor ─── */
	.focus-editor {
		flex: 1;
		overflow: auto;
		padding-top: 20px;
		min-height: 0;
	}
	/* Kill lines, borders, outlines — but NOT the cursor */
	.focus-editor :global(.cm-editor) {
		border: none !important;
		outline: none !important;
		box-shadow: none !important;
	}
	.focus-editor :global(.cm-editor:focus),
	.focus-editor :global(.cm-editor.cm-focused) {
		outline: none !important;
		border: none !important;
	}
	.focus-editor :global(.cm-content) {
		border: none !important;
		outline: none !important;
	}
	.focus-editor :global(.cm-activeLine) {
		background: transparent !important;
	}
	.focus-editor :global(.cm-line) {
		border-top: none !important;
		border-bottom: none !important;
	}
	.focus-editor :global(.cm-scroller) {
		border: none !important;
		outline: none !important;
		overflow: auto !important;
	}
	/* Plain hairline cursor */
	.focus-editor :global(.cm-cursor) {
		border-left: 1.5px solid var(--text-normal, #1a1a1a) !important;
	}

	/* ─── Footer ─── */
	.focus-footer {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		text-align: center;
		padding: 8px;
		pointer-events: none;
	}
	.focus-footer span {
		font-size: 12px;
		color: var(--text-muted, #888);
		opacity: 0.6;
		font-family: var(--font-interface-theme, sans-serif);
	}
	.focus-footer { pointer-events: auto; display: flex; justify-content: center; gap: 16px; align-items: center; }
</style>

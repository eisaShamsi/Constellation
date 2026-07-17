<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import CascadeFreezeOverlay from '$lib/components/CascadeFreezeOverlay.svelte'; // sweep #3 — rename-cascade freeze
	import { appSettings, getEffectiveScriptFonts } from '$lib/libraries/store';
	import { bidiPlugin, bidiTheme, scriptFontsField, setScriptFonts } from '$lib/editor/bidiPlugin';
	import { RTL_MOTION_ENABLED } from '$lib/editor/rtlFlag'; // PJ-106 §A1
	import { detectDir } from '$lib/utils'; // PJ-106 §A1 — deterministic Focus base direction
	import { tripleClickTextOnly } from '$lib/editor/tripleClickLine'; // PJ-106 §B0
	import { logicalArrowKeymap } from '$lib/editor/rtlMotion'; // PJ-106 §A5
	import { paragraphNavKeymap, selectUnitKeymap } from '$lib/editor/paragraphNav'; // PJ-106 §B1/§B2
	import { ctrlClickSentence, sentenceSelectKeymap } from '$lib/editor/sentenceSelect'; // PJ-106 §B3
	import { paragraphDirKeys } from '$lib/editor/paragraphDir'; // PJ-106 §B4
	import { t, tn } from '$lib/i18n';

	let {
		value = '',
		title = '',
		mode = 'blank-page' as 'blank-page' | 'typewriter' | 'manuscript' | 'flow',
		dir = 'ltr' as 'ltr' | 'rtl',
		frozen = false,
		onchange,
		ontitlechange,
		onexit,
		onflush,
	}: {
		value: string;
		title?: string;
		mode?: 'blank-page' | 'typewriter' | 'manuscript' | 'flow';
		dir?: 'ltr' | 'rtl';
		// Sweep-2026-07-18 #3 (APP-KILLER) — TRUE while the note open in Focus is inside a
		// rename+wikilink cascade. FocusPane was the one editable surface OUTSIDE the cascade
		// freeze: the user could keep typing during the ~7s walk with no signal, and the post-
		// cascade reloadTabsFromDisk force-adopt then silently discarded those keystrokes (and an
		// armed commitFocusSave could revert the walker's rewrite). While frozen the editor goes
		// HARD read-only (keystrokes can't dirty the model → the force-adopt is always safe) and
		// shows the same "Updating…" overlay NotePane gets. Belt: commitFocusSave is isCascading-
		// gated in +layout. Mirrors NotePane's handleSave/handleFlush cascade gate + overlay.
		frozen?: boolean;
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
	// Sweep-2026-07-18 #3 — the hard read-only gate driven by `frozen` (rename-cascade freeze).
	// readOnly blocks edit transactions; editable:false also drops contentEditable so a focused
	// editor takes no keystrokes at all during the cascade window.
	const editGate = new Compartment();
	const frozenExt = (f: boolean) => (f ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : []);
	let wordCount = $state(0);
	let idleSaveTimer: ReturnType<typeof setInterval> | null = null;
	let lastFlushedText = value;
	const IDLE_SAVE_INTERVAL = 30000; // 30 s crash-safety belt during gap-free typing (mirrors NotePane)
	// APP-KILLER #2 (safety sweep, 2026-07-08) — durability on window/webview TEARDOWN.
	// Svelte onDestroy does NOT run on a full window unload (only beforeunload / visibility-
	// change fire), and Focus's only other persistence is a pause-only 1500 ms debounce in the
	// layout — so typing without a >1.5 s gap and then closing the window lost the whole run.
	// Mirror NotePane: flush on beforeunload + tab-hide + a 30 s idle tick. flush → onflush →
	// commitFocusSave sets the SYNCHRONOUS write-ahead net (recoverable on reopen even if the
	// async disk write is cut off by the unload).
	function flushNow() {
		if (!view) return;
		lastFlushedText = view.state.doc.toString();
		onflush?.(lastFlushedText);
	}
	function idleFlush() {
		if (view && view.state.doc.toString() !== lastFlushedText) flushNow();
	}
	function handleVisibilityChange() { if (document.hidden) flushNow(); }

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
				editGate.of(frozenExt(frozen)), // sweep #3 — hard read-only while the cascade freeze is up
				getTheme(mode),
				history(),
				drawSelection(),
				keymap.of([
					...defaultKeymap,
					...historyKeymap,
					{ key: 'Escape', run: () => { onexit?.(); return true; } },
				]),
				/* PJ-106 §A1 (SI2-2 parity) — deterministic base from the note's content (not the
				   viewport-first-strong 'auto'); both editor + content attrs so the base governs
				   the empty-line caret side. (The `dir` prop drives the pane CHROME; the editor
				   base deliberately derives from content — audit-corrected comment.) */
				EditorView.editorAttributes.of({ dir: detectDir(value) }),
				EditorView.contentAttributes.of({ dir: detectDir(value) }),
				scriptFontsField,
				bidiPlugin,
				bidiTheme,
				/* PJ-106 §A1 — connect per-line direction to the caret/selection MOTION engine. */
				...(RTL_MOTION_ENABLED ? [EditorView.perLineTextDirection.of(true)] : []),
				tripleClickTextOnly,
				/* PJ-106 §A5 — Word-style LOGICAL arrows. No skip source: Focus is parser-free and
				   has no collapsed widgets (Rule 6 — this must not pull livePreview into Focus). */
				...(RTL_MOTION_ENABLED ? [logicalArrowKeymap()] : []),
				/* PJ-106 §B1 — Ctrl+↑/↓ paragraph navigation (direction-blind, parser-free). */
				...(RTL_MOTION_ENABLED ? [paragraphNavKeymap()] : []),
				/* PJ-106 §B2 — Ctrl+L select line / Ctrl+Shift+L select paragraph block. */
				...(RTL_MOTION_ENABLED ? [selectUnitKeymap()] : []),
				/* PJ-106 §B3 — Ctrl+click / Ctrl+Shift+S select the sentence (Intl.Segmenter). */
				...(RTL_MOTION_ENABLED ? [ctrlClickSentence, sentenceSelectKeymap()] : []),
				/* PJ-106 §B4 — Right/Left-Ctrl+Shift paragraph direction (parser-free). */
				...(RTL_MOTION_ENABLED ? [paragraphDirKeys()] : []),
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

		// Durability triggers that survive a window unload (see flushNow above).
		window.addEventListener('beforeunload', flushNow);
		document.addEventListener('visibilitychange', handleVisibilityChange);
		idleSaveTimer = setInterval(idleFlush, IDLE_SAVE_INTERVAL);
	});

	// Script fonts — dispatch to bidiPlugin whenever appSettings changes (typewriter preset aware)
	$effect(() => {
		const sf = getEffectiveScriptFonts($appSettings);
		if (view) {
			view.dispatch({ effects: setScriptFonts.of(sf) });
		}
	});

	// Sweep-2026-07-18 #3 — reconfigure the hard read-only gate whenever the cascade freeze
	// toggles, so the editor stops taking keystrokes the instant the rename cascade raises the
	// freeze and becomes editable again the instant it lifts (or on the focusReseed remount).
	$effect(() => {
		if (view) view.dispatch({ effects: editGate.reconfigure(frozenExt(frozen)) });
	});

	// No $effect for value→editor sync. Editor owns its content after mount.
	// Tab switches destroy/recreate FocusPane with new value prop.

	onDestroy(() => {
		if (pauseTimer) clearTimeout(pauseTimer);
		if (idleSaveTimer) clearInterval(idleSaveTimer);
		window.removeEventListener('beforeunload', flushNow);
		document.removeEventListener('visibilitychange', handleVisibilityChange);
		// Safety Audit G1 — flush the final buffer IMMEDIATELY on teardown (tab/mode switch,
		// close). onchange only debounces; flushNow reads the live doc + persists via onflush
		// so the last keystrokes before a fast exit are never lost. (Single-sourced flush.)
		flushNow();
		if (view) view.destroy();
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

	<!-- Sweep-2026-07-18 #3 — the rename-cascade "Updating…" freeze, reusing the shared overlay
	     (controlled `frozen` mode). .focus-pane is position:fixed, so the absolute overlay fills it. -->
	<CascadeFreezeOverlay {frozen} />
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

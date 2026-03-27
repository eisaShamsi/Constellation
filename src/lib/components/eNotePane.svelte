<script lang="ts">
	/**
	 * eNotePane — Phase 3: Breadcrumb & Properties
	 * Gray desk + breadcrumb + white paper + title + properties + CM6 editor + persistence.
	 * Zero custom plugins. Typing must be instant.
	 * Spec: docs/eNotePane-spec.md, Section 10 (Phase 3)
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import type { FrontmatterProperty } from '$lib/libraries/store';
	import PropertyEditor from './PropertyEditor.svelte';
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
		/* Phase 3: breadcrumb + properties */
		libraryName = '',
		tabId = '',
		filePath = '',
		properties = [] as FrontmatterProperty[],
		rawYaml = '',
		canGoBack = false,
		canGoForward = false,
		saving = false,
		/* Callbacks */
		onchange,
		onsave,
		onflush,
		ontitlechange,
		onnavigateback,
		onnavigateforward,
		onmoreaction,
		onpropschange,
	}: {
		value?: string;
		title?: string;
		dir?: 'ltr' | 'rtl';
		initialCursorPos?: number;
		initialScrollTop?: number;
		libraryName?: string;
		tabId?: string;
		filePath?: string;
		properties?: FrontmatterProperty[];
		rawYaml?: string;
		canGoBack?: boolean;
		canGoForward?: boolean;
		saving?: boolean;
		onchange?: (value: string) => void;
		onsave?: (value: string) => void;
		onflush?: (text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number) => void;
		ontitlechange?: (newTitle: string) => void;
		onnavigateback?: () => void;
		onnavigateforward?: () => void;
		onmoreaction?: (action: string) => void;
		onpropschange?: () => void;
	} = $props();

	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	let latestText = value;
	let dirty = false;
	let idleSaveTimer: ReturnType<typeof setInterval> | null = null;
	let rafHandle: number | null = null;
	const dirCompartment = new Compartment();

	/* ─── Phase 3 state ─── */
	let propsCollapsed = $state(false);
	let showMoreMenu = $state(false);
	let moreMenuEl: HTMLDivElement | undefined;
	const hasHistory = $derived(canGoBack || canGoForward);
	const propsMode = $derived($appSettings.propertiesInDocument ?? 'visible');

	/* ─── More menu ─── */
	function toggleMoreMenu() {
		showMoreMenu = !showMoreMenu;
		if (showMoreMenu) {
			setTimeout(() => window.addEventListener('click', closeMoreMenu, { once: true }), 0);
		}
	}
	function closeMoreMenu() { showMoreMenu = false; }
	function handleMoreAction(action: string) {
		showMoreMenu = false;
		onmoreaction?.(action);
	}

	/* ─── Background save ─── */
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
	function handleVisibilityChange() { if (document.hidden && dirty) doSave(); }
	function handleBeforeUnload() { doFlush(); }

	/* ─── Mount ─── */
	onMount(() => {
		const state = EditorState.create({
			doc: value,
			extensions: [
				history(),
				drawSelection(),
				markdown({ base: markdownLanguage }),
				keymap.of([...defaultKeymap, ...historyKeymap]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir: dir || 'auto' })),
				EditorView.contentAttributes.of({ dir: 'auto' }),
				EditorView.lineWrapping,
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						const text = update.state.doc.toString();
						latestText = text;
						dirty = true;
						onchange?.(text);
					}
				}),
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

		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
			view.focus();
		} else {
			titleEl?.focus();
		}
		if (initialScrollTop > 0) {
			rafHandle = requestAnimationFrame(() => {
				rafHandle = requestAnimationFrame(() => {
					rafHandle = null;
					view?.scrollDOM.scrollTo({ top: initialScrollTop });
				});
			});
		}

		idleSaveTimer = setInterval(() => { requestIdleCallback(() => doSave()); }, IDLE_SAVE_INTERVAL);
		document.addEventListener('visibilitychange', handleVisibilityChange);
		window.addEventListener('beforeunload', handleBeforeUnload);
	});

	/* ─── Destroy ─── */
	onDestroy(() => {
		if (idleSaveTimer) clearInterval(idleSaveTimer);
		document.removeEventListener('visibilitychange', handleVisibilityChange);
		window.removeEventListener('beforeunload', handleBeforeUnload);
		if (rafHandle !== null) { cancelAnimationFrame(rafHandle); rafHandle = null; }
		doFlush();
		view?.destroy();
		view = null;
	});

	/* ─── Dir sync ─── */
	let prevDir = dir;
	$effect(() => {
		if (view && dir !== prevDir) {
			prevDir = dir;
			view.dispatch({ effects: dirCompartment.reconfigure(EditorView.editorAttributes.of({ dir })) });
		}
	});

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
		if (!trimmed) titleValue = generateAutoTitle();
		if (titleValue !== title) ontitlechange?.(titleValue);
	}
	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') { e.preventDefault(); view?.focus(); }
	}

	const titleAlignment = $derived($appSettings.titleAlignment ?? 'center');
</script>

<div class="e-desk" dir={dir}>
	<!-- ─── Breadcrumb ─── -->
	<div class="e-breadcrumb">
		{#if hasHistory}
			<button class="e-bc-nav" onclick={() => onnavigateback?.()} disabled={!canGoBack} title={$t('eNotePane.back')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
			</button>
			<button class="e-bc-nav" onclick={() => onnavigateforward?.()} disabled={!canGoForward} title={$t('eNotePane.forward')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
			</button>
		{/if}
		<span class="e-bc-lib">{libraryName}</span>
		<span class="e-bc-sep">/</span>
		<span class="e-bc-note">{title}</span>
		<div class="e-bc-actions">
			{#if saving}<span class="e-bc-saving">{$t('eNotePane.saving')}</span>{/if}
			<div class="e-bc-more-wrap" bind:this={moreMenuEl}>
				<button class="e-bc-dots" onclick={toggleMoreMenu} title={$t('eNotePane.moreOptions')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="12" cy="19" r="2"/></svg>
				</button>
				{#if showMoreMenu}
					<div class="e-bc-menu">
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('addProperty')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/></svg>
							{$t('contextMenu.addProperty')}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('rename')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
							{$t('contextMenu.rename')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('revealInTree')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9z"/><path d="M9 22V12h6v10"/></svg>
							{$t('contextMenu.revealInTree')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('showInExplorer')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2v11z"/></svg>
							{$t('contextMenu.showInExplorer')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('openDefaultApp')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><path d="M15 3h6v6"/><path d="M10 14L21 3"/></svg>
							{$t('contextMenu.openDefaultApp')}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('copyPath')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
							{$t('contextMenu.copyPath')}
						</button>
						<button class="e-bc-menu-item" onclick={() => handleMoreAction('copyName')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1"/></svg>
							{$t('contextMenu.copyName')}
						</button>
						<div class="e-bc-menu-sep"></div>
						<button class="e-bc-menu-item e-bc-menu-danger" onclick={() => handleMoreAction('delete')}>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
							{$t('contextMenu.deleteFile')}
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- ─── Paper ─── -->
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

		<!-- ─── Properties ─── -->
		{#if propsMode !== 'hidden' && (properties.length > 0 || rawYaml)}
			{#if propsMode === 'source'}
				<button class="e-props-toggle" onclick={() => propsCollapsed = !propsCollapsed}>
					<svg class="e-props-chevron" class:collapsed={propsCollapsed} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
					<span>{$t('eNotePane.properties')}</span>
				</button>
				{#if !propsCollapsed}
					<pre class="e-props-source">{rawYaml}</pre>
				{/if}
			{:else}
				<PropertyEditor
					{properties}
					body={value}
					{tabId}
					{filePath}
					{libraryName}
					noteDir={dir}
					collapsed={propsCollapsed}
					onToggle={() => propsCollapsed = !propsCollapsed}
				/>
			{/if}
			<hr class="e-props-divider" />
		{/if}

		<div class="e-editor" bind:this={editorEl}></div>
	</div>
</div>

<style>
	/* ─── The Desk (spec 3.1) ─── */
	.e-desk {
		flex: 1; display: flex; flex-direction: column; align-items: center;
		background: #e8e8ec; padding-inline: 24px;
		overflow-y: auto; overflow-x: hidden; min-width: 0; min-height: 0;
	}

	/* ─── Breadcrumb (above paper) ─── */
	.e-breadcrumb {
		padding: 4px 16px; font-size: 0.78rem; color: var(--text-faint);
		display: flex; align-items: center; min-height: 28px; flex-shrink: 0;
		width: 100%; max-width: 1200px; background: #ffffff;
		border-bottom: 1px solid var(--background-modifier-border, #e0e0e0);
	}
	.e-bc-lib { color: var(--text-muted); }
	.e-bc-sep { margin: 0 4px; color: var(--background-modifier-border-focus); }
	.e-bc-note { color: var(--text-normal); }
	.e-bc-actions { margin-inline-start: auto; display: flex; align-items: center; gap: 4px; position: relative; }
	.e-bc-saving { font-size: 0.7rem; color: var(--interactive-accent); }
	.e-bc-nav {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer; flex-shrink: 0;
	}
	.e-bc-nav:hover:not(:disabled) { background: var(--background-modifier-hover); color: var(--text-normal); }
	.e-bc-nav:disabled { opacity: 0.3; cursor: default; }
	:global([dir="rtl"]) .e-bc-nav svg { transform: scaleX(-1); }
	.e-bc-dots {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer;
	}
	.e-bc-dots:hover { background: var(--background-modifier-border); color: var(--text-normal); }
	.e-bc-more-wrap { position: relative; }
	.e-bc-menu {
		position: absolute; top: 100%; right: 0; z-index: 100;
		background: var(--background-primary); border: 1px solid var(--background-modifier-border);
		border-radius: 8px; padding: 4px 0; min-width: 220px; max-height: 80vh; overflow-y: auto;
		box-shadow: 0 4px 16px rgba(0,0,0,0.15); direction: ltr;
	}
	:global([dir="rtl"]) .e-bc-menu { right: auto; left: 0; direction: rtl; }
	.e-bc-menu-item {
		display: flex; align-items: center; gap: 10px; width: 100%; padding: 7px 14px;
		border: none; background: none; cursor: pointer; font-size: 13px;
		color: var(--text-normal); text-align: start; font-family: var(--font-interface-theme);
	}
	.e-bc-menu-item:hover { background: var(--background-modifier-hover); }
	.e-bc-menu-item svg { flex-shrink: 0; opacity: 0.6; }
	.e-bc-menu-danger { color: var(--text-error, #e53935); }
	.e-bc-menu-danger:hover { background: color-mix(in srgb, var(--text-error, #e53935) 10%, transparent); }
	.e-bc-menu-sep { height: 1px; margin: 4px 10px; background: var(--background-modifier-border); }

	/* ─── The Paper (spec 3.1) ─── */
	.e-paper {
		width: 100%; max-width: 1200px; flex: 1;
		display: flex; flex-direction: column; background: #ffffff;
		padding: 48px; min-width: 0; overflow-y: auto; overflow-x: hidden;
	}

	/* ─── Title (spec 0.3) ─── */
	.e-title {
		display: block; width: 100%; border: none; outline: none; background: transparent;
		font-size: 28px; font-weight: 700; font-family: inherit;
		color: var(--text-normal, #1a1a1a); padding: 0;
		margin-block: 0 24px; margin-inline: 0; text-align: start;
	}
	.e-title.e-title-center { text-align: center; }
	.e-title::placeholder { color: var(--text-faint, #ccc); font-weight: 400; }

	/* ─── Properties toggle + source view ─── */
	.e-props-toggle {
		display: flex; align-items: center; gap: 6px;
		border: none; background: none; cursor: pointer; padding: 4px 0;
		font-size: 0.8rem; color: var(--text-muted); font-family: inherit;
	}
	.e-props-toggle:hover { color: var(--text-normal); }
	.e-props-chevron { transition: transform 0.2s; flex-shrink: 0; }
	.e-props-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .e-props-chevron.collapsed { transform: rotate(90deg); }
	.e-props-source {
		font-size: 0.8rem; color: var(--text-muted); background: var(--background-secondary, #f5f5f5);
		padding: 8px 12px; border-radius: 4px; margin: 4px 0 0; overflow-x: auto;
		white-space: pre-wrap; font-family: var(--font-monospace-theme, monospace);
	}
	.e-props-divider {
		border: none; border-top: 1px solid var(--background-modifier-border, #e8e8e8);
		margin: 12px 0;
	}

	/* ─── Editor ─── */
	.e-editor { flex: 1; min-height: 0; }
	.e-editor :global(.cm-editor) { height: 100%; }
	.e-editor :global(.cm-line) { unicode-bidi: plaintext; }
	.e-editor :global(.cm-editor),
	.e-editor :global(.cm-editor.cm-focused) { outline: none !important; border: none !important; }
</style>

<script lang="ts">
	/**
	 * eNotePane — Phase 4: Toolbar
	 * Desk + paper + breadcrumb + title + properties + toolbar + CM6 editor + save.
	 * Spec: docs/eNotePane-spec.md, Sections 3.2, 3.6, 0.3.1
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { defaultKeymap, history, historyKeymap, undo, redo } from '@codemirror/commands';

	interface NoteProperty {
		key: string;
		value: string;
		type?: string;
	}

	let {
		value = '',
		title = '',
		dir = 'ltr' as 'ltr' | 'rtl',
		libraryName = '',
		breadcrumbPath = '',
		properties = [] as NoteProperty[],
		initialCursorPos = 0,
		initialScrollTop = 0,
		onchange,
		ontitlechange,
		onpropertieschange,
		oncursorchange,
		onscrollchange,
		onsave,
		onflush,
		onnavigateback,
		onnavigateforward,
		onmoreoptions,
	}: {
		value?: string;
		title?: string;
		dir?: 'ltr' | 'rtl';
		libraryName?: string;
		breadcrumbPath?: string;
		properties?: NoteProperty[];
		initialCursorPos?: number;
		initialScrollTop?: number;
		onchange?: (value: string) => void;
		ontitlechange?: (newTitle: string) => void;
		onpropertieschange?: (props: NoteProperty[]) => void;
		oncursorchange?: (pos: number) => void;
		onscrollchange?: (top: number) => void;
		onsave?: (text: string) => void;
		onflush?: (text: string) => void;
		onnavigateback?: () => void;
		onnavigateforward?: () => void;
		onmoreoptions?: () => void;
	} = $props();

	/* ─── State ─── */
	let titleValue = $state(title);
	let titleEl: HTMLInputElement | undefined;
	let editorEl: HTMLDivElement | undefined;
	let view: EditorView | null = null;
	let latestText = value;                /* non-reactive: tracks content for saving */
	let saveTimer: ReturnType<typeof setTimeout> | null = null;
	let propsCollapsed = $state(true);     /* properties start collapsed */
	const dirCompartment = new Compartment();
	const SAVE_DEBOUNCE = 1500;            /* ms — spec 4.2 */

	/* ─── Mount: create editor, restore cursor/scroll, focus title ─── */
	onMount(() => {
		titleEl?.focus();

		const state = EditorState.create({
			doc: value,
			extensions: [
				history(),
				drawSelection(),
				markdown({ base: markdownLanguage }), /* no codeLanguages — saves 500KB+ */
				keymap.of([...defaultKeymap, ...historyKeymap]),
				dirCompartment.of(EditorView.editorAttributes.of({ dir })),
				EditorView.lineWrapping,
				EditorView.contentAttributes.of({ dir }),
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						const text = update.state.doc.toString();
						latestText = text;
						onchange?.(text);
						if (saveTimer) clearTimeout(saveTimer);
						saveTimer = setTimeout(() => { onsave?.(latestText); }, SAVE_DEBOUNCE);
					}
					if (update.selectionSet) {
						oncursorchange?.(update.state.selection.main.head);
						/* Toolbar state: batch to rAF so it doesn't block typing */
						if (!toolbarRafPending) {
							toolbarRafPending = true;
							requestAnimationFrame(() => {
								toolbarRafPending = false;
								if (update.view && !update.view.destroyed) updateToolbarState(update.view);
							});
						}
					}
				}),
				EditorView.domEventHandlers({
					scroll(event, editorView) {
						onscrollchange?.(editorView.scrollDOM.scrollTop);
						return false;
					},
				}),
				EditorView.theme({
					'&': { background: 'transparent', fontSize: '16px', fontFamily: 'inherit' },
					'&.cm-focused': { outline: 'none' },
					'.cm-scroller': { fontFamily: 'inherit' },
					'.cm-content': { caretColor: 'var(--text-normal, #1a1a1a)', padding: '0' },
					'.cm-cursor': { borderLeftColor: 'var(--text-normal, #1a1a1a)', borderLeftWidth: '1.5px' },
					'.cm-line': { padding: '0' },
					'.cm-activeLine': { background: 'transparent' },
					'.cm-gutters': { display: 'none' },
					'.cm-selectionBackground': { background: 'color-mix(in srgb, var(--interactive-accent, #7c3aed) 20%, transparent)' },
				}),
			],
		});

		view = new EditorView({ state, parent: editorEl! });

		if (initialCursorPos > 0 && initialCursorPos <= view.state.doc.length) {
			view.dispatch({ selection: { anchor: initialCursorPos } });
		}
		if (initialScrollTop > 0) {
			requestAnimationFrame(() => { view?.scrollDOM.scrollTo({ top: initialScrollTop }); });
		}
	});

	/* ─── Destroy ─── */
	onDestroy(() => {
		if (saveTimer) clearTimeout(saveTimer);
		if (latestText !== value) onflush?.(latestText);
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

	/* ─── Properties ─── */
	function handlePropertyChange(index: number, field: 'key' | 'value', newVal: string) {
		const updated = [...properties];
		updated[index] = { ...updated[index], [field]: newVal };
		onpropertieschange?.(updated);
	}

	function addProperty() {
		onpropertieschange?.([...properties, { key: '', value: '' }]);
	}

	function removeProperty(index: number) {
		onpropertieschange?.(properties.filter((_, i) => i !== index));
	}

	const titleAlignment = $derived($appSettings.titleAlignment ?? 'center');
	/* Cache properties length at mount — parent controls properties, not typing (PA requirement) */
	let hasProperties = properties.length > 0;

	/* ─── Toolbar — CM6 commands, NO DOM manipulation (spec 3.6) ─── */

	/** Wrap selected text or insert markers at cursor */
	function wrapSelection(before: string, after: string) {
		if (!view) return;
		const { from, to } = view.state.selection.main;
		if (from === to) {
			view.dispatch({ changes: { from, insert: before + after }, selection: { anchor: from + before.length } });
		} else {
			const text = view.state.sliceDoc(from, to);
			view.dispatch({ changes: { from, to, insert: before + text + after }, selection: { anchor: from + before.length, head: to + before.length } });
		}
		view.focus();
	}

	/** Apply or toggle heading level */
	function applyHeading(level: number) {
		if (!view) return;
		const line = view.state.doc.lineAt(view.state.selection.main.head);
		const match = line.text.match(/^(#{1,6})\s/);
		const prefix = '#'.repeat(level) + ' ';
		if (match && match[1].length === level) {
			view.dispatch({ changes: { from: line.from, to: line.from + match[0].length, insert: '' } });
		} else if (match) {
			view.dispatch({ changes: { from: line.from, to: line.from + match[0].length, insert: prefix } });
		} else {
			view.dispatch({ changes: { from: line.from, insert: prefix } });
		}
		view.focus();
	}

	/** Insert prefix at line start (for lists, quotes, etc.) */
	function toggleLinePrefix(prefix: string) {
		if (!view) return;
		const line = view.state.doc.lineAt(view.state.selection.main.head);
		if (line.text.startsWith(prefix)) {
			view.dispatch({ changes: { from: line.from, to: line.from + prefix.length } });
		} else {
			view.dispatch({ changes: { from: line.from, insert: prefix } });
		}
		view.focus();
	}

	/** Insert text at cursor */
	function insertAtCursor(text: string) {
		if (!view) return;
		const pos = view.state.selection.main.head;
		view.dispatch({ changes: { from: pos, insert: text } });
		view.focus();
	}

	/** Toolbar active state — only updated on selectionSet, NOT on docChanged */
	let toolbarRafPending = false;
	let activeHeading = $state(0);
	let activeBold = $state(false);
	let activeItalic = $state(false);

	function updateToolbarState(editorView: EditorView) {
		const line = editorView.state.doc.lineAt(editorView.state.selection.main.head);
		const hMatch = line.text.match(/^(#{1,6})\s/);
		activeHeading = hMatch ? hMatch[1].length : 0;

		const { from, to } = editorView.state.selection.main;
		if (from !== to) {
			const sel = editorView.state.sliceDoc(from, to);
			activeBold = sel.startsWith('**') && sel.endsWith('**');
			activeItalic = (sel.startsWith('_') && sel.endsWith('_')) || (sel.startsWith('*') && sel.endsWith('*'));
		} else {
			activeBold = false;
			activeItalic = false;
		}
	}

	/* Script symbols — contextual based on dir (spec 3.6) */
	const scriptSymbols = $derived(
		dir === 'rtl'
			? [
				{ label: '«»', insert: '«»', title: 'قوسا اقتباس' },
				{ label: '؛', insert: '؛', title: 'فاصلة منقوطة' },
				{ label: '،', insert: '،', title: 'فاصلة عربية' },
				{ label: '؟', insert: '؟', title: 'علامة استفهام' },
				{ label: '﷽', insert: '﷽', title: 'بسملة' },
				{ label: 'ﷺ', insert: 'ﷺ', title: 'صلى الله عليه وسلم' },
			]
			: [
				{ label: '""', insert: '""', title: 'Smart quotes' },
				{ label: '—', insert: '—', title: 'Em dash' },
				{ label: '–', insert: '–', title: 'En dash' },
				{ label: '…', insert: '…', title: 'Ellipsis' },
				{ label: '©', insert: '©', title: 'Copyright' },
				{ label: '™', insert: '™', title: 'Trademark' },
			]
	);

	export function focus() { view?.focus(); }
	export function getText(): string { return latestText; }
</script>

<div class="e-desk" dir={dir}>
	<!-- Breadcrumb — on the paper, above the title (spec 3.2) -->
	<div class="e-breadcrumb">
		<div class="e-bc-nav">
			{#if onnavigateback}
				<button class="e-bc-btn" onclick={onnavigateback} title={$t('notePane.back')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
				</button>
			{/if}
			{#if onnavigateforward}
				<button class="e-bc-btn" onclick={onnavigateforward} title={$t('notePane.forward')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
				</button>
			{/if}
		</div>
		<span class="e-bc-path">
			{#if libraryName}<span class="e-bc-lib">{libraryName}</span>{/if}
			{#if libraryName && breadcrumbPath}<span class="e-bc-sep"> / </span>{/if}
			{#if breadcrumbPath}<span class="e-bc-note">{breadcrumbPath}</span>{/if}
		</span>
		{#if onmoreoptions}
			<button class="e-bc-btn e-bc-more" onclick={onmoreoptions} title={$t('notePane.moreOptions')}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="12" cy="19" r="1.5"/></svg>
			</button>
		{/if}
	</div>

	<div class="e-paper">
		<!-- Title -->
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

		<!-- Properties — collapsible (spec 3.2, 0.3.1) -->
		{#if hasProperties || !propsCollapsed}
			<div class="e-props">
				<button class="e-props-toggle" onclick={() => propsCollapsed = !propsCollapsed}>
					<svg class="e-props-chevron" class:collapsed={propsCollapsed} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
					<span>{$t('notePane.properties')}</span>
				</button>
				{#if !propsCollapsed}
					<div class="e-props-list">
						{#each properties as prop, i}
							<div class="e-prop-row">
								<input class="e-prop-key" value={prop.key} dir="auto"
									placeholder={$t('eNotePane.propertyKey')}
									oninput={(e) => handlePropertyChange(i, 'key', (e.target as HTMLInputElement).value)} />
								<input class="e-prop-val" value={prop.value} dir="auto"
									placeholder={$t('eNotePane.propertyValue')}
									oninput={(e) => handlePropertyChange(i, 'value', (e.target as HTMLInputElement).value)} />
								<button class="e-prop-remove" onclick={() => removeProperty(i)}>×</button>
							</div>
						{/each}
						<button class="e-prop-add" onclick={addProperty}>+ {$t('notePane.addProperty')}</button>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Toolbar — dispatches CM6 commands, state updates on selectionSet only (spec 3.6) -->
		{#if view}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="e-toolbar" dir={dir} onmousedown={(e) => e.preventDefault()}>
				<button class="e-tb e-tb-dir" title="Undo" onclick={() => undo(view!)}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6.69 3L3 13"/></svg>
				</button>
				<button class="e-tb e-tb-dir" title="Redo" onclick={() => redo(view!)}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6.69 3L21 13"/></svg>
				</button>
				<span class="e-tb-sep"></span>
				<button class="e-tb" class:active={activeHeading===1} title="H1" onclick={() => applyHeading(1)}>H1</button>
				<button class="e-tb" class:active={activeHeading===2} title="H2" onclick={() => applyHeading(2)}>H2</button>
				<button class="e-tb" class:active={activeHeading===3} title="H3" onclick={() => applyHeading(3)}>H3</button>
				<span class="e-tb-sep"></span>
				<button class="e-tb" class:active={activeBold} title="Bold" onclick={() => wrapSelection('**', '**')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/></svg>
				</button>
				<button class="e-tb" class:active={activeItalic} title="Italic" onclick={() => wrapSelection('_', '_')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg>
				</button>
				<button class="e-tb" title="Underline" onclick={() => wrapSelection('<u>', '</u>')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 3v7a6 6 0 0 0 6 6 6 6 0 0 0 6-6V3"/><line x1="4" y1="21" x2="20" y2="21"/></svg>
				</button>
				<button class="e-tb" title="Strikethrough" onclick={() => wrapSelection('~~', '~~')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4H9a3 3 0 0 0-2.83 4"/><path d="M14 12a4 4 0 0 1 0 8H6"/><line x1="4" y1="12" x2="20" y2="12"/></svg>
				</button>
				<button class="e-tb" title="Highlight" onclick={() => wrapSelection('==', '==')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 11-6 6v3h9l3-3"/><path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"/></svg>
				</button>
				<button class="e-tb" title="Inline code" onclick={() => wrapSelection('`', '`')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
				</button>
				<span class="e-tb-sep"></span>
				<button class="e-tb" title="Bullet list" onclick={() => toggleLinePrefix('- ')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><circle cx="3" cy="6" r="1" fill="currentColor"/><circle cx="3" cy="12" r="1" fill="currentColor"/><circle cx="3" cy="18" r="1" fill="currentColor"/></svg>
				</button>
				<button class="e-tb" title="Numbered list" onclick={() => toggleLinePrefix('1. ')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="10" y1="6" x2="21" y2="6"/><line x1="10" y1="12" x2="21" y2="12"/><line x1="10" y1="18" x2="21" y2="18"/><text x="2" y="8" font-size="8" fill="currentColor" stroke="none">1</text><text x="2" y="14" font-size="8" fill="currentColor" stroke="none">2</text><text x="2" y="20" font-size="8" fill="currentColor" stroke="none">3</text></svg>
				</button>
				<button class="e-tb" title="Task list" onclick={() => toggleLinePrefix('- [ ] ')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="5" width="14" height="14" rx="2"/><path d="M9 12l2 2 4-4"/></svg>
				</button>
				<button class="e-tb" title="Blockquote" onclick={() => toggleLinePrefix('> ')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 17h3l2-4V7H5v6h3"/><path d="M15 17h3l2-4V7h-6v6h3"/></svg>
				</button>
				<button class="e-tb" title="Link" onclick={() => wrapSelection('[', '](url)')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
				</button>
				<button class="e-tb" title="Horizontal rule" onclick={() => insertAtCursor('\n---\n')}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="2" y1="12" x2="22" y2="12"/></svg>
				</button>
				{#if $appSettings.enableScriptToolbar}
					<span class="e-tb-sep"></span>
					{#each scriptSymbols as sym}
						<button class="e-tb e-tb-script" title={sym.title} onclick={() => insertAtCursor(sym.insert)}>
							{sym.label}
						</button>
					{/each}
				{/if}
			</div>
		{/if}

		<!-- CM6 Editor -->
		<div class="e-editor" bind:this={editorEl}></div>
	</div>
</div>

<style>
	/* ─── The Desk (spec 3.1) ─── */
	.e-desk {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		background: #e8e8ec; /* desk surface */
		overflow: hidden;
		min-width: 0;
		min-height: 0;
	}

	/* ─── Breadcrumb — centered on desk, same width as paper (spec 3.2) ─── */
	.e-breadcrumb {
		width: 100%;
		max-width: 1200px; /* matches paper width */
		display: flex;
		align-items: center;
		padding: 6px 48px; /* horizontal matches paper padding */
		font-size: 13px;
		color: var(--text-muted, #888);
		flex-shrink: 0;
		background: #ffffff; /* paper color — breadcrumb is part of paper top */
	}
	.e-bc-nav {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-inline-end: 8px;
	}
	.e-bc-btn {
		width: 24px; height: 24px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 4px;
		color: var(--text-faint, #aaa); cursor: pointer;
	}
	.e-bc-btn:hover { background: var(--background-modifier-hover, #f0f0f0); color: var(--text-normal, #333); }
	:global([dir="rtl"]) .e-bc-nav svg { transform: scaleX(-1); }
	.e-bc-path { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: end; }
	.e-bc-lib { color: var(--text-muted, #888); }
	.e-bc-sep { color: var(--text-faint, #ccc); margin-inline: 4px; }
	.e-bc-note { color: var(--text-normal, #333); }
	.e-bc-more { margin-inline-start: 8px; }

	/* ─── The Paper (spec 3.1) ─── */
	.e-paper {
		width: 100%;
		max-width: 1200px; /* paper width */
		flex: 1;
		display: flex;
		flex-direction: column;
		background: #ffffff; /* paper color */
		padding: 48px; /* paper padding */
		overflow-y: auto;
		overflow-x: hidden;
	}

	/* ─── Title (spec 0.3) ─── */
	.e-title {
		display: block; width: 100%;
		border: none; outline: none; background: transparent;
		font-size: 28px; font-weight: 700; font-family: inherit;
		color: var(--text-normal, #1a1a1a);
		padding: 0;
		margin-block: 0 8px; /* reduced: properties may follow */
		margin-inline: 0;
		text-align: start;
	}
	.e-title.e-title-center { text-align: center; }
	.e-title::placeholder { color: var(--text-faint, #ccc); font-weight: 400; }

	/* ─── Properties (spec 0.3.1, 3.2) ─── */
	.e-props {
		margin-block-end: 16px; /* space before editor */
		border: 1px solid var(--background-modifier-border, #e0e0e0);
		border-radius: 6px;
		padding: 8px 12px;
	}
	.e-props-toggle {
		display: flex; align-items: center; gap: 6px;
		border: none; background: none; cursor: pointer;
		font-size: 12px; color: var(--text-muted, #888);
		font-family: inherit; padding: 0;
	}
	.e-props-toggle:hover { color: var(--text-normal, #333); }
	.e-props-chevron { transition: transform 0.15s ease; }
	.e-props-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .e-props-chevron.collapsed { transform: rotate(90deg); }
	.e-props-list { margin-block-start: 8px; }
	.e-prop-row {
		display: flex; align-items: center; gap: 8px;
		margin-block-end: 4px;
	}
	.e-prop-key {
		flex: 0 0 120px; border: none;
		border-block-end: 1px solid var(--background-modifier-border, #e0e0e0);
		background: transparent; padding: 4px 2px;
		font-size: 13px; color: var(--text-muted, #888);
		font-family: inherit; outline: none;
	}
	.e-prop-key:focus { border-block-end-color: var(--interactive-accent, #7c3aed); }
	.e-prop-val {
		flex: 1; border: none;
		border-block-end: 1px solid var(--background-modifier-border, #e0e0e0);
		background: transparent; padding: 4px 2px;
		font-size: 13px; color: var(--text-normal, #333);
		font-family: inherit; outline: none;
	}
	.e-prop-val:focus { border-block-end-color: var(--interactive-accent, #7c3aed); }
	.e-prop-remove {
		width: 20px; height: 20px; border: none; background: none;
		color: var(--text-faint, #ccc); cursor: pointer; font-size: 14px;
		border-radius: 50%; display: flex; align-items: center; justify-content: center;
	}
	.e-prop-remove:hover { color: var(--text-error, #e53935); background: rgba(229,57,53,0.08); }
	.e-prop-add {
		border: none; background: none;
		color: var(--text-muted, #888); font-size: 12px;
		cursor: pointer; padding: 4px 0; font-family: inherit;
	}
	.e-prop-add:hover { color: var(--interactive-accent, #7c3aed); }

	/* ─── Editor (fills remaining paper space) ─── */
	/* ─── Toolbar (spec 3.6) ─── */
	.e-toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 2px;
		padding-block: 8px;
		margin-block-end: 8px; /* spacing before editor content */
		border-block-end: 1px solid var(--background-modifier-border, #e8e8ec);
		user-select: none;
	}
	.e-tb {
		width: 28px; height: 28px;
		display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 4px;
		color: var(--text-muted, #888); cursor: pointer;
		font-size: 12px; font-weight: 600; font-family: inherit;
		flex-shrink: 0;
	}
	.e-tb:hover { background: var(--background-modifier-hover, #f0f0f0); color: var(--text-normal, #333); }
	.e-tb.active { background: var(--background-modifier-hover, #f0f0f0); color: var(--interactive-accent, #7c3aed); }
	.e-tb-sep { width: 1px; height: 16px; background: var(--background-modifier-border, #e0e0e0); margin-inline: 4px; flex-shrink: 0; }
	.e-tb-script { font-size: 13px; width: auto; padding-inline: 6px; }
	:global([dir="rtl"]) .e-tb-dir svg { transform: scaleX(-1); } /* flip undo/redo arrows in RTL */

	/* ─── Editor (spec 3.3) ─── */
	.e-editor { flex: 1; min-height: 0; }
	.e-editor :global(.cm-editor) { height: 100%; }
	.e-editor :global(.cm-scroller) { overflow: auto; }
</style>

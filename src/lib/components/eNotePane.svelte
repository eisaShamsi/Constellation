<script lang="ts">
	/**
	 * eNotePane — Phase 3: Breadcrumb & Properties
	 * Desk + paper + breadcrumb + title + properties + CM6 editor + save.
	 * Spec: docs/eNotePane-spec.md, Sections 3.2, 0.3.1
	 */
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { appSettings } from '$lib/libraries/store';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';

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
	.e-editor { flex: 1; min-height: 0; }
	.e-editor :global(.cm-editor) { height: 100%; }
	.e-editor :global(.cm-scroller) { overflow: auto; }
</style>

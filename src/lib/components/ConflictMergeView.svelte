<script lang="ts">
	// PJ-088 — the conflict-resolution SIDE-BY-SIDE MERGE view (full-center-zone overlay).
	// Left = YOUR version (editable, seeded from the live note model's current unsaved content).
	// Right = the OUTSIDE copy (read-only, from the .conflict side-file). The official CM6
	// @codemirror/merge MergeView renders the diff + per-chunk copy-across arrows (outside→yours).
	// Save merged goes through resolveConflictMerge — the model + durability gate, NEVER a raw write
	// (the PJ-070 clobber). Cancel is a pure no-op: the merged text lives here and reaches the model
	// only at Save. Both versions stay on disk until an explicit durable Save.
	import { onDestroy, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { mergeViewTarget, closeMergeView } from '$lib/stores/mergeView';
	import { openTabs, readNote, resolveConflictMerge, appSettings, getEffectiveScriptFonts } from '$lib/libraries/store';
	import { compose } from '$lib/editor/noteModel';
	import { t } from '$lib/i18n';
	import { detectDir } from '$lib/utils';
	import { EditorState, type Extension } from '@codemirror/state';
	import { EditorView, keymap, drawSelection } from '@codemirror/view';
	import { RTL_MOTION_ENABLED } from '$lib/editor/rtlFlag'; // PJ-106 §A1 (SI4-01)
	import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
	import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
	import { history, defaultKeymap, historyKeymap, indentWithTab } from '@codemirror/commands';
	import {
		livePreviewPlugin, livePreviewTheme, baseLensField,
		libraryPathField, setLibraryPath, notePathField, setNotePath,
		attachmentFolderField, setAttachmentFolder, linkTraversalMapField,
	} from '$lib/editor/livePreview';
	import { calloutPlugin, calloutTheme, calloutCollapseField } from '$lib/editor/calloutPlugin';
	import { bidiPlugin, bidiTheme, scriptFontsField, setScriptFonts } from '$lib/editor/bidiPlugin';
	import { Highlight as HighlightExt } from '$lib/editor/markdownHighlight';

	// The FocusPane reseed hook (FocusPane is not under the reloadVersion {#key}) — passed from +layout.
	let { focusReseed }: { focusReseed?: (path: string) => void } = $props();

	let mountEl = $state<HTMLDivElement | undefined>();
	let mergeView: any = null;         // the @codemirror/merge MergeView instance
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');            // a durable-save failure message (kept open, retryable)
	let target = $derived($mergeViewTarget);

	/** The shared live-preview rendering extensions (Editor Parity — full live preview, the Boss's choice).
	 *  `editable` distinguishes YOUR pane (left, editable) from the OUTSIDE pane (right, read-only). */
	function paneExtensions(editable: boolean, dir: 'ltr' | 'rtl'): Extension[] {
		const base: Extension[] = [
			markdown({ base: markdownLanguage, extensions: [HighlightExt] }),
			syntaxHighlighting(defaultHighlightStyle),
			calloutCollapseField,
			livePreviewPlugin, livePreviewTheme, calloutPlugin, calloutTheme, baseLensField,
			scriptFontsField, bidiPlugin, bidiTheme,
			/* PJ-106 §A1 (SI4-01) — the merge panes are editable bilingual surfaces too:
			   connect per-line direction to the caret engine + a deterministic base (not 'auto'). */
			...(RTL_MOTION_ENABLED ? [EditorView.perLineTextDirection.of(true)] : []),
			libraryPathField, notePathField, attachmentFolderField, linkTraversalMapField,
			EditorView.lineWrapping,
			EditorView.editorAttributes.of({ dir }),
			EditorView.contentAttributes.of({ dir }),
		];
		if (editable) {
			base.push(history(), drawSelection(), keymap.of([indentWithTab, ...defaultKeymap, ...historyKeymap]));
		} else {
			base.push(EditorState.readOnly.of(true), EditorView.editable.of(false));
		}
		return base;
	}

	/** Seed the live-preview path fields on a pane so image/wikilink resolution works. */
	function seedFields(view: EditorView, notePath: string, libraryPath: string) {
		view.dispatch({ effects: [
			setNotePath.of(notePath),
			setLibraryPath.of(libraryPath),
			setAttachmentFolder.of(''),
			setScriptFonts.of(getEffectiveScriptFonts($appSettings)),
		] });
	}

	async function build(t0: NonNullable<typeof target>) {
		loading = true; error = '';
		try {
			// YOUR version = the live model's current (unsaved) content. A conflict is only ever raised
			// on an OPEN note, so a tab + model normally exist; compose() reads the model out (identity-
			// guarded). If the note was closed after the conflict, ask the user to reopen it (the merge
			// needs the live model as "yours", not stale disk).
			const tab = get(openTabs).find((x) => x.path === t0.notePath);
			if (!tab) { error = $t('conflict.mergeReopen'); loading = false; return; }
			const mine = compose(tab.id, t0.notePath) as { ok: boolean; content?: string };
			const yourText = mine.ok ? (mine.content ?? '') : (tab.content ?? '');
			const libraryPath = tab.libraryPath ?? '';
			// The OUTSIDE copy = the .conflict side-file (read_note has no .md restriction → reads the .md.txt).
			const theirText = await readNote(t0.sidecarPath);

			const dirYours = detectDir(yourText) as 'ltr' | 'rtl';
			const dirTheirs = detectDir(theirText) as 'ltr' | 'rtl';

			const copyLabel = get(t)('conflict.copyAcross'); // captured for the custom control's label/tooltip
			const { MergeView } = await import('@codemirror/merge'); // lazy — off the main bundle / hot path
			mergeView = new MergeView({
				parent: mountEl!,
				orientation: 'a-b',            // a = left (yours), b = right (outside)
				revertControls: 'b-to-a',      // copy-across direction: an outside chunk → yours (a is editable)
				// PJ-088 Boss feedback — the default gutter chevron was too subtle. Render a clear, LABELED
				// button per changed chunk instead (the library delegates the click on the container).
				renderRevertControl: () => {
					const b = document.createElement('button');
					b.className = 'cm-copy-across';
					b.textContent = '◀ ' + copyLabel;
					b.title = copyLabel;
					b.setAttribute('aria-label', copyLabel);
					return b;
				},
				highlightChanges: true,
				gutter: true,
				collapseUnchanged: { margin: 3, minSize: 4 },
				a: { doc: yourText, extensions: paneExtensions(true, dirYours) },
				b: { doc: theirText, extensions: paneExtensions(false, dirTheirs) },
			});
			seedFields(mergeView.a, t0.notePath, libraryPath);
			seedFields(mergeView.b, t0.notePath, libraryPath);
		} catch (e) {
			console.error('[PJ-088] merge view build failed', e);
			error = $t('conflict.mergeLoadError');
		} finally {
			loading = false;
		}
	}

	// (Re)build when the target changes; tear down when it clears.
	$effect(() => {
		const t0 = target;
		if (t0 && mountEl) { void rebuild(t0); }
		else destroyView();
	});
	async function rebuild(t0: NonNullable<typeof target>) { destroyView(); await tick(); await build(t0); }
	function destroyView() { if (mergeView) { try { mergeView.destroy(); } catch {} mergeView = null; } }
	onDestroy(destroyView);

	async function saveMerged() {
		if (!target || !mergeView || saving) return;
		saving = true; error = '';
		const merged = mergeView.a.state.doc.toString(); // YOUR pane = the reconciled result
		const r = await resolveConflictMerge(target.notePath, target.sidecarPath, merged, { focusReseed });
		saving = false;
		if (r.ok) closeMergeView();
		else error = $t('conflict.mergeSaveError'); // durable-save failed — everything kept, retryable
	}
	function cancel() { closeMergeView(); } // pure no-op: nothing was pushed into the model
</script>

{#if target}
	<div class="cm-overlay" role="dialog" aria-modal="true" aria-label={$t('conflict.mergeTitle')}>
		<div class="cm-head">
			<div class="cm-titles">
				<span class="cm-title">{$t('conflict.mergeTitle')}</span>
				<span class="cm-sub" dir={detectDir(target.noteName)}>{target.noteName}</span>
			</div>
			<div class="cm-colheads" aria-hidden="true">
				<span class="cm-colhead cm-yours">{$t('conflict.yourVersion')}</span>
				<span class="cm-colhead cm-theirs">{$t('conflict.outsideCopy')}</span>
			</div>
		</div>

		<p class="cm-help">{$t('conflict.mergeHelp')}</p>

		<div class="cm-body">
			{#if loading}<div class="cm-loading">{$t('conflict.mergeLoading')}</div>{/if}
			{#if error}<div class="cm-error" role="alert">{error}</div>{/if}
			<div class="cm-merge" bind:this={mountEl}></div>
		</div>

		<div class="cm-foot">
			<button class="cm-btn cm-cancel" type="button" onclick={cancel} disabled={saving}>{$t('conflict.mergeCancel')}</button>
			<button class="cm-btn cm-save" type="button" onclick={saveMerged} disabled={saving || loading}>
				{saving ? $t('conflict.mergeSaving') : $t('conflict.mergeSave')}
			</button>
		</div>
	</div>
{/if}

<style>
	.cm-overlay {
		position: fixed; inset: 0; z-index: 100000;
		display: flex; flex-direction: column;
		background: var(--background-primary, #fff);
		color: var(--text-normal, #222);
		padding: 18px 24px 16px;
		gap: 10px;
	}
	.cm-head { display: flex; flex-direction: column; gap: 8px; flex: 0 0 auto; }
	.cm-titles { display: flex; align-items: baseline; gap: 12px; }
	.cm-title { font-size: 18px; font-weight: 700; }
	.cm-sub { font-size: 14px; color: var(--text-muted, #888); min-width: 0; overflow: hidden; text-overflow: ellipsis; }
	.cm-colheads { display: flex; gap: 0; }
	.cm-colhead { flex: 1 1 50%; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted, #888); padding: 0 6px; }
	.cm-yours { color: var(--interactive-accent, #7c3aed); }
	.cm-help { flex: 0 0 auto; margin: 0; font-size: 13px; color: var(--text-muted, #888); }
	.cm-body { flex: 1 1 auto; min-height: 0; position: relative; border: 1px solid var(--background-modifier-border, #ddd); border-radius: 8px; overflow: hidden; }
	.cm-merge { position: absolute; inset: 0; overflow: auto; }
	.cm-merge :global(.cm-mergeView) { height: 100%; }
	.cm-merge :global(.cm-editor) { height: 100%; }
	/* PJ-088 — the copy-across control as a clear, labeled button (default chevron was too subtle). */
	.cm-merge :global(.cm-merge-revert) { width: auto !important; min-width: 96px; }
	.cm-merge :global(.cm-copy-across) {
		display: inline-flex; align-items: center; gap: 4px;
		background: var(--interactive-accent, #7c3aed); color: #fff;
		border: none; border-radius: 6px; padding: 3px 9px;
		font-size: 12px; font-weight: 600; line-height: 1.4; white-space: nowrap; cursor: pointer;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.28);
	}
	.cm-merge :global(.cm-copy-across:hover) { filter: brightness(1.12); }
	.cm-merge :global(.cm-copy-across:active) { transform: translateY(1px); }
	.cm-loading, .cm-error { position: absolute; inset-block-start: 8px; inset-inline: 8px; z-index: 2; padding: 6px 10px; border-radius: 6px; font-size: 13px; }
	.cm-loading { background: var(--background-secondary, #f4f4f6); color: var(--text-muted, #888); }
	.cm-error { background: #8a5a00; color: #fff; }
	.cm-foot { flex: 0 0 auto; display: flex; justify-content: flex-end; gap: 10px; }
	.cm-btn { padding: 8px 18px; border-radius: 7px; cursor: pointer; font-size: 14px; border: 1px solid var(--background-modifier-border, #ccc); background: var(--background-secondary, #f4f4f6); color: inherit; }
	.cm-btn:disabled { opacity: 0.5; cursor: default; }
	.cm-cancel:hover:not(:disabled) { background: var(--background-modifier-hover, #e8e8ec); }
	.cm-save { background: var(--interactive-accent, #7c3aed); color: #fff; border-color: transparent; font-weight: 600; }
	.cm-save:hover:not(:disabled) { filter: brightness(1.08); }
</style>

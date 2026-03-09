<script lang="ts">
	import { parseFrontmatter, extractHeadings, editingTabIds, toggleEditMode, saveTabContent, resolveWikilink, openNoteTab, vaultAppearances, vaults } from '$lib/vaults/store';
	import type { OpenTab } from '$lib/vaults/store';
	import { detectDir, renderMarkdown } from '$lib/utils';
	import { dir } from '$lib/i18n';
	import { get } from 'svelte/store';
	import PropertyEditor from './PropertyEditor.svelte';

	let {
		tab,
		isFocused = false,
		onFocus,
		ar = false,
		color = '#7c3aed',
		splitView = false
	}: {
		tab: OpenTab | null;
		isFocused?: boolean;
		onFocus: () => void;
		ar?: boolean;
		color?: string;
		splitView?: boolean;
	} = $props();

	const parsed = $derived(tab ? parseFrontmatter(tab.content) : null);
	const properties = $derived(parsed?.properties ?? []);
	const noteBody = $derived(parsed?.body ?? '');
	const noteDir = $derived(noteBody ? detectDir(noteBody) : $dir);
	const editing = $derived(tab ? $editingTabIds.has(tab.id) : false);

	// Vault appearance
	const vaultId = $derived(tab ? get(vaults).find(v => tab!.vaultPath === v.path)?.id : null);
	const appearance = $derived(vaultId ? $vaultAppearances[vaultId] : null);
	const paneStyle = $derived.by(() => {
		if (!appearance) return '';
		const vars: string[] = [];
		if (appearance.accent_color) vars.push(`--vault-accent: ${appearance.accent_color}`);
		if (appearance.base_font_size) vars.push(`--vault-font-size: ${appearance.base_font_size}px`);
		if (appearance.text_font_family) vars.push(`--vault-text-font: ${appearance.text_font_family}`);
		if (appearance.monospace_font_family) vars.push(`--vault-mono-font: ${appearance.monospace_font_family}`);
		return vars.join('; ');
	});

	// Edit mode state
	let editBody = $state('');
	let saveTimeout: ReturnType<typeof setTimeout>;
	let saving = $state(false);
	let prevTabId = $state('');

	// Sync editBody when tab changes
	$effect(() => {
		if (tab && tab.id !== prevTabId) {
			editBody = parseFrontmatter(tab.content).body;
			prevTabId = tab.id;
		}
	});

	// Re-sync when entering edit mode
	$effect(() => {
		if (editing && tab) {
			editBody = parseFrontmatter(tab.content).body;
		}
	});

	function handleBodyInput(e: Event) {
		editBody = (e.target as HTMLTextAreaElement).value;
		debouncedSaveBody();
	}

	function debouncedSaveBody() {
		clearTimeout(saveTimeout);
		saveTimeout = setTimeout(async () => {
			if (!tab) return;
			saving = true;
			const currentParsed = parseFrontmatter(tab.content);
			await saveTabContent(tab.id, tab.path, currentParsed.properties, editBody);
			saving = false;
		}, 800);
	}

	function handleToggleEdit() {
		if (tab) toggleEditMode(tab.id);
	}

	// ─── Smart bracket/pair wrapping (Obsidian-style) ───
	// Uses execCommand('insertText') to preserve the browser's native undo/redo stack.
	const WRAP_PAIRS: Record<string, string> = {
		'(': ')',
		'[': ']',
		'{': '}',
		'"': '"',
		"'": "'",
		'`': '`',
		'_': '_',
		'*': '*',
	};

	function handleEditorKeydown(e: KeyboardEvent) {
		const ta = e.target as HTMLTextAreaElement;
		const { selectionStart, selectionEnd, value } = ta;
		const selectedText = value.substring(selectionStart, selectionEnd);
		const close = WRAP_PAIRS[e.key];

		if (!close || e.ctrlKey || e.metaKey || e.altKey) return;

		e.preventDefault();
		ta.focus();

		if (selectionStart === selectionEnd) {
			// No selection — auto-close: insert pair, cursor between
			document.execCommand('insertText', false, e.key + close);
			// Move cursor back between the pair
			ta.selectionStart = ta.selectionEnd = selectionStart + 1;
		} else if (e.key === '[' && selectedText.startsWith('[') && selectedText.endsWith(']')) {
			// Special case: upgrade [text] → [[text]]
			const inner = selectedText.slice(1, -1);
			document.execCommand('insertText', false, '[[' + inner + ']]');
			// Select the whole [[inner]]
			ta.selectionStart = selectionStart;
			ta.selectionEnd = selectionStart + inner.length + 4;
		} else {
			// Normal wrap: surround selection with the pair
			document.execCommand('insertText', false, e.key + selectedText + close);
			// Keep the inner text selected (inside the brackets)
			ta.selectionStart = selectionStart + 1;
			ta.selectionEnd = selectionEnd + 1;
		}

		// execCommand triggers input event → handleBodyInput runs automatically
	}

	// WikiLink click handler via event delegation
	async function handleNoteContentClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		const wikilinkEl = target.closest('a.wikilink') as HTMLAnchorElement | null;
		if (!wikilinkEl || !tab) return;

		e.preventDefault();
		const linkTarget = decodeURIComponent(wikilinkEl.dataset.wikilink ?? '');
		if (!linkTarget) return;

		const resolved = await resolveWikilink(tab.vaultPath, linkTarget);
		if (resolved) {
			await openNoteTab(resolved, tab.vaultName, tab.vaultColor);
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pane" class:focused={isFocused} onclick={onFocus}>
	{#if tab}
		{#if splitView}
			<div class="pane-tab-bar" style:--vault-color={color}>
				<div class="pane-tab">
					<span class="pane-tab-vault">{tab.vaultName}</span>
					<span class="pane-tab-title">{tab.name}</span>
				</div>
				<div class="pane-tab-actions">
					{#if saving}<span class="bc-saving">{ar ? 'جارٍ الحفظ...' : 'Saving...'}</span>{/if}
					<button class="bc-edit-btn" class:active={editing} onclick={handleToggleEdit}
						title={editing ? (ar ? 'وضع القراءة' : 'Reading mode') : (ar ? 'وضع التحرير' : 'Editing mode')}>
						{#if editing}
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
						{:else}
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
						{/if}
					</button>
				</div>
			</div>
		{:else}
			<div class="pane-breadcrumb">
				<span class="bc-vault">{tab.vaultName}</span>
				<span class="bc-sep">/</span>
				<span class="bc-note">{tab.name}</span>
				<div class="bc-actions">
					{#if saving}<span class="bc-saving">{ar ? 'جارٍ الحفظ...' : 'Saving...'}</span>{/if}
					<button class="bc-edit-btn" class:active={editing} onclick={handleToggleEdit}
						title={editing ? (ar ? 'وضع القراءة' : 'Reading mode') : (ar ? 'وضع التحرير' : 'Editing mode')}>
						{#if editing}
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
						{:else}
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>
						{/if}
					</button>
				</div>
			</div>
		{/if}
		<div class="note-scroll" dir={noteDir} style={paneStyle}>
			{#if tab}
				<PropertyEditor
					properties={properties}
					body={noteBody}
					tabId={tab.id}
					filePath={tab.path}
					{ar}
				/>
				{#if properties.length > 0}
					<hr class="props-divider"/>
				{/if}
			{/if}
			{#if editing}
				<textarea
					class="note-editor"
					dir={noteDir}
					value={editBody}
					oninput={handleBodyInput}
					onkeydown={handleEditorKeydown}
					placeholder={ar ? 'اكتب هنا...' : 'Start writing...'}
				></textarea>
			{:else}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="note-content" onclick={handleNoteContentClick}>
					{@html renderMarkdown(noteBody)}
				</div>
			{/if}
		</div>
	{:else}
		<div class="pane-empty">
			{ar ? 'اختر ملاحظة' : 'Select a note'}
		</div>
	{/if}
</div>

<style>
	.pane {
		flex: 1; display: flex; flex-direction: column;
		overflow: hidden; min-width: 0; min-height: 0;
	}
	.pane.focused { box-shadow: inset 0 0 0 2px #7c3aed33; }

	.pane-tab-bar {
		display: flex; align-items: flex-end;
		background: #f0f0f4; border-bottom: 1px solid #e0e0e4;
		padding: 12px 4px 0; flex-shrink: 0;
	}
	.pane-tab {
		position: relative;
		display: inline-flex; align-items: center; gap: 6px;
		background: #fff; color: #1f2328;
		border: 1px solid #e0e0e4;
		border-top: 3px solid var(--vault-color, #7c3aed);
		border-bottom: 1px solid #fff;
		margin-bottom: -1px;
		border-radius: 6px 6px 0 0;
		padding: 5px 10px;
		font-size: 0.8rem;
	}
	.pane-tab-vault {
		position: absolute; bottom: 100%; inset-inline-end: 8px;
		font-size: 0.55rem; line-height: 1.3; letter-spacing: 0.02em;
		color: #1f2328;
		background: #f0f0f4;
		padding: 0 5px;
		border-radius: 3px 3px 0 0;
		border: 1px solid #e0e0e4; border-bottom: none;
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
		max-width: 100%; pointer-events: none;
	}
	.pane-tab-title {
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.pane-tab-actions {
		display: flex; align-items: center; gap: 4px;
		margin-inline-start: auto; margin-bottom: 4px;
	}

	.pane-breadcrumb {
		padding: 4px 16px; border-bottom: 1px solid #f0f0f4;
		font-size: 0.78rem; color: #8b8b96; flex-shrink: 0;
		display: flex; align-items: center; min-height: 28px;
	}
	.bc-vault { color: #5c5c66; }
	.bc-sep { margin: 0 4px; color: #d0d0d6; }
	.bc-note { color: #1f2328; }
	.bc-actions { margin-inline-start: auto; display: flex; align-items: center; gap: 4px; }
	.bc-saving { font-size: 0.7rem; color: #7c3aed; }
	.bc-edit-btn {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: #8b8b96; cursor: pointer;
	}
	.bc-edit-btn:hover { background: #e0e0e4; color: #1f2328; }
	.bc-edit-btn.active { color: #7c3aed; }

	.note-scroll {
		flex: 1; overflow-y: auto; padding: 1.5rem 3rem; max-width: 800px;
		font-size: var(--vault-font-size, 0.95rem);
		font-family: var(--vault-text-font, inherit);
		display: flex; flex-direction: column;
	}

	.props-divider { border: none; border-top: 1px solid #e8e8ec; margin: 12px 0; }

	.note-editor {
		width: 100%; min-height: 300px; flex: 1;
		border: none; background: none; resize: none;
		font-family: var(--vault-mono-font, 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', monospace);
		font-size: 0.92rem; line-height: 1.7; color: #1f2328;
		outline: none; padding: 0;
	}

	.note-content { line-height: 1.8; color: #1f2328; flex: 1; }

	.note-content :global(h1) { font-size: 1.8rem; margin: 1.5rem 0 0.75rem; color: #1f2328; }
	.note-content :global(h2) { font-size: 1.4rem; margin: 1.3rem 0 0.5rem; }
	.note-content :global(h3) { font-size: 1.15rem; margin: 1rem 0 0.4rem; }
	.note-content :global(p) { margin: 0.5rem 0; }
	.note-content :global(a) { color: var(--vault-accent, #7c3aed); }
	.note-content :global(a.wikilink) {
		color: var(--vault-accent, #7c3aed);
		text-decoration: none;
		border-bottom: 1px dashed color-mix(in srgb, var(--vault-accent, #7c3aed) 40%, transparent);
		cursor: pointer;
	}
	.note-content :global(a.wikilink:hover) {
		border-bottom-color: var(--vault-accent, #7c3aed);
	}
	.note-content :global(code) { background: #f0f0f4; padding: 0.15em 0.35em; border-radius: 3px; font-size: 0.9em; }
	.note-content :global(pre) { background: #f6f6f9; border: 1px solid #e0e0e4; border-radius: 6px; padding: 1rem; overflow-x: auto; }
	.note-content :global(pre code) { background: none; padding: 0; }
	.note-content :global(blockquote) { border-inline-start: 3px solid var(--vault-accent, #7c3aed); padding: 0.25rem 1rem; margin: 0.5rem 0; color: #5c5c66; }
	.note-content :global(ul), .note-content :global(ol) { padding-inline-start: 1.5rem; }
	.note-content :global(li) { margin: 0.2rem 0; }
	.note-content :global(hr) { border: none; border-top: 1px solid #e0e0e4; margin: 1.5rem 0; }
	.note-content :global(table) { border-collapse: collapse; width: 100%; margin: 0.75rem 0; }
	.note-content :global(th), .note-content :global(td) { border: 1px solid #e0e0e4; padding: 0.4rem 0.7rem; text-align: start; }
	.note-content :global(th) { background: #f6f6f9; }
	.note-content :global(img) { max-width: 100%; border-radius: 4px; }
	.note-content :global(input[type="checkbox"]) { margin-inline-end: 0.4rem; }
	.note-content :global(strong) { font-weight: 600; }

	.pane-empty {
		flex: 1; display: flex; align-items: center; justify-content: center;
		color: #b0b0b8; font-size: 0.85rem;
	}
</style>

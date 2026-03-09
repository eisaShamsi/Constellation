<script lang="ts">
	import { parseFrontmatter, extractHeadings, editingTabIds, toggleEditMode, saveTabContent, resolveWikilink, openNoteTab, vaultAppearances, vaults } from '$lib/vaults/store';
	import type { OpenTab } from '$lib/vaults/store';
	import { detectDir, renderMarkdown, postProcessRenderedContent, collectNoteNames } from '$lib/utils';
	import { dir } from '$lib/i18n';
	import { get } from 'svelte/store';
	import PropertyEditor from './PropertyEditor.svelte';
	import CodeMirrorEditor from './CodeMirrorEditor.svelte';

	let {
		tab,
		isFocused = false,
		onFocus,
		ar = false,
		color = '#7c3aed',
		splitView = false,
		vaultTrees = {} as Record<string, any[]>,
		allTags = [] as string[],
	}: {
		tab: OpenTab | null;
		isFocused?: boolean;
		onFocus: () => void;
		ar?: boolean;
		color?: string;
		splitView?: boolean;
		vaultTrees?: Record<string, any[]>;
		allTags?: string[];
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

	// Note names for autocomplete
	const noteNames = $derived.by(() => {
		if (!vaultId || !vaultTrees[vaultId]) return [];
		return collectNoteNames(vaultTrees[vaultId]);
	});

	// Edit mode state
	let editBody = $state('');
	let saveTimeout: ReturnType<typeof setTimeout>;
	let saving = $state(false);
	let prevTabId = $state('');
	let contentEl: HTMLDivElement | undefined;

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

	// Post-process rendered content (math, mermaid, callout toggles)
	$effect(() => {
		if (!editing && noteBody && contentEl) {
			// Need to wait for DOM update
			requestAnimationFrame(() => {
				if (contentEl) postProcessRenderedContent(contentEl);
			});
		}
	});

	function handleEditorChange(newValue: string) {
		editBody = newValue;
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

	async function handleExportHTML() {
		if (!tab || !contentEl) return;
		const htmlContent = `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>${tab.name}</title>
<style>
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif; max-width: 700px; margin: 40px auto; padding: 0 20px; line-height: 1.7; color: #1f2328; }
h1, h2, h3, h4, h5, h6 { margin-top: 1.5em; }
code { background: #f0f0f4; padding: 2px 4px; border-radius: 3px; font-size: 0.9em; }
pre { background: #f6f6f9; padding: 12px; border-radius: 6px; overflow-x: auto; }
pre code { background: none; padding: 0; }
blockquote { border-left: 3px solid #7c3aed; margin: 0; padding: 4px 16px; color: #5c5c66; }
img { max-width: 100%; }
a { color: #7c3aed; }
mark { background: #fff3a3; padding: 1px 2px; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid #e0e0e4; padding: 6px 10px; text-align: start; }
th { background: #f6f6f9; font-weight: 600; }
.task-list-item { list-style: none; }
</style>
</head>
<body>
${contentEl.innerHTML}
</body>
</html>`;
		const blob = new Blob([htmlContent], { type: 'text/html' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `${tab.name}.html`;
		a.click();
		URL.revokeObjectURL(url);
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
					<button class="bc-edit-btn" onclick={handleExportHTML} title={ar ? 'تصدير HTML' : 'Export as HTML'}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
				</button>
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
				<CodeMirrorEditor
					value={editBody}
					dir={noteDir}
					placeholder={ar ? 'اكتب هنا...' : 'Start writing...'}
					onchange={handleEditorChange}
					{noteNames}
					{allTags}
					{ar}
				/>
			{:else}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="note-content" bind:this={contentEl} onclick={handleNoteContentClick}>
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
	.pane.focused { box-shadow: inset 0 0 0 2px hsla(var(--accent-h), var(--accent-s), var(--accent-l), 0.2); }

	.pane-tab-bar {
		display: flex; align-items: flex-end;
		background: var(--background-secondary-alt); border-bottom: 1px solid var(--background-modifier-border);
		padding: 12px 4px 0; flex-shrink: 0;
	}
	.pane-tab {
		position: relative;
		display: inline-flex; align-items: center; gap: 6px;
		background: var(--background-primary); color: var(--text-normal);
		border: 1px solid var(--background-modifier-border);
		border-top: 3px solid var(--vault-color, var(--interactive-accent));
		border-bottom: 1px solid var(--background-primary);
		margin-bottom: -1px;
		border-radius: 6px 6px 0 0;
		padding: 5px 10px;
		font-size: 0.8rem;
	}
	.pane-tab-vault {
		position: absolute; bottom: 100%; inset-inline-end: 8px;
		font-size: 0.55rem; line-height: 1.3; letter-spacing: 0.02em;
		color: var(--text-normal);
		background: var(--background-secondary-alt);
		padding: 0 5px;
		border-radius: 3px 3px 0 0;
		border: 1px solid var(--background-modifier-border); border-bottom: none;
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
		padding: 4px 16px; border-bottom: 1px solid var(--background-secondary-alt);
		font-size: 0.78rem; color: var(--text-faint); flex-shrink: 0;
		display: flex; align-items: center; min-height: 28px;
	}
	.bc-vault { color: var(--text-muted); }
	.bc-sep { margin: 0 4px; color: var(--background-modifier-border-focus); }
	.bc-note { color: var(--text-normal); }
	.bc-actions { margin-inline-start: auto; display: flex; align-items: center; gap: 4px; }
	.bc-saving { font-size: 0.7rem; color: var(--interactive-accent); }
	.bc-edit-btn {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer;
	}
	.bc-edit-btn:hover { background: var(--background-modifier-border); color: var(--text-normal); }
	.bc-edit-btn.active { color: var(--interactive-accent); }

	.note-scroll {
		flex: 1; overflow-y: auto; padding: 1.5rem 3rem; max-width: 800px;
		font-size: var(--vault-font-size, 0.95rem);
		font-family: var(--vault-text-font, inherit);
		display: flex; flex-direction: column;
	}

	.props-divider { border: none; border-top: 1px solid var(--background-modifier-border-focus); margin: 12px 0; }

	.note-content { line-height: 1.8; color: var(--text-normal); flex: 1; }

	/* ─── Headings ─── */
	.note-content :global(h1) { font-size: 1.8rem; margin: 1.5rem 0 0.75rem; color: var(--text-normal); }
	.note-content :global(h2) { font-size: 1.4rem; margin: 1.3rem 0 0.5rem; }
	.note-content :global(h3) { font-size: 1.15rem; margin: 1rem 0 0.4rem; }
	.note-content :global(h4) { font-size: 1.05rem; margin: 0.8rem 0 0.3rem; }
	.note-content :global(h5) { font-size: 0.95rem; margin: 0.6rem 0 0.2rem; }
	.note-content :global(h6) { font-size: 0.85rem; margin: 0.5rem 0 0.2rem; color: var(--text-muted); }
	.note-content :global(p) { margin: 0.5rem 0; }

	/* ─── Links ─── */
	.note-content :global(a) { color: var(--vault-accent, var(--interactive-accent)); }
	.note-content :global(a.wikilink) {
		color: var(--vault-accent, var(--interactive-accent));
		text-decoration: none;
		border-bottom: 1px dashed color-mix(in srgb, var(--vault-accent, var(--interactive-accent)) 40%, transparent);
		cursor: pointer;
	}
	.note-content :global(a.wikilink:hover) {
		border-bottom-color: var(--vault-accent, var(--interactive-accent));
	}

	/* ─── Code ─── */
	.note-content :global(code) { background: var(--background-secondary-alt); padding: 0.15em 0.35em; border-radius: 3px; font-size: 0.9em; }
	.note-content :global(pre) { background: var(--background-secondary); border: 1px solid var(--background-modifier-border); border-radius: 6px; padding: 1rem; overflow-x: auto; }
	.note-content :global(pre code) { background: none; padding: 0; font-size: 0.85rem; line-height: 1.6; }

	/* ─── Blockquote & Lists ─── */
	.note-content :global(blockquote) { border-inline-start: 3px solid var(--vault-accent, var(--interactive-accent)); padding: 0.25rem 1rem; margin: 0.5rem 0; color: var(--text-muted); }
	.note-content :global(ul), .note-content :global(ol) { padding-inline-start: 1.5rem; }
	.note-content :global(li) { margin: 0.2rem 0; }
	.note-content :global(hr) { border: none; border-top: 1px solid var(--background-modifier-border); margin: 1.5rem 0; }

	/* ─── Table ─── */
	.note-content :global(table) { border-collapse: collapse; width: 100%; margin: 0.75rem 0; }
	.note-content :global(th), .note-content :global(td) { border: 1px solid var(--background-modifier-border); padding: 0.4rem 0.7rem; text-align: start; }
	.note-content :global(th) { background: var(--background-secondary); }

	/* ─── Media ─── */
	.note-content :global(img) { max-width: 100%; border-radius: 4px; }
	.note-content :global(input[type="checkbox"]) { margin-inline-end: 0.4rem; }
	.note-content :global(strong) { font-weight: 600; }

	/* ─── Highlights ─── */
	.note-content :global(mark) {
		background: color-mix(in srgb, var(--color-yellow) 35%, transparent);
		padding: 0.1em 0.2em;
		border-radius: 2px;
	}

	/* ─── Callouts ─── */
	.note-content :global(.callout) {
		border: 1px solid var(--background-modifier-border);
		border-inline-start: 4px solid var(--callout-color, var(--color-blue));
		border-radius: 4px;
		margin: 0.75rem 0;
		background: color-mix(in srgb, var(--callout-color, var(--color-blue)) 5%, transparent);
		overflow: hidden;
	}
	.note-content :global(.callout-title) {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		font-weight: 600;
		font-size: 0.9rem;
		color: var(--callout-color, var(--color-blue));
		cursor: default;
	}
	.note-content :global(.callout-foldable .callout-title) {
		cursor: pointer;
	}
	.note-content :global(.callout-fold) {
		transition: transform 0.15s ease;
		display: inline-block;
		font-size: 0.7rem;
	}
	.note-content :global(.callout:not(.callout-collapsed) .callout-fold) {
		transform: rotate(90deg);
	}
	.note-content :global(.callout-icon) {
		font-size: 1rem;
	}
	.note-content :global(.callout-content) {
		padding: 0 12px 8px;
		font-size: 0.9rem;
	}
	.note-content :global(.callout-collapsed .callout-content) {
		display: none;
	}

	/* ─── Math ─── */
	.note-content :global(.math-inline) {
		font-family: 'KaTeX_Main', serif;
	}
	.note-content :global(.math-block) {
		text-align: center;
		margin: 1rem 0;
		overflow-x: auto;
	}
	.note-content :global(.math-rendered) {
		font-family: inherit;
	}

	/* ─── Mermaid ─── */
	.note-content :global(.mermaid-container) {
		margin: 1rem 0;
		text-align: center;
		overflow-x: auto;
	}
	.note-content :global(.mermaid-rendered) {
		background: none;
	}

	/* ─── Footnotes ─── */
	.note-content :global(.footnote-ref a) {
		color: var(--vault-accent, var(--interactive-accent));
		text-decoration: none;
		font-weight: 600;
	}
	.note-content :global(.footnotes) {
		margin-top: 2rem;
		font-size: 0.85rem;
		color: var(--text-muted);
	}
	.note-content :global(.footnotes hr) {
		margin-bottom: 1rem;
	}
	.note-content :global(.footnote-backref) {
		text-decoration: none;
		color: var(--vault-accent, var(--interactive-accent));
	}

	/* ─── Embeds ─── */
	.note-content :global(.embed-note) {
		border: 1px solid var(--background-modifier-border);
		border-inline-start: 3px solid var(--vault-accent, var(--interactive-accent));
		border-radius: 4px;
		padding: 8px 12px;
		margin: 0.5rem 0;
		background: var(--background-primary-alt);
		font-size: 0.9rem;
		cursor: pointer;
	}
	.note-content :global(.embed-note:hover) {
		background: var(--background-secondary-alt);
	}
	.note-content :global(.embed-icon) {
		margin-inline-end: 4px;
	}

	/* ─── Highlight.js Code Theme ─── */
	.note-content :global(.hljs-keyword) { color: var(--code-keyword); }
	.note-content :global(.hljs-string) { color: var(--code-string); }
	.note-content :global(.hljs-number) { color: var(--code-number); }
	.note-content :global(.hljs-comment) { color: var(--code-comment); font-style: italic; }
	.note-content :global(.hljs-function) { color: var(--code-function); }
	.note-content :global(.hljs-title) { color: var(--code-function); }
	.note-content :global(.hljs-built_in) { color: var(--code-builtin); }
	.note-content :global(.hljs-type) { color: var(--code-type); }
	.note-content :global(.hljs-attr) { color: var(--code-attr); }
	.note-content :global(.hljs-literal) { color: var(--code-attr); }
	.note-content :global(.hljs-meta) { color: var(--code-meta); }
	.note-content :global(.hljs-tag) { color: var(--code-tag); }
	.note-content :global(.hljs-name) { color: var(--code-tag); }
	.note-content :global(.hljs-attribute) { color: var(--code-keyword); }
	.note-content :global(.hljs-selector-class) { color: var(--code-keyword); }
	.note-content :global(.hljs-selector-id) { color: var(--code-function); }
	.note-content :global(.hljs-variable) { color: var(--code-variable); }

	.pane-empty {
		flex: 1; display: flex; align-items: center; justify-content: center;
		color: var(--color-base-40); font-size: 0.85rem;
	}
</style>

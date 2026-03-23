<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { parseFrontmatter, extractHeadings, editingTabIds, toggleEditMode, saveTabContent, resolveWikilinkCrossLibrary, openNoteTab, openTabs, libraryAppearances, libraries, navigateBack, navigateForward, readNote, appSettings, updateSettings, renameItem } from '$lib/libraries/store';
	import type { OpenTab } from '$lib/libraries/store';
	import { detectDir, renderMarkdown, postProcessRenderedContent, collectNoteNames } from '$lib/utils';
	import { dir, t } from '$lib/i18n';
	import { get } from 'svelte/store';
	import PropertyEditor from './PropertyEditor.svelte';
	import CodeMirrorEditor from './CodeMirrorEditor.svelte';
	import BaseView from './BaseView.svelte';
	import type { BaseDefinition } from '$lib/bases/types';

	let {
		tab,
		isFocused = false,
		onFocus,
		color = '#7c3aed',
		splitView = false,
		libraryTrees = {} as Record<string, any[]>,
		allTags = [] as string[],
		allNotes = [] as { name: string; path: string; libraryName: string }[],
		libraryColorMap = {} as Record<string, string>,
		onCreateNote,
		onQuickSwitch,
		onCloseTab,
	}: {
		tab: OpenTab | null;
		isFocused?: boolean;
		onFocus: () => void;
		color?: string;
		splitView?: boolean;
		libraryTrees?: Record<string, any[]>;
		allTags?: string[];
		allNotes?: { name: string; path: string; libraryName: string }[];
		libraryColorMap?: Record<string, string>;
		onCreateNote?: () => void;
		onQuickSwitch?: () => void;
		onCloseTab?: () => void;
	} = $props();

	const hasHistory = $derived(tab ? (tab.history?.length ?? 0) > 1 : false);
	const canGoBack = $derived(tab ? (tab.historyIndex ?? 0) > 0 : false);
	const canGoForward = $derived(tab ? (tab.historyIndex ?? 0) < ((tab.history?.length ?? 1) - 1) : false);
	const isEmptyTab = $derived(tab ? !tab.path : false);
	const isBaseFile = $derived(tab?.path?.endsWith('.base') ?? false);

	// Parse .base file content into BaseDefinition
	const baseDefinition: BaseDefinition | null = $derived.by(() => {
		if (!isBaseFile || !tab?.content) return null;
		try {
			return JSON.parse(tab.content) as BaseDefinition;
		} catch {
			return null;
		}
	});

	const parsed = $derived(tab ? parseFrontmatter(tab.content) : null);
	const properties = $derived(parsed?.properties ?? []);
	const noteBody = $derived(parsed?.body ?? '');
	const noteDir = $derived(noteBody ? detectDir(noteBody) : $dir);
	const editing = $derived(tab ? $editingTabIds.has(tab.id) : false);
	let livePreviewEnabled = $state(true);

	// Library appearance
	const libraryId = $derived(tab ? get(libraries).find(v => tab!.libraryPath === v.path)?.id : null);
	const appearance = $derived(libraryId ? $libraryAppearances[libraryId] : null);
	const paneStyle = $derived.by(() => {
		if (!appearance) return '';
		const vars: string[] = [];
		if (appearance.accent_color) vars.push(`--library-accent: ${appearance.accent_color}`);
		if (appearance.base_font_size) vars.push(`--library-font-size: ${appearance.base_font_size}px`);
		if (appearance.text_font_family) vars.push(`--library-text-font: ${appearance.text_font_family}`);
		if (appearance.monospace_font_family) vars.push(`--library-mono-font: ${appearance.monospace_font_family}`);
		return vars.join('; ');
	});

	// Note names for autocomplete (cross-library)
	const noteNames = $derived(allNotes.map(n => ({ name: n.name, path: n.path, libraryName: n.libraryName })));

	// Edit mode state
	let editBody = $state('');
	let saveTimeout: ReturnType<typeof setTimeout>;
	let saving = $state(false);
	let propsCollapsed = $state(false);
	let noteWidth = $state(100); // percentage 50-100
	let rafId: number | null = null;
	let rafId2: number | null = null;

	function handleToggleLivePreview() { livePreviewEnabled = !livePreviewEnabled; }

	onMount(() => {
		document.addEventListener('constellation:toggle-live-preview', handleToggleLivePreview);
	});

	onDestroy(() => {
		// Flush any pending save before the component is destroyed
		if (saveTimeout) {
			clearTimeout(saveTimeout);
			if (tab) {
				const currentParsed = parseFrontmatter(tab.content);
				saveTabContent(tab.id, tab.path, currentParsed.properties, editBody).catch((e) => console.error('[NotePane] Flush save failed:', e));
			}
		}
		if (rafId !== null) cancelAnimationFrame(rafId);
		if (rafId2 !== null) cancelAnimationFrame(rafId2);
		document.removeEventListener('constellation:toggle-live-preview', handleToggleLivePreview);
	});
	let prevTabId = $state('');
	let contentEl: HTMLDivElement | undefined;
	let titleInputEl: HTMLInputElement | undefined;

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

	// Post-process rendered content (math, mermaid, callout toggles, embeds)
	$effect(() => {
		if (!editing && noteBody && contentEl) {
			// Cancel any pending RAF from previous render
			if (rafId !== null) cancelAnimationFrame(rafId);
			// Need to wait for DOM update
			rafId = requestAnimationFrame(async () => {
				rafId = null;
				if (contentEl) {
					await postProcessRenderedContent(contentEl);
					await processEmbeds(contentEl, 0);
				}
			});
		}
	});

	// Highlight index term in reading mode
	const highlightTerm = $derived(tab?.highlightTerm);
	$effect(() => {
		if (!editing && highlightTerm && contentEl) {
			if (rafId2 !== null) cancelAnimationFrame(rafId2);
			rafId2 = requestAnimationFrame(() => {
				rafId2 = null;
				if (!contentEl) return;
				highlightTermInContent(contentEl, highlightTerm);
				// Scroll to the first highlighted match
				const first = contentEl.querySelector('.index-hl');
				if (first) first.scrollIntoView({ behavior: 'smooth', block: 'center' });
				// Clear the highlightTerm so it doesn't re-trigger
				if (tab) {
					openTabs.update(tabs => tabs.map(t => t.id === tab!.id ? { ...t, highlightTerm: undefined } : t));
				}
			});
		}
	});

	function highlightTermInContent(container: HTMLElement, term: string) {
		// Remove any previous index highlights
		container.querySelectorAll('.index-hl').forEach(el => {
			const parent = el.parentNode;
			if (parent) {
				parent.replaceChild(document.createTextNode(el.textContent ?? ''), el);
				parent.normalize();
			}
		});
		// Walk text nodes and wrap matches
		const regex = new RegExp(`(${term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
		const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT);
		const matches: { node: Text; index: number; length: number }[] = [];
		let node: Text | null;
		while ((node = walker.nextNode() as Text | null)) {
			let match: RegExpExecArray | null;
			regex.lastIndex = 0;
			while ((match = regex.exec(node.textContent ?? '')) !== null) {
				matches.push({ node, index: match.index, length: match[0].length });
			}
		}
		// Apply highlights in reverse to preserve indices
		for (let i = matches.length - 1; i >= 0; i--) {
			const { node: textNode, index, length } = matches[i];
			const range = document.createRange();
			range.setStart(textNode, index);
			range.setEnd(textNode, index + length);
			const mark = document.createElement('mark');
			mark.className = 'index-hl';
			range.surroundContents(mark);
		}
	}

	function handleEditorChange(newValue: string) {
		editBody = newValue;
		debouncedSaveBody();
	}

	function debouncedSaveBody() {
		clearTimeout(saveTimeout);
		saveTimeout = setTimeout(async () => {
			if (!tab) return;
			const currentParsed = parseFrontmatter(tab.content);
			await saveTabContent(tab.id, tab.path, currentParsed.properties, editBody);
		}, 800);
	}


	// Process note embeds (![[note]])
	async function processEmbeds(container: HTMLElement, depth: number) {
		if (depth >= 3) return; // Max nesting depth
		const embedEls = container.querySelectorAll('.embed-note:not(.embed-loaded)');
		for (const el of embedEls) {
			const embedTarget = decodeURIComponent(el.getAttribute('data-embed') || '');
			if (!embedTarget || !tab) continue;

			// Strip fragment
			const hashIdx = embedTarget.indexOf('#');
			const noteTarget = hashIdx >= 0 ? embedTarget.slice(0, hashIdx) : embedTarget;
			const fragment = hashIdx >= 0 ? embedTarget.slice(hashIdx + 1) : null;

			try {
				const resolved = await resolveWikilinkCrossLibrary(tab.libraryPath, noteTarget);
				if (!resolved) continue;

				const content = await readNote(resolved.path);
				// Strip frontmatter
				let body = content;
				if (body.startsWith('---')) {
					const end = body.indexOf('\n---', 3);
					if (end >= 0) body = body.slice(end + 4).trim();
				}

				// If fragment is a heading, embed only that section
				if (fragment && !fragment.startsWith('^')) {
					const sectionRegex = new RegExp(`^(#{1,6})\\s+${fragment.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*$`, 'im');
					const match = sectionRegex.exec(body);
					if (match) {
						const level = match[1].length;
						const startIdx = match.index;
						// Find next heading of same or higher level
						const rest = body.slice(startIdx + match[0].length);
						const nextHeading = rest.search(new RegExp(`^#{1,${level}}\\s`, 'm'));
						body = nextHeading >= 0
							? body.slice(startIdx, startIdx + match[0].length + nextHeading).trim()
							: body.slice(startIdx).trim();
					}
				}

				const html = renderMarkdown(body);
				el.innerHTML = html;
				el.classList.add('embed-loaded');

				// Recursively process nested embeds
				await postProcessRenderedContent(el as HTMLElement);
				await processEmbeds(el as HTMLElement, depth + 1);
			} catch { /* ignore failed embeds */ }
		}
	}

	// WikiLink click handler via event delegation
	async function handleNoteContentClick(e: MouseEvent) {
		const target = e.target as HTMLElement;

		// Dataview inline link click
		const dvLink = target.closest('a.dv-inline-link') as HTMLAnchorElement | null;
		if (dvLink) {
			e.preventDefault();
			const path = dvLink.dataset.path;
			const lib = dvLink.dataset.library;
			if (path && lib) {
				const vc = libraryColorMap[lib] ?? '#7c3aed';
				await openNoteTab(path, lib, vc);
			}
			return;
		}

		const wikilinkEl = target.closest('a.wikilink') as HTMLAnchorElement | null;
		if (!wikilinkEl || !tab) return;

		e.preventDefault();
		const linkTarget = decodeURIComponent(wikilinkEl.dataset.wikilink ?? '');
		const fragment = wikilinkEl.dataset.fragment ? decodeURIComponent(wikilinkEl.dataset.fragment) : null;
		if (!linkTarget && !fragment) return;

		// Same-note fragment link (e.g. [[#heading]])
		if (!linkTarget && fragment) {
			scrollToFragment(fragment);
			return;
		}

		const resolved = await resolveWikilinkCrossLibrary(tab.libraryPath, linkTarget);
		if (resolved) {
			const newTab = e.ctrlKey || e.metaKey || e.button === 1;
			const libraryColor = libraryColorMap[resolved.library_name] ?? tab.libraryColor;
			await openNoteTab(resolved.path, resolved.library_name, libraryColor, undefined, newTab);
			// Scroll to fragment after note loads
			const frag = fragment || resolved.fragment;
			if (frag) {
				setTimeout(() => scrollToFragment(frag), 150);
			}
		}
	}

	function scrollToFragment(fragment: string) {
		if (!contentEl) return;
		const isBlock = fragment.startsWith('^');
		if (isBlock) {
			// Block reference: find element with matching block-id in text
			const blockId = fragment.slice(1);
			const walker = document.createTreeWalker(contentEl, NodeFilter.SHOW_TEXT);
			while (walker.nextNode()) {
				if (walker.currentNode.textContent?.includes(`^${blockId}`)) {
					(walker.currentNode.parentElement)?.scrollIntoView({ behavior: 'smooth', block: 'center' });
					return;
				}
			}
		} else {
			// Heading reference: find matching heading
			const headings = contentEl.querySelectorAll('h1, h2, h3, h4, h5, h6');
			const target = fragment.toLowerCase().replace(/-/g, ' ');
			for (const h of headings) {
				const text = (h.textContent ?? '').trim().toLowerCase();
				if (text === target || text.replace(/\s+/g, '-') === fragment.toLowerCase()) {
					h.scrollIntoView({ behavior: 'smooth', block: 'start' });
					return;
				}
			}
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pane" class:focused={isFocused} onclick={onFocus} dir={noteDir}>
	{#if tab}
		{#if isEmptyTab}
			<div class="pane-breadcrumb">
				<span class="bc-note">{tab.name.replace(/\.md$/, '')}</span>
			</div>
			<div class="empty-tab">
				<span class="empty-tab-title">{tab.name.replace(/\.md$/, '')}</span>
				{#if onCreateNote}
					<button class="empty-tab-action" onclick={onCreateNote}>
						{$t('notePane.createNote')} <span class="empty-tab-shortcut">(Ctrl + N)</span>
					</button>
				{/if}
				{#if onQuickSwitch}
					<button class="empty-tab-action" onclick={onQuickSwitch}>
						{$t('notePane.goToFile')} <span class="empty-tab-shortcut">(Ctrl + O)</span>
					</button>
				{/if}
				{#if onCloseTab}
					<button class="empty-tab-action" onclick={onCloseTab}>
						{$t('notePane.close')}
					</button>
				{/if}
			</div>
		{:else if splitView}
			<div class="pane-tab-bar" style:--library-color={color}>
				<div class="pane-tab">
					<span class="pane-tab-lib">{tab.libraryName}</span>
					<span class="pane-tab-title">{tab.name.replace(/\.md$/, '')}</span>
				</div>
				<div class="pane-tab-actions">
									</div>
			</div>
		{:else}
			<div class="pane-breadcrumb">
				{#if hasHistory}
					<button class="bc-nav-btn" onclick={() => navigateBack()} disabled={!canGoBack} title="Back (Alt+←)">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
					</button>
					<button class="bc-nav-btn" onclick={() => navigateForward()} disabled={!canGoForward} title="Forward (Alt+→)">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
					</button>
				{/if}
				<span class="bc-lib-name">{tab.libraryName}</span>
				<span class="bc-sep">/</span>
				<span class="bc-note">{tab.name.replace(/\.md$/, '')}</span>
				<div class="bc-actions">
					<div class="bc-width-control" title="Note width: {noteWidth}%">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M21 12H3M21 12l-4-4m4 4l-4 4M3 12l4-4m-4 4l4 4"/>
						</svg>
						<input type="range" class="bc-width-slider" min="50" max="100" step="5" bind:value={noteWidth} />
					</div>
										<button
						class="bc-editor-switch"
						class:source={!livePreviewEnabled}
						onclick={() => livePreviewEnabled = !livePreviewEnabled}
						title={livePreviewEnabled ? 'Source mode' : 'Live Preview'}
					>
						<span class="switch-track">
							<span class="switch-thumb"></span>
						</span>
					</button>
				</div>
			</div>
		{/if}
		{#if isBaseFile && baseDefinition && tab}
			<!-- Base view -->
			<BaseView
				definition={baseDefinition}
				filePath={tab.path}
				onOpenNote={(path, libraryName) => {
					const vc = libraryColorMap[libraryName] ?? color;
					openNoteTab(path, libraryName, vc);
				}}
				onCreateNote={(folderPath, properties) => {
					if (onCreateNote) onCreateNote();
				}}
			/>
		{:else if !isEmptyTab}
		<div class="note-scroll" class:editing dir={noteDir} style="{paneStyle}; max-width: {noteWidth}%">
			{#if tab}
				<input class="note-title" dir="auto" spellcheck="false"
					bind:this={titleInputEl}
					value={tab.name.replace(/\.md$/, '')}
					onfocus={(e) => (e.target as HTMLInputElement).select()}
					onblur={async (e) => {
						const newName = (e.target as HTMLInputElement).value.trim();
						if (newName && newName !== tab.name.replace(/\.md$/, '') && tab.path) {
							const dir = tab.path.substring(0, tab.path.lastIndexOf('/') + 1) || tab.path.substring(0, tab.path.lastIndexOf('\\') + 1);
							const newPath = dir + newName + '.md';
							try { await renameItem(tab.path, newPath); } catch {}
						}
					}}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							e.preventDefault();
							(e.target as HTMLInputElement).blur();
							// Move focus to editor like Obsidian
							const editor = (e.target as HTMLElement).closest('.note-scroll')?.querySelector('.cm-content') as HTMLElement;
							if (editor) editor.focus();
						}
						if (e.key === 'Escape') { (e.target as HTMLInputElement).value = tab.name.replace(/\.md$/, ''); (e.target as HTMLInputElement).blur(); }
					}}
				/>
			{/if}
			{#if tab && $appSettings.propertiesInDocument !== 'hidden'}
				{#if $appSettings.propertiesInDocument === 'source'}
					{#if parsed?.rawYaml}
						<button class="props-toggle" onclick={() => propsCollapsed = !propsCollapsed}>
							<svg class="props-chevron" class:collapsed={propsCollapsed} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
							<span>Properties</span>
						</button>
						{#if !propsCollapsed}
							<pre class="props-source">{parsed.rawYaml}</pre>
						{/if}
						<hr class="props-divider"/>
					{/if}
				{:else}
					<PropertyEditor
						properties={properties}
						body={noteBody}
						tabId={tab.id}
						filePath={tab.path}
						libraryName={tab.libraryName}
						collapsed={propsCollapsed}
						onToggle={() => propsCollapsed = !propsCollapsed}
						onNoteClick={async (noteName) => {
							if (!tab) return;
							const resolved = await resolveWikilinkCrossLibrary(tab.libraryPath, noteName);
							if (resolved) {
								const vc = libraryColorMap[resolved.library_name] ?? color;
								await openNoteTab(resolved.path, resolved.library_name, vc);
							}
						}}
					/>
					{#if properties.length > 0}
						<hr class="props-divider"/>
					{/if}
				{/if}
			{/if}
			{#if editing}
				<CodeMirrorEditor
					value={editBody}
					dir={noteDir}
					placeholder={$t('notePane.placeholder')}
					onchange={handleEditorChange}
					{noteNames}
					{allTags}
					livePreview={livePreviewEnabled}
					showLineNumbers={$appSettings.showLineNumbers}
					foldHeading={$appSettings.foldHeading}
					foldIndent={$appSettings.foldIndent}
					indentationGuides={$appSettings.indentationGuides}
					indentWithTabs={$appSettings.indentWithTabs}
					tabSize={$appSettings.tabSize}
					autoPairMarkdown={$appSettings.autoPairMarkdown}
					initialCursorPos={tab?.cursorPos ?? 0}
					initialScrollTop={tab?.scrollTop ?? 0}
					onCursorChange={(pos) => { if (tab) tab.cursorPos = pos; }}
					onScrollChange={(top) => { if (tab) tab.scrollTop = top; }}
				/>
			{:else}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="note-content" bind:this={contentEl} onclick={handleNoteContentClick}>
					{@html renderMarkdown(noteBody)}
				</div>
			{/if}
		</div>
		{/if}
	{:else}
		<div class="pane-empty">
			{$t('notePane.selectNote')}
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
		border-top: 3px solid var(--library-color, var(--interactive-accent));
		border-bottom: 1px solid var(--background-primary);
		margin-bottom: -1px;
		border-radius: 6px 6px 0 0;
		padding: 5px 10px;
		font-size: 0.8rem;
	}
	.pane-tab-lib {
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
	.bc-lib-name { color: var(--text-muted); }
	.bc-sep { margin: 0 4px; color: var(--background-modifier-border-focus); }
	.bc-note { color: var(--text-normal); }
	.bc-actions { margin-inline-start: auto; display: flex; align-items: center; gap: 4px; position: relative; }
	.bc-width-control { display: flex; align-items: center; gap: 6px; color: var(--text-muted); padding: 0 4px; }
	.bc-width-slider { width: 80px; height: 4px; accent-color: var(--interactive-accent); cursor: pointer; }
	.bc-nav-btn {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer; flex-shrink: 0;
	}
	.bc-nav-btn:hover:not(:disabled) { background: var(--background-modifier-hover); color: var(--text-normal); }
	.bc-nav-btn:disabled { opacity: 0.3; cursor: default; }
	:global([dir="rtl"]) .bc-nav-btn svg { transform: scaleX(-1); }
	.bc-saving { font-size: 0.7rem; color: var(--interactive-accent); }
	.bc-edit-btn {
		width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
		border: none; background: none; border-radius: 3px; color: var(--text-faint); cursor: pointer;
	}
	.bc-edit-btn:hover { background: var(--background-modifier-border); color: var(--text-normal); }

	.bc-editor-switch {
		display: flex; align-items: center; padding: 0; border: none;
		background: none; cursor: pointer;
	}
	.switch-track {
		width: 28px; height: 16px; border-radius: 8px; position: relative;
		background: var(--interactive-accent);
		box-shadow: inset 0 1px 3px rgba(0,0,0,0.15);
		transition: background 0.25s;
	}
	.bc-editor-switch.source .switch-track { background: var(--text-muted); }
	.switch-thumb {
		width: 12px; height: 12px; border-radius: 50%; position: absolute;
		top: 2px; left: 2px;
		background: white;
		box-shadow: 0 1px 2px rgba(0,0,0,0.2);
		transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
	}
	.bc-editor-switch.source .switch-thumb { transform: translateX(12px); }
	.note-scroll {
		flex: 1; overflow-y: auto; padding: 1.5rem 3rem; max-width: 100%; align-self: center; width: 100%;
		font-size: var(--library-font-size, 0.95rem);
		font-family: var(--library-text-font, inherit);
		display: flex; flex-direction: column;
	}
	.note-scroll.editing {
		overflow: hidden;
	}
	.note-scroll :global(.cm-editor) {
		font-family: var(--library-text-font, var(--font-text-theme, inherit));
	}
	.note-scroll :global(.cm-editor .cm-content) {
		font-family: var(--library-text-font, var(--font-text-theme, inherit)) !important;
		font-size: var(--library-font-size, var(--font-text-size, 0.95rem)) !important;
	}
	.note-scroll :global(.cm-editor .cm-line) {
		font-family: inherit;
	}

	.props-toggle {
		display: flex; align-items: center; gap: 4px;
		background: none; border: 1px solid var(--background-modifier-border);
		border-radius: 6px; padding: 4px 12px;
		font-size: 0.78rem; color: var(--text-muted); cursor: pointer;
		width: 100%;
		transition: background 0.15s;
	}
	.props-toggle:hover { background: var(--background-modifier-hover); }
	.props-chevron { transition: transform 0.2s; flex-shrink: 0; }
	.props-chevron.collapsed { transform: rotate(-90deg); }
	:global([dir="rtl"]) .props-chevron.collapsed { transform: rotate(90deg); }

	.props-divider { border: none; border-top: 1px solid var(--background-modifier-border-focus); margin: 12px 0; }
	.props-source {
		font-family: var(--font-monospace, 'Fira Code', monospace);
		font-size: 0.82rem;
		background: var(--background-secondary);
		color: var(--text-muted);
		padding: 12px 16px;
		border-radius: 6px;
		margin: 8px 16px;
		white-space: pre-wrap;
		word-break: break-word;
		border: 1px solid var(--background-modifier-border);
	}

	.note-title {
		display: block; width: 100%; box-sizing: border-box;
		font-size: 1.8rem; font-weight: 700; margin: 0 0 0.5rem;
		color: var(--text-normal); line-height: 1.3;
		outline: none; border: none; border-radius: 4px;
		padding: 2px 4px; margin-inline: -4px;
		background: transparent; font-family: inherit;
		transition: background 0.15s;
	}
	.note-title:hover { background: var(--background-modifier-hover); }
	.note-title:focus { background: var(--background-secondary); }
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
	.note-content :global(a) { color: var(--library-accent, var(--interactive-accent)); }
	.note-content :global(a.wikilink) {
		color: var(--library-accent, var(--interactive-accent));
		text-decoration: none;
		border-bottom: 1px dashed color-mix(in srgb, var(--library-accent, var(--interactive-accent)) 40%, transparent);
		cursor: pointer;
	}
	.note-content :global(a.wikilink:hover) {
		border-bottom-color: var(--library-accent, var(--interactive-accent));
	}
	.note-content :global(a.wikilink.cross-library) {
		color: var(--text-accent-hover, #a855f7);
		border-bottom-style: dotted;
	}
	.note-content :global(a.wikilink.cross-library::before) {
		content: '↗';
		font-size: 0.7em;
		margin-inline-end: 2px;
		opacity: 0.6;
	}

	/* ─── Code ─── */
	.note-content :global(code) { background: var(--background-secondary-alt); padding: 0.15em 0.35em; border-radius: 3px; font-size: 0.9em; }
	.note-content :global(pre) { background: var(--background-secondary); border: 1px solid var(--background-modifier-border); border-radius: 6px; padding: 1rem; overflow-x: auto; }
	.note-content :global(pre code) { background: none; padding: 0; font-size: 0.85rem; line-height: 1.6; }

	/* ─── Blockquote & Lists ─── */
	.note-content :global(blockquote) { border-inline-start: 3px solid var(--library-accent, var(--interactive-accent)); padding: 0.25rem 1rem; margin: 0.5rem 0; color: var(--text-muted); }
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
	.note-content :global(mark.index-hl) {
		background: color-mix(in srgb, var(--color-yellow) 60%, transparent);
		padding: 0.1em 0.25em;
		border-radius: 3px;
		outline: 2px solid color-mix(in srgb, var(--color-yellow) 40%, transparent);
		animation: index-pulse 1.5s ease-in-out 2;
	}
	@keyframes index-pulse {
		0%, 100% { outline-color: color-mix(in srgb, var(--color-yellow) 40%, transparent); }
		50% { outline-color: color-mix(in srgb, var(--color-yellow) 80%, transparent); }
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
		color: var(--library-accent, var(--interactive-accent));
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
		color: var(--library-accent, var(--interactive-accent));
	}

	/* ─── Embeds ─── */
	.note-content :global(.embed-note) {
		border: 1px solid var(--background-modifier-border);
		border-inline-start: 3px solid var(--library-accent, var(--interactive-accent));
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

	/* ─── Dataview Inline Results ─── */
	.note-content :global(.dataview-query) {
		margin: 12px 0;
		border: 1px solid var(--background-modifier-border);
		border-radius: 6px;
		overflow: hidden;
		font-size: 13px;
	}
	.note-content :global(.dataview-source) {
		display: none;
	}
	.note-content :global(.dv-inline-loading) {
		padding: 16px;
		text-align: center;
		color: var(--text-muted);
		font-size: 12px;
	}
	.note-content :global(.dv-inline-error) {
		padding: 12px 16px;
		color: var(--text-error);
		background: rgba(255, 0, 0, 0.05);
		font-size: 12px;
		border-radius: 4px;
	}
	.note-content :global(.dv-inline-empty) {
		padding: 16px;
		text-align: center;
		color: var(--text-muted);
		font-size: 12px;
	}
	.note-content :global(.dv-inline-table-wrap) {
		overflow-x: auto;
	}
	.note-content :global(.dv-inline-table) {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}
	.note-content :global(.dv-inline-table th) {
		text-align: start;
		padding: 6px 10px;
		font-weight: 600;
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.3px;
		border-bottom: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		position: sticky;
		top: 0;
	}
	.note-content :global(.dv-inline-table td) {
		padding: 5px 10px;
		border-bottom: 1px solid var(--background-modifier-border-hover, rgba(0,0,0,0.04));
		color: var(--text-normal);
	}
	.note-content :global(.dv-inline-table tr:hover td) {
		background: var(--background-modifier-hover);
	}
	.note-content :global(.dv-inline-link) {
		color: var(--interactive-accent);
		cursor: pointer;
		text-decoration: none;
		background: none;
		border: none;
		font-size: inherit;
		padding: 0;
	}
	.note-content :global(.dv-inline-link:hover) {
		text-decoration: underline;
	}
	.note-content :global(.dv-inline-list) {
		list-style: none;
		padding: 8px 12px;
		margin: 0;
	}
	.note-content :global(.dv-inline-list li) {
		padding: 3px 0;
	}
	.note-content :global(.dv-inline-footer) {
		padding: 4px 10px;
		font-size: 11px;
		color: var(--text-faint);
		border-top: 1px solid var(--background-modifier-border);
		background: var(--background-secondary);
		text-align: end;
	}

	.empty-tab {
		flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px;
	}
	.empty-tab-title {
		font-size: 0.85rem; color: var(--text-muted); margin-bottom: 16px;
	}
	.empty-tab-action {
		background: none; border: none; cursor: pointer; font-family: inherit;
		font-size: 0.9rem; color: var(--interactive-accent); padding: 6px 12px;
		border-radius: 4px;
	}
	.empty-tab-action:hover { background: var(--background-modifier-hover); }
	.empty-tab-shortcut { color: var(--text-faint); font-size: 0.8rem; }

	.pane-empty {
		flex: 1; display: flex; align-items: center; justify-content: center;
		color: var(--color-base-40); font-size: 0.85rem;
	}
</style>

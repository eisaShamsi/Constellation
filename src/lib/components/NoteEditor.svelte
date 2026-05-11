<script lang="ts">
	/**
	 * NoteEditor — shared wrapper around NotePane.
	 *
	 * Accepts a tab-like object and handles ALL prop extraction, save/flush/rename
	 * callbacks, and stage promotion internally. Every place that needs a note editor
	 * (main window, split view, index preview, second screen, dashboard) uses this
	 * component with one line. The "winning" pattern lives here — tested once, used everywhere.
	 */
	import { invoke } from '@tauri-apps/api/core';
	import { dir } from '$lib/i18n';
	import {
		parseFrontmatter, buildFullContent,
		writeNote, markRecentWrite, setWriteAhead, clearWriteAhead,
		renameItem, openTabs, openNoteTab,
		resolveWikilinkCrossLibrary,
		createNote, buildDefaultFrontmatter, appSettings, libraries,
		isCascading,
		type FrontmatterProperty
	} from '$lib/libraries/store';
	import { broadcastNoteSaved } from '$lib/secondScreen';
	import { buildLibraryColorMap } from '$lib/libraries/colors';
	import { detectDir } from '$lib/utils';
	import { get } from 'svelte/store';
	import NotePane from './NotePane.svelte';

	/** Minimal tab shape — any object with these fields works */
	interface TabLike {
		id: string;
		path: string;
		content: string;
		name: string;
		libraryName: string;
		libraryPath?: string;
		libraryColor?: string;
		cursorPos?: number;
		scrollTop?: number;
		historyIndex?: number;
		history?: string[];
		highlightTerm?: string;
		/** §3-redo.4 — incremented by reloadTabsFromDisk after the cascade
		 *  rewrites this tab's file. Folded into the `{#key}` so NotePane
		 *  destroys + remounts with fresh disk content. Per Concept Paper
		 *  D6, recreate is the safe primitive against BUG-015. */
		reloadVersion?: number;
	}

	let {
		tab,
		noteNames = [] as { name: string; path: string; libraryName?: string }[],
		allTags = [] as string[],
		trail = '',
		trailIndex = 0,
		trailTotal = 0,
		onTrailPrev,
		onTrailNext,
		onnavigateback,
		onnavigateforward,
		onmoreaction,
		onStageChanged,
		linkTraversalMap,
	}: {
		tab: TabLike;
		noteNames?: { name: string; path: string; libraryName?: string }[];
		allTags?: string[];
		trail?: string;
		trailIndex?: number;
		trailTotal?: number;
		onTrailPrev?: () => void;
		onTrailNext?: () => void;
		onnavigateback?: () => void;
		onnavigateforward?: () => void;
		onmoreaction?: (action: string) => void;
		onStageChanged?: (path: string, stage: string) => void;
		linkTraversalMap?: Map<string, number>;
	} = $props();

	// Internal derived state — recalculated when tab changes OR when the
	// openTabs store entry for this tab changes. The store dereference is
	// what makes promote/demote (which mutates ct.content + openTabs.update)
	// propagate into PropertyEditor: without it, $derived only watches the
	// `tab` prop reference, not its mutable `content` field, so Svelte 5
	// doesn't see the post-promote update — the breadcrumb advanced (local
	// state) but Properties stayed stale (BUG-019, MIG-014 §2D Boss test).
	let parsed = $derived.by(() => {
		const ct = $openTabs.find(x => x.id === tab.id);
		return parseFrontmatter(ct?.content ?? tab.content ?? '');
	});
	let body = $derived(parsed.body);
	let noteDir = $derived(detectDir(body) || $dir);
	let stage = $derived(parsed.properties.find((p: FrontmatterProperty) => p.key.toLowerCase() === 'stage')?.value ?? '');

	// Save guard — prevents double saves
	let saving = false;

	/** Re-read the latest tab content from the openTabs store (properties may have changed) */
	function freshProps(): FrontmatterProperty[] {
		const ct = get(openTabs).find(x => x.id === tab.id);
		return ct ? parseFrontmatter(ct.content || '').properties : parsed.properties;
	}
	function freshBody(): string {
		const ct = get(openTabs).find(x => x.id === tab.id);
		return ct ? parseFrontmatter(ct.content || '').body : parsed.body;
	}

	function handlePromote(nextStage: string) {
		// Stage promote/demote writes to disk via writeNote like saveTabContent
		// does, so the same F2 post-cascade-stomp gate applies here too.
		// See `isCascading` for the full rationale.
		if (isCascading(tab.path)) return;
		const props = freshProps();
		const bd = freshBody();
		let newProps: FrontmatterProperty[];
		if (!nextStage) {
			newProps = props.filter(p => p.key.toLowerCase() !== 'stage');
		} else {
			let updated = false;
			newProps = props.map(p => {
				if (p.key.toLowerCase() === 'stage') { updated = true; return { ...p, value: nextStage }; }
				return p;
			});
			if (!updated) newProps.push({ key: 'stage', value: nextStage, type: 'text' as any });
		}
		const fc = buildFullContent(newProps, bd);
		// Update in-store tab if it exists there
		const ct = get(openTabs).find(x => x.id === tab.id);
		if (ct) {
			ct.content = fc;
			openTabs.update(tabs => tabs);
		}
		// Also update the local tab reference
		tab.content = fc;
		markRecentWrite(tab.path);
		writeNote(tab.path, fc).catch(() => {});
		onStageChanged?.(tab.path, nextStage);
	}

	// `filePath` is captured by NotePane at mount and passed back on every
	// save/flush. If it doesn't match the current tab.path, this callback is
	// arriving from an already-destroyed editor whose tab has been repurposed
	// by a wikilink click / Alt+← nav. Using `tab.path` / `freshProps()` at
	// that point would (a) reconstruct content as `current-tab frontmatter
	// + old-tab body` — corruption — and (b) write that corruption to the
	// wrong file on disk, or at minimum poison `setWriteAhead` for the new
	// tab. Bail in that case.
	function handleSave(text: string, filePath: string) {
		if (saving) return;
		if (!filePath || filePath !== tab.path) return;
		if (isCascading(filePath)) return; // see isCascading() — F2 post-cascade-stomp gate
		saving = true;
		const props = freshProps();
		markRecentWrite(filePath);
		const content = buildFullContent(props, text);
		writeNote(filePath, content)
			.then(() => {
				broadcastNoteSaved(filePath);
				// Reindex for search (non-blocking) — updates FTS5, tags, links,
				// and (MIG-002) word_count / created_at / enrichment. Without this
				// call, body edits never re-run index_note: the DB row stays at
				// whatever shape was indexed on initial file creation.
				invoke('constellation_search_reindex', {
					notePath: filePath,
					libraryName: tab.libraryName,
				}).catch(() => {});
				// MIG-021v3 V3-§10.A — CECE background scan on save.
				// When enabled in Settings, fire classifier_suggest_for_note
				// after the disk write completes. This rides the existing
				// 1500ms-debounced save cycle (handleSave is the on-save
				// callback from NotePane's debouncedSaveTimer), so it
				// inherits the same "type stays instant" guarantee — never
				// fires per-keystroke.
				if (get(appSettings).cece?.backgroundScan === 'on_save') {
					invoke('classifier_suggest_for_note', { notePath: filePath })
						.catch((err) => {
							console.warn('[CECE on-save] classifier_suggest_for_note failed:', err);
						});
				}
			})
			.catch(() => {})
			.finally(() => { saving = false; });
	}

	function handleFlush(text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number, filePath: string) {
		if (!filePath || filePath !== tab.path) return;
		// Flush fires on tab close, visibility change, and the {#key}-bump
		// destroy itself — all paths must respect the cascade gate.
		if (isCascading(filePath)) return; // see isCascading() — F2 post-cascade-stomp gate
		const props = freshProps();
		const content = buildFullContent(props, text);
		// Update store tab if present
		const ct = get(openTabs).find(x => x.id === tab.id);
		if (ct) {
			ct.content = content;
			ct.cursorPos = cursorPos;
			ct.scrollTop = scrollTop;
		}
		setWriteAhead(filePath, content, cursorPos, scrollTop);
		if (needsDiskSave) {
			markRecentWrite(filePath);
			writeNote(filePath, content)
				.then(() => {
					clearWriteAhead(filePath);
					broadcastNoteSaved(filePath);
					// Same reindex call as handleSave — flush paths (tab close,
					// window unload, visibility change) must also trigger reindex.
					invoke('constellation_search_reindex', {
						notePath: filePath,
						libraryName: tab.libraryName,
					}).catch(() => {});
				})
				.catch(() => {});
		}
	}

	async function handleTitleChange(newTitle: string, filePath: string) {
		if (!newTitle || !tab.path) return;
		// Same staleness guard as handleSave/handleFlush. If the title-blur
		// event arrives from a NotePane whose mounted file no longer matches
		// the current tab, the tab has been swapped (e.g. by a wikilink
		// click). Firing renameItem with `tab.path` (now the TARGET) and
		// `newTitle` (still the SOURCE's stale title) would rewrite the
		// target's frontmatter title to the source's — the title-leak data
		// corruption bug. Bail in that case.
		if (!filePath || filePath !== tab.path) return;
		const currentName = tab.name.replace(/\.md$/, '');
		if (newTitle === currentName) return;

		// Skip rename if the file doesn't exist (e.g., during initial load)
		const newPath = filePath.replace(/[^/\\]+$/, newTitle + '.md');
		try {
			await renameItem(filePath, newPath);
		} catch (e) {
			// Rename failed — log but don't disrupt the user
			if (String(e).includes('does not exist')) {
				// File might have been moved/renamed externally — silently ignore
			} else {
				console.error('[NoteEditor] Rename failed:', e);
			}
		}
	}

	function handlePropsChange() {
		openTabs.update(tabs => tabs);
	}

	function handleMoreAction(action: string) {
		if (onmoreaction) {
			onmoreaction(action);
		} else {
			// Default handler for common actions
			switch (action) {
				case 'showInExplorer':
					invoke('constellation_show_in_folder', { path: tab.path }).catch(() => {});
					break;
				case 'openDefaultApp':
					invoke('open_path', { path: tab.path }).catch(() => {});
					break;
				case 'copyPath':
					navigator.clipboard.writeText(tab.path).catch(() => {});
					break;
				case 'copyName':
					navigator.clipboard.writeText(tab.name).catch(() => {});
					break;
			}
		}
	}

	async function handleLinkClick(link: string, newTab?: boolean) {
		if (!tab.libraryPath) return;
		try {
			const resolved = await resolveWikilinkCrossLibrary(tab.libraryPath, link);
			if (resolved) {
				const libColors = buildLibraryColorMap(get(libraries));
				await openNoteTab(resolved.path, resolved.library_name, libColors[resolved.library_name] || '#7c3aed', undefined, newTab, tab.path);
			} else {
				// Note doesn't exist — create it in the same folder with default frontmatter
				const folder = tab.path.replace(/[/\\][^/\\]+$/, '');
				const frontmatter = buildDefaultFrontmatter(get(appSettings));
				const newPath = await createNote(folder, link + '.md', frontmatter);
				const libName = tab.libraryName;
				const colors = buildLibraryColorMap([{ name: libName }]);
				await openNoteTab(newPath, libName, colors[libName] || '#7c3aed', undefined, newTab);
			}
		} catch {}
	}
</script>

{#key tab.id + '|' + tab.path + '|' + (tab.reloadVersion ?? 0)}
<NotePane
	value={body}
	title={tab.name.replace(/\.md$/, '')}
	dir={noteDir}
	initialCursorPos={tab.cursorPos ?? 0}
	initialScrollTop={tab.scrollTop ?? 0}
	libraryName={tab.libraryName}
	tabId={tab.id}
	filePath={tab.path}
	libraryPath={tab.libraryPath || ''}
	{noteNames}
	{allTags}
	properties={parsed.properties}
	rawYaml={parsed.rawYaml ?? ''}
	{stage}
	{trail}
	{trailIndex}
	{trailTotal}
	{onTrailPrev}
	{onTrailNext}
	canGoBack={(tab.historyIndex ?? 0) > 0}
	canGoForward={(tab.historyIndex ?? 0) < (tab.history?.length ?? 1) - 1}
	{linkTraversalMap}
	onchange={() => {}}
	onpromote={handlePromote}
	onsave={handleSave}
	onflush={handleFlush}
	ontitlechange={handleTitleChange}
	onpropschange={handlePropsChange}
	onnavigateback={onnavigateback}
	onnavigateforward={onnavigateforward}
	onmoreaction={handleMoreAction}
	highlightTerm={tab.highlightTerm ?? ''}
	onlinkclick={handleLinkClick}
/>
{/key}

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
		createNote, buildDefaultFrontmatter, appSettings,
		type FrontmatterProperty
	} from '$lib/libraries/store';
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
	} = $props();

	// Internal derived state — recalculated when tab changes
	let parsed = $derived(parseFrontmatter(tab.content || ''));
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

	function handleSave(text: string) {
		if (saving) return;
		saving = true;
		const props = freshProps();
		markRecentWrite(tab.path);
		const content = buildFullContent(props, text);
		writeNote(tab.path, content)
			.catch(() => {})
			.finally(() => { saving = false; });
	}

	function handleFlush(text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number) {
		const props = freshProps();
		const content = buildFullContent(props, text);
		// Update store tab if present
		const ct = get(openTabs).find(x => x.id === tab.id);
		if (ct) {
			ct.content = content;
			ct.cursorPos = cursorPos;
			ct.scrollTop = scrollTop;
		}
		// Update local tab
		tab.content = content;
		tab.cursorPos = cursorPos;
		tab.scrollTop = scrollTop;
		setWriteAhead(tab.path, content, cursorPos, scrollTop);
		if (needsDiskSave) {
			markRecentWrite(tab.path);
			writeNote(tab.path, content)
				.then(() => clearWriteAhead(tab.path))
				.catch(() => {});
		}
	}

	function handleTitleChange(newTitle: string) {
		if (newTitle !== tab.name.replace(/\.md$/, '')) {
			renameItem(tab.path, tab.path.replace(/[^/\\]+$/, newTitle + '.md'));
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
				await openNoteTab(resolved.path, resolved.libraryName, resolved.libraryColor || '#7c3aed', undefined, newTab);
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

{#key tab.id + '|' + tab.path}
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

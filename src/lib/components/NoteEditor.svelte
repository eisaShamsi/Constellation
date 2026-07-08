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
		writeNote, markRecentWrite, setWriteAhead, clearWriteAhead, standardSaveEnv,
		renameItem, openTabs, openNoteTab,
		resolveWikilinkCrossLibrary,
		createNote, buildDefaultFrontmatter, appSettings, libraries,
		isCascading,
		type FrontmatterProperty
	} from '$lib/libraries/store';
	import { broadcastNoteSaved } from '$lib/secondScreen';
	// MIG-076 §C — single content ownership. This step (foundation) only
	// MAINTAINS the model (ensure on tab change + live push per edit); the
	// model is not yet READ for seed/save — that swap lands flag-gated next,
	// so the app behaves identically to the §C-1 safe state right now.
	import { ensure as ensureModel, editBody, editProps, seedBody, save as saveNoteSession } from '$lib/editor/noteSession';
	import { compose, getModel } from '$lib/editor/noteModel';
	import { SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';
	import type { Text } from '@codemirror/state';
	import { buildLibraryColorMap } from '$lib/libraries/colors';
	import { detectDir } from '$lib/utils';
	import { get } from 'svelte/store';
	import NotePane from './NotePane.svelte';
	// MIG-043 Phase 1 — show the active note's NSC summary headline in a thin
	// band above the editor so the user has "what is this note about" in
	// context. Cache-first/batched via the shared store; subscribes only when
	// the active note (tab.path) changes — never on every keystroke.
	import { getSummaryFor } from '$lib/nsc/summaryStore';

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
		onTitleRename,
		onLiveStats,
		onLiveProps,
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
		/** MIG-076 §D2 — when the host provides this AND single ownership is on, a
		 *  title rename delegates to the host's full quiesced-rename orchestration
		 *  (handleRenameComplete: freeze overlay + flush-through-model + rename_item
		 *  + wikilink cascade + deliberate remount), closing the orientation-§13
		 *  gap on the §C/§D-safe foundation. Without it (or under rollback) the
		 *  direct-renameItem fallback keeps today's behavior. */
		onTitleRename?: (oldPath: string, newName: string) => void | Promise<void>;
		/** One-way, display-only live-content notification for the host's status-bar
		 *  word/character counter. Fires on every CM6 doc change with this tab's id +
		 *  the live rope. It NEVER feeds back into content flow (no value-prop / doc
		 *  sync), so it sits outside the BUG-015 / §C-2 vector; the host debounces the
		 *  count. Purely an observer — the editor still owns its content. */
		onLiveStats?: (id: string, doc: Text) => void;
		/** MIG-087 §E (item 2) — one-way, display-only live props-count observer for
		 *  the host's status-bar properties count. Forwarded to NotePane's embedded
		 *  PropertyEditor; fires on every property edit with this tab's id + the
		 *  non-empty-key count. Like onLiveStats it NEVER feeds back into content. */
		onLiveProps?: (id: string, count: number) => void;
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

	// MIG-043 Phase 1 — the active note's NSC summary headline (1-line).
	// Fetched via the shared NSC store when tab.path changes. The store is
	// cache-first + file-watcher-invalidated, so this fires at most one IPC
	// per tab open + one more when the note is re-saved (the watcher drops the
	// cached entry, and the next render triggers a re-fetch).
	let activeHeadline = $state<string>('');
	$effect(() => {
		const p = tab.path;
		// MIG-079 §C.2c-3 follow-up — the note-title summary (the NSC headline line
		// under the title) is gated by its own toggle. Off → skip the NSC IPC and
		// hide the line. Reading $appSettings here makes this re-run when toggled.
		if (!p || !$appSettings.noteTitleSummaryEnabled) { activeHeadline = ''; return; }
		// Read-and-write of different reactive vars — no Rule 2 loop.
		void getSummaryFor(p).then((entry) => {
			// Guard: tab may have switched while we awaited; only commit if still current.
			if (tab.path === p) activeHeadline = entry?.headline ?? '';
		}).catch(() => { /* keep prior value on transient errors */ });
	});

	// MIG-076 §C — keep this tab's note model alive and current. `ensure`
	// creates it from the tab's own content when absent (covers every host:
	// main window, split, index preview, dashboard, second screen) and leaves
	// an existing same-path model untouched so live edits win. Writes only to
	// the non-reactive model Map — no store update, so this cannot re-enter a
	// {#key} teardown (the §C-2 lesson). Runs before any keystroke can arrive.
	$effect(() => {
		ensureModel(tab.id, tab.path, tab.content ?? '');
	});

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
		// MIG-076 §C — props come from the model (single source) when on;
		// the body is already live in the model, so compose pulls both.
		const props = SINGLE_OWNERSHIP ? (getModel(tab.id)?.props ?? freshProps()) : freshProps();
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
		let fc: string;
		if (SINGLE_OWNERSHIP) {
			editProps(tab.id, newProps, tab.path);
			const r = compose(tab.id, tab.path);
			if (!r.ok) return; // identity refusal — never write a frankenstein
			fc = r.content; // for the in-store tab display update below (NOT marked saved here)
		} else {
			fc = buildFullContent(newProps, freshBody());
		}
		// Update in-store tab if it exists there
		const ct = get(openTabs).find(x => x.id === tab.id);
		if (ct) {
			ct.content = fc;
			openTabs.update(tabs => tabs);
		}
		// Also update the local tab reference
		tab.content = fc;
		markRecentWrite(tab.path);
		if (SINGLE_OWNERSHIP) {
			// Save-Durability — route through the ONE gate: clean only on a durable
			// write, net + surface on failure, and ADD the reindex the stage promote
			// never had (INV-7 — a stage change now updates the search/derived index).
			saveNoteSession(tab.id, tab.path, standardSaveEnv({
				origin: 'stage_promote',
				name: tab.name,
				onSaved: (savedPath) => {
					broadcastNoteSaved(savedPath);
					invoke('constellation_search_reindex', { notePath: savedPath, libraryName: tab.libraryName }).catch(() => {});
				},
			}), 'stage_promote');
		} else {
			writeNote(tab.path, fc, 'stage_promote').catch(() => {});
		}
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
		if (SINGLE_OWNERSHIP) {
			// Save-Durability (2026-07-08) — push this view's body to the model, then
			// go through the ONE durability gate (noteSession.save + standardSaveEnv):
			// compose (identity-checked; a path mismatch is REFUSED) → net BEFORE the
			// write → await → mark clean ONLY on a durable write + reindex/embed/
			// broadcast/CECE. On failure the model stays DIRTY, the net is RETAINED,
			// and the save-health banner surfaces it (no silent swallow, no false-clean
			// — the app-killer this migration kills).
			editBody(tab.id, text, filePath);
			markRecentWrite(filePath);
			saveNoteSession(tab.id, tab.path, standardSaveEnv({
				origin: 'editor_save',
				name: tab.name,
				onSaved: (savedPath) => {
					broadcastNoteSaved(savedPath);
					// Reindex for search (non-blocking) — FTS5, tags, links, word_count.
					invoke('constellation_search_reindex', {
						notePath: savedPath,
						libraryName: tab.libraryName,
					}).catch(() => {});
					// MIG-071 — re-embed for semantic search (vector went stale after edits).
					if (get(appSettings).enabledFeatures?.semanticSearch) {
						invoke('constellation_embed_notes', {
							notes: [{ path: savedPath, name: tab.name, content: text }],
							force: true,
						}).catch(() => {});
					}
					// MIG-021v3 — CECE on-save background scan (rides the 1500ms debounce,
					// never per-keystroke). Dispatches the same event the manual
					// "Suggest sources & content type" menu uses (Source Review listener).
					if (get(appSettings).cece?.backgroundScan === 'on_save') {
						window.dispatchEvent(new CustomEvent('constellation:classify-and-show', {
							detail: { notePath: savedPath },
						}));
					}
				},
			}), 'editor_save').finally(() => { saving = false; });
		} else {
			// Legacy (dead under SINGLE_OWNERSHIP=true) — kept for the flag-off path.
			const content = buildFullContent(freshProps(), text);
			markRecentWrite(filePath);
			writeNote(filePath, content, 'editor_save')
				.then(() => {
					broadcastNoteSaved(filePath);
					invoke('constellation_search_reindex', { notePath: filePath, libraryName: tab.libraryName }).catch(() => {});
				})
				.catch(() => {})
				.finally(() => { saving = false; });
		}
	}

	function handleFlush(text: string, needsDiskSave: boolean, cursorPos: number, scrollTop: number, filePath: string) {
		if (!filePath || filePath !== tab.path) return;
		// Flush fires on tab close, visibility change, and the {#key}-bump
		// destroy itself — all paths must respect the cascade gate.
		if (isCascading(filePath)) return; // see isCascading() — F2 post-cascade-stomp gate
		// MIG-076 §C — same single-source composition as handleSave; the buffer
		// ops are inert Map writes, safe at this teardown moment (the §C-2 lesson).
		let content: string;
		if (SINGLE_OWNERSHIP) {
			editBody(tab.id, text, filePath);
			const r = compose(tab.id, filePath);
			if (!r.ok) return; // identity refusal
			content = r.content; // for the store-tab display update + no-write recovery net (NOT marked saved here)
		} else {
			content = buildFullContent(freshProps(), text);
		}
		// Update store tab if present (display state)
		const ct = get(openTabs).find(x => x.id === tab.id);
		if (ct) {
			ct.content = content;
			ct.cursorPos = cursorPos;
			ct.scrollTop = scrollTop;
		}
		if (!needsDiskSave) {
			// No disk write (nothing changed since last save) — still stash the current
			// buffer for crash recovery, then done.
			setWriteAhead(filePath, content, cursorPos, scrollTop);
			return;
		}
		markRecentWrite(filePath);
		if (SINGLE_OWNERSHIP) {
			// Save-Durability — the ONE gate: net-before-write → mark clean ONLY on a
			// durable write → compare-and-clear the net → reindex/embed/broadcast. On
			// failure the model stays dirty, the net is retained, save-health surfaces it.
			saveNoteSession(tab.id, tab.path, standardSaveEnv({
				origin: 'editor_flush',
				name: tab.name,
				cursorPos,
				scrollTop,
				onSaved: (savedPath) => {
					broadcastNoteSaved(savedPath);
					invoke('constellation_search_reindex', {
						notePath: savedPath,
						libraryName: tab.libraryName,
					}).catch(() => {});
					if (get(appSettings).enabledFeatures?.semanticSearch) {
						invoke('constellation_embed_notes', {
							notes: [{ path: savedPath, name: tab.name, content: text }],
							force: true,
						}).catch(() => {});
					}
				},
			}), 'editor_flush');
		} else {
			setWriteAhead(filePath, content, cursorPos, scrollTop);
			writeNote(filePath, content, 'editor_flush')
				.then(() => {
					clearWriteAhead(filePath);
					broadcastNoteSaved(filePath);
					invoke('constellation_search_reindex', { notePath: filePath, libraryName: tab.libraryName }).catch(() => {});
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

		// MIG-076 §D2 — when the host wires the full rename orchestration AND single
		// ownership is on, delegate the whole title rename to it (the SAME quiesced
		// path a sidebar rename uses: freeze overlay → model flush → rename_item →
		// wikilink cascade → deliberate remount). This closes the orientation-§13
		// gap — a bare renameItem here never ran the cascade, so every [[link]] to
		// a title-renamed note silently broke. The earlier re-land (a086e1ee) caused
		// BUG-023 because the pre-§C cascade composed writes from drifting parts;
		// §C's identity-bound compose + §D1's freeze make this safe now. Under
		// rollback (SINGLE_OWNERSHIP off) the direct-renameItem fallback runs.
		if (onTitleRename && SINGLE_OWNERSHIP) {
			try {
				await onTitleRename(filePath, newTitle);
			} catch (e) {
				console.error('[NoteEditor] Title rename (delegated) failed:', e);
			}
			return;
		}

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
<div class="ne-wrap">
<NotePane
	value={SINGLE_OWNERSHIP ? seedBody(tab.id, tab.path, body) : body}
	summaryHeadline={activeHeadline}
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
	onDocChange={(doc: Text) => { editBody(tab.id, doc, tab.path); onLiveStats?.(tab.id, doc); }}
	{onLiveProps}
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
</div>
{/key}

<style>
	/* MIG-043 Phase 1 — wrapper makes the summary band + NotePane stack
	 * vertically without breaking NotePane's existing full-height layout. */
	.ne-wrap {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.ne-wrap > :global(*:last-child) { flex: 1 1 auto; min-height: 0; }
</style>

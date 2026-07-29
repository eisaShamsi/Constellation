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
		isCascading, isReseeding,
		type FrontmatterProperty, type PropertyType
	} from '$lib/libraries/store';
	import { broadcastNoteSaved } from '$lib/secondScreen';
	// MIG-076 §C — single content ownership. This step (foundation) only
	// MAINTAINS the model (ensure on tab change + live push per edit); the
	// model is not yet READ for seed/save — that swap lands flag-gated next,
	// so the app behaves identically to the §C-1 safe state right now.
	import { ensure as ensureModel, editBody, editProps, seedBody, save as saveNoteSession, editPropValue, addPropTo, removePropFrom } from '$lib/editor/noteSession';
	import { compose, getModel, isDirty as isModelDirty } from '$lib/editor/noteModel';
	import { propsVersion } from '$lib/editor/propsSignal';
	import { getActiveEditorForPath } from '$lib/editor/activeEditor';
	import { SINGLE_OWNERSHIP, PROPS_SINGLE_OWNERSHIP } from '$lib/editor/ownershipFlag';
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
		readOnly = false,
		onLinkClick,
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
		/** G3 — mount this editor as a READ-ONLY display (the second screen's default).
		 *  Every write callback below early-returns as a belt, and NotePane makes the
		 *  CM6 body + title non-editable. Belt-and-suspenders with NotePane/PropertyEditor:
		 *  even if a view somehow emits a save, the note is never written to disk. */
		readOnly?: boolean;
		/** PJ-089 — override the wikilink click. When provided, a link click in this view calls
		 *  this instead of the default open-in-a-real-tab behavior. The Index read-only preview
		 *  passes a handler that makes the PEEK follow the link (plain click) or open a real tab +
		 *  leave the Index (Ctrl/middle-click) — so a link click never silently opens a hidden
		 *  background tab under the Index overlay. `newTab` is true for Ctrl/⌘/middle-click. */
		onLinkClick?: (link: string, newTab?: boolean) => void;
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
	/**
	 * MIG-107 — the header's stage badge reads the MODEL, not `tab.content`.
	 *
	 * `parsed` comes from `tab.content`, which only refreshes when someone calls `openTabs.update()`.
	 * The header's own Promote button does; a Properties panel never has. So changing the stage from
	 * the sidebar left this badge showing the OLD stage while both Properties panels showed the new
	 * one — Boss-found, 2026-07-28, and pre-existing (PropertyEditor has never notified the tab
	 * store, verified against the pre-migration file).
	 *
	 * Reading the model makes the badge correct for EVERY writer, present and future, instead of
	 * correct for the writers that remember to notify. Falls back to the projection when no model
	 * exists yet (index preview, dashboard).
	 */
	let stage = $derived.by(() => {
		void $propsVersion; // re-read when any note's properties change
		const fromModel = getModel(tab.id)?.props;
		const src = fromModel ?? parsed.properties;
		return src.find((p: FrontmatterProperty) => p.key.toLowerCase() === 'stage')?.value ?? '';
	});

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
		if (readOnly) return; // G3 — read-only display never writes
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
			// MIG-107 Slice 5 — say WHICH property changed instead of handing over the whole array.
			// This site was already sound (Slice 1: it reads the model and writes back with no
			// intervening await), so this is uniformity, not repair: after it, there is no
			// whole-array write left that a future `await` could turn stale.
			// The lookup preserves the CASE-INSENSITIVE match the array version had — a note whose
			// frontmatter says `Stage:` must keep its own spelling, not gain a second key.
			if (PROPS_SINGLE_OWNERSHIP) {
				const existing = getModel(tab.id)?.props.find(p => p.key.toLowerCase() === 'stage')?.key;
				if (!nextStage) { if (existing) removePropFrom(tab.id, existing, tab.path); }
				else if (existing) editPropValue(tab.id, existing, nextStage, undefined, tab.path);
				else addPropTo(tab.id, { key: 'stage', value: nextStage, type: 'text' as PropertyType }, tab.path);
			} else {
				editProps(tab.id, newProps, tab.path);
			}
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
		if (readOnly) return; // G3 — read-only display never writes
		if (saving) return;
		if (!filePath || filePath !== tab.path) return;
		if (isCascading(filePath) || isReseeding(filePath)) return; // F2 post-cascade-stomp gate + PJ-070 watcher-reseed teardown gate
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
		if (readOnly) return; // G3 — read-only display never writes
		if (!filePath || filePath !== tab.path) return;
		// Flush fires on tab close, visibility change, and the {#key}-bump
		// destroy itself — all paths must respect the cascade gate.
		// PJ-070: also skip while this path is mid-reseed from a watcher/external adopt — the
		// outgoing (stale) editor's {#key} teardown must not flush its pre-adopt body back into
		// the freshly-adopted model (hazard #6 re-stale).
		if (isCascading(filePath) || isReseeding(filePath)) return; // F2 post-cascade-stomp gate + PJ-070 watcher-reseed teardown gate
		// MIG-076 §C — same single-source composition as handleSave; the buffer
		// ops are inert Map writes, safe at this teardown moment (the §C-2 lesson).
		let content: string;
		if (SINGLE_OWNERSHIP) {
			// PJ-070 — a merely-viewed note's teardown flush pushes its unchanged body here. That is now
			// inert at the source: setBody's STRING path no-ops an identical-content push (noteModel.ts),
			// so this can't spuriously dirty a clean model (which would defeat adoptDisk + raise phantom
			// `.conflict` sidecars). Same guard protects FocusPane's onflush and flushAllDirtyTabs.
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
			// buffer for crash recovery (and for cursor/scroll across a restart), then done.
			//
			// PJ-181 — flag this entry a SNAPSHOT (content the disk already holds) so it can
			// never outrank a NEWER file on reopen: merely viewing a note leaves one of
			// these, nothing clears it for a closed note, and an external edit (Syncthing /
			// another device / `git pull`) does not change `cid_cn`, so the stale view used
			// to win the screen and the next tab switch wrote it over the newer file.
			//
			// ★ The flag comes from the MODEL, never from `needsDiskSave`. This originally
			// passed a bare `true` on the reasoning that `!needsDiskSave` means "nothing
			// changed since the last durable save" — it does NOT. `needsDiskSave` is
			// NotePane's view-level `dirty`, which `doSave()` clears at save-REQUEST time
			// (NotePane.svelte:340, before the write is even attempted) and never restores
			// on failure. So it is ALSO false while a FAILED or in-flight save's only copy
			// is still unwritten — and flagging that as "already durable" made the reopen
			// reject and DELETE the user's sole recovery copy. The build's own safety
			// inspection caught it, measured, before it shipped.
			//
			// The model tracks DURABILITY: `markSaved` trails the durable write, so clean
			// ⟺ every byte here is already on disk. Under `SINGLE_OWNERSHIP = false` there
			// is no model, so the flag stays false = "real work" = pre-PJ-181 behaviour,
			// which is the direction that never discards.
			setWriteAhead(filePath, content, cursorPos, scrollTop, SINGLE_OWNERSHIP && !isModelDirty(tab.id));
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
		if (readOnly) return; // G3 — read-only display never renames
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

	/**
	 * MIG-101 §A — set (or clear, with `null`) the note's SHAPE.
	 *
	 * **Shape goes through the MODEL, never to disk directly.** An open note's
	 * in-memory model owns its content (MIG-076 single content ownership) and
	 * composes frontmatter from the byte-base captured when the note was OPENED.
	 * A Rust command writing `shape:` straight to disk therefore left the model
	 * unaware: the next debounced save re-composed from the stale base and
	 * SILENTLY DROPPED the key (Boss-reported 2026-07-20 as "nothing changed" —
	 * the write had in fact succeeded on disk while the panel, and the model,
	 * knew nothing about it). Routing through the model fixes both halves at
	 * once: the value survives the next save, and the Properties panel updates
	 * immediately because it renders the thing that changed.
	 *
	 * This mirrors the `stage` promote path above step for step — same
	 * model→compose→durable-save shape, same identity refusal, same reindex.
	 * If a third single-key frontmatter mutator appears, extract the three of
	 * them rather than copying this a third time.
	 *
	 * Shape never touches the body, so a revert is byte-exact by construction.
	 */
	function applyShape(next: string | null, recordHistory = true) {
		// G3 — read-only display never writes. Missing this guard made applyShape
		// the ONLY write path in this component without it (handlePromote:183,
		// handleSave, handleFlush and handleTitleChange all carry it), which handed
		// every read-only host a durable disk write: the Index preview mounts a
		// SECOND model for a path that may already be open in a real tab, so a
		// shape click there would compose that preview's stale body over the live
		// note — and because the real tab's model is clean, the watcher would ADOPT
		// the reverted content rather than raise a conflict. Silent revert, on
		// screen and on disk. Caught by the safety inspection, 2026-07-20.
		if (readOnly) return; // G3 — read-only display never writes
		if (isCascading(tab.path)) return;
		const model = getModel(tab.id);
		const prev =
			(model?.props ?? []).find((p) => p.key.toLowerCase() === 'shape')?.value ?? null;
		if ((prev ?? null) === next) return; // no-op: identical shape

		if (SINGLE_OWNERSHIP && model) {
			const props = model.props;
			let newProps: FrontmatterProperty[];
			if (!next) {
				newProps = props.filter((p) => p.key.toLowerCase() !== 'shape');
			} else {
				let updated = false;
				newProps = props.map((p) => {
					if (p.key.toLowerCase() === 'shape') { updated = true; return { ...p, value: next }; }
					return p;
				});
				if (!updated) newProps.push({ key: 'shape', value: next, type: 'text' as any });
			}
			// MIG-107 Slice 5 — same conversion as the stage setter above, and for the same reason.
			if (PROPS_SINGLE_OWNERSHIP) {
				const existing = model.props.find((p) => p.key.toLowerCase() === 'shape')?.key;
				if (!next) { if (existing) removePropFrom(tab.id, existing, tab.path); }
				else if (existing) editPropValue(tab.id, existing, next, undefined, tab.path);
				else addPropTo(tab.id, { key: 'shape', value: next, type: 'text' as PropertyType }, tab.path);
			} else {
				editProps(tab.id, newProps, tab.path);
			}
			const r = compose(tab.id, tab.path);
			if (!r.ok) return; // identity refusal — never write a frankenstein
			const fc = r.content;
			const ct = get(openTabs).find((x) => x.id === tab.id);
			if (ct) {
				ct.content = fc;
				openTabs.update((tabs) => tabs);
			}
			tab.content = fc;
			markRecentWrite(tab.path);
			saveNoteSession(tab.id, tab.path, standardSaveEnv({
				origin: 'shape_set',
				name: tab.name,
				onSaved: (savedPath) => {
					broadcastNoteSaved(savedPath);
					invoke('constellation_search_reindex', { notePath: savedPath, libraryName: tab.libraryName }).catch(() => {});
				},
			}), 'shape_set');
		} else {
			// No model for this note (not open through the editor) — Rust owns the
			// write AND records its own history row.
			invoke(next ? 'set_note_shape' : 'clear_note_shape', { filePath: tab.path, shape: next })
				.catch((e) => console.error('[NoteEditor] shape write failed:', e));
			return;
		}

		// History is recorded separately here because the DISK write above was the
		// model's, not Rust's. Best-effort: losing a history row must not fail the
		// edit, and the value on disk is the source of truth either way.
		//
		// §A3-fix — an UNDO must not record. `undo_shape` already consumed the step
		// it handed back; appending the inverse as a new change is precisely what
		// made repeated undo oscillate page→scrap→page instead of walking back to
		// unshaped.
		if (!recordHistory) return;
		invoke('record_shape_change', {
			filePath: tab.path,
			fromShape: prev,
			toShape: next,
		}).catch((e) => console.error('[NoteEditor] record_shape_change failed:', e));
	}

	function handleMoreAction(action: string) {
		// The four pure FILE ops are handled here, ALWAYS — they depend only on the tab and
		// must behave identically in every host, including hosts that pass no handler at all
		// (the second screen). They used to live behind `if (onmoreaction) … else`, which made
		// the host's mere EXISTENCE shadow them: the main window's handler knew only its own
		// five actions, so Copy path / Copy name / Show in explorer / Open in default app fell
		// into its switch, matched nothing, and silently died (Boss-reported 2026-07-18).
		// Host-owned actions (rename, delete, focus…) still delegate below.
		switch (action) {
			case 'showInExplorer':
				invoke('constellation_show_in_folder', { path: tab.path })
					.catch((e) => console.error('[NoteEditor] showInExplorer failed:', e));
				return;
			case 'openDefaultApp':
				invoke('open_path', { path: tab.path })
					.catch((e) => console.error('[NoteEditor] openDefaultApp failed:', e));
				return;
			case 'copyPath':
				navigator.clipboard.writeText(tab.path)
					.catch((e) => console.error('[NoteEditor] copyPath failed:', e));
				return;
			case 'copyName':
				navigator.clipboard.writeText(tab.name)
					.catch((e) => console.error('[NoteEditor] copyName failed:', e));
				return;
			// MIG-101 Phase A — shape ops depend only on the tab, so they belong in
			// this always-handled block alongside the file ops. They go THROUGH THE
			// MODEL, never to disk directly — see applyShape.
			case 'shapeScrap':
				applyShape('scrap');
				return;
			case 'shapePage':
				applyShape('page');
				return;
			case 'shapeClear':
				applyShape(null);
				return;
			// MIG-103 §1 — Save as Template. Reads the LIVE model when the note is
			// open (the model is the authority — MIG-076; disk may be a stale cast),
			// falls back to tab.content otherwise. Creates a NEW file in the
			// universe's Templates folder, so it is a write and read-only hosts
			// refuse (Display-not-Domain).
			case 'saveTplWhole':
			case 'saveTplFrontmatter':
			case 'saveTplSnippet': {
				if (readOnly) return; // G3 — read-only display never writes
				const model = getModel(tab.id);
				let liveContent: string | undefined;
				if (SINGLE_OWNERSHIP && model) {
					const r = compose(tab.id, tab.path);
					if (r.ok) liveContent = r.content;
				}
				// The title-confirm prompt lives in the layout (where the modals are),
				// so the actual create_template call happens there after the user
				// accepts or edits the template name (Boss request, 2026-07-21). We
				// capture the LIVE content here — the model is the authority on an
				// open note (MIG-076) — and hand it off with the note's name and the
				// chosen KIND (whole | frontmatter | snippet).
				const kind = action === 'saveTplFrontmatter' ? 'frontmatter'
					: action === 'saveTplSnippet' ? 'snippet' : 'whole';
				// A snippet is a FRAGMENT, so its extent is the user's choice (Boss,
				// 2026-07-21). Read whatever is selected in the live editor; the
				// layout offers "Selection vs Whole note" only when there IS one —
				// no selection means there is nothing to choose between, so it does
				// not ask (a stated need is not an invitation to interrogate).
				let selection = '';
				if (kind === 'snippet') {
					const view = getActiveEditorForPath(tab.path);
					const sel = view?.state.selection.main;
					if (view && sel && !sel.empty) selection = view.state.sliceDoc(sel.from, sel.to);
				}
				document.dispatchEvent(new CustomEvent('constellation:save-as-template', {
					detail: {
						path: tab.path,
						defaultName: tab.name.replace(/\.md$/, ''),
						content: liveContent ?? tab.content ?? '',
						kind,
						selection,
					},
				}));
				return;
			}
			case 'shapeRevert':
				// `undo_shape` consumes one step and returns the shape to restore
				// (null = back to unshaped). Applied WITHOUT recording, so the next
				// undo takes the step before it rather than undoing this undo.
				// A rejection means there is nothing left to undo — a normal end
				// state, not a fault.
				invoke<string | null>('undo_shape', { filePath: tab.path })
					.then((target) => applyShape(target ?? null, false))
					// Never swallow this. A bare `.catch(() => {})` here is what turned
					// a schema-upgrade bug into an INVISIBLE one: every `undo_shape`
					// call was failing with "no such column: undone" and the UI simply
					// did nothing, with no trace anywhere (Boss-reported 2026-07-20 as
					// "no effect at all"). "Nothing to undo" is the one expected
					// rejection; anything else is a fault and must say so.
					.catch((e) => {
						const msg = String(e);
						if (!msg.includes('Nothing to undo')) {
							console.error('[NoteEditor] undo_shape failed:', e);
						}
					});
				return;
		}
		onmoreaction?.(action);
	}

	async function handleLinkClick(link: string, newTab?: boolean) {
		if (!tab.libraryPath) return;
		try {
			const resolved = await resolveWikilinkCrossLibrary(tab.libraryPath, link);
			if (resolved) {
				const libColors = buildLibraryColorMap(get(libraries));
				// PJ-108 — a read-only host (second screen) passes preserveNet so following a
				// wikilink never consumes the shared crash-recovery net (it has no writable
				// editor to re-stash it). Writable hosts keep the consume-and-re-stash default.
				await openNoteTab(resolved.path, resolved.library_name, libColors[resolved.library_name] || '#7c3aed', undefined, newTab, tab.path, undefined, readOnly);
			} else if (!readOnly) {
				// Note doesn't exist — create it in the same folder with default frontmatter.
				// NEVER from a read-only display (Display-not-Domain — a read-only surface must
				// not write a new file to disk); an unresolved link there is simply inert.
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
	{readOnly}
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
	onDocChange={(doc: Text) => { if (!readOnly) editBody(tab.id, doc, tab.path); onLiveStats?.(tab.id, doc); }}
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
	onlinkclick={onLinkClick ?? handleLinkClick}
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

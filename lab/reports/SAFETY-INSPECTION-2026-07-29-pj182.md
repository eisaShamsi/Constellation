# Safety Inspection — 2026-07-29 (PJ-182 build)

Invoked **diff-scoped** with `args.files` (13 files). It returned `mode: "whole-app"` —
**PJ-166's EIGHTH strike**. 88 agents, ~10.8 M tokens, ~30 min. Every candidate was
adversarially refuted before confirmation.

**52 confirmed findings / 50 unique sites** — 5 APP-KILLER · 10 HIGH · 29 MED · 7 LOW.

## Triage against THIS build

- **Caused by this change → FIXED before commit:** `sources/mod.rs` — the PJ-182 pass
  routed the block-SKIP half through the shared rule and left the key-MATCH half testing
  the TRIMMED line, so an indented `sources:` / `content_type:` (a nested map's key, or
  prose inside a block scalar) was matched as the note's own key and DELETED. Fixed at
  all four sites (both writers and both readers, so they agree), RED-proven.

- **Pre-existing, in files this change touched → filed, not fixed here:**
  `yamlDoc.ts:311` (APP-KILLER — the H1 branch silently discards every property edit on a
  note whose YAML is malformed, and reports success), `yamlDoc.ts:362` (the CST
  splice+append deletes comments attached to the edited key), `PropertyEditor.svelte:430`
  (`seededRows` is cloned RAW while `editableProps` is display-normalised, so untouched
  keys read as touched).

- **The remaining 46 sites** are the whole-app register, unrelated to PJ-182. Headline:
  **APP-KILLER `PropertyEditor.svelte:974`** — the right-sidebar panel is never
  `{#key}`-remounted, so a pending 800 ms debounce that survives an in-place navigation
  writes note A's properties onto note B, durably.


---

## The register

### [APP-KILLER] `src/lib/components/PropertyEditor.svelte:974` — cross-note-bleed

**The right-sidebar PropertyEditor instance is never keyed/destroyed on an in-place navigation, so its pending 800 ms debounce fires with the LIVE (already-swapped) `tabId`/`filePath` and commits the OUTGOING note's property rows into the INCOMING note's model — which is then composed and written to disk.**

*Scenario:* Sidebar → Properties tab is open on note A. User edits a value (e.g. `tags: draft, urgent`) → `debouncedSave()` arms an 800 ms timer (PropertyEditor.svelte:962-979). Within 800 ms the user clicks a wikilink / file-tree row → `openNoteTab` in-place reuse re-seeds the SAME tab id to note B (store.ts:2703-2727). Because the sidebar panel is mounted under a bare `{#if rightSidebarTab === 'properties' && sidebarTab}` (+layout.svelte:9051-9071) with NO `{#key}`, the instance survives; the seed `$effect` sees `tabChanged === false` (same tab id) and `localEditPending === true`, so it takes the skip branch at PropertyEditor.svelte:429 and `editableProps` still hold A's rows. The timer then fires: line 973 writes `buildFullContent(A-rows, B-body)` into B's `tab.content`, and line 974 calls `commitAndSave(tabId, filePath)` where `filePath` is now B's path. Every `expectPath` guard passes (model pa


### [APP-KILLER] `src/lib/editor/yamlDoc.ts:311` — false-success

**composeFrontmatter's H1 branch silently discards EVERY property edit on a note whose frontmatter has any YAML parse error, and reports the save as successful.**

*Scenario:* A note's frontmatter contains any construct `yaml`'s parseDocument flags as an error (verified shapes: `title: Chapter 1: Beginnings`, `title: [Draft] Notes`, `sched: @daily`, ``note: `code` ``, one mis-indented continuation line). The user opens Properties, changes `stage` from spark to growth. The intent lands in the model (panel shows growth), commitAndSave → saveTabContent → saveNoteSession → compose → composeFrontmatter. Line 311 returns `---{rawYaml}---{body}` — the ORIGINAL frontmatter bytes, edit dropped. The write succeeds, markSaved runs, the model goes clean, no banner, no console warning. On the next reopen the note is back at spark. Every later property edit on that note is lost the same way, forever. The FmDoc.hasErrors field that could surface this is dead code (parseFrontmatterDoc has no callers), so nothing anywhere tells the user their frontmatter is unwritable.


### [APP-KILLER] `src-tauri/src/universe.rs:85` — silent-data-loss

**`load_registry` collapses BOTH a read error and a parse error into an empty `UniverseRegistry` with no backup, and every registry-WRITE path (`create_universe:662`, `open_existing_universe:1080`, `link_library_as_universe:1211`, `remove_universe_from_registry:937`, `migrate_legacy_data:1761`) builds its new registry on top of that empty value and atomically renames it over `universes.json` — deregistering every other Universe with no error. This is the exact anti-pattern that was declared an APP-KILLER and fixed on 2026-07-21/22 for `libraries.json` (`libraries.rs:71 try_load_libraries`, "Absent is a fact; unreadable is an unknown") and at `universe.rs:392`; the registry loader is the surviving instance of the same class.**

*Scenario:* `%APPDATA%/world.uconstellation.app/universes.json` (an app-data path routinely touched by OneDrive/Defender/backup agents) is momentarily unreadable — or externally corrupt — at boot. `list_universes` (universe.rs:579) calls `load_registry`, gets `entries: []`, and `+layout.svelte:3182` reads `universes.length === 0` as FIRST LAUNCH and shows the Universe Setup wizard. The user, told this is a fresh install, creates a Universe: `create_universe` calls `load_registry` again, pushes ONE entry, and `save_registry:666` fsync-renames a one-entry `universes.json` over the file that listed all N Universes (or, on the `needsMigration` branch, `migrate_legacy_data:1761` builds `UniverseRegistry { entries: vec![entry] }` outright). Every other Universe is now unregistered — the app's only index of where the user's knowledge lives — with no error, no dialog, and no `.corrupt-*` backup (the guard `


### [APP-KILLER] `src/routes/+layout.svelte:3407` — content-loss

**The `wasRecentlyWritten(p)` guard in the `library-changed` listener silently discards the Rust-side frontmatter announce (`announce_frontmatter_write`/`announce_frontmatter_writes`), so an open note never re-bases and its next save erases the just-accepted `sources:`/`content_type:` block from the .md on disk.**

*Scenario:* Note X is open in NotePane with the note-scoped Source Review rail showing X's card. The user types; the 1500 ms debounce fires and NoteEditor.svelte:294 calls `markRecentWrite(X)` (store.ts:1538 stamps `recentWrites`; `wasRecentlyWritten` returns true for 2000 ms — store.ts:1561-1564). Within that 2 s the user clicks Accept. `sources_set_manual` (src-tauri/src/sources/mod.rs:707-708) writes the `sources:` block to X.md through `gate_rmw` — which marks the path watcher-SUPPRESSED, so the ONLY channel telling the frontend is the `library-changed` announce on the next line. The listener at +layout.svelte:3407 evaluates `wasRecentlyWritten(X)` → true → X is added to NEITHER `pendingTabReloads` NOR `pendingReindex`, so `adoptExternalChangeIntoTabs` (line 3369) never runs for X. X's note model keeps its OPEN-TIME frontmatter props. The user types one more character; `saveUnchained` (src/lib/e


### [APP-KILLER] `src-tauri/src/universe.rs:117` — concurrency-race

**`universe::atomic_write` derives its temp file from the TARGET filename alone (`collections.json.tmp`) and takes no per-path lock, so two concurrent writers of the same persisted JSON share one temp path — unlike `write_gate::atomic_write`, which uses a pid+counter-unique temp AND holds the file's lock.**

*Scenario:* `save_universe_collections` (universe.rs:1652) is `#[tauri::command(async)]`, so every invocation runs on the Tokio pool — genuinely parallel. `saveCollections()` (store.ts:1722) has NO serialization chain (unlike session.ts, which chains on `inFlight`) and is called from ~8 mutation sites. Concrete: the Collections panel's hydration resolves and calls `adoptCollectionIdentities` → `saveCollections()` (write A) while the user clicks ⭐ on a note → `toggleStarred` → `saveCollections()` (write B), ~100 ms apart. Both run `File::create("…/collections.json.tmp")` — B's create TRUNCATES the temp A is mid-`write_all` on. Then A's `sync_all` + `fs::rename` publishes a torn/zero-length `collections.json` (or, on Windows, A's rename hits ERROR_SHARING_VIOLATION against B's open handle and returns Err, which store.ts:1723 swallows to `console.error` — invisible in a release build, devtools disabled


### [HIGH] `src-tauri/src/write_gate.rs:249` — freeze-hang

**atomic_write suppresses the watcher for only the temp path and the final path, never the containing directory — violating the explicit THREE-key contract documented on watcher_suppress::mark_with_parent, which has zero production callers — so every gated .md write leaks an unsuppressed bare-directory event that drives a full subtree reindex walk.**

*Scenario:* watcher_suppress.rs:52-64 states the measured Windows fact: a temp+ReplaceFileW reports a BARE-DIRECTORY event in addition to the file events, and was_recent is exact-path keyed, so the directory is a separate key; callers 'must mark THREE keys: the temp path, the final path, and (via this helper) the containing directory'. atomic_write marks only two (write_gate.rs:249-250); gate_rename (:598-599, :725-726) and gate_delete (:763) likewise. grep across src-tauri shows mark_with_parent has NO production caller. Concrete run: the user edits a note living at the Universe root (the default flat universe_notes library) and the 1.5 s debounce fires -> write_note -> gate_write -> atomic_write. The .md path is suppressed, but the Universe-root directory event is not. watcher.rs:111-113 passes it (metadata().is_dir() -> pass), watcher.rs:119 was_recent(dir) is false, so 'library-changed' is emitt


### [HIGH] `src-tauri/src/libraries.rs:1270` — freeze-hang

**The folder branch of rename_item runs the whole descendant DB cascade (search_state.db held across the migrate loop) plus one reindex_single_note per descendant INSIDE the awaited (async) invoke — the exact shape the same function's .md branch detaches to a worker at :1371 because an awaited IPC that parks on the writer lock never settles.**

*Scenario:* libraries.rs:1349-1359 states the rule this branch breaks: 'the DB tail parks on the UNBOUNDED SearchState writer mutex ... and with the command now (async) that park is INVISIBLE: the invoke promise never settles, so the frontend's whole post-rename orchestration silently never runs'. The .md path obeys it (spawn_blocking at :1371). The folder path at :1264-1289 does not: it takes search_state.db.lock() (:1270) and holds it across migrate_note_db_paths for every descendant (each opening its own BEGIN IMMEDIATE/COMMIT at libraries.rs:1129-1135), then runs N sequential reindex_single_note calls (:1286), each re-acquiring the writer lock — all before `return Ok(new_path)` at :1290. Concrete run: the user renames a folder holding ~2,000 notes while a boot backfill / reconcile / CECE pass holds the writer lock. rename_item's fs::rename already completed (journal line 'renamed'), but the comm


### [HIGH] `src/lib/libraries/store.ts:2600` — silent-data-loss

**Both nav paths CONSUME the incoming note's write-ahead recovery net (`resolveNoteContent` → `clearWriteAhead`) before any of the abort/supersede early-returns, so an aborted or superseded navigation destroys the only copy of that note's unsaved, save-failed content without ever creating a model for it.**

*Scenario:* Note X previously failed a save (locked .md / sync tool), so its unsaved paragraph lives ONLY in the write-ahead net (`noteSession.saveUnchained` sets the net before the write and retains it on failure — noteSession.ts:250-257). The user, sitting on a dirty note Y, clicks X: `openNoteTab` runs `resolveNoteContent(X, {preserveNet: false})` (store.ts:2600), which reaches `clearWriteAhead(filePath)` at store.ts:2512 and returns X's recovered bytes in a local `content` variable. Execution then awaits `flushOutgoing(currentTab.id)` for Y; if Y's flush fails (same lock) the function returns at store.ts:2694, and if the user clicked another note in the meantime it returns at store.ts:2695 (`superseded`). Either return discards `content` — no tab, no `openNoteModel`, no `markModelRecoveredFromNet` — while X's net has already been deleted from memory and localStorage. X's unsaved paragraph now ex


### [HIGH] `src/routes/+layout.svelte:4394` — content-corruption

**The template-create frontmatter merge re-emits the note's canonical property values as raw unquoted YAML (`${p.key}: ${p.value}`), producing invalid frontmatter for any title that needs quoting.**

*Scenario:* Templates are enabled and a folder template (or Templates/default.md) exists. The user creates a note titled `Chapter 1: Beginnings`. create_note (libraries.rs:955) correctly writes `title: "Chapter 1: Beginnings"`. This block then re-reads the stub, parses it (unquoting the value), and rebuilds the block as `title: Chapter 1: Beginnings` — verified against parseDocument to be a hard error ("Nested mappings are not allowed in compact mappings"). write_note lands it on disk with origin 'template_create'; nothing validates the result. From that moment the note is permanently in the yamlDoc.ts:311 H1 state: the Properties panel shows edits, disk never receives them, and no error is surfaced. Same for a title containing `[`, `#` at value start (silently becomes `title: null`), a leading `@`, or a backtick.


### [HIGH] `src/lib/libraries/store.ts:457` — content-corruption

**rebrandCopyFrontmatter strips the quotes off `title:` and re-emits the value unquoted, so a 'Save a copy' recovery of a note whose title needs quoting produces invalid frontmatter.**

*Scenario:* A note's own file is locked (sync tool / antivirus) and the user takes the app's documented exit, 'Save a copy'. The original has `title: "Study: Part II"`. The regex captures `Study: Part II` (the `"?` groups are dropped) and re-emits `title: Study: Part II (recovered copy)` — verified to be a parseDocument error. The recovered copy — the artifact holding the user's unsaved work — is written through the gated path and opened as a real tab, and from that point every property edit on it is silently discarded by composeFrontmatter's H1 branch. No error at any step.


### [HIGH] `src/lib/libraries/store.ts:6008` — silent-data-loss

**An unreadable/corrupt `settings.json` is degraded to `{}` in Rust (`boot_bundle.rs:100` `read_universe_settings(...).unwrap_or(Object{})`), and `applyParsedSettings` early-returns on an empty object (`if (!parsed || Object.keys(parsed).length === 0) return;`) — so the in-memory `appSettings` is never told the load failed. `saveSettings` (store.ts:6226) then persists `get(appSettings)` — the WHOLE object — over the file 300 ms after the next of ~101 `updateSettings()` call sites fires.**

*Scenario:* COLD BOOT: `.constellation/settings.json` is locked by a sync client/AV scanner (or partially synced) when `constellation_boot_bundle` reads it. Rust swallows the error into `{}`; `applyParsedSettings({})` returns immediately; `appSettings` stays at `DEFAULT_SETTINGS`. The user then drags the note-pane splitter — `+layout.svelte:1273` calls `updateSettings({ leftOfNoteWidth, rightOfNoteWidth })` — and 300 ms later the DEFAULTS are atomically written over `settings.json`. Every per-Universe setting is permanently gone with no error: `styleOverride` (the whole Style Setter look), interface/text/mono/script fonts, `panelPlacements`, `customShortcuts`, `customThemes`, `styleSwatches`, `colorScheme`, and the nested `index`/`review`/`security`/`cece`/`sight` blocks. SWITCH VARIANT (worse): switching Universe A → B while B's `settings.json` is unreadable leaves `appSettings` holding A's setting


### [HIGH] `src/lib/libraries/store.ts:1834` — silent-data-loss

**`loadCollections` swallows a failed `read_universe_collections` with `catch { /* ignore — empty until first save */ }`, leaving `collectionSets` at its previous value (`[]` at boot, or the PREVIOUS Universe's collections after a switch); `saveCollections` (store.ts:1722) then writes that whole stale/empty array back over `collections.json` on the very next collection mutation. Unlike `libraries.json` (backed up on corruption at `libraries.rs:88`) and `review-pulse.json` (set aside at `review.rs:790`), nothing preserves the file before it is overwritten.**

*Scenario:* `read_universe_collections` (universe.rs:1640-1642) returns Err on BOTH a read error and a parse error. `.constellation/collections.json` is transiently locked (Syncthing/OneDrive/AV) during the post-paint `loadCollections()` at `+layout.svelte:2663` → the catch fires → `collectionSets` stays `[]`. The user then clicks the ⭐ on any note: `toggleStarred` → `addToStarred` → `saveCollections()` → `save_universe_collections` atomically renames a one-item `[Starred]` array over the file that held every named Collection and every starred note. All hand-curated working sets are gone, silently — no banner, no error, and the un-awaited `.catch(e => console.error(...))` is invisible in a release binary (devtools disabled). SWITCH VARIANT: `collectionSets` is a module-level store that `loadCollections` only `.set()`s on success, so a failed load after a Universe switch leaves Universe A's collectio


### [HIGH] `src-tauri/src/sources/bulk_ops.rs:401` — swallowed-write-error

**In `accept_one`, the disk frontmatter is already written by `gate_rmw` before step 4; if `write_sources_to_db` / `write_content_type_to_db` / `clear_suggestions` fails, `?` returns Err, so the path is never announced and the note_meta mirror silently diverges from disk — and the error is recorded only in `last_error`, which no frontend code ever reads.**

*Scenario:* Approve-All runs over a 7,600-note universe on the background thread spawned at bulk_ops.rs:127. For note X, `gate_rmw` (line 380) succeeds and X.md on disk now carries the accepted `sources:`/`content_type:` blocks. Step 4 then hits a failure — `search_state.db.lock()` yields `None` because a universe switch/DB re-init tore the connection down mid-run (line 398-400 `.ok_or("Search database not initialized")?`), or `write_sources_to_db` errors on a locked DB (line 401). `accept_one` returns Err, so the caller at line 244-250 takes the `Err(e)` arm: it stores the message in `state.last_error` and does NOT call `announce_pending.record(...)`. Three silent consequences: (a) note_meta.sources/content_type keeps the pre-accept value forever — the gate suppressed the watcher and no reindex was queued, so the Index/Search/Cataloger surfaces permanently disagree with the .md; (b) `clear_suggesti


### [HIGH] `src/routes/+layout.svelte:3382` — index-divergence

**The watcher's external-change reindex is `try { await invoke('reindex_changed_paths', …) } catch {}` (and `.catch(()=>{})` on the >250-path burst branch) AFTER the pending set has already been cleared — a failed reindex discards the paths forever, with no retry and no boot walk to heal it.**

*Scenario:* `scheduleWatcherFlush` (+layout.svelte:3344-3347) drains `pendingReindex` into `reindexPaths` and calls `pendingReindex.clear()` BEFORE issuing the IPC. At :3382 the invoke is wrapped in `catch {}`; at :3384 the burst branch swallows with `.catch(()=>{})`. `reindex_changed_paths` (search.rs:10425) opens with `ensure_search_db_ready(&app)?`, which returns Err whenever there is no active universe / the DB cannot be opened — reachable when a Syncthing or `git pull` burst lands during a universe switch (handleUniverseSwitch tears down and re-inits the search state while the watchers for the outgoing universe are still flushing) or in the boot window before search init. Concrete: user runs `git pull` bringing 40 edited `.md` files while switching universes. The tree refreshes (files visible), `adoptExternalChangeIntoTabs` updates any open tab, then the reindex Errs and is swallowed; the 40 pa


### [HIGH] `src/lib/components/PropertyEditor.svelte:429` — silent-data-loss

**The seeding effect's `|| tabChanged` bypasses the `localEditPending` guard and re-seeds `editableProps` from the incoming note, discarding an in-debounce property edit that (under MIG-107) lives ONLY in the panel's local rows and was never pushed to the note model.**

*Scenario:* Under PROPS_SINGLE_OWNERSHIP, a property edit reaches the model only at `commitAndSave` (line 921), 800 ms after the last keystroke (line 979). Type a new value into `stage` in the RIGHT-SIDEBAR Properties panel for note A, then click note B in the file tree within 800 ms. That panel is mounted at +layout.svelte:9056 with plain props and NO `{#key sidebarTab.id}`, so it is never destroyed — `tabId`/`filePath`/`properties` swap in place. This effect fires with `tabChanged` true, takes the branch despite `saveTimeout !== undefined`, and overwrites `editableProps` with B's rows. When the 800 ms timer fires it reads the CURRENT `tabId`/`filePath` (B) and commits B's unchanged rows → `changed` false, `isNoteDirty(B)` false → returns having written nothing. A's edit exists nowhere: not in `editableProps`, not in the model, not in the write-ahead net, not on disk. No error, no save-health banne


### [MED] `src/lib/libraries/store.ts:4064` — content-loss

**moveItem has neither the flush-before-mutate nor the markCascading gate that its sibling renameItem carries, so a debounced save firing between move_item's filesystem rename and repathNoteModel writes to the OLD path and — because the source folder still exists — gate_write silently CREATES a duplicate .md there carrying the same cid_cn and the user's newest edits.**

*Scenario:* renameItem (store.ts:3916-3926) explicitly markCascading's the tab and flushes the dirty model BEFORE invoking, with the rationale 'a dirty open tab's last <=1.5 s of typing lives only in the editor model ... once async, could race the rename'. moveItem (store.ts:4064-4067) does neither: it invokes move_item immediately and only repaths the model at :4072 AFTER the await. Rust-side, move_item's fs rename completes within the first milliseconds (libraries.rs:2107) but the command then holds the awaited IPC through migrate_note_db_paths + reindex_single_note (:2139-2152), which can take seconds under writer-lock contention. Concrete run: a save is armed (or the ~10 s save-health auto-retry is pending) for the open note when the move is confirmed. It fires inside that window and calls write_note with the tab's still-OLD path. libraries.rs:595-601 passes (the SOURCE folder still exists — onl


### [MED] `src-tauri/src/libraries.rs:2160` — index-divergence

**collect_md_paths — the descendant enumerator shared by the folder rename cascade, move_item and delete_path — has no dot-segment skip, unlike every sibling walker, so a folder rename/move whose subtree contains `.trash` re-indexes trashed notes back into note_meta at .trash paths.**

*Scenario:* collect_md_paths (libraries.rs:2160-2177) filters only symlinks and the .md extension. Its siblings all skip dot dirs: collect_folders at :2247 (`if name.starts_with('.') { continue; }`), read_dir_recursive, and — explicitly, as a fix for this exact damage — reindex_changed_paths Pass 1, whose MIG-104 Slice 2b comment (search.rs:10442-10457) records that failing to skip dot segments 'is the PRODUCER of the 62 live note_meta rows sitting at .trash paths', with '543 history rows across 40 trashed notes, two of them STILL ACCRUING history while sitting in the trash'. Concrete run: a library root (or any folder that owns a `.trash`, since move_into_trash_folder creates it under the chosen trash_root at libraries.rs:6723) is renamed or moved. rename_item :1269 / move_item :2122 call collect_md_paths over the new tree, which descends into `.trash`; each trashed note is then migrate_note_db_pat


### [MED] `src-tauri/src/canonical.rs:1311` — index-divergence

**`ensure_cid_cn_cmd` injects `cid_cn:` into a note's frontmatter on disk through the (watcher-suppressed) gate and never reindexes, so `note_meta.cid_cn` stays `''` while the file carries a real identity — and every cid-keyed heal path then reads the stale empty value.**

*Scenario:* An externally-created note (Obsidian/git on a linked library) is already indexed by the boot walk with `cid_cn = ''`. The user opens it: `openNoteTab` calls `invoke('ensure_cid_cn_cmd')` (store.ts:2621; the deferred restore path does the same at store.ts:2927). `ensure_cid_cn` writes the new frontmatter via `gate_write(..., "ensure_cid_cn")` (canonical.rs:1303), which marks the path watcher-suppressed — and neither the Rust command nor the two frontend call sites follow with `constellation_search_reindex`. `note_meta.cid_cn` therefore remains `''` for the whole session while disk says otherwise. Concrete damage: if that note is renamed in the same session and the detached rename DB tail is starved (the exact MIG-097 case reconcile.rs exists for), the next boot's reconcile finds a dead row whose `cid` is `''` → `let target = if cid.is_empty() { None }` (reconcile.rs step 7) → the row fall


### [MED] `src-tauri/src/write_gate.rs:250` — freeze-hang

**`atomic_write` marks only the temp path and the final path for watcher suppression, violating the module's own measured three-key contract (`mark_with_parent`, which has ZERO production callers) — so every gated note write leaks an unsuppressed bare-directory watcher event that drives a full subtree reindex walk plus a library tree re-walk.**

*Scenario:* `watcher_suppress.rs:52-70` states the contract as measured (`ReadDirectoryChangesW` probe): "a temp+replace or a rename-aside reports a bare-directory event as well as the file events, and `was_recent` is EXACT-PATH keyed, so the directory is a separate key… Callers doing an atomic replace must mark THREE keys: the temp path, the final path, and the containing directory." `atomic_write` IS the sole atomic-replace implementation (temp create + fsync + `ReplaceFileW`) and marks only two (write_gate.rs:249-250); `mark_with_parent` is defined and unit-tested but grep shows no production caller. Consequence per debounced 1.5 s autosave of any note outside `.constellation`: the directory event passes `is_app_bookkeeping_path`, passes the metadata filter (`m.is_dir() => true`, watcher.rs:113), passes `was_recent` (the dir was never marked), and is emitted as `library-changed`. The frontend's o


### [MED] `src/lib/libraries/store.ts:1694` — silent-data-loss

**`loadTabHistoryEntry` flushes the outgoing model BEFORE an awaited disk read, then force-replaces the model with `openNoteModel` — keystrokes that land during that await are silently discarded (the APP-KILLER #2 nav-loss class). `openNoteTab` orders these the other way and has no such window.**

*Scenario:* User is typing in note A and presses Alt+← . `loadTabHistoryEntry` awaits `flushOutgoing(tabId)` (store.ts:1656) — the model is now clean — and then awaits `resolveNoteContent(filePath)` (store.ts:1664), a real `read_note` IPC round-trip (tens of ms, more on OneDrive/network drives). NotePane's CM6 updateListener pushes every keystroke straight into the model synchronously (NotePane.svelte:643 → NoteEditor.svelte:721 → `editBody`), and nothing gates it during this window: `markCascading`/`isReseeding` are not set, and the 1500 ms debounced save has not fired, so the characters exist ONLY in the model and in no write-ahead net. When the read resolves, store.ts:1680-1694 synchronously rewrites the tab and calls `openNoteModel(tabId, filePath, content)`, which replaces the model object outright (noteModel.ts:157-170) — the during-await keystrokes are gone. The `{#key}` remount then fires No


### [MED] `src/lib/components/PropertyEditor.svelte:583` — content-loss

**A property edit pending in PropertyEditor's 800 ms debounce has not yet reached the note model, and no departure path flushes it — `flushOutgoing`/`flushIfDirty` only flush the MODEL. On an in-place nav the embedded panel's onDestroy identity gate correctly refuses to write, which means the edit is silently dropped.**

*Scenario:* User types a new value into the in-note Properties panel (e.g. changes `stage`) and, within the 800 ms debounce, clicks a wikilink. `openNoteTab`'s departure flush (store.ts:2690-2696) calls `isNoteDirty(currentTab.id)`, but the property edit is still only in the panel's local `editableProps` — `commitAndSave` (PropertyEditor.svelte:921-948) is what pushes it into the model, and it has not run. So `isDirty` reports false and no flush happens. The `{#key tab.id|tab.path|reloadVersion}` remount (NoteEditor.svelte:693) then destroys the panel; its onDestroy clears the timer (PropertyEditor.svelte:566) and the identity gate at :583 evaluates `filePath === mountedFilePath` against the LIVE prop, which now holds the incoming note's path → mismatch → clean skip. The edit is discarded: never in the model, never on disk, never in the write-ahead net, and nothing is surfaced. The tab-switch varian


### [MED] `src/lib/components/FocusPane.svelte:155` — content-loss

**FocusPane commits a typed title only from the input's `blur` handler, but Escape inside the title input tears the component down directly (`reportCaret(); onexit?.()`) and `onDestroy` flushes only the BODY — so a title typed in Focus and dismissed with Escape is silently discarded from screen, model and disk.**

*Scenario:* The user is in Focus mode on note A. The title line is visible (FocusPane.svelte:333 renders it once `wordCount > 0`). They click into the title input and type a new title — `bind:value={titleValue}` updates local state only; `ontitlechange` is fired exclusively from `handleTitleBlur` (FocusPane.svelte:137-144). They press Escape while the caret is still in the title input. `handleTitleKeydown` (line 150) hits the Escape branch at line 155: `reportCaret(); onexit?.()`. `onexit` sets `focusMode = false` (+layout.svelte:8626), which destroys the FocusPane. Removing a focused input from the DOM does not fire a `blur` event, so `handleTitleBlur` never runs and `ontitlechange` — the prop the layout wires to `handleRenameComplete(focusSessionPath, newTitle)` at +layout.svelte:8593 (the PJ-116 §0.1 fix that exists precisely because Focus titles were being discarded) — is never called. `onDestro


### [MED] `src/lib/libraries/store.ts:559` — content-corruption

**`flushOutgoing` — the single choke point for every DEPARTURE flush (openNoteTab in-place reuse :2693, loadTabHistoryEntry :1656, closeTab :3083) — has no `isCascading` gate, so a model dirtied DURING the rename-cascade window is composed from its PRE-cascade body and written over the file the walker already rewrote, silently reverting that note's [[link]] update.**

*Scenario:* autoUpdateLinks ON. Note X (open, contains [[A]]) is in library L. User renames A → 'A v2'; handleRenameComplete flushes every tab in L clean (flushAllTabsInLibrary :6837), raises the freeze on L's EDITOR PANES only, and starts the multi-second walk. During the walk the user edits a property of X in the RIGHT-SIDEBAR PropertyEditor — reachable by design, and by design PJ-174 #1c lets that edit land in the model (dirty) while `saveTabContent` skips only the write (store.ts:1479-1499; its own comment: 'The right-sidebar PropertyEditor is reachable during the window too — the freeze overlay covers editor panes, not the sidebar'). The walker then rewrites X on disk to [[A v2]] under gate_rmw. The user now clicks another note in the sidebar TREE (never frozen — see +layout.svelte:6829-6833). openNoteTab's in-place-reuse branch calls `flushOutgoing(currentTab.id,'nav_flush')` → flushIfDirty → 


### [MED] `src/routes/+layout.svelte:6537` — toctou

**`addTagToNote`'s CLOSED-note branch does an unguarded, non-atomic readNote → writeNote of the WHOLE file with no `isCascading(path)` gate — the exact defect its two documented siblings (`addLinkToNote` store.ts:1237, `linkMentionInNote` store.ts:1318) were fixed for; a tag added during a rename cascade writes a pre-cascade snapshot back over the walker's rewrite.**

*Scenario:* autoUpdateLinks ON, note X closed and containing [[A]]. The user renames A → 'A v2' from the file tree; the walk over a large library takes seconds and blocks only the editor panes. Still during the walk the user right-clicks X in the tree → 'Add tag' → confirm (or selects 50 notes and runs the BATCH tag loop at :9651, which serially awaits one note at a time and easily straddles the window). `const content = await readNote(path)` returns X's PRE-cascade bytes; the walker's gate_rmw then rewrites X to [[A v2]]; `writeNote(path, composeUpdatedContent(content, addTagToProps(...), body))` at :6541 writes the whole stale file back with the tag added — the per-path write lock serializes the two writes but never re-reads, so the walker's update is a lost update. `reindexNote` at :6543 then persists the reverted [[A]] edge into note_links. X is closed, so no tab reload, no conflict sidecar, no 


### [MED] `src/routes/+layout.svelte:6802` — concurrency-race

**`cascadeFreeze` is set/cleared as a bare whole-set assignment (`set(new Set([lib.path]))` :6802, `set(new Set())` :6937) while BOTH write-gate maps it mirrors are refcounted for exactly this reason — so with two overlapping renames the first to finish lifts the read-only overlay for a library whose walk is still running, and a rename in a second library replaces (unfreezes) the first library entirely.**

*Scenario:* `renamesInFlight` is keyed per source path and only blocks the double-commit of the SAME path (:6695), and the sidebar tree stays fully interactive during a cascade (:6829-6833) — the codebase already treats overlapping cascades as reachable: `cascadingPaths` is refcounted specifically because 'the user spam-renames two notes in the same library' (store.ts:723-733) and `cascadingLibraries` likewise (:772-799). Recipe: rename note A in library L (freeze={L}, walk starts), then rename note B in library M a second later. B's `cascadeFreeze.set(new Set([M]))` drops L from the frozen set while L's walker is mid-rewrite; when B (or A) finishes first, its `finally` runs `cascadeFreeze.set(new Set())` and lifts the freeze for the other cascade's whole library. Every pane in the still-walking library loses both the input block and the 'Updating links…' signal — the user sees a normal editable not


### [MED] `src/lib/components/SecondScreenPage.svelte:333` — resource-leak

**Second-screen peek tabs create a NoteModel per peeked note (`peek-<path>`) that is never disposed — `closePeek`, peek replacement, and `onWorkspaceRestore`'s `openTabs.set([])` all drop the tab without `closeNoteModel`.**

*Scenario:* Each peek mounts NoteEditor, whose `$effect` calls `ensureModel(tab.id, tab.path, tab.content)` (NoteEditor.svelte:185-187) → `openModel` inserts into the module-level `models` Map (noteModel.ts:94). The id is `peek-${note.path}` (SecondScreenPage.svelte:316), so one model per distinct note. `closePeek()` (lines 332-335) sets `peekNote = null; peekTab = null` with no `closeNoteModel`; `loadPeekPreview` overwrites `peekTab` with no disposal of the previous model; and `onWorkspaceRestore` (line 719) calls `openTabs.set([])` directly rather than `flushDisposeClearTabs`, which store.ts:2791 documents as the reason models leak ('a raw openTabs.set([]) alone leaks every NoteModel for the process lifetime'). Every backlink/outgoing click in the split companion (lines 1175/1198) opens a peek, so a long second-screen browsing session on a 7,600-note universe accumulates one full body+props copy p


### [MED] `src-tauri/src/search.rs:6559` — index-divergence

**`note_links.target_cid_cn` is resolved ONLY at the SOURCE note's index time and is never re-resolved when the target note is created later; no trigger, no backfill and no reconcile re-derives it, so a forward-link written before its target exists keeps `target_cid_cn = NULL` forever.**

*Scenario:* User types `[[Ibn Khaldun]]` in note A while no note by that name exists yet (the normal PKM forward-link). index_note (search.rs:6559) resolves `target_cid_cn` via `SELECT cid_cn FROM note_meta WHERE name_lower = ?` → no row → NULL is stored on the edge. A week later the user creates `Ibn Khaldun.md`. Creating B re-indexes B only; nothing touches A's edge row. `maintain_incoming_after_save` (search.rs:1629) repairs `incoming_count` by NAME, so the backlink badge looks correct — but `target_cid_cn` stays NULL. The MIG-083 Mode-2 staleness probe `stale_probe_sql` (review.rs:131-144) JOINs `note_meta dep ON dep.cid_cn = jl.target_cid_cn` AND filters `jl.target_cid_cn IS NOT NULL AND != ''`, so that dependency is invisible to the Reviewer forever: when B's content later changes, A is silently never flagged as stale. `review_rehearse.rs:150/205` has the same JOIN. The only repair paths are (


### [MED] `src-tauri/src/search.rs:9925` — index-divergence

**`reconcile_filesystem` drops the outgoing-link aggregate triggers DATABASE-WIDE for the bulk walk and only repopulates via `recompute_all_outgoing` at the end; a process kill mid-walk leaves the columns permanently stale, because nothing on the next boot recomputes them (init_db only recreates the triggers).**

*Scenario:* User triggers a rebuild (Add Library / New Library / empty-index auto-recover) on a large universe. `reconcile_filesystem` calls `drop_outgoing_link_triggers(&walk_conn)` (search.rs:9925) — triggers are schema-level, so they are gone for the LIVE `state.db` connection too (the comment at 9919 claiming 'live single-edge edits on state.db still maintain the columns write-time' is false). The walk then runs `index_note` per file, whose incremental edge diff DELETEs and re-INSERTs changed `note_links` rows (search.rs:6663-6724) with no trigger firing. The user closes the app (or it crashes) mid-walk, so `create_outgoing_link_triggers` + `recompute_all_outgoing` at search.rs:9942-9945 never run. On next boot `init_db` recreates the triggers (search.rs:4399) but recomputes nothing; `links_backfill::maybe_schedule` is a no-op because `is_needed` sees the version current and the vocabulary finge


### [MED] `src/lib/editor/yamlDoc.ts:362` — content-loss

**The CST splice+append path in composeFrontmatter deletes YAML comments attached to the edited or removed key, because a preceding comment lives in that map item's `start` tokens.**

*Scenario:* A note's frontmatter is:\n```\ntitle: Foo\n# an important note about tags\ntags:\n  - a\n  - b\nstatus: draft\n```\nThe user adds one tag in the Properties panel. `tags` is type 'list', so the SET branch at line 383 is skipped and the ADD branch splices the `tags` map item out (line 387) and appends a fresh `tags:` block. Verified against the vendored yaml CST: the comment `# an important note about tags` is stored in `items[1].start`, so the splice removes it. Result on disk: `title: Foo\nstatus: draft\ntags:\n  - a\n  - b\n  - c` — the user's comment is gone with no error, and the file re-parses cleanly so nothing ever notices. Same loss on a key REMOVE (line 362) and on editing any QUOTED scalar: verified that `title: "Book Review"` has CST token type `double-quoted-scalar`, which the SET guard (`item.value.type === 'scalar'`) rejects, so every Constellation-created note's title edit 


### [MED] `src-tauri/src/sources/mod.rs:481` — content-loss

**rewrite_frontmatter_sources drops any line whose TRIMMED text starts with `sources:`, so an indented `sources:` belonging to a nested map is silently deleted; rewrite_frontmatter_content_type has the identical defect.**

*Scenario:* A note carries\n```\n---\ntitle: X\ncitation:\n  sources: interview transcript\n---\n```\nThe user sets the note's classification sources (Properties panel / Cataloger accept). The strip loop at line 481 does `let trimmed = line.trim_start(); if trimmed.starts_with("sources:") { continue; }` — indentation is not checked, so the nested `  sources: interview transcript` line is dropped from the file, leaving `citation:` with no value. The user's data is gone from the .md with no error. Identical shape at line 1096 for `content_type:`. This is the same indentation-is-data class already fixed in libraries.rs::update_frontmatter_title and set_frontmatter_parent (both now use is_top_level_key_line) and never propagated here — a Whole-Ecosystem-Fix-Law gap. Secondary in the same function: `new_fm_lines.join("\n")` at line 506 (and 1116) normalizes the frontmatter block from CRLF to LF on every 


### [MED] `src/lib/libraries/propertyTypeRegistry.ts:33` — silent-data-loss

**`boot_bundle.rs:130` degrades an unreadable/corrupt `property-types.json` to `{}`; `seedFromBundle` then sets `cache = {}` and `loaded = true` (indistinguishable from "this Universe has no property types"), and the first `setRegisteredType` (line 66) → `persistPropertyTypes` (line 41) writes that near-empty cache back over the file.**

*Scenario:* `.constellation/property-types.json` is locked or corrupt when `constellation_boot_bundle` reads it. `read_universe_property_types(...).unwrap_or(Object{})` hides the failure; `seedFromBundle({})` wipes the in-memory cache and marks it loaded. The user opens any note's PropertyEditor and sets one key's type (e.g. `due` → date): `setRegisteredType` writes `cache = { "Lib": { "due": "date" } }` and the 500 ms debounce persists exactly that over `property-types.json`. Every library-wide property-type assignment the user had built up is silently erased — no error surfaces (the `.catch` at line 46 only reaches `console.error`, dev-only), and every property in every note silently reverts to heuristic type detection. `loadPropertyTypes` (line 17) has the identical `catch { cache = {}; loaded = true; }` shape on the fallback path.


### [MED] `src/lib/libraries/store.ts:6391` — silent-data-loss

**`persistWorkspaces` writes the whole `workspaces` store over `workspaces.json`, but the load side treats a failed read as "no workspaces": `boot_bundle.rs:105` does `read_universe_workspaces(...).unwrap_or(Array[])` and `+layout.svelte:2578` only sets the store `if (Array.isArray(...) && length > 0)` — so an unreadable file and an empty file are the same state, and the next save clobbers the real one.**

*Scenario:* `.constellation/workspaces.json` is transiently unreadable at boot (sync/AV lock). The bundle returns `[]`; the `length > 0` guard means the `workspaces` store is left at its initial `[]`. The user later saves a new desk layout (`saveWorkspace`) or deletes one: `persistWorkspaces` atomically renames a one-element (or empty) array over `workspaces.json`, destroying every named workspace the user had saved — no error path exists (the invoke is fire-and-forget with a dev-only `console.error`). The same collapse happens on a Universe switch, where the store additionally still holds the PREVIOUS Universe's workspaces, so A's layouts get written into B's file.


### [MED] `src-tauri/src/universe.rs:1196` — silent-data-loss

**`link_library_as_universe` gates only on `.constellation/universe.json` existing (line 1150); if that one file is absent while the rest of `.constellation/` survives, it unconditionally overwrites `libraries.json` with a single-entry array (line 1192, non-atomic `fs::write`) and resets `bookmarks.json`/`settings.json`/`workspaces.json`/`property-types.json` to empty (lines 1196-1199) — with NO `.exists()` guard, unlike the parallel `migrate_legacy_data:1740-1750` which guards every one of those same writes.**

*Scenario:* A Universe folder's `.constellation/universe.json` goes missing while its siblings remain — a sync-conflict rename (`universe.sync-conflict-….json`), a partial restore from backup, or an interrupted `create_universe` whose `fs::write` at line 628 failed after `create_dir_all` succeeded. `open_existing_universe` refuses the folder ("not a valid Constellation universe"), so the user reaches for the other entry point, "link an existing folder", in the setup wizard (`UniverseSetup.svelte:120 handleLinkLibrary`). `link_library_as_universe` finds no `universe.json`, takes the create branch, and silently overwrites the surviving state: every registered Library in that Universe is replaced by one entry pointing at the folder root, and settings / workspaces / property-types are truncated to `{}`/`[]`. The command returns Ok with a healthy-looking new UniverseEntry; the user sees a Universe that h


### [MED] `src/lib/components/ReviewStatusPanel.svelte:97` — swallowed-write-error

**`set_review_priority` — a fail-closed Rust command — is wrapped in a bare `catch {}` at both review surfaces, while the slider keeps rendering the dragged value (`priorityDraft ?? effective`), so a refused write reads on screen as a successful one.**

*Scenario:* `commitPriority` (ReviewStatusPanel.svelte:97; twins at ReviewerView.svelte:286 and :290) is `try { await invoke('set_review_priority', …); await load(); onRefresh?.(); } catch {}`. `set_review_priority` (review.rs:635-692) returns `Err("database not ready")` at three points when `SearchState`/`state.db` is unavailable, and `?`-propagates a failed `link_life::append`/`fsync` of the earned ledger — the MIG-104 Slice-4 fail-closed contract ("if the record cannot be made durable, the DB change must NOT happen"). Concrete: the user drags the priority slider on a note within the boot/universe-switch window before search init completes (or while the ledger file is momentarily locked by AV/OneDrive). `oninput` has already set `priorityDraft` to the dragged number; the invoke rejects; the empty catch discards it; `await load()` / `onRefresh?.()` never run, and in ReviewStatusPanel `priorityDraft


### [MED] `src/lib/components/GlobalTasksView.svelte:139` — false-success

**All three task-toggle surfaces swallow a failed `toggle_task` with console.error only and skip their post-toggle refresh, leaving the DOM checkbox visually checked (`checked={task.completed}` never changes, so Svelte never re-writes it) while the `- [ ]` on disk is untouched.**

*Scenario:* `handleToggle` (GlobalTasksView.svelte:135-144) awaits `toggleTaskReconciled` then `loadAllTasks()`, both inside one try whose catch is `console.error` only. Identical shape at +layout.svelte:9174 (right-sidebar TasksPanel `onToggle`, refresh via `scanNoteTasks` after the await) and +layout.svelte:2017 (`handleCalendarToggleTask`, refresh via `refreshCalendarData()` after the await). `toggleTaskReconciled` (store.ts:1123-1140) rethrows whatever `toggleTask(filePath, lineNumber)` rejects with — the Rust `toggle_task` gate-writes the `.md`, so a file locked by OneDrive/Syncthing/AV, or a `gate_write` refusal, rejects. Concrete: user clicks a task checkbox in the Tasks view while OneDrive holds the note's `.md`. The browser flips the DOM checkbox on click. `toggleTaskReconciled` throws; the catch logs to a console that release binaries disable; `loadAllTasks()` (line 140) — the only thing t


### [MED] `src-tauri/src/sky_backfill.rs:154` — index-divergence

**The Sky back-fill carries its resume cursor across a universe switch in a Rust local while the connection under `state.db` is swapped, then stamps `schema_versions.sky` complete on the NEW universe — permanently disabling that universe's Sky back-fill after processing none of its notes.**

*Scenario:* Universe A is opened for the first time; `ensure_search_db_ready` (search.rs:9635) calls `sky_backfill::maybe_schedule`, which spawns `run()` (sky_backfill.rs:102). The loop at :150-161 resolves the connection FRESHLY from `state.db` on every batch (`process_batch(&state.db, &last_path)`, :151) but keeps `last_path` in a Rust local, and NO back-fill anywhere checks `SearchState.federation_generation` (grep: only search.rs references it). After batch N the loop calls `write_cursor` (A's cursor, e.g. `E:/Cognitive Knowledge/Notes/M….md`) and sleeps 50 ms. Inside that sleep the user switches to universe B: `invalidate_search_state` NULLs `state.db`, and the next command's `ensure_search_db_ready` installs B's connection into the SAME slot (fast on a universe already opened this session — init_db is then PRAGMAs + `CREATE TABLE IF NOT EXISTS` against a warm page cache). The loop wakes and ru


### [MED] `src-tauri/src/links_backfill.rs:184` — index-divergence

**The outgoing-link aggregate back-fill has the same unguarded cursor-across-switch + unconditional stamp, silently marking a universe's `note_meta.outgoing_*` aggregates materialized when none were.**

*Scenario:* `run()` (links_backfill.rs:130) reads the cursor at :175 then loops `process_batch(&state.db, &last_path)` (:177) with a 50 ms inter-batch sleep (:187), re-resolving the connection from the shared `state.db` slot each time and checking no generation. A universe switch landing in one sleep swaps the connection to B; the next batch scans B with A's cursor, drains at 0, and `finalize(&state.db, run_fp)` (:184 → :447) stamps BOTH `schema_versions.links_outgoing` and `schema_versions.links_vocab` into B and clears B's cursor — with no completeness verification. `is_needed` is then false for B forever, so B's `note_meta.outgoing_count` / `outgoing_link_types_json` / `outgoing_top_rank` stay at their pre-back-fill values for every note that predates the write-time triggers. Every surface that reads those write-time columns (link badges, connectivity/fragility lenses via `connectivity::derives_f


### [MED] `src-tauri/src/review.rs:1122` — swallowed-write-error

**`sync_action_to_row` discards the write result with `let _ = f(conn)` and no-ops entirely when `state.db` is `None`, so a ✓ Reviewed / Snooze / Dismiss is recorded in review-pulse.json but never reaches `review_schedule`, the table the queue actually reads.**

*Scenario:* `mark_reviewed` / `snooze_note` / `dismiss_note` (review.rs:707, 737, 763) never call `ensure_search_db_ready`. Each writes `review-pulse.json` durably, then calls `sync_action_to_row(&app, …)` (:728, :752, :775), whose body is `if let Ok(db) = state.db.lock() { if let Some(conn) = db.as_ref() { if is_stamped(conn) { let _ = f(conn); } } }` (:1119-1124) and which returns `()` — the command then returns `Ok(())` unconditionally. Two ways the row write is lost with no signal: (a) `state.db` is `None` during the post-switch cold-init window; (b) `conn.execute` fails (e.g. SQLITE_BUSY past the 5s busy_timeout while a back-fill batch or `reconcile_filesystem` holds the writer) — `let _ =` eats it. Once `schema_versions.review` is stamped, `get_due_notes` reads ONLY `review_schedule` (:85-90) and `upsert_schedule_row` re-derives `last_reviewed` / `interval` from the ROW, explicitly not from re


### [MED] `src/lib/components/PropertyEditor.svelte:430` — content-corruption

**`seededRows` is cloned from the RAW `sourceProps` while `editableProps` is built with display normalisation (`forced ?? registeredType ?? p.type` plus synthesised `listItems`), so `touchedSince(seededRows, editableProps)` reports keys the user never touched as touched — defeating the PJ-174 #1e untouched-key protection exactly for `parent`/`contains` and any key with a registered-type override.**

*Scenario:* `samePropRow` (propRow.ts:77) compares `type` and `listItems`. For a note whose frontmatter holds the scalar `parent: "[[Root A]]"`, `parseFrontmatter` yields `{type:'link', listItems:undefined}` (detectPropertyType:183; `parent` is not in LIST_KEYS), while line 437–441 forces `{type:'list', listItems:['[[Root A]]']}`. So immediately after seeding, and with zero user interaction, `touchedSince` (line 928) already contains `parent`. Two consequences. (a) Cross-writer revert — the exact class the guard exists to prevent: the user edits `stage` (arming the 800 ms debounce); within that window another writer changes `parent` to `[[Root B]]` (the Structure panel's Keep/Move-here resolve, a second-screen save adopted via `adoptDisk`, or `bases.rs::update_note_property`); the seeding effect SKIPS the re-seed because `localEditPending` is true (line 428), so `editableProps` still shows `[[Root A


### [MED] `src/lib/libraries/store.ts:1723` — swallowed-write-error

**`saveCollections()` fires `invoke('save_universe_collections')` with `.catch(e => console.error(...))` — a failed write of the collections source-of-truth is reported only to a console that release builds do not expose, and there is no retry, no dirty flag, and no banner.**

*Scenario:* Every collection mutation (createCollection:1730, renameCollection:1735, deleteCollection:1741, addToCollection:1755, removeFromCollection:1760, adoptCollectionIdentities:1795, migrateCollectionPath:1806, the one-time bookmarks migration:1836) calls this one fire-and-forget helper. If `save_universe_collections` returns Err — a transient AV/sync lock on `collections.json.tmp`, a full disk, a network drive stall, or the rename-collision described in the universe.rs:117 finding — the Svelte store keeps the change so the panel shows the new collection/star, while disk still holds the old array. Nothing retries (contrast session.ts:154, which sets `dirtyRetry` and re-attempts on the next mutation) and nothing surfaces (contrast `standardSaveEnv`'s `onError` → save-health banner). Per the project's own devtools-is-DEV-ONLY rule the `console.error` is unobservable in a release build, so the us


### [MED] `src-tauri/src/libraries.rs:559` — freeze-hang

**`write_note` — the single command every note save goes through — is a sync `#[tauri::command]`, so the whole atomic-write (temp create + write + `sync_all()` fsync + `ReplaceFileW` + up-to-500 ms of `thread::sleep` retries + the journal's global-mutex append) executes inline on the WebView2 IPC dispatch thread.**

*Scenario:* User types in NotePane on a Universe stored on a synced/network folder (OneDrive, Syncthing, external drive) with real-time antivirus active. Every 1500 ms the autosave fires: `NoteEditor.svelte:326 writeNote(filePath, content, 'editor_save')` → `store.ts:2282 invoke('write_note')`. Because the command is sync, Tauri's macro takes the `body_blocking` path and runs it on the dispatch thread; `write_gate.rs:234 atomic_write` then does `File::create(tmp)` + `write_all` + `f.sync_all()` (FlushFileBuffers) + `ReplaceFileW`. The UI thread is parked for the whole fsync. When AV holds the freshly-created temp file, `write_gate.rs:274-280` retries 5× with `thread::sleep(50/100/150/200 ms)` — 500 ms of the UI thread sleeping, plus the retry sleeps inside `journal_ext`'s global mutex contention. Symptom: the caret stops, keystrokes queue, and every other IPC message (search, tree, stats) queues beh


### [MED] `src-tauri/src/libraries.rs:2194` — freeze-hang

**`list_universe_folders` is a sync `#[tauri::command]` that recursively walks the directory tree of every library (and every federated cUniverse library) with a `p.is_dir()` metadata syscall per entry, on the IPC dispatch thread.**

*Scenario:* User right-clicks a note in the sidebar → Move (`+layout.svelte:6444 openMoveDialog` → `buildUniverseFolderEntries` → `+layout.svelte:6397 await invoke('list_universe_folders')`), or picks a destination for New-note-from-template (`+layout.svelte:5006`). `collect_folders` (`libraries.rs:2212`) recurses to depth 30 across every registered library, calling `entry.file_type()` and `p.is_dir()` on every single directory entry — on a 7,600-note Universe that is 8,000+ metadata syscalls, all on the blocked UI thread. With a cold filesystem cache (first Move of the session, or an external/network-mounted library) the window is unresponsive for a second or more while the dialog shows `loading: true` that cannot repaint, because the thread that would repaint it is the thread doing the walk. No error, no timeout — it just hangs and then pops open. Same class as the already-fixed `scan_note_stages`


### [MED] `src/lib/editor/livePreview.ts:285` — resource-leak

**`_embedCache` is a module-level `Map` that is never cleared, never evicted and never invalidated, and its values carry full transcluded note bodies plus whole-directory listings — so it grows for the entire session and also serves permanently stale transclusion content.**

*Scenario:* User browses a knowledge base that uses `![[Note]]` transclusions. Each distinct `libraryPath|notePath|target` triple hits `livePreview.ts:419 _embedCache.set(cacheKey, res)`; the stored `EmbedResolution` includes `note_body` (the ENTIRE transcluded note's text), `attachment_folder_listing` (a full directory listing), `tried_paths` and `similar_files`. There is no `.delete`, no `.clear`, no size cap and no invalidation hook anywhere in the file (`grep -n '_embedCache' livePreview.ts` → only the declaration, one `get`, one `set`). Browsing 500 notes with transclusions retains hundreds of full note bodies in JS heap for the whole session — the app's memory climbs monotonically until restart, with no error and nothing in the UI to show why (Rule 4 violation, and the same shape as the already-registered `_imageCache` leak two declarations above, which this one is NOT a duplicate of). Seconda


### [LOW] `src-tauri/src/libraries.rs:6753` — silent-data-loss

**move_into_trash_folder treats any gate_rename failure as 'cross-device' and falls back to fs::copy, but since PJ-140 #18 gate_rename returns Err specifically for a destination that already exists — converting the gate's never-clobber refusal into a silent overwrite of an earlier trashed file.**

*Scenario:* write_gate.rs:593-596 (added 2026-07-25) makes gate_rename return Err('An item with this name already exists at the destination.') when the destination exists under the lock. move_into_trash_folder's fallback at libraries.rs:6753 branches on `.is_err()` without inspecting the reason and then runs `fs::copy(source, &dest)` (:6760) / copy_dir_recursive (:6756), both of which overwrite the destination. Concrete run: the user trashes 'Notes.md'; the de-collide loop at :6730-6741 picks '.trash/Notes 1.md' because '.trash/Notes.md' already exists; between that exists() probe and gate_rename taking the lock, a second delete (batch delete, or the second-screen window) lands its own file at '.trash/Notes 1.md'. gate_rename correctly REFUSES with Err; the fallback then copies over it, destroying the other trashed note's content, and returns Ok(()) — the delete reports success. The gate's own comme


### [LOW] `src/lib/libraries/store.ts:4138` — resource-leak

**`deleteWithSetting` removes a deleted note's tab by filtering `openTabs` but never calls `closeNoteModel(t.id)`, so every note deleted while open leaves its full NoteModel (props array + whole-body CM6 `Text` rope) in the module-level `models` Map for the life of the process, still keyed to a path that no longer exists on disk.**

*Scenario:* The user deletes an open note from the file tree: `handleDeleteConfirm` (+layout.svelte:6669) → `deleteWithSetting(path)` (store.ts:4116). It moves the file to trash, clears the path-keyed write-ahead/recent-write state (line 4137), then at line 4138 does `openTabs.update(tabs => tabs.filter(...))` — dropping the tab object only. Every sibling departure primitive in this file disposes the model explicitly and says why: `closeTab` calls `closeNoteModel(tabId)` at store.ts:3098 ("MIG-076 §C — dispose this tab's content model"), and `flushDisposeClearTabs` deliberately does `openTabs.set([])` → `await tick()` → `for (const t of departing) closeNoteModel(t.id)` at store.ts:2791 with the comment "a raw `openTabs.set([])` alone leaks every NoteModel for the process lifetime". `deleteWithSetting` — and the batch loop at +layout.svelte:6658 that calls it once per selected item — never got that t


### [LOW] `src-tauri/src/search.rs:10247` — index-divergence

**Write-time maintenance of `sky_nodes.stratum`/`maturity` is gated on the UNRELATED incoming-links backfill stamp, while the per-edge sky stratum/maturity triggers are dropped unconditionally on every boot — so whenever that stamp is absent there is no sky maintenance at all.**

*Scenario:* `init_db` unconditionally executes `DROP TRIGGER IF EXISTS note_links_sky_stratum_ai/ad/au` and `note_links_sky_maturity_ai/ad/au` (search.rs:4310-4314 and 4362-4367) — PJ-066 §B4 removed them in favour of the Rust diff. But in `reindex_single_note` the Rust diff is reached only through `if let Some((old_t, old_n, old_a)) = inc_old { … maintain_sky_after_save(…) }` (search.rs:10247-10257), and `inc_old` is `None` unless `crate::incoming_links_backfill::is_stamped(conn)` (search.rs:10207). So on any DB where that stamp is missing, edge changes update `note_links`/`sky_links` but `sky_nodes.stratum` and `.maturity` are maintained by nothing. Reachable two ways: (1) the whole first-boot window of an existing large universe — the incoming pass does a full-table recompute plus `CREATE INDEX idx_nl_tnl` measured at ~50 s on 234k edges (incoming_links_backfill.rs:88-94) — every save in that win


### [LOW] `src/lib/components/PropertyEditor.svelte:929` — false-success

**The auto-`updated` date is computed but can never be written for a note that already has an `updated:`/`modified:` date property, because the touched-keys filter excludes it.**

*Scenario:* A note has `updated: 2026-07-01` (type date). The user edits `stage` in the Properties panel. `touched = touchedSince(seededRows, editableProps)` (line 928) is computed from the PRE-auto-date rows, so `updated` is identical on both sides and is NOT in `touched`. `plan()` is then called with `withAutoUpdatedDate(editableProps)` — the row now carries today's date — but propsCommit.plan's SET branch does `if (touchedKeys && !touchedKeys.has(row.key)) continue`, so the op is never emitted. The note's `updated:` field freezes at whatever date it was first added and silently stops tracking edits, while the code's own comment at line 919 claims the rule is applied. The frontmatter-derived `note_meta` mirror inherits the stale value.


### [LOW] `src-tauri/src/cece/orchestrator.rs:153` — freeze-hang

**`run_one_safe` wraps the cataloger worker in `std::thread::scope`, which JOINS all spawned threads when the closure returns — so the `rx.recv_timeout(timeout)` budget on line 160 cannot actually bound anything, and a blocked cataloger hangs `classifier_suggest_for_note` indefinitely despite the documented V3-§8.r4.4 timeout.**

*Scenario:* The user has semantic search enabled and a bulk embed is running: `constellation_embed_notes` (src-tauri/src/embeddings.rs:428) takes `state.engine.lock()` and holds it across its whole `for note in &notes` loop of multi-second ONNX inferences — minutes for a large batch. Concurrently the CECE background scan (classifier/scan_job.rs:133) classifies a note; the Semantic/Linguistic cataloger calls `embed_text` (cece/wiring.rs:128), which blocks on that same `state.engine.lock()`. The 2 s budget from `cataloger_timeout` expires and `recv_timeout` correctly returns `Err(Timeout)` at orchestrator.rs:171, producing an abstain trail — but control then falls out of the `std::thread::scope` closure, and `scope` blocks until the worker thread finishes, i.e. until the embed batch releases the engine mutex minutes later. Observable damage, all silent: the scan thread stalls with no progress event an


### [LOW] `src-tauri/src/cece/reliability.rs:199` — toctou

**`sweep_tmp_orphans` unlinks every `.cataloger_reliability.*.tmp` in the Library's `.constellation/` dir, and it runs from `load_or_default` WITHOUT holding `RELIABILITY_LOCK` — so it can delete another thread's in-flight `NamedTempFile` between its write and its `persist`, silently discarding the user's reliability correction (the deletion error is swallowed by `let _ =`).**

*Scenario:* On macOS/Linux (Cross-Platform-by-Design rule: the app ships on both, and POSIX `unlink` succeeds on an open file where Windows would refuse with a sharing violation). The user Accepts a Source Review card; `cece_record_correction_for_card` → `update_reliability_from_correction` takes `RELIABILITY_LOCK` (line 319) and calls `save`, which creates `.cataloger_reliability.<rand>.tmp` in `<library>/.constellation/` (line 152-160), writes it and `sync_all`s it. Concurrently the CECE background scan thread is inside `classifier_suggest_for_note`, which calls `crate::cece::reliability::load_or_default(lr)` at classifier/mod.rs:165 — a path that does NOT take `RELIABILITY_LOCK` — and its first statement is `sweep_tmp_orphans`, whose `fs::read_dir` enumerates that live temp file and `fs::remove_file`s it at line 199, with the result discarded by `let _ =`. `tmp.persist(&path)` then fails with ENO


### [LOW] `src/lib/components/StyleSetter.svelte:962` — swallowed-write-error

**Every Style Setter preset mutation (`saveAsStyle` / `confirmRename` / `removeStyle` / `updateStyle`) awaits `saveStylePresets` with no try/catch, so a rejected `save_style_presets` becomes an unhandled rejection while the in-memory gallery already shows the change.**

*Scenario:* `saveStylePresets` (stylePresets.ts:134) is a bare `await invoke('save_style_presets', { presets })`; Rust `save_style_presets` (style_presets.rs:48-52) does a plain non-atomic `fs::write` and returns `Err("Failed to save style presets: …")` on any IO failure. StyleSetter mutates `savedStyles` FIRST and then awaits the persist at :962 (save-as), :985 (rename), :989 (delete) and :1001 (update) — none wrapped in a try/catch, so a rejection surfaces only as an unhandled promise rejection, invisible in a release build. Concrete: the user deletes an old Style and saves the current look as a new one while `style-presets.json` is momentarily locked (backup/AV) or the disk is full. The gallery immediately shows the new Style and the old one gone; the on-disk file is untouched (or, with the partner failure mode, `loadStylePresets` at :1172 having swallowed a read error into `[]` on a prior boot, 


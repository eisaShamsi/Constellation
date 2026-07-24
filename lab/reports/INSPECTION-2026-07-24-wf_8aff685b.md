# Safety Inspection register — 2026-07-24 (`wf_8aff685b-b8b`)

Whole-app cycle sweep (PJ-124: `args.files` ignored again, 8th+ occurrence — the
diff-scoped request ran whole-app). 87 agents, 14 hunt scopes, every candidate
adversarially refuted before confirmation.

**62 confirmed** — 2 APP-KILLER, 14 HIGH, 27 MED, 19 LOW.

This run REPLACES `wf_acc3ca2c-4f6`, which was killed ~2 min in by the Boss-directed
PCS: 14 agents started, **0 produced any output**, so that run had no findings to
triage — nothing was hidden, nothing salvageable.

## Fixed in this pass (8)

Both APP-KILLERs, the three findings that live in the Slice-2 batch this per-build
inspection was owed for, and the library-resolver class (one canonical-resolver swap
at each of four sites — the fix was a one-liner because the correct resolver already
existed and the defective sites simply did not call it).

| # | Sev | Where | Fix |
|---|-----|-------|-----|
| 0 | APP-KILLER | `store.ts` parseFrontmatter + `yamlDoc.ts` | seq-of-maps / spaced block lists no longer projected as truncated EDITABLE lists; `immutableBlockKeys` now refuses any seq holding a non-scalar. Regression test `tests/mig-103/seqOfMapsRoundtrip.test.ts` (7 cases). |
| 1 | APP-KILLER | `sources/bulk_ops.rs` | Approve-All now announces its watcher-suppressed frontmatter writes (batched), so open notes re-base instead of silently erasing the batch on the next keystroke. |
| 4 | HIGH | `libraries.rs:1807` | rename wikilink-cascade reindex → `library_name_for_path`. |
| 19 | MED | `libraries.rs:918` | create_note first-index → `library_name_for_path`. |
| 25 | MED | `+layout.svelte` handleRenameComplete | cascade library resolution → shared boundary-checked `libraryForPath`. |
| 41 | MED | `universe.rs` merge_fields_into_template | unguarded RMW → `gate_rmw` (one critical section) + announce. |
| 58 | LOW | `props_reparse_backfill.rs` | completeness verify before stamping (sibling `note_body_backfill::finalize` pattern) — a universe switch mid-run can no longer stamp an unconverted universe as done. |
| 59 | LOW | `TemplateStudioDetail` | detail column keyed on the selected kind — the previous kind's naming evidence can no longer head another kind's panel. |

Also fixed en route: `libraries.rs:1203` and `universe.rs:2119`, the two further
first-match sites the finding named but did not separately number.

## NOT fixed — awaiting a Boss ruling (54)

Per WA#6 these are surfaced explicitly, never silently parked. They are whole-app
findings unrelated to the Slice-2 batch or MIG-103; several cross subsystem
boundaries and need their own `/migration` rather than a drive-by fix inside this
build. Full detail for every one is in the scratchpad register; the ledger carries
them as PJ items.

### [2] HIGH — index-divergence — `src-tauri/src/libraries.rs:1015`

Folder rename has NO index cascade for descendants, and the gated rename's watcher suppression blinds the only same-session heal: every note under the renamed folder keeps its old path in note_meta/note_links/notes_fts/sky/review for the rest of the session.

### [3] HIGH — index-divergence — `src-tauri/src/libraries.rs:6304`

delete_path mode='trash' on a FOLDER purges nothing from the index: reindex_delete_note is called with the folder path (exact-path DELETE matches zero note rows) and the gate_rename into .trash suppresses the watcher events that would have prefix-purged the descendants.

### [5] HIGH — content-loss — `src/lib/libraries/store.ts:2325`

openNoteTab's in-place reuse registers/checks the _navTokens supersede token ONLY when the outgoing model is dirty at the gate (line 2325). A CLEAN departure neither bumps nor checks the token, so a concurrent loadTabHistoryEntry (Alt+Left/Right) on the same tab is never superseded and never supersedes: both appliers land, last-wins, and the loser's openNoteModel (store.ts:2362 / 1443) force-replaces a model that may have gone DIRTY in the gap.

### [6] HIGH — silent-data-loss — `src/lib/libraries/store.ts:2581`

drainCidEnsure's adopt branch calls clearWriteAhead(tab.path) after force-adopting cid-bearing disk into a born-clean restored model — destroying a write-ahead net that resolveNoteContent's restore path DELIBERATELY preserved as 'may be the ONLY copy of unsaved edits' (store.ts:2132-2137, identity-unproven rejection with preserveNet).

### [7] HIGH — content-loss — `src/lib/libraries/store.ts:3613`

renameItem's post-rename force re-seed (openNoteModel with fresh disk content) unconditionally replaces the note model, discarding keystrokes typed during the rename window; for a PATH-CHANGING rename (non-canonical/human-named file, e.g. any linked Obsidian library note) every rescue path is identity-REFUSED, so the typed text is silently lost with no error, no net, no banner.

### [8] HIGH — silent-data-loss — `src/routes/+layout.svelte:6783`

cascadeFreeze is a plain set/clear while the underlying write-gate (cascadingPaths) is refcounted for overlapping cascades: two concurrent renames in the same library (renamesInFlight only blocks the SAME oldPath) let the first rename's inner finally `cascadeFreeze.set(new Set())` clear the freeze overlay for ALL panes while the second rename's walker is still running.

### [9] HIGH — toctou — `src/routes/+layout.svelte:6691`

The cascade's gate/flush/freeze all operate on a snapshot of tabsInLibrary taken BEFORE the multi-second walker runs, but nothing blocks opening a new tab in that library mid-cascade; a note opened and edited inside the window is in result.rewritten (it was never excluded), and reloadTabsFromDisk force-adopts it — the documented PJ-092 invariant ('must NEVER be handed a dirty path', store.ts:802-809) is violated by any tab born after the snapshot.

### [10] HIGH — content-loss — `src/lib/libraries/store.ts:1633`

A block scalar (`summary: |` / `key: >`) is projected by parseFrontmatter as an ordinary text property whose value is the literal '|' or '>' character — the multi-line content is invisible in the panel — and editing that row replaces the entire block on disk with the typed scalar (yamlDoc.ts:318-323: block-scalar CST token is not 'scalar', so the item is spliced and re-serialized from the projection).

### [11] HIGH — concurrency-race — `src/lib/components/PropertyEditor.svelte:851`

Two live PropertyEditor instances for the SAME note (NotePane-embedded at NotePane.svelte:1639 + right-sidebar standalone at +layout.svelte:8951) each hold an independent full editableProps copy and each save the WHOLE array; a save from one instance mutates tab.content directly (no openTabs.update, deliberately) so the peer instance's `properties` prop never recomputes — the peer stays stale indefinitely, and its next edit writes the stale full array through editNoteProps, silently reverting on disk every property change made in the other instance since.

### [12] HIGH — silent-data-loss — `src-tauri/src/review.rs:762`

save_pulse_data writes review-pulse.json with a plain non-atomic fs::write (truncate-then-write), while load_pulse_data (lines 747-757) swallows any read/parse failure and silently falls back to ReviewPulseData::default() with no backup taken — the exact G6 anti-pattern that was fixed for every other persisted-JSON file (universe.rs::atomic_write, libraries.rs::save_libraries) but missed in review.rs. review-pulse.json is the AUTHORITATIVE review action state: the periodic reconcile (search.rs:9195) rebuilds the review_schedule table FROM it via recompute_all_in.

### [13] HIGH — false-success — `src/lib/lens/store.ts:132`

updateNoteProperty resolves void after silently SKIPPING the write: when the edited row's note is open+dirty and flushOpenTabOrAbort fails, the function `return`s before invoke('update_note_property') — the promise still RESOLVES, so BaseTab.svelte:257 runs `row.dimensions[dim] = next` (optimistic commit), saveError stays null, and the spinner clears.

### [14] HIGH — index-divergence — `src/lib/libraries/store.ts:534`

The post-durable-write `constellation_search_reindex` is fire-and-forget with `.catch(()=>{})` at every runtime save site (store.ts:442 retrySaveFailure, :534 navFlushEnv, :1265 saveTabContent runPostSave; NoteEditor.svelte:230/272/297/356; +layout.svelte:1607) — AND the write itself is watcher-suppressed via markRecentWrite/recentWrites, so a failed reindex has no reconcile net: notes_fts/note_meta silently diverge from the .md that was just written.

### [15] HIGH — silent-data-loss — `src-tauri/src/search.rs:8776`

The schema-version rebuild (`if needs_rebuild { let _ = std::fs::remove_file(&path) }`) deletes search.db, which is the ONLY store of user-earned Living-Link properties and review-priority overrides — they are silently destroyed on every version bump with no export, no prompt, no error.

### [16] MED — silent-data-loss — `src-tauri/src/libraries.rs:1871`

move_item silently destroys the moved note's review_schedule history (and orphans its 'rename'-source aliases + embedding): it uses delete+reindex instead of the migrate cascade rename_item_db_tail uses, so last_reviewed/interval/snooze are dropped, not carried.

### [17] MED — cross-note-bleed — `src-tauri/src/search.rs:9292`

reindex_delete_note (the delete tail for delete_path/move_to_trash/move_item) never deletes note_aliases or note_embeddings rows, leaving permanent orphans that can rebind a deleted note's aliases to a FUTURE unrelated note created at the same path.

### [18] MED — toctou — `src-tauri/src/write_gate.rs:587`

gate_rename performs fs::rename with no dest-exists re-check under the lock — safe today only because Windows MoveFile refuses an existing dest; on the mandated macOS port POSIX rename silently REPLACES, turning every caller's outside-the-lock exists() pre-check (move_item:1837, move_to_trash:6214, rename_item folder:1011) into a silent note/folder clobber window.

### [20] MED — silent-data-loss — `src/lib/components/NoteEditor.svelte:249`

handleSave's `if (saving) return;` silently DROPS a debounced save that arrives while a previous write is still in flight — and NotePane.doSave (NotePane.svelte:340) has already cleared its `dirty` flag before invoking onsave, so neither the debounce nor handleVisibilityChange re-arms. The drop bypasses noteSession's saveChains, which exists precisely to serialize overlapping saves (newest-last); the model stays dirty with no retry, no net update for the newest keystrokes, and no surfaced signal (save-health only fires on write FAILURE, not on a dropped request).

### [21] MED — content-loss — `src/lib/libraries/store.ts:1443`

loadTabHistoryEntry flushes the outgoing dirty model BEFORE the awaited resolveNoteContent read (1405 → 1413), then force re-seeds the model (openNoteModel, 1443) with no dirty re-check — keystrokes typed into the still-mounted outgoing editor during the disk-read await are silently discarded. openNoteTab has the safe ordering (read first, flush immediately before apply, synchronous gap); this site inverted it.

### [22] MED — silent-data-loss — `src/lib/libraries/store.ts:972`

adoptExternalChangeIntoTabs clears the write-ahead net for every adopter (line 972), including a restore-preserved recovery net whose model is born-clean by design (Gate #8): the PJ-102b baseline guard refuses only PHANTOM events (disk === baseline), so a GENUINELY changed disk adopts into the clean model and the only copy of pre-crash unsaved edits is destroyed — with zero cue when the tab is a background tab.

### [23] MED — swallowed-write-error — `src/routes/+layout.svelte:6702`

CascadeResult.failed and failed_truncated are never read anywhere in the frontend (grep: zero consumers of result.failed / failed_truncated in src/); per-file rewrite failures the Rust walker deliberately collects (libraries.rs:5804-5813, capped at 100 'for the toast UX') are dropped on the floor — no toast exists, and in the release build even console output is invisible (devtools disabled).

### [24] MED — content-corruption — `src/lib/libraries/store.ts:814`

reloadTabsFromDisk matches rewritten paths to open tabs by EXACT string equality (`t.path === fp` at :814 and `byPath.get(t.path)` at :834) while every other seam of the same pipeline deliberately normalizes because mismatches are documented-live: markCascading/isCascading fold separators (store.ts:724-727 'a Windows tab path travels through the JS layer with mixed separators'), the PJ-092 belt folds case+NFC (normPathLC+NFC, +layout:6712), and Rust's path_identity_key exists because 'tab.path (JS) and to_string_lossy() (Rust) may carry different forms' (libraries.rs:5866-5867). A tab whose path differs from the walker's returned form by separator, case, or NFC/NFD silently skips its reload.

### [26] MED — index-divergence — `src/lib/components/SecondScreenPage.svelte:130`

The SS's onNoteMutation subscription handles rename/move/delete ONLY by reloading split-companion panels; allNotes, the SS openTabs (their path/name), the dashboard lists, and the skyview companion link resolution are never refreshed. Because app-driven renames/moves/deletes are watcher-suppressed (write gate), `library-changed` (u5) never fires for them either — so nothing in the SS realm ever learns the mutation.

### [27] MED — index-divergence — `src-tauri/src/search.rs:6076`

index_note resolves note_links.target_cid_cn via ASCII `LOWER(name) = LOWER(?1)` against a fold_match_key'd (Unicode-folded) target, so the cid is permanently NULL for any target note whose title contains non-ASCII case (Russian, French, German, Turkish, Greek — 'Москва', 'Île-de-France'), and also stays NULL forever for links indexed before their target note existed unless the source note is later re-saved; the Mode-2 staleness lens (review.rs:135 `JOIN note_meta dep ON dep.cid_cn = jl.target_cid_cn`) silently never fires for those dependencies, and no backfill heals it (mig003_step2_backfill's target_cid_cn UPDATE keys on target_path, which is never populated → matches zero rows).

### [28] MED — freeze-hang — `src-tauri/src/search.rs:6074`

The same per-edge target_cid_cn lookup (`SELECT cid_cn FROM note_meta WHERE LOWER(name) = LOWER(?1) LIMIT 1`) is non-sargable (no LOWER expression index; idx_note_name_lower unusable) and runs once per parsed link on EVERY forced reindex (every save via reindex_single_note), inside index_note's BEGIN IMMEDIATE — the exact PJ-066 landmine class ('never COALESCE/expression-WHERE on note_meta: full scan drags inline body_text, measured 22 s cold on the 1.7–2 GB universe'). The fix-shape already exists (name_lower = folded target is exact and index-served) but this site was never converted.

### [29] MED — index-divergence — `src-tauri/src/incoming_links_backfill.rs:32`

The INCOMING aggregate columns have no vocabulary-fingerprint gate: links_backfill::is_needed re-materializes OUTGOING aggregates when the link-type vocabulary changes (MIG-067 §B fingerprint), and on_link_vocabulary_changed (search.rs:1616) schedules only that outgoing pass — but incoming_links_backfill stamps SCHEMA_VERSION=1 once and never re-runs, so stored incoming_link_types / incoming_link_types_json / incoming_top_rank remain materialized under the OLD vocabulary (a newly added custom type never appears in any note's incoming breakdown; rank order and the IN-list filter stay stale) until a coincidental target-set-changing save touches each note or the user runs a manual full reindex (reconcile_filesystem).

### [30] MED — silent-data-loss — `src-tauri/src/search.rs:6186`

Any save of a source note whose body still contains the wikilink silently resurrects a user-archived link: index_note's edge diff treats a non-'active' stored edge as 'changed' (the fast-path requires o_status=='active'; the rebuild comment itself lists 'archived→active') and DELETE+re-INSERTs it with status='active' (keeping the zeroed weight 0.0) — undoing the explicit constellation_link_archive action, removing the link from the Archived tab, and re-adding it to every cognitive surface plus sky_links (via note_links_sky_ai) with no notice; archiveLink (store.ts:3008) is DB-only, so the body wikilink always remains and the resurrect is the guaranteed outcome of the next edit.

### [31] MED — content-corruption — `src/lib/libraries/store.ts:1776`

parseFrontmatter strips surrounding quotes but never unescapes embedded \" sequences, so a double-quoted value containing escaped quotes is projected with literal backslashes; on the next edit of that key, serializeLine (yamlDoc.ts:190, yamlStringify) faithfully encodes those backslashes as CONTENT, semantically corrupting the on-disk value (and compounding on every subsequent edit round-trip).

### [32] MED — silent-data-loss — `src-tauri/src/libraries.rs:248`

save_libraries does temp-write + rename but NO fsync (f.sync_all) before the rename — diverging from universe.rs::atomic_write (lines 121-131), whose MIG-100/G6 hardening comment documents exactly this hole: 'power loss can land the rename while the data blocks are still unflushed, leaving a zero-length/garbage file under the FINAL name'. libraries.json is the registry of every library registration in the universe.

### [33] MED — silent-data-loss — `src-tauri/src/universe.rs:771`

The same-name consolidation in set_active_universe moves every entry of the nested universe dir (including .constellation/ itself, with universe.json, libraries.json, settings.json, collections.json, search.db) via `let _ = fs::rename(&src, &dest)` — every failure swallowed — then unconditionally repoints the registry entry to the parent (lines 779-784) and persists it at line 901. There is no verification that the .constellation move succeeded before the repoint.

### [34] MED — fire-and-forget — `src/lib/libraries/store.ts:1472`

saveCollections() persists the Collections/Starred JSON fire-and-forget with only console.error on failure (devtools are disabled in release builds) — no retry, no dirty flag, no banner; every mutation path (createCollection, renameCollection, toggleStarred, add/remove members) shows success in the UI regardless.

### [35] MED — fire-and-forget — `src/lib/libraries/store.ts:5803`

saveSettings() debounces 300ms then fires save_universe_settings with console-only error handling AND the pending debounce timer is not flushed by the graceful-close handshake (+layout.svelte:2932 final-flush awaits persistSessionNow + flushAllForAppClose only) — a settings change made <300ms before close never reaches disk; a failed write is never retried or surfaced.

### [36] MED — fire-and-forget — `src/lib/libraries/store.ts:5970`

persistWorkspaces() writes workspaces.json fire-and-forget with console-only error handling; saveWorkspace/deleteWorkspace update the in-memory store first, so the UI always shows the workspace saved/deleted even when the disk write failed.

### [37] MED — silent-data-loss — `src/lib/components/SenseMakingCanvas.svelte:144`

The canvas debounced save (1000ms) swallows write_canvas failures with a bare `catch {}` AND the component has no onDestroy at all — no flush of the pending saveTimer on unmount and no cleanup; the last ≤1s of canvas edits is dropped every time the view unmounts mid-debounce, and a persistent write failure loses the entire canvas session with zero surface.

### [38] MED — swallowed-write-error — `src/lib/components/ConfidencePicker.svelte:61`

applyConf and applyArchive (line 70) swallow the constellation_link_set_confidence / archive-link write with `catch { /* ignore */ }` — the popover is already closed (onClose() precedes the await), so a failed write to note_links (a Living-Link source-of-truth property) produces no feedback of any kind; only the skipped onConfidenceChange keeps the old value on screen.

### [39] MED — swallowed-write-error — `src/lib/components/BacklinksPanel.svelte:183`

linkMention() rewrites ANOTHER note's .md on disk (plain-text mention → [[wikilink]]) inside `try { ... } catch { /* ignore */ }`, and success and failure are visually identical — the panel never refreshes the unlinked-mentions list after the write, so a failed write_note is fully indistinguishable from a successful one.

### [40] MED — swallowed-write-error — `src-tauri/src/search.rs:8776`

remove_file failure during the version rebuild is swallowed (`let _ =`), yet the version file is stamped to current unconditionally at search.rs:8787-8789 — a failed rebuild is silently recorded as done, permanently cancelling the schema migration.

### [42] MED — freeze-hang — `src-tauri/src/search.rs:8267`

constellation_link_backfill_confidence is a plain sync #[tauri::command] (missing (async)) that acquires the writer state.db Mutex and runs two full-table UPDATEs over note_links ON the WebView2 IPC dispatch thread; it is live-reachable from the Settings button (SettingsModal.svelte:1472 -> :543). Its neighbours (constellation_link_archive at search.rs:8300, etc.) were converted to (async) in the 2026-07-03 note-open-freeze batch; this command was missed.

### [43] LOW — freeze-hang — `src-tauri/src/libraries.rs:6381`

copy_dir_recursive (the cross-volume folder-trash fallback) has no symlink/junction guard — `from.is_dir()` follows junctions, so a directory-junction cycle inside a trashed folder recurses unboundedly (stack overflow / disk-fill), unlike collect_md_paths which guards exactly this (libraries.rs:1889).

### [44] LOW — silent-data-loss — `src/lib/libraries/store.ts:2718`

closeTab proceeds to dispose the model on a still_dirty flush result on the assumption 'the write-ahead net + save-health banner preserve a failed write' — but the still_dirty outcome after FOUR SUCCESSFUL writes (flushIfDirty MAX=4, noteSession.ts:227) has no failed write: every pass's clearNetIf compare-and-cleared the net, and no saveHealth entry exists. closeNoteModel then discards the residual dirty delta with no net and no banner.

### [45] LOW — fire-and-forget — `src/lib/libraries/store.ts:1249`

saveTabContent's saveLocks early-return ('the model has this edit; the next save/flush persists it') pushes the concurrent property edit into the model but schedules NO re-drive: if no further doc change / prop edit / departure occurs, the model stays dirty indefinitely with no net entry (the first write's success compare-and-cleared it) and no saveHealth retry (nothing failed).

### [46] LOW — false-success — `src/lib/components/NotePane.svelte:340`

doSave() clears the view-level dirty flag BEFORE the host accepts the save; when NoteEditor.handleSave then refuses (the `saving` in-flight guard at NoteEditor.svelte:249, or the isCascading/isReseeding gate at :251), NotePane believes it is clean and the 1.5s-debounce/30s-idle/visibility autosave loop permanently stops re-attempting — the edits live only in the in-memory model until some departure event.

### [47] LOW — resource-leak — `src/lib/components/SecondScreenPage.svelte:308`

SS peek NoteModels are never disposed: loadPeekPreview creates a tab id `peek-${note.path}` per peeked note; NoteEditor's ensureModel $effect (NoteEditor.svelte:167) creates a full NoteModel (props + CM6 Text rope of the whole body) for it, but SecondScreenPage contains no closeNoteSession/closeModel call anywhere (closePeek at :332 only nulls the $state). The main window's Index preview closes its sibling model via a disposal $effect (+layout.svelte:6947) — the SS peek is the sibling gap.

### [48] LOW — index-divergence — `src-tauri/src/search.rs:1538`

sky_affected_paths keys the source-note recompute on target-NAME-set / name / alias changes only, so re-typing a link in place (same target, different type — [[X]] → [[supports::X]] or → [[generalizes::X]]) never recomputes the SOURCE's sky_nodes.stratum, whose +1 signals depend directly on link TYPE ('generalizes' edge present; 'causes'/'supports' edge present); the per-edge sky stratum triggers were removed in PJ-066 §B4, so nothing else fires. This is the source-side sky facet of the same root cause as registered W2-14 (which covers only the TARGET's incoming_* columns).

### [49] LOW — content-corruption — `src/lib/libraries/store.ts:1783`

Inline-list parsing splits on bare commas with no quote awareness, so a quoted item containing a comma (`tags: ["a, b", c]`) is projected as mangled fragments; any edit of that list then persists the mangled items to disk (one tag silently becomes two).

### [50] LOW — toctou — `src/lib/libraries/store.ts:1113`

Closed-note typed-link connect is an unlocked read-modify-write: readNote → composeUpdatedContent → writeNote, with markRecentWrite set BEFORE the write — an external write landing in the read→write gap is clobbered by our stale-based composition AND its watcher event is swallowed by the 2s recent-write suppression, so neither adoption nor a conflict surface fires.

### [51] LOW — concurrency-race — `src-tauri/src/universe.rs:901`

All universes.json mutations are unserialized load-modify-save round-trips: set_active_universe (async, Tokio worker) loads the registry at line 718 early in a potentially long switch (migrations, heals, consolidation) and saves its snapshot at line 901, while create_universe / open_existing_universe / remove_universe_from_registry (sync commands on the dispatch thread) do their own load→mutate→save concurrently. The switch_lock only serializes switches against other switches, not against the other registry writers.

### [52] LOW — silent-data-loss — `src-tauri/src/universe.rs:1603`

The one-time legacy workbench.json → collections.json adoption in read_universe_collections uses fs::write(&tmp) + fs::rename with NO fsync before the rename — the same unflushed-data-blocks power-loss window that atomic_write (same file, lines 121-131) exists to close. On success the legacy source is immediately renamed away to workbench.json.migrated, so the adopted copy is briefly the only live generation under its final name.

### [53] LOW — silent-data-loss — `src-tauri/src/classifier/mod.rs:347`

cece_resolve_disambiguation snapshots the prior suggestion with read_suggestions(conn, …).ok().flatten() — a genuine DB error (locked/corrupt) collapses to None, the exact error-collapse shape MIG-080 §D fixed in read_suggestions itself. With prior=None, other_axis_still_split is false for a both-axes-Split card, so the resolve takes the 'settled' path: sources_set_manual clears the suggestion row and no re-insert happens — the OTHER axis's pending disambiguation (candidate chips + composite trail) is silently discarded.

### [54] LOW — swallowed-write-error — `src/routes/+layout.svelte:6423`

addTagToNote's CLOSED-note branch surfaces a failed writeNote/reindex only via console.error (invisible in release, devtools disabled) — unlike the open-note branch, which rides the save-health banner via saveTabContent; a failed tag-add on a closed note is silent.

### [55] LOW — swallowed-write-error — `src/routes/+layout.svelte:4770`

The daily-note template application swallows ALL failures with `catch { /* template not found — OK */ }` — but the try-block includes the write_note at line 4768, so a failed WRITE of the templated daily note (not just a missing template) is silently ignored and the empty note opens as if templating succeeded.

### [56] LOW — swallowed-write-error — `src-tauri/src/search.rs:8788`

The post-rebuild version-file stamp itself is fire-and-forget (`let _ = std::fs::write(&version_path, ...)`) — if it fails, every subsequent boot silently deletes and rebuilds the entire index again.

### [57] LOW — resource-leak — `src-tauri/src/search.rs:8432`

spawn_wal_checkpoint_daemon spawns an immortal thread (infinite loop, no shutdown/generation check) once per successful init — every universe switch leaks another daemon that keeps re-opening and TRUNCATE-checkpointing its (now stale) universe's search.db every 300s forever.

### [60] LOW — freeze-hang — `src-tauri/src/libraries.rs:4705`

read_cooccurring_terms is a plain sync #[tauri::command] that, per Index-panel expand click, runs an FTS5 MATCH then fetches and fully re-tokenizes (stopwords + stemming via process_word_for_fts) up to 200 note bodies on the IPC dispatch thread (its sibling read_term_mentions at libraries.rs:4538 is the same sync class, cross-language expansion up to 500 rows). Wired live from +layout.svelte:7847 (loadCooccurrence/loadMentions on the Index panel).

### [61] LOW — resource-leak — `src-tauri/src/perf_trace.rs:47`

TRACE_LOG (perf_trace.rs:37) is a static append-only Vec<(String,u64)> pushed on EVERY IPC command dispatch by the invoke_handler wrapper (lib.rs:686) with no bound and no production clearing path: get_perf_trace_log (+layout.svelte:3759, boot:hydrated) clones without truncating, and clear_perf_trace_log has zero frontend callers (grep-verified).


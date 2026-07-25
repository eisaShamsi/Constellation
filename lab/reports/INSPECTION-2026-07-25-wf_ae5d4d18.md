# Safety Inspection register — 2026-07-25 (`wf_ae5d4d18-5fc`)

Per-build over the PJ-140 Rust remediation batch (libraries/search/write_gate/
review/universe). PJ-124 again → ran whole-app (33 agents).

## Verdict: the Rust batch introduced ZERO new silent-failures.

**9 confirmed — 1 HIGH, 7 MED, 1 LOW.** The two in changed Rust files
(`search.rs:1639` retype-aggregate = the deferred #48; `search.rs:9507`
reindex-ensure = known #6) are in functions the batch never touched — verified
against the exact `git diff` ranges (6216-6229, 8439-8443, 8609-8631, 9549-9556).

All 9 are pre-existing whole-app findings. Routing:
- **[0] BacklinksPanel linkMention (HIGH)** — open-model overwrite + false-success +
  no-reindex. → FRONTEND batch (was PJ-140 #39, upgraded HIGH).
- **[1] ExpressionForge / [2] SenseMakingCanvas — createNote-then-write-no-reindex** —
  NEW. The composition/canvas "promote to note" writes the real body after an
  empty-stub create but never reindexes → note invisible to search, no backlinks.
  → FRONTEND batch.
- **[3] NotePane editor-init fallback** — NEW; → frontend batch.
- **[4] NoteEditor cascade-freeze still accepts keystrokes** — editor-lifecycle
  cluster → its own migration.
- **[5] search.rs retype incoming-aggregate** — the deferred #48.
- **[6] bases.rs update_frontmatter_property block-scalar** — NEW (frontmatter class).
- **[7] search.rs reindex missing ensure_search_db_ready** — known #6.
- **[8] +layout addTagToNote closed-note swallowed** — PJ-140 #54, frontend batch.

## The register

### [0] HIGH — content-loss — `src/lib/components/BacklinksPanel.svelte:181`

The backlinks-panel 'link it' action (linkMention) turns a plain-text mention into a [[wikilink]] via a raw invoke('write_note') inside try{...}catch{/*ignore*/}, with no open-model check, no cascade gate, no reindex, and the write error fully swallowed.

**Scenario:** User has note B open in a tab (model live, possibly with unsaved body edits) and note B contains a plain-text mention of the active note A. In the backlinks panel the user clicks 'link it' on B's unlinked-mention row. linkMention reads B from disk, inserts [[A]], and writes B directly with invoke('write_note') — bypassing B's live in-memory model. B's model still holds the pre-wikilink body; on its next autosave/flush it composes from that stale body and durably overwrites disk, silently discarding the [[A]] the user just added (the exact hazard addTagToNote at +layout.svelte:6445 forbids and addLinkToNote at store.ts:1100 guards). Separately, if gate_write returns Err (conn None during init, WriteGate lock contention, FS error), the catch{/*ignore*/} discards it — the user believes the mention is now linked but disk is unchanged (false success). And on success no reindexNote is called (write_note does not reindex — libraries.rs:507-554), so the new note_links/backlink edge never appears until a boot reindex (index↔disk divergence). All three failure modes are fully silent.

---

### [1] MED — index-divergence — `src/lib/components/ExpressionForge.svelte:144`

Composition export writes the full note to disk after createNote but never reindexes, so note_meta/FTS/note_links keep the empty create-time content forever.

**Scenario:** User builds a composition in Expression Forge and clicks 'Export as Note'. createNote (store.ts:3522 → Rust create_note) writes a stub file (title/cid_cn/kind, EMPTY body) and synchronously indexes THAT into note_meta. The very next line, writeNote(newPath, content, 'expression_forge'), overwrites the file with the real composition (frontmatter `stage: maturity` + the full synthesized markdown body, which routinely contains [[wikilinks]]). No reindexNote / constellation_search_reindex is ever issued, and exportComposition then calls onClose?.() WITHOUT opening the note. write_note gates the path as watcher-suppressed, so the file watcher never fires library-changed to reindex it either. Result: note_meta.body_text stays empty, notes_fts has no composition text (invisible to full-text search), word_count=0 → wrong sky stratum/maturity, and note_links has none of the composition's wikilinks → every target silently loses this note as a backlink source. The .md on disk is correct; only the derived index diverges, with no error. reconcile.rs (dead-row/orphan only) never heals a same-path content divergence, so it persists until the user manually opens AND edits the note.

---

### [2] MED — index-divergence — `src/lib/components/SenseMakingCanvas.svelte:271`

Canvas 'promote to note' writes full content after createNote with no reindex; the follow-up openNoteTab does not reindex the body either.

**Scenario:** User promotes a canvas item to a note (confirmPromote). createNote makes a stub file and indexes it empty; writeNote(newPath, frontmatter + item.content, 'canvas_export') then overwrites the file with `stage: growth` + canvas_origin + the item body — with NO reindex. It does openNoteTab afterward, but opening a note never reindexes its body (openNoteTab only reads content + runs ensure_cid_cn_cmd, which writes the cid but does not reindex — canonical.rs:1264). So note_meta/notes_fts keep the empty create-time row: the promoted note's stage, canvas metadata, and body text are absent from search/sky, silently, with no error. Persists until the note is edited in-app (reconcile does not touch same-path content).

---

### [3] MED — silent-data-loss — `src/lib/components/NotePane.svelte:742`

The NotePane editor-init fallback (used when `new EditorView` throws — the documented RangeError on content with line-spanning replace decorations) builds `fallbackState` WITHOUT the `EditorView.updateListener` that the primary state carries (lines 633-666). In the fallback editor `dirty` is never set to true, `onDocChange` never pushes the doc to the model, and the 1500ms debounced save is never armed. `doSave()` early-returns on `!dirty`, and `doFlush()` (on tab switch / onDestroy / beforeunload) reports `needsDiskSave=false`, so handleFlush only stashes a write-ahead net and never writes disk.

**Scenario:** A note whose body triggers a livePreview decoration RangeError falls into the fallback editor. The user types edits into a fully-functional-looking editor; because no updateListener fires, `dirty` stays false, the model is never updated, and no debounced/idle/flush save ever runs. On tab switch or app close the edits are discarded with no error, no save-health banner, and no on-disk change — a completely silent loss of the entire editing session for that note.

---

### [4] MED — content-loss — `src/lib/components/NoteEditor.svelte:681`

During a rename cascade, a NotePane whose note is frozen still accepts keystrokes (its CM6 is never set read-only from cascadeFreeze, unlike FocusPane; the overlay blocks only pointer events, and onDocChange→editBody is not isCascading-gated). Those keystrokes mark the model dirty, then reloadTabsFromDisk force-adopts disk over the model and silently discards them.

**Scenario:** Split view: pane A shows the note being renamed, pane B shows a backlink to it, both in the same library, B's body editor has keyboard focus. User renames the note from the file tree. handleRenameComplete raises cascadeFreeze over both tabs (+layout.svelte:6723) and flushAllTabsInLibrary flushes B while it is still clean (so B is NOT in excludedPaths). The ~7s CascadeFreezeOverlay appears over B but blocks only pointer events; B's CodeMirror keeps DOM focus and stays editable (NotePane.svelte:599/1079 wire readOnly only from the static prop, never from cascadeFreeze/isCascading; FocusPane.svelte:75 does the hard gate but NotePane has no equivalent). The user types into B; onDocChange fires editBody(tab.id,doc,tab.path) gated only by !readOnly (NoteEditor.svelte:681), so the model goes dirty. The walker rewrites B's [[OldTitle]]→[[NewTitle]] on disk, B is in result.rewritten, and handleRenameComplete awaits reloadTabsFromDisk(reloadPaths) which force-adopts disk via openNoteModel with no dirty-guard (store.ts:840), overwriting the dirty model. B's keystrokes typed during the freeze window are lost with no error, no save-health entry, and no conflict sidecar.

---

### [5] MED — index-divergence — `src-tauri/src/search.rs:1639`

Re-typing an existing link (same target name) never recomputes the target note's incoming aggregates, because the save-path diff compares target NAMES only, not link types.

**Scenario:** Note A contains the wikilink [[B]] (associative). The user edits A and re-types the same link to a different type — e.g. [[contradicts::B]], or crossing the cognitive↔structural boundary to [[parent::B]]. On save, reindex_single_note(A) captures inc_old via incoming_signature(A), which returns only DISTINCT LOWER(target_name) = {"b"} (search.rs:1556-1562, no link_type captured). index_note rebuilds A's note_links (DELETE+INSERT); the outgoing triggers recompute A's OUTGOING aggregates only. maintain_incoming_after_save then computes new_targets = {"b"}; old_targets.symmetric_difference(new_targets) is EMPTY and A's name/aliases are unchanged, so `affected` is empty and B is never recomputed (search.rs:1639-1649). There is no note_links_incoming_* trigger (only DROP stubs at search.rs:1722-1724 — none is ever CREATEd), so nothing else recomputes B. Result: B's note_meta.incoming_link_types / incoming_link_types_json / incoming_top_rank stay STALE (still show 'supports (1)' when disk says 'contradicts'). In the associative→structural case it is worse: incoming_count excludes structural links (sx clause, search.rs:1514/1525), so B's incoming_count should DECREMENT — but it stays inflated, silently driving B's Reviewer maturity/state (compute_state(incoming_count)) and the inbound type breakdown too high. No error is surfaced. The divergence persists for the whole session and only self-heals on the next full reconcile_filesystem (search.rs:9414), i.e. typically next reboot.

---

### [6] MED — content-corruption — `src-tauri/src/bases.rs:539`

update_frontmatter_property only skips `- ` list continuations when replacing a key; a block-scalar or nested-map value's continuation lines are left orphaned under the rewritten scalar, producing invalid YAML.

**Scenario:** A note has a block-scalar or nested-map frontmatter property, e.g. `summary: |\n  Line one\n  Line two` (valid, common in Obsidian imports). In the Bases table the user edits that `prop.summary` cell (BaseTab.svelte commitEdit → updateNoteProperty → update_note_property → update_frontmatter_property). The function matches top-level `summary:`, writes `summary: newval`, sets skipping_list_items=true, but the next line `  Line one` is not a `- ` item so skipping flips off and the indented lines are preserved verbatim, yielding `summary: newval\n  Line one\n  Line two`. That is unparseable YAML. reindex_single_note then can't parse the block, so the note's OTHER properties silently drop from note_meta; and on the next NotePane save the frontend's composeFrontmatter hits parseDocument(...).errors and enters H1 verbatim-passthrough, so the corruption is frozen on disk with no error ever surfaced. Same class the frontend's immutableBlockKeys guards against — but this Rust path has no such guard.

---

### [7] MED — index-divergence — `src-tauri/src/search.rs:9507`

constellation_search_reindex (the note-save reindex IPC command) calls reindex_single_note without first calling ensure_search_db_ready; reindex_single_note returns Ok(()) when state.db is None (line 9621 `if let Some(conn)` with an implicit Ok(()) fall-through at 9691), so the reindex silently no-ops with a resolved Promise.

**Scenario:** User switches universe: invalidate_search_state (search.rs:8662) sets state.db = None and db_ready = false; the DB is re-opened only by the next ensure_search_db_ready. In that window, a dirty tab flushed during the switch (or an in-flight debounced save) writes the .md to disk and the frontend invokes constellation_search_reindex. The command has no ensure-first guard, so reindex_single_note hits `db.as_ref() == None`, skips index_note entirely, and returns Ok(()). The Promise resolves as success. The .md on disk is correct, but note_meta/notes_fts, backlink counts, sky maturity, and the Index panel keep the pre-edit content with no error anywhere. The boot reconcile only removes rows for vanished files / relocates by cid — it does NOT re-tokenize edited bodies — so the divergence persists silently until the user happens to re-save that exact note while db is ready, or performs a full rebuild.

---

### [8] LOW — swallowed-write-error — `src/routes/+layout.svelte:6470`

addTagToNote's closed-note branch awaits writeNote(...) inside a try/catch whose only handler is console.error, so a failed tag write is silent in a release build (devtools disabled).

**Scenario:** User right-clicks a CLOSED note in the file tree and chooses Add tag. The closed branch calls writeNote(path, composeUpdatedContent(...), 'add_tag'). If gate_write returns Err (conn None, lock contention, disk error), writeNote throws; the catch at 6470 only calls console.error, which is invisible in the release binary. The tag is not persisted to the .md frontmatter and the user receives no surfaced error — they believe the tag was applied. Lower severity than the linkMention case because the write is at least routed through the durable writeNote gate and there is little contradicting on-screen state for a closed note, but it is a genuine swallowed source-of-truth write.

---


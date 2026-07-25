# Safety Inspection register — 2026-07-25 (`wf_1b68be62-d7e`)

**Per-build inspection** over the four index/lifecycle files changed in the
2026-07-24 afternoon batch (`search.rs`, `libraries.rs`, `cece/wiring.rs`,
`classifier/scan_job.rs`). PJ-124 struck again — `args.files` ignored, ran
**whole-app** (14 scopes, 31 agents).

## Verdict: this build introduced ZERO new silent-failures.

**11 confirmed — 7 MED, 4 LOW. No APP-KILLER, no HIGH.** Every finding was checked
against the batch's exact `git diff` line ranges: **none sits at a line this batch
changed.** The five in touched files (`libraries.rs:248/1898`,
`search.rs:1639/9478/9584`) are all in *other* functions the batch never edited.

All 11 are **pre-existing whole-app findings** — the same backlog the 2026-07-24
whole-app sweep surfaced, awaiting the Boss's PJ-140 sequencing ruling. They are
folded into PJ-140 (roughly 7 already listed there; ~4 are net-new register
entries). Per WA#4, they are NOT fixed inside this feature/fix commit — drive-by
cross-cutting changes are exactly what that rule forbids, and PJ-140 already holds
the ruling request.

## The register

### [0] MED — silent-data-loss — `src-tauri/src/libraries.rs:1898`

move_item reindexes by delete-old + index-new, which resets each moved note's review_schedule (last_reviewed / interval / snoozed_until) because it never migrates the row the way rename_item_db_tail does.

**Scenario:** A note that has been reviewed (review feature stamped) has a review_schedule row: last_reviewed set, an earned spaced-repetition interval, possibly an active snooze. User moves it to another folder (or moves a folder containing it). move_item calls reindex_delete_note(old_path) -> delete_schedule_row deletes that row (search.rs:9524-9525), then reindex_single_note(new_path) -> index_note -> upsert_schedule_row(new_path) (review.rs:985). upsert reads existing history ONLY from a row at new_path, which does not exist, so it falls back to (last_reviewed=None, interval=0, snoozed_until=None) and writes a fresh default row. The note's entire review history and any active snooze are silently lost with no error; the note re-enters the due queue as if never reviewed. rename_item_db_tail (libraries.rs:1200-1211) explicitly UPDATEs review_schedule old->new precisely to prevent this; the move path omits the migration. review_schedule lives only in search.db and is not recomputable from the .md on disk.

---

### [1] MED — index-divergence — `src/lib/libraries/store.ts:1175`

flushAllTabsInLibrary flushes dirty tabs to disk via a bare standardSaveEnv (no onSaved), so the flushed notes' body edits land on disk but are NEVER reindexed into note_meta/notes_fts.

**Scenario:** User is editing note X (unsaved/dirty) in library L. User renames some other note Y in L. The rename pre-cascade calls flushAllTabsInLibrary(L) which flushes X's dirty model to disk with origin 'flush_all' and NO onSaved reindex hook (unlike navFlushEnv, which the code comments at 524-528 say was added precisely to avoid this index-stale divergence). The rename walker (update_links_on_rename) only rewrites+reindexes notes that CONTAIN [[Y]]; X does not, so it is never touched by the cascade. markRecentWrite(X.path) at line 1172 suppresses the file watcher for 2s, so no external-change reindex fires. The boot bulk reindex_library uses only_if_unindexed:true (libraries.rs:2972-2984) which skips the entire walk once L has any indexed row, and reconcile.rs only heals rows whose file is missing — not changed content of existing rows. Result: X's disk has the new body, note_meta.body_text + notes_fts hold the OLD body permanently. Search, Index panel, backlinks and counts silently reflect stale content with no error.

---

### [2] MED — fire-and-forget — `src/lib/libraries/store.ts:1266`

Every save path fires constellation_search_reindex fire-and-forget with .catch(()=>{}); a reindex that errors (SQLite BUSY/lock, poisoned mutex) or returns Ok-on-None-conn leaves note_meta/notes_fts stale under old content with no retry and no self-heal.

**Scenario:** saveTabContent (1266), navFlushEnv (535), retrySaveFailure (443) and handleSave all reindex via invoke('constellation_search_reindex',...).catch(()=>{}). writeNote persists the .md durably, then reindex is dispatched fire-and-forget. If reindex_single_note returns Err — e.g. state.db.lock() poisoned, or index_note's BEGIN IMMEDIATE hits SQLITE_BUSY under WAL contention from a concurrent writer/embed job — the .catch(()=>{}) swallows it. note_meta.body_text and notes_fts stay at the prior content. There is no retry (the durability gate only re-drives the DISK write, never the reindex) and no self-heal: boot reindex_library is gated off by only_if_unindexed once the library is indexed, the watcher is suppressed for the app's own write (markRecentWrite), and reconcile only removes/relocates rows whose file is gone. The note remains searchable under its stale body until it happens to be saved again successfully.

---

### [3] MED — false-success — `src/lib/components/NoteEditor.svelte:249`

handleSave's `if (saving) return` drops a debounced trailing save after NotePane's doSave has already cleared its `dirty` flag, so the newest edits reach neither the write-ahead net nor disk and no in-app timer retries them.

**Scenario:** A save's disk write is slow (>1.5s: lock contention, a large note, or PJ-103 chaining behind an app-close/flush-all save), so `saving=true` in NoteEditor. The user types a few more chars (onDocChange → editBody bumps the model to version N+1, NotePane sets dirty=true and reschedules the 1500ms debounce), then stops. 1.5s later doSave() runs (NotePane.svelte:339-340): it sets `dirty=false`, snapshots text, and calls onsave→handleSave, which hits `if (saving) return` and does NOTHING. Version N+1 now lives ONLY in the in-memory model — it was never composed here, so setNet never ran for it (the net still holds the older version N), it's not on disk, and NotePane's `dirty` is now false so neither the debounce nor the 30s idle-save (both gated on `dirty`, NotePane.svelte:339 & 962) will ever retry. The in-flight save resolves markSaved(N)/saving=false with nothing re-triggering. If the app is then hard-killed (crash/power-loss/OS-kill) before the user types again or navigates away, reopening restores the net = version N and the trailing keystrokes are silently gone — no error, no save-health banner. The guard is also redundant: PJ-103's per-id save chaining already serializes concurrent saves newest-last, so rescheduling (or removing the guard) would be strictly safer than dropping the save and clearing dirty.

---

### [4] MED — index-divergence — `src-tauri/src/search.rs:1639`

Retyping an existing link (same target name, new link_type) never recomputes the TARGET note's incoming link-type aggregates, because the save-path incoming diff keys only on target NAME, not link_type.

**Scenario:** Note A contains a wikilink [[supports::X]]. The user edits A's body to change it to [[contradicts::X]] (a retype — identical target name 'X', different type) and saves. reindex_single_note(A) -> index_note rebuilds A's note_links: the edge key is (target_name, link_type) (search.rs:6316), so the old ('x','supports') row is DELETEd and a new ('x','contradicts') row INSERTed. The outgoing triggers (note_links_outgoing_ad/ai) correctly recompute A's OUTGOING aggregate, and note_links_sky_au keeps sky_links correct. But X's INCOMING aggregates have no triggers (note_links_incoming_* are only ever dropped, never created — search.rs:1721) — they are maintained solely by maintain_incoming_after_save(A). That function derives old_targets/new_targets from incoming_signature (search.rs:1557), which SELECTs only 'DISTINCT LOWER(target_name)' and ignores link_type. So old_targets == new_targets == {'x'}, A's own name/aliases are unchanged, `affected` is empty (search.rs:1647), and X is never recomputed. X.note_meta.incoming_link_types_json stays {"supports":1} and incoming_top_rank stays the supports rank, permanently disagreeing with the .md on disk. Consequence: the [contested] chip — which collections.rs:43-45 derives client-side by reading `contradicts` out of incoming_link_types_json — silently fails to appear on X even though X now has a contradicting inbound link (a core Living-Link tension signal), and any Base/Collection sorted by inbound-type or incoming_top_rank shows X wrong. No error is surfaced to the user or any test. It heals only if the user happens to run Settings -> Rebuild Index (reconcile_filesystem -> recompute_all_incoming, search.rs:9385); there is no auto-reconcile on boot (the walk-free cache_boot_snapshot is used), so the divergence persists indefinitely.

---

### [5] MED — silent-data-loss — `src-tauri/src/libraries.rs:248`

save_libraries does temp-write + rename but does NOT fsync the temp file before the rename — unlike universe.rs::atomic_write, which was explicitly hardened (MIG-100 / G6, universe.rs:121-131) to sync_all() before rename precisely because 'power loss can land the rename while the data blocks are still unflushed, leaving a zero-length/garbage file under the FINAL name.' libraries.json is the registry of every library; the inline comment at libraries.rs:127 falsely asserts 'With the atomic write above, crash-corruption can no longer occur.'

**Scenario:** User adds/renames/removes a library → save_libraries writes libraries.json.tmp and renames it over libraries.json. Power loss or hard crash occurs after the rename metadata commits but before the temp file's data blocks are flushed to disk. On next boot the file exists under its final name but is zero-length or contains garbage. resolve_libraries_recursive (companion finding) then silently reads it as an empty library list — every library registration vanishes with no error and no backup. This is the exact durability failure atomic_write's fsync exists to prevent, left un-applied to the most-frequently-written and most-critical persisted manifest.

---

### [6] MED — index-divergence — `src-tauri/src/search.rs:9478`

The note-save reindex IPC `constellation_search_reindex` does NOT call ensure_search_db_ready and delegates to reindex_single_note, which silently returns Ok(()) when state.db is None (search.rs:9583-9584 `if let Some(conn) = db.as_ref()` with a bare Ok() fallthrough). Every other search.db write command in this subsystem calls ensure_search_db_ready + `.ok_or("Search database not initialized")?`; this one relies solely on the silent None-skip.

**Scenario:** A universe switch calls invalidate_search_state (search.rs:8646-8654) which sets db_ready=false and state.db=None until the next ensure_search_db_ready re-initializes the new universe's connection. In that window (also reachable at very-early cold boot before constellation_search_init runs), a debounced content-edit save of an open note fires constellation_search_reindex. reindex_single_note locks db, finds None, and returns Ok(()) — the frontend Promise resolves as success. The .md on disk is written by the separate save path (note content is safe), but the FTS/note_meta index never records the edit. reconcile (reconcile.rs:180-257) only relocates renames, removes gone files, and re-adopts orphan files — it does NOT re-tokenize a note whose row still exists but whose body changed on disk. So the edited text is silently absent from search indefinitely, until that specific note happens to be saved again. Silent false-success + index<->disk content divergence with no surfaced error.

---

### [7] LOW — false-success — `src-tauri/src/search.rs:9584`

reindex_single_note returns Ok(()) when db.as_ref() is None (search DB not yet initialized), silently skipping the reindex while reporting success to the awaited/fire-and-forget caller.

**Scenario:** reindex_single_note guards its body with `if let Some(conn) = db.as_ref()` and falls through to `Ok(())` when the connection is None (before ensure_search_db_ready sets the db, or after a schema-version rename-aside on a fresh universe). A save that fires during that init window (e.g. an auto-restored tab's early autosave, or a save immediately after switching universes before the new DB is live) writes the .md to disk durably but the reindex conditionally skips the work and returns Ok — a false success. note_meta/notes_fts never receive the note's content. As with the fire-and-forget path, the only-if-unindexed boot walk and reconcile do not re-index changed content of an existing row, so the note stays absent/stale in every derived surface until a later successful save re-triggers indexing.

---

### [8] LOW — swallowed-write-error — `src/routes/+layout.svelte:6761`

handleRenameComplete consumes result.rewritten but never reads result.failed — a backlink whose on-disk wikilink rewrite errored in gate_rmw is dropped with no toast, journal marker, or retry.

**Scenario:** User renames Note A (OldTitle -> NewTitle) with autoUpdateLinks on. Note B contains [[OldTitle]]. During the walk, gate_rmw on B's file returns Err (transient lock / permission / EBUSY), so the Rust walker records B in CascadeResult.failed and NOT in rewritten (libraries.rs:5856-5865). The frontend only uses result.rewritten (line 6761) and result.failed is never surfaced anywhere in the app (grep confirms no consumer). B's disk silently keeps [[OldTitle]] while every other backlink now says [[NewTitle]]. The rename's alias stamp (OldTitle->A) keeps the stale link resolving for now, so the failure is invisible — but it is a latent mis-resolution: if the user later renames A again, removes the OldTitle alias, or renames another note to OldTitle, B's un-updated [[OldTitle]] silently binds to the wrong note. No error is ever shown for the partially-failed cascade.

---

### [9] LOW — content-corruption — `src/lib/editor/yamlDoc.ts:366`

The scalar SET path uses CST.setScalarValue(item.value, np.value) with the raw string, which — unlike the ADD path's serializeLine/yamlStringify — does NOT quote YAML-reserved literals, so editing a text property to 'true'/'false'/'null'/'yes'/'no' or a numeric string writes it UNQUOTED (e.g. `key: true`), which a strict YAML consumer reads back as a boolean/null/number rather than the string the user typed.

**Scenario:** User has a text frontmatter property, e.g. `label: draft`, and edits its value to the literal string `null` (or `true`, `yes`, `123`). The value is unchanged-type text, the CST item is a scalar and np.type !== 'list', so line 365-366 takes the setScalarValue branch. Verified live: setScalarValue('label:draft','null') emits `label: null` UNQUOTED (whereas the first-time ADD via serializeLine/yamlStringify would emit `label: "null"`). The .md on disk now holds a YAML null/boolean/number where the user meant the string. No error, and Constellation's own line-based store.ts parseFrontmatter happens to read the literal text back so the app never notices — but any strict YAML reader (Obsidian, the Rust frontmatter index reader, sync tooling, a future parseDocument-based reader) silently reads the wrong type, and a subsequent Constellation edit-cycle that re-parses through parseDocument would drop the string. Same value → two different disk encodings depending on whether it was added vs edited (round-trip asymmetry).

---

### [10] LOW — false-success — `src/lib/components/BacklinksPanel.svelte:181`

The 'Link this mention' action awaits write_note inside a try/catch whose catch is a bare `{ /* ignore */ }` — a failed write of the mentioning note's .md (file lock, transient IO, gate refusal) is fully swallowed with no console log, no save-health entry, and no user-visible error. The action visually completes as if the plain-text mention was converted to a [[wikilink]], but the source note on disk was never rewritten.

**Scenario:** User opens note B which mentions note A's title in plain text. In the Backlinks panel they click the 'link this mention' control for B. readNote(B) succeeds, the regex produces newContent with [[A]], and invoke('write_note', {filePath:B,...}) is dispatched — but the write fails (e.g. B is momentarily locked by a sync tool/antivirus, or a write-gate contention). The rejection is caught by `catch { /* ignore */ }`: nothing is logged (release builds have no console anyway), no banner is raised, and the panel just re-renders. The user believes the mention is now a real typed backlink; on next boot/reindex B still contains only the plain-text mention, so the backlink silently never existed. Lower severity than a data-loss because B's existing content is not corrupted or lost — the enrichment simply, silently, did not happen (false-success). Unlike the note save path (durable saveNoteSession gate) and the persist paths (which at least console.error per the G6 ruling), this write has zero surfacing of any kind.

---


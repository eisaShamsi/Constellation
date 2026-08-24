# Safety & Integrity Sweep — WHOLE-APP register, 2026-08-24

**Provenance, stated plainly.** Launched during PJ-369 Step 2 as the per-build *diff-scoped*
inspection. I passed `args.files` as a **string** instead of an object, so the workflow fell
back to the whole-app cycle sweep — **the identical mistake made on 2026-08-23**, one day
earlier, by me, after it had already been written down in that day's register. The sweep's
findings are real and owned; the repeat is a process failure worth its own line, because a
guard that lives only in a report I wrote yesterday is not a guard.

73 agents, 14 hunt scopes, every candidate adversarially refuted before confirmation.
**59 confirmed findings**: 1 APP-KILLER, 8 HIGH, ~26 MEDIUM, ~24 LOW.

Scopes scanned: rename-move-delete-gate, note-save-index, notemodel-ownership,
editor-lifecycle, rename-cascade-integrity, cross-window-integrity, derived-index-triggers,
frontmatter-property-writes, persisted-json-state, cece-sources-derived,
frontend-write-callers, boot-init-ordering, reactivity-concurrency, freeze-and-leaks.

---

## Disposition

**FIXED IN THIS PASS (before any commit) — the APP-KILLER.**

`src/lib/libraries/store.ts:5291` — `preserveWorkBeforeVacating` decided whether it was safe to
destroy a path's recovery state by asking `isNoteDirty` **alone**, ignoring `hasUnsavedRecovery`.
A model can be *clean* and still hold the only copy of the user's work: after a failed save the
edit lives only in the write-ahead net, and the next boot's `restoreSessionTabs` seeds a model
from that net and truth-sets its baseline, leaving `netUnsaved = true` on a model that is clean
by construction. Delete, an **ancestor-folder** delete, or an Overwrite in a collision dialog
then reported "already durable" and the caller wiped the net, its localStorage backup, the
banner and the model. The trashed file was the pre-edit version; the paragraph existed nowhere;
no error surfaced; two of the three triggers never touch the note directly.

*Verified before fixing, not assumed:* `hasUnsavedRecovery`'s own doc-comment states that every
site arbitrating "is there work here disk does not have" must ask it, and its four siblings do
(`store.ts:1169`, `:1374`, `:4993`, `NoteEditor.svelte:444`). This one — whose entire job is
preserving work before its file is displaced — did not.

*The obvious fix was wrong and was rejected.* Adding `|| hasUnsavedRecovery(t.id)` to the flush
list looks right: it is not. `flushIfDirty` returns `{ok:true}` **without writing** when the
model is clean (`noteSession.ts:392`), so the function would still have reported durability
having written nothing — the bug surviving behind a correct-looking guard, which is exactly the
"half a sweep inside the fix for half a sweep" this function's own comments already record twice.
Nor can the content be made durable there: the path itself is being vacated. The fix therefore
keeps the net and returns `false`, which both callers already honour — they complete the delete
but skip the aux-state wipe.

*Reproduced before shipping (Reproduce-First).* `tests/pj-377/vacateKeepsRecoveryNet.test.ts`
pins all three triggers plus a control. Against the pre-fix logic all three go **red** with
`expected undefined to be 'the paragraph that never reached disk'` — the net wiped — while the
control stays green. A test seeded with a merely *dirty* model would have passed against the
naive fix; this one seeds the clean-but-recovering model on purpose. Ledger: **PJ-377**.

**NOT FIXED IN THIS PASS — the remaining 58.** They are a full cycle's remediation and are filed
as **PJ-378** for triage and ranking. They are NOT "logged and shipped": no ruling has been
requested on them yet, because Standing Order #10 requires the PCS and orientation to be current
first. Several cluster into families that should be fixed as families rather than one-by-one
(the Whole-Ecosystem Fix Law): the missing-`ensure_search_db_ready` group (`bases.rs:437`,
`tasks.rs:540`, `shape.rs:214`, `universe.rs:2643`, `libraries.rs:1598`), the swallowed-write-error
group, and the YAML quote/escape group (`store.ts:2846`, `libraries.rs:3131`, `:2849`, `:3002`,
`ExpressionForge.svelte:139`).

---

## Confirmed register (severity-ranked)

```
[APP-KILLER] src/lib/libraries/store.ts:5291  (silent-data-loss)
    `preserveWorkBeforeVacating` decides whether it is safe to destroy a vacated path's recovery state using `isNoteDirty(t.id)` ALONE, ignoring `hasUnsavedRecovery(t.id)` — so a model holding w...
[HIGH      ] src-tauri/src/libraries.rs:1598  (init-ordering)
    `create_note` is the one note-writing command with no `ensure_search_db_ready` guard: its MIG-099 §3 synchronous reindex fails silently while `state.db` is None, and the index-authoritative ...
[HIGH      ] src/lib/components/PropertyEditor.svelte:457  (silent-data-loss)
    The right-sidebar PropertyEditor is mounted without a {#key} (+layout.svelte:9848), so a tab switch changes only its tabId/filePath props. The seed $effect re-seeds unconditionally on `|| ta...
[HIGH      ] src-tauri/src/libraries.rs:3131  (content-corruption)
    `remove_frontmatter_contains_item` re-emits an INLINE `contains:` flow array as `format!("contains: [{}]", kept.join(", "))` from items that `split_flow_seq_items` has already UNQUOTED (yaml...
[HIGH      ] src-tauri/src/libraries.rs:1756  (content-loss)
    migrate_note_db_paths claims note_links/note_aliases "cannot collide and need no pre-delete" (line 1751) — both carry a path-bearing UNIQUE/PRIMARY KEY, so both UPDATEs abort whole-statement...
[HIGH      ] src/lib/libraries/store.ts:3284  (content-corruption)
    openNoteTab's one-path-one-tab dedup is a synchronous snapshot taken BEFORE two awaited IPCs (`resolveNoteContent`, `ensure_cid_cn_cmd`); with no tab currently active, two opens of the same ...
[HIGH      ] src/lib/components/PropertyEditor.svelte:457  (silent-data-loss)
    The standalone right-sidebar Properties panel is mounted without a {#key}, so a tab switch re-seeds `editableProps` unconditionally (`|| tabChanged` overrides the pending-edit guard) and sil...
[HIGH      ] src-tauri/src/search.rs:12669  (index-divergence)
    constellation_search_reindex — the save-path reindex IPC that ~20 frontend callers fire on every durable write — calls reindex_single_note with expected_generation = None, so the universe pa...
[HIGH      ] src/lib/components/GraphMindView.svelte:865  (resource-leak)
    The deferred PIXI teardown (`setTimeout(() => capturedEngine?.destroy(), 0)`) races the component's own `await engine.init()`: on any unmount during PIXI initialisation the destroy runs FIRS...
[LOW       ] src/lib/components/CatalogerView.svelte:185  (resource-leak)
    Two Tauri listeners leak if the Cataloger is unmounted before onMount's awaits resolve — there is no `destroyed` guard, unlike the sibling SourceReviewPanel.
[LOW       ] src-tauri/src/classifier/scan_job.rs:261  (freeze-hang)
    enumerate_pending takes the SQLite WRITER lock for a pure full-table read of note_meta — the same half-sweep that cece/wiring.rs was fixed for and this sibling was not.
[LOW       ] src/lib/libraries/propertyTypeRegistry.ts:126  (swallowed-write-error)
    A failed property-types persist is reported only through console.error, with no visible error state — unlike its three siblings (settingsError store.ts:7534, workspacesError store.ts:7799, c...
[LOW       ] src-tauri/src/libraries.rs:2144  (swallowed-write-error)
    All three detached path-migration tails swallow a poisoned or None SearchState.db guard with a bare `if let Ok(guard) = ... .lock()` and no else-arm, so the 11-table path cascade and the non...
[LOW       ] src-tauri/src/libraries.rs:9870  (silent-data-loss)
    move_into_trash_folder's copy+remove fallback takes the lock on the FOLDER path only, so for a directory delete a debounced editor save on a descendant .md is not serialized against the copy...
[LOW       ] src/lib/components/NoteEditor.svelte:622  (index-divergence)
    `record_shape_change` is fired unawaited immediately after a NON-awaited `saveNoteSession`, so the shape-history row is written whether or not the disk write lands, and its own failure is co...
[LOW       ] src/routes/+layout.svelte:4900  (content-loss)
    A failed `write_note` for the template body of a newly created note is swallowed by a bare `catch { }`, so the note is created and opened without its template and the create action reports s...
[LOW       ] src/lib/libraries/store.ts:2846  (content-corruption)
    `quoteIfNeeded` emits a DOUBLE-quoted YAML scalar escaping only `"` (`v.replace(/"/g, '\\"')`) and never the backslash, whereas its Rust twin `bases.rs::format_yaml_value:863` escapes both. ...
[LOW       ] src-tauri/src/libraries.rs:1753  (index-divergence)
    The destination pre-delete `DELETE FROM note_meta WHERE path = new_path` drops a live row without applying the tag_counts −delta, so the discarded row's tags stay counted in the tag browser ...
[LOW       ] src-tauri/src/search.rs:8635  (index-divergence)
    review_schedule.stratum is stamped only from the saved note's own sky_nodes row at index time, so a link change in ANOTHER note that moves this note's stratum leaves the Reviewer's copy stal...
[LOW       ] src/lib/libraries/store.ts:3336  (cross-window-clobber)
    In a display-only window resolveNoteContent preserves the net but still returns recoveredFromNet:true, so restashConsumedNet re-writes the SHARED localStorage crash-net with the older bytes ...
[LOW       ] src/routes/+layout.svelte:7183  (index-divergence)
    addTagToNote's CLOSED-note branch writes the tag through the gate and then makes the reindex conditional on `if (lib)` — a null libraryForPath silently skips it, where the sibling addLinkToN...
[LOW       ] src/lib/components/NotePane.svelte:1015  (content-loss)
    The idle-save interval schedules doSave through requestIdleCallback; onDestroy clears the interval but never cancels an already-queued idle callback, so a callback firing after teardown push...
[LOW       ] src/routes/+layout.svelte:10168  (resource-leak)
    The workspace-save second-screen state probe registers `onStateResponse(...)` and only stores its unlisten function in a `.then()`; if the second screen's reply lands before that microtask r...
[LOW       ] src/lib/components/CatalogerView.svelte:196  (resource-leak)
    `onMount` is async and registers the `classifier:scan` and `nsc:backfill` listeners AFTER two awaited `invoke()` round-trips, with no destroyed-flag guard — the exact W3-3 class fixed in Sou...
[LOW       ] src/lib/nsc/summaryStore.ts:100  (toctou)
    A `library-changed` invalidation that arrives while a batch is in flight is a no-op (the path is not in the cache yet), and the batch then writes the pre-save summary into the long-lived cac...
[LOW       ] src/lib/components/CollectionsPanel.svelte:64  (concurrency-race)
    The collection re-hydration effect writes `hydratedRows` from a resolved promise with no stale-result guard, so an out-of-order resolve can leave the newly-selected collection's members perm...
[MED       ] src-tauri/src/sources/bulk_ops.rs:286  (false-success)
    Approve-All counts a note whose frontmatter write FAILED as completed, and the terminal "done" event reports error: None because only mirror failures feed the summary.
[MED       ] src-tauri/src/sources/bulk_ops.rs:157  (concurrency-race)
    run_bulk_accept has no federation_generation guard — a universe switch mid-batch silently turns every remaining note into a refused no-op that is still counted as completed.
[MED       ] src-tauri/src/sources/mod.rs:838  (index-divergence)
    sources_set_manual releases the DB lock across the disk RMW, then re-acquires it without a federation_generation or db_ready re-check — a universe switch inside that window mirrors into the ...
[MED       ] src-tauri/src/classifier/mod.rs:418  (false-success)
    cece_resolve_disambiguation's still-Split return path uses `.ok().flatten()` on the read-back, so a DB read failure returns Ok(None) — which the panel reads as "card consumed" and drops, orp...
[MED       ] src-tauri/src/file_kinds.rs:187  (swallowed-write-error)
    KindRegistry::save() persists the per-universe file_kinds.json with a bare truncate-then-write fs::write whose error is discarded (`let _ =`), and its loader (line 98) reads the same file wi...
[MED       ] src/lib/libraries/store.ts:2198  (fire-and-forget)
    The collections write is an unawaited promise chain (`saveCollections()` is never awaited by any caller — toggleStarred/addToCollection/createCollection/renameCollection/deleteCollection/ado...
[MED       ] src/lib/libraries/store.ts:7807  (fire-and-forget)
    persistWorkspaces() issues an unawaited invoke('save_universe_workspaces') and has no close-time flush — the same close-handshake gap as collections, on the second persisted-JSON store the 2...
[MED       ] src-tauri/src/libraries.rs:3190  (cross-note-bleed)
    resolve_structural_conflict authorises its frontmatter write through the FEDERATED resolver (validate_path_in_any_library) and never calls require_own_library, so it rewrites a Linked Univer...
[MED       ] src-tauri/src/libraries.rs:3396  (index-divergence)
    move_item_db_tail has no third-universe anti-adoption fence, while its sibling rename_folder_db_tail builds one strictly and skips its whole reindex loop when it cannot be built.
[MED       ] src/lib/libraries/propertyTypeRegistry.ts:111  (false-success)
    When the boot read of property-types.json failed, every property-type assignment the user makes for the rest of the session is discarded with no surface at all — the refusal is a console.err...
[MED       ] src/lib/libraries/propertyTypeRegistry.ts:126  (swallowed-write-error)
    The debounced property-types write swallows a genuine IPC/disk failure into console.error only — no error store exists for this file, so a failing `save_universe_property_types` silently dro...
[MED       ] src-tauri/src/bases.rs:437  (index-divergence)
    `update_note_property` (Base cell edit) has no `ensure_search_db_ready`; during the None-conn window the gated frontmatter write lands on disk while the reindex fails to a diagnostics line o...
[MED       ] src-tauri/src/tasks.rs:540  (index-divergence)
    `toggle_task` has no `ensure_search_db_ready`; a checkbox toggle in the None-conn window writes to disk through the gate while its reindex fails to a diagnostics line only.
[MED       ] src-tauri/src/shape.rs:214  (index-divergence)
    `set_note_shape` / `clear_note_shape` / `revert_note_shape` reach `apply_shape` with no `ensure_search_db_ready`; the shape lands in frontmatter on disk while the reindex silently fails in t...
[MED       ] src-tauri/src/universe.rs:2643  (index-divergence)
    `reindex_written_template` (create_template / adopt-kind) has no `ensure_search_db_ready`; a template written in the None-conn window is never indexed and its own docstring names the consequ...
[MED       ] src-tauri/src/shape.rs:239  (swallowed-write-error)
    `record_shape_change` returns Ok(()) after `record_change` silently returns on a None conn and swallows both of its writes with `let _ =`; the lost row makes a later undo restore a STALE sha...
[MED       ] src-tauri/src/search.rs:11973  (false-success)
    `ensure_search_db_ready` returns Ok(()) with `state.db` still None on the generation-mismatch discard path, so every "ensure-or-refuse before the fs op" caller proceeds to write with its DB ...
[MED       ] src-tauri/src/index_repair.rs:853  (false-success)
    The boot cold-start repair walk discards every per-note reindex error with `let _ =` and returns a completion with `walk: None`, so the run reports `ok: true` and no failure count even when ...
[MED       ] src/lib/libraries/store.ts:1658  (toctou)
    `linkMentionInNote`'s OPEN-note branch has no rename-cascade gate: the closed-note branch is guarded by `if (!openTab && isCascading(mentionPath)) return false;`, but when the note IS open t...
[MED       ] src/lib/libraries/store.ts:3307  (content-corruption)
    `openNoteTab` reads the note's bytes at :3307 (`resolveNoteContent`) and installs them into the model at :3496 / :3521 (`openNoteModel`) with no freshness re-check and no `isCascading` consu...
[MED       ] src/lib/components/ExpressionForge.svelte:139  (content-corruption)
    The exported composition's frontmatter is hand-built by raw interpolation into a DOUBLE-QUOTED YAML scalar — `title: "${compositionTitle.trim()}"` — with no escaping of `"` or `\`. The FILEN...
[MED       ] src-tauri/src/libraries.rs:2849  (content-corruption)
    The Rust rename path never got the PJ-207 §15 escape-DECODE fix its TS twin received. `extract_frontmatter_title` strips quotes with `trim_matches('"')` and KEEPS the escape syntax (the TS `...
[MED       ] src-tauri/src/search.rs:7418  (index-divergence)
    `extract_aliases` — the writer of `note_aliases`, the table alias-aware wikilink resolution JOINs on — matches `trimmed.starts_with("aliases:")` after `line.trim_start()`, so INDENTATION IS ...
[MED       ] src-tauri/src/libraries.rs:3002  (content-corruption)
    The three rename/structural frontmatter writers rebuild the block with `fm.lines()` + `new_lines.join("\n")` and hardcoded `---\n` fences (libraries.rs:3002 update_frontmatter_title, :3076 s...
[MED       ] src/routes/+layout.svelte:4900  (swallowed-write-error)
    The template-merge create path awaits `invoke('write_note', …)` at line 4894 to write the merged frontmatter + rendered template body over the freshly-created stub, inside a `try` whose `cat...
[MED       ] src-tauri/src/link_life_restore.rs:398  (index-divergence)
    The boot-time earned-layer restore flips note_links.status (active→archived) with no incoming/sky recompute, so every target note keeps a backlink count that includes the re-retired link — a...
[MED       ] src-tauri/src/search.rs:8584  (false-success)
    The cid self-heal declares "cid self-heal OK … dead row gone, note indexed" on the strength of a note_meta count alone, so a relocation whose note_links / note_aliases / review_schedule stat...
[MED       ] src/lib/components/PropertyEditor.svelte:628  (silent-data-loss)
    The NotePane-embedded PropertyEditor's teardown clears its 800 ms debounce and then returns without committing when the note is mid-reseed or mid-cascade — and because a panel edit only reac...
[MED       ] src/lib/libraries/store.ts:1234  (cross-window-clobber)
    followExternalRename (the second screen's only reaction to a rename) repaths the tab and the model but never re-reads disk, so the SS keeps serving PRE-rename frontmatter forever — and the m...
[MED       ] src/lib/libraries/store.ts:1432  (cross-window-clobber)
    Four durable note-write paths (task toggle, typed-link connect, link-mention, closed-note add-tag) write the .md through the watcher-suppressed gate and never emit screen:note-saved — the se...
[MED       ] src/lib/components/SecondScreenPage.svelte:615  (cross-window-clobber)
    adoptFreshDiskIntoSS matches tab paths with raw string equality while its documented main-window twin normalizes separators — a separator mismatch silently disables the SS's entire external-...
[MED       ] src/lib/components/NotePane.svelte:1015  (concurrency-race)
    The 30 s idle save is queued via a bare `requestIdleCallback` (no timeout) that is never cancelled in onDestroy, and `doSave()` has no destroyed-guard — a torn-down pane can therefore push i...
[MED       ] src/routes/+layout.svelte:3775  (index-divergence)
    The watcher flush's terminal give-up after REINDEX_MAX_RETRIES drops the whole batch of externally-changed paths with only a console.error — no indexHealthError, no banner — while its choke-...
```

---

## PJ-377 — the obvious objection to the fix, checked

*"If a deleted note keeps its recovery net forever, can that net later resurrect deleted
content, or leak without bound?"*

**No.** The restore path resolves every saved tab with `requireDisk: true`, under a comment that
states the rule directly: *"a wab entry alone must not resurrect a tab whose file is gone (the
ghost-tab failure) — restore restores FILES, the net stays for real recovery"*
(`store.ts:3788`). A net kept for a vacated path is therefore **inert**: it cannot reopen a tab,
and it pays off only if the file comes back — which, for a delete, it can, since the file went to
`.trash` and a delete is advertised as recoverable. That is precisely the case worth protecting:
restore the note from trash and the unsaved paragraph is still there.

The residual cost is one map/localStorage entry per note deleted while holding unsaved recovered
work — rare, small, and bounded. Weighed against silently destroying the only copy of a user's
paragraph, it is not a close call.

---

## CORRECTION — PJ-377 was NOT fixed when this register first said it was

The adversarial panel reviewed the fix and found it incomplete. Two lenses wrote their own probes
and reproduced doors the fix left open. Everything below replaces the account above it.

**1. The fix asked about the TAB; the thing protected is keyed to the PATH.**
`preserveWorkBeforeVacating` filtered `get(openTabs)` — but the recovery net **outlives the tab**.
`closeTab` clears neither the net nor the save-failure banner, and says so in its own comment:
they "preserve a failed write and restore it on reopen." So the commonest real sequence — save
fails, user closes the tab, user deletes the note or its folder later — walked past every guard,
because by then there was no tab to ask about. **Verified by execution:** the new
`Deleting a note with NO TAB OPEN keeps its net` case goes red against both the original code and
my first fix.

The predicate now reads the net directly (`netPathsToPreserve`): an entry still present that is
not a `snapshot` view-stash means "work disk does not have", regardless of tabs, models or
sessions. `setNet` stashes before the write and `clearNetIf` clears only on durable success, so
the net — not the tab — is the record. This ends a sequence of three successively-narrower
questions (exact-path vs at-or-under; `isDirty` vs `netUnsaved`; open-tab vs any-path) rather
than adding a fourth.

**2. Protecting one note leaked every sibling.** Returning a bare `false` skipped
`clearPathKeyedAuxStateOnDelete` for the ENTIRE deleted folder, so one unrecoverable note left
every sibling's aux state behind in a map and a localStorage blob that only grow — and whose
quota overflow is swallowed by an empty catch. `AuxStateAction` gained a `keep` variant; the
cleanup is now per-key. Pinned by the `SIBLINGS are still cleaned up` case.

**3. The exposure was wider than recorded.** `netUnsaved` is set at **two** sites, not one:
`noteModel.ts:535` (via the restore path, as recorded) **and** `noteSession.ts:333`, on the PJ-287
superseded-write branch — **in a live session, no restart involved**. The original account
described the bug as reachable only after a failed save plus a restart. It was reachable while
simply working. That second site also calls `setNet`, so the net-based predicate covers it; the
tab-based one would not have.

**4. A test named a trigger it did not run.** The third case was titled "Overwrite-on-collision …
the third trigger" and called `moveToTrash` alone. A green test asserting coverage it does not
have is worse than no test, because it is the evidence offered for the claim. Retitled to name
the primitive it actually exercises, with the gap stated in the test body.

**5. Overwrite-on-collision is NOT fixed, and cannot be fixed by this predicate.** The collision
dialog runs `moveToTrash` then `renameItem` on one click, so the vacated path is immediately
re-occupied by the incoming note. Path-keyed preservation has nowhere to live. The real remedies
— a `(recovered copy).md` sibling written before vacating, or an honest warning in the dialog —
are product decisions, not refactors. **This goes to the Boss as a question, not as a fix.**

**6. Drop the one-sided "kept nets cannot resurrect deleted notes."** That framing was defensive
and half-true. Restore-from-`.trash` plus reopening the note **is** the recovery route, and it
works — which is the point, since a delete is advertised as recoverable. What the app must not be
said to do is restore it automatically: `requireDisk: true` means a net alone never reopens a tab.

**Verification after the rework:** all six PJ-377 cases pass; five of the six go red against the
pre-fix code with the control staying green; the full frontend suite is 1008 passing across three
consecutive clean runs; `svelte-check` 0 errors; Rust 1559 passing.

---

## The DIFF-SCOPED inspection — run properly at last, and it earned its keep

With the argument guard in place, the per-build inspection finally ran as intended:
`mode: "diff"`, five files scanned. It returned **one confirmed finding — in code I had written
and "fixed" two hours earlier.**

**`phantom_prune.rs` — the Attack-1 guard refused only a TOTALLY unresolved federation.**

The field's own doc says: *"If ANY linked universe fails to resolve, this is not built and
`refused` is set."* The code tested `linked_roots.is_empty()`. Those are different claims, and
the gap is reachable because the guard's two inputs disagree about a missing child:
`resolve_child_universe_roots_recursive_strict` deliberately **keeps** a child whose folder is
`NotFound` (universe.rs:750), while `resolve_libraries_recursive` **silently skips** it
(universe.rs:641-644).

So: two Linked Universes declared, A present, B renamed in Explorer between sessions. `declared`
counts B; `linked_roots` does not contain it; the set is **non-empty**; the guard stays silent.
Every parent-index row pointing under B's old path then classifies as a phantom — and those notes
exist, they are merely somewhere else.

Why it matters beyond a wrong number: today the only consumer is a count, so the harm is a
user-facing claim that N entries "point at notes that no longer exist on disk" when they do.
**At Step 3/4 the identical verdict feeds `reindex_delete_note`** — deleting a Linked Universe's
index rows, which is both the Attack-1 scenario this guard exists for and a write-sovereignty
violation (MIG-111). Fixed in-pass per WA#6 rather than deferred to the step that would weaponise it.

**The fix** compares the resolved federation against the strict declared set: every declared child
must have contributed at least one resolved library root. A child universe always registers its own
root as a library (`universe_notes`), so "contributed nothing" means unreadable, never legitimately
empty. The decision was extracted as a pure `federation_is_complete(declared, linked)` — free of
`AppHandle`, following the `libraries::foreign_roots_of` precedent — precisely so a regression test
exercises the real function rather than a copy.

**Four new tests**, two of which go red against the old semantics:
`attack1c_one_missing_child_among_several_refuses_the_whole_run` (the reported scenario),
`attack1f_a_prefix_lookalike_does_not_count_as_resolved` (a sibling named `child b2` must not
satisfy `child b`), plus `attack1d` (all resolved → complete) and `attack1e` (no federation at all
→ complete, not refused — otherwise the feature would disable itself for the common case).

**The lesson worth keeping:** `attack1b` already existed and passed. It proved that a *refused*
context yields `Unknown` — it exercised the **consumption** of the refusal, never the **decision**
to refuse, which is where the bug was. A guard needs a test on the predicate that arms it, not only
on the behaviour it triggers.

Also corrected in the same pass: two doc drifts in this module's own header — condition 4 still
described "any `review_schedule` row" (the policy that made the ground-truth audit return
`Prune: 0`, fixed in the code this morning but not in the prose), and a provenance insertion had
orphaned a sentence from its subject.

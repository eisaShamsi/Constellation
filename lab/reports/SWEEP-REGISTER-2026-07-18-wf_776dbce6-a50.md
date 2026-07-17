# Whole-app safety sweep register — 2026-07-18 cycle (`wf_776dbce6-a50`, 82 agents, 62 confirmed)

**PJ-106 cycle-close per-cycle whole-app `safety-inspection` sweep.** Run 2026-07-17 (a day
ahead of the scheduled `pj106-cycle-close-sweep-jul18` 4 am fire — Boss directed running it now).
HEAD at run = `d148f7f8` (post PJ-103 close). Refute-first: every candidate was adversarially
refuted before confirmation. **62 confirmed** — **3 APP-KILLER · 11 HIGH · 38 MED · 10 LOW**, zero
agent errors. Ranked APP-KILLER first below.

This sweep is THREE things at once: (a) the PJ-106 migration's formal cycle boundary (Phase-4 C3),
(b) the §B4 post-gate promised when the Boss directed building §B4 ahead of this reset, and (c) the
check on the audit's one open edge — the §B4 gesture's toolbar-Ctrl+Shift-click disarm.

> **Process note.** This is the per-cycle register. Per the Charter + WA#6, the **confirmed
> APP-KILLERs are remediated Reproduce-First (each Boss-tested) before the PJ-106 cycle is declared
> closed**; the HIGH/MED/LOW confirmations are filed to the Pending-Jobs ledger and worked through
> the ranked Group-1 queue (many already carry PJ numbers — mapped per-finding under **Triage**).
> Nothing here is committed or fixed by the sweep itself.

---

## §B4 post-gate + the toolbar-click disarm edge — VERDICT: **REACHABLE** (a real bug)

The one open edge from the §B4 review (`wf_34d75a00`) is **confirmed reachable** by an independent
code trace (read-only). The paragraph-direction gesture in `src/lib/editor/paragraphDir.ts`
registers all its listeners via `EditorView.domEventObservers({...})`, which CM6 attaches to the
editor's **contentDOM** (`@codemirror/view` `ensureHandlers` → `dom = this.view.contentDOM`,
`dom.addEventListener`). The gesture arms on Ctrl+Shift keydown and disarms on contentDOM
`mousedown` (:210) or `blur` (:213) — but **not** on a mousedown outside contentDOM.

- **Blur double-neutralizes ordinary chrome.** Any outside mousedown whose default action proceeds
  moves focus off contentDOM → `blur` fires (registered on the element itself) → disarm; and the
  subsequent keyup then targets the newly-focused element, never reaching contentDOM. So ordinary
  buttons (no `preventDefault`) self-neutralize.
- **The surviving path — focus-preserving `preventDefault`-on-mousedown chrome.** Constellation's
  note toolbar strip container calls `onmousedown={(e) => e.preventDefault()}`
  (`NotePane.svelte:1449`) precisely so a formatting click keeps the editor focused. That suppresses
  the browser's focus-transfer default → contentDOM never blurs (:213 never runs), the mousedown
  never passes through contentDOM (:210 never runs), `armed` survives, and the Ctrl/Shift keyup
  dispatches to `document.activeElement` = contentDOM → the observer fires the flip.

**Reachable recipe (deterministic):** open a note in NotePane, click into a prose paragraph, hold
**Right Ctrl + Shift**, still holding click the **Bold "B"** (or any pixel of the toolbar strip),
release Ctrl+Shift → the whole caret paragraph is silently forced 100% RTL (an invisible U+200F
inserted at each content line start, dispatched as a real edit → note dirtied → the marks persist to
disk on the next durable save). Same class: the breadcrumb promote/demote/trail buttons
(`NotePane.svelte:1306/1318/1330/1333`) and every layout resize divider
(`+layout.svelte` `startResize`/`startFlankResize`/`startSplitResize`/WiW handles) — hold Ctrl+Shift
while dragging a splitter, release → flip.

**Safe by construction:** the FileTree Ctrl/Shift+click multi-select (the app's real Ctrl+Shift+click
semantic, `FileTree.svelte:50-67`) `preventDefault`s the **click**, not the mousedown → focus is
lost at mousedown → blur disarms (double-neutralized). The `CodeMirrorEditor`/`FormattingToolbar`
`.cm-toolbar` is mounted nowhere in the shipped app. PropertyEditor suggestion dropdowns preserve the
property *input's* focus, not the editor's.

**Minor adjacent gap (same family, LOW):** mouse **wheel** is not a disarm trigger anywhere —
Ctrl+Shift+scroll then release also fires; arguably still "no intervening key or click," so it may be
by design.

**Fix direction (not applied — this is the register):** a window-level capture-phase `mousedown`
disarm belt registered/cleaned per editor instance (disarming on `keyup` when `!view.hasFocus` is
NOT sufficient — focus IS preserved in this edge). Fold the wheel case in. → filed as a new PJ; it is
an editor content-integrity dirty-write, so it is treated in the same batch as the sweep APP-KILLERs.

---

## APP-KILLER findings

### [APP-KILLER] `src/lib/components/PropertyEditor.svelte:480` (editor-lifecycle) — cross-note-bleed

**#1. Summary.** The embedded PropertyEditor's onDestroy flush calls saveTabContent(tabId, filePath, editableProps, body) with LIVE prop getters: during the NoteEditor {#key} teardown (tab switch or in-place wikilink nav) tabId/filePath already read the INCOMING note (the exact window NotePane.svelte:290-297 documents and guards with mountedFilePath), while the local $state editableProps still holds the OUTGOING note's frontmatter (its re-sync $effect is torn down, never re-run). saveTabContent then runs editNoteProps(tabId, oldProps, newPath) — the model was already re-seeded to the new path by openNoteTab/openNoteModel, so the expectPath identity guard PASSES — and durably writes the new note's body under the OLD note's frontmatter (title, cid_cn, tags, stage). The gate `if (saveTimeout)` (line 146/474) is a stale-truthy timer handle never nulled after firing, so this flush fires on EVERY teardown following any props edit (or the auto date-converter's debouncedSave) in that pane instance's life, not only pending ones. No guard in the chain can refuse: isCascading is false, saveLocks false, model path matches.

**Scenario.** Expand the Properties strip in note A's NotePane and edit any property (or let the calendar auto-converter call debouncedSave). Any time later, click a wikilink to note B (in-place tab reuse) or switch to tab B. openNoteTab flushes the (clean) model, re-seeds it to B, the {#key} bumps, and the destroyed PropertyEditor's onDestroy runs: tabId/filePath now resolve to B, editableProps still = A's props → editNoteProps poisons B's model with A's frontmatter (guard passes: both sides say B) → saveNoteSession composes B's body + A's props and durably writes B's .md. B's disk file silently acquires A's title/cid_cn/stage/tags (duplicate cid_cn also corrupts the index); tab.content is stomped to the same frankenstein; no error, no banner — the BUG-023 content-integrity class reintroduced through the props channel.

**Triage.** **NEW** — not in the ledger. Content-integrity class (BUG-023 lineage) via the props channel; LL-014 three-strike class; MIG-076 Single-Ownership territory.

### [APP-KILLER] `src/lib/libraries/store.ts:1348` (notemodel-ownership) — content-loss

**#2. Summary.** loadTabHistoryEntry has NO one-path-one-tab dedup (the B1 DEDUP_ALL_TABS guard exists only in openNoteTab at line 2100), so Alt+Left/Right can land a tab on a path that is ALREADY open in another tab, minting a second independent NoteModel for the same note — the exact two-models-one-path clobber class B1 was built to kill.

**Scenario.** Tab B visits note N, then navigates on to M (N stays in B's history). User opens N again — dedup finds no tab on N, so it opens in tab A. User types in A; the 1500ms debounce saves — disk now has A's edits; the app's own write is watcher-suppressed (markRecentWrite/recentWrites), so tab B is never healed. User presses Alt+Left in tab B: loadTabHistoryEntry raw-reads N... wait, it reads CURRENT disk, so B's model seeds with A's edits — but from that moment the two models diverge: user keeps typing in A (autosaved), then clicks tab B (stale clean model, no reload on switchTab), types one character — B's save composes its stale body + the character and writes it: every edit A made since B's seed is silently reverted on disk. No error, no conflict sidecar (the write is 'ours', not external), model A still believes it is clean and saved. Index reindexes the stale content too. APP-KILLER: committed on-disk content lost with zero surfaced signal.

**Triage.** Escalated. Flagged as HIGH in the 2026-07-14 register (`store.ts:1283`, loadTabHistoryEntry bypasses B1 dedup); never got its own PJ. Now confirmed APP-KILLER. Sibling of PJ-099 (same function, different facet).

### [APP-KILLER] `src/routes/+layout.svelte:8019` (rename-cascade-integrity) — silent-data-loss

**#3. Summary.** FocusPane sits entirely outside the rename-cascade freeze: the focus branch (8013-8056) mounts no CascadeFreezeOverlay (the overlay lives only in the non-focus else branch at 8202 and the split/index panes at 7991/7453), so the user can keep typing during the whole ~7s cascade window with zero visual signal — and reloadTabsFromDisk then FORCE-adopts disk into the now-dirty model (store.ts:815 openNoteModel, unconditional, no dirty check), after which the H3 focusReseed remount (+layout 6373-6376) re-seeds FocusPane from the reseeded model. Every keystroke typed during the cascade is silently discarded from model, screen, and disk.

**Scenario.** Note B (backlinks A) open in Focus mode; the file tree is still visible beside FocusPane. User right-click-renames A in the tree -> cascade starts: cascadeFreeze covers B's tab.path but no overlay renders over FocusPane; markCascading gates NoteEditor saves but not focus typing. During the multi-second walker scan the user clicks back into FocusPane (no pointer block) and keeps capturing -> editNoteBody makes model B dirty. Walker rewrites B on disk ([[A]]->[[A2]]); B is in result.rewritten; reloadTabsFromDisk force-adopts disk into the DIRTY model (the exact PJ-092 invariant breach documented at store.ts:778-785: 'force-reseeding the dirty model IS the data-loss'), then focusReseed remounts FocusPane on the reseeded model. The typed text vanishes; the armed commitFocusSave timer later composes from the reseeded model, so the keystrokes are never written anywhere. No error, no conflict, no overlay ever told the user to stop typing.

**Triage.** Escalated. Maps to **PJ-097** (FocusPane not covered by CascadeFreezeOverlay — filed LOW/MED, 'contrived re-type race'). The sweep re-classifies it APP-KILLER (silent edit-loss + force-adopt into a dirty model — the PJ-092 invariant breach). Family: #14, #26.


## HIGH findings

### [HIGH] `src-tauri/src/sources/mod.rs:468` (cece-sources-derived) — false-success

**#4. Summary.** On a note with unclosed/malformed frontmatter (starts with '---' but no later line beginning '\n---'), rewrite_frontmatter_sources/rewrite_frontmatter_content_type silently return the content UNCHANGED, yet rewrite_note_sources_on_disk (mod.rs:571-582) still returns `effective` ids and the callers (sources_set_manual mod.rs:667/677, content_type_set_manual mod.rs:1177/1181, bulk accept_one bulk_ops.rs:317-333) write those ids into note_meta and return Ok — the classification never landed on the source of truth.

**Scenario.** A note's first line is '---' used as a horizontal rule (or its frontmatter close was corrupted), with no other line starting '---'. User sets sources/content_type in the PropertyEditor (or Approve-All accepts a suggestion — which ALSO clears the suggestion row): rewrite returns content unchanged, gate_rmw writes identical bytes (no error), note_meta.sources gets the ids, the IPC returns Ok, the UI shows the note classified. Later ANY reindex of that file (editor save → index_note, search.rs:5798/5808 re-extracts from frontmatter → empty) silently wipes note_meta.sources/content_type back to []. The user's asserted classification vanishes with no error at any point; in the bulk path the consumed suggestion is gone too, and the background scan may or may not re-propose. Disk never held the value → false success + delayed silent loss.

**Triage.** **NEW** — CE sources/content_type. Malformed-frontmatter false-success then reindex-wipe.

### [HIGH] `src/lib/libraries/store.ts:880` (cross-window-integrity) — toctou

**#5. Summary.** adoptExternalChangeIntoTabs captures `tabs = get(openTabs)` (line 880) BEFORE the awaited batch disk reads (887) and then loops the STALE snapshot (908): an in-place nav during the await REPLACES the tab object (openNoteTab line 2222 sets path to the new note and re-seeds the model), but the stale captured tab still carries the old path+same id, and adoptDisk (noteModel.ts:341) has NO path guard — so the OLD note's disk content is adopted into the NEW note's clean model.

**Scenario.** Note P_old is open in tab T; Syncthing/Obsidian modifies P_old -> watcher flush calls adoptExternalChangeIntoTabs([P_old]). During the awaited Promise.all read, the user clicks a wikilink in T -> in-place reuse: tab T becomes P_new, openNoteModel re-seeds T's model to P_new (clean). The read resolves; the loop over the STALE tabs array finds t.path===P_old, calls externalChangeNoteModel(T.id, oldDisk); adoptDisk sees a clean model, content differs, baseline differs -> ADOPTS: the P_new model now holds P_old's body AND FRONTMATTER (props, tags, cid_cn, stage), marked clean with poisoned baseline. The store update keys on adopted.has(t.path) against the CURRENT tabs (path=P_new) -> no remount, no reseeding gate, no visible change (screen still shows P_new correctly). The user types one character -> editBody pushes only the BODY from the view; the adopted P_old props remain -> compose writes P_old's frontmatter + P_new's body durably to P_new on disk — a silent frankenstein write including a cid_cn identity swap. Window is milliseconds but the same helper serves BOTH the watcher and the cross-window screen:note-saved adopt (+layout.svelte:3453). Fix shape: re-get(openTabs) after the reads (as SecondScreenPage.adoptFreshDiskIntoSS:564 correctly does) and/or add an expectPath guard to adoptDisk like markSaved/noteDiskSynced have.

**Triage.** **NEW** — adoptExternalChangeIntoTabs stale-snapshot TOCTOU → frankenstein cross-note write. Sibling of #13 (both cross-note-bleed via a missing generation/path guard).

### [HIGH] `src/lib/components/SenseMakingCanvas.svelte:148` (frontend-write-callers) — swallowed-write-error

**#6. Summary.** Sense-Making Canvas debounced write_canvas swallows every write failure with a bare catch {} — the canvas is the ONLY persistence for the user's arranged knowledge items and there is no retry, no banner, and no app-close flush for the pending 1000ms debounce.

**Scenario.** User opens the Sense-Making Canvas (command palette, CE Phase 11) and spends a session dragging notes/ideas into the Cynefin quadrants. The canvas file becomes unwritable (cloud-sync/AV lock, read-only attribute, or the library path moved) → every debouncedSave() invoke('write_canvas') rejects and catch {} (line 148) discards it. The on-screen canvas keeps showing every placement, so the user keeps arranging for an hour with zero feedback. Next open of the canvas reverts to the last successful write — the whole arrangement session is gone. Second shape, no error needed at all: user drags an item and closes the app (or toggles to another full-page view then quits) within the 1000ms debounce — the timer never fires; the PJ-103 app-close flush covers dirty NOTE models only, not the canvas timer, so the final edits are silently dropped.

**Triage.** Maps to **PJ-100** (SenseMakingCanvas write_canvas swallows errors; no net/retry/flush-on-destroy). Re-confirmed HIGH; the app-close-flush gap is new emphasis.

### [HIGH] `src/lib/editor/yamlDoc.ts:213` (frontmatter-property-writes) — false-success

**#7. Summary.** composeFrontmatter's H1 passthrough silently discards EVERY frontmatter property edit on a note whose YAML fails strict parse (duplicate keys, tab indentation, unclosed quote), while the save succeeds, the model is marked clean, and the UI keeps showing the edit — hasErrors is consumed nowhere and nothing is reported.

**Scenario.** Note imported from Obsidian carries a duplicate key (a common sync-merge artifact) — eemeli parseDocument flags it as an error, but the tolerant hand-rolled parseFrontmatter happily projects its props, so the PropertyEditor is fully editable. User promotes the stage (or adds a tag, connects a typed link, edits any property) → saveTabContent → editNoteProps → composeModel → composeFrontmatter hits `if (parseDocument(rawYaml).errors.length) return ...rawYaml...` and returns the ORIGINAL frontmatter verbatim, never applying the diff. The write lands, markSaved cleans the model, the UI shows the promoted stage. On tab reopen / restart the edit is gone; the index reindexes the unchanged disk. This repeats on every future edit of that note, forever, with zero error anywhere (the header comment promises 'preserved-as-is + reported' — the report was never implemented). Reviewer typed-link connects on such a note also silently no-op while 'updating optimistically'.

**Triage.** Related to **PJ-085/PJ-073** (frontmatter/YAML round-trip). This facet: strict-parse-fail → composeFrontmatter passthrough silently discards ALL prop edits, forever, on that note.

### [HIGH] `src/lib/editor/yamlDoc.ts:150` (frontmatter-property-writes) — content-corruption

**#8. Summary.** serializeLine has no nested-object-list case — a changed ikhtilāf prop falls into the plain `value = prop.value` branch, so composeFrontmatter splices the CST seq-of-maps and writes back the flat compact-summary string, destroying the structured YAML on disk (legacy reconstructFrontmatter at store.ts:1721-1740 handled this; the G4 byte-perfect replacement dropped it).

**Scenario.** Note has `ikhtilāf:` with rows `- school: Hanafī / position: permissible` etc. User edits one row (or adds/removes a row) in the PropertyEditor ikhtilāf widget → updateNestedField rebuilds the summary value → debouncedSave → composeFrontmatter detects the value diff, finds the CST item is a block-seq (not 'scalar'), splices it and appends serializeLine(key, np) — which for type 'nested-object-list' emits `ikhtilāf: "school: Hanafī / position: permissible | school: Mālikī / ..."` — a flat scalar. The structured seq-of-maps is gone from disk. On reopen, parseFrontmatter projects a plain scalar with no nestedObjects, so the widget shows zero rows: the user's comparative-jurisprudence data is silently flattened and effectively lost. Same flattening hits a NEWLY created nested-object-list property (rows are never persisted as structure).

**Triage.** Maps to **PJ-073/PJ-085** — the explicitly-named `yamlDoc.ts:150` nested-object-list ikhtilāf collapse. Re-confirmed HIGH.

### [HIGH] `src/lib/components/SenseMakingCanvas.svelte:271` (note-save-index) — index-divergence

**#9. Summary.** Canvas promote-to-note: createNote() indexes the empty skeleton synchronously (Rust create_note), then writeNote(newPath, frontmatter + item.content, 'canvas_export') overwrites it with the real content with NO reindex call — note_meta/notes_fts permanently hold the empty skeleton for this note.

**Scenario.** User promotes a canvas item to a note. create_note writes+indexes '---\ntitle...---\n\n' (empty body), then the frontend immediately writes the real content (stage: growth frontmatter + the item text) via the gated write_note, which is watcher-suppressed and never reindexed. The note's body is unsearchable, its wikilinks never derive note_links/sky edges, word_count=0, and because boot uses reindex_library onlyIfUnindexed:true and reconcile.rs only heals path-level drift, nothing ever refreshes the row — divergence is permanent until the user happens to edit and save that exact note or manually rebuilds the whole index. No error anywhere; the note opens fine on screen, so the user believes it is fully captured.

**Triage.** Related to the create-then-write-same-second index-divergence class (2026-07-14 register had ExpressionForge:144 + this at HIGH). No dedicated PJ yet.

### [HIGH] `src/lib/libraries/store.ts:1361` (notemodel-ownership) — silent-data-loss

**#10. Summary.** loadTabHistoryEntry reads the note via raw invoke('read_note'), bypassing resolveNoteContent entirely — no write-ahead-net recovery, no recoveredFromNet/markModelRecoveredFromNet dirty-marking. A note whose ONLY copy of unsaved edits lives in the write-ahead net (failed save, tab closed — closeTab's documented contract at line 2593 relies on the net + reopen-restore) reopened via another tab's history seeds a CLEAN model from stale disk.

**Scenario.** Note Y's save fails (locked .md) → model dirty + net set + banner. User closes Y's tab (close_flush fails, proceeds by contract — 'the write-ahead net preserves it and restores it on reopen'). Later, another tab whose history contains Y gets Alt+Left: loadTabHistoryEntry seeds Y from stale DISK, model born clean; the saveHealth retry sees the tab open but NOT dirty and CLEARS the failure entry (retrySaveFailure line 427) — the banner vanishes. User edits + saves Y: setNet overwrites the recovery net with content composed from the stale disk base. The failed-save edits are now gone from screen, disk, AND the net — the documented reopen-restore route (openNoteTab→resolveNoteContent) never ran. Totally silent.

**Triage.** loadTabHistoryEntry cluster (with #2). Facet: raw read bypasses resolveNoteContent → no write-ahead-net recovery. Ties to PJ-099 + the net-durability work (PJ-110).

### [HIGH] `src-tauri/src/review.rs:750` (persisted-json-state) — silent-data-loss

**#11. Summary.** load_pulse_data swallows BOTH read errors and parse errors on review-pulse.json (if let Ok chains, fallback ReviewPulseData::default), and mark_reviewed/snooze_note/dismiss_note do a read-modify-write on that result — a transient read failure makes the RMW overwrite the user's entire review history with a near-empty file.

**Scenario.** User clicks '✓ Reviewed' while review-pulse.json is momentarily locked by AV/sync (read_to_string fails) or is partial from a prior crash (parse fails): load_pulse_data returns default() with empty last_reviewed/intervals/snoozed/dismissed maps; mark_reviewed inserts just this one note and save_pulse_data (line 687) writes a 1-entry pulse file over the whole history. Every note's interval ladder resets to day 0, all dismissals resurface, all snoozes vanish — command returns Ok(()), nothing surfaced. review-pulse.json is the documented JSON source of truth that the review_schedule table is backfilled from, so the loss propagates to the index too.

**Triage.** Maps to **PJ-075/PJ-087** (review-pulse non-atomic). This is the READ-side twin: load_pulse_data swallows read+parse errors → RMW overwrites history. Same file as the 2026-07-14 AK (review.rs:762).

### [HIGH] `src-tauri/src/universe.rs:384` (persisted-json-state) — silent-data-loss

**#12. Summary.** ensure_universe_notes_folder (flat branch) swallows a libraries.json read/parse failure via .ok().and_then(...).unwrap_or_default() and then REWRITES libraries.json with only the auto-inserted universe_notes entry (persist_json_best_effort at line 398), destroying every other library registration.

**Scenario.** Runs on EVERY boot/universe switch (set_active_universe -> line 739). libraries.json is momentarily unreadable (Windows share-violation from AV/OneDrive/Syncthing holding the file) or is partial/corrupt: fs::read_to_string(...).ok() -> None -> unwrap_or_default() -> empty vec -> 'no universe_notes entry' -> inserts one -> atomic_write commits a ONE-entry libraries.json. Every additional/external library registration is silently and cleanly overwritten. Unlike libraries.rs::load_libraries (G6 W1-8), this path makes NO timestamped backup, prints only an invisible eprintln, and the command returns Ok. User's notes on disk survive but all non-default libraries vanish from the app with no error.

**Triage.** **NEW** — ensure_universe_notes_folder swallows a libraries.json read failure → rewrites with ONLY the universe_notes entry → destroys every other library registration. Likely a mechanism behind **PJ-072** (registry mystery); adjacent to PJ-104.

### [HIGH] `src/lib/components/ConflictMergeView.svelte:144` (reactivity-concurrency) — cross-note-bleed

**#13. Summary.** The merge-view rebuild $effect has NO stale-result/generation guard: build() spans two long awaits (readNote IPC of the .conflict sidecar + the lazy @codemirror/merge dynamic import) and unconditionally assigns `mergeView` when it lands, so a build started for conflict A can overwrite the MergeView AFTER `target` has switched to conflict B — and saveMerged() (line 153) has no identity check that `mergeView` was built FOR `target`, so it passes A's pane text to resolveConflictMerge(B.notePath, B.sidecarPath, ...), which replaceContent()s note B's model and durably writes note A's content into note B's .md (store.ts:4411-4441). The superseded EditorViews are also never destroyed (both stay parented to mountEl — a Rule-4 leak).

**Scenario.** A sync tool touches two open dirty notes -> two .conflict sidecars, two banner rows. User clicks Merge on note A; while the view is still loading (readNote + cold dynamic import, hundreds of ms) they hit Cancel (target->null runs destroyView but the in-flight build(A) is NOT cancelled) and click Merge on note B. build(B) completes first (import now cached); build(A)'s readNote resolves last and stomps `mergeView` with A's panes while `target` remains B. User resolves chunks and clicks Save merged -> resolveConflictMerge(B.notePath, B.sidecarPath, mergedText = note A's content) -> B's model is replaceContent()ed and durably saved: note B's file on disk now holds note A's content, every step reporting success (the BUG-023 cross-note contamination class, delivered THROUGH the durability gate). Fix shape already standard in this codebase: a capture/token compare after each await (the semanticFetchToken pattern) + destroy the stale view.

**Triage.** **NEW** — ConflictMergeView rebuild $effect has no stale-generation guard → saveMerged writes conflict A's content into note B's file. Cross-note-bleed through the durability gate (BUG-023 class). Sibling of #5.

### [HIGH] `src/routes/+layout.svelte:1546` (rename-cascade-integrity) — toctou

**#14. Summary.** commitFocusSave bypasses the F2 post-cascade-stomp gate: it calls saveNoteSession directly, which is NOT gated by isCascading (store.ts:976 documents this: 'saveNoteSession (a DIRECT write, NOT gated by isCascading)'), while every NotePane save path checks isCascading (NoteEditor.svelte:250 handleSave, :311 handleFlush) and saveTabContent checks it too (store.ts:1201). A debounced focus save landing inside the cascade window writes the model's pre-cascade body over the walker's rewrite — and noteSession.save writes UNCONDITIONALLY even when the model is clean (noteSession.ts:177-198, no dirty check), so even an already-flushed model's armed timer re-writes stale bytes.

**Scenario.** Note B open in Focus, backlinking A. User types (1500ms focusSaveTimer armed / or types during the cascade per the sibling finding), then rename of A starts. flushAllTabsInLibrary flushes B durably, excluded=[]; the walker rewrites B's [[A]]->[[A2]] on disk under gate_rmw. The ungated commitFocusSave fires mid-walk and its gate_write lands AFTER the walker's rewrite of B (gate_rmw only serializes, it does not order): B's disk reverts to [[A]] + the focus text. reloadTabsFromDisk then reads B and adopts the reverted content — everything looks consistent, the toast reports B as rewritten, but B's wikilink silently still says [[A]]. note_links reindex of B (from the focus save's own reindexNote) matches the reverted disk, so no surface ever flags the divergence between 'cascade reported rewritten' and the actual disk.

**Triage.** FocusPane-cascade family (with #3, #26). commitFocusSave bypasses the isCascading gate; noteSession.save writes unconditionally even when clean.


## MED findings

### [MED] `src-tauri/src/search.rs:10267` (boot-init-ordering) — false-success

**#15. Summary.** reindex_notes_matching_text returns Ok(0) when state.db is None, and its sole IPC caller reindex_arabic_overrides (arabic/overrides.rs:800-806) never calls ensure_search_db_ready — an Arabic-override add/remove during the DB-not-ready window silently skips the targeted FTS re-tokenization while the Settings UI shows a success toast.

**Scenario.** User opens a large universe (or switches universes; cold init takes 20-40s per MIG-078 comments, during which state.db is None), immediately opens Settings → Arabic Overrides and pins an override for a surface (e.g. "خليفة"). add_arabic_override persists the JSON + updates ACTIVE_STORE (fine), then ArabicOverridesPanel.reindexFor invokes reindex_arabic_overrides → reindex_notes_matching_text hits db.as_ref()==None at search.rs:10265-10268 and returns Ok(0). The panel renders the success status "Reindexed 0 notes" (ArabicOverridesPanel.svelte:171-175) — indistinguishable from "no note mentions it". Every note whose body/name mentions the surface keeps its OLD Layer-0 tokenization in notes_fts permanently: no .md file changed, so the watcher never fires, the boot reconcile (dead-path/cid relocate only, reconcile.rs) never re-tokenizes, and index_note's mtime gate skips these notes on any future walk. Query-time tokenization uses the NEW verdict while the index carries the old tokens → those notes silently vanish from matching searches until each note is individually edited or the user manually rebuilds the whole index. Unlike the registered PJ-093 sites, this divergence has NO self-heal path at all. Fix shape: ensure_search_db_ready(&app)? at the top of reindex_arabic_overrides (the ensure-first discipline reindex_changed_paths adopted at search.rs:9530) + Err instead of Ok(0) on a None conn.

### [MED] `src-tauri/src/cece/history.rs:69` (cece-sources-derived) — cross-note-bleed

**#16. Summary.** note_state_history (the SOLE store of the epistemic temporal record — not derivable from .md files) relies on FOREIGN KEY ... ON DELETE CASCADE for lifecycle, but PRAGMA foreign_keys is NEVER enabled on the production connection (only in a history.rs unit test at line 433), and note renames (reconcile.rs:274, libraries.rs:1049: UPDATE note_meta SET path=?2) migrate nothing in this table — history rows silently detach on rename and orphan on delete, then bleed to any new note created at the old path.

**Scenario.** User renames note A (or the watcher/reconcile detects an external rename): note_meta.path updates, but A's entire note_state_history timeline stays keyed to the old path — cece_get_note_history(A-new-path) returns [] with no error; the user's 'evolution of my stance' record has silently vanished from every surface. Delete works the same way (the CASCADE never fires, orphan rows persist). If the user later creates a DIFFERENT note at the old path, cece_get_note_history returns the dead note's epistemic timeline as if it belonged to the new note — cross-note bleed of temporal data. Same dead-FK applies to sources_suggestions (mod.rs:173), whose docstring falsely claims the cascade 'keeps the queue clean when notes are deleted'.

### [MED] `src-tauri/src/sources/mod.rs:481` (cece-sources-derived) — content-loss

**#17. Summary.** The frontmatter strip in rewrite_frontmatter_sources (and the twin at mod.rs:1054 for content_type) drops ANY line whose trimmed form starts with 'sources:'/'content_type:' anywhere in the frontmatter — including a line inside another field's multiline block scalar (e.g. 'summary: |') — and if that embedded line has an empty value, skip_block then also eats the user's following '- ' lines.

**Scenario.** A note's frontmatter contains a multiline field, e.g. 'summary: |' followed by an indented line 'sources: various citations below' and '- Ibn Khaldun'. The user accepts a classifier suggestion (or sets sources in the PropertyEditor): the rewriter deletes the embedded 'sources:' line and the '- ' lines from inside the summary block, writes the mutilated frontmatter to disk through the gate with no error, and mirrors cleanly to the DB. The user's own frontmatter prose is silently destroyed on the source-of-truth .md; nothing surfaces until the user happens to reopen properties and notice the truncated summary.

### [MED] `src/lib/components/SecondScreenPage.svelte:613` (cross-window-integrity) — cross-note-bleed

**#18. Summary.** The SS's screen:note-saved handler checks `editorPanelsData?.notePath === path` BEFORE awaiting read_note but spreads the result AFTER the await with no re-check and no generation token (`editorPanelsData = { ...editorPanelsData, content }`), so a cockpit focus-switch that lands during the read produces note B's identity carrying note A's content — the one stale-result site in this file without the gen-counter guard used everywhere else (scGeneration/peekGeneration/skyviewGeneration).

**Scenario.** Cockpit shows note A. Main's 1500ms debounced editor_save of A completes -> broadcastNoteSaved(A) -> SS u2 passes the notePath guard and starts invoke('read_note', A). Within that IPC roundtrip the user switches to note B on main -> the $effect at +layout.svelte:562 emits editor-panels(B) -> SS's onEditorPanels (u18) sets editorPanelsData = {notePath: B, content: B}. The read_note(A) then resolves and executes `editorPanelsData = { ...editorPanelsData, content: A-content }` -> the Knowledge Cockpit renders note B's name/path with note A's CONTENT and link zones parsed from A's body. No error, no console output; the display self-corrects only on the next tab switch or save. Read-only window so it cannot reach disk, but it is exactly the on-screen cross-note contamination class (BUG-023 shape) on the second screen.

### [MED] `src/lib/components/SecondScreenPage.svelte:683` (cross-window-integrity) — index-divergence

**#19. Summary.** The SS's `library-changed` listener only debounce-reloads the note LIST (loadAllData) and never calls adoptFreshDiskIntoSS, and the main window's watcher adopt (adoptExternalChangeIntoTabs) broadcasts nothing — so a note changed OUTSIDE the app that is open in the SS's OWN tabs or peek keeps displaying the pre-edit body indefinitely.

**Scenario.** User opens note X on the second screen via a DashboardView click (SS-realm openNoteTab -> SS-local tab; X is NOT open in the main window). X is then modified by Obsidian/Syncthing/another device sync. Rust emits library-changed to both windows: main has no tab for X (nothing to adopt); the SS's u5 handler only refreshes allNotes after 3s. screen:note-saved never fires (external edits are not app saves) and cascade:rewrote never fires (not a rename). The SS's NoteEditor for X keeps rendering the stale pre-sync body for hours with zero cue — the Boss reads outdated knowledge from a window whose entire purpose is contextual truth. The G3 adopt machinery (adoptFreshDiskIntoSS, freshness-gated, remount-on-adopt) exists in this file but is only wired to screen:note-saved and cascade:rewrote, not to the watcher event.

### [MED] `src/routes/+layout.svelte:2740` (cross-window-integrity) — index-divergence

**#20. Summary.** Universe switch never notifies the second screen: notifyUniverseSwitch() (secondScreen.ts:96) has ZERO callers anywhere in the codebase and no Rust emitter, so the SS's onUniverseSwitch listener (SecondScreenPage.svelte:650) is dead code — after a switch the SS silently keeps the OLD universe's title, allNotes, $libraries, link-type registry, and its own open tabs, and the SS window survives hidden (lib.rs:264 hides, never destroys) so the stale state persists indefinitely.

**Scenario.** SS open on monitor 2 showing Universe A's dashboard/tabs. Boss switches to Universe B via UniverseManager -> handleUniverseSwitch tears down and re-inits the MAIN window only; 'screen:universe-switched' is never emitted. The SS still shows 'Constellation - UniverseA' with Universe A's notes as if current. Main then pushes editorPanels for a Universe-B note; the SS cockpit resolves its library colors/paths and link zones against Universe A's stale $libraries/allNotes (mixed-universe display). Worse: right-clicking a stale Universe-A row on the SS forwards rename/move/delete/addTag to the main window (screen:request-note-action), which executes the mutation on a Universe-A file while the app is in Universe B — the write lands and the reindex targets Universe B's DB (a foreign-path index write), all with no error anywhere. Partial incidental healing only occurs if a Universe-B file later changes (the 3s library-changed debounce reload of the note LIST), never for the title, tabs, or registry.

### [MED] `src-tauri/src/search.rs:6007` (derived-index-triggers) — index-divergence

**#21. Summary.** note_links.target_cid_cn is resolved only when the SOURCE note is indexed and is never refreshed when the TARGET later acquires its cid_cn, so the Mode-2 staleness lens (review.rs:135 JOIN dep.cid_cn = jl.target_cid_cn with a NOT-NULL guard at review.rs:139) is permanently blind for every forward-link / external-note edge.

**Scenario.** User writes note A with `supports::[[B]]` before B exists (a red link — the natural PKM capture order), or B is an Obsidian-created note in a linked library that has never been opened (cid_cn injected lazily at first open per search.rs:5722-5725). A's note_links row is written with target_cid_cn = NULL (the LOWER(name) lookup at search.rs:6007-6013 misses). B is then created/opened (gets its cid) and its content is substantively edited weeks after A was ✓-reviewed. The Reviewer's stale lens and get_note_review_status never flag A — the probe requires jl.target_cid_cn IS NOT NULL — so the Boss-mandated 'stale because a load-bearing dependency changed' alarm silently never fires. Nothing heals it: no code path updates existing edges' target_cid_cn when a target gains a cid (the one-shot backfill at search.rs:2775 keys on target_path, which index_note never populates); a manual Rebuild-Index walk mtime-gates A out (index_note force=false early-return at 5695); recompute_all_incoming touches only aggregates. The NULL persists until A itself happens to be edited or B is renamed.

### [MED] `src-tauri/src/search.rs:9029` (derived-index-triggers) — index-divergence

**#22. Summary.** reconcile_filesystem — the ONLY caller of recompute_all_incoming / recompute_all_sky / review::recompute_all_in / tag_counts recompute — no longer runs on any cadence: the 2026-07-08 boot rewrite removed the boot/universe-switch walk (+layout.svelte:2578 'Runs on … Settings → Rebuild Index only'; the only remaining auto-triggers are add-library and the zero-index recovery), so every 'best-effort; reconcile is the authoritative self-heal' claim baked into the write-time maintenance (search.rs:1436, 1502, 9345, 9353, 4049-4054) now names a compensating control that never fires in steady state.

**Scenario.** maintain_incoming_after_save or maintain_sky_after_save fails once (SQLITE_BUSY past timeout during a concurrent backfill/walk-conn write) — the error is eprintln-swallowed at search.rs:9347/9354 while constellation_search_reindex resolves Ok — or any of the known best-effort gaps fires (PJ-074 archive/unarchive, a dropped sync_action_to_row). Pre-2026-07-08 that divergence was bounded: the next boot's reconcile walk ran recompute_all_incoming/recompute_all_sky/review recompute and healed it (the 07-14 sweep register's own scenarios still say 'heals at next boot via the mtime walk'). Today no boot, universe-switch, or watcher path ever calls reconcile_filesystem, so incoming_count / incoming_link_types / sky stratum+maturity / review_schedule / tag_counts drift is PERMANENT for the install unless the user manually adds a library or the index is wiped — silently converting a whole class of registered until-next-boot MED findings into forever-divergence, with zero surfaced signal.

### [MED] `src/lib/components/NoteEditor.svelte:248` (editor-lifecycle) — false-success

**#23. Summary.** NotePane.doSave clears its view-level dirty flag BEFORE the durable write (NotePane.svelte:300 `dirty = false` then onsave), and NoteEditor.handleSave drops the save entirely when a previous write is still in flight (`if (saving) return`, line 248) instead of letting noteSession's per-id saveChains serialize it. After the drop: view dirty=false (so the 30s idle timer's doSave no-ops via `if (!dirty) return`), model still dirty with the newer version, save-health has no entry (nothing failed), and nothing re-attempts the save until a later keystroke, a tab-switch flush, or app close. The saveChains machinery built for exactly this concurrent-save case (noteSession.ts:123-175) is starved by the component-level flag.

**Scenario.** A save write stalls >1.5s (locked .md — Syncthing/OneDrive/Defender, the MIG-098 contention class). The user keeps typing; the next 1500ms debounce fires doSave → dirty=false → handleSave returns on `saving`. The user stops typing and leaves the note open: disk and FTS index silently lag the screen indefinitely (no banner — no write failed), the write-ahead net still holds the OLDER first-save content (the newer content was never netted; the !needsDiskSave flush path only re-stashes on a tab switch). A crash/power loss in this state loses the post-stall keystrokes from disk AND the net, with zero surfaced error at any point.

### [MED] `src/lib/components/PropertyEditor.svelte:351` (editor-lifecycle) — silent-data-loss

**#24. Summary.** Property edits reach the note model only inside the 800ms-debounced saveTabContent (editNoteProps at store.ts:1217), so a fresh props edit is invisible to isNoteDirty and to every model-based departure flush (flushOutgoing, flushAllDirtyTabs, flushAllForAppClose). In the STANDALONE right-sidebar instance (+layout.svelte:8467, not {#key}-remounted, no onDestroy on tab switch), the tabChanged $effect (lines 344-375) unconditionally re-seeds editableProps from the new note's properties before the pending timer fires, discarding the edit; the timer then saves the new note's own props. The same sub-800ms window applies at app close (PropertyEditor has no beforeunload hook and Svelte onDestroy does not run on window unload), where flushAllForAppClose sees a clean model.

**Scenario.** User edits a property value in the right-sidebar Properties panel and within 800ms clicks another note in the file tree (or the app's graceful close fires). The $effect re-seeds editableProps from the incoming note (tabChanged branch runs unconditionally, `!saving || tabChanged`), the 800ms timer later composes the incoming note's unchanged props, and the outgoing note's property edit never reaches the model, the disk, or the write-ahead net. Switching back shows the old value. No error surfaces anywhere — a pure silent drop of user input with no recovery copy.

### [MED] `src/lib/components/PropertyEditor.svelte:815` (editor-lifecycle) — content-loss

**#25. Summary.** Two writable PropertyEditor instances are mounted simultaneously for the same note (embedded in NotePane + standalone right-sidebar panel), each holding its own $state editableProps copy, and debouncedSave publishes its update via DIRECT mutation `tab.content = buildFullContent(...)` (line 815) without an openTabs store update — so the sibling instance's `properties` prop (derived from $openTabs in NoteEditor.parsed) never re-fires and its snapshot goes stale. saveTabContent then REPLACES the model's entire props array from whichever instance saves last (editNoteProps, store.ts:1217), silently reverting the other instance's already-durably-saved edit on the next write.

**Scenario.** User expands the embedded Properties strip in the note, then edits property X in the right-sidebar Properties panel (saved durably after 800ms via the model). The embedded instance's editableProps still predate X (no store notification reached it). The user then edits property Y in the embedded strip: its debouncedSave pushes the FULL stale props array (old X + new Y) into the model and writes it to disk — property X's saved edit is reverted on disk and in the index with no error, no conflict banner, and nothing on screen hinting the revert until the note is reopened.

### [MED] `src/routes/+layout.svelte:8019` (editor-lifecycle) — silent-data-loss

**#26. Summary.** The FocusPane instance in the layout is wired with value/title/dir/onchange/onflush/onexit but NOT ontitlechange, so FocusPane.handleTitleBlur (FocusPane.svelte:96-98) calls ontitlechange?.(titleValue) on undefined — a no-op. The Focus title input is fully editable (bind:value, blur commit), so a title typed or corrected in Focus mode is accepted on screen and then silently discarded; on exit the note reverts to its old title with no rename, no frontmatter update, no error.

**Scenario.** User enters Focus mode, pauses typing so the title field appears, types a new title (the field visibly accepts it), continues writing, then exits Focus (Escape). The body edits persist through the model flush, but the title change vanished at blur time: no renameItem, no title: frontmatter update, no wikilink cascade, no error. Back in NotePane the old title is shown — the user's deliberate title edit was silently swallowed by an unwired callback.

### [MED] `src-tauri/src/libraries.rs:5940` (freeze-and-leaks) — freeze-hang

**#27. Summary.** resolve_embed_image is a SYNC #[tauri::command] (no `(async)`) that reads a full image file into memory and base64-encodes it on the IPC dispatch thread; it is the sibling of resolve_embed (which was moved to `(async)` in Batch-W 2026-07-04) and was missed.

**Scenario.** The user opens a note in NotePane containing several embedded images (e.g. `![[photo.png]]`). livePreview's ImageWidget.toDOM() fires one `invoke('resolve_embed_image', …)` per uncached image on render. Each call runs synchronously on the single IPC dispatch thread: for a multi-MB image the file read + base64 encode blocks that thread for tens-to-hundreds of ms; N images serialize (they queue on the one dispatch thread). While the thread is blocked, EVERY other IPC — including the note's debounced write_note save and any keystroke-adjacent command — is stalled. The UI freezes for the summed duration with no error surfaced. Matches the exact G8 dispatch-thread-blocking class the project already fixed for scan_note_stages/execute_lens; one-word fix `(async)` (the resolve_embed sibling already has it).

### [MED] `src-tauri/src/libraries.rs:1794` (freeze-and-leaks) — freeze-hang

**#28. Summary.** list_universe_folders is a SYNC #[tauri::command] (no `(async)`) that recursively walks the folder tree of EVERY registered library (depth 30, `collect_folders`) on the IPC dispatch thread.

**Scenario.** The user right-clicks a note/folder → Move (or otherwise opens the Move picker); +layout.svelte:5945 does `await invoke('list_universe_folders')`. On a large universe (7,600+ notes with deep nesting across multiple libraries) the recursive read_dir walk of the entire directory structure runs on the dispatch thread, freezing all IPC (including in-flight saves) until the walk completes, with no error and no progress indication. Same G8 dispatch-thread class as W3-1/W3-2; fix is `(async)` to move it onto the worker pool.

### [MED] `src/lib/components/BacklinksPanel.svelte:199` (frontend-write-callers) — swallowed-write-error

**#29. Summary.** linkMention() wraps its whole read→replace→write_note of ANOTHER note's .md in catch { /* ignore */ } — the user's explicit 'link this unlinked mention' action can fail with zero surfacing and the wikilink is never written.

**Scenario.** In the Backlinks panel's Unlinked Mentions list the user clicks the link-mention button for a note that mentions the active note by name. The target file is momentarily unreadable/unwritable (sync lock) or write_note rejects → the entire operation lands in catch { /* ignore */ }: no error, no panel update. The user believes the mention is now a real [[wikilink]] feeding backlinks/note_links; on disk nothing changed and the knowledge connection they explicitly formed does not exist. Because nothing refreshes on failure, the row just sits there unchanged — indistinguishable from a slow refresh.

### [MED] `src/lib/components/ReviewStatusPanel.svelte:97` (frontend-write-callers) — swallowed-write-error

**#30. Summary.** commitPriority (line 97) and act() mark_reviewed/snooze_note/dismiss_note (line 78) swallow set_review_priority / review-schedule write failures with bare catch {} — the review_schedule source-of-truth silently keeps the old state with no feedback on an explicit user action.

**Scenario.** User drags the priority lever on the note's Review Status tab, or clicks 'Mark reviewed' after re-reading a note. The invoke rejects (DB briefly locked by a backfill/reindex writer, conn contention). catch {} drops it: no error, and the follow-up load()/onRefresh?.() inside the try is skipped, so the panel doesn't even re-render to betray the failure — the dragged priorityDraft value remains on screen looking committed. review_schedule never gets the override / reviewed-timestamp; the note keeps resurfacing as due (or keeps the computed priority) with the user's explicit review judgement silently discarded. Same bare-catch class in ReviewerView.svelte:286/290 (commitPriority/resetPriority), where a failure leaves priorityDraft displayed as if committed.

### [MED] `src/lib/components/UniverseSetup.svelte:244` (frontend-write-callers) — silent-data-loss

**#31. Summary.** migrateLocalStorage() swallows each save_universe_settings/bookmarks/workspaces/property_types failure with catch { /* ignore */ } (lines 204/212/220/235), then UNCONDITIONALLY deletes every constellation-* localStorage key (lines 237–244) — a failed migration destroys the only copy of the legacy data.

**Scenario.** A legacy install (pre-universe, settings/bookmarks/workspaces/prop-types in localStorage) creates its first Universe. During migrateLocalStorage the universe root is not yet writable for one of the saves (or JSON.parse throws mid-block) → that block's catch ignores it. Execution then reaches the cleanup loop which removes ALL 'constellation-' keys regardless of whether the corresponding save succeeded. The user's bookmarks/workspaces/property-type assignments are now neither on disk nor in localStorage — permanently gone, with no error and nothing visibly wrong until they look for a bookmark that no longer exists.

### [MED] `src/lib/libraries/store.ts:1413` (frontend-write-callers) — false-success

**#32. Summary.** saveCollections() is fire-and-forget with a console-only .catch — a failed save_universe_collections write is invisible in a release build (devtools disabled), while the collectionSets store was already updated, so the UI shows the collection/star as saved.

**Scenario.** User stars 20 notes and creates a new collection (createCollection updates the store THEN calls saveCollections). The collections JSON write fails (universe root on a disconnected/locked drive, permissions). .catch(e => console.error(...)) logs to a console the user cannot see in release; no banner, no retry (unlike note saves). The sidebar shows the collection and stars all session — false success. On next launch the collections file still holds the old state: the collection and all 20 memberships are silently gone. Collections is an explicitly listed persisted-JSON source of truth.

### [MED] `src/lib/libraries/store.ts:5622` (frontend-write-callers) — false-success

**#33. Summary.** saveSettings() debounces 300ms then fires invoke('save_universe_settings') with a console-only .catch (same pattern at line 5788 for save_universe_workspaces) — a failed persist of universe settings is silent in release while the in-memory appSettings already applied, and the 300ms timer has no app-close flush.

**Scenario.** User reworks their setup — theme/style overrides via the Style Setter (mergeStyleOverride → saveSettings), fonts, panel placements, feature flags. Every change applies live (appSettings.update runs first), so everything looks committed. If invoke('save_universe_settings') rejects (settings.json locked by sync tooling, disk full), the only trace is console.error — invisible with devtools disabled in release. All subsequent saveSettings calls keep failing silently for the session; on restart the universe settings revert wholesale to the pre-session state. Narrow second window: change a setting and quit within 300ms — the debounce timer dies with the webview; nothing flushes it at close (the PJ-103 close flush covers note models only).

### [MED] `src-tauri/src/bases.rs:448` (frontmatter-property-writes) — content-corruption

**#34. Summary.** update_frontmatter_property's continuation-line skip loop breaks on a block-list item's indented continuation field (seq-of-maps) and never matches tab-indented items, leaving orphaned indented lines under the new scalar value — invalid YAML on disk that then arms the yamlDoc H1 passthrough (finding 1) for every future edit of that note.

**Scenario.** A Lens/Base table exposes a `prop.ikhtilāf`-style column (any key whose on-disk value is a seq-of-maps: `ikhtilāf:\n  - school: Hanafī\n    position: permissible`). User edits that cell → updateNoteProperty → update_note_property → the replace loop pushes `ikhtilāf: <new>`, skips `  - school: Hanafī` (matches "  - "), then hits `    position: permissible` — starts_with("  ") is true but trim doesn't start with "- " → break — so the orphan `    position: permissible` line survives directly under the new scalar. gate_rmw commits it; Ok(()) returned; the table refreshes normally. The note's frontmatter is now strict-parse-invalid, so from this moment composeFrontmatter's H1 passthrough silently drops every subsequent property edit on the note (the APP-KILLER above), and the orphaned field data is invisible to every parser. Tab-indented list items (`\t- a`) aren't skipped at all, leaving stale duplicate items under the replaced key.

### [MED] `src/lib/libraries/store.ts:1674` (frontmatter-property-writes) — content-corruption

**#35. Summary.** parseFrontmatter's inline flow-list handling (naive split(',') + bracket slice) corrupts quoted items containing commas and eats a bracket pair off scalar wikilink values; the corrupted projection is serialized to disk the moment that key is next edited through any props path.

**Scenario.** (a) Note has `aliases: ["Doe, John", "JD"]` — the projection splits on the embedded comma into three phantom items Doe / John / JD. User adds one alias chip in the PropertyEditor → the diff fires on the aliases key → serializeLine writes the block list `- Doe`, `- John`, `- JD`, `- new` — the alias "Doe, John" is permanently destroyed on disk and note_aliases lookups for it break at the next reindex, with no error. (b) Note has an unquoted inline wikilink `supports: [[Note A]]` — startsWith('[')/endsWith(']') treats it as a flow list, slice(1,-1) yields item `[Note A]`. Reviewer connects another 'supports' typed link → addTypedLinkToProps appends to the corrupted list → disk gets `- '[Note A]'` (no longer a wikilink) — the existing note_links edge to Note A silently disappears at reindex. Multi-wikilink lines (`[[A]], [[B]]`) shred even worse (`[A]]` / `[[B`).

### [MED] `src/lib/components/ExpressionForge.svelte:144` (note-save-index) — index-divergence

**#36. Summary.** Expression Forge export: identical shape — createNote() indexes an empty skeleton, then writeNote(newPath, content, 'expression_forge') writes the real forged markdown with no reindex; the export's content never enters note_meta/FTS.

**Scenario.** User forges an expression from selected Sky notes and exports it. The created file is indexed empty, then silently overwritten (gated, watcher-suppressed) with the full markdown including wikilinks to the source notes. Those links never appear in note_links/backlinks/sky, the text is invisible to search, and no boot pass ever re-reads it (onlyIfUnindexed boot walk + path-only reconcile). The dialog closes with no error — a write-once export is exactly the note the user never edits again, so the divergence is permanent.

### [MED] `src/lib/libraries/store.ts:482` (note-save-index) — index-divergence

**#37. Summary.** saveRecoveredCopy writes a brand-new note via writeNote('recovered_copy') — not create_note — so it is never indexed: the gated write is watcher-suppressed, the 'note-created' listener (+layout.svelte:3321) only refreshes the file tree (no reindex, unlike library-changed which feeds pendingReindex), and openNoteTab does not reindex.

**Scenario.** A note's file stays locked; the user clicks 'Save a copy'. The copy — the ONLY durable holder of their rescued unsaved work — gets no note_meta row for the entire session: invisible to search, Sky, pickers, backlinks, and to the MIG-099 index-backed title-collision check (which trusts an index miss as 'does not exist', so a same-title create is silently allowed). Healed only at the NEXT universe open (reconcile step 9 re-adopts orphan files), so the divergence is session-long and completely silent — the copy opens fine in its tab.

### [MED] `src/lib/libraries/store.ts:1093` (note-save-index) — fire-and-forget

**#38. Summary.** addLinkToNote (CLOSED-note branch): after the frontmatter typed-link write, reindexNote is deliberately fire-and-forget; its own comment claims an interrupted reindex is caught 'on the next boot's reindex' — but no such boot reindex exists (boot walk is onlyIfUnindexed:true; reconcile never refreshes an existing path), so an interrupted/failed reindex leaves the typed link underived forever.

**Scenario.** User connects a typed link (Reviewer 'connect') to a closed link-dense note — the code comment itself notes this reindex can take multi-seconds (PJ-066 per-edge cost) — then quits the app. flushAllForAppClose awaits reindexes only for dirty OPEN tabs, so this in-flight reindex is killed with the process (SQLite tx rolled back). Disk frontmatter carries the link; note_links, the target's incoming_count, and sky never learn of it. console.error is the only surface (invisible in release, devtools disabled). Nothing ever re-reads the note: permanent silent link-index divergence.

### [MED] `src/lib/libraries/store.ts:2101` (notemodel-ownership) — toctou

**#39. Summary.** openNoteTab's B1 dedup check (get(openTabs).find by path) runs BEFORE the awaits (resolveNoteContent at 2121, ensure_cid_cn_cmd at 2140), but the tab+model creation happens after them — two concurrent openNoteTab calls for the same path with newTab=true both pass the check before either inserts, creating two tabs + two models for one path (same clobber class as B1).

**Scenario.** Double Ctrl+click on a search result / file-tree row (a double-click fires two click events ~50-100ms apart): call 1 passes the dedup check and parks on the resolveNoteContent IPC; call 2 enters, finds no tab on that path yet, also proceeds. Both create tabs and models (lines 2258-2266). Edits typed in tab 1 autosave to disk; tab 2's stale clean model later composes its old body on the user's first keystroke there and silently reverts tab 1's edits on disk — no error, watcher suppressed for our own write.

### [MED] `src/lib/libraries/store.ts:1387` (notemodel-ownership) — concurrency-race

**#40. Summary.** In loadTabHistoryEntry the departure flush (1357) completes BEFORE the awaited read_note IPC (1361); keystrokes typed into the still-mounted outgoing editor during that read window land on the model (editBody, path still matches → accepted, version++), and openNoteModel at 1387 then replaces the now-DIRTY model on a different path — discarding them with only a dev-only console.warn (noteModel.ts:135 tripwire is import.meta.env.DEV). openNoteTab avoids exactly this by reading content BEFORE its flush with no await between flush and re-seed (2212→2246); the history-nav path has the ordering backwards.

**Scenario.** User is typing rapidly and hits Alt+Left mid-burst: flushOutgoing flushes what exists (or no-ops if clean), then read_note parks on IPC for ~5-50ms (longer for a large history note / cold disk); two more keystrokes land in the outgoing editor during that window → model dirty; read_note resolves, token check passes, openNoteModel replaces the dirty model — those keystrokes exist nowhere (no net: setNet only runs inside a save). Release build: zero signal.

### [MED] `src/lib/libraries/store.ts:2209` (notemodel-ownership) — concurrency-race

**#41. Summary.** openNoteTab's in-place reuse bumps the shared _navTokens supersede token ONLY inside the isNoteDirty guard (2209-2214) — a CLEAN-tab click-nav performs no bump, so it cannot supersede an in-flight loadTabHistoryEntry, contradicting the comment 'Shares _navTokens ... so a click-nav and an Alt-nav on this tab supersede each other'. The in-flight Alt-nav's token check (1363) still passes and stomps the just-applied click-nav, replacing its model.

**Scenario.** User hits Alt+Left (loadTabHistoryEntry sets token, parks on read_note for a large history note), then immediately clicks note X in the tree: currentTab is clean → no token bump → tab + model synchronously re-seeded to X; user starts typing into X. The slow read_note resolves, finds the token unchanged, and applies the HISTORY note over X — openNoteModel replaces X's now-dirty model on a different path: the keystrokes typed into X are silently discarded (dev-only warn) and the tab flips to a note the user did not choose.

### [MED] `src/lib/libraries/store.ts:1218` (notemodel-ownership) — false-success

**#42. Summary.** saveTabContent's single-flight guard ('if (saveLocks.get(tabId)) return') pushes the concurrent property edit into the model (dirty) but resolves the promise successfully having SKIPPED the write, and NOTHING reschedules it: no timer watches dirty-but-never-failed models (the ~10s auto-retry keys off saveHealth failures only), and a prop-only edit produces no editor debounce. The comment's 'the next save/flush persists it' has no guaranteed next trigger until a departure.

**Scenario.** User makes two PropertyEditor edits in quick succession (e.g. toggles a checkbox then fixes a date while the first prop_save's writeNote IPC is in flight): the second edit hits the lock and returns; the first write completes and markSaved leaves the model dirty (correct), but no retry ever fires. Disk + FTS index stay stale for the rest of the sitting (screen shows the edit — silent divergence, stale search results); setNet never ran for the skipped save, so a hard crash before the next nav/close/edit loses the edit with no net and no signal.

### [MED] `src-tauri/src/libraries.rs:198` (persisted-json-state) — silent-data-loss

**#43. Summary.** save_libraries does tmp-write + rename WITHOUT fsync before the rename — violating the codebase's own documented requirement (universe.rs atomic_write, lines 121-125: 'power loss can land the rename while the data blocks are still unflushed, leaving a zero-length/garbage file under the FINAL name').

**Scenario.** User adds/removes/renames a library; save_libraries renames the un-fsynced tmp over libraries.json; power loss within the OS flush window commits the rename metadata but not the data blocks -> libraries.json is zero-length/garbage under its final name. Next boot: load_libraries backs it up and returns empty (eprintln only — invisible in release), then ensure_universe_notes_folder's flat branch rewrites a one-entry file (compounding with the universe.rs:384 finding), so all library registrations silently disappear with no surfaced error. The G6 W1-8 comment claims crash-safety this code does not actually provide.

### [MED] `src-tauri/src/link_types.rs:535` (persisted-json-state) — silent-data-loss

**#44. Summary.** save_universe_link_types persists link-types.json with plain non-atomic fs::write, and read_deltas (lines 510-512) swallows read AND parse errors to an empty delta list — the 'property-types.json pattern' sibling that missed the G6 atomic_write hardening.

**Scenario.** Crash/power loss mid-write leaves link-types.json partial; next boot read_deltas returns [] ('absent/corrupt => the 8 seeds' by design), so every user-created custom link type — first-class cognitive-vocabulary data — silently vanishes from the registry, pickers, and SQL generators. The next save from the UI (e.g. any recolour) writes the seeds-only state back, making the loss permanent. Same clobber also fires without a crash on a transient read lock: read empty -> UI shows seeds -> user edits a colour -> save overwrites the intact file with seeds-only deltas. No error surfaced anywhere.

### [MED] `src-tauri/src/review.rs:762` (persisted-json-state) — silent-data-loss

**#45. Summary.** save_pulse_data persists review-pulse.json with a plain non-atomic fs::write (truncate-then-write) — the exact crash-window shape Safety Audit G6 fixed for settings/workspaces/collections/property-types via atomic_write was never applied here.

**Scenario.** review-pulse.json is rewritten in full on every Reviewed/Snooze/Dismiss action. App crash, power loss, or kill mid-write leaves the file truncated/partial. On next launch load_pulse_data (lines 747-757) swallows the parse error and returns default() — the entire review history reads as empty — and the user's next review action persists that empty state permanently via the RMW. No error is ever surfaced; the Reviewer just silently treats every note as never-reviewed.

### [MED] `src-tauri/src/review.rs:1056` (persisted-json-state) — swallowed-write-error

**#46. Summary.** sync_action_to_row discards the result of the review_schedule row update (`let _ = f(conn)`), so a failed DB write after a successful pulse-JSON write leaves the review_schedule index silently diverged from its JSON source of truth — and the divergence is self-perpetuating.

**Scenario.** mark_reviewed writes review-pulse.json OK, then review_row_mark hits a transient rusqlite error (e.g. SQLITE_BUSY/locked during a backfill) — swallowed. The schedule row keeps the OLD last_reviewed/due_days, so the note the user just reviewed stays 'due' in the Reviewer and Collections chips. It never heals: the write-time re-index upsert (line ~1001) deliberately preserves last_reviewed/interval FROM THE EXISTING ROW rather than re-reading the JSON, so the stale row survives every subsequent save/re-index. Same silent skip when the DB lock is poisoned or conn is None mid-action.

### [MED] `src-tauri/src/universe.rs:101` (persisted-json-state) — silent-data-loss

**#47. Summary.** load_registry swallows read errors and corrupt JSON on the global universes registry to an EMPTY registry (eprintln only, no backup — unlike load_libraries' G6 backup), and registry-mutating flows then save over it: create_universe (line 647-652) load->push->save writes a registry containing ONLY the new entry.

**Scenario.** The appdata registry file is momentarily unreadable (AV/indexer lock) or partial exactly when the user creates or links a universe: load_registry returns empty, the new entry is pushed, save_registry atomically commits a 1-entry registry. Every previously registered universe silently disappears from the universe picker; command returns Ok. Universe directories on disk are intact but the app has forgotten them — the MIG-098-class silence (recoverable only by manually re-linking each universe, which the user has no reason to suspect).

### [MED] `src-tauri/src/universe.rs:117` (reactivity-concurrency) — toctou

**#48. Summary.** universe.rs::atomic_write derives ONE FIXED temp name per target (`settings.json.tmp`, `collections.json.tmp`, ...) — unlike write_gate.rs:239-243 which suffixes PID+TMP_COUNTER — so two concurrent invocations of the same #[tauri::command(async)] save (each spawned independently on the async pool, completion order NOT FIFO) collide on the same tmp path: writer B's File::create truncates writer A's fully-fsynced tmp mid-flight, A's rename can commit a partial/foreign snapshot or fail, and an OLDER payload can rename last (silent lost-update). saveCollections (store.ts:1412) fires one unserialized invoke per mutation and saveSettings' 300ms-debounced saves can overlap their own multi-second fsyncs (the file's own comments: '100ms-seconds on network/USB/AV-scanned disks'), so overlap is reachable.

**Scenario.** User bulk-adds notes to a collection (N rapid mutations -> N concurrent save_universe_collections invokes, each snapshotting collectionSets at call time). The pool executes save k+1 fully, then save k: k truncates the shared collections.json.tmp, writes the OLDER list, renames -> disk now lacks the last additions while the UI shows them; the only failure surface on any losing writer is console.error (invisible in a release build, devtools disabled). Worse: a crash/AV-lock in the create->rename window, or an interleaved partial commit, leaves collections.json unparseable -> next boot loadCollections' catch{} silently seeds an EMPTY list (store.ts:1524) and the first star/mutation persists that near-empty list over the corrupt file — the user's Starred + all working sets permanently gone with zero error. Same shared-tmp exposure for settings.json / property-types.json / workspaces.json. Fix: unique tmp names (the write_gate pattern) + serialize per-target.

### [MED] `src/lib/libraries/store.ts:790` (rename-cascade-integrity) — index-divergence

**#49. Summary.** reloadTabsFromDisk matches rewritten paths to open tabs by raw string equality (t.path === fp at :790 and byPath.get(t.path) at :810) with NO separator or Unicode-NFC folding — while every neighboring seam normalizes precisely because form divergence between JS tab paths and Rust walker paths is a documented live surface: cascadingPaths folds separators ('a Windows tab path that travels through the JS layer with mixed separators', store.ts:700-702), the PJ-092 belt folds normPathLC+NFC because 'a leak that reaches here differs by NFC/NFD, the exact Arabic-root form' (+layout 6353-6357), and Rust needed path_identity_key for the same reason (libraries.rs:5481-5500). A rewritten tab whose path form differs is silently never reloaded.

**Scenario.** Tab for backlinker C holds a JS-constructed mixed-separator or NFD-form path (the forms the codebase's own comments attest exist, e.g. under the Arabic universe root) while update_links_on_rename returns the walker's backslash/NFC form for the same file. reloadTabsFromDisk's filter finds no tab match -> C's tab and model silently keep the pre-cascade body ([[old]]). The cascade's own writes are watcher-suppressed, so no library-changed adopt corrects it; clearCascading lifts the gate; the user's next keystroke in C triggers the 1500ms autosave which composes the stale body and writes [[old]] back over the rewrite. Disk, index, and screen all agree on the reverted state — the cascade result reported to the user is silently undone with no error. The H3 focus check (+layout 6375) shares a milder form of the gap (normPathLC without .normalize('NFC')).

### [MED] `src/routes/+layout.svelte:6336` (rename-cascade-integrity) — concurrency-race

**#50. Summary.** The cascade freeze and write-gate are a START-OF-CASCADE SNAPSHOT, not a window: cascadeFreeze (6313) and the markCascading loop (6336-6337) capture tabsInLibrary once; a tab opened DURING the multi-second cascade (Ctrl+O quick switcher / tree click on an unfrozen surface — the overlay only blocks pointer input on already-open panes) is neither frozen nor save-gated, and reloadTabsFromDisk will still force-adopt it if its path is in result.rewritten.

**Scenario.** Rename of A starts its ~7s cascade. User opens backlinker D via the quick switcher (nothing gates openNoteTab); D's tab reads pre-rewrite disk. User types into D — no overlay (D's path is not in the cascadeFreeze snapshot), no isCascading gate (D was never markCascading'd), so the 1500ms autosave can land after the walker rewrote D (silently reverting the [[link]] rewrite), or, if the autosave hasn't fired, reloadTabsFromDisk force-adopts disk into D's dirty model (D is in rewritten and its tab exists at reload time) and the {#key} remount discards the just-typed keystrokes. Either arm is silent: no error, no conflict sidecar.

### [MED] `src-tauri/src/libraries.rs:1713` (rename-move-delete-gate) — toctou

**#51. Summary.** move_item checks dest.exists() outside any lock, then calls gate_rename which has no dest-exists check under the lock — a file created at the destination between the check and the rename is silently replaced (Windows fs::rename = MOVEFILE_REPLACE_EXISTING), destroying the newly created note with Ok returned to both callers.

**Scenario.** move_item(async, pool thread) validates dest.exists()==false at libraries.rs:1713 for target 'X/Foo.md'. Concurrently a create surface (create_note via template/quick-capture, or a watcher adopt) creates 'X/Foo.md' — gate_create_exclusive takes the dest path lock, writes, releases, returns Ok. move_item's gate_rename (libraries.rs:1717) then acquires the dest lock and fs::rename (write_gate.rs:587) replaces the just-created note with the moved file. The created note's content (already indexed by create_note's synchronous reindex) is gone from disk; note_meta still carries its row until the moved note's reindex overwrites it — both operations report success, nothing surfaces. The folder-rename branch is safe only by accident (Windows rename fails onto an existing directory); the file case silently clobbers. Root fix locus: gate_rename (write_gate.rs:566) needs the same under-lock RefusedExists dest check gate_rmw_rename performs at write_gate.rs:684.

### [MED] `src-tauri/src/libraries.rs:1117` (rename-move-delete-gate) — index-divergence

**#52. Summary.** rename_item_db_tail (and create_note at :832, resolve_structural_conflict at :1683) resolve the note's library with first-match `.find(|l| path.starts_with(&l.path))` instead of the longest-match resolver library_name_for_path (:167) — and the universe_notes library (path == universe ROOT) is always FIRST in the list (universe.rs:359/390 insert(0)), so every note in a sub-folder library reindexes with the ROOT library's name, silently flipping note_meta.library_name.

**Scenario.** A universe at E:\U auto-registers universe_notes with path E:\U at index 0 of libraries.json; a second library 'Research' lives at E:\U\Research. User renames E:\U\Research\note.md. rename_item_db_tail Step 6 (libraries.rs:1117) iterates load_all_libraries in order; E:\U\Research\note2.md starts_with 'E:\U' → first match is universe_notes → reindex_single_note runs with library_name = the universe's display name. index_note's UPSERT (search.rs: ON CONFLICT(path) DO UPDATE SET library_name = excluded.library_name) overwrites the row's correct 'Research' attribution. From that moment the note counts under the root library in get_all_library_stats, vanishes from every library_name-scoped surface for 'Research' (scoped search, tag scans, per-library counts) — no error anywhere, and reconcile does not audit library_name of existing rows. Heals only if the note is later saved from an open tab (frontend passes tab.libraryName) or externally edited (watcher flush uses the correct library_name_for_path). create_note (:832) mis-attributes every new note in a sub-library the same way at creation; the raw starts_with also lacks the separator bound ('E:/U/Research' matches files under 'E:/U/Research Notes'), which library_name_for_path explicitly guards.


## LOW findings

### [LOW] `src-tauri/src/bases.rs:404` (boot-init-ordering) — index-divergence

**#53. Summary.** update_note_property (Base table cell edit) follows its gate_rmw disk write with `let _ = reindex_single_note(...)` and no ensure_search_db_ready — when state.db is None the reindex is a silent double no-op (swallowed AND Ok-on-None), a new unregistered site of the PJ-093 class.

**Scenario.** A Base tab is interactive when a universe switch begins (invalidate_search_state nulls state.db at search.rs:8421-8423 before the new init publishes) or a lock-window during cold re-init: the user edits a cell → gate_rmw rewrites the note's frontmatter on disk (source of truth OK) → reindex_single_note returns Ok(()) at search.rs:9289/9358 without touching note_meta, and the `let _ =` at bases.rs:404 would swallow even a real error. The gated write mark()s watcher suppression (watcher.rs echo-window, register #23), so Watcher-Index-Freshness never re-indexes it; the boot reconcile only relocates/removes dead-path rows (reconcile.rs), so note_meta/notes_fts keep the pre-edit property value until the note is next saved in the editor. Base re-queries (which read note_meta, per the MIG-065 §H comment at bases.rs:396-400), Dataview-style lenses, and property search silently show/serve the stale value — no error anywhere. Narrower reachability than the arabic-overrides candidate (needs an interactive Base view straddling the None window), reported as a new site to fold into PJ-093's fix (ensure-first + surfaced error, matching create_note's non-swallowed shape at libraries.rs:828-846).

### [LOW] `src/routes/+layout.svelte:1563` (cross-window-integrity) — index-divergence

**#54. Summary.** commitFocusSave gates broadcastNoteSaved on `secondScreenOpen` (as does the task-toggle at line 8591), but close_second_screen only HIDES the SS webview (lib.rs:266) — its listeners, tabs and models stay alive — so focus-mode edits and task toggles made while the SS is closed never reach it, and on re-show the SS's own tabs/peek display the pre-edit body; NoteEditor's broadcasts (NoteEditor.svelte:228/266/351) are ungated, making these two write paths the inconsistent silent gap.

**Scenario.** Note X is open in an SS-local tab. Boss closes the SS (window hides; secondScreenOpen=false). In main, Boss enters Focus mode on X and writes three paragraphs -> commitFocusSave persists to disk but skips the broadcast because secondScreenOpen is false. Boss reopens the SS -> the same webview re-shows with its retained tab for X; nothing re-reads it (onMount does not re-run on show; the reopen's editorPanels emit only refreshes the cockpit, not SS-local tabs). The SS silently shows X without the three paragraphs until the next NotePane save of X broadcasts (ungated) — the two windows show different bodies for the same note with no indication. Same for a task ticked via the right-sidebar TasksPanel while the SS was hidden.

### [LOW] `src-tauri/src/search.rs:9223` (derived-index-triggers) — swallowed-write-error

**#55. Summary.** reindex_delete_note swallows the two source-of-truth index deletions with `let _ = conn.execute(DELETE FROM note_links / note_meta …)` and unconditionally returns Ok(()), so a failed de-index reports success to every caller (note delete flow, watcher vanish-path, reconcile.rs run() which then counts it 'removed').

**Scenario.** A note is deleted (or reconcile removes a truly-gone row) while the DELETE hits a transient SQLITE_BUSY/I-O error (a concurrent walk_conn/backfill transaction on the same DB past busy_timeout, or a disk-full WAL append). Both `let _ =` at search.rs:9223-9224 discard the failure; the function proceeds (tag_counts delta applied against a row that still exists, note_body deleted at 9235) and returns Ok. The phantom note_meta row keeps its notes_fts entry (the ad-trigger never fired), so Quick Switcher/search/backlinks keep serving a note whose .md is gone — clicking it opens an empty/dead path — while reconcile.rs:226-228 logs it healed ('removed += 1'). Recovery only if the once-per-universe-open phantom reconcile retries on a later boot AND the failure was transient; within the session the divergence is fully silent.

### [LOW] `src/lib/editor/livePreview.ts:261` (freeze-and-leaks) — resource-leak

**#56. Summary.** Module-level `_embedCache = new Map<string, EmbedResolution>()` grows unbounded and is never cleared or evicted — the sibling of the already-registered `_imageCache` (line 242), but distinct: its values include transcluded `note_body` strings, so it retains full note bodies.

**Scenario.** Over a long session the user browses many notes containing `![[note]]` / `![[file]]` embeds. Each distinct `libraryPath|notePath|target` key inserts an EmbedResolution (including note_body transclusion text) into `_embedCache` and it is never deleted — no size cap, no eviction on tab close or universe switch (only `_transcludeStack` is per-render-cleaned; `typeDecoCache`/`_imageCache` have their own paths). Memory grows monotonically with the number of unique embeds viewed (Rule 4 unbounded-cache violation). Slow-death leak, no error surfaced. Distinct from the registered _imageCache:242, same file/class.

### [LOW] `src/lib/components/StyleSetter.svelte:936` (frontend-write-callers) — false-success

**#57. Summary.** saveAsStyle/confirmRename/removeStyle/updateStyle (lines 936/959/963/975) update savedStyles in the UI first, then await saveStylePresets with no catch — a rejected save_style_presets becomes an unhandled promise rejection, invisible in release, while the preset list on screen shows the change as saved.

**Scenario.** User clicks '+ Save current as a style' after tuning a look. savedStyles is appended BEFORE the write; if invoke('save_style_presets') rejects (presets file locked/permissions), the rejection escapes the event handler unhandled — console-only, invisible with devtools disabled. The Setter lists the new style all session (false success); after restart the preset is gone. Same for rename/delete/update-in-place of existing styles: the on-screen CRUD result silently diverges from the persisted presets JSON.

### [LOW] `src/lib/libraries/store.ts:2361` (note-save-index) — index-divergence

**#58. Summary.** flushAllForAppClose awaits FTS reindex only for the `dirty` set captured BEFORE the first flush pass; a tab that was clean at close-start but dirtied during the interactive ≤5s close hold is flushed by the final_flush_repass but is NOT in `dirty`, so its reindex is never issued — disk newer than index across the restart, the exact gap this awaited-reindex block was added to close.

**Scenario.** User clicks X, then types a few words into a tab that was clean at that moment (the window stays interactive during Rust's close hold). Pass 1 sees it clean; the keystrokes land; the repass flushes it durably to disk — but the awaited reindex loop iterates only the close-start dirty list, and the process exits. Watcher-suppressed write + no boot re-walk + path-only reconcile: those final words are on disk but permanently missing from note_meta/FTS, with no error and no journal marker (the note is not residual-dirty).

### [LOW] `src/lib/libraries/store.ts:1201` (notemodel-ownership) — false-success

**#59. Summary.** saveTabContent returns void-success when isCascading(filePath) — the prop edit is dropped BEFORE editNoteProps pushes it to the model (unlike the saveLocks skip at 1218 which at least lands it in the model), so a PropertyEditor save issued during a cascade window on that path vanishes entirely: not in the model, not on disk, promise resolved, no error.

**Scenario.** A rename/task-toggle/structural-resolve marks the note cascading (refcounted, spans awaited IPC + reload — can be seconds on a slow disk or big cascade). The user edits a property in that window: saveTabContent returns silently; the subsequent reloadTabsFromDisk re-seeds model + PropertyEditor from disk, erasing the edit from screen too. Unless the user happens to re-edit properties later (each save passes the full array), the edit is gone with zero signal. Narrow window, but a genuine conditional-skip false-success on a source-of-truth write.

### [LOW] `src-tauri/src/universe.rs:1589` (persisted-json-state) — toctou

**#60. Summary.** The one-time workbench.json -> collections.json adoption writes the tmp without fsync before rename, and retires the legacy file (rename to .migrated at line 1591) based on the in-process rename Ok — before the adopted data is validated or durable.

**Scenario.** Power loss right after the adopt commits in-process: rename metadata lands but data blocks don't -> collections.json is garbage under its final name AND workbench.json was already renamed to .migrated, so the '!path.exists()' re-adoption gate never fires again. Additionally, if the legacy data is itself corrupt, the code writes it into collections.json and retires the legacy BEFORE the parse at line 1598 fails. Partial mitigations: subsequent reads return Err (surfaced, not fully silent) and the .migrated backup survives — hence LOW.

### [LOW] `src/lib/libraries/store.ts:5621` (reactivity-concurrency) — silent-data-loss

**#61. Summary.** The PJ-103 graceful-close final-flush handler (+layout.svelte:2885-2893) persists the session snapshot and flushes dirty NOTE models, but never flushes the pending 300ms saveSettings debounce timer — a settings change made <300ms before close leaves save_universe_settings scheduled in a setTimeout that dies with the webview, so the change silently reverts on next boot.

**Scenario.** User toggles a setting (e.g. restoreTabsOnRelaunch, a Style-Setter override commit, a per-note toggle stored in appSettings) and immediately closes the app. updateSettings() updated the in-memory store and scheduled the disk write 300ms out; Rust's CloseRequested hold fires session:final-flush, which awaits persistSessionNow() + flushAllForAppClose() and acks — the settings timer never fires, the webview is destroyed, settings.json still holds the OLD value. Next boot the toggle is silently back to its previous state with no error anywhere (the user just watched the UI confirm the change). Fix: in the session:final-flush listener, cancel saveSettingsTimer and await one direct save_universe_settings invoke when a save is pending (mirror persistSessionNow's cancel-and-flush shape).

### [LOW] `src/routes/+layout.svelte:6304` (rename-cascade-integrity) — index-divergence

**#62. Summary.** handleRenameComplete binds the library with an unbounded prefix match — $libraryStats.find(v => oldPath.startsWith(v.path)) (also 6231, 6447) — lacking the separator-boundary guard that libraryIdForPath (6459-6467) was written to provide against exactly this bug ('a library named Research would steal notes living in a sibling folder named Research Notes'). With sibling non-nested libraries sharing a name prefix, the cascade can bind lib to the WRONG root: cascadeFreeze/flushAllTabsInLibrary/the walker all run against a library that contains neither the note nor its backlinkers, and the real library's [[links]] are silently never rewritten.

**Scenario.** Two registered libraries E:/Foo and E:/Foo Bar (non-nested siblings), with E:/Foo ordered first in $libraryStats. User renames note A living in E:/Foo Bar. find() returns E:/Foo (startsWith matches the prefix without a separator check). The rename itself succeeds, then updateLinksOnRename walks E:/Foo — zero matches — result.rewritten=[] , no toast anomaly, no error. Every [[A-old-title]] in E:/Foo Bar stays stale on disk and in note_links; open backlinker tabs in E:/Foo Bar were never frozen, flushed, or reloaded. Only the rename-stamped alias keeps the links resolving, masking the divergence indefinitely.

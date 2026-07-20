# PJ-130 — Safety Backlog Remediation Plan

**Boss ruling 2026-07-20: "Clear the safety backlog first, PJ-130."**

Source: whole-app safety inspection `wf_5ff846df-00c` (37 confirmed findings), re-triaged against
HEAD after MIG-101 Phase A landed by `wf_78cd0b34-e3f`. Raw findings: `lab/reports/pj130_findings.json`.

**Re-triage outcome: 37 reported → 30 genuinely distinct live defects.**
4 already fixed by Phase A (1·9·19 are one defect — the read-only shape guard; 3 is the
nested-object-list serializer). 3 are double-reported (6=20, 7=34, 8=10).
**1 APP-KILLER remains** (finding 2 — the second screen can write).

Every "STILL_LIVE" verdict below was established by reading the current source, not the inspection's
line numbers, which had shifted.

---

## 1. The headline

**37 findings → 30 genuinely distinct live defects. Four are already fixed, three more are the same bug reported twice by different hunt groups.**

- **Already fixed** (MIG-101 Phase A, 2026-07-20): findings 1, 9, 19 (all one defect — the read-only shape guard) and finding 3 (the nested-object-list serializer). No work needed; verified in current source.
- **Double-reported**: 6 = 20, 7 = 34, 8 = 10. Three commits' worth of work, not six.
- **Genuinely live: 30 defects.** One is an APP-KILLER (finding 2). Sixteen are HIGH. The rest are MED/LOW.

Honest shape of the remaining risk — it clusters into five families, and that is why 30 findings is far less work than it sounds:

| Family | Findings | What it means in plain language |
|---|---|---|
| **Cascade / rename content-integrity** | 8·10, 11, 12, 13 | If you rename a note while typing in it, your typing can be silently thrown away. The worst family. |
| **Database rows left pointing at dead files** | 4, 5, 23, 28 | Move or rename a *folder*, and the search index, review history and aliases keep the old paths. Nothing crashes; things just quietly stop being findable. |
| **Files written without a power-loss-safe sequence** | 17, 29, 30, 31 | Review history, style presets, the library registry and settings can end up empty after a hard crash, and the app reads "empty" as "you have none." |
| **A write happens but the index isn't told** | 6·20, 7·34, 24, 25, 35 | The file on disk is right; search, backlinks and the second screen disagree with it. |
| **Everything else** | 2, 14, 15, 16, 18, 21, 22, 26, 27, 32, 33, 36 | Assorted — including the one APP-KILLER. |

Only **two** batches touch the note save/compose path (the code that runs on every keystroke-triggered save). Those are called out explicitly below and are deliberately not first.

---

## 2. Batches

Ordered by *risk removed per unit of risk introduced* — cheap, contained, high-value first.

### Batch 1 — Second screen must never write (APP-KILLER)
- **Findings**: 2
- **Root cause**: the read-only second-screen window opens notes in a state the app considers "has unsaved work," so closing or navigating there can write over the real note.
- **Files**: `src/lib/libraries/store.ts` (`flushOutgoing` entry, plus the two `markModelRecoveredFromNet` sites)
- **Test asserts**: with a note dirty in the main window and the same note open on the second screen, closing the second-screen tab produces **zero** disk writes to that note, and the main window's unsaved work survives.
- **Owner clicks**: Open a note in the main window, type a sentence, don't wait for the save. Open the same note on the second screen. Close it there. Return to the main window — your sentence is still there, and the note on disk still has whatever it had before.
- **Why first**: one function, one early return, kills the only APP-KILLER in the list.

### Batch 2 — Files that survive a power cut
- **Findings**: 17, 29, 31
- **Root cause**: three places write important files by truncating and rewriting instead of using the project's existing safe-write helper.
- **Files**: `src-tauri/src/review.rs`, `src-tauri/src/style_presets.rs`, `src-tauri/src/libraries.rs` (`save_libraries`) — all routed through the existing `universe::atomic_write`.
- **Test asserts**: a truncated/corrupt file is moved aside rather than silently read as empty; the saved file is fully on disk before it replaces the old one.
- **Owner clicks**: Mark a few notes reviewed, save a style preset, then force-quit the app from Task Manager. Reopen: your review history, your presets and your library list are all intact.

### Batch 3 — Settings changes survive closing the app
- **Findings**: 30
- **Root cause**: settings and property-type changes wait up to half a second before saving; closing the app inside that window loses them.
- **Files**: `src/lib/libraries/store.ts`, `src/lib/libraries/propertyTypeRegistry.ts`, `src/routes/+layout.svelte`, `SettingsModal.svelte`
- **Test asserts**: a settings change followed immediately by app close is present on next launch.
- **Owner clicks**: Open Settings, flip a toggle, close the app within a second. Reopen — the toggle is still flipped.

### Batch 4 — Two freezes
- **Findings**: 21, 22
- **Root cause**: two background scans run on the thread that draws the window; three sibling functions were already fixed the same way and these two were missed.
- **Files**: `src-tauri/src/provenance.rs`, `src-tauri/src/libraries.rs`
- **Test asserts**: both commands run off the UI thread; the title-reading helper reads a bounded prefix, not whole files.
- **Owner clicks**: With the large library loaded, open the second screen. The main window stays responsive throughout — no multi-second stall.

### Batch 5 — Writes that forget to tell the index
- **Findings**: 24, 25, 35, and 7·34
- **Root cause**: several write paths skip the "now re-index this note" step; separately, the re-index command silently reports success when the index isn't loaded.
- **Files**: `src/lib/libraries/store.ts`, `ExpressionForge.svelte`, `SenseMakingCanvas.svelte`, `src-tauri/src/search.rs`
- **Test asserts**: after each of those writes, searching for text unique to the new content finds the note; a re-index against an unloaded index returns an error rather than success.
- **Owner clicks**: Export a composition from Expression Forge, then search for a distinctive phrase from its body. It's found. (Today it isn't, and never becomes findable.)
- **Note**: finding 34's suggested change to how *every* caller handles re-index failure is deliberately **not** in this batch — see §4.

### Batch 6 — Renaming or moving a folder must carry its records with it
- **Findings**: 4, 5, 23, 28
- **Root cause**: renaming a note migrates its search row, aliases, review schedule and shape history. Renaming or moving a *folder*, and deleting one, does not — and the self-healing watcher is deliberately suppressed on those paths.
- **Files**: `src-tauri/src/libraries.rs` (rename/move/delete), `src-tauri/src/search.rs`, `src-tauri/src/reconcile.rs`. Extract one shared `migrate_note_aux_rows(conn, old, new)` used by all three.
- **Test asserts**: after a folder rename, every descendant note is still findable at its new path, still has its review history, and no search result points at a path that no longer exists. After a folder delete, no descendant rows remain.
- **Owner clicks**: Put three reviewed notes in a folder. Rename the folder. Search for one of them — found, at the new location, with its review history intact. Then delete the folder and search again — nothing stale comes back.

### Batch 7 — The `sources` rewriter stops eating other people's data
- **Findings**: 18, 32
- **Root cause**: the code that rewrites the `sources`/`content_type` properties (a) discards any value not in the built-in list, and (b) matches the key at *any* indentation, so it can delete an unrelated nested block.
- **Files**: `src-tauri/src/sources/mod.rs`
- **Test asserts**: unit tests for a nested `meta:\n  sources: x`, a text block containing the word `sources:`, and a user-typed citation not in the built-in list — all three survive a rewrite.
- **Owner clicks**: Add your own citation text to a note's sources, then accept a suggested source. Your original citation is still there.
- **Ruling needed** — see §3.

### Batch 8 — Links with a heading anchor count as links
- **Findings**: 14
- **Root cause**: `[[Stone Age#Tools]]` navigates correctly but is stored in the index with the anchor attached, so it never matches "Stone Age" — the backlink is invisible.
- **Files**: `src-tauri/src/search.rs` (`parse_link_body`), plus a re-index pass to converge existing rows.
- **Test asserts**: a link written with `#anchor` produces the same index edge as one without.
- **Owner clicks**: In note A, link to a heading inside note B. Open B's Backlinks panel — A appears. (Today it doesn't.)

### Batch 9 — Never write behind an open note
- **Findings**: 6 · 20
- **Root cause**: the Backlinks panel's "link this mention" writes straight to disk even when that note is open and has unsaved edits — those edits then overwrite the new link, or the link overwrites them.
- **Files**: `src/lib/components/BacklinksPanel.svelte` — adopt the existing proven pattern from `addTagToNote`.
- **Test asserts**: linking a mention in a note that is open and dirty routes through the open note, not the disk; the failure case surfaces instead of being swallowed.
- **Owner clicks**: Open note B and type something. Without saving, go to note A's Backlinks and click "link" on a mention of B. Both your typing and the new link are present in B.

### Batch 10 — YAML compose safety ⚠️ **TOUCHES THE SAVE/COMPOSE PATH**
- **Findings**: 15, 16
- **Root cause**: when a note's properties block has a YAML error, property edits are silently discarded and the save still reports success. Separately, a display-only type label can leak into the writer and flatten a nested property block into an empty value.
- **Files**: `src/lib/editor/yamlDoc.ts`, `src/lib/components/PropertyEditor.svelte`
- **Test asserts**: (a) editing a property in a note with broken YAML refuses the write, keeps the note dirty, keeps the safety net, and surfaces "can't save until the properties are fixed" — it must **not** silently pass through; (b) saving a note with a nested property map leaves that map byte-identical.
- **Owner clicks**: Full Editor-Surface Gate checklist (all 8 items) plus: a note with a nested property block, edited and saved repeatedly, never loses the block.
- **Why it's here and not earlier**: this is the compose function that runs on every save in the app. It must be reproduced first (a broken-YAML note and a nested-map note, both scripted), fixed, and put through the whole gate checklist before the owner sees it.

### Batch 11 — Rename-while-typing ⚠️ **TOUCHES THE SAVE PATH — build the harness first**
- **Findings**: 8 · 10, 11, 12, 13
- **Root cause** (one, four symptoms): during a rename cascade the editor stays live, and the code assumes it isn't. So (8·10) the note is reloaded from disk over your unsaved typing; (11) a note opened mid-cascade is in none of the protection sets; (12) two overlapping renames lift the protection early; (13) a property edit during a cascade is dropped before it reaches the note.
- **Files**: `src/lib/libraries/store.ts`, `src/routes/+layout.svelte`
- **Approach**: per Solve-the-Class, this is one fix, not four patches — make the cascade's protection *dynamic and reference-counted* (so a note opened mid-cascade inherits it and an overlapping cascade can't lift it early), and make the post-rename reload use the existing conflict-sidecar policy instead of clobbering a dirty note. Finding 13's one-line reorder lands with it, not before it — on its own it only narrows the window.
- **Reproduce-First**: this class cannot be shipped on static checks. Build the scripted reproduction first — type-during-rename, rename-during-open, two-renames-at-once — and get each one red before any fix, green after.
- **Test asserts**: after every recipe, on-screen content === disk content, and no keystroke entered during the rename is missing.
- **Owner clicks**: Open a note, start typing continuously, and have a rename of that note fire mid-burst. Every character you typed is present afterwards, and the note has its new title. Repeat while a second note in the same library is also being renamed.

### Batch 12 — The undo step that gets eaten
- **Findings**: 26, 37
- **Root cause**: "undo shape" marks the step used on the Rust side *before* the frontend decides whether it can apply it — so several early-exit paths consume your undo and do nothing.
- **Files**: `src/lib/components/NoteEditor.svelte`, `src-tauri/src/shape.rs`
- **Test asserts**: an undo that can't be applied leaves the history untouched; an undo issued just before switching notes never applies to the *new* note.
- **Owner clicks**: Change a note's shape, then undo — it reverts. Change shape, switch tabs immediately, then undo on the original note — it still reverts, and the note you switched to is untouched.

### Batch 13 — Background worker cleanups
- **Findings**: 33, 36
- **Root cause**: a background classifier can park indefinitely on a database lock despite claiming a timeout; a cleanup sweep can delete a file that another thread is still writing.
- **Files**: `src-tauri/src/cece/orchestrator.rs`, `src-tauri/src/cece/reliability.rs`
- **Fix**: the *small* version only — point the two database readers at the read-only connection so the worker can't block, and give the sweep an age guard. Correct the misleading comment.
- **Test asserts**: the classifier abstains within its budget while a long write is in progress; the sweep never removes a file younger than 60 seconds.
- **Owner clicks**: Trigger a large re-index and use the classifier suggestions at the same time — the app stays responsive and suggestions either arrive or say nothing, never hang.

---

## 3. Needs your ruling

**Q1 — When you clear the "sources" field on a note, should Constellation also delete citations it doesn't recognise?**
Today the code deletes any source value that isn't one of its built-in options — always, even when just accepting a suggestion (that part is unambiguously a bug and Batch 7 fixes it). But there's a second case: when you *deliberately* set or clear the sources list yourself, that's documented as your manual authority. Should your own hand-typed citations be wiped there too?
**Recommendation: no.** Keep unrecognised values you typed, and only remove what you actually removed. Deleting text a user typed, because the app doesn't recognise it, is never the safe default.

**Q2 — While a rename cascade is running (up to a few seconds), should opening a different note be blocked, or allowed but read-only?**
Batch 11 has to pick one. **Recommendation: allowed but read-only**, with the same frozen-overlay you already see on the other tabs. Blocking navigation feels broken; a visible "hold on" does not.

**Q3 — When a save can't be written because the note's properties block is malformed, what should you see?**
Batch 10 changes this from "silently pretend it saved" to a refusal. **Recommendation: refuse the write, keep the note marked unsaved, keep the safety-net copy, and show a plain message naming the note and the problem line.** Silence here is how edits vanish.

---

## 4. What I would *not* fix (and why)

Everything above gets fixed. These are specific *sub-options* proposed inside findings that I'd decline, not findings I'd park:

1. **Finding 33's option (b) — rebuilding the classifier worker as a detached thread.** The defect is that a background job can wait too long on a lock. Option (a) — read from the read-only connection so it can't wait at all — removes the cause completely with a change confined to two lines. Option (b) rewrites the threading model to add a timeout for a wait that would no longer happen. More risk introduced than removed.

2. **Finding 34's option (2) applied blanket — making re-index failure an error for every caller at once.** Roughly six internal callers currently ignore that result. Flipping them all in one commit means six behaviour changes landing together with no way to attribute a regression. I'd take part (1) (initialise the index properly before use, in Batch 5) and then convert the callers **one per commit**, each with its own decision between "surface it" and "queue a retry."

3. **Finding 27's suggested alternative — keeping the re-entrancy guard and adding a retry flag.** The finding itself notes this is strictly worse than deleting the guard, because the save queue underneath already serialises correctly. I'd delete it. But note this touches the save path, so it belongs *inside* Batch 10's verification run, not as a standalone commit.

4. **Finding 5/4's suggested "delete and re-index" as a shortcut** instead of migrating rows. It's simpler code and it destroys review history. Do the migration.

**Recommended first commit: Batch 1.** One function, one early return, and it closes the only APP-KILLER on the list.
# SESSION LOG — 2026-07-24 (session B, continuation after morning PCS)

**Function in hand:** MIG-103 — new-note-from-template **destination propose +
show** (the Boss-found silent-location bug from the 07-24 close), preceded by
triage of the in-flight safety-inspection run.

## 1. Triage of `wf_acc3ca2c-4f6` (the inspection that was running at PCS)

- Journal shows **14 finder agents started, 0 completed** — the run was killed
  ~2 minutes in (09:27 → 09:29) when the session closed. No agent reached its
  structured output; **zero findings exist from that run** — nothing was hidden,
  nothing to salvage. The Slice-2 batch inspection was therefore still OWED.
- **Re-launched fresh**: run `wf_8aff685b-b8b`, diff-scoped args over the
  Slice-2 batch's code files. PJ-124 fired again (8th+): `args.files` ignored,
  ran whole-app — acceptable, since whole-app covers the batch.
- **Result: 87 agents, 14 scopes, 62 CONFIRMED** — 2 APP-KILLER, 14 HIGH,
  27 MED, 19 LOW. Full register committed at
  `lab/reports/INSPECTION-2026-07-24-wf_8aff685b.md`.

### Fixed in this pass (8) — every one verified in code before the edit

- **APP-KILLER #1 — silent frontmatter child deletion** (`store.ts`
  `parseFrontmatter` + `yamlDoc.ts`). The list branch consumed only consecutive
  `- item` lines, so a **seq-of-maps** and a **block list with an interior blank
  line** reached the panel TRUNCATED and *editable*; one chip edit made compose
  splice the block and rewrite it from the truncated projection. Fixed on both
  sides: the parser now takes the block's FULL extent and projects what it
  cannot round-trip as READ-ONLY; `immutableBlockKeys` refuses any seq holding a
  non-scalar. **`ikhtilāf` exempted by key** — it has a real lossless serializer
  (MIG-101), and the regression that exemption prevents was caught by the
  existing `nestedObjectListRoundtrip` suite going red mid-fix. The key set now
  lives once, in `yamlDoc` beside the serializer that makes the claim true.
  New: `tests/mig-103/seqOfMapsRoundtrip.test.ts`, 7 cases incl. two guards
  proving ordinary `tags:` lists stay editable.
- **APP-KILLER #2 — Approve All could erase the batch it just accepted**
  (`sources/bulk_ops.rs`): the missed sibling of the 2026-07-22 announce fix.
  Now announced, **batched** on the progress boundary, drained on cancel.
- **The library-resolver class, 4 sites** (`libraries.rs:918/1203/1807`,
  `universe.rs:2119`) → `library_name_for_path`. The canonical longest-root-wins
  resolver already existed; the defective sites never called it, so every note
  created/renamed/cascade-rewritten in a nested sub-library was indexed under the
  universe-root library. Frontend twin (`handleRenameComplete`) → the shared
  `libraryForPath`, where the cascade could otherwise walk the WRONG library.
- **The three Slice-2-batch findings**: `merge_fields_into_template` unguarded
  RMW → `gate_rmw` + announce; `props_reparse_backfill` **verifies completeness
  before stamping** (the `note_body_backfill::finalize` pattern); the Studio
  detail column **keyed on the selected kind**.

### NOT fixed — 54, explicit ruling requested (PJ-140)

Whole-app findings unrelated to MIG-103, several crossing subsystem boundaries.
Per WA#6 they are recorded in full, never silently parked; fixing them as
drive-bys inside a feature build is the unproven cross-cutting change WA#4
forbids. Standouts: folder rename has no descendant index cascade; `delete_path`
on a folder purges nothing from the index; `review-pulse.json` is a non-atomic
write whose loader silently defaults (a corrupt file loses all review history);
the schema-version rebuild deletes `search.db` — the only store of user-earned
Living-Link properties — on every version bump.

## 2. Predecessor → Replacement (written before the code edit)

- **Predecessor:** `newNoteFromTemplate` (+layout.svelte, MIG-103 §1) chose the
  destination SILENTLY — focused note's folder, else `stats[0].path` (the first
  library = universe root). No user-visible destination anywhere.
- **Replacement (same place):** destination is chosen BEFORE creation and SHOWN
  in the title prompt. `newNoteFromTemplate` now takes `(templatePath, title,
  folder, libraryName)` — the silent fallback is deleted.
- **Cut:** the silent `stats[0]` fallback. **Kept:** the same create path
  (`createNote` → `openNoteTab`), same template processing, same error surface.

## 3. What was built (uncommitted, pending Boss test)

The ruled interaction model (Boss 2026-07-24): **propose + show; library picker
when nothing is open.** Reuse MoveDialog — reuse, don't rebuild.

- **TemplatePrompt.svelte** — optional `destinationLabel` +
  `onChangeDestination(currentValue)` props: a destination row under the title
  input (folder icon + "Will be created in:" + `Library / sub / folder` +
  **Change…** button). The callback carries the current input value so an
  edited title survives the round-trip through the picker. All other mount
  sites unchanged (props optional).
- **MoveDialog.svelte** — optional `title` + `confirmLabel` props; empty =
  original Move labels, every existing mount unchanged.
- **+layout.svelte** —
  - `libraryForPath()` extracted (longest-prefix library match) — now shared by
    `revealInTree`, `openMoveDialog`, and the new destination code (was inlined
    twice before; one source of truth).
  - `buildUniverseFolderEntries()` extracted from `openMoveDialog` (the whole-
    universe folder walk + the nested-library dedupe) — shared by Move and the
    new destination picker.
  - `describeDestination()` — names a folder the way the sidebar does:
    `Library / sub / folder`, or just the library name at a library root.
  - `openNewNoteDestPicker()` + `newNoteDestPicker` state — mounts MoveDialog
    with picker labels; `returnTo` restores the title prompt when a
    Change…-opened picker is cancelled; cancel of a nothing-open picker closes
    the flow.
  - `handleTemplateSelect` ('newNote'): focused tab → propose its folder and
    show it in the title prompt; nothing open → picker FIRST, then the title
    prompt with the chosen destination shown.
  - `newNoteFromTemplate(templatePath, title, folder, libraryName)` — explicit
    destination, silent path removed.
- **i18n ×15** — 4 new keys under `templates`: `newNoteDestination`,
  `newNoteDestinationChange`, `newNoteDestinationPick`,
  `newNoteDestinationConfirm` (ar de en es fa fr he hi ja ko pt ru tr ur zh);
  all files parse.

## 4. Verification so far

- svelte-check: **0 errors** (1581 files).
- vitest: **605/609 passed; the 4 failures are the two known PJ-132 Sight perf
  flakes** (parallel-load timing); both files pass isolated **31/31**. My diff
  touches no Sight code.
- Frontend rebuilt (`npm run build`, 47s) and the new strings verified INSIDE
  `build/` (`newNoteDestinationPick`, "Will be created in:") — the
  stale-embed trap is closed.
- `cargo build --release` running in background (task bszd241q6).
- Safety inspection `wf_8aff685b-b8b` running; findings gate the commit.

## Boss validation (on the 10:34 release binary)

- **Stage 1 — destination propose + show: PASS** (all three tests: the shown
  destination, Change… into the folder tree, and picker-first with nothing open).
- **Stage 2 Test 1 — the frontmatter app-killer: PASS, verified on disk.** The
  Boss edited `stage` (spark-seed → birth-seed) and added the `EisaTest` tag on
  the probe note; the `authors:` block survived byte-intact — both `name:` and
  both `role:` lines, correct indentation — and `tags:` remained fully editable.
- **Stage 2 Test 2 — the bulk-accept app-killer: NOT RUNNABLE, recorded as such.**
  The Cataloger reported "Apply suggestions to 0 notes": `skipSplit` is hardcoded
  `true` and all 7,179 cards are split (`Catalogers agreed (0)`), so every card
  was filtered and `accept_one` never executed. The app was correct and said so.
  **Verified by construction instead:** the announce bookkeeping was extracted
  into a pure `AnnounceBuffer` and 6 Rust tests prove *every written path is
  announced exactly once* across queue lengths 0–40, cancel inside the unflushed
  window, no double-announce, and no announce for idempotent no-ops. The emit is
  identical to the four per-card seams Boss-validated on 07-22. **Residual gap
  filed as PJ-142** — no end-to-end run; the repo has no Tauri mock-app harness.

**Final: Rust 1139/0 (+6) · svelte-check 0 · vitest 616/616 (all green).**

## Build-chain lesson (cost ~25 min)

`npm run build … | tail -3 && cargo build --release … | tail -2` reported **exit
0 while leaving the old binary in place**. Two compounding causes: `tail` buffers
until EOF so cargo's output never surfaced, and the real error —
`failed to remove constellation.exe: Access is denied (os error 5)` — was the
**running app holding the binary**. This is the same empty-log-looks-like-progress
trap recorded in the 07-24 morning log. Rules reaffirmed: never pipe a build
through `tail`; verify the artifact's mtime, never the exit code; and when a
monitor watches a build, make it report the FAILURE case too (the one armed here
did, which is how it was caught). The app was NOT killed — the watcher waited for
the Boss to close it, then built cleanly.

## Open at this point

- Boss test of the destination flow (tutorial after the binary lands).
- §1 use-side remainder (mixing heads-up), then D4 — per ledger v1.46 queue.

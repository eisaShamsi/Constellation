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

---

# Second batch (afternoon) — Boss-directed: 3 fixes · CLAUDE.md · scan freeze

## 1. The search.db destruction gate — FIXED (`search.rs`)

Three compounding defects, fixed together. **The gate never deletes now** — it
RENAMES the old database aside (`search.db.pre-v7-<UTC>.db`), so even a real
rebuild leaves the earned data recoverable.

The subtle one and the most likely to fire: **absent is not stale.** A missing or
unreadable `.version` marker used to mean "rebuild", so one absent 4-byte file
authorised destroying a 1.93 GB database (a restore that skipped it, a sync
filter, one failed write). Unknown now means adopt-and-stamp, never destroy.
Also: the version is stamped ONLY if the set-aside actually succeeded (a failed
rebuild recorded as done cancels the migration permanently), and the stamp's own
failure is surfaced instead of swallowed (otherwise: full rebuild every launch,
forever). Decision extracted to a pure `schema_gate()`; **5 tests** incl.
whitespace-normalised markers and corrupt-vs-absent.

## 2. CLAUDE.md — CORRECTED (the documentation was the bigger hazard)

CLAUDE.md asserted *"Dual-layer storage: LINK files on disk (source of truth) +
note_links SQLite table"* and a `LINK` file kind. **Neither is implemented** —
verified by exhaustive search: no code writes a LINK file; no code persists
weight/confidence/traversal/archival to any file. The section now leads with that,
enumerates exactly which fields are recomputable from the `.md` files and which
live only in `search.db`, and states that **search.db is currently the system of
record** for the earned half of the Living Link Architecture — not a disposable
index. A doc that promises a guarantee the code does not provide is how the next
session confidently ships the next deletion.

## 3. The hour-long Cataloger scan — diagnosed + partly fixed

Multi-agent investigation with adversarial refutation (9 agents). Measured on the
real 7,339-note Universe: ~19 min of the hour is database time alone, from TWO
full-table scans per note, 7,339 times, whose answers never change during the run.

**Fixed now (zero behaviour change):**
- **The freeze** — both neighbour lookups took the WRITER lock for pure reads
  (`cece/wiring.rs`), so for an hour every save/watcher/reindex queued behind the
  scan. Now on the read-only connection (`with_read_conn`, PJ-066 §C3).
- **The missing yield** — this was the ONLY background job in the codebase with no
  inter-item pause; every sibling has 30–50 ms. Added at 30 ms (`scan_job.rs`).
- **The lying header comment** — claimed "~30ms per note … ~3.5 minutes for 7,000
  notes"; reality is >1 hour. Replaced with the measurement and a warning not to
  restore a throughput claim without measuring on a large Universe.

**Deliberately NOT applied — the report's own top fix.** It proposed making the
dead link query fast by skipping rows. Root cause: `note_links.target_path` is
empty on all 234,192 rows, so the Graph Cataloger abstains on every note. Verified
why: `resolve_incoming_target_paths` is misleadingly named — it resolves a NAME to
paths for aggregate maintenance and never writes `target_path`; the only writers
are a one-time MIG-003 migration. **Making a broken query fast would cement the
bug.** Filed as PJ-143 (correctness), and the structural per-scan-snapshot fix as
PJ-144 (`/migration`).

## 4. APP-KILLER — archiving a link was reversed by the next save (FIXED)

Found independently by BOTH the safety inspection (finding 30) and the MIG-104
architect pass — which is why it got promoted out of PJ-140 and fixed here.

Archival is deliberately "archival, not deletion", so the `[[wikilink]]` stays in
the note. But `index_note`'s unchanged-edge fast path requires `status=='active'`,
so an archived edge NEVER matched, was deleted, and re-inserted with `status`
hardcoded `'active'` — and the `preserved` map didn't carry status at all. One
ordinary edit to the note un-retired the link, silently. Worse: it returned as
ACTIVE with weight 0.0. No index loss required — this fired on any save.
**Fixed:** status is preserved and RESTORED; `status != "active"` qualifies a row
for preservation on its own (not relying on archive's weight=0.0 side effect).
**5 tests** pin the rule, incl. that ordinary active links are still skipped.

## 5. MIG-104 Architect document — WRITTEN, awaiting Boss ruling

`docs/migrations/MIG-104-Architect-durable-earned-link-data.md` (13 agents; 4
independent designs + a 4-judge panel). Recommends an **Earned-Life Ledger:
snapshot + tail** — append-only, so the write mechanism is structurally incapable
of destroying what it protects. Measured scale: of 234,192 links only **35** carry
earned state — small enough to build now and prove the mechanism long before it
carries years of reading. **5 open questions for the Boss.**

**Rust 1149/0 · svelte-check 0 · vitest 616/616.** Binary rebuilt 18:39.

---

# Third batch — sidebar library duplication (Boss-found) + MIG-105 logged

## The bug (Boss-found, live)

Creating a library inside the Universe root duplicated it in the sidebar: once as
a top-level library, once as a folder inside the root library "Eisa Cognitive
Knowledge". Root cause: `ensure_universe_notes_folder` (universe.rs:403) registers
the root library with `path` = the Universe ROOT, so `read_library_tree`'s recursive
walk descended into every OTHER registered library nested under the root and rendered
it as a folder. Violates "Library ≠ Folder".

## Blast-radius verified BEFORE the fix (Explore agent)

VERDICT: SAFE. The decisive check — does any FUNCTIONAL path find a nested
library's notes by walking the PARENT tree? NO. Indexing is per-library via
`index_library_recursive` (search.rs:9363-9366), NOT `read_library_tree`; the
watcher attributes via `library_name_for_path` (longest-root-wins); reveal uses
`libraryForPath` (longest-prefix). The only consumers of `read_library_tree` are the
sidebar tree (the duplicate) and OrgChart (a double-count) — both FIXED, not broken,
by the exclusion. `read_dir_recursive` has exactly one caller, so its signature is
safe to change.

## The fix — display-layer only (libraries.rs)

`read_library_tree` builds a normalized set of all registered library paths EXCEPT
the one being walked (from the `load_all_libraries` it already loads — zero extra
cost), and threads it into `read_dir_recursive`, which `continue`s on any child dir
whose normalized path is in the set. Self-scoping: covers "library under the Universe
root" and "library under another library" at any depth, matching the longest-root-wins
behaviour the indexer and Move dialog already use. OrgChart's double-count is fixed
automatically (it calls the same command). Touches NO data model — that is MIG-105.
3 tests (nested at root, nested at depth, empty-set regression guard) drive the real
walker over a temp tree.

## MIG-105 LOGGED (Boss-directed: "log it in its own /migration")

`docs/migrations/MIG-105-root-library-vs-flat-universe.md` — the Boss's question
"why not flat, like Obsidian?" reserved as its own migration. Concept + symptom +
the six-patch evidence trail that this one design decision (root library named after
the Universe, claiming its path, at index 0) is a ROOT CAUSE. Three options
(exclusive-scope root / remove entirely / forbid nesting). Architect workflow to run
after the current job closes. Filed as PJ-145.

Also noted in MIG-105 as further evidence: `index_library_recursive` does NOT stop at
nested library roots, so a nested note is indexed twice (root name, then nested name;
nested wins by iteration order) — a pre-existing double-INDEX, out of scope for the
display fix, a MIG-105 concern.

**Rust 1152/0 (+3) · svelte-check 0.** Binary rebuilding; per-build safety
inspection running over the changed index/lifecycle files.

## Per-build safety inspection — PASS (no regression introduced)

`wf_1b68be62-d7e` over the four changed index/lifecycle files. PJ-124 again → ran
whole-app (31 agents, 14 scopes). **11 confirmed — 7 MED, 4 LOW; zero APP-KILLER,
zero HIGH.** Checked every finding against the batch's exact `git diff` line ranges:
**none is at a line this batch changed.** The five in touched files are all in other
functions (move_item, save_libraries, incoming-aggregate diff, reindex commands) the
batch never edited. All 11 are the pre-existing whole-app backlog (≈7 already in
PJ-140, ≈4 net-new register entries) — folded into PJ-140, NOT fixed inside this
commit (WA#4 drive-by prohibition; PJ-140's ruling is already pending).
Register: `lab/reports/INSPECTION-2026-07-25-wf_1b68be62.md`.

---

# THE WHOLE-ECOSYSTEM FIX LAW + the file-tree sweep (Boss-dictated 2026-07-25)

## The law

Boss, after finding the Move picker still showed nested libraries as folders (my
sidebar fix left its sibling walker inconsistent): "You should be thorough when
fixing anything… tackle everything related to the file tree/explorer in every
function or aspect within the Constellation ecosystem. Consider this as a law."

Codified as a top principal in CLAUDE.md ("The Whole-Ecosystem Fix Law") + memory
`feedback_whole_ecosystem_fix_law.md`: fix the WHOLE concern across the ENTIRE
ecosystem — every surface — not just the call site; grep exhaustively (spawn an
audit workflow for broad concerns); shared helper so surfaces can't drift.

## The sweep (an exhaustive 7-agent audit found 34 broken surfaces)

The concern: enumerating/rendering/attributing the file tree & library structure,
honoring "Library ≠ Folder" (a nested registered library must appear ONCE, not as a
folder of its parent) and longest-root-wins attribution (never first-match, which
always returns universe_notes at index 0). The root of every symptom: universe_notes'
path IS the Universe root, so every naive walk from it swallows nested libraries.

**Rust — 10 walkers fixed with two shared helpers** (`nested_library_paths` +
`is_nested_library`, beside `library_name_for_path`):
- **index_library_recursive (THE data-model core)** — was fixed-name + no-exclude, so
  after any rebuild every nested-library note carried `library_name='universe_notes'`
  and the nested library reported **0 notes** ("Eisa Test looks empty"), corrupting
  every count/search/scope. Now attributes PER FILE via `library_name_for_path`
  (matches the correct `reindex_md_descendants` model) + excludes nested libs.
- **reindex_library** — same per-file attribution.
- **7 aggregate walkers** (scan_links, scan_tags, notes_by_tag, scan_stages,
  scan_index_words, scan_tasks, scan_dates) — exclude-set so a nested library's
  links/tags/tasks/dates/stages/words are counted once, under its own library, not
  double-counted into the parent (all called per-library, verified).
- **read_dir_recursive + collect_folders** (the two Boss-found ones) consolidated onto
  the shared helper.
- **library_attribution_backfill.rs** (NEW) — a one-shot, versioned, completeness-
  checked boot pass that re-attributes `note_meta.library_name` rows a PRIOR reconcile
  corrupted, so an existing universe self-heals ("Eisa Test" regains its notes) without
  a manual rebuild.

**Frontend — 33 first-match sites → `libraryForPath` (longest-root-wins)** in
+layout.svelte (note-open attribution, tree refresh, rename collision scope, bookmark/
collection stamping, copyRelativePath, ~11 overlay/panel open-note handlers). Regex
sweep, 0 remaining. store.ts's `deriveLibraryForPath` was already longest-root-wins.

**Dormant (Map/Sight OFF):** map.rs `build_library_node` noted for the same fix at
re-enable (MIG-038).

**Tests:** shared-helper tests (exclude-self, nested-depth, longest-root-wins);
Rust 1157/0 + the new helper tests; svelte-check 0; vitest 616/616.

This is the SYMPTOM fix across every surface; MIG-105 remains the data-model root
cause (the root library sharing the Universe's name + claiming its path at index 0).

## Per-build inspection over the whole-ecosystem sweep — `wv6f7sl7i`

Whole-app again (PJ-124). **8 confirmed — 6 MED, 2 LOW; zero APP-KILLER/HIGH.**

- **The 33-site frontend sweep + the 10 Rust walkers introduced NO new finding** —
  no finding sits at a swept line.
- **[5]/[6] — `migrate_note_db_paths` (my new helper) FIXED IN-PASS.** I'd guarded
  review_schedule against a pre-existing destination row but not note_meta /
  note_embeddings (both PK on path); a stale phantom row at the destination would
  make the UPDATE silently fail and orphan the moved note. Since I *widened* this
  helper's use to move + folder-rename (WA#6 + the law: fix what you touch), added
  the delete-first guard for both PK tables + a collision test. 4 migrate tests green.
- **Deferred (pre-existing, → PJ-140):** [0]/[1]/[2] store.ts editor-lifecycle
  cluster (own migration); [3] search.rs sky maintenance gated on the incoming stamp
  instead of the sky stamp (the #48 family); [4] BacklinksPanel linkMention (frontend
  PJ-140 batch); [7] provenance.rs sync-command freeze.

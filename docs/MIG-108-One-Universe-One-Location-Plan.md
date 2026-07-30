# MIG-108 — One Universe, One Location — Plan (Phase 2)

**Date** 2026-07-30 · **Status** awaiting Boss approval → Build
**Architect:** `docs/MIG-108-One-Universe-One-Location-Architect.md` (hazards H1–H12,
invariants I1–I10 referenced below).

**Boss rulings received (2026-07-30):**
- **D1 — layout: FLAT.** Each relocated library lands at `<universe root>\<library folder name>`.
- **D2 — bring-in: ASK EACH TIME.** The bring-in dialog offers Copy (default) and Move, plainly explained.
- **D3 — PJ-065-test-book: relocate BY COPY.** Its 15 notes are copied under the root and the
  registration re-points to the copy; the originals stay in the source repo as git-tracked
  fixtures. The other 17 externals are true same-volume moves.
- **D4 — order: MIG-108 first**, MIG-104 Slice 8 immediately after.

Slices land in order, one commit each (or a small commit series where noted), each with a
verification clause. Boss-testable pauses are marked ⏸. Per the safety-inspection standing
order, every slice that touches a write path runs the diff-scoped inspection before its commit.

---

## Slice 0 — Pre-work: close the two unexcluded walkers (H8)

`canonical.rs::collect_files_recursive` (feeds `repair_external_libraries_on_startup`) and
`embeds.rs::build_vault_index` gain the `nested_library_paths` exclusion every sibling walker
carries. Needed regardless of MIG-108 (two libraries are already nested under the root today);
after relocation ~19 are, and these two would double-walk everything.

**Verify:** new Rust tests — a nested library's files are excluded from the parent's canonical
probe and embed index; the embed shadow-precedence behaviour pinned. Full suite green.

## Slice 1 — Migration engine I: journal, classifier, snapshot

New `src-tauri/src/mig108.rs`:
- **Journal** (`.constellation/mig108-journal.json`, atomic_write): per-entry states
  `planned → moved → rewritten → done`, plus snapshot paths and the old→new map. Idempotent
  loader; a journal with unfinished entries is the resume signal.
- **Pre-flight classifier** (read-only): for every `libraries.json` entry —
  `under-root` (skip) · `external-own` (move) · `repo-resident` (copy, per D3 — detected by
  path-under-source-repo… generalized: a `copy` classification list supplied by the caller;
  the Boss universe's proposal marks PJ-065-test-book) · `foreign-universe` (another registered
  universe's root or child → SKIP + report, H6, via `registered_universe_roots` +
  `resolve_child_universe_roots`) · `missing` (report). Flat-layout destination
  `<root>/<basename>` with basename de-collision (numeric suffix) and a collision report.
- **Snapshot** (H5): `wal_checkpoint(TRUNCATE)` → copy `search.db` (+ sidecars if non-empty)
  → open the copy read-only and assert row counts; copy the eight path-bearing JSON stores.

**Verify:** Rust tests — classifier over fixture layouts (each class + a basename collision +
a foreign-universe entry); snapshot round-trip row-count assertion; journal state machine
(resume from each state).

## Slice 2 — Migration engine II: move + rewrite + verify

- **Move phase:** per journal entry — `gate_rename(old_dir, new_dir)` for moves;
  `copy_dir_recursive` (+ no delete of source) for the D3 copy class; cross-volume fallback =
  copy+remove. Journal `moved` after each. Halt-clean on failure (open handle) with the journal
  naming the resume point.
- **Rewrite phase, one transaction:** `BEGIN IMMEDIATE` + `defer_foreign_keys=ON` (H1);
  drop the outgoing + sky trigger sets (H2); enumerate stored rows per moved library via
  normalized-prefix match (H3 — the `delete_rows_under_prefix` pattern, never SQL
  replace/LIKE); rewrite the 11-table set (the `migrate_note_db_paths` statement set,
  prefix-batched) **plus** sky_nodes / sky_links / note_aliases explicitly (their triggers are
  down); destination pre-deletes for phantom rows; reset the 4 backfill cursors; wipe sight_v3
  rows; recreate triggers; `recompute_all_outgoing` once; flag link_stats recompute.
  **In-tx verification before COMMIT (I2):** zero rows under any old prefix · per-library row
  counts preserved · `SUM(weight)` + `COUNT(*)` over note_links and `COUNT(review_schedule)`
  identical to the pre-move snapshot values · FK check clean. Any failure → ROLLBACK + journal
  `verify-failed` (fs moves reversible by journal replay).
- **JSON rewrites** (H12, after COMMIT, each atomic_write, journaled per store):
  `libraries.json` (the pivot — new paths, ids/names unchanged) · `review-pulse.json` (4 maps
  re-keyed) · `workspaces.json` · `session.json` · `collections.json` (cid-less + folder/search
  members) · `settings.json` (`folderTemplates` keys, absolute `templateFolder`).
- **Copy-class cleanup:** the repo original is UNREGISTERED (its files untouched); the copy is
  registered at the new path.

**Verify:** an end-to-end Rust test against a scratch fixture universe (tempdir, 3 mini
libraries incl. one copy-class, seeded search.db with earned rows + FK children + aliases):
every I-invariant asserted mechanically; an interrupt-resume test (process the journal, stop
after `moved`, re-run → completes); a RED variant (verification catches a deliberately dropped
row → ROLLBACK). Full Rust suite green.

## Slice 3 — Trash consolidation + settings collapse + PJ-192

- Consolidate every `<library>/.trash` top-level entry → root `.trash` via
  `trash_move_decolliding`; remove empty source `.trash` dirs. Runs as the migration's T step
  AND as a standalone idempotent pass (covers universes that never need relocation).
- Remove `trashFolderScope`: type + default + UI rows + explicit `delete parsed.trashFolderScope`
  purge in `applyParsedSettings` (the spread resurrects stale keys otherwise) + 60 i18n keys
  (+ the 2 verified-orphan keys). `resolveTrashDestination` collapses to universe-root.
- **PJ-192 closes:** retire the Rust `move_to_trash` command; re-point its single caller
  (`universe.rs` Template-Studio undo) to the `delete_path` pair (reindex parity verified).
- Rewrite `tests/pj-187/trashDestination.test.ts` (its fixture layout and scope axis both die).

**Verify:** Rust + vitest green; a settings file carrying the stale key loads clean and
round-trips without it.

## Slice 4 — Proposal UI, progress, resume-on-boot ⏸

- Detection at activation (after `ensure_universe_notes_folder`): externals present → the
  **proposal dialog** (what was found, where each lands, what is skipped and why, where the
  backup goes). Nothing moves without the user's click (The Constellation Way).
- Run envelope: flush dirty tabs → force-close the second screen (H9) → `markFreeze` +
  `markCascadingLibrary` per library → `unwatch_library` ×N (H10) → engine (Slices 1–3) with a
  progress modal → rewatch at new paths → unfreeze → cache refresh → **summary screen**.
- Resume-on-boot: an unfinished journal found at activation → resume dialog (not silent).
- i18n ×15 for every new string.

**Verify:** svelte-check + suites green. ⏸ **Boss Stage-A test (on a scratch copy):** the
proposal, progress, summary and resume dialogs walked through per the Testing Instructions
Rule — on a copy, not the live universe.

## Slice 5 — Standing constraints: the bring-in flows

- `add_library` re-shaped: external path → the **Ask dialog** (D2): Copy in (default) / Move in,
  each explained; destination `<root>/<basename>` de-collided; then register + reindex + watch.
- `create_new_library_at` constrained under the root; `pick_folder` scoped for library
  destinations; legacy `create_new_library` retired.
- Importer target-library list restricted to OWN libraries (closes the federated-target hole).
- `link_library_as_universe` double-entry registration fixed in the registry-normalization pass.
- Doctrine strings rewritten ×15: `libraries.linkLibrary`("Link Existing Library"),
  `linkLibraryDesc`, `universe.setup.linkLibrary/Desc`, `libraryManager.openLibrary`,
  `app.tagline` ("A Vault of Vaults" — also retires forbidden "vault" wording), and the
  in-code doctrine comments.

**Verify:** suites green; a manual bring-in on the scratch universe lands the copy under the
root with the original untouched (Copy) / moved (Move). ⏸ folded into Stage-B below.

## Slice 6 — The rehearsal (I-invariants on a full copy)

Script: copy the REAL universe root + all external trees to a scratch root → run the entire
migration headless there → assert every I1–I10 mechanically (incl. aggregate equality against
the live DB's numbers) → open the rehearsal universe in the app and spot-check search, links,
backlinks, review queue, collections, Sky. Fix-what-it-finds, rerun to green.

**Verify:** the rehearsal report (counts, aggregates, timings) — attached to the session log.

## Slice 7 — The live run ⏸

⏸ **Boss Stage-B:** on the real universe — the proposal dialog appears, Boss clicks Unify,
watches progress, gets the summary; then an in-app validation walk (tutorial per the Testing
Instructions Rule): search a relocated note · follow links both directions · review queue
intact (priorities/snoozes survive) · collections + starred intact · Sky View alive · create,
rename, delete, bring-in each work · one `.trash` at the root. The snapshot is retained until
the Boss declares pass.

## Slice 8 — Docs + the Phase 4 audit

- `CLAUDE.md` Knowledge Hierarchy amended (the repealed sentence replaced by the One-Location
  rule + bring-in semantics). Orientation bump; PJ ledger (MIG-108 closed; PJ-192 closed;
  file the relative-paths end-state PJ); User Manual ×15 + help topics (universe structure,
  bring-in flows, one trash).
- **Phase 4 audit** per the Migration Rule: three parallel agents — invariants (I1–I10 against
  the shipped code), drift (new guards the system doesn't know about), migration path
  (first-boot, foreign-universe skip, mid-journal interrupt, rollback).

---

**Estimated shape:** 8 slices, ~6 commits + the audit. Slices 0–3 are pure Rust/engine work
(no Boss time); 4 and 7 are the two Boss pauses; 6 runs unattended.
**Rollback story:** before Slice 7 nothing touches the live universe; at Slice 7 the journal +
snapshot make every step reversible until the Boss pass, after which the snapshot is archived.

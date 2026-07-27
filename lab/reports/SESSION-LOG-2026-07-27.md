# Session Log — 2026-07-27

**Branch:** `main` @ `95390d3a` at start. **Function in hand:** MIG-104 — the Earned-Life Ledger
(durable earned link data + note change history for the Boss's time machine).

---

## MIG-104 Phase 2 (Plan) — APPROVED by the Boss

`docs/migrations/MIG-104-Plan-earned-life-ledger.md` (1,640 lines, 16 slices). Boss rulings on the
eight approval decisions, all recorded in the doc's header: #1 accept (walks append immediately, no
coalescing) · #2 accept (Q3 = fix all four review-state defects + PJ-163) · #3 accept (one store per
Universe in `.constellation/`) · **#4 "Ok" → BUILD the continuous note-history mirror** (Slice 9) ·
#5 accept (archive former names) · **#6 "Yes" → ARCHIVE THE NOTE BODY TOO** (new sub-slice 8b — the
time machine must survive an emptied Recycle Bin) · **#7 "Ok" → the LINK authoring surface ships as
sibling MIG-106** (PJ-169), opened after Slice 6 validates; Q4's "build both" honored by sequencing ·
#8 accept (the corrected `.gitignore` list — `.constellation/` 2,836 MB → 38 KB).

### The Plan's headline finding — a defect in MIG-105 Stage 0 (`042802c5`), reproduced and corrected

FK enforcement means `ON DELETE CASCADE` fires at `DELETE FROM note_meta` (search.rs:9845), so the
three FK-bearing purges Stage 0 added ~30 lines later (:9874-9876) match **ZERO rows** — and the
Stage-0 comment calling the cascade "inert in production" was **FALSE**, contradicting
`tests_pj150_fk_enforcement_reality` landed in the *same commit*. Consequence for the Boss's
archive-first ruling: **an archive hook at the purge would archive NOTHING**; it must go BEFORE the
parent delete. Pinned by `tests_stage0_delete_order_defect`; comment corrected in place (`95390d3a`).
A second destroyer the Plan found: `migrate_note_db_paths`' destination pre-delete (libraries.rs
:1157/:1190) destroys a phantom's trail on every rename onto an occupied path — its own slice (10).

### Corrected figures (the Architect's were stale)

7,817 notes · 234,233 links · **15** empty cids (not 17; 14 are templates by design) · **33**
recordable earned links (not 35 — two are structural) · 39 real clicks (not 41) · **236 weights are
arithmetically impossible** so `weight != 1.0` is banned from the earned predicate · 19,481 history
rows / 15.1 MB across 7,785 notes, **693 already lost** · Slice 6's headline defect is already fixed
in the tree.

### Honest gaps in the Plan's evidence

- The **Q3 investigation FAILED** (structured-output retry cap); Q3's answer rests on the plan
  author's own reading, not a dedicated investigation. Re-run before Slice 14.
- **Nothing in Constellation reads `note_state_history` today** — `cece_get_note_history` /
  `cece_query_history` are registered but invoked from nowhere (the Sight v3 surface that would read
  them is retired). Slices 8-11 are therefore verified by reading the archive file + the harness,
  **not** by a screen. The screen is the time-machine feature itself (MIG-104 §7.2).

---

## Slice 0 — the performance baseline (LANDED, no product code)

**Files:** `tests/mig104/harness.ts` (the shared ledger machinery + both fold algebras),
`tests/mig104/baseline.test.ts` (12 tests, all green), `tests/mig104/README.md` (the recipe registry
+ the baseline table), and `mod tests_mig104_baseline` in search.rs.

**The baseline, measured — never again to be argued:**

| Metric | Median | p90/p95 |
|---|---|---|
| `paint_ms` | 672 ms | 906 ms |
| `libraries_loaded_ms` | 752 ms | 1,012 ms |
| `hydrated_ms` | 34,871 ms | 37,462 ms |
| `graph_ready_ms` | 35,546 ms | 37,839 ms |
| append 200 B, no fsync | **168 µs** | 333 µs |
| append 200 B + fsync | **3,418 µs** | 4,922 µs |

(Boot figures: the app's own `boot-perf.history.jsonl`, 814 full-universe boots, last-20 window.
fsync figures: release-mode measurement on the Boss's E: drive, 200 iterations.)

**★ What the fsync measurement changes.** fsync is **20× a plain append**. The Plan had specified a
uniform "file-first with fsync" for decisions; the number makes the per-site rule explicit and is
now recorded in `tests/mig104/README.md`: fsync is MANDATORY for archive-before-purge (the purge
destroys the only other copy microseconds later) and for user decisions (rare, irreplaceable); walk
counters and the continuous history mirror use a **plain append (no fsync)** — which is precisely
what makes the Boss's decision #1 (no coalescing) affordable at 168 µs per click instead of 3.4 ms.

**Verified:** `conn.path()`'s parent IS the `.constellation` dir — the mechanism that lets the ledger
need zero path plumbing at any writer. Rust 1182/0 (+2) · vitest 12/12 on the new dir.

### Slice-0 side finding — the Sight perf tests are load-dependent, and that is a test-design defect (PJ-132)

The first full-suite run after Slice 0 showed **1 file failed / 53 passed**: `tests/sight-v6/perf.test.ts`
+ `tradition-perf.test.ts`. **Not a regression** — the same file passes **27/27** when run alone;
it fails only when the machine is loaded (concurrent cargo/vitest/workflow runs). Verified both ways.

The real defect: these tests assert **wall-clock budgets** (`≤16 ms` per tradition switch, `≤32 ms`
for facet rebalancing) inside a *parallel* runner, so they measure the machine, not the code. That is
PJ-132 ("Sight flake") and it now matters more than before: since PJ-157 made the suite glob-driven,
a green suite is the gate for every MIG-104 slice, and a load-sensitive test makes that gate lie in
both directions (a real regression could hide behind an assumed flake). **Recommended fix at PJ-132:**
move wall-clock budget assertions into their own serial lane (`vitest --no-file-parallelism` or a
dedicated config, like `vitest.manual.config.mjs`), or convert them to operation-count/complexity
assertions that are load-invariant. Until then, MIG-104 slices treat these two files as a **separate,
serially-run gate** rather than part of the parallel suite.

---

## Slice 1 — the `.constellation` watcher predicate (LANDED)

**Concept:** the app's own bookkeeping folder must never look like the user's knowledge changing.

**Why first:** Boss ruling Q1 puts the ledger inside a folder that sits within a **recursive** watch —
the Universe root IS a registered library and `watcher.rs` watches `RecursiveMode::Recursive`.
`EXCLUDED_DIRS` names `.constellation` but is referenced only by the importers and `canonical.rs`,
never by the watcher.

**Change:** `is_app_bookkeeping_path` — one component-wise predicate, placed **FIRST** in the
`watcher.rs` filter chain because it is the only filter that can reject the two shapes the existing
checks deliberately PASS: a bare-directory event (`m.is_dir()` → pass) and any vanished path
(`Err(_)` → pass). Both are produced by the ledger's own writes (an append reports a bare-directory
event non-deterministically; a temp+replace or rename-aside reports both). Left unfiltered each costs
a full `refreshLibraryTree` re-walk + `loadAllStats`, and a vanished non-`.md` path additionally
drives `delete_rows_under_prefix` → the writer lock plus a lowercase scan of all 7,817 `note_meta`
paths to find zero victims. **Live today (D3)** via `cece/reliability.rs`'s tempfile persist —
so this slice fixes an existing stall, not just a future one.

**Scoped deliberately to that ONE segment, not all dot-dirs** — `.trash` carries real `note_meta`
rows (62 measured) and a restored note must still reach the indexer. Component-wise, never a
substring, so a user folder named `My .constellation notes` can never be swallowed.

**Belt-and-braces:** new `watcher_suppress::mark_with_parent` — `was_recent` is exact-path keyed
(`HashMap<PathBuf>`), so the bare-directory event is a SEPARATE key. Every temp+rename the ledger
performs must mark three keys (temp, final, containing dir); marking only the files leaves the
directory event unsuppressed. Named as a rule because the predicate could be refactored away.

**Tests:** 4 new (3 predicate + 1 suppression contract). Rust **1187/0** (+5) · vitest **607/607**
(52 files, excluding the two PJ-132 load-sensitive perf files, run as their own serial gate).

**Boss test:** pending — bundled with the next Boss-testable slice rather than shipping a binary for
a change whose visible effect is "the file tree stops blinking."

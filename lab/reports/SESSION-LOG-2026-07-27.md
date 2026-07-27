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

---

## Slice 2 — determinism + honesty (LANDED; must precede any archiving)

**Concept:** the history stream must record what the USER changed — not what a HashMap felt like
serializing.

**2a — the churn (D1), RED-PROVEN.** `search.rs` serialized `parse_frontmatter`'s `HashMap` directly.
Rust gives each `HashMap` instance its own hash keys, so identical frontmatter produced a
byte-different `properties_json` per call → the CECE trigger's
`OLD.properties_json IS NOT NEW.properties_json` guard fired → a fake "the user changed a property"
row. **Reproduced exactly: 6 re-indexes → 6 fake rows; with the sorted serializer → 0.** That is the
measured live pattern (~one row per app boot; 2,861 of 10,299 property events = 27.8% are this
artifact, 14.7% of all 19,481 history rows; the worst note pair holds 179 rows of which 175 are
no-ops). Fix: serialize through a `BTreeMap`. A genuine edit is still recorded — pinned by its own
test, because a determinism fix must not silence the stream it exists to clean.

**2b — the dot-segment guard (D4).** `reindex_changed_paths` Pass 1 was the ONE walker without the
`has_dot_segment` check that the index walk, the reconcile walk and the Move picker all have.
**Producer question settled by evidence, not assumption:** trashing a note does `gate_rename` into
`.trash` then `reindex_delete_note`, which PURGES the row — so a row at a `.trash` path can only have
been RE-CREATED afterwards, and Pass 1 is the only walker that would (the watcher deliberately still
passes `.trash` so a restore reaches the indexer; the file exists, ends in `.md`, and a library owns
it). `move_item` is excluded: it migrates existing rows and the Move picker excludes dot dirs. Live
consequence fixed: 62 `note_meta` rows at `.trash` paths, 543 history rows across 40 trashed notes,
two still ACCRUING history while sitting in the trash — deleted notes generating the very history the
archive is meant to seal. Guarded at the indexer, not the watcher, on purpose: a restored note moves
to a dot-free path and indexes normally. `has_dot_segment_pub` exported so there is ONE definition.

**2c — the second false comment.** `cece/history.rs` claimed unchanged fields are "absent from the
object (`json_object` natively skips NULL)". FALSE — `json_object` writes an explicit `null`; the
claim was contradicted by this file's own trigger doc ~40 lines below and by every live row.
Corrected, with the consequence stated for the archive + restore readers: **a key's presence is
meaningless; test the VALUE.** (My own false comment from Stage 0 was corrected in `95390d3a`.)

**Tests:** 4 new (byte-stable serialization · the RED-proven churn test · real-edit-still-recorded ·
the dot-segment predicate incl. the restore case). Rust **1191/0** (+4).

---

## Slice 3 — `link_life.rs`: the appender, the union reader, the contract (LANDED)

**Purely additive** — a new module writing files nothing reads yet, so it cannot regress anything.
~430 lines incl. 13 tests. Rust **1204/0** (+13).

**Why an append, stated in the module docs so it survives a future refactor:** every other candidate
design rewrote a file, and a rewriter holding a stale or empty in-memory map writes an empty store —
destroying exactly what it exists to protect. An append has no such surface. This is the mechanism
being *structurally incapable* of the failure, not disciplined against it.

**Two streams, one appender.** `earned.jsonl` (+ `earned.snapshot.jsonl`) folds — `n` by MAX,
decisions latest-wins, commutative and idempotent **by arithmetic**. `note-history.jsonl`
**NEVER folds and never compacts** — the record IS the payload. `read_folded` is the ONLY fold in
the module and reads Stream A only; it also *skips* any `nh`/`nd`/`nr` record defensively, so a
hand-concatenated file cannot leak history into the fold. `read_history_for` orders by `hid`
(the source ordinal) — never by `at`, which collides across 765 groups / 1,536 live rows.

**The corrupt-store contract, implemented and tested:** absent = a FACT (an empty store, never an
error); ONE unparseable line costs one line and is COUNTED (`LoadReport.skipped_lines`), with good
lines loading on both sides of it; a structurally unusable store is renamed **aside** — never
deleted — and sets `refuse_write`, because a blind overwrite destroys the backup that was about to
save the user.

**`.gitignore` (Boss decision #8), by NAME never by folder** — excluding `.constellation/` wholesale
to skip the databases would exclude the earned data living inside it, in the same event it exists to
survive. `*.db` rather than `search.db*` is what also catches the orphaned 939 MB
`Constellation SV Test.db` (PJ-159); the test asserts all 8 machine files are excluded and that no
ledger or config file is. `ensure_gitignore` never overwrites a user edit.

**fsync is per-site, not uniform** — the Slice-0 measurement (3,418 µs vs 168 µs) is quoted in the
module docs beside the `fsync` function, so the next reader knows why `append` deliberately does not
call it.

Also: `adopt_conflict_copies` folds Syncthing `.sync-conflict-*` copies back in then removes them
(nearly free — the fold is already commutative), and `store_dir(conn)` derives the location from
`conn.path()`'s parent, which is why no writer needs a path threaded to it.

---

## Slice 4 — the 6 link-life write hooks (BUILT, binary @12:40, awaiting Boss test)

**The load-bearing part was the lock boundary.** `constellation_link_traverse` held
`state.db.lock()` to the end of its body. Every DB touch is now inside an explicit scope whose
guard DROPS before the ledger append — never hold the DB lock across filesystem I/O, which is the
canonical freeze shape (PJ-066). The append costs 168 µs with **no lock held at all**.

**Two write orders, each stated as a rule rather than a habit at four call sites:**
- **Walks — DB first, then a plain append, no fsync, failure logged and swallowed.** A walk count
  is cheap to re-earn and feeds a logarithm; losing one must never fail the navigation the user
  asked for. 168 µs is what makes Boss decision #1 (no coalescing) affordable.
- **Decisions (retire / restore / trust / priority) — FILE FIRST + fsync, THEN the DB, and the
  error PROPAGATES.** New shared `record_decision`. Retiring is archival, not deletion — the
  wikilink deliberately stays in the note — so the DB is the only record and a rebuild from the
  notes would resurrect it. If the record cannot be made durable, the change must not happen.
  fsync is 3,418 µs, invisible on an action taken a few times a day.

**Hooked:** traverse (`walk`) · set_confidence (`trust`) · archive (`retire`) · unarchive
(`restore`) · set_review_priority (`priority`). The unarchive hook lives at the COMMAND, not in
`unarchive_link_rows`, because that helper borrows a connection from a lock-holding caller — writing
inside it would put file I/O under the lock. One production caller, so nothing is missed.

**The auto-tier is never recorded.** A confidence that is merely derivable from the count
(≥10 established, ≥3 evidence) carries no user judgment; recording it would fill the ledger with
decisions nobody made. `is_derivable_tier` + its test.

**A test caught a false claim of mine.** I wrote that `serde_json::json!` preserves insertion order.
It does not — without the `preserve_order` feature its map is a BTreeMap and it SORTS keys, so the
lines came out as `at,cid,n,t,tn,to,v`. The file is meant to be read by a human in a text editor,
where `v,t,cid,to,tn,n,at` reads as a sentence. Enabling `preserve_order` globally would change
every JSON write in the app, so the lines are now built by an explicit ordered writer (values still
escaped by serde, so the output is always valid JSON). The comment is corrected in place.

**Tests:** 9 new (30 in the module). Rust **1212/0**. Binary rebuilt 12:40.

**Environment note:** `cargo test`/`build` intermittently hit `LNK1104` — a transient Windows lock
on a freshly-linked test exe (no process holds it; a retry succeeds). Not a code fault; the C7 agent
hit the same. Builds are now run with a small retry loop.

### Slice 4 — Boss test PASS (2026-07-27, binary @12:40), verified in the DB not the log

Boss's `earned.jsonl` after 3 link clicks + Archive link + Contested — **exactly the 5 predicted
lines**, correct field order, correct shapes:
```
walk   Africa -> Madagascar  n=1  09:13:20
walk   Africa -> Earth       n=1  09:13:51
walk   Africa -> France      n=1  09:14:03
retire Africa -> France           09:20:51
trust  Africa -> Earth  conf=contested  09:21:25
```
**DB cross-check (read-only):** `france` → `status='archived'`, `weight=0.0`; `earth` →
`confidence='contested'`; all three `traversal_count=1` with `last_traversed` matching the ledger
timestamps to the second; weight 1.693 = `1+ln(2)`, the earned formula for one walk. **File-first
ordering held on every decision** — the line exists AND the DB changed, in that order. No `walk`
line for the auto-tier promotions, as designed.

**One defect the Boss found by reading the file (fixed in-pass, WA#6).** The same link was recorded
as `"France"` from a walk and `"france"` from a decision: walks take the target name from the EDITOR
(the wikilink text as typed), decisions from the PANEL (which passes `note_links.target_name`, stored
lowercased). Harmless to the fold — the key is the cid pair — but this file exists to be READ by the
user, and one link under two spellings makes a record harder to trust. `ledger_ids` now resolves the
target's identity AND its display `name` in one hop, and all four writers use it; the clicked text
remains the fallback for an unresolved target, which is the same case the name-key fallback covers.
Rust **1212/0** after the change.

**Filed:** `note_links.target_name` is stored lowercased, which is also why the Outgoing/Backlinks
panels display link targets in lowercase (`earth`, `france`) rather than the note's real title — a
pre-existing display-fidelity issue in a knowledge app, now visible. → **PJ-170**.

---

## Slice 5 — the back-fill: seed the ledger from the index (BUILT, binary @13:47, awaiting Boss test)

**This is the slice where the already-earned data stops being single-copy.** New
`link_life_backfill.rs` (~380 lines incl. 8 tests), cloned from the proven `link_boot_index` shape:
background thread after paint, own connection, 30 s busy timeout, `schema_versions` stamp written
LAST and only on success, failure non-fatal and logged. Rust **1220/0** (+8).

**The ONE earned predicate**, as a named constant with its two prohibitions written into the doc
comment so nobody "improves" it: `traversal_count > 0 OR status <> 'active' OR (confidence NOT IN
('hypothesis','structural'))`. **Forbidden:** `weight <> 1.0` (236 live rows carry values the earned
curve cannot produce — 115 at 0.526, 119 at 0.564, all with `traversal_count = 0`; decay residue,
and seeding them would put 236 junk records in the durable store) and `last_traversed <> ''`
(non-empty on ALL 234,233 rows — it identifies nothing).

**Structural rows are filtered in the LOOP, not the WHERE clause** — a structural edge can carry
`traversal_count > 0`, so the predicate alone cannot exclude it. Counted, not silently dropped.

**Records nothing it cannot key, and says how many it skipped.** A row whose SOURCE note has no
identity can never be restored to anything; writing it would be theatre and dropping it silently
would be dishonest. The diagnostics line states `recorded / matched / skipped-no-identity /
skipped-structural`.

**Two things deliberately NOT done, both verified:** no force-stamping of missing identities
(`ensure_cid_cn` WRITES the note file, and the cid-less notes are templates and `.trash` copies —
stamping a template changes what every future note spawned from it emits; zero live content notes
lack an identity), and no recording of the 236 off-curve weights (`weight` is DERIVED from `n` on
restore, which heals all 236 for free).

**Idempotence is arithmetic, not the stamp** — its own test runs the seed three times without
consulting the stamp and asserts an identical fold. Stated in the module docs as the right way
round: *a correctness guard that can be lost with a restored database is not a guard.* A companion
test proves a newer walk already in the store is never ratcheted down by a seed from an older DB.

One test assertion of mine was too crude and I corrected it rather than the code: it asserted the
`.gitignore` text does not contain "earned", but "earned" appears legitimately in the file's
explanatory comment (which is the point — it tells a reader why the folder must not be excluded
wholesale). It now asserts the SEMANTIC via `gitignore_excludes()` per ledger file.

### Slice 5 — Boss test found TWO defects in the seed; both fixed (SCHEMA_VERSION → 2)

The Boss read his seeded `earned.jsonl` and it held **44 lines**. Verified against the live index:
the truth is **38 earned rows**. Both defects were mine.

**(a) The target join fanned out on duplicate note names.** `LEFT JOIN note_meta tgt ON
LOWER(tgt.name) = LOWER(l.target_name)` emits one row per same-named note. Measured live: **3** notes
named `السعودية`, **2** `فلسفة`, **2** `banana`, **2** `collision test` → 38 earned links became 44
records, and **6 asserted links the user never walked**, each naming a different target identity. On
restore those could hand an earned count to the wrong link. His Arabic libraries are what exposed it —
`السعودية`/`فلسفة` naturally recur across libraries; a single-language corpus would have sailed past.
**Fix: a correlated subquery that resolves the identity ONLY when exactly one indexed note carries
the name, and otherwise refuses to guess** — yielding `''` so the record keys on the target NAME,
which is precisely what the fold's name-key fallback exists for. One output row per link row, always.

**(b) Seeded decision timestamps were derived but presented as observed.** The index has no "when was
this archived/judged" column, so a seeded `trust`/`retire` borrows `last_traversed`. His data made the
gap visible: a Contested click at **09:21:25** was seeded as **09:13:51** — the walk's time. The
timestamp cannot be made true, so every seeded line is now marked **`"seed":1`** (new
`link_life::mark_seeded`): a reader — human or restore — can tell a witnessed decision from a
reconstructed one, and a future re-seed from real activity. Additive, still valid JSON, does not
disturb the fold.

**Also corrected: the Plan's "33 recordable" is now 38** — not drift, but the Boss's own Slice-4 test
adding real earned data (3 walks + an archive + a Contested). The predicate is unchanged.

`SCHEMA_VERSION` → **2** to force the corrected pass. Documented on the constant: re-running is safe
by arithmetic, but the 6 spurious v1 records key on identities the corrected pass never writes, so
they cannot be folded away — the v1 file must be **deleted** before the re-seed, not merged with it.

3 new tests (41 MIG-104 total). Rust **1223/0**. Binary @14:24.

### Slice 5 re-test — PASS, verified against the DB (2026-07-27, binary @14:24)

Boss's corrected `earned.jsonl`: **38 lines = exactly the 38 earned rows** in `note_links`
(36 `walk` + 1 `trust` + 1 `retire`, from 36 recorded links — the 2 `structural` rows correctly
skipped in the loop). **All 38 carry `"seed":1`.** **5 records refuse to guess** (`"to":""`):
`فلسفة`, `السعودية`, `collision test`, `banana` ×2 — the fan-out is gone (`السعودية` 3 records → 1,
`فلسفة` 2 → 1). Folds to **34 distinct links**; the two `The Four Books` and two `banana` lines
collapse, which is Q2's type-free key working as ruled. **Zero spurious identities.**

### ★ A LIMITATION THE RE-TEST EXPOSED — a hard constraint on Slice 6 (the restore)

The two `banana` records are **two genuinely different links** (one source note → two different notes
both named "Banana"). Neither target is identifiable, so both key on the NAME and therefore fold to
ONE entry. Harmless in the live data — both carry `n=1`, so restoring 1 to both is correct — but if
their counts differed (5 and 2), the max-fold would restore **5 to both**, handing one link three
walks it never earned.

The fix does not belong in the seed (refusing to guess is right). It is a **rule for Slice 6**:
> **A NAME-KEYED record (`to` empty) may be restored ONLY when it resolves to exactly ONE
> `note_links` row. If several rows match, SKIP it and report the skip — never distribute one
> folded count across links that may have earned different amounts.**
> An identity-keyed record (`to` non-empty) is unambiguous and restores normally.

Recorded here and in `tests/mig104/README.md` so Slice 6 cannot quietly get it wrong; it needs its
own RED-provable recipe (two same-named targets with DIFFERENT counts → the restore must skip, not
average or max).

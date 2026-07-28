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

---

## Slice 6 — the restore (BUILT, binary @15:58, awaiting the HEADLINE Boss test)

New `link_life_restore.rs` (~460 lines incl. 9 tests). Scheduled after the seed, after paint, own
connection, stamped, failure non-fatal. Rust **1232/0** (+9; 50 MIG-104 tests total).

**`weight` is DERIVED, never restored** — `1 + ln(1 + n)`. A stored weight is a cache of an
arithmetic function of `n`, and 236 live rows carry values that function cannot produce. Recomputing
heals all 236 for free and stops `index_note`'s live `weight != 1.0` clause treating them as earned
forever.

**Batched (50/txn)** because each `note_links` UPDATE fires `note_links_sky_au` (DELETE + INSERT over
234k `sky_links` rows) plus the outgoing-aggregate pair's two `note_meta` UPDATEs.

**The Boss's `banana` constraint is implemented and RED-provable:** a name-keyed record that matches
several rows is SKIPPED and counted, never distributed. Its test gives the two same-named links
DIFFERENT counts (5 and 1) and asserts **both stay 0** — proving the rule structurally rather than
relying on today's data where both happen to be 1.

**A gap my own test caught, and the fix is a design point not a patch.** `db_loss_round_trip` failed
first time: a link with 7 walks restored its count but came back `confidence = 'hypothesis'`. Cause —
`evidence` at n≥3 is the DERIVABLE tier, which the seed deliberately never records (it carries no
user judgment). So the restore had nothing to write and preserved the rebuilt row's stale value. Fix:
the restore now **derives the tier from `n`, exactly as it derives `weight`** — a recorded tier is a
user judgment and wins; otherwise the tier is computed. New `link_life::auto_tier` +
`conf_rank` ensure a restore can never DOWNGRADE a user's judgment (`contested` outranks everything,
mirroring traverse's CASE WHEN preservation). This makes the symmetry explicit: **the ledger stores
only what cannot be derived.**

Also covered: a newer DB count is never ratcheted down by an older ledger; an unreadable store writes
NOTHING (`refuse_write` honoured — that is precisely how a restore destroys what it was protecting);
an identity-keyed record survives a TARGET rename, because the identity is the durable half of the key
and the name is not.

### ★ Slice 6 Boss test FAILED — and it found the exact bug the migration exists to prevent

The Boss renamed `search.db` aside and rebooted. The status bar sat at "68 notes"; four minutes later
the index was at 221 notes / 7,431 links — **progressing, not stuck** (a full rebuild of 7,817 notes is
far slower than loading an existing index; the Slice-0 baseline of ~35 s is a LOAD figure). He closed
the app mid-rebuild. His `earned.jsonl` (38 lines) and the 2 GB `search.db.MOVED.db` are both intact —
nothing lost.

**The bug, from his own diagnostics line:**
```
[link_life_restore] earned layer restored: 0 of 34 records written
                    (34 no longer in the index) — stamped
```
The restore raced the initial indexing, found that none of the 34 records had a link to attach to yet,
read that as "the links are gone" — and **stamped itself complete.** It would never have run again and
the earned data would never have come back. **The precise scenario MIG-104 exists for, defeated by the
wrong gate.**

**The error was conceptual, not incidental: I gave a RECONCILER a MIGRATION's stamp.** The restore
writes only where the DB disagrees, reports `already_current` otherwise, and is bounded by earned-link
count (34, not 234,233). Its correct shape is to run on EVERY boot — the same discipline
`reconcile::maybe_schedule` already follows for index-vs-disk drift, and for the same reason: the
condition it repairs can recur at any time. A one-shot stamp is only ever right for a pass whose work
cannot recur.

**Two fixes:**
1. **The restore is now unstamped** — `SCHEMA_VERSION` / `is_stamped` deleted, runs every boot, silent
   in the steady state (no log line when nothing differs or no ledger exists).
2. **Neither pass may conclude anything from an unpopulated index.** New `index_not_ready` guard: if
   `note_links` holds **zero** rows the restore reports NOT-READY and writes nothing, rather than
   counting every record as vanished. The seed gets the same guard on its stamp — *"I found nothing"
   and "there is nothing" are different claims, and only the second may be recorded as done.*

**Tests:** the new `an_index_still_rebuilding_concludes_nothing_and_writes_nothing` pins both halves —
an empty index reports NOT-READY (never "gone"), and a LATER pass restores everything, which is only
possible because the pass is unstamped. One existing test was corrected rather than the code: it
deleted ALL links to simulate one vanished link, which is now (correctly) indistinguishable from a
rebuild in progress; it now deletes one link and keeps the index healthy.

Rust **1233/0** (51 MIG-104). Binary **@17:03**.

### Slice 6 small-test — the restore reported "0 of 34 written" and I could not say WHY (instrumentation shipped)

The small test worked as a test: I snapshotted all 38 earned rows (`EARNED-SNAPSHOT.json`, my
reversible net) and wiped the earned layer only — 0 earned rows left in the DB, `earned.jsonl` intact
as the sole remaining copy. The Boss booted; the badges did not come back. **The DB confirms nothing
was restored.**

**The log line, and the tell:**
```
earned layer restored: 0 of 34 records written
  (0 already current, 0 no longer in the index, 1 ambiguous-skipped), 0 weights healed
```
34 records, and only **1** is accounted for. **33 vanished from the tally** — they were planned as
writes and the batch FAILED. And the failure went to `eprintln!`, which Windows GUI release builds
send nowhere (documented in-code at `search.rs:884-887`). **Third occurrence of this exact class in
two days** (reconcile's discarded error; the two false-success lines).

**Established, not theorized:** the SQL is NOT at fault. Ran the exact UPDATE from Python against the
live DB with `PRAGMA foreign_keys = ON` (matching rusqlite's default) — one row, then **all 38 in a
single transaction, rolled back: 0 failures.** Triggers fire in that path too. So the cause is
environmental (a concurrent writer during boot), not the statement — and it cannot be reproduced
outside the running app. **Per Reproduce-First, the only shippable work is the instrumentation.**

**Shipped:**
1. **The batch error is now logged** (row id + n/conf/status/weight/at) instead of going to stderr.
2. **New `planned` counter.** "0 written" was ambiguous between *nothing needed doing* and
   *everything failed* — precisely the ambiguity that cost this cycle. The line now reads
   `N of M PLANNED writes applied`, and appends **"ALL BATCHES FAILED, see the row error above"**
   when `planned > 0 && restored == 0`.
3. **A real omission found by re-reading the pattern I cloned:** `link_boot_index` sets
   `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;` on its dedicated connection and I left that
   out of BOTH `link_life_backfill` and `link_life_restore`. A dedicated connection that does not
   declare the pragmas is not the pattern these files claim to follow. Restored in both. Whether it
   is the cause is unknown — it is a genuine deviation either way.

Rust **1233/0**. Binary **@17:30**. The Boss's wiped state is preserved for the next boot, so the
instrumented run reproduces the failure exactly.

### ROOT CAUSE FOUND AND FIXED — `no such tokenizer: constellation` (LL-036)

The instrumented boot named it in one line:

    row 569079 UPDATE FAILED: no such tokenizer: constellation — (n=1, conf=Some("hypothesis"), ...)

**Verified mechanism (trigger chain read from the live schema, not assumed):** `UPDATE note_links`
fires `note_links_outgoing_au` → UPDATEs `note_meta` → fires `note_meta_au` → **writes `notes_fts`**,
an FTS5 table declared `tokenize='constellation'`. That tokenizer is registered **per connection**
inside `init_db`. `run()` opened a bare `Connection::open`, so every row failed and all 33 planned
writes were lost.

**The mistake:** `link_boot_index` — the module I cloned — documents the precondition that made ITS
bare connection sufficient, two lines above the code I copied: *"CREATE INDEX is pure DDL — it fires
NO row triggers — so no FTS tokenizer registration is needed."* I copied the setup and not the
condition, then wrote a module that violates it on its first statement. → **LL-036: when cloning a
proven pattern, clone its PRECONDITIONS; the comment explaining why it is safe is part of the code.**

**Why 52 tests missed it — the sharper half.** Every test in the module builds its DB with `init_db`,
**which registers the tokenizer**, so the fixture was *more capable than production*. Green suite,
100% write loss live. The new regression test reproduces PRODUCTION's shape (raw `Connection::open`,
no tokenizer), asserts the precondition is real (the write fails with a tokenizer error), then asserts
registration makes the restore land — so deleting the call goes red.

**Fixed:** `register_fts5_tokenizer(&mut conn)` in `link_life_restore::run`. `link_life_backfill` gets
a documented precondition instead of a cargo-culted call (it only reads `note_links` and writes
trigger-free `schema_versions`) **plus an explicit warning that any writer added later must register
one.** Also found in the same pass: both modules had omitted `journal_mode=WAL` /
`synchronous=NORMAL` — restored.

Rust **1234/0** (52 MIG-104). Binary **@18:11**. The Boss's wiped state is still in place, so the next
boot is the real proof.

### ★★ SLICE 6 BOSS TEST — PASS. The earned layer survived being destroyed. (binary @18:11)

Method (the small test): I snapshotted all 38 earned rows to `EARNED-SNAPSHOT.json`, then wiped the
earned layer ONLY — counts to 0, confidence to `hypothesis`, retired links back to active, weights to
1.0. Verified 0 earned rows left in the DB, `earned.jsonl` intact as the sole surviving copy. The Boss
booted the fixed binary.

```
[link_life_restore] earned layer restored: 34 of 34 records written
                    (0 already current, 0 no longer in the index, 1 ambiguous-skipped)
```

**Independent verification against the snapshot — every one of the 38 accounted for, none unexplained:**

| Outcome | Count | Why |
|---|---|---|
| **Restored EXACTLY** (n, confidence, status, weight) | **34** | the promise of the migration |
| of which had an arithmetically-impossible weight, healed | 1 | `weight` is derived from `n`, never trusted |
| Correctly NEVER recorded | 2 | `james williamson (film pioneer)`, `target note_v1` — **source note absent from the index**, so the record is unkeyable and can never be restored to anything. The seed skipped and counted them by design; writing them would have been theatre. |
| Correctly SKIPPED by the ambiguity rule | 2 | the `banana` pair — 2 notes share the name, 2 sibling link rows, ONE folded name-keyed record. The Boss-forced rule refuses to distribute one count across links that may have earned different amounts. |
| **MISMATCHED** | **0** | — |

Spot-checks in the app: `Earth ×1 + Contested`, `Madagascar ×1`, and **`France` id=408533 → n=1,
status=`archived`** — a retirement decision surviving the destruction of the layer that held it, while
the wikilink stayed in the note. That is the case a rebuild-from-notes could never recover, and the
reason MIG-104 exists.

**A verification error of mine, corrected for the record:** my first spot-check query used
`source_path LIKE '%Africa.md'`, which also matches *East Africa.md* and *West Africa.md*, and
`fetchone()` returned the wrong row — making France look unrestored. The app was right; the query was
wrong. Checked by id thereafter.

**MIG-104's core promise is now proven on live data: losing the index costs the user nothing they
created.**

---

# MIG-104 Slice 7 — the snapshot + the 2 MB compactor

**Function in hand:** the Earned-Life Ledger's **snapshot + compactor**.
**Concept (the horse):** *the cost of reading the ledger must be bounded by how much you have
EARNED, not by how long you have been using Constellation.*

An append-only store has exactly one weakness — it only grows, and a store that is read in full on
every boot forever eventually costs the boot. This slice removes that weakness without giving up
the property that made append-only the right choice: **nothing is ever rewritten in place, and
nothing is ever deleted.**

## What landed

`earned.snapshot.jsonl` — one line per earned link (plus one per note-level decision), carrying the
current folded state. Compaction fires on a **byte threshold (2 MB) and never on a timer**, so an
idle Universe produces zero writes.

**The order is the safety argument, and it is the reverse of the intuitive one:**

1. build every line from the folded state (pure, in memory)
2. write to a **unique temp** in the same directory (PJ-087 — never a fixed `<name>.tmp`)
3. **fsync the temp**
4. persist it over `earned.snapshot.jsonl` (atomic rename within the volume)
5. rename `earned.jsonl` aside to `earned.tail-<UTC>.jsonl` — **never delete** (invariant #4)

A crash anywhere before step 5 leaves snapshot + full tail, which the fold reads as the state it
already had — duplicated input, identical answer. There is no window in which the state lives only
in a file that is not read. Step 3's fsync is mandatory *here* (unlike the walk path's) precisely
because step 5 makes the snapshot the only file in the load path that holds the folded history.

`note-history.jsonl` is structurally excluded: **`maybe_compact` has no stream parameter.** Folding
Stream B would collapse a thought into a keystroke (the live rows `hid` 8251/8252/8253 record one
property being typed, `ma` → `mas` → `masadir`). Making the stream a parameter would turn that
prohibition into a code-review convention; leaving it out makes it a fact about the signature.

## ★ Three defects the slice exposed — all fixed in-pass (WA#6)

Building a compactor means asking "can the loaded state re-express every record the writers emit?"
That question found three things a green 52-test suite did not:

**1. `priority` records were written since Slice 4 and never read back.**
`set_review_priority` appends **and fsyncs** a `priority` line before touching the DB. The fold's
key function required a target, so every one of those records was silently dropped, and the Plan's
Slice-6 clause *"restores review priority too"* was never true. **Losing `search.db` still cost the
user every review priority they had set** — in the one pass whose whole job is that it must not.
Now folded (`LedgerState.notes`), snapshotted, and restored, with `-1` mapping back to SQL `NULL`
and the same one-row-or-skip rule that governs an ambiguous link record. Had this not been caught,
compaction would have moved those records out of the loaded store permanently.

**2. The `refuse_write` guard could not fire.** `link_life_restore`'s *"do NOT write a thing from a
store we could not read"* read as a live protection while being **structurally unable to trigger**:
the flag was set only inside `link_life::quarantine`, which returns its *own* report, while every
reader built a fresh one in which it was `false`. This is the **LL-035 shape** — a claim that a
protection is active is a *runtime* claim. The reader now OBSERVES the quarantine on disk (an
`earned.corrupt-*.jsonl` beside the store = un-acknowledged; acknowledging is the user moving the
file away — File-Over-App, no hidden flag to drift). Both the restore's guard and the compactor's
are now real, and a test puts the store in the state that fires them.

**3. A false log line.** The restore's diagnostics ended `— stamped` while the pass is deliberately
**unstamped** (that stamp was the Slice-6b bug). LL-035 rule 2: a log line must be evidence, never
intent. Removed. Also defused the related trap: the Boss's live DB still carries a
`schema_versions` row `link_life_restore = 1` from the first cut, and the module now says in-code
that it must never be consulted.

Also removed two unused imports in `link_life_restore.rs` (pre-existing).

## Correctness detail worth keeping

An absent timestamp now round-trips as **absent**, not as `""`. The snapshot must write the `at`
field regardless (field order is part of the format), so `""` on the way out would have folded back
to `Some("")` on the way in — making the state after a compaction differ from the state before.
Nothing user-visible would have broken, which is exactly why it would have gone unnoticed.

## Verification

| Gate | Result |
|---|---|
| Rust `cargo test --lib` | **1258 passed / 0 failed** (11 ignored) — was 1234 |
| `svelte-check` | **0 errors** (268 warnings, pre-existing) |
| vitest main lane | **52 files / 607 tests passed** |
| vitest **Sight perf, SERIAL lane** (PJ-172) | **2 files / 31 tests passed** (`--no-file-parallelism --maxWorkers=1`) |
| Slice 7 modules | `tests_mig104_compact` **19/19**, restore additions **5/5** |

**Live store before/after (Rule 8):** `earned.jsonl` **6,222 B → 6,222 B, unchanged**; no snapshot
written. The threshold is 2 MB, so this Universe is ~340× below it and the pass costs one
`metadata()` call per boot. That is the intended steady state, and the reason the Plan marks this
slice not-Boss-testable — the *compactor* has no user-visible surface until a Universe has recorded
on the order of 10,000 decisions. **The priority restore added in-pass DOES have one, and is
Boss-tested below.**

## Still open, stated rather than parked

`link_life::quarantine` has **no production caller** — nothing yet decides that a store is
"structurally unusable". Its *effect* is now observable and both guards honour it, but the detector
itself is a Slice-3 gap. The compactor's own "the fold is empty while the tail is not" refusal
covers the shape that actually occurs (a truncated or garbage store), so there is no silent-loss
exposure today. Filed for a Boss ruling rather than silently parked (WA#6).

## ★★ The safety inspection found a real app-killer IN THIS SLICE'S OWN NEW CODE — fixed before commit

The per-build inspection (standing order) confirmed, via **three independent verifier agents**, an
unguarded **TOCTOU in `maybe_compact`** — code written an hour earlier in this same slice.

**The mechanism.** `maybe_compact` folded the tail at one moment and renamed it aside at another,
and between those two moments it wrote and fsync'd a multi-megabyte snapshot — tens of
milliseconds. `append` took **no lock of any kind**. Every record appended inside that window was
renamed into `earned.tail-<stamp>.jsonl`, **which nothing ever reads back** (that is precisely what
bounds the load). On Windows the rename even succeeds while an append handle is open
(`FILE_SHARE_DELETE`), so the handle keeps writing into the aside file.

**The appenders are provably concurrent.** `constellation_link_traverse`, `record_decision`
(retire / restore / trust) and `set_review_priority` all append from Tauri command threads **after
deliberately dropping the DB guard** — at any moment of an interactive session, while the boot
thread compacts.

**Why it is worse than a missing line — and why it is exactly the class MIG-104 exists to prevent.**
The restore treats the ledger as authoritative for DECISIONS (confidence, retired/active, review
priority). A decision lost this way is not merely absent: on the next boot the fold still carries
the *pre-decision* value, disagrees with the DB, and **writes the old value back** — silently
un-retiring a link or reverting a priority the user set, with every step logging success. Walk
counts self-heal (absolute `n`, max-fold), so the permanent loss lands precisely on the data this
migration was built to make durable.

**My own error, named.** The comment I had written in `link_life_restore` said compaction riding
the restore's thread made the race *"impossible instead of unlikely."* That was a **sequencing**
argument mistaken for an **exclusion** argument: it covered restore-vs-compact and said nothing
about the live appenders, who are unaffected by which thread compaction runs on. The comment is
corrected in-tree to say so explicitly, because the next person to read it would have inherited the
same false confidence.

**The fix.** A module-level `FILE_LOCK` in `link_life.rs` serializing every operation that MUTATES
the store — `append`, `fsync`, `quarantine`, and the whole read→write→fsync→rename sequence of
`maybe_compact`. Deliberate details: the threshold probe stays **outside** the lock (one
`metadata()` call, run on every boot of every Universe, must never contend); the size is re-read
**inside** it (another pass may have already handled the tail); reads stay lock-free (the
write-snapshot-first ordering already makes every intermediate state fold correctly); and a
poisoned lock uses `into_inner()` rather than `unwrap()`, because a panic elsewhere must never stop
the ledger from writing the user's earned data. This is a dedicated FILE lock, not the DB lock —
PJ-066's "never hold the DB lock across file I/O" is untouched, and no call site takes the two in
conflicting orders.

**Reproduce-First, and a second lesson inside it.** The first regression test I wrote **passed
without the fix** — a fixed-count appender that finished before compaction reached the window. A
regression test that cannot see the regression is worse than none, so it was rebuilt twice:

1. the appender now runs **until compaction returns** (atomic stop flag), so its writes are
   guaranteed to span the window rather than race it;
2. the fixture was changed from 700 links × 24 rounds to **20,000 distinct links** — the old one
   folded to a 700-line snapshot that was too fast to write to expose anything. *The window being
   tested is the snapshot write, so the fixture has to make that write slow.*

**RED proven across 3 consecutive runs with the exclusion removed: 666 of 730 and 1,110 of 1,168
decisions silently lost per run.** GREEN across 3 consecutive runs with it restored.

A third test — `a_stale_fold_strands_a_decision_in_the_aside_tail` — performs the interleaving **by
hand, with no threads**, so the mechanism is pinned deterministically for whoever next touches this
code, independent of timing.

## Also confirmed in MIG-104's territory — NOT fixed here, flagged for a ruling (WA#6)

`ConfidencePicker.svelte:61,70` — the **only** user entry to the trust/retire decisions — wraps
both calls in `catch { /* ignore */ }`. Slice 4's Rust contract deliberately **fails closed** ("if
the record cannot be made durable, the DB change must NOT happen; the error propagates and the user
is told"), and the popover throws that error away. The DB correctly stays unchanged and the UI
correctly does not update — so the user clicks *Contested*, the menu closes, and **nothing happens,
with no explanation.**

Not fixed in this commit deliberately: it needs an error surface (there is no toast system; the
in-repo pattern is an inline message like `conflict.mergeSaveError`) plus a new key across **15
locales**, and *where* the message appears is a design decision. The Plan already has **Slice 14 —
adjacent defects** for exactly this class. Filed as **PJ-173**; recommend folding into Slice 14.

## PJ-166 struck a FOURTH time

The inspection was invoked diff-scoped (`args.files = [link_life.rs, link_life_restore.rs]`) and
returned `mode: "whole-app"` — `args.files` ignored again. 83 agents, 0 errors, ~10.1 M tokens,
31 minutes. **The upside was real** (the whole-app pass is what surfaced the TOCTOU at all), but the
per-build gate the standing order asks for still does not exist, and a 31-minute whole-app sweep
cannot run per build. The sweep also produced a large whole-app confirmed register — including an
**APP-KILLER at `+layout.svelte:6779`** (the rename cascade's protection sets are pre-walk snapshots,
so a tab opened DURING the multi-second walk is never frozen, never flushed, yet still force-adopted)
and a **silent-data-loss at `PropertyEditor.svelte:852`**. Those are **per-cycle** findings, not
this build's — appended to the Charter register and filed in the PJ ledger, NOT absorbed silently
into this slice.

## ★ BOSS TEST — PASS (2026-07-27)

The compactor itself has no user-visible surface yet (6 KB store, ~340× below the 2 MB threshold),
so the Boss test targeted the clause this slice discovered was missing: **a review priority
surviving the loss of the index.**

1. Boss set **Priority = 80** on `Africa` via the right-sidebar Review Pulse panel, and confirmed
   the record had landed in `earned.jsonl`:
   `{"v":1,"t":"priority","cid":"20260414T092241Z_NOTE_7AB7","p":80,"at":"2026-07-27T15:52:03…"}`
2. **I erased it from the index myself** (WA#1 — not handed to the Boss):
   `UPDATE note_meta SET review_priority = NULL WHERE cid_cn = '20260414T092241Z_NOTE_7AB7'`.
   Verified `review_priority IS NOT NULL` went from **1 row to 0 across the whole index**, and that
   `review-pulse.json` carries no priority data and never mentions the note — so **the ledger line
   was the only copy of that number anywhere on disk.** (Keyed on `cid_cn`, never
   `LIKE '%Africa.md'` — that also matches *East Africa.md* / *West Africa.md*.)
3. Boss reopened: **Priority reads 80 with the `manual` tag, "Computed would be 30".** Diagnostics:

```
[link_life_restore] earned layer restored: 0 of 34 records written (34 already current,
0 no longer in the index, 1 ambiguous-skipped), 0 weights healed,
1 priorities restored (0 already current, 0 unresolved), 0 bad lines
```

Every number is the designed one: the link layer was already correct (`34 already current` — the
steady state, silent by design), the `banana` pair was **skipped by the ambiguity rule** rather than
handed a count it might not have earned, and **the priority came back from a plain-text file the
database had no memory of.**

### A false mechanism in my own comment, corrected by the surgery

Clearing the column required deciding whether a bare connection could do it (LL-036). Reading the
live triggers rather than assuming: **`note_meta_au` is guarded** — `WHEN OLD.name IS NOT NEW.name
OR OLD.body_text IS NOT NEW.body_text` — so a review-priority write never reaches `notes_fts`,
which the bare `sqlite3` UPDATE then confirmed by succeeding with no tokenizer registered. My
comment on the priority batch had claimed it fires `note_meta_au` and that this is *why* the
connection needs the tokenizer. **Wrong mechanism, right conclusion for the wrong loop** — the
tokenizer is required by the `note_links` writes, not this one. Corrected in-tree, with the
verification recorded, plus the fact that actually applies: `sight_v6_layout_invalidate_au` is
**unguarded** and drops the note's cached layout row on every `note_meta` update (harmless — derived
— but it is the real per-row cost that justifies the batching).

---

# ⟲ PIVOT — state-of-standing record (SO#5), 2026-07-27

**Boss ruling:** *"Divert now — fix the 3 app-killers."* The MIG-104 cascade is PAUSED at the
Slice 7/8 boundary. This record exists so the pause is resumable without rediscovery.

### (a) Verified-shipped & protected
- **MIG-104 Slices 0–7** — pushed, `a86bf3ca`, tree clean. Slice 6 Boss-validated on live data
  (34/34 restored, a retired link returned still retired); **Slice 7 Boss-validated today** (a review
  priority restored from the ledger after the index was wiped of it: `1 priorities restored`).
- **MIG-105 Stage 0** — nine live defects fixed; FK-enforcement reality pinned by test.
- Gates at the pause point: Rust **1261/0** · svelte-check **0** · vitest **52 files/607** ·
  Sight perf **31/31** in a serial lane. Binary `constellation.exe` @ 19:41 = the tested one.

### (b) At-risk / in-flight
**Nothing uncommitted.** The MIG-104 cascade resumes at **Slice 8 + 8b** with one hard constraint
already proven by test: the archive hook must go **BEFORE `DELETE FROM note_meta`**
(`search.rs:9845`) — FK enforcement fires the CASCADE there, so a hook at the later explicit purge
archives **nothing** (`tests_stage0_delete_order_defect`). 8b adds the note body (Boss decision #6).

### (c) Known-broken — THE PIVOT TARGET
Three confirmed APP-KILLERs, none introduced by Slice 7, all live in the build the Boss runs daily.
Full text: `lab/reports/SAFETY-INSPECTION-2026-07-27-whole-app.md`. Filed as PJ-174.

1. **`+layout.svelte:6779` — the rename cascade's protection sets are PRE-WALK SNAPSHOTS.**
   `tabsInLibrary` (:6779), the freeze set (:6756) and `flushAllTabsInLibrary`'s own snapshot (:6783)
   are all taken before a multi-second library walk, but `reloadTabsFromDisk` (:6811) force-reseeds
   whatever is in `openTabs` **at reload time**. The sidebar is not blocked during the walk, so a
   note opened and typed in mid-walk is never frozen, never flushed — and is still force-adopted
   from disk. Either the paragraph is erased wholesale, or an ungated autosave lands after the
   walker and re-introduces the old wikilink while the index records the rewritten text.
2. **`PropertyEditor.svelte:852` — a stale whole-props replay erases frontmatter another writer
   already persisted.** The panel's `properties` prop derives from `tab.content`, which the
   model-based writers (`saveTabContent` / `addTagToNote`) deliberately never update or notify.
3. **`PropertyEditor.svelte:851` — two PropertyEditor instances are mounted for the SAME tabId**
   (NotePane-embedded + right-sidebar), each with its own snapshot, and the save mutates
   `tab.content` directly ("no store.update = no cascade") so neither re-seeds from the other.

> **(2) and (3) are one root cause, not two bugs:** props have **no single owner in the UI layer**.
> Every PropertyEditor keeps its own `editableProps` derived from a stale projection, and every
> writer REPLACES `model.props` wholesale from its own snapshot. This is the **content-integrity
> class** whose three-strike law is already spent (LL-014; BUG-012 / 015 / 019 / 023), so
> **Solve-the-Class applies: the fix is single content ownership for PROPS — the model is the
> authority every reader and writer goes through — not another symptom patch.** MIG-076 did this
> for the note BODY and stopped there.

### (d) Pending, not started
MIG-104 Slices 8–15 · MIG-105 Phase 2 · MIG-106/PJ-169 · PJ-173 (folds into Slice 14) ·
the other 47 sweep findings (20 HIGH / 24 MED / 4 LOW) · PJ-171 · PJ-172.

### (e) Process
- **PJ-166, fourth strike** — `args.files` ignored again; the per-build diff gate the standing order
  requires still does not exist. A 31-minute, 10.1 M-token whole-app run cannot serve as one.
- Sight perf tests still need a **permanent** serial lane (PJ-172); run manually as one this job.

**Order of work from here:** Reproduce-First on all three — no fix is designed before its defect
fires on demand under instrumentation.

---

# AK-1 — the rename cascade's unprotected mid-walk tab (PJ-174 #1) — FIXED

**Boss ruling:** divert from the MIG-104 cascade and fix the three app-killers. This is the first.

## Reproduce-First — RED before any design

`tests/pj-174/renameCascadeMidWalkTab.test.ts`. The failing assertions were exactly the damage:

```
AssertionError: expected 'links to [[New]]' to contain 'my unsaved paragraph'
AssertionError: expected false to be true          // isDirty('b') after the reload
```

The paragraph was replaced by disk text **and** `isDirty` flipped to false — so after the loss the
app no longer knew there had ever been unsaved work. That is what made it silent: nothing to
retry, nothing to surface, no net (it was cleared in the same breath).

## Three holes, not one

**#1 — the protection sets were PRE-WALK SNAPSHOTS.** `cascadeFreeze` (:6756), `markCascading`
(:6779) and `flushAllTabsInLibrary` (:6783) are all built from `tabsInLibrary(lib.path)` *before* a
multi-second walk, while the sidebar stays clickable throughout. A note opened mid-walk was in none
of them. **A snapshot cannot be repaired by taking it later — there is no "later" that is after
every tab the user might open.** So the predicate stopped being a snapshot: `markCascadingLibrary`
makes `isCascading` answer *"is this path inside a library currently cascading?"*, which is true for
a tab that does not exist yet at mark time. `cascadeFreeze` now holds library ROOTS through a shared
`isPathFrozen`, collapsing the freeze and the gate into ONE concept with one boundary rule instead
of two representations that could disagree.

**#2 — `reloadTabsFromDisk` force-adopted over a dirty model.** Its own docstring says a dirty path
must never reach it and that *"the guard lives UPSTREAM at every caller"* — and upstream was the
stale snapshot. An invariant every caller must uphold, inside a function whose only job is
destructive re-seeding, is a promise waiting to be broken; it is now enforced where the damage
happens. A dirty model is refused and routed to the SAME `.conflict` sidecar + banner the watcher's
external-change path uses, so the user's edits stay live and the cascade's version is preserved.

> **The WA#4 consumer sweep earned its keep here.** One of the nine callers — `discardFailedSave`,
> the PJ-102c *"Discard my changes"* button — **depends** on force-adopting over a dirty model. A
> blanket refusal would have silently broken it. So destroying edits is now opt-in **by name**
> (`discardLocalEdits: true`), which exactly one call site passes. `linkMentionInNote` already
> carried a comment asserting this guard existed; it now does.

**#3 — the freeze overlay had the same snapshot hole**, so the pane most at risk (one being
rewritten under the cursor) was the one pane with no overlay. Fixed with the same library-root model.

## ★ The inspection found a FOURTH hole — in my own blind spot

The per-build inspection on this fix confirmed an **APP-KILLER at `store.ts:3879`**: `renameItem`
re-seeds the model from disk after `await invoke('rename_item')` and `await readNote()` with **no
dirty re-check**, having cleared the write-ahead net three lines earlier.

`markCascading` gates disk WRITES (`handleSave` / `handleFlush`) — it does **not** gate
`onDocChange → editBody`, so the model keeps accepting keystrokes; and the freeze overlay cannot
cover this window because the caller raises it only *after* `renameItem` returns. NotePane's Enter
handler focuses the **body**, so *"rename the title, press Enter, keep writing"* — an ordinary
gesture — lands the caret exactly where typing was unprotected. The text was then gone from the
model, the screen and the net, with the only tripwire being DEV-only and therefore invisible in the
release build the Boss runs.

**This is my miss, and it is the Whole-Ecosystem Fix Law's own failure shape committed while
applying it.** When I swept for other force-reseed surfaces I ran
`grep openNoteModel | grep -v libraries/store.ts` — excluding the file I was editing — and concluded
"`reloadTabsFromDisk` was the single primitive." Re-run without the exclusion, `store.ts` holds
seven call sites, and one of them was this. The sibling `drainCidEnsure` (:2841) already carries the
correct guard **with a comment explaining it**; `renameItem` never got it. Fixed by mirroring that
sibling, using the correct branch this function already had (`repathNoteModel` — keep the user's
model, move its identity), and by clearing the net **only for a tab that actually adopted disk**.

## And a fix that would have become a regression

`store.ts:1452` — HIGH — the cascade gate in `saveTabContent` returned **before** `editNoteProps`
pushed the edit into the model, so a property edited during a cascade was neither written nor kept.
The comment two lines below states the governing rule for the write-lock — *"the guard serializes
the WRITE, never the model update"* — and the cascade gate was breaking it.

**Making the gate LIVE widened that window from "tabs open at rename time" to "the whole library",
so shipping #1 without this would have made an existing silent loss fire more often.** The gate now
sits after the model push: the property lands in the model (dirty), only the write waits, and the
cascade's reload then refuses to adopt over it and raises a conflict. Preserved and surfaced instead
of dropped. The control test asserts the gate still blocks the disk write (F2 post-cascade-stomp is
still prevented).

## Verification

RED proven by removing each guard: #1 (paragraph → disk text, `isDirty` false), #1b (2 of 3 fail:
*expected 'first line' to contain 'the next sentence'*), #1c (the property never reaches the model).
Every one GREEN with the guard, and each carries a control test that the fix does **not** over-block
— a clean tab still adopts the cascade, a clean model still adopts the rename, Discard-my-changes
still discards, and the cascade gate still gates the write.

| Gate | Result |
|---|---|
| vitest | **54 files / 619 tests** (was 52/607) |
| Sight perf, **SERIAL lane** (PJ-172) | **2 files / 31** |
| svelte-check | **0 errors** |
| Rust | **1261 / 0** |

## PJ-166 — FIFTH strike

Invoked diff-scoped, returned `mode: "whole-app"` again (84 agents, ~10.3 M tokens, 32 min).
46 unique confirmed: **3 APP-KILLER · 10 HIGH · 27 MED · 6 LOW** →
`lab/reports/SAFETY-INSPECTION-2026-07-28-ak1-build.md`. Two in the same family are filed, not
absorbed: **`moveItem` (:3913) has neither `markCascading` nor a pre-move flush** — the exact
two-part gate its sibling `renameItem` carries — and **`deleteWithSetting` (:3986) never calls
`closeNoteModel`**, so a deleted note's surviving model can be re-created by the teardown flush.

## ★ BOSS TEST — AK-1 PASS, both stages (2026-07-28)

- **Stage 1 (#1b — the everyday gesture).** Rename via the title bar, press Enter, keep typing
  immediately. Boss: *"Stage 1 passed, sentence survived."* Before this build that sentence was
  destroyed as the rename completed — from the model, the screen and the recovery net.
- **Stage 2 (#1/#3 — the mid-walk freeze).** Renamed `Doi (identifier)` in Biology (535 of that
  library's 550 notes link to it — the longest possible walk) and opened a different Biology note
  during the cascade. Boss: *"Pass. I saw the 'Updating links…'. It was so fast, I just got a
  glimpse of it."* The glimpse is the point: that note previously had **no** overlay at all and was
  freely typeable while the walker rewrote it.

Committed after the pass, per the mandatory Boss-test gate.
